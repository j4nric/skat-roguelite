// What the info card says about a thing on screen (../ui/infocard.ts). An
// element names its subject with `data-info="<kind>:<id>"`; this file turns
// that key and the core's view into the card's content: name, kind, the
// effect text from the content, and what the view tells about it (Augen and
// flags of a card, counters of a relic, what fired in the last game).
// Only lookups and labels: every fact comes from the view or the catalog.

import { de, cardName, contentName, contentText, isCurse, relicCounters, t } from '../i18n';
import type { CardKind, CardView, ConsumableInfo, DeckCard, Rarity, RelicKind, RunView, ScoreLine, Seat, Source } from '../types';
import { cardSvg } from './cards';
import { attrs, esc } from './dom';
import { bossCoaster, consumableCoaster, relicCoaster } from './emblems';
import { seatPerson } from './guests';
import { oddsLine } from './odds';
import { bossPortraitId, portrait } from './people';
import { changeText } from './score';
import type { UiState } from './state';

/** A line of state under an effect text: on (it fired), off, or just a fact. */
export interface InfoStatus {
  text: string;
  on: boolean | null;
}

/** One effect on the card: a modification of a card, or the text of a relic. */
export interface InfoSection {
  name?: string;
  tag?: string;
  curse?: boolean;
  text: string;
  status: InfoStatus[];
}

export interface InfoModel {
  key: string;
  title: string;
  kind: string;
  /** Picture (HTML): the card, a portrait, an item. */
  art: string;
  /** Short facts from the view ("Zählt 11 Augen"). */
  facts: string[];
  sections: InfoSection[];
  /** More HTML under the sections (deal odds). */
  extra?: string;
  /** Buttons with `data-action` (buy, use), handled by the app. */
  actions?: string;
}

/** Where a key points: "card:12" → ["card", "12"]. */
export function splitKey(key: string): [string, string] {
  const i = key.indexOf(':');
  return i < 0 ? [key, ''] : [key.slice(0, i), key.slice(i + 1)];
}

/** Score lines of the last game from this source. */
function lastLines(view: RunView, match: (s: Exclude<Source, string>) => boolean): ScoreLine[] {
  return (view.last_score?.breakdown.lines ?? []).filter((l) => typeof l.source === 'object' && match(l.source));
}

function lastGameStatus(lines: ScoreLine[]): InfoStatus[] {
  return lines.map((l) => ({ text: t(de.info.lastGame, { effect: changeText(l.lever, l.change) }), on: true }));
}

/** A card of the game or the deck by id, as complete as the view has it. */
function findCard(view: RunView, ui: UiState, id: number): { kind: CardKind; mods: string[]; card: CardView | null; holder: Seat | null } | null {
  const game = view.game;
  const lists: CardView[][] = [];
  if (ui.anim) lists.push(ui.anim.hand, ui.anim.trick.map((p) => p.card));
  if (game) {
    lists.push(game.hand, game.trick.map((p) => p.card), game.last_trick?.cards.map((p) => p.card) ?? [], game.skat ?? [], game.skat_swap_cards);
  }
  for (const list of lists) {
    const c = list.find((x) => x.id === id);
    if (c) return { kind: c.kind, mods: c.mods, card: c, holder: null };
  }
  const seen = game?.revealed.find((r) => r.card.id === id);
  if (seen) return { kind: seen.card.kind, mods: seen.card.mods, card: seen.card, holder: seen.holder };
  const deck: DeckCard | undefined = view.deck.find((c) => c.id === id);
  return deck ? { kind: deck.kind, mods: deck.mods, card: null, holder: null } : null;
}

function cardModel(key: string, view: RunView, ui: UiState, id: number): InfoModel | null {
  const found = findCard(view, ui, id);
  if (!found) return null;
  const { kind, mods, card, holder } = found;
  const facts: string[] = [];
  if (card) {
    facts.push(t(card.card_points === 1 ? de.info.pointsOne : de.info.points, { n: card.card_points }));
    if (card.rank_bonus > 0) facts.push(t(card.rank_bonus === 1 ? de.info.rankBonus : de.info.rankBonusMany, { n: card.rank_bonus }));
    if (card.never_wins) facts.push(de.info.neverWins);
    if (card.honorary_jack) facts.push(de.card.honoraryJack);
    if (card.no_discard) facts.push(de.card.noDiscard);
  }
  if (holder !== null && holder !== 0) facts.push(t(de.info.holder, { name: seatPerson(view, holder).name }));
  const sections = mods.map((m): InfoSection => {
    const curse = isCurse(m);
    const lines = lastLines(view, (s) => 'CardMod' in s && s.CardMod.card_mod === m && s.CardMod.card === id);
    return {
      name: contentName('card_mods', m),
      tag: curse ? de.info.kinds.curse : de.info.kinds.cardMod,
      curse,
      text: contentText('card_mods', m) || de.info.unknownMod,
      status: lastGameStatus(lines),
    };
  });
  return {
    key,
    title: cardName(kind),
    kind: de.info.kinds.card,
    art: `<span class="card info-card-face">${cardSvg(kind, mods)}</span>`,
    facts,
    sections: sections.length ? sections : [{ text: de.info.noMods, status: [] }],
  };
}

function relicModel(
  key: string,
  view: RunView,
  ui: UiState,
  relic: { id: string; kind: RelicKind; rarity: Rarity },
  label: string,
  buy?: { slot: number; price: number; ok: boolean },
): InfoModel {
  const id = relic.id;
  const status: InfoStatus[] = relicCounters(id)
    .filter((c) => view.counters[c] !== undefined && de.counters[c])
    .map((c) => ({ text: t(de.counters[c] ?? c, { value: view.counters[c] ?? 0 }), on: null }));
  status.push(...lastGameStatus(lastLines(view, (s) => 'Relic' in s && s.Relic === id)));
  const actions = buy?.ok && !ui.busy ? `<button class="btn btn-primary" ${attrs({ 'data-action': 'buy-relic', 'data-slot': buy.slot })}>${esc(t(de.shop.buy, { price: buy.price }))}</button>` : '';
  return {
    key,
    title: contentName('relics', id),
    kind: `${label} · ${de.rarity[relic.rarity]}`,
    art: `<span class="info-emblem">${relicCoaster(relic, { size: 72 })}</span>`,
    facts: [],
    sections: [{ text: contentText('relics', id), status }],
    ...(actions ? { actions } : {}),
  };
}

function itemModel(key: string, item: ConsumableInfo, extra: string, actions: string): InfoModel {
  const id = item.id;
  return {
    key,
    title: contentName('consumables', id),
    kind: item.category === 'Schnapps' ? de.shop.schnaps : de.info.kinds.item,
    art: `<span class="info-emblem">${consumableCoaster(item, { size: 72 })}</span>`,
    facts: [],
    sections: [{ text: contentText('consumables', id), status: [] }],
    ...(extra ? { extra } : {}),
    ...(actions ? { actions } : {}),
  };
}

function bossModel(key: string, view: RunView, id: string, kind: string, face = false): InfoModel {
  const boss = [view.series.boss, view.upcoming_boss, view.next_series?.boss, view.game?.boss_rule].find((b) => b?.id === id) ?? null;
  const facts = boss && boss.target_factor !== 1 && kind === de.info.kinds.boss ? [t(de.info.targetFactor, { factor: boss.target_factor.toLocaleString('de-DE') })] : [];
  return {
    key,
    title: contentName('bosses', id),
    kind,
    art: face ? portrait(bossPortraitId(id), 'portrait-sm') : `<span class="info-emblem">${bossCoaster(id, { size: 72 })}</span>`,
    facts,
    sections: [{ text: contentText('bosses', id) || boss?.text || '', status: [] }],
  };
}

function seatModel(key: string, view: RunView, seat: 1 | 2): InfoModel | null {
  const person = seatPerson(view, seat);
  const game = view.game;
  const facts: string[] = [];
  if (game) {
    const position = game.positions[seat];
    if (position) facts.push(t(de.info.position, { position: de.positions[position] }));
    if (game.phase === 'Playing' || game.phase === 'Finished') {
      const n = game.seat_tricks[seat] ?? 0;
      facts.push(t(n === 1 ? de.info.trick : de.info.tricks, { n }));
    }
    const size = game.hand_sizes[seat];
    if (size !== undefined) facts.push(t(de.info.handSize, { n: size }));
  }
  if (person.isBoss && view.series.boss) {
    const m = bossModel(key, view, view.series.boss.id, de.info.kinds.boss, true);
    return { ...m, facts: [...m.facts, ...facts] };
  }
  const greet = person.guest ? `„${person.guest.greet}“` : '';
  return {
    key,
    title: person.name,
    kind: de.info.kinds.guest,
    art: portrait(person.face, 'portrait-sm'),
    facts,
    sections: [{ text: de.info.guestText, status: greet ? [{ text: greet, on: null }] : [] }],
  };
}

/** The content of the info card for a key, or null if it is not on screen any more. */
export function infoModel(key: string, view: RunView | null, ui: UiState): InfoModel | null {
  if (!view) return null;
  const [kind, arg] = splitKey(key);
  const n = Number(arg);
  switch (kind) {
    case 'card':
      return cardModel(key, view, ui, n);
    case 'relic': {
      const r = view.relics.find((x) => x.relic.id === arg);
      return r ? relicModel(key, view, ui, r.relic, de.relicKind[r.relic.kind]) : null;
    }
    case 'evening': {
      const r = view.evening_relics.find((x) => x.id === arg);
      return r ? relicModel(key, view, ui, r, de.info.kinds.evening) : null;
    }
    case 'offer-relic': {
      const o = view.shop?.relics.find((x) => x?.slot === n);
      return o ? relicModel(key, view, ui, o.relic, de.relicKind[o.relic.kind], { slot: o.slot, price: o.price, ok: o.can_buy }) : null;
    }
    case 'item': {
      const c = view.consumables.find((x) => x.slot === n);
      if (!c) return null;
      const use = c.can_use && !ui.busy ? `<button class="btn btn-primary" ${attrs({ 'data-action': 'use-item', 'data-slot': c.slot })}>${esc(de.shop.use)}</button>` : '';
      return itemModel(key, c.consumable, '', use);
    }
    case 'offer-item': {
      const o = view.shop?.consumables.find((x) => x?.slot === n);
      if (!o) return null;
      const buy = o.can_buy && !ui.busy ? `<button class="btn btn-primary" ${attrs({ 'data-action': 'buy-item', 'data-slot': o.slot })}>${esc(t(de.shop.buy, { price: o.price }))}</button>` : '';
      return itemModel(key, o.consumable, o.consumable.needs_target ? oddsLine(view.deal_odds) : '', buy);
    }
    case 'seat':
      return n === 1 || n === 2 ? seatModel(key, view, n) : null;
    case 'boss':
      return bossModel(key, view, arg, de.info.kinds.boss);
    case 'rule':
      return bossModel(key, view, arg, de.info.kinds.bossRule);
    default:
      return null;
  }
}

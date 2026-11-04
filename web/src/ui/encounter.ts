// An encounter (Ereignis) on the way, in the style of the pub: a chalk
// sign, the people who speak, the story from the content and the two or
// three options as cards on the table (DESIGN 13, "Wege und Begegnungen
// v2", 03.11.2026). One must be taken: there is no "Weitergehen". What each
// option does (its effects, with what was drawn for them), what it costs
// and whether it can be chosen, with the reason if not, all come from the
// core (EncounterView). Fundgroschen and Würfelbude are encounters too.

import { de, contentName, contentText, isCurse, modText, relicLook, t } from '../i18n';
import type { EncounterEffectView, EncounterOptionView, RunView } from '../types';
import { modIcon } from './cards';
import { glossarySample } from './glossary';
import { attrs, esc } from './dom';
import { coinIcon, heart } from './hud';
import { consumableCoaster, relicCoaster } from './emblems';
import { mapIcon } from './mapicons';
import { encounterFaces, portrait } from './people';
import { categoryIcon } from './route';

/** A beer glass, for higher stakes and rounds. */
function glass(): string {
  return `<svg class="enc-glass" viewBox="0 0 40 52" width="34" height="44" aria-hidden="true"><path d="M7 10 L33 10 L30 48 L10 48 Z" fill="#e8a417" stroke="#2b1a0e" stroke-width="2"/><path d="M5 10 C5 3 12 2 15 5 C18 1 25 1 27 5 C31 2 36 5 35 10 Z" fill="#fffaf0" stroke="#2b1a0e" stroke-width="2"/><path d="M33 18 C40 18 40 34 32 34" fill="none" stroke="#2b1a0e" stroke-width="2.4"/><path d="M14 18 L13 42 M20 18 L20 42" stroke="#fff" stroke-width="1.6" opacity=".45"/></svg>`;
}

/** A marked card, for a Zinke. */
function markedCard(chosen: boolean): string {
  return `<svg class="enc-card" viewBox="0 0 40 52" width="34" height="44" aria-hidden="true"><rect x="5" y="3" width="30" height="45" rx="4" fill="#fbf3e2" stroke="#2b1a0e" stroke-width="2"/><path d="M12 10 L28 10 M12 15 L22 15" stroke="#c9b78f" stroke-width="2"/><circle cx="20" cy="30" r="8" fill="none" stroke="#5e3796" stroke-width="2.4"/><path d="M20 24 L22 29 L27 29 L23 32 L24.5 37 L20 34 L15.5 37 L17 32 L13 29 L18 29 Z" fill="#5e3796"/>${chosen ? '<path d="M29 40 L37 48" stroke="#2b1a0e" stroke-width="3" stroke-linecap="round"/>' : '<text x="31" y="47" font-size="16" fill="#2b1a0e" text-anchor="middle">?</text>'}</svg>`;
}

/** Two skat cards face up, for the card trick. */
function skatPair(): string {
  return `<svg class="enc-card" viewBox="0 0 40 52" width="34" height="44" aria-hidden="true"><rect x="3" y="8" width="22" height="34" rx="3" fill="#fbf3e2" stroke="#2b1a0e" stroke-width="2" transform="rotate(-10 14 25)"/><rect x="15" y="10" width="22" height="34" rx="3" fill="#fbf3e2" stroke="#2b1a0e" stroke-width="2" transform="rotate(8 26 27)"/><circle cx="26" cy="27" r="4" fill="#a3263a"/><circle cx="13" cy="24" r="4" fill="#2f6f3a"/></svg>`;
}

/** A door, for watching. */
function door(): string {
  return `<svg class="enc-door" viewBox="0 0 40 52" width="34" height="44" aria-hidden="true"><rect x="7" y="3" width="26" height="46" rx="2" fill="#6b4427" stroke="#2b1a0e" stroke-width="2"/><rect x="11" y="8" width="18" height="16" fill="#e8c56a" opacity=".7"/><circle cx="27" cy="31" r="2" fill="#e2b04a"/></svg>`;
}

/** A die showing as many pips as Groschen are staked (2, 4, 6). */
function die(pips: number): string {
  const spots: Record<number, [number, number][]> = {
    1: [[20, 26]],
    2: [
      [13, 19],
      [27, 33],
    ],
    3: [
      [13, 19],
      [20, 26],
      [27, 33],
    ],
    4: [
      [13, 19],
      [27, 19],
      [13, 33],
      [27, 33],
    ],
    5: [
      [13, 19],
      [27, 19],
      [20, 26],
      [13, 33],
      [27, 33],
    ],
    6: [
      [13, 17],
      [27, 17],
      [13, 26],
      [27, 26],
      [13, 35],
      [27, 35],
    ],
  };
  const dots = (spots[Math.max(1, Math.min(6, pips))] ?? []).map(([x, y]) => `<rect x="${x - 2.5}" y="${y - 2.5}" width="5" height="5" fill="#fbf3e2"/>`).join('');
  return `<svg class="enc-die" viewBox="0 0 40 52" width="34" height="44" shape-rendering="crispEdges" aria-hidden="true"><rect x="4" y="8" width="32" height="36" rx="4" fill="#6e2f4f" stroke="#2b1a0e" stroke-width="2"/>${dots}</svg>`;
}

/** Payout of a stake at this percentage, as the core counts it. */
function payout(stake: number, percent: number): number {
  return Math.floor((stake * percent) / 100);
}

function isStake(o: EncounterOptionView): boolean {
  return o.effects.some((e) => typeof e.effect === 'object' && ('Bet' in e.effect || 'Dice' in e.effect));
}

/** The cost tag: free, a price (cheaper on some cards) or a stake. */
function costTag(o: EncounterOptionView): string {
  if (!o.cost) return `<span class="enc-cost is-free">${esc(de.encounter.free)}</span>`;
  // A Zinke may cost less on some cards ("Bruderherz": half on Rot).
  const lowest = Math.min(o.cost, ...(o.target_costs ?? []));
  const text = isStake(o)
    ? t(de.encounter.stake, { cost: o.cost })
    : lowest < o.cost
      ? t(de.encounter.costFrom, { cost: o.cost, low: lowest })
      : t(de.encounter.cost, { cost: o.cost });
  return `<span class="enc-cost">${coinIcon(14)}${esc(text)}</span>`;
}

/** A Zinke or curse: badge, name, curse mark, text and where it goes. */
function modLine(e: EncounterEffectView, where: string): string {
  const mod = e.card_mod;
  if (!mod) return '';
  const curse = mod.curse || isCurse(mod.id);
  const name = contentName('card_mods', mod.id) || mod.name;
  const head = t(curse ? de.encounter.curseMod : de.encounter.cardMod, { mod: name });
  return `<span class="enc-mod ${curse ? 'is-curse' : ''}" ${attrs({ 'data-card-mod': mod.id })}>${modIcon(mod.id, 22)}<span><b>${esc(head)}</b>${curse ? `<span class="curse-tag">${esc(de.card.curse)}</span>` : ''}<span class="enc-mod-text">${esc(mod.text || contentText('card_mods', mod.id))}</span><span class="enc-mod-text">${esc(where)}</span></span></span>`;
}

/** Most lives a player can hold, for the lines (set per render). */
let maxLives = 4;

/** One line per effect; a second Zinke of an option goes on "another" card. */
function effectLine(e: EncounterEffectView, o: EncounterOptionView, i: number): string {
  const f = e.effect;
  const line = (text: string): string => `<span class="enc-line">${esc(text)}</span>`;
  if (f === 'SeeSkat') return line(de.encounter.seeSkat);
  if (f === 'Painter') return line(de.encounter.painter);
  if (f === 'Cleanse') return line(de.encounter.cleanse);
  if ('Coins' in f) return line(t(de.encounter.coins, { n: f.Coins }));
  if ('EveningRule' in f) {
    const id = f.EveningRule;
    return id ? line(t(de.encounter.rule, { name: contentName('relics', id), text: contentText('relics', id) })) : '';
  }
  if ('Mark' in f) {
    const earlier = o.effects.slice(0, i).some((x) => typeof x.effect === 'object' && 'Mark' in x.effect);
    const where = f.Mark.card === 'Chosen' ? de.encounter.onChosen : earlier ? de.encounter.onRandomCurse : de.encounter.onRandom;
    const gamble = f.Mark.curse_percent ? line(t(de.encounter.cursePercent, { n: Math.round(100 / f.Mark.curse_percent) })) : '';
    return modLine(e, where) + gamble;
  }
  if ('Marke' in f) {
    // A Marke (04.11.2026): which form, then the Marken with their chance or as a choice.
    const { card, marke } = f.Marke;
    const text =
      marke === 'Random'
        ? card === 'Chosen'
          ? de.encounter.markeRandomChosen
          : de.encounter.markeRandomRandom
        : card === 'Chosen'
          ? de.encounter.markeChosenChosen
          : de.encounter.markeChosenRandom;
    const marken = (o.marken ?? [])
      .map((m) => {
        const name = contentName('card_mods', m.card_mod.id) || m.card_mod.name;
        const label = marke === 'Random' && m.percent > 0 ? t(de.encounter.markeChance, { name, percent: m.percent }) : name;
        return `<span class="enc-marke ${m.can_choose ? '' : 'is-off'}" ${attrs({ 'data-card-mod': m.card_mod.id })}>${modIcon(m.card_mod.id, 18)}<span>${esc(label)}</span></span>`;
      })
      .join('');
    return line(text) + (marken ? `<span class="enc-marken" data-testid="enc-marken">${marken}</span>` : '');
  }
  if ('SwapRelic' in f) {
    const id = f.SwapRelic;
    return id ? line(t(de.encounter.swapFor, { name: contentName('relics', id) || e.relic?.name || id, text: contentText('relics', id) })) : '';
  }
  if ('SellRelic' in f) return line(f.SellRelic >= 100 ? de.encounter.sellRelic : t(de.encounter.sellRelicPart, { percent: f.SellRelic }));
  if ('Relic' in f) {
    const id = f.Relic;
    return id ? line(t(de.encounter.takeRelic, { name: contentName('relics', id), text: contentText('relics', id) })) : '';
  }
  if ('RelicCoins' in f) return '';
  if ('NextGameFactor' in f) return line(t(de.encounter.factor, { n: f.NextGameFactor }));
  if ('NextPubRedeals' in f) {
    const n = f.NextPubRedeals;
    return line(n >= 0 ? t(de.encounter.redealsMore, { n }) : t(de.encounter.redealsFewer, { n: -n }));
  }
  if ('Schnapps' in f) {
    const id = f.Schnapps;
    if (!id) return '';
    const text = line(t(de.encounter.schnapps, { name: contentName('consumables', id), text: contentText('consumables', id) }));
    return e.coins_instead != null ? text + line(t(de.encounter.schnappsFull, { n: e.coins_instead })) : text;
  }
  if ('Lives' in f) return line(t(de.encounter.lives, { n: f.Lives, max: maxLives }));
  if ('Scout' in f) return line(t(de.encounter.scout, { n: f.Scout }));
  if ('Bet' in f) {
    const text = f.Bet.on === 'Win' ? de.encounter.betWin : de.encounter.betSchneider;
    return line(t(text, { payout: payout(o.cost, f.Bet.percent) }));
  }
  if ('Dice' in f) return line(t(de.encounter.dice, { chance: f.Dice.chance, payout: payout(o.cost, f.Dice.percent) }));
  return '';
}

/** The picture of an option: what its first effect gives. */
function optionPic(o: EncounterOptionView): string {
  const first = o.effects[0];
  if (!first) return o.fallback ? door() : coinIcon(34);
  const f = first.effect;
  if (f === 'SeeSkat') return skatPair();
  if (f === 'Painter' || f === 'Cleanse') return markedCard(true);
  if ('Mark' in f) return markedCard(f.Mark.card === 'Chosen');
  if ('Marke' in f) return markedCard(f.Marke.card === 'Chosen');
  if ('Dice' in f) return die(o.cost);
  if ('Lives' in f) return heart(true, 34);
  if ('Scout' in f) return mapIcon('position', 34);
  if ('NextGameFactor' in f || 'NextPubRedeals' in f) return glass();
  if ('Schnapps' in f && first.consumable) return `<span class="enc-coaster">${consumableCoaster(first.consumable, { size: 44 })}</span>`;
  const relic = 'EveningRule' in f ? f.EveningRule : 'SwapRelic' in f ? f.SwapRelic : 'Relic' in f ? f.Relic : null;
  if (relic) return `<span class="enc-coaster">${relicCoaster(first.relic ?? relicLook(relic), { size: 44 })}</span>`;
  return coinIcon(34);
}

function optionCard(o: EncounterOptionView, i: number): string {
  const rule = o.effects.some((e) => typeof e.effect === 'object' && 'EveningRule' in e.effect);
  const tag = rule ? `<span class="enc-tag">${esc(de.encounter.takeRule)}</span>` : '';
  const lines = o.effects.map((e, k) => effectLine(e, o, k)).join('');
  const empty = o.effects.length ? '' : `<span class="enc-line">${esc(o.fallback ? de.encounter.watch : de.encounter.nothing)}</span>`;
  const reason = o.blocked ? `<span class="enc-blocked" data-testid="enc-blocked">${esc(de.encounter.blocked[o.blocked] ?? '')}</span>` : '';
  const curse = o.effects.some((e) => e.card_mod && (e.card_mod.curse || isCurse(e.card_mod.id)));
  const cls = [rule ? 'is-rule' : '', curse ? 'has-curse' : '', o.can_choose ? '' : 'is-blocked'].filter(Boolean).join(' ');
  return `<button class="enc-option ${cls}" style="--i:${i}" ${attrs({
    'data-action': 'choose-encounter',
    'data-option': o.index,
    'data-target': o.target,
    'data-testid': 'enc-option',
    disabled: !o.can_choose,
  })}>
    <span class="enc-pic">${optionPic(o)}</span>
    <span class="enc-body"><b class="enc-title">${esc(o.label)}</b>${tag}${lines}${empty}${reason}</span>
    ${costTag(o)}
  </button>`;
}

export function renderEncounter(view: RunView): string {
  const e = view.encounter;
  if (!e) return '';
  const faces = encounterFaces(e.id)
    .map((f) => portrait(f, 'portrait-enc'))
    .join('');
  maxLives = view.tour.max_lives;
  // A fallback ("Zuschauen") shows only when nothing else can be taken.
  const options = e.options
    .filter((o) => o.blocked !== 'NotNeeded')
    .map(optionCard)
    .join('');
  const category = e.category ?? 'Event';
  const kicker = category === 'Event' ? de.encounter.kicker : (de.route.categories[category] ?? de.encounter.kicker);
  return `<main class="encounter enc-${esc(category.toLowerCase())}" data-testid="encounter" ${attrs({ 'data-encounter': e.id, 'data-category': category })}>
    <div class="enc-wall">
      <div class="enc-sign"><span class="enc-sign-icon">${categoryIcon(category, 22)}</span><span class="enc-kicker">${esc(kicker)}</span><span class="enc-sub">${esc(de.encounter.sign)}</span></div>
      <div class="enc-faces">${faces}</div>
    </div>
    <section class="enc-story">
      <h2>${esc(contentName('encounters', e.id) || e.name)}</h2>
      <p>${esc(contentText('encounters', e.id) || e.text)}</p>
      ${e.options.length > 1 ? `<p class="enc-must small">${esc(de.encounter.mustChoose)}</p>` : ''}
    </section>
    <div class="enc-options">${options}</div>
  </main>`;
}

/** The Marke effect of an option, if it has one (04.11.2026). */
export function markeEffect(o: EncounterOptionView): { card: 'Random' | 'Chosen'; marke: 'Random' | 'Chosen' } | null {
  for (const e of o.effects) {
    if (typeof e.effect === 'object' && 'Marke' in e.effect) return e.effect.Marke;
  }
  return null;
}

/**
 * The sheet body to pick the Marke of a Marke option: the three Marken on
 * a sample card with their text; one that cannot be taken is greyed with
 * its reason. Then the card picker (a chosen card) or the core draws it.
 */
export function renderMarkePick(o: EncounterOptionView): string {
  const random = markeEffect(o)?.card === 'Random';
  const marken = (o.marken ?? [])
    .map((m) => {
      const id = m.card_mod.id;
      const name = contentName('card_mods', id) || m.card_mod.name;
      // The text with its value filled in ("3 Groschen").
      const text = modText(id, m.card_mod.value ?? 1) || m.card_mod.text;
      return `<button class="painter-option marke-option" ${attrs({ 'data-action': 'encounter-marke', 'data-card-mod': id, 'data-testid': `marke-${id}`, disabled: !m.can_choose })}>
        <span class="card">${glossarySample(id)}</span>
        <b class="zinke-name">${esc(name)}</b>
        <span class="zinke-text">${esc(text)}</span>
        ${m.can_choose ? '' : `<span class="zinke-note">${esc(de.encounter.markeBlocked)}</span>`}
        <span class="btn btn-small btn-buy">${esc(de.encounter.markeTake)}</span>
      </button>`;
    })
    .join('');
  return `<h2>${esc(de.encounter.pickMarke)}</h2>
    <p class="small">${esc(random ? de.encounter.pickMarkeHintRandom : de.encounter.pickMarkeHint)}</p>
    <div class="painter-grid" data-testid="marke-options">${marken}</div>
    <button class="btn btn-wide" ${attrs({ 'data-action': 'close-overlay' })}>${esc(de.shop.cancel)}</button>`;
}

/** The sheet body to pick an owned relic for a swap or a sale. */
export function renderRelicPick(view: RunView, option: EncounterOptionView): string {
  const swap = option.effects.find((e) => typeof e.effect === 'object' && 'SwapRelic' in e.effect);
  const sellEffect = option.effects.map((e) => e.effect).find((f) => typeof f === 'object' && 'SellRelic' in f);
  const sellPercent = sellEffect && typeof sellEffect === 'object' && 'SellRelic' in sellEffect ? sellEffect.SellRelic : 0;
  const hint = swap ? t(de.encounter.pickRelicSwap, { new: swap.relic ? contentName('relics', swap.relic.id) || swap.relic.name : '' }) : t(de.encounter.pickRelicSell, { percent: sellPercent });
  const relics = view.relics
    .map((r) => {
      const ok = option.targets.includes(r.index);
      const price = sellEffect ? `<span class="enc-cost">${coinIcon(14)}${esc(t(de.encounter.relicPrice, { price: payout(r.relic.price, sellPercent) }))}</span>` : '';
      return `<button class="enc-option enc-relic" ${attrs({ 'data-action': 'encounter-relic', 'data-index': r.index, 'data-testid': 'enc-relic', disabled: !ok })}>
        <span class="enc-pic"><span class="enc-coaster">${relicCoaster(r.relic, { size: 44 })}</span></span>
        <span class="enc-body"><b class="enc-title">${esc(contentName('relics', r.relic.id))}</b><span class="enc-line">${esc(contentText('relics', r.relic.id))}</span></span>
        ${price}
      </button>`;
    })
    .join('');
  return `<h2>${esc(de.encounter.pickRelic)}</h2>
    <p class="small">${esc(hint)}</p>
    <div class="enc-relics" data-testid="enc-relics">${relics}</div>
    <button class="btn btn-wide" ${attrs({ 'data-action': 'close-overlay' })}>${esc(de.shop.cancel)}</button>`;
}

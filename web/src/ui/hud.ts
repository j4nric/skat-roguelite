// Head of the run screen: where we are in the tour, Groschen and the menu;
// a slate panel with account and target, games and Einmischen left as pips;
// the Leiste: the collected relics as beer mats (./emblems.ts), scrolling
// inside itself, and the Schnaps slots next to them.

import { de, bossFiguresText, contentName, t } from '../i18n';
import type { RunView } from '../types';
import { attrs, esc } from './dom';
import { bossCoaster, consumableCoaster, relicCoaster } from './emblems';

/** A Groschen as an 8 × 8 pixel coin in copper. */
export function coinIcon(size = 18): string {
  return `<svg class="coin" viewBox="0 0 8 8" width="${size}" height="${size}" shape-rendering="crispEdges" aria-hidden="true"><path d="M2 0h4v1h-4zM1 1h6v1h-6zM0 2h8v4h-8zM1 6h6v1h-6zM2 7h4v1h-4z" fill="#d1773f"/><path d="M2 0h4v1h-4zM1 1h1v1h-1zM6 1h1v1h-1zM0 2h1v4h-1zM7 2h1v4h-1zM1 6h1v1h-1zM6 6h1v1h-1zM2 7h4v1h-4z" fill="#8a4a22"/><path d="M3 2h2v1h-2zM3 3h1v2h-1zM3 5h2v1h-2z" fill="#f1b48a"/></svg>`;
}

/** A heart as an 8 × 7 pixel icon: full in red, empty as an outline. */
export function heart(full: boolean, size = 16): string {
  const fill = full ? '#c8312b' : 'none';
  const edge = full ? '#6e1410' : '#8a7a66';
  return `<svg class="heart ${full ? 'is-full' : 'is-empty'}" viewBox="0 0 8 7" width="${size}" height="${Math.round((size * 7) / 8)}" shape-rendering="crispEdges" aria-hidden="true"><path d="M1 0h2v1h2v-1h2v1h1v3h-1v1h-1v1h-1v1h-2v-1h-1v-1h-1v-1h-1v-3h1z" fill="${fill}" stroke="${edge}" stroke-width=".6"/>${full ? '<path d="M1 1h1v1h-1z" fill="#f4a39c"/>' : ''}</svg>`;
}

/** The lives as a row of hearts (filled for each left, empty up to the maximum). */
export function hearts(lives: number, max: number, size = 16): string {
  const n = Math.max(lives, max);
  return Array.from({ length: n }, (_, i) => heart(i < lives, size)).join('');
}

/** The foam crown of a full account bar (hidden by CSS until `.is-full`). */
export function foamCrown(id: string): string {
  return `<svg class="foam" width="100%" height="10" shape-rendering="crispEdges" aria-hidden="true"><defs><pattern id="${id}" width="12" height="10" patternUnits="userSpaceOnUse"><path d="M0 5h12v5h-12zM1 2h5v3h-5zM2 1h3v1h-3zM7 3h4v2h-4zM8 2h2v1h-2z" fill="#f4ecd8"/><path d="M0 9h12v1h-12zM5 3h1v2h-1zM10 4h1v1h-1z" fill="#cdbb92"/><path d="M2 2h1v1h-1zM8 3h1v1h-1z" fill="#ffffff"/></pattern></defs><rect width="100%" height="10" fill="url(#${id})"/></svg>`;
}

/** Meter fill in percent: purely the share of score in target. */
export function meterPercent(score: number, target: number): number {
  return target > 0 ? Math.max(0, Math.min(100, (score / target) * 100)) : 0;
}

/** A row of pips: `left` of `limit` filled. */
function pips(left: number, limit: number, cls: string): string {
  const n = Math.max(limit, left);
  if (n > 8) return '';
  return Array.from({ length: n }, (_, i) => `<i class="${cls} ${i < left ? 'is-on' : ''}"></i>`).join('');
}

/** Relics that work once per series: a small note on their chip. */
const ONCE_PER_SERIES = new Set(['three_discard']);

function onceNote(id: string): string {
  return ONCE_PER_SERIES.has(id) ? `<small class="chip-badge" data-testid="once-${esc(id)}" ${attrs({ title: de.hud.oncePerSeries })}>${esc(de.hud.onceBadge)}</small>` : '';
}

/** "Drei drücken" was used in this series (the core's `discard_three_ready` is off). */
function seriesUsed(view: RunView, id: string): boolean {
  return id === 'three_discard' && Boolean(view.game) && view.game?.discard_three_ready === false;
}

/** A tiny pixel hourglass: "not now" on a Schnaps that cannot be drunk yet. */
function hourglass(): string {
  return `<svg class="slot-wait" viewBox="0 0 7 7" width="14" height="14" shape-rendering="crispEdges" aria-hidden="true"><path d="M0 0h7v1h-7zM0 6h7v1h-7zM0 1h1v1h-1zM6 1h1v1h-1zM1 2h1v1h-1zM5 2h1v1h-1zM2 3h1v1h-1zM4 3h1v1h-1zM1 4h1v1h-1zM5 4h1v1h-1zM0 5h1v1h-1zM6 5h1v1h-1z" fill="#0b0c0e"/><path d="M1 1h5v1h-5zM2 2h3v1h-3zM3 3h1v1h-1zM2 5h3v1h-3zM3 4h1v1h-1z" fill="#a4adb3"/></svg>`;
}

/**
 * The relic part of the Leiste: every relic as its beer mat only (no
 * name; a tap opens the info card), the house rules of the evening, then
 * waiting factors, flags and counters. It scrolls inside itself, the page
 * never sideways (relic slots are unlimited).
 */
export function relicChips(view: RunView): string {
  const relics = view.relics
    .map((r) => {
      const name = contentName('relics', r.relic.id);
      return `<button class="coaster-chip rarity-${esc(r.relic.rarity)} ${seriesUsed(view, r.relic.id) ? 'is-used' : ''}" ${attrs({ 'data-action': 'info-relic', 'data-index': r.index, 'data-relic': r.relic.id, 'data-info': `relic:${r.relic.id}`, 'aria-label': name, title: name })}>${relicCoaster(r.relic, { size: 38 })}${onceNote(r.relic.id)}</button>`;
    })
    .join('');
  const evening = view.evening_relics
    .map((r, i) => {
      const name = `${contentName('relics', r.id)} (${de.evening.tag})`;
      return `<button class="coaster-chip is-evening rarity-${esc(r.rarity)}" ${attrs({ 'data-action': 'info-evening', 'data-index': i, 'data-relic': r.id, 'data-info': `evening:${r.id}`, 'aria-label': name, title: name })}>${relicCoaster(r, { size: 38 })}</button>`;
    })
    .join('');
  const pending = view.pending_factor
    ? `<span class="flag-chip is-pending" data-testid="pending-factor">${esc(t(de.header.pending, { factor: view.pending_factor }))}</span>`
    : '';
  const schnaps = schnapsNotes(view, de.header)
    .map((text) => `<span class="flag-chip is-pending is-schnaps" data-testid="pending-schnaps">${esc(text)}</span>`)
    .join('');
  const flags = view.flags
    .filter((f) => de.flags[f])
    .map((f) => `<span class="flag-chip">${esc(de.flags[f] ?? f)}</span>`)
    .join('');
  const counters = Object.entries(view.counters)
    .filter(([name]) => de.counters[name])
    .map(([name, value]) => `<span class="flag-chip">${esc(t(de.counters[name] ?? name, { value }))}</span>`)
    .join('');
  return relics + evening + pending + schnaps + flags + counters;
}

/**
 * The Schnaps slots of the Leiste (DESIGN 8.9): one per slot, empty ones
 * as dashed outlines, a Schnaps as its bare motif, greyed with an
 * hourglass while the core says it cannot be drunk now (`usable_now`).
 * A drinkable one is lifted by a tap or dragged onto its target
 * (./drink.ts); the others open their info card.
 */
export function schnapsSlots(view: RunView): string {
  const n = Math.max(view.consumable_slots, ...view.consumables.map((c) => c.slot + 1));
  return Array.from({ length: n }, (_, slot) => {
    const c = view.consumables.find((x) => x.slot === slot);
    if (!c) return `<span class="schnaps-slot is-empty" ${attrs({ 'data-testid': `schnaps-slot-${slot}`, 'data-empty-slot': slot, title: de.hud.emptySlot })}></span>`;
    const name = contentName('consumables', c.consumable.id);
    const ready = c.usable_now;
    return `<button class="item-chip schnaps-slot ${ready ? 'is-ready' : 'is-off'}" ${attrs({
      'data-action': ready ? 'lift-item' : 'info-item',
      'data-slot': slot,
      'data-schnaps': slot,
      'data-target': c.consumable.target ?? 'Self',
      'data-info': `item:${slot}`,
      'data-testid': `schnaps-slot-${slot}`,
      'data-usable': ready ? 'true' : 'false',
      'aria-label': ready ? t(de.hud.drinkAria, { name }) : t(de.hud.notNowAria, { name }),
      title: name,
    })}>${consumableCoaster(c.consumable, { size: 40 })}${ready ? '' : hourglass()}</button>`;
  }).join('');
}

/** The Leiste: relics (scrolling inside), a divider, the Schnaps slots. */
export function leiste(view: RunView, malus = ''): string {
  return `<div class="leiste" data-testid="leiste"><div class="leiste-relics" data-testid="leiste-relics">${malus}${relicChips(view)}</div><div class="leiste-slots" data-testid="schnaps-slots">${schnapsSlots(view)}</div></div>`;
}

/** "Doppelkorn ×2" and the like, for each Schnaps waiting for the next game, in the given texts. */
export function schnapsNotes(
  view: RunView,
  texts: { pendingSchnapsMul: string; pendingSchnapsLoss: string; pendingSchnapsWin: string },
): string[] {
  return (view.pending_next_game ?? []).map((p) => {
    const name = contentName('consumables', p.consumable);
    if (typeof p.effect !== 'object') return t(texts.pendingSchnapsLoss, { name });
    return 'MultiplyWin' in p.effect
      ? t(texts.pendingSchnapsWin, { name, factor: p.effect.MultiplyWin })
      : t(texts.pendingSchnapsMul, { name, factor: p.effect.Multiply });
  });
}

/**
 * "Noch nötig" before the last game of a pub (DESIGN 8.7): what the account
 * still needs, or that no game can bring it any more. Empty otherwise.
 */
export function neededLine(view: RunView): string {
  const n = view.needed;
  if (!n || view.phase !== 'Game') return '';
  let text: string;
  if (n.points <= 0) text = de.header.neededDone;
  else if (!n.reachable) text = t(de.header.neededHopeless, { n: n.points, max: n.max_value });
  else if (n.smallest_game !== null && n.smallest_game !== n.points) text = t(de.header.neededGame, { n: n.points, value: n.smallest_game });
  else text = t(de.header.needed, { n: n.points });
  return `<p class="hud-needed ${n.reachable ? '' : 'is-hopeless'}" role="status" data-testid="needed" ${attrs({ 'data-reachable': n.reachable ? 'true' : 'false' })}>${esc(text)}</p>`;
}

export function renderHud(view: RunView): string {
  const s = view.series;
  const where = t(s.is_boss ? de.header.whereBoss : de.header.where, { section: s.section + 1, series: s.index + 1 });
  const tour = view.tour;
  const pub = tour.current;
  const lives = `<span class="hud-lives" data-testid="lives" ${attrs({ 'data-lives': tour.lives, 'aria-label': t(de.lives.aria, { lives: tour.lives, max: tour.max_lives }), title: t(de.lives.aria, { lives: tour.lives, max: tour.max_lives }) })}>${hearts(tour.lives, tour.max_lives, 14)}</span>`;
  const game = Math.min(s.games_played + 1, s.game_limit);
  // "Spiel 2 von 3" as the slate's label; the account follows visibly in
  // the slate, written out for screen readers.
  const progress = `<span class="pub-progress" data-testid="pub-progress">${esc(t(de.pub.game, { game, limit: s.game_limit }))}<span class="sr-only"> · ${esc(t(de.pub.points, { score: s.score, target: s.target }))}</span></span>`;
  const malus = pub?.malus
    ? `<span class="flag-chip is-malus" data-testid="pub-malus" ${attrs({ title: pub.malus.text })}>${esc(t(de.pub.malus, { name: contentName('maluses', pub.malus.id) || pub.malus.name }))}</span>`
    : '';
  // The boss chip only while the boss pub is the one at hand: at its
  // table, or on the route when it is the next target. After the boss pub
  // the route shows the district label instead.
  const atBossTable = (view.phase === 'Game' || view.phase === 'Lost') && s.is_boss;
  const bossNext = view.phase === 'Route' && tour.route?.next.is_boss;
  const bossInfo = atBossTable ? s.boss : bossNext ? (tour.route?.next.boss ?? null) : null;
  // ADR 0017: the chip names both boss players.
  const bossTitle = bossInfo ? `${contentName('bosses', bossInfo.id)}: ${bossFiguresText(bossInfo)}` : '';
  const boss = bossInfo
    ? `<button class="boss-chip" data-testid="hud-boss" ${attrs({ 'data-action': 'info-boss', 'data-info': `boss:${bossInfo.id}`, 'aria-label': bossTitle, title: bossTitle })}>${bossCoaster(bossInfo.id, { size: 34 })}<span class="boss-chip-names"><span>${esc(contentName('bosses', bossInfo.id))}</span><small data-testid="hud-boss-figures">${esc(bossFiguresText(bossInfo))}</small></span></button>`
    : '';
  const fill = meterPercent(s.score, s.target);
  return `<header class="hud">
  <div class="hud-top">
    <button class="hud-where" ${attrs({ 'data-action': 'overview', 'aria-label': de.header.overview })}>
      <span class="hud-where-main">${esc(where)}</span>
      <span class="hud-where-sub"><span data-testid="pub-name">${esc(pub ? (pub.malus ? `${de.pub.spelunke}: ${pub.name}` : pub.name) : de.header.tourHint)}</span> · ${esc(t(de.header.section, { section: s.section + 1, sections: view.sections }))}${
        view.stake > 0 ? `<span data-testid="hud-stake" ${attrs({ title: t(de.header.stakeTitle, { stake: view.stake, augen: view.win_threshold }) })}> · ${esc(t(de.header.stake, { augen: view.win_threshold }))}</span>` : ''
      }</span>
    </button>
    ${boss}
    ${lives}
    <span class="hud-coins" aria-label="${esc(de.header.coins)}">${coinIcon(16)}<b data-testid="coins">${esc(view.coins)}</b></span>
    <button class="icon-btn" ${attrs({ 'data-action': 'menu', 'aria-label': de.header.menu })}><span class="burger"></span></button>
  </div>
  <div class="hud-board">
    <div class="slate" role="meter" aria-label="${esc(de.header.accountTarget)}" aria-valuemin="0" aria-valuemax="${esc(s.target)}" aria-valuenow="${esc(s.score)}">
      <div class="slate-text">
        <span class="slate-label">${view.phase === 'Game' ? progress : esc(de.header.accountTarget)}</span>
        <span class="slate-row"><b class="slate-score ${s.score < 0 ? 'is-negative' : ''}" data-testid="account">${esc(s.score)}</b><span class="slate-sep"> / </span><b class="slate-target">${esc(s.target)}</b></span>
      </div>
      <div class="chalk-meter ${fill >= 100 ? 'is-full' : ''}"><div class="chalk-fill" style="width:${fill.toFixed(1)}%"></div>${foamCrown('foam-hud')}</div>
    </div>
    <div class="hud-counts">
      <div class="count-row" aria-label="${esc(t(de.header.gamesAria, { left: s.games_left, limit: s.game_limit }))}"><span class="count-label">${esc(de.header.games)}</span><span class="pips">${pips(s.games_left, s.game_limit, 'pip-game')}</span><b data-testid="games-left" class="sr-only">${esc(t(de.header.gamesLeft, { left: s.games_left, limit: s.game_limit }))}</b></div>
      <div class="count-row" aria-label="${esc(t(de.header.passesAria, { left: s.redeals_left }))}"><span class="count-label">${esc(de.header.passes)}</span><span class="pips">${pips(s.redeals_left, 0, 'pip-pass')}</span><b data-testid="passes-left" class="count-num">${esc(s.redeals_left)}</b></div>
    </div>
  </div>
  ${neededLine(view)}
  ${leiste(view, malus)}
</header>`;
}

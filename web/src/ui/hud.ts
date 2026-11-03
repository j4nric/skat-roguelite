// Head of the run screen: where we are in the tour, Groschen and the menu;
// a slate panel with account and target, games and Einmischen left as pips;
// the collected relics and items as beer mats (./emblems.ts) in one row.

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
  return ONCE_PER_SERIES.has(id) ? `<small data-testid="once-${esc(id)}">${esc(de.hud.oncePerSeries)}</small>` : '';
}

/** "Drei drücken" was used in this series (the core's `discard_three_ready` is off). */
function seriesUsed(view: RunView, id: string): boolean {
  return id === 'three_discard' && Boolean(view.game) && view.game?.discard_three_ready === false;
}

export function relicChips(view: RunView): string {
  const relics = view.relics
    .map(
      (r) =>
        `<button class="coaster-chip rarity-${esc(r.relic.rarity)} ${seriesUsed(view, r.relic.id) ? 'is-used' : ''}" ${attrs({ 'data-action': 'info-relic', 'data-index': r.index, 'data-relic': r.relic.id, 'data-info': `relic:${r.relic.id}` })}>${relicCoaster(r.relic, { size: 40 })}<span class="chip-name">${esc(contentName('relics', r.relic.id))}${onceNote(r.relic.id)}</span></button>`,
    )
    .join('');
  const evening = view.evening_relics
    .map(
      (r, i) =>
        `<button class="coaster-chip is-evening rarity-${esc(r.rarity)}" ${attrs({ 'data-action': 'info-evening', 'data-index': i, 'data-relic': r.id, 'data-info': `evening:${r.id}` })}>${relicCoaster(r, { size: 40 })}<span class="chip-name">${esc(contentName('relics', r.id))}<small>${esc(de.evening.tag)}</small></span></button>`,
    )
    .join('');
  const pending = view.pending_factor
    ? `<span class="flag-chip is-pending" data-testid="pending-factor">${esc(t(de.header.pending, { factor: view.pending_factor }))}</span>`
    : '';
  const schnaps = schnapsNotes(view, de.header)
    .map((text) => `<span class="flag-chip is-pending is-schnaps" data-testid="pending-schnaps">${esc(text)}</span>`)
    .join('');
  const items = view.consumables
    .map(
      (c) =>
        `<button class="item-chip" ${attrs({ 'data-action': 'info-item', 'data-slot': c.slot, 'data-info': `item:${c.slot}`, 'aria-label': contentName('consumables', c.consumable.id) })}>${consumableCoaster(c.consumable, { size: 40 })}</button>`,
    )
    .join('');
  const flags = view.flags
    .filter((f) => de.flags[f])
    .map((f) => `<span class="flag-chip">${esc(de.flags[f] ?? f)}</span>`)
    .join('');
  const counters = Object.entries(view.counters)
    .filter(([name]) => de.counters[name])
    .map(([name, value]) => `<span class="flag-chip">${esc(t(de.counters[name] ?? name, { value }))}</span>`)
    .join('');
  return relics + evening + items + pending + schnaps + flags + counters;
}

/** "Doppelkorn ×2" and the like, for each Schnaps waiting for the next game, in the given texts. */
export function schnapsNotes(view: RunView, texts: { pendingSchnapsMul: string; pendingSchnapsLoss: string }): string[] {
  return (view.pending_next_game ?? []).map((p) => {
    const name = contentName('consumables', p.consumable);
    return typeof p.effect === 'object'
      ? t(texts.pendingSchnapsMul, { name, factor: p.effect.Multiply })
      : t(texts.pendingSchnapsLoss, { name });
  });
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
  const chips = relicChips(view);
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
      <div class="chalk-meter"><div class="chalk-fill" style="width:${fill.toFixed(1)}%"></div></div>
    </div>
    <div class="hud-counts">
      <div class="count-row" aria-label="${esc(t(de.header.gamesAria, { left: s.games_left, limit: s.game_limit }))}"><span class="count-label">${esc(de.header.games)}</span><span class="pips">${pips(s.games_left, s.game_limit, 'pip-game')}</span><b data-testid="games-left" class="sr-only">${esc(t(de.header.gamesLeft, { left: s.games_left, limit: s.game_limit }))}</b></div>
      <div class="count-row" aria-label="${esc(t(de.header.passesAria, { left: s.redeals_left }))}"><span class="count-label">${esc(de.header.passes)}</span><span class="pips">${pips(s.redeals_left, 0, 'pip-pass')}</span><b data-testid="passes-left" class="count-num">${esc(s.redeals_left)}</b></div>
    </div>
  </div>
  ${chips || malus ? `<div class="chips">${malus}${chips}</div>` : ''}
</header>`;
}

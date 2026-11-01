// Head of the run screen, a wooden beam over the table: where we are in the
// tour, the chalk slate with account and target, games and passes left,
// Groschen, and the collected Bierdeckel (relics) and items.

import { de, contentName, t } from '../i18n';
import type { RunView } from '../types';
import { attrs, esc } from './dom';
import { goodSvg } from './goods';
import { bossPortraitId, portrait } from './people';

export function coinIcon(size = 18): string {
  return `<svg class="coin" viewBox="0 0 20 20" width="${size}" height="${size}" aria-hidden="true"><circle cx="10" cy="10" r="8.5" fill="#e9b949" stroke="#8a6512" stroke-width="1.5"/><circle cx="10" cy="10" r="5.2" fill="none" stroke="#8a6512" stroke-width="1.1"/><path d="M8.4 7.5 L10.6 6.6 L10.6 13.4" fill="none" stroke="#8a6512" stroke-width="1.4"/></svg>`;
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

export function relicChips(view: RunView): string {
  const relics = view.relics
    .map(
      (r) =>
        `<button class="coaster-chip rarity-${esc(r.relic.rarity)}" ${attrs({ 'data-action': 'info-relic', 'data-index': r.index, 'data-relic': r.relic.id })}><span>${esc(contentName('relics', r.relic.id))}</span></button>`,
    )
    .join('');
  const items = view.consumables
    .map(
      (c) =>
        `<button class="item-chip" ${attrs({ 'data-action': 'info-item', 'data-slot': c.slot, 'aria-label': contentName('consumables', c.consumable.id) })}>${goodSvg(c.consumable.id, 30)}</button>`,
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
  return relics + items + flags + counters;
}

export function renderHud(view: RunView): string {
  const s = view.series;
  const where = t(s.is_boss ? de.header.whereBoss : de.header.where, { section: s.section + 1, series: s.index + 1 });
  const boss =
    s.is_boss && s.boss
      ? `<button class="boss-chip" ${attrs({ 'data-action': 'info-boss' })}>${portrait(bossPortraitId(s.boss.id), 'portrait-xs')}<span>${esc(contentName('bosses', s.boss.id))}</span></button>`
      : '';
  const fill = meterPercent(s.score, s.target);
  const chips = relicChips(view);
  return `<header class="hud">
  <div class="hud-top">
    <button class="hud-where" ${attrs({ 'data-action': 'overview', 'aria-label': de.header.overview })}>
      <span class="hud-where-main">${esc(where)}</span>
      <span class="hud-where-sub">${esc(de.header.tourHint)} · ${esc(t(de.header.section, { section: s.section + 1, sections: view.sections }))}</span>
    </button>
    ${boss}
    <span class="hud-coins" aria-label="${esc(de.header.coins)}">${coinIcon(20)}<b data-testid="coins">${esc(view.coins)}</b></span>
    <button class="icon-btn" ${attrs({ 'data-action': 'menu', 'aria-label': de.header.menu })}><span class="burger"></span></button>
  </div>
  <div class="hud-board">
    <div class="slate" role="meter" aria-valuemin="0" aria-valuemax="${esc(s.target)}" aria-valuenow="${esc(s.score)}">
      <div class="slate-row"><span class="slate-label">${esc(de.header.account)}</span><b class="slate-score ${s.score < 0 ? 'is-negative' : ''}" data-testid="account">${esc(s.score)}</b><span class="slate-target">${esc(t(de.header.target, { target: s.target }))}</span></div>
      <div class="chalk-meter"><div class="chalk-fill" style="width:${fill.toFixed(1)}%"></div></div>
    </div>
    <div class="hud-counts">
      <div class="count-row" aria-label="${esc(t(de.header.gamesAria, { left: s.games_left, limit: s.game_limit }))}"><span class="count-label">${esc(de.header.games)}</span><span class="pips">${pips(s.games_left, s.game_limit, 'pip-game')}</span><b data-testid="games-left" class="sr-only">${esc(t(de.header.gamesLeft, { left: s.games_left, limit: s.game_limit }))}</b></div>
      <div class="count-row" aria-label="${esc(t(de.header.passesAria, { left: s.passes_left }))}"><span class="count-label">${esc(de.header.passes)}</span><span class="pips">${pips(s.passes_left, 0, 'pip-pass')}</span><b data-testid="passes-left" class="count-num">${esc(s.passes_left)}</b></div>
    </div>
  </div>
  ${chips ? `<div class="chips">${chips}</div>` : ''}
</header>`;
}

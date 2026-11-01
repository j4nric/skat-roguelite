// Head of the run screen: section, series, account and target, games and
// passes left, Groschen, relics and consumables.

import { de, contentName, t } from '../i18n';
import type { RunView } from '../types';
import { attrs, esc } from './dom';

export function coinIcon(): string {
  return `<svg class="coin" viewBox="0 0 20 20" width="18" height="18" aria-hidden="true"><circle cx="10" cy="10" r="8.5" fill="#e9b949" stroke="#8a6512" stroke-width="1.5"/><circle cx="10" cy="10" r="5" fill="none" stroke="#8a6512" stroke-width="1.2"/></svg>`;
}

export function renderHud(view: RunView): string {
  const s = view.series;
  const where = s.is_boss && s.boss
    ? `<button class="chip chip-boss" ${attrs({ 'data-action': 'info-boss' })}>${esc(t(de.header.bossSeries, { name: contentName('bosses', s.boss.id) }))}</button>`
    : `<span class="hud-series">${esc(t(de.header.series, { series: s.index + 1 }))}</span>`;
  const fill = s.target > 0 ? Math.max(0, Math.min(100, (s.score / s.target) * 100)) : 0;
  const relics = view.relics
    .map(
      (r) =>
        `<button class="chip chip-relic" ${attrs({ 'data-action': 'info-relic', 'data-index': r.index })}>${esc(contentName('relics', r.relic.id))}</button>`,
    )
    .join('');
  const items = view.consumables
    .map(
      (c) =>
        `<button class="chip chip-item" ${attrs({ 'data-action': 'info-item', 'data-slot': c.slot })}>${esc(contentName('consumables', c.consumable.id))}</button>`,
    )
    .join('');
  const flags = view.flags
    .filter((f) => de.flags[f])
    .map((f) => `<span class="chip chip-flag">${esc(de.flags[f] ?? f)}</span>`)
    .join('');
  const counters = Object.entries(view.counters)
    .filter(([name]) => de.counters[name])
    .map(([name, value]) => `<span class="chip chip-flag">${esc(t(de.counters[name] ?? name, { value }))}</span>`)
    .join('');
  const chips = relics + items + flags + counters;
  return `<header class="hud">
  <div class="hud-top">
    <span class="hud-where">${esc(t(de.header.section, { section: s.section + 1, sections: view.sections }))}</span>
    ${where}
    <span class="hud-coins" aria-label="${esc(de.header.coins)}">${coinIcon()}<b data-testid="coins">${esc(view.coins)}</b></span>
    <button class="icon-btn" ${attrs({ 'data-action': 'menu', 'aria-label': de.header.menu })}>☰</button>
  </div>
  <div class="meter" role="meter" aria-valuemin="0" aria-valuemax="${esc(s.target)}" aria-valuenow="${esc(s.score)}">
    <div class="meter-fill ${s.score < 0 ? 'is-negative' : ''}" style="width:${fill.toFixed(1)}%"></div>
    <div class="meter-text"><span>${esc(de.header.account)} <b data-testid="account">${esc(s.score)}</b></span><span>${esc(t(de.header.target, { target: s.target }))}</span></div>
  </div>
  <div class="hud-sub">
    <span>${esc(de.header.games)}: <b data-testid="games-left">${esc(t(de.header.gamesLeft, { left: s.games_left, limit: s.game_limit }))}</b></span>
    <span>${esc(de.header.passes)}: <b data-testid="passes-left">${esc(s.passes_left)}</b></span>
  </div>
  ${chips ? `<div class="chips">${chips}</div>` : ''}
</header>`;
}

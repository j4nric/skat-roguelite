// The Kreidetafel: the Kneipentour as a chalk board (districts × pubs), the
// statistics of this run with tally marks, the record over all runs, and
// the shared deck. Shows only what the core's run view and meta view say;
// future targets are not known here and stay empty.

import { de, contentName, t } from '../i18n';
import type { MetaView, RunView } from '../types';
import { cardSvg } from './cards';
import { attrs, esc } from './dom';
import { oddsDetail } from './odds';
import { bossCoaster, relicCoaster } from './emblems';
import { hearts } from './hud';

/** Tally marks in groups of five, drawn in chalk; large numbers as digits. */
export function tally(n: number): string {
  if (n <= 0) return `<span class="tally-zero">0</span>`;
  if (n > 25) return `<span class="tally-num">${esc(n)}</span>`;
  const groups: string[] = [];
  for (let left = n; left > 0; left -= 5) {
    const k = Math.min(5, left);
    const w = k === 5 ? 26 : k * 6 + 2;
    const strokes = Array.from({ length: Math.min(k, 4) }, (_, i) => `<path d="M${3 + i * 6} 3 L${2 + i * 6} 21"/>`).join('');
    const cross = k === 5 ? `<path d="M0 17 L25 6"/>` : '';
    groups.push(`<svg class="tally" viewBox="0 0 ${w} 24" width="${w}" height="24" aria-hidden="true">${strokes}${cross}</svg>`);
  }
  return `<span class="tally-group" aria-label="${esc(n)}">${groups.join('')}</span>`;
}

const CHECK = `<svg class="chalk-mark" viewBox="0 0 40 30" aria-hidden="true"><path d="M5 16 L15 25 L36 4"/></svg>`;
const CROSS = `<svg class="chalk-mark" viewBox="0 0 40 30" aria-hidden="true"><path d="M8 4 L32 26 M32 4 L8 26"/></svg>`;

/** The whole tour (ADR 0016): a row per district, a cell per pub. */
function grid(view: RunView, lost: boolean): string {
  const cur = view.series;
  const head = `<div class="tour-cell tour-head"></div>${de.overview.columns.map((c) => `<div class="tour-cell tour-head">${esc(c)}</div>`).join('')}`;
  const rows = view.tour.districts.map((d) => {
    const cells = d.pubs
      .map((p) => {
        const spelunke = d.spelunken.find((s) => s.index === p.index);
        const state = spelunke ? spelunke.state : p.state;
        const title = spelunke ? `${de.pub.spelunke}: ${spelunke.name}` : p.name;
        const node = attrs({ 'data-node': spelunke ? spelunke.id : p.id, title, 'data-state': state });
        const face = p.is_boss && d.boss ? bossCoaster(d.boss.id, { size: 28 }) : '';
        const mark = spelunke ? `<span class="tour-spelunke">${esc(de.pub.spelunke)}</span>` : '';
        if (state === 'Won') return `<div class="tour-cell is-done" ${node} aria-label="${esc(de.overview.done)}">${CHECK}${mark}</div>`;
        if (state === 'Lost') return `<div class="tour-cell is-done is-lost" ${node}>${CROSS}${mark}</div>`;
        if (state === 'Current') {
          return `<div class="tour-cell is-now ${lost ? 'is-lost' : ''}" ${node}>${lost ? CROSS : ''}${face}${mark}<b>${esc(t(de.overview.current, { score: cur.score, target: cur.target }))}</b></div>`;
        }
        if (p.is_boss) return `<div class="tour-cell is-later is-next" ${node}>${face || `<span class="tour-q">${esc(de.boss.unknown)}</span>`}</div>`;
        return `<div class="tour-cell is-later" ${node}></div>`;
      })
      .join('');
    return `<div class="tour-cell tour-row-head ${d.index === cur.section ? 'is-current' : ''}" title="${esc(d.name)}">${esc(t(de.overview.section, { n: d.index + 1 }))}<small>${esc(d.name)}</small></div>${cells}`;
  });
  return `<div class="tour-grid" data-testid="tour-grid">${head}${rows.join('')}</div>`;
}

export function renderOverview(view: RunView, meta: MetaView | null, lost = false): string {
  const s = view.stats;
  const stats: [string, string][] = [
    [de.end.gamesWon, tally(s.games_won)],
    [de.end.gamesLost, tally(s.games_lost)],
    [de.end.gamesPassed, tally(s.redeals)],
    [de.end.seriesWon, tally(s.series_won)],
    [de.lives.label, hearts(view.tour.lives, view.tour.max_lives, 14)],
    [de.end.bestGame, `<b class="chalk-num">${esc(s.best_game)}</b>`],
    [de.end.coins, `<b class="chalk-num">${esc(view.coins)}</b>`],
  ];
  const m = meta?.stats;
  const all: [string, string][] = m
    ? [
        [de.overview.runs, tally(m.runs_finished)],
        [de.overview.runsWon, tally(m.runs_won)],
        [de.overview.bestSection, `<b class="chalk-num">${esc(m.best_section)}</b>`],
      ]
    : [];
  const dl = (list: [string, string][]): string =>
    `<dl class="chalk-stats">${list.map(([k, v]) => `<dt>${esc(k)}</dt><dd>${v}</dd>`).join('')}</dl>`;
  const deck = view.edition
    .map((c) => `<span class="deck-card" ${attrs({ 'data-info': `card:${c.id}`, tabindex: 0 })}>${cardSvg(c.kind, c.mods, 'full', c.mod_values)}</span>`)
    .join('');
  const relics =
    view.relics.map((r) => `<span class="mini-coaster rarity-${esc(r.relic.rarity)}">${relicCoaster(r.relic, { size: 26 })}${esc(contentName('relics', r.relic.id))}</span>`).join('') +
    view.evening_relics.map((r) => `<span class="mini-coaster is-evening rarity-${esc(r.rarity)}">${relicCoaster(r, { size: 26 })}${esc(contentName('relics', r.id))}</span>`).join('');
  return `<div class="tour">
    <h3 class="chalk-h">${esc(de.overview.tour)}</h3>
    ${grid(view, lost)}
    ${relics ? `<div class="relic-strip">${relics}</div>` : ''}
    <h3 class="chalk-h">${esc(de.overview.thisRun)}</h3>
    <p class="chalk-small">${esc(t(de.overview.character, { name: contentName('characters', view.character.character) }))}</p>
    ${dl(stats)}
    ${all.length ? `<h3 class="chalk-h">${esc(de.overview.allRuns)}</h3>${dl(all)}` : ''}
    <h3 class="chalk-h">${esc(t(de.overview.deck, { count: view.edition.length }))}</h3>
    <p class="chalk-small">${esc(de.overview.deckHint)}</p>
    ${oddsDetail(view.deal_odds)}
    <div class="deck-grid">${deck}</div>
  </div>`;
}

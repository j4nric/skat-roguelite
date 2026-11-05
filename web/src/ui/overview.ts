// The Kreidetafel: the Kneipentour as a chalk board (districts × pubs), the
// statistics of this run and the record over all runs as rows of
// numbers (no tally marks: they read like lives, playtest 05.11.2026), and
// the shared deck. Shows only what the core's run view and meta view say;
// future targets are not known here and stay empty.

import { de, contentName, t } from '../i18n';
import type { MetaView, RunView } from '../types';
import { editionLook, lookSvg, type SettledWild } from './cards';
import { attrs, esc } from './dom';
import { oddsDetail } from './odds';
import { bossCoaster, relicCoaster } from './emblems';

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

/**
 * The tour, this run, all runs and the edition. `settled`: the Wandelkarten
 * the core settled in the game in progress (the edition shows them settled).
 */
export function renderOverview(view: RunView, meta: MetaView | null, lost = false, settled: Readonly<Record<number, SettledWild>> = {}): string {
  const s = view.stats;
  const stats: [string, number][] = [
    [de.end.gamesWon, s.games_won],
    [de.end.gamesLost, s.games_lost],
    [de.end.gamesPassed, s.redeals],
    [de.end.seriesWon, s.series_won],
    [de.end.bestGame, s.best_game],
    [de.end.coins, view.coins],
  ];
  const m = meta?.stats;
  const all: [string, number][] = m
    ? [
        [de.overview.runs, m.runs_finished],
        [de.overview.runsWon, m.runs_won],
        [de.overview.bestSection, m.best_section],
      ]
    : [];
  // One row of chalk numbers, each with its label under it.
  const row = (list: [string, number][], id: string): string =>
    `<ul class="chalk-tiles" data-testid="${id}">${list.map(([k, v]) => `<li><b class="chalk-num">${esc(v)}</b><span>${esc(k)}</span></li>`).join('')}</ul>`;
  const deck = view.edition
    .map((c) => `<span class="deck-card" ${attrs({ 'data-info': `card:${c.id}`, tabindex: 0 })}>${lookSvg(editionLook(c, view, settled), 'full')}</span>`)
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
    ${row(stats, 'stats-run')}
    ${all.length ? `<h3 class="chalk-h">${esc(de.overview.allRuns)}</h3>${row(all, 'stats-all')}` : ''}
    <h3 class="chalk-h">${esc(t(de.overview.deck, { count: view.edition.length }))}</h3>
    <p class="chalk-small">${esc(de.overview.deckHint)}</p>
    ${oddsDetail(view.deal_odds)}
    <div class="deck-grid">${deck}</div>
  </div>`;
}

// Start and end screens and the small dialogs.

import { de, cardName, contentName, contentText, t } from '../i18n';
import type { RunView } from '../types';
import { cardSvg, suitIcon } from './cards';
import { attrs, esc } from './dom';
import type { Overlay, UiState } from './state';

export function renderLoading(): string {
  return `<main class="start"><h1>${esc(de.app.title)}</h1><p>${esc(de.app.loading)}</p></main>`;
}

export function renderError(): string {
  return `<main class="start"><h1>${esc(de.app.title)}</h1><p class="is-warning">${esc(de.app.loadFailed)}</p></main>`;
}

export function renderStart(ui: UiState, buildInfo: string): string {
  const resume = ui.saveInfo
    ? `<button class="btn btn-primary btn-wide" ${attrs({ 'data-action': 'resume' })}>${esc(de.start.resume)}</button>
       <p class="small" data-testid="resume-info">${esc(ui.saveInfo)}</p>`
    : '';
  const record = ui.record.runs
    ? t(de.start.record, { runs: ui.record.runs, won: ui.record.won, best: ui.record.bestSection })
    : de.start.noRecord;
  return `<main class="start">
    <div class="logo">${(['Clubs', 'Spades', 'Hearts', 'Diamonds'] as const).map((s) => suitIcon(s, 40)).join('')}</div>
    <h1>${esc(de.app.title)}</h1>
    <p class="tagline">${esc(de.app.tagline)}</p>
    ${ui.message ? `<p class="is-warning" role="alert">${esc(ui.message)}</p>` : ''}
    <div class="start-actions">
      ${resume}
      <button class="btn ${ui.saveInfo ? '' : 'btn-primary'} btn-wide" ${attrs({ 'data-action': 'new-run' })}>${esc(de.start.newRun)}</button>
      <label class="seed"><span>${esc(de.start.seedLabel)}</span>
        <input id="seed" name="seed" inputmode="numeric" autocomplete="off" placeholder="${esc(de.start.seedPlaceholder)}"></label>
    </div>
    <p class="howto">${esc(de.start.howTo)}</p>
    <p class="small">${esc(record)}</p>
    <footer class="build" data-testid="build">${esc(t(de.app.build, { info: buildInfo }))}</footer>
  </main>`;
}

export function renderEnd(won: boolean, view: RunView): string {
  const s = view.stats;
  const rows: [string, string | number][] = [
    [de.end.gamesWon, s.games_won],
    [de.end.gamesLost, s.games_lost],
    [de.end.gamesPassed, s.games_passed],
    [de.end.seriesWon, s.series_won],
    [de.end.bestGame, s.best_game],
    [de.end.coins, view.coins],
    [de.end.seed, view.seed],
  ];
  return `<main class="end" data-testid="end">
    <h1 class="${won ? 'is-won' : 'is-lost'}">${esc(won ? de.end.won : de.end.lost)}</h1>
    <p>${esc(won ? de.end.wonText : t(de.end.lostText, { section: view.series.section + 1, series: view.series.index + 1 }))}</p>
    <dl class="stats">${rows.map(([k, v]) => `<dt>${esc(k)}</dt><dd>${esc(v)}</dd>`).join('')}</dl>
    <button class="btn btn-primary btn-wide" ${attrs({ 'data-action': 'new-run' })}>${esc(de.end.newRun)}</button>
    <button class="btn btn-wide" ${attrs({ 'data-action': 'to-start' })}>${esc(de.end.toStart)}</button>
  </main>`;
}

function sheet(inner: string, closable = true): string {
  return `<div class="overlay" ${closable ? attrs({ 'data-action': 'close-overlay' }) : ''}><div class="sheet" role="dialog" aria-modal="true">${inner}</div></div>`;
}

export function renderKontra(view: RunView): string {
  const kontra = view.game?.kontra;
  const seat = typeof kontra === 'object' && 'Kontra' in kontra ? kontra.Kontra : 1;
  return sheet(
    `<h2>${esc(de.kontra.title)}</h2>
     <p>${esc(t(de.kontra.text, { name: de.seats[seat] ?? '' }))}</p>
     <p>${esc(de.kontra.question)}</p>
     <div class="sheet-actions">
       <button class="btn btn-primary" ${attrs({ 'data-action': 're', 'data-re': 'true' })}>${esc(de.kontra.re)}</button>
       <button class="btn" ${attrs({ 'data-action': 're', 'data-re': 'false' })}>${esc(de.kontra.noRe)}</button>
     </div>`,
    false,
  );
}

export function renderOverlay(overlay: Overlay, view: RunView | null): string {
  switch (overlay.kind) {
    case 'boss':
      return sheet(
        `<p class="small">${esc(de.boss.title)} · ${esc(de.boss.intro)}</p>
         <h2>${esc(contentName('bosses', overlay.boss.id))}</h2>
         <p>${esc(contentText('bosses', overlay.boss.id) || overlay.boss.text)}</p>
         <button class="btn btn-primary btn-wide" ${attrs({ 'data-action': 'boss-go' })}>${esc(de.boss.go)}</button>`,
        false,
      );
    case 'info':
      return sheet(
        `<p class="small">${esc(overlay.subtitle)}</p>
         <h2>${esc(overlay.title)}</h2>
         <p>${esc(overlay.text)}</p>
         <div class="sheet-actions">
           ${overlay.use !== undefined ? `<button class="btn btn-primary" ${attrs({ 'data-action': 'use-item', 'data-slot': overlay.use })}>${esc(de.shop.use)}</button>` : ''}
           <button class="btn" ${attrs({ 'data-action': 'close-overlay' })}>${esc(de.app.close)}</button>
         </div>`,
      );
    case 'menu':
      return sheet(
        `<h2>${esc(de.header.menu)}</h2>
         <p class="small">${esc(view ? t(de.header.seed, { seed: view.seed }) : '')}</p>
         <div class="sheet-actions">
           <button class="btn" ${attrs({ 'data-action': 'to-start' })}>${esc(de.end.toStart)}</button>
           <button class="btn" ${attrs({ 'data-action': 'close-overlay' })}>${esc(de.app.close)}</button>
         </div>`,
      );
    case 'target': {
      const item = view?.consumables.find((c) => c.slot === overlay.slot);
      if (!view || !item) return '';
      const cards = view.deck
        .map((c) => {
          const ok = item.targets.includes(c.id);
          return `<button class="card target-card ${ok ? '' : 'is-dim'}" ${attrs({
            'data-action': 'target',
            'data-id': c.id,
            disabled: !ok,
            'aria-label': cardName(c.kind),
          })}>${cardSvg(c.kind, c.mods)}</button>`;
        })
        .join('');
      return sheet(
        `<h2>${esc(de.shop.pickTarget)}</h2>
         <p class="small">${esc(t(de.shop.pickHint, { name: contentName('consumables', item.consumable.id), text: contentText('consumables', item.consumable.id) }))}</p>
         <div class="target-grid">${cards}</div>
         <button class="btn btn-wide" ${attrs({ 'data-action': 'close-overlay' })}>${esc(de.shop.cancel)}</button>`,
      );
    }
    case 'score':
      return '';
  }
}

// Start and end screens and the dialogs: Kontra, boss, info, menu with
// settings, Kreidetafel, Abspann, target picking.

import { de, cardName, contentName, contentText, t } from '../i18n';
import type { RunView } from '../types';
import { cardSvg } from './cards';
import { attrs, esc, reducedMotion } from './dom';
import { goodSvg } from './goods';
import { oddsLine } from './odds';
import { renderOverview } from './overview';
import { bossPortraitId, portrait } from './people';
import type { Overlay, UiState } from './state';

export function renderLoading(): string {
  return `<main class="start"><div class="sign"><h1>${esc(de.app.title)}</h1></div><p>${esc(de.app.loading)}</p></main>`;
}

export function renderError(): string {
  return `<main class="start"><div class="sign"><h1>${esc(de.app.title)}</h1></div><p class="is-warning">${esc(de.app.loadFailed)}</p></main>`;
}

function startFan(): string {
  const cards = ['CA', 'SA', 'HA', 'DA']
    .map((c, i) => `<span class="fan-card" style="--i:${i}">${cardSvg({ Normal: c })}</span>`)
    .join('');
  return `<div class="start-fan" aria-hidden="true">${cards}</div>`;
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
    <div class="lamp" aria-hidden="true"></div>
    <div class="sign">
      <span class="sign-small">${esc(de.start.sign)}</span>
      <h1>${esc(de.app.title)}</h1>
      <span class="sign-tag">${esc(de.app.tagline)}</span>
    </div>
    ${startFan()}
    ${ui.message ? `<p class="is-warning" role="alert">${esc(ui.message)}</p>` : ''}
    <div class="start-actions">
      ${resume}
      <button class="btn ${ui.saveInfo ? '' : 'btn-primary'} btn-wide" ${attrs({ 'data-action': 'new-run' })}>${esc(de.start.newRun)}</button>
      <label class="seed"><span>${esc(de.start.seedLabel)}</span>
        <input id="seed" name="seed" inputmode="numeric" autocomplete="off" placeholder="${esc(de.start.seedPlaceholder)}"></label>
    </div>
    <p class="howto">${esc(de.start.howTo)}</p>
    <p class="small">${esc(record)}</p>
    <footer class="start-foot">
      <button class="link-btn" ${attrs({ 'data-action': 'settings' })}>${esc(de.start.settings)}</button>
      <button class="link-btn" ${attrs({ 'data-action': 'credits' })}>${esc(de.start.credits)}</button>
      <span class="build" data-testid="build">${esc(t(de.app.build, { info: buildInfo }))}</span>
    </footer>
  </main>`;
}

export function renderEnd(won: boolean, view: RunView, ui: UiState): string {
  return `<main class="end" data-testid="end">
    <div class="sheet chalkboard end-board">
      <h1 class="${won ? 'is-won' : 'is-lost'}">${esc(won ? de.end.won : de.end.lost)}</h1>
      <p class="chalk-small">${esc(won ? de.end.wonText : t(de.end.lostText, { section: view.series.section + 1, series: view.series.index + 1 }))}</p>
      <p class="chalk-small">${esc(de.end.seed)}: ${esc(view.seed)}</p>
      ${renderOverview(view, ui.record, !won)}
    </div>
    <div class="end-actions">
      <button class="btn btn-primary btn-wide" ${attrs({ 'data-action': 'new-run' })}>${esc(de.end.newRun)}</button>
      <button class="btn btn-wide" ${attrs({ 'data-action': 'to-start' })}>${esc(de.end.toStart)}</button>
    </div>
  </main>`;
}

function sheet(inner: string, closable = true, cls = '', overlayCls = ''): string {
  return `<div class="overlay ${overlayCls}" ${closable ? attrs({ 'data-action': 'close-overlay' }) : ''}><div class="sheet ${cls}" role="dialog" aria-modal="true">${inner}</div></div>`;
}

export function renderKontra(view: RunView): string {
  const kontra = view.game?.kontra;
  const seat = typeof kontra === 'object' && 'Kontra' in kontra ? kontra.Kontra : 1;
  const boss = seat === 1 && view.series.is_boss && view.series.boss ? view.series.boss : null;
  const face = boss ? bossPortraitId(boss.id) : `p-opp-${seat}`;
  const name = boss ? contentName('bosses', boss.id) : (de.seats[seat] ?? '');
  return sheet(
    `<div class="speaker">${portrait(face, 'portrait-lg')}<div class="speech"><h2>${esc(de.kontra.title)}</h2>
     <p>${esc(t(de.kontra.text, { name }))}</p></div></div>
     <p>${esc(de.kontra.question)}</p>
     <div class="sheet-actions">
       <button class="btn btn-primary" ${attrs({ 'data-action': 're', 'data-re': 'true' })}>${esc(de.kontra.re)}</button>
       <button class="btn" ${attrs({ 'data-action': 're', 'data-re': 'false' })}>${esc(de.kontra.noRe)}</button>
     </div>`,
    false,
    'sheet-kontra',
    'overlay-kontra',
  );
}

function settingsBlock(ui: UiState): string {
  const tempos = (['slow', 'normal', 'fast'] as const)
    .map(
      (k) =>
        `<button class="seg ${ui.settings.tempo === k ? 'is-picked' : ''}" ${attrs({ 'data-action': 'tempo', 'data-tempo': k, 'aria-pressed': ui.settings.tempo === k ? 'true' : 'false' })}>${esc(de.menu.tempos[k])}</button>`,
    )
    .join('');
  const sound = ui.settings.sound;
  return `<div class="setting"><span class="setting-label">${esc(de.menu.tempo)}</span><div class="segmented">${tempos}</div></div>
    ${reducedMotion() ? `<p class="small">${esc(de.menu.reducedMotion)}</p>` : ''}
    <div class="setting"><span class="setting-label">${esc(de.menu.sound)}</span><div class="segmented">
      <button class="seg ${sound ? 'is-picked' : ''}" ${attrs({ 'data-action': 'sound', 'data-on': 'true', 'aria-pressed': sound ? 'true' : 'false' })}>${esc(de.menu.soundOn)}</button>
      <button class="seg ${!sound ? 'is-picked' : ''}" ${attrs({ 'data-action': 'sound', 'data-on': 'false', 'aria-pressed': !sound ? 'true' : 'false' })}>${esc(de.menu.soundOff)}</button>
    </div></div>`;
}

function credits(): string {
  const rows = de.credits.entries
    .map((e) => `<li><b>${esc(e.what)}</b><span>${esc(e.who)}</span><span class="credit-license">${esc(e.license)}</span></li>`)
    .join('');
  return `<h2>${esc(de.credits.title)}</h2>
    <p>${esc(de.credits.intro)}</p>
    <ul class="credits">${rows}</ul>
    <p class="credits-thanks">${esc(de.credits.thanks)}</p>
    <button class="btn btn-wide" ${attrs({ 'data-action': 'close-overlay' })}>${esc(de.app.close)}</button>`;
}

export function renderOverlay(overlay: Overlay, view: RunView | null, ui: UiState): string {
  switch (overlay.kind) {
    case 'boss':
      return sheet(
        `<div class="boss-intro">${portrait(bossPortraitId(overlay.boss.id), 'portrait-boss')}
         <p class="small">${esc(de.boss.title)} · ${esc(de.boss.intro)}</p>
         <h2>${esc(contentName('bosses', overlay.boss.id))}</h2>
         <p>${esc(contentText('bosses', overlay.boss.id) || overlay.boss.text)}</p></div>
         <button class="btn btn-primary btn-wide" ${attrs({ 'data-action': 'boss-go' })}>${esc(de.boss.go)}</button>`,
        false,
        'sheet-boss',
      );
    case 'info': {
      const pic = overlay.good ? `<div class="info-pic">${goodSvg(overlay.good, 72)}</div>` : '';
      const body = overlay.coaster
        ? `<article class="coaster coaster-big rarity-${esc(overlay.coaster)}"><div class="coaster-kind">${esc(overlay.subtitle)}</div><h4 class="coaster-name">${esc(overlay.title)}</h4><p class="coaster-text">${esc(overlay.text)}</p></article>`
        : `${pic}<p class="small">${esc(overlay.subtitle)}</p><h2>${esc(overlay.title)}</h2><p>${esc(overlay.text)}</p>${overlay.odds && view ? oddsLine(view.deal_odds) : ''}`;
      return sheet(
        `${body}
         <div class="sheet-actions">
           ${overlay.use !== undefined ? `<button class="btn btn-primary" ${attrs({ 'data-action': 'use-item', 'data-slot': overlay.use })}>${esc(de.shop.use)}</button>` : ''}
           ${overlay.buy ? `<button class="btn btn-primary" ${attrs({ 'data-action': overlay.buy.action, 'data-slot': overlay.buy.slot })}>${esc(t(de.shop.buy, { price: overlay.buy.price }))}</button>` : ''}
           <button class="btn" ${attrs({ 'data-action': 'close-overlay' })}>${esc(de.app.close)}</button>
         </div>`,
      );
    }
    case 'menu':
      return sheet(
        `<h2>${esc(de.header.menu)}</h2>
         ${settingsBlock(ui)}
         <div class="menu-list">
           ${view ? `<button class="btn btn-wide" ${attrs({ 'data-action': 'overview' })}>${esc(de.menu.overview)}</button>` : ''}
           <button class="btn btn-wide" ${attrs({ 'data-action': 'credits' })}>${esc(de.menu.credits)}</button>
           ${view && ui.screen === 'run' ? `<button class="btn btn-wide" ${attrs({ 'data-action': 'to-start' })}>${esc(de.end.toStart)}</button>` : ''}
           <button class="btn btn-wide" ${attrs({ 'data-action': 'close-overlay' })}>${esc(de.app.close)}</button>
         </div>
         ${view ? `<p class="small">${esc(t(de.header.seed, { seed: view.seed }))}</p>` : ''}`,
      );
    case 'overview':
      if (!view) return '';
      return sheet(
        `<h2>${esc(de.overview.title)}</h2>
         ${renderOverview(view, ui.record)}
         <button class="btn btn-wide" ${attrs({ 'data-action': 'close-overlay' })}>${esc(de.app.close)}</button>`,
        true,
        'chalkboard sheet-overview',
      );
    case 'credits':
      return sheet(credits(), true, 'sheet-credits');
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
         ${oddsLine(view.deal_odds)}
         <div class="target-grid">${cards}</div>
         <button class="btn btn-wide" ${attrs({ 'data-action': 'close-overlay' })}>${esc(de.shop.cancel)}</button>`,
      );
    }
    case 'score':
      return '';
  }
}

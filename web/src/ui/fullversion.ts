// The Kauf-Seite of the iOS app (ADR 0028): a slip of paper from the
// Wirt (DESIGN 16.6), "Noch 'ne Runde?", what the full version opens, the
// button with the price StoreKit names, "Käufe wiederherstellen" and
// "Später". Every state of a purchase has its line: loading, no price
// (offline), buying, cancelled, pending (Ask to Buy), failed, restoring,
// nothing found, and the joy when it is done. Presentation only; what
// happens on a tap is ./app/fullversion.ts, the store is ../store.ts.

import { de, t } from '../i18n';
import type { FullOverlay } from './state';
import { attrs, esc } from './dom';
import { portrait } from './people';

/** A full beer glass, for the joy. */
function glass(): string {
  return `<svg class="full-glass" viewBox="0 0 40 52" width="64" height="83" aria-hidden="true"><path d="M7 10 L33 10 L30 48 L10 48 Z" fill="#e8a417" stroke="#2b1a0e" stroke-width="2"/><path d="M5 10 C5 3 12 2 15 5 C18 1 25 1 27 5 C31 2 36 5 35 10 Z" fill="#fffaf0" stroke="#2b1a0e" stroke-width="2"/><path d="M33 18 C40 18 40 34 32 34" fill="none" stroke="#2b1a0e" stroke-width="2.4"/><path d="M14 18 L13 42 M20 18 L20 42" stroke="#fff" stroke-width="1.6" opacity=".45"/></svg>`;
}

/** The line under the list: where the purchase stands. */
function statusLine(o: FullOverlay): string {
  const F = de.full;
  const text: Partial<Record<FullOverlay['step'], string>> = {
    loading: F.loading,
    offline: F.offline,
    buying: F.buying,
    cancelled: F.cancelled,
    pending: F.pending,
    failed: F.failed,
    restoring: F.restoring,
    nothing: F.nothingFound,
  };
  const line = text[o.step];
  if (!line) return `<p class="full-status" role="status" ${attrs({ 'data-testid': 'full-status', 'data-step': o.step })}></p>`;
  const warn = o.step === 'failed' || o.step === 'offline' || o.step === 'nothing';
  return `<p class="full-status ${warn ? 'is-warn' : ''}" role="status" ${attrs({ 'data-testid': 'full-status', 'data-step': o.step })}>${esc(line)}</p>`;
}

/** The sheet's content (without the overlay around it). */
export function renderFullVersion(o: FullOverlay): string {
  const F = de.full;
  if (o.step === 'done') {
    return `<div class="full-joy" data-testid="full-done" role="status">
      ${glass()}
      <h2>${esc(F.doneTitle)}</h2>
      <p>${esc(o.wall ? F.doneWall : F.doneText)}</p>
    </div>`;
  }
  const busy = o.step === 'buying' || o.step === 'restoring';
  const items = F.items.map((item) => `<li>${esc(item)}</li>`).join('');
  const buy =
    o.step === 'offline'
      ? `<button class="btn btn-wide" ${attrs({ 'data-action': 'full-retry', 'data-testid': 'full-retry' })}>${esc(F.retry)}</button>`
      : `<button class="btn btn-primary btn-wide full-buy" ${attrs({ 'data-action': 'full-buy', 'data-testid': 'full-buy', disabled: busy || !o.offer || o.step === 'pending', 'aria-busy': o.step === 'buying' ? 'true' : null })}>${esc(o.offer ? t(F.buy, { price: o.offer.price }) : F.buyNoPrice)}</button>`;
  return `<div class="full-head">
      ${portrait('p-wirt', 'portrait-full')}
      <div class="full-sign"><span class="full-kicker">${esc(o.wall ? F.kickerWall : F.kicker)}</span><h2>${esc(F.title)}</h2></div>
    </div>
    <p class="full-lead">${esc(o.wall ? F.leadWall : F.lead)}</p>
    <ul class="full-list" data-testid="full-list">${items}</ul>
    <p class="full-fine small">${esc(F.fine)}</p>
    ${statusLine(o)}
    <div class="full-actions">
      ${buy}
      <button class="btn btn-wide" ${attrs({ 'data-action': 'full-restore', 'data-testid': 'full-restore', disabled: busy })}>${esc(F.restore)}</button>
      <button class="link-btn full-later" ${attrs({ 'data-action': 'full-later', 'data-testid': 'full-later', disabled: o.step === 'buying' })}>${esc(o.wall ? F.laterWall : F.later)}</button>
    </div>`;
}

// The sheet "Fehler melden" (ADR 0027): a text field, what goes along
// (folded list), the save code box, "Senden"; then the report's number,
// or the note that it waits for the network, or the report as text.

import { de, t } from '../i18n';
import { MESSAGE_CHARS } from '../report';
import { attrs, esc } from './dom';
import type { ReportOverlay } from './state';

type Sheet = (inner: string, closable?: boolean, cls?: string, overlayCls?: string) => string;

function copyStatus(copied: boolean | null): string {
  if (copied === true) return `<p class="share-ok" role="status" data-testid="report-copied">${esc(de.report.copied)}</p>`;
  if (copied === false) return `<p class="small is-warning" role="status">${esc(de.report.copyFailed)}</p>`;
  return '';
}

export function renderReport(sheet: Sheet, o: ReportOverlay, hadTroubleBefore: boolean): string {
  const state = { class: 'report-body', 'data-testid': 'report-sheet', 'data-state': o.phase };
  if (o.phase === 'sent') {
    return sheet(
      `<div ${attrs(state)}>
       <h2>${esc(de.report.sentTitle)}</h2>
       <p>${esc(de.report.sentText)}</p>
       <p class="report-id" data-testid="report-id">${esc(o.id ?? '')}</p>
       <p class="small">${esc(de.report.sentHint)}</p>
       <button class="btn btn-primary btn-wide" ${attrs({ 'data-action': 'close-overlay', 'data-testid': 'report-close' })}>${esc(de.report.close)}</button></div>`,
      true,
      'sheet-report',
    );
  }
  if (o.phase === 'queued') {
    return sheet(
      `<div ${attrs(state)}>
       <h2>${esc(de.report.queuedTitle)}</h2>
       <p>${esc(de.report.queuedText)}</p>
       <div class="sheet-actions">
         <button class="btn btn-primary" ${attrs({ 'data-action': 'close-overlay', 'data-testid': 'report-close' })}>${esc(de.report.close)}</button>
         <button class="btn" ${attrs({ 'data-action': 'report-text', 'data-testid': 'report-as-text' })}>${esc(de.report.asText)}</button>
       </div></div>`,
      true,
      'sheet-report',
    );
  }
  if (o.phase === 'text') {
    return sheet(
      `<div ${attrs(state)}>
       <h2>${esc(de.report.textTitle)}</h2>
       <p class="small">${esc(de.report.textHint)}</p>
       <textarea class="save-code" readonly rows="6" spellcheck="false" ${attrs({ 'data-testid': 'report-json', 'aria-label': de.report.textTitle })}>${esc(o.upload ?? '')}</textarea>
       <div class="sheet-actions">
         <button class="btn btn-primary" ${attrs({ 'data-action': 'report-copy', 'data-testid': 'report-copy' })}>${esc(de.report.copy)}</button>
         <button class="btn" ${attrs({ 'data-action': 'report-back' })}>${esc(de.report.back)}</button>
       </div>
       ${copyStatus(o.copied)}</div>`,
      true,
      'sheet-report',
    );
  }
  const busy = o.phase === 'sending' || o.phase === 'building';
  const items = de.report.what.map((line) => `<li>${esc(line)}</li>`).join('');
  return sheet(
    `<div ${attrs(state)}>
     <h2>${esc(de.report.title)}</h2>
     <p>${esc(de.report.intro)}</p>
     <label class="report-label" for="report-text">${esc(de.report.label)}</label>
     <textarea id="report-text" class="save-code save-input report-text" rows="4" maxlength="${MESSAGE_CHARS}" ${attrs({ placeholder: de.report.placeholder, 'data-testid': 'report-text', disabled: busy })}>${esc(o.text)}</textarea>
     <label class="report-check"><input type="checkbox" id="report-save" ${attrs({ 'data-testid': 'report-save', checked: o.withSave, disabled: busy })}> <span>${esc(de.report.withSave)}</span></label>
     <details class="report-what" data-testid="report-what"><summary>${esc(de.report.whatTitle)}</summary><ul>${items}</ul><p class="small">${esc(de.report.notSent)}</p></details>
     ${hadTroubleBefore ? `<p class="small" data-testid="report-previous">${esc(de.report.previous)}</p>` : ''}
     ${o.error ? `<p class="is-warning" role="alert" data-testid="report-error">${esc(o.error)}</p>` : ''}
     <div class="sheet-actions">
       <button class="btn btn-primary" ${attrs({ 'data-action': 'report-send', 'data-testid': 'report-send', disabled: busy })}>${esc(o.phase === 'sending' ? de.report.sending : de.report.send)}</button>
       <button class="btn" ${attrs({ 'data-action': 'report-text', 'data-testid': 'report-as-text', disabled: busy })}>${esc(de.report.asText)}</button>
     </div>
     <button class="btn btn-wide" ${attrs({ 'data-action': 'close-overlay' })}>${esc(de.report.cancel)}</button></div>`,
    true,
    'sheet-report',
  );
}

/** "Fehler melden" with its count of reports waiting, if any. */
export function reportButtonLabel(waiting: number): string {
  return waiting > 0 ? t(de.report.menuWaiting, { n: waiting }) : de.report.menu;
}

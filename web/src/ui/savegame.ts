// Dialogs for the save code (../savecode.ts): show it to copy or share,
// paste one, and confirm replacing the current progress with it.

import { de, t } from '../i18n';
import { attrs, esc } from './dom';

type Sheet = (inner: string, closable?: boolean, cls?: string, overlayCls?: string) => string;

/** Whether the system share sheet exists here. */
function canShare(): boolean {
  try {
    return typeof navigator.share === 'function';
  } catch {
    return false;
  }
}

/** The small panel behind the menu button "Spielstand". */
export function renderSaveMenu(sheet: Sheet): string {
  return sheet(
    `<h2>${esc(de.save.title)}</h2>
     <p>${esc(de.save.why)}</p>
     <div class="menu-list">
       <button class="btn btn-primary btn-wide" ${attrs({ 'data-action': 'save-export', 'data-testid': 'save-export' })}>${esc(de.save.export)}</button>
       <button class="btn btn-wide" ${attrs({ 'data-action': 'save-import', 'data-testid': 'save-import' })}>${esc(de.save.import)}</button>
       <button class="btn btn-wide" ${attrs({ 'data-action': 'menu' })}>${esc(de.save.back)}</button>
     </div>`,
    true,
    'sheet-save',
  );
}

export function renderSaveExport(sheet: Sheet, code: string, copied: boolean | null): string {
  const status =
    copied === true
      ? `<p class="share-ok" role="status" data-testid="save-copied">${esc(de.save.copied)}</p>`
      : copied === false
        ? `<p class="small is-warning" role="status" data-testid="save-copy-failed">${esc(de.save.copyFailed)}</p>`
        : '';
  const share = canShare() ? `<button class="btn" ${attrs({ 'data-action': 'save-share' })}>${esc(de.save.share)}</button>` : '';
  return sheet(
    `<h2>${esc(de.save.export)}</h2>
     <p>${esc(de.save.exportText)}</p>
     <p class="small">${esc(de.save.exportWhy)}</p>
     <textarea class="save-code" readonly rows="5" spellcheck="false" ${attrs({
       'data-action': 'save-select',
       'data-testid': 'save-code',
       'aria-label': de.save.codeLabel,
     })}>${esc(code)}</textarea>
     <p class="small save-length">${esc(t(de.save.length, { n: code.length.toLocaleString('de-DE') }))}</p>
     <div class="sheet-actions">
       <button class="btn btn-primary" ${attrs({ 'data-action': 'save-copy', 'data-testid': 'save-copy' })}>${esc(de.save.copy)}</button>
       ${share}
     </div>
     ${status}
     <button class="btn btn-wide" ${attrs({ 'data-action': 'save-menu' })}>${esc(de.save.back)}</button>`,
    true,
    'sheet-save',
  );
}

export function renderSaveImport(sheet: Sheet, text: string, error: string | null, checking: boolean): string {
  return sheet(
    `<h2>${esc(de.save.import)}</h2>
     <p>${esc(de.save.importText)}</p>
     <textarea id="save-input" class="save-code save-input" rows="6" spellcheck="false" autocomplete="off" autocapitalize="off" ${attrs({
       autocorrect: 'off',
       placeholder: de.save.placeholder,
       'aria-label': de.save.codeLabel,
       'data-testid': 'save-input',
     })}>${esc(text)}</textarea>
     ${error ? `<p class="is-warning" role="alert" data-testid="save-error">${esc(error)}</p>` : ''}
     <div class="sheet-actions">
       <button class="btn btn-primary" ${attrs({ 'data-action': 'save-check', 'data-testid': 'save-check', disabled: checking })}>${esc(checking ? de.save.checking : de.save.check)}</button>
       <button class="btn" ${attrs({ 'data-action': 'save-menu' })}>${esc(de.save.back)}</button>
     </div>`,
    true,
    'sheet-save',
  );
}

export function renderSaveConfirm(sheet: Sheet, lines: string[]): string {
  const items = lines.map((l) => `<li>${esc(l)}</li>`).join('');
  return sheet(
    `<h2>${esc(de.save.confirmTitle)}</h2>
     <p>${esc(de.save.confirmText)}</p>
     <section class="share save-contains" data-testid="save-contains">
       <div class="share-head"><b>${esc(de.save.contains)}</b></div>
       <ul>${items}</ul>
     </section>
     <p class="small">${esc(de.save.confirmHint)}</p>
     <div class="sheet-actions">
       <button class="btn btn-primary" ${attrs({ 'data-action': 'save-apply', 'data-testid': 'save-apply' })}>${esc(de.save.go)}</button>
       <button class="btn" ${attrs({ 'data-action': 'close-overlay' })}>${esc(de.save.cancel)}</button>
     </div>`,
    true,
    'sheet-confirm',
  );
}

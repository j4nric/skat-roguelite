// The move to the new address (ADR 0025, ../app/move.ts): the notice on
// the start screen of the old address, its sheet with the link, and on
// the new address the sheet after a move link signed the account in.

import { HOME_HOST } from '../domain';
import { de, t } from '../i18n';
import { attrs, esc } from './dom';

type Sheet = (inner: string, closable?: boolean, cls?: string, overlayCls?: string) => string;

/** A text with `{host}`, the host kept on one line. */
function withHost(text: string): string {
  return esc(t(text, { host: '\u0000' })).replace('\u0000', `<span class="move-host">${esc(HOME_HOST)}</span>`);
}

/** "Wir sind umgezogen" on the start screen (only on the old address). */
export function renderMoveNotice(): string {
  return `<section class="shared-offer move-notice" data-testid="move-notice">
    <div class="shared-head"><h2>${esc(de.move.title)}</h2></div>
    <p>${withHost(de.move.text)}</p>
    <div class="sheet-actions">
      <button class="btn btn-primary" ${attrs({ 'data-action': 'move-open', 'data-testid': 'move-open' })}>${esc(de.move.open)}</button>
    </div>
  </section>`;
}

export interface MoveSheet {
  busy: boolean;
  error: string | null;
  link: string | null;
  what: 'account' | 'save' | null;
  copied: boolean | null;
}

/** The sheet on the old address: make the link, then open or copy it. */
export function renderMove(sheet: Sheet, o: MoveSheet, signedIn: boolean, online: boolean): string {
  const error = o.error ? `<p class="is-warning" role="alert" data-testid="move-error">${esc(o.error)}</p>` : '';
  if (!o.link) {
    const lead = signedIn ? de.move.accountText : de.move.saveText;
    const make = signedIn ? de.move.makeAccount : de.move.makeSave;
    const disabled = o.busy || (signedIn && !online) ? 'disabled' : null;
    return sheet(
      `<h2>${esc(de.move.title)}</h2>
       <div class="sheet-body" data-testid="move-sheet" ${attrs({ 'data-state': signedIn ? 'account' : 'save' })}>
         <p>${withHost(de.move.why)}</p>
         <p>${esc(lead)}</p>
         ${signedIn ? '' : `<p class="small">${esc(de.move.haveAccount)}</p>`}
         ${error}
         <div class="menu-list">
           <button class="btn btn-primary btn-wide" ${attrs({ 'data-action': 'move-make', 'data-testid': 'move-make', disabled })}>${esc(o.busy ? de.account.working : make)}</button>
           ${signedIn ? '' : `<button class="btn btn-wide" ${attrs({ 'data-action': 'account', 'data-testid': 'move-signin' })}>${esc(de.move.signIn)}</button>`}
         </div>
         <p class="small">${esc(de.move.stays)}</p>
       </div>
       <button class="btn btn-wide" ${attrs({ 'data-action': 'close-overlay' })}>${esc(de.app.close)}</button>`,
      true,
      'sheet-account has-body',
    );
  }
  const status =
    o.copied === true
      ? `<p class="share-ok" role="status">${esc(de.move.copied)}</p>`
      : o.copied === false
        ? `<p class="small is-warning" role="status">${esc(de.move.copyFailed)}</p>`
        : '';
  return sheet(
    `<h2>${esc(de.move.title)}</h2>
     <div class="sheet-body" data-testid="move-sheet" data-state="link">
       <p>${esc(o.what === 'account' ? de.move.linkAccount : de.move.linkSave)}</p>
       <a class="btn btn-primary btn-wide" ${attrs({ href: o.link, rel: 'noopener', 'data-testid': 'move-go' })}>${esc(t(de.move.go, { host: HOME_HOST }))}</a>
       <div class="sheet-actions">
         <button class="btn" ${attrs({ 'data-action': 'move-copy', 'data-testid': 'move-copy' })}>${esc(de.move.copy)}</button>
       </div>
       ${status}
       <p class="small">${esc(o.what === 'account' ? de.move.linkAccountHint : de.move.linkSaveHint)}</p>
     </div>
     <button class="btn btn-wide" ${attrs({ 'data-action': 'close-overlay' })}>${esc(de.app.close)}</button>`,
    true,
    'sheet-account has-body',
  );
}

/** On the new address after a move link: the account is here, now a passkey for this address. */
export function renderAccountMoved(sheet: Sheet, busy: boolean, error: string | null, done: boolean, name: string, canAddPasskey: boolean): string {
  if (!done) {
    const body = busy
      ? `<p role="status">${esc(de.account.working)}</p>`
      : `<p class="is-warning" role="alert" data-testid="moved-error">${esc(error ?? '')}</p>
         <p class="small">${esc(de.move.failedHint)}</p>
         <div class="menu-list">
           <button class="btn btn-wide" ${attrs({ 'data-action': 'account-recover' })}>${esc(de.account.recover)}</button>
         </div>`;
    return sheet(
      `<h2>${esc(de.move.arrivedTitle)}</h2>
       <div class="sheet-body" data-testid="moved-sheet" ${attrs({ 'data-state': busy ? 'busy' : 'failed' })}>${body}</div>
       <button class="btn btn-wide" ${attrs({ 'data-action': 'close-overlay', disabled: busy ? 'disabled' : null })}>${esc(de.app.close)}</button>`,
      !busy,
      'sheet-account has-body',
    );
  }
  const add = canAddPasskey
    ? `<p class="small">${esc(t(de.move.passkeyHint, { host: HOME_HOST }))}</p>
       <button class="btn btn-primary btn-wide" ${attrs({ 'data-action': 'account-add-passkey', 'data-testid': 'moved-add-passkey' })}>${esc(de.account.codeAddPasskey)}</button>`
    : '';
  return sheet(
    `<h2>${esc(de.move.arrivedTitle)}</h2>
     <div class="sheet-body" data-testid="moved-sheet" data-state="done">
       <p>${esc(t(de.move.arrivedText, { name, host: HOME_HOST }))}</p>
       ${add}
     </div>
     <button class="btn btn-wide" ${attrs({ 'data-action': 'account-moved-done', 'data-testid': 'moved-done' })}>${esc(de.move.later)}</button>`,
    false,
    'sheet-account has-body',
  );
}

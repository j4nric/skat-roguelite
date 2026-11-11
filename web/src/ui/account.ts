// Dialogs of the account (ADR 0021): sign up, sign in with a passkey or
// the recovery code, the recovery code shown once, name, sign out, delete,
// and the choice between two states after playing on two devices.

import { de, locale, t } from '../i18n';
import type { AccountUser, PasskeyInfo } from '../account';
import type { SyncStatus } from '../sync';
import { attrs, esc } from './dom';
import { HOME_HOST, OLD_HOSTS } from '../domain';

/** Whether the page runs on the new address (ADR 0025). */
function onHomeAddress(): boolean {
  try {
    return window.location.hostname === HOME_HOST;
  } catch {
    return false;
  }
}

type Sheet = (inner: string, closable?: boolean, cls?: string, overlayCls?: string) => string;

/** One side of a sync conflict, described by the app (with the core). */
export interface ConflictSide {
  at: number;
  lines: string[];
}

/** The account as the screens see it. */
export interface AccountView {
  user: AccountUser | null;
  status: SyncStatus;
  supported: boolean;
  /** The browser is online; offline every button that needs the server is off. */
  online: boolean;
}

function notice(error: string | null, note: string | null): string {
  if (error) return `<p class="is-warning" role="alert" data-testid="account-error">${esc(error)}</p>`;
  if (note) return `<p class="share-ok" role="status" data-testid="account-note">${esc(note)}</p>`;
  return '';
}

/**
 * The small status word next to the account ("gespeichert", "offline",
 * "Konflikt"); a conflict opens the choice when tapped (`interactive`,
 * not inside another button).
 */
export function syncPill(status: SyncStatus, interactive = true): string {
  const text = de.account.status[status] ?? '';
  if (!text) return '';
  const label = t(de.account.statusLabel, { status: text });
  if (status === 'conflict' && interactive) {
    return `<button class="sync-pill is-${status}" ${attrs({ 'data-action': 'sync-conflict', 'data-testid': 'sync-status', 'data-status': status, 'aria-label': label })}>${esc(text)}</button>`;
  }
  return `<span class="sync-pill is-${status}" ${attrs({ 'data-testid': 'sync-status', 'data-status': status, title: label })}>${esc(text)}</span>`;
}

/** The account's name, or "Namenlos". */
export function accountName(user: AccountUser): string {
  return user.name ?? de.account.noName;
}

export function renderAccount(sheet: Sheet, account: AccountView, busy: boolean, error: string | null, note: string | null): string {
  const user = account.user;
  const disabled = busy || !account.online ? 'disabled' : null;
  const offline = account.online ? '' : `<p class="small is-warning" role="status" data-testid="account-offline">${esc(de.offline.account)}</p>`;
  if (!user) {
    const create = account.supported
      ? `<label class="account-field"><span>${esc(de.account.nameLabel)}</span>
           <input id="account-name" class="account-input" maxlength="24" autocomplete="nickname" ${attrs({ placeholder: de.account.namePlaceholder, 'data-testid': 'account-name' })}></label>
         <button class="btn btn-primary btn-wide" ${attrs({ 'data-action': 'account-create', 'data-testid': 'account-create', disabled })}>${esc(busy ? de.account.working : de.account.create)}</button>
         <button class="btn btn-wide" ${attrs({ 'data-action': 'account-signin', 'data-testid': 'account-signin', disabled })}>${esc(de.account.signIn)}</button>`
      : `<p class="small is-warning">${esc(de.account.noPasskeys)}</p>`;
    return sheet(
      `<h2>${esc(de.account.title)}</h2>
       <div class="sheet-body account-body" data-testid="account-sheet" data-state="out">
         <p>${esc(de.account.why)}</p>
         <p class="small">${esc(de.account.how)}</p>
         ${offline}
         ${notice(error, note)}
         <div class="menu-list">
           ${create}
           <button class="btn btn-wide" ${attrs({ 'data-action': 'account-recover', 'data-testid': 'account-recover', disabled })}>${esc(de.account.recover)}</button>
         </div>
         ${onHomeAddress() ? `<p class="small" data-testid="account-old-hint">${esc(t(de.account.oldAddress, { host: OLD_HOSTS[0] ?? '' }))}</p>` : ''}
         <p class="small">${esc(de.account.stored)}</p>
       </div>
       <button class="btn btn-wide" ${attrs({ 'data-action': 'menu' })}>${esc(de.account.back)}</button>`,
      true,
      'sheet-account has-body',
    );
  }
  return sheet(
    `<h2>${esc(de.account.title)}</h2>
     <div class="sheet-body account-body" data-testid="account-sheet" data-state="in">
       <p class="account-who"><span data-testid="account-who">${esc(t(de.account.signedInAs, { name: accountName(user) }))}</span> ${syncPill(account.status)}</p>
       ${account.status === 'outdated' ? `<p class="small is-warning">${esc(de.account.outdatedText)}</p>` : ''}
       ${offline}
       ${notice(error, note)}
       <label class="account-field"><span>${esc(de.account.nameLabel)}</span>
         <span class="account-name-row">
           <input id="account-name" class="account-input" maxlength="24" autocomplete="nickname" ${attrs({ value: user.name ?? '', placeholder: de.account.noName, 'data-testid': 'account-name' })}>
           <button class="btn" ${attrs({ 'data-action': 'account-name-save', 'data-testid': 'account-name-save', disabled })}>${esc(de.account.nameSave)}</button>
         </span></label>
       <p class="small">${esc(de.account.nameHint)}</p>
       ${user.passkeys ? `<p class="small" data-testid="account-passkeys">${esc(t(de.account.passkeys, { n: user.passkeys }))}</p>` : ''}
       <div class="menu-list menu-grid account-grid">
         ${account.supported ? `<button class="btn" ${attrs({ 'data-action': 'account-add-passkey', 'data-testid': 'account-add-passkey', disabled })}>${esc(de.account.addPasskey)}</button>` : ''}
         <button class="btn" ${attrs({ 'data-action': 'account-passkeys', 'data-testid': 'account-passkeys-open', disabled })}>${esc(de.account.passkeysManage)}</button>
         <button class="btn" ${attrs({ 'data-action': 'account-new-code', 'data-testid': 'account-new-code', disabled })}>${esc(de.account.newCode)}</button>
         <button class="btn" ${attrs({ 'data-action': 'account-signout', 'data-testid': 'account-signout', disabled })}>${esc(de.account.signOut)}</button>
         <button class="btn" ${attrs({ 'data-action': 'account-signout-all', 'data-testid': 'account-signout-all', disabled })}>${esc(de.account.signOutAll)}</button>
         <button class="btn btn-danger" ${attrs({ 'data-action': 'account-delete', 'data-testid': 'account-delete', disabled })}>${esc(de.account.delete)}</button>
       </div>
     </div>
     <button class="btn btn-wide" ${attrs({ 'data-action': 'menu' })}>${esc(de.account.back)}</button>`,
    true,
    'sheet-account has-body',
  );
}

/**
 * The account's passkeys: when each was added and last used, whether a
 * password manager syncs it, and a button to remove it (a lost phone, one
 * somebody else added). The last one cannot be removed. `list` is null
 * while it loads.
 */
export function renderPasskeys(sheet: Sheet, list: PasskeyInfo[] | null, busy: boolean, error: string | null, note: string | null): string {
  const only = (list?.length ?? 0) <= 1;
  const rows = list
    ? list
        .map((p, i) => {
          const used = p.lastUsedAt ? t(de.account.passkeyUsed, { date: when(p.lastUsedAt) }) : de.account.passkeyNeverUsed;
          const synced = p.synced ? `<span class="recent-daily">${esc(de.account.passkeySynced)}</span>` : '';
          // Since the move (ADR 0025) a passkey opens either the new or the old address.
          const here = typeof window === 'undefined' ? '' : window.location.hostname;
          const where = p.rpId && p.rpId !== here ? ` · ${esc(t(de.account.passkeyFor, { host: p.rpId }))}` : '';
          return `<section class="share" ${attrs({ 'data-testid': 'passkey-row', 'data-id': p.id })}>
            <div class="share-head"><b>${esc(t(de.account.passkeyName, { n: i + 1 }))}</b>${synced}</div>
            <p class="small">${esc(t(de.account.passkeyCreated, { date: when(p.createdAt) }))} · ${esc(used)}${where}</p>
            <div class="share-actions">
              <button class="btn btn-small" ${attrs({ 'data-action': 'account-passkey-remove', 'data-id': p.id, 'data-testid': 'passkey-remove', disabled: busy || only ? 'disabled' : null })}>${esc(de.account.passkeyRemove)}</button>
            </div>
          </section>`;
        })
        .join('')
    : `<p class="small" role="status">${esc(de.account.working)}</p>`;
  return sheet(
    `<h2>${esc(de.account.passkeysTitle)}</h2>
     <div class="sheet-body account-body" data-testid="passkeys-sheet">
       <p class="small">${esc(de.account.passkeysText)}</p>
       ${notice(error, note)}
       ${rows}
       ${list && only ? `<p class="small">${esc(de.account.passkeysLast)}</p>` : ''}
     </div>
     <button class="btn btn-wide" ${attrs({ 'data-action': 'account', 'data-testid': 'passkeys-back' })}>${esc(de.account.back)}</button>`,
    true,
    'sheet-account has-body',
  );
}

export function renderRecoveryCode(sheet: Sheet, code: string, why: 'new' | 'recovered' | 'renewed', copied: boolean | null, canAddPasskey: boolean): string {
  const lead = why === 'new' ? de.account.codeNew : why === 'recovered' ? de.account.codeRecovered : de.account.codeRenewed;
  const status =
    copied === true
      ? `<p class="share-ok" role="status">${esc(de.account.codeCopied)}</p>`
      : copied === false
        ? `<p class="small is-warning" role="status">${esc(de.account.codeCopyFailed)}</p>`
        : '';
  const addPasskey =
    why === 'recovered' && canAddPasskey
      ? `<p class="small">${esc(de.account.codeAddHint)}</p>
         <button class="btn btn-primary btn-wide" ${attrs({ 'data-action': 'account-add-passkey', 'data-testid': 'code-add-passkey' })}>${esc(de.account.codeAddPasskey)}</button>`
      : '';
  return sheet(
    `<h2>${esc(de.account.codeTitle)}</h2>
     <div class="sheet-body">
       <p>${esc(lead)}</p>
       <p>${esc(de.account.codeText)}</p>
       <p class="recovery-code" data-testid="recovery-code">${esc(code)}</p>
       <p class="is-warning account-write">${esc(de.account.codeWrite)}</p>
       <div class="sheet-actions">
         <button class="btn" ${attrs({ 'data-action': 'account-code-copy', 'data-testid': 'code-copy' })}>${esc(de.account.codeCopy)}</button>
       </div>
       ${status}
       ${addPasskey}
     </div>
     <button class="btn ${addPasskey ? '' : 'btn-primary'} btn-wide" ${attrs({ 'data-action': 'account-code-done', 'data-testid': 'code-done' })}>${esc(de.account.codeDone)}</button>`,
    false,
    'sheet-account sheet-code has-body',
  );
}

export function renderRecover(sheet: Sheet, busy: boolean, error: string | null): string {
  return sheet(
    `<h2>${esc(de.account.recoverTitle)}</h2>
     <p>${esc(de.account.recoverText)}</p>
     <input id="recover-code" class="account-input recover-input" autocomplete="off" autocapitalize="characters" spellcheck="false" ${attrs({
       autocorrect: 'off',
       placeholder: de.account.recoverPlaceholder,
       'aria-label': de.account.codeTitle,
       'data-testid': 'recover-input',
     })}>
     ${notice(error, null)}
     <div class="sheet-actions">
       <button class="btn btn-primary" ${attrs({ 'data-action': 'account-recover-go', 'data-testid': 'recover-go', disabled: busy ? 'disabled' : null })}>${esc(busy ? de.account.working : de.account.recoverGo)}</button>
       <button class="btn" ${attrs({ 'data-action': 'account' })}>${esc(de.account.back)}</button>
     </div>`,
    true,
    'sheet-account',
  );
}

export function renderDeleteAccount(sheet: Sheet, busy: boolean, error: string | null): string {
  return sheet(
    `<h2>${esc(de.account.deleteTitle)}</h2>
     <p>${esc(de.account.deleteText)}</p>
     <p class="small">${esc(de.account.deleteKeep)}</p>
     ${notice(error, null)}
     <div class="sheet-actions">
       <button class="btn btn-danger" ${attrs({ 'data-action': 'account-delete-go', 'data-testid': 'account-delete-go', disabled: busy ? 'disabled' : null })}>${esc(busy ? de.account.working : de.account.deleteGo)}</button>
       <button class="btn" ${attrs({ 'data-action': 'account' })}>${esc(de.account.back)}</button>
     </div>`,
    true,
    'sheet-confirm',
  );
}

function when(at: number): string {
  if (!at) return '';
  const d = new Date(at);
  return d.toLocaleString(locale(), { day: '2-digit', month: '2-digit', year: 'numeric', hour: '2-digit', minute: '2-digit' });
}

function side(title: string, s: ConflictSide, newer: boolean, action: string, testid: string, label: string): string {
  const items = s.lines.map((l) => `<li>${esc(l)}</li>`).join('');
  return `<section class="share conflict-side${newer ? ' is-newer' : ''}" ${attrs({ 'data-testid': testid })}>
    <div class="share-head"><b>${esc(title)}</b>${newer ? `<span class="recent-daily">${esc(de.account.conflictNewer)}</span>` : ''}</div>
    <p class="small">${esc(t(de.account.conflictAt, { date: when(s.at) }))}</p>
    <ul>${items}</ul>
    <button class="btn btn-wide" ${attrs({ 'data-action': action, 'data-testid': `${testid}-keep` })}>${esc(label)}</button>
  </section>`;
}

export function renderConflict(sheet: Sheet, here: ConflictSide, server: ConflictSide): string {
  return sheet(
    `<h2>${esc(de.account.conflictTitle)}</h2>
     <div class="sheet-body" data-testid="sync-conflict">
       <p>${esc(de.account.conflictText)}</p>
       <div class="conflict-sides">
         ${side(de.account.conflictHere, here, here.at > server.at, 'sync-keep-here', 'conflict-here', de.account.keepHere)}
         ${side(de.account.conflictServer, server, server.at >= here.at, 'sync-keep-server', 'conflict-server', de.account.keepServer)}
       </div>
     </div>
     <button class="btn btn-wide" ${attrs({ 'data-action': 'close-overlay' })}>${esc(de.account.conflictLater)}</button>`,
    true,
    'sheet-account has-body',
  );
}

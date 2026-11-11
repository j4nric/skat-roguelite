// Account and sync (ADR 0021), and the daily board. Transport only: the
// save is the save code; the server never sees the game.

import { AccountError, addPasskey, deleteAccount, fetchDailyBoard, fetchMe, fetchPasskeys, fetchWeeklyBoard, recover, redeemMove, register, removePasskey, renewRecoveryCode, setName, signIn, signOut, signOutEverywhere, submitDaily, submitWeekly, type AccountUser } from '../account';
import { de, t } from '../i18n';
import { dailySeed } from '../seed';
import { Sync, type Reconciled } from '../sync';
import type { RunView } from '../types';
import { syncPill } from '../ui/account';
import type { Overlay } from '../ui/state';
import { copyText } from './consts';
import type { Actions, AppCtx } from './context';
import type { Layer } from './layer';
import { describeKeys, isFreshMeta } from './savecode';

/** Session flag: the page was reloaded to take the account's state (a note after the reload). */
const LOADED_NOTE = 'stammtisch-sync-loaded';

/** Whether the account's state was just taken (read once). */
export function takeLoadedNote(): boolean {
  try {
    const set = window.sessionStorage.getItem(LOADED_NOTE) === '1';
    window.sessionStorage.removeItem(LOADED_NOTE);
    return set;
  } catch {
    return false;
  }
}

/** The account's sheets that show "Einen Moment …" and an error of their own. */
type AccountSheet = Extract<Overlay, { kind: 'account' | 'account-recover' | 'account-delete' | 'account-passkeys' }>;

function isAccountSheet(o: Overlay | null): o is AccountSheet {
  return o?.kind === 'account' || o?.kind === 'account-recover' || o?.kind === 'account-delete' || o?.kind === 'account-passkeys';
}

export class AccountFlow {
  /** The account and the sync of the stored progress (ADR 0021). */
  readonly sync: Sync;
  /** The day whose daily board was asked for in this visit. */
  private boardDay = '';
  /** The week whose weekly board was asked for in this visit. */
  private boardWeek = '';
  /** What signing in with the recovery code brought, acted on once its code was seen. */
  private pendingSync: Reconciled = 'none';

  constructor(
    private readonly ctx: AppCtx,
    private readonly layer: Layer,
  ) {
    this.sync = new Sync({ readable: async (keys) => (await describeKeys(keys)) !== null, freshMeta: (meta) => isFreshMeta(meta), changed: () => this.accountChanged() });
    ctx.ui.account.user = this.sync.user;
    ctx.ui.account.status = this.sync.status;
  }

  /** The sync changed: show the new status (and the account, if it changed). */
  private accountChanged(): void {
    const ui = this.ctx.ui;
    const account = ui.account;
    const userChanged = account.user !== this.sync.user;
    account.user = this.sync.user;
    account.status = this.sync.status;
    if (this.sync.status === 'conflict' && ui.overlay?.kind !== 'sync-conflict' && ui.screen === 'run') {
      this.ctx.showToast(de.account.conflictToast);
    }
    if (userChanged && !ui.busy && ui.screen !== 'loading') {
      this.layer.markDirty();
      this.ctx.render();
      return;
    }
    // Only the small status word: no re-render (a name may be being typed).
    this.ctx.root.querySelectorAll<HTMLElement>('[data-testid="sync-status"]').forEach((el) => {
      el.outerHTML = syncPill(account.status, !el.parentElement?.closest('button'));
    });
  }

  /** Online or offline (DESIGN 14.5): the account buttons and the daily board follow. */
  networkChanged(online: boolean): void {
    const ui = this.ctx.ui;
    ui.account.online = online;
    if (online) this.loadBoard(true);
    if (!ui.busy && (ui.screen === 'start' || isAccountSheet(ui.overlay))) {
      this.layer.markDirty();
      this.ctx.render();
    }
  }

  /** Ask the server who is signed in (the name may have changed elsewhere). */
  async refreshAccount(): Promise<void> {
    if (!this.sync.signedIn) return;
    try {
      const user = await fetchMe();
      if (user) this.sync.updateUser(user);
      else this.sync.forget();
    } catch {
      // Offline: the status says so.
    }
  }

  /** Today's daily board and this week's, once per visit and period (and again after a result). */
  loadBoard(force = false): void {
    this.loadWeeklyBoard(force);
    const day = dailySeed();
    if (!force && this.boardDay === day) return;
    this.boardDay = day;
    fetchDailyBoard(day).then(
      (board) => {
        this.ctx.ui.board = board;
        if (this.ctx.ui.screen === 'start' && !this.ctx.ui.busy) this.ctx.render();
      },
      () => {
        // No server (local preview, offline): no board.
        this.ctx.ui.board = null;
      },
    );
  }

  /** This week's board (DESIGN 5.6), once per visit and week. */
  private loadWeeklyBoard(force: boolean): void {
    const week = this.ctx.ui.week ? String(this.ctx.ui.week.week) : '';
    if (!week || (!force && this.boardWeek === week)) return;
    this.boardWeek = week;
    fetchWeeklyBoard(week).then(
      (board) => {
        this.ctx.ui.weeklyBoard = board;
        const sheet = this.ctx.ui.overlay?.kind === 'weekly-board';
        if ((this.ctx.ui.screen === 'start' || sheet) && !this.ctx.ui.busy) this.ctx.render();
      },
      () => {
        this.ctx.ui.weeklyBoard = null;
      },
    );
  }

  /**
   * A finished weekly run of a signed-in player goes to the Tafel der Woche:
   * the first of the week counts, every later one is practice and the
   * server keeps it off the board (as the daily board does).
   */
  submitWeeklyResult(view: RunView, week: number): void {
    if (!this.sync.signedIn || view.tutorial) return;
    submitWeekly({
      week: String(week),
      character: view.character.character,
      won: view.phase === 'Won',
      section: view.series.section + 1,
      series: view.series.index + 1,
      stake: view.stake ?? 0,
      bestGame: view.stats.best_game,
    }).then(
      (res) => {
        if (res.stored && res.me) this.ctx.showToast(t(de.weeklyBoard.submitted, { rank: res.me.rank }));
        this.loadWeeklyBoard(true);
      },
      (err: unknown) => console.warn('weekly result not sent', err),
    );
  }

  /** A finished daily run of a signed-in player goes on the board. */
  submitDailyResult(view: RunView): void {
    // The Einführung (DESIGN 16.1) never goes on a board.
    if (!this.sync.signedIn || view.tutorial) return;
    submitDaily({
      day: view.seed,
      character: view.character.character,
      won: view.phase === 'Won',
      section: view.series.section + 1,
      series: view.series.index + 1,
      stake: view.stake ?? 0,
      bestGame: view.stats.best_game,
    }).then(
      (res) => {
        if (res.stored && res.me) this.ctx.showToast(t(de.board.submitted, { rank: res.me.rank }));
        this.loadBoard(true);
      },
      (err: unknown) => console.warn('daily result not sent', err),
    );
  }

  accountError(err: unknown): string {
    const code = err instanceof AccountError ? err.code : 'default';
    return de.account.errors[code] ?? de.account.errors.default ?? '';
  }

  openAccount(note: string | null = null, error: string | null = null): void {
    this.ctx.ui.overlay = { kind: 'account', busy: false, error, note };
    this.ctx.render();
  }

  /** Run an account call with the sheet showing "Einen Moment …". */
  private async accountBusy<T>(work: () => Promise<T>): Promise<T | null> {
    const ui = this.ctx.ui;
    const o = ui.overlay;
    if (isAccountSheet(o)) {
      if (o.busy) return null;
      ui.overlay = { ...o, busy: true, error: null };
      this.ctx.render();
    }
    try {
      return await work();
    } catch (err) {
      const now = ui.overlay;
      const error = this.accountError(err);
      if (isAccountSheet(now)) {
        ui.overlay = { ...now, busy: false, error };
        this.ctx.render();
      } else {
        this.ctx.showToast(error);
      }
      return null;
    }
  }

  /**
   * Signed in: compare the stored progress with the account. A state from
   * the account is taken by reloading the page (after the recovery code
   * was seen); a conflict asks the player.
   */
  private async afterSignIn(user: AccountUser): Promise<Reconciled> {
    this.sync.signedInAs(user);
    const result = await this.sync.reconcileAfterSignIn();
    this.loadBoard(true);
    return result;
  }

  /**
   * Signing in found the account's state to take: into storage now, then
   * reload. False (with a note) if the storage refused.
   */
  private takeAccountState(): boolean {
    if (this.sync.takeServer()) return true;
    this.ctx.showToast(de.save.errors.write ?? '');
    return false;
  }

  /** Reload to start from the account's state, with a note after the reload. */
  private reloadWithAccountState(): void {
    try {
      window.sessionStorage.setItem(LOADED_NOTE, '1');
    } catch {
      // The note is lost, the state is not.
    }
    window.location.reload();
  }

  async createAccount(): Promise<void> {
    const name = this.ctx.root.querySelector<HTMLInputElement>('#account-name')?.value.trim() ?? '';
    const res = await this.accountBusy(() => register(name || null));
    if (!res) return;
    await this.afterSignIn(res.user);
    this.ctx.ui.overlay = { kind: 'account-code', code: res.recoveryCode, why: 'new', copied: null };
    this.ctx.render();
  }

  async signInWithPasskey(): Promise<void> {
    const res = await this.accountBusy(() => signIn());
    if (!res) return;
    const synced = await this.afterSignIn(res.user);
    if (synced === 'applied' && this.takeAccountState()) {
      this.reloadWithAccountState();
      return;
    }
    if (synced === 'conflict') {
      void this.openConflict();
      return;
    }
    this.openAccount(t(de.account.signedInAs, { name: res.user.name ?? de.account.noName }));
  }

  async recoverWithCode(): Promise<void> {
    const code = this.ctx.root.querySelector<HTMLInputElement>('#recover-code')?.value ?? '';
    const res = await this.accountBusy(() => recover(code));
    if (!res) return;
    this.pendingSync = await this.afterSignIn(res.user);
    this.ctx.ui.overlay = { kind: 'account-code', code: res.recoveryCode, why: 'recovered', copied: null };
    this.ctx.render();
  }

  /**
   * A move link from the old address (`#umzug=…`, ADR 0025): sign the
   * same account in with its one-time code, then offer a passkey for this
   * address (the old ones only open the old address). What the account
   * holds is taken as after any sign-in, once the sheet is closed.
   */
  async signInWithMoveCode(code: string): Promise<void> {
    const ui = this.ctx.ui;
    const pending: Overlay = { kind: 'account-moved', busy: true, error: null, done: false };
    ui.overlay = pending;
    this.ctx.render();
    let user: AccountUser;
    try {
      user = (await redeemMove(code)).user;
    } catch (err) {
      if (ui.overlay !== pending) return;
      ui.overlay = { kind: 'account-moved', busy: false, error: this.accountError(err), done: false };
      this.ctx.render();
      return;
    }
    this.pendingSync = await this.afterSignIn(user);
    ui.overlay = { kind: 'account-moved', busy: false, error: null, done: true };
    this.ctx.render();
  }

  /** The recovery code was seen: go on with what signing in brought. */
  codeDone(): void {
    const pending = this.pendingSync;
    this.pendingSync = 'none';
    if (pending === 'applied' && this.takeAccountState()) {
      this.reloadWithAccountState();
      return;
    }
    if (pending === 'conflict' && this.sync.conflict) {
      void this.openConflict();
      return;
    }
    this.openAccount();
  }

  copyRecoveryCode(): void {
    const ui = this.ctx.ui;
    const o = ui.overlay;
    if (o?.kind !== 'account-code') return;
    const done = (ok: boolean): void => {
      if (ui.overlay !== o) return;
      ui.overlay = { ...o, copied: ok };
      this.ctx.render();
    };
    copyText(o.code, done);
  }

  async addPasskeyHere(): Promise<void> {
    const kind = this.ctx.ui.overlay?.kind;
    const fromCode = kind === 'account-code' || kind === 'account-moved';
    if (fromCode) this.openAccount();
    const res = await this.accountBusy(() => addPasskey());
    if (!res) return;
    this.sync.updateUser(res.user);
    if (fromCode && this.pendingSync !== 'none') {
      this.codeDone();
      return;
    }
    this.openAccount(de.account.passkeyAdded);
  }

  async saveName(): Promise<void> {
    const name = this.ctx.root.querySelector<HTMLInputElement>('#account-name')?.value ?? '';
    const res = await this.accountBusy(() => setName(name));
    if (!res) return;
    this.sync.updateUser(res.user);
    this.loadBoard(true);
    this.openAccount(de.account.nameSaved);
  }

  async newRecoveryCode(): Promise<void> {
    const res = await this.accountBusy(() => renewRecoveryCode());
    if (!res) return;
    this.ctx.ui.overlay = { kind: 'account-code', code: res.recoveryCode, why: 'renewed', copied: null };
    this.ctx.render();
  }

  async signOutHere(): Promise<void> {
    // Upload what is new first; then the device forgets the account.
    const res = await this.accountBusy(async () => {
      this.ctx.persist();
      await this.sync.push();
      await signOut();
      return true;
    });
    if (!res) return;
    this.sync.forget();
    this.loadBoard(true);
    this.openAccount(de.account.signedOut);
  }

  /** Sign out on every device (a code or passkey somebody else may have). */
  async signOutEverywhereNow(): Promise<void> {
    const res = await this.accountBusy(async () => {
      await this.sync.push();
      await signOutEverywhere();
      return true;
    });
    if (!res) return;
    this.sync.forget();
    this.loadBoard(true);
    this.openAccount(de.account.signedOutAll);
  }

  /** The passkey list; `note` after a removal. */
  async openPasskeys(note: string | null = null): Promise<void> {
    const ui = this.ctx.ui;
    ui.overlay = { kind: 'account-passkeys', list: null, busy: true, error: null, note };
    this.ctx.render();
    let list = null;
    let error: string | null = null;
    try {
      list = await fetchPasskeys();
    } catch (err) {
      error = this.accountError(err);
    }
    if (ui.overlay?.kind !== 'account-passkeys') return;
    ui.overlay = { kind: 'account-passkeys', list: list ?? [], busy: false, error, note };
    this.ctx.render();
  }

  async removePasskeyNow(id: string): Promise<void> {
    const res = await this.accountBusy(() => removePasskey(id));
    if (!res) return;
    this.sync.updateUser(res.user);
    await this.openPasskeys(de.account.passkeyRemoved);
  }

  async deleteAccountNow(): Promise<void> {
    const res = await this.accountBusy(async () => {
      await deleteAccount();
      return true;
    });
    if (!res) return;
    this.sync.forget();
    this.loadBoard(true);
    this.openAccount(de.account.deleted);
  }

  /** Played on two devices: show both states and let the player choose. */
  async openConflict(): Promise<void> {
    const c = this.sync.conflict;
    if (!c) return;
    const [here, server] = await Promise.all([describeKeys(c.localKeys), describeKeys(c.serverKeys)]);
    // Settled meanwhile (another device, a choice): nothing to ask.
    if (this.sync.conflict !== c) return;
    this.ctx.ui.overlay = {
      kind: 'sync-conflict',
      here: { at: c.localAt, lines: here ?? [] },
      server: { at: c.server.updatedAt, lines: server ?? [] },
    };
    this.ctx.render();
  }

  async keepHere(): Promise<void> {
    this.ctx.ui.overlay = null;
    this.ctx.render();
    await this.sync.keepLocal();
    if (this.sync.status === 'saved') this.ctx.showToast(de.account.keptHere);
  }

  keepServer(): void {
    if (this.sync.takeServer()) this.reloadWithAccountState();
    else this.ctx.showToast(de.save.errors.write ?? '');
  }

  actions(): Actions {
    const ui = this.ctx.ui;
    return {
      'account': () => {
        if (ui.busy) return;
        this.openAccount();
      },
      'account-create': () => void this.createAccount(),
      'account-signin': () => void this.signInWithPasskey(),
      'account-recover': () => {
        ui.overlay = { kind: 'account-recover', busy: false, error: null };
        this.ctx.render();
      },
      'account-recover-go': () => void this.recoverWithCode(),
      'account-code-copy': () => this.copyRecoveryCode(),
      'account-code-done': () => this.codeDone(),
      'account-moved-done': () => this.codeDone(),
      'account-add-passkey': () => void this.addPasskeyHere(),
      'account-name-save': () => void this.saveName(),
      'account-new-code': () => void this.newRecoveryCode(),
      'account-signout': () => void this.signOutHere(),
      'account-signout-all': () => void this.signOutEverywhereNow(),
      'account-passkeys': () => {
        const overlay = ui.overlay;
        if (overlay?.kind === 'account' && overlay.busy) return;
        void this.openPasskeys();
      },
      'account-passkey-remove': ({ el }) => {
        if (el.dataset.id) void this.removePasskeyNow(el.dataset.id);
      },
      'account-delete': () => {
        ui.overlay = { kind: 'account-delete', busy: false, error: null };
        this.ctx.render();
      },
      'account-delete-go': () => void this.deleteAccountNow(),
      'sync-conflict': () => {
        if (ui.busy) return;
        void this.openConflict();
      },
      'sync-keep-here': () => void this.keepHere(),
      'sync-keep-server': () => this.keepServer(),
    };
  }
}

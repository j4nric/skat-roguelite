// The move to the new address (ADR 0025, ../domain.ts). On the old
// address the start screen says "Wir sind umgezogen"; its sheet makes the
// link that takes the account (a one-time move code, signed in) or the
// stored progress (the save code, without an account) to the new address.
// On the new address the link's fragment is read once at the start.

import { startMove } from '../account';
import { moveUrl, onOldAddress, parseMoveHash } from '../domain';
import { de } from '../i18n';
import { encodeSave } from '../savecode';
import { readAllKeys } from '../storage';
import type { AccountFlow } from './account';
import { copyText } from './consts';
import type { Actions, AppCtx } from './context';
import type { SaveCodeFlow } from './savecode';

/** Longest link we hand to the browser; a bigger save goes by copied text. */
const MAX_LINK = 1_500_000;

export class MoveFlow {
  constructor(
    private readonly ctx: AppCtx,
    private readonly account: AccountFlow,
    private readonly saveCode: SaveCodeFlow,
  ) {}

  /** Whether the start screen shows the notice (only on the old address). */
  get notice(): boolean {
    return onOldAddress();
  }

  /**
   * A move link opened here: take its code out of the address bar first
   * (it must not stay in the history or be shared on), then act on it.
   */
  readMoveHash(): void {
    const move = parseMoveHash(window.location.hash);
    if (!move) return;
    try {
      const { pathname, search } = window.location;
      window.history.replaceState(null, '', `${pathname}${search}`);
    } catch {
      // Some embedded browsers forbid it; the code is single-use anyway.
    }
    if (move.kind === 'account') void this.account.signInWithMoveCode(move.code);
    else void this.saveCode.offerImport(move.code);
  }

  open(): void {
    this.ctx.ui.overlay = { kind: 'move', busy: false, error: null, link: null, what: null, copied: null };
    this.ctx.render();
  }

  /** Make the link: with the account's move code when signed in, else with the save code. */
  async makeLink(): Promise<void> {
    const ui = this.ctx.ui;
    const o = ui.overlay;
    if (o?.kind !== 'move' || o.busy) return;
    const pending = { ...o, busy: true, error: null };
    ui.overlay = pending;
    this.ctx.render();
    let link: string;
    let what: 'account' | 'save';
    let error: string | null = null;
    try {
      this.ctx.persist();
      if (this.account.sync.signedIn) {
        what = 'account';
        // What is new here goes up first, so the new address finds it in the account.
        await this.account.sync.push();
        const status = this.account.sync.status;
        if (status === 'conflict' || status === 'outdated') throw new Error(status);
        link = moveUrl({ kind: 'account', code: (await startMove()).code });
      } else {
        what = 'save';
        link = moveUrl({ kind: 'save', code: await encodeSave(readAllKeys()) });
        if (link.length > MAX_LINK) throw new Error('too-long');
      }
    } catch (err) {
      const reason = err instanceof Error ? err.message : '';
      error =
        reason === 'conflict' || reason === 'outdated'
          ? de.move.errors.conflict ?? ''
          : reason === 'too-long'
            ? de.move.errors.tooLong ?? ''
            : this.account.sync.signedIn
              ? this.account.accountError(err)
              : de.move.errors.save ?? '';
      if (ui.overlay !== pending) return;
      ui.overlay = { ...pending, busy: false, error };
      this.ctx.render();
      return;
    }
    if (ui.overlay !== pending) return;
    ui.overlay = { kind: 'move', busy: false, error: null, link, what, copied: null };
    this.ctx.render();
  }

  copyLink(): void {
    const ui = this.ctx.ui;
    const o = ui.overlay;
    if (o?.kind !== 'move' || !o.link) return;
    copyText(o.link, (ok) => {
      if (ui.overlay !== o) return;
      ui.overlay = { ...o, copied: ok };
      this.ctx.render();
    });
  }

  actions(): Actions {
    return {
      'move-open': () => {
        if (this.ctx.ui.busy) return;
        this.open();
      },
      'move-make': () => void this.makeLink(),
      'move-copy': () => this.copyLink(),
    };
  }
}

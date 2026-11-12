// The dialog layer: rendered only when the dialog changes, so the score
// board can animate undisturbed. Also the sheets a playback waits on (the
// score board, the boss) and their buttons.

import { DialogFocus } from '../ui/a11y';
import { editionLook } from '../ui/cards';
import { renderScore, ScoreBoard } from '../ui/score';
import { renderOverlay } from '../ui/screens';
import type { Overlay } from '../ui/state';
import type { Actions, AppCtx } from './context';

/** How long a tap waits for its click before it acts without one (ms), and how far the click may lie from the finger (px). */
const CLICK_WAIT_MS = 350;
const CLICK_SLOP = 30;

/** How far a finger may move and still tap (px). */
const TAP_SLOP = 12;
/** After the board shows "Weiter", how long a tap on the board itself does not go on yet (ms). */
const BOARD_SETTLE_MS = 400;

export class Layer {
  private rendered: Overlay | null = null;
  private dirty = false;
  /** The score board on the layer, while it shows. */
  scoreBoard: ScoreBoard | null = null;
  /**
   * The score board or boss sheet a playback is waiting on right now. A
   * board on the layer that nothing waits for any more (its playback
   * broke off) is stale: its button closes it instead of resolving nothing.
   */
  awaited: Overlay | null = null;

  /** Focus into a dialog when it opens, back to its opener when it closes (../ui/a11y.ts). */
  private readonly focus: DialogFocus;
  /** When the score board on the layer reached its end ("Weiter" shown), ms. */
  private boardDoneAt = 0;

  constructor(private readonly ctx: AppCtx) {
    this.focus = new DialogFocus(() => ctx.layerEl);
  }

  /**
   * Taps on the sheets a playback waits on, sure to arrive (playtest
   * 12.11.2026, iPhone app: the slip scrolled, but "Weiter" did nothing
   * until a restart). A tap's click is synthesised by the browser after
   * the finger lifts, and WebKit on iOS may drop it: it turns a tap into a
   * mere hover when content changes under the finger, the double-tap
   * gesture holds it back, and after a scroll inside the slip it may
   * hit-test against stale positions. The finger going up is always
   * delivered. So a tap on "Weiter", "Los" or the board is noted on
   * `pointerup`; its click, when it comes, does the action (as before);
   * when no click comes within a moment, the tap does it without one.
   * Never twice, and a late click of a tap that already acted is
   * swallowed, so it does not land on the screen under the closed sheet.
   *
   * On the score board: while it plays, a tap anywhere skips to its end;
   * once it shows "Weiter", a tap anywhere on it but the scrolling bill
   * goes on as well (not within a moment of the end, so the tap that
   * skipped never goes on too).
   */
  watchTaps(): void {
    const el = this.ctx.layerEl;
    let down: { id: number; x: number; y: number; target: Element | null } | null = null;
    let pending: { act: () => void; x: number; y: number; timer: number } | null = null;
    let late: { x: number; y: number; until: number } | null = null;
    const near = (e: MouseEvent, p: { x: number; y: number }): boolean => e.detail !== 0 && Math.hypot(e.clientX - p.x, e.clientY - p.y) <= CLICK_SLOP;
    el.addEventListener(
      'pointerdown',
      (e) => {
        down = e.isPrimary && (e.pointerType !== 'mouse' || e.button === 0) ? { id: e.pointerId, x: e.clientX, y: e.clientY, target: e.target instanceof Element ? e.target : null } : null;
      },
      { passive: true },
    );
    el.addEventListener('pointercancel', () => (down = null), { passive: true });
    el.addEventListener(
      'pointerup',
      (e) => {
        const d = down;
        down = null;
        if (!d || d.id !== e.pointerId || Math.hypot(e.clientX - d.x, e.clientY - d.y) > TAP_SLOP) return;
        const act = this.tapAction(d.target, e.target instanceof Element ? e.target : null);
        if (!act) return;
        if (pending) window.clearTimeout(pending.timer);
        const x = e.clientX;
        const y = e.clientY;
        // No click came: the tap acts on its own.
        const timer = window.setTimeout(() => {
          pending = null;
          late = { x, y, until: performance.now() + CLICK_WAIT_MS };
          act();
        }, CLICK_WAIT_MS);
        pending = { act, x, y, timer };
      },
      { passive: true },
    );
    // Before the app's click handlers: the tap's own click does its action.
    window.addEventListener(
      'click',
      (e) => {
        const p = pending;
        if (p && near(e, p)) {
          pending = null;
          window.clearTimeout(p.timer);
          e.preventDefault();
          e.stopImmediatePropagation();
          p.act();
          return;
        }
        if (late && performance.now() < late.until && near(e, late)) {
          late = null;
          e.preventDefault();
          e.stopImmediatePropagation();
        }
      },
      true,
    );
  }

  /** What a tap from `start` to `end` does on the sheet the playback waits on (see `watchTaps`), if anything. */
  private tapAction(start: Element | null, end: Element | null): (() => void) | null {
    if (!start || !end) return null;
    const o = this.ctx.ui.overlay;
    const button = (sel: string): boolean => {
      const b = end.closest(sel);
      return b !== null && b === start.closest(sel) && !(b instanceof HTMLButtonElement && b.disabled);
    };
    if (o?.kind === 'boss') return button('[data-action="boss-go"]') ? () => this.closeAwaited('boss') : null;
    if (o?.kind !== 'score' || !end.closest('.overlay-score')) return null;
    if (button('[data-action="score-continue"]')) return () => this.closeAwaited('score');
    const board = this.scoreBoard;
    if (!board) return null;
    if (!board.done) return () => board.skip();
    // The bill scrolls in itself: a tap there is a look, not a "Weiter".
    if (start.closest('.steps') || end.closest('.steps')) return null;
    if (performance.now() - this.boardDoneAt < BOARD_SETTLE_MS) return null;
    return () => this.closeAwaited('score');
  }

  /** Draw the layer with the next render even if the dialog is the same. */
  markDirty(): void {
    this.dirty = true;
  }

  /** The dialog layer: only rebuilt when the dialog itself changes. */
  render(): void {
    const o = this.ctx.ui.overlay;
    if (o === this.rendered && !this.dirty) return;
    const same = o !== null && o === this.rendered;
    const keep = this.focus.before();
    this.draw(same);
    this.focus.after(keep, same);
  }

  private draw(same: boolean): void {
    const ui = this.ctx.ui;
    const layerEl = this.ctx.layerEl;
    const o = ui.overlay;
    this.rendered = o;
    this.dirty = false;
    layerEl.classList.toggle('is-static', same);
    if (!o) {
      layerEl.innerHTML = '';
      this.scoreBoard = null;
      return;
    }
    if (o.kind === 'score') {
      layerEl.innerHTML = renderScore(o, (ui.view?.edition ?? []).map((c) => editionLook(c, ui.view, ui.wildSettled)));
      const board = new ScoreBoard(layerEl, o, (coins, from) => this.ctx.hudFx.gainTo(coins, from), () => (this.boardDoneAt = performance.now()));
      this.scoreBoard = board;
      // Whatever breaks in the show, the board ends with its "Weiter" button.
      board.play().catch((err: unknown) => {
        console.warn('score board', err);
        board.skip();
      });
      return;
    }
    const view = ui.screen === 'run' || ui.screen === 'end' ? ui.view : null;
    layerEl.innerHTML = renderOverlay(o, view, ui);
  }

  /**
   * A button of a dialog that is not the one in force (a board left on the
   * layer): draw the layer and the screen anew from the state, so nothing
   * stale stays under the finger.
   */
  redraw(): void {
    this.dirty = true;
    this.ctx.render();
  }

  /**
   * Show a sheet the playback waits on (the score board, the boss) until its
   * button resolves it. A render that throws (playtest 05.11.2026: after a
   * won game the board stayed with a "Weiter" that did nothing, because the
   * render inside this promise threw after drawing the board, which
   * rejected the promise and left the board on screen with a resolve
   * nobody awaited) does not stop the sheet: the layer is drawn on its own
   * and the wait goes on.
   */
  async waitOn(make: (resolve: () => void) => Extract<Overlay, { kind: 'score' } | { kind: 'boss' }>): Promise<void> {
    let resolve: () => void = () => undefined;
    const done = new Promise<void>((r) => (resolve = r));
    const overlay = make(resolve);
    this.ctx.ui.overlay = overlay;
    this.awaited = overlay;
    try {
      this.ctx.render();
    } catch (err) {
      console.error('render under a sheet failed', err);
      this.dirty = true;
      try {
        this.render();
      } catch (layerErr) {
        // Without its board nothing could go on: go on without it.
        console.error('sheet not drawn', layerErr);
        resolve();
      }
    }
    try {
      await done;
    } finally {
      if (this.awaited === overlay) this.awaited = null;
    }
  }

  /**
   * "Weiter" of the score board, "Los" of the boss: resolve the sheet the
   * playback waits on. A sheet nothing waits on any more (left on the
   * layer by a playback that broke off) goes, and the screen is drawn
   * anew from the state, so the button never does nothing.
   */
  closeAwaited(kind: 'score' | 'boss'): void {
    const o = this.ctx.ui.overlay;
    if (o?.kind === kind && this.awaited === o) {
      o.resolve();
      return;
    }
    if (o?.kind === 'score' || o?.kind === 'boss') this.ctx.ui.overlay = null;
    // The stale sheet goes even if drawing the screen anew throws: a
    // button left on screen must never do nothing.
    try {
      this.redraw();
    } finally {
      if (!this.ctx.ui.overlay && this.ctx.layerEl.querySelector('.overlay-score, .sheet-boss')) {
        this.rendered = null;
        this.scoreBoard = null;
        this.ctx.layerEl.innerHTML = '';
      }
    }
  }

  /** The dialogs' close buttons and backdrop, and the buttons of the sheets a playback waits on. */
  actions(): Actions {
    const ui = this.ctx.ui;
    return {
      'close-overlay': () => {
        const overlay = ui.overlay;
        if (overlay && overlay.kind !== 'score' && overlay.kind !== 'boss') {
          ui.overlay = null;
          this.ctx.render();
        }
      },
      'score-skip': () => {
        if (ui.overlay?.kind === 'score' && this.scoreBoard) this.scoreBoard.skip();
        else this.redraw();
      },
      'score-continue': () => this.closeAwaited('score'),
      'boss-go': () => this.closeAwaited('boss'),
    };
  }
}

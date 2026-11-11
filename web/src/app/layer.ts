// The dialog layer: rendered only when the dialog changes, so the score
// board can animate undisturbed. Also the sheets a playback waits on (the
// score board, the boss) and their buttons.

import { DialogFocus } from '../ui/a11y';
import { editionLook } from '../ui/cards';
import { renderScore, ScoreBoard } from '../ui/score';
import { renderOverlay } from '../ui/screens';
import type { Overlay } from '../ui/state';
import type { Actions, AppCtx } from './context';

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

  constructor(private readonly ctx: AppCtx) {
    this.focus = new DialogFocus(() => ctx.layerEl);
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
      const board = new ScoreBoard(layerEl, o, (coins, from) => this.ctx.hudFx.gainTo(coins, from));
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
    this.redraw();
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

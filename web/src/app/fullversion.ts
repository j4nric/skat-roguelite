// Where the demo of the iOS app ends (ADR 0028), and the Kauf-Seite.
//
// The demo is the first district with its boss. Once that boss is beaten
// and his relic taken, the run stands on the way into district 2 (`Route`
// with the next pub in district 2): there, instead of the map, the
// Kauf-Seite opens. The run stays stored as it is; "Später" goes to the
// start screen, "Weiter" from there opens the page again, and after a
// purchase or a restore the same run goes on where it stood. Nothing of
// this is game logic: the core runs unchanged, the app only stops sending
// the actions that would go on (a guard in `blocks`), and the browser
// never sees any of it (./store.ts: always full there).
//
// Special cases (the decision in ADR 0028):
// - Daily run, weekly challenge, a shared or replayed seed: the same
//   boundary; they can be played to the end of district 1.
// - Die Einführung (`view.tutorial`): never stopped (one guided pub).
// - The long night and the Einsatz-Leiter need a won run, so the full
//   version; after a refund they stop at the same boundary (district 4 and
//   on are behind district 2).
// - A stored run that already stands further (from before the purchase was
//   refunded, a save code or the account from the browser): it loads and
//   stays stored, and shows the Kauf-Seite instead of going on. Nothing is
//   lost; after the purchase it continues.
// - The cache in local storage is not part of save codes or uploads, so an
//   import from the web neither unlocks nor breaks anything.

import { sound } from '../audio';
import { de } from '../i18n';
import { buyFull, isFull, loadOffer, onFullChange, restoreFull, storeActive, storeErrorCode } from '../store';
import type { RunAction, RunView } from '../types';
import { reducedMotion } from '../ui/dom';
import type { FullOverlay } from '../ui/state';
import type { Actions, AppCtx } from './context';
import type { Layer } from './layer';

/** Districts the demo plays (from the first). */
export const DEMO_DISTRICTS = 1;

/** How long the joy of a purchase shows before the run goes on (ms). */
const JOY_MS = 1600;

/**
 * The district (from 0) the run is in, or on its way to between two pubs:
 * after the relic of a boss the route already leads into the next one.
 * The boss reward itself still belongs to the district of its boss.
 */
export function districtReached(view: RunView): number {
  let d = view.series.section;
  const between = view.phase === 'Route' || view.phase === 'Shop' || view.phase === 'Encounter' || view.phase === 'CardReward';
  if (between && view.next_series) d = Math.max(d, view.next_series.section);
  if (view.phase === 'Route' && view.tour.route) d = Math.max(d, view.tour.route.next.section);
  return d;
}

/** Whether this run stands at (or beyond) the end of the demo and the app is not unlocked. */
export function fullWall(view: RunView | null): boolean {
  if (!view || !storeActive() || isFull() || view.tutorial) return false;
  return districtReached(view) >= DEMO_DISTRICTS;
}

/** Actions that still go at the wall: what a won district-1 pub left to take. */
function allowedAtWall(action: RunAction): boolean {
  return typeof action === 'object' && ('ChooseSchnappsReward' in action || 'ChoosePaint' in action);
}

export class FullVersion {
  /** The joy after a purchase, waiting to close the page. */
  private joyTimer: number | undefined;

  constructor(
    private readonly ctx: AppCtx,
    private readonly layer: Layer,
    /** "Später" at the wall: the start screen (./lifecycle.ts). */
    private readonly toStart: () => void,
    /** The run goes on after the wall (an opponent may be to move in a resumed game). */
    private readonly goOn: () => void,
  ) {
    onFullChange((full) => this.changed(full));
  }

  /** Whether `action` must not go now: the run stands at the wall. */
  blocks(action: RunAction): boolean {
    const ui = this.ctx.ui;
    return fullWall(ui.view) && !allowedAtWall(action);
  }

  /**
   * Before every render: open the Kauf-Seite when the run stands at the
   * wall and nothing else is going on; close a wall page that is no longer
   * one (the start screen came).
   */
  sync(): void {
    const ui = this.ctx.ui;
    const o = ui.overlay;
    if (o?.kind === 'full-version') {
      if (o.wall && ui.screen !== 'run') ui.overlay = null;
      return;
    }
    if (ui.screen === 'run' && fullWall(ui.view) && !ui.busy && !ui.anim && !o) this.open(true);
  }

  /** Show the Kauf-Seite; its price is loaded from the App Store. */
  open(wall: boolean): void {
    const overlay: FullOverlay = { kind: 'full-version', wall, step: 'loading', offer: null };
    this.ctx.ui.overlay = overlay;
    void this.loadOffer(overlay);
  }

  private async loadOffer(overlay: FullOverlay): Promise<void> {
    overlay.step = 'loading';
    this.redraw(overlay);
    const offer = await loadOffer();
    if (this.ctx.ui.overlay !== overlay || overlay.step !== 'loading') return;
    overlay.offer = offer;
    overlay.step = offer ? 'ready' : 'offline';
    this.redraw(overlay);
  }

  /** Draw the page again if it is still the one on screen. */
  private redraw(overlay: FullOverlay): void {
    if (this.ctx.ui.overlay !== overlay) return;
    this.markDirty();
    this.ctx.render();
  }

  /** The layer draws a dialog only when it changes; this one changed inside. */
  private markDirty(): void {
    this.layer.markDirty();
  }

  private async buy(overlay: FullOverlay): Promise<void> {
    if (overlay.step === 'buying' || overlay.step === 'restoring' || overlay.step === 'done') return;
    overlay.step = 'buying';
    this.redraw(overlay);
    try {
      const result = await buyFull();
      if (this.ctx.ui.overlay !== overlay) return;
      if (result === 'purchased' || isFull()) this.joy(overlay);
      else {
        overlay.step = result === 'pending' ? 'pending' : 'cancelled';
        this.redraw(overlay);
      }
    } catch (err) {
      if (this.ctx.ui.overlay !== overlay) return;
      const code = storeErrorCode(err);
      overlay.step = code === 'cancelled' ? 'cancelled' : code === 'unavailable' ? 'offline' : 'failed';
      if (code === 'unavailable') overlay.offer = null;
      this.redraw(overlay);
    }
  }

  private async restore(overlay: FullOverlay): Promise<void> {
    if (overlay.step === 'buying' || overlay.step === 'restoring' || overlay.step === 'done') return;
    const before = overlay.step;
    overlay.step = 'restoring';
    this.redraw(overlay);
    try {
      const found = await restoreFull();
      if (this.ctx.ui.overlay !== overlay) return;
      if (found) this.joy(overlay);
      else {
        overlay.step = 'nothing';
        this.redraw(overlay);
      }
    } catch (err) {
      if (this.ctx.ui.overlay !== overlay) return;
      overlay.step = storeErrorCode(err) === 'cancelled' ? before : 'failed';
      this.redraw(overlay);
    }
  }

  /** Unlocked: a short joy on the page, then the run goes on (or the page closes). */
  private joy(overlay: FullOverlay): void {
    if (overlay.step === 'done') return;
    overlay.step = 'done';
    sound.play('unlock');
    this.redraw(overlay);
    window.clearTimeout(this.joyTimer);
    this.joyTimer = window.setTimeout(() => this.close(overlay), reducedMotion() ? JOY_MS / 2 : JOY_MS);
  }

  private close(overlay: FullOverlay): void {
    if (this.ctx.ui.overlay !== overlay) return;
    this.ctx.ui.overlay = null;
    this.ctx.render();
    if (overlay.wall) this.goOn();
  }

  /** StoreKit changed its answer: bought elsewhere or approved (joy), or taken back (the wall comes again). */
  private changed(full: boolean): void {
    const o = this.ctx.ui.overlay;
    if (full && o?.kind === 'full-version') {
      this.joy(o);
      return;
    }
    this.ctx.render();
  }

  /** "Käufe wiederherstellen" in the menu: no page, a short note. */
  private async restoreFromMenu(): Promise<void> {
    this.ctx.showToast(de.full.restoring);
    try {
      const found = await restoreFull();
      this.ctx.showToast(found ? de.full.restored : de.full.nothingFound);
    } catch (err) {
      if (storeErrorCode(err) !== 'cancelled') this.ctx.showToast(de.full.restoreFailed);
    }
    this.markDirty();
    this.ctx.render();
  }

  actions(): Actions {
    const ui = this.ctx.ui;
    const page = (): FullOverlay | null => (ui.overlay?.kind === 'full-version' ? ui.overlay : null);
    return {
      'full-open': () => {
        if (!storeActive()) return;
        this.open(false);
        this.ctx.render();
      },
      'full-buy': () => {
        const o = page();
        if (o) void this.buy(o);
      },
      'full-restore': () => {
        const o = page();
        if (o) void this.restore(o);
      },
      'full-retry': () => {
        const o = page();
        if (o) void this.loadOffer(o);
      },
      'full-later': () => {
        const o = page();
        if (!o) return;
        window.clearTimeout(this.joyTimer);
        if (o.wall && fullWall(ui.view)) {
          this.toStart();
          return;
        }
        ui.overlay = null;
        this.ctx.render();
        if (o.wall) this.goOn();
      },
      'full-restore-menu': () => {
        if (storeActive()) void this.restoreFromMenu();
      },
    };
  }
}

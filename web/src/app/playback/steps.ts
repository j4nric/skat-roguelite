// A step of the core played back: the player's action, then the
// opponents one card at a time, the thought bubble at that opponent
// painted while the core (in its worker, ADR 0022) computes his card. The
// player's own steps are stored at once, the opponents' cards with the
// next one.

import { sound } from '../../audio';
import { coreOffMainThread, type CoreRun } from '../../core';
import { buzz } from '../../haptics';
import { de, t } from '../../i18n';
import { clearRun } from '../../storage';
import type { Sync } from '../../sync';
import type { RunAction, RunView, Step } from '../../types';
import { motionOn } from '../../ui/fx';
import { fireworks, lightsOut, lightsUp } from '../../ui/juice';
import { playTravel } from '../../ui/route';
import { errorText, nextPaint } from '../consts';
import type { AppCtx, RunActions } from '../context';
import type { Layer } from '../layer';
import type { Progress } from '../progress';
import { resetTableChoices } from '../table-input';
import type { EventPlayback } from './events';
import type { Flights } from './flights';
import type { Fx } from './fx';

/** How long the cards slide away before a settled game is scored. */
const SETTLE_MS = 260;
/** The trip along a way on the map at tempo "Normal". */
const TRAVEL_MS = 1600;

export class Steps {
  constructor(
    private readonly ctx: AppCtx,
    private readonly fx: Fx,
    private readonly flights: Flights,
    private readonly events: EventPlayback,
    private readonly layer: Layer,
    private readonly progress: Progress,
    private readonly sync: Sync,
    /** Runs once a playback is over (the Einführung left meanwhile goes then). */
    private readonly afterPlayback: () => void,
  ) {}

  /**
   * Send an action to the core and play back what happened. `fast` skips
   * the card animations and pauses ("Rest abkürzen"); the score board
   * still comes.
   */
  async act(action: RunAction, fast = false): Promise<void> {
    const core = this.ctx.core;
    if (!core || this.ctx.ui.busy) return;
    const before = this.ctx.ui.view;
    // Busy until the playback is over: taps meanwhile are ignored, as
    // while an opponent thinks.
    this.ctx.ui.busy = true;
    // Any move ends a look at the last trick.
    this.ctx.ui.peekTrick = false;
    this.flights.noteCardSources(action);
    this.showBusy();
    let step;
    try {
      step = await core.apply(action);
    } catch (err) {
      console.warn('action rejected', action, err);
      this.flights.pickupFrom = null;
      this.flights.pressFlight = null;
      this.ctx.ui.busy = false;
      this.ctx.render();
      if (this.ctx.core === core) this.ctx.showToast(t(de.app.actionFailed, { reason: errorText(err) }));
      else this.afterPlayback();
      return;
    }
    // The run was let go meanwhile (the Einführung left): nothing to play back.
    if (this.ctx.core !== core) {
      this.flights.pickupFrom = null;
      this.flights.pressFlight = null;
      this.ctx.ui.busy = false;
      this.afterPlayback();
      return;
    }
    this.ctx.fastForward = fast;
    this.ctx.ui.overlay = null;
    await this.playSteps(before, step);
  }

  /**
   * The table on screen says at once that it waits for the core, whose
   * answer comes from its worker a moment later: the look of a busy table
   * (`.run.is-busy`) and no playable card, until the playback draws the new
   * state. A whole render would cost the main thread a frame for nothing.
   */
  private showBusy(): void {
    const run = this.ctx.screenEl.querySelector('.run');
    if (!run) return;
    run.classList.add('is-busy');
    for (const el of run.querySelectorAll('[data-action="play"]')) {
      el.removeAttribute('data-action');
      // Screen readers: the card is not playable while the core answers.
      el.setAttribute('aria-disabled', 'true');
    }
  }

  /**
   * "Los": the position travels along the chosen way on the map (not with
   * reduced motion; the tempo setting scales it, a tap skips it), then the
   * core takes the choice and the stop follows.
   */
  async travel(option: number): Promise<void> {
    if (this.ctx.ui.busy) return;
    sound.play('chalk');
    if (motionOn() && !this.ctx.ui.routeOverview) {
      this.ctx.ui.busy = true;
      this.ctx.screenEl.querySelectorAll<HTMLButtonElement>('.route button').forEach((b) => (b.disabled = true));
      await playTravel(this.ctx.screenEl, TRAVEL_MS * this.fx.tempo());
      this.ctx.ui.busy = false;
    }
    await this.ctx.act({ ChooseRoute: option });
  }

  /**
   * "Rest abkürzen": the cards still on the table and in the hand slide
   * away (not with reduced motion), then the core plays the rest out and
   * the score board follows.
   */
  async settle(): Promise<void> {
    if (this.ctx.ui.busy) return;
    this.ctx.ui.busy = true;
    const cards = Array.from(this.ctx.screenEl.querySelectorAll<HTMLElement>('.table .trick-card, .table .hand-card'));
    if (motionOn() && cards.length) {
      this.ctx.screenEl.querySelector('.table')?.classList.add('is-settling');
      await new Promise((resolve) => window.setTimeout(resolve, SETTLE_MS));
    }
    this.ctx.ui.busy = false;
    await this.ctx.act('SettleGame', true);
  }

  /** A run loaded while an opponent was to move: let the opponents play on. */
  continueOpponents(): void {
    const view = this.ctx.ui.view;
    if (!this.ctx.core || this.ctx.ui.busy || view?.ai_to_move == null) return;
    this.ctx.ui.busy = true;
    this.ctx.fastForward = false;
    void this.playSteps(view, null);
  }

  /**
   * Play a step of the core back (`first`; null to start with an opponent's
   * card), then, while an opponent is to move, one card of his at a time:
   * the thought bubble at his seat is painted while the core computes his
   * card (on the main thread only after the paint), then it is animated.
   *
   * Storing (W6 of the review): the player's own step is stored at once,
   * so a reload never takes a decision back. The opponents' cards after it
   * are stored with the player's next step, or before when the phase
   * changes, a pub ends, the start screen or the save code reads the
   * storage, or the page is hidden; until then the run and the meta state
   * wait in memory together (`persist`). A reload in between loads the state after the
   * player's step, and the core plays the same opponent cards again (their
   * choice follows from the stored state), with the meta state at the same
   * point, so nothing is counted twice. What waits in memory is the run's
   * save text together with the meta state that has seen its events, so a
   * `persist` between the core's answer and the meta state's never stores
   * one without the other.
   */
  private async playSteps(before: RunView | null, first: Step | null): Promise<void> {
    let previous = before;
    let view = before;
    let step = first;
    /** This step is the player's own (stored at once). */
    let own = first !== null;
    let earned: string[] = [];
    // A pub ended (won, lost, the run over): the account gets the new state.
    let pubEnded = false;
    try {
      for (;;) {
        const core: CoreRun | null = this.ctx.core;
        if (!core) break;
        if (!step) {
          const seat = view?.ai_to_move ?? null;
          if (seat === null) break;
          this.ctx.ui.thinking = this.ctx.ui.settings.thinking ? seat : null;
          this.ctx.render();
          // The bubble is painted at least once. The worker computes
          // meanwhile; on the main thread the core waits for the paint.
          const painted = nextPaint();
          const pending = coreOffMainThread() ? core.step() : painted.then(() => core.step());
          try {
            [, step] = await Promise.all([painted, pending]);
          } catch (err) {
            console.warn('opponent move failed', err);
            if (this.ctx.core === core) this.ctx.showToast(t(de.app.actionFailed, { reason: errorText(err) }));
            break;
          } finally {
            this.ctx.ui.thinking = null;
          }
          // The run was let go meanwhile (the Einführung left).
          if (this.ctx.core !== core) break;
        }
        view = step.view;
        // The run's state after this step, kept for `persist` together with
        // the meta state that has seen its events: both change below, in one go.
        const save = core.save();
        if (!view.tutorial) earned = earned.concat(await this.progress.observe(step.events, true));
        // The Einführung (DESIGN 16.1) is no run: never saved, never
        // counted, never synced, and a run in progress stays as it was stored.
        if (!view.tutorial && step.events.some((e) => typeof e === 'object' && ('SeriesWon' in e || 'RunWon' in e || 'RunLost' in e))) pubEnded = true;
        if (view.tutorial) {
          // Nothing to keep.
        } else if (view.phase === 'Won' || view.phase === 'Lost') {
          this.progress.unsaved.run = null;
          clearRun();
          // A night that ends (ADR 0020) is no run of its own: the win
          // was recorded when der Alte fell.
          if (!view.night) this.progress.recordRun(view);
        } else if (this.ctx.core === core) {
          this.progress.unsaved.run = { core, save };
        }
        // Stored now: the player's own step, a new phase, the run over.
        const store = own || view.phase !== previous?.phase || view.phase === 'Won' || view.phase === 'Lost';
        if (store) this.ctx.persist();
        own = false;
        await this.events.playEvents(previous, step.events, view);
        // A lost run: the pub lights go out before the end screen.
        if (view.phase === 'Lost') await lightsOut();
        // The table as the core has it now, before the next opponent thinks.
        this.ctx.ui.anim = null;
        this.ctx.ui.view = view;
        this.flights.playedFrom = null;
        this.flights.thrown = null;
        previous = view;
        step = null;
      }
    } finally {
      // The account gets the new state: stored first. Otherwise what the
      // opponents played waits in memory for the player's next step.
      if (pubEnded) {
        this.ctx.persist();
        void this.sync.push();
      }
      this.ctx.ui.busy = false;
      this.ctx.ui.anim = null;
      this.ctx.ui.thinking = null;
      // Nothing waits on a score board or boss sheet once the playback is over.
      if (this.ctx.ui.overlay?.kind === 'score' || this.ctx.ui.overlay?.kind === 'boss') this.ctx.ui.overlay = null;
      this.layer.awaited = null;
      if (view) this.ctx.ui.view = view;
      this.flights.playedFrom = null;
      this.flights.thrown = null;
      resetTableChoices(this.ctx.ui);
      if (view && (view.phase === 'Won' || view.phase === 'Lost')) {
        this.ctx.ui.ended = { won: view.phase === 'Won', view };
        this.ctx.ui.screen = 'end';
        this.ctx.ui.copy = null;
        sound.play(view.phase === 'Won' ? 'win' : 'lose', { delay: 200 });
        // The lost pub buzzed with its score board already.
        if (view.phase === 'Won') buzz('runWon');
        if (view.phase === 'Won') this.ctx.afterRender.push(() => fireworks());
      }
      // The end screen must come whatever went wrong before it (playtest
      // 04.11.2026: stuck on a lost board without a working button).
      try {
        this.progress.announceUnlocks(earned);
      } catch (err) {
        console.warn('unlocks not shown', err);
      }
      try {
        this.ctx.render();
      } finally {
        lightsUp();
      }
      // The Einführung was left during the playback: now it can go.
      this.afterPlayback();
    }
  }

  /** The two buttons that play out with a scene first: "Rest abkürzen" and the route's "Los". */
  runActions(): RunActions {
    return {
      'settle': () => void this.settle(),
      'choose-route': ({ num }) => void this.travel(num('option')),
    };
  }
}

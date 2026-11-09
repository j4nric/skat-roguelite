// The run's lifecycle: the boot, the start screen, a new run (typed,
// shared, daily or replayed seed), resuming a stored one, the long night,
// and the Einführung (DESIGN 16.1), whose run is never saved nor counted.

import { CoreRun, loadCatalog, loadCore } from '../core';
import { de, setCatalog } from '../i18n';
import { clearShareHash, isSeedText, parseShareHash } from '../seed';
import { pickedStake } from '../stake';
import { clearRun, loadRun, saveRun, saveRunInfo } from '../storage';
import type { Reconciled } from '../sync';
import { markTutorial, tutorialSeen } from '../tutorial';
import type { RunLength, RunView } from '../types';
import { reshuffle } from '../ui/scatter';
import type { PendingStart } from '../ui/state';
import { takeLoadedNote, type AccountFlow } from './account';
import { errorCode, errorText, runLine, yieldToBrowser } from './consts';
import type { AppCtx } from './context';
import type { Guidance } from './guidance';
import type { Layer } from './layer';
import type { Flights } from './playback/flights';
import type { Fx } from './playback/fx';
import type { Steps } from './playback/steps';
import type { Progress } from './progress';
import { resetTableChoices } from './table-input';

export class Lifecycle {
  /** The Einführung was left while a playback ran: where to go once it is over. */
  private leaveTutorialTo: 'start' | 'run' | null = null;

  constructor(
    private readonly ctx: AppCtx,
    private readonly account: AccountFlow,
    private readonly progress: Progress,
    private readonly fx: Fx,
    private readonly flights: Flights,
    private readonly guidance: Guidance,
    private readonly steps: Steps,
    private readonly layer: Layer,
  ) {}

  async start(): Promise<void> {
    this.ctx.render();
    try {
      await loadCore();
      // A breath between instantiating the core and reading the catalog,
      // and before the stored states: the boot is no single long task.
      await yieldToBrowser();
      setCatalog(loadCatalog());
    } catch (err) {
      console.warn('core failed to load', err);
      this.ctx.ui.screen = 'error';
      this.ctx.render();
      return;
    }
    await yieldToBrowser();
    // Signed in: compare with the account before anything is read from storage.
    let synced: Reconciled = 'none';
    if (this.account.sync.signedIn) synced = await this.account.sync.reconcileAtStart(5000);
    this.progress.loadMetaState();
    this.readShareHash();
    window.addEventListener('hashchange', () => {
      this.readShareHash();
      if (this.ctx.ui.screen === 'start') this.ctx.render();
    });
    this.toStart();
    if (synced === 'applied' || takeLoadedNote()) this.ctx.showToast(de.account.loadedFromServer);
    if (synced === 'conflict') this.account.openConflict();
    void this.account.refreshAccount();
  }


  /** Offer the seed of a shared link (`#seed-…`), if the address has one. */
  private readShareHash(): void {
    const shared = parseShareHash(window.location.hash);
    if (!shared) return;
    const entry = this.ctx.ui.meta?.characters.find((c) => c.character === shared.character);
    const locked = shared.character !== 'Declarer' && entry?.unlocked !== true;
    this.ctx.ui.shared = { ...shared, locked };
  }


  toStart(): void {
    // The start screen reads the run from storage.
    this.ctx.persist();
    this.ctx.core?.dispose();
    this.ctx.core = null;
    this.ctx.ui.view = null;
    this.ctx.ui.overlay = null;
    this.ctx.ui.ended = null;
    this.ctx.ui.saveInfo = null;
    const saved = loadRun();
    if (saved) {
      try {
        this.ctx.core = CoreRun.load(saved);
        const view = this.ctx.core.view();
        this.ctx.ui.view = view;
        this.ctx.ui.saveInfo = runLine(view);
      } catch (err) {
        console.warn('saved run discarded', err);
        clearRun();
        this.ctx.core = null;
        // A run of the old rules (shared deck, before ADR 0015) is told once.
        this.ctx.ui.message = errorCode(err) === 'OutdatedSave' ? de.start.newRules : de.start.resumeFailed;
      }
    }
    // Die Einführung (DESIGN 16.1) is offered once to a new player.
    this.ctx.ui.tutorialOffer = !tutorialSeen() && !saved && !this.ctx.ui.history.length && !(this.ctx.ui.meta?.stats.runs_finished ?? 0);
    this.ctx.ui.screen = 'start';
    this.account.loadBoard();
    this.ctx.render();
  }


  /** Whether a run in progress is stored (it must not be overwritten unasked). */
  private hasRunningRun(): boolean {
    return loadRun() !== null;
  }


  /** "Neuer Run" on the start or end screen: the typed seed or a random one. */
  newRun(): void {
    const input = this.ctx.root.querySelector<HTMLInputElement>('#seed');
    const typed = input?.value.trim() ?? '';
    let seed = typed;
    if (!seed) {
      const random = new Uint32Array(1);
      crypto.getRandomValues(random);
      seed = String(random[0]);
    } else if (!isSeedText(seed)) {
      this.ctx.ui.message = de.start.seedInvalid;
      this.ctx.render();
      return;
    }
    this.requestStart({ seed, character: this.ctx.ui.pickCharacter, stake: this.pickedStake(), daily: false });
  }


  /** The stake picked on the start screen, at most the highest unlocked. */
  pickedStake(): number {
    return pickedStake(this.ctx.ui.meta, this.ctx.ui.pickCharacter, this.ctx.ui.pickStake);
  }


  /** Start a run, after asking if that gives up a running one. */
  requestStart(start: PendingStart): void {
    if (this.hasRunningRun()) {
      this.ctx.ui.overlay = { kind: 'confirm-new', start };
      this.ctx.render();
      return;
    }
    this.startRun(start);
  }


  startRun({ seed, character, stake, daily }: PendingStart): void {
    // Since ADR 0016 every run is the whole evening.
    const length: RunLength = 'Full';
    let core: CoreRun;
    let startEvents = '[]';
    try {
      if (this.ctx.meta) {
        const started = CoreRun.createWith(seed, character, length, stake, this.ctx.meta);
        core = started.run;
        startEvents = started.startEvents;
      } else {
        core = CoreRun.create(seed);
      }
    } catch (err) {
      console.warn('run not started', err);
      this.ctx.ui.overlay = null;
      this.ctx.ui.message = errorCode(err) === 'InvalidSeed' ? de.start.seedInvalid : errorText(err);
      this.ctx.render();
      return;
    }
    this.ctx.persist();
    this.ctx.core?.dispose();
    this.ctx.core = core;
    this.ctx.ui.message = null;
    this.ctx.ui.ended = null;
    this.ctx.ui.copy = null;
    if (this.ctx.ui.shared) {
      this.ctx.ui.shared = null;
      clearShareHash();
    }
    saveRun(core.save());
    const view = core.view();
    saveRunInfo({ seed: view.seed, daily });
    const earned = this.progress.observe(startEvents);
    this.enterRun(view, true);
    if (earned.length) {
      this.progress.announceUnlocks(earned);
      this.ctx.render();
    }
  }


  /**
   * "Weiterziehen in die lange Nacht" (ADR 0020): the won run goes on. Its
   * win is recorded already; from here it is saved again as it goes.
   */
  enterNight(): void {
    const ended = this.ctx.ui.ended;
    if (!this.ctx.core || !ended?.won || !ended.view.can_enter_night || this.ctx.ui.busy) return;
    this.ctx.ui.ended = null;
    this.ctx.ui.screen = 'run';
    this.ctx.ui.view = ended.view;
    this.ctx.ui.nightSeen = null;
    void this.ctx.act('EnterNight');
  }


  resume(): void {
    if (!this.ctx.core) return;
    this.ctx.ui.message = null;
    this.enterRun(this.ctx.core.view(), true);
    // Saved while an opponent was to move: let him play on.
    this.steps.continueOpponents();
  }


  private enterRun(view: RunView, dealt: boolean): void {
    this.ctx.hudFx.reset();
    this.ctx.ui.view = view;
    this.ctx.ui.screen = 'run';
    this.ctx.ui.anim = null;
    this.ctx.ui.overlay = null;
    resetTableChoices(this.ctx.ui);
    if (dealt && view.game) {
      reshuffle();
      this.ctx.afterRender.push(() => this.flights.dealEffect());
    }
    // A fresh series says hello; a run resumed mid-series does not.
    if (dealt && view.game && view.series.games_played === 0) this.fx.greetSeries(view);
    else this.fx.quipSeries = `${view.series.section}-${view.series.index}`;
    this.ctx.render();
  }


  /**
   * Start the Einführung: the core's guided pub with fixed deals. A run in
   * progress stays stored untouched; the tutorial run is never saved and
   * its events never reach the meta state.
   */
  startTutorial(): void {
    if (this.ctx.ui.busy) return;
    let core: CoreRun;
    try {
      core = CoreRun.tutorial();
    } catch (err) {
      console.warn('tutorial not started', err);
      this.ctx.showToast(errorText(err));
      return;
    }
    markTutorial('done');
    this.ctx.persist();
    this.ctx.core?.dispose();
    this.ctx.core = core;
    this.guidance.startCoach();
    this.ctx.ui.tutorialOffer = false;
    this.ctx.ui.message = null;
    this.ctx.ui.ended = null;
    this.ctx.ui.copy = null;
    this.enterRun(core.view(), true);
  }


  /** The coach goes; the bubble and ring with him. */
  endTutorial(): void {
    this.guidance.endCoach();
  }


  /**
   * Leave the Einführung (skipped, or finished): to the start screen, or
   * into a real evening ('run') when no run in progress would be given up.
   * During a playback the core is let go first, the rest waits for its end.
   */
  leaveTutorial(go: 'start' | 'run'): void {
    this.endTutorial();
    markTutorial('done');
    if (this.ctx.ui.busy) {
      this.leaveTutorialTo = go;
      this.ctx.core?.dispose();
      this.ctx.core = null;
      const o = this.ctx.ui.overlay;
      if ((o?.kind === 'score' || o?.kind === 'boss') && this.layer.awaited === o) o.resolve();
      this.fx.hurry();
      return;
    }
    this.afterTutorial(go);
  }


  /** A playback is over: the Einführung left meanwhile can go now. */
  afterPlayback(): void {
    const leave = this.leaveTutorialTo;
    if (leave) {
      this.leaveTutorialTo = null;
      this.afterTutorial(leave);
    }
  }


  private afterTutorial(go: 'start' | 'run'): void {
    this.toStart();
    if (go === 'run' && !this.hasRunningRun()) this.newRun();
  }
}

// The run's lifecycle: the boot, the start screen, a new run (typed,
// shared, daily or replayed seed), resuming a stored one, the long night,
// and the Einführung (DESIGN 16.1), whose run is never saved nor counted.

import { sound } from '../audio';
import { CoreRun, loadCatalog, loadCore, weekInfo } from '../core';
import { de, setCatalog } from '../i18n';
import { clearShareHash, dailySeed, isSeedText, parseShareHash } from '../seed';
import { maxStake, pickedStake } from '../stake';
import { clearRun, loadRun, saveCharacter, saveRun, saveRunInfo } from '../storage';
import type { Reconciled } from '../sync';
import { markTutorial, tutorialSeen } from '../tutorial';
import type { RunLength, RunView } from '../types';
import { reducedMotion } from '../ui/dom';
import { reshuffle } from '../ui/scatter';
import { scrollCharacterPicker, sharedStake } from '../ui/screens';
import type { PendingStart } from '../ui/state';
import { takeLoadedNote, type AccountFlow } from './account';
import { errorCode, errorText, isCharacter, runLine, yieldToBrowser } from './consts';
import type { Actions, AppCtx } from './context';
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
  /**
   * Counts the moves between screens that wait for the core (start screen,
   * new run, Einführung): one that is overtaken by a later one is dropped.
   */
  private moves = 0;

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
      setCatalog(await loadCatalog());
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
    await this.progress.loadMetaState();
    this.readShareHash();
    window.addEventListener('hashchange', () => {
      this.readShareHash();
      if (this.ctx.ui.screen === 'start') this.ctx.render();
    });
    await this.toStart();
    if (synced === 'applied' || takeLoadedNote()) this.ctx.showToast(de.account.loadedFromServer);
    if (synced === 'conflict') void this.account.openConflict();
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


  /**
   * To the start screen, with the stored run loaded for "Weiter". The
   * screen changes once the core has loaded it; a later move (another
   * start, a new run) wins over this one.
   */
  async toStart(): Promise<void> {
    const ticket = ++this.moves;
    // The start screen reads the run from storage.
    this.ctx.persist();
    this.ctx.core?.dispose();
    this.ctx.core = null;
    const saved = loadRun();
    let loaded: CoreRun | null = null;
    let failed: unknown = null;
    if (saved) {
      try {
        loaded = await CoreRun.load(saved);
      } catch (err) {
        failed = err;
      }
    }
    // This week's challenge (DESIGN 5.6): the core names its rule and seed.
    const week = await weekInfo();
    if (ticket !== this.moves) {
      loaded?.dispose();
      return;
    }
    this.ctx.ui.view = null;
    this.ctx.ui.overlay = null;
    this.ctx.ui.ended = null;
    this.ctx.ui.saveInfo = null;
    if (loaded) {
      this.ctx.core = loaded;
      const view = loaded.view();
      this.ctx.ui.view = view;
      this.ctx.ui.saveInfo = runLine(view);
    } else if (saved) {
      console.warn('saved run discarded', failed);
      clearRun();
      // A run of the old rules (shared deck, before ADR 0015) is told once.
      this.ctx.ui.message = errorCode(failed) === 'OutdatedSave' ? de.start.newRules : de.start.resumeFailed;
    }
    // Die Einführung (DESIGN 16.1) is offered once to a new player.
    this.ctx.ui.tutorialOffer = !tutorialSeen() && !saved && !this.ctx.ui.history.length && !(this.ctx.ui.meta?.stats.runs_finished ?? 0);
    this.ctx.ui.screen = 'start';
    this.ctx.ui.week = week;
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
    void this.startRun(start);
  }


  async startRun({ seed, character, stake, daily, weekly }: PendingStart): Promise<void> {
    const ticket = ++this.moves;
    // Since ADR 0016 every run is the whole evening.
    const length: RunLength = 'Full';
    let core: CoreRun;
    let startEvents = '[]';
    try {
      if (this.ctx.meta && weekly) {
        const started = await CoreRun.createWeekly(weekly, character, stake, this.ctx.meta);
        core = started.run;
        startEvents = started.startEvents;
      } else if (this.ctx.meta) {
        const started = await CoreRun.createWith(seed, character, length, stake, this.ctx.meta);
        core = started.run;
        startEvents = started.startEvents;
      } else {
        core = await CoreRun.create(seed);
      }
    } catch (err) {
      if (ticket !== this.moves) return;
      console.warn('run not started', err);
      this.ctx.ui.overlay = null;
      this.ctx.ui.message = errorCode(err) === 'InvalidSeed' ? de.start.seedInvalid : errorText(err);
      this.ctx.render();
      return;
    }
    if (ticket !== this.moves) {
      core.dispose();
      return;
    }
    // The meta state learns of the start before anything is stored, so the
    // run and the meta state go to storage together below. From here on
    // the start counts: the meta state has seen it.
    const earned = await this.progress.observe(startEvents, true);
    // A move begun meanwhile (the start screen) would load what is stored
    // before this run: it gives way.
    ++this.moves;
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
      sound.play('shuffle');
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
  async startTutorial(): Promise<void> {
    if (this.ctx.ui.busy) return;
    const ticket = ++this.moves;
    let core: CoreRun;
    try {
      core = await CoreRun.tutorial();
    } catch (err) {
      console.warn('tutorial not started', err);
      if (ticket === this.moves) this.ctx.showToast(errorText(err));
      return;
    }
    if (ticket !== this.moves || this.ctx.ui.busy) {
      core.dispose();
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
    void this.afterTutorial(go);
  }


  /** A playback is over: the Einführung left meanwhile can go now. */
  afterPlayback(): void {
    const leave = this.leaveTutorialTo;
    if (leave) {
      this.leaveTutorialTo = null;
      void this.afterTutorial(leave);
    }
  }


  private async afterTutorial(go: 'start' | 'run'): Promise<void> {
    await this.toStart();
    if (go === 'run' && !this.hasRunningRun()) this.newRun();
  }

  /** The start and end screens' buttons, and those of the Einführung (DESIGN 16.1). */
  actions(): Actions {
    const ui = this.ctx.ui;
    return {
      'new-run': () => {
        this.endTutorial();
        this.newRun();
      },
      'tutorial-start': () => void this.startTutorial(),
      'tutorial-offer-no': () => {
        markTutorial('declined');
        ui.tutorialOffer = false;
        this.ctx.render();
      },
      'tutorial-skip': () => this.leaveTutorial('start'),
      'tutorial-finish': ({ el }) => this.leaveTutorial(el.dataset.go === 'run' ? 'run' : 'start'),
      // Die lange Nacht (ADR 0020): on from the end screen of a won run.
      'enter-night': () => this.enterNight(),
      'resume': () => this.resume(),
      'play-shared': () => {
        const shared = ui.shared;
        if (shared) this.requestStart({ seed: shared.seed, character: shared.locked ? 'Declarer' : shared.character, stake: sharedStake(ui), daily: false });
      },
      'dismiss-shared': () => {
        ui.shared = null;
        clearShareHash();
        this.ctx.render();
      },
      'play-daily': () => this.requestStart({ seed: dailySeed(), character: ui.pickCharacter, stake: this.pickedStake(), daily: true }),
      'play-weekly': () => {
        const week = ui.week;
        if (!week) return;
        // A rule with its own character plays that one, at stake 0.
        const fixed = week.character;
        this.requestStart({ seed: week.seed, character: fixed ?? ui.pickCharacter, stake: fixed ? 0 : this.pickedStake(), daily: false, weekly: week.week });
      },
      'replay': ({ el, num }) => {
        const c = el.dataset.character ?? null;
        const seed = el.dataset.seed ?? '';
        const entry = ui.meta?.characters.find((x) => x.character === c);
        // Replayed at its stake, or the highest unlocked if that is lower.
        const stake = isCharacter(c) ? Math.min(num('stake') < 0 ? 0 : num('stake'), maxStake(ui.meta, c)) : 0;
        if (isCharacter(c) && entry?.unlocked && isSeedText(seed)) this.requestStart({ seed, character: c, stake, daily: false });
      },
      'confirm-new': () => {
        const overlay = ui.overlay;
        if (overlay?.kind === 'confirm-new') {
          const start = overlay.start;
          ui.overlay = null;
          clearRun();
          void this.startRun(start);
        }
      },
      'to-start': () => {
        this.endTutorial();
        void this.toStart();
      },
      'pick-stake': ({ num }) => {
        const max = maxStake(ui.meta, ui.pickCharacter);
        const stake = num('stake');
        if (stake >= 0 && stake <= max) {
          ui.pickStake = stake;
          this.ctx.render();
        }
      },
      'char-scroll': ({ num }) => scrollCharacterPicker(this.ctx.screenEl, num('dir'), !reducedMotion()),
      'pick-character': ({ el }) => {
        const c = el.dataset.character ?? null;
        const entry = ui.meta?.characters.find((x) => x.character === c);
        if (isCharacter(c) && entry?.unlocked) {
          ui.pickCharacter = c;
          ui.pickStake = null;
          saveCharacter(c);
          this.ctx.render();
        }
      },
    };
  }
}

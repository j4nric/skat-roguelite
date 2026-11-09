// The app controller: holds the core run and the UI state, turns taps into
// RunActions, plays the core's events back as animations with sound, and
// saves the core state after every action of the player. No game rules live here: what
// is legal, what a game is worth and who takes a trick all come from the
// core.
//
// The DOM has three layers: the screen (drawn on every change, part by
// part: only the parts whose HTML changed are rebuilt, ./ui/patch.ts), the
// overlay (rendered only when the dialog changes, so the score board can
// animate undisturbed) and the toast line.
//
// Next to the run the app keeps the meta state of the core (unlocks and
// statistics across runs). It learns from the events of every step; what
// it newly unlocks is announced at the end of the step.
//
// Seeds can be shared as a link (`#seed-<number>-<character>`, ./seed.ts);
// opening one offers that run on the start screen without touching a run
// in progress. The core plays the opponents one card per call: after the
// player's action the app asks for one opponent card at a time while the
// view names who is to move (`ai_to_move`). Each such call runs after the
// next frame, so the thought bubble at that opponent is painted before the
// main thread is busy; the player's own steps are stored at once, the
// opponents' cards with the next one (playSteps).
//
// Above the dialogs sits the Wirt's speech bubble (./hints.ts, "Geführter
// erster Abend"): after every render the app asks which hint fits what is
// on screen, shows it once and remembers it when it is dismissed or the
// situation has passed.

import { sound, unlockAudio } from './audio';
import { music, type MusicId } from './music';
import { CoreMeta, CoreRun, loadCatalog, loadCore } from './core';
import { de, cardName, consumableMod, contentName, face, setCatalog, t } from './i18n';
import { loadSettings, saveSettings, TEMPO_FACTOR, type Tempo } from './settings';
import { passkeysSupported } from './account';
import { isOnline, watchNetwork } from './offline';
import { type Reconciled } from './sync';
import { clearShareHash, dailySeed, isSeedText, parseShareHash } from './seed';
import { maxStake, pickedStake } from './stake';
import {
  clearRun,
  loadCharacter,
  loadHistory,
  loadRun,
  saveCharacter,
  saveRun,
  saveRunInfo,
} from './storage';
import { type CardKind, type CardView, type DealtCard, type DeckCard, type EditionEntry, type NullRisk, type RiskLevel, type RunAction, type RunEvent, type RunLength, type RunView, type Seat, type Source, type Step, type Suit } from './types';
import { esc, reducedMotion } from './ui/dom';
import { markeEffect, renderEncounter } from './ui/encounter';
import { renderBossReward } from './ui/bossreward';
import { renderCardReward } from './ui/cardreward';
import { defaultPick, legKey, playTravel, renderRoute, watchMapLabels } from './ui/route';
import { seatLabel as seatLabelOf } from './ui/guests';
import { markTutorial, tutorialSeen } from './tutorial';
import { infoModel } from './ui/info';
import { InfoCard } from './ui/infocard';
import { watchPresses } from './ui/press';
import { TriggerFeed } from './ui/triggers';
import { crumbleInto, finishAll, finishOn, type FlightSnapshot, flip, flyIn, glideAll, motionOn, noteFlights, resumeFlights, setTempo, spotsOf, springBack, sweepAway } from './ui/fx';
import { fireworks, HudFx, lightsOut, lightsUp, spring, watchSprings } from './ui/juice';
import { bossEntrance, mirrorShimmer, relicToStrip, sceneRunning, sceneWord, schnapsToSlot } from './ui/scenes';
import { setPlayerBack } from './ui/backs';
import { cardBack, cardSvg, lookSvg, showWildSuit } from './ui/cards';
import { SUITS } from './ui/art';
import { watchDrags } from './ui/drag';
import { watchKeys } from './ui/keys';
import { buzz, setVibration } from './haptics';
import { runCardBlob, runCardName, runCardText } from './ui/runcard';
import { Drink } from './ui/drink';
import { reshuffle } from './ui/scatter';
import { coinIcon, renderHud } from './ui/hud';
import { sourceName } from './ui/score';
import { renderEnd, renderError, renderKontra, renderRiseChoice, updateRiseChoice, renderLoading, renderStart, scrollCharacterPicker, settleCharacterPicker, sharedStake } from './ui/screens';
import { renderShop, watchShop } from './ui/shop';
import type { PendingStart, ScoreOverlay, UiState } from './ui/state';
import { discardNeed, renderTable } from './ui/table';
import { Patcher, shell, type Part } from './ui/patch';
import { DELAY, errorCode, errorText, isCharacter, nextPaint, runLine, type ToastPlace, yieldToBrowser } from './app/consts';
import { AccountFlow, takeLoadedNote } from './app/account';
import type { AppCtx } from './app/context';
import { Guidance } from './app/guidance';
import { Layer } from './app/layer';
import { Mods } from './app/mods';
import { DrinkFx } from './app/playback/drinkfx';
import { Flights } from './app/playback/flights';
import { Fx } from './app/playback/fx';
import { Progress } from './app/progress';
import { SaveCodeFlow } from './app/savecode';
import { resetTableChoices, TableInput } from './app/table-input';

/** The boss edition's modifications on a boss card (ADR 0015), from the filters the core names. */
function bossEditionOf(card: DealtCard, edition: EditionEntry[]): { mods: string[]; mod_values: Record<string, number> } | null {
  if (typeof card.kind !== 'object') return null;
  const code = card.kind.Normal;
  const suits: Record<string, Suit> = { C: 'Clubs', S: 'Spades', H: 'Hearts', D: 'Diamonds' };
  const ranks: Record<string, string> = { '7': 'Seven', '8': 'Eight', '9': 'Nine', T: 'Ten', J: 'Jack', Q: 'Queen', K: 'King', A: 'Ace' };
  const suit = suits[code[0] ?? ''];
  const rank = ranks[code[1] ?? ''];
  const mods: string[] = [];
  const mod_values: Record<string, number> = {};
  for (const e of edition) {
    if ((e.filter.suit && e.filter.suit !== suit) || (e.filter.rank && e.filter.rank !== rank)) continue;
    mods.push(e.card_mod.id);
    if (e.card_mod.value != null) mod_values[e.card_mod.id] = e.card_mod.value;
  }
  return { mods, mod_values };
}

/**
 * Cards whose flights survive a re-render: the open trick, the hand, the
 * cards to press on the table and the pressed stack.
 */
const FLYING = '.trick:not(.is-last) .trick-card, .hand-card, .skat-card, .pressed-card[data-fly]';

/** How long the cards slide away before a settled game is scored. */
const SETTLE_MS = 260;
/** The trip along a way on the map at tempo "Normal". */
const TRAVEL_MS = 1600;

/** "Untot": the event of a card that rose from the trick. */
/** "Doppelgänger": the event of its mirror. */
type DoppelMirror = Extract<Extract<RunEvent, { Game: unknown }>['Game'], { DoppelgangerMirrored: unknown }>['DoppelgangerMirrored'];
type RoseEvent = Extract<Extract<RunEvent, { Game: unknown }>['Game'], { CardRose: unknown }>['CardRose'];

const BUILD_INFO = typeof __BUILD_INFO__ === 'string' ? __BUILD_INFO__ : '';

export class App implements AppCtx {
  core: CoreRun | null = null;
  meta: CoreMeta | null = null;
  readonly ui: UiState = {
    screen: 'loading',
    view: null,
    anim: null,
    busy: false,
    selected: [],
    risePick: [],
    handGame: false,
    pickGame: null,
    pickRisk: null,
    pickNullRisk: null,
    omaPick: false,
    swapTake: null,
    peekTrick: false,
    handOrder: null,
    routePick: null,
    routeLeg: '',
    routeOverview: false,
    rewardPick: null,
    risen: null,
    rising: null,
    wildSettled: {},
    thinking: null,
    quip: null,
    shared: null,
    copy: null,
    history: loadHistory(),
    pickCharacter: (() => {
      const stored = loadCharacter();
      return isCharacter(stored) ? stored : 'Declarer';
    })(),
    // The short evening is dropped (ADR 0016): always the whole evening.
    pickStake: null,
    overlay: null,
    message: null,
    saveInfo: null,
    meta: null,
    settings: loadSettings(),
    ended: null,
    account: { user: null, status: 'off', supported: passkeysSupported(), online: isOnline() },
    board: null,
  };
  fastForward = false;
  readonly screenEl: HTMLElement;
  /** Draws the screen part by part (./ui/patch.ts). */
  private readonly screen: Patcher;
  /** Cards in flight in the parts the patcher just replaced. */
  private snapshots: FlightSnapshot[] = [];
  /** Account and Groschen in the head: how their changes get there. */
  readonly hudFx: HudFx;
  readonly layerEl: HTMLElement;
  /** "Untot": cards whose rising was already shown with their trick in this game. */
  private readonly riseShown = new Set<number>();
  /** The Einführung was left while a playback ran: where to go once it is over. */
  private leaveTutorialTo: 'start' | 'run' | null = null;
  /** The info card (long-press, hover, "i"): one for everything. */
  readonly info: InfoCard;
  /** Lines and wiggles when a relic or modification fires. */
  readonly triggers: TriggerFeed;
  /** Effects to run once the next render is in the DOM. */
  readonly afterRender: (() => void)[] = [];
  /** Drinking a Schnaps from the Leiste (drag and drop or tap, ./ui/drink.ts). */
  readonly drink: Drink;
  /** A boss pub just opened: its entrance plays once the table is free of dialogs. */
  entrancePending = false;
  /** The picture of the ended run for "Teilen", drawn in advance (./ui/runcard.ts). */
  private runCard: { view: RunView; blob: Promise<Blob> } | null = null;

  /** The dialog layer (./app/layer.ts). */
  private readonly layer = new Layer(this);
  /** The Wirt's hints and coaching (./app/guidance.ts). */
  private readonly guidance: Guidance;
  /** Pauses, toasts, quips and triggers of the playback (./app/playback/fx.ts). */
  private readonly fx: Fx;
  /** Cards in the air and the table's reactions (./app/playback/flights.ts). */
  private readonly flights: Flights;
  /** What a Schnaps does on screen (./app/playback/drinkfx.ts). */
  private readonly drinkFx: DrinkFx;
  /** Account, sync and daily board (./app/account.ts). */
  private readonly account: AccountFlow;
  /** The stored run and meta state (./app/progress.ts). */
  private readonly progress: Progress;
  /** Export and import of the save code (./app/savecode.ts). */
  private readonly saveCode: SaveCodeFlow;
  /** Card modifications from a card picker (./app/mods.ts). */
  private readonly mods: Mods;
  /** Pouring, dragging and throwing at the table (./app/table-input.ts). */
  private readonly table: TableInput;

  constructor(readonly root: HTMLElement) {
    root.innerHTML = `<div class="screen"></div><div class="layer"></div><div class="hint-layer" aria-live="polite"></div><div class="coach-layer" aria-live="polite"></div><div class="info-layer"></div><div class="trigger-layer" role="status" aria-live="polite"></div><div class="toast-layer" role="status" aria-live="polite"></div><div class="drink-layer"></div>`;
    this.screenEl = root.querySelector<HTMLElement>('.screen') ?? root;
    this.guidance = new Guidance(this);
    this.fx = new Fx(this, this.guidance);
    this.flights = new Flights(this, this.fx);
    this.drinkFx = new DrinkFx(this, this.fx);
    this.account = new AccountFlow(this, this.layer);
    this.progress = new Progress(this, this.account);
    this.saveCode = new SaveCodeFlow(this);
    this.mods = new Mods(this);
    this.table = new TableInput(this, this.fx, this.flights, this.drinkFx, this.mods);
    this.screen = new Patcher(this.screenEl, (nodes) => {
      for (const n of nodes) if (n instanceof Element) this.snapshots.push(...noteFlights(n, FLYING));
    });
    this.layerEl = root.querySelector<HTMLElement>('.layer') ?? root;
    this.hudFx = new HudFx(this.screenEl);
    watchSprings(root);
    watchMapLabels(root);
    const infoEl = root.querySelector<HTMLElement>('.info-layer') ?? root;
    this.info = new InfoCard(infoEl, (key) => infoModel(key, this.ui.view, this.ui));
    this.triggers = new TriggerFeed(root.querySelector<HTMLElement>('.trigger-layer') ?? root);
    watchPresses(root, {
      open: (key, el) => {
        this.info.show(key, 'sheet', el);
      },
      hover: (key, el) => {
        this.info.show(key, 'popover', el);
      },
      leave: () => this.info.closeHoverSoon(),
      // Only where the run is on screen, and not behind the score board.
      blocked: () => !this.ui.view || (this.ui.screen !== 'run' && this.ui.screen !== 'end') || this.ui.overlay?.kind === 'score' || this.ui.overlay?.kind === 'boss' || this.drink.quiet,
    });
    // Drag cards between hand and table, the deck into the middle.
    watchDrags(root, {
      drop: (el, zone, over, at, fling) => this.table.onDrop(el, zone, over, at, fling),
      blocked: () => this.ui.busy || this.ui.screen !== 'run' || Boolean(this.ui.overlay) || this.info.state?.mode === 'sheet' || this.drink.active,
    });
    // A Schnaps from the Leiste onto its target (or lifted by a tap).
    this.drink = new Drink(this.screenEl, root.querySelector<HTMLElement>('.drink-layer') ?? root, {
      item: (slot) => this.ui.view?.consumables.find((c) => c.slot === slot),
      blocked: () => this.ui.busy || this.ui.screen !== 'run' || Boolean(this.ui.overlay) || this.info.state?.mode === 'sheet',
      pour: (p) => this.table.pourSchnaps(p),
    });
    // No text selection and no browser callout anywhere in the game: a
    // long-press opens the info card and must not mark the page. Text
    // fields (seed, save code) and the shown seed keep theirs; links keep
    // their menu.
    const selectable = 'input, textarea, [contenteditable="true"], .seed-num';
    const elementOf = (t: EventTarget | null): Element | null => (t instanceof Element ? t : t instanceof Node ? t.parentElement : null);
    document.addEventListener('selectstart', (e) => {
      if (!elementOf(e.target)?.closest(selectable)) e.preventDefault();
    });
    document.addEventListener('contextmenu', (e) => {
      if (!elementOf(e.target)?.closest(`${selectable}, a[href]`)) e.preventDefault();
    });
    // A press anywhere else closes the popover of the mouse.
    root.addEventListener('pointerdown', (e) => {
      if (this.info.state?.mode === 'popover' && !infoEl.contains(e.target as Node | null)) this.info.close();
    });
    // The layout may settle a little after a resize (a phone turning, the
    // address bar going away); follow it then once more.
    let settle: number | undefined;
    window.addEventListener('resize', () => {
      this.guidance.positionHint();
      this.guidance.positionCoach();
      window.clearTimeout(settle);
      settle = window.setTimeout(() => {
        this.guidance.positionHint();
        this.guidance.positionCoach();
      }, 100);
    });
    // Leaving the page (tab switch, app to the background, closing): store
    // what an opponent's card left in memory, then upload what changed.
    document.addEventListener('visibilitychange', () => {
      if (document.visibilityState !== 'hidden') return;
      this.persist();
      void this.account.sync.push(true);
    });
    window.addEventListener('pagehide', () => {
      this.persist();
      // The request must start now: the upload was encoded in advance.
      this.account.sync.sendOnHide();
    });
    watchNetwork((online) => this.account.networkChanged(online));
    this.applySettings();
    root.addEventListener('click', (e) => this.onClick(e));
    // A tap while a scene plays (boss entrance, stamp) ends it; the tap still counts.
    window.addEventListener(
      'pointerdown',
      () => {
        if (sceneRunning()) finishAll();
      },
      { capture: true },
    );
    // Browsers allow audio only after a user gesture.
    // iOS Safari only counts touchend, pointerup and click as an activation
    // that may start or resume an AudioContext; pointerdown alone leaves it
    // suspended for good, so every one of them unlocks and resumes.
    for (const type of ['pointerdown', 'pointerup', 'touchend', 'click'] as const) {
      window.addEventListener(type, unlockAudio, { passive: true, capture: true });
    }
    window.addEventListener('keydown', unlockAudio);
    window.addEventListener('keydown', (e) => {
      const o = this.ui.overlay;
      if (e.key === 'Escape' && o && o.kind !== 'score' && o.kind !== 'boss') {
        this.ui.overlay = null;
        this.render();
      }
    });
    // Die Einführung: Space and Enter go on with the Wirt's bubble (./app/guidance.ts).
    window.addEventListener('keydown', (e) => this.guidance.coachKey(e), { capture: true });
    // Keys on the desktop: cards by digit, Space or Enter for the main button.
    watchKeys(root, this.screenEl, this.layerEl, {
      blocked: () => this.ui.screen === 'loading' || this.info.state?.mode === 'sheet' || this.drink.active,
      dialogOpen: () => Boolean(this.ui.overlay),
      refused: (card) => {
        if (this.ui.busy || this.ui.anim || !this.ui.view?.game?.your_turn) return;
        springBack(card, null, true);
        sound.play('card', { volume: 0.35, rate: 0.7 });
        this.showToast(de.table.notPlayable, 'top');
      },
      idle: () => {
        if (this.ui.busy) this.fx.hurry();
      },
    });
  }

  /** Draw the loading screen (before the core and the sprite are in). */
  showLoading(): void {
    this.render();
  }

  async start(): Promise<void> {
    this.render();
    try {
      await loadCore();
      // A breath between instantiating the core and reading the catalog,
      // and before the stored states: the boot is no single long task.
      await yieldToBrowser();
      setCatalog(loadCatalog());
    } catch (err) {
      console.warn('core failed to load', err);
      this.ui.screen = 'error';
      this.render();
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
      if (this.ui.screen === 'start') this.render();
    });
    this.toStart();
    if (synced === 'applied' || takeLoadedNote()) this.showToast(de.account.loadedFromServer);
    if (synced === 'conflict') this.account.openConflict();
    void this.account.refreshAccount();
  }

  // -------------------------------------------------------------------------
  // Account and sync (ADR 0021). Transport only: the save is the save code.
  // -------------------------------------------------------------------------

  /** Offer the seed of a shared link (`#seed-…`), if the address has one. */
  private readShareHash(): void {
    const shared = parseShareHash(window.location.hash);
    if (!shared) return;
    const entry = this.ui.meta?.characters.find((c) => c.character === shared.character);
    const locked = shared.character !== 'Declarer' && entry?.unlocked !== true;
    this.ui.shared = { ...shared, locked };
  }

  private applySettings(): void {
    setTempo(TEMPO_FACTOR[this.ui.settings.tempo]);
    sound.setEnabled(this.ui.settings.sound);
    music.setEnabled(this.ui.settings.music);
    // Pure presentation: style.css tints the cards' paper under this flag.
    document.documentElement.toggleAttribute('data-suit-tint', this.ui.settings.suitTint);
    setVibration(this.ui.settings.vibrate);
  }

  /**
   * The music loop for what is on screen: the Tresen loop at the Tresen and
   * at an encounter, the boss loop in a boss series, the table loop at the
   * table and on the start and end screens.
   */
  private musicScene(): MusicId | null {
    const ui = this.ui;
    if (ui.screen === 'start' || ui.screen === 'end') return 'table';
    if (ui.screen !== 'run' || !ui.view) return null;
    const view = ui.view;
    if ((view.phase === 'Shop' || view.phase === 'Encounter' || view.phase === 'Route' || view.phase === 'BossReward' || view.phase === 'CardReward') && !ui.anim) return 'shop';
    return view.series.is_boss ? 'boss' : 'table';
  }

  // -------------------------------------------------------------------------
  // Screens
  // -------------------------------------------------------------------------

  private toStart(): void {
    // The start screen reads the run from storage.
    this.persist();
    this.core?.dispose();
    this.core = null;
    this.ui.view = null;
    this.ui.overlay = null;
    this.ui.ended = null;
    this.ui.saveInfo = null;
    const saved = loadRun();
    if (saved) {
      try {
        this.core = CoreRun.load(saved);
        const view = this.core.view();
        this.ui.view = view;
        this.ui.saveInfo = runLine(view);
      } catch (err) {
        console.warn('saved run discarded', err);
        clearRun();
        this.core = null;
        // A run of the old rules (shared deck, before ADR 0015) is told once.
        this.ui.message = errorCode(err) === 'OutdatedSave' ? de.start.newRules : de.start.resumeFailed;
      }
    }
    // Die Einführung (DESIGN 16.1) is offered once to a new player.
    this.ui.tutorialOffer = !tutorialSeen() && !saved && !this.ui.history.length && !(this.ui.meta?.stats.runs_finished ?? 0);
    this.ui.screen = 'start';
    this.account.loadBoard();
    this.render();
  }

  /** Whether a run in progress is stored (it must not be overwritten unasked). */
  private hasRunningRun(): boolean {
    return loadRun() !== null;
  }

  /** "Neuer Run" on the start or end screen: the typed seed or a random one. */
  private newRun(): void {
    const input = this.root.querySelector<HTMLInputElement>('#seed');
    const typed = input?.value.trim() ?? '';
    let seed = typed;
    if (!seed) {
      const random = new Uint32Array(1);
      crypto.getRandomValues(random);
      seed = String(random[0]);
    } else if (!isSeedText(seed)) {
      this.ui.message = de.start.seedInvalid;
      this.render();
      return;
    }
    this.requestStart({ seed, character: this.ui.pickCharacter, stake: this.pickedStake(), daily: false });
  }

  /** The stake picked on the start screen, at most the highest unlocked. */
  private pickedStake(): number {
    return pickedStake(this.ui.meta, this.ui.pickCharacter, this.ui.pickStake);
  }

  /** Start a run, after asking if that gives up a running one. */
  private requestStart(start: PendingStart): void {
    if (this.hasRunningRun()) {
      this.ui.overlay = { kind: 'confirm-new', start };
      this.render();
      return;
    }
    this.startRun(start);
  }

  private startRun({ seed, character, stake, daily }: PendingStart): void {
    // Since ADR 0016 every run is the whole evening.
    const length: RunLength = 'Full';
    let core: CoreRun;
    let startEvents = '[]';
    try {
      if (this.meta) {
        const started = CoreRun.createWith(seed, character, length, stake, this.meta);
        core = started.run;
        startEvents = started.startEvents;
      } else {
        core = CoreRun.create(seed);
      }
    } catch (err) {
      console.warn('run not started', err);
      this.ui.overlay = null;
      this.ui.message = errorCode(err) === 'InvalidSeed' ? de.start.seedInvalid : errorText(err);
      this.render();
      return;
    }
    this.persist();
    this.core?.dispose();
    this.core = core;
    this.ui.message = null;
    this.ui.ended = null;
    this.ui.copy = null;
    if (this.ui.shared) {
      this.ui.shared = null;
      clearShareHash();
    }
    saveRun(core.save());
    const view = core.view();
    saveRunInfo({ seed: view.seed, daily });
    const earned = this.progress.observe(startEvents);
    this.enterRun(view, true);
    if (earned.length) {
      this.progress.announceUnlocks(earned);
      this.render();
    }
  }

  /**
   * The picture of the ended run, drawn once as the end screen comes, so a
   * tap on "Teilen" reaches the share sheet while it still counts as the
   * player's gesture (Safari is strict about that).
   */
  private prepareRunCard(): Promise<Blob> {
    const ended = this.ui.ended;
    if (!ended) return Promise.reject(new Error('no ended run'));
    const view = ended.view;
    if (this.runCard?.view === view) return this.runCard.blob;
    const record = this.ui.history[0];
    const daily = Boolean(record && record.seed === view.seed && record.daily);
    const blob = runCardBlob(view, { won: ended.won, daily, date: record && record.seed === view.seed ? new Date(record.date) : new Date() });
    // A failed drawing is tried again on the tap.
    blob.catch(() => {
      if (this.runCard?.blob === blob) this.runCard = null;
    });
    this.runCard = { view, blob };
    return blob;
  }

  /**
   * "Teilen" on the end screen: the run as a picture (./ui/runcard.ts),
   * into the share sheet with the file where the browser can, else saved
   * as a download.
   */
  private async shareRun(button: HTMLElement): Promise<void> {
    const ended = this.ui.ended;
    if (!ended || button.hasAttribute('aria-busy')) return;
    const view = ended.view;
    button.setAttribute('aria-busy', 'true');
    let blob: Blob;
    try {
      blob = await this.prepareRunCard();
    } catch (err) {
      console.warn('run picture failed', err);
      this.showToast(de.runCard.failed);
      button.removeAttribute('aria-busy');
      return;
    }
    button.removeAttribute('aria-busy');
    const name = runCardName(view);
    const text = runCardText(view, ended.won);
    try {
      const file = new File([blob], name, { type: 'image/png' });
      if (typeof navigator.share === 'function' && navigator.canShare?.({ files: [file] })) {
        await navigator.share({ files: [file], title: de.runCard.shareTitle, text });
        return;
      }
    } catch (err) {
      // Closed by the player: nothing more to do.
      if (err instanceof DOMException && err.name === 'AbortError') return;
      console.warn('share sheet failed, saving the picture instead', err);
    }
    const url = URL.createObjectURL(blob);
    const link = document.createElement('a');
    link.href = url;
    link.download = name;
    link.rel = 'noopener';
    document.body.appendChild(link);
    link.click();
    link.remove();
    window.setTimeout(() => URL.revokeObjectURL(url), 10000);
    this.showToast(de.runCard.saved);
  }

  /** Copy the link of a seed; on failure show it to be copied by hand. */
  private copyLink(url: string): void {
    const done = (ok: boolean): void => {
      this.ui.copy = { url, ok };
      this.layer.markDirty();
      this.render();
      if (ok) sound.play('clink');
      else this.root.querySelector<HTMLInputElement>('[data-testid="share-fallback"]')?.select();
    };
    try {
      const clip = navigator.clipboard as Clipboard | undefined;
      if (!clip) {
        done(false);
        return;
      }
      clip.writeText(url).then(
        () => done(true),
        (err: unknown) => {
          console.warn('clipboard refused', err);
          done(false);
        },
      );
    } catch (err) {
      console.warn('clipboard unavailable', err);
      done(false);
    }
  }

  /** The share sheet of the system, where there is one. */
  private shareNative(url: string, text: string): void {
    try {
      if (typeof navigator.share !== 'function') {
        this.copyLink(url);
        return;
      }
      navigator.share({ title: de.app.title, text, url }).catch((err: unknown) => {
        // Closing the sheet is no error.
        if (!(err instanceof DOMException && err.name === 'AbortError')) this.copyLink(url);
      });
    } catch {
      this.copyLink(url);
    }
  }

  // -------------------------------------------------------------------------
  // Save code: export and import of everything stored (./savecode.ts)
  // -------------------------------------------------------------------------

  /**
   * "Weiterziehen in die lange Nacht" (ADR 0020): the won run goes on. Its
   * win is recorded already; from here it is saved again as it goes.
   */
  private enterNight(): void {
    const ended = this.ui.ended;
    if (!this.core || !ended?.won || !ended.view.can_enter_night || this.ui.busy) return;
    this.ui.ended = null;
    this.ui.screen = 'run';
    this.ui.view = ended.view;
    this.ui.nightSeen = null;
    void this.act('EnterNight');
  }

  private resume(): void {
    if (!this.core) return;
    this.ui.message = null;
    this.enterRun(this.core.view(), true);
    // Saved while an opponent was to move: let him play on.
    this.continueOpponents();
  }

  private enterRun(view: RunView, dealt: boolean): void {
    this.hudFx.reset();
    this.ui.view = view;
    this.ui.screen = 'run';
    this.ui.anim = null;
    this.ui.overlay = null;
    resetTableChoices(this.ui);
    if (dealt && view.game) {
      reshuffle();
      this.afterRender.push(() => this.flights.dealEffect());
    }
    // A fresh series says hello; a run resumed mid-series does not.
    if (dealt && view.game && view.series.games_played === 0) this.fx.greetSeries(view);
    else this.fx.quipSeries = `${view.series.section}-${view.series.index}`;
    this.render();
  }

  // -------------------------------------------------------------------------
  // Actions
  // -------------------------------------------------------------------------

  /** Send an action to the core, then play its events and the opponents' cards back. */
  /**
   * Send an action to the core and play back what happened. `fast` skips
   * the card animations and pauses ("Rest abkürzen"); the score board
   * still comes.
   */
  async act(action: RunAction, fast = false): Promise<void> {
    if (!this.core || this.ui.busy) return;
    const before = this.ui.view;
    this.ui.busy = true;
    // Any move ends a look at the last trick.
    this.ui.peekTrick = false;
    this.flights.noteCardSources(action);
    let step;
    try {
      step = this.core.apply(action);
    } catch (err) {
      console.warn('action rejected', action, err);
      this.flights.pickupFrom = null;
      this.flights.pressFlight = null;
      this.ui.busy = false;
      this.render();
      this.showToast(t(de.app.actionFailed, { reason: errorText(err) }));
      return;
    }
    this.fastForward = fast;
    this.ui.overlay = null;
    await this.playSteps(before, step);
  }

  /**
   * "Los": the position travels along the chosen way on the map (not with
   * reduced motion; the tempo setting scales it, a tap skips it), then the
   * core takes the choice and the stop follows.
   */
  private async travel(option: number): Promise<void> {
    if (this.ui.busy) return;
    if (motionOn() && !this.ui.routeOverview) {
      this.ui.busy = true;
      this.screenEl.querySelectorAll<HTMLButtonElement>('.route button').forEach((b) => (b.disabled = true));
      await playTravel(this.screenEl, TRAVEL_MS * this.fx.tempo());
      this.ui.busy = false;
    }
    await this.act({ ChooseRoute: option });
  }

  /**
   * "Rest abkürzen": the cards still on the table and in the hand slide
   * away (not with reduced motion), then the core plays the rest out and
   * the score board follows.
   */
  private async settle(): Promise<void> {
    if (this.ui.busy) return;
    this.ui.busy = true;
    const cards = Array.from(this.screenEl.querySelectorAll<HTMLElement>('.table .trick-card, .table .hand-card'));
    if (motionOn() && cards.length) {
      this.screenEl.querySelector('.table')?.classList.add('is-settling');
      await new Promise((resolve) => window.setTimeout(resolve, SETTLE_MS));
    }
    this.ui.busy = false;
    await this.act('SettleGame', true);
  }

  /** A run loaded while an opponent was to move: let the opponents play on. */
  private continueOpponents(): void {
    const view = this.ui.view;
    if (!this.core || this.ui.busy || view?.ai_to_move == null) return;
    this.ui.busy = true;
    this.fastForward = false;
    void this.playSteps(view, null);
  }

  /**
   * Play a step of the core back (`first`; null to start with an opponent's
   * card), then, while an opponent is to move, one card of his at a time:
   * the thought bubble at his seat is painted, then the core computes his
   * card, then it is animated.
   *
   * Storing (W6 of the review): the player's own step is stored at once,
   * so a reload never takes a decision back. The opponents' cards after it
   * are stored with the player's next step, or before when the phase
   * changes, a pub ends, the start screen or the save code reads the
   * storage, or the page is hidden; until then the run and the meta state
   * wait in memory together (`persist`). A reload in between loads the state after the
   * player's step, and the core plays the same opponent cards again (their
   * choice follows from the stored state), with the meta state at the same
   * point, so nothing is counted twice.
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
        if (!step) {
          const seat = view?.ai_to_move ?? null;
          if (seat === null || !this.core) break;
          this.ui.thinking = this.ui.settings.thinking ? seat : null;
          this.render();
          await nextPaint();
          try {
            step = this.core.step();
          } catch (err) {
            console.warn('opponent move failed', err);
            this.showToast(t(de.app.actionFailed, { reason: errorText(err) }));
            break;
          } finally {
            this.ui.thinking = null;
          }
        }
        view = step.view;
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
        } else if (this.core) {
          this.progress.unsaved.run = this.core;
        }
        // Stored now: the player's own step, a new phase, the run over.
        const store = own || view.phase !== previous?.phase || view.phase === 'Won' || view.phase === 'Lost';
        if (!view.tutorial) earned = earned.concat(this.progress.observe(step.events, true));
        if (store) this.persist();
        own = false;
        await this.playEvents(previous, step.events, view);
        // A lost run: the pub lights go out before the end screen.
        if (view.phase === 'Lost') await lightsOut();
        // The table as the core has it now, before the next opponent thinks.
        this.ui.anim = null;
        this.ui.view = view;
        this.flights.playedFrom = null;
        this.flights.thrown = null;
        previous = view;
        step = null;
      }
    } finally {
      // The account gets the new state: stored first. Otherwise what the
      // opponents played waits in memory for the player's next step.
      if (pubEnded) {
        this.persist();
        void this.account.sync.push();
      }
      this.ui.busy = false;
      this.ui.anim = null;
      this.ui.thinking = null;
      // Nothing waits on a score board or boss sheet once the playback is over.
      if (this.ui.overlay?.kind === 'score' || this.ui.overlay?.kind === 'boss') this.ui.overlay = null;
      this.layer.awaited = null;
      if (view) this.ui.view = view;
      this.flights.playedFrom = null;
      this.flights.thrown = null;
      resetTableChoices(this.ui);
      if (view && (view.phase === 'Won' || view.phase === 'Lost')) {
        this.ui.ended = { won: view.phase === 'Won', view };
        this.ui.screen = 'end';
        this.ui.copy = null;
        sound.play(view.phase === 'Won' ? 'win' : 'lose', { delay: 200 });
        // The lost pub buzzed with its score board already.
        if (view.phase === 'Won') buzz('runWon');
        if (view.phase === 'Won') this.afterRender.push(() => fireworks());
      }
      // The end screen must come whatever went wrong before it (playtest
      // 04.11.2026: stuck on a lost board without a working button).
      try {
        this.progress.announceUnlocks(earned);
      } catch (err) {
        console.warn('unlocks not shown', err);
      }
      try {
        this.render();
      } finally {
        lightsUp();
      }
      // The Einführung was left during the playback: now it can go.
      const leave = this.leaveTutorialTo;
      if (leave) {
        this.leaveTutorialTo = null;
        this.afterTutorial(leave);
      }
    }
  }

  /** A card from an event as the table shows it: the modifications of its owner's edition (ADR 0015). */
  private cardOf(card: DealtCard, view: RunView): CardView {
    const owner = card.owner ?? 'Player';
    const own = owner === 'Player' ? view.edition.find((d) => d.id === card.id) : undefined;
    // ADR 0017: each boss player plays the edition of his figure of the rule in force.
    const figure = owner === 'Boss' ? 0 : owner === 'BossPartner' ? 1 : -1;
    const boss = figure >= 0 ? bossEditionOf(card, view.game?.boss_rule?.figures?.[figure]?.edition ?? []) : null;
    const mods = own?.mods ?? boss?.mods ?? [];
    const mod_values = own?.mod_values ?? boss?.mod_values ?? {};
    const back = owner === 'Player' ? undefined : (view.game?.backs?.[owner === 'Boss' ? 1 : 2] ?? 'regulars');
    return { ...card, mods, mod_values, ...(back ? { back } : {}), legal: false, from_skat: false, discardable: false, named: this.ui.wildSettled[card.id]?.printed ?? null };
  }

  /** Animate what happened, using only the events and views of the core. */
  private async playEvents(before: RunView | null, events: RunEvent[], after: RunView): Promise<void> {
    const game = before?.game;
    if (game) {
      this.ui.anim = {
        trick: game.trick.slice(),
        hand: game.hand.map((c) => ({ ...c, legal: false, from_skat: false, discardable: false })),
        handSizes: game.hand_sizes.slice(),
        status: '',
        turn: null,
      };
    }
    // Cards a trade cursed: their curse is told by the trade, not again as a Zinke.
    const tradeCursed = new Set<number>();
    // Zinken stamped on in this step, one after another.
    let stamps = 0;
    for (let i = 0; i < events.length; i++) {
      const ev = events[i];
      if (ev === undefined) continue;
      if (typeof ev === 'string') {
        if (ev === 'Re') this.showToast(de.toast.re);
        if (ev === 'Redealt') {
          // Kräuter, Doppelkorn: the old cards are washed off the table first.
          if (this.drinkFx.pouring?.wash.length) {
            this.drinkFx.washAway(this.drinkFx.pouring.wash);
            this.drinkFx.pouring.wash = [];
          }
          this.guidance.passedOnce = true;
          this.showToast(t(de.toast.redealt, { left: after.series.redeals_left }));
          reshuffle();
          this.afterRender.push(() => this.flights.dealEffect());
        }
        if (ev === 'RedealUndone') {
          this.showToast(de.toast.redealUndone);
          reshuffle();
          this.afterRender.push(() => this.flights.dealEffect());
        }
        if (ev === 'RamschDealt') {
          this.showToast(de.toast.ramsch);
          sound.play('knock');
        }
        if (ev === 'EveningRulesEnded') this.showToast(de.toast.eveningEnded);
        if (ev === 'RoutesRerolled') {
          this.showToast(de.toast.routesRerolled);
          sound.play('card');
          this.afterRender.push(() => this.drinkFx.reelRoutes());
        }
        if (ev === 'SkatPeekPlanned') this.showToast(de.toast.skatPeekPlanned);
        continue;
      }
      if ('MarkeGiven' in ev) {
        // A Marke from an encounter (04.11.2026): say which, and where, when drawn.
        const g = ev.MarkeGiven;
        if (g.drawn || g.random_card) {
          const card = after.edition.find((c) => c.id === g.card);
          const text = g.drawn ? de.toast.markeGivenDrawn : de.toast.markeGiven;
          this.showToast(t(text, { mod: contentName('card_mods', g.card_mod), card: card ? cardName(card.kind) : '' }));
        }
        continue;
      }
      if ('CardMadeTransparent' in ev) {
        // "Glasauge": one toast for all cards of this deal, at the first one.
        const prev = events[i - 1];
        if (prev !== undefined && typeof prev === 'object' && 'CardMadeTransparent' in prev) continue;
        const cards: string[] = [];
        for (let j = i; j < events.length; j++) {
          const e = events[j];
          if (e === undefined || typeof e !== 'object' || !('CardMadeTransparent' in e)) break;
          const { card, seat } = e.CardMadeTransparent;
          cards.push(t(de.toast.transparentCard, { name: this.seatLabel(seat, after), card: cardName(card.kind) }));
        }
        this.showToast(t(de.toast.glassEye, { cards: cards.join(', ') }));
        continue;
      }
      if ('CardPeeked' in ev) {
        const { card, seat } = ev.CardPeeked;
        // "Der Zaungast" sees the whole left hand at the table every game:
        // no whisper per card.
        if (after.game?.revealed.some((r) => r.card.id === card.id && r.source === 'Onlooker')) continue;
        this.showToast(t(de.toast.peeked, { name: this.seatLabel(seat, after), card: cardName(card.kind) }));
        continue;
      }
      if ('VoucherReturned' in ev) {
        // "Der Drückeberger": a voucher of the pub just ended is back on its card.
        const id = ev.VoucherReturned.card;
        const card = after.edition.find((c) => c.id === id);
        this.showToast(t(de.toast.voucherReturned, { card: card ? cardName(card.kind) : '', mod: contentName('card_mods', ev.VoucherReturned.card_mod) }), 'top');
        continue;
      }
      if ('VoucherRedeemed' in ev) {
        // Effekte v2: a voucher was redeemed when its card was discarded (or by a hand game).
        const id = ev.VoucherRedeemed.card;
        const card = before?.edition.find((c) => c.id === id) ?? after.edition.find((c) => c.id === id);
        // Declaring a hand game redeems one Marke of the hand (04.11.2026): the card stays in the hand.
        const text = ev.VoucherRedeemed.via_hand ? de.toast.voucherRedeemedHand : de.toast.voucherRedeemed;
        this.showToast(t(text, { card: card ? cardName(card.kind) : '', mod: contentName('card_mods', ev.VoucherRedeemed.card_mod) }), 'top');
        sound.play('clink');
        if (card) await this.redeemVoucher(card, after.edition.find((c) => c.id === id) ?? null);
        continue;
      }
      if ('EveningRuleAdded' in ev) {
        this.showToast(t(de.toast.eveningRule, { name: contentName('relics', ev.EveningRuleAdded) }));
        sound.play('clink');
        continue;
      }
      if ('RelicSwapped' in ev) {
        this.showToast(t(de.toast.relicSwapped, { old: contentName('relics', ev.RelicSwapped.old), new: contentName('relics', ev.RelicSwapped.new) }));
        sound.play('clink');
        continue;
      }
      if ('NextGameDoubled' in ev) {
        this.showToast(t(de.toast.nextDoubled, { factor: ev.NextGameDoubled.factor }));
        sound.play('clink');
        continue;
      }
      if ('TradeCurse' in ev) {
        // Curses as trade: Schnäppchen, Hehlerware, Fusel.
        const { card: id, curse, cause } = ev.TradeCurse;
        const card = after.edition.find((c) => c.id === id);
        tradeCursed.add(id);
        this.showToast(t(de.toast.tradeCurse[cause], { card: card ? cardName(card.kind) : '', curse: contentName('card_mods', curse) }));
        sound.play('clink');
        continue;
      }
      if ('DeckChanged' in ev) {
        const change = ev.DeckChanged;
        if ('Modified' in change && tradeCursed.delete(change.Modified.card)) continue;
        if ('Modified' in change) {
          const card = after.edition.find((c) => c.id === change.Modified.card);
          if (card) this.showToast(t(de.toast.cardMarked, { card: cardName(card.kind), mod: contentName('card_mods', change.Modified.card_mod) }));
          // A Zinke put on at the Tresen, by the Kartenmaler or at an encounter: stamped on.
          const old = before?.edition.find((c) => c.id === change.Modified.card);
          if (card && old && (before?.phase === 'Shop' || before?.phase === 'Encounter')) this.flights.stampLater(old, card, stamps++);
        }
        if ('Cleansed' in change) {
          const card = after.edition.find((c) => c.id === change.Cleansed.card);
          if (card) this.showToast(t(de.toast.cardCleansed, { card: cardName(card.kind) }));
        }
        continue;
      }
      if ('Game' in ev) {
        // "Glasdaus": cards that break with this trick (their events follow it).
        const breaking: number[] = [];
        // "Untot": a card that rises from this trick (its event follows the trick's hooks).
        let rising: RoseEvent | null = null;
        if ('TrickWon' in ev.Game) {
          for (const next of events.slice(i + 1)) {
            if (typeof next !== 'object') break;
            if ('Game' in next) {
              if ('CardRose' in next.Game) rising = next.Game.CardRose;
              break;
            }
          }
          if (!rising) {
            const later = events.slice(i + 1).find((e): e is Extract<RunEvent, { Game: unknown }> => typeof e === 'object' && 'Game' in e);
            if (later && 'CardRose' in later.Game) rising = later.Game.CardRose;
          }
        }
        await this.playGameEvent(ev.Game, after, breaking, rising);
        continue;
      }
      if ('RiseChoice' in ev) {
        // "Untot": the sheet of the core's `rise_choice` asks for the three cards.
        continue;
      }
      if ('CoinsGained' in ev) {
        await this.coinsGained(ev.CoinsGained.source, ev.CoinsGained.amount, ev.CoinsGained.coins, after);
        continue;
      }
      if ('GameDealt' in ev) {
        this.ui.wildSettled = {};
        this.ui.handOrder = null;
        this.ui.risen = null;
        this.riseShown.clear();
        // A new game: from here on the final view tells the story.
        this.ui.anim = null;
        this.ui.view = after;
        resetTableChoices(this.ui);
        this.fx.greetSeries(after);
        reshuffle();
        this.afterRender.push(() => this.flights.dealEffect());
        continue;
      }
      if ('BossAnnounced' in ev) {
        this.showToast(t(de.toast.bossAnnounced, { name: contentName('bosses', ev.BossAnnounced.boss) }));
        continue;
      }
      if ('Kontra' in ev) {
        this.showToast(t(de.toast.kontra, { name: this.seatLabel(ev.Kontra.seat, after) }));
        sound.play('knock');
        await this.fx.pause(DELAY.notice);
        continue;
      }
      if ('RedealsAdded' in ev) {
        this.showToast(t(de.toast.passesAdded, { left: ev.RedealsAdded.redeals_left }));
        continue;
      }
      if ('AccountCleared' in ev) {
        this.showToast(t(de.toast.accountCleared, { before: ev.AccountCleared.before }));
        sound.play('clink');
        continue;
      }
      if ('GamesAdded' in ev) {
        this.showToast(t(de.toast.gamesAdded, { left: ev.GamesAdded.games_left }));
        sound.play('clink');
        continue;
      }
      if ('PubGivenUp' in ev) {
        // The toasts of the lost pub follow (life, or the end of the run).
        continue;
      }
      if ('SkatPeeked' in ev) {
        this.afterRender.push(() => this.drinkFx.revealSkat());
      }
      if ('ConsumableUsed' in ev) {
        const id = ev.ConsumableUsed.consumable;
        // Poured over the table: a factor for the next game slams onto the felt.
        const poured = this.drinkFx.pouring;
        if (poured?.id === id && poured.kind === 'Table') {
          const effect = after.pending_next_game?.find((p) => p.consumable === id)?.effect;
          const factor = effect && typeof effect === 'object' ? ('Multiply' in effect ? effect.Multiply : effect.MultiplyWin) : null;
          if (factor) this.afterRender.push(() => this.drinkFx.slamFactor(factor));
        }
        // Klarer and Absacker have said what they did.
        if (events.some((e) => typeof e === 'object' && ('AccountCleared' in e || 'GamesAdded' in e))) continue;
        // A Schnaps that changes the next game says so; the others just get used.
        const waiting = after.pending_next_game?.find((p) => p.consumable === id);
        this.showToast(
          waiting
            ? t(de.toast.schnapsWaiting, { name: contentName('consumables', id) })
            : t(de.toast.consumableUsed, { name: contentName('consumables', id) }),
        );
        continue;
      }
      if ('RelicBought' in ev) {
        this.showToast(t(de.toast.bought, { name: contentName('relics', ev.RelicBought.relic) }));
        sound.play('coin');
        const bought = this.flights.bought;
        this.flights.bought = null;
        const relic = ev.RelicBought.relic;
        if (bought) {
          this.afterRender.push(() => {
            const chips = this.screenEl.querySelectorAll<HTMLElement>(`.leiste-relics .coaster-chip[data-relic="${CSS.escape(relic)}"]`);
            relicToStrip(bought.html, bought.rect, chips[chips.length - 1] ?? null);
          });
        }
        continue;
      }
      if ('ConsumableBought' in ev) {
        this.showToast(t(de.toast.bought, { name: contentName('consumables', ev.ConsumableBought.consumable) }));
        sound.play('coin');
        const bought = this.flights.bought;
        this.flights.bought = null;
        // Its slot: the one the core filled with it in this step.
        const id = ev.ConsumableBought.consumable;
        const slot = after.consumables.find((c) => c.consumable.id === id && !before?.consumables.some((b) => b.slot === c.slot && b.consumable.id === id))?.slot;
        if (bought) {
          this.afterRender.push(() => {
            const target = slot === undefined ? null : this.screenEl.querySelector<HTMLElement>(`.leiste-slots [data-testid="schnaps-slot-${slot}"]`);
            schnapsToSlot(bought.html, bought.rect, target);
          });
        }
        continue;
      }
      if ('RelicSold' in ev) {
        this.showToast(t(de.toast.sold, { name: contentName('relics', ev.RelicSold.relic), price: ev.RelicSold.price }));
        sound.play('coin');
        continue;
      }
      if ('ConsumableSold' in ev) {
        this.showToast(
          t(de.toast.sold, { name: contentName('consumables', ev.ConsumableSold.consumable), price: ev.ConsumableSold.price }),
        );
        sound.play('coin');
        continue;
      }
      if ('ShopRerolled' in ev) {
        this.showToast(de.toast.rerolled);
        sound.play('card');
        continue;
      }
      if ('CardOfferBought' in ev || 'PainterHired' in ev || 'CardCleansed' in ev) {
        // The card itself is told by the DeckChanged that follows (or the painter's sheet).
        if (!('PainterHired' in ev) || ev.PainterHired.price > 0) sound.play('coin');
        continue;
      }
      if ('CounterRaised' in ev) {
        this.fx.counterRaised(ev.CounterRaised.counter, ev.CounterRaised.value, after);
        continue;
      }
      if ('SkatCardShown' in ev) {
        // "Spickzettel": the card lies face up beside the trick from now on.
        this.fx.wiggleSource(ev.SkatCardShown.source);
        this.showToast(t(de.toast.skatShown, { card: cardName(ev.SkatCardShown.card.kind) }), 'top');
        continue;
      }
      if ('SkatPointsRaised' in ev) {
        // "Skatpfleger": the discards count more, say so at the relic.
        const { source, card_points } = ev.SkatPointsRaised;
        this.fx.fireTriggers([{ source, lever: 'CardPoints', change: { Add: card_points } }], after);
        continue;
      }
      if ('GameScored' in ev) {
        // What fired, on the chips and cards still on screen, before the board covers them.
        // The regulars take the result, a Schneider is cut across the felt; what fired lights up meanwhile.
        let wait = this.fastForward ? 0 : await this.flights.gameEndEffect(ev.GameScored.breakdown);
        if (this.fx.fireTriggers(ev.GameScored.breakdown.lines, after)) wait = Math.max(wait, DELAY.trigger);
        if (wait) await this.fx.pause(wait);
        const rest = events.slice(i + 1);
        const won = rest.find((e): e is Extract<RunEvent, { SeriesWon: unknown }> => typeof e === 'object' && 'SeriesWon' in e);
        // One loss sound per chain (playtest 04.11.2026): when this lost game ends the run
        // (ADR 0019), the end screen sounds it, not the board.
        const runEnds = rest.some((e) => typeof e === 'object' && 'RunLost' in e);
        // One pattern per chain: the pub's result, else the game's.
        const pubLost = runEnds || rest.some((e) => typeof e === 'object' && 'PubLost' in e);
        buzz(won ? 'pubWon' : pubLost ? 'pubLost' : ev.GameScored.breakdown.won ? 'gameWon' : 'gameLost');
        await this.showScore(
          before?.series.score ?? 0,
          ev.GameScored.series_score,
          before?.series.target ?? after.series.target,
          after,
          won ? { payout: won.SeriesWon.payout, coins: won.SeriesWon.coins } : null,
          runEnds,
        );
        continue;
      }
      if ('DebtChanged' in ev) {
        const debt = ev.DebtChanged.debt;
        const took = (before?.debt ?? 0) < debt;
        this.showToast(took ? t(de.toast.debt, { debt }) : debt === 0 ? de.toast.debtPaid : t(de.toast.debtLeft, { debt }));
        continue;
      }
      // ADR 0017: "Die Oma" curses half the edition for the pub, and lifts it after.
      if ('BossCursesLaid' in ev) {
        const { boss, cards } = ev.BossCursesLaid;
        this.showToast(t(de.toast.bossCursed, { name: contentName('bosses', boss), n: cards.length }), 'top');
        sound.play('clink');
        continue;
      }
      if ('BossCursesLifted' in ev) {
        this.showToast(de.toast.bossCursesLifted);
        continue;
      }
      if ('RelicGained' in ev) {
        this.showToast(t(de.toast.relicGained, { name: contentName('relics', ev.RelicGained.relic) }));
        sound.play('clink');
        continue;
      }
      if ('SecondChance' in ev) {
        // ADR 0019: the Schutzengel has the lost pub played again and goes.
        const name = contentName('relics', ev.SecondChance.relic);
        this.showToast(t(de.toast.secondChance, { name }), 'top');
        sound.play('clink');
        if (motionOn()) await this.fx.pause(900);
        continue;
      }
      if ('CoinsStashed' in ev) {
        const { source, amount, pot } = ev.CoinsStashed;
        const name = typeof source === 'object' && 'Relic' in source ? contentName('relics', source.Relic) : '';
        this.fx.wiggleSource(source);
        this.showToast(t(de.toast.stashed, { name, amount, pot }));
        sound.play('coin');
        continue;
      }
      if ('StashPaidOut' in ev) {
        const { relic, amount } = ev.StashPaidOut;
        this.showToast(t(de.toast.stashPaidOut, { name: contentName('relics', relic), amount }));
        sound.play('coin');
        continue;
      }
      if ('NextPubPlanned' in ev) {
        const { redeals, target } = ev.NextPubPlanned;
        const parts: string[] = [];
        if (redeals) parts.push(t(de.toast.nextPubRedeals, { n: redeals > 0 ? `+${redeals}` : `−${-redeals}` }));
        if (target) parts.push(t(de.toast.nextPubTarget, { n: target > 0 ? `+${target}` : `−${-target}` }));
        if (parts.length) this.showToast(t(de.toast.nextPub, { parts: parts.join(', ') }));
        continue;
      }
      if ('WagerPlaced' in ev) {
        const bet = after.wager;
        const text = bet?.on === 'Win' ? de.toast.wagerPlacedWin : de.toast.wagerPlaced;
        this.showToast(t(text, { stake: ev.WagerPlaced.stake }));
        sound.play('coin');
        continue;
      }
      if ('DiceThrown' in ev) {
        const { stake, won, payout } = ev.DiceThrown;
        this.showToast(won ? t(de.toast.diceWon, { payout }) : t(de.toast.diceLost, { stake }), 'top');
        sound.play(won ? 'coin' : 'lose');
        continue;
      }
      if ('WagerSettled' in ev) {
        const { stake, won, payout } = ev.WagerSettled;
        this.showToast(won ? t(de.toast.wagerWon, { payout }) : t(de.toast.wagerLost, { stake }), 'top');
        continue;
      }
      if ('StopsRevealed' in ev) {
        this.showToast(t(de.toast.stopsRevealed, { n: ev.StopsRevealed.legs }));
        continue;
      }
      if ('MalusChosen' in ev) {
        this.showToast(t(de.toast.malusChosen, { name: contentName('maluses', ev.MalusChosen.malus) }));
        continue;
      }
      if ('GambleResolved' in ev) {
        this.showToast(ev.GambleResolved.curse ? de.toast.gambleCurse : de.toast.gambleGood, 'top');
        sound.play(ev.GambleResolved.curse ? 'lose' : 'clink');
        continue;
      }
      if ('SchnappsRewarded' in ev) {
        if (!ev.SchnappsRewarded.slots_full) {
          this.showToast(t(de.toast.schnappsRewarded, { name: contentName('consumables', ev.SchnappsRewarded.consumable) }));
          sound.play('clink');
        }
        continue;
      }
      if ('SchnappsRewardTaken' in ev) {
        this.showToast(
          t(de.toast.schnappsRewardTaken, {
            name: contentName('consumables', ev.SchnappsRewardTaken.consumable),
            old: contentName('consumables', ev.SchnappsRewardTaken.given_up),
          }),
        );
        continue;
      }
      if ('ConsumableGained' in ev) {
        this.showToast(t(de.toast.consumableGained, { name: contentName('consumables', ev.ConsumableGained.consumable) }));
        sound.play('clink');
        continue;
      }
      if ('SeriesStarted' in ev && ev.SeriesStarted.boss && after.series.boss) {
        this.ui.view = after;
        this.ui.anim = null;
        this.ui.screen = 'run';
        const boss = after.series.boss;
        await this.layer.waitOn((resolve) => ({ kind: 'boss', boss, resolve }));
        this.ui.overlay = null;
        // The table opens: lights down, the spotlight onto both boss players
        // (once no dialog is open any more, see render()).
        this.entrancePending = true;
      }
    }
  }

  private async playGameEvent(ev: Extract<RunEvent, { Game: unknown }>['Game'], after: RunView, breaking: number[] = [], rising: RoseEvent | null = null): Promise<void> {
    const anim = this.ui.anim;
    if ('DoppelgangerMirrored' in ev) {
      await this.doppelMirror(ev.DoppelgangerMirrored, after);
      return;
    }
    if ('CardPlayed' in ev) {
      if (!anim) return;
      const { seat, card } = ev.CardPlayed;
      if (anim.trick.length >= 3) anim.trick = [];
      // Where the card comes from, measured before the table changes.
      const from = seat === 0 ? (this.flights.playedFrom ?? this.rectOf('.hand')) : this.rectOf(`.opp[data-seat="${seat}"] .portrait`);
      anim.trick.push({ seat, card: this.cardOf(card, after) });
      anim.handSizes[seat] = Math.max(0, (anim.handSizes[seat] ?? 0) - 1);
      if (seat === 0) anim.hand = anim.hand.filter((c) => c.id !== card.id);
      anim.turn = seat === 0 ? null : seat;
      anim.status = '';
      this.render();
      const el = this.screenEl.querySelector(`[data-trick-seat="${seat}"]`);
      const played = anim.trick[anim.trick.length - 1]?.card;
      if (el instanceof HTMLElement && from && !this.fastForward) this.flights.throwOnto(el, seat, from, played?.back ?? null);
      sound.play('card', { rate: 0.94 + seat * 0.05 });
      await this.fx.pause(seat === 0 ? DELAY.ownCard : DELAY.aiCard);
      return;
    }
    if ('TrickWon' in ev) {
      if (!anim) return;
      const { trick, card_points } = ev.TrickWon;
      anim.turn = null;
      anim.status =
        trick.winner === 0
          ? t(de.table.trickToYou, { points: card_points })
          : t(de.table.trickTo, { name: this.seatLabel(trick.winner, after), points: card_points });
      this.fx.trickQuip(after, trick.winner, card_points);
      if (trick.winner === 0 && !this.fastForward) buzz('trick');
      this.render();
      if (!this.fastForward) this.flights.reactToTrick(after, trick.winner, card_points);
      await this.fx.pause(DELAY.trickShow);
      if (breaking.length) {
        // "Glasdaus": the card cracks and shatters before the trick leaves.
        for (const id of breaking) this.screenEl.querySelector(`.trick-card[data-card-id="${id}"]`)?.classList.add('is-shattering');
        sound.play('clink', { rate: 1.6 });
        await this.fx.pause(DELAY.notice);
      }
      // "Untot": the card rises back into the hand, the substitute takes its place, then the trick leaves.
      if (rising) await this.riseCard(rising, after);
      const target = this.flights.sweepTarget(trick.winner);
      const cards = Array.from(this.screenEl.querySelectorAll('.trick-card'));
      // A card still sliding onto its place lies there now.
      finishOn(cards);
      sound.play('trick');
      // The stack forms on the card that took the trick.
      const onto = cards.findIndex((c) => c.getAttribute('data-trick-seat') === String(trick.winner));
      if (target && !this.fastForward) await Promise.race([sweepAway(cards, target, DELAY.trickFly, onto), this.fx.pause(DELAY.trickFly + 200)]);
      anim.trick = [];
      anim.status = '';
      this.render();
      return;
    }
    if ('Conceded' in ev) {
      this.showToast(de.toast.conceded);
      await this.fx.pause(DELAY.notice);
      return;
    }
    if ('DefenderHandsSwapped' in ev) {
      if (anim) anim.handSizes = ev.DefenderHandsSwapped.hand_sizes.slice();
      this.showToast(de.toast.handsSwapped);
      sound.play('card');
      this.render();
      await this.fx.pause(DELAY.notice);
      return;
    }
    if ('CardRose' in ev) {
      // Shown with its trick (riseCard) unless the trick was not on screen.
      if (!this.riseShown.has(ev.CardRose.card.id)) await this.riseCard(ev.CardRose, after);
      return;
    }
    if ('CardChalked' in ev) {
      const { old, new: painted } = ev.CardChalked;
      this.showToast(t(de.toast.chalked, { old: cardName(old.kind), new: cardName(painted.kind) }));
      return;
    }
    if ('WildCardSettled' in ev) {
      // "Wandelkarte": its suit is settled as the game starts.
      const { old, new: settled } = ev.WildCardSettled;
      const printed = face(old.kind)?.suit;
      if (printed) this.ui.wildSettled = { ...this.ui.wildSettled, [old.id]: { kind: settled.kind, printed } };
      this.showToast(t(de.toast.wildSettled, { old: cardName(old.kind), new: cardName(settled.kind) }));
      if (anim && printed) {
        // The hand shows the settled card in the core's order from now on.
        anim.hand = anim.hand.map((c) => (c.id === old.id ? { ...c, kind: settled.kind, unsettled: false, named: printed } : c));
        const order = after.game?.hand.map((c) => c.id) ?? [];
        if (order.length) anim.hand.sort((a, b) => order.indexOf(a.id) - order.indexOf(b.id));
        this.render();
        await this.cycleWild(old.id, settled.kind);
      }
      return;
    }
    if ('SkatSwapped' in ev) {
      const { took, gave } = ev.SkatSwapped;
      const g = after.game;
      if (anim && g) {
        anim.hand = g.hand.map((c) => ({ ...c, legal: false }));
        anim.handSizes[0] = anim.hand.length;
      }
      this.showToast(t(de.toast.skatSwapped, { took: cardName(took.kind), gave: cardName(gave.kind) }));
      this.render();
      const from = this.rectOf('.deck-spot');
      const el = this.screenEl.querySelector(`.hand-card[data-id="${took.id}"]`);
      if (from && el && !this.fastForward) flyIn(el, from, { ms: 320 });
      sound.play('card');
      await this.fx.pause(DELAY.notice);
      return;
    }
    if ('CardDrawn' in ev) {
      const { seat, card } = ev.CardDrawn;
      if (anim) {
        anim.handSizes[seat] = Math.max(0, (anim.handSizes[seat] ?? 0) - 1);
        this.render();
      }
      this.showToast(t(de.toast.cardDrawn, { card: cardName(card.kind), name: this.seatLabel(seat, after) }), 'top');
      const from = this.rectOf(`.opp[data-seat="${seat}"]`);
      const el = this.screenEl.querySelector(`.skat-zone [data-id="${card.id}"], .hand-card[data-id="${card.id}"]`);
      if (from && el && !this.fastForward) flyIn(el, from, { ms: 360 });
      sound.play('card');
      await this.fx.pause(DELAY.notice);
      return;
    }
    if ('CardReturned' in ev) {
      const { seat, card } = ev.CardReturned;
      if (anim) {
        anim.hand = anim.hand.filter((c) => c.id !== card.id);
        anim.handSizes[0] = anim.hand.length;
        anim.handSizes[seat] = (anim.handSizes[seat] ?? 0) + 1;
        this.render();
      }
      this.showToast(t(de.toast.cardReturned, { card: cardName(card.kind), name: this.seatLabel(seat, after) }), 'top');
      return;
    }
    if ('Discarded' in ev && anim) {
      const gone = ev.Discarded.cards.map((c) => c.id);
      anim.hand = anim.hand.filter((c) => !gone.includes(c.id));
      anim.handSizes[0] = anim.hand.length;
      this.render();
      return;
    }
    if (('Declared' in ev || 'SkatPickedUp' in ev) && anim) {
      // Show the new hand order and the declaration right away.
      const g = after.game;
      // "Kurzer": the skat lay face up already, nothing to turn over.
      const peeked = Boolean(this.ui.view?.game?.skat_peek?.length);
      if (g && 'SkatPickedUp' in ev) {
        anim.hand = g.hand.map((c) => ({ ...c, legal: false }));
        anim.handSizes[0] = anim.hand.length;
        // The skat stays on the table, face up: the cards to press lie there.
        const loose = new Set(g.hand.filter((c) => c.discardable).map((c) => c.id));
        // "Drei drücken": the card drawn from a defender joins them on the table.
        const drawn = g.hand.filter((c) => c.drawn_from != null).map((c) => c.id);
        if (g.phase === 'Discarding') this.ui.selected = [...ev.SkatPickedUp.skat.map((c) => c.id), ...drawn].filter((id) => loose.has(id));
      }
      if (g && this.ui.view?.game) {
        this.ui.view = { ...this.ui.view, game: { ...this.ui.view.game, declaration: g.declaration, phase: g.phase, kontra: g.kontra } };
      }
      // "Hand sortieren: automatisch": the hand takes the core's order for the game declared at once.
      const resort = Boolean(g && 'Declared' in ev && this.ui.settings.handSort === 'auto');
      const spots = resort ? spotsOf(this.screenEl, '.hand .hand-card') : null;
      if (g && resort) {
        const order = g.hand.map((c) => c.id);
        anim.hand.sort((a, b) => order.indexOf(a.id) - order.indexOf(b.id));
      }
      this.render();
      if (spots) glideAll(Array.from(this.screenEl.querySelectorAll<HTMLElement>('.hand .hand-card')), spots, { ms: 320 });
      if ('Declared' in ev && !this.fastForward) this.flights.declareEffect(ev.Declared.declaration);
      if ('SkatPickedUp' in ev && !this.fastForward) {
        this.flights.pickUpFlights(ev.SkatPickedUp.skat, peeked);
        sound.play('card');
        await this.fx.pause(DELAY.flip);
      }
    }
  }

  /**
   * Groschen paid by content (a curse when it is attached): a coin pops up
   * at the purse with the amount and where it came from, the purse shows
   * the new total from the core.
   */
  private async coinsGained(source: Source, amount: number, coins: number, after: RunView): Promise<void> {
    const from = sourceName(source, after.edition);
    this.fx.wiggleSource(source);
    const sign = amount > 0 ? `+${amount}` : `−${Math.abs(amount)}`;
    this.showToast(t(de.toast.coins, { amount: sign, source: from }));
    // The coins fly from what paid them (its chip or card on screen), else from the last tap.
    const payer = this.fx.triggerTargets(source)[0]?.getBoundingClientRect() ?? null;
    if (amount > 0) this.hudFx.gainTo(coins, payer?.width ? payer : null);
    else this.hudFx.loseTo(coins);
    const box = this.screenEl.querySelector('.hud-coins')?.getBoundingClientRect();
    const pop = document.createElement('div');
    pop.className = `coin-pop ${amount < 0 ? 'is-minus' : ''}`;
    pop.dataset.testid = 'coin-pop';
    pop.setAttribute('aria-hidden', 'true');
    pop.innerHTML = `${coinIcon(22)}<b>${esc(sign)}</b><span>${esc(from)}</span>`;
    if (box) {
      pop.style.top = `${Math.round(box.bottom + 6)}px`;
      pop.style.right = `${Math.max(8, Math.round(window.innerWidth - box.right))}px`;
    }
    this.root.appendChild(pop);
    window.setTimeout(() => pop.remove(), Math.max(1400, 2200 * TEMPO_FACTOR[this.ui.settings.tempo]));
    await this.fx.pause(DELAY.notice);
  }

  /**
   * A voucher redeemed (VoucherRedeemed, board "Einlösen in drei
   * Schritten"): the card shows big over the table, its stamp margin tears
   * off, then a coin flies to the Groschen counter, or the card turns over
   * into its new form (Untot, Wandelkarte). Both faces come from the core's
   * views before and after the discard; the Groschen follow as their own
   * event. Skipped with reduced motion and when the table is hurried.
   */
  private async redeemVoucher(card: DeckCard, now: DeckCard | null): Promise<void> {
    if (!motionOn() || this.fastForward) return;
    const from = this.screenEl.querySelector(`[data-id="${card.id}"]`)?.getBoundingClientRect() ?? null;
    const box = document.createElement('div');
    box.className = 'voucher-redeem';
    box.dataset.testid = 'voucher-redeem';
    box.dataset.stage = 'voucher';
    box.setAttribute('aria-hidden', 'true');
    box.innerHTML = `<span class="card">${cardSvg(card.kind, card.mods, 'full', card.mod_values)}</span>`;
    this.root.appendChild(box);
    const el = box.querySelector<HTMLElement>('.card');
    if (!el) return;
    if (from?.width) flyIn(el, from, { ms: 260 });
    await this.fx.pause(DELAY.notice * 0.5);
    // Step 2: the stamp margin tears off.
    el.innerHTML = cardSvg(card.kind, card.mods, 'full', card.mod_values, { redeeming: true });
    box.dataset.stage = 'torn';
    el.animate([{ transform: 'rotate(0)' }, { transform: 'rotate(-4deg)' }, { transform: 'rotate(3deg)' }, { transform: 'rotate(0)' }], { duration: 260, easing: 'ease-out' });
    sound.play('card', { rate: 1.3 });
    await this.fx.pause(DELAY.notice * 0.6);
    // Step 3: what the voucher gives.
    if (now) {
      const turned = now.mods.some((m) => !card.mods.includes(m));
      if (turned) {
        el.innerHTML = cardSvg(now.kind, now.mods, 'full', now.mod_values);
        flip(el, cardBack(), { ms: 360 });
        box.dataset.stage = 'turned';
      } else {
        el.innerHTML = cardSvg(now.kind, now.mods, 'full', now.mod_values);
        box.dataset.stage = 'coin';
        const purse = this.screenEl.querySelector('.hud-coins')?.getBoundingClientRect();
        if (purse) {
          const coin = document.createElement('span');
          coin.className = 'voucher-coin';
          coin.innerHTML = coinIcon(28);
          coin.style.left = `${Math.round(purse.left + purse.width / 2 - 14)}px`;
          coin.style.top = `${Math.round(purse.top + purse.height / 2 - 14)}px`;
          this.root.appendChild(coin);
          const anim = flyIn(coin, el.getBoundingClientRect(), { ms: 420, arc: 40 });
          const gone = (): void => coin.remove();
          if (anim) anim.finished.then(gone, gone);
          else gone();
        }
      }
    }
    await this.fx.pause(DELAY.notice * 0.8);
    box.remove();
  }

  /**
   * "Untot" (CardRose): the card lifts out of the trick back into the hand,
   * the substitute from the hand turns over into its place; a toast above
   * the Stammgäste says so. Everything from the event and the core's view.
   */
  private async riseCard(rose: RoseEvent, after: RunView): Promise<void> {
    const { card, substitute } = rose;
    // Since 03.11.2026 it comes back as a copy of the trick's winning card.
    const risen = rose.risen ?? card;
    this.riseShown.add(card.id);
    this.ui.risen = { substitute: substitute.id, card: cardName(card.kind) };
    this.showToast(t(de.toast.cardRose, { card: cardName(card.kind), risen: cardName(risen.kind), substitute: cardName(substitute.kind) }), 'top');
    const anim = this.ui.anim;
    if (!anim) return;
    const subFrom = this.screenEl.querySelector(`.hand-card[data-id="${substitute.id}"]`)?.getBoundingClientRect() ?? null;
    // The card as it was played crumbles big over the table into the
    // zombie card it comes back as (playtest 05.11.2026), then that one
    // flies into the hand; without motion or hurried it is just there.
    // The pick sheet goes first (the run is busy now), so the scene is seen.
    if (this.screenEl.querySelector('.sheet-rise')) this.render();
    const played = this.screenEl.querySelector(`.trick-card[data-card-id="${card.id}"]`)?.getBoundingClientRect() ?? null;
    const morphed = this.fastForward ? null : await crumbleInto(this.root, lookSvg(this.cardOf(card, after)), lookSvg({ ...this.cardOf(risen, after), risen: true }));
    const from = morphed ?? played;
    anim.trick = anim.trick.map((p) => (p.card.id === card.id ? { seat: p.seat, card: this.cardOf(substitute, after) } : p));
    // The trick's Augen changed with the card; the core's count follows with the score.
    anim.status = '';
    const hand = after.game?.hand;
    anim.hand = hand
      ? hand.map((c) => ({ ...c, legal: false }))
      : [...anim.hand.filter((c) => c.id !== substitute.id), { ...this.cardOf(risen, after), can_rise: false, risen: true }];
    // A short green glow while it climbs back into the hand (style.css; with reduced motion a still outline).
    this.ui.rising = card.id;
    this.render();
    if (!this.fastForward) {
      const back = this.screenEl.querySelector(`.hand-card[data-id="${card.id}"]`);
      if (back && from) flyIn(back, from, { ms: 520, arc: -46 });
      const sub = this.screenEl.querySelector<HTMLElement>(`.trick-card[data-card-id="${substitute.id}"]`);
      if (sub) {
        if (subFrom) flyIn(sub, subFrom, { ms: 300, delay: 120 });
        flip(sub, cardBack(), { ms: 320, delay: 160 });
      }
    }
    sound.play('card', { rate: 0.8 });
    await this.fx.pause(DELAY.notice);
    this.ui.rising = null;
  }

  /**
   * "Doppelgänger", Spiegelbild (DoppelgangerMirrored): the played card
   * flashes in its violet mirror look with a shimmer, "Gespiegelt!"; then
   * its CardPlayed throws the copy (the copied face with the mirror look)
   * into the trick. Tempo-scaled, off with reduced motion, a tap ends it.
   */
  private async doppelMirror(m: DoppelMirror, after: RunView): Promise<void> {
    this.showToast(t(de.toast.doppelMirrored, { card: cardName(m.card.kind), copied: cardName(m.copied.kind) }), 'top');
    const rect = this.flights.playedFrom ?? this.rectOf(`.hand-card[data-id="${m.card.id}"]`);
    if (this.fastForward || !rect) return;
    sceneWord(de.doppel.mirrored, rect, 'doppel-word');
    await mirrorShimmer(lookSvg(this.cardOf(m.card, after)), rect);
  }

  private seatLabel(seat: Seat, view: RunView): string {
    return seatLabelOf(view, seat);
  }

  private rectOf(selector: string): DOMRect | null {
    const el = this.screenEl.querySelector(selector);
    return el ? el.getBoundingClientRect() : null;
  }

  private async showScore(
    previousScore: number,
    seriesScore: number,
    target: number,
    after: RunView,
    payout: ScoreOverlay['payout'],
    quietLoss = false,
  ): Promise<void> {
    const score = after.last_score;
    if (!score) return;
    const seatNames = ([0, 1, 2] as const).map((seat) => this.seatLabel(seat, after));
    await this.layer.waitOn((resolve) => ({ kind: 'score', score, previousScore, seriesScore, target, payout, relics: after.relics, seatNames, quietLoss, resolve }));
    this.layer.scoreBoard = null;
    this.ui.overlay = null;
    // The board goes; the account in the head counts to the new value.
    this.layer.render();
    if (previousScore !== seriesScore && this.screenEl.querySelector('.hud [data-testid="account"]')) {
      void this.hudFx.account(seriesScore, target);
      if (motionOn()) await this.fx.pause(520);
    }
  }

  /**
   * "Wandelkarte": as the core settles its suit, the card in the hand runs
   * through the four suits' colours (about 0.8 s) and stops on the new one.
   * Already drawn settled, so reduced motion and hurrying just skip it.
   */
  private async cycleWild(id: number, kind: CardKind): Promise<void> {
    const f = face(kind);
    const card = this.screenEl.querySelector<HTMLElement>(`.hand [data-id="${id}"]`);
    const svg = card?.querySelector('svg.card-svg');
    if (!f || !card || !svg || reducedMotion() || this.fastForward) return;
    const steps = 8;
    const last = SUITS.indexOf(f.suit);
    card.classList.add('is-settling');
    sound.play('card', { rate: 1.4 });
    for (let i = 0; i < steps; i++) {
      const suit = SUITS[(last - (steps - 1 - i) + 4 * steps) % 4] ?? f.suit;
      showWildSuit(svg, f.rank, suit);
      if (this.fastForward) break;
      await this.fx.pause(i === steps - 1 ? 180 : 85);
    }
    showWildSuit(svg, f.rank, f.suit);
    card.classList.remove('is-settling');
  }

  /** Store what the playback kept in memory (./app/progress.ts). */
  persist(): void {
    this.progress.persist();
  }

  /** A short message (./app/playback/fx.ts). */
  showToast(text: string, place: ToastPlace = 'middle'): void {
    this.fx.showToast(text, place);
  }

  private setSettings(change: Partial<UiState['settings']>): void {
    this.ui.settings = { ...this.ui.settings, ...change };
    saveSettings(this.ui.settings);
    this.applySettings();
    this.layer.markDirty();
    this.render();
  }

  // -------------------------------------------------------------------------
  // Hints of the Wirt
  // -------------------------------------------------------------------------

  // -------------------------------------------------------------------------
  // Die Einführung (DESIGN 16.1)
  // -------------------------------------------------------------------------

  /**
   * Start the Einführung: the core's guided pub with fixed deals. A run in
   * progress stays stored untouched; the tutorial run is never saved and
   * its events never reach the meta state.
   */
  private startTutorial(): void {
    if (this.ui.busy) return;
    let core: CoreRun;
    try {
      core = CoreRun.tutorial();
    } catch (err) {
      console.warn('tutorial not started', err);
      this.showToast(errorText(err));
      return;
    }
    markTutorial('done');
    this.persist();
    this.core?.dispose();
    this.core = core;
    this.guidance.startCoach();
    this.ui.tutorialOffer = false;
    this.ui.message = null;
    this.ui.ended = null;
    this.ui.copy = null;
    this.enterRun(core.view(), true);
  }

  /** The coach goes; the bubble and ring with him. */
  private endTutorial(): void {
    this.guidance.endCoach();
  }

  /**
   * Leave the Einführung (skipped, or finished): to the start screen, or
   * into a real evening ('run') when no run in progress would be given up.
   * During a playback the core is let go first, the rest waits for its end.
   */
  private leaveTutorial(go: 'start' | 'run'): void {
    this.endTutorial();
    markTutorial('done');
    if (this.ui.busy) {
      this.leaveTutorialTo = go;
      this.core?.dispose();
      this.core = null;
      const o = this.ui.overlay;
      if ((o?.kind === 'score' || o?.kind === 'boss') && this.layer.awaited === o) o.resolve();
      this.fx.hurry();
      return;
    }
    this.afterTutorial(go);
  }

  private afterTutorial(go: 'start' | 'run'): void {
    this.toStart();
    if (go === 'run' && !this.hasRunningRun()) this.newRun();
  }

  // -------------------------------------------------------------------------
  // Input
  // -------------------------------------------------------------------------

  private onClick(e: MouseEvent): void {
    const target = e.target as HTMLElement | null;
    const el = target?.closest<HTMLElement>('[data-action]');
    const overlay = this.ui.overlay;
    if (!el) {
      if (this.ui.busy && !overlay) this.fx.hurry();
      return;
    }
    const action = el.dataset.action ?? '';
    this.hudFx.noteTap(el);
    // Cards that can be dragged spring here, not with :active (that would shrink a drag).
    if (el.matches('[data-drag]')) spring(el);
    // A tap on the backdrop of a dialog only counts if it hit the backdrop.
    if (action === 'close-overlay' && el.classList.contains('overlay') && target !== el) {
      return;
    }
    if (el instanceof HTMLButtonElement && el.disabled) return;
    const num = (key: string): number => Number(el.dataset[key] ?? '-1');
    const view = this.ui.view;
    switch (action) {
      case 'new-run':
        this.endTutorial();
        this.newRun();
        return;
      // Die Einführung (DESIGN 16.1).
      case 'tutorial-start':
        this.startTutorial();
        return;
      case 'tutorial-offer-no':
        markTutorial('declined');
        this.ui.tutorialOffer = false;
        this.render();
        return;
      case 'coach-next':
        this.guidance.nextCoach();
        return;
      case 'tutorial-skip':
        this.leaveTutorial('start');
        return;
      case 'tutorial-finish':
        this.leaveTutorial(el.dataset.go === 'run' ? 'run' : 'start');
        return;
      // Die lange Nacht (ADR 0020): on from the end screen of a won run.
      case 'enter-night':
        this.enterNight();
        return;
      case 'resume':
        this.resume();
        return;
      case 'play-shared': {
        const shared = this.ui.shared;
        if (shared) this.requestStart({ seed: shared.seed, character: shared.locked ? 'Declarer' : shared.character, stake: sharedStake(this.ui), daily: false });
        return;
      }
      case 'dismiss-shared':
        this.ui.shared = null;
        clearShareHash();
        this.render();
        return;
      case 'play-daily':
        this.requestStart({ seed: dailySeed(), character: this.ui.pickCharacter, stake: this.pickedStake(), daily: true });
        return;
      case 'replay': {
        const c = el.dataset.character ?? null;
        const seed = el.dataset.seed ?? '';
        const entry = this.ui.meta?.characters.find((x) => x.character === c);
        // Replayed at its stake, or the highest unlocked if that is lower.
        const stake = isCharacter(c) ? Math.min(num('stake') < 0 ? 0 : num('stake'), maxStake(this.ui.meta, c)) : 0;
        if (isCharacter(c) && entry?.unlocked && isSeedText(seed)) this.requestStart({ seed, character: c, stake, daily: false });
        return;
      }
      case 'confirm-new':
        if (overlay?.kind === 'confirm-new') {
          const start = overlay.start;
          this.ui.overlay = null;
          clearRun();
          this.startRun(start);
        }
        return;
      case 'copy-link':
        this.copyLink(el.dataset.url ?? '');
        return;
      case 'share-run':
        void this.shareRun(el);
        return;
      case 'share-native':
        this.shareNative(el.dataset.url ?? '', el.dataset.text ?? '');
        return;
      case 'book-tab':
        if (overlay?.kind === 'book') {
          this.ui.overlay = { kind: 'book', tab: el.dataset.tab === 'stats' ? 'stats' : 'book' };
          this.render();
        }
        return;
      case 'thinking':
        this.setSettings({ thinking: el.dataset.on === 'true' });
        return;
      case 'hints':
        this.setSettings({ hints: el.dataset.on === 'true' });
        return;
      case 'suit-tint':
        this.setSettings({ suitTint: el.dataset.on === 'true' });
        return;
      case 'vibrate':
        this.setSettings({ vibrate: el.dataset.on === 'true' });
        if (el.dataset.on === 'true') buzz('trick');
        return;
      case 'hand-sort':
        this.setSettings({ handSort: el.dataset.sort === 'manual' ? 'manual' : 'auto' });
        return;
      case 'card-back':
        this.setSettings({ back: el.dataset.back ? el.dataset.back : null });
        return;
      case 'hints-reset':
        this.guidance.resetHints();
        return;
      case 'hint-dismiss':
        this.guidance.dismissHint();
        return;
      case 'rules':
        this.ui.overlay = { kind: 'rules' };
        this.render();
        return;
      case 'glossary':
        this.ui.overlay = { kind: 'glossary' };
        this.render();
        return;
      case 'to-start':
        this.endTutorial();
        this.toStart();
        return;
      case 'menu':
      case 'settings':
        if (this.ui.busy) return;
        this.ui.overlay = { kind: 'menu' };
        this.render();
        return;
      case 'overview':
        if (this.ui.busy || !view) return;
        this.ui.overlay = { kind: 'overview' };
        this.render();
        return;
      case 'save-menu':
        if (this.ui.busy) return;
        this.ui.overlay = { kind: 'save-menu' };
        this.render();
        return;
      case 'account':
        if (this.ui.busy) return;
        this.account.openAccount();
        return;
      case 'account-create':
        void this.account.createAccount();
        return;
      case 'account-signin':
        void this.account.signInWithPasskey();
        return;
      case 'account-recover':
        this.ui.overlay = { kind: 'account-recover', busy: false, error: null };
        this.render();
        return;
      case 'account-recover-go':
        void this.account.recoverWithCode();
        return;
      case 'account-code-copy':
        this.account.copyRecoveryCode();
        return;
      case 'account-code-done':
        this.account.codeDone();
        return;
      case 'account-add-passkey':
        void this.account.addPasskeyHere();
        return;
      case 'account-name-save':
        void this.account.saveName();
        return;
      case 'account-new-code':
        void this.account.newRecoveryCode();
        return;
      case 'account-signout':
        void this.account.signOutHere();
        return;
      case 'account-signout-all':
        void this.account.signOutEverywhereNow();
        return;
      case 'account-passkeys':
        if (overlay?.kind === 'account' && overlay.busy) return;
        void this.account.openPasskeys();
        return;
      case 'account-passkey-remove':
        if (el.dataset.id) void this.account.removePasskeyNow(el.dataset.id);
        return;
      case 'account-delete':
        this.ui.overlay = { kind: 'account-delete', busy: false, error: null };
        this.render();
        return;
      case 'account-delete-go':
        void this.account.deleteAccountNow();
        return;
      case 'sync-conflict':
        if (this.ui.busy) return;
        this.account.openConflict();
        return;
      case 'sync-keep-here':
        void this.account.keepHere();
        return;
      case 'sync-keep-server':
        this.account.keepServer();
        return;
      case 'save-export':
        if (this.ui.busy || overlay?.kind !== 'save-menu') return;
        void this.saveCode.openSaveExport();
        return;
      case 'save-copy':
        this.saveCode.copySave();
        return;
      case 'save-share':
        this.saveCode.shareSave();
        return;
      case 'save-select':
        this.saveCode.selectSaveCode();
        return;
      case 'save-import':
        if (this.ui.busy) return;
        this.ui.overlay = { kind: 'save-import', text: '', error: null, checking: false };
        this.render();
        return;
      case 'save-check':
        void this.saveCode.checkImport();
        return;
      case 'save-apply':
        this.saveCode.applyImport();
        return;
      case 'credits':
        this.ui.overlay = { kind: 'credits' };
        this.render();
        return;
      case 'book':
        if (this.ui.busy || !this.ui.meta) return;
        this.ui.overlay = { kind: 'book', tab: 'book' };
        this.render();
        return;
      case 'pick-stake': {
        const max = maxStake(this.ui.meta, this.ui.pickCharacter);
        const stake = num('stake');
        if (stake >= 0 && stake <= max) {
          this.ui.pickStake = stake;
          this.render();
        }
        return;
      }
      case 'char-scroll':
        scrollCharacterPicker(this.screenEl, num('dir'), !reducedMotion());
        return;
      case 'pick-character': {
        const c = el.dataset.character ?? null;
        const entry = this.ui.meta?.characters.find((x) => x.character === c);
        if (isCharacter(c) && entry?.unlocked) {
          this.ui.pickCharacter = c;
          this.ui.pickStake = null;
          saveCharacter(c);
          this.render();
        }
        return;
      }
      case 'info-rule':
      case 'info-evening':
      case 'info-boss':
      case 'info-relic':
      case 'info-offer-relic':
      case 'info-offer-item':
      case 'info-item':
      case 'info-malus':
      case 'info-night':
      case 'info-card': {
        // A tap on these opens their info card (the same as a long-press).
        const key = el.dataset.info;
        if (key) this.info.show(key, 'sheet', el);
        return;
      }
      case 'tempo':
        this.setSettings({ tempo: (el.dataset.tempo ?? 'normal') as Tempo });
        return;
      case 'sound':
        this.setSettings({ sound: el.dataset.on === 'true' });
        if (el.dataset.on === 'true') sound.play('clink');
        return;
      case 'music':
        this.setSettings({ music: el.dataset.on === 'true' });
        return;
      case 'close-overlay':
        if (overlay && overlay.kind !== 'score' && overlay.kind !== 'boss') {
          this.ui.overlay = null;
          this.render();
        }
        return;
      case 'fast':
        if (this.ui.busy) this.fx.hurry();
        // A tap on the open trick while nothing plays: the last one, as with "Letzter Stich".
        else if (!this.ui.anim && view?.game?.phase === 'Playing' && view.game.last_trick) {
          this.ui.peekTrick = true;
          this.render();
        }
        return;
      case 'last-trick':
        // Show the last finished trick on the felt, or the open one again.
        if (this.ui.anim || this.ui.busy || !view?.game?.last_trick) return;
        this.ui.peekTrick = !this.ui.peekTrick;
        this.render();
        return;
      case 'score-skip':
        if (overlay?.kind === 'score' && this.layer.scoreBoard) this.layer.scoreBoard.skip();
        else this.layer.redraw();
        return;
      case 'score-continue':
      case 'boss-go':
        this.layer.closeAwaited(action === 'score-continue' ? 'score' : 'boss');
        return;
      case 'scroll-goods':
        this.root.querySelector('#shop-goods')?.scrollIntoView({ behavior: motionOn() ? 'smooth' : 'auto', block: 'start' });
        return;
    }
    if (this.ui.busy || !view) return;
    switch (action) {
      case 'pick-game':
        this.ui.pickGame = num('index');
        this.render();
        return;
      case 'pick-risk':
        this.ui.pickRisk = (el.dataset.risk ?? 'Normal') as RiskLevel;
        this.render();
        return;
      case 'pick-null-risk':
        this.ui.pickNullRisk = (el.dataset.risk ?? 'Normal') as NullRisk;
        this.render();
        return;
      case 'pick-up-skat':
        void this.act('PickUpSkat');
        return;
      case 'give-up':
        // "Kneipe verlassen" (DESIGN 8.7): ask first, a boss pub ends the run.
        this.ui.overlay = { kind: 'confirm-give-up' };
        this.render();
        return;
      case 'confirm-give-up':
        this.ui.overlay = null;
        void this.act('GiveUpPub');
        return;
      case 'play-hand':
        this.ui.handGame = true;
        this.render();
        return;
      case 'hand-back':
        this.ui.handGame = false;
        this.ui.pickGame = null;
        this.ui.pickRisk = null;
        this.ui.pickNullRisk = null;
        this.render();
        return;
      case 'declare-null':
        void this.act({ Declare: { game: 'Null', risk: (el.dataset.risk ?? 'Normal') as NullRisk } });
        return;
      case 'oma':
        this.ui.omaPick = true;
        this.render();
        return;
      case 'settle':
        void this.settle();
        return;
      case 'oma-cancel':
        this.ui.omaPick = false;
        this.render();
        return;
      case 'swap-take':
        this.ui.swapTake = num('id');
        this.render();
        return;
      case 'oma-give':
        void this.act({ TakeFromSkat: { give: num('id'), take: this.ui.swapTake } });
        return;
      case 'pick-route':
        this.ui.routePick = num('option');
        this.ui.routeLeg = legKey(view);
        this.render();
        return;
      case 'route-overview':
      case 'route-map':
        this.ui.routeOverview = action === 'route-overview';
        this.ui.routeLeg = legKey(view);
        this.render();
        return;
      case 'choose-route':
        void this.travel(num('option'));
        return;
      // The relic pick after a won boss pub (ADR 0017).
      case 'boss-relic': {
        const replace = el.dataset.replace === undefined ? null : num('replace');
        this.ui.rewardPick = null;
        void this.act({ ChooseBossRelic: { offer: num('offer'), replace } });
        return;
      }
      case 'boss-relic-pick':
        this.ui.rewardPick = num('offer');
        this.render();
        return;
      case 'boss-relic-back':
        this.ui.rewardPick = null;
        this.render();
        return;
      case 'boss-relic-skip':
        this.ui.rewardPick = null;
        void this.act({ ChooseBossRelic: { offer: null, replace: null } });
        return;
      // The card reward after a won pub (DESIGN 5.0, fixed pairs since 05.11.2026): an offer, then "Anbringen".
      case 'card-reward-pick': {
        const o = view.card_reward?.offers.find((x) => x.index === num('offer'));
        if (!o || !o.can_take) return;
        this.flights.noteStamp(el.querySelector('.card'));
        this.mods.applyMod(view, { TakeCardReward: { offer: o.index } }, o.card.id, o.card_mod.id, false, null, 'apply', { pair: true });
        return;
      }
      case 'card-reward-skip':
        this.ui.overlay = null;
        void this.act('SkipCardReward');
        return;
      case 'choose-encounter': {
        const option = view.encounter?.options.find((o) => o.index === num('option'));
        if (!option) return;
        if (markeEffect(option)?.marke === 'Chosen') {
          // A Marke of the player's choice first (04.11.2026), then the card or the draw.
          this.ui.overlay = { kind: 'encounter-marke', option: option.index };
          this.render();
        } else if (option.target === 'Card') {
          const marks = option.effects.some((e) => e.card_mod);
          const reasons = marks ? this.mods.refusals(view, option.targets, (id) => ({ ChooseEncounter: { option: option.index, target: id } })) : {};
          this.ui.overlay = { kind: 'encounter-target', option: option.index, reasons };
          this.render();
        } else if (option.target === 'Relic') {
          this.ui.overlay = { kind: 'encounter-relic', option: option.index };
          this.render();
        } else {
          void this.act({ ChooseEncounter: { option: option.index, target: null } });
        }
        return;
      }
      case 'encounter-target': {
        const o = this.ui.overlay;
        if (o?.kind !== 'encounter-target') return;
        const option = view.encounter?.options.find((x) => x.index === o.option);
        const action: RunAction = { ChooseEncounter: { option: o.option, target: num('id'), marke: o.marke ?? null } };
        this.flights.noteStamp(el);
        // The Zinke on the chosen card (a Painter or cleansing names none); a chosen Marke.
        const chosen = option?.effects.find((e) => typeof e.effect === 'object' && 'Mark' in e.effect && e.effect.Mark.card === 'Chosen');
        // A Marke drawn when taken: no preview of the card afterwards (04.11.2026).
        const drawn = option ? markeEffect(option)?.marke === 'Random' : false;
        if (drawn && option) {
          const marken = (option.marken ?? []).filter((m) => m.can_choose).map((m) => contentName('card_mods', m.card_mod.id) || m.card_mod.name);
          const names = marken.length > 1 ? `${marken.slice(0, -1).join(', ')} ${de.encounter.or} ${marken[marken.length - 1]}` : (marken[0] ?? '');
          this.mods.applyRandomMarke(view, action, num('id'), names, o);
          return;
        }
        // The card before and after, then "Anbringen" (playtest 04.11.2026).
        this.mods.applyMod(view, action, num('id'), o.marke ?? chosen?.card_mod?.id ?? null, false, o, 'apply');
        return;
      }
      case 'encounter-marke': {
        const o = this.ui.overlay;
        if (o?.kind !== 'encounter-marke') return;
        const option = view.encounter?.options.find((x) => x.index === o.option);
        const marke = el.dataset.cardMod;
        if (!option || !marke) return;
        if (option.target === 'Card') {
          // Then the card picker, with the cards this Marke may go on.
          const reasons = this.mods.refusals(view, option.marken?.find((m) => m.card_mod.id === marke)?.targets ?? [], (id) => ({ ChooseEncounter: { option: option.index, target: id, marke } }));
          this.ui.overlay = { kind: 'encounter-target', option: option.index, reasons, marke };
          this.render();
        } else {
          void this.act({ ChooseEncounter: { option: option.index, target: null, marke } });
        }
        return;
      }
      case 'encounter-relic': {
        const o = this.ui.overlay;
        if (o?.kind !== 'encounter-relic') return;
        void this.act({ ChooseEncounter: { option: o.option, target: num('index') } });
        return;
      }
      case 'mod-confirm-go': {
        const o = this.ui.overlay;
        // The Zinke comes down on the card shown "Danach".
        if (o?.kind === 'mod-confirm') this.flights.noteStamp(this.layerEl.querySelector('.mod-confirm-card:last-of-type .card'));
        if (o?.kind === 'mod-confirm') void this.act(o.action);
        return;
      }
      case 'mod-confirm-back': {
        const o = this.ui.overlay;
        if (o?.kind === 'mod-confirm') {
          this.ui.overlay = o.back;
          this.render();
        }
        return;
      }
      case 'declare': {
        const options = view.game?.declare;
        const game = this.ui.pickGame === null ? undefined : options?.games[this.ui.pickGame];
        if (!options || !game) return;
        const risk = this.ui.pickRisk ?? options.risks.find((r) => r.allowed)?.risk ?? 'Normal';
        void this.act({ Declare: { game: game.game, risk } });
        return;
      }
      case 'pass':
        // Einmischen: one tap on the deck (or the button) mixes in at once.
        void this.act('Redeal');
        return;
      case 're':
        void this.act({ AnswerKontra: { re: el.dataset.re === 'true' } });
        return;
      case 'rise-pick': {
        // "Untot": a tap picks or drops a hand card, at most `pick` of them.
        const id = num('id');
        const need = view.game?.rise_choice?.pick ?? 3;
        const pick = this.ui.risePick;
        this.ui.risePick = pick.includes(id) ? pick.filter((x) => x !== id) : pick.length < need ? [...pick, id] : pick;
        // Only the tapped card and the button change: no re-render, the sheet does not enter again.
        const sheetEl = this.screenEl.querySelector<HTMLElement>('.sheet-rise');
        if (sheetEl) updateRiseChoice(sheetEl, this.ui.risePick, need);
        else this.render();
        return;
      }
      case 'rise-go': {
        const [a, b, c] = this.ui.risePick;
        if (a === undefined || b === undefined || c === undefined) return;
        this.ui.risePick = [];
        void this.act({ ChooseRiseCards: [a, b, c] });
        return;
      }
      case 'select': {
        // A tap moves a card between hand and table (the fallback for dragging).
        const id = num('id');
        const sel = this.ui.selected;
        const need = view.game ? discardNeed(view.game) : 2;
        if (sel.includes(id)) this.table.moveCards(sel.filter((x) => x !== id), id, null);
        else if (sel.length < need) this.table.moveCards([...sel, id], id, null);
        else this.table.moveCards([...sel.slice(1), id], id, null);
        return;
      }
      case 'discard': {
        const sel = this.ui.selected;
        const need = view.game ? discardNeed(view.game) : 2;
        const [a, b, c] = sel;
        if (sel.length !== need || a === undefined || b === undefined) return;
        if (need === 3 && c !== undefined) void this.act({ DiscardThree: [a, b, c] });
        else void this.act({ Discard: [a, b] });
        return;
      }
      case 'choose-malus':
        void this.act({ ChooseMalus: { option: num('option'), malus: num('malus') } });
        return;
      case 'play':
        this.table.playCard(num('id'), el.getBoundingClientRect(), null);
        return;
      case 'buy-relic':
        this.flights.noteBought(el.closest('.relic-tile')?.querySelector('.relic-tile-emblem svg') ?? null);
        void this.act({ BuyRelic: num('slot') });
        return;
      case 'replace-relic-pick':
        this.ui.overlay = { kind: 'relic-replace', slot: num('slot') };
        this.render();
        return;
      case 'replace-relic':
        this.ui.overlay = null;
        void this.act({ ReplaceRelic: { slot: num('slot'), replace: num('replace') } });
        return;
      case 'buy-item':
        this.flights.noteBought(el.closest('.good')?.querySelector('.good-pic svg') ?? null);
        void this.act({ BuyConsumable: num('slot') });
        return;
      case 'sell-relic':
        void this.act({ SellRelic: num('index') });
        return;
      case 'sell-item':
        void this.act({ SellConsumable: num('slot') });
        return;
      case 'lift-item':
        // A tap on a drinkable Schnaps lifts it; the next tap on its target pours it (./ui/drink.ts).
        if (!this.ui.busy) this.drink.lift(num('slot'));
        return;
      case 'use-item': {
        const slot = num('slot');
        const item = view.consumables.find((c) => c.slot === slot);
        if (!item) return;
        if (item.consumable.needs_suit) {
          this.ui.overlay = { kind: 'chalk', slot, card: null };
          this.render();
        } else if (item.consumable.needs_target) {
          const marks = consumableMod(item.consumable.id);
          const reasons = marks.mod || marks.random ? this.mods.refusals(view, item.targets, (id) => ({ UseConsumable: { slot, target: id } })) : {};
          this.ui.overlay = { kind: 'target', slot, reasons };
          this.render();
        } else {
          void this.act({ UseConsumable: { slot, target: null } });
        }
        return;
      }
      case 'target': {
        const o = this.ui.overlay;
        if (o?.kind !== 'target') return;
        const item = view.consumables.find((c) => c.slot === o.slot);
        const marks = item ? consumableMod(item.consumable.id) : { mod: null, random: false };
        this.flights.noteStamp(el);
        const action: RunAction = { UseConsumable: { slot: o.slot, target: num('id') } };
        if (marks.mod || marks.random) this.mods.applyMod(view, action, num('id'), marks.mod, marks.random, o, 'apply');
        else void this.act(action);
        return;
      }
      case 'chalk-card': {
        const o = this.ui.overlay;
        if (o?.kind === 'chalk') {
          this.ui.overlay = { ...o, card: num('id') };
          this.render();
        }
        return;
      }
      case 'chalk-suit': {
        const o = this.ui.overlay;
        const suit = el.dataset.suit as Suit | undefined;
        if (o?.kind === 'chalk' && o.card !== null && suit) void this.act({ UseConsumable: { slot: o.slot, target: o.card, suit } });
        return;
      }
      case 'reroll':
        void this.act('Reroll');
        return;
      case 'zinke-tile':
      case 'buy-zinke': {
        // Zinken (DESIGN 8.6): the card is fixed; like the card reward (05.11.2026) the
        // tile opens the sheet with the card before and after, then "Kaufen".
        const offer = view.shop?.card_offers?.find((o) => o?.slot === num('slot'));
        if (!offer || !offer.can_buy) return;
        this.flights.noteStamp(el.closest('.zinke')?.querySelector('.zinke-card') ?? null);
        this.mods.applyMod(view, { BuyCardOffer: offer.slot }, offer.card.id, offer.card_mod.id, false, null, 'apply', { pair: true, price: offer.price });
        return;
      }
      case 'painter':
      case 'cleanse': {
        const service: 'painter' | 'cleanse' = action === 'painter' ? 'painter' : 'cleanse';
        const s = service === 'painter' ? view.shop?.painter : view.shop?.cleanse;
        if (!s) return;
        const reasons = this.mods.refusals(view, s.targets, (id) => (service === 'painter' ? { HirePainter: id } : { CleanseCard: id }));
        this.ui.overlay = { kind: 'zinken-card', service, reasons };
        this.render();
        return;
      }
      case 'zinken-card': {
        const o = this.ui.overlay;
        if (o?.kind !== 'zinken-card') return;
        // The Kartenmaler's sheet shows each Zinke on the card; cleansing shows the card before and after first.
        if (o.service === 'painter') void this.act({ HirePainter: num('id') });
        else {
          this.flights.noteStamp(el);
          this.mods.applyMod(view, { CleanseCard: num('id') }, num('id'), null, false, o, 'cleanse');
        }
        return;
      }
      case 'schnapps-swap':
        if (view.schnapps_reward) void this.act({ ChooseSchnappsReward: { give_up: num('slot') } });
        return;
      case 'schnapps-sell':
        if (view.schnapps_reward) void this.act({ ChooseSchnappsReward: { give_up: null } });
        return;
      case 'paint': {
        const painter = view.painter;
        const option = painter?.options.find((p) => p.index === num('index'));
        if (!painter || !option) return;
        this.flights.noteStamp(el.querySelector('.card'));
        this.mods.applyMod(view, { ChoosePaint: option.index }, painter.card.id, option.card_mod.id, false, { kind: 'painter' });
        return;
      }
      case 'leave-shop':
        void this.act('LeaveShop');
        return;
    }
  }

  // -------------------------------------------------------------------------
  // Rendering
  // -------------------------------------------------------------------------

  render(): void {
    const ui = this.ui;
    this.table.keepHandOrder();
    // A waiting explanation of a new special card, once the table is quiet and no other sheet is open.
    if (this.progress.specialQueue.length && !ui.overlay && !ui.busy && !ui.anim && ui.screen === 'run') {
      ui.overlay = { kind: 'special', ids: this.progress.specialQueue.splice(0) };
    }
    // The player's edition shows his character's back (DESIGN 9.3).
    // A back earned in the long night replaces it if chosen (ADR 0020).
    const chosenBack = ui.settings.back && ui.meta?.card_backs?.includes(ui.settings.back) ? ui.settings.back : null;
    setPlayerBack(chosenBack ?? (ui.screen === 'end' ? ui.ended?.view : ui.view)?.character.back);
    let parts: Part[] = [];
    switch (ui.screen) {
      case 'loading':
        parts = [renderLoading()];
        break;
      case 'error':
        parts = [renderError()];
        break;
      case 'start':
        parts = [renderStart(ui, BUILD_INFO)];
        break;
      case 'end':
        parts = [ui.ended ? renderEnd(ui.ended.won, ui.ended.view, ui) : ''];
        if (ui.ended) this.afterRender.push(() => void this.prepareRunCard().catch(() => undefined));
        break;
      case 'run': {
        const view = ui.view;
        if (!view) break;
        const inShop = view.phase === 'Shop' && !ui.anim;
        const inEncounter = view.phase === 'Encounter' && !ui.anim;
        const inRoute = view.phase === 'Route' && !ui.anim;
        const inReward = view.phase === 'BossReward' && !ui.anim;
        const inCardReward = view.phase === 'CardReward' && !ui.anim;
        if (!inReward) ui.rewardPick = null;
        if (inRoute && ui.routeLeg !== legKey(view)) {
          // A new leg: the default way, the map in front.
          ui.routeLeg = legKey(view);
          ui.routePick = defaultPick(view);
          ui.routeOverview = false;
        }
        const main: Part[] = inShop
          ? [renderShop(view)]
          : inEncounter
            ? [renderEncounter(view)]
            : inReward
              ? [renderBossReward(view, ui.rewardPick)]
              : inCardReward
                ? [renderCardReward(view)]
                : inRoute
                ? [renderRoute(view, ui.routePick ?? defaultPick(view), ui.routeOverview)]
                : renderTable(view, ui);
        const cls = inShop ? 'is-shop' : inEncounter || inReward || inCardReward ? 'is-encounter' : inRoute ? 'is-route' : 'is-table';
        // HUD and main area are parts of their own (./ui/patch.ts): what did not change stays in the DOM.
        const run = shell(`<div class="run ${cls} ${ui.busy ? 'is-busy' : ''} ${view.night ? 'is-night' : ''}">`, [renderHud(view), ...main], { sticky: true });
        // The Kartenmaler waits (Tresen or Werkstatt): his sheet stays until a Zinke is taken.
        if (view.painter && !ui.busy && !ui.anim && (!ui.overlay || ui.overlay.kind === 'zinken-card')) ui.overlay = { kind: 'painter' };
        if (!view.painter && ui.overlay?.kind === 'painter') ui.overlay = null;
        // Die lange Nacht (ADR 0020): a new night district shows its Verschärfung first.
        const night = view.night;
        if (inRoute && night?.announced && ui.nightSeen !== night.district && !ui.busy && !ui.anim && !ui.overlay) {
          ui.nightSeen = night.district;
          ui.overlay = { kind: 'night' };
        }
        // The Schnaps of a won pub with every slot full: its sheet stays until it is taken or sold.
        if (view.schnapps_reward && !ui.busy && !ui.anim && !ui.overlay) ui.overlay = { kind: 'schnapps-reward' };
        if (!view.schnapps_reward && ui.overlay?.kind === 'schnapps-reward') ui.overlay = null;
        const kontra = view.game?.awaiting_kontra_answer && !ui.busy && !ui.overlay;
        // "Untot": the card waits in the trick for three cards of the hand.
        const rise = view.game?.rise_choice;
        parts = [run, kontra ? renderKontra(view) : '', rise && !ui.busy && !ui.overlay && !kontra ? renderRiseChoice(view, ui.risePick) : ''];
        if (!rise && ui.risePick.length) ui.risePick = [];
        break;
      }
    }
    // Keep the typed seed and the picker row across re-renders of the start screen.
    const seed = this.root.querySelector<HTMLInputElement>('#seed')?.value;
    const charScroll = this.root.querySelector<HTMLElement>('[data-testid="characters"]')?.scrollLeft;
    // The Tresen stays where it was scrolled to after buying or selling.
    const shopScroll = this.screenEl.querySelector<HTMLElement>('.run > .shop')?.scrollTop ?? 0;
    // A sheet on the screen that is drawn again (the Untot pick) does not enter again.
    const riseShown = this.screenEl.querySelector('.sheet-rise') !== null;
    // Cards flying onto the felt keep flying through the re-render: the
    // patcher names the nodes it replaces (noteFlights in the constructor).
    this.snapshots = [];
    this.screen.patch(parts);
    const flights = this.snapshots;
    this.snapshots = [];
    if (riseShown) {
      const still = this.screenEl.querySelector('.sheet-rise')?.closest('.overlay');
      if (still && !still.classList.contains('is-still')) still.classList.add('is-still');
    }
    if (flights.length) resumeFlights(this.screenEl, FLYING, flights);
    if (seed) {
      const input = this.root.querySelector<HTMLInputElement>('#seed');
      if (input) input.value = seed;
    }
    const shop = this.screenEl.querySelector<HTMLElement>('.run > .shop');
    if (shop && shopScroll) shop.scrollTop = shopScroll;
    if (ui.screen === 'start') settleCharacterPicker(this.screenEl, charScroll);
    this.layer.render();
    // The score board and the boss take the screen: no info card, no trigger lines over them.
    const big = ui.overlay?.kind === 'score' || ui.overlay?.kind === 'boss';
    if (big) {
      this.info.close();
      this.triggers.clear();
    } else {
      this.info.refresh();
    }
    this.guidance.updateHint();
    this.guidance.updateCoach();
    music.setScene(this.musicScene());
    watchShop(this.screenEl);
    const effects = this.afterRender.splice(0);
    // An effect is decoration: one that fails must not take the render (and the playback) down.
    for (const fx of effects) {
      try {
        fx();
      } catch (err) {
        console.error('effect after render failed', err);
      }
    }
    if (ui.screen === 'run' && ui.view) this.hudFx.sync(ui.view);
    else if (ui.screen === 'start') this.hudFx.reset();
    if (ui.screen === 'run' && !ui.busy && !ui.overlay) this.drink.refresh();
    else this.drink.cancel();
    this.flights.flyPressed();
    if (ui.screen !== 'run') this.entrancePending = false;
    if (this.entrancePending && !ui.overlay && !ui.anim) {
      this.entrancePending = false;
      if (ui.view?.series.is_boss && ui.view.phase === 'Game') bossEntrance(this.screenEl);
    }
  }

}

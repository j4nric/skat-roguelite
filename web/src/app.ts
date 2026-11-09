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
import { music } from './music';
import { CoreMeta, CoreRun } from './core';
import { de, consumableMod, contentName } from './i18n';
import { loadSettings, saveSettings, TEMPO_FACTOR, type Tempo } from './settings';
import { passkeysSupported } from './account';
import { isOnline, watchNetwork } from './offline';
import { clearShareHash, dailySeed, isSeedText } from './seed';
import { maxStake } from './stake';
import {
  clearRun,
  loadCharacter,
  loadHistory,
  saveCharacter,
} from './storage';
import { type NullRisk, type RiskLevel, type RunAction, type Suit } from './types';
import { reducedMotion } from './ui/dom';
import { markeEffect } from './ui/encounter';
import { legKey, watchMapLabels } from './ui/route';
import { markTutorial } from './tutorial';
import { infoModel } from './ui/info';
import { InfoCard } from './ui/infocard';
import { watchPresses } from './ui/press';
import { TriggerFeed } from './ui/triggers';
import { finishAll, motionOn,  setTempo, springBack } from './ui/fx';
import { HudFx, spring, watchSprings } from './ui/juice';
import { sceneRunning } from './ui/scenes';
import { watchDrags } from './ui/drag';
import { watchKeys } from './ui/keys';
import { buzz, setVibration } from './haptics';
import { Drink } from './ui/drink';
import { updateRiseChoice,  scrollCharacterPicker, sharedStake } from './ui/screens';
import type { UiState } from './ui/state';
import { discardNeed } from './ui/table';
import { isCharacter, type ToastPlace } from './app/consts';
import { AccountFlow } from './app/account';
import type { AppCtx } from './app/context';
import { Guidance } from './app/guidance';
import { Layer } from './app/layer';
import { Lifecycle } from './app/lifecycle';
import { Mods } from './app/mods';
import { EventPlayback } from './app/playback/events';
import { GamePlayback } from './app/playback/game';
import { Steps } from './app/playback/steps';
import { DrinkFx } from './app/playback/drinkfx';
import { Flights } from './app/playback/flights';
import { Fx } from './app/playback/fx';
import { Progress } from './app/progress';
import { Renderer } from './app/render';
import { SaveCodeFlow } from './app/savecode';
import { Share } from './app/share';
import { TableInput } from './app/table-input';

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
  /** Account and Groschen in the head: how their changes get there. */
  readonly hudFx: HudFx;
  readonly layerEl: HTMLElement;
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
  /** Actions sent to the core and played back (./app/playback/steps.ts). */
  private readonly steps: Steps;
  /** "Teilen": the run's picture and the seed's link (./app/share.ts). */
  private readonly share: Share;
  /** Start, new run, resume, the Einführung (./app/lifecycle.ts). */
  private readonly lifecycle: Lifecycle;
  /** Draws the screen part by part (./app/render.ts). */
  private readonly renderer: Renderer;

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
    const game = new GamePlayback(this, this.fx, this.flights, this.layer);
    const events = new EventPlayback(this, this.fx, this.flights, this.drinkFx, game, this.layer, this.guidance);
    this.steps = new Steps(this, this.fx, this.flights, events, this.layer, this.progress, this.account.sync, () => this.lifecycle.afterPlayback());
    this.share = new Share(this, this.layer);
    this.lifecycle = new Lifecycle(this, this.account, this.progress, this.fx, this.flights, this.guidance, this.steps, this.layer);
    this.renderer = new Renderer(this, this.layer, this.guidance, this.share, this.table, this.flights, this.progress);
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

  /** Load the core, then the stored states, and show the start screen (./app/lifecycle.ts). */
  start(): Promise<void> {
    return this.lifecycle.start();
  }

  // -------------------------------------------------------------------------
  // Account and sync (ADR 0021). Transport only: the save is the save code.
  // -------------------------------------------------------------------------

  private applySettings(): void {
    setTempo(TEMPO_FACTOR[this.ui.settings.tempo]);
    sound.setEnabled(this.ui.settings.sound);
    music.setEnabled(this.ui.settings.music);
    // Pure presentation: style.css tints the cards' paper under this flag.
    document.documentElement.toggleAttribute('data-suit-tint', this.ui.settings.suitTint);
    setVibration(this.ui.settings.vibrate);
  }

  // -------------------------------------------------------------------------
  // Screens
  // -------------------------------------------------------------------------

  // -------------------------------------------------------------------------
  // Save code: export and import of everything stored (./savecode.ts)
  // -------------------------------------------------------------------------

  // -------------------------------------------------------------------------
  // Actions
  // -------------------------------------------------------------------------

  /** Draw the screen from the state (./app/render.ts). */
  render(): void {
    this.renderer.render();
  }

  /** Send an action to the core and play back what happened (./app/playback/steps.ts). */
  act(action: RunAction, fast = false): Promise<void> {
    return this.steps.act(action, fast);
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
        this.lifecycle.endTutorial();
        this.lifecycle.newRun();
        return;
      // Die Einführung (DESIGN 16.1).
      case 'tutorial-start':
        this.lifecycle.startTutorial();
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
        this.lifecycle.leaveTutorial('start');
        return;
      case 'tutorial-finish':
        this.lifecycle.leaveTutorial(el.dataset.go === 'run' ? 'run' : 'start');
        return;
      // Die lange Nacht (ADR 0020): on from the end screen of a won run.
      case 'enter-night':
        this.lifecycle.enterNight();
        return;
      case 'resume':
        this.lifecycle.resume();
        return;
      case 'play-shared': {
        const shared = this.ui.shared;
        if (shared) this.lifecycle.requestStart({ seed: shared.seed, character: shared.locked ? 'Declarer' : shared.character, stake: sharedStake(this.ui), daily: false });
        return;
      }
      case 'dismiss-shared':
        this.ui.shared = null;
        clearShareHash();
        this.render();
        return;
      case 'play-daily':
        this.lifecycle.requestStart({ seed: dailySeed(), character: this.ui.pickCharacter, stake: this.lifecycle.pickedStake(), daily: true });
        return;
      case 'replay': {
        const c = el.dataset.character ?? null;
        const seed = el.dataset.seed ?? '';
        const entry = this.ui.meta?.characters.find((x) => x.character === c);
        // Replayed at its stake, or the highest unlocked if that is lower.
        const stake = isCharacter(c) ? Math.min(num('stake') < 0 ? 0 : num('stake'), maxStake(this.ui.meta, c)) : 0;
        if (isCharacter(c) && entry?.unlocked && isSeedText(seed)) this.lifecycle.requestStart({ seed, character: c, stake, daily: false });
        return;
      }
      case 'confirm-new':
        if (overlay?.kind === 'confirm-new') {
          const start = overlay.start;
          this.ui.overlay = null;
          clearRun();
          this.lifecycle.startRun(start);
        }
        return;
      case 'copy-link':
        this.share.copyLink(el.dataset.url ?? '');
        return;
      case 'share-run':
        void this.share.shareRun(el);
        return;
      case 'share-native':
        this.share.shareNative(el.dataset.url ?? '', el.dataset.text ?? '');
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
        this.lifecycle.endTutorial();
        this.lifecycle.toStart();
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
        void this.steps.settle();
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
        void this.steps.travel(num('option'));
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

}

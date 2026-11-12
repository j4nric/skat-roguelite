// The app: holds the core run and the UI state and wires the modules in
// ./app/ together. No game rules live here or there: what is legal, what
// a game is worth and who takes a trick all come from the core.
//
// The modules (each gets this App as its context, ./app/context.ts, and
// the modules below it; the few calls every module needs go through the
// context: render, act, persist, showToast):
//
// - ./app/lifecycle.ts: boot, start screen, new and resumed runs, the long
//   night, the Einführung (DESIGN 16.1)
// - ./app/playback/: an action sent to the core and its events played back
//   as animations with sound (steps, events, game), with the pauses,
//   toasts, quips and trigger lines (fx), the cards in the air (flights)
//   and a Schnaps's pictures (drinkfx)
// - ./app/render.ts and ./app/layer.ts: the screen, drawn part by part,
//   and the dialog layer, drawn only when the dialog changes
// - ./app/guidance.ts: the Wirt's hints and coaching
// - ./app/table-input.ts, ./app/tresen.ts, ./app/mods.ts, ./app/menu.ts:
//   taps, drags and drops at the table, between the pubs and in the menu
// - ./app/progress.ts, ./app/account.ts, ./app/savecode.ts, ./app/share.ts:
//   what is stored, the account and sync (ADR 0021), the save code, "Teilen"
//
// Taps on `data-action` elements go through one table of handlers
// (./app/actions.ts) that the modules fill.

import { sound, unlockAudio } from './audio';
import type { CoreMeta, CoreRun } from './core';
import { de } from './i18n';
import { noteRender, watchApp } from './diag';
import { isOnline, watchNetwork } from './offline';
import { passkeysSupported } from './account';
import { loadSettings } from './settings';
import { loadCharacter, loadHistory } from './storage';
import type { RunAction } from './types';
import { mountLive, trapFocus } from './ui/a11y';
import { watchDrags } from './ui/drag';
import { Drink } from './ui/drink';
import { finishAll, springBack } from './ui/fx';
import { infoModel } from './ui/info';
import { InfoCard } from './ui/infocard';
import { HudFx, watchSprings } from './ui/juice';
import { watchKeys } from './ui/keys';
import { watchPresses } from './ui/press';
import { watchMapLabels } from './ui/route';
import { sceneRunning } from './ui/scenes';
import type { UiState } from './ui/state';
import { TriggerFeed } from './ui/triggers';
import { AccountFlow } from './app/account';
import { ActionTable } from './app/actions';
import { isCharacter, type ToastPlace } from './app/consts';
import type { AppCtx } from './app/context';
import { Guidance } from './app/guidance';
import { Layer } from './app/layer';
import { Lifecycle } from './app/lifecycle';
import { MoveFlow } from './app/move';
import { Menu } from './app/menu';
import { Mods } from './app/mods';
import { DrinkFx } from './app/playback/drinkfx';
import { EventPlayback } from './app/playback/events';
import { Flights } from './app/playback/flights';
import { Fx } from './app/playback/fx';
import { GamePlayback } from './app/playback/game';
import { Steps } from './app/playback/steps';
import { Progress } from './app/progress';
import { ReportFlow } from './app/report';
import { Renderer } from './app/render';
import { SaveCodeFlow } from './app/savecode';
import { Share } from './app/share';
import { TableInput } from './app/table-input';
import { Tresen } from './app/tresen';

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
    week: null,
    weeklyBoard: null,
  };
  fastForward = false;
  readonly screenEl: HTMLElement;
  readonly layerEl: HTMLElement;
  /** Account and Groschen in the head: how their changes get there. */
  readonly hudFx: HudFx;
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

  private readonly layer = new Layer(this);
  private readonly guidance: Guidance;
  private readonly fx: Fx;
  private readonly account: AccountFlow;
  private readonly progress: Progress;
  private readonly table: TableInput;
  private readonly steps: Steps;
  private readonly lifecycle: Lifecycle;
  private readonly renderer: Renderer;
  private readonly menu: Menu;
  private readonly actions: ActionTable;
  private readonly report: ReportFlow;

  constructor(readonly root: HTMLElement) {
    root.innerHTML = `<div class="screen"></div><div class="layer"></div><div class="hint-layer" aria-live="polite"></div><div class="coach-layer" aria-live="polite"></div><div class="info-layer"></div><div class="trigger-layer" role="status" aria-live="polite"></div><div class="toast-layer" role="status" aria-live="polite"></div><div class="drink-layer"></div>`;
    this.screenEl = root.querySelector<HTMLElement>('.screen') ?? root;
    this.layerEl = root.querySelector<HTMLElement>('.layer') ?? root;
    // The modules, each after those it uses.
    this.guidance = new Guidance(this);
    this.fx = new Fx(this, this.guidance);
    const flights = new Flights(this, this.fx);
    const drinkFx = new DrinkFx(this, this.fx);
    this.account = new AccountFlow(this, this.layer);
    this.progress = new Progress(this, this.account);
    const saveCode = new SaveCodeFlow(this);
    const mods = new Mods(this);
    this.table = new TableInput(this, this.fx, flights, drinkFx, mods);
    const game = new GamePlayback(this, this.fx, flights, this.layer);
    const events = new EventPlayback(this, this.fx, flights, drinkFx, game, this.layer, this.guidance);
    this.steps = new Steps(this, this.fx, flights, events, this.layer, this.progress, this.account.sync, () => this.lifecycle.afterPlayback());
    const share = new Share(this, this.layer);
    const move = new MoveFlow(this, this.account, saveCode);
    this.lifecycle = new Lifecycle(this, this.account, this.progress, this.fx, flights, this.guidance, this.steps, this.layer, move);
    this.renderer = new Renderer(this, this.layer, this.guidance, share, this.table, flights, this.progress);
    this.menu = new Menu(this, this.layer);
    const tresen = new Tresen(this, mods, flights);
    this.report = new ReportFlow(this, () => this.diagState());
    // Every `data-action` of the templates has its handler in one of them (scripts/check-actions.mjs).
    this.actions = new ActionTable(this, this.fx);
    this.actions.add(this.lifecycle.actions());
    this.actions.add(this.menu.actions());
    this.actions.add(this.layer.actions());
    this.actions.add(this.guidance.actions());
    this.actions.add(share.actions());
    this.actions.add(this.account.actions());
    this.actions.add(saveCode.actions());
    this.actions.add(move.actions());
    this.actions.add(this.report.actions());
    this.actions.add(this.table.actions(), this.table.runActions());
    this.actions.add({}, this.steps.runActions());
    this.actions.add(tresen.actions(), tresen.runActions());
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
    const selectable = 'input, textarea, [contenteditable="true"], .seed-num, .report-id';
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
    watchNetwork((online) => {
      this.account.networkChanged(online);
      // Bug reports made offline go out now (ADR 0027).
      if (online) void this.report.flush();
    });
    // The diagnosis buffer (./diag.ts): taps, dead taps, where the app stands.
    watchApp(root, [this.screenEl, this.layerEl], { state: () => this.diagState() });
    this.menu.applySettings();
    // Screen readers: the polite live region; keyboard: Tab stays inside an open dialog (./ui/a11y.ts).
    mountLive(root);
    trapFocus();
    root.addEventListener('click', (e) => this.actions.onClick(e));
    // The volume sliders of the menu (./app/menu.ts): live while moved, stored when let go.
    root.addEventListener('input', (e) => this.menu.onVolume(e, false));
    root.addEventListener('change', (e) => this.menu.onVolume(e, true));
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
  async start(): Promise<void> {
    await this.lifecycle.start();
    // Bug reports that waited offline since the last visit (ADR 0027).
    void this.report.flush();
  }

  /** Draw the screen from the state (./app/render.ts). */
  render(): void {
    this.renderer.render();
    noteRender();
  }

  /**
   * Where the app stands, in one line, for the diagnosis buffer: screen,
   * phase of the run and the table, the dialog (whether a playback still
   * waits on the score board or the boss sheet), busy and animating.
   */
  private diagState(): string {
    const ui = this.ui;
    const view = ui.screen === 'end' ? ui.ended?.view : ui.view;
    const o = ui.overlay;
    const dialog = !o ? '-' : o.kind === 'score' || o.kind === 'boss' ? `${o.kind}(${this.layer.awaited === o ? 'awaited' : 'stale'})` : o.kind;
    const parts = [ui.screen, view?.phase ?? '-', view?.game?.phase ?? '-', dialog];
    if (ui.busy) parts.push('busy');
    if (ui.anim) parts.push('anim');
    if (this.fastForward) parts.push('fast');
    return parts.join(' ');
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
}

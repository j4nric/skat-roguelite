// The app controller: holds the core run and the UI state, turns taps into
// RunActions, plays the core's events back as animations with sound, and
// saves the core state after every action. No game rules live here: what
// is legal, what a game is worth and who takes a trick all come from the
// core.
//
// The DOM has three layers: the screen (re-rendered on every change), the
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
// main thread is busy, and the run is saved after every card.
//
// Above the dialogs sits the Wirt's speech bubble (./hints.ts, "Geführter
// erster Abend"): after every render the app asks which hint fits what is
// on screen, shows it once and remembers it when it is dismissed or the
// situation has passed.

import { sound, unlockAudio } from './audio';
import { music, type MusicId } from './music';
import { CoreMeta, CoreRun, loadCatalog, loadCore } from './core';
import { de, cardName, consumableMod, contentName, face, modKind, relicCounters, setCatalog, t } from './i18n';
import { loadSettings, saveSettings, TEMPO_FACTOR, type Tempo } from './settings';
import { clearSeenHints, hintApplies, loadSeenHints, pickHint, saveSeenHints, type HintAnchor, type HintId } from './hints';
import { decodeSave, encodeSave } from './savecode';
import { clearShareHash, dailySeed, isSeedText, parseShareHash } from './seed';
import { maxStake, pickedStake } from './stake';
import {
  addToHistory,
  backupMeta,
  clearRun,
  loadCharacter,
  loadHistory,
  loadMeta,
  loadRun,
  loadRunInfo,
  readAllKeys,
  replaceAllKeys,
  runAndMetaOf,
  saveCharacter,
  saveLength,
  saveMeta,
  saveRun,
  saveRunInfo,
} from './storage';
import { CHARACTERS, type CardKind, type CardView, type Character, type DealtCard, type Declaration, type DeckCard, type EditionEntry, type NullRisk, type RiskLevel, type RunAction, type RunEvent, type RunLength, type RunView, type ScoreBreakdown, type ScoreLine, type Seat, type Source, type Step, type Suit } from './types';
import { esc, reducedMotion } from './ui/dom';
import { markeEffect, renderEncounter } from './ui/encounter';
import { renderBossReward } from './ui/bossreward';
import { renderCardReward } from './ui/cardreward';
import { defaultPick, legKey, playTravel, renderRoute } from './ui/route';
import { seatLabel as seatLabelOf, seatPerson } from './ui/guests';
import { hintText, renderHint } from './ui/hint';
import { infoModel } from './ui/info';
import { InfoCard } from './ui/infocard';
import { watchPresses } from './ui/press';
import { targetsOf, TriggerFeed, type Trigger } from './ui/triggers';
import { crumbleInto, deal, declareSign, finishAll, finishOn, flip, flyIn, frown, glide, glideAll, motionOn, nod, noteFlights, play, resumeFlights, setTempo, snipAcross, type Spot, spotOf, spotOfRect, spotsOf, springBack, sweepAway, throwCard } from './ui/fx';
import { fireworks, HudFx, lightsOut, lightsUp, spring, watchSprings } from './ui/juice';
import { bossEntrance, doppelReach, mirrorShimmer, relicToStrip, sceneRunning, sceneWord, schnapsToSlot, swapFlight, zinkeStamp } from './ui/scenes';
import { backDesign, setPlayerBack } from './ui/backs';
import { cardBack, cardSvg, editionLook, lookSvg, modValueText, showWildSuit } from './ui/cards';
import { SUITS } from './ui/art';
import { watchDrags, type Fling } from './ui/drag';
import { Drink, type Pour } from './ui/drink';
import { reshuffle } from './ui/scatter';
import { coinIcon, renderHud } from './ui/hud';
import { changeText, renderScore, ScoreBoard, sourceName } from './ui/score';
import { renderEnd, renderError, renderKontra, renderRiseChoice, updateRiseChoice, renderLoading, renderOverlay, renderStart, scrollCharacterPicker, settleCharacterPicker, sharedStake } from './ui/screens';
import { renderShop, watchShop } from './ui/shop';
import type { ModQuestion, Overlay, PendingStart, ScoreOverlay, UiState } from './ui/state';
import { declarationSign, discardNeed, renderTable } from './ui/table';

/** The stable code of a core error (JSON `{code, message}`), if any. */
function errorCode(err: unknown): string | null {
  const raw = err instanceof Error ? err.message : String(err);
  try {
    const parsed = JSON.parse(raw) as { code?: string };
    return parsed.code ?? null;
  } catch {
    return null;
  }
}

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

/** Turn a core error into a German text. */
function errorText(err: unknown): string {
  const code = errorCode(err);
  const fallback = de.app.errors.default ?? '';
  return code ? (de.app.errors[code] ?? fallback) : fallback;
}

/** One line about a saved run: who plays, where, the account. */
function runLine(view: RunView): string {
  const character = contentName('characters', view.character.character);
  const line = view.phase === 'Encounter' || view.phase === 'Route' || view.phase === 'Shop' || view.phase === 'BossReward' || view.phase === 'CardReward'
    ? t(de.start.resumeEncounter, { character, section: view.series.section + 1 })
    : t(de.start.resumeInfo, {
        character,
        section: view.series.section + 1,
        series: view.series.index + 1,
        score: view.series.score,
        target: view.series.target,
      });
  const short = view.length === 'Short' ? `${line} · ${de.start.lengths.Short.name}` : line;
  return view.stake > 0 ? short + t(de.share.lineStake, { stake: view.stake }) : short;
}

function isCharacter(value: string | null): value is Character {
  return CHARACTERS.includes(value as Character);
}

/** How long a Stammgast's bubble stays at tempo "Normal" (reading time, not motion). */
const QUIP_MS = 2800;
/** At least this many tricks between two quips: about one every second game. */
const QUIP_EVERY_TRICKS = 14;

/** Pauses in milliseconds at tempo "Normal"; reduced motion shortens them. */
const DELAY = {
  ownCard: 340,
  aiCard: 560,
  trickShow: 650,
  trickFly: 480,
  toast: 1900,
  notice: 700,
  /** The skat turning over on the table. */
  flip: 420,
  /** Relics and modifications that fired, before the score board covers them. */
  trigger: 650,
};

/** A trick with this many Augen or more the player took annoys the regulars (the same bar as their quips). */
const BIG_TRICK = 10;

/** A flick upwards this fast (px per ms) throws a card even if let go over the hand. */
const FLICK_UP = 0.9;

/**
 * Cards whose flights survive a re-render: the open trick, the hand, the
 * cards to press on the table and the pressed stack.
 */
const FLYING = '.trick:not(.is-last) .trick-card, .hand-card, .skat-card, .pressed-card[data-fly]';

/** The cards of the hand and those lying on the table to be pressed. */
const PRESS_CARDS = '.hand-card, .skat-card';

/** A card picked up or laid down: flight (ms at tempo Normal), lift and curve (px). */
const PICK = { ms: 440, lift: 28, arc: 16 };
/** The other hand cards making room (FLIP). */
const MAKE_ROOM_MS = 260;

/** Cards on their way to the pressed skat: where they were seen, and their face. */
interface PressFlight {
  cards: { spot: Spot; face: string }[];
  /** Only the stack in the felt's corner takes them (after the Ansage); else also the Ansage's row. */
  stackOnly: boolean;
  /** Dropped when no render showed a target by then (performance.now()). */
  until: number;
}

/** How long the cards slide away before a settled game is scored. */
const SETTLE_MS = 260;
/** The trip along a way on the map at tempo "Normal". */
const TRAVEL_MS = 1600;

/** Wait until the next frame has been painted (or 120 ms in a hidden tab). */
function nextPaint(): Promise<void> {
  return new Promise((resolve) => {
    let done = false;
    const finish = (): void => {
      if (done) return;
      done = true;
      resolve();
    };
    // rAF runs just before the paint; the timeout after it runs once the
    // frame is on screen.
    requestAnimationFrame(() => window.setTimeout(finish, 0));
    window.setTimeout(finish, 120);
  });
}

/** Toasts waiting at most; older ones are dropped. */
const TOAST_QUEUE = 3;

/** Where a toast shows: the middle of the screen, or above the Stammgäste. */
type ToastPlace = 'middle' | 'top';
/** "Untot": the event of a card that rose from the trick. */
type DoppelSwap = Extract<Extract<RunEvent, { Game: unknown }>['Game'], { DoppelgangerSwapped: unknown }>['DoppelgangerSwapped'];
type RoseEvent = Extract<Extract<RunEvent, { Game: unknown }>['Game'], { CardRose: unknown }>['CardRose'];

const BUILD_INFO = typeof __BUILD_INFO__ === 'string' ? __BUILD_INFO__ : '';

export class App {
  private core: CoreRun | null = null;
  private meta: CoreMeta | null = null;
  private ui: UiState = {
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
    pickLength: 'Full',
    pickStake: null,
    overlay: null,
    message: null,
    saveInfo: null,
    meta: null,
    settings: loadSettings(),
    ended: null,
  };
  private fastForward = false;
  private wakers: (() => void)[] = [];
  private toastTimer: number | undefined;
  private toasts: { text: string; place: ToastPlace }[] = [];
  private readonly screenEl: HTMLElement;
  /** Account and Groschen in the head: how their changes get there. */
  private readonly hudFx: HudFx;
  private readonly layerEl: HTMLElement;
  private readonly toastEl: HTMLElement;
  /** "Untot": cards whose rising was already shown with their trick in this game. */
  private readonly riseShown = new Set<number>();
  /** "Doppelgänger": the stolen card the swap scene already laid into the trick (its CardPlayed follows). */
  private doppelLanded: number | null = null;
  private readonly hintEl: HTMLElement;
  /** The info card (long-press, hover, "i"): one for everything. */
  private readonly info: InfoCard;
  /** Lines and wiggles when a relic or modification fires. */
  private readonly triggers: TriggerFeed;
  /** Hints of the Wirt already seen (stored). */
  private hintSeen = loadSeenHints();
  /** The hint on screen and where it sits. */
  private hint: { id: HintId; anchor: HintAnchor; html: string } | null = null;
  /** Hints shown but not dismissed: seen once their situation has passed. */
  private hintsShown = new Set<HintId>();
  /** The player passed a game in this visit (for the hint about passing). */
  private passedOnce = false;
  /** Series ("section-index") whose Stammgast already said hello. */
  private quipSeries = '';
  /** Tricks since the last quip: quips stay rare. */
  private tricksSinceQuip = 0;
  private quipTimer: number | undefined;
  private renderedOverlay: Overlay | null = null;
  private overlayDirty = false;
  private scoreBoard: ScoreBoard | null = null;
  /**
   * The score board or boss sheet a playback is waiting on right now. A
   * board on the layer that nothing waits for any more (its playback
   * broke off) is stale: its button closes it instead of resolving nothing.
   */
  private awaited: Overlay | null = null;
  /** Effects to run once the next render is in the DOM. */
  private afterRender: (() => void)[] = [];
  /** Special cards received for the first time in this save, waiting for their explanation sheet. */
  private specialQueue: string[] = [];
  /** Where the card the player just tapped was. */
  private playedFrom: DOMRect | null = null;
  /** The face-down skat and the hand, seen when the player picked it up. */
  private pickupFrom: { pile: Spot[]; hand: Map<string, Spot> } | null = null;
  /** The pressed cards still to fly onto their place (./ui/table.ts pressedStack). */
  private pressFlight: PressFlight | null = null;
  /** Drinking a Schnaps from the Leiste (drag and drop or tap, ./ui/drink.ts). */
  private readonly drink: Drink;
  /** The Schnaps just poured, for its effects while its events play: the table's cards washed away. */
  private pouring: { id: string; kind: Pour['kind']; wash: HTMLElement[] } | null = null;
  /** How the player's card was thrown (drag and flick), for its flight. */
  private thrown: Fling | null = null;
  /** A boss pub just opened: its entrance plays once the table is free of dialogs. */
  private entrancePending = false;
  /** What was bought at the Tresen, seen as it was tapped: its picture flies into the Leiste. */
  private bought: { html: string; rect: DOMRect } | null = null;
  /** Where the card was that a Zinke goes onto, seen as it was tapped (for its stamp). */
  private stampAt: { rect: DOMRect; at: number } | null = null;

  constructor(private readonly root: HTMLElement) {
    root.innerHTML = `<div class="screen"></div><div class="layer"></div><div class="hint-layer" aria-live="polite"></div><div class="info-layer"></div><div class="trigger-layer" role="status" aria-live="polite"></div><div class="toast-layer" role="status" aria-live="polite"></div><div class="drink-layer"></div>`;
    this.screenEl = root.querySelector<HTMLElement>('.screen') ?? root;
    this.layerEl = root.querySelector<HTMLElement>('.layer') ?? root;
    this.hudFx = new HudFx(this.screenEl);
    watchSprings(root);
    this.toastEl = root.querySelector<HTMLElement>('.toast-layer') ?? root;
    this.hintEl = root.querySelector<HTMLElement>('.hint-layer') ?? root;
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
      drop: (el, zone, over, at, fling) => this.onDrop(el, zone, over, at, fling),
      blocked: () => this.ui.busy || this.ui.screen !== 'run' || Boolean(this.ui.overlay) || this.info.state?.mode === 'sheet' || this.drink.active,
    });
    // A Schnaps from the Leiste onto its target (or lifted by a tap).
    this.drink = new Drink(this.screenEl, root.querySelector<HTMLElement>('.drink-layer') ?? root, {
      item: (slot) => this.ui.view?.consumables.find((c) => c.slot === slot),
      blocked: () => this.ui.busy || this.ui.screen !== 'run' || Boolean(this.ui.overlay) || this.info.state?.mode === 'sheet',
      pour: (p) => this.pourSchnaps(p),
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
      this.positionHint();
      window.clearTimeout(settle);
      settle = window.setTimeout(() => this.positionHint(), 100);
    });
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
  }

  async start(): Promise<void> {
    this.render();
    try {
      await loadCore();
      setCatalog(loadCatalog());
    } catch (err) {
      console.warn('core failed to load', err);
      this.ui.screen = 'error';
      this.render();
      return;
    }
    this.loadMetaState();
    this.readShareHash();
    window.addEventListener('hashchange', () => {
      this.readShareHash();
      if (this.ui.screen === 'start') this.render();
    });
    this.toStart();
  }

  /** Offer the seed of a shared link (`#seed-…`), if the address has one. */
  private readShareHash(): void {
    const shared = parseShareHash(window.location.hash);
    if (!shared) return;
    const entry = this.ui.meta?.characters.find((c) => c.character === shared.character);
    const locked = shared.character !== 'Declarer' && entry?.unlocked !== true;
    this.ui.shared = { ...shared, locked };
  }

  /** Load the meta state, or start a new one (keeping an unreadable save aside). */
  private loadMetaState(): void {
    const saved = loadMeta();
    if (saved) {
      try {
        this.meta = CoreMeta.load(saved);
      } catch (err) {
        console.warn('meta state discarded', err);
        backupMeta(saved);
        this.ui.message = de.start.metaFailed;
      }
    }
    try {
      this.meta ??= CoreMeta.fresh();
      if (!saved) saveMeta(this.meta.save());
      this.ui.meta = this.meta.view();
    } catch (err) {
      console.warn('meta state unavailable', err);
      this.meta = null;
      this.ui.meta = null;
    }
    const picked = this.ui.meta?.characters.find((c) => c.character === this.ui.pickCharacter);
    if (!picked?.unlocked) this.ui.pickCharacter = 'Declarer';
  }

  /** Feed the events of a step to the meta state; returns the unlocks just earned. */
  private observe(events: RunEvent[] | string): string[] {
    if (!this.meta) return [];
    try {
      const step = this.meta.observe(events);
      this.ui.meta = step.view;
      saveMeta(this.meta.save());
      // The first Untot or Wandelkarte of this save: explained once (DESIGN 7.2), shown by render().
      for (const id of step.first_specials ?? []) if (!this.specialQueue.includes(id)) this.specialQueue.push(id);
      return step.earned;
    } catch (err) {
      console.warn('meta state did not take the events', err);
      return [];
    }
  }

  /** Announce unlocks just earned. */
  private announceUnlocks(ids: string[]): void {
    if (!ids.length || !this.ui.meta) return;
    this.ui.overlay = { kind: 'unlock', ids };
    sound.play('win', { delay: 150 });
  }

  private applySettings(): void {
    setTempo(TEMPO_FACTOR[this.ui.settings.tempo]);
    sound.setEnabled(this.ui.settings.sound);
    music.setEnabled(this.ui.settings.music);
    // Pure presentation: style.css tints the cards' paper under this flag.
    document.documentElement.toggleAttribute('data-suit-tint', this.ui.settings.suitTint);
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
    this.ui.screen = 'start';
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
    this.requestStart({ seed, character: this.ui.pickCharacter, length: this.ui.pickLength, stake: this.pickedStake(), daily: false });
  }

  /** The stake picked on the start screen, at most the highest unlocked. */
  private pickedStake(): number {
    return pickedStake(this.ui.meta, this.ui.pickCharacter, this.ui.pickLength, this.ui.pickStake);
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

  private startRun({ seed, character, length: _asked, stake, daily }: PendingStart): void {
    // Since ADR 0016 every run is the whole evening, also from old `-kurz` links.
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
    const earned = this.observe(startEvents);
    this.enterRun(view, true);
    if (earned.length) {
      this.announceUnlocks(earned);
      this.render();
    }
  }

  /** Remember a finished run in the list of this browser. */
  private recordRun(view: RunView): void {
    const info = loadRunInfo();
    this.ui.history = addToHistory({
      character: view.character.character,
      seed: view.seed,
      ...(view.length === 'Short' ? { short: true } : {}),
      ...(view.stake > 0 ? { stake: view.stake } : {}),
      section: view.series.section + 1,
      series: view.series.index + 1,
      won: view.phase === 'Won',
      bestGame: view.stats.best_game,
      date: new Date().toISOString(),
      daily: info?.seed === view.seed && info.daily,
    });
  }

  /** Copy the link of a seed; on failure show it to be copied by hand. */
  private copyLink(url: string): void {
    const done = (ok: boolean): void => {
      this.ui.copy = { url, ok };
      this.overlayDirty = true;
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

  /** Menu "Spielstand sichern": the code of everything stored, to copy or share. */
  private async openSaveExport(): Promise<void> {
    const from = this.ui.overlay;
    let code: string;
    try {
      code = await encodeSave(readAllKeys());
    } catch (err) {
      console.warn('save code not made', err);
      this.showToast(de.save.exportFailed);
      return;
    }
    // The player closed the menu meanwhile.
    if (this.ui.overlay !== from) return;
    this.ui.overlay = { kind: 'save-export', code, copied: null };
    this.render();
  }

  /** Mark the whole save code, for copying by hand. */
  private selectSaveCode(): HTMLTextAreaElement | null {
    const area = this.root.querySelector<HTMLTextAreaElement>('[data-testid="save-code"]');
    if (!area) return null;
    area.focus();
    area.select();
    area.setSelectionRange(0, area.value.length);
    return area;
  }

  /** Copy the save code; without the Clipboard API mark it and try the old way. */
  private copySave(): void {
    const o = this.ui.overlay;
    if (o?.kind !== 'save-export') return;
    const done = (ok: boolean): void => {
      if (this.ui.overlay !== o) return;
      this.ui.overlay = { ...o, copied: ok };
      this.render();
      if (ok) sound.play('clink');
      else this.selectSaveCode();
    };
    const fallback = (): void => {
      let ok = false;
      try {
        ok = this.selectSaveCode() !== null && document.execCommand('copy');
      } catch {
        ok = false;
      }
      done(ok);
    };
    try {
      const clip = navigator.clipboard as Clipboard | undefined;
      if (!clip) {
        fallback();
        return;
      }
      clip.writeText(o.code).then(
        () => done(true),
        (err: unknown) => {
          console.warn('clipboard refused', err);
          fallback();
        },
      );
    } catch (err) {
      console.warn('clipboard unavailable', err);
      fallback();
    }
  }

  /** The share sheet of the system with the save code as text. */
  private shareSave(): void {
    const o = this.ui.overlay;
    if (o?.kind !== 'save-export') return;
    try {
      if (typeof navigator.share !== 'function') {
        this.copySave();
        return;
      }
      navigator.share({ title: de.save.shareTitle, text: o.code }).catch((err: unknown) => {
        if (!(err instanceof DOMException && err.name === 'AbortError')) this.copySave();
      });
    } catch {
      this.copySave();
    }
  }

  /** "Prüfen": read the pasted code and let the core load what it carries. */
  private async checkImport(): Promise<void> {
    const o = this.ui.overlay;
    if (o?.kind !== 'save-import' || o.checking) return;
    const text = this.root.querySelector<HTMLTextAreaElement>('#save-input')?.value ?? '';
    const pending: Overlay = { kind: 'save-import', text, error: null, checking: true };
    this.ui.overlay = pending;
    this.render();
    const result = await this.readSaveCode(text);
    if (this.ui.overlay !== pending) return;
    this.ui.overlay =
      'error' in result
        ? { kind: 'save-import', text, error: result.error, checking: false }
        : { kind: 'save-confirm', keys: result.keys, lines: result.lines };
    this.render();
  }

  /**
   * Decode a save code and check that the core loads its run and meta
   * state. Returns the keys and what they hold, or the German reason.
   */
  private async readSaveCode(text: string): Promise<{ keys: Record<string, string>; lines: string[] } | { error: string }> {
    let decoded;
    try {
      decoded = await decodeSave(text);
    } catch (err) {
      console.warn('save code unreadable', err);
      return { error: de.save.errors.damaged ?? '' };
    }
    if (!decoded.ok) return { error: de.save.errors[decoded.error] ?? '' };
    const { run, meta } = runAndMetaOf(decoded.keys);
    if (!run && !meta) return { error: de.save.errors.nothing ?? '' };
    const lines: string[] = [];
    try {
      if (run) {
        const core = CoreRun.load(run);
        try {
          lines.push(t(de.save.containsRun, { line: runLine(core.view()) }));
        } finally {
          core.dispose();
        }
      } else {
        lines.push(de.save.containsNoRun);
      }
      if (meta) {
        const loaded = CoreMeta.load(meta);
        try {
          const view = loaded.view();
          lines.push(
            t(de.save.containsBook, {
              runs: view.stats.runs_finished,
              done: view.unlocks.filter((u) => u.unlocked).length,
              total: view.unlocks.length,
            }),
          );
        } finally {
          loaded.meta.free();
        }
      }
    } catch (err) {
      console.warn('save code rejected by the core', err);
      return { error: de.save.errors.rejected ?? '' };
    }
    return { keys: decoded.keys, lines };
  }

  /** Replace everything stored by the checked code and start over with it. */
  private applyImport(): void {
    const o = this.ui.overlay;
    if (o?.kind !== 'save-confirm') return;
    if (!replaceAllKeys(o.keys)) {
      this.ui.overlay = { kind: 'save-import', text: '', error: de.save.errors.write ?? '', checking: false };
      this.render();
      return;
    }
    window.location.reload();
  }

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
    this.resetTableChoices();
    if (dealt && view.game) {
      reshuffle();
      this.afterRender.push(() => this.dealEffect());
    }
    // A fresh series says hello; a run resumed mid-series does not.
    if (dealt && view.game && view.series.games_played === 0) this.greetSeries(view);
    else this.quipSeries = `${view.series.section}-${view.series.index}`;
    this.render();
  }

  private resetTableChoices(): void {
    this.ui.selected = this.skatOnTable();
    this.ui.handGame = false;
    this.ui.pickGame = null;
    this.ui.pickRisk = null;
    this.ui.pickNullRisk = null;
    this.ui.omaPick = false;
    this.ui.swapTake = null;
    this.ui.peekTrick = false;
    this.ui.risePick = [];
  }

  /**
   * While discarding, the cards that came from the skat lie on the table
   * at first (those the core lets go; a "Ramponiert" one goes to the hand).
   */
  private skatOnTable(): number[] {
    const game = this.ui.view?.game;
    if (game?.phase !== 'Discarding') return [];
    // "Drei drücken": the card drawn from a defender lies there too.
    return game.hand.filter((c) => (c.from_skat || c.drawn_from != null) && c.discardable).map((c) => c.id);
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
  private async act(action: RunAction, fast = false): Promise<void> {
    if (!this.core || this.ui.busy) return;
    const before = this.ui.view;
    this.ui.busy = true;
    // Any move ends a look at the last trick.
    this.ui.peekTrick = false;
    this.noteCardSources(action);
    let step;
    try {
      step = this.core.apply(action);
    } catch (err) {
      console.warn('action rejected', action, err);
      this.pickupFrom = null;
      this.pressFlight = null;
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
      await playTravel(this.screenEl, TRAVEL_MS * this.tempo());
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
   * card, then it is animated. The run is saved after every step.
   */
  private async playSteps(before: RunView | null, first: Step | null): Promise<void> {
    let previous = before;
    let view = before;
    let step = first;
    let earned: string[] = [];
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
        if (view.phase === 'Won' || view.phase === 'Lost') {
          clearRun();
          // A night that ends (ADR 0020) is no run of its own: the win
          // was recorded when der Alte fell.
          if (!view.night) this.recordRun(view);
        } else if (this.core) {
          saveRun(this.core.save());
        }
        earned = earned.concat(this.observe(step.events));
        await this.playEvents(previous, step.events, view);
        // A lost run: the pub lights go out before the end screen.
        if (view.phase === 'Lost') await lightsOut();
        // The table as the core has it now, before the next opponent thinks.
        this.ui.anim = null;
        this.ui.view = view;
        this.playedFrom = null;
        this.thrown = null;
        previous = view;
        step = null;
      }
    } finally {
      this.ui.busy = false;
      this.ui.anim = null;
      this.ui.thinking = null;
      // Nothing waits on a score board or boss sheet once the playback is over.
      if (this.ui.overlay?.kind === 'score' || this.ui.overlay?.kind === 'boss') this.ui.overlay = null;
      this.awaited = null;
      if (view) this.ui.view = view;
      this.playedFrom = null;
      this.thrown = null;
      this.resetTableChoices();
      if (view && (view.phase === 'Won' || view.phase === 'Lost')) {
        this.ui.ended = { won: view.phase === 'Won', view };
        this.ui.screen = 'end';
        this.ui.copy = null;
        sound.play(view.phase === 'Won' ? 'win' : 'lose', { delay: 200 });
        if (view.phase === 'Won') this.afterRender.push(() => fireworks());
      }
      // The end screen must come whatever went wrong before it (playtest
      // 04.11.2026: stuck on a lost board without a working button).
      try {
        this.announceUnlocks(earned);
      } catch (err) {
        console.warn('unlocks not shown', err);
      }
      try {
        this.render();
      } finally {
        lightsUp();
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
          if (this.pouring?.wash.length) {
            this.washAway(this.pouring.wash);
            this.pouring.wash = [];
          }
          this.passedOnce = true;
          this.showToast(t(de.toast.redealt, { left: after.series.redeals_left }));
          reshuffle();
          this.afterRender.push(() => this.dealEffect());
        }
        if (ev === 'RedealUndone') {
          this.showToast(de.toast.redealUndone);
          reshuffle();
          this.afterRender.push(() => this.dealEffect());
        }
        if (ev === 'RamschDealt') {
          this.showToast(de.toast.ramsch);
          sound.play('knock');
        }
        if (ev === 'EveningRulesEnded') this.showToast(de.toast.eveningEnded);
        if (ev === 'RoutesRerolled') {
          this.showToast(de.toast.routesRerolled);
          sound.play('card');
          this.afterRender.push(() => this.reelRoutes());
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
      if ('VoucherCapped' in ev) {
        // One Untot and one Wandelkarte per edition (DESIGN 7.2): the voucher paid Groschen instead.
        const { card_mod, special, holder, coins } = ev.VoucherCapped;
        const held = after.edition.find((c) => c.id === holder);
        this.showToast(
          t(de.toast.voucherCapped, { mod: contentName('card_mods', card_mod), holder: held ? cardName(held.kind) : '', special: contentName('card_mods', special), coins }),
          'top',
        );
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
          if (card && old && (before?.phase === 'Shop' || before?.phase === 'Encounter')) this.stampLater(old, card, stamps++);
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
      if ('DoppelgangerUnmatched' in ev) {
        await this.doppelAlone(ev.DoppelgangerUnmatched.card, after);
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
        this.ui.risen = null;
        this.riseShown.clear();
        // A new game: from here on the final view tells the story.
        this.ui.anim = null;
        this.ui.view = after;
        this.resetTableChoices();
        this.greetSeries(after);
        reshuffle();
        this.afterRender.push(() => this.dealEffect());
        continue;
      }
      if ('BossAnnounced' in ev) {
        this.showToast(t(de.toast.bossAnnounced, { name: contentName('bosses', ev.BossAnnounced.boss) }));
        continue;
      }
      if ('Kontra' in ev) {
        this.showToast(t(de.toast.kontra, { name: this.seatLabel(ev.Kontra.seat, after) }));
        sound.play('knock');
        await this.pause(DELAY.notice);
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
        this.afterRender.push(() => this.revealSkat());
      }
      if ('ConsumableUsed' in ev) {
        const id = ev.ConsumableUsed.consumable;
        // Poured over the table: a factor for the next game slams onto the felt.
        const poured = this.pouring;
        if (poured?.id === id && poured.kind === 'Table') {
          const effect = after.pending_next_game?.find((p) => p.consumable === id)?.effect;
          const factor = effect && typeof effect === 'object' ? ('Multiply' in effect ? effect.Multiply : effect.MultiplyWin) : null;
          if (factor) this.afterRender.push(() => this.slamFactor(factor));
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
        const bought = this.bought;
        this.bought = null;
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
        const bought = this.bought;
        this.bought = null;
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
      if ('CardOffersRerolled' in ev) {
        this.showToast(de.toast.cardsRerolled);
        sound.play('card');
        continue;
      }
      if ('CardOfferBought' in ev || 'PainterHired' in ev || 'CardCleansed' in ev) {
        // The card itself is told by the DeckChanged that follows (or the painter's sheet).
        if (!('PainterHired' in ev) || ev.PainterHired.price > 0) sound.play('coin');
        continue;
      }
      if ('CounterRaised' in ev) {
        this.counterRaised(ev.CounterRaised.counter, ev.CounterRaised.value, after);
        continue;
      }
      if ('SkatCardShown' in ev) {
        // "Spickzettel": the card lies face up beside the trick from now on.
        this.wiggleSource(ev.SkatCardShown.source);
        this.showToast(t(de.toast.skatShown, { card: cardName(ev.SkatCardShown.card.kind) }), 'top');
        continue;
      }
      if ('SkatPointsRaised' in ev) {
        // "Skatpfleger": the discards count more, say so at the relic.
        const { source, card_points } = ev.SkatPointsRaised;
        this.fireTriggers([{ source, lever: 'CardPoints', change: { Add: card_points } }], after);
        continue;
      }
      if ('GameScored' in ev) {
        // What fired, on the chips and cards still on screen, before the board covers them.
        // The regulars take the result, a Schneider is cut across the felt; what fired lights up meanwhile.
        let wait = this.fastForward ? 0 : await this.gameEndEffect(ev.GameScored.breakdown);
        if (this.fireTriggers(ev.GameScored.breakdown.lines, after)) wait = Math.max(wait, DELAY.trigger);
        if (wait) await this.pause(wait);
        const rest = events.slice(i + 1);
        const won = rest.find((e): e is Extract<RunEvent, { SeriesWon: unknown }> => typeof e === 'object' && 'SeriesWon' in e);
        // One loss sound per chain (playtest 04.11.2026): when this lost game ends the run
        // (ADR 0019), the end screen sounds it, not the board.
        const runEnds = rest.some((e) => typeof e === 'object' && 'RunLost' in e);
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
        if (motionOn()) await this.pause(900);
        continue;
      }
      if ('CoinsStashed' in ev) {
        const { source, amount, pot } = ev.CoinsStashed;
        const name = typeof source === 'object' && 'Relic' in source ? contentName('relics', source.Relic) : '';
        this.wiggleSource(source);
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
        await this.waitOn((resolve) => ({ kind: 'boss', boss, resolve }));
        this.ui.overlay = null;
        // The table opens: lights down, the spotlight onto both boss players
        // (once no dialog is open any more, see render()).
        this.entrancePending = true;
      }
    }
  }

  private async playGameEvent(ev: Extract<RunEvent, { Game: unknown }>['Game'], after: RunView, breaking: number[] = [], rising: RoseEvent | null = null): Promise<void> {
    const anim = this.ui.anim;
    if ('DoppelgangerSwapped' in ev) {
      await this.doppelSwap(ev.DoppelgangerSwapped, after);
      return;
    }
    if ('CardPlayed' in ev) {
      if (!anim) return;
      const { seat, card } = ev.CardPlayed;
      if (seat === 0 && this.doppelLanded === card.id) {
        // "Doppelgänger": the swap scene laid it into the trick already.
        this.doppelLanded = null;
        return;
      }
      if (anim.trick.length >= 3) anim.trick = [];
      // Where the card comes from, measured before the table changes.
      const from = seat === 0 ? (this.playedFrom ?? this.rectOf('.hand')) : this.rectOf(`.opp[data-seat="${seat}"] .portrait`);
      anim.trick.push({ seat, card: this.cardOf(card, after) });
      anim.handSizes[seat] = Math.max(0, (anim.handSizes[seat] ?? 0) - 1);
      if (seat === 0) anim.hand = anim.hand.filter((c) => c.id !== card.id);
      anim.turn = seat === 0 ? null : seat;
      anim.status = '';
      this.render();
      const el = this.screenEl.querySelector(`[data-trick-seat="${seat}"]`);
      const played = anim.trick[anim.trick.length - 1]?.card;
      if (el instanceof HTMLElement && from && !this.fastForward) this.throwOnto(el, seat, from, played?.back ?? null);
      sound.play('card', { rate: 0.94 + seat * 0.05 });
      await this.pause(seat === 0 ? DELAY.ownCard : DELAY.aiCard);
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
      this.trickQuip(after, trick.winner, card_points);
      this.render();
      if (!this.fastForward) this.reactToTrick(after, trick.winner, card_points);
      await this.pause(DELAY.trickShow);
      if (breaking.length) {
        // "Glasdaus": the card cracks and shatters before the trick leaves.
        for (const id of breaking) this.screenEl.querySelector(`.trick-card[data-card-id="${id}"]`)?.classList.add('is-shattering');
        sound.play('clink', { rate: 1.6 });
        await this.pause(DELAY.notice);
      }
      // "Untot": the card rises back into the hand, the substitute takes its place, then the trick leaves.
      if (rising) await this.riseCard(rising, after);
      const target = this.sweepTarget(trick.winner);
      const cards = Array.from(this.screenEl.querySelectorAll('.trick-card'));
      // A card still sliding onto its place lies there now.
      finishOn(cards);
      sound.play('trick');
      // The stack forms on the card that took the trick.
      const onto = cards.findIndex((c) => c.getAttribute('data-trick-seat') === String(trick.winner));
      if (target && !this.fastForward) await Promise.race([sweepAway(cards, target, DELAY.trickFly, onto), this.pause(DELAY.trickFly + 200)]);
      anim.trick = [];
      anim.status = '';
      this.render();
      return;
    }
    if ('Conceded' in ev) {
      this.showToast(de.toast.conceded);
      await this.pause(DELAY.notice);
      return;
    }
    if ('DefenderHandsSwapped' in ev) {
      if (anim) anim.handSizes = ev.DefenderHandsSwapped.hand_sizes.slice();
      this.showToast(de.toast.handsSwapped);
      sound.play('card');
      this.render();
      await this.pause(DELAY.notice);
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
      await this.pause(DELAY.notice);
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
      await this.pause(DELAY.notice);
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
      this.render();
      if ('Declared' in ev && !this.fastForward) this.declareEffect(ev.Declared.declaration);
      if ('SkatPickedUp' in ev && !this.fastForward) {
        this.pickUpFlights(ev.SkatPickedUp.skat, peeked);
        sound.play('card');
        await this.pause(DELAY.flip);
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
    this.wiggleSource(source);
    const sign = amount > 0 ? `+${amount}` : `−${Math.abs(amount)}`;
    this.showToast(t(de.toast.coins, { amount: sign, source: from }));
    // The coins fly from what paid them (its chip or card on screen), else from the last tap.
    const payer = this.triggerTargets(source)[0]?.getBoundingClientRect() ?? null;
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
    await this.pause(DELAY.notice);
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
    await this.pause(DELAY.notice * 0.5);
    // Step 2: the stamp margin tears off.
    el.innerHTML = cardSvg(card.kind, card.mods, 'full', card.mod_values, { redeeming: true });
    box.dataset.stage = 'torn';
    el.animate([{ transform: 'rotate(0)' }, { transform: 'rotate(-4deg)' }, { transform: 'rotate(3deg)' }, { transform: 'rotate(0)' }], { duration: 260, easing: 'ease-out' });
    sound.play('card', { rate: 1.3 });
    await this.pause(DELAY.notice * 0.6);
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
    await this.pause(DELAY.notice * 0.8);
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
    await this.pause(DELAY.notice);
    this.ui.rising = null;
  }

  /**
   * "Doppelgänger" (DoppelgangerSwapped): a little scene. The played card
   * flashes in its violet mirror look with a shimmer, a dotted reach goes
   * to the robbed opponent (who frowns), the two cards cross in the air:
   * the stolen one into the trick, the Doppelgänger into his hand;
   * "Getauscht!". Tempo-scaled, off with reduced motion, a tap ends it.
   */
  private async doppelSwap(swap: DoppelSwap, after: RunView): Promise<void> {
    const { card, from, stolen } = swap;
    const name = this.seatLabel(from, after);
    this.showToast(t(de.toast.doppelSwapped, { card: cardName(card.kind), name, stolen: cardName(stolen.kind) }), 'top');
    const anim = this.ui.anim;
    if (!anim) return;
    const handRect = this.playedFrom ?? this.rectOf(`.hand-card[data-id="${card.id}"]`) ?? this.rectOf('.hand');
    const portrait = this.screenEl.querySelector(`.opp[data-seat="${from}"] .portrait`);
    const portraitRect = portrait?.getBoundingClientRect() ?? null;
    const mirror = lookSvg(this.cardOf(card, after));
    const stolenView = this.cardOf(stolen, after);
    if (anim.trick.length >= 3) anim.trick = [];
    anim.hand = anim.hand.filter((c) => c.id !== card.id);
    anim.handSizes[0] = Math.max(0, (anim.handSizes[0] ?? 0) - 1);
    anim.trick.push({ seat: 0, card: stolenView });
    anim.turn = null;
    anim.status = '';
    this.doppelLanded = stolen.id;
    this.render();
    const slot = this.screenEl.querySelector<HTMLElement>('[data-trick-seat="0"]');
    if (this.fastForward || !handRect || !portraitRect || !slot) {
      sound.play('card');
      await this.pause(DELAY.ownCard);
      return;
    }
    const slotRect = slot.getBoundingClientRect();
    // A card-sized box on the portrait, where the stolen card comes from.
    const atPortrait = new DOMRect(portraitRect.left + portraitRect.width / 2 - slotRect.width / 2, portraitRect.top + portraitRect.height / 2 - slotRect.height / 2, slotRect.width, slotRect.height);
    slot.style.visibility = 'hidden';
    try {
      await mirrorShimmer(mirror, handRect);
      doppelReach(handRect, portraitRect);
      if (portrait) frown(portrait);
      sound.play('card', { rate: 0.85 });
      await Promise.all([
        swapFlight(mirror, handRect, portraitRect, -48, 'doppel-flight-given', true),
        swapFlight(lookSvg(stolenView), atPortrait, slotRect, 48, 'doppel-flight-stolen'),
      ]);
    } finally {
      slot.style.visibility = '';
    }
    sceneWord(de.doppel.swapped, slotRect, 'doppel-word');
    sound.play('card', { rate: 1.1 });
    await this.pause(DELAY.notice * 0.6);
  }

  /** "Doppelgänger" without a counterpart: only its shimmer and "Kein Gegenstück"; the card is played as itself. */
  private async doppelAlone(card: DealtCard, after: RunView): Promise<void> {
    this.showToast(t(de.toast.doppelUnmatched, { card: cardName(card.kind) }), 'top');
    const rect = this.playedFrom ?? this.rectOf(`.hand-card[data-id="${card.id}"]`);
    if (this.fastForward || !rect) return;
    sceneWord(de.doppel.unmatched, rect, 'doppel-word');
    await mirrorShimmer(lookSvg(this.cardOf(card, after)), rect);
  }

  private seatLabel(seat: Seat, view: RunView): string {
    return seatLabelOf(view, seat);
  }

  /** Tempo factor of the settings, for effects outside ./ui/fx.ts. */
  private tempo(): number {
    return TEMPO_FACTOR[this.ui.settings.tempo];
  }

  /** What stands for a source on screen (its relic chip, its card). */
  private triggerTargets(source: Source): Element[] {
    return targetsOf(this.screenEl, source);
  }

  /** Wiggle what stands for a source on screen (its relic chip, its card). */
  private wiggleSource(source: Source): void {
    if (this.fastForward) return;
    this.triggers.fire([{ source, text: '' }], this.screenEl, this.tempo(), false);
  }

  /**
   * Relics and card modifications whose score lines are in the result: wiggle
   * them and say what they did. Returns whether anything fired.
   */
  private fireTriggers(lines: ScoreLine[], after: RunView): boolean {
    const fired: Trigger[] = lines
      .filter((l) => typeof l.source === 'object' && ('Relic' in l.source || 'CardMod' in l.source))
      .map((l) => ({
        source: l.source,
        text: t(de.trigger.line, { source: sourceName(l.source, after.edition), effect: changeText(l.lever, l.change) }),
      }));
    if (!fired.length || this.fastForward) return false;
    this.positionTriggers();
    this.triggers.fire(fired, this.screenEl, this.tempo());
    return true;
  }

  /** A run counter went up: the relics that count it wiggle and say the new value. */
  private counterRaised(counter: string, value: number, after: RunView): void {
    const template = de.counters[counter];
    if (!template || this.fastForward) return;
    const owners = after.relics.filter((r) => relicCounters(r.relic.id).includes(counter));
    if (!owners.length) return;
    this.positionTriggers();
    this.triggers.fire(
      owners.map((r) => ({ source: { Relic: r.relic.id }, text: t(template, { value }) })),
      this.screenEl,
      this.tempo(),
    );
  }

  /** The trigger lines sit on the felt above the trick, else right below the head of the run. */
  private positionTriggers(): void {
    const felt = this.screenEl.querySelector('.table .center')?.getBoundingClientRect();
    const hud = this.screenEl.querySelector('.hud')?.getBoundingClientRect();
    this.triggers.place(felt ? felt.top + 8 : hud ? hud.bottom + 6 : null);
  }

  /**
   * A Stammgast says one of his lines (from the core content) in a small
   * bubble at his seat: hello at the first game of a series, and now and
   * then after a trick worth something. One bubble at a time, never while
   * the Wirt explains something, gone after a few seconds (tempo setting).
   * Pure decoration: it changes nothing and catches no taps.
   */
  private sayQuip(view: RunView, seat: 1 | 2, line: 'greet' | 'win' | 'lose'): void {
    const guest = seatPerson(view, seat).guest;
    if (!guest || this.hint) return;
    this.ui.quip = { seat, text: guest[line] };
    this.tricksSinceQuip = 0;
    window.clearTimeout(this.quipTimer);
    const ms = Math.max(1800, QUIP_MS * TEMPO_FACTOR[this.ui.settings.tempo]);
    this.quipTimer = window.setTimeout(() => {
      this.ui.quip = null;
      // Remove just the bubble: a full render could cut a running animation.
      this.screenEl.querySelector('[data-testid="quip"]')?.remove();
    }, ms);
  }

  /** Hello from a Stammgast when a new series sits down at the table. */
  private greetSeries(view: RunView): void {
    const key = `${view.series.section}-${view.series.index}`;
    if (key === this.quipSeries) return;
    this.quipSeries = key;
    // The boss on the left has no lines; else the two take turns per series.
    const seat = view.series.is_boss ? 2 : (((view.series.section * 3 + view.series.index) % 2) + 1) as 1 | 2;
    this.sayQuip(view, seat, 'greet');
  }

  /** Now and then a line after a trick worth something (Augen from the core). */
  private trickQuip(view: RunView, winner: Seat, points: number): void {
    this.tricksSinceQuip += 1;
    if (this.tricksSinceQuip < QUIP_EVERY_TRICKS || points < 10) return;
    if (winner !== 0) {
      this.sayQuip(view, winner, 'win');
      return;
    }
    const seat = view.series.is_boss ? 2 : ((this.tricksSinceQuip % 2) + 1) as 1 | 2;
    this.sayQuip(view, seat, 'lose');
  }

  /**
   * Before an action goes to the core: where the cards it will move are
   * seen now (later renders while the core's events play take them away).
   * Picking up the skat notes the face-down skat and the hand; Drücken the
   * cards lying on the table; the Ansage after the skat its row of pressed
   * cards, which then fly into the stack in the felt's corner.
   */
  private noteCardSources(action: RunAction): void {
    this.pickupFrom = null;
    if (!motionOn()) return;
    if (action === 'PickUpSkat') {
      const pile = Array.from(this.screenEl.querySelectorAll<HTMLElement>('.skat-pile .skat-spot')).map(spotOf);
      this.pickupFrom = { pile, hand: spotsOf(this.screenEl, '.hand-card') };
      return;
    }
    if (typeof action !== 'object') return;
    const from = 'Discard' in action || 'DiscardThree' in action ? '.skat-zone .skat-card' : 'Declare' in action ? '[data-testid="pressed"] .mini-card' : null;
    if (!from) return;
    const cards = Array.from(this.screenEl.querySelectorAll<HTMLElement>(from)).map((el) => ({ spot: spotOf(el), face: el.querySelector('svg')?.outerHTML ?? '' }));
    if (cards.length) this.pressFlight = { cards, stackOnly: 'Declare' in action, until: performance.now() + 5000 };
  }

  /**
   * The skat is picked up: each card is lifted off the face-down pile,
   * turns face up on the way (not after "Kurzer": it lay face up) and
   * lands where it goes, on the table to be pressed or, one the core keeps
   * from the skat ("Ramponiert"), in its sorted place in the hand. The hand
   * cards glide into their new places meanwhile.
   */
  private pickUpFlights(skat: DealtCard[], peeked: boolean): void {
    const seen = this.pickupFrom;
    this.pickupFrom = null;
    const zone = this.rectOf('.skat-zone') ?? this.rectOf('.deck-spot');
    skat.forEach((c, i) => {
      const el = this.screenEl.querySelector<HTMLElement>(`.skat-zone [data-id="${c.id}"], .hand-card[data-id="${c.id}"]`);
      if (!el) return;
      const at = seen?.pile[i] ?? (zone ? spotOfRect(zone) : null);
      if (!at) return;
      glide(el, at, { ...PICK, arc: (i % 2 ? -1 : 1) * PICK.arc, delay: i * 90, above: true, ...(peeked ? {} : { cover: cardBack() }) });
    });
    if (seen) glideAll(Array.from(this.screenEl.querySelectorAll<HTMLElement>('.hand-card')), seen.hand, { ms: 320, skip: new Set(skat.map((c) => String(c.id))) });
  }

  /**
   * The pressed cards fly onto their place once a render shows it: after
   * Drücken into the row of the Ansage (face up), after the Ansage into the
   * stack in the felt's corner, turning face down on the way.
   */
  private flyPressed(): void {
    const p = this.pressFlight;
    if (!p) return;
    if (this.fastForward || performance.now() > p.until || this.ui.screen !== 'run') {
      this.pressFlight = null;
      return;
    }
    const stack = Array.from(this.screenEl.querySelectorAll<HTMLElement>('[data-testid="pressed-play"] .pressed-card'));
    const row = p.stackOnly ? [] : Array.from(this.screenEl.querySelectorAll<HTMLElement>('[data-testid="pressed"] .mini-card'));
    const targets = stack.length ? stack : row;
    if (!targets.length) return;
    this.pressFlight = null;
    p.cards.forEach((c, i) => {
      const el = targets[i];
      if (!el) return;
      // Into the stack it turns face down; into the Ansage's row its full face shrinks onto the small one.
      const cover = c.face ? { cover: c.face, turn: stack.length > 0 } : {};
      glide(el, c.spot, { ms: 480, lift: 22, arc: (i % 2 ? -1 : 1) * 12, delay: i * 80, above: true, ...cover });
    });
  }

  private rectOf(selector: string): DOMRect | null {
    const el = this.screenEl.querySelector(selector);
    return el ? el.getBoundingClientRect() : null;
  }

  /** The opponents' portraits on the table, by seat. */
  private faces(): Map<1 | 2, Element> {
    const out = new Map<1 | 2, Element>();
    for (const seat of [1, 2] as const) {
      const el = this.screenEl.querySelector(`.opp[data-seat="${seat}"] .portrait`);
      if (el) out.set(seat, el);
    }
    return out;
  }

  /**
   * The regulars react to a trick: the one who took it nods; a big one
   * the player took makes both frown. In a Ramsch everyone plays for
   * himself and a trick is no gain, so nobody reacts.
   */
  private reactToTrick(view: RunView, winner: Seat, points: number): void {
    if (view.game?.declaration?.game === 'Ramsch') return;
    const faces = this.faces();
    if (winner !== 0) {
      const el = faces.get(winner);
      if (el) nod(el);
      return;
    }
    if (points < BIG_TRICK) return;
    for (const el of faces.values()) frown(el);
  }

  /**
   * The game is scored: the regulars frown when the player won it, nod
   * when he lost; a win with Schneider is snipped across the felt. Returns
   * how long to let it show (ms at tempo Normal), 0 if nothing plays.
   */
  private async gameEndEffect(b: ScoreBreakdown): Promise<number> {
    if (!motionOn() || b.declaration.game === 'Ramsch') return 0;
    for (const el of this.faces().values()) (b.won ? frown : nod)(el);
    const felt = this.rectOf('.center');
    if (b.won && b.schneider && felt) {
      await Promise.race([snipAcross(this.root, felt), this.pause(1100)]);
      return 250;
    }
    return 520;
  }

  /**
   * The Ansage slams onto the felt as a sign, then shrinks into the game
   * line. Play goes on underneath at once (the sign takes no tap), so the
   * game is no slower for it.
   */
  private declareEffect(d: Declaration): void {
    const felt = this.rectOf('.center');
    if (!felt || !motionOn()) return;
    const into = this.rectOf('.gameline .decl');
    declareSign(this.root, declarationSign(d), felt, into, { onHit: () => sound.play('knock', { rate: 1.15, volume: 0.5 }) });
  }

  /**
   * Where a finished trick goes: into the portrait of the regular who
   * took it (his name plate), or down past the hand to the player. Without
   * a portrait, off the edge of the table on his side.
   */
  private sweepTarget(seat: Seat): { x: number; y: number } | null {
    const table = this.rectOf('.table');
    const trick = this.rectOf('.trick');
    if (!table || !trick) return null;
    const card = trick.height;
    const cy = trick.top + trick.height / 2;
    if (seat === 0) return { x: trick.left + trick.width / 2, y: table.bottom + card * 0.6 };
    const face = this.rectOf(`.opp[data-seat="${seat}"] .portrait`);
    if (face) return { x: face.left + face.width / 2, y: face.top + face.height / 2 };
    return seat === 1 ? { x: table.left - card, y: cy } : { x: table.right + card, y: cy };
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
  private async waitOn(make: (resolve: () => void) => Extract<Overlay, { kind: 'score' } | { kind: 'boss' }>): Promise<void> {
    let resolve: () => void = () => undefined;
    const done = new Promise<void>((r) => (resolve = r));
    const overlay = make(resolve);
    this.ui.overlay = overlay;
    this.awaited = overlay;
    try {
      this.render();
    } catch (err) {
      console.error('render under a sheet failed', err);
      this.overlayDirty = true;
      try {
        this.renderLayer();
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

  /** Cards fly from the deck to the hands, in turns, then two to the skat. */
  private dealEffect(): void {
    const from = this.rectOf('.deck-pile') ?? this.rectOf('.deck-spot');
    if (!from) return;
    const hand = Array.from(this.screenEl.querySelectorAll('.hand-card'));
    const left = Array.from(this.screenEl.querySelectorAll('.opp[data-seat="1"] .opp-hand .card-back'));
    const right = Array.from(this.screenEl.querySelectorAll('.opp[data-seat="2"] .opp-hand .card-back'));
    const skat = Array.from(this.screenEl.querySelectorAll('.skat-pile .skat-spot'));
    const order: Element[] = [];
    for (let i = 0; i < Math.max(hand.length, left.length, right.length); i++) {
      for (const list of [hand, left, right]) {
        const el = list[i];
        if (el) order.push(el);
      }
    }
    order.push(...skat);
    deal(order, from, {
      step: 22,
      ms: 280,
      onCard: (i) => {
        if (i % 4 === 0) sound.play('card', { volume: 0.45, rate: 0.95 + Math.random() * 0.2 });
      },
    });
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
    await this.waitOn((resolve) => ({ kind: 'score', score, previousScore, seriesScore, target, payout, relics: after.relics, seatNames, quietLoss, resolve }));
    this.scoreBoard = null;
    this.ui.overlay = null;
    // The board goes; the account in the head counts to the new value.
    this.renderLayer();
    if (previousScore !== seriesScore && this.screenEl.querySelector('.hud [data-testid="account"]')) {
      void this.hudFx.account(seriesScore, target);
      if (motionOn()) await this.pause(520);
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
      await this.pause(i === steps - 1 ? 180 : 85);
    }
    showWildSuit(svg, f.rank, f.suit);
    card.classList.remove('is-settling');
  }

  /** Wait, unless the player asked to hurry. */
  private pause(ms: number): Promise<void> {
    if (this.fastForward) return Promise.resolve();
    const scaled = ms * TEMPO_FACTOR[this.ui.settings.tempo] * (reducedMotion() ? 0.25 : 1);
    return new Promise((resolve) => {
      const timer = window.setTimeout(done, scaled);
      const wakers = this.wakers;
      function done(): void {
        window.clearTimeout(timer);
        const i = wakers.indexOf(done);
        if (i >= 0) wakers.splice(i, 1);
        resolve();
      }
      wakers.push(done);
    });
  }

  private hurry(): void {
    this.fastForward = true;
    finishAll();
    for (const wake of this.wakers.slice()) wake();
  }

  /** Show a toast; while one is up, the next ones wait their turn. */
  /**
   * A short message. `top`: above the Stammgäste, for messages while
   * cards lie on the felt (Drücken, the Ansage, the trick) that must stay
   * readable.
   */
  private showToast(text: string, place: ToastPlace = 'middle'): void {
    if (this.toastTimer !== undefined) {
      this.toasts.push({ text, place });
      if (this.toasts.length > TOAST_QUEUE) this.toasts.shift();
      return;
    }
    this.toastEl.innerHTML = `<div class="toast">${esc(text)}</div>`;
    this.toastEl.classList.toggle('is-top', place === 'top');
    this.toastEl.style.top = '';
    if (place === 'top') {
      // The toast's bottom edge sits just above the opponents' row.
      const opps = this.screenEl.querySelector('.opponents')?.getBoundingClientRect();
      const height = this.toastEl.querySelector('.toast')?.getBoundingClientRect().height ?? 0;
      if (opps) this.toastEl.style.top = `${Math.round(Math.max(height + 4, opps.top - 4))}px`;
    }
    // Shorter while others wait.
    const ms = this.toasts.length ? DELAY.toast * 0.7 : DELAY.toast;
    this.toastTimer = window.setTimeout(() => {
      this.toastTimer = undefined;
      this.toastEl.innerHTML = '';
      const next = this.toasts.shift();
      if (next !== undefined) this.showToast(next.text, next.place);
    }, ms);
  }

  private setSettings(change: Partial<UiState['settings']>): void {
    this.ui.settings = { ...this.ui.settings, ...change };
    saveSettings(this.ui.settings);
    this.applySettings();
    this.overlayDirty = true;
    this.render();
  }

  // -------------------------------------------------------------------------
  // Hints of the Wirt
  // -------------------------------------------------------------------------

  private markHintSeen(id: HintId): void {
    if (this.hintSeen.has(id)) return;
    this.hintSeen.add(id);
    saveSeenHints(this.hintSeen);
  }

  /**
   * Show the hint that fits the screen now, if any. A hint shown before
   * whose situation has passed counts as seen; a dialog in front of it only
   * hides it for the moment.
   */
  private updateHint(): void {
    const ctx = { ui: this.ui, passed: this.passedOnce };
    const on = this.ui.settings.hints;
    if (on) {
      for (const id of this.hintsShown) {
        if (hintApplies(id, ctx)) continue;
        this.markHintSeen(id);
        this.hintsShown.delete(id);
      }
    }
    const pick = on ? pickHint(ctx, this.hintSeen) : null;
    if (!pick) {
      if (this.hint) this.hintEl.innerHTML = '';
      this.hint = null;
      return;
    }
    // One bubble at a time: the Wirt goes before a Stammgast's line.
    if (this.ui.quip) {
      this.ui.quip = null;
      this.screenEl.querySelector('[data-testid="quip"]')?.remove();
    }
    const html = renderHint(pick.id, pick.anchor, hintText(pick.id, this.ui));
    if (html !== this.hint?.html) {
      // The same hint with a new number does not pop in again.
      this.hintEl.classList.toggle('is-static', this.hint?.id === pick.id);
      this.hintEl.innerHTML = html;
    }
    this.hint = { ...pick, html };
    this.hintsShown.add(pick.id);
    this.positionHint();
  }

  /** Put the bubble right below what it hangs from: the head of the run, the Wirt at the Tresen, the end title, or the top. */
  private positionHint(): void {
    const hint = this.hint;
    if (!hint) return;
    const selector = { hud: '.hud', end: '.end-board h1', shop: '.shop .backbar, .shop .backbar > *', top: null }[hint.anchor];
    const bottoms = selector ? Array.from(this.screenEl.querySelectorAll(selector), (el) => el.getBoundingClientRect().bottom) : [];
    this.hintEl.style.top = bottoms.length ? `${Math.round(Math.max(...bottoms) + 6)}px` : '';
  }

  private dismissHint(): void {
    const id = this.hint?.id;
    if (!id) return;
    this.markHintSeen(id);
    this.hintsShown.delete(id);
    this.updateHint();
  }

  /** Menu "Hinweise zurücksetzen": every hint shows again. */
  private resetHints(): void {
    clearSeenHints();
    this.hintSeen = new Set();
    this.hintsShown.clear();
    this.passedOnce = false;
    this.showToast(de.menu.hintsResetDone);
    sound.play('clink');
    this.render();
  }

  // -------------------------------------------------------------------------
  // Input
  // -------------------------------------------------------------------------

  private onClick(e: MouseEvent): void {
    const target = e.target as HTMLElement | null;
    const el = target?.closest<HTMLElement>('[data-action]');
    const overlay = this.ui.overlay;
    if (!el) {
      if (this.ui.busy && !overlay) this.hurry();
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
        this.newRun();
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
        if (shared) this.requestStart({ seed: shared.seed, character: shared.locked ? 'Declarer' : shared.character, length: shared.length, stake: sharedStake(this.ui), daily: false });
        return;
      }
      case 'dismiss-shared':
        this.ui.shared = null;
        clearShareHash();
        this.render();
        return;
      case 'play-daily':
        this.requestStart({ seed: dailySeed(), character: this.ui.pickCharacter, length: this.ui.pickLength, stake: this.pickedStake(), daily: true });
        return;
      case 'replay': {
        const c = el.dataset.character ?? null;
        const seed = el.dataset.seed ?? '';
        const entry = this.ui.meta?.characters.find((x) => x.character === c);
        const length: RunLength = el.dataset.length === 'Short' ? 'Short' : 'Full';
        // Replayed at its stake, or the highest unlocked if that is lower.
        const stake = isCharacter(c) ? Math.min(num('stake') < 0 ? 0 : num('stake'), maxStake(this.ui.meta, c, length)) : 0;
        if (isCharacter(c) && entry?.unlocked && isSeedText(seed)) this.requestStart({ seed, character: c, length, stake, daily: false });
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
      case 'card-back':
        this.setSettings({ back: el.dataset.back ? el.dataset.back : null });
        return;
      case 'hints-reset':
        this.resetHints();
        return;
      case 'hint-dismiss':
        this.dismissHint();
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
      case 'save-export':
        if (this.ui.busy || overlay?.kind !== 'save-menu') return;
        void this.openSaveExport();
        return;
      case 'save-copy':
        this.copySave();
        return;
      case 'save-share':
        this.shareSave();
        return;
      case 'save-select':
        this.selectSaveCode();
        return;
      case 'save-import':
        if (this.ui.busy) return;
        this.ui.overlay = { kind: 'save-import', text: '', error: null, checking: false };
        this.render();
        return;
      case 'save-check':
        void this.checkImport();
        return;
      case 'save-apply':
        this.applyImport();
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
      case 'pick-length': {
        const length = el.dataset.length;
        if (length === 'Full' || length === 'Short') {
          this.ui.pickLength = length;
          this.ui.pickStake = null;
          saveLength(length);
          this.render();
        }
        return;
      }
      case 'pick-stake': {
        const max = maxStake(this.ui.meta, this.ui.pickCharacter, this.ui.pickLength);
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
        if (this.ui.busy) this.hurry();
        return;
      case 'last-trick':
        // Show the last finished trick on the felt, or the open one again.
        if (this.ui.anim || this.ui.busy || !view?.game?.last_trick) return;
        this.ui.peekTrick = !this.ui.peekTrick;
        this.render();
        return;
      case 'score-skip':
        if (overlay?.kind === 'score' && this.scoreBoard) this.scoreBoard.skip();
        else this.redrawLayer();
        return;
      case 'score-continue':
      case 'boss-go':
        this.closeAwaited(action === 'score-continue' ? 'score' : 'boss');
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
      // The card reward after a won pub (DESIGN 5.0): a Zinke, then its card, then "Anbringen".
      case 'card-reward-pick': {
        const o = view.card_reward?.offers.find((x) => x.index === num('offer'));
        if (!o) return;
        const reasons = this.refusals(view, o.targets, (id) => ({ TakeCardReward: { offer: o.index, card: id } }));
        this.ui.overlay = { kind: 'reward-target', offer: o.index, reasons };
        this.render();
        return;
      }
      case 'reward-target': {
        const o = this.ui.overlay;
        if (o?.kind !== 'reward-target') return;
        const offer = view.card_reward?.offers.find((x) => x.index === o.offer);
        if (!offer) return;
        this.noteStamp(el);
        const action: RunAction = { TakeCardReward: { offer: o.offer, card: num('id') } };
        this.applyMod(view, action, num('id'), offer.card_mod.id, false, o, 'apply');
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
          const reasons = marks ? this.refusals(view, option.targets, (id) => ({ ChooseEncounter: { option: option.index, target: id } })) : {};
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
        this.noteStamp(el);
        // The Zinke on the chosen card (a Painter or cleansing names none); a chosen Marke.
        const chosen = option?.effects.find((e) => typeof e.effect === 'object' && 'Mark' in e.effect && e.effect.Mark.card === 'Chosen');
        // A Marke drawn when taken: no preview of the card afterwards (04.11.2026).
        const drawn = option ? markeEffect(option)?.marke === 'Random' : false;
        if (drawn && option) {
          const marken = (option.marken ?? []).filter((m) => m.can_choose).map((m) => contentName('card_mods', m.card_mod.id) || m.card_mod.name);
          const names = marken.length > 1 ? `${marken.slice(0, -1).join(', ')} ${de.encounter.or} ${marken[marken.length - 1]}` : (marken[0] ?? '');
          this.applyRandomMarke(view, action, num('id'), names, o);
          return;
        }
        // The card before and after, then "Anbringen" (playtest 04.11.2026).
        this.applyMod(view, action, num('id'), o.marke ?? chosen?.card_mod?.id ?? null, false, o, 'apply');
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
          const reasons = this.refusals(view, option.marken?.find((m) => m.card_mod.id === marke)?.targets ?? [], (id) => ({ ChooseEncounter: { option: option.index, target: id, marke } }));
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
        if (o?.kind === 'mod-confirm') this.noteStamp(this.layerEl.querySelector('.mod-confirm-card:last-of-type .card'));
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
        if (sel.includes(id)) this.moveCards(sel.filter((x) => x !== id), id, null);
        else if (sel.length < need) this.moveCards([...sel, id], id, null);
        else this.moveCards([...sel.slice(1), id], id, null);
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
        this.playCard(num('id'), el.getBoundingClientRect(), null);
        return;
      case 'buy-relic':
        this.noteBought(el.closest('.relic-tile')?.querySelector('.relic-tile-emblem svg') ?? null);
        void this.act({ BuyRelic: num('slot') });
        return;
      case 'buy-item':
        this.noteBought(el.closest('.good')?.querySelector('.good-pic svg') ?? null);
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
          const reasons = marks.mod || marks.random ? this.refusals(view, item.targets, (id) => ({ UseConsumable: { slot, target: id } })) : {};
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
        this.noteStamp(el);
        const action: RunAction = { UseConsumable: { slot: o.slot, target: num('id') } };
        if (marks.mod || marks.random) this.applyMod(view, action, num('id'), marks.mod, marks.random, o, 'apply');
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
      case 'buy-zinke': {
        // Zinken (DESIGN 8.6): the card is fixed; ask first if it replaces or raises.
        const offer = view.shop?.card_offers?.find((o) => o?.slot === num('slot'));
        if (!offer) return;
        this.noteStamp(el.closest('.zinke')?.querySelector('.zinke-card') ?? null);
        this.applyMod(view, { BuyCardOffer: offer.slot }, offer.card.id, offer.card_mod.id, false, null);
        return;
      }
      case 'reroll-zinken':
        void this.act('RerollCardOffers');
        return;
      case 'painter':
      case 'cleanse': {
        const service: 'painter' | 'cleanse' = action === 'painter' ? 'painter' : 'cleanse';
        const s = service === 'painter' ? view.shop?.painter : view.shop?.cleanse;
        if (!s) return;
        const reasons = this.refusals(view, s.targets, (id) => (service === 'painter' ? { HirePainter: id } : { CleanseCard: id }));
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
          this.noteStamp(el);
          this.applyMod(view, { CleanseCard: num('id') }, num('id'), null, false, o, 'cleanse');
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
        this.noteStamp(el.querySelector('.card'));
        this.applyMod(view, { ChoosePaint: option.index }, painter.card.id, option.card_mod.id, false, { kind: 'painter' });
        return;
      }
      case 'leave-shop':
        void this.act('LeaveShop');
        return;
    }
  }

  /**
   * A Schnaps poured on its target (./ui/drink.ts): the same action as
   * "Trinken" on its info card. Blue Curaçao on a hand card dyes it and
   * opens the suit picker for that card; on the table the cards are kept
   * as pictures, to be washed away if the core deals anew.
   */
  private pourSchnaps(p: Pour): void {
    const view = this.ui.view;
    const item = view?.consumables.find((c) => c.slot === p.slot);
    if (!view || !item || this.ui.busy) return;
    const c = item.consumable;
    if (c.needs_suit) {
      const card = p.kind === 'Card' ? p.card : null;
      if (card !== null) this.dyeCard(card, p.at);
      const open = (): void => {
        if (this.ui.busy || this.ui.overlay || this.ui.view !== view) return;
        this.ui.overlay = { kind: 'chalk', slot: p.slot, card };
        this.render();
      };
      if (card !== null && motionOn()) window.setTimeout(open, 480 * this.tempo());
      else open();
      return;
    }
    if (c.needs_target) {
      // A Schnaps for a deck card: the card picker, as from the info card.
      const marks = consumableMod(c.id);
      const reasons = marks.mod || marks.random ? this.refusals(view, item.targets, (id) => ({ UseConsumable: { slot: p.slot, target: id } })) : {};
      this.ui.overlay = { kind: 'target', slot: p.slot, reasons };
      this.render();
      return;
    }
    this.pouring = { id: c.id, kind: p.kind, wash: p.kind === 'Table' ? this.snapshotTable() : [] };
    void this.act({ UseConsumable: { slot: p.slot, target: null } }).finally(() => {
      for (const el of this.pouring?.wash ?? []) el.remove();
      this.pouring = null;
    });
  }

  /** Pictures of the cards on the felt and in the hand, laid over them until the table is washed. */
  private snapshotTable(): HTMLElement[] {
    if (!motionOn()) return [];
    const layer = this.root.querySelector<HTMLElement>('.drink-layer');
    if (!layer) return [];
    return Array.from(this.screenEl.querySelectorAll<HTMLElement>('.declare-choice .felt .card, .declare-choice .felt .skat-pile, .table .hand .hand-card')).map((el) => {
      const r = el.getBoundingClientRect();
      const copy = el.cloneNode(true) as HTMLElement;
      copy.removeAttribute('data-testid');
      copy.removeAttribute('data-action');
      copy.removeAttribute('data-drag');
      copy.removeAttribute('data-info');
      copy.setAttribute('aria-hidden', 'true');
      copy.classList.add('wash-card');
      copy.style.left = `${r.left}px`;
      copy.style.top = `${r.top}px`;
      copy.style.width = `${r.width}px`;
      copy.style.height = `${r.height}px`;
      layer.appendChild(copy);
      return copy;
    });
  }

  /** Kräuter and Doppelkorn: a green wave washes the old cards off the table to the right. */
  private washAway(cards: HTMLElement[]): void {
    const layer = this.root.querySelector<HTMLElement>('.drink-layer');
    const ms = 760 * this.tempo();
    if (!layer || !cards.length || !motionOn()) {
      for (const el of cards) el.remove();
      return;
    }
    const table = this.rectOf('.run .table') ?? new DOMRect(0, 0, window.innerWidth, window.innerHeight);
    // The wave stays on the table (a box that clips it); the cards fly off the screen.
    const box = document.createElement('div');
    box.className = 'wash-box';
    box.style.left = `${table.left}px`;
    box.style.top = `${table.top}px`;
    box.style.width = `${table.width}px`;
    box.style.height = `${table.height}px`;
    const wave = document.createElement('div');
    wave.className = 'wash-wave';
    wave.dataset.testid = 'wash-wave';
    box.appendChild(wave);
    layer.appendChild(box);
    const w = window.innerWidth;
    const sweep = play(wave, [{ transform: 'translateX(-100%)' }, { transform: `translateX(${(table.width * 1.05).toFixed(0)}px)` }], { duration: ms, easing: 'cubic-bezier(.4,0,.7,1)', fill: 'both' });
    const done = (): void => box.remove();
    sweep.finished.then(done, done);
    for (const el of cards) {
      const r = el.getBoundingClientRect();
      const reach = Math.max(0, Math.min(1, (r.left + r.width / 2 - table.left) / table.width));
      const spin = (Math.random() - 0.3) * 50;
      const anim = play(el, 
        [
          { transform: 'translate(0, 0) rotate(0deg)', opacity: 1 },
          { transform: `translate(${(w - r.left + 40).toFixed(0)}px, ${(-20 - Math.random() * 60).toFixed(0)}px) rotate(${spin.toFixed(0)}deg)`, opacity: 0.9 },
        ],
        { duration: ms * 0.7, delay: ms * reach * 0.5, easing: 'cubic-bezier(.5,0,.8,.6)', fill: 'both' },
      );
      const gone = (): void => el.remove();
      anim.finished.then(gone, gone);
    }
    this.bigWord(de.drink.redeal, table, ms * 0.3, 'is-redeal');
  }

  /** A big pixel word over a box for a moment ("Neu geben!", "Wird neu ausgelost …"). */
  private bigWord(text: string, over: DOMRect, delay: number, cls: string, at = 0.38): void {
    const layer = this.root.querySelector<HTMLElement>('.drink-layer');
    if (!layer || !motionOn()) return;
    const word = document.createElement('div');
    word.className = `drink-big ${cls}`;
    word.dataset.testid = `drink-big-${cls.replace('is-', '')}`;
    word.textContent = text;
    word.style.left = `${over.left + over.width / 2}px`;
    word.style.top = `${over.top + over.height * at}px`;
    layer.appendChild(word);
    const ms = 1100 * this.tempo();
    const anim = play(word, 
      [
        { transform: 'translate(-50%, -50%) rotate(-6deg) scale(.3)', opacity: 0 },
        { transform: 'translate(-50%, -50%) rotate(-6deg) scale(1.1)', opacity: 1, offset: 0.15 },
        { transform: 'translate(-50%, -50%) rotate(-6deg) scale(1)', opacity: 1, offset: 0.25 },
        { transform: 'translate(-50%, -50%) rotate(-6deg) scale(1)', opacity: 1, offset: 0.8 },
        { transform: 'translate(-50%, -60%) rotate(-6deg) scale(1)', opacity: 0 },
      ],
      { duration: ms, delay, fill: 'both' },
    );
    const gone = (): void => word.remove();
    anim.finished.then(gone, gone);
  }

  /** Blue Curaçao on a hand card: the card takes on the blue. */
  private dyeCard(card: number, at: DOMRect): void {
    const layer = this.root.querySelector<HTMLElement>('.drink-layer');
    if (!layer || !motionOn()) return;
    const dye = document.createElement('div');
    dye.className = 'drink-dye';
    dye.dataset.testid = 'drink-dye';
    dye.dataset.card = String(card);
    dye.style.left = `${at.left}px`;
    dye.style.top = `${at.top}px`;
    dye.style.width = `${at.width}px`;
    dye.style.height = `${at.height}px`;
    layer.appendChild(dye);
    const ms = 1100 * this.tempo();
    const anim = play(dye, 
      [
        { clipPath: 'inset(0 0 100% 0)', opacity: 0.85 },
        { clipPath: 'inset(0 0 0 0)', opacity: 0.85, offset: 0.4 },
        { clipPath: 'inset(0 0 0 0)', opacity: 0.6, offset: 0.75 },
        { clipPath: 'inset(0 0 0 0)', opacity: 0 },
      ],
      { duration: ms, fill: 'both', easing: 'steps(8, end)' },
    );
    const gone = (): void => dye.remove();
    anim.finished.then(gone, gone);
  }

  /** Schwenker: the stops of every way spin like the reels of a slot machine and settle one by one. */
  private reelRoutes(): void {
    if (!motionOn()) return;
    const map = this.rectOf('.route-map');
    if (map) this.bigWord(de.drink.reroll, map, 0, 'is-reroll', 0.2);
    const reels = Array.from(this.screenEl.querySelectorAll<HTMLElement | SVGElement>('.route-options > *, .route-map .map-badge, .route-map .map-target'));
    reels.forEach((el, i) => {
      el.classList.add('is-reeling');
      el.style.setProperty('--reel-delay', `${Math.round(i * 90 * this.tempo())}ms`);
      el.style.setProperty('--reel-ms', `${Math.round(620 * this.tempo())}ms`);
      el.addEventListener('animationend', () => el.classList.remove('is-reeling'), { once: true });
    });
  }

  /** Kurzer: the skat turns face up. */
  private revealSkat(): void {
    if (!motionOn()) return;
    this.screenEl.querySelectorAll<HTMLElement>('.skat-pile .skat-spot').forEach((el, i) => flip(el, cardBack(), { ms: 380, delay: 80 + i * 120 }));
  }

  /** Doppelkorn and Fusel: "×2" slams onto the felt. */
  private slamFactor(factor: number): void {
    const felt = this.rectOf('.run .felt') ?? this.rectOf('.run .table');
    if (!felt || !motionOn()) return;
    const html = `<span class="decl-sign-text x2-sign" data-testid="x2-sign"><b>${esc(t(de.drink.times, { factor }))}</b></span>`;
    declareSign(this.root, html, felt, this.rectOf('.slate'), { ms: 1500, onHit: () => sound.play('knock', { rate: 0.9, volume: 0.6 }) });
  }

  /**
   * Why the core refuses each card a modification would go on: its error
   * code from a preview of the action (CardCursed, ModAtMaximum, …), for
   * the dimmed cards of the picker. Read-only.
   */
  private refusals(view: RunView, targets: number[], action: (id: number) => RunAction): Record<number, string> {
    const out: Record<number, string> = {};
    if (!this.core) return out;
    for (const c of view.edition) {
      if (targets.includes(c.id)) continue;
      try {
        this.core.preview(action(c.id));
      } catch (err) {
        const code = errorCode(err);
        if (code) out[c.id] = code;
      }
    }
    return out;
  }

  /** The picture of what is being bought at the Tresen and where it is (for its flight into the Leiste). */
  private noteBought(pic: Element | null): void {
    const rect = pic?.getBoundingClientRect();
    this.bought = pic && rect?.width ? { html: pic.outerHTML, rect } : null;
  }

  /** Where the card is that a Zinke is about to go onto (its stamp comes down there). */
  private noteStamp(card: Element | null): void {
    const rect = card?.getBoundingClientRect();
    this.stampAt = rect?.width ? { rect, at: performance.now() } : null;
  }

  /**
   * After the next render, stamp a Zinke onto `card` (as it was: `old`),
   * where it was tapped, or in the middle of the screen if that place is
   * gone or out of sight. Only pictures: the faces come from the core's
   * edition before and after the step.
   */
  private stampLater(old: DeckCard, card: DeckCard, order: number): void {
    const noted = this.stampAt && performance.now() - this.stampAt.at < 60000 ? this.stampAt.rect : null;
    if (order === 0) this.stampAt = null;
    const w = window.innerWidth;
    const h = window.innerHeight;
    const seen = noted && noted.top >= 0 && noted.bottom <= h && noted.left >= 0 && noted.right <= w;
    const cw = Math.min(110, Math.round(w * 0.26));
    const ch = Math.round(cw * 1.5);
    const shift = order * Math.round(cw * 0.35);
    const place = seen && order === 0 ? noted : new DOMRect(Math.round(w / 2 - cw / 2 + shift), Math.round(h * 0.42 - ch / 2), cw, ch);
    const before = cardSvg(old.kind, old.mods, 'full', old.mod_values);
    const after = cardSvg(card.kind, card.mods, 'full', card.mod_values);
    this.afterRender.push(() => zinkeStamp(before, after, place, order));
  }

  /**
   * A card modification goes on a card: if the core's preview says it
   * replaces, raises or cancels the one on the card, ask first (an in-app
   * sheet); otherwise apply it at once. One drawn at random is not
   * previewed: the sheet only says that the card carries one already.
   */
  private applyMod(view: RunView, action: RunAction, id: number, mod: string | null, random: boolean, back: Overlay | null, always: 'apply' | 'cleanse' | null = null): void {
    const card = view.edition.find((c) => c.id === id);
    const ask = card ? this.modQuestion(card, action, mod, random, always) : null;
    if (!ask) {
      void this.act(action);
      return;
    }
    this.ui.overlay = { kind: 'mod-confirm', action, back, ask };
    this.render();
  }

  /**
   * A Marke drawn when the option is taken goes on the chosen card
   * (Kartenwerkstatt, 04.11.2026): the card now, the Marken that may come
   * and "Ziehen". The core is not asked for a preview, which would tell the
   * draw. With another Zinke on the card the sheet says it may go.
   */
  private applyRandomMarke(view: RunView, action: RunAction, id: number, marken: string, back: Overlay): void {
    const card = view.edition.find((c) => c.id === id);
    if (!card) return;
    const old = card.mods.find((m) => modKind(m) !== 'Special' && modKind(m) !== 'SpecialCurse') ?? null;
    const blank = { card, after: null, mod: null, from: '', to: '' };
    const ask: ModQuestion = old ? { ...blank, type: 'random', old } : { ...blank, type: 'randomMarke', old: null, marken };
    this.ui.overlay = { kind: 'mod-confirm', action, back, ask };
    this.render();
  }

  /**
   * `always` (a card picker): with nothing to warn about, still show the
   * card before and after, from the core's preview, and wait for
   * "Anbringen" (or "Reinigen").
   */
  private modQuestion(card: DeckCard, action: RunAction, mod: string | null, random: boolean, always: 'apply' | 'cleanse' | null = null): ModQuestion | null {
    const blank = { card, after: null, old: null, mod: null, from: '', to: '' };
    if (random) {
      const old = card.mods.find((m) => modKind(m) !== 'Special' && modKind(m) !== 'SpecialCurse');
      return old ? { ...blank, type: 'random', old } : null;
    }
    let step: Step;
    try {
      step = this.core!.preview(action);
    } catch {
      // The core refuses it: applying shows its error.
      return null;
    }
    const after = step.view.edition.find((c) => c.id === card.id) ?? null;
    // Only a card the step changes is shown (the Kartenmaler's offers open his own sheet).
    const changed = after !== null && JSON.stringify(after) !== JSON.stringify(card);
    if (always === 'cleanse' && changed) return { ...blank, after, type: 'cleanse' };
    const plain = always && changed ? { ...blank, after, type: always, mod } : null;
    for (const ev of step.events) {
      if (typeof ev !== 'object' || !('DeckChanged' in ev)) continue;
      const change = ev.DeckChanged;
      if ('Modified' in change && change.Modified.card === card.id) {
        const m = change.Modified;
        if (m.replaced) return { ...blank, after, type: 'replace', old: m.replaced, mod: m.card_mod };
        if (card.mods.includes(m.card_mod)) {
          // The same again raises it; another (Ramponiert on Augen) is netted with it.
          const values = { from: modValueText(m.card_mod, card.mod_values), to: modValueText(m.card_mod, after?.mod_values ?? {}) };
          if (mod && mod !== m.card_mod) return { ...blank, ...values, after, type: 'change', old: m.card_mod, mod };
          return { ...blank, ...values, after, type: 'raise', mod: m.card_mod };
        }
        return plain ? { ...plain, mod: m.card_mod } : null;
      }
      if ('Unmodified' in change && change.Unmodified.card === card.id) {
        return { ...blank, after, type: 'drop', old: change.Unmodified.card_mod, mod };
      }
    }
    return plain;
  }

  /**
   * A drag ended (./ui/drag.ts): the deck in the middle mixes in, a card
   * goes between hand and table. Onto a card on the table it takes that
   * card's place. Only which two lie there is decided here; the core checks
   * the pair on Drücken.
   */
  private onDrop(el: HTMLElement, zone: HTMLElement | null, over: Element | null, at: DOMRect | null, fling: Fling): void {
    if (this.ui.busy || !this.ui.view?.game) return;
    const kind = el.dataset.drag;
    const where = zone?.dataset.drop;
    if (kind === 'play') {
      this.onThrow(el, where === 'table', at, fling);
      return;
    }
    if (kind === 'deck') {
      if (where === 'mix' && this.ui.view.game.declare?.can_redeal) {
            void this.act('Redeal');
      }
      return;
    }
    if (kind !== 'card' || this.ui.view.game.phase !== 'Discarding') return;
    const id = Number(el.dataset.id ?? '-1');
    const sel = this.ui.selected;
    if (where === 'hand' && sel.includes(id)) {
      this.moveCards(
        sel.filter((x) => x !== id),
        id,
        at,
      );
      return;
    }
    if (where !== 'skat' || sel.includes(id)) return;
    const target = Number(over?.closest<HTMLElement>('.skat-card')?.dataset.id ?? '-1');
    const need = discardNeed(this.ui.view.game);
    if (sel.includes(target)) this.moveCards(sel.map((x) => (x === target ? id : x)), id, at, target);
    else if (sel.length < need) this.moveCards([...sel, id], id, at);
    else this.moveCards([...sel.slice(1), id], id, at);
  }

  /**
   * A hand card thrown while it is the player's turn: it counts when let
   * go over the felt or flicked hard upwards. Whether the card may be
   * played the core said (`data-legal` from the view); one that may not
   * springs back into the hand and shakes. Otherwise it goes back quietly.
   */
  private onThrow(el: HTMLElement, overFelt: boolean, at: DOMRect | null, fling: Fling): void {
    const game = this.ui.view?.game;
    const upward = fling.vy < -FLICK_UP && Math.abs(fling.vx) < -fling.vy * 1.2;
    if (!game?.your_turn || !(overFelt || upward)) {
      springBack(el, at, false);
      return;
    }
    if (el.dataset.legal !== 'true') {
      springBack(el, at, true);
      sound.play('card', { volume: 0.35, rate: 0.7 });
      this.showToast(de.table.notPlayable, 'top');
      return;
    }
    this.playCard(Number(el.dataset.id ?? '-1'), at, fling);
  }

  /** Play a hand card (tap or throw). */
  private playCard(id: number, from: DOMRect | null, fling: Fling | null): void {
    this.playedFrom = from;
    this.thrown = fling;
    void this.act({ Play: id });
  }

  /**
   * A card lands on the felt (cosmetic): an opponent's flies in from his
   * portrait turning over from his back; the player's from the hand, or,
   * thrown, as far as the flick sent it across the felt. Each comes down a
   * little askew, lies there for a moment and slides onto its place.
   */
  private throwOnto(el: HTMLElement, seat: Seat, from: DOMRect, back: string | null): void {
    const side = seat === 1 ? -1 : seat === 2 ? 1 : Math.random() < 0.5 ? -1 : 1;
    if (seat !== 0) {
      throwCard(el, {
        from,
        spin: side * (200 + Math.random() * 220),
        flightMs: 340,
        holdMs: 340,
        settleMs: 260,
        ...(back ? { back: cardBack('', backDesign(back)) } : {}),
      });
      return;
    }
    const fling = this.thrown;
    if (!fling) {
      throwCard(el, { from, spin: side * (90 + Math.random() * 120), flightMs: 260, holdMs: 320, settleMs: 240 });
      return;
    }
    // Thrown: the faster the flick, the further and quicker it flies.
    // It comes down on the felt around the trick, never off it.
    const row = this.rectOf('.trick-row');
    const felt = row ? new DOMRect(row.left, row.top - 24, row.width, row.height + 36) : null;
    const speed = Math.min(4, Math.hypot(fling.vx, fling.vy));
    const sx = from.left + from.width / 2;
    const sy = from.top + from.height / 2;
    const reach = Math.min(1, speed / 2.5) * 260;
    let land = { x: sx + (speed ? (fling.vx / speed) * reach : 0), y: sy + (speed ? (fling.vy / speed) * reach : 0) };
    if (felt) {
      const card = el.getBoundingClientRect();
      const mx = card.width / 2;
      const my = card.height / 2;
      land = { x: Math.max(felt.left + mx, Math.min(felt.right - mx, land.x)), y: Math.max(felt.top + my, Math.min(felt.bottom - my, land.y)) };
    }
    const way = Math.hypot(land.x - sx, land.y - sy);
    const slot = el.getBoundingClientRect();
    const home = Math.hypot(slot.left + slot.width / 2 - land.x, slot.top + slot.height / 2 - land.y);
    throwCard(el, {
      from,
      fromRotate: fling.tilt,
      land,
      spin: (fling.vx === 0 ? side : Math.sign(fling.vx)) * (40 + speed * 150),
      flightMs: Math.max(140, Math.min(440, way / Math.max(0.7, speed))),
      holdMs: 300,
      settleMs: Math.max(220, Math.min(520, 200 + home * 0.9)),
    });
  }

  /**
   * Lay out the cards to press anew. A card changing sides is lifted and
   * flies in a curve to its new place (into its sorted slot in the hand, or
   * onto the table); the other cards glide aside to make room (FLIP). Each
   * starts where it is seen, so taps in quick succession pick the cards up
   * mid-air and nothing waits for a flight to end. `from`: where a dragged
   * card was let go.
   */
  private moveCards(selected: number[], moved: number, from: DOMRect | null, swapped: number | null = null): void {
    const before = spotsOf(this.screenEl, PRESS_CARDS);
    if (from) before.set(String(moved), spotOfRect(from));
    const was = new Set(this.ui.selected);
    const now = new Set(selected);
    const changing = new Set([moved, ...(swapped === null ? [] : [swapped]), ...[...was].filter((x) => !now.has(x)), ...selected.filter((x) => !was.has(x))].map(String));
    this.ui.selected = selected;
    this.render();
    const els = Array.from(this.screenEl.querySelectorAll<HTMLElement>(PRESS_CARDS));
    let n = 0;
    for (const el of els) {
      const id = el.dataset.id ?? '';
      const at = before.get(id);
      if (!changing.has(id) || !at) continue;
      // Into the hand it curves one way, onto the table the other.
      const toHand = el.classList.contains('hand-card');
      glide(el, at, { ...PICK, arc: (toHand ? -1 : 1) * PICK.arc, delay: n++ * 60, above: true });
    }
    glideAll(els, before, { ms: MAKE_ROOM_MS, skip: changing });
    sound.play('card', { volume: 0.5, rate: 1.05 });
  }

  // -------------------------------------------------------------------------
  // Rendering
  // -------------------------------------------------------------------------

  private render(): void {
    const ui = this.ui;
    // A waiting explanation of a new special card, once the table is quiet and no other sheet is open.
    if (this.specialQueue.length && !ui.overlay && !ui.busy && !ui.anim && ui.screen === 'run') {
      ui.overlay = { kind: 'special', ids: this.specialQueue.splice(0) };
    }
    // The player's edition shows his character's back (DESIGN 9.3).
    // A back earned in the long night replaces it if chosen (ADR 0020).
    const chosenBack = ui.settings.back && ui.meta?.card_backs?.includes(ui.settings.back) ? ui.settings.back : null;
    setPlayerBack(chosenBack ?? (ui.screen === 'end' ? ui.ended?.view : ui.view)?.character.back);
    let body = '';
    switch (ui.screen) {
      case 'loading':
        body = renderLoading();
        break;
      case 'error':
        body = renderError();
        break;
      case 'start':
        body = renderStart(ui, BUILD_INFO);
        break;
      case 'end':
        body = ui.ended ? renderEnd(ui.ended.won, ui.ended.view, ui) : '';
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
        const main = inShop
          ? renderShop(view)
          : inEncounter
            ? renderEncounter(view)
            : inReward
              ? renderBossReward(view, ui.rewardPick)
              : inCardReward
                ? renderCardReward(view)
                : inRoute
                ? renderRoute(view, ui.routePick ?? defaultPick(view), ui.routeOverview)
                : renderTable(view, ui);
        const cls = inShop ? 'is-shop' : inEncounter || inReward || inCardReward ? 'is-encounter' : inRoute ? 'is-route' : 'is-table';
        body = `<div class="run ${cls} ${ui.busy ? 'is-busy' : ''} ${view.night ? 'is-night' : ''}">${renderHud(view)}${main}</div>`;
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
        if (kontra) body += renderKontra(view);
        // "Untot": the card waits in the trick for three cards of the hand.
        const rise = view.game?.rise_choice;
        if (rise && !ui.busy && !ui.overlay && !kontra) body += renderRiseChoice(view, ui.risePick);
        if (!rise && ui.risePick.length) ui.risePick = [];
        break;
      }
    }
    // Keep the typed seed and the picker row across re-renders of the start screen.
    const seed = this.root.querySelector<HTMLInputElement>('#seed')?.value;
    const charScroll = this.root.querySelector<HTMLElement>('[data-testid="characters"]')?.scrollLeft;
    // The Tresen stays where it was scrolled to after buying or selling.
    const shopScroll = this.screenEl.querySelector<HTMLElement>('.run > .shop')?.scrollTop ?? 0;
    // Cards flying onto the felt keep flying through the re-render.
    const flights = noteFlights(this.screenEl, FLYING);
    // A sheet on the screen that is drawn again (the Untot pick) does not enter again.
    const riseShown = this.screenEl.querySelector('.sheet-rise') !== null;
    this.screenEl.innerHTML = body;
    if (riseShown) this.screenEl.querySelector('.sheet-rise')?.closest('.overlay')?.classList.add('is-still');
    if (flights.length) resumeFlights(this.screenEl, FLYING, flights);
    if (seed) {
      const input = this.root.querySelector<HTMLInputElement>('#seed');
      if (input) input.value = seed;
    }
    const shop = this.screenEl.querySelector<HTMLElement>('.run > .shop');
    if (shop && shopScroll) shop.scrollTop = shopScroll;
    if (ui.screen === 'start') settleCharacterPicker(this.screenEl, charScroll);
    this.renderLayer();
    // The score board and the boss take the screen: no info card, no trigger lines over them.
    const big = ui.overlay?.kind === 'score' || ui.overlay?.kind === 'boss';
    if (big) {
      this.info.close();
      this.triggers.clear();
    } else {
      this.info.refresh();
    }
    this.updateHint();
    music.setScene(this.musicScene());
    watchShop(this.screenEl);
    const effects = this.afterRender;
    this.afterRender = [];
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
    this.flyPressed();
    if (ui.screen !== 'run') this.entrancePending = false;
    if (this.entrancePending && !ui.overlay && !ui.anim) {
      this.entrancePending = false;
      if (ui.view?.series.is_boss && ui.view.phase === 'Game') bossEntrance(this.screenEl);
    }
  }

  /**
   * A button of a dialog that is not the one in force (a board left on the
   * layer): draw the layer and the screen anew from the state, so nothing
   * stale stays under the finger.
   */
  private redrawLayer(): void {
    this.overlayDirty = true;
    this.render();
  }

  /**
   * "Weiter" of the score board, "Los" of the boss: resolve the sheet the
   * playback waits on. A sheet nothing waits on any more (left on the
   * layer by a playback that broke off) goes, and the screen is drawn
   * anew from the state, so the button never does nothing.
   */
  private closeAwaited(kind: 'score' | 'boss'): void {
    const o = this.ui.overlay;
    if (o?.kind === kind && this.awaited === o) {
      o.resolve();
      return;
    }
    if (o?.kind === 'score' || o?.kind === 'boss') this.ui.overlay = null;
    this.redrawLayer();
  }

  /** The dialog layer: only rebuilt when the dialog itself changes. */
  private renderLayer(): void {
    const o = this.ui.overlay;
    if (o === this.renderedOverlay && !this.overlayDirty) return;
    const same = o !== null && o === this.renderedOverlay;
    this.renderedOverlay = o;
    this.overlayDirty = false;
    this.layerEl.classList.toggle('is-static', same);
    if (!o) {
      this.layerEl.innerHTML = '';
      this.scoreBoard = null;
      return;
    }
    if (o.kind === 'score') {
      this.layerEl.innerHTML = renderScore(o, (this.ui.view?.edition ?? []).map((c) => editionLook(c, this.ui.view, this.ui.wildSettled)));
      const board = new ScoreBoard(this.layerEl, o, (coins, from) => this.hudFx.gainTo(coins, from));
      this.scoreBoard = board;
      // Whatever breaks in the show, the board ends with its "Weiter" button.
      board.play().catch((err: unknown) => {
        console.warn('score board', err);
        board.skip();
      });
      return;
    }
    const view = this.ui.screen === 'run' || this.ui.screen === 'end' ? this.ui.view : null;
    this.layerEl.innerHTML = renderOverlay(o, view, this.ui);
  }
}

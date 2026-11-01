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

import { sound } from './audio';
import { CoreMeta, CoreRun, loadCatalog, loadCore } from './core';
import { de, cardName, contentName, contentText, setCatalog, t } from './i18n';
import { loadSettings, saveSettings, TEMPO_FACTOR, type Tempo } from './settings';
import { clearShareHash, dailySeed, isSeedText, parseShareHash } from './seed';
import {
  addToHistory,
  backupMeta,
  clearRun,
  loadCharacter,
  loadHistory,
  loadMeta,
  loadRun,
  loadRunInfo,
  saveCharacter,
  saveMeta,
  saveRun,
  saveRunInfo,
} from './storage';
import type { CardView, Character, DealtCard, NullRisk, RiskLevel, RunAction, RunEvent, RunView, Seat, Source, Step } from './types';
import { esc, reducedMotion } from './ui/dom';
import { renderEncounter } from './ui/encounter';
import { deal, finishAll, flyAway, flyIn, motionOn, pulse, setTempo } from './ui/fx';
import { coinIcon, renderHud } from './ui/hud';
import { renderScore, ScoreBoard, sourceName } from './ui/score';
import { renderEnd, renderError, renderKontra, renderLoading, renderOverlay, renderStart } from './ui/screens';
import { renderShop, watchShop } from './ui/shop';
import type { Overlay, PendingStart, ScoreOverlay, UiState } from './ui/state';
import { renderTable } from './ui/table';

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

/** Turn a core error into a German text. */
function errorText(err: unknown): string {
  const code = errorCode(err);
  const fallback = de.app.errors.default ?? '';
  return code ? (de.app.errors[code] ?? fallback) : fallback;
}

function isCharacter(value: string | null): value is Character {
  return value === 'Declarer' || value === 'NullPlayer';
}

/** Pauses in milliseconds at tempo "Normal"; reduced motion shortens them. */
const DELAY = {
  ownCard: 260,
  aiCard: 560,
  trickShow: 650,
  trickFly: 360,
  toast: 1900,
  notice: 700,
};

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
    handGame: false,
    pickGame: null,
    pickRisk: null,
    pickNullRisk: null,
    omaPick: false,
    swapTake: null,
    thinking: null,
    shared: null,
    copy: null,
    history: loadHistory(),
    pickCharacter: (() => {
      const stored = loadCharacter();
      return isCharacter(stored) ? stored : 'Declarer';
    })(),
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
  private toasts: string[] = [];
  private readonly screenEl: HTMLElement;
  private readonly layerEl: HTMLElement;
  private readonly toastEl: HTMLElement;
  private renderedOverlay: Overlay | null = null;
  private overlayDirty = false;
  private scoreBoard: ScoreBoard | null = null;
  /** Effects to run once the next render is in the DOM. */
  private afterRender: (() => void)[] = [];
  /** Where the card the player just tapped was. */
  private playedFrom: DOMRect | null = null;

  constructor(private readonly root: HTMLElement) {
    root.innerHTML = `<div class="screen"></div><div class="layer"></div><div class="toast-layer" role="status" aria-live="polite"></div>`;
    this.screenEl = root.querySelector<HTMLElement>('.screen') ?? root;
    this.layerEl = root.querySelector<HTMLElement>('.layer') ?? root;
    this.toastEl = root.querySelector<HTMLElement>('.toast-layer') ?? root;
    this.applySettings();
    root.addEventListener('click', (e) => this.onClick(e));
    // Browsers allow audio only after a user gesture.
    const unlock = (): void => sound.unlock();
    window.addEventListener('pointerdown', unlock, { passive: true });
    window.addEventListener('keydown', unlock);
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
        const character = contentName('characters', view.character.character);
        this.ui.saveInfo =
          view.phase === 'Encounter'
            ? t(de.start.resumeEncounter, { character, section: view.series.section + 1 })
            : t(de.start.resumeInfo, {
                character,
                section: view.series.section + 1,
                series: view.series.index + 1,
                score: view.series.score,
                target: view.series.target,
              });
      } catch (err) {
        console.warn('saved run discarded', err);
        clearRun();
        this.core = null;
        this.ui.message = de.start.resumeFailed;
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
    this.requestStart({ seed, character: this.ui.pickCharacter, daily: false });
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

  private startRun({ seed, character, daily }: PendingStart): void {
    let core: CoreRun;
    let startEvents = '[]';
    try {
      if (this.meta) {
        const started = CoreRun.createWith(seed, character, this.meta);
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

  private resume(): void {
    if (!this.core) return;
    this.ui.message = null;
    this.enterRun(this.core.view(), true);
    // Saved while an opponent was to move: let him play on.
    this.continueOpponents();
  }

  private enterRun(view: RunView, dealt: boolean): void {
    this.ui.view = view;
    this.ui.screen = 'run';
    this.ui.anim = null;
    this.ui.overlay = null;
    this.resetTableChoices();
    if (dealt && view.game) this.afterRender.push(() => this.dealEffect());
    this.render();
  }

  private resetTableChoices(): void {
    this.ui.selected = [];
    this.ui.handGame = false;
    this.ui.pickGame = null;
    this.ui.pickRisk = null;
    this.ui.pickNullRisk = null;
    this.ui.omaPick = false;
    this.ui.swapTake = null;
  }

  // -------------------------------------------------------------------------
  // Actions
  // -------------------------------------------------------------------------

  /** Send an action to the core, then play its events and the opponents' cards back. */
  private async act(action: RunAction): Promise<void> {
    if (!this.core || this.ui.busy) return;
    const before = this.ui.view;
    this.ui.busy = true;
    let step;
    try {
      step = this.core.apply(action);
    } catch (err) {
      console.warn('action rejected', action, err);
      this.ui.busy = false;
      this.render();
      this.showToast(t(de.app.actionFailed, { reason: errorText(err) }));
      return;
    }
    this.fastForward = false;
    this.ui.overlay = null;
    await this.playSteps(before, step);
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
          this.recordRun(view);
        } else if (this.core) {
          saveRun(this.core.save());
        }
        earned = earned.concat(this.observe(step.events));
        await this.playEvents(previous, step.events, view);
        // The table as the core has it now, before the next opponent thinks.
        this.ui.anim = null;
        this.ui.view = view;
        this.playedFrom = null;
        previous = view;
        step = null;
      }
    } finally {
      this.ui.busy = false;
      this.ui.anim = null;
      this.ui.thinking = null;
      if (view) this.ui.view = view;
      this.playedFrom = null;
      this.resetTableChoices();
      if (view && (view.phase === 'Won' || view.phase === 'Lost')) {
        this.ui.ended = { won: view.phase === 'Won', view };
        this.ui.screen = 'end';
        this.ui.copy = null;
        sound.play(view.phase === 'Won' ? 'win' : 'lose', { delay: 200 });
      }
      this.announceUnlocks(earned);
      this.render();
    }
  }

  private cardOf(card: DealtCard, view: RunView): CardView {
    const mods = view.deck.find((d) => d.id === card.id)?.mods ?? [];
    return { ...card, mods, legal: false, from_skat: false, discardable: false };
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
    for (let i = 0; i < events.length; i++) {
      const ev = events[i];
      if (ev === undefined) continue;
      if (typeof ev === 'string') {
        if (ev === 'Re') this.showToast(de.toast.re);
        if (ev === 'Redealt') {
          this.showToast(de.toast.redealt);
          this.afterRender.push(() => this.dealEffect());
        }
        if (ev === 'RamschDealt') {
          this.showToast(de.toast.ramsch);
          sound.play('knock');
        }
        if (ev === 'EveningRulesEnded') this.showToast(de.toast.eveningEnded);
        continue;
      }
      if ('CardPeeked' in ev) {
        const { card, seat } = ev.CardPeeked;
        this.showToast(t(de.toast.peeked, { name: this.seatLabel(seat, after), card: cardName(card.kind) }));
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
      if ('DeckChanged' in ev) {
        const change = ev.DeckChanged;
        if ('Modified' in change) {
          const card = after.deck.find((c) => c.id === change.Modified.card);
          if (card) this.showToast(t(de.toast.cardMarked, { card: cardName(card.kind), mod: contentName('card_mods', change.Modified.card_mod) }));
        }
        continue;
      }
      if ('Game' in ev) {
        await this.playGameEvent(ev.Game, after);
        continue;
      }
      if ('CoinsGained' in ev) {
        await this.coinsGained(ev.CoinsGained.source, ev.CoinsGained.amount, ev.CoinsGained.coins, after);
        continue;
      }
      if ('GameDealt' in ev) {
        // A new game: from here on the final view tells the story.
        this.ui.anim = null;
        this.ui.view = after;
        this.resetTableChoices();
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
      if ('Passed' in ev) {
        this.showToast(t(de.toast.passed, { left: ev.Passed.passes_left }));
        continue;
      }
      if ('PassesAdded' in ev) {
        this.showToast(t(de.toast.passesAdded, { left: ev.PassesAdded.passes_left }));
        continue;
      }
      if ('ConsumableUsed' in ev) {
        this.showToast(t(de.toast.consumableUsed, { name: contentName('consumables', ev.ConsumableUsed.consumable) }));
        continue;
      }
      if ('RelicBought' in ev) {
        this.showToast(t(de.toast.bought, { name: contentName('relics', ev.RelicBought.relic) }));
        sound.play('coin');
        continue;
      }
      if ('ConsumableBought' in ev) {
        this.showToast(t(de.toast.bought, { name: contentName('consumables', ev.ConsumableBought.consumable) }));
        sound.play('coin');
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
      if ('GameScored' in ev) {
        const rest = events.slice(i + 1);
        const won = rest.find((e): e is Extract<RunEvent, { SeriesWon: unknown }> => typeof e === 'object' && 'SeriesWon' in e);
        await this.showScore(
          before?.series.score ?? 0,
          ev.GameScored.series_score,
          before?.series.target ?? after.series.target,
          after,
          won ? { payout: won.SeriesWon.payout, coins: won.SeriesWon.coins } : null,
        );
        continue;
      }
      if ('SeriesStarted' in ev && ev.SeriesStarted.boss && after.series.boss) {
        this.ui.view = after;
        this.ui.anim = null;
        this.ui.screen = 'run';
        const boss = after.series.boss;
        await new Promise<void>((resolve) => {
          this.ui.overlay = { kind: 'boss', boss, resolve };
          this.render();
        });
        this.ui.overlay = null;
      }
    }
  }

  private async playGameEvent(ev: Extract<RunEvent, { Game: unknown }>['Game'], after: RunView): Promise<void> {
    const anim = this.ui.anim;
    if ('CardPlayed' in ev) {
      if (!anim) return;
      const { seat, card } = ev.CardPlayed;
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
      if (el && from && !this.fastForward) flyIn(el, from, { ms: seat === 0 ? 220 : 300, rotate: seat === 1 ? -20 : seat === 2 ? 20 : 0 });
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
      this.render();
      await this.pause(DELAY.trickShow);
      const target = this.winnerRect(trick.winner);
      const cards = Array.from(this.screenEl.querySelectorAll('.trick-card'));
      sound.play('trick');
      if (target && !this.fastForward) await Promise.race([flyAway(cards, target, DELAY.trickFly), this.pause(DELAY.trickFly + 200)]);
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
      if (g && 'SkatPickedUp' in ev) {
        anim.hand = g.hand.map((c) => ({ ...c, legal: false }));
        anim.handSizes[0] = anim.hand.length;
      }
      if (g && this.ui.view?.game) {
        this.ui.view = { ...this.ui.view, game: { ...this.ui.view.game, declaration: g.declaration, phase: g.phase, kontra: g.kontra } };
      }
      this.render();
      if ('SkatPickedUp' in ev) {
        // The two Skat cards slide from the middle into the hand.
        const from = this.rectOf('.deck-spot');
        if (from) {
          for (const c of ev.SkatPickedUp.skat) {
            const el = this.screenEl.querySelector(`.hand-card[data-id="${c.id}"]`);
            if (el) flyIn(el, from, { ms: 320 });
          }
          sound.play('card');
        }
      }
    }
  }

  /**
   * Groschen paid by content (a curse when it is attached): a coin pops up
   * at the purse with the amount and where it came from, the purse shows
   * the new total from the core.
   */
  private async coinsGained(source: Source, amount: number, coins: number, after: RunView): Promise<void> {
    const from = sourceName(source, after.deck);
    const sign = amount > 0 ? `+${amount}` : `−${Math.abs(amount)}`;
    this.showToast(t(de.toast.coins, { amount: sign, source: from }));
    sound.play('coin');
    const purse = this.screenEl.querySelector<HTMLElement>('[data-testid="coins"]');
    if (purse) {
      purse.textContent = String(coins);
      pulse(purse.closest('.hud-coins') ?? purse, 1.3, 360);
    }
    const box = purse?.closest('.hud-coins')?.getBoundingClientRect();
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

  private seatLabel(seat: Seat, view: RunView): string {
    if (seat === 1 && view.series.is_boss && view.series.boss) return contentName('bosses', view.series.boss.id);
    return de.seats[seat] ?? '';
  }

  private rectOf(selector: string): DOMRect | null {
    const el = this.screenEl.querySelector(selector);
    return el ? el.getBoundingClientRect() : null;
  }

  private winnerRect(seat: Seat): DOMRect | null {
    if (seat === 0) return this.rectOf('[data-tricks]') ?? this.rectOf('.hand');
    return this.rectOf(`.opp[data-seat="${seat}"] .portrait`);
  }

  /** Cards fly from the middle of the table to the hands, in turns. */
  private dealEffect(): void {
    const from = this.rectOf('.deck-spot');
    if (!from) return;
    const hand = Array.from(this.screenEl.querySelectorAll('.hand-card'));
    const left = Array.from(this.screenEl.querySelectorAll('.opp[data-seat="1"] .opp-hand .card-back'));
    const right = Array.from(this.screenEl.querySelectorAll('.opp[data-seat="2"] .opp-hand .card-back'));
    const order: Element[] = [];
    for (let i = 0; i < Math.max(hand.length, left.length, right.length); i++) {
      for (const list of [hand, left, right]) {
        const el = list[i];
        if (el) order.push(el);
      }
    }
    deal(order, from, {
      step: 22,
      ms: 280,
      onCard: (i) => {
        if (i % 4 === 0) sound.play('card', { volume: 0.45, rate: 1.1 });
      },
    });
  }

  private async showScore(
    previousScore: number,
    seriesScore: number,
    target: number,
    after: RunView,
    payout: ScoreOverlay['payout'],
  ): Promise<void> {
    const score = after.last_score;
    if (!score) return;
    await new Promise<void>((resolve) => {
      const seatNames = ([0, 1, 2] as const).map((seat) => this.seatLabel(seat, after));
      this.ui.overlay = { kind: 'score', score, previousScore, seriesScore, target, payout, relics: after.relics, seatNames, resolve };
      this.render();
    });
    this.scoreBoard = null;
    this.ui.overlay = null;
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
  private showToast(text: string): void {
    if (this.toastTimer !== undefined) {
      this.toasts.push(text);
      if (this.toasts.length > TOAST_QUEUE) this.toasts.shift();
      return;
    }
    this.toastEl.innerHTML = `<div class="toast">${esc(text)}</div>`;
    // Shorter while others wait.
    const ms = this.toasts.length ? DELAY.toast * 0.7 : DELAY.toast;
    this.toastTimer = window.setTimeout(() => {
      this.toastTimer = undefined;
      this.toastEl.innerHTML = '';
      const next = this.toasts.shift();
      if (next !== undefined) this.showToast(next);
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
      case 'resume':
        this.resume();
        return;
      case 'play-shared': {
        const shared = this.ui.shared;
        if (shared) this.requestStart({ seed: shared.seed, character: shared.locked ? 'Declarer' : shared.character, daily: false });
        return;
      }
      case 'dismiss-shared':
        this.ui.shared = null;
        clearShareHash();
        this.render();
        return;
      case 'play-daily':
        this.requestStart({ seed: dailySeed(), character: this.ui.pickCharacter, daily: true });
        return;
      case 'replay': {
        const c = el.dataset.character ?? null;
        const seed = el.dataset.seed ?? '';
        const entry = this.ui.meta?.characters.find((x) => x.character === c);
        if (isCharacter(c) && entry?.unlocked && isSeedText(seed)) this.requestStart({ seed, character: c, daily: false });
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
      case 'credits':
        this.ui.overlay = { kind: 'credits' };
        this.render();
        return;
      case 'book':
        if (this.ui.busy || !this.ui.meta) return;
        this.ui.overlay = { kind: 'book', tab: 'book' };
        this.render();
        return;
      case 'pick-character': {
        const c = el.dataset.character ?? null;
        const entry = this.ui.meta?.characters.find((x) => x.character === c);
        if (isCharacter(c) && entry?.unlocked) {
          this.ui.pickCharacter = c;
          saveCharacter(c);
          this.render();
        }
        return;
      }
      case 'info-rule': {
        const rule = view?.game?.boss_rule;
        if (rule) {
          this.ui.overlay = { kind: 'info', title: contentName('bosses', rule.id), subtitle: de.table.bossRule, text: contentText('bosses', rule.id) || rule.text };
          this.render();
        }
        return;
      }
      case 'info-evening': {
        const relic = view?.evening_relics[num('index')];
        if (relic) {
          this.ui.overlay = {
            kind: 'info',
            title: contentName('relics', relic.id),
            subtitle: de.evening.subtitle,
            text: contentText('relics', relic.id),
            coaster: relic.rarity,
          };
          this.render();
        }
        return;
      }
      case 'tempo':
        this.setSettings({ tempo: (el.dataset.tempo ?? 'normal') as Tempo });
        return;
      case 'sound':
        this.setSettings({ sound: el.dataset.on === 'true' });
        if (el.dataset.on === 'true') sound.play('clink');
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
      case 'score-skip':
        if (overlay?.kind === 'score') this.scoreBoard?.skip();
        return;
      case 'score-continue':
        if (overlay?.kind === 'score') overlay.resolve();
        return;
      case 'boss-go':
        if (overlay?.kind === 'boss') overlay.resolve();
        return;
      case 'info-boss':
        if (view?.series.boss) {
          const boss = view.series.boss;
          this.ui.overlay = { kind: 'info', title: contentName('bosses', boss.id), subtitle: de.boss.title, text: contentText('bosses', boss.id) || boss.text };
          this.render();
        }
        return;
      case 'info-relic': {
        const relic = view?.relics.find((r) => r.index === num('index'));
        if (relic) {
          this.ui.overlay = {
            kind: 'info',
            title: contentName('relics', relic.relic.id),
            subtitle: `${de.relicKind[relic.relic.kind]} · ${de.rarity[relic.relic.rarity]}`,
            text: contentText('relics', relic.relic.id),
            coaster: relic.relic.rarity,
          };
          this.render();
        }
        return;
      }
      case 'info-offer-relic': {
        const offer = view?.shop?.relics.find((o) => o?.slot === num('slot'));
        if (offer) {
          this.ui.overlay = {
            kind: 'info',
            title: contentName('relics', offer.relic.id),
            subtitle: `${de.relicKind[offer.relic.kind]} · ${de.rarity[offer.relic.rarity]}`,
            text: contentText('relics', offer.relic.id),
            coaster: offer.relic.rarity,
            ...(offer.can_buy && !this.ui.busy ? { buy: { action: 'buy-relic', slot: offer.slot, price: offer.price } } : {}),
          };
          this.render();
        }
        return;
      }
      case 'info-offer-item': {
        const offer = view?.shop?.consumables.find((o) => o?.slot === num('slot'));
        if (offer) {
          this.ui.overlay = {
            kind: 'info',
            title: contentName('consumables', offer.consumable.id),
            subtitle: de.shop.consumables,
            text: contentText('consumables', offer.consumable.id),
            good: offer.consumable.id,
            odds: offer.consumable.needs_target,
            ...(offer.can_buy && !this.ui.busy ? { buy: { action: 'buy-item', slot: offer.slot, price: offer.price } } : {}),
          };
          this.render();
        }
        return;
      }
      case 'scroll-goods':
        this.root.querySelector('#shop-goods')?.scrollIntoView({ behavior: motionOn() ? 'smooth' : 'auto', block: 'start' });
        return;
      case 'info-item': {
        const item = view?.consumables.find((c) => c.slot === num('slot'));
        if (item) {
          this.ui.overlay = {
            kind: 'info',
            title: contentName('consumables', item.consumable.id),
            subtitle: de.shop.consumables,
            text: contentText('consumables', item.consumable.id),
            good: item.consumable.id,
            ...(item.can_use && !this.ui.busy ? { use: item.slot } : {}),
          };
          this.render();
        }
        return;
      }
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
      case 'choose-encounter': {
        const option = view.encounter?.options.find((o) => o.index === num('option'));
        if (!option) return;
        if (option.needs_target) {
          this.ui.overlay = { kind: 'encounter-target', option: option.index };
          this.render();
        } else {
          void this.act({ ChooseEncounter: { option: option.index, target: null } });
        }
        return;
      }
      case 'encounter-target': {
        const o = this.ui.overlay;
        if (o?.kind === 'encounter-target') void this.act({ ChooseEncounter: { option: o.option, target: num('id') } });
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
        void this.act('Pass');
        return;
      case 'redraw':
        void this.act('Redeal');
        return;
      case 're':
        void this.act({ AnswerKontra: { re: el.dataset.re === 'true' } });
        return;
      case 'select': {
        const id = num('id');
        const sel = this.ui.selected;
        if (sel.includes(id)) this.ui.selected = sel.filter((x) => x !== id);
        else if (sel.length < 2) this.ui.selected = [...sel, id];
        else this.ui.selected = [sel[1] ?? id, id];
        this.render();
        return;
      }
      case 'discard': {
        const [a, b] = this.ui.selected;
        if (a !== undefined && b !== undefined) void this.act({ Discard: [a, b] });
        return;
      }
      case 'play':
        this.playedFrom = el.getBoundingClientRect();
        void this.act({ Play: num('id') });
        return;
      case 'buy-relic':
        void this.act({ BuyRelic: num('slot') });
        return;
      case 'buy-item':
        void this.act({ BuyConsumable: num('slot') });
        return;
      case 'sell-relic':
        void this.act({ SellRelic: num('index') });
        return;
      case 'sell-item':
        void this.act({ SellConsumable: num('slot') });
        return;
      case 'use-item': {
        const slot = num('slot');
        const item = view.consumables.find((c) => c.slot === slot);
        if (!item) return;
        if (item.consumable.needs_target) {
          this.ui.overlay = { kind: 'target', slot };
          this.render();
        } else {
          void this.act({ UseConsumable: { slot, target: null } });
        }
        return;
      }
      case 'target': {
        const o = this.ui.overlay;
        if (o?.kind === 'target') void this.act({ UseConsumable: { slot: o.slot, target: num('id') } });
        return;
      }
      case 'reroll':
        void this.act('Reroll');
        return;
      case 'leave-shop':
        void this.act('LeaveShop');
        return;
    }
  }

  // -------------------------------------------------------------------------
  // Rendering
  // -------------------------------------------------------------------------

  private render(): void {
    const ui = this.ui;
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
        const main = inShop ? renderShop(view) : inEncounter ? renderEncounter(view) : renderTable(view, ui);
        const cls = inShop ? 'is-shop' : inEncounter ? 'is-encounter' : 'is-table';
        body = `<div class="run ${cls} ${ui.busy ? 'is-busy' : ''}">${renderHud(view)}${main}</div>`;
        const kontra = view.game?.awaiting_kontra_answer && !ui.busy && !ui.overlay;
        if (kontra) body += renderKontra(view);
        break;
      }
    }
    // Keep the typed seed across re-renders of the start screen.
    const seed = this.root.querySelector<HTMLInputElement>('#seed')?.value;
    this.screenEl.innerHTML = body;
    if (seed) {
      const input = this.root.querySelector<HTMLInputElement>('#seed');
      if (input) input.value = seed;
    }
    this.renderLayer();
    watchShop(this.screenEl);
    const effects = this.afterRender;
    this.afterRender = [];
    for (const fx of effects) fx();
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
      this.layerEl.innerHTML = renderScore(o, this.ui.view?.deck ?? []);
      this.scoreBoard = new ScoreBoard(this.layerEl, o);
      void this.scoreBoard.play();
      return;
    }
    const view = this.ui.screen === 'run' || this.ui.screen === 'end' ? this.ui.view : null;
    this.layerEl.innerHTML = renderOverlay(o, view, this.ui);
  }
}

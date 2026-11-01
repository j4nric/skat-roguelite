// The app controller: holds the core run and the UI state, turns taps into
// RunActions, plays the core's events back as animations with sound, and
// saves the core state after every action. No game rules live here: what
// is legal, what a game is worth and who takes a trick all come from the
// core.
//
// The DOM has three layers: the screen (re-rendered on every change), the
// overlay (rendered only when the dialog changes, so the score board can
// animate undisturbed) and the toast line.

import { sound } from './audio';
import { CoreRun, loadCatalog, loadCore } from './core';
import { de, contentName, contentText, setCatalog, t } from './i18n';
import { loadSettings, saveSettings, TEMPO_FACTOR, type Tempo } from './settings';
import { clearRun, loadRecord, loadRun, recordRun, saveRun } from './storage';
import type { CardView, DealtCard, RiskLevel, RunAction, RunEvent, RunView, Seat } from './types';
import { esc, reducedMotion } from './ui/dom';
import { deal, finishAll, flyAway, flyIn, setTempo } from './ui/fx';
import { renderHud } from './ui/hud';
import { renderScore, ScoreBoard } from './ui/score';
import { renderEnd, renderError, renderKontra, renderLoading, renderOverlay, renderStart } from './ui/screens';
import { renderShop } from './ui/shop';
import type { Overlay, ScoreOverlay, UiState } from './ui/state';
import { renderTable } from './ui/table';

/** Turn a core error (JSON `{code, message}` since the M2 polish) into a German text. */
function errorText(err: unknown): string {
  const raw = err instanceof Error ? err.message : String(err);
  try {
    const parsed = JSON.parse(raw) as { code?: string };
    const fallback = de.app.errors.default ?? '';
    if (parsed.code) return de.app.errors[parsed.code] ?? fallback;
  } catch {
    /* not JSON: fall through */
  }
  return de.app.errors.default ?? '';
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

const BUILD_INFO = typeof __BUILD_INFO__ === 'string' ? __BUILD_INFO__ : '';

export class App {
  private core: CoreRun | null = null;
  private ui: UiState = {
    screen: 'loading',
    view: null,
    anim: null,
    busy: false,
    selected: [],
    pickGame: null,
    pickRisk: null,
    overlay: null,
    message: null,
    saveInfo: null,
    record: loadRecord(),
    settings: loadSettings(),
    ended: null,
  };
  private fastForward = false;
  private wakers: (() => void)[] = [];
  private toastTimer: number | undefined;
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
    this.toStart();
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
    this.ui.record = loadRecord();
    const saved = loadRun();
    if (saved) {
      try {
        this.core = CoreRun.load(saved);
        const view = this.core.view();
        this.ui.view = view;
        this.ui.saveInfo = t(de.start.resumeInfo, {
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

  private newRun(): void {
    const input = this.root.querySelector<HTMLInputElement>('#seed');
    const typed = input?.value.trim() ?? '';
    let seed = typed;
    if (!seed) {
      const random = new Uint32Array(1);
      crypto.getRandomValues(random);
      seed = String(random[0]);
    }
    let core: CoreRun;
    try {
      core = CoreRun.create(seed);
    } catch {
      this.ui.message = de.start.seedInvalid;
      this.render();
      return;
    }
    this.core?.dispose();
    this.core = core;
    this.ui.message = null;
    this.ui.ended = null;
    saveRun(core.save());
    this.enterRun(core.view(), true);
  }

  private resume(): void {
    if (!this.core) return;
    this.ui.message = null;
    this.enterRun(this.core.view(), true);
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
    this.ui.pickGame = null;
    this.ui.pickRisk = null;
  }

  // -------------------------------------------------------------------------
  // Actions
  // -------------------------------------------------------------------------

  /** Send an action to the core, save, then play the events back. */
  private async act(action: RunAction): Promise<void> {
    if (!this.core || this.ui.busy) return;
    const before = this.ui.view;
    let step;
    try {
      step = this.core.apply(action);
    } catch (err) {
      console.warn('action rejected', action, err);
      this.showToast(t(de.app.actionFailed, { reason: errorText(err) }));
      return;
    }
    const view = step.view;
    if (view.phase === 'Won' || view.phase === 'Lost') {
      clearRun();
      this.ui.record = recordRun(view.phase === 'Won', view.series.section + 1);
    } else {
      saveRun(this.core.save());
    }
    this.ui.busy = true;
    this.fastForward = false;
    this.ui.overlay = null;
    try {
      await this.playEvents(before, step.events, view);
    } finally {
      this.ui.busy = false;
      this.ui.anim = null;
      this.ui.view = view;
      this.playedFrom = null;
      this.resetTableChoices();
      if (view.phase === 'Won' || view.phase === 'Lost') {
        this.ui.ended = { won: view.phase === 'Won', view };
        this.ui.screen = 'end';
        sound.play(view.phase === 'Won' ? 'win' : 'lose', { delay: 200 });
      }
      this.render();
    }
  }

  private cardOf(card: DealtCard, view: RunView): CardView {
    const mods = view.deck.find((d) => d.id === card.id)?.mods ?? [];
    return { ...card, mods, legal: false, from_skat: false };
  }

  /** Animate what happened, using only the events and views of the core. */
  private async playEvents(before: RunView | null, events: RunEvent[], after: RunView): Promise<void> {
    const game = before?.game;
    if (game) {
      this.ui.anim = {
        trick: game.trick.slice(),
        hand: game.hand.map((c) => ({ ...c, legal: false, from_skat: false })),
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
        continue;
      }
      if ('Game' in ev) {
        await this.playGameEvent(ev.Game, after);
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
      this.ui.overlay = { kind: 'score', score, previousScore, seriesScore, target, payout, relics: after.relics, resolve };
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

  private showToast(text: string): void {
    this.toastEl.innerHTML = `<div class="toast">${esc(text)}</div>`;
    window.clearTimeout(this.toastTimer);
    this.toastTimer = window.setTimeout(() => {
      this.toastEl.innerHTML = '';
    }, DELAY.toast);
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
        const main = inShop ? renderShop(view) : renderTable(view, ui);
        body = `<div class="run ${inShop ? 'is-shop' : 'is-table'} ${ui.busy ? 'is-busy' : ''}">${renderHud(view)}${main}</div>`;
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

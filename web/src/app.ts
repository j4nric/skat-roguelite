// The app controller: holds the core run and the UI state, turns taps into
// RunActions, plays the core's events back with short delays and saves the
// core state after every action. No game rules live here: what is legal,
// what a game is worth and who takes a trick all come from the core.

import { CoreRun, loadCatalog, loadCore } from './core';
import { de, contentName, contentText, setCatalog, t } from './i18n';
import { clearRun, loadRecord, loadRun, recordRun, saveRun } from './storage';
import type { CardView, DealtCard, Payout, RiskLevel, RunAction, RunEvent, RunView } from './types';
import { esc, reducedMotion } from './ui/dom';
import { renderHud } from './ui/hud';
import { renderScore } from './ui/score';
import { renderEnd, renderError, renderKontra, renderLoading, renderOverlay, renderStart } from './ui/screens';
import { renderShop } from './ui/shop';
import type { UiState } from './ui/state';
import { renderTable } from './ui/table';

/** Delays in milliseconds; reduced motion shortens them. */
const DELAY = {
  ownCard: 180,
  aiCard: 520,
  trickDone: 850,
  toast: 1800,
  scoreStep: 380,
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
    toast: null,
    message: null,
    saveInfo: null,
    record: loadRecord(),
    ended: null,
  };
  private fastForward = false;
  private wakers: (() => void)[] = [];
  private toastTimer: number | undefined;
  private scoreTimer: number | undefined;

  constructor(private readonly root: HTMLElement) {
    root.addEventListener('click', (e) => this.onClick(e));
    window.addEventListener('keydown', (e) => {
      if (e.key === 'Escape' && this.ui.overlay && this.ui.overlay.kind !== 'score' && this.ui.overlay.kind !== 'boss') {
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
    this.enterRun(core.view());
  }

  private resume(): void {
    if (!this.core) return;
    this.ui.message = null;
    this.enterRun(this.core.view());
  }

  private enterRun(view: RunView): void {
    this.ui.view = view;
    this.ui.screen = 'run';
    this.ui.anim = null;
    this.ui.overlay = null;
    this.resetTableChoices();
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
      this.showToast(t(de.app.actionFailed, { reason: String(err) }));
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
      this.resetTableChoices();
      if (view.phase === 'Won' || view.phase === 'Lost') {
        this.ui.ended = { won: view.phase === 'Won', view };
        this.ui.screen = 'end';
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
        if (ev === 'Redealt') this.showToast(de.toast.redealt);
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
        continue;
      }
      if ('Kontra' in ev) {
        this.showToast(t(de.toast.kontra, { name: de.seats[ev.Kontra.seat] ?? '' }));
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
        continue;
      }
      if ('ConsumableBought' in ev) {
        this.showToast(t(de.toast.bought, { name: contentName('consumables', ev.ConsumableBought.consumable) }));
        continue;
      }
      if ('RelicSold' in ev) {
        this.showToast(t(de.toast.sold, { name: contentName('relics', ev.RelicSold.relic), price: ev.RelicSold.price }));
        continue;
      }
      if ('ConsumableSold' in ev) {
        this.showToast(
          t(de.toast.sold, { name: contentName('consumables', ev.ConsumableSold.consumable), price: ev.ConsumableSold.price }),
        );
        continue;
      }
      if ('ShopRerolled' in ev) {
        this.showToast(de.toast.rerolled);
        continue;
      }
      if ('GameScored' in ev) {
        const rest = events.slice(i + 1);
        const won = rest.find((e): e is Extract<RunEvent, { SeriesWon: unknown }> => typeof e === 'object' && 'SeriesWon' in e);
        await this.showScore(
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
      anim.trick.push({ seat, card: this.cardOf(card, after) });
      anim.handSizes[seat] = Math.max(0, (anim.handSizes[seat] ?? 0) - 1);
      if (seat === 0) anim.hand = anim.hand.filter((c) => c.id !== card.id);
      anim.turn = seat === 0 ? null : seat;
      anim.status = '';
      this.render();
      await this.pause(seat === 0 ? DELAY.ownCard : DELAY.aiCard);
      return;
    }
    if ('TrickWon' in ev) {
      if (!anim) return;
      const { trick, card_points } = ev.TrickWon;
      anim.turn = null;
      anim.status = t(de.table.trickTo, { name: de.seats[trick.winner] ?? '', points: card_points });
      this.render();
      await this.pause(DELAY.trickDone);
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
    if ('SkatRedrawn' in ev) {
      this.showToast(de.toast.redrawn);
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
    }
  }

  private async showScore(
    seriesScore: number,
    target: number,
    after: RunView,
    payout: { payout: Payout; coins: number } | null,
  ): Promise<void> {
    const score = after.last_score;
    if (!score) return;
    await new Promise<void>((resolve) => {
      const instant = reducedMotion() || this.fastForward;
      this.ui.overlay = {
        kind: 'score',
        score,
        seriesScore,
        target,
        payout,
        shown: instant ? score.steps.length : 0,
        resolve,
      };
      this.render();
      this.scheduleScoreStep();
    });
    window.clearTimeout(this.scoreTimer);
    this.ui.overlay = null;
  }

  private scheduleScoreStep(): void {
    const o = this.ui.overlay;
    if (!o || o.kind !== 'score' || o.shown >= o.score.steps.length) return;
    this.scoreTimer = window.setTimeout(() => {
      const cur = this.ui.overlay;
      if (!cur || cur.kind !== 'score') return;
      cur.shown += 1;
      this.render();
      this.scheduleScoreStep();
    }, DELAY.scoreStep);
  }

  /** Wait, unless the player asked to hurry. */
  private pause(ms: number): Promise<void> {
    if (this.fastForward) return Promise.resolve();
    const scaled = reducedMotion() ? ms * 0.25 : ms;
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
    for (const wake of this.wakers.slice()) wake();
  }

  private showToast(text: string): void {
    this.ui.toast = text;
    window.clearTimeout(this.toastTimer);
    this.toastTimer = window.setTimeout(() => {
      this.ui.toast = null;
      this.render();
    }, DELAY.toast);
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
    if ((action === 'close-overlay' || action === 'score-skip') && el.classList.contains('overlay') && target !== el) {
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
        if (this.ui.busy) return;
        this.ui.overlay = { kind: 'menu' };
        this.render();
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
        if (overlay?.kind === 'score') {
          overlay.shown = overlay.score.steps.length;
          window.clearTimeout(this.scoreTimer);
          this.render();
        }
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
          };
          this.render();
        }
        return;
      }
      case 'info-item': {
        const item = view?.consumables.find((c) => c.slot === num('slot'));
        if (item) {
          const usable = item.can_use && !item.consumable.needs_target && !this.ui.busy;
          this.ui.overlay = {
            kind: 'info',
            title: contentName('consumables', item.consumable.id),
            subtitle: de.shop.consumables,
            text: contentText('consumables', item.consumable.id),
            ...(usable || (item.can_use && item.consumable.needs_target) ? { use: item.slot } : {}),
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
        void this.act('RedrawSkat');
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
        body = ui.ended ? renderEnd(ui.ended.won, ui.ended.view) : '';
        break;
      case 'run': {
        const view = ui.view;
        if (!view) break;
        const main = view.phase === 'Shop' && !ui.anim ? renderShop(view) : renderTable(view, ui);
        body = `<div class="run ${ui.busy ? 'is-busy' : ''}">${renderHud(view)}${main}</div>`;
        const kontra = view.game?.awaiting_kontra_answer && !ui.busy && !ui.overlay;
        if (kontra) body += renderKontra(view);
        if (view.phase === 'Shop' && ui.toast && !ui.anim) body += `<div class="toast toast-floating" role="status">${esc(ui.toast)}</div>`;
        break;
      }
    }
    if (ui.overlay) {
      body +=
        ui.overlay.kind === 'score'
          ? renderScore(ui.overlay, ui.view?.deck ?? [])
          : renderOverlay(ui.overlay, ui.view);
    }
    // Keep the typed seed across re-renders of the start screen.
    const seed = this.root.querySelector<HTMLInputElement>('#seed')?.value;
    this.root.innerHTML = body;
    if (seed) {
      const input = this.root.querySelector<HTMLInputElement>('#seed');
      if (input) input.value = seed;
    }
  }
}

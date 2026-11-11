// Input at the table beyond plain taps: a Schnaps poured from the Leiste,
// cards dragged between hand and table, a card thrown onto the felt, the
// hand sorted by hand. Whether a move is legal the core says (the view's
// flags, or its answer to the action).

import { sound } from '../audio';
import { consumableMod, de } from '../i18n';
import type { UiState } from '../ui/state';
import type { Fling } from '../ui/drag';
import type { Pour } from '../ui/drink';
import { glide, glideAll, motionOn, spotOfRect, spotsOf, springBack } from '../ui/fx';
import { arrangeHand, moveInOrder } from '../ui/handorder';
import type { NullRisk, RiskLevel, RunAction, Suit } from '../types';
import { updateRiseChoice } from '../ui/screens';
import { discardNeed } from '../ui/table';
import { PICK } from './consts';
import type { Actions, AppCtx, RunActions } from './context';
import type { Mods } from './mods';
import type { DrinkFx } from './playback/drinkfx';
import type { Flights } from './playback/flights';
import type { Fx } from './playback/fx';

/** A flick upwards this fast (px per ms) throws a card even if let go over the hand. */
const FLICK_UP = 0.9;
/** The cards of the hand and those lying on the table to be pressed. */
const PRESS_CARDS = '.hand-card, .skat-card';
/** The other hand cards making room (FLIP). */
const MAKE_ROOM_MS = 260;

/**
 * While discarding, the cards that came from the skat lie on the table
 * at first (those the core lets go; a "Ramponiert" one goes to the hand).
 */
function skatOnTable(ui: UiState): number[] {
  const game = ui.view?.game;
  if (game?.phase !== 'Discarding') return [];
  // "Drei drücken": the card drawn from a defender lies there too.
  return game.hand.filter((c) => (c.from_skat || c.drawn_from != null) && c.discardable).map((c) => c.id);
}

/** The choices at the table start over (a new game, a new view after a step). */
export function resetTableChoices(ui: UiState): void {
  ui.selected = skatOnTable(ui);
  ui.handGame = false;
  ui.pickGame = null;
  ui.pickRisk = null;
  ui.pickNullRisk = null;
  ui.omaPick = false;
  ui.swapTake = null;
  ui.peekTrick = false;
  ui.risePick = [];
}

export class TableInput {
  constructor(
    private readonly ctx: AppCtx,
    private readonly fx: Fx,
    private readonly flights: Flights,
    private readonly drinkFx: DrinkFx,
    private readonly mods: Mods,
  ) {}

  /**
   * A Schnaps poured on its target (../ui/drink.ts): the same action as
   * "Trinken" on its info card. Blue Curaçao on a hand card dyes it and
   * opens the suit picker for that card; on the table the cards are kept
   * as pictures, to be washed away if the core deals anew.
   */
  pourSchnaps(p: Pour): void {
    const ui = this.ctx.ui;
    const view = ui.view;
    const item = view?.consumables.find((c) => c.slot === p.slot);
    if (!view || !item || ui.busy) return;
    const c = item.consumable;
    if (c.needs_suit) {
      const card = p.kind === 'Card' ? p.card : null;
      if (card !== null) this.drinkFx.dyeCard(card, p.at);
      const open = (): void => {
        if (ui.busy || ui.overlay || ui.view !== view) return;
        ui.overlay = { kind: 'chalk', slot: p.slot, card };
        this.ctx.render();
      };
      if (card !== null && motionOn()) window.setTimeout(open, 480 * this.fx.tempo());
      else open();
      return;
    }
    if (c.needs_target) {
      // A Schnaps for a deck card: the card picker, as from the info card.
      const marks = consumableMod(c.id);
      const ask = marks.mod || marks.random ? { targets: item.targets, action: (id: number): RunAction => ({ UseConsumable: { slot: p.slot, target: id } }) } : null;
      void this.mods.openPicker(view, ask, (reasons) => ({ kind: 'target', slot: p.slot, reasons }));
      return;
    }
    this.drinkFx.pouring = { id: c.id, kind: p.kind, wash: p.kind === 'Table' ? this.drinkFx.snapshotTable() : [] };
    void this.ctx.act({ UseConsumable: { slot: p.slot, target: null } }).finally(() => {
      for (const el of this.drinkFx.pouring?.wash ?? []) el.remove();
      this.drinkFx.pouring = null;
    });
  }

  /**
   * "Hand sortieren: manuell": the hand keeps the order the player sees
   * (and dragged); cards new to it come in after their neighbour in the
   * core's order. Automatic: the core's order (null).
   */
  keepHandOrder(): void {
    const ui = this.ctx.ui;
    const game = ui.view?.game;
    if (ui.settings.handSort !== 'manual' || !game) {
      ui.handOrder = null;
      return;
    }
    const hand = ui.anim ? ui.anim.hand : game.hand;
    ui.handOrder = arrangeHand(hand, ui.handOrder).map((c) => c.id);
  }

  /** A hand card dragged within the hand (manual order): it goes before or after the card nearest to where it was let go. */
  private reorderHand(el: HTMLElement, at: DOMRect | null): void {
    const ui = this.ctx.ui;
    const screenEl = this.ctx.screenEl;
    const order = ui.handOrder;
    const id = el.dataset.id;
    if (!order || id === undefined || !at) return;
    const x = at.left + at.width / 2;
    const y = at.top + at.height / 2;
    let best: { id: number; after: boolean; d: number } | null = null;
    for (const card of Array.from(screenEl.querySelectorAll<HTMLElement>('.hand .hand-card'))) {
      if (card === el || card.dataset.id === undefined) continue;
      const r = card.getBoundingClientRect();
      const cx = r.left + r.width / 2;
      const d = Math.hypot(cx - x, r.top + r.height / 2 - y);
      if (!best || d < best.d) best = { id: Number(card.dataset.id), after: x > cx, d };
    }
    if (!best) return;
    const spots = spotsOf(screenEl, '.hand .hand-card');
    spots.set(id, spotOfRect(at));
    ui.handOrder = moveInOrder(order, Number(id), best.id, best.after);
    sound.play('card', { volume: 0.4, rate: 1.1 });
    this.ctx.render();
    glideAll(Array.from(screenEl.querySelectorAll<HTMLElement>('.hand .hand-card')), spots, { ms: 200 });
  }

  /**
   * A drag ended (../ui/drag.ts): the deck in the middle mixes in, a card
   * goes between hand and table. Onto a card on the table it takes that
   * card's place. Only which two lie there is decided here; the core checks
   * the pair on Drücken.
   */
  onDrop(el: HTMLElement, zone: HTMLElement | null, over: Element | null, at: DOMRect | null, fling: Fling): void {
    const ui = this.ctx.ui;
    if (ui.busy || !ui.view?.game) return;
    const kind = el.dataset.drag;
    const where = zone?.dataset.drop;
    // "Hand sortieren: manuell": a hand card let go over the hand changes places.
    if (where === 'hand' && ui.handOrder && el.classList.contains('hand-card')) {
      this.reorderHand(el, at);
      return;
    }
    if (kind === 'play') {
      this.onThrow(el, where === 'table', at, fling);
      return;
    }
    if (kind === 'deck') {
      if (where === 'mix' && ui.view.game.declare?.can_redeal) {
        void this.ctx.act('Redeal');
      }
      return;
    }
    if (kind !== 'card' || ui.view.game.phase !== 'Discarding') return;
    const id = Number(el.dataset.id ?? '-1');
    const sel = ui.selected;
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
    const need = discardNeed(ui.view.game);
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
    const game = this.ctx.ui.view?.game;
    const upward = fling.vy < -FLICK_UP && Math.abs(fling.vx) < -fling.vy * 1.2;
    if (!game?.your_turn || !(overFelt || upward)) {
      springBack(el, at, false);
      return;
    }
    if (el.dataset.legal !== 'true') {
      springBack(el, at, true);
      sound.play('card', { volume: 0.35, rate: 0.7 });
      this.ctx.showToast(de.table.notPlayable, 'top');
      return;
    }
    this.playCard(Number(el.dataset.id ?? '-1'), at, fling);
  }

  /** Play a hand card (tap or throw). */
  playCard(id: number, from: DOMRect | null, fling: Fling | null): void {
    this.flights.playedFrom = from;
    this.flights.thrown = fling;
    void this.ctx.act({ Play: id });
  }

  /**
   * Lay out the cards to press anew. A card changing sides is lifted and
   * flies in a curve to its new place (into its sorted slot in the hand, or
   * onto the table); the other cards glide aside to make room (FLIP). Each
   * starts where it is seen, so taps in quick succession pick the cards up
   * mid-air and nothing waits for a flight to end. `from`: where a dragged
   * card was let go.
   */
  moveCards(selected: number[], moved: number, from: DOMRect | null, swapped: number | null = null): void {
    const ui = this.ctx.ui;
    const screenEl = this.ctx.screenEl;
    const before = spotsOf(screenEl, PRESS_CARDS);
    if (from) before.set(String(moved), spotOfRect(from));
    const was = new Set(ui.selected);
    const now = new Set(selected);
    const changing = new Set([moved, ...(swapped === null ? [] : [swapped]), ...[...was].filter((x) => !now.has(x)), ...selected.filter((x) => !was.has(x))].map(String));
    ui.selected = selected;
    this.ctx.render();
    const els = Array.from(screenEl.querySelectorAll<HTMLElement>(PRESS_CARDS));
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

  /** Taps at the table that also count during a playback. */
  actions(): Actions {
    const ui = this.ctx.ui;
    return {
      'fast': () => {
        const view = ui.view;
        if (ui.busy) this.fx.hurry();
        // A tap on the open trick while nothing plays: the last one, as with "Letzter Stich".
        else if (!ui.anim && view?.game?.phase === 'Playing' && view.game.last_trick) {
          ui.peekTrick = true;
          this.ctx.render();
        }
      },
      'last-trick': () => {
        // Show the last finished trick on the felt, or the open one again.
        if (ui.anim || ui.busy || !ui.view?.game?.last_trick) return;
        ui.peekTrick = !ui.peekTrick;
        this.ctx.render();
      },
    };
  }

  /** The table's choices: Skat oder Hand, Drücken, the Ansage, playing, Kontra, Untot, the Schnaps-Leiste. */
  runActions(): RunActions {
    const ui = this.ctx.ui;
    const render = (): void => this.ctx.render();
    const act = (action: RunAction): void => void this.ctx.act(action);
    return {
      'pick-game': ({ num }) => {
        ui.pickGame = num('index');
        render();
      },
      'pick-risk': ({ el }) => {
        ui.pickRisk = (el.dataset.risk ?? 'Normal') as RiskLevel;
        render();
      },
      'pick-null-risk': ({ el }) => {
        ui.pickNullRisk = (el.dataset.risk ?? 'Normal') as NullRisk;
        render();
      },
      'pick-up-skat': () => act('PickUpSkat'),
      'give-up': () => {
        // "Kneipe verlassen" (DESIGN 8.7): ask first, a boss pub ends the run.
        ui.overlay = { kind: 'confirm-give-up' };
        render();
      },
      'confirm-give-up': () => {
        ui.overlay = null;
        act('GiveUpPub');
      },
      'play-hand': () => {
        ui.handGame = true;
        render();
      },
      'hand-back': () => {
        ui.handGame = false;
        ui.pickGame = null;
        ui.pickRisk = null;
        ui.pickNullRisk = null;
        render();
      },
      'declare-null': ({ el }) => act({ Declare: { game: 'Null', risk: (el.dataset.risk ?? 'Normal') as NullRisk } }),
      'oma': () => {
        ui.omaPick = true;
        render();
      },
      'oma-cancel': () => {
        ui.omaPick = false;
        render();
      },
      'swap-take': ({ num }) => {
        ui.swapTake = num('id');
        render();
      },
      'oma-give': ({ num }) => act({ TakeFromSkat: { give: num('id'), take: ui.swapTake } }),
      'declare': (_tap, view) => {
        const options = view.game?.declare;
        const game = ui.pickGame === null ? undefined : options?.games[ui.pickGame];
        if (!options || !game) return;
        const risk = ui.pickRisk ?? options.risks.find((r) => r.allowed)?.risk ?? 'Normal';
        act({ Declare: { game: game.game, risk } });
      },
      // Einmischen: one tap on the deck (or the button) mixes in at once.
      'pass': () => act('Redeal'),
      're': ({ el }) => act({ AnswerKontra: { re: el.dataset.re === 'true' } }),
      'rise-pick': ({ num }, view) => {
        // "Untot": a tap picks or drops a hand card, at most `pick` of them.
        const id = num('id');
        const need = view.game?.rise_choice?.pick ?? 3;
        const pick = ui.risePick;
        ui.risePick = pick.includes(id) ? pick.filter((x) => x !== id) : pick.length < need ? [...pick, id] : pick;
        // Only the tapped card and the button change: no re-render, the sheet does not enter again.
        const sheetEl = this.ctx.screenEl.querySelector<HTMLElement>('.sheet-rise');
        if (sheetEl) updateRiseChoice(sheetEl, ui.risePick, need);
        else render();
      },
      'rise-go': () => {
        const [a, b, c] = ui.risePick;
        if (a === undefined || b === undefined || c === undefined) return;
        ui.risePick = [];
        act({ ChooseRiseCards: [a, b, c] });
      },
      'select': ({ num }, view) => {
        // A tap moves a card between hand and table (the fallback for dragging).
        const id = num('id');
        const sel = ui.selected;
        const need = view.game ? discardNeed(view.game) : 2;
        if (sel.includes(id)) this.moveCards(sel.filter((x) => x !== id), id, null);
        else if (sel.length < need) this.moveCards([...sel, id], id, null);
        else this.moveCards([...sel.slice(1), id], id, null);
      },
      'discard': (_tap, view) => {
        const sel = ui.selected;
        const need = view.game ? discardNeed(view.game) : 2;
        const [a, b, c] = sel;
        if (sel.length !== need || a === undefined || b === undefined) return;
        if (need === 3 && c !== undefined) act({ DiscardThree: [a, b, c] });
        else act({ Discard: [a, b] });
      },
      'play': ({ el, num }) => this.playCard(num('id'), el.getBoundingClientRect(), null),
      'lift-item': ({ num }) => {
        // A tap on a drinkable Schnaps lifts it; the next tap on its target pours it (../ui/drink.ts).
        if (!ui.busy) this.ctx.drink.lift(num('slot'));
      },
      'use-item': ({ num }, view) => {
        const slot = num('slot');
        const item = view.consumables.find((c) => c.slot === slot);
        if (!item) return;
        if (item.consumable.needs_suit) {
          ui.overlay = { kind: 'chalk', slot, card: null };
          render();
        } else if (item.consumable.needs_target) {
          const marks = consumableMod(item.consumable.id);
          const ask = marks.mod || marks.random ? { targets: item.targets, action: (id: number): RunAction => ({ UseConsumable: { slot, target: id } }) } : null;
          void this.mods.openPicker(view, ask, (reasons) => ({ kind: 'target', slot, reasons }));
        } else {
          act({ UseConsumable: { slot, target: null } });
        }
      },
      'target': ({ el, num }, view) => {
        const o = ui.overlay;
        if (o?.kind !== 'target') return;
        const item = view.consumables.find((c) => c.slot === o.slot);
        const marks = item ? consumableMod(item.consumable.id) : { mod: null, random: false };
        this.flights.noteStamp(el);
        const action: RunAction = { UseConsumable: { slot: o.slot, target: num('id') } };
        if (marks.mod || marks.random) void this.mods.applyMod(view, action, num('id'), marks.mod, marks.random, o, 'apply');
        else act(action);
      },
      'chalk-card': ({ num }) => {
        const o = ui.overlay;
        if (o?.kind === 'chalk') {
          ui.overlay = { ...o, card: num('id') };
          render();
        }
      },
      'chalk-suit': ({ el }) => {
        const o = ui.overlay;
        const suit = el.dataset.suit as Suit | undefined;
        if (o?.kind === 'chalk' && o.card !== null && suit) act({ UseConsumable: { slot: o.slot, target: o.card, suit } });
      },
    };
  }
}

// Cards in the air and the table's reactions: the deal, the skat picked
// up, the pressed cards, a card thrown onto the felt, the trick swept to
// its winner, the regulars nodding and frowning, a Zinke stamped on.
// Only pictures: which cards move and where to comes from the core.

import { sound } from '../../audio';
import type { DealtCard, Declaration, DeckCard, RunAction, RunView, ScoreBreakdown, Seat } from '../../types';
import { backDesign } from '../../ui/backs';
import { cardBack, cardSvg } from '../../ui/cards';
import type { Fling } from '../../ui/drag';
import { deal, declareSign, frown, glide, glideAll, motionOn, nod, snipAcross, type Spot, spotOf, spotOfRect, spotsOf, throwCard } from '../../ui/fx';
import { zinkeStamp } from '../../ui/scenes';
import { declarationSign } from '../../ui/table';
import { PICK, rectOf } from '../consts';
import type { AppCtx } from '../context';
import type { Fx } from './fx';

/** A trick with this many Augen or more the player took annoys the regulars (the same bar as their quips). */
const BIG_TRICK = 10;

/** Cards on their way to the pressed skat: where they were seen, and their face. */
interface PressFlight {
  cards: { spot: Spot; face: string }[];
  /** Only the stack in the felt's corner takes them (after the Ansage); else also the Ansage's row. */
  stackOnly: boolean;
  /** Dropped when no render showed a target by then (performance.now()). */
  until: number;
}

export class Flights {
  /** Where the card the player just tapped was. */
  playedFrom: DOMRect | null = null;
  /** How the player's card was thrown (drag and flick), for its flight. */
  thrown: Fling | null = null;
  /** The face-down skat and the hand, seen when the player picked it up. */
  pickupFrom: { pile: Spot[]; hand: Map<string, Spot> } | null = null;
  /** The pressed cards still to fly onto their place (../../ui/table.ts pressedStack). */
  pressFlight: PressFlight | null = null;
  /** What was bought at the Tresen, seen as it was tapped: its picture flies into the Leiste. */
  bought: { html: string; rect: DOMRect } | null = null;
  /** Where the card was that a Zinke goes onto, seen as it was tapped (for its stamp). */
  private stampAt: { rect: DOMRect; at: number } | null = null;

  constructor(
    private readonly ctx: AppCtx,
    private readonly fx: Fx,
  ) {}

  private rect(selector: string): DOMRect | null {
    return rectOf(this.ctx.screenEl, selector);
  }

  /**
   * Before an action goes to the core: where the cards it will move are
   * seen now (later renders while the core's events play take them away).
   * Picking up the skat notes the face-down skat and the hand; Drücken the
   * cards lying on the table; the Ansage after the skat its row of pressed
   * cards, which then fly into the stack in the felt's corner.
   */
  noteCardSources(action: RunAction): void {
    const screenEl = this.ctx.screenEl;
    this.pickupFrom = null;
    if (!motionOn()) return;
    if (action === 'PickUpSkat') {
      const pile = Array.from(screenEl.querySelectorAll<HTMLElement>('.skat-pile .skat-spot')).map(spotOf);
      this.pickupFrom = { pile, hand: spotsOf(screenEl, '.hand-card') };
      return;
    }
    if (typeof action !== 'object') return;
    const from = 'Discard' in action || 'DiscardThree' in action ? '.skat-zone .skat-card' : 'Declare' in action ? '[data-testid="pressed"] .mini-card' : null;
    if (!from) return;
    const cards = Array.from(screenEl.querySelectorAll<HTMLElement>(from)).map((el) => ({ spot: spotOf(el), face: el.querySelector('svg')?.outerHTML ?? '' }));
    if (cards.length) this.pressFlight = { cards, stackOnly: 'Declare' in action, until: performance.now() + 5000 };
  }

  /**
   * The skat is picked up: each card is lifted off the face-down pile,
   * turns face up on the way (not after "Kurzer": it lay face up) and
   * lands where it goes, on the table to be pressed or, one the core keeps
   * from the skat ("Ramponiert"), in its sorted place in the hand. The hand
   * cards glide into their new places meanwhile.
   */
  pickUpFlights(skat: DealtCard[], peeked: boolean): void {
    const screenEl = this.ctx.screenEl;
    const seen = this.pickupFrom;
    this.pickupFrom = null;
    const zone = this.rect('.skat-zone') ?? this.rect('.deck-spot');
    skat.forEach((c, i) => {
      const el = screenEl.querySelector<HTMLElement>(`.skat-zone [data-id="${c.id}"], .hand-card[data-id="${c.id}"]`);
      if (!el) return;
      const at = seen?.pile[i] ?? (zone ? spotOfRect(zone) : null);
      if (!at) return;
      glide(el, at, { ...PICK, arc: (i % 2 ? -1 : 1) * PICK.arc, delay: i * 90, above: true, ...(peeked ? {} : { cover: cardBack() }) });
    });
    if (seen) glideAll(Array.from(screenEl.querySelectorAll<HTMLElement>('.hand-card')), seen.hand, { ms: 320, skip: new Set(skat.map((c) => String(c.id))) });
  }

  /**
   * The pressed cards fly onto their place once a render shows it: after
   * Drücken into the row of the Ansage (face up), after the Ansage into the
   * stack in the felt's corner, turning face down on the way.
   */
  flyPressed(): void {
    const p = this.pressFlight;
    if (!p) return;
    if (this.ctx.fastForward || performance.now() > p.until || this.ctx.ui.screen !== 'run') {
      this.pressFlight = null;
      return;
    }
    const screenEl = this.ctx.screenEl;
    const stack = Array.from(screenEl.querySelectorAll<HTMLElement>('[data-testid="pressed-play"] .pressed-card'));
    const row = p.stackOnly ? [] : Array.from(screenEl.querySelectorAll<HTMLElement>('[data-testid="pressed"] .mini-card'));
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

  /** The opponents' portraits on the table, by seat. */
  private faces(): Map<1 | 2, Element> {
    const out = new Map<1 | 2, Element>();
    for (const seat of [1, 2] as const) {
      const el = this.ctx.screenEl.querySelector(`.opp[data-seat="${seat}"] .portrait`);
      if (el) out.set(seat, el);
    }
    return out;
  }

  /**
   * The regulars react to a trick: the one who took it nods; a big one
   * the player took makes both frown. In a Ramsch everyone plays for
   * himself and a trick is no gain, so nobody reacts.
   */
  reactToTrick(view: RunView, winner: Seat, points: number): void {
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
  async gameEndEffect(b: ScoreBreakdown): Promise<number> {
    if (!motionOn() || b.declaration.game === 'Ramsch') return 0;
    for (const el of this.faces().values()) (b.won ? frown : nod)(el);
    const felt = this.rect('.center');
    if (b.won && b.schneider && felt) {
      await Promise.race([snipAcross(this.ctx.root, felt), this.fx.pause(1100)]);
      return 250;
    }
    return 520;
  }

  /**
   * The Ansage slams onto the felt as a sign, then shrinks into the game
   * line. Play goes on underneath at once (the sign takes no tap), so the
   * game is no slower for it.
   */
  declareEffect(d: Declaration): void {
    // Grand, Null and every game above Normal (Hand, Schneider, Schwarz, Ouvert announced) get the fist on the table.
    const big = d.game === 'Grand' || d.game === 'Null' || d.risk !== 'Normal';
    const hit = (): void => sound.play(big ? 'bang' : 'declare');
    const felt = this.rect('.center');
    if (!felt || !motionOn()) {
      // Without the sign the sound still comes (reduced motion does not mute).
      hit();
      return;
    }
    const into = this.rect('.gameline .decl');
    declareSign(this.ctx.root, declarationSign(d), felt, into, { onHit: hit });
  }

  /**
   * Where a finished trick goes: into the portrait of the regular who
   * took it (his name plate), or down past the hand to the player. Without
   * a portrait, off the edge of the table on his side.
   */
  sweepTarget(seat: Seat): { x: number; y: number } | null {
    const table = this.rect('.table');
    const trick = this.rect('.trick');
    if (!table || !trick) return null;
    const card = trick.height;
    const cy = trick.top + trick.height / 2;
    if (seat === 0) return { x: trick.left + trick.width / 2, y: table.bottom + card * 0.6 };
    const face = this.rect(`.opp[data-seat="${seat}"] .portrait`);
    if (face) return { x: face.left + face.width / 2, y: face.top + face.height / 2 };
    return seat === 1 ? { x: table.left - card, y: cy } : { x: table.right + card, y: cy };
  }

  /** Cards fly from the deck to the hands, in turns, then two to the skat. */
  dealEffect(): void {
    const screenEl = this.ctx.screenEl;
    const from = this.rect('.deck-pile') ?? this.rect('.deck-spot');
    if (!from) return;
    const hand = Array.from(screenEl.querySelectorAll('.hand-card'));
    const left = Array.from(screenEl.querySelectorAll('.opp[data-seat="1"] .opp-hand .card-back'));
    const right = Array.from(screenEl.querySelectorAll('.opp[data-seat="2"] .opp-hand .card-back'));
    const skat = Array.from(screenEl.querySelectorAll('.skat-pile .skat-spot'));
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

  /**
   * A card lands on the felt (cosmetic): an opponent's flies in from his
   * portrait turning over from his back; the player's from the hand, or,
   * thrown, as far as the flick sent it across the felt. Each comes down a
   * little askew, lies there for a moment and slides onto its place.
   */
  throwOnto(el: HTMLElement, seat: Seat, from: DOMRect, back: string | null): void {
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
    const row = this.rect('.trick-row');
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

  /** The picture of what is being bought at the Tresen and where it is (for its flight into the Leiste). */
  noteBought(pic: Element | null): void {
    const rect = pic?.getBoundingClientRect();
    this.bought = pic && rect?.width ? { html: pic.outerHTML, rect } : null;
  }

  /** Where the card is that a Zinke is about to go onto (its stamp comes down there). */
  noteStamp(card: Element | null): void {
    const rect = card?.getBoundingClientRect();
    this.stampAt = rect?.width ? { rect, at: performance.now() } : null;
  }

  /**
   * After the next render, stamp a Zinke onto `card` (as it was: `old`),
   * where it was tapped, or in the middle of the screen if that place is
   * gone or out of sight. Only pictures: the faces come from the core's
   * edition before and after the step.
   */
  stampLater(old: DeckCard, card: DeckCard, order: number): void {
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
    this.ctx.afterRender.push(() => zinkeStamp(before, after, place, order));
  }
}

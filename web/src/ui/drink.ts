// Drinking a Schnaps (DESIGN 8.9, user decision 03.11.2026): a Schnaps of
// the Leiste is dragged onto its target, or lifted with a tap and poured
// with a tap on the target (a second tap on it puts it back). Targets by
// the core's `target` of the Schnaps: the skat, the table (the felt), a
// hand card (`targets` of the core), the city map, or the player himself:
// an oversized pixel mouth slides up from the bottom. While a Schnaps is
// in the hand only its valid targets glow; let go anywhere else it springs
// back into its slot.
//
// Pure input and presentation: whether a Schnaps can be drunk now and on
// which cards is decided by the core (`usable_now`, `targets`); a pour
// hands the drop to the app, which sends `UseConsumable`. Everything here
// lives in its own layer outside the re-renders of the screen, never takes
// a pointer itself, respects the tempo and reduced motion, and ends at once
// when hurried (finishAll). Kept apart from ./drag.ts: a Schnaps slot has
// no `data-drag`, a card has no `data-schnaps`.

import { contentName, de, t } from '../i18n';
import type { ConsumableTarget, OwnedConsumable } from '../types';
import { esc } from './dom';
import { dur, motionOn, play } from './fx';
import { MOUTH_COLS, MOUTH_LIPS, MOUTH_ROWS, mouthSvg } from './mouth';
import { swallowClick } from './press';

/** Movement that turns a press on a Schnaps into a drag. */
const SLOP = 8;

/** Where a Schnaps went: its slot, the kind of target, the card, the boxes. */
export interface Pour {
  slot: number;
  /** The target it was poured on (Self also when its own target is not on screen). */
  kind: ConsumableTarget;
  /** The hand card (Blue Curaçao). */
  card: number | null;
  /** Box of the target. */
  at: DOMRect;
}

export interface DrinkHandlers {
  /** The Schnaps in a slot, from the core's view. */
  item: (slot: number) => OwnedConsumable | undefined;
  /** Whether lifting or dragging is off right now (busy, a sheet open). */
  blocked: () => boolean;
  /** A Schnaps was poured on a valid target. */
  pour: (p: Pour) => void;
}

interface Zone {
  el: Element;
  kind: ConsumableTarget;
  card: number | null;
}

interface Held {
  slot: number;
  /** The Schnaps' own target. */
  target: ConsumableTarget;
  /** Lifted by a tap (no ghost following a finger). */
  tapped: boolean;
  pointer: number;
  x0: number;
  y0: number;
  ghost: HTMLElement | null;
  over: Zone | null;
}

/** The colour of a drink, for its drops. */
const DRINK_INK: Record<string, string> = {
  shot: '#f4e9bf',
  double_grain: '#f4e9bf',
  herbal: '#5fae5a',
  blue_curacao: '#1fa8f0',
  snifter: '#c8732a',
  rotgut: '#a8883a',
  counter_beer: '#e8a417',
  road_beer: '#e8a417',
  eggnog: '#f3d36b',
};

export function drinkInk(id: string): string {
  return DRINK_INK[id] ?? '#f6e7b0';
}

function inside(r: DOMRect, x: number, y: number, pad = 0): boolean {
  return x >= r.left - pad && x <= r.right + pad && y >= r.top - pad && y <= r.bottom + pad;
}

function centre(r: DOMRect): { x: number; y: number } {
  return { x: r.left + r.width / 2, y: r.top + r.height / 2 };
}

export class Drink {
  private held: Held | null = null;
  /** A press on a Schnaps that may still become a drag. */
  private press: { pointer: number; slot: number; x0: number; y0: number } | null = null;
  private mouth: HTMLElement | null = null;
  private banner: HTMLElement | null = null;
  private lit: Element[] = [];
  private quietUntil = 0;

  constructor(
    private readonly screen: HTMLElement,
    private readonly layer: HTMLElement,
    private readonly h: DrinkHandlers,
  ) {
    screen.addEventListener('pointerdown', (e) => this.down(e));
    window.addEventListener('pointermove', (e) => this.move(e));
    window.addEventListener('pointerup', (e) => this.up(e));
    window.addEventListener('pointercancel', (e) => this.up(e, true));
    // A lifted Schnaps takes the next tap: on a target it pours, anywhere
    // else it goes back. Before the app sees the click.
    window.addEventListener('click', (e) => this.tap(e), true);
    window.addEventListener('keydown', (e) => {
      if (e.key === 'Escape' && this.held) this.cancel();
    });
  }

  /** Whether a Schnaps is in the hand (dragged or lifted). */
  get active(): boolean {
    return this.held !== null;
  }

  /** In the hand or just poured: no popover of what the pointer rests on. */
  get quiet(): boolean {
    return this.held !== null || performance.now() < this.quietUntil;
  }

  /** A tap on a drinkable Schnaps: lift it, or put it back when it is the lifted one. */
  lift(slot: number): void {
    if (this.held?.slot === slot) {
      this.cancel();
      return;
    }
    this.cancel();
    const item = this.h.item(slot);
    if (!item?.usable_now || this.h.blocked()) return;
    this.held = { slot, target: item.consumable.target ?? 'Self', tapped: true, pointer: -1, x0: 0, y0: 0, ghost: null, over: null };
    this.show();
  }

  /** Put a held Schnaps back without pouring. */
  cancel(): void {
    const held = this.held;
    if (!held) return;
    this.held = null;
    if (held.ghost) this.springBack(held);
    this.hide();
  }

  /** After a re-render of the screen: mark slot and targets again (or let go if the Schnaps is gone). */
  refresh(): void {
    const held = this.held;
    if (!held) return;
    const item = this.h.item(held.slot);
    if (!item?.usable_now) {
      this.cancel();
      return;
    }
    this.mark();
  }

  // ------------------------------------------------------------------ input

  private down(e: PointerEvent): void {
    if (e.pointerType === 'mouse' && e.button !== 0) return;
    const el = e.target instanceof Element ? e.target.closest<HTMLElement>('[data-schnaps].is-ready') : null;
    if (!el || this.h.blocked()) return;
    this.press = { pointer: e.pointerId, slot: Number(el.dataset.schnaps), x0: e.clientX, y0: e.clientY };
  }

  private move(e: PointerEvent): void {
    const p = this.press;
    if (p && p.pointer === e.pointerId && !this.held?.ghost) {
      if (Math.hypot(e.clientX - p.x0, e.clientY - p.y0) <= SLOP) return;
      this.press = null;
      // An info card opened by a long-press owns the finger now.
      if (this.h.blocked()) return;
      this.cancel();
      const item = this.h.item(p.slot);
      if (!item?.usable_now) return;
      this.held = { slot: p.slot, target: item.consumable.target ?? 'Self', tapped: false, pointer: p.pointer, x0: p.x0, y0: p.y0, ghost: null, over: null };
      this.show();
      this.held.ghost = this.makeGhost(p.slot);
    }
    const held = this.held;
    if (!held?.ghost || held.pointer !== e.pointerId) return;
    const zone = this.zoneAt(e.clientX, e.clientY);
    this.hover(zone);
    const tilt = zone ? (zone.kind === 'Self' ? 0 : -38) : Math.max(-14, Math.min(14, (e.clientX - held.x0) / 10));
    held.ghost.style.left = `${e.clientX}px`;
    held.ghost.style.top = `${e.clientY}px`;
    held.ghost.style.transform = `translate(-50%, -60%) rotate(${tilt}deg) scale(1.35)`;
  }

  private up(e: PointerEvent, cancelled = false): void {
    if (this.press?.pointer === e.pointerId) this.press = null;
    const held = this.held;
    if (!held?.ghost || held.pointer !== e.pointerId) return;
    // The click after a drag is no tap.
    swallowClick(80);
    const zone = cancelled ? null : this.zoneAt(e.clientX, e.clientY);
    if (zone) this.pour(zone);
    else this.cancel();
  }

  private tap(e: MouseEvent): void {
    const held = this.held;
    if (!held?.tapped) return;
    const target = e.target instanceof Element ? e.target : null;
    const slot = target?.closest<HTMLElement>('[data-schnaps]');
    // Another drinkable Schnaps: the app lifts that one instead.
    if (slot && Number(slot.dataset.schnaps) !== held.slot && slot.classList.contains('is-ready')) {
      this.cancel();
      return;
    }
    e.preventDefault();
    e.stopImmediatePropagation();
    if (slot) {
      this.cancel();
      return;
    }
    // A keyboard "click" (Enter on a focused card or the skat) has no point: the target that holds it.
    const zone = e.detail === 0 && target ? (this.zones().find((z) => z.el.contains(target)) ?? null) : this.zoneAt(e.clientX, e.clientY);
    if (zone) this.pour(zone);
    else this.cancel();
  }

  // ---------------------------------------------------------------- targets

  /** The valid targets of the held Schnaps now on screen. */
  private zones(): Zone[] {
    const held = this.held;
    if (!held) return [];
    const item = this.h.item(held.slot);
    const scr = this.screen;
    const out: Zone[] = [];
    switch (held.target) {
      case 'Card':
        for (const el of Array.from(scr.querySelectorAll<HTMLElement>('.hand .hand-card[data-id]'))) {
          const id = Number(el.dataset.id);
          if (item?.targets.includes(id)) out.push({ el, kind: 'Card', card: id });
        }
        break;
      case 'Skat': {
        const el = scr.querySelector('.skat-pile:not(:disabled)');
        if (el) out.push({ el, kind: 'Skat', card: null });
        break;
      }
      case 'Table': {
        const el = scr.querySelector('.run.is-table .declare-choice .felt') ?? scr.querySelector('.run.is-table .table');
        if (el) out.push({ el, kind: 'Table', card: null });
        break;
      }
      case 'Map': {
        const el = scr.querySelector('.route-map');
        if (el) out.push({ el, kind: 'Map', card: null });
        break;
      }
      default:
        break;
    }
    // Drunk yourself; also the way for a Schnaps whose target is not on this screen.
    if (held.target === 'Self' || (held.target !== 'Card' && !out.length)) {
      const mouth = this.mouth;
      if (mouth) out.push({ el: mouth, kind: 'Self', card: null });
    }
    return out;
  }

  private zoneAt(x: number, y: number): Zone | null {
    for (const z of this.zones()) {
      const r = z.el.getBoundingClientRect();
      // Only the visible face of the mouth counts.
      if (z.kind === 'Self') {
        if (inside(r, x, y) && y >= Math.max(r.top, window.innerHeight * 0.45)) return z;
        continue;
      }
      if (inside(r, x, y, 6)) return z;
    }
    return null;
  }

  private hover(zone: Zone | null): void {
    const held = this.held;
    if (!held) return;
    if (held.over?.el === zone?.el) return;
    held.over?.el.classList.remove('is-drink-over');
    zone?.el.classList.add('is-drink-over');
    held.over = zone;
  }

  /** Slot lifted, targets glowing, the banner; the mouth for a Schnaps drunk yourself. */
  private show(): void {
    const held = this.held;
    if (!held) return;
    this.screen.ownerDocument.body.classList.add('is-drinking');
    const needsMouth = held.target === 'Self' || (held.target !== 'Card' && !this.zonesWithoutMouth().length);
    if (needsMouth) this.showMouth();
    this.mark();
    this.showBanner();
  }

  private zonesWithoutMouth(): Zone[] {
    return this.zones().filter((z) => z.kind !== 'Self');
  }

  private mark(): void {
    const held = this.held;
    if (!held) return;
    for (const el of this.lit) el.classList.remove('drink-target', 'is-drink-over');
    this.lit = this.zones().map((z) => z.el);
    for (const el of this.lit) el.classList.add('drink-target');
    if (held.over && this.lit.includes(held.over.el)) held.over.el.classList.add('is-drink-over');
    for (const el of Array.from(this.screen.querySelectorAll('.schnaps-slot.is-lifted'))) el.classList.remove('is-lifted');
    this.screen.querySelector(`.schnaps-slot[data-schnaps="${held.slot}"]`)?.classList.add('is-lifted');
    this.screen.querySelector(`.schnaps-slot[data-schnaps="${held.slot}"]`)?.classList.toggle('is-held', Boolean(held.ghost));
  }

  private hide(): void {
    for (const el of this.lit) el.classList.remove('drink-target', 'is-drink-over');
    this.lit = [];
    for (const el of Array.from(this.screen.querySelectorAll('.schnaps-slot.is-lifted'))) el.classList.remove('is-lifted', 'is-held');
    this.screen.ownerDocument.body.classList.remove('is-drinking');
    this.banner?.remove();
    this.banner = null;
    this.hideMouth();
  }

  private showBanner(): void {
    const held = this.held;
    if (!held) return;
    const item = this.h.item(held.slot);
    if (!item) return;
    const id = item.consumable.id;
    const name = contentName('consumables', id);
    const kind = held.target !== 'Card' && held.target !== 'Self' && !this.zonesWithoutMouth().length ? 'Self' : held.target;
    const title = held.tapped ? de.drink.tapTarget : (de.drink.dragTo[kind] ?? de.drink.tapTarget);
    const sub = held.tapped ? t(de.drink.tapCancel, { name }) : t(de.drink.sub, { name, effect: de.drink.effect[id] ?? '' });
    this.banner?.remove();
    const banner = document.createElement('div');
    banner.className = 'drink-banner';
    banner.dataset.testid = 'drink-banner';
    banner.setAttribute('role', 'status');
    banner.innerHTML = `<b>${esc(title)}</b><small>${esc(sub)}</small>`;
    this.layer.appendChild(banner);
    this.banner = banner;
    // Above the mouth, else just below the Leiste.
    const leiste = this.screen.querySelector('.leiste')?.getBoundingClientRect();
    if (this.mouth) {
      const top = this.mouthTop() - banner.offsetHeight - 14;
      banner.style.top = `${Math.max(leiste ? leiste.bottom + 8 : 8, top)}px`;
    } else {
      banner.style.top = `${leiste ? leiste.bottom + 10 : 80}px`;
    }
  }

  // ------------------------------------------------------------------ mouth

  private mouthBox(): { w: number; h: number } {
    const w = Math.min(window.innerWidth, 440);
    return { w, h: (w * MOUTH_ROWS) / MOUTH_COLS };
  }

  /** Top of the mouth on screen when it is up. */
  private mouthTop(): number {
    const { h } = this.mouthBox();
    return window.innerHeight - Math.min(h, window.innerHeight * 0.52);
  }

  private showMouth(): void {
    if (!this.mouth) {
      const m = document.createElement('div');
      m.className = 'drink-mouth';
      m.dataset.testid = 'drink-mouth';
      m.setAttribute('aria-hidden', 'true');
      this.layer.appendChild(m);
      this.mouth = m;
    }
    const m = this.mouth;
    const { w, h } = this.mouthBox();
    m.style.width = `${w}px`;
    m.style.height = `${h}px`;
    m.style.top = `${this.mouthTop()}px`;
    m.innerHTML = mouthSvg('open', true);
    m.classList.remove('is-down');
    // Slides up from below (CSS transition; none with reduced motion).
    void m.offsetWidth;
    m.classList.add('is-up');
  }

  private hideMouth(): void {
    const m = this.mouth;
    this.mouth = null;
    this.dropMouth(m);
  }

  /** Centre of the lips on screen. */
  private lips(m: HTMLElement): { x: number; y: number } {
    const r = m.getBoundingClientRect();
    return { x: r.left + (MOUTH_LIPS[0] / MOUTH_COLS) * r.width, y: r.top + (MOUTH_LIPS[1] / MOUTH_ROWS) * r.height };
  }

  // ----------------------------------------------------------------- pouring

  private makeGhost(slot: number): HTMLElement {
    const src = this.screen.querySelector(`.schnaps-slot[data-schnaps="${slot}"] svg.schnaps-pic`);
    const g = document.createElement('div');
    g.className = 'drink-glass';
    g.dataset.testid = 'drink-glass';
    g.setAttribute('aria-hidden', 'true');
    if (src) g.appendChild(src.cloneNode(true));
    const r = this.slotRect(slot);
    if (r) {
      const c = centre(r);
      g.style.left = `${c.x}px`;
      g.style.top = `${c.y}px`;
    }
    g.style.transform = 'translate(-50%, -60%) scale(1.35)';
    this.layer.appendChild(g);
    return g;
  }

  private slotRect(slot: number): DOMRect | null {
    return this.screen.querySelector(`.schnaps-slot[data-schnaps="${slot}"]`)?.getBoundingClientRect() ?? null;
  }

  /** Let go on nothing: the glass flies back into its slot, which shakes its head. */
  private springBack(held: Held): void {
    const ghost = held.ghost;
    if (!ghost) return;
    held.ghost = null;
    const slotEl = this.screen.querySelector<HTMLElement>(`.schnaps-slot[data-schnaps="${held.slot}"]`);
    const r = slotEl?.getBoundingClientRect();
    const ms = dur(240);
    if (!r || !ms) {
      ghost.remove();
      return;
    }
    const from = ghost.getBoundingClientRect();
    const a = centre(from);
    const b = centre(r);
    ghost.classList.add('is-back');
    ghost.dataset.testid = 'drink-spring';
    const anim = play(
      ghost,
      [
        { transform: `${ghost.style.transform}` },
        { transform: `translate(calc(-50% + ${(b.x - a.x).toFixed(1)}px), calc(-60% + ${(b.y - a.y).toFixed(1)}px)) rotate(0deg) scale(1)` },
      ],
      { duration: ms, easing: 'cubic-bezier(.3,1.4,.5,1)', fill: 'forwards' },
    );
    const gone = (): void => {
      ghost.remove();
      if (slotEl?.isConnected && motionOn()) {
        play(
          slotEl,
          [
            { transform: 'translateX(0)' },
            { transform: 'translateX(-5px)', offset: 0.25 },
            { transform: 'translateX(4px)', offset: 0.5 },
            { transform: 'translateX(-2px)', offset: 0.75 },
            { transform: 'translateX(0)' },
          ],
          { duration: 260 },
        );
      }
    };
    anim.finished.then(gone, gone);
  }

  private pour(zone: Zone): void {
    const held = this.held;
    if (!held) return;
    const item = this.h.item(held.slot);
    const at = zone.el.getBoundingClientRect();
    let ghost = held.ghost;
    held.ghost = null;
    this.held = null;
    this.quietUntil = performance.now() + 2500;
    const keepMouth = zone.kind === 'Self' ? this.mouth : null;
    if (keepMouth) this.mouth = null;
    this.hide();
    if (!item) {
      ghost?.remove();
      this.dropMouth(keepMouth);
      return;
    }
    if (!ghost && motionOn()) ghost = this.makeGhost(held.slot);
    const id = item.consumable.id;
    if (zone.kind === 'Self' && keepMouth) this.gulp(keepMouth, ghost, id);
    else if (ghost) this.pourOnto(ghost, at, id);
    else this.dropMouth(keepMouth);
    this.h.pour({ slot: held.slot, kind: zone.kind, card: zone.card, at });
  }

  /** Let a mouth no longer in use slide down (after `after` ms). */
  private dropMouth(m: HTMLElement | null, after = 0): void {
    if (!m) return;
    const go = (): void => {
      m.classList.remove('is-up');
      m.classList.add('is-down');
      window.setTimeout(() => m.remove(), motionOn() ? 320 : 0);
    };
    if (after > 0) window.setTimeout(go, after);
    else go();
  }

  /** Poured onto a box: the glass moves above it, tips, drops fall onto it with a "gluck". */
  private pourOnto(ghost: HTMLElement, at: DOMRect, id: string): void {
    const ms = dur(620);
    if (!ms) {
      ghost.remove();
      return;
    }
    const g0 = centre(ghost.getBoundingClientRect());
    const c = centre(at);
    // Above a small target (skat, card); over the middle of a large one (table, map).
    const large = at.height > 220;
    const over = large
      ? { x: Math.min(window.innerWidth - 40, c.x + 50), y: c.y - 70 }
      : { x: Math.min(window.innerWidth - 40, c.x + Math.min(70, at.width * 0.3)), y: Math.max(40, at.top - 26) };
    const base = 'translate(-50%, -60%)';
    const anim = play(
      ghost,
      [
        { transform: `${base} scale(1.35)`, left: `${g0.x}px`, top: `${g0.y}px`, opacity: 1 },
        { transform: `${base} rotate(-25deg) scale(1.35)`, left: `${over.x}px`, top: `${over.y}px`, opacity: 1, offset: 0.25 },
        { transform: `${base} rotate(-112deg) scale(1.35)`, left: `${over.x}px`, top: `${over.y}px`, opacity: 1, offset: 0.45 },
        { transform: `${base} rotate(-118deg) scale(1.35)`, left: `${over.x}px`, top: `${over.y}px`, opacity: 1, offset: 0.85 },
        { transform: `${base} rotate(-30deg) scale(1.1)`, left: `${over.x}px`, top: `${over.y - 16}px`, opacity: 0 },
      ],
      { duration: ms, fill: 'forwards', easing: 'ease-out' },
    );
    const gone = (): void => ghost.remove();
    anim.finished.then(gone, gone);
    const ink = drinkInk(id);
    const n = 16;
    for (let i = 0; i < n; i++) {
      const d = document.createElement('i');
      d.className = 'drink-drop';
      d.style.background = ink;
      const size = 4 + (i % 3) * 2;
      d.style.width = `${size}px`;
      d.style.height = `${size}px`;
      d.style.left = `${over.x - 14}px`;
      d.style.top = `${over.y + 4}px`;
      this.layer.appendChild(d);
      const tx = c.x - (over.x - 14) + (Math.random() - 0.5) * Math.min(90, at.width * 0.6);
      const ty = c.y - (over.y + 4) + (Math.random() - 0.5) * Math.min(40, at.height * 0.4);
      const drop = play(
        d,
        [
          { transform: 'translate(0, 0)', opacity: 1 },
          { transform: `translate(${(tx * 0.45).toFixed(1)}px, ${(ty * 0.2).toFixed(1)}px)`, opacity: 1, offset: 0.45 },
          { transform: `translate(${tx.toFixed(1)}px, ${ty.toFixed(1)}px)`, opacity: 1, offset: 0.9 },
          { transform: `translate(${tx.toFixed(1)}px, ${(ty + 4).toFixed(1)}px) scale(.4)`, opacity: 0 },
        ],
        { duration: ms * 0.45, delay: ms * (0.38 + (i / n) * 0.42), easing: 'ease-in', fill: 'both' },
      );
      const off = (): void => d.remove();
      drop.finished.then(off, off);
    }
    this.word(de.drink.gluck, over.x + 18, over.y - 34, ms * 0.4, ms * 0.6, 'is-gluck');
    // The target lights up as it is wet.
    const wet = document.createElement('div');
    wet.className = 'drink-wet';
    wet.style.left = `${at.left}px`;
    wet.style.top = `${at.top}px`;
    wet.style.width = `${at.width}px`;
    wet.style.height = `${at.height}px`;
    wet.style.setProperty('--ink', ink);
    this.layer.appendChild(wet);
    const glow = play(wet, [{ opacity: 0 }, { opacity: 0.75, offset: 0.4 }, { opacity: 0 }], { duration: ms * 0.7, delay: ms * 0.45, fill: 'both' });
    const out = (): void => wet.remove();
    glow.finished.then(out, out);
  }

  /** Drunk yourself: the glass tips into the mouth, a gulp, "Ahh!" and what it did. */
  private gulp(m: HTMLElement, ghost: HTMLElement | null, id: string): void {
    const ms = dur(1700);
    if (!ms || !ghost) {
      ghost?.remove();
      this.dropMouth(m);
      return;
    }
    const lips = this.lips(m);
    const g0 = centre(ghost.getBoundingClientRect());
    const base = 'translate(-50%, -60%)';
    const tip = play(
      ghost,
      [
        { transform: `${base} scale(1.35)`, left: `${g0.x}px`, top: `${g0.y}px`, opacity: 1 },
        { transform: `${base} rotate(150deg) scale(1.5)`, left: `${lips.x + 8}px`, top: `${lips.y - 50}px`, opacity: 1, offset: 0.16 },
        { transform: `${base} rotate(165deg) scale(1.5)`, left: `${lips.x + 8}px`, top: `${lips.y - 46}px`, opacity: 1, offset: 0.28 },
        { transform: `${base} rotate(165deg) scale(1.3)`, left: `${lips.x + 8}px`, top: `${lips.y - 70}px`, opacity: 0, offset: 0.38 },
        { transform: `${base} rotate(165deg) scale(1.3)`, left: `${lips.x + 8}px`, top: `${lips.y - 70}px`, opacity: 0 },
      ],
      { duration: ms, fill: 'forwards', easing: 'ease-out' },
    );
    const gone = (): void => ghost.remove();
    tip.finished.then(gone, gone);
    const step = (at: number, fn: () => void): void => {
      window.setTimeout(fn, at * ms);
    };
    step(0.14, () => (m.innerHTML = mouthSvg('drink')));
    step(0.32, () => (m.innerHTML = mouthSvg('gulp')));
    const gain = de.drink.gain[id];
    this.word(de.drink.ahh, lips.x + 30, lips.y - 120, ms * 0.36, ms * 0.64, 'is-ahh');
    if (gain) this.word(gain, lips.x - 60, lips.y - 175, ms * 0.44, ms * 0.56, 'is-gain');
    this.dropMouth(m, ms);
  }

  /** A word that pops up and goes (a bubble, a sign). */
  private word(text: string, x: number, y: number, delay: number, ms: number, cls: string): void {
    const w = document.createElement('div');
    w.className = `drink-word ${cls}`;
    w.dataset.testid = `drink-word-${cls.replace('is-', '')}`;
    w.textContent = text;
    w.style.left = `${Math.max(8, Math.min(window.innerWidth - 8, x))}px`;
    w.style.top = `${Math.max(8, y)}px`;
    this.layer.appendChild(w);
    const anim = play(
      w,
      [
        { transform: 'translate(-50%, 0) scale(.4)', opacity: 0 },
        { transform: 'translate(-50%, 0) scale(1.12)', opacity: 1, offset: 0.18 },
        { transform: 'translate(-50%, 0) scale(1)', opacity: 1, offset: 0.3 },
        { transform: 'translate(-50%, -6px) scale(1)', opacity: 1, offset: 0.85 },
        { transform: 'translate(-50%, -14px) scale(1)', opacity: 0 },
      ],
      { duration: ms, delay, fill: 'both' },
    );
    const gone = (): void => w.remove();
    anim.finished.then(gone, gone);
  }
}

// Scene effects (animation package C): the boss entrance under a
// spotlight, a relic sliding
// over the counter into the Leiste, a Schnaps popping its cork on the way
// into its slot, and the Zinke stamp coming down on a card.
//
// Pure presentation: what is shown comes from the core's views and events
// (the app passes the pictures and places in). Everything goes through
// ./fx.ts: tempo-scaled, off with reduced motion, ended at once by
// finishAll(). Nothing here catches a tap; a tap while a scene runs ends
// it (see sceneRunning()).

import { sound } from '../audio';
import { de } from '../i18n';
import { dur, play, pulse } from './fx';
import { STAMP } from './fxart';
import { layer } from './juice';

/** Scenes in progress (a tap anywhere hurries them, ./app.ts). */
let running = 0;

/** Whether a scene of this file is playing right now. */
export function sceneRunning(): boolean {
  return running > 0;
}

/** Count `anim` as a scene until it ends; `end` runs then (finished or hurried). */
function scene(anim: Animation, end?: () => void): void {
  running += 1;
  const done = (): void => {
    running = Math.max(0, running - 1);
    end?.();
  };
  anim.finished.then(done, done);
}

/**
 * A linear timeline of `ms` (already tempo-scaled) on `host`: `frame(p)`
 * every animation frame, p from 0 to 1 and clamped (a frame's time may lie
 * before the start). Tracked through fx.play, so finishAll() ends it with
 * frame(1).
 */
function timeline(host: Element, ms: number, frame: (p: number) => void): Animation {
  const timer = play(host, [], { duration: ms });
  let over = false;
  const tick = (): void => {
    if (over) return;
    const p = timer.playState === 'finished' ? 1 : Math.max(0, Math.min(1, Number(timer.currentTime ?? 0) / ms));
    frame(p);
    if (p < 1) requestAnimationFrame(tick);
  };
  const last = (): void => {
    if (over) return;
    over = true;
    frame(1);
  };
  timer.finished.then(last, last);
  requestAnimationFrame(tick);
  return timer;
}

function clamp01(v: number): number {
  return Math.max(0, Math.min(1, v));
}

/** Round to the 4 px grid of the scene's pixels. */
function px4(v: number): number {
  return Math.round(v / 4) * 4;
}

function centre(r: DOMRect): { x: number; y: number } {
  return { x: r.left + r.width / 2, y: r.top + r.height / 2 };
}

/** A random number in [-n, n]: cosmetic only (never the core's RNG). */
function jitter(n: number): number {
  return (Math.random() * 2 - 1) * n;
}

/** A timer that does not fire once `anim` was hurried to its end. */
function at(anim: Animation, ms: number, fn: () => void): void {
  const timer = window.setTimeout(fn, ms);
  const quiet = (): void => window.clearTimeout(timer);
  anim.finished.then(quiet, quiet);
}

// ---------------------------------------------------------------- boss entrance

let spotIds = 0;

/**
 * The light cone onto one boss seat as rows of 4 px: from a narrow point
 * at the top of the screen widening to the seat's box, then the box
 * itself. Only rows above `upTo` are drawn (the light falls).
 */
function cone(box: DOMRect, upTo: number): string {
  const cx = box.left + box.width / 2;
  const half = box.width / 2 + 8;
  const top = Math.max(4, box.top - 4);
  const bottom = box.bottom + 8;
  let d = '';
  for (let y = 0; y < Math.min(bottom, upTo); y += 4) {
    const h = y < top ? 10 + (half - 10) * (y / top) : half;
    const x0 = px4(cx - h);
    const w = Math.max(4, px4(cx + h) - x0);
    d += `M${x0} ${y}h${w}v4h${-w}z`;
  }
  return d;
}

/**
 * A boss pub opens: the light dims, a spotlight falls onto both boss
 * players and their name plates drop in from above with a bounce. About
 * 1.2 s at tempo "Normal"; a tap ends it.
 */
export function bossEntrance(screen: HTMLElement): void {
  const D = dur(1200);
  if (!D) return;
  const seats = Array.from(screen.querySelectorAll<HTMLElement>('.opp.is-boss'));
  if (!seats.length) return;
  const W = window.innerWidth;
  const H = window.innerHeight;
  const boxes = seats.map((s) => (s.querySelector('.opp-main') ?? s).getBoundingClientRect());
  const reach = Math.max(...boxes.map((b) => b.bottom + 8));
  const id = `fx-spot-${++spotIds}`;
  const el = document.createElement('div');
  el.className = 'fx-spot';
  el.dataset.testid = 'boss-entrance';
  el.setAttribute('aria-hidden', 'true');
  el.innerHTML = `<svg width="${W}" height="${H}" viewBox="0 0 ${W} ${H}" shape-rendering="crispEdges"><defs><mask id="${id}" maskUnits="userSpaceOnUse" x="0" y="0" width="${W}" height="${H}"><rect width="${W}" height="${H}" fill="#fff"/><path class="fx-spot-hole" fill="#000"/></mask></defs><rect class="fx-spot-dark" width="${W}" height="${H}" fill="#07060a" mask="url(#${id})" opacity="0"/><path class="fx-spot-beam" fill="#ffd98a" opacity=".14"/></svg>`;
  document.body.appendChild(el);
  const dark = el.querySelector('.fx-spot-dark');
  const hole = el.querySelector('.fx-spot-hole');
  const beam = el.querySelector('.fx-spot-beam');
  let shownRows = -1;
  const timer = timeline(el, D, (p) => {
    // The light dims in steps, holds, and comes back at the end.
    const dim = p < 0.15 ? Math.ceil((p / 0.15) * 4) / 4 : p < 0.82 ? 1 : Math.floor((1 - (p - 0.82) / 0.18) * 4) / 4;
    dark?.setAttribute('opacity', (0.8 * clamp01(dim)).toFixed(2));
    // The cone falls from the ceiling onto the two seats.
    const rows = px4(clamp01((p - 0.14) / 0.24) * reach);
    if (rows !== shownRows) {
      shownRows = rows;
      const d = boxes.map((b) => cone(b, rows)).join('');
      hole?.setAttribute('d', d);
      beam?.setAttribute('d', d);
    }
    beam?.setAttribute('opacity', p > 0.82 ? '0' : '.14');
  });
  scene(timer, () => el.remove());
  // The name plates drop in from above, one after the other, and bounce.
  seats.forEach((seat, i) => {
    const plate = seat.querySelector<HTMLElement>('.opp-name');
    if (!plate) return;
    const fall = Math.round(plate.getBoundingClientRect().bottom + 24);
    const delay = D * (0.4 + i * 0.1);
    const anim = play(
      plate,
      [
        { transform: `translateY(${-fall}px) rotate(${jitter(4).toFixed(1)}deg)`, easing: 'cubic-bezier(.55,0,.9,.45)' },
        { transform: 'translateY(0) scale(1.06, .86)', offset: 0.5, easing: 'cubic-bezier(.2,.7,.4,1)' },
        { transform: 'translateY(-12px) scale(1)', offset: 0.68, easing: 'cubic-bezier(.5,0,.8,.4)' },
        { transform: 'translateY(0) scale(1.02, .94)', offset: 0.82, easing: 'ease-out' },
        { transform: 'translateY(-3px)', offset: 0.91, easing: 'ease-in' },
        { transform: 'none' },
      ],
      { duration: D * 0.4, delay, fill: 'backwards' },
    );
    anim.id = 'boss-plate';
    at(anim, delay + D * 0.2, () => sound.play('knock', { volume: 0.45, rate: 1.15 + i * 0.1 }));
  });
}

// ---------------------------------------------------------------- the Tresen

/** A sprite in the fx layer, its centre at (x, y). */
function sprite(cls: string, html: string, x: number, y: number): HTMLElement {
  const el = document.createElement('span');
  el.className = `fx-sprite ${cls}`;
  el.setAttribute('aria-hidden', 'true');
  el.innerHTML = html;
  el.style.left = `${Math.round(x)}px`;
  el.style.top = `${Math.round(y)}px`;
  layer().appendChild(el);
  return el;
}

/** Hide `el` until a flight lands on it (`ms`), then let it show and bump. */
function waitFor(el: HTMLElement | null, ms: number): void {
  if (!el) return;
  const hide = play(el, [{ opacity: 0 }, { opacity: 0, offset: 0.97 }, { opacity: 1 }], { duration: ms, fill: 'backwards' });
  const land = (): void => {
    if (el.isConnected) pulse(el, 1.25, 240);
  };
  hide.finished.then(land, () => undefined);
}

/** The flight from `from` (centre of the sprite, `size` px wide) into `to`, bending upwards. */
function flightFrames(from: { x: number; y: number }, to: DOMRect | null, size: number): Keyframe[] {
  const start = 'translate(0, 0) scale(1)';
  if (!to || !to.width) return [{ transform: start, opacity: 1 }, { transform: 'translate(0, -30px) scale(.6)', opacity: 0 }];
  const b = centre(to);
  const dx = b.x - from.x;
  const dy = b.y - from.y;
  const s = Math.max(0.3, Math.min(1.2, to.width / size));
  return [
    { transform: start, opacity: 1, easing: 'cubic-bezier(.4,0,.6,1)' },
    { transform: `translate(${(dx * 0.45).toFixed(1)}px, ${(dy * 0.45 - 40).toFixed(1)}px) scale(${((1 + s) / 2 + 0.1).toFixed(2)}) rotate(-8deg)`, opacity: 1, offset: 0.5, easing: 'cubic-bezier(.5,0,.8,.6)' },
    { transform: `translate(${dx.toFixed(1)}px, ${dy.toFixed(1)}px) scale(${s.toFixed(2)}) rotate(0deg)`, opacity: 1 },
  ];
}

/**
 * A relic bought at the Tresen: its beer mat (`html`, the tile's emblem)
 * slides over the counter towards the player, then flies into the relic
 * strip of the Leiste (`target`, its new chip, hidden until it lands).
 */
export function relicToStrip(html: string, from: DOMRect, target: HTMLElement | null): void {
  const D = dur(1050);
  if (!D || !from.width) return;
  const a = centre(from);
  const el = sprite('fx-buy', html, a.x, a.y);
  el.dataset.testid = 'relic-fly';
  el.style.width = `${Math.round(from.width)}px`;
  el.style.height = `${Math.round(from.height)}px`;
  // Over the counter: down towards the player, growing a little, with a wobble.
  const slide = Math.round(Math.min(110, window.innerHeight * 0.12));
  const sx = jitter(10);
  const ex = sx * 1.1;
  const ey = slide + 4;
  const frames: Keyframe[] = [
    { transform: 'translate(0, 0) scale(1) rotate(0deg)', opacity: 1, easing: 'ease-out' },
    { transform: 'translate(0, -6px) scale(1.06) rotate(-2deg)', opacity: 1, offset: 0.1, easing: 'cubic-bezier(.3,.6,.4,1)' },
    { transform: `translate(${sx.toFixed(1)}px, ${slide}px) scale(1.22) rotate(3deg)`, opacity: 1, offset: 0.36 },
    { transform: `translate(${ex.toFixed(1)}px, ${ey}px) scale(1.24) rotate(-1deg)`, opacity: 1, offset: 0.44, easing: 'cubic-bezier(.4,0,.6,1)' },
  ];
  const rest = target?.getBoundingClientRect();
  if (rest?.width) {
    // Then up into its chip in the Leiste, bending over the screen.
    const b = centre(rest);
    const tx = b.x - a.x;
    const ty = b.y - a.y;
    const sf = Math.max(0.3, Math.min(1.2, rest.width / from.width));
    const mx = ex + (tx - ex) * 0.5;
    const my = ey + (ty - ey) * 0.5 - 50;
    frames.push(
      { transform: `translate(${mx.toFixed(1)}px, ${my.toFixed(1)}px) scale(${((1.24 + sf) / 2).toFixed(2)}) rotate(-8deg)`, opacity: 1, offset: 0.72, easing: 'cubic-bezier(.5,0,.8,.6)' },
      { transform: `translate(${tx.toFixed(1)}px, ${ty.toFixed(1)}px) scale(${sf.toFixed(2)}) rotate(0deg)`, opacity: 1 },
    );
  } else {
    frames.push({ transform: `translate(${ex.toFixed(1)}px, ${ey - 30}px) scale(.8)`, opacity: 0 });
  }
  const anim = play(el, frames, { duration: D, fill: 'forwards' });
  waitFor(target, D);
  at(anim, D * 0.97, () => sound.play('clink', { volume: 0.5, rate: 1.2 }));
  scene(anim, () => el.remove());
}

/** A cork, 4 × 5 pixels. */
function cork(): string {
  return `<svg viewBox="0 0 4 5" width="12" height="15" shape-rendering="crispEdges"><path d="M0 0h4v4h-4z" fill="#c8955a"/><path d="M1 4h2v1h-2z" fill="#a8743c"/><path d="M0 0h1v4h-1zM3 0h1v4h-1z" fill="#8a5a2a"/><path d="M1 1h1v1h-1zM2 2h1v1h-1z" fill="#e6bd84"/></svg>`;
}

/**
 * A Schnaps bought: the bottle mat (`html`) shakes, its cork pops off with
 * a "Plopp!", and it flies into its slot of the Leiste (`target`, hidden
 * until it lands).
 */
export function schnapsToSlot(html: string, from: DOMRect, target: HTMLElement | null): void {
  const D = dur(1100);
  if (!D || !from.width) return;
  const a = centre(from);
  const el = sprite('fx-buy', html, a.x, a.y);
  el.dataset.testid = 'schnaps-fly';
  el.style.width = `${Math.round(from.width)}px`;
  el.style.height = `${Math.round(from.height)}px`;
  const rest = target ? target.getBoundingClientRect() : null;
  const fly = flightFrames(a, rest, from.width);
  const pop = 0.2;
  const frames: Keyframe[] = [
    { transform: 'translate(0, 0) scale(1)', opacity: 1 },
    { transform: 'translate(-2px, 0) scale(1.04) rotate(-4deg)', opacity: 1, offset: 0.06 },
    { transform: 'translate(2px, 0) scale(1.04) rotate(4deg)', opacity: 1, offset: 0.12 },
    { transform: 'translate(0, 2px) scale(1.12, .86)', opacity: 1, offset: pop, easing: 'cubic-bezier(.2,.8,.4,1)' },
    { transform: 'translate(0, -6px) scale(.94, 1.1)', opacity: 1, offset: 0.27 },
    { transform: 'translate(0, 0) scale(1)', opacity: 1, offset: 0.36 },
    ...fly.slice(1).map((k) => ({ ...k, offset: k.offset === undefined ? 1 : 0.36 + 0.64 * Number(k.offset) })),
  ];
  const anim = play(el, frames, { duration: D, fill: 'forwards' });
  scene(anim, () => el.remove());
  waitFor(target, D);
  // The cork flies up and off to one side, turning.
  const side = Math.random() < 0.5 ? -1 : 1;
  const c = sprite('fx-cork', cork(), a.x, from.top + 4);
  c.dataset.testid = 'schnaps-cork';
  const flight = play(
    c,
    [
      { transform: 'translate(0, 0) rotate(0deg)', opacity: 0, easing: 'steps(1, end)' },
      { transform: 'translate(0, 0) rotate(0deg)', opacity: 1, offset: 0.01, easing: 'cubic-bezier(.2,.8,.5,1)' },
      { transform: `translate(${side * 22}px, -74px) rotate(${side * 240}deg)`, opacity: 1, offset: 0.42, easing: 'cubic-bezier(.5,0,.9,.5)' },
      { transform: `translate(${side * 46}px, 26px) rotate(${side * 600}deg)`, opacity: 1, offset: 0.9 },
      { transform: `translate(${side * 50}px, 34px) rotate(${side * 640}deg)`, opacity: 0 },
    ],
    { duration: D * 0.62, delay: D * pop, fill: 'both' },
  );
  scene(flight, () => c.remove());
  // "Plopp!" next to the bottle.
  const w = sprite('fx-word', '', a.x - side * 26, from.top - 10);
  w.textContent = de.drink.plopp;
  w.dataset.testid = 'schnaps-plopp';
  const word = play(
    w,
    [
      { transform: 'translate(-50%, -50%) scale(.3)', opacity: 0, easing: 'steps(1, end)' },
      { transform: 'translate(-50%, -50%) scale(.3)', opacity: 1, offset: 0.01, easing: 'cubic-bezier(.2,.9,.3,1.3)' },
      { transform: 'translate(-50%, -50%) scale(1.1)', opacity: 1, offset: 0.25 },
      { transform: 'translate(-50%, -60%) scale(1)', opacity: 1, offset: 0.75 },
      { transform: 'translate(-50%, -80%) scale(1)', opacity: 0 },
    ],
    { duration: D * 0.55, delay: D * pop, fill: 'both' },
  );
  scene(word, () => w.remove());
  at(anim, D * pop, () => sound.play('knock', { volume: 0.5, rate: 2.4 }));
  at(anim, D * 0.97, () => sound.play('clink', { volume: 0.45, rate: 1.4 }));
}

// ---------------------------------------------------------------- the Zinke stamp

/** A rubber stamp, 12 × 14 pixels: knob, handle, block and the rubber in Zinke ink. */
function rubberStamp(): string {
  return `<svg viewBox="0 0 12 14" shape-rendering="crispEdges"><path d="M4 0h4v1h1v3h-1v1h-4v-1h-1v-3h1z" fill="#8a5a2b"/><path d="M4 1h1v2h-1z" fill="#c08a4c"/><path d="M5 5h2v3h-2z" fill="#6b3e1f"/><path d="M1 8h10v3h-10z" fill="#a46b35"/><path d="M1 8h10v1h-10z" fill="#c8955a"/><path d="M0 11h12v3h-12z" fill="${STAMP}"/><path d="M0 11h12v1h-12z" fill="#4fa58d"/></svg>`;
}

/**
 * A Zinke goes onto a card: the card (`before`, the card's face before)
 * lies at `place`, a rubber stamp comes down onto it with a "plopp" and a
 * little ink splash, and the card shows its new frame (`after`); then it
 * goes. `order` staggers several stamps of one step.
 */
export function zinkeStamp(before: string, after: string, place: DOMRect, order = 0): void {
  const D = dur(1150);
  if (!D || !place.width) return;
  const delay = order * D * 0.55;
  const card = document.createElement('span');
  card.className = 'card fx-stamp-card';
  card.dataset.testid = 'zinke-stamp-card';
  card.setAttribute('aria-hidden', 'true');
  card.innerHTML = before;
  card.style.left = `${Math.round(place.left)}px`;
  card.style.top = `${Math.round(place.top)}px`;
  card.style.width = `${Math.round(place.width)}px`;
  card.style.height = `${Math.round(place.height)}px`;
  layer().appendChild(card);
  const hit = 0.36;
  const cardAnim = play(
    card,
    [
      { transform: 'scale(.9)', opacity: 0 },
      { transform: 'scale(1)', opacity: 1, offset: 0.1 },
      { transform: 'scale(1)', opacity: 1, offset: hit, easing: 'steps(1, end)' },
      { transform: 'scale(1.04, .94)', opacity: 1, offset: hit + 0.03, easing: 'cubic-bezier(.2,.8,.4,1)' },
      { transform: 'scale(1.06)', opacity: 1, offset: 0.56 },
      { transform: 'scale(1)', opacity: 1, offset: 0.66 },
      { transform: 'scale(1)', opacity: 1, offset: 0.88, easing: 'steps(3, end)' },
      { transform: 'translateY(10px) scale(.92)', opacity: 0 },
    ],
    { duration: D, delay, fill: 'both' },
  );
  cardAnim.id = 'zinke-stamp';
  scene(cardAnim, () => card.remove());
  // The stamp: down from above, pressed a moment, lifted off again.
  const sw = Math.round(Math.max(36, Math.min(72, place.width * 0.62)));
  const sh = Math.round((sw * 14) / 12);
  const c = centre(place);
  const s = sprite('fx-stamp', rubberStamp(), c.x, c.y - sh * 0.15);
  s.dataset.testid = 'zinke-stamp';
  s.style.width = `${sw}px`;
  s.style.height = `${sh}px`;
  const up = Math.round(place.height * 0.8 + 30);
  const stamp = play(
    s,
    [
      { transform: `translate(-50%, -50%) translateY(${-up}px) scale(1.3)`, opacity: 0, easing: 'steps(1, end)' },
      { transform: `translate(-50%, -50%) translateY(${-up}px) scale(1.3)`, opacity: 1, offset: 0.1, easing: 'cubic-bezier(.6,0,.95,.5)' },
      { transform: 'translate(-50%, -50%) translateY(2px) scale(1.06, .9)', opacity: 1, offset: hit, easing: 'ease-out' },
      { transform: 'translate(-50%, -50%) scale(1)', opacity: 1, offset: 0.48, easing: 'cubic-bezier(.4,0,.7,.4)' },
      { transform: `translate(-50%, -50%) translateY(${-up * 0.6}px) scale(1.15)`, opacity: 1, offset: 0.62 },
      { transform: `translate(-50%, -50%) translateY(${-up}px) scale(1.25)`, opacity: 0, offset: 0.72 },
      { transform: `translate(-50%, -50%) translateY(${-up}px) scale(1.25)`, opacity: 0 },
    ],
    { duration: D, delay, fill: 'both' },
  );
  scene(stamp, () => s.remove());
  // On the hit: the card turns into its new look, "plopp", ink splashes.
  at(cardAnim, delay + D * hit, () => {
    card.innerHTML = after;
    sound.play('knock', { volume: 0.55, rate: 2.0 });
  });
  for (let i = 0; i < 9; i++) {
    const angle = (i / 9) * Math.PI * 2 + jitter(0.3);
    const r = place.width * (0.38 + Math.random() * 0.22);
    const drop = sprite('fx-ink', '', c.x + Math.cos(angle) * sw * 0.3, c.y + sh * 0.25 + Math.sin(angle) * 6);
    drop.style.background = i % 3 === 0 ? '#4fa58d' : STAMP;
    const splash = play(
      drop,
      [
        { transform: 'translate(0, 0) scale(1)', opacity: 0, easing: 'steps(1, end)' },
        { transform: 'translate(0, 0) scale(1)', opacity: 1, offset: 0.01, easing: 'cubic-bezier(.1,.8,.3,1)' },
        { transform: `translate(${(Math.cos(angle) * r).toFixed(1)}px, ${(Math.sin(angle) * r * 0.6).toFixed(1)}px) scale(1)`, opacity: 1, offset: 0.6 },
        { transform: `translate(${(Math.cos(angle) * r * 1.1).toFixed(1)}px, ${(Math.sin(angle) * r * 0.6 + 8).toFixed(1)}px) scale(.5)`, opacity: 0 },
      ],
      { duration: D * 0.4, delay: delay + D * hit, easing: 'steps(6, end)', fill: 'both' },
    );
    scene(splash, () => drop.remove());
  }
}

// ---------------------------------------------------------------- the Doppelgänger swap

/** A card face as a floating sprite of the scene layer at `place`. */
function floatingCard(html: string, place: DOMRect, testid: string): HTMLElement {
  const card = document.createElement('span');
  card.className = 'card fx-stamp-card fx-doppel-card';
  card.dataset.testid = testid;
  card.setAttribute('aria-hidden', 'true');
  card.innerHTML = html;
  card.style.left = `${Math.round(place.left)}px`;
  card.style.top = `${Math.round(place.top)}px`;
  card.style.width = `${Math.round(place.width)}px`;
  card.style.height = `${Math.round(place.height)}px`;
  layer().appendChild(card);
  return card;
}

/**
 * The mirror shimmer of a "Doppelgänger" (DG-A): the card at `place`
 * flashes in its violet mirror look and a band of light pixels sweeps
 * across it. Resolves when it is over (at once with reduced motion or
 * when hurried).
 */
export function mirrorShimmer(html: string, place: DOMRect): Promise<void> {
  const D = dur(560);
  if (!D || !place.width) return Promise.resolve();
  const card = floatingCard(html, place, 'doppel-shimmer');
  const sweep = document.createElement('span');
  sweep.className = 'fx-doppel-sweep';
  card.appendChild(sweep);
  const flash = play(
    card,
    [
      { filter: 'brightness(1) drop-shadow(0 0 0 #a493d6)', transform: 'scale(1)' },
      { filter: 'brightness(1.5) drop-shadow(0 0 6px #a493d6)', transform: 'scale(1.06)', offset: 0.3 },
      { filter: 'brightness(1.15) drop-shadow(0 0 4px #6a58a0)', transform: 'scale(1.03)', offset: 0.7 },
      { filter: 'brightness(1) drop-shadow(0 0 0 #6a58a0)', transform: 'scale(1)' },
    ],
    { duration: D, easing: 'steps(8, end)', fill: 'both' },
  );
  play(sweep, [{ transform: 'translateX(-120%)' }, { transform: 'translateX(220%)' }], { duration: D * 0.8, easing: 'steps(10, end)', fill: 'both' });
  scene(flash, () => card.remove());
  sound.play('clink', { volume: 0.4, rate: 1.9 });
  return flash.finished.then(
    () => undefined,
    () => undefined,
  );
}

/**
 * The reach of a "Doppelgänger": a dotted line of pixels from the card
 * at `from` to the robbed opponent's portrait at `to`, drawn out and
 * faded again.
 */
export function doppelReach(from: DOMRect, to: DOMRect): void {
  const D = dur(620);
  if (!D || !from.width || !to.width) return;
  const a = centre(from);
  const b = centre(to);
  const steps = Math.max(4, Math.round(Math.hypot(b.x - a.x, b.y - a.y) / 12));
  for (let i = 1; i < steps; i++) {
    const p = i / steps;
    const dot = sprite('fx-doppel-dot', '', px4(a.x + (b.x - a.x) * p), px4(a.y + (b.y - a.y) * p));
    dot.dataset.testid = 'doppel-reach';
    const anim = play(
      dot,
      [
        { opacity: 0, transform: 'scale(.4)' },
        { opacity: 1, transform: 'scale(1)', offset: 0.25 },
        { opacity: 1, transform: 'scale(1)', offset: 0.7 },
        { opacity: 0, transform: 'scale(.6)' },
      ],
      { duration: D * 0.7, delay: D * 0.3 * p, easing: 'steps(4, end)', fill: 'both' },
    );
    scene(anim, () => dot.remove());
  }
}

/**
 * One card of the swap flying in a high arc from `from` to `to` (the
 * cards cross mid-air: give the two flights arcs of opposite sign). With
 * `shrink` it disappears into the hand it lands in. Resolves on landing.
 */
export function swapFlight(html: string, from: DOMRect, to: DOMRect, arc: number, testid: string, shrink = false): Promise<void> {
  const D = dur(640);
  if (!D || !from.width || !to.width) return Promise.resolve();
  const card = floatingCard(html, from, testid);
  const dx = to.left + to.width / 2 - (from.left + from.width / 2);
  const dy = to.top + to.height / 2 - (from.top + from.height / 2);
  const end = shrink ? 0.35 : Math.max(0.4, Math.min(2, to.width / from.width));
  const flight = play(
    card,
    [
      { transform: 'translate(0, 0) rotate(0deg) scale(1)', opacity: 1 },
      { transform: `translate(${(dx * 0.5).toFixed(1)}px, ${(dy * 0.5 + arc).toFixed(1)}px) rotate(${arc > 0 ? 8 : -8}deg) scale(${((1 + end) / 2).toFixed(2)})`, opacity: 1, offset: 0.5 },
      { transform: `translate(${dx.toFixed(1)}px, ${dy.toFixed(1)}px) rotate(0deg) scale(${end.toFixed(2)})`, opacity: shrink ? 0 : 1 },
    ],
    { duration: D, easing: 'cubic-bezier(.3,.1,.3,1)', fill: 'both' },
  );
  scene(flight, () => card.remove());
  return flight.finished.then(
    () => undefined,
    () => undefined,
  );
}

/** A short pixel word ("Getauscht!", "Kein Gegenstück") popping up at `at` and fading. */
export function sceneWord(text: string, at: DOMRect, testid: string): void {
  const D = dur(900);
  if (!D || !at.width) return;
  const c = centre(at);
  const word = sprite('fx-doppel-word', '', c.x, c.y);
  word.textContent = text;
  word.dataset.testid = testid;
  const anim = play(
    word,
    [
      { transform: 'translate(-50%, -50%) translateY(6px) scale(.6)', opacity: 0 },
      { transform: 'translate(-50%, -50%) scale(1.15)', opacity: 1, offset: 0.2 },
      { transform: 'translate(-50%, -50%) scale(1)', opacity: 1, offset: 0.7 },
      { transform: 'translate(-50%, -50%) translateY(-10px) scale(1)', opacity: 0 },
    ],
    { duration: D, easing: 'steps(9, end)', fill: 'both' },
  );
  scene(anim, () => word.remove());
}

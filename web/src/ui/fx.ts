// Visual effects with the Web Animations API: cards flying in, tricks
// sliding off towards the winner, dealing (a little different each time),
// cards turning over, number count-ups. Every effect respects
// the tempo setting and prefers-reduced-motion, and can be finished at
// once (a tap hurries the table). Effects never change what is shown,
// only how it gets there.

import { reducedMotion } from './dom';

let tempo = 1;
const running = new Set<Animation>();

/** Factor for all durations (Gemütlich 1.5, Normal 1, Flott 0.5). */
export function setTempo(factor: number): void {
  tempo = factor;
}

/** The tempo factor in force (for effects timed outside the Web Animations API). */
export function tempoFactor(): number {
  return tempo;
}

/** Whether flights and count-ups play at all. */
export function motionOn(): boolean {
  return !reducedMotion();
}

/** A duration scaled by tempo; 0 with reduced motion. */
export function dur(ms: number): number {
  return motionOn() ? ms * tempo : 0;
}

function track(anim: Animation): Animation {
  running.add(anim);
  const done = (): void => {
    running.delete(anim);
  };
  anim.addEventListener('finish', done);
  anim.addEventListener('cancel', done);
  return anim;
}

/**
 * Run `frame(p)` every animation frame for `ms` (tempo-scaled), p from 0 to
 * 1, eased out. Driven by a target-less WAAPI timer, so finishAll() ends it
 * at once with frame(1). Without motion only frame(1) runs.
 */
export function tween(ms: number, frame: (p: number) => void): Promise<void> {
  const duration = dur(ms);
  if (!duration) {
    frame(1);
    return Promise.resolve();
  }
  const timer = track(new Animation(new KeyframeEffect(null, [], duration), document.timeline));
  timer.play();
  return new Promise((resolve) => {
    let done = false;
    const end = (): void => {
      if (done) return;
      done = true;
      frame(1);
      resolve();
    };
    const tick = (): void => {
      if (done) return;
      const p = timer.playState === 'finished' ? 1 : Math.max(0, Math.min(1, Number(timer.currentTime ?? 0) / duration));
      if (p >= 1) {
        end();
        return;
      }
      frame(1 - (1 - p) ** 2);
      requestAnimationFrame(tick);
    };
    requestAnimationFrame(tick);
    timer.finished.then(end, end);
  });
}

/** Play keyframes on `el` as a tracked effect (finishAll() ends it). */
export function play(el: Element, frames: Keyframe[], opts: KeyframeAnimationOptions): Animation {
  return track(el.animate(frames, opts));
}

/** Jump every running effect to its end. */
export function finishAll(): void {
  for (const anim of Array.from(running)) {
    try {
      anim.finish();
    } catch {
      anim.cancel();
    }
  }
}

function center(r: DOMRect): { x: number; y: number } {
  return { x: r.left + r.width / 2, y: r.top + r.height / 2 };
}

/**
 * Animate `el` from the place of `from` into its own place. `arc` bends the
 * path sideways by that many px at its middle (a card thrown, not slid).
 */
export function flyIn(el: Element, from: DOMRect, opts: { ms?: number; delay?: number; rotate?: number; arc?: number } = {}): Animation | null {
  const duration = dur(opts.ms ?? 280);
  if (!duration) return null;
  const to = el.getBoundingClientRect();
  if (!to.width) return null;
  const a = center(from);
  const b = center(to);
  const scale = Math.max(0.2, Math.min(2, from.width / to.width));
  const rotate = opts.rotate ?? 0;
  const dx = a.x - b.x;
  const dy = a.y - b.y;
  const frames: Keyframe[] = [{ transform: `translate(${dx}px, ${dy}px) scale(${scale}) rotate(${rotate}deg)`, opacity: 0.85 }];
  if (opts.arc) {
    const len = Math.hypot(dx, dy) || 1;
    const mx = dx / 2 + (-dy / len) * opts.arc;
    const my = dy / 2 + (dx / len) * opts.arc;
    frames.push({ transform: `translate(${mx}px, ${my}px) scale(${(scale + 1) / 2}) rotate(${rotate / 3}deg)`, opacity: 1, offset: 0.55 });
  }
  frames.push({ transform: 'none', opacity: 1 });
  // A card of the fanned hand starts upright and turns into the fan on the way.
  const turn = el instanceof HTMLElement ? getComputedStyle(el).rotate : 'none';
  if (turn && turn !== 'none') {
    (frames[0] as Keyframe).rotate = '0deg';
    (frames[frames.length - 1] as Keyframe).rotate = turn;
  }
  return track(el.animate(frames, { duration, delay: dur(opts.delay ?? 0), easing: 'cubic-bezier(.2,.75,.25,1)', fill: 'backwards' }));
}

/** A random number in [-n, n]: cosmetic only (Math.random, never the core's RNG). */
export function jitter(n: number): number {
  return (Math.random() * 2 - 1) * n;
}

/**
 * Deal: cards fly in from the deck, one after another. Every deal looks a
 * little different: the start point, the spin, the curve of the path and
 * the rhythm vary at random (cosmetic, no game state).
 */
export function deal(cards: Element[], from: DOMRect, opts: { step?: number; ms?: number; onCard?: (i: number) => void } = {}): void {
  if (!motionOn()) return;
  const step = opts.step ?? 45;
  const ms = opts.ms ?? 300;
  let at = 0;
  cards.forEach((el, i) => {
    const start = new DOMRect(from.x + jitter(9), from.y + jitter(7), from.width, from.height);
    const anim = flyIn(el, start, { ms: ms * (0.85 + Math.random() * 0.4), delay: at, rotate: jitter(40), arc: jitter(28) });
    if (anim && opts.onCard) {
      const cb = opts.onCard;
      window.setTimeout(() => cb(i), dur(at));
    }
    // Uneven rhythm, like a hand that deals: now and then a short pause.
    at += step * (0.6 + Math.random() * 0.8) + (i % 3 === 2 ? step * Math.random() : 0);
  });
}

/**
 * A finished trick leaves the table towards the seat that took it: the
 * cards fly together onto the card that took it (`onto`, an index into
 * `els`; default the middle one) and are lifted a little as one stack,
 * then the stack slides off to `to` (the winner's seat, or beyond the
 * edge of the table on his side), turning and shrinking, fading at the
 * end. Resolves when done.
 */
export async function sweepAway(els: Element[], to: { x: number; y: number }, ms = 460, onto?: number): Promise<void> {
  const duration = dur(ms);
  if (!duration || !els.length) return;
  const rects = els.map((el) => el.getBoundingClientRect());
  const top = onto !== undefined && onto >= 0 && onto < rects.length ? onto : Math.floor(rects.length / 2);
  const mid = rects[top] ?? rects[0];
  if (!mid) return;
  const g = center(mid);
  const spin = to.x < g.x - 20 ? -1 : to.x > g.x + 20 ? 1 : 0;
  const lastTilt = spin * 16 + jitter(6);
  // The stack lies a little fanned: each card a step off, the one that took it on top.
  let order = 0;
  const anims = els.map((el, i) => {
    const r = rects[i] ?? mid;
    const a = center(r);
    const step = i === top ? 0 : ++order;
    const gx = g.x - a.x + (step ? (step % 2 ? -1 : 1) * 3 : 0);
    const gy = g.y - a.y + step * 2;
    const tilt = step ? jitter(8) : jitter(2);
    const sx = gx + to.x - g.x;
    const sy = gy + to.y - g.y;
    // The slots stack the cards; the one that took the trick goes on top.
    const slot = el.closest('.slot');
    if (slot instanceof HTMLElement) slot.style.zIndex = String(i === top ? 9 : 8 - step);
    return track(
      el.animate(
        [
          // Together quickly, a beat lifted as one stack, then off with growing speed.
          { transform: 'none', opacity: 1, easing: 'cubic-bezier(.2,.7,.3,1)' },
          { transform: `translate(${gx}px, ${gy}px) rotate(${tilt}deg) scale(1.06)`, opacity: 1, offset: 0.3 },
          { transform: `translate(${gx}px, ${gy - 3}px) rotate(${tilt}deg) scale(1.06)`, opacity: 1, offset: 0.4, easing: 'cubic-bezier(.55,0,.8,.45)' },
          { transform: `translate(${sx}px, ${sy}px) rotate(${lastTilt + tilt}deg) scale(0.55)`, opacity: 0 },
        ],
        { duration, delay: step * dur(16), fill: 'forwards' },
      ),
    );
  });
  for (const a of anims) a.id = 'sweep';
  await Promise.all(anims.map((a) => a.finished.catch(() => undefined)));
}

/**
 * Turn a card over: it narrows to an edge showing `back` (HTML of a card
 * back laid over it), and widens again with its face. Plain 2D, so it
 * looks the same in every browser.
 */
export function flip(el: HTMLElement, back: string, opts: { ms?: number; delay?: number } = {}): void {
  const duration = dur(opts.ms ?? 320);
  if (!duration) return;
  const delay = dur(opts.delay ?? 0);
  const cover = document.createElement('span');
  cover.className = 'flip-cover';
  cover.setAttribute('aria-hidden', 'true');
  cover.innerHTML = back;
  el.appendChild(cover);
  track(
    el.animate([{ transform: 'scaleX(1)' }, { transform: 'scaleX(0.04)', offset: 0.5 }, { transform: 'scaleX(1)' }], {
      duration,
      delay,
      easing: 'ease-in-out',
    }),
  );
  const hide = track(cover.animate([{ opacity: 1 }, { opacity: 1, offset: 0.5 }, { opacity: 0, offset: 0.5 }, { opacity: 0 }], { duration, delay, fill: 'both' }));
  const gone = (): void => cover.remove();
  hide.finished.then(gone, gone);
}

/**
 * Let a block that was just shown open up from zero height, so the content
 * below slides down instead of jumping. `gap` is the flex gap in front of
 * the block, which opens up too. Resolves when the block has its height.
 */
export function grow(el: HTMLElement, opts: { ms?: number; gap?: number; fade?: boolean } = {}): Promise<void> {
  const duration = dur(opts.ms ?? 180);
  if (!duration) return Promise.resolve();
  const cs = getComputedStyle(el);
  const height = el.getBoundingClientRect().height;
  if (!height) return Promise.resolve();
  const gap = opts.gap ?? 0;
  const from: Keyframe = {
    height: '0px',
    minHeight: '0px',
    paddingTop: '0px',
    paddingBottom: '0px',
    borderTopWidth: '0px',
    borderBottomWidth: '0px',
    marginTop: `${-gap}px`,
  };
  const to: Keyframe = {
    height: `${height}px`,
    minHeight: '0px',
    paddingTop: cs.paddingTop,
    paddingBottom: cs.paddingBottom,
    borderTopWidth: cs.borderTopWidth,
    borderBottomWidth: cs.borderBottomWidth,
    marginTop: cs.marginTop,
  };
  if (opts.fade) {
    from.opacity = 0;
    to.opacity = 0;
  }
  el.style.overflow = 'hidden';
  const anim = track(el.animate([from, to], { duration, easing: 'cubic-bezier(.25,.8,.35,1)' }));
  const done = (): void => {
    el.style.overflow = '';
  };
  return anim.finished.then(done, done);
}

/** A short bump to draw the eye. */
export function pulse(el: Element, scale = 1.18, ms = 260): void {
  const duration = dur(ms);
  if (!duration) return;
  track(
    el.animate([{ transform: 'scale(1)' }, { transform: `scale(${scale})` }, { transform: 'scale(1)' }], {
      duration,
      easing: 'ease-out',
    }),
  );
}

/** The result lands: big, then settles; the board shakes a little. */
export function slam(el: Element, board: Element | null): void {
  const duration = dur(420);
  if (!duration) return;
  track(
    el.animate(
      [
        { transform: 'scale(2.4)', opacity: 0 },
        { transform: 'scale(0.92)', opacity: 1, offset: 0.6 },
        { transform: 'scale(1)', opacity: 1 },
      ],
      { duration, easing: 'cubic-bezier(.3,.6,.4,1)', fill: 'backwards' },
    ),
  );
  if (board) {
    track(
      board.animate(
        [
          { transform: 'none' },
          { transform: 'translate(-3px, 2px)', offset: 0.65 },
          { transform: 'translate(3px, -2px)', offset: 0.75 },
          { transform: 'translate(-2px, 1px)', offset: 0.85 },
          { transform: 'none' },
        ],
        { duration: dur(420), delay: duration * 0.5 },
      ),
    );
  }
}

/** Count a number shown in `el` from `from` to `to`. */
export function countUp(el: HTMLElement, from: number, to: number, ms: number, format: (n: number) => string = String): Promise<void> {
  const duration = dur(ms);
  if (!duration || from === to) {
    el.textContent = format(to);
    return Promise.resolve();
  }
  // Driven by a WAAPI timer, so finishAll() ends it too.
  const timer = track(el.animate([{ opacity: 1 }, { opacity: 1 }], { duration }));
  return new Promise((resolve) => {
    // The frame time of requestAnimationFrame can lie before the moment the
    // count started (and far before it after a stalled tab), so the progress
    // is clamped; outside 0..1 the easing would overshoot wildly.
    const start = performance.now();
    const frame = (now: number): void => {
      const p = timer.playState === 'finished' ? 1 : Math.max(0, Math.min(1, (now - start) / duration));
      el.textContent = format(Math.round(from + (to - from) * (1 - (1 - p) ** 2)));
      if (p < 1) requestAnimationFrame(frame);
      else resolve();
    };
    requestAnimationFrame(frame);
    timer.finished.then(
      () => {
        el.textContent = format(to);
        resolve();
      },
      () => resolve(),
    );
  });
}

/** How a card is thrown onto the felt (./table.ts places it in its slot). */
export interface ThrowOpts {
  /** Where it starts: the hand card, the copy let go, an opponent's portrait. */
  from: DOMRect;
  /** Its turn at the start, in degrees (the copy's tilt). */
  fromRotate?: number;
  /**
   * A point (screen) the path bends towards on its way to the place: where
   * a flick sent the card. Default: a gentle arc to one side.
   */
  toward?: { x: number; y: number };
  /** Turns on the way, in degrees (sign = direction); a small turn, it settles to straight. */
  spin: number;
  /** The whole flight (ms at tempo Normal). */
  ms: number;
  /** HTML of a back it shows for the first part of the flight (an opponent's card turning over). */
  back?: string;
}

/**
 * The progress of a thrown card over time (playtest 12.11.2026: the old
 * throw flew half way, lay still and slid on, which read as a hitch). One
 * curve from start to place: it leaves at twice the mean speed like an
 * ease-out quad (steeper and the first frame already covers a third of the
 * way, playtest 04.11.2026), slows down without a stop, runs about one per
 * cent past its place at four fifths of the time and settles back onto it
 * with no speed left: 1 − (1 − t)² + 2.5 t³ (1 − t)².
 */
export function throwProgress(t: number): number {
  if (t <= 0) return 0;
  if (t >= 1) return 1;
  const u = 1 - t;
  return 1 - u * u + 2.5 * t * t * t * u * u;
}

/** How many keyframes a throw is drawn with: about one per frame at 60 Hz, linear in between. */
const THROW_STEPS = 24;

/**
 * Throw a card onto the felt: one flight from `from` onto its place along
 * a gentle curve (towards where a flick sent it, else a little to one
 * side), lifted a touch in the middle, turning a little and settling
 * straight; it runs a hair past its place and sets down on it. No stop on
 * the way. The card stays in its slot in the DOM all the time; only its
 * transform moves (precomputed keyframes, nothing measured per frame).
 * Cosmetic only: the jitter comes from Math.random.
 */
export function throwCard(el: HTMLElement, o: ThrowOpts): Animation | null {
  const total = dur(o.ms);
  if (!total) return null;
  const to = el.getBoundingClientRect();
  if (!to.width) return null;
  const b = center(to);
  const a = center(o.from);
  const dx = b.x - a.x;
  const dy = b.y - a.y;
  const len = Math.hypot(dx, dy) || 1;
  // The bend: towards the flick's point, or sideways by a share of the way.
  const side = o.spin === 0 ? (Math.random() < 0.5 ? -1 : 1) : Math.sign(o.spin);
  const bend = Math.min(56, len * 0.16) * side;
  const ctrl = o.toward ?? { x: a.x + dx / 2 - (dy / len) * bend, y: a.y + dy / 2 + (dx / len) * bend };
  const scale0 = Math.max(0.3, Math.min(2, o.from.width / to.width));
  const r0 = (o.fromRotate ?? 0) - o.spin;
  const frames: Keyframe[] = [];
  for (let i = 0; i <= THROW_STEPS; i++) {
    const t = i / THROW_STEPS;
    const p = throwProgress(t);
    // Along the quadratic Bézier a → ctrl → b; past 1 it runs on along the arrival.
    const u = Math.min(1, p);
    const over = p - u;
    const x = (1 - u) * (1 - u) * a.x + 2 * (1 - u) * u * ctrl.x + u * u * b.x + over * 2 * (b.x - ctrl.x);
    const y = (1 - u) * (1 - u) * a.y + 2 * (1 - u) * u * ctrl.y + u * u * b.y + over * 2 * (b.y - ctrl.y);
    const lift = 1 + 0.06 * Math.sin(Math.PI * Math.min(1, t * 1.15));
    const s = (scale0 + (1 - scale0) * p) * lift;
    const r = r0 * (1 - p);
    frames.push({
      transform:
        i === THROW_STEPS
          ? 'translate(0px, 0px) rotate(0deg) scale(1)'
          : `translate(${(x - b.x).toFixed(1)}px, ${(y - b.y).toFixed(1)}px) rotate(${r.toFixed(2)}deg) scale(${s.toFixed(3)})`,
      offset: t,
    });
  }
  const anim = track(el.animate(frames, { duration: total, easing: 'linear', fill: 'backwards' }));
  anim.id = 'throw';
  if (o.back) {
    // Turned over in the first part of the flight: the back narrows to an edge, the face widens.
    const cover = document.createElement('span');
    cover.className = 'flip-cover';
    cover.setAttribute('aria-hidden', 'true');
    cover.innerHTML = o.back;
    el.appendChild(cover);
    const turned = 0.55;
    const mid = turned / 2;
    track(
      el.animate(
        [
          { transform: 'scaleX(1)', offset: 0 },
          { transform: 'scaleX(0.06)', offset: mid },
          { transform: 'scaleX(1)', offset: turned },
          { transform: 'scaleX(1)', offset: 1 },
        ],
        { duration: total, composite: 'add' },
      ),
    );
    const hide = track(cover.animate([{ opacity: 1 }, { opacity: 1, offset: mid }, { opacity: 0, offset: mid }, { opacity: 0 }], { duration: total, fill: 'both' }));
    const gone = (): void => cover.remove();
    hide.finished.then(gone, gone);
  }
  return anim;
}

/**
 * A card that may not be played goes back: it glides from `from` (where
 * it was let go) into its place in the hand, then shakes its head.
 */
export function springBack(el: HTMLElement, from: DOMRect | null, shake: boolean): void {
  const back = from ? dur(220) : 0;
  if (from && back) flyIn(el, from, { ms: 220 });
  if (!shake || !motionOn()) return;
  track(
    el.animate(
      [
        { transform: 'translateX(0)' },
        { transform: 'translateX(-7px)', offset: 0.2 },
        { transform: 'translateX(6px)', offset: 0.4 },
        { transform: 'translateX(-4px)', offset: 0.6 },
        { transform: 'translateX(3px)', offset: 0.8 },
        { transform: 'translateX(0)' },
      ],
      { duration: 300, delay: back, composite: 'add' },
    ),
  );
}

/** Running animations of the cards on the felt, kept across a re-render. */
export interface FlightSnapshot {
  id: string;
  name: string;
  cover: string | null;
  anims: { onCover: boolean; frames: Keyframe[]; timing: EffectTiming; composite: CompositeOperation; time: number }[];
  /** When it was noted (ms): the render in between does not hold the flight up. */
  at: number;
}

/** The key a card keeps across a re-render: its card id, or a place name (`data-fly`). */
function keyOf(el: HTMLElement): string | undefined {
  const k = el.dataset.cardId ?? el.dataset.id ?? el.dataset.fly;
  return k === undefined ? undefined : `${el.classList.contains('skat-card') ? 's' : ''}${k}`;
}

/** Transitions and animations of the stylesheet: the new element runs its own. */
function cssDriven(a: Animation): boolean {
  return (typeof CSSTransition !== 'undefined' && a instanceof CSSTransition) || (typeof CSSAnimation !== 'undefined' && a instanceof CSSAnimation);
}

/**
 * The app re-renders the table from scratch while cards still fly or
 * settle (the next opponent thinks). Before a render, note what runs on
 * the cards matching `selector` (keyed by `data-card-id`); after it,
 * `resumeFlights` starts the same animations on the new elements where
 * they were.
 */
export function noteFlights(root: Element, selector: string): FlightSnapshot[] {
  if (!motionOn()) return [];
  const out: FlightSnapshot[] = [];
  for (const el of Array.from(root.querySelectorAll<HTMLElement>(selector))) {
    const id = keyOf(el);
    const anims = el.getAnimations({ subtree: true }).filter((x) => x.playState === 'running' && x.effect instanceof KeyframeEffect && !cssDriven(x));
    if (!id || !anims.length) continue;
    const cover = el.querySelector(':scope > .flip-cover');
    const snap: FlightSnapshot = { id, name: anims.find((x) => x.id)?.id ?? '', cover: cover ? cover.innerHTML : null, anims: [], at: performance.now() };
    for (const x of anims) {
      const effect = x.effect as KeyframeEffect;
      const onCover = effect.target !== el;
      if (onCover && effect.target !== cover) continue;
      snap.anims.push({ onCover, frames: effect.getKeyframes(), timing: effect.getTiming(), composite: effect.composite, time: Number(x.currentTime ?? 0) });
      x.cancel();
    }
    out.push(snap);
  }
  return out;
}

/** Continue the animations noted before a render on the new elements. */
export function resumeFlights(root: Element, selector: string, notes: FlightSnapshot[]): void {
  for (const n of notes) {
    const el = Array.from(root.querySelectorAll<HTMLElement>(selector)).find((x) => keyOf(x) === n.id);
    if (!el) continue;
    let cover: HTMLElement | null = null;
    if (n.cover !== null) {
      cover = document.createElement('span');
      cover.className = 'flip-cover';
      cover.setAttribute('aria-hidden', 'true');
      cover.innerHTML = n.cover;
      el.appendChild(cover);
    }
    for (const a of n.anims) {
      const target = a.onCover ? cover : el;
      if (!target) continue;
      const frames = a.frames.map((f) => {
        const { computedOffset: _c, ...rest } = f as Keyframe & { computedOffset?: number };
        return rest;
      });
      const anim = track(target.animate(frames, { ...(a.timing as KeyframeAnimationOptions), composite: a.composite }));
      // On from where it would be now: the time the render took passes in the flight too (no hitch).
      anim.currentTime = a.time + Math.max(0, performance.now() - n.at);
      if (!a.onCover && a.composite !== 'add') anim.id = n.name;
      if (a.onCover && cover) {
        const c = cover;
        const gone = (): void => c.remove();
        anim.finished.then(gone, gone);
      }
    }
  }
}

/** Jump the animations on these elements (and inside them) to their end. */
export function finishOn(els: Element[]): void {
  for (const el of els) {
    for (const a of el.getAnimations({ subtree: true })) {
      try {
        a.finish();
      } catch {
        a.cancel();
      }
    }
  }
}

/** Where a card is on the screen: its centre, its width as seen, its turn in degrees. */
export interface Spot {
  x: number;
  y: number;
  w: number;
  turn: number;
}

/** The turn of a computed `transform` (a matrix or "none"), in degrees. */
function matrixOf(transform: string): DOMMatrix {
  try {
    return new DOMMatrix(transform === 'none' ? undefined : transform);
  } catch {
    return new DOMMatrix();
  }
}

function turnOf(m: DOMMatrix): number {
  return (Math.atan2(m.b, m.a) * 180) / Math.PI;
}

/** The computed `rotate` property in degrees ("none" or "7deg"; only turns in the plane occur here). */
function rotateOf(cs: CSSStyleDeclaration): number {
  const v = cs.rotate;
  if (!v || v === 'none') return 0;
  const n = parseFloat(v.split(' ').pop() ?? '0');
  return Number.isFinite(n) ? n : 0;
}

function translateOf(cs: CSSStyleDeclaration): { x: number; y: number } {
  const v = cs.translate;
  if (!v || v === 'none') return { x: 0, y: 0 };
  const [x, y] = v.split(' ').map((p) => parseFloat(p));
  return { x: Number.isFinite(x) ? (x as number) : 0, y: Number.isFinite(y) ? (y as number) : 0 };
}

function scaleOf(cs: CSSStyleDeclaration): number {
  const v = cs.scale;
  if (!v || v === 'none') return 1;
  const n = parseFloat(v);
  return Number.isFinite(n) ? n : 1;
}

/** The turn its parent adds (a skat card lies in a slot turned askew). */
function parentTurn(el: HTMLElement): number {
  const p = el.parentElement;
  return p ? turnOf(matrixOf(getComputedStyle(p).transform)) : 0;
}

/**
 * Where `el` is seen right now, a running flight included (so a card
 * caught mid-air flies on from there, not from where it set out).
 */
export function spotOf(el: HTMLElement): Spot {
  const r = el.getBoundingClientRect();
  const cs = getComputedStyle(el);
  const m = matrixOf(cs.transform);
  const w = (el.offsetWidth || r.width) * scaleOf(cs) * Math.hypot(m.a, m.b);
  // The box of a turned rectangle has the same centre as the rectangle.
  return { x: r.left + r.width / 2, y: r.top + r.height / 2, w, turn: rotateOf(cs) + turnOf(m) + parentTurn(el) };
}

/** The upright box of `el` around its centre as seen (a fanned hand card without its turn). */
export function boxOf(el: HTMLElement): DOMRect {
  const s = spotOf(el);
  const h = el.offsetWidth ? (s.w * el.offsetHeight) / el.offsetWidth : el.getBoundingClientRect().height;
  return new DOMRect(s.x - s.w / 2, s.y - h / 2, s.w, h);
}

/** A spot from a plain box (a card let go after dragging, upright). */
export function spotOfRect(r: DOMRect, turn = 0): Spot {
  return { x: r.left + r.width / 2, y: r.top + r.height / 2, w: r.width, turn };
}

/** Spots of the cards under `root` matching `selector`, by `data-id`. */
export function spotsOf(root: ParentNode, selector: string): Map<string, Spot> {
  const out = new Map<string, Spot>();
  for (const el of Array.from(root.querySelectorAll<HTMLElement>(selector))) {
    const id = el.dataset.id;
    if (id !== undefined) out.set(id, spotOf(el));
  }
  return out;
}

function rot(v: { x: number; y: number }, deg: number): { x: number; y: number } {
  const a = (deg * Math.PI) / 180;
  return { x: v.x * Math.cos(a) - v.y * Math.sin(a), y: v.x * Math.sin(a) + v.y * Math.cos(a) };
}

export interface GlideOpts {
  ms?: number;
  delay?: number;
  /** Lifted off the table on the way: px upwards at the middle (and a little bigger). */
  lift?: number;
  /** Bends the path sideways by that many px at its middle. */
  arc?: number;
  /**
   * HTML laid over the card for the first half of the turn: a back (the
   * card turns face up) or a face (it turns face down onto its back).
   */
  cover?: string;
  /** With `cover`: turn over (default), or keep the cover and fade it out at the end (a big face landing as a small one). */
  turn?: boolean;
  /** Paint above its neighbours (and the felt) while it flies. */
  above?: boolean;
}

/**
 * Move a card from `from` into its own place (FLIP): it keeps its slot in
 * the DOM, only `translate`, `rotate` and `scale` move, ending exactly at
 * the values its CSS gives it (the fan of the hand, a lifted card). With
 * `lift` and `arc` it is picked up and thrown in a curve, with `cover` it
 * turns over on the way. Whatever ran on the card before stops: it was
 * measured where it was seen.
 */
export function glide(el: HTMLElement, from: Spot, opts: GlideOpts = {}): Animation | null {
  const duration = dur(opts.ms ?? 300);
  if (!duration) return null;
  for (const a of el.getAnimations({ subtree: true })) if (!cssDriven(a)) a.cancel();
  el.querySelectorAll(':scope > .flip-cover').forEach((c) => c.remove());
  if (!el.offsetWidth) return null;
  const to = spotOf(el);
  const cs = getComputedStyle(el);
  const m = matrixOf(cs.transform);
  const phi = parentTurn(el);
  const restRot = rotateOf(cs);
  const restTr = translateOf(cs);
  const restScale = scaleOf(cs);
  // The centre seen from the transform origin, through the card's own transform.
  const [ox = 0, oy = 0] = cs.transformOrigin.split(' ').map((p) => parseFloat(p));
  const c0 = { x: el.offsetWidth / 2 - ox, y: el.offsetHeight / 2 - oy };
  const q = { x: m.a * c0.x + m.c * c0.y, y: m.b * c0.x + m.d * c0.y };
  const restQ = rot({ x: q.x * restScale, y: q.y * restScale }, restRot);
  // The translate that puts the centre at (x, y) with this turn and scale.
  const place = (x: number, y: number, turn: number, scale: number): Keyframe => {
    const theta = turn - phi - turnOf(m);
    const qq = rot({ x: q.x * scale, y: q.y * scale }, theta);
    const d = rot({ x: x - to.x, y: y - to.y }, -phi);
    const tx = restTr.x + restQ.x - qq.x + d.x;
    const ty = restTr.y + restQ.y - qq.y + d.y;
    return { translate: `${tx.toFixed(1)}px ${ty.toFixed(1)}px`, rotate: `${theta.toFixed(2)}deg`, scale: scale.toFixed(3) };
  };
  const s0 = Math.max(0.2, Math.min(3, (from.w / (to.w || 1)) * restScale));
  const frames: Keyframe[] = [{ ...place(from.x, from.y, from.turn, s0), easing: 'cubic-bezier(.3,.1,.4,1)' }];
  if (opts.lift || opts.arc) {
    const dx = to.x - from.x;
    const dy = to.y - from.y;
    const len = Math.hypot(dx, dy) || 1;
    const arc = opts.arc ?? 0;
    const lift = opts.lift ?? 0;
    const mx = from.x + dx * 0.5 + (-dy / len) * arc;
    const my = from.y + dy * 0.5 + (dx / len) * arc - lift;
    const ms = Math.max(s0, restScale) * (1 + Math.min(0.12, lift / 200));
    frames.push({ ...place(mx, my, (from.turn + to.turn) / 2 + Math.sign(arc || dx) * 4, ms), offset: 0.5, easing: 'cubic-bezier(.4,0,.3,1)' });
  }
  frames.push({ translate: `${restTr.x}px ${restTr.y}px`, rotate: `${restRot}deg`, scale: String(restScale) });
  if (opts.above) for (const f of frames) f.zIndex = '20';
  const delay = dur(opts.delay ?? 0);
  const anim = track(el.animate(frames, { duration, delay, fill: 'backwards' }));
  anim.id = 'glide';
  if (opts.cover !== undefined) {
    // Turned over half way: the cover narrows to an edge, the other side widens.
    const cover = document.createElement('span');
    cover.className = 'flip-cover';
    cover.setAttribute('aria-hidden', 'true');
    cover.innerHTML = opts.cover;
    el.appendChild(cover);
    if (opts.turn === false) {
      const fade = track(cover.animate([{ opacity: 1 }, { opacity: 1, offset: 0.7 }, { opacity: 0 }], { duration, delay, fill: 'both' }));
      const gone = (): void => cover.remove();
      fade.finished.then(gone, gone);
      return anim;
    }
    track(
      el.animate(
        [
          { transform: 'scaleX(1)', offset: 0 },
          { transform: 'scaleX(1)', offset: 0.2 },
          { transform: 'scaleX(0.05)', offset: 0.45 },
          { transform: 'scaleX(1)', offset: 0.7 },
          { transform: 'scaleX(1)', offset: 1 },
        ],
        { duration, delay, composite: 'add' },
      ),
    );
    const hide = track(cover.animate([{ opacity: 1 }, { opacity: 1, offset: 0.45 }, { opacity: 0, offset: 0.45 }, { opacity: 0 }], { duration, delay, fill: 'both' }));
    const gone = (): void => cover.remove();
    hide.finished.then(gone, gone);
  }
  return anim;
}

/**
 * FLIP for a row of cards laid out anew: each found in `before` (by
 * `data-id`) glides from there into its new place; `skip` are left to the
 * caller (the card picked up flies its own way).
 */
export function glideAll(els: HTMLElement[], before: Map<string, Spot>, opts: { ms?: number; skip?: Set<string> } = {}): void {
  if (!motionOn()) return;
  for (const el of els) {
    const id = el.dataset.id;
    if (id === undefined || opts.skip?.has(id)) continue;
    const from = before.get(id);
    if (!from) continue;
    const flying = el.getAnimations().some((a) => a.id === 'glide');
    const now = spotOf(el);
    if (!flying && Math.abs(now.x - from.x) < 1 && Math.abs(now.y - from.y) < 1 && Math.abs(now.turn - from.turn) < 0.5 && Math.abs(now.w - from.w) < 1) continue;
    glide(el, from, { ms: opts.ms ?? 240 });
  }
}

/**
 * The Ansage lands: a pixel sign with the game's name (`html`, from
 * de.ts) slams onto the felt at the middle of `at`, bounces, raises a
 * little dust, lies a moment and then shrinks into `into` (the game line,
 * where the same name stays). It lives in `layer`, outside the table's
 * re-renders, never takes a tap, and goes when done or hurried
 * (finishAll); `onHit` runs as it hits the felt (a knock). Returns the
 * time until it lies on the felt, 0 if not shown.
 */
export function declareSign(layer: HTMLElement, html: string, at: DOMRect, into: DOMRect | null, opts: { ms?: number; onHit?: () => void } = {}): number {
  const duration = dur(opts.ms ?? 1500);
  if (!duration || !at.width) return 0;
  const sign = document.createElement('div');
  sign.className = 'decl-sign';
  sign.setAttribute('aria-hidden', 'true');
  sign.dataset.testid = 'decl-sign';
  sign.innerHTML = html;
  const c = center(at);
  sign.style.left = `${c.x.toFixed(1)}px`;
  sign.style.top = `${c.y.toFixed(1)}px`;
  layer.appendChild(sign);
  const box = sign.getBoundingClientRect();
  // Into the game line: centre onto its centre, as tall as it is.
  const end = into && into.width ? center(into) : { x: c.x, y: c.y - box.height };
  const s = into && into.height && box.height ? Math.max(0.15, Math.min(1, into.height / box.height)) : 0.3;
  const dx = end.x - c.x;
  const dy = end.y - c.y;
  const hit = 0.16;
  const tilt = jitter(3);
  const anim = track(
    sign.animate(
      [
        { transform: `translateY(-38px) scale(2.3) rotate(${tilt - 7}deg)`, opacity: 0, easing: 'cubic-bezier(.55,0,.9,.4)' },
        { transform: `translateY(0) scale(0.9, 0.84) rotate(${tilt}deg)`, opacity: 1, offset: hit, easing: 'cubic-bezier(.2,.8,.4,1)' },
        { transform: `translateY(-7px) scale(1.05) rotate(${tilt * 0.5}deg)`, opacity: 1, offset: 0.24, easing: 'cubic-bezier(.5,0,.7,.4)' },
        { transform: `translateY(0) scale(1) rotate(${tilt * 0.4}deg)`, opacity: 1, offset: 0.3 },
        { transform: `translateY(0) scale(1) rotate(${tilt * 0.4}deg)`, opacity: 1, offset: 0.68, easing: 'cubic-bezier(.5,0,.3,1)' },
        { transform: `translate(${dx.toFixed(1)}px, ${dy.toFixed(1)}px) scale(${s.toFixed(3)}) rotate(0deg)`, opacity: 0.9, offset: 0.94 },
        { transform: `translate(${dx.toFixed(1)}px, ${dy.toFixed(1)}px) scale(${s.toFixed(3)}) rotate(0deg)`, opacity: 0 },
      ],
      { duration, fill: 'both' },
    ),
  );
  const gone = (): void => sign.remove();
  anim.finished.then(gone, gone);
  if (opts.onHit) {
    const hitNow = opts.onHit;
    const timer = window.setTimeout(hitNow, duration * hit);
    // Hurried before it hit: no knock after the fact.
    const quiet = (): void => window.clearTimeout(timer);
    anim.finished.then(quiet, quiet);
  }
  // Dust puffs out from under its lower edge as it hits the felt, to both sides.
  const puffs = 10;
  for (let i = 0; i < puffs; i++) {
    const side = i % 2 ? 1 : -1;
    const dust = document.createElement('span');
    dust.className = 'decl-dust';
    dust.setAttribute('aria-hidden', 'true');
    const x0 = c.x + side * (box.width / 2 - 6 - Math.random() * box.width * 0.25);
    const y0 = c.y + box.height / 2 + 1 - Math.random() * 6;
    dust.style.left = `${x0.toFixed(1)}px`;
    dust.style.top = `${y0.toFixed(1)}px`;
    layer.appendChild(dust);
    const fx = side * (16 + Math.random() * 34);
    const fy = -(Math.random() * 14) + 4;
    const size = 0.7 + Math.random() * 0.6;
    const puff = track(
      dust.animate(
        [
          { transform: `translate(0, 0) scale(${size.toFixed(2)})`, opacity: 0, easing: 'steps(1, end)' },
          { transform: `translate(0, 0) scale(${size.toFixed(2)})`, opacity: 1, offset: 0.01, easing: 'cubic-bezier(.1,.7,.3,1)' },
          { transform: `translate(${(fx * 0.75).toFixed(1)}px, ${(fy * 0.75).toFixed(1)}px) scale(${size.toFixed(2)})`, opacity: 0.85, offset: 0.55 },
          { transform: `translate(${fx.toFixed(1)}px, ${fy.toFixed(1)}px) scale(0.3)`, opacity: 0 },
        ],
        { duration: duration * 0.34, delay: duration * hit, fill: 'both' },
      ),
    );
    const off = (): void => dust.remove();
    puff.finished.then(off, off);
  }
  return duration * 0.3;
}

/** Pixel rows of the scissors, open and shut: h handle, b blade, d rivet and edge. */
const SCISSORS = {
  open: ['hhh.............', 'h.h.......bbbb..', 'hhhd...bbbb.....', '...dbbbb........', '....dd..........', '...dbbbb........', 'hhhd...bbbb.....', 'h.h.......bbbb..', 'hhh.............'],
  shut: ['................', 'hhh.............', 'h.h.............', 'hhhd............', '...ddbbbbbbbbbbb', 'hhhd............', 'h.h.............', 'hhh.............', '................'],
};
const SCISSOR_INK: Record<string, string> = { h: '#c0262f', b: '#d8dde3', d: '#2b1a0e' };

function pixelRows(rows: string[], cls: string): string {
  let out = '';
  rows.forEach((row, y) => {
    for (let x = 0; x < row.length; x++) {
      const ink = SCISSOR_INK[row[x] ?? '.'];
      if (ink) out += `<rect x="${x}" y="${y}" width="1" height="1" fill="${ink}"/>`;
    }
  });
  return `<g class="${cls}">${out}</g>`;
}

/**
 * Schneider: a small pixel scissors snips once across `across` (the
 * felt), opening and shutting on the way, leaving a cut line that fades.
 * Cosmetic; it lives in `layer` and goes when done or hurried. Resolves
 * when it has crossed.
 */
export async function snipAcross(layer: HTMLElement, across: DOMRect, ms = 950): Promise<void> {
  const duration = dur(ms);
  if (!duration || !across.width) return;
  const y = across.top + across.height * 0.42;
  const x0 = across.left - 40;
  const x1 = across.right + 40;
  const line = document.createElement('span');
  line.className = 'snip-line';
  line.setAttribute('aria-hidden', 'true');
  line.style.left = `${across.left.toFixed(1)}px`;
  line.style.top = `${y.toFixed(1)}px`;
  line.style.width = `${across.width.toFixed(1)}px`;
  const tool = document.createElement('span');
  tool.className = 'snip-scissors';
  tool.setAttribute('aria-hidden', 'true');
  tool.dataset.testid = 'snip';
  tool.innerHTML = `<svg viewBox="0 0 16 9" shape-rendering="crispEdges">${pixelRows(SCISSORS.open, 'snip-open')}${pixelRows(SCISSORS.shut, 'snip-shut')}</svg>`;
  tool.style.left = `${x0.toFixed(1)}px`;
  tool.style.top = `${y.toFixed(1)}px`;
  layer.append(line, tool);
  const span = x1 - x0;
  const fly = track(
    tool.animate(
      [
        { transform: 'translate(0, 0) rotate(-4deg)', opacity: 1 },
        { transform: `translate(${(span * 0.5).toFixed(1)}px, 3px) rotate(3deg)`, opacity: 1, offset: 0.5 },
        { transform: `translate(${span.toFixed(1)}px, 0) rotate(-2deg)`, opacity: 1 },
      ],
      { duration, easing: 'cubic-bezier(.35,.1,.65,.9)', fill: 'both' },
    ),
  );
  const cut = track(
    line.animate(
      [
        { transform: 'scaleX(0)', opacity: 0.9 },
        { transform: 'scaleX(1)', opacity: 0.9, offset: 0.78 },
        { transform: 'scaleX(1)', opacity: 0 },
      ],
      { duration: duration * 1.25, easing: 'linear', fill: 'both' },
    ),
  );
  const gone = (): void => {
    tool.remove();
  };
  fly.finished.then(gone, gone);
  const off = (): void => line.remove();
  cut.finished.then(off, off);
  await fly.finished.catch(() => undefined);
}

/** An opponent pleased: his portrait nods once and brightens. */
export function nod(el: Element): void {
  const duration = dur(520);
  if (!duration) return;
  track(
    el.animate(
      [
        { transform: 'translateY(0)', filter: 'brightness(1)' },
        { transform: 'translateY(3px)', filter: 'brightness(1.35)', offset: 0.3 },
        { transform: 'translateY(-1px)', filter: 'brightness(1.25)', offset: 0.6 },
        { transform: 'translateY(0)', filter: 'brightness(1)' },
      ],
      { duration, easing: 'ease-out' },
    ),
  );
}

/** An opponent annoyed: his portrait darkens and shakes its head. */
export function frown(el: Element): void {
  const duration = dur(560);
  if (!duration) return;
  track(
    el.animate(
      [
        { transform: 'translateX(0)', filter: 'brightness(1) saturate(1)' },
        { transform: 'translateX(-2px)', filter: 'brightness(.72) saturate(.6)', offset: 0.18 },
        { transform: 'translateX(2px)', filter: 'brightness(.72) saturate(.6)', offset: 0.36 },
        { transform: 'translateX(-2px)', filter: 'brightness(.75) saturate(.65)', offset: 0.54 },
        { transform: 'translateX(1px)', filter: 'brightness(.8) saturate(.75)', offset: 0.72 },
        { transform: 'translateX(0)', filter: 'brightness(1) saturate(1)' },
      ],
      { duration, easing: 'ease-in-out' },
    ),
  );
}

/** A settled promise of an animation: finished or cancelled (finishAll, a removed element). */
function settled(anim: Animation): Promise<void> {
  return anim.finished.then(
    () => undefined,
    () => undefined,
  );
}

/** Columns and rows of the bits an "Untot" card crumbles into. */
const CRUMBLE_COLS = 6;
const CRUMBLE_ROWS = 9;

/**
 * "Untot" rises (playtest 05.11.2026): the card as it was played (`before`,
 * an inline card SVG) lies big over the table, crumbles from the top down
 * into pixel bits that fall away, and the card it comes back as (`after`,
 * the zombie picture) is left in its place. About 1.6 s at tempo Normal;
 * nothing with reduced motion; finishAll() (a tap) ends it at once.
 * Resolves with the box the new card stood in (for its flight into the
 * hand), or null when nothing played. Cosmetic: the cards come from the
 * core's event and view, `Math.random` only scatters the bits.
 */
export async function crumbleInto(root: HTMLElement, before: string, after: string): Promise<DOMRect | null> {
  if (!dur(1)) return null;
  const box = document.createElement('div');
  box.className = 'rise-morph';
  box.dataset.testid = 'rise-morph';
  box.dataset.stage = 'before';
  box.setAttribute('aria-hidden', 'true');
  const bits: string[] = [];
  for (let r = 0; r < CRUMBLE_ROWS; r++) {
    for (let c = 0; c < CRUMBLE_COLS; c++) {
      const top = (r / CRUMBLE_ROWS) * 100;
      const left = (c / CRUMBLE_COLS) * 100;
      const bottom = 100 - ((r + 1) / CRUMBLE_ROWS) * 100;
      const right = 100 - ((c + 1) / CRUMBLE_COLS) * 100;
      bits.push(`<span class="card rise-bit" data-row="${r}" style="clip-path: inset(${top.toFixed(2)}% ${right.toFixed(2)}% ${bottom.toFixed(2)}% ${left.toFixed(2)}%)">${before}</span>`);
    }
  }
  // Whole until it crumbles: the bits only take over then (no seams between them before).
  box.innerHTML = `<span class="card rise-new">${after}</span><span class="rise-old"><span class="card rise-whole">${before}</span>${bits.join('')}</span>`;
  root.appendChild(box);
  const rect = box.getBoundingClientRect();
  try {
    const old = box.querySelector<HTMLElement>('.rise-old');
    const fresh = box.querySelector<HTMLElement>('.rise-new');
    if (!old || !fresh) return rect;
    // The played card comes up and shudders.
    await settled(
      play(
        old,
        [
          { transform: 'scale(.7)', opacity: 0.4 },
          { transform: 'scale(1.04)', opacity: 1, offset: 0.45 },
          { transform: 'scale(1) rotate(-2deg)', offset: 0.65 },
          { transform: 'scale(1) rotate(2deg)', offset: 0.82 },
          { transform: 'scale(1) rotate(0)', opacity: 1 },
        ],
        { duration: dur(520), easing: 'ease-out', fill: 'backwards' },
      ),
    );
    if (!box.isConnected) return rect;
    // It crumbles from the top down; the zombie card shows through.
    box.dataset.stage = 'crumble';
    const falls = Array.from(box.querySelectorAll<HTMLElement>('.rise-bit')).map((bit) => {
      const row = Number(bit.dataset.row ?? 0);
      const dx = (Math.random() * 2 - 1) * 14;
      const dy = 40 + Math.random() * 50;
      const turn = (Math.random() * 2 - 1) * 40;
      return settled(
        play(
          bit,
          [
            { transform: 'translate(0, 0) rotate(0)', opacity: 1 },
            { transform: `translate(${dx.toFixed(1)}px, ${dy.toFixed(1)}px) rotate(${turn.toFixed(0)}deg)`, opacity: 0 },
          ],
          { duration: dur(480), delay: dur(row * 55 + Math.random() * 90), easing: 'steps(6, end)', fill: 'both' },
        ),
      );
    });
    const reveal = settled(
      play(
        fresh,
        [
          { opacity: 0, filter: 'brightness(1.6) saturate(.4)' },
          { opacity: 1, filter: 'drop-shadow(0 0 6px #7edc6e) brightness(1.15)', offset: 0.6 },
          { opacity: 1, filter: 'drop-shadow(0 0 0 rgba(126, 220, 110, 0)) brightness(1)' },
        ],
        { duration: dur(760), delay: dur(160), easing: 'ease-out', fill: 'both' },
      ),
    );
    await Promise.all([...falls, reveal]);
    box.dataset.stage = 'after';
    // The new card stays a moment, so the player sees what came back.
    await settled(play(fresh, [{ transform: 'scale(1)' }, { transform: 'scale(1.05)' }, { transform: 'scale(1)' }], { duration: dur(360), easing: 'ease-in-out' }));
    return box.getBoundingClientRect();
  } finally {
    box.remove();
  }
}

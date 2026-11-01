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
  return track(el.animate(frames, { duration, delay: dur(opts.delay ?? 0), easing: 'cubic-bezier(.2,.75,.25,1)', fill: 'backwards' }));
}

/** A random number in [-n, n]: cosmetic only (Math.random, never the core's RNG). */
function jitter(n: number): number {
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
 * cards close up on the middle card, then slide off together to `to` (a
 * point beyond the edge of the table on the winner's side), turning a
 * little and fading at the end. Resolves when done.
 */
export async function sweepAway(els: Element[], to: { x: number; y: number }, ms = 460): Promise<void> {
  const duration = dur(ms);
  if (!duration || !els.length) return;
  const rects = els.map((el) => el.getBoundingClientRect());
  const mid = rects[Math.floor(rects.length / 2)] ?? rects[0];
  if (!mid) return;
  const g = center(mid);
  const spin = to.x < g.x - 20 ? -1 : to.x > g.x + 20 ? 1 : 0;
  const anims = els.map((el, i) => {
    const r = rects[i] ?? mid;
    const a = center(r);
    const gx = g.x - a.x + (i - 1) * 3;
    const gy = g.y - a.y + (i - 1) * 2;
    const tilt = jitter(7);
    const last = spin * 18 + jitter(10);
    return track(
      el.animate(
        [
          // Close up quickly, then slide off with growing speed.
          { transform: 'none', opacity: 1, easing: 'cubic-bezier(.2,.7,.3,1)' },
          { transform: `translate(${gx}px, ${gy}px) rotate(${tilt}deg)`, opacity: 1, offset: 0.26, easing: 'cubic-bezier(.5,0,.85,.5)' },
          { transform: `translate(${gx + to.x - g.x}px, ${gy + to.y - g.y}px) rotate(${last}deg)`, opacity: 0.15 },
        ],
        { duration, delay: i * dur(20), fill: 'forwards' },
      ),
    );
  });
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

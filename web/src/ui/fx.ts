// Visual effects with the Web Animations API: cards flying in, tricks
// moving to the winner, dealing, number count-ups. Every effect respects
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

/** Animate `el` from the place of `from` into its own place. */
export function flyIn(el: Element, from: DOMRect, opts: { ms?: number; delay?: number; rotate?: number } = {}): Animation | null {
  const duration = dur(opts.ms ?? 280);
  if (!duration) return null;
  const to = el.getBoundingClientRect();
  if (!to.width) return null;
  const a = center(from);
  const b = center(to);
  const scale = Math.max(0.2, Math.min(2, from.width / to.width));
  const rotate = opts.rotate ?? 0;
  return track(
    el.animate(
      [
        { transform: `translate(${a.x - b.x}px, ${a.y - b.y}px) scale(${scale}) rotate(${rotate}deg)`, opacity: 0.85 },
        { transform: 'none', opacity: 1 },
      ],
      { duration, delay: dur(opts.delay ?? 0), easing: 'cubic-bezier(.2,.75,.25,1)', fill: 'backwards' },
    ),
  );
}

/** Move elements towards `to`, shrinking and fading; resolves when done. */
export async function flyAway(els: Element[], to: DOMRect, ms = 340): Promise<void> {
  const duration = dur(ms);
  if (!duration || !els.length) return;
  const b = center(to);
  const anims = els.map((el, i) => {
    const r = el.getBoundingClientRect();
    const a = center(r);
    return track(
      el.animate(
        [
          { transform: 'none', opacity: 1 },
          { transform: `translate(${b.x - a.x}px, ${b.y - a.y}px) scale(0.35) rotate(${(i - 1) * 8}deg)`, opacity: 0.2 },
        ],
        { duration, delay: i * dur(30), easing: 'cubic-bezier(.5,0,.75,.4)', fill: 'forwards' },
      ),
    );
  });
  await Promise.all(anims.map((a) => a.finished.catch(() => undefined)));
}

/** Deal: cards fly in from the deck, one after another. */
export function deal(cards: Element[], from: DOMRect, opts: { step?: number; ms?: number; onCard?: (i: number) => void } = {}): void {
  if (!motionOn()) return;
  const step = opts.step ?? 45;
  cards.forEach((el, i) => {
    const anim = flyIn(el, from, { ms: opts.ms ?? 300, delay: i * step, rotate: -25 + ((i * 37) % 50) });
    if (anim && opts.onCard) {
      const cb = opts.onCard;
      window.setTimeout(() => cb(i), dur(i * step));
    }
  });
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
    const start = performance.now();
    const frame = (now: number): void => {
      const p = timer.playState === 'finished' ? 1 : Math.min(1, (now - start) / duration);
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

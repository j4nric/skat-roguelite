// Reward effects: Groschen flying into the purse, the account counting up
// with its bar, beer foam when the target is reached, hearts that break
// or pop in, the pixel firework of a won run and the pub lights going out
// on a lost one, and a short spring on tapped cards.
//
// Pure presentation. Every number comes from the core's views and events;
// this file only moves small sprites drawn as SVG or CSS boxes. All of it
// goes through ./fx.ts: tempo-scaled, off with reduced motion, ended at
// once by finishAll() (a tap hurries the table), and never in the way of
// a tap (the sprites catch no pointer).

import { sound } from '../audio';
import type { RunView } from '../types';
import { dur, motionOn, play, tween } from './fx';
import { coinIcon, heart, meterPercent } from './hud';

/** The fixed layer above everything (dialogs too) that holds the flying sprites. */
function layer(): HTMLElement {
  let el = document.querySelector<HTMLElement>('body > .fx-layer');
  if (!el) {
    el = document.createElement('div');
    el.className = 'fx-layer';
    el.setAttribute('aria-hidden', 'true');
    document.body.appendChild(el);
  }
  return el;
}

/** A sprite in the fx layer, its centre at (x, y). */
function sprite(cls: string, html: string, x: number, y: number): HTMLElement {
  const el = document.createElement('span');
  el.className = `fx-sprite ${cls}`;
  el.innerHTML = html;
  el.style.left = `${Math.round(x)}px`;
  el.style.top = `${Math.round(y)}px`;
  layer().appendChild(el);
  return el;
}

function gone(anim: Animation, el: Element): void {
  const rm = (): void => el.remove();
  anim.finished.then(rm, rm);
}

function mid(r: DOMRect): { x: number; y: number } {
  return { x: r.left + r.width / 2, y: r.top + r.height / 2 };
}

/** A random number in [-n, n]: cosmetic only (never the core's RNG). */
function jitter(n: number): number {
  return (Math.random() * 2 - 1) * n;
}

/**
 * The target is reached: the bar's foam crown pops up and a few bubbles
 * rise from it. With reduced motion the crown is simply there.
 */
export function foamUp(meter: HTMLElement): void {
  meter.classList.add('is-full');
  const foam = meter.querySelector('.foam');
  if (!motionOn()) return;
  if (foam) {
    play(foam, [{ transform: 'scaleY(0)' }, { transform: 'scaleY(1.6)', offset: 0.5 }, { transform: 'scaleY(0.85)', offset: 0.75 }, { transform: 'scaleY(1)' }], {
      duration: dur(420),
      easing: 'steps(6)',
    });
  }
  meter.dataset.bubbling = 'true';
  const r = meter.getBoundingClientRect();
  const n = 9;
  let last: Animation | null = null;
  for (let i = 0; i < n; i++) {
    const x = r.left + 4 + ((r.width - 8) * (i + 0.5)) / n + jitter(3);
    const size = Math.random() < 0.4 ? 4 : 3;
    const b = sprite('fx-bubble', '', x, r.top + r.height / 2);
    b.style.width = `${size}px`;
    b.style.height = `${size}px`;
    const rise = 14 + Math.random() * 16;
    last = play(
      b,
      [
        { transform: 'translate(0, 0)', opacity: 0 },
        { transform: `translate(${jitter(2)}px, ${-rise * 0.4}px)`, opacity: 1, offset: 0.3 },
        { transform: `translate(${jitter(4)}px, ${-rise}px)`, opacity: 0 },
      ],
      { duration: dur(520 + Math.random() * 300), delay: dur(i * 45 + Math.random() * 80), easing: 'steps(7)', fill: 'backwards' },
    );
    gone(last, b);
  }
  const done = (): void => {
    delete meter.dataset.bubbling;
  };
  if (last) last.finished.then(done, done);
}

/** Paint an account value and its bar into the given elements. */
function paintAccount(num: HTMLElement | null, meter: HTMLElement | null, value: number, target: number, burst: boolean): void {
  if (num) {
    num.textContent = String(value);
    num.classList.toggle('is-negative', value < 0);
  }
  const fill = meter?.querySelector<HTMLElement>('.chalk-fill');
  if (!meter || !fill) return;
  const pct = meterPercent(value, target);
  fill.style.transition = 'none';
  // Whole percent steps keep the bar's edge on the pixel grid.
  fill.style.width = `${Math.round(pct)}%`;
  const full = pct >= 100;
  if (full && !meter.classList.contains('is-full')) {
    if (burst) foamUp(meter);
    else meter.classList.add('is-full');
  } else if (!full) {
    meter.classList.remove('is-full');
  }
}

/**
 * Count an account from `from` to `to` in `num` while its bar fills,
 * ticking now and then; foam when the target is reached on the way.
 * `els` is asked anew every frame (the head may be drawn again meanwhile);
 * `alive` stops the count when its screen went away.
 */
export function countAccount(
  els: () => { num: HTMLElement | null; meter: HTMLElement | null },
  from: number,
  to: number,
  target: number,
  ms: number,
  alive: () => boolean = () => true,
  onValue: (n: number) => void = () => undefined,
): Promise<void> {
  let shown = from;
  let lastTick = 0;
  return tween(ms, (p) => {
    if (!alive()) return;
    const value = Math.round(from + (to - from) * p);
    onValue(value);
    const { num, meter } = els();
    paintAccount(num, meter, value, target, true);
    if (value !== shown) {
      shown = value;
      const now = performance.now();
      if (p < 1 && now - lastTick > 70) {
        lastTick = now;
        sound.play('tick', { volume: 0.5, rate: 1 + Math.min(1, Math.abs(value - from) / Math.max(1, Math.abs(to - from))) * 0.5 });
      }
    }
  });
}

/** Throttle for the clink of landing coins (many at once sound as one). */
let lastClink = 0;

function clink(i: number): void {
  const now = performance.now();
  if (now - lastClink < 55) return;
  lastClink = now;
  sound.play('coin', { volume: 0.55, rate: 0.95 + Math.min(i, 8) * 0.05 });
}

/**
 * What the head of the run shows of the account, the Groschen and the
 * lives, and how changes get there: the account counts up, coins fly into
 * the purse, a lost heart breaks. The head is drawn anew on every render;
 * `sync` compares the view with what was shown last and animates the
 * difference.
 */
export class HudFx {
  private shown: { run: string; series: string; score: number; target: number; coins: number; lives: number } | null = null;
  /** Bumped when a count of the account must stop (new series, new run). */
  private token = 0;
  /** The account value a running count shows right now (repainted after a render). */
  private counting: number | null = null;
  private inAir = 0;
  private ghost: HTMLElement | null = null;
  private ghostValue = 0;
  private tap: { rect: DOMRect; at: number } | null = null;

  constructor(private readonly screen: HTMLElement) {}

  /** Forget everything shown (a new run, the start screen). */
  reset(): void {
    this.shown = null;
    this.token += 1;
    this.counting = null;
  }

  /** Where the last tap was: coins of a sale fly from there. */
  noteTap(el: Element): void {
    this.tap = { rect: el.getBoundingClientRect(), at: performance.now() };
  }

  private tapRect(): DOMRect | null {
    return this.tap && performance.now() - this.tap.at < 5000 ? this.tap.rect : null;
  }

  private hudEls(): { num: HTMLElement | null; meter: HTMLElement | null } {
    return {
      num: this.screen.querySelector<HTMLElement>('.hud [data-testid="account"]'),
      meter: this.screen.querySelector<HTMLElement>('.hud .slate .chalk-meter'),
    };
  }

  /** After a render of the run screen: animate what changed since the last one. */
  sync(view: RunView): void {
    const run = `${view.seed}|${view.character.character}|${view.length}|${view.stake ?? 0}`;
    const series = `${view.series.section}|${view.series.index}`;
    const now = { run, series, score: view.series.score, target: view.series.target, coins: view.coins, lives: view.tour.lives };
    const s = this.shown;
    if (!s || s.run !== run) {
      this.reset();
      this.shown = now;
      return;
    }
    if (s.series !== series || s.target !== now.target) {
      // Another series: its slate starts as drawn.
      this.token += 1;
      this.counting = null;
      s.series = series;
      s.target = now.target;
      s.score = now.score;
    } else if (s.score !== now.score) {
      void this.account(now.score, now.target);
    } else if (this.counting !== null) {
      const { num, meter } = this.hudEls();
      paintAccount(num, meter, this.counting, s.target, false);
    }
    if (s.coins !== now.coins) {
      if (now.coins > s.coins) this.gainTo(now.coins, null);
      else this.loseTo(now.coins);
    } else if (this.ghost) {
      this.placeGhost();
    }
    if (s.lives !== now.lives) this.lives(now.lives);
  }

  /** Count the account in the head up (or down) to `to`. */
  async account(to: number, target: number): Promise<void> {
    const s = this.shown;
    if (!s) return;
    const from = this.counting ?? s.score;
    s.score = to;
    s.target = target;
    const token = ++this.token;
    const alive = (): boolean => token === this.token;
    await countAccount(
      () => this.hudEls(),
      from,
      to,
      target,
      Math.min(900, 380 + Math.abs(to - from) * 4),
      alive,
      (n) => {
        if (alive()) this.counting = n;
      },
    );
    if (alive()) this.counting = null;
  }

  /** The copy of the purse sits exactly over the purse in the head. */
  private placeGhost(): void {
    const purse = this.screen.querySelector<HTMLElement>('.hud-coins');
    if (!purse || !this.ghost) return;
    const r = purse.getBoundingClientRect();
    this.ghost.style.left = `${Math.round(r.left)}px`;
    this.ghost.style.top = `${Math.round(r.top)}px`;
  }

  private setPurse(value: number): void {
    const b = this.screen.querySelector<HTMLElement>('.hud [data-testid="coins"]');
    if (b) b.textContent = String(value);
  }

  /**
   * Groschen came in: up to eight coins fly one by one from `from` (the
   * score board's payout, an encounter, a sold item; the middle of the
   * screen if unknown) into the purse in the head, each with a clink, and
   * the purse counts up as they land. A copy of the purse above every
   * dialog shows the count while they fly.
   */
  gainTo(total: number, from: DOMRect | null): void {
    const s = this.shown;
    const start = this.ghost ? this.ghostValue : (s?.coins ?? total);
    if (s) s.coins = total;
    const purse = this.screen.querySelector<HTMLElement>('.hud-coins');
    if (!purse || !motionOn() || total <= start) {
      this.setPurse(total);
      if (this.ghost) this.ghostValue = Math.max(this.ghostValue, total);
      this.paintGhost();
      return;
    }
    const to = purse.getBoundingClientRect();
    if (!this.ghost) {
      this.ghost = document.createElement('span');
      this.ghost.className = 'hud-coins purse-ghost';
      this.ghost.dataset.testid = 'purse-ghost';
      this.ghost.innerHTML = `${coinIcon(16)}<b></b>`;
      this.ghost.style.width = `${Math.round(to.width)}px`;
      layer().appendChild(this.ghost);
      this.ghostValue = start;
      this.placeGhost();
      this.paintGhost();
    }
    const src = from ?? this.tapRect() ?? new DOMRect(window.innerWidth / 2 - 10, window.innerHeight * 0.45, 20, 20);
    const a = mid(src);
    const b = mid(to);
    const gain = total - start;
    const n = Math.max(1, Math.min(8, gain));
    for (let i = 0; i < n; i++) {
      const coin = sprite('fx-coin', coinIcon(18), b.x, b.y);
      coin.dataset.testid = 'coin-fly';
      const sx = a.x + jitter(Math.min(18, src.width / 3)) - b.x;
      const sy = a.y + jitter(Math.min(10, src.height / 3)) - b.y;
      const len = Math.hypot(sx, sy) || 1;
      const bend = 40 + jitter(20);
      const mx = sx / 2 + (-sy / len) * bend;
      const my = sy / 2 + (sx / len) * bend;
      const anim = play(
        coin,
        [
          { transform: `translate(${sx}px, ${sy}px) scale(1.3)`, opacity: 0 },
          { transform: `translate(${sx}px, ${sy - 8}px) scale(1.3)`, opacity: 1, offset: 0.12 },
          { transform: `translate(${mx}px, ${my}px) scale(1.1)`, opacity: 1, offset: 0.55 },
          { transform: 'translate(0, 0) scale(0.8)', opacity: 1 },
        ],
        { duration: dur(560), delay: dur(i * 95), easing: 'cubic-bezier(.45,0,.75,1)', fill: 'backwards' },
      );
      // The coin turns as it flies (a pixel coin seen from the edge).
      const svg = coin.querySelector('svg');
      if (svg) play(svg, [{ transform: 'scaleX(1)' }, { transform: 'scaleX(0.2)' }, { transform: 'scaleX(1)' }], { duration: dur(280), delay: dur(i * 95), iterations: 2, easing: 'steps(4)' });
      this.inAir += 1;
      const value = start + Math.round(((i + 1) / n) * gain);
      const land = (): void => {
        coin.remove();
        this.inAir -= 1;
        this.ghostValue = Math.max(this.ghostValue, value);
        this.paintGhost();
        clink(i);
        if (this.ghost) play(this.ghost, [{ transform: 'scale(1.18)' }, { transform: 'scale(1)' }], { duration: dur(140), easing: 'steps(3)' });
        if (this.inAir === 0) this.landed();
      };
      anim.finished.then(land, land);
    }
  }

  private paintGhost(): void {
    const b = this.ghost?.querySelector('b');
    if (b) b.textContent = String(this.ghostValue);
  }

  /** All coins are in: the purse in the head shows the total again. */
  private landed(): void {
    const total = this.shown?.coins ?? this.ghostValue;
    this.setPurse(total);
    this.ghost?.remove();
    this.ghost = null;
  }

  /** Groschen went out: a coin drops from the purse and fades. */
  loseTo(total: number): void {
    const s = this.shown;
    const before = s?.coins ?? total;
    if (s) s.coins = total;
    this.setPurse(total);
    const purse = this.screen.querySelector<HTMLElement>('.hud-coins');
    if (!purse || !motionOn() || total >= before) return;
    const r = purse.getBoundingClientRect();
    const drop = sprite('fx-coin-drop', `${coinIcon(14)}<b>−${before - total}</b>`, r.left + r.width / 2, r.bottom + 4);
    drop.dataset.testid = 'coin-drop';
    gone(
      play(
        drop,
        [
          { transform: 'translate(-50%, -10px)', opacity: 1 },
          { transform: 'translate(-50%, 6px)', opacity: 1, offset: 0.35 },
          { transform: 'translate(-50%, 26px)', opacity: 0 },
        ],
        { duration: dur(760), easing: 'steps(8)' },
      ),
      drop,
    );
    const b = purse.querySelector('b');
    if (b) play(b, [{ color: '#e0675c' }, { color: '#e0675c', offset: 0.6 }, {}], { duration: dur(600), easing: 'steps(3)' });
  }

  /**
   * The lives changed (LivesChanged, or seen on a render): a lost heart in
   * the head breaks into pixel shards that fall and fade; a new one pops in
   * with a pulse. Works on the head as it is on screen right now.
   */
  lives(lives: number): void {
    const s = this.shown;
    const before = s?.lives ?? lives;
    if (s) s.lives = lives;
    const box = this.screen.querySelector<HTMLElement>('.hud [data-testid="lives"]');
    if (!box || before === lives) return;
    const hearts = Array.from(box.querySelectorAll<SVGElement>('.heart'));
    const size = Number(hearts[0]?.getAttribute('width') ?? 14);
    const swap = (i: number, full: boolean): Element | null => {
      const tpl = document.createElement('template');
      tpl.innerHTML = heart(full, size);
      const fresh = tpl.content.firstElementChild;
      if (!fresh) return null;
      const old = hearts[i];
      if (old) old.replaceWith(fresh);
      else box.appendChild(fresh);
      return fresh;
    };
    box.dataset.lives = String(lives);
    if (lives < before) {
      for (let i = lives; i < before; i++) {
        const old = hearts[i];
        if (!old) continue;
        const r = old.getBoundingClientRect();
        swap(i, false);
        if (motionOn()) this.shatter(r, i - lives);
      }
      return;
    }
    for (let i = before; i < lives; i++) {
      const fresh = swap(i, true);
      if (!fresh || !motionOn()) continue;
      fresh.classList.add('is-popping');
      const anim = play(
        fresh,
        [
          { transform: 'scale(0)', opacity: 0 },
          { transform: 'scale(1.7)', opacity: 1, offset: 0.45 },
          { transform: 'scale(0.85)', offset: 0.7 },
          { transform: 'scale(1.25)', offset: 0.85 },
          { transform: 'scale(1)' },
        ],
        { duration: dur(620), delay: dur((i - before) * 180), easing: 'steps(8)', fill: 'backwards' },
      );
      const off = (): void => fresh.classList.remove('is-popping');
      anim.finished.then(off, off);
      // A ring of light around it, a pixel square that grows and fades.
      const r = fresh.getBoundingClientRect();
      const ring = sprite('fx-ring', '', r.left + r.width / 2, r.top + r.height / 2);
      gone(
        play(ring, [{ transform: 'translate(-50%, -50%) scale(0.4)', opacity: 1 }, { transform: 'translate(-50%, -50%) scale(2.2)', opacity: 0 }], {
          duration: dur(520),
          delay: dur((i - before) * 180 + 200),
          easing: 'steps(6)',
          fill: 'backwards',
        }),
        ring,
      );
    }
  }

  /** A heart at `r` breaks: two halves tilt apart, pixel shards fall and fade. */
  private shatter(r: DOMRect, order: number): void {
    const c = mid(r);
    const delay = order * 160;
    const half = (side: -1 | 1): void => {
      const el = sprite(`fx-heart-half ${side < 0 ? 'is-left' : 'is-right'}`, heart(true, Math.round(r.width)), c.x, c.y);
      el.dataset.testid = 'heart-shard';
      gone(
        play(
          el,
          [
            { transform: 'translate(-50%, -50%)', opacity: 1 },
            { transform: `translate(calc(-50% + ${side * 2}px), -50%) rotate(${side * 12}deg)`, opacity: 1, offset: 0.25 },
            { transform: `translate(calc(-50% + ${side * 9}px), calc(-50% + 34px)) rotate(${side * 40}deg)`, opacity: 0 },
          ],
          { duration: dur(820), delay: dur(delay), easing: 'steps(10)', fill: 'backwards' },
        ),
        el,
      );
    };
    half(-1);
    half(1);
    const colours = ['#c8312b', '#c8312b', '#6e1410', '#f4a39c'];
    for (let k = 0; k < 8; k++) {
      const el = sprite('fx-shard', '', c.x + jitter(r.width / 3), c.y + jitter(r.height / 3));
      el.style.background = colours[k % colours.length] ?? '#c8312b';
      el.dataset.testid = 'heart-shard';
      const dx = jitter(16);
      const fall = 26 + Math.random() * 22;
      gone(
        play(
          el,
          [
            { transform: 'translate(0, 0)', opacity: 1 },
            { transform: `translate(${dx * 0.5}px, -6px)`, opacity: 1, offset: 0.2 },
            { transform: `translate(${dx}px, ${fall}px)`, opacity: 0 },
          ],
          { duration: dur(700 + Math.random() * 300), delay: dur(delay + 120), easing: 'steps(9)', fill: 'backwards' },
        ),
        el,
      );
    }
  }
}

/** Colours of the firework, the four suits' inks and the beer. */
const SPARKS = ['#f3c25a', '#c8312b', '#5f9a3c', '#e8dcc0', '#d1773f', '#7fb2d8'];

/**
 * A won run: a pixel firework over the end screen, a few rockets that
 * climb and burst into squares, one after another. Catches no taps.
 */
export function fireworks(bursts = 8): void {
  if (!motionOn()) return;
  const w = window.innerWidth;
  const h = window.innerHeight;
  const box = document.createElement('div');
  box.className = 'fx-fireworks';
  box.dataset.testid = 'fireworks';
  layer().appendChild(box);
  let pending = 0;
  const finish = (): void => {
    pending -= 1;
    if (pending <= 0) box.remove();
  };
  for (let k = 0; k < bursts; k++) {
    const x = w * (0.18 + Math.random() * 0.64);
    const y = h * (0.12 + Math.random() * 0.3);
    const colour = SPARKS[k % SPARKS.length] ?? '#f3c25a';
    const at = k * 380 + Math.random() * 160;
    const climb = 420;
    // The rocket: a short streak from below.
    const rocket = document.createElement('span');
    rocket.className = 'fx-rocket';
    rocket.style.left = `${Math.round(x)}px`;
    rocket.style.top = `${Math.round(y)}px`;
    box.appendChild(rocket);
    pending += 1;
    const up = play(
      rocket,
      [
        { transform: `translate(0, ${Math.round(h - y)}px)`, opacity: 1 },
        { transform: 'translate(0, 0)', opacity: 1, offset: 0.95 },
        { transform: 'translate(0, 0)', opacity: 0 },
      ],
      { duration: dur(climb), delay: dur(at), easing: 'cubic-bezier(.2,.6,.4,1)', fill: 'both' },
    );
    up.finished.then(
      () => {
        rocket.remove();
        finish();
      },
      () => {
        rocket.remove();
        finish();
      },
    );
    window.setTimeout(() => sound.play('clink', { volume: 0.35, rate: 1.3 + Math.random() * 0.5 }), dur(at + climb));
    const n = 20;
    for (let i = 0; i < n; i++) {
      const spark = document.createElement('span');
      spark.className = 'fx-spark';
      spark.style.left = `${Math.round(x)}px`;
      spark.style.top = `${Math.round(y)}px`;
      spark.style.background = i % 4 === 0 ? '#fff6dc' : colour;
      box.appendChild(spark);
      pending += 1;
      const angle = (i / n) * Math.PI * 2 + jitter(0.15);
      const dist = 50 + Math.random() * 46;
      const dx = Math.cos(angle) * dist;
      const dy = Math.sin(angle) * dist;
      const anim = play(
        spark,
        [
          { transform: 'translate(0, 0) scale(1)', opacity: 0 },
          { transform: 'translate(0, 0) scale(1)', opacity: 1, offset: 0.02 },
          { transform: `translate(${dx * 0.8}px, ${dy * 0.8}px) scale(1)`, opacity: 1, offset: 0.4 },
          { transform: `translate(${dx}px, ${dy + 26}px) scale(0.5)`, opacity: 0 },
        ],
        { duration: dur(1000 + Math.random() * 300), delay: dur(at + climb), easing: 'steps(12)', fill: 'both' },
      );
      const done = (): void => {
        spark.remove();
        finish();
      };
      anim.finished.then(done, done);
    }
  }
}

/** A hanging pub lamp, 12 × 14 pixels. */
function lamp(): string {
  return `<svg viewBox="0 0 12 14" width="48" height="56" shape-rendering="crispEdges" aria-hidden="true"><path d="M6 0h1v5h-1z" fill="#3a3530"/><path d="M4 5h4v1h-4zM2 6h8v1h-8zM1 7h10v2h-10z" fill="#5d7a4a"/><path d="M1 9h10v1h-10z" fill="#3f5532"/><path d="M4 10h4v2h-4z" fill="#ffe7a1"/><path d="M5 12h2v1h-2z" fill="#fff6dc"/></svg>`;
}

/**
 * A lost run: the lights of the pub go out. The room darkens towards a
 * single lamp that flickers once, then holds. Resolves when it is dark
 * (at once with reduced motion, or when the player hurries); then
 * `lightsUp` lifts the dark over the end screen.
 */
export async function lightsOut(): Promise<void> {
  if (!motionOn()) return;
  document.querySelector('body > .fx-lights')?.remove();
  const el = document.createElement('div');
  el.className = 'fx-lights';
  el.dataset.testid = 'lights-out';
  el.setAttribute('aria-hidden', 'true');
  el.innerHTML = `<div class="fx-dark"></div><div class="fx-lamp">${lamp()}</div>`;
  document.body.appendChild(el);
  const dark = el.querySelector('.fx-dark');
  const lampEl = el.querySelector('.fx-lamp');
  if (lampEl) play(lampEl, [{ opacity: 0 }, { opacity: 0, offset: 0.4 }, { opacity: 1, offset: 0.5 }, { opacity: 0.3, offset: 0.6 }, { opacity: 1 }], { duration: dur(1400), fill: 'both' });
  sound.play('knock', { volume: 0.4, rate: 0.7, delay: dur(500) });
  if (!dark) return;
  const anim = play(
    dark,
    [
      { transform: 'translate(-50%, -50%) scale(6)', opacity: 0 },
      { transform: 'translate(-50%, -50%) scale(6)', opacity: 0.6, offset: 0.1 },
      { transform: 'translate(-50%, -50%) scale(6)', opacity: 0.15, offset: 0.17 },
      { transform: 'translate(-50%, -50%) scale(4)', opacity: 0.85, offset: 0.3 },
      { transform: 'translate(-50%, -50%) scale(1.15)', opacity: 1, offset: 0.8 },
      { transform: 'translate(-50%, -50%) scale(1)', opacity: 1 },
    ],
    { duration: dur(1400), easing: 'steps(14)', fill: 'forwards' },
  );
  await anim.finished.catch(() => undefined);
  // A moment in the dark.
  await play(el, [{ opacity: 1 }, { opacity: 1 }], { duration: dur(350) }).finished.catch(() => undefined);
}

/** The end screen is there: the dark lifts, the lamp stays a moment longer. */
export function lightsUp(): void {
  const el = document.querySelector<HTMLElement>('body > .fx-lights');
  if (!el) return;
  if (!motionOn()) {
    el.remove();
    return;
  }
  gone(play(el, [{ opacity: 1 }, { opacity: 1, offset: 0.3 }, { opacity: 0 }], { duration: dur(900), easing: 'steps(8)', fill: 'forwards' }), el);
}

/** A short spring on a tapped thing: from pressed (0.95) a little past its size and back. */
export function spring(el: Element): void {
  if (!motionOn()) return;
  // Not tracked: a spring of 0.2 s needs no hurrying, and finishAll() must not cut it.
  el.animate([{ scale: '0.95' }, { scale: '1.04', offset: 0.55 }, { scale: '1' }], { duration: 200, easing: 'ease-out' });
}

/** What springs back on release (pressed via CSS :active, see style.css). */
const PRESSABLE = 'button:not(:disabled):not([data-drag]), .card[data-action]:not([data-drag])';

/** Spring buttons and tappable cards back when the finger lifts. Installed once on `root`. */
export function watchSprings(root: HTMLElement): void {
  root.addEventListener('pointerup', (e) => {
    const el = (e.target as Element | null)?.closest(PRESSABLE);
    if (el) spring(el);
  });
}

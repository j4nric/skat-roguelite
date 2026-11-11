// Sound effects through the Web Audio API. The files are synthesized by
// scripts/make-sounds.mjs and shipped with the app; nothing is loaded from
// other servers. Audio starts only after the first user interaction, as
// browsers require. Any failure keeps the game silent, never broken.
//
// One AudioContext serves the sound effects and the music (./music.ts). It
// is made on the first user interaction, and only if one of them is on; it
// is suspended while the page is hidden.

import bangUrl from './assets/sounds/bang.mp3?url';
import bossUrl from './assets/sounds/boss.mp3?url';
import cardUrl from './assets/sounds/card.mp3?url';
import chalkUrl from './assets/sounds/chalk.mp3?url';
import clinkUrl from './assets/sounds/clink.mp3?url';
import coinUrl from './assets/sounds/coin.mp3?url';
import declareUrl from './assets/sounds/declare.mp3?url';
import diceUrl from './assets/sounds/dice.mp3?url';
import discardUrl from './assets/sounds/discard.mp3?url';
import gulpUrl from './assets/sounds/gulp.mp3?url';
import knockUrl from './assets/sounds/knock.mp3?url';
import kontraUrl from './assets/sounds/kontra.mp3?url';
import loseUrl from './assets/sounds/lose.mp3?url';
import nightUrl from './assets/sounds/night.mp3?url';
import pickupUrl from './assets/sounds/pickup.mp3?url';
import pourUrl from './assets/sounds/pour.mp3?url';
import prizeUrl from './assets/sounds/prize.mp3?url';
import quipUrl from './assets/sounds/quip.mp3?url';
import schwarzUrl from './assets/sounds/schwarz.mp3?url';
import shuffleUrl from './assets/sounds/shuffle.mp3?url';
import slamUrl from './assets/sounds/slam.mp3?url';
import snipUrl from './assets/sounds/snip.mp3?url';
import tapUrl from './assets/sounds/tap.mp3?url';
import tickUrl from './assets/sounds/tick.mp3?url';
import tillUrl from './assets/sounds/till.mp3?url';
import trickUrl from './assets/sounds/trick.mp3?url';
import unlockUrl from './assets/sounds/unlock.mp3?url';
import winUrl from './assets/sounds/win.mp3?url';

const URLS = {
  // At the table, routine.
  card: cardUrl,
  trick: trickUrl,
  tick: tickUrl,
  tap: tapUrl,
  quip: quipUrl,
  shuffle: shuffleUrl,
  pickup: pickupUrl,
  discard: discardUrl,
  slam: slamUrl,
  chalk: chalkUrl,
  // Notable moments.
  clink: clinkUrl,
  coin: coinUrl,
  till: tillUrl,
  prize: prizeUrl,
  dice: diceUrl,
  pour: pourUrl,
  gulp: gulpUrl,
  knock: knockUrl,
  declare: declareUrl,
  kontra: kontraUrl,
  snip: snipUrl,
  schwarz: schwarzUrl,
  // Big moments.
  bang: bangUrl,
  unlock: unlockUrl,
  boss: bossUrl,
  night: nightUrl,
  win: winUrl,
  lose: loseUrl,
} as const;

export type SoundName = keyof typeof URLS;

/**
 * Level of each sound before the player's volume. Routine sounds (a card,
 * a tap) sit low, as they come dozens of times a game; the moments that
 * matter (the Ansage, the result, an unlock) stand out.
 */
const VOLUME: Record<SoundName, number> = {
  card: 0.55,
  trick: 0.5,
  tick: 0.45,
  tap: 0.22,
  quip: 0.2,
  shuffle: 0.45,
  pickup: 0.5,
  discard: 0.5,
  slam: 0.6,
  chalk: 0.35,
  clink: 0.5,
  coin: 0.55,
  till: 0.55,
  prize: 0.5,
  dice: 0.6,
  pour: 0.5,
  gulp: 0.5,
  knock: 0.8,
  declare: 0.6,
  kontra: 0.75,
  snip: 0.7,
  schwarz: 0.7,
  bang: 0.8,
  unlock: 0.6,
  boss: 0.7,
  night: 0.65,
  win: 0.6,
  lose: 0.6,
};

/**
 * How much a sound matters in the mix: 0 routine, 1 notable, 2 big. Two big
 * sounds never sound at once (the later one waits for the first, or is
 * dropped if that would take too long), and while a big one sounds the
 * others step back (MIX).
 */
const PRIORITY: Record<SoundName, 0 | 1 | 2> = {
  card: 0,
  trick: 0,
  tick: 0,
  tap: 0,
  quip: 0,
  shuffle: 0,
  pickup: 0,
  discard: 0,
  slam: 1,
  chalk: 0,
  clink: 1,
  coin: 1,
  till: 1,
  prize: 1,
  dice: 1,
  pour: 1,
  gulp: 1,
  knock: 1,
  declare: 1,
  kontra: 1,
  snip: 1,
  schwarz: 1,
  bang: 2,
  unlock: 2,
  boss: 2,
  night: 2,
  win: 2,
  lose: 2,
};

/** The rules of the mix (seconds and factors). */
const MIX = {
  /** The same sound again within this time is dropped (four clinks of one step sound as one). */
  sameGap: 0.07,
  /** Routine sounds may follow each other closer (a deal, a run of ticks). */
  routineGap: 0.025,
  /** Two notable sounds this close: the later one at this factor. */
  notableGap: 0.09,
  notableDuck: 0.6,
  /** While a big sound sounds: notable ones at this factor, routine ones at that. */
  underBigNotable: 0.5,
  underBigRoutine: 0.7,
  /** A big sound counts as sounding for this part of its length (its tail is soft). */
  bigShare: 0.75,
  /** A big sound waits at most this long for another one; else it is dropped. */
  bigWait: 2.5,
  /** The music steps back to this factor while a big sound sounds. */
  musicDuck: 0.5,
};

/**
 * Random spread per play: the rate varies by up to this fraction either
 * way, the volume only downward by up to twice it (never louder than set),
 * so a sound heard many times in a row does not sound machine-made. Decoration
 * only, hence Math.random and not the core's seeded RNG.
 */
const SPREAD: Partial<Record<SoundName, { rate: number; volume: number }>> = {
  card: { rate: 0.06, volume: 0.12 },
  tap: { rate: 0.08, volume: 0.15 },
  quip: { rate: 0.05, volume: 0.15 },
  shuffle: { rate: 0.05, volume: 0.1 },
  pickup: { rate: 0.05, volume: 0.1 },
  discard: { rate: 0.06, volume: 0.1 },
  slam: { rate: 0.05, volume: 0.1 },
  chalk: { rate: 0.08, volume: 0.1 },
  coin: { rate: 0.04, volume: 0.1 },
  till: { rate: 0.03, volume: 0.08 },
  prize: { rate: 0.03, volume: 0.08 },
  dice: { rate: 0.06, volume: 0.1 },
  pour: { rate: 0.05, volume: 0.08 },
  gulp: { rate: 0.06, volume: 0.08 },
  declare: { rate: 0.04, volume: 0.08 },
  kontra: { rate: 0.04, volume: 0.08 },
};

/** The volume setting (0 to 100, ./settings.ts) as a gain: 80 % is the level as designed, a squared taper. */
export function volumeGain(percent: number): number {
  const p = Math.max(0, Math.min(100, Number.isFinite(percent) ? percent : 80));
  return (p / 80) ** 2;
}

/**
 * The smoke test (e2e/smoke.mjs, SMOKE_ONLY=sounds) puts an array at
 * `window.__soundLog` before the app starts; then every sound asked for is
 * noted with what the mix did with it and its start (ms of the audio clock).
 * Without that array nothing is noted.
 */
type SoundOutcome = 'played' | 'ducked' | 'queued' | 'dropped' | 'silent';
function note(name: SoundName, outcome: SoundOutcome, at = 0): void {
  try {
    const log = (window as unknown as { __soundLog?: unknown }).__soundLog;
    if (Array.isArray(log)) log.push({ name, outcome, at: Math.round(at * 1000) });
  } catch {
    // No window (tests).
  }
}

// ------------------------------------------------------------ shared context

let context: AudioContext | null = null;
let unlocked = false;
const unlockListeners: (() => void)[] = [];
const visibilityListeners: ((hidden: boolean) => void)[] = [];

function pageHidden(): boolean {
  try {
    return document.visibilityState === 'hidden';
  } catch {
    return false;
  }
}

/** Whether the player has interacted with the page (audio may start). */
export function audioUnlocked(): boolean {
  return unlocked;
}

/**
 * The shared AudioContext, made on demand once audio is unlocked. Null
 * before the first interaction or when the browser has no Web Audio.
 */
export function audioContext(): AudioContext | null {
  if (!unlocked) return null;
  if (context) return context;
  try {
    const Ctor = window.AudioContext ?? (window as unknown as { webkitAudioContext?: typeof AudioContext }).webkitAudioContext;
    if (!Ctor) return null;
    // Safari 17+: play Web Audio even with the ring/silent switch on, like
    // a media player. Older browsers lack the API and keep their default.
    try {
      const session = (navigator as unknown as { audioSession?: { type: string } }).audioSession;
      if (session) session.type = 'playback';
    } catch {
      // Ignore.
    }
    context = new Ctor();
  } catch {
    context = null;
  }
  return context;
}

/** Resume the context if it waits (after an interaction or a hidden page). */
function resumeContext(): void {
  const ctx = context;
  if (!ctx || pageHidden()) return;
  try {
    // iOS also reports 'interrupted' (after a call or the lock screen).
    if (ctx.state !== 'running' && ctx.state !== 'closed') void ctx.resume().catch(() => undefined);
  } catch {
    // Stay silent.
  }
}

/** Called on every pointer or key event: the first one unlocks audio. */
export function unlockAudio(): void {
  const first = !unlocked;
  unlocked = true;
  if (first) for (const fn of unlockListeners) fn();
  resumeContext();
}

/** Run `fn` once audio is unlocked (at once if it already is). */
export function onAudioUnlock(fn: () => void): void {
  unlockListeners.push(fn);
  if (unlocked) fn();
}

/** Be told when the page is hidden or shown again. */
export function onPageVisibility(fn: (hidden: boolean) => void): void {
  visibilityListeners.push(fn);
}

if (typeof document !== 'undefined') {
  document.addEventListener('visibilitychange', () => {
    const hidden = pageHidden();
    try {
      if (hidden && context?.state === 'running') void context.suspend().catch(() => undefined);
    } catch {
      // Ignore.
    }
    if (!hidden) resumeContext();
    for (const fn of visibilityListeners) fn(hidden);
  });
}

// ------------------------------------------------------------- sound effects

const bigListeners: ((startIn: number, seconds: number) => void)[] = [];

/** Be told when a big sound will start (seconds from now) and how long it counts: the music steps back meanwhile. */
export function onBigSound(fn: (startIn: number, seconds: number) => void): void {
  bigListeners.push(fn);
}

/** How far the music steps back while a big sound sounds. */
export const MUSIC_DUCK = MIX.musicDuck;

class Sound {
  private ctx: AudioContext | null = null;
  private master: GainNode | null = null;
  private buffers = new Map<SoundName, AudioBuffer>();
  private loading = false;
  private enabled = true;
  private volume = 80;
  /** When each sound last started (context time). */
  private lastStart = new Map<SoundName, number>();
  /** The last notable sound's start. */
  private lastNotable = -1;
  /** Until when the last big sound counts as sounding. */
  private bigUntil = -1;

  constructor() {
    onAudioUnlock(() => this.prepare());
  }

  setEnabled(on: boolean): void {
    this.enabled = on;
    if (on) this.prepare();
  }

  /** The player's volume for the sound effects, 0 to 100. */
  setVolume(percent: number): void {
    this.volume = percent;
    try {
      if (this.master && this.ctx) this.master.gain.setTargetAtTime(this.level(), this.ctx.currentTime, 0.02);
    } catch {
      // Ignore.
    }
  }

  private level(): number {
    return 0.9 * volumeGain(this.volume);
  }

  /** Once unlocked and on: take the shared context and load the sounds. */
  private prepare(): void {
    if (!this.enabled || !audioUnlocked()) return;
    try {
      const ctx = audioContext();
      if (!ctx) return;
      if (this.ctx !== ctx) {
        this.ctx = ctx;
        this.master = ctx.createGain();
        this.master.gain.value = this.level();
        // A gentle limiter: should sounds pile up after all, they do not clip.
        let out: AudioNode = ctx.destination;
        try {
          const limit = ctx.createDynamicsCompressor();
          limit.threshold.value = -4;
          limit.knee.value = 4;
          limit.ratio.value = 12;
          limit.attack.value = 0.003;
          limit.release.value = 0.15;
          limit.connect(ctx.destination);
          out = limit;
        } catch {
          // Without one the sounds go straight out.
        }
        this.master.connect(out);
      }
      void this.load();
    } catch {
      this.ctx = null;
    }
  }

  private async load(): Promise<void> {
    const ctx = this.ctx;
    if (!ctx || this.loading) return;
    this.loading = true;
    await Promise.all(
      (Object.keys(URLS) as SoundName[]).map(async (name) => {
        try {
          const res = await fetch(URLS[name]);
          if (!res.ok) return;
          const data = await res.arrayBuffer();
          this.buffers.set(name, await ctx.decodeAudioData(data));
        } catch {
          // This sound stays silent.
        }
      }),
    );
  }

  /**
   * The mix's say on a sound asked to start at `at` (context time): when it
   * starts and at which factor, or null to drop it. Notes the start when it
   * plays.
   */
  private admit(name: SoundName, at: number, length: number): { at: number; factor: number; queued: boolean } | null {
    const priority = PRIORITY[name];
    let queued = false;
    if (priority === 2 && at < this.bigUntil) {
      // A big sound waits for the one sounding, if not for too long.
      if (this.bigUntil - at > MIX.bigWait) return null;
      at = this.bigUntil + 0.04;
      queued = true;
    }
    const last = this.lastStart.get(name);
    const gap = priority === 0 ? MIX.routineGap : MIX.sameGap;
    if (last !== undefined && Math.abs(at - last) < gap) return null;
    const underBig = at < this.bigUntil;
    let factor = 1;
    if (priority === 2) {
      this.bigUntil = at + length * MIX.bigShare;
      for (const fn of bigListeners) fn(Math.max(0, at - (this.ctx?.currentTime ?? at)), length * MIX.bigShare);
    } else if (priority === 1) {
      if (underBig) factor = MIX.underBigNotable;
      else if (Math.abs(at - this.lastNotable) < MIX.notableGap) factor = MIX.notableDuck;
      this.lastNotable = at;
    } else if (underBig) {
      factor = MIX.underBigRoutine;
    }
    this.lastStart.set(name, at);
    return { at, factor, queued };
  }

  /** Play a sound; `rate` shifts the pitch (rising ticks in the score), `delay` in ms. */
  play(name: SoundName, opts: { rate?: number; volume?: number; delay?: number } = {}): void {
    if (!this.enabled) return;
    const ctx = this.ctx;
    const buffer = this.buffers.get(name);
    if (!ctx || !this.master || !buffer || ctx.state !== 'running') {
      note(name, 'silent');
      return;
    }
    try {
      const spread = SPREAD[name];
      const wobble = (amount: number): number => 1 + (Math.random() * 2 - 1) * amount;
      const rate = (opts.rate ?? 1) * (spread ? wobble(spread.rate) : 1);
      const asked = ctx.currentTime + Math.max(0, opts.delay ?? 0) / 1000;
      const slot = this.admit(name, asked, buffer.duration / rate);
      if (!slot) {
        note(name, 'dropped', asked);
        return;
      }
      const { at, factor } = slot;
      note(name, slot.queued ? 'queued' : factor < 1 ? 'ducked' : 'played', at);
      const src = ctx.createBufferSource();
      src.buffer = buffer;
      src.playbackRate.value = rate;
      const gain = ctx.createGain();
      gain.gain.value = VOLUME[name] * (opts.volume ?? 1) * factor * (spread ? wobble(spread.volume) - spread.volume : 1);
      src.connect(gain).connect(this.master);
      src.start(at);
    } catch {
      // Ignore: sound is decoration.
    }
  }
}

export const sound = new Sound();

/**
 * A Stammgast's quip bubble appears (./app/playback/fx.ts): a soft hummed
 * "hm" from his seat, a little lower on the left than on the right.
 */
export function playQuipSound(seat: 1 | 2): void {
  sound.play('quip', { rate: seat === 1 ? 0.9 : 1.1 });
}

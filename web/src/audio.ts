// Sound effects through the Web Audio API. The files are synthesized by
// scripts/make-sounds.mjs and shipped with the app; nothing is loaded from
// other servers. Audio starts only after the first user interaction, as
// browsers require. Any failure keeps the game silent, never broken.
//
// One AudioContext serves the sound effects and the music (./music.ts). It
// is made on the first user interaction, and only if one of them is on; it
// is suspended while the page is hidden.

import cardUrl from './assets/sounds/card.mp3?url';
import clinkUrl from './assets/sounds/clink.mp3?url';
import coinUrl from './assets/sounds/coin.mp3?url';
import knockUrl from './assets/sounds/knock.mp3?url';
import loseUrl from './assets/sounds/lose.mp3?url';
import tickUrl from './assets/sounds/tick.mp3?url';
import trickUrl from './assets/sounds/trick.mp3?url';
import winUrl from './assets/sounds/win.mp3?url';

const URLS = {
  card: cardUrl,
  trick: trickUrl,
  tick: tickUrl,
  clink: clinkUrl,
  coin: coinUrl,
  knock: knockUrl,
  win: winUrl,
  lose: loseUrl,
} as const;

export type SoundName = keyof typeof URLS;

const VOLUME: Record<SoundName, number> = {
  card: 0.55,
  trick: 0.5,
  tick: 0.45,
  clink: 0.5,
  coin: 0.55,
  knock: 0.8,
  win: 0.6,
  lose: 0.6,
};


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
    if (ctx.state === 'suspended') void ctx.resume().catch(() => undefined);
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

class Sound {
  private ctx: AudioContext | null = null;
  private master: GainNode | null = null;
  private buffers = new Map<SoundName, AudioBuffer>();
  private loading = false;
  private enabled = true;

  constructor() {
    onAudioUnlock(() => this.prepare());
  }

  setEnabled(on: boolean): void {
    this.enabled = on;
    if (on) this.prepare();
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
        this.master.gain.value = 0.9;
        this.master.connect(ctx.destination);
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

  /** Play a sound; `rate` shifts the pitch (rising ticks in the score). */
  play(name: SoundName, opts: { rate?: number; volume?: number; delay?: number } = {}): void {
    if (!this.enabled) return;
    const ctx = this.ctx;
    const buffer = this.buffers.get(name);
    if (!ctx || !this.master || !buffer || ctx.state !== 'running') return;
    try {
      const src = ctx.createBufferSource();
      src.buffer = buffer;
      src.playbackRate.value = opts.rate ?? 1;
      const gain = ctx.createGain();
      gain.gain.value = VOLUME[name] * (opts.volume ?? 1);
      src.connect(gain).connect(this.master);
      src.start(ctx.currentTime + (opts.delay ?? 0) / 1000);
    } catch {
      // Ignore: sound is decoration.
    }
  }
}

export const sound = new Sound();

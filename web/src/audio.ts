// Sound effects through the Web Audio API. The files are synthesized by
// scripts/make-sounds.mjs and shipped with the app; nothing is loaded from
// other servers. Audio starts only after the first user interaction, as
// browsers require. Any failure keeps the game silent, never broken.

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

class Sound {
  private ctx: AudioContext | null = null;
  private master: GainNode | null = null;
  private buffers = new Map<SoundName, AudioBuffer>();
  private loading = false;
  private enabled = true;

  setEnabled(on: boolean): void {
    this.enabled = on;
    if (on) this.unlock();
  }

  /** Called on the first pointer or key event: create and resume the context. */
  unlock(): void {
    if (!this.enabled) return;
    try {
      if (!this.ctx) {
        const Ctor = window.AudioContext ?? (window as unknown as { webkitAudioContext?: typeof AudioContext }).webkitAudioContext;
        if (!Ctor) return;
        this.ctx = new Ctor();
        this.master = this.ctx.createGain();
        this.master.gain.value = 0.9;
        this.master.connect(this.ctx.destination);
      }
      if (this.ctx.state === 'suspended') void this.ctx.resume().catch(() => undefined);
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

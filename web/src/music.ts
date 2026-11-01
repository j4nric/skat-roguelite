// Music of the web app (DESIGN 16, "Musik"): quiet chiptune loops for the
// table, the Tresen and the boss series, synthesized with the Web Audio API
// from the note data in ./songs.ts. No audio files.
//
// A small scheduler looks ahead on `AudioContext.currentTime` and places
// every note there, so the timer only has to wake up now and then and
// cannot drift. A change of loop crossfades over about a second. Music
// starts only after the first user interaction (./audio.ts) and pauses
// while the page is hidden. The chosen loop is mirrored in
// `<html data-music="table|shop|boss|off">` for the smoke test.

import { audioContext, audioUnlocked, onAudioUnlock, onPageVisibility } from './audio';
import { SONGS, type Instrument, type MusicId, type Part, type Song, type Wave } from './songs';

export type { MusicId } from './songs';

/**
 * Level of the music: about -33 LUFS, some 10 dB under the sound effects
 * (their master is 0.9). Measured on offline renders (renderLoop).
 */
const MUSIC_VOLUME = 0.18;
/** Crossfade between loops, seconds. */
const CROSSFADE = 1;
/** Fade when the music is switched off, seconds. */
const FADE_OUT = 0.35;
/** How far ahead notes are placed, seconds. */
const LOOKAHEAD = 0.3;
/** How often the scheduler wakes up, milliseconds. */
const TICK_MS = 60;

// ----------------------------------------------------------------- compile

interface NoteEvent {
  kind: 'note';
  /** Start in beats from the start of the loop. */
  t: number;
  /** Length in beats. */
  d: number;
  midi: number;
  inst: Instrument;
}

interface DrumEvent {
  kind: 'drum';
  t: number;
  drum: 'k' | 's' | 'x' | 'o';
  gain: number;
}

type MusicEvent = NoteEvent | DrumEvent;

interface Compiled {
  song: Song;
  /** Length of the loop in beats. */
  length: number;
  /** Seconds per beat. */
  spb: number;
  events: MusicEvent[];
}

const NOTE_BASE: Record<string, number> = { C: 0, D: 2, E: 4, F: 5, G: 7, A: 9, B: 11 };

/** MIDI number of a note name like "C#5" or "Bb4" (C4 = 60). */
function noteMidi(name: string): number | null {
  const m = /^([A-G])([#b]?)(-?\d)$/.exec(name);
  if (!m) return null;
  const base = NOTE_BASE[m[1] ?? ''];
  if (base === undefined) return null;
  const shift = m[2] === '#' ? 1 : m[2] === 'b' ? -1 : 0;
  return 12 * (Number(m[3]) + 1) + base + shift;
}

const QUALITY: Record<string, number[]> = {
  '': [0, 4, 7],
  m: [0, 3, 7],
  dim: [0, 3, 6],
  sus4: [0, 5, 7],
};

/** Root pitch class and intervals of a chord symbol like "Am", "D7", "Bbmaj7". */
function parseChord(symbol: string): { root: number; tones: number[] } | null {
  const m = /^([A-G])([#b]?)(m|dim|sus4|)(7|maj7)?$/.exec(symbol);
  if (!m) return null;
  const base = NOTE_BASE[m[1] ?? ''];
  const triad = QUALITY[m[3] ?? ''];
  if (base === undefined || !triad) return null;
  const shift = m[2] === '#' ? 1 : m[2] === 'b' ? -1 : 0;
  const tones = [...triad];
  if (m[4] === '7') tones.push(10);
  if (m[4] === 'maj7') tones.push(11);
  return { root: (base + shift + 12) % 12, tones };
}

/** The chord sounding at beat `beat` of bar `bar`. */
function chordAt(song: Song, bar: number, beat: number): { root: number; tones: number[] } | null {
  const symbols = (song.chords[bar % song.chords.length] ?? '').split(/\s+/).filter(Boolean);
  if (!symbols.length) return null;
  const i = Math.min(symbols.length - 1, Math.floor((beat / song.beats) * symbols.length));
  return parseChord(symbols[i] ?? '');
}

/** MIDI notes of a chord-relative token ("1", "3,", "1'", "c"). */
function chordTokenNotes(token: string, chord: { root: number; tones: number[] }, base: number): number[] {
  const root = base + ((((chord.root - base) % 12) + 12) % 12);
  const at = (k: number): number => {
    const n = chord.tones.length;
    return root + (chord.tones[(k - 1) % n] ?? 0) + 12 * Math.floor((k - 1) / n);
  };
  if (token === 'c') return chord.tones.map((_, i) => at(i + 1));
  const m = /^([1-9])([',]*)$/.exec(token);
  if (!m) return [];
  let shift = 0;
  for (const ch of m[2] ?? '') shift += ch === "'" ? 12 : -12;
  return [at(Number(m[1])) + shift];
}

/** Turn the bars of a part into note events; tokens it cannot read are rests. */
function compilePart(song: Song, part: Part, relative: boolean, out: MusicEvent[], problems: string[], name: string): void {
  const steps = song.beats * part.div;
  const bars = relative ? song.chords.length : part.bars.length;
  let open: NoteEvent[] = [];
  for (let bar = 0; bar < bars; bar++) {
    const tokens = (part.bars[bar % part.bars.length] ?? '').split(/\s+/).filter(Boolean);
    if (tokens.length !== steps) problems.push(`${song.id} ${name} bar ${bar + 1}: ${tokens.length} steps, expected ${steps}`);
    tokens.forEach((token, step) => {
      const t = bar * song.beats + step / part.div;
      if (token === '-') {
        for (const n of open) n.d += 1 / part.div;
        return;
      }
      open = [];
      if (token === '.') return;
      let midis: number[];
      if (relative) {
        const chord = chordAt(song, bar, step / part.div);
        midis = chord ? chordTokenNotes(token, chord, part.base ?? 48) : [];
      } else {
        const midi = noteMidi(token);
        midis = midi === null ? [] : [midi];
      }
      if (!midis.length) problems.push(`${song.id} ${name} bar ${bar + 1}: cannot read "${token}"`);
      for (const midi of midis) {
        const ev: NoteEvent = { kind: 'note', t, d: 1 / part.div, midi, inst: part.inst };
        open.push(ev);
        out.push(ev);
      }
    });
  }
}

/** Compile a song into one sorted list of events; `problems` collects notation errors. */
export function compileSong(song: Song, problems: string[] = []): Compiled {
  const events: MusicEvent[] = [];
  if (song.lead.bars.length !== song.chords.length) {
    problems.push(`${song.id}: ${song.lead.bars.length} lead bars, ${song.chords.length} chord bars`);
  }
  for (const [bar, symbols] of song.chords.entries()) {
    for (const symbol of symbols.split(/\s+/).filter(Boolean)) {
      if (!parseChord(symbol)) problems.push(`${song.id} chord bar ${bar + 1}: cannot read "${symbol}"`);
    }
  }
  compilePart(song, song.lead, false, events, problems, 'lead');
  compilePart(song, song.bass, true, events, problems, 'bass');
  compilePart(song, song.harmony, true, events, problems, 'harmony');
  const drumSteps = song.beats * 4;
  for (let bar = 0; bar < song.chords.length; bar++) {
    const pattern = song.drums.bars[bar % song.drums.bars.length] ?? '';
    if (pattern.length !== drumSteps) problems.push(`${song.id} drums bar ${bar + 1}: ${pattern.length} steps, expected ${drumSteps}`);
    [...pattern].forEach((ch, step) => {
      if (ch === 'k' || ch === 's' || ch === 'x' || ch === 'o') {
        events.push({ kind: 'drum', t: bar * song.beats + step / 4, drum: ch, gain: song.drums.gain });
      }
    });
  }
  events.sort((a, b) => a.t - b.t);
  return { song, length: song.chords.length * song.beats, spb: 60 / song.bpm, events };
}

// --------------------------------------------------------------- synthesis

interface Voices {
  waves: Partial<Record<Wave, PeriodicWave>>;
  noise: AudioBuffer;
}

const voiceCache = new WeakMap<BaseAudioContext, Voices>();

/** Fourier series of a pulse wave with the given duty cycle. */
function pulseWave(ctx: BaseAudioContext, duty: number): PeriodicWave {
  const n = 64;
  const real = new Float32Array(n);
  const imag = new Float32Array(n);
  for (let k = 1; k < n; k++) real[k] = (2 / (k * Math.PI)) * Math.sin(k * Math.PI * duty);
  return ctx.createPeriodicWave(real, imag);
}

function voices(ctx: BaseAudioContext): Voices {
  let v = voiceCache.get(ctx);
  if (v) return v;
  // One second of white noise from a fixed seed (renders are repeatable).
  const noise = ctx.createBuffer(1, ctx.sampleRate, ctx.sampleRate);
  const data = noise.getChannelData(0);
  let seed = 0x5eed1234;
  for (let i = 0; i < data.length; i++) {
    seed ^= seed << 13;
    seed ^= seed >>> 17;
    seed ^= seed << 5;
    data[i] = ((seed >>> 0) / 0xffffffff) * 2 - 1;
  }
  v = { waves: { pulse25: pulseWave(ctx, 0.25), pulse12: pulseWave(ctx, 0.125) }, noise };
  voiceCache.set(ctx, v);
  return v;
}

function oscillator(ctx: BaseAudioContext, wave: Wave): OscillatorNode {
  const osc = ctx.createOscillator();
  if (wave === 'square' || wave === 'triangle') {
    osc.type = wave;
  } else {
    const periodic = voices(ctx).waves[wave];
    if (periodic) osc.setPeriodicWave(periodic);
    else osc.type = 'square';
  }
  return osc;
}

/** One note at time `t` (seconds) for `dur` seconds. */
function playNote(ctx: BaseAudioContext, out: AudioNode, inst: Instrument, midi: number, t: number, dur: number): void {
  const freq = 440 * 2 ** ((midi - 69) / 12);
  const env = ctx.createGain();
  const peak = inst.gain;
  const hold = Math.max(inst.attack, dur * inst.legato);
  const end = t + hold + inst.release * 6;
  env.gain.setValueAtTime(0, t);
  env.gain.linearRampToValueAtTime(peak, t + inst.attack);
  env.gain.setTargetAtTime(peak * inst.sustain, t + inst.attack, inst.decay);
  env.gain.setTargetAtTime(0, t + hold, inst.release);
  env.connect(out);
  const detunes = inst.chorus ? [-inst.chorus, inst.chorus] : [0];
  const share = 1 / detunes.length;
  let lfoGain: GainNode | null = null;
  if (inst.vibrato && hold > inst.vibrato.delay + 0.1) {
    const lfo = ctx.createOscillator();
    lfo.frequency.value = inst.vibrato.rate;
    lfoGain = ctx.createGain();
    lfoGain.gain.setValueAtTime(0, t);
    lfoGain.gain.setValueAtTime(0, t + inst.vibrato.delay);
    lfoGain.gain.linearRampToValueAtTime(inst.vibrato.cents, t + inst.vibrato.delay + 0.25);
    lfo.connect(lfoGain);
    lfo.start(t);
    lfo.stop(end);
  }
  for (const cents of detunes) {
    const osc = oscillator(ctx, inst.wave);
    osc.frequency.value = freq;
    osc.detune.value = cents;
    if (lfoGain) lfoGain.connect(osc.detune);
    const g = ctx.createGain();
    g.gain.value = share;
    osc.connect(g).connect(env);
    osc.start(t);
    osc.stop(end);
  }
}

/** A kick, snare or hi-hat at time `t`. */
function playDrum(ctx: BaseAudioContext, out: AudioNode, drum: DrumEvent['drum'], level: number, t: number): void {
  const env = ctx.createGain();
  env.connect(out);
  if (drum === 'k') {
    const osc = ctx.createOscillator();
    osc.type = 'sine';
    osc.frequency.setValueAtTime(130, t);
    osc.frequency.exponentialRampToValueAtTime(42, t + 0.12);
    env.gain.setValueAtTime(0, t);
    env.gain.linearRampToValueAtTime(0.42 * level, t + 0.004);
    env.gain.setTargetAtTime(0, t + 0.02, 0.06);
    osc.connect(env);
    osc.start(t);
    osc.stop(t + 0.4);
    return;
  }
  const src = ctx.createBufferSource();
  src.buffer = voices(ctx).noise;
  const filter = ctx.createBiquadFilter();
  let peak: number;
  let length: number;
  if (drum === 's') {
    filter.type = 'bandpass';
    filter.frequency.value = 1900;
    filter.Q.value = 0.7;
    peak = 0.16;
    length = 0.05;
  } else {
    filter.type = 'highpass';
    filter.frequency.value = 7000;
    peak = drum === 'o' ? 0.06 : 0.07;
    length = drum === 'o' ? 0.08 : 0.018;
  }
  env.gain.setValueAtTime(0, t);
  env.gain.linearRampToValueAtTime(peak * level, t + 0.002);
  env.gain.setTargetAtTime(0, t + 0.004, length);
  src.connect(filter).connect(env);
  // Start somewhere in the noise so that hits do not sound identical.
  const offset = (t * 7.31) % 0.8;
  src.start(t, offset);
  src.stop(t + length * 7);
}

// ------------------------------------------------------------------ player

/** One loop playing into its own fader. */
class LoopPlayer {
  readonly fader: GainNode;
  private readonly filter: BiquadFilterNode;
  private loopStart: number;
  private cursor = 0;
  /** Context time after which this player is silent and can go. */
  doneAt = Infinity;

  constructor(
    private readonly ctx: BaseAudioContext,
    readonly compiled: Compiled,
    out: AudioNode,
    at: number,
    fadeIn: number,
  ) {
    this.loopStart = at;
    this.filter = ctx.createBiquadFilter();
    this.filter.type = 'lowpass';
    this.filter.frequency.value = compiled.song.cutoff;
    this.filter.Q.value = 0.5;
    this.fader = ctx.createGain();
    const level = compiled.song.level;
    if (fadeIn > 0) {
      this.fader.gain.setValueAtTime(0, at);
      this.fader.gain.linearRampToValueAtTime(level, at + fadeIn);
    } else {
      this.fader.gain.value = level;
    }
    this.filter.connect(this.fader).connect(out);
  }

  get id(): MusicId {
    return this.compiled.song.id;
  }

  /** Place every event that starts before `horizon` (context seconds). */
  pump(horizon: number): void {
    const { events, spb, length } = this.compiled;
    if (!events.length) return;
    const now = this.ctx.currentTime;
    for (let guard = 0; guard < 4096; guard++) {
      const ev = events[this.cursor];
      if (!ev) {
        this.cursor = 0;
        this.loopStart += length * spb;
        continue;
      }
      const at = this.loopStart + ev.t * spb;
      if (at >= horizon || at >= this.doneAt) return;
      this.cursor += 1;
      if (at < now - 0.01) continue;
      try {
        if (ev.kind === 'note') playNote(this.ctx, this.filter, ev.inst, ev.midi, at, ev.d * spb);
        else playDrum(this.ctx, this.filter, ev.drum, ev.gain, at);
      } catch {
        // A note that cannot be made stays silent.
      }
    }
  }

  fadeOut(seconds: number): void {
    const now = this.ctx.currentTime;
    const g = this.fader.gain;
    g.cancelScheduledValues(now);
    g.setValueAtTime(g.value, now);
    g.linearRampToValueAtTime(0, now + seconds);
    this.doneAt = now + seconds;
  }

  dispose(): void {
    try {
      this.fader.disconnect();
    } catch {
      // Already gone.
    }
  }
}

// -------------------------------------------------------------- controller

class Music {
  private enabled = true;
  private scene: MusicId | null = null;
  private ctx: AudioContext | null = null;
  private master: GainNode | null = null;
  private current: LoopPlayer | null = null;
  private fading: LoopPlayer[] = [];
  private timer: number | undefined;
  private hidden = false;
  private compiled = new Map<MusicId, Compiled>();

  constructor() {
    if (import.meta.env.DEV) {
      const problems = songProblems();
      if (problems.length) console.warn('music notation:', problems);
    }
    onAudioUnlock(() => this.update());
    onPageVisibility((hidden) => {
      this.hidden = hidden;
      if (hidden) this.stopTimer();
      else this.update();
    });
    this.mirror();
  }

  setEnabled(on: boolean): void {
    if (this.enabled === on) return;
    this.enabled = on;
    this.update();
  }

  /** The loop the screen asks for; null for none. */
  setScene(scene: MusicId | null): void {
    if (this.scene === scene) return;
    this.scene = scene;
    this.update();
  }

  /** The loop that should sound now, if any. */
  private wanted(): MusicId | null {
    return this.enabled && audioUnlocked() ? this.scene : null;
  }

  private update(): void {
    const want = this.wanted();
    try {
      if (want && want !== this.current?.id) this.start(want);
      else if (!want && this.current) this.fadeCurrent(FADE_OUT);
    } catch {
      this.current = null;
    }
    this.mirror();
    if (this.current || this.fading.length) this.startTimer();
  }

  private context(): AudioContext | null {
    const ctx = audioContext();
    if (!ctx) return null;
    if (ctx !== this.ctx) {
      this.ctx = ctx;
      this.master = ctx.createGain();
      this.master.gain.value = MUSIC_VOLUME;
      this.master.connect(ctx.destination);
    }
    return ctx;
  }

  private start(id: MusicId): void {
    const ctx = this.context();
    if (!ctx || !this.master) return;
    let compiled = this.compiled.get(id);
    if (!compiled) {
      compiled = compileSong(SONGS[id]);
      this.compiled.set(id, compiled);
    }
    const fadeIn = this.current ? CROSSFADE : CROSSFADE * 0.6;
    this.fadeCurrent(CROSSFADE);
    this.current = new LoopPlayer(ctx, compiled, this.master, ctx.currentTime + 0.06, fadeIn);
  }

  private fadeCurrent(seconds: number): void {
    const old = this.current;
    if (!old) return;
    this.current = null;
    old.fadeOut(seconds);
    this.fading.push(old);
  }

  private startTimer(): void {
    if (this.timer !== undefined || this.hidden) return;
    const tick = (): void => {
      this.timer = undefined;
      const ctx = this.ctx;
      if (!ctx || this.hidden) return;
      const now = ctx.currentTime;
      // A suspended context does not advance: wait without placing notes.
      if (ctx.state === 'running') {
        this.current?.pump(now + LOOKAHEAD);
        for (const p of this.fading) p.pump(now + LOOKAHEAD);
      }
      const gone = this.fading.filter((p) => now > p.doneAt + 0.5);
      for (const p of gone) p.dispose();
      this.fading = this.fading.filter((p) => !gone.includes(p));
      if (this.current || this.fading.length) this.timer = window.setTimeout(tick, TICK_MS);
    };
    tick();
  }

  private stopTimer(): void {
    window.clearTimeout(this.timer);
    this.timer = undefined;
  }

  /** `<html data-music>`: the loop chosen to sound, or "off". */
  private mirror(): void {
    try {
      document.documentElement.dataset.music = this.current?.id ?? 'off';
    } catch {
      // No document (tests): nothing to mirror.
    }
  }
}

export const music = new Music();

/**
 * Render `seconds` of a loop offline (for listening to it outside the app,
 * e2e/smoke.mjs). Returns the mono samples at `sampleRate`, music volume
 * applied as in the game.
 */
export async function renderLoop(id: MusicId, seconds: number, sampleRate = 44100): Promise<Float32Array> {
  const ctx = new OfflineAudioContext(1, Math.ceil(seconds * sampleRate), sampleRate);
  const master = ctx.createGain();
  master.gain.value = MUSIC_VOLUME;
  master.connect(ctx.destination);
  const player = new LoopPlayer(ctx, compileSong(SONGS[id]), master, 0, 0);
  player.pump(seconds);
  const buffer = await ctx.startRendering();
  return buffer.getChannelData(0);
}

/** Notation errors of all songs (empty when every bar reads cleanly). */
export function songProblems(): string[] {
  const problems: string[] = [];
  for (const song of Object.values(SONGS)) compileSong(song, problems);
  return problems;
}

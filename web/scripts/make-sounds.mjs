// Synthesizes the sound effects of the web app and writes them as small
// MP3 files to src/assets/sounds/. Everything is generated from scratch
// (noise, sine partials, a little "accordion"), so the sounds belong to
// this project and need no third-party licence.
//
//   node scripts/make-sounds.mjs        (needs ffmpeg with libmp3lame)
//
// The output is deterministic: noise comes from a seeded generator.

import { execFileSync } from 'node:child_process';
import { mkdirSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const RATE = 44100;
const here = dirname(fileURLToPath(import.meta.url));
const out = resolve(here, '..', 'src', 'assets', 'sounds');

// --------------------------------------------------------------- helpers

let seed = 0x2f6e2b1;
function noise() {
  // xorshift32, mapped to -1..1
  seed ^= seed << 13;
  seed ^= seed >>> 17;
  seed ^= seed << 5;
  return ((seed >>> 0) / 0xffffffff) * 2 - 1;
}

function buffer(seconds) {
  return new Float32Array(Math.ceil(seconds * RATE));
}

/** RBJ biquad band-pass, processed in place over a range. */
function bandpass(data, freqAt, q) {
  let x1 = 0, x2 = 0, y1 = 0, y2 = 0;
  for (let i = 0; i < data.length; i++) {
    const f = typeof freqAt === 'function' ? freqAt(i / RATE) : freqAt;
    const w = (2 * Math.PI * f) / RATE;
    const alpha = Math.sin(w) / (2 * q);
    const a0 = 1 + alpha;
    const b0 = alpha / a0, b2 = -alpha / a0;
    const a1 = (-2 * Math.cos(w)) / a0, a2 = (1 - alpha) / a0;
    const x = data[i];
    const y = b0 * x + b2 * x2 - a1 * y1 - a2 * y2;
    x2 = x1; x1 = x; y2 = y1; y1 = y;
    data[i] = y;
  }
  return data;
}

/** One-pole low-pass, in place. */
function lowpass(data, freq) {
  const k = 1 - Math.exp((-2 * Math.PI * freq) / RATE);
  let y = 0;
  for (let i = 0; i < data.length; i++) {
    y += k * (data[i] - y);
    data[i] = y;
  }
  return data;
}

/** Add `src` into `dst` at time `at` with gain. */
function mix(dst, src, at = 0, gain = 1) {
  const start = Math.round(at * RATE);
  for (let i = 0; i < src.length && start + i < dst.length; i++) dst[start + i] += src[i] * gain;
  return dst;
}

function normalize(data, peak = 0.89) {
  let max = 0;
  for (const v of data) max = Math.max(max, Math.abs(v));
  if (max > 0) for (let i = 0; i < data.length; i++) data[i] *= peak / max;
  // Short fades against clicks.
  const fade = Math.min(data.length >> 1, Math.round(0.004 * RATE));
  for (let i = 0; i < fade; i++) {
    data[data.length - 1 - i] *= i / fade;
  }
  return data;
}

/** Decaying sine partials: [freq, gain, decay seconds]. */
function partials(seconds, list, attack = 0.001) {
  const data = buffer(seconds);
  for (const [freq, gain, decay] of list) {
    const phase = noise() * Math.PI;
    for (let i = 0; i < data.length; i++) {
      const t = i / RATE;
      const env = Math.min(1, t / attack) * Math.exp(-t / decay);
      data[i] += gain * env * Math.sin(2 * Math.PI * freq * t + phase);
    }
  }
  return data;
}

/** A noise burst with exponential decay. */
function burst(seconds, decay, attack = 0.0015) {
  const data = buffer(seconds);
  for (let i = 0; i < data.length; i++) {
    const t = i / RATE;
    data[i] = noise() * Math.min(1, t / attack) * Math.exp(-t / decay);
  }
  return data;
}

// ---------------------------------------------------------------- sounds

/** A card laid on the felt: a soft slap and a low thump. */
function card() {
  const out = buffer(0.16);
  mix(out, bandpass(burst(0.12, 0.014), 2600, 0.9), 0, 1);
  mix(out, bandpass(burst(0.12, 0.03, 0.004), 900, 0.7), 0.002, 0.5);
  mix(out, partials(0.1, [[150, 0.5, 0.018]]), 0, 1);
  return normalize(lowpass(out, 9000), 0.8);
}

/** A trick swept together: three quick slides and a tap. */
function trick() {
  const out = buffer(0.42);
  for (let k = 0; k < 3; k++) {
    const slide = buffer(0.16);
    for (let i = 0; i < slide.length; i++) {
      const t = i / RATE;
      const env = Math.sin(Math.PI * Math.min(1, t / 0.15)) ** 2;
      slide[i] = noise() * env;
    }
    bandpass(slide, (t) => 1400 + t * 16000, 1.4);
    mix(out, slide, k * 0.07, 0.7 - k * 0.12);
  }
  mix(out, bandpass(burst(0.1, 0.012), 2200, 1), 0.27, 0.9);
  mix(out, partials(0.1, [[130, 0.4, 0.02]]), 0.27, 1);
  return normalize(out, 0.75);
}

/** Chalk tap on the board for each step of the score. */
function tick() {
  const out = buffer(0.14);
  mix(out, partials(0.14, [[1180, 0.7, 0.022], [2630, 0.35, 0.012], [520, 0.3, 0.03]]), 0, 1);
  mix(out, bandpass(burst(0.03, 0.003), 4200, 1.2), 0, 0.6);
  return normalize(out, 0.7);
}

/** Two beer glasses clinking: a relic lights up. */
function clink() {
  const out = buffer(0.8);
  const glass = (detune) =>
    partials(0.75, [
      [2637 * detune, 0.5, 0.32],
      [3951 * detune, 0.35, 0.22],
      [5274 * detune, 0.25, 0.14],
      [6912 * detune, 0.15, 0.09],
      [1318 * detune, 0.12, 0.25],
    ]);
  mix(out, glass(1), 0, 1);
  mix(out, glass(1.031), 0.006, 0.8);
  mix(out, bandpass(burst(0.02, 0.002), 6000, 1), 0, 0.5);
  return normalize(out, 0.6);
}

/** Groschen: a coin dropped on the counter, bouncing twice. */
function coin() {
  const out = buffer(0.5);
  const hit = () =>
    partials(0.3, [
      [3150, 0.5, 0.09],
      [4720, 0.4, 0.06],
      [6340, 0.25, 0.04],
      [2100, 0.2, 0.05],
    ]);
  mix(out, hit(), 0, 1);
  mix(out, hit(), 0.11, 0.55);
  mix(out, hit(), 0.18, 0.3);
  mix(out, hit(), 0.22, 0.15);
  return normalize(out, 0.6);
}

/** Two knocks on the wooden table: the result lands. */
function knock() {
  const out = buffer(0.42);
  const one = () => {
    const data = buffer(0.25);
    for (let i = 0; i < data.length; i++) {
      const t = i / RATE;
      const f = 70 + 90 * Math.exp(-t / 0.02);
      data[i] = Math.sin(2 * Math.PI * f * t) * Math.exp(-t / 0.05);
    }
    mix(data, partials(0.25, [[410, 0.35, 0.03], [860, 0.15, 0.02]]), 0, 1);
    mix(data, bandpass(burst(0.05, 0.006), 1800, 0.8), 0, 0.5);
    return data;
  };
  mix(out, one(), 0, 1);
  mix(out, one(), 0.15, 0.85);
  return normalize(out, 0.95);
}

/** A tiny accordion: detuned reeds through a soft filter with tremolo. */
function reeds(freq, seconds, gain = 1, attack = 0.03, release = 0.12) {
  const data = buffer(seconds + release);
  const detunes = [1, 1.004, 0.997];
  for (const d of detunes) {
    let phase = 0;
    for (let i = 0; i < data.length; i++) {
      const t = i / RATE;
      phase += (freq * d) / RATE;
      const saw = 2 * (phase % 1) - 1;
      const square = phase % 1 < 0.5 ? 0.6 : -0.6;
      data[i] += (saw * 0.6 + square * 0.4) / detunes.length;
    }
  }
  for (let i = 0; i < data.length; i++) {
    const t = i / RATE;
    const env = Math.min(1, t / attack) * (t > seconds ? Math.exp(-(t - seconds) / (release / 3)) : 1);
    const tremolo = 1 + 0.12 * Math.sin(2 * Math.PI * 5.5 * t);
    data[i] *= env * tremolo * gain;
  }
  return lowpass(lowpass(data, 2400), 3200);
}

const NOTE = (n) => 440 * 2 ** ((n - 69) / 12);

/** Sieg: a short squeezebox flourish ending in a major chord. */
function win() {
  const out = buffer(1.35);
  const run = [67, 72, 76];
  run.forEach((n, k) => mix(out, reeds(NOTE(n), 0.1, 0.8), k * 0.12, 1));
  const chord = [60, 64, 67, 72, 79];
  for (const n of chord) mix(out, reeds(NOTE(n), 0.62, n === 79 ? 0.5 : 0.7, 0.02, 0.3), 0.36, 1);
  return normalize(out, 0.7);
}

/** Niederlage: a sagging squeezebox, down a semitone at a time. */
function lose() {
  const out = buffer(1.5);
  const steps = [67, 66, 65];
  steps.forEach((n, k) => {
    mix(out, reeds(NOTE(n), 0.22, 0.8), k * 0.26, 1);
    mix(out, reeds(NOTE(n - 9), 0.22, 0.5), k * 0.26, 1);
  });
  const last = buffer(0.8);
  // Last note bends down.
  let phase = 0;
  for (let i = 0; i < last.length; i++) {
    const t = i / RATE;
    const f = NOTE(64) * (1 - 0.03 * Math.min(1, t / 0.6)) * (1 + 0.006 * Math.sin(2 * Math.PI * 4 * t));
    phase += f / RATE;
    const saw = 2 * (phase % 1) - 1;
    last[i] = saw * Math.min(1, t / 0.03) * Math.exp(-t / 0.45);
  }
  mix(out, lowpass(lowpass(last, 1800), 2400), 0.78, 0.8);
  mix(out, reeds(NOTE(52), 0.5, 0.45, 0.03, 0.3), 0.78, 1);
  return normalize(out, 0.65);
}

// ---------------------------------------------------------------- output

function wav(data) {
  const bytes = Buffer.alloc(44 + data.length * 2);
  bytes.write('RIFF', 0);
  bytes.writeUInt32LE(36 + data.length * 2, 4);
  bytes.write('WAVEfmt ', 8);
  bytes.writeUInt32LE(16, 16);
  bytes.writeUInt16LE(1, 20);
  bytes.writeUInt16LE(1, 22);
  bytes.writeUInt32LE(RATE, 24);
  bytes.writeUInt32LE(RATE * 2, 28);
  bytes.writeUInt16LE(2, 32);
  bytes.writeUInt16LE(16, 34);
  bytes.write('data', 36);
  bytes.writeUInt32LE(data.length * 2, 40);
  for (let i = 0; i < data.length; i++) {
    const v = Math.max(-1, Math.min(1, data[i]));
    bytes.writeInt16LE(Math.round(v * 32767), 44 + i * 2);
  }
  return bytes;
}

const sounds = { card, trick, tick, clink, coin, knock, win, lose };
mkdirSync(out, { recursive: true });
const tmp = join(tmpdir(), `stammtisch-sounds-${process.pid}`);
mkdirSync(tmp, { recursive: true });
try {
  for (const [name, make] of Object.entries(sounds)) {
    const file = join(tmp, `${name}.wav`);
    writeFileSync(file, wav(make()));
    execFileSync('ffmpeg', ['-y', '-loglevel', 'error', '-i', file, '-ac', '1', '-codec:a', 'libmp3lame', '-b:a', '64k', '-map_metadata', '-1', join(out, `${name}.mp3`)]);
    console.log(`wrote ${name}.mp3`);
  }
} finally {
  rmSync(tmp, { recursive: true, force: true });
}

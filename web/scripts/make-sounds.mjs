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

/** RBJ biquad high-pass, processed in place. */
function highpass(data, freq, q) {
  const w = (2 * Math.PI * freq) / RATE;
  const alpha = Math.sin(w) / (2 * q);
  const cos = Math.cos(w);
  const a0 = 1 + alpha;
  const b0 = (1 + cos) / 2 / a0, b1 = -(1 + cos) / a0, b2 = (1 + cos) / 2 / a0;
  const a1 = (-2 * cos) / a0, a2 = (1 - alpha) / a0;
  let x1 = 0, x2 = 0, y1 = 0, y2 = 0;
  for (let i = 0; i < data.length; i++) {
    const x = data[i];
    const y = b0 * x + b1 * x1 + b2 * x2 - a1 * y1 - a2 * y2;
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

/**
 * A card snapped onto the table: a short, bright, papery "schnipp". A
 * high-passed noise flick with a near-instant attack, a faint crisp paper
 * tail and a tiny pitched click of the card edge; no low thump.
 */
function card() {
  const start = seed;
  const out = buffer(0.09);
  // The flick: very fast attack, ~5 ms decay, nothing below ~1.8 kHz.
  const flick = highpass(highpass(burst(0.06, 0.0045, 0.0003), 1800, 0.7), 1800, 0.7);
  mix(out, bandpass(flick, 4300, 0.6), 0, 1);
  // Paper: a softer, slightly longer rustle up high.
  mix(out, bandpass(burst(0.08, 0.016, 0.0008), 6800, 1.3), 0.001, 0.22);
  // The card edge: a tiny pitched click.
  mix(out, partials(0.03, [[2950, 0.22, 0.0035], [5300, 0.1, 0.002]], 0.0003), 0, 1);
  // A hint of body so it is not only hiss.
  mix(out, partials(0.03, [[640, 0.08, 0.006]], 0.0005), 0, 1);
  // Leave the shared noise sequence where the earlier card sound left it,
  // so the other sounds stay byte for byte as shipped.
  seed = start;
  for (let i = 0; i < 10585; i++) noise();
  return normalize(lowpass(out, 12000), 0.8);
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

// ------------------------------------------------------ more sounds (v2)
//
// Each sound below starts its own noise sequence, so changing one leaves
// the others (and the eight above) as they were.

function reseed(n) {
  seed = n >>> 0 || 1;
}

/** A random number in [a, b) from the shared noise sequence. */
function between(a, b) {
  return a + ((noise() + 1) / 2) * (b - a);
}

/** One short knock on wood: a falling thump, wood partials, a click. */
function woodHit(seconds, { low = 70, drop = 90, body = 0.05, ring = 410, click = 1800, clickGain = 0.5 } = {}) {
  const data = buffer(seconds);
  for (let i = 0; i < data.length; i++) {
    const t = i / RATE;
    const f = low + drop * Math.exp(-t / 0.02);
    data[i] = Math.sin(2 * Math.PI * f * t) * Math.exp(-t / body);
  }
  mix(data, partials(seconds, [[ring, 0.35, 0.03], [ring * 2.1, 0.15, 0.02]]), 0, 1);
  mix(data, bandpass(burst(Math.min(seconds, 0.05), 0.006), click, 0.8), 0, clickGain);
  return data;
}

/** One card flicked: a much smaller "card" for riffles and slams. */
function flick(gain = 1, bright = 4300) {
  const data = buffer(0.05);
  mix(data, bandpass(highpass(burst(0.05, 0.003, 0.0003), 1600, 0.7), bright, 0.7), 0, gain);
  return data;
}

/** A card slid over the felt: a short swish rising in brightness. */
function slide(seconds, from, to, gain = 1) {
  const data = buffer(seconds);
  for (let i = 0; i < data.length; i++) {
    const t = i / RATE;
    data[i] = noise() * Math.sin(Math.PI * Math.min(1, t / seconds)) ** 2 * gain;
  }
  return bandpass(data, (t) => from + (to - from) * (t / seconds), 1.2);
}

/** A soft UI tap: a tiny wooden tick, quiet enough to hear on every button. */
function tap() {
  reseed(0x51a7e01);
  const out = buffer(0.06);
  mix(out, partials(0.06, [[1850, 0.5, 0.005], [3300, 0.18, 0.0025], [900, 0.15, 0.006]], 0.0004), 0, 1);
  mix(out, bandpass(burst(0.02, 0.0015, 0.0002), 5200, 1), 0, 0.25);
  return normalize(lowpass(out, 9000), 0.5);
}

/** Shuffle before the deal: a riffle of many small flicks, then the deck squared on the table. */
function shuffle() {
  reseed(0x5f0ff1e);
  const out = buffer(0.56);
  // The riffle: flicks close together in the middle, sparser at both ends.
  let t = 0.01;
  for (let k = 0; t < 0.38; k++) {
    const p = t / 0.38;
    mix(out, flick(0.35 + 0.65 * Math.sin(Math.PI * p) * between(0.6, 1), between(3600, 5200)), t, 1);
    t += 0.006 + 0.016 * (1 - Math.sin(Math.PI * p)) + between(0, 0.004);
  }
  // A soft rush of paper under it.
  mix(out, slide(0.38, 2500, 5500, 0.25), 0.01, 1);
  // Squared: two light taps of the deck's edge on the wood.
  mix(out, woodHit(0.12, { low: 140, drop: 80, body: 0.02, ring: 620, click: 2600, clickGain: 0.3 }), 0.43, 0.45);
  mix(out, woodHit(0.1, { low: 150, drop: 70, body: 0.015, ring: 680, click: 2800, clickGain: 0.25 }), 0.49, 0.3);
  return normalize(lowpass(out, 11000), 0.7);
}

/** The skat picked up: two cards slid off the felt into the hand. */
function pickup() {
  reseed(0x0b1c4a9);
  const out = buffer(0.26);
  mix(out, slide(0.12, 1500, 4800, 1), 0, 0.9);
  mix(out, slide(0.12, 1700, 5200, 1), 0.08, 0.75);
  mix(out, flick(0.4, 3800), 0.2, 1);
  return normalize(out, 0.6);
}

/** Drücken: two cards pressed face down onto the felt, muffled pats. */
function discard() {
  reseed(0x0d15ca7);
  const out = buffer(0.22);
  const pat = () => {
    const data = lowpass(burst(0.08, 0.012, 0.0008), 2200);
    mix(data, partials(0.08, [[210, 0.6, 0.02], [470, 0.2, 0.012]]), 0, 1);
    return data;
  };
  mix(out, pat(), 0, 1);
  mix(out, pat(), 0.095, 0.8);
  return normalize(out, 0.6);
}

/** A stroke of chalk on the slate: grainy, bright, short. */
function chalkStroke(seconds, gain = 1) {
  const data = buffer(seconds);
  let grain = 0;
  for (let i = 0; i < data.length; i++) {
    const t = i / RATE;
    if (i % 90 === 0) grain = 0.4 + 0.6 * Math.abs(noise());
    data[i] = noise() * grain * Math.sin(Math.PI * Math.min(1, t / seconds)) ** 0.7 * gain;
  }
  return bandpass(bandpass(data, 3300, 1.6), 3600, 1.2);
}

/** The game declared: a firm knuckle rap and a stroke of chalk. */
function declare() {
  reseed(0xdec1a4e);
  const out = buffer(0.32);
  mix(out, woodHit(0.2, { low: 90, drop: 110, body: 0.035, ring: 520, click: 2200, clickGain: 0.6 }), 0, 1);
  mix(out, chalkStroke(0.16, 1), 0.12, 0.35);
  return normalize(out, 0.85);
}

/** Grand, Null, Hand or announced Schneider: a fist on the table, the glasses rattle. */
function bang() {
  reseed(0xb16dec1);
  const out = buffer(0.72);
  mix(out, woodHit(0.4, { low: 50, drop: 90, body: 0.09, ring: 300, click: 1400, clickGain: 0.7 }), 0, 1);
  // The glasses on the table jump: a few small, high clinks.
  for (let k = 0; k < 5; k++) {
    const f = between(2400, 3600);
    mix(out, partials(0.25, [[f, 0.3, 0.05], [f * 1.51, 0.18, 0.03], [f * 2.03, 0.1, 0.02]]), 0.03 + k * 0.045 + between(0, 0.02), 0.22 - k * 0.03);
  }
  return normalize(out, 0.92);
}

/** An Unter slammed onto the trick: a sharp card snap with a thump under it. */
function slam() {
  reseed(0x51a33ed);
  const out = buffer(0.22);
  mix(out, flick(1, 3800), 0, 1);
  mix(out, woodHit(0.2, { low: 80, drop: 120, body: 0.04, ring: 380, click: 1600, clickGain: 0.4 }), 0.004, 0.8);
  return normalize(out, 0.85);
}

/** Kontra (or Re): a flat hand slapped onto the table. */
function kontra() {
  reseed(0x6c0a7a1);
  const out = buffer(0.34);
  const slap = bandpass(burst(0.1, 0.018, 0.0005), 1300, 0.5);
  mix(out, slap, 0, 1);
  mix(out, bandpass(burst(0.05, 0.006, 0.0003), 3800, 0.8), 0, 0.5);
  mix(out, woodHit(0.3, { low: 65, drop: 70, body: 0.06, ring: 330, click: 1200, clickGain: 0.2 }), 0.002, 0.7);
  return normalize(out, 0.9);
}

/** One snip of the scissors: a short metallic click with a hiss. */
function snipOnce() {
  const data = buffer(0.08);
  mix(data, highpass(burst(0.05, 0.006, 0.0003), 3000, 0.8), 0, 0.6);
  mix(data, partials(0.08, [[4700, 0.4, 0.01], [6900, 0.25, 0.007], [3100, 0.2, 0.012]], 0.0003), 0, 1);
  return data;
}

/** Schneider: the scissors go across the felt, snip snap. */
function snip() {
  reseed(0x5c155e5);
  const out = buffer(0.36);
  mix(out, snipOnce(), 0, 0.9);
  mix(out, slide(0.12, 2500, 6000, 0.35), 0.04, 1);
  mix(out, snipOnce(), 0.17, 1);
  return normalize(out, 0.7);
}

/** Schwarz: the cloth torn right through, a long rip and a thump. */
function schwarz() {
  reseed(0x5c4a12a);
  const out = buffer(0.82);
  const rip = buffer(0.6);
  let crackle = 0;
  for (let i = 0; i < rip.length; i++) {
    const t = i / RATE;
    if (i % 140 === 0) crackle = Math.abs(noise()) ** 2;
    rip[i] = noise() * (0.3 + 0.7 * crackle) * Math.min(1, t / 0.03) * (t > 0.45 ? Math.exp(-(t - 0.45) / 0.05) : 1);
  }
  bandpass(rip, (t) => 700 + t * 4200, 0.9);
  mix(out, rip, 0, 1);
  mix(out, woodHit(0.3, { low: 55, drop: 80, body: 0.07, ring: 290, click: 1200, clickGain: 0.3 }), 0.52, 0.9);
  return normalize(out, 0.85);
}

/** A bell of the counter: inharmonic partials of a small brass bell. */
function bellTone(freq, seconds, gain = 1, decay = 0.5) {
  return partials(seconds, [
    [freq, 0.55 * gain, decay],
    [freq * 2.76, 0.28 * gain, decay * 0.55],
    [freq * 5.4, 0.14 * gain, decay * 0.3],
    [freq * 8.93, 0.06 * gain, decay * 0.16],
    [freq * 0.5, 0.08 * gain, decay * 1.2],
  ]);
}

/** An unlock: the bell on the counter, ding ding, rising. */
function unlock() {
  reseed(0x0711ab1);
  const out = buffer(0.95);
  mix(out, bellTone(NOTE(84), 0.8, 1, 0.32), 0, 0.9);
  mix(out, bellTone(NOTE(91), 0.8, 1, 0.38), 0.15, 1);
  mix(out, bandpass(burst(0.02, 0.002), 6000, 1), 0, 0.3);
  mix(out, bandpass(burst(0.02, 0.002), 6500, 1), 0.15, 0.3);
  return normalize(out, 0.65);
}

/** Bought at the Tresen: coins slid over the counter, then the till's bell. */
function till() {
  reseed(0x7111b11);
  const out = buffer(0.62);
  const hit = (f) => partials(0.15, [[f, 0.5, 0.05], [f * 1.5, 0.35, 0.035], [f * 2.01, 0.2, 0.025]]);
  mix(out, slide(0.12, 2500, 4500, 0.3), 0, 1);
  mix(out, hit(3150), 0.02, 0.6);
  mix(out, hit(3480), 0.07, 0.45);
  mix(out, bellTone(NOTE(88), 0.5, 1, 0.22), 0.13, 0.85);
  return normalize(out, 0.62);
}

/** Dice rattled in a leather cup and thrown onto the wood. */
function dice() {
  reseed(0xd1ce5ee);
  const out = buffer(0.62);
  const click = (f, g) => {
    const data = partials(0.04, [[f, g, 0.006], [f * 1.7, g * 0.5, 0.004]], 0.0003);
    mix(data, bandpass(burst(0.02, 0.002, 0.0002), f * 1.3, 1), 0, g * 0.5);
    return data;
  };
  // In the cup: dull rattles.
  for (let k = 0; k < 9; k++) mix(out, lowpass(click(between(900, 1500), 0.5), 2500), k * 0.022 + between(0, 0.008), 0.5);
  // On the table: brighter hits, further apart, dying away.
  let t = 0.24;
  for (let k = 0; k < 10; k++) {
    mix(out, click(between(1700, 2900), 1 - k * 0.08), t, 1);
    t += 0.018 + k * 0.009 + between(0, 0.01);
  }
  return normalize(out, 0.7);
}

/** A Schnaps poured: a short gurgle rising as the little glass fills. */
function pour() {
  reseed(0x90a1a1a);
  const out = buffer(0.5);
  let t = 0.01;
  while (t < 0.4) {
    const p = t / 0.4;
    const len = between(0.012, 0.028);
    const f0 = 380 + 520 * p + between(-60, 60);
    const bubble = buffer(len);
    let phase = 0;
    for (let i = 0; i < bubble.length; i++) {
      const s = i / RATE;
      phase += (f0 * (1 + 0.9 * (s / len))) / RATE;
      bubble[i] = Math.sin(2 * Math.PI * phase) * Math.sin(Math.PI * (s / len));
    }
    mix(out, bubble, t, between(0.4, 1) * Math.sin(Math.PI * Math.min(1, p * 1.1 + 0.05)));
    t += between(0.012, 0.03);
  }
  // The stream: a soft, high hiss.
  const stream = buffer(0.42);
  for (let i = 0; i < stream.length; i++) {
    const s = i / RATE;
    stream[i] = noise() * Math.sin(Math.PI * Math.min(1, s / 0.42)) ** 0.5;
  }
  mix(out, bandpass(stream, (s) => 2200 + s * 2000, 1.5), 0, 0.12);
  return normalize(lowpass(out, 7000), 0.55);
}

/** Down in one: a gulp, then the empty glass set down on the wood. */
function gulp() {
  reseed(0x6017905);
  const out = buffer(0.46);
  const gl = buffer(0.12);
  let phase = 0;
  for (let i = 0; i < gl.length; i++) {
    const t = i / RATE;
    const f = 120 + 230 * Math.exp(-t / 0.025);
    phase += f / RATE;
    gl[i] = (Math.sin(2 * Math.PI * phase) + 0.3 * Math.sin(4 * Math.PI * phase)) * Math.min(1, t / 0.004) * Math.exp(-t / 0.035);
  }
  mix(out, lowpass(gl, 1400), 0, 1);
  // The glass on the table: a tock with a little ring of glass.
  mix(out, woodHit(0.14, { low: 160, drop: 120, body: 0.015, ring: 900, click: 2600, clickGain: 0.5 }), 0.27, 0.6);
  mix(out, partials(0.15, [[2900, 0.2, 0.04], [4400, 0.1, 0.025]]), 0.27, 1);
  return normalize(out, 0.7);
}

/** Something gained (a relic, a Schnaps): a beer mat put down, a small sparkle. */
function prize() {
  reseed(0x6a1a6a1);
  const out = buffer(0.52);
  mix(out, lowpass(burst(0.08, 0.01, 0.0008), 1800), 0, 0.7);
  mix(out, partials(0.1, [[240, 0.5, 0.02]]), 0, 1);
  mix(out, bellTone(NOTE(86), 0.4, 0.8, 0.16), 0.06, 0.5);
  mix(out, bellTone(NOTE(93), 0.4, 0.8, 0.2), 0.13, 0.45);
  return normalize(out, 0.6);
}

/** A boss walks in: the pub door creaks open and falls shut. */
function boss() {
  reseed(0xb055d00);
  const out = buffer(1.3);
  // The hinge: stick-slip pulses whose rate wavers.
  const creak = buffer(0.85);
  let next = 0;
  for (let i = 0; i < creak.length; i++) {
    const t = i / RATE;
    if (t >= next) {
      const env = Math.sin(Math.PI * Math.min(1, t / 0.85)) ** 0.6;
      creak[i] = env * (0.7 + 0.3 * Math.abs(noise()));
      const rate = 34 + 26 * Math.sin(2 * Math.PI * 1.4 * t) + 18 * t + between(-4, 4);
      next = t + 1 / rate;
    }
  }
  const a = bandpass(Float32Array.from(creak), 620, 6);
  const b = bandpass(Float32Array.from(creak), 1450, 7);
  for (let i = 0; i < creak.length; i++) creak[i] = a[i] + 0.6 * b[i];
  mix(out, creak, 0, 1);
  // The door falls shut: a deep, wide thump.
  mix(out, woodHit(0.4, { low: 45, drop: 60, body: 0.11, ring: 210, click: 900, clickGain: 0.5 }), 0.92, 1.4);
  return normalize(lowpass(out, 6000), 0.9);
}

/** A way chosen on the map: two quick strokes of chalk. */
function chalk() {
  reseed(0xc4a1c00);
  const out = buffer(0.34);
  mix(out, chalkStroke(0.12, 1), 0, 1);
  mix(out, chalkStroke(0.16, 0.8), 0.15, 1);
  return normalize(out, 0.5);
}

/** The long night begins: the church clock strikes, low and far away. */
function night() {
  reseed(0x4e16470);
  const out = buffer(1.7);
  const strike = (f) =>
    partials(1.6, [
      [f * 0.5, 0.35, 1.1],
      [f, 0.5, 0.8],
      [f * 1.19, 0.3, 0.5],
      [f * 1.5, 0.2, 0.42],
      [f * 2.0, 0.22, 0.35],
      [f * 2.52, 0.1, 0.22],
      [f * 3.01, 0.06, 0.15],
    ], 0.003);
  mix(out, strike(NOTE(57)), 0, 1);
  mix(out, lowpass(burst(0.04, 0.006), 1500), 0, 0.3);
  return normalize(lowpass(out, 4000), 0.7);
}

/** A Stammgast's quip: a soft hummed "hm", rising a little like a question. */
function quip() {
  reseed(0x9e1b0ff);
  const seconds = 0.3;
  const out = buffer(seconds);
  let phase = 0;
  for (let i = 0; i < out.length; i++) {
    const t = i / RATE;
    const p = t / seconds;
    // "Hm": up a little, then down at the end.
    const f = 118 * (1 + 0.12 * Math.sin(Math.PI * Math.min(1, p * 1.3)) - 0.05 * p);
    phase += f / RATE;
    const ph = phase % 1;
    // Glottal pulse: open part as a raised sine, closed part silent.
    const pulse = ph < 0.45 ? Math.sin((Math.PI * ph) / 0.45) ** 2 : 0;
    const env = Math.min(1, t / 0.045) * (t > seconds - 0.11 ? Math.max(0, (seconds - t) / 0.11) : 1);
    out[i] = (pulse - 0.22) * env + noise() * 0.015 * env;
  }
  // Mouth closed: the nasal murmur, little above a few hundred Hz.
  const nasal = bandpass(Float32Array.from(out), 260, 2.5);
  lowpass(lowpass(out, 700), 900);
  for (let i = 0; i < out.length; i++) out[i] = out[i] * 0.6 + nasal[i] * 0.8;
  highpass(out, 80, 0.7);
  return normalize(out, 0.6);
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

const sounds = {
  card, trick, tick, clink, coin, knock, win, lose,
  tap, shuffle, pickup, discard, declare, bang, slam, kontra, snip, schwarz,
  unlock, till, dice, pour, gulp, prize, boss, chalk, night, quip,
};
// The two long ones are encoded a little thinner (low, soft sounds, no loss heard).
const LONG = new Set(['boss', 'night']);
// `node scripts/make-sounds.mjs tap quip` writes only those (the others stay as shipped).
const only = new Set(process.argv.slice(2));
for (const name of only) if (!(name in sounds)) throw new Error(`no sound "${name}"`);
mkdirSync(out, { recursive: true });
const tmp = join(tmpdir(), `stammtisch-sounds-${process.pid}`);
mkdirSync(tmp, { recursive: true });
try {
  for (const [name, make] of Object.entries(sounds)) {
    if (only.size && !only.has(name)) {
      // Still made, so the shared noise sequence of the eight first sounds stays the same.
      make();
      continue;
    }
    const file = join(tmp, `${name}.wav`);
    writeFileSync(file, wav(make()));
    execFileSync('ffmpeg', ['-y', '-loglevel', 'error', '-i', file, '-ac', '1', '-codec:a', 'libmp3lame', '-b:a', LONG.has(name) ? '48k' : '64k', '-map_metadata', '-1', join(out, `${name}.mp3`)]);
    console.log(`wrote ${name}.mp3`);
  }
} finally {
  rmSync(tmp, { recursive: true, force: true });
}

// The three music loops of the web app (DESIGN 16, "Musik"): Tisch, Tresen
// and Boss. Composed for this project and synthesized in the browser by
// ./music.ts, so no audio file and no third-party licence is involved.
//
// Notation (one token per grid step, `div` steps per quarter beat):
// - lead: a note name with octave ("A4", "C#5", "Bb4"), "-" holds the note
//   before, "." is a rest.
// - bass, harmony: chord tones of the chord sounding at that step, counted
//   from the root ("1" root, "2" third, "3" fifth, "4" the next tone or the
//   root an octave up); "'" raises by an octave, "," lowers by one; "c"
//   plays the whole chord at once; "-" and "." as above.
// - drums (always sixteenths): "k" kick, "s" snare, "x" closed hi-hat,
//   "o" open hi-hat, "." nothing.
// - chords: one string per bar; several chords in a bar share it evenly.
// Patterns of bass, harmony and drums repeat bar by bar.

export type MusicId = 'table' | 'shop' | 'boss';

export type Wave = 'square' | 'pulse25' | 'pulse12' | 'triangle';

export interface Instrument {
  wave: Wave;
  /** Peak gain of one note. */
  gain: number;
  attack: number;
  /** Time constant of the decay to the sustain level, in seconds. */
  decay: number;
  /** Sustain level as a fraction of the peak. */
  sustain: number;
  /** Time constant of the release, in seconds. */
  release: number;
  /** Fraction of the written length that sounds before the release. */
  legato: number;
  /** Delayed vibrato on longer notes. */
  vibrato?: { rate: number; cents: number; delay: number };
  /** Two voices detuned by ± this many cents (an accordion's beating). */
  chorus?: number;
}

export interface Part {
  inst: Instrument;
  /** Steps per quarter beat. */
  div: number;
  /** Lowest MIDI note a chord root may take in this part (bass, harmony). */
  base?: number;
  bars: string[];
}

export interface DrumKit {
  /** Overall level of the kit. */
  gain: number;
  bars: string[];
}

export interface Song {
  id: MusicId;
  bpm: number;
  /** Quarter beats per bar. */
  beats: number;
  /** Low-pass on the whole loop, Hz: keeps the square waves soft. */
  cutoff: number;
  /** Level of the loop relative to the others. */
  level: number;
  chords: string[];
  lead: Part;
  bass: Part;
  harmony: Part;
  drums: DrumKit;
}

// ------------------------------------------------------------------ Tisch
// D minor, 96 BPM, 4/4, 24 bars (A B A'). A quiet pub late at night: a
// square lead with a slow vibrato, a soft triangle bass on root and fifth,
// a music-box arpeggio and a hi-hat on the off-beats.

const tableA = [
  '. A4 D5 - C5 A4 - -',
  '. G4 C5 - Bb4 G4 - -',
  '. F4 Bb4 - A4 F4 G4 A4',
  'E5 - D5 C#5 - - . .',
  '. A4 D5 - E5 F5 - -',
  'G5 - F5 - E5 C5 - -',
];

const TABLE: Song = {
  id: 'table',
  bpm: 96,
  beats: 4,
  cutoff: 2200,
  level: 0.5,
  chords: [
    // A
    'Dm', 'C', 'Bb', 'A', 'Dm', 'F', 'Gm', 'A',
    // B
    'Bb', 'F', 'C', 'Dm', 'Bb', 'F', 'Gm', 'A7',
    // A'
    'Dm', 'C', 'Bb', 'A', 'Dm', 'F', 'Gm A', 'Dm',
  ],
  lead: {
    inst: { wave: 'square', gain: 0.2, attack: 0.012, decay: 0.3, sustain: 0.6, release: 0.07, legato: 0.92, vibrato: { rate: 4.8, cents: 14, delay: 0.25 } },
    div: 2,
    bars: [
      ...tableA,
      '. Bb4 D5 - C5 Bb4 A4 G4',
      'A4 - - - - - . .',
      'F5 - - - D5 - Bb4 -',
      'C5 - - - - - A4 C5',
      'E5 - D5 - C5 - G4 -',
      'A4 - - - - - . .',
      'F5 - - - D5 - Bb4 D5',
      'C5 - - - A4 - C5 F5',
      'E5 - D5 - Bb4 - G4 -',
      'A4 - C#5 - E5 - . .',
      ...tableA,
      '. Bb4 D5 - C#5 - E5 -',
      'D5 - - - - - . .',
    ],
  },
  bass: {
    inst: { wave: 'triangle', gain: 0.5, attack: 0.01, decay: 0.4, sustain: 0.7, release: 0.09, legato: 0.95 },
    div: 2,
    base: 45,
    bars: ['1 - - - 3, - - -', '1 - - - 3, - 2, -'],
  },
  harmony: {
    inst: { wave: 'triangle', gain: 0.13, attack: 0.006, decay: 0.15, sustain: 0.35, release: 0.1, legato: 0.8 },
    div: 2,
    base: 57,
    bars: ['1 2 3 4 3 2 3 2'],
  },
  drums: {
    gain: 0.7,
    bars: ['..x...x...x...x.', '..x...x...x...xx'],
  },
};

// ----------------------------------------------------------------- Tresen
// G major, 112 BPM, 3/4, 24 bars (A B A'). Warmer and a little jaunty, a
// waltz at the bar: the lead is a pulse wave doubled and detuned like an
// accordion's reeds, the bass alternates root and fifth on the one, the
// chord answers on two and three.

const shopA = [
  'D4 G4 B4 - D5 -',
  'G5 - - - D5 B4',
  'C5 - E5 - G5 -',
  'B4 - D5 - - -',
  'C5 - - B4 A4 C5',
  'D5 - - C5 B4 A4',
];

const SHOP: Song = {
  id: 'shop',
  bpm: 112,
  beats: 3,
  cutoff: 3000,
  level: 1,
  chords: [
    // A
    'G', 'G', 'C', 'G', 'Am', 'D', 'D7', 'G',
    // B
    'C', 'C', 'G', 'G', 'A7', 'A7', 'D', 'D7',
    // A'
    'G', 'G', 'C', 'G', 'Am', 'D7', 'D7', 'G',
  ],
  lead: {
    inst: { wave: 'pulse25', gain: 0.2, attack: 0.01, decay: 0.2, sustain: 0.7, release: 0.06, legato: 0.85, chorus: 7, vibrato: { rate: 5.5, cents: 8, delay: 0.3 } },
    div: 2,
    bars: [
      ...shopA,
      'F#4 - A4 - C5 -',
      'B4 - G4 - . .',
      'E5 - - G5 - E5',
      'C5 - - - C5 D5',
      'B4 - D5 - G5 -',
      'D5 - - - E5 F#5',
      'G5 - E5 - C#5 -',
      'E5 - - - . A4',
      'D5 - F#5 - A5 -',
      'F#5 - - - C5 A4',
      ...shopA,
      'C5 - A4 - F#4 A4',
      'G4 - - - . .',
    ],
  },
  bass: {
    inst: { wave: 'triangle', gain: 0.55, attack: 0.005, decay: 0.2, sustain: 0.5, release: 0.06, legato: 0.8 },
    div: 2,
    base: 48,
    bars: ['1 - . . . .', '3, - . . . .'],
  },
  harmony: {
    inst: { wave: 'pulse12', gain: 0.075, attack: 0.003, decay: 0.08, sustain: 0.4, release: 0.04, legato: 0.55 },
    div: 2,
    base: 55,
    bars: ['. . c - c -'],
  },
  drums: {
    gain: 0.6,
    bars: ['k...x...x...', 'k...x...x.x.'],
  },
};

// ------------------------------------------------------------------- Boss
// E minor, 120 BPM, 4/4, 16 bars. Tense: a staccato square lead, the
// Phrygian F against E minor, a pulsing bass in eighths with octave jumps,
// a fast sixteenth arpeggio and kick, snare and hi-hat.

const BOSS: Song = {
  id: 'boss',
  bpm: 120,
  beats: 4,
  cutoff: 3400,
  level: 1,
  chords: [
    'Em', 'Em', 'F', 'Em', 'Em', 'Em', 'C', 'B',
    'Am', 'Am', 'Em', 'Em', 'F', 'F', 'B', 'B7',
  ],
  lead: {
    inst: { wave: 'square', gain: 0.18, attack: 0.005, decay: 0.15, sustain: 0.55, release: 0.05, legato: 0.8, vibrato: { rate: 6, cents: 10, delay: 0.2 } },
    div: 2,
    bars: [
      'E5 . E5 . G5 - F#5 -',
      'E5 - - - B4 - - -',
      'F5 . F5 . A5 - G5 -',
      'E5 - - - - - . .',
      'B4 . B4 . E5 - G5 -',
      'F#5 - E5 - D5 - B4 -',
      'C5 - E5 - G5 - E5 -',
      'D#5 - - - F#5 - - -',
      'A5 - - - G5 - E5 -',
      'C5 - - - E5 - A4 -',
      'B4 - - - E5 - G5 -',
      'B5 - - - - - . .',
      'A5 - G5 - F5 - C5 -',
      'F5 - - - A4 - C5 -',
      'D#5 - - - F#5 - B5 -',
      "A5 - - - F#5 - D#5 -",
    ],
  },
  bass: {
    inst: { wave: 'pulse25', gain: 0.2, attack: 0.003, decay: 0.1, sustain: 0.6, release: 0.03, legato: 0.6 },
    div: 2,
    base: 40,
    bars: ["1 1 1' 1 1 1 1' 1"],
  },
  harmony: {
    inst: { wave: 'pulse12', gain: 0.045, attack: 0.002, decay: 0.05, sustain: 0.3, release: 0.02, legato: 0.5 },
    div: 4,
    base: 57,
    bars: ['1 2 3 2 4 2 3 2 1 2 3 2 4 3 2 3'],
  },
  drums: {
    gain: 0.8,
    bars: ['k.x.s.x.k.x.s.x.', 'k.x.s.x.k.k.s.xo'],
  },
};

export const SONGS: Record<MusicId, Song> = { table: TABLE, shop: SHOP, boss: BOSS };

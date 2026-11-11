// Player settings of the web app (presentation only): animation tempo,
// sound effects, music, whether a thought bubble shows while the opponents think, and
// whether the Wirt explains things the first time they happen, and whether
// the cards' paper takes a light tint of their suit, how the hand is
// sorted and whether the phone vibrates; and the display aids
// (Barrierefreiheit): the suit letters of the "Farbenblind-Hilfe" and the
// text size ("Schriftgröße").
// Stored in localStorage, every access guarded.

import { readJson, writeKey } from './storage';

export type Tempo = 'slow' | 'normal' | 'fast';

export interface Settings {
  tempo: Tempo;
  /** Sound effects. */
  sound: boolean;
  /** Music loops (./music.ts), switched apart from the sound effects. */
  music: boolean;
  /** Show a thought bubble over the opponents while the core computes their moves. */
  thinking: boolean;
  /** Hints of the Wirt ("Geführter erster Abend", ./hints.ts). */
  hints: boolean;
  /** "Kartenfarbe tönen": each card's paper lightly tinted in its suit (CSS, ./style.css). */
  suitTint: boolean;
  /** Card back of the player's edition chosen among the earned ones (ADR 0020); null: the character's own. */
  back: string | null;
  /** "Hand sortieren": the core's order for the game declared, or the player's own (drag to reorder). */
  handSort: HandSort;
  /** Vibration on the player's tricks and on results where the device has it (./haptics.ts). */
  vibrate: boolean;
  /** "Farbenblind-Hilfe": suit letters on the cards and shapes beside colour-only marks (CSS, ./style.css). */
  colorBlind: boolean;
  /** "Schriftgröße": the type scale of the whole app (CSS variables, ./style.css). */
  textSize: TextSize;
}

export type HandSort = 'auto' | 'manual';

export type TextSize = 'normal' | 'large' | 'xlarge';

/** The text sizes in the order the menu toggle steps through them. */
export const TEXT_SIZES: readonly TextSize[] = ['normal', 'large', 'xlarge'];

const KEY = 'stammtisch.settings';
const DEFAULTS: Settings = { tempo: 'normal', sound: true, music: true, thinking: true, hints: true, suitTint: false, back: null, handSort: 'auto', vibrate: true, colorBlind: false, textSize: 'normal' };

/** Factor applied to every animation duration and pause. */
export const TEMPO_FACTOR: Record<Tempo, number> = {
  slow: 1.5,
  normal: 1,
  fast: 0.5,
};

export function loadSettings(): Settings {
  const stored = readJson(KEY);
  if (!stored || typeof stored !== 'object') return { ...DEFAULTS };
  const parsed = stored as Partial<Settings>;
  const tempo = parsed.tempo === 'slow' || parsed.tempo === 'fast' ? parsed.tempo : 'normal';
  return {
    tempo,
    sound: parsed.sound !== false,
    music: parsed.music !== false,
    thinking: parsed.thinking !== false,
    hints: parsed.hints !== false,
    suitTint: parsed.suitTint === true,
    back: typeof parsed.back === 'string' ? parsed.back : null,
    handSort: parsed.handSort === 'manual' ? 'manual' : 'auto',
    vibrate: parsed.vibrate !== false,
    colorBlind: parsed.colorBlind === true,
    textSize: TEXT_SIZES.includes(parsed.textSize as TextSize) ? (parsed.textSize as TextSize) : 'normal',
  };
}

/** Storage blocked: the settings last for this visit only. */
export function saveSettings(settings: Settings): void {
  writeKey(KEY, JSON.stringify(settings));
}

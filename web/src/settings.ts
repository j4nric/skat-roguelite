// Player settings of the web app (presentation only): animation tempo,
// sound effects, music, whether a thought bubble shows while the opponents think, and
// whether the Wirt explains things the first time they happen, and whether
// the cards' paper takes a light tint of their suit.
// Stored in localStorage, every access guarded.

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
}

const KEY = 'stammtisch.settings';
const DEFAULTS: Settings = { tempo: 'normal', sound: true, music: true, thinking: true, hints: true, suitTint: false, back: null };

/** Factor applied to every animation duration and pause. */
export const TEMPO_FACTOR: Record<Tempo, number> = {
  slow: 1.5,
  normal: 1,
  fast: 0.5,
};

export function loadSettings(): Settings {
  try {
    const text = window.localStorage.getItem(KEY);
    if (!text) return { ...DEFAULTS };
    const parsed = JSON.parse(text) as Partial<Settings>;
    const tempo = parsed.tempo === 'slow' || parsed.tempo === 'fast' ? parsed.tempo : 'normal';
    return {
      tempo,
      sound: parsed.sound !== false,
      music: parsed.music !== false,
      thinking: parsed.thinking !== false,
      hints: parsed.hints !== false,
      suitTint: parsed.suitTint === true,
      back: typeof parsed.back === 'string' ? parsed.back : null,
    };
  } catch {
    return { ...DEFAULTS };
  }
}

export function saveSettings(settings: Settings): void {
  try {
    window.localStorage.setItem(KEY, JSON.stringify(settings));
  } catch {
    // Storage blocked: settings last for this visit only.
  }
}

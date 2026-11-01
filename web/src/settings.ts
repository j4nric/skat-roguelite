// Player settings of the web app (presentation only): animation tempo,
// sound, and whether a thought bubble shows while the opponents think.
// Stored in localStorage, every access guarded.

export type Tempo = 'slow' | 'normal' | 'fast';

export interface Settings {
  tempo: Tempo;
  sound: boolean;
  /** Show a thought bubble over the opponents while the core computes their moves. */
  thinking: boolean;
}

const KEY = 'stammtisch.settings';
const DEFAULTS: Settings = { tempo: 'normal', sound: true, thinking: true };

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
    return { tempo, sound: parsed.sound !== false, thinking: parsed.thinking !== false };
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

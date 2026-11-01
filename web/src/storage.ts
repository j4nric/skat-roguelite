// Browser storage: the running run and the meta state across runs (unlocks
// and statistics), both as serialized core state, kept apart so that a
// broken run save never costs the unlocks. Every access is guarded; a full
// or blocked storage must never break the game.

const RUN_KEY = 'stammtisch.run';
const META_KEY = 'stammtisch.meta';
const CHARACTER_KEY = 'stammtisch.character';

function read(key: string): string | null {
  try {
    return window.localStorage.getItem(key);
  } catch {
    return null;
  }
}

function write(key: string, value: string): void {
  try {
    window.localStorage.setItem(key, value);
  } catch {
    // Storage full or blocked: the game goes on without saving.
  }
}

function remove(key: string): void {
  try {
    window.localStorage.removeItem(key);
  } catch {
    // Ignore.
  }
}

export function loadRun(): string | null {
  return read(RUN_KEY);
}

export function saveRun(state: string): void {
  write(RUN_KEY, state);
}

export function clearRun(): void {
  remove(RUN_KEY);
}

export function loadMeta(): string | null {
  return read(META_KEY);
}

export function saveMeta(state: string): void {
  write(META_KEY, state);
}

/** Keep a meta save the core no longer reads, instead of overwriting it. */
export function backupMeta(state: string): void {
  write(`${META_KEY}.backup`, state);
}

/** The character picked last on the start screen (presentation only). */
export function loadCharacter(): string | null {
  return read(CHARACTER_KEY);
}

export function saveCharacter(character: string): void {
  write(CHARACTER_KEY, character);
}

// ---------------------------------------------------------------------------
// Recent runs (presentation only). The core's meta state keeps totals but no
// list of runs, so the app remembers the last finished runs itself: who
// played, the seed, how far it went, the best game, the date.
// ---------------------------------------------------------------------------

const HISTORY_KEY = 'stammtisch.history';
const RUN_INFO_KEY = 'stammtisch.runinfo';

/** Runs kept in the list. */
export const HISTORY_SIZE = 20;

export interface RunRecord {
  character: 'Declarer' | 'NullPlayer';
  seed: string;
  /** Section reached, from 1. */
  section: number;
  /** Series reached in that section, from 1. */
  series: number;
  won: boolean;
  /** Highest Spielwert of a won game in the run, from the core. */
  bestGame: number;
  /** ISO time the run ended. */
  date: string;
  /** Started as the daily seed. */
  daily: boolean;
}

function isRecord(value: unknown): value is RunRecord {
  if (typeof value !== 'object' || value === null) return false;
  const r = value as Record<string, unknown>;
  return (
    (r.character === 'Declarer' || r.character === 'NullPlayer') &&
    typeof r.seed === 'string' &&
    typeof r.section === 'number' &&
    typeof r.series === 'number' &&
    typeof r.won === 'boolean' &&
    typeof r.bestGame === 'number' &&
    typeof r.date === 'string' &&
    typeof r.daily === 'boolean'
  );
}

/** The recent runs, newest first. Broken entries are skipped. */
export function loadHistory(): RunRecord[] {
  const raw = read(HISTORY_KEY);
  if (!raw) return [];
  try {
    const parsed: unknown = JSON.parse(raw);
    return Array.isArray(parsed) ? parsed.filter(isRecord).slice(0, HISTORY_SIZE) : [];
  } catch {
    return [];
  }
}

/** Put a finished run on top of the list; returns the new list. */
export function addToHistory(record: RunRecord): RunRecord[] {
  const list = [record, ...loadHistory()].slice(0, HISTORY_SIZE);
  write(HISTORY_KEY, JSON.stringify(list));
  return list;
}

/** How the running run was started (the core does not know about daily seeds). */
export interface RunInfo {
  seed: string;
  daily: boolean;
}

export function loadRunInfo(): RunInfo | null {
  const raw = read(RUN_INFO_KEY);
  if (!raw) return null;
  try {
    const parsed = JSON.parse(raw) as Partial<RunInfo>;
    return typeof parsed.seed === 'string' ? { seed: parsed.seed, daily: parsed.daily === true } : null;
  } catch {
    return null;
  }
}

export function saveRunInfo(info: RunInfo): void {
  write(RUN_INFO_KEY, JSON.stringify(info));
}

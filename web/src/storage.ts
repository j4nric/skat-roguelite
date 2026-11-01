// Browser storage: the running run and the meta state across runs (unlocks
// and statistics), both as serialized core state, kept apart so that a
// broken run save never costs the unlocks. Every access is guarded; a full
// or blocked storage must never break the game.

const RUN_KEY = 'stammtisch.run';
const META_KEY = 'stammtisch.meta';
const CHARACTER_KEY = 'stammtisch.character';
const LENGTH_KEY = 'stammtisch.length';

function read(key: string): string | null {
  try {
    return window.localStorage.getItem(key);
  } catch {
    return null;
  }
}

function write(key: string, value: string): boolean {
  try {
    window.localStorage.setItem(key, value);
    return true;
  } catch {
    // Storage full or blocked: the game goes on without saving.
    return false;
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
  if (write(RUN_KEY, state)) askPersistence();
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

/** The run length picked last on the start screen (presentation only). */
export function loadLength(): string | null {
  return read(LENGTH_KEY);
}

export function saveLength(length: string): void {
  write(LENGTH_KEY, length);
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
  /** Kurzer Abend; absent (entries from before) means ganzer Abend. */
  short?: boolean;
  /** Stake of the Einsatz-Leiter; absent means 0. */
  stake?: number;
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
    typeof r.daily === 'boolean' &&
    (r.short === undefined || typeof r.short === 'boolean') &&
    (r.stake === undefined || typeof r.stake === 'number')
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

// ---------------------------------------------------------------------------
// Keeping the data: ask the browser once to keep this site's storage
// (otherwise Safari may clear it after a while without visits), and the
// save code (./savecode.ts) that carries every key of ours to another
// browser or address.
// ---------------------------------------------------------------------------

/** Every key of the app starts with this. */
const PREFIX = 'stammtisch.';

let persistenceAsked = false;

/**
 * Ask for persistent storage, once per visit, after the first real save
 * (a run started or played on), never on page load. Failures and a "no"
 * change nothing: the game keeps saving as before.
 */
export function askPersistence(): void {
  if (persistenceAsked) return;
  persistenceAsked = true;
  try {
    const storage = navigator.storage as StorageManager | undefined;
    if (!storage || typeof storage.persist !== 'function') return;
    const already = typeof storage.persisted === 'function' ? storage.persisted() : Promise.resolve(false);
    already
      .then((kept) => (kept ? true : storage.persist()))
      .then(
        (kept) => {
          if (import.meta.env.DEV) console.info(`persistent storage: ${kept ? 'granted' : 'not granted'}`);
        },
        () => undefined,
      );
  } catch {
    // Not available here.
  }
}

/** All stored keys of the app with their values. */
export function readAllKeys(): Record<string, string> {
  const keys: Record<string, string> = {};
  try {
    const store = window.localStorage;
    for (let i = 0; i < store.length; i++) {
      const key = store.key(i);
      if (key === null || !key.startsWith(PREFIX)) continue;
      const value = store.getItem(key);
      if (value !== null) keys[key] = value;
    }
  } catch {
    // Storage blocked: nothing to read.
  }
  return keys;
}

/**
 * Replace every stored key of the app by the given ones (keys of other
 * apps are left alone, foreign keys in `keys` are skipped). Returns false
 * if the storage refused.
 */
export function replaceAllKeys(keys: Record<string, string>): boolean {
  try {
    const store = window.localStorage;
    const old: string[] = [];
    for (let i = 0; i < store.length; i++) {
      const key = store.key(i);
      if (key !== null && key.startsWith(PREFIX)) old.push(key);
    }
    // Write first, then drop what the code does not have: a full storage
    // fails before anything of the current progress is removed.
    for (const [key, value] of Object.entries(keys)) {
      if (key.startsWith(PREFIX)) store.setItem(key, value);
    }
    for (const key of old) {
      if (!(key in keys)) store.removeItem(key);
    }
    askPersistence();
    return true;
  } catch {
    return false;
  }
}

/** The stored run and meta state inside a set of keys, for checking. */
export function runAndMetaOf(keys: Record<string, string>): { run: string | null; meta: string | null } {
  return { run: keys[RUN_KEY] ?? null, meta: keys[META_KEY] ?? null };
}

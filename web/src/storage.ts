// Browser storage: the running run and the meta state across runs (unlocks
// and statistics), both as serialized core state, kept apart so that a
// broken run save never costs the unlocks. Every access is guarded; a full
// or blocked storage must never break the game.

import { CHARACTERS, type Character } from './types';

const RUN_KEY = 'stammtisch.run';
const META_KEY = 'stammtisch.meta';
const CHARACTER_KEY = 'stammtisch.character';

/**
 * Read a key; null if it is missing or the storage is blocked. For the
 * app's own small records too (settings, hints, sync bookkeeping).
 */
export function readKey(key: string): string | null {
  try {
    return window.localStorage.getItem(key);
  } catch {
    return null;
  }
}

/** Write a key; false if the storage is full or blocked (the game goes on without saving). */
export function writeKey(key: string, value: string): boolean {
  try {
    window.localStorage.setItem(key, value);
  } catch {
    return false;
  }
  changed(key);
  return true;
}

export function removeKey(key: string): void {
  try {
    window.localStorage.removeItem(key);
  } catch {
    return;
  }
  changed(key);
}

/** Whether the storage can be used at all (some embedded browsers block it). */
export function storageUsable(): boolean {
  try {
    return Boolean(window.localStorage);
  } catch {
    return false;
  }
}

/** Read and parse a JSON key; null if missing, blocked or broken. */
export function readJson(key: string): unknown {
  const raw = readKey(key);
  if (raw === null) return null;
  try {
    return JSON.parse(raw) as unknown;
  } catch {
    return null;
  }
}

const watchers: ((key: string | null) => void)[] = [];

/**
 * Be told after any write through this module, with the key (null: all
 * keys of the app). The sync prepares its upload with it (./sync.ts).
 */
export function watchStorage(fn: (key: string | null) => void): void {
  watchers.push(fn);
}

function changed(key: string | null): void {
  for (const fn of watchers) {
    try {
      fn(key);
    } catch (err) {
      console.warn('storage watcher failed', err);
    }
  }
}

export function loadRun(): string | null {
  return readKey(RUN_KEY);
}

export function saveRun(state: string): void {
  if (writeKey(RUN_KEY, state)) askPersistence();
}

export function clearRun(): void {
  removeKey(RUN_KEY);
}

export function loadMeta(): string | null {
  return readKey(META_KEY);
}

export function saveMeta(state: string): void {
  writeKey(META_KEY, state);
}

/** Keep a meta save the core no longer reads, instead of overwriting it. */
export function backupMeta(state: string): void {
  writeKey(`${META_KEY}.backup`, state);
}

/** The character picked last on the start screen (presentation only). */
export function loadCharacter(): string | null {
  return readKey(CHARACTER_KEY);
}

export function saveCharacter(character: string): void {
  writeKey(CHARACTER_KEY, character);
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
  character: Character;
  seed: string;
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
  /** Started as the run of this week (JJJJWW, DESIGN 5.6); absent otherwise. */
  weekly?: number;
}

function isRecord(value: unknown): value is RunRecord {
  if (typeof value !== 'object' || value === null) return false;
  const r = value as Record<string, unknown>;
  return (
    CHARACTERS.includes(r.character as Character) &&
    typeof r.seed === 'string' &&
    typeof r.section === 'number' &&
    typeof r.series === 'number' &&
    typeof r.won === 'boolean' &&
    typeof r.bestGame === 'number' &&
    typeof r.date === 'string' &&
    typeof r.daily === 'boolean' &&
    (r.stake === undefined || typeof r.stake === 'number') &&
    (r.weekly === undefined || typeof r.weekly === 'number')
  );
}

/** The recent runs, newest first. Broken entries are skipped. */
export function loadHistory(): RunRecord[] {
  const raw = readKey(HISTORY_KEY);
  if (!raw) return [];
  try {
    const parsed: unknown = JSON.parse(raw);
    // Entries of a short evening (`short`, dropped with ADR 0016) read as the whole evening.
    return Array.isArray(parsed) ? parsed.filter(isRecord).slice(0, HISTORY_SIZE).map(({ short: _short, ...r }: RunRecord & { short?: unknown }) => r) : [];
  } catch {
    return [];
  }
}

/** Put a finished run on top of the list; returns the new list. */
export function addToHistory(record: RunRecord): RunRecord[] {
  const list = [record, ...loadHistory()].slice(0, HISTORY_SIZE);
  writeKey(HISTORY_KEY, JSON.stringify(list));
  return list;
}

/** How the running run was started (the core does not know about daily seeds). */
export interface RunInfo {
  seed: string;
  daily: boolean;
}

export function loadRunInfo(): RunInfo | null {
  const raw = readKey(RUN_INFO_KEY);
  if (!raw) return null;
  try {
    const parsed = JSON.parse(raw) as Partial<RunInfo>;
    return typeof parsed.seed === 'string' ? { seed: parsed.seed, daily: parsed.daily === true } : null;
  } catch {
    return null;
  }
}

export function saveRunInfo(info: RunInfo): void {
  writeKey(RUN_INFO_KEY, JSON.stringify(info));
}

// ---------------------------------------------------------------------------
// Keeping the data: ask the browser once to keep this site's storage
// (otherwise Safari may clear it after a while without visits), and the
// save code (./savecode.ts) that carries every key of ours to another
// browser or address.
// ---------------------------------------------------------------------------

/** Every key of the app starts with this. */
const PREFIX = 'stammtisch.';

/** Whether a key is part of the save (save code, upload). */
export function isSaveKey(key: string): boolean {
  return key.startsWith(PREFIX);
}

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
 * apps are left alone, foreign keys in `keys` are skipped). All or
 * nothing: if the storage refuses a write (full), every key gets its old
 * value back and false is returned.
 */
export function replaceAllKeys(keys: Record<string, string>): boolean {
  let store: Storage;
  const before = readAllKeys();
  try {
    store = window.localStorage;
  } catch {
    return false;
  }
  const restore = (): void => {
    for (const key of Object.keys(readAllKeys())) {
      if (!(key in before)) store.removeItem(key);
    }
    for (const [key, value] of Object.entries(before)) {
      try {
        store.setItem(key, value);
      } catch {
        // Was there before, so it fitted; nothing more to do.
      }
    }
  };
  try {
    // Drop what the new state does not have first: that frees room for it.
    for (const key of Object.keys(before)) {
      if (!(key in keys)) store.removeItem(key);
    }
    for (const [key, value] of Object.entries(keys)) {
      if (key.startsWith(PREFIX)) store.setItem(key, value);
    }
  } catch {
    try {
      restore();
    } catch {
      // Blocked midway: nothing more to do.
    }
    changed(null);
    return false;
  }
  changed(null);
  askPersistence();
  return true;
}

/** The stored run and meta state inside a set of keys, for checking. */
export function runAndMetaOf(keys: Record<string, string>): { run: string | null; meta: string | null } {
  return { run: keys[RUN_KEY] ?? null, meta: keys[META_KEY] ?? null };
}

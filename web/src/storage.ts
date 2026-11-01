// Browser storage: the running run (serialized core state), and the record
// across runs. Every access is guarded; a full or blocked storage must never
// break the game.

const RUN_KEY = 'stammtisch.run';
const RECORD_KEY = 'stammtisch.record';

export interface Record {
  runs: number;
  won: number;
  /** Furthest section reached, from 1. */
  bestSection: number;
}

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

export function loadRecord(): Record {
  const empty: Record = { runs: 0, won: 0, bestSection: 0 };
  const text = read(RECORD_KEY);
  if (!text) return empty;
  try {
    const parsed = JSON.parse(text) as Partial<Record>;
    return {
      runs: Number(parsed.runs) || 0,
      won: Number(parsed.won) || 0,
      bestSection: Number(parsed.bestSection) || 0,
    };
  } catch {
    return empty;
  }
}

/** Count a finished run. */
export function recordRun(won: boolean, section: number): Record {
  const record = loadRecord();
  const next: Record = {
    runs: record.runs + 1,
    won: record.won + (won ? 1 : 0),
    bestSection: Math.max(record.bestSection, section),
  };
  write(RECORD_KEY, JSON.stringify(next));
  return next;
}

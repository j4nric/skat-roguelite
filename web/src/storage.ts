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

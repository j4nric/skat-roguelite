// Keeping the stored progress in step with the account (ADR 0021).
//
// What goes up is exactly what the save code carries (every
// `stammtisch.` key, ./storage.ts readAllKeys), encoded as a save code
// (./savecode.ts); the server keeps it as an opaque blob with a version.
// No game logic here: whether the core can read a state is asked of the
// app (`SyncHost.readable`), which asks the core.
//
// When:
// - at start, before the app reads its storage: fetch the server's state
//   and compare,
// - after each pub won or lost, at the end of a run, when the page is
//   hidden: upload if anything changed since the last sync.
//
// Which state wins:
// - Only one side changed since the last sync: that one (it is newer).
//   The server's state replaces the local one only at start or right
//   after signing in, with a visible note.
// - Both changed (or this device was never synced and has progress of
//   its own): the player is asked; nothing is overwritten until then.
// - A newer local state never silently replaces a newer server state:
//   every upload names the version it builds on, the server refuses
//   otherwise (409) and the player is asked.
//
// The bookkeeping (`stammtisch-sync`, no dot) lies outside the save keys,
// so it is never part of a save code or an upload.

import { AccountError, fetchSave, putSave, type AccountUser, type ServerSave } from './account';
import { decodeSave, encodeSave } from './savecode';
import { readAllKeys, replaceAllKeys } from './storage';

const RECORD_KEY = 'stammtisch-sync';

/** Uploads on page hide go with `keepalive`, which allows small bodies only. */
const KEEPALIVE_MAX = 60_000;

export type SyncStatus = 'off' | 'saving' | 'saved' | 'offline' | 'conflict' | 'outdated';

/** Both states, waiting for the player to pick one. */
export interface SyncConflict {
  server: ServerSave;
  serverKeys: Record<string, string>;
  localKeys: Record<string, string>;
  /** When the local state last changed (as far as this device noticed). */
  localAt: number;
}

interface SyncRecord {
  signedIn: boolean;
  /** Server version of the last sync, null if never synced. */
  version: number | null;
  /** Hash of the keys at the last sync. */
  hash: string | null;
  /** When the local state was last seen changed (ms). */
  localAt: number;
  /** Name as last seen, to show before the server answers. */
  name: string | null;
}

/** What the app provides. */
export interface SyncHost {
  /** Whether the core can load the run and meta state among these keys. */
  readable(keys: Record<string, string>): boolean;
  /** Something about the account or its status changed: render. */
  changed(): void;
}

function readRecord(): SyncRecord | null {
  try {
    const raw = window.localStorage.getItem(RECORD_KEY);
    if (!raw) return null;
    const r = JSON.parse(raw) as Partial<SyncRecord>;
    if (r.signedIn !== true) return null;
    return {
      signedIn: true,
      version: typeof r.version === 'number' ? r.version : null,
      hash: typeof r.hash === 'string' ? r.hash : null,
      localAt: typeof r.localAt === 'number' ? r.localAt : 0,
      name: typeof r.name === 'string' ? r.name : null,
    };
  } catch {
    return null;
  }
}

function writeRecord(r: SyncRecord | null): void {
  try {
    if (r) window.localStorage.setItem(RECORD_KEY, JSON.stringify(r));
    else window.localStorage.removeItem(RECORD_KEY);
  } catch {
    // Storage blocked: the account works for this visit only.
  }
}

/** A fast 53-bit hash of the stored keys (order does not matter). */
export function hashKeys(keys: Record<string, string>): string {
  const text = JSON.stringify(Object.keys(keys).sort().map((k) => [k, keys[k]]));
  let h1 = 0xdeadbeef;
  let h2 = 0x41c6ce57;
  for (let i = 0; i < text.length; i++) {
    const c = text.charCodeAt(i);
    h1 = Math.imul(h1 ^ c, 2654435761);
    h2 = Math.imul(h2 ^ c, 1597334677);
  }
  h1 = Math.imul(h1 ^ (h1 >>> 16), 2246822507) ^ Math.imul(h2 ^ (h2 >>> 13), 3266489909);
  h2 = Math.imul(h2 ^ (h2 >>> 16), 2246822507) ^ Math.imul(h1 ^ (h1 >>> 13), 3266489909);
  return (4294967296 * (2097151 & h2) + (h1 >>> 0)).toString(36);
}

/** No progress worth keeping: no run in progress and no finished run. */
export function isEmptyProgress(keys: Record<string, string>): boolean {
  if (keys['stammtisch.run']) return false;
  const history = keys['stammtisch.history'];
  if (!history) return true;
  try {
    const list: unknown = JSON.parse(history);
    return !Array.isArray(list) || list.length === 0;
  } catch {
    return true;
  }
}

export type Reconciled = 'applied' | 'none' | 'conflict';

export class Sync {
  user: AccountUser | null = null;
  status: SyncStatus = 'off';
  conflict: SyncConflict | null = null;
  /** When the last upload or download succeeded (ms). */
  lastSync = 0;
  private record: SyncRecord | null = readRecord();
  private running: Promise<void> | null = null;
  private again = false;

  constructor(private readonly host: SyncHost) {
    if (this.record) {
      // Shown until the server answers.
      this.user = { name: this.record.name, passkeys: 0, createdAt: 0 };
      this.status = 'saved';
    }
    window.addEventListener('online', () => void this.push());
  }

  /** Whether this device thinks it is signed in. */
  get signedIn(): boolean {
    return this.record !== null;
  }

  private setStatus(status: SyncStatus): void {
    this.status = status;
    this.host.changed();
  }

  private remember(change: Partial<SyncRecord>): void {
    if (!this.record) return;
    this.record = { ...this.record, ...change };
    writeRecord(this.record);
  }

  /** Signed in (sign-up, passkey, recovery code): start keeping track. */
  signedInAs(user: AccountUser): void {
    this.user = user;
    if (!this.record) {
      this.record = { signedIn: true, version: null, hash: null, localAt: Date.now(), name: user.name };
    }
    this.remember({ name: user.name });
    this.host.changed();
  }

  /** The name changed. */
  updateUser(user: AccountUser): void {
    this.user = user;
    this.remember({ name: user.name });
    this.host.changed();
  }

  /** Signed out or account gone: forget the account on this device (the progress stays). */
  forget(): void {
    this.record = null;
    writeRecord(null);
    this.user = null;
    this.conflict = null;
    this.status = 'off';
    this.host.changed();
  }

  /**
   * Compare with the server and settle what can be settled alone. Called
   * at start (before the app reads its storage) and right after signing
   * in. `applied`: the server's state was written to storage.
   */
  async reconcile(timeout?: number): Promise<Reconciled> {
    if (!this.record) return 'none';
    this.setStatus('saving');
    let server: ServerSave | null;
    try {
      server = await fetchSave(timeout);
    } catch (err) {
      this.failed(err);
      return 'none';
    }
    const record = this.record;
    const local = readAllKeys();
    const hash = hashKeys(local);
    const dirty = hash !== record.hash;
    if (dirty) this.remember({ localAt: Date.now() });
    if (!server) {
      if (!isEmptyProgress(local)) await this.upload(local, hash, null);
      else this.setStatus('saved');
      return 'none';
    }
    if (server.version === record.version) {
      if (dirty) await this.upload(local, hash, server.version);
      else this.done();
      return 'none';
    }
    // Another device saved since.
    const decoded = await decodeSave(server.data).catch(() => null);
    if (!decoded?.ok || !this.host.readable(decoded.keys)) {
      // From a newer version of the game (or broken): never overwrite it from here.
      this.setStatus('outdated');
      return 'none';
    }
    const serverHash = hashKeys(decoded.keys);
    if (serverHash === hash) {
      this.remember({ version: server.version, hash });
      this.done();
      return 'none';
    }
    if (!dirty || isEmptyProgress(local)) {
      if (!replaceAllKeys(decoded.keys)) {
        this.setStatus('offline');
        return 'none';
      }
      this.remember({ version: server.version, hash: serverHash, localAt: server.updatedAt });
      this.done();
      return 'applied';
    }
    this.conflict = { server, serverKeys: decoded.keys, localKeys: local, localAt: this.record?.localAt ?? Date.now() };
    this.setStatus('conflict');
    return 'conflict';
  }

  private done(): void {
    this.lastSync = Date.now();
    this.setStatus('saved');
  }

  private failed(err: unknown): void {
    const code = err instanceof AccountError ? err.code : 'offline';
    if (code === 'unauthorized') {
      // The session ended (deleted elsewhere, expired): signed out here too.
      this.forget();
      return;
    }
    this.setStatus('offline');
  }

  private async upload(keys: Record<string, string>, hash: string, ifVersion: number | null, keepalive = false): Promise<void> {
    this.setStatus('saving');
    try {
      const data = await encodeSave(keys);
      const res = await putSave(data, this.record?.localAt ?? Date.now(), ifVersion, keepalive && data.length < KEEPALIVE_MAX);
      this.remember({ version: res.version, hash });
      this.done();
    } catch (err) {
      if (err instanceof AccountError && err.code === 'conflict') {
        // Someone was faster: compare again; with changes here it asks the player.
        await this.reconcile();
        return;
      }
      this.failed(err);
    }
  }

  /**
   * Upload if anything changed (after a pub, at run end, on page hide).
   * Resolves when this and any upload asked for meanwhile are done.
   */
  // Only what localStorage holds goes up: the Einführung (DESIGN 16.1) is
  // never saved there, so it is never uploaded.
  push(keepalive = false): Promise<void> {
    if (!this.record || this.status === 'conflict' || this.status === 'outdated') return Promise.resolve();
    if (this.running) {
      this.again = true;
      return this.running;
    }
    this.running = (async () => {
      try {
        do {
          this.again = false;
          const record = this.record;
          if (!record) return;
          const keys = readAllKeys();
          const hash = hashKeys(keys);
          if (hash === record.hash) {
            if (this.status !== 'saved') this.done();
            continue;
          }
          this.remember({ localAt: Date.now() });
          await this.upload(keys, hash, record.version, keepalive);
        } while (this.again && this.record && this.status !== 'conflict');
      } finally {
        this.running = null;
      }
    })();
    return this.running;
  }

  /** Conflict: keep this device's state and put it on the server. */
  async keepLocal(): Promise<void> {
    const c = this.conflict;
    if (!c) return;
    this.conflict = null;
    const keys = readAllKeys();
    await this.upload(keys, hashKeys(keys), c.server.version);
  }

  /** Conflict: take the server's state into storage (the app reloads then). */
  takeServer(): boolean {
    const c = this.conflict;
    if (!c || !replaceAllKeys(c.serverKeys)) return false;
    this.conflict = null;
    this.remember({ version: c.server.version, hash: hashKeys(c.serverKeys), localAt: c.server.updatedAt });
    this.status = 'saved';
    return true;
  }
}

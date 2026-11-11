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
//   hidden: upload if anything changed since the last sync,
// - offline, nothing is tried; when the browser is back online or the app
//   returns to the foreground, whatever changed meanwhile goes up.
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
//
// Order: every exchange with the server (compare, upload, keep this
// device's state) runs one after the other through one queue, so an upload
// never builds on a version a compare running at the same time is about
// to change. Storage is only replaced at start (before the app has read
// it) or right before the page reloads; while the app runs, a newer state
// on the server is a conflict for the player to settle.
//
// Leaving the page: the encoded upload is prepared whenever the stored
// keys change, so `pagehide` can send it at once (an async encoding there
// may never get to its request).

import { AccountError, fetchSave, putSave, type AccountUser, type ServerSave } from './account';
import { hashText } from './hash';
import { isOnline } from './offline';
import { decodeSave, encodeSave } from './savecode';
import { isSaveKey, readAllKeys, readJson, removeKey, replaceAllKeys, watchStorage, writeKey } from './storage';

const RECORD_KEY = 'stammtisch-sync';

/** Uploads on page hide go with `keepalive`, which allows small bodies only. */
const KEEPALIVE_MAX = 60_000;

/** Wait after a change of the stored keys before encoding the upload. */
const PREPARE_MS = 400;

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
  readable(keys: Record<string, string>): Promise<boolean>;
  /** Whether a stored meta state holds no progress yet (asked of the core). */
  freshMeta(meta: string): Promise<boolean>;
  /** Something about the account or its status changed: render. */
  changed(): void;
}

function readRecord(): SyncRecord | null {
  const r = readJson(RECORD_KEY) as Partial<SyncRecord> | null;
  if (!r || typeof r !== 'object' || r.signedIn !== true) return null;
  return {
    signedIn: true,
    version: typeof r.version === 'number' ? r.version : null,
    hash: typeof r.hash === 'string' ? r.hash : null,
    localAt: typeof r.localAt === 'number' ? r.localAt : 0,
    name: typeof r.name === 'string' ? r.name : null,
  };
}

/** Storage blocked: the account works for this visit only. */
function writeRecord(r: SyncRecord | null): void {
  if (r) writeKey(RECORD_KEY, JSON.stringify(r));
  else removeKey(RECORD_KEY);
}

/** A fast 53-bit hash of the stored keys (order does not matter). */
export function hashKeys(keys: Record<string, string>): string {
  return hashText(JSON.stringify(Object.keys(keys).sort().map((k) => [k, keys[k]])));
}

/**
 * No progress worth keeping: no run in progress, no finished run and a
 * meta state without progress (no unlocks, no statistics; `freshMeta`
 * asks the core).
 */
export async function isEmptyProgress(keys: Record<string, string>, freshMeta: (meta: string) => Promise<boolean>): Promise<boolean> {
  if (keys['stammtisch.run']) return false;
  const meta = keys['stammtisch.meta'];
  if (meta && !(await freshMeta(meta))) return false;
  const history = keys['stammtisch.history'];
  if (!history) return true;
  try {
    const list: unknown = JSON.parse(history);
    return !Array.isArray(list) || list.length === 0;
  } catch {
    return true;
  }
}

/**
 * `applied`: the server's state is in storage (at start) or waits to be
 * taken right before the page reloads (after signing in, `takeServer`).
 */
export type Reconciled = 'applied' | 'none' | 'conflict';

/** When a compare runs: at start, right after signing in, or mid-session (an upload was refused). */
type When = 'start' | 'signin' | 'session';

export class Sync {
  user: AccountUser | null = null;
  status: SyncStatus = 'off';
  conflict: SyncConflict | null = null;
  /** When the last upload or download succeeded (ms). */
  lastSync = 0;
  private record: SyncRecord | null = readRecord();
  /** Every exchange with the server, one after the other. */
  private queue: Promise<unknown> = Promise.resolve();
  /** A push waiting in the queue that has not started yet (later pushes join it). */
  private queuedPush: Promise<void> | null = null;
  private queuedKeepalive = false;
  /** The server's state found right after signing in, taken by `takeServer` before the reload. */
  private incoming: { server: ServerSave; keys: Record<string, string> } | null = null;
  /** The upload encoded in advance for leaving the page. */
  private prepared: { hash: string; data: string } | null = null;
  private prepareTimer: number | undefined;

  constructor(private readonly host: SyncHost) {
    if (this.record) {
      // Shown until the server answers.
      this.user = { name: this.record.name, passkeys: 0, createdAt: 0 };
      this.status = 'saved';
    }
    // Offline (DESIGN 14.5): say so at once; what changed meanwhile goes up
    // when the connection is back or the app returns to the foreground.
    window.addEventListener('offline', () => {
      if (this.record && (this.status === 'saved' || this.status === 'saving')) this.setStatus('offline');
    });
    window.addEventListener('online', () => void this.push());
    document.addEventListener('visibilitychange', () => {
      if (document.visibilityState === 'visible' && this.status === 'offline') void this.push();
    });
    watchStorage((key) => {
      if (key === null || isSaveKey(key)) this.schedulePrepare();
    });
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

  /** Run `fn` after everything queued before it. */
  private serial<T>(fn: () => Promise<T>): Promise<T> {
    const run = this.queue.then(fn, fn);
    this.queue = run.catch(() => undefined);
    return run;
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
    this.incoming = null;
    this.prepared = null;
    this.status = 'off';
    this.host.changed();
  }

  /**
   * Compare with the server and settle what can be settled alone. At
   * start (before the app reads its storage) the server's
   * state may be written to storage (`applied`). Right after signing in it
   * is only kept for `takeServer`, which the app calls right before it
   * reloads.
   */
  reconcileAtStart(timeout: number): Promise<Reconciled> {
    return this.serial(() => this.compare('start', timeout));
  }

  /** The same right after signing in; `applied` waits for `takeServer`. */
  reconcileAfterSignIn(): Promise<Reconciled> {
    return this.serial(() => this.compare('signin'));
  }

  private async compare(when: When, timeout?: number): Promise<Reconciled> {
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
    if (!record) return 'none';
    const local = readAllKeys();
    const hash = hashKeys(local);
    const dirty = hash !== record.hash;
    if (dirty) this.remember({ localAt: Date.now() });
    if (!server) {
      if (!(await isEmptyProgress(local, (m) => this.host.freshMeta(m)))) await this.upload(local, hash, null);
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
    if (!decoded?.ok || !(await this.host.readable(decoded.keys))) {
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
    // Mid-session the app runs on what it read from storage: never replace it under it.
    const take = when !== 'session' && (!dirty || (await isEmptyProgress(local, (m) => this.host.freshMeta(m))));
    if (take && when === 'signin') {
      this.incoming = { server, keys: decoded.keys };
      this.done();
      return 'applied';
    }
    if (take) {
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

  /** The encoded keys: prepared in advance if they have not changed since, else encoded now. */
  private async encoded(keys: Record<string, string>, hash: string): Promise<string> {
    const ready = this.prepared;
    if (ready && ready.hash === hash) return ready.data;
    const data = await encodeSave(keys);
    this.prepared = { hash, data };
    return data;
  }

  /** Inside the queue only. */
  private async upload(keys: Record<string, string>, hash: string, ifVersion: number | null, keepalive = false): Promise<void> {
    this.setStatus('saving');
    try {
      const data = await this.encoded(keys, hash);
      const res = await putSave(data, this.record?.localAt ?? Date.now(), ifVersion, keepalive && data.length < KEEPALIVE_MAX);
      this.remember({ version: res.version, hash });
      this.done();
    } catch (err) {
      if (err instanceof AccountError && err.code === 'conflict') {
        // Someone was faster: compare again. The app runs on its storage,
        // so a newer server state is put to the player, never written.
        await this.compare('session');
        return;
      }
      this.failed(err);
    }
  }

  /**
   * Upload if anything changed (after a pub, at run end, on page hide).
   * Resolves when this upload is done; pushes asked for before it started
   * share it.
   */
  // Only what localStorage holds goes up: the Einführung (DESIGN 16.1) is
  // never saved there, so it is never uploaded.
  push(keepalive = false): Promise<void> {
    if (!this.record || this.status === 'conflict' || this.status === 'outdated') return Promise.resolve();
    if (keepalive) this.queuedKeepalive = true;
    this.queuedPush ??= this.serial(async () => {
      this.queuedPush = null;
      const ka = this.queuedKeepalive;
      this.queuedKeepalive = false;
      await this.pushNow(ka);
    });
    return this.queuedPush;
  }

  private async pushNow(keepalive: boolean): Promise<void> {
    const record = this.record;
    if (!record || this.status === 'conflict' || this.status === 'outdated') return;
    if (!isOnline()) {
      // No point trying; the `online` event brings us back.
      this.setStatus('offline');
      return;
    }
    const keys = readAllKeys();
    const hash = hashKeys(keys);
    if (hash === record.hash) {
      if (this.status !== 'saved') this.done();
      return;
    }
    this.remember({ localAt: Date.now() });
    await this.upload(keys, hash, record.version, keepalive);
  }

  /** Encode the upload soon after the stored keys changed (for `sendOnHide`). */
  private schedulePrepare(): void {
    if (!this.record) return;
    window.clearTimeout(this.prepareTimer);
    this.prepareTimer = window.setTimeout(() => {
      const record = this.record;
      if (!record || this.status === 'conflict' || this.status === 'outdated') return;
      const keys = readAllKeys();
      const hash = hashKeys(keys);
      if (hash === record.hash || this.prepared?.hash === hash) return;
      encodeSave(keys).then(
        (data) => {
          this.prepared = { hash, data };
        },
        () => undefined,
      );
    }, PREPARE_MS);
  }

  /**
   * The page goes away (`pagehide`): send what changed with the request
   * started right now, from the upload prepared in advance. If the keys
   * changed after it was prepared, or it is too big for `keepalive`, the
   * normal push tries (and the next start compares anyway).
   */
  sendOnHide(): void {
    const record = this.record;
    if (!record || this.status === 'conflict' || this.status === 'outdated' || !isOnline()) return;
    const keys = readAllKeys();
    const hash = hashKeys(keys);
    if (hash === record.hash) return;
    const ready = this.prepared;
    if (!ready || ready.hash !== hash || ready.data.length >= KEEPALIVE_MAX) {
      void this.push(true);
      return;
    }
    this.remember({ localAt: Date.now() });
    // Started synchronously: putSave calls fetch before its first await.
    const sent = putSave(ready.data, record.localAt, record.version, true);
    sent.then(
      (res) => {
        // Only if nothing else synced meanwhile.
        if (this.record && this.record.version === record.version) {
          this.remember({ version: res.version, hash });
          this.done();
        }
      },
      () => undefined,
    );
  }

  /** Conflict: keep this device's state and put it on the server. */
  keepLocal(): Promise<void> {
    return this.serial(async () => {
      const c = this.conflict;
      if (!c) return;
      this.conflict = null;
      const keys = readAllKeys();
      await this.upload(keys, hashKeys(keys), c.server.version);
    });
  }

  /**
   * Take the server's state into storage: the one of a conflict, or the
   * one found right after signing in. The app reloads right after; false
   * if the storage refused (nothing changed then).
   */
  takeServer(): boolean {
    const from = this.conflict ? { server: this.conflict.server, keys: this.conflict.serverKeys } : this.incoming;
    if (!from || !replaceAllKeys(from.keys)) return false;
    this.conflict = null;
    this.incoming = null;
    this.remember({ version: from.server.version, hash: hashKeys(from.keys), localAt: from.server.updatedAt });
    this.status = 'saved';
    return true;
  }
}

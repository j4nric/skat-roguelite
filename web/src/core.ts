// The only place that talks to the WebAssembly core. Every game fact the
// app shows comes out of here as JSON produced by skat_core. Views pass
// through `localize` (./i18n): the content's German display texts become
// those of the active language (ADR 0023).
//
// The core runs in a dedicated worker (ADR 0022, ./core.worker.ts), so the
// opponents' search never blocks the page. Every call is a request with an
// id; the worker answers one request at a time, in the order they were
// sent, so the answers come back in that order too. Where no worker can
// run the core (no Worker, a module worker refused, WebAssembly blocked in
// it), the same engine (./core-engine.ts) runs on the main thread behind
// the same promises.
//
// The main thread keeps what the core said last: the view of each run and
// meta state and their save texts. So `view()` and `save()` stay
// synchronous, and storing on `pagehide` needs no answer from the worker.

import type { CoreOp, MetaState, RunState, StepState } from './core-engine';
import wasmUrl from './wasm/pkg/skat_wasm_bg.wasm?url';
import { localize } from './i18n';
import type { Catalog, Character, MetaStep, MetaView, RunAction, RunEvent, RunLength, RunView, Step, WeekInfo } from './types';

/** Where the core runs. */
interface Backend {
  /** Send a request; resolves with the core's answer, rejects with its error. */
  call<T>(op: CoreOp, args: unknown[], transfer?: Transferable[]): Promise<T>;
  /** The core runs in a worker, not on the main thread. */
  readonly offMainThread: boolean;
}

/** A core error: its JSON text as the message (../app/consts.ts errorCode reads it). */
function coreError(message: string): Error {
  return new Error(message);
}

/** The core in a dedicated worker; requests and answers paired by id. */
class WorkerBackend implements Backend {
  readonly offMainThread = true;
  private nextId = 1;
  private readonly pending = new Map<number, { resolve: (v: unknown) => void; reject: (e: Error) => void }>();
  private broken: Error | null = null;

  private constructor(private readonly worker: Worker) {
    worker.onmessage = (e: MessageEvent<{ id: number; value?: unknown; error?: string }>) => {
      const p = this.pending.get(e.data.id);
      if (!p) return;
      this.pending.delete(e.data.id);
      if (e.data.error !== undefined) p.reject(coreError(e.data.error));
      else p.resolve(e.data.value);
    };
    worker.onerror = (e) => {
      e.preventDefault();
      this.fail(new Error(`core worker failed: ${e.message || 'script error'}`));
    };
    worker.onmessageerror = () => this.fail(new Error('core worker sent an unreadable message'));
  }

  /** Start the worker and the core in it; rejects if either cannot run. */
  static async start(bytes: Promise<ArrayBuffer>): Promise<WorkerBackend> {
    const worker = new Worker(new URL('./core.worker.ts', import.meta.url), { type: 'module', name: 'core' });
    const backend = new WorkerBackend(worker);
    try {
      const buffer = await bytes;
      await backend.call('init', [buffer], [buffer]);
    } catch (err) {
      backend.fail(err instanceof Error ? err : new Error(String(err)));
      throw err;
    }
    return backend;
  }

  call<T>(op: CoreOp, args: unknown[], transfer: Transferable[] = []): Promise<T> {
    if (this.broken) return Promise.reject(this.broken);
    const id = this.nextId++;
    return new Promise<T>((resolve, reject) => {
      this.pending.set(id, { resolve: resolve as (v: unknown) => void, reject });
      this.worker.postMessage({ id, op, args }, transfer);
    });
  }

  /** The worker is gone: every waiting request fails, and every later one. */
  private fail(err: Error): void {
    if (this.broken) return;
    this.broken = err;
    for (const p of this.pending.values()) p.reject(err);
    this.pending.clear();
    this.worker.terminate();
  }
}

/** The core on the main thread (no worker): one request after the other, like the worker. */
class LocalBackend implements Backend {
  readonly offMainThread = false;
  private queue: Promise<unknown> = Promise.resolve();

  private constructor(private readonly engine: typeof import('./core-engine')) {}

  static async start(): Promise<LocalBackend> {
    const engine = await import('./core-engine');
    await engine.initCore(wasmUrl);
    return new LocalBackend(engine);
  }

  call<T>(op: CoreOp, args: unknown[]): Promise<T> {
    const result = this.queue.then(() => {
      try {
        return this.engine.handle(op, args);
      } catch (err) {
        throw coreError(this.engine.errorMessage(err));
      }
    });
    this.queue = result.catch(() => undefined);
    return result as Promise<T>;
  }
}

/** The core's bytes (the page preloads them, web/sprite/plugin.ts). */
async function fetchCore(): Promise<ArrayBuffer> {
  const res = await fetch(wasmUrl);
  if (!res.ok) throw new Error(`core: HTTP ${res.status}`);
  return res.arrayBuffer();
}

/** Whether the core runs off the main thread; false until it is loaded. */
let offMain = false;

async function startBackend(): Promise<Backend> {
  if (typeof Worker === 'function') {
    try {
      // The worker's script loads while the bytes come in.
      const worker = await WorkerBackend.start(fetchCore());
      offMain = true;
      return worker;
    } catch (err) {
      console.warn('core worker unavailable, the core runs on the main thread', err);
    }
  }
  return LocalBackend.start();
}

let backend: Promise<Backend> | null = null;

/** Send a request to the core, after every one sent before it. */
function call<T>(op: CoreOp, args: unknown[]): Promise<T> {
  backend ??= startBackend();
  return backend.then((b) => b.call<T>(op, args));
}

/** Load and instantiate the core once (in its worker if it can). */
export function loadCore(): Promise<void> {
  backend ??= startBackend();
  return backend.then(() => undefined);
}

/** The core runs in its worker: the main thread need not paint before asking it something slow. */
export function coreOffMainThread(): boolean {
  return offMain;
}

/** Names and texts of all content. */
export async function loadCatalog(): Promise<Catalog> {
  return JSON.parse(await call<string>('catalog', [])) as Catalog;
}

/**
 * The Wochen-Herausforderung of the ISO week of a local date (DESIGN 5.6):
 * its number, seed and Sonderregel, from the core. Null if the core has none.
 */
export async function weekInfo(date: Date = new Date()): Promise<WeekInfo | null> {
  try {
    return JSON.parse(await call<string>('weekInfo', [date.getFullYear(), date.getMonth() + 1, date.getDate()])) as WeekInfo;
  } catch {
    return null;
  }
}

/** Whether a stored meta state is still the one of a new player; rejects if the core cannot load it. */
export function isFreshMetaState(meta: string): Promise<boolean> {
  return call<boolean>('isFreshMeta', [meta]);
}

/** The views of a stored run and meta state (either may be null); rejects if the core cannot load one. */
export async function describeStored(run: string | null, meta: string | null): Promise<{ run: RunView | null; meta: MetaView | null }> {
  const out = await call<{ run: string | null; meta: string | null }>('describe', [run, meta]);
  return { run: out.run ? localize(JSON.parse(out.run) as RunView) : null, meta: out.meta ? localize(JSON.parse(out.meta) as MetaView) : null };
}

/** The error code in a core error's JSON text, or null. */
function codeOf(message: string): string | null {
  try {
    return (JSON.parse(message) as { code?: string }).code ?? null;
  } catch {
    return null;
  }
}

const disposedError = (): Error => coreError(JSON.stringify({ code: 'Internal', message: 'disposed' }));

/** The meta state across runs: unlocks and statistics (DESIGN 9.3). */
export class CoreMeta {
  private disposed = false;

  private constructor(
    /** The meta state's handle in the core. */
    readonly handle: number,
    private lastView: MetaView,
    private lastSave: string,
  ) {}

  private static from(s: MetaState): CoreMeta {
    return new CoreMeta(s.handle, localize(JSON.parse(s.view) as MetaView), s.save);
  }

  /** A new player: nothing unlocked. */
  static async fresh(): Promise<CoreMeta> {
    return CoreMeta.from(await call<MetaState>('metaFresh', []));
  }

  /** Load a saved meta state; rejects if the save does not fit. */
  static async load(state: string): Promise<CoreMeta> {
    return CoreMeta.from(await call<MetaState>('metaLoad', [state]));
  }

  /** The meta state as text for localStorage, as the core last gave it. */
  save(): string {
    return this.lastSave;
  }

  /** The meta view as the core last gave it. */
  view(): MetaView {
    return localize(this.lastView);
  }

  /** Let the meta state learn from the events of a step. */
  async observe(events: RunEvent[] | string): Promise<MetaStep> {
    if (this.disposed) throw disposedError();
    const json = typeof events === 'string' ? events : JSON.stringify(events);
    const s = await call<StepState>('metaObserve', [this.handle, json]);
    const step = localize(JSON.parse(s.step) as MetaStep);
    this.lastView = step.view;
    this.lastSave = s.save;
    return step;
  }

  dispose(): void {
    if (this.disposed) return;
    this.disposed = true;
    void call('metaFree', [this.handle]).catch(() => undefined);
  }
}

/** A run in the core. */
export class CoreRun {
  private disposed = false;

  private constructor(
    private readonly handle: number,
    private lastView: RunView,
    private lastSave: string,
  ) {}

  private static from(s: RunState): CoreRun {
    return new CoreRun(s.handle, localize(JSON.parse(s.view) as RunView), s.save);
  }

  /** Start a new run of the Alleinspieler with every relic unlocked. */
  static async create(seed: string): Promise<CoreRun> {
    return CoreRun.from(await call<RunState>('runCreate', [seed]));
  }

  /**
   * Start a new run with a character, length and stake (Einsatz-Leiter) from the
   * meta state. Returns the run and the events of its start as JSON (feed them to
   * the meta state). A stake above the unlocked one rejects with `StakeLocked`.
   */
  static async createWith(seed: string, character: Character, length: RunLength, stake: number, meta: CoreMeta): Promise<{ run: CoreRun; startEvents: string }> {
    const s = await call<RunState & { startEvents: string }>('runCreateWith', [seed, character, length, stake, meta.handle]);
    return { run: CoreRun.from(s), startEvents: s.startEvents };
  }

  /**
   * Start the run of a week (JJJJWW, DESIGN 5.6) with a character and stake
   * from the meta state: the week's seed and Sonderregel. A rule with its
   * own character plays that one. Returns the run and its start events.
   */
  static async createWeekly(week: number, character: Character, stake: number, meta: CoreMeta): Promise<{ run: CoreRun; startEvents: string }> {
    const s = await call<RunState & { startEvents: string }>('runCreateWeekly', [week, character, stake, meta.handle]);
    return { run: CoreRun.from(s), startEvents: s.startEvents };
  }

  /**
   * Start the Einführung (DESIGN 16.1): one guided pub with the fixed deals
   * of the core's `tutorial.ron`. Never feed its events to the meta state
   * and never save it: it is no run.
   */
  static async tutorial(): Promise<CoreRun> {
    return CoreRun.from(await call<RunState>('runTutorial', []));
  }

  /** Load a saved run; rejects if the save does not fit. */
  static async load(state: string): Promise<CoreRun> {
    return CoreRun.from(await call<RunState>('runLoad', [state]));
  }

  /** The run as text for localStorage, after the last step the core answered. */
  save(): string {
    return this.lastSave;
  }

  /** The view after the last step the core answered. */
  view(): RunView {
    return localize(this.lastView);
  }

  /**
   * Apply a player decision. The opponents do not play here: while the
   * returned view's `ai_to_move` is set, call `step()`. Rejects if the core
   * rejects it.
   */
  async apply(action: RunAction): Promise<Step> {
    return this.took(await this.call<StepState>('runApply', [this.handle, JSON.stringify(action)]));
  }

  /**
   * What an action would do, without doing it: the core applies it to a
   * copy of the run and returns that copy's events and view. Rejects with
   * the core's error (`CardCursed`, `ModAtMaximum`, …) like `apply`.
   * Read-only; an action that draws at random draws on the copy, so its
   * outcome must not be shown.
   */
  async preview(action: RunAction): Promise<Step> {
    return localize(JSON.parse(await this.call<string>('runPreview', [this.handle, JSON.stringify(action)])) as Step);
  }

  /**
   * Whether the core would accept each action now: null if so, else its
   * error code (`CardCursed`, `ModAtMaximum`, …, as `apply` would reject
   * it). Builds no view and no events, unlike `preview`: for asking about
   * many actions at once (the dimmed cards of a picker), in one request.
   */
  async check(actions: RunAction[]): Promise<(string | null)[]> {
    const out = await this.call<({ code: string | null } | { error: string })[]>('runCheck', [this.handle, actions.map((a) => JSON.stringify(a))]);
    return out.map((r) => ('error' in r ? codeOf(r.error) : r.code));
  }

  /** Let the opponent named by `view.ai_to_move` play one card. */
  async step(): Promise<Step> {
    return this.took(await this.call<StepState>('runStep', [this.handle]));
  }

  /** Let the run go; requests sent before still get their answers. */
  dispose(): void {
    if (this.disposed) return;
    this.disposed = true;
    void call('runFree', [this.handle]).catch(() => undefined);
  }

  private call<T>(op: CoreOp, args: unknown[]): Promise<T> {
    if (this.disposed) return Promise.reject(disposedError());
    return call<T>(op, args);
  }

  /** A step changed the run: keep its view and save text. */
  private took(s: StepState): Step {
    const step = localize(JSON.parse(s.step) as Step);
    this.lastView = step.view;
    this.lastSave = s.save;
    return step;
  }
}

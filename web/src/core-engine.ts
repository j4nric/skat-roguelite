// The WebAssembly core behind a message interface (ADR 0022). It runs in
// the core worker (./core.worker.ts), or on the main thread where no worker
// is available (./core.ts). It holds the core's objects by handle and
// passes their JSON on as text; it decides nothing itself.

import init, { WasmMeta, WasmRun, catalog } from './wasm/pkg/skat_wasm.js';

/** A request to the engine: `id` pairs it with its answer. */
export interface CoreRequest {
  id: number;
  op: CoreOp;
  args: unknown[];
}

/** The engine's answer: the value, or the core's error text (the JSON of the crate docs). */
export type CoreResponse = { id: number; value: unknown } | { id: number; error: string };

/** A run as it comes out of the core: its handle, view and save text. */
export interface RunState {
  handle: number;
  view: string;
  save: string;
}

/** A step of a run: `{ events, view }` as JSON, and the run's save text after it. */
export interface StepState {
  step: string;
  save: string;
}

/** A meta state: its handle, view and save text. */
export interface MetaState {
  handle: number;
  view: string;
  save: string;
}

export type CoreOp =
  | 'init'
  | 'catalog'
  | 'runCreate'
  | 'runCreateWith'
  | 'runTutorial'
  | 'runLoad'
  | 'runApply'
  | 'runPreview'
  | 'runCheck'
  | 'runStep'
  | 'runFree'
  | 'metaFresh'
  | 'metaLoad'
  | 'metaObserve'
  | 'metaFree'
  | 'isFreshMeta'
  | 'describe';

const runs = new Map<number, WasmRun>();
const metas = new Map<number, WasmMeta>();
let nextHandle = 1;
let ready: Promise<void> | null = null;

function run(handle: unknown): WasmRun {
  const r = runs.get(Number(handle));
  if (!r) throw new Error(JSON.stringify({ code: 'Internal', message: `no run ${String(handle)}` }));
  return r;
}

function meta(handle: unknown): WasmMeta {
  const m = metas.get(Number(handle));
  if (!m) throw new Error(JSON.stringify({ code: 'Internal', message: `no meta state ${String(handle)}` }));
  return m;
}

function keepRun(r: WasmRun): RunState {
  const handle = nextHandle++;
  runs.set(handle, r);
  return { handle, view: r.view(), save: r.save() };
}

function keepMeta(m: WasmMeta): MetaState {
  const handle = nextHandle++;
  metas.set(handle, m);
  return { handle, view: m.view(), save: m.save() };
}

/** The core's message for an error thrown by wasm-bindgen (or anything else). */
export function errorMessage(err: unknown): string {
  return err instanceof Error ? err.message : String(err);
}

/** Load and instantiate the core once, from its bytes or its address. */
export function initCore(wasm: ArrayBuffer | string): Promise<void> {
  ready ??= init({ module_or_path: wasm }).then(() => undefined);
  return ready;
}

/** Carry out one request; throws the core's error. Only `init` is asynchronous. */
export function handle(op: CoreOp, args: unknown[]): unknown {
  const [a, b, c, d, e] = args;
  switch (op) {
    case 'init':
      return initCore(a as ArrayBuffer | string);
    case 'catalog':
      return catalog();
    case 'runCreate':
      return keepRun(WasmRun.newRun(String(a)));
    case 'runCreateWith': {
      const r = WasmRun.newRunWith(String(a), String(b), String(c), Number(d), meta(e));
      // The start events once, then the state.
      const startEvents = r.startEvents();
      return { ...keepRun(r), startEvents };
    }
    case 'runTutorial':
      return keepRun(WasmRun.tutorial());
    case 'runLoad':
      return keepRun(WasmRun.load(String(a)));
    case 'runApply': {
      const r = run(a);
      const step = r.apply(String(b));
      return { step, save: r.save() } satisfies StepState;
    }
    case 'runStep': {
      const r = run(a);
      const step = r.step();
      return { step, save: r.save() } satisfies StepState;
    }
    case 'runPreview':
      return run(a).preview(String(b));
    case 'runCheck': {
      // Many actions at once (the dimmed cards of a picker): per action
      // null, the error code, or the error text of JSON that is no action.
      const r = run(a);
      return (b as string[]).map((action) => {
        try {
          return { code: r.check(action) ?? null };
        } catch (err) {
          return { error: errorMessage(err) };
        }
      });
    }
    case 'runFree': {
      const r = runs.get(Number(a));
      runs.delete(Number(a));
      r?.free();
      return null;
    }
    case 'metaFresh':
      return keepMeta(WasmMeta.fresh());
    case 'metaLoad':
      return keepMeta(WasmMeta.load(String(a)));
    case 'metaObserve': {
      const m = meta(a);
      const step = m.observe(String(b));
      return { step, save: m.save() } satisfies StepState;
    }
    case 'metaFree': {
      const m = metas.get(Number(a));
      metas.delete(Number(a));
      m?.free();
      return null;
    }
    case 'isFreshMeta': {
      // Whether a stored meta state is still the one of a new player.
      const stored = WasmMeta.load(String(a));
      const fresh = WasmMeta.fresh();
      try {
        return stored.save() === fresh.save();
      } finally {
        stored.free();
        fresh.free();
      }
    }
    case 'describe': {
      // The views of a stored run and meta state, for the save code and the
      // account's conflict sheet; throws if the core cannot load one.
      const out: { run: string | null; meta: string | null } = { run: null, meta: null };
      if (a != null) {
        const r = WasmRun.load(String(a));
        try {
          out.run = r.view();
        } finally {
          r.free();
        }
      }
      if (b != null) {
        const m = WasmMeta.load(String(b));
        try {
          out.meta = m.view();
        } finally {
          m.free();
        }
      }
      return out;
    }
    default:
      throw new Error(JSON.stringify({ code: 'Internal', message: `unknown op ${String(op)}` }));
  }
}

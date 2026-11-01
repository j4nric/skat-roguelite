// The only place that talks to the WebAssembly core. Every game fact the
// app shows comes out of here as JSON produced by skat_core.

import init, { WasmRun, catalog as wasmCatalog } from './wasm/pkg/skat_wasm.js';
import wasmUrl from './wasm/pkg/skat_wasm_bg.wasm?url';
import type { Catalog, RunAction, RunView, Step } from './types';

let ready: Promise<void> | null = null;

/** Load and instantiate the core once. */
export function loadCore(): Promise<void> {
  ready ??= init({ module_or_path: wasmUrl }).then(() => undefined);
  return ready;
}

/** Names and texts of all content. */
export function loadCatalog(): Catalog {
  return JSON.parse(wasmCatalog()) as Catalog;
}

/** A run in the core. */
export class CoreRun {
  private constructor(private readonly run: WasmRun) {}

  /** Start a new run with a decimal seed. */
  static create(seed: string): CoreRun {
    return new CoreRun(WasmRun.newRun(seed));
  }

  /** Load a saved run; throws if the save does not fit. */
  static load(state: string): CoreRun {
    return new CoreRun(WasmRun.load(state));
  }

  /** The run as text for localStorage. */
  save(): string {
    return this.run.save();
  }

  view(): RunView {
    return JSON.parse(this.run.view()) as RunView;
  }

  /** Apply a player decision. Throws if the core rejects it. */
  apply(action: RunAction): Step {
    return JSON.parse(this.run.apply(JSON.stringify(action))) as Step;
  }

  dispose(): void {
    this.run.free();
  }
}

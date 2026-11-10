// The only place that talks to the WebAssembly core. Every game fact the
// app shows comes out of here as JSON produced by skat_core. Views pass
// through `localize` (./i18n): the content's German display texts become
// those of the active language.

import init, { WasmMeta, WasmRun, catalog as wasmCatalog } from './wasm/pkg/skat_wasm.js';
import wasmUrl from './wasm/pkg/skat_wasm_bg.wasm?url';
import { localize } from './i18n';
import type { Catalog, Character, MetaStep, MetaView, RunAction, RunEvent, RunLength, RunView, Step } from './types';

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

/** The meta state across runs: unlocks and statistics (DESIGN 9.3). */
export class CoreMeta {
  private constructor(readonly meta: WasmMeta) {}

  /** A new player: nothing unlocked. */
  static fresh(): CoreMeta {
    return new CoreMeta(WasmMeta.fresh());
  }

  /** Load a saved meta state; throws if the save does not fit. */
  static load(state: string): CoreMeta {
    return new CoreMeta(WasmMeta.load(state));
  }

  /** The meta state as text for localStorage. */
  save(): string {
    return this.meta.save();
  }

  view(): MetaView {
    return localize(JSON.parse(this.meta.view()) as MetaView);
  }

  /** Let the meta state learn from the events of a step. */
  observe(events: RunEvent[] | string): MetaStep {
    const json = typeof events === 'string' ? events : JSON.stringify(events);
    return localize(JSON.parse(this.meta.observe(json)) as MetaStep);
  }
}

/** A run in the core. */
export class CoreRun {
  private constructor(private readonly run: WasmRun) {}

  /** Start a new run of the Alleinspieler with every relic unlocked. */
  static create(seed: string): CoreRun {
    return new CoreRun(WasmRun.newRun(seed));
  }

  /**
   * Start a new run with a character, length and stake (Einsatz-Leiter) from the
   * meta state. Returns the run and the events of its start as JSON (feed them to
   * the meta state). A stake above the unlocked one throws `StakeLocked`.
   */
  static createWith(seed: string, character: Character, length: RunLength, stake: number, meta: CoreMeta): { run: CoreRun; startEvents: string } {
    const run = WasmRun.newRunWith(seed, character, length, stake, meta.meta);
    return { run: new CoreRun(run), startEvents: run.startEvents() };
  }

  /**
   * Start the Einführung (DESIGN 16.1): one guided pub with the fixed deals
   * of the core's `tutorial.ron`. Never feed its events to the meta state
   * and never save it: it is no run.
   */
  static tutorial(): CoreRun {
    return new CoreRun(WasmRun.tutorial());
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
    return localize(JSON.parse(this.run.view()) as RunView);
  }

  /**
   * Apply a player decision. The opponents do not play here: while the
   * returned view's `ai_to_move` is set, call `step()`. Throws if the core
   * rejects it.
   */
  apply(action: RunAction): Step {
    return localize(JSON.parse(this.run.apply(JSON.stringify(action))) as Step);
  }

  /**
   * What an action would do, without doing it: the core applies it to a
   * copy of the run and returns that copy's events and view. Throws the
   * core's error (`CardCursed`, `ModAtMaximum`, …) like `apply`. Read-only;
   * an action that draws at random draws on the copy, so its outcome must
   * not be shown.
   */
  preview(action: RunAction): Step {
    return localize(JSON.parse(this.run.preview(JSON.stringify(action))) as Step);
  }

  /**
   * Whether the core would accept an action now: null if so, else its
   * error code (`CardCursed`, `ModAtMaximum`, …, as `apply` would throw
   * it). Builds no view and no events, unlike `preview`: for asking about
   * many actions at once (the dimmed cards of a picker). Throws only on
   * JSON that is no action.
   */
  check(action: RunAction): string | null {
    return this.run.check(JSON.stringify(action)) ?? null;
  }

  /** Let the opponent named by `view.ai_to_move` play one card. */
  step(): Step {
    return localize(JSON.parse(this.run.step()) as Step);
  }

  dispose(): void {
    this.run.free();
  }
}

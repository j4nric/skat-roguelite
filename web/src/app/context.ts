// What the app modules (./*.ts, ./playback/*.ts) share: the core, the UI
// state, the DOM layers and the few calls every module needs. The App
// (../app.ts) is this context; each module gets it and, where it needs
// them, the modules below it, so the imports never go round in a circle.

import type { CoreMeta, CoreRun } from '../core';
import type { RunAction, RunView } from '../types';
import type { Drink } from '../ui/drink';
import type { InfoCard } from '../ui/infocard';
import type { HudFx } from '../ui/juice';
import type { UiState } from '../ui/state';
import type { TriggerFeed } from '../ui/triggers';
import type { ToastPlace } from './consts';

export interface AppCtx {
  readonly root: HTMLElement;
  /** The screen layer, drawn part by part (./render.ts). */
  readonly screenEl: HTMLElement;
  /** The dialog layer (./layer.ts). */
  readonly layerEl: HTMLElement;
  readonly ui: UiState;
  /** The run in the core; null on the start screen without a saved run. */
  core: CoreRun | null;
  /** Unlocks and statistics across runs (null if the core cannot keep them). */
  meta: CoreMeta | null;
  /** The player asked to hurry: animations and pauses are skipped. */
  fastForward: boolean;
  /** Effects to run once the next render is in the DOM. */
  readonly afterRender: (() => void)[];
  /** A boss pub just opened: its entrance plays once the table is free of dialogs. */
  entrancePending: boolean;
  /** Account and Groschen in the head: how their changes get there. */
  readonly hudFx: HudFx;
  /** The info card (long-press, hover, "i"): one for everything. */
  readonly info: InfoCard;
  /** Lines and wiggles when a relic or modification fires. */
  readonly triggers: TriggerFeed;
  /** Drinking a Schnaps from the Leiste (drag and drop or tap, ../ui/drink.ts). */
  readonly drink: Drink;
  /** Draw the screen from the state (./render.ts). */
  render(): void;
  /** Send an action to the core and play back what happened (./playback/steps.ts). */
  act(action: RunAction, fast?: boolean): Promise<void>;
  /** Store what the playback kept in memory (./progress.ts). */
  persist(): void;
  /** A short message (./playback/fx.ts). */
  showToast(text: string, place?: ToastPlace): void;
}

/** A tap on an element with `data-action` (./actions.ts). */
export interface Tap {
  readonly el: HTMLElement;
  readonly action: string;
  /** A number from the element's data set (-1 if missing). */
  num(key: string): number;
}

/** Handlers by `data-action` that run whatever is on screen. */
export type Actions = Record<string, (tap: Tap) => void>;

/** Handlers that need a run on screen and no playback: they get its view. */
export type RunActions = Record<string, (tap: Tap, view: RunView) => void>;

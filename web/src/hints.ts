// "Geführter erster Abend" (DESIGN 16): the Wirt explains in a short speech
// bubble what happens for the first time. Which hints the player has seen
// is kept in localStorage under a `stammtisch.` key, so the save code
// (./savecode.ts) carries it along. Every access is guarded.
//
// The triggers only read what the screen already shows (the core's view,
// the open dialog, an event just played back). No rule is decided here.

import type { UiState } from './ui/state';

export type HintId =
  | 'welcome'
  | 'skatOrHand'
  | 'afterSkat'
  | 'value'
  | 'pass'
  | 'score'
  | 'shop'
  | 'boss'
  | 'encounter'
  | 'runLost';

/**
 * Where the bubble sits: under the head of the run, at the top (over a
 * dialog), under the end title, or at the Tresen under the Wirt who already
 * stands there (then without a second portrait).
 */
export type HintAnchor = 'hud' | 'top' | 'end' | 'shop';

/** What a hint may look at. */
export interface HintContext {
  ui: UiState;
  /** The player passed a game during this visit (event `Passed`). */
  passed: boolean;
}

interface HintDef {
  id: HintId;
  anchor: HintAnchor;
  /** The dialog the hint talks about; without, it only shows when no dialog is open. */
  overlay?: 'score' | 'boss';
  /** The situation the hint is about is on screen. */
  when: (ctx: HintContext) => boolean;
}

/** The table of a game is on screen (not the Tresen, not an encounter). */
function table(ui: UiState): NonNullable<NonNullable<UiState['view']>['game']> | null {
  const view = ui.view;
  if (ui.screen !== 'run' || !view || view.phase !== 'Game') return null;
  return view.game ?? null;
}

/**
 * The hints in the order they win when several apply at once. A pass deals
 * a new game: its hint comes before the hints about declaring.
 */
const HINTS: HintDef[] = [
  { id: 'runLost', anchor: 'end', when: ({ ui }) => ui.screen === 'end' && ui.ended?.won === false },
  { id: 'boss', anchor: 'top', overlay: 'boss', when: ({ ui }) => ui.overlay?.kind === 'boss' },
  { id: 'score', anchor: 'top', overlay: 'score', when: ({ ui }) => ui.overlay?.kind === 'score' },
  { id: 'pass', anchor: 'hud', when: ({ ui, passed }) => passed && table(ui)?.phase === 'Declaring' },
  {
    id: 'welcome',
    anchor: 'hud',
    when: ({ ui }) => {
      const phase = table(ui)?.phase;
      return phase === 'Declaring' || phase === 'Discarding' || phase === 'DeclaringAfterSkat';
    },
  },
  {
    id: 'skatOrHand',
    anchor: 'hud',
    when: ({ ui }) => {
      const game = table(ui);
      return game?.phase === 'Declaring' && Boolean(game.declare) && !ui.anim;
    },
  },
  {
    id: 'afterSkat',
    anchor: 'hud',
    when: ({ ui }) => {
      const game = table(ui);
      return game?.phase === 'DeclaringAfterSkat' && Boolean(game.declare) && !ui.anim;
    },
  },
  { id: 'value', anchor: 'hud', when: ({ ui }) => table(ui)?.phase === 'Playing' && Boolean(table(ui)?.value_preview) },
  { id: 'shop', anchor: 'shop', when: ({ ui }) => ui.screen === 'run' && ui.view?.phase === 'Shop' && !ui.anim },
  { id: 'encounter', anchor: 'hud', when: ({ ui }) => ui.screen === 'run' && ui.view?.phase === 'Encounter' && !ui.anim },
];

/** Whether nothing else is in the way of a hint (a dialog, the Kontra question). */
function free(def: HintDef, ui: UiState): boolean {
  if (def.overlay) return ui.overlay?.kind === def.overlay;
  if (ui.overlay) return false;
  return !ui.view?.game?.awaiting_kontra_answer;
}

/** Whether the situation of a hint is still on screen (a dialog in front does not count). */
export function hintApplies(id: HintId, ctx: HintContext): boolean {
  return HINTS.find((h) => h.id === id)?.when(ctx) ?? false;
}

/** The hint to show now: the first one not yet seen whose situation is on screen. */
export function pickHint(ctx: HintContext, seen: ReadonlySet<string>): { id: HintId; anchor: HintAnchor } | null {
  for (const def of HINTS) {
    if (seen.has(def.id)) continue;
    if (def.when(ctx) && free(def, ctx.ui)) return { id: def.id, anchor: def.anchor };
  }
  return null;
}

// ---------------------------------------------------------------------------
// Storage
// ---------------------------------------------------------------------------

const KEY = 'stammtisch.hints';

/** The hints the player has seen (dismissed, or moved on from). */
export function loadSeenHints(): Set<string> {
  try {
    const raw = window.localStorage.getItem(KEY);
    if (!raw) return new Set();
    const parsed: unknown = JSON.parse(raw);
    return new Set(Array.isArray(parsed) ? parsed.filter((x): x is string => typeof x === 'string') : []);
  } catch {
    return new Set();
  }
}

export function saveSeenHints(seen: ReadonlySet<string>): void {
  try {
    window.localStorage.setItem(KEY, JSON.stringify([...seen].sort()));
  } catch {
    // Storage blocked: the hints come back next visit.
  }
}

/** "Hinweise zurücksetzen": every hint shows again. */
export function clearSeenHints(): void {
  try {
    window.localStorage.removeItem(KEY);
  } catch {
    // Ignore.
  }
}

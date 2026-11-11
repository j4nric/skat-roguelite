// Taps on elements with `data-action`: one table of handlers by name,
// filled by the feature modules (start and menu, table, Tresen, account,
// save code, …). The names in the DOM are the keys; scripts/check-actions.mjs
// checks that every name the templates use has a handler here.

import { sound } from '../audio';
import type { RunView } from '../types';
import { spring } from '../ui/juice';
import type { Actions, AppCtx, RunActions, Tap } from './context';
import type { Fx } from './playback/fx';

export class ActionTable {
  /** Handlers that run whatever is on screen. */
  private readonly any = new Map<string, (tap: Tap) => void>();
  /** Handlers that need a run on screen and no playback. */
  private readonly run = new Map<string, (tap: Tap, view: RunView) => void>();

  constructor(
    private readonly ctx: AppCtx,
    private readonly fx: Fx,
  ) {}

  /** Register a module's handlers; a name registered twice is a mistake in the code. */
  add(any: Actions, run: RunActions = {}): void {
    for (const [name, handler] of Object.entries(any)) {
      this.claim(name);
      this.any.set(name, handler);
    }
    for (const [name, handler] of Object.entries(run)) {
      this.claim(name);
      this.run.set(name, handler);
    }
  }

  private claim(name: string): void {
    if (this.any.has(name) || this.run.has(name)) throw new Error(`action "${name}" has two handlers`);
  }

  /** Every action name with a handler. */
  names(): string[] {
    return [...this.any.keys(), ...this.run.keys()];
  }

  onClick(e: MouseEvent): void {
    const ui = this.ctx.ui;
    const target = e.target as HTMLElement | null;
    const el = target?.closest<HTMLElement>('[data-action]');
    if (!el) {
      if (ui.busy && !ui.overlay) this.fx.hurry();
      return;
    }
    const action = el.dataset.action ?? '';
    this.ctx.hudFx.noteTap(el);
    // Cards that can be dragged spring here, not with :active (that would shrink a drag).
    if (el.matches('[data-drag]')) spring(el);
    // A tap on the backdrop of a dialog only counts if it hit the backdrop.
    if (action === 'close-overlay' && el.classList.contains('overlay') && target !== el) {
      return;
    }
    if (el instanceof HTMLButtonElement && el.disabled) return;
    // A quiet tap for buttons; cards make their own sound when they move.
    if (el instanceof HTMLButtonElement && !el.matches('.card, [data-drag]') && action !== 'sound') sound.play('tap');
    const tap: Tap = { el, action, num: (key) => Number(el.dataset[key] ?? '-1') };
    const any = this.any.get(action);
    if (any) {
      any(tap);
      return;
    }
    const view = ui.view;
    if (ui.busy || !view) return;
    const run = this.run.get(action);
    if (run) run(tap, view);
    else if (import.meta.env.DEV) console.warn(`no handler for data-action "${action}"`);
  }
}

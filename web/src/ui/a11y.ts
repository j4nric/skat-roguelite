// Screen reader and keyboard aids (Barrierefreiheit, DESIGN 16): one polite
// live region for the few events worth saying aloud (an opponent's card, a
// trick won, a game's result), and focus handling for dialogs (focus into
// a dialog when it opens, kept inside it while it is open, back to where
// it was when it closes). Escape closes a dialog where the app allows it
// (app.ts, ./infocard.ts, ./keys.ts).
//
// Pure presentation: what is said comes from the view data and de.ts, as
// the screen shows it.

import { de, t } from '../i18n';
import { lookLabel } from './cards';
import { seatLabel } from './guests';
import { signed } from './score';
import type { UiState } from './state';

// ------------------------------------------------------------ live region

let live: HTMLElement | null = null;

/** Put the live region into `root` once (a visually hidden status line). */
export function mountLive(root: HTMLElement): void {
  if (live?.isConnected) return;
  live = document.createElement('div');
  live.className = 'sr-only sr-live';
  live.setAttribute('role', 'status');
  live.setAttribute('aria-live', 'polite');
  live.setAttribute('aria-relevant', 'additions');
  live.setAttribute('data-testid', 'sr-live');
  root.appendChild(live);
}

/** How many lines the live region keeps (older ones have been read). */
const KEEP = 4;

/**
 * Say `text` politely: a new line in the live region, read after what the
 * screen reader is saying now. Lines are added, not replaced, so two
 * events in quick succession are both heard.
 */
export function announce(text: string): void {
  if (!live || !text) return;
  const line = document.createElement('p');
  line.textContent = text;
  live.appendChild(line);
  while (live.childElementCount > KEEP) live.firstElementChild?.remove();
}

/**
 * Watches the state after every render and announces what changed: a card
 * an opponent put into the trick, the line of a trick won (the status line
 * of the playback), the result when the score board opens.
 */
export class Announcer {
  /** Cards already said in this game (a card is played once per game). */
  private said = new Set<number>();
  private game = '';
  private status = '';
  private score: unknown = null;

  observe(ui: UiState, quiet: boolean): void {
    const anim = ui.anim;
    const view = ui.view;
    const game = view?.game ? `${view.seed}:${view.series.section}:${view.series.index}:${view.game.number}` : '';
    if (game !== this.game) {
      this.game = game;
      this.said.clear();
    }
    for (const p of anim?.trick ?? []) {
      if (this.said.has(p.card.id)) continue;
      this.said.add(p.card.id);
      if (quiet || p.seat === 0 || !view) continue;
      announce(t(de.a11y.played, { name: seatLabel(view, p.seat), card: lookLabel(p.card) }));
    }
    const status = anim?.status ?? '';
    if (status && status !== this.status && !quiet) announce(status);
    this.status = status;
    const o = ui.overlay;
    if (o?.kind === 'score') {
      if (o !== this.score) {
        const b = o.score.breakdown;
        announce(t(b.won ? de.a11y.won : de.a11y.lost, { score: signed(b.score) }));
      }
      this.score = o;
    } else {
      this.score = null;
    }
  }
}

// ------------------------------------------------------------------ focus

const FOCUSABLE = 'button:not([disabled]), a[href], input:not([disabled]), select:not([disabled]), textarea:not([disabled]), [tabindex]:not([tabindex="-1"])';

function shown(el: Element): boolean {
  return el.getClientRects().length > 0 && getComputedStyle(el).visibility !== 'hidden';
}

/** The controls inside `box` that Tab reaches, in order. */
export function focusables(box: Element): HTMLElement[] {
  return Array.from(box.querySelectorAll<HTMLElement>(FOCUSABLE)).filter((el) => !el.closest('[inert], [aria-hidden="true"]') && shown(el));
}

/** The modal dialog on top: the last one shown in document order. */
function topDialog(): HTMLElement | null {
  const all = Array.from(document.querySelectorAll<HTMLElement>('[role="dialog"][aria-modal="true"]')).filter(shown);
  return all[all.length - 1] ?? null;
}

/** A way to find an element again after its part of the screen was drawn anew. */
function keyOf(el: HTMLElement): string | null {
  const action = el.dataset.action;
  if (!action) return el.id ? `#${CSS.escape(el.id)}` : null;
  const testid = el.dataset.testid;
  return testid ? `[data-action="${CSS.escape(action)}"][data-testid="${CSS.escape(testid)}"]` : `[data-action="${CSS.escape(action)}"]`;
}

/**
 * Focus for the dialogs of one container (the layer, or the screen with its
 * own sheets like the Kontra): on opening, focus goes to the dialog (its
 * name is read); while the same dialog is drawn again, it stays on the
 * same control; on closing it returns to the control that opened it.
 */
export class DialogFocus {
  private opener: { el: HTMLElement; key: string | null } | null = null;
  private open: HTMLElement | null = null;

  /** `box` is asked for when needed: the app's containers may not exist yet when this is made. */
  constructor(private readonly container: () => HTMLElement) {}

  private get box(): HTMLElement {
    return this.container();
  }

  /** Before the container is drawn again: the focused control's place in it, if any. */
  before(): number | null {
    const active = document.activeElement;
    if (!(active instanceof HTMLElement) || active === document.body) return null;
    if (!this.box.contains(active)) {
      // Focus outside: whoever has it opens the next dialog.
      if (!this.open) this.opener = { el: active, key: keyOf(active) };
      return null;
    }
    const i = focusables(this.box).indexOf(active);
    return i >= 0 ? i : null;
  }

  /** After drawing: `same` is true when the dialog shown is the one shown before. */
  after(keep: number | null, same: boolean): void {
    const dialog = this.box.querySelector<HTMLElement>('[role="dialog"][aria-modal="true"]');
    if (dialog) {
      const fresh = !this.open || !same;
      this.open = dialog;
      const active = document.activeElement;
      if (active instanceof HTMLElement && dialog.contains(active)) return;
      const again = !fresh && keep !== null ? focusables(dialog)[keep] : undefined;
      if (again) {
        again.focus({ preventScroll: true });
        return;
      }
      if (!dialog.hasAttribute('tabindex')) dialog.setAttribute('tabindex', '-1');
      dialog.focus({ preventScroll: true });
      return;
    }
    if (!this.open) return;
    this.open = null;
    const opener = this.opener;
    this.opener = null;
    const active = document.activeElement;
    // Focus already went somewhere on purpose (a new screen took it): leave it.
    if (active instanceof HTMLElement && active !== document.body && active.isConnected) return;
    const back = opener?.el.isConnected ? opener.el : opener?.key ? document.querySelector<HTMLElement>(opener.key) : null;
    back?.focus({ preventScroll: true });
  }
}

/**
 * Keep Tab inside the modal dialog on top: from its last control to its
 * first and back; focus outside the dialog comes into it.
 */
export function trapFocus(): void {
  window.addEventListener('keydown', (e) => {
    if (e.key !== 'Tab' || e.defaultPrevented) return;
    const dialog = topDialog();
    if (!dialog) return;
    const items = focusables(dialog);
    const first = items[0];
    const last = items[items.length - 1];
    const active = document.activeElement;
    if (!first || !last) {
      e.preventDefault();
      dialog.focus({ preventScroll: true });
      return;
    }
    const inside = active instanceof HTMLElement && dialog.contains(active);
    if (!inside || (e.shiftKey && (active === first || active === dialog)) || (!e.shiftKey && active === last)) {
      e.preventDefault();
      (e.shiftKey ? last : first).focus({ preventScroll: true });
    }
  });
}

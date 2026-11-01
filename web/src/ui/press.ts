// Input for the info card (./infocard.ts) on everything marked with
// `data-info`, kept apart from the normal tap:
//
// - Long-press (400 ms, a small ring fills at the finger) opens the card.
//   Moving more than a few pixels or lifting the finger earlier cancels it,
//   and the tap does what it always did. Once the card opened, the click
//   that follows the release is swallowed, so a long-press never plays a
//   card. The context menu of a long touch (and a right click) is replaced
//   by the card.
// - With a mouse, resting on a thing for 300 ms shows the card as a popover;
//   it goes when the pointer leaves (unless onto the popover).
// - Keyboard: "i" or "?" on the focused thing opens the card; on things
//   without an action of their own Enter and Space do too.
//
// Pure input handling: what the card says comes from ./info.ts.

import { clearSelection } from './dom';

/** How long a press must last to open the card. */
export const LONG_PRESS_MS = 400;
/** How long the mouse must rest before the popover shows. */
export const HOVER_MS = 300;
/** Movement that turns a press into a scroll or a swipe. */
const SLOP = 10;

let swallowUntil = 0;

/** Swallow the next click (within a moment): it belongs to a gesture already handled. */
export function swallowClick(ms = 700): void {
  swallowUntil = performance.now() + ms;
}

// Installed once: a click right after a handled gesture goes nowhere.
if (typeof window !== 'undefined') {
  window.addEventListener(
    'click',
    (e) => {
      if (performance.now() > swallowUntil) return;
      swallowUntil = 0;
      e.preventDefault();
      e.stopImmediatePropagation();
    },
    true,
  );
}

export interface PressHandlers {
  /** Open the card as a sheet for the thing with this key. */
  open: (key: string, el: HTMLElement) => void;
  /** Show the popover (mouse rests on the thing). */
  hover: (key: string, el: HTMLElement) => void;
  /** The mouse left the thing. */
  leave: () => void;
  /** Whether input for the card is off right now. */
  blocked?: () => boolean;
}

function infoTarget(target: EventTarget | null): HTMLElement | null {
  return target instanceof Element ? target.closest<HTMLElement>('[data-info]') : null;
}

/** Listen on `root` for long-press, hover and keys on `[data-info]` things. */
export function watchPresses(root: HTMLElement, h: PressHandlers): void {
  let press: { id: number; x: number; y: number; key: string; el: HTMLElement; timer: number; ring: HTMLElement | null } | null = null;
  let hoverTimer: number | undefined;
  let hoverKey: string | null = null;

  const cancel = (): void => {
    if (!press) return;
    window.clearTimeout(press.timer);
    press.ring?.remove();
    press = null;
  };

  root.addEventListener('pointerdown', (e) => {
    cancel();
    window.clearTimeout(hoverTimer);
    if (e.pointerType === 'mouse' && e.button !== 0) return;
    const el = infoTarget(e.target);
    const key = el?.dataset.info;
    if (!el || !key || h.blocked?.()) return;
    const ring = document.createElement('div');
    ring.className = 'press-ring';
    ring.setAttribute('aria-hidden', 'true');
    ring.style.left = `${Math.round(e.clientX)}px`;
    ring.style.top = `${Math.round(e.clientY)}px`;
    ring.style.setProperty('--press-ms', `${LONG_PRESS_MS}ms`);
    ring.innerHTML = '<svg viewBox="0 0 36 36"><circle cx="18" cy="18" r="15"/></svg>';
    // The ring only shows after a short moment, so a quick tap shows nothing.
    root.appendChild(ring);
    const timer = window.setTimeout(() => {
      if (!press) return;
      const p = press;
      cancel();
      swallowClick();
      clearSelection();
      h.open(p.key, p.el);
    }, LONG_PRESS_MS);
    press = { id: e.pointerId, x: e.clientX, y: e.clientY, key, el, timer, ring };
  });

  root.addEventListener('pointermove', (e) => {
    if (press && press.id === e.pointerId && Math.hypot(e.clientX - press.x, e.clientY - press.y) > SLOP) cancel();
  });
  for (const type of ['pointerup', 'pointercancel', 'lostpointercapture'] as const) {
    root.addEventListener(type, (e) => {
      if (press && press.id === e.pointerId) cancel();
    });
  }
  window.addEventListener('scroll', cancel, true);
  window.addEventListener('blur', cancel);

  // A long touch or a right click: the card instead of the browser's menu.
  root.addEventListener('contextmenu', (e) => {
    const el = infoTarget(e.target);
    const key = el?.dataset.info;
    if (!el || !key) return;
    e.preventDefault();
    if (h.blocked?.()) return;
    if (press) {
      cancel();
      swallowClick();
    }
    h.open(key, el);
  });

  // Hover with a mouse: the popover.
  root.addEventListener('pointerover', (e) => {
    if (e.pointerType !== 'mouse') return;
    const el = infoTarget(e.target);
    const key = el?.dataset.info ?? null;
    if (key === hoverKey) return;
    window.clearTimeout(hoverTimer);
    hoverKey = key;
    if (!el || !key) return;
    hoverTimer = window.setTimeout(() => {
      if (hoverKey === key && el.isConnected && !press && !h.blocked?.()) h.hover(key, el);
    }, HOVER_MS);
  });
  root.addEventListener('pointerout', (e) => {
    if (e.pointerType !== 'mouse') return;
    const el = infoTarget(e.target);
    if (!el || el.contains(e.relatedTarget as Node | null)) return;
    // Onto another thing with a card: pointerover takes it from here.
    if (infoTarget(e.relatedTarget)) return;
    window.clearTimeout(hoverTimer);
    hoverKey = null;
    h.leave();
  });

  // Keyboard.
  root.addEventListener('keydown', (e) => {
    if (e.altKey || e.ctrlKey || e.metaKey) return;
    const el = infoTarget(e.target);
    const key = el?.dataset.info;
    if (!el || !key) return;
    const ownAction = el.hasAttribute('data-action') || el.closest('[data-action]') !== null;
    if (e.key === 'i' || e.key === 'I' || e.key === '?' || (!ownAction && (e.key === 'Enter' || e.key === ' '))) {
      e.preventDefault();
      if (!h.blocked?.()) h.open(key, el);
    }
  });
}

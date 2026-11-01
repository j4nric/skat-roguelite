// Dragging on the table: things marked `data-drag="<kind>"` can be pulled
// onto places marked `data-drop="<name>"` that accept that kind
// (`data-accept`). A copy of the thing follows the finger; the place under
// it lights up; letting go there hands the drop to the app. A quick flick
// towards a place marked `data-swipe` counts as a drop there too (swiping
// the deck into the middle). A press that does not move further than a few
// pixels stays a normal tap, so every drag has a tap as its fallback.
//
// Pure input handling: what a drop means is decided by the app (and what
// is allowed by the core).

import { motionOn } from './fx';
import { swallowClick } from './press';

/** Movement that turns a press into a drag (as for the long-press). */
const SLOP = 10;
/** A flick: at least this fast (px per ms) and this far, towards the place. */
const SWIPE_SPEED = 0.45;
const SWIPE_DISTANCE = 36;

export interface DragHandlers {
  /**
   * The thing `el` was let go over `zone` (null: nowhere). `over` is the
   * element under the finger inside the zone, for drops onto a card; `at`
   * where the copy was let go.
   */
  drop: (el: HTMLElement, zone: HTMLElement | null, over: Element | null, at: DOMRect | null) => void;
  /** Whether dragging is off right now. */
  blocked?: () => boolean;
}

interface Drag {
  id: number;
  el: HTMLElement;
  kind: string;
  x0: number;
  y0: number;
  /** Recent positions for the speed of a flick. */
  trail: { x: number; y: number; t: number }[];
  ghost: HTMLElement | null;
  zone: HTMLElement | null;
}

function zonesFor(root: HTMLElement, kind: string): HTMLElement[] {
  return Array.from(root.querySelectorAll<HTMLElement>('[data-drop]')).filter((z) => (z.dataset.accept ?? '').split(' ').includes(kind));
}

function inside(r: DOMRect, x: number, y: number, pad = 0): boolean {
  return x >= r.left - pad && x <= r.right + pad && y >= r.top - pad && y <= r.bottom + pad;
}

/** Listen on `root` for drags of `[data-drag]` things. */
export function watchDrags(root: HTMLElement, h: DragHandlers): void {
  let drag: Drag | null = null;

  const zoneAt = (kind: string, x: number, y: number, origin: HTMLElement): HTMLElement | null => {
    // The place the thing came from does not count as a drop.
    const from = origin.closest<HTMLElement>('[data-drop]');
    return zonesFor(root, kind).find((z) => z !== from && inside(z.getBoundingClientRect(), x, y, 6)) ?? null;
  };

  const mark = (zone: HTMLElement | null): void => {
    if (!drag || drag.zone === zone) return;
    drag.zone?.classList.remove('is-over');
    zone?.classList.add('is-over');
    drag.zone = zone;
  };

  const end = (): void => {
    if (!drag) return;
    drag.ghost?.remove();
    drag.el.classList.remove('is-dragging');
    drag.zone?.classList.remove('is-over');
    root.classList.remove(`is-dragging-${drag.kind}`);
    drag = null;
  };

  root.addEventListener('pointerdown', (e) => {
    end();
    if (e.pointerType === 'mouse' && e.button !== 0) return;
    const el = e.target instanceof Element ? e.target.closest<HTMLElement>('[data-drag]') : null;
    if (!el || h.blocked?.()) return;
    if (el instanceof HTMLButtonElement && el.disabled) return;
    drag = { id: e.pointerId, el, kind: el.dataset.drag ?? '', x0: e.clientX, y0: e.clientY, trail: [{ x: e.clientX, y: e.clientY, t: e.timeStamp }], ghost: null, zone: null };
  });

  window.addEventListener('pointermove', (e) => {
    const d = drag;
    if (!d || d.id !== e.pointerId) return;
    d.trail.push({ x: e.clientX, y: e.clientY, t: e.timeStamp });
    if (d.trail.length > 6) d.trail.shift();
    const dx = e.clientX - d.x0;
    const dy = e.clientY - d.y0;
    if (!d.ghost) {
      if (Math.hypot(dx, dy) <= SLOP) return;
      // An info sheet opened by a long-press owns the finger now.
      if (h.blocked?.()) {
        end();
        return;
      }
      const r = d.el.getBoundingClientRect();
      const ghost = d.el.cloneNode(true) as HTMLElement;
      ghost.removeAttribute('data-testid');
      ghost.removeAttribute('data-drag');
      ghost.removeAttribute('data-action');
      ghost.removeAttribute('data-info');
      ghost.setAttribute('aria-hidden', 'true');
      ghost.classList.add('drag-ghost');
      ghost.style.left = `${r.left}px`;
      ghost.style.top = `${r.top}px`;
      ghost.style.width = `${r.width}px`;
      ghost.style.height = `${r.height}px`;
      document.body.appendChild(ghost);
      d.ghost = ghost;
      d.el.classList.add('is-dragging');
      root.classList.add(`is-dragging-${d.kind}`);
    }
    const tilt = Math.max(-12, Math.min(12, dx / 12));
    d.ghost.style.transform = `translate(${dx}px, ${dy}px) rotate(${tilt}deg) scale(1.06)`;
    mark(zoneAt(d.kind, e.clientX, e.clientY, d.el));
  });

  const release = (e: PointerEvent): void => {
    const d = drag;
    if (!d || d.id !== e.pointerId) return;
    if (!d.ghost) {
      // Never moved: a tap, the click does the rest.
      drag = null;
      return;
    }
    // The click a mouse sends after the drag (on what lies under both
    // ends) is no tap; it follows at once, so only that one is swallowed.
    swallowClick(80);
    let zone = zoneAt(d.kind, e.clientX, e.clientY, d.el);
    if (!zone && e.type === 'pointerup') zone = flickTarget(d, e);
    const over = zone ? document.elementsFromPoint(e.clientX, e.clientY).find((x) => x !== d.ghost && zone?.contains(x) && x !== zone) ?? null : null;
    const el = d.el;
    const ghost = d.ghost;
    const at = ghost.getBoundingClientRect();
    if (zone?.hasAttribute('data-swipe') && motionOn()) {
      // The deck lands in the middle, then goes (the new deal follows).
      d.ghost = null;
      const z = zone.getBoundingClientRect();
      const base = ghost.style.transform;
      const anim = ghost.animate(
        [{ transform: base }, { transform: `${base} translate(${z.left + z.width / 2 - (at.left + at.width / 2)}px, ${z.top + z.height / 2 - (at.top + at.height / 2)}px) scale(0.9)`, opacity: 0.3 }],
        { duration: 160, easing: 'ease-out', fill: 'forwards' },
      );
      const gone = (): void => ghost.remove();
      anim.finished.then(gone, gone);
    }
    end();
    if (e.type === 'pointerup') h.drop(el, zone, over, at);
  };
  window.addEventListener('pointerup', release);
  window.addEventListener('pointercancel', release);

  /** A flick: fast enough and pointing at a place that takes swipes. */
  function flickTarget(d: Drag, e: PointerEvent): HTMLElement | null {
    const first = d.trail[0];
    if (!first) return null;
    const dt = Math.max(1, e.timeStamp - first.t);
    const vx = (e.clientX - first.x) / dt;
    const vy = (e.clientY - first.y) / dt;
    const speed = Math.hypot(vx, vy);
    if (speed < SWIPE_SPEED || Math.hypot(e.clientX - d.x0, e.clientY - d.y0) < SWIPE_DISTANCE) return null;
    return (
      zonesFor(root, d.kind)
        .filter((z) => z.hasAttribute('data-swipe'))
        .find((z) => {
          const r = z.getBoundingClientRect();
          const tx = r.left + r.width / 2 - e.clientX;
          const ty = r.top + r.height / 2 - e.clientY;
          // Within about 35 degrees of the way to the place.
          return (vx * tx + vy * ty) / (speed * Math.hypot(tx, ty) || 1) > 0.82;
        }) ?? null
    );
  }
}

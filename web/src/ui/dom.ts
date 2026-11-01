// Small DOM helpers.

const ESCAPES: Record<string, string> = {
  '&': '&amp;',
  '<': '&lt;',
  '>': '&gt;',
  '"': '&quot;',
  "'": '&#39;',
};

/** Escape text for HTML. */
export function esc(text: string | number): string {
  return String(text).replace(/[&<>"']/g, (c) => ESCAPES[c] ?? c);
}

/** Attribute string from a map, skipping null/false values. */
export function attrs(map: Record<string, string | number | boolean | null | undefined>): string {
  return Object.entries(map)
    .filter(([, v]) => v !== null && v !== undefined && v !== false)
    .map(([k, v]) => (v === true ? k : `${k}="${esc(v as string | number)}"`))
    .join(' ');
}

/** Whether the user prefers reduced motion. */
export function reducedMotion(): boolean {
  try {
    return window.matchMedia('(prefers-reduced-motion: reduce)').matches;
  } catch {
    return false;
  }
}

/**
 * Drop a text selection the browser may have started (a long touch on the
 * table). A selection inside a text field is left alone.
 */
export function clearSelection(): void {
  const active = document.activeElement;
  if (active instanceof HTMLInputElement || active instanceof HTMLTextAreaElement) return;
  try {
    const sel = window.getSelection();
    if (sel && sel.rangeCount && !sel.isCollapsed) sel.removeAllRanges();
  } catch {
    // Nothing selected, nothing to do.
  }
}

// The desktop layout (DESIGN 16.8). Two thresholds, both on the window:
// - wide: a desktop window. Every screen sits in one column of the same
//   width (style.css `--desk-col`), and the hand lies in one row.
// - sides: a desktop window with room beside that column. At the table
//   the column is flanked by two of its own: on the left the relics and
//   Schnäpse (out of the head), on the right the last trick, the tricks
//   and "Der Zähler" (out of the felt).
// The table draws different HTML for them (one row, the side columns), so
// the thresholds live here and the stylesheet follows the classes the
// table sets (`.run.is-sides`, `.table.is-wide`), never a media query of
// its own. Presentation only.

export const WIDE_QUERY = '(min-width: 900px) and (min-height: 600px)';
export const SIDES_QUERY = '(min-width: 1100px) and (min-height: 640px)';

function matches(query: string): boolean {
  try {
    return typeof window !== 'undefined' && window.matchMedia(query).matches;
  } catch {
    return false;
  }
}

/** Whether the window is a desktop window (one column, the hand in one row). */
export function isWide(): boolean {
  return matches(WIDE_QUERY);
}

/** Whether the table has room for its side columns. */
export function hasSides(): boolean {
  return matches(SIDES_QUERY);
}

/** Calls `change` whenever the window crosses one of the thresholds. */
export function watchLayout(change: () => void): void {
  if (typeof window === 'undefined' || !window.matchMedia) return;
  for (const q of [WIDE_QUERY, SIDES_QUERY]) window.matchMedia(q).addEventListener('change', change);
}

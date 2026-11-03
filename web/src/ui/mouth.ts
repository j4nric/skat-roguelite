// The first-person mouth for drinking a Schnaps yourself (DESIGN 8.9,
// user decision 03.11.2026: comic mouth "M-A"): big lips, teeth, tongue,
// drawn on a pixel grid of 65 × 70 cells. Three states: open (waiting for
// the glass, with a glowing ring as the drop target), drink (wide open,
// the Schnaps flowing in) and gulp (shut, cheeks puffed). Pure drawing.

export type MouthState = 'open' | 'drink' | 'gulp';

interface Key {
  c: string;
  /** Group: an outline is drawn where a higher group meets a lower one. */
  g: string;
  r: number;
  /** Open to the picture's edge (no outline there). */
  openEdge?: boolean;
  noOutline?: boolean;
}

const LINE = '#0b0c0e';

const KEYS: Record<string, Key> = {
  s: { c: '#e9b78c', g: 'skin', r: 1, openEdge: true },
  S: { c: '#c98a62', g: 'skin', r: 1, openEdge: true },
  b: { c: '#e59a7c', g: 'skin', r: 1, openEdge: true },
  n: { c: '#f2c39c', g: 'nose', r: 2 },
  N: { c: '#7a3b2a', g: 'nose', r: 2, noOutline: true },
  L: { c: '#bd3b32', g: 'lip', r: 3 },
  l: { c: '#e5776d', g: 'lip', r: 3 },
  h: { c: '#f6c3a8', g: 'lip', r: 3, noOutline: true },
  i: { c: '#3b1418', g: 'in', r: 0.5 },
  t: { c: '#f4efe4', g: 'teeth', r: 0.7 },
  T: { c: '#c9c3b8', g: 'teeth', r: 0.7, noOutline: true },
  g: { c: '#d98e9a', g: 'tongue', r: 0.6 },
  G: { c: '#b85868', g: 'tongue', r: 0.6, noOutline: true },
  a: { c: '#e8a417', g: 'drink', r: 0.65 },
  A: { c: '#f6e7b0', g: 'drink', r: 0.65, noOutline: true },
  k: { c: LINE, g: 'ink', r: 9, noOutline: true },
  m: { c: '#8fc7a8', g: 'ring', r: 4 },
  M: { c: '#d6f0e0', g: 'ring', r: 4, noOutline: true },
};

export const MOUTH_COLS = 65;
export const MOUTH_ROWS = 70;
/** Centre of the lips in grid cells. */
export const MOUTH_LIPS: [number, number] = [32.5, 36];

/** The key of each cell of the mouth in a state. */
function cellAt(state: MouthState, glow: boolean, c: number, r: number): string | null {
  const cx = 32;
  const lipC = 36;
  const lip = state === 'drink' ? { rx: 24, ry: 16 } : state === 'gulp' ? { rx: 19, ry: 8 } : { rx: 22, ry: 13 };
  const inner = state === 'drink' ? { rx: 18, ry: 11, cy: 37 } : state === 'gulp' ? null : { rx: 16, ry: 8, cy: 37 };
  const puff = state === 'gulp';
  const u = c + 0.5;
  const v = r + 0.5;
  const e = (x0: number, y0: number, rx: number, ry: number): number => ((u - x0) / rx) ** 2 + ((v - y0) / ry) ** 2;
  const dLip = e(cx + 0.5, lipC, lip.rx, lip.ry);
  // The glowing ring around the lips: the drop target.
  if (glow && dLip > 1.28 && dLip < 1.62) return (c + r) % 3 === 0 ? 'M' : 'm';
  const notch = Math.abs(u - cx - 0.5) < 2 && v < lipC - lip.ry + 2.2;
  if (dLip <= 1 && !notch) {
    if (inner) {
      const dIn = e(cx + 0.5, inner.cy, inner.rx, inner.ry);
      if (dIn <= 1) {
        const top = inner.cy - inner.ry;
        if (state === 'drink' && v > inner.cy + 2) return (c + r) % 5 === 0 ? 'A' : 'a';
        if (v < top + 3.2) return (c - cx) % 4 === 0 ? 'T' : 't';
        if (e(cx + 0.5, inner.cy + inner.ry - 3, inner.rx * 0.55, 3.5) <= 1) return v > inner.cy + inner.ry - 2 ? 'G' : 'g';
        return 'i';
      }
    }
    if (!inner && Math.abs(v - lipC) < 0.6 && Math.abs(u - cx - 0.5) < lip.rx - 1) return 'k';
    if (v < lipC) return 'L';
    if (Math.abs(u - (cx - 6)) < 4 && v > lipC + lip.ry * 0.45 && v < lipC + lip.ry * 0.7) return 'h';
    return 'l';
  }
  // The corners of a satisfied gulp.
  if (puff) {
    const nearCorner = (x: number): boolean => Math.abs(u - x) < 1 && Math.abs(v - (lipC - 1.5)) < 1;
    if (nearCorner(cx + 0.5 - lip.rx - 1) || nearCorner(cx + 0.5 + lip.rx + 1)) return 'k';
  }
  const dNose = e(cx + 0.5, 12, 8, 5.5);
  if (dNose <= 1) {
    if ((Math.abs(u - (cx - 2.5)) < 1.4 || Math.abs(u - (cx + 3.5)) < 1.4) && Math.abs(v - 14.5) < 1) return 'N';
    return 'n';
  }
  // The lower face fills the width; its top edge curves down to the sides.
  const top = 9 + 13 * ((u - cx - 0.5) / 32.5) ** 2;
  if (v >= top) {
    const bl = puff ? { rx: 9, ry: 6 } : { rx: 5, ry: 3.5 };
    if (e(10, 31, bl.rx, bl.ry) <= 1 || e(55, 31, bl.rx, bl.ry) <= 1) return 'b';
    if (puff) {
      const dl = e(8, 33, 15, 14);
      const dr = e(57, 33, 15, 14);
      if ((dl > 0.82 && dl <= 1 && u > 8) || (dr > 0.82 && dr <= 1 && u < 57)) return 'S';
    }
    if (v < top + 2.2 || u < 2 || u > 63) return 'S';
    if (v > 58 && (c * 7 + r * 3) % 11 === 0) return 'S';
    return 's';
  }
  return null;
}

const cache = new Map<string, string>();

/** The mouth as SVG paths in grid units (viewBox 0 0 65 70). */
export function mouthPaths(state: MouthState, glow = false): string {
  const id = `${state}:${glow ? 1 : 0}`;
  const hit = cache.get(id);
  if (hit !== undefined) return hit;
  const grid: (string | null)[][] = [];
  for (let r = 0; r < MOUTH_ROWS; r++) {
    const row: (string | null)[] = [];
    for (let c = 0; c < MOUTH_COLS; c++) row.push(cellAt(state, glow, c, r));
    grid.push(row);
  }
  const at = (c: number, r: number): string | null => (r < 0 || r >= MOUTH_ROWS || c < 0 || c >= MOUTH_COLS ? null : (grid[r]?.[c] ?? null));
  const runs = new Map<string, string[]>();
  for (let r = 0; r < MOUTH_ROWS; r++) {
    for (let c = 0; c < MOUTH_COLS; c++) {
      const k = grid[r]?.[c];
      if (!k) continue;
      const key = KEYS[k];
      if (!key) continue;
      let edge = false;
      if (!key.noOutline) {
        for (const [dc, dr] of [
          [1, 0],
          [-1, 0],
          [0, 1],
          [0, -1],
        ] as const) {
          const n = at(c + dc, r + dr);
          if (!n) {
            const inGrid = r + dr >= 0 && r + dr < MOUTH_ROWS && c + dc >= 0 && c + dc < MOUTH_COLS;
            if (!key.openEdge || inGrid) edge = true;
            break;
          }
          const other = KEYS[n];
          if (other && other.g !== key.g && key.r > other.r) {
            edge = true;
            break;
          }
        }
      }
      const colour = edge ? LINE : key.c;
      let list = runs.get(colour);
      if (!list) runs.set(colour, (list = []));
      list.push(`M${c} ${r}h1v1h-1z`);
    }
  }
  let out = '';
  for (const [colour, d] of runs) out += `<path fill="${colour}" d="${d.join('')}"/>`;
  cache.set(id, out);
  return out;
}

/** The mouth as a whole SVG element, scaled by CSS. */
export function mouthSvg(state: MouthState, glow = false): string {
  return `<svg class="mouth-svg" viewBox="0 0 ${MOUTH_COLS} ${MOUTH_ROWS}" preserveAspectRatio="xMidYMax meet" shape-rendering="crispEdges" aria-hidden="true">${mouthPaths(state, glow)}</svg>`;
}

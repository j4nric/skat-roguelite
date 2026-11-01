// Card backs, a small registry of designs (a cosmetic display setting, no
// game logic). The default is "Distel": our own pixel drawing after the
// traditional Altenburger thistle back (ochre rim, dark blue frame, a cross
// band of black and white triangles, four panels of thistles, cornflower
// fans and golden scrolls on a cream ground, the lower panels mirrored).
// No external asset is used; see web/LICENSES.md.
//
// Each design comes in two sizes: the full picture and a simplified one for
// tiny backs (the opponents' fans, about 10 px wide), where fine detail would
// only turn into noise. Both are <symbol>s in the card sprite (./art.ts),
// in the card's 120 × 180 box.

/** The available back designs. Only one so far; more will be unlocked later. */
export type BackDesign = 'distel';

export const DEFAULT_BACK: BackDesign = 'distel';

/** Size variant of a back: the full picture or the simplified one. */
export type BackSize = 'full' | 'small';

interface BackArt {
  full: () => string;
  small: () => string;
  /** Extra symbols the pictures use. */
  parts?: () => string;
}

// ------------------------------------------------------------ Distel colours

const OCHRE = '#e6a93e';
const OCHRE_DARK = '#b97a1d';
const NAVY = '#1b2a4a';
const CREAM = '#f3e6c2';
const CREAM_LINE = '#e6d4a4';
const GOLD = '#dda23a';
const GOLD_DARK = '#a8701a';
const BLUE = '#2f62c4';
const BLUE_DARK = '#1c3a86';
const THISTLE = '#9a6a35';
const THISTLE_DARK = '#563619';
const INK = '#3b2d1f';
const WHITE = '#fbf6e8';
const BLACK = '#141414';

// ------------------------------------------------------------- pixel grids

/** Colours of the characters in a motif grid; "." is the ground. */
const DISTEL_INK: Record<string, string> = {
  o: GOLD,
  O: GOLD_DARK,
  b: BLUE,
  B: BLUE_DARK,
  t: THISTLE,
  T: THISTLE_DARK,
  k: INK,
};

// The panel motif is rasterised from a few simple shapes (thistle heads,
// petal fans, golden strokes) into a pixel grid once, when the sprite is
// built. Shapes are given for the left half of the upper panel in panel
// units (42 × 70, one unit per pixel); the right half is its mirror image.

type Cell = string;
type Shape = (x: number, y: number) => Cell | null;

/** Distance of p to the segment a–b, with the position t (0..1) along it. */
function segment(px: number, py: number, ax: number, ay: number, bx: number, by: number): { d: number; t: number; side: number } {
  const vx = bx - ax;
  const vy = by - ay;
  const len2 = vx * vx + vy * vy || 1;
  const t = Math.max(0, Math.min(1, ((px - ax) * vx + (py - ay) * vy) / len2));
  const dx = px - (ax + t * vx);
  const dy = py - (ay + t * vy);
  return { d: Math.hypot(dx, dy), t, side: vx * (py - ay) - vy * (px - ax) };
}

/** Points along a quadratic Bézier curve. */
function curve(ax: number, ay: number, cx: number, cy: number, bx: number, by: number, n = 16): [number, number][] {
  const pts: [number, number][] = [];
  for (let i = 0; i <= n; i++) {
    const t = i / n;
    const u = 1 - t;
    pts.push([u * u * ax + 2 * u * t * cx + t * t * bx, u * u * ay + 2 * u * t * cy + t * t * by]);
  }
  return pts;
}

/**
 * A golden stroke along a polyline, `w0` wide at the start and `w1` at the
 * end; the right-hand side of its direction is shaded.
 */
function stroke(pts: [number, number][], w0: number, w1: number, light = 'o', dark = 'O'): Shape {
  const total = pts.length - 1;
  return (x, y) => {
    let best: { d: number; t: number; side: number } | null = null;
    for (let i = 0; i < total; i++) {
      const a = pts[i] ?? [0, 0];
      const b = pts[i + 1] ?? [0, 0];
      const s = segment(x, y, a[0], a[1], b[0], b[1]);
      if (!best || s.d < best.d) best = { d: s.d, t: (i + s.t) / total, side: s.side };
    }
    if (!best) return null;
    const w = w0 + (w1 - w0) * best.t;
    if (best.d > w / 2) return null;
    return best.side > 0 && best.d > w / 6 ? dark : light;
  };
}

/** A thistle head: an egg pointed at the top, cross-hatched in two browns. */
function thistle(cx: number, cy: number, rx: number, ry: number): Shape {
  return (x, y) => {
    const v = (y - cy) / ry;
    if (v < -1 || v > 1) return null;
    const taper = v < 0 ? 0.45 + 0.55 * (1 + v) : 1;
    const half = rx * taper * Math.sqrt(1 - v * v);
    if (Math.abs(x - cx) > half) return null;
    const i = Math.floor(x);
    const j = Math.floor(y);
    return (i + j) % 3 === 0 || (((i - j) % 3) + 3) % 3 === 0 ? 'T' : 't';
  };
}

/** A filled ellipse; its lower part in `dark` if given. */
function blob(cx: number, cy: number, rx: number, ry: number, light: Cell, dark?: Cell): Shape {
  return (x, y) => {
    const dx = (x - cx) / rx;
    const dy = (y - cy) / ry;
    if (dx * dx + dy * dy > 1) return null;
    return dark && dy > 0.3 ? dark : light;
  };
}

/** A ring (the curl of a scroll). */
function ring(cx: number, cy: number, r0: number, r1: number): Shape {
  return (x, y) => {
    const d = Math.hypot(x - cx, y - cy);
    return d >= r0 && d <= r1 ? (y > cy ? 'O' : 'o') : null;
  };
}

/** A fan of cornflower petals from (ox, oy), one per angle (degrees, 0 = right, 90 = down). */
function fan(ox: number, oy: number, angles: number[], len: number, w0: number, w1: number): Shape {
  const petals = angles.map((a) => {
    const r = (a * Math.PI) / 180;
    return [ox + Math.cos(r) * len, oy + Math.sin(r) * len] as const;
  });
  return (x, y) => {
    for (const [bx, by] of petals) {
      const s = segment(x, y, ox, oy, bx, by);
      const w = w0 + (w1 - w0) * s.t;
      if (s.d <= w / 2 && s.t < 1) return s.d < w / 6 && s.t > 0.25 ? 'B' : 'b';
    }
    return null;
  };
}

/** The upper panel's shapes, back to front. */
const DISTEL_SHAPES: Shape[] = [
  // Golden palmette on top: a middle leaf and curling leaves to the sides.
  blob(21, 7, 2.2, 6.5, 'o', 'O'),
  stroke([...curve(20, 14, 15, 2, 8, 4), ...curve(8, 4, 5, 6, 8, 9)], 2.6, 1.4),
  stroke([...curve(19, 15, 9, 9, 3, 4), ...curve(3, 4, 0, 9, 4, 10)], 2.4, 1.4),
  stroke(curve(19, 16, 10, 17, 3, 15), 2.2, 1.2),
  // The stem down the middle.
  stroke([[21, 12], [21, 36]], 2.6, 2.6),
  // Stems of the two upper thistles.
  stroke(curve(12, 27, 13, 34, 20, 35), 1.8, 1.8, 'O', 'O'),
  // Cornflower fans beside the upper thistles, pointing out and down.
  fan(9, 27, [122, 146, 170, 194], 10, 1.2, 3.6),
  thistle(12, 21, 3.8, 5.8),
  blob(12, 27.5, 3, 1.4, 'o', 'O'),
  // Golden leaves round the big thistle, like a lyre.
  stroke([...curve(18, 51, 10, 50, 11, 40), ...curve(11, 40, 11, 36, 8, 37)], 2.2, 1.2),
  // Golden scrolls at the bottom corners and a small leaf in the middle.
  stroke([...curve(2, 56, 1, 67, 9, 67), ...curve(9, 67, 13, 66, 12, 63)], 2.4, 1.4),
  ring(12, 62.5, 0.8, 2),
  blob(21, 67, 2, 2.6, 'o', 'O'),
  // The big thistle in the middle, over a big cornflower fan.
  fan(21, 52, [90, 114, 138, 162], 13, 1, 3.6),
  thistle(21, 41, 6.4, 8.4),
  blob(21, 51, 5, 1.8, 'o', 'O'),
  // Small thistle buds at the edge with blue leaves.
  fan(4, 48, [100, 130], 6, 1, 2.6),
  thistle(4, 42, 2.4, 4.4),
  blob(4, 47.2, 2, 1, 'o', 'O'),
];

const PANEL_COLS = 42;
const PANEL_ROWS = 70;

/** Rasterise the shapes into rows of cells, mirrored about the middle. */
function rasterise(shapes: Shape[], cols: number, rows: number): string[] {
  const out: string[] = [];
  for (let j = 0; j < rows; j++) {
    let row = '';
    for (let i = 0; i < cols; i++) {
      const x = Math.min(i, cols - 1 - i) + 0.5;
      let cell = '.';
      for (const shape of shapes) cell = shape(x, j + 0.5) ?? cell;
      row += cell;
    }
    out.push(row);
  }
  return out;
}

/** Rectangles of one colour as a single path (`u` units per cell). */
function runs(rows: string[], ch: string, x: number, y: number, u: number): string {
  let d = '';
  rows.forEach((row, j) => {
    let i = 0;
    while (i < row.length) {
      if (row[i] !== ch) {
        i++;
        continue;
      }
      let k = i;
      while (k < row.length && row[k] === ch) k++;
      d += `M${x + i * u} ${y + j * u}h${(k - i) * u}v${u}h${-(k - i) * u}z`;
      i = k;
    }
  });
  return d;
}

/**
 * A motif grid at (x, y), one unit per cell: every coloured cell, and a dark
 * outline in the ground cells next to the motif, like the engraved lines of
 * the old printed backs.
 */
function motif(rows: string[], x: number, y: number, colours: Record<string, string>): string {
  const at = (i: number, j: number): string => rows[j]?.[i] ?? '.';
  const inked = rows.map((row, j) =>
    row
      .split('')
      .map((c, i) => (c === '.' && (at(i - 1, j) !== '.' || at(i + 1, j) !== '.' || at(i, j - 1) !== '.' || at(i, j + 1) !== '.') ? 'k' : c))
      .join(''),
  );
  return Object.entries(colours)
    .map(([ch, fill]) => {
      const d = runs(inked, ch, x, y, 1);
      return d ? `<path d="${d}" fill="${fill}"/>` : '';
    })
    .join('');
}

/** Cream ground with fine horizontal lines, as on the printed back. */
function ground(x: number, y: number, w: number, h: number): string {
  let d = '';
  for (let j = y + 1; j < y + h; j += 2) d += `M${x} ${j}h${w}v1h${-w}z`;
  return `<rect x="${x}" y="${y}" width="${w}" height="${h}" fill="${CREAM}"/><path d="${d}" fill="${CREAM_LINE}"/>`;
}

/**
 * One stretch of the cross band: cells of hourglass triangles, black and
 * white in turn, on ochre, divided by thin dark lines. `vertical` runs the
 * cells top to bottom, otherwise left to right.
 */
function band(x: number, y: number, len: number, width: number, cell: number, vertical: boolean): string {
  const tris: Record<'w' | 'k', string> = { w: '', k: '' };
  let lines = '';
  const n = Math.round(len / cell);
  for (let c = 0; c < n; c++) {
    const a = c * cell;
    const first = c % 2 === 0 ? 'w' : 'k';
    const second = first === 'w' ? 'k' : 'w';
    if (vertical) {
      const top = y + a;
      const mid = top + cell / 2;
      const cx = x + width / 2;
      tris[first] += `M${x} ${top}H${x + width}L${cx} ${mid}z`;
      tris[second] += `M${x} ${top + cell}H${x + width}L${cx} ${mid}z`;
      if (c > 0) lines += `M${x} ${top}h${width}v1h${-width}z`;
    } else {
      const left = x + a;
      const mid = left + cell / 2;
      const cy = y + width / 2;
      tris[first] += `M${left} ${y}V${y + width}L${mid} ${cy}z`;
      tris[second] += `M${left + cell} ${y}V${y + width}L${mid} ${cy}z`;
      if (c > 0) lines += `M${left} ${y}h1v${width}h-1z`;
    }
  }
  const w = vertical ? width : len;
  const h = vertical ? len : width;
  return `<rect x="${x}" y="${y}" width="${w}" height="${h}" fill="${OCHRE}"/><path d="${tris.w}" fill="${WHITE}"/><path d="${tris.k}" fill="${BLACK}"/><path d="${lines}" fill="${NAVY}"/>`;
}

/** The square where the bands cross: four triangles around a small diamond. */
function crossing(x: number, y: number, s: number): string {
  const m = s / 2;
  return `<rect x="${x}" y="${y}" width="${s}" height="${s}" fill="${OCHRE}"/>
<path d="M${x} ${y}H${x + s}L${x + m} ${y + m}zM${x} ${y + s}H${x + s}L${x + m} ${y + m}z" fill="${BLACK}"/>
<path d="M${x} ${y}V${y + s}L${x + m} ${y + m}zM${x + s} ${y}V${y + s}L${x + m} ${y + m}z" fill="${WHITE}"/>
<path d="M${x + m - 2} ${y + m - 2}h4v4h-4z" fill="${OCHRE}"/>`;
}

/**
 * The rim: ochre with a dark blue frame line whose corners are cut, so the
 * frame reads rounded like the printed card's.
 */
function rim(): string {
  return `<rect width="120" height="180" fill="${OCHRE}"/>
<path d="M2 2h116v1h-116zM2 177h116v1h-116zM2 3h1v174h-1zM117 3h1v174h-1z" fill="${OCHRE_DARK}" opacity=".45"/>
<rect x="8" y="10" width="104" height="160" fill="${NAVY}"/>
<path d="M8 10h2v2h-2zM110 10h2v2h-2zM8 168h2v2h-2zM110 168h2v2h-2z" fill="${OCHRE}"/>`;
}

// Layout of the full Distel back in the 120 × 180 box: ochre rim (8 at the
// sides, 10 top and bottom), a 2 unit navy frame, panels of 42 × 70, the
// band 12 wide with 2 units of navy on each side.
const PANEL_W = 42;
const PANEL_H = 70;
const BAND = 12;
const LEFT = 10;
const TOP = 12;
const RIGHT = LEFT + PANEL_W + 2 + BAND + 2;
const BOTTOM = TOP + PANEL_H + 2 + BAND + 2;

/** One upper panel of the Distel back, as its own symbol (used four times). */
function distelPanel(): string {
  const rows = rasterise(DISTEL_SHAPES, PANEL_COLS, PANEL_ROWS);
  return `${ground(0, 0, PANEL_W, PANEL_H)}${motif(rows, 0, 0, DISTEL_INK)}`;
}

function distelFull(): string {
  const panel = (x: number, y: number, flip: boolean): string =>
    `<use href="#bk-distel-panel" x="${x}" y="${flip ? -y - PANEL_H : y}" width="${PANEL_W}" height="${PANEL_H}"${flip ? ' transform="scale(1 -1)"' : ''}/>`;
  const vx = LEFT + PANEL_W + 2;
  const hy = TOP + PANEL_H + 2;
  return `${rim()}
${panel(LEFT, TOP, false)}${panel(RIGHT, TOP, false)}${panel(LEFT, BOTTOM, true)}${panel(RIGHT, BOTTOM, true)}
${band(vx, TOP, PANEL_H + 2, BAND, 12, true)}${band(vx, hy + BAND, PANEL_H + 2, BAND, 12, true)}
${band(LEFT, hy, PANEL_W + 2, BAND, 11, false)}${band(vx + BAND, hy, PANEL_W + 2, BAND, 11, false)}
${crossing(vx, hy, BAND)}`;
}

/**
 * The Distel back for tiny sizes: rim, frame, the cross band and in each
 * panel just a brown thistle over a blue fan.
 */
function distelSmall(): string {
  const panel = (x: number, y: number, up: boolean): string => {
    const t = up ? y + 14 : y + 30;
    const f = up ? y + 40 : y + 12;
    return `<rect x="${x}" y="${y}" width="36" height="62" fill="${CREAM}"/>
<rect x="${x + 11}" y="${t}" width="14" height="18" fill="${THISTLE_DARK}"/>
<rect x="${x + 5}" y="${f}" width="26" height="10" fill="${BLUE}"/>`;
  };
  return `<rect width="120" height="180" fill="${OCHRE}"/>
<rect x="12" y="14" width="96" height="152" fill="${NAVY}"/>
${panel(16, 18, true)}${panel(68, 18, true)}${panel(16, 100, false)}${panel(68, 100, false)}
<path d="M56 18h8v18h-8zM56 54h8v18h-8zM56 108h8v18h-8zM56 144h8v18h-8zM16 86h18v8h-18zM86 86h18v8h-18z" fill="${WHITE}"/>
<path d="M56 36h8v18h-8zM56 126h8v18h-8zM34 86h18v8h-18zM68 86h18v8h-18z" fill="${BLACK}"/>
<rect x="56" y="84" width="8" height="12" fill="${OCHRE}"/>`;
}

const BACKS: Record<BackDesign, BackArt> = {
  distel: {
    full: distelFull,
    small: distelSmall,
    parts: () => `<symbol id="bk-distel-panel" viewBox="0 0 ${PANEL_W} ${PANEL_H}" shape-rendering="crispEdges">${distelPanel()}</symbol>`,
  },
};

/** Symbol id of a back design in the sprite. */
export function backId(design: BackDesign = DEFAULT_BACK, size: BackSize = 'full'): string {
  return `bk-${design}${size === 'small' ? '-s' : ''}`;
}

/** The <symbol>s of every back design, for the card sprite. */
export function backSymbols(): string {
  return (Object.keys(BACKS) as BackDesign[])
    .map((design) => {
      const art = BACKS[design];
      return `${art.parts?.() ?? ''}<symbol id="${backId(design, 'full')}" viewBox="0 0 120 180" shape-rendering="crispEdges">${art.full()}</symbol><symbol id="${backId(design, 'small')}" viewBox="0 0 120 180" shape-rendering="crispEdges">${art.small()}</symbol>`;
    })
    .join('');
}

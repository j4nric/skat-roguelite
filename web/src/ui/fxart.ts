// The art of the card modifications, "Effekte v3" (approved board of
// 02.11.2026, tools/cardart/fx3.py): pixels on the card's art grid
// (33 × 49.5 art pixels, one art pixel = 2 × 2 CSS px at 66 × 99).
//
//   frame (ring 1, 2)        what: Augen gold double, Trumpf purple dashed,
//                            Erster Stich rust double, Letzter Stich petrol
//                            dashed, Gutschein a stamp margin with a tear line
//   mark top right           when: finish flag (Letzter Stich), play sign
//                            (Erster Stich); none = always
//   band bottom middle       how much: a colour band of fixed height with
//                            white glyphs ("+4", "×2", chevrons, a coin …)
//   curses                   anthracite, no corner sign: Ramponiert tears,
//                            Fleckig stains, Durchsichtig fades the card
//   Untot, Wandelkarte       change the picture only (./art.ts), no frame,
//                            so a special card carries one normal frame
//
// The same pixels as the generator, drawn at run time because frames and
// bands combine per card and the band text is the card's value. Pure
// drawing: which modification a card carries and its value come from the
// core.

import { ART_GLYPHS, ART_H, ART_SIGNS, ART_W } from './pixelcards.gen';

export const W = ART_W;
export const H = ART_H + 0.5;
/** Full art rows; the half row at the bottom stays paper. */
const ROWS = Math.floor(H);

/** Pixels on the art grid as "x,y" keys. */
export type Cells = Set<string>;

const key = (x: number, y: number): string => `${x},${y}`;
const xy = (c: string): [number, number] => {
  const [x = 0, y = 0] = c.split(',').map(Number);
  return [x, y];
};
function filter(cells: Cells, f: (x: number, y: number) => boolean): Cells {
  return new Set([...cells].filter((c) => f(...xy(c))));
}
function minus(a: Cells, b: Cells): Cells {
  return new Set([...a].filter((c) => !b.has(c)));
}
function union(...sets: Cells[]): Cells {
  const out: Cells = new Set();
  for (const s of sets) for (const c of s) out.add(c);
  return out;
}

/** Rectangles on the art grid, "x y w h" separated by commas. */
function rects(list: string): Cells {
  const out: Cells = new Set();
  for (const r of list.split(',')) {
    const [x = 0, y = 0, w = 0, h = 0] = r.trim().split(/\s+/).map(Number);
    for (let j = 0; j < h; j++) for (let i = 0; i < w; i++) out.add(key(x + i, y + j));
  }
  return out;
}

/** A pixel icon from rows of "#" and ".", top left at (x, y). */
function shape(rows: string[], x: number, y: number): Cells {
  const out: Cells = new Set();
  rows.forEach((row, j) => [...row].forEach((ch, i) => ch === '#' && out.add(key(x + i, y + j))));
  return out;
}

/** The one-pixel neighbourhood of the cells, the cells included. */
function halo(cells: Cells): Cells {
  const out: Cells = new Set();
  for (const c of cells) {
    const [x, y] = xy(c);
    for (let dy = -1; dy <= 1; dy++) for (let dx = -1; dx <= 1; dx++) out.add(key(x + dx, y + dy));
  }
  return out;
}

/** A fill as SVG attribute; CSS variables need the style property. */
export function fillOf(fill: string): string {
  return fill.startsWith('var(') ? `style="fill:${fill}"` : `fill="${fill}"`;
}

/** The cells on the card as one path, merged into runs per row. */
export function draw(cells: Cells, fill: string, opacity?: number): string {
  const rows = new Map<number, number[]>();
  for (const c of cells) {
    const [x, y] = xy(c);
    if (x < 0 || x >= W || y < 0 || y >= ROWS) continue;
    rows.set(y, [...(rows.get(y) ?? []), x]);
  }
  let d = '';
  for (const [y, xs] of [...rows].sort((a, b) => a[0] - b[0])) {
    xs.sort((a, b) => a - b);
    for (let i = 0; i < xs.length; ) {
      let k = i;
      while (k + 1 < xs.length && xs[k + 1] === (xs[k] ?? 0) + 1) k++;
      const n = k - i + 1;
      d += `M${xs[i]} ${y}h${n}v1h-${n}z`;
      i = k + 1;
    }
  }
  const op = opacity === undefined ? '' : ` opacity="${opacity}"`;
  return d ? `<path d="${d}" ${fillOf(fill)}${op}/>` : '';
}

// ---------------------------------------------------------------- rings

/** Corner index notches: rings 1 to 3 leave them free (fx2.py). */
const NOTCH_TL: [number, number] = [7, 13];
const NOTCH_BR: [number, number] = [26, 36];

/** The k-th ring of pixels from the card's edge (fx2.py `ring`). */
export function ring(k: number, notch = true): Cells {
  const out: Cells = new Set();
  for (let x = k; x < W - k; x++) {
    out.add(key(x, k));
    out.add(key(x, ROWS - 1 - k));
  }
  for (let y = k; y < ROWS - k; y++) {
    out.add(key(k, y));
    out.add(key(W - 1 - k, y));
  }
  return notch ? filter(out, (x, y) => !(x < NOTCH_TL[0] && y < NOTCH_TL[1]) && !(x >= NOTCH_BR[0] && y >= NOTCH_BR[1])) : out;
}

// --------------------------------------------------------------- colours

export const GOLD = '#d98e0b';
const GOLD_L = '#ffe08a';
export const PURPLE = '#6a3fa8';
const TEAL = '#16707a';
const PLAY = '#d2552a';
const PLAY_L = '#f6b48e';
export const ANTH = '#33363b';
const ANTH_L = '#9a9ea4';
export const STAMP = '#2f7d6a';
const STAMP_L = '#bfe3d6';
/** The band of a voucher being redeemed (board "Einlösen", step 2). */
const STAMP_REDEEM = '#3fae90';
const WHITE = '#ffffff';
const ZOMBIE = '#5e7a3c';
// Zinken v2 (board Zinken-v2, 03.11.2026).
const LEAD = '#5f6870';
const LEAD_L = '#b9c0c7';
const LEAD_D = '#353b42';
const GILT = '#f7c21a';
const GILT_D = '#a87800';
const GILT_BAND = '#7a5210';
const XH = '#c0263a';
const XH_L = '#f4a3ae';
const TAILOR = '#3d78b0';
const TAILOR_L = '#cfe3f3';
/** The violet copying pencil of "Winkel" (Winkel v2, ink of WK-B). */
export const MARKER = '#5b2a86';
/** "Doppelgänger": its violet mirror half and the seam. */
export const MIRROR = '#6a58a0';

/** The band's dark bottom row: each channel × 0.62 (fx3.py `shade`). */
function shade(c: string): string {
  const ch = [1, 3, 5].map((i) => Math.floor(parseInt(c.slice(i, i + 2), 16) * 0.62));
  return `#${ch.map((v) => v.toString(16).padStart(2, '0')).join('')}`;
}

// ---------------------------------------------------------------- glyphs

/** Glyphs of the band, 5 high (fx.py DIGITS and fx3.py G). */
const G: Record<string, string[]> = {
  '0': ['###', '#.#', '#.#', '#.#', '###'],
  '1': ['#', '#', '#', '#', '#'],
  '2': ['###', '..#', '###', '#..', '###'],
  '3': ['###', '..#', '###', '..#', '###'],
  '4': ['#.#', '#.#', '###', '..#', '..#'],
  '5': ['###', '#..', '###', '..#', '###'],
  '6': ['###', '#..', '###', '#.#', '###'],
  '7': ['###', '..#', '.#.', '.#.', '.#.'],
  '8': ['###', '#.#', '###', '#.#', '###'],
  '9': ['###', '#.#', '###', '..#', '###'],
  '+': ['...', '.#.', '###', '.#.', '...'],
  '−': ['...', '...', '###', '...', '...'],
  '×': ['...', '#.#', '.#.', '#.#', '...'],
  coin: ['.###.', '##.##', '#.#.#', '##.##', '.###.'],
  skull: ['.###.', '#####', '#.#.#', '#####', '.#.#.'],
  eye: ['..###..', '.#...#.', '#..#..#', '.#...#.', '..###..'],
  rang1: ['...', '.#.', '#.#', '...', '...'],
  rang2: ['.#.', '#.#', '...', '.#.', '#.#'],
  rang3: ['...........', '.#...#...#.', '#.#.#.#.#.#', '...........', '...........'],
  plumb: ['..#..', '.#.#.', '.###.', '#####', '#####'],
  xhair: ['.###.', '#.#.#', '##.##', '#.#.#', '.###.'],
  chalk: ['.....', '#####', '.###.', '..#..', '.....'],
  scissor: ['#...#', '.#.#.', '..#..', '##.##', '##.##'],
  daus: ['.#.', '#.#', '###', '#.#', '#.#'],
  mirror: ['##.##', '#...#', '#...#', '#...#', '##.##'],
  // Werte-Icons (user decision 03.11.2026): G-B "Grundstein" for the
  // Grundwert; the Stufen (Gewinnstufen, code `levels`) a plain staircase of
  // three steps (user decision 04.11.2026, replaced the M-D "Steilpfeil").
  base: ['..###..', '..###..', '.......', '###.###', '###.###'],
  levels: ['....##', '....##', '..####', '..####', '######'],
  // Faktor (ADR 0018, provisional until the user picks one): a "×" with a
  // superscript caret, "× hoch": the whole Spielwert multiplied.
  factor: ['....#.', '...#.#', '#.#...', '.#....', '#.#...'],
};

/** One item of a band: a number text ("+4", "−2", "×2") or a named glyph. */
export type BandItem = string;

/** The coloured cells of one band item at (x, y) and its width. */
function glyphCells(item: BandItem, x: number, y: number): { parts: [Cells, string | null][]; w: number } {
  if (item === 'wild') {
    // A rainbow diamond on a white tile with a black edge (reads on any band colour).
    const inner = rects(`${x} ${y} 5 5`);
    const parts: [Cells, string | null][] = [
      [minus(rects(`${x - 1} ${y - 1} 7 7`), inner), '#1c1a1a'],
      [inner, WHITE],
    ];
    const dia: [string, string][] = [
      ['..#..', '#e0281e'],
      ['.###.', '#f5d52a'],
      ['#####', '#2e9a3c'],
      ['.###.', '#2a63c4'],
      ['..#..', '#8b4a1c'],
    ];
    dia.forEach(([row, c], j) => parts.push([shape([row], x, y + j), c]));
    return { parts, w: 5 };
  }
  if (item === 'pips') {
    // Augen (A-C "Würfelauge", 03.11.2026): a white die face with five dark pips.
    const face = shape(['#####', '#.#.#', '##.##', '#.#.#', '#####'], x, y);
    const pips = shape(['.....', '.#.#.', '..#..', '.#.#.', '.....'], x, y);
    return { parts: [[face, WHITE], [pips, '#2b2f33']], w: 5 };
  }
  if (item === 'coin') return { parts: [[shape(G.coin ?? [], x, y), '#f7c21a'], [new Set([key(x + 2, y + 2)]), '#8a5a0a']], w: 5 };
  const named = G[item];
  if (named && item.length > 1) return { parts: [[shape(named, x, y), null]], w: named[0]?.length ?? 0 };
  const cells: Cells = new Set();
  let w = 0;
  for (const ch of item) {
    const g = G[ch];
    if (!g) continue;
    for (const c of shape(g, x + w, y)) cells.add(c);
    w += (g[0]?.length ?? 0) + 1;
  }
  return { parts: [[cells, null]], w: Math.max(0, w - 1) };
}

/**
 * The band at the bottom middle: a colour band of fixed height with
 * stepped ends and a paper halo, glyphs in white (fx3.py `band`).
 */
export function band(items: BandItem[], col: string, paper: string, y = 41): string {
  const widths = items.map((it) => glyphCells(it, 0, 0).w);
  const cw = widths.reduce((a, b) => a + b, 0) + 2 * (items.length - 1);
  // Every band is at least as wide as "+3".
  const pad = Math.max(0, 7 - cw);
  let x0 = Math.max(5, Math.min(Math.floor((W - cw - pad) / 2), 23 - cw - pad));
  const bx0 = x0 - 2;
  const bx1 = x0 + cw + pad + 1;
  x0 += Math.floor(pad / 2);
  const bw = bx1 - bx0 + 1;
  const body = rects(`${bx0} ${y} ${bw} 8, ${bx0 - 1} ${y + 1} 1 6, ${bx1 + 1} ${y + 1} 1 6`);
  let out = draw(minus(halo(body), body), paper) + draw(body, col) + draw(rects(`${bx0} ${y + 7} ${bw} 1`), shade(col));
  let x = x0;
  items.forEach((it, i) => {
    // Centred in the seven rows above the dark bottom row: one row free above and below.
    for (const [cells, c] of glyphCells(it, x, y + 1).parts) out += draw(cells, c ?? WHITE);
    x += (widths[i] ?? 0) + 2;
  });
  return out;
}

// ---------------------------------------------------------------- frames

/** A double frame: outer colour on ring 1, a light line on ring 2. */
function frameDouble(c: string, l: string): string {
  return draw(ring(1), c) + draw(ring(2), l);
}

/** A dashed line on ring 1, two on, two off. */
function frameDashed(col: string): string {
  return draw(
    filter(ring(1), (x, y) => Math.floor((x + y) / 2) % 2 === 0),
    col,
  );
}

/** Goldkante (GK-A): the outermost ring bright gold with glints, dark gold inside. */
function frameGilt(): string {
  const r0 = ring(0, false);
  return draw(r0, GILT) + draw(filter(r0, (x, y) => (x * 3 + y) % 7 === 0), WHITE) + draw(ring(1), GILT_D);
}

/** Schneiderkreide (SK-B): a light edge with a seam of short stitches. */
function frameSeam(): string {
  return draw(ring(1), TAILOR_L) + draw(filter(ring(2), (x, y) => (x + y) % 3 !== 0), TAILOR);
}

/** Gutschein: a light stamp margin with white perforation dots and a dashed tear line. */
function frameStamp(): string {
  const margin = union(ring(0, false), ring(1, false));
  const dots = filter(ring(0, false), (x, y) => (x + y) % 2 === 0);
  const tear = filter(ring(2), (x, y) => Math.floor((x + y) / 2) % 2 === 0);
  return draw(margin, STAMP_L) + draw(dots, WHITE) + draw(tear, STAMP);
}

/** Gutschein being redeemed: the margin torn off from the top right, a jagged edge. */
function frameStampTorn(paper: string): string {
  const all = rects(`0 0 ${W} ${ROWS}`);
  const cut = filter(all, (x, y) => x - y > 6 && (x <= 2 || x >= W - 3 || y <= 2));
  const jag = filter(all, (x, y) => x - y === 6 + (x % 2) && (x <= 3 || x >= W - 4 || y <= 3));
  return frameStamp() + draw(cut, paper) + draw(jag, '#9a8a70');
}

// ----------------------------------------------------------- marks (when)

const FINISH = ['#.#.#', '.#.#.', '#.#.#', '.#.#.'];
const PLAY_SIGN = ['#..', '##.', '###', '##.', '#..'];

/** A stepped corner top right with its sign (fx3.py `mark`). */
function mark(col: string, sign: string[], paper: string): string {
  let list = '';
  for (let y = 0; y < 10; y++) list += `${list ? ', ' : ''}${23 + y} ${y} ${W - 23 - y} 1`;
  const cells = rects(list);
  const x = (sign[0]?.length ?? 0) === 5 ? 28 : 29;
  return draw(minus(halo(cells), cells), paper) + draw(cells, col) + draw(shape(sign, x, 1), WHITE);
}

// ---------------------------------------------------------------- curses

/** Ramponiert: irregular tears of varied depth into all four edges (fx3.py `torn_deep`). */
function tornDeep(): string {
  const bites: Cells = new Set();
  const spec: [string, number, number, number][] = [
    ['l', 16, 3, 2],
    ['l', 29, 4, 3],
    ['r', 19, 2, 2],
    ['r', 30, 4, 2],
    ['t', 13, 3, 3],
    ['t', 22, 1, 2],
    ['b', 15, 4, 3],
    ['b', 8, 2, 2],
    ['l', 22, 1, 1],
  ];
  for (const [side, pos, depth, width] of spec) {
    for (let d = 0; d < depth; d++) {
      const w = Math.max(1, width - Math.floor(d / 2));
      for (let k = Math.floor(-w / 2); k < w - Math.floor(w / 2); k++) {
        const o = d % 2;
        if (side === 'l') bites.add(key(d, pos + k + o));
        else if (side === 'r') bites.add(key(W - 1 - d, pos + k - o));
        else if (side === 't') bites.add(key(pos + k + o, d));
        else bites.add(key(pos + k - o, ROWS - 1 - d));
      }
    }
  }
  const rims: Cells = new Set();
  for (const c of bites) {
    const [x, y] = xy(c);
    for (const [dx, dy] of [
      [1, 0],
      [-1, 0],
      [0, 1],
      [0, -1],
    ] as const) {
      const q = key(x + dx, y + dy);
      if (!bites.has(q) && x + dx >= 0 && x + dx < W && y + dy >= 0 && y + dy < ROWS) rims.add(q);
    }
  }
  return draw(bites, '#1b1e21') + draw(rims, '#8a7a60');
}

/** Fleckig: strong stains over the picture. */
function stains(): string {
  const blobs = union(rects('17 13 6 2, 16 15 8 3, 17 18 6 2'), rects('7 27 4 2, 6 29 6 2, 7 31 3 1'), rects('20 31 5 2, 19 33 7 3'));
  return draw(blobs, '#8a5a22', 0.82) + draw(rects('18 14 2 1, 8 28 1 1, 21 32 2 1'), '#5e3a12', 0.85);
}

/** Durchsichtig: the whole card fades. */
function veil(paper: string): string {
  return `<rect width="${W}" height="${H}" ${fillOf(paper)} opacity=".55"/>`;
}

// ------------------------------------------------------------ the designs

/** What the card shows for one normal modification. */
export interface ModDraw {
  /** Band items from the card's value text (e.g. "+4"), or fixed ones. */
  band: (text: string, value: number) => BandItem[];
  /** Colour of the band. */
  col: string;
  /** Frame, marks and curse art under the band. */
  art: (paper: string, redeeming: boolean) => string;
}

/** The design of each modification id (core/content/card_mods.ron). */
export const MOD_DRAW: Record<string, ModDraw> = {
  // Rand: always (no mark).
  points: { band: (t) => [t, 'pips'], col: GOLD, art: () => frameDouble(GOLD, GOLD_L) },
  // Winkel (Winkel v2): no band, hand sketches on the face and the index (winkelSketch, winkelScribble).
  rank: { band: () => [], col: MARKER, art: () => '' },
  lead: { band: () => ['plumb'], col: LEAD, art: () => frameDouble(LEAD_D, LEAD_L) },
  gilt_edge: { band: (t) => [t, 'coin'], col: GILT_BAND, art: () => frameGilt() },
  trump_bonus: { band: (t) => [t, 'base'], col: PURPLE, art: () => frameDashed(PURPLE) },
  // Moment: the mark top right says when.
  last_trick: { band: (t) => [t, 'levels'], col: TEAL, art: (p) => frameDashed(TEAL) + mark(TEAL, FINISH, p) },
  first_trick: { band: (t) => [t, 'pips'], col: PLAY, art: (p) => frameDouble(PLAY, PLAY_L) + mark(PLAY, PLAY_SIGN, p) },
  crosshair: { band: (t) => [t, 'levels'], col: XH, art: (p) => frameDouble(XH, XH_L) + mark(XH, G.xhair ?? [], p) },
  tailors_chalk: { band: (t) => [t, 'levels'], col: TAILOR, art: (p) => frameSeam() + mark(TAILOR, G.scissor ?? [], p) },
  // Gutschein: when discarded; the band shows the reward.
  voucher_coins: { band: (t) => ['coin', t], col: STAMP, art: (p, r) => (r ? frameStampTorn(p) : frameStamp()) },
  voucher_undead: { band: () => ['skull'], col: STAMP, art: (p, r) => (r ? frameStampTorn(p) : frameStamp()) },
  voucher_wild: { band: () => ['wild'], col: STAMP, art: (p, r) => (r ? frameStampTorn(p) : frameStamp()) },
  // Flüche: anthracite, recognisable on the card itself.
  transparent: { band: () => ['eye'], col: ANTH, art: (p) => veil(p) },
  battered: { band: (t) => [t], col: ANTH, art: () => frameDouble(ANTH, ANTH_L) + tornDeep() },
  stained: { band: (t) => [t], col: ANTH, art: () => frameDashed(ANTH) + stains() },
};

/** The band colour of a voucher being redeemed. */
export const REDEEM_BAND = STAMP_REDEEM;

/**
 * The layer of one normal modification over the picture: its frame, mark
 * and curse art, then the band with `text` (the card's value formatted).
 */
export function modLayer(mod: string, text: string, value: number, paper: string, redeeming = false): string {
  const d = MOD_DRAW[mod];
  if (!d) return '';
  const col = redeeming && mod.startsWith('voucher_') ? STAMP_REDEEM : d.col;
  const items = d.band(text, value);
  return d.art(paper, redeeming) + (items.length ? band(items, col, paper) : '');
}

// ------------------------------------------------------------ Winkel scribble

// "Winkel v2" (user decision 04.11.2026, board Winkel-v2, gen-winkel.ts):
// the printed picture stays and is drawn over by hand in a violet copying
// pencil (WK-B's ink). Each step adds a layer: a hand-drawn outline pip
// when the number rises, a sketched figure when it becomes a picture card
// (Oberin with a bun, König with crown and beard), a big pip for the Daus;
// sketches are filled with paper (WK-A). In the corner the printed rank is
// scribbled out with a thick blot (WK-C) and each higher rank written
// beside it in a round hand font, all but the last blotted out too.

/** The hand font of the corner: 5 rows high, upright, round corners. */
const HAND: Record<string, string[]> = {
  '7': ['####', '...#', '.###', '..#.', '..#.'],
  '8': ['.##.', '#..#', '.##.', '#..#', '.##.'],
  '9': ['.##.', '#..#', '.###', '...#', '..#.'],
  T: ['.#..##.', '##.#..#', '.#.#..#', '.#.#..#', '.#..##.'],
  Q: ['.##.', '#..#', '#..#', '#..#', '.##.'],
  K: ['#..#', '#.#.', '##..', '#.#.', '#..#'],
  A: ['.##.', '#..#', '####', '#..#', '#..#'],
};
const handW = (rank: string): number => HAND[rank]?.[0]?.length ?? 4;
const printedW = (rank: string): number => ART_GLYPHS[rank]?.[0]?.length ?? 3;

/** Cells of `ch` in rows, top left at (x, y). */
function cellsOf(rows: string[], ch: string, x: number, y: number): Cells {
  const out: Cells = new Set();
  rows.forEach((row, j) => [...row].forEach((c, i) => c === ch && out.add(key(x + i, y + j))));
  return out;
}

/** A thick blot scribble over a glyph box (w × 5 at x, y). */
function strike(x: number, y: number, w: number): Cells {
  const out: Cells = new Set();
  for (let j = -1; j <= 5; j++) for (let i = -1; i <= w; i++) if ((i + j) % 3 !== 0 || j === -1 || j === 5) out.add(key(x + i, y + j));
  return filter(out, (xx, yy) => !((xx === x - 1 || xx === x + w) && (yy === y - 1 || yy === y + 5)));
}

/**
 * "Winkel" on both corner indices: the printed rank blotted out, each
 * higher rank hand-written beside it, all but the last blotted out too.
 * `chain` is the printed rank code first, e.g. ['8', '9', 'Q'].
 */
export function winkelScribble(chain: string[], paper: string): string {
  if (chain.length < 2) return '';
  let marks: Cells = new Set();
  let paperHalo: Cells = new Set();
  // Top left: the printed index at x 2, y 2; new ranks to the right.
  let x = 2;
  chain.forEach((r, i) => {
    const w = i === 0 ? printedW(r) : handW(r);
    if (i > 0) {
      const g = cellsOf(HAND[r] ?? [], '#', x, 2);
      paperHalo = union(paperHalo, halo(g));
      marks = union(marks, g);
    }
    if (i < chain.length - 1) marks = union(marks, strike(x, 2, w));
    x += w + 2;
  });
  // Bottom right: the printed index ends at x 30, y 42; new ranks to the left.
  let xr = 31;
  chain.forEach((r, i) => {
    const w = i === 0 ? printedW(r) : handW(r);
    const x0 = xr - w;
    if (i > 0) {
      const g = cellsOf(HAND[r] ?? [], '#', x0, 42);
      paperHalo = union(paperHalo, halo(g));
      marks = union(marks, g);
    }
    if (i < chain.length - 1) marks = union(marks, strike(x0, 42, w));
    xr = x0 - 2;
  });
  return draw(minus(paperHalo, marks), paper) + draw(marks, MARKER);
}

/** A sketch on the face: "k" pencil stroke, "p" inside (paper). */
interface Sketch {
  rows: string[];
  x: number;
  y: number;
}

/** The suit sign as an outline sketch (`scale` 2 for the Daus), with a pen tail. */
function pipRows(suit: string, scale = 1): string[] {
  const s = (ART_SIGNS[suit] ?? []).flatMap((r) =>
    Array.from({ length: scale }, () => [...r].flatMap((c) => Array.from({ length: scale }, () => (c === '.' ? '.' : '#'))).join('')),
  );
  const h = s.length;
  const w = s[0]?.length ?? 0;
  const on = (x: number, y: number): boolean => x >= 0 && y >= 0 && x < w && y < h && s[y]?.[x] === '#';
  const out = s.map((r, y) => [...r].map((c, x) => (c !== '#' ? '.' : on(x - 1, y) && on(x + 1, y) && on(x, y - 1) && on(x, y + 1) ? 'p' : 'k')));
  const lx = out[h - 1]?.lastIndexOf('k') ?? -1;
  out.push(Array.from({ length: w }, (_, i) => (i === lx + 1 ? 'k' : '.')));
  return out.map((r) => r.join(''));
}

const OBERIN = [
  '.....kkk.....',
  '....kpppk....',
  '.....kkk.....',
  '....kkkkk....',
  '...kpppppk...',
  '..kpkpppkpk..',
  '..kpppppppk..',
  '..kppkkkppk..',
  '...kpppppk...',
  '....kkkkk....',
  '.....kpk.....',
  '...kkkkkkk...',
  '..kpppppppk..',
  '.kpppppppppk.',
  'kpppppppppppk',
  'kkkkkkkkkkkkk',
];
const CROWN = ['k...k...k', 'kk.kkk.kk', 'kpkpppkpk', 'kpppppppk', 'kkkkkkkkk'];
const BEARD = ['k.k.k', 'kkkkk', '.kpk.', '..k..'];

/** What is sketched on the printed face for each step of the chain. */
function sketches(suit: string, chain: string[]): Sketch[] {
  const printed = chain[0] ?? '';
  const out: Sketch[] = [];
  const pip = (x: number, y: number): void => {
    out.push({ rows: pipRows(suit), x, y });
  };
  const two = chain.length === 3;
  for (let i = 1; i < chain.length; i++) {
    const to = chain[i];
    if (to === '8' || to === '9') {
      if (printed === '7') pip(i === 1 ? 13 : 23, 29);
      else pip(13, two ? 13 : 21);
    } else if (to === 'Q') {
      out.push({ rows: OBERIN, x: 10, y: printed === '8' ? 26 : 16 });
    } else if (to === 'K') {
      const y = printed === 'Q' ? 1 : 11;
      out.push({ rows: CROWN, x: 12, y }, { rows: BEARD, x: 14, y: y + 12 });
    } else if (to === 'T') {
      pip(13, printed === 'K' ? (two ? 16 : 22) : 24);
    } else if (to === 'A') {
      out.push({ rows: pipRows(suit, 2), x: 9, y: printed === 'T' ? 16 : 26 });
    }
  }
  return out;
}

/**
 * "Winkel" on the face: the hand sketches over the printed picture, one
 * layer per step. `suit` is the art code (C, S, H, D).
 */
export function winkelSketch(suit: string, chain: string[], paper: string): string {
  if (chain.length < 2) return '';
  return sketches(suit, chain)
    .map((s) => draw(cellsOf(s.rows, 'p', s.x, s.y), paper) + draw(cellsOf(s.rows, 'k', s.x, s.y), MARKER))
    .join('');
}

/**
 * "Untot" before it rose (playtest 05.11.2026): the printed card with a
 * green band and skull across its top, nothing else. Only the card that
 * came back into the hand shows the zombie picture.
 */
export function undeadBand(paper: string): string {
  return band(['skull'], ZOMBIE, paper, 1);
}

/** Ring 0 and 1 of a tiny card in the colour of its state (curse anthracite, Untot green). */
export function smallState(curse: boolean, undead: boolean): string {
  if (!curse && !undead) return '';
  const r = union(ring(0, false), ring(1, false));
  if (curse && undead) {
    const dark = filter(r, (x, y) => Math.floor((x + y) / 4) % 2 === 0);
    return draw(dark, ANTH) + draw(minus(r, dark), ZOMBIE);
  }
  return draw(r, curse ? ANTH : ZOMBIE);
}

// ------------------------------------------------------------------ icons

/** The glyphs of each modification's icon: its band without the card's value. */
const ICON: Record<string, { items: BandItem[]; col: string }> = {
  points: { items: ['pips'], col: GOLD },
  rank: { items: ['rang1'], col: MARKER },
  lead: { items: ['plumb'], col: LEAD },
  gilt_edge: { items: ['coin'], col: GILT_BAND },
  crosshair: { items: ['xhair'], col: XH },
  tailors_chalk: { items: ['scissor'], col: TAILOR },
  doppelganger: { items: ['mirror'], col: MIRROR },
  trump_bonus: { items: ['base'], col: PURPLE },
  last_trick: { items: ['levels'], col: TEAL },
  first_trick: { items: ['×2'], col: PLAY },
  voucher_coins: { items: ['coin'], col: STAMP },
  voucher_undead: { items: ['skull'], col: STAMP },
  voucher_wild: { items: ['wild'], col: STAMP },
  transparent: { items: ['eye'], col: ANTH },
  battered: { items: ['−'], col: ANTH },
  stained: { items: ['−'], col: ANTH },
  undead: { items: ['skull'], col: ZOMBIE },
  wild_card: { items: ['wild'], col: '#1c1a1a' },
};

/** Ids of the modifications with an icon of their own. */
export const ICON_MODS = Object.keys(ICON);

/**
 * A modification as a small pixel icon (viewBox 0 0 16 16), for lists
 * (encounters, score steps, unlocks): a tile in its band colour with its
 * glyph, or null for an unknown id.
 */
export function modIconBody(mod: string): string | null {
  const icon = ICON[mod];
  if (!icon) return null;
  const widths = icon.items.map((it) => glyphCells(it, 0, 0).w);
  const cw = widths.reduce((a, b) => a + b, 0) + 2 * (icon.items.length - 1);
  let x = Math.floor((16 - cw) / 2);
  let out = `<rect x="0" y="0" width="16" height="16" fill="#0b0c0e"/><rect x="1" y="1" width="14" height="14" fill="${icon.col}"/><rect x="1" y="14" width="14" height="1" fill="${shade(icon.col)}"/>`;
  icon.items.forEach((it, i) => {
    for (const [cells, c] of glyphCells(it, x, 5).parts) {
      for (const cell of cells) {
        const [cx, cy] = xy(cell);
        out += `<rect x="${cx}" y="${cy}" width="1" height="1" fill="${c ?? WHITE}"/>`;
      }
    }
    x += (widths[i] ?? 0) + 2;
  });
  return out;
}

// ---------------------------------------------------------- value icons, large

/** Kinds of the value icons: Augen, Grundwert, Stufen, Faktor. */
export type ValueIconKind = 'base' | 'levels' | 'pips' | 'factor';

/** The large Werte-Icons (board "Werte-Icons", G-B, A-C and the Stufen staircase): grid and palette, before the outline. */
const VALUE_ICONS: Record<ValueIconKind, { rows: string[]; pal: Record<string, string> }> = {
  pips: {
    rows: ['.pppppppp.', 'ppppppppps', 'ppddppddps', 'ppddppddps', 'ppppddppps', 'ppppddppps', 'ppddppddps', 'ppddppddps', 'ppppppppps', '.ssssssss.'],
    pal: { p: '#fdfdfb', s: '#c9cfd2', d: '#2b2f33' },
  },
  base: {
    rows: ['...www...', '...sss...', '.........', '.www.www.', '.sss.sss.', '.........', 'ww.www.ww', 'ss.sss.ss'],
    pal: { w: '#e4ede8', s: '#93aea2' },
  },
  // Stufen: three copper steps rising to the right, lit treads on top.
  levels: {
    rows: ['......lll', '......ccc', '......ccc', '...lllccc', '...cccccc', '...cccccc', 'lllcccccc', 'ccccccccc', 'ccccccccc'],
    pal: { c: '#d1773f', l: '#f1b48a' },
  },
  // Faktor (ADR 0018), provisional: an amber "×" with a gold caret above
  // it to the right, the exponent mark of "multiplied as a whole".
  factor: {
    rows: ['......gg..', '.....g..g.', '....g....g', 'cl..lc....', '.cllc.....', '..cc......', '.clcl.....', 'cl..cl....'],
    pal: { c: '#c98a12', l: '#ffd25e', g: '#f7c21a' },
  },
};

/**
 * The Grundwert, Stufen, Faktor or Augen icon in its large form
 * (stone bricks, copper staircase, amber "×" with a caret, a die) with a dark one-pixel outline, as inline SVG `height` px
 * high: beside the labels of the score boxes.
 */
export function valueIcon(kind: ValueIconKind, height = 14): string {
  const { rows, pal } = VALUE_ICONS[kind];
  const h = rows.length + 2;
  const w = (rows[0]?.length ?? 0) + 2;
  const at = (x: number, y: number): string => rows[y - 1]?.[x - 1] ?? '.';
  let d = '';
  const fills = new Map<string, string>();
  for (let y = 0; y < h; y++) {
    for (let x = 0; x < w; x++) {
      const c = at(x, y);
      if (c !== '.') {
        fills.set(c, (fills.get(c) ?? '') + `M${x} ${y}h1v1h-1z`);
        continue;
      }
      let near = false;
      for (let dy = -1; dy <= 1; dy++) for (let dx = -1; dx <= 1; dx++) if (at(x + dx, y + dy) !== '.') near = true;
      if (near) d += `M${x} ${y}h1v1h-1z`;
    }
  }
  const paths = `<path d="${d}" fill="#0b0c0e"/>` + [...fills].map(([c, p]) => `<path d="${p}" fill="${pal[c] ?? '#fff'}"/>`).join('');
  return `<svg class="value-icon" viewBox="0 0 ${w} ${h}" width="${Math.round((height * w) / h)}" height="${height}" shape-rendering="crispEdges" aria-hidden="true">${paths}</svg>`;
}

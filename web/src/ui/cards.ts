// Cards as small SVGs that reference the sprite in ./art.ts. A face is
// drawn in layers on the art grid of the deck (33 × 49.5 art pixels, one
// art pixel = 2 × 2 CSS px at the 66 × 99 hand size): paper (tinted by
// some modifications, else plain or, with the setting "Kartenfarbe
// tönen", lightly in the suit's colour), the picture (the zombie picture
// for an "Untot" picture card), the frames and marks of modifications,
// then the corner index on top.
//
// Frame language (spec of 02.11.2026, tools/cardart/fx2.py): the edge is
// the quick hint, numbers sit at the bottom middle.
//
//   ring 0        the state: Fluch red and broken, Untot green and ragged,
//                 both alternating
//   ring 1 and 2  the first family (Glas, Augen, Trumpf, Wandel,
//                 Ehrenunter, Sicht, Heimweh, in that order of strength)
//   ring 3        a second family as a single line; a third is a badge
//   top left      corner index; Rang chevrons in the column below it
//   top right     the stepped flag of "Letzter Stich" (HTML plaques too)
//   bottom left   one corner sign (the skull of Untot first)
//   bottom middle one or two printed numbers in their frame's colour
//   bottom right  corner index
//   left edge     badges of whatever found its place taken
//
// Rings 1 to 3 leave both index corners free. Marks over the picture get
// a one-pixel paper halo, so they read on the busy art. Tiny cards (about
// 16 to 26 px) use the small face: the index letter and the suit sign,
// big, with the state as a thin ring. Pure drawing, no game logic: the
// printed numbers come from the core's catalog.

import { de, cardName, contentName, face, factorText, isCurse, modFigure, t } from '../i18n';
import type { CardKind, ModFigure, Suit } from '../types';
import { CARD_H, CARD_W, faceId, indexId, modIconId, PAPER, smallFaceId, suitId, undeadFaceId } from './art';
import { backId, DEFAULT_BACK, type BackDesign, type BackSize } from './backs';
import { esc } from './dom';

/** A suit sign (the held sign of the deck) as a standalone inline SVG. */
export function suitIcon(suit: Suit, size = 24, label: string | null = de.suits[suit]): string {
  const a11y = label ? `role="img" aria-label="${esc(label)}"` : 'aria-hidden="true"';
  return `<svg class="suit-icon" viewBox="0 0 8 8" width="${size}" height="${size}" shape-rendering="crispEdges" ${a11y}><use href="#${suitId(suit)}"/></svg>`;
}

/** A card modification icon (badge) as inline SVG. */
export function modIcon(mod: string, size = 20): string {
  return `<svg class="mod-icon" viewBox="0 0 24 24" width="${size}" height="${size}" aria-hidden="true"><use href="#${modIconId(mod)}"/></svg>`;
}

// ------------------------------------------------------- modification art

const CURSE_RED = '#bd3b32';
const UNDEAD_GREEN = '#5e7a3c';
const UNDEAD_DRIP = '#9cb86e';
const UNDEAD_SKULL = '#3c5226';

/**
 * The plain paper of a face. A CSS variable, so the setting "Kartenfarbe
 * tönen" can tint it per suit (`data-suit` on the SVG, see style.css)
 * without drawing the cards again; the halos of marks follow it. The
 * paper of a modification (Glasdaus, ...) replaces it, so the
 * modification stays recognisable.
 */
const PLAIN = `var(--suit-paper, ${PAPER})`;

/** A fill as SVG attribute; CSS variables need the style property. */
function fillOf(fill: string): string {
  return fill.startsWith('var(') ? `style="fill:${fill}"` : `fill="${fill}"`;
}
const W = CARD_W;
const H = CARD_H;
/** Full art rows; the half row at the bottom stays paper. */
const ROWS = Math.floor(H);

/** Pixels on the art grid as "x,y" keys. */
type Cells = Set<string>;

const key = (x: number, y: number): string => `${x},${y}`;
const xy = (c: string): [number, number] => {
  const [x = 0, y = 0] = c.split(',').map(Number);
  return [x, y];
};
function cellsOf(list: Iterable<[number, number]>): Cells {
  const out: Cells = new Set();
  for (const [x, y] of list) out.add(key(x, y));
  return out;
}
function filter(cells: Cells, f: (x: number, y: number) => boolean): Cells {
  return new Set([...cells].filter((c) => f(...xy(c))));
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

/** Every cell blown up to a k × k block, from (x0, y0). */
function scaled(cells: Cells, k: number, x0: number, y0: number): Cells {
  const out: Cells = new Set();
  for (const c of cells) {
    const [x, y] = xy(c);
    for (let j = 0; j < k; j++) for (let i = 0; i < k; i++) out.add(key(x0 + x * k + i, y0 + y * k + j));
  }
  return out;
}

/** The cells on the card as one path, merged into runs per row. */
function draw(cells: Cells, fill: string): string {
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
  return d ? `<path d="${d}" ${fillOf(fill)}/>` : '';
}

/** Marks over the picture with a one-pixel halo in the paper colour around them all. */
function marked(parts: [Cells, string][], halo: string): string {
  const ring: Cells = new Set();
  for (const [cells] of parts) {
    for (const c of cells) {
      const [x, y] = xy(c);
      for (let dy = -1; dy <= 1; dy++) for (let dx = -1; dx <= 1; dx++) ring.add(key(x + dx, y + dy));
    }
  }
  return draw(ring, halo) + parts.map(([cells, fill]) => draw(cells, fill)).join('');
}

// ---------------------------------------------------------------- rings

/** Corner index notches: rings 1 to 3 leave them free. */
const NOTCH_TL: [number, number] = [7, 13];
const NOTCH_BR: [number, number] = [26, 36];

/** The k-th ring of pixels from the card's edge (tools/cardart/fx2.py `ring`). */
function ring(k: number, notch = true): Cells {
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

/**
 * Frames of the families, on ring 1 (with ring 2 where the family has a
 * second line) or, for a second family, on ring 3 alone.
 */
type Ring = 1 | 3;
const frames = {
  /** Augen: a double frame, outer colour and light inner line. */
  augen: (col: string, light: string) => (k: Ring) => (k === 1 ? draw(ring(1), col) + draw(ring(2), light) : draw(ring(k), col)),
  /** Trumpf: the perforated edge of a postage stamp. */
  trumpf: (col: string) => (k: Ring) => draw(filter(ring(k), (x, y) => (x + y) % 2 === 0), col),
  /** Sicht: a dashed line, two on, two off. */
  sicht: (col: string) => (k: Ring) => draw(filter(ring(k), (x, y) => Math.floor((x + y) / 2) % 2 === 0), col),
  /** Glas: light blue, double, with a glint across the picture. */
  glas: () => (k: Ring) => {
    let out = draw(ring(k), '#7fc4e8');
    if (k === 1) {
      out += draw(ring(2), '#d6eefa');
      const glint = cellsOf([...Array.from({ length: 12 }, (_, i): [number, number] => [5 + i, 35 - i]), ...Array.from({ length: 8 }, (_, i): [number, number] => [8 + i, 37 - i])]);
      out += draw(glint, '#ffffff');
    }
    return out;
  },
  /** Wandel: the four suit colours, Eichel top left to Schellen bottom right. */
  wandel: () => (k: Ring) => {
    const r = ring(k);
    const half = Math.floor(ROWS / 2);
    const top = filter(r, (_x, y) => y <= 3);
    const bottom = filter(r, (_x, y) => y >= ROWS - 4);
    const union = (a: Cells, b: Cells): Cells => new Set([...a, ...b]);
    return (
      draw(union(top, filter(r, (x, y) => y < half && x <= 3)), '#8b4a1c') +
      draw(filter(r, (x, y) => y < half && x >= W - 4), '#2e9a3c') +
      draw(union(filter(r, (x, y) => y >= half && x <= 3), bottom), '#e0281e') +
      draw(filter(r, (x, y) => y >= half && x >= W - 4), '#f7c21a')
    );
  },
  /** Ehrenunter: blue, with a dotted line inside. */
  ehren: () => (k: Ring) => draw(ring(k), '#1f5a8c') + (k === 1 ? draw(filter(ring(2), (x, y) => (x + y) % 3 === 0), '#1f5a8c') : ''),
  /** Heimweh: wine-red stitches. */
  heim: () => (k: Ring) => draw(filter(ring(k), (x, y) => (x + y) % 3 !== 2), '#8e2430'),
};

/** Ring 0, the state "Fluch": red and broken. */
const CURSE_FRAME = draw(
  rects(
    '0 0 6 1, 8 0 2 1, 12 0 9 1, 23 0 2 1, 27 0 6 1, 0 1 1 6, 0 9 1 2, 0 13 1 8, 0 23 1 3, 0 28 1 8, 0 38 1 2, 0 42 1 7, ' +
      '32 1 1 6, 32 9 1 2, 32 13 1 8, 32 23 1 3, 32 28 1 8, 32 38 1 2, 32 42 1 7, 0 48 6 1, 8 48 2 1, 12 48 9 1, 23 48 2 1, 27 48 6 1',
  ),
  CURSE_RED,
);

/** Ring 0, the state "Untot": ragged zombie green with drips inwards. */
function undeadFrame(): string {
  const r = ring(0, false);
  const cells = filter(r, (x, y) => (x * 7 + y * 3) % 5 !== 0);
  const drips: Cells = new Set();
  for (const c of r) {
    const [x, y] = xy(c);
    if ((x * 5 + y * 11) % 13 !== 0) continue;
    if (y === 0) {
      drips.add(key(x, 1));
      drips.add(key(x, 2));
    } else if (x === 0) drips.add(key(1, y));
    else if (x === W - 1) drips.add(key(W - 2, y));
  }
  return draw(cells, UNDEAD_GREEN) + draw(drips, UNDEAD_DRIP);
}

/** Ring 0 with both states: red and green every four pixels. */
function curseUndeadFrame(): string {
  const r = ring(0, false);
  const red = filter(r, (x, y) => Math.floor((x + y) / 4) % 2 === 0);
  return draw(red, CURSE_RED) + draw(filter(r, (x, y) => !red.has(key(x, y))), UNDEAD_GREEN);
}

// ------------------------------------------------------------- numbers

/** Pixel figures for the printed numbers, 5 high, in the style of the corner index. */
const DIGITS: Record<string, string[]> = {
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
  ',': ['.', '.', '.', '#', '#'],
};

function glyphsOf(text: string): string[][] {
  return [...text].map((ch) => DIGITS[ch]).filter((g): g is string[] => Boolean(g));
}

function numberWidth(text: string): number {
  const glyphs = glyphsOf(text);
  return glyphs.reduce((w, g) => w + (g[0]?.length ?? 0), 0) + Math.max(0, glyphs.length - 1);
}

/** A printed number in pixel figures from column x (centred without), row 42, with its halo. */
function pixelNumber(text: string, fill: string, paper: string, at?: number): string {
  let x = at ?? Math.floor((W - numberWidth(text)) / 2);
  const cells: Cells = new Set();
  for (const g of glyphsOf(text)) {
    for (const c of shape(g, x, 42)) cells.add(c);
    x += (g[0]?.length ?? 0) + 1;
  }
  return marked([[cells, fill]], paper);
}

/** One or two printed numbers at the bottom middle; two side by side, the first one left. */
function numbers(list: { text: string; fill: string }[], paper: string): string {
  const [a, b] = list;
  if (!a) return '';
  if (!b) return pixelNumber(a.text, a.fill, paper);
  const w1 = numberWidth(a.text);
  const x = Math.floor((W - (w1 + numberWidth(b.text) + 3)) / 2);
  return pixelNumber(a.text, a.fill, paper, x) + pixelNumber(b.text, b.fill, paper, x + w1 + 3);
}

// --------------------------------------------------------------- marks

/** Rang: chevrons in the column under the top index, one per step. */
const CHEVRON = ['...#...', '..###..', '.##.##.', '##...##'];
function chevrons(n: number, fill: string, paper: string): string {
  return marked(Array.from({ length: n }, (_, i): [Cells, string] => [shape(CHEVRON, 0, 13 + i * 5), fill]), paper);
}

/** Letzter Stich: a stepped flag top right with its sign. */
function lastTrickFlag(fill: string, sign: string[]): string {
  let list = '';
  for (let y = 0; y <= 12; y++) list += `${list ? ', ' : ''}${20 + y} ${y} ${W - 20 - y} 1`;
  return draw(rects(list), fill) + draw(shape(sign, 27, 1), '#ffffff');
}

/** The faded look of a card that never takes a trick. */
const VEIL = `<rect width="${W}" height="${H}" ${fillOf(PLAIN)} opacity=".7"/>`;

const SKULL = ['.###.', '#####', '#.#.#', '#####', '.#.#.'];

/** A sign at the bottom left corner (5 × 5 from x 2, y 42). */
interface CornerSign {
  rows: string[];
  fill: string;
  /** Cells besides the sign, in their own colour (the eye's pupil). */
  extra?: [Cells, string];
}

/** The Schwarze Katze sits at the bottom left and looks over the edge. */
function blackCat(paper: string): string {
  const cat = shape(['#...#.....', '##.##.....', '#####.....', '#.#.#.....', '#####...#.', '.###...#..', '#####..#..', '######.#..', '#######...'], 1, 40);
  return marked([[cat, '#1d1a18']], paper) + draw(cellsOf([[2, 43], [4, 43]]), '#f7c21a');
}

// ------------------------------------------------------------ designs

type Family = 'glas' | 'augen' | 'trumpf' | 'wandel' | 'ehren' | 'sicht' | 'heim';

/** Which family gets ring 1 and 2 when a card has two: the stronger frame first. */
const FAMILY_ORDER: Family[] = ['glas', 'augen', 'trumpf', 'wandel', 'ehren', 'sicht', 'heim'];

/** Where a mark without a frame sits; two of the same place do not fit. */
type Place = 'top' | 'flag' | 'stain' | 'crack' | 'notch' | 'center';

interface ModArt {
  family?: Family;
  /** The family's frame on ring 1 (and 2) or on ring 3. */
  frame?: (k: Ring) => string;
  /** Also takes ring 3 when on ring 1 (Goldrand, threefold). */
  ring3?: boolean;
  /** The picture fades (Sicht). */
  veil?: boolean;
  /** Colour of the printed number in the frame's colour; the number comes from the core's catalog. */
  value?: string;
  sign?: CornerSign;
  /** The Schwarze Katze or the torn corner take the corner like a sign. */
  corner?: (paper: string) => string;
  /** A mark at a place of its own. */
  mark?: { place: Place; draw: (paper: string) => string };
  /** The paper where it carries meaning. */
  paper?: string;
}

const GOLD = '#d98e0b';
const GOLD_LIGHT = '#ffe08a';
const GOLD_DARK = '#8a5a0a';
const PURPLE = '#6a3fa8';
const EYE: CornerSign = { rows: ['.....', '.###.', '#####', '.###.', '.....'], fill: '#2f86c0', extra: [cellsOf([[4, 44]]), '#0b0c0e'] };

/** The design per modification id (frame language, tools/cardart/fx2.py and the spec of 02.11.2026). */
const MOD_ART: Record<string, ModArt> = {
  // Augen: a double frame in gold or lead, the extra Augen in its colour.
  fat: { family: 'augen', frame: frames.augen(GOLD, GOLD_LIGHT), value: GOLD_DARK },
  gold_rim: { family: 'augen', frame: frames.augen('#f7c21a', '#fff3b0'), ring3: true, value: GOLD_DARK },
  lead_weight: { family: 'augen', frame: frames.augen('#4a4f57', '#a9adb3'), value: '#4a4f57' },
  // Rang: chevrons under the index, no frame.
  marked: { mark: { place: 'top', draw: (p) => chevrons(1, '#8e2430', p) } },
  double_marked: { mark: { place: 'top', draw: (p) => chevrons(2, '#8e2430', p) } },
  dressed_up: { mark: { place: 'top', draw: (p) => chevrons(2, '#b0336e', p) }, sign: { rows: ['#...#', '##.##', '#####', '##.##', '#...#'], fill: '#b0336e' } },
  // Trumpf: the perforated edge of a stamp, the number in its colour.
  stamp: { family: 'trumpf', frame: frames.trumpf(PURPLE), value: PURPLE },
  crown: { family: 'trumpf', frame: frames.trumpf('#c08a12'), value: '#c08a12', sign: { rows: ['#.#.#', '#####', '#####', '.....', '.....'], fill: '#c08a12' } },
  seal: { family: 'trumpf', frame: frames.trumpf('#a3263a'), value: '#a3263a', sign: { rows: ['.###.', '##.##', '#...#', '##.##', '.###.'], fill: '#a3263a' } },
  horseshoe: { family: 'trumpf', frame: frames.trumpf('#4f6272'), value: '#4f6272', sign: { rows: ['#...#', '#...#', '#...#', '.###.', '.....'], fill: '#4f6272' } },
  tape_measure: { family: 'trumpf', frame: frames.trumpf('#b07d0c'), value: '#b07d0c', sign: { rows: ['#####', '#.#.#', '#####', '.....', '.....'], fill: '#b07d0c' } },
  // Letzter Stich: the flag top right, no frame.
  last_trick: { mark: { place: 'flag', draw: () => lastTrickFlag('#16707a', ['#....', '###..', '####.', '#....', '#....']) } },
  lucky_penny: { mark: { place: 'flag', draw: () => lastTrickFlag('#b06a2c', ['.###.', '#.#.#', '#.#.#', '#.#.#', '.###.']) } },
  tail_light: { mark: { place: 'flag', draw: () => lastTrickFlag('#b3202a', ['.###.', '#####', '#####', '.###.', '#.#.#']) } },
  // Sicht: a dashed frame; faded where the card never takes a trick.
  transparent: { family: 'sicht', frame: frames.sicht('#2f86c0'), sign: EYE },
  invisible: { family: 'sicht', frame: frames.sicht('#7a7f8a'), veil: true },
  dummy: {
    family: 'sicht',
    frame: frames.sicht('#7a7f8a'),
    veil: true,
    sign: EYE,
    mark: { place: 'center', draw: () => draw(scaled(shape(['.###.', '#...#', '...#.', '..#..', '..#..', '.....', '..#..'], 0, 0), 2, 12, 17), '#7d6a50') },
  },
  honorary_jack: { family: 'ehren', frame: frames.ehren(), paper: '#e3ebf5', sign: { rows: ['#...#', '.#.#.', '.###.', '.###.', '..#..'], fill: '#1f5a8c' } },
  wild_card: { family: 'wandel', frame: frames.wandel(), sign: { rows: ['##.##', '##.##', '.....', '##.##', '##.##'], fill: '#2e7d4f' } },
  glass_ace: { family: 'glas', frame: frames.glas(), paper: '#e8f4fc', value: '#2f86c0' },
  homesick: { family: 'heim', frame: frames.heim(), sign: { rows: ['..#..', '.###.', '#####', '.#.#.', '.###.'], fill: '#8e2430' } },
  // Flüche: ring 0 red and broken (for every curse), and their mark.
  beer_stain: { value: CURSE_RED, mark: { place: 'stain', draw: (p) => marked([[rects('19 15 6 2, 18 17 9 3, 19 20 7 2, 21 22 3 1'), '#c98a2a']], p) } },
  cracked: {
    mark: {
      place: 'crack',
      draw: (p) => marked([[rects('13 0 1 5, 14 5 1 3, 12 8 2 1, 11 9 1 5, 12 14 1 1, 13 15 1 4, 14 19 1 4, 13 23 1 3, 12 26 1 5, 13 31 2 1, 15 32 1 5, 14 37 1 4, 13 41 1 8'), '#1d1a18']], p),
    },
  },
  black_cat: { paper: '#ece4d4', value: CURSE_RED, corner: blackCat },
  battered: {
    corner: () => draw(rects('0 44 1 1, 0 45 2 1, 0 46 3 1, 0 47 4 1, 0 48 5 1'), '#1b1e21'),
    mark: { place: 'notch', draw: () => draw(rects('30 22 3 1, 29 23 4 1, 30 24 3 1'), '#1b1e21') },
  },
  // Untot: ring 0 ragged green (drawn for the state), the skull in the corner.
  undead: { sign: { rows: SKULL, fill: UNDEAD_SKULL } },
};

/** The modification "Untot" (core/content/card_mods.ron). */
export const UNDEAD = 'undead';

/** "+4", "−3", "×2", "56": the printed number of a modification. Formatting only. */
export function figureText(figure: ModFigure | null): string {
  if (!figure) return '';
  const c = figure.change;
  if ('Multiply' in c) return `×${c.Multiply}`;
  if ('Scale' in c) return `×${factorText(c.Scale)}`;
  if ('Set' in c) return String(c.Set);
  return c.Add > 0 ? `+${c.Add}` : c.Add < 0 ? `−${Math.abs(c.Add)}` : '0';
}

interface Layers {
  paper: string;
  /** Over the picture, under the corner index. */
  over: string;
  badges: string[];
  curse: boolean;
  undead: boolean;
}

/**
 * Lay out the modifications of one card in the frame language: ring 0 the
 * state (Fluch, Untot), ring 1 and 2 the first family, ring 3 a second one,
 * a third becomes a badge; numbers at the bottom middle (two side by side),
 * one corner sign bottom left (the skull of Untot first), marks at their
 * places. Whatever does not fit is a badge on the left edge.
 */
function layoutMods(mods: string[]): Layers {
  const curse = mods.some(isCurse);
  const undead = mods.includes(UNDEAD);
  const out: Layers = { paper: PLAIN, over: '', badges: [], curse, undead };
  const badge = (mod: string): void => {
    if (!out.badges.includes(mod)) out.badges.push(mod);
  };
  for (const mod of mods) {
    const art = MOD_ART[mod];
    if (!art) badge(mod);
    else if (art.paper && out.paper === PLAIN) out.paper = art.paper;
  }
  const known = mods.filter((m) => MOD_ART[m]);
  // Frames: by family strength; the first on ring 1 and 2, the second on ring 3.
  const framed = known
    .filter((m) => MOD_ART[m]?.frame)
    .sort((a, b) => FAMILY_ORDER.indexOf(MOD_ART[a]?.family ?? 'heim') - FAMILY_ORDER.indexOf(MOD_ART[b]?.family ?? 'heim'));
  const placed: string[] = [];
  let veil = false;
  let frameLayer = '';
  framed.forEach((mod, i) => {
    const art = MOD_ART[mod];
    const ring3Free = !(i === 1 && MOD_ART[framed[0] ?? '']?.ring3);
    if (!art?.frame || i > 1 || !ring3Free) {
      badge(mod);
      return;
    }
    frameLayer += art.frame(i === 0 ? 1 : 3);
    if (i === 0 && art.ring3) frameLayer += art.frame(3);
    if (art.veil) veil = true;
    placed.push(mod);
  });
  // The rest: Rang, Letzter Stich, curses and Untot, in the card's order.
  for (const mod of known) if (!MOD_ART[mod]?.frame) placed.push(mod);
  // Marks at their places.
  const used = new Set<Place>();
  let marks = '';
  for (const mod of placed) {
    const mark = MOD_ART[mod]?.mark;
    if (!mark) continue;
    if (used.has(mark.place)) {
      badge(mod);
      continue;
    }
    used.add(mark.place);
    marks += mark.draw(out.paper);
  }
  // The corner bottom left: the skull of Untot first, then in order.
  let corner = '';
  const cornerMods = placed.filter((m) => MOD_ART[m]?.sign || MOD_ART[m]?.corner);
  if (undead) cornerMods.sort((a, b) => Number(b === UNDEAD) - Number(a === UNDEAD));
  cornerMods.forEach((mod, i) => {
    const art = MOD_ART[mod];
    if (!art) return;
    if (i > 0) {
      badge(mod);
      return;
    }
    if (art.corner) corner += art.corner(out.paper);
    else if (art.sign) {
      const parts: [Cells, string][] = [[shape(art.sign.rows, 2, 42), art.sign.fill]];
      if (art.sign.extra) parts.push(art.sign.extra);
      corner += marked(parts, out.paper);
    }
  });
  // Numbers: the first family left, then the second, then the curses.
  const values: { text: string; fill: string }[] = [];
  for (const mod of placed) {
    const art = MOD_ART[mod];
    if (!art?.value) continue;
    const text = figureText(modFigure(mod));
    if (!text) continue;
    if (values.length >= 2) badge(mod);
    else values.push({ text, fill: art.value });
  }
  let state = '';
  if (undead) state = curse ? curseUndeadFrame() : undeadFrame();
  out.over = (veil ? VEIL : '') + frameLayer + state + marks + corner + numbers(values, out.paper) + (curse && !undead ? CURSE_FRAME : '');
  return out;
}

/** Ring 0 of a tiny card in the colour of its state, 2 art pixels wide (about 1 CSS px). */
function smallState(curse: boolean, undead: boolean): string {
  if (!curse && !undead) return '';
  const r = new Set([...ring(0, false), ...ring(1, false)]);
  if (curse && undead) {
    const red = filter(r, (x, y) => Math.floor((x + y) / 4) % 2 === 0);
    return draw(red, CURSE_RED) + draw(filter(r, (x, y) => !red.has(key(x, y))), UNDEAD_GREEN);
  }
  return draw(r, curse ? CURSE_RED : UNDEAD_GREEN);
}

/** Size variant of a face: the full picture, or index and sign for tiny cards. */
export type FaceSize = 'full' | 'small';

/**
 * The face of a card as inline SVG (viewBox 33 × 49.5 art pixels). `size`
 * "small" is for tiny cards (about 16 to 26 px wide): the index letter and
 * the suit sign, big, with the paper of the modifications and the state
 * (Fluch, Untot) as a thin ring, but without frames and marks. An "Untot"
 * picture card shows its zombie face.
 */
export function cardSvg(kind: CardKind, mods: string[] = [], size: FaceSize = 'full'): string {
  const f = face(kind);
  const modNames = mods.map((m) => (isCurse(m) ? `${contentName('card_mods', m)} (${de.card.curse})` : contentName('card_mods', m))).join(', ');
  const label = esc(cardName(kind) + (mods.length ? ', ' + t(de.card.mods, { mods: modNames }) : ''));
  const layers = layoutMods(mods);
  const paper = `<rect class="paper" width="${W}" height="${H}" ${fillOf(layers.paper)}/>`;
  const suit = f ? ` data-suit="${f.suit}"` : '';
  const state = layers.undead || layers.curse ? ` data-state="${layers.undead && layers.curse ? 'both' : layers.undead ? 'undead' : 'curse'}"` : '';
  let body: string;
  if (size === 'small') {
    body = f ? `${paper}<use href="#${smallFaceId(f.suit, f.rank)}"/>` : `<use href="#f-blank"/>`;
    body += smallState(layers.curse, layers.undead);
  } else {
    const badges = layers.badges.map((m, i) => `<use href="#${modIconId(m)}" x="1" y="${32 - i * 9}" width="8" height="8"/>`).join('');
    const picture = f ? (layers.undead ? undeadFaceId(f.suit, f.rank) ?? faceId(f.suit, f.rank) : faceId(f.suit, f.rank)) : null;
    body = f && picture ? `${paper}<use href="#${picture}"/>${layers.over}<use href="#${indexId(f.suit, f.rank)}"/>` : `<use href="#f-blank"/>${layers.over}`;
    body += badges;
  }
  return `<svg class="card-svg${size === 'small' ? ' is-small' : ''}" viewBox="0 0 ${W} ${H}" preserveAspectRatio="none" shape-rendering="crispEdges" role="img" aria-label="${label}"${suit}${state}>${body}</svg>`;
}

/**
 * Back of a card in one of the back designs (a cosmetic setting, ./backs.ts).
 * `size` "small" picks the simplified picture for tiny backs.
 */
export function cardBack(className = '', design: BackDesign = DEFAULT_BACK, size: BackSize = 'full'): string {
  return `<span class="card-back ${className}" role="img" aria-label="${esc(de.card.back)}"><svg viewBox="0 0 120 180" shape-rendering="crispEdges" aria-hidden="true"><use href="#${backId(design, size)}"/></svg></span>`;
}

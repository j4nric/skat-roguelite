// Cards as small SVGs that reference the sprite in ./art.ts. A face is
// drawn in layers on the art grid of the deck (33 × 49.5 art pixels, one
// art pixel = 2 × 2 CSS px at the 66 × 99 hand size): paper (tinted by
// some modifications), the picture, the washes and tints of modifications,
// the corner index, then the marks of modifications on top. Every
// modification has the design the player chose, as pixel art on the same
// grid, at fixed places that keep both corner indexes free:
//
//   top left      corner index; Rang chevrons in the column below it
//   top right     the stepped flag of "Letzter Stich" (HTML plaques too)
//   bottom left   the corner sign of a modification (eye, house, ...)
//   bottom middle the printed number
//   bottom right  corner index
//   left edge     badges of modifications that found their place taken
//
// Marks over the picture get a one-pixel paper halo, so they read on the
// busy art. The Augen frame lies outside the card (a CSS outline, see
// `data-frame` in style.css). Tiny cards (about 16 to 26 px) use the
// small face: the index letter and the suit sign, big. Pure drawing, no
// game logic: the printed numbers come from the core's catalog.

import { de, cardName, contentName, face, factorText, isCurse, modFigure, t } from '../i18n';
import type { CardKind, ModFigure, Suit } from '../types';
import { CARD_H, CARD_W, faceId, indexId, modIconId, PAPER, smallFaceId, suitId } from './art';
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
const W = CARD_W;
const H = CARD_H;

/**
 * Where a modification draws. Two modifications with a common place do not
 * both fit: the later one becomes a badge.
 */
type Place = 'outline' | 'value' | 'wash' | 'tint' | 'top' | 'corner' | 'flag' | 'edge' | 'center' | 'stain' | 'crack' | 'notch';

interface ModArt {
  places: Place[];
  /** Colour of the paper instead of the plain one. */
  paper?: string;
  /** Colour of the frame outside the card (CSS outline). */
  frame?: string;
  /** Over the picture, under the corner index. */
  under?: () => string;
  /** Over everything; `paper` is the card's paper (for halos). */
  over?: (paper: string) => string;
  /** Colour of the printed number, centred at the bottom. */
  value?: { fill: string };
}

/** Pixels on the art grid as "x,y" keys. */
type Cells = Set<string>;

/** Rectangles on the art grid, "x y w h" separated by commas. */
function rects(list: string): Cells {
  const out: Cells = new Set();
  for (const r of list.split(',')) {
    const [x = 0, y = 0, w = 0, h = 0] = r.trim().split(/\s+/).map(Number);
    for (let j = 0; j < h; j++) for (let i = 0; i < w; i++) out.add(`${x + i},${y + j}`);
  }
  return out;
}

/** A pixel icon from rows of "#" and ".", top left at (x, y). */
function shape(rows: string[], x: number, y: number): Cells {
  const out: Cells = new Set();
  rows.forEach((row, j) => [...row].forEach((ch, i) => ch === '#' && out.add(`${x + i},${y + j}`)));
  return out;
}

/** The cells as one path, merged into runs per row. */
function draw(cells: Cells, fill: string): string {
  const rows = new Map<number, number[]>();
  for (const c of cells) {
    const [x = 0, y = 0] = c.split(',').map(Number);
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
  return d ? `<path d="${d}" fill="${fill}"/>` : '';
}

/** Marks over the picture with a one-pixel halo in the paper colour around them all. */
function marked(parts: [Cells, string][], halo: string): string {
  const all: Cells = new Set();
  for (const [cells] of parts) for (const c of cells) all.add(c);
  const ring: Cells = new Set();
  for (const c of all) {
    const [x = 0, y = 0] = c.split(',').map(Number);
    for (let dy = -1; dy <= 1; dy++) for (let dx = -1; dx <= 1; dx++) if (x + dx >= 0 && x + dx < W && y + dy >= 0) ring.add(`${x + dx},${y + dy}`);
  }
  return draw(ring, halo) + parts.map(([cells, fill]) => draw(cells, fill)).join('');
}

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

/** A printed number in pixel figures, centred on the card, top at row `y`, with its halo. */
function pixelNumber(text: string, y: number, fill: string, paper: string): string {
  const glyphs = [...text].map((ch) => DIGITS[ch]).filter((g): g is string[] => Boolean(g));
  const width = glyphs.reduce((w, g) => w + (g[0]?.length ?? 0), 0) + Math.max(0, glyphs.length - 1);
  let x = Math.floor((W - width) / 2);
  const cells: Cells = new Set();
  for (const g of glyphs) {
    for (const c of shape(g, x, y)) cells.add(c);
    x += (g[0]?.length ?? 0) + 1;
  }
  return marked([[cells, fill]], paper);
}

/** A 5 × 5 sign at the bottom left corner, with its halo. */
function cornerSign(rows: string[], fill: string, paper: string): string {
  return marked([[shape(rows, 2, 42), fill]], paper);
}

/** A big faint pixel shape over the picture: 4-pixel blocks from (6, 14). */
function wash(rows: string[], fill: string, opacity = 0.22): string {
  return `<g opacity="${opacity}">${draw(
    new Set([...shape(rows, 0, 0)].flatMap((c) => {
      const [x = 0, y = 0] = c.split(',').map(Number);
      return [...rects(`${6 + x * 4} ${14 + y * 4} 4 4`)];
    })),
    fill,
  )}</g>`;
}

/** Rang: chevrons in the column under the top index, one per step. */
const CHEVRON = ['...#...', '..###..', '.##.##.', '##...##'];
function chevrons(n: number, fill: string, paper: string): string {
  return marked(Array.from({ length: n }, (_, i): [Cells, string] => [shape(CHEVRON, 0, 13 + i * 5), fill]), paper);
}

/** The flag of the last trick: a stepped triangle top right with its sign. */
function lastTrickFlag(fill: string, sign: string[]): string {
  let list = '';
  for (let y = 0; y <= 12; y++) list += `${list ? ', ' : ''}${20 + y} ${y} ${W - 20 - y} 1`;
  return draw(rects(list), fill) + draw(shape(sign, 27, 1), '#ffffff');
}

/** A dashed line just inside the card's edge. */
function dashedEdge(stroke: string): string {
  return `<rect x="0.5" y="0.5" width="${W - 1}" height="${H - 1}" fill="none" stroke="${stroke}" stroke-width="1" stroke-dasharray="2 2"/>`;
}

/** The faded look of a card that never takes a trick. */
const VEIL = `<rect width="${W}" height="${H}" fill="${PAPER}" opacity=".7"/>`;

/** An eye at the bottom left: you see who holds this card. */
const EYE = (paper: string): string => marked([[shape(['.###.', '#####', '.###.'], 2, 43), '#2f86c0'], [rects('4 44 1 1'), '#0b0c0e']], paper);

/** The broken red frame of every curse, on the outermost pixels. */
const CURSE_FRAME =
  draw(rects('0 0 6 1, 8 0 2 1, 12 0 9 1, 23 0 2 1, 27 0 6 1, 0 1 1 6, 0 9 1 2, 0 13 1 8, 0 23 1 3, 0 28 1 8, 0 38 1 2, 0 42 1 7, 32 1 1 6, 32 9 1 2, 32 13 1 8, 32 23 1 3, 32 28 1 8, 32 38 1 2, 32 42 1 7'), CURSE_RED) +
  `<path d="M0 48.5h6v1h-6zM8 48.5h2v1h-2zM12 48.5h9v1h-9zM23 48.5h2v1h-2zM27 48.5h6v1h-6zM0 48h1v.5h-1zM32 48h1v.5h-1z" fill="${CURSE_RED}"/>`;

/** The design per modification id, as chosen in the design review. */
const MOD_ART: Record<string, ModArt> = {
  // Augen: a frame around the whole card, the extra Augen at the bottom.
  fat: { places: ['outline', 'value'], frame: '#d98e0b', value: { fill: '#d98e0b' } },
  gold_rim: { places: ['outline', 'value'], frame: '#b8901c', value: { fill: '#b8901c' } },
  lead_weight: { places: ['outline', 'value'], frame: '#4a4f57', value: { fill: '#4a4f57' } },
  // Rang: chevrons under the index, one per step.
  marked: { places: ['top'], over: (p) => chevrons(1, '#8e2430', p) },
  double_marked: { places: ['top'], over: (p) => chevrons(2, '#8e2430', p) },
  dressed_up: {
    places: ['top', 'corner'],
    over: (p) => chevrons(2, '#b0336e', p) + cornerSign(['#...#', '##.##', '#####', '##.##', '#...#'], '#b0336e', p),
  },
  // Trumpf: a faint seal over the picture, the number at the bottom.
  stamp: { places: ['wash', 'value'], under: () => wash(['..#..', '#####', '.###.', '.#.#.', '#...#'], '#5e3796'), value: { fill: '#5e3796' } },
  crown: { places: ['wash', 'value'], under: () => wash(['#.#.#', '#####', '#####', '#####'], '#c08a12'), value: { fill: '#c08a12' } },
  seal: { places: ['wash', 'value'], under: () => wash(['.###.', '##.##', '#...#', '##.##', '.###.'], '#a3263a'), value: { fill: '#a3263a' } },
  horseshoe: { places: ['wash', 'value'], under: () => wash(['#...#', '#...#', '#...#', '.###.'], '#4f6272'), value: { fill: '#4f6272' } },
  tape_measure: { places: ['wash', 'value'], under: () => wash(['#####', '#.#.#', '#####'], '#b07d0c'), value: { fill: '#b07d0c' } },
  // Letzter Stich: the flag top right with its sign.
  last_trick: { places: ['flag'], over: () => lastTrickFlag('#16707a', ['#....', '###..', '####.', '#....', '#....']) },
  lucky_penny: { places: ['flag'], over: () => lastTrickFlag('#b06a2c', ['.###.', '#.#.#', '#.#.#', '#.#.#', '.###.']) },
  tail_light: { places: ['flag'], over: () => lastTrickFlag('#b3202a', ['.###.', '#####', '#####', '.###.', '#.#.#']) },
  // Sicht und Stich: the whole card tinted or faded.
  transparent: {
    places: ['tint', 'edge', 'corner'],
    under: () => `<rect width="${W}" height="${H}" fill="#8fc7a8" opacity=".28"/>`,
    over: (p) => dashedEdge('#2f86c0') + EYE(p),
  },
  invisible: { places: ['tint', 'edge'], over: () => VEIL + dashedEdge('#7a7f8a') },
  dummy: {
    places: ['tint', 'edge', 'corner', 'center'],
    over: (p) => {
      const ask = new Set([...shape(['.###.', '#...#', '...#.', '..#..', '..#..', '.....', '..#..'], 0, 0)].flatMap((c) => {
        const [x = 0, y = 0] = c.split(',').map(Number);
        return [...rects(`${12 + x * 2} ${17 + y * 2} 2 2`)];
      }));
      return `${VEIL}${dashedEdge('#7a7f8a')}${draw(ask, '#7d6a50')}${EYE(p)}`;
    },
  },
  honorary_jack: {
    places: ['tint', 'corner'],
    paper: '#cfdcec',
    over: (p) => cornerSign(['#...#', '.#.#.', '.###.', '.###.', '..#..'], '#1f5a8c', p),
  },
  // Flüche: the broken red frame (drawn for every curse) and their mark.
  beer_stain: {
    places: ['stain', 'value'],
    over: (p) => marked([[rects('19 15 6 2, 18 17 9 3, 19 20 7 2, 21 22 3 1, 27 14 1 1'), '#c98a2a']], p),
    value: { fill: CURSE_RED },
  },
  cracked: {
    places: ['crack'],
    over: (p) =>
      marked([[rects('13 0 1 5, 14 5 1 3, 12 8 2 1, 11 9 1 5, 12 14 1 1, 13 15 1 4, 14 19 1 4, 13 23 1 3, 12 26 1 5, 13 31 2 1, 15 32 1 5, 14 37 1 4, 13 41 1 9'), '#1d1a18']], p),
  },
  black_cat: {
    places: ['tint', 'corner', 'value'],
    paper: '#d8ccb6',
    under: () =>
      `<rect width="${W}" height="${H}" fill="#5a4a3a" opacity=".18"/>${draw(rects('0 0 4 1, 0 1 3 1, 0 2 2 1, 0 3 1 1, 29 48 4 1, 30 47 3 1, 31 46 2 1, 32 45 1 1'), '#3a3330')}<path d="M29 49h4v.5h-4z" fill="#3a3330"/>`,
    over: (p) => cornerSign(['#...#', '#####', '#.#.#', '#####', '.###.'], '#1d1a18', p),
    value: { fill: CURSE_RED },
  },
  // Ramponiert: a torn corner bottom left, a notch in the right edge.
  battered: {
    places: ['corner', 'notch'],
    over: () =>
      draw(rects('0 44 1 1, 0 45 2 1, 0 46 3 1, 0 47 4 1, 0 48 5 1'), '#1b1e21') +
      `<path d="M0 49h5v.5h-5z" fill="#1b1e21"/>` +
      draw(rects('1 44 1 1, 2 45 1 1, 3 46 1 1, 4 47 1 1, 5 48 1 1, 1 43 1 1, 2 44 1 1, 3 45 1 1, 4 46 1 1, 5 47 1 1, 6 48 1 1'), '#c9c4b6') +
      draw(rects('30 22 3 1, 29 23 4 1, 30 24 3 1'), '#1b1e21'),
  },
  // M8 wave 3 (designs from the vote). Wandelkarte: the four suit
  // colours over the picture, a four-square sign in the corner.
  wild_card: {
    places: ['tint', 'corner'],
    under: () =>
      `<g opacity=".18"><rect width="16.5" height="25" fill="#8b4a1c"/><rect x="16.5" width="16.5" height="25" fill="#2e9a3c"/><rect y="25" width="16.5" height="24.5" fill="#e0281e"/><rect x="16.5" y="25" width="16.5" height="24.5" fill="#f7c21a"/></g>`,
    over: (p) => cornerSign(['##.##', '##.##', '.....', '##.##', '##.##'], '#2e7d4f', p),
  },
  // Glasdaus: glass-blue paper and edge with a glint, "×2" at the bottom.
  glass_ace: {
    places: ['tint', 'edge', 'value'],
    paper: '#e3f1fb',
    under: () => `<rect width="${W}" height="${H}" fill="#9fd3ff" opacity=".3"/>`,
    over: () =>
      `<rect x="0.5" y="0.5" width="${W - 1}" height="${H - 1}" fill="none" stroke="#7fc4e8" stroke-width="1"/>` +
      draw(
        rects(
          '5 35 1 1, 6 34 1 1, 7 33 1 1, 8 32 1 1, 9 31 1 1, 10 30 1 1, 11 29 1 1, 12 28 1 1, 13 27 1 1, 14 26 1 1, 15 25 1 1, 16 24 1 1, ' +
            '8 37 1 1, 9 36 1 1, 10 35 1 1, 11 34 1 1, 12 33 1 1, 13 32 1 1, 14 31 1 1, 15 30 1 1',
        ),
        '#ffffff',
      ),
    value: { fill: '#2f86c0' },
  },
  // Heimweh: a faint house over the picture, the house in the corner.
  homesick: {
    places: ['wash', 'corner'],
    under: () => wash(['..#..', '.###.', '#####', '.#.#.', '.###.'], '#8e2430', 0.2),
    over: (p) => cornerSign(['..#..', '.###.', '#####', '.#.#.', '.###.'], '#8e2430', p),
  },
};

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
  under: string;
  over: string;
  /** The modification whose frame lies around the card. */
  frame: string | null;
  badges: string[];
  curse: boolean;
}

/** Lay out the modifications of one card: each at its places, or as a badge. */
function layoutMods(mods: string[]): Layers {
  const used = new Set<Place>();
  const out: Layers = { paper: PAPER, under: '', over: '', frame: null, badges: [], curse: mods.some(isCurse) };
  const placed: { art: ModArt; figure: string }[] = [];
  for (const mod of mods) {
    const art = MOD_ART[mod];
    const figure = art?.value ? figureText(modFigure(mod)) : '';
    // A design without its number does not need the place for it.
    const places = art ? art.places.filter((p) => p !== 'value' || figure) : [];
    if (!art || places.some((p) => used.has(p))) {
      out.badges.push(mod);
      continue;
    }
    for (const p of places) used.add(p);
    placed.push({ art, figure });
    if (art.paper && out.paper === PAPER) out.paper = art.paper;
    if (art.frame && !out.frame) out.frame = mod;
  }
  let values = '';
  for (const { art, figure } of placed) {
    out.under += art.under?.() ?? '';
    out.over += art.over?.(out.paper) ?? '';
    // The printed numbers go over every mark.
    if (art.value && figure) {
      values += pixelNumber(figure, 42, art.value.fill, out.paper);
    }
  }
  out.over += values;
  // Every curse shows the broken red frame, also when it is a badge.
  if (out.curse) out.over += CURSE_FRAME;
  return out;
}

/** Size variant of a face: the full picture, or index and sign for tiny cards. */
export type FaceSize = 'full' | 'small';

/**
 * The face of a card as inline SVG (viewBox 33 × 49.5 art pixels). `size`
 * "small" is for tiny cards (about 16 to 26 px wide): the index letter and
 * the suit sign, big, with the paper tint, the Augen frame and the curse
 * frame of the modifications but without their marks.
 */
export function cardSvg(kind: CardKind, mods: string[] = [], size: FaceSize = 'full'): string {
  const f = face(kind);
  const modNames = mods.map((m) => (isCurse(m) ? `${contentName('card_mods', m)} (${de.card.curse})` : contentName('card_mods', m))).join(', ');
  const label = esc(cardName(kind) + (mods.length ? ', ' + t(de.card.mods, { mods: modNames }) : ''));
  const layers = layoutMods(mods);
  const frame = layers.frame ? ` data-frame="${esc(layers.frame)}"` : '';
  const paper = `<rect width="${W}" height="${H}" fill="${layers.paper}"/>`;
  let body: string;
  if (size === 'small') {
    body = f ? `${paper}<use href="#${smallFaceId(f.suit, f.rank)}"/>` : `<use href="#f-blank"/>`;
    if (layers.curse) body += CURSE_FRAME;
  } else {
    const badges = layers.badges.map((m, i) => `<use href="#${modIconId(m)}" x="1" y="${32 - i * 9}" width="8" height="8"/>`).join('');
    body = f
      ? `${paper}<use href="#${faceId(f.suit, f.rank)}"/>${layers.under}<use href="#${indexId(f.suit, f.rank)}"/>`
      : `<use href="#f-blank"/>${layers.under}`;
    body += layers.over + badges;
  }
  return `<svg class="card-svg${size === 'small' ? ' is-small' : ''}" viewBox="0 0 ${W} ${H}" preserveAspectRatio="none" shape-rendering="crispEdges" role="img" aria-label="${label}"${frame}>${body}</svg>`;
}

/**
 * Back of a card in one of the back designs (a cosmetic setting, ./backs.ts).
 * `size` "small" picks the simplified picture for tiny backs.
 */
export function cardBack(className = '', design: BackDesign = DEFAULT_BACK, size: BackSize = 'full'): string {
  return `<span class="card-back ${className}" role="img" aria-label="${esc(de.card.back)}"><svg viewBox="0 0 120 180" shape-rendering="crispEdges" aria-hidden="true"><use href="#${backId(design, size)}"/></svg></span>`;
}

// Cards as small SVGs that reference the sprite in ./art.ts. A face is
// drawn in layers: paper (tinted by some modifications), the picture, the
// washes and tints of modifications, the corner index, then the marks of
// modifications on top. Every modification has the design the player chose
// (pixel art on the 30 × 45 grid of the designs, drawn × 4) at fixed places
// of the card; when two want the same place, the later one falls back to a
// small badge bottom left. The Augen frame lies outside the card (a CSS
// outline, see `data-frame` in style.css), so it never covers the index.
// Pure drawing, no game logic: the printed numbers come from the core's
// catalog.

import { de, cardName, contentName, face, factorText, isCurse, modFigure, t } from '../i18n';
import type { CardKind, ModFigure, Suit } from '../types';
import { faceId, indexId, modIconId, paper, PALETTE, pixels, suitId } from './art';
import { backId, DEFAULT_BACK, type BackDesign, type BackSize } from './backs';
import { esc } from './dom';

/** Ink colour per suit, for text next to suit icons. */
export const SUIT_COLORS: Record<Suit, string> = {
  Clubs: PALETTE.Clubs.ink,
  Spades: PALETTE.Spades.ink,
  Hearts: PALETTE.Hearts.ink,
  Diamonds: PALETTE.Diamonds.ink,
};

/** A suit symbol as a standalone inline SVG. */
export function suitIcon(suit: Suit, size = 24, label: string | null = de.suits[suit]): string {
  const a11y = label ? `role="img" aria-label="${esc(label)}"` : 'aria-hidden="true"';
  return `<svg class="suit-icon" viewBox="0 0 8 8" width="${size}" height="${size}" shape-rendering="crispEdges" ${a11y}><use href="#${suitId(suit)}"/></svg>`;
}

/** A card modification icon (badge) as inline SVG. */
export function modIcon(mod: string, size = 20): string {
  return `<svg class="mod-icon" viewBox="0 0 24 24" width="${size}" height="${size}" aria-hidden="true"><use href="#${modIconId(mod)}"/></svg>`;
}

// ------------------------------------------------------- modification art

/** One pixel of the design grid (30 × 45) in card units (120 × 180). */
const G = 4;
const PAPER = '#ecebe4';
const CURSE_RED = '#bd3b32';
const PIXEL_FONT = "'VT323', monospace";

/**
 * Where a modification draws. Two modifications with a common place do not
 * both fit: the later one becomes a badge.
 */
type Place = 'outline' | 'value' | 'wash' | 'tint' | 'top' | 'corner' | 'edge' | 'center' | 'stain' | 'crack' | 'notch';

interface ModArt {
  places: Place[];
  /** Colour of the paper instead of the plain one. */
  paper?: string;
  /** Colour of the frame outside the card (CSS outline). */
  frame?: string;
  /** Over the picture, under the corner index. */
  under?: () => string;
  /** Over everything. */
  over?: () => string;
  /** Colour and size of the printed number, centred at the bottom. */
  value?: { fill: string; size: number };
}

/** Rectangles on the design grid, "x y w h" separated by commas. */
function grid(rects: string, fill: string): string {
  const d = rects
    .split(',')
    .map((r) => {
      const [x = 0, y = 0, w = 0, h = 0] = r.trim().split(/\s+/).map(Number);
      return `M${x * G} ${y * G}h${w * G}v${h * G}h${-w * G}z`;
    })
    .join('');
  return `<path d="${d}" fill="${fill}"/>`;
}

/** A small pixel icon on the design grid, top left at (x, y). */
function icon(rows: string[], x: number, y: number, fill: string): string {
  return pixels(rows, x * G, y * G, G, fill);
}

/** A big faint pixel shape over the picture: 16-unit blocks from (20, 52). */
function wash(rows: string[], fill: string, opacity = 0.22): string {
  return `<g opacity="${opacity}">${pixels(rows, 20, 52, 16, fill)}</g>`;
}

/** Arrows next to the index: one chevron, 2 pixels thick, per step. */
const CHEVRON = ['...##...', '..####..', '.##..##.', '##....##'];
function chevrons(n: number, fill: string): string {
  return Array.from({ length: n }, (_, i) => icon(CHEVRON, 11, 2 + i * 4, fill)).join('');
}

/** The corner of the last trick: a stepped triangle top right, big enough for its sign. */
function lastTrickCorner(fill: string, sign: string[]): string {
  let d = 'M52 0h68v4h-68z';
  for (let k = 1; k <= 17; k++) {
    const x = (12 + k) * G;
    d += `M${x} ${k * G}h${120 - x}v${G}h${x - 120}z`;
  }
  return `<path d="${d}" fill="${fill}"/>${icon(sign, 23, 2, '#ffffff')}`;
}

/** A dashed line just inside the card's edge. */
function dashedEdge(stroke: string): string {
  return `<rect x="6" y="6" width="108" height="168" fill="none" stroke="${stroke}" stroke-width="4" stroke-dasharray="8 8"/>`;
}

/** The faded look of a card that never takes a trick. */
const VEIL = `<rect width="120" height="180" fill="${PAPER}" opacity=".7"/>`;

/** An eye top right: you see who holds this card. */
const EYE = icon(['.###.', '#####', '.###.'], 23, 3, '#2f86c0') + grid('25 4 1 1', '#0b0c0e');

/** The broken red frame of every curse. */
const CURSE_FRAME = grid(
  '1 1 6 1, 9 1 2 1, 13 1 6 1, 21 1 2 1, 25 1 4 1, 1 43 6 1, 9 43 2 1, 13 43 6 1, 21 43 2 1, 25 43 4 1, ' +
    '1 3 1 6, 1 11 1 2, 1 15 1 6, 1 23 1 2, 1 27 1 6, 1 35 1 2, 1 39 1 3, 28 3 1 6, 28 11 1 2, 28 15 1 6, 28 23 1 2, 28 27 1 6, 28 35 1 2, 28 39 1 3',
  CURSE_RED,
);

/**
 * The design per modification id, as chosen in the design review. Ids
 * without content yet (wandelkarte, glasdaus, heimweh) are drawn already,
 * so they show as soon as the core has them.
 */
const MOD_ART: Record<string, ModArt> = {
  // Augen: a frame around the whole card, the extra Augen at the bottom.
  fat: { places: ['outline', 'value'], frame: '#d98e0b', value: { fill: '#d98e0b', size: 36 } },
  gold_rim: { places: ['outline', 'value'], frame: '#b8901c', value: { fill: '#b8901c', size: 36 } },
  lead_weight: { places: ['outline', 'value'], frame: '#4a4f57', value: { fill: '#4a4f57', size: 36 } },
  // Rang: arrows next to the index, one per step.
  marked: { places: ['top'], over: () => chevrons(1, '#8e2430') },
  double_marked: { places: ['top'], over: () => chevrons(2, '#8e2430') },
  dressed_up: { places: ['top', 'corner'], over: () => chevrons(2, '#b0336e') + grid('21 3 2 3, 25 3 2 3, 23 4 2 1', '#b0336e') },
  // Trumpf: a faint seal over the picture, the number at the bottom.
  stamp: { places: ['wash', 'value'], under: () => wash(['..#..', '#####', '.###.', '.#.#.', '#...#'], '#5e3796'), value: { fill: '#5e3796', size: 32 } },
  crown: { places: ['wash', 'value'], under: () => wash(['#.#.#', '#####', '#####', '#####'], '#c08a12'), value: { fill: '#c08a12', size: 32 } },
  seal: { places: ['wash', 'value'], under: () => wash(['.###.', '##.##', '#...#', '##.##', '.###.'], '#a3263a'), value: { fill: '#a3263a', size: 32 } },
  horseshoe: { places: ['wash', 'value'], under: () => wash(['#...#', '#...#', '#...#', '.###.'], '#4f6272'), value: { fill: '#4f6272', size: 32 } },
  tape_measure: { places: ['wash', 'value'], under: () => wash(['#####', '#.#.#', '#####'], '#b07d0c'), value: { fill: '#b07d0c', size: 32 } },
  // Letzter Stich: the corner top right with its sign.
  last_trick: { places: ['corner', 'top'], over: () => lastTrickCorner('#16707a', ['#....', '###..', '####.', '#....', '#....']) },
  lucky_penny: { places: ['corner', 'top'], over: () => lastTrickCorner('#b06a2c', ['.###.', '#.#.#', '#.#.#', '#.#.#', '.###.']) },
  tail_light: { places: ['corner', 'top'], over: () => lastTrickCorner('#b3202a', ['.###.', '#####', '#####', '.###.', '#.#.#']) },
  // Sicht und Stich: the whole card tinted or faded.
  transparent: {
    places: ['tint', 'edge', 'corner'],
    under: () => `<rect width="120" height="180" fill="#8fc7a8" opacity=".28"/>`,
    over: () => dashedEdge('#2f86c0') + EYE,
  },
  invisible: { places: ['tint', 'edge'], over: () => VEIL + dashedEdge('#7a7f8a') },
  dummy: {
    places: ['tint', 'edge', 'corner', 'center'],
    over: () => `${VEIL}${dashedEdge('#7a7f8a')}<text x="44" y="116" font-family="${PIXEL_FONT}" font-size="64" fill="#7d6a50">?</text>${EYE}`,
  },
  honorary_jack: {
    places: ['tint', 'corner'],
    paper: '#cfdcec',
    over: () => icon(['#...#', '.#.#.', '.###.', '.###.', '..#..'], 23, 2, '#1f5a8c'),
  },
  // Flüche: the broken red frame (drawn for every curse) and their mark.
  beer_stain: {
    places: ['stain', 'value'],
    over: () => grid('18 8 6 2, 17 10 9 3, 18 13 7 2, 20 15 3 1, 26 7 1 1', '#c98a2a'),
    value: { fill: CURSE_RED, size: 32 },
  },
  cracked: {
    places: ['crack'],
    over: () =>
      grid('12 0 1 5, 13 5 1 3, 11 8 2 1, 10 9 1 4, 11 13 1 1, 12 14 1 3, 13 17 1 4, 12 21 1 3, 11 24 1 4, 12 28 2 1, 14 29 1 5, 13 34 1 4, 12 38 1 7', '#1d1a18'),
  },
  black_cat: {
    places: ['tint', 'corner', 'value'],
    paper: '#d8ccb6',
    under: () =>
      `<rect width="120" height="180" fill="#5a4a3a" opacity=".18"/><path d="M0 0h20v8h-20zM0 8h12v4h-12zM0 12h8v4h-8zM100 172h20v8h-20zM108 168h12v4h-12zM112 164h8v4h-8z" fill="#3a3330"/>`,
    over: () => icon(['#...#', '#####', '#.#.#', '#####', '.###.'], 23, 2, '#1d1a18'),
    value: { fill: CURSE_RED, size: 32 },
  },
  battered: {
    places: ['corner', 'notch'],
    over: () =>
      `<path d="M100 0h20v8h-20zM104 8h16v4h-16zM108 12h12v4h-12zM112 16h8v4h-8z" fill="#1b1e21"/>` +
      `<path d="M96 4h4v4h-4zM96 8h8v4h-8zM96 12h12v4h-12zM100 16h12v4h-12z" fill="#c9c4b6"/>` +
      `<path d="M0 80h12v4h-12zM0 84h16v4h-16zM0 88h12v4h-12z" fill="#1d1a18"/>`,
  },
  // New from the vote, drawn ahead of their content.
  wandelkarte: {
    places: ['tint', 'corner'],
    under: () =>
      `<g opacity=".18"><rect width="60" height="92" fill="#8a5a2b"/><rect x="60" width="60" height="92" fill="#2e7d4f"/><rect y="92" width="60" height="88" fill="#bd3b32"/><rect x="60" y="92" width="60" height="88" fill="#b8840f"/></g>`,
    over: () => icon(['##.##', '##.##', '.....', '##.##', '##.##'], 23, 2, '#2e7d4f'),
  },
  glasdaus: {
    places: ['tint', 'edge', 'value'],
    paper: '#e3f1fb',
    under: () => `<rect width="120" height="180" fill="#9fd3ff" opacity=".3"/>`,
    over: () =>
      grid('1 1 28 1, 1 43 28 1, 1 1 1 43, 28 1 1 43', '#7fc4e8') +
      grid(
        '4 32 1 1, 5 31 1 1, 6 30 1 1, 7 29 1 1, 8 28 1 1, 9 27 1 1, 10 26 1 1, 11 25 1 1, 12 24 1 1, 13 23 1 1, 14 22 1 1, 15 21 1 1, ' +
          '7 33 1 1, 8 32 1 1, 9 31 1 1, 10 30 1 1, 11 29 1 1, 12 28 1 1, 13 27 1 1, 14 26 1 1',
        '#ffffff',
      ),
    value: { fill: '#2f86c0', size: 36 },
  },
  heimweh: {
    places: ['wash', 'corner'],
    under: () => wash(['..#..', '.###.', '#####', '.#.#.', '.###.'], '#8e2430', 0.2),
    over: () => icon(['..#..', '.###.', '#####', '.#.#.', '.###.'], 23, 2, '#8e2430'),
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
}

/** Lay out the modifications of one card: each at its places, or as a badge. */
function layoutMods(mods: string[]): Layers {
  const used = new Set<Place>();
  const out: Layers = { paper: PAPER, under: '', over: '', frame: null, badges: [] };
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
  for (const { art, figure } of placed) {
    out.under += art.under?.() ?? '';
    out.over += art.over?.() ?? '';
    if (art.value && figure) {
      out.over += `<text x="60" y="170" text-anchor="middle" font-family="${PIXEL_FONT}" font-size="${art.value.size}" fill="${art.value.fill}" stroke="${out.paper}" stroke-width="6" stroke-linejoin="round" paint-order="stroke">${esc(figure)}</text>`;
    }
  }
  // Every curse shows the broken red frame, also when it is a badge.
  if (mods.some(isCurse)) out.over += CURSE_FRAME;
  return out;
}

/** The face of a card as inline SVG (viewBox 120 × 180). */
export function cardSvg(kind: CardKind, mods: string[] = []): string {
  const f = face(kind);
  const modNames = mods.map((m) => (isCurse(m) ? `${contentName('card_mods', m)} (${de.card.curse})` : contentName('card_mods', m))).join(', ');
  const label = esc(cardName(kind) + (mods.length ? ', ' + t(de.card.mods, { mods: modNames }) : ''));
  const layers = layoutMods(mods);
  const badges = layers.badges.map((m, i) => `<use href="#${modIconId(m)}" x="6" y="${146 - i * 30}" width="28" height="28"/>`).join('');
  const frame = layers.frame ? ` data-frame="${esc(layers.frame)}"` : '';
  const body = f
    ? `${paper(layers.paper)}<use href="#${faceId(f.suit, f.rank)}"/>${layers.under}<use href="#${indexId(f.suit, f.rank)}"/>`
    : `<use href="#f-blank"/>${layers.under}`;
  return `<svg class="card-svg" viewBox="0 0 120 180" shape-rendering="crispEdges" role="img" aria-label="${label}"${frame}>${body}${layers.over}${badges}</svg>`;
}

/**
 * Back of a card in one of the back designs (a cosmetic setting, ./backs.ts).
 * `size` "small" picks the simplified picture for tiny backs.
 */
export function cardBack(className = '', design: BackDesign = DEFAULT_BACK, size: BackSize = 'full'): string {
  return `<span class="card-back ${className}" role="img" aria-label="${esc(de.card.back)}"><svg viewBox="0 0 120 180" shape-rendering="crispEdges" aria-hidden="true"><use href="#${backId(design, size)}"/></svg></span>`;
}

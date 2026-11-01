// The German deck in the pixel style "Kupfer und Schiefer" (ADR 0010,
// DESIGN 16): paper cards with a big corner index and the four suits as
// 8 × 8 pixel glyphs. Everything is built once into a hidden sprite of
// <symbol>s; cards then only reference the symbols with <use>, which keeps
// the DOM small. Pure drawing, no rules.
//
// Coordinates: suit symbols in an 8 × 8 box, cards in 120 × 180. The card's
// outer frame (thick dark line, hard pixel shadow) comes from CSS, so it
// stays crisp at every size.

import { de } from '../i18n';
import type { Rank, Suit } from '../types';
import { peopleSymbols } from './people';

export const SUITS: Suit[] = ['Clubs', 'Spades', 'Hearts', 'Diamonds'];
export const RANKS: Rank[] = ['7', '8', '9', 'T', 'J', 'Q', 'K', 'A'];

const CODE: Record<Suit, string> = { Clubs: 'C', Spades: 'S', Hearts: 'H', Diamonds: 'D' };

interface Palette {
  /** Corner index and the main colour of the glyph. */
  ink: string;
  /** Second colour of the glyph (the acorn's cap). */
  ink2: string;
}

/** Four clearly different hues: brown, green, red, ochre. */
export const PALETTE: Record<Suit, Palette> = {
  Clubs: { ink: '#8a5a2b', ink2: '#4a2c12' },
  Spades: { ink: '#2e7d4f', ink2: '#2e7d4f' },
  Hearts: { ink: '#bd3b32', ink2: '#bd3b32' },
  Diamonds: { ink: '#b8840f', ink2: '#b8840f' },
};

/** Darker ink for the corner index of the acorn (reads better on paper). */
const INDEX_INK: Record<Suit, string> = {
  Clubs: '#5e3a17',
  Spades: '#2e7d4f',
  Hearts: '#bd3b32',
  Diamonds: '#9a6c06',
};

const PAPER = '#ecebe4';
const PAPER_LINE = '#cfccc0';
const LINE = '#0b0c0e';
const SLATE = '#2a2f34';
const SLATE_DARK = '#23282c';
const COPPER = '#d1773f';
const PIXEL_FONT = "'VT323', monospace";

// ------------------------------------------------------------ suit glyphs

/** The suit glyphs on an 8 × 8 grid: main shape and second colour. */
const GLYPH: Record<Suit, { main: string; second: string }> = {
  // Eichel: an acorn, dark cap over a brown nut.
  Clubs: {
    second: 'M3 0h1v1h-1zM1 1h6v1h-6zM0 2h8v1h-8z',
    main: 'M1 3h6v1h-6zM1 4h6v1h-6zM2 5h4v1h-4zM2 6h4v1h-4zM3 7h2v1h-2z',
  },
  // Grün: a leaf, tip up, with a short stem.
  Spades: {
    second: '',
    main: 'M4 0h1v1h-1zM3 1h3v1h-3zM2 2h5v1h-5zM1 3h6v1h-6zM1 4h6v1h-6zM2 5h4v1h-4zM3 6h2v1h-2zM3 7h1v1h-1z',
  },
  // Rot: a heart.
  Hearts: {
    second: '',
    main: 'M1 0h2v1h-2zM5 0h2v1h-2zM0 1h8v1h-8zM0 2h8v1h-8zM0 3h8v1h-8zM1 4h6v1h-6zM2 5h4v1h-4zM3 6h2v1h-2z',
  },
  // Schellen: a sleigh bell with its slit.
  Diamonds: {
    second: '',
    main: 'M3 0h2v1h-2zM2 1h4v1h-4zM1 2h6v1h-6zM1 3h6v1h-6zM0 4h8v1h-8zM0 5h8v1h-8zM1 6h1v1h-1zM3 6h2v1h-2zM6 6h1v1h-1zM3 7h2v1h-2z',
  },
};

/** The glyph's paths in its 8 × 8 box. */
function glyphPaths(suit: Suit): string {
  const g = GLYPH[suit];
  const p = PALETTE[suit];
  return `${g.second ? `<path d="${g.second}" fill="${p.ink2}"/>` : ''}<path d="${g.main}" fill="${p.ink}"/>`;
}

/** A glyph placed at (x, y) with `size` units per side. */
function glyph(suit: Suit, x: number, y: number, size: number): string {
  return `<use href="#s-${CODE[suit]}" x="${x}" y="${y}" width="${size}" height="${size}"/>`;
}

/** A pixel shape from rows of "#" and ".", at (x, y) with `u` units per pixel. */
export function pixels(rows: string[], x: number, y: number, u: number, fill: string): string {
  let d = '';
  rows.forEach((row, j) => {
    let i = 0;
    while (i < row.length) {
      if (row[i] !== '#') {
        i++;
        continue;
      }
      let k = i;
      while (k < row.length && row[k] === '#') k++;
      d += `M${x + i * u} ${y + j * u}h${(k - i) * u}v${u}h${-(k - i) * u}z`;
      i = k;
    }
  });
  return d ? `<path d="${d}" fill="${fill}"/>` : '';
}

// --------------------------------------------------------------- the faces

/** Centre line of the corner index. */
const INDEX_X = 23;

/**
 * Corner index: the rank, big, centred above the small glyph, both in the
 * left column, so the top middle and the top right stay free for the
 * marks of card modifications (./cards.ts).
 */
function index(suit: Suit, rank: Rank): string {
  // VT323 is narrow (0.4 em per letter): the 10 fits at the same size as a single letter.
  return `<text x="${INDEX_X}" y="40" text-anchor="middle" font-family="${PIXEL_FONT}" font-size="50" fill="${INDEX_INK[suit]}" class="idx">${de.rankShort[rank]}</text>
${glyph(suit, INDEX_X - 12, 46, 24)}`;
}

/**
 * A pixel frame around the middle of a court card: below the corner index,
 * above the bottom strip where card modifications print their number.
 */
function courtFrame(suit: Suit): string {
  const ink = PALETTE[suit].ink;
  return `<path d="M16 76h88v4h-88zM16 146h88v4h-88zM16 80h4v66h-4zM100 80h4v66h-4z" fill="${ink}"/>`;
}

/** A small pixel crown for the König. */
const CROWN = ['#..#..#', '#######', '#.#.#.#', '#######'];
/** The Ober holds his suit high: a raised arm under the glyph. */
const ARM_UP = ['..##..', '.####.', '##..##'];
/** The Unter holds his suit low: a hand on top of the glyph. */
const ARM_DOWN = ['##..##', '.####.', '..##..'];

function courtFace(suit: Suit, rank: 'K' | 'Q' | 'J'): string {
  const ink = PALETTE[suit].ink;
  const frame = courtFrame(suit);
  if (rank === 'K') {
    return `${frame}${pixels(CROWN, 43, 84, 5, '#c9962a')}${glyph(suit, 40, 104, 40)}`;
  }
  if (rank === 'Q') {
    // Ober: the glyph high in the frame, an arm holding it up.
    return `${frame}${glyph(suit, 40, 84, 40)}${pixels(ARM_UP, 45, 126, 5, ink)}`;
  }
  // Unter: the glyph low in the frame, a hand resting on it.
  return `${frame}${pixels(ARM_DOWN, 45, 85, 5, ink)}${glyph(suit, 40, 102, 40)}`;
}

/** The Daus: a big glyph on a copper ribbon. */
function aceFace(suit: Suit): string {
  return `${glyph(suit, 28, 64, 64)}
<path d="M20 132h80v16h-80z" fill="${COPPER}"/><path d="M12 136h8v12h-8zM100 136h8v12h-8z" fill="#9c5228"/>
<path d="M28 138h64v4h-64z" fill="${PAPER}" opacity=".55"/>`;
}

function numberFace(suit: Suit): string {
  return glyph(suit, 32, 78, 56);
}

/** The paper of a card face, optionally tinted (card modifications). */
export function paper(fill: string = PAPER): string {
  return `<rect width="120" height="180" fill="${fill}"/><path d="M4 4h112v2h-112zM4 174h112v2h-112zM4 6h2v168h-2zM114 6h2v168h-2z" fill="${PAPER_LINE}"/>`;
}

/** The picture in the middle of a face, without paper and index. */
function faceArt(suit: Suit, rank: Rank): string {
  if (rank === 'A') return aceFace(suit);
  if (rank === 'K' || rank === 'Q' || rank === 'J') return courtFace(suit, rank);
  return numberFace(suit);
}

// ------------------------------------------------------------- other faces

function backBody(): string {
  const emblem = SUITS.map((s, i) => {
    const pos = [
      [44, 70],
      [64, 70],
      [44, 94],
      [64, 94],
    ][i] ?? [44, 70];
    return glyph(s, pos[0] ?? 44, pos[1] ?? 70, 16);
  }).join('');
  return `<rect width="120" height="180" fill="${SLATE}"/>
<rect x="8" y="8" width="104" height="164" fill="url(#back-lattice)"/>
<path d="M8 8h104v4h-104zM8 168h104v4h-104zM8 12h4v156h-4zM108 12h4v156h-4z" fill="${COPPER}"/>
<rect x="36" y="62" width="48" height="56" fill="${PAPER}"/>
<path d="M36 62h48v4h-48zM36 114h48v4h-48zM36 66h4v48h-4zM80 66h4v48h-4z" fill="${LINE}"/>
${emblem}`;
}

function blankBody(): string {
  return `<rect width="120" height="180" fill="#dcd9cf"/>
<rect x="10" y="10" width="100" height="160" fill="url(#blank-hatch)"/>
<rect x="14" y="76" width="92" height="30" fill="#dcd9cf"/>
<text x="60" y="98" text-anchor="middle" font-family="${PIXEL_FONT}" font-size="25" fill="#6f7479" class="idx">${de.card.blank}</text>`;
}

// --------------------------------------------------------- modifications

/**
 * Badge icons of the card modifications, in a 24 × 24 box. Curses get a
 * black square with a red rim, so they read as bad luck at a glance.
 */
const CURSE_FILL = '#1d1a18';
const CURSE_RING = '#e0505a';
const MOD_ICON: Record<string, { fill: string; glyph: string; ring?: string }> = {
  // Fett: more points.
  fat: { fill: '#d98e0b', glyph: '<path d="M12 6 V18 M6 12 H18" stroke="#fff" stroke-width="3.6" stroke-linecap="round"/>' },
  // Gezinkt: one rank higher.
  marked: { fill: '#8e2430', glyph: '<path d="M6.5 15 L12 8.5 L17.5 15" fill="none" stroke="#fff" stroke-width="3.2" stroke-linecap="round" stroke-linejoin="round"/>' },
  // Stempel: an inked star.
  stamp: { fill: '#5e3796', glyph: '<path d="M12 5 L14 10 L19.2 10.2 L15.1 13.4 L16.5 18.5 L12 15.6 L7.5 18.5 L8.9 13.4 L4.8 10.2 L10 10 Z" fill="#fff"/>' },
  // Spitze: a pennant for the last trick.
  last_trick: { fill: '#16707a', glyph: '<path d="M8 19 V5 M8 5.5 L17.5 9 L8 12.5" fill="#fff" stroke="#fff" stroke-width="2" stroke-linejoin="round" stroke-linecap="round"/>' },
  // Durchsichtig: an eye.
  transparent: { fill: '#2f86c0', glyph: '<path d="M4 12 C7 7 17 7 20 12 C17 17 7 17 4 12 Z" fill="none" stroke="#fff" stroke-width="2"/><circle cx="12" cy="12" r="2.6" fill="#fff"/>' },
  // Unsichtbar: a dashed outline, nothing inside.
  invisible: { fill: '#7a7f8a', glyph: '<circle cx="12" cy="12" r="6" fill="none" stroke="#fff" stroke-width="2.2" stroke-dasharray="2.6 2.2"/>' },
  // Goldrand: many more points, a golden plus in a ring.
  gold_rim: { fill: '#b8901c', glyph: '<circle cx="12" cy="12" r="7.2" fill="none" stroke="#fff6cf" stroke-width="1.6"/><path d="M12 8 V16 M8 12 H16" stroke="#fff" stroke-width="2.8" stroke-linecap="round"/>' },
  // Doppelt gezinkt: two ranks higher.
  double_marked: { fill: '#6e1622', glyph: '<path d="M7 12.5 L12 7.5 L17 12.5 M7 17 L12 12 L17 17" fill="none" stroke="#fff" stroke-width="2.6" stroke-linecap="round" stroke-linejoin="round"/>' },
  // Krone: the Grundwert doubles when trump.
  crown: { fill: '#c08a12', glyph: '<path d="M5.5 16.5 L6.5 8.5 L10 12 L12 7 L14 12 L17.5 8.5 L18.5 16.5 Z" fill="#fff" stroke="#fff" stroke-width="1" stroke-linejoin="round"/>' },
  // Siegel: a wax seal with ribbons.
  seal: { fill: '#a3263a', glyph: '<path d="M9.5 14 L8 20 L10.5 18.5 L12 20.5 M14.5 14 L16 20 L13.5 18.5 L12 20.5" fill="#fff" stroke="#fff" stroke-width="1" stroke-linejoin="round"/><circle cx="12" cy="10" r="5" fill="#fff"/><circle cx="12" cy="10" r="2.4" fill="#a3263a"/>' },
  // Hufeisen: luck, won with fewer Augen.
  horseshoe: { fill: '#4f6272', glyph: '<path d="M7.5 6.5 V12 A4.5 4.5 0 0 0 16.5 12 V6.5" fill="none" stroke="#fff" stroke-width="3" stroke-linecap="round"/><circle cx="7.5" cy="10" r=".9" fill="#4f6272"/><circle cx="16.5" cy="10" r=".9" fill="#4f6272"/>' },
  // Maßband: Schneider measured generously.
  tape_measure: { fill: '#d29a18', glyph: '<rect x="4.5" y="9" width="15" height="6" rx="1" fill="#fff"/><path d="M7 9 V11.5 M9.5 9 V12.5 M12 9 V11.5 M14.5 9 V12.5 M17 9 V11.5" stroke="#d29a18" stroke-width="1.2"/>' },
  // Glückspfennig: a lucky coin.
  lucky_penny: { fill: '#b06a2c', glyph: '<circle cx="12" cy="12" r="6.2" fill="#fff"/><circle cx="12" cy="12" r="4.4" fill="none" stroke="#b06a2c" stroke-width="1.2"/><path d="M11 10 L12.6 9.2 V15" fill="none" stroke="#b06a2c" stroke-width="1.6"/>' },
  // Fluch Bierfleck: a beer stain.
  beer_stain: { fill: CURSE_FILL, ring: CURSE_RING, glyph: '<path d="M12 5.5 C14.5 5.5 14.6 7.6 16.6 8 C19 8.5 18.6 11.5 17.6 12.8 C16.8 14 18.8 16 16.4 17.6 C14.6 18.8 13.2 17.2 11.4 18.2 C9 19.5 6.4 17.8 6.8 15.6 C7.1 14 5 13 5.8 10.6 C6.5 8.4 9.2 9 9.8 7.4 C10.3 6.2 10.8 5.5 12 5.5 Z" fill="#c98a2a"/><circle cx="18.6" cy="6" r="1.1" fill="#c98a2a"/>' },
  // Fluch Angeknackst: a crack.
  cracked: { fill: CURSE_FILL, ring: CURSE_RING, glyph: '<path d="M10 4.5 L13 9 L9.5 12 L14 15.5 L11.5 19.5" fill="none" stroke="#fff" stroke-width="2.2" stroke-linejoin="round" stroke-linecap="round"/>' },
  // Fluch Schwarze Katze: a cat's head with yellow eyes.
  black_cat: { fill: CURSE_FILL, ring: CURSE_RING, glyph: '<path d="M6.5 18 C5.5 13 6 10 7 6 L10 9 C11.3 8.6 12.7 8.6 14 9 L17 6 C18 10 18.5 13 17.5 18 Z" fill="#3a3330" stroke="#fff" stroke-width="1.2" stroke-linejoin="round"/><ellipse cx="10" cy="13" rx="1.3" ry="1.6" fill="#f2d24a"/><ellipse cx="14" cy="13" rx="1.3" ry="1.6" fill="#f2d24a"/>' },
  // Fluch Ramponiert: a dog-eared, torn card that stays in the hand.
  battered: { fill: CURSE_FILL, ring: CURSE_RING, glyph: '<path d="M7.5 5 H13.5 L16.5 8 V19 H7.5 Z" fill="none" stroke="#fff" stroke-width="1.8" stroke-linejoin="round"/><path d="M13.5 5 V8 H16.5" fill="#fff" stroke="#fff" stroke-width="1.2" stroke-linejoin="round"/><path d="M7.5 12.5 L10 11.5 L11.5 13.5 L14 12 L16.5 13" fill="none" stroke="#e0505a" stroke-width="1.8" stroke-linejoin="round" stroke-linecap="round"/>' },
  // Unter ehrenhalber: a medal on a ribbon, a trump by honour.
  honorary_jack: { fill: '#1f5a8c', glyph: '<path d="M9 4.5 L11 10 M15 4.5 L13 10" stroke="#e2b04a" stroke-width="2.4" stroke-linecap="round"/><circle cx="12" cy="14" r="5" fill="#e2b04a"/><path d="M12 10.8 L13 13.1 L15.4 13.2 L13.5 14.7 L14.2 17 L12 15.7 L9.8 17 L10.5 14.7 L8.6 13.2 L11 13.1 Z" fill="#fff"/>' },
  // Bleigewicht: a heavy weight.
  lead_weight: { fill: '#4a4f57', glyph: '<path d="M8.5 9.5 H15.5 L18 18 H6 Z" fill="#fff" stroke="#fff" stroke-width="1" stroke-linejoin="round"/><circle cx="12" cy="7.2" r="2.2" fill="none" stroke="#fff" stroke-width="1.6"/>' },
  // Schlusslicht: a red lamp for the last trick of a Null.
  tail_light: { fill: '#b3202a', glyph: '<circle cx="12" cy="12" r="3.6" fill="#fff"/><path d="M12 4.8 V6.6 M12 17.4 V19.2 M4.8 12 H6.6 M17.4 12 H19.2 M6.9 6.9 L8.1 8.1 M15.9 15.9 L17.1 17.1 M17.1 6.9 L15.9 8.1 M8.1 15.9 L6.9 17.1" stroke="#fff" stroke-width="1.6" stroke-linecap="round"/>' },
  // Aufgebrezelt: a bow tie.
  dressed_up: { fill: '#b0336e', glyph: '<path d="M4.5 8 L11 11 V13 L4.5 16 Z M19.5 8 L13 11 V13 L19.5 16 Z" fill="#fff"/><rect x="10.6" y="10.4" width="2.8" height="3.2" rx=".8" fill="#fff"/>' },
  // Wandelkarte: four squares, one per suit.
  wandelkarte: { fill: '#2e7d4f', glyph: '<path d="M6 6h5v5h-5zM13 6h5v5h-5zM6 13h5v5h-5zM13 13h5v5h-5z" fill="#fff"/>' },
  // Glasdaus: a pane of glass with a glint.
  glasdaus: { fill: '#2f86c0', glyph: '<rect x="6" y="5" width="12" height="14" fill="#cfe8fd"/><path d="M8.5 15.5 L14.5 8.5" stroke="#fff" stroke-width="1.8" stroke-linecap="round"/>' },
  // Heimweh: a little house.
  heimweh: { fill: '#8e2430', glyph: '<path d="M12 5 L19 11 H17 V18.5 H7 V11 H5 Z" fill="#fff"/><rect x="10.5" y="13.5" width="3" height="5" fill="#8e2430"/>' },
  // Attrappe: a dummy, a card with a question mark.
  dummy: { fill: '#7d6a50', glyph: '<rect x="7.5" y="5.5" width="9" height="13" rx="1.5" fill="none" stroke="#fff" stroke-width="1.8"/><path d="M10.4 10 C10.4 8.4 13.6 8.4 13.6 10 C13.6 11.4 12 11.4 12 13" fill="none" stroke="#fff" stroke-width="1.6" stroke-linecap="round"/><circle cx="12" cy="15.4" r=".9" fill="#fff"/>' },
};

export function modIconId(mod: string): string {
  return MOD_ICON[mod] ? `m-${mod}` : 'm-other';
}

function modSymbols(): string {
  const icons = Object.entries(MOD_ICON)
    .map(
      ([id, icon]) =>
        `<symbol id="m-${id}" viewBox="0 0 24 24"><rect x="1" y="1" width="22" height="22" fill="${icon.fill}" stroke="${icon.ring ?? '#0b0c0e'}" stroke-width="2"/>${icon.glyph}</symbol>`,
    )
    .join('');
  return `${icons}<symbol id="m-other" viewBox="0 0 24 24"><rect x="1" y="1" width="22" height="22" fill="#6a6a5a" stroke="#0b0c0e" stroke-width="2"/><circle cx="12" cy="12" r="3" fill="#fff"/></symbol>`;
}

// ---------------------------------------------------------------- sprite

/** The hidden SVG sprite with every symbol of the deck. */
export function spriteSvg(): string {
  const suits = SUITS.map((s) => `<symbol id="s-${CODE[s]}" viewBox="0 0 8 8" shape-rendering="crispEdges">${glyphPaths(s)}</symbol>`).join('');
  // A face is paper (drawn by ./cards.ts, so modifications can tint it),
  // its picture (f-…) and its corner index (x-…), with the marks of card
  // modifications layered in between.
  const faces = SUITS.flatMap((s) =>
    RANKS.map(
      (r) =>
        `<symbol id="f-${CODE[s]}${r}" viewBox="0 0 120 180" shape-rendering="crispEdges">${faceArt(s, r)}</symbol><symbol id="x-${CODE[s]}${r}" viewBox="0 0 120 180" shape-rendering="crispEdges">${index(s, r)}</symbol>`,
    ),
  ).join('');
  return `<svg class="sprite" xmlns="http://www.w3.org/2000/svg" aria-hidden="true" focusable="false" width="0" height="0" style="position:absolute;width:0;height:0;overflow:hidden">
<defs>
<pattern id="back-lattice" width="16" height="16" patternUnits="userSpaceOnUse">
<rect width="16" height="16" fill="${SLATE_DARK}"/><path d="M6 2h4v4h-4zM14 10h4v4h-4zM-2 10h4v4h-4z" fill="${COPPER}" opacity=".5"/></pattern>
<pattern id="blank-hatch" width="8" height="8" patternUnits="userSpaceOnUse">
<path d="M0 0h4v4h-4z" fill="#cbc7bb"/></pattern>
</defs>
${suits}${faces}
<symbol id="f-back" viewBox="0 0 120 180" shape-rendering="crispEdges">${backBody()}</symbol>
<symbol id="f-blank" viewBox="0 0 120 180" shape-rendering="crispEdges">${blankBody()}</symbol>
${modSymbols()}
${peopleSymbols()}
</svg>`;
}

/** Symbol id of a face's picture. */
export function faceId(suit: Suit, rank: Rank): string {
  return `f-${CODE[suit]}${rank}`;
}

/** Symbol id of a face's corner index. */
export function indexId(suit: Suit, rank: Rank): string {
  return `x-${CODE[suit]}${rank}`;
}

export function suitId(suit: Suit): string {
  return `s-${CODE[suit]}`;
}

let installed = false;

/** Put the sprite into the document once. */
export function installSprite(): void {
  if (installed || typeof document === 'undefined') return;
  installed = true;
  document.body.insertAdjacentHTML('afterbegin', spriteSvg());
}

// The German deck, drawn by hand as SVG (ADR 0010). Everything is built
// once into a hidden sprite of <symbol>s; cards then only reference the
// symbols with <use>, which keeps the DOM small. Pure drawing, no rules.
//
// Coordinates: suit symbols in a 100 × 100 box, cards in 120 × 180.

import { de } from '../i18n';
import type { Rank, Suit } from '../types';
import { peopleSymbols } from './people';

export const SUITS: Suit[] = ['Clubs', 'Spades', 'Hearts', 'Diamonds'];
export const RANKS: Rank[] = ['7', '8', '9', 'T', 'J', 'Q', 'K', 'A'];

const CODE: Record<Suit, string> = { Clubs: 'C', Spades: 'S', Hearts: 'H', Diamonds: 'D' };

interface Palette {
  /** Corner index and outlines. */
  ink: string;
  /** Clothing of the figures. */
  robe: string;
  robeDark: string;
  /** Trim and the second colour of a figure. */
  trim: string;
}

/** Four clearly different hues: brown, green, red, orange-gold. */
export const PALETTE: Record<Suit, Palette> = {
  Clubs: { ink: '#4b2a0e', robe: '#8a5426', robeDark: '#4b2a0e', trim: '#e3b24c' },
  Spades: { ink: '#1d7433', robe: '#2f8a45', robeDark: '#175a28', trim: '#ead27a' },
  Hearts: { ink: '#c21d2c', robe: '#c62f3a', robeDark: '#7d1520', trim: '#f2c14e' },
  Diamonds: { ink: '#b36500', robe: '#e19a1f', robeDark: '#8a5200', trim: '#b8262d' },
};

const OUTLINE = '#2b1a0e';
const PAPER = '#fbf6e9';
const SKIN = '#f1c8a0';

// ------------------------------------------------------------ suit symbols

function suitBody(suit: Suit): string {
  switch (suit) {
    case 'Hearts':
      // Rot: a full, round German heart with a soft highlight.
      return `<path d="M50 93 C31 78 5 60 5 34 C5 17 18 6 32 6 C41 6 47 11 50 19 C53 11 59 6 68 6 C82 6 95 17 95 34 C95 60 69 78 50 93 Z" fill="#d42433" stroke="#6e0d16" stroke-width="3.5" stroke-linejoin="round"/>
<path d="M20 30 C20 20 28 15 35 16" fill="none" stroke="#fff" stroke-width="5" stroke-linecap="round" opacity=".35"/>`;
    case 'Spades':
      // Grün: a lime leaf, tip up, stem down, with midrib and veins.
      return `<path d="M50 3 C64 15 93 34 92 59 C91 77 73 85 57 76 L55 79 L58 97 L42 97 L45 79 L43 76 C27 85 9 77 8 59 C7 34 36 15 50 3 Z" fill="#2f9444" stroke="#0f4a1c" stroke-width="3.5" stroke-linejoin="round"/>
<path d="M50 12 L50 80 M50 32 L36 24 M50 32 L64 24 M50 47 L28 37 M50 47 L72 37 M50 62 L26 54 M50 62 L74 54" fill="none" stroke="#0f4a1c" stroke-width="2.6" stroke-linecap="round"/>
<path d="M22 50 C22 38 30 28 38 23" fill="none" stroke="#c9f0b8" stroke-width="3.5" stroke-linecap="round" opacity=".45"/>`;
    case 'Clubs':
      // Eichel: an acorn on its stem, cap cross-hatched.
      return `<path d="M51 3 C54 6 55 11 53 17" fill="none" stroke="#3a210a" stroke-width="5" stroke-linecap="round"/>
<path d="M26 46 C25 72 38 92 50 97 C62 92 75 72 74 46 Z" fill="#c98a35" stroke="#3a210a" stroke-width="3.5" stroke-linejoin="round"/>
<path d="M34 58 C35 72 40 82 46 87" fill="none" stroke="#fff3d6" stroke-width="4" stroke-linecap="round" opacity=".55"/>
<path d="M15 47 C14 27 30 15 50 15 C70 15 86 27 85 47 C70 54 30 54 15 47 Z" fill="#6b4116" stroke="#3a210a" stroke-width="3.5" stroke-linejoin="round"/>
<path d="M25 27 L41 50 M36 19 L58 51 M50 16 L72 48 M64 19 L81 42 M20 39 L33 21 M28 49 L50 17 M45 52 L69 20 M62 51 L80 30" stroke="#3a210a" stroke-width="2" opacity=".75"/>`;
    case 'Diamonds':
      // Schellen: a round sleigh bell with ring, band and slit.
      return `<circle cx="50" cy="10" r="6.5" fill="none" stroke="#5a3a00" stroke-width="3.5"/>
<circle cx="50" cy="55" r="38" fill="#f0b81c" stroke="#5a3a00" stroke-width="3.5"/>
<path d="M13 49 C30 60 70 60 87 49 L87 57 C70 68 30 68 13 57 Z" fill="#c0262f" stroke="#5a3a00" stroke-width="2"/>
<path d="M50 70 L50 86" stroke="#3b2400" stroke-width="5" stroke-linecap="round"/>
<circle cx="50" cy="73" r="5.5" fill="#3b2400"/>
<path d="M27 36 C31 28 38 24 45 23" fill="none" stroke="#fff6c8" stroke-width="5" stroke-linecap="round" opacity=".6"/>`;
  }
}

// --------------------------------------------------------------- the faces

const L = 44.5;
const R = 75.5;
const PIP_LAYOUT: Record<'7' | '8' | '9' | 'T', [number, number][]> = {
  // Two columns, the extra pips in the middle (German pattern). The
  // columns leave room for the large corner indices.
  '7': [[L, 30], [R, 30], [60, 60], [L, 90], [R, 90], [L, 150], [R, 150]],
  '8': [[L, 28], [R, 28], [L, 69], [R, 69], [L, 111], [R, 111], [L, 152], [R, 152]],
  '9': [[L, 28], [R, 28], [L, 62], [R, 62], [60, 90], [L, 118], [R, 118], [L, 152], [R, 152]],
  T: [[L, 26], [R, 26], [60, 48], [L, 70], [R, 70], [L, 110], [R, 110], [60, 132], [L, 154], [R, 154]],
};

function pip(suit: Suit, cx: number, cy: number, size: number): string {
  const h = size / 2;
  return `<use href="#s-${CODE[suit]}" x="${cx - h}" y="${cy - h}" width="${size}" height="${size}"/>`;
}

/** Corner index: rank big, suit below; mirrored in the bottom right. */
function index(suit: Suit, rank: Rank): string {
  const text = de.rankShort[rank];
  const wide = text.length > 1;
  const color = PALETTE[suit].ink;
  const one = `<rect x="2" y="2" width="31" height="66" rx="8" fill="${PAPER}"/>
<text x="17.5" y="33" text-anchor="middle" font-size="${wide ? 27 : 36}" font-weight="700" fill="${color}" ${wide ? 'letter-spacing="-2.5"' : ''} class="idx">${text}</text>
${pip(suit, 17.5, 51, 25)}`;
  return `${one}<g transform="rotate(180 60 90)">${one}</g>`;
}

function frame(suit: Suit): string {
  return `<rect x="9" y="9" width="102" height="162" rx="6" fill="#f6ecd3" stroke="${PALETTE[suit].ink}" stroke-width="1.6"/>`;
}

function numberFace(suit: Suit, rank: '7' | '8' | '9' | 'T'): string {
  const size = rank === 'T' ? 22 : rank === '9' ? 23 : 25;
  return PIP_LAYOUT[rank].map(([x, y]) => pip(suit, x, y, size)).join('');
}

function aceFace(suit: Suit): string {
  const ink = PALETTE[suit].ink;
  // A big symbol with scrolls above and a ribbon below.
  return `<g fill="none" stroke="${ink}" stroke-width="2" stroke-linecap="round" opacity=".7">
<path d="M60 26 C50 22 42 26 44 33 C46 38 53 36 51 31"/><path d="M60 26 C70 22 78 26 76 33 C74 38 67 36 69 31"/>
<path d="M60 154 C50 158 42 154 44 147 C46 142 53 144 51 149"/><path d="M60 154 C70 158 78 154 76 147 C74 142 67 144 69 149"/>
</g>
<path d="M30 126 L90 126 L84 134 L90 142 L30 142 L36 134 Z" fill="${PALETTE[suit].robe}" stroke="${ink}" stroke-width="1.6" stroke-linejoin="round"/>
<path d="M44 134 L76 134" stroke="${PAPER}" stroke-width="1.6" stroke-dasharray="3 3"/>
${pip(suit, 60, 84, 66)}`;
}

/** One half of a double-headed court card (König, Ober, Unter). */
function figureHalf(suit: Suit, rank: 'K' | 'Q' | 'J'): string {
  const p = PALETTE[suit];
  const hair = rank === 'K' ? '#ebe6da' : rank === 'Q' ? '#4f2c12' : '#c98d3c';
  const torso = `<path d="M38 90 C38 76 44 67 55 64 L81 64 C92 67 98 76 98 90 Z" fill="${p.robe}" stroke="${OUTLINE}" stroke-width="1.5"/>
<path d="M55 64 L68 90 L81 64" fill="${p.robeDark}" opacity=".5"/>
<path d="M43 90 C43 81 46 75 51 71 M93 90 C93 81 90 75 85 71" fill="none" stroke="${p.trim}" stroke-width="3"/>`;
  const emblem = `<circle cx="68" cy="81" r="8" fill="${PAPER}" stroke="${OUTLINE}" stroke-width="1.2"/>${pip(suit, 68, 81, 11.5)}`;
  const ruff = `<ellipse cx="68" cy="63" rx="15" ry="4.5" fill="#fffaf0" stroke="${OUTLINE}" stroke-width="1.2" stroke-dasharray="2.2 1.6"/>`;
  const neck = `<rect x="63" y="54" width="10" height="8" fill="${SKIN}"/>`;
  const head = `<ellipse cx="68" cy="42" rx="12" ry="14" fill="${SKIN}" stroke="${OUTLINE}" stroke-width="1.3"/>
<circle cx="61.5" cy="47" r="2.4" fill="#e9967a" opacity=".45"/><circle cx="74.5" cy="47" r="2.4" fill="#e9967a" opacity=".45"/>`;
  const hairSides = `<path d="M56.5 37 C55.5 45 57.5 51 60 54 M79.5 37 C80.5 45 78.5 51 76 54" fill="none" stroke="${hair}" stroke-width="4.5" stroke-linecap="round"/>`;
  const eyes = `<path d="M61.5 40 L65.5 39.5 M70.5 39.5 L74.5 40" stroke="${OUTLINE}" stroke-width="1.1" stroke-linecap="round"/>
<circle cx="63.5" cy="42.5" r="1.4" fill="${OUTLINE}"/><circle cx="72.5" cy="42.5" r="1.4" fill="${OUTLINE}"/>
<path d="M68 43 L66.6 48 L69 48" fill="none" stroke="${OUTLINE}" stroke-width="1" stroke-linecap="round"/>`;
  let face = '';
  let hat = '';
  let held = '';
  if (rank === 'K') {
    // König: crown, white full beard, sceptre.
    face = `<path d="M56.5 45 C56.5 58 62 63.5 68 63.5 C74 63.5 79.5 58 79.5 45 C76 50.5 72 50 68 50 C64 50 60 50.5 56.5 45 Z" fill="${hair}" stroke="${OUTLINE}" stroke-width="1"/>
<path d="M63 49.5 C65.5 48.5 70.5 48.5 73 49.5" fill="none" stroke="${OUTLINE}" stroke-width="1.2" stroke-linecap="round"/>`;
    hat = `<path d="M55 31 L55 15 L61 22.5 L64.5 11 L68 20 L71.5 11 L75 22.5 L81 15 L81 31 Z" fill="#e8b52a" stroke="#6b4c00" stroke-width="1.3" stroke-linejoin="round"/>
<rect x="54.5" y="28" width="27" height="5.5" rx="1" fill="#b8262d" stroke="#6b4c00" stroke-width="1"/>
<circle cx="68" cy="24" r="2.2" fill="#2f6fb0"/><circle cx="61" cy="30.7" r="1.2" fill="#fff4c8"/><circle cx="75" cy="30.7" r="1.2" fill="#fff4c8"/>`;
    held = `<path d="M95 70 L101 26" stroke="#8a6a1c" stroke-width="2.6" stroke-linecap="round"/><circle cx="101.3" cy="23" r="4.2" fill="#e8b52a" stroke="#6b4c00" stroke-width="1.2"/>`;
  } else if (rank === 'Q') {
    // Ober: feathered beret, dark moustache, symbol held high on a staff.
    face = `<path d="M61 50 C63.5 47.5 66.5 48.5 68 49.5 C69.5 48.5 72.5 47.5 75 50 C72.5 52 70 51 68 51 C66 51 63.5 52 61 50 Z" fill="${hair}"/>
<path d="M66 54.5 L68 59 L70 54.5" fill="${hair}"/>`;
    hat = `<path d="M50 33 C51 24 59 20.5 68 20.5 C79 20.5 87 25 88 33 Z" fill="${p.robeDark}" stroke="${OUTLINE}" stroke-width="1.3"/>
<ellipse cx="68" cy="33" rx="21" ry="4.2" fill="${p.robeDark}" stroke="${OUTLINE}" stroke-width="1.3"/>
<path d="M78 24 C88 18 93 9 89 3 C87 11 82 17 74 22" fill="#fffaf0" stroke="${OUTLINE}" stroke-width="1"/>`;
    held = `<path d="M97 70 L99 34" stroke="#7a5a2a" stroke-width="2.6" stroke-linecap="round"/>${pip(suit, 99, 23, 23)}`;
  } else {
    // Unter: young, a cap, symbol carried low next to the belt.
    face = `<path d="M63.5 51.5 C66 53.5 70 53.5 72.5 51.5" fill="none" stroke="${OUTLINE}" stroke-width="1.1" stroke-linecap="round"/>`;
    hat = `<path d="M55 35 C55 24 61 19.5 69 19.5 C77.5 19.5 82 25 82 33 C76 31 63 31 55 35 Z" fill="${p.trim}" stroke="${OUTLINE}" stroke-width="1.3"/>
<path d="M79 24 C85 20.5 90 21.5 92 25" fill="none" stroke="${p.robe}" stroke-width="3.2" stroke-linecap="round"/>`;
    held = `<path d="M40 74 L27 84" stroke="#7a5a2a" stroke-width="2.6" stroke-linecap="round"/>${pip(suit, 25, 79, 22)}`;
  }
  return `${torso}${ruff}${neck}${head}${hairSides}${eyes}${face}${hat}${emblem}${held}`;
}

function courtFace(suit: Suit, rank: 'K' | 'Q' | 'J'): string {
  const half = figureHalf(suit, rank);
  return `${frame(suit)}
<g clip-path="url(#clip-top)">${half}</g>
<g transform="rotate(180 60 90)" clip-path="url(#clip-top)">${half}</g>
<path d="M10 90 L110 90" stroke="${PALETTE[suit].ink}" stroke-width="1.6"/>`;
}

function faceBody(suit: Suit, rank: Rank): string {
  let center: string;
  if (rank === 'A') center = aceFace(suit);
  else if (rank === 'K' || rank === 'Q' || rank === 'J') center = courtFace(suit, rank);
  else center = numberFace(suit, rank);
  return `<rect x="1" y="1" width="118" height="178" rx="10" fill="${PAPER}" stroke="#bfb397" stroke-width="2"/>${center}${index(suit, rank)}`;
}

// ------------------------------------------------------------- other faces

function backBody(): string {
  const medallion = SUITS.map((s, i) => {
    const pos = [[60, 72], [76, 90], [60, 108], [44, 90]][i] ?? [60, 90];
    return pip(s, pos[0] ?? 60, pos[1] ?? 90, 15);
  }).join('');
  return `<rect x="1" y="1" width="118" height="178" rx="10" fill="#f3e9d2" stroke="#bfb397" stroke-width="2"/>
<rect x="7" y="7" width="106" height="166" rx="6" fill="#7a1f2b"/>
<rect x="7" y="7" width="106" height="166" rx="6" fill="url(#back-lattice)"/>
<rect x="11" y="11" width="98" height="158" rx="4" fill="none" stroke="#e3b24c" stroke-width="1.5" opacity=".8"/>
<ellipse cx="60" cy="90" rx="27" ry="32" fill="#f3e9d2" stroke="#e3b24c" stroke-width="3"/>
<ellipse cx="60" cy="90" rx="22" ry="27" fill="none" stroke="#7a1f2b" stroke-width="1" stroke-dasharray="2 2"/>
${medallion}`;
}

function blankBody(): string {
  return `<rect x="1" y="1" width="118" height="178" rx="10" fill="#ece6d6" stroke="#bfb397" stroke-width="2"/>
<rect x="10" y="10" width="100" height="160" rx="6" fill="url(#blank-hatch)" stroke="#b5ab95" stroke-width="1.5" stroke-dasharray="6 4"/>
<rect x="18" y="78" width="84" height="26" rx="4" fill="#ece6d6"/>
<text x="60" y="97" text-anchor="middle" font-size="19" font-weight="700" fill="#8a826f" class="idx">${de.card.blank}</text>`;
}

// --------------------------------------------------------- modifications

/** Badge icons of the card modifications, in a 24 × 24 box. */
const MOD_ICON: Record<string, { fill: string; glyph: string }> = {
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
};

export function modIconId(mod: string): string {
  return MOD_ICON[mod] ? `m-${mod}` : 'm-other';
}

function modSymbols(): string {
  const icons = Object.entries(MOD_ICON)
    .map(
      ([id, icon]) =>
        `<symbol id="m-${id}" viewBox="0 0 24 24"><circle cx="12" cy="12" r="11" fill="${icon.fill}" stroke="#fff" stroke-width="1.6"/>${icon.glyph}</symbol>`,
    )
    .join('');
  return `${icons}<symbol id="m-other" viewBox="0 0 24 24"><circle cx="12" cy="12" r="11" fill="#6a6a5a" stroke="#fff" stroke-width="1.6"/><circle cx="12" cy="12" r="3" fill="#fff"/></symbol>`;
}

// ---------------------------------------------------------------- sprite

/** The hidden SVG sprite with every symbol of the deck. */
export function spriteSvg(): string {
  const suits = SUITS.map((s) => `<symbol id="s-${CODE[s]}" viewBox="0 0 100 100">${suitBody(s)}</symbol>`).join('');
  const faces = SUITS.flatMap((s) =>
    RANKS.map((r) => `<symbol id="f-${CODE[s]}${r}" viewBox="0 0 120 180">${faceBody(s, r)}</symbol>`),
  ).join('');
  return `<svg class="sprite" xmlns="http://www.w3.org/2000/svg" aria-hidden="true" focusable="false" width="0" height="0" style="position:absolute;width:0;height:0;overflow:hidden">
<defs>
<pattern id="back-lattice" width="12" height="12" patternUnits="userSpaceOnUse" patternTransform="rotate(45)">
<path d="M0 0 H12 M0 0 V12" stroke="#e3b24c" stroke-width="1.2" opacity=".55"/><circle cx="6" cy="6" r="1.4" fill="#e3b24c" opacity=".55"/></pattern>
<pattern id="blank-hatch" width="8" height="8" patternUnits="userSpaceOnUse" patternTransform="rotate(45)">
<path d="M0 0 V8" stroke="#d6cdb7" stroke-width="2"/></pattern>
<clipPath id="clip-top"><rect x="10" y="10" width="100" height="80"/></clipPath>
<linearGradient id="glass" x1="0" y1="0" x2="1" y2="1">
<stop offset="0" stop-color="#d8f0ff" stop-opacity=".7"/><stop offset=".45" stop-color="#bfe3ff" stop-opacity=".15"/>
<stop offset=".55" stop-color="#fff" stop-opacity=".55"/><stop offset="1" stop-color="#9fd3f5" stop-opacity=".25"/></linearGradient>
</defs>
${suits}${faces}
<symbol id="f-back" viewBox="0 0 120 180">${backBody()}</symbol>
<symbol id="f-blank" viewBox="0 0 120 180">${blankBody()}</symbol>
${modSymbols()}
${peopleSymbols()}
</svg>`;
}

/** Symbol id of a face. */
export function faceId(suit: Suit, rank: Rank): string {
  return `f-${CODE[suit]}${rank}`;
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

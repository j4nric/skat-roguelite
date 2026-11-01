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

// Pips of the number cards (Sieben to Zehn).
//
// The pattern follows the German deck as far as it fits the large corner
// indices of this app: two straight columns, symmetric about the middle of
// the card, and the extra pips centred between them. The exact historical
// pattern of the Altenburg German deck was not at hand, so this is a
// chosen, documented layout:
//
//   Sieben  2-1-2-2  three pairs, the seventh pip centred in the upper gap
//   Acht    2-2-2-2  two columns of four
//   Neun    2-2-1-2-2  two columns of four, the ninth pip in the middle
//   Zehn    2-2-2-2-2  two columns of five
//
// The columns (x = 45 and 75) stay clear of both corner indices (the top
// left one ends at x = 33, the bottom right one starts at x = 87). As on a
// real double-ended card, pips below the middle stand on their head.
const L = 45;
const R = 75;
const ROWS4 = [28, 69.3, 110.7, 152];
const ROWS5 = [28, 59, 90, 121, 152];
const pair = (y: number): [number, number][] => [
  [L, y],
  [R, y],
];
const PIP_LAYOUT: Record<'7' | '8' | '9' | 'T', [number, number][]> = {
  '7': [...pair(28), [60, 59], ...pair(90), ...pair(152)],
  '8': ROWS4.flatMap(pair),
  '9': [...pair(28), ...pair(59), [60, 90], ...pair(121), ...pair(152)],
  T: ROWS5.flatMap(pair),
};
const PIP_SIZE: Record<'7' | '8' | '9' | 'T', number> = { '7': 23, '8': 23, '9': 22, T: 22 };

function pip(suit: Suit, cx: number, cy: number, size: number): string {
  const h = size / 2;
  return `<use href="#s-${CODE[suit]}" x="${cx - h}" y="${cy - h}" width="${size}" height="${size}"/>`;
}

/** A pip that stands on its head (lower half of a double-ended card). */
function pipDown(suit: Suit, cx: number, cy: number, size: number): string {
  return `<g transform="rotate(180 ${cx} ${cy})">${pip(suit, cx, cy, size)}</g>`;
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
  const size = PIP_SIZE[rank];
  return PIP_LAYOUT[rank].map(([x, y]) => (y > 90 ? pipDown(suit, x, y, size) : pip(suit, x, y, size))).join('');
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

// The court cards are double-headed, as in the Altenburg Skat deck. Each
// half shows one figure in the top half (x 10..110, y 10..90); the corner
// index covers x < 33, y < 68, so the figure stands right of it.
//
// What tells them apart at hand size, beyond the corner letter:
// - König: a big golden crown, a white beard and an ermine collar, seated
//   on a dark blue throne whose round back frames his head; a sceptre, no
//   suit pip in hand.
// - Ober: holds his suit pip high above his head (top right corner, and
//   bottom left on the turned half): the classic mark of the Ober.
// - Unter: holds his suit pip low, next to the dividing line (left of the
//   middle, and right of it on the turned half): the classic mark of the
//   Unter. Young, beardless, with a cap.

const HEAD = { x: 64, y: 42 };

function figureBody(suit: Suit, collar: boolean): string {
  const p = PALETTE[suit];
  const torso = `<path d="M36 90 C36 74 44 64.5 56 61.5 L72 61.5 C84 64.5 92 74 92 90 Z" fill="${p.robe}" stroke="${OUTLINE}" stroke-width="1.5"/>
<path d="M56 61.5 L64 90 L72 61.5" fill="${p.robeDark}" opacity=".55"/>
<path d="M58 66 L64 90 L70 66" fill="none" stroke="${p.trim}" stroke-width="2.2"/>`;
  const ruff = collar
    ? ''
    : `<ellipse cx="64" cy="61" rx="13.5" ry="4.2" fill="#fffaf0" stroke="${OUTLINE}" stroke-width="1.2" stroke-dasharray="2.2 1.6"/>`;
  return `${torso}${ruff}`;
}

function figureHead(hair: string): string {
  const { x, y } = HEAD;
  return `<rect x="${x - 5}" y="${y + 9}" width="10" height="9" fill="${SKIN}" stroke="${OUTLINE}" stroke-width="1"/>
<path d="M${x - 11.5} ${y - 3} C${x - 13} ${y + 6} ${x - 10} ${y + 12} ${x - 6} ${y + 14} M${x + 11.5} ${y - 3} C${x + 13} ${y + 6} ${x + 10} ${y + 12} ${x + 6} ${y + 14}" fill="none" stroke="${hair}" stroke-width="5" stroke-linecap="round"/>
<ellipse cx="${x}" cy="${y}" rx="11.5" ry="13" fill="${SKIN}" stroke="${OUTLINE}" stroke-width="1.3"/>
<circle cx="${x - 6.5}" cy="${y + 5}" r="2.3" fill="#e9967a" opacity=".5"/><circle cx="${x + 6.5}" cy="${y + 5}" r="2.3" fill="#e9967a" opacity=".5"/>
<path d="M${x - 7} ${y - 3.5} L${x - 2.5} ${y - 4.2} M${x + 2.5} ${y - 4.2} L${x + 7} ${y - 3.5}" stroke="${OUTLINE}" stroke-width="1.2" stroke-linecap="round"/>
<circle cx="${x - 4.5}" cy="${y - 0.5}" r="1.5" fill="${OUTLINE}"/><circle cx="${x + 4.5}" cy="${y - 0.5}" r="1.5" fill="${OUTLINE}"/>
<path d="M${x - 0.5} ${y + 1} C${x - 1} ${y + 3} ${x - 2.4} ${y + 4.5} ${x - 1.2} ${y + 5.6} C${x - 0.4} ${y + 6.2} ${x + 1} ${y + 5.9} ${x + 1.8} ${y + 5.2}" fill="none" stroke="${OUTLINE}" stroke-width="1" stroke-linecap="round"/>`;
}

function kingHalf(suit: Suit): string {
  const p = PALETTE[suit];
  const gold = '#e8b52a';
  const goldDark = '#6b4c00';
  const throne = `<path d="M37 90 L37 36 C37 19 49 11 64 11 C79 11 91 19 91 36 L91 90 Z" fill="#2c3e74" stroke="${OUTLINE}" stroke-width="1.3"/>
<path d="M41.5 90 L41.5 37 C41.5 23 51 15.5 64 15.5 C77 15.5 86.5 23 86.5 37 L86.5 90" fill="none" stroke="${gold}" stroke-width="1.6" stroke-dasharray="1 3.2" stroke-linecap="round"/>`;
  const ermine = `<path d="M38 80 C40 70 48 64.5 64 64.5 C80 64.5 88 70 90 80 C82 74 74 72.5 64 72.5 C54 72.5 46 74 38 80 Z" fill="#fffaf0" stroke="${OUTLINE}" stroke-width="1.2"/>
<path d="M45 73 v2.6 M52 69.5 v2.6 M76 69.5 v2.6 M83 73 v2.6" stroke="${OUTLINE}" stroke-width="1.8" stroke-linecap="round"/>`;
  const beard = `<path d="M52.5 44 C52 58 57 70 64 75 C71 70 76 58 75.5 44 C72 50 68 50.5 64 50.5 C60 50.5 56 50 52.5 44 Z" fill="#ebe6da" stroke="${OUTLINE}" stroke-width="1"/>
<path d="M58.5 49 C61 47.6 67 47.6 69.5 49" fill="none" stroke="${OUTLINE}" stroke-width="1.3" stroke-linecap="round"/>
<path d="M60 56 C61 62 62.5 66 64 69 M68 56 C67 62 65.5 66 64 69" fill="none" stroke="#bdb6a6" stroke-width="1"/>`;
  const crown = `<path d="M49.5 32 L47.5 15 L56 22 L64 10 L72 22 L80.5 15 L78.5 32 Z" fill="${gold}" stroke="${goldDark}" stroke-width="1.3" stroke-linejoin="round"/>
<circle cx="47.5" cy="14" r="2.3" fill="${gold}" stroke="${goldDark}" stroke-width="1"/><circle cx="64" cy="9.5" r="2.5" fill="${gold}" stroke="${goldDark}" stroke-width="1"/><circle cx="80.5" cy="14" r="2.3" fill="${gold}" stroke="${goldDark}" stroke-width="1"/>
<rect x="48.5" y="28" width="31" height="6" rx="1.2" fill="#b8262d" stroke="${goldDark}" stroke-width="1"/>
<circle cx="64" cy="23" r="2.6" fill="#2f6fb0" stroke="${goldDark}" stroke-width=".8"/><circle cx="55" cy="31" r="1.4" fill="#fff4c8"/><circle cx="64" cy="31" r="1.4" fill="#fff4c8"/><circle cx="73" cy="31" r="1.4" fill="#fff4c8"/>`;
  const sceptre = `<path d="M84 70 C88 66 92 64 95 62 L98.5 66 C95 70 91 74 87 76 Z" fill="${p.robe}" stroke="${OUTLINE}" stroke-width="1.2"/>
<path d="M95 84 L101 25" stroke="${goldDark}" stroke-width="4" stroke-linecap="round"/><path d="M95 84 L101 25" stroke="${gold}" stroke-width="2.2" stroke-linecap="round"/>
<path d="M101.5 13 L104.5 19 L101.5 25 L98.5 19 Z M96 19.5 L107 18.5" fill="${gold}" stroke="${goldDark}" stroke-width="1.2" stroke-linejoin="round"/>
<ellipse cx="97.5" cy="64" rx="4" ry="4.5" fill="${SKIN}" stroke="${OUTLINE}" stroke-width="1.1"/>`;
  return `${throne}${figureBody(suit, true)}${ermine}${figureHead('#ebe6da')}${beard}${crown}${sceptre}`;
}

function oberHalf(suit: Suit): string {
  const p = PALETTE[suit];
  const hair = '#4f2c12';
  const arm = `<path d="M80 64 C88 62 92 55 93.5 46 L101 47.5 C100 58 95 68 86 73 Z" fill="${p.robe}" stroke="${OUTLINE}" stroke-width="1.3"/>
<path d="M92.5 50 L100.5 51.5" stroke="${p.trim}" stroke-width="2.4"/>
<ellipse cx="97.5" cy="42.5" rx="4.6" ry="4.2" fill="${SKIN}" stroke="${OUTLINE}" stroke-width="1.1"/>`;
  const held = pip(suit, 94.5, 25.5, 32);
  const face = `<path d="M57.5 49.5 C60 46.6 63 47.6 64 48.6 C65 47.6 68 46.6 70.5 49.5 C68 51.2 66 50.5 64 50.5 C62 50.5 60 51.2 57.5 49.5 Z" fill="${hair}"/>
<path d="M61.5 53 L64 59.5 L66.5 53 Z" fill="${hair}"/>`;
  const hat = `<path d="M50 33 C50 22 56 17.5 64 17.5 C73 17.5 78.5 22 78.5 33 Z" fill="${p.robeDark}" stroke="${OUTLINE}" stroke-width="1.3"/>
<ellipse cx="64" cy="33" rx="18" ry="4" fill="${p.robeDark}" stroke="${OUTLINE}" stroke-width="1.3"/>
<path d="M51 30 C51 29 77 29 78 30" fill="none" stroke="${p.trim}" stroke-width="2"/>
<path d="M56 22 C47 19 41 12 42 4 C47 11 53 15 60 18 Z" fill="#fffaf0" stroke="${OUTLINE}" stroke-width="1"/>`;
  return `${figureBody(suit, false)}${figureHead(hair)}${face}${hat}${arm}${held}`;
}

function unterHalf(suit: Suit): string {
  const p = PALETTE[suit];
  const hair = '#c98d3c';
  const arm = `<path d="M47 64.5 C42 68 39.5 74 40.5 80 L48 80.5 C48 75 50 71 53.5 67.5 Z" fill="${p.robe}" stroke="${OUTLINE}" stroke-width="1.3"/>
<path d="M40.5 77 L48 77.5" stroke="${p.trim}" stroke-width="2.2"/>`;
  const held = `<circle cx="40" cy="74.5" r="15.5" fill="${PAPER}" stroke="${p.ink}" stroke-width="1.5"/>${pip(suit, 40, 74.5, 27.5)}
<ellipse cx="50" cy="81.5" rx="4.4" ry="4" fill="${SKIN}" stroke="${OUTLINE}" stroke-width="1.1"/>`;
  const face = `<path d="M60.5 50 C62.5 51.8 65.5 51.8 67.5 50" fill="none" stroke="${OUTLINE}" stroke-width="1.1" stroke-linecap="round"/>`;
  const cap = `<path d="M51 35 C50 24 56 19 65 19 C74 19 79 24 78.5 32 C70 29 59 30 51 35 Z" fill="${p.trim}" stroke="${OUTLINE}" stroke-width="1.3"/>
<path d="M51 35 C59 30 70 29 78.5 32" fill="none" stroke="${p.robeDark}" stroke-width="2.4"/>
<path d="M75 23 C81 17 88 16 92 19 C87 20 81 23 77.5 27 Z" fill="${p.robe}" stroke="${OUTLINE}" stroke-width="1"/>`;
  const belt = `<path d="M38.5 83 C50 85 78 85 89.5 83" fill="none" stroke="${OUTLINE}" stroke-width="3.2"/><rect x="61" y="81" width="6" height="5.5" rx="1" fill="${p.trim}" stroke="${OUTLINE}" stroke-width=".8"/>`;
  return `${figureBody(suit, false)}${belt}${figureHead(hair)}${face}${cap}${arm}${held}`;
}

/** One half of a double-headed court card (König, Ober, Unter). */
function figureHalf(suit: Suit, rank: 'K' | 'Q' | 'J'): string {
  if (rank === 'K') return kingHalf(suit);
  if (rank === 'Q') return oberHalf(suit);
  return unterHalf(suit);
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

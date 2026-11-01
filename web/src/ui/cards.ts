// Card faces of the German deck, drawn as inline SVG (ADR 0010). Simple M2
// version: large corner index, one suit symbol, rank name on picture cards.
// Blanks and card modifications are marked. Pure drawing, no game logic.

import { de, cardName, contentName, face, t } from '../i18n';
import type { CardKind, Suit } from '../types';
import { esc } from './dom';

/** Ink colour per suit: clearly different hue and lightness. */
export const SUIT_COLORS: Record<Suit, string> = {
  Clubs: '#6b3f12',
  Spades: '#23803a',
  Hearts: '#c8202f',
  Diamonds: '#c27c00',
};

/** Suit symbol paths in a 100 × 100 box. */
function suitShape(suit: Suit): string {
  switch (suit) {
    case 'Hearts':
      // Herz.
      return `<path d="M50 90 C24 70 6 52 6 32 C6 16 18 6 31 6 C40 6 46 11 50 19 C54 11 60 6 69 6 C82 6 94 16 94 32 C94 52 76 70 50 90 Z" fill="#d72631" stroke="#7d0f18" stroke-width="3"/>`;
    case 'Spades':
      // Grün: a leaf with stem and midrib.
      return `<path d="M50 4 C74 22 92 42 89 62 C87 77 71 84 56 77 L60 96 L40 96 L44 77 C29 84 13 77 11 62 C8 42 26 22 50 4 Z" fill="#2f9a46" stroke="#13531f" stroke-width="3"/>
<path d="M50 14 L50 80 M50 40 L34 30 M50 40 L66 30 M50 58 L30 48 M50 58 L70 48" stroke="#13531f" stroke-width="3" fill="none" stroke-linecap="round"/>`;
    case 'Clubs':
      // Eichel: acorn with a hatched cap.
      return `<rect x="46" y="4" width="8" height="16" rx="3" fill="#4a2a0c"/>
<ellipse cx="50" cy="64" rx="24" ry="30" fill="#b9792f" stroke="#4a2a0c" stroke-width="3"/>
<path d="M18 46 C18 24 82 24 82 46 Z" fill="#5c3510" stroke="#2f1806" stroke-width="3"/>
<path d="M28 36 L40 44 M40 30 L54 44 M54 28 L68 42 M66 30 L74 38" stroke="#2f1806" stroke-width="2.5" fill="none"/>
<path d="M40 70 C42 80 48 86 54 88" stroke="#fbe6c3" stroke-width="3" fill="none" stroke-linecap="round" opacity="0.7"/>`;
    case 'Diamonds':
      // Schellen: a round bell with band and slit.
      return `<circle cx="50" cy="12" r="6" fill="none" stroke="#6b4300" stroke-width="3"/>
<circle cx="50" cy="54" r="36" fill="#f3b80f" stroke="#6b4300" stroke-width="3"/>
<path d="M15 46 Q50 60 85 46" stroke="#c0281f" stroke-width="7" fill="none"/>
<path d="M36 72 Q50 82 64 72" stroke="#3b2400" stroke-width="5" fill="none" stroke-linecap="round"/>
<circle cx="50" cy="86" r="3" fill="#3b2400"/>`;
  }
}

/** A suit symbol as a standalone inline SVG. */
export function suitIcon(suit: Suit, size = 24, label = de.suits[suit]): string {
  return `<svg class="suit-icon" viewBox="0 0 100 100" width="${size}" height="${size}" role="img" aria-label="${esc(label)}">${suitShape(suit)}</svg>`;
}

function symbolAt(suit: Suit, x: number, y: number, size: number): string {
  return `<g transform="translate(${x} ${y}) scale(${size / 100})">${suitShape(suit)}</g>`;
}

/** The face of a card as inline SVG (viewBox 120 × 180). */
export function cardSvg(kind: CardKind, mods: string[] = []): string {
  const f = face(kind);
  const label = esc(cardName(kind) + (mods.length ? ', ' + t(de.card.mods, { mods: mods.map((m) => contentName('card_mods', m)).join(', ') }) : ''));
  const modBadges = mods
    .map((m, i) => {
      const short = esc(de.modShort[m] ?? m.slice(0, 2));
      const cx = 22 + i * 33;
      return `<g><circle cx="${cx}" cy="156" r="16" fill="#f5c542" stroke="#7a5600" stroke-width="2.5"/><text x="${cx}" y="162" text-anchor="middle" font-size="17" font-weight="800" fill="#3a2a00">${short}</text></g>`;
    })
    .join('');
  const frame = mods.length
    ? `<rect x="4" y="4" width="112" height="172" rx="9" fill="none" stroke="#e0a800" stroke-width="4"/>`
    : '';
  if (!f) {
    return `<svg class="card-svg" viewBox="0 0 120 180" role="img" aria-label="${label}">
<rect x="1.5" y="1.5" width="117" height="177" rx="11" fill="#e4e0d6" stroke="#9b958a" stroke-width="3" stroke-dasharray="8 6"/>
<text x="60" y="98" text-anchor="middle" font-size="22" font-weight="700" fill="#77726a">${esc(de.card.blank)}</text>
${frame}${modBadges}</svg>`;
  }
  const color = SUIT_COLORS[f.suit];
  const short = esc(de.rankShort[f.rank]);
  const isPicture = f.rank === 'J' || f.rank === 'Q' || f.rank === 'K';
  const indexSize = short.length > 1 ? 30 : 36;
  let center: string;
  if (f.rank === 'A') {
    center = symbolAt(f.suit, 22, 50, 76);
  } else if (isPicture) {
    // Ober carries his symbol high, Unter low, König in the middle.
    const y = f.rank === 'Q' ? 40 : f.rank === 'J' ? 66 : 54;
    center =
      symbolAt(f.suit, 34, y, 52) +
      `<text x="60" y="136" text-anchor="middle" font-size="19" font-weight="700" fill="${color}">${esc(de.ranks[f.rank])}</text>`;
  } else {
    center = symbolAt(f.suit, 32, 62, 56);
  }
  return `<svg class="card-svg" viewBox="0 0 120 180" role="img" aria-label="${label}">
<rect x="1.5" y="1.5" width="117" height="177" rx="11" fill="#fffdf6" stroke="#b9b09c" stroke-width="3"/>
<text x="8" y="${indexSize + 2}" font-size="${indexSize}" font-weight="800" fill="${color}" font-family="system-ui, sans-serif" letter-spacing="-1">${short}</text>
${symbolAt(f.suit, 8, indexSize + 8, 26)}
<g transform="rotate(180 60 90)"><text x="8" y="${indexSize + 2}" font-size="${indexSize}" font-weight="800" fill="${color}" font-family="system-ui, sans-serif" letter-spacing="-1">${short}</text></g>
${center}${frame}${modBadges}</svg>`;
}

/** Back of a card (CSS pattern). */
export function cardBack(className = ''): string {
  return `<span class="card-back ${className}" role="img" aria-label="${esc(de.card.back)}"></span>`;
}

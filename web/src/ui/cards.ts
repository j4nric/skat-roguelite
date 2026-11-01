// Cards as small SVGs that reference the sprite in ./art.ts. Blanks and
// card modifications are marked: a square badge per modification in the
// free bottom-left corner, plus an ink stamp or a mint tint where it helps.
// Pure drawing, no game logic.

import { de, cardName, contentName, face, isCurse, t } from '../i18n';
import type { CardKind, Suit } from '../types';
import { faceId, modIconId, PALETTE, suitId } from './art';
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

function modLayers(mods: string[]): string {
  const overlays = mods
    .map((m) => {
      if (m === 'stamp') {
        return `<g transform="rotate(-16 64 112)" opacity=".5"><circle cx="64" cy="112" r="21" fill="none" stroke="#5e3796" stroke-width="3"/><circle cx="64" cy="112" r="16.5" fill="none" stroke="#5e3796" stroke-width="1.2"/><path d="M64 100 L67 109 L76 109 L69 115 L72 124 L64 118 L56 124 L59 115 L52 109 L61 109 Z" fill="#5e3796"/></g>`;
      }
      if (m === 'transparent') {
        return `<rect width="120" height="180" fill="#8fc7a8" opacity=".22"/><rect x="4" y="4" width="112" height="172" fill="none" stroke="#2f86c0" stroke-width="4" stroke-dasharray="8 8"/>`;
      }
      return '';
    })
    .join('');
  // A curse darkens the card's edge with a red, broken frame.
  const curse = mods.some(isCurse)
    ? `<rect x="4" y="4" width="112" height="172" fill="none" stroke="#bd3b32" stroke-width="4" stroke-dasharray="12 4 4 4"/>`
    : '';
  const badges = mods
    .map((m, i) => `<use href="#${modIconId(m)}" x="6" y="${146 - i * 30}" width="28" height="28"/>`)
    .join('');
  return overlays + curse + badges;
}

/** The face of a card as inline SVG (viewBox 120 × 180). */
export function cardSvg(kind: CardKind, mods: string[] = []): string {
  const f = face(kind);
  const modNames = mods.map((m) => (isCurse(m) ? `${contentName('card_mods', m)} (${de.card.curse})` : contentName('card_mods', m))).join(', ');
  const label = esc(cardName(kind) + (mods.length ? ', ' + t(de.card.mods, { mods: modNames }) : ''));
  const id = f ? faceId(f.suit, f.rank) : 'f-blank';
  return `<svg class="card-svg" viewBox="0 0 120 180" role="img" aria-label="${label}"><use href="#${id}"/>${modLayers(mods)}</svg>`;
}

/** Back of a card. */
export function cardBack(className = ''): string {
  return `<span class="card-back ${className}" role="img" aria-label="${esc(de.card.back)}"><svg viewBox="0 0 120 180" aria-hidden="true"><use href="#f-back"/></svg></span>`;
}

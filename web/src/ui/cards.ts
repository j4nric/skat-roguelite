// Cards as small SVGs that reference the sprite in ./art.ts. Blanks and
// card modifications are marked: a badge per modification in the free
// bottom-left corner, plus an ink stamp or a glass sheen where it helps.
// Pure drawing, no game logic.

import { de, cardName, contentName, face, t } from '../i18n';
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
  return `<svg class="suit-icon" viewBox="0 0 100 100" width="${size}" height="${size}" ${a11y}><use href="#${suitId(suit)}"/></svg>`;
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
        return `<rect x="1" y="1" width="118" height="178" rx="10" fill="url(#glass)" opacity=".5"/><rect x="3" y="3" width="114" height="174" rx="9" fill="none" stroke="#2f86c0" stroke-width="2.5" stroke-dasharray="7 5"/>`;
      }
      return '';
    })
    .join('');
  const badges = mods
    .map((m, i) => `<use href="#${modIconId(m)}" x="3" y="${150 - i * 27}" width="27" height="27"/>`)
    .join('');
  return overlays + badges;
}

/** The face of a card as inline SVG (viewBox 120 × 180). */
export function cardSvg(kind: CardKind, mods: string[] = []): string {
  const f = face(kind);
  const modNames = mods.map((m) => contentName('card_mods', m)).join(', ');
  const label = esc(cardName(kind) + (mods.length ? ', ' + t(de.card.mods, { mods: modNames }) : ''));
  const id = f ? faceId(f.suit, f.rank) : 'f-blank';
  return `<svg class="card-svg" viewBox="0 0 120 180" role="img" aria-label="${label}"><use href="#${id}"/>${modLayers(mods)}</svg>`;
}

/** Back of a card. */
export function cardBack(className = ''): string {
  return `<span class="card-back ${className}" role="img" aria-label="${esc(de.card.back)}"><svg viewBox="0 0 120 180" aria-hidden="true"><use href="#f-back"/></svg></span>`;
}

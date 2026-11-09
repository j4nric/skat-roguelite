// The deck's order and the size of its art, shared by the runtime ids
// (./art.ts) and the build-time sprite (./sprite.ts). No rules.

import type { Rank, Suit } from '../types';
import { ART_H, ART_PAPER, ART_W } from './pixelcards.gen';

export const SUITS: Suit[] = ['Clubs', 'Spades', 'Hearts', 'Diamonds'];
export const RANKS: Rank[] = ['7', '8', '9', 'T', 'J', 'Q', 'K', 'A'];

/** The card in art pixels: 33 wide, 49.5 high (the last half row is paper). */
export const CARD_W = ART_W;
export const CARD_H = ART_H + 0.5;

/** The paper of the deck. */
export const PAPER = ART_PAPER;

const CODE: Record<Suit, string> = { Clubs: 'C', Spades: 'S', Hearts: 'H', Diamonds: 'D' };

/** The art code of a suit (C, S, H, D), as in the generated art. */
export function suitCode(suit: Suit): string {
  return CODE[suit];
}

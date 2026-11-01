// Einsatz-Leiter (DESIGN 9.3): every won run raises the win threshold of
// the next one by one Auge (61, 62, 63 …), per character and run length.
// The core keeps the ladder in its meta state (`max_stakes` of a
// character) and checks every start; this file only reads it for display.

import type { Character, MetaView, RunLength } from './types';

/** Augen a suit game or Grand needs to win at stake 0 (Skatordnung). */
export const BASE_WIN = 61;

/** Augen needed to win at `stake`, before relics. */
export function winThreshold(stake: number): number {
  return BASE_WIN + stake;
}

/** Highest stake unlocked for a character and length; 0 without a meta state. */
export function maxStake(meta: MetaView | null, character: Character, length: RunLength): number {
  const entry = meta?.characters.find((c) => c.character === character);
  return entry?.max_stakes?.[length] ?? 0;
}

/** Whether any character has climbed the ladder at all (then the start screen shows it). */
export function ladderStarted(meta: MetaView | null): boolean {
  return (meta?.characters ?? []).some((c) => Object.values(c.max_stakes ?? {}).some((n) => (n ?? 0) > 0));
}

/** The stake to play: the one picked, at most the highest unlocked; by default the highest. */
export function pickedStake(meta: MetaView | null, character: Character, length: RunLength, picked: number | null): number {
  const max = maxStake(meta, character, length);
  return picked === null ? max : Math.max(0, Math.min(picked, max));
}

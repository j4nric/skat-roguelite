// Where the loose cards lie on the felt: the two skat cards (face down
// while the player chooses, face up while he discards) and the deck. Each
// deal throws them a little differently. Purely cosmetic: the offsets come
// from Math.random, never from the core's RNG, and change nothing in the
// game.

export interface Toss {
  /** Offset in px. */
  x: number;
  y: number;
  /** Rotation in degrees. */
  r: number;
}

const COUNT = 4;

function spread(n: number): number {
  return (Math.random() * 2 - 1) * n;
}

function throwAll(): Toss[] {
  return Array.from({ length: COUNT }, (_, i) => ({
    x: Math.round(spread(5)),
    y: Math.round(spread(6)),
    // The two skat cards lean apart a little, never quite parallel.
    r: Math.round((i === 0 ? -6 : i === 1 ? 7 : 0) + spread(6)),
  }));
}

let tosses = throwAll();

/** A new deal: the cards land somewhere else. */
export function reshuffle(): void {
  tosses = throwAll();
}

/** Inline style of the loose card `i` (0 and 1: the skat). */
export function tossStyle(i: number): string {
  const t = tosses[i % COUNT] ?? { x: 0, y: 0, r: 0 };
  return `--tx:${t.x}px;--ty:${t.y}px;--tr:${t.r}deg`;
}

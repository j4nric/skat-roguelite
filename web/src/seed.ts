// Seeds to share (DESIGN 15, M5): a run is fully determined by its seed
// and character, so a link `#seed-<number>-<character>` lets someone else
// play the same run. The hash is the only part of a URL that survives every
// messenger, and it never reaches the server. Only letters, digits and
// hyphens are used. The core checks the number itself (InvalidSeed).
//
// The daily seed is the date as JJJJMMTT, so everyone who plays on the
// same day (local date) gets the same run.

import type { Character } from './types';

/** URL tokens of the characters (lower case, no umlauts). */
const SLUGS: Record<Character, string> = {
  Declarer: 'alleinspieler',
  NullPlayer: 'nullspieler',
};

/** A u64 in decimal: at most 20 digits. */
const SEED_RE = /^[0-9]{1,20}$/;

export interface SharedSeed {
  seed: string;
  character: Character;
}

/** Whether a text can be a seed (the core has the last word). */
export function isSeedText(text: string): boolean {
  return SEED_RE.test(text);
}

/** Read `#seed-123-alleinspieler` (character optional); null if it is no seed link. */
export function parseShareHash(hash: string): SharedSeed | null {
  const m = /^#?seed-([0-9]{1,20})(?:-([a-z]+))?$/i.exec(hash.trim());
  if (!m) return null;
  const seed = (m[1] ?? '').replace(/^0+(?=\d)/, '');
  const slug = (m[2] ?? SLUGS.Declarer).toLowerCase();
  const character = (Object.keys(SLUGS) as Character[]).find((c) => SLUGS[c] === slug);
  if (!character || !isSeedText(seed)) return null;
  return { seed, character };
}

/** `seed-123-alleinspieler`, without the `#`. */
export function shareHash(seed: string, character: Character): string {
  return `seed-${seed}-${SLUGS[character]}`;
}

/** The full link to this page with the seed in the hash. */
export function shareUrl(seed: string, character: Character): string {
  const { origin, pathname, search } = window.location;
  return `${origin}${pathname}${search}#${shareHash(seed, character)}`;
}

/** Remove a seed link from the address bar (after it was used or dismissed). */
export function clearShareHash(): void {
  try {
    const { pathname, search } = window.location;
    window.history.replaceState(null, '', `${pathname}${search}`);
  } catch {
    // Some embedded browsers forbid it; the offer is dismissed anyway.
  }
}

/** The daily seed: today's local date as JJJJMMTT. */
export function dailySeed(date: Date = new Date()): string {
  const y = date.getFullYear();
  const m = String(date.getMonth() + 1).padStart(2, '0');
  const d = String(date.getDate()).padStart(2, '0');
  return `${y}${m}${d}`;
}

/** The date of the daily seed for display: "01.11.2026". */
export function dailyLabel(date: Date = new Date()): string {
  return date.toLocaleDateString('de-DE', { day: '2-digit', month: '2-digit', year: 'numeric' });
}

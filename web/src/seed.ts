// Seeds to share (DESIGN 15, M5): a run is fully determined by its seed,
// character and length, so a link `#seed-<number>-<character>` lets
// someone else play the same run. A short evening (DESIGN 5.3, M7) adds
// `-kurz`; links without it are a full evening, as all links from before.
// A stake of the Einsatz-Leiter (DESIGN 9.3) adds `-einsatz<n>` at the end;
// links without it are stake 0. The hash is the only part of a URL that survives every
// messenger, and it never reaches the server. Only letters, digits and
// hyphens are used. The core checks the number itself (InvalidSeed).
//
// The daily seed is the date as JJJJMMTT, so everyone who plays on the
// same day (local date) gets the same run.

import type { Character, RunLength } from './types';

/** URL tokens of the characters (lower case, no umlauts). */
const SLUGS: Record<Character, string> = {
  Declarer: 'alleinspieler',
  NullPlayer: 'nullspieler',
};

/** URL token of the short evening; the full evening has none. */
const SHORT_SLUG = 'kurz';

/** URL token of the stake, followed by its number; stake 0 has none. */
const STAKE_SLUG = 'einsatz';

/** Highest stake a link may carry (four digits). */
const STAKE_MAX = 9999;

/** A u64 in decimal: at most 20 digits. */
const SEED_RE = /^[0-9]{1,20}$/;

export interface SharedSeed {
  seed: string;
  character: Character;
  length: RunLength;
  /** Stake of the Einsatz-Leiter, 0 for links without it. */
  stake: number;
}

/** Whether a text can be a seed (the core has the last word). */
export function isSeedText(text: string): boolean {
  return SEED_RE.test(text);
}

/**
 * Read `#seed-123-alleinspieler`, `#seed-123-alleinspieler-kurz` or
 * `#seed-123-alleinspieler-kurz-einsatz2` (character, length and stake
 * optional); null if it is no seed link.
 */
export function parseShareHash(hash: string): SharedSeed | null {
  const m = /^#?seed-([0-9]{1,20})(?:-([a-z]+))?(?:-([a-z]+))?(?:-einsatz([0-9]{1,4}))?$/i.exec(hash.trim());
  if (!m) return null;
  const seed = (m[1] ?? '').replace(/^0+(?=\d)/, '');
  let slug = (m[2] ?? SLUGS.Declarer).toLowerCase();
  let lengthSlug = m[3]?.toLowerCase() ?? null;
  // `#seed-123-kurz`: the short evening of the Alleinspieler.
  if (!m[3] && slug === SHORT_SLUG) {
    slug = SLUGS.Declarer;
    lengthSlug = SHORT_SLUG;
  }
  const character = (Object.keys(SLUGS) as Character[]).find((c) => SLUGS[c] === slug);
  if (!character || !isSeedText(seed)) return null;
  if (lengthSlug !== null && lengthSlug !== SHORT_SLUG) return null;
  const stake = Math.min(Number(m[4] ?? '0'), STAKE_MAX);
  return { seed, character, length: lengthSlug === SHORT_SLUG ? 'Short' : 'Full', stake };
}

/** `seed-123-alleinspieler`, `…-kurz`, `…-einsatz2`, without the `#`. */
export function shareHash(seed: string, character: Character, length: RunLength = 'Full', stake = 0): string {
  const short = length === 'Short' ? `-${SHORT_SLUG}` : '';
  const staked = stake > 0 ? `-${STAKE_SLUG}${Math.min(Math.floor(stake), STAKE_MAX)}` : '';
  return `seed-${seed}-${SLUGS[character]}${short}${staked}`;
}

/** The full link to this page with the seed in the hash. */
export function shareUrl(seed: string, character: Character, length: RunLength = 'Full', stake = 0): string {
  const { origin, pathname, search } = window.location;
  return `${origin}${pathname}${search}#${shareHash(seed, character, length, stake)}`;
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

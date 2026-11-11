// Seeds to share (DESIGN 15, M5): a run is fully determined by its seed
// and character, so a link `#seed-<number>-<character>` lets someone else
// play the same run. Links of the short evening (`-kurz`, dropped with
// ADR 0016) still open, as the whole evening.
// A stake of the Einsatz-Leiter (DESIGN 9.3) adds `-einsatz<n>` at the end;
// links without it are stake 0. The hash is the only part of a URL that survives every
// messenger, and it never reaches the server. Only letters, digits and
// hyphens are used. The core checks the number itself (InvalidSeed).
//
// The daily seed is the date as JJJJMMTT, so everyone who plays on the
// same day (local date) gets the same run.

import type { Character } from './types';
import { dateText } from './i18n';

/** URL tokens of the characters (lower case, no umlauts). */
const SLUGS: Record<Character, string> = {
  Declarer: 'alleinspieler',
  NullPlayer: 'nullspieler',
  Shirker: 'drueckeberger',
  Tailoress: 'schneiderin',
  HeartBrother: 'bruderherz',
  Gambler: 'zocker',
  RamschAunt: 'ramschtante',
};

/** URL token of the short evening of old links (ADR 0016: read, never written). */
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
  /** Stake of the Einsatz-Leiter, 0 for links without it. */
  stake: number;
}

/** Whether a text can be a seed (the core has the last word). */
export function isSeedText(text: string): boolean {
  return SEED_RE.test(text);
}

/**
 * Read `#seed-123-alleinspieler`, `#seed-123-alleinspieler-kurz` or
 * `#seed-123-alleinspieler-kurz-einsatz2` (character, the old length and
 * stake optional; `-kurz` opens the whole evening too, ADR 0016); null if
 * it is no seed link.
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
  return { seed, character, stake };
}

/** `seed-123-alleinspieler`, `…-einsatz2`, without the `#`. */
export function shareHash(seed: string, character: Character, stake = 0): string {
  const staked = stake > 0 ? `-${STAKE_SLUG}${Math.min(Math.floor(stake), STAKE_MAX)}` : '';
  return `seed-${seed}-${SLUGS[character]}${staked}`;
}

/** The full link to this page with the seed in the hash. */
export function shareUrl(seed: string, character: Character, stake = 0): string {
  const { origin, pathname, search } = window.location;
  return `${origin}${pathname}${search}#${shareHash(seed, character, stake)}`;
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
  return dateText(date);
}

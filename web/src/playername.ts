// Player names from building blocks (ADR 0029). A name is an adjective,
// a noun and an optional number, sent and stored as ids (`a3.n12` or
// `a3.n12.7`); the words are in the active language's texts
// (`de.playerNames`, ./i18n/de.ts and ./i18n/en.ts). Nobody types text
// that others read, so the boards hold no user-generated content. The
// server checks the same form against the same list sizes
// (server/src/names.rs). Only display, no game logic.

import { de, t } from './i18n';

/** The parts of a name: list indices and the number (1 to 99, or none). */
export interface NameParts {
  adjective: number;
  noun: number;
  number: number | null;
}

/** Largest number after a name (server/src/names.rs `MAX_NUMBER`). */
export const MAX_NAME_NUMBER = 99;

const ID = /^a(0|[1-9]\d{0,2})\.n(0|[1-9]\d{0,2})(?:\.([1-9]\d?))?$/;

/** The parts of a stored name; null for none, a free-text name from before, or ids outside the lists. */
export function parseName(id: string | null | undefined): NameParts | null {
  const m = id ? ID.exec(id) : null;
  if (!m) return null;
  const adjective = Number(m[1]);
  const noun = Number(m[2]);
  const number = m[3] ? Number(m[3]) : null;
  if (adjective >= de.playerNames.adjectives.length || noun >= de.playerNames.nouns.length) return null;
  return { adjective, noun, number };
}

/** The id the server stores. */
export function nameId(parts: NameParts): string {
  return `a${parts.adjective}.n${parts.noun}${parts.number ? `.${parts.number}` : ''}`;
}

/** Grammatical gender of a noun entry ("Wirtin|f"): index of the adjective form, 0 without a mark. */
function nounOf(entry: string): { text: string; form: number } {
  const [text = '', gender = 'm'] = entry.split('|');
  return { text, form: gender === 'f' ? 1 : gender === 'n' ? 2 : 0 };
}

/** An adjective in the form for a noun's gender ("Durstige" for "Wirtin"). */
export function adjectiveFor(adjective: number, noun: number): string {
  const forms = (de.playerNames.adjectives[adjective] ?? '').split('|');
  const form = nounOf(de.playerNames.nouns[noun] ?? '').form;
  return forms[form] ?? forms[0] ?? '';
}

/** A noun without its gender mark. */
export function nounText(noun: number): string {
  return nounOf(de.playerNames.nouns[noun] ?? '').text;
}

/** The name in the active language. */
export function nameFromParts(parts: NameParts): string {
  const name = t(de.playerNames.pattern, { adjective: adjectiveFor(parts.adjective, parts.noun), noun: nounText(parts.noun) });
  return parts.number ? t(de.playerNames.numbered, { name, n: parts.number }) : name;
}

/** A stored name as text in the active language; "Namenlos" for none or one that is not from the blocks. */
export function displayName(id: string | null | undefined): string {
  const parts = parseName(id);
  return parts ? nameFromParts(parts) : de.account.noName;
}

/** A random name without a number (the die of the picker; not game randomness). */
export function randomName(random: () => number = Math.random): NameParts {
  const pick = (n: number): number => Math.min(n - 1, Math.floor(random() * n));
  return { adjective: pick(de.playerNames.adjectives.length), noun: pick(de.playerNames.nouns.length), number: null };
}

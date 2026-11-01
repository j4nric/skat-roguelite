// Text helpers: fill placeholders and turn core identifiers into German
// display names. Only translation, no game logic.

import { de } from './de';
import type { Catalog, CardKind, Declaration, DeclaredGame, NullRisk, Rank, RiskLevel, Suit } from '../types';

export { de };

/** Fill `{name}` placeholders of a template. */
export function t(template: string, params: Record<string, string | number> = {}): string {
  return template.replace(/\{(\w+)\}/g, (match, key: string) =>
    key in params ? String(params[key]) : match,
  );
}

/** A probability from the core (thousandths) as "37,5 %". */
export function percent(permille: number): string {
  return `${(permille / 10).toLocaleString('de-DE', { maximumFractionDigits: 1 })} %`;
}

/** Suit and rank of a card kind, or null for a blank. */
export function face(kind: CardKind): { suit: Suit; rank: Rank } | null {
  if (kind === 'Blank') return null;
  const code = kind.Normal;
  const suit = ({ C: 'Clubs', S: 'Spades', H: 'Hearts', D: 'Diamonds' } as const)[code[0] as 'C' | 'S' | 'H' | 'D'];
  const rank = code[1] as Rank;
  return suit ? { suit, rank } : null;
}

/** "Eichel-Unter", "Blanko". */
export function cardName(kind: CardKind): string {
  const f = face(kind);
  if (!f) return de.card.blank;
  return t(de.card.name, { suit: de.suits[f.suit], rank: de.ranks[f.rank] });
}

/** "Eichel", "Grand", "Ramsch". */
export function gameName(game: DeclaredGame): string {
  if (game === 'Grand') return de.games.Grand;
  if (game === 'Null') return de.games.Null;
  if (game === 'Ramsch') return de.games.Ramsch;
  return de.suits[game.Suit];
}

/** "Eichel · Hand", "Null Ouvert Hand", "Ramsch". */
export function declarationName(d: Declaration): string {
  if (d.game === 'Ramsch') return de.games.Ramsch;
  if (d.game === 'Null') return de.nullRisks[d.risk as NullRisk] ?? de.games.Null;
  return t(de.table.declaration, { game: gameName(d.game), risk: de.risks[d.risk as RiskLevel] ?? d.risk });
}

type Category = keyof Catalog;

let catalog: Catalog | null = null;

/** Install the content catalog from the core. */
export function setCatalog(c: Catalog): void {
  catalog = c;
}

/** Display name of a content item, German-deck override first. */
export function contentName(category: Category, id: string): string {
  return de.content[category][id]?.name ?? catalog?.[category][id]?.name ?? id;
}

/** Display text of a content item, German-deck override first. */
export function contentText(category: Category, id: string): string {
  return de.content[category][id]?.text ?? catalog?.[category][id]?.text ?? '';
}

/**
 * Whether a card modification is a curse. The content has no type for it;
 * by convention (GLOSSARY) its text starts with "Fluch:".
 */
export function isCurse(mod: string): boolean {
  return contentText('card_mods', mod).startsWith(de.card.curseMark);
}

// Text helpers: fill placeholders and turn core identifiers into German
// display names. Only translation, no game logic.

import { de } from './de';
import type { BossInfo, Catalog, EditionEntry, CardKind, ConsumableCategory, Rarity, RelicGroup, RelicKind, ModFigure, ModKind, Declaration, DeclaredGame, NullRisk, Rank, RiskLevel, Suit } from '../types';

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

/** A factor from the core with the German decimal comma: "2", "1,5", "2,25". */
export function factorText(n: number): string {
  return n.toLocaleString('de-DE', { maximumFractionDigits: 3 });
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
 * The text of a card modification with its value filled in (Effekte v2:
 * `{n}` the amount, `{+n}` with its sign). Formatting only.
 */
export function modText(mod: string, value: number): string {
  const signed = value > 0 ? `+${value}` : value < 0 ? `−${Math.abs(value)}` : '0';
  return contentText('card_mods', mod).replaceAll('{+n}', signed).replaceAll('{n}', String(Math.abs(value)));
}

/**
 * The editions of a boss's two figures in one line (ADR 0015, ADR 0017):
 * "Boss-Decks: Der Schieber: Siebenen Augen +4 · Die Hehlerin: Achten Augen +3".
 * Empty if neither has one. Formatting only; the entries come from the core.
 */
export function bossEditionText(boss: BossInfo | null | undefined): string {
  const figures = (boss?.figures ?? []).filter((f) => f.edition.length);
  if (!figures.length) return '';
  const parts = figures.map((f) => t(de.info.bossFigureEdition, { name: f.name, entries: editionEntries(f.edition) }));
  return t(de.info.bossEdition, { entries: parts.join(' · ') });
}

/**
 * The boss with his players in one line: "Die Zwillinge: Fritz und Franz",
 * or only the players when the first is the boss ("Der Schieber und Die Hehlerin").
 */
export function bossLine(boss: BossInfo): string {
  const name = contentName('bosses', boss.id) || boss.name;
  const figures = bossFiguresText(boss);
  return boss.figures?.[0]?.name === name ? figures : `${name}: ${figures}`;
}

/** The two boss players by name ("Der Schieber und Die Hehlerin"), else the boss's name. */
export function bossFiguresText(boss: BossInfo | null | undefined): string {
  const names = (boss?.figures ?? []).map((f) => f.name);
  if (names.length < 2) return boss ? contentName('bosses', boss.id) || boss.name : '';
  return t(de.info.bossFigures, { a: names[0] ?? '', b: names[1] ?? '' });
}

function editionEntries(entries: EditionEntry[]): string {
  const rankKey: Record<string, Rank> = { Seven: '7', Eight: '8', Nine: '9', Ten: 'T', Jack: 'J', Queen: 'Q', King: 'K', Ace: 'A' };
  const parts = entries.map((e) => {
    const { suit, rank } = e.filter;
    const cards = suit && rank ? `${de.suits[suit]}-${de.ranks[rankKey[rank] ?? '7']}` : rank ? (de.info.rankPlural[rank] ?? rank) : suit ? de.suits[suit] : '';
    const value = e.card_mod.value ?? 1;
    const name = contentName('card_mods', e.card_mod.id);
    const negative = e.card_mod.id === 'battered' || e.card_mod.id === 'stained';
    const counted = ['points', 'battered', 'rank', 'trump_bonus', 'stained'].includes(e.card_mod.id);
    const mod = counted ? `${name} ${negative ? '−' : '+'}${Math.abs(value)}` : name;
    return t(de.info.bossEditionEntry, { cards, mod });
  });
  return parts.join(', ');
}

/** The number a card modification prints on its card, from the core's catalog. */
export function modFigure(mod: string): ModFigure | null {
  return catalog?.card_mods[mod]?.figure ?? null;
}

/** The run counters a relic reads or raises, from the core's catalog. */
export function relicCounters(relic: string): string[] {
  return catalog?.relics[relic]?.counters ?? [];
}

/** German names of the build directions of a relic (04.11.2026), from the core's catalog. */
export function relicDirections(relic: string): string[] {
  return (catalog?.relics[relic]?.tags ?? []).map((tag) => de.buildTags[tag] ?? tag);
}

/** Kind, rarity and group of a relic from the core's catalog, for its beer mat. */
export function relicLook(relic: string): { id: string; kind: RelicKind | null; rarity: Rarity | null; group: RelicGroup | null } {
  const e = catalog?.relics[relic];
  return { id: relic, kind: e?.kind ?? null, rarity: e?.rarity ?? null, group: e?.group ?? null };
}

/** Group and rarity of a consumable from the core's catalog, for its beer mat. */
export function consumableLook(item: string): { id: string; category: ConsumableCategory | null; rarity: Rarity | null } {
  const e = catalog?.consumables[item];
  return { id: item, category: e?.category ?? null, rarity: e?.rarity ?? null };
}

/** Ids of every card modification in the core's catalog. */
export function cardModIds(): string[] {
  return Object.keys(catalog?.card_mods ?? {});
}

/** The first value a card modification is offered with (1 without values), from the core's catalog. */
export function modFirstValue(mod: string): number {
  return catalog?.card_mods[mod]?.values?.[0] ?? 1;
}

/** Whether a card modification is a curse (Fluch), from the core's catalog. */
/** Rand, Moment, Gutschein, Fluch or Spezialkarte, from the core's catalog. */
export function modKind(mod: string): ModKind | null {
  return catalog?.card_mods[mod]?.kind ?? null;
}

/** The consumable's card modification as the catalog names it (null: none or drawn at random). */
export function consumableMod(id: string): { mod: string | null; random: boolean } {
  const entry = catalog?.consumables[id];
  return { mod: entry?.card_mod ?? null, random: entry?.random_card_mod === true };
}

/** Whether one of the cards carries a Marke (Gutschein): a hand game would redeem one (DESIGN 7.2). */
export function holdsVoucher(cards: { mods: string[] }[]): boolean {
  return cards.some((c) => c.mods.some((m) => modKind(m) === 'Voucher'));
}

export function isCurse(mod: string): boolean {
  return catalog?.card_mods[mod]?.curse === true;
}

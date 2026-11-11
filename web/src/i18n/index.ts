// Text helpers: the active language (German by default, English as the
// second language, ADR 0023), placeholders, number formats and the display
// names of core identifiers. Only translation, no game logic.
//
// `de` is the strings of the ACTIVE language, not always German: it is a
// live ES module binding that `setLanguage` points at ./de.ts or ./en.ts.
// The name stays so that the many `de.…` call sites (and branches written
// against them) keep working; read it as "the display texts".

import { de as deStrings } from './de';
import { fillModValue, localizeContent, type ContentTexts } from './localize';
import { readKey, writeKey } from '../storage';
import type { BossInfo, Catalog, EditionEntry, CardKind, ConsumableCategory, Rarity, RelicGroup, RelicKind, ModFigure, ModKind, Declaration, DeclaredGame, NullRisk, Rank, RiskLevel, Suit } from '../types';

/** Deep shape of the German strings with every text widened to `string`: what every language must provide. */
type Widen<T> = T extends string
  ? string
  : T extends readonly unknown[]
    ? { [K in keyof T]: Widen<T[K]> }
    : T extends object
      ? { [K in keyof T]: Widen<T[K]> }
      : T;

/** The texts of one language; ./en.ts is typed against it, so a missing key fails `tsc`. */
export type Strings = Widen<typeof deStrings>;

export type Lang = 'de' | 'en';
export const LANGS: readonly Lang[] = ['de', 'en'];

/**
 * The loaded languages. German is built in; English is a chunk of its own,
 * fetched only when it is chosen (so German players do not download it).
 */
const PACKS: Partial<Record<Lang, Strings>> = { de: deStrings };
/** Locale for numbers and dates: decimal comma in German, point in English. */
const LOCALES: Record<Lang, string> = { de: 'de-DE', en: 'en-GB' };
/** Display texts of the core's content that the language replaces (German comes from the core). */
const CONTENT: Partial<Record<Lang, ContentTexts>> = {};
const LANG_KEY = 'stammtisch.lang';

let current: Lang = 'de';

/** The texts of the active language (see the note at the top). */
export let de: Strings = deStrings;

/** The active language. */
export function lang(): Lang {
  return current;
}

/** Locale tag of the active language for `toLocaleString` and `Intl`. */
export function locale(): string {
  return LOCALES[current];
}

/** The language chosen in the menu, else the browser's: German for German browsers, English for all others. */
export function storedOrDetectedLanguage(): Lang {
  const stored = readKey(LANG_KEY);
  if (stored === 'de' || stored === 'en') return stored;
  let wanted: readonly string[] = [];
  try {
    wanted = navigator.languages?.length ? navigator.languages : [navigator.language];
  } catch {
    wanted = [];
  }
  const first = (wanted[0] ?? 'de').toLowerCase();
  return first.startsWith('de') || first === '' ? 'de' : 'en';
}

/** Fetch a language's texts once (English: its own chunk). */
async function loadPack(next: Lang): Promise<Strings> {
  const have = PACKS[next];
  if (have) return have;
  const [{ en }, { EN_CONTENT }] = await Promise.all([import('./en'), import('./en-content')]);
  PACKS.en = en;
  CONTENT.en = EN_CONTENT;
  return en;
}

/**
 * Switch the display language (fetching its texts first). `remember`: the
 * player chose it (menu), so it is stored like the other settings; the
 * detected one is not stored. Should English fail to load, German stays.
 */
export async function setLanguage(next: Lang, remember = false): Promise<void> {
  const pack = await loadPack(next);
  current = next;
  de = pack;
  if (remember) writeKey(LANG_KEY, next);
  try {
    document.documentElement.lang = next;
    document.querySelector('meta[name="description"]')?.setAttribute('content', de.lang.description);
  } catch {
    // No document (tests): nothing to mark.
  }
}

/**
 * A view, meta view or catalog from the core with the content texts of the
 * active language in place (in place; German: the core's own texts).
 */
export function localize<T>(value: T): T {
  return localizeContent(value, current === 'de' ? null : (CONTENT[current] ?? null));
}

/** Fill `{name}` placeholders of a template. */
export function t(template: string, params: Record<string, string | number> = {}): string {
  return template.replace(/\{(\w+)\}/g, (match, key: string) =>
    key in params ? String(params[key]) : match,
  );
}

/** A probability from the core (thousandths) as "37,5 %". */
export function percent(permille: number): string {
  return t(de.lang.percent, { n: (permille / 10).toLocaleString(locale(), { maximumFractionDigits: 1 }) });
}

/** A number in the active language ("1,5" or "1.5"). */
export function num(n: number, digits = 3): string {
  return n.toLocaleString(locale(), { maximumFractionDigits: digits });
}

/** A date as day, month, year in the active language ("01.11.2026", "01/11/2026"). */
export function dateText(date: Date, opts: Intl.DateTimeFormatOptions = { day: '2-digit', month: '2-digit', year: 'numeric' }): string {
  return date.toLocaleDateString(locale(), opts);
}

/** A line someone says, in the language's quotation marks. */
export function quote(text: string): string {
  return t(de.lang.quote, { text });
}

/** A factor from the core in the language's decimals: "2", "1,5", "2,25" (German). */
export function factorText(n: number): string {
  return num(n, 3);
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
  return de.content[category]?.[id]?.name ?? catalog?.[category]?.[id]?.name ?? id;
}

/** Display text of a content item, German-deck override first. */
export function contentText(category: Category, id: string): string {
  return de.content[category]?.[id]?.text ?? catalog?.[category]?.[id]?.text ?? '';
}

/**
 * The text of a card modification with its value filled in (Effekte v2:
 * `{n}` the amount, `{+n}` with its sign). Formatting only.
 */
export function modText(mod: string, value: number): string {
  return fillModValue(contentText('card_mods', mod), value);
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
    const cards = suit && rank ? t(de.card.name, { suit: de.suits[suit], rank: de.ranks[rankKey[rank] ?? '7'] }) : rank ? (de.info.rankPlural[rank] ?? rank) : suit ? de.suits[suit] : '';
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
  const tags = (catalog?.relics[relic]?.tags ?? []).map((tag) => de.buildTags[tag] ?? tag);
  // The relics of the long night (ADR 0020) say so.
  return catalog?.relics[relic]?.night ? [...tags, de.night.pool] : tags;
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

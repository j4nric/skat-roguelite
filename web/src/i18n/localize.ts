// The core's content speaks German: names and texts of relics, card
// modifications, Schnäpse, bosses, encounters, unlocks, maluses, the
// Stammgäste's lines and so on come out of `core/content/*.ron` inside
// every view. For another language this walks a view (or the meta view,
// or the catalog) and puts the display texts of that language in place,
// looked up by content id (ADR 0023). The German originals are kept on the
// side, so a switch back to German restores them. Only display texts are
// touched; ids and numbers stay as the core sent them.

/** Name and text of one content item. */
export interface ContentEntry {
  name?: string;
  text?: string;
}

/** The content texts of one language other than German. */
export interface ContentTexts {
  /** Relics, card modifications, Schnäpse, bosses, encounters, unlocks, maluses, Verschärfungen, card backs, by id (ids are unique across the content files). */
  byId: Record<string, ContentEntry>;
  /** Characters by their core name (`Declarer`, …). */
  characters: Record<string, ContentEntry>;
  /** Stammgäste by id: name and the three lines of their bubble. */
  guests: Record<string, { name?: string; greet?: string; win?: string; lose?: string }>;
  /** Encounter option titles by encounter id, in the order of the content file. */
  options: Record<string, string[]>;
  /** Boss players by their German name (they have no id of their own). */
  figures: Record<string, string>;
  /** Districts of the made-up city by their German name. */
  districts: Record<string, string>;
}

type Obj = Record<string, unknown>;

/**
 * The text of a card modification with its value filled in, as the core
 * fills it (`{n}` the amount, `{+n}` with its sign). Formatting only.
 */
export function fillModValue(text: string, value: number): string {
  const signed = value > 0 ? `+${value}` : value < 0 ? `−${Math.abs(value)}` : '0';
  return text.replaceAll('{+n}', signed).replaceAll('{n}', String(Math.abs(value)));
}

/** German originals of every field replaced, per object. */
const originals = new WeakMap<object, Obj>();

const isObj = (v: unknown): v is Obj => typeof v === 'object' && v !== null;
const DISTRICT_LOOKS = new Set(['OldTown', 'Harbour', 'Station']);

function set(node: Obj, key: string, value: string | undefined): void {
  if (value === undefined || typeof node[key] !== 'string' || node[key] === value) return;
  let saved = originals.get(node);
  if (!saved) {
    saved = {};
    originals.set(node, saved);
  }
  if (!(key in saved)) saved[key] = node[key];
  node[key] = value;
}

function restore(node: Obj): void {
  const saved = originals.get(node);
  if (!saved) return;
  for (const [key, value] of Object.entries(saved)) node[key] = value;
}

function rewardEntry(reward: unknown, texts: ContentTexts): ContentEntry | undefined {
  if (!isObj(reward)) return undefined;
  if (typeof reward.Character === 'string') return texts.characters[reward.Character];
  for (const k of ['Relic', 'CardMod', 'CardBack']) {
    const id = reward[k];
    if (typeof id === 'string') return texts.byId[id];
  }
  return undefined;
}

function apply(node: Obj, texts: ContentTexts): void {
  const id = typeof node.id === 'string' ? node.id : null;
  // A Stammgast: name and bubble lines.
  if (id && typeof node.look === 'string' && typeof node.greet === 'string') {
    const g = texts.guests[id];
    if (g) {
      set(node, 'name', g.name);
      set(node, 'greet', g.greet);
      set(node, 'win', g.win);
      set(node, 'lose', g.lose);
    }
    return;
  }
  // A district of the city: its name by look.
  if (typeof node.look === 'string' && DISTRICT_LOOKS.has(node.look) && typeof node.name === 'string') {
    set(node, 'name', texts.districts[node.name]);
  }
  if (typeof node.district === 'string') set(node, 'district', texts.districts[node.district]);
  const entry = id ? texts.byId[id] : undefined;
  if (entry) {
    set(node, 'name', entry.name);
    // A card modification's text comes from the core with its value filled in (CardModInfo).
    const value = typeof node.value === 'number' ? node.value : 1;
    set(node, 'text', entry.text !== undefined && typeof node.curse === 'boolean' ? fillModValue(entry.text, value) : entry.text);
  }
  // A character (CharacterInfo, CharacterEntry).
  if (typeof node.character === 'string' && typeof node.name === 'string' && typeof node.text === 'string') {
    const c = texts.characters[node.character];
    set(node, 'name', c?.name);
    set(node, 'text', c?.text);
  }
  // An unlock's reward (RewardInfo).
  if ('reward' in node && typeof node.name === 'string') {
    const r = rewardEntry(node.reward, texts);
    set(node, 'name', r?.name);
    set(node, 'text', r?.text);
  }
  // A boss player: name and edition, no id.
  if (!id && Array.isArray(node.edition) && typeof node.name === 'string') set(node, 'name', texts.figures[node.name]);
  // An open encounter: its options' titles.
  if (id && Array.isArray(node.options)) {
    const labels = texts.options[id];
    if (labels) {
      for (const option of node.options) {
        if (isObj(option) && typeof option.index === 'number') set(option, 'label', labels[option.index]);
      }
    }
  }
}

/** Every object of a view once (a view may share objects). */
function walk(value: unknown, visit: (node: Obj) => void, seen: Set<object>): void {
  if (!isObj(value) || seen.has(value)) return;
  seen.add(value);
  if (Array.isArray(value)) {
    for (const v of value) walk(v, visit, seen);
    return;
  }
  visit(value);
  for (const v of Object.values(value)) walk(v, visit, seen);
}

/**
 * Put the display texts of a language into a view from the core, in place
 * (null: German, the core's own texts, restored where they were replaced).
 */
export function localizeContent<T>(value: T, texts: ContentTexts | null): T {
  // First the German everywhere, then the other language: a parent may set
  // texts of its children (an encounter its options' titles).
  walk(value, restore, new Set());
  if (texts) walk(value, (node) => apply(node, texts), new Set());
  return value;
}

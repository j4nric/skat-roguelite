// The two languages of the web app (ADR 0022) must say the same things:
// ./src/i18n/de.ts and ./src/i18n/en.ts have the same keys (tsc checks the
// typed ones; this also covers the open maps like the error texts), every
// text has the same placeholders ({n}, {name}, …) in both, and arrays have
// the same length. The English content texts (./src/i18n/en-content.ts)
// cover every id of core/content/*.ron with the same placeholders as the
// German text there. Run by `npm run typecheck`.

import { readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { rolldown } from 'rolldown';

const web = join(dirname(fileURLToPath(import.meta.url)), '..');
const contentDir = join(web, '..', 'core', 'content');

/** Load a TypeScript module of src/i18n through rolldown (no imports of the app run). */
async function load(file) {
  const bundle = await rolldown({ input: join(web, 'src', 'i18n', file), logLevel: 'silent' });
  const { output } = await bundle.generate({ format: 'esm' });
  await bundle.close();
  return import(`data:text/javascript;base64,${Buffer.from(output[0].code).toString('base64')}`);
}

const placeholders = (s) => [...s.matchAll(/\{([+\w]+)\}/g)].map((m) => m[1]).sort().join(',');
const problems = [];

function compare(a, b, path) {
  if (typeof a === 'string' || typeof b === 'string') {
    if (typeof a !== typeof b) return problems.push(`${path}: a text in one language, something else in the other`);
    if (placeholders(a) !== placeholders(b)) problems.push(`${path}: placeholders differ: de {${placeholders(a)}} en {${placeholders(b)}}`);
    return;
  }
  if (Array.isArray(a) || Array.isArray(b)) {
    if (!Array.isArray(a) || !Array.isArray(b) || a.length !== b.length) return problems.push(`${path}: arrays of different length`);
    a.forEach((v, i) => compare(v, b[i], `${path}[${i}]`));
    return;
  }
  if (a && b && typeof a === 'object' && typeof b === 'object') {
    for (const k of Object.keys(a)) if (!(k in b)) problems.push(`${path}.${k}: missing in en`);
    for (const k of Object.keys(b)) if (!(k in a)) problems.push(`${path}.${k}: only in en`);
    for (const k of Object.keys(a)) if (k in b && !PER_LANGUAGE.has(`${path}.${k}`)) compare(a[k], b[k], `${path}.${k}`);
    return;
  }
  if (a !== b) problems.push(`${path}: differs (${a} / ${b})`);
}

const { de } = await load('de.ts');
const { en } = await load('en.ts');
const { EN_CONTENT, EN_CATALOG } = await load('en-content.ts');

// The keyword maps and the content overrides are per language by design:
// their keys are words of that language, or content ids that only one
// language replaces. Everything else must match key for key.
// `rules.words` explains the Skat words that stay German: only other languages need it.
const PER_LANGUAGE = new Set(['keywords', 'keywordForms', 'content', 'rules.words']);
for (const k of Object.keys(de)) {
  if (!(k in en)) problems.push(`${k}: missing in en`);
  else if (!PER_LANGUAGE.has(k)) compare(de[k], en[k], k);
}
for (const k of Object.keys(en)) if (!(k in de)) problems.push(`${k}: only in en`);
// Every keyword form points at a keyword of its language.
for (const [name, lang] of [['de', de], ['en', en]]) {
  for (const [form, key] of Object.entries(lang.keywordForms)) if (!lang.keywords[key]) problems.push(`${name}.keywordForms.${form}: no keyword ${key}`);
}

// The content files: ids with name and text, read loosely (RON).
function ron(file) {
  const src = readFileSync(join(contentDir, file), 'utf8')
    .split('\n')
    .filter((l) => !l.trim().startsWith('//'))
    .join('\n');
  return [...src.matchAll(/\b(id|name|text|label|character)\s*:\s*(?:Some\()?(?:"((?:[^"\\]|\\.)*)"|([A-Za-z_]+))/g)].map((m) => [m[1], m[2] ?? m[3]]);
}
function entries(file, key = 'id') {
  const out = {};
  let id = null;
  for (const [k, v] of ron(file)) {
    if (k === key) {
      id = v;
      out[id] = {};
    } else if (id && (k === 'name' || k === 'text') && !(k in out[id])) out[id][k] = v;
  }
  return out;
}
const files = {
  relics: 'house_rules.ron',
  card_mods: 'card_mods.ron',
  consumables: 'consumables.ron',
  bosses: 'bosses.ron',
  encounters: 'encounters.ron',
  unlocks: 'unlocks.ron',
};
const check = (where, german, english) => {
  for (const [id, g] of Object.entries(german)) {
    const e = english[id];
    if (!e) {
      problems.push(`${where}.${id}: no English entry`);
      continue;
    }
    for (const f of ['name', 'text']) {
      if (g[f] !== undefined && !e[f]) problems.push(`${where}.${id}.${f}: no English ${f}`);
      if (g[f] !== undefined && e[f] && placeholders(g[f]) !== placeholders(e[f])) problems.push(`${where}.${id}.${f}: placeholders differ from the content file`);
    }
  }
};
for (const [cat, file] of Object.entries(files)) check(cat, entries(file), EN_CATALOG[cat]);
check('characters', entries('characters.ron', 'character'), EN_CATALOG.characters);
// Maluses, Verschärfungen and card backs sit in nested lists of tour.ron and night.ron.
for (const file of ['tour.ron', 'night.ron']) {
  const all = entries(file);
  for (const [id, g] of Object.entries(all)) if (g.name !== undefined) check(file, { [id]: g }, EN_CONTENT.byId);
}
// Stammgäste and encounter options.
for (const id of Object.keys(entries('guests.ron'))) if (!EN_CONTENT.guests[id]?.greet) problems.push(`guests.${id}: no English lines`);
{
  const src = readFileSync(join(contentDir, 'encounters.ron'), 'utf8');
  let id = null;
  const labels = {};
  for (const m of src.matchAll(/\b(id|label)\s*:\s*"([^"]*)"/g)) {
    if (m[1] === 'id') labels[(id = m[2])] = [];
    else if (id) labels[id].push(m[2]);
  }
  for (const [eid, list] of Object.entries(labels)) {
    if ((EN_CONTENT.options[eid] ?? []).length !== list.length) problems.push(`options.${eid}: ${list.length} options in the content, ${(EN_CONTENT.options[eid] ?? []).length} in English`);
  }
}

if (problems.length) {
  console.error(`check-i18n: ${problems.length} problem(s)\n${problems.map((p) => `  ${p}`).join('\n')}`);
  process.exit(1);
}
console.log(`check-i18n: de and en agree (${Object.keys(de).length} sections), English content covers core/content`);

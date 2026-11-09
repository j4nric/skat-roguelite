// Every `data-action` the templates use must have a handler in the action
// table of src/app/ (src/app/actions.ts), and every handler should be
// used. Run by `npm run typecheck`.
//
// The templates name an action in four ways, all read here:
// - `data-action="name"` in HTML text,
// - `'data-action': 'name'` (or a condition with names) in attrs({…}),
// - `'data-action': param` where `param` is a parameter of the enclosing
//   function: the string literals passed for it in the same file,
// - `'data-action': local` where `local` is a variable of the enclosing
//   function: the string literals assigned to it there.
// ./ui/keys.ts presses buttons by name (`press('name')`), which counts too.
// Handlers are the quoted keys of the tables in src/app/ (`'name': …`).

import { readdirSync, readFileSync, statSync } from 'node:fs';
import { dirname, join, relative } from 'node:path';
import { fileURLToPath } from 'node:url';

const src = join(dirname(fileURLToPath(import.meta.url)), '..', 'src');
const NAME = /^[a-z][a-z0-9-]*$/;

function files(dir) {
  return readdirSync(dir).flatMap((name) => {
    const path = join(dir, name);
    if (statSync(path).isDirectory()) return name === 'wasm' ? [] : files(path);
    return path.endsWith('.ts') ? [path] : [];
  });
}

const literals = (text) => [...text.matchAll(/'([^'\n]*)'/g)].map((m) => m[1]).filter((s) => NAME.test(s));

/** The text of the function around `index`: from its `function name(` to the next top-level function. */
function enclosing(text, index) {
  const starts = [...text.matchAll(/^(?:export )?function (\w+)\(([^)]*)\)/gm)];
  let found = null;
  for (const m of starts) if (m.index <= index) found = m;
  if (!found) return null;
  const next = starts.find((m) => m.index > found.index);
  return { name: found[1], params: found[2], body: text.slice(found.index, next ? next.index : text.length) };
}

/** Split the arguments of a call at its top-level commas. */
function argsAt(text, open) {
  const args = [];
  let depth = 0;
  let quote = null;
  let start = open + 1;
  for (let i = open + 1; i < text.length; i++) {
    const c = text[i];
    if (quote) {
      if (c === '\\') i++;
      else if (c === quote) quote = null;
      continue;
    }
    if (c === "'" || c === '"' || c === '`') quote = c;
    else if ('([{'.includes(c)) depth++;
    else if (')]}'.includes(c)) {
      if (depth === 0) {
        args.push(text.slice(start, i).trim());
        return args;
      }
      depth--;
    } else if (c === ',' && depth === 0) {
      args.push(text.slice(start, i).trim());
      start = i + 1;
    }
  }
  return args;
}

const used = new Map();
const problems = [];
const note = (name, where) => {
  if (!used.has(name)) used.set(name, where);
};

for (const path of files(src)) {
  const rel = relative(src, path);
  if (rel.startsWith('app')) continue;
  const text = readFileSync(path, 'utf8');
  for (const m of text.matchAll(/data-action="([^"$]+)"/g)) note(m[1], rel);
  for (const m of text.matchAll(/'data-action':\s*([^,}\n]+)/g)) {
    const expr = m[1].trim();
    const names = literals(expr);
    if (names.length) {
      for (const n of names) note(n, rel);
      continue;
    }
    const ident = /^\w+$/.test(expr) ? expr : null;
    const fn = ident ? enclosing(text, m.index) : null;
    if (!ident || !fn) {
      problems.push(`${rel}: cannot read the action of 'data-action': ${expr}`);
      continue;
    }
    const params = fn.params.split(',').map((p) => p.split(':')[0].trim());
    const at = params.indexOf(ident);
    const found = [];
    if (at >= 0) {
      for (const call of text.matchAll(new RegExp(`(?<![.\\w])${fn.name}\\(`, 'g'))) {
        const arg = argsAt(text, call.index + call[0].length - 1)[at] ?? '';
        found.push(...literals(arg));
      }
    } else {
      for (const set of fn.body.matchAll(new RegExp(`\\b${ident}\\b(?:\\s*:[^=;\\n]+)?\\s*=(?![=>])\\s*([^;\\n]+);`, 'g'))) found.push(...literals(set[1]));
    }
    if (!found.length) problems.push(`${rel}: no action names found for 'data-action': ${ident} in ${fn.name}()`);
    for (const n of found) note(n, rel);
  }
  if (rel === join('ui', 'keys.ts')) for (const m of text.matchAll(/\bpress\('([^']+)'\)/g)) note(m[1], rel);
}

const handled = new Set();
for (const path of files(join(src, 'app'))) {
  const text = readFileSync(path, 'utf8');
  for (const m of text.matchAll(/^\s+'([a-z][a-z0-9-]*)':\s/gm)) handled.add(m[1]);
}

for (const [name, where] of used) if (!handled.has(name)) problems.push(`${where}: data-action "${name}" has no handler in src/app/`);
const unused = [...handled].filter((name) => !used.has(name)).sort();

if (unused.length) console.warn(`check-actions: handlers without a data-action in the templates: ${unused.join(', ')}`);
if (problems.length) {
  console.error(problems.join('\n'));
  process.exit(1);
}
console.log(`check-actions: ${used.size} actions in the templates, all with a handler (${handled.size} handlers)`);

// The Regelkarte (menu): how a game runs, the Spielwert, what is bent
// compared with the Skatordnung, and the structure of a run. Only text from
// the translation file; one compact card that scrolls inside.

import { de } from '../i18n';
import type { Suit } from '../types';
import { suitIcon } from './cards';
import { attrs, esc } from './dom';

function isSuit(key: string): key is Suit {
  return key in de.suits;
}

/** Display name of a game in the Grundwert list: the German suit, Grand, Null. */
function baseName(key: string): string {
  if (isSuit(key)) return de.suits[key];
  return key === 'Grand' || key === 'Null' ? de.games[key] : key;
}

function terms(rows: [string, string][], cls: string): string {
  return `<dl class="rules-terms ${cls}">${rows.map(([k, v]) => `<dt>${esc(k)}</dt><dd>${esc(v)}</dd>`).join('')}</dl>`;
}

export function renderRules(): string {
  const r = de.rules;
  const flow = r.flow.map(([k, v]) => `<li><b>${esc(k)}</b><span>${esc(v)}</span></li>`).join('');
  const bases = r.bases
    .map(([key, value]) => {
      const icon = isSuit(key) ? suitIcon(key, 14, null) : '<span class="rules-nosuit"></span>';
      return `<li>${icon}<span>${esc(baseName(key))}</span><b>${esc(value)}</b></li>`;
    })
    .join('');
  const list = (items: readonly string[]): string => items.map((i) => `<li>${esc(i)}</li>`).join('');
  return `<h2>${esc(r.title)}</h2>
    <div class="rules-body" data-testid="rules">
      <p class="rules-intro">${esc(r.intro)}</p>
      <h3 class="rules-h">${esc(r.flowTitle)}</h3>
      <ol class="rules-flow">${flow}</ol>
      <h3 class="rules-h">${esc(r.valueTitle)}</h3>
      <p class="rules-formula">${esc(r.valueFormula)}</p>
      <h4 class="rules-sub">${esc(r.baseTitle)}</h4>
      <ul class="rules-bases">${bases}</ul>
      <p class="rules-note">${esc(r.nullNote)}</p>
      <h4 class="rules-sub">${esc(r.levelsTitle)}</h4>
      <ul class="rules-list">${list(r.levels)}</ul>
      <p class="rules-note is-warn">${esc(r.lostNote)}</p>
      <h3 class="rules-h">${esc(r.bentTitle)}</h3>
      <ul class="rules-list">${list(r.bent)}</ul>
      <h3 class="rules-h">${esc(r.runTitle)}</h3>
      ${terms(r.run, 'rules-run')}
    </div>
    <button class="btn btn-wide" ${attrs({ 'data-action': 'close-overlay', 'data-testid': 'rules-close' })}>${esc(de.app.close)}</button>`;
}

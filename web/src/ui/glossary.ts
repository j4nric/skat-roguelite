// The Glossar (menu): every Zinke (card modification) of the content,
// grouped like the frame language of the cards (Rand, Ecke, Marke,
// Spezialkarten, Flüche), each with a sample card drawn by the real
// renderer, its name and its effect text from the core's catalog. On top
// the legend of the value icons. A tap on an entry opens its info card
// (`mod:<id>`, ./info.ts). Only lookups and labels: which Zinken exist,
// their kind, values and texts come from the catalog.

import { de, cardModIds, contentName, isCurse, modFirstValue, modKind, modText } from '../i18n';
import type { CardKind, ModKind } from '../types';
import { cardSvg } from './cards';
import { attrs, esc } from './dom';
import { MOD_DRAW, valueIcon } from './fxart';
import { coinIcon } from './hud';

/** The groups in the order of the frame language; a special curse counts as a curse. */
type Group = keyof typeof de.glossary.groups;
const GROUPS: Group[] = ['Edge', 'Moment', 'Voucher', 'Special', 'Curse'];

/** The card every sample is drawn on: a plain number card shows frames and sketches best. */
const SAMPLE: CardKind = { Normal: 'S9' };

/** Display order inside a group: the order of the card art, special cards as on the board, the rest by name. */
const ORDER = [...Object.keys(MOD_DRAW), 'wild_card', 'undead', 'doppelganger'];

function groupOf(kind: ModKind | null, curse: boolean): Group {
  if (kind === 'SpecialCurse' || kind === 'Curse' || (!kind && curse)) return 'Curse';
  return kind ?? 'Edge';
}

function rankOf(mod: string): number {
  const i = ORDER.indexOf(mod);
  return i < 0 ? ORDER.length : i;
}

/** The sample card of a modification as inline SVG, with its first value. */
export function glossarySample(mod: string): string {
  return cardSvg(SAMPLE, [mod], 'full', { [mod]: modFirstValue(mod) });
}

/** The group heading of a modification (for its info card). */
export function glossaryGroup(mod: string): string {
  return de.glossary.groups[groupOf(modKind(mod), isCurse(mod))][0];
}

/** The effect text of a modification with its first value. */
export function glossaryText(mod: string): string {
  return modText(mod, modFirstValue(mod));
}

function entry(mod: string): string {
  const name = contentName('card_mods', mod);
  return `<li><button type="button" class="glossary-entry" ${attrs({ 'data-action': 'info-card', 'data-info': `mod:${mod}`, 'data-mod': mod, 'aria-label': name })}>
    <span class="card glossary-card">${glossarySample(mod)}</span>
    <span class="glossary-text"><b>${esc(name)}</b><span>${esc(glossaryText(mod))}</span></span>
  </button></li>`;
}

function legend(): string {
  const v = de.glossary.values;
  const rows: [string, readonly string[]][] = [
    [coinIcon(18), v.coins],
    [valueIcon('pips', 18), v.pips],
    [valueIcon('base', 18), v.base],
    [valueIcon('mult', 18), v.mult],
  ];
  return `<ul class="glossary-values" data-testid="glossary-values">${rows
    .map(([icon, [name, line]]) => `<li><span class="glossary-icon">${icon}</span><b>${esc(name ?? '')}</b><span>${esc(line ?? '')}</span></li>`)
    .join('')}</ul>`;
}

export function renderGlossary(): string {
  const g = de.glossary;
  const byGroup = new Map<Group, string[]>(GROUPS.map((k) => [k, []]));
  for (const mod of cardModIds()) byGroup.get(groupOf(modKind(mod), isCurse(mod)))?.push(mod);
  const sections = GROUPS.map((k) => {
    const mods = (byGroup.get(k) ?? []).sort((a, b) => rankOf(a) - rankOf(b) || contentName('card_mods', a).localeCompare(contentName('card_mods', b), 'de'));
    if (!mods.length) return '';
    const [title, line] = g.groups[k];
    return `<section class="glossary-group" ${attrs({ 'data-group': k, 'data-testid': `glossary-${k}` })}>
      <h3 class="rules-h">${esc(title)}</h3>
      <p class="glossary-lead">${esc(line)}</p>
      <ul class="glossary-list">${mods.map(entry).join('')}</ul>
    </section>`;
  }).join('');
  return `<h2>${esc(g.title)}</h2>
    <div class="rules-body glossary-body" data-testid="glossary">
      <p class="rules-intro">${esc(g.intro)}</p>
      <h3 class="rules-h">${esc(g.valuesTitle)}</h3>
      ${legend()}
      ${sections}
    </div>
    <button class="btn btn-wide" ${attrs({ 'data-action': 'close-overlay', 'data-testid': 'glossary-close' })}>${esc(de.app.close)}</button>`;
}

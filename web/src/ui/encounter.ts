// An encounter (Ereignis) between two series, in the style of the pub: a
// chalk sign, the people who speak, the story from the content and the
// options as cards on the table. What each option does, what it costs and
// whether it can be chosen all come from the core (EncounterView).

import { de, contentName, contentText, t } from '../i18n';
import type { EncounterOptionView, RunView } from '../types';
import { attrs, esc } from './dom';
import { coinIcon } from './hud';
import { encounterFaces, portrait } from './people';
import { coasterEmblem } from './shop';

/** A beer glass, for "Letzte Runde". */
function glass(): string {
  return `<svg class="enc-glass" viewBox="0 0 40 52" width="34" height="44" aria-hidden="true"><path d="M7 10 L33 10 L30 48 L10 48 Z" fill="#e8a417" stroke="#2b1a0e" stroke-width="2"/><path d="M5 10 C5 3 12 2 15 5 C18 1 25 1 27 5 C31 2 36 5 35 10 Z" fill="#fffaf0" stroke="#2b1a0e" stroke-width="2"/><path d="M33 18 C40 18 40 34 32 34" fill="none" stroke="#2b1a0e" stroke-width="2.4"/><path d="M14 18 L13 42 M20 18 L20 42" stroke="#fff" stroke-width="1.6" opacity=".45"/></svg>`;
}

/** A marked card, for the Kartengeber. */
function markedCard(chosen: boolean): string {
  return `<svg class="enc-card" viewBox="0 0 40 52" width="34" height="44" aria-hidden="true"><rect x="5" y="3" width="30" height="45" rx="4" fill="#fbf3e2" stroke="#2b1a0e" stroke-width="2"/><path d="M12 10 L28 10 M12 15 L22 15" stroke="#c9b78f" stroke-width="2"/><circle cx="20" cy="30" r="8" fill="none" stroke="#5e3796" stroke-width="2.4"/><path d="M20 24 L22 29 L27 29 L23 32 L24.5 37 L20 34 L15.5 37 L17 32 L13 29 L18 29 Z" fill="#5e3796"/>${chosen ? '<path d="M29 40 L37 48" stroke="#2b1a0e" stroke-width="3" stroke-linecap="round"/>' : '<text x="31" y="47" font-size="12" font-weight="700" fill="#2b1a0e" text-anchor="middle">?</text>'}</svg>`;
}

/** A door, for walking on. */
function door(): string {
  return `<svg class="enc-door" viewBox="0 0 40 52" width="34" height="44" aria-hidden="true"><rect x="7" y="3" width="26" height="46" rx="2" fill="#6b4427" stroke="#2b1a0e" stroke-width="2"/><rect x="11" y="8" width="18" height="16" fill="#e8c56a" opacity=".7"/><circle cx="27" cy="31" r="2" fill="#e2b04a"/></svg>`;
}

function costTag(o: EncounterOptionView): string {
  if (!o.cost) return `<span class="enc-cost is-free">${esc(de.encounter.free)}</span>`;
  return `<span class="enc-cost">${coinIcon(14)}${esc(t(de.encounter.cost, { cost: o.cost }))}</span>`;
}

/** Title, hint and picture of one option. */
function optionParts(o: EncounterOptionView): { title: string; hint: string; pic: string; cls: string } {
  const c = o.choice;
  if (c === 'Leave') return { title: de.encounter.leave, hint: de.encounter.leaveHint, pic: door(), cls: 'is-leave' };
  if (c === 'MarkRandomCard') return { title: de.encounter.markRandom, hint: de.encounter.markRandomHint, pic: markedCard(false), cls: '' };
  if (c === 'MarkChosenCard') return { title: de.encounter.markChosen, hint: de.encounter.markChosenHint, pic: markedCard(true), cls: '' };
  if ('DoubleNextGame' in c) {
    return { title: de.encounter.double, hint: t(de.encounter.doubleHint, { factor: c.DoubleNextGame }), pic: glass(), cls: '' };
  }
  const relic = o.relic;
  const id = relic?.id ?? ('TakeRule' in c ? c.TakeRule : c.SwapRelic.relic);
  const name = contentName('relics', id);
  const pic = `<span class="enc-coaster rarity-${esc(relic?.rarity ?? 'Common')}">${coasterEmblem(relic?.kind ?? 'HouseRule')}</span>`;
  if ('TakeRule' in c) return { title: name, hint: contentText('relics', id), pic, cls: 'is-rule' };
  return { title: t(de.encounter.swapRelic, { name }), hint: de.encounter.swapHint, pic, cls: 'is-swap' };
}

function optionCard(o: EncounterOptionView, i: number): string {
  const p = optionParts(o);
  const tag = p.cls === 'is-rule' ? `<span class="enc-tag">${esc(de.encounter.takeRule)}</span>` : '';
  return `<button class="enc-option ${p.cls}" style="--i:${i}" ${attrs({
    'data-action': 'choose-encounter',
    'data-option': o.index,
    'data-target': o.needs_target ? 'true' : 'false',
    disabled: !o.can_choose,
  })}>
    <span class="enc-pic">${p.pic}</span>
    <span class="enc-body"><b class="enc-title">${esc(p.title)}</b>${tag}<span class="enc-hint">${esc(p.hint)}</span></span>
    ${o.choice === 'Leave' ? '' : costTag(o)}
  </button>`;
}

export function renderEncounter(view: RunView): string {
  const e = view.encounter;
  if (!e) return '';
  const faces = encounterFaces(e.id)
    .map((f) => portrait(f, 'portrait-enc'))
    .join('');
  const options = e.options.map(optionCard).join('');
  return `<main class="encounter" data-testid="encounter" ${attrs({ 'data-encounter': e.id })}>
    <div class="enc-wall">
      <div class="enc-sign"><span class="enc-kicker">${esc(de.encounter.kicker)}</span><span class="enc-sub">${esc(de.encounter.sign)}</span></div>
      <div class="enc-faces">${faces}</div>
    </div>
    <section class="enc-story">
      <h2>${esc(contentName('encounters', e.id) || e.name)}</h2>
      <p>${esc(contentText('encounters', e.id) || e.text)}</p>
    </section>
    <div class="enc-options">${options}</div>
  </main>`;
}

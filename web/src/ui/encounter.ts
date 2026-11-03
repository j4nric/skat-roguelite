// An encounter (Ereignis) between two series, in the style of the pub: a
// chalk sign, the people who speak, the story from the content and the
// options as cards on the table. What each option does, what it costs and
// whether it can be chosen all come from the core (EncounterView), also
// which modification a card gets or which relic a swap brings, when the
// core drew them in advance.

import { de, contentName, contentText, isCurse, relicLook, t } from '../i18n';
import type { EncounterOptionView, RunView } from '../types';
import { modIcon } from './cards';
import { attrs, esc } from './dom';
import { coinIcon, heart } from './hud';
import { consumableCoaster, relicCoaster } from './emblems';
import { encounterFaces, portrait } from './people';

/** A beer glass, for "Letzte Runde". */
function glass(): string {
  return `<svg class="enc-glass" viewBox="0 0 40 52" width="34" height="44" aria-hidden="true"><path d="M7 10 L33 10 L30 48 L10 48 Z" fill="#e8a417" stroke="#2b1a0e" stroke-width="2"/><path d="M5 10 C5 3 12 2 15 5 C18 1 25 1 27 5 C31 2 36 5 35 10 Z" fill="#fffaf0" stroke="#2b1a0e" stroke-width="2"/><path d="M33 18 C40 18 40 34 32 34" fill="none" stroke="#2b1a0e" stroke-width="2.4"/><path d="M14 18 L13 42 M20 18 L20 42" stroke="#fff" stroke-width="1.6" opacity=".45"/></svg>`;
}

/** A marked card, for the Kartengeber. */
function markedCard(chosen: boolean): string {
  return `<svg class="enc-card" viewBox="0 0 40 52" width="34" height="44" aria-hidden="true"><rect x="5" y="3" width="30" height="45" rx="4" fill="#fbf3e2" stroke="#2b1a0e" stroke-width="2"/><path d="M12 10 L28 10 M12 15 L22 15" stroke="#c9b78f" stroke-width="2"/><circle cx="20" cy="30" r="8" fill="none" stroke="#5e3796" stroke-width="2.4"/><path d="M20 24 L22 29 L27 29 L23 32 L24.5 37 L20 34 L15.5 37 L17 32 L13 29 L18 29 Z" fill="#5e3796"/>${chosen ? '<path d="M29 40 L37 48" stroke="#2b1a0e" stroke-width="3" stroke-linecap="round"/>' : '<text x="31" y="47" font-size="16" fill="#2b1a0e" text-anchor="middle">?</text>'}</svg>`;
}

/** A door, for walking on. */
function door(): string {
  return `<svg class="enc-door" viewBox="0 0 40 52" width="34" height="44" aria-hidden="true"><rect x="7" y="3" width="26" height="46" rx="2" fill="#6b4427" stroke="#2b1a0e" stroke-width="2"/><rect x="11" y="8" width="18" height="16" fill="#e8c56a" opacity=".7"/><circle cx="27" cy="31" r="2" fill="#e2b04a"/></svg>`;
}

function costTag(o: EncounterOptionView): string {
  if (!o.cost) return `<span class="enc-cost is-free">${esc(de.encounter.free)}</span>`;
  // A Zinke may cost less on some cards ("Bruderherz": half on Rot).
  const lowest = Math.min(o.cost, ...(o.target_costs ?? []));
  const text = lowest < o.cost ? t(de.encounter.costFrom, { cost: o.cost, low: lowest }) : t(de.encounter.cost, { cost: o.cost });
  return `<span class="enc-cost">${coinIcon(14)}${esc(text)}</span>`;
}

/** The modification an option puts on a card: badge, name, curse mark and text. */
function modLine(o: EncounterOptionView): string {
  const mod = o.card_mod;
  if (!mod) return o.random_gain ? `<span class="enc-mod is-random">${esc(de.encounter.randomMod)}</span>` : '';
  const curse = mod.curse || isCurse(mod.id);
  return `<span class="enc-mod ${curse ? 'is-curse' : ''}" ${attrs({ 'data-card-mod': mod.id })}>${modIcon(mod.id, 22)}<span><b>${esc(t(de.encounter.cardMod, { mod: contentName('card_mods', mod.id) || mod.name }))}</b>${curse ? `<span class="curse-tag">${esc(de.card.curse)}</span>` : ''}<span class="enc-mod-text">${esc(mod.text || contentText('card_mods', mod.id))}</span></span></span>`;
}

/** The curse that comes with an option (Alte Karten, Portemonnaie). */
function curseLine(o: EncounterOptionView): string {
  const mod = o.curse;
  if (!mod) return '';
  return `<span class="enc-mod is-curse" ${attrs({ 'data-card-mod': mod.id })}>${modIcon(mod.id, 22)}<span><b>${esc(t(de.encounter.curseMod, { mod: contentName('card_mods', mod.id) || mod.name }))}</b><span class="curse-tag">${esc(de.card.curse)}</span><span class="enc-mod-text">${esc(mod.text || contentText('card_mods', mod.id))}</span></span></span>`;
}

/** Title, hint and picture of one option. */
function optionParts(o: EncounterOptionView): { title: string; hint: string; pic: string; cls: string; extra: string } {
  const c = o.choice;
  if (c === 'Leave') return { title: de.encounter.leave, hint: de.encounter.leaveHint, pic: door(), cls: 'is-leave', extra: '' };
  if (c === 'MarkRandomCard') return { title: de.encounter.markRandom, hint: de.encounter.markRandomHint, pic: markedCard(false), cls: '', extra: modLine(o) };
  if (c === 'MarkChosenCard') return { title: de.encounter.markChosen, hint: de.encounter.markChosenHint, pic: markedCard(true), cls: '', extra: modLine(o) };
  if (c === 'CleanseChosenCard') return { title: de.encounter.cleanse, hint: de.encounter.cleanseHint, pic: markedCard(true), cls: '', extra: '' };
  if (c === 'Gamble') return { title: de.encounter.gamble, hint: de.encounter.gambleHint, pic: markedCard(true), cls: '', extra: modLine(o) };
  if ('DoubleNextGame' in c) {
    return { title: de.encounter.double, hint: t(de.encounter.doubleHint, { factor: c.DoubleNextGame }), pic: glass(), cls: '', extra: '' };
  }
  if ('Heal' in c) {
    return { title: de.encounter.heal, hint: t(de.encounter.healHint, { n: c.Heal, max: maxLives }), pic: heart(true, 34), cls: 'is-heal', extra: '' };
  }
  if ('TakeCoins' in c) {
    const hint = encounterId === 'landlord_treat' ? de.encounter.treatCoinsHint : de.encounter.takeCoinsHint;
    return { title: t(de.encounter.takeCoins, { coins: c.TakeCoins }), hint, pic: coinIcon(34), cls: '', extra: '' };
  }
  if ('TakeConsumable' in c) {
    const id = c.TakeConsumable;
    const item = o.consumable ?? { id, category: 'Schnapps' as const };
    return {
      title: t(de.encounter.takeConsumable, { name: contentName('consumables', id) }),
      hint: o.can_choose ? contentText('consumables', id) : de.encounter.takeConsumableFull,
      pic: `<span class="enc-coaster">${consumableCoaster(item, { size: 44 })}</span>`,
      cls: 'is-rule',
      extra: '',
    };
  }
  if ('KeepWallet' in c) {
    return { title: t(de.encounter.keepWallet, { coins: c.KeepWallet.coins }), hint: de.encounter.keepWalletHint, pic: coinIcon(34), cls: '', extra: curseLine(o) };
  }
  if ('Lesson' in c) {
    return { title: de.encounter.lesson, hint: t(de.encounter.lessonHint, { n: c.Lesson.redeals }), pic: markedCard(true), cls: '', extra: modLine(o) };
  }
  if ('Wager' in c) {
    return { title: t(de.encounter.wager, { stake: o.cost }), hint: t(de.encounter.wagerHint, { payout: o.cost * c.Wager.factor }), pic: coinIcon(34), cls: '', extra: '' };
  }
  if ('OldCards' in c) {
    return { title: de.encounter.oldCards, hint: de.encounter.oldCardsHint, pic: markedCard(true), cls: '', extra: modLine(o) + curseLine(o) };
  }
  if ('Scout' in c) return { title: de.encounter.scout, hint: t(de.encounter.scoutHint, { n: c.Scout }), pic: door(), cls: '', extra: '' };
  if ('NextPubRedeals' in c) {
    return { title: de.encounter.nextPubRedeals, hint: t(de.encounter.nextPubRedealsHint, { n: c.NextPubRedeals }), pic: glass(), cls: '', extra: '' };
  }
  const relic = o.relic;
  const id = relic?.id ?? ('TakeRule' in c ? c.TakeRule : 'TakeRelic' in c ? c.TakeRelic : c.SwapRelic.relic);
  const name = contentName('relics', id);
  const shown = relic ?? relicLook(id);
  const pic = `<span class="enc-coaster">${relicCoaster(shown, { size: 44 })}</span>`;
  if ('TakeRule' in c) return { title: name, hint: contentText('relics', id), pic, cls: 'is-rule', extra: '' };
  if ('TakeRelic' in c) {
    return { title: t(de.encounter.takeRelic, { name }), hint: o.can_choose ? contentText('relics', id) : de.encounter.takeRelicFull, pic, cls: 'is-rule', extra: '' };
  }
  const fresh = o.new_relic;
  if (fresh) {
    const newName = contentName('relics', fresh.id) || fresh.name;
    return {
      title: t(de.encounter.swapFor, { old: name, new: newName }),
      hint: t(de.encounter.swapForHint, { old: name, new: newName, text: contentText('relics', fresh.id) || fresh.text }),
      pic,
      cls: 'is-swap',
      extra: '',
    };
  }
  return { title: t(de.encounter.swapRelic, { name }), hint: de.encounter.swapHint, pic, cls: 'is-swap', extra: '' };
}

/** Most lives a player can hold, for the Rast's hint (set per render). */
let maxLives = 5;
/** The open encounter's id (set per render). */
let encounterId = '';

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
    <span class="enc-body"><b class="enc-title">${esc(p.title)}</b>${tag}<span class="enc-hint">${esc(p.hint)}</span>${p.extra}</span>
    ${o.choice === 'Leave' ? '' : costTag(o)}
  </button>`;
}

export function renderEncounter(view: RunView): string {
  const e = view.encounter;
  if (!e) return '';
  const faces = encounterFaces(e.id)
    .map((f) => portrait(f, 'portrait-enc'))
    .join('');
  maxLives = view.tour.max_lives;
  encounterId = e.id;
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

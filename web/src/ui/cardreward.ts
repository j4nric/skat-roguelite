// The card reward after a won normal pub or Spelunke (DESIGN 5.0, decided
// by the user on 04.11.2026; fixed pairs since 05.11.2026): the innkeeper
// offers three cards of the player's deck, each with a Zinke, drawn like
// an offer of the Auslage (the card as it will be, the Zinke, what it does
// to the card). Tapping one opens the before/after sheet, "Anbringen"
// takes it for free; or skip for Groschen. What is on offer and what a
// choice does come from the core (view.card_reward, TakeCardReward,
// SkipCardReward); this only draws it.

import { de, cardName, contentName, modText, t } from '../i18n';
import type { CardRewardOfferView, RunView } from '../types';
import { cardSvg } from './cards';
import { attrs, esc } from './dom';
import { coinIcon } from './hud';
import { portrait } from './people';
import { zinkeName } from './shop';

/** The line under the Zinke: on which card, and what happens to its Zinke. */
function pairNote(o: CardRewardOfferView): { on: string; note: string; kept: boolean } {
  const special = o.card_mod.kind === 'Special';
  const kept = special ? o.card.mods.find((m) => m !== o.card_mod.id) : undefined;
  const note = o.replaces
    ? t(de.shop.zinkeReplaces, { old: contentName('card_mods', o.replaces) })
    : o.raises
      ? t(de.shop.zinkeRaises, { mod: contentName('card_mods', o.card_mod.id) })
      : kept
        ? t(de.shop.zinkeKeeps, { mod: contentName('card_mods', kept) })
        : '';
  return { on: t(de.shop.zinkeOn, { card: cardName(o.card.kind) }), note, kept: !!kept && !o.replaces && !o.raises };
}

/** One offer: the card with its Zinke, name, effect, and "Diese". */
function rewardOffer(o: CardRewardOfferView, i: number): string {
  const mod = o.card_mod;
  const special = mod.kind === 'Special';
  const name = special ? t(de.shop.zinkeBecomes, { special: contentName('card_mods', mod.id) || mod.name }) : zinkeName(mod);
  // The text with its value filled in ("+3 Augen").
  const text = modText(mod.id, mod.value ?? 1) || mod.text;
  const rare = mod.rarity === 'Rare' || special;
  const { on, note, kept } = pairNote(o);
  return `<button class="painter-option reward-option ${rare ? 'is-rare' : ''}" style="--i:${i}" ${attrs({
    'data-action': 'card-reward-pick',
    'data-offer': o.index,
    'data-card': o.card.id,
    'data-testid': `card-reward-${mod.id}`,
    disabled: !o.can_take,
  })}>
    ${rare ? `<span class="zinke-rare">${esc(de.cardReward.rare)}</span>` : ''}
    <span class="card">${cardSvg(o.result.kind, o.result.mods, 'full', o.result.mod_values)}</span>
    <span class="reward-body">
      <b class="zinke-name">${esc(name)}</b>
      <span class="zinke-on">${esc(on)}${note ? ` · <span class="zinke-note${kept ? ' is-kept' : ''}">${esc(note)}</span>` : ''}</span>
      <span class="zinke-text">${esc(text)}</span>
      <span class="btn btn-small btn-buy">${esc(de.cardReward.take)}</span>
    </span>
  </button>`;
}

/** The reward screen: the innkeeper, the three cards with their Zinke and "Überspringen". */
export function renderCardReward(view: RunView): string {
  const reward = view.card_reward;
  if (!reward) return '';
  const offers = reward.offers.map(rewardOffer).join('');
  return `<main class="encounter card-reward" data-testid="card-reward">
    <div class="enc-wall">
      <div class="enc-sign"><span class="enc-kicker">${esc(de.cardReward.kicker)}</span><span class="enc-sub">${esc(de.cardReward.sub)}</span></div>
      <div class="enc-faces">${portrait('p-wirt', 'portrait-enc')}</div>
    </div>
    <section class="enc-story">
      <h2>${esc(de.cardReward.title)}</h2>
      <p>${esc(de.cardReward.text)}</p>
    </section>
    <div class="painter-grid reward-grid" data-testid="card-reward-offers">${offers}</div>
    <button class="btn btn-wide" ${attrs({ 'data-action': 'card-reward-skip', 'data-testid': 'card-reward-skip' })}>${coinIcon(16)}<span>${esc(t(de.cardReward.skip, { coins: reward.skip_coins }))}</span></button>
  </main>`;
}

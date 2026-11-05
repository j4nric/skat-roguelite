// The card reward after a won normal pub or Spelunke (DESIGN 5.0, decided
// by the user on 04.11.2026): the innkeeper offers three Zinken, each on a
// sample card with its name and text (like the Marke pick), the player
// takes one for free and then picks the card, or skips for Groschen. What
// is on offer, which cards each may go on and what a choice does come from
// the core (view.card_reward, TakeCardReward, SkipCardReward); this only
// draws it.

import { de, contentName, modText, t } from '../i18n';
import type { RunView } from '../types';
import { attrs, esc } from './dom';
import { glossarySample } from './glossary';
import { coinIcon } from './hud';
import { portrait } from './people';

/** The reward screen: the innkeeper, the three Zinken and "Überspringen". */
export function renderCardReward(view: RunView): string {
  const reward = view.card_reward;
  if (!reward) return '';
  const offers = reward.offers
    .map((o, i) => {
      const id = o.card_mod.id;
      const name = contentName('card_mods', id) || o.card_mod.name;
      // The text with its value filled in ("+3 Augen").
      const text = modText(id, o.card_mod.value ?? 1) || o.card_mod.text;
      const can = o.targets.length > 0;
      const rare = o.card_mod.rarity === 'Rare';
      return `<button class="painter-option reward-option ${rare ? 'is-rare' : ''}" style="--i:${i}" ${attrs({
        'data-action': 'card-reward-pick',
        'data-offer': o.index,
        'data-testid': `card-reward-${id}`,
        disabled: !can,
      })}>
        ${rare ? `<span class="zinke-rare">${esc(de.cardReward.rare)}</span>` : ''}
        <span class="card">${glossarySample(id, o.card_mod.value ?? undefined)}</span>
        <span class="reward-body">
          <b class="zinke-name">${esc(name)}</b>
          <span class="zinke-text">${esc(text)}</span>
          <span class="btn btn-small btn-buy">${esc(de.cardReward.take)}</span>
        </span>
      </button>`;
    })
    .join('');
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

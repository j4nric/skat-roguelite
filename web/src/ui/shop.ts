// The Tresen: offers with prices, owned relics and items, Neuauslage.
// What can be bought, sold or used is decided by the core (can_buy etc.).

import { de, contentName, contentText, t } from '../i18n';
import type { RunView } from '../types';
import { attrs, esc } from './dom';

function tag(kind: string, rarity: string): string {
  return `<span class="tag">${esc(kind)} · ${esc(rarity)}</span>`;
}

export function renderShop(view: RunView): string {
  const shop = view.shop;
  if (!shop) return '';
  const relicOffers = shop.relics
    .map((o) => {
      if (!o) return `<article class="offer is-sold"><span>${esc(de.shop.soldOut)}</span></article>`;
      return `<article class="offer">
        <div class="offer-head"><b>${esc(contentName('relics', o.relic.id))}</b>${tag(de.relicKind[o.relic.kind], de.rarity[o.relic.rarity])}</div>
        <p>${esc(contentText('relics', o.relic.id))}</p>
        <button class="btn btn-buy" ${attrs({ 'data-action': 'buy-relic', 'data-slot': o.slot, disabled: !o.can_buy })}>${esc(t(de.shop.buy, { price: o.relic.price }))}</button>
      </article>`;
    })
    .join('');
  const itemOffers = shop.consumables
    .map((o) => {
      if (!o) return `<article class="offer is-sold"><span>${esc(de.shop.soldOut)}</span></article>`;
      return `<article class="offer">
        <div class="offer-head"><b>${esc(contentName('consumables', o.consumable.id))}</b>${tag(de.shop.consumableKind, de.rarity[o.consumable.rarity])}</div>
        <p>${esc(contentText('consumables', o.consumable.id))}</p>
        <button class="btn btn-buy" ${attrs({ 'data-action': 'buy-item', 'data-slot': o.slot, disabled: !o.can_buy })}>${esc(t(de.shop.buy, { price: o.consumable.price }))}</button>
      </article>`;
    })
    .join('');
  const owned = view.relics
    .map(
      (r) => `<li class="owned">
        <div><b>${esc(contentName('relics', r.relic.id))}</b><p>${esc(contentText('relics', r.relic.id))}</p></div>
        <button class="btn btn-small" ${attrs({ 'data-action': 'sell-relic', 'data-index': r.index, disabled: !r.can_sell })}>${esc(t(de.shop.sell, { price: r.sell_price }))}</button>
      </li>`,
    )
    .join('');
  const items = view.consumables
    .map(
      (c) => `<li class="owned">
        <div><b>${esc(contentName('consumables', c.consumable.id))}</b><p>${esc(contentText('consumables', c.consumable.id))}</p></div>
        <div class="owned-actions">
          <button class="btn btn-small" ${attrs({ 'data-action': 'use-item', 'data-slot': c.slot, disabled: !c.can_use })}>${esc(de.shop.use)}</button>
          <button class="btn btn-small" ${attrs({ 'data-action': 'sell-item', 'data-slot': c.slot, disabled: !c.can_sell })}>${esc(t(de.shop.sell, { price: c.sell_price }))}</button>
        </div>
      </li>`,
    )
    .join('');
  return `<main class="shop" data-testid="shop">
    <h2>${esc(de.shop.title)}</h2>
    <p class="small">${esc(de.shop.subtitle)}</p>
    <section><h3>${esc(de.shop.relics)}</h3><div class="offers">${relicOffers}</div></section>
    <section><h3>${esc(de.shop.consumables)}</h3><div class="offers">${itemOffers}</div></section>
    <section><h3>${esc(t(de.shop.owned, { count: view.relics.length, slots: view.relic_slots }))}</h3>
      ${owned ? `<ul class="owned-list">${owned}</ul>` : `<p class="small">${esc(de.shop.empty)}</p>`}</section>
    <section><h3>${esc(t(de.shop.ownedItems, { count: view.consumables.length, slots: view.consumable_slots }))}</h3>
      ${items ? `<ul class="owned-list">${items}</ul>` : `<p class="small">${esc(de.shop.empty)}</p>`}</section>
    <div class="shop-bar">
      <button class="btn" ${attrs({ 'data-action': 'reroll', disabled: !shop.can_reroll })}>${esc(t(de.shop.reroll, { cost: shop.reroll_cost }))}</button>
      <button class="btn btn-primary" ${attrs({ 'data-action': 'leave-shop' })}>${esc(de.shop.leave)}</button>
    </div>
  </main>`;
}

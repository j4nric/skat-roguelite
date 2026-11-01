// The Tresen: the Wirt behind the counter, relics as Bierdeckel, items as
// things on the counter, owned relics and items, Neuauslage. What can be
// bought, sold or used is decided by the core (can_buy, can_sell, can_use).

import { de, contentName, contentText, t } from '../i18n';
import type { OwnedRelic, RelicInfo, RunView } from '../types';
import { attrs, esc } from './dom';
import { goodSvg, shelfSvg } from './goods';
import { coinIcon } from './hud';
import { oddsLine } from './odds';
import { bossPortraitId, portrait } from './people';

function price(n: number): string {
  return `<span class="price">${coinIcon(16)}${esc(t(de.shop.price, { price: n }))}</span>`;
}

/** The printed round mark of a Bierdeckel: a paragraph for house rules, a figure for regulars. */
export function coasterEmblem(kind: RelicInfo['kind']): string {
  const glyph =
    kind === 'HouseRule'
      ? '<text x="20" y="27.5" text-anchor="middle" font-size="22" font-weight="700" fill="currentColor" class="idx">§</text>'
      : '<circle cx="20" cy="15" r="5.5" fill="currentColor"/><path d="M10 31 C10 23 14 21 20 21 C26 21 30 23 30 31 Z" fill="currentColor"/>';
  return `<svg class="coaster-emblem" viewBox="0 0 40 40" aria-hidden="true"><circle cx="20" cy="20" r="18" fill="none" stroke="currentColor" stroke-width="2.5"/><circle cx="20" cy="20" r="14" fill="none" stroke="currentColor" stroke-width="1" stroke-dasharray="2 2"/>${glyph}</svg>`;
}

/** A relic as a Bierdeckel; `footer` holds price and buttons. */
export function coaster(relic: RelicInfo, footer: string, extra = ''): string {
  return `<article class="coaster rarity-${esc(relic.rarity)} ${extra}">
    ${coasterEmblem(relic.kind)}
    <div class="coaster-kind">${esc(de.relicKind[relic.kind])} · ${esc(de.rarity[relic.rarity])}</div>
    <h4 class="coaster-name">${esc(contentName('relics', relic.id))}</h4>
    <p class="coaster-text">${esc(contentText('relics', relic.id))}</p>
    ${footer ? `<div class="coaster-foot">${footer}</div>` : ''}
  </article>`;
}

function ownedCoaster(r: OwnedRelic): string {
  return coaster(
    r.relic,
    `<button class="btn btn-small" ${attrs({ 'data-action': 'sell-relic', 'data-index': r.index, disabled: !r.can_sell })}>${esc(t(de.shop.sell, { price: r.sell_price }))}</button>`,
    'coaster-owned',
  );
}

/** The chalk board on the back wall: the next series and the boss ahead. */
function nextBoard(view: RunView): string {
  const next = view.next_series;
  if (!next) return `<div class="next-board chalkboard"><p class="chalk-small">${esc(de.shop.lastSeries)}</p></div>`;
  const where = t(next.is_boss ? de.shop.nextBoss : de.shop.nextSeries, { section: next.section + 1, series: next.index + 1 });
  const boss = next.boss ?? view.upcoming_boss;
  const bossBlock = boss
    ? `<div class="next-boss">${portrait(bossPortraitId(boss.id), 'portrait-sm')}<div>
        <div class="chalk-small">${esc(next.boss ? de.boss.title : de.shop.bossTonight)}</div>
        <b class="next-boss-name">${esc(contentName('bosses', boss.id))}</b>
        <p class="next-boss-text">${esc(contentText('bosses', boss.id) || boss.text)}</p></div></div>`
    : '';
  return `<div class="next-board chalkboard" data-testid="next-series">
    <div class="next-head"><span class="chalk-small">${esc(de.shop.next)}</span><b>${esc(where)}</b></div>
    <div class="next-facts"><span class="next-target">${esc(t(de.shop.nextTarget, { target: next.target }))}</span><span>${esc(t(de.shop.nextGames, { games: next.game_limit, passes: next.passes }))}</span></div>
    ${bossBlock}
  </div>`;
}

export function renderShop(view: RunView): string {
  const shop = view.shop;
  if (!shop) return '';
  const relicOffers = shop.relics
    .map((o, i) => {
      if (!o) return `<div class="coaster-ring is-sold" style="--i:${i}"><span>${esc(de.shop.soldOut)}</span></div>`;
      return `<div class="coaster-wrap" style="--i:${i}">${coaster(
        o.relic,
        `${price(o.relic.price)}<button class="btn btn-buy btn-small" ${attrs({ 'data-action': 'buy-relic', 'data-slot': o.slot, disabled: !o.can_buy })}>${esc(de.shop.buyShort)}</button>`,
      )}</div>`;
    })
    .join('');
  const itemOffers = shop.consumables
    .map((o) => {
      if (!o) return `<div class="good is-sold"><span>${esc(de.shop.soldOut)}</span></div>`;
      return `<article class="good">
        <div class="good-pic">${goodSvg(o.consumable.id, 72)}</div>
        <div class="good-body">
          <h4>${esc(contentName('consumables', o.consumable.id))}</h4>
          <p>${esc(contentText('consumables', o.consumable.id))}</p>
          ${o.consumable.needs_target ? oddsLine(view.deal_odds) : ''}
          <div class="good-foot">${price(o.consumable.price)}<button class="btn btn-buy btn-small" ${attrs({ 'data-action': 'buy-item', 'data-slot': o.slot, disabled: !o.can_buy })}>${esc(de.shop.buyShort)}</button></div>
        </div>
      </article>`;
    })
    .join('');
  const owned = view.relics.map(ownedCoaster).join('');
  const items = view.consumables
    .map(
      (c) => `<article class="good good-owned">
        <div class="good-pic">${goodSvg(c.consumable.id, 52)}</div>
        <div class="good-body">
          <h4>${esc(contentName('consumables', c.consumable.id))}</h4>
          <p>${esc(contentText('consumables', c.consumable.id))}</p>
          <div class="good-foot">
            <button class="btn btn-small" ${attrs({ 'data-action': 'use-item', 'data-slot': c.slot, disabled: !c.can_use })}>${esc(de.shop.use)}</button>
            <button class="btn btn-small" ${attrs({ 'data-action': 'sell-item', 'data-slot': c.slot, disabled: !c.can_sell })}>${esc(t(de.shop.sell, { price: c.sell_price }))}</button>
          </div>
        </div>
      </article>`,
    )
    .join('');
  const anyAffordable = shop.relics.some((o) => o?.can_buy) || shop.consumables.some((o) => o?.can_buy);
  return `<main class="shop" data-testid="shop">
    <div class="backbar">
      ${shelfSvg()}
      <div class="wirt">${portrait('p-wirt', 'portrait-wirt')}</div>
      <div class="bubble"><h2>${esc(de.shop.title)}</h2><p>${esc(anyAffordable ? de.shop.wirtSays : de.shop.wirtBroke)}</p></div>
    </div>
    ${nextBoard(view)}
    <div class="counter">
      <h3>${esc(de.shop.relics)}</h3>
      <div class="coasters">${relicOffers}</div>
      <h3>${esc(de.shop.consumables)}</h3>
      <div class="goods">${itemOffers}</div>
    </div>
    <section class="shop-owned">
      <h3>${esc(t(de.shop.owned, { count: view.relics.length, slots: view.relic_slots }))}</h3>
      ${owned ? `<div class="coasters coasters-owned">${owned}</div>` : `<p class="small">${esc(de.shop.noRelics)}</p>`}
      <h3>${esc(t(de.shop.ownedItems, { count: view.consumables.length, slots: view.consumable_slots }))}</h3>
      ${items ? `<div class="goods">${items}</div>` : `<p class="small">${esc(de.shop.noItems)}</p>`}
    </section>
    <div class="shop-bar">
      <button class="btn" ${attrs({ 'data-action': 'reroll', disabled: !shop.can_reroll })}>${esc(t(de.shop.reroll, { cost: shop.reroll_cost }))}</button>
      <button class="btn btn-primary" ${attrs({ 'data-action': 'leave-shop' })}>${esc(de.shop.leave)}</button>
    </div>
  </main>`;
}

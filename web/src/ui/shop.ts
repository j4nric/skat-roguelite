// The Tresen: the Wirt behind the counter, relics as little paper cards,
// items as things on the counter, owned relics and items, Neuauslage. What can be
// bought, sold or used is decided by the core (can_buy, can_sell, can_use).

import { de, contentName, contentText, t } from '../i18n';
import type { ConsumableCategory, OwnedRelic, RelicInfo, RunView } from '../types';
import { pixels } from './art';
import { attrs, esc } from './dom';
import { goodSvg } from './goods';
import { coinIcon } from './hud';
import { bossPortraitId, portrait } from './people';

function price(n: number): string {
  return `<span class="price">${coinIcon(16)}${esc(t(de.shop.price, { price: n }))}</span>`;
}

/** A paragraph sign, for house rules (8 × 8 pixels). */
const PARAGRAPH = ['..####..', '.#......', '..###...', '.#...#..', '..###...', '.....#..', '.####...', '........'];
/** A figure, for regulars (8 × 8 pixels). */
const FIGURE = ['...##...', '..####..', '...##...', '.######.', '#.####.#', '..####..', '..#..#..', '..#..#..'];

/** The printed mark of a relic card: a paragraph for house rules, a figure for regulars. */
export function coasterEmblem(kind: RelicInfo['kind']): string {
  const glyph = pixels(kind === 'HouseRule' ? PARAGRAPH : FIGURE, 2, 2, 1, 'currentColor');
  return `<svg class="coaster-emblem" viewBox="0 0 12 12" shape-rendering="crispEdges" aria-hidden="true"><path d="M0 0h12v1h-12zM0 11h12v1h-12zM0 1h1v10h-1zM11 1h1v10h-1z" fill="currentColor"/>${glyph}</svg>`;
}

/**
 * A relic as a little paper card: emblem, name, the start of the text and
 * `footer` (price and buttons). The whole coaster is a button that opens
 * the full text (`more` = data attributes of that button).
 */
export function coaster(
  relic: RelicInfo,
  footer: string,
  more: Record<string, string | number>,
  opts: { price?: number; extra?: string } = {},
): string {
  const name = contentName('relics', relic.id);
  const label = t(de.shop.moreAbout, { name });
  const kind = de.relicKind[relic.kind];
  return `<article class="coaster rarity-${esc(relic.rarity)} ${opts.extra ?? ''}">
    <button class="coaster-cover" ${attrs({ ...more, 'aria-label': label, title: label })}></button>
    ${opts.price === undefined ? '' : `<span class="price-tag">${price(opts.price)}</span>`}
    <span class="coaster-emblem-wrap" title="${esc(kind)}">${coasterEmblem(relic.kind)}<span class="sr-only">${esc(kind)}</span></span>
    <h4 class="coaster-name">${esc(name)}</h4>
    <p class="coaster-text">${esc(contentText('relics', relic.id))}</p>
    ${footer ? `<div class="coaster-foot">${footer}</div>` : ''}
  </article>`;
}

function ownedCoaster(r: OwnedRelic, i: number): string {
  return `<div class="coaster-wrap" style="--i:${i}">${coaster(
    r.relic,
    `<button class="btn btn-small" ${attrs({ 'data-action': 'sell-relic', 'data-index': r.index, disabled: !r.can_sell })}>${esc(t(de.shop.sell, { price: r.sell_price }))}</button>`,
    { 'data-action': 'info-relic', 'data-index': r.index, 'data-info': `relic:${r.relic.id}` },
    { extra: 'coaster-owned' },
  )}</div>`;
}

/** An item as a thing on the counter; tap the picture or name for the full text. */
function good(id: string, more: Record<string, string | number>, footer: string, extra = '', category?: ConsumableCategory): string {
  const name = contentName('consumables', id);
  const label = t(de.shop.moreAbout, { name });
  const schnaps = category === 'Schnapps';
  return `<article class="good ${extra} ${schnaps ? 'is-schnaps' : ''}">
    <button class="good-cover" ${attrs({ ...more, 'aria-label': label, title: label })}></button>
    <div class="good-pic">${goodSvg(id, 56)}${schnaps ? `<span class="good-tag">${esc(de.shop.schnaps)}</span>` : ''}</div>
    <h4>${esc(name)}</h4>
    <p class="good-text">${esc(contentText('consumables', id))}</p>
    <div class="good-foot">${footer}</div>
  </article>`;
}

/** The chalk board on the back wall: the next series and the boss ahead. */
function nextBoard(view: RunView): string {
  const next = view.next_series;
  if (!next) return `<div class="next-board chalkboard"><p class="chalk-small">${esc(de.shop.lastSeries)}</p></div>`;
  const where = t(next.is_boss ? de.shop.nextBoss : de.shop.nextSeries, { section: next.section + 1, series: next.index + 1 });
  const boss = next.boss ?? view.upcoming_boss;
  const bossBlock = boss
    ? `<div class="next-boss" ${attrs({ 'data-info': `boss:${boss.id}`, tabindex: 0, role: 'button', 'aria-label': t(de.info.label, { name: contentName('bosses', boss.id) }) })}>${portrait(bossPortraitId(boss.id), 'portrait-sm')}<div>
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
      if (!o) return `<div class="coaster-wrap" style="--i:${i}"><div class="coaster-ring is-sold"><span>${esc(de.shop.soldOut)}</span></div></div>`;
      return `<div class="coaster-wrap" style="--i:${i}">${coaster(
        o.relic,
        `<button class="btn btn-buy btn-small" ${attrs({ 'data-action': 'buy-relic', 'data-slot': o.slot, disabled: !o.can_buy })}>${esc(de.shop.buyShort)}</button>`,
        { 'data-action': 'info-offer-relic', 'data-slot': o.slot, 'data-info': `offer-relic:${o.slot}` },
        { price: o.price },
      )}</div>`;
    })
    .join('');
  const itemOffers = shop.consumables
    .map((o) => {
      if (!o) return `<div class="good is-sold"><span>${esc(de.shop.soldOut)}</span></div>`;
      return good(
        o.consumable.id,
        { 'data-action': 'info-offer-item', 'data-slot': o.slot, 'data-info': `offer-item:${o.slot}` },
        `${price(o.price)}<button class="btn btn-buy btn-small" ${attrs({ 'data-action': 'buy-item', 'data-slot': o.slot, disabled: !o.can_buy })}>${esc(de.shop.buyShort)}</button>`,
        '',
        o.consumable.category,
      );
    })
    .join('');
  const owned = view.relics.map(ownedCoaster).join('');
  const items = view.consumables
    .map((c) =>
      good(
        c.consumable.id,
        { 'data-action': 'info-item', 'data-slot': c.slot, 'data-info': `item:${c.slot}` },
        `<button class="btn btn-small" ${attrs({ 'data-action': 'use-item', 'data-slot': c.slot, disabled: !c.can_use })}>${esc(de.shop.use)}</button>
         <button class="btn btn-small" ${attrs({ 'data-action': 'sell-item', 'data-slot': c.slot, disabled: !c.can_sell })}>${esc(t(de.shop.sell, { price: c.sell_price }))}</button>`,
        'good-owned',
        c.consumable.category,
      ),
    )
    .join('');
  const anyAffordable = shop.relics.some((o) => o?.can_buy) || shop.consumables.some((o) => o?.can_buy);
  return `<main class="shop" data-testid="shop">
    <div class="backbar">
      <div class="wirt">${portrait('p-wirt', 'portrait-wirt')}</div>
      <div class="bubble"><h2>${esc(de.shop.title)}</h2><p>${esc(anyAffordable ? de.shop.wirtSays : de.shop.wirtBroke)}</p></div>
    </div>
    ${nextBoard(view)}
    <div class="counter">
      <h3>${esc(de.shop.relics)}</h3>
      <div class="coasters">${relicOffers}</div>
      <h3 id="shop-goods">${esc(de.shop.consumables)}</h3>
      <div class="goods" data-goods>${itemOffers}</div>
    </div>
    <section class="shop-owned">
      <h3>${esc(t(de.shop.owned, { count: view.relics.length, slots: view.relic_slots }))}</h3>
      ${owned ? `<div class="coasters coasters-owned">${owned}</div>` : `<p class="small">${esc(de.shop.noRelics)}</p>`}
      <h3>${esc(t(de.shop.ownedItems, { count: view.consumables.length, slots: view.consumable_slots }))}</h3>
      ${items ? `<div class="goods">${items}</div>` : `<p class="small">${esc(de.shop.noItems)}</p>`}
    </section>
    <div class="shop-bar">
      <div class="more-hint" hidden aria-hidden="true" ${attrs({ 'data-action': 'scroll-goods' })}>${esc(de.shop.moreBelow)}</div>
      <button class="btn" ${attrs({ 'data-action': 'reroll', disabled: !shop.can_reroll })}>${esc(t(de.shop.reroll, { cost: shop.reroll_cost }))}</button>
      <button class="btn btn-primary" ${attrs({ 'data-action': 'leave-shop' })}>${esc(de.shop.leave)}</button>
    </div>
  </main>`;
}

let goodsWatch: IntersectionObserver | null = null;

/**
 * Show the hint "more goods below" while the goods on the counter are out
 * of sight (phones). Call after every render; it rewires itself.
 */
export function watchShop(root: HTMLElement): void {
  goodsWatch?.disconnect();
  goodsWatch = null;
  const goods = root.querySelector('[data-goods]');
  const hint = root.querySelector<HTMLElement>('.more-hint');
  const bar = root.querySelector<HTMLElement>('.shop-bar');
  if (!goods || !hint || !bar || typeof IntersectionObserver === 'undefined') return;
  goodsWatch = new IntersectionObserver(
    (entries) => {
      const entry = entries[entries.length - 1];
      if (!entry) return;
      // Hidden once the goods are in sight or already scrolled past.
      hint.hidden = entry.intersectionRatio >= 0.5 || entry.boundingClientRect.top < 0;
    },
    { rootMargin: `0px 0px -${bar.offsetHeight}px 0px`, threshold: [0, 0.5, 1] },
  );
  goodsWatch.observe(goods);
}

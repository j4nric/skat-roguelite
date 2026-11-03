// The Tresen: the Wirt behind the counter, the relics on offer as a row of
// three tiles like the Auslage (03.11.2026), items as things on the
// counter, owned relics as little paper cards and items, Neuauslage. The
// Zinken (DESIGN 8.6): the Auslage of three cards drawn with their new
// Zinke, "Neu auslosen", the Kartenmaler and "Karte reinigen". Curses as
// trade: "Hehlerware" among the relics, "Schnäppchen" in the Auslage, each
// with a badge and the price struck through. What can be bought, sold or
// used is decided by the core (can_buy, can_sell, can_use).

import { de, bossEditionText, bossFiguresText, cardName, contentName, contentText, modText, t } from '../i18n';
import type { CardModInfo, CardOfferView, ConsumableInfo, OwnedRelic, RelicInfo, RelicOffer, RunView, ServiceView, ShopView } from '../types';
import { backDesign } from './backs';
import { cardBack, cardSvg, modValueText } from './cards';
import { attrs, esc } from './dom';
import { bossCoaster, consumableCoaster, relicCoaster } from './emblems';
import { coinIcon } from './hud';
import { mapIcon } from './mapicons';
import { portrait } from './people';

function price(n: number): string {
  return `<span class="price">${coinIcon(16)}${esc(t(de.shop.price, { price: n }))}</span>`;
}

/** A price tag; a trade offer (Schnäppchen, Hehlerware) shows the price it would have without the curse struck through. */
function priceTag(n: number, was?: number | null): string {
  const old = was != null && was > n ? `<s class="price-was" aria-label="${esc(t(de.shop.tradeWas, { price: was }))}">${esc(String(was))}</s>` : '';
  return `<span class="price-tag">${old}${price(n)}</span>`;
}

/** The badge of a trade offer (curses as trade, 03.11.2026). */
function tradeBadge(text: string, testid: string): string {
  return `<span class="trade-badge" data-testid="${esc(testid)}">${esc(text)}</span>`;
}

/**
 * A relic on offer at the Tresen as a tile of the row of three, like the
 * Auslage of the Zinken: the coaster's emblem, name, text and the buy
 * button. The whole tile opens the full text. "Hehlerware" is marked.
 */
function relicTile(o: RelicOffer | null, i: number): string {
  if (!o) return `<div class="zinke relic-tile is-sold" style="--i:${i}"><span>${esc(de.shop.soldOut)}</span></div>`;
  const relic = o.relic;
  const name = contentName('relics', relic.id);
  const label = t(de.shop.moreAbout, { name });
  const kind = de.relicKind[relic.kind];
  const fence = o.fence === true;
  return `<article class="zinke relic-tile rarity-${esc(relic.rarity)}${fence ? ' is-trade' : ''}" style="--i:${i}" data-testid="relic-offer" ${attrs({ 'data-slot': o.slot, 'data-relic': relic.id, 'data-fence': fence ? 'true' : null })}>
    <button class="coaster-cover" ${attrs({ 'data-action': 'info-offer-relic', 'data-slot': o.slot, 'data-info': `offer-relic:${o.slot}`, 'aria-label': label, title: label })}></button>
    ${priceTag(o.price, o.was)}${fence ? tradeBadge(de.shop.fence, 'fence-badge') : ''}
    <span class="relic-tile-emblem" title="${esc(kind)}">${relicCoaster(relic, { size: 56 })}<span class="sr-only">${esc(kind)}</span></span>
    <h4 class="zinke-name">${esc(name)}</h4>
    ${fence ? `<p class="zinke-on"><span class="zinke-note">${esc(de.shop.fenceNote)}</span></p>` : ''}
    <p class="zinke-text relic-tile-text">${esc(contentText('relics', relic.id))}</p>
    <div class="good-foot"><button class="btn btn-buy btn-small" ${attrs({ 'data-action': 'buy-relic', 'data-slot': o.slot, disabled: !o.can_buy })}>${esc(de.shop.buyShort)}</button></div>
  </article>`;
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
    <span class="coaster-emblem-wrap" title="${esc(kind)}">${relicCoaster(relic, { size: 52 })}<span class="sr-only">${esc(kind)}</span></span>
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

/** An item as a beer mat on the counter; tap the picture or name for the full text. */
function good(item: ConsumableInfo, more: Record<string, string | number>, footer: string, extra = ''): string {
  const id = item.id;
  const name = contentName('consumables', id);
  const label = t(de.shop.moreAbout, { name });
  const schnaps = item.category === 'Schnapps';
  return `<article class="good ${extra} ${schnaps ? 'is-schnaps' : ''}">
    <button class="good-cover" ${attrs({ ...more, 'aria-label': label, title: label })}></button>
    <div class="good-pic">${consumableCoaster(item, { size: 52 })}${schnaps ? `<span class="good-tag">${esc(de.shop.schnaps)}</span>` : ''}</div>
    <h4>${esc(name)}</h4>
    <p class="good-text">${esc(contentText('consumables', id))}</p>
    <div class="good-foot">${footer}</div>
  </article>`;
}

/** "Siegel +2": the name of a Zinke with its value, as printed on the card. */
export function zinkeName(mod: CardModInfo): string {
  const name = contentName('card_mods', mod.id) || mod.name;
  return mod.value == null ? name : `${name} ${modValueText(mod.id, { [mod.id]: mod.value })}`;
}

/** One offer of the Auslage: the card as it will be, the Zinke, the price. */
function zinkenOffer(o: CardOfferView | null, i: number): string {
  if (!o) return `<div class="zinke is-sold" style="--i:${i}"><span>${esc(de.shop.soldOut)}</span></div>`;
  const mod = o.card_mod;
  const card = cardName(o.card.kind);
  // Rare: a special card right on the card ("wird untot"); its Zinke stays next to it (DESIGN 7.2, 8.6).
  const special = mod.kind === 'Special';
  const kept = special ? o.card.mods.find((m) => m !== mod.id) : undefined;
  const note = o.replaces
    ? t(de.shop.zinkeReplaces, { old: contentName('card_mods', o.replaces) })
    : o.raises
      ? t(de.shop.zinkeRaises, { mod: contentName('card_mods', mod.id) })
      : kept
        ? t(de.shop.zinkeKeeps, { mod: contentName('card_mods', kept) })
        : '';
  const name = special ? t(de.shop.zinkeBecomes, { special: contentName('card_mods', mod.id) || mod.name }) : zinkeName(mod);
  const bargain = o.bargain === true;
  return `<article class="zinke${special ? ' is-special' : ''}${bargain ? ' is-trade' : ''}" style="--i:${i}" data-testid="zinke" ${attrs({ 'data-slot': o.slot, 'data-card': o.card.id, 'data-card-mod': mod.id, 'data-special': special ? mod.id : null, 'data-bargain': bargain ? 'true' : null })}>
    ${priceTag(o.price, o.was)}${special ? `<span class="zinke-rare">${esc(de.shop.zinkeSpecial)}</span>` : ''}${bargain ? tradeBadge(de.shop.bargain, 'bargain-badge') : ''}
    <span class="card zinke-card">${cardSvg(o.result.kind, o.result.mods, 'full', o.result.mod_values)}</span>
    <h4 class="zinke-name">${esc(name)}</h4>
    <p class="zinke-on">${esc(t(de.shop.zinkeOn, { card }))}${note ? ` · <span class="zinke-note${kept && !o.replaces && !o.raises ? ' is-kept' : ''}">${esc(note)}</span>` : ''}</p>
    ${bargain ? `<p class="zinke-on"><span class="zinke-note">${esc(de.shop.bargainNote)}</span></p>` : ''}
    <p class="zinke-text">${esc(modText(mod.id, mod.value ?? 1) || mod.text)}</p>
    <div class="good-foot"><button class="btn btn-buy btn-small" ${attrs({ 'data-action': 'buy-zinke', 'data-slot': o.slot, disabled: !o.can_buy })}>${esc(de.shop.buyShort)}</button></div>
  </article>`;
}

/** Kartenmaler or Karte reinigen: a little sign with price and button. */
function service(kind: 'painter' | 'cleanse', s: ServiceView | undefined): string {
  if (!s) return '';
  const painter = kind === 'painter';
  const title = painter ? de.shop.painter : de.shop.cleanse;
  const hint = s.used ? (painter ? de.shop.painterUsed : de.shop.cleanseUsed) : !painter && !s.targets.length ? de.shop.cleanseNone : painter ? de.shop.painterHint : de.shop.cleanseHint;
  const go = t(painter ? de.shop.painterGo : de.shop.cleanseGo, { price: s.price });
  return `<article class="service ${s.used ? 'is-used' : ''}" data-testid="service-${kind}">
    <span class="service-pic" aria-hidden="true">${painter ? brush() : sponge()}</span>
    <div class="service-body"><h4>${esc(title)}</h4><p class="small">${esc(hint)}</p>
    <button class="btn btn-small" ${attrs({ 'data-action': kind, 'data-testid': `${kind}-go`, disabled: !s.can_use })}>${esc(go)}</button></div>
  </article>`;
}

/** A painter's brush. */
function brush(): string {
  return `<svg viewBox="0 0 32 32" width="32" height="32" shape-rendering="crispEdges"><path d="M22 3 L29 10 L15 22 L10 17 Z" fill="#8a5a2b" stroke="#2b1a0e" stroke-width="2"/><path d="M10 17 L15 22 L9 28 C6 30 3 29 3 26 C3 23 6 22 7 20 Z" fill="#5e3796" stroke="#2b1a0e" stroke-width="2"/></svg>`;
}

/** A sponge with a drop. */
function sponge(): string {
  return `<svg viewBox="0 0 32 32" width="32" height="32" shape-rendering="crispEdges"><rect x="4" y="12" width="24" height="14" rx="3" fill="#e8c56a" stroke="#2b1a0e" stroke-width="2"/><circle cx="10" cy="18" r="1.6" fill="#b8902e"/><circle cx="17" cy="21" r="1.6" fill="#b8902e"/><circle cx="23" cy="17" r="1.6" fill="#b8902e"/><path d="M16 2 C19 6 20 8 16 10 C12 8 13 6 16 2 Z" fill="#7fb2d9" stroke="#2b1a0e" stroke-width="1.5"/></svg>`;
}

/** The Zinken at the counter: Auslage, Neu auslosen, Kartenmaler, Karte reinigen. */
function zinkenBlock(shop: ShopView): string {
  const offers = shop.card_offers;
  if (!offers) return '';
  const row = offers.length ? offers.map(zinkenOffer).join('') : `<p class="small">${esc(de.shop.noOffers)}</p>`;
  return `<section class="zinken" data-testid="zinken">
      <h3 id="shop-zinken">${esc(de.shop.zinken)}</h3>
      <div class="zinken-head"><b>${esc(de.shop.auslage)}</b><span class="small">${esc(de.shop.auslageHint)}</span></div>
      <div class="zinken-row" data-testid="auslage">${row}</div>
      <button class="btn btn-small zinken-reroll" ${attrs({ 'data-action': 'reroll-zinken', 'data-testid': 'reroll-zinken', disabled: !shop.can_reroll_cards })}>${esc(t(de.shop.rerollCards, { cost: shop.card_reroll_cost ?? 0 }))}</button>
      <div class="services">${service('painter', shop.painter)}${service('cleanse', shop.cleanse)}</div>
    </section>`;
}

/** The chalk board on the back wall: the next series and the boss ahead. */
function nextBoard(view: RunView): string {
  const next = view.next_series;
  if (!next) return `<div class="next-board chalkboard"><p class="chalk-small">${esc(de.shop.lastSeries)}</p></div>`;
  const where = t(next.is_boss ? de.shop.nextBoss : de.shop.nextSeries, { section: next.section + 1, series: next.index + 1 });
  const boss = next.boss ?? view.upcoming_boss;
  const bossBlock = boss
    ? `<div class="next-boss" ${attrs({ 'data-info': `boss:${boss.id}`, tabindex: 0, role: 'button', 'aria-label': t(de.info.label, { name: contentName('bosses', boss.id) }) })}><span class="next-boss-marks">${bossCoaster(boss.id, { size: 48 })}<span class="next-boss-back">${cardBack('', backDesign(boss.back))}</span></span><div>
        <div class="chalk-small">${esc(next.boss ? de.boss.title : de.shop.bossTonight)}</div>
        <b class="next-boss-name">${esc(contentName('bosses', boss.id))}</b>
        <p class="next-boss-text" data-testid="next-boss-figures">${esc(bossFiguresText(boss))}</p>
        <p class="next-boss-text">${esc(contentText('bosses', boss.id) || boss.text)}</p>${bossEditionText(boss) ? `<p class="next-boss-text boss-edition">${esc(bossEditionText(boss))}</p>` : ''}</div></div>`
    : '';
  return `<div class="next-board chalkboard" data-testid="next-series">
    <div class="next-head"><span class="chalk-small">${esc(de.shop.next)}</span><b>${esc(where)}</b></div>
    <div class="next-facts"><span class="next-target">${esc(t(de.shop.nextTarget, { target: next.target }))}</span><span>${esc(t(de.shop.nextGames, { games: next.game_limit, redeals: next.redeals }))}</span></div>
    ${bossBlock}
  </div>`;
}

/**
 * The Kiosk on the way (03.11.2026): two Schnäpse, each cheaper than at
 * the Tresen; one may be bought. Prices and what can be bought come from
 * the core.
 */
function renderKiosk(view: RunView, shop: ShopView): string {
  const discount = shop.kiosk_discount ?? 0;
  const offers = shop.consumables.filter((o) => o !== null);
  const done = offers.length === 0;
  const goods = shop.consumables
    .map((o) => {
      if (!o) return '';
      const was = o.price + discount;
      return good(
        o.consumable,
        { 'data-action': 'info-offer-item', 'data-slot': o.slot, 'data-info': `offer-item:${o.slot}` },
        `${price(o.price)}${discount ? `<span class="kiosk-was">${esc(t(de.shop.kioskWas, { price: was }))}</span>` : ''}<button class="btn btn-buy btn-small" ${attrs({ 'data-action': 'buy-item', 'data-slot': o.slot, 'data-testid': 'kiosk-buy', disabled: !o.can_buy })}>${esc(de.shop.buyShort)}</button>`,
      );
    })
    .join('');
  const items = view.consumables
    .map((c) =>
      good(
        c.consumable,
        { 'data-action': 'info-item', 'data-slot': c.slot, 'data-info': `item:${c.slot}` },
        `<button class="btn btn-small" ${attrs({ 'data-action': 'use-item', 'data-slot': c.slot, disabled: !c.can_use })}>${esc(de.shop.use)}</button>
         <button class="btn btn-small" ${attrs({ 'data-action': 'sell-item', 'data-slot': c.slot, disabled: !c.can_sell })}>${esc(t(de.shop.sell, { price: c.sell_price }))}</button>`,
        'good-owned',
      ),
    )
    .join('');
  return `<main class="shop kiosk" data-testid="kiosk">
    <div class="backbar kiosk-front">
      <div class="kiosk-stand">${mapIcon('kiosk', 64)}</div>
      <div class="bubble"><h2>${esc(de.shop.kioskTitle)}</h2><p>${esc(done ? de.shop.kioskDone : t(de.shop.kioskSays, { n: discount }))}</p></div>
    </div>
    <div class="counter kiosk-counter">
      ${goods ? `<div class="goods kiosk-goods" data-goods>${goods}</div>` : ''}
    </div>
    <section class="shop-owned">
      <h3>${esc(t(de.shop.ownedItems, { count: view.consumables.length, slots: view.consumable_slots }))}</h3>
      ${items ? `<div class="goods">${items}</div>` : `<p class="small">${esc(de.shop.noItems)}</p>`}
    </section>
    <div class="shop-bar">
      <button class="btn btn-primary" ${attrs({ 'data-action': 'leave-shop', 'data-testid': 'kiosk-leave' })}>${esc(de.shop.kioskLeave)}</button>
    </div>
  </main>`;
}

export function renderShop(view: RunView): string {
  const shop = view.shop;
  if (!shop) return '';
  if (shop.kiosk) return renderKiosk(view, shop);
  const relicOffers = shop.relics.map(relicTile).join('');
  const itemOffers = shop.consumables
    .map((o) => {
      if (!o) return `<div class="good is-sold"><span>${esc(de.shop.soldOut)}</span></div>`;
      return good(
        o.consumable,
        { 'data-action': 'info-offer-item', 'data-slot': o.slot, 'data-info': `offer-item:${o.slot}` },
        `${price(o.price)}<button class="btn btn-buy btn-small" ${attrs({ 'data-action': 'buy-item', 'data-slot': o.slot, disabled: !o.can_buy })}>${esc(de.shop.buyShort)}</button>`,
      );
    })
    .join('');
  const owned = view.relics.map(ownedCoaster).join('');
  const items = view.consumables
    .map((c) =>
      good(
        c.consumable,
        { 'data-action': 'info-item', 'data-slot': c.slot, 'data-info': `item:${c.slot}` },
        `<button class="btn btn-small" ${attrs({ 'data-action': 'use-item', 'data-slot': c.slot, disabled: !c.can_use })}>${esc(de.shop.use)}</button>
         <button class="btn btn-small" ${attrs({ 'data-action': 'sell-item', 'data-slot': c.slot, disabled: !c.can_sell })}>${esc(t(de.shop.sell, { price: c.sell_price }))}</button>`,
        'good-owned',
      ),
    )
    .join('');
  const anyAffordable =
    shop.relics.some((o) => o?.can_buy) || shop.consumables.some((o) => o?.can_buy) || (shop.card_offers ?? []).some((o) => o?.can_buy) || shop.painter?.can_use === true;
  return `<main class="shop" data-testid="shop">
    <div class="backbar">
      <div class="wirt">${portrait('p-wirt', 'portrait-wirt')}</div>
      <div class="bubble"><h2>${esc(de.shop.title)}</h2><p>${esc(anyAffordable ? de.shop.wirtSays : de.shop.wirtBroke)}</p></div>
    </div>
    ${nextBoard(view)}
    ${debtLine(view)}
    <div class="counter">
      <h3>${esc(de.shop.relics)}</h3>
      <div class="zinken-row relic-row" data-testid="relic-offers">${relicOffers}</div>
      <h3 id="shop-goods">${esc(de.shop.consumables)}</h3>
      <div class="goods" data-goods>${itemOffers}</div>
      ${zinkenBlock(shop)}
    </div>
    <section class="shop-owned">
      <h3>${esc(view.relic_slots === null ? t(de.shop.owned, { count: view.relics.length }) : t(de.shop.ownedCapped, { count: view.relics.length, slots: view.relic_slots }))}</h3>
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

/** "Deckel anschreiben": debt and credit left, as the core counts them. */
function debtLine(view: RunView): string {
  const debt = view.debt ?? 0;
  const credit = view.credit_left ?? 0;
  const lines: string[] = [];
  if (debt > 0) lines.push(t(de.shop.debt, { n: debt }));
  if (credit > 0) lines.push(t(de.shop.credit, { n: credit }));
  return lines.length ? `<p class="shop-debt" data-testid="shop-debt">${esc(lines.join(' '))}</p>` : '';
}

let goodsWatch: IntersectionObserver | null = null;

/**
 * Show the hint "more goods below" while the goods on the counter are out
 * of sight (phones). Call after every render; it rewires itself.
 */
export function watchShop(root: HTMLElement): void {
  goodsWatch?.disconnect();
  goodsWatch = null;
  // The last thing on the counter: the Zinken services, else the goods.
  const goods = root.querySelector('.services') ?? root.querySelector('[data-goods]');
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

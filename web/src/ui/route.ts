// The route choice between two pubs (Kneipentour, ADR 0016): the next pub
// with its target and guests, then three ways, each with its transport,
// a decorative travel time and the category of its one stop; sometimes a
// way leads into a Spelunke (its own place, target and malus shown).
// Everything comes from the core's TourView; the structure (one card per
// way with stable node ids) is meant to sit under a map drawn later.

import { de, contentName, t } from '../i18n';
import type { PubPreview, RouteOptionView, RunView, StopCategory, Transport } from '../types';
import { attrs, esc } from './dom';
import { bossCoaster } from './emblems';
import { hearts } from './hud';

/** A small pixel icon per stop category. */
export function categoryIcon(category: StopCategory, size = 28): string {
  const svg = (body: string): string =>
    `<svg class="stop-icon stop-${category.toLowerCase()}" viewBox="0 0 8 8" width="${size}" height="${size}" shape-rendering="crispEdges" aria-hidden="true">${body}</svg>`;
  switch (category) {
    case 'Tresen':
      // A beer glass.
      return svg('<path d="M1 1h5v6h-5z" fill="#e8a417"/><path d="M1 1h5v1h-5z" fill="#fffaf0"/><path d="M6 2h1v3h-1z" fill="#6b4427"/>');
    case 'Event':
      // A speech bubble.
      return svg('<path d="M0 1h8v4h-4l-2 2v-2h-2z" fill="#f1e3c0"/><path d="M2 2h1v1h-1zM4 2h1v1h-1zM6 2h1v1h-1z" fill="#6b4427"/>');
    case 'Upgrade':
      // A brush on a card.
      return svg('<path d="M1 0h4v7h-4z" fill="#fbf3e2"/><path d="M2 2h2v2h-2z" fill="#5e3796"/><path d="M5 3h1v1h1v4h-1v-1h-1z" fill="#8a4a22"/>');
    case 'Rest':
      // A heart.
      return svg('<path d="M1 1h2v1h2v-1h2v1h1v2h-1v1h-1v1h-1v1h-1v-1h-1v-1h-1v-1h-1v-2h1z" fill="#c8312b"/>');
  }
}

/** A small pixel icon per transport. */
function transportIcon(transport: Transport): string {
  const body: Record<Transport, string> = {
    Walk: '<path d="M3 0h2v2h-2zM2 2h3v3h-3zM2 5h1v3h-1zM4 5h1v2h1v1h-2z" fill="#d8cdb8"/>',
    Train: '<path d="M1 0h6v6h-6z" fill="#2f5d8a"/><path d="M2 1h4v2h-4z" fill="#cfe3f5"/><path d="M1 6h1v2h-1zM6 6h1v2h-1z" fill="#3a2a1a"/>',
    Bike: '<path d="M0 4h3v3h-3zM5 4h3v3h-3z" fill="none" stroke="#d8cdb8" stroke-width=".8"/><path d="M2 3h4v1h-4zM4 2h1v1h-1z" fill="#a33"/>',
  };
  return `<svg class="transport-icon" viewBox="0 0 8 8" width="26" height="26" shape-rendering="crispEdges" aria-hidden="true">${body[transport]}</svg>`;
}

function guestNames(pub: PubPreview): string {
  const names = pub.guests.map((g) => g.name);
  if (pub.is_boss && pub.boss) names[0] = contentName('bosses', pub.boss.id);
  return names.join(', ');
}

function pubCard(pub: PubPreview): string {
  const boss = pub.is_boss && pub.boss ? `<span class="route-boss">${bossCoaster(pub.boss.id, { size: 34 })}${esc(t(de.route.bossAt, { name: contentName('bosses', pub.boss.id) }))}</span>` : '';
  return `<section class="route-next" data-testid="route-next" ${attrs({ 'data-node': pub.id })}>
    <span class="route-kicker">${esc(pub.is_boss ? de.route.nextBoss : de.route.next)} · ${esc(pub.district)}</span>
    <h2 class="route-pub-name">${esc(pub.name)}</h2>
    <p class="route-target" data-testid="route-target">${esc(t(de.route.target, { target: pub.target, games: pub.game_limit }))}</p>
    <p class="route-guests">${esc(t(de.route.guests, { names: guestNames(pub) }))}</p>
    ${boss}
  </section>`;
}

function spelunkeBox(sp: PubPreview, next: PubPreview): string {
  const malus = sp.malus;
  return `<span class="route-spelunke" data-testid="route-spelunke" ${attrs({ 'data-node': sp.id })}>
    <b>${esc(de.route.spelunke)}: ${esc(sp.name)}</b>
    <span>${esc(t(de.route.spelunkeTarget, { name: next.name, target: sp.target }))}</span>
    ${malus ? `<span class="route-malus" data-testid="route-malus">${esc(t(de.route.malus, { name: contentName('maluses', malus.id) || malus.name }))}: ${esc(malus.text)}</span>` : ''}
    <span class="route-reward">${esc(de.route.spelunkeReward)}</span>
  </span>`;
}

function optionCard(o: RouteOptionView, next: PubPreview, i: number): string {
  const stop = o.stops[0]?.category ?? 'Tresen';
  return `<button class="route-option ${o.spelunke ? 'is-spelunke' : ''}" style="--i:${i}" ${attrs({
    'data-action': 'choose-route',
    'data-option': o.index,
    'data-testid': 'route-option',
    'data-category': stop,
    'data-transport': o.transport,
    'data-node': o.id,
    'data-spelunke': o.spelunke ? 'true' : 'false',
    disabled: !o.can_choose,
  })}>
    <span class="route-way">${transportIcon(o.transport)}<b>${esc(de.route.transports[o.transport] ?? o.transport)}</b><small>${esc(t(de.route.minutes, { n: o.minutes }))}</small></span>
    <span class="route-stop">${categoryIcon(stop)}<span><b>${esc(de.route.categories[stop] ?? stop)}</b><small>${esc(de.route.categoryHints[stop] ?? '')}</small></span></span>
    ${o.spelunke ? spelunkeBox(o.spelunke, next) : ''}
  </button>`;
}

export function renderRoute(view: RunView): string {
  const route = view.tour.route;
  if (!route) return '';
  const options = route.options.map((o, i) => optionCard(o, route.next, i)).join('');
  return `<main class="route" data-testid="route">
    <div class="route-head">
      <h1>${esc(de.route.title)}</h1>
      <span class="route-lives">${hearts(view.tour.lives, view.tour.max_lives, 16)}</span>
    </div>
    <p class="route-sub">${esc(de.route.subtitle)}</p>
    ${pubCard(route.next)}
    <div class="route-options">${options}</div>
  </main>`;
}

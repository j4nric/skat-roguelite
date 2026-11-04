// The route choice between two pubs (Kneipentour, ADR 0016) on the city
// map (approved design "Karte Runde 3"): the current leg as a static pixel
// map with the position, the next pub (and a Spelunke as a place of its
// own) and the three ways along the streets; the chosen way stands out with
// the badge of its stop. Below it a travel bar with the three transports
// and "Los"; "−" shows the whole tour as a line plan, "+" goes back.
//
// Everything shown comes from the core's TourView: which way leads where,
// its stop, its minutes, the targets. The map itself is decoration drawn
// from the district's map seed and the node ids (./citymap.ts).

import { de, bossFiguresText, bossLine, contentName, t } from '../i18n';
import type { DistrictView, PubPreview, RouteOptionView, RunView, StopCategory, Transport } from '../types';
import { MH, MW, INK, cityImageTag, genCity, lookOf, planLeg, stamps, type City, type LegPlan, type Pattern, type Way } from './citymap';
import { attrs, esc } from './dom';
import { play } from './fx';
import { glyph, mapIcon, type MapIcon } from './mapicons';
import { bossPortraitId, guestPortraitId, portrait } from './people';

const PAPER = '#fbf6e8';

const WAY: Record<Transport, Way> = { Walk: 'fuss', Train: 'bahn', Bike: 'rad' };
const LINE: Record<Way, { colour: string; pattern: Pattern }> = {
  fuss: { colour: '#7a2232', pattern: 'solid' },
  bahn: { colour: '#1b4a4c', pattern: 'dashed' },
  rad: { colour: '#9a6418', pattern: 'dotted' },
};
const STOP_ICON: Record<StopCategory, MapIcon> = {
  Tresen: 'shop',
  Event: 'begegnung',
  Upgrade: 'upgrade',
  Rest: 'rast',
  FoundCoins: 'fundgroschen',
  Kiosk: 'kiosk',
  DiceStall: 'wuerfel',
};

/** The pixel icon of a stop category. */
export function categoryIcon(category: StopCategory, size = 28): string {
  return mapIcon(STOP_ICON[category] ?? 'begegnung', size, undefined, `stop-icon stop-${category.toLowerCase()}`);
}

/** The way picked by default: the first open one that does not turn off into a Spelunke. */
export function defaultPick(view: RunView): number {
  const options = view.tour.route?.options ?? [];
  const plain = options.find((o) => o.can_choose && !o.spelunke) ?? options.find((o) => o.can_choose) ?? options[0];
  return plain?.index ?? 0;
}

/** A key of the leg: where the player stands and where the ways lead. */
export function legKey(view: RunView): string {
  const route = view.tour.route;
  return `${view.tour.current?.id ?? 'start'}>${route?.next.id ?? ''}`;
}

// ---------------------------------------------------------------- the map

const CITIES = new Map<string, City>();

function cityFor(district: DistrictView | undefined, fallbackSeed: number): City {
  const look = lookOf(district?.look);
  const seed = String(district?.map_seed ?? fallbackSeed);
  const key = `${look}:${seed}`;
  let city = CITIES.get(key);
  if (!city) {
    city = genCity(seed, look);
    CITIES.set(key, city);
  }
  return city;
}

function placeIcon(pub: PubPreview): MapIcon {
  if (pub.malus) return 'spelunke';
  return pub.is_boss ? 'festsaal' : 'kneipe';
}

/** A framed tile standing on its place, with a small spike to the street. */
function tile(kind: MapIcon, x: number, y: number, big: boolean, frame: string, extra: string): string {
  const s = big ? 30 : 26;
  const u = big ? 1.5 : 1.25;
  const x0 = Math.round(x - s / 2);
  const y0 = y - s - 7;
  return `<g ${extra}><rect x="${x0 - 1}" y="${y0 - 1}" width="${s + 2}" height="${s + 2}" fill="${INK}"/><rect x="${x0}" y="${y0}" width="${s}" height="${s}" fill="${frame}"/><rect x="${x0 + 2}" y="${y0 + 2}" width="${s - 4}" height="${s - 4}" fill="${PAPER}"/>${glyph(kind, x0 + (s - 16 * u) / 2, y0 + (s - 16 * u) / 2, u)}<path d="M${x - 3} ${y0 + s + 1}h6v2h-1v2h-1v2h-2v-2h-1v-2h-1z" fill="${INK}"/></g>`;
}

function badge(kind: MapIcon, x: number, y: number, frame: string, category: string): string {
  const s = 24;
  const x0 = x - s / 2;
  const y0 = y - s / 2;
  return `<g class="map-badge" data-testid="map-badge" ${attrs({ 'data-category': category })}><rect x="${x0 - 1}" y="${y0 - 1}" width="${s + 2}" height="${s + 2}" fill="${INK}"/><rect x="${x0}" y="${y0}" width="${s}" height="${s}" fill="${frame}"/><rect x="${x0 + 2}" y="${y0 + 2}" width="${s - 4}" height="${s - 4}" fill="${PAPER}"/>${glyph(kind, x0 + 2, y0 + 2, 1.25)}</g>`;
}

function positionMark(x: number, y: number): string {
  const px = Math.round(x);
  const py = Math.round(y);
  return `<g class="map-pos" data-testid="map-pos"><g class="map-pin"><rect x="${px - 8}" y="${py - 8}" width="16" height="16" fill="${PAPER}" opacity=".85"/>${glyph('position', px - 8, py - 8, 1)}</g></g>`;
}

interface Leg {
  city: City;
  plan: LegPlan;
}

function legOf(view: RunView): Leg | null {
  const route = view.tour.route;
  if (!route) return null;
  const next = route.next;
  const district = view.tour.districts.find((d) => d.index === next.section);
  const city = cityFor(district, view.tour.map_seed);
  const spelunkeOption = route.options.find((o) => o.spelunke);
  const spelunkeWay = spelunkeOption ? WAY[spelunkeOption.transport] : null;
  const ahead = (district?.pubs ?? []).filter((p) => p.index > next.index && p.state === 'Ahead').length;
  const plan = planLeg(city, legKey(view), spelunkeWay, Math.min(2, ahead));
  return { city, plan };
}

/** The leg as one SVG: city, future pubs, the three ways, targets, position. */
export function mapSvg(view: RunView, pick: number): string {
  const route = view.tour.route;
  const leg = legOf(view);
  if (!route || !leg) return '';
  const { city, plan } = leg;
  let out = cityImageTag(city);
  for (const [x, y] of plan.future) out += `<g opacity=".4">${glyph('kneipe', Math.round(x) - 8, Math.round(y) - 8, 1, '#5a5650')}</g>`;
  const picked = route.options.find((o) => o.index === pick) ?? route.options[0];
  for (const o of route.options) {
    if (o === picked) continue;
    const way = WAY[o.transport];
    out += `<g class="map-route" ${attrs({ 'data-route': o.transport, 'data-option': o.index })}>${stamps(plan.routes[way], LINE[way].colour, 3, LINE[way].pattern, ' opacity=".4"')}</g>`;
  }
  if (picked) {
    const way = WAY[picked.transport];
    const cells = plan.routes[way];
    const { colour, pattern } = LINE[way];
    const [mx, my] = cells[Math.floor(cells.length / 2)] ?? plan.pos;
    const stop = picked.stops[0]?.category ?? 'Tresen';
    out += `<g class="map-route is-selected" data-testid="map-route" ${attrs({ 'data-route': picked.transport, 'data-option': picked.index, 'data-cells': cells.map(([x, y]) => `${x},${y}`).join(' ') })}>${stamps(cells, '#ffffff', 7, pattern)}${stamps(cells, colour, 5, pattern)}</g>`;
    out += badge(STOP_ICON[stop], mx, my, colour, stop);
  }
  const next = route.next;
  const [tx, ty] = plan.target;
  out += tile(placeIcon(next), Math.round(tx), Math.round(ty), next.is_boss, next.is_boss ? '#8a6a22' : '#7a2232', attrs({ class: 'map-target', 'data-testid': 'map-target', 'data-node': next.id }));
  const sp = route.options.find((o) => o.spelunke)?.spelunke;
  if (sp && plan.spelunke) {
    const [sx, sy] = plan.spelunke;
    out += tile('spelunke', Math.round(sx), Math.round(sy), false, '#383b42', attrs({ class: 'map-spelunke', 'data-testid': 'map-spelunke', 'data-node': sp.id }));
  }
  out += positionMark(plan.pos[0], plan.pos[1]);
  const label = t(de.route.mapAria, { from: view.tour.current?.name ?? '', to: next.name });
  return `<svg class="map-svg look-${city.look}" data-testid="map-svg" viewBox="0 0 ${MW} ${MH}" preserveAspectRatio="xMidYMid meet" shape-rendering="crispEdges" overflow="visible" role="img" ${attrs({ 'aria-label': label, 'data-leg': legKey(view), 'data-look': city.look, 'data-seed': city.key })}>${out}</svg>`;
}

// ---------------------------------------------------------------- the target card

function guestFaces(pub: PubPreview): string {
  // ADR 0017: in a boss pub both seats are the boss's figures.
  return pub.guests
    .map((g, i) => {
      const face = pub.is_boss && pub.boss ? bossPortraitId(pub.boss.id, i) : guestPortraitId(g.look, g.seat === 2 ? 2 : 1);
      return portrait(face, 'portrait-sm route-face');
    })
    .join('');
}

function guestNames(pub: PubPreview): string {
  if (pub.is_boss && pub.boss) return bossFiguresText(pub.boss);
  return pub.guests.map((g) => g.name).join(', ');
}

function targetCard(view: RunView, picked: RouteOptionView | undefined): string {
  const route = view.tour.route;
  if (!route) return '';
  const pub = picked?.spelunke ?? route.next;
  const kind = placeIcon(pub);
  let extra = '';
  if (pub.malus) {
    const options = picked?.spelunke ? (picked.malus_options ?? []) : [];
    const pickMalus = options.length
      ? `<span class="route-malus-pick" role="group" data-testid="malus-pick" ${attrs({ 'aria-label': de.route.malusPick })}><small>${esc(de.route.malusPick)}</small>${options
          .map(
            (m, i) =>
              `<button class="btn btn-small ${m.id === pub.malus?.id ? 'is-picked' : ''}" ${attrs({ 'data-action': 'choose-malus', 'data-option': picked?.index ?? 0, 'data-malus': i, 'data-info': `malus:${m.id}`, 'aria-pressed': m.id === pub.malus?.id ? 'true' : 'false' })}>${esc(contentName('maluses', m.id) || m.name)}</button>`,
          )
          .join('')}</span>`
      : '';
    extra = `<span class="route-malus" data-testid="route-malus" ${attrs({ 'data-info': `malus:${pub.malus.id}` })}><b>${esc(t(de.route.malus, { name: contentName('maluses', pub.malus.id) || pub.malus.name }))}:</b> ${esc(pub.malus.text)}</span>
      ${pickMalus}<span class="route-reward">${esc(de.route.spelunkeReward)}</span>`;
  } else if (pub.is_boss) {
    const boss = pub.boss ? contentName('bosses', pub.boss.id) : de.route.bossUnknown;
    // ADR 0017: both boss players by name.
    const text = pub.boss ? bossLine(pub.boss) : t(de.route.bossEnd, { name: boss });
    extra = `<span class="route-warn"><b class="route-tag is-boss">${esc(de.route.bossTag)}</b><span class="route-boss" data-testid="route-boss-figures">${esc(text)}</span></span>`;
  }
  const scouted = picked?.stops[0]?.encounter;
  if (scouted) extra += `<span class="route-scout" data-testid="route-scout">${esc(t(de.route.scouted, { name: contentName('encounters', scouted.id) || scouted.name }))}</span>`;
  const plan = view.next_pub;
  if (plan && (plan.redeals || plan.target)) {
    const parts: string[] = [];
    if (plan.redeals > 0) parts.push(t(de.route.planRedeals, { n: `+${plan.redeals}` }));
    if (plan.redeals < 0) parts.push(t(de.route.planRedealsFirst, { n: -plan.redeals }));
    if (plan.target) parts.push(t(de.route.planTarget, { n: plan.target > 0 ? `+${plan.target}` : `−${-plan.target}` }));
    extra += `<span class="route-plan-note" data-testid="route-plan-note">${esc(t(de.route.plan, { parts: parts.join(', ') }))}</span>`;
  }
  return `<section class="route-next ${pub.malus ? 'is-spelunke' : ''} ${pub.is_boss ? 'is-boss' : ''}" data-testid="route-next" ${attrs({ 'data-node': pub.id, 'data-kind': kind })}>
    ${mapIcon(kind, 44, undefined, 'route-kind')}
    <span class="route-text">
      <h2 class="route-pub-name">${esc(pub.name)}</h2>
      <span class="route-target" data-testid="route-target">${esc(de.route.kinds[kind] ?? '')} · ${esc(t(de.route.target, { target: pub.target, games: pub.game_limit }))}</span>
      <span class="sr-only">${esc(t(de.route.guests, { names: guestNames(pub) }))}</span>
    </span>
    <span class="route-faces" ${attrs({ title: guestNames(pub) })}>${guestFaces(pub)}</span>
    ${extra}
  </section>`;
}

// ---------------------------------------------------------------- the travel bar

function lineSample(way: Way): string {
  const { colour, pattern } = LINE[way];
  return `<span class="route-line" style="border-top: 4px ${pattern} ${colour}"></span>`;
}

function wayButton(o: RouteOptionView, picked: boolean, i: number, maxLives: number): string {
  const stop = o.stops[0]?.category ?? 'Tresen';
  const way = WAY[o.transport];
  const label = de.route.transportsShort[o.transport] ?? o.transport;
  const aria = `${label}, ${de.route.categories[stop] ?? stop}, ${t(de.route.minutes, { n: o.minutes })}${o.spelunke ? `, ${de.route.spelunkeTag}` : ''}`;
  return `<button class="route-option ${picked ? 'is-picked' : ''} ${o.spelunke ? 'is-spelunke' : ''}" style="--i:${i}" ${attrs({
    'data-action': 'pick-route',
    'data-option': o.index,
    'data-testid': 'route-option',
    'data-category': stop,
    'data-transport': o.transport,
    'data-node': o.id,
    'data-spelunke': o.spelunke ? 'true' : 'false',
    'aria-pressed': picked ? 'true' : 'false',
    'aria-label': aria,
    title: t(de.route.categoryHints[stop] ?? '', { max: maxLives }),
    disabled: !o.can_choose,
  })}>
    <span class="route-way">${mapIcon(way, 30, undefined, 'transport-icon')}<b>${esc(label)}</b></span>
    ${lineSample(way)}
    <span class="route-stop">${categoryIcon(stop, 24)}${o.spelunke ? mapIcon('spelunke', 24, undefined, 'stop-icon') : ''}<small>${esc(t(de.route.minutes, { n: o.minutes }))}</small></span>
  </button>`;
}

// ---------------------------------------------------------------- the whole tour

function planRow(icon: MapIcon, name: string, state: string, mark: string, node: string): string {
  return `<li class="plan-stop is-${state}" ${attrs({ 'data-node': node, 'data-state': state })}><i class="plan-line"></i>${mapIcon(icon, 26)}<span class="plan-name">${esc(name)}</span>${mark}</li>`;
}

function tourPlan(view: RunView): string {
  const here = view.tour.current?.id ?? '';
  const nextId = view.tour.route?.next.id ?? '';
  const current = view.tour.route?.next.section ?? view.series.section;
  const blocks = view.tour.districts.map((d) => {
    const rows = d.pubs
      .map((p) => {
        const sp = d.spelunken.find((s) => s.index === p.index);
        const id = sp ? sp.id : p.id;
        const state = sp ? sp.state : p.state;
        const icon: MapIcon = sp ? 'spelunke' : p.is_boss ? 'festsaal' : 'kneipe';
        const name = sp ? sp.name : p.name;
        let mark = '';
        if (state === 'Won') mark = `<b class="plan-mark is-won">✓</b>`;
        if (state === 'Lost') mark = `<b class="plan-mark is-lost">✗</b>`;
        if (id === here || p.id === here) mark += `<b class="plan-mark is-here" data-testid="plan-here">${esc(de.route.here)}</b>`;
        if (p.id === nextId) mark += `<b class="plan-mark is-next">${esc(de.route.goal)}</b>`;
        const shown = state === 'Won' || state === 'Lost' ? 'done' : id === here ? 'done' : p.id === nextId ? 'next' : 'ahead';
        return planRow(icon, name, shown, mark, id);
      })
      .join('');
    const cls = d.index < current ? 'is-done' : d.index === current ? 'is-current' : 'is-ahead';
    return `<section class="plan-district ${cls}" ${attrs({ 'data-district': d.index, 'data-look': d.look })}><h3>${esc(t(de.route.district, { n: d.index + 1, name: d.name }))}</h3><ol>${rows}</ol></section>`;
  });
  return `<div class="route-plan" data-testid="route-plan"><h2>${esc(de.route.planTitle)}</h2>${blocks.join('')}</div>`;
}

/** "+" and "−" as pixel bars (the font's minus is too thin). */
function zoomGlyph(plus: boolean): string {
  return `<svg viewBox="0 0 9 9" width="18" height="18" shape-rendering="crispEdges" aria-hidden="true"><path d="M0 4h9v1h-9z${plus ? 'M4 0h1v9h-1z' : ''}" fill="currentColor"/></svg>`;
}

function zoomButtons(overview: boolean): string {
  return `<div class="map-zoom">
    <button class="map-zoom-btn" ${attrs({ 'data-action': 'route-map', 'data-testid': 'zoom-in', 'aria-label': de.route.zoomIn, title: de.route.zoomIn, disabled: !overview })}>${zoomGlyph(true)}</button>
    <button class="map-zoom-btn" ${attrs({ 'data-action': 'route-overview', 'data-testid': 'zoom-out', 'aria-label': de.route.zoomOut, title: de.route.zoomOut, disabled: overview })}>${zoomGlyph(false)}</button>
  </div>`;
}

export function renderRoute(view: RunView, pick: number, overview: boolean): string {
  const route = view.tour.route;
  if (!route) return '';
  const picked = route.options.find((o) => o.index === pick && o.can_choose) ?? route.options.find((o) => o.index === defaultPick(view));
  const sel = picked?.index ?? 0;
  const buttons = route.options.map((o, i) => wayButton(o, o.index === sel, i, view.tour.max_lives)).join('');
  const body = overview ? tourPlan(view) : `${mapSvg(view, sel)}${targetCard(view, picked)}`;
  const go = picked ? t(de.route.go, { way: de.route.transportsShort[picked.transport] ?? '' }) : de.route.goShort;
  return `<main class="route ${overview ? 'is-overview' : ''}" data-testid="route" ${attrs({ 'aria-label': de.route.title })}>
    <div class="route-map ${picked?.spelunke ? 'is-spelunke' : ''}" data-testid="route-map">
      ${body}
      ${zoomButtons(overview)}
    </div>
    <div class="route-bar">
      <div class="route-options">${buttons}</div>
      <button class="btn route-go" ${attrs({ 'data-action': 'choose-route', 'data-option': sel, 'data-testid': 'route-go', disabled: !picked?.can_choose })}>${esc(go)}</button>
    </div>
  </main>`;
}

// ---------------------------------------------------------------- travelling

/**
 * The trip: a small pixel walker (the icon of the chosen transport: on
 * foot, train or bike) goes along the chosen way and leaves a chalk trail;
 * the walker hops a little on foot and turns to face where it goes.
 * Resolves when it arrives, at once after a tap on the screen, or when the
 * table is hurried (finishAll: the trip is timed by a tracked animation).
 */
export function playTravel(root: HTMLElement, ms: number): Promise<void> {
  const svg = root.querySelector<SVGSVGElement>('.map-svg');
  const sel = svg?.querySelector<SVGGElement>('.map-route.is-selected');
  const pos = svg?.querySelector<SVGGElement>('.map-pos');
  const cells = (sel?.dataset.cells ?? '')
    .split(' ')
    .filter(Boolean)
    .map((c) => c.split(',').map(Number) as [number, number]);
  if (!svg || !sel || !pos || cells.length < 2 || !(ms > 0)) return Promise.resolve();
  const NS = 'http://www.w3.org/2000/svg';
  const trail = document.createElementNS(NS, 'path');
  trail.setAttribute('fill', PAPER);
  trail.setAttribute('class', 'map-trail');
  pos.before(trail);
  const [x0, y0] = cells[0] as [number, number];
  // The walker stands on the position, its feet on the way.
  const way = WAY[(sel.dataset.route ?? 'Walk') as Transport] ?? 'fuss';
  const walker = document.createElementNS(NS, 'g');
  walker.setAttribute('class', 'map-walker');
  walker.dataset.testid = 'map-walker';
  walker.dataset.way = way;
  const body = document.createElementNS(NS, 'g');
  // Transparent like the other map icons: only the glyph, no paper box behind it.
  body.innerHTML = glyph(way, x0 - 10, y0 - 19, 1.25);
  walker.appendChild(body);
  pos.appendChild(walker);
  root.querySelector('.route')?.classList.add('is-travelling');
  const hops = way === 'fuss';
  // Tracked, so finishAll() ends the trip with the rest of the table.
  const timer = play(trail, [], { duration: ms });
  return new Promise((resolve) => {
    let done = false;
    let facing = 1;
    const finish = (): void => {
      if (done) return;
      done = true;
      root.removeEventListener('pointerdown', finish);
      resolve();
    };
    root.addEventListener('pointerdown', finish);
    const place = (p: number): void => {
      const k = Math.floor((cells.length - 1) * p);
      const [x, y] = cells[k] as [number, number];
      const [nx] = cells[Math.min(cells.length - 1, k + 1)] as [number, number];
      if (nx !== x) facing = nx < x ? -1 : 1;
      trail.setAttribute('d', cells.slice(0, k + 1).map(([cx, cy]) => `M${cx - 1} ${cy - 1}h2v2h-2z`).join(''));
      pos.setAttribute('transform', `translate(${x - x0} ${y - y0})`);
      // A hop every few cells on foot; mirrored around its own middle when going left.
      const lift = hops && p < 1 && Math.floor(k / 3) % 2 ? -2 : 0;
      body.setAttribute('transform', `translate(0 ${lift})${facing < 0 ? ` translate(${2 * x0} 0) scale(-1 1)` : ''}`);
    };
    const frame = (): void => {
      if (done) return;
      // Progress from the timer, clamped: never before the first cell, never past the last.
      const p = timer.playState === 'finished' ? 1 : Math.min(1, Math.max(0, Number(timer.currentTime ?? 0) / ms));
      place(p);
      if (p < 1) requestAnimationFrame(frame);
    };
    const arrived = (): void => {
      if (done) return;
      place(1);
      window.setTimeout(finish, Math.min(400, ms / 4));
    };
    timer.finished.then(arrived, finish);
    requestAnimationFrame(frame);
  });
}

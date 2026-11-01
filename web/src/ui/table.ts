// The table: opponents on top, the open trick in the middle, the player's
// hand at the bottom. While declaring, the Ansage panel takes the middle.

import { de, cardName, gameName, t } from '../i18n';
import type { CardView, DeclareOptions, GameType, GameView, PlayedCard, RunView, Seat } from '../types';
import { cardBack, cardSvg, suitIcon } from './cards';
import { attrs, esc } from './dom';
import type { UiState } from './state';

function seatName(seat: Seat): string {
  return de.seats[seat] ?? '';
}

function miniCard(card: CardView): string {
  return `<span class="mini-card" title="${esc(cardName(card.kind))}">${cardSvg(card.kind, card.mods)}</span>`;
}

function opponent(game: GameView, seat: Seat, ui: UiState): string {
  const size = ui.anim ? (ui.anim.handSizes[seat] ?? 0) : (game.hand_sizes[seat] ?? 0);
  const turn = ui.anim ? ui.anim.turn === seat : false;
  const backs = Array.from({ length: Math.min(size, 10) }, () => cardBack('fan')).join('');
  const kontra = game.kontra;
  let badge = '';
  if (typeof kontra === 'object' && 'Kontra' in kontra && kontra.Kontra === seat) {
    badge = `<span class="badge badge-kontra">${esc(de.table.kontraBadge)}</span>`;
  } else if (typeof kontra === 'object' && 'Re' in kontra && kontra.Re === seat) {
    badge = `<span class="badge badge-kontra">${esc(de.table.kontraBadge)}</span>`;
  }
  const revealed = game.revealed
    .filter((r) => r.holder === seat)
    .map((r) => miniCard(r.card))
    .join('');
  const position = game.positions[seat];
  return `<div class="opp ${turn ? 'is-turn' : ''}" data-seat="${seat}">
    <div class="opp-head"><b>${esc(seatName(seat))}</b>${badge}</div>
    <div class="opp-row">
      <span class="opp-pos">${esc(position ? de.positions[position] : '')}</span>
      <span class="opp-hand">${backs}<span class="opp-count" aria-label="${esc(t(de.table.cards, { count: size }))}">${esc(size)}</span></span>
    </div>
    ${revealed ? `<div class="opp-revealed"><span>${esc(de.table.holds)}</span>${revealed}</div>` : ''}
  </div>`;
}

function trickSlot(cards: PlayedCard[], seat: Seat): string {
  const played = cards.find((c) => c.seat === seat);
  const inner = played
    ? `<div class="card trick-card" data-trick-seat="${seat}">${cardSvg(played.card.kind, played.card.mods)}</div>`
    : `<div class="card-slot"></div>`;
  return `<div class="slot slot-${seat}">${inner}<span class="slot-label">${esc(de.seatsShort[seat] ?? '')}</span></div>`;
}

function gameIcon(game: GameType): string {
  if (game === 'Grand' || game === 'Null') {
    return `<span class="grand-icon">${(['Clubs', 'Spades', 'Hearts', 'Diamonds'] as const).map((s) => suitIcon(s, 14)).join('')}</span>`;
  }
  return suitIcon(game.Suit, 30);
}

function declarePanel(options: DeclareOptions, ui: UiState, passesLeft: number): string {
  const games = options.games
    .map((g, i) => {
      const picked = ui.pickGame === i;
      return `<button class="game-btn ${picked ? 'is-picked' : ''}" ${attrs({
        'data-action': 'pick-game',
        'data-index': i,
        disabled: !g.allowed,
        'aria-pressed': picked ? 'true' : 'false',
      })}>${gameIcon(g.game)}<span class="game-name">${esc(gameName(g.game))}</span><span class="game-base">${esc(g.base_value)}</span></button>`;
    })
    .join('');
  const risk = ui.pickRisk ?? options.risks.find((r) => r.allowed)?.risk ?? 'Normal';
  const risks = options.risks
    .map((r) => {
      const picked = r.risk === risk;
      return `<button class="risk-btn ${picked ? 'is-picked' : ''}" ${attrs({
        'data-action': 'pick-risk',
        'data-risk': r.risk,
        disabled: !r.allowed,
        'aria-pressed': picked ? 'true' : 'false',
      })}>${esc(de.risksShort[r.risk])}</button>`;
    })
    .join('');
  const hint = options.hand_required ? `<p class="declare-hint is-warning">${esc(de.table.handRequired)}</p>` : '';
  const redraw = options.can_redeal
    ? `<button class="btn" ${attrs({ 'data-action': 'redraw' })}>${esc(t(de.table.redraw, { left: options.redeals_left }))}</button>`
    : '';
  return `<section class="declare" aria-label="${esc(de.table.declareTitle)}">
    <div class="declare-label">${esc(de.table.gameType)}</div>
    <div class="games">${games}</div>
    <div class="declare-label">${esc(de.table.risk)}</div>
    <div class="risks">${risks}</div>
    <p class="declare-hint">${esc(de.riskHints[risk])}</p>
    ${hint}
    <div class="declare-actions">
      <button class="btn" ${attrs({ 'data-action': 'pass', disabled: !options.can_pass })}>${esc(t(de.table.pass, { left: passesLeft }))}</button>
      ${redraw}
      <button class="btn btn-primary" ${attrs({ 'data-action': 'declare', disabled: ui.pickGame === null })}>${esc(de.table.declare)}</button>
    </div>
  </section>`;
}

function handCards(game: GameView, ui: UiState): string {
  const hand = ui.anim ? ui.anim.hand : game.hand;
  const discarding = !ui.anim && game.phase === 'Discarding' && !game.awaiting_kontra_answer;
  const playing = !ui.anim && game.your_turn;
  const cols = Math.max(5, Math.ceil(hand.length / 2));
  const cards = hand
    .map((c) => {
      const selected = ui.selected.includes(c.id);
      let action: string | null = null;
      if (discarding) action = 'select';
      else if (playing && c.legal) action = 'play';
      const dim = playing && !c.legal;
      const cls = ['card', 'hand-card', dim ? 'is-dim' : '', selected ? 'is-selected' : '', action ? 'is-active' : ''].join(' ');
      const skatTag = c.from_skat ? `<span class="skat-tag">${esc(de.card.fromSkat)}</span>` : '';
      const label = cardName(c.kind);
      if (action) {
        return `<button class="${cls}" ${attrs({
          'data-action': action,
          'data-id': c.id,
          'data-legal': c.legal ? 'true' : 'false',
          'aria-pressed': action === 'select' ? (selected ? 'true' : 'false') : null,
          'aria-label': label,
        })}>${cardSvg(c.kind, c.mods)}${skatTag}</button>`;
      }
      return `<div class="${cls}" ${attrs({ 'data-id': c.id, 'aria-disabled': 'true' })}>${cardSvg(c.kind, c.mods)}${skatTag}</div>`;
    })
    .join('');
  return `<div class="hand" style="--cols:${cols}" data-testid="hand">${cards}</div>`;
}

function statusLine(game: GameView, ui: UiState): string {
  if (ui.anim) {
    return `<div class="status">${esc(ui.anim.status)}</div>`;
  }
  switch (game.phase) {
    case 'Declaring':
      return `<div class="status">${esc(t(de.table.game, { number: game.number }))} · ${esc(de.positions[game.positions[0] ?? 'Forehand'])}</div>`;
    case 'Discarding':
      return `<div class="status status-discard"><span>${esc(de.table.discardHint)}</span>
        <button class="btn btn-primary" ${attrs({ 'data-action': 'discard', disabled: ui.selected.length !== 2 })}>${esc(de.table.discard)}</button></div>`;
    case 'Playing':
      return `<div class="status ${game.your_turn ? 'is-your-turn' : ''}">${esc(game.your_turn ? de.table.yourTurn : '')}</div>`;
    default:
      return `<div class="status"></div>`;
  }
}

function gameLine(game: GameView): string {
  if (!game.declaration) return '';
  const d = game.declaration;
  const kontra =
    game.kontra === 'None'
      ? ''
      : typeof game.kontra === 'object' && 'Re' in game.kontra
        ? `<span class="badge badge-kontra">${esc(de.table.kontraBadge)}</span><span class="badge badge-kontra">${esc(de.table.reBadge)}</span>`
        : `<span class="badge badge-kontra">${esc(de.table.kontraBadge)}</span>`;
  const skat = game.skat
    ? `<span class="skat"><span>${esc(de.table.skat)}</span>${game.skat.map(miniCard).join('')}</span>`
    : '';
  return `<div class="gameline">
    <span class="decl">${esc(t(de.table.declaration, { game: gameName(d.game), risk: de.risks[d.risk] }))}</span>
    ${kontra}
    <span class="tricks">${esc(t(de.table.tricks, { own: game.declarer_tricks, theirs: game.defender_tricks }))}</span>
    ${skat}
  </div>`;
}

export function renderTable(view: RunView, ui: UiState): string {
  const game = view.game;
  if (!game) return '';
  const declaring = game.phase === 'Declaring' && !ui.anim && game.declare;
  const trick = ui.anim ? ui.anim.trick : game.trick;
  const center = declaring && game.declare
    ? declarePanel(game.declare, ui, view.series.passes_left)
    : `<div class="trick" data-action="fast">${trickSlot(trick, 1)}${trickSlot(trick, 2)}${trickSlot(trick, 0)}</div>`;
  const toast = ui.toast ? `<div class="toast" role="status">${esc(ui.toast)}</div>` : '';
  return `<div class="table phase-${game.phase.toLowerCase()}">
    <div class="opponents">${opponent(game, 1, ui)}${opponent(game, 2, ui)}</div>
    <div class="center">${center}${toast}</div>
    ${gameLine(game)}
    ${statusLine(game, ui)}
    ${handCards(game, ui)}
  </div>`;
}

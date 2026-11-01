// The table: the two regulars (or the boss) on top, the open trick on the
// felt, the player's hand at the bottom in two rows. While declaring, the
// Ansage panel takes the middle (for the Nullspieler with the Null levels).
// Around it: the boss rule of this game, the Ramsch notice, the count of
// "Der Zähler", the Kiebitz card and the Oma's offer, all from the view.

import { de, cardName, contentName, contentText, declarationName, gameName, t } from '../i18n';
import type { BossInfo, CardView, DeclaredGame, DeclareOptions, GameView, NullRisk, PlayedCard, RunView, Seat } from '../types';
import { cardBack, cardSvg, suitIcon } from './cards';
import { attrs, esc } from './dom';
import { bossPortraitId, portrait } from './people';
import type { UiState } from './state';

function seatName(seat: Seat): string {
  return de.seats[seat] ?? '';
}

function miniCard(card: CardView): string {
  return `<span class="mini-card" title="${esc(cardName(card.kind))}">${cardSvg(card.kind, card.mods)}</span>`;
}

/** An eye, for the Kiebitz. */
function eyeIcon(size = 14): string {
  return `<svg class="eye-icon" viewBox="0 0 24 16" width="${size}" height="${Math.round((size * 2) / 3)}" aria-hidden="true"><path d="M1 8 C5 1 19 1 23 8 C19 15 5 15 1 8 Z" fill="#fbf3e2" stroke="#2b1a0e" stroke-width="1.6"/><circle cx="12" cy="8" r="4" fill="#2f6aa8"/><circle cx="12" cy="8" r="1.8" fill="#111"/></svg>`;
}

/**
 * A card the player sees in a defender's hand. Cards shown by the Kiebitz
 * are not "Durchsichtig" (their `revealed` flag is unset) and get his eye.
 */
function revealedCard(card: CardView): string {
  if (card.revealed) return miniCard(card);
  return `<span class="mini-card is-kiebitz" title="${esc(`${cardName(card.kind)} (${de.table.kiebitzHint})`)}" data-kiebitz="${esc(card.id)}">${cardSvg(card.kind, card.mods)}</span>`;
}

function kontraBadge(game: GameView, seat: Seat): string {
  const k = game.kontra;
  if (typeof k === 'object' && (('Kontra' in k && k.Kontra === seat) || ('Re' in k && k.Re === seat))) {
    return `<span class="badge badge-kontra">${esc(de.table.kontraBadge)}</span>`;
  }
  return '';
}

function opponent(view: RunView, game: GameView, seat: Seat, ui: UiState): string {
  const size = ui.anim ? (ui.anim.handSizes[seat] ?? 0) : (game.hand_sizes[seat] ?? 0);
  const turn = ui.anim ? ui.anim.turn === seat : false;
  const backs = Array.from({ length: Math.min(size, 12) }, () => cardBack('fan')).join('');
  const seen = game.revealed.filter((r) => r.holder === seat);
  const open = seen.filter((r) => r.card.revealed).map((r) => revealedCard(r.card)).join('');
  const peeked = seen.filter((r) => !r.card.revealed).map((r) => revealedCard(r.card)).join('');
  const revealed =
    (open ? `<span>${esc(de.table.holds)}</span>${open}` : '') +
    (peeked ? `<span class="kiebitz-tag">${eyeIcon(14)}${esc(de.table.kiebitz)}</span>${peeked}` : '');
  const position = game.positions[seat];
  // Bosses sit in the left chair with their own figure and name.
  const boss = seat === 1 && view.series.is_boss && view.series.boss ? view.series.boss : null;
  const face = boss ? bossPortraitId(boss.id) : `p-opp-${seat}`;
  const name = boss ? contentName('bosses', boss.id) : seatName(seat);
  return `<div class="opp ${turn ? 'is-turn' : ''} ${boss ? 'is-boss' : ''}" data-seat="${seat}">
    ${portrait(face, 'portrait-opp')}
    <div class="opp-info">
      <div class="opp-head"><b class="opp-name">${esc(name)}</b></div>
      <div class="opp-pos">${esc(position ? de.positions[position] : '')}${kontraBadge(game, seat)}</div>
      <div class="opp-hand">${backs}<span class="opp-count" aria-label="${esc(t(de.table.cards, { count: size }))}">${esc(size)}</span></div>
    </div>
    ${revealed ? `<div class="opp-revealed">${revealed}</div>` : ''}
  </div>`;
}

function trickSlot(cards: PlayedCard[], seat: Seat): string {
  const played = cards.find((c) => c.seat === seat);
  const inner = played
    ? `<div class="card trick-card" data-trick-seat="${seat}">${cardSvg(played.card.kind, played.card.mods)}</div>`
    : `<div class="card-slot"></div>`;
  return `<div class="slot slot-${seat}">${inner}</div>`;
}

function gameIcon(game: DeclaredGame, size = 30): string {
  if (game === 'Grand' || game === 'Null' || game === 'Ramsch') {
    return `<span class="grand-icon">${(['Clubs', 'Spades', 'Hearts', 'Diamonds'] as const).map((s) => suitIcon(s, Math.round(size / 2), null)).join('')}</span>`;
  }
  return suitIcon(game.Suit, size);
}

/** The Ansage of the Nullspieler: the four Null levels with their values from the core. */
function nullDeclarePanel(options: DeclareOptions, ui: UiState, passesLeft: number): string {
  const risk: NullRisk = ui.pickNullRisk ?? options.null_risks.find((r) => r.allowed)?.risk ?? 'Normal';
  const risks = options.null_risks
    .map((r) => {
      const picked = r.risk === risk;
      return `<button class="null-btn ${picked ? 'is-picked' : ''}" ${attrs({
        'data-action': 'pick-null-risk',
        'data-risk': r.risk,
        disabled: !r.allowed,
        'aria-pressed': picked ? 'true' : 'false',
      })}><span class="null-name">${esc(de.nullRisks[r.risk])}</span><span class="null-value">${esc(t(de.table.nullValue, { value: r.value }))}</span></button>`;
    })
    .join('');
  const hints = [
    options.ouvert_required ? de.table.ouvertRequired : '',
    options.hand_required ? de.table.handRequired : '',
  ]
    .filter(Boolean)
    .map((h) => `<p class="declare-hint is-warning">${esc(h)}</p>`)
    .join('');
  const redraw = options.can_redeal
    ? `<button class="btn" ${attrs({ 'data-action': 'redraw' })}>${esc(t(de.table.redraw, { left: options.redeals_left }))}</button>`
    : '';
  const anyAllowed = options.null_risks.some((r) => r.allowed);
  return `<section class="declare declare-null" aria-label="${esc(de.table.nullTitle)}" data-testid="null-declare">
    <h2 class="declare-title">${esc(de.table.nullTitle)}</h2>
    <p class="no-trick" role="note"><b>${esc(de.table.noTrick)}</b> <span>${esc(de.table.nullHint)}</span></p>
    <div class="null-risks">${risks}</div>
    <p class="declare-hint">${esc(de.nullRiskHints[risk])} ${esc(de.table.nullStyle)}</p>
    ${hints}
    <div class="declare-actions">
      <button class="btn" ${attrs({ 'data-action': 'pass', disabled: !options.can_pass })}>${esc(t(de.table.pass, { left: passesLeft }))}</button>
      ${redraw}
      <button class="btn btn-primary" ${attrs({ 'data-action': 'declare-null', 'data-risk': risk, disabled: !anyAllowed })}>${esc(de.table.declare)}</button>
    </div>
  </section>`;
}

function declarePanel(options: DeclareOptions, ui: UiState, passesLeft: number): string {
  if (options.null_risks.length) return nullDeclarePanel(options, ui, passesLeft);
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
  const hint = [options.hand_required ? de.table.handRequired : '', options.ouvert_required ? de.table.ouvertRequired : '']
    .filter(Boolean)
    .map((h) => `<p class="declare-hint is-warning">${esc(h)}</p>`)
    .join('');
  const redraw = options.can_redeal
    ? `<button class="btn" ${attrs({ 'data-action': 'redraw' })}>${esc(t(de.table.redraw, { left: options.redeals_left }))}</button>`
    : '';
  return `<section class="declare" aria-label="${esc(de.table.declareTitle)}">
    <h2 class="declare-title">${esc(de.table.declareTitle)}</h2>
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
  const oma = !ui.anim && ui.omaPick && game.can_take_from_skat;
  const playing = !ui.anim && game.your_turn && !oma;
  const cols = Math.max(5, Math.ceil(hand.length / 2));
  const cards = hand
    .map((c, i) => {
      const selected = ui.selected.includes(c.id);
      let action: string | null = null;
      if (discarding) action = 'select';
      else if (oma) action = 'oma-give';
      else if (playing && c.legal) action = 'play';
      const dim = playing && !c.legal;
      const cls = ['card', 'hand-card', dim ? 'is-dim' : '', selected ? 'is-selected' : '', action ? 'is-active' : ''].join(' ');
      const skatTag = c.from_skat ? `<span class="skat-tag">${esc(de.card.fromSkat)}</span>` : '';
      const label = cardName(c.kind);
      // Slight arc per row, like cards held in the hand.
      const off = (i % cols) - (cols - 1) / 2;
      const style = `--tilt:${(off * 1.6).toFixed(1)}deg;--lift:${(off * off * 0.9).toFixed(1)}px`;
      if (action) {
        return `<button class="${cls}" style="${style}" ${attrs({
          'data-action': action,
          'data-id': c.id,
          'data-legal': c.legal ? 'true' : 'false',
          'aria-pressed': action === 'select' ? (selected ? 'true' : 'false') : null,
          'aria-label': label,
        })}>${cardSvg(c.kind, c.mods)}${skatTag}</button>`;
      }
      return `<div class="${cls}" style="${style}" ${attrs({ 'data-id': c.id, 'aria-disabled': 'true' })}>${cardSvg(c.kind, c.mods)}${skatTag}</div>`;
    })
    .join('');
  const open = isOpen(game);
  const banner = open ? `<p class="open-banner">${eyeIcon(16)}<span>${esc(de.table.openHand)}</span></p>` : '';
  return `${banner}<div class="hand ${open ? 'is-open' : ''} ${oma ? 'is-oma' : ''}" style="--cols:${cols}" data-testid="hand">${cards}</div>`;
}

/** Whether the player plays with open cards (Ouvert, also Null Ouvert). */
function isOpen(game: GameView): boolean {
  const risk = game.declaration?.risk;
  return game.phase !== 'Declaring' && (risk === 'Ouvert' || risk === 'OuvertHand');
}

function statusLine(game: GameView, ui: UiState): string {
  if (ui.anim) {
    return `<div class="status" aria-live="polite">${esc(ui.anim.status)}</div>`;
  }
  switch (game.phase) {
    case 'Declaring':
      return `<div class="status">${esc(t(de.table.game, { number: game.number }))} · ${esc(de.positions[game.positions[0] ?? 'Forehand'])}</div>`;
    case 'Discarding':
      return `<div class="status status-discard"><span>${esc(de.table.discardHint)}</span>
        <button class="btn btn-primary" ${attrs({ 'data-action': 'discard', disabled: ui.selected.length !== 2 })}>${esc(de.table.discard)}</button></div>`;
    case 'Playing':
      if (game.can_take_from_skat && ui.omaPick) {
        return `<div class="status status-oma"><span>${esc(de.table.omaHint)}</span>
          <button class="btn" ${attrs({ 'data-action': 'oma-cancel' })}>${esc(de.table.omaCancel)}</button></div>`;
      }
      if (game.can_take_from_skat) {
        return `<div class="status ${game.your_turn ? 'is-your-turn' : ''}"><span>${esc(game.your_turn ? de.table.yourTurn : '')}</span>
          <button class="btn btn-oma" ${attrs({ 'data-action': 'oma' })}>${portrait('p-boss-grandma', 'portrait-xs')}<span>${esc(de.table.oma)}</span></button></div>`;
      }
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
  const special =
    d.game === 'Null'
      ? `<span class="badge badge-null">${esc(de.table.noTrick)}</span>`
      : '';
  return `<div class="gameline">
    <span class="decl">${gameIcon(d.game, 20)}<span>${esc(declarationName(d))}</span></span>
    ${special}
    ${kontra}
    <span class="tricks" data-tricks>${esc(t(de.table.tricks, { own: game.declarer_tricks, theirs: game.defender_tricks }))}</span>
    ${skat}
  </div>`;
}

/**
 * The boss rule in force in this game. "Der Alte" lays another boss's rule
 * on every game; then the strip names him and the rule of this game.
 */
function ruleStrip(view: RunView, game: GameView): string {
  const rule: BossInfo | null = game.boss_rule;
  if (!rule) return '';
  const boss = view.series.boss;
  const laidOn = boss && boss.id !== rule.id;
  // The Ramsch has its own notice; only "Der Alte" needs the strip then.
  if (game.declaration?.game === 'Ramsch' && !laidOn) return '';
  const head = laidOn ? t(de.table.oldMan, { name: contentName('bosses', boss.id) }) : de.table.bossRule;
  return `<button class="rule-strip ${laidOn ? 'is-laid-on' : ''}" ${attrs({ 'data-action': 'info-rule', 'data-testid': 'boss-rule' })}>
    ${portrait(bossPortraitId(laidOn ? boss.id : rule.id), 'portrait-xs')}
    <span class="rule-head">${esc(head)}</span>
    <b class="rule-name">${esc(contentName('bosses', rule.id))}</b>
    <span class="rule-text">${esc(contentText('bosses', rule.id) || rule.text)}</span>
  </button>`;
}

/** The count of "Der Zähler", as chalk on a little slate. */
function countPanel(game: GameView): string {
  const c = game.count;
  if (!c || !game.declaration) return '';
  const cell = (label: string, n: number, cls: string): string => `<span class="count-cell ${cls}"><span>${esc(label)}</span><b>${esc(n)}</b></span>`;
  return `<div class="counter-slate" data-testid="count" aria-label="${esc(de.table.countTitle)}">
    <span class="counter-title">${esc(de.table.countTitle)}</span>
    ${cell(de.table.countYou, c.declarer, 'is-you')}${cell(de.table.countThem, c.defenders, 'is-them')}${cell(de.table.countUnseen, c.unseen, 'is-unseen')}
  </div>`;
}

/** The Ramsch: nobody declared, everyone plays for himself. */
function ramschBanner(game: GameView): string {
  if (game.declaration?.game !== 'Ramsch' || game.phase === 'Finished') return '';
  return `<div class="ramsch-banner" data-testid="ramsch">${portrait('p-boss-ramscher', 'portrait-xs')}<div><b>${esc(de.table.ramschTitle)}</b> <span>${esc(de.table.ramschText)}</span></div></div>`;
}

/** "Letzte Runde": this game counts more (shown while declaring). */
function pendingNote(view: RunView, game: GameView): string {
  if (!view.pending_factor || game.phase !== 'Declaring') return '';
  return `<p class="pending-note">${esc(t(de.table.pendingFactor, { factor: view.pending_factor }))}</p>`;
}

export function renderTable(view: RunView, ui: UiState): string {
  const game = view.game;
  if (!game) return '';
  const declaring = game.phase === 'Declaring' && !ui.anim && game.declare;
  const trick = ui.anim ? ui.anim.trick : game.trick;
  const center = declaring && game.declare
    ? declarePanel(game.declare, ui, view.series.passes_left)
    : `<div class="trick" data-action="fast">${trickSlot(trick, 1)}${trickSlot(trick, 2)}${trickSlot(trick, 0)}</div>`;
  const ramsch = game.declaration?.game === 'Ramsch';
  return `<div class="table phase-${game.phase.toLowerCase()} ${ramsch ? 'is-ramsch' : ''}">
    <div class="opponents">${opponent(view, game, 1, ui)}${opponent(view, game, 2, ui)}</div>
    ${ruleStrip(view, game)}${ramschBanner(game)}${pendingNote(view, game)}
    <div class="center"><div class="deck-spot" aria-hidden="true"></div>${center}</div>
    ${gameLine(game)}
    ${countPanel(game)}
    ${statusLine(game, ui)}
    ${handCards(game, ui)}
  </div>`;
}

// The table: the two regulars (or the boss) on top, the open trick on the
// felt, the player's hand at the bottom in two rows. While declaring, the
// Ansage panel takes the middle (for the Nullspieler with the Null levels).
// Around it: the boss rule of this game, the Ramsch notice, the count of
// "Der Zähler", the Kiebitz card and the skat swap of "Die Oma" or "Der
// Opa", all from the view. Card flags the core resolved ("Unter
// ehrenhalber", "Ramponiert") show as plaques on the hand cards.

import { de, cardName, contentName, contentText, declarationName, gameName, t } from '../i18n';
import type { BossInfo, CardView, DeclaredGame, DeclareOptions, GameView, NullRisk, PlayedCard, RevealedCard, RunView, Seat } from '../types';
import { cardBack, cardSvg, modIcon, suitIcon } from './cards';
import { lockIcon } from './icons';
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
 * A card the player sees in a defender's hand, marked by why he knows it
 * (the core's `source`): the Kiebitz's eye, or the modification's badge.
 */
function revealedCard(r: RevealedCard): string {
  const card = r.card;
  if (r.source === 'Kibitz') {
    return `<span class="mini-card is-kiebitz" title="${esc(`${cardName(card.kind)} (${de.table.kiebitzHint})`)}" data-kiebitz="${esc(card.id)}">${cardSvg(card.kind, card.mods)}</span>`;
  }
  const mod = r.source.CardMod;
  return `<span class="mini-card is-revealed" ${attrs({ title: `${cardName(card.kind)} (${contentName('card_mods', mod)})`, 'data-reveal-mod': mod })}>${cardSvg(card.kind, card.mods)}<span class="reveal-mod">${modIcon(mod, 14)}</span></span>`;
}

/** A small thought bubble while the core computes this opponent's card. */
function thinkBubble(): string {
  return `<span class="think-bubble" data-testid="think" role="status" aria-label="${esc(de.table.thinking)}"><i></i><i></i><i></i></span>`;
}

/** Plaques for what the core resolved on a card: "Unter ehrenhalber", "Ramponiert". */
function plaques(c: CardView): string {
  const out: string[] = [];
  if (c.honorary_jack) out.push(`<span class="plaque plaque-hj" title="${esc(de.card.honoraryJack)}" aria-hidden="true">U</span>`);
  if (c.no_discard) out.push(`<span class="plaque plaque-lock" title="${esc(de.card.noDiscard)}" aria-hidden="true">${lockIcon(13)}</span>`);
  return out.length ? `<span class="plaques">${out.join('')}</span>` : '';
}

/** Card name plus what its plaques say, for screen readers. */
function cardLabel(c: CardView): string {
  const extra = [c.honorary_jack ? de.card.honoraryJack : '', c.no_discard ? de.card.noDiscard : ''].filter(Boolean);
  return extra.length ? `${cardName(c.kind)} (${extra.join(', ')})` : cardName(c.kind);
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
  const turn = ui.anim ? ui.anim.turn === seat : view.ai_to_move === seat;
  const thinking = ui.thinking === seat;
  const backs = Array.from({ length: Math.min(size, 12) }, () => cardBack('fan')).join('');
  const seen = game.revealed.filter((r) => r.holder === seat);
  const open = seen.filter((r) => r.source !== 'Kibitz').map(revealedCard).join('');
  const peeked = seen.filter((r) => r.source === 'Kibitz').map(revealedCard).join('');
  const revealed =
    (open ? `<span>${esc(de.table.holds)}</span>${open}` : '') +
    (peeked ? `<span class="kiebitz-tag">${eyeIcon(14)}${esc(de.table.kiebitz)}</span>${peeked}` : '');
  const position = game.positions[seat];
  // Bosses sit in the left chair with their own figure and name.
  const boss = seat === 1 && view.series.is_boss && view.series.boss ? view.series.boss : null;
  const face = boss ? bossPortraitId(boss.id) : `p-opp-${seat}`;
  const name = boss ? contentName('bosses', boss.id) : seatName(seat);
  const tricks = game.phase === 'Playing' || game.phase === 'Finished' ? (game.seat_tricks[seat] ?? 0) : null;
  const trickTag =
    tricks === null
      ? ''
      : `<span class="opp-tricks" aria-label="${esc(t(de.table.oppTricksAria, { n: tricks }))}" data-testid="opp-tricks">${esc(t(de.table.oppTricks, { n: tricks }))}</span>`;
  return `<div class="opp ${turn ? 'is-turn' : ''} ${boss ? 'is-boss' : ''} ${thinking ? 'is-thinking' : ''}" data-seat="${seat}">
    ${portrait(face, 'portrait-opp')}${thinking ? thinkBubble() : ''}
    <div class="opp-info">
      <div class="opp-head"><b class="opp-name">${esc(name)}</b></div>
      <div class="opp-pos">${esc(position ? de.positions[position] : '')}${trickTag}${kontraBadge(game, seat)}</div>
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

/**
 * The buttons below an Ansage: back to the choice of skat or hand (when the
 * skat may still be picked up), else Einpassen; Schieben; Ansagen.
 */
function declareActions(options: DeclareOptions, afterSkat: boolean, passesLeft: number, declare: string): string {
  const back = !afterSkat && options.can_pick_up_skat;
  const first = back
    ? `<button class="btn" ${attrs({ 'data-action': 'hand-back' })}>${esc(de.table.back)}</button>`
    : !afterSkat || options.can_pass
      ? `<button class="btn" ${attrs({ 'data-action': 'pass', disabled: !options.can_pass })}>${esc(t(de.table.pass, { left: passesLeft }))}</button>`
      : '';
  const redraw = options.can_redeal
    ? `<button class="btn" ${attrs({ 'data-action': 'redraw' })}>${esc(t(de.table.redraw, { left: options.redeals_left }))}</button>`
    : '';
  return `<div class="declare-actions">${first}${redraw}${declare}</div>`;
}

/** Warnings of the boss rules on the Ansage. */
function declareWarnings(options: DeclareOptions, afterSkat: boolean): string {
  return [
    options.hand_required ? de.table.handRequired : '',
    options.ouvert_required ? de.table.ouvertRequired : '',
    !afterSkat && !options.can_pick_up_skat && !options.hand_required && !options.ouvert_required ? de.table.noPickUp : '',
  ]
    .filter(Boolean)
    .map((h) => `<p class="declare-hint is-warning">${esc(h)}</p>`)
    .join('');
}

/** The two discarded cards, shown on the Ansage after the skat. */
function pressedCards(game: GameView): string {
  if (!game.skat?.length) return '';
  return `<p class="declare-pressed" data-testid="pressed"><span>${esc(de.table.skat)}</span>${game.skat.map(miniCard).join('')}</p>`;
}

/**
 * The first decision with ten cards (Skatordnung): pick up the skat, or
 * play a hand game. Einpassen and Schieben stay here as before.
 */
function choicePanel(options: DeclareOptions, passesLeft: number): string {
  const handPossible = options.null_risks.length ? options.null_risks.some((r) => r.allowed) : options.games.some((g) => g.allowed);
  return `<section class="declare declare-choice" aria-label="${esc(de.table.choiceTitle)}" data-testid="skat-choice">
    <h2 class="declare-title">${esc(de.table.choiceTitle)}</h2>
    <div class="choices">
      <button class="choice-btn is-skat" ${attrs({ 'data-action': 'pick-up-skat' })}>
        <span class="choice-cards" aria-hidden="true">${cardBack('mini')}${cardBack('mini')}</span>
        <b class="choice-name">${esc(de.table.pickUpSkat)}</b>
        <span class="choice-hint">${esc(de.table.pickUpSkatHint)}</span>
      </button>
      <button class="choice-btn is-hand" ${attrs({ 'data-action': 'play-hand', disabled: !handPossible })}>
        <span class="choice-cards is-hand" aria-hidden="true">${cardBack('mini')}${cardBack('mini')}</span>
        <b class="choice-name">${esc(de.table.playHand)}</b>
        <span class="choice-hint">${esc(de.table.playHandHint)}</span>
      </button>
    </div>
    ${declareWarnings(options, false)}
    ${declareActions({ ...options, can_pick_up_skat: false }, false, passesLeft, '')}
  </section>`;
}

/**
 * The Ansage of the Nullspieler: the Null levels the core offers in this
 * phase (Null Hand and Null Ouvert Hand before the skat, Null and Null
 * Ouvert after it) with their values from the core.
 */
function nullDeclarePanel(options: DeclareOptions, game: GameView, ui: UiState, passesLeft: number): string {
  const afterSkat = game.phase === 'DeclaringAfterSkat';
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
  const anyAllowed = options.null_risks.some((r) => r.allowed);
  const title = afterSkat ? de.table.nullTitle : de.table.nullHandTitle;
  const declare = `<button class="btn btn-primary" ${attrs({ 'data-action': 'declare-null', 'data-risk': risk, disabled: !anyAllowed })}>${esc(de.table.declare)}</button>`;
  return `<section class="declare declare-null" aria-label="${esc(title)}" data-testid="null-declare" data-phase="${esc(game.phase)}">
    <h2 class="declare-title">${esc(title)}</h2>
    ${pressedCards(game)}
    <p class="no-trick" role="note"><b>${esc(de.table.noTrick)}</b> <span>${esc(de.table.nullHint)}</span></p>
    <div class="null-risks">${risks}</div>
    <p class="declare-hint">${esc(de.nullRiskHints[risk])} ${esc(de.table.nullStyle)}</p>
    ${declareWarnings(options, afterSkat)}
    ${declareActions(options, afterSkat, passesLeft, declare)}
  </section>`;
}

/**
 * The Ansage: with ten cards first the choice of skat or hand, then the
 * hand Ansage; after the skat the Ansage without hand levels. Which levels
 * there are and which are allowed comes from the core.
 */
function declarePanel(options: DeclareOptions, game: GameView, ui: UiState, passesLeft: number): string {
  const afterSkat = game.phase === 'DeclaringAfterSkat';
  if (!afterSkat && options.can_pick_up_skat && !ui.handGame) return choicePanel(options, passesLeft);
  if (options.null_risks.length) return nullDeclarePanel(options, game, ui, passesLeft);
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
  // A single level (after the skat: only Normal) needs no row of buttons.
  const riskRow =
    options.risks.length > 1
      ? `<div class="declare-label">${esc(de.table.risk)}</div>
    <div class="risks" style="--n:${options.risks.length}">${risks}</div>`
      : '';
  const title = afterSkat ? de.table.afterSkatTitle : de.table.handTitle;
  const declare = `<button class="btn btn-primary" ${attrs({ 'data-action': 'declare', disabled: ui.pickGame === null })}>${esc(de.table.declare)}</button>`;
  return `<section class="declare ${afterSkat ? 'declare-after-skat' : 'declare-hand'}" aria-label="${esc(title)}" data-phase="${esc(game.phase)}">
    <h2 class="declare-title">${esc(title)}</h2>
    ${pressedCards(game)}
    <div class="declare-label">${esc(de.table.gameType)}</div>
    <div class="games">${games}</div>
    ${riskRow}
    <p class="declare-hint">${esc(de.riskHints[risk])}</p>
    ${declareWarnings(options, afterSkat)}
    ${declareActions(options, afterSkat, passesLeft, declare)}
  </section>`;
}

function handCards(game: GameView, ui: UiState): string {
  const hand = ui.anim ? ui.anim.hand : game.hand;
  // While the core works (and the opponents think) nothing in the hand is tappable.
  const idle = !ui.anim && !ui.busy;
  const discarding = idle && game.phase === 'Discarding' && !game.awaiting_kontra_answer;
  const oma = idle && ui.omaPick && game.can_take_from_skat;
  const playing = idle && game.your_turn && !oma;
  const cols = Math.max(5, Math.ceil(hand.length / 2));
  const cards = hand
    .map((c, i) => {
      const selected = ui.selected.includes(c.id);
      let action: string | null = null;
      // The core says which cards may go into the skat ("Ramponiert" may not).
      if (discarding) action = c.discardable ? 'select' : null;
      else if (oma) action = 'oma-give';
      else if (playing && c.legal) action = 'play';
      const dim = playing && !c.legal;
      const locked = discarding && !c.discardable;
      const cls = ['card', 'hand-card', dim ? 'is-dim' : '', selected ? 'is-selected' : '', action ? 'is-active' : '', locked ? 'is-locked' : ''].join(' ');
      const skatTag = (c.from_skat ? `<span class="skat-tag">${esc(de.card.fromSkat)}</span>` : '') + plaques(c);
      const label = cardLabel(c);
      // Slight arc per row, like cards held in the hand.
      const off = (i % cols) - (cols - 1) / 2;
      const style = `--tilt:${(off * 1.6).toFixed(1)}deg;--lift:${(off * off * 0.9).toFixed(1)}px`;
      if (action) {
        return `<button class="${cls}" style="${style}" ${attrs({
          'data-action': action,
          'data-id': c.id,
          'data-legal': c.legal ? 'true' : 'false',
          'data-no-discard': c.no_discard ? 'true' : null,
          'aria-pressed': action === 'select' ? (selected ? 'true' : 'false') : null,
          'aria-label': label,
        })}>${cardSvg(c.kind, c.mods)}${skatTag}</button>`;
      }
      return `<div class="${cls}" style="${style}" ${attrs({ 'data-id': c.id, 'aria-disabled': 'true', 'aria-label': label, role: 'img' })}>${cardSvg(c.kind, c.mods)}${skatTag}</div>`;
    })
    .join('');
  const open = isOpen(game);
  const banner = open ? `<p class="open-banner">${eyeIcon(16)}<span>${esc(de.table.openHand)}</span></p>` : '';
  return `${banner}<div class="hand ${open ? 'is-open' : ''} ${oma ? 'is-oma' : ''}" style="--cols:${cols}" data-testid="hand">${cards}</div>`;
}

/** Whether the player plays with open cards (Ouvert, also Null Ouvert). */
function isOpen(game: GameView): boolean {
  const risk = game.declaration?.risk;
  return game.phase !== 'Declaring' && game.phase !== 'DeclaringAfterSkat' && (risk === 'Ouvert' || risk === 'OuvertHand');
}

function statusLine(game: GameView, ui: UiState): string {
  if (ui.anim) {
    return `<div class="status" aria-live="polite">${esc(ui.anim.status)}</div>`;
  }
  switch (game.phase) {
    case 'Declaring':
      return `<div class="status">${esc(t(de.table.game, { number: game.number }))} · ${esc(de.positions[game.positions[0] ?? 'Forehand'])}</div>`;
    case 'DeclaringAfterSkat':
      return `<div class="status">${esc(t(de.table.game, { number: game.number }))} · ${esc(de.table.declareAfterSkat)}</div>`;
    case 'Discarding': {
      // Saves from before the Skatordnung order already hold the declaration here.
      const locked = game.hand.some((c) => c.no_discard) ? `<span class="discard-locked">${lockIcon(13)}${esc(de.table.discardLocked)}</span>` : '';
      return `<div class="status status-discard"><span>${esc(game.declaration ? de.table.discardHintDeclared : de.table.discardHint)}${locked}</span>
        <button class="btn btn-primary" ${attrs({ 'data-action': 'discard', disabled: ui.selected.length !== 2 })}>${esc(de.table.discard)}</button></div>`;
    }
    case 'Playing': {
      const swap = game.can_take_from_skat ? game.skat_swap_cards : [];
      const take = swap.find((c) => c.id === ui.swapTake) ?? swap[0];
      if (take && ui.omaPick) {
        const choices =
          swap.length > 1
            ? `<span class="swap-pick"><span>${esc(de.table.swapPick)}</span>${swap
                .map(
                  (c) =>
                    `<button class="mini-card swap-card ${c.id === take.id ? 'is-picked' : ''}" ${attrs({
                      'data-action': 'swap-take',
                      'data-id': c.id,
                      'aria-pressed': c.id === take.id ? 'true' : 'false',
                      'aria-label': cardName(c.kind),
                    })}>${cardSvg(c.kind, c.mods)}</button>`,
                )
                .join('')}</span>`
            : '';
        return `<div class="status status-oma"><span>${esc(t(de.table.swapHint, { card: cardName(take.kind) }))}</span>${choices}
          <button class="btn" ${attrs({ 'data-action': 'oma-cancel' })}>${esc(de.table.omaCancel)}</button></div>`;
      }
      if (take) {
        return `<div class="status ${game.your_turn ? 'is-your-turn' : ''}"><span>${esc(game.your_turn ? de.table.yourTurn : '')}</span>
          <button class="btn btn-oma" ${attrs({ 'data-action': 'oma', 'data-testid': 'skat-swap' })}><span class="mini-card">${cardSvg(take.kind, take.mods)}</span><span>${esc(t(de.table.swapTake, { card: cardName(take.kind) }))}</span></button></div>`;
      }
      return `<div class="status ${game.your_turn ? 'is-your-turn' : ''}">${esc(game.your_turn ? de.table.yourTurn : '')}</div>`;
    }
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
    <span class="tricks" data-tricks>${esc(
      d.game === 'Ramsch'
        ? t(de.table.seatTricks, { a: game.seat_tricks[0], b: game.seat_tricks[1], c: game.seat_tricks[2] })
        : t(de.table.tricks, { own: game.declarer_tricks, theirs: game.defender_tricks }),
    )}</span>
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
  // In a Ramsch everyone counts for himself: Augen per seat.
  const cells =
    game.declaration.game === 'Ramsch'
      ? cell(de.table.countYou, c.seats[0], 'is-you') + cell(de.table.countLeft, c.seats[1], 'is-them') + cell(de.table.countRight, c.seats[2], 'is-them')
      : cell(de.table.countYou, c.declarer, 'is-you') + cell(de.table.countThem, c.defenders, 'is-them');
  return `<div class="counter-slate" data-testid="count" aria-label="${esc(de.table.countTitle)}">
    <span class="counter-title">${esc(de.table.countTitle)}</span>
    ${cells}${cell(de.table.countUnseen, c.unseen, 'is-unseen')}
  </div>`;
}

/** The Ramsch: nobody declared, everyone plays for himself. */
function ramschBanner(game: GameView): string {
  if (game.declaration?.game !== 'Ramsch' || game.phase === 'Finished') return '';
  return `<div class="ramsch-banner" data-testid="ramsch">${portrait('p-boss-ramscher', 'portrait-xs')}<div><b>${esc(de.table.ramschTitle)}</b> <span>${esc(de.table.ramschText)}</span></div></div>`;
}

/** "Letzte Runde": this game counts more (shown while declaring). */
function pendingNote(view: RunView, game: GameView): string {
  if (!view.pending_factor || (game.phase !== 'Declaring' && game.phase !== 'DeclaringAfterSkat')) return '';
  return `<p class="pending-note">${esc(t(de.table.pendingFactor, { factor: view.pending_factor }))}</p>`;
}

export function renderTable(view: RunView, ui: UiState): string {
  const game = view.game;
  if (!game) return '';
  const declaring = (game.phase === 'Declaring' || game.phase === 'DeclaringAfterSkat') && !ui.anim && game.declare;
  const trick = ui.anim ? ui.anim.trick : game.trick;
  const center = declaring && game.declare
    ? declarePanel(game.declare, game, ui, view.series.passes_left)
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

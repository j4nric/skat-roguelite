// The table: the two regulars (or the boss) on top, the open trick on the
// felt between the pressed skat and the "Letzter Stich" button (which shows
// the core's last trick instead), the player's hand at the bottom in two rows. While declaring, the
// Ansage panel takes the middle (for the Nullspieler with the Null levels).
// Around it: the boss rule of this game, the Ramsch notice, the count of
// "Der Zähler", the Kiebitz card and the skat swap of "Die Oma" or "Der
// Opa", all from the view. Card flags the core resolved ("Unter
// ehrenhalber", "Ramponiert") show as plaques on the hand cards.

import { de, cardName, contentName, contentText, declarationName, gameName, t } from '../i18n';
import type { BossInfo, CardView, DeclaredGame, DeclareOptions, GameView, NullRisk, PlayedCard, RevealedCard, RunView, Seat, Suit, TrickView } from '../types';
import { cardBack, cardSvg, modIcon, suitIcon } from './cards';
import { lockIcon } from './icons';
import { attrs, esc } from './dom';
import { quipBubble, seatLabel, seatPerson } from './guests';
import { schnapsNotes } from './hud';
import { bossCoaster } from './emblems';
import { portrait } from './people';
import type { UiState } from './state';
import { BASE_WIN } from '../stake';

/** Marks a card for the info card (long-press, hover, "i"). */
function infoAttrs(id: number, focusable = true): Record<string, string | number | null> {
  return { 'data-info': `card:${id}`, tabindex: focusable ? 0 : null };
}

function miniCard(card: CardView): string {
  return `<span class="mini-card" ${attrs({ title: cardName(card.kind), ...infoAttrs(card.id) })}>${cardSvg(card.kind, card.mods)}</span>`;
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
  if (r.source === 'Onlooker') {
    return `<span class="mini-card is-onlooker" ${attrs({ title: `${cardName(card.kind)} (${de.table.onlookerHint})`, 'data-onlooker': card.id, ...infoAttrs(card.id, false) })}>${cardSvg(card.kind, card.mods)}</span>`;
  }
  if (r.source === 'Kibitz') {
    return `<span class="mini-card is-kiebitz" ${attrs({ title: `${cardName(card.kind)} (${de.table.kiebitzHint})`, 'data-kiebitz': card.id, ...infoAttrs(card.id) })}>${cardSvg(card.kind, card.mods)}</span>`;
  }
  const mod = r.source.CardMod;
  return `<span class="mini-card is-revealed" ${attrs({ title: `${cardName(card.kind)} (${contentName('card_mods', mod)})`, 'data-reveal-mod': mod, ...infoAttrs(card.id) })}>${cardSvg(card.kind, card.mods)}<span class="reveal-mod">${modIcon(mod, 14)}</span></span>`;
}

/** A small thought bubble while the core computes this opponent's card. */
function thinkBubble(): string {
  return `<span class="think-bubble" data-testid="think" role="status" aria-label="${esc(de.table.thinking)}"><i></i><i></i><i></i></span>`;
}

/** Plaques for what the core resolved on a card: "Unter ehrenhalber", "Ramponiert", "Kreide". */
function plaques(c: CardView): string {
  const out: string[] = [];
  if (c.chalked) out.push(`<span class="plaque plaque-chalk" title="${esc(chalkLabel(c.chalked))}" aria-hidden="true">${suitIcon(c.chalked, 11, null)}</span>`);
  if (c.honorary_jack) out.push(`<span class="plaque plaque-hj" title="${esc(de.card.honoraryJack)}" aria-hidden="true">U</span>`);
  if (c.no_discard) out.push(`<span class="plaque plaque-lock" title="${esc(de.card.noDiscard)}" aria-hidden="true">${lockIcon(13)}</span>`);
  return out.length ? `<span class="plaques">${out.join('')}</span>` : '';
}

/** "Kreide, eigentlich Rot". */
function chalkLabel(printed: Suit): string {
  return t(de.card.chalked, { suit: de.suits[printed] });
}

/** Card name plus what its plaques say, for screen readers. */
function cardLabel(c: CardView): string {
  const extra = [c.honorary_jack ? de.card.honoraryJack : '', c.no_discard ? de.card.noDiscard : '', c.chalked ? chalkLabel(c.chalked) : ''].filter(Boolean);
  const name = extra.length ? `${cardName(c.kind)} (${extra.join(', ')})` : cardName(c.kind);
  return c.mods.length ? `${name}, ${t(de.card.mods, { mods: c.mods.map((m) => contentName('card_mods', m)).join(', ') })}` : name;
}

function kontraBadge(game: GameView, seat: Seat): string {
  const k = game.kontra;
  if (typeof k === 'object' && (('Kontra' in k && k.Kontra === seat) || ('Re' in k && k.Re === seat))) {
    return `<span class="badge badge-kontra">${esc(de.table.kontraBadge)}</span>`;
  }
  return '';
}

function opponent(view: RunView, game: GameView, seat: 1 | 2, ui: UiState): string {
  const size = ui.anim ? (ui.anim.handSizes[seat] ?? 0) : (game.hand_sizes[seat] ?? 0);
  const turn = ui.anim ? ui.anim.turn === seat : view.ai_to_move === seat;
  const thinking = ui.thinking === seat;
  const seen = game.revealed.filter((r) => r.holder === seat);
  const open = seen.filter((r) => typeof r.source === 'object').map(revealedCard).join('');
  const peeked = seen.filter((r) => r.source === 'Kibitz').map(revealedCard).join('');
  // "Der Zaungast": the whole hand, as a tight fan in one row.
  const onlooker = seen.filter((r) => r.source === 'Onlooker').map(revealedCard).join('');
  const revealed =
    (onlooker
      ? `<span class="opp-open-hand" data-testid="onlooker-hand" ${attrs({ 'aria-label': t(de.table.onlookerAria, { count: seen.filter((r) => r.source === 'Onlooker').length }) })}><span class="opp-tag kiebitz-tag" title="${esc(de.table.onlooker)}">${eyeIcon(14)}</span>${onlooker}</span>`
      : '') +
    (open ? `<span class="opp-tag">${esc(de.table.holds)}</span>${open}` : '') +
    (peeked ? `<span class="opp-tag kiebitz-tag">${eyeIcon(14)}${esc(de.table.kiebitz)}</span>${peeked}` : '');
  const position = game.positions[seat];
  // Bosses sit in the left chair with their own figure and name, else the
  // Stammgast the core seated for this series.
  const person = seatPerson(view, seat);
  const boss = person.isBoss;
  const quip = ui.quip?.seat === seat ? quipBubble(ui.quip, person.name) : '';
  const tricks = game.phase === 'Playing' || game.phase === 'Finished' ? (game.seat_tricks[seat] ?? 0) : null;
  const trickTag =
    tricks === null
      ? ''
      : `<span class="opp-tricks" aria-label="${esc(t(de.table.oppTricksAria, { n: tricks }))}" data-testid="opp-tricks">${esc(t(de.table.oppTricks, { n: tricks }))}</span>`;
  return `<div class="opp ${turn ? 'is-turn' : ''} ${boss ? 'is-boss' : ''} ${thinking ? 'is-thinking' : ''}" ${attrs({ 'data-seat': seat, 'data-guest': person.guest?.id ?? null, 'data-face': person.face })}>
    <div class="opp-main" ${attrs({ 'data-info': `seat:${seat}`, tabindex: 0, role: 'button', 'aria-label': t(de.info.label, { name: person.name }) })}>
      ${portrait(person.face, 'portrait-opp')}${thinking ? thinkBubble() : ''}${quip}
      <div class="opp-info">
        <b class="opp-name" data-testid="opp-name">${esc(person.name)}</b>
        <span class="opp-pos">${esc(position ? de.positions[position] : '')}</span>
        <span class="opp-meta"><span class="opp-hand" aria-label="${esc(t(de.table.cards, { count: size }))}">${size ? cardBack('fan') : ''}<span class="opp-count">${esc(size)}</span></span>${trickTag}${kontraBadge(game, seat)}</span>
      </div>
    </div>
    ${revealed ? `<div class="opp-revealed">${revealed}</div>` : ''}
  </div>`;
}

/** One seat's place in the trick; `winner` marks the card that took a finished trick. */
function trickSlot(cards: PlayedCard[], seat: Seat, yourTurn = false, winner: Seat | null = null): string {
  const played = cards.find((c) => c.seat === seat);
  const won = winner === seat;
  const tag = won ? `<span class="trick-won-tag">${esc(de.table.lastTrickWon)}</span>` : '';
  const inner = played
    ? `<div class="card trick-card ${won ? 'is-winner' : ''}" ${attrs({ 'data-trick-seat': winner === null ? seat : null, 'data-last-seat': winner === null ? null : seat, ...infoAttrs(played.card.id) })}>${cardSvg(played.card.kind, played.card.mods)}${tag}</div>`
    : yourTurn
      ? `<div class="card-slot is-you">${esc(de.table.slotYou)}</div>`
      : `<div class="card-slot"></div>`;
  return `<div class="slot slot-${seat}">${inner}</div>`;
}

function gameIcon(game: DeclaredGame, size = 30): string {
  if (game === 'Null') return '<span class="null-icon" aria-hidden="true">0</span>';
  if (game === 'Grand' || game === 'Ramsch') {
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

/** The skat seen with "Kurzer", shown on the hand Ansage. */
function peekedCards(game: GameView): string {
  if (!game.skat_peek?.length || game.phase !== 'Declaring') return '';
  return `<p class="declare-pressed" data-testid="skat-peek-hand"><span>${esc(de.table.skatPeekShort)}</span>${game.skat_peek.map(miniCard).join('')}</p>`;
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
function choicePanel(options: DeclareOptions, passesLeft: number, peek: CardView[] | null | undefined): string {
  // Every Spielart (Null too) comes from the core with `allowed`.
  const handPossible = options.games.some((g) => g.allowed);
  // "Kurzer": the skat face up on the skat button, the choice stays open.
  const skatCards = peek?.length ? peek.map(miniCard).join('') : `${cardBack('mini')}${cardBack('mini')}`;
  return `<section class="declare declare-choice" aria-label="${esc(de.table.choiceTitle)}" data-testid="skat-choice">
    <h2 class="declare-title">${esc(de.table.choiceTitle)}</h2>
    <div class="choices">
      <button class="choice-btn is-skat" ${attrs({ 'data-action': 'pick-up-skat' })}>
        <span class="choice-cards ${peek?.length ? 'is-peeked' : ''}" ${attrs({ 'aria-hidden': peek?.length ? null : 'true', 'data-testid': peek?.length ? 'skat-peek' : null })}>${skatCards}</span>
        <b class="choice-name">${esc(de.table.pickUpSkat)}</b>
        <span class="choice-hint">${esc(de.table.pickUpSkatHint)}</span>
      </button>
      <button class="choice-btn is-hand" ${attrs({ 'data-action': 'play-hand', disabled: !handPossible })}>
        <span class="choice-cards is-hand" aria-hidden="true">${cardBack('mini')}${cardBack('mini')}</span>
        <b class="choice-name">${esc(de.table.playHand)}</b>
        <span class="choice-hint">${esc(de.table.playHandHint)}</span>
      </button>
    </div>
    ${peek?.length ? `<p class="declare-hint skat-peek-hint">${esc(t(de.table.skatPeek, { cards: peek.map((c) => cardName(c.kind)).join(', ') }))}</p>` : ''}
    ${declareWarnings(options, false)}
    ${declareActions({ ...options, can_pick_up_skat: false }, false, passesLeft, '')}
  </section>`;
}

/**
 * The Null levels of the Ansage (Null Hand and Null Ouvert Hand before the
 * skat, Null and Null Ouvert after it) with their values from the core.
 */
function nullRiskRow(options: DeclareOptions, risk: NullRisk): string {
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
  return `<div class="null-risks">${risks}</div>
    <p class="declare-hint no-trick" role="note"><b>${esc(de.table.noTrick)}</b> ${esc(de.nullRiskHints[risk])}${options.null_style ? ` <span class="null-style">${esc(de.table.nullStyle)}</span>` : ''}</p>`;
}

/**
 * The Ansage: with ten cards first the choice of skat or hand, then the
 * hand Ansage; after the skat the Ansage without hand levels. Every
 * character may declare every Spielart (ADR 0012); Null shows its own
 * levels with their values. The Nullspieler starts with Null picked.
 * Which levels there are and which are allowed comes from the core.
 */
function declarePanel(options: DeclareOptions, game: GameView, ui: UiState, passesLeft: number): string {
  const afterSkat = game.phase === 'DeclaringAfterSkat';
  if (!afterSkat && options.can_pick_up_skat && !ui.handGame) return choicePanel(options, passesLeft, game.skat_peek);
  const nullIndex = options.games.findIndex((g) => g.game === 'Null');
  const specialist = ui.view?.character.character === 'NullPlayer';
  const pick = ui.pickGame ?? (specialist && nullIndex >= 0 && options.games[nullIndex]?.allowed ? nullIndex : null);
  const pickedNull = pick !== null && pick === nullIndex;
  const games = options.games
    .map((g, i) => {
      const picked = pick === i;
      return `<button class="game-btn ${picked ? 'is-picked' : ''}" ${attrs({
        'data-action': 'pick-game',
        'data-index': i,
        'data-game': g.game === 'Null' ? 'Null' : null,
        disabled: !g.allowed,
        'aria-pressed': picked ? 'true' : 'false',
      })}>${gameIcon(g.game)}<span class="game-name">${esc(gameName(g.game))}</span><span class="game-base">${esc(g.base_value)}</span></button>`;
    })
    .join('');
  const nullRisk: NullRisk = ui.pickNullRisk ?? options.null_risks.find((r) => r.allowed)?.risk ?? 'Normal';
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
  const levels = pickedNull ? nullRiskRow(options, nullRisk) : `${riskRow}<p class="declare-hint">${esc(de.riskHints[risk])}</p>`;
  const title = afterSkat ? de.table.afterSkatTitle : de.table.handTitle;
  const declare = pickedNull
    ? `<button class="btn btn-primary" ${attrs({ 'data-action': 'declare-null', 'data-risk': nullRisk, disabled: !options.null_risks.some((r) => r.allowed) })}>${esc(de.table.declare)}</button>`
    : `<button class="btn btn-primary" ${attrs({ 'data-action': 'declare', disabled: pick === null })}>${esc(de.table.declare)}</button>`;
  return `<section class="declare ${afterSkat ? 'declare-after-skat' : 'declare-hand'}${pickedNull ? ' declare-null' : ''}" aria-label="${esc(title)}" data-phase="${esc(game.phase)}" ${attrs({ 'data-testid': pickedNull ? 'null-declare' : null })}>
    <h2 class="declare-title">${esc(title)}</h2>
    ${pressedCards(game)}${peekedCards(game)}
    <div class="declare-label">${esc(de.table.gameType)}</div>
    <div class="games">${games}</div>
    ${levels}
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
          ...infoAttrs(c.id, false),
        })}>${cardSvg(c.kind, c.mods)}${skatTag}</button>`;
      }
      return `<div class="${cls}" style="${style}" ${attrs({ 'data-id': c.id, 'aria-disabled': 'true', 'aria-label': label, role: 'img', ...infoAttrs(c.id) })}>${cardSvg(c.kind, c.mods)}${skatTag}</div>`;
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
                      ...infoAttrs(c.id, false),
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

/**
 * Grundwert × Gewinnstufen of the game in progress, as the core previews
 * it (only what is certain; factors such as Kontra appended). Nothing in
 * a Ramsch or before the declaration.
 */
function worthChip(game: GameView): string {
  const p = game.value_preview;
  if (!p) return '';
  const label =
    p.multiplier > 1
      ? t(de.table.worthFactor, { base: p.base_value, levels: p.levels, factor: p.multiplier })
      : t(de.table.worth, { base: p.base_value, levels: p.levels });
  return `<span class="worth" ${attrs({ 'data-testid': 'value-preview', title: t(de.table.worthTitle, { value: p.game_value }) })}>${esc(label)}</span>`;
}

/**
 * The Augen the game needs when they differ from 61: a stake of the
 * Einsatz-Leiter or a relic that lowers them. Suit games and Grand only;
 * Null and Ramsch count no Augen.
 */
function winFromChip(game: GameView): string {
  const d = game.declaration;
  if (!d || d.game === 'Null' || d.game === 'Ramsch' || game.win_threshold === BASE_WIN) return '';
  const augen = game.win_threshold;
  return `<span class="worth win-from" ${attrs({ 'data-testid': 'win-from', title: t(de.table.winFromTitle, { augen }) })}>${esc(t(de.table.winFrom, { augen }))}</span>`;
}

/**
 * The small line above the trick: what is played, Kontra, the tricks so far
 * with the value at stake. The score of the game only comes big at the end.
 * While the player looks at the last trick, its caption covers the line.
 */
function gameLine(view: RunView, game: GameView, peek: TrickView | null): string {
  if (!game.declaration) return '';
  const d = game.declaration;
  const kontra =
    game.kontra === 'None'
      ? ''
      : typeof game.kontra === 'object' && 'Re' in game.kontra
        ? `<span class="badge badge-kontra">${esc(de.table.kontraBadge)}</span><span class="badge badge-kontra">${esc(de.table.reBadge)}</span>`
        : `<span class="badge badge-kontra">${esc(de.table.kontraBadge)}</span>`;
  const special = d.game === 'Null' ? `<span class="badge badge-null">${esc(de.table.noTrick)}</span>` : '';
  return `<div class="gameline ${peek ? 'is-peek' : ''}">
    <span class="decl">${gameIcon(d.game, 14)}<span>${esc(declarationName(d))}</span></span>
    ${special}${kontra}
    <span class="tricks" data-tricks>${esc(
      d.game === 'Ramsch'
        ? t(de.table.seatTricks, { a: game.seat_tricks[0], b: game.seat_tricks[1], c: game.seat_tricks[2] })
        : t(de.table.tricks, { own: game.declarer_tricks, theirs: game.defender_tricks }),
    )}</span>${worthChip(game)}${winFromChip(game)}
    ${peek ? `<span class="peek-caption" data-testid="last-trick-caption">${esc(peek.winner === 0 ? de.table.lastTrickToYou : t(de.table.lastTrickTo, { name: seatLabel(view, peek.winner) }))}</span>` : ''}
  </div>`;
}

/**
 * The two pressed cards beside the trick, fanned so both corner indexes
 * show, big enough to read on a phone. A tap opens the card large (the
 * info card).
 */
function pressedStack(game: GameView): string {
  const skat = game.skat;
  if (!skat?.length) return '';
  const cards = skat
    .map(
      (c) =>
        `<button class="pressed-card" ${attrs({ 'data-action': 'info-card', 'aria-label': cardName(c.kind), ...infoAttrs(c.id, false) })}>${cardSvg(c.kind, c.mods)}</button>`,
    )
    .join('');
  const label = t(de.table.pressedAria, { cards: skat.map((c) => cardName(c.kind)).join(', ') });
  return `<div class="pressed-stack" role="group" ${attrs({ 'aria-label': label, 'data-testid': 'pressed-play' })}><span class="pressed-label" aria-hidden="true">${esc(de.table.skat)}</span><span class="pressed-cards">${cards}</span></div>`;
}

/** Small stack of three cards: the sign of "Letzter Stich". */
function lastTrickIcon(): string {
  return `<svg class="last-trick-icon" viewBox="0 0 20 16" width="20" height="16" aria-hidden="true" shape-rendering="crispEdges"><path d="M1 3h7v11h-7z" fill="#5d666d"/><path d="M6 2h7v11h-7z" fill="#8b9399"/><path d="M11 1h7v11h-7z" fill="#ecebe4"/><path d="M11 1h7v1h-7zM11 11h7v1h-7zM11 1h1v11h-1zM17 1h1v11h-1z" fill="#0b0c0e"/></svg>`;
}

/**
 * "Letzter Stich" beside the trick: shows the last finished trick (the
 * core's `last_trick`) on the felt until tapped again. Off while cards fly.
 */
function lastTrickToggle(game: GameView, ui: UiState): string {
  if (game.phase !== 'Playing' || !game.last_trick) return '';
  const on = ui.peekTrick && !ui.anim && !ui.busy;
  return `<button class="last-trick-btn ${on ? 'is-on' : ''}" ${attrs({
    'data-action': 'last-trick',
    'data-testid': 'last-trick-toggle',
    'aria-pressed': on ? 'true' : 'false',
    disabled: Boolean(ui.anim || ui.busy),
  })}>${lastTrickIcon()}<span>${esc(de.table.lastTrick)}</span></button>`;
}

/**
 * The felt: the pressed skat on the left, the trick in the middle (or the
 * last trick while the player looks at it), the "Letzter Stich" button on
 * the right. The sides take only the room the trick leaves.
 */
function trickRow(game: GameView, ui: UiState, peek: TrickView | null): string {
  const trick = peek
    ? `<div class="trick is-last" ${attrs({ 'data-action': 'last-trick', 'data-testid': 'last-trick', title: de.table.lastTrickClose })}>${trickSlot(peek.cards, 1, false, peek.winner)}${trickSlot(peek.cards, 0, false, peek.winner)}${trickSlot(peek.cards, 2, false, peek.winner)}</div>`
    : (() => {
        const open = ui.anim ? ui.anim.trick : game.trick;
        return `<div class="trick" data-action="fast">${trickSlot(open, 1)}${trickSlot(open, 0, !ui.anim && !ui.busy && game.your_turn)}${trickSlot(open, 2)}</div>`;
      })();
  return `<div class="trick-row"><div class="trick-side is-left">${pressedStack(game)}</div>${trick}<div class="trick-side is-right">${lastTrickToggle(game, ui)}</div></div>`;
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
  return `<button class="rule-strip ${laidOn ? 'is-laid-on' : ''}" ${attrs({ 'data-action': 'info-rule', 'data-testid': 'boss-rule', 'data-info': `rule:${rule.id}` })}>
    ${bossCoaster(laidOn ? boss.id : rule.id, { size: 34 })}
    <span class="rule-head">${esc(head)}</span>
    <b class="rule-name">${esc(contentName('bosses', rule.id))}</b>
    <span class="rule-text">${esc(contentText('bosses', rule.id) || rule.text)}</span>
  </button>`;
}

/** The count of "Der Zähler": one small row under the trick. */
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
  return `<div class="ramsch-banner" data-testid="ramsch">${bossCoaster('ramscher', { size: 34 })}<div><b>${esc(de.table.ramschTitle)}</b> <span>${esc(de.table.ramschText)}</span></div></div>`;
}

/** "Letzte Runde": this game counts more (shown while declaring). */
function pendingNote(view: RunView, game: GameView): string {
  if (game.phase !== 'Declaring' && game.phase !== 'DeclaringAfterSkat') return '';
  const notes = [view.pending_factor ? t(de.table.pendingFactor, { factor: view.pending_factor }) : '', ...schnapsNotes(view, de.table)].filter(Boolean);
  return notes.length ? `<p class="pending-note">${esc(notes.join(' · '))}</p>` : '';
}

export function renderTable(view: RunView, ui: UiState): string {
  const game = view.game;
  if (!game) return '';
  const declaring = (game.phase === 'Declaring' || game.phase === 'DeclaringAfterSkat') && !ui.anim && game.declare;
  const peek = ui.peekTrick && !ui.anim && !ui.busy && game.phase === 'Playing' ? game.last_trick : null;
  const center =
    declaring && game.declare
      ? declarePanel(game.declare, game, ui, view.series.passes_left)
      : `${gameLine(view, game, peek)}${trickRow(game, ui, peek)}${countPanel(game)}`;
  const ramsch = game.declaration?.game === 'Ramsch';
  const cols = Math.max(5, Math.ceil((ui.anim ? ui.anim.hand : game.hand).length / 2));
  return `<div class="table phase-${game.phase.toLowerCase()} ${ramsch ? 'is-ramsch' : ''} ${declaring ? 'is-declaring' : ''}" style="--cols:${cols}">
    <div class="opponents">${opponent(view, game, 1, ui)}${opponent(view, game, 2, ui)}</div>
    ${ruleStrip(view, game)}${ramschBanner(game)}${pendingNote(view, game)}
    <div class="center"><div class="deck-spot" aria-hidden="true"></div>${center}</div>
    ${handCards(game, ui)}
    ${statusLine(game, ui)}
  </div>`;
}

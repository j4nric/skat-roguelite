// Deal odds from the core (DESIGN 7.4): where a card of the shared deck
// lands. Only formatting; the numbers come from RunView.deal_odds.

import { de, percent, t } from '../i18n';
import type { DealOdds } from '../types';
import { esc } from './dom';

/** One line: chance to end up with the player (hand or Skat). */
export function oddsLine(odds: DealOdds): string {
  return `<p class="odds">${esc(t(de.odds.line, { pct: percent(odds.hand_or_skat.permille) }))}</p>`;
}

/** The full split: hand, Skat, defenders, talon. */
export function oddsDetail(odds: DealOdds): string {
  const talon = odds.talon.numerator ? t(de.odds.talon, { talon: percent(odds.talon.permille) }) : '';
  const text = t(de.odds.detail, {
    hand: percent(odds.hand.permille),
    skat: percent(odds.skat.permille),
    defenders: percent(odds.defenders.permille),
  });
  return `<p class="odds">${esc(text + talon)}</p>`;
}

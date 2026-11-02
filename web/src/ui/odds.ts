// Deal odds from the core (DESIGN 7.4): where a card lands. Since ADR 0015
// always 12 of 32 for the player; with a defender the card plays in his
// edition. Only formatting; the numbers come from RunView.deal_odds.

import { de, percent, t } from '../i18n';
import type { DealOdds } from '../types';
import { esc } from './dom';

/** One line: chance to end up with the player (hand or Skat). */
export function oddsLine(odds: DealOdds): string {
  return `<p class="odds">${esc(t(de.odds.line, { pct: percent(odds.hand_or_skat.permille) }))}</p>`;
}

/** The full split: hand, Skat, defenders. */
export function oddsDetail(odds: DealOdds): string {
  const text = t(de.odds.detail, {
    hand: percent(odds.hand.permille),
    skat: percent(odds.skat.permille),
    defenders: percent(odds.defenders.permille),
  });
  return `<p class="odds">${esc(text)}</p>`;
}

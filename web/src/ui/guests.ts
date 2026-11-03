// Who sits on an opponent's seat: the Stammgast the core seated for this
// series (view.series.guests, name and lines from core/content/guests.ron),
// or in a boss pub the boss's two figures, one per seat (ADR 0017). And the small speech
// bubble of a Stammgast (a quip). Pure presentation, no game logic.

import { contentName, de } from '../i18n';
import type { GuestInfo, RunView, Seat } from '../types';
import { attrs, esc } from './dom';
import { bossPortraitId, guestPortraitId } from './people';

export interface SeatPerson {
  /** Portrait symbol id. */
  face: string;
  name: string;
  isBoss: boolean;
  /** The Stammgast on this seat, if it is one. */
  guest: GuestInfo | null;
}

/** The person on an opponent's seat (1 or 2) in the current series. */
export function seatPerson(view: RunView, seat: 1 | 2): SeatPerson {
  // ADR 0017: both seats of a boss pub are the boss's two figures.
  const boss = view.series.is_boss ? view.series.boss : null;
  if (boss) {
    const figure = seat - 1;
    const name = boss.figures?.[figure]?.name ?? contentName('bosses', boss.id);
    return { face: bossPortraitId(boss.id, figure), name, isBoss: true, guest: null };
  }
  // Views from before the Stammgäste have no guests: the old anonymous seat.
  const guest = (view.series.guests ?? []).find((g) => g.seat === seat) ?? null;
  if (!guest) return { face: `p-opp-${seat}`, name: de.seats[seat] ?? '', isBoss: false, guest: null };
  return { face: guestPortraitId(guest.look, seat), name: guest.name, isBoss: false, guest };
}

/** Display name of any seat: "Du" for the player. */
export function seatLabel(view: RunView, seat: Seat): string {
  return seat === 0 ? (de.seats[0] ?? '') : seatPerson(view, seat).name;
}

/** A quip on screen: who says what. */
export interface Quip {
  seat: 1 | 2;
  text: string;
}

/**
 * The tiny speech bubble under an opponent. Decorative and not
 * interactive (pointer-events: none in CSS), so it never blocks a tap;
 * screen readers get it as a polite status.
 */
export function quipBubble(quip: Quip, name: string): string {
  return `<span class="quip quip-${quip.seat}" ${attrs({ 'data-testid': 'quip', role: 'status', 'aria-label': `${name}: ${quip.text}` })}>${esc(quip.text)}</span>`;
}

// The lines (Sprüche) of the people at the table, from the core content:
// the Stammgäste (view.series.guests[].quips, core/content/guests.ron) and
// the two figures of a boss (boss.figures[].quips, bosses.ron), several
// variants per situation (DESIGN 10.4). Which variant shows is decoration:
// Math.random here, never the core's generator, so no game changes.

import { quote } from '../i18n';
import type { BossInfo, Quips, QuipSituation, RunView } from '../types';
import { esc } from './dom';
import { seatPerson } from './guests';

/** Who speaks on an opponent's seat and what he has to say. */
export interface Speaker {
  seat: 1 | 2;
  name: string;
  quips: Quips;
}

/** The person on a seat with his lines: a Stammgast, or a boss figure in a boss pub. */
export function speakerAt(view: RunView, seat: 1 | 2): Speaker | null {
  const person = seatPerson(view, seat);
  if (person.guest?.quips) return { seat, name: person.name, quips: person.guest.quips };
  const boss = view.series.is_boss ? view.series.boss : null;
  const quips = boss?.figures?.[seat - 1]?.quips;
  return quips ? { seat, name: person.name, quips } : null;
}

/** The lines of a situation (none in a view from before the lines existed). */
export function linesOf(quips: Quips | null | undefined, situation: QuipSituation): string[] {
  return quips?.[situation] ?? [];
}

/**
 * A Stammgast's greeting of this series: the same variant for the bubble
 * and the words on the felt, so both say the same. Chosen by the series
 * and the seat (deterministic, no random stream).
 */
export function greetingOf(view: RunView, seat: 1 | 2): string {
  const lines = linesOf(speakerAt(view, seat)?.quips, 'greet');
  if (!lines.length) return '';
  const n = view.series.section * 7 + view.series.index * 3 + seat;
  return lines[n % lines.length] ?? '';
}

/** Last variant shown per person and situation, so the next one differs. */
const lastShown = new Map<string, number>();

/** Index of a variant, not the one this person said last time here. */
function pickIndex(count: number, key: string): number {
  const last = lastShown.get(key);
  let i = Math.floor(Math.random() * count);
  if (count > 1 && i === last) i = (i + 1) % count;
  lastShown.set(key, i);
  return i;
}

/** A variant of the lines, not the one this person said last time here. */
export function pickLine(lines: string[], key: string): string {
  return lines.length ? (lines[pickIndex(lines.length, key)] ?? '') : '';
}

/** Variants picked for a screen, kept while it is drawn again and again. */
const stable = new Map<string, number>();

/**
 * A boss figure's line for a screen that is drawn many times (the boss
 * sheet, the reward, the end): picked once per `occasion`, then the same
 * variant (also after a switch of the language).
 */
export function bossQuipLine(boss: BossInfo, figure: 0 | 1, situation: QuipSituation, occasion: string): { name: string; text: string } | null {
  const f = boss.figures?.[figure];
  const lines = linesOf(f?.quips, situation);
  if (!f || !lines.length) return null;
  const key = `${boss.id}:${figure}:${situation}:${occasion}`;
  let i = stable.get(key);
  if (i === undefined) {
    i = pickIndex(lines.length, `${boss.id}:${figure}:${situation}`);
    stable.set(key, i);
  }
  return { name: f.name, text: lines[i % lines.length] ?? '' };
}

/** When a boss figure has its say on a screen: the pub it is about (its line stays while the screen is drawn again). */
export function occasionOf(view: RunView | null): string {
  return view ? `${view.seed}-${view.series.section}-${view.series.index}` : '';
}

/** A boss figure's line on a screen (the boss sheet, the reward, the end), written out like the table talk. */
export function bossQuip(line: { name: string; text: string } | null): string {
  if (!line) return '';
  return `<p class="boss-quip" data-testid="boss-quip"><b>${esc(line.name)}:</b> ${esc(quote(line.text))}</p>`;
}

// Small talk at the table (DESIGN 10.4): when a Stammgast or a boss figure
// says one of his lines in the small bubble at his seat. The lines come
// from the core content (../ui/quips.ts); when and which variant is
// decoration and may use Math.random, it changes nothing in the game.
//
// Kept rare and readable: one bubble at a time, never while the Wirt
// explains something, gone after a few seconds (reading time, scaled by
// the tempo setting, not by reduced motion). After a bubble the table is
// quiet for a while, counted in tricks; the hello of a new series always
// gets through.

import { playQuipSound } from '../audio';
import { TEMPO_FACTOR } from '../settings';
import type { CompletedTrick, Declaration, QuipSituation, RunView, ScoreBreakdown, Seat } from '../types';
import { quipBubble } from '../ui/guests';
import { greetingOf, linesOf, pickLine, speakerAt, type Speaker } from '../ui/quips';
import type { AppCtx } from './context';
import type { Guidance } from './guidance';

/** How long a bubble stays at tempo "Normal" (reading time, not motion). */
const QUIP_MS = 2800;
/** Never shorter than this, at any tempo. */
const QUIP_MIN_MS = 1800;
/** A trick worth this many Augen or more gets a word (the bar of flights.ts too). */
const TRICK_POINTS = 10;

/**
 * Tricks of quiet after a bubble before the next one, per occasion, and
 * how likely a line is when the occasion comes. About one bubble a game:
 * a trick needs 14 quiet tricks (as before, roughly every second game),
 * an Ansage or a result 7, a Schneider only 3.
 */
const RULES = {
  trick: { quiet: 14, chance: 1 },
  smear: { quiet: 9, chance: 1 },
  declare: { quiet: 7, chance: 0.6 },
  result: { quiet: 7, chance: 0.45 },
  schneider: { quiet: 3, chance: 1 },
} as const;

type Occasion = keyof typeof RULES;

/** The two opponents' seats. */
const SEATS = [1, 2] as const;

export class TableTalk {
  /** Series ("section-index") whose hello was said. */
  greeted = '';
  /** Series whose first Ramsch got a word. */
  private ramschSeries = '';
  /** Tricks since the last bubble (starts open: the first occasion may speak). */
  private quiet = 99;
  private timer: number | undefined;

  constructor(
    private readonly ctx: AppCtx,
    private readonly guidance: Guidance,
  ) {}

  /** Hello when a new series sits down at the table (a boss pub: the right figure, the left spoke on the boss sheet). */
  greetSeries(view: RunView): void {
    const key = seriesKey(view);
    if (key === this.greeted) return;
    this.greeted = key;
    // The two take turns per series.
    const seat = view.series.is_boss ? 2 : ((((view.series.section * 3 + view.series.index) % 2) + 1) as 1 | 2);
    this.say(seat, greetingOf(view, seat));
  }

  /**
   * A trick is done: now and then a word. The partner who dropped a Daus
   * or a Zehn on it (schmieren) says so, else the winner or the one beaten
   * after a trick worth something.
   */
  trick(view: RunView, trick: CompletedTrick, points: number): void {
    this.quiet += 1;
    const ramsch = view.game?.declaration?.game === 'Ramsch';
    // A Stammgast's trick lines are for a game against the player, not for a Ramsch.
    if (ramsch && !view.series.is_boss) return;
    const smearer = ramsch ? null : smearedBy(trick);
    if (smearer && this.allowed('smear')) {
      const speaker = speakerAt(view, smearer);
      if (speaker && this.sayLine(speaker, 'smear')) return;
    }
    if (points < TRICK_POINTS || !this.allowed('trick')) return;
    const winner = trick.winner;
    if (winner !== 0) {
      const speaker = speakerAt(view, winner);
      if (speaker) this.sayLine(speaker, 'win');
      return;
    }
    const seat = ((this.quiet % 2) + 1) as 1 | 2;
    const speaker = speakerAt(view, seat) ?? speakerAt(view, other(seat));
    if (speaker) this.sayLine(speaker, 'lose');
  }

  /** The player declared (or a Ramsch began): a Grand, a Null or a bold Ansage gets a word. */
  declared(view: RunView, d: Declaration): void {
    const situation = declareSituation(d);
    if (!situation) return;
    if (situation === 'ramsch') {
      // Only the first Ramsch of a series.
      const key = seriesKey(view);
      if (key === this.ramschSeries) return;
      this.ramschSeries = key;
    }
    if (!this.allowed('declare')) return;
    this.sayOne(view, situation);
  }

  /**
   * The score board went and the table is back: a word on the game just
   * played. Not when the pub ended with it (the bosses have their say on
   * the reward and end screens).
   */
  scored(view: RunView, b: ScoreBreakdown): void {
    if (view.phase !== 'Game' || this.ctx.ui.screen !== 'run') return;
    const schneider = b.won && (b.schneider || b.schwarz) && b.declaration.game !== 'Ramsch';
    if (!this.allowed(schneider ? 'schneider' : 'result')) return;
    if (schneider && this.sayOne(view, 'schneider', true)) return;
    this.sayOne(view, b.won ? 'lost' : 'won', true);
  }

  /** The bubble goes now (a new game, the Wirt speaks). */
  hush(): void {
    window.clearTimeout(this.timer);
    this.guidance.hushQuip();
  }

  /** Whether the table has been quiet long enough for this occasion, and the dice say yes. */
  private allowed(occasion: Occasion): boolean {
    const rule = RULES[occasion];
    return this.quiet >= rule.quiet && (rule.chance >= 1 || Math.random() < rule.chance);
  }

  /** One of the two with lines for the situation says one. */
  private sayOne(view: RunView, situation: QuipSituation, paint = false): boolean {
    const speakers = SEATS.map((s) => speakerAt(view, s)).filter((s): s is Speaker => !!s && linesOf(s.quips, situation).length > 0);
    if (!speakers.length) return false;
    const speaker = speakers[Math.floor(Math.random() * speakers.length)] ?? speakers[0];
    return speaker ? this.sayLine(speaker, situation, paint) : false;
  }

  private sayLine(speaker: Speaker, situation: QuipSituation, paint = false): boolean {
    const text = pickLine(linesOf(speaker.quips, situation), `${speaker.name}:${situation}`);
    return this.say(speaker.seat, text, paint ? speaker.name : null);
  }

  /**
   * Show a line at a seat. The caller renders the table next; with
   * `paint` (the name of the speaker) the bubble goes straight into the
   * box, for when no render follows.
   */
  private say(seat: 1 | 2, text: string, paint: string | null = null): boolean {
    if (!text || this.guidance.hint) return false;
    const ui = this.ctx.ui;
    ui.quip = { seat, text };
    playQuipSound(seat);
    this.quiet = 0;
    window.clearTimeout(this.timer);
    const ms = Math.max(QUIP_MIN_MS, QUIP_MS * TEMPO_FACTOR[ui.settings.tempo]);
    this.timer = window.setTimeout(() => this.guidance.hushQuip(), ms);
    if (paint !== null) {
      this.ctx.screenEl.querySelector('[data-testid="quip"]')?.remove();
      const info = this.ctx.screenEl.querySelector(`.opp[data-seat="${seat}"] .opp-info`);
      info?.insertAdjacentHTML('beforebegin', quipBubble(ui.quip, paint));
    }
    return true;
  }
}

function seriesKey(view: RunView): string {
  return `${view.series.section}-${view.series.index}`;
}

function other(seat: 1 | 2): 1 | 2 {
  return seat === 1 ? 2 : 1;
}

/** What an Ansage is to the regulars: Null, Grand, a bold Risikostufe, a Ramsch, or nothing to talk about. */
export function declareSituation(d: Declaration): QuipSituation | null {
  if (d.game === 'Ramsch') return 'ramsch';
  if (d.game === 'Null') return 'null';
  if (d.risk !== 'Normal') return 'bold';
  if (d.game === 'Grand') return 'grand';
  return null;
}

/**
 * The defender who smeared this trick (schmieren): his partner took it and
 * he dropped a Daus or a Zehn on it. Only between the two opponents.
 */
export function smearedBy(trick: CompletedTrick): 1 | 2 | null {
  if (trick.winner === 0) return null;
  const partner = other(trick.winner as 1 | 2);
  const full = trick.cards.some((card, i) => {
    const seat = ((trick.lead + i) % 3) as Seat;
    if (seat !== partner || typeof card.kind !== 'object' || !('Normal' in card.kind)) return false;
    const rank = card.kind.Normal[1];
    return rank === 'A' || rank === 'T';
  });
  return full ? partner : null;
}

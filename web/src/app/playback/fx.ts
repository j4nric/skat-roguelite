// Timing and small talk of the playback: pauses the player can hurry,
// the toast line, the Stammgäste's quips and the lines and wiggles of
// relics and modifications that fire. Pure presentation: every fact comes
// from the core's events and views.

import { de, relicCounters, t } from '../../i18n';
import { TEMPO_FACTOR } from '../../settings';
import type { RunView, ScoreLine, Seat, Source } from '../../types';
import { esc, reducedMotion } from '../../ui/dom';
import { finishAll } from '../../ui/fx';
import { seatPerson } from '../../ui/guests';
import { changeText, sourceName } from '../../ui/score';
import { targetsOf, type Trigger } from '../../ui/triggers';
import { DELAY, type ToastPlace } from '../consts';
import type { AppCtx } from '../context';
import type { Guidance } from '../guidance';

/** Toasts waiting at most; older ones are dropped. */
const TOAST_QUEUE = 3;
/** How long a Stammgast's bubble stays at tempo "Normal" (reading time, not motion). */
const QUIP_MS = 2800;
/** At least this many tricks between two quips: about one every second game. */
const QUIP_EVERY_TRICKS = 14;

export class Fx {
  private wakers: (() => void)[] = [];
  private readonly toastEl: HTMLElement;
  private toastTimer: number | undefined;
  private toasts: { text: string; place: ToastPlace }[] = [];
  /** Series ("section-index") whose Stammgast already said hello. */
  quipSeries = '';
  /** Tricks since the last quip: quips stay rare. */
  private tricksSinceQuip = 0;
  private quipTimer: number | undefined;

  constructor(
    private readonly ctx: AppCtx,
    private readonly guidance: Guidance,
  ) {
    this.toastEl = ctx.root.querySelector<HTMLElement>('.toast-layer') ?? ctx.root;
  }

  /** Tempo factor of the settings, for effects outside ../../ui/fx.ts. */
  tempo(): number {
    return TEMPO_FACTOR[this.ctx.ui.settings.tempo];
  }

  /** Wait, unless the player asked to hurry. */
  pause(ms: number): Promise<void> {
    if (this.ctx.fastForward) return Promise.resolve();
    const scaled = ms * TEMPO_FACTOR[this.ctx.ui.settings.tempo] * (reducedMotion() ? 0.25 : 1);
    return new Promise((resolve) => {
      const timer = window.setTimeout(done, scaled);
      const wakers = this.wakers;
      function done(): void {
        window.clearTimeout(timer);
        const i = wakers.indexOf(done);
        if (i >= 0) wakers.splice(i, 1);
        resolve();
      }
      wakers.push(done);
    });
  }

  hurry(): void {
    this.ctx.fastForward = true;
    finishAll();
    for (const wake of this.wakers.slice()) wake();
  }

  /**
   * A short message; while one is up, the next ones wait their turn. `top`:
   * above the Stammgäste, for messages while cards lie on the felt
   * (Drücken, the Ansage, the trick) that must stay readable.
   */
  showToast(text: string, place: ToastPlace = 'middle'): void {
    if (this.toastTimer !== undefined) {
      this.toasts.push({ text, place });
      if (this.toasts.length > TOAST_QUEUE) this.toasts.shift();
      return;
    }
    this.toastEl.innerHTML = `<div class="toast">${esc(text)}</div>`;
    this.toastEl.classList.toggle('is-top', place === 'top');
    this.toastEl.style.top = '';
    if (place === 'top') {
      // The toast's bottom edge sits just above the opponents' row.
      const opps = this.ctx.screenEl.querySelector('.opponents')?.getBoundingClientRect();
      const height = this.toastEl.querySelector('.toast')?.getBoundingClientRect().height ?? 0;
      if (opps) this.toastEl.style.top = `${Math.round(Math.max(height + 4, opps.top - 4))}px`;
    }
    // Shorter while others wait.
    const ms = this.toasts.length ? DELAY.toast * 0.7 : DELAY.toast;
    this.toastTimer = window.setTimeout(() => {
      this.toastTimer = undefined;
      this.toastEl.innerHTML = '';
      const next = this.toasts.shift();
      if (next !== undefined) this.showToast(next.text, next.place);
    }, ms);
  }

  /**
   * A Stammgast says one of his lines (from the core content) in a small
   * bubble at his seat: hello at the first game of a series, and now and
   * then after a trick worth something. One bubble at a time, never while
   * the Wirt explains something, gone after a few seconds (tempo setting).
   * Pure decoration: it changes nothing and catches no taps.
   */
  private sayQuip(view: RunView, seat: 1 | 2, line: 'greet' | 'win' | 'lose'): void {
    const guest = seatPerson(view, seat).guest;
    if (!guest || this.guidance.hint) return;
    const ui = this.ctx.ui;
    ui.quip = { seat, text: guest[line] };
    this.tricksSinceQuip = 0;
    window.clearTimeout(this.quipTimer);
    const ms = Math.max(1800, QUIP_MS * TEMPO_FACTOR[ui.settings.tempo]);
    this.quipTimer = window.setTimeout(() => this.guidance.hushQuip(), ms);
  }

  /** Hello from a Stammgast when a new series sits down at the table. */
  greetSeries(view: RunView): void {
    const key = `${view.series.section}-${view.series.index}`;
    if (key === this.quipSeries) return;
    this.quipSeries = key;
    // The boss on the left has no lines; else the two take turns per series.
    const seat = view.series.is_boss ? 2 : (((view.series.section * 3 + view.series.index) % 2) + 1) as 1 | 2;
    this.sayQuip(view, seat, 'greet');
  }

  /** Now and then a line after a trick worth something (Augen from the core). */
  trickQuip(view: RunView, winner: Seat, points: number): void {
    this.tricksSinceQuip += 1;
    if (this.tricksSinceQuip < QUIP_EVERY_TRICKS || points < 10) return;
    if (winner !== 0) {
      this.sayQuip(view, winner, 'win');
      return;
    }
    const seat = view.series.is_boss ? 2 : ((this.tricksSinceQuip % 2) + 1) as 1 | 2;
    this.sayQuip(view, seat, 'lose');
  }

  /** What stands for a source on screen (its relic chip, its card). */
  triggerTargets(source: Source): Element[] {
    return targetsOf(this.ctx.screenEl, source);
  }

  /** Wiggle what stands for a source on screen (its relic chip, its card). */
  wiggleSource(source: Source): void {
    if (this.ctx.fastForward) return;
    this.ctx.triggers.fire([{ source, text: '' }], this.ctx.screenEl, this.tempo(), false);
  }

  /**
   * Relics and card modifications whose score lines are in the result: wiggle
   * them and say what they did. Returns whether anything fired.
   */
  fireTriggers(lines: ScoreLine[], after: RunView): boolean {
    const fired: Trigger[] = lines
      .filter((l) => typeof l.source === 'object' && ('Relic' in l.source || 'CardMod' in l.source))
      .map((l) => ({
        source: l.source,
        text: t(de.trigger.line, { source: sourceName(l.source, after.edition), effect: changeText(l.lever, l.change) }),
      }));
    if (!fired.length || this.ctx.fastForward) return false;
    this.positionTriggers();
    this.ctx.triggers.fire(fired, this.ctx.screenEl, this.tempo());
    return true;
  }

  /** A run counter went up: the relics that count it wiggle and say the new value. */
  counterRaised(counter: string, value: number, after: RunView): void {
    const template = de.counters[counter];
    if (!template || this.ctx.fastForward) return;
    const owners = after.relics.filter((r) => relicCounters(r.relic.id).includes(counter));
    if (!owners.length) return;
    this.positionTriggers();
    this.ctx.triggers.fire(
      owners.map((r) => ({ source: { Relic: r.relic.id }, text: t(template, { value }) })),
      this.ctx.screenEl,
      this.tempo(),
    );
  }

  /** The trigger lines sit on the felt above the trick, else right below the head of the run. */
  private positionTriggers(): void {
    const felt = this.ctx.screenEl.querySelector('.table .center')?.getBoundingClientRect();
    const hud = this.ctx.screenEl.querySelector('.hud')?.getBoundingClientRect();
    this.ctx.triggers.place(felt ? felt.top + 8 : hud ? hud.bottom + 6 : null);
  }
}

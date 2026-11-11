// Timing and small talk of the playback: pauses the player can hurry,
// the toast line, the quips of the people at the table (../talk.ts) and
// the lines and wiggles of relics and modifications that fire. Pure presentation: every fact comes
// from the core's events and views.

import { de, relicCounters, t } from '../../i18n';
import { TEMPO_FACTOR } from '../../settings';
import type { CompletedTrick, Declaration, RunView, ScoreBreakdown, ScoreLine, Source } from '../../types';
import { esc, reducedMotion } from '../../ui/dom';
import { finishAll } from '../../ui/fx';
import { changeText, sourceName } from '../../ui/score';
import { targetsOf, type Trigger } from '../../ui/triggers';
import { DELAY, type ToastPlace } from '../consts';
import type { AppCtx } from '../context';
import type { Guidance } from '../guidance';
import { TableTalk } from '../talk';

/** Toasts waiting at most; older ones are dropped. */
const TOAST_QUEUE = 3;

export class Fx {
  private wakers: (() => void)[] = [];
  private readonly toastEl: HTMLElement;
  private toastTimer: number | undefined;
  private toasts: { text: string; place: ToastPlace }[] = [];
  /** The quips at the table: when someone says what (../talk.ts). */
  readonly talk: TableTalk;

  constructor(
    private readonly ctx: AppCtx,
    guidance: Guidance,
  ) {
    this.toastEl = ctx.root.querySelector<HTMLElement>('.toast-layer') ?? ctx.root;
    this.talk = new TableTalk(ctx, guidance);
  }

  /** Series ("section-index") whose hello was said. */
  get quipSeries(): string {
    return this.talk.greeted;
  }

  set quipSeries(key: string) {
    this.talk.greeted = key;
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
   * A short message; while one is up, the next ones wait their turn. In a
   * run it is a strip over the bar of the head (where, account, Groschen),
   * whatever the place asked for: never over the seats, the skat, the trick
   * or the hand. Elsewhere it stands in the middle of the screen.
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
    const bar = this.ctx.screenEl.querySelector('.run .hud-top')?.getBoundingClientRect();
    this.toastEl.classList.toggle('is-head', Boolean(bar));
    if (bar) {
      this.toastEl.style.top = `${Math.round(bar.top)}px`;
      this.toastEl.style.setProperty('--toast-w', `${Math.round(bar.width)}px`);
      this.toastEl.style.setProperty('--toast-h', `${Math.round(bar.height)}px`);
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

  /** Hello from the table when a new series sits down. */
  greetSeries(view: RunView): void {
    this.talk.greetSeries(view);
  }

  /** Now and then a word after a trick (Augen from the core). */
  trickQuip(view: RunView, trick: CompletedTrick, points: number): void {
    this.talk.trick(view, trick, points);
  }

  /** Now and then a word on the Ansage. */
  declareQuip(view: RunView, declaration: Declaration): void {
    this.talk.declared(view, declaration);
  }

  /** Now and then a word on the game just scored, once the board is gone. */
  scoreQuip(view: RunView, breakdown: ScoreBreakdown): void {
    this.talk.scored(view, breakdown);
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

// Trigger feedback: when a relic or a card modification fires (a score line
// or Groschen with that source in the core's events), its chip in the head
// or its card on the table wiggles and a short line says what it did
// ("Spitze: +1 Stufe"). The lines come in a small stack below the head and
// never flood: at most a few at once, the rest summed up in one line.
// Reduced motion: no wiggle, only a short highlight. Pure presentation.

import { de, t } from '../i18n';
import type { Source } from '../types';
import { esc, reducedMotion } from './dom';

/** At most this many lines on screen at once. */
const MAX_LINES = 3;
/** How long a line stays at tempo "Normal". */
const LINE_MS = 1700;

export interface Trigger {
  source: Source;
  /** The line to show, e.g. "Spitze: +1 Stufe". */
  text: string;
}

/** The elements on screen that stand for a source: its relic chip, its card. */
function targetsOf(screen: HTMLElement, source: Source): Element[] {
  if (typeof source !== 'object') return [];
  if ('Relic' in source) return Array.from(screen.querySelectorAll(`[data-info="relic:${CSS.escape(source.Relic)}"], [data-info="evening:${CSS.escape(source.Relic)}"]`));
  if ('CardMod' in source) return Array.from(screen.querySelectorAll(`[data-info="card:${source.CardMod.card}"]`));
  return [];
}

/** Wiggle an element once (a highlight with reduced motion). */
export function wiggle(el: Element, tempo: number): void {
  el.classList.remove('is-firing');
  // Restart the animation on an element that just fired.
  void (el as HTMLElement).offsetWidth;
  el.classList.add('is-firing');
  (el as HTMLElement).style.setProperty('--fire-ms', `${Math.round(520 * tempo)}ms`);
  window.setTimeout(() => el.classList.remove('is-firing'), Math.round((reducedMotion() ? 700 : 560) * tempo) + 60);
}

export class TriggerFeed {
  private lines: { el: HTMLElement; timer: number }[] = [];
  private more = 0;
  private moreEl: HTMLElement | null = null;

  constructor(private readonly layer: HTMLElement) {}

  /** Show what fired: wiggle the things on `screen`, add the lines (rate-limited) unless `lines` is false. */
  fire(triggers: Trigger[], screen: HTMLElement, tempo: number, lines = true): void {
    for (const trig of triggers) {
      for (const el of targetsOf(screen, trig.source)) wiggle(el, tempo);
      if (lines && trig.text) this.add(trig.text, tempo);
    }
  }

  /** Clear the stack (a dialog takes the screen). */
  clear(): void {
    for (const l of this.lines) window.clearTimeout(l.timer);
    this.lines = [];
    this.more = 0;
    this.moreEl = null;
    this.layer.innerHTML = '';
  }

  /** Below the head of the run. */
  place(top: number | null): void {
    this.layer.style.top = top === null ? '' : `${Math.round(top)}px`;
  }

  private add(text: string, tempo: number): void {
    const ms = Math.round(LINE_MS * tempo);
    if (this.lines.length >= MAX_LINES) {
      // Full: one line counts the rest.
      this.more += 1;
      if (!this.moreEl) {
        this.moreEl = document.createElement('div');
        this.moreEl.className = 'trigger-line is-more';
        this.layer.appendChild(this.moreEl);
      }
      this.moreEl.textContent = t(de.trigger.more, { n: this.more });
      return;
    }
    const el = document.createElement('div');
    el.className = 'trigger-line';
    el.dataset.testid = 'trigger';
    el.innerHTML = esc(text);
    if (this.moreEl) this.layer.insertBefore(el, this.moreEl);
    else this.layer.appendChild(el);
    const entry = {
      el,
      timer: window.setTimeout(() => {
        el.remove();
        this.lines = this.lines.filter((l) => l !== entry);
        if (!this.lines.length) {
          this.moreEl?.remove();
          this.moreEl = null;
          this.more = 0;
        }
      }, ms),
    };
    this.lines.push(entry);
  }
}

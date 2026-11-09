// The Wirt above the dialogs: his hints ("Geführter erster Abend",
// ../hints.ts) and, in the Einführung (DESIGN 16.1), his coaching
// (../tutorial.ts). After every render the app asks which hint fits what
// is on screen, shows it once and remembers it when it is dismissed or the
// situation has passed.

import { sound } from '../audio';
import { de } from '../i18n';
import { clearSeenHints, hintApplies, loadSeenHints, pickHint, saveSeenHints, type HintAnchor, type HintId } from '../hints';
import { Coach } from '../tutorial';
import { renderCoach } from '../ui/coach';
import { hintText, renderHint } from '../ui/hint';
import type { AppCtx } from './context';

export class Guidance {
  private readonly hintEl: HTMLElement;
  /** The Wirt as coach of the Einführung (../tutorial.ts). */
  private readonly coachEl: HTMLElement;
  /** Hints of the Wirt already seen (stored). */
  private hintSeen = loadSeenHints();
  /** The hint on screen and where it sits. */
  hint: { id: HintId; anchor: HintAnchor; html: string } | null = null;
  /** Hints shown but not dismissed: seen once their situation has passed. */
  private hintsShown = new Set<HintId>();
  /** The player passed a game in this visit (for the hint about passing). */
  passedOnce = false;
  /** The Einführung in progress; null otherwise. Its run is never saved nor counted. */
  coach: Coach | null = null;
  /** The coach's bubble on screen (to draw it only when it changes). */
  private coachHtml = '';

  constructor(private readonly ctx: AppCtx) {
    this.hintEl = ctx.root.querySelector<HTMLElement>('.hint-layer') ?? ctx.root;
    this.coachEl = ctx.root.querySelector<HTMLElement>('.coach-layer') ?? ctx.root;
  }

  private markHintSeen(id: HintId): void {
    if (this.hintSeen.has(id)) return;
    this.hintSeen.add(id);
    saveSeenHints(this.hintSeen);
  }

  /**
   * Show the hint that fits the screen now, if any. A hint shown before
   * whose situation has passed counts as seen; a dialog in front of it only
   * hides it for the moment.
   */
  updateHint(): void {
    const ui = this.ctx.ui;
    // In the Einführung the Wirt coaches instead (../tutorial.ts).
    if (this.coach) {
      if (this.hint) this.hintEl.innerHTML = '';
      this.hint = null;
      return;
    }
    const seen = { ui, passed: this.passedOnce };
    const on = ui.settings.hints;
    if (on) {
      for (const id of this.hintsShown) {
        if (hintApplies(id, seen)) continue;
        this.markHintSeen(id);
        this.hintsShown.delete(id);
      }
    }
    const pick = on ? pickHint(seen, this.hintSeen) : null;
    if (!pick) {
      if (this.hint) this.hintEl.innerHTML = '';
      this.hint = null;
      return;
    }
    // One bubble at a time: the Wirt goes before a Stammgast's line.
    if (ui.quip) {
      ui.quip = null;
      this.ctx.screenEl.querySelector('[data-testid="quip"]')?.remove();
    }
    const html = renderHint(pick.id, pick.anchor, hintText(pick.id, ui));
    if (html !== this.hint?.html) {
      // The same hint with a new number does not pop in again.
      this.hintEl.classList.toggle('is-static', this.hint?.id === pick.id);
      this.hintEl.innerHTML = html;
    }
    this.hint = { ...pick, html };
    this.hintsShown.add(pick.id);
    this.positionHint();
  }

  /** Put the bubble right below what it hangs from: the head of the run, the Wirt at the Tresen, the end title, or the top. */
  positionHint(): void {
    const hint = this.hint;
    if (!hint) return;
    const selector = { hud: '.hud', end: '.end-board h1', shop: '.shop .backbar, .shop .backbar > *', top: null }[hint.anchor];
    const bottoms = selector ? Array.from(this.ctx.screenEl.querySelectorAll(selector), (el) => el.getBoundingClientRect().bottom) : [];
    this.hintEl.style.top = bottoms.length ? `${Math.round(Math.max(...bottoms) + 6)}px` : '';
  }

  dismissHint(): void {
    const id = this.hint?.id;
    if (!id) return;
    this.markHintSeen(id);
    this.hintsShown.delete(id);
    this.updateHint();
  }

  /** Menu "Hinweise zurücksetzen": every hint shows again. */
  resetHints(): void {
    clearSeenHints();
    this.hintSeen = new Set();
    this.hintsShown.clear();
    this.passedOnce = false;
    this.ctx.showToast(de.menu.hintsResetDone);
    sound.play('clink');
    this.ctx.render();
  }

  /** The Wirt starts coaching (the Einführung begins). */
  startCoach(): void {
    this.coach = new Coach();
    this.coachHtml = '';
  }

  /** The coach goes; the bubble and ring with him. */
  endCoach(): void {
    if (!this.coach) return;
    this.coach = null;
    this.coachHtml = '';
    this.coachEl.innerHTML = '';
  }

  /** The coach's next bubble ("Weiter", Space or Enter). */
  nextCoach(): void {
    this.coach?.next();
    this.updateCoach();
  }

  /**
   * Die Einführung: while the Wirt explains (an info step), Space and
   * Enter go on with his bubble instead of pressing the screen's main
   * button, so no lesson is skipped by the keyboard; on his last bubble
   * they do nothing (its two buttons want a deliberate choice). Runs in
   * the capture phase, before ../ui/keys.ts, which then sees it handled.
   */
  coachKey(e: KeyboardEvent): void {
    if (e.key !== ' ' && e.key !== 'Enter') return;
    const kind = this.coach?.step?.kind;
    if (!kind || kind === 'do' || e.ctrlKey || e.metaKey || e.altKey || e.isComposing) return;
    const t = e.target;
    if (t instanceof HTMLElement && t !== document.body && t.closest('button, a[href], input, textarea, select, [contenteditable="true"]')) return;
    if (document.querySelector('[data-testid="keys-help"]') || this.ctx.info.state?.mode === 'sheet') return;
    e.preventDefault();
    if (e.repeat || kind === 'end') return;
    this.nextCoach();
  }

  /** Show the coach's step for what is on screen (after every render). */
  updateCoach(): void {
    const ui = this.ctx.ui;
    const coach = this.coach;
    if (!coach || !ui.view?.tutorial) {
      if (this.coachHtml) this.coachEl.innerHTML = '';
      this.coachHtml = '';
      return;
    }
    const step = coach.update(ui);
    if (!step) {
      if (this.coachHtml) this.coachEl.innerHTML = '';
      this.coachHtml = '';
      return;
    }
    // One bubble at a time: the Wirt goes before a Stammgast's line.
    if (ui.quip) {
      ui.quip = null;
      this.ctx.screenEl.querySelector('[data-testid="quip"]')?.remove();
    }
    const html = renderCoach(step.id, step.kind, step.text(ui), coach.label(ui));
    if (html !== this.coachHtml) {
      this.coachEl.innerHTML = html;
      this.coachHtml = html;
    }
    this.positionCoach();
    // The layout may still settle (cards dealt, a sheet sliding in).
    requestAnimationFrame(() => this.positionCoach());
    window.setTimeout(() => this.positionCoach(), 360);
  }

  /**
   * Ring what the step talks about and put the bubble above or below it,
   * whichever half of the screen is free, the arrow towards it. Without a
   * target the bubble hangs below the head of the run.
   */
  positionCoach(): void {
    const step = this.coach?.step;
    const box = this.coachEl.querySelector<HTMLElement>('.coach');
    const ring = this.coachEl.querySelector<HTMLElement>('.coach-ring');
    if (!step || !box || !ring) return;
    const selector = step.target?.(this.ctx.ui) ?? null;
    const target = selector
      ? Array.from(this.ctx.root.querySelectorAll(selector)).find((el) => {
          const r = el.getBoundingClientRect();
          return r.width > 0 && r.height > 0;
        })
      : undefined;
    const vh = window.innerHeight;
    const vw = window.innerWidth;
    const h = box.offsetHeight;
    box.classList.remove('is-above', 'is-below');
    if (!target) {
      ring.hidden = true;
      const hud = this.ctx.screenEl.querySelector('.hud')?.getBoundingClientRect();
      const top = hud && hud.bottom + h + 8 < vh ? hud.bottom + 6 : 8;
      box.style.top = `${Math.round(top)}px`;
      return;
    }
    const r = target.getBoundingClientRect();
    const pad = 4;
    ring.hidden = false;
    ring.style.left = `${Math.round(Math.max(2, r.left - pad))}px`;
    ring.style.top = `${Math.round(Math.max(2, r.top - pad))}px`;
    ring.style.width = `${Math.round(Math.min(vw - 4, r.width + 2 * pad))}px`;
    ring.style.height = `${Math.round(Math.min(vh - 4, r.height + 2 * pad))}px`;
    const below = step.place ? step.place === 'below' : r.top + r.height / 2 < vh / 2;
    let top = below ? r.bottom + 14 : r.top - 14 - h;
    top = Math.max(8, Math.min(vh - h - 8, top));
    box.style.top = `${Math.round(top)}px`;
    box.classList.add(below ? 'is-below' : 'is-above');
    const left = box.getBoundingClientRect().left;
    const x = Math.max(22, Math.min(box.offsetWidth - 22, r.left + r.width / 2 - left));
    box.style.setProperty('--arrow-x', `${Math.round(x)}px`);
  }
}

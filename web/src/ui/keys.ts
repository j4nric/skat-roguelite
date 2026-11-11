// Keyboard on the desktop: 1 to 9 and 0 play the first to tenth card of the
// hand as shown, Space or Enter press the main button of the screen
// ("Weiter", "Los", "Drücken"), S picks up the skat, H plays a hand game,
// L shows the last trick, ? or F1 a small list of these keys. Every key
// presses a button that is on screen anyway, so the core decides as with a
// tap. The first key marks the page (`data-kbd`): the hand cards then show
// their digit. A touch puts the marks away again.
//
// Pure input handling, no rules: which card may go the view said
// (`data-action` and `data-legal` on the hand cards).

import { de } from '../i18n';
import { attrs, esc } from './dom';

export interface KeyHandlers {
  /** Something else owns the keys right now (an info sheet, a lifted Schnaps). */
  blocked: () => boolean;
  /** A dialog is open over the screen: only its main button counts. */
  dialogOpen: () => boolean;
  /** A hand card the core does not let go now was asked for (as if thrown). */
  refused: (card: HTMLElement) => void;
  /** Space or Enter found no button: hurry what is playing. */
  idle: () => void;
}

/** Buttons that count as the main one of a screen or dialog. The hand game is no default. */
const PRIMARY = '.btn-primary:not(.btn-hand), .route-go, button[data-action="score-skip"]';

function visible(el: HTMLElement): boolean {
  if (el instanceof HTMLButtonElement && el.disabled) return false;
  const r = el.getBoundingClientRect();
  return r.width > 0 && r.height > 0 && getComputedStyle(el).visibility !== 'hidden';
}

/**
 * The main button in `box`: the only one, or the only wide one among
 * several. None when that is not clear (a choice of offers).
 */
export function primaryButton(box: HTMLElement): HTMLElement | null {
  const all = Array.from(box.querySelectorAll<HTMLElement>(PRIMARY)).filter(visible);
  if (all.length === 1) return all[0] ?? null;
  const wide = all.filter((b) => b.classList.contains('btn-wide'));
  return wide.length === 1 ? (wide[0] ?? null) : null;
}

function typing(t: EventTarget | null): boolean {
  return t instanceof HTMLElement && (t.isContentEditable || t.closest('input, textarea, select') !== null);
}

/** Whether a focused control takes Space and Enter itself. */
function focusTakes(t: EventTarget | null): boolean {
  return t instanceof HTMLElement && t.closest('button, a[href], [role="button"]') !== null;
}

function helpHtml(): string {
  const rows = de.keys.rows.map(([k, text]) => `<li><kbd>${esc(k)}</kbd><span>${esc(text)}</span></li>`).join('');
  return `<div class="keys-help-card sheet" role="dialog" aria-modal="true" aria-labelledby="keys-help-title">
    <h2 id="keys-help-title">${esc(de.keys.title)}</h2>
    <ul class="keys-list">${rows}</ul>
    <button class="btn btn-wide" ${attrs({ type: 'button', 'data-keys-close': '' })}>${esc(de.keys.close)}</button>
  </div>`;
}

/** Listen for the keys; `screen` holds the current screen, `layer` its dialogs. */
export function watchKeys(root: HTMLElement, screen: HTMLElement, layer: HTMLElement, h: KeyHandlers): void {
  let help: HTMLElement | null = null;
  /** Where focus was before the list opened: it goes back there. */
  let before: HTMLElement | null = null;
  const closeHelp = (): void => {
    help?.remove();
    help = null;
    if (before?.isConnected) before.focus({ preventScroll: true });
    before = null;
  };
  const openHelp = (): void => {
    if (help) return;
    before = document.activeElement instanceof HTMLElement && document.activeElement !== document.body ? document.activeElement : null;
    help = document.createElement('div');
    help.className = 'overlay keys-help';
    help.setAttribute('data-testid', 'keys-help');
    help.innerHTML = helpHtml();
    help.addEventListener('click', (e) => {
      const card = help?.querySelector('.keys-help-card');
      const t = e.target instanceof Element ? e.target : null;
      if (t?.closest('[data-keys-close]') || !card?.contains(t)) closeHelp();
      e.stopPropagation();
    });
    root.appendChild(help);
    help.querySelector<HTMLElement>('[data-keys-close]')?.focus({ preventScroll: true });
  };

  // A finger on the glass: no keyboard hints (a phone with a keyboard case keeps them until then).
  window.addEventListener(
    'pointerdown',
    (e) => {
      if (e.pointerType === 'touch') document.documentElement.removeAttribute('data-kbd');
    },
    { passive: true, capture: true },
  );

  window.addEventListener('keydown', (e) => {
    if (e.defaultPrevented || e.ctrlKey || e.metaKey || e.altKey || e.isComposing) return;
    if (typing(e.target)) return;
    // Only real keys mark the page (not the modifier alone).
    if (e.key.length === 1 || e.key === 'Enter' || e.key === 'F1') document.documentElement.setAttribute('data-kbd', '');
    if (help) {
      if (e.key === 'Escape' || e.key === '?' || e.key === 'F1' || e.key === 'Enter' || e.key === ' ') {
        e.preventDefault();
        e.stopImmediatePropagation();
        closeHelp();
      }
      return;
    }
    if (e.key === '?' || e.key === 'F1') {
      e.preventDefault();
      openHelp();
      return;
    }
    if (h.blocked()) return;
    const dialog = h.dialogOpen();
    if (e.key === ' ' || e.key === 'Enter') {
      if (focusTakes(e.target) && e.target !== document.body) return;
      e.preventDefault();
      if (e.repeat) return;
      const button = primaryButton(dialog ? layer : screen);
      if (button) button.click();
      else if (!dialog) h.idle();
      return;
    }
    if (dialog || e.repeat) return;
    if (/^[0-9]$/.test(e.key)) {
      const index = e.key === '0' ? 9 : Number(e.key) - 1;
      const card = screen.querySelectorAll<HTMLElement>('.hand .hand-card')[index];
      if (!card) return;
      e.preventDefault();
      if (card.dataset.action) card.click();
      else if (card.dataset.legal === 'false') h.refused(card);
      return;
    }
    const press = (action: string): void => {
      const button = screen.querySelector<HTMLElement>(`[data-action="${action}"]`);
      if (button && visible(button)) {
        e.preventDefault();
        button.click();
      }
    };
    switch (e.key.toLowerCase()) {
      case 's':
        press('pick-up-skat');
        return;
      case 'h':
        press('play-hand');
        return;
      case 'l':
        press('last-trick');
        return;
      default:
    }
  });
}

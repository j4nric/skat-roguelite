// The Wirt's speech bubble ("Geführter erster Abend", ../hints.ts): his
// pixel portrait and a short text. The whole bubble is one button that
// dismisses it; around it every tap goes through to the table. It sits
// fixed below the head of the run (or at the top over a dialog), so it
// never covers the hand or the buttons it talks about, and never makes the
// page scroll.

import type { HintAnchor, HintId } from '../hints';
import { de, t } from '../i18n';
import { attrs, esc } from './dom';
import { portrait } from './people';
import type { UiState } from './state';

/** The text of a hint, with what the view shows filled in. */
export function hintText(id: HintId, ui: UiState): string {
  const game = ui.view?.game;
  switch (id) {
    case 'value': {
      const p = game?.value_preview;
      return t(de.hints.value, { base: p?.base_value ?? '', levels: p?.levels ?? '' });
    }
    case 'afterSkat':
      // The Nullspieler's Ansage after the skat offers Null levels (from the core).
      return game?.declare?.null_risks.length ? de.hints.afterSkatNull : de.hints.afterSkat;
    default:
      return de.hints[id];
  }
}

export function renderHint(id: HintId, anchor: HintAnchor, text: string): string {
  return `<button class="hint hint-${esc(anchor)}" ${attrs({
    'data-action': 'hint-dismiss',
    'data-hint': id,
    'data-testid': 'hint',
    'aria-label': `${de.hints.speaker}: ${text} ${de.hints.dismiss}`,
    title: de.hints.dismiss,
  })}>
    ${portrait('p-wirt', 'portrait-hint')}
    <span class="hint-bubble">
      <span class="hint-who">${esc(de.hints.speaker)}</span>
      <span class="hint-text">${esc(text)}</span>
      <span class="hint-x" aria-hidden="true">×</span>
    </span>
  </button>`;
}

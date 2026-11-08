// The Wirt as coach of the Einführung (../tutorial.ts): his portrait and a
// speech bubble with the lesson, a ring around what he talks about, and a
// way out at any time. The bubble is one button that goes on to the next
// step; the app places it above or below the ring (positionCoach) so it
// never covers what it points at. Around it every tap goes through.

import type { CoachKind } from '../tutorial';
import { de } from '../i18n';
import { attrs, esc } from './dom';
import { portrait } from './people';

export function renderCoach(id: string, kind: CoachKind, text: string, label: string): string {
  const TD = de.tutorial;
  const end =
    kind === 'end'
      ? `<span class="coach-end">
          <button class="btn btn-primary" ${attrs({ 'data-action': 'tutorial-finish', 'data-go': 'run', 'data-testid': 'coach-finish-run' })}>${esc(TD.finishRun)}</button>
          <button class="btn" ${attrs({ 'data-action': 'tutorial-finish', 'data-testid': 'coach-finish-start' })}>${esc(TD.finishStart)}</button>
        </span>`
      : '';
  const main =
    kind === 'end'
      ? `<div class="coach-main" role="note">${bubble(text, label, '')}</div>`
      : `<button class="coach-main" ${attrs({
          'data-action': 'coach-next',
          'data-testid': 'coach-next',
          'aria-label': `${TD.speaker}: ${text} ${kind === 'info' ? TD.next : ''}`,
        })}>${bubble(text, label, kind === 'info' ? `<span class="coach-go">${esc(TD.next)} ›</span>` : `<span class="coach-go is-do">${esc(TD.doIt)}</span>`)}</button>`;
  return `<div class="coach" ${attrs({ 'data-testid': 'coach', 'data-step': id, 'data-kind': kind })}>
    <span class="coach-arrow" aria-hidden="true"></span>
    ${main}
    ${end}
    ${kind === 'end' ? '' : `<button class="coach-skip" ${attrs({ 'data-action': 'tutorial-skip', 'data-testid': 'coach-skip', 'aria-label': TD.skipAria })}>${esc(TD.skip)}</button>`}
  </div>
  <div class="coach-ring" aria-hidden="true" hidden></div>`;
}

function bubble(text: string, label: string, go: string): string {
  return `${portrait('p-wirt', 'portrait-hint')}
    <span class="hint-bubble coach-bubble">
      <span class="hint-who">${esc(label)}</span>
      <span class="hint-text coach-text">${esc(text)}</span>
      ${go}
    </span>`;
}

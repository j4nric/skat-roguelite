// Die lange Nacht (ADR 0020): the moon, the Verschärfung chips and the
// start sheet of a night district. Presentation only; what a Verschärfung
// does and when it is drawn is the core's.

import { de, t } from '../i18n';
import type { RunView, VerschaerfungInfo } from '../types';
import { attrs, esc } from './dom';

/** A crescent moon for the long night (ADR 0020). */
export function moonIcon(size: number): string {
  return `<svg class="moon-icon" viewBox="0 0 16 16" width="${size}" height="${size}" aria-hidden="true"><path d="M10 1a7 7 0 1 0 5 11.5A6 6 0 0 1 10 1z" fill="currentColor"/></svg>`;
}

/**
 * "Die Sperrstunde rückt näher" (ADR 0020): the start sheet of a night
 * district with the Verschärfung just drawn, the ones in force and the
 * growth of the Zielwerte.
 */
export function renderNightSheet(view: RunView): string {
  const night = view.night;
  const fresh = night?.announced;
  if (!night || !fresh) return '';
  const older = night.verschaerfungen.filter((v) => v.district !== fresh.district);
  const chips = older.map((v) => `<li>${nightChip(v)}</li>`).join('');
  const factor = (night.target_permille / 1000).toLocaleString('de-DE', { maximumFractionDigits: 2 });
  return `<div class="night-sky" aria-hidden="true">${nightStars(14, fresh.district)}</div>
    <p class="night-kicker">${moonIcon(16)} ${esc(de.night.title)}</p>
    <h2 data-testid="night-sheet-title">${esc(t(de.night.sheetTitle, { n: fresh.district }))}</h2>
    <p class="small">${esc(de.night.sheetLead)}</p>
    <div class="night-new" data-testid="night-new" ${attrs({ 'data-verschaerfung': fresh.id })}><b>${esc(fresh.name)}</b><span>${esc(fresh.text)}</span></div>
    ${older.length ? `<p class="small">${esc(de.night.sheetActive)}:</p><ul class="night-chips">${chips}</ul>` : ''}
    <p class="small night-targets">${esc(t(de.night.sheetTargets, { factor }))}</p>
    <button class="btn btn-primary btn-wide" ${attrs({ 'data-action': 'close-overlay', 'data-testid': 'night-go' })}>${esc(de.night.sheetGo)}</button>`;
}

/** A Verschärfung as a chip; long press (data-info) explains it. */
export function nightChip(v: VerschaerfungInfo): string {
  return `<button type="button" class="night-chip" ${attrs({ 'data-action': 'info-night', 'data-info': `night:${v.id}:${v.district}`, 'data-testid': 'night-chip', title: v.name })}>${moonIcon(11)}<span>${esc(v.name)}</span></button>`;
}

/** A few stars in a deterministic pattern (decoration of the night). */
export function nightStars(n: number, salt: number): string {
  let x = (salt * 2654435761) >>> 0;
  const next = (): number => {
    x = (Math.imul(x ^ (x >>> 15), 2246822519) + 3266489917) >>> 0;
    return x / 4294967296;
  };
  let out = '';
  for (let i = 0; i < n; i++) {
    out += `<i style="left:${(next() * 100).toFixed(1)}%;top:${(next() * 100).toFixed(1)}%;--d:${(next() * 3).toFixed(2)}s"></i>`;
  }
  return out;
}


/**
 * The chips of the Verschärfungen in force for the Leiste, with the night
 * district first. Empty outside the night.
 */
export function nightChips(view: RunView): string {
  const night = view.night;
  if (!night) return '';
  const chips = night.verschaerfungen.map(nightChip).join('');
  return `<span class="night-chips-row" data-testid="night-chips" ${attrs({ 'aria-label': de.night.chips })}>${chips}</span>`;
}

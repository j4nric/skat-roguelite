// Timings, small helpers and texts the app modules share. Nothing here
// holds state; the rules come from the core.

import { de, contentName, t } from '../i18n';
import { CHARACTERS, type Character, type RunView } from '../types';

/** Pauses in milliseconds at tempo "Normal"; reduced motion shortens them. */
export const DELAY = {
  ownCard: 340,
  aiCard: 560,
  trickShow: 650,
  trickFly: 480,
  toast: 1900,
  notice: 700,
  /** The skat turning over on the table. */
  flip: 420,
  /** Relics and modifications that fired, before the score board covers them. */
  trigger: 650,
};

/** A card picked up or laid down: flight (ms at tempo Normal), lift and curve (px). */
export const PICK = { ms: 440, lift: 28, arc: 16 };

/** Where a toast shows: the middle of the screen, or above the Stammgäste. */
export type ToastPlace = 'middle' | 'top';

/** Let the browser handle input and paint before the next piece of work. */
export function yieldToBrowser(): Promise<void> {
  return new Promise((resolve) => window.setTimeout(resolve, 0));
}

/** Wait until the next frame has been painted (or 120 ms in a hidden tab). */
export function nextPaint(): Promise<void> {
  return new Promise((resolve) => {
    let done = false;
    const finish = (): void => {
      if (done) return;
      done = true;
      resolve();
    };
    // rAF runs just before the paint; the timeout after it runs once the
    // frame is on screen.
    requestAnimationFrame(() => window.setTimeout(finish, 0));
    window.setTimeout(finish, 120);
  });
}

/** Where the first element of `selector` below `root` is on screen, if any. */
export function rectOf(root: ParentNode, selector: string): DOMRect | null {
  const el = root.querySelector(selector);
  return el ? el.getBoundingClientRect() : null;
}

/** The stable code of a core error (JSON `{code, message}`), if any. */
export function errorCode(err: unknown): string | null {
  const raw = err instanceof Error ? err.message : String(err);
  try {
    const parsed = JSON.parse(raw) as { code?: string };
    return parsed.code ?? null;
  } catch {
    return null;
  }
}

/** Turn a core error into a German text. */
export function errorText(err: unknown): string {
  const code = errorCode(err);
  const fallback = de.app.errors.default ?? '';
  return code ? (de.app.errors[code] ?? fallback) : fallback;
}

/** One line about a saved run: who plays, where, the account. */
export function runLine(view: RunView): string {
  const character = contentName('characters', view.character.character);
  const line = view.phase === 'Encounter' || view.phase === 'Route' || view.phase === 'Shop' || view.phase === 'BossReward' || view.phase === 'CardReward'
    ? t(de.start.resumeEncounter, { character, section: view.series.section + 1 })
    : t(de.start.resumeInfo, {
        character,
        section: view.series.section + 1,
        series: view.series.index + 1,
        score: view.series.score,
        target: view.series.target,
      });
  const short = view.length === 'Short' ? `${line} · ${de.start.shortEvening}` : line;
  return view.stake > 0 ? short + t(de.share.lineStake, { stake: view.stake }) : short;
}

export function isCharacter(value: string | null): value is Character {
  return CHARACTERS.includes(value as Character);
}

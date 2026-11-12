// Vibration on the phone (presentation only): a short tap when the player
// takes a trick, a pattern when a game or a pub is won or lost. Off in the
// settings, or where the browser has no `navigator.vibrate` (iOS Safari):
// then nothing happens. In the iOS app (ADR 0026) the Taptic Engine plays
// it instead (./native.ts). Every call is guarded; a vibration is never needed.

import { isNativeApp } from './platform';

export type Buzz = 'trick' | 'gameWon' | 'gameLost' | 'pubWon' | 'pubLost' | 'runWon';

/** Patterns in ms: vibrate, pause, vibrate, … */
const PATTERNS: Record<Buzz, number | number[]> = {
  trick: 12,
  gameWon: [25, 60, 25],
  gameLost: [70],
  pubWon: [30, 50, 30, 50, 90],
  pubLost: [140, 80, 140],
  runWon: [40, 60, 40, 60, 40, 60, 160],
};

let enabled = true;

/** Whether this browser can vibrate at all (the menu shows the switch only then). */
export function canVibrate(): boolean {
  if (isNativeApp()) return true;
  try {
    return typeof navigator !== 'undefined' && typeof navigator.vibrate === 'function';
  } catch {
    return false;
  }
}

/** The setting "Vibration". */
export function setVibration(on: boolean): void {
  enabled = on;
}

export function buzz(kind: Buzz): void {
  if (!enabled || !canVibrate()) return;
  if (isNativeApp()) {
    import('./native').then((n) => n.haptic(kind)).catch(() => undefined);
    return;
  }
  try {
    navigator.vibrate(PATTERNS[kind]);
  } catch {
    // Not allowed now (no user gesture yet, a frame): no vibration.
  }
}

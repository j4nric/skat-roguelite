// Entry point of the Stammtisch web app.
//
// Boot order (W1 of the review): the WebAssembly core starts downloading
// and compiling first, the app shows its loading screen at once, and the
// card sprite and the pixel portraits are prepared while the core loads.
// The HTML preloads the core and the main sprite (web/sprite/plugin.ts),
// so both are on the way before this script runs.

import './style.css';
import { App } from './app';
import { loadCore } from './core';
import { setLanguage, storedOrDetectedLanguage } from './i18n';
import { registerServiceWorker } from './offline';
import { installSprite, prefetchRareSprite } from './ui/art';
import { blinkJobs, portraitJobs } from './ui/people';
import { preparePixelArt } from './ui/pixelate';

// The app reports a core that fails to load (App.start); here it only starts.
loadCore().catch(() => undefined);

const root = document.getElementById('app');
watchArtStep();

/**
 * 33 device pixels in CSS px, for card sizes that keep the pixel deck's
 * art pixels even (--art-step in style.css). Follows zoom and screens.
 */
function watchArtStep(): void {
  const dpr = window.devicePixelRatio || 1;
  document.documentElement.style.setProperty('--art-step', `${(33 / dpr).toFixed(4)}px`);
  window.matchMedia(`(resolution: ${dpr}dppx)`).addEventListener('change', watchArtStep, { once: true });
}

/** Run `fn` when the browser is idle (or soon, where it has no idle callback). */
function whenIdle(fn: () => void): void {
  if (typeof window.requestIdleCallback === 'function') window.requestIdleCallback(() => fn(), { timeout: 2000 });
  else setTimeout(fn, 200);
}

/** The sprite, then the pixel portraits (they draw the suit signs from it). */
async function prepareArt(): Promise<void> {
  await installSprite();
  await preparePixelArt(portraitJobs());
}

/** Show the app at once; render the portraits first, but never wait long for them. */
async function boot(el: HTMLElement): Promise<void> {
  // The language first (ADR 0022): German at once, English as a small chunk.
  await setLanguage(storedOrDetectedLanguage()).catch(() => undefined);
  const app = new App(el);
  app.showLoading();
  const timeout = new Promise<void>((resolve) => window.setTimeout(resolve, 1500));
  await Promise.race([prepareArt(), timeout]);
  await app.start();
  // Not needed for the first screen: the opponents' blink and the rare card faces.
  whenIdle(() => {
    void preparePixelArt(blinkJobs());
    void prefetchRareSprite();
  });
}

if (root) {
  void boot(root);
}
// Offline play (DESIGN 14.5); production builds only.
registerServiceWorker();

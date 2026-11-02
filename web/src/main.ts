// Entry point of the Stammtisch web app.

import './style.css';
import { App } from './app';
import { installSprite } from './ui/art';
import { portraitJobs } from './ui/people';
import { preparePixelArt } from './ui/pixelate';

const root = document.getElementById('app');
installSprite();
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

/** Render the pixel portraits first, but never wait long for them. */
async function boot(el: HTMLElement): Promise<void> {
  const timeout = new Promise<void>((resolve) => window.setTimeout(resolve, 1500));
  await Promise.race([preparePixelArt(portraitJobs()), timeout]);
  await new App(el).start();
}

if (root) {
  void boot(root);
}

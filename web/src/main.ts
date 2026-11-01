// Entry point of the Stammtisch web app.

import './style.css';
import { App } from './app';
import { installSprite } from './ui/art';
import { goodJobs } from './ui/goods';
import { portraitJobs } from './ui/people';
import { preparePixelArt } from './ui/pixelate';

const root = document.getElementById('app');
installSprite();

/** Render the pixel portraits first, but never wait long for them. */
async function boot(el: HTMLElement): Promise<void> {
  const timeout = new Promise<void>((resolve) => window.setTimeout(resolve, 1500));
  await Promise.race([preparePixelArt([...portraitJobs(), ...goodJobs()]), timeout]);
  await new App(el).start();
}

if (root) {
  void boot(root);
}

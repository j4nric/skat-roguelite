// Vite plugin: the press kit's downloads (web/presse.html, web/en/press.html)
// in full size, copied into dist/presse/ at build time from where they
// already live in the repo, so no image is kept twice:
//
// - store/screenshots/<de|en>/*.png: the App Store screenshots (1320 x 2868),
// - store/art/header-3840x1646.png: the key art (store/make-landing-media.py),
// - the iOS app icon (1024 x 1024) from the Xcode asset catalogue.
//
// Build only. The service worker leaves dist/presse/ out of its precache
// (sw/plugin.ts): nobody needs them offline. The iOS workflow deletes
// dist/presse/ and dist/landing/ before the app is built
// (.github/workflows/ios.yml); the app does not show those pages.

import { copyFileSync, existsSync, mkdirSync, readdirSync } from 'node:fs';
import { join, resolve } from 'node:path';
import type { Plugin } from 'vite';

export function pressKit(): Plugin {
  let outDir = '';
  let repo = '';
  return {
    name: 'stammtisch-press-kit',
    apply: 'build',
    configResolved(config) {
      outDir = resolve(config.root, config.build.outDir);
      repo = resolve(config.root, '..');
    },
    // writeBundle runs before closeBundle, where the service worker lists the build.
    writeBundle() {
      const press = join(outDir, 'presse');
      const copy = (from: string, to: string): void => {
        if (!existsSync(from)) throw new Error(`press kit: ${from} is missing`);
        mkdirSync(join(to, '..'), { recursive: true });
        copyFileSync(from, to);
      };
      copy(join(repo, 'store', 'art', 'header-3840x1646.png'), join(press, 'header-3840x1646.png'));
      copy(join(repo, 'ios', 'App', 'App', 'Assets.xcassets', 'AppIcon.appiconset', 'AppIcon-1024.png'), join(press, 'icon-1024.png'));
      for (const lang of ['de', 'en']) {
        const dir = join(repo, 'store', 'screenshots', lang);
        for (const name of readdirSync(dir).filter((n) => n.endsWith('.png'))) {
          copy(join(dir, name), join(press, 'screenshots', lang, name));
        }
      }
    },
  };
}

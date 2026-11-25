// Vite plugin: writes the service worker (./sw.js) into the build output
// with the build's file list and a version derived from their contents.
// Build only (`vite build`); `vite dev` has no service worker.

import { createHash } from 'node:crypto';
import { readdirSync, readFileSync, writeFileSync } from 'node:fs';
import { join, relative, resolve, sep } from 'node:path';
import type { Logger, Plugin } from 'vite';
import { ROUTES } from '../site/routes.ts';

// Not precached: source maps, the worker itself, dotfiles, the 512 px
// icons (only installing the app needs them, which happens online), the
// landing page's video and pictures (a first visit is online, and a
// returning player is sent on to the game) and the press kit's downloads.
const SKIP = [/\.map$/, /^sw\.js$/, /^\./, /^icon(-maskable)?-512\.png$/, /^landing\//, /^presse\//];

function listFiles(dir: string): string[] {
  return readdirSync(dir, { withFileTypes: true }).flatMap((entry) => {
    const path = join(dir, entry.name);
    return entry.isDirectory() ? listFiles(path) : [path];
  });
}

export function serviceWorker(): Plugin {
  let outDir = '';
  let templatePath = '';
  let logger: Logger | null = null;
  return {
    name: 'stammtisch-service-worker',
    apply: 'build',
    configResolved(config) {
      outDir = resolve(config.root, config.build.outDir);
      templatePath = resolve(config.root, 'sw', 'sw.js');
      logger = config.logger;
    },
    closeBundle() {
      const files = listFiles(outDir)
        .map((path) => relative(outDir, path).split(sep).join('/'))
        .filter((name) => !SKIP.some((re) => re.test(name)))
        .sort();
      const hash = createHash('sha256');
      for (const name of files) {
        hash.update(name);
        hash.update(readFileSync(join(outDir, name)));
      }
      const template = readFileSync(templatePath, 'utf8');
      hash.update(template);
      hash.update(JSON.stringify(ROUTES));
      const version = hash.digest('hex').slice(0, 12);
      // Every file under its own name; the worker maps the site's addresses
      // (/ the landing page, /spielen the game) to them with ROUTES.
      const urls = files.map((name) => `/${name}`);
      const source = template
        .replace("'__SW_VERSION__'", JSON.stringify(version))
        .replace('self.__SW_PRECACHE__', JSON.stringify(urls))
        .replace('self.__SW_ROUTES__', JSON.stringify(ROUTES));
      writeFileSync(join(outDir, 'sw.js'), source);
      logger?.info(`service worker ${version}: ${urls.length} files precached`);
    },
  };
}

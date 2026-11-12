// Vite plugin: the card sprite (web/src/ui/sprite.ts) is drawn at build
// time, not in the browser. `vite build` writes two hashed files,
// assets/sprite-<hash>.svg (everything the first screens need) and
// assets/sprite-rare-<hash>.svg (Doppelgänger, Untot and Wandelkarte
// faces), and the app fetches them at boot (web/src/ui/art.ts; the rare
// one in idle time, into the document when a card needs it). The service
// worker precaches them like every other file of the build (../sw/plugin.ts).
//
// The app reads the file names from the virtual module
// `virtual:stammtisch-sprite`. `vite dev` serves the same files from
// memory under /@stammtisch/ and draws them again after a change of the
// drawing code.
//
// The page also gets <link rel="preload"> for the main sprite and the
// WebAssembly core, so both downloads start with the HTML instead of after
// the script (W1 of the review).

import { createHash } from 'node:crypto';
import { resolve } from 'node:path';
import { runnerImport, type Plugin, type ResolvedConfig } from 'vite';

const VIRTUAL = 'virtual:stammtisch-sprite';
const RESOLVED = `\0${VIRTUAL}`;
const DEV_PREFIX = '/@stammtisch/';

interface SpriteFiles {
  main: string;
  rare: string;
  undead: string[];
}

interface Built {
  files: SpriteFiles;
  /** Source files the drawing read (dev: draw again when one changes). */
  dependencies: string[];
}

async function draw(config: ResolvedConfig): Promise<Built> {
  const entry = resolve(config.root, 'src/ui/sprite.ts');
  const { module, dependencies } = await runnerImport<{ spriteFiles: () => SpriteFiles }>(entry, {
    root: config.root,
    logLevel: 'error',
  });
  return { files: module.spriteFiles(), dependencies: dependencies.map((d) => resolve(d)) };
}

function hashed(name: string, source: string): string {
  const hash = createHash('sha256').update(source).digest('hex').slice(0, 10);
  return `${name}-${hash}.svg`;
}

export function sprite(): Plugin {
  let config: ResolvedConfig;
  let built: Promise<Built> | null = null;
  /** Public URLs of the two files ("/assets/…" in a build). */
  let urls = { main: '', rare: '' };
  let names = { main: '', rare: '' };
  const isBuild = (): boolean => config.command === 'build';

  const current = (): Promise<Built> => {
    built ??= draw(config);
    return built;
  };

  return {
    name: 'stammtisch-sprite',
    configResolved(resolved) {
      config = resolved;
    },
    async buildStart() {
      if (!isBuild()) return;
      const { files } = await current();
      const dir = config.build.assetsDir;
      names = { main: `${dir}/${hashed('sprite', files.main)}`, rare: `${dir}/${hashed('sprite-rare', files.rare)}` };
      urls = { main: `${config.base}${names.main}`, rare: `${config.base}${names.rare}` };
    },
    resolveId(id) {
      return id === VIRTUAL ? RESOLVED : null;
    },
    async load(id) {
      if (id !== RESOLVED) return null;
      const { files } = await current();
      const main = isBuild() ? urls.main : `${DEV_PREFIX}sprite.svg`;
      const rare = isBuild() ? urls.rare : `${DEV_PREFIX}sprite-rare.svg`;
      return [
        `export const SPRITE_URL = ${JSON.stringify(main)};`,
        `export const RARE_SPRITE_URL = ${JSON.stringify(rare)};`,
        `export const UNDEAD = ${JSON.stringify(files.undead)};`,
      ].join('\n');
    },
    async generateBundle() {
      const { files } = await current();
      this.emitFile({ type: 'asset', fileName: names.main, source: files.main });
      this.emitFile({ type: 'asset', fileName: names.rare, source: files.rare });
    },
    configureServer(server) {
      // Draw again after any change to the drawing code; the page reloads.
      server.watcher.on('change', (file) => {
        if (!built) return;
        built.then(
          ({ dependencies }) => {
            if (dependencies.includes(resolve(file))) {
              built = null;
              const mod = server.moduleGraph.getModuleById(RESOLVED);
              if (mod) server.moduleGraph.invalidateModule(mod);
            }
          },
          () => {
            built = null;
          },
        );
      });
      server.middlewares.use((req, res, next) => {
        const path = (req.url ?? '').split('?')[0];
        const which = path === `${DEV_PREFIX}sprite.svg` ? 'main' : path === `${DEV_PREFIX}sprite-rare.svg` ? 'rare' : null;
        if (!which) {
          next();
          return;
        }
        current().then(
          ({ files }) => {
            res.setHeader('Content-Type', 'image/svg+xml');
            res.setHeader('Cache-Control', 'no-store');
            res.end(files[which]);
          },
          (err: unknown) => {
            built = null;
            next(err);
          },
        );
      });
    },
    transformIndexHtml: {
      order: 'post',
      handler(_html, ctx) {
        // Only the game's page; the static pages beside it (imprint,
        // privacy policy, help) need neither sprite nor core.
        if (ctx.path !== '/index.html' && ctx.path !== '/') return [];
        const preload = (href: string) => ({ tag: 'link', attrs: { rel: 'preload', href, as: 'fetch', crossorigin: '' }, injectTo: 'head' as const });
        const tags = [preload(isBuild() ? urls.main : `${DEV_PREFIX}sprite.svg`)];
        // The core: its hashed name is only known once the bundle is written.
        const wasm = Object.values(ctx.bundle ?? {}).find((f) => f.type === 'asset' && /skat_wasm_bg-[^/]*\.wasm$/.test(f.fileName));
        if (wasm) tags.push(preload(`${config.base}${wasm.fileName}`));
        return tags;
      },
    },
  };
}

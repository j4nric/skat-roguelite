import { resolve } from 'node:path';
import { defineConfig } from 'vite';
import { pressKit } from './site/presskit.ts';
import { siteRoutes } from './site/routes.ts';
import { sprite } from './sprite/plugin.ts';
import { serviceWorker } from './sw/plugin.ts';

// BUILD_INFO (commit and date) is set by the deploy workflow and shown in
// the footer of the start screen.
const buildInfo = process.env.BUILD_INFO ?? 'lokal';

// The account server (server/, ADR 0021) answers /api. In production our
// Caddy proxies it; locally `cargo run -p stammtisch_server` listens on
// 127.0.0.1:8787 (STAMMTISCH_API to change). Without it the game works as
// before and the account shows "offline".
const api = {
  '/api': {
    target: process.env.STAMMTISCH_API ?? 'http://127.0.0.1:8787',
    changeOrigin: false,
  },
};

export default defineConfig({
  base: '/',
  // The card sprite, drawn at build time (sprite/plugin.ts). Offline play
  // (DESIGN 14.5): dist/sw.js with the build's file list, build and
  // preview only, never in `vite dev`. The site's addresses (/ the landing
  // page, /spielen the game) in dev and preview like on the server
  // (site/routes.ts), and the press kit's downloads (site/presskit.ts).
  plugins: [siteRoutes(), sprite(), pressKit(), serviceWorker()],
  define: {
    __BUILD_INFO__: JSON.stringify(buildInfo),
  },
  // The core worker (src/core.worker.ts, ADR 0022) is a module worker:
  // its chunk lies next to the others under /assets/ and is precached.
  worker: {
    format: 'es',
  },
  build: {
    outDir: 'dist',
    assetsDir: 'assets',
    target: 'es2022',
    // The WebAssembly core is loaded as a file, never inlined.
    assetsInlineLimit: 0,
    // One chunk of about 575 kB since the card sprite is drawn at build
    // time (sprite/plugin.ts), 640 kB on 12.11.2026; the limit warns well
    // before it doubles again.
    chunkSizeWarningLimit: 680,
    // The game, and the static pages beside it (imprint, privacy policy,
    // help; German binding, English translation under en/). They are plain
    // HTML with one stylesheet and no script; Caddy serves them without
    // `.html` (deploy/Caddyfile), the service worker keeps them offline.
    // The landing page (start.html, served as /) and the press kit
    // (presse.html) are static pages too; the landing page carries one
    // small classic script (public/landing.js) for returning players.
    rolldownOptions: {
      input: {
        main: resolve(import.meta.dirname, 'index.html'),
        impressum: resolve(import.meta.dirname, 'impressum.html'),
        datenschutz: resolve(import.meta.dirname, 'datenschutz.html'),
        support: resolve(import.meta.dirname, 'support.html'),
        'en/imprint': resolve(import.meta.dirname, 'en/imprint.html'),
        'en/privacy': resolve(import.meta.dirname, 'en/privacy.html'),
        'en/support': resolve(import.meta.dirname, 'en/support.html'),
        start: resolve(import.meta.dirname, 'start.html'),
        presse: resolve(import.meta.dirname, 'presse.html'),
        'en/start': resolve(import.meta.dirname, 'en/start.html'),
        'en/press': resolve(import.meta.dirname, 'en/press.html'),
      },
    },
  },
  server: {
    proxy: api,
  },
  preview: {
    port: 4173,
    strictPort: true,
    proxy: api,
  },
});

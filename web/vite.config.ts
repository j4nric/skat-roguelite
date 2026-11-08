import { defineConfig } from 'vite';

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
  define: {
    __BUILD_INFO__: JSON.stringify(buildInfo),
  },
  build: {
    outDir: 'dist',
    assetsDir: 'assets',
    target: 'es2022',
    // The WebAssembly core is loaded as a file, never inlined.
    assetsInlineLimit: 0,
    chunkSizeWarningLimit: 1024,
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

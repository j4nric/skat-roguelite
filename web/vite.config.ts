import { defineConfig } from 'vite';

// BUILD_INFO (commit and date) is set by the deploy workflow and shown in
// the footer of the start screen.
const buildInfo = process.env.BUILD_INFO ?? 'lokal';

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
  preview: {
    port: 4173,
    strictPort: true,
  },
});

// The site's addresses that are not a file of the build (DESIGN 16.9,
// ADR 0009). In production deploy/Caddyfile rewrites them; this map is the
// same table for `vite dev` and `vite preview` (the plugin below) and for
// the service worker, which serves the cached file for an address offline
// (sw/plugin.ts writes the map into dist/sw.js). Change all three together.
//
// - `/` is the landing page for first-time visitors (web/start.html); its
//   script (public/landing.js) sends returning players on to the game.
// - `/spielen` is the game (web/index.html). The file keeps its name, so the
//   iOS app, which opens index.html from its bundle, is unchanged.
// - `/en` is the English landing page.
//
// The other static pages (/impressum, /presse, /start, /en/press, ...) need
// no entry: their address is the file name without `.html`, which Vite's
// own fallback and the service worker already resolve.

import type { Connect, Plugin } from 'vite';

/** Clean address (no trailing slash) to the build's file. */
export const ROUTES: Readonly<Record<string, string>> = {
  '/': '/start.html',
  '/en': '/en/start.html',
  '/spielen': '/index.html',
};

/** The file for a request path, or null if the path is not one of ours. */
export function routeFor(pathname: string): string | null {
  const path = pathname.length > 1 ? pathname.replace(/\/+$/, '') : pathname;
  return ROUTES[path === '' ? '/' : path] ?? null;
}

function rewrite(): Connect.NextHandleFunction {
  return (req, _res, next) => {
    const url = req.url ?? '/';
    const q = url.indexOf('?');
    const path = q < 0 ? url : url.slice(0, q);
    const file = routeFor(path);
    if (file) req.url = file + (q < 0 ? '' : url.slice(q));
    next();
  };
}

/** The same addresses in `vite dev` and `vite preview` as on the server. */
export function siteRoutes(): Plugin {
  return {
    name: 'stammtisch-site-routes',
    configureServer(server) {
      server.middlewares.use(rewrite());
    },
    configurePreviewServer(server) {
      server.middlewares.use(rewrite());
    },
  };
}

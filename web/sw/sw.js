// Service worker of Stammtisch: the game plays offline (DESIGN 14.5).
//
// This file is a template. The Vite plugin next to it (./plugin.ts)
// writes it to dist/sw.js after each build and fills in the build's
// version and the list of its files, so every deploy brings a new worker
// with its own cache. Not used by `vite dev`.
//
// Strategy:
// - Install: precache the whole build (index.html as "/", every hashed
//   file under /assets/ with the WebAssembly core, the card sprites, fonts
//   and sounds, the small icons and the manifest) into one cache per
//   version. A hashed file an older version already holds is copied from
//   its cache, so an update only downloads what changed. A failed download
//   fails the install; the old worker keeps serving and the browser tries
//   again later. The 512 px icons are left out: only installing the app
//   needs them, and that happens online.
// - /assets/*: cache first (the names are hashed, so a hit is never
//   stale), a miss goes to the network and is kept.
// - Navigations: network first with a short timeout, then the cached
//   shell. A page from the network is never written to the cache: the
//   cached shell must always be the one whose assets lie next to it. The
//   static pages beside the game (imprint, privacy policy, help, served
//   as /impressum for impressum.html) fall back to their own cached copy,
//   never to the game.
// - /api/*, /.well-known/* and other origins: never touched, never cached.
// - Updates: a new worker waits until the page asks it to take over
//   ({ type: 'skip-waiting' } after the player taps the toast). On
//   activate it deletes the caches of older versions but keeps the one
//   just before it, so a tab still running the previous version finds
//   its files even after the server deleted them.

const VERSION = '__SW_VERSION__';
const PRECACHE = self.__SW_PRECACHE__;
const PREFIX = 'stammtisch-';
const CACHE = `${PREFIX}${VERSION}`;
const SHELL = '/';
const NAV_TIMEOUT_MS = 3000;

self.addEventListener('install', (event) => {
  event.waitUntil(
    caches.open(CACHE).then((cache) => Promise.all(PRECACHE.map((url) => precache(cache, url)))),
  );
});

/** One file into the new cache: a hashed one from an older cache if it is there, else from the network. */
async function precache(cache, url) {
  if (url.startsWith('/assets/')) {
    // The name carries the content's hash: the same name is the same file.
    const kept = await caches.match(url, { ignoreSearch: true, ignoreVary: true });
    if (kept) {
      await cache.put(url, kept);
      return;
    }
  }
  // `reload` skips the HTTP cache, so the shell and its assets come from the same deploy.
  const response = await fetch(new Request(url, { cache: 'reload' }));
  if (!response.ok) throw new Error(`precache ${url}: HTTP ${response.status}`);
  await cache.put(url, response);
}

self.addEventListener('activate', (event) => {
  event.waitUntil(
    (async () => {
      const keys = (await caches.keys()).filter((k) => k.startsWith(PREFIX) && k !== CACHE);
      // caches.keys() lists in creation order: the last one is the version before.
      const previous = keys.pop();
      await Promise.all(keys.map((k) => caches.delete(k)));
      if (previous) await trimPrevious(previous);
      await self.clients.claim();
    })(),
  );
});

/** Of the version before, keep only the hashed assets (the shell would be stale). */
async function trimPrevious(name) {
  const cache = await caches.open(name);
  const requests = await cache.keys();
  await Promise.all(requests.filter((r) => !new URL(r.url).pathname.startsWith('/assets/')).map((r) => cache.delete(r)));
}

self.addEventListener('message', (event) => {
  if (event.data && event.data.type === 'skip-waiting') self.skipWaiting();
});

self.addEventListener('fetch', (event) => {
  const request = event.request;
  if (request.method !== 'GET') return;
  const url = new URL(request.url);
  if (url.origin !== self.location.origin) return;
  if (url.pathname === '/api' || url.pathname.startsWith('/api/')) return;
  if (url.pathname === '/sw.js') return;
  // Apple's app site association and the like (ADR 0025): always the server's.
  if (url.pathname.startsWith('/.well-known/')) return;
  if (request.mode === 'navigate') {
    event.respondWith(navigate(request));
    return;
  }
  if (url.pathname.startsWith('/assets/')) {
    event.respondWith(cacheFirst(request, true));
    return;
  }
  if (PRECACHE.includes(url.pathname)) event.respondWith(cacheFirst(request, false));
});

/** Hashed files: from any of our caches, else the network (kept for next time). */
async function cacheFirst(request, keep) {
  // ignoreVary: servers answer "Vary: Origin" or "Accept-Encoding", and a
  // module script asks with an Origin header the precache request lacked.
  const hit = await caches.match(request, { ignoreSearch: true, ignoreVary: true });
  if (hit) return hit;
  const response = await fetch(request);
  // Never keep a page under an asset's name (a server with an SPA fallback
  // answers a deleted file with index.html).
  const html = (response.headers.get('Content-Type') ?? '').includes('text/html');
  if (keep && response.ok && response.type === 'basic' && !html) {
    const copy = response.clone();
    caches.open(CACHE).then((cache) => cache.put(request, copy)).catch(() => {});
  }
  return response;
}

/** The page: fresh from the network if it answers in time, else the cached shell. */
async function navigate(request) {
  const network = fetch(request);
  // Answered late or failed after the shell was served: nobody waits for it.
  network.catch(() => {});
  let timer;
  const timeout = new Promise((resolve) => {
    timer = setTimeout(resolve, NAV_TIMEOUT_MS, null);
  });
  const page = cachedPage(new URL(request.url).pathname);
  try {
    const response = await Promise.race([network, timeout]);
    // A proxy error (502 while deploying) is no better than offline.
    if (response && (response.ok || !(await page))) return response;
  } catch {
    // Offline or the server is gone: the cached page.
  } finally {
    clearTimeout(timer);
  }
  return (await page) ?? network;
}

/** The cached answer for a navigation: a static page by its own name, else the shell. */
function cachedPage(pathname) {
  const own = pathname.endsWith('.html') ? pathname : `${pathname.replace(/\/$/, '')}.html`;
  const url = PRECACHE.includes(own) && own !== '/index.html' ? own : SHELL;
  return caches.open(CACHE).then((cache) => cache.match(url, { ignoreVary: true }));
}

// Offline play (DESIGN 14.5): the service worker (../sw/sw.js, written into
// the build by ../sw/plugin.ts) and whether the browser is online.
//
// Updates never reload on their own: a new worker waits, a small toast
// says "Neue Version verfügbar – neu laden", and only a tap lets it take
// over and reloads the page. A run in progress is in storage anyway, so
// the reload continues it.

import { de } from './i18n';

/** Whether the browser thinks it is online. */
export function isOnline(): boolean {
  return typeof navigator === 'undefined' || navigator.onLine !== false;
}

/** Call `listener` whenever the browser goes online or offline. */
export function watchNetwork(listener: (online: boolean) => void): void {
  window.addEventListener('online', () => listener(true));
  window.addEventListener('offline', () => listener(false));
}

let toastEl: HTMLElement | null = null;
let applyUpdate: () => void = () => {};

/** The page makes room for the bar (style.css: `html.has-update`) or takes it back. */
function makeRoom(on: boolean): void {
  document.documentElement.classList.toggle('has-update', on);
  // Positions measured on resize (hints, the table) follow.
  window.dispatchEvent(new Event('resize'));
}

/**
 * The slim bar that offers the new version; `apply` runs on a tap. It
 * sits above the app, which moves down by its height, so it never covers
 * a control. The small cross puts it away until the next visit.
 */
function showUpdateToast(apply: () => void): void {
  applyUpdate = apply;
  if (toastEl) return;
  const el = document.createElement('div');
  el.className = 'update-toast';
  el.setAttribute('role', 'status');
  const go = document.createElement('button');
  go.type = 'button';
  go.className = 'update-go';
  go.dataset.testid = 'update-toast';
  const long = document.createElement('span');
  long.className = 'update-long';
  long.textContent = de.offline.update;
  const short = document.createElement('span');
  short.className = 'update-short';
  short.textContent = de.offline.updateShort;
  go.append(long, short);
  go.setAttribute('aria-label', de.offline.updateLabel);
  go.onclick = () => applyUpdate();
  const later = document.createElement('button');
  later.type = 'button';
  later.className = 'update-later';
  later.dataset.testid = 'update-later';
  later.textContent = de.offline.updateLaterMark;
  later.setAttribute('aria-label', de.offline.updateLater);
  later.onclick = () => {
    el.remove();
    makeRoom(false);
  };
  el.append(go, later);
  document.body.append(el);
  toastEl = el;
  makeRoom(true);
}

/**
 * Register the service worker (production builds only; `vite dev` has
 * none) and offer updates as they come.
 */
export function registerServiceWorker(): void {
  if (!import.meta.env.PROD || !('serviceWorker' in navigator)) return;
  const sw = navigator.serviceWorker;
  // Did this tab ask the waiting worker to take over? Then reload as soon
  // as it does. Another tab's update only offers the reload here.
  let requested = false;
  // The first worker of a fresh visitor takes over without a reload; that is no update.
  const hadController = sw.controller !== null;
  const reload = (): void => window.location.reload();
  sw.addEventListener('controllerchange', () => {
    if (requested) reload();
    else if (hadController) showUpdateToast(reload);
  });

  const offer = (reg: ServiceWorkerRegistration): void => {
    if (!reg.waiting || !sw.controller) return;
    showUpdateToast(() => {
      const waiting = reg.waiting;
      if (!waiting) {
        reload();
        return;
      }
      requested = true;
      waiting.postMessage({ type: 'skip-waiting' });
      // Should the switch not come (the worker died meanwhile), reload anyway.
      window.setTimeout(reload, 4000);
    });
  };

  const register = (): void => {
    sw.register('/sw.js', { scope: '/', updateViaCache: 'none' }).then(
      (reg) => {
        offer(reg);
        reg.addEventListener('updatefound', () => {
          const installing = reg.installing;
          installing?.addEventListener('statechange', () => {
            if (installing.state === 'installed') offer(reg);
          });
        });
        // An app left open for days: look for a new version when it comes back.
        const check = (): void => {
          if (document.visibilityState === 'visible' && isOnline()) reg.update().catch(() => {});
        };
        document.addEventListener('visibilitychange', check);
        window.addEventListener('online', check);
      },
      (err: unknown) => console.warn('service worker not registered', err),
    );
  };
  // After the page has loaded, so the precache does not compete with the first screen.
  if (document.readyState === 'complete') register();
  else window.addEventListener('load', register, { once: true });
}

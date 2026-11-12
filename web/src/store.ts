// The full version (ADR 0028): in the iOS app the game is a free demo (the
// first district with its boss); a non-consumable In-App Purchase
// (`de.onemoreskat.app.full`) unlocks the rest. In the browser and the PWA
// everything is free and always full: nothing here does anything there.
//
// The truth is StoreKit's (./native-store.ts, ios/App/App/StorePlugin.swift):
// it verifies the signed transactions on the device and caches them, so the
// answer comes offline too. A small cache in local storage
// (`stammtisch-full`, no dot: outside the save keys, never in a save code,
// an upload or a bug report's save) only makes the start quick; StoreKit's
// answer replaces it as soon as it comes, also when it takes the full
// version away again (a refund). Prices come from StoreKit, never from here.
//
// No game logic: the core knows nothing of this. ./app/fullversion.ts
// decides where the demo ends.

import { note } from './diag';
import { isNativeApp } from './platform';

const CACHE_KEY = 'stammtisch-full';

/** The product id in App Store Connect (docs/ios.md). */
export const FULL_VERSION_ID = 'de.onemoreskat.app.full';

/** What the App Store says of the full version: its localized name and price. */
export interface Offer {
  name: string;
  price: string;
}

export type PurchaseResult = 'purchased' | 'cancelled' | 'pending';

/** Why something with the store failed: `unavailable` (no product, offline), `cancelled`, `unverified`, `failed`. */
export function storeErrorCode(err: unknown): string {
  const code = (err as { code?: unknown } | null)?.code;
  return typeof code === 'string' && code ? code : 'failed';
}

function readCache(): boolean {
  try {
    return window.localStorage.getItem(CACHE_KEY) === '1';
  } catch {
    return false;
  }
}

function writeCache(full: boolean): void {
  try {
    if (full) window.localStorage.setItem(CACHE_KEY, '1');
    else window.localStorage.removeItem(CACHE_KEY);
  } catch {
    // A blocked storage: the next start asks StoreKit again.
  }
}

/** Whether the store matters here at all: only inside the iOS app. */
export function storeActive(): boolean {
  return isNativeApp();
}

let full = !storeActive() || readCache();
/** StoreKit has answered in this visit (else `full` is the cache). */
let confirmed = !storeActive();
const watchers: ((full: boolean) => void)[] = [];

/** Whether the whole game is open: always in the browser, in the app with the purchase. */
export function isFull(): boolean {
  return full;
}

/** For bug reports (ADR 0027): `web`, `full` or `demo`, and whether StoreKit confirmed it. No receipt, no transaction id. */
export function fullStatus(): string {
  if (!storeActive()) return 'web';
  return `${full ? 'full' : 'demo'}${confirmed ? '' : ' (cached)'}`;
}

/** Be told when the full version comes or goes. */
export function onFullChange(fn: (full: boolean) => void): void {
  watchers.push(fn);
}

function setFull(next: boolean): void {
  confirmed = true;
  writeCache(next);
  if (next === full) return;
  full = next;
  note('store', next ? 'full' : 'demo');
  for (const fn of watchers) {
    try {
      fn(next);
    } catch (err) {
      console.warn('full version watcher failed', err);
    }
  }
}

/**
 * The plugin's module. Never the plugin itself out of an async function:
 * Capacitor's plugin proxy answers every property, `then` too, so awaiting
 * it would call a native method "then".
 */
function plugin(): Promise<typeof import('./native-store')> {
  return import('./native-store');
}

/**
 * In the app: ask StoreKit for the entitlement and listen for changes
 * (another device, Family Sharing, an approved Ask to Buy, a refund).
 * Safe to call once at boot; never throws.
 */
export async function startStore(): Promise<void> {
  if (!storeActive()) return;
  try {
    const { Store: store } = await plugin();
    await store.addListener('entitlementChanged', (data) => setFull(data.full === true));
    const { full: now } = await store.entitlement();
    note('store', `entitlement ${now === true ? 'full' : 'demo'}`);
    setFull(now === true);
  } catch (err) {
    // No answer (an old app build without the plugin): the cache stays.
    console.warn('store not reachable', err);
  }
}

/** The full version's name and price from the App Store; null offline or when it is not there. */
export async function loadOffer(): Promise<Offer | null> {
  if (!storeActive()) return null;
  try {
    const p = await (await plugin()).Store.products();
    return typeof p.price === 'string' && p.price ? { name: p.name, price: p.price } : null;
  } catch (err) {
    console.warn('no offer', storeErrorCode(err));
    return null;
  }
}

/** Buy the full version. Throws with a code (`storeErrorCode`) when it failed. */
export async function buyFull(): Promise<PurchaseResult> {
  const result = await (await plugin()).Store.purchase();
  note('store', `purchase ${result.state}`);
  if (result.state === 'purchased' || result.full) setFull(true);
  return result.state;
}

/** "Käufe wiederherstellen": `AppStore.sync()`, then whether the full version is there. Throws with a code when it failed. */
export async function restoreFull(): Promise<boolean> {
  const { full: now } = await (await plugin()).Store.restore();
  setFull(now === true);
  return now === true;
}

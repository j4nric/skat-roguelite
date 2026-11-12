// Where the web app runs: in a browser, or inside the iOS app (a Capacitor
// shell around this very build, ADR 0026). The app loads the bundled files
// from `capacitor://localhost`, so a few things differ there, all of them
// transport or presentation, never game logic:
//
// - the account server is `https://one-more-skat.de/api`, reached with a
//   bearer token instead of the cookie (./account.ts);
// - passkeys go through iOS (`ASAuthorization`, ./native.ts) for the
//   relying party one-more-skat.de;
// - vibration uses the Taptic Engine; there is no service worker (the
//   files are in the app already).
//
// Capacitor puts its bridge into the page before any script runs, so this
// check needs no import and works on the first line of the app.

interface CapacitorGlobal {
  isNativePlatform?: () => boolean;
}

/** Whether the page runs inside the native app. */
export function isNativeApp(): boolean {
  try {
    const cap = (globalThis as { Capacitor?: CapacitorGlobal }).Capacitor;
    return cap?.isNativePlatform?.() === true;
  } catch {
    return false;
  }
}

// The game's address (ADR 0025). It moved from the sslip.io address to its
// own domain; the old address keeps serving the game, because whatever a
// browser stored there (localStorage, passkeys) belongs to that address
// and cannot be read anywhere else. There it shows "Wir sind umgezogen"
// and takes the account or the stored progress along in a link.
//
// The link carries what it moves in its fragment, which a browser never
// sends to a server: `#umzug=<code>` (a one-time move code of the account,
// ../server/src/moving.rs) or `#spielstand=<save code>` (./savecode.ts,
// without an account). The new address reads it once and removes it from
// the address bar.
//
// The iOS app (ADR 0026) runs at `capacitor://localhost`, but for links and
// passkeys it is one-more-skat.de.

import { isNativeApp } from './platform';

/** The canonical address of the game. */
export const HOME_ORIGIN = 'https://one-more-skat.de';
/** Its host, as the player reads it. */
export const HOME_HOST = 'one-more-skat.de';
/** Addresses the game moved away from. */
export const OLD_HOSTS: readonly string[] = ['stammtisch-62-83-23-90.sslip.io'];

const ACCOUNT_KEY = 'umzug';
const SAVE_KEY = 'spielstand';

function currentHost(): string {
  try {
    return window.location.hostname;
  } catch {
    return '';
  }
}

/** Whether the page runs on an address the game moved away from. */
export function onOldAddress(host: string = currentHost()): boolean {
  return OLD_HOSTS.includes(host.toLowerCase());
}

/** The origin to put into shared links: the new address when on an old one or in the app. */
export function publicOrigin(): string {
  return onOldAddress() || isNativeApp() ? HOME_ORIGIN : window.location.origin;
}

/** The host the passkeys made here belong to (their RP id): one-more-skat.de in the app. */
export function passkeyHost(): string {
  return isNativeApp() ? HOME_HOST : currentHost();
}

/** What a move link carries. */
export type MoveLink = { kind: 'account'; code: string } | { kind: 'save'; code: string };

/** The link to the new address that takes this along. */
export function moveUrl(move: MoveLink): string {
  const key = move.kind === 'account' ? ACCOUNT_KEY : SAVE_KEY;
  return `${HOME_ORIGIN}/#${key}=${encodeURIComponent(move.code)}`;
}

/** Read a move link's fragment (`#umzug=…`, `#spielstand=…`), else null. */
export function parseMoveHash(hash: string): MoveLink | null {
  const m = /^#?(umzug|spielstand)=(.+)$/s.exec(hash);
  if (!m) return null;
  let code: string;
  try {
    code = decodeURIComponent(m[2] ?? '').trim();
  } catch {
    return null;
  }
  if (!code) return null;
  return m[1] === ACCOUNT_KEY ? { kind: 'account', code } : { kind: 'save', code };
}

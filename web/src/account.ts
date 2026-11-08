// The account server's API (ADR 0021): passkeys, recovery code, name,
// save blob and daily board. Only transport here: what is stored is the
// save code of ./savecode.ts, the server never reads it, and no game logic
// lives on either side of this file.
//
// Passkeys use the browser's WebAuthn API. The server (webauthn-rs) sends
// its options as JSON with base64url strings; they are turned into the
// ArrayBuffers `navigator.credentials` wants, and the answer back into
// JSON.

/** The signed-in player as the server describes them. */
export interface AccountUser {
  name: string | null;
  passkeys: number;
  createdAt: number;
}

/** The stored save of the account. */
export interface ServerSave {
  version: number;
  updatedAt: number;
  data: string;
}

/** A row of the daily board. */
export interface DailyEntry {
  rank: number;
  name: string | null;
  character: string;
  won: boolean;
  section: number;
  series: number;
  stake: number;
  bestGame: number;
  me: boolean;
}

export interface DailyBoard {
  day: string;
  total: number;
  entries: DailyEntry[];
  me: DailyEntry | null;
  signedIn: boolean;
}

/** What a daily run reports. */
export interface DailyResult {
  day: string;
  character: string;
  won: boolean;
  section: number;
  series: number;
  stake: number;
  bestGame: number;
}

/**
 * A failed call: `offline` (no answer), `cancelled` (the player closed the
 * passkey dialog), `unsupported` (no WebAuthn here) or the server's code
 * (`conflict`, `code`, `name`, `unauthorized`, …).
 */
export class AccountError extends Error {
  constructor(
    readonly code: string,
    readonly status = 0,
    readonly body: unknown = null,
  ) {
    super(code);
  }
}

const BASE = '/api';

/** Calls give up after this long (the game never waits longer for the server). */
const TIMEOUT_MS = 8000;

function timeoutSignal(ms: number): AbortSignal | undefined {
  try {
    return typeof AbortSignal.timeout === 'function' ? AbortSignal.timeout(ms) : undefined;
  } catch {
    return undefined;
  }
}

async function call<T>(method: string, path: string, body?: unknown, opts: { keepalive?: boolean; timeout?: number } = {}): Promise<T> {
  let res: Response;
  try {
    const signal = timeoutSignal(opts.timeout ?? TIMEOUT_MS);
    res = await fetch(`${BASE}${path}`, {
      method,
      credentials: 'same-origin',
      headers: body === undefined ? {} : { 'Content-Type': 'application/json' },
      ...(body === undefined ? {} : { body: JSON.stringify(body) }),
      ...(signal ? { signal } : {}),
      keepalive: opts.keepalive ?? false,
    });
  } catch {
    throw new AccountError('offline');
  }
  let parsed: unknown = null;
  if (res.status !== 204) {
    try {
      parsed = await res.json();
    } catch {
      parsed = null;
    }
  }
  if (!res.ok) {
    const code = (parsed as { error?: unknown } | null)?.error;
    // A proxy without the server behind it (502/503/504) is as good as offline.
    const fallback = res.status >= 502 && res.status <= 504 ? 'offline' : `http${res.status}`;
    throw new AccountError(typeof code === 'string' ? code : fallback, res.status, parsed);
  }
  return parsed as T;
}

// ------------------------------------------------------------ base64url

function toB64u(buf: ArrayBuffer | ArrayBufferView): string {
  const bytes = buf instanceof ArrayBuffer ? new Uint8Array(buf) : new Uint8Array(buf.buffer, buf.byteOffset, buf.byteLength);
  let bin = '';
  for (let i = 0; i < bytes.length; i++) bin += String.fromCharCode(bytes[i] ?? 0);
  return btoa(bin).replace(/\+/g, '-').replace(/\//g, '_').replace(/=+$/, '');
}

function fromB64u(text: string): ArrayBuffer {
  const b64 = text.replace(/-/g, '+').replace(/_/g, '/');
  const bin = atob(b64 + '='.repeat((4 - (b64.length % 4)) % 4));
  const bytes = new Uint8Array(bin.length);
  for (let i = 0; i < bin.length; i++) bytes[i] = bin.charCodeAt(i);
  return bytes.buffer;
}

type Json = Record<string, unknown>;

function credentialList(list: unknown): PublicKeyCredentialDescriptor[] | undefined {
  if (!Array.isArray(list)) return undefined;
  return list.map((c: Json) => ({ ...(c as object), id: fromB64u(String(c.id)) }) as PublicKeyCredentialDescriptor);
}

function creationOptions(options: Json): CredentialCreationOptions {
  const pk = options.publicKey as Json;
  const user = pk.user as Json;
  const publicKey = {
    ...pk,
    challenge: fromB64u(String(pk.challenge)),
    user: { ...user, id: fromB64u(String(user.id)) },
  } as unknown as PublicKeyCredentialCreationOptions;
  const exclude = credentialList(pk.excludeCredentials);
  if (exclude) publicKey.excludeCredentials = exclude;
  return { publicKey };
}

function requestOptions(options: Json): CredentialRequestOptions {
  const pk = options.publicKey as Json;
  const publicKey = { ...pk, challenge: fromB64u(String(pk.challenge)) } as unknown as PublicKeyCredentialRequestOptions;
  const allow = credentialList(pk.allowCredentials);
  if (allow) publicKey.allowCredentials = allow;
  return { publicKey };
}

function extensionsOf(cred: PublicKeyCredential): Json {
  try {
    return cred.getClientExtensionResults() as Json;
  } catch {
    return {};
  }
}

function registrationJson(cred: PublicKeyCredential): Json {
  const r = cred.response as AuthenticatorAttestationResponse;
  const response: Json = {
    attestationObject: toB64u(r.attestationObject),
    clientDataJSON: toB64u(r.clientDataJSON),
  };
  try {
    if (typeof r.getTransports === 'function') response.transports = r.getTransports();
  } catch {
    // Not offered by this browser.
  }
  return { id: cred.id, rawId: toB64u(cred.rawId), type: cred.type, response, extensions: extensionsOf(cred) };
}

function assertionJson(cred: PublicKeyCredential): Json {
  const r = cred.response as AuthenticatorAssertionResponse;
  return {
    id: cred.id,
    rawId: toB64u(cred.rawId),
    type: cred.type,
    response: {
      authenticatorData: toB64u(r.authenticatorData),
      clientDataJSON: toB64u(r.clientDataJSON),
      signature: toB64u(r.signature),
      userHandle: r.userHandle ? toB64u(r.userHandle) : null,
    },
    extensions: extensionsOf(cred),
  };
}

/** Whether this browser can use passkeys at all. */
export function passkeysSupported(): boolean {
  try {
    return typeof window.PublicKeyCredential === 'function' && typeof navigator.credentials?.create === 'function' && window.isSecureContext;
  } catch {
    return false;
  }
}

function webauthnError(err: unknown): AccountError {
  if (err instanceof AccountError) return err;
  const name = err instanceof DOMException ? err.name : '';
  if (name === 'NotAllowedError' || name === 'AbortError') return new AccountError('cancelled');
  if (name === 'InvalidStateError') return new AccountError('passkey_exists');
  if (name === 'NotSupportedError' || name === 'SecurityError') return new AccountError('unsupported');
  return new AccountError('passkey');
}

async function createPasskey(options: Json): Promise<Json> {
  if (!passkeysSupported()) throw new AccountError('unsupported');
  try {
    const cred = (await navigator.credentials.create(creationOptions(options))) as PublicKeyCredential | null;
    if (!cred) throw new AccountError('cancelled');
    return registrationJson(cred);
  } catch (err) {
    throw webauthnError(err);
  }
}

async function usePasskey(options: Json): Promise<Json> {
  if (!passkeysSupported()) throw new AccountError('unsupported');
  try {
    const cred = (await navigator.credentials.get(requestOptions(options))) as PublicKeyCredential | null;
    if (!cred) throw new AccountError('cancelled');
    return assertionJson(cred);
  } catch (err) {
    throw webauthnError(err);
  }
}

type Started = { ceremony: string; options: Json };

// ------------------------------------------------------------------ API

/** The signed-in player, or null. */
export async function fetchMe(): Promise<AccountUser | null> {
  const res = await call<{ user: AccountUser | null }>('GET', '/me');
  return res.user;
}

/** Sign up with a new passkey; the recovery code comes back once. */
export async function register(name: string | null): Promise<{ user: AccountUser; recoveryCode: string }> {
  const start = await call<Started>('POST', '/register/start', name ? { name } : {});
  const credential = await createPasskey(start.options);
  return call('POST', '/register/finish', { ceremony: start.ceremony, credential });
}

/** Sign in with a passkey this device (or its password manager) has. */
export async function signIn(): Promise<{ user: AccountUser }> {
  const start = await call<Started>('POST', '/login/start');
  const credential = await usePasskey(start.options);
  return call('POST', '/login/finish', { ceremony: start.ceremony, credential });
}

/** Sign in with the recovery code; it is used up and a new one comes back. */
export function recover(code: string): Promise<{ user: AccountUser; recoveryCode: string }> {
  return call('POST', '/recover', { code });
}

/** Add a passkey on this device to the signed-in account. */
export async function addPasskey(): Promise<{ user: AccountUser }> {
  const start = await call<Started>('POST', '/passkey/start');
  const credential = await createPasskey(start.options);
  return call('POST', '/passkey/finish', { ceremony: start.ceremony, credential });
}

/** Replace the recovery code. */
export function renewRecoveryCode(): Promise<{ recoveryCode: string }> {
  return call('POST', '/recovery-code');
}

export function setName(name: string): Promise<{ user: AccountUser }> {
  return call('PUT', '/me/name', { name });
}

export async function signOut(): Promise<void> {
  await call('POST', '/logout');
}

export async function deleteAccount(): Promise<void> {
  await call('DELETE', '/account');
}

export async function fetchSave(timeout?: number): Promise<ServerSave | null> {
  const res = await call<{ save: ServerSave | null }>('GET', '/save', undefined, timeout ? { timeout } : {});
  return res.save;
}

/** Store the save on top of `ifVersion`; AccountError `conflict` if another device was faster. */
export function putSave(data: string, updatedAt: number, ifVersion: number | null, keepalive = false): Promise<{ version: number; updatedAt: number }> {
  return call('PUT', '/save', { data, updatedAt, ifVersion }, { keepalive });
}

export function submitDaily(result: DailyResult): Promise<{ stored: boolean; me: DailyEntry | null }> {
  return call('POST', '/daily', result);
}

export function fetchDailyBoard(day: string, limit = 10): Promise<DailyBoard> {
  return call('GET', `/daily/${encodeURIComponent(day)}?limit=${limit}`);
}

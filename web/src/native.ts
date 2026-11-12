// The native side of the iOS app (ADR 0026), loaded only there (dynamic
// import from ./account.ts and ./haptics.ts, so the web build never runs
// it). Transport and presentation only:
//
// - `NativeAccount` is the app's own small plugin
//   (ios/App/App/NativeAccountPlugin.swift): passkeys through
//   `ASAuthorizationPlatformPublicKeyCredentialProvider` for the relying
//   party the server names (one-more-skat.de), and the session token in
//   the Keychain. It takes the server's WebAuthn options as they come
//   (JSON, base64url) and answers in the JSON the server reads, exactly
//   what ./account.ts makes of a browser credential.
// - Vibration through @capacitor/haptics.

import { registerPlugin } from '@capacitor/core';
import { Haptics, ImpactStyle, NotificationType } from '@capacitor/haptics';
import type { Buzz } from './haptics';

type Json = Record<string, unknown>;

interface NativeAccountPlugin {
  /** A new passkey; `publicKey` as the server sent it. */
  create(options: { publicKey: Json }): Promise<Json>;
  /** Sign in with a passkey; `publicKey` as the server sent it. */
  get(options: { publicKey: Json }): Promise<Json>;
  /** The stored session token, or null. */
  readSession(): Promise<{ token: string | null }>;
  /** Store the session token; null removes it. */
  writeSession(options: { token: string | null }): Promise<void>;
}

const NativeAccount = registerPlugin<NativeAccountPlugin>('NativeAccount');

/** The error code the plugin rejected with (`cancelled`, `passkey_exists`, `unsupported`, `passkey`). */
export function nativeErrorCode(err: unknown): string {
  const code = (err as { code?: unknown } | null)?.code;
  return typeof code === 'string' && code ? code : 'passkey';
}

export function createPasskey(publicKey: Json): Promise<Json> {
  return NativeAccount.create({ publicKey });
}

export function usePasskey(publicKey: Json): Promise<Json> {
  return NativeAccount.get({ publicKey });
}

export async function readSession(): Promise<string | null> {
  const { token } = await NativeAccount.readSession();
  return typeof token === 'string' && token ? token : null;
}

export function writeSession(token: string | null): Promise<void> {
  return NativeAccount.writeSession({ token });
}

/** One tap for a trick, the system's success or error feel for the rest. */
export function haptic(kind: Buzz): Promise<void> {
  switch (kind) {
    case 'trick':
      return Haptics.impact({ style: ImpactStyle.Light });
    case 'gameWon':
    case 'pubWon':
    case 'runWon':
      return Haptics.notification({ type: NotificationType.Success });
    case 'gameLost':
      return Haptics.notification({ type: NotificationType.Warning });
    case 'pubLost':
      return Haptics.notification({ type: NotificationType.Error });
  }
}

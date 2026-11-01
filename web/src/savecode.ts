// The save code: everything the app keeps in localStorage (the serialized
// core states of run and meta, plus settings, character, run info and the
// list of recent runs) as one line of text the player can keep somewhere
// else, so a cleared browser or a new address does not cost the progress.
//
//   STAMMTISCH1:<enc>:<payload>:<crc>
//
// - `STAMMTISCH1` names the format of the envelope (version 1).
// - `<enc>` is `d` (payload compressed with deflate-raw) or `j` (plain).
// - `<payload>` is base64url (no padding) of the UTF-8 bytes of the JSON
//   `{"v":1,"keys":{"stammtisch.run":"…",…}}`, compressed if `<enc>` is `d`.
// - `<crc>` is the CRC-32 (IEEE) of `<enc>:<payload>` as eight hex digits,
//   so a typo or a cut-off paste is caught before anything is touched.
//
// The code carries the stored strings as they are; whether the core can
// read them is checked by the app with the core's own load functions.

const PREFIX = 'STAMMTISCH';
const FORMAT = 1;
const CONTENT_VERSION = 1;

/** Why a pasted code was rejected. */
export type SaveCodeError = 'empty' | 'notACode' | 'newer' | 'damaged' | 'unsupported';

export type DecodeResult = { ok: true; keys: Record<string, string> } | { ok: false; error: SaveCodeError };

// ---------------------------------------------------------------- CRC-32

let crcTable: Uint32Array | null = null;

function crc32(text: string): string {
  if (!crcTable) {
    crcTable = new Uint32Array(256);
    for (let n = 0; n < 256; n++) {
      let c = n;
      for (let k = 0; k < 8; k++) c = c & 1 ? 0xedb88320 ^ (c >>> 1) : c >>> 1;
      crcTable[n] = c >>> 0;
    }
  }
  let crc = 0xffffffff;
  for (let i = 0; i < text.length; i++) {
    crc = (crcTable[(crc ^ text.charCodeAt(i)) & 0xff] ?? 0) ^ (crc >>> 8);
  }
  return ((crc ^ 0xffffffff) >>> 0).toString(16).padStart(8, '0');
}

// ------------------------------------------------------------- base64url

function toBase64Url(bytes: Uint8Array): string {
  let binary = '';
  const chunk = 0x8000;
  for (let i = 0; i < bytes.length; i += chunk) {
    binary += String.fromCharCode(...bytes.subarray(i, i + chunk));
  }
  return btoa(binary).replace(/\+/g, '-').replace(/\//g, '_').replace(/=+$/, '');
}

function fromBase64Url(text: string): Uint8Array {
  const b64 = text.replace(/-/g, '+').replace(/_/g, '/');
  const binary = atob(b64 + '='.repeat((4 - (b64.length % 4)) % 4));
  const bytes = new Uint8Array(binary.length);
  for (let i = 0; i < binary.length; i++) bytes[i] = binary.charCodeAt(i);
  return bytes;
}

// ----------------------------------------------------------- compression

function canCompress(): boolean {
  return typeof CompressionStream === 'function' && typeof DecompressionStream === 'function';
}

async function pipe(bytes: Uint8Array, stream: CompressionStream | DecompressionStream): Promise<Uint8Array> {
  const body = new Blob([bytes as Uint8Array<ArrayBuffer>]).stream().pipeThrough(stream);
  return new Uint8Array(await new Response(body).arrayBuffer());
}

// ----------------------------------------------------------------- codec

/** Make the code for the given stored keys. */
export async function encodeSave(keys: Record<string, string>): Promise<string> {
  const json = new TextEncoder().encode(JSON.stringify({ v: CONTENT_VERSION, keys }));
  let enc = 'j';
  let bytes: Uint8Array = json;
  if (canCompress()) {
    try {
      bytes = await pipe(json, new CompressionStream('deflate-raw'));
      enc = 'd';
    } catch {
      bytes = json;
      enc = 'j';
    }
  }
  const body = `${enc}:${toBase64Url(bytes)}`;
  return `${PREFIX}${FORMAT}:${body}:${crc32(body)}`;
}

/**
 * Read a pasted code. Whitespace and line breaks are ignored, and so is
 * text around the code (a messenger may add a title in front).
 */
export async function decodeSave(input: string): Promise<DecodeResult> {
  const text = input.replace(/\s+/g, '');
  if (!text) return { ok: false, error: 'empty' };
  const start = text.indexOf(PREFIX);
  if (start < 0) return { ok: false, error: 'notACode' };
  const match = /^STAMMTISCH(\d+):([a-z]):([A-Za-z0-9_-]*):([0-9a-fA-F]{8})/.exec(text.slice(start));
  const format = /^STAMMTISCH(\d+):/.exec(text.slice(start));
  if (format && Number(format[1]) > FORMAT) return { ok: false, error: 'newer' };
  if (!match || Number(match[1]) !== FORMAT) return { ok: false, error: 'damaged' };
  const [, , enc = '', payload = '', crc = ''] = match;
  if (crc32(`${enc}:${payload}`) !== crc.toLowerCase()) return { ok: false, error: 'damaged' };
  let bytes: Uint8Array;
  try {
    bytes = fromBase64Url(payload);
  } catch {
    return { ok: false, error: 'damaged' };
  }
  if (enc === 'd') {
    if (!canCompress()) return { ok: false, error: 'unsupported' };
    try {
      bytes = await pipe(bytes, new DecompressionStream('deflate-raw'));
    } catch {
      return { ok: false, error: 'damaged' };
    }
  } else if (enc !== 'j') {
    return { ok: false, error: 'newer' };
  }
  let parsed: unknown;
  try {
    parsed = JSON.parse(new TextDecoder('utf-8', { fatal: true }).decode(bytes));
  } catch {
    return { ok: false, error: 'damaged' };
  }
  if (typeof parsed !== 'object' || parsed === null) return { ok: false, error: 'damaged' };
  const content = parsed as { v?: unknown; keys?: unknown };
  if (typeof content.v !== 'number') return { ok: false, error: 'damaged' };
  if (content.v > CONTENT_VERSION) return { ok: false, error: 'newer' };
  if (typeof content.keys !== 'object' || content.keys === null || Array.isArray(content.keys)) {
    return { ok: false, error: 'damaged' };
  }
  const keys: Record<string, string> = {};
  for (const [key, value] of Object.entries(content.keys as Record<string, unknown>)) {
    if (typeof value !== 'string') return { ok: false, error: 'damaged' };
    keys[key] = value;
  }
  return { ok: true, keys };
}

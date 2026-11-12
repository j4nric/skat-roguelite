// Bug reports ("Fehler melden", ADR 0027): what goes into one, and the
// queue for reports made offline. Only transport; the server stores the
// report as an opaque text and never reads it.
//
// A report is JSON in a string: what the player wrote, the device's facts
// (platform, build, language, tempo, reduced motion, viewport), where the
// app stands, the diagnosis buffer of this visit and of the last one
// (./diag.ts) and, if the player leaves the box ticked, the save code
// (./savecode.ts, the same text as "Spielstand sichern"). No account name,
// no e-mail, no token, no passkey data: signed in, the server notes the
// account by the session the request carries anyway.

import { AccountError, submitReport, type ReportUpload } from './account';
import { diagEntries, hadTrouble, previousSession, sessionStart, viewportLine, type DiagEntry } from './diag';
import { lang } from './i18n';
import { isNativeApp } from './platform';
import { encodeSave } from './savecode';
import type { Settings } from './settings';
import { readAllKeys } from './storage';

/** Format of the report JSON. */
export const REPORT_FORMAT = 1;
/** Longest text the player can write. */
export const MESSAGE_CHARS = 2000;
/** The report stays below this many bytes (the server takes 250 000). */
const REPORT_BYTES = 240_000;
/** A save code longer than this stays home (the report says so). */
const SAVE_CHARS = 180_000;
const QUEUE_KEY = 'stammtisch-diag.queue';
/** At most this many reports wait offline (the oldest goes). */
const QUEUE_SIZE = 2;

const BUILD = typeof __BUILD_INFO__ === 'string' ? __BUILD_INFO__ : '';

/** `web`, `pwa` (installed to the home screen) or `ios` (the app, ADR 0026). */
export function platformName(): string {
  if (isNativeApp()) return 'ios';
  try {
    if (window.matchMedia('(display-mode: standalone)').matches || (navigator as { standalone?: boolean }).standalone === true) return 'pwa';
  } catch {
    // No media queries: a plain browser.
  }
  return 'web';
}

function prefersReducedMotion(): boolean {
  try {
    return window.matchMedia('(prefers-reduced-motion: reduce)').matches;
  } catch {
    return false;
  }
}

/** What the app knows of itself when the report is made. */
export interface ReportContext {
  settings: Settings;
  /** Where the app stands (the same line the buffer notes). */
  state: string;
  signedIn: boolean;
}

/** The report's JSON and what was left out. */
export interface BuiltReport {
  upload: ReportUpload;
  /** The save code went along. */
  withSave: boolean;
}

function bytes(text: string): number {
  try {
    return new TextEncoder().encode(text).length;
  } catch {
    return text.length * 3;
  }
}

/** Put a report together; the save code only if `withSave` and it fits. */
export async function buildReport(message: string, withSave: boolean, ctx: ReportContext): Promise<BuiltReport> {
  let save: string | null = null;
  let saveNote: string | null = withSave ? null : 'off';
  if (withSave) {
    try {
      save = await encodeSave(readAllKeys());
      if (save.length > SAVE_CHARS) {
        save = null;
        saveNote = 'too_large';
      }
    } catch {
      saveNote = 'failed';
    }
  }
  const s = ctx.settings;
  const prev = previousSession();
  let log: DiagEntry[] = diagEntries();
  let previous = prev ? { ...prev } : null;
  const make = (): string =>
    JSON.stringify({
      format: REPORT_FORMAT,
      message: message.slice(0, MESSAGE_CHARS),
      madeAt: new Date().toISOString(),
      app: {
        platform: platformName(),
        build: BUILD,
        lang: lang(),
        userAgent: navigator.userAgent,
        online: navigator.onLine !== false,
        signedIn: ctx.signedIn,
      },
      settings: { tempo: s.tempo, reducedMotion: prefersReducedMotion(), sound: s.sound, music: s.music, hints: s.hints, textSize: s.textSize, colorBlind: s.colorBlind, handSort: s.handSort, vibrate: s.vibrate },
      viewport: viewportLine(),
      state: ctx.state,
      session: { startedAt: new Date(sessionStart()).toISOString(), trouble: hadTrouble(log) },
      log,
      previous: previous ? { startedAt: new Date(previous.startedAt).toISOString(), savedAt: new Date(previous.savedAt).toISOString(), build: previous.build, trouble: hadTrouble(previous.entries), log: previous.entries } : null,
      save,
      saveNote,
    });
  let text = make();
  // Too long: the last visit's buffer first, then this one's oldest lines, then the save.
  while (bytes(text) > REPORT_BYTES) {
    if (previous && previous.entries.length > 0) previous = { ...previous, entries: previous.entries.slice(Math.ceil(previous.entries.length / 2)) };
    else if (log.length > 20) log = log.slice(Math.ceil(log.length / 4));
    else if (save) {
      save = null;
      saveNote = 'too_large';
    } else break;
    text = make();
  }
  return { upload: { platform: platformName(), version: BUILD || 'unknown', report: text }, withSave: save !== null };
}

/** How sending went: the id, waiting in the queue, or failed for good. */
export type SendResult = { ok: true; id: string } | { ok: false; queued: boolean; code: string };

/** Errors after which a report waits for a better moment instead of failing. */
function worthRetrying(code: string): boolean {
  return code === 'offline' || code === 'rate_limited' || code === 'busy' || code === 'internal' || code === 'storage_full';
}

/** Send a report; offline (or the server busy) it goes into the queue. */
export async function sendReport(upload: ReportUpload): Promise<SendResult> {
  try {
    const { id } = await submitReport(upload);
    return { ok: true, id };
  } catch (err) {
    const code = err instanceof AccountError ? err.code : 'offline';
    if (!worthRetrying(code)) return { ok: false, queued: false, code };
    return { ok: false, queued: enqueue(upload), code };
  }
}

interface Queued extends ReportUpload {
  queuedAt: number;
}

function readQueue(): Queued[] {
  try {
    const raw = window.localStorage.getItem(QUEUE_KEY);
    const list = raw ? (JSON.parse(raw) as unknown) : [];
    return Array.isArray(list) ? list.filter((q): q is Queued => typeof q === 'object' && q !== null && typeof (q as Queued).report === 'string') : [];
  } catch {
    return [];
  }
}

function writeQueue(list: Queued[]): boolean {
  try {
    if (list.length) window.localStorage.setItem(QUEUE_KEY, JSON.stringify(list));
    else window.localStorage.removeItem(QUEUE_KEY);
    return true;
  } catch {
    return false;
  }
}

function enqueue(upload: ReportUpload): boolean {
  const list = [...readQueue(), { ...upload, queuedAt: Date.now() }].slice(-QUEUE_SIZE);
  return writeQueue(list);
}

/** How many reports wait. */
export function queuedReports(): number {
  return readQueue().length;
}

let flushing = false;

/**
 * Send what waits, the oldest first; stops at the first that cannot go
 * yet. Returns the ids of those sent.
 */
export async function flushReports(): Promise<string[]> {
  if (flushing) return [];
  flushing = true;
  const sent: string[] = [];
  try {
    for (;;) {
      const [first] = readQueue();
      if (!first) break;
      let id: string | null = null;
      let drop = false;
      try {
        id = (await submitReport({ platform: first.platform, version: first.version, report: first.report })).id;
      } catch (err) {
        const code = err instanceof AccountError ? err.code : 'offline';
        // A report the server will never take (too large, malformed) goes.
        drop = !worthRetrying(code);
        if (!drop) break;
      }
      writeQueue(readQueue().filter((q) => q.queuedAt !== first.queuedAt || q.report !== first.report));
      if (id) sent.push(id);
    }
  } finally {
    flushing = false;
  }
  return sent;
}

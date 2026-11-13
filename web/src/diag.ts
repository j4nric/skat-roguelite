// The diagnosis buffer for bug reports ("Fehler melden", ADR 0027).
//
// A ring of the last entries of this visit: screens and phases, taps on
// `data-action` buttons, errors (window.onerror, unhandled rejections,
// console.error and console.warn), taps after which nothing changed
// ("dead taps", only noted, never acted on), presses that never became a
// click, the viewport, going hidden or offline. A report sends it along
// with the device's facts (./report.ts).
//
// Only presentation and transport facts, never personal data: no text the
// player types (only taps on buttons are noted, by their action name), no
// account name or e-mail, no session token, no passkey data, no address
// fragment (a move link carries a code there, ADR 0025). Long tokens,
// save codes and e-mail addresses in error texts are cut out (`scrub`).
//
// The buffer lives in memory; a shortened copy goes to localStorage
// (`stammtisch-diag.last`, not a save key: it never travels with the save
// code or the account), so a report after a restart still has what the
// last visit saw ("Beim letzten Mal hing etwas?").

/**
 * What an entry is about. `store`: the full version of the iOS app
 * (ADR 0028), only its state (`demo`, `full`, a purchase cancelled or
 * pending), never a receipt or a transaction id.
 */
export type DiagKind = 'start' | 'screen' | 'tap' | 'dead-tap' | 'no-click' | 'error' | 'rejection' | 'console' | 'viewport' | 'visibility' | 'net' | 'store';

/** One line of the buffer: ms since the visit began, kind, text. */
export interface DiagEntry {
  t: number;
  k: DiagKind;
  m: string;
}

/** The last visit's buffer, as stored. */
export interface DiagSession {
  /** When that visit began (ms since 1970, the device's clock). */
  startedAt: number;
  /** When it was stored last. */
  savedAt: number;
  /** Its build. */
  build: string;
  entries: DiagEntry[];
}

/** What the app tells the buffer about itself. */
export interface DiagProbe {
  /** A short line of where the app is: screen, phase, dialog, busy. Compared to note changes. */
  state(): string;
}

/** Entries kept in memory. */
export const DIAG_SIZE = 200;
/** Entries kept in localStorage, and at most this many characters as JSON. */
const STORED_SIZE = 120;
const STORED_CHARS = 40_000;
/** A tap after which nothing changes for this long is noted as dead. */
export const DEAD_TAP_MS = 1500;
/** A press that does not become a click within this long is noted. */
const NO_CLICK_MS = 700;
const LAST_KEY = 'stammtisch-diag.last';

const BUILD = typeof __BUILD_INFO__ === 'string' ? __BUILD_INFO__ : '';

const entries: DiagEntry[] = [];
const startedAt = Date.now();
const startedPerf = typeof performance !== 'undefined' ? performance.now() : 0;
let previous: DiagSession | null = null;
let installed = false;
let probe: DiagProbe | null = null;
let lastState = '';
/** DOM changes in the screen and the dialog layer since the visit began. */
let mutations = 0;
let saveTimer: number | undefined;

function now(): number {
  return Math.round((typeof performance !== 'undefined' ? performance.now() : 0) - startedPerf);
}

/** Cut what could identify someone or open something out of a text. */
export function scrub(text: string): string {
  return text
    .replace(/STAMMTISCH\d+:\S*/g, 'STAMMTISCH…')
    .replace(/#(umzug|spielstand)=\S*/gi, '#$1=…')
    .replace(/Bearer\s+\S+/gi, 'Bearer …')
    .replace(/[^\s@"'<>]+@[^\s@"'<>]+\.[a-z]{2,}/gi, '…@…')
    .replace(/[A-Za-z0-9_\-+/=]{32,}/g, '…');
}

function clip(text: string, max: number): string {
  return text.length > max ? `${text.slice(0, max)}…` : text;
}

/** Add an entry (texts are scrubbed and cut). */
export function note(k: DiagKind, m: string, max = 300): void {
  entries.push({ t: now(), k, m: clip(scrub(m), max) });
  if (entries.length > DIAG_SIZE) entries.splice(0, entries.length - DIAG_SIZE);
  scheduleSave();
}

/** The entries of this visit (a copy). */
export function diagEntries(): DiagEntry[] {
  return entries.slice();
}

/** The last visit's buffer, if one was stored. */
export function previousSession(): DiagSession | null {
  return previous;
}

/** When this visit began. */
export function sessionStart(): number {
  return startedAt;
}

/** Whether a buffer holds trouble: an error or a dead tap. */
export function hadTrouble(list: readonly DiagEntry[]): boolean {
  return list.some((e) => e.k === 'error' || e.k === 'rejection' || e.k === 'dead-tap' || e.k === 'no-click');
}

function readPrevious(): DiagSession | null {
  try {
    const raw = window.localStorage.getItem(LAST_KEY);
    if (!raw) return null;
    const parsed = JSON.parse(raw) as Partial<DiagSession>;
    if (!Array.isArray(parsed.entries) || typeof parsed.startedAt !== 'number') return null;
    const list = parsed.entries
      .filter((e): e is DiagEntry => typeof e === 'object' && e !== null && typeof (e as DiagEntry).m === 'string' && typeof (e as DiagEntry).t === 'number')
      .slice(-STORED_SIZE);
    return { startedAt: parsed.startedAt, savedAt: typeof parsed.savedAt === 'number' ? parsed.savedAt : parsed.startedAt, build: typeof parsed.build === 'string' ? parsed.build : '', entries: list };
  } catch {
    return null;
  }
}

/** Store the shortened buffer now. */
export function saveDiag(): void {
  window.clearTimeout(saveTimer);
  saveTimer = undefined;
  try {
    let list = entries.slice(-STORED_SIZE);
    let text = '';
    for (;;) {
      text = JSON.stringify({ startedAt, savedAt: Date.now(), build: BUILD, entries: list } satisfies DiagSession);
      if (text.length <= STORED_CHARS || list.length < 10) break;
      list = list.slice(Math.ceil(list.length / 4));
    }
    window.localStorage.setItem(LAST_KEY, text);
  } catch {
    // Storage full or blocked: the buffer stays in memory only.
  }
}

function scheduleSave(): void {
  if (!installed || saveTimer !== undefined) return;
  try {
    saveTimer = window.setTimeout(saveDiag, 1000);
  } catch {
    saveTimer = undefined;
  }
}

/** A short name of an element: its action, test id or tag and class. */
export function describe(el: Element | null): string {
  if (!el) return '-';
  const action = el.closest('[data-action]');
  if (action instanceof HTMLElement) {
    const id = action.dataset.testid ? `#${action.dataset.testid}` : '';
    return `${action.dataset.action ?? ''}${id}`;
  }
  const cls = typeof el.className === 'string' && el.className ? `.${el.className.trim().split(/\s+/).slice(0, 2).join('.')}` : '';
  return `${el.tagName.toLowerCase()}${cls}`;
}

function errorText(err: unknown): string {
  if (err instanceof Error) return `${err.name}: ${err.message}${err.stack ? `\n${err.stack}` : ''}`;
  try {
    return typeof err === 'string' ? err : JSON.stringify(err);
  } catch {
    return String(err);
  }
}

function argsText(args: unknown[]): string {
  return args.map((a) => (typeof a === 'string' ? a : errorText(a))).join(' ');
}

/** The viewport, the safe area and the device pixel ratio, in one line. */
export function viewportLine(): string {
  try {
    const vv = window.visualViewport;
    const inset = safeArea();
    const orient = screen.orientation?.type ?? '';
    return `${window.innerWidth}x${window.innerHeight}${vv ? ` vv ${Math.round(vv.width)}x${Math.round(vv.height)}@${vv.scale.toFixed(2)}` : ''} dpr ${window.devicePixelRatio} safe ${inset} ${orient}`.trim();
  } catch {
    return '?';
  }
}

/** The safe-area insets (top right bottom left, px) as CSS sees them. */
function safeArea(): string {
  try {
    const el = document.createElement('div');
    el.style.cssText = 'position:fixed;visibility:hidden;pointer-events:none;padding:env(safe-area-inset-top) env(safe-area-inset-right) env(safe-area-inset-bottom) env(safe-area-inset-left)';
    document.body.appendChild(el);
    const cs = getComputedStyle(el);
    const out = [cs.paddingTop, cs.paddingRight, cs.paddingBottom, cs.paddingLeft].map((p) => String(Math.round(parseFloat(p) || 0))).join(' ');
    el.remove();
    return out;
  } catch {
    return '?';
  }
}

/**
 * The app drew itself anew: note where it is when that changed
 * (screen, phase, dialog). Cheap, called with every render.
 */
export function noteRender(): void {
  if (!probe) return;
  let state: string;
  try {
    state = probe.state();
  } catch {
    return;
  }
  if (state === lastState) return;
  lastState = state;
  note('screen', state);
}

/** Watch taps on `data-action` buttons: note them, and those after which nothing happens. */
function watchTaps(root: HTMLElement): void {
  // A press on a button that never becomes a click (a covered button, a
  // touch the browser took for a scroll): what lay under the finger.
  let pressed: { el: HTMLElement; x: number; y: number } | null = null;
  root.addEventListener(
    'click',
    (e) => {
      const el = (e.target as Element | null)?.closest?.('[data-action]');
      if (!(el instanceof HTMLElement)) return;
      if (el instanceof HTMLButtonElement && el.disabled) return;
      const name = describe(el);
      pressed = null;
      note('tap', name);
      // Text fields and links do their work outside the app's state.
      if (el.matches('textarea, input, a[href]')) return;
      const before = probe?.state() ?? '';
      const count = mutations;
      window.setTimeout(() => {
        if (!el.isConnected || mutations !== count) return;
        const after = probe?.state() ?? '';
        if (after === before) note('dead-tap', `${name} · ${after}`);
      }, DEAD_TAP_MS);
    },
    { capture: true },
  );
  root.addEventListener(
    'pointerdown',
    (e) => {
      const el = (e.target as Element | null)?.closest?.('[data-action]');
      pressed = el instanceof HTMLElement && !el.matches('[data-drag], textarea, input') ? { el, x: e.clientX, y: e.clientY } : null;
    },
    { capture: true, passive: true },
  );
  root.addEventListener(
    'pointerup',
    (e) => {
      const p = pressed;
      if (!p || Math.hypot(e.clientX - p.x, e.clientY - p.y) > 12) return;
      window.setTimeout(() => {
        if (pressed !== p) return;
        pressed = null;
        const under = document.elementFromPoint(e.clientX, e.clientY);
        note('no-click', `${describe(p.el)} · under ${describe(under)}`);
      }, NO_CLICK_MS);
    },
    { capture: true, passive: true },
  );
}

/**
 * Start the buffer, first thing at boot: read the last visit's, hook
 * errors, the console, the viewport, the page's visibility and the network.
 */
export function startDiag(): void {
  if (installed) return;
  installed = true;
  previous = readPrevious();
  note('start', `${BUILD} · ${viewportLine()}`);
  window.addEventListener('error', (e) => {
    const where = e.filename ? ` @ ${e.filename.split('/').pop() ?? ''}:${e.lineno}:${e.colno}` : '';
    note('error', `${e.error ? errorText(e.error) : e.message}${where}`, 1500);
    saveDiag();
  });
  window.addEventListener('unhandledrejection', (e) => {
    note('rejection', errorText(e.reason), 1500);
    saveDiag();
  });
  for (const level of ['error', 'warn'] as const) {
    const original = console[level].bind(console);
    console[level] = (...args: unknown[]) => {
      try {
        note('console', `${level}: ${argsText(args)}`, level === 'error' ? 1500 : 500);
      } catch {
        // Never let the buffer break a log line.
      }
      original(...args);
    };
  }
  let resizeTimer: number | undefined;
  window.addEventListener('resize', () => {
    window.clearTimeout(resizeTimer);
    resizeTimer = window.setTimeout(() => note('viewport', viewportLine()), 400);
  });
  document.addEventListener('visibilitychange', () => {
    note('visibility', document.visibilityState);
    if (document.visibilityState === 'hidden') saveDiag();
  });
  window.addEventListener('pagehide', saveDiag);
  window.addEventListener('online', () => note('net', 'online'));
  window.addEventListener('offline', () => note('net', 'offline'));
  saveDiag();
}

/**
 * Watch the app once it is mounted: taps on its buttons, and where it
 * stands (`p`). `observe` are the elements whose changes count as
 * "something happened" after a tap (the screen and the dialog layer).
 */
export function watchApp(root: HTMLElement, observe: HTMLElement[], p: DiagProbe): void {
  startDiag();
  if (probe) return;
  probe = p;
  try {
    const mo = new MutationObserver((records) => {
      mutations += records.length;
    });
    for (const el of observe) mo.observe(el, { childList: true, subtree: true, characterData: true });
  } catch {
    // No MutationObserver: dead taps are not noted.
  }
  watchTaps(root);
}

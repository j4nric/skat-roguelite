// Shared helpers of the store scripts (screenshots and preview video).
//
// Everything runs against the built web app (web/dist) served by
// `vite preview`, with Playwright's Chromium. Runs come from the saved
// core states in web/e2e/fixtures/ (the same the smoke test uses), so
// every scene is reproducible: fixed seeds, sound off, hints off, the
// Einführung declined, the service worker blocked and the clock fixed.

import { spawn } from 'node:child_process';
import { createRequire } from 'node:module';
import { readFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const require = createRequire(import.meta.url);
export const { chromium } = require('playwright');

export const storeDir = dirname(fileURLToPath(import.meta.url));
export const rootDir = resolve(storeDir, '..');
export const webDir = join(rootDir, 'web');
export const fixturesDir = join(webDir, 'e2e', 'fixtures');

/** A fixed moment (Monday 16.11.2026, 19:30 CET), so daily and weekly blocks do not change. */
export const FIXED_TIME = new Date('2026-11-16T18:30:00Z');

export function fixture(name) {
  return readFileSync(join(fixturesDir, name), 'utf8');
}

/** A fixture with `change` applied to its parsed save, as JSON again. */
export function patched(name, change) {
  const save = JSON.parse(fixture(name));
  change?.(save);
  return JSON.stringify(save);
}

/** Start `vite preview` of web/dist on `port`; resolves with the child once it answers. */
export async function startPreview(port) {
  const vite = join(webDir, 'node_modules', '.bin', 'vite');
  const child = spawn(vite, ['preview', '--port', String(port), '--strictPort'], { cwd: webDir, stdio: ['ignore', 'pipe', 'pipe'] });
  let output = '';
  child.stdout.on('data', (d) => (output += d));
  child.stderr.on('data', (d) => (output += d));
  const base = `http://localhost:${port}/`;
  for (let i = 0; i < 150; i++) {
    try {
      const res = await fetch(base);
      if (res.ok) return { child, base };
    } catch {
      // not up yet
    }
    if (child.exitCode !== null) throw new Error(`vite preview exited:\n${output}`);
    await new Promise((r) => setTimeout(r, 100));
  }
  child.kill();
  throw new Error(`vite preview did not start:\n${output}`);
}

/**
 * A browser context of the app in `lang` ('de' or 'en'): quiet settings,
 * the Einführung declined, no service worker, the account API answered
 * locally (no leaderboard entries), the clock fixed.
 */
export async function appContext(browser, { lang = 'de', viewport, scale = 3, motion = 'reduce', settings = {}, meta = null, tempo = 'normal' }) {
  const context = await browser.newContext({
    viewport,
    deviceScaleFactor: scale,
    reducedMotion: motion,
    locale: lang === 'de' ? 'de-DE' : 'en-US',
    serviceWorkers: 'block',
    hasTouch: false,
  });
  await context.addInitScript(
    ([s, l, m]) => {
      try {
        if (!localStorage.getItem('stammtisch.settings')) localStorage.setItem('stammtisch.settings', JSON.stringify(s));
        if (!localStorage.getItem('stammtisch.tutorial')) localStorage.setItem('stammtisch.tutorial', 'declined');
        localStorage.setItem('stammtisch.lang', l);
        if (m && !localStorage.getItem('stammtisch.meta')) localStorage.setItem('stammtisch.meta', m);
      } catch {
        // storage blocked: nothing to set
      }
    },
    [{ tempo, sound: false, music: false, thinking: false, hints: false, vibrate: false, ...settings }, lang, meta],
  );
  await context.route('**/api/**', (route) => {
    const path = new URL(route.request().url()).pathname;
    if (/\/api\/(weekly|daily|board)/.test(path)) {
      return route.fulfill({ status: 200, contentType: 'application/json', body: JSON.stringify({ week: path.split('/').pop(), total: 0, entries: [], me: null, signedIn: false }) });
    }
    return route.fulfill({ status: 404, contentType: 'application/json', body: '{}' });
  });
  return context;
}

/** Collect console errors and page errors of a page into `errors`. */
export function watch(page, errors, tag) {
  page.on('console', (m) => {
    if (m.type() === 'error' && !/\/api\//.test(m.location()?.url ?? '') && !/Failed to load resource/.test(m.text())) errors.push(`${tag}: ${m.text()}`);
  });
  page.on('pageerror', (e) => errors.push(`${tag}: ${e.message}`));
}

/** Open the app with this saved run and resume it. */
export async function resume(page, base, state, { fixTime = true } = {}) {
  // The fixed clock fakes Date and performance.now; the video leaves it off.
  if (fixTime) await page.clock.setFixedTime(FIXED_TIME);
  await page.goto(base);
  await page.waitForSelector('[data-action="new-run"], [data-action="resume"]');
  await page.evaluate((s) => localStorage.setItem('stammtisch.run', s), state);
  await page.reload();
  await page.locator('[data-action="resume"]').click({ timeout: 15000 });
}

/** Wait until the table is idle (no animation of the run busy). */
export async function idle(page, timeout = 30000) {
  await page.waitForFunction(() => !document.querySelector('.run.is-busy'), null, { timeout });
}

/** Move the pointer out of the way (no hover, no info card). */
export async function park(page) {
  const v = page.viewportSize();
  await page.mouse.move(v.width - 2, 2);
}

/** Declare a Hand game with the game tile `index` (0 Eichel … 4 Grand) and the Hand level, if offered. */
export async function declareHand(page, index) {
  await page.waitForSelector('.declare');
  await page.click('[data-action="play-hand"]');
  await page.waitForSelector('.declare-hand');
  await page.click(`[data-action="pick-game"][data-index="${index}"]`);
  if (await page.locator('[data-action="pick-risk"][data-risk="Hand"]').count()) await page.click('[data-action="pick-risk"][data-risk="Hand"]');
  await page.click('[data-action="declare"]');
  await page.waitForSelector('.declare', { state: 'detached' });
}

// Smoke test of the built web app (web/dist) with Playwright and Chromium.
//
//   npm run build && node e2e/smoke.mjs
//
// Starts `vite preview`, then on a phone viewport (390 × 844):
// - starts a run with a fixed seed and passes once,
// - reloads the page and checks that the run is resumed,
// - plays a whole game (Ansage, Drücken, 10 Stiche, Abrechnung) and checks
//   that the series account changed,
// - visits the Tresen, buys something if possible and starts the next series.
// Screenshots of table, score and Tresen at 390 × 844 and 1280 × 800 go to
// $SMOKE_SCREENS (default: web/e2e/screens). Any console error fails the test.
// Runs with reduced motion for speed; SMOKE_FULL_MOTION=1 plays every
// animation at normal speed.
//
// The seed is chosen so that this simple click strategy (Grand, discard the
// first two cards, always play the first legal card) wins the first series.

import { spawn } from 'node:child_process';
import { mkdirSync } from 'node:fs';
import { createRequire } from 'node:module';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const require = createRequire(import.meta.url);
const { chromium } = require('playwright');

const here = dirname(fileURLToPath(import.meta.url));
const webDir = resolve(here, '..');
const screens = resolve(process.env.SMOKE_SCREENS ?? join(here, 'screens'));
const port = Number(process.env.SMOKE_PORT ?? 4178);
const base = `http://localhost:${port}/`;
const SEED = '64';
const PHONE = { width: 390, height: 844 };
const DESKTOP = { width: 1280, height: 800 };

mkdirSync(screens, { recursive: true });

function log(message) {
  console.log(`[smoke] ${message}`);
}

function fail(message) {
  throw new Error(message);
}

async function startPreview() {
  const vite = join(webDir, 'node_modules', '.bin', 'vite');
  const child = spawn(vite, ['preview', '--port', String(port), '--strictPort'], {
    cwd: webDir,
    stdio: ['ignore', 'pipe', 'pipe'],
  });
  let output = '';
  child.stdout.on('data', (d) => (output += d));
  child.stderr.on('data', (d) => (output += d));
  for (let i = 0; i < 100; i++) {
    try {
      const res = await fetch(base);
      if (res.ok) return child;
    } catch {
      // not up yet
    }
    if (child.exitCode !== null) fail(`vite preview exited:\n${output}`);
    await new Promise((r) => setTimeout(r, 100));
  }
  child.kill();
  fail(`vite preview did not start:\n${output}`);
}

async function shoot(page, name) {
  await page.setViewportSize(PHONE);
  await page.waitForTimeout(150);
  await page.screenshot({ path: join(screens, `${name}-390x844.png`) });
  await page.setViewportSize(DESKTOP);
  await page.waitForTimeout(150);
  await page.screenshot({ path: join(screens, `${name}-1280x800.png`) });
  await page.setViewportSize(PHONE);
  await page.waitForTimeout(100);
  log(`screenshot ${name}`);
}

async function text(page, testId) {
  return (await page.locator(`[data-testid="${testId}"]`).first().textContent())?.trim() ?? '';
}

async function run() {
  const preview = await startPreview();
  const browser = await chromium.launch();
  const errors = [];
  try {
    const reducedMotion = process.env.SMOKE_FULL_MOTION ? 'no-preference' : 'reduce';
    const context = await browser.newContext({ viewport: PHONE, reducedMotion });
    const page = await context.newPage();
    page.on('console', (msg) => {
      if (msg.type() === 'error') errors.push(`console: ${msg.text()}`);
    });
    page.on('pageerror', (err) => errors.push(`page: ${err.message}`));
    page.on('requestfailed', (req) => errors.push(`request failed: ${req.url()}`));

    // Start screen and a new run with a fixed seed.
    await page.goto(base);
    await page.waitForSelector('[data-action="new-run"]');
    const build = await text(page, 'build');
    if (!build) fail('build info missing on the start screen');
    await page.fill('#seed', SEED);
    await page.click('[data-action="new-run"]');
    await page.waitForSelector('.declare');
    log(`run started with seed ${SEED} (${build})`);
    await shoot(page, 'table-declare');

    // Pass once.
    const passes = Number(await text(page, 'passes-left'));
    await page.click('[data-action="pass"]');
    await page.waitForFunction(
      (before) => document.querySelector('[data-testid="passes-left"]')?.textContent?.trim() === String(before - 1),
      passes,
    );
    await page.waitForSelector('.declare');
    log(`passed: ${passes} -> ${passes - 1} passes`);

    // Reload: the run must be offered for resuming and continue unchanged.
    await page.reload();
    await page.waitForSelector('[data-action="resume"]');
    await page.click('[data-action="resume"]');
    await page.waitForSelector('.declare');
    const resumedPasses = Number(await text(page, 'passes-left'));
    if (resumedPasses !== passes - 1) fail(`resumed run has ${resumedPasses} passes, expected ${passes - 1}`);
    log('run resumed after reload');

    // Ansage: Grand, normal risk.
    const accountBefore = await text(page, 'account');
    await page.click('[data-action="pick-game"][data-index="4"]');
    await page.click('[data-action="declare"]');

    // Skat aufnehmen und drücken.
    await page.waitForSelector('[data-action="select"]');
    const handSize = await page.locator('.hand-card').count();
    if (handSize !== 12) fail(`expected 12 cards while discarding, saw ${handSize}`);
    await page.locator('[data-action="select"]').nth(0).click();
    await page.locator('[data-action="select"]').nth(1).click();
    await shoot(page, 'table-discard');
    await page.click('[data-action="discard"]');

    // Ten tricks: always the first legal card.
    let played = 0;
    for (;;) {
      await page.waitForSelector('[data-action="play"], .score-sheet', { timeout: 30000 });
      if (await page.locator('.score-sheet').count()) break;
      const dimmed = await page.locator('.hand-card.is-dim').count();
      if (played === 0) await shoot(page, 'table-play');
      if (played === 3 && dimmed > 0) await shoot(page, 'table-follow');
      await page.locator('[data-action="play"]').first().click();
      played += 1;
      if (played > 10) fail('played more than ten cards');
    }
    if (played !== 10) fail(`played ${played} cards, expected 10`);
    log('ten tricks played');

    // Abrechnung.
    if (await page.locator('[data-action="score-skip"].btn').count()) {
      await page.locator('[data-action="score-skip"].btn').click();
    }
    await page.waitForSelector('[data-action="score-continue"]');
    const result = await text(page, 'score-result');
    log(`score: ${result}`);
    await shoot(page, 'score');
    await page.click('[data-action="score-continue"]');
    await page.waitForSelector('.score-sheet', { state: 'detached' });
    const accountAfter = await text(page, 'account');
    if (accountAfter === accountBefore) fail(`account unchanged: ${accountAfter}`);
    log(`account ${accountBefore} -> ${accountAfter}`);

    // Tresen.
    await page.waitForSelector('[data-testid="shop"]');
    const coinsBefore = Number(await text(page, 'coins'));
    await shoot(page, 'shop');
    const buy = page.locator('[data-action="buy-relic"]:not([disabled]), [data-action="buy-item"]:not([disabled])');
    if (await buy.count()) {
      await buy.first().click();
      await page.waitForFunction(
        (before) => Number(document.querySelector('[data-testid="coins"]')?.textContent) < before,
        coinsBefore,
      );
      log(`bought at the Tresen: ${coinsBefore} -> ${await text(page, 'coins')} Groschen`);
      await shoot(page, 'shop-bought');
    }
    await page.click('[data-action="leave-shop"]');
    await page.waitForSelector('.declare, [data-action="boss-go"]');
    log('next series started');

    // Reload once more: the run is still there.
    await page.reload();
    await page.waitForSelector('[data-action="resume"]');
    log('saved after the Tresen');
    await context.close();
  } finally {
    await browser.close();
    preview.kill();
  }
  if (errors.length) fail(`console errors:\n${errors.join('\n')}`);
  log(`OK. Screenshots in ${screens}`);
}

run().catch((err) => {
  console.error(`[smoke] FAILED: ${err.message}`);
  process.exit(1);
});

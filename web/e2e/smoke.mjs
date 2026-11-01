// Smoke test of the built web app (web/dist) with Playwright and Chromium.
//
//   npm run build && node e2e/smoke.mjs
//
// Starts `vite preview`, then on a phone viewport (390 × 844):
// - checks the start screen of a new player: a fresh meta state is stored,
//   the Alleinspieler is picked, the Nullspieler is locked with his
//   condition, and the Vereinsbuch lists every unlock as open,
// - starts a run with a fixed seed and passes once,
// - reloads the page and checks that the run is resumed,
// - plays a whole game in the order of the Skatordnung (Skat oder Hand,
//   Skat aufnehmen, Drücken, Ansage, 10 Stiche, Abrechnung) and checks that
//   the series account changed; the hand Ansage offers only the hand levels,
//   the Ansage after the skat only the levels without Hand,
// - sees the unlock earned by winning the series with games left
//   ("Neu am Stammtisch") and the meta state counting the game,
// - visits the Tresen, buys something if possible, opens the Kreidetafel and
//   the menu (tempo setting is stored), and starts the next series.
// The card sprite (German deck) and the shipped pixel fonts (Kenney, CC0)
// must be present, and a face for every Stammgast of core/content/guests.ron.
// The opponents are Stammgäste with names of their own (two different
// ones, new ones in the next series, the boss on the left in a boss
// series); the first game of a series starts with a quip that stays in its
// opponent's box, and the table still fits with it at 390 × 844 and
// 375 × 667. The table must fit the visible height without scrolling
// (DESIGN 16): while choosing Skat oder Hand, in the hand Ansage, while
// discarding with twelve cards, in the Ansage after the skat and while
// playing, at 390 × 844, 375 × 667, 360 × 740, 412 × 915, 1280 × 800 and
// as an installed app (390 × 844 and 375 × 812 with safe areas) the page
// does not scroll, the Ansage is not cut off and the last hand card ends
// inside the viewport.
// Screenshots of table, score and Tresen at 390 × 844 and 1280 × 800 go to
// $SMOKE_SCREENS (default: web/e2e/screens). Any console error fails the test.
// Runs with reduced motion for speed; SMOKE_FULL_MOTION=1 plays every
// animation at normal speed.
//
// The seed is chosen so that this simple click strategy (pick up the skat,
// discard the first two cards, Grand, always play the first legal card) wins
// the first series. The Daus carries an "A" in its corner.
//
// A second pass starts from saved runs in e2e/fixtures/ (the serialized
// core state, as the app keeps it in localStorage), because the click
// strategy does not get far enough on its own:
// - boss-tresen.json: seed 13 played by the core's RunBot up to the Tresen
//   before the boss series of Der Kontra-Hans. Leaving the Tresen must show
//   the boss dialog, then the boss sits at the table.
// - kontra.json: seed 29 played by the RunBot into that boss series, the
//   Kontra-Hans has just said Kontra. The Kontra dialog must show, and
//   "Kein Re" must lead back into the game. The save is from before the
//   Skatordnung order (declared, then the skat picked up): after the discard
//   the game is played without a second Ansage.
// - encounter.json: seed 14, the RunBot's run up to "Der Kartengeber"
//   with the paid option affordable. Choosing a card of one's own must
//   open the card picker and start the next series.
// - encounter-dispute.json: seed 12 up to "Stammtisch-Streit". Taking a
//   house rule must show it in the head as a rule of the evening.
// - ramsch.json: seed 11 up to the Ramsch of "Der Ramscher". The table
//   must say so; playing it out must end in a Ramsch score.
// - old-man.json: seed 98 (content of commit 2259e7a; with the M4 content
//   no seed below 4000 gets there) up to the second game against the final boss
//   "Der Alte"; the table must name the rule he lays on this game.
// - oma.json: seed 1720 up to a hand game in which "Die Oma" can take a
//   Unter from the skat; giving a card for it must use her up.
// - meta-null.json: a meta state (not a run) with four unlocks, among them
//   the Nullspieler. With it the hand Ansage offers Null Hand and Null Ouvert
//   Hand; after picking up the skat Null and Null Ouvert; a Null is played.
//
// M5 (fixtures made with the M5 core, same way):
// - seed links: `#seed-4242-alleinspieler` opened fresh offers that seed and
//   starts it; the menu shows the seed and copies the link; a second link
//   (a locked character) while a run is saved must not overwrite it unasked
//   (confirm dialog, cancel keeps the run). The daily seed is JJJJMMTT.
// - the thought bubble appears while the core computes an opponent's card,
//   one at a time at the opponent who is to move; the setting is stored.
// - run-end.json: seed 13, the RunBot's run up to the last game of a series
//   far below its target; playing it out ends the run. The end screen shares
//   the seed, the Vereinsbuch's statistics page lists the run.
// - coins-curse.json: seed 14 up to "Der Kartengeber" offering the curse
//   "Ramponiert" on a random card: choosing it pays 3 Groschen (CoinsGained)
//   with a coin animation naming the source.
// - discard-battered.json: seed 14, that card in the hand at the Ansage;
//   the core marks it not discardable, so it cannot be picked for the skat.
// - honorary-jack.json: seed 116, a "Unter ehrenhalber" in the hand: plaque.
// - the Ramsch (ramsch.json) score shows Augen and tricks per seat and
//   marks the most.
// Save code (menu "Spielstand sichern" / "Spielstand laden"), at 390 × 844
// and at 1280 × 800: a run with seed 64 is played into its first game, the
// code is exported from the menu (and copied), localStorage is cleared, a
// damaged code and a foreign text are rejected with a message, the real
// code is imported after the confirmation, and the resumed run has the
// same seed, account, passes and hand; every stored key is back as it was.
// Persistent storage is asked for once, after the run started, not on load.
// All passes above run with the Wirt's hints switched off (quietContext).
//
// M6 "Musik" (musicPass and checks in the passes above): the music loops
// are mirrored in <html data-music>; an init script counts AudioContexts and
// oscillators. No AudioContext and no music before the first interaction;
// after starting a run the table loop plays (notes are made), the Tresen
// and an encounter switch to the Tresen loop, a boss series to the boss
// loop. In the menu "Klänge" and "Musik" are separate toggles: Klänge off
// keeps the music, Musik off stops it (no new notes) and stays off after a
// reload, Klänge on does not bring it back; the menu still fits.
//
// M6 "Geführter erster Abend" (hintsAndRules), with a fresh profile: the
// Regelkarte opens from the menu and closes; the Wirt's hints come one at a
// time (welcome, Skat oder Hand, pass, Ansage after the skat, value line,
// score, Tresen, boss, encounter, lost run), a dismissed hint stays away
// after a reload, the menu switch "Hinweise des Wirts" hides them and the
// reset brings them back. With a hint on screen the table still fits at
// 390 × 844 and 375 × 667, and the bubble covers neither the hand nor the
// buttons (score, boss, Tresen, encounter, end).
//
// The runs were made with skat_wasm::Session and RunBot (create a session
// with Session::new(seed), apply RunBot::next_action until the view
// matches, save; search the seeds from 0 upwards). When the save format
// changes (SAVE_VERSION, META_VERSION), make them again the same way. The
// run fixtures are kept from before the Skatordnung order on purpose: they
// show that old saves still load and play on.

import { spawn } from 'node:child_process';
import { mkdirSync, readFileSync } from 'node:fs';
import { createRequire } from 'node:module';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { crc32 } from 'node:zlib';

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

/**
 * Viewports the table must fit without scrolling (DESIGN 16). The last one
 * is an installed app (PWA) on a phone without browser bars: the full
 * 390 × 844 with the notch and home bar as safe areas, simulated through
 * the CSS variables that carry env(safe-area-inset-*) (style.css).
 */
const FIT_SIZES = [
  PHONE,
  { width: 375, height: 667 },
  { width: 360, height: 740 },
  { width: 412, height: 915 },
  DESKTOP,
  { ...PHONE, insets: { top: 59, bottom: 34 } },
  { width: 375, height: 812, insets: { top: 50, bottom: 34 } },
];

/**
 * The table fits the visible height: nothing scrolls (neither the page nor
 * the Ansage panel), and the bottom of the last hand card is inside the
 * viewport (above a bottom safe area), at every size in FIT_SIZES.
 */
async function assertFits(page, label, sizes = FIT_SIZES) {
  for (const size of sizes) {
    await page.setViewportSize({ width: size.width, height: size.height });
    const insets = size.insets ?? { top: 0, bottom: 0 };
    await page.evaluate((i) => {
      const root = document.documentElement.style;
      if (i.top) root.setProperty('--safe-top', `${i.top}px`);
      else root.removeProperty('--safe-top');
      if (i.bottom) root.setProperty('--safe-bottom', `${i.bottom}px`);
      else root.removeProperty('--safe-bottom');
    }, insets);
    await page.waitForTimeout(120);
    const r = await page.evaluate(() => {
      const cards = [...document.querySelectorAll('.hand-card')];
      const last = cards[cards.length - 1];
      const screen = document.querySelector('.screen');
      const declare = document.querySelector('.declare');
      return {
        scroll: document.scrollingElement.scrollHeight,
        inner: window.innerHeight,
        screen: screen ? screen.scrollHeight - screen.clientHeight : 0,
        declare: declare ? declare.scrollHeight - declare.clientHeight : 0,
        cards: cards.length,
        bottom: last ? last.getBoundingClientRect().bottom : null,
      };
    });
    await page.evaluate(() => {
      document.documentElement.style.removeProperty('--safe-top');
      document.documentElement.style.removeProperty('--safe-bottom');
    });
    const where = `${label} at ${size.width}x${size.height}${size.insets ? ' (PWA, safe areas)' : ''}`;
    if (r.scroll > r.inner) fail(`${where}: the page scrolls (${r.scroll} > ${r.inner})`);
    if (r.screen > 0) fail(`${where}: the table scrolls by ${r.screen}px`);
    if (r.declare > 1) fail(`${where}: the Ansage is cut off by ${r.declare}px`);
    if (!r.cards || r.bottom === null) fail(`${where}: no hand cards`);
    if (r.bottom > r.inner - insets.bottom) fail(`${where}: the last hand card ends at ${Math.round(r.bottom)}, below ${r.inner - insets.bottom}`);
  }
  await page.setViewportSize(PHONE);
  await page.waitForTimeout(80);
  log(`${label}: table fits ${sizes.map((s) => `${s.width}x${s.height}${s.insets ? ' PWA' : ''}`).join(', ')}`);
}

/** Look ids of the Stammgäste in the core content (core/content/guests.ron). */
function guestLooks() {
  const ron = readFileSync(resolve(webDir, '..', 'core', 'content', 'guests.ron'), 'utf8');
  return [...ron.matchAll(/\blook:\s*"([^"]+)"/g)].map((m) => m[1]);
}

/**
 * The two opponents are Stammgäste with a name and a face of their own
 * (DESIGN 16, M6), the names differ; in a boss series the boss sits on
 * the left and a Stammgast on the right. A quip, if one shows, stays in
 * its opponent's box and covers neither the hand nor a button.
 */
async function assertGuests(page, label, { boss = false } = {}) {
  const r = await page.evaluate(() => {
    const box = (el) => el.getBoundingClientRect();
    const hit = (a, b) => a.left < b.right && b.left < a.right && a.top < b.bottom && b.top < a.bottom;
    const quip = document.querySelector('[data-testid="quip"]');
    const q = quip ? box(quip) : null;
    const owner = quip?.closest('.opp');
    const covered = q
      ? [...document.querySelectorAll('.hand-card, button, .trick')].filter((el) => !quip.contains(el) && hit(q, box(el))).map((el) => el.className || el.tagName)
      : [];
    return {
      opps: [...document.querySelectorAll('.opp')].map((o) => ({
        seat: o.dataset.seat,
        guest: o.dataset.guest ?? null,
        face: o.dataset.face ?? '',
        boss: o.classList.contains('is-boss'),
        name: o.querySelector('[data-testid="opp-name"]')?.textContent?.trim() ?? '',
      })),
      quip: quip ? { text: quip.textContent.trim(), inside: owner ? (() => { const o = box(owner); return q.left >= o.left - 1 && q.right <= o.right + 1 && q.top >= o.top - 1 && q.bottom <= o.bottom + 1; })() : false, covered } : null,
    };
  });
  const where = `${label}: opponents`;
  if (r.opps.length !== 2) fail(`${where}: ${r.opps.length} opponent boxes`);
  const [left, right] = r.opps;
  for (const o of r.opps) {
    if (!o.name || /^Gegner (links|rechts)$/.test(o.name)) fail(`${where}: seat ${o.seat} has no name of its own (${o.name || 'empty'})`);
    if (!o.boss && (!o.guest || !o.face.startsWith('p-guest-'))) fail(`${where}: seat ${o.seat} is no Stammgast with a face (${JSON.stringify(o)})`);
  }
  if (left.name === right.name) fail(`${where}: both are called ${left.name}`);
  if (boss && !(left.boss && !right.boss)) fail(`${where}: the boss should sit left, a Stammgast right (${JSON.stringify(r.opps)})`);
  if (!boss && (left.boss || right.boss)) fail(`${where}: a boss in a normal series`);
  if (r.quip && !r.quip.inside) fail(`${where}: the quip "${r.quip.text}" leaves its opponent's box`);
  if (r.quip?.covered.length) fail(`${where}: the quip covers ${r.quip.covered.join(', ')}`);
  log(`${where}: ${left.name} and ${right.name}${r.quip ? `, quip "${r.quip.text}"` : ''}`);
  return r;
}

/**
 * A browser context in which the Wirt keeps quiet: before the app starts,
 * the settings get `hints: false` unless the page already stored settings.
 * The passes above test other things and click where a bubble might hang.
 */
async function quietContext(browser, options) {
  const context = await browser.newContext(options);
  await context.addInitScript(() => {
    try {
      if (!localStorage.getItem('stammtisch.settings')) {
        localStorage.setItem('stammtisch.settings', JSON.stringify({ tempo: 'normal', sound: true, thinking: true, hints: false }));
      }
    } catch {
      // Storage blocked: nothing to set.
    }
  });
  return context;
}

function fixture(name) {
  return readFileSync(join(here, 'fixtures', name), 'utf8');
}

/** The stored meta state, parsed. */
async function storedMeta(page) {
  const raw = await page.evaluate(() => localStorage.getItem('stammtisch.meta'));
  if (!raw) fail('no meta state in localStorage');
  return JSON.parse(raw);
}

/** Collect console errors of a page. */
function watch(page, errors) {
  page.on('console', (msg) => {
    if (msg.type() === 'error') errors.push(`console: ${msg.text()}`);
  });
  page.on('pageerror', (err) => errors.push(`page: ${err.message}`));
  page.on('requestfailed', (req) => errors.push(`request failed: ${req.url()}`));
}

/** Count AudioContexts and oscillators (every music note makes one or two). */
async function spyAudio(context) {
  await context.addInitScript(() => {
    window.__audio = { contexts: 0, oscillators: 0 };
    const Orig = window.AudioContext;
    if (!Orig) return;
    window.AudioContext = class extends Orig {
      constructor(...args) {
        super(...args);
        window.__audio.contexts += 1;
      }
    };
    const create = Orig.prototype.createOscillator;
    Orig.prototype.createOscillator = function (...args) {
      window.__audio.oscillators += 1;
      return create.apply(this, args);
    };
  });
}

/** Wait until <html data-music> names the expected loop ("off" for none). */
async function expectMusic(page, loop, label) {
  try {
    await page.waitForFunction((want) => document.documentElement.dataset.music === want, loop, { timeout: 5000 });
  } catch {
    const now = await page.evaluate(() => document.documentElement.dataset.music);
    fail(`${label}: music should be "${loop}", is "${now}"`);
  }
  log(`${label}: music ${loop}`);
}

/** Play the first legal card until the score board shows (answering Kontra without Re), then close it. */
async function playOut(page, label, shotAt = null) {
  let played = 0;
  for (;;) {
    await page.waitForSelector('[data-action="play"], [data-action="select"], [data-action="re"][data-re="false"], .score-sheet', { timeout: 30000 });
    if (await page.locator('.score-sheet').count()) break;
    if (await page.locator('[data-action="re"][data-re="false"]').count()) {
      await page.click('[data-action="re"][data-re="false"]');
      await page.waitForSelector('[data-action="re"]', { state: 'detached' });
      continue;
    }
    if (await page.locator('[data-action="select"]').count()) {
      await page.locator('[data-action="select"]').nth(0).click();
      await page.locator('[data-action="select"]').nth(1).click();
      await page.click('[data-action="discard"]');
      continue;
    }
    if (shotAt !== null && played === shotAt.at) await shoot(page, shotAt.name);
    await page.locator('[data-action="play"]').first().click();
    played += 1;
    if (played > 12) fail(`${label}: played more than ten cards`);
  }
  if (await page.locator('[data-action="score-skip"].btn').count()) {
    await page.locator('[data-action="score-skip"].btn').click();
  }
  await page.waitForSelector('[data-action="score-continue"]');
  log(`${label}: ${played} cards played, score ${await text(page, 'score-result')}`);
  return played;
}

async function text(page, testId) {
  return (await page.locator(`[data-testid="${testId}"]`).first().textContent())?.trim() ?? '';
}

/** Pick up the skat and press the first two cards the core lets go. */
async function pickUpAndDiscard(page, label, shot = null) {
  await page.waitForSelector('[data-testid="skat-choice"]');
  await page.click('[data-action="pick-up-skat"]');
  await page.waitForSelector('[data-action="select"]');
  const handSize = await page.locator('.hand-card').count();
  if (handSize !== 12) fail(`${label}: expected 12 cards while discarding, saw ${handSize}`);
  if (await page.locator('.declare').count()) fail(`${label}: an Ansage while discarding`);
  await page.locator('[data-action="select"]').nth(0).click();
  await page.locator('[data-action="select"]').nth(1).click();
  if (shot) {
    await shoot(page, shot);
    await assertFits(page, `${label}, discarding (12 cards)`);
  }
  await page.click('[data-action="discard"]');
  await page.waitForSelector('.declare[data-phase="DeclaringAfterSkat"]');
  const pressed = await page.locator('[data-testid="pressed"] .mini-card').count();
  if (pressed !== 2) fail(`${label}: the Ansage after the skat shows ${pressed} pressed cards`);
}

/** Skat aufnehmen, drücken, then declare a Grand without hand levels. */
async function skatThenGrand(page, label, shot = null) {
  await pickUpAndDiscard(page, label, shot);
  if (await page.locator('[data-action="pick-risk"]').count()) fail(`${label}: hand levels after the skat`);
  if (await page.locator('[data-action="hand-back"]').count()) fail(`${label}: back to the skat choice after the skat`);
  await page.click('[data-action="pick-game"][data-index="4"]');
  await page.click('[data-action="declare"]');
  await page.waitForSelector('.declare', { state: 'detached' });
}

/** Open the app with a saved run from e2e/fixtures and resume it. */
async function resumeFixture(page, name) {
  const state = fixture(name);
  await page.goto(base);
  await page.waitForSelector('[data-action="new-run"]');
  await page.evaluate((s) => localStorage.setItem('stammtisch.run', s), state);
  await page.reload();
  const resume = page.locator('[data-action="resume"]');
  try {
    await resume.waitFor({ timeout: 10000 });
  } catch {
    fail(`saved run ${name} is not offered for resuming; the core may reject it (save format changed?), see the notes at the top`);
  }
  await resume.click();
}

/** The music: silent until the first interaction, loops per screen, its own switch. */
async function musicPass(browser, errors, reducedMotion) {
  const context = await quietContext(browser, { viewport: PHONE, reducedMotion });
  await spyAudio(context);
  const page = await context.newPage();
  watch(page, errors);
  const audio = () => page.evaluate(() => ({ ...window.__audio }));
  const stored = () => page.evaluate(() => JSON.parse(localStorage.getItem('stammtisch.settings') ?? '{}'));

  // Before any interaction: no AudioContext, no music.
  await page.goto(base);
  await page.waitForSelector('[data-action="new-run"]');
  await expectMusic(page, 'off', 'before the first interaction');
  if ((await audio()).contexts !== 0) fail('an AudioContext was made before the first interaction');

  // Starting a run is the first interaction: the table loop plays notes.
  await page.fill('#seed', SEED);
  await page.click('[data-action="new-run"]');
  await page.waitForSelector('[data-testid="skat-choice"]');
  await expectMusic(page, 'table', 'run started');
  await page.waitForFunction(() => window.__audio.oscillators > 0, null, { timeout: 5000 }).catch(() => fail('the table loop makes no notes'));

  // Menu: "Klänge" and "Musik" side by side, the menu still fits.
  await page.click('[data-action="menu"]');
  await page.waitForSelector('[data-testid="toggle-music"]');
  await assertMenuNoScroll(page, 'menu with Klänge and Musik');
  for (const id of ['toggle-sound', 'toggle-music']) {
    if ((await page.getAttribute(`[data-testid="${id}"]`, 'aria-pressed')) !== 'true') fail(`${id} is not on by default`);
  }

  // Klänge off: the music plays on.
  await page.click('[data-testid="toggle-sound"]');
  await page.waitForSelector('[data-testid="toggle-sound"][aria-pressed="false"]');
  let settings = await stored();
  if (settings.sound !== false || settings.music === false) fail(`Klänge off stored as ${JSON.stringify(settings)}`);
  await expectMusic(page, 'table', 'Klänge off');
  const playing = (await audio()).oscillators;
  await page.waitForFunction((n) => window.__audio.oscillators > n, playing, { timeout: 5000 }).catch(() => fail('the music stopped with the Klänge'));

  // Musik off: no more notes once the short fade is over.
  await page.click('[data-testid="toggle-music"]');
  await page.waitForSelector('[data-testid="toggle-music"][aria-pressed="false"]');
  settings = await stored();
  if (settings.music !== false || settings.sound !== false) fail(`Musik off stored as ${JSON.stringify(settings)}`);
  await expectMusic(page, 'off', 'Musik off');
  await page.waitForTimeout(1200);
  const stopped = (await audio()).oscillators;
  await page.waitForTimeout(800);
  if ((await audio()).oscillators !== stopped) fail('notes are still made with the music off');

  // Klänge on again: the music stays off.
  await page.click('[data-testid="toggle-sound"]');
  await page.waitForSelector('[data-testid="toggle-sound"][aria-pressed="true"]');
  await expectMusic(page, 'off', 'Klänge on, Musik off');
  await assertMenuNoScroll(page, 'menu with the music off');
  await page.click('[data-testid="menu-close"]');

  // After a reload the music is still off, also after an interaction.
  await page.reload();
  await page.waitForSelector('[data-action="resume"]');
  await page.click('[data-action="resume"]');
  await page.waitForSelector('.declare');
  await expectMusic(page, 'off', 'Musik off after a reload');
  await page.click('[data-action="menu"]');
  await page.waitForSelector('[data-testid="toggle-music"][aria-pressed="false"]');
  await page.click('[data-testid="toggle-music"]');
  await expectMusic(page, 'table', 'Musik on again');
  settings = await stored();
  if (settings.music !== true || settings.sound !== true) fail(`Musik on stored as ${JSON.stringify(settings)}`);
  await context.close();
}

/** Boss dialog and Kontra dialog, from saved runs. */
async function bossAndKontra(browser, errors, reducedMotion) {
  const context = await quietContext(browser, { viewport: PHONE, reducedMotion });
  const page = await context.newPage();
  watch(page, errors);

  // Tresen before the boss series: the preview names the boss.
  await resumeFixture(page, 'boss-tresen.json');
  await page.waitForSelector('[data-testid="shop"]');
  const next = await text(page, 'next-series');
  if (!next.includes('Kontra-Hans')) fail(`the Tresen does not announce the boss: ${next}`);
  await shoot(page, 'shop-boss');
  await expectMusic(page, 'shop', 'Tresen before the boss');
  await page.click('[data-action="leave-shop"]');
  await page.waitForSelector('[data-action="boss-go"]');
  await expectMusic(page, 'boss', 'boss dialog');
  const bossText = (await page.locator('.sheet-boss').textContent()) ?? '';
  if (!bossText.includes('Kontra-Hans')) fail(`boss dialog without the boss: ${bossText}`);
  log('boss dialog shown');
  await shoot(page, 'boss');
  await page.click('[data-action="boss-go"]');
  await page.waitForSelector('.declare');
  if (!(await page.locator('.opp.is-boss').count())) fail('the boss does not sit at the table');
  await expectMusic(page, 'boss', 'boss series');
  await assertGuests(page, 'boss series', { boss: true });
  await shoot(page, 'table-boss');
  await assertFits(page, 'boss at the table');

  // The Kontra-Hans says Kontra: answer without Re, the game goes on.
  await resumeFixture(page, 'kontra.json');
  await page.waitForSelector('[data-action="re"][data-re="false"]');
  const kontraText = (await page.locator('.sheet').textContent()) ?? '';
  if (!kontraText.includes('Kontra')) fail(`Kontra dialog without Kontra: ${kontraText}`);
  log('Kontra dialog shown');
  await shoot(page, 'kontra');
  await page.click('[data-action="re"][data-re="false"]');
  await page.waitForSelector('[data-action="re"]', { state: 'detached' });
  await page.waitForSelector('[data-action="play"], [data-action="select"]', { timeout: 30000 });
  if (!(await page.locator('.badge-kontra').count())) fail('no Kontra mark at the table after the answer');
  log('Kontra answered, game goes on');
  await shoot(page, 'table-kontra');
  await assertFits(page, 'after Kontra');
  await context.close();
}

/** M4: the Nullspieler, encounters, the Ramsch and "Der Alte". */
async function m4(browser, errors, reducedMotion) {
  const context = await quietContext(browser, { viewport: PHONE, reducedMotion });
  const page = await context.newPage();
  watch(page, errors);

  // A meta state with the Nullspieler unlocked: pick him, declare Null.
  await page.goto(base);
  await page.waitForSelector('[data-action="new-run"]');
  await page.evaluate((m) => localStorage.setItem('stammtisch.meta', m), fixture('meta-null.json'));
  await page.reload();
  await page.waitForSelector('[data-action="pick-character"][data-character="NullPlayer"]');
  await page.click('[data-action="pick-character"][data-character="NullPlayer"]');
  await page.waitForSelector('[data-character="NullPlayer"][aria-pressed="true"]');
  const stored = await page.evaluate(() => localStorage.getItem('stammtisch.character'));
  if (stored !== 'NullPlayer') fail(`picked character not stored: ${stored}`);
  await shoot(page, 'start-null-unlocked');
  await page.click('[data-action="book"]');
  await page.waitForSelector('.ledger-row.is-done');
  const done = await page.locator('.ledger-row.is-done').count();
  if (done !== 4) fail(`Vereinsbuch shows ${done} done unlocks, expected 4`);
  await shoot(page, 'book');
  await page.click('.ledger [data-action="close-overlay"]');
  await page.fill('#seed', '5');
  await page.click('[data-action="new-run"]');
  await page.waitForSelector('[data-testid="skat-choice"]');
  if (!(await page.locator('.mini-card.is-kiebitz').count())) fail('the Kiebitz shows no card');
  // Hand: Null Hand and Null Ouvert Hand.
  await page.click('[data-action="play-hand"]');
  await page.waitForSelector('[data-testid="null-declare"][data-phase="Declaring"]');
  const handLevels = await page.locator('[data-action="pick-null-risk"]').evaluateAll((els) => els.map((e) => e.getAttribute('data-risk')).join(','));
  if (handLevels !== 'Hand,OuvertHand') fail(`Null hand Ansage offers ${handLevels}`);
  await shoot(page, 'null-declare-hand');
  await assertFits(page, 'Null hand Ansage');
  await page.click('[data-action="hand-back"]');
  // With the skat: Null and Null Ouvert.
  await pickUpAndDiscard(page, 'Null');
  await page.waitForSelector('[data-testid="null-declare"][data-phase="DeclaringAfterSkat"]');
  const levels = await page.locator('[data-action="pick-null-risk"]').evaluateAll((els) => els.map((e) => e.getAttribute('data-risk')).join(','));
  if (levels !== 'Normal,Ouvert') fail(`Null Ansage after the skat offers ${levels}`);
  const values = ((await page.locator('.null-risks').textContent()) ?? '').replace(/\s+/g, ' ');
  if (!/Wert \d+/.test(values)) fail(`Null levels without values: ${values}`);
  log(`Null Ansage: hand ${handLevels}; after the skat ${values.trim()}; Kiebitz card shown`);
  await shoot(page, 'null-declare');
  await assertFits(page, 'Null Ansage with the Kiebitz');
  await page.click('[data-action="pick-null-risk"][data-risk="Normal"]');
  await page.click('[data-action="declare-null"]');
  await page.waitForSelector('.badge-null');
  await playOut(page, 'Null game', { at: 1, name: 'null-play' });
  const decl = await page.locator('.score-decl').textContent();
  if (!decl?.includes('Null')) fail(`score board of a Null game says: ${decl}`);
  if (!(await page.locator('[data-testid="score-note"]').count())) fail('Null score without the note on the tricks');
  await shoot(page, 'null-score');
  await page.click('[data-action="score-continue"]');
  await page.waitForSelector('.score-sheet', { state: 'detached' });

  // Der Kartengeber: pick the card yourself.
  await resumeFixture(page, 'encounter.json');
  await page.waitForSelector('[data-testid="encounter"]');
  await expectMusic(page, 'shop', 'encounter');
  await shoot(page, 'encounter');
  await page.click('[data-action="choose-encounter"][data-target="true"]');
  await page.waitForSelector('[data-action="encounter-target"]:not([disabled])');
  await shoot(page, 'encounter-target');
  await page.locator('[data-action="encounter-target"]:not([disabled])').first().click();
  await page.waitForSelector('[data-testid="encounter"]', { state: 'detached' });
  await page.waitForSelector('.declare, [data-action="boss-go"]');
  log('encounter Der Kartengeber resolved with a chosen card');

  // Stammtisch-Streit: the house rule holds for the evening.
  await resumeFixture(page, 'encounter-dispute.json');
  await page.waitForSelector('[data-testid="encounter"][data-encounter="stammtisch_dispute"]');
  await shoot(page, 'encounter-dispute');
  await page.locator('[data-action="choose-encounter"]').first().click();
  await page.waitForSelector('.coaster-chip.is-evening');
  log('Stammtisch-Streit: rule of the evening in the head');
  await shoot(page, 'evening-rule');

  // Der Ramscher: nobody declares, the Ramsch is played out.
  await resumeFixture(page, 'ramsch.json');
  await page.waitForSelector('[data-testid="ramsch"]');
  if (await page.locator('.declare').count()) fail('Ramsch offers an Ansage');
  await shoot(page, 'ramsch');
  await assertFits(page, 'Ramsch');
  await playOut(page, 'Ramsch');
  const ramschDecl = await page.locator('.score-decl').textContent();
  if (!ramschDecl?.includes('Ramsch')) fail(`score board of the Ramsch says: ${ramschDecl}`);
  const seatRows = await page.locator('[data-testid="ramsch-seats"] .seat-row').count();
  if (seatRows !== 3) fail(`Ramsch score shows ${seatRows} seats, expected 3`);
  if (!(await page.locator('[data-testid="ramsch-seats"] .seat-row.is-most').count())) fail('Ramsch score does not mark who has the most Augen');
  const seatsText = ((await page.locator('[data-testid="ramsch-seats"]').textContent()) ?? '').replace(/\s+/g, ' ').trim();
  if (!/\d+ Augen/.test(seatsText)) fail(`Ramsch seats without Augen: ${seatsText}`);
  log(`Ramsch seats: ${seatsText}`);
  await shoot(page, 'ramsch-score');

  // Die Oma: a Unter from the skat for a card of the hand.
  await resumeFixture(page, 'oma.json');
  await page.waitForSelector('[data-action="oma"]');
  await page.click('[data-action="oma"]');
  await page.waitForSelector('.hand.is-oma [data-action="oma-give"]');
  await shoot(page, 'oma');
  await assertFits(page, 'Die Oma');
  const handBefore = await page.locator('.hand-card').count();
  await page.locator('[data-action="oma-give"]').last().click();
  await page.waitForSelector('[data-action="oma"]', { state: 'detached' });
  await page.waitForSelector('.hand.is-oma', { state: 'detached' });
  const handAfter = await page.locator('.hand-card').count();
  if (handAfter !== handBefore) fail(`hand changed size after the Oma: ${handBefore} -> ${handAfter}`);
  log('Die Oma: card taken from the skat');

  // Der Alte lays another boss's rule on every game.
  await resumeFixture(page, 'old-man.json');
  await page.waitForSelector('[data-testid="boss-rule"]');
  const rule = ((await page.locator('[data-testid="boss-rule"]').textContent()) ?? '').replace(/\s+/g, ' ').trim();
  if (!rule.includes('Der Alte')) fail(`the final boss's rule strip does not name him: ${rule}`);
  log(`Der Alte: ${rule}`);
  await shoot(page, 'old-man');
  await assertFits(page, 'Der Alte');
  await context.close();
}

/** Today's daily seed as the app computes it: JJJJMMTT, local date. */
function todaySeed() {
  const d = new Date();
  return `${d.getFullYear()}${String(d.getMonth() + 1).padStart(2, '0')}${String(d.getDate()).padStart(2, '0')}`;
}

/** The seed of the running run, read from the menu. */
async function menuSeed(page) {
  await page.click('.hud [data-action="menu"]');
  await page.waitForSelector('.sheet [data-testid="run-seed"]');
  const seed = await text(page, 'run-seed');
  return seed;
}

async function closeSheet(page) {
  await page.click('.sheet .menu-list [data-action="close-overlay"], .sheet > .btn[data-action="close-overlay"]');
  await page.waitForSelector('.layer .sheet', { state: 'detached' });
}

/** M5: seed links, daily seed, thought bubble, statistics, Groschen from a curse, plaques. */
async function m5(browser, errors, reducedMotion) {
  const context = await quietContext(browser, { viewport: PHONE, reducedMotion });
  await context.grantPermissions(['clipboard-read', 'clipboard-write'], { origin: base.replace(/\/$/, '') });
  const page = await context.newPage();
  watch(page, errors);

  // A seed link, opened in a fresh browser: the start screen offers it.
  await page.goto(`${base}#seed-4242-alleinspieler`);
  await page.waitForSelector('[data-testid="shared-seed"]');
  const offered = await page.locator('[data-testid="shared-seed"]').getAttribute('data-seed');
  if (offered !== '4242') fail(`seed link offers seed ${offered}, expected 4242`);
  await shoot(page, 'shared-seed');
  await page.click('[data-action="play-shared"]');
  await page.waitForSelector('.declare');
  if (await page.evaluate(() => window.location.hash)) fail('the seed link stays in the address after starting it');
  const linkSeed = await menuSeed(page);
  if (linkSeed !== '4242') fail(`run from the link has seed ${linkSeed}`);
  log('seed link opened and played: seed 4242');

  // Copy the link from the menu (or show it to copy by hand).
  await page.click('[data-action="copy-link"]');
  await page.waitForSelector('[data-testid="share-copied"], [data-testid="share-fallback"]');
  if (await page.locator('[data-testid="share-copied"]').count()) {
    const copied = await page.evaluate(() => navigator.clipboard.readText());
    if (!copied.endsWith('#seed-4242-alleinspieler')) fail(`copied link is ${copied}`);
    log(`link copied: ${copied}`);
  } else {
    const shown = await page.locator('[data-testid="share-fallback"]').inputValue();
    if (!shown.endsWith('#seed-4242-alleinspieler')) fail(`link shown for copying is ${shown}`);
    log(`clipboard refused, link shown: ${shown}`);
  }
  // Settings: the opponents' thinking time.
  await page.click('[data-action="thinking"][data-on="false"]');
  let thinking = await page.evaluate(() => JSON.parse(localStorage.getItem('stammtisch.settings') ?? '{}').thinking);
  if (thinking !== false) fail(`thinking setting not stored: ${thinking}`);
  await page.click('[data-action="thinking"][data-on="true"]');
  thinking = await page.evaluate(() => JSON.parse(localStorage.getItem('stammtisch.settings') ?? '{}').thinking);
  if (thinking !== true) fail(`thinking setting not stored: ${thinking}`);
  await shoot(page, 'menu-share');
  await closeSheet(page);

  // The thought bubble shows while the core computes an opponent's card:
  // one bubble at a time, at the opponent who is to move.
  await page.evaluate(() => {
    window.__thinkSeen = 0;
    window.__thinkWrong = [];
    new MutationObserver((records) => {
      for (const r of records) {
        for (const n of r.addedNodes) {
          if (n instanceof Element && (n.matches('.think-bubble') || n.querySelector('.think-bubble'))) {
            window.__thinkSeen += 1;
            const bubbles = Array.from(document.querySelectorAll('.think-bubble'));
            const seats = bubbles.map((b) => b.closest('.opp')?.getAttribute('data-seat'));
            const atTurn = bubbles.every((b) => b.closest('.opp')?.classList.contains('is-turn'));
            if (bubbles.length !== 1 || !atTurn) window.__thinkWrong.push(`${bubbles.length} bubble(s) at ${seats.join(',')}, at turn: ${atTurn}`);
          }
        }
      }
    }).observe(document.body, { childList: true, subtree: true });
  });
  await skatThenGrand(page, 'seed link');
  await page.waitForSelector('[data-action="play"]');
  await page.locator('[data-action="play"]').first().click();
  await page.waitForSelector('[data-action="play"], .score-sheet', { timeout: 30000 });
  const seen = await page.evaluate(() => window.__thinkSeen);
  if (!seen) fail('no thought bubble while the opponents played');
  const wrong = await page.evaluate(() => window.__thinkWrong);
  if (wrong.length) fail(`thought bubble not at the opponent to move: ${wrong.join('; ')}`);
  log(`thought bubble shown ${seen} times`);

  // A second link while this run is saved: offered, but nothing is overwritten unasked.
  const runBefore = await page.evaluate(() => localStorage.getItem('stammtisch.run'));
  await page.click('.hud [data-action="menu"]');
  await page.click('.sheet [data-action="to-start"]');
  await page.waitForSelector('[data-action="resume"]');
  await page.goto(`${base}#seed-777-nullspieler`);
  await page.waitForSelector('[data-testid="shared-seed"][data-seed="777"]');
  const lockedNote = (await page.locator('[data-testid="shared-seed"]').textContent()) ?? '';
  if (!lockedNote.includes('gesperrt')) fail(`a link with a locked character does not say so: ${lockedNote}`);
  if (!(await page.locator('[data-action="resume"]').count())) fail('the running run is no longer offered after opening a link');
  await page.click('[data-action="play-shared"]');
  await page.waitForSelector('.sheet-confirm');
  await shoot(page, 'confirm-new');
  await page.click('.sheet-confirm [data-action="close-overlay"]');
  await page.waitForSelector('.sheet-confirm', { state: 'detached' });
  if ((await page.evaluate(() => localStorage.getItem('stammtisch.run'))) !== runBefore) fail('cancelling the new run changed the saved run');
  await page.click('[data-action="play-shared"]');
  await page.click('.sheet-confirm [data-action="confirm-new"]');
  await page.waitForSelector('.declare');
  const lockedSeed = await menuSeed(page);
  if (lockedSeed !== '777') fail(`run from the second link has seed ${lockedSeed}`);
  log('second link: confirm asked, cancel kept the run, confirm started seed 777');

  // The daily seed: JJJJMMTT of today.
  await page.click('.sheet [data-action="to-start"]');
  await page.waitForSelector('[data-testid="daily"]');
  const daily = await page.locator('[data-testid="daily"]').getAttribute('data-seed');
  if (daily !== todaySeed()) fail(`daily seed is ${daily}, expected ${todaySeed()}`);
  await shoot(page, 'start-daily');
  await page.click('[data-action="play-daily"]');
  await page.click('.sheet-confirm [data-action="confirm-new"]');
  await page.waitForSelector('.declare');
  const dailyRun = await menuSeed(page);
  if (dailyRun !== daily) fail(`daily run has seed ${dailyRun}, expected ${daily}`);
  await closeSheet(page);
  log(`daily seed ${daily} played`);

  // A run that ends: the end screen shares the seed, the statistics page lists the run.
  await resumeFixture(page, 'run-end.json');
  await page.waitForSelector('.declare');
  await skatThenGrand(page, 'last game');
  await playOut(page, 'last game');
  await page.click('[data-action="score-continue"]');
  await page.waitForSelector('[data-testid="end"]');
  if (await page.locator('.sheet-unlock').count()) await page.click('.sheet-unlock [data-action="close-overlay"]');
  const endSeed = await text(page, 'run-seed');
  if (endSeed !== '13') fail(`end screen shows seed ${endSeed}, expected 13`);
  if (await page.evaluate(() => localStorage.getItem('stammtisch.run'))) fail('the finished run is still saved');
  await shoot(page, 'end');
  await page.click('[data-action="to-start"]');
  await page.click('[data-action="book"]');
  await page.click('[data-action="book-tab"][data-tab="stats"]');
  await page.waitForSelector('[data-testid="recent-runs"]');
  const recent = await page.locator('[data-testid="recent-runs"] .recent-row[data-seed="13"]').count();
  if (recent !== 1) fail(`statistics page lists the finished run ${recent} times`);
  const statsText = ((await page.locator('[data-testid="meta-stats"]').textContent()) ?? '').replace(/\s+/g, ' ');
  if (!/Runs gespielt\s*1/.test(statsText)) fail(`meta statistics do not count the run: ${statsText}`);
  log(`statistics page: ${statsText.trim()}`);
  await shoot(page, 'book-stats');
  await page.click('.ledger > .btn[data-action="close-overlay"]');

  // Der Kartengeber with the curse "Ramponiert": 3 Groschen with their source.
  await resumeFixture(page, 'coins-curse.json');
  await page.waitForSelector('[data-testid="encounter"]');
  const coinsBefore = Number(await text(page, 'coins'));
  const curse = page.locator('[data-action="choose-encounter"][data-target="false"]:has([data-card-mod="battered"])');
  if (!(await curse.count())) fail('the encounter does not name the curse it gives');
  await shoot(page, 'encounter-curse');
  await curse.click();
  await page.waitForSelector('[data-testid="coin-pop"]');
  const pop = ((await page.locator('[data-testid="coin-pop"]').textContent()) ?? '').trim();
  if (!pop.includes('+3') || !pop.includes('Ramponiert')) fail(`Groschen animation says: ${pop}`);
  await shoot(page, 'coins');
  await page.waitForSelector('.declare, [data-action="boss-go"]');
  const coinsAfter = Number(await text(page, 'coins'));
  if (coinsAfter !== coinsBefore + 3) fail(`Groschen ${coinsBefore} -> ${coinsAfter}, expected +3`);
  log(`curse paid: ${pop.replace(/\s+/g, ' ')}; ${coinsBefore} -> ${coinsAfter} Groschen`);

  // A "Ramponiert" card in the hand: the core marks it not discardable, so
  // it is locked and cannot be picked; two other cards go into the skat.
  await resumeFixture(page, 'discard-battered.json');
  await page.waitForSelector('.declare');
  if (!(await page.locator('.hand-card .plaque-lock').count())) fail('the Ramponiert card has no lock plaque');
  await page.click('[data-action="pick-up-skat"]');
  await page.waitForSelector('[data-action="select"]');
  const lockedCards = page.locator('.hand-card.is-locked');
  const lockedCount = await lockedCards.count();
  if (!lockedCount) fail('the Ramponiert card is not locked while discarding');
  if (await page.locator('[data-action="select"][data-no-discard="true"]').count()) fail('the Ramponiert card can be picked for the skat');
  await lockedCards.first().click();
  if (await page.locator('[data-action="select"][aria-pressed="true"]').count()) fail('tapping the Ramponiert card picked something');
  await shoot(page, 'discard-locked');
  await assertFits(page, 'Ramponiert while discarding');
  await page.locator('[data-action="select"]').nth(0).click();
  await page.locator('[data-action="select"]').nth(1).click();
  await page.click('[data-action="discard"]');
  await page.waitForSelector('.declare[data-phase="DeclaringAfterSkat"]');
  await page.click('[data-action="pick-game"][data-index="4"]');
  await page.click('[data-action="declare"]');
  await page.waitForSelector('[data-action="play"]', { timeout: 30000 });
  log(`Ramponiert: ${lockedCount} card(s) locked while discarding`);

  // "Unter ehrenhalber": a plaque on the card.
  await resumeFixture(page, 'honorary-jack.json');
  await page.waitForSelector('.declare');
  if (!(await page.locator('.hand-card .plaque-hj').count())) fail('the Unter ehrenhalber has no plaque');
  await shoot(page, 'honorary-jack');
  log('Unter ehrenhalber plaque shown');
  await context.close();
}

/** A save code without compression, made here (format: web/src/savecode.ts). */
function plainSaveCode(keys) {
  const payload = Buffer.from(JSON.stringify({ v: 1, keys }), 'utf8').toString('base64url');
  const body = `j:${payload}`;
  return `STAMMTISCH1:${body}:${crc32(body).toString(16).padStart(8, '0')}`;
}

/** The app's keys in localStorage. */
async function storedKeys(page) {
  return page.evaluate(() => {
    const keys = {};
    for (let i = 0; i < localStorage.length; i++) {
      const key = localStorage.key(i);
      if (key?.startsWith('stammtisch.')) keys[key] = localStorage.getItem(key);
    }
    return keys;
  });
}

/** What the table shows of the run: seed, account, passes, hand. */
async function tableState(page) {
  const seed = await menuSeed(page);
  await closeSheet(page);
  return {
    seed,
    account: await text(page, 'account'),
    passes: await text(page, 'passes-left'),
    hand: await page.locator('.hand-card').evaluateAll((els) => els.map((e) => e.getAttribute('aria-label')).join(', ')),
  };
}

/** Paste a text into "Spielstand laden" and press "Prüfen". */
async function pasteSaveCode(page, code) {
  await page.fill('[data-testid="save-input"]', code);
  await page.click('[data-action="save-check"]');
  await page.waitForSelector('[data-testid="save-error"], [data-testid="save-contains"]');
}

/** The menu fits the viewport (it scrolls inside), the page itself does not scroll. */
async function assertMenuFits(page, label) {
  const r = await page.evaluate(async () => {
    await Promise.all(document.getAnimations().map((a) => a.finished.catch(() => undefined)));
    const sheet = document.querySelector('.layer .sheet')?.getBoundingClientRect();
    const doc = document.scrollingElement;
    return { top: sheet?.top ?? -1, bottom: sheet?.bottom ?? 1e9, height: innerHeight, scroll: (doc?.scrollHeight ?? 0) - innerHeight };
  });
  if (r.top < 0 || r.bottom > r.height + 1) fail(`${label}: dialog ${r.top}..${r.bottom} leaves the viewport of ${r.height}`);
  if (r.scroll > 1) fail(`${label}: the page scrolls by ${r.scroll}px`);
}

/**
 * The menu needs no scrolling (the player does not want any): at 390 × 844
 * and 375 × 667 the dialog has no inner overflow and "Schließen" is fully
 * in view. Leaves the viewport as it found it.
 */
async function assertMenuNoScroll(page, label) {
  const original = page.viewportSize();
  for (const size of [PHONE, { width: 375, height: 667 }]) {
    await page.setViewportSize(size);
    const r = await page.evaluate(async () => {
      await new Promise((resolve) => requestAnimationFrame(() => resolve()));
      await Promise.all(document.getAnimations().map((a) => a.finished.catch(() => undefined)));
      const sheet = document.querySelector('.layer .sheet-menu');
      const close = document.querySelector('.layer [data-testid="menu-close"]')?.getBoundingClientRect();
      return {
        found: Boolean(sheet && close),
        overflow: sheet ? sheet.scrollHeight - sheet.clientHeight : 0,
        closeTop: close?.top ?? -1,
        closeBottom: close?.bottom ?? 1e9,
        height: innerHeight,
      };
    });
    const where = `${label} at ${size.width}x${size.height}`;
    if (!r.found) fail(`${where}: menu or its "Schließen" button missing`);
    if (r.overflow > 0) fail(`${where}: the menu scrolls inside by ${r.overflow}px`);
    if (r.closeTop < 0 || r.closeBottom > r.height) fail(`${where}: "Schließen" at ${r.closeTop}..${r.closeBottom} is not in view (${r.height})`);
  }
  log(`${label}: menu fits 390x844 and 375x667 without scrolling`);
  if (original) await page.setViewportSize(original);
}

/** Export the save code, clear the browser, import it again. */
async function saveCode(browser, errors, reducedMotion, viewport, shots) {
  const label = `save code ${viewport.width}x${viewport.height}`;
  const context = await quietContext(browser, { viewport, reducedMotion });
  await context.grantPermissions(['clipboard-read', 'clipboard-write'], { origin: base.replace(/\/$/, '') });
  // Count the requests for persistent storage.
  await context.addInitScript(() => {
    window.__persistCalls = 0;
    if (navigator.storage) {
      navigator.storage.persist = () => {
        window.__persistCalls += 1;
        return Promise.resolve(true);
      };
    }
  });
  const page = await context.newPage();
  watch(page, errors);

  await page.goto(base);
  await page.waitForSelector('[data-action="new-run"]');
  if ((await page.evaluate(() => window.__persistCalls)) !== 0) fail(`${label}: persistent storage asked for on page load`);
  await page.fill('#seed', SEED);
  await page.click('[data-action="new-run"]');
  await page.waitForSelector('[data-testid="skat-choice"]');
  await page.waitForFunction(() => window.__persistCalls === 1).catch(() => fail(`${label}: persistent storage not asked for after the run started`));
  const passes = Number(await text(page, 'passes-left'));
  await page.click('[data-action="pass"]');
  await page.waitForFunction(
    (n) => document.querySelector('[data-testid="passes-left"]')?.textContent?.trim() === String(n - 1),
    passes,
  );
  await skatThenGrand(page, label);
  for (let i = 0; i < 2; i++) {
    await page.waitForSelector('[data-action="play"]', { timeout: 30000 });
    await page.locator('[data-action="play"]').first().click();
  }
  await page.waitForSelector('[data-action="play"]', { timeout: 30000 });
  await page.waitForFunction(() => !document.querySelector('.run.is-busy'));
  const before = await tableState(page);
  const keysBefore = await storedKeys(page);
  if (!keysBefore['stammtisch.run'] || !keysBefore['stammtisch.meta']) fail(`${label}: run or meta not stored`);

  // Export from the menu.
  await page.click('.hud [data-action="menu"]');
  await page.waitForSelector('[data-testid="save-menu"]');
  if (shots) await shoot(page, 'menu');
  await assertMenuFits(page, `${label}, menu`);
  await assertMenuNoScroll(page, `${label}, menu in a run`);
  await page.click('[data-action="copy-link"]');
  await page.waitForSelector('[data-testid="share-copied"], [data-testid="share-fallback"]');
  await assertMenuNoScroll(page, `${label}, menu after copying the link`);
  await page.click('[data-testid="save-menu"]');
  await page.waitForSelector('[data-testid="save-export"]');
  if (shots) await shoot(page, 'save-menu');
  await page.click('[data-testid="save-export"]');
  await page.waitForSelector('[data-testid="save-code"]');
  const code = await page.locator('[data-testid="save-code"]').inputValue();
  if (!/^STAMMTISCH1:[dj]:[A-Za-z0-9_-]+:[0-9a-f]{8}$/.test(code)) fail(`${label}: save code looks wrong: ${code.slice(0, 40)}…`);
  await page.click('[data-action="save-copy"]');
  await page.waitForSelector('[data-testid="save-copied"], [data-testid="save-copy-failed"]');
  if (await page.locator('[data-testid="save-copied"]').count()) {
    const copied = await page.evaluate(() => navigator.clipboard.readText());
    if (copied !== code) fail(`${label}: the clipboard holds something else than the code`);
    log(`${label}: code copied to the clipboard`);
  } else {
    log(`${label}: clipboard refused, code marked for copying by hand`);
  }
  if (shots) await shoot(page, 'save-export');
  await assertMenuFits(page, `${label}, export`);
  log(`${label}: code exported, ${code.length} characters (${code.slice(0, 14)}…)`);

  // A cleared browser: no run, a fresh meta state.
  await page.evaluate(() => localStorage.clear());
  await page.reload();
  await page.waitForSelector('[data-action="new-run"]');
  if (await page.locator('[data-action="resume"]').count()) fail(`${label}: a run is offered after clearing the storage`);

  // Import from the menu of the start screen; broken codes are rejected.
  await page.click('[data-action="settings"]');
  await page.waitForSelector('[data-testid="save-menu"]');
  await assertMenuNoScroll(page, `${label}, menu on the start screen`);
  if (shots) await shoot(page, 'menu-start');
  await page.click('[data-testid="save-menu"]');
  await page.click('[data-testid="save-import"]');
  await page.waitForSelector('[data-testid="save-input"]');
  const parts = code.split(':');
  const payload = parts[2];
  const flip = payload[20] === 'A' ? 'B' : 'A';
  const typo = [parts[0], parts[1], payload.slice(0, 20) + flip + payload.slice(21), parts[3]].join(':');
  await pasteSaveCode(page, typo);
  const typoError = await text(page, 'save-error');
  if (!typoError.includes('beschädigt')) fail(`${label}: a typo in the code gives: ${typoError || 'no error'}`);
  await pasteSaveCode(page, code.slice(0, Math.floor(code.length / 2)));
  if (!(await text(page, 'save-error')).includes('beschädigt')) fail(`${label}: a cut-off code is not rejected`);
  await pasteSaveCode(page, 'Hallo Stammtisch');
  if (!(await text(page, 'save-error')).includes('kein Stammtisch-Spielstand')) fail(`${label}: foreign text not rejected`);
  // Fine checksum, but the run inside is no run the core can load.
  await pasteSaveCode(page, plainSaveCode({ 'stammtisch.run': '{"version":1,"run":{}}' }));
  if (!(await text(page, 'save-error')).includes('passt nicht')) fail(`${label}: a run the core rejects is not refused`);
  if ((await storedKeys(page))['stammtisch.run']) fail(`${label}: a rejected code wrote a run`);
  await pasteSaveCode(page, typo);
  if (shots) await shoot(page, 'save-import-error');
  log(`${label}: damaged code rejected: ${typoError}`);

  // The real code, with line breaks as from a messenger: confirm, reload.
  await pasteSaveCode(page, `Mein Spielstand:\n${code.slice(0, 60)}\n${code.slice(60)}\n`);
  const contains = ((await page.locator('[data-testid="save-contains"]').textContent()) ?? '').replace(/\s+/g, ' ').trim();
  if (!contains.includes('Laufender Run')) fail(`${label}: confirmation does not name the run: ${contains}`);
  if (shots) await shoot(page, 'save-confirm');
  await assertMenuFits(page, `${label}, confirm`);
  await Promise.all([page.waitForEvent('load'), page.click('[data-action="save-apply"]')]);
  await page.waitForSelector('[data-action="resume"]');
  const keysAfter = await storedKeys(page);
  if (JSON.stringify(Object.entries(keysAfter).sort()) !== JSON.stringify(Object.entries(keysBefore).sort())) {
    fail(`${label}: stored keys differ after the import: ${Object.keys(keysAfter).join(', ')}`);
  }
  await page.click('[data-action="resume"]');
  await page.waitForSelector('[data-action="play"]', { timeout: 30000 });
  const after = await tableState(page);
  for (const key of Object.keys(before)) {
    if (after[key] !== before[key]) fail(`${label}: ${key} after the import is ${after[key]}, was ${before[key]}`);
  }
  log(`${label}: imported, run goes on: seed ${after.seed}, account ${after.account}, ${after.hand.split(', ').length} cards`);
  await page.locator('[data-action="play"]').first().click();
  await page.waitForSelector('[data-action="play"], .score-sheet', { timeout: 30000 });
  await context.close();
}

/** The hint on screen (its id), or null. */
async function hintId(page) {
  const hint = page.locator('[data-testid="hint"]');
  return (await hint.count()) ? hint.getAttribute('data-hint') : null;
}

/** Wait for a hint of the Wirt and return its text. */
async function expectHint(page, id) {
  await page.waitForSelector(`[data-testid="hint"][data-hint="${id}"]`, { timeout: 15000 }).catch(async () => {
    fail(`hint "${id}" not shown (on screen: ${await hintId(page)})`);
  });
  await page.evaluate(() => Promise.all(document.getAnimations().map((a) => a.finished.catch(() => undefined))));
  return ((await page.locator('[data-testid="hint"] .hint-text').textContent()) ?? '').trim();
}

/** Tap the bubble; it must go and be stored as seen. */
async function dismissHint(page, id) {
  await page.click(`[data-testid="hint"][data-hint="${id}"]`);
  await page.waitForSelector(`[data-testid="hint"][data-hint="${id}"]`, { state: 'detached' });
  const seen = await page.evaluate(() => JSON.parse(localStorage.getItem('stammtisch.hints') ?? '[]'));
  if (!seen.includes(id)) fail(`hint "${id}" dismissed but not stored as seen: ${JSON.stringify(seen)}`);
}

/** What a hint must never cover: the hand and the buttons of the table, the score, the boss, the Tresen, the encounter, the end. */
const HINT_KEEP_CLEAR =
  '.hand-card, .declare button, .status .btn, [data-action="score-continue"], [data-action="score-skip"].btn, [data-action="boss-go"], [data-action="leave-shop"], [data-action="buy-relic"], [data-action="buy-item"], [data-action="choose-encounter"], [data-action="copy-link"], .end-actions .btn';

/**
 * The bubble is inside the viewport, makes nothing scroll and covers none
 * of HINT_KEEP_CLEAR, at 390 × 844 and 375 × 667 (or the given sizes).
 */
async function assertHintClear(page, label, sizes = [PHONE, { width: 375, height: 667 }]) {
  const original = page.viewportSize();
  for (const size of sizes) {
    await page.setViewportSize(size);
    // The app follows a resize once more after 100 ms.
    await page.waitForTimeout(220);
    const r = await page.evaluate((keep) => {
      const hint = document.querySelector('[data-testid="hint"]')?.getBoundingClientRect();
      if (!hint) return null;
      const hits = [...document.querySelectorAll(keep)]
        .filter((el) => {
          const b = el.getBoundingClientRect();
          if (!b.width || !b.height) return false;
          return b.left < hint.right && b.right > hint.left && b.top < hint.bottom && b.bottom > hint.top;
        })
        .map((el) => el.getAttribute('data-action') ?? el.className);
      return { top: hint.top, bottom: hint.bottom, height: innerHeight, scroll: document.scrollingElement.scrollHeight - innerHeight, hits };
    }, HINT_KEEP_CLEAR);
    const where = `${label} at ${size.width}x${size.height}`;
    if (!r) fail(`${where}: no hint on screen`);
    if (r.top < 0 || r.bottom > r.height) fail(`${where}: the hint at ${r.top}..${r.bottom} leaves the viewport of ${r.height}`);
    if (r.scroll > 0) fail(`${where}: the page scrolls by ${r.scroll}px with the hint`);
    if (r.hits.length) fail(`${where}: the hint covers ${r.hits.join(', ')}`);
  }
  if (original) await page.setViewportSize(original);
  log(`${label}: hint clear of hand and buttons at ${sizes.map((s) => `${s.width}x${s.height}`).join(', ')}`);
}

/**
 * "Geführter erster Abend": a fresh profile sees the Wirt's hints one at a
 * time, each once; dismissed hints stay away after a reload; the menu
 * switches them off and resets them; the Regelkarte opens and closes. The
 * table still fits with a hint on screen.
 */
async function hintsAndRules(browser, errors, reducedMotion) {
  const context = await browser.newContext({ viewport: PHONE, reducedMotion });
  const page = await context.newPage();
  watch(page, errors);

  // The Regelkarte from the menu of the start screen.
  await page.goto(base);
  await page.waitForSelector('[data-action="new-run"]');
  if (await page.locator('[data-testid="hint"]').count()) fail('a hint on the start screen');
  await page.click('[data-action="settings"]');
  await assertMenuNoScroll(page, 'menu with the Wirt\'s hints (start screen)');
  await page.click('[data-testid="rules-open"]');
  await page.waitForSelector('[data-testid="rules"]');
  const rules = ((await page.locator('[data-testid="rules"]').textContent()) ?? '').replace(/\s+/g, ' ');
  for (const must of ['Skat oder Hand', 'Drücken', 'Grundwert × Gewinnstufen', 'Schellen', 'Eichel', 'Grand', 'Null', 'Kein Reizen', 'Boss', 'Tresen']) {
    if (!rules.includes(must)) fail(`the Regelkarte lacks "${must}"`);
  }
  await assertMenuFits(page, 'Regelkarte');
  await shoot(page, 'rules');
  await page.click('[data-testid="rules-close"]');
  await page.waitForSelector('[data-testid="rules"]', { state: 'detached' });
  log('Regelkarte opened and closed');

  // First run: the welcome, then Skat oder Hand.
  await page.fill('#seed', SEED);
  await page.click('[data-action="new-run"]');
  const welcome = await expectHint(page, 'welcome');
  log(`hint welcome: ${welcome}`);
  await shoot(page, 'hint-welcome');
  await assertHintClear(page, 'hint welcome');
  await assertFits(page, 'Skat oder Hand with a hint', [PHONE, { width: 375, height: 667 }]);
  await dismissHint(page, 'welcome');
  const skat = await expectHint(page, 'skatOrHand');
  log(`hint skatOrHand: ${skat}`);

  // Reload: the welcome stays away, the hint not yet dismissed comes back.
  await page.reload();
  await page.click('[data-action="resume"]');
  await expectHint(page, 'skatOrHand');
  if ((await hintId(page)) === 'welcome') fail('the dismissed welcome came back after a reload');
  await shoot(page, 'hint-skat');
  await assertHintClear(page, 'hint Skat oder Hand');
  await dismissHint(page, 'skatOrHand');
  log('after reload: welcome stays dismissed');

  // A pass.
  await page.click('[data-action="pass"]');
  log(`hint pass: ${await expectHint(page, 'pass')}`);
  await shoot(page, 'hint-pass');
  await dismissHint(page, 'pass');

  // The Ansage after the skat.
  await pickUpAndDiscard(page, 'hints');
  log(`hint afterSkat: ${await expectHint(page, 'afterSkat')}`);
  await shoot(page, 'hint-after-skat');
  await assertHintClear(page, 'hint after the skat');
  await assertFits(page, 'Ansage after the skat with a hint', [PHONE, { width: 375, height: 667 }]);
  await dismissHint(page, 'afterSkat');
  await page.click('[data-action="pick-game"][data-index="4"]');
  await page.click('[data-action="declare"]');
  await page.waitForSelector('.declare', { state: 'detached' });

  // Playing: the value line. Left alone, it goes by itself at the end of the game.
  const value = await expectHint(page, 'value');
  if (!/\d+ × \d+/.test(value)) fail(`the value hint does not name the numbers of the table: ${value}`);
  log(`hint value: ${value}`);
  await page.waitForSelector('[data-action="play"]', { timeout: 30000 });
  await shoot(page, 'hint-value');
  await assertHintClear(page, 'hint value line');
  await assertFits(page, 'playing with a hint', [PHONE, { width: 375, height: 667 }]);
  await page.waitForSelector('[data-action="play"]', { timeout: 30000 });
  // Play out (the bubble must not be in the way of a card), up to the score board.
  for (let played = 0; ; played++) {
    await page.waitForSelector('[data-action="play"], .score-sheet', { timeout: 30000 });
    if (await page.locator('.score-sheet').count()) break;
    await page.locator('[data-action="play"]').first().click();
    if (played > 10) fail('hints: played more than ten cards');
  }
  log(`hint score: ${await expectHint(page, 'score')}`);
  await page.waitForSelector('[data-action="score-continue"]');
  await shoot(page, 'hint-score');
  await assertHintClear(page, 'hint score');
  await dismissHint(page, 'score');
  await page.click('[data-action="score-continue"]');
  await page.waitForSelector('.score-sheet', { state: 'detached' });
  // The unlock comes first; the Tresen hint waits for it.
  await page.waitForSelector('[data-testid="unlock-new"]');
  if (await page.locator('[data-testid="hint"]').count()) fail('a hint over the unlock dialog');
  await page.click('.sheet-unlock [data-action="close-overlay"]');
  log(`hint shop: ${await expectHint(page, 'shop')}`);
  const seenValue = await page.evaluate(() => JSON.parse(localStorage.getItem('stammtisch.hints') ?? '[]'));
  if (!seenValue.includes('value')) fail('the value hint, left alone, did not count as seen once the game was over');
  await shoot(page, 'hint-shop');
  await assertHintClear(page, 'hint Tresen');

  // Menu: off hides it, on brings it back (it was not dismissed).
  await page.click('.hud [data-action="menu"]');
  if (await page.locator('[data-testid="hint"]').count()) fail('the hint shows over the menu');
  await assertMenuNoScroll(page, 'menu with the Wirt\'s hints (run)');
  await page.click('[data-action="hints"][data-on="false"]');
  const off = await page.evaluate(() => JSON.parse(localStorage.getItem('stammtisch.settings') ?? '{}').hints);
  if (off !== false) fail(`hints setting not stored: ${off}`);
  await closeSheet(page);
  await page.waitForTimeout(200);
  if (await page.locator('[data-testid="hint"]').count()) fail('a hint shows although the hints are off');
  log('hints off: no bubble');
  await page.click('.hud [data-action="menu"]');
  await page.click('[data-action="hints"][data-on="true"]');
  await closeSheet(page);
  await expectHint(page, 'shop');
  await dismissHint(page, 'shop');

  // Reset: every hint comes back, here the one of the Tresen.
  await page.click('.hud [data-action="menu"]');
  await page.click('[data-testid="hints-reset"]');
  await closeSheet(page);
  await expectHint(page, 'shop');
  const afterReset = await page.evaluate(() => JSON.parse(localStorage.getItem('stammtisch.hints') ?? '[]'));
  if (afterReset.length) fail(`hints still stored as seen after the reset: ${JSON.stringify(afterReset)}`);
  log('hints reset: the Tresen hint is back');
  await dismissHint(page, 'shop');

  // The boss and the encounter, from saved runs, with a fresh profile.
  await page.evaluate(() => localStorage.removeItem('stammtisch.hints'));
  await resumeFixture(page, 'boss-tresen.json');
  await dismissHint(page, 'shop');
  await page.click('[data-action="leave-shop"]');
  await page.waitForSelector('[data-action="boss-go"]');
  log(`hint boss: ${await expectHint(page, 'boss')}`);
  await shoot(page, 'hint-boss');
  await assertHintClear(page, 'hint boss');
  await page.click('[data-action="boss-go"]');
  await page.waitForSelector('.declare');
  await resumeFixture(page, 'encounter.json');
  log(`hint encounter: ${await expectHint(page, 'encounter')}`);
  await shoot(page, 'hint-encounter');
  await assertHintClear(page, 'hint encounter');

  // A lost run: what stays.
  await resumeFixture(page, 'run-end.json');
  await page.waitForSelector('.declare');
  await skatThenGrand(page, 'hints, last game');
  await playOut(page, 'hints, last game');
  await page.click('[data-action="score-continue"]');
  await page.waitForSelector('[data-testid="end"]');
  if (await page.locator('.sheet-unlock').count()) await page.click('.sheet-unlock [data-action="close-overlay"]');
  log(`hint runLost: ${await expectHint(page, 'runLost')}`);
  await shoot(page, 'hint-run-lost');
  await assertHintClear(page, 'hint run lost');
  await dismissHint(page, 'runLost');
  await context.close();
}

async function run() {
  const preview = await startPreview();
  const browser = await chromium.launch();
  const errors = [];
  try {
    const reducedMotion = process.env.SMOKE_FULL_MOTION ? 'no-preference' : 'reduce';
    const context = await quietContext(browser, { viewport: PHONE, reducedMotion });
    const page = await context.newPage();
    watch(page, errors);

    // Start screen and a new run with a fixed seed.
    await page.goto(base);
    await page.waitForSelector('[data-action="new-run"]');
    const build = await text(page, 'build');
    if (!build) fail('build info missing on the start screen');
    const sprite = await page.evaluate(() => ['f-CA', 'f-SJ', 'f-HQ', 'f-DK', 'f-back', 'f-blank', 'p-wirt'].filter((id) => !document.getElementById(id)));
    if (sprite.length) fail(`card sprite incomplete, missing: ${sprite.join(', ')}`);
    const aceCorner = await page.evaluate(() => document.querySelector('#f-CA .idx')?.textContent?.trim());
    if (aceCorner !== 'A') fail(`the Daus shows "${aceCorner}" in its corner, expected "A"`);
    const fonts = await page.evaluate(async () => {
      await document.fonts.ready;
      const families = ['Kenney Mini', 'Kenney Mini Square'];
      await Promise.all(families.map((f) => document.fonts.load(`16px "${f}"`, 'Skat')));
      const loaded = [...document.fonts].filter((f) => f.status === 'loaded').map((f) => f.family.replace(/"/g, ''));
      return families.filter((f) => !loaded.includes(f)).map((f) => `${f} (loaded: ${loaded.join(', ') || 'none'})`);
    });
    if (fonts.length) fail(`fonts not loaded: ${fonts.join(', ')}`);
    const portraits = await page.evaluate(() => ['p-char-Declarer', 'p-char-NullPlayer', 'p-dealer', 'p-boss-the_old_man'].filter((id) => !document.getElementById(id)));
    if (portraits.length) fail(`portraits missing: ${portraits.join(', ')}`);
    // Every Stammgast of the core content has a face drawn in ui/people.ts.
    const looks = guestLooks();
    if (looks.length < 12) fail(`only ${looks.length} Stammgäste in guests.ron`);
    const faceless = await page.evaluate((ids) => ids.filter((look) => !document.getElementById(`p-guest-${look}`)), looks);
    if (faceless.length) fail(`Stammgäste without a face in ui/people.ts: ${faceless.join(', ')}`);
    log(`${looks.length} Stammgäste, each with a face`);

    // A new player: fresh meta state, Alleinspieler picked, Nullspieler locked.
    const fresh = await storedMeta(page);
    if (fresh.version !== 1 || !fresh.meta) fail(`unexpected meta save: ${JSON.stringify(fresh).slice(0, 120)}`);
    await page.waitForSelector('[data-action="pick-character"][data-character="Declarer"][aria-pressed="true"]');
    const locked = page.locator('.char-card.is-locked[data-character="NullPlayer"]');
    if (!(await locked.count())) fail('the Nullspieler is not shown as locked');
    const condition = (await locked.textContent()) ?? '';
    if (!condition.includes('Alleinspieler')) fail(`locked Nullspieler without his condition: ${condition}`);
    log('start screen: Alleinspieler picked, Nullspieler locked');
    await shoot(page, 'start-characters');
    await page.click('[data-action="book"]');
    await page.waitForSelector('.ledger [data-testid="unlocks"]');
    const rows = await page.locator('.ledger-row').count();
    const open = await page.locator('.ledger-row.is-open').count();
    if (rows < 5 || open !== rows) fail(`Vereinsbuch of a new player: ${rows} unlocks, ${open} open`);
    log(`Vereinsbuch: ${rows} unlocks, all open`);
    await shoot(page, 'book-fresh');
    await page.click('.ledger [data-action="close-overlay"]');
    await page.waitForSelector('.ledger', { state: 'detached' });

    await page.fill('#seed', SEED);
    await page.click('[data-action="new-run"]');
    await page.waitForSelector('[data-testid="skat-choice"]');
    log(`run started with seed ${SEED} (${build})`);
    const seated = await assertGuests(page, 'first series');
    if (!seated.quip) fail('no Stammgast says hello at the first game of the series');
    await shoot(page, 'table-declare');
    await assertFits(page, 'Skat oder Hand');
    await assertFits(page, 'Skat oder Hand with a quip', [PHONE, { width: 375, height: 667 }]);

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

    // Skat oder Hand: the hand Ansage offers only the hand levels.
    const accountBefore = await text(page, 'account');
    await page.click('[data-action="play-hand"]');
    await page.waitForSelector('.declare-hand');
    const handRisks = await page.locator('[data-action="pick-risk"]').evaluateAll((els) => els.map((e) => e.getAttribute('data-risk')).join(','));
    if (handRisks !== 'Hand,SchneiderDeclared,SchwarzDeclared,Ouvert') fail(`hand Ansage offers ${handRisks}`);
    await shoot(page, 'table-declare-hand');
    await assertFits(page, 'hand Ansage');
    await page.click('[data-action="hand-back"]');
    log(`hand Ansage: ${handRisks}`);

    // Skat aufnehmen, drücken, then the Ansage: Grand.
    await pickUpAndDiscard(page, 'first game', 'table-discard');
    if (await page.locator('[data-action="pick-risk"]').count()) fail('hand levels after the skat');
    await page.click('[data-action="pick-game"][data-index="4"]');
    await shoot(page, 'table-declare-after');
    await assertFits(page, 'Ansage after the skat');
    await page.click('[data-action="declare"]');
    await page.waitForSelector('.declare', { state: 'detached' });
    log('skat picked up, two cards pressed, Grand declared');

    // Ten tricks: always the first legal card.
    let played = 0;
    for (;;) {
      await page.waitForSelector('[data-action="play"], .score-sheet', { timeout: 30000 });
      if (await page.locator('.score-sheet').count()) break;
      const dimmed = await page.locator('.hand-card.is-dim').count();
      if (played === 0) {
        // The value at stake comes from the core: "Grundwert × Stufen".
        const worth = await page.locator('[data-testid="value-preview"]').textContent();
        if (!/^\d+ × \d+( × \d+)?$/.test((worth ?? '').trim())) fail(`value preview while playing: "${worth}"`);
        log(`value preview: ${worth}`);
        await shoot(page, 'table-play');
        await assertFits(page, 'playing');
      }
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
    // Won the series with games to spare: the meta state unlocks something.
    await page.waitForSelector('[data-testid="unlock-new"]', { timeout: 10000 }).catch(() => fail('no unlock announced after winning the first series early'));
    const unlocked = ((await page.locator('[data-testid="unlock-new"]').textContent()) ?? '').replace(/\s+/g, ' ').trim();
    log(`unlock announced: ${unlocked}`);
    if (!(await page.locator('[data-testid="unlock-kind"]').count())) fail('the unlock does not name the kind of its reward');
    await shoot(page, 'unlock');
    await page.click('.sheet-unlock [data-action="close-overlay"]');
    await page.waitForSelector('[data-testid="unlock-new"]', { state: 'detached' });
    const learned = await storedMeta(page);
    if ((learned.meta.stats?.games_won ?? 0) < 1) fail(`meta state did not count the won game: ${JSON.stringify(learned.meta.stats)}`);
    if (!(learned.meta.unlocked ?? []).length) fail('meta state stored no unlock');
    const accountAfter = await text(page, 'account');
    if (accountAfter === accountBefore) fail(`account unchanged: ${accountAfter}`);
    log(`account ${accountBefore} -> ${accountAfter}`);

    // Tresen.
    await page.waitForSelector('[data-testid="shop"]');
    await expectMusic(page, 'shop', 'Tresen');
    const coinsBefore = Number(await text(page, 'coins'));
    const next = await text(page, 'next-series');
    if (!next) fail('the Tresen shows no preview of the next series');
    log(`next series: ${next.replace(/\s+/g, ' ')}`);
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
    // Kreidetafel: the tour with the won series ticked off.
    await page.click('.hud [data-action="overview"]');
    await page.waitForSelector('.tour-grid');
    if ((await page.locator('.tour-cell.is-done').count()) < 1) fail('Kreidetafel shows no finished series');
    await shoot(page, 'overview');
    await page.click('.sheet [data-action="close-overlay"]');
    await page.waitForSelector('.tour-grid', { state: 'detached' });

    // Menu: the tempo setting is stored in the browser.
    await page.click('[data-action="menu"]');
    await page.click('[data-action="tempo"][data-tempo="fast"]');
    const tempo = await page.evaluate(() => JSON.parse(localStorage.getItem('stammtisch.settings') ?? '{}').tempo);
    if (tempo !== 'fast') fail(`tempo setting not stored: ${tempo}`);
    await page.click('.sheet [data-action="close-overlay"]');
    log('Kreidetafel and menu checked');

    await page.click('[data-action="leave-shop"]');
    await page.waitForSelector('.declare, [data-action="boss-go"]');
    log('next series started');
    if (await page.locator('.declare').count()) {
      await expectMusic(page, 'table', 'second series');
      // New series, new Stammgäste: neither of the first two sits down again.
      const next = await assertGuests(page, 'second series');
      const before = seated.opps.map((o) => o.guest);
      const again = next.opps.filter((o) => before.includes(o.guest));
      if (again.length) fail(`the same Stammgast in two series in a row: ${again.map((o) => o.name).join(', ')}`);
    }

    // Reload once more: the run is still there.
    await page.reload();
    await page.waitForSelector('[data-action="resume"]');
    log('saved after the Tresen');
    await context.close();

    await musicPass(browser, errors, reducedMotion);
    await bossAndKontra(browser, errors, reducedMotion);
    await m4(browser, errors, reducedMotion);
    await m5(browser, errors, reducedMotion);
    await saveCode(browser, errors, reducedMotion, PHONE, true);
    await saveCode(browser, errors, reducedMotion, DESKTOP, false);
    await hintsAndRules(browser, errors, reducedMotion);
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

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
// The card sprite (German deck) and the shipped fonts must be present.
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
  if (shot) await shoot(page, shot);
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

/** Boss dialog and Kontra dialog, from saved runs. */
async function bossAndKontra(browser, errors, reducedMotion) {
  const context = await browser.newContext({ viewport: PHONE, reducedMotion });
  const page = await context.newPage();
  watch(page, errors);

  // Tresen before the boss series: the preview names the boss.
  await resumeFixture(page, 'boss-tresen.json');
  await page.waitForSelector('[data-testid="shop"]');
  const next = await text(page, 'next-series');
  if (!next.includes('Kontra-Hans')) fail(`the Tresen does not announce the boss: ${next}`);
  await shoot(page, 'shop-boss');
  await page.click('[data-action="leave-shop"]');
  await page.waitForSelector('[data-action="boss-go"]');
  const bossText = (await page.locator('.sheet-boss').textContent()) ?? '';
  if (!bossText.includes('Kontra-Hans')) fail(`boss dialog without the boss: ${bossText}`);
  log('boss dialog shown');
  await shoot(page, 'boss');
  await page.click('[data-action="boss-go"]');
  await page.waitForSelector('.declare');
  if (!(await page.locator('.opp.is-boss').count())) fail('the boss does not sit at the table');
  await shoot(page, 'table-boss');

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
  await context.close();
}

/** M4: the Nullspieler, encounters, the Ramsch and "Der Alte". */
async function m4(browser, errors, reducedMotion) {
  const context = await browser.newContext({ viewport: PHONE, reducedMotion });
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
  const context = await browser.newContext({ viewport: PHONE, reducedMotion });
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

async function run() {
  const preview = await startPreview();
  const browser = await chromium.launch();
  const errors = [];
  try {
    const reducedMotion = process.env.SMOKE_FULL_MOTION ? 'no-preference' : 'reduce';
    const context = await browser.newContext({ viewport: PHONE, reducedMotion });
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
      return ['Grenze Gotisch', 'Alegreya Sans'].filter((f) => !document.fonts.check(`16px "${f}"`, 'Skat'));
    });
    if (fonts.length) fail(`fonts not loaded: ${fonts.join(', ')}`);
    const portraits = await page.evaluate(() => ['p-char-Declarer', 'p-char-NullPlayer', 'p-dealer', 'p-boss-the_old_man'].filter((id) => !document.getElementById(id)));
    if (portraits.length) fail(`portraits missing: ${portraits.join(', ')}`);

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

    // Skat oder Hand: the hand Ansage offers only the hand levels.
    const accountBefore = await text(page, 'account');
    await page.click('[data-action="play-hand"]');
    await page.waitForSelector('.declare-hand');
    const handRisks = await page.locator('[data-action="pick-risk"]').evaluateAll((els) => els.map((e) => e.getAttribute('data-risk')).join(','));
    if (handRisks !== 'Hand,SchneiderDeclared,SchwarzDeclared,Ouvert') fail(`hand Ansage offers ${handRisks}`);
    await shoot(page, 'table-declare-hand');
    await page.click('[data-action="hand-back"]');
    log(`hand Ansage: ${handRisks}`);

    // Skat aufnehmen, drücken, then the Ansage: Grand.
    await pickUpAndDiscard(page, 'first game', 'table-discard');
    if (await page.locator('[data-action="pick-risk"]').count()) fail('hand levels after the skat');
    await page.click('[data-action="pick-game"][data-index="4"]');
    await shoot(page, 'table-declare-after');
    await page.click('[data-action="declare"]');
    await page.waitForSelector('.declare', { state: 'detached' });
    log('skat picked up, two cards pressed, Grand declared');

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

    // Reload once more: the run is still there.
    await page.reload();
    await page.waitForSelector('[data-action="resume"]');
    log('saved after the Tresen');
    await context.close();

    await bossAndKontra(browser, errors, reducedMotion);
    await m4(browser, errors, reducedMotion);
    await m5(browser, errors, reducedMotion);
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

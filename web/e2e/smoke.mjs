// Smoke test of the built web app (web/dist) with Playwright and Chromium.
//
//   npm run build && node e2e/smoke.mjs
//
// Starts `vite preview`, then on a phone viewport (390 × 844):
// - checks the start screen of a new player: a fresh meta state is stored,
//   the Alleinspieler is picked, the Nullspieler is locked with his
//   condition, and the Vereinsbuch lists every unlock as open,
// - starts a run with a fixed seed; Skat oder Hand lies on the felt: the
//   Stammgäste's greetings written out, short labels ("Hand spielen", the
//   deck as "Einmischen") with their explanations on the info card (hover),
//   the skat face down, the hand bigger than under the Ansage; a long press
//   and a drag across the table mark no text; with reduced motion nothing
//   flies in; then it passes once by dragging the deck into the middle,
// - reloads the page and checks that the run is resumed,
// - plays a whole game in the order of the Skatordnung (Skat oder Hand,
//   Skat aufnehmen, Drücken, Ansage, 10 Stiche, Abrechnung) and checks that
//   (Drücken on the table: the picked-up skat lies face up on the table,
//   a hand card dropped onto a table card swaps with it, one dragged back
//   leaves Drücken disabled, taps move cards as the fallback; the click
//   strategy below moves the skat back and taps the first two hand cards)
//   the series account changed; the hand Ansage offers only the hand levels
//   and, for the Alleinspieler too (ADR 0012), Null Hand and Null Ouvert Hand
//   with their values; the Ansage after the skat only the levels without Hand
//   (Null and Null Ouvert too, 23 and 69: 23 × (1, 1 + Ouvert 2), DESIGN 16),
// - sees the unlock earned by winning the series with games left
//   ("Neu am Stammtisch") and the meta state counting the game,
// - while playing sees the pressed cards face down as a small stack in the
//   bottom right corner of the felt (a tap shows both face up), the trick
//   as a triangle (left regular top left, right regular top right, the
//   player's card bottom centre), throws one card with the mouse onto the
//   felt instead of tapping it, and, after the first trick, "Letzter
//   Stich": the last trick with its winner on the felt (in the same
//   triangle) until tapped again, without scrolling,
// - with full motion (tableFeelPass, 390 × 844 and 1280 × 800): a card
//   thrown with a flick flies with a turn, comes down askew and off its
//   place, lies there a moment and slides onto it; an opponent's card
//   flies in from his seat; a card let go over the hand goes back unplayed;
//   one the core does not let go springs back and shakes,
// - "Kartenfarbe tönen" (suitTintPass): the menu switch tints each card's
//   paper in its suit (off by default, stored, survives a reload), and each
//   rank letter in the corner has its suit's dark ink, never black,
// - visits the Tresen, buys something if possible, opens the Kreidetafel and
//   the menu (tempo setting is stored), and starts the next series. The
//   Tresen (and an encounter) never scroll sideways at 390 × 844, 375 × 667
//   and 360 × 740: the page is no wider than the viewport, no element leaves it.
// The card sprite (German deck) and the shipped font (VT323, SIL OFL)
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
//   "Der Alte"; the table must name the rule he lays on this game, and his
//   cards show his own back, not the back of the boss whose rule is on.
// - route-spelunke.json: seed 1 by the RunBot (save version 3) at the
//   route choice after the first pub, one way leading into a Spelunke: the
//   Spelunke stands on the map apart from the next pub, picking its way
//   shows it with its malus on the card.
// - route-station.json: seed 1 at the route into the second district,
//   right after the first boss pub: the map draws the look the core gives
//   (tour.ron, one district of each look per evening), the head shows the
//   district label and no boss chip; with full motion "Los" walks the
//   position along the way with a chalk trail before the stop opens.
// The route choice (routePass) is a city map: the leg as a pixel image
// with three ways, a tap on a transport highlights its way, "−" opens the
// whole tour as a line plan and "+" closes it, the same leg is drawn
// identically after a reload (same seed), and the screen fits every size
// of the table check without scrolling.
// Card backs (ADR 0015): every back of tools/cardart/backs.py is in the
// sprite; the Stammgäste show "regulars" on their fans, a boss series the
// boss's back (Kontra-Hans in the boss dialog and on the fan).
// - oma.json: seed 1720 up to a hand game with "Die Oma" (removed on
//   02.11.2026); kept for the core's load test (the relic drops on load).
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
// - discard-battered.json, honorary-jack.json: old saves with mods that
//   effects v2 removed; no UI step any more, the core's load test uses them.
// - wave2-relics.json (M8, crafted from a fresh run with seed 77): five
//   relics of wave 2a in the head with their own beer mats, and
//   "Stammfarbe" two suit games into its streak (every trump suit flagged).
//   A suit game after the skat shows "× 1,5" in the score line and the
//   Abrechnung names Stammfarbe and Allrounder (Altes Blatt, removed on
//   02.11.2026, drops from the save on load), the factor box
//   showing 1,5.
// - the Ramsch (ramsch.json) score shows Augen and tricks per seat and
//   marks the most.
// - wave3-mods.json (M8 wave 3, an old save: "Wandelkarte" on the Rot-Daus,
//   "Glasdaus" on the Grün-Daus, "Heimweh" on the Eichel-Unter; since
//   effects v2 converted on load to Wandelkarte, Augen +11 and nothing):
//   both legendary beer mats in the orange legendary ink, the player is Vorhand.
//   Eichel Hand, the Grün-Daus led, then a Schellen lead: the Wandelkarte
//   opens the suit picker with only Schellen allowed, and played it lies on
//   the table as the Schellen-Daus with its printed suit as a plaque.
// Effekte v3 and "Untot" (framesPass, modConfirmPass, voucherPass, undeadPass):
// - frames.json: boss-tresen.json with its deck patched by text (mods_version
//   2): every modification of effects v2 once, plus Untot with Augen +5,
//   Wandelkarte with Trumpf +4 and Untot with Ramponiert. On the Kreidetafel
//   (the gallery of every modification) every one is named and drawn as on
//   the board "Effekte v3": its frame, mark and band colour, curses in
//   anthracite without badge, Untot with its zombie face (rotten pips on a
//   number card), the Wandelkarte with its joker face.
// - mods-confirm.json: frames.json with "Augen +3" and "Rang +1" in the bag
//   and 20 Groschen: the in-app sheet before a modification replaces or
//   raises another (see modConfirmPass).
// - voucher.json: frames.json with "Gutschein" on the Grün-Neun and
//   "Gutschein Untot" on the Grün-Acht, both dealt to the player in the boss
//   game: pressed, they are redeemed with their animation (voucherPass).
// - undead.json: wave3-mods.json with "Untot" on the Eichel-Daus (31) and
//   the Grün-König (22), both with their Untot look. In an Eichel Hand
//   both carry the skull plaque (can rise); the Daus led comes back into the hand when the trick is
//   complete (toast "Untot"), the last trick shows the substitute with its
//   note "Ersatz" and not the Daus, and the Daus has no plaque any more.
// M8 wave 2b (wave2b): a new run with seed 64, its save patched in the
// browser (owned relics and the rules of the dealt game, by text so the
// big rng numbers stay exact), then resumed:
// - "Drei drücken" (first game of each pub, "1. Spiel" on its chip, dimmed
//   once used): picking up lays three cards on the table (the skat
//   and the card drawn from the left Stammgast, tagged with his name),
//   Drücken waits for three, a card tapped back into the hand and another
//   tapped out works as with two; after Drücken the Ansage shows two
//   pressed cards and the Stammgast holds the card given back (revealed).
// - "Spickzettel" (second run): after "Hand spielen" and a Grand Hand both
//   skat cards lie face up beside the trick (since 02.11.2026).
// Save code (menu "Spielstand sichern" / "Spielstand laden"), at 390 × 844
// and at 1280 × 800: a run with seed 64 is played into its first game, the
// code is exported from the menu (and copied), localStorage is cleared, a
// damaged code and a foreign text are rejected with a message, the real
// code is imported after the confirmation, and the resumed run has the
// same seed, account, passes and hand; every stored key is back as it was.
// Persistent storage is asked for once, after the run started, not on load.
// All passes above run with the Wirt's hints switched off (quietContext).
//
// Motion (motionPass, always with animations): a deal flies in from the
// deck with varied spins and a second deal differs, the skat turns over on
// the table after picking it up, and finished tricks slide off towards
// their winner (left regular left, right regular right, the player down);
// screenshots trick-leaves-*.
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
// M7 "Kurzer Abend" (shortEvening): the start screen offers "Ganzer Abend"
// and "Kurzer Abend" between the characters and the start button, both
// visible without scrolling at 390 × 844 and 375 × 667; the full evening is
// picked for a new player, the pick is stored. A short evening with seed
// 4343 shows "Abschnitt 1/3" in the HUD, its link ends in
// `#seed-4343-alleinspieler-kurz`, and that link opened in a fresh browser
// offers and starts a short evening again. An old link without `-kurz`
// stays a full evening ("Abschnitt 1/5").
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

/**
 * Nothing scrolls sideways (play-test feedback: the Tresen did on a
 * phone): at 390 × 844, 375 × 667 and 360 × 740 the page is no wider than
 * the viewport, no scroll area inside `root` can be scrolled sideways, and
 * no element of `root` reaches out of the viewport.
 */
async function assertNoSideScroll(page, label, root = '.shop') {
  const original = page.viewportSize();
  for (const size of [PHONE, { width: 375, height: 667 }, { width: 360, height: 740 }]) {
    await page.setViewportSize(size);
    await page.waitForTimeout(150);
    const r = await page.evaluate((sel) => {
      const box = document.querySelector(sel);
      if (!box) return null;
      const name = (el) => `${el.tagName.toLowerCase()}.${String(el.className?.baseVal ?? el.className ?? '').trim().replace(/\s+/g, '.')}`;
      const scrollers = [box, ...box.querySelectorAll('*')]
        .filter((el) => el.scrollWidth > el.clientWidth + 1 && !['visible', 'clip'].includes(getComputedStyle(el).overflowX) && !el.classList.contains('sr-only'))
        .map((el) => `${name(el)} (${el.scrollWidth} > ${el.clientWidth})`);
      // Drawings inside an SVG (the city map) may reach beyond it when a
      // clipping box around them stays inside the viewport.
      const clipped = (el) => {
        if (!el.ownerSVGElement) return false;
        for (let up = el.ownerSVGElement.parentElement; up; up = up.parentElement) {
          if (getComputedStyle(up).overflowX === 'visible') continue;
          const b = up.getBoundingClientRect();
          return b.left >= -0.5 && b.right <= innerWidth + 0.5;
        }
        return false;
      };
      const wide = [...box.querySelectorAll('*')]
        .filter((el) => {
          if (el.closest('.sr-only') || clipped(el)) return false;
          const b = el.getBoundingClientRect();
          return b.width > 0 && (b.left < -0.5 || b.right > innerWidth + 0.5);
        })
        .map((el) => {
          const b = el.getBoundingClientRect();
          return `${name(el)} ${Math.round(b.left)}..${Math.round(b.right)}`;
        });
      return { doc: document.scrollingElement.scrollWidth, width: innerWidth, scrollers, wide };
    }, root);
    const where = `${label} at ${size.width}x${size.height}`;
    if (!r) fail(`${where}: ${root} not on screen`);
    if (r.doc > r.width) fail(`${where}: the page is ${r.doc}px wide, the viewport ${r.width}px`);
    if (r.scrollers.length) fail(`${where}: scrolls sideways: ${r.scrollers.slice(0, 5).join(', ')}`);
    if (r.wide.length) fail(`${where}: out of the viewport: ${r.wide.slice(0, 5).join(', ')}`);
  }
  if (original) await page.setViewportSize(original);
  log(`${label}: nothing scrolls sideways at 390x844, 375x667, 360x740`);
}

/**
 * While playing, the pressed cards lie face down as a small stack in the
 * bottom right corner of the felt (play-test feedback: face up they were
 * too present): only backs, right of the trick, at the bottom of the trick
 * row, a touch target of 44 px. A tap shows them face up on the info card.
 */
async function assertPressed(page, label, shot = null) {
  const original = page.viewportSize();
  for (const size of [PHONE, { width: 375, height: 667 }, DESKTOP]) {
    await page.setViewportSize(size);
    await page.waitForTimeout(120);
    const r = await page.evaluate(() => {
      const stack = document.querySelector('[data-testid="pressed-play"]');
      const row = document.querySelector('.trick-row')?.getBoundingClientRect();
      const trick = document.querySelector('[data-testid="trick"], [data-testid="last-trick"]')?.getBoundingClientRect();
      if (!stack || !row || !trick) return null;
      const box = stack.getBoundingClientRect();
      return {
        backs: stack.querySelectorAll('.pressed-card.is-back').length,
        faces: stack.querySelectorAll('.card-svg, use[href^="#f-"]').length,
        box: { left: box.left, right: box.right, bottom: box.bottom, width: box.width, height: box.height },
        row: { right: row.right, bottom: row.bottom },
        trickRight: trick.right,
      };
    });
    const where = `${label} at ${size.width}x${size.height}`;
    if (!r) fail(`${where}: no pressed stack on the felt`);
    if (r.backs !== 2 || r.faces) fail(`${where}: the pressed stack shows ${r.backs} backs and ${r.faces} faces, want 2 face down`);
    if (r.box.left < r.trickRight || r.row.right - r.box.right > 2 || r.row.bottom - r.box.bottom > 2) fail(`${where}: the pressed stack is not in the bottom right corner (${JSON.stringify(r)})`);
    if (r.box.width < 44 || r.box.height < 44) fail(`${where}: the pressed stack is ${Math.round(r.box.width)}x${Math.round(r.box.height)}, below 44 px`);
  }
  if (original) await page.setViewportSize(original);
  if (shot) await shoot(page, shot);
  await page.click('[data-testid="pressed-play"]');
  await page.waitForSelector('[data-testid="info-card"] [data-testid="pressed-peek"]');
  const peeked = await page.locator('[data-testid="pressed-peek"] .info-pressed-card .card-svg').count();
  if (peeked !== 2) fail(`${label}: the look at the pressed skat shows ${peeked} cards face up`);
  if (shot) await shoot(page, `${shot}-peek`);
  await page.click('[data-testid="info-card"] [data-info-close]');
  await page.waitForSelector('[data-testid="info-card"]', { state: 'detached' });
  log(`${label}: pressed skat face down in the bottom right corner, a tap shows both face up`);
}

/**
 * The trick is a triangle pointing at the player (play-test feedback):
 * the left regular's place top left, the right one's top right, the
 * player's bottom centre, lower than both; no two places hide each
 * other's corner index. A played card lies exactly on its place.
 */
async function assertTriangle(page, label, sizes = [PHONE, { width: 360, height: 740 }, DESKTOP]) {
  const original = page.viewportSize();
  for (const size of sizes) {
    await page.setViewportSize(size);
    await page.waitForTimeout(120);
    const r = await page.evaluate(() => {
      const rect = (sel) => {
        const b = document.querySelector(sel)?.getBoundingClientRect();
        return b ? { x: b.left + b.width / 2, y: b.top + b.height / 2, left: b.left, right: b.right, top: b.top, bottom: b.bottom, w: b.width } : null;
      };
      const cards = [...document.querySelectorAll('.trick .trick-card')].map((c) => {
        const slot = c.closest('.slot')?.getBoundingClientRect();
        const b = c.getBoundingClientRect();
        return { seat: c.dataset.trickSeat ?? c.dataset.lastSeat, slot: c.closest('.slot')?.dataset.slot, dx: slot ? Math.abs(b.left - slot.left) + Math.abs(b.top - slot.top) : 99 };
      });
      return { s0: rect('.trick .slot-0'), s1: rect('.trick .slot-1'), s2: rect('.trick .slot-2'), cards };
    });
    const where = `${label} at ${size.width}x${size.height}`;
    const { s0, s1, s2 } = r;
    if (!s0 || !s1 || !s2) fail(`${where}: the trick lacks a place (${JSON.stringify(r)})`);
    if (!(s1.x < s0.x && s0.x < s2.x)) fail(`${where}: the places are not left, player, right (${Math.round(s1.x)}, ${Math.round(s0.x)}, ${Math.round(s2.x)})`);
    if (Math.abs(s0.x - (s1.x + s2.x) / 2) > 2) fail(`${where}: the player's place is not centred between the two others`);
    if (!(s0.y > s1.y + s0.w * 0.5 && Math.abs(s1.y - s2.y) < 1)) fail(`${where}: the player's place is not below the two others`);
    if (s0.left < s1.right - s0.w * 0.08 || s0.right > s2.left + s0.w * 0.08) fail(`${where}: the player's place covers a corner of another`);
    for (const c of r.cards) {
      if (c.seat !== c.slot) fail(`${where}: the card of seat ${c.seat} lies on place ${c.slot}`);
      if (c.dx > 1) fail(`${where}: the card of seat ${c.seat} lies ${c.dx}px off its place`);
    }
  }
  if (original) await page.setViewportSize(original);
  log(`${label}: the trick is a triangle (left, right, the player bottom centre), cards on their places`);
}

/**
 * Throw a hand card with the mouse: pressed, dragged up in steps, let go
 * with a flick of `flick` px per step (0: let go standing still) at `to`.
 */
async function throwCard(page, from, to, flick = 40) {
  const a = await page.locator(from).first().boundingBox();
  if (!a) fail(`nothing to throw: ${from}`);
  const x0 = a.x + a.width / 2;
  const y0 = a.y + a.height / 2;
  await page.mouse.move(x0, y0);
  await page.mouse.down();
  const steps = 8;
  for (let i = 1; i <= steps; i++) await page.mouse.move(x0 + ((to.x - x0) * i) / steps, y0 + ((to.y - y0) * i) / steps);
  if (flick) for (let i = 1; i <= 2; i++) await page.mouse.move(to.x + i * flick * 0.2, to.y - i * flick);
  // Standing still before letting go: no speed left.
  else await page.waitForTimeout(200);
  await page.mouse.up();
}

/**
 * "Letzter Stich": the button beside the trick shows the last finished
 * trick (three cards, the winner marked, his name above) until tapped
 * again; the page does not scroll with it, and the open trick comes back.
 */
async function assertLastTrick(page, label) {
  await page.waitForFunction(() => !document.querySelector('.run.is-busy'));
  if (!(await page.locator('[data-testid="last-trick-toggle"]').count())) fail(`${label}: no "Letzter Stich" button after a finished trick`);
  await page.click('[data-testid="last-trick-toggle"]');
  await page.waitForSelector('[data-testid="last-trick"]');
  const r = await page.evaluate(() => ({
    cards: document.querySelectorAll('[data-testid="last-trick"] .trick-card').length,
    winners: document.querySelectorAll('[data-testid="last-trick"] .trick-card.is-winner').length,
    caption: document.querySelector('[data-testid="last-trick-caption"]')?.textContent?.trim() ?? '',
    pressed: document.querySelector('[data-testid="last-trick-toggle"]')?.getAttribute('aria-pressed'),
    playable: document.querySelectorAll('[data-action="play"]').length,
  }));
  if (r.cards !== 3 || r.winners !== 1) fail(`${label}: last trick shows ${r.cards} cards, ${r.winners} marked as winner`);
  if (!/^Letzter Stich · an \S/.test(r.caption)) fail(`${label}: last trick caption "${r.caption}"`);
  if (r.pressed !== 'true') fail(`${label}: the button is not pressed while the last trick shows`);
  if (!r.playable) fail(`${label}: no card playable while the last trick shows`);
  await shoot(page, 'table-last-trick');
  await assertTriangle(page, `${label}, last trick`, [PHONE, DESKTOP]);
  await assertFits(page, `${label}, last trick shown`, [PHONE, { width: 375, height: 667 }, { width: 360, height: 740 }, DESKTOP]);
  // A tap on the trick puts the open one back.
  await page.click('[data-testid="last-trick"]');
  await page.waitForSelector('[data-testid="last-trick"]', { state: 'detached' });
  await page.waitForSelector('[data-testid="last-trick-toggle"][aria-pressed="false"]');
  if (await page.locator('[data-testid="last-trick-caption"]').count()) fail(`${label}: the caption stays after closing the last trick`);
  // Open again: playing a card closes it.
  await page.click('[data-testid="last-trick-toggle"]');
  await page.waitForSelector('[data-testid="last-trick"]');
  log(`${label}: last trick "${r.caption}" shown and closed`);
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
      await discardFirstTwo(page, label);
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

/**
 * Drücken with the old click strategy, so the seeds still win: the skat
 * cards lying on the table go back to the hand (a tap each), then the
 * first two cards of the hand the core lets go are tapped onto the table.
 * `atTwelve` runs while all twelve cards are in the hand.
 */
async function discardFirstTwo(page, label, atTwelve = null) {
  const onTable = page.locator('[data-testid="skat-zone"] [data-action="select"]');
  for (let i = 0; (await onTable.count()) && i < 3; i++) await onTable.first().click();
  if (await onTable.count()) fail(`${label}: the skat cards do not go back to the hand on a tap`);
  if (!(await page.locator('[data-action="discard"]').isDisabled())) fail(`${label}: Drücken possible with no card on the table`);
  if (atTwelve) await atTwelve();
  const hand = page.locator('.hand [data-action="select"]');
  await hand.first().click();
  await hand.first().click();
  const lying = await page.locator('[data-testid="skat-zone"] .skat-card').count();
  if (lying !== 2) fail(`${label}: ${lying} cards on the table after tapping two`);
  await page.click('[data-action="discard"]');
}

/**
 * Drag the element `from` onto `to` with the mouse, in small steps; with
 * `shot`, a screenshot (390 × 844) just before letting go.
 */
async function dragOnto(page, from, to, steps = 12, shot = null) {
  const a = await page.locator(from).first().boundingBox();
  const b = await page.locator(to).first().boundingBox();
  if (!a || !b) fail(`nothing to drag: ${from} -> ${to}`);
  await page.mouse.move(a.x + a.width / 2, a.y + a.height / 2);
  await page.mouse.down();
  for (let i = 1; i <= steps; i++) {
    await page.mouse.move(a.x + a.width / 2 + ((b.x + b.width / 2 - a.x - a.width / 2) * i) / steps, a.y + a.height / 2 + ((b.y + b.height / 2 - a.y - a.height / 2) * i) / steps);
  }
  if (shot) {
    await page.screenshot({ path: join(screens, `${shot}-390x844.png`) });
    log(`screenshot ${shot} (while dragging)`);
  }
  await page.mouse.up();
  await page.mouse.move(1, 1);
}

/** Einmischen with the tap fallback: tap the deck, then the button in the middle. */
async function mixIn(page) {
  await page.click('[data-testid="deck"]');
  await page.waitForSelector('[data-testid="deck"][aria-expanded="true"]');
  await page.click('[data-testid="mix-confirm"]');
}

/**
 * Skat oder Hand on the felt (play-test feedback): what the Stammgäste say
 * stands written out, the choices have short labels ("Einmischen" on the
 * deck, "Hand spielen") with their explanations on the info card (hover
 * here), the skat lies face down in the middle. A long press opens the
 * info card and marks no text; dragging across the table marks none either.
 */
async function assertChoice(page, seated) {
  const talk = (await page.locator('[data-testid="table-talk"] .talk-line').allTextContents()).map((l) => l.replace(/\s+/g, ' ').trim());
  const names = seated.opps.map((o) => o.name);
  if (talk.length !== 2 || !names.every((n, i) => talk[i]?.startsWith(`${n}:`))) fail(`table talk ${JSON.stringify(talk)} does not name ${names.join(', ')}`);
  if (seated.quip && !talk.some((l) => l.includes(seated.quip.text))) fail(`the greeting "${seated.quip.text}" is not on the felt: ${JSON.stringify(talk)}`);
  const hand = (await page.locator('[data-action="play-hand"]').textContent())?.trim();
  if (hand !== 'Hand spielen') fail(`the hand button reads "${hand}"`);
  const deck = await page.locator('[data-testid="deck"]').getAttribute('aria-label');
  if (!deck?.startsWith('Einmischen')) fail(`the deck is labelled "${deck}"`);
  if ((await page.locator('[data-testid="skat-pile"] .skat-spot.is-back').count()) !== 2) fail('the skat does not lie face down in the middle');
  for (const [sel, key] of [
    ['[data-action="play-hand"]', 'choice:hand'],
    ['[data-action="pick-up-skat"]', 'choice:skat'],
    ['[data-testid="deck"]', 'choice:pass'],
  ]) {
    await page.hover(sel);
    await page.waitForSelector(`[data-testid="info-card"][data-info-key="${key}"]`, { timeout: 3000 }).catch(() => fail(`no explanation on hovering ${sel}`));
  }
  await page.mouse.move(1, 1);
  await page.waitForSelector('[data-testid="info-card"]', { state: 'detached' });
  // Long press on a hand card, the mouse still down moving over the sheet.
  const card = await page.locator('.hand-card').first().boundingBox();
  await page.mouse.move(card.x + card.width / 2, card.y + card.height / 2);
  await page.mouse.down();
  await page.waitForTimeout(650);
  await page.mouse.move(20, 300, { steps: 6 });
  await page.mouse.up();
  await page.waitForSelector('.info-card.is-sheet');
  const marked = await page.evaluate(() => window.getSelection()?.toString() ?? '');
  if (marked) fail(`a long press marked text: "${marked.slice(0, 60)}"`);
  await page.keyboard.press('Escape');
  await page.waitForSelector('.info-card', { state: 'detached' });
  // Dragging across the talk and the opponents marks nothing.
  const line = await page.locator('[data-testid="table-talk"]').boundingBox();
  await page.mouse.move(line.x + 2, line.y + 4);
  await page.mouse.down();
  await page.mouse.move(line.x + line.width - 2, line.y - 60, { steps: 6 });
  await page.mouse.up();
  const dragged = await page.evaluate(() => window.getSelection()?.toString() ?? '');
  if (dragged) fail(`dragging across the table marked text: "${dragged.slice(0, 60)}"`);
  await page.mouse.move(1, 1);
  log(`Skat oder Hand: talk ${JSON.stringify(talk)}, explanations on hover, no text marked`);
}

/**
 * Drücken on the table by dragging: a hand card dropped onto a card on the
 * table takes its place (that one goes to the hand); a card dragged back
 * into the hand leaves one on the table, and Drücken waits for two.
 */
async function assertDiscardDrag(page, label) {
  const ids = () =>
    page.evaluate(() => ({
      table: [...document.querySelectorAll('[data-testid="skat-zone"] .skat-card')].map((e) => e.dataset.id),
      hand: [...document.querySelectorAll('.hand-card')].map((e) => e.dataset.id),
    }));
  const before = await ids();
  const handId = await page.locator('.hand [data-action="select"]').first().getAttribute('data-id');
  const target = before.table[0];
  await dragOnto(page, `.hand [data-id="${handId}"]`, `[data-testid="skat-zone"] .skat-card[data-id="${target}"]`, 12, 'table-discard-dragging');
  const swapped = await ids();
  if (!swapped.table.includes(handId) || swapped.table.includes(target) || !swapped.hand.includes(target)) fail(`${label}: dropping a hand card onto the table did not swap (${JSON.stringify(before)} -> ${JSON.stringify(swapped)})`);
  await shoot(page, 'table-discard-drag');
  await dragOnto(page, `[data-testid="skat-zone"] .skat-card[data-id="${handId}"]`, '.hand');
  const back = await ids();
  if (back.table.length !== 1 || !back.hand.includes(handId)) fail(`${label}: dragging a card back into the hand left ${JSON.stringify(back)}`);
  if (!(await page.locator('[data-action="discard"]').isDisabled())) fail(`${label}: Drücken possible with one card on the table`);
  log(`${label}: cards dragged between hand and table`);
}

/**
 * Pick up the skat: its two cards lie face up on the table (Drücken on the
 * table), the hand holds the other ten; press the first two cards the core
 * lets go (discardFirstTwo).
 */
async function pickUpAndDiscard(page, label, shot = null) {
  await page.waitForSelector('[data-testid="skat-choice"]');
  await page.click('[data-action="pick-up-skat"]');
  await page.waitForSelector('[data-testid="skat-zone"] [data-action="select"]');
  const onTable = await page.locator('[data-testid="skat-zone"] .skat-card').count();
  const handSize = await page.locator('.hand-card').count();
  if (onTable + handSize !== 12) fail(`${label}: expected 12 cards while discarding, saw ${handSize} in the hand and ${onTable} on the table`);
  if (onTable !== 2) fail(`${label}: the skat should lie on the table after picking it up, ${onTable} cards there`);
  if (await page.locator('[data-action="discard"]').isDisabled()) fail(`${label}: Drücken not possible with the skat on the table`);
  if (await page.locator('.declare').count()) fail(`${label}: an Ansage while discarding`);
  if (shot) {
    await shoot(page, shot);
    await assertFits(page, `${label}, discarding (skat on the table)`);
    await assertDiscardDrag(page, label);
  }
  await discardFirstTwo(page, label, shot ? () => assertFits(page, `${label}, discarding (12 cards in the hand)`) : null);
  await page.waitForSelector('.declare[data-phase="DeclaringAfterSkat"]');
  const pressed = await page.locator('[data-testid="pressed"] .mini-card').count();
  if (pressed !== 2) fail(`${label}: the Ansage after the skat shows ${pressed} pressed cards`);
}

/**
 * Skat aufnehmen, drücken, then declare a plain Grand. After the skat the
 * Ansage offers Normal and, since ADR 0013, Schneider, Schwarz and Ouvert,
 * but never Hand.
 */
async function skatThenGrand(page, label, shot = null) {
  await pickUpAndDiscard(page, label, shot);
  const risks = await page.locator('[data-action="pick-risk"]').evaluateAll((els) => els.map((e) => e.getAttribute('data-risk')).join(','));
  if (risks !== 'Normal,SchneiderDeclared,SchwarzDeclared,Ouvert') fail(`${label}: the Ansage after the skat offers ${risks}`);
  const picked = await page.locator('[data-action="pick-risk"][aria-pressed="true"]').getAttribute('data-risk');
  if (picked !== 'Normal') fail(`${label}: after the skat ${picked} is picked, not Normal`);
  if (shot) {
    await shoot(page, `${shot}-ansage`);
    await assertFits(page, `${label}, Ansage after the skat`);
  }
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

/**
 * "Kartenfarbe tönen": off by default (plain paper); the menu switch tints
 * each card's paper in its suit, is stored and survives a reload. Every
 * corner index uses its suit's dark ink, not black.
 */
async function suitTintPass(browser, errors, reducedMotion) {
  const context = await quietContext(browser, { viewport: PHONE, reducedMotion });
  const page = await context.newPage();
  watch(page, errors);
  const papers = () =>
    page.$$eval('.hand .card-svg[data-suit] rect.paper', (rects) =>
      rects.map((r) => ({ suit: r.closest('.card-svg').dataset.suit, fill: getComputedStyle(r).fill })),
    );
  const bySuit = (list) => {
    const out = {};
    for (const { suit, fill } of list) (out[suit] ??= new Set()).add(fill);
    return out;
  };
  await page.goto(base);
  await page.fill('#seed', SEED);
  await page.click('[data-action="new-run"]');
  await page.waitForSelector('[data-testid="skat-choice"]');

  // Each rank letter has the dark ink of its suit (F3 colours, at least 4.5:1 on plain and tinted paper).
  for (const [id, ink] of [['x-C7', '#24507f'], ['x-S7', '#1f7a30'], ['x-H7', '#c0221a'], ['x-D7', '#8a5f00']]) {
    const inks = await page.$$eval(`#${id} path`, (paths) => paths.map((p) => p.getAttribute('fill')));
    if (!inks.includes(ink)) fail(`index ${id} inks ${inks.join(', ')}: expected ${ink} for the letter`);
  }

  const plain = await papers();
  const suits = Object.keys(bySuit(plain));
  if (suits.length < 4) fail(`the hand of seed ${SEED} should hold all four suits, has ${suits.join(', ')}`);
  const plainFills = new Set(plain.map((p) => p.fill));
  if (plainFills.size !== 1) fail(`with the tint off the papers differ: ${[...plainFills].join(', ')}`);

  await page.click('[data-action="menu"]');
  await page.waitForSelector('[data-testid="setting-suit-tint"]');
  await page.click('[data-testid="setting-suit-tint"] [data-on="true"]');
  await page.waitForSelector('[data-testid="setting-suit-tint"] [data-on="true"][aria-pressed="true"]');
  await assertMenuNoScroll(page, 'menu with "Kartenfarbe tönen"');
  const stored = await page.evaluate(() => JSON.parse(localStorage.getItem('stammtisch.settings') ?? '{}'));
  if (stored.suitTint !== true) fail(`"Kartenfarbe tönen" not stored: ${JSON.stringify(stored)}`);
  await page.click('[data-testid="menu-close"]');

  const check = async (where) => {
    const tinted = bySuit(await papers());
    const per = Object.entries(tinted).map(([suit, fills]) => {
      if (fills.size !== 1) fail(`${where}: the ${suit} cards have different papers`);
      return [...fills][0];
    });
    if (new Set(per).size !== 4) fail(`${where}: the four suits do not have four papers: ${per.join(', ')}`);
    if (per.some((f) => plainFills.has(f))) fail(`${where}: a suit kept the plain paper: ${per.join(', ')}`);
  };
  await check('tint on');
  await page.reload();
  await page.click('[data-action="resume"]');
  await page.waitForSelector('[data-testid="skat-choice"]');
  await check('tint on after a reload');

  await page.click('[data-action="menu"]');
  await page.click('[data-testid="setting-suit-tint"] [data-on="false"]');
  await page.click('[data-testid="menu-close"]');
  const off = new Set((await papers()).map((p) => p.fill));
  if (off.size !== 1 || !plainFills.has([...off][0])) fail(`tint off again: papers ${[...off].join(', ')}`);
  log('"Kartenfarbe tönen": off by default, tints the paper per suit, stored, off again');
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
  await assertNoSideScroll(page, 'Tresen before the boss');
  await expectMusic(page, 'shop', 'Tresen before the boss');
  await page.click('[data-action="leave-shop"]');
  await page.waitForSelector('[data-action="boss-go"]');
  await expectMusic(page, 'boss', 'boss dialog');
  const bossText = (await page.locator('.sheet-boss').textContent()) ?? '';
  if (!bossText.includes('Kontra-Hans')) fail(`boss dialog without the boss: ${bossText}`);
  const dialogBack = await page.locator('[data-testid="boss-back"]').getAttribute('data-back');
  if (dialogBack !== 'kontra_hans') fail(`the boss dialog shows the back ${dialogBack}`);
  log('boss dialog shown with his back');
  await shoot(page, 'boss');
  await page.click('[data-action="boss-go"]');
  await page.waitForSelector('.declare');
  if (!(await page.locator('.opp.is-boss').count())) fail('the boss does not sit at the table');
  if (!(await page.locator('.hud [data-testid="hud-boss"]').count())) fail('the head shows no boss chip at the boss table');
  await expectBacks(page, 'Kontra-Hans series', 'kontra_hans', 'regulars');
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

/**
 * The backs on the opponents' fans (ADR 0015): `left` and `right` are the
 * back ids of seat 1 and 2 (the boss sits left), in their small picture.
 */
async function expectBacks(page, label, left, right) {
  const seen = await page.evaluate(() =>
    [1, 2].map((s) => document.querySelector(`.opp[data-seat="${s}"] .opp-hand use`)?.getAttribute('href') ?? null),
  );
  const want = [`#bk-${left}-s`, `#bk-${right}-s`];
  if (seen[0] !== want[0] || seen[1] !== want[1]) fail(`${label}: opponent backs ${JSON.stringify(seen)}, expected ${JSON.stringify(want)}`);
  log(`${label}: backs ${seen.join(', ')}`);
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
  await assertNoSideScroll(page, 'encounter', '.encounter');
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

  // Der Alte lays another boss's rule on every game.
  await resumeFixture(page, 'old-man.json');
  await page.waitForSelector('[data-testid="boss-rule"]');
  const rule = ((await page.locator('[data-testid="boss-rule"]').textContent()) ?? '').replace(/\s+/g, ' ').trim();
  if (!rule.includes('Der Alte')) fail(`the final boss's rule strip does not name him: ${rule}`);
  log(`Der Alte: ${rule}`);
  // His own back, not the one of the boss whose rule is on.
  await expectBacks(page, 'Der Alte', 'the_old_man', 'regulars');
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

  // Effekte v2: "Ramponiert" means −X Augen now, the old "darf nicht
  // gedrückt werden" and "Unter ehrenhalber" are gone (DESIGN 7.2); their
  // fixtures stay as old saves (save version 1).
  // M9 (ADR 0015): a run saved under the old rules (shared deck) is
  // discarded once with a note; a new run can start.
  await page.goto(base);
  await page.waitForSelector('[data-action="new-run"]');
  await page.evaluate((s) => localStorage.setItem('stammtisch.run', s), fixture('discard-battered.json'));
  await page.reload();
  await page.waitForSelector('[data-action="new-run"]');
  const note = (await page.locator('.is-warning[role="alert"]').first().textContent().catch(() => '')) ?? '';
  if (!note.includes('Neue Regeln')) fail(`an old run is not discarded with a note: "${note}"`);
  if (await page.locator('[data-action="resume"]').count()) fail('an old run is still offered for resuming');
  if (await page.evaluate(() => localStorage.getItem('stammtisch.run'))) fail('the old run is still saved');
  log(`old run discarded: ${note.trim()}`);

  // M8 wave 2a: the new relics at the table and in the Abrechnung.
  await resumeFixture(page, 'wave2-relics.json');
  await page.waitForSelector('.declare');
  // M9 (ADR 0015): the Stammgäste's cards show their back, the player's
  // stay the Distel.
  await expectBacks(page, 'Stammgäste', 'regulars', 'regulars');
  // "Altes Blatt" was removed on 02.11.2026 and drops from the old save.
  for (const id of ['home_suit', 'all_rounder', 'skat_keeper', 'watertight']) {
    if (!(await page.locator(`[data-info="relic:${id}"] svg[data-deckel="${id}"]`).count())) fail(`relic ${id} has no beer mat in the head`);
  }
  await pickUpAndDiscard(page, 'wave 2a');
  await page.click('[data-action="pick-game"][data-index="0"]');
  await page.click('[data-action="declare"]');
  await page.waitForSelector('.declare', { state: 'detached' });
  const worth = await text(page, 'value-preview');
  if (!worth.includes('× 1,5')) fail(`Stammfarbe: the score line says "${worth}"`);
  await shoot(page, 'wave2-table');
  await playOut(page, 'wave 2a');
  const board = ((await page.locator('.score-sheet').textContent()) ?? '').replace(/\s+/g, ' ');
  for (const want of ['Stammfarbe', 'Spielwert ×1,5', 'Allrounder']) {
    if (!board.includes(want)) fail(`wave 2a: the Abrechnung does not say "${want}"`);
  }
  const mul = ((await page.locator('[data-num="mul"]').textContent()) ?? '').trim();
  if (mul !== '1,5') fail(`wave 2a: the factor box shows "${mul}"`);
  await shoot(page, 'wave2-score');
  log(`wave 2a: score line ${worth}, Abrechnung with Stammfarbe ×1,5`);
  await context.close();
}

/**
 * Start a run with `seed` and give it `relics` (after the character's start
 * relics) from the first game on: the
 * save in localStorage is patched as text (JSON.parse would round the big
 * rng numbers), `rules` are extra replacements in the dealt game's rules.
 */
async function patchedRun(page, seed, relics, rules = []) {
  await page.goto(base);
  await page.waitForSelector('[data-action="new-run"]');
  // A run left from before would be offered for resuming instead.
  await page.evaluate(() => localStorage.removeItem('stammtisch.run'));
  await page.reload();
  await page.waitForSelector('[data-action="new-run"]');
  await page.fill('#seed', seed);
  await page.click('[data-action="new-run"]');
  await page.waitForSelector('[data-testid="skat-choice"]');
  const ok = await page.evaluate(
    ({ relics, rules }) => {
      let s = localStorage.getItem('stammtisch.run') ?? '';
      // The owned relics (the character's start relics first) get the new ones.
      const owned = /"relics":\[([^\]]*)\]/;
      if (!owned.test(s)) return false;
      s = s.replace(owned, (_, inner) => `"relics":[${[inner, ...relics.map((r) => JSON.stringify(r))].filter(Boolean).join(',')}]`);
      for (const [from, to] of rules) {
        if (!s.includes(from)) return false;
        s = s.replace(from, to);
      }
      localStorage.setItem('stammtisch.run', s);
      return true;
    },
    { relics, rules },
  );
  if (!ok) fail(`could not patch the save of seed ${seed} with ${relics.join(', ')}`);
  await page.reload();
  await page.click('[data-action="resume"]');
  await page.waitForSelector('[data-testid="skat-choice"]');
  for (const id of relics) {
    if (!(await page.locator(`[data-info="relic:${id}"] svg[data-deckel="${id}"]`).count())) fail(`relic ${id} has no beer mat in the head`);
  }
}

/** M8 wave 2b: Drei drücken and Spickzettel at the table. */
async function wave2b(browser, errors, reducedMotion) {
  const context = await quietContext(browser, { viewport: PHONE, reducedMotion });
  const page = await context.newPage();
  watch(page, errors);

  // Drei drücken.
  await patchedRun(page, SEED, ['three_discard'], [['"discard_three":null', '"discard_three":{"Relic":"three_discard"}']]);
  const note = (await page.locator('[data-testid="once-three_discard"]').textContent())?.trim();
  if (note !== '1. Spiel') fail(`Drei drücken: the chip says "${note}"`);
  if (await page.locator('.coaster-chip.is-used').count()) fail('Drei drücken: marked used before the skat');
  await shoot(page, 'wave2b-chip');
  await page.click('[data-action="pick-up-skat"]');
  await page.waitForSelector('[data-testid="skat-zone"] [data-action="select"]');
  const zone = '[data-testid="skat-zone"]';
  const onTable = await page.locator(`${zone} .skat-card`).count();
  const inHand = await page.locator('.hand-card').count();
  if (onTable !== 3 || inHand !== 10) fail(`Drei drücken: ${onTable} cards on the table, ${inHand} in the hand`);
  const drawnTag = ((await page.locator(`${zone} .skat-tag.is-drawn`).textContent()) ?? '').trim();
  if (!drawnTag.startsWith('von ')) fail(`Drei drücken: the drawn card is tagged "${drawnTag}"`);
  const label = ((await page.locator(`${zone} .skat-zone-label`).textContent()) ?? '').trim();
  if (label !== 'Drei drücken (1. Spiel) · 3 von 3') fail(`Drei drücken: the table says "${label}"`);
  // Its toast stands above the Stammgäste, never over the cards on the table.
  const toast = await page
    .waitForSelector('.toast-layer.is-top .toast', { timeout: 3000 })
    .then((el) => el.boundingBox())
    .catch(() => null);
  const table = await page.locator(zone).boundingBox();
  if (!toast) fail('Drei drücken: no toast above the Stammgäste');
  if (toast.y + toast.height > table.y) fail(`Drei drücken: the toast (bottom ${Math.round(toast.y + toast.height)}) covers the table zone (top ${Math.round(table.y)})`);
  if (!(await page.locator('.coaster-chip.is-used [data-testid="once-three_discard"]').count())) fail('Drei drücken: the chip does not show "1. Spiel" as used');
  if (await page.locator('[data-action="discard"]').isDisabled()) fail('Drei drücken: Drücken not possible with three on the table');
  await shoot(page, 'wave2b-three-discard');
  await assertFits(page, 'Drei drücken', [PHONE, { width: 375, height: 667 }, DESKTOP]);
  // One back into the hand: Drücken waits; another one out: three again.
  await page.locator(`${zone} [data-action="select"]`).first().click();
  if (!(await page.locator('[data-action="discard"]').isDisabled())) fail('Drei drücken: Drücken possible with two on the table');
  await page.locator('.hand [data-action="select"]').first().click();
  if ((await page.locator(`${zone} .skat-card`).count()) !== 3) fail('Drei drücken: a tapped hand card did not go onto the table');
  await page.click('[data-action="discard"]');
  await page.waitForSelector('.declare[data-phase="DeclaringAfterSkat"]');
  const pressed = await page.locator('[data-testid="pressed"] .mini-card').count();
  if (pressed !== 2) fail(`Drei drücken: the Ansage shows ${pressed} pressed cards`);
  if ((await page.locator('.opp[data-seat="1"] [data-returned]').count()) !== 1) fail('Drei drücken: the card given back is not shown at the left Stammgast');
  if ((await page.locator('.hand-card').count()) !== 10) fail('Drei drücken: not ten cards after Drücken');
  await shoot(page, 'wave2b-three-discard-ansage');
  log('Drei drücken: three on the table, two pressed, one back to the left Stammgast');

  await page.click('[data-action="pick-game"][data-index="4"]');
  await page.click('[data-action="declare"]');
  await page.waitForSelector('.declare', { state: 'detached' });
  await playOut(page, 'wave 2b');

  // Spickzettel.
  await patchedRun(page, '65', ['crib_sheet']);
  await page.click('[data-action="play-hand"]');
  await page.waitForSelector('.declare-hand');
  await page.click('[data-action="pick-game"][data-index="4"]');
  await page.click('[data-action="declare"]');
  await page.waitForSelector('[data-testid="crib-skat"]', { timeout: 30000 });
  const open = await page.locator('[data-testid="crib-skat"] button.pressed-card').count();
  const hidden = await page.locator('[data-testid="crib-skat"] .pressed-card.is-back').count();
  if (open !== 2 || hidden !== 0) fail(`Spickzettel: ${open} skat card(s) open, ${hidden} face down`);
  await shoot(page, 'wave2b-crib');
  await assertFits(page, 'Spickzettel', [PHONE, { width: 375, height: 667 }]);
  log('Spickzettel: both skat cards face up beside the trick');
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
  await mixIn(page);
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
  for (const must of ['Skat oder Hand', 'Drücken', 'Grundwert × Multiplikator', 'Schellen', 'Eichel', 'Grand', 'Null', 'Kein Reizen', 'Boss', 'Tresen']) {
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

  // A pass (Einmischen: the deck into the middle, here with the tap fallback).
  await mixIn(page);
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
  // Kneipentour: the route choice comes first, with its own hint.
  log(`hint route: ${await expectHint(page, 'route')}`);
  await shoot(page, 'hint-route');
  await dismissHint(page, 'route');
  await takeRoute(page, 'Tresen');
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

/** The link of the running run, from the menu (clipboard or the field to copy by hand). */
async function menuLink(page) {
  await page.click('.hud [data-action="menu"]');
  await page.waitForSelector('.sheet [data-action="copy-link"]');
  await page.click('.sheet [data-action="copy-link"]');
  await page.waitForSelector('[data-testid="share-copied"], [data-testid="share-fallback"]');
  const link = (await page.locator('[data-testid="share-copied"]').count())
    ? await page.evaluate(() => navigator.clipboard.readText())
    : await page.locator('[data-testid="share-fallback"]').inputValue();
  await closeSheet(page);
  return link;
}

/** The "Abschnitt x/y" line of the HUD. */
async function hudSection(page) {
  return ((await page.locator('.hud .hud-where-sub').first().textContent()) ?? '').replace(/\s+/g, ' ').trim();
}

/**
 * The short evening is dropped (ADR 0016): no length choice on the start
 * screen, the start button fits; an old `-kurz` link is offered and played
 * as the whole evening (three districts in the HUD).
 */
async function shortEvening(browser, errors, reducedMotion) {
  const context = await quietContext(browser, { viewport: PHONE, reducedMotion });
  const page = await context.newPage();
  watch(page, errors);
  await page.goto(base);
  await page.waitForSelector('[data-action="new-run"]');
  if (await page.locator('[data-testid="run-length"]').count()) fail('the start screen still offers a run length');
  for (const size of [PHONE, { width: 375, height: 667 }]) {
    await page.setViewportSize(size);
    await page.waitForTimeout(100);
    const r = await page.evaluate(() => ({
      inner: window.innerHeight,
      start: document.querySelector('[data-action="new-run"]')?.getBoundingClientRect().bottom ?? Infinity,
    }));
    if (r.start > r.inner) fail(`start screen at ${size.width}x${size.height}: the start button ends at ${Math.round(r.start)}, below ${r.inner}`);
  }
  await page.setViewportSize(PHONE);
  await context.close();
  const other = await quietContext(browser, { viewport: PHONE, reducedMotion });
  const page2 = await other.newPage();
  watch(page2, errors);
  await page2.goto(base);
  await page2.evaluate(() => localStorage.setItem('stammtisch.length', 'Short'));
  await page2.goto(`${base}#seed-4343-alleinspieler-kurz`);
  await page2.waitForSelector('[data-testid="shared-seed"][data-seed="4343"]');
  const offer = ((await page2.locator('[data-testid="shared-seed"]').textContent()) ?? '').replace(/\s+/g, ' ');
  if (!offer.includes('ganzen Abend')) fail(`the offer of an old short link does not say it plays the whole evening: ${offer}`);
  await page2.click('[data-action="play-shared"]');
  await page2.waitForSelector('.declare');
  const hud = await hudSection(page2);
  if (!hud.includes('Stadtteil 1/3')) fail(`HUD of an old short link reads "${hud}"`);
  const saved = await page2.evaluate(() => JSON.parse(localStorage.getItem('stammtisch.run') ?? '{}').run?.length);
  if (saved !== 'Full') fail(`saved run has length ${saved}`);
  log(`short evening dropped: no length choice, an old short link plays the whole evening ("${hud}")`);
  await other.close();
}

/**
 * The table in motion (play-test feedback), with animations on whatever
 * the other passes use: every deal flies in from the deck with its own
 * spins (cosmetic randomness: a second deal differs), the skat turns over
 * where it lies after picking it up, and a finished trick slides off
 * towards the seat that took it (left regular: to the left, right
 * regular: to the right, the player: down).
 */
async function motionPass(browser, errors) {
  const context = await quietContext(browser, { viewport: PHONE, reducedMotion: 'no-preference' });
  const page = await context.newPage();
  watch(page, errors);
  await page.goto(base);
  await page.waitForSelector('[data-action="new-run"]');
  await page.fill('#seed', SEED);
  await page.click('[data-action="new-run"]');
  await page.waitForSelector('[data-testid="skat-choice"]');
  const spins = () =>
    page.evaluate(() =>
      [...document.querySelectorAll('.hand-card')].map((el) => {
        const a = el.getAnimations()[0];
        return a ? String(a.effect.getKeyframes()[0].transform) : null;
      }),
    );
  const first = await spins();
  if (first.filter(Boolean).length < 10) fail(`motion: only ${first.filter(Boolean).length} hand cards fly in when dealt`);
  if (new Set(first).size < 5) fail('motion: the cards of a deal all fly in the same way');
  await page.waitForTimeout(1500);
  await page.mouse.move(1, 1);
  await mixIn(page);
  await page.waitForFunction(() => document.querySelectorAll('.hand-card').length === 10 && document.querySelector('.hand-card')?.getAnimations().length);
  const second = await spins();
  if (second.every((t, i) => t === first[i])) fail('motion: the second deal flies in exactly like the first');
  log('motion: deals fly in from the deck, each a little different');
  await page.waitForTimeout(2000);
  await page.click('[data-action="pick-up-skat"]');
  await page
    .waitForFunction(() => [...document.querySelectorAll('[data-testid="skat-zone"] .skat-card')].filter((el) => el.querySelector('.flip-cover') && el.getAnimations().length).length === 2, null, { timeout: 5000 })
    .catch(() => fail('motion: the skat does not turn over on the table'));
  log('motion: the skat turns over on the table');
  await page.waitForSelector('[data-testid="skat-zone"] [data-action="select"]');
  await discardFirstTwo(page, 'motion');
  await page.waitForSelector('.declare[data-phase="DeclaringAfterSkat"]');
  await page.click('[data-action="pick-game"][data-index="4"]');
  await page.click('[data-action="declare"]');
  const names = await page.evaluate(() => Object.fromEntries([...document.querySelectorAll('.opp')].map((o) => [o.dataset.seat, o.querySelector('[data-testid="opp-name"]')?.textContent?.trim()])));
  const seen = {};
  for (let n = 0; n < 10 && Object.keys(seen).length < 3; n++) {
    await page.waitForSelector('[data-action="play"]', { timeout: 30000 });
    await page.locator('[data-action="play"]').first().click();
    await page.mouse.move(1, 1);
    const sweep = await page
      .waitForFunction(
        () => {
          const a = document.getAnimations().find((x) => x.effect?.target?.classList?.contains('trick-card') && x.effect.getKeyframes().length === 3);
          if (!a) return null;
          const m = /translate\((-?[\d.]+)px, (-?[\d.]+)px\)/.exec(String(a.effect.getKeyframes()[2].transform));
          return m ? { x: Number(m[1]), y: Number(m[2]), status: document.querySelector('.status')?.textContent?.trim() ?? '' } : null;
        },
        null,
        { timeout: 20000, polling: 16 },
      )
      .then((h) => h.jsonValue());
    const winner = sweep.status.startsWith('Dein') ? 'you' : sweep.status.includes(names[1]) ? 'left' : sweep.status.includes(names[2]) ? 'right' : '?';
    const ok = winner === 'left' ? sweep.x < -60 : winner === 'right' ? sweep.x > 60 : winner === 'you' ? sweep.y > 60 && Math.abs(sweep.x) < sweep.y : false;
    if (!ok) fail(`motion: trick to ${winner} (${sweep.status}) leaves by ${Math.round(sweep.x)}, ${Math.round(sweep.y)}`);
    if (!seen[winner]) {
      // Half way off the table.
      await page.waitForFunction(() => {
        const a = document.getAnimations().find((x) => x.effect?.target?.classList?.contains('trick-card') && x.effect.getKeyframes().length === 3);
        return !a || a.currentTime > a.effect.getTiming().duration * 0.7;
      }, null, { polling: 16 });
      await page.screenshot({ path: join(screens, `trick-leaves-${winner}-390x844.png`) });
    }
    seen[winner] = true;
  }
  if (Object.keys(seen).length < 2) fail(`motion: tricks left the table only towards ${Object.keys(seen).join(', ')}`);
  log(`motion: tricks slide off towards ${Object.keys(seen).join(', ')}`);
  await context.close();
}

/**
 * The feel of the table with full motion (play-test feedback), at one
 * viewport: cards are thrown onto the felt with a turn, come down askew
 * and off their place, lie there a moment and slide onto it; a hand card
 * can be thrown with the mouse (the stronger the flick, the further it
 * flies), one the core does not let go springs back and shakes, one let
 * go over the hand without a flick goes back without being played.
 * Screenshots: the throw in flight, an opponent's card in flight, the
 * trick with all three cards, the pressed skat and its look.
 */
async function tableFeelPass(browser, errors, viewport) {
  const context = await browser.newContext({ viewport, reducedMotion: 'no-preference' });
  // Tempo "Gemütlich": time for the screenshots while the trick lies there.
  await context.addInitScript(() => {
    if (!localStorage.getItem('stammtisch.settings')) localStorage.setItem('stammtisch.settings', JSON.stringify({ tempo: 'slow', sound: false, thinking: true, hints: false }));
  });
  const page = await context.newPage();
  watch(page, errors);
  const tag = `${viewport.width}x${viewport.height}`;
  const snap = (name) => page.screenshot({ path: join(screens, `${name}-${tag}.png`) });
  await page.goto(base);
  await page.waitForSelector('[data-action="new-run"]');
  await page.fill('#seed', SEED);
  await page.click('[data-action="new-run"]');
  await page.waitForSelector('[data-action="pick-up-skat"]');
  await page.waitForTimeout(1200);
  await page.click('[data-action="pick-up-skat"]');
  await page.waitForSelector('[data-testid="skat-zone"] [data-action="select"]');
  await page.waitForTimeout(700);
  await page.click('[data-action="discard"]');
  await page.waitForSelector('.declare[data-phase="DeclaringAfterSkat"]');
  await page.click('[data-action="pick-game"][data-index="4"]');
  await page.click('[data-action="declare"]');
  await page.waitForSelector('[data-action="play"]', { timeout: 30000 });
  await page.waitForFunction(() => !document.getAnimations().some((a) => a.playState === 'running' && a.effect?.target?.closest?.('.trick, .hand')));

  // The pressed skat: face down in the corner, a tap shows it.
  await snap('feel-pressed');
  await page.click('[data-testid="pressed-play"]');
  await page.waitForSelector('[data-testid="pressed-peek"]');
  await page.waitForTimeout(300);
  await snap('feel-pressed-peek');
  await page.click('[data-testid="info-card"] [data-info-close]');
  await page.waitForSelector('[data-testid="info-card"]', { state: 'detached' });

  // Thrown with a strong flick from the hand.
  const id = await page.locator('[data-action="play"]').first().getAttribute('data-id');
  const felt = await page.locator('[data-testid="trick"]').boundingBox();
  await throwCard(page, `.hand [data-action="play"][data-id="${id}"]`, { x: felt.x + felt.width / 2, y: felt.y + felt.height }, 60);
  const flight = await page
    .waitForFunction(
      (cardId) => {
        const el = document.querySelector(`.trick-card[data-card-id="${cardId}"]`);
        const a = el?.getAnimations().find((x) => x.id === 'throw');
        if (!a) return null;
        const k = a.effect.getKeyframes();
        return { frames: k.map((f) => String(f.transform)), offsets: k.map((f) => f.computedOffset), slot: el.closest('.slot')?.dataset.slot, time: a.currentTime, total: a.effect.getTiming().duration };
      },
      id,
      { timeout: 10000, polling: 16 },
    )
    .then((h) => h.jsonValue())
    .catch(() => fail(`feel ${tag}: the card thrown does not fly onto the felt`));
  await snap('feel-throw-flight');
  if (flight.slot !== '0') fail(`feel ${tag}: the player's card lies on place ${flight.slot}`);
  // translate(x, y) rotate(r) scale(s) of a keyframe.
  const parts = (t) => (/translate\((-?[\d.]+)px, (-?[\d.]+)px\) rotate\((-?[\d.]+)deg\) scale\(([\d.]+)\)/.exec(t) ?? []).slice(1).map(Number);
  const [start, land, , end] = flight.frames.map(parts);
  if (!start?.length || !land?.length || !end?.length) fail(`feel ${tag}: unexpected keyframes ${JSON.stringify(flight.frames)}`);
  if (flight.frames[1] !== flight.frames[2]) fail(`feel ${tag}: the card does not lie still a moment before it slides onto its place (${flight.frames[1]} / ${flight.frames[2]})`);
  if (end[0] !== 0 || end[1] !== 0 || end[2] !== 0 || end[3] !== 1) fail(`feel ${tag}: the card does not end exactly on its place: ${flight.frames[3]}`);
  if (Math.abs(land[2]) < 4) fail(`feel ${tag}: the card comes down straight, not askew: ${flight.frames[1]}`);
  if (Math.abs(start[2] - land[2]) < 30) fail(`feel ${tag}: the card does not turn in flight (${flight.frames[0]} -> ${flight.frames[1]})`);
  if (Math.hypot(land[0], land[1]) < 3) fail(`feel ${tag}: the card comes down right on its place, not off it`);
  const holdMs = (flight.offsets[2] - flight.offsets[1]) * flight.total;
  if (holdMs < 300 * 0.9 || holdMs > 500 * 1.5 * 1.1) fail(`feel ${tag}: the card lies askew for ${Math.round(holdMs)} ms`);
  log(`feel ${tag}: thrown card flies (${Math.round(flight.total)} ms), lies askew ${Math.round(holdMs)} ms, slides onto the player's place`);
  // An opponent's card in flight.
  const opp = await page
    .waitForFunction(() => [...document.querySelectorAll('.trick-card')].find((el) => el.dataset.trickSeat !== '0' && el.getAnimations().some((a) => a.id === 'throw' && a.currentTime < a.effect.getTiming().duration * 0.3))?.dataset.trickSeat ?? null, null, { timeout: 20000, polling: 16 })
    .then((h) => h.jsonValue())
    .catch(() => null);
  if (opp) await snap('feel-opponent-flight');
  // All three cards on the felt, then lying on their places.
  await page.waitForFunction(() => document.querySelectorAll('[data-testid="trick"] .trick-card').length === 3, null, { timeout: 20000, polling: 16 });
  await page.waitForFunction(() => ![...document.querySelectorAll('[data-testid="trick"] .trick-card')].some((el) => el.getAnimations({ subtree: true }).length), null, { polling: 16 });
  await page.mouse.move(1, 1);
  await snap('feel-trick-three');
  await assertTriangle(page, `feel ${tag}, three cards`, [viewport]);
  // Faster from here on.
  await page.waitForSelector('[data-action="play"]', { timeout: 30000 });
  await page.click('[data-action="menu"]');
  await page.click('[data-action="tempo"][data-tempo="fast"]');
  await page.click('.sheet [data-action="close-overlay"]');

  // Let go over the hand, standing still: back into the hand, not played.
  await page.waitForSelector('[data-action="play"]', { timeout: 30000 });
  const keep = await page.locator('[data-action="play"]').first().getAttribute('data-id');
  const kb = await page.locator(`.hand [data-id="${keep}"]`).boundingBox();
  await throwCard(page, `.hand [data-id="${keep}"]`, { x: kb.x + kb.width / 2 + 30, y: kb.y + kb.height / 2 - 20 }, 0);
  await page.waitForTimeout(400);
  if (!(await page.locator(`.hand [data-id="${keep}"]`).count()) || (await page.locator('.run.is-busy').count())) fail(`feel ${tag}: a card let go over the hand was played`);
  log(`feel ${tag}: a card let go over the hand goes back`);

  // A card the core does not let go: thrown, it springs back and shakes.
  let dim = null;
  for (let n = 0; n < 9 && !dim; n++) {
    await page.waitForSelector('[data-action="play"], .score-sheet', { timeout: 30000 });
    if (await page.locator('.score-sheet').count()) break;
    if (await page.locator('.hand-card.is-dim').count()) dim = await page.locator('.hand-card.is-dim').first().getAttribute('data-id');
    else await page.locator('[data-action="play"]').first().click();
    await page.mouse.move(1, 1);
  }
  if (dim) {
    const t = await page.locator('[data-testid="trick"]').boundingBox();
    await throwCard(page, `.hand [data-id="${dim}"]`, { x: t.x + t.width / 2, y: t.y + t.height / 2 }, 40);
    const shook = await page
      .waitForFunction((cardId) => document.querySelector(`.hand-card[data-id="${cardId}"]`)?.getAnimations().some((a) => a.effect.composite === 'add'), dim, { timeout: 3000, polling: 16 })
      .then(() => true)
      .catch(() => false);
    if (!shook) fail(`feel ${tag}: a card that may not be played does not shake when thrown`);
    if (!(await page.locator(`.hand [data-id="${dim}"]`).count())) fail(`feel ${tag}: a card that may not be played left the hand`);
    await page.waitForSelector('.toast', { timeout: 3000 }).catch(() => fail(`feel ${tag}: no note for a card that may not be played`));
    log(`feel ${tag}: a card the core does not let go springs back and shakes`);
  } else {
    log(`feel ${tag}: no card to follow suit with in this game, spring back not checked`);
  }
  await context.close();
}

/** Effekte v3: every modification on the Kreidetafel, the gallery of frames.json. */
async function framesPass(browser, errors, reducedMotion) {
  const context = await quietContext(browser, { viewport: PHONE, reducedMotion });
  const page = await context.newPage();
  watch(page, errors);
  await resumeFixture(page, 'frames.json');
  await page.waitForSelector('[data-testid="shop"]');
  await page.click('.hud [data-action="overview"]');
  await page.waitForSelector('.deck-card .card-svg');
  const cards = await page.locator('.deck-card .card-svg').evaluateAll((els) =>
    els.map((el) => ({
      label: el.getAttribute('aria-label') ?? '',
      state: el.getAttribute('data-state'),
      picture: [...el.querySelectorAll('use')].map((u) => u.getAttribute('href') ?? ''),
      fills: [...el.querySelectorAll('path')].map((p) => p.getAttribute('fill') ?? ''),
    })),
  );
  if (cards.length !== 32) fail(`Kreidetafel shows ${cards.length} cards`);
  const names = ['Augen', 'Rang', 'Trumpf', 'Letzter Stich', 'Erster Stich', 'Gutschein', 'Gutschein Untot', 'Gutschein Wandelkarte', 'Durchsichtig', 'Ramponiert', 'Fleckig', 'Wandelkarte', 'Untot'];
  const missing = names.filter((n) => !cards.some((c) => c.label.includes(n)));
  if (missing.length) fail(`Kreidetafel: no card names ${missing.join(', ')}`);
  const card = (name) => cards.find((c) => c.label.startsWith(`${name},`));
  const expect = (name, ok, what) => {
    const c = card(name);
    if (!c || !ok(c)) fail(`frame language: ${name} ${what} (${JSON.stringify(c)})`);
  };
  // Effekte v3 (tools/cardart/fx3.py): frame = what, mark top right =
  // when, band at the bottom middle = how much (its colour per
  // modification), curses anthracite; Untot and Wandelkarte change the
  // picture; one normal modification per card, a special card one more.
  const band = { Augen: '#d98e0b', Rang: '#8e2430', Trumpf: '#6a3fa8', 'Letzter Stich': '#16707a', 'Erster Stich': '#d2552a', Gutschein: '#2f7d6a', Durchsichtig: '#33363b', Ramponiert: '#33363b', Fleckig: '#33363b' };
  expect('Schellen-Zehn', (c) => c.fills.includes(band.Augen) && c.fills.includes('#ffe08a'), 'has no gold Augen frame and band');
  expect('Schellen-Unter', (c) => c.fills.includes(band.Rang), 'has no wine Rang band');
  expect('Schellen-König', (c) => c.fills.includes(band.Trumpf), 'has no purple Trumpf frame');
  expect('Schellen-Daus', (c) => c.fills.includes(band['Letzter Stich']), 'has no petrol Letzter-Stich frame');
  expect('Rot-Sieben', (c) => c.fills.includes(band['Erster Stich']), 'has no rust Erster-Stich frame');
  for (const name of ['Rot-Acht', 'Rot-Neun', 'Rot-Zehn']) expect(name, (c) => c.fills.includes(band.Gutschein) && c.fills.includes('#bfe3d6'), 'has no Gutschein stamp margin');
  for (const name of ['Schellen-Sieben', 'Schellen-Acht', 'Schellen-Neun']) {
    expect(name, (c) => c.state === 'curse' && c.fills.includes(band.Ramponiert) && !c.picture.some((h) => h.startsWith('#m-')), 'is not an anthracite curse without badge');
  }
  expect('Schellen-Sieben', (c) => c.fills.includes('#1b1e21'), 'Ramponiert shows no tears');
  expect('Rot-Unter', (c) => c.picture.includes('#w-J') && c.picture.includes('#wx-J'), 'Wandelkarte shows no joker face');
  expect('Rot-Daus', (c) => c.picture.includes('#w-A') && c.fills.includes(band.Trumpf), 'Wandelkarte with Trumpf shows not both');
  expect('Rot-König', (c) => c.state === 'undead' && c.picture.includes('#u-HK') && c.fills.includes(band.Augen), 'Untot with Augen shows no zombie face and gold band');
  expect('Grün-Sieben', (c) => c.state === 'both' && c.picture.includes('#u-S7') && c.fills.includes('#1b1e21'), 'Untot number card with Ramponiert shows no rotten pips and tears');
  log(`Effekte v3: ${cards.filter((c) => c.label.includes(',')).length} modified cards on the Kreidetafel, each with its frame and band`);
  await page.locator('.deck-grid').scrollIntoViewIfNeeded();
  await shoot(page, 'frames-gallery');
  await context.close();
}

/**
 * Gutschein (voucher.json: frames.json with "Gutschein" on the Grün-Neun
 * and "Gutschein Untot" on the Grün-Acht, both dealt to the player in the
 * boss game). With animations: both pressed, the stamp margin tears off,
 * a coin flies to the Groschen counter (+3), the Grün-Acht turns over into
 * its Untot face; a toast above the Stammgäste names each voucher.
 */
async function voucherPass(browser, errors) {
  const context = await quietContext(browser, { viewport: PHONE });
  const page = await context.newPage();
  watch(page, errors);
  await resumeFixture(page, 'voucher.json');
  await page.waitForSelector('[data-testid="shop"]');
  await page.click('[data-action="leave-shop"]');
  await page.click('[data-action="boss-go"]');
  // Reaching the boss pub earns "Vor dem Boss" with a fresh meta state.
  if (await page.waitForSelector('.sheet-unlock', { timeout: 3000 }).catch(() => null)) {
    await page.click('.sheet-unlock [data-action="close-overlay"]');
  }
  await page.waitForSelector('[data-action="pick-up-skat"]');
  for (const id of [17, 18]) if (!(await page.locator(`.hand-card[data-id="${id}"] path[fill="#bfe3d6"]`).count())) fail(`Gutschein: card ${id} shows no stamp margin in the hand`);
  await page.click('[data-action="pick-up-skat"]');
  await page.waitForSelector('[data-action="discard"]');
  await page.evaluate(() => {
    window.__toasts = [];
    const layer = document.querySelector('.toast-layer');
    new MutationObserver(() => {
      const t = layer?.textContent?.trim();
      if (t) window.__toasts.push(t);
    }).observe(layer, { childList: true, subtree: true, characterData: true });
  });
  for (const id of await page.locator('.skat-card[data-action="select"]').evaluateAll((els) => els.map((e) => e.dataset.id))) {
    await page.click(`.skat-card[data-id="${id}"]`);
    await page.waitForSelector(`.skat-card[data-id="${id}"]`, { state: 'detached' });
  }
  for (const id of [18, 17]) {
    await page.click(`.hand-card[data-id="${id}"]`);
    await page.waitForSelector(`.skat-card[data-id="${id}"]`);
  }
  const coins = Number(await text(page, 'coins'));
  await page.click('[data-action="discard"]');
  const stages = new Set();
  const deadline = Date.now() + 15000;
  let shotTorn = false;
  let turnedFace = '';
  while (Date.now() < deadline && !(stages.has('coin') && stages.has('turned'))) {
    const now = await page.evaluate(() => {
      const box = document.querySelector('[data-testid="voucher-redeem"]');
      return box ? { stage: box.dataset.stage, uses: [...box.querySelectorAll('use')].map((u) => u.getAttribute('href')) } : null;
    });
    if (now) {
      stages.add(now.stage);
      if (now.stage === 'torn' && !shotTorn) {
        shotTorn = true;
        await page.screenshot({ path: join(screens, 'voucher-torn-390x844.png') });
      }
      if (now.stage === 'turned') turnedFace = now.uses.join(' ');
    }
    await page.waitForTimeout(30);
  }
  for (const st of ['torn', 'coin', 'turned']) if (!stages.has(st)) fail(`Gutschein: no redeem step "${st}" (saw ${[...stages].join(', ')})`);
  if (!turnedFace.includes('#u-S8')) fail(`Gutschein Untot: the Grün-Acht did not turn into its Untot face (${turnedFace})`);
  await page.waitForSelector('[data-testid="voucher-redeem"]', { state: 'detached' });
  const after = Number(await text(page, 'coins'));
  if (after !== coins + 3) fail(`Gutschein: Groschen ${coins} -> ${after}, expected +3`);
  await page
    .waitForFunction(() => window.__toasts.some((t) => t.startsWith('Gutschein eingelöst')) && window.__toasts.some((t) => t.startsWith('Gutschein Untot eingelöst')), null, { timeout: 15000 })
    .catch(async () => fail(`Gutschein: no toast per voucher (${(await page.evaluate(() => window.__toasts)).join(' | ')})`));
  log(`Gutschein: margin torn, coin to the counter (${coins} -> ${after}), Grün-Acht turned Untot`);
  await shoot(page, 'voucher-redeemed');
  await context.close();
}

/**
 * Replace confirm (mods-confirm.json: frames.json with "Augen +3" and "Rang
 * +1" in the bag, 20 Groschen). At the Tresen "Augen +3" on the
 * Schellen-König (Trumpf) asks "ersetzt Trumpf durch Augen" and "Lieber
 * nicht" goes back to the picker; on the Schellen-Zehn (Augen +4) it shows
 * the raise "+4 → +7" and applies it. Cursed cards are dimmed with the
 * reason "Fluch". "Rang +1" refuses the Schellen-Oberin with
 * "Höchstwert": the fixture saved her at Rang 3 (before the cap of
 * 02.11.2026), the core loads her at Rang 2, the maximum. The Stempelkissen bought and used on the Schellen-König
 * only says that the card carries a modification already (drawn at random).
 */
async function modConfirmPass(browser, errors, reducedMotion) {
  const context = await quietContext(browser, { viewport: PHONE, reducedMotion });
  const page = await context.newPage();
  watch(page, errors);
  await resumeFixture(page, 'mods-confirm.json');
  await page.waitForSelector('[data-testid="shop"]');
  await page.click('[data-action="use-item"][data-slot="0"]');
  await page.waitForSelector('.target-grid');
  const reasons = await page.locator('.target-card[data-reason]').evaluateAll((els) => els.map((e) => ({ id: e.dataset.id, reason: e.dataset.reason, tag: e.querySelector('.target-reason')?.textContent ?? '' })));
  for (const id of ['0', '1', '2', '16']) if (!reasons.some((r) => r.id === id && r.reason === 'CardCursed' && r.tag === 'Fluch')) fail(`replace confirm: cursed card ${id} not dimmed with "Fluch" (${JSON.stringify(reasons)})`);
  await shoot(page, 'mod-picker');
  await page.click('.target-card[data-id="6"]');
  await page.waitForSelector('[data-testid="mod-confirm-text"]');
  const replace = await text(page, 'mod-confirm-text');
  if (replace !== 'Schellen-König: ersetzt Trumpf durch Augen.') fail(`replace confirm: says "${replace}"`);
  if ((await page.locator('[data-testid="mod-confirm-cards"] .card').count()) !== 2) fail('replace confirm: does not show the card now and after');
  await shoot(page, 'mod-replace');
  await page.click('[data-testid="mod-confirm-back"]');
  await page.waitForSelector('.target-grid');
  await page.click('.target-card[data-id="3"]');
  await page.waitForSelector('[data-testid="mod-confirm-text"]');
  const raise = await text(page, 'mod-confirm-text');
  if (raise !== 'Schellen-Zehn: Augen +4 → +7.') fail(`raise confirm: says "${raise}"`);
  await shoot(page, 'mod-raise');
  await page.click('[data-testid="mod-confirm-go"]');
  await page.waitForSelector('[data-testid="mod-confirm-text"]', { state: 'detached' });
  await page.waitForFunction(() => {
    const run = JSON.parse(localStorage.getItem('stammtisch.run') ?? '{}').run;
    return run?.edition?.cards?.find((c) => c.id === 3)?.mod_values?.points === 7;
  }, null, { timeout: 5000 }).catch(() => fail('raise confirm: the Schellen-Zehn does not carry Augen +7 afterwards'));
  // "Rang +1" (now slot 0): the Oberin (saved at Rang 3, loaded at 2) is at the maximum.
  await page.click('[data-action="use-item"][data-slot="0"]');
  await page.waitForSelector('.target-grid');
  if (!(await page.locator('.target-card[data-id="5"][data-reason="ModAtMaximum"]').count())) fail('Rang +1: the Schellen-Oberin at Rang 2 is not refused as "Höchstwert"');
  await page.click('.sheet [data-action="close-overlay"]');
  // Stempelkissen: drawn at random, only the hint that the card carries one.
  await page.click('[data-action="buy-item"]:not([disabled])');
  await page.waitForSelector('[data-action="use-item"][data-slot="1"]');
  await page.click('[data-action="use-item"][data-slot="1"]');
  await page.waitForSelector('.target-grid');
  await page.click('.target-card[data-id="6"]');
  await page.waitForSelector('[data-testid="mod-confirm-text"]');
  const random = await text(page, 'mod-confirm-text');
  if (!random.startsWith('Schellen-König trägt schon Trumpf.')) fail(`random confirm: says "${random}"`);
  await page.click('[data-testid="mod-confirm-back"]');
  log(`replace confirm: "${replace}" / "${raise}" / "${random}"`);
  await context.close();
}

/** "Untot": a card rises from the trick back into the hand (undead.json). */
async function undeadPass(browser, errors, reducedMotion) {
  const context = await quietContext(browser, { viewport: PHONE, reducedMotion });
  const page = await context.newPage();
  watch(page, errors);
  await resumeFixture(page, 'undead.json');
  await page.waitForSelector('.declare');
  await page.evaluate(() => {
    window.__toasts = [];
    const layer = document.querySelector('.toast-layer');
    new MutationObserver(() => {
      const t = layer?.textContent?.trim();
      if (t) window.__toasts.push(t);
    }).observe(layer, { childList: true, subtree: true, characterData: true });
  });
  if (!(await page.locator('.hand-card[data-id="22"] use[href="#u-SK"]').count())) fail('Untot: the Grün-König shows no zombie face');
  await page.click('[data-action="play-hand"]');
  await page.waitForSelector('.declare-hand');
  await page.click('[data-action="pick-game"][data-index="0"]');
  if (await page.locator('[data-action="pick-risk"][data-risk="Hand"]').count()) await page.click('[data-action="pick-risk"][data-risk="Hand"]');
  await page.click('[data-action="declare"]');
  await page.waitForSelector('.declare', { state: 'detached' });
  await page.waitForSelector('[data-action="play"][data-id="31"]', { timeout: 30000 });
  // In the suit game both can rise: the skull plaque on each.
  for (const id of ['31', '22']) {
    if (!(await page.locator(`.hand-card[data-id="${id}"] .plaque-rise`).count())) fail(`Untot: card ${id} shows no skull plaque in the hand`);
  }
  await shoot(page, 'undead-hand');
  await page.click('[data-action="play"][data-id="31"]');
  await page.waitForFunction(() => window.__toasts.some((t) => t.startsWith('Untot:')), null, { timeout: 30000 });
  await page.waitForSelector('[data-action="play"]', { timeout: 30000 });
  await page.waitForFunction(() => !document.querySelector('.run.is-busy'));
  if (!(await page.locator('.hand-card[data-id="31"]').count())) fail('Untot: the Eichel-Daus did not come back into the hand');
  if (await page.locator('.hand-card[data-id="31"] .plaque-rise').count()) fail('Untot: the risen Daus still says it can rise');
  await page.click('[data-testid="last-trick-toggle"]');
  await page.waitForSelector('[data-testid="last-trick"]');
  if (await page.locator('[data-testid="last-trick"] .trick-card[data-card-id="31"]').count()) fail('Untot: the Daus still lies in the last trick');
  if ((await page.locator('[data-testid="last-trick"] .trick-card').count()) !== 3) fail('Untot: the last trick does not hold three cards');
  if (!(await page.locator('[data-testid="last-trick"] [data-testid="rose-note"]').count())) fail('Untot: the substitute in the last trick has no note');
  const toast = await page.evaluate(() => window.__toasts.find((t) => t.startsWith('Untot:')));
  log(`Untot: ${toast}`);
  await shoot(page, 'undead-last-trick');
  await context.close();
}

async function wave3(browser, errors, reducedMotion) {
  const context = await quietContext(browser, { viewport: PHONE, reducedMotion });
  const page = await context.newPage();
  watch(page, errors);
  await resumeFixture(page, 'wave3-mods.json');
  await page.waitForSelector('.declare');
  await page.evaluate(() => {
    window.__toasts = [];
    const layer = document.querySelector('.toast-layer');
    new MutationObserver(() => {
      const t = layer?.textContent?.trim();
      if (t) window.__toasts.push(t);
    }).observe(layer, { childList: true, subtree: true, characterData: true });
  });
  for (const id of ['skat_god', 'honorary_chair']) {
    const mat = page.locator(`[data-info="relic:${id}"] svg[data-deckel="${id}"]`);
    if (!(await mat.count())) fail(`legendary relic ${id} has no beer mat in the head`);
    if (!(await mat.first().locator('[fill="#ff8a1c"], [stroke="#ff8a1c"]').count())) fail(`legendary relic ${id} lacks the legendary ink`);
  }
  const labels = await page.locator('.hand-card').evaluateAll((els) => els.map((e) => e.getAttribute('aria-label') ?? '').join(' | '));
  // Effekte v2: the old save's Glasdaus became Augen +11, Heimweh fell away.
  for (const mod of ['Wandelkarte', 'Augen']) {
    if (!labels.includes(mod)) fail(`wave 3: no hand card names ${mod}: ${labels}`);
  }
  await shoot(page, 'wave3-hand');
  // The info card names the tier.
  await page.click('[data-info="relic:skat_god"]');
  await page.waitForSelector('[data-testid="info-card"]');
  const info = ((await page.locator('[data-testid="info-card"]').textContent()) ?? '').replace(/\s+/g, ' ');
  if (!info.includes('legendär')) fail(`Skatgott info card: ${info}`);
  await shoot(page, 'wave3-legendary-info');
  await page.click('[data-testid="info-card"] [data-info-close]');
  await page.waitForSelector('[data-testid="info-card"]', { state: 'detached' });
  await page.click('[data-action="play-hand"]');
  await page.waitForSelector('.declare-hand');
  await page.click('[data-action="pick-game"][data-index="0"]');
  if (await page.locator('[data-action="pick-risk"][data-risk="Hand"]').count()) await page.click('[data-action="pick-risk"][data-risk="Hand"]');
  await page.click('[data-action="declare"]');
  await page.waitForSelector('.declare', { state: 'detached' });
  // "Ehrenvorsitz": the player leads. The Grün-Daus (id 23, a Glasdaus in
  // the old save, dealt before effects v2) first; it no longer breaks.
  await page.waitForSelector('[data-action="play"][data-id="23"]', { timeout: 30000 });
  await page.click('[data-action="play"][data-id="23"]');
  // The Wandelkarte (Rot-Daus, id 15): a Schellen lead, only Schellen may be named.
  await page.waitForSelector('[data-action="play"][data-id="15"]', { timeout: 30000 });
  await page.click('[data-action="play"][data-id="15"]');
  await page.waitForSelector('[data-testid="wild-suits"]');
  const allowed = await page.locator('[data-action="wild-suit"]:not([disabled])').evaluateAll((els) => els.map((e) => e.getAttribute('data-suit')).join(','));
  if (allowed !== 'Diamonds') fail(`Wandelkarte: the picker allows ${allowed}, expected Diamonds`);
  await page.waitForFunction(() => !document.querySelector('.toast-layer')?.textContent?.trim(), null, { timeout: 15000 });
  await shoot(page, 'wave3-wild-picker');
  await page.click('[data-testid="wild-Diamonds"]');
  await page.waitForSelector('[data-testid="wild-suits"]', { state: 'detached' });
  await page.waitForFunction(() => window.__toasts.some((t) => t.includes('Wandelkarte')), null, { timeout: 30000 });
  // The trick on the table, or once it is finished the "Letzter Stich".
  await page.waitForSelector('[data-action="play"], .score-sheet', { timeout: 30000 });
  if (!(await page.locator('.trick-card[data-card-id="15"]').count())) {
    await page.waitForFunction(() => !document.querySelector('.run.is-busy'));
    await page.click('[data-testid="last-trick-toggle"]');
    await page.waitForSelector('[data-testid="last-trick"]');
  }
  const wild = page.locator('.trick-card[data-card-id="15"]').first();
  if (!(await wild.locator('.plaque-wild').count())) fail('the named Wandelkarte has no plaque on the table');
  const shown = await wild.locator('svg.card-svg').getAttribute('aria-label');
  if (!(shown ?? '').startsWith('Schellen-Daus')) fail(`the named Wandelkarte shows as ${shown}`);
  await page.waitForFunction(() => !document.querySelector('.toast-layer')?.textContent?.trim(), null, { timeout: 15000 });
  await shoot(page, 'wave3-wild-played');
  if (await page.locator('[data-testid="last-trick"]').count()) {
    await page.click('[data-testid="last-trick"]');
    await page.waitForSelector('[data-testid="last-trick"]', { state: 'detached' });
  }
  await playOut(page, 'wave 3');
  await shoot(page, 'wave3-score');
  const toasts = await page.evaluate(() => window.__toasts.join(' | '));
  log(`wave 3: ${toasts.replace(/\s+/g, ' ').slice(0, 300)}`);
  await context.close();
}

/** Pick the way over a stop of `category` on the map, then "Los". */
async function takeRoute(page, category) {
  await page.locator(`[data-testid="route-option"][data-category="${category}"]`).first().click();
  await page.waitForSelector(`[data-testid="route-option"][data-category="${category}"][aria-pressed="true"]`);
  await page.click('[data-testid="route-go"]');
}

/** The map as the player sees it: the SVG with its image, the highlighted way, the targets. */
async function mapState(page) {
  return page.evaluate(() => {
    const svg = document.querySelector('[data-testid="map-svg"]');
    const box = (sel) => {
      const el = document.querySelector(sel);
      if (!el) return null;
      const b = el.getBoundingClientRect();
      return { x: b.left + b.width / 2, y: b.top + b.height / 2, node: el.getAttribute('data-node') };
    };
    const sel = svg?.querySelector('.map-route.is-selected');
    const image = svg?.querySelector('image');
    return {
      svg: svg ? svg.outerHTML : '',
      image: image ? (image.getAttribute('href') ?? '').length : 0,
      routes: svg ? svg.querySelectorAll('.map-route').length : 0,
      selected: sel?.getAttribute('data-route') ?? null,
      cells: sel?.getAttribute('data-cells') ?? '',
      badge: svg?.querySelector('[data-testid="map-badge"]')?.getAttribute('data-category') ?? null,
      target: box('[data-testid="map-target"]'),
      spelunke: box('[data-testid="map-spelunke"]'),
      pos: box('[data-testid="map-pos"]'),
    };
  });
}

/**
 * The route screen fits like the table (FIT_SIZES, PWA sizes included):
 * the page does not scroll, the map has room, the travel bar with "Los"
 * ends inside the viewport and the target card does not cover the
 * position or the target.
 */
async function assertRouteFits(page, label) {
  for (const size of FIT_SIZES) {
    await page.setViewportSize({ width: size.width, height: size.height });
    const insets = size.insets ?? { top: 0, bottom: 0 };
    await page.evaluate((i) => {
      const root = document.documentElement.style;
      if (i.top) root.setProperty('--safe-top', `${i.top}px`);
      if (i.bottom) root.setProperty('--safe-bottom', `${i.bottom}px`);
    }, insets);
    await page.waitForTimeout(120);
    const r = await page.evaluate(() => {
      const rect = (sel) => document.querySelector(sel)?.getBoundingClientRect() ?? null;
      return {
        scroll: document.scrollingElement.scrollHeight,
        inner: window.innerHeight,
        wide: document.scrollingElement.scrollWidth > window.innerWidth,
        map: rect('[data-testid="route-map"]'),
        go: rect('[data-testid="route-go"]'),
        card: rect('[data-testid="route-next"]'),
        pos: rect('[data-testid="map-pos"]'),
        target: rect('[data-testid="map-target"]'),
      };
    });
    await page.evaluate(() => {
      document.documentElement.style.removeProperty('--safe-top');
      document.documentElement.style.removeProperty('--safe-bottom');
    });
    const where = `${label} at ${size.width}x${size.height}${size.insets ? ' (PWA, safe areas)' : ''}`;
    if (r.scroll > r.inner) fail(`${where}: the page scrolls (${r.scroll} > ${r.inner})`);
    if (r.wide) fail(`${where}: the page scrolls sideways`);
    if (!r.map || r.map.height < 300) fail(`${where}: the map is ${r.map ? Math.round(r.map.height) : 0}px high`);
    if (!r.go || r.go.bottom > r.inner - insets.bottom + 0.5) fail(`${where}: "Los" ends at ${r.go ? Math.round(r.go.bottom) : '?'}, below ${r.inner - insets.bottom}`);
    if (r.card && r.pos && r.pos.top + r.pos.height / 2 > r.card.top) fail(`${where}: the target card covers the position`);
    if (r.card && r.target && r.target.bottom > r.card.top) fail(`${where}: the target card covers the target`);
  }
  await page.setViewportSize(PHONE);
  await page.waitForTimeout(80);
  log(`${label}: route screen fits ${FIT_SIZES.map((s) => `${s.width}x${s.height}${s.insets ? ' PWA' : ''}`).join(', ')}`);
}

/**
 * The route choice on the city map (Kneipentour, ADR 0016, design "Karte
 * Runde 3"): hearts in the head, the map of the leg (a pixel image, three
 * ways as paths, the position, the next pub), the target card (name,
 * target, guests), three transport buttons each with its stop, a Tresen
 * among them, and "Los". A tap on another transport highlights its way;
 * "−" shows the whole tour as a line plan, "+" goes back. Fits the phone,
 * the desktop and an installed app without scrolling.
 */
async function routePass(page, label) {
  await page.waitForSelector('[data-testid="route"]');
  const options = page.locator('[data-testid="route-option"]');
  if ((await options.count()) !== 3) fail(`${label}: ${await options.count()} ways, expected 3`);
  const categories = await options.evaluateAll((els) => els.map((e) => e.getAttribute('data-category')));
  if (!categories.includes('Tresen')) fail(`${label}: no way over the Tresen: ${categories}`);
  const transports = await options.evaluateAll((els) => els.map((e) => e.getAttribute('data-transport')).join(','));
  if (transports !== 'Walk,Train,Bike') fail(`${label}: ways ${transports}`);
  const next = ((await page.locator('[data-testid="route-next"]').textContent()) ?? '').replace(/\s+/g, ' ');
  if (!/Ziel \d+ Augen in 3 Spielen/.test(next) || !next.includes('Am Tisch:')) fail(`${label}: next pub ${next}`);
  const node = await page.locator('[data-testid="route-next"]').getAttribute('data-node');
  if (!/^d\d-p\d$/.test(node ?? '')) fail(`${label}: node id ${node}`);
  if ((await page.locator('.hud .heart').count()) < 3) fail(`${label}: no hearts in the head of the route screen`);
  if (!(await page.locator('.hud [data-testid="coins"]').count())) fail(`${label}: no Groschen in the head of the route screen`);
  // The map: the city as an image, three ways, one highlighted with its stop.
  const map = await mapState(page);
  if (map.image < 1000) fail(`${label}: the map has no city image`);
  if (map.routes !== 3) fail(`${label}: ${map.routes} ways on the map`);
  const picked = await page.locator('[data-testid="route-option"][aria-pressed="true"]').getAttribute('data-transport');
  if (map.selected !== picked) fail(`${label}: the map highlights ${map.selected}, the bar ${picked}`);
  const pickedCategory = await page.locator('[data-testid="route-option"][aria-pressed="true"]').getAttribute('data-category');
  if (map.badge !== pickedCategory) fail(`${label}: the badge shows ${map.badge}, the way stops at ${pickedCategory}`);
  if (map.target?.node !== node) fail(`${label}: the map's target is ${map.target?.node}, the card's ${node}`);
  if (!map.pos) fail(`${label}: no position on the map`);
  log(`${label}: ${categories.join(' / ')}, next ${next.slice(0, 80)}`);
  await shoot(page, 'route');
  await assertNoSideScroll(page, label, '.route');
  await assertRouteFits(page, label);
  // Switching the transport highlights another way (another path).
  const other = transports.split(',').find((t) => t !== picked);
  await page.click(`[data-testid="route-option"][data-transport="${other}"]`);
  await page.waitForSelector(`[data-testid="route-option"][data-transport="${other}"][aria-pressed="true"]`);
  const switched = await mapState(page);
  if (switched.selected !== other) fail(`${label}: tapping ${other} highlights ${switched.selected}`);
  if (switched.cells === map.cells) fail(`${label}: ${other} highlights the same path as ${picked}`);
  if ((await page.getAttribute('[data-testid="route-go"]', 'data-option')) !== (await page.getAttribute(`[data-testid="route-option"][data-transport="${other}"]`, 'data-option'))) fail(`${label}: "Los" does not take the highlighted way`);
  log(`${label}: ${picked} -> ${other} switches the highlighted way`);
  await shoot(page, 'route-switched');
  // "−": the whole tour as a line plan; "+" back to the map.
  if (!(await page.locator('[data-testid="zoom-in"]').isDisabled())) fail(`${label}: "+" is active on the map`);
  await page.click('[data-testid="zoom-out"]');
  await page.waitForSelector('[data-testid="route-plan"]');
  if (await page.locator('[data-testid="map-svg"]').count()) fail(`${label}: the map stays under the overview`);
  if (!(await page.locator('[data-testid="zoom-out"]').isDisabled())) fail(`${label}: "−" is active in the overview`);
  if ((await page.locator('.route-plan .plan-district').count()) !== 3) fail(`${label}: the overview does not show three districts`);
  if (!(await page.locator('[data-testid="plan-here"]').count())) fail(`${label}: the overview does not say where the player is`);
  if ((await page.locator('.route-plan .plan-stop.is-done').count()) < 1) fail(`${label}: the overview shows no pub done`);
  await shoot(page, 'route-overview');
  await assertRouteFits(page, `${label} (overview)`);
  await page.click('[data-testid="zoom-in"]');
  await page.waitForSelector('[data-testid="map-svg"]');
  if (await page.locator('[data-testid="route-plan"]').count()) fail(`${label}: "+" did not close the overview`);
  // Same seed, same map: after a reload the leg is drawn identically.
  await page.click(`[data-testid="route-option"][data-transport="${picked}"]`);
  const again = await mapState(page);
  if (again.svg !== map.svg) fail(`${label}: the map changed between two renders of the same leg`);
  await page.reload();
  await page.click('[data-action="resume"]');
  await page.waitForSelector('[data-testid="map-svg"]');
  const reloaded = await mapState(page);
  if (reloaded.svg !== map.svg) fail(`${label}: the map is not the same after a reload (same seed)`);
  log(`${label}: map identical after a reload (${Math.round(map.svg.length / 1024)} kB)`);
}

/**
 * The map from saved runs (e2e/fixtures, played by the core's RunBot):
 * route-spelunke.json (seed 1) has a way into a Spelunke. It stands on the
 * map as a place of its own, away from the next pub; picking that way
 * shows the Spelunke on the card with its malus. route-station.json has
 * the leg into the second district after the boss pub, drawn with the look
 * the core gives it, the head without the boss chip. With
 * full motion the position travels along the chosen way and leaves a
 * chalk trail before the stop opens.
 */
async function mapPass(browser, errors) {
  const context = await quietContext(browser, { viewport: PHONE, reducedMotion: 'reduce' });
  const page = await context.newPage();
  watch(page, errors);
  await resumeFixture(page, 'route-spelunke.json');
  await page.waitForSelector('[data-testid="map-svg"]');
  const map = await mapState(page);
  if (!map.spelunke || !map.target) fail('Spelunke: no Spelunke or no target on the map');
  const apart = Math.hypot(map.spelunke.x - map.target.x, map.spelunke.y - map.target.y);
  if (apart < 40) fail(`Spelunke: drawn ${Math.round(apart)}px from the next pub, not a place of its own`);
  const way = page.locator('[data-testid="route-option"][data-spelunke="true"]');
  if ((await way.count()) !== 1) fail(`Spelunke: ${await way.count()} ways into a Spelunke`);
  if ((await way.getAttribute('aria-pressed')) === 'true') fail('Spelunke: the way into the Spelunke is picked by default');
  await way.click();
  await page.waitForSelector('[data-testid="route-option"][data-spelunke="true"][aria-pressed="true"]');
  const card = await page.getAttribute('[data-testid="route-next"]', 'data-node');
  if (card !== map.spelunke.node || !/^d\d-p\d-s$/.test(card ?? '')) fail(`Spelunke: the card shows ${card}, the map's Spelunke is ${map.spelunke.node}`);
  if (!(await page.locator('[data-testid="route-malus"]').count())) fail('Spelunke: the card shows no malus');
  const end = (await mapState(page)).cells.split(' ').pop();
  log(`Spelunke ${card} ${Math.round(apart)}px from the next pub; its way ends at ${end}`);
  await shoot(page, 'route-spelunke');
  await assertRouteFits(page, 'route with a Spelunke picked');
  await context.close();

  const station = await quietContext(browser, { viewport: PHONE, reducedMotion: 'no-preference' });
  const page2 = await station.newPage();
  watch(page2, errors);
  await resumeFixture(page2, 'route-station.json');
  await page2.waitForSelector('[data-testid="map-svg"]');
  // The look comes from the core's content (tour.ron), one district of each
  // look per evening; the map draws the look the view gives.
  const look = await page2.getAttribute('[data-testid="map-svg"]', 'data-look');
  await page2.click('[data-testid="zoom-out"]');
  await page2.waitForSelector('[data-testid="route-plan"]');
  const looks = await page2.locator('.route-plan .plan-district').evaluateAll((els) => els.map((e) => e.getAttribute('data-look')));
  if (new Set(looks).size !== 3 || looks.some((l) => !['OldTown', 'Harbour', 'Station'].includes(l ?? ''))) fail(`the evening's districts have the looks ${looks}, expected one of each`);
  const drawn = { OldTown: 'altstadt', Harbour: 'hafen', Station: 'bahnhof' }[looks[1] ?? ''];
  if (look !== drawn) fail(`second district drawn as ${look}, its look in the view is ${looks[1]}`);
  log(`district looks from the core: ${looks.join(', ')}; the second drawn as ${look}`);
  await page2.click('[data-testid="zoom-in"]');
  await page2.waitForSelector('[data-testid="map-svg"]');
  // After the boss pub the head shows the district label, no boss chip.
  if (await page2.locator('.hud [data-testid="hud-boss"]').count()) fail('route after the boss pub: the boss chip is still in the head');
  const label = await page2.evaluate(() => {
    const sub = document.querySelector('.hud .hud-where-sub');
    if (!sub) return null;
    const r = sub.getBoundingClientRect();
    const top = document.elementFromPoint(r.left + Math.min(r.width - 2, 40), r.top + r.height / 2);
    return { text: sub.textContent ?? '', width: r.width, onTop: !!top && sub.contains(top) };
  });
  if (!label || !/Stadtteil \d\/\d/.test(label.text) || label.width < 60 || !label.onTop) fail(`route after the boss pub: district label hidden ${JSON.stringify(label)}`);
  log(`route after the boss pub: head shows "${label.text.trim()}"`);
  await shoot(page2, 'route-station');
  await page2.click('[data-testid="route-go"]');
  await page2.waitForSelector('.map-trail');
  await page2.waitForTimeout(500);
  const moved = await page2.evaluate(() => document.querySelector('.map-pos')?.getAttribute('transform') ?? '');
  if (!/translate/.test(moved)) fail('travel: the position does not move along the way');
  await page2.waitForSelector('[data-testid="route"]', { state: 'detached', timeout: 10000 });
  log('travel: the position walked the way with a chalk trail, then the stop opened');
  await station.close();
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
    const sprite = await page.evaluate(() => ['f-CA', 'f-SJ', 'f-HQ', 'f-DK', 'u-S7', 'u-CA', 'w-A', 'wx-T', 'wt-K', 'm-points', 'bk-distel', 'bk-distel-s', 'f-blank', 'p-wirt'].filter((id) => !document.getElementById(id)));
    // Every opponent back (tools/cardart/backs.py) in both sizes, with a picture.
    const backIds = ['regulars', 'black_peter', 'grandma', 'the_pusher', 'ramscher', 'kontra_hans', 'revolutionary', 'kibitz_killer', 'twins', 'the_old_man'];
    const emptyBacks = await page.evaluate((ids) => ids.flatMap((id) => [`bk-${id}`, `bk-${id}-s`]).filter((id) => !document.getElementById(id)?.childElementCount), backIds);
    if (emptyBacks.length) fail(`card backs missing or empty: ${emptyBacks.join(', ')}`);
    if (sprite.length) fail(`card sprite incomplete, missing: ${sprite.join(', ')}`);
    // The pixel deck (tools/cardart): the index layer names its rank and has pixels.
    const aceCorner = await page.evaluate(() => {
      const index = document.getElementById('x-CA');
      return index?.querySelector('path') ? index.getAttribute('data-rank') : null;
    });
    if (aceCorner !== 'A') fail(`the Daus shows "${aceCorner}" in its corner, expected "A"`);
    const fonts = await page.evaluate(async () => {
      await document.fonts.ready;
      const families = ['VT323'];
      await Promise.all(families.map((f) => document.fonts.load(`16px "${f}"`, 'Skat')));
      const loaded = [...document.fonts].filter((f) => f.status === 'loaded').map((f) => f.family.replace(/"/g, ''));
      return families.filter((f) => !loaded.includes(f)).map((f) => `${f} (loaded: ${loaded.join(', ') || 'none'})`);
    });
    if (fonts.length) fail(`fonts not loaded: ${fonts.join(', ')}`);
    const portraits = await page.evaluate(() => ['p-char-Declarer', 'p-char-NullPlayer', 'p-char-GrandPlayer', 'p-char-Onlooker', 'p-char-Landlady', 'p-dealer', 'p-boss-the_old_man'].filter((id) => !document.getElementById(id)));
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
    if (reducedMotion === 'reduce') {
      // Reduced motion: the cards are there at once, nothing flies in (a
      // transition of 1 ms is the reduced-motion stand-in, not a flight).
      const anims = await page.evaluate(() => document.getAnimations().filter((a) => a.effect?.target?.closest?.('.hand, .skat-pile') && Number(a.effect?.getTiming?.().duration) > 1).map((a) => `${a.animationName ?? a.constructor.name} ${a.effect?.target?.getAttribute?.('class')} ${a.effect?.getTiming?.().duration}`));
      const flying = anims.length;
      if (flying) console.log('[smoke] debug anims', anims.join(' | '));
      if (flying) fail(`${flying} cards fly in although reduced motion is asked for`);
    }
    const seated = await assertGuests(page, 'first series');
    const progress = await text(page, 'pub-progress');
    if (!/Spiel 1 von 3 · 0 \/ \d+ Augen/.test(progress)) fail(`pub progress in the head: ${progress}`);
    const hearts = await page.locator('[data-testid="lives"] .heart.is-full').count();
    if (hearts !== 3) fail(`${hearts} hearts in the head, expected 3`);
    log(`head: ${progress}, ${hearts} hearts`);
    if (!seated.quip) fail('no Stammgast says hello at the first game of the series');
    await assertChoice(page, seated);
    await shoot(page, 'table-declare');
    await assertFits(page, 'Skat oder Hand');
    await assertFits(page, 'Skat oder Hand with a quip', [PHONE, { width: 375, height: 667 }]);
    const choiceCard = await page.locator('.hand-card').first().evaluate((e) => e.getBoundingClientRect().height);

    // Pass once: the deck dragged into the middle (Einmischen).
    const passes = Number(await text(page, 'passes-left'));
    await dragOnto(page, '[data-testid="deck"]', '.mix-zone', 12, 'table-deck-drag');
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
    // While choosing Skat oder Hand the hand is bigger than under the Ansage.
    const ansageCard = await page.locator('.hand-card').first().evaluate((e) => e.getBoundingClientRect().height);
    if (!(choiceCard > ansageCard)) fail(`hand cards ${Math.round(choiceCard)}px high while choosing, ${Math.round(ansageCard)}px under the Ansage`);
    log(`hand cards ${Math.round(choiceCard)}px while choosing, ${Math.round(ansageCard)}px under the Ansage`);
    await shoot(page, 'table-declare-hand');
    await assertFits(page, 'hand Ansage');
    // Every character may declare Null (ADR 0012): the Alleinspieler too,
    // with the values of the Skatordnung.
    await page.click('[data-action="pick-game"][data-game="Null"]');
    await page.waitForSelector('[data-testid="null-declare"][data-phase="Declaring"]');
    const declarerNull = await page.locator('[data-action="pick-null-risk"]').evaluateAll((els) => els.map((e) => e.getAttribute('data-risk')).join(','));
    if (declarerNull !== 'Hand,OuvertHand') fail(`the Alleinspieler's hand Ansage offers Null ${declarerNull}`);
    const declarerNullValues = ((await page.locator('.null-risks').textContent()) ?? '').replace(/\s+/g, ' ').trim();
    if (!/Wert 46.*Wert 92/.test(declarerNullValues)) fail(`the Alleinspieler's Null levels: ${declarerNullValues}`);
    await shoot(page, 'table-declare-hand-null');
    await assertFits(page, 'hand Ansage with Null');
    await page.click('[data-action="hand-back"]');
    log(`hand Ansage: ${handRisks}; Null for the Alleinspieler: ${declarerNullValues}`);

    // Skat aufnehmen, drücken, then the Ansage: Grand.
    await pickUpAndDiscard(page, 'first game', 'table-discard');
    // ADR 0013: Schneider, Schwarz and Ouvert also after the skat, no Hand.
    const afterRisks = await page.locator('[data-action="pick-risk"]').evaluateAll((els) => els.map((e) => e.getAttribute('data-risk')).join(','));
    if (afterRisks !== 'Normal,SchneiderDeclared,SchwarzDeclared,Ouvert') fail(`the Ansage after the skat offers ${afterRisks}`);
    log(`Ansage after the skat: ${afterRisks}`);
    // After the skat the Alleinspieler may declare Null and Null Ouvert
    // (ADR 0012), with the values of the Skatordnung.
    await page.click('[data-action="pick-game"][data-game="Null"]');
    await page.waitForSelector('[data-testid="null-declare"][data-phase="DeclaringAfterSkat"]');
    const declarerNullAfter = await page.locator('[data-action="pick-null-risk"]').evaluateAll((els) => els.map((e) => e.getAttribute('data-risk')).join(','));
    if (declarerNullAfter !== 'Normal,Ouvert') fail(`the Alleinspieler's Ansage after the skat offers Null ${declarerNullAfter}`);
    const declarerNullAfterValues = ((await page.locator('.null-risks').textContent()) ?? '').replace(/\s+/g, ' ').trim();
    if (!/Wert 23.*Wert 69/.test(declarerNullAfterValues)) fail(`the Alleinspieler's Null levels after the skat: ${declarerNullAfterValues}`);
    await shoot(page, 'table-declare-after-null');
    await assertFits(page, 'Ansage after the skat with Null');
    log(`Null after the skat for the Alleinspieler: ${declarerNullAfterValues}`);
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
        // The value at stake comes from the core: "Grundwert × Multiplikator".
        const worth = await page.locator('[data-testid="value-preview"]').textContent();
        if (!/^\d+ × \d+( × \d+)?$/.test((worth ?? '').trim())) fail(`value preview while playing: "${worth}"`);
        log(`value preview: ${worth}`);
        await shoot(page, 'table-play');
        await assertFits(page, 'playing');
        await assertPressed(page, 'playing', 'table-pressed');
        await assertTriangle(page, 'playing');
        // Before the first trick is finished there is nothing to look back at.
        const tricks = ((await page.locator('[data-tricks]').textContent()) ?? '').trim();
        if (tricks === 'Stiche 0 : 0' && (await page.locator('[data-testid="last-trick-toggle"]').count())) fail('a "Letzter Stich" button before any trick was finished');
      }
      if (played === 2) await assertLastTrick(page, 'playing');
      if (played === 3 && (await page.locator('[data-testid="last-trick"]').count())) fail('the last trick still shows after playing a card');
      if (played === 3 && dimmed > 0) await shoot(page, 'table-follow');
      if (played === 1) {
        // Thrown with the mouse instead of a tap: let go over the felt.
        const id = await page.locator('[data-action="play"]').first().getAttribute('data-id');
        const felt = await page.locator('[data-testid="trick"]').boundingBox();
        await throwCard(page, `.hand [data-action="play"][data-id="${id}"]`, { x: felt.x + felt.width / 2, y: felt.y + felt.height * 0.6 }, 30);
        await page.waitForFunction((cardId) => !document.querySelector(`.hand-card[data-id="${cardId}"]`), id, { timeout: 10000 }).catch(() => fail(`the card ${id} thrown onto the felt was not played`));
        await page.mouse.move(1, 1);
        log(`card ${id} thrown onto the felt and played`);
      } else {
        await page.locator('[data-action="play"]').first().click();
      }
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

    // Kneipentour (ADR 0016): the pub was won in its first game, so the
    // route choice to the next pub opens: lives as hearts, the next pub
    // with name, target and guests, three ways each with its stop.
    await routePass(page, 'first route');
    await takeRoute(page, 'Tresen');

    // Tresen.
    await page.waitForSelector('[data-testid="shop"]');
    await expectMusic(page, 'shop', 'Tresen');
    const coinsBefore = Number(await text(page, 'coins'));
    const next = await text(page, 'next-series');
    if (!next) fail('the Tresen shows no preview of the next series');
    log(`next series: ${next.replace(/\s+/g, ' ')}`);
    await shoot(page, 'shop');
    await assertNoSideScroll(page, 'Tresen');
    const buy = page.locator('[data-action="buy-relic"]:not([disabled]), [data-action="buy-item"]:not([disabled])');
    if (await buy.count()) {
      await buy.first().click();
      await page.waitForFunction(
        (before) => Number(document.querySelector('[data-testid="coins"]')?.textContent) < before,
        coinsBefore,
      );
      log(`bought at the Tresen: ${coinsBefore} -> ${await text(page, 'coins')} Groschen`);
      await shoot(page, 'shop-bought');
      await assertNoSideScroll(page, 'Tresen after buying');
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

    await mapPass(browser, errors);
    await musicPass(browser, errors, reducedMotion);
    await suitTintPass(browser, errors, reducedMotion);
    await bossAndKontra(browser, errors, reducedMotion);
    await m4(browser, errors, reducedMotion);
    await m5(browser, errors, reducedMotion);
    await wave3(browser, errors, reducedMotion);
    await framesPass(browser, errors, reducedMotion);
    await modConfirmPass(browser, errors, reducedMotion);
    await voucherPass(browser, errors);
    await undeadPass(browser, errors, reducedMotion);
    await shortEvening(browser, errors, reducedMotion);
    await wave2b(browser, errors, reducedMotion);
    await saveCode(browser, errors, reducedMotion, PHONE, true);
    await saveCode(browser, errors, reducedMotion, DESKTOP, false);
    await hintsAndRules(browser, errors, reducedMotion);
    await motionPass(browser, errors);
    await tableFeelPass(browser, errors, PHONE);
    await tableFeelPass(browser, errors, DESKTOP);
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

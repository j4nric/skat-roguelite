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
//   one the core does not let go springs back and shakes; a skat card
//   tapped on the table is lifted and flies into its sorted slot in the
//   hand (sampled every frame), a hand card tapped flies onto the table,
//   on Drücken both fly into the Ansage and after it into the stack in
//   the felt's corner, turning face down,
// - "Kartenfarbe tönen" (suitTintPass): the menu switch tints each card's
//   paper in its suit (off by default, stored, survives a reload), and each
//   rank letter in the corner has its suit's dark ink, never black,
// - visits the Tresen, buys something if possible, opens the Kreidetafel and
//   the menu (tempo setting is stored), and starts the next series. The
//   Tresen (and an encounter) never scroll sideways at 390 × 844, 375 × 667
//   and 360 × 740: the page is no wider than the viewport, no element leaves it.
// - with full motion (rewardFxPass, 390 × 844 and 1280 × 800): the score
//   board counts the account up with its bar, foam crowns the full bar, the
//   payout's and an encounter's Groschen fly into the purse; the lights go
//   out before the end of a lost run (ADR 0019: a lost pub ends it); a won
//   run (the last boss pub, patched from boss-table.json) gets a firework; a pressed button gives way and springs back; nothing scrolls
//   sideways.
// - Die lange Nacht (nightPass, 390 × 844 and 1280 × 800; alone with
//   SMOKE_ONLY=night): the won run's end screen offers the night, the start
//   sheet of Stadtteil 4 shows its Verschärfung, the HUD its chip, the
//   first night pub its grown Zielwert, a night relic its info card and the
//   start screen the Bestenliste.
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
//   before the boss series of Der Kontra-Hans (removed by ADR 0017: the save
//   meets his stand-in Die Zwillinge). Leaving the Tresen must show the boss
//   dialog with both boss players, then both sit at the table.
// - boss-route.json, boss-table.json, boss-oma.json, boss-reward.json,
//   boss-reward-full.json, boss-lost.json (ADR 0017, written by the core's
//   `regenerate_boss_fixtures`, bossesV2): the route to the boss pub names
//   both players; Fritz and Franz with name plates and the twins' back on
//   both fans; the boss rule written out before the first card, during
//   play a chip in the felt's corner that opens the rule, the trick cards
//   as wide as in a normal pub (within 2 px at 390 × 844, 1280 × 800 and
//   375 × 667); Die Oma's 16 curses; the relic pick with few relics, with
//   nine (slots unlimited since 03.11.2026: taken without a replace, the
//   HUD strip scrolls inside itself) and passed; the last
//   game of a boss pub that cannot be won ends the run.
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
//   Both encounter saves hold the form before 03.11.2026 (options with a
//   `choice` and "Weitergehen"); the core opens them anew with two or three
//   options, one of which must be taken. coins-curse.json and
//   route-station.json are patched in the test (`patched`) to the new
//   encounter form and to the new stops (Fundgroschen, Kiosk, Würfelbude).
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
// Effekte v3 and "Untot" (framesPass, zinkenPass, voucherPass, undeadPass):
// - frames.json: boss-tresen.json with its deck patched by text (mods_version
//   2): every modification of effects v2 once, plus Untot with Augen +5,
//   Wandelkarte with Trumpf +4 and Untot with Ramponiert. On the Kreidetafel
//   (the gallery of every modification) every one is named and drawn as on
//   the board "Effekte v3": its frame, mark and band colour, curses in
//   anthracite without badge, Untot with its zombie face (rotten pips on a
//   number card), the Wandelkarte with its joker face.
// - zinken.json: frames.json (removed items out of the bag) with an
//   Auslage of three set offers and 20 Groschen: the Zinken at the Tresen
//   (DESIGN 8.6), the in-app sheet before a Zinke replaces or raises
//   another, the Kartenmaler and Karte reinigen (see zinkenPass).
// - card-reward.json: seed 5 after the first pub, the card reward open
//   with three pairs: Winkel on the Schellen-König, Aufdruck on the
//   Grün-Daus (already +3) and the Narrenmarke on the Herz-König (core:
//   regenerate_card_reward_fixture; cardRewardPass).
// - card-reward-v1.json: the same reward in the shape of before
//   05.11.2026 (three Zinken without cards); loading draws pairs again.
// - voucher.json: frames.json with "Gutschein" on the Grün-Neun and
//   "Gutschein Untot" on the Grün-Acht, both dealt to the player in the boss
//   game: pressed, they are redeemed with their animation (voucherPass).
// - voucher.json (since 03.11.2026 without the other Untot cards of
//   frames.json, one Untot per edition): the Totenmarke turns the Grün-Acht
//   Untot, and the first Untot of the save opens its explanation sheet.
// - special.json: zinken.json without special cards, its Auslage set to
//   "wird Untot", "wird Wandelkarte" and a plain Zinke (specialPass).
// - zinken.json patched in the browser (tradePass, curses as trade,
//   DESIGN 8.8, at 390 × 844 and 1280 × 800): the relics on offer as a row
//   of three tiles like the Auslage, Pechvogel as "Hehlerware" and a
//   "Schnäppchen" in the Auslage with badge, struck price and note; buying
//   either curses a card (toast, no Groschen for the curse). wave3-mods.json with a
//   Fusel in the bag: drunk before Skat oder Hand it waits in the head and
//   a toast names the cursed card.
// - undead.json: wave3-mods.json with "Untot" on the Eichel-Daus (31) and
//   the Grün-König (22), both with their Untot look. In an Eichel Hand
//   both carry the skull plaque (can rise); the Daus led comes back into the hand when the trick is
//   complete (toast "Untot") after the sheet "Wähle 3 Karten" took three
//   hand cards (the button waits for three, a fourth is not taken), the
//   last trick shows the substitute with its note "Ersatz" and not the
//   Daus, and the Daus has no plaque any more.
// - Zinken v2 (zinkenV2Pass, undead.json patched): Blei, Goldkante,
//   Fadenkreuz, Schneiderkreide and Winkel (the Grün-Acht with Winkel 2
//   keeps its printed picture under violet sketches, Winkel v2) in the hand, the
//   Doppelgänger on the Grün-König with its mirror picture (and on every
//   other hand card but the Grün-Acht, so one is legal whatever is led);
//   the left Stammgast leads, the player plays last: the Doppelgänger
//   shimmers, "Gespiegelt!" pops up and it lies in the trick as a copy of
//   the highest card (the copied face in its mirror look) and takes the
//   trick (Spiegelbild, 07.11.2026). Mid-animation screenshots on phone
//   and desktop.
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
// Regelkarte opens from the menu and closes, so does the Glossar (each group of
// Zinken with sample cards, the value icons, an entry's info card, no
// sideways scrolling at 390, 375, 360); the Wirt's hints come one at a
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
// The Schnaps-Leiste (schnapsPass, DESIGN 8.9, full motion at 390 × 844,
// 375 × 667, 360 × 740 and 1280 × 800): relics as beer mats only, three
// Schnaps slots, greyed when the core says not now; a Kurzer dragged onto
// the skat shows it, a drop on a wrong target springs back, a tap lifts and
// a tap on the felt pours (Kräuter), the mouth comes up for a Schnaps drunk
// yourself, Doppelkorn's ×2, Blue Curaçao onto a hand card, the Schwenker
// onto the map, and the sheet for a won pub's Schnaps with every slot full.
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
    const measure = () =>
      page.evaluate(() => {
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
    const fits = (m) => m.scroll <= m.inner && m.screen <= 0 && m.declare <= 1 && m.bottom !== null && m.bottom <= m.inner - insets.bottom;
    // On a loaded machine the layout after a resize may take longer than
    // the first wait: measure again a few times before failing.
    let r = await measure();
    for (let i = 0; i < 5 && !fits(r); i++) {
      await page.waitForTimeout(300);
      r = await measure();
    }
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
        // Once the trick sweeps off to its winner (slow tempo, a slow screenshot) the card's own place is its layout box.
        const sweeping = c.getAnimations().some((a) => a.id === 'sweep');
        const dx = sweeping ? Math.abs(c.offsetLeft) + Math.abs(c.offsetTop) : slot ? Math.abs(b.left - slot.left) + Math.abs(b.top - slot.top) : 99;
        return { seat: c.dataset.trickSeat ?? c.dataset.lastSeat, slot: c.closest('.slot')?.dataset.slot, dx };
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
 * (DESIGN 16, M6), the names differ; in a boss pub both seats are the
 * boss's two figures (ADR 0017). A quip, if one shows, stays in
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
  // ADR 0017: both seats of a boss pub are the boss's two figures.
  if (boss && !(left.boss && right.boss)) fail(`${where}: both boss players should sit at the table (${JSON.stringify(r.opps)})`);
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

/**
 * Collect console errors of a page. The account server (/api, ADR 0021) is
 * not part of the smoke test: without it the start screen's daily board
 * request fails (vite preview proxies to a server that is not running),
 * and the game must work as before.
 */
function watch(page, errors) {
  const isApi = (url) => /\/api\//.test(url ?? '');
  page.on('console', (msg) => {
    if (msg.type() === 'error' && !isApi(msg.location()?.url)) errors.push(`console: ${msg.text()}`);
  });
  page.on('pageerror', (err) => errors.push(`page: ${err.message}`));
  page.on('requestfailed', (req) => {
    if (!isApi(req.url())) errors.push(`request failed: ${req.url()}`);
  });
}

/**
 * Note every sound effect started (its file's name, from the buffers the
 * app decodes), in `window.__sounds`.
 */
async function spySounds(context) {
  await context.addInitScript(() => {
    window.__sounds = [];
    const names = new WeakMap();
    const fetchFn = window.fetch;
    window.fetch = async (...args) => {
      const res = await fetchFn(...args);
      const url = String(args[0]);
      if (url.endsWith('.mp3')) {
        const read = res.arrayBuffer.bind(res);
        res.arrayBuffer = async () => {
          const data = await read();
          names.set(data, (url.match(/\/([a-z]+)[-.][^/]*$/) ?? [])[1] ?? url);
          return data;
        };
      }
      return res;
    };
    const decode = BaseAudioContext.prototype.decodeAudioData;
    BaseAudioContext.prototype.decodeAudioData = function (data, ...rest) {
      const name = names.get(data);
      return decode.call(this, data, ...rest).then((buffer) => {
        if (name) names.set(buffer, name);
        return buffer;
      });
    };
    const start = AudioBufferSourceNode.prototype.start;
    AudioBufferSourceNode.prototype.start = function (...args) {
      const name = names.get(this.buffer);
      if (name) window.__sounds.push(name);
      return start.apply(this, args);
    };
  });
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

/**
 * A card picker ("Welche Karte soll es treffen?"): the cards wrap in a
 * grid, nothing scrolls sideways (playtest 04.11.2026).
 */
async function assertPickerGrid(page, label) {
  const m = await page.evaluate(() => {
    const g = document.querySelector('.target-grid');
    if (!g) return null;
    const cards = [...g.querySelectorAll('.target-card')].map((c) => c.getBoundingClientRect());
    const rows = new Set(cards.map((r) => Math.round(r.top))).size;
    const wide = [g, g.closest('.sheet'), document.documentElement].some((e) => e && e.scrollWidth > e.clientWidth + 1);
    const out = cards.some((r) => r.left < 0 || r.right > window.innerWidth);
    return { rows, wide, out };
  });
  if (!m) fail(`${label}: no card grid`);
  if (m.wide || m.out || m.rows < 2) fail(`${label}: the picker does not wrap into a grid without scrolling sideways (${JSON.stringify(m)})`);
}

/** The card before and after a Zinke from a picker, with "Anbringen" and the way back. */
/**
 * Buy the offer of the Auslage in `slot`: since 05.11.2026 a tap opens
 * the sheet with the card before and after (like the card reward); confirm
 * it.
 */
async function buyZinke(page, slot) {
  await page.click(`[data-action="buy-zinke"][data-slot="${slot}"]`);
  await page.waitForSelector('[data-testid="mod-confirm-go"]', { timeout: 5000 }).catch(() => fail(`Auslage: slot ${slot} opens no sheet`));
  await page.click('[data-testid="mod-confirm-go"]');
  await page.waitForSelector('[data-testid="mod-confirm-go"]', { state: 'detached', timeout: 5000 });
}

async function assertApplyConfirm(page, label) {
  await page.waitForSelector('[data-testid="mod-confirm-cards"][data-type="apply"]', { timeout: 5000 }).catch(() => fail(`${label}: a tap on a card does not show it before and after`));
  if ((await page.locator('[data-testid="mod-confirm-cards"] .card').count()) !== 2) fail(`${label}: the confirm does not show two cards`);
  const go = ((await page.locator('[data-testid="mod-confirm-go"]').textContent()) ?? '').trim();
  if (go !== 'Anbringen') fail(`${label}: the confirm button says "${go}"`);
}

/**
 * Einmischen with the tap fallback: one tap on the deck mixes in at once,
 * no second button to confirm (playtest 04.11.2026).
 */
async function mixIn(page) {
  const left = await page.locator('[data-testid="passes-left"]').textContent().catch(() => null);
  await page.click('[data-testid="deck"]');
  if (await page.locator('[data-testid="mix-confirm"]').count()) fail('Einmischen asks for a second tap');
  if (left !== null && /^\d+$/.test(left.trim())) {
    await page
      .waitForFunction((before) => document.querySelector('[data-testid="passes-left"]')?.textContent?.trim() !== before, left.trim(), { timeout: 10000 })
      .catch(() => fail('one tap on the deck did not mix in'));
  }
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

/** A fixture with `change` applied to its parsed save, as JSON again. */
function patched(name, change) {
  const save = JSON.parse(fixture(name));
  change(save);
  return JSON.stringify(save);
}

/**
 * coins-curse.json with its encounter in the form since 03.11.2026:
 * "Umgekipptes Bier", whose random card gets the curse "Ramponiert" (it
 * pays 3 Groschen).
 */
function spilledBeer() {
  return patched('coins-curse.json', (save) => {
    save.run.encounter = {
      id: 'spilled_beer',
      options: [
        { cost: 0, effects: [{ Mark: { card: 'Random', card_mod: 'battered', value: 5 } }] },
        { cost: 4, effects: [{ Mark: { card: 'Chosen', card_mod: 'battered', value: 5 } }] },
        { cost: 2, effects: [] },
      ],
    };
  });
}

/**
 * coins-curse.json with "Die Markensammlerin" open (04.11.2026): a Marke of
 * the player's choice on a card of his choice, bought or traded.
 */
function stampCollector() {
  return patched('coins-curse.json', (save) => {
    save.run.coins = 9;
    save.run.encounter = {
      id: 'stamp_collector',
      options: [
        { cost: 5, effects: [{ Marke: { card: 'Chosen', marke: 'Chosen' } }] },
        { cost: 0, effects: [{ Marke: { card: 'Chosen', marke: 'Chosen' } }, { Mark: { card: 'Random', curse: true } }] },
        { cost: 0, effects: [{ Coins: 1 }] },
      ],
    };
  });
}

/** Open the app with a saved run from e2e/fixtures and resume it. */
async function resumeFixture(page, name, raw = null) {
  await resumeState(page, name, raw ?? fixture(name));
}

/** Open the app with this saved run (`name` for messages) and resume it. */
async function resumeState(page, name, state) {
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

  // Tresen before the boss pub: the preview names the boss and both of his
  // players. The save drew Der Kontra-Hans, removed by ADR 0017: it meets
  // the stand-in, Die Zwillinge (Fritz and Franz).
  await resumeFixture(page, 'boss-tresen.json');
  await page.waitForSelector('[data-testid="shop"]');
  const next = await text(page, 'next-series');
  if (!next.includes('Die Zwillinge') || !next.includes('Fritz und Franz')) fail(`the Tresen does not announce the boss and his players: ${next}`);
  await shoot(page, 'shop-boss');
  await assertNoSideScroll(page, 'Tresen before the boss');
  await expectMusic(page, 'shop', 'Tresen before the boss');
  await page.click('[data-action="leave-shop"]');
  await page.waitForSelector('[data-action="boss-go"]');
  await expectMusic(page, 'boss', 'boss dialog');
  const bossText = (await page.locator('.sheet-boss').textContent()) ?? '';
  if (!bossText.includes('Fritz und Franz')) fail(`boss dialog without both boss players: ${bossText}`);
  if (!bossText.includes('Verlierst du, ist der Abend vorbei')) fail(`boss dialog without the stakes: ${bossText}`);
  if ((await page.locator('.sheet-boss .portrait-boss').count()) !== 2) fail('the boss dialog does not show both boss players');
  const dialogBack = await page.locator('[data-testid="boss-back"]').getAttribute('data-back');
  if (dialogBack !== 'twins') fail(`the boss dialog shows the back ${dialogBack}`);
  log('boss dialog shown with both players and their back');
  await shoot(page, 'boss');
  await page.click('[data-action="boss-go"]');
  await page.waitForSelector('.declare');
  if (!(await page.locator('.opp.is-boss').count())) fail('the boss does not sit at the table');
  if (!(await page.locator('.hud [data-testid="hud-boss"]').count())) fail('the head shows no boss chip at the boss table');
  await expectBacks(page, 'boss pub', 'twins', 'twins');
  await expectMusic(page, 'boss', 'boss series');
  await assertGuests(page, 'boss series', { boss: true });
  await shoot(page, 'table-boss');
  await assertFits(page, 'boss at the table');

  // Kontra said in a save from the Kontra-Hans (removed by ADR 0017): the
  // dialog still shows, the game goes on.
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
 * back ids of seat 1 and 2 (in a boss pub both the boss's), in their small picture.
 */
async function expectBacks(page, label, left, right) {
  const seen = await page.evaluate(() =>
    [1, 2].map((s) => document.querySelector(`.opp[data-seat="${s}"] .opp-hand use`)?.getAttribute('href') ?? null),
  );
  const want = [`#bk-${left}-s`, `#bk-${right}-s`];
  if (seen[0] !== want[0] || seen[1] !== want[1]) fail(`${label}: opponent backs ${JSON.stringify(seen)}, expected ${JSON.stringify(want)}`);
  log(`${label}: backs ${seen.join(', ')}`);
}

/** Portraits of the boss players (ADR 0017): every boss a pair. */
const BOSS_FACES = ['the_pusher', 'grandma', 'twins', 'ramscher', 'black_peter', 'the_old_man'].flatMap((b) => [`p-boss-${b}`, `p-boss-${b}-2`]);

/** The names on both opponents' plates, left first. */
async function oppNames(page) {
  return page.locator('.opp [data-testid="opp-name"]').evaluateAll((els) => els.map((e) => ({ name: e.textContent?.trim() ?? '', plate: e.getAttribute('data-plate') })));
}

/**
 * Bosses v2 (ADR 0017), from saved runs made by the core
 * (regenerate_boss_fixtures): the route names both boss players, the boss
 * table seats both with name plates and the boss back, Die Oma's curses,
 * the relic pick after a won boss pub (a free slot and all slots full) and
 * a lost boss pub that ends the run.
 */
async function bossesV2(browser, errors, reducedMotion) {
  const context = await quietContext(browser, { viewport: PHONE, reducedMotion });
  const page = await context.newPage();
  watch(page, errors);

  // The route to the boss pub: the target card and the boss chip name both.
  await resumeFixture(page, 'boss-route.json');
  await page.waitForSelector('[data-testid="route-next"]');
  const figures = await text(page, 'route-boss-figures');
  if (!figures.includes('Der Schieber und Die Hehlerin')) fail(`route before the boss pub without both players: ${figures}`);
  const chip = await text(page, 'hud-boss-figures');
  if (chip !== 'Der Schieber und Die Hehlerin') fail(`boss chip before the boss pub names ${chip}`);
  log(`route to the boss pub: ${figures}; chip ${chip}`);
  await shoot(page, 'bosses-route');
  await assertRouteFits(page, 'route before the boss pub');
  // The legend names every kind of place on the leg; a tap on a place names it.
  const legend = ((await page.locator('[data-testid="map-legend"]').innerText()) ?? '').replace(/\s+/g, ' ');
  for (const word of ['Du bist hier', 'Festsaal']) if (!legend.includes(word)) fail(`route legend without "${word}": ${legend}`);
  await page.locator('[data-testid="map-target"]').click();
  const named = ((await page.locator('[data-testid="map-label"]').innerText({ timeout: 3000 })) ?? '').replace(/\s+/g, ' ');
  if (!named.includes('Festsaal') || !named.includes('Der Schieber')) fail(`tap on the boss pub on the map names "${named}"`);
  await page.locator('[data-testid="map-badge"]').click();
  const stop = ((await page.locator('[data-testid="map-label"] b').textContent()) ?? '').trim();
  if (!stop || !legend.includes(stop)) fail(`tap on the stop on the map names "${stop}", not in the legend "${legend}"`);
  await page.locator('[data-testid="route-next"]').click();
  if (await page.locator('[data-testid="map-label"]').count()) fail('the name on the map stays after a tap elsewhere');
  log(`route legend: ${legend}; tap names "${named}" and "${stop}"`);

  // At the table of Die Zwillinge: Fritz left, Franz right, both plates, both backs.
  await resumeFixture(page, 'boss-table.json');
  await page.waitForSelector('[data-testid="skat-choice"]');
  const names = await oppNames(page);
  if (names.map((n) => n.name).join(',') !== 'Fritz,Franz' || names.some((n) => n.plate !== 'boss')) fail(`boss table seats ${JSON.stringify(names)}`);
  await assertGuests(page, 'boss table', { boss: true });
  await expectBacks(page, 'Die Zwillinge', 'twins', 'twins');
  const faces = await page.locator('.opp').evaluateAll((els) => els.map((e) => e.getAttribute('data-face')));
  if (faces.join(',') !== 'p-boss-twins,p-boss-twins-2') fail(`boss table faces ${faces.join(',')}`);
  if ((await text(page, 'hud-boss-figures')) !== 'Fritz und Franz') fail('the boss chip at the table does not name both twins');
  await expectMusic(page, 'boss', 'boss table');
  await shoot(page, 'bosses-table');
  await assertFits(page, 'boss table with two players');
  if (!(await page.locator('.rule-strip[data-testid="boss-rule"]').count())) fail('the boss rule is not written out before the first card');
  await assertBossTrick(page);

  // Die Oma's curses lie on half of the player's edition for this pub.
  await resumeFixture(page, 'boss-oma.json');
  await page.waitForSelector('[data-testid="skat-choice"]');
  const oma = await oppNames(page);
  if (oma.map((n) => n.name).join(',') !== 'Die Oma,Der Opa') fail(`Oma table seats ${JSON.stringify(oma)}`);
  const cursed = await page.evaluate(() => JSON.parse(localStorage.getItem('stammtisch.run') ?? '{}').run?.series?.boss_curses?.length ?? 0);
  if (cursed !== 16) fail(`Die Oma cursed ${cursed} cards, expected 16`);
  await shoot(page, 'bosses-oma');
  log('Die Oma and Der Opa at the table, 16 cards cursed');

  // The relic pick: three relics, take one into a free slot.
  await resumeFixture(page, 'boss-reward.json');
  await page.waitForSelector('[data-testid="boss-reward"]');
  const offers = await page.locator('[data-testid="boss-relic-take"]').count();
  if (offers !== 3) fail(`the boss reward offers ${offers} relics`);
  if (await page.locator('[data-testid="boss-reward-full"]').count()) fail('boss reward says the slots are full with a free one');
  await expectMusic(page, 'shop', 'boss reward');
  await shoot(page, 'bosses-reward');
  await assertNoSideScroll(page, 'boss reward', '.boss-reward');
  // 3 × 1 (playtest 05.11.2026): the three mats in one row, at 390 and 360.
  for (const size of [PHONE, { width: 360, height: 740 }]) {
    await page.setViewportSize(size);
    const tops = await page.locator('.boss-reward-offers .coaster').evaluateAll((els) => els.map((e) => Math.round(e.getBoundingClientRect().top)));
    if (new Set(tops).size !== 1) fail(`boss reward at ${size.width}: the relics are not in one row (${tops})`);
  }
  await page.setViewportSize(PHONE);
  const relicsBefore = await page.locator('.hud .coaster-chip').count();
  await page.locator('[data-testid="boss-relic-take"]').first().click();
  await page.waitForSelector('[data-testid="route-next"]');
  const relicsAfter = await page.locator('.hud .coaster-chip').count();
  if (relicsAfter !== relicsBefore + 1) fail(`the relic did not arrive: ${relicsBefore} -> ${relicsAfter} chips`);
  log('boss reward: relic taken, on to the route');

  // Nine relics (an old save from the time of unlimited slots, 8 since
  // 05.11.2026): the "full" note, a pick needs an owned relic to replace
  // (sold for half), the HUD strip keeps the page.
  await resumeFixture(page, 'boss-reward-full.json');
  await page.waitForSelector('[data-testid="boss-reward"]');
  if (!(await page.locator('[data-testid="boss-reward-full"]').count())) fail('boss reward with nine relics does not say the slots are full');
  const manyBefore = await page.locator('.hud .coaster-chip').count();
  await shoot(page, 'bosses-reward-many');
  await assertNoSideScroll(page, 'boss reward, many relics', '.boss-reward');
  await page.locator('[data-testid="boss-relic-take"]').nth(1).click();
  await page.waitForSelector('[data-testid="boss-relic-replace"]');
  await page.locator('[data-testid="boss-relic-replace"] [data-action="boss-relic"]').first().click();
  await page.waitForSelector('[data-testid="route-next"]');
  const manyAfter = await page.locator('.hud .coaster-chip').count();
  if (manyAfter !== manyBefore) fail(`the replace did not keep the count: ${manyBefore} -> ${manyAfter} chips`);
  const pageWide = await page.evaluate(() => document.documentElement.scrollWidth > innerWidth + 1);
  if (pageWide) fail('many relics widen the page');
  log(`boss reward with every slot full: replaced, ${manyBefore} -> ${manyAfter} chips`);
  // Or none at all.
  await resumeFixture(page, 'boss-reward.json');
  await page.waitForSelector('[data-testid="boss-relic-skip"]');
  await page.click('[data-testid="boss-relic-skip"]');
  await page.waitForSelector('[data-testid="route-next"]');

  // The last game of a lost boss pub: the run ends. No hearts in the head (ADR 0019).
  // Der Schieber demands a hand game: the hand Ansage is open at once.
  await resumeFixture(page, 'boss-lost.json');
  await page.waitForSelector('.declare[data-phase="Declaring"]');
  if (await page.locator('[data-testid="lives"], .hud .heart').count()) fail('hearts in the head (ADR 0019: no lives)');
  await page.click('[data-action="pick-game"][data-index="4"]');
  await page.click('[data-action="declare"]');
  await page.waitForSelector('.declare', { state: 'detached' });
  await playOut(page, 'lost boss pub');
  await page.click('[data-action="score-continue"]');
  await page.waitForSelector('[data-testid="end"]');
  const reason = await text(page, 'end-reason');
  if (!reason.includes('Boss-Kneipe') || reason.includes('Leben')) fail(`the end of a lost boss pub says: ${reason}`);
  log(`lost boss pub: ${reason}`);
  await shoot(page, 'bosses-lost');
  await context.close();
}

/**
 * The Faktor (ADR 0018), from factor-win.json (made by the core,
 * regenerate_factor_fixture): Schellenbaum, Zapfhahn after three pubs and
 * Heiße Hand after two wins; a Schellen Hand that takes every trick. The
 * value preview names the factor, the score board shows the third box
 * "Faktor" with ×1,5 × ×1,3 × ×1,3 = 2,535, not greyed; the Glossar legend
 * has five value icons (checked in hintsAndRules).
 */
async function factorPass(browser, errors, reducedMotion) {
  const context = await quietContext(browser, { viewport: PHONE, reducedMotion });
  const page = await context.newPage();
  watch(page, errors);
  await resumeFixture(page, 'factor-win.json');
  await page.waitForSelector('[data-action="play-hand"]');
  await page.click('[data-action="play-hand"]');
  const schellen = page.locator('[data-action="pick-game"]', { has: page.locator('.game-base', { hasText: /^9$/ }) });
  await schellen.click();
  await page.click('[data-action="declare"]');
  await page.waitForSelector('.declare', { state: 'detached' });
  const worth = await text(page, 'value-preview');
  if (!worth.includes('2,535')) fail(`Faktor: the value preview says "${worth}"`);
  await playOut(page, 'Faktor');
  const box = page.locator('[data-testid="score-factor"]');
  const mul = ((await page.locator('[data-num="mul"]').textContent()) ?? '').trim();
  if (mul !== '2,535') fail(`Faktor: the box shows "${mul}"`);
  if (!(await box.count())) fail('Faktor: no Faktor box on ×2,535');
  await shoot(page, 'factor-score');
  log(`Faktor: preview "${worth}", box ${mul}`);
  await context.close();
}

/** Width of the first trick slot at each size, rounded to 0.1 px. */
async function trickWidths(page, sizes) {
  const widths = [];
  for (const size of sizes) {
    await page.setViewportSize(size);
    await page.waitForTimeout(150);
    const w = await page.locator('[data-testid="trick"] .slot').first().evaluate((e) => e.getBoundingClientRect().width);
    widths.push(Math.round(w * 10) / 10);
  }
  await page.setViewportSize(PHONE);
  return widths;
}

/**
 * The boss rule during play (play-test feedback): the strip above the
 * middle gives way to a chip in the felt's top left corner, which opens the
 * rule on a tap, so the trick cards in a boss pub are as big as in a normal
 * pub (within 2 px) at 390 × 844, 1280 × 800 and 375 × 667. The page is on
 * the boss table at Skat oder Hand; a normal pub (seed SEED) is measured in
 * a second page of the same context.
 */
async function assertBossTrick(page) {
  const sizes = [PHONE, DESKTOP, { width: 375, height: 667 }];
  await skatThenGrand(page, 'boss table');
  await page.waitForSelector('[data-testid="trick"]');
  if (await page.locator('.rule-strip').count()) fail('the boss rule strip still stands above the trick during play');
  const chip = page.locator('.trick-side .rule-chip[data-testid="boss-rule"]');
  if (!(await chip.count())) fail('no boss rule chip on the felt during play');
  const boss = await trickWidths(page, sizes);
  await shoot(page, 'bosses-trick');
  await chip.click();
  await page.waitForSelector('[data-testid="info-card"][data-info-key^="rule:"]');
  await page.click('[data-testid="info-card"] [data-info-close]');
  await page.waitForSelector('[data-testid="info-card"]', { state: 'detached' });

  const normal = await page.context().newPage();
  await normal.goto(base);
  await normal.evaluate(() => localStorage.removeItem('stammtisch.run'));
  await normal.reload();
  await normal.waitForSelector('[data-action="new-run"]');
  await normal.fill('#seed', SEED);
  await normal.click('[data-action="new-run"]');
  await skatThenGrand(normal, 'normal pub');
  await normal.waitForSelector('[data-testid="trick"]');
  const plain = await trickWidths(normal, sizes);
  await normal.close();
  sizes.forEach((size, i) => {
    if (Math.abs(boss[i] - plain[i]) > 2) fail(`trick cards at ${size.width} × ${size.height}: ${boss[i]} px in the boss pub, ${plain[i]} px in a normal pub`);
  });
  log(`boss rule as a chip during play; trick card widths boss ${boss.join('/')} px, normal ${plain.join('/')} px`);
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
  // Only the pages scroll: title, tabs and "Schließen" stay in view (playtest 07.11.2026).
  const pinned = await page.evaluate(() => {
    const sheet = document.querySelector('.ledger');
    const body = sheet?.querySelector('.ledger-page');
    if (!sheet || !body) return 'no ledger';
    if (body.scrollHeight <= body.clientHeight) return 'pages do not scroll';
    body.scrollTop = body.scrollHeight;
    const s = sheet.getBoundingClientRect();
    const out = [...sheet.querySelectorAll(':scope > h2, :scope > .ledger-head, :scope > [data-action="close-overlay"]')].filter((el) => {
      const r = el.getBoundingClientRect();
      return r.top < s.top - 1 || r.bottom > s.bottom + 1 || r.bottom > innerHeight;
    });
    return out.length ? `out of view after scrolling: ${out.map((el) => el.className).join(', ')}` : null;
  });
  if (pinned) fail(`Vereinsbuch: ${pinned}`);
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
  // Saved in the form before 03.11.2026: the core opens it anew, three
  // options, one must be taken (no "Weitergehen").
  if ((await page.locator('[data-testid="enc-option"]').count()) !== 3) fail('Der Kartengeber does not show three options');
  if (await page.getByText('Weitergehen', { exact: true }).count()) fail('an encounter still offers "Weitergehen"');
  await page.click('[data-action="choose-encounter"][data-target="Card"]');
  await page.waitForSelector('[data-action="encounter-target"]:not([disabled])');
  await shoot(page, 'encounter-target');
  await assertPickerGrid(page, 'Kartengeber picker');
  // A tap shows the card before and after; "Andere Karte" goes back to the
  // picker, "Anbringen" applies it (playtest 04.11.2026).
  await page.locator('[data-action="encounter-target"]:not([disabled])').first().click();
  await assertApplyConfirm(page, 'Kartengeber');
  await page.click('[data-testid="mod-confirm-back"]');
  await page.waitForSelector('[data-action="encounter-target"]:not([disabled])');
  await page.locator('[data-action="encounter-target"]:not([disabled])').first().click();
  await assertApplyConfirm(page, 'Kartengeber');
  await shoot(page, 'encounter-target-confirm');
  await page.click('[data-testid="mod-confirm-go"]');
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
  await expectBacks(page, 'Der Alte', 'the_old_man', 'the_old_man');
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

  // "Umgekipptes Bier": the curse "Ramponiert" pays 3 Groschen with their
  // source. The old save's encounter is replaced by the new form.
  await resumeState(page, 'coins-curse.json', spilledBeer());
  await page.waitForSelector('[data-testid="encounter"]');
  const coinsBefore = Number(await text(page, 'coins'));
  const curse = page.locator('[data-action="choose-encounter"][data-target="None"]:has([data-card-mod="battered"])');
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

  // Marken from encounters (04.11.2026): "Die Markensammlerin" — pick a
  // Marke on its sheet (art and text), then the card in the picker, then
  // "Zinke anbringen?" with the card before and after.
  await resumeState(page, 'coins-curse.json', stampCollector());
  await page.waitForSelector('[data-testid="encounter"][data-encounter="stamp_collector"]');
  if ((await page.locator('[data-testid="enc-marken"]').count()) < 2) fail('the Marke options do not list their Marken');
  await shoot(page, 'encounter-marke');
  await page.locator('[data-action="choose-encounter"][data-option="0"]').click();
  await page.waitForSelector('[data-testid="marke-options"]');
  if ((await page.locator('[data-testid="marke-options"] [data-action="encounter-marke"]').count()) !== 3) fail('the Marke sheet does not show three Marken');
  await shoot(page, 'encounter-marke-pick');
  await page.click('[data-testid="marke-voucher_coins"]');
  await page.waitForSelector('[data-action="encounter-target"]:not([disabled])');
  await shoot(page, 'encounter-marke-card');
  await page.locator('[data-action="encounter-target"]:not([disabled])').first().click();
  await page.waitForSelector('[data-testid="mod-confirm-cards"]');
  const confirmText = (await page.locator('[data-testid="mod-confirm-text"]').textContent()) ?? '';
  if (!confirmText.includes('Pfandmarke') && !(await page.locator('[data-testid="mod-confirm-cards"] figure').count())) fail(`Marke confirm says: ${confirmText}`);
  await shoot(page, 'encounter-marke-confirm');
  const coinsMarke = Number(await text(page, 'coins'));
  await page.click('[data-testid="mod-confirm-go"]');
  await page.waitForSelector('[data-testid="encounter"]', { state: 'detached' });
  await page.waitForSelector('.declare, [data-action="boss-go"]');
  const coinsMarkeAfter = Number(await text(page, 'coins'));
  if (coinsMarkeAfter !== coinsMarke - 5) fail(`Markensammlerin: Groschen ${coinsMarke} -> ${coinsMarkeAfter}, expected -5`);
  log(`Markensammlerin: Pfandmarke on a chosen card for 5 Groschen (${confirmText.trim()})`);

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
  for (const want of ['Stammfarbe', 'Faktor ×1,5', 'Allrounder']) {
    if (!board.includes(want)) fail(`wave 2a: the Abrechnung does not say "${want}"`);
  }
  const mul = ((await page.locator('[data-num="mul"]').textContent()) ?? '').trim();
  if (mul !== '1,5') fail(`wave 2a: the factor box shows "${mul}"`);
  if (!(await page.locator('[data-testid="score-factor"]').count())) fail('wave 2a: no Faktor box at ×1,5');
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
  // The Leiste shows mats only (03.11.2026): a "1." badge, "1. Spiel" as its title.
  const note = (await page.locator('[data-testid="once-three_discard"]').textContent())?.trim();
  const noteTitle = await page.getAttribute('[data-testid="once-three_discard"]', 'title');
  if (note !== '1.' || noteTitle !== '1. Spiel') fail(`Drei drücken: the chip says "${note}" ("${noteTitle}")`);
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
  for (const must of ['Skat oder Hand', 'Drücken', 'Grundwert × Stufen', 'Schellen', 'Eichel', 'Grand', 'Null', 'Kein Reizen', 'Boss', 'Tresen']) {
    if (!rules.includes(must)) fail(`the Regelkarte lacks "${must}"`);
  }
  await assertMenuFits(page, 'Regelkarte');
  await shoot(page, 'rules');
  await page.click('[data-testid="rules-close"]');
  await page.waitForSelector('[data-testid="rules"]', { state: 'detached' });
  log('Regelkarte opened and closed');

  // The Glossar from the same menu: every group of the frame language with
  // its Zinken (sample card, name and text from the catalog), the value
  // icons on top, no sideways scrolling at 390, 375 and 360; a tap opens the
  // entry's info card; "Schließen" closes it.
  await page.click('[data-action="settings"]');
  await page.click('[data-testid="glossary-open"]');
  await page.waitForSelector('[data-testid="glossary"]');
  if ((await page.locator('[data-testid="glossary-values"] li').count()) !== 5) fail('Glossar: not five value icons (Faktor since ADR 0018)');
  for (const [group, must] of [['Edge', ['Aufdruck', 'Winkel']], ['Moment', ['Eselsohr', 'Kerbe']], ['Voucher', ['Pfandmarke']], ['Special', ['Wandelkarte', 'Untot', 'Doppelgänger']], ['Curse', ['Ramponiert', 'Fleckig']]]) {
    const box = page.locator(`[data-testid="glossary-${group}"]`);
    if (!(await box.count())) fail(`Glossar: no group ${group}`);
    const entries = box.locator('.glossary-entry');
    const n = await entries.count();
    if (!n) fail(`Glossar: group ${group} is empty`);
    if ((await box.locator('.glossary-entry .card-svg').count()) !== n) fail(`Glossar: an entry of ${group} has no card`);
    if (!(await entries.first().isVisible())) {
      await entries.first().scrollIntoViewIfNeeded();
    }
    const text = ((await box.textContent()) ?? '').replace(/\s+/g, ' ');
    for (const name of must) if (!text.includes(name)) fail(`Glossar: ${group} lacks ${name}`);
    if (text.includes('{n}') || text.includes('{+n}')) fail(`Glossar: ${group} shows a raw placeholder`);
  }
  await assertMenuFits(page, 'Glossar');
  for (const size of [PHONE, { width: 375, height: 667 }, { width: 360, height: 740 }]) {
    await page.setViewportSize(size);
    const wide = await page.evaluate(() => ({
      page: document.documentElement.scrollWidth - innerWidth,
      out: [...document.querySelectorAll('.sheet-glossary *')].filter((e) => e.getBoundingClientRect().right > innerWidth + 0.5).length,
    }));
    if (wide.page > 0 || wide.out) fail(`Glossar at ${size.width}x${size.height}: scrolls sideways (${JSON.stringify(wide)})`);
  }
  await page.setViewportSize(PHONE);
  await shoot(page, 'glossary');
  await page.locator('.glossary-entry[data-mod="rank"]').click();
  await page.waitForSelector('[data-testid="info-card"][data-info-key="mod:rank"]');
  await page.click('[data-testid="info-card"] [data-info-close]');
  await page.waitForSelector('[data-testid="info-card"]', { state: 'detached' });
  await page.click('[data-testid="glossary-close"]');
  await page.waitForSelector('[data-testid="glossary"]', { state: 'detached' });
  log('Glossar: every group with its Zinken, the value icons, an info card; fits 390, 375 and 360');

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
  // The card reward of the won pub (DESIGN 5.0), then the route.
  await takeCardReward(page);
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
  // No lives since ADR 0019: nothing on the end screen that reads like them (playtest 05.11.2026).
  if (await page.locator('[data-testid="end"] :is(.tally, .heart, [data-testid="lives"])').count()) fail('the lost end screen shows tally marks or hearts');
  if ((await page.locator('[data-testid="end"] [data-testid="stats-run"] li').count()) !== 6) fail('the lost end screen lacks the six numbers of the run');
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
          const a = document.getAnimations().find((x) => x.id === 'sweep' && x.effect?.target?.classList?.contains('trick-card'));
          if (!a) return null;
          const k = a.effect.getKeyframes();
          const m = /translate\((-?[\d.]+)px, (-?[\d.]+)px\)/.exec(String(k[k.length - 1].transform));
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
        const a = document.getAnimations().find((x) => x.id === 'sweep' && x.effect?.target?.classList?.contains('trick-card'));
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
 * Reward effects with full motion (animation package A), at one viewport:
 * the score board counts the account up while its bar fills, foam crowns
 * the bar when the target is reached and Groschen of the payout fly into
 * the purse (a copy of it above the board counts them in); an encounter's
 * Groschen fly there too; the pub lights go out before the end screen of a
 * lost run (ADR 0019: no lives, no hearts); a won run (the last boss pub,
 * patched from boss-table.json) gets a firework. Buttons give way when pressed and spring back. Every
 * effect ends in the state the core reports, nothing scrolls sideways.
 */
async function rewardFxPass(browser, errors, viewport) {
  const context = await quietContext(browser, { viewport, reducedMotion: 'no-preference' });
  await spySounds(context);
  const page = await context.newPage();
  watch(page, errors);
  const tag = `${viewport.width}x${viewport.height}`;
  const snap = (name) => page.screenshot({ path: join(screens, `${name}-${tag}.png`) });
  const sideways = async (where) => {
    const wide = await page.evaluate(() => document.documentElement.scrollWidth > window.innerWidth + 1);
    if (wide) fail(`reward fx ${tag}: ${where} scrolls sideways`);
  };

  // The first series of SEED: won with the payout.
  await page.goto(base);
  await page.waitForSelector('[data-action="new-run"]');
  await page.fill('#seed', SEED);
  await page.click('[data-action="new-run"]');
  await page.waitForSelector('[data-testid="skat-choice"]');
  await page.waitForTimeout(1200);
  // A button gives way while pressed and springs back on release.
  const deck = await page.locator('[data-action="menu"]').first().boundingBox();
  if (deck) {
    await page.mouse.move(deck.x + deck.width / 2, deck.y + deck.height / 2);
    await page.mouse.down();
    const pressed = await page.evaluate(() => getComputedStyle(document.querySelector('[data-action="menu"]')).scale);
    // Released (a pointerup alone, before the click draws the screen anew).
    const springs = await page.evaluate(() => {
      const btn = document.querySelector('[data-action="menu"]');
      btn.dispatchEvent(new PointerEvent('pointerup', { bubbles: true }));
      return btn.getAnimations().some((a) => a.effect.getKeyframes().some((k) => k.scale));
    });
    await page.mouse.up();
    if (pressed !== '0.95' || !springs) fail(`reward fx ${tag}: a pressed button scales ${pressed}, springs back: ${springs}`);
    await page.waitForSelector('.sheet [data-action="close-overlay"]');
    await page.click('.sheet [data-action="close-overlay"]');
  }
  await mixIn(page);
  await page.waitForTimeout(1200);
  await skatThenGrand(page, `reward fx ${tag}`);
  for (let n = 0; n < 12 && !(await page.locator('.score-sheet').count()); n++) {
    await page.waitForSelector('[data-action="play"], .score-sheet', { timeout: 30000 });
    if (await page.locator('.score-sheet').count()) break;
    await page.locator('[data-action="play"]').first().click();
  }
  await page.waitForSelector('[data-part="account"]:not(.is-pending)', { timeout: 20000 });
  const counting = await page.evaluate(() => Number(document.querySelector('[data-num="account"]')?.textContent));
  await page.waitForSelector('[data-testid="score-meter"][data-bubbling]', { timeout: 5000 }).catch(() => fail(`reward fx ${tag}: no foam on the full account bar`));
  await snap('fx-score-foam');
  await page.waitForSelector('[data-testid="coin-fly"]', { timeout: 5000 }).catch(() => fail(`reward fx ${tag}: no Groschen fly from the payout`));
  await page.waitForTimeout(150);
  await snap('fx-payout-coins');
  await page.waitForSelector('[data-testid="purse-ghost"]', { state: 'detached', timeout: 5000 });
  const paid = await page.evaluate(() => ({
    account: Number(document.querySelector('[data-num="account"]')?.textContent),
    coins: Number(document.querySelector('.hud [data-testid="coins"]')?.textContent),
    payout: Number(document.querySelector('.payout dd:last-of-type b')?.textContent?.trim()),
    full: document.querySelector('[data-testid="score-meter"]')?.classList.contains('is-full'),
  }));
  if (!(counting < paid.account)) fail(`reward fx ${tag}: the account does not count up (${counting} -> ${paid.account})`);
  if (paid.coins !== paid.payout || !paid.full) fail(`reward fx ${tag}: after the payout ${JSON.stringify(paid)}`);
  await sideways('the score board');
  // The Faktor box (ADR 0018) only shows when the game has a factor other than ×1.
  if (!(await page.locator('.fbox-levels').count())) fail(`reward fx ${tag}: no Stufen box on the score board`);
  if ((await page.evaluate(() => document.querySelector('[data-num="mul"]')?.textContent?.trim())) === '1') fail(`reward fx ${tag}: a Faktor box at ×1`);
  log(`reward fx ${tag}: account counts ${counting} -> ${paid.account}, foam, ${paid.coins} Groschen in the purse`);
  await page.click('[data-action="score-continue"]');

  // An encounter's Groschen fly into the purse.
  await resumeState(page, 'coins-curse.json', spilledBeer());
  await page.waitForSelector('[data-testid="encounter"]');
  const before = Number(await text(page, 'coins'));
  await page.click('[data-action="choose-encounter"][data-target="None"]:has([data-card-mod="battered"])');
  await page.waitForSelector('[data-testid="coin-fly"]', { timeout: 5000 }).catch(() => fail(`reward fx ${tag}: no Groschen fly from the encounter`));
  await snap('fx-encounter-coins');
  await page.waitForSelector('[data-testid="purse-ghost"]', { state: 'detached', timeout: 5000 });
  await page.waitForSelector('.declare, [data-action="boss-go"]');
  const after = Number(await text(page, 'coins'));
  if (after !== before + 3) fail(`reward fx ${tag}: Groschen ${before} -> ${after} after the encounter`);
  await sideways('the encounter');

  // A lost run: the lights go out, then the end screen.
  await resumeFixture(page, 'run-end.json');
  await page.waitForSelector('.declare');
  await page.evaluate(() => (window.__sounds = []));
  await skatThenGrand(page, `reward fx ${tag}, lost run`);
  await playOut(page, `reward fx ${tag}, lost run`);
  await page.click('[data-action="score-continue"]');
  await page.waitForSelector('[data-testid="lights-out"]', { timeout: 5000 }).catch(() => fail(`reward fx ${tag}: the lights do not go out`));
  await page.waitForTimeout(700);
  await snap('fx-lights-out');
  await page.waitForSelector('[data-testid="end"]');
  await page.waitForSelector('[data-testid="lights-out"]', { state: 'detached', timeout: 5000 });
  await sideways('the end screen');
  // One loss sound for the lost game and the run together (playtest 04.11.2026).
  await page.waitForTimeout(600);
  const sounds = await page.evaluate(() => window.__sounds);
  const losses = sounds.filter((n) => n === 'lose').length;
  if (sounds.length && losses !== 1) fail(`reward fx ${tag}: the lost run sounds "lose" ${losses} times (${sounds.join(' ')})`);
  if (!sounds.length) log(`reward fx ${tag}: no sounds heard (audio locked?), loss sound not checked`);
  await assertEndButtons(page, `reward fx ${tag}, lost run`);
  log(`reward fx ${tag}: lights go out, end screen with working buttons`);

  // A won run: the last boss pub, its target already reached.
  const won = JSON.parse(fixture('boss-table.json'));
  won.run.series.section = 2;
  won.run.series.target = 1;
  won.run.series.score = 1000;
  won.run.series.games_played = 2;
  await resumeFixture(page, null, JSON.stringify(won));
  await page.waitForSelector('.declare');
  await skatThenGrand(page, `reward fx ${tag}, won run`);
  await playOut(page, `reward fx ${tag}, won run`);
  await page.click('[data-action="score-continue"]');
  await page.waitForSelector('[data-testid="end"] h1.is-won', { timeout: 10000 });
  await page.waitForSelector('[data-testid="fireworks"] .fx-spark', { timeout: 5000 }).catch(() => fail(`reward fx ${tag}: no firework for a won run`));
  await page.waitForTimeout(900);
  await snap('fx-fireworks');
  await page.waitForSelector('[data-testid="fireworks"]', { state: 'detached', timeout: 10000 });
  await sideways('the firework');
  await assertEndButtons(page, `reward fx ${tag}, won run`);
  log(`reward fx ${tag}: firework over the won run`);
  await context.close();
}

/**
 * The buttons of an end screen (playtest 04.11.2026: stuck on a lost one):
 * nothing lies over them once scrolled into sight, and "Zum Start" leads
 * to the start screen.
 */
async function assertEndButtons(page, label) {
  await page.waitForSelector('[data-testid="end"]');
  // An unlock earned with the run is told over the end screen first; its button must work too.
  await page.waitForTimeout(300);
  if (await page.locator('.sheet-unlock').count()) {
    const close = page.locator('.sheet-unlock [data-action="close-overlay"]').last();
    await close.scrollIntoViewIfNeeded();
    await close.click({ timeout: 5000 }).catch(() => fail(`${label}: the unlock over the end screen cannot be closed`));
    await page.waitForSelector('.sheet-unlock', { state: 'detached', timeout: 5000 });
  }
  for (const action of ['new-run', 'to-start']) {
    const btn = page.locator(`[data-testid="end"] [data-action="${action}"]`);
    await btn.scrollIntoViewIfNeeded();
    const hit = await btn.evaluate((b) => {
      const r = b.getBoundingClientRect();
      const top = document.elementFromPoint(r.left + r.width / 2, r.top + r.height / 2);
      return b.contains(top) ? null : (top?.outerHTML ?? 'nothing').slice(0, 120);
    });
    if (hit) fail(`${label}: "${action}" lies under ${hit}`);
  }
  await page.click('[data-testid="end"] [data-action="to-start"]');
  await page.waitForSelector('#seed', { timeout: 5000 }).catch(() => fail(`${label}: "Zum Start" does not lead to the start screen`));
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
/**
 * Drücken with full motion: a skat card tapped on the table is lifted and
 * flies into its slot in the hand. Its place is sampled every frame: it
 * starts up at the table, moves over several frames and comes to rest in
 * the hand, in the place of the core's order (the hand with both skat
 * cards back holds it in the same order), with the other cards in reading
 * order around it. Tapping it again sends it back onto the table, the
 * same way. Leaves the skat on the table as it was.
 */
async function assertPickupFlight(page, tag, snap) {
  const zone = '[data-testid="skat-zone"] .skat-card';
  const [first, second] = await page.locator(zone).evaluateAll((els) => els.map((e) => e.dataset.id));
  if (!first || !second) fail(`feel ${tag}: the skat does not lie on the table`);
  const flight = page.evaluate(
    (id) =>
      new Promise((resolve) => {
        const start = document.querySelector(`[data-testid="skat-zone"] .skat-card[data-id="${id}"]`).getBoundingClientRect();
        document.querySelector(`[data-testid="skat-zone"] .skat-card[data-id="${id}"]`).click();
        const track = [];
        const t0 = performance.now();
        const frame = () => {
          const el = document.querySelector(`.hand-card[data-id="${id}"]`);
          const r = el?.getBoundingClientRect();
          if (r) track.push({ t: Math.round(performance.now() - t0), x: r.left + r.width / 2, y: r.top + r.height / 2, flying: el.getAnimations().some((a) => a.id === 'glide') });
          if (performance.now() - t0 < 1400) requestAnimationFrame(frame);
          else resolve({ start: { x: start.left + start.width / 2, y: start.top + start.height / 2 }, track });
        };
        requestAnimationFrame(frame);
      }),
    first,
  );
  await page.waitForTimeout(160);
  await snap('feel-pickup-flight');
  const { start, track } = await flight;
  const moving = track.filter((p) => p.flying);
  const end = track[track.length - 1];
  if (!end || end.flying) fail(`feel ${tag}: the skat card picked up is still flying after 1.4 s (${JSON.stringify(end)})`);
  if (moving.length < 4) fail(`feel ${tag}: the skat card jumps into the hand (${moving.length} frames in flight)`);
  const way = new Set(moving.map((p) => `${Math.round(p.x)},${Math.round(p.y)}`));
  if (way.size < 4) fail(`feel ${tag}: the skat card does not move over time: ${JSON.stringify(moving)}`);
  const firstSeen = moving[0];
  if (Math.hypot(firstSeen.x - start.x, firstSeen.y - start.y) > Math.hypot(end.x - start.x, end.y - start.y) * 0.35) fail(`feel ${tag}: the flight does not start at the table (${JSON.stringify(start)} -> ${JSON.stringify(firstSeen)})`);
  const hand = await page.evaluate(() =>
    [...document.querySelectorAll('.hand-card')].map((e) => {
      const r = e.getBoundingClientRect();
      return { id: e.dataset.id, x: r.left + r.width / 2, y: r.top + r.height / 2 };
    }),
  );
  const mine = hand.find((c) => c.id === first);
  if (!mine || Math.hypot(mine.x - end.x, mine.y - end.y) > 1) fail(`feel ${tag}: the card picked up does not rest where it landed`);
  // Reading order (row by row, left to right) is the hand's order.
  const rows = [...new Set(hand.map((c) => Math.round(c.y / 20)))].sort((a, b) => a - b);
  const reading = [...hand].sort((a, b) => rows.indexOf(Math.round(a.y / 20)) - rows.indexOf(Math.round(b.y / 20)) || a.x - b.x).map((c) => c.id);
  if (reading.join() !== hand.map((c) => c.id).join()) fail(`feel ${tag}: the hand is not laid out in its order after the flight: ${reading.join()} vs ${hand.map((c) => c.id).join()}`);
  // The other skat card too: the hand then holds all twelve in the core's order.
  await page.locator(`${zone}[data-id="${second}"]`).click();
  await page.waitForFunction(() => !document.getAnimations().some((a) => a.id === 'glide'));
  const twelve = await page.locator('.hand-card').evaluateAll((els) => els.map((e) => e.dataset.id));
  const eleven = hand.map((c) => c.id);
  if (twelve.filter((x) => x !== second).join() !== eleven.join()) fail(`feel ${tag}: the card picked up did not land in its sorted slot (${eleven.join()} within ${twelve.join()})`);
  // Back onto the table, the way it came.
  await page.locator(`.hand-card[data-id="${first}"]`).click();
  await page
    .waitForFunction((id) => document.querySelector(`[data-testid="skat-zone"] .skat-card[data-id="${id}"]`)?.getAnimations().some((a) => a.id === 'glide'), first, { timeout: 3000, polling: 16 })
    .catch(() => fail(`feel ${tag}: a hand card tapped onto the table does not fly there`));
  await page.locator(`.hand-card[data-id="${second}"]`).click();
  await page.waitForFunction(() => !document.getAnimations().some((a) => a.id === 'glide'));
  const back = await page.locator(zone).evaluateAll((els) => els.map((e) => e.dataset.id));
  if (back.join() !== [first, second].join()) fail(`feel ${tag}: the skat is not back on the table (${back.join()})`);
  log(`feel ${tag}: a skat card picked up flies into its slot in the hand (${moving.length} frames, ${way.size} places), the others make room`);
}

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
  await assertPickupFlight(page, tag, snap);
  await page.click('[data-action="discard"]');
  // Drücken: the two cards fly from the table into the Ansage's row.
  await page
    .waitForFunction(() => [...document.querySelectorAll('[data-testid="pressed"] .mini-card')].filter((el) => el.getAnimations().some((a) => a.id === 'glide')).length === 2, null, { timeout: 5000, polling: 16 })
    .catch(() => fail(`feel ${tag}: the pressed cards do not fly into the Ansage`));
  await snap('feel-discard-flight');
  await page.waitForSelector('.declare[data-phase="DeclaringAfterSkat"]');
  await page.waitForFunction(() => !document.getAnimations().some((a) => a.id === 'glide'));
  await page.click('[data-action="pick-game"][data-index="4"]');
  // The Ansage as a sign slammed onto the felt: noted when it comes and goes.
  await page.evaluate(() => {
    const w = window;
    w.__declSign = { seen: 0, text: '', gone: false };
    new MutationObserver(() => {
      const sign = document.querySelector('[data-testid="decl-sign"]');
      if (sign && !w.__declSign.seen) w.__declSign = { seen: performance.now(), text: sign.querySelector('b')?.textContent ?? '', gone: false };
      if (!sign && w.__declSign.seen) w.__declSign.gone = true;
    }).observe(document.body, { childList: true, subtree: true });
  });
  await page.click('[data-action="declare"]');
  // After the Ansage they fly on into the stack in the felt's corner, turning face down.
  await page
    .waitForFunction(() => [...document.querySelectorAll('[data-testid="pressed-play"] .pressed-card')].filter((el) => el.querySelector('.flip-cover') && el.getAnimations().some((a) => a.id === 'glide')).length === 2, null, { timeout: 5000, polling: 16 })
    .catch(() => fail(`feel ${tag}: the pressed cards do not fly into the stack turning face down`));
  await snap('feel-press-flight');
  await page.waitForSelector('[data-action="play"]', { timeout: 30000 });
  await page.waitForFunction(() => !document.getAnimations().some((a) => a.playState === 'running' && a.effect?.target?.closest?.('.trick, .hand')));

  // The sign went again (shrunk into the game line) and never took a tap.
  await page.waitForFunction(() => window.__declSign?.gone, null, { timeout: 8000 }).catch(() => fail(`feel ${tag}: the Ansage sign does not come or does not go`));
  const signText = await page.evaluate(() => window.__declSign.text);
  const lineText = (await page.locator('.gameline .decl').textContent()) ?? '';
  if (!signText.trim() || !lineText.toLowerCase().includes(signText.trim().toLowerCase())) fail(`feel ${tag}: the sign "${signText}" does not name the game of "${lineText}"`);
  // The regulars blink: the frame with shut eyes runs its CSS animation.
  const blinks = await page.evaluate(() => [...document.querySelectorAll('.opp .portrait-blink')].filter((el) => el.getAnimations().some((a) => a.animationName === 'portrait-blink')).length);
  if (blinks !== 2) fail(`feel ${tag}: ${blinks} of 2 opponent portraits blink`);
  log(`feel ${tag}: Ansage sign "${signText.trim()}" came and went, both regulars blink`);

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
  // Display names of the Zinken since 02.11.2026 (DESIGN 8.6).
  const names = ['Aufdruck', 'Winkel', 'Siegel', 'Kerbe', 'Eselsohr', 'Pfandmarke', 'Totenmarke', 'Narrenmarke', 'Durchsichtig', 'Ramponiert', 'Fleckig', 'Wandelkarte', 'Untot'];
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
  const band = { Augen: '#d98e0b', Trumpf: '#6a3fa8', 'Letzter Stich': '#16707a', 'Erster Stich': '#d2552a', Gutschein: '#2f7d6a', Durchsichtig: '#33363b', Ramponiert: '#33363b', Fleckig: '#33363b' };
  expect('Schellen-Zehn', (c) => c.fills.includes(band.Augen) && c.fills.includes('#ffe08a'), 'has no gold Augen frame and band');
  // Winkel since 03.11.2026: the save's Winkel on the Schellen-Unter became Aufdruck (no Winkel on a Unter),
  // Winkel v2 (04.11.2026): its Winkel 2 on the Schellen-Oberin keeps the printed picture and index, with the
  // violet copying pencil's sketches and corner scribble on top.
  expect('Schellen-Unter', (c) => c.label.includes('Aufdruck') && c.fills.includes(band.Augen), 'was not converted to Aufdruck');
  expect('Schellen-Oberin', (c) => c.picture.includes('#f-DQ') && c.picture.includes('#x-DQ') && c.fills.includes('#5b2a86'), 'shows no violet Winkel sketch on the printed Oberin');
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
  // Untot before it rose (playtest 05.11.2026): the printed picture with the green skull band, no zombie face.
  expect('Rot-König', (c) => c.state === 'undead' && c.picture.includes('#f-HK') && !c.picture.includes('#u-HK') && c.fills.includes('#5e7a3c') && c.fills.includes(band.Augen), 'Untot with Augen shows not the printed face, the green band and the gold band');
  expect('Grün-Sieben', (c) => c.state === 'both' && c.picture.includes('#f-S7') && !c.picture.includes('#u-S7') && c.fills.includes('#5e7a3c') && c.fills.includes('#1b1e21'), 'Untot number card with Ramponiert shows not the printed pips, the green band and tears');
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
  // Untot until it rises: the printed Acht with the green band, not the zombie face (playtest 05.11.2026).
  if (!turnedFace.includes('#f-S8') || turnedFace.includes('#u-S8')) fail(`Gutschein Untot: the Grün-Acht did not turn into the Untot look (${turnedFace})`);
  await page.waitForSelector('[data-testid="voucher-redeem"]', { state: 'detached' });
  const after = Number(await text(page, 'coins'));
  if (after !== coins + 3) fail(`Gutschein: Groschen ${coins} -> ${after}, expected +3`);
  await page
    .waitForFunction(() => window.__toasts.some((t) => t.startsWith('Pfandmarke eingelöst')) && window.__toasts.some((t) => t.startsWith('Totenmarke eingelöst')), null, { timeout: 15000 })
    .catch(async () => fail(`Gutschein: no toast per voucher (${(await page.evaluate(() => window.__toasts)).join(' | ')})`));
  log(`Gutschein: margin torn, coin to the counter (${coins} -> ${after}), Grün-Acht turned Untot`);
  // The first Untot of this save: a one-time sheet with the rules, kept in the meta state (DESIGN 7.2).
  await page.waitForSelector('[data-testid="special-hint"] [data-special="undead"]', { timeout: 15000 }).catch(() => fail('Untot: no explanation sheet on the first receipt'));
  if (!(await page.locator('[data-testid="special-hint"] .special-card .card-svg').count())) fail('Untot sheet: the card is not drawn');
  const seen = (await storedMeta(page))?.meta?.specials_seen ?? [];
  if (!seen.includes('undead')) fail(`Untot: the meta state does not remember it (${JSON.stringify(seen)})`);
  await shoot(page, 'special-hint-undead');
  await page.click('[data-testid="special-ok"]');
  await page.waitForSelector('[data-testid="special-hint"]', { state: 'detached' });
  await shoot(page, 'voucher-redeemed');
  await context.close();
}

/**
 * Special cards in the Auslage (special.json: zinken.json without any
 * special card or special voucher, the Auslage set to "wird Untot" on the
 * Eichel-Daus, "wird Wandelkarte" on the Schellen-Zehn with Aufdruck +4 and
 * Aufdruck +3 on the Grün-König). The special tiles carry "Selten", their
 * price and the card with its special look; the Wandelkarte says the
 * Aufdruck stays. Buying each shows its explanation sheet once (the meta
 * state remembers both); a reload brings no sheet back.
 */
async function specialPass(browser, errors, reducedMotion) {
  const context = await quietContext(browser, { viewport: PHONE, reducedMotion });
  const page = await context.newPage();
  watch(page, errors);
  await resumeFixture(page, 'special.json');
  await page.waitForSelector('[data-testid="auslage"]');
  const tiles = await page.locator('[data-testid="zinke"]').evaluateAll((els) =>
    els.map((e) => ({ special: e.dataset.special ?? '', name: e.querySelector('.zinke-name')?.textContent ?? '', rare: e.querySelector('.zinke-rare')?.textContent ?? '', price: e.querySelector('.price-tag')?.textContent ?? '', on: e.querySelector('.zinke-on')?.textContent ?? '', svg: e.querySelector('.card-svg')?.getAttribute('data-special') ?? '' })),
  );
  const [undead, wild, plain] = tiles;
  if (undead?.special !== 'undead' || undead.name !== 'wird Untot' || undead.rare !== 'Selten' || undead.svg !== 'undead') fail(`Auslage: Untot tile wrong ${JSON.stringify(undead)}`);
  if (!undead.price.includes('7')) fail(`Auslage: Untot on the Daus does not cost 7 (${undead.price})`);
  if (wild?.special !== 'wild_card' || wild.svg !== 'wild' || !wild.on.includes('Aufdruck bleibt')) fail(`Auslage: Wandelkarte tile wrong ${JSON.stringify(wild)}`);
  if (plain?.special || plain?.rare) fail('Auslage: a plain Zinke looks special');
  await page.locator('[data-testid="zinken"]').scrollIntoViewIfNeeded();
  await shoot(page, 'special-auslage');
  await buyZinke(page, 0);
  await page.waitForSelector('[data-testid="special-hint"] [data-special="undead"]', { timeout: 10000 }).catch(() => fail('Untot bought: no explanation sheet'));
  await page.click('[data-testid="special-ok"]');
  await page.waitForSelector('[data-testid="special-hint"]', { state: 'detached' });
  await buyZinke(page, 1);
  await page.waitForSelector('[data-testid="special-hint"] [data-special="wild_card"]', { timeout: 10000 }).catch(() => fail('Wandelkarte bought: no explanation sheet'));
  if (await page.locator('[data-testid="special-hint"] [data-special="undead"]').count()) fail('the Untot sheet came twice');
  await shoot(page, 'special-hint-wild');
  await page.click('[data-testid="special-ok"]');
  await page.waitForSelector('[data-testid="special-hint"]', { state: 'detached' });
  const stored = await page.evaluate(() => JSON.parse(localStorage.getItem('stammtisch.run') ?? '{}').run);
  const mods = (id) => stored.edition.cards.find((c) => c.id === id)?.mods ?? [];
  if (!mods(31).includes('undead')) fail(`the Eichel-Daus is not Untot (${mods(31)})`);
  if (mods(3).join(',') !== 'points,wild_card') fail(`the Schellen-Zehn did not keep its Aufdruck next to the Wandelkarte (${mods(3)})`);
  if (stored.coins !== 7) fail(`special offers paid wrong, ${stored.coins} Groschen left`);
  const seen = (await storedMeta(page))?.meta?.specials_seen ?? [];
  if (seen.join(',') !== 'undead,wild_card') fail(`meta state: specials_seen ${JSON.stringify(seen)}`);
  await page.reload();
  await page.click('[data-action="resume"]');
  await page.waitForSelector('[data-testid="auslage"]');
  await page.waitForTimeout(300);
  if (await page.locator('[data-testid="special-hint"]').count()) fail('the explanation sheet came back after a reload');
  log('Spezialkarten: rare Auslage tiles, one explanation sheet each, remembered in the meta state');
  await context.close();
}

/**
 * Eight relic slots (user decision 05.11.2026, DESIGN 8.0) at the Tresen
 * (zinken.json): with one relic the Leiste shows seven free slots as
 * dashed beer mats and the pouch "Deine Relikte (1/8)"; with eight none,
 * every relic offer reads "Tauschen" instead of "Kaufen", and the sheet
 * lists the owned relics with their half price; one replaces the first
 * owned relic (sold for half) with the offer. Nothing scrolls sideways.
 */
async function relicSlotsPass(browser, errors, reducedMotion, viewport) {
  const size = `${viewport.width}x${viewport.height}`;
  const context = await quietContext(browser, { viewport, reducedMotion });
  const page = await context.newPage();
  watch(page, errors);
  const run = () => page.evaluate(() => JSON.parse(localStorage.getItem('stammtisch.run') ?? '{}').run);
  const shot = (name) => page.screenshot({ path: join(screens, `slots-${name}-${size}.png`) });
  const offer = (save) => {
    save.run.shop.relics = ['jinx', 'virgin', 'skat_redraw'];
    save.run.shop.fence = null;
    save.run.coins = 20;
  };
  await resumeFixture(page, 'slots-one.json', patched('zinken.json', offer));
  await page.waitForSelector('[data-testid="relic-offers"]');
  const empty = await page.locator('.leiste-relics [data-testid="relic-slot-empty"]').count();
  if (empty !== 7) fail(`relic slots at ${size}: ${empty} free slots in the Leiste with one relic, expected 7`);
  const pouch = (await page.locator('[data-testid="pouch"] h3').nth(1).textContent())?.trim();
  if (pouch !== 'Deine Relikte (1/8)') fail(`relic slots at ${size}: the pouch says "${pouch}"`);
  if (await page.locator('[data-testid="relic-replace-pick"]').count()) fail(`relic slots at ${size}: "Tauschen" with free slots`);
  await assertNoSideScroll(page, `relic slots ${size}`);
  await shot('free');
  const eight = ['old_lady', 'kibitzer', 'beer_mat', 'tailor', 'lucky_streak', 'blank_king', 'with_four', 'card_counter'];
  await resumeFixture(
    page,
    'slots-full.json',
    patched('zinken.json', (save) => {
      offer(save);
      save.run.relics = eight;
    }),
  );
  await page.waitForSelector('[data-testid="relic-offers"]');
  if (await page.locator('.leiste-relics [data-testid="relic-slot-empty"]').count()) fail(`relic slots at ${size}: free slots shown with eight relics`);
  if ((await page.locator('[data-testid="relic-replace-pick"]').count()) !== 3) fail(`relic slots at ${size}: not every offer reads "Tauschen"`);
  if (await page.locator('[data-action="buy-relic"]').count()) fail(`relic slots at ${size}: a plain "Kaufen" with every slot full`);
  await assertNoSideScroll(page, `relic slots full ${size}`);
  await shot('full');
  await page.click('[data-testid="relic-replace-pick"][data-slot="1"]');
  await page.waitForSelector('[data-testid="relic-replace-list"]');
  const rows = await page.locator('[data-testid="relic-replace"]').count();
  if (rows !== 8) fail(`relic slots at ${size}: the sheet lists ${rows} owned relics`);
  const refund = (await page.locator('[data-testid="relic-replace"][data-replace="0"] .enc-cost').textContent())?.trim();
  if (refund !== '+4 Groschen') fail(`relic slots at ${size}: the Alte Dame sells for "${refund}"`);
  await shot('sheet');
  await page.click('[data-testid="relic-replace"][data-replace="0"]');
  await page
    .waitForFunction(() => JSON.parse(localStorage.getItem('stammtisch.run') ?? '{}').run?.relics?.includes('virgin'), null, { timeout: 5000 })
    .catch(() => fail(`relic slots at ${size}: Jungfrau not taken`));
  const after = await run();
  if (after.relics.length !== 8 || after.relics.includes('old_lady')) fail(`relic slots at ${size}: relics after the swap ${after.relics.join(' ')}`);
  await page.waitForSelector('[data-testid="relic-replace-list"]', { state: 'detached', timeout: 5000 }).catch(() => fail(`relic slots at ${size}: the sheet stays open`));
  await context.close();
  log(`eight relic slots and "Tauschen" at the Tresen at ${size}`);
}

/**
 * Curses as trade (DESIGN 8.8, 03.11.2026), on `viewport`. The Tresen of
 * zinken.json with three relics (Pechvogel as "Hehlerware" in slot 0) and a
 * "Schnäppchen" in the Auslage (slot 1, Aufdruck +3 for 1 instead of 3):
 * the relics lie as a row of three tiles like the Auslage (one row, also
 * at 390 × 844, nothing scrolls sideways), Hehlerware and Schnäppchen wear
 * their badge, the full price struck through and the line that says what
 * the curse is; the info card of the Hehlerware explains the trade. Buying
 * either curses a card (a toast says so); a trade curse pays no Groschen.
 * Then Fusel before Skat oder Hand (wave3-mods.json): the HUD shows it
 * waiting, a toast names the cursed card.
 */
async function tradePass(browser, errors, reducedMotion, viewport) {
  const size = `${viewport.width}x${viewport.height}`;
  const context = await quietContext(browser, { viewport, reducedMotion });
  const page = await context.newPage();
  watch(page, errors);
  const run = () => page.evaluate(() => JSON.parse(localStorage.getItem('stammtisch.run') ?? '{}').run);
  const shot = (name) => page.screenshot({ path: join(screens, `trade-${name}-${size}.png`) });
  const save = JSON.parse(fixture('zinken.json'));
  save.run.shop.relics = ['jinx', 'virgin', 'skat_redraw'];
  save.run.shop.fence = 0;
  save.run.shop.card_offers[1] = { card: 3, card_mod: 'points', value: 3, price: 1, bargain: 2 };
  save.run.coins = 20;
  await resumeFixture(page, 'trade.json', JSON.stringify(save));
  await page.waitForSelector('[data-testid="relic-offers"]');
  const tiles = await page.locator('[data-testid="relic-offer"]').evaluateAll((els) => els.map((e) => ({ ...e.getBoundingClientRect().toJSON(), relic: e.dataset.relic, fence: e.dataset.fence ?? '' })));
  if (tiles.length !== 3) fail(`trade: ${tiles.length} relic tiles, expected 3`);
  if (tiles.map((t) => t.relic).join(' ') !== 'jinx virgin skat_redraw') fail(`trade: relic tiles ${tiles.map((t) => t.relic).join(' ')}`);
  const row = await page.locator('[data-testid="relic-offers"]').evaluate((e) => e.getBoundingClientRect().toJSON());
  for (const t of tiles) {
    if (Math.abs(t.top - tiles[0].top) > 6) fail(`trade: the relic tiles are not in one row at ${size} (${tiles.map((x) => Math.round(x.top)).join(', ')})`);
    if (t.width > row.width / 3 + 1) fail(`trade: a relic tile is wider than a third at ${size}`);
  }
  // Like the Auslage: tiles of the same width.
  const zinke = await page.locator('[data-testid="zinke"]').first().evaluate((e) => e.getBoundingClientRect().width);
  if (Math.abs(zinke - tiles[0].width) > 2) fail(`trade: relic tile ${tiles[0].width} px wide, Auslage tile ${zinke} px`);
  if (tiles[0].fence !== 'true' || tiles[1].fence) fail('trade: Hehlerware not on slot 0 only');
  const fence = page.locator('[data-testid="relic-offer"][data-slot="0"]');
  if ((await fence.locator('[data-testid="fence-badge"]').textContent())?.trim() !== 'Hehlerware') fail('trade: no "Hehlerware" badge');
  const fencePrice = (await fence.locator('.price-tag .price').textContent())?.trim();
  if (fencePrice !== '4 Gr.') fail(`trade: Hehlerware costs "${fencePrice}"`);
  if ((await fence.locator('.price-was').textContent()) !== '9') fail('trade: the full price is not struck through');
  const fenceNote = (await fence.locator('.zinke-note').textContent())?.trim();
  if (fenceNote !== 'Hehlerware: dafür ein Fluch auf einer zufälligen Karte, ohne Groschen') fail(`trade: Hehlerware note says "${fenceNote}"`);
  if (await page.locator('[data-testid="relic-offer"][data-slot="1"] [data-testid="fence-badge"]').count()) fail('trade: a normal relic wears the Hehlerware badge');
  const bargain = page.locator('[data-testid="zinke"][data-slot="1"]');
  if ((await bargain.getAttribute('data-bargain')) !== 'true') fail('trade: the Schnäppchen is not marked');
  if ((await bargain.locator('[data-testid="bargain-badge"]').textContent())?.trim() !== 'Schnäppchen') fail('trade: no "Schnäppchen" badge');
  const bargainNote = (await bargain.locator('.zinke-note').last().textContent())?.trim();
  if (bargainNote !== 'Schnäppchen: dafür ein Fluch auf einer anderen Karte, ohne Groschen') fail(`trade: Schnäppchen note says "${bargainNote}"`);
  if ((await bargain.locator('.price-was').textContent()) !== '2') fail('trade: the Schnäppchen does not strike its full price through');
  if (await page.locator('[data-testid="zinke"][data-slot="0"] [data-testid="bargain-badge"]').count()) fail('trade: a normal offer wears the Schnäppchen badge');
  await assertNoSideScroll(page, `trade Tresen ${size}`);
  await page.locator('[data-testid="relic-offers"]').scrollIntoViewIfNeeded();
  await shot('tresen');
  await page.locator('[data-testid="zinken"]').scrollIntoViewIfNeeded();
  await shot('auslage');
  // The info card of the Hehlerware explains the trade.
  await page.locator('[data-testid="relic-offers"]').scrollIntoViewIfNeeded();
  await fence.locator('.coaster-cover').click();
  await page.waitForSelector('[data-testid="info-card"][data-info-key="offer-relic:0"]', { timeout: 5000 }).catch(() => fail('trade: no info card for the Hehlerware'));
  const info = (await page.locator('[data-testid="info-card"]').textContent()) ?? '';
  if (!info.includes('Hehlerware: Du zahlst den halben Preis')) fail('trade: the info card does not explain the Hehlerware');
  await shot('info');
  const close = page.locator('[data-testid="info-card"] [data-info-close]');
  if (await close.count()) await close.click();
  else await page.keyboard.press('Escape');
  await page.waitForSelector('[data-testid="info-card"]', { state: 'detached', timeout: 5000 }).catch(() => fail('trade: the info card does not close'));
  // Buy the Hehlerware: half price, a curse that pays nothing.
  const cursedBefore = (await run()).edition.cards.filter((c) => c.mods.some((m) => ['transparent', 'battered', 'stained'].includes(m))).length;
  await page.click('[data-action="buy-relic"][data-slot="0"]');
  await page
    .waitForFunction(() => JSON.parse(localStorage.getItem('stammtisch.run') ?? '{}').run?.relics?.includes('jinx'), null, { timeout: 5000 })
    .catch(() => fail('trade: Pechvogel not bought'));
  await page.waitForFunction(() => [...document.querySelectorAll('.toast')].some((t) => t.textContent?.startsWith('Hehlerware:') && t.textContent.includes('verflucht')), null, { timeout: 8000 }).catch(() => fail('trade: no toast for the Hehlerware curse'));
  await shot('fence-bought');
  let after = await run();
  if (after.coins !== 20 - 4) fail(`trade: ${after.coins} Groschen after the Hehlerware, expected 16 (the curse pays nothing)`);
  const cursedAfter = after.edition.cards.filter((c) => c.mods.some((m) => ['transparent', 'battered', 'stained'].includes(m))).length;
  if (cursedAfter !== cursedBefore + 1) fail(`trade: ${cursedAfter - cursedBefore} cards cursed by the Hehlerware`);
  // Buy the Schnäppchen (Aufdruck +3 on the Schellen-Zehn raises +4 → +7: confirm).
  await page.locator('[data-testid="zinken"]').scrollIntoViewIfNeeded();
  await page.click('[data-action="buy-zinke"][data-slot="1"]');
  const confirm = page.locator('[data-testid="mod-confirm-go"]');
  await confirm.waitFor({ timeout: 3000 }).then(() => confirm.click()).catch(() => {});
  await page.waitForFunction(() => [...document.querySelectorAll('.toast')].some((t) => t.textContent?.startsWith('Schnäppchen:')), null, { timeout: 8000 }).catch(() => fail('trade: no toast for the Schnäppchen curse'));
  after = await run();
  if (after.coins !== 16 - 1) fail(`trade: ${after.coins} Groschen after the Schnäppchen, expected 15 (the curse pays nothing)`);
  const cursedLast = after.edition.cards.filter((c) => c.mods.some((m) => ['transparent', 'battered', 'stained'].includes(m))).length;
  if (cursedLast !== cursedAfter + 1) fail('trade: the Schnäppchen cursed no card');
  if (after.edition.cards.find((c) => c.id === 3)?.mod_values?.points !== 7) fail('trade: the Schnäppchen did not raise the Schellen-Zehn');
  await shot('bargain-bought');
  await context.close();

  // Fusel before Skat oder Hand.
  const game = await quietContext(browser, { viewport, reducedMotion });
  const gamePage = await game.newPage();
  watch(gamePage, errors);
  const table = JSON.parse(fixture('wave3-mods.json'));
  table.run.consumables = ['rotgut'];
  await resumeFixture(gamePage, 'fusel.json', JSON.stringify(table));
  await gamePage.waitForSelector('.hud .item-chip[data-slot="0"]');
  if (!(await gamePage.locator('.hud .item-chip[data-slot="0"] svg').count())) fail('Fusel: no motif in the head');
  // A tap lifts a drinkable Schnaps (schnapsPass); "i" opens its info card.
  await gamePage.focus('.hud .item-chip[data-slot="0"]');
  await gamePage.keyboard.press('i');
  await gamePage.waitForSelector('[data-testid="info-card"]');
  const fuselInfo = (await gamePage.locator('[data-testid="info-card"]').textContent()) ?? '';
  if (!fuselInfo.includes('Fusel')) fail('Fusel: the info card does not name it');
  await gamePage.screenshot({ path: join(screens, `trade-fusel-info-${size}.png`) });
  await gamePage.click('[data-action="use-item"][data-slot="0"]');
  await gamePage.waitForSelector('[data-testid="pending-schnaps"]', { timeout: 8000 }).catch(() => fail('Fusel: the HUD does not show it waiting'));
  const waiting = await text(gamePage, 'pending-schnaps');
  if (!waiting.startsWith('Fusel')) fail(`Fusel: the HUD says "${waiting}"`);
  await gamePage
    .waitForFunction(() => [...document.querySelectorAll('.toast')].some((t) => t.textContent?.startsWith('Fusel:') && t.textContent.includes('verflucht')), null, { timeout: 10000 })
    .catch(() => fail('Fusel: no toast names the cursed card'));
  await gamePage.screenshot({ path: join(screens, `trade-fusel-${size}.png`) });
  const drunk = await gamePage.evaluate(() => JSON.parse(localStorage.getItem('stammtisch.run') ?? '{}').run);
  if (drunk.consumables.length !== 0) fail('Fusel: still in the bag');
  if (!drunk.pending_next_game?.some(([id]) => id === 'rotgut')) fail('Fusel: not waiting for the next game');
  await game.close();
  log(`curses as trade at ${size}`);
}

/**
 * Zinken at the Tresen (DESIGN 8.6, zinken.json: frames.json with an
 * Auslage of three set offers, nothing in the bag, 20 Groschen). The
 * Auslage shows three cards drawn with their new Zinke. Aufdruck +3 on the
 * Schellen-König (Siegel) asks "ersetzt Siegel durch Aufdruck" and "Lieber
 * nicht" goes back to the Tresen; on the Schellen-Zehn (Aufdruck +4) it
 * shows the raise "+4 → +7" and applies it. Winkel on the Schellen-Oberin
 * (Rang 2, the maximum) cannot be bought. "Neu auslosen" (the whole Tresen)
 * costs 2, once per visit.
 * The Kartenmaler: pay, pick a card (cursed ones dimmed with "Fluch"), take
 * one of two Zinken; once per Tresen. "Karte reinigen" takes the curse off
 * the Schellen-Sieben; once per Tresen.
 */
async function zinkenPass(browser, errors, reducedMotion) {
  const context = await quietContext(browser, { viewport: PHONE, reducedMotion });
  const page = await context.newPage();
  watch(page, errors);
  const run = () => page.evaluate(() => JSON.parse(localStorage.getItem('stammtisch.run') ?? '{}').run);
  await resumeFixture(page, 'zinken.json');
  await page.waitForSelector('[data-testid="auslage"]');
  if ((await page.locator('[data-testid="zinke"]').count()) !== 3) fail('Auslage: not three offers');
  const names = await page.locator('[data-testid="zinke"] .zinke-name').allTextContents();
  if (names.join(' | ') !== 'Aufdruck +3 | Aufdruck +3 | Winkel +1') fail(`Auslage: offers named ${names.join(' | ')}`);
  if (!(await page.locator('[data-testid="zinke"][data-slot="0"] .card-svg').count())) fail('Auslage: no card face drawn');
  if (!(await page.locator('[data-action="buy-zinke"][data-slot="2"]').isDisabled())) fail('Auslage: Winkel on the Oberin at Rang 2 can be bought');
  await page.locator('[data-testid="zinken"]').scrollIntoViewIfNeeded();
  await shoot(page, 'zinken-auslage');
  await page.click('[data-action="buy-zinke"][data-slot="0"]');
  await page.waitForSelector('[data-testid="mod-confirm-text"]');
  const replace = await text(page, 'mod-confirm-text');
  if (replace !== 'Schellen-König: ersetzt Siegel durch Aufdruck.') fail(`replace confirm: says "${replace}"`);
  if ((await page.locator('[data-testid="mod-confirm-cards"] .card').count()) !== 2) fail('replace confirm: does not show the card now and after');
  await shoot(page, 'zinken-replace');
  await page.click('[data-testid="mod-confirm-back"]');
  await page.waitForSelector('[data-testid="mod-confirm-text"]', { state: 'detached' });
  await page.click('[data-action="buy-zinke"][data-slot="1"]');
  await page.waitForSelector('[data-testid="mod-confirm-text"]');
  const raise = await text(page, 'mod-confirm-text');
  if (raise !== 'Schellen-Zehn: Aufdruck +4 → +7.') fail(`raise confirm: says "${raise}"`);
  await page.click('[data-testid="mod-confirm-go"]');
  await page.waitForSelector('[data-testid="mod-confirm-text"]', { state: 'detached' });
  await page
    .waitForFunction(() => JSON.parse(localStorage.getItem('stammtisch.run') ?? '{}').run?.edition?.cards?.find((c) => c.id === 3)?.mod_values?.points === 7, null, { timeout: 5000 })
    .catch(() => fail('raise: the Schellen-Zehn does not carry Aufdruck +7 afterwards'));
  if ((await run()).coins !== 18) fail(`Auslage: paid wrong, ${(await run()).coins} Groschen left`);
  // Neu auslosen (05.11.2026): the whole Tresen for 2, once per visit.
  if (await page.locator('[data-testid="reroll-zinken"]').count()) fail('Neu auslosen: the Auslage still has its own reroll');
  const rerollText = await text(page, 'reroll');
  if (rerollText !== 'Neu auslosen · 2 Gr.') fail(`Neu auslosen: says "${rerollText}"`);
  await shoot(page, 'zinken-reroll');
  await page.click('[data-testid="reroll"]');
  await page.waitForFunction(() => JSON.parse(localStorage.getItem('stammtisch.run') ?? '{}').run?.coins === 16, null, { timeout: 5000 }).catch(() => fail('Neu auslosen: did not cost 2'));
  await page.waitForFunction(() => document.querySelector('[data-testid="reroll"]')?.textContent === 'Schon neu ausgelost', null, { timeout: 5000 }).catch(() => fail('Neu auslosen: does not say it was used'));
  if (!(await page.locator('[data-testid="reroll"]').isDisabled())) fail('Neu auslosen: a second time possible');
  // A new draw uses run.ron's count (3 again since 05.11.2026).
  if ((await page.locator('[data-testid="zinke"]').count()) !== 3) fail('Neu auslosen: not three offers');
  await shoot(page, 'zinken-rerolled');
  // Kartenmaler.
  await page.click('[data-testid="painter-go"]');
  await page.waitForSelector('[data-testid="zinken-cards"]');
  const reasons = await page.locator('.target-card[data-reason]').evaluateAll((els) => els.map((e) => ({ id: e.dataset.id, reason: e.dataset.reason, tag: e.querySelector('.target-reason')?.textContent ?? '' })));
  for (const id of ['0', '1', '2', '16']) if (!reasons.some((r) => r.id === id && r.reason === 'CardCursed' && r.tag === 'Fluch')) fail(`Kartenmaler: cursed card ${id} not dimmed with "Fluch" (${JSON.stringify(reasons)})`);
  await shoot(page, 'zinken-painter-pick');
  await page.click('.target-card[data-id="31"]');
  await page.waitForSelector('[data-testid="painter-options"]');
  if ((await page.locator('.painter-option').count()) !== 2) fail('Kartenmaler: not two Zinken to choose from');
  await shoot(page, 'zinken-painter');
  if ((await run()).coins !== 10) fail(`Kartenmaler: paid wrong, ${(await run()).coins} Groschen left`);
  const pick = await page.getAttribute('[data-testid="paint-0"]', 'data-card-mod');
  await page.click('[data-testid="paint-0"]');
  await page.waitForSelector('[data-testid="painter-options"]', { state: 'detached' });
  await page
    .waitForFunction((m) => JSON.parse(localStorage.getItem('stammtisch.run') ?? '{}').run?.edition?.cards?.find((c) => c.id === 31)?.mods?.includes(m), pick, { timeout: 5000 })
    .catch(() => fail(`Kartenmaler: the Eichel-Daus does not carry ${pick}`));
  if (!(await page.locator('[data-testid="painter-go"]').isDisabled())) fail('Kartenmaler: can be hired twice at one Tresen');
  // Karte reinigen.
  await page.click('[data-testid="cleanse-go"]');
  await page.waitForSelector('[data-testid="zinken-cards"]');
  await assertPickerGrid(page, 'Karte reinigen');
  await page.click('.target-card[data-id="0"]');
  await page.waitForSelector('[data-testid="mod-confirm-cards"][data-type="cleanse"]', { timeout: 5000 }).catch(() => fail('Karte reinigen: no card before and after'));
  await page.click('[data-testid="mod-confirm-go"]');
  await page
    .waitForFunction(() => JSON.parse(localStorage.getItem('stammtisch.run') ?? '{}').run?.edition?.cards?.find((c) => c.id === 0)?.mods?.length === 0, null, { timeout: 5000 })
    .catch(() => fail('Karte reinigen: the Schellen-Sieben is still cursed'));
  if (!(await page.locator('[data-testid="cleanse-go"]').isDisabled())) fail('Karte reinigen: twice at one Tresen');
  if ((await run()).coins !== 7) fail(`Karte reinigen: paid wrong, ${(await run()).coins} Groschen left`);
  await page.locator('[data-testid="zinken"]').scrollIntoViewIfNeeded();
  await shoot(page, 'zinken-done');
  // "Der Duzfreund": Neu auslosen for free, still once per visit.
  const friend = JSON.parse(fixture('zinken.json'));
  friend.run.relics = [...(friend.run.relics ?? []), 'first_names'];
  await resumeFixture(page, 'zinken.json', JSON.stringify(friend));
  await page.waitForSelector('[data-testid="reroll"]');
  const free = await text(page, 'reroll');
  if (free !== 'Neu auslosen · gratis') fail(`Duzfreund: Neu auslosen says "${free}"`);
  const coins = (await run()).coins;
  await page.click('[data-testid="reroll"]');
  await page.waitForFunction(() => document.querySelector('[data-testid="reroll"]')?.textContent === 'Schon neu ausgelost', null, { timeout: 5000 }).catch(() => fail('Duzfreund: Neu auslosen not used'));
  if ((await run()).coins !== coins) fail('Duzfreund: Neu auslosen was not free');
  log(`Zinken: "${replace}" / "${raise}" / Neu auslosen once / Kartenmaler ${pick} / Karte reinigen / Duzfreund free`);
  await context.close();
}

/**
 * Pub rescue (DESIGN 8.7), from wave3-mods.json changed in place: the last
 * game of a normal pub deep in the minus with Klarer and Eierlikör. The
 * "noch nötig" line says "Nicht mehr zu schaffen" and "Kneipe verlassen"
 * is highlighted; Klarer sets the account to 0 (toast, the line says what
 * is needed now), Eierlikör waits for the next game; giving up asks, then
 * costs a life and opens the route. In a boss pub (boss-lost.json) the
 * question says that giving up ends the run, and it does. The Tresen
 * shows the three new Schnäpse with their motifs.
 */
/**
 * Drink the Schnaps in `slot` by tapping: the tap lifts it, the mouth
 * slides up from below, a tap on the mouth pours it in (DESIGN 8.9).
 */
async function drinkByTap(page, slot) {
  await page.click(`.hud .schnaps-slot[data-slot="${slot}"]`);
  await page.waitForSelector(`.hud .schnaps-slot[data-slot="${slot}"].is-lifted`, { timeout: 5000 }).catch(() => fail(`Schnaps ${slot}: a tap does not lift it`));
  await page.waitForSelector('[data-testid="drink-mouth"].is-up', { timeout: 5000 }).catch(() => fail(`Schnaps ${slot}: no mouth for a Schnaps drunk yourself`));
  await page.waitForTimeout(350);
  const box = await page.locator('[data-testid="drink-mouth"]').boundingBox();
  await page.mouse.click(box.x + box.width / 2, box.y + box.height * 0.55);
}

/** Drag the Schnaps in `slot` with the mouse to (x, y), in small steps, then let go. */
async function dragSchnaps(page, slot, x, y, { hold = null } = {}) {
  // A popover of the mouse resting somewhere (the resume button lies over the deck) goes first.
  await page.mouse.move(2, 2);
  await page.keyboard.press('Escape');
  const s = await page.locator(`.hud .schnaps-slot[data-slot="${slot}"]`).boundingBox();
  const sx = s.x + s.width / 2;
  const sy = s.y + s.height / 2;
  await page.mouse.move(sx, sy);
  await page.mouse.down();
  for (let i = 1; i <= 14; i++) await page.mouse.move(sx + ((x - sx) * i) / 14, sy + ((y - sy) * i) / 14);
  if (hold) await hold();
  await page.mouse.up();
}

/** Centre of the first element matching `selector`. */
async function centreOf(page, selector) {
  const b = await page.locator(selector).first().boundingBox();
  if (!b) fail(`no ${selector} on screen`);
  return { x: b.x + b.width / 2, y: b.y + b.height / 2 };
}

/**
 * The Schnaps-Leiste (DESIGN 8.9, user decisions 03.11.2026), with full
 * motion at 390 × 844, 375 × 667, 360 × 740 and 1280 × 800 (wave3-mods.json
 * before Skat oder Hand, route-station.json on the route):
 * - the head shows the relics as beer mats only, scrolling inside
 *   themselves with nine relics (the page does not scroll sideways), then
 *   three Schnaps slots, empty ones dashed; Schnäpse as bare motifs with a
 *   rarity dot; one the core does not let be drunk now (Wegbier at the
 *   table) is greyed;
 * - a Kurzer dragged onto the skat turns it face up; dropped on the hand
 *   (a wrong target) the glass springs back and the Schnaps stays;
 * - a tap lifts a Schnaps (its targets glow, a banner says what to do), a
 *   second tap puts it back, a tap on the felt pours Kräuter: the cards
 *   are washed away and dealt anew;
 * - a Schnaps drunk yourself (Eierlikör) raises the mouth while dragged;
 *   dropped on it: "Ahh!";
 * - Blue Curaçao onto a hand card opens the suit picker for that card;
 *   Schwenker onto the city map rerolls the ways;
 * - a won pub's Schnaps with every slot full: the sheet shows the new one
 *   and the three held ones; taking it for one, or selling it.
 * Not every check at every size: the smaller phones see the Leiste, the
 * Kurzer and the mouth.
 */
async function schnapsPass(browser, errors) {
  for (const viewport of [PHONE, { width: 375, height: 667 }, { width: 360, height: 740 }, DESKTOP]) {
    const full = viewport.width === PHONE.width || viewport.width === DESKTOP.width;
    const tag = `${viewport.width}x${viewport.height}`;
    const context = await quietContext(browser, { viewport, reducedMotion: 'no-preference' });
    const page = await context.newPage();
    watch(page, errors);
    const shot = (name) => page.screenshot({ path: join(screens, `schnaps-${name}-${tag}.png`) });
    const bag = async () => (await page.evaluate(() => JSON.parse(localStorage.getItem('stammtisch.run') ?? '{}').run)).consumables;
    const table = (items, change = () => {}) =>
      patched('wave3-mods.json', (save) => {
        save.run.consumables = items;
        change(save);
      });

    // The Leiste: nine relics scroll inside, three slots, one greyed.
    const many = ['card_counter', 'skat_god', 'honorary_chair', 'beer_mat', 'tailor', 'kibitzer', 'lucky_streak', 'blank_king', 'with_four'];
    await resumeState(page, 'schnaps leiste', table(['shot', 'road_beer'], (save) => (save.run.relics = many)));
    await page.waitForSelector('[data-testid="skat-choice"]');
    // The deal flies in first; targets are measured where they lie.
    await page.waitForTimeout(1300);
    const slots = await page.locator('[data-testid="schnaps-slots"] .schnaps-slot').count();
    if (slots !== 3) fail(`Leiste at ${tag}: ${slots} Schnaps slots, expected 3`);
    if ((await page.locator('[data-testid="schnaps-slots"] .schnaps-slot.is-empty').count()) !== 1) fail(`Leiste at ${tag}: the free slot is not shown empty`);
    if (await page.locator('.leiste .chip-name').count()) fail(`Leiste at ${tag}: relics still carry names`);
    if ((await page.locator('.leiste .coaster-chip').count()) !== many.length) fail(`Leiste at ${tag}: not every relic in the head`);
    if ((await page.locator('.hud .schnaps-slot[data-slot="0"] svg[data-form="Bare"] .rarity-dot').count()) !== 1) fail(`Leiste at ${tag}: the Kurzer is not a bare motif with a rarity dot`);
    if (!(await page.locator('.hud .schnaps-slot[data-slot="0"].is-ready').count())) fail(`Leiste at ${tag}: the Kurzer is not drinkable before Skat oder Hand`);
    if (!(await page.locator('.hud .schnaps-slot[data-slot="1"].is-off[data-usable="false"]').count())) fail(`Leiste at ${tag}: Wegbier at the table is not greyed`);
    const strip = await page.evaluate(() => {
      const r = document.querySelector('.leiste-relics');
      return { scrolls: r.scrollWidth > r.clientWidth, page: document.scrollingElement.scrollWidth - innerWidth, slots: document.querySelector('.leiste-slots').getBoundingClientRect().right - innerWidth };
    });
    if (!strip.scrolls) fail(`Leiste at ${tag}: nine relics do not scroll inside the strip`);
    if (strip.page > 0 || strip.slots > 0) fail(`Leiste at ${tag}: the page is wider than the screen (${strip.page}, slots ${strip.slots})`);
    await shot('leiste');
    if (viewport.width < 600) await assertFits(page, `Schnaps-Leiste at ${tag}`, [{ width: viewport.width, height: viewport.height }]);
    await page.setViewportSize(viewport);

    // Wrong target: the glass springs back, the Schnaps stays.
    const hand = await centreOf(page, '.hand .hand-card');
    await dragSchnaps(page, 0, hand.x, hand.y, {
      hold: async () => {
        if (!(await page.locator('.skat-pile.drink-target').count())) fail(`Kurzer at ${tag}: the skat does not glow while dragging`);
        if (await page.locator('.hand-card.drink-target').count()) fail(`Kurzer at ${tag}: hand cards glow for the Kurzer`);
        await shot('kurzer-drag');
      },
    });
    await page.waitForSelector('[data-testid="drink-spring"]', { timeout: 2000 }).catch(() => fail(`wrong target at ${tag}: the glass does not spring back`));
    await page.waitForTimeout(600);
    if ((await bag()).length !== 2) fail(`wrong target at ${tag}: the Schnaps was drunk`);
    if (await page.locator('[data-testid="drink-glass"]').count()) fail(`wrong target at ${tag}: the glass is left over`);

    // Kurzer onto the skat: it turns face up.
    const skat = await centreOf(page, '.skat-pile');
    await dragSchnaps(page, 0, skat.x, skat.y);
    await page.waitForSelector('[data-testid="skat-peek"]', { timeout: 6000 }).catch(() => fail(`Kurzer at ${tag}: the skat is not shown after the drop`));
    await page.waitForTimeout(250);
    await shot('kurzer-pour');
    if ((await bag()).length !== 1) fail(`Kurzer at ${tag}: still in the bag`);
    await page.waitForTimeout(1200);
    log(`Schnaps-Leiste at ${tag}: slots, greyed, Kurzer dragged onto the skat, wrong target springs back`);

    // A Schnaps drunk yourself: the mouth comes up while dragging; "Ahh!".
    await resumeState(page, 'eggnog', table(['eggnog']));
    await page.waitForSelector('[data-testid="skat-choice"]');
    // The deal flies in first; targets are measured where they lie.
    await page.waitForTimeout(1300);
    const low = { x: viewport.width / 2, y: viewport.height * 0.82 };
    await dragSchnaps(page, 0, low.x, low.y, {
      hold: async () => {
        await page.waitForSelector('[data-testid="drink-mouth"].is-up', { timeout: 3000 }).catch(() => fail(`Eierlikör at ${tag}: no mouth while dragging`));
        await page.waitForTimeout(300);
        await shot('mouth');
      },
    });
    await page.waitForSelector('[data-testid="drink-word-ahh"]', { timeout: 3000 }).catch(() => fail(`Eierlikör at ${tag}: no "Ahh!"`));
    await page.waitForTimeout(950);
    await shot('mouth-ahh');
    await page.waitForSelector('[data-testid="pending-schnaps"]', { timeout: 6000 }).catch(() => fail(`Eierlikör at ${tag}: not waiting for the next game`));
    if ((await bag()).length !== 0) fail(`Eierlikör at ${tag}: still in the bag`);
    log(`mouth at ${tag}: up while dragging, Eierlikör drunk`);
    if (!full) {
      await context.close();
      continue;
    }

    // Tap alternative: lift, put back, lift, pour onto the felt (Kräuter).
    await resumeState(page, 'herbal', table(['herbal']));
    await page.waitForSelector('[data-testid="skat-choice"]');
    // The deal flies in first; targets are measured where they lie.
    await page.waitForTimeout(1300);
    const handBefore = await page.evaluate(() => [...document.querySelectorAll('.hand .hand-card')].map((c) => c.dataset.id).join(','));
    await page.click('.hud .schnaps-slot[data-slot="0"]');
    await page.waitForSelector('.hud .schnaps-slot.is-lifted');
    if (!(await page.locator('.felt.drink-target').count())) fail(`Kräuter at ${tag}: the felt does not glow`);
    const banner = (await page.locator('[data-testid="drink-banner"]').textContent()) ?? '';
    if (!banner.includes('Tippe auf das Ziel')) fail(`Kräuter at ${tag}: the banner says "${banner}"`);
    await shot('tap-lifted');
    await page.click('.hud .schnaps-slot[data-slot="0"]');
    await page.waitForSelector('.hud .schnaps-slot.is-lifted', { state: 'detached', timeout: 2000 }).catch(() => fail(`Kräuter at ${tag}: a second tap does not put it back`));
    if (await page.locator('.drink-target').count()) fail(`Kräuter at ${tag}: targets still glow after putting it back`);
    await page.click('.hud .schnaps-slot[data-slot="0"]');
    await page.waitForSelector('.hud .schnaps-slot.is-lifted');
    const felt = await centreOf(page, '.felt');
    await page.mouse.click(felt.x, felt.y);
    await page.waitForSelector('[data-testid="wash-wave"]', { timeout: 3000 }).catch(() => fail(`Kräuter at ${tag}: no wave washes the cards away`));
    await page.waitForTimeout(300);
    await shot('kraeuter-wash');
    await page.waitForFunction((before) => [...document.querySelectorAll('.hand .hand-card')].map((c) => c.dataset.id).join(',') !== before, handBefore, { timeout: 6000 }).catch(() => fail(`Kräuter at ${tag}: no new cards`));
    if ((await bag()).length !== 0) fail(`Kräuter at ${tag}: still in the bag`);
    await page.waitForTimeout(1200);
    log(`tap at ${tag}: lift, put back, Kräuter poured onto the felt`);

    // Doppelkorn over the table: "×2" slams.
    await resumeState(page, 'double grain', table(['double_grain']));
    await page.waitForSelector('[data-testid="skat-choice"]');
    // The deal flies in first; targets are measured where they lie.
    await page.waitForTimeout(1300);
    const felt2 = await centreOf(page, '.felt');
    await dragSchnaps(page, 0, felt2.x, felt2.y);
    await page.waitForSelector('[data-testid="x2-sign"]', { timeout: 4000 }).catch(() => fail(`Doppelkorn at ${tag}: no ×2 sign`));
    await page.waitForTimeout(250);
    await shot('doppelkorn');
    await page.waitForTimeout(1500);

    // Blue Curaçao onto a hand card: dyed, then the suit picker for that card.
    await resumeState(page, 'curacao', table(['blue_curacao']));
    await page.waitForSelector('[data-testid="skat-choice"]');
    // The deal flies in first; targets are measured where they lie.
    await page.waitForTimeout(1300);
    const card = page.locator('.hand .hand-card').nth(2);
    const cardId = await card.getAttribute('data-id');
    const cb = await card.boundingBox();
    await dragSchnaps(page, 0, cb.x + cb.width / 2, cb.y + cb.height / 2, {
      hold: async () => {
        const glowing = await page.locator('.hand-card.drink-target').count();
        if (glowing < 1) fail(`Blue Curaçao at ${tag}: no hand card glows`);
        if (await page.locator('.skat-pile.drink-target').count()) fail(`Blue Curaçao at ${tag}: the skat glows`);
      },
    });
    await page.waitForSelector(`[data-testid="chalk-cards"] [data-id="${cardId}"].is-picked`, { timeout: 4000 }).catch(() => fail(`Blue Curaçao at ${tag}: the suit picker does not open for the dropped card`));
    await shot('curacao-suit');
    await page.locator('[data-testid="chalk-suits"] button:not([disabled])').first().click();
    await page.waitForFunction(() => JSON.parse(localStorage.getItem('stammtisch.run') ?? '{}').run.consumables.length === 0, null, { timeout: 6000 }).catch(() => fail(`Blue Curaçao at ${tag}: not drunk after the suit`));
    log(`Doppelkorn ×2 and Blue Curaçao on a card at ${tag}`);

    // Schwenker onto the city map: the ways are drawn anew.
    const station = patched('route-station.json', (save) => (save.run.consumables = ['snifter']));
    await resumeState(page, 'snifter', station);
    await page.waitForSelector('[data-testid="map-svg"]');
    await page.waitForTimeout(800);
    if (!(await page.locator('.run.is-route [data-testid="schnaps-slots"]').count())) fail(`route at ${tag}: no Schnaps slots on the route`);
    const map = await centreOf(page, '.route-map');
    await dragSchnaps(page, 0, map.x, map.y);
    await page.waitForFunction(() => JSON.parse(localStorage.getItem('stammtisch.run') ?? '{}').run.consumables.length === 0, null, { timeout: 6000 }).catch(() => fail(`Schwenker at ${tag}: not drunk`));
    await page.waitForTimeout(350);
    await shot('schwenker');
    await assertNoSideScroll(page, 'route with the Leiste', '.route');
    await page.waitForTimeout(1200);
    log(`Schwenker onto the map at ${tag}`);

    // A won pub's Schnaps with every slot full: take it for one, or sell it.
    const full3 = (save) => {
      save.run.consumables = ['snifter', 'road_beer', 'shot'];
      save.run.schnapps_reward = 'blue_curacao';
    };
    await resumeState(page, 'schnapps reward', patched('route-station.json', full3));
    await page.waitForSelector('[data-testid="schnapps-reward"]', { timeout: 6000 }).catch(() => fail(`reward at ${tag}: no sheet for a full Leiste`));
    if ((await page.locator('[data-testid="schnapps-reward"] [data-action="schnapps-swap"]').count()) !== 3) fail(`reward at ${tag}: not three Schnäpse to give up`);
    if (!(await page.locator('[data-testid="schnapps-reward-new"] svg[data-deckel="blue_curacao"]').count())) fail(`reward at ${tag}: the new Schnaps is not shown`);
    const sell = (await page.locator('[data-testid="schnapps-sell"]').textContent()) ?? '';
    if (!/Verkaufen für \d+ Gr\./.test(sell)) fail(`reward at ${tag}: the sell button says "${sell}"`);
    await page.waitForTimeout(400);
    await shot('reward-full');
    await page.click('[data-testid="schnapps-swap-1"]');
    await page.waitForSelector('[data-testid="schnapps-reward"]', { state: 'detached', timeout: 6000 }).catch(() => fail(`reward at ${tag}: the sheet stays after a swap`));
    const swapped = await bag();
    if (swapped.includes('road_beer') || !swapped.includes('blue_curacao') || swapped.length !== 3) fail(`reward at ${tag}: the bag after the swap is ${swapped.join(', ')}`);
    await resumeState(page, 'schnapps reward sell', patched('route-station.json', full3));
    await page.waitForSelector('[data-testid="schnapps-sell"]');
    const coins = Number(await text(page, 'coins'));
    await page.click('[data-testid="schnapps-sell"]');
    await page.waitForSelector('[data-testid="schnapps-reward"]', { state: 'detached', timeout: 6000 });
    const sold = await page.evaluate(() => JSON.parse(localStorage.getItem('stammtisch.run') ?? '{}').run);
    if (sold.coins <= coins || sold.consumables.includes('blue_curacao')) fail(`reward at ${tag}: selling gave ${coins} -> ${sold.coins} Groschen, bag ${sold.consumables.join(', ')}`);
    log(`reward with a full Leiste at ${tag}: swapped, sold`);
    await context.close();
  }
}

async function rescuePass(browser, errors, reducedMotion, viewport) {
  const tag = viewport.width < 600 ? 'phone' : 'desktop';
  const context = await quietContext(browser, { viewport, reducedMotion });
  const page = await context.newPage();
  watch(page, errors);
  const shot = (name) => page.screenshot({ path: join(screens, `rescue-${name}-${viewport.width}x${viewport.height}.png`) });
  const save = JSON.parse(fixture('wave3-mods.json'));
  save.run.relics = [];
  save.run.series.games_played = 2;
  save.run.series.score = -2000;
  save.run.consumables = ['clear_spirit', 'eggnog'];
  await page.goto(base);
  await page.waitForSelector('[data-action="new-run"]');
  await page.evaluate((s) => localStorage.setItem('stammtisch.run', s), JSON.stringify(save));
  await page.reload();
  await page.click('[data-action="resume"]');
  await page.waitForSelector('[data-testid="needed"]');
  if ((await page.getAttribute('[data-testid="needed"]', 'data-reachable')) !== 'false') fail('rescue: −2000 is called reachable');
  const hopeless = await text(page, 'needed');
  if (!hopeless.startsWith('Nicht mehr zu schaffen')) fail(`rescue: the line says "${hopeless}"`);
  if (!(await page.locator('[data-testid="give-up"].is-urgent').count())) fail('rescue: "Kneipe verlassen" is not highlighted');
  await shot('hopeless');
  if (viewport.width < 600) await assertFits(page, 'rescue, Skat oder Hand with the needed line', [PHONE]);
  // Klarer from the HUD (tapped, then poured into the mouth): account 0, the line tells what is needed.
  await drinkByTap(page, 0);
  await page.waitForFunction(() => document.querySelector('[data-testid="account"]')?.textContent?.trim() === '0', null, { timeout: 5000 }).catch(() => fail('Klarer: the account is not 0'));
  await page.waitForSelector('[data-testid="needed"][data-reachable="true"]');
  const needed = await text(page, 'needed');
  if (!needed.startsWith('Du brauchst noch 33 Punkte')) fail(`Klarer: the line says "${needed}"`);
  await page.waitForSelector('.toast');
  await shot('klarer');
  // Eierlikör: waits for the next game, the HUD says so.
  await drinkByTap(page, 0);
  await page.waitForSelector('[data-testid="pending-schnaps"]');
  const pending = await text(page, 'pending-schnaps');
  if (pending !== 'Eierlikör: gewonnen ×2') fail(`Eierlikör: the HUD says "${pending}"`);
  // Give up: asked first; with the Schutzengel the pub is played again (ADR 0019).
  const angelSave = await page.evaluate(() => JSON.parse(localStorage.getItem('stammtisch.run') ?? '{}'));
  await page.click('[data-testid="give-up"]');
  await page.waitForSelector('[data-testid="give-up-confirm"]');
  if (!(await page.locator('[data-testid="give-up-end"]').count())) fail('give up: the question does not say the run ends');
  await shot('confirm');
  await page.click('[data-testid="give-up-confirm"]');
  await page.waitForSelector('[data-testid="end"]', { timeout: 15000 }).catch(() => fail('give up: the run does not end'));
  await shot('given-up');
  angelSave.run.relics = ['guardian_angel'];
  await page.evaluate((s) => localStorage.setItem('stammtisch.run', s), JSON.stringify(angelSave));
  await page.goto(base);
  await page.click('[data-action="resume"]');
  await page.waitForSelector('[data-testid="give-up"]');
  await page.click('[data-testid="give-up"]');
  await page.waitForSelector('[data-testid="give-up-angel"]');
  await page.click('[data-testid="give-up-confirm"]');
  await page.waitForFunction(() => {
    const run = JSON.parse(localStorage.getItem('stammtisch.run') ?? '{}').run;
    return run && run.series.games_played === 0 && !run.relics.includes('guardian_angel');
  }, null, { timeout: 15000 }).catch(() => fail('Schutzengel: the pub is not played again'));
  if (await page.locator('[data-testid="end"]').count()) fail('Schutzengel: the run ended anyway');
  await shot('second-chance');
  await context.close();

  // Boss pub: giving up ends the run.
  const boss = await quietContext(browser, { viewport, reducedMotion });
  const bossPage = await boss.newPage();
  watch(bossPage, errors);
  await resumeFixture(bossPage, 'boss-lost.json');
  await bossPage.waitForSelector('[data-testid="give-up"]', { timeout: 15000 }).catch(() => fail('boss pub: no "Kneipe verlassen"'));
  await bossPage.click('[data-testid="give-up"]');
  await bossPage.waitForSelector('[data-testid="give-up-boss"]');
  const warning = (await bossPage.locator('[data-testid="give-up-boss"]').textContent())?.trim();
  if (warning !== 'Aufgeben beendet den Run.') fail(`boss pub: the question says "${warning}"`);
  await bossPage.screenshot({ path: join(screens, `rescue-boss-confirm-${viewport.width}x${viewport.height}.png`) });
  await bossPage.click('[data-testid="give-up-confirm"]');
  await bossPage
    .waitForFunction(() => JSON.parse(localStorage.getItem('stammtisch.run') ?? 'null')?.run?.phase !== 'Game', null, { timeout: 15000 })
    .catch(() => fail('boss pub: the run goes on after giving up'));
  await bossPage.waitForTimeout(1500);
  await bossPage.screenshot({ path: join(screens, `rescue-boss-end-${viewport.width}x${viewport.height}.png`) });
  await boss.close();

  // The Tresen with the three new Schnäpse.
  const shop = await quietContext(browser, { viewport, reducedMotion });
  const shopPage = await shop.newPage();
  watch(shopPage, errors);
  const tresen = JSON.parse(fixture('zinken.json'));
  tresen.run.shop.consumables = ['clear_spirit', 'nightcap', 'eggnog'];
  await shopPage.goto(base);
  await shopPage.waitForSelector('[data-action="new-run"]');
  await shopPage.evaluate((s) => localStorage.setItem('stammtisch.run', s), JSON.stringify(tresen));
  await shopPage.reload();
  await shopPage.click('[data-action="resume"]');
  await shopPage.waitForSelector('[data-testid="shop"]');
  const goods = await shopPage.locator('[data-testid="shop"]').textContent();
  for (const name of ['Klarer', 'Absacker', 'Eierlikör']) if (!goods?.includes(name)) fail(`Tresen: ${name} not on offer`);
  await shopPage.getByText('Eierlikör', { exact: true }).last().scrollIntoViewIfNeeded();
  await shopPage.screenshot({ path: join(screens, `rescue-tresen-${viewport.width}x${viewport.height}.png`) });
  if (viewport.width < 600) await assertNoSideScroll(shopPage, 'Tresen with the rescue Schnäpse');
  await shop.close();
  log(`rescue (${tag}): "${hopeless}" / "${needed}" / ${pending} / given up / boss pub ends the run / Tresen`);
}

/** The Werkstatt (route-station.json, the Bahn): the Kartenmaler for free, then on to the pub. */
/**
 * "Weiter" after a won game (playtest 05.11.2026: after winning the first
 * game the board stayed and its "Weiter" did nothing). Full motion, touch,
 * the board left to play out without skipping: a tap on "Weiter" closes it
 * and the next screen works (seed 64 wins the pub in its first game: the
 * unlock, then the card reward takes a tap). Then factor-win.json twice:
 * the second time a render under the board throws once (patched in the
 * page); "Weiter" still closes the board and the game goes on.
 */
async function scoreContinuePass(browser, errors) {
  const context = await quietContext(browser, { viewport: PHONE, hasTouch: true, isMobile: true, reducedMotion: 'no-preference' });
  await context.addInitScript(() => {
    let armed = false;
    try {
      armed = localStorage.getItem('smoke.break-under-board') === '1';
      localStorage.removeItem('smoke.break-under-board');
    } catch {
      // Storage blocked: nothing armed.
    }
    if (!armed) return;
    const qs = Element.prototype.querySelector;
    Element.prototype.querySelector = function (sel) {
      // The Tresen watcher runs at the end of every render, after the layer is drawn.
      if (armed && sel === '.more-hint' && document.querySelector('.overlay-score')) {
        armed = false;
        throw new Error('smoke: a render under the score board breaks');
      }
      return qs.call(this, sel);
    };
  });
  const page = await context.newPage();
  const own = [];
  watch(page, own);
  const tapContinue = async (label) => {
    await page.waitForSelector('[data-action="score-continue"]', { timeout: 30000 });
    const box = await page.locator('[data-action="score-continue"]').boundingBox();
    if (!box) fail(`${label}: "Weiter" has no box`);
    const hit = await page.evaluate(({ x, y }) => document.elementFromPoint(x, y)?.closest('[data-action]')?.getAttribute('data-action') ?? null, { x: box.x + box.width / 2, y: box.y + box.height / 2 });
    if (hit !== 'score-continue') fail(`${label}: something lies over "Weiter" (${hit})`);
    await page.touchscreen.tap(box.x + box.width / 2, box.y + box.height / 2);
    await page.waitForSelector('.score-sheet', { state: 'detached', timeout: 5000 }).catch(() => fail(`${label}: "Weiter" did not close the score board`));
  };
  const playToBoard = async (label) => {
    for (let played = 0; ; played++) {
      await page.waitForSelector('[data-action="play"], .score-sheet', { timeout: 30000 });
      if (await page.locator('.score-sheet').count()) break;
      await page.locator('[data-action="play"]').first().tap();
      if (played > 10) fail(`${label}: played more than ten cards`);
    }
  };

  // Seed 64: the pub is won in its first game.
  await page.goto(base);
  await page.waitForSelector('[data-action="new-run"]');
  await page.fill('#seed', SEED);
  await page.click('[data-action="new-run"]');
  await page.waitForSelector('[data-testid="skat-choice"]');
  await mixIn(page);
  await page.waitForSelector('.declare');
  await pickUpAndDiscard(page, 'Weiter after a win');
  await page.click('[data-action="pick-game"][data-index="4"]');
  await page.click('[data-action="declare"]');
  await page.waitForSelector('.declare', { state: 'detached' });
  await playToBoard('Weiter after a win');
  if (!(await page.locator('#score-title.is-won').count())) fail('Weiter after a win: the first game of seed 64 is not won');
  await tapContinue('Weiter after the first game, pub won');
  await page.waitForSelector('[data-testid="unlock-new"]', { timeout: 10000 });
  await page.tap('.sheet-unlock [data-action="close-overlay"]');
  await page.waitForSelector('[data-testid="card-reward"]');
  await page.locator('[data-action="card-reward-pick"]:not([disabled])').first().tap();
  await page.waitForSelector('[data-testid="mod-confirm-go"]', { timeout: 5000 }).catch(() => fail('Weiter after a win: the card reward takes no tap'));
  log('Weiter after a won first game: board closed by a tap, unlock, card reward answers');

  // A render under the board that throws must not leave a dead "Weiter".
  for (const broken of [false, true]) {
    const label = broken ? 'Weiter with a broken render' : 'Weiter, factor-win.json';
    await page.goto(base);
    await page.waitForSelector('[data-action="new-run"]');
    await page.evaluate(([s, b]) => {
      localStorage.setItem('stammtisch.run', s);
      if (b) localStorage.setItem('smoke.break-under-board', '1');
    }, [fixture('factor-win.json'), broken]);
    await page.reload();
    await page.click('[data-action="resume"]');
    await page.waitForSelector('[data-action="play-hand"]');
    await page.click('[data-action="play-hand"]');
    await page.locator('[data-action="pick-game"]', { has: page.locator('.game-base', { hasText: /^9$/ }) }).click();
    await page.click('[data-action="declare"]');
    await page.waitForSelector('.declare', { state: 'detached' });
    await playToBoard(label);
    await tapContinue(label);
    if (await page.locator('.sheet-unlock [data-action="close-overlay"]').count()) await page.tap('.sheet-unlock [data-action="close-overlay"]');
    await page.waitForSelector('.declare, [data-testid="card-reward"], [data-testid="route-go"], [data-testid="end"]', { timeout: 10000 }).catch(() => fail(`${label}: no next screen after "Weiter"`));
  }
  const unexpected = own.filter((e) => !e.includes('smoke: a render under the score board breaks'));
  if (!own.some((e) => e.includes('smoke: a render under the score board breaks'))) fail('Weiter with a broken render: the render did not break (patch not hit)');
  errors.push(...unexpected);
  log('Weiter: a render that throws under the board leaves no dead button');
  await context.close();
}

/**
 * The card reward after a won pub (DESIGN 5.0; fixed pairs since
 * 05.11.2026), from card-reward.json (made by the core,
 * regenerate_card_reward_fixture): three cards with their Zinke (Winkel on
 * the Schellen-König, Aufdruck on the Grün-Daus that has one, the rare
 * Narrenmarke on the Herz-König), each drawn as it will look. Tapping the
 * Winkel shows the card before and after, "Anderes Angebot" goes back; the
 * Aufdruck rises to +6 for free, then the route. A save of the old shape
 * (card-reward-v1.json, Zinken without cards) loads with three pairs drawn
 * again; "Überspringen" pays 2 Groschen. No side scrolling at 390, 375 and
 * 360.
 */
/** The card reward after a won pub: the first offer that can be taken, "Anbringen". */
async function takeCardReward(page) {
  await page.waitForSelector('[data-testid="card-reward"]');
  await page.locator('[data-action="card-reward-pick"]:not([disabled])').first().click();
  await page.waitForSelector('[data-testid="mod-confirm-go"]');
  await page.click('[data-testid="mod-confirm-go"]');
  await page.waitForSelector('[data-testid="card-reward"]', { state: 'detached', timeout: 15000 });
}

async function cardRewardPass(browser, errors, reducedMotion) {
  const context = await quietContext(browser, { viewport: PHONE, reducedMotion });
  const page = await context.newPage();
  watch(page, errors);
  const stored = () => page.evaluate(() => JSON.parse(localStorage.getItem('stammtisch.run') ?? '{}').run);
  await resumeFixture(page, 'card-reward.json');
  await page.waitForSelector('[data-testid="card-reward"]');
  const offers = await page.locator('[data-testid="card-reward-offers"] .reward-option').count();
  if (offers !== 3) fail(`card reward: ${offers} offers`);
  if ((await page.locator('.reward-option.is-rare').count()) !== 1) fail('card reward: the Narrenmarke has no "Selten" band');
  if ((await page.locator('.reward-option .card .card-svg').count()) !== 3) fail('card reward: not every offer draws its card');
  const on = await page.locator('[data-testid="card-reward-points"] .zinke-on').textContent();
  if (!on?.includes('Grün-Daus') || !on.includes('steigert')) fail(`card reward: the Aufdruck says "${on}"`);
  const skipText = await text(page, 'card-reward-skip');
  if (!skipText.includes('2 Groschen')) fail(`card reward: the skip button says "${skipText}"`);
  await shoot(page, 'card-reward');
  await assertNoSideScroll(page, 'card reward', '.card-reward');
  const before = await stored();
  const ace = before.edition.cards.find((c) => (c.mods ?? []).includes('points'));
  if (!ace) fail('card reward fixture: no Daus with an Aufdruck');
  // Winkel on the bare König: before and after, "Anbringen", "Anderes Angebot".
  await page.click('[data-testid="card-reward-rank"]');
  await assertApplyConfirm(page, 'card reward');
  const back = await text(page, 'mod-confirm-back');
  if (back !== 'Anderes Angebot') fail(`card reward: the way back says "${back}"`);
  if (await page.locator('[data-testid="reward-cards"]').count()) fail('card reward: a card picker opened');
  await shoot(page, 'card-reward-confirm');
  await page.click('[data-testid="mod-confirm-back"]');
  await page.waitForSelector('[data-testid="mod-confirm-cards"]', { state: 'detached' });
  await page.waitForSelector('[data-testid="card-reward"]');
  // The Aufdruck on the Daus that has it: it rises.
  await page.click('[data-testid="card-reward-points"]');
  await page.waitForSelector('[data-testid="mod-confirm-cards"][data-type="raise"]');
  await page.click('[data-testid="mod-confirm-go"]');
  await page.waitForSelector('[data-testid="route-next"]', { timeout: 15000 });
  const after = await stored();
  const raised = after.edition.cards.find((c) => c.id === ace.id);
  if (raised?.mod_values?.points !== 6) fail(`card reward: the Aufdruck did not rise to +6 (${JSON.stringify(raised)})`);
  if (after.coins !== before.coins) fail(`card reward: the Zinke was not free (${before.coins} -> ${after.coins})`);
  log('card reward: Aufdruck raised to +6 for free, on to the route');
  // A save of the old shape: three pairs drawn again.
  await resumeFixture(page, 'card-reward-v1.json');
  await page.waitForSelector('[data-testid="card-reward"]');
  const cards = await page.locator('[data-testid="card-reward-offers"] .reward-option').evaluateAll((els) => els.map((e) => e.dataset.card));
  if (cards.length !== 3 || new Set(cards).size !== 3) fail(`card reward: the old save gives ${JSON.stringify(cards)}`);
  // Or skip it for Groschen.
  await resumeFixture(page, 'card-reward.json');
  await page.waitForSelector('[data-testid="card-reward-skip"]');
  await page.click('[data-testid="card-reward-skip"]');
  await page.waitForSelector('[data-testid="route-next"]', { timeout: 15000 });
  const skipped = await stored();
  if (skipped.coins !== before.coins + 2) fail(`card reward: skipping paid ${skipped.coins - before.coins} Groschen`);
  log('card reward: old save drawn again as pairs, skipped for 2 Groschen');
  await context.close();
}

async function workshopPass(browser, errors, reducedMotion) {
  const context = await quietContext(browser, { viewport: PHONE, reducedMotion });
  const page = await context.newPage();
  watch(page, errors);
  await resumeFixture(page, 'route-station.json');
  await page.waitForSelector('[data-testid="map-svg"]');
  await page.click('[data-action="pick-route"][data-option="1"]');
  await page.click('[data-testid="route-go"]');
  await page.waitForSelector('[data-testid="encounter"][data-encounter="workshop"]', { timeout: 15000 });
  const coins = await page.evaluate(() => JSON.parse(localStorage.getItem('stammtisch.run') ?? '{}').run?.coins);
  await page.click('[data-action="choose-encounter"][data-option="0"]');
  await page.waitForSelector('.target-grid');
  // The Eichel-König carries nothing (the Daus has a Narrenmarke).
  await page.click('.target-card[data-id="30"]');
  await page.waitForSelector('[data-testid="painter-options"]');
  await shoot(page, 'workshop-painter');
  await page.click('[data-testid="paint-1"]');
  if (await page.locator('[data-testid="mod-confirm-go"]').count()) fail('Werkstatt: a bare card asks before the Zinke');
  await page.waitForSelector('[data-testid="encounter"]', { state: 'detached', timeout: 15000 });
  const after = await page.evaluate(() => JSON.parse(localStorage.getItem('stammtisch.run') ?? '{}').run);
  if (after.coins !== coins) fail(`Werkstatt: the Kartenmaler was not free (${coins} -> ${after.coins})`);
  if (!after.edition.cards.find((c) => c.id === 30)?.mods?.length) fail('Werkstatt: the Eichel-König carries nothing');
  log('Werkstatt: Kartenmaler for free, then on to the next pub');
  await context.close();
}

/**
 * Routes and encounters v2 (03.11.2026), from route-station.json with its
 * stops replaced: the map shows the new stops Fundgroschen, Kiosk and
 * Würfelbude with their pixel icons; the Fundgroschen give 2 Groschen; the
 * Kiosk sells one of two cheaper Schnäpse; the Würfelbude takes a stake (and
 * offers only "Zuschauen" without Groschen); "Alter Bekannter" asks which
 * relic. No encounter offers "Weitergehen".
 */
async function encountersV2Pass(browser, errors, reducedMotion) {
  const context = await quietContext(browser, { viewport: PHONE, reducedMotion });
  const page = await context.newPage();
  watch(page, errors);
  const route = (stops, change = () => {}) =>
    patched('route-station.json', (save) => {
      save.run.route.options.forEach((o, i) => {
        o.stops = [stops[i]];
        delete o.spelunke;
      });
      change(save);
    });
  const run = () => page.evaluate(() => JSON.parse(localStorage.getItem('stammtisch.run') ?? '{}').run);
  const stops = [
    { category: 'FoundCoins', encounter: 'found_coins' },
    { category: 'Kiosk' },
    { category: 'DiceStall', encounter: 'dice_stall' },
  ];
  // The route: three new stops, each with its own icon.
  await resumeState(page, 'route (new stops)', route(stops));
  await page.waitForSelector('[data-testid="map-svg"]');
  const categories = await page.locator('[data-testid="route-option"]').evaluateAll((els) => els.map((e) => e.getAttribute('data-category')));
  if (categories.join(',') !== 'FoundCoins,Kiosk,DiceStall') fail(`new stops: ways show ${categories}`);
  for (const icon of ['stop-foundcoins', 'stop-kiosk', 'stop-dicestall']) {
    if (!(await page.locator(`.${icon}`).count())) fail(`new stops: no icon .${icon}`);
  }
  await shoot(page, 'route-new-stops');
  await assertNoSideScroll(page, 'route with new stops', '.route');
  // Fundgroschen: one option, 2 Groschen.
  const coins = (await run()).coins;
  await page.click('[data-action="pick-route"][data-option="0"]');
  await page.click('[data-testid="route-go"]');
  await page.waitForSelector('[data-testid="encounter"][data-category="FoundCoins"]', { timeout: 15000 });
  if ((await page.locator('[data-testid="enc-option"]').count()) !== 1) fail('Fundgroschen: not one option');
  await shoot(page, 'found-coins');
  await page.click('[data-action="choose-encounter"]');
  await page.waitForSelector('[data-testid="encounter"]', { state: 'detached', timeout: 15000 });
  if ((await run()).coins !== coins + 2) fail(`Fundgroschen: ${coins} -> ${(await run()).coins}`);
  log('Fundgroschen: +2 Groschen');

  // The Kiosk: two Schnäpse, one may be bought, cheaper than at the Tresen.
  await resumeState(page, 'kiosk', route([stops[1], stops[1], stops[1]], (save) => (save.run.consumables = [])));
  await page.waitForSelector('[data-testid="map-svg"]');
  await page.click('[data-testid="route-go"]');
  await page.waitForSelector('[data-testid="kiosk"]', { timeout: 15000 });
  const buys = page.locator('[data-testid="kiosk-buy"]');
  if ((await buys.count()) !== 2) fail(`Kiosk: ${await buys.count()} Schnäpse`);
  if (!(await page.locator('.kiosk-was').count())) fail('Kiosk: the Tresen price is not shown');
  if (await page.locator('[data-action="reroll"], [data-testid="zinken"]').count()) fail('Kiosk: Tresen services shown');
  await shoot(page, 'kiosk');
  await assertNoSideScroll(page, 'kiosk', '.shop');
  await buys.first().click();
  await page.waitForFunction(() => JSON.parse(localStorage.getItem('stammtisch.run') ?? '{}').run?.consumables?.length === 1);
  if (await page.locator('[data-testid="kiosk-buy"]:not([disabled])').count()) fail('Kiosk: a second Schnaps can be bought');
  await shoot(page, 'kiosk-bought');
  await page.click('[data-testid="kiosk-leave"]');
  await page.waitForSelector('[data-testid="kiosk"]', { state: 'detached', timeout: 15000 });
  log('Kiosk: one of two Schnäpse bought, then on');

  // The Würfelbude: stakes 2, 4, 6; watching only without Groschen.
  await resumeState(page, 'dice stall', route([stops[2], stops[2], stops[2]], (save) => (save.run.coins = 10)));
  await page.waitForSelector('[data-testid="map-svg"]');
  await page.click('[data-testid="route-go"]');
  await page.waitForSelector('[data-testid="encounter"][data-category="DiceStall"]', { timeout: 15000 });
  if ((await page.locator('[data-testid="enc-option"]').count()) !== 3) fail('Würfelbude: not three stakes');
  if (await page.getByText('Zuschauen', { exact: true }).count()) fail('Würfelbude: "Zuschauen" offered with Groschen');
  await shoot(page, 'dice-stall');
  await page.click('[data-action="choose-encounter"][data-option="0"]');
  await page.waitForSelector('[data-testid="encounter"]', { state: 'detached', timeout: 15000 });
  const thrown = (await run()).coins;
  if (thrown !== 8 && thrown !== 12) fail(`Würfelbude: 10 -> ${thrown} Groschen for a stake of 2`);
  log(`Würfelbude: stake 2, now ${thrown} Groschen`);
  await resumeState(page, 'dice stall (broke)', route([stops[2], stops[2], stops[2]], (save) => (save.run.coins = 1)));
  await page.waitForSelector('[data-testid="map-svg"]');
  await page.click('[data-testid="route-go"]');
  await page.waitForSelector('[data-testid="encounter"][data-category="DiceStall"]', { timeout: 15000 });
  const open = page.locator('[data-testid="enc-option"]:not([disabled])');
  if ((await open.count()) !== 1 || !((await open.textContent()) ?? '').includes('Zuschauen')) fail('Würfelbude without Groschen: not only "Zuschauen"');
  if ((await page.locator('[data-testid="enc-blocked"]').count()) !== 3) fail('Würfelbude without Groschen: the stakes do not say why');
  await shoot(page, 'dice-stall-broke');
  await open.click();
  await page.waitForSelector('[data-testid="encounter"]', { state: 'detached', timeout: 15000 });
  log('Würfelbude without Groschen: only watching');

  // "Alter Bekannter": swap or sell asks which relic; the Schnaps is free.
  const friend = { category: 'Event', encounter: 'old_friend' };
  await resumeState(page, 'old friend', route([friend, friend, friend]));
  await page.waitForSelector('[data-testid="map-svg"]');
  await page.click('[data-testid="route-go"]');
  await page.waitForSelector('[data-testid="encounter"][data-encounter="old_friend"]', { timeout: 15000 });
  await shoot(page, 'encounter-old-friend');
  await assertNoSideScroll(page, 'encounter old friend', '.encounter');
  const before = await run();
  await page.click('[data-action="choose-encounter"][data-option="1"][data-target="Relic"]');
  await page.waitForSelector('[data-testid="enc-relics"]');
  await shoot(page, 'encounter-relic-pick');
  await page.locator('[data-action="encounter-relic"]:not([disabled])').first().click();
  await page.waitForSelector('[data-testid="encounter"]', { state: 'detached', timeout: 15000 });
  const after = await run();
  if (after.relics.length !== before.relics.length - 1 || after.coins <= before.coins) fail(`Alter Bekannter: sold? ${before.relics} -> ${after.relics}, ${before.coins} -> ${after.coins}`);
  log(`Alter Bekannter: relic sold at the full price, ${before.coins} -> ${after.coins} Groschen`);
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
  // Before it rose an Untot card is the printed card with the green band (playtest 05.11.2026).
  if (!(await page.locator('.hand-card[data-id="22"] [data-undead="band"] use[href="#f-SK"]').count())) fail('Untot: the Grün-König is not the printed card with the Untot band');
  if (await page.locator('.hand-card use[href^="#u-"]').count()) fail('Untot: a card shows the zombie face before it rose');
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
  // It climbs back into the hand with a green glow (a still outline with reduced motion).
  await page.evaluate(() => {
    window.__rising = false;
    new MutationObserver(() => {
      if (document.querySelector('.hand-card.is-rising[data-id="31"]')) window.__rising = true;
    }).observe(document.body, { childList: true, subtree: true });
  });
  await page.click('[data-action="play"][data-id="31"]');
  // Since 03.11.2026 the player picks three cards first; one of them goes into the trick.
  await pickRiseCards(page, 'Untot', 'undead-pick');
  await page.waitForFunction(() => window.__toasts.some((t) => t.startsWith('Untot:')), null, { timeout: 30000 });
  await page.waitForFunction(() => window.__rising, null, { timeout: 5000 }).catch(() => fail('Untot: the risen Daus does not glow'));
  await page.waitForSelector('[data-action="play"]', { timeout: 30000 });
  await page.waitForFunction(() => !document.querySelector('.run.is-busy'));
  if (!(await page.locator('.hand-card[data-id="31"]').count())) fail('Untot: the Eichel-Daus did not come back into the hand');
  if (await page.locator('.hand-card[data-id="31"] .plaque-rise').count()) fail('Untot: the risen Daus still says it can rise');
  // Only the card that came back wears the zombie face; the other Untot card keeps its band.
  if (!(await page.locator('.hand-card[data-id="31"] [data-undead="risen"] use[href^="#u-"]').count())) fail('Untot: the risen card shows no zombie face');
  if (!(await page.locator('.hand-card[data-id="22"] [data-undead="band"]').count())) fail('Untot: the Grün-König lost its band');
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

/**
 * "Untot" (03.11.2026): after the trick a sheet asks for three cards of
 * the hand; the button only works with three picked.
 */
async function pickRiseCards(page, label, shot = null) {
  await page.waitForSelector('[data-testid="rise-choice"]', { timeout: 30000 }).catch(() => fail(`${label}: no sheet to pick three cards`));
  const title = await page.locator('.sheet-rise h2').textContent();
  if (!title?.includes('Wähle 3 Karten')) fail(`${label}: the pick sheet says "${title}"`);
  if (!(await page.locator('[data-testid="rise-go"][disabled]').count())) fail(`${label}: the button works before three cards are picked`);
  const cards = page.locator('[data-testid="rise-choice"] .rise-card');
  // A tap toggles only that card: the sheet is not drawn (and does not enter) again.
  await page.evaluate(() => (window.__riseSheet = document.querySelector('.sheet-rise')));
  for (let i = 0; i < 3; i++) await cards.nth(i).click();
  if (!(await page.evaluate(() => window.__riseSheet === document.querySelector('.sheet-rise')))) fail(`${label}: the pick sheet is drawn anew on a tap`);
  if ((await page.locator('[data-testid="rise-choice"] .rise-card.is-picked').count()) !== 3) fail(`${label}: three cards are not marked`);
  await cards.nth(3).click();
  if ((await page.locator('[data-testid="rise-choice"] .rise-card.is-picked').count()) !== 3) fail(`${label}: a fourth card could be picked`);
  if (shot) await shoot(page, shot);
  await page.click('[data-testid="rise-go"]');
  await page.waitForSelector('[data-testid="rise-choice"]', { state: 'detached', timeout: 30000 });
}

/**
 * "Untot" with full motion and the slow tempo (undead.json): the Daus
 * climbs out of the trick back into the hand and glows green for a moment.
 */
async function undeadGlowPass(browser, errors) {
  const context = await browser.newContext({ viewport: PHONE, reducedMotion: 'no-preference' });
  await context.addInitScript(() => {
    if (!localStorage.getItem('stammtisch.settings')) {
      localStorage.setItem('stammtisch.settings', JSON.stringify({ tempo: 'slow', sound: false, thinking: false, hints: false }));
    }
  });
  const page = await context.newPage();
  watch(page, errors);
  await resumeFixture(page, 'undead.json');
  await page.waitForSelector('.declare');
  await page.click('[data-action="play-hand"]');
  await page.waitForSelector('.declare-hand');
  await page.click('[data-action="pick-game"][data-index="0"]');
  if (await page.locator('[data-action="pick-risk"][data-risk="Hand"]').count()) await page.click('[data-action="pick-risk"][data-risk="Hand"]');
  await page.click('[data-action="declare"]');
  await page.waitForSelector('.declare', { state: 'detached' });
  await page.waitForSelector('[data-action="play"][data-id="31"]', { timeout: 30000 });
  await page.click('[data-action="play"][data-id="31"]');
  await page.evaluate(() => {
    window.__morph = { stages: [], before: '', after: '' };
    new MutationObserver(() => {
      const box = document.querySelector('[data-testid="rise-morph"]');
      if (!box) return;
      const st = box.getAttribute('data-stage');
      if (st && !window.__morph.stages.includes(st)) window.__morph.stages.push(st);
      window.__morph.before = box.querySelector('.rise-bit [data-undead]')?.getAttribute('data-undead') ?? window.__morph.before;
      window.__morph.after = box.querySelector('.rise-new [data-undead]')?.getAttribute('data-undead') ?? window.__morph.after;
    }).observe(document.body, { childList: true, subtree: true, attributes: true, attributeFilter: ['data-stage'] });
  });
  await pickRiseCards(page, 'Untot glow');
  // Out of the way, so no info card opens over the hand.
  await page.mouse.move(380, 20);
  // The played card crumbles over the table into the zombie card it comes back as.
  await page.waitForSelector('[data-testid="rise-morph"][data-stage="crumble"]', { timeout: 30000 }).catch(() => fail('Untot: the played card does not crumble'));
  await page.waitForTimeout(250);
  await page.screenshot({ path: join(screens, 'undead-crumble-390x844.png') });
  await page.waitForSelector('[data-testid="rise-morph"]', { state: 'detached', timeout: 15000 });
  const morph = await page.evaluate(() => window.__morph);
  if (morph.before !== 'band' || morph.after !== 'risen' || !morph.stages.includes('after')) fail(`Untot: the crumble is not band to zombie (${JSON.stringify(morph)})`);
  log(`Untot: crumbled ${morph.stages.join(' > ')}, ${morph.before} to ${morph.after}`);
  await page.waitForSelector('.hand-card.is-rising[data-id="31"]', { state: 'attached', timeout: 30000 }).catch(() => fail('Untot: no glow with full motion'));
  await page.waitForTimeout(380);
  await page.screenshot({ path: join(screens, 'undead-rising-390x844.png') });
  log('screenshot undead-rising');
  await context.close();
}

/**
 * Zinken v2 (03.11.2026) on undead.json: the new Zinken on hand cards
 * (Blei, Goldkante, Fadenkreuz, Schneiderkreide, Winkel 1 and 2 with the
 * black scribble) and the Doppelgänger on the Grün-König (22), in place
 * of its Untot, and beside the Zinken on every other hand card but the
 * Grün-Acht (17), so that the player holds a legal Doppelgänger whatever
 * is led. The left Stammgast (seat 1) leads: the player plays last.
 */
function zinkenV2Save() {
  return patched('undead.json', (save) => {
    const put = (id, mods, values = {}) => {
      const card = save.run.edition.cards.find((c) => c.id === id);
      card.mods = mods;
      card.mod_values = values;
    };
    put(16, ['lead']);
    put(17, ['rank'], { rank: 2 });
    put(2, ['rank'], { rank: 1 });
    put(19, ['gilt_edge'], { gilt_edge: 1 });
    put(23, ['crosshair'], { crosshair: 1 });
    put(27, ['tailors_chalk'], { tailors_chalk: 1 });
    put(22, ['doppelganger']);
    for (const id of [2, 15, 16, 19, 23, 27, 28, 31]) {
      const card = save.run.edition.cards.find((c) => c.id === id);
      card.mods = ['doppelganger', ...card.mods];
    }
    const game = save.run.game.game;
    game.forehand = 1;
    game.trick_lead = 1;
    for (const ev of game.log) if (ev.Dealt) ev.Dealt.forehand = 1;
  });
}

/** Declare Eichel Hand on the undead.json deal. */
async function declareClubsHand(page) {
  await page.waitForSelector('.declare');
  await page.click('[data-action="play-hand"]');
  await page.waitForSelector('.declare-hand');
  await page.click('[data-action="pick-game"][data-index="0"]');
  if (await page.locator('[data-action="pick-risk"][data-risk="Hand"]').count()) await page.click('[data-action="pick-risk"][data-risk="Hand"]');
  await page.click('[data-action="declare"]');
  await page.waitForSelector('.declare', { state: 'detached' });
}

/**
 * Zinken v2: the new looks in the hand and the Doppelgänger's mirror
 * scene with full motion and the slow tempo: played as the last card it
 * shimmers, "Gespiegelt!", and it lies in the trick as the copy of the
 * highest card, with the mirror look, and takes the trick. Mid-animation
 * screenshots per viewport.
 */
async function zinkenV2Pass(browser, errors, viewport) {
  const size = `${viewport.width}x${viewport.height}`;
  const context = await browser.newContext({ viewport, reducedMotion: 'no-preference' });
  await context.addInitScript(() => {
    if (!localStorage.getItem('stammtisch.settings')) {
      localStorage.setItem('stammtisch.settings', JSON.stringify({ tempo: 'slow', sound: false, thinking: false, hints: false }));
    }
  });
  const page = await context.newPage();
  watch(page, errors);
  await resumeState(page, 'zinken v2', zinkenV2Save());
  await page.waitForSelector('.declare');
  const looks = {
    16: 'lead',
    19: 'gilt_edge',
    23: 'crosshair',
    27: 'tailors_chalk',
  };
  for (const [id, mod] of Object.entries(looks)) {
    if (!(await page.locator(`.hand-card[data-id="${id}"] svg[aria-label*="${mod === 'lead' ? 'Blei' : mod === 'gilt_edge' ? 'Goldkante' : mod === 'crosshair' ? 'Fadenkreuz' : 'Schneiderkreide'}"]`).count())) fail(`Zinken v2: card ${id} does not carry ${mod}`);
  }
  // Winkel v2 on the Grün-Acht (Winkel 2): the printed Acht stays, sketched over in violet, the Acht scribbled out.
  if (!(await page.locator('.hand-card[data-id="17"] use.face-pic[href="#f-S8"]').count())) fail('Winkel: the Grün-Acht does not keep its printed picture');
  if (!(await page.locator('.hand-card[data-id="17"] path[fill="#5b2a86"]').count())) fail('Winkel: the Grün-Acht shows no violet sketch');
  if (!(await page.locator('.hand-card[data-id="17"] use.face-idx[href="#x-S8"]').count())) fail('Winkel: the printed index is not the Acht');
  if (!(await page.locator('.hand-card[data-id="22"] svg[data-special="doppelganger"] use.face-pic[href="#d-SK"]').count())) fail('Doppelgänger: the Grün-König shows no mirror picture');
  await page.waitForFunction(() => !document.querySelector('.run.is-busy'), null, { timeout: 30000 });
  await page.waitForTimeout(1500);
  await page.screenshot({ path: join(screens, `zinken-v2-hand-${size}.png`) });
  await page.locator('.hand').screenshot({ path: join(screens, `zinken-v2-cards-${size}.png`) });
  await declareClubsHand(page);
  // The two Stammgäste play first; then a legal Doppelgänger (not the Grün-Acht).
  await page.waitForFunction(() => document.querySelectorAll('[data-trick-seat="1"], [data-trick-seat="2"]').length === 2 && document.querySelector('.hand-card[data-action="play"]:not([data-id="17"])'), null, { timeout: 30000 }).catch(() => fail('Doppelgänger: the player is not last in the first trick'));
  const id = await page.locator('.hand-card[data-action="play"]:not([data-id="17"])').first().getAttribute('data-id');
  await page.evaluate(() => {
    window.__doppel = { word: false, shimmer: false, copy: null };
    new MutationObserver(() => {
      const d = window.__doppel;
      if (document.querySelector('[data-testid="doppel-word"]')) d.word = true;
      if (document.querySelector('[data-testid="doppel-shimmer"]')) d.shimmer = true;
      const copy = document.querySelector('[data-trick-seat="0"] svg[data-special="doppelganger"], [data-last-seat="0"] svg[data-special="doppelganger"]');
      if (copy && !d.copy) d.copy = copy.querySelector('use.face-pic')?.getAttribute('href') ?? 'no picture';
    }).observe(document.body, { childList: true, subtree: true, attributes: true });
  });
  await page.click(`.hand-card[data-action="play"][data-id="${id}"]`);
  await page.mouse.move(viewport.width - 10, 10);
  await page.waitForSelector('[data-testid="doppel-shimmer"]', { state: 'attached', timeout: 15000 }).catch(() => fail('Doppelgänger: no mirror shimmer'));
  await page.waitForTimeout(250);
  await page.screenshot({ path: join(screens, `doppel-shimmer-${size}.png`) });
  await page.waitForSelector('[data-testid="doppel-word"]', { state: 'attached', timeout: 15000 }).catch(() => fail('Doppelgänger: no "Gespiegelt!"'));
  const word = await page.locator('[data-testid="doppel-word"]').first().textContent().catch(() => '');
  if (word && word !== 'Gespiegelt!') fail(`Doppelgänger: the word says "${word}"`);
  await page.waitForSelector('[data-trick-seat="0"] svg[data-special="doppelganger"], [data-last-seat="0"] svg[data-special="doppelganger"]', { state: 'attached', timeout: 15000 }).catch(() => fail('Doppelgänger: no mirrored copy in the trick'));
  await page.waitForTimeout(200);
  await page.screenshot({ path: join(screens, `doppel-copy-${size}.png`) });
  await page.waitForFunction(() => !document.querySelector('.run.is-busy'), null, { timeout: 30000 });
  const seen = await page.evaluate(() => window.__doppel);
  if (!seen.shimmer || !seen.word || !seen.copy) fail(`Doppelgänger: shimmer, word and copy must show (${JSON.stringify(seen)})`);
  if (await page.locator(`.hand-card[data-id="${id}"]`).count()) fail('Doppelgänger: the card is still in the hand');
  // It took the trick: the player leads the next one.
  await page.waitForSelector('.hand-card[data-action="play"]', { timeout: 30000 }).catch(() => fail('Doppelgänger: the player does not lead after the mirror'));
  if (await page.locator('[data-trick-seat="1"], [data-trick-seat="2"]').count()) fail('Doppelgänger: a Stammgast took the trick');
  log(`Doppelgänger: ${JSON.stringify(seen)}`);
  await context.close();
}

/** A Wandelkarte settled into Grün (the wave 3 fixture's Rot-Daus): the fool in green, the normal Grün index. */
async function assertSettledWild(svg, where) {
  if ((await svg.getAttribute('data-wild')) !== 'settled') fail(`the Wandelkarte ${where} is not shown settled`);
  if ((await svg.getAttribute('data-suit')) !== 'Spades') fail(`the settled Wandelkarte ${where} is not tinted as Grün`);
  const pic = await svg.locator('use.face-pic').getAttribute('href');
  const idx = await svg.locator('use.face-idx').getAttribute('href');
  if (pic !== '#ws-SA' || idx !== '#x-SA') fail(`the settled Wandelkarte ${where} shows ${pic} / ${idx}`);
}

/**
 * "Wandelkarte" with motion on: as the game starts its colours run through
 * the four suits and stop on Grün; a frame from the middle of it.
 */
async function wildSettlePass(browser, errors) {
  const context = await quietContext(browser, { viewport: PHONE, reducedMotion: 'no-preference' });
  const page = await context.newPage();
  watch(page, errors);
  await resumeFixture(page, 'wave3-mods.json');
  await page.waitForSelector('.declare');
  await page.click('[data-action="play-hand"]');
  await page.waitForSelector('.declare-hand');
  await page.click('[data-action="pick-game"][data-index="0"]');
  if (await page.locator('[data-action="pick-risk"][data-risk="Hand"]').count()) await page.click('[data-action="pick-risk"][data-risk="Hand"]');
  // Note every face the card shows while it settles.
  await page.evaluate(() => {
    window.__wildFaces = [];
    new MutationObserver((list) => {
      for (const m of list) {
        const el = m.target;
        if (el instanceof Element && el.matches('.hand-card[data-id="15"] use.face-pic')) window.__wildFaces.push(el.getAttribute('href'));
      }
    }).observe(document.body, { subtree: true, attributes: true, attributeFilter: ['href'] });
  });
  await page.click('[data-action="declare"]');
  await page.waitForSelector('.hand-card[data-id="15"].is-settling', { timeout: 15000 });
  await page.waitForTimeout(250);
  await page.screenshot({ path: join(screens, 'wild-settling-390x844.png') });
  await page.locator('.hand-card[data-id="15"]').screenshot({ path: join(screens, 'wild-settling-closeup.png') });
  await page.waitForSelector('.hand-card[data-id="15"].is-settling', { state: 'detached', timeout: 15000 });
  const faces = await page.evaluate(() => window.__wildFaces);
  const suits = new Set(faces);
  if (suits.size < 4 || faces[faces.length - 1] !== '#ws-SA') fail(`the Wandelkarte did not cycle through the suits: ${faces.join(', ')}`);
  await assertSettledWild(page.locator('.hand-card[data-id="15"] svg.card-svg'), 'after the cycle');
  log(`Wandelkarte cycle: ${faces.join(' ')}`);
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
  for (const mod of ['Wandelkarte', 'Aufdruck']) {
    if (!labels.includes(mod)) fail(`wave 3: no hand card names ${mod}: ${labels}`);
  }
  // The Wandelkarte before the game starts: the open harlequin with the four suit pips.
  const open = page.locator('.hand-card[data-id="15"] svg.card-svg');
  if ((await open.getAttribute('data-wild')) !== 'open') fail('the Wandelkarte before the declaration is not shown open');
  if (!(await open.locator('use.face-pic[href="#w-A"]').count()) || !(await open.locator('use.face-idx[href="#wx-A"]').count())) {
    fail('the open Wandelkarte does not show the harlequin face and index');
  }
  if (await open.getAttribute('data-suit')) fail('the open Wandelkarte claims a suit');
  await shoot(page, 'wave3-hand');
  await page.locator('.hand-card[data-id="15"]').screenshot({ path: join(screens, 'wild-open-closeup.png') });
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
  // The Wandelkarte (Rot-Daus, id 15) settles as the game starts: five
  // spades in the hand, it is a Grün-Daus now (DESIGN 7.2, 03.11.2026).
  // "Ehrenvorsitz": the player leads it.
  await page.waitForSelector('[data-action="play"][data-id="15"]', { timeout: 30000 });
  const settled = (await page.locator('.hand-card[data-id="15"]').first().getAttribute('aria-label')) ?? '';
  if (!settled.startsWith('Grün-Daus')) fail(`the settled Wandelkarte shows as ${settled}`);
  await assertSettledWild(page.locator('.hand-card[data-id="15"] svg.card-svg'), 'in the hand');
  await page.waitForFunction(() => !document.querySelector('.run.is-busy'));
  await shoot(page, 'wave3-wild-settled-hand');
  await page.locator('.hand-card[data-id="15"]').screenshot({ path: join(screens, 'wild-settled-closeup.png') });
  // The Infokarte shows the settled card, too.
  await page.locator('.hand-card[data-id="15"]').focus();
  await page.keyboard.press('i');
  await page.waitForSelector('[data-testid="info-card"]');
  {
    await assertSettledWild(page.locator('[data-testid="info-card"] svg.card-svg').first(), 'on the Infokarte');
    await page.locator('[data-testid="info-card"]').screenshot({ path: join(screens, 'wild-settled-info.png') });
    await page.click('[data-testid="info-card"] [data-info-close]');
    await page.waitForSelector('[data-testid="info-card"]', { state: 'detached' });
  }
  // The edition on the Kreidetafel shows it settled while the game runs.
  await page.click('.hud [data-action="overview"]');
  await page.waitForSelector('.deck-grid');
  await assertSettledWild(page.locator('.deck-grid [data-info="card:15"] svg.card-svg'), 'in the edition');
  await page.locator('.deck-grid').screenshot({ path: join(screens, 'wild-settled-edition.png') });
  await page.click('.sheet [data-action="close-overlay"]');
  await page.waitForSelector('.deck-grid', { state: 'detached' });
  await page.click('[data-action="play"][data-id="15"]');
  await page.waitForFunction(() => window.__toasts.some((t) => t.includes('Wandelkarte')), null, { timeout: 30000 });
  // The trick on the table, or once it is finished the "Letzter Stich".
  await page.waitForSelector('[data-action="play"], .score-sheet', { timeout: 30000 });
  if (!(await page.locator('.trick-card[data-card-id="15"]').count())) {
    await page.waitForFunction(() => !document.querySelector('.run.is-busy'));
    await page.click('[data-testid="last-trick-toggle"]');
    await page.waitForSelector('[data-testid="last-trick"]');
  }
  const wild = page.locator('.trick-card[data-card-id="15"]').first();
  if (!(await wild.locator('.plaque-wild').count())) fail('the settled Wandelkarte has no plaque on the table');
  const shown = await wild.locator('svg.card-svg').getAttribute('aria-label');
  if (!(shown ?? '').startsWith('Grün-Daus')) fail(`the settled Wandelkarte shows as ${shown}`);
  await assertSettledWild(wild.locator('svg.card-svg'), 'in the trick');
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
/**
 * The characters of 03.11.2026 (DESIGN 9): the picker lists five, the
 * removed ones (Grandspieler, Zaungast, Wirtin) are gone. An old meta save
 * that still names a removed character loads, its stored pick falls back
 * to the Alleinspieler, and a run of Bruderherz shows his card back.
 */
async function charactersPass(browser, errors) {
  const context = await quietContext(browser, { viewport: PHONE });
  const page = await context.newPage();
  watch(page, errors);
  await page.goto(base);
  await page.waitForSelector('[data-testid="characters"]');
  const old = {
    version: 1,
    meta: {
      version: 1,
      unlocked: ['full_till', 'grand_master', 'tour_won', 'true_colours'],
      stats: { runs_finished: 4, characters: { Landlady: { runs: 2 }, Declarer: { runs: 2 } } },
      stakes: { Landlady: { Full: 1 }, Declarer: { Full: 1 } },
    },
  };
  await page.evaluate((meta) => {
    localStorage.setItem('stammtisch.meta', JSON.stringify(meta));
    localStorage.setItem('stammtisch.character', 'Landlady');
    localStorage.removeItem('stammtisch.run');
  }, old);
  await page.reload();
  await page.waitForSelector('[data-action="pick-character"][data-character="Declarer"][aria-pressed="true"]');
  const text = (await page.textContent('body')) ?? '';
  if (text.includes('passte nicht zu dieser Version')) fail('the old meta save with a removed character was discarded');
  const ids = await page.$$eval('[data-testid="characters"] .char-card', (cards) => cards.map((c) => c.dataset.character));
  if (ids.join(',') !== 'Declarer,NullPlayer,Shirker,Tailoress,HeartBrother') fail(`characters on the start screen: ${ids.join(', ')}`);
  for (const id of ['Shirker', 'Tailoress']) {
    const card = page.locator(`.char-card.is-locked[data-character="${id}"]`);
    if (!(await card.count())) fail(`${id} is not shown as locked`);
    const cond = (await card.textContent()) ?? '';
    if (!/Gewinne/.test(cond)) fail(`locked ${id} without his condition: ${cond}`);
  }
  if (await page.locator('.char-card.is-locked[data-character="HeartBrother"]').count()) fail('Bruderherz should be unlocked by true_colours');
  await page.click('[data-action="pick-character"][data-character="HeartBrother"]');
  await page.waitForSelector('[data-testid="char-detail"]');
  const detail = (await page.textContent('[data-testid="char-detail"]')) ?? '';
  if (!detail.includes('Bruderherz') || detail.includes('Multiplikator')) fail(`detail of Bruderherz: ${detail}`);
  await page.evaluate(() => {
    const row = document.querySelector('[data-testid="characters"]');
    if (row) row.scrollLeft = row.scrollWidth;
  });
  log('start screen: five characters, old pick of a removed one fell back to the Alleinspieler');
  await shoot(page, 'start-characters-new');
  await page.fill('#seed', SEED);
  await page.click('[data-action="new-run"]');
  await page.waitForSelector('[data-testid="skat-choice"]');
  const backs = await page.$$eval('.card-back use', (uses) => [...new Set(uses.map((u) => u.getAttribute('href')))]);
  if (!backs.includes('#bk-heart_brother')) fail(`Bruderherz's back not shown: ${backs.join(', ')}`);
  if (backs.includes('#bk-distel')) fail(`the Distel still shows for Bruderherz: ${backs.join(', ')}`);
  log(`Bruderherz plays with his own back (${backs.join(', ')})`);
  await shoot(page, 'heart-brother-table');
  await context.close();
}

async function routePass(page, label) {
  await page.waitForSelector('[data-testid="route"]');
  const options = page.locator('[data-testid="route-option"]');
  if ((await options.count()) !== 3) fail(`${label}: ${await options.count()} ways, expected 3`);
  const categories = await options.evaluateAll((els) => els.map((e) => e.getAttribute('data-category')));
  if (!categories.includes('Tresen')) fail(`${label}: no way over the Tresen: ${categories}`);
  if (new Set(categories).size !== 3) fail(`${label}: the ways do not show three different stops: ${categories}`);
  const transports = await options.evaluateAll((els) => els.map((e) => e.getAttribute('data-transport')).join(','));
  if (transports !== 'Walk,Train,Bike') fail(`${label}: ways ${transports}`);
  const next = ((await page.locator('[data-testid="route-next"]').textContent()) ?? '').replace(/\s+/g, ' ');
  if (!/Ziel \d+ Augen in 3 Spielen/.test(next) || !next.includes('Am Tisch:')) fail(`${label}: next pub ${next}`);
  const node = await page.locator('[data-testid="route-next"]').getAttribute('data-node');
  if (!/^d\d-p\d$/.test(node ?? '')) fail(`${label}: node id ${node}`);
  if (await page.locator('.hud .heart, [data-testid="lives"]').count()) fail(`${label}: hearts in the head of the route screen (ADR 0019: no lives)`);
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
  // The malus has an info card like relics and Schnäpse; a long press opens
  // it and marks no text, the page allows no selection (playtest 04.11.2026).
  const malusBox = await page.locator('[data-testid="route-malus"]').boundingBox();
  await page.mouse.move(malusBox.x + 20, malusBox.y + malusBox.height / 2);
  await page.mouse.down();
  await page.waitForTimeout(650);
  await page.mouse.up();
  await page.waitForSelector('[data-testid="info-card"][data-info-key^="malus:"]', { timeout: 3000 }).catch(() => fail('Spelunke: a long press on the malus opens no info card'));
  const malusMarked = await page.evaluate(() => document.getSelection()?.toString() ?? '');
  if (malusMarked) fail(`Spelunke: a long press on the malus marked text: "${malusMarked.slice(0, 60)}"`);
  const selectable = await page.evaluate(() => getComputedStyle(document.body).userSelect);
  if (selectable !== 'none') fail(`the page allows text selection (user-select: ${selectable})`);
  await page.keyboard.press('Escape');
  await page.waitForSelector('[data-testid="info-card"]', { state: 'detached' });
  const end = (await mapState(page)).cells.split(' ').pop();
  log(`Spelunke ${card} ${Math.round(apart)}px from the next pub; its way ends at ${end}`);
  await shoot(page, 'route-spelunke');
  await assertRouteFits(page, 'route with a Spelunke picked');
  // Into the Spelunke (the stop on the way is an encounter: swap a relic):
  // at its table the malus chip in the Leiste opens its info card on a
  // long press and on a tap (playtest 04.11.2026).
  await page.click('[data-testid="route-go"]');
  await page.locator('[data-action="choose-encounter"]').first().click({ timeout: 10000 });
  await page.locator('[data-action="encounter-relic"]').first().click({ timeout: 5000 });
  await page.waitForSelector('[data-testid="pub-malus"]', { timeout: 10000 }).catch(() => fail('Spelunke: its table shows no malus chip'));
  if (await page.locator('[data-action="hint-dismiss"]').count()) await page.click('[data-action="hint-dismiss"]');
  const chip = await page.locator('[data-testid="pub-malus"]').boundingBox();
  await page.mouse.move(chip.x + chip.width / 2, chip.y + chip.height / 2);
  await page.mouse.down();
  await page.waitForTimeout(650);
  await page.mouse.up();
  await page.waitForSelector('[data-testid="info-card"][data-info-key^="malus:"]', { timeout: 3000 }).catch(() => fail('Spelunke: a long press on the malus chip opens no info card'));
  const chipMarked = await page.evaluate(() => document.getSelection()?.toString() ?? '');
  if (chipMarked) fail(`Spelunke: a long press on the malus chip marked text: "${chipMarked.slice(0, 60)}"`);
  await shoot(page, 'spelunke-malus-info');
  await page.keyboard.press('Escape');
  await page.waitForSelector('[data-testid="info-card"]', { state: 'detached' });
  await page.click('[data-testid="pub-malus"]');
  await page.waitForSelector('[data-testid="info-card"][data-info-key^="malus:"]', { timeout: 3000 }).catch(() => fail('Spelunke: a tap on the malus chip opens no info card'));
  log('Spelunke: the malus opens its info card on the route and at the table, no text marked');
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
  // The trail in the accent colour, not white (playtest 05.11.2026).
  const trailFill = await page2.evaluate(() => document.querySelector('.map-trail')?.getAttribute('fill'));
  if (trailFill?.toLowerCase() !== '#d1773f') fail(`travel: the trail is drawn in ${trailFill}, not the accent colour`);
  await page2.waitForTimeout(500);
  const moved = await page2.evaluate(() => document.querySelector('.map-pos')?.getAttribute('transform') ?? '');
  if (!/translate/.test(moved)) fail('travel: the position does not move along the way');
  await page2.waitForSelector('[data-testid="route"]', { state: 'detached', timeout: 10000 });
  log('travel: the position walked the way with a chalk trail, then the stop opened');
  await station.close();
}

/**
 * Scenes with full motion (animation package C), at one viewport: a relic
 * bought at the Tresen slides over the counter and lands in the Leiste, a
 * Schnaps pops its cork into its slot, a Zinke is stamped onto its card;
 * a boss pub opens under a spotlight with the plates dropping in (a tap
 * ends it at once); out of the pub straight onto the route (no door,
 * playtest 04.11.2026); a walker goes along the way. Each scene ends in what the core reports.
 */
async function scenesPass(browser, errors, viewport) {
  const context = await quietContext(browser, { viewport, reducedMotion: 'no-preference' });
  const page = await context.newPage();
  watch(page, errors);
  const tag = `${viewport.width}x${viewport.height}`;
  const snap = (name) => page.screenshot({ path: join(screens, `${name}-${tag}.png`) });
  const sideways = async (where) => {
    const wide = await page.evaluate(() => document.documentElement.scrollWidth > window.innerWidth + 1);
    if (wide) fail(`scenes ${tag}: ${where} scrolls sideways`);
  };

  // The Tresen: a relic into the strip, a Schnaps into its slot, a Zinke stamped on.
  await resumeFixture(page, 'zinken.json');
  await page.waitForSelector('[data-testid="shop"]');
  const relics = await page.locator('.leiste-relics .coaster-chip').count();
  await page.locator('[data-action="buy-relic"]:not([disabled])').first().click();
  await page.waitForSelector('[data-testid="relic-fly"]', { timeout: 5000 }).catch(() => fail(`scenes ${tag}: the bought relic does not fly`));
  await page.waitForTimeout(500);
  await snap('scene-relic');
  await page.waitForSelector('[data-testid="relic-fly"]', { state: 'detached', timeout: 5000 });
  if ((await page.locator('.leiste-relics .coaster-chip').count()) !== relics + 1) fail(`scenes ${tag}: the relic did not land in the Leiste`);
  await page.locator('[data-action="buy-item"]:not([disabled])').first().click();
  await page.waitForSelector('[data-testid="schnaps-cork"]', { timeout: 5000 }).catch(() => fail(`scenes ${tag}: the Schnaps pops no cork`));
  await page.waitForTimeout(350);
  await snap('scene-schnaps');
  await page.waitForSelector('[data-testid="schnaps-fly"]', { state: 'detached', timeout: 5000 });
  if (!(await page.locator('.leiste-slots .schnaps-slot:not(.is-empty)').count())) fail(`scenes ${tag}: the Schnaps is not in its slot`);
  await page.locator('[data-testid="zinke"] [data-action="buy-zinke"]:not([disabled])').first().click();
  await page.waitForSelector('[data-testid="mod-confirm-go"]', { timeout: 5000 }).catch(() => fail(`scenes ${tag}: the Auslage opens no sheet`));
  await page.click('[data-testid="mod-confirm-go"]');
  await page.waitForSelector('[data-testid="zinke-stamp"]', { timeout: 5000 }).catch(() => fail(`scenes ${tag}: no stamp comes down for the Zinke`));
  const faces = [];
  for (let n = 0; n < 2; n++) {
    await page.waitForTimeout(n ? 400 : 120);
    faces.push(await page.evaluate(() => document.querySelector('[data-testid="zinke-stamp-card"]')?.innerHTML ?? ''));
    if (n) await snap('scene-stamp');
  }
  if (!faces[0] || faces[0] === faces[1]) fail(`scenes ${tag}: the stamped card does not change its face`);
  await page.waitForSelector('[data-testid="zinke-stamp-card"]', { state: 'detached', timeout: 5000 });
  await sideways('the Tresen');
  log(`scenes ${tag}: relic into the Leiste, Schnaps cork, Zinke stamped`);

  // The boss pub opens: dark, spotlight, plates; a tap ends it.
  await resumeFixture(page, 'boss-tresen.json');
  await page.waitForSelector('[data-testid="shop"]');
  await page.click('[data-action="leave-shop"]');
  await page.click('[data-action="boss-go"]');
  await page.waitForTimeout(300);
  if (await page.locator('.sheet-unlock [data-action="close-overlay"]').count()) await page.click('.sheet-unlock [data-action="close-overlay"]');
  await page.waitForSelector('[data-testid="boss-entrance"]', { timeout: 5000 }).catch(() => fail(`scenes ${tag}: the boss pub opens without its entrance`));
  await page.mouse.move(2, viewport.height - 2);
  await page.waitForTimeout(600);
  await snap('scene-boss');
  const dropping = await page.evaluate(() => document.getAnimations().filter((a) => a.id === 'boss-plate').length);
  if (dropping < 1) fail(`scenes ${tag}: no name plate drops in`);
  await page.mouse.click(2, viewport.height - 2);
  await page.waitForSelector('[data-testid="boss-entrance"]', { state: 'detached', timeout: 400 }).catch(() => fail(`scenes ${tag}: a tap does not end the boss entrance`));
  await sideways('the boss entrance');
  log(`scenes ${tag}: boss entrance with ${dropping} plates, ended by a tap`);

  // Out of the boss pub straight onto the route (the door scene is gone, playtest 04.11.2026), then the walk.
  await resumeFixture(page, 'boss-reward.json');
  await page.click('[data-action="boss-relic-skip"]');
  await page.waitForSelector('[data-testid="route-go"]');
  if (await page.locator('.fx-door, [data-testid="door-wipe"]').count()) fail(`scenes ${tag}: a door still swings open onto the route`);
  const way = await page.getAttribute('[data-testid="route-option"][aria-pressed="true"]', 'data-transport');
  await page.click('[data-testid="route-go"]');
  await page.waitForSelector('[data-testid="map-walker"]', { timeout: 5000 }).catch(() => fail(`scenes ${tag}: no walker on the way`));
  const walker = await page.getAttribute('[data-testid="map-walker"]', 'data-way');
  if ({ Walk: 'fuss', Train: 'bahn', Bike: 'rad' }[way ?? ''] !== walker) fail(`scenes ${tag}: the way ${way} is walked as ${walker}`);
  // Transparent like the other map icons: no paper box behind it (playtest 04.11.2026).
  if (await page.locator('[data-testid="map-walker"] rect').count()) fail(`scenes ${tag}: the walker carries a box behind it`);
  await page.waitForTimeout(700);
  await snap('scene-walker');
  await page.waitForSelector('[data-testid="route"]', { state: 'detached', timeout: 10000 });
  log(`scenes ${tag}: straight onto the route, ${walker} along the way`);
  await context.close();
}

/**
 * Die lange Nacht (ADR 0020): a won run (the last boss pub, patched from
 * boss-table.json) offers "Weiterziehen in die lange Nacht" on its end
 * screen; going on opens the start sheet of Stadtteil 4 with its
 * Verschärfung, the HUD names the night district and shows the
 * Verschärfung as a chip (a tap explains it), the run is saved again, the
 * first pub of the night plays its grown Zielwert, the start screen's
 * Bestenliste counts the night, and a night relic has its beer mat and info
 * card. Nothing scrolls sideways.
 */
async function nightPass(browser, errors, viewport) {
  const context = await quietContext(browser, { viewport, reducedMotion: 'reduce' });
  const page = await context.newPage();
  watch(page, errors);
  const tag = `${viewport.width}x${viewport.height}`;
  const snap = (name) => page.screenshot({ path: join(screens, `${name}-${tag}.png`) });
  const won = JSON.parse(fixture('boss-table.json'));
  won.run.series.section = 2;
  won.run.series.target = 1;
  won.run.series.score = 1000;
  won.run.series.games_played = 2;
  await resumeFixture(page, null, JSON.stringify(won));
  await page.waitForSelector('.declare');
  await skatThenGrand(page, `night ${tag}`);
  await playOut(page, `night ${tag}`);
  await page.click('[data-action="score-continue"]');
  await page.waitForSelector('[data-testid="end"] h1.is-won', { timeout: 10000 });
  await page.waitForTimeout(300);
  while (await page.locator('.sheet-unlock').count()) {
    await page.locator('.sheet-unlock [data-action="close-overlay"]').last().click();
    await page.waitForTimeout(200);
  }
  const enter = page.locator('[data-testid="enter-night"]');
  if (!(await enter.count())) fail(`night ${tag}: no "Weiterziehen in die lange Nacht" on the won end screen`);
  if (!(await page.locator('[data-testid="end-run"]').count())) fail(`night ${tag}: no "Run beenden" next to it`);
  if (!(await page.evaluate(() => localStorage.getItem('stammtisch.run') === null))) fail(`night ${tag}: the won run is still saved`);
  await enter.scrollIntoViewIfNeeded();
  await snap('night-end');
  await assertNoSideScroll(page, `night ${tag}: end screen`, '[data-testid="end"]');
  await enter.click();
  await page.waitForSelector('.sheet-night [data-testid="night-sheet-title"]', { timeout: 10000 }).catch(() => fail(`night ${tag}: no start sheet of the night district`));
  const title = (await page.locator('[data-testid="night-sheet-title"]').textContent()) ?? '';
  if (!title.includes('Stadtteil 4')) fail(`night ${tag}: the start sheet says "${title}"`);
  const fresh = await page.locator('[data-testid="night-new"]').getAttribute('data-verschaerfung');
  if (!fresh) fail(`night ${tag}: the start sheet names no Verschärfung`);
  await page.waitForTimeout(200);
  await snap('night-sheet');
  await assertNoSideScroll(page, `night ${tag}: start sheet`, '.sheet-night');
  await page.click('[data-testid="night-go"]');
  await page.waitForSelector('.sheet-night', { state: 'detached' });
  await page.waitForSelector('[data-testid="route"]');
  const saved = await page.evaluate(() => JSON.parse(localStorage.getItem('stammtisch.run') ?? '{}').run?.night ?? null);
  if (!saved || saved.verschaerfungen?.[0] !== fresh) fail(`night ${tag}: the night is not saved (${JSON.stringify(saved)})`);
  const chips = await page.locator('.hud [data-testid="night-chip"]').count();
  if (chips !== 1) fail(`night ${tag}: ${chips} Verschärfung chips in the HUD`);
  const where = (await page.locator('.hud-where-sub').textContent()) ?? '';
  if (!where.includes('Lange Nacht · Stadtteil 4')) fail(`night ${tag}: the HUD says "${where}"`);
  if (!(await page.locator('.run.is-night').count())) fail(`night ${tag}: no night look`);
  await page.locator('.hud [data-testid="night-chip"]').first().click();
  await page.waitForSelector('[data-testid="info-card"][data-info-key^="night:"]', { timeout: 3000 }).catch(() => fail(`night ${tag}: a Verschärfung chip explains nothing`));
  await page.click('[data-testid="info-card"] [data-info-close]');
  // To the first pub of Stadtteil 4 over the Tresen.
  await page.click('[data-action="pick-route"][data-category="Tresen"]');
  await page.click('[data-testid="route-go"]');
  await page.waitForSelector('[data-action="leave-shop"]', { timeout: 15000 });
  await page.click('[data-action="leave-shop"]');
  await page.waitForSelector('.declare, [data-testid="skat-choice"]', { timeout: 15000 });
  const target = Number(await page.locator('.slate-target').textContent());
  // Stadtteil 3 (115) × 0.6 since balance round 5 (05.11.2026, eight relic
  // slots), × 1.1 with a Zielwert-Sprung, × 0.85 in a Spelunke.
  if (!(target >= 55 && target <= 80)) fail(`night ${tag}: the first pub of the night has Zielwert ${target}`);
  // Unlocks of a fresh meta state (Stadtteil 2 and on) first, and the toast of the new district.
  await page.waitForTimeout(300);
  while (await page.locator('.sheet-unlock').count()) {
    await page.locator('.sheet-unlock [data-action="close-overlay"]').last().click();
    await page.waitForTimeout(200);
  }
  await page.mouse.move(1, 1);
  await page.waitForTimeout(3500);
  await snap('night-hud');
  log(`night ${tag}: end screen, start sheet (${fresh}), chip, first night pub with Zielwert ${target}`);
  // A night relic: its beer mat in the Leiste and its info card.
  const run = JSON.parse(await page.evaluate(() => localStorage.getItem('stammtisch.run')));
  run.run.relics.push('night_owl');
  await resumeState(page, 'night with a night relic', JSON.stringify(run));
  await page.waitForSelector('.hud [data-relic="night_owl"]');
  await page.click('.hud [data-relic="night_owl"]');
  await page.waitForSelector('[data-testid="info-card"][data-info-key="relic:night_owl"]', { timeout: 3000 }).catch(() => fail(`night ${tag}: no info card for the night relic`));
  await page.waitForTimeout(200);
  await snap('night-relic');
  await page.click('[data-testid="info-card"] [data-info-close]');
  // The Bestenliste on the start screen counts the night.
  await page.goto(base);
  await page.waitForSelector('[data-testid="start-board"]', { timeout: 10000 }).catch(() => fail(`night ${tag}: no Bestenliste on the start screen`));
  const board = (await page.locator('[data-testid="start-board"] tr[data-character="Declarer"]').textContent()) ?? '';
  if (!board.includes('Stadtteil 4')) fail(`night ${tag}: the Bestenliste says "${board}"`);
  await page.locator('[data-testid="start-board"]').scrollIntoViewIfNeeded();
  await snap('night-board');
  await assertNoSideScroll(page, `night ${tag}: Bestenliste`, '[data-testid="start-board"]');
  log(`night ${tag}: night relic info card, Bestenliste "${board.replace(/\s+/g, ' ').trim()}"`);
  await context.close();
}

async function run() {
  const preview = await startPreview();
  const browser = await chromium.launch();
  const errors = [];
  try {
    const reducedMotion = process.env.SMOKE_FULL_MOTION ? 'no-preference' : 'reduce';
    // SMOKE_ONLY=night: only the long night (ADR 0020), for quick checks.
    if (process.env.SMOKE_ONLY === 'weiter') {
      await scoreContinuePass(browser, errors);
      return;
    }
    // SMOKE_ONLY=undead: the Untot looks and the crumble (playtest 05.11.2026).
    if (process.env.SMOKE_ONLY === 'undead') {
      await framesPass(browser, errors, reducedMotion);
      await voucherPass(browser, errors);
      await undeadPass(browser, errors, reducedMotion);
      await undeadGlowPass(browser, errors);
      return;
    }
    // SMOKE_ONLY=bosses: the boss passes with the route legend and the names on the map.
    if (process.env.SMOKE_ONLY === 'bosses') {
      await bossesV2(browser, errors, reducedMotion);
      return;
    }
    // SMOKE_ONLY=doppel: the Zinken v2 looks and the Doppelgänger's mirror (07.11.2026).
    if (process.env.SMOKE_ONLY === 'doppel') {
      await zinkenV2Pass(browser, errors, PHONE);
      await zinkenV2Pass(browser, errors, DESKTOP);
      return;
    }
    if (process.env.SMOKE_ONLY === 'night') {
      await nightPass(browser, errors, PHONE);
      await nightPass(browser, errors, DESKTOP);
      return;
    }
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
    const backIds = ['regulars', 'black_peter', 'grandma', 'the_pusher', 'ramscher', 'twins', 'the_old_man'];
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
    const portraits = await page.evaluate((faces) => ['p-char-Declarer', 'p-char-NullPlayer', 'p-char-Shirker', 'p-char-Tailoress', 'p-char-HeartBrother', 'p-dealer', ...faces].filter((id) => !document.getElementById(id)), BOSS_FACES);
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
    // No lives since ADR 0019: no hearts in the head.
    if (await page.locator('[data-testid="lives"], .hud .heart').count()) fail('hearts in the head (ADR 0019: no lives)');
    log(`head: ${progress}`);
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
        // The value at stake comes from the core: "Grundwert × Stufen".
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

    // The card reward (DESIGN 5.0) comes after a won pub: the first Zinke
    // that fits, on the first card it fits, before and after, "Anbringen".
    await takeCardReward(page);
    log('card reward taken after the first pub');

    // Kneipentour (ADR 0016): the pub was won in its first game, so the
    // route choice to the next pub opens: no hearts (ADR 0019), the next pub
    // with name, target and guests, three ways each with its stop.
    await routePass(page, 'first route');
    await takeRoute(page, 'Tresen');

    // Tresen.
    await page.waitForSelector('[data-testid="shop"]');
    await expectMusic(page, 'shop', 'Tresen');
    // Wares on the counter, own things in the bag; no "Benutzen" at the Tresen (playtest 05.11.2026).
    if (!(await page.locator('[data-testid="counter"]').count()) || !(await page.locator('[data-testid="pouch"] .pouch-title').count())) fail('Tresen: no counter and bag');
    if (await page.locator('.shop [data-action="use-item"]').count()) fail('Tresen: a Schnaps offers "Benutzen"');
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
    await factorPass(browser, errors, reducedMotion);
    await musicPass(browser, errors, reducedMotion);
    await suitTintPass(browser, errors, reducedMotion);
    await bossAndKontra(browser, errors, reducedMotion);
    await bossesV2(browser, errors, reducedMotion);
    await m4(browser, errors, reducedMotion);
    await m5(browser, errors, reducedMotion);
    await wave3(browser, errors, reducedMotion);
    await wildSettlePass(browser, errors);
    await framesPass(browser, errors, reducedMotion);
    await zinkenPass(browser, errors, reducedMotion);
    await tradePass(browser, errors, reducedMotion, PHONE);
    await tradePass(browser, errors, reducedMotion, DESKTOP);
    await relicSlotsPass(browser, errors, reducedMotion, PHONE);
    await relicSlotsPass(browser, errors, reducedMotion, DESKTOP);
    await rescuePass(browser, errors, reducedMotion, PHONE);
    await rescuePass(browser, errors, reducedMotion, DESKTOP);
    await schnapsPass(browser, errors);
    await workshopPass(browser, errors, reducedMotion);
    await cardRewardPass(browser, errors, reducedMotion);
    await scoreContinuePass(browser, errors);
    await encountersV2Pass(browser, errors, reducedMotion);
    await voucherPass(browser, errors);
    await specialPass(browser, errors, reducedMotion);
    await undeadPass(browser, errors, reducedMotion);
    await undeadGlowPass(browser, errors);
    await zinkenV2Pass(browser, errors, PHONE);
    await zinkenV2Pass(browser, errors, DESKTOP);
    await shortEvening(browser, errors, reducedMotion);
    await charactersPass(browser, errors);
    await wave2b(browser, errors, reducedMotion);
    await saveCode(browser, errors, reducedMotion, PHONE, true);
    await saveCode(browser, errors, reducedMotion, DESKTOP, false);
    await hintsAndRules(browser, errors, reducedMotion);
    await motionPass(browser, errors);
    await tableFeelPass(browser, errors, PHONE);
    await tableFeelPass(browser, errors, DESKTOP);
    await rewardFxPass(browser, errors, PHONE);
    await rewardFxPass(browser, errors, DESKTOP);
    await nightPass(browser, errors, PHONE);
    await nightPass(browser, errors, DESKTOP);
    await scenesPass(browser, errors, PHONE);
    await scenesPass(browser, errors, DESKTOP);
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

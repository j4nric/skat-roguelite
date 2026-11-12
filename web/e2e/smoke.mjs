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
//   thrown with a flick flies in one go (sampled every frame, no stop) with
//   a little turn and a bend, sets down a hair past its place and onto it
//   (playtest 12.11.2026, before: askew, a moment still, a slide); an opponent's card
//   flies in from his seat; a card let go over the hand goes back unplayed;
//   one the core does not let go springs back and shakes; a skat card
//   tapped on the table is lifted and flies into its sorted slot in the
//   hand (sampled every frame), a hand card tapped flies onto the table,
//   on Drücken both fly into the Ansage and after it into the stack in
//   the felt's corner, turning face down,
// - "Kartenfarbe tönen" (suitTintPass): the switch "Tönen" tints each card's
//   paper in its suit (off by default, stored, survives a reload), and each
//   rank letter in the corner has its suit's dark ink, never black,
// - Tresen (tresenPass, DESIGN 16.5; with SMOKE_ONLY=tresen): the wares
//   come before the strip with the next pub, which unfolds (aria-expanded);
//   buy buttons at least 44 px, a padlock on those the core says no to;
//   every cut text shows "mehr" on a ware that opens its info card; the
//   first group of wares and "Weiter" in sight without scrolling,
// - Papier (paperPass, DESIGN 16.6; with SMOKE_ONLY=paper): the score slip,
//   info cards, bubbles and the map's cards lie on the paper token; the role
//   inks have 4.5:1 on every paper tone; on every slip (assertScoreSlip, in
//   playOut) the Spielwert stands once, nothing scrolls, "Weiter" is in sight,
//   every text has 4.5:1; the longest slip at all sizes, text sizes, English,
// - Startseite (startScreenPass, DESIGN 16.4; with SMOKE_ONLY=characters):
//   locked characters after the free ones, "Neuer Run als …" docked at the
//   bottom edge without scrolling and covering nothing at the end of the
//   page, "Weitere Spielarten" and "mehr" fold and unfold (aria-expanded),
// - Barrierefreiheit (a11yPass; alone with SMOKE_ONLY=a11y): the row
//   "Darstellung" in the menu, "Farbenblind-Hilfe" puts a suit letter on
//   every face (stored), "Schriftgröße" Groß and Sehr groß keep the table
//   fitting and nothing scrolling sideways, cards and HUD have names, a
//   polite live region says opponents' cards, tricks and the result, the
//   menu takes the focus, keeps Tab inside and gives it back on Escape;
//   screenshots with simulated deuteranopia and protanopia (a11y-*.png),
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
// opponent's seat, and the table still fits with it at 390 × 844 and
// 375 × 667. Sprüche v2 (quipsPass; alone with SMOKE_ONLY=quips), in
// German and English: the hello is a greet line of its guest and stands
// on the felt, every bubble of a game is a line of its speaker, never two
// at once, not more than five in a game; the boss figures speak on the
// boss sheet, at the table, on the relic pick and at the end. The table must fit the visible height without scrolling
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
// Impressum, Datenschutz, Hilfe (legalPass; alone with SMOKE_ONLY=legal):
// the static pages answer 200 in German and English, link to each other,
// never scroll sideways at 360 px; the start screen's footer and the
// Abspann in the menu link to them; offline they come from the service worker's cache.
//
// The build's service worker stays on in every pass (it never touches
// /api). At the end the offline pass (SMOKE_ONLY=offline) reloads without
// network: the start screen and a new run come from the cache.
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

/**
 * Type a seed on the start screen. The field sits behind "Weitere
 * Spielarten" (DESIGN 16.4): open it first if it is folded.
 */
async function fillSeed(page, seed) {
  await openStartMore(page);
  await page.fill('#seed', seed);
}

/** Unfold "Weitere Spielarten" on the start screen (daily run, seed, Einführung), if folded. */
async function openStartMore(page) {
  const toggle = page.locator('[data-testid="start-more-toggle"]');
  await toggle.waitFor();
  if ((await toggle.getAttribute('aria-expanded')) !== 'true') await toggle.click();
  await page.waitForSelector('#start-more-body:not([hidden])');
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
 * too present): only backs, at the right end and the bottom of the trick
 * row, in the room the triangle of the trick leaves (over none of its
 * three places, right of the player's), a touch target of 44 px. A tap
 * shows them face up on the info card.
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
        ownRight: document.querySelector('.trick .slot-0')?.getBoundingClientRect().right ?? Infinity,
        // The places of the trick the stack lies over (none: the corner is free).
        over: [...document.querySelectorAll('.trick .slot')].filter((s) => {
          const b = s.getBoundingClientRect();
          return b.left < box.right && box.left < b.right && b.top < box.bottom && box.top < b.bottom;
        }).length,
      };
    });
    const where = `${label} at ${size.width}x${size.height}`;
    if (!r) fail(`${where}: no pressed stack on the felt`);
    if (r.backs !== 2 || r.faces) fail(`${where}: the pressed stack shows ${r.backs} backs and ${r.faces} faces, want 2 face down`);
    if (r.box.left < r.ownRight || r.over || r.row.right - r.box.right > 2 || r.row.bottom - r.box.bottom > 2) fail(`${where}: the pressed stack is not in the free bottom right corner (${JSON.stringify(r)})`);
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
 * The table is a place and says whose turn it is (11.11.2026, DESIGN 16.3),
 * at 390 × 844 while the player is to play: the head (everything above the
 * felt) takes at most 112 px; both opponents sit on the table top, each
 * with a fan of as many backs as the number beside it; "Du bist dran"
 * stands in a band between the felt and the hand, right above the cards;
 * the cards that may be played have a frame, the others are shaded, and
 * with reduced motion no card is lifted out of its row.
 */
async function assertTableLook(page, label) {
  await page.setViewportSize(PHONE);
  await page.waitForTimeout(250);
  const r = await page.evaluate(() => {
    const box = (sel) => document.querySelector(sel)?.getBoundingClientRect() ?? null;
    const top = box('.tabletop');
    const turn = box('[data-testid="your-turn"]');
    const hand = box('[data-testid="hand"]');
    const cards = [...document.querySelectorAll('.hand-card')];
    const css = (el) => getComputedStyle(el);
    return {
      felt: top ? { top: top.top, bottom: top.bottom, left: top.left, right: top.right } : null,
      seats: [...document.querySelectorAll('.tabletop .opp')].map((o) => {
        const b = o.querySelector('.portrait').getBoundingClientRect();
        return {
          inside: !!top && b.top >= top.top && b.bottom <= top.bottom && b.left >= top.left && b.right <= top.right,
          backs: o.querySelectorAll('.opp-hand .card-back').length,
          count: Number(o.querySelector('.opp-count')?.textContent ?? -1),
        };
      }),
      turn: turn ? { top: turn.top, bottom: turn.bottom, height: turn.height, text: document.querySelector('[data-testid="your-turn"]').textContent.trim() } : null,
      handTop: hand?.top ?? null,
      firstCardTop: Math.min(...cards.map((c) => c.getBoundingClientRect().top)),
      playable: cards.filter((c) => c.dataset.action === 'play').map((c) => ({ ring: css(c).getPropertyValue('--ring').trim(), rise: css(c).getPropertyValue('--rise').trim(), filter: css(c).filter })),
      dimmed: cards.filter((c) => c.classList.contains('is-dim')).map((c) => ({ filter: css(c).filter, opacity: css(c).opacity })),
      reduced: matchMedia('(prefers-reduced-motion: reduce)').matches,
      band: turn ? css(document.querySelector('[data-testid="your-turn"]')).backgroundColor : '',
      primary: [...document.querySelectorAll('.btn-primary')].map((b) => css(b).backgroundColor),
      view: `${innerWidth}x${innerHeight}`,
      hud: [...document.querySelectorAll('.hud > *, .run > *')].map((e) => `${e.className.split(' ')[0]} ${Math.round(e.getBoundingClientRect().top)}..${Math.round(e.getBoundingClientRect().bottom)}`),
    };
  });
  if (!r.felt) fail(`${label}: no table top`);
  if (r.felt.top > 112) fail(`${label}: the head takes ${Math.round(r.felt.top)}px above the felt, more than 112 (${r.view}: ${r.hud.join(', ')})`);
  if (r.seats.length !== 2 || r.seats.some((s) => !s.inside)) fail(`${label}: the opponents do not sit on the table top (${JSON.stringify(r.seats)})`);
  for (const s of r.seats) if (s.backs !== s.count) fail(`${label}: an opponent's fan shows ${s.backs} backs beside the number ${s.count}`);
  if (!r.turn || r.turn.text !== 'Du bist dran') fail(`${label}: no "Du bist dran" while the player is to play (${JSON.stringify(r.turn)})`);
  if (r.turn.top < r.felt.bottom || r.turn.bottom > r.firstCardTop + 2 || r.handTop - r.turn.bottom > 16) fail(`${label}: "Du bist dran" is not between the felt and the hand (${JSON.stringify(r)})`);
  if (r.turn.height < 24) fail(`${label}: the band of the turn is ${Math.round(r.turn.height)}px high`);
  // The colour of actions is ivory (#f1e6c8) since 11.11.2026: the band and the ring of a playable card.
  const IVORY = 'rgb(241, 230, 200)';
  if (r.band !== IVORY) fail(`${label}: the band of the turn is ${r.band}, not ivory`);
  if (r.playable.some((c) => c.ring !== IVORY)) fail(`${label}: the ring of a playable card is ${r.playable[0].ring}, not ivory`);
  if (r.primary.some((c) => c !== IVORY)) fail(`${label}: a primary button is ${r.primary.join(', ')}, not ivory`);
  if (!r.playable.length) fail(`${label}: no playable card`);
  for (const c of r.playable) {
    if (!c.ring || c.ring === 'transparent' || c.filter !== 'none') fail(`${label}: a playable card has no clear frame (${JSON.stringify(c)})`);
    if (r.reduced && c.rise !== '0px') fail(`${label}: a playable card is lifted by ${c.rise} with reduced motion`);
    if (!r.reduced && c.rise === '0px') fail(`${label}: a playable card is not lifted`);
  }
  for (const c of r.dimmed) if (c.filter === 'none' || c.opacity !== '1') fail(`${label}: a card that may not be played is not shaded, or see-through (${JSON.stringify(c)})`);
  log(`${label}: head ${Math.round(r.felt.top)}px, both at the table with their fans, "Du bist dran" above the hand, ${r.playable.length} playable cards framed, ${r.dimmed.length} shaded`);
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
  await assertScoreSlip(page, label);
  log(`${label}: ${played} cards played, score ${await text(page, 'score-result')}`);
  return played;
}

/**
 * The score slip (DESIGN 16.6), at its end: on the paper token; the
 * Spielwert stands once as a number (the sum "Spielwert = …"; the account
 * shows from and to, not the value again); the slip does not scroll and
 * "Weiter" is in sight (from 740 px height); every text on it has 4.5:1
 * against the paper or the fill under it.
 */
async function assertScoreSlip(page, label) {
  const r = await page.evaluate(() => {
    const sheet = document.querySelector('.score-sheet');
    if (!sheet) return { error: 'no slip' };
    const probe = document.createElement('i');
    probe.style.color = 'var(--paper)';
    document.body.append(probe);
    const paper = getComputedStyle(probe).color;
    probe.remove();
    const num = (sel) => (document.querySelector(sel)?.textContent ?? '').trim();
    const value = num('[data-num="value"]');
    const others = [num('[data-num="base"]'), num('[data-num="levels"]'), ...[...sheet.querySelectorAll('.step-effect')].map((e) => e.textContent.trim())];
    const re = new RegExp(`^[+−-]?${value}$`);
    const shown = [...sheet.querySelectorAll('*')].filter(
      (el) => !el.children.length && re.test((el.textContent ?? '').trim()) && !el.closest('.sr-only, .score-account, .payout'),
    ).length;
    // Contrast of every text against the first opaque fill behind it.
    const rgb = (c) => (c.match(/[\d.]+/g) ?? []).map(Number);
    const lum = ([r, g, b]) => [r, g, b].map((v) => ((v /= 255) <= 0.03928 ? v / 12.92 : ((v + 0.055) / 1.055) ** 2.4)).reduce((a, v, i) => a + v * [0.2126, 0.7152, 0.0722][i], 0);
    const fill = (el) => {
      for (let e = el; e; e = e.parentElement) {
        const c = rgb(getComputedStyle(e).backgroundColor);
        if (c.length === 3 || c[3] > 0.9) return c.slice(0, 3);
      }
      return [0, 0, 0];
    };
    const low = [];
    for (const el of sheet.querySelectorAll('*')) {
      const own = [...el.childNodes].some((n) => n.nodeType === 3 && n.textContent.trim());
      if (!own || el.closest('.sr-only, .is-pending, svg')) continue;
      const cs = getComputedStyle(el);
      if (cs.visibility === 'hidden' || Number(cs.opacity) < 1) continue;
      const a = lum(rgb(cs.color).slice(0, 3));
      const b = lum(fill(el));
      const ratio = (Math.max(a, b) + 0.05) / (Math.min(a, b) + 0.05);
      if (ratio < 4.5) low.push(`${el.className || el.tagName} "${el.textContent.trim().slice(0, 16)}" ${ratio.toFixed(2)}`);
    }
    const btn = document.querySelector('[data-action="score-continue"]')?.getBoundingClientRect();
    return {
      paper,
      bg: getComputedStyle(sheet).backgroundColor,
      value,
      shown,
      ambiguous: others.includes(value),
      old: document.querySelectorAll('.score-sheet .result-big, .score-sheet .value-line').length,
      scrolls: sheet.scrollHeight > sheet.clientHeight + 1,
      out: !btn || btn.bottom > innerHeight + 0.5 || btn.top < 0,
      tall: innerHeight >= 740,
      low,
    };
  });
  if (r.error) fail(`${label}: ${r.error}`);
  if (r.bg !== r.paper) fail(`${label}: the score slip is ${r.bg}, not the paper token ${r.paper}`);
  if (r.old) fail(`${label}: the old value line or big result is still on the slip`);
  if (!r.ambiguous && r.shown !== 1) fail(`${label}: the Spielwert ${r.value} stands ${r.shown} times on the slip`);
  if (r.tall && r.scrolls) fail(`${label}: the score slip scrolls`);
  if (r.tall && r.out) fail(`${label}: "Weiter" is out of sight on the score slip`);
  if (r.low.length) fail(`${label}: low contrast on the score slip: ${r.low.slice(0, 4).join('; ')}`);
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
 * Skat oder Hand on the felt (play-test feedback): the hello of a Stammgast
 * is the bubble at his seat and nothing else (11.11.2026: it is not written
 * out a second time under the question), the choices have short labels ("Einmischen" on the
 * deck, "Hand spielen") with their explanations on the info card (hover
 * here), the skat lies face down in the middle. A long press opens the
 * info card and marks no text; dragging across the table marks none either.
 */
async function assertChoice(page, seated) {
  if (await page.locator('[data-testid="table-talk"], .talk-line').count()) fail('the greetings are written out on the felt again, beside the bubble');
  // The hello stands once: in the bubble at its speaker's seat, nowhere else on the table.
  const said = seated.quip
    ? await page.evaluate((text) => [...document.querySelectorAll('.table *')].filter((el) => !el.children.length && el.textContent.includes(text)).map((el) => el.className), seated.quip.text)
    : [];
  if (seated.quip && (said.length !== 1 || !said[0].includes('quip'))) fail(`the greeting "${seated.quip.text}" stands ${said.length} times on the table (${said.join(', ')})`);
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
  // Dragging across the question and the opponents marks nothing.
  const line = await page.locator('.declare-choice .declare-title').boundingBox();
  await page.mouse.move(line.x + 2, line.y + 4);
  await page.mouse.down();
  await page.mouse.move(line.x + line.width - 2, line.y - 60, { steps: 6 });
  await page.mouse.up();
  const dragged = await page.evaluate(() => window.getSelection()?.toString() ?? '');
  if (dragged) fail(`dragging across the table marked text: "${dragged.slice(0, 60)}"`);
  await page.mouse.move(1, 1);
  log(`Skat oder Hand: the hello only in the bubble${seated.quip ? ` ("${seated.quip.text}")` : ''}, explanations on hover, no text marked`);
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
  await fillSeed(page, SEED);
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
  await fillSeed(page, SEED);
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
  // "Tönen" is a toggle in the row "Darstellung": its data-on is the state a tap sets.
  await page.waitForSelector('[data-testid="setting-display"] [data-testid="toggle-suit-tint"]');
  await page.click('[data-testid="toggle-suit-tint"][data-on="true"]');
  await page.waitForSelector('[data-testid="toggle-suit-tint"][aria-pressed="true"]');
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
  await page.click('[data-testid="toggle-suit-tint"][data-on="false"]');
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
const BOSS_FACES = ['the_pusher', 'grandma', 'twins', 'ramscher', 'black_peter', 'the_old_man', 'gravedigger'].flatMap((b) => [`p-boss-${b}`, `p-boss-${b}-2`]);

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
 * middle gives way to a chip in the felt's bottom right corner, which opens the
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
  await fillSeed(normal, SEED);
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
  await fillSeed(page, '5');
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
  // (The opponents' last cards are stored on the way to the start screen, so read it there.)
  await page.click('.hud [data-action="menu"]');
  await page.click('.sheet [data-action="to-start"]');
  await page.waitForSelector('[data-action="resume"]');
  const runBefore = await page.evaluate(() => localStorage.getItem('stammtisch.run'));
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
  await openStartMore(page);
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
  await fillSeed(page, seed);
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
  // Its toast is a strip over the head (11.11.2026), never over the seats or the cards on the table.
  const toast = await page
    .waitForSelector('.toast-layer.is-head .toast', { timeout: 3000 })
    .then((el) => el.boundingBox())
    .catch(() => null);
  const table = await page.locator(zone).boundingBox();
  const top = await page.locator('.tabletop').boundingBox();
  if (!toast) fail('Drei drücken: no toast over the head');
  if (toast.y + toast.height > table.y) fail(`Drei drücken: the toast (bottom ${Math.round(toast.y + toast.height)}) covers the table zone (top ${Math.round(table.y)})`);
  if (toast.y + toast.height > top.y) fail(`Drei drücken: the toast (bottom ${Math.round(toast.y + toast.height)}) reaches onto the table top (top ${Math.round(top.y)})`);
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
    await Promise.all(document.getAnimations().filter((a) => a.effect?.getComputedTiming?.().endTime !== Infinity).map((a) => a.finished.catch(() => undefined)));
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
      await Promise.all(document.getAnimations().filter((a) => a.effect?.getComputedTiming?.().endTime !== Infinity).map((a) => a.finished.catch(() => undefined)));
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
  await fillSeed(page, SEED);
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
  // Read with the code: making it stores what the opponents' last cards left in memory.
  const keysBefore = await storedKeys(page);
  if (!keysBefore['stammtisch.run'] || !keysBefore['stammtisch.meta']) fail(`${label}: run or meta not stored`);
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
  await page.evaluate(() => Promise.all(document.getAnimations().filter((a) => a.effect?.getComputedTiming?.().endTime !== Infinity).map((a) => a.finished.catch(() => undefined))));
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
      return { top: hint.top, bottom: hint.bottom, height: innerHeight, scroll: document.scrollingElement.scrollHeight - innerHeight, hits, table: document.querySelector('.hint-layer.is-head') ? (document.querySelector('.tabletop .opp')?.getBoundingClientRect().top ?? null) : null };
    }, HINT_KEEP_CLEAR);
    const where = `${label} at ${size.width}x${size.height}`;
    if (!r) fail(`${where}: no hint on screen`);
    if (r.top < 0 || r.bottom > r.height) fail(`${where}: the hint at ${r.top}..${r.bottom} leaves the viewport of ${r.height}`);
    if (r.scroll > 0) fail(`${where}: the page scrolls by ${r.scroll}px with the hint`);
    if (r.hits.length) fail(`${where}: the hint covers ${r.hits.join(', ')}`);
    if (r.table !== null && r.bottom > r.table + 1) fail(`${where}: the hint (bottom ${Math.round(r.bottom)}) reaches the seats at the table (top ${Math.round(r.table)})`);
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
  await fillSeed(page, SEED);
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
 * screen, the start button fits; an old `-kurz` link is offered like any
 * link and played as the whole evening (three districts in the HUD).
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
  if (/kurz/i.test(offer)) fail(`the offer of an old short link still speaks of the short evening: ${offer}`);
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
  await fillSeed(page, SEED);
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
    // The two opponents sit at the far edge of the table top (11.11.2026): a trick of theirs
    // leaves towards the winner's seat, up and to his side; the player's comes down to the hand.
    const toSeat = (side) => (sweep.x * side > 60 || (sweep.y < -60 && sweep.x * side > -8));
    const ok = winner === 'left' ? toSeat(-1) : winner === 'right' ? toSeat(1) : winner === 'you' ? sweep.y > 60 && Math.abs(sweep.x) < sweep.y : false;
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
  await fillSeed(page, SEED);
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
  await page.waitForSelector('#seed', { state: 'attached', timeout: 5000 }).catch(() => fail(`${label}: "Zum Start" does not lead to the start screen`));
}

/**
 * The feel of the table with full motion (play-test feedback), at one
 * viewport: cards are thrown onto the felt in one flight with a little
 * turn and a bend, setting down on their place; a hand card
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

/**
 * The picked-up skat keeps its places (playtest 12.11.2026: "the cards
 * change shortly after"). Seeds 1 and 13 deal the skat in another order
 * than the hand sorts it; before the fix the two cards swapped places
 * when the pick-up was over. Full motion, every frame sampled.
 */
async function skatStaysPass(browser, errors) {
  for (const seed of ['1', '13']) {
    const context = await quietContext(browser, { viewport: PHONE, reducedMotion: 'no-preference' });
    const page = await context.newPage();
    watch(page, errors);
    await page.goto(base);
    await page.waitForSelector('[data-action="new-run"]');
    await fillSeed(page, seed);
    await page.click('[data-action="new-run"]');
    await page.waitForSelector('[data-action="pick-up-skat"]:not([disabled])');
    await page.waitForTimeout(1200);
    await page.evaluate(() => {
      window.__skatOrders = [];
      const t0 = performance.now();
      const frame = () => {
        const ids = [...document.querySelectorAll('[data-testid="skat-zone"] .skat-card')].map((e) => e.dataset.id).join(',');
        if (ids && ids !== window.__skatOrders[window.__skatOrders.length - 1]) window.__skatOrders.push(ids);
        if (performance.now() - t0 < 2500) requestAnimationFrame(frame);
      };
      requestAnimationFrame(frame);
    });
    await page.click('[data-action="pick-up-skat"]');
    await page.waitForTimeout(2600);
    const orders = await page.evaluate(() => window.__skatOrders);
    if (orders.length !== 1 || orders[0].split(',').length !== 2) fail(`skat, seed ${seed}: the cards on the table change places after the pick-up (${orders.join(' -> ')})`);
    log(`skat, seed ${seed}: the picked-up skat keeps its places (${orders[0]})`);
    await context.close();
  }
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
  await fillSeed(page, SEED);
  await page.click('[data-action="new-run"]');
  await page.waitForSelector('[data-action="pick-up-skat"]');
  await page.waitForTimeout(1200);
  // The cards on the table keep their places from the first frame they
  // show (playtest 12.11.2026: after the flight they swapped places).
  await page.evaluate(() => {
    window.__skatOrders = [];
    window.__skatStop = false;
    const frame = () => {
      const ids = [...document.querySelectorAll('[data-testid="skat-zone"] .skat-card')].map((e) => e.dataset.id).join(',');
      if (ids && ids !== window.__skatOrders[window.__skatOrders.length - 1]) window.__skatOrders.push(ids);
      if (!window.__skatStop) requestAnimationFrame(frame);
    };
    requestAnimationFrame(frame);
  });
  await page.click('[data-action="pick-up-skat"]');
  await page.waitForSelector('[data-testid="skat-zone"] [data-action="select"]');
  await page.waitForTimeout(700);
  const skatOrders = await page.evaluate(() => {
    window.__skatStop = true;
    return window.__skatOrders;
  });
  if (skatOrders.length !== 1 || skatOrders[0].split(',').length !== 2) fail(`feel ${tag}: the skat cards on the table change places after the pick-up (${skatOrders.join(' -> ')})`);
  log(`feel ${tag}: the picked-up skat keeps its places (${skatOrders[0]})`);
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

  // Thrown with a strong flick from the hand. Its place is sampled every
  // frame (playtest 12.11.2026: it flew half way, stood still and flew on).
  const id = await page.locator('[data-action="play"]').first().getAttribute('data-id');
  const felt = await page.locator('[data-testid="trick"]').boundingBox();
  await page.evaluate((cardId) => {
    window.__throwTrack = [];
    const t0 = performance.now();
    const frame = () => {
      const el = document.querySelector(`.trick-card[data-card-id="${cardId}"]`);
      const a = el?.getAnimations().find((x) => x.id === 'throw');
      if (el) {
        const r = el.getBoundingClientRect();
        window.__throwTrack.push({ t: performance.now() - t0, x: r.left + r.width / 2, y: r.top + r.height / 2, flying: Boolean(a) });
      }
      if (performance.now() - t0 < 2500) requestAnimationFrame(frame);
    };
    requestAnimationFrame(frame);
  }, id);
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
  const pts = flight.frames.map(parts);
  const start = pts[0];
  const end = pts[pts.length - 1];
  if (pts.some((p) => p.length !== 4)) fail(`feel ${tag}: unexpected keyframes ${JSON.stringify(flight.frames.slice(0, 3))}`);
  if (end[0] !== 0 || end[1] !== 0 || end[2] !== 0 || end[3] !== 1) fail(`feel ${tag}: the card does not end exactly on its place: ${flight.frames[flight.frames.length - 1]}`);
  // One flight: no two keyframes alike (a card lying still), a little turn, a bend, a set-down past the place.
  for (let i = 1; i < flight.frames.length; i++) if (flight.frames[i] === flight.frames[i - 1]) fail(`feel ${tag}: the card stands still in its flight (${flight.frames[i]})`);
  if (Math.abs(start[2]) < 4 || Math.abs(start[2]) > 60) fail(`feel ${tag}: the card turns ${start[2]}° in flight (a little wanted)`);
  const len = Math.hypot(start[0], start[1]) || 1;
  const bend = Math.max(...pts.map((p) => Math.abs(p[0] * start[1] - p[1] * start[0]) / len));
  if (bend < 3) fail(`feel ${tag}: the card flies a straight line (bend ${bend.toFixed(1)} px)`);
  const past = pts.filter((p) => p[0] * start[0] + p[1] * start[1] < 0).map((p) => Math.hypot(p[0], p[1]));
  if (!past.length || Math.max(...past) > 16) fail(`feel ${tag}: the card does not set down a little past its place (${past.map((d) => d.toFixed(1)).join(', ')})`);
  if (flight.total < 250 || flight.total > 700) fail(`feel ${tag}: the throw takes ${Math.round(flight.total)} ms`);
  // Every frame on screen: once moving, it moves on until it is near its place.
  await page.waitForFunction(() => window.__throwTrack.length && !window.__throwTrack[window.__throwTrack.length - 1].flying && window.__throwTrack.some((p) => p.flying), null, { timeout: 5000, polling: 50 });
  const track = await page.evaluate(() => window.__throwTrack.filter((p) => p.flying));
  const last = track[track.length - 1];
  let stalls = 0;
  let moving = false;
  for (let i = 1; i < track.length; i++) {
    const near = Math.hypot(track[i].x - last.x, track[i].y - last.y) < 14;
    const step = Math.hypot(track[i].x - track[i - 1].x, track[i].y - track[i - 1].y);
    if (moving && !near && step < 0.5) stalls += 1;
    if (step >= 0.5) moving = true;
  }
  if (track.length < 8) fail(`feel ${tag}: the throw is seen in only ${track.length} frames`);
  if (stalls > 1) fail(`feel ${tag}: the card stops ${stalls} frames on its way (${JSON.stringify(track.map((p) => [Math.round(p.t), Math.round(p.x), Math.round(p.y)]))})`);
  log(`feel ${tag}: thrown card flies in one go (${Math.round(flight.total)} ms, ${track.length} frames, bend ${bend.toFixed(0)} px, turn ${start[2]}°), sets down on the player's place`);
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
  // The Leiste draws the relics held and counts the slots ("1/8"); free slots are not drawn (11.11.2026).
  const empty = await page.locator('.leiste-relics [data-testid="relic-slot-empty"]').count();
  if (empty) fail(`relic slots at ${size}: ${empty} free slots drawn in the Leiste, expected only the counter`);
  const counted = ((await page.locator('.leiste [data-testid="relic-count"]').textContent()) ?? '').trim();
  if (counted !== '1/8') fail(`relic slots at ${size}: the counter in the Leiste says "${counted}" with one relic, expected 1/8`);
  const countName = await page.locator('.leiste [data-testid="relic-count"]').getAttribute('aria-label');
  if (countName !== 'Relikte: 1 von 8 Plätzen belegt') fail(`relic slots at ${size}: the counter is called "${countName}"`);
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
  const full = ((await page.locator('.leiste [data-testid="relic-count"]').textContent()) ?? '').trim();
  if (full !== '8/8') fail(`relic slots at ${size}: the counter in the Leiste says "${full}" with eight relics, expected 8/8`);
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
 * The curse of a deal is seen (playtest 12.11.2026): with full motion,
 * buying the Schnäppchen of the Tresen (zinken.json as in tradePass) shows
 * the cursed card big over the screen, first as it was, then crumbling
 * into its cursed self like an "Untot" rising (rise-morph, kind "curse"),
 * the pictures from the core's views (before: no curse on it; after: the
 * curse the core drew, named on the card). Frames are saved
 * (curse-scene-*.png). Buying the Hehlerware then plays it again and a tap
 * ends it at once. With reduced motion (tradePass) nothing of it shows.
 */
async function curseScenePass(browser, errors) {
  const context = await quietContext(browser, { viewport: PHONE, reducedMotion: 'no-preference' });
  const page = await context.newPage();
  watch(page, errors);
  const save = JSON.parse(fixture('zinken.json'));
  save.run.shop.relics = ['jinx', 'virgin', 'skat_redraw'];
  save.run.shop.fence = 0;
  save.run.shop.card_offers[1] = { card: 3, card_mod: 'points', value: 3, price: 1, bargain: 2 };
  save.run.coins = 20;
  await resumeFixture(page, 'trade.json', JSON.stringify(save));
  await page.waitForSelector('[data-testid="zinken"]');
  await page.locator('[data-testid="zinken"]').scrollIntoViewIfNeeded();
  await page.click('[data-action="buy-zinke"][data-slot="1"]');
  const confirm = page.locator('[data-testid="mod-confirm-go"]');
  await confirm.waitFor({ timeout: 3000 }).then(() => confirm.click()).catch(() => {});
  const morph = page.locator('[data-testid="rise-morph"][data-kind="curse"]');
  await morph.waitFor({ timeout: 8000 }).catch(() => fail('curse scene: the cursed card is not shown after buying the Schnäppchen'));
  const labels = await morph.evaluate((box) => ({
    before: box.querySelector('.rise-whole svg.card-svg')?.getAttribute('aria-label') ?? '',
    after: box.querySelector('.rise-new svg.card-svg')?.getAttribute('aria-label') ?? '',
  }));
  const run = await page.evaluate(() => JSON.parse(localStorage.getItem('stammtisch.run') ?? '{}').run);
  const cursed = run.edition.cards.find((c) => c.mods.some((m) => ['transparent', 'battered', 'stained', 'demoted'].includes(m)) && c.id !== 3);
  if (!labels.before || !labels.after || labels.before === labels.after) fail(`curse scene: before and after are the same picture (${labels.before} / ${labels.after})`);
  if (!cursed) fail('curse scene: no cursed card in the saved run');
  log(`curse scene: "${labels.before}" crumbles into "${labels.after}"`);
  const frames = [];
  for (const stage of ['before', 'crumble', 'after']) {
    await page.waitForSelector(`[data-testid="rise-morph"][data-stage="${stage}"]`, { timeout: 4000 }).catch(() => fail(`curse scene: no stage "${stage}"`));
    await page.waitForTimeout(stage === 'crumble' ? 260 : 120);
    await page.screenshot({ path: join(screens, `curse-scene-${stage}-390x844.png`) });
    frames.push(stage);
  }
  await morph.waitFor({ state: 'detached', timeout: 5000 }).catch(() => fail('curse scene: the scene does not end'));
  // The Hehlerware: the scene again, and a tap ends it.
  await page.locator('[data-testid="relic-offers"]').scrollIntoViewIfNeeded();
  await page.click('[data-action="buy-relic"][data-slot="0"]');
  await morph.waitFor({ timeout: 8000 }).catch(() => fail('curse scene: the Hehlerware shows no cursed card'));
  const t0 = Date.now();
  await page.mouse.click(8, 300);
  await morph.waitFor({ state: 'detached', timeout: 1000 }).catch(() => fail('curse scene: a tap does not end it'));
  log(`curse scene: stages ${frames.join(', ')} seen, a tap ended the second one after ${Date.now() - t0} ms`);
  await context.close();
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
  if (reducedMotion === 'reduce' && (await page.locator('[data-testid="rise-morph"]').count())) fail('trade: the curse scene plays with reduced motion');
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
  // With full motion the mouth of the Schnaps before may still be gulping: the new one is the last.
  const box = await page.locator('[data-testid="drink-mouth"]').last().boundingBox();
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
      const box = r.getBoundingClientRect();
      // Relics reaching out of their strip without it scrolling (none: they fit or scroll inside).
      const spilled = r.scrollWidth > r.clientWidth ? 0 : [...r.querySelectorAll('.coaster-chip')].filter((c) => c.getBoundingClientRect().right > box.right + 1).length;
      return { scrolls: r.scrollWidth > r.clientWidth, spilled, page: document.scrollingElement.scrollWidth - innerWidth, slots: document.querySelector('.leiste-slots').getBoundingClientRect().right - innerWidth };
    });
    // On a phone nine relics are wider than the strip; the wider table of a desktop holds them.
    if (viewport.width < 600 && !strip.scrolls) fail(`Leiste at ${tag}: nine relics do not scroll inside the strip`);
    if (strip.spilled) fail(`Leiste at ${tag}: ${strip.spilled} relics reach out of the strip`);
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
    // The shown cards stay the same cards in the same places while they turn over and after.
    const peekSeen = await page.evaluate(
      () =>
        new Promise((resolve) => {
          const seen = [];
          const t0 = performance.now();
          const frame = () => {
            const ids = [...document.querySelectorAll('.skat-pile .skat-spot')].map((e) => `${e.dataset.skatId}:${e.querySelector('svg.card-svg, svg')?.getAttribute('aria-label') ?? ''}`).join('|');
            if (ids !== seen[seen.length - 1]) seen.push(ids);
            if (performance.now() - t0 < 1400) requestAnimationFrame(frame);
            else resolve(seen);
          };
          requestAnimationFrame(frame);
        }),
    );
    if (peekSeen.length !== 1 || peekSeen[0].includes('undefined')) fail(`Kurzer at ${tag}: the shown skat changes (${peekSeen.join(' -> ')})`);
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
    // The banner is a strip over the head (11.11.2026): never over the seats, the skat or the hand.
    const bannerAt = await page.evaluate(() => ({ bottom: document.querySelector('[data-testid="drink-banner"]').getBoundingClientRect().bottom, table: document.querySelector('.tabletop').getBoundingClientRect().top }));
    if (bannerAt.bottom > bannerAt.table + 1) fail(`Kurzer at ${tag}: the drag banner (bottom ${Math.round(bannerAt.bottom)}) reaches onto the table top (top ${Math.round(bannerAt.table)})`);
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
  await fillSeed(page, SEED);
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
 * "Weiter" under the finger (playtest 12.11.2026, iPhone app: "I can
 * scroll the slip but not tap Weiter, I have to restart the app"; again
 * after #86 and #103). WebKit on iOS may drop the click it synthesises
 * after a tap (a tap read as a hover, a double-tap gesture holding it
 * back, a stale hit-test after a scroll), so "Weiter", the board itself
 * and the boss sheet's "Los" act on the finger going up (./app/layer.ts).
 * Every end of a game, full motion and touch, several sizes and text
 * sizes, some played out without skipping: the element under the centre
 * (and four inner points) of "Weiter" is the button itself, also after
 * the bill was scrolled; then a tap whose click the page never sees (a
 * capture listener eats it, as WebKit would) still closes the board, and
 * so does a tap on the finished slip outside the bill. The next screen
 * answers (unlock, card reward, end screen).
 */
async function scoreTapPass(browser, errors) {
  const cases = [
    { label: 'won game', size: PHONE, text: 'normal', drop: true, wait: true },
    { label: 'lost game, the pub goes on', size: { width: 360, height: 640 }, text: 'xlarge', drop: false, wait: false, slip: false },
    { label: 'lost run', size: { width: 375, height: 667 }, text: 'large', drop: true, wait: true },
    { label: 'lost boss pub', size: { width: 390, height: 664 }, text: 'normal', drop: true, wait: false, slip: true },
    { label: 'won run, last boss pub', size: PHONE, text: 'xlarge', drop: true, wait: true },
    { label: 'pub won, unlock after', size: { width: 360, height: 740 }, text: 'normal', drop: true, wait: true },
  ];
  for (const c of cases) {
    const context = await quietContext(browser, { viewport: c.size, hasTouch: true, isMobile: true, reducedMotion: 'no-preference' });
    await context.addInitScript((textSize) => {
      try {
        const s = JSON.parse(localStorage.getItem('stammtisch.settings') ?? '{}');
        localStorage.setItem('stammtisch.settings', JSON.stringify({ ...s, tempo: 'normal', hints: false, textSize }));
      } catch {
        // Storage blocked: defaults.
      }
      // WebKit's dropped click, simulated: while armed, no click on a sheet reaches the page.
      window.__dropClicks = false;
      window.addEventListener(
        'click',
        (e) => {
          if (window.__dropClicks && e.target instanceof Element && e.target.closest('.overlay-score, .sheet-boss')) {
            e.preventDefault();
            e.stopImmediatePropagation();
          }
        },
        true,
      );
    }, c.text);
    const page = await context.newPage();
    watch(page, errors);
    const label = `Weiter under the finger, ${c.label} (${c.size.width}x${c.size.height}, ${c.text})`;
    if (c.label === 'won game') {
      await resumeFixture(page, 'factor-win.json');
      await page.waitForSelector('[data-action="play-hand"]');
      await page.click('[data-action="play-hand"]');
      await page.locator('[data-action="pick-game"]', { has: page.locator('.game-base', { hasText: /^9$/ }) }).click();
      await page.click('[data-action="declare"]');
    } else if (c.label.startsWith('lost game')) {
      await resumeFixture(page, 'run-end.json', patched('run-end.json', (save) => (save.run.series.games_played = 0)));
      await skatThenGrand(page, label);
    } else if (c.label === 'lost run') {
      await resumeFixture(page, 'run-end.json');
      await skatThenGrand(page, label);
    } else if (c.label === 'lost boss pub') {
      await resumeFixture(page, 'boss-lost.json');
      await page.waitForSelector('.declare[data-phase="Declaring"]');
      await page.click('[data-action="pick-game"][data-index="4"]');
      await page.click('[data-action="declare"]');
    } else if (c.label.startsWith('won run')) {
      const won = JSON.parse(fixture('boss-table.json'));
      won.run.series.section = 2;
      won.run.series.target = 1;
      won.run.series.score = 1000;
      won.run.series.games_played = 2;
      await resumeFixture(page, 'boss-table.json', JSON.stringify(won));
      await skatThenGrand(page, label);
    } else {
      await page.goto(base);
      await page.waitForSelector('[data-action="new-run"]');
      await fillSeed(page, SEED);
      await page.click('[data-action="new-run"]');
      await page.waitForSelector('[data-testid="skat-choice"]');
      await mixIn(page);
      await page.waitForSelector('.declare');
      await pickUpAndDiscard(page, label);
      await page.click('[data-action="pick-game"][data-index="4"]');
      await page.click('[data-action="declare"]');
    }
    await page.waitForSelector('.declare', { state: 'detached' });
    for (let played = 0; ; played++) {
      await page.waitForSelector('[data-action="play"], [data-action="re"][data-re="false"], .score-sheet', { timeout: 30000 });
      if (await page.locator('.score-sheet').count()) break;
      if (await page.locator('[data-action="re"][data-re="false"]').count()) {
        await page.click('[data-action="re"][data-re="false"]');
        continue;
      }
      await page.locator('[data-action="play"]').first().tap();
      if (played > 12) fail(`${label}: played more than ten cards`);
    }
    const title = (await page.locator('#score-title').getAttribute('class')) ?? '';
    if (!c.wait) {
      // A tap on the slip skips the show (the finger, not a click).
      const box = await page.locator('.score-head').boundingBox();
      await page.touchscreen.tap(box.x + box.width / 2, box.y + box.height / 2);
    }
    await page.waitForSelector('[data-action="score-continue"]', { timeout: 30000 });
    // What lies under "Weiter": the button, at its centre and four inner points.
    const under = async () =>
      page.evaluate(() => {
        const b = document.querySelector('[data-action="score-continue"]');
        b.scrollIntoView({ block: 'nearest' });
        const r = b.getBoundingClientRect();
        const pts = [[0.5, 0.5], [0.15, 0.3], [0.85, 0.3], [0.15, 0.7], [0.85, 0.7]];
        const wrong = pts
          .map(([fx, fy]) => {
            const x = r.left + r.width * fx;
            const y = r.top + r.height * fy;
            const el = document.elementFromPoint(x, y);
            return el && (el === b || b.contains(el)) ? null : `${Math.round(x)},${Math.round(y)}: ${el ? `${el.tagName}.${el.className}` : 'nothing'}`;
          })
          .filter(Boolean);
        return { wrong, inView: r.top >= 0 && r.bottom <= innerHeight + 0.5 };
      });
    let hit = await under();
    if (!hit.inView) fail(`${label}: "Weiter" cannot be brought into sight`);
    if (hit.wrong.length) fail(`${label}: something lies over "Weiter": ${hit.wrong.join('; ')}`);
    // Scroll the bill (if it scrolls) as a finger would, then look again.
    const scrolled = await page.evaluate(() => {
      const list = document.querySelector('.score-sheet .steps');
      if (!list || list.scrollHeight <= list.clientHeight + 1) return false;
      list.scrollTop = list.scrollHeight;
      return true;
    });
    if (scrolled) {
      await page.waitForTimeout(150);
      hit = await under();
      if (hit.wrong.length) fail(`${label}: after scrolling the bill something lies over "Weiter": ${hit.wrong.join('; ')}`);
    }
    await page.waitForTimeout(450);
    await page.evaluate((drop) => (window.__dropClicks = drop), c.drop);
    if (c.slip) {
      // The finished slip outside the bill goes on as well.
      const box = await page.locator('[data-part="account"]').boundingBox();
      await page.touchscreen.tap(box.x + box.width / 2, box.y + box.height / 2);
    } else {
      const box = await page.locator('[data-action="score-continue"]').boundingBox();
      await page.touchscreen.tap(box.x + box.width / 2, box.y + box.height / 2);
    }
    await page.waitForSelector('.score-sheet', { state: 'detached', timeout: 5000 }).catch(() => fail(`${label}: the tap${c.drop ? ' (its click dropped)' : ''} did not close the score board`));
    await page.evaluate(() => (window.__dropClicks = false));
    // The next screen answers.
    await page.waitForSelector('.sheet-unlock, [data-testid="card-reward"], .declare, [data-testid="route-go"], [data-testid="end"], [data-testid="shop"], [data-action="boss-go"]', { timeout: 15000 }).catch(() => fail(`${label}: no next screen after "Weiter"`));
    if (c.label.startsWith('pub won')) {
      await page.waitForSelector('[data-testid="unlock-new"]', { timeout: 10000 }).catch(() => fail(`${label}: no unlock after the won pub`));
      await page.tap('.sheet-unlock [data-action="close-overlay"]');
      await page.waitForSelector('[data-testid="card-reward"]', { timeout: 10000 }).catch(() => fail(`${label}: no card reward after the unlock`));
    }
    if (c.label.includes('run')) await assertEndButtons(page, label);
    log(`${label}: game ${title.includes('is-won') ? 'won' : 'lost'}, "Weiter" under the finger${scrolled ? ' (bill scrolled)' : ''}, closed by ${c.slip ? 'a tap on the slip' : 'a tap'}${c.drop ? ' whose click was dropped' : ''}`);
    await context.close();
  }

  // The boss sheet's "Los" with its click dropped.
  const context = await quietContext(browser, { viewport: PHONE, hasTouch: true, isMobile: true, reducedMotion: 'no-preference' });
  await context.addInitScript(() => {
    window.addEventListener(
      'click',
      (e) => {
        if (e.target instanceof Element && e.target.closest('.sheet-boss')) {
          e.preventDefault();
          e.stopImmediatePropagation();
        }
      },
      true,
    );
  });
  const page = await context.newPage();
  watch(page, errors);
  await resumeFixture(page, 'boss-tresen.json');
  await page.waitForSelector('[data-testid="shop"]');
  await page.click('[data-action="leave-shop"]');
  await page.waitForSelector('[data-action="boss-go"]');
  await page.waitForTimeout(400);
  const go = await page.locator('[data-action="boss-go"]').boundingBox();
  const goHit = await page.evaluate(({ x, y }) => document.elementFromPoint(x, y)?.closest('[data-action]')?.getAttribute('data-action') ?? null, { x: go.x + go.width / 2, y: go.y + go.height / 2 });
  if (goHit !== 'boss-go') fail(`boss sheet: something lies over "Los" (${goHit})`);
  await page.touchscreen.tap(go.x + go.width / 2, go.y + go.height / 2);
  await page.waitForSelector('.declare', { timeout: 10000 }).catch(() => fail('boss sheet: "Los" with its click dropped does not open the table'));
  log('boss sheet: "Los" under the finger, works without its click');
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
    // A mat on screen is a <use> of its symbol (ui/emblems.ts): look into the symbol.
    const ink = await mat.first().evaluate((svg) => {
      const href = svg.querySelector('use')?.getAttribute('href');
      const drawing = href ? document.querySelector(href) : svg;
      return Boolean(drawing?.querySelector('[fill="#ff8a1c"], [stroke="#ff8a1c"]'));
    });
    if (!ink) fail(`legendary relic ${id} lacks the legendary ink`);
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
  // Locked characters come last (DESIGN 16.4): Bruderherz is free and moves up.
  if (ids.join(',') !== 'Declarer,NullPlayer,HeartBrother,Shirker,Tailoress,Gambler,RamschAunt') fail(`characters on the start screen: ${ids.join(', ')}`);
  for (const id of ['Shirker', 'Tailoress', 'Gambler', 'RamschAunt']) {
    const card = page.locator(`.char-card.is-locked[data-character="${id}"]`);
    if (!(await card.count())) fail(`${id} is not shown as locked`);
    const cond = (await card.textContent()) ?? '';
    if (!/Gewinne|Schlage/.test(cond)) fail(`locked ${id} without his condition: ${cond}`);
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
  log('start screen: seven characters, old pick of a removed one fell back to the Alleinspieler');
  await shoot(page, 'start-characters-new');
  await fillSeed(page, SEED);
  await page.click('[data-action="new-run"]');
  await page.waitForSelector('[data-testid="skat-choice"]');
  const backs = await page.$$eval('.card-back use', (uses) => [...new Set(uses.map((u) => u.getAttribute('href')))]);
  if (!backs.includes('#bk-heart_brother')) fail(`Bruderherz's back not shown: ${backs.join(', ')}`);
  if (backs.includes('#bk-distel')) fail(`the Distel still shows for Bruderherz: ${backs.join(', ')}`);
  log(`Bruderherz plays with his own back (${backs.join(', ')})`);
  await shoot(page, 'heart-brother-table');
  await context.close();
}

/**
 * The Tresen of 11.11.2026 (DESIGN 16.5) at 390 × 844, 360 × 740 and
 * 1280 × 800, with Groß and Sehr groß and in English, before the boss
 * (boss-tresen.json), with Groschen (zinken.json) and broke with debt:
 * the counter comes before the strip with the next pub in the page; the
 * strip is folded at first, names the boss in one line and unfolds his
 * details (aria-expanded, aria-controls) and folds again; every buy button
 * is at least 44 px high and those the core says no to carry a padlock and
 * a dashed edge; every cut text shows "mehr" and lies on a ware whose
 * cover opens the info card; without scrolling the whole first group of
 * wares and "Weiter" are in sight and nothing covers "Weiter".
 */
/**
 * Die Papier-Ebene (DESIGN 16.6; alone with SMOKE_ONLY=paper): the roles'
 * paper steps have 4.5:1 on every paper tone; the score slip of the Faktor
 * game (nine lines and the payout) fits without scrolling at 390 × 844,
 * 360 × 740, 1280 × 800, with "Groß", "Sehr groß" and in English (see
 * assertScoreSlip); info card, the Wirt's bubble at the Tresen, his hint
 * and a Stammgast's quip lie on the paper token, and so do the cards of
 * the city map.
 */
async function paperPass(browser, errors) {
  const SMALL = { width: 360, height: 740 };
  const paperOf = (page, sel) =>
    page.evaluate((q) => {
      const el = document.querySelector(q);
      if (!el) return { missing: q };
      const probe = document.createElement('i');
      probe.style.color = 'var(--paper)';
      document.body.append(probe);
      const paper = getComputedStyle(probe).color;
      probe.remove();
      return { paper, bg: getComputedStyle(el).backgroundColor };
    }, sel);
  const onPaper = async (page, sel, label) => {
    const r = await paperOf(page, sel);
    if (r.missing) fail(`paper: ${label}: ${sel} missing`);
    if (r.bg !== r.paper) fail(`paper: ${label} is ${r.bg}, not the paper token ${r.paper}`);
  };
  const open = async (size, settings = {}, locale = 'de-DE') => {
    const context = await quietContext(browser, { viewport: size, reducedMotion: 'reduce', locale });
    await context.addInitScript((st) => {
      if (sessionStorage.getItem('smoke-paper')) return;
      sessionStorage.setItem('smoke-paper', '1');
      localStorage.setItem('stammtisch.tutorial', 'declined');
      localStorage.setItem('stammtisch.settings', JSON.stringify({ sound: false, music: false, hints: false, ...st }));
    }, settings);
    const page = await context.newPage();
    watch(page, errors);
    return page;
  };

  // The tokens themselves.
  {
    const page = await open(PHONE);
    await page.goto(base);
    const pairs = await page.evaluate(() => {
      const cs = getComputedStyle(document.documentElement);
      const hex = (n) => cs.getPropertyValue(n).trim();
      const lum = (h) => [1, 3, 5].map((i) => parseInt(h.slice(i, i + 2), 16) / 255).map((v) => (v <= 0.03928 ? v / 12.92 : ((v + 0.055) / 1.055) ** 2.4)).reduce((a, v, i) => a + v * [0.2126, 0.7152, 0.0722][i], 0);
      const ratio = (a, b) => (Math.max(lum(a), lum(b)) + 0.05) / (Math.min(lum(a), lum(b)) + 0.05);
      const out = [];
      for (const ink of ['--paper-ink', '--paper-ink-soft', '--paper-gain', '--paper-money', '--paper-danger', '--paper-wine', '--paper-teal']) {
        for (const ground of ['--paper', '--paper-light', '--paper-dark', '--paper-map']) out.push([ink, ground, ratio(hex(ink), hex(ground))]);
      }
      return out;
    });
    const low = pairs.filter(([, , r]) => r < 4.5);
    if (low.length) fail(`paper: role inks below 4.5:1: ${low.map(([a, b, r]) => `${a} on ${b} ${r.toFixed(2)}`).join(', ')}`);
    log(`paper: ${pairs.length} ink/paper pairs, lowest ${Math.min(...pairs.map((p) => p[2])).toFixed(2)}:1`);
    await page.context().close();
  }

  // The longest slip: the Faktor game with its payout, at every size.
  for (const [size, settings, locale] of [
    [PHONE, {}, 'de-DE'],
    [SMALL, {}, 'de-DE'],
    [DESKTOP, {}, 'de-DE'],
    [PHONE, { textSize: 'large' }, 'de-DE'],
    [PHONE, { textSize: 'xlarge' }, 'de-DE'],
    [SMALL, { textSize: 'xlarge' }, 'de-DE'],
    [PHONE, {}, 'en-GB'],
  ]) {
    const tag = `paper ${size.width}x${size.height} ${settings.textSize ?? 'normal'} ${locale}`;
    const page = await open(size, settings, locale);
    await resumeFixture(page, 'factor-win.json');
    await page.waitForSelector('[data-action="play-hand"]');
    await page.click('[data-action="play-hand"]');
    await page.locator('[data-action="pick-game"]', { has: page.locator('.game-base', { hasText: /^9$/ }) }).click();
    await page.click('[data-action="declare"]');
    await page.waitForSelector('.declare', { state: 'detached' });
    await playOut(page, tag);
    if (!(await page.locator('.payout').count())) fail(`${tag}: no payout on the Faktor slip`);
    // A bill longer than its room scrolls in itself and says so.
    const list = await page.evaluate(() => {
      const l = document.querySelector('.score-sheet .steps');
      const more = [...document.querySelectorAll('.steps-more')].filter((e) => !e.hidden).length;
      return { scrolls: l.scrollHeight > l.clientHeight + 1, more, tab: l.tabIndex };
    });
    if (list.scrolls && (!list.more || list.tab !== 0)) fail(`${tag}: a scrolling bill without "more lines" or Tab (${JSON.stringify(list)})`);
    await page.screenshot({ path: join(screens, `paper-score-${size.width}x${size.height}-${settings.textSize ?? 'normal'}-${locale.slice(0, 2)}.png`) });
    log(`${tag}: slip fits, bill ${list.scrolls ? 'scrolls in itself' : 'whole'}`);
    await page.context().close();
  }

  // Info card and the Wirt's bubble at the Tresen.
  {
    const page = await open(PHONE);
    await resumeFixture(page, 'zinken.json');
    await page.waitForSelector('[data-testid="shop"]');
    await page.mouse.move(2, 2);
    await onPaper(page, '.shop .bubble', 'the Wirt at the Tresen');
    await page.locator('[data-action="info-offer-item"]').first().click();
    await page.waitForSelector('[data-testid="info-card"]');
    await onPaper(page, '[data-testid="info-card"]', 'the info card');
    const kaufen = await page.evaluate(() => {
      const b = document.querySelector('[data-testid="info-card"] .btn-primary');
      return b ? getComputedStyle(b).backgroundColor : null;
    });
    const ivory = await page.evaluate(() => {
      const probe = document.createElement('i');
      probe.style.color = 'var(--ivory)';
      document.body.append(probe);
      const c = getComputedStyle(probe).color;
      probe.remove();
      return c;
    });
    if (kaufen && kaufen === ivory) fail('paper: the ivory button on the info card is not inverted');
    await page.context().close();
  }
  // The Wirt's hint and a Stammgast's quip.
  {
    const page = await open(PHONE, { hints: true });
    await page.goto(base);
    await fillSeed(page, SEED);
    await page.click('[data-action="new-run"]');
    await page.waitForSelector('[data-testid="hint"]', { timeout: 8000 }).catch(() => fail('paper: no hint of the Wirt'));
    await onPaper(page, '[data-testid="hint"] .hint-bubble', 'the Wirt\'s hint');
    await page.context().close();
    const q = await open(PHONE);
    await q.goto(base);
    await fillSeed(q, SEED);
    await q.click('[data-action="new-run"]');
    await q.waitForSelector('[data-testid="quip"]', { timeout: 8000 }).catch(() => fail('paper: no quip at the first game'));
    await onPaper(q, '[data-testid="quip"]', 'a Stammgast\'s quip');
    await q.context().close();
  }
  // The city map's card of the next stop and its legend.
  {
    const page = await open(PHONE);
    await resumeFixture(page, 'route-spelunke.json');
    await page.waitForSelector('[data-testid="route-next"]');
    await onPaper(page, '[data-testid="route-next"]', 'the next stop on the map');
    await onPaper(page, '.map-legend', 'the map legend');
    await page.context().close();
  }
  log('paper: slip, info card, bubbles and map on the paper token');
}

async function tresenPass(browser, errors) {
  const rich = patched('zinken.json', (save) => (save.run.coins = 20));
  const broke = patched('zinken.json', (save) => {
    save.run.coins = 1;
    save.run.debt = 4;
  });
  const cases = [
    { size: PHONE, text: 'normal', lang: 'de', save: 'boss-tresen.json', raw: fixture('boss-tresen.json') },
    { size: PHONE, text: 'normal', lang: 'de', save: 'tresen-rich.json', raw: rich },
    { size: PHONE, text: 'normal', lang: 'de', save: 'tresen-broke.json', raw: broke, broke: true },
    { size: { width: 360, height: 740 }, text: 'normal', lang: 'de', save: 'tresen-rich.json', raw: rich },
    { size: DESKTOP, text: 'normal', lang: 'de', save: 'boss-tresen.json', raw: fixture('boss-tresen.json') },
    { size: PHONE, text: 'large', lang: 'de', save: 'tresen-rich.json', raw: rich },
    { size: PHONE, text: 'xlarge', lang: 'de', save: 'tresen-rich.json', raw: rich },
    { size: { width: 360, height: 740 }, text: 'xlarge', lang: 'de', save: 'boss-tresen.json', raw: fixture('boss-tresen.json') },
    { size: PHONE, text: 'normal', lang: 'en', save: 'tresen-rich.json', raw: rich },
  ];
  for (const c of cases) {
    const tag = `Tresen ${c.size.width}x${c.size.height} ${c.text} ${c.lang} ${c.save}`;
    const context = await quietContext(browser, { viewport: c.size, reducedMotion: 'reduce', locale: c.lang === 'en' ? 'en-GB' : 'de-DE' });
    await context.addInitScript(([text, lang]) => {
      if (sessionStorage.getItem('smoke-tresen')) return;
      sessionStorage.setItem('smoke-tresen', '1');
      localStorage.setItem('stammtisch.tutorial', 'declined');
      localStorage.setItem('stammtisch.lang', lang);
      localStorage.setItem('stammtisch.settings', JSON.stringify({ sound: false, music: false, hints: false, textSize: text }));
    }, [c.text, c.lang]);
    const page = await context.newPage();
    watch(page, errors);
    await resumeFixture(page, c.save, c.raw);
    await page.waitForSelector('[data-testid="shop"]');
    await page.mouse.move(2, 2);
    await page.waitForTimeout(250);
    // Wares first: the counter comes before the preview in the page.
    const order = await page.evaluate(() => {
      const counter = document.querySelector('[data-testid="counter"]');
      const next = document.querySelector('[data-testid="next-series"]');
      return counter && next ? Boolean(counter.compareDocumentPosition(next) & Node.DOCUMENT_POSITION_FOLLOWING) : null;
    });
    if (order !== true) fail(`${tag}: the preview of the next pub does not come after the wares (${order})`);
    // The strip: folded, one line with the boss, unfolds and folds.
    const toggle = page.locator('[data-testid="next-toggle"]');
    if (!(await toggle.count())) fail(`${tag}: no strip with the next pub`);
    if ((await toggle.getAttribute('aria-expanded')) !== 'false') fail(`${tag}: the strip is not folded at first`);
    if ((await toggle.getAttribute('aria-controls')) !== 'next-details') fail(`${tag}: the strip does not name its details`);
    if (await page.locator('#next-details').isVisible()) fail(`${tag}: the details of the next pub show while folded`);
    const tb = await toggle.boundingBox();
    if (!tb || tb.height < 44) fail(`${tag}: the strip is only ${tb?.height}px high`);
    if (c.save === 'boss-tresen.json') {
      const boss = c.lang === 'en' ? 'The Twins' : 'Die Zwillinge';
      if (!(await page.locator('.next-strip.is-boss').count())) fail(`${tag}: the strip does not mark the boss pub`);
      const line = ((await toggle.textContent()) ?? '').replace(/\s+/g, ' ');
      if (!line.includes(boss)) fail(`${tag}: the strip does not name the boss: ${line}`);
    }
    await toggle.click();
    await page.waitForSelector('#next-details', { state: 'visible', timeout: 3000 }).catch(() => fail(`${tag}: the strip does not unfold`));
    if ((await page.locator('[data-testid="next-toggle"]').getAttribute('aria-expanded')) !== 'true') fail(`${tag}: aria-expanded stays false when unfolded`);
    if (c.save === 'boss-tresen.json' && !(await page.locator('#next-details [data-testid="next-boss-figures"]').isVisible())) fail(`${tag}: the unfolded strip shows no boss figures`);
    await page.click('[data-testid="next-toggle"]');
    await page.waitForSelector('#next-details', { state: 'hidden', timeout: 3000 }).catch(() => fail(`${tag}: the strip does not fold again`));
    // Measured unscrolled.
    const m = await page.evaluate(() => {
      const shop = document.querySelector('.run > .shop');
      if (shop) shop.scrollTop = 0;
      const rect = (el) => el.getBoundingClientRect();
      const buys = [...document.querySelectorAll('.shop .btn-buy, .shop .shop-bar .btn, .shop [data-action="sell-relic"], .shop [data-action="sell-item"], .shop .service .btn')].map((el) => ({
        action: el.dataset.action ?? el.className,
        height: rect(el).height,
        disabled: el.disabled,
        lock: Boolean(el.querySelector('.buy-lock')),
        dashed: getComputedStyle(el).borderTopStyle === 'dashed',
        buy: el.classList.contains('btn-buy') && el.dataset.action !== 'replace-relic-pick',
      }));
      const cut = [...document.querySelectorAll('.shop [data-clamp]')]
        .filter((el) => el.scrollHeight > el.clientHeight + 1)
        .map((el) => {
          const ware = el.parentElement;
          const more = ware?.querySelector(':scope > .more-tag');
          const cover = ware?.querySelector(':scope > .coaster-cover[data-info], :scope > .good-cover[data-info]');
          return { text: el.textContent.slice(0, 30), more: Boolean(more && more.offsetHeight > 0), cover: cover?.getAttribute('data-info') ?? null };
        });
      const group = document.querySelector('[data-testid="relic-offers"]');
      const bar = document.querySelector('.shop-bar');
      const leave = document.querySelector('[data-action="leave-shop"]');
      const lb = rect(leave);
      const hit = document.elementFromPoint(lb.left + lb.width / 2, lb.top + lb.height / 2);
      return { buys, cut, groupTop: rect(group).top, groupBottom: rect(group).bottom, barTop: rect(bar).top, leaveTop: lb.top, leaveBottom: lb.bottom, vh: innerHeight, leaveOnTop: hit === leave || leave.contains(hit) };
    });
    for (const b of m.buys) {
      if (b.height < 43.5) fail(`${tag}: "${b.action}" is only ${Math.round(b.height)}px high`);
      if (b.buy && b.disabled && (!b.lock || !b.dashed)) fail(`${tag}: a buy button the core says no to has no padlock or dashed edge`);
      if (b.buy && !b.disabled && b.lock) fail(`${tag}: a buy button that works carries a padlock`);
    }
    if (c.broke && !m.buys.some((b) => b.buy && b.disabled)) fail(`${tag}: broke, yet every buy button works`);
    for (const x of m.cut) {
      if (!x.more || !x.cover) fail(`${tag}: cut text "${x.text}…" without "mehr" or an info card (${JSON.stringify(x)})`);
    }
    // A cut text opens the full text with one tap on its ware.
    const cutCover = m.cut.find((x) => x.cover)?.cover;
    if (cutCover) {
      await page.locator(`.shop [data-info="${cutCover}"]`).first().click();
      await page.waitForSelector('[data-testid="info-card"]', { timeout: 3000 }).catch(() => fail(`${tag}: a tap on a cut ware opens no info card`));
      await page.keyboard.press('Escape');
      await page.waitForSelector('[data-testid="info-card"]', { state: 'detached', timeout: 3000 }).catch(() => {});
    }
    if (m.groupTop < 0 || m.groupBottom > m.barTop + 0.5) fail(`${tag}: the first group of wares is not whole in sight without scrolling (${Math.round(m.groupTop)}..${Math.round(m.groupBottom)}, bar at ${Math.round(m.barTop)})`);
    if (m.leaveTop < 0 || m.leaveBottom > m.vh + 0.5) fail(`${tag}: "Weiter" is not in sight without scrolling (${Math.round(m.leaveTop)}..${Math.round(m.leaveBottom)} of ${m.vh})`);
    if (!m.leaveOnTop) fail(`${tag}: something covers "Weiter"`);
    await assertNoSideScroll(page, tag);
    await page.screenshot({ path: join(screens, `tresen-${c.save.replace('.json', '')}-${c.text}-${c.lang}-${c.size.width}x${c.size.height}.png`) });
    await context.close();
    log(`${tag}: wares first, strip folds, ${m.buys.length} buttons ≥ 44 px, ${m.cut.length} cut texts with "mehr"`);
  }
}

/**
 * The start screen of 11.11.2026 (DESIGN 16.4) at 390 × 844, 360 × 740 and
 * 1280 × 800, with the text sizes Groß and Sehr groß and in English: the
 * locked characters stand after every free one; "Neuer Run als …" is
 * visible at the bottom edge without scrolling, nothing covers it, and at
 * the end of the page it covers nothing; "Weitere Spielarten" unfolds the
 * daily run, the seed and the Einführung (aria-expanded) and folds them
 * again, the weekly challenge stays on the first level; "mehr" unfolds the
 * picked character's text (alone with SMOKE_ONLY=characters).
 */
async function startScreenPass(browser, errors) {
  const meta = { version: 1, meta: { version: 1, unlocked: ['tour_won', 'true_colours'], stats: { runs_finished: 3, runs_won: 1, best_section: 4, characters: { Declarer: { runs: 3, won: 1 } } } } };
  const cases = [
    { size: PHONE, text: 'normal', lang: 'de' },
    { size: { width: 360, height: 740 }, text: 'normal', lang: 'de' },
    { size: DESKTOP, text: 'normal', lang: 'de' },
    { size: PHONE, text: 'large', lang: 'de' },
    { size: { width: 360, height: 740 }, text: 'xlarge', lang: 'de' },
    { size: { width: 360, height: 740 }, text: 'normal', lang: 'en' },
  ];
  for (const c of cases) {
    const tag = `start ${c.size.width}x${c.size.height} ${c.text} ${c.lang}`;
    const context = await quietContext(browser, { viewport: c.size, reducedMotion: 'reduce', locale: c.lang === 'en' ? 'en-GB' : 'de-DE' });
    await context.addInitScript(([m, text, lang]) => {
      if (sessionStorage.getItem('smoke-start')) return;
      sessionStorage.setItem('smoke-start', '1');
      localStorage.setItem('stammtisch.meta', m);
      localStorage.setItem('stammtisch.tutorial', 'declined');
      localStorage.setItem('stammtisch.lang', lang);
      localStorage.setItem('stammtisch.settings', JSON.stringify({ sound: false, music: false, textSize: text }));
    }, [JSON.stringify(meta), c.text, c.lang]);
    const page = await context.newPage();
    watch(page, errors);
    await page.goto(base);
    await page.waitForSelector('[data-testid="characters"]');
    // Locked last.
    const row = await page.$$eval('[data-testid="characters"] .char-card', (cards) => cards.map((el) => (el.classList.contains('is-locked') ? 'L' : 'U')).join(''));
    if (!/^U+L+$/.test(row)) fail(`${tag}: locked characters are not at the end of the row: ${row}`);
    // The main button: on screen at the bottom without scrolling, not covered.
    const dock = await page.evaluate(() => {
      const screen = document.querySelector('.screen');
      const btn = document.querySelector('.start-dock [data-action="new-run"]');
      if (!screen || !btn) return null;
      screen.scrollTop = 0;
      const b = btn.getBoundingClientRect();
      const hit = document.elementFromPoint(b.left + b.width / 2, b.top + b.height / 2);
      return { top: b.top, bottom: b.bottom, height: b.height, vh: innerHeight, scrolls: screen.scrollHeight > screen.clientHeight + 1, onTop: hit === btn || btn.contains(hit) };
    });
    if (!dock) fail(`${tag}: no main button in the dock`);
    if (!dock.scrolls) fail(`${tag}: the start screen does not scroll, the dock test proves nothing`);
    if (dock.top < 0 || dock.bottom > dock.vh + 0.5 || dock.bottom < dock.vh - 40) fail(`${tag}: main button not at the bottom edge: ${Math.round(dock.top)}..${Math.round(dock.bottom)} of ${dock.vh}`);
    if (dock.height < 44) fail(`${tag}: main button only ${dock.height}px high`);
    if (!dock.onTop) fail(`${tag}: something covers the main button`);
    // At the end of the page the dock covers nothing: the last line (build) stays above it.
    const end = await page.evaluate(() => {
      const screen = document.querySelector('.screen');
      screen.scrollTop = screen.scrollHeight;
      const d = document.querySelector('.start-dock').getBoundingClientRect();
      const last = document.querySelector('[data-testid="build"]').getBoundingClientRect();
      return { dockTop: d.top, lastBottom: last.bottom };
    });
    if (end.lastBottom > end.dockTop + 0.5) fail(`${tag}: the dock covers the end of the page (${Math.round(end.lastBottom)} > ${Math.round(end.dockTop)})`);
    await page.evaluate(() => { document.querySelector('.screen').scrollTop = 0; });
    // Weitere Spielarten: folded, weekly visible; unfold, fold.
    const toggle = page.locator('[data-testid="start-more-toggle"]');
    if ((await toggle.getAttribute('aria-expanded')) !== 'false') fail(`${tag}: "Weitere Spielarten" is not folded at first`);
    if (await page.locator('#seed').isVisible()) fail(`${tag}: the seed field shows on the first level`);
    if (await page.locator('[data-testid="daily"]').isVisible()) fail(`${tag}: the daily block shows on the first level`);
    if (!(await page.locator('[data-testid="weekly"]').count())) fail(`${tag}: no weekly challenge on the first level`);
    if (await page.locator('#start-more-body [data-testid="weekly"]').count()) fail(`${tag}: the weekly challenge is folded away`);
    const tb = await toggle.boundingBox();
    if (!tb || tb.height < 44) fail(`${tag}: the toggle is ${tb?.height}px high`);
    await toggle.click();
    await page.waitForSelector('#seed', { state: 'visible' });
    if ((await toggle.getAttribute('aria-expanded')) !== 'true') fail(`${tag}: aria-expanded not true after unfolding`);
    for (const sel of ['[data-testid="daily"]', '[data-testid="tutorial-start"]']) if (!(await page.locator(sel).isVisible())) fail(`${tag}: ${sel} not shown after unfolding`);
    if (c.size === PHONE && c.text === 'normal') await shoot(page, 'start-more-open');
    await assertNoSideways(page, `${tag}: unfolded`);
    // The typed seed survives folding (the field stays in the page).
    await page.fill('#seed', '4711');
    await page.locator('[data-testid="start-more-toggle"]').click();
    await page.waitForSelector('#seed', { state: 'hidden' });
    if ((await page.locator('[data-testid="start-more-toggle"]').getAttribute('aria-expanded')) !== 'false') fail(`${tag}: aria-expanded not false after folding`);
    if ((await page.inputValue('#seed')) !== '4711') fail(`${tag}: the typed seed was lost on folding`);
    // "mehr": the Nullspieler's text is long enough to fold.
    await page.click('[data-action="pick-character"][data-character="NullPlayer"]');
    const more = page.locator('[data-testid="char-more"]');
    await more.waitFor();
    const folded = await page.$eval('[data-testid="char-blurb"]', (el) => ({ h: el.clientHeight, full: el.scrollHeight, lh: parseFloat(getComputedStyle(el).lineHeight) }));
    if (folded.full <= folded.h + 1) fail(`${tag}: the Nullspieler's text is not folded (${folded.h} of ${folded.full}px)`);
    if (folded.h > folded.lh * 2 + 2) fail(`${tag}: the folded text takes more than two lines (${folded.h}px, line ${folded.lh}px)`);
    if (!(await more.isVisible())) fail(`${tag}: no "mehr" under a folded text`);
    const mb = await more.boundingBox();
    if (!mb || mb.height < 44) fail(`${tag}: "mehr" is ${mb?.height}px high`);
    await more.click();
    await page.waitForSelector('[data-testid="char-blurb"].is-open');
    if ((await page.locator('[data-testid="char-more"]').getAttribute('aria-expanded')) !== 'true') fail(`${tag}: "mehr" without aria-expanded`);
    const open = await page.$eval('[data-testid="char-blurb"]', (el) => el.scrollHeight - el.clientHeight);
    if (open > 1) fail(`${tag}: the unfolded text is still cut by ${open}px`);
    await assertNoSideways(page, tag);
    log(`${tag}: locked last (${row}), main button at ${Math.round(dock.top)}..${Math.round(dock.bottom)}, both toggles work`);
    await context.close();
  }
}

/**
 * The characters of 10.11.2026 on phone and desktop, in German and in
 * English (ADR 0023): Die Ramschtante (Aunt Ramsch) sees "Ramsch spielen"
 * before the skat and plays it, Der Zocker (The Gambler) sees how often
 * the skat may still be picked up; both with their own back, names and
 * texts in the language, and with "Farbenblind-Hilfe" on every face of the
 * hand keeps its suit letter (A L H B in English; alone with SMOKE_ONLY=characters).
 */
async function newCharactersPass(browser, errors) {
  const LANG = {
    de: { names: { RamschAunt: 'Die Ramschtante', Gambler: 'Der Zocker' }, ramsch: 'Ramsch spielen', pickups: /Kneipe noch 1× aufnehmen/ },
    en: { names: { RamschAunt: 'Aunt Ramsch', Gambler: 'The Gambler' }, ramsch: 'Play Ramsch', pickups: /1× more in this pub/ },
  };
  // `shoot` takes every picture at 390 × 844 and 1280 × 800.
  for (const lang of ['de', 'en']) {
    const tag = lang;
    const context = await quietContext(browser, { viewport: PHONE, reducedMotion: 'reduce', locale: lang === 'en' ? 'en-GB' : 'de-DE' });
    const page = await context.newPage();
    watch(page, errors);
    await page.goto(base);
    await page.waitForSelector('[data-testid="characters"]');
    await page.evaluate((lang) => {
      localStorage.setItem('stammtisch.meta', JSON.stringify({ version: 1, meta: { version: 1, unlocked: ['beat_ramscher', 'big_hand'] } }));
      localStorage.setItem('stammtisch.lang', lang);
      localStorage.setItem('stammtisch.settings', JSON.stringify({ colorBlind: true }));
      localStorage.removeItem('stammtisch.run');
    }, lang);
    await page.reload();
    await page.waitForFunction((lang) => document.documentElement.lang === lang, lang);
    for (const [id, back] of [['RamschAunt', 'ramsch_aunt'], ['Gambler', 'gambler']]) {
      if (await page.locator(`.char-card.is-locked[data-character="${id}"]`).count()) fail(`${tag}: ${id} should be unlocked`);
      await page.click(`[data-action="pick-character"][data-character="${id}"]`);
      await page.waitForSelector('[data-testid="char-detail"]');
      const detail = (await page.textContent('[data-testid="char-detail"]')) ?? '';
      if (!detail.includes(LANG[lang].names[id])) fail(`${tag}: the detail of ${id} does not name ${LANG[lang].names[id]}: ${detail}`);
      await shoot(page, `${back}-detail-${lang}`);
      await fillSeed(page, SEED);
      await page.click('[data-action="new-run"]');
      await page.waitForSelector('[data-testid="skat-choice"]');
      const backs = await page.$$eval('.card-back use', (uses) => [...new Set(uses.map((u) => u.getAttribute('href')))]);
      if (!backs.includes(`#bk-${back}`)) fail(`${tag}: ${id}'s back not shown: ${backs.join(', ')}`);
      // "Farbenblind-Hilfe": every settled face of the hand shows its suit letter in the language.
      const letters = await page.$$eval('.hand .card-svg[data-suit]', (svgs) =>
        svgs.map((svg) => {
          const use = svg.querySelector('use.face-letter');
          return { href: use?.getAttribute('href') ?? '', shown: use ? getComputedStyle(use).display !== 'none' : false };
        }),
      );
      const want = lang === 'en' ? /^#cb-en-[CSHD]$/ : /^#cb-[CSHD]$/;
      if (!letters.length || letters.some((l) => !l.shown || !want.test(l.href))) fail(`${tag}: ${id}'s suit letters: ${JSON.stringify(letters)}`);
      if (id === 'RamschAunt') {
        await page.waitForSelector('[data-testid="declare-ramsch"]');
        const button = (await page.textContent('[data-testid="declare-ramsch"]')) ?? '';
        if (!button.includes(LANG[lang].ramsch)) fail(`${tag}: the Ramsch button says ${button}`);
        await shoot(page, `ramsch-aunt-choice-${lang}`);
        await page.click('[data-testid="declare-ramsch"]');
        await page.waitForFunction(() => !document.querySelector('[data-testid="skat-choice"]'));
        log(`${tag}: the Ramschtante plays a Ramsch from the choice of skat or hand`);
      } else {
        await page.waitForSelector('[data-testid="skat-pickups"]');
        const pickups = (await page.textContent('[data-testid="skat-pickups"]')) ?? '';
        if (!LANG[lang].pickups.test(pickups)) fail(`${tag}: the Zocker's pickups say ${pickups}`);
        if (await page.locator('[data-testid="declare-ramsch"]').count()) fail(`${tag}: the Zocker may not Ramsch`);
        await shoot(page, `gambler-choice-${lang}`);
        log(`${tag}: the Zocker sees how often the skat may be picked up`);
      }
      await page.evaluate(() => localStorage.removeItem('stammtisch.run'));
      await page.goto(base);
      await page.waitForSelector('[data-testid="characters"]');
    }
    await context.close();
  }
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
  // The trail in dark ink (11.11.2026; the accent was copper), not white (playtest 05.11.2026).
  const trailFill = await page2.evaluate(() => document.querySelector('.map-trail')?.getAttribute('fill'));
  if (trailFill?.toLowerCase() !== '#1b1e21') fail(`travel: the trail is drawn in ${trailFill}, not dark ink`);
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

/**
 * Die Einführung (DESIGN 16.1; alone with SMOKE_ONLY=tutorial): a new
 * player is offered it on the start screen, plays the guided pub with the
 * fixed deals of tutorial.ron as the Wirt suggests (Eichel after the skat
 * pressing both Schellen, Null pressing both Daus, Grand Hand), every
 * coach bubble pointing at something on screen and inside the viewport,
 * then the card reward, the way and the Tresen are explained and "Zum
 * Start" ends it. Nothing of it is stored: no run, no history, the meta
 * state unchanged. On the phone a run in progress is stored first; the
 * Einführung, skipped half-way, leaves it untouched.
 */
async function tutorialPass(browser, errors, viewport, shotDir = null, locale = 'de-DE') {
  const tag = `${viewport.width}x${viewport.height}${locale === 'de-DE' ? '' : ` ${locale}`}`;
  const context = await quietContext(browser, { viewport, reducedMotion: 'reduce', deviceScaleFactor: shotDir ? 2 : 1, locale });
  const page = await context.newPage();
  watch(page, errors);
  const snap = async (name) => {
    if (!shotDir) return;
    await page.waitForTimeout(250);
    await page.screenshot({ path: join(shotDir, `${name}-${tag}.png`) });
    log(`tutorial ${tag}: screenshot ${name}`);
  };
  const keep = async () =>
    page.evaluate(() => {
      const out = {};
      for (let i = 0; i < localStorage.length; i++) {
        const k = localStorage.key(i);
        // stammtisch-pixels: the cached portraits (ui/pixelate.ts), stammtisch-diag.*: the diagnosis
        // buffer for bug reports (diag.ts, ADR 0027); no part of the save.
        if (k && k !== 'stammtisch.tutorial' && k !== 'stammtisch.settings' && k !== 'stammtisch-pixels' && !k.startsWith('stammtisch-diag.')) out[k] = localStorage.getItem(k);
      }
      return out;
    });
  await page.goto(base);
  await page.waitForSelector('[data-testid="tutorial-offer"]', { timeout: 10000 }).catch(() => fail(`tutorial ${tag}: no offer for a new player`));
  const before = await keep();
  await page.click('[data-testid="tutorial-offer-play"]');
  await page.waitForSelector('[data-testid="coach"]', { timeout: 10000 }).catch(() => fail(`tutorial ${tag}: no coach after starting`));
  const seen = [];
  const shots = { goal: 'tutorial-goal', press: 'tutorial-press', score: 'tutorial-score', evening: 'tutorial-evening', trump: 'tutorial-trump' };
  for (let i = 0; i < 400; i++) {
    await page.waitForTimeout(120);
    const coach = page.locator('[data-testid="coach"]');
    const step = (await coach.count()) ? await coach.getAttribute('data-step') : null;
    if (step && seen[seen.length - 1] !== step) {
      seen.push(step);
      // The bubble is inside the viewport; a ring sits on what it talks about.
      const box = await page.locator('.coach-main').boundingBox();
      if (!box || box.y < 0 || box.y + box.height > viewport.height + 1 || box.x < 0 || box.x + box.width > viewport.width + 1) fail(`tutorial ${tag}: bubble "${step}" leaves the viewport: ${JSON.stringify(box)}`);
      const ring = await page.locator('.coach-ring').boundingBox().catch(() => null);
      if (ring && box.y < ring.y + ring.height - 6 && box.y + box.height > ring.y + 6 && step !== 'evening') {
        // A bubble over its own target would hide it: only allowed when there is no room.
        log(`tutorial ${tag}: note: bubble "${step}" overlaps its ring`);
      }
      const text = ((await page.locator('.coach-text').textContent()) ?? '').trim();
      if (text.length < 20) fail(`tutorial ${tag}: step "${step}" has no text`);
      if (locale !== 'de-DE' && GERMAN_WORDS.test(text)) fail(`tutorial ${tag}: step "${step}" speaks German: ${text.slice(0, 80)}`);
      if (shots[step]) await snap(shots[step]);
    }
    const sheet = page.locator('[data-action="score-continue"]');
    if (step === 'evening') break;
    const kind = step ? await coach.getAttribute('data-kind') : null;
    if (kind === 'info') {
      if (viewport.width >= 1000) {
        // On the desktop by keyboard: Space goes on with the Wirt and
        // presses nothing else (keys.ts would press the main button).
        await page.evaluate(() => document.activeElement instanceof HTMLElement && document.activeElement.blur());
        const screen = await page.evaluate(() => document.querySelector('.run')?.className ?? '');
        await page.keyboard.press(' ');
        await page.waitForFunction((was) => document.querySelector('[data-testid="coach"]')?.getAttribute('data-step') !== was, step, { timeout: 5000 }).catch(() => fail(`tutorial ${tag}: Space did not go on from "${step}"`));
        const now = await page.evaluate(() => document.querySelector('.run')?.className ?? '');
        if (step === 'goal' && now !== screen) fail(`tutorial ${tag}: Space on "${step}" changed the screen too`);
      } else {
        await page.click('[data-testid="coach-next"]');
      }
      continue;
    }
    if (step === 'skat' || step === 'nullSkat') {
      await page.click('[data-testid="skat-pile"]');
      await page.waitForSelector('[data-testid="skat-zone"]');
      continue;
    }
    if (step === 'press') {
      // Eichel-Zehn back to the hand, Schellen-8 onto the table: Schellen 7 and 8 pressed.
      await page.click('.skat-card[data-id="27"]');
      await page.click('.hand-card[data-id="1"]');
      await page.click('[data-testid="discard"]:not([disabled])');
      continue;
    }
    if (step === 'nullPress' || step === 'pressGeneric') {
      await page.click('[data-testid="discard"]:not([disabled])');
      continue;
    }
    if (step === 'pickClubs' || step === 'grandPick' || step === 'nullPick') {
      const sel = await page.evaluate(() => {
        const ring = document.querySelector('.coach-ring');
        const r = ring?.getBoundingClientRect();
        if (!r) return null;
        const el = document.elementFromPoint(r.left + r.width / 2, r.top + r.height / 2)?.closest('.game-btn');
        return el ? el.getAttribute('data-index') : null;
      });
      if (sel === null) fail(`tutorial ${tag}: the ring of "${step}" is not on a Spielart`);
      await page.click(`.game-btn[data-index="${sel}"]`);
      continue;
    }
    if (step === 'declareGo') {
      await page.click('[data-action="declare"]:not([disabled]), [data-action="declare-null"]:not([disabled])');
      continue;
    }
    if (step === 'grandHand') {
      await page.click('[data-action="play-hand"]');
      continue;
    }
    // No step to act on: play on as the table asks.
    if (await sheet.count()) {
      // The Wirt's word on the board comes with it: give it a moment.
      await page.waitForTimeout(300);
      if (await page.locator('[data-testid="coach"]').count()) continue;
      if (await page.locator('[data-action="score-skip"].btn').count()) await page.locator('[data-action="score-skip"].btn').click();
      await page.click('[data-action="score-continue"]');
      continue;
    }
    if (await page.locator('[data-testid="card-reward"]').count()) {
      if (!seen.includes('pubWon')) fail(`tutorial ${tag}: the card reward came without the Wirt's word on Zinken`);
      await takeCardReward(page);
      continue;
    }
    const declare = page.locator('[data-action="declare"]:not([disabled]), [data-action="declare-null"]:not([disabled])');
    if (await declare.count()) {
      if (await page.locator('[data-testid="coach"]').count()) continue;
      await declare.first().click();
      continue;
    }
    const play = page.locator('.hand [data-action="play"]');
    if (await play.count()) {
      // The table and the Wirt come back together: let him speak first.
      if (await page.locator('[data-testid="coach"]').count()) continue;
      await play.first().click();
      continue;
    }
  }
  const lessons = ['goal', 'hand', 'skat', 'pressWhy', 'press', 'ansage', 'pickClubs', 'declareGo', 'trump', 'follow', 'augen', 'value', 'score', 'nullIntro', 'nullPress', 'nullPick', 'nullPlay', 'nullScore', 'grandIntro', 'grandHand', 'grandPick', 'grandPlay', 'pubWon', 'route', 'tresen', 'evening'];
  const missing = lessons.filter((id) => !seen.includes(id));
  if (missing.length) fail(`tutorial ${tag}: lessons not shown: ${missing.join(', ')} (seen ${seen.join(', ')})`);
  log(`tutorial ${tag}: ${seen.length} steps: ${seen.join(', ')}`);
  // The Wirt's regular hints stay away during the Einführung.
  if (await page.locator('[data-testid="hint"]').count()) fail(`tutorial ${tag}: a regular hint over the coach`);
  await page.click('[data-testid="coach-finish-start"]');
  await page.waitForSelector('[data-testid="tutorial-start"]', { state: 'attached', timeout: 10000 }).catch(() => fail(`tutorial ${tag}: "Zum Start" did not lead to the start screen`));
  if (await page.locator('[data-testid="tutorial-offer"]').count()) fail(`tutorial ${tag}: the offer comes again after the Einführung`);
  if (await page.locator('[data-action="resume"]').count()) fail(`tutorial ${tag}: the Einführung left a run to resume`);
  const after = await keep();
  if (JSON.stringify(after) !== JSON.stringify(before)) fail(`tutorial ${tag}: storage changed: ${JSON.stringify(Object.keys(after))} vs ${JSON.stringify(Object.keys(before))}`);
  log(`tutorial ${tag}: played through, nothing stored`);

  if (shotDir || viewport.width < 1000) {
    // A run in progress survives the Einführung.
    await page.click('[data-action="new-run"]');
    await page.waitForSelector('.declare');
    await page.goto(base);
    await page.waitForSelector('[data-action="resume"]');
    const run = await page.evaluate(() => localStorage.getItem('stammtisch.run'));
    await openStartMore(page);
    await page.click('[data-testid="tutorial-start"]');
    await page.waitForSelector('[data-testid="coach"]');
    await page.click('[data-testid="coach-next"]');
    await page.click('[data-testid="coach-skip"]');
    await page.waitForSelector('[data-action="resume"]', { timeout: 10000 }).catch(() => fail(`tutorial ${tag}: no resume after skipping`));
    const still = await page.evaluate(() => localStorage.getItem('stammtisch.run'));
    if (still !== run) fail(`tutorial ${tag}: the run in progress changed`);
    await page.click('[data-action="resume"]');
    await page.waitForSelector('.declare');
    if (await page.locator('[data-testid="coach"]').count()) fail(`tutorial ${tag}: the coach followed into the real run`);
    log(`tutorial ${tag}: skipped half-way, the run in progress stayed`);
  }
  await context.close();
}

/**
 * "Teilen" on the end screen (shareRunPass, 390 × 844 and 1280 × 800):
 * a won run (the last boss pub, patched from boss-table.json, with five
 * relics) draws its picture; without a share sheet it is saved as a PNG
 * of 1080 × 1350, with one the picture goes into it as a file.
 */
async function shareRunPass(browser, errors, viewport) {
  const tag = `${viewport.width}x${viewport.height}`;
  const context = await quietContext(browser, { viewport, reducedMotion: 'reduce', acceptDownloads: true });
  const page = await context.newPage();
  watch(page, errors);
  const won = JSON.parse(fixture('boss-table.json'));
  won.run.series.section = 2;
  won.run.series.target = 1;
  won.run.series.score = 1000;
  won.run.series.games_played = 2;
  won.run.relics.push('home_suit', 'all_rounder', 'old_deck', 'skat_keeper');
  await resumeFixture(page, null, JSON.stringify(won));
  await page.waitForSelector('.declare');
  await skatThenGrand(page, `share ${tag}`);
  await playOut(page, `share ${tag}`);
  await page.click('[data-action="score-continue"]');
  await page.waitForSelector('[data-testid="end"] h1.is-won', { timeout: 10000 });
  await page.waitForTimeout(300);
  while (await page.locator('.sheet-unlock').count()) {
    await page.locator('.sheet-unlock [data-action="close-overlay"]').last().click();
    await page.waitForTimeout(200);
  }
  const button = page.locator('[data-testid="share-run"]');
  if (!(await button.count())) fail(`share ${tag}: no "Teilen" on the end screen`);
  await button.scrollIntoViewIfNeeded();
  await page.screenshot({ path: join(screens, `qol-end-share-${tag}.png`) });
  // No share sheet here: the picture is saved.
  const hasShare = await page.evaluate(() => typeof navigator.share === 'function');
  if (!hasShare) {
    const [download] = await Promise.all([page.waitForEvent('download', { timeout: 10000 }), button.click()]);
    const file = join(screens, `qol-run-card-${tag}.png`);
    await download.saveAs(file);
    const png = readFileSync(file);
    const w = png.readUInt32BE(16);
    const h = png.readUInt32BE(20);
    if (png.toString('latin1', 1, 4) !== 'PNG' || w !== 1080 || h !== 1350) fail(`share ${tag}: saved picture ${download.suggestedFilename()} is ${w} × ${h}`);
    if (!/^stammtisch-.+\.png$/.test(download.suggestedFilename())) fail(`share ${tag}: picture named ${download.suggestedFilename()}`);
    log(`share ${tag}: picture saved as ${download.suggestedFilename()} (${w} × ${h}, ${png.length} bytes)`);
  }
  // With a share sheet that takes files, the picture goes there.
  await page.evaluate(() => {
    window.__shared = [];
    navigator.canShare = (data) => Array.isArray(data?.files);
    navigator.share = async (data) => {
      window.__shared.push({ files: data.files.map((f) => ({ name: f.name, type: f.type, size: f.size })), text: data.text });
    };
  });
  await button.click();
  await page.waitForFunction(() => window.__shared.length === 1, null, { timeout: 5000 }).catch(() => fail(`share ${tag}: nothing went to the share sheet`));
  const shared = await page.evaluate(() => window.__shared[0]);
  if (shared.files.length !== 1 || shared.files[0].type !== 'image/png' || shared.files[0].size < 2000) fail(`share ${tag}: shared ${JSON.stringify(shared)}`);
  // The canonical address goes along (ADR 0025).
  if (!shared.text?.includes('https://one-more-skat.de')) fail(`share ${tag}: the text names no address: "${shared.text}"`);
  log(`share ${tag}: share sheet got ${shared.files[0].name} with "${shared.text}"`);
  await context.close();
  if (errors.length) fail(`share ${tag}: console errors:\n${errors.join('\n')}`);
}

/** The suit game to declare in qolPass: Rot if allowed, else another suit. */
async function declarableSuit(page) {
  for (const [index, name] of [[2, 'Rot'], [1, 'Grün'], [3, 'Schellen'], [0, 'Eichel']]) {
    if (await page.locator(`[data-action="pick-game"][data-index="${index}"]:not(:disabled)`).count()) return { index, name };
  }
  fail('no suit game may be declared');
  return null;
}

/** The hand as groups in the order shown: "Unter", then the suits (from the cards' names). */
async function handGroups(page) {
  const labels = await page.locator('.hand .hand-card').evaluateAll((els) => els.map((e) => e.getAttribute('aria-label') ?? ''));
  const groups = [];
  for (const label of labels) {
    const g = /\bUnter\b/.test(label) ? 'Unter' : label.split(/[- ]/)[0];
    if (groups[groups.length - 1] !== g) groups.push(g);
  }
  return groups;
}

/** Ids of the hand cards in the order shown. */
async function handIds(page) {
  return page.locator('.hand .hand-card').evaluateAll((els) => els.map((e) => e.getAttribute('data-id')));
}

/**
 * "Hand sortieren: manuell" (handSortPass): the switch in the menu is
 * stored; the Ansage leaves the hand as it was; a card dragged within the
 * hand takes its new place and keeps it while the game goes on.
 */
async function handSortPass(browser, errors, viewport) {
  const tag = `${viewport.width}x${viewport.height}`;
  const context = await quietContext(browser, { viewport, reducedMotion: 'reduce' });
  const page = await context.newPage();
  watch(page, errors);
  await page.goto(base);
  await page.waitForSelector('[data-action="new-run"]');
  await fillSeed(page, SEED);
  await page.click('[data-action="new-run"]');
  await page.waitForSelector('[data-testid="skat-choice"]');
  await page.click('[data-action="menu"]');
  await page.click('[data-testid="toggle-hand-sort"][data-sort="manual"]');
  const stored = await page.evaluate(() => JSON.parse(localStorage.getItem('stammtisch.settings') ?? '{}').handSort);
  if (stored !== 'manual') fail(`hand sort ${tag}: setting stored as ${stored}`);
  await page.keyboard.press('Escape');
  await page.waitForSelector('.overlay', { state: 'detached' });
  await pickUpAndDiscard(page, `hand sort ${tag}`);
  const before = await handIds(page);
  const suit = await declarableSuit(page);
  await page.click(`[data-action="pick-game"][data-index="${suit.index}"]`);
  await page.click('[data-action="declare"]');
  await page.waitForSelector('.declare', { state: 'detached' });
  await page.waitForFunction(() => !document.querySelector('.run.is-busy'));
  await page.waitForSelector('[data-action="play"]');
  const after = await handIds(page);
  if (after.join() !== before.join()) fail(`hand sort ${tag}: manual hand re-sorted by the Ansage (${before.join()} → ${after.join()})`);
  // Drag the first card behind the last one.
  const first = page.locator('.hand .hand-card').first();
  const last = page.locator('.hand .hand-card').last();
  const a = await first.boundingBox();
  const b = await last.boundingBox();
  await page.mouse.move(a.x + a.width / 2, a.y + a.height / 2);
  await page.mouse.down();
  for (let i = 1; i <= 12; i++) await page.mouse.move(a.x + a.width / 2 + ((b.x + b.width * 0.8 - a.x - a.width / 2) * i) / 12, a.y + a.height / 2 + ((b.y + b.height / 2 - a.y - a.height / 2) * i) / 12);
  await page.screenshot({ path: join(screens, `qol-hand-drag-${tag}.png`) });
  await page.mouse.up();
  const moved = await handIds(page);
  const want = [...after.slice(1), after[0]];
  if (moved.join() !== want.join()) fail(`hand sort ${tag}: after the drag ${moved.join()}, expected ${want.join()}`);
  // Play a card: the rest keeps the player's order.
  // (The screenshot also lets the click after a drag pass: ./drag.ts swallows one at once.)
  await page.screenshot({ path: join(screens, `qol-hand-dropped-${tag}.png`) });
  const playId = await page.locator('.hand [data-action="play"]').first().getAttribute('data-id');
  await page.locator('.hand [data-action="play"]').first().click();
  await page.waitForSelector(`.hand .hand-card[data-id="${playId}"]`, { state: 'detached', timeout: 10000 });
  await page.waitForSelector('[data-action="play"], .score-sheet', { timeout: 30000 });
  const rest = await handIds(page);
  if (rest.join() !== want.filter((id) => id !== playId).join()) fail(`hand sort ${tag}: order after a card ${rest.join()}, expected ${want.filter((id) => id !== playId).join()} (played ${playId})`);
  log(`hand sort ${tag}: manual order kept through the Ansage, a drag and a played card`);
  await context.close();
  if (errors.length) fail(`hand sort ${tag}: console errors:\n${errors.join('\n')}`);
}

/**
 * Quality of life on the table (qolPass, 390 × 844 and 1280 × 800; alone
 * with SMOKE_ONLY=qol): the keyboard (S picks up the skat, a digit plays
 * that card of the hand, a card the core does not let go shakes with the
 * same note as a thrown one, the digits show on the cards only after a
 * key, "?" lists the keys, Space skips and closes the score board); the
 * last trick by L and by a tap on the open trick; the hand sorted for the
 * suit declared; a vibration per own trick and one for the result, the
 * switch in the menu.
 */
async function qolPass(browser, errors, viewport) {
  const tag = `${viewport.width}x${viewport.height}`;
  const context = await quietContext(browser, { viewport, reducedMotion: 'reduce' });
  // Vibration: every pattern the app asks for, in window.__buzz.
  await context.addInitScript(() => {
    window.__buzz = [];
    Object.defineProperty(Navigator.prototype, 'vibrate', {
      configurable: true,
      value: (pattern) => {
        window.__buzz.push(pattern);
        return true;
      },
    });
  });
  const page = await context.newPage();
  watch(page, errors);
  await page.goto(base);
  await page.waitForSelector('[data-action="new-run"]');
  await fillSeed(page, SEED);
  await page.click('[data-action="new-run"]');
  await page.waitForSelector('[data-testid="skat-choice"]');
  // No digits before a key (a phone never shows them).
  if (await page.locator('.hand .key-hint').first().isVisible()) fail(`qol ${tag}: key digits shown before any key`);
  await page.keyboard.press('s');
  await page.waitForSelector('[data-testid="skat-zone"] [data-action="select"]');
  const hints = await page.locator('.hand .key-hint').evaluateAll((els) => els.filter((e) => e.getBoundingClientRect().width > 0).map((e) => e.textContent).join(''));
  if (!hints.startsWith('1234567890')) fail(`qol ${tag}: key digits on the hand "${hints}"`);
  await page.keyboard.press('?');
  await page.waitForSelector('[data-testid="keys-help"]');
  await page.screenshot({ path: join(screens, `qol-keys-help-${tag}.png`) });
  await page.keyboard.press('Escape');
  await page.waitForSelector('[data-testid="keys-help"]', { state: 'detached' });
  await discardFirstTwo(page, `qol ${tag}`);
  await page.waitForSelector('.declare[data-phase="DeclaringAfterSkat"]');
  // A suit game other than the order before the Ansage (the Grand order): Rot if it may be declared.
  const suit = await declarableSuit(page);
  await page.click(`[data-action="pick-game"][data-index="${suit.index}"]`);
  // Space: the main button (Ansagen).
  await page.keyboard.press('Space');
  await page.waitForSelector('.declare', { state: 'detached' });
  await page.waitForFunction(() => !document.querySelector('.run.is-busy'));
  // Hand sortieren (automatisch): Unter first, then the trump suit, then the other suits.
  const groups = await handGroups(page);
  const want = ['Unter', suit.name, ...['Eichel', 'Grün', 'Rot', 'Schellen'].filter((x) => x !== suit.name)];
  const order = groups.filter((g) => want.includes(g));
  if (order.join() !== want.filter((g) => groups.includes(g)).join()) fail(`qol ${tag}: hand after ${suit.name} declared in the order ${groups.join(', ')}`);
  log(`qol ${tag}: hand sorted for ${suit.name}: ${groups.join(', ')}`);
  let played = 0;
  let refused = false;
  for (;;) {
    await page.waitForSelector('[data-action="play"], .score-sheet', { timeout: 30000 });
    if (await page.locator('.score-sheet').count()) break;
    if (played === 1) {
      // Letzter Stich: L shows it and puts it away, a tap on the open trick shows it too.
      await page.keyboard.press('l');
      await page.waitForSelector('[data-testid="last-trick"]', { timeout: 3000 }).catch(() => fail(`qol ${tag}: L shows no last trick`));
      if ((await page.locator('[data-testid="last-trick"] .trick-card').count()) !== 3) fail(`qol ${tag}: the last trick has not three cards`);
      await page.keyboard.press('l');
      await page.waitForSelector('[data-testid="last-trick"]', { state: 'detached' });
      await page.click('[data-testid="trick"]');
      await page.waitForSelector('[data-testid="last-trick"]', { timeout: 3000 }).catch(() => fail(`qol ${tag}: a tap on the trick shows no last trick`));
      await page.screenshot({ path: join(screens, `qol-last-trick-${tag}.png`) });
    }
    const cards = page.locator('.hand .hand-card');
    const legal = await cards.evaluateAll((els) => els.map((e) => e.getAttribute('data-action') === 'play'));
    const dim = legal.findIndex((x) => !x);
    if (!refused && dim >= 0 && dim < 10) {
      await page.keyboard.press(String((dim + 1) % 10));
      await page.waitForFunction(() => document.querySelector('.toast-layer')?.textContent?.includes('Diese Karte geht gerade nicht'), null, { timeout: 3000 }).catch(() => fail(`qol ${tag}: no note for a card that may not be played`));
      if ((await cards.count()) !== legal.length) fail(`qol ${tag}: a card that may not be played left the hand`);
      refused = true;
    }
    const first = legal.indexOf(true);
    const id = await cards.nth(first).getAttribute('data-id');
    await page.keyboard.press(String((first + 1) % 10));
    await page.waitForSelector(`.hand .hand-card[data-id="${id}"]`, { state: 'detached', timeout: 10000 }).catch(() => fail(`qol ${tag}: key ${first + 1} did not play card ${id}`));
    played += 1;
    if (played > 10) fail(`qol ${tag}: played more than ten cards`);
  }
  if (played !== 10) fail(`qol ${tag}: ${played} cards played by key`);
  // Space skips the counting (if it still runs), then closes the board.
  await page.waitForSelector('[data-action="score-continue"], [data-action="score-skip"].btn');
  if (await page.locator('[data-action="score-skip"].btn').count()) {
    await page.keyboard.press('Space');
    await page.waitForSelector('[data-action="score-continue"]');
  }
  await page.keyboard.press('Space');
  await page.waitForSelector('.score-sheet', { state: 'detached' });
  log(`qol ${tag}: a game played by keyboard (${played} cards${refused ? ', one refused' : ''})`);
  // Vibration: a short one per trick taken, one pattern for the game's (or pub's) result.
  const buzzed = await page.evaluate(() => window.__buzz);
  const taps = buzzed.filter((p) => p === 12).length;
  const results = buzzed.filter((p) => Array.isArray(p));
  if (results.length !== 1) fail(`qol ${tag}: vibration patterns for the result: ${JSON.stringify(results)}`);
  log(`qol ${tag}: vibrated ${taps}× for a trick, result ${JSON.stringify(results[0])}`);
  // Switched off in the menu, nothing vibrates (the switch shows where the browser can vibrate).
  await page.click('[data-action="menu"]');
  await page.click('[data-testid="toggle-vibrate"][aria-pressed="true"]');
  const off = await page.evaluate(() => JSON.parse(localStorage.getItem('stammtisch.settings') ?? '{}').vibrate);
  if (off !== false) fail(`qol ${tag}: vibration switched off stored as ${off}`);
  await page.keyboard.press('Escape');
  await context.close();
  if (errors.length) fail(`qol ${tag}: console errors:\n${errors.join('\n')}`);
}

/**
 * Offline play (DESIGN 14.5): the service worker of the build installs and
 * takes over on the first visit; offline a reload still brings the start
 * screen (shell and assets from the cache), the daily board says
 * "offline", a run starts; back online nothing is left broken.
 */
async function offlinePass(browser, errors) {
  const context = await quietContext(browser, { viewport: PHONE, reducedMotion: 'reduce' });
  const page = await context.newPage();
  watch(page, errors);
  await page.goto(base);
  await page.waitForSelector('[data-action="new-run"]');
  await page.waitForFunction(() => navigator.serviceWorker.controller !== null, null, { timeout: 30000 }).catch(() => fail('offline: no service worker in control after the first visit'));
  await context.setOffline(true);
  await page.reload();
  await page.waitForSelector('[data-action="new-run"]', { timeout: 15000 }).catch(() => fail('offline: the start screen does not load from the cache'));
  await openStartMore(page);
  await page.waitForSelector('[data-testid="board-offline"]').catch(() => fail('offline: no offline note at the daily board'));
  await fillSeed(page, SEED);
  await page.click('[data-action="new-run"]');
  await page.waitForSelector('[data-testid="skat-choice"]', { timeout: 20000 }).catch(() => fail('offline: no run starts'));
  await context.setOffline(false);
  await page.reload();
  await page.waitForSelector('[data-action="resume"]');
  await context.close();
  log('offline: service worker in control, start screen and a new run without network');
}

/**
 * Die Wochen-Herausforderung and the Statistik (DESIGN 5.6 and 9.3; alone
 * with SMOKE_ONLY=weekly). The start screen shows this week's block next
 * to the Tafel des Tages: the Sonderregel from the core, the time left in
 * the week. "Tafel der Woche" opens the board sheet (the board is served by
 * a stub of /api/weekly here; the real server is checked by hand, see
 * DESIGN 5.6). The weekly run starts with the week's seed, the menu names
 * its rule. The Vereinsbuch's Statistik shows the core's records per
 * character (win rate, favourite relics) and the new page "Relikte" per
 * relic, from a meta state saved before the relic statistics existed plus
 * one with them. Nothing scrolls sideways.
 */
async function weeklyPass(browser, errors, viewport) {
  const context = await quietContext(browser, { viewport, reducedMotion: 'reduce' });
  const page = await context.newPage();
  watch(page, errors);
  const tag = `${viewport.width}x${viewport.height}`;
  const snap = (name) => page.screenshot({ path: join(screens, `${name}-${tag}.png`) });
  let asked = null;
  await page.route('**/api/weekly/*', (route) => {
    asked = new URL(route.request().url()).pathname.split('/').pop();
    const row = (rank, name, character, won, section, series, me = false) => ({ rank, name, character, won, section, series, stake: 0, bestGame: 40 + rank, me });
    route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify({
        week: asked,
        total: 12,
        entries: [row(1, 'Uschi', 'Declarer', true, 3, 5), row(2, 'Kalle', 'NullPlayer', false, 3, 2), row(3, null, 'Shirker', false, 2, 4)],
        me: null,
        signedIn: false,
      }),
    });
  });
  // A meta state from before the relic statistics, with a run counted since.
  const meta = JSON.parse(readFileSync(resolve(webDir, '../core/tests/fixtures/meta-before-relic-stats.json'), 'utf8'));
  meta.meta.stats.relics = { tour: { taken: 4, held: 3, won: 1 }, beer_mat: { taken: 2, held: 1, won: 0 }, guardian_angel: { taken: 1, held: 0, won: 0 } };
  meta.meta.stats.character_relics = { Declarer: { tour: 3, beer_mat: 1, card_counter: 2 } };
  await context.addInitScript((m) => {
    if (!localStorage.getItem('stammtisch.meta')) localStorage.setItem('stammtisch.meta', m);
  }, JSON.stringify(meta));
  await page.goto(base);
  const block = page.locator('[data-testid="weekly"]');
  await block.waitFor({ timeout: 15000 }).catch(() => fail(`weekly ${tag}: no weekly block on the start screen`));
  const week = Number(await block.getAttribute('data-week'));
  if (!(week > 202600 && week < 210000)) fail(`weekly ${tag}: the block names week ${week}`);
  const rule = ((await page.locator('[data-testid="weekly-rule"]').textContent()) ?? '').trim();
  if (rule.length < 12) fail(`weekly ${tag}: the Sonderregel reads "${rule}"`);
  const left = (await page.locator('[data-testid="weekly-left"]').textContent()) ?? '';
  if (!/^noch \d+ (Tage?|Std\.)/.test(left)) fail(`weekly ${tag}: the time left reads "${left}"`);
  // Visible on the first level (DESIGN 16.4); the daily block waits behind "Weitere Spielarten", below it.
  if (!(await block.isVisible())) fail(`weekly ${tag}: the weekly block is not visible on the start screen`);
  if (await page.locator('[data-testid="daily"]').isVisible()) fail(`weekly ${tag}: the daily block is not folded away`);
  await openStartMore(page);
  const daily = await page.locator('[data-testid="daily"]').boundingBox();
  const box = await block.boundingBox();
  if (!daily || !box || daily.y < box.y + box.height - 1) fail(`weekly ${tag}: the daily block is not below the weekly block`);
  await block.scrollIntoViewIfNeeded();
  await snap('weekly-start');
  await assertNoSideScroll(page, `weekly ${tag}: start screen`, '[data-testid="weekly"]');
  // The board sheet.
  await page.click('[data-testid="weekly-board-open"]');
  await page.waitForSelector('.sheet-weekly-board [data-testid="weekly-board"] .board-row', { timeout: 10000 }).catch(() => fail(`weekly ${tag}: no rows on the Tafel der Woche`));
  if (asked !== String(week)) fail(`weekly ${tag}: the board asked for week ${asked}, the block shows ${week}`);
  const rows = await page.locator('.sheet-weekly-board .board-row').count();
  if (rows !== 3) fail(`weekly ${tag}: ${rows} rows on the board`);
  const head = (await page.locator('.sheet-weekly-board .board-head').textContent()) ?? '';
  if (!head.includes('12 Spieler diese Woche')) fail(`weekly ${tag}: the board head reads "${head}"`);
  if (!(await page.locator('.sheet-weekly-board [data-testid="board-signin"]').count())) fail(`weekly ${tag}: no sign-in hint without an account`);
  await snap('weekly-board');
  await assertNoSideScroll(page, `weekly ${tag}: board`, '.sheet-weekly-board');
  await page.click('.sheet-weekly-board [data-action="close-overlay"]');
  await page.waitForSelector('.sheet-weekly-board', { state: 'detached' });
  // The Vereinsbuch: Statistik and Relikte.
  await page.click('[data-action="book"]');
  await page.click('[data-action="book-tab"][data-tab="stats"]');
  const declarer = (await page.locator('.ledger-char[data-character="Declarer"]').textContent()) ?? '';
  if (!declarer.includes('22 %') || !declarer.includes('bestes Spiel 288')) fail(`weekly ${tag}: the Statistik of the Alleinspieler reads "${declarer.replace(/\s+/g, ' ')}"`);
  const favs = (await page.locator('.ledger-char[data-character="Declarer"] [data-testid="favourite-relics"]').textContent()) ?? '';
  if (!/Tournee \(3×\), Der Zähler \(2×\)/.test(favs)) fail(`weekly ${tag}: favourite relics read "${favs}"`);
  await snap('weekly-stats');
  await page.click('[data-action="book-tab"][data-tab="relics"]');
  await page.waitForSelector('[data-testid="relic-stats"]');
  const relics = await page.locator('[data-testid="relic-stats"] [data-relic]').evaluateAll((els) => els.map((e) => e.getAttribute('data-relic')));
  if (relics.join(',') !== 'tour,beer_mat,guardian_angel') fail(`weekly ${tag}: relics listed as ${relics.join(',')}`);
  const tour = (await page.locator('[data-relic="tour"]').textContent()) ?? '';
  if (!tour.includes('4× genommen') || !tour.includes('3× am Ende') || !tour.includes('1 gewonnen (33 %)')) fail(`weekly ${tag}: the Tournee reads "${tour.replace(/\s+/g, ' ')}"`);
  await snap('weekly-relics');
  await assertNoSideScroll(page, `weekly ${tag}: relic statistics`, '.ledger');
  await page.click('.ledger [data-action="close-overlay"]');
  // The weekly run: the week's seed and its rule in the menu.
  await page.click('[data-testid="play-weekly"]');
  await page.waitForSelector('[data-testid="skat-choice"], .declare, [data-testid="route"]', { timeout: 20000 }).catch(() => fail(`weekly ${tag}: the weekly run does not start`));
  const saved = await page.evaluate(() => JSON.parse(localStorage.getItem('stammtisch.run') ?? '{}').run?.weekly ?? null);
  if (!saved || saved.week !== week || !saved.rule) fail(`weekly ${tag}: the run is saved without its week (${JSON.stringify(saved)})`);
  while (await page.locator('.sheet-unlock').count()) {
    await page.locator('.sheet-unlock [data-action="close-overlay"]').last().click();
    await page.waitForTimeout(200);
  }
  await page.click('[data-action="menu"]');
  const menuRule = (await page.locator('[data-testid="menu-weekly"]').textContent().catch(() => '')) ?? '';
  if (!menuRule.includes('Sonderregel der Woche')) fail(`weekly ${tag}: the menu names no rule ("${menuRule}")`);
  await snap('weekly-menu');
  log(`weekly ${tag}: week ${week} "${rule.slice(0, 40)}…", ${left}, board ${rows} rows, Statistik and Relikte, run with rule ${saved.rule}`);
  await context.close();
}

/**
 * Colour vision deficiencies as SVG colour matrices (Machado et al. 2009,
 * severity 1, linear RGB), laid over the whole page for screenshots.
 */
const CVD = {
  deuteranopia: '0.367322 0.860646 -0.227968 0 0  0.280085 0.672501 0.047413 0 0  -0.011820 0.042940 0.968881 0 0  0 0 0 1 0',
  protanopia: '0.152286 1.052583 -0.204868 0 0  0.114503 0.786281 0.099216 0 0  -0.003882 -0.048116 1.051998 0 0  0 0 0 1 0',
};

async function simulateCvd(page, kind) {
  await page.evaluate((m) => {
    document.getElementById('cvd-sim')?.remove();
    document.documentElement.style.filter = '';
    if (!m) return;
    const svg = document.createElementNS('http://www.w3.org/2000/svg', 'svg');
    svg.id = 'cvd-sim';
    svg.setAttribute('width', '0');
    svg.setAttribute('height', '0');
    svg.style.position = 'absolute';
    svg.innerHTML = `<filter id="cvd-f" color-interpolation-filters="linearRGB"><feColorMatrix type="matrix" values="${m}"/></filter>`;
    document.body.appendChild(svg);
    document.documentElement.style.filter = 'url(#cvd-f)';
  }, kind ? CVD[kind] : '');
}

/** Screenshots of the page as it is and as seen with deuteranopia and protanopia. */
async function shootCvd(page, name) {
  const size = page.viewportSize();
  const tag = size ? `${size.width}x${size.height}` : 'page';
  await page.screenshot({ path: join(screens, `${name}-${tag}.png`) });
  for (const kind of Object.keys(CVD)) {
    await simulateCvd(page, kind);
    await page.screenshot({ path: join(screens, `${name}-${kind}-${tag}.png`) });
  }
  await simulateCvd(page, null);
}

/** The page never scrolls sideways and no element of the screen or dialog leaves the viewport. */
async function assertNoSideways(page, label) {
  const r = await page.evaluate(() => {
    const w = innerWidth;
    const out = [];
    for (const el of document.querySelectorAll('.screen *, .layer *')) {
      const b = el.getBoundingClientRect();
      if (!b.width || !b.height || el.closest('svg, [data-testid="characters"], .leiste')) continue;
      // Inside a box that scrolls or clips sideways on purpose (the HUD strip, the character row).
      if (el.parentElement?.closest('.hud-leiste, .relic-strip, .char-pick')) continue;
      if (b.right > w + 1 || b.left < -1) out.push(`${el.tagName.toLowerCase()}.${String(el.className?.baseVal ?? el.className).split(' ')[0]} ${Math.round(b.left)}..${Math.round(b.right)}`);
    }
    return { page: document.documentElement.scrollWidth - w, out: out.slice(0, 4) };
  });
  if (r.page > 0) fail(`${label}: the page scrolls sideways by ${r.page}px`);
  if (r.out.length) fail(`${label}: elements leave the viewport: ${r.out.join(', ')}`);
}

/**
 * Barrierefreiheit (DESIGN 16): the menu row "Darstellung" with
 * "Farbenblind-Hilfe" (suit letters on every face, stored) and
 * "Schriftgröße" (Normal, Groß, Sehr groß: the type scale grows, nothing
 * scrolls sideways at 390 × 844 and 360 × 740, the table still fits), the
 * screen reader's names of cards, the HUD and the opponents, a polite live
 * region for an opponent's card, a trick and the result, and the focus in
 * dialogs (into the menu, kept inside on Tab, Escape closes it and the
 * focus returns). Screenshots with deuteranopia and protanopia simulated.
 */
async function a11yPass(browser, errors, reducedMotion = 'reduce') {
  const context = await quietContext(browser, { viewport: PHONE, reducedMotion });
  // Every line the live region gets, in window.__said.
  await context.addInitScript(() => {
    window.__said = [];
    new MutationObserver(() => {
      const live = document.querySelector('[data-testid="sr-live"]');
      if (!live || live.__watched) return;
      live.__watched = true;
      new MutationObserver((records) => {
        for (const r of records) for (const n of r.addedNodes) if (n.textContent) window.__said.push(n.textContent);
      }).observe(live, { childList: true });
    }).observe(document, { childList: true, subtree: true });
  });
  const page = await context.newPage();
  watch(page, errors);
  await page.goto(base);
  await page.waitForSelector('[data-action="new-run"]');
  await fillSeed(page, SEED);
  await page.click('[data-action="new-run"]');
  await page.waitForSelector('[data-testid="skat-choice"]');
  const letters = () =>
    page.$$eval('.hand .card-svg[data-suit]', (svgs) =>
      svgs.map((svg) => {
        const use = svg.querySelector('use.face-letter');
        return { suit: svg.dataset.suit, href: use?.getAttribute('href') ?? '', shown: use ? getComputedStyle(use).display !== 'none' : false };
      }),
    );
  const off = await letters();
  if (!off.length || off.some((l) => l.shown)) fail('a11y: suit letters shown without "Farbenblind-Hilfe"');
  await shootCvd(page, 'a11y-cards-plain');

  // The menu by keyboard: focus goes into the dialog, Tab stays in it, Escape closes it and focus returns.
  await page.focus('.hud [data-action="menu"]');
  await page.keyboard.press('Enter');
  await page.waitForSelector('.layer .sheet-menu');
  const into = await page.evaluate(() => Boolean(document.activeElement?.closest('.layer [role="dialog"]')));
  if (!into) fail('a11y: the menu opened, but the focus did not go into it');
  const named = await page.locator('.layer [role="dialog"]').getAttribute('aria-labelledby');
  if (!named || !(await page.locator(`#${named}`).count())) fail('a11y: the menu dialog has no name');
  for (let i = 0; i < 40; i++) await page.keyboard.press('Tab');
  if (!(await page.evaluate(() => Boolean(document.activeElement?.closest('.layer [role="dialog"]'))))) fail('a11y: Tab left the open menu');
  for (let i = 0; i < 5; i++) await page.keyboard.press('Shift+Tab');
  if (!(await page.evaluate(() => Boolean(document.activeElement?.closest('.layer [role="dialog"]'))))) fail('a11y: Shift+Tab left the open menu');
  const ring = await page.evaluate(() => getComputedStyle(document.activeElement).outlineStyle);
  if (ring === 'none') fail('a11y: no visible focus ring on the focused control');
  await page.keyboard.press('Escape');
  await page.waitForSelector('.layer .sheet-menu', { state: 'detached' });
  const back = await page.evaluate(() => document.activeElement?.getAttribute('data-action'));
  if (back !== 'menu') fail(`a11y: after Escape the focus is on "${back}", not on the menu button`);
  log('a11y: menu by keyboard: focus in, kept inside on Tab, Escape closes, focus back on the menu button');

  // "Farbenblind-Hilfe": off by default, on in the row "Darstellung", stored.
  await page.click('[data-action="menu"]');
  const cb = page.locator('[data-testid="setting-display"] [data-testid="toggle-color-blind"]');
  if ((await cb.getAttribute('aria-pressed')) !== 'false') fail('a11y: "Farbenblind-Hilfe" is on by default');
  if ((await cb.getAttribute('aria-label')) !== 'Farbenblind-Hilfe') fail(`a11y: the toggle is named "${await cb.getAttribute('aria-label')}"`);
  await assertMenuNoScroll(page, 'menu with the row "Darstellung"');
  await cb.click();
  await page.waitForSelector('[data-testid="toggle-color-blind"][aria-pressed="true"]');
  const stored = await page.evaluate(() => JSON.parse(localStorage.getItem('stammtisch.settings') ?? '{}'));
  if (stored.colorBlind !== true) fail(`a11y: "Farbenblind-Hilfe" not stored: ${JSON.stringify(stored)}`);
  await page.click('[data-testid="menu-close"]');
  const on = await letters();
  const code = { Clubs: 'C', Spades: 'S', Hearts: 'H', Diamonds: 'D' };
  const wrong = on.filter((l) => !l.shown || l.href !== `#cb-${code[l.suit]}`);
  if (wrong.length) fail(`a11y: suit letters wrong: ${JSON.stringify(wrong.slice(0, 3))}`);
  if (new Set(on.map((l) => l.suit)).size < 4) fail('a11y: the hand should hold all four suits');
  const glyphs = await page.evaluate(() => ['C', 'S', 'H', 'D'].filter((c) => !document.getElementById(`cb-${c}`)?.querySelector('path')));
  if (glyphs.length) fail(`a11y: suit letter symbols missing in the sprite: ${glyphs.join(', ')}`);
  await shootCvd(page, 'a11y-cards-letters');
  await page.reload();
  await page.click('[data-action="resume"]');
  await page.waitForSelector('[data-testid="skat-choice"]');
  if ((await letters()).some((l) => !l.shown)) fail('a11y: suit letters gone after a reload');
  log('a11y: "Farbenblind-Hilfe" off by default, a letter per suit on every face, stored');

  // Screen reader names: cards, the opponents' boxes, the HUD.
  const cardNames = await page.$$eval('.hand .hand-card', (els) => els.map((e) => e.getAttribute('aria-label') ?? ''));
  if (cardNames.some((n) => !/^(Eichel|Grün|Rot|Schellen)-(Sieben|Acht|Neun|Zehn|Unter|Ober|Oberin|König|Daus)/.test(n))) fail(`a11y: hand card names: ${cardNames.join(' | ')}`);
  const oppName = (await page.locator('.opp[data-seat="1"] .opp-main').getAttribute('aria-label')) ?? '';
  if (!/10 Karten/.test(oppName)) fail(`a11y: the opponent's box is named "${oppName}"`);
  const hud = await page.locator('.hud').ariaSnapshot();
  if (!/Groschen/.test(hud) || !/meter "Konto \/ Ziel"/.test(hud)) fail(`a11y: the HUD reads:\n${hud}`);
  log(`a11y: names: "${cardNames[0]}", "${oppName}"`);

  // A game: the live region says the opponents' cards, the tricks and the result, politely.
  await page.click('[data-action="pick-up-skat"]');
  await page.waitForSelector('[data-testid="skat-zone"] [data-action="select"]');
  await discardFirstTwo(page, 'a11y');
  await page.waitForSelector('.declare[data-phase="DeclaringAfterSkat"]');
  await page.click('[data-action="pick-game"][data-index="4"]');
  await page.click('[data-action="declare"]');
  await page.waitForSelector('.declare', { state: 'detached' });
  const live = await page.locator('[data-testid="sr-live"]').evaluate((el) => ({ role: el.getAttribute('role'), polite: el.getAttribute('aria-live') }));
  if (live.role !== 'status' || live.polite !== 'polite') fail(`a11y: live region ${JSON.stringify(live)}`);
  await page.waitForSelector('[data-action="play"]');
  const trickName = await page.evaluate(() => document.querySelector('.trick-card[aria-label]')?.getAttribute('aria-label') ?? '');
  await playOut(page, 'a11y');
  const said = await page.evaluate(() => window.__said);
  if (!said.some((l) => / spielt (Eichel|Grün|Rot|Schellen)-/.test(l))) fail(`a11y: no opponent's card said: ${said.slice(0, 6).join(' | ')}`);
  if (!said.some((l) => /Stich/.test(l))) fail(`a11y: no trick said: ${said.slice(0, 6).join(' | ')}`);
  if (!said.some((l) => /^(Gewonnen|Verloren): /.test(l))) fail(`a11y: no result said: ${said.slice(-3).join(' | ')}`);
  if (said.some((l) => l.length > 90)) fail(`a11y: a live line is too long: ${said.find((l) => l.length > 90)}`);
  const scoreFocus = await page.evaluate(() => Boolean(document.activeElement?.closest('.score-sheet')));
  if (!scoreFocus) fail('a11y: the score board took no focus');
  log(`a11y: live region said ${said.length} lines, e.g. "${said[0]}", "${said[said.length - 1]}"${trickName ? `; trick card "${trickName}"` : ''}`);
  await shootCvd(page, 'a11y-score');
  await context.close();

  // "Schriftgröße": Normal, Groß, Sehr groß; the scale grows, nothing scrolls sideways, the table fits.
  const sizes = await quietContext(browser, { viewport: PHONE, reducedMotion });
  const p2 = await sizes.newPage();
  watch(p2, errors);
  await p2.goto(base);
  await p2.waitForSelector('[data-action="new-run"]');
  const body = () => p2.evaluate(() => parseFloat(getComputedStyle(document.documentElement).getPropertyValue('--fs-body')));
  let last = await body();
  for (const size of ['large', 'xlarge']) {
    await p2.click('[data-action="settings"], [data-action="menu"]');
    await p2.click('[data-testid="toggle-text-size"]');
    await p2.waitForSelector(`[data-testid="toggle-text-size"][data-now="${size}"]`);
    const now = await body();
    if (!(now > last)) fail(`a11y: text size ${size}: --fs-body ${now}px, not above ${last}px`);
    last = now;
    for (const vp of [PHONE, { width: 360, height: 740 }]) {
      await p2.setViewportSize(vp);
      await p2.waitForTimeout(120);
      await assertNoSideways(p2, `text ${size}, menu at ${vp.width}x${vp.height}`);
    }
    await p2.setViewportSize(PHONE);
    if (size === 'xlarge') await p2.screenshot({ path: join(screens, 'a11y-text-xlarge-menu-390x844.png') });
    await p2.click('[data-testid="menu-close"]');
    const stored = await p2.evaluate(() => JSON.parse(localStorage.getItem('stammtisch.settings') ?? '{}'));
    if (stored.textSize !== size) fail(`a11y: text size ${size} not stored: ${JSON.stringify(stored)}`);
    for (const vp of [PHONE, { width: 360, height: 740 }]) {
      await p2.setViewportSize(vp);
      await p2.waitForTimeout(120);
      await assertNoSideways(p2, `text ${size}, start screen at ${vp.width}x${vp.height}`);
    }
    await p2.setViewportSize(PHONE);
    await fillSeed(p2, SEED);
    await p2.click('[data-action="new-run"]');
    // From the second size on a run is going: give it up for the new one.
    await p2.waitForSelector('[data-testid="skat-choice"], [data-action="confirm-new"]');
    if (await p2.locator('[data-action="confirm-new"]').count()) await p2.click('[data-action="confirm-new"]');
    await p2.waitForSelector('[data-testid="skat-choice"]');
    await assertFits(p2, `text ${size}, Skat oder Hand`);
    for (const vp of [PHONE, { width: 360, height: 740 }]) {
      await p2.setViewportSize(vp);
      await p2.waitForTimeout(120);
      await assertNoSideways(p2, `text ${size}, table at ${vp.width}x${vp.height}`);
      await p2.screenshot({ path: join(screens, `a11y-text-${size}-${vp.width}x${vp.height}.png`) });
    }
    await p2.setViewportSize(PHONE);
    await p2.click('[data-action="pick-up-skat"]');
    await p2.waitForSelector('[data-testid="skat-zone"] [data-action="select"]');
    await discardFirstTwo(p2, `text ${size}`);
    await p2.waitForSelector('.declare[data-phase="DeclaringAfterSkat"]');
    await assertFits(p2, `text ${size}, Ansage after the skat`);
    await p2.click('[data-action="pick-game"][data-index="4"]');
    await p2.click('[data-action="declare"]');
    await p2.waitForSelector('[data-action="play"]');
    await assertFits(p2, `text ${size}, playing`);
    await p2.goto(base);
    await p2.waitForSelector('[data-action="new-run"]');
  }
  log('a11y: "Schriftgröße" Groß and Sehr groß: bigger type, stored, no sideways scroll, the table fits');
  await sizes.close();
}

/**
 * German words that must not show in the English game (ADR 0023). Pub
 * names stay German on purpose (place names: "Zum Goldenen Hirschen"), so
 * the list holds only words of the interface, never of a pub's name.
 */
const GERMAN_WORDS = /(?<![\p{L}])(Fortsetzen|Einstellungen|Stadtteil|Einführung|Schließen|Gegner|Stich|Stiche|Augen|Einmischen|Drücken|Ansagen|Ansage|Tresen|Weiter|Überspringen|Spielwert|Grundwert|Stufe|Stufen|Konto|Zinke|Zinken|Relikt|Relikte|Schnäpse|Hausregel|Hausregeln|Eichel|Schellen|Daus|Oberin|König|nicht|Gewonnen|Verloren|Karte|Karten|Wähle|Spiele|Abbrechen|Zurück|Kaufen|Verkaufen|Begegnung|Werkstatt|Rast|Vereinsbuch|Kreidetafel|Regelkarte|Glossar|Abspann|Wirt|Stammgast|Bierdeckel|Fluch|Flüche)(?![\p{L}])/u;

/**
 * The English game shows no German interface text: neither in the visible
 * text nor in aria-labels, titles and placeholders of `root`.
 */
async function assertEnglish(page, label, root = 'body') {
  const found = await page.evaluate(
    ({ sel, source }) => {
      const re = new RegExp(source, 'u');
      const box = document.querySelector(sel);
      if (!box) return [`${sel} not on screen`];
      const hits = [];
      const walker = document.createTreeWalker(box, NodeFilter.SHOW_TEXT);
      for (let n = walker.nextNode(); n; n = walker.nextNode()) {
        const el = n.parentElement;
        if (!el || el.closest('.sr-only, script, style, [aria-hidden="true"]')) continue;
        // Names of pubs and districts' pubs are German on purpose.
        if (el.closest('[data-pub-name], .pub-name, .map-label')) continue;
        const m = re.exec(n.textContent ?? '');
        if (m) hits.push(`"${m[1]}" in "${(n.textContent ?? '').trim().slice(0, 80)}"`);
      }
      for (const el of box.querySelectorAll('[aria-label], [title], [placeholder]')) {
        for (const a of ['aria-label', 'title', 'placeholder']) {
          const v = el.getAttribute(a);
          const m = v ? re.exec(v) : null;
          if (m) hits.push(`"${m[1]}" in ${a} "${v.slice(0, 80)}"`);
        }
      }
      return hits;
    },
    { sel: root, source: GERMAN_WORDS.source },
  );
  if (found.length) fail(`English ${label}: German text: ${found.slice(0, 6).join('; ')}`);
}

/**
 * Longer English strings must not break the layout: at 390 × 844, 375 ×
 * 667, 360 × 740 and 1280 × 800 the page never scrolls sideways and no
 * button, label or heading cuts off its text.
 */
async function assertEnglishFits(page, label) {
  const original = page.viewportSize();
  for (const size of [PHONE, { width: 375, height: 667 }, { width: 360, height: 740 }, DESKTOP]) {
    await page.setViewportSize(size);
    await page.waitForTimeout(150);
    const r = await page.evaluate(() => {
      const name = (el) => `${el.tagName.toLowerCase()}.${String(el.className?.baseVal ?? el.className ?? '').trim().replace(/\s+/g, '.')}`;
      const clipped = [...document.querySelectorAll('button, .btn, .seg, .chip, h1, h2, h3, .setting-label, .info-tag, .badge, label')]
        .filter((el) => {
          if (el.closest('.sr-only') || !el.textContent?.trim()) return false;
          const st = getComputedStyle(el);
          if (st.display === 'none' || st.visibility === 'hidden') return false;
          const b = el.getBoundingClientRect();
          if (!b.width || !b.height) return false;
          const cut = (st.overflowX !== 'visible' && el.scrollWidth > el.clientWidth + 1) || (st.textOverflow === 'ellipsis' && el.scrollWidth > el.clientWidth + 1);
          return cut;
        })
        .map((el) => `${name(el)} "${el.textContent.trim().replace(/\s+/g, ' ').slice(0, 40)}" (${el.scrollWidth} > ${el.clientWidth})`);
      return { doc: document.scrollingElement.scrollWidth, width: innerWidth, clipped };
    });
    const where = `English ${label} at ${size.width}x${size.height}`;
    if (r.doc > r.width) fail(`${where}: the page is ${r.doc}px wide, the viewport ${r.width}px`);
    if (r.clipped.length) fail(`${where}: text cut off: ${r.clipped.slice(0, 5).join(', ')}`);
  }
  if (original) await page.setViewportSize(original);
}

/** Screenshot, German check and layout check of one English screen. */
async function englishScreen(page, name, label, root = 'body') {
  await shoot(page, `en-${name}`);
  await assertEnglish(page, label, root);
  await assertEnglishFits(page, label);
  log(`English ${label}: no German text, nothing cut off or scrolling sideways`);
}

/**
 * English (ADR 0023; alone with SMOKE_ONLY=english): an English browser
 * gets the English game. Start screen, a whole game (redeal, Skat, discard,
 * Grand, ten tricks, score), the unlock, the card reward, the route, the
 * Tresen, the menu with the language switch (and back, without a reload),
 * an encounter, the rules card, the glossary and the Einführung, each at
 * 390 × 844 and 1280 × 800: no German interface text, no text cut off, no
 * sideways scrolling.
 */
async function englishPass(browser, errors) {
  const context = await quietContext(browser, { viewport: PHONE, reducedMotion: 'reduce', locale: 'en-GB' });
  const page = await context.newPage();
  watch(page, errors);
  await page.goto(base);
  await page.waitForSelector('[data-action="new-run"]');
  if ((await page.evaluate(() => document.documentElement.lang)) !== 'en') fail('English: the page is not marked lang="en" for an English browser');
  const startText = await page.locator('[data-action="new-run"]').first().textContent();
  if (!/^New run/.test((startText ?? '').trim())) fail(`English: the start button says "${startText}"`);
  await englishScreen(page, 'start', 'start screen');

  // The menu from the start screen: the language row, then the rules and the glossary.
  await page.click('[data-action="settings"]');
  await page.waitForSelector('[data-testid="setting-language"]');
  if ((await page.locator('[data-testid="lang-en"][aria-pressed="true"]').count()) !== 1) fail('English: the menu does not mark English as the language');
  await englishScreen(page, 'menu', 'menu', '.overlay');
  // Switch to German and back, without a reload.
  const loads = await page.evaluate(() => performance.getEntriesByType('navigation').length);
  await page.click('[data-testid="lang-de"]');
  await page.waitForFunction(() => document.documentElement.lang === 'de');
  await page.waitForSelector('[data-testid="lang-de"][aria-pressed="true"]');
  const german = (await page.locator('.menu-head h2').textContent()) ?? '';
  if (german.trim() !== 'Menü' || !(await page.locator('.overlay').filter({ hasText: 'Klänge' }).count())) fail('English: switching to Deutsch does not draw the menu in German');
  await page.click('[data-testid="lang-en"]');
  await page.waitForFunction(() => document.documentElement.lang === 'en');
  if (!(await page.locator('.overlay').filter({ hasText: 'Effects' }).count())) fail('English: switching back does not draw the menu in English');
  if ((await page.evaluate(() => localStorage.getItem('stammtisch.lang'))) !== 'en') fail('English: the chosen language is not stored');
  if ((await page.evaluate(() => performance.getEntriesByType('navigation').length)) !== loads) fail('English: the language switch reloaded the page');
  log('English: menu switches to Deutsch and back without a reload, choice stored');
  await page.click('.overlay [data-action="close-overlay"]');
  await page.waitForSelector('[data-testid="setting-language"]', { state: 'detached' });

  // A whole game with the main pass's seed: one redeal, Skat, discard, Grand.
  await fillSeed(page, SEED);
  await page.click('[data-action="new-run"]');
  await page.waitForSelector('[data-testid="skat-choice"]');
  const progress = await text(page, 'pub-progress');
  if (!/^Game 1 of 3 · 0 \/ \d+ points$/.test(progress)) fail(`English: pub progress in the head: ${progress}`);
  await englishScreen(page, 'table-choice', 'Skat or Hand');
  // "Colour-blind aid": the corner letters follow the language (A L H B, sprite cb-en-*).
  const hrefs = await page.$$eval('.hand use.face-letter', (uses) => uses.map((u) => u.getAttribute('href')));
  if (!hrefs.length || hrefs.some((h) => !/^#cb-en-[CSHD]$/.test(h ?? ''))) fail(`English: suit letters use ${[...new Set(hrefs)].join(', ')}`);
  const enGlyphs = await page.evaluate(() => ['C', 'S', 'H', 'D'].filter((c) => !document.getElementById(`cb-en-${c}`)?.querySelector('path')));
  if (enGlyphs.length) fail(`English: no letter glyph for ${enGlyphs.join(', ')}`);
  await assertFits(page, 'English Skat or Hand');
  await dragOnto(page, '[data-testid="deck"]', '.mix-zone', 12);
  await page.waitForSelector('.declare');
  await page.click('[data-action="play-hand"]');
  await page.waitForSelector('.declare-hand');
  await englishScreen(page, 'table-declare-hand', 'hand declaration');
  await assertFits(page, 'English hand declaration');
  await page.click('[data-action="hand-back"]');
  await pickUpAndDiscard(page, 'English first game');
  await englishScreen(page, 'table-declare', 'declaration after the Skat');
  await assertFits(page, 'English declaration after the Skat');
  await page.click('[data-action="pick-game"][data-index="4"]');
  await page.click('[data-action="declare"]');
  await page.waitForSelector('.declare', { state: 'detached' });
  await page.waitForSelector('[data-action="play"]');
  const worth = await page.locator('[data-testid="value-preview"]').textContent();
  if (!/^\d+ × \d+( × \d+)?$/.test((worth ?? '').trim())) fail(`English: value preview while playing: "${worth}"`);
  await englishScreen(page, 'table-play', 'table while playing');
  await assertFits(page, 'English table while playing');
  await playOut(page, 'English first game');
  const result = await text(page, 'score-result');
  // The screen reader's live region speaks English too (it is .sr-only, so assertEnglish skips it).
  const said = ((await page.locator('[data-testid="sr-live"]').textContent().catch(() => '')) ?? '').trim();
  if (!said) fail('English: the live region said nothing during the game');
  const germanSaid = GERMAN_WORDS.exec(said);
  if (germanSaid || /\b(spielt|Punkte)\b/.test(said)) fail(`English: the live region speaks German: ${said.slice(0, 120)}`);
  log(`English: live region says "${said.slice(-80)}"`);
  await englishScreen(page, 'score', 'score board', '.overlay');
  log(`English: game played, score ${result}`);
  await page.click('[data-action="score-continue"]');
  await page.waitForSelector('.score-sheet', { state: 'detached' });
  await page.waitForSelector('[data-testid="unlock-new"]', { timeout: 10000 }).catch(() => fail('English: no unlock announced after the first pub'));
  await englishScreen(page, 'unlock', 'unlock', '.overlay');
  await page.click('.sheet-unlock [data-action="close-overlay"]');
  await page.waitForSelector('[data-testid="unlock-new"]', { state: 'detached' });

  // The card reward, the route, the Tresen.
  await page.waitForSelector('[data-testid="card-reward"]');
  await englishScreen(page, 'card-reward', 'card reward');
  await takeCardReward(page);
  await page.waitForSelector('[data-testid="map-svg"]');
  await englishScreen(page, 'route', 'route');
  await takeRoute(page, 'Tresen');
  await page.waitForSelector('[data-testid="shop"]');
  await englishScreen(page, 'shop', 'bar');
  await assertNoSideScroll(page, 'English bar');
  // An info card of a relic at the bar: the content texts come in English.
  const relic = page.locator('.shop [data-action="info-offer-relic"]').first();
  if (await relic.count()) {
    await relic.click();
    await page.waitForSelector('.info-card, [data-testid="info-card"]');
    await englishScreen(page, 'info-relic', 'relic info card');
    await page.keyboard.press('Escape');
  }
  // The menu in a run: rules card and glossary.
  await page.click('.hud [data-action="menu"]');
  await page.waitForSelector('[data-testid="setting-language"]');
  await englishScreen(page, 'menu-run', 'menu in a run', '.overlay');
  await page.click('[data-action="rules"]');
  await page.waitForSelector('.overlay .sheet');
  await englishScreen(page, 'rules', 'rules card', '.overlay');
  await page.click('.overlay [data-action="close-overlay"]');
  await page.click('.hud [data-action="menu"]');
  await page.click('[data-action="glossary"]');
  await page.waitForSelector('.overlay .sheet');
  await englishScreen(page, 'glossary', 'glossary', '.overlay');
  await page.click('.overlay [data-action="close-overlay"]');

  // An encounter (encounter.json: "Der Kartengeber").
  await resumeFixture(page, 'encounter.json');
  await page.waitForSelector('[data-testid="encounter"]');
  const title = ((await page.locator('[data-testid="encounter"]').textContent()) ?? '').replace(/\s+/g, ' ');
  if (!/The Dealer/.test(title)) fail(`English: the encounter is not in English: ${title.slice(0, 120)}`);
  await englishScreen(page, 'encounter', 'encounter');
  await assertNoSideScroll(page, 'English encounter', '[data-testid="encounter"]');
  await context.close();

  // The Einführung, played through in English.
  await tutorialPass(browser, errors, PHONE, null, 'en-GB');
  await tutorialPass(browser, errors, DESKTOP, null, 'en-GB');
  log('English: start, game, unlock, card reward, route, bar, menu, rules, glossary, encounter and introduction');
}

/**
 * Sounds v2 (soundsPass; alone with SMOKE_ONLY=sounds). There is no ear in
 * the test: an init script puts an array at `window.__soundLog`, and
 * ./src/audio.ts notes there every sound asked for, with what the mix did
 * with it (played, ducked, dropped, or silent when audio is locked) and its
 * start in ms of the audio clock. A scripted run at 390 × 844 (reduced
 * motion: the sounds must come anyway) and 1280 × 800 (full motion):
 * - a new run: the shuffle and a Stammgast's hummed quip, a quiet tap for
 *   buttons; a whole game (skat picked up, Drücken, a Grand declared with
 *   the bigger bang, cards, an Unter slammed, tricks, the result's knock
 *   and win or lose);
 * - kontra.json answered with Re: the slap; boss-tresen.json: buying rings
 *   the till, the boss sheet opens the creaking door; route-station.json:
 *   chalk for the chosen way; a Schnaps drunk yourself (Eierlikör): pour and
 *   gulp; a run won (boss-table.json): the counter bell for its unlock, the
 *   long night's church clock;
 * - the mix: two big sounds never start within 300 ms of each other.
 * Then the menu's volume sliders (soundVolumePass) in German and English at
 * 390 × 844 and 1280 × 800: the row "Lautstärke" under "Ton" with both
 * sliders at 80 % by default, moved by keyboard and by setting the value,
 * stored when let go and shown after a reload; older settings without the
 * volumes get 80 %, a broken value is put right; the menu still fits.
 */
async function soundContext(browser, options) {
  const context = await quietContext(browser, options);
  await context.addInitScript(() => {
    window.__soundLog = [];
  });
  return context;
}

async function soundNames(page) {
  return page.evaluate(() => window.__soundLog.map((e) => e.name));
}

async function expectSounds(page, want, label) {
  const deadline = Date.now() + 4000;
  let heard = [];
  for (;;) {
    heard = await soundNames(page);
    const missing = want.filter((n) => !heard.includes(n));
    if (!missing.length) break;
    if (Date.now() > deadline) {
      fail(`${label}: no sound "${missing.join('", "')}" asked for (heard: ${[...new Set(heard)].join(' ')})`);
    }
    await page.waitForTimeout(150);
  }
  log(`${label}: ${want.join(', ')}`);
}

async function clearSounds(page) {
  await page.evaluate(() => (window.__soundLog = []));
}

/** No two big sounds start within 300 ms (the later one is dropped by the mix). */
async function assertMix(page, label) {
  const BIG = ['bang', 'unlock', 'boss', 'night', 'win', 'lose'];
  const log_ = await page.evaluate(() => window.__soundLog);
  const big = log_.filter((e) => BIG.includes(e.name) && (e.outcome === 'played' || e.outcome === 'queued')).sort((a, b) => a.at - b.at);
  for (let i = 1; i < big.length; i++) {
    if (big[i].at - big[i - 1].at < 300) fail(`${label}: "${big[i - 1].name}" and "${big[i].name}" sound on top of each other (${big[i - 1].at} / ${big[i].at} ms)`);
  }
  const outcomes = new Set(log_.map((e) => e.outcome));
  const dropped = log_.filter((e) => e.outcome === 'dropped').map((e) => e.name);
  log(`${label}: mix ok (${log_.length} sounds, outcomes ${[...outcomes].join(', ')}${dropped.length ? `; dropped ${dropped.join(' ')}` : ''})`);
}

async function soundsPass(browser, errors, viewport, reducedMotion) {
  const tag = `${viewport.width}x${viewport.height}`;
  const context = await soundContext(browser, { viewport, reducedMotion });
  const page = await context.newPage();
  watch(page, errors);

  // A new run: shuffled, greeted, then one whole game.
  await page.goto(base);
  await page.waitForSelector('[data-action="new-run"]');
  await page.click('[data-action="new-run"]');
  await page.waitForSelector('[data-testid="skat-choice"]');
  await expectSounds(page, ['tap', 'shuffle', 'quip'], `sounds ${tag}: new run`);
  await clearSounds(page);
  await skatThenGrand(page, `sounds ${tag}`);
  await expectSounds(page, ['pickup', 'discard', 'bang'], `sounds ${tag}: skat, Drücken, Grand`);
  await playOut(page, `sounds ${tag}`);
  await page.waitForTimeout(1400);
  const game = await soundNames(page);
  const result = game.includes('win') ? 'win' : 'lose';
  await expectSounds(page, ['card', 'trick', 'knock', result], `sounds ${tag}: tricks and result`);
  if (!game.includes('slam')) log(`sounds ${tag}: no Unter came to the table in this game`);
  await assertMix(page, `sounds ${tag}: first game`);

  // Kontra answered with Re: the slap.
  await resumeFixture(page, 'kontra.json');
  await page.waitForSelector('[data-action="re"][data-re="true"]');
  await clearSounds(page);
  await page.click('[data-action="re"][data-re="true"]');
  await page.waitForSelector('[data-action="re"]', { state: 'detached' });
  await expectSounds(page, ['kontra'], `sounds ${tag}: Re`);

  // The Tresen: buying rings the till; the boss sheet opens the door.
  await resumeState(page, 'Tresen with money', patched('boss-tresen.json', (save) => (save.run.coins = 99)));
  await page.waitForSelector('[data-testid="shop"]');
  await clearSounds(page);
  const buy = page.locator('[data-action="buy-relic"]:not([disabled]), [data-action="buy-item"]:not([disabled])').first();
  if (!(await buy.count())) fail(`sounds ${tag}: nothing to buy at the Tresen with 99 Groschen`);
  await buy.click();
  await expectSounds(page, ['till'], `sounds ${tag}: bought at the Tresen`);
  await page.keyboard.press('Escape');
  await page.click('[data-action="leave-shop"]');
  await page.waitForSelector('[data-action="boss-go"]');
  await expectSounds(page, ['boss'], `sounds ${tag}: boss sheet`);
  await page.click('[data-action="boss-go"]');
  await page.waitForSelector('.declare');

  // The map: chalk for the chosen way.
  await resumeFixture(page, 'route-station.json');
  await page.waitForSelector('[data-testid="route-option"]');
  await clearSounds(page);
  await page.locator('[data-testid="route-option"]').first().click();
  await page.click('[data-testid="route-go"]');
  await expectSounds(page, ['chalk'], `sounds ${tag}: way chosen`);

  // A Schnaps drunk yourself: poured and gulped.
  await resumeState(page, 'eggnog', patched('wave3-mods.json', (save) => (save.run.consumables = ['eggnog'])));
  await page.waitForSelector('[data-testid="skat-choice"]');
  await page.waitForTimeout(400);
  await clearSounds(page);
  await drinkByTap(page, 0);
  await expectSounds(page, ['pour', 'gulp'], `sounds ${tag}: Eierlikör`);

  // A won run: the counter bell for an unlock, then the long night.
  const won = JSON.parse(fixture('boss-table.json'));
  won.run.series.section = 2;
  won.run.series.target = 1;
  won.run.series.score = 1000;
  won.run.series.games_played = 2;
  await resumeState(page, 'won run', JSON.stringify(won));
  await page.waitForSelector('.declare');
  await clearSounds(page);
  await skatThenGrand(page, `sounds ${tag} won run`);
  await playOut(page, `sounds ${tag} won run`);
  await page.click('[data-action="score-continue"]');
  await page.waitForSelector('[data-testid="end"] h1.is-won', { timeout: 10000 });
  await page.waitForTimeout(600);
  const unlocked = await page.locator('.sheet-unlock').count();
  if (unlocked) await expectSounds(page, ['unlock'], `sounds ${tag}: unlock`);
  else log(`sounds ${tag}: no unlock with this run (counter bell not checked)`);
  await assertMix(page, `sounds ${tag}: won run`);
  while (await page.locator('.sheet-unlock').count()) {
    await page.locator('.sheet-unlock [data-action="close-overlay"]').last().click();
    await page.waitForTimeout(200);
  }
  await clearSounds(page);
  await page.click('[data-testid="enter-night"]');
  await expectSounds(page, ['night'], `sounds ${tag}: the long night`);
  await context.close();
}

async function volumeShot(page, name) {
  const dir = process.env.SMOKE_SOUND_SHOTS;
  if (dir) {
    mkdirSync(dir, { recursive: true });
    await page.screenshot({ path: join(dir, `${name}.png`) });
  }
  await shoot(page, name);
}

async function soundVolumePass(browser, errors, viewport, locale) {
  const tag = `${viewport.width}x${viewport.height}-${locale.slice(0, 2)}`;
  const en = locale.startsWith('en');
  const context = await soundContext(browser, { viewport, reducedMotion: 'reduce', locale });
  const page = await context.newPage();
  watch(page, errors);
  const stored = () => page.evaluate(() => JSON.parse(localStorage.getItem('stammtisch.settings') ?? '{}'));
  await page.goto(base);
  await page.waitForSelector('[data-action="new-run"]');
  await page.click('[data-action="new-run"]');
  await page.waitForSelector('[data-testid="skat-choice"]');
  await page.click('[data-action="menu"]');
  await page.waitForSelector('[data-testid="setting-volume"]');
  const row = ((await page.locator('[data-testid="setting-volume"]').textContent()) ?? '').replace(/\s+/g, ' ').trim();
  const want = en ? ['Volume', 'Effects 80%', 'Music 80%'] : ['Lautstärke', 'Klänge 80 %', 'Musik 80 %'];
  for (const w of want) if (!row.includes(w)) fail(`volume ${tag}: the row reads "${row}", missing "${w}"`);
  for (const kind of ['sound', 'music']) {
    const box = await page.locator(`[data-testid="volume-${kind}"]`).boundingBox();
    if (!box || box.height < 44) fail(`volume ${tag}: the ${kind} slider is ${box?.height}px high, under 44`);
    if (box.x < 0 || box.x + box.width > viewport.width) fail(`volume ${tag}: the ${kind} slider leaves the screen`);
  }
  if (viewport.width < 600) await assertMenuNoScroll(page, `volume ${tag}`);
  else await assertMenuFits(page, `volume ${tag}`);
  await volumeShot(page, `volume-menu-${tag}`);

  // By keyboard: four steps down, stored when let go.
  const sfx = page.locator('[data-testid="volume-sound"] input');
  await sfx.focus();
  await clearSounds(page);
  for (let i = 0; i < 4; i++) await page.keyboard.press('ArrowLeft');
  await page.waitForTimeout(100);
  let settings = await stored();
  if (settings.soundVolume !== 60) fail(`volume ${tag}: four steps down stored ${settings.soundVolume}, expected 60`);
  const shown = await page.locator('[data-testid="volume-sound"] [data-vol-value]').textContent();
  if (shown !== (en ? '60%' : '60 %')) fail(`volume ${tag}: the effects show "${shown}" after four steps`);
  if ((await sfx.getAttribute('aria-valuetext')) !== shown) fail(`volume ${tag}: aria-valuetext does not follow`);
  if (!(await page.evaluate(() => document.activeElement?.matches('[data-volume="sound"]')))) fail(`volume ${tag}: the slider lost the focus`);
  if (!(await soundNames(page)).length) fail(`volume ${tag}: moving the effects' slider sounds nothing`);
  // Set directly: the music down to 25 (rounded to 25), the music still on.
  await page.locator('[data-testid="volume-music"] input').fill('25');
  await page.waitForTimeout(100);
  settings = await stored();
  if (settings.musicVolume !== 25 || settings.music !== true) fail(`volume ${tag}: music stored as ${JSON.stringify(settings)}`);
  await volumeShot(page, `volume-moved-${tag}`);
  // Music off: its slider dims and stays.
  await page.click('[data-testid="toggle-music"]');
  await page.waitForSelector('[data-testid="volume-music"].is-off');
  await page.click('[data-testid="toggle-music"]');
  await page.click('[data-testid="menu-close"]');

  // After a reload the values are back.
  await page.reload();
  await page.waitForSelector('[data-action="resume"]');
  await page.click('[data-action="resume"]');
  await page.waitForSelector('[data-testid="skat-choice"]');
  await page.click('[data-action="menu"]');
  await page.waitForSelector('[data-testid="setting-volume"]');
  const values = await page.evaluate(() => ['sound', 'music'].map((k) => document.querySelector(`[data-volume="${k}"]`)?.value));
  if (values.join(',') !== '60,25') fail(`volume ${tag}: after a reload the sliders show ${values.join(',')}`);

  // Older settings (no volumes) get 80; a broken value is put right.
  await page.evaluate(() => localStorage.setItem('stammtisch.settings', JSON.stringify({ tempo: 'normal', sound: true, music: true, thinking: true, hints: false, soundVolume: 250 })));
  await page.reload();
  await page.waitForSelector('[data-action="resume"]');
  await page.click('[data-action="resume"]');
  await page.waitForSelector('[data-testid="skat-choice"]');
  await page.click('[data-action="menu"]');
  await page.waitForSelector('[data-testid="setting-volume"]');
  const migrated = await page.evaluate(() => ['sound', 'music'].map((k) => document.querySelector(`[data-volume="${k}"]`)?.value));
  if (migrated.join(',') !== '100,80') fail(`volume ${tag}: old settings show ${migrated.join(',')}, expected 100,80`);
  log(`volume ${tag}: sliders in the menu, stored, reloaded, older settings migrated`);
  await context.close();
}

async function soundsAll(browser, errors) {
  await soundsPass(browser, errors, PHONE, 'reduce');
  await soundsPass(browser, errors, DESKTOP, 'no-preference');
  for (const locale of ['de-DE', 'en-GB']) {
    await soundVolumePass(browser, errors, PHONE, locale);
    await soundVolumePass(browser, errors, DESKTOP, locale);
  }
}

/**
 * The quips (Sprüche v2, DESIGN 10.4) of a language: per Stammgast and per
 * boss figure, situation -> lines. German from core/content/*.ron, English
 * from src/i18n/en-content.ts (by id, as the app puts them in place).
 */
async function quipLines(locale) {
  if (locale.startsWith('en')) {
    const { rolldown } = await import('rolldown');
    const bundle = await rolldown({ input: join(webDir, 'src', 'i18n', 'en-content.ts'), logLevel: 'silent' });
    const { output } = await bundle.generate({ format: 'esm' });
    await bundle.close();
    const { EN_CONTENT } = await import(`data:text/javascript;base64,${Buffer.from(output[0].code).toString('base64')}`);
    return { guests: Object.fromEntries(Object.entries(EN_CONTENT.guests).map(([id, g]) => [id, g.quips])), bosses: EN_CONTENT.figureQuips };
  }
  const blocks = (file) => {
    const src = readFileSync(resolve(webDir, '..', 'core', 'content', file), 'utf8')
      .split('\n')
      .filter((l) => !l.trim().startsWith('//'))
      .join('\n');
    const ids = [...src.matchAll(/\bid:\s*"([^"]+)"/g)];
    return Object.fromEntries(
      ids.map((m, i) => {
        const part = src.slice(m.index, ids[i + 1]?.index ?? src.length);
        const quips = [...part.matchAll(/quips:\s*\(([\s\S]*?)\n\s*\)/g)].map((q) =>
          Object.fromEntries([...q[1].matchAll(/(\w+):\s*\[([^\]]*)\]/g)].map((l) => [l[1], [...l[2].matchAll(/"([^"]*)"/g)].map((x) => x[1])])),
        );
        return [m[1], quips];
      }),
    );
  };
  return { guests: Object.fromEntries(Object.entries(blocks('guests.ron')).map(([id, [q]]) => [id, q])), bosses: blocks('bosses.ron') };
}

/** Every line of one situation over all boss figures. */
function bossLinesOf(lines, situation) {
  return Object.values(lines.bosses).flatMap((figures) => figures.flatMap((f) => f?.[situation] ?? []));
}

/** Note every bubble that shows (text, seat) and the most on screen at once, in `window.__quips`. */
async function spyQuips(page) {
  await page.evaluate(() => {
    const seen = { texts: [], most: 0 };
    window.__quips = seen;
    const look = () => {
      const now = [...document.querySelectorAll('[data-testid="quip"]')];
      seen.most = Math.max(seen.most, now.length);
      for (const q of now) {
        const key = `${q.closest('.opp')?.getAttribute('data-seat')}|${q.textContent.trim()}`;
        if (seen.texts[seen.texts.length - 1] !== key) seen.texts.push(key);
      }
    };
    new MutationObserver(look).observe(document.body, { childList: true, subtree: true });
    look();
  });
}

/** A boss figure's written-out line on a screen, checked against the lines of its situation. */
async function expectBossQuip(page, label, lines, situation) {
  const el = page.locator('[data-testid="boss-quip"]');
  if ((await el.count()) !== 1) fail(`${label}: no boss line on the screen (${situation})`);
  const said = ((await el.textContent()) ?? '').replace(/\s+/g, ' ').trim();
  const allowed = bossLinesOf(lines, situation);
  if (!allowed.some((l) => said.includes(l))) fail(`${label}: "${said}" is no ${situation} line of a boss figure`);
  log(`${label}: ${said}`);
  return said;
}

/**
 * Sprüche v2 (DESIGN 10.4 and 11; alone with SMOKE_ONLY=quips), in the
 * language of `locale`: the greeting of a new series is one of the
 * guest's greet lines and stands on the felt too; every bubble of a whole
 * game is a line of the one who speaks, never two at once and not many
 * (no bubble spam); the boss sheet names the left figure's greeting, the
 * right one greets at the table; the relic pick has a "beaten" line, the
 * end of a lost boss pub a "victory" line.
 */
async function quipsPass(browser, errors, locale, viewport = PHONE) {
  const tag = `quips-${locale.slice(0, 2)}${viewport === PHONE ? '' : '-wide'}`;
  const lines = await quipLines(locale);
  const german = locale.startsWith('de') ? null : await quipLines('de-DE');
  const context = await quietContext(browser, { viewport, reducedMotion: 'reduce', locale });
  const page = await context.newPage();
  watch(page, errors);
  await page.goto(base);
  await page.waitForSelector('[data-action="new-run"]');
  await fillSeed(page, SEED);
  await page.click('[data-action="new-run"]');
  await page.waitForSelector('[data-testid="skat-choice"]');
  await spyQuips(page);
  const guests = await page.locator('.opp').evaluateAll((els) => els.map((e) => ({ seat: e.getAttribute('data-seat'), id: e.getAttribute('data-guest') })));
  const bySeat = Object.fromEntries(guests.map((g) => [g.seat, lines.guests[g.id]]));
  if (guests.some((g) => !bySeat[g.seat])) fail(`${tag}: no lines for the guests ${JSON.stringify(guests)}`);
  const hello = page.locator('[data-testid="quip"]');
  if ((await hello.count()) !== 1) fail(`${tag}: ${await hello.count()} bubbles at the first game, expected the hello`);
  const helloSeat = await hello.evaluate((q) => q.closest('.opp')?.getAttribute('data-seat'));
  const helloText = ((await hello.textContent()) ?? '').trim();
  if (!bySeat[helloSeat]?.greet?.includes(helloText)) fail(`${tag}: the hello "${helloText}" is no greet line of seat ${helloSeat}`);
  if (german && Object.values(german.guests).some((q) => Object.values(q).flat().includes(helloText))) fail(`${tag}: the hello "${helloText}" is German`);
  if (await page.locator('[data-testid="table-talk"], .talk-line').count()) fail(`${tag}: the hello is written out on the felt beside its bubble`);
  const label = await hello.getAttribute('aria-label');
  if (!label?.endsWith(helloText) || (await hello.getAttribute('role')) !== 'status') fail(`${tag}: the bubble is no polite status for screen readers (${label})`);
  await assertGuests(page, `${tag} first game`);
  await shoot(page, `${tag}-hello`);
  log(`${tag}: hello "${helloText}" (seat ${helloSeat}), only in the bubble`);

  // A whole game: Skat, Grand, every card; each bubble a line of its speaker.
  await skatThenGrand(page, tag);
  let shot = false;
  for (let played = 0; ; ) {
    await page.waitForSelector('[data-action="play"], [data-action="re"][data-re="false"], .score-sheet', { timeout: 30000 });
    if (await page.locator('.score-sheet').count()) break;
    if (await page.locator('[data-action="re"][data-re="false"]').count()) {
      await page.click('[data-action="re"][data-re="false"]');
      continue;
    }
    if (!shot && (await page.locator('[data-testid="quip"]').count())) {
      const text = ((await page.locator('[data-testid="quip"]').textContent()) ?? '').trim();
      if (text !== helloText) {
        await shoot(page, `${tag}-ingame`);
        shot = true;
      }
    }
    await page.locator('[data-action="play"]').first().click();
    if (++played > 12) fail(`${tag}: played more than ten cards`);
  }
  if (await page.locator('[data-action="score-skip"].btn').count()) await page.locator('[data-action="score-skip"].btn').click();
  await page.waitForSelector('[data-action="score-continue"]');
  await page.click('[data-action="score-continue"]');
  await page.waitForSelector('[data-testid="skat-choice"], .declare', { timeout: 30000 });
  await page.waitForTimeout(300);
  const spied = await page.evaluate(() => window.__quips);
  if (spied.most > 1) fail(`${tag}: ${spied.most} bubbles on the table at once`);
  for (const key of spied.texts) {
    const [seat, ...rest] = key.split('|');
    const said = rest.join('|');
    if (!Object.values(bySeat[seat] ?? {}).flat().includes(said)) fail(`${tag}: seat ${seat} said "${said}", none of his lines`);
  }
  // Hello plus about one a game; never a bubble every trick.
  if (spied.texts.length > 5) fail(`${tag}: ${spied.texts.length} bubbles in one game: ${spied.texts.join(' / ')}`);
  log(`${tag}: ${spied.texts.length} bubble(s) in the first game: ${spied.texts.join(' / ')}`);

  // The boss sheet: the left figure's greeting; at the table the right one says hello.
  await resumeFixture(page, 'boss-tresen.json');
  await page.waitForSelector('[data-testid="shop"]');
  await page.click('[data-action="leave-shop"]');
  await page.waitForSelector('[data-action="boss-go"]');
  const sheet = await expectBossQuip(page, `${tag} boss sheet`, { bosses: { twins: [lines.bosses.twins[0]] } }, 'greet');
  await shoot(page, `${tag}-boss-sheet`);
  await page.click('[data-action="boss-go"]');
  await page.waitForSelector('.declare');
  if (await page.locator('.sheet-unlock').count()) await page.click('.sheet-unlock [data-action="close-overlay"]');
  const franz = page.locator('.opp[data-seat="2"] [data-testid="quip"]');
  await franz.waitFor({ timeout: 5000 }).catch(() => fail(`${tag}: the right boss player does not say hello at the boss table`));
  const franzSays = ((await franz.textContent()) ?? '').trim();
  if (!lines.bosses.twins[1].greet.includes(franzSays)) fail(`${tag}: Franz says "${franzSays}", no greeting of his`);
  if (sheet.includes(franzSays)) fail(`${tag}: the table repeats the sheet's line`);
  await assertGuests(page, `${tag} boss table`, { boss: true });
  await shoot(page, `${tag}-boss-table`);

  // Beaten: the relic pick after a won boss pub.
  await resumeFixture(page, 'boss-reward.json');
  await page.waitForSelector('[data-testid="boss-reward"]');
  await expectBossQuip(page, `${tag} boss reward`, lines, 'beaten');
  await assertNoSideScroll(page, `${tag} boss reward`, '.boss-reward');
  await shoot(page, `${tag}-boss-reward`);

  // Victorious: the end of a lost boss pub (Der Schieber: Hand, Grand).
  await resumeFixture(page, 'boss-lost.json');
  await page.waitForSelector('.declare[data-phase="Declaring"]');
  await page.click('[data-action="pick-game"][data-index="4"]');
  await page.click('[data-action="declare"]');
  await page.waitForSelector('.declare', { state: 'detached' });
  await playOut(page, `${tag} lost boss pub`);
  await page.click('[data-action="score-continue"]');
  await page.waitForSelector('[data-testid="end"]');
  await expectBossQuip(page, `${tag} end`, lines, 'victory');
  await shoot(page, `${tag}-end`);
  await context.close();
}

/**
 * Der Umzug (ADR 0025; alone with SMOKE_ONLY=move). The game moved to
 * one-more-skat.de; the old sslip.io address stays up and must not
 * redirect, or its localStorage could not be reached any more. Both
 * addresses are served here from the preview through Playwright routes
 * (so the real host names of src/domain.ts are what decides; the service
 * worker is blocked in this context, its requests would bypass the routes):
 * - localhost (every other pass) and the new address show no notice; the
 *   old one shows "Wir sind umgezogen" on the start screen; the account
 *   sheet of the new address tells where old passkeys still work,
 * - without an account the sheet puts the save code into the link's
 *   fragment; following it, the new address asks before replacing, takes
 *   the address bar's fragment away, and after the reload the run of the
 *   old address is there with every key as it was,
 * - signed in (stubbed /api), the sheet uploads first, asks for a move
 *   code and links `#umzug=<code>`; the new address redeems it (the code
 *   goes in the body, never in a URL), shows the account and offers a
 *   passkey for the new address; a used code shows why and the way back.
 */
async function movePass(browser, errors, viewport) {
  const tag = `move ${viewport.width}x${viewport.height}`;
  const OLD = 'https://stammtisch-62-83-23-90.sslip.io';
  const NEW = 'https://one-more-skat.de';
  const local = base.replace(/\/$/, '');
  /** Serve both hosts from the preview; `api` answers /api/* where it wants to. */
  const serve = async (context, api = () => null) => {
    for (const origin of [OLD, NEW]) {
      await context.route(`${origin}/**`, async (route) => {
        const url = new URL(route.request().url());
        const stub = url.pathname.startsWith('/api/') ? api(url, route.request(), origin) : null;
        if (stub) return route.fulfill({ status: stub.status ?? 200, contentType: 'application/json', body: JSON.stringify(stub.body) });
        if (url.pathname.startsWith('/api/')) return route.fulfill({ status: 502, contentType: 'application/json', body: '{"error":"offline"}' });
        const response = await route.fetch({ url: `${local}${url.pathname}${url.search}` });
        return route.fulfill({ response });
      });
    }
  };
  const snap = (page, name) => page.screenshot({ path: join(screens, `${name}-${viewport.width}x${viewport.height}.png`) });

  // 1. Without an account: the progress travels in the link.
  {
    const context = await quietContext(browser, { viewport, reducedMotion: 'reduce', serviceWorkers: 'block' });
    await serve(context);
    const page = await context.newPage();
    watch(page, errors);
    await page.goto(base);
    await page.waitForSelector('[data-action="new-run"]');
    if (await page.locator('[data-testid="move-notice"]').count()) fail(`${tag}: notice on localhost`);
    await page.goto(`${NEW}/`);
    await page.waitForSelector('[data-action="new-run"]');
    if (await page.locator('[data-testid="move-notice"]').count()) fail(`${tag}: notice on the new address`);
    // Signing in there: a hint for passkeys of the old address.
    await page.click('[data-testid="account-open"]');
    await page.waitForSelector('[data-testid="account-sheet"][data-state="out"]');
    if (!(await page.locator('[data-testid="account-old-hint"]').count())) fail(`${tag}: no hint for old passkeys on the new address`);
    await page.click('.sheet [data-action="menu"]');

    const oldRes = await page.goto(`${OLD}/`);
    if (oldRes?.status() !== 200 || new URL(page.url()).origin !== OLD) fail(`${tag}: the old address does not stay (${oldRes?.status()} ${page.url()})`);
    await page.waitForSelector('[data-testid="move-notice"]').catch(() => fail(`${tag}: no notice on the old address`));
    await fillSeed(page, SEED);
    await page.click('[data-action="new-run"]');
    await page.waitForSelector('[data-testid="skat-choice"]');
    await page.click('.hud [data-action="menu"]');
    await page.click('.sheet [data-action="to-start"]');
    await page.waitForSelector('[data-action="resume"]');
    await page.waitForSelector('[data-testid="move-notice"]');
    await assertNoSideScroll(page, `${tag}, start with the notice`, '.move-notice');
    await snap(page, 'move-notice');
    const keysBefore = await page.evaluate(() => Object.fromEntries(Object.keys(localStorage).filter((k) => k.startsWith('stammtisch.')).map((k) => [k, localStorage.getItem(k)])));
    if (!keysBefore['stammtisch.run']) fail(`${tag}: no run stored on the old address`);

    await page.click('[data-testid="move-open"]');
    await page.waitForSelector('[data-testid="move-sheet"][data-state="save"]');
    await page.click('[data-testid="move-make"]');
    await page.waitForSelector('[data-testid="move-sheet"][data-state="link"]');
    await snap(page, 'move-link');
    const href = await page.getAttribute('[data-testid="move-go"]', 'href');
    if (!href?.startsWith(`${NEW}/#spielstand=STAMMTISCH`)) fail(`${tag}: link ${href?.slice(0, 80)}`);
    await page.click('[data-testid="move-go"]');
    await page.waitForURL(`${NEW}/**`);
    await page.waitForSelector('[data-testid="save-moved"]', { timeout: 20000 }).catch(() => fail(`${tag}: the new address does not offer the progress`));
    if (new URL(page.url()).hash) fail(`${tag}: the save code stays in the address bar`);
    if (await page.evaluate(() => localStorage.getItem('stammtisch.run'))) fail(`${tag}: replaced before asking`);
    await page.click('[data-testid="save-apply"]');
    await page.waitForSelector('[data-action="resume"]', { timeout: 20000 });
    const keysAfter = await page.evaluate(() => Object.fromEntries(Object.keys(localStorage).filter((k) => k.startsWith('stammtisch.')).map((k) => [k, localStorage.getItem(k)])));
    for (const [k, v] of Object.entries(keysBefore)) {
      if (keysAfter[k] !== v) fail(`${tag}: ${k} did not come along`);
    }
    if (await page.locator('[data-testid="move-notice"]').count()) fail(`${tag}: notice after the move`);
    log(`${tag}: progress moved in the link (${href.length} characters, ${Object.keys(keysBefore).length} keys)`);
    await context.close();
  }

  // 2. Signed in: a one-time move code takes the account along.
  {
    const context = await quietContext(browser, { viewport, reducedMotion: 'reduce', serviceWorkers: 'block' });
    const user = { name: 'Uschi', passkeys: 1, createdAt: 1 };
    const seen = { put: 0, start: 0, redeem: [], urls: [] };
    let used = false;
    await serve(context, (url, req, origin) => {
      seen.urls.push(url.href);
      const p = url.pathname;
      if (p === '/api/me') return { body: { user: origin === OLD || seen.redeem.length ? user : null } };
      if (p === '/api/save' && req.method() === 'GET') return { body: { save: null } };
      if (p === '/api/save' && req.method() === 'PUT') {
        seen.put += 1;
        return { body: { version: seen.put, updatedAt: Date.now() } };
      }
      if (p === '/api/move/start' && origin === OLD) {
        seen.start += 1;
        return { body: { code: 'Zm9vYmFyLWNvZGU', expiresAt: Date.now() + 600000 } };
      }
      if (p === '/api/move/redeem' && origin === NEW) {
        seen.redeem.push(req.postDataJSON());
        if (used) return { status: 401, body: { error: 'move_code' } };
        used = true;
        return { body: { user } };
      }
      return null;
    });
    // Signed in on the old address only (the sync record belongs to it).
    await context.addInitScript((old) => {
      if (location.origin === old && !localStorage.getItem('stammtisch-sync')) {
        localStorage.setItem('stammtisch-sync', JSON.stringify({ signedIn: true, version: null, hash: null, localAt: 0, name: 'Uschi' }));
      }
    }, OLD);
    const page = await context.newPage();
    watch(page, errors);
    await page.goto(`${OLD}/`);
    await page.waitForSelector('[data-testid="move-notice"]');
    await page.click('[data-testid="move-open"]');
    await page.waitForSelector('[data-testid="move-sheet"][data-state="account"]');
    await page.click('[data-testid="move-make"]');
    await page.waitForSelector('[data-testid="move-sheet"][data-state="link"]', { timeout: 15000 }).catch(async () => fail(`${tag}: no account link (${await text(page, 'move-error')})`));
    const href = await page.getAttribute('[data-testid="move-go"]', 'href');
    if (href !== `${NEW}/#umzug=Zm9vYmFyLWNvZGU`) fail(`${tag}: account link ${href}`);
    if (seen.start !== 1) fail(`${tag}: ${seen.start} move codes asked for`);
    await page.click('[data-testid="move-go"]');
    await page.waitForURL(`${NEW}/**`);
    await page.waitForSelector('[data-testid="moved-sheet"][data-state="done"]', { timeout: 20000 }).catch(() => fail(`${tag}: the account did not arrive`));
    if (new URL(page.url()).hash) fail(`${tag}: the move code stays in the address bar`);
    if (seen.redeem.length !== 1 || seen.redeem[0]?.code !== 'Zm9vYmFyLWNvZGU') fail(`${tag}: redeemed ${JSON.stringify(seen.redeem)}`);
    if (seen.urls.some((u) => u.includes('Zm9vYmFyLWNvZGU'))) fail(`${tag}: the move code was in a request URL`);
    if (!(await page.locator('[data-testid="moved-sheet"]').textContent())?.includes('Uschi')) fail(`${tag}: the moved sheet does not name the account`);
    if (!(await page.locator('[data-testid="moved-add-passkey"]').count())) fail(`${tag}: no passkey offered for the new address`);
    await snap(page, 'move-arrived');
    await page.click('[data-testid="moved-done"]');
    await page.waitForSelector('[data-testid="moved-sheet"]', { state: 'detached' });
    // The same link again: used up, with the way back.
    await page.goto(`${NEW}/#umzug=Zm9vYmFyLWNvZGU`);
    await page.waitForSelector('[data-testid="moved-error"]', { timeout: 20000 }).catch(() => fail(`${tag}: a used code shows no error`));
    log(`${tag}: account moved with a one-time code (${seen.put} upload(s) before), a used code refused`);
    await context.close();
  }
}

/**
<<<<<<< HEAD
 * Impressum, Datenschutz, Hilfe (alone with SMOKE_ONLY=legal): the static
 * pages beside the game (web/impressum.html and the like) answer 200 under
 * their clean addresses in German and English, carry no script, link to
 * each other and to their translation, and scroll nowhere sideways at
 * 360 px, 390 px and on the desktop. The start screen's footer and the menu
 * link to them (the menu through its Abspann) in the active language and
 * open them in a new tab. Offline
 * the service worker serves a page's own cached copy, never the game.
 */
async function legalPass(browser, errors) {
  const PAGES = {
    de: { imprint: '/impressum', privacy: '/datenschutz', support: '/support' },
    en: { imprint: '/en/imprint', privacy: '/en/privacy', support: '/en/support' },
  };
  const H1 = { '/impressum': 'Impressum', '/datenschutz': 'Datenschutzerklärung', '/support': 'Hilfe', '/en/imprint': 'Imprint', '/en/privacy': 'Privacy policy', '/en/support': 'Help' };
  const paths = Object.keys(H1);
  const sideways = (page) => page.evaluate(() => document.documentElement.scrollWidth - window.innerWidth);
  for (const viewport of [PHONE, { width: 360, height: 740 }, DESKTOP]) {
    const tag = `${viewport.width}x${viewport.height}`;
    const context = await quietContext(browser, { viewport, reducedMotion: 'reduce' });
    const page = await context.newPage();
    watch(page, errors);
    for (const path of paths) {
      const response = await page.goto(new URL(path, base).href);
      if (response?.status() !== 200) fail(`legal ${tag}: ${path} answers ${response?.status()}`);
      const h1 = (await page.locator('h1').textContent())?.replace(/­/g, '').trim();
      if (h1 !== H1[path]) fail(`legal ${tag}: ${path} shows "${h1}", not "${H1[path]}"`);
      if (await page.locator('script').count()) fail(`legal ${tag}: ${path} carries a script`);
      if (!(await page.locator('a[href="mailto:kontakt@one-more-skat.de"]').count())) fail(`legal ${tag}: ${path} has no contact address`);
      const over = await sideways(page);
      if (over > 0) fail(`legal ${tag}: ${path} scrolls ${over} px sideways`);
      // Every own link on the page answers 200 (the other pages, the translation, the game).
      const hrefs = await page.$$eval('a[href^="/"]', (as) => [...new Set(as.map((a) => a.getAttribute('href')))]);
      for (const href of hrefs) {
        const res = await page.request.get(new URL(href, base).href);
        if (res.status() !== 200) fail(`legal ${tag}: ${path} links to ${href}, which answers ${res.status()}`);
      }
      if (viewport === PHONE || viewport === DESKTOP) {
        const name = path.replace(/^\//, '').replace('/', '-');
        await page.screenshot({ path: join(screens, `legal-${name}-${tag}.png`), fullPage: true });
      }
    }
    // The start screen's footer and the menu link to the German pages.
    await page.goto(base);
    await page.waitForSelector('[data-action="new-run"]');
    for (const [where, open] of [
      ['start-legal', async () => {}],
      ['credits-legal', async () => {
        // The menu has no room for another row: the Abspann in it carries the links.
        await page.click('[data-action="settings"]');
        await page.click('.sheet-menu [data-action="credits"]');
      }],
    ]) {
      await open();
      const links = page.locator(`[data-testid="${where}"] a`);
      await links.first().waitFor().catch(() => fail(`legal ${tag}: no links in ${where}`));
      const hrefs = await links.evaluateAll((as) => as.map((a) => a.getAttribute('href')));
      const want = Object.values(PAGES.de);
      if (JSON.stringify(hrefs) !== JSON.stringify(want)) fail(`legal ${tag}: ${where} links ${JSON.stringify(hrefs)}`);
      for (const a of await links.all()) {
        const box = await a.boundingBox();
        if (!box || box.height < 44) fail(`legal ${tag}: a link in ${where} is lower than 44 px`);
        if ((await a.getAttribute('target')) !== '_blank') fail(`legal ${tag}: ${where} opens the page in the game's tab`);
      }
      // A tap opens the page in a new tab.
      const [tab] = await Promise.all([context.waitForEvent('page'), links.nth(1).click()]);
      await tab.waitForLoadState();
      if (new URL(tab.url()).pathname !== '/datenschutz') fail(`legal ${tag}: ${where} opened ${tab.url()}`);
      if (!(await tab.locator('h1').textContent())?.includes('Datenschutz')) fail(`legal ${tag}: ${where} did not open the privacy policy`);
      await tab.close();
      if (await sideways(page)) fail(`legal ${tag}: ${where} scrolls sideways`);
    }
    await context.close();
  }
  // English: the footer links to the English pages.
  {
    const context = await quietContext(browser, { viewport: PHONE, reducedMotion: 'reduce', locale: 'en-GB' });
    const page = await context.newPage();
    watch(page, errors);
    await page.goto(base);
    await page.waitForSelector('[data-testid="start-legal"] a');
    const hrefs = await page.locator('[data-testid="start-legal"] a').evaluateAll((as) => as.map((a) => a.getAttribute('href')));
    if (JSON.stringify(hrefs) !== JSON.stringify(Object.values(PAGES.en))) fail(`legal en: footer links ${JSON.stringify(hrefs)}`);
    const labels = await page.locator('[data-testid="start-legal"] a').allTextContents();
    if (labels.join('|') !== 'Imprint|Privacy|Help') fail(`legal en: footer says ${labels.join('|')}`);
    await context.close();
  }
  // Offline the service worker serves the page itself, not the game.
  {
    const context = await quietContext(browser, { viewport: PHONE, reducedMotion: 'reduce' });
    const page = await context.newPage();
    watch(page, errors);
    await page.goto(base);
    await page.waitForSelector('[data-action="new-run"]');
    await page.waitForFunction(() => navigator.serviceWorker.controller !== null, null, { timeout: 30000 }).catch(() => fail('legal offline: no service worker in control'));
    await context.setOffline(true);
    for (const path of ['/datenschutz', '/en/support']) {
      await page.goto(new URL(path, base).href);
      const h1 = (await page.locator('h1').textContent({ timeout: 10000 }).catch(() => ''))?.replace(/­/g, '').trim();
      if (h1 !== H1[path]) fail(`legal offline: ${path} shows "${h1}" without network`);
    }
    await context.setOffline(false);
    await context.close();
  }
  log('legal: imprint, privacy policy and help in German and English, linked from the start screen and the menu, offline too');
=======
 * Fehler melden (ADR 0027; alone with SMOKE_ONLY=report). /api/reports is
 * stubbed with a Playwright route:
 * - a dead tap (a button after which nothing changes) is noted in the
 *   diagnosis buffer and kept over a reload; the report sheet then says
 *   the last session's log goes along,
 * - menu → "Fehler melden" → text → "Senden": the body has platform and
 *   version, the report the text, the buffer (taps, screens), the build,
 *   the save code and the last session; no session token or cookie, no
 *   account name; the number from the server is shown,
 * - offline the report waits in the queue (the menu says so), and goes
 *   out by itself once the browser is online again, with a toast.
 */
async function reportPass(browser, errors, viewport) {
  const tag = `report ${viewport.width}x${viewport.height}`;
  const context = await quietContext(browser, { viewport, deviceScaleFactor: 2, reducedMotion: 'reduce', serviceWorkers: 'block' });
  const posted = [];
  let offline = false;
  let next = 0;
  await context.route('**/api/reports', async (route) => {
    const req = route.request();
    if (offline) return route.abort('internetdisconnected');
    posted.push({ body: req.postData() ?? '', headers: await req.allHeaders() });
    next += 1;
    return route.fulfill({ status: 201, contentType: 'application/json', body: JSON.stringify({ id: next === 1 ? 'B-7K3Q' : `B-9X${next}Z` }) });
  });
  const page = await context.newPage();
  watch(page, errors);
  const snap = (name) => page.screenshot({ path: join(screens, `${name}-${viewport.width}x${viewport.height}.png`) });
  await page.goto(base);
  await page.waitForSelector('[data-action="new-run"]');
  await fillSeed(page, SEED);
  await page.click('[data-action="new-run"]');
  await page.waitForSelector('[data-testid="skat-choice"]');
  // A dead tap: a button whose action changes nothing (no dialog to close).
  await page.evaluate(() => {
    const b = document.createElement('button');
    b.dataset.action = 'close-overlay';
    b.dataset.testid = 'dead-button';
    document.querySelector('.screen')?.appendChild(b);
  });
  await page.waitForTimeout(300);
  await page.evaluate(() => document.querySelector('[data-testid="dead-button"]')?.click());
  await page.waitForFunction(() => (localStorage.getItem('stammtisch-diag.last') ?? '').includes('dead-tap'), null, { timeout: 5000 }).catch(() => fail(`${tag}: the dead tap was not noted`));
  await page.reload();
  await page.waitForSelector('[data-testid="skat-choice"], [data-action="resume"]');
  if (await page.locator('[data-action="resume"]').count()) {
    await page.click('[data-action="resume"]');
    await page.waitForSelector('[data-testid="skat-choice"]');
  }

  // 1. Online: the report goes out and its number shows.
  await page.click('.hud [data-action="menu"]');
  await page.click('[data-testid="report-open"]');
  await page.waitForSelector('[data-testid="report-sheet"][data-state="form"]');
  if (!(await page.locator('[data-testid="report-previous"]').count())) fail(`${tag}: no hint that the last session's log goes along`);
  if (!(await page.isChecked('[data-testid="report-save"]'))) fail(`${tag}: "Spielstand mitschicken" is not ticked by default`);
  await snap('report-form');
  await page.click('[data-testid="report-what"] summary');
  await page.waitForSelector('[data-testid="report-what"][open]');
  await snap('report-what');
  await assertNoSideScroll(page, `${tag}, report sheet`, '.sheet-report');
  await page.fill('[data-testid="report-text"]', 'Weiter reagiert nicht');
  await page.click('[data-testid="report-send"]');
  await page.waitForSelector('[data-testid="report-sheet"][data-state="sent"]', { timeout: 15000 }).catch(() => fail(`${tag}: the report was not sent`));
  if ((await text(page, 'report-id')) !== 'B-7K3Q') fail(`${tag}: report number ${await text(page, 'report-id')}`);
  await snap('report-sent');
  if (posted.length !== 1) fail(`${tag}: ${posted.length} reports posted`);
  const first = JSON.parse(posted[0].body);
  if (first.platform !== 'web' || !first.version) fail(`${tag}: platform ${first.platform}, version ${first.version}`);
  const body = JSON.parse(first.report);
  if (body.message !== 'Weiter reagiert nicht') fail(`${tag}: message ${body.message}`);
  if (!body.save?.startsWith('STAMMTISCH')) fail(`${tag}: no save code in the report (${body.saveNote})`);
  if (!body.app?.build || !body.viewport || !body.settings?.tempo || typeof body.settings.reducedMotion !== 'boolean') fail(`${tag}: device facts missing ${JSON.stringify(body.app)} ${body.viewport}`);
  if (!body.log?.some((e) => e.k === 'tap' && e.m.startsWith('report-open'))) fail(`${tag}: the tap on "Fehler melden" is not in the log`);
  if (!body.log?.some((e) => e.k === 'screen' && e.m.startsWith('run'))) fail(`${tag}: no screen in the log`);
  if (!body.previous?.log?.some((e) => e.k === 'dead-tap' && e.m.includes('close-overlay'))) fail(`${tag}: the last session's dead tap is not in the report`);
  const h = posted[0].headers;
  if (h.authorization || h.cookie) fail(`${tag}: credentials sent with the report`);
  if (/Bearer|stammtisch_session|x-session-token/i.test(posted[0].body)) fail(`${tag}: a token in the report`);
  if (Buffer.byteLength(posted[0].body) > 256 * 1024) fail(`${tag}: report of ${Buffer.byteLength(posted[0].body)} bytes`);
  await page.click('[data-testid="report-close"]');
  await page.waitForSelector('[data-testid="report-sheet"]', { state: 'detached' });

  // 2. Offline: it waits, the menu says so, and it goes once online again.
  offline = true;
  await context.setOffline(true);
  await page.click('.hud [data-action="menu"]');
  await page.click('[data-testid="report-open"]');
  await page.waitForSelector('[data-testid="report-sheet"][data-state="form"]');
  await page.fill('[data-testid="report-text"]', 'Offline gemeldet');
  await page.uncheck('[data-testid="report-save"]');
  await page.click('[data-testid="report-send"]');
  await page.waitForSelector('[data-testid="report-sheet"][data-state="queued"]', { timeout: 15000 }).catch(() => fail(`${tag}: an offline report does not wait`));
  await snap('report-queued');
  const waiting = await page.evaluate(() => JSON.parse(localStorage.getItem('stammtisch-diag.queue') ?? '[]').length);
  if (waiting !== 1) fail(`${tag}: ${waiting} reports in the queue`);
  // Copy as text: the report as it would go.
  await page.click('[data-testid="report-as-text"]');
  await page.waitForSelector('[data-testid="report-sheet"][data-state="text"]');
  if (!(await page.inputValue('[data-testid="report-json"]')).includes('Offline gemeldet')) fail(`${tag}: the text view lacks the report`);
  await page.click('[data-action="report-back"]');
  await page.waitForSelector('[data-testid="report-sheet"][data-state="queued"]');
  await page.click('[data-testid="report-close"]');
  await page.click('.hud [data-action="menu"]');
  if (!(await text(page, 'report-open')).includes('1')) fail(`${tag}: the menu does not say a report waits (${await text(page, 'report-open')})`);
  await page.click('[data-testid="menu-close"]');
  offline = false;
  await context.setOffline(false);
  await page.waitForFunction(() => !localStorage.getItem('stammtisch-diag.queue'), null, { timeout: 15000 }).catch(() => fail(`${tag}: the queued report was not sent once online`));
  await page.waitForSelector('.toast-layer .toast:has-text("B-9X2Z")', { timeout: 10000 }).catch(() => fail(`${tag}: no note that the queued report went out`));
  if (posted.length !== 2) fail(`${tag}: ${posted.length} reports posted in all`);
  const second = JSON.parse(JSON.parse(posted[1].body).report);
  if (second.message !== 'Offline gemeldet' || second.save !== null || second.saveNote !== 'off') fail(`${tag}: queued report ${second.message} save ${second.saveNote}`);
  log(`${tag}: report sent (B-7K3Q, ${Buffer.byteLength(posted[0].body)} bytes with save), dead tap of the last session included, offline report queued and sent later`);
  await context.close();
>>>>>>> origin/main
}

async function run() {
  const preview = await startPreview();
  const browser = await chromium.launch();
  // German stays the default (ADR 0023): every context gets a German
  // browser unless it asks for another locale (the English pass).
  const newContext = browser.newContext.bind(browser);
  browser.newContext = (options = {}) => newContext({ locale: 'de-DE', ...options });
  const errors = [];
  try {
    const reducedMotion = process.env.SMOKE_FULL_MOTION ? 'no-preference' : 'reduce';
    // SMOKE_ONLY=night: only the long night (ADR 0020), for quick checks.
    // SMOKE_ONLY=curse: the curse of a deal is seen (playtest 12.11.2026).
    if (process.env.SMOKE_ONLY === 'curse') {
      await curseScenePass(browser, errors);
      return report(errors);
    }
    // SMOKE_ONLY=feel: the table with full motion (throw, pick-up, Drücken), phone and desktop.
    if (process.env.SMOKE_ONLY === 'feel') {
      await skatStaysPass(browser, errors);
      await tableFeelPass(browser, errors, PHONE);
      await tableFeelPass(browser, errors, DESKTOP);
      return report(errors);
    }
    if (process.env.SMOKE_ONLY === 'weiter') {
      await scoreContinuePass(browser, errors);
      await scoreTapPass(browser, errors);
      return report(errors);
    }
    // SMOKE_ONLY=undead: the Untot looks and the crumble (playtest 05.11.2026).
    if (process.env.SMOKE_ONLY === 'undead') {
      await framesPass(browser, errors, reducedMotion);
      await voucherPass(browser, errors);
      await undeadPass(browser, errors, reducedMotion);
      await undeadGlowPass(browser, errors);
      return report(errors);
    }
    // SMOKE_ONLY=bosses: the boss passes with the route legend and the names on the map.
    if (process.env.SMOKE_ONLY === 'bosses') {
      await bossesV2(browser, errors, reducedMotion);
      return report(errors);
    }
    // SMOKE_ONLY=doppel: the Zinken v2 looks and the Doppelgänger's mirror (07.11.2026).
    if (process.env.SMOKE_ONLY === 'doppel') {
      await zinkenV2Pass(browser, errors, PHONE);
      await zinkenV2Pass(browser, errors, DESKTOP);
      return report(errors);
    }
    // SMOKE_ONLY=tutorial: the Einführung (DESIGN 16.1).
    if (process.env.SMOKE_ONLY === 'offline') {
      await offlinePass(browser, errors);
      return report(errors);
    }
    if (process.env.SMOKE_ONLY === 'tutorial') {
      await tutorialPass(browser, errors, PHONE, process.env.SMOKE_TUTORIAL_SHOTS ?? null);
      await tutorialPass(browser, errors, DESKTOP, process.env.SMOKE_TUTORIAL_SHOTS ?? null);
      return report(errors);
    }
    if (process.env.SMOKE_ONLY === 'qol') {
      await qolPass(browser, errors, PHONE);
      await qolPass(browser, errors, DESKTOP);
      await handSortPass(browser, errors, PHONE);
      await handSortPass(browser, errors, DESKTOP);
      await shareRunPass(browser, errors, PHONE);
      await shareRunPass(browser, errors, DESKTOP);
      return report(errors);
    }
    // SMOKE_ONLY=move: the move to one-more-skat.de (ADR 0025).
    if (process.env.SMOKE_ONLY === 'move') {
      await movePass(browser, errors, PHONE);
      await movePass(browser, errors, DESKTOP);
      return report(errors);
    }
    // SMOKE_ONLY=report: "Fehler melden" (ADR 0027).
    if (process.env.SMOKE_ONLY === 'report') {
      await reportPass(browser, errors, PHONE);
      await reportPass(browser, errors, DESKTOP);
      return report(errors);
    }
    if (process.env.SMOKE_ONLY === 'weekly') {
      await weeklyPass(browser, errors, PHONE);
      await weeklyPass(browser, errors, DESKTOP);
      return report(errors);
    }
    // SMOKE_ONLY=a11y: Barrierefreiheit (colour-blind aid, text size, screen reader, focus).
    if (process.env.SMOKE_ONLY === 'a11y') {
      await a11yPass(browser, errors, reducedMotion);
      return report(errors);
    }
    // SMOKE_ONLY=english: the English game (ADR 0023).
    if (process.env.SMOKE_ONLY === 'english') {
      await englishPass(browser, errors);
      return report(errors);
    }
    // SMOKE_ONLY=paper: the paper layer (DESIGN 16.6).
    if (process.env.SMOKE_ONLY === 'paper') {
      await paperPass(browser, errors);
      return report(errors);
    }
    // SMOKE_ONLY=tresen: the layout of the Tresen (DESIGN 16.5).
    if (process.env.SMOKE_ONLY === 'tresen') {
      await tresenPass(browser, errors);
      return report(errors);
    }
    // SMOKE_ONLY=characters: the start screen's characters and the two of 10.11.2026.
    if (process.env.SMOKE_ONLY === 'characters') {
      await charactersPass(browser, errors);
      await newCharactersPass(browser, errors);
      await startScreenPass(browser, errors);
      return report(errors);
    }
    // SMOKE_ONLY=sounds: the sounds asked for in a scripted run and the volume sliders.
    if (process.env.SMOKE_ONLY === 'sounds') {
      await soundsAll(browser, errors);
      return report(errors);
    }
    // SMOKE_ONLY=quips: Sprüche v2 of guests and bosses, German and English.
    if (process.env.SMOKE_ONLY === 'quips') {
      await quipsPass(browser, errors, 'de-DE');
      await quipsPass(browser, errors, 'en-GB');
      await quipsPass(browser, errors, 'de-DE', DESKTOP);
      return report(errors);
    }
    // SMOKE_ONLY=legal: imprint, privacy policy and help beside the game.
    if (process.env.SMOKE_ONLY === 'legal') {
      await legalPass(browser, errors);
      return report(errors);
    }
    if (process.env.SMOKE_ONLY === 'night') {
      await nightPass(browser, errors, PHONE);
      await nightPass(browser, errors, DESKTOP);
      return report(errors);
    }
    const context = await quietContext(browser, { viewport: PHONE, reducedMotion });
    const page = await context.newPage();
    watch(page, errors);

    // Start screen and a new run with a fixed seed.
    await page.goto(base);
    await page.waitForSelector('[data-action="new-run"]');
    const build = await text(page, 'build');
    if (!build) fail('build info missing on the start screen');
    const sprite = await page.evaluate(() => ['f-CA', 'f-SJ', 'f-HQ', 'f-DK', 'm-points', 'bk-distel', 'bk-distel-s', 'f-blank', 'p-wirt'].filter((id) => !document.getElementById(id)));
    // The rare faces (Untot, Wandelkarte, Doppelgänger) are a second sprite file, fetched after boot
    // and put into the document only when a card needs them (ui/art.ts).
    const rareUrl = await page
      .waitForFunction(() => performance.getEntriesByType('resource').find((e) => /\/sprite-rare-[^/]*\.svg$/.test(e.name))?.name, null, { timeout: 10000 })
      .then((h) => h.jsonValue())
      .catch(() => fail('the rare card sprite was not fetched after boot'));
    if (await page.evaluate(() => document.querySelector('svg.sprite-rare'))) fail('the rare card sprite is in the document before a card needs it');
    const rareMissing = await page.evaluate(async (url) => {
      const doc = new DOMParser().parseFromString(await (await fetch(url)).text(), 'image/svg+xml');
      return ['u-S7', 'u-CA', 'd-HQ', 'w-A', 'wx-T', 'wt-K', 'ws-SA'].filter((id) => !doc.getElementById(id));
    }, rareUrl);
    if (rareMissing.length) fail(`rare card sprite incomplete, missing: ${rareMissing.join(', ')}`);
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
    const portraits = await page.evaluate((faces) => ['p-char-Declarer', 'p-char-NullPlayer', 'p-char-Shirker', 'p-char-Tailoress', 'p-char-HeartBrother', 'p-char-Gambler', 'p-char-RamschAunt', 'p-dealer', ...faces].filter((id) => !document.getElementById(id)), BOSS_FACES);
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

    await fillSeed(page, SEED);
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
        await assertTableLook(page, 'playing');
        // Before the first trick is finished there is nothing to look back at.
        const tricks = ((await page.locator('[data-tricks]').textContent()) ?? '').trim();
        if (tricks === 'Stiche 0 : 0' && (await page.locator('[data-testid="last-trick-toggle"]').count())) fail('a "Letzter Stich" button before any trick was finished');
      }
      if (played === 2) await assertLastTrick(page, 'playing');
      if (played === 3 && (await page.locator('[data-testid="last-trick"]').count())) fail('the last trick still shows after playing a card');
      if (played === 3 && dimmed > 0) {
        await shoot(page, 'table-follow');
        await assertTableLook(page, 'following suit');
      }
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
    await a11yPass(browser, errors, reducedMotion);
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
    await curseScenePass(browser, errors);
    await relicSlotsPass(browser, errors, reducedMotion, PHONE);
    await relicSlotsPass(browser, errors, reducedMotion, DESKTOP);
    await rescuePass(browser, errors, reducedMotion, PHONE);
    await rescuePass(browser, errors, reducedMotion, DESKTOP);
    await schnapsPass(browser, errors);
    await workshopPass(browser, errors, reducedMotion);
    await cardRewardPass(browser, errors, reducedMotion);
    await scoreContinuePass(browser, errors);
    await scoreTapPass(browser, errors);
    await encountersV2Pass(browser, errors, reducedMotion);
    await voucherPass(browser, errors);
    await specialPass(browser, errors, reducedMotion);
    await undeadPass(browser, errors, reducedMotion);
    await undeadGlowPass(browser, errors);
    await zinkenV2Pass(browser, errors, PHONE);
    await zinkenV2Pass(browser, errors, DESKTOP);
    await shortEvening(browser, errors, reducedMotion);
    await charactersPass(browser, errors);
    await newCharactersPass(browser, errors);
    await startScreenPass(browser, errors);
    await tresenPass(browser, errors);
    await paperPass(browser, errors);
    await wave2b(browser, errors, reducedMotion);
    await saveCode(browser, errors, reducedMotion, PHONE, true);
    await saveCode(browser, errors, reducedMotion, DESKTOP, false);
    await hintsAndRules(browser, errors, reducedMotion);
    await motionPass(browser, errors);
    await tableFeelPass(browser, errors, PHONE);
    await tableFeelPass(browser, errors, DESKTOP);
    await skatStaysPass(browser, errors);
    await rewardFxPass(browser, errors, PHONE);
    await rewardFxPass(browser, errors, DESKTOP);
    await nightPass(browser, errors, PHONE);
    await nightPass(browser, errors, DESKTOP);
    await soundsAll(browser, errors);
    await weeklyPass(browser, errors, PHONE);
    await weeklyPass(browser, errors, DESKTOP);
    await scenesPass(browser, errors, PHONE);
    await scenesPass(browser, errors, DESKTOP);
    await tutorialPass(browser, errors, PHONE);
    await tutorialPass(browser, errors, DESKTOP);
    await qolPass(browser, errors, PHONE);
    await qolPass(browser, errors, DESKTOP);
    await handSortPass(browser, errors, PHONE);
    await handSortPass(browser, errors, DESKTOP);
    await shareRunPass(browser, errors, PHONE);
    await shareRunPass(browser, errors, DESKTOP);
    await movePass(browser, errors, PHONE);
    await movePass(browser, errors, DESKTOP);
    await reportPass(browser, errors, PHONE);
    await offlinePass(browser, errors);
    await englishPass(browser, errors);
    await quipsPass(browser, errors, 'de-DE');
    await quipsPass(browser, errors, 'en-GB');
    await legalPass(browser, errors);
  } finally {
    await browser.close();
    preview.kill();
  }
  report(errors);
}

/** Fail on console errors of any pass (also with SMOKE_ONLY), else say OK. */
function report(errors) {
  if (errors.length) fail(`console errors:\n${errors.join('\n')}`);
  log(`OK. Screenshots in ${screens}`);
}

run().catch((err) => {
  console.error(`[smoke] FAILED: ${err.message}`);
  process.exit(1);
});

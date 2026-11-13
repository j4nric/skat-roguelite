// App Store preview video of "Noch 'ne Runde Skat" (886 × 1920, 30 fps,
// H.264 High, AAC stereo 44.1 kHz, about 28 s), German and English.
//
//   cd web && npm run build                      # the app as it ships
//   node ../store/make-preview.mjs [de|en] [--skip-record] [--only=clip,clip] [--music=jazz-a,jazz-b] [--no-assemble]
//
// 1. Records the clips: real app scenes from the saved runs of
//    web/e2e/fixtures, frame by frame in virtual time (./vclock.mjs) at
//    443 × 960 CSS px and device scale 2, with the sound effects the app
//    starts noted (name and time). Output: store/build/<lang>/<clip>/.
// 2. Renders the lines (pixel font) and the end card to PNG frames.
// 3. store/make-music.py writes the soundtrack of each variant (--music, the same for both
//    languages) and store/assemble.py cuts the clips to the beat, lays the
//    lines over them, mixes the game sounds onto the music and encodes.
// Output: store/preview/preview-<lang>-<variant>.mp4 and poster-<lang>-<variant>.png (not in git).

import { execFileSync } from 'node:child_process';
import { existsSync, mkdirSync, readFileSync } from 'node:fs';
import { join } from 'node:path';
import { chromium, appContext, resume, fixture, patched, park, watch, startPreview, storeDir, webDir, FIXED_TIME } from './lib.mjs';
import { recorder, spySounds, seedRandom } from './vclock.mjs';
import { VIDEO } from './lines.mjs';

const VIEW = { width: 443, height: 960 };
const args = process.argv.slice(2);
const langs = args.filter((a) => a === 'de' || a === 'en');
const skipRecord = args.includes('--skip-record');
const only = (args.find((a) => a.startsWith('--only=')) ?? '').slice(7).split(',').filter(Boolean);
const variants = (args.find((a) => a.startsWith('--music=')) ?? '--music=jazz-a').slice(8).split(',').filter(Boolean);
const buildDir = join(storeDir, 'build');
const outDir = join(storeDir, 'preview');

/** Page of the app for a clip: full motion, sound on (to note the sounds), music off. */
async function clipPage(browser, lang, errors, name) {
  const context = await appContext(browser, { lang, viewport: VIEW, scale: 2, motion: 'no-preference', settings: { sound: true, music: false } });
  await spySounds(context);
  await seedRandom(context);
  const page = await context.newPage();
  watch(page, errors, `${lang}/${name}`);
  return page;
}

/** Open the app with a saved run, up to the "Fortsetzen" button (not pressed). */
async function toResume(page, base, state) {
  await page.goto(base);
  await page.waitForSelector('[data-action="new-run"], [data-action="resume"]');
  await page.evaluate((s) => localStorage.setItem('stammtisch.run', s), state);
  await page.reload();
  await page.waitForSelector('[data-action="resume"]');
}

const isIdle = () => !document.querySelector('.run.is-busy');
const canPlay = () => !document.querySelector('.run.is-busy') && !!document.querySelector('.hand-card[data-action="play"]');

/** factor-win.json with the left Stammgast as forehand (the player plays last in the first trick). */
function swapForehand(name) {
  return patched(name, (save) => {
    const game = save.run.game.game;
    game.forehand = 1;
    game.trick_lead = 1;
    for (const ev of game.log) if (ev.Dealt) ev.Dealt.forehand = 1;
  });
}

/** The clips. Each gets a fresh page and records into `dir`. */
const CLIPS = {
  /** The start screen with the sign "Noch 'ne Runde Skat?". */
  async sign(page, base, dir) {
    await page.clock.setFixedTime(FIXED_TIME);
    await page.goto(base);
    await page.waitForSelector('[data-action="new-run"]');
    await page.waitForTimeout(800);
    const rec = await recorder(page, dir);
    await rec.start();
    await rec.hold(3);
    return rec.finish();
  },

  /** A fresh run (seed 64), Grand after the skat: the player leads, both Stammgäste follow, the trick is taken. */
  async trick(page, base, dir) {
    await page.goto(base);
    await page.waitForSelector('[data-action="new-run"]');
    const toggle = page.locator('[data-testid="start-more-toggle"]');
    if ((await toggle.getAttribute('aria-expanded')) !== 'true') await toggle.click();
    await page.fill('#seed', '64');
    await page.click('[data-action="new-run"]');
    await page.waitForSelector('[data-testid="skat-choice"]');
    await page.click('[data-action="pick-up-skat"]');
    await page.waitForSelector('[data-testid="skat-zone"] [data-action="select"]');
    for (let i = 0; i < 2; i++) await page.locator('.hand [data-action="select"]').first().click();
    await page.click('[data-action="discard"]');
    await page.waitForSelector('.declare[data-phase="DeclaringAfterSkat"]');
    await page.click('[data-action="pick-game"][data-index="4"]');
    await page.click('[data-action="declare"]');
    await page.waitForSelector('.declare', { state: 'detached' });
    await page.waitForFunction(canPlay, null, { timeout: 30000 });
    // Play the first trick as it comes (the Grün-Unter leads in the clip's second trick).
    await page.locator('.hand-card[data-action="play"]').first().click();
    await park(page);
    await page.waitForFunction(canPlay, null, { timeout: 30000 });
    await page.waitForTimeout(600);
    const rec = await recorder(page, dir);
    await rec.start();
    await rec.hold(0.3);
    rec.mark('play');
    await rec.click('.hand-card[data-action="play"]');
    await rec.until(canPlay, null, 15);
    rec.mark('next');
    await rec.hold(0.6);
    return rec.finish();
  },

  /**
   * factor-win.json from "Fortsetzen": Tante Erna's hello in her bubble,
   * Schellen Hand declared, ten tricks, the score slip with the stamp 342.
   */
  async factor(page, base, dir) {
    await toResume(page, base, fixture('factor-win.json'));
    const rec = await recorder(page, dir);
    await rec.start();
    await rec.hold(0.2);
    await rec.click('[data-action="resume"]');
    await rec.untilSelector('[data-testid="quip"]');
    rec.mark('quip');
    await rec.hold(2.4);
    rec.mark('hand');
    await rec.click('[data-action="play-hand"]');
    await rec.hold(0.45);
    const tiles = await page.locator('[data-action="pick-game"]').evaluateAll((els) => els.map((e) => e.querySelector('.game-base')?.textContent?.trim()));
    rec.mark('pick');
    await rec.click('[data-action="pick-game"]', { index: tiles.indexOf('9') });
    await rec.hold(0.45);
    rec.mark('declare');
    await rec.click('[data-action="declare"]');
    for (let played = 0; played < 12; played++) {
      await rec.until(() => !!document.querySelector('.score-sheet') || (!document.querySelector('.run.is-busy') && !!document.querySelector('.hand-card[data-action="play"]')), null, 20);
      if (await page.locator('.score-sheet').count()) break;
      await rec.click('.hand-card[data-action="play"]');
    }
    rec.mark('score');
    await rec.untilSelector('[data-num="value"].is-landed', 30);
    rec.mark('stamp');
    await rec.hold(2.2);
    return rec.finish();
  },

  /** undead.json, Eichel Hand: the Eichel-Daus is played, three cards picked, it crumbles and rises as a zombie. */
  async undead(page, base, dir) {
    await resume(page, base, fixture('undead.json'), { fixTime: false });
    await page.waitForSelector('.declare');
    await page.click('[data-action="play-hand"]');
    await page.waitForSelector('.declare-hand');
    await page.click('[data-action="pick-game"][data-index="0"]');
    if (await page.locator('[data-action="pick-risk"][data-risk="Hand"]').count()) await page.click('[data-action="pick-risk"][data-risk="Hand"]');
    await page.click('[data-action="declare"]');
    await page.waitForSelector('.declare', { state: 'detached' });
    await page.waitForFunction(canPlay, null, { timeout: 30000 });
    await park(page);
    await page.click('[data-action="play"][data-id="31"]');
    await park(page);
    await page.waitForSelector('[data-testid="rise-choice"]', { timeout: 30000 });
    const cards = page.locator('[data-testid="rise-choice"] .rise-card');
    for (let i = 0; i < 3; i++) await cards.nth(i).click();
    await page.waitForTimeout(400);
    const rec = await recorder(page, dir);
    await rec.start();
    await rec.hold(0.2);
    rec.mark('go');
    await rec.click('[data-testid="rise-go"]');
    await rec.untilSelector('[data-testid="rise-morph"]');
    rec.mark('morph');
    await rec.hold(3);
    return rec.finish();
  },

  /** undead.json with Doppelgänger cards (the smoke test's Zinken v2 save): played last, it mirrors the highest card. */
  async doppel(page, base, dir) {
    const save = patched('undead.json', (s) => {
      const card = (id) => s.run.edition.cards.find((c) => c.id === id);
      card(22).mods = ['doppelganger'];
      card(22).mod_values = {};
      for (const id of [2, 15, 16, 19, 23, 27, 28, 31]) card(id).mods = ['doppelganger', ...card(id).mods];
      const game = s.run.game.game;
      game.forehand = 1;
      game.trick_lead = 1;
      for (const ev of game.log) if (ev.Dealt) ev.Dealt.forehand = 1;
    });
    await resume(page, base, save, { fixTime: false });
    await page.waitForSelector('.declare');
    await page.click('[data-action="play-hand"]');
    await page.waitForSelector('.declare-hand');
    await page.click('[data-action="pick-game"][data-index="0"]');
    if (await page.locator('[data-action="pick-risk"][data-risk="Hand"]').count()) await page.click('[data-action="pick-risk"][data-risk="Hand"]');
    await page.click('[data-action="declare"]');
    await page.waitForSelector('.declare', { state: 'detached' });
    await page.waitForFunction(() => document.querySelectorAll('[data-trick-seat="1"], [data-trick-seat="2"]').length === 2 && !!document.querySelector('.hand-card[data-action="play"]:not([data-id="17"])') && !document.querySelector('.run.is-busy'), null, { timeout: 30000 });
    await park(page);
    await page.waitForTimeout(500);
    const rec = await recorder(page, dir);
    await rec.start();
    await rec.hold(0.3);
    rec.mark('play');
    await rec.click('.hand-card[data-action="play"]:not([data-id="17"])');
    await rec.untilSelector('[data-testid="doppel-shimmer"]', 10);
    rec.mark('shimmer');
    await rec.untilSelector('[data-testid="doppel-word"]', 10);
    rec.mark('word');
    await rec.hold(2.5);
    return rec.finish();
  },

  /** A Schnäppchen at the Tresen (zinken.json): the cursed card crumbles red. */
  async curse(page, base, dir) {
    const save = JSON.parse(fixture('zinken.json'));
    save.run.shop.relics = ['jinx', 'virgin', 'skat_redraw'];
    save.run.shop.fence = 0;
    save.run.shop.card_offers[1] = { card: 3, card_mod: 'points', value: 3, price: 1, bargain: 2 };
    save.run.coins = 20;
    await resume(page, base, JSON.stringify(save), { fixTime: false });
    await page.waitForSelector('[data-testid="zinken"]');
    await page.locator('[data-testid="zinken"]').scrollIntoViewIfNeeded();
    await park(page);
    await page.waitForTimeout(500);
    const rec = await recorder(page, dir);
    await rec.start();
    await rec.hold(0.2);
    rec.mark('buy');
    await rec.click('[data-action="buy-zinke"][data-slot="1"]');
    await rec.hold(0.2);
    if (await page.locator('[data-testid="mod-confirm-go"]').count()) await rec.click('[data-testid="mod-confirm-go"]');
    await rec.untilSelector('[data-testid="rise-morph"][data-kind="curse"]', 10);
    rec.mark('morph');
    await rec.until(() => !document.querySelector('[data-testid="rise-morph"]'), null, 10);
    rec.mark('done');
    await rec.hold(0.5);
    return rec.finish();
  },

  /** The Tresen (zinken.json): a Schnaps and a house rule bought, the Groschen go. */
  async tresen(page, base, dir) {
    await resume(page, base, fixture('zinken.json'), { fixTime: false });
    await page.waitForSelector('[data-testid="shop"]');
    await park(page);
    await page.waitForTimeout(600);
    const rec = await recorder(page, dir);
    await rec.start();
    await rec.hold(0.4);
    rec.mark('relic');
    await rec.click('[data-action="buy-relic"]:not([disabled])');
    await rec.hold(1.1);
    rec.mark('item');
    await rec.click('[data-action="buy-item"]:not([disabled])');
    await rec.hold(1.6);
    return rec.finish();
  },

  /** Leaving the Tresen before Die Zwillinge: the boss sheet with Fritz's line, then "Los geht's". */
  async boss(page, base, dir) {
    await resume(page, base, fixture('boss-tresen.json'), { fixTime: false });
    await page.waitForSelector('[data-testid="shop"]');
    await park(page);
    await page.waitForTimeout(500);
    const rec = await recorder(page, dir);
    await rec.start();
    await rec.hold(0.2);
    rec.mark('leave');
    await rec.click('[data-action="leave-shop"]');
    await rec.untilSelector('[data-action="boss-go"]');
    rec.mark('sheet');
    await rec.hold(3.2);
    rec.mark('go');
    await rec.click('[data-action="boss-go"]');
    await rec.hold(1.0);
    return rec.finish();
  },

  /** The city map (route-station.json): a way picked, "Los". */
  async map(page, base, dir) {
    await resume(page, base, fixture('route-station.json'), { fixTime: false });
    await page.waitForSelector('[data-testid="map-svg"]');
    await park(page);
    await page.waitForTimeout(800);
    const rec = await recorder(page, dir);
    await rec.start();
    await rec.hold(0.3);
    rec.mark('pick');
    await rec.click('[data-testid="route-option"]', { index: 1 });
    await rec.hold(0.7);
    rec.mark('go');
    await rec.click('[data-testid="route-go"]');
    await rec.hold(2.5);
    return rec.finish();
  },
};

/** Lines and end card as PNGs (pixel font), rendered in the browser. */
async function renderText(browser, lang) {
  const dir = join(buildDir, lang, 'text');
  mkdirSync(dir, { recursive: true });
  const font = `data:font/woff2;base64,${readFileSync(join(webDir, 'src', 'assets', 'fonts', 'vt323.woff2')).toString('base64')}`;
  const context = await browser.newContext({ viewport: { width: 886, height: 1920 }, deviceScaleFactor: 1 });
  const page = await context.newPage();
  const css = `@font-face { font-family: 'VT323'; src: url('${font}') format('woff2'); }
html, body { margin: 0; background: transparent; }
.tag { display: inline-block; font-family: 'VT323', monospace; color: #f1e6c8; text-align: center; line-height: 0.95;
  background: #2a2f34; border: 8px solid #0b0c0e; border-top: 14px solid #f1e6c8; padding: 18px 34px 24px;
  box-shadow: 14px 14px 0 rgba(4, 5, 7, 0.85); text-shadow: 6px 6px 0 #0b0c0e; white-space: nowrap; }
.wrap { position: absolute; left: 0; top: 0; padding: 0 22px 22px 0; }`;
  const lines = {};
  for (const [key, text] of Object.entries(VIDEO[lang].lines)) {
    const size = 116;
    const html = text.split('\n').map((l) => `<div>${l.replace(/&/g, '&amp;').replace(/</g, '&lt;')}</div>`).join('');
    await page.setContent(`<!doctype html><style>${css} .tag { font-size: ${size}px; }</style><div class="wrap"><div class="tag">${html}</div></div>`);
    await page.evaluate(() => document.fonts.ready);
    // Never wider than 820 px of the 886.
    await page.evaluate(() => {
      const tag = document.querySelector('.tag');
      const w = tag.getBoundingClientRect().width;
      if (w > 820) tag.style.fontSize = `${Math.floor((parseFloat(getComputedStyle(tag).fontSize) * 760) / w)}px`;
    });
    const file = join(dir, `line-${key}.png`);
    await page.locator('.wrap').screenshot({ path: file, omitBackground: true });
    lines[key] = file;
  }
  await context.close();
  return lines;
}

/** The end card as an animated page, recorded like a clip. */
async function recordEndCard(browser, lang, errors) {
  const dir = join(buildDir, lang, 'end');
  const font = `data:font/woff2;base64,${readFileSync(join(webDir, 'src', 'assets', 'fonts', 'vt323.woff2')).toString('base64')}`;
  const icon = `data:image/png;base64,${readFileSync(join(webDir, 'public', 'icon-512.png')).toString('base64')}`;
  const t = VIDEO[lang].end;
  const context = await browser.newContext({ viewport: VIEW, deviceScaleFactor: 2 });
  const page = await context.newPage();
  watch(page, errors, `${lang}/end`);
  await page.setContent(`<!doctype html><html><head><style>
@font-face { font-family: 'VT323'; src: url('${font}') format('woff2'); }
html, body { margin: 0; height: 100%; overflow: hidden; }
body { background-color: #1b1e21; background-image: radial-gradient(#272c30 1.5px, transparent 1.5px); background-size: 8px 8px;
  font-family: 'VT323', monospace; display: flex; flex-direction: column; align-items: center; justify-content: center; gap: 30px; }
body::after { content: ''; position: fixed; inset: 0; pointer-events: none; background: radial-gradient(ellipse 90% 75% at 50% 45%, transparent 55%, rgba(4, 5, 7, 0.6) 100%); }
.icon { width: 200px; height: 200px; image-rendering: pixelated; border: 4px solid #0b0c0e; box-shadow: 8px 8px 0 rgba(4,5,7,.8);
  animation: pop 520ms steps(6, end) 80ms both; }
.sign { display: flex; flex-direction: column; align-items: center; gap: 6px; padding: 18px 26px 20px; width: 392px; box-sizing: border-box;
  border: 4px solid #0b0c0e; border-top: 8px solid #f1e6c8; background: #2a2f34; box-shadow: 6px 6px 0 rgba(4,5,7,.85);
  animation: drop 560ms cubic-bezier(.2,1.4,.4,1) 360ms both; }
.small { font-size: 22px; letter-spacing: 3px; color: #a4adb3; text-transform: uppercase; }
h1 { margin: 0; font-weight: 400; font-size: 90px; line-height: .95; color: #f1e6c8; text-shadow: 4px 4px 0 #0b0c0e; text-align: center; }
.tagline { font-size: 22px; letter-spacing: 3px; color: #8fc7a8; text-transform: uppercase; }
/* The badge under the sign: dark ink on ivory, a 2 px dark outline with notched
   (pixel) corners, no offset shadow, so it stays crisp when small. */
.store { white-space: nowrap; animation: blink 900ms steps(1, end) 1100ms both; }
.store b { display: inline-block; font-weight: 400; font-size: 34px; line-height: 1; color: #1b1e21; background: #f1e6c8;
  padding: 7px 18px 9px; margin-top: 4px;
  box-shadow: 0 -2px 0 0 #0b0c0e, 0 2px 0 0 #0b0c0e, -2px 0 0 0 #0b0c0e, 2px 0 0 0 #0b0c0e; }
@keyframes pop { 0% { transform: scale(0); } 60% { transform: scale(1.15); } 100% { transform: scale(1); } }
@keyframes drop { 0% { transform: translateY(-140px); opacity: 0; } 30% { opacity: 1; } 100% { transform: none; opacity: 1; } }
@keyframes blink { 0% { opacity: 0; } 20% { opacity: 1; } 35% { opacity: 0; } 50% { opacity: 1; } 100% { opacity: 1; } }
</style></head><body>
<img class="icon" src="${icon}" alt="">
<div class="sign"><div class="small">${t.kicker}</div><h1>${t.title.replace(/\n/g, '<br>')}</h1><div class="tagline">${t.tagline}</div></div>
<div class="store"><b>${t.store}</b></div>
</body></html>`);
  await page.evaluate(() => document.fonts.ready);
  // Hold everything at its first frame until the recording starts.
  await page.evaluate(() => document.getAnimations().forEach((a) => a.pause()));
  const rec = await recorder(page, dir);
  await page.evaluate(() => document.getAnimations().forEach((a) => (a.currentTime = 0)));
  await rec.start();
  await rec.hold(5);
  const r = await rec.finish();
  await context.close();
  return r;
}

const own = process.env.STORE_BASE || skipRecord ? null : await startPreview(Number(process.env.STORE_PORT ?? 4194));
const base = process.env.STORE_BASE ?? own?.base;
const errors = [];
try {
  if (!skipRecord) {
    const browser = await chromium.launch();
    try {
      for (const lang of langs.length ? langs : ['de', 'en']) {
        for (const [name, clip] of Object.entries(CLIPS)) {
          if (only.length && !only.includes(name)) continue;
          const page = await clipPage(browser, lang, errors, name);
          const t0 = Date.now();
          const r = await clip(page, base, join(buildDir, lang, name));
          console.log(`[preview] ${lang}/${name}: ${r.frames.length} frames, ${r.sounds.length} sounds (${((Date.now() - t0) / 1000).toFixed(0)} s)`);
          await page.context().close();
        }
        if (!only.length || only.includes('end')) await recordEndCard(browser, lang, errors);
        await renderText(browser, lang);
      }
    } finally {
      await browser.close();
    }
  }
} finally {
  own?.child.kill();
}
if (errors.length) {
  console.error(`console errors:\n${errors.join('\n')}`);
  process.exit(1);
}

// Soundtrack and assembly, for each soundtrack variant asked for.
mkdirSync(outDir, { recursive: true });
if (!args.includes('--no-assemble')) {
  for (const variant of variants) {
    const music = join(buildDir, `music-${variant}.wav`);
    if (!existsSync(music) || process.env.STORE_MUSIC) execFileSync('python3', [join(storeDir, 'make-music.py'), variant, music], { stdio: 'inherit' });
    for (const lang of langs.length ? langs : ['de', 'en']) {
      execFileSync('python3', [join(storeDir, 'assemble.py'), lang, variant], { stdio: 'inherit' });
    }
  }
}

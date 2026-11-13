// App Store screenshots of "Noch 'ne Runde Skat" (iPhone 6.9", 1320 × 2868,
// PNG without alpha), German and English.
//
//   cd web && npm run build          # the app as it ships (web/dist)
//   node ../store/make-screenshots.mjs [de|en] [scene number]
//
// Each picture is a line of text in the game's pixel font (VT323) on the
// game's dark background, above a real picture of the app on a phone
// (430 × 780 CSS px, device scale 3). The scenes come from the saved runs
// of web/e2e/fixtures (the smoke test's), with sound, hints and the
// Einführung off and the clock fixed, so the output is the same each time.
// Output: store/screenshots/<lang>/<nn>-<name>.png.

import { execFileSync } from 'node:child_process';
import { mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import { join } from 'node:path';
import { chromium, appContext, resume, fixture, patched, idle, park, declareHand, watch, startPreview, storeDir, webDir } from './lib.mjs';
import { LINES } from './lines.mjs';

const W = 1320;
const H = 2868;
/** The app's viewport in CSS px; captured at scale 3. */
const APP = { width: 430, height: 780 };
const APP_SCALE = 3;

const [onlyLang, onlyScene] = process.argv.slice(2);
const langs = onlyLang ? [onlyLang] : ['de', 'en'];

/** Wait for the pointer's info card (if any) to go, so a still shows no hover. */
async function settle(page, ms = 400) {
  await park(page);
  await page.waitForSelector('[data-testid="info-card"]', { state: 'detached', timeout: 3000 }).catch(() => {});
  await page.waitForTimeout(ms);
}

/**
 * Pause every animation of the page as soon as the element with
 * data-testid `testid` reaches data-stage `stage` (sets window.__frozen).
 */
export async function freezeAt(page, testid, stage) {
  await page.evaluate(([id, st]) => {
    window.__frozen = false;
    const obs = new MutationObserver(() => {
      if (document.querySelector(`[data-testid="${id}"][data-stage="${st}"]`)) {
        for (const a of document.getAnimations()) a.pause();
        window.__frozen = true;
        obs.disconnect();
      }
    });
    obs.observe(document.body, { subtree: true, childList: true, attributes: true, attributeFilter: ['data-stage'] });
  }, [testid, stage]);
}

/** factor-win.json with the left Stammgast as forehand: the player plays last in the first trick. */
function factorLast() {
  return patched('factor-win.json', (save) => {
    const game = save.run.game.game;
    game.forehand = 1;
    game.trick_lead = 1;
    for (const ev of game.log) if (ev.Dealt) ev.Dealt.forehand = 1;
  });
}

/** Declare Schellen Hand on factor-win.json (the Schellen tile has the base value 9). */
async function declareSchellenHand(page) {
  await page.waitForSelector('[data-action="play-hand"]');
  await page.click('[data-action="play-hand"]');
  await page.locator('[data-action="pick-game"]', { has: page.locator('.game-base', { hasText: /^9$/ }) }).click();
  await page.click('[data-action="declare"]');
  await page.waitForSelector('.declare', { state: 'detached' });
}

/**
 * The six scenes, in the order of the store (the first three show in the
 * search). Each brings a fresh page to its moment.
 */
const SCENES = [
  {
    name: 'table',
    // Mid trick: both Stammgäste have played, "Du bist dran".
    async go(page, base) {
      await resume(page, base, factorLast());
      await declareSchellenHand(page);
      await page.waitForFunction(() => document.querySelectorAll('[data-trick-seat="1"], [data-trick-seat="2"]').length === 2 && document.querySelector('.hand-card[data-action="play"]'), null, { timeout: 30000 });
      await idle(page);
      await settle(page, 600);
    },
  },
  {
    name: 'score',
    // The score slip of the Faktor game: Schellen Hand, three relics, Spielwert 342.
    async go(page, base) {
      await resume(page, base, fixture('factor-win.json'));
      await declareSchellenHand(page);
      for (let played = 0; ; played++) {
        await page.waitForSelector('[data-action="play"], .score-sheet', { timeout: 30000 });
        if (await page.locator('.score-sheet').count()) break;
        await page.locator('[data-action="play"]').first().click();
        if (played > 12) throw new Error('score: more than ten cards');
      }
      if (await page.locator('[data-action="score-skip"].btn').count()) await page.locator('[data-action="score-skip"].btn').click();
      await page.waitForSelector('[data-action="score-continue"]');
      await settle(page, 600);
    },
  },
  {
    name: 'special',
    // undead.json: the Eichel-Daus (Untot) has just risen and lies in the
    // trick as a zombie; the hand holds the second Untot card, the
    // Wandelkarte and an Aufdruck +11.
    motion: true,
    tempo: 'slow',
    async go(page, base) {
      await resume(page, base, fixture('undead.json'));
      await declareHand(page, 0);
      await page.waitForSelector('[data-action="play"][data-id="31"]', { timeout: 30000 });
      await idle(page);
      await page.click('[data-action="play"][data-id="31"]');
      await park(page);
      await page.waitForSelector('[data-testid="rise-choice"]', { timeout: 30000 });
      const cards = page.locator('[data-testid="rise-choice"] .rise-card');
      for (let i = 0; i < 3; i++) await cards.nth(i).click();
      // The scene is driven by Web Animations: hold them all the moment
      // the zombie card stands alone, so the still catches it.
      await freezeAt(page, 'rise-morph', 'after');
      await page.click('[data-testid="rise-go"]');
      await park(page);
      await page.waitForFunction(() => window.__frozen, null, { timeout: 30000 });
      await page.waitForTimeout(300);
    },
  },
  {
    name: 'tresen',
    async go(page, base) {
      await resume(page, base, fixture('zinken.json'));
      await page.waitForSelector('[data-testid="shop"]');
      // The hint "Weitere Waren weiter unten" points at scrolling, which a still cannot do.
      await page.addStyleTag({ content: '.more-hint { display: none !important; }' });
      await settle(page, 800);
    },
  },
  {
    name: 'boss',
    // Die Zwillinge: the boss rule on the felt, Franz says hello.
    async go(page, base) {
      await resume(page, base, fixture('boss-tresen.json'));
      await page.waitForSelector('[data-testid="shop"]');
      await page.click('[data-action="leave-shop"]');
      await page.waitForSelector('[data-action="boss-go"]');
      await page.click('[data-action="boss-go"]');
      await page.waitForSelector('.declare');
      if (await page.locator('.sheet-unlock').count()) await page.click('.sheet-unlock [data-action="close-overlay"]');
      await page.locator('.opp[data-seat="2"] [data-testid="quip"]').waitFor({ timeout: 8000 });
      await settle(page, 500);
    },
  },
  {
    name: 'map',
    async go(page, base) {
      await resume(page, base, fixture('route-station.json'));
      await page.waitForSelector('[data-testid="map-svg"]');
      await settle(page, 1200);
    },
  },
];

/** The font as a data URL, for the composing page. */
const fontUrl = `data:font/woff2;base64,${readFileSync(join(webDir, 'src', 'assets', 'fonts', 'vt323.woff2')).toString('base64')}`;

/** HTML of one store picture: the line above the app's picture. */
function composeHtml(line, shotPng, fontSize) {
  const img = `data:image/png;base64,${shotPng.toString('base64')}`;
  const appW = 1180;
  const appH = Math.round((appW * APP.height) / APP.width);
  const top = H - appH - 70;
  const lines = line.split('\n').map((l) => `<div>${l.replace(/&/g, '&amp;').replace(/</g, '&lt;')}</div>`).join('');
  return `<!doctype html><html><head><style>
@font-face { font-family: 'VT323'; src: url('${fontUrl}') format('woff2'); }
html, body { margin: 0; width: ${W}px; height: ${H}px; overflow: hidden; }
body {
  position: relative;
  background-color: #1b1e21;
  background-image: radial-gradient(#272c30 4px, transparent 4px);
  background-size: 24px 24px;
  font-family: 'VT323', monospace;
}
.head {
  position: absolute; left: 0; right: 0; top: 0; height: ${top}px;
  display: flex; flex-direction: column; align-items: center; justify-content: center;
}
.bar { width: 180px; height: 18px; background: #f1e6c8; box-shadow: 9px 9px 0 #0b0c0e; margin-bottom: 54px; }
.line {
  color: #f1e6c8; font-size: ${fontSize}px; line-height: 0.98; text-align: center;
  text-shadow: 9px 9px 0 #0b0c0e; white-space: nowrap;
}
.app {
  position: absolute; left: ${(W - appW) / 2}px; top: ${top}px; width: ${appW}px; height: ${appH}px;
  border: 9px solid #0b0c0e; box-sizing: content-box; margin-left: -9px; margin-top: -9px;
  box-shadow: 18px 18px 0 rgba(4, 5, 7, 0.75);
  background: url('${img}') center / cover no-repeat;
  image-rendering: auto;
}
</style></head><body><div class="head"><div class="bar"></div><div class="line">${lines}</div></div><div class="app"></div></body></html>`;
}

/** The largest font size (px) at which every line of `lang` fits 1200 px wide, at most 176. */
async function fitFontSize(page, lang) {
  await page.setContent(`<!doctype html><style>@font-face { font-family: 'VT323'; src: url('${fontUrl}') format('woff2'); } span { font-family: 'VT323'; font-size: 100px; white-space: nowrap; }</style><body></body>`);
  await page.evaluate(() => document.fonts.load("100px 'VT323'"));
  const widest = await page.evaluate((all) => {
    let max = 0;
    for (const line of all) {
      for (const l of line.split('\n')) {
        const s = document.createElement('span');
        s.textContent = l;
        document.body.appendChild(s);
        max = Math.max(max, s.getBoundingClientRect().width);
        s.remove();
      }
    }
    return max;
  }, LINES[lang].shots);
  return Math.min(176, Math.floor((1200 / widest) * 100));
}

/** Strip the alpha channel (App Store Connect wants none) with Pillow. */
function flatten(path) {
  execFileSync('python3', ['-c', 'import sys\nfrom PIL import Image\nfor p in sys.argv[1:]:\n    Image.open(p).convert("RGB").save(p, optimize=True)', path]);
}

const own = process.env.STORE_BASE ? null : await startPreview(Number(process.env.STORE_PORT ?? 4193));
const base = process.env.STORE_BASE ?? own.base;
const browser = await chromium.launch();
const errors = [];
try {
  const composer = await (await browser.newContext({ viewport: { width: W, height: H }, deviceScaleFactor: 1 })).newPage();
  for (const lang of langs) {
    const outDir = join(storeDir, 'screenshots', lang);
    mkdirSync(outDir, { recursive: true });
    const fontSize = await fitFontSize(composer, lang);
    for (const [i, scene] of SCENES.entries()) {
      if (onlyScene && Number(onlyScene) !== i + 1) continue;
      const context = await appContext(browser, { lang, viewport: APP, scale: APP_SCALE, motion: scene.motion ? 'no-preference' : 'reduce', tempo: scene.tempo ?? 'normal' });
      const page = await context.newPage();
      watch(page, errors, `${lang}/${scene.name}`);
      await scene.go(page, base);
      const shot = await page.screenshot();
      await context.close();
      await composer.setContent(composeHtml(LINES[lang].shots[i], shot, fontSize));
      await composer.evaluate(() => document.fonts.ready);
      const file = join(outDir, `${String(i + 1).padStart(2, '0')}-${scene.name}.png`);
      writeFileSync(file, await composer.screenshot({ type: 'png' }));
      flatten(file);
      console.log(`[store] ${file} (font ${fontSize}px)`);
    }
  }
} finally {
  await browser.close();
  own?.child.kill();
}
if (errors.length) {
  console.error(`console errors:\n${errors.join('\n')}`);
  process.exit(1);
}

// Short vertical clips of "Noch 'ne Runde Skat" for Reddit and the like
// (1080 × 1920, 9:16, 30 fps, H.264 High + AAC, 10–15 s), German and English.
//
//   cd web && npm run build                     # the app as it ships
//   node ../store/make-clips.mjs [de|en] [--only=342,untot,boss] [--skip-record]
//
// Each clip is one moment of the real app, recorded in virtual time like the
// preview video (./vclock.mjs, helpers from ./make-preview.mjs), at 432 × 664
// CSS px and device scale 2.5, so a frame is 1080 × 1660 device pixels and is
// never scaled. Above it a band of 260 px with the clip's hook in the pixel
// font, rendered at its size; at the end the end card (from make-preview.mjs)
// with the name and the address. The sound is the preview's soundtrack
// (store/make-music.py, jazz-a), placed so the clip's key moment lands on a
// hit of it, with the game sounds the app made at their recorded times;
// loudness -16 LUFS like the preview.
// Output: store/clips/<clip>-<lang>.mp4 and poster-<clip>-<lang>.png (not in git).

import { execFileSync, spawnSync } from 'node:child_process';
import { existsSync, mkdirSync, readFileSync, rmSync, symlinkSync } from 'node:fs';
import { join } from 'node:path';
import { chromium, resume, fixture, park, watch, startPreview, storeDir, webDir } from './lib.mjs';
import { FPS, recorder } from './vclock.mjs';
import { clipPage, canPlay, recordEndCard } from './make-preview.mjs';

const W = 1080;
const H = 1920;
/** The hook band above the app. */
const BAND = 260;
/** The app: 432 × 664 CSS px at scale 2.5 = 1080 × 1660, right under the band. */
const VIEW = { width: 432, height: 664 };
const SCALE = 2.5;
/** The end card: the whole frame. */
const END_VIEW = { width: 432, height: 768 };
const END_SECONDS = 2;

const args = process.argv.slice(2);
const langs = args.filter((a) => a === 'de' || a === 'en');
const only = (args.find((a) => a.startsWith('--only=')) ?? '').slice(7).split(',').filter(Boolean);
const skipRecord = args.includes('--skip-record');
const buildDir = join(storeDir, 'build', 'clips');
const outDir = join(storeDir, 'clips');
const soundsDir = join(webDir, 'src', 'assets', 'sounds');
const music = join(storeDir, 'build', 'music-jazz-a.wav');
/** Where the end card's big hit sits in jazz-a (store/grid.py: scene `end`, beat 60 at 160 BPM). */
const MUSIC_END = 22.5;

/** The end card's texts. */
const END = {
  de: { kicker: 'Gaststätte', title: "Noch 'ne\nRunde\nSkat", tagline: 'Skat als Roguelike', store: 'Gratis im Browser:\none-more-skat.de' },
  en: { kicker: 'Pub', title: "Noch 'ne\nRunde\nSkat", tagline: 'Skat as a roguelike', store: 'Free in your browser:\none-more-skat.de' },
};

/** Level of each game sound under the music (as in assemble.py). */
const SFX_GAIN = {
  card: 0.55, trick: 0.5, tap: 0.25, quip: 0.8, bang: 0.8, slam: 0.6, declare: 0.7, clink: 0.5, tick: 0.35, knock: 0.7, coin: 0.7,
  till: 0.6, boss: 0.9, chalk: 0.6, prize: 0.6, win: 0.5, shuffle: 0.35, unlock: 0.4, kontra: 0.5, pickup: 0.4, discard: 0.4,
  gulp: 0.5, pour: 0.5, dice: 0.5, snip: 0.5, schwarz: 0.5, lose: 0, night: 0,
};
const SFX_MASTER = 0.5;

/** Declare a Hand game with the game tile `index` (0 Eichel … 4 Grand), Hand level if offered. */
async function declareHandTile(page, index) {
  await page.waitForSelector('.declare');
  await page.click('[data-action="play-hand"]');
  await page.waitForSelector('.declare-hand');
  await page.click(`[data-action="pick-game"][data-index="${index}"]`);
  if (await page.locator('[data-action="pick-risk"][data-risk="Hand"]').count()) await page.click('[data-action="pick-risk"][data-risk="Hand"]');
  await page.click('[data-action="declare"]');
  await page.waitForSelector('.declare', { state: 'detached' });
}

/**
 * The clips. `record` brings a fresh page to its moment and records it;
 * `pieces` are the spans of the recording shown, in order ([from, to) as a
 * frame or 'mark+n' / 'mark-n'), `music` the mark that lands on `at` (s)
 * of the soundtrack, `poster` the frame of the poster picture.
 */
const CLIPS = {
  /** factor-win.json: Schellen Hand, the last tricks, the score slip counting up to 342 with three relics lit. */
  342: {
    hook: { de: 'Skat, aber mit 342 Punkten.', en: 'Skat, but worth 342 points.' },
    async record(page, base, dir) {
      await resume(page, base, fixture('factor-win.json'), { fixTime: false });
      await page.waitForSelector('[data-action="play-hand"]');
      await park(page);
      await page.waitForTimeout(600);
      const rec = await recorder(page, dir, { scale: SCALE });
      await rec.start();
      await rec.hold(0.3);
      rec.mark('hand');
      await rec.click('[data-action="play-hand"]');
      await rec.hold(0.5);
      const tiles = await page.locator('[data-action="pick-game"]').evaluateAll((els) => els.map((e) => e.querySelector('.game-base')?.textContent?.trim()));
      await rec.click('[data-action="pick-game"]', { index: tiles.indexOf('9') });
      await rec.hold(0.5);
      rec.mark('declare');
      await rec.click('[data-action="declare"]');
      for (let played = 0; played < 12; played++) {
        await rec.until(() => !!document.querySelector('.score-sheet') || (!document.querySelector('.run.is-busy') && !!document.querySelector('.hand-card[data-action="play"]')), null, 20);
        if (await page.locator('.score-sheet').count()) break;
        rec.mark(`play${played}`);
        await rec.click('.hand-card[data-action="play"]');
      }
      rec.mark('score');
      await rec.untilSelector('[data-num="value"].is-landed', 30);
      rec.mark('stamp');
      await rec.hold(3.2);
      return rec.finish();
    },
    pieces: [
      ['hand+0', 'declare+12'],
      ['play4+0', 'stamp+90'],
    ],
    music: ['stamp', 9.0],
    poster: 'stamp+20',
  },

  /** undead.json, Eichel Hand: the Eichel-Daus (Untot) is played, crumbles and rises again as a zombie. */
  untot: {
    hook: { de: 'Die Karte steht wieder auf.', en: 'The card gets back up.' },
    async record(page, base, dir) {
      await resume(page, base, fixture('undead.json'), { fixTime: false });
      await declareHandTile(page, 0);
      await page.waitForFunction(canPlay, null, { timeout: 30000 });
      await park(page);
      await page.waitForTimeout(500);
      const rec = await recorder(page, dir, { scale: SCALE });
      await rec.start();
      await rec.hold(1.2);
      rec.mark('play');
      await rec.click('[data-action="play"][data-id="31"]');
      await rec.untilSelector('[data-testid="rise-choice"]', 30);
      rec.mark('choice');
      await rec.hold(0.8);
      for (let i = 0; i < 3; i++) {
        await rec.click('[data-testid="rise-choice"] .rise-card', { index: i });
        await rec.hold(0.4);
      }
      await rec.hold(0.3);
      rec.mark('go');
      await rec.click('[data-testid="rise-go"]');
      await rec.untilSelector('[data-testid="rise-morph"]');
      rec.mark('morph');
      await rec.hold(5.2);
      return rec.finish();
    },
    pieces: [['play-20', 'morph+150']],
    music: ['morph', 10.5],
    poster: 'morph+45',
  },

  /** boss-tresen.json: out of the Tresen to Die Zwillinge, the boss sheet, then the table with the boss rule and Franz's hello. */
  boss: {
    hook: { de: 'Am Stammtisch sitzt der Boss.', en: 'The boss sits at the table.' },
    async record(page, base, dir) {
      await resume(page, base, fixture('boss-tresen.json'), { fixTime: false });
      await page.waitForSelector('[data-testid="shop"]');
      await park(page);
      await page.waitForTimeout(500);
      const rec = await recorder(page, dir, { scale: SCALE });
      await rec.start();
      await rec.hold(1.0);
      rec.mark('leave');
      await rec.click('[data-action="leave-shop"]');
      await rec.untilSelector('[data-action="boss-go"]');
      rec.mark('sheet');
      await rec.hold(3.6);
      rec.mark('go');
      await rec.click('[data-action="boss-go"]');
      await rec.untilSelector('.declare', 20);
      rec.mark('table');
      if (await page.locator('.sheet-unlock').count()) await rec.click('.sheet-unlock [data-action="close-overlay"]');
      try {
        await rec.untilSelector('.opp[data-seat="2"] [data-testid="quip"]', 8);
        rec.mark('quip');
        // Seen 16.11.2026: the bubble comes while the table is still dimmed by the boss's entrance and is gone before it lights up.
        console.log(`[clips] boss: Franz's bubble at frame ${rec.count}`);
      } catch {
        console.warn('[clips] boss: no bubble of Franz (seat 2) within 8 s of the table');
      }
      await rec.hold(4.6);
      return rec.finish();
    },
    pieces: [[0, 'table+135']],
    music: ['sheet', 16.5],
    poster: 'sheet+60',
  },
};

const font = () => `data:font/woff2;base64,${readFileSync(join(webDir, 'src', 'assets', 'fonts', 'vt323.woff2')).toString('base64')}`;
const esc = (s) => s.replace(/&/g, '&amp;').replace(/</g, '&lt;');

/** The background of a clip (1080 × 1920): the band with the hook on the app's dot pattern, a dark line under it. */
async function renderBand(browser, hook, file) {
  const context = await browser.newContext({ viewport: { width: W, height: H }, deviceScaleFactor: 1 });
  const page = await context.newPage();
  await page.setContent(`<!doctype html><html><head><style>
@font-face { font-family: 'VT323'; src: url('${font()}') format('woff2'); }
html, body { margin: 0; width: ${W}px; height: ${H}px; overflow: hidden; }
body { position: relative; background-color: #1b1e21; background-image: radial-gradient(#272c30 3px, transparent 3px); background-size: 20px 20px; font-family: 'VT323', monospace; }
.band { position: absolute; left: 0; right: 0; top: 0; height: ${BAND - 8}px; display: flex; align-items: center; justify-content: center; }
.hook { color: #f1e6c8; font-size: 112px; line-height: 1; white-space: nowrap; text-shadow: 7px 7px 0 #0b0c0e; padding-bottom: 6px; }
.rule { position: absolute; left: 0; right: 0; top: ${BAND - 8}px; height: 8px; background: #0b0c0e; }
</style></head><body><div class="band"><div class="hook">${esc(hook)}</div></div><div class="rule"></div></body></html>`);
  await page.evaluate(() => document.fonts.ready);
  // One line, at most 1000 px wide: the font shrinks to fit.
  const size = await page.evaluate(() => {
    const el = document.querySelector('.hook');
    const w = el.getBoundingClientRect().width;
    const px = parseFloat(getComputedStyle(el).fontSize);
    if (w > 1000) el.style.fontSize = `${Math.floor((px * 1000) / w)}px`;
    return parseFloat(getComputedStyle(el).fontSize);
  });
  // A busy machine sometimes refuses a capture this large: try again.
  for (let i = 0; ; i++) {
    try {
      await page.screenshot({ path: file, type: 'png' });
      break;
    } catch (e) {
      if (i >= 3) throw e;
      await page.waitForTimeout(500);
    }
  }
  await context.close();
  return size;
}

function startFrame(clip, at) {
  if (typeof at === 'number') return at;
  const m = /^([a-z0-9]+)([+-]\d+)?$/i.exec(at);
  if (!m || !(m[1] in clip.marks)) throw new Error(`clips: no mark ${at}`);
  return clip.marks[m[1]] + Number(m[2] ?? 0);
}

/** The frames to show (source indices) and where each piece starts in the output. */
function cut(def, clip) {
  const frames = [];
  const pieces = [];
  for (const [a, b] of def.pieces) {
    const from = startFrame(clip, a);
    const to = Math.min(startFrame(clip, b), clip.frames.length);
    if (from < 0 || to <= from) throw new Error(`clips: bad piece ${a}..${b}`);
    pieces.push({ from, to, at: frames.length });
    for (let f = from; f < to; f++) frames.push(f);
  }
  return { frames, pieces };
}

/** The output frame of a source frame (the first piece that holds it), or -1. */
function outFrame(pieces, src) {
  for (const p of pieces) if (src >= p.from && src < p.to) return p.at + src - p.from;
  return -1;
}

function ffmpeg(argv, opts = {}) {
  return execFileSync('ffmpeg', ['-hide_banner', '-v', 'error', '-y', ...argv], { maxBuffer: 1 << 26, ...opts });
}

/** Video, sound and poster of one clip in one language. */
function assemble(name, def, lang) {
  const dir = join(buildDir, lang, name);
  const clip = JSON.parse(readFileSync(join(dir, 'rec', 'clip.json'), 'utf8'));
  const end = JSON.parse(readFileSync(join(buildDir, lang, 'end', 'clip.json'), 'utf8'));
  const { frames, pieces } = cut(def, clip);
  const endFrames = end.frames.slice(0, END_SECONDS * FPS);
  const content = frames.length / FPS;
  const total = content + endFrames.length / FPS;

  // The frames in order, as links (image2 reads a numbered sequence).
  const seq = join(dir, 'seq');
  rmSync(seq, { recursive: true, force: true });
  mkdirSync(seq, { recursive: true });
  frames.forEach((f, i) => symlinkSync(join(dir, 'rec', clip.frames[f]), join(seq, `c${String(i).padStart(5, '0')}.jpg`)));
  endFrames.forEach((f, i) => symlinkSync(join(buildDir, lang, 'end', f), join(seq, `e${String(i).padStart(5, '0')}.jpg`)));

  // Sound: the soundtrack from where the key moment lands on its hit, then its end-card hit
  // under the end card; the game sounds at their times.
  const [mark, at] = def.music;
  const keyOut = outFrame(pieces, startFrame(clip, mark)) / FPS;
  const offset = Math.max(0, Math.min(at - keyOut, MUSIC_END - content));
  const sfx = [];
  for (const s of clip.sounds) {
    const gain = SFX_GAIN[s.name] ?? 0.4;
    const o = outFrame(pieces, Math.round(s.t * FPS));
    if (gain > 0 && o >= 0) sfx.push({ name: s.name, t: o / FPS, gain: gain * SFX_MASTER });
  }
  const inputs = ['-i', music, '-i', music];
  const chains = [
    `[0:a]atrim=${offset.toFixed(4)}:${(offset + content).toFixed(4)},asetpts=PTS-STARTPTS,afade=t=out:st=${(content - 0.04).toFixed(4)}:d=0.04[ma]`,
    `[1:a]atrim=${MUSIC_END}:${(MUSIC_END + total - content).toFixed(4)},asetpts=PTS-STARTPTS,afade=t=out:st=${(total - content - 0.6).toFixed(4)}:d=0.6[mb]`,
    `[ma][mb]concat=n=2:v=0:a=1[mus]`,
  ];
  const mixIn = ['[mus]'];
  sfx.forEach((s, i) => {
    inputs.push('-i', join(soundsDir, `${s.name}.mp3`));
    const ms = Math.round(s.t * 1000);
    chains.push(`[${i + 2}:a]aresample=44100,aformat=channel_layouts=stereo,volume=${s.gain},adelay=${ms}|${ms}[s${i}]`);
    mixIn.push(`[s${i}]`);
  });
  chains.push(`${mixIn.join('')}amix=inputs=${mixIn.length}:normalize=0:duration=first,atrim=0:${total.toFixed(4)}[mix]`);
  const raw = join(dir, 'audio.wav');
  ffmpeg([...inputs, '-filter_complex', chains.join(';'), '-map', '[mix]', '-ar', '44100', '-ac', '2', raw]);
  // Two-pass loudness normalization to -16 LUFS, true peak -1.5 dBTP.
  const stats = loudness(raw);
  const audio = join(dir, 'audio-norm.wav');
  ffmpeg(['-i', raw, '-af', `loudnorm=I=-16:TP=-1.5:LRA=11:measured_I=${stats.input_i}:measured_TP=${stats.input_tp}:measured_LRA=${stats.input_lra}:measured_thresh=${stats.input_thresh}:offset=${stats.target_offset}:linear=true`, '-ar', '44100', audio]);

  // Picture: the band, the app under it, then the end card; H.264 High, yuv420p.
  mkdirSync(outDir, { recursive: true });
  const out = join(outDir, `${name}-${lang}.mp4`);
  ffmpeg([
    '-loop', '1', '-framerate', String(FPS), '-i', join(dir, 'band.png'),
    '-framerate', String(FPS), '-i', join(seq, 'c%05d.jpg'),
    '-framerate', String(FPS), '-i', join(seq, 'e%05d.jpg'),
    '-i', audio,
    '-filter_complex',
    `[0:v][1:v]overlay=0:${BAND}:shortest=1,format=yuv420p,setsar=1[main];[2:v]scale=${W}:${H},format=yuv420p,setsar=1[end];[main][end]concat=n=2:v=1:a=0[v]`,
    '-map', '[v]', '-map', '3:a',
    '-c:v', 'libx264', '-profile:v', 'high', '-level', '4.1', '-preset', 'slow', '-crf', '18', '-maxrate', '7M', '-bufsize', '14M', '-pix_fmt', 'yuv420p', '-r', String(FPS),
    '-c:a', 'aac', '-b:a', '192k', '-ar', '44100', '-ac', '2', '-shortest', '-movflags', '+faststart', out,
  ]);
  // Poster: that frame of the clip, as PNG.
  const poster = join(outDir, `poster-${name}-${lang}.png`);
  const pf = outFrame(pieces, startFrame(clip, def.poster));
  ffmpeg(['-i', out, '-vf', `select=eq(n\\,${pf})`, '-frames:v', '1', poster]);
  return { out, poster, content, total, offset, sounds: sfx.length };
}

/** loudnorm's measurement of `file` (first pass; it reports on stderr). */
function loudness(file) {
  const r = spawnSync('ffmpeg', ['-hide_banner', '-i', file, '-af', 'loudnorm=I=-16:TP=-1.5:LRA=11:print_format=json', '-f', 'null', '-'], { encoding: 'utf8' });
  const err = r.stderr ?? '';
  return JSON.parse(err.slice(err.lastIndexOf('{'), err.lastIndexOf('}') + 1));
}

const own = process.env.STORE_BASE || skipRecord ? null : await startPreview(Number(process.env.STORE_PORT ?? 4195));
const base = process.env.STORE_BASE ?? own?.base;
const errors = [];
const names = Object.keys(CLIPS).filter((n) => !only.length || only.includes(n));
try {
  if (!existsSync(music)) execFileSync('python3', [join(storeDir, 'make-music.py'), 'jazz-a', music], { stdio: 'inherit' });
  const browser = await chromium.launch();
  try {
    for (const lang of langs.length ? langs : ['de', 'en']) {
      if (!skipRecord) {
        await recordEndCard(browser, lang, errors, { texts: END[lang], view: END_VIEW, scale: SCALE, seconds: END_SECONDS, speed: 2, dir: join(buildDir, lang, 'end') });
      }
      for (const name of names) {
        const def = CLIPS[name];
        const dir = join(buildDir, lang, name);
        mkdirSync(dir, { recursive: true });
        const size = await renderBand(browser, def.hook[lang], join(dir, 'band.png'));
        if (!skipRecord) {
          const page = await clipPage(browser, lang, errors, name, { view: VIEW, scale: SCALE });
          const t0 = Date.now();
          const r = await def.record(page, base, join(dir, 'rec'));
          console.log(`[clips] ${lang}/${name}: ${r.frames.length} frames, ${r.sounds.length} sounds, marks ${JSON.stringify(r.marks)} (${((Date.now() - t0) / 1000).toFixed(0)} s)`);
          await page.context().close();
        }
        if (errors.length) throw new Error(`console errors:\n${errors.join('\n')}`);
        const a = assemble(name, def, lang);
        console.log(`[clips] ${a.out}: ${a.total.toFixed(2)} s (${a.content.toFixed(2)} s + end card), hook ${size}px, music from ${a.offset.toFixed(2)} s, ${a.sounds} game sounds`);
      }
    }
  } finally {
    await browser.close();
  }
} finally {
  own?.child.kill();
}

// Frame-stepped recording for the preview video ("virtual time").
//
// The DevTools screencast only delivers CSS-pixel frames (443 × 960), and a
// screenshot at device scale 2 takes longer than a frame. So the clip is
// recorded in virtual time: Playwright's fake clock drives timers,
// requestAnimationFrame, Date and performance.now; every Web Animation (the
// app's card flights, the crumble, CSS animations and transitions) is
// paused and set to the virtual time by hand. Each step advances both by
// one frame (1/30 s) and takes a screenshot at device pixels (886 × 1920).
// The core's worker answers on the real clock between frames, a frame or
// two at most. Math.random is seeded, so scattered bits fall the same way.

import { mkdirSync, rmSync, writeFileSync } from 'node:fs';
import { join } from 'node:path';

export const FPS = 30;
const STEP = 1000 / FPS;

/** Seed Math.random (mulberry32) before the app runs; for the context. */
export async function seedRandom(context, seed = 20261112) {
  await context.addInitScript((s) => {
    let a = s >>> 0;
    Math.random = () => {
      a = (a + 0x6d2b79f5) >>> 0;
      let t = a;
      t = Math.imul(t ^ (t >>> 15), t | 1);
      t ^= t + Math.imul(t ^ (t >>> 7), t | 61);
      return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
    };
  }, seed);
}

/** Note every sound effect the app starts (name, virtual ms) in window.__sounds; for the context. */
export async function spySounds(context) {
  await context.addInitScript(() => {
    window.__sounds = [];
    const names = new WeakMap();
    const fetchFn = window.fetch;
    window.fetch = async (...args) => {
      const res = await fetchFn(...args);
      const url = String(args[0] instanceof Request ? args[0].url : args[0]);
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
      if (name && window.__vt) window.__sounds.push({ name, t: window.__vt.now });
      return start.apply(this, args);
    };
  });
}

/**
 * A recorder on `page` writing frames to `dir`. Call `start()` once the
 * page is at the scene, then `frames(n)`, `until(fn)`, `click(sel)` ...
 * and `finish()`, which writes dir/clip.json ({ frames, sounds, marks }).
 */
export async function recorder(page, dir) {
  rmSync(dir, { recursive: true, force: true });
  mkdirSync(dir, { recursive: true });
  const cdp = await page.context().newCDPSession(page);
  const frames = [];
  const marks = {};
  let n = 0;

  async function shoot() {
    // A CDP session reports device scale 1: render the capture at twice the
    // CSS size, so each frame has the device pixels (886 × 1920), not an upscale.
    const v = page.viewportSize();
    const { data } = await cdp.send('Page.captureScreenshot', { format: 'jpeg', quality: 93, optimizeForSpeed: true, clip: { x: 0, y: 0, width: v.width, height: v.height, scale: 2 } });
    const file = `f${String(n).padStart(5, '0')}.jpg`;
    writeFileSync(join(dir, file), Buffer.from(data, 'base64'));
    frames.push(file);
    n += 1;
  }

  /** One frame: timers, then animations, then a picture. */
  async function frame() {
    await page.clock.runFor(STEP);
    await page.evaluate((dt) => window.__advance(dt), STEP);
    await shoot();
  }

  const rec = {
    async start() {
      await page.clock.install();
      await page.evaluate(() => {
        const vt = { now: 0, seen: new WeakMap(), first: true };
        window.__vt = vt;
        window.__advance = (dt) => {
          vt.now += dt;
          for (const a of document.getAnimations()) {
            if (!vt.seen.has(a)) {
              // Running before the recording: keep its phase. New ones began within the last frame.
              vt.seen.set(a, vt.first ? vt.now - (Number(a.currentTime) || 0) : vt.now - dt);
            }
            if (a.playState === 'finished' || a.playState === 'idle') continue;
            const t = vt.now - vt.seen.get(a);
            const end = a.effect?.getComputedTiming().endTime;
            if (typeof end === 'number' && Number.isFinite(end) && t >= end / Math.abs(a.playbackRate || 1)) {
              try {
                a.finish();
              } catch {
                a.cancel();
              }
            } else {
              if (a.playState !== 'paused') a.pause();
              a.currentTime = t * (a.playbackRate || 1);
            }
          }
          vt.first = false;
        };
        window.__advance(0);
      });
      await shoot();
    },
    /** `count` frames. */
    async frames(count) {
      for (let i = 0; i < count; i++) await frame();
    },
    /** Seconds as frames. */
    async hold(seconds) {
      await rec.frames(Math.round(seconds * FPS));
    },
    /** Step frames until `fn` (a page function) is true; at most `limit` seconds. */
    async until(fn, arg, limit = 20) {
      for (let i = 0; i < limit * FPS; i++) {
        if (await page.evaluate(fn, arg)) return;
        await frame();
      }
      throw new Error(`recorder: condition not met in ${limit}s: ${fn}`);
    },
    async untilSelector(selector, limit = 20) {
      await rec.until((s) => !!document.querySelector(s), selector, limit);
    },
    /** Note this moment (frame index). */
    mark(name) {
      marks[name] = n;
    },
    /** Click without waiting on Playwright's stability checks (the animations are held). */
    async click(selector, { index = 0 } = {}) {
      const box = await page.locator(selector).nth(index).boundingBox();
      if (!box) throw new Error(`recorder: nothing to click at ${selector}`);
      await page.mouse.click(box.x + box.width / 2, box.y + box.height / 2);
      const v = page.viewportSize();
      await page.mouse.move(v.width - 2, 2);
    },
    async finish() {
      const sounds = (await page.evaluate(() => window.__sounds ?? [])).map((s) => ({ name: s.name, t: s.t / 1000 }));
      writeFileSync(join(dir, 'clip.json'), JSON.stringify({ fps: FPS, frames, sounds, marks }, null, 1));
      await cdp.detach().catch(() => {});
      return { frames, sounds, marks };
    },
    get count() {
      return n;
    },
  };
  return rec;
}

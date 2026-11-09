// Pixel art from the vector drawings: the portraits and the goods are drawn
// as SVG (./people.ts, ./goods.ts) and rendered once at start into a tiny
// canvas (a few dozen pixels per side). Shown scaled up with
// `image-rendering: pixelated`, they match the pixel look of the deck. The
// colours are pulled a little toward the slate palette for the dark mood.
// If anything fails, the vector drawing is shown instead. Pure drawing.
//
// The images are kept in localStorage (W15 of the review): drawing them
// costs the main thread at every start, and they only change with the
// drawings or this code. Each is stored with a hash of its drawing, the
// build and PIXEL_VERSION (raise it when `grade` or `render` change); the
// key has no dot, so it is no part of the save (./storage.ts).

import { hashText } from '../hash';
import { readJson, writeKey } from '../storage';

const cache = new Map<string, string>();

const STORE_KEY = 'stammtisch-pixels';
/** Raise when the way a drawing becomes pixels changes. */
const PIXEL_VERSION = 1;

/** Stored images: key → [hash of the job, data URL]. */
type Stored = Record<string, [string, string]>;

let stored: Stored | null = null;

function storedImages(): Stored {
  if (stored) return stored;
  const raw = readJson(STORE_KEY);
  stored = raw && typeof raw === 'object' && !Array.isArray(raw) ? (raw as Stored) : {};
  return stored;
}

function jobHash(job: PixelJob): string {
  const build = typeof __BUILD_INFO__ === 'string' ? __BUILD_INFO__ : '';
  return hashText(`${PIXEL_VERSION}|${build}|${job.size}|${job.markup}`);
}

/** Pixel image (data URL) for a key, once prepared. */
export function pixelUrl(key: string): string | null {
  return cache.get(key) ?? null;
}

export interface PixelJob {
  key: string;
  /** Inner SVG markup in a 100 × 100 box. */
  markup: string;
  /** Pixels per side. */
  size: number;
}

/** The suit symbols, so drawings that reference them render standalone. */
function sharedDefs(): string {
  const ids = ['s-C', 's-S', 's-H', 's-D'];
  return ids
    .map((id) => document.getElementById(id)?.outerHTML ?? '')
    .join('');
}

/** An image, loaded; rejects if it does not load. */
export function loadImage(src: string): Promise<HTMLImageElement> {
  return new Promise((resolve, reject) => {
    const img = new Image();
    img.onload = () => resolve(img);
    img.onerror = () => reject(new Error('image failed'));
    img.src = src;
  });
}

/** Shades toward slate: a touch less saturation, a cool shadow. */
function grade(data: Uint8ClampedArray): void {
  for (let i = 0; i < data.length; i += 4) {
    const a = data[i + 3] ?? 0;
    // Hard alpha: a pixel is there or not.
    if (a < 110) {
      data[i + 3] = 0;
      continue;
    }
    data[i + 3] = 255;
    const r = data[i] ?? 0;
    const g = data[i + 1] ?? 0;
    const b = data[i + 2] ?? 0;
    const grey = 0.3 * r + 0.59 * g + 0.11 * b;
    const sat = 0.78;
    // Posterize lightly to keep the pixel palette small.
    const q = (v: number): number => Math.round(v / 12) * 12;
    data[i] = q(grey + (r - grey) * sat - 6);
    data[i + 1] = q(grey + (g - grey) * sat - 2);
    data[i + 2] = q(grey + (b - grey) * sat + 6);
  }
}

async function render(job: PixelJob, defs: string): Promise<void> {
  const svg = `<svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" viewBox="0 0 100 100" width="${job.size * 4}" height="${job.size * 4}"><defs>${defs}</defs>${job.markup}</svg>`;
  const url = URL.createObjectURL(new Blob([svg], { type: 'image/svg+xml' }));
  try {
    const img = await loadImage(url);
    const canvas = document.createElement('canvas');
    canvas.width = job.size;
    canvas.height = job.size;
    const ctx = canvas.getContext('2d');
    if (!ctx) return;
    ctx.imageSmoothingEnabled = true;
    ctx.imageSmoothingQuality = 'high';
    ctx.drawImage(img, 0, 0, job.size, job.size);
    const pixels = ctx.getImageData(0, 0, job.size, job.size);
    grade(pixels.data);
    ctx.putImageData(pixels, 0, 0);
    cache.set(job.key, canvas.toDataURL('image/png'));
  } finally {
    URL.revokeObjectURL(url);
  }
}

/** Render all jobs (or take them from storage); failures leave the vector drawing in place. */
export async function preparePixelArt(jobs: PixelJob[]): Promise<void> {
  if (typeof document === 'undefined') return;
  const store = storedImages();
  const hashes = new Map(jobs.map((job) => [job.key, jobHash(job)]));
  const todo = jobs.filter((job) => {
    const hit = store[job.key];
    if (!hit || hit[0] !== hashes.get(job.key)) return true;
    cache.set(job.key, hit[1]);
    return false;
  });
  if (!todo.length) return;
  const defs = sharedDefs();
  await Promise.all(
    todo.map((job) =>
      render(job, defs).catch((err: unknown) => {
        console.warn('pixel art failed', job.key, err);
      }),
    ),
  );
  for (const job of todo) {
    const url = cache.get(job.key);
    const hash = hashes.get(job.key);
    if (url && hash) store[job.key] = [hash, url];
  }
  // Full or blocked storage: drawn again next time.
  writeKey(STORE_KEY, JSON.stringify(store));
}

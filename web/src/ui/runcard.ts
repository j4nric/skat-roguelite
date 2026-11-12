// "Teilen" on the end screen: the run as one picture to pass on, drawn on
// a canvas in the game's pixel look (slate, chalk, VT323): the character,
// how far the run came (Stadtteil and Kneipe, or the night), the numbers
// of the run as tiles, the relics as their beer mats (./emblems.ts), the
// seed (or the daily seed), the date and the game's address. Everything shown comes from the
// core's run view; nothing is computed here but the layout.

import { HOME_HOST, HOME_ORIGIN } from '../domain';
import { dateText, de, t } from '../i18n';
import type { RunView } from '../types';
import { relicCoaster } from './emblems';
import { characterPortraitId } from './people';
import { loadImage as loadOrFail, pixelUrl } from './pixelate';

/** An image, or null if it does not load. */
const loadImage = (src: string): Promise<HTMLImageElement | null> => loadOrFail(src).catch(() => null);

/** Size of the picture: 4:5, as phones and feeds show it whole. */
const W = 1080;
const H = 1350;
/** The grid every edge sits on, so the picture stays pixel-sharp. */
const PX = 6;

const C = {
  bg: '#1b1e21',
  dot: '#272c30',
  board: '#23282c',
  panel: '#2a2f34',
  line: '#0b0c0e',
  text: '#ecebe4',
  muted: '#a4adb3',
  /** Ivory: the brand line, like the sign of the start screen. */
  ivory: '#f1e6c8',
  /** Brass: Groschen. */
  money: '#ecd28a',
  cool: '#8fc7a8',
  rot: '#e5776d',
  night: '#9fb4e8',
};

const FONT = "'VT323', ui-monospace, monospace";

export interface RunCardInfo {
  won: boolean;
  /** Started as the daily seed (the browser's record; the core does not know). */
  daily: boolean;
  /** The day the run ended. */
  date: Date;
}

function snap(n: number): number {
  return Math.round(n / PX) * PX;
}

function svgImage(svg: string): Promise<HTMLImageElement | null> {
  return loadImage(`data:image/svg+xml;charset=utf-8,${encodeURIComponent(svg)}`);
}

/** A pixel box: fill, a dark edge and the hard shadow of the buttons. */
function box(g: CanvasRenderingContext2D, x: number, y: number, w: number, h: number, fill: string, shadow = PX): void {
  g.fillStyle = C.line;
  if (shadow) g.fillRect(x, y + shadow, w, h);
  g.fillRect(x - PX / 2, y - PX / 2, w + PX, h + PX);
  g.fillStyle = fill;
  g.fillRect(x + PX / 2, y + PX / 2, w - PX, h - PX);
}

function text(g: CanvasRenderingContext2D, s: string, x: number, y: number, size: number, color: string, align: CanvasTextAlign = 'left', maxWidth?: number): void {
  g.font = `${size}px ${FONT}`;
  g.fillStyle = color;
  g.textAlign = align;
  g.textBaseline = 'alphabetic';
  g.fillText(s, x, y, maxWidth);
}

/** Wrap `s` to lines no wider than `width` (at most `max` lines). */
function wrap(g: CanvasRenderingContext2D, s: string, size: number, width: number, max: number): string[] {
  g.font = `${size}px ${FONT}`;
  const lines: string[] = [];
  let line = '';
  for (const word of s.split(' ')) {
    const next = line ? `${line} ${word}` : word;
    if (g.measureText(next).width > width && line) {
      lines.push(line);
      line = word;
    } else line = next;
  }
  if (line) lines.push(line);
  return lines.slice(0, max);
}

/** Title and tone of the run's end, as the end screen says it. */
function outcome(view: RunView, won: boolean): { title: string; color: string } {
  if (won) return { title: de.end.won, color: C.cool };
  if (view.night) return { title: de.night.over, color: C.night };
  return { title: de.end.lost, color: C.rot };
}

/** "Stadtteil 2 · Kneipe 3" and the district's name; in the night its district. */
export function reachedLine(view: RunView): { line: string; name: string } {
  const s = view.series;
  const district = view.tour.districts.find((d) => d.index === s.section);
  if (view.night) return { line: t(de.runCard.night, { n: view.night.district }), name: district?.name ?? '' };
  return { line: t(de.runCard.reached, { section: s.section + 1, series: s.index + 1 }), name: district?.name ?? '' };
}

/** Draw the picture of the run. */
export async function drawRunCard(view: RunView, info: RunCardInfo): Promise<HTMLCanvasElement> {
  try {
    await document.fonts.load(`40px ${FONT}`, 'Stammtisch');
  } catch {
    // The fallback font draws the same picture, a little plainer.
  }
  const canvas = document.createElement('canvas');
  canvas.width = W;
  canvas.height = H;
  const g = canvas.getContext('2d');
  if (!g) throw new Error('no 2d canvas');
  g.imageSmoothingEnabled = false;

  // The felt of the app: slate with a dot grid.
  g.fillStyle = C.bg;
  g.fillRect(0, 0, W, H);
  g.fillStyle = C.dot;
  for (let y = PX * 2; y < H; y += PX * 4) for (let x = PX * 2; x < W; x += PX * 4) g.fillRect(x, y, PX / 2, PX / 2);

  // The chalk board.
  const M = PX * 8;
  box(g, M, M, W - 2 * M, H - 2 * M - PX * 2, C.board, PX * 2);

  // Head: the game's name, the result.
  const { title, color } = outcome(view, info.won);
  text(g, de.runCard.brand, W / 2, snap(M + 96), 64, C.ivory, 'center');
  text(g, title, W / 2, snap(M + 170), 72, color, 'center', W - 2 * M - 60);
  g.fillStyle = C.line;
  g.fillRect(M + PX * 6, snap(M + 204), W - 2 * M - PX * 12, PX);

  // The character and how far he came.
  const top = snap(M + 240);
  const face = 216;
  const fx = M + PX * 8;
  box(g, fx, top, face, face, C.panel);
  const url = pixelUrl(characterPortraitId(view.character.character));
  const img = url ? await loadImage(url) : null;
  if (img) g.drawImage(img, fx + PX, top + PX, face - 2 * PX, face - 2 * PX);
  const tx = fx + face + PX * 8;
  const tw = W - M - PX * 8 - tx;
  const name = de.characters[view.character.character]?.short ?? view.character.name;
  text(g, name, tx, top + 52, 54, C.text, 'left', tw);
  const reached = reachedLine(view);
  text(g, reached.line, tx, top + 112, 46, color, 'left', tw);
  wrap(g, reached.name, 36, tw, 1).forEach((l, i) => text(g, l, tx, top + 160 + i * 36, 36, C.muted, 'left', tw));
  if (view.stake > 0) text(g, t(de.runCard.stake, { stake: view.stake }), tx, top + 204, 32, C.text, 'left', tw);

  // The numbers of the run, as on the Kreidetafel.
  const s = view.stats;
  const tiles: [string, number][] = [
    [de.end.seriesWon, s.series_won],
    [de.end.gamesWon, s.games_won],
    [de.end.gamesLost, s.games_lost],
    [de.end.bestGame, s.best_game],
    [de.end.coins, view.coins],
    [de.end.gamesPassed, s.redeals],
  ];
  const tileTop = top + face + PX * 8;
  const cols = 3;
  const gap = PX * 4;
  const inner = W - 2 * M - PX * 16;
  const tw2 = snap((inner - gap * (cols - 1)) / cols);
  const th = 150;
  tiles.forEach(([label, value], i) => {
    const x = M + PX * 8 + (i % cols) * (tw2 + gap);
    const y = tileTop + Math.floor(i / cols) * (th + gap);
    box(g, x, y, tw2, th, C.panel);
    text(g, String(value), x + tw2 / 2, y + 86, 84, i === 3 ? C.money : C.text, 'center', tw2 - 20);
    text(g, label, x + tw2 / 2, y + 128, 32, C.muted, 'center', tw2 - 20);
  });

  // The relics as their beer mats.
  const matsTop = tileTop + 2 * th + gap + PX * 10;
  text(g, view.relics.length ? de.runCard.relics : de.runCard.noRelics, M + PX * 8, matsTop, 40, C.muted);
  const mat = 126;
  const perRow = 7;
  const step = snap((inner - mat) / (perRow - 1));
  const mats = await Promise.all(view.relics.slice(0, perRow * 2).map((r) => svgImage(relicCoaster(r.relic, { size: mat, standalone: true }))));
  mats.forEach((m, i) => {
    if (!m) return;
    const x = M + PX * 8 + (i % perRow) * step;
    const y = matsTop + PX * 4 + Math.floor(i / perRow) * (mat + PX * 2);
    g.drawImage(m, x, y, mat, mat);
  });
  if (view.relics.length > perRow * 2) {
    text(g, `+${view.relics.length - perRow * 2}`, W - M - PX * 8, matsTop, 40, C.muted, 'right');
  }

  // Foot: the seed (to play the same run) and the day.
  const footY = H - M - PX * 2 - PX * 7;
  g.fillStyle = C.line;
  g.fillRect(M + PX * 6, footY - 56, W - 2 * M - PX * 12, PX);
  const day = dateText(info.date);
  const seedText = info.daily ? t(de.runCard.daily, { seed: view.seed }) : t(de.runCard.seed, { seed: view.seed });
  text(g, seedText, M + PX * 8, footY, 40, C.text);
  text(g, day, W - M - PX * 8, footY, 40, C.muted, 'right');
  // Below the board: where to play (ADR 0025).
  text(g, HOME_HOST, W / 2, H - PX * 2, 30, C.muted, 'center');
  return canvas;
}

/** The picture as a PNG. */
export async function runCardBlob(view: RunView, info: RunCardInfo): Promise<Blob> {
  const canvas = await drawRunCard(view, info);
  return new Promise((resolve, reject) => canvas.toBlob((b) => (b ? resolve(b) : reject(new Error('no png'))), 'image/png'));
}

/** File name of the picture. */
export function runCardName(view: RunView): string {
  return t(de.runCard.file, { seed: view.seed.replace(/[^0-9A-Za-z_-]/g, '') || 'run' });
}

/** A line for the share sheet next to the picture. */
export function runCardText(view: RunView, won: boolean): string {
  const r = reachedLine(view);
  return t(de.runCard.shareText, { result: outcome(view, won).title, reached: r.line, site: HOME_ORIGIN });
}

// Draws the pixel art of the table top (style.css `.tabletop`) as small
// SVG files of plain rectangles, one unit per art pixel:
//
//   src/assets/table/lamp.svg  the pool of light the lamp throws onto the
//                              felt: three stepped ellipses, each with a
//                              dithered ring around it (no gradient),
//   src/assets/table/felt.svg  the nap of the felt, a tile of a few specks,
//   src/assets/table/rim.svg   the wooden edge as a nine-slice border image
//                              with a light upper and a dark lower side.
//
// Drawn for this game (no third-party art). The files are checked in; run
// `node scripts/gen-table-art.mjs` again after changing a shape or colour.

import { mkdirSync, writeFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const out = join(dirname(fileURLToPath(import.meta.url)), '..', 'src', 'assets', 'table');
mkdirSync(out, { recursive: true });

const svg = (w, h, body) => `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 ${w} ${h}" width="${w}" height="${h}" shape-rendering="crispEdges">${body}</svg>\n`;

/** One rectangle per row of a stepped ellipse with radii rx, ry around cx, cy. */
function ellipse(cx, cy, rx, ry, fill) {
  let d = '';
  for (let y = -ry; y < ry; y++) {
    const t = (y + 0.5) / ry;
    const half = Math.round(rx * Math.sqrt(Math.max(0, 1 - t * t)));
    if (half > 0) d += `M${cx - half} ${cy + y}h${half * 2}v1h${-half * 2}z`;
  }
  return `<path d="${d}" fill="${fill}"/>`;
}

// The light: from the rim of the pool inwards, each band a little lighter.
// The ring between two bands is a checkerboard of both (the dither).
{
  const W = 160;
  const H = 112;
  const bands = [
    { rx: 78, ry: 54, color: '#2a5142' },
    { rx: 58, ry: 39, color: '#2f5a49' },
    { rx: 36, ry: 23, color: '#35634f' },
  ];
  const ring = 5;
  let body = '<defs>';
  bands.forEach((b, i) => {
    body += `<pattern id="d${i}" width="2" height="2" patternUnits="userSpaceOnUse"><path d="M0 0h1v1h-1zM1 1h1v1h-1z" fill="${b.color}"/></pattern>`;
  });
  body += '</defs>';
  bands.forEach((b, i) => {
    body += ellipse(W / 2, H / 2, b.rx, b.ry, `url(#d${i})`);
    body += ellipse(W / 2, H / 2, b.rx - ring, b.ry - ring, b.color);
  });
  writeFileSync(join(out, 'lamp.svg'), svg(W, H, body));
}

// The nap: a few darker and lighter specks on a transparent tile.
{
  const dark = [
    [1, 2],
    [9, 6],
    [5, 12],
    [13, 10],
    [3, 15],
  ];
  const light = [
    [6, 3],
    [12, 1],
    [2, 9],
    [10, 14],
  ];
  const at = (list) => list.map(([x, y]) => `M${x} ${y}h1v1h-1z`).join('');
  writeFileSync(join(out, 'felt.svg'), svg(16, 16, `<path d="${at(dark)}" fill="#1f3f33"/><path d="${at(light)}" fill="#2f5a49"/>`));
}

// The edge: 6 art pixels of wood around a 4 × 4 middle (the felt shows
// through it). From outside in: the dark line, a light upper edge (a dark
// one below), three of wood with a line of grain, its shadow side towards
// the felt on the upper and the left side. The outer corners are cut by
// one pixel, a peg sits in each.
{
  const S = 16;
  const line = '#0b0c0e';
  const wood = '#7b4a2b';
  const light = '#a2683c';
  const dark = '#4f2e1a';
  const grain = '#6a3f24';
  let body = '';
  body += `<path d="M1 0h14v1h-14zM0 1h1v14h-1zM15 1h1v14h-1zM1 15h14v1h-14z" fill="${line}"/>`;
  body += `<path d="M1 1h14v5h-14zM1 10h14v5h-14zM1 6h5v4h-5zM10 6h5v4h-5z" fill="${wood}"/>`;
  // Light from above: the upper and the left outer edge.
  body += `<path d="M1 1h14v1h-14zM1 2h1v12h-1z" fill="${light}"/>`;
  // The grain: one darker line along each side.
  body += `<path d="M3 3h10v1h-10zM3 12h10v1h-10zM3 4h1v8h-1zM12 4h1v8h-1z" fill="${grain}"/>`;
  // Shadow: the lower and the right outer edge, and the inner upper and left edges (the felt lies lower).
  body += `<path d="M2 14h13v1h-13zM14 2h1v12h-1z" fill="${dark}"/>`;
  body += `<path d="M5 5h6v1h-6zM5 6h1v5h-1z" fill="${dark}"/>`;
  // A peg in each corner.
  body += `<path d="M2 2h2v2h-2zM12 2h2v2h-2zM2 12h2v2h-2zM12 12h2v2h-2z" fill="${dark}"/>`;
  body += `<path d="M2 2h1v1h-1zM12 2h1v1h-1zM2 12h1v1h-1zM12 12h1v1h-1z" fill="${light}"/>`;
  writeFileSync(join(out, 'rim.svg'), svg(S, S, body));
}

console.log(`table art written to ${out}`);

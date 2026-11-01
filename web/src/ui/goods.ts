// Things on the Tresen, drawn as SVG: the consumables as objects. Picked
// by content id for looks only; unknown items get a plain parcel. At start they are
// turned into pixel images (./pixelate.ts).

import { pixelUrl, type PixelJob } from './pixelate';

const OUT = '#2b1a0e';

const GOODS: Record<string, string> = {
  // Ein Bier: a Willi glass with foam.
  beer: `<path d="M30 22 L70 22 L66 88 C66 92 62 94 58 94 L42 94 C38 94 34 92 34 88 Z" fill="#f0a71c" stroke="${OUT}" stroke-width="2.5"/>
<path d="M33 40 L67 40" stroke="#c9810c" stroke-width="2"/>
<path d="M27 24 C25 12 38 10 42 14 C46 6 58 7 60 13 C66 9 76 14 73 24 C72 30 28 30 27 24 Z" fill="#fffaf0" stroke="${OUT}" stroke-width="2.5"/>
<path d="M40 50 L42 84 M50 48 L50 86" stroke="#fff4c0" stroke-width="3" stroke-linecap="round" opacity=".6"/>
<circle cx="58" cy="62" r="1.8" fill="#fff4c0"/><circle cx="55" cy="74" r="1.4" fill="#fff4c0"/>`,
  // Stempelkissen: a tin with violet ink and a rubber stamp.
  ink_pad: `<rect x="10" y="58" width="56" height="26" rx="4" fill="#5a5a62" stroke="${OUT}" stroke-width="2.5"/>
<rect x="15" y="60" width="46" height="12" rx="3" fill="#5e3796"/>
<path d="M70 84 L92 84 L90 72 L72 72 Z" fill="#8a5a34" stroke="${OUT}" stroke-width="2.5"/>
<rect x="76" y="40" width="10" height="32" rx="3" fill="#a8743f" stroke="${OUT}" stroke-width="2.5"/>
<ellipse cx="81" cy="38" rx="9" ry="6" fill="#c08a52" stroke="${OUT}" stroke-width="2.5"/>`,
  // Frisches Deck: a sealed card box.
  fresh_deck: `<rect x="24" y="18" width="52" height="74" rx="5" fill="#7a1f2b" stroke="${OUT}" stroke-width="2.5"/>
<rect x="30" y="24" width="40" height="62" rx="3" fill="none" stroke="#e3b24c" stroke-width="2"/>
<ellipse cx="50" cy="55" rx="13" ry="16" fill="#f3e9d2" stroke="#e3b24c" stroke-width="2"/>
<use href="#s-C" x="43" y="42" width="14" height="14"/><use href="#s-H" x="43" y="56" width="14" height="14"/>
<path d="M24 34 L76 34" stroke="#e3b24c" stroke-width="4" opacity=".8"/>`,
  // Neues Blatt: a freshly shuffled fan.
  new_deal: `<g stroke="${OUT}" stroke-width="2.2">
<rect x="22" y="26" width="34" height="50" rx="4" fill="#fbf6e9" transform="rotate(-22 39 76)"/>
<rect x="33" y="22" width="34" height="50" rx="4" fill="#fbf6e9" transform="rotate(-4 50 72)"/>
<rect x="44" y="26" width="34" height="50" rx="4" fill="#7a1f2b" transform="rotate(16 61 76)"/></g>
<use href="#s-S" x="28" y="40" width="14" height="14" transform="rotate(-22 39 76)"/><use href="#s-D" x="41" y="34" width="16" height="16"/>
<path d="M18 88 C34 96 66 96 82 88" fill="none" stroke="#e3b24c" stroke-width="3" stroke-linecap="round" stroke-dasharray="2 6"/>`,
};

const PARCEL = `<rect x="22" y="34" width="56" height="48" rx="4" fill="#b98a55" stroke="${OUT}" stroke-width="2.5"/>
<path d="M22 50 L78 50 M50 34 L50 82" stroke="#6b4529" stroke-width="3"/>`;

/** Jobs for ./pixelate.ts: every item as a small pixel image. */
export function goodJobs(): PixelJob[] {
  return [...Object.entries(GOODS), ['parcel', PARCEL] as const].map(([id, markup]) => ({ key: `good-${id}`, markup, size: 26 }));
}

/** An item on the counter: the pixel image, else the SVG (100 × 100). */
export function goodSvg(id: string, size = 64): string {
  const url = pixelUrl(`good-${id in GOODS ? id : 'parcel'}`);
  if (url) return `<img class="good-svg px" src="${url}" width="${size}" height="${size}" alt="" draggable="false">`;
  return `<svg class="good-svg" viewBox="0 0 100 100" width="${size}" height="${size}" aria-hidden="true">${GOODS[id] ?? PARCEL}</svg>`;
}

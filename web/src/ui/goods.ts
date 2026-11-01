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
  // Kurzer: a small shot glass, filled to the brim.
  shot: `<path d="M32 30 L68 30 L62 84 C62 88 58 90 54 90 L46 90 C42 90 38 88 38 84 Z" fill="#eef4f6" stroke="${OUT}" stroke-width="2.5"/>
<path d="M34 40 L66 40 L61 82 C61 85 58 86 55 86 L45 86 C42 86 39 85 39 82 Z" fill="#f6e7b0"/>
<path d="M44 46 L46 78" stroke="#ffffff" stroke-width="3" stroke-linecap="round" opacity=".8"/>
<path d="M38 74 L62 74" stroke="#c9b56a" stroke-width="2" opacity=".5"/>`,
  // Doppelkorn: a tall clear bottle with a blue label.
  double_grain: `<rect x="44" y="8" width="12" height="18" rx="2" fill="#c9c2a8" stroke="${OUT}" stroke-width="2.5"/>
<path d="M44 26 L56 26 L58 36 C66 40 68 46 68 54 L68 88 C68 91 66 93 63 93 L37 93 C34 93 32 91 32 88 L32 54 C32 46 34 40 42 36 Z" fill="#e7f0f2" stroke="${OUT}" stroke-width="2.5"/>
<rect x="34" y="56" width="32" height="22" fill="#2f5f9e" stroke="${OUT}" stroke-width="2"/>
<path d="M42 62 L42 72 M48 62 L48 72 M56 62 L58 67 L56 72" stroke="#f3e9d2" stroke-width="2.5" fill="none"/>`,
  // Kräuter: a dark green bottle with a sprig.
  herbal: `<rect x="44" y="10" width="12" height="16" rx="2" fill="#7a4a24" stroke="${OUT}" stroke-width="2.5"/>
<path d="M44 26 L56 26 L58 34 C68 38 70 46 70 54 L70 88 C70 91 68 93 65 93 L35 93 C32 93 30 91 30 88 L30 54 C30 46 32 38 42 34 Z" fill="#2e5b2e" stroke="${OUT}" stroke-width="2.5"/>
<rect x="34" y="54" width="32" height="24" fill="#f3e9d2" stroke="${OUT}" stroke-width="2"/>
<path d="M50 74 L50 58 M50 66 C44 64 42 60 42 58 M50 62 C56 60 58 56 58 54" stroke="#3f7a35" stroke-width="2.5" fill="none" stroke-linecap="round"/>`,
  // Magenbitter: a squat brown bottle with a red label.
  bitters: `<rect x="44" y="18" width="12" height="14" rx="2" fill="#c08a52" stroke="${OUT}" stroke-width="2.5"/>
<path d="M42 32 L58 32 L60 42 C72 44 76 52 76 60 L76 86 C76 90 73 93 69 93 L31 93 C27 93 24 90 24 86 L24 60 C24 52 28 44 40 42 Z" fill="#5a2e14" stroke="${OUT}" stroke-width="2.5"/>
<rect x="30" y="58" width="40" height="22" fill="#a8322a" stroke="${OUT}" stroke-width="2"/>
<path d="M36 69 L64 69" stroke="#f3e9d2" stroke-width="3"/>`,
  // Kreide: a stick of chalk on a little slate.
  chalk: `<rect x="14" y="30" width="72" height="54" rx="3" fill="#2f3a33" stroke="#7a4a24" stroke-width="5"/>
<path d="M24 50 C30 44 36 56 42 50 M24 64 L44 64" stroke="#e8e4d8" stroke-width="2.5" fill="none" stroke-linecap="round" opacity=".8"/>
<rect x="46" y="58" width="34" height="9" rx="3" fill="#fbf8ef" stroke="${OUT}" stroke-width="2" transform="rotate(-28 63 62)"/>`,
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

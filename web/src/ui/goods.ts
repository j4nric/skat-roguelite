// Things on the Tresen, drawn as SVG: the consumables as objects, the
// relics as Bierdeckel, and the shelf behind the Wirt. Picked by content
// id for looks only; unknown items get a plain parcel.

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

/** An item on the counter as inline SVG (100 × 100). */
export function goodSvg(id: string, size = 64): string {
  return `<svg class="good-svg" viewBox="0 0 100 100" width="${size}" height="${size}" aria-hidden="true">${GOODS[id] ?? PARCEL}</svg>`;
}

/** The shelf behind the Wirt: bottles and glasses, as a wide strip. */
export function shelfSvg(): string {
  const bottles = [
    [14, '#2f6b3a', 34], [30, '#7a3b1c', 40], [46, '#c9a24a', 30], [62, '#2f4f7a', 38], [78, '#6b1f2b', 34],
    [176, '#7a3b1c', 36], [192, '#2f6b3a', 42], [208, '#d98e0b', 30], [224, '#6b1f2b', 38], [240, '#3a5a2a', 34],
  ] as const;
  const b = bottles
    .map(
      ([x, c, h]) =>
        `<path d="M${x} 58 L${x} ${58 - h + 14} C${x} ${58 - h + 8} ${x + 3} ${58 - h + 6} ${x + 3} ${58 - h} L${x + 7} ${58 - h} C${x + 7} ${58 - h + 6} ${x + 10} ${58 - h + 8} ${x + 10} ${58 - h + 14} L${x + 10} 58 Z" fill="${c}" stroke="#1a0f08" stroke-width="1.2"/><rect x="${x + 1.5}" y="${58 - h / 2}" width="7" height="8" fill="#f3e9d2" opacity=".75"/>`,
    )
    .join('');
  const glasses = [100, 112, 124, 136, 148]
    .map((x) => `<path d="M${x} 40 L${x + 9} 40 L${x + 8} 58 L${x + 1} 58 Z" fill="#cfe6ef" fill-opacity=".35" stroke="#e9f3f6" stroke-width="1" opacity=".8"/>`)
    .join('');
  return `<svg class="shelf-svg" viewBox="0 0 260 64" preserveAspectRatio="xMidYMax slice" aria-hidden="true">
<rect x="0" y="58" width="260" height="6" fill="#5a3a22"/><rect x="0" y="58" width="260" height="1.5" fill="#8a5a34"/>${b}${glasses}</svg>`;
}

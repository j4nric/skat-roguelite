// Portraits of the people at the table, drawn as flat SVG symbols in a
// 100 × 100 box: two anonymous regulars, the Wirt, and the bosses. The
// boss portrait is picked by content id; unknown bosses get a generic one.
// Pure drawing.

const OUT = '#2b1a0e';

function shoulders(fill: string, extra = ''): string {
  return `<path d="M8 101 C10 80 27 70 50 70 C73 70 90 80 92 101 Z" fill="${fill}" stroke="${OUT}" stroke-width="2"/>${extra}`;
}

function headShape(skin: string, rx = 19, ry = 22, cy = 44): string {
  return `<rect x="43" y="${cy + ry - 8}" width="14" height="12" fill="${skin}" stroke="${OUT}" stroke-width="1.5"/>
<ellipse cx="50" cy="${cy}" rx="${rx}" ry="${ry}" fill="${skin}" stroke="${OUT}" stroke-width="2"/>
<ellipse cx="${50 - rx}" cy="${cy + 2}" rx="4" ry="6" fill="${skin}" stroke="${OUT}" stroke-width="1.6"/>
<ellipse cx="${50 + rx}" cy="${cy + 2}" rx="4" ry="6" fill="${skin}" stroke="${OUT}" stroke-width="1.6"/>`;
}

function eyes(y = 43, brow = OUT, dx = 8): string {
  return `<circle cx="${50 - dx}" cy="${y}" r="2.3" fill="${OUT}"/><circle cx="${50 + dx}" cy="${y}" r="2.3" fill="${OUT}"/>
<path d="M${50 - dx - 5} ${y - 6} L${50 - dx + 4} ${y - 7} M${50 + dx - 4} ${y - 7} L${50 + dx + 5} ${y - 6}" stroke="${brow}" stroke-width="2.6" stroke-linecap="round"/>`;
}

function cheeks(y = 52): string {
  return `<circle cx="38" cy="${y}" r="4.5" fill="#e07a62" opacity=".35"/><circle cx="62" cy="${y}" r="4.5" fill="#e07a62" opacity=".35"/>`;
}

/** Stammgast with a flat cap and a grey moustache. */
function regularCap(): string {
  const skin = '#e9b78c';
  return `${shoulders('#6b4a2e', '<path d="M38 72 L50 88 L62 72" fill="#f4efe4" stroke="#2b1a0e" stroke-width="1.6"/><path d="M50 88 L50 101" stroke="#2b1a0e" stroke-width="1.5"/>')}
${headShape(skin)}
${eyes(44, '#9a9286')}${cheeks(53)}
<ellipse cx="50" cy="51" rx="4.5" ry="5" fill="#d9826a" stroke="${OUT}" stroke-width="1.5"/>
<path d="M36 58 C40 54 46 55 50 57 C54 55 60 54 64 58 C59 62 54 60 50 60 C46 60 41 62 36 58 Z" fill="#c9c3b8" stroke="${OUT}" stroke-width="1.4"/>
<path d="M30 33 C30 20 40 15 52 15 C66 15 73 22 72 31 Z" fill="#6d5a46" stroke="${OUT}" stroke-width="2"/>
<path d="M30 32 C44 28 66 26 82 31 C76 36 62 35 50 34 C40 34 34 35 30 32 Z" fill="#5a4936" stroke="${OUT}" stroke-width="2"/>
<path d="M38 22 C46 19 58 19 66 23" fill="none" stroke="#857059" stroke-width="2"/>`;
}

/** Stammgästin with a silver bob and round glasses. */
function regularGlasses(): string {
  const skin = '#f2cba8';
  return `<path d="M27 40 C24 62 28 74 34 76 L66 76 C72 74 76 62 73 40 Z" fill="#c9c3b8" stroke="${OUT}" stroke-width="2"/>
${shoulders('#3f6b4a', '<path d="M40 72 L50 84 L60 72" fill="#2f5238" stroke="#2b1a0e" stroke-width="1.5"/><circle cx="42" cy="82" r="2" fill="#f4efe4"/><circle cx="46" cy="86" r="2" fill="#f4efe4"/><circle cx="54" cy="86" r="2" fill="#f4efe4"/><circle cx="58" cy="82" r="2" fill="#f4efe4"/><circle cx="50" cy="88" r="2" fill="#f4efe4"/>')}
${headShape(skin, 18, 21)}
<path d="M31 40 C30 22 42 18 50 18 C60 18 70 22 69 40 C62 30 48 28 31 40 Z" fill="#d6d0c4" stroke="${OUT}" stroke-width="2"/>
${eyes(45, '#8f877a', 8)}${cheeks(54)}
<circle cx="42" cy="45" r="6.5" fill="#fff" fill-opacity=".25" stroke="#7a2a3a" stroke-width="2"/><circle cx="58" cy="45" r="6.5" fill="#fff" fill-opacity=".25" stroke="#7a2a3a" stroke-width="2"/>
<path d="M48.5 45 L51.5 45" stroke="#7a2a3a" stroke-width="2"/>
<path d="M44 57 C47 60 53 60 56 57" fill="#c0505e" stroke="${OUT}" stroke-width="1.4"/>
<circle cx="31" cy="51" r="2.3" fill="#e3b24c"/><circle cx="69" cy="51" r="2.3" fill="#e3b24c"/>`;
}

/** The Wirt: bald, big moustache, apron and a towel over the shoulder. */
function wirt(): string {
  const skin = '#eab48a';
  return `${shoulders('#f4efe4', '<path d="M30 101 L33 80 L67 80 L70 101 Z" fill="#2c5e3c" stroke="#2b1a0e" stroke-width="1.6"/><path d="M36 80 L33 72 M64 80 L67 72" stroke="#2c5e3c" stroke-width="3"/><path d="M68 72 C76 74 84 82 88 101 L78 101 C76 88 72 80 66 76 Z" fill="#fff" stroke="#2b1a0e" stroke-width="1.6"/><path d="M71 78 L80 99 M75 76 L84 98" stroke="#c0262f" stroke-width="2"/>')}
${headShape(skin, 21, 23, 42)}
<path d="M29 40 C28 32 31 28 34 27 M71 40 C72 32 69 28 66 27" fill="none" stroke="#4a2a12" stroke-width="5" stroke-linecap="round"/>
<path d="M38 24 C44 21 56 21 62 24" fill="none" stroke="#fff" stroke-width="3" stroke-linecap="round" opacity=".35"/>
${eyes(41, '#4a2a12', 8.5)}${cheeks(50)}
<path d="M46 46 C47 50 53 50 54 46" fill="#d98c6c" stroke="${OUT}" stroke-width="1.3"/>
<path d="M28 52 C32 60 42 54 50 53 C58 54 68 60 72 52 C70 56 62 50 50 49 C38 50 30 56 28 52 Z" fill="#4a2a12" stroke="${OUT}" stroke-width="1.4"/>
<path d="M44 59 C47 61 53 61 56 59" fill="none" stroke="${OUT}" stroke-width="1.6" stroke-linecap="round"/>`;
}

/** Der Schieber: fedora, pencil moustache, toothpick. */
function pusher(): string {
  const skin = '#e3b38e';
  return `${shoulders('#3b3c44', '<path d="M38 72 L50 92 L62 72" fill="#f4efe4" stroke="#2b1a0e" stroke-width="1.5"/><path d="M47 76 L50 92 L53 76 Z" fill="#8e2430"/><path d="M22 86 L28 101 M78 86 L72 101 M34 80 L38 101 M66 80 L62 101" stroke="#55565f" stroke-width="1.2"/>')}
${headShape(skin, 17, 22)}
${eyes(45, OUT, 7)}
<path d="M42 41 L47 42 M53 42 L58 41" stroke="${OUT}" stroke-width="2"/>
<path d="M42 55 C46 53 54 53 58 55" fill="none" stroke="#2b1a0e" stroke-width="2.2" stroke-linecap="round"/>
<path d="M45 59 C48 60 52 60 55 59" fill="none" stroke="${OUT}" stroke-width="1.5" stroke-linecap="round"/>
<path d="M55 59 L66 63" stroke="#d9b36b" stroke-width="2" stroke-linecap="round"/>
<path d="M30 30 C30 14 40 10 50 10 C62 10 70 14 70 30 Z" fill="#2e2e33" stroke="${OUT}" stroke-width="2"/>
<path d="M40 12 C45 17 55 17 60 12" fill="none" stroke="#1e1e22" stroke-width="2.5"/>
<rect x="30" y="24" width="40" height="6" fill="#8e2430"/>
<path d="M18 31 C32 26 68 26 82 31 C74 36 26 36 18 31 Z" fill="#2e2e33" stroke="${OUT}" stroke-width="2"/>
<rect x="60" y="13" width="9" height="13" rx="1.5" transform="rotate(14 64 19)" fill="#fbf6e9" stroke="${OUT}" stroke-width="1.2"/>
<path d="M63 17 L66 20" stroke="#c21d2c" stroke-width="2"/>`;
}

/** Der Kontra-Hans: red face, angry brows, shouting. */
function kontraHans(): string {
  const skin = '#e79a7a';
  return `${shoulders('#2f4f7a', '<path d="M36 72 L44 82 L50 74 L56 82 L64 72" fill="none" stroke="#1f3456" stroke-width="2.5"/><circle cx="50" cy="90" r="2" fill="#1f3456"/>')}
${headShape(skin, 21, 22)}
<path d="M29 38 C27 22 38 16 50 16 C64 16 74 22 71 38 C68 30 62 26 50 26 C40 26 32 30 29 38 Z" fill="#1d1a18" stroke="${OUT}" stroke-width="2"/>
<path d="M33 26 L36 18 M42 21 L44 13 M52 20 L54 12 M61 21 L65 14" stroke="#1d1a18" stroke-width="3" stroke-linecap="round"/>
<circle cx="42" cy="44" r="2.4" fill="${OUT}"/><circle cx="58" cy="44" r="2.4" fill="${OUT}"/>
<path d="M34 35 L46 40 M66 35 L54 40" stroke="#1d1a18" stroke-width="4.2" stroke-linecap="round"/>
<path d="M30 50 C32 62 40 66 50 66 C60 66 68 62 70 50" fill="none" stroke="#5a4a40" stroke-width="5" stroke-dasharray="1.2 2.4" opacity=".7"/>
<ellipse cx="50" cy="57" rx="7" ry="5" fill="#5a1820" stroke="${OUT}" stroke-width="1.6"/>
<path d="M45 55 L55 55" stroke="#fff" stroke-width="1.6"/>
<path d="M76 30 L84 24 M78 40 L88 39 M76 20 L80 12" stroke="#c21d2c" stroke-width="2.4" stroke-linecap="round"/>`;
}

/** Die Oma: grey bun, round glasses, shawl and pearls. */
function grandma(): string {
  const skin = '#f1caa8';
  return `<circle cx="50" cy="14" r="9" fill="#d9d4ca" stroke="${OUT}" stroke-width="2"/>
${shoulders('#6b3d7a', '<path d="M14 92 C30 78 70 78 86 92 L80 101 L20 101 Z" fill="#8a5a98" stroke="#2b1a0e" stroke-width="1.6"/><path d="M22 96 L26 90 M34 92 L36 86 M64 86 L66 92 M74 90 L78 96" stroke="#c6a3d1" stroke-width="2"/><circle cx="41" cy="76" r="2.2" fill="#fbf6e9"/><circle cx="45.5" cy="79" r="2.2" fill="#fbf6e9"/><circle cx="50" cy="80" r="2.2" fill="#fbf6e9"/><circle cx="54.5" cy="79" r="2.2" fill="#fbf6e9"/><circle cx="59" cy="76" r="2.2" fill="#fbf6e9"/>')}
${headShape(skin, 18, 21)}
<path d="M31 42 C29 26 40 20 50 20 C61 20 71 26 69 42 C66 32 58 29 50 29 C42 29 34 32 31 42 Z" fill="#d9d4ca" stroke="${OUT}" stroke-width="2"/>
<path d="M50 20 C46 26 42 28 36 30 M50 20 C54 26 58 28 64 30" fill="none" stroke="#b9b3a8" stroke-width="1.5"/>
${eyes(45, '#b9b3a8', 8)}${cheeks(54)}
<circle cx="42" cy="45" r="6.5" fill="#fff" fill-opacity=".25" stroke="#7a5a2a" stroke-width="1.8"/><circle cx="58" cy="45" r="6.5" fill="#fff" fill-opacity=".25" stroke="#7a5a2a" stroke-width="1.8"/>
<path d="M48.5 45 L51.5 45" stroke="#7a5a2a" stroke-width="1.8"/>
<path d="M43 56 C46 60 54 60 57 56" fill="none" stroke="${OUT}" stroke-width="1.8" stroke-linecap="round"/>
<path d="M38 52 L40 54 M62 52 L60 54" stroke="#c99a7a" stroke-width="1.2"/>`;
}

/** A boss without its own portrait: top hat and cigar. */
function genericBoss(): string {
  const skin = '#e6b48c';
  return `${shoulders('#20201f', '<path d="M38 72 L50 90 L62 72" fill="#f4efe4" stroke="#2b1a0e" stroke-width="1.5"/><path d="M44 76 L50 80 L56 76 L56 82 L50 80 L44 82 Z" fill="#8e2430" stroke="#2b1a0e" stroke-width="1"/>')}
${headShape(skin, 18, 21, 46)}
${eyes(46, OUT, 7.5)}
<circle cx="57.5" cy="46" r="5" fill="none" stroke="#c9a24a" stroke-width="1.6"/>
<path d="M62 50 L66 64" stroke="#c9a24a" stroke-width="1"/>
<path d="M41 58 C46 56 54 56 59 58" fill="none" stroke="${OUT}" stroke-width="2.4" stroke-linecap="round"/>
<rect x="55" y="57" width="18" height="5" rx="2" fill="#7a4a1c" stroke="${OUT}" stroke-width="1.2"/>
<circle cx="73" cy="59.5" r="2" fill="#e8651f"/>
<path d="M75 55 C78 50 74 46 78 41 C82 37 78 33 80 30" fill="none" stroke="#c9c3b8" stroke-width="2" stroke-linecap="round" opacity=".7"/>
<rect x="33" y="6" width="34" height="24" fill="#1a1a1a" stroke="${OUT}" stroke-width="2"/>
<rect x="33" y="24" width="34" height="5" fill="#8e2430"/>
<ellipse cx="50" cy="30" rx="25" ry="4" fill="#1a1a1a" stroke="${OUT}" stroke-width="2"/>`;
}

/** Der Alleinspieler: slick parting, waistcoat with a watch chain, sure smile. */
function declarer(): string {
  const skin = '#ecb98f';
  return `${shoulders('#3a2a20', '<path d="M34 72 L50 96 L66 72 Z" fill="#f4efe4" stroke="#2b1a0e" stroke-width="1.5"/><path d="M30 74 L42 101 L58 101 L70 74 C64 80 58 84 50 86 C42 84 36 80 30 74 Z" fill="#8e2430" stroke="#2b1a0e" stroke-width="1.6"/><circle cx="46" cy="90" r="1.6" fill="#e2b04a"/><circle cx="46" cy="97" r="1.6" fill="#e2b04a"/><path d="M47 92 C52 96 58 95 61 90" fill="none" stroke="#e2b04a" stroke-width="1.4"/><path d="M45 74 L50 80 L55 74 L50 77 Z" fill="#1f3456" stroke="#2b1a0e" stroke-width="1"/>')}
${headShape(skin, 18, 22)}
<path d="M31 38 C29 20 42 16 52 16 C64 16 72 24 69 38 C66 30 60 27 54 27 L44 33 C40 30 34 33 31 38 Z" fill="#3b2414" stroke="${OUT}" stroke-width="2"/>
<path d="M44 33 C48 24 58 22 64 26" fill="none" stroke="#5a3a22" stroke-width="1.5"/>
${eyes(44, '#3b2414', 8)}${cheeks(53)}
<path d="M42 56 C46 61 55 61 59 55" fill="#fff" stroke="${OUT}" stroke-width="1.6" stroke-linecap="round"/>
<path d="M43 52 C46 50 54 50 57 52" fill="none" stroke="#3b2414" stroke-width="2.4" stroke-linecap="round"/>`;
}

/** Der Nullspieler: thin, big round glasses, bow tie, pencil behind the ear. */
function nullPlayer(): string {
  const skin = '#f0c9a6';
  return `${shoulders('#5b6470', '<path d="M38 72 L50 88 L62 72" fill="#f4efe4" stroke="#2b1a0e" stroke-width="1.5"/><path d="M41 76 L50 80 L59 76 L59 84 L50 80 L41 84 Z" fill="#2c5e3c" stroke="#2b1a0e" stroke-width="1.2"/><path d="M30 80 L36 101 M70 80 L64 101" stroke="#474f5a" stroke-width="1.4"/>')}
${headShape(skin, 16, 23, 44)}
<path d="M34 34 C34 26 38 24 40 24 M66 34 C66 26 62 24 60 24" fill="none" stroke="#8a7a66" stroke-width="4" stroke-linecap="round"/>
<path d="M38 24 C44 20 56 20 62 24" fill="none" stroke="#fff" stroke-width="2.5" stroke-linecap="round" opacity=".3"/>
<circle cx="42" cy="45" r="8" fill="#fff" fill-opacity=".3" stroke="#1d1a18" stroke-width="2.4"/><circle cx="58" cy="45" r="8" fill="#fff" fill-opacity=".3" stroke="#1d1a18" stroke-width="2.4"/>
<path d="M50 44 L50 44" stroke="#1d1a18" stroke-width="2"/><path d="M48 44 C49 42.5 51 42.5 52 44" fill="none" stroke="#1d1a18" stroke-width="2"/>
<circle cx="42" cy="45" r="2" fill="${OUT}"/><circle cx="58" cy="45" r="2" fill="${OUT}"/>
<path d="M45 59 C48 58 52 58 55 59" fill="none" stroke="${OUT}" stroke-width="1.8" stroke-linecap="round"/>
<path d="M64 28 L80 18" stroke="#e2b04a" stroke-width="4" stroke-linecap="round"/><path d="M80 18 L84 15.5" stroke="#f1caa8" stroke-width="4" stroke-linecap="round"/>`;
}

/** Der Kartengeber: green eyeshade, sleeve garters, curled moustache. */
function dealer(): string {
  const skin = '#e8b48c';
  return `${shoulders('#f4efe4', '<path d="M38 72 L50 84 L62 72" fill="none" stroke="#2b1a0e" stroke-width="1.5"/><path d="M44 76 L50 81 L56 76" fill="#1d1a18"/><path d="M14 92 L26 88 M86 92 L74 88" stroke="#c0262f" stroke-width="4" stroke-linecap="round"/><path d="M30 101 L33 84 L67 84 L70 101" fill="#3b2a1e" stroke="#2b1a0e" stroke-width="1.4"/>')}
${headShape(skin, 18, 22, 46)}
<path d="M32 40 C30 30 36 26 50 26 C64 26 70 30 68 40" fill="#2b1d10" stroke="${OUT}" stroke-width="1.8"/>
${eyes(47, OUT, 7.5)}${cheeks(55)}
<path d="M36 57 C40 52 46 54 50 56 C54 54 60 52 64 57 C61 55 58 58 55 58 C53 58 51 57 50 57 C49 57 47 58 45 58 C42 58 39 55 36 57 Z" fill="#2b1d10" stroke="${OUT}" stroke-width="1.2"/>
<path d="M36 57 C34 55 34 53 36 52 M64 57 C66 55 66 53 64 52" fill="none" stroke="#2b1d10" stroke-width="1.6"/>
<path d="M22 36 C34 26 66 26 78 36 L72 42 C60 35 40 35 28 42 Z" fill="#2f8a52" fill-opacity=".85" stroke="${OUT}" stroke-width="2"/>
<path d="M30 30 C40 24 60 24 70 30" fill="none" stroke="#1d5e36" stroke-width="3"/>`;
}

/** Alter Bekannter: Tyrolean hat with a feather, full beard, loden jacket. */
function oldFriend(): string {
  const skin = '#e3ad84';
  return `${shoulders('#4d5e3a', '<path d="M40 72 L50 82 L60 72" fill="#c9c3b8" stroke="#2b1a0e" stroke-width="1.4"/><path d="M36 74 L42 101 M64 74 L58 101" stroke="#2f3b22" stroke-width="3"/><circle cx="44" cy="88" r="2" fill="#c9a24a"/><circle cx="56" cy="88" r="2" fill="#c9a24a"/>')}
${headShape(skin, 18, 21, 46)}
${eyes(45, '#6d5a46', 8)}${cheeks(52)}
<path d="M32 50 C32 66 40 72 50 72 C60 72 68 66 68 50 C64 56 58 58 50 58 C42 58 36 56 32 50 Z" fill="#8a7a66" stroke="${OUT}" stroke-width="1.6"/>
<path d="M44 59 C47 62 53 62 56 59" fill="#c0505e" stroke="${OUT}" stroke-width="1.2"/>
<path d="M28 33 C30 18 40 14 50 14 C62 14 70 18 72 33 Z" fill="#3e5230" stroke="${OUT}" stroke-width="2"/>
<rect x="29" y="27" width="42" height="5" fill="#8e2430"/>
<path d="M16 34 C30 29 70 29 84 34 C74 38 26 38 16 34 Z" fill="#3e5230" stroke="${OUT}" stroke-width="2"/>
<path d="M66 28 C74 14 82 8 90 4 C84 12 78 20 70 28 Z" fill="#f4efe4" stroke="${OUT}" stroke-width="1.2"/><path d="M70 26 L86 8" stroke="#8a8170" stroke-width="1"/>`;
}

/** Der Alte: the final boss, long white beard, bushy brows, sailor's cap. */
function oldMan(): string {
  const skin = '#e9bf9c';
  return `${shoulders('#1f2a44', '<path d="M30 80 L34 101 M70 80 L66 101" stroke="#141c30" stroke-width="2"/><circle cx="38" cy="90" r="2.2" fill="#e2b04a"/><circle cx="62" cy="90" r="2.2" fill="#e2b04a"/>')}
${headShape(skin, 18, 21, 46)}
<path d="M30 52 C30 74 40 90 50 92 C60 90 70 74 70 52 C64 60 58 62 50 62 C42 62 36 60 30 52 Z" fill="#f4f1ea" stroke="${OUT}" stroke-width="1.8"/>
<path d="M40 70 C44 78 48 84 50 88 M60 70 C56 78 52 84 50 88 M50 66 L50 86" fill="none" stroke="#c9c3b8" stroke-width="1.2"/>
<path d="M38 55 C42 52 47 54 50 56 C53 54 58 52 62 55 C58 58 54 58 50 58 C46 58 42 58 38 55 Z" fill="#e6e1d6" stroke="${OUT}" stroke-width="1.2"/>
<circle cx="42" cy="46" r="2.2" fill="${OUT}"/><circle cx="58" cy="46" r="2.2" fill="${OUT}"/>
<path d="M33 41 C37 36 44 37 47 40 M53 40 C56 37 63 36 67 41" fill="none" stroke="#f4f1ea" stroke-width="4.5" stroke-linecap="round"/>
<path d="M33 41 C37 36 44 37 47 40 M53 40 C56 37 63 36 67 41" fill="none" stroke="${OUT}" stroke-width="1" stroke-linecap="round" opacity=".5"/>
<path d="M30 32 C30 20 40 17 50 17 C60 17 70 20 70 32 Z" fill="#1f2a44" stroke="${OUT}" stroke-width="2"/>
<path d="M26 33 C36 30 64 30 74 33 L72 37 C60 35 40 35 28 37 Z" fill="#111" stroke="${OUT}" stroke-width="1.6"/>
<path d="M44 24 L50 21 L56 24 L50 27 Z" fill="#e2b04a" stroke="#8a6512" stroke-width="1"/>`;
}

/** Der Ramscher: stubble, wool cap, a cigarette behind the ear. */
function ramscher(): string {
  const skin = '#dfa982';
  return `${shoulders('#5a3a2a', '<path d="M38 72 L50 82 L62 72" fill="#3a2a20" stroke="#2b1a0e" stroke-width="1.4"/><path d="M24 84 L76 84" stroke="#4a2e20" stroke-width="2" stroke-dasharray="4 3"/>')}
${headShape(skin, 19, 21, 46)}
<path d="M32 54 C34 66 42 70 50 70 C58 70 66 66 68 54" fill="none" stroke="#6d5a46" stroke-width="7" stroke-dasharray="0.8 2.2" opacity=".55"/>
${eyes(46, OUT, 8)}
<path d="M42 60 C46 57 54 58 58 61" fill="none" stroke="${OUT}" stroke-width="2.2" stroke-linecap="round"/>
<path d="M29 38 C28 20 40 14 50 14 C60 14 72 20 71 38 Z" fill="#c0262f" stroke="${OUT}" stroke-width="2"/>
<path d="M28 34 L72 34 L72 40 L28 40 Z" fill="#9e1d26" stroke="${OUT}" stroke-width="1.6"/>
<path d="M36 22 L36 34 M44 18 L44 34 M52 17 L52 34 M60 19 L60 34" stroke="#9e1d26" stroke-width="1.5"/>
<circle cx="50" cy="12" r="5" fill="#f4efe4" stroke="${OUT}" stroke-width="1.4"/>
<path d="M68 44 L82 36" stroke="#f4efe4" stroke-width="3.2" stroke-linecap="round"/><path d="M80 37 L83 35.5" stroke="#d98c3c" stroke-width="3.2" stroke-linecap="round"/>`;
}

const PORTRAITS: Record<string, () => string> = {
  'p-opp-1': regularCap,
  'p-opp-2': regularGlasses,
  'p-wirt': wirt,
  'p-boss-the_pusher': pusher,
  'p-boss-kontra_hans': kontraHans,
  'p-boss-grandma': grandma,
  'p-boss-the_old_man': oldMan,
  'p-boss-ramscher': ramscher,
  'p-boss': genericBoss,
  'p-char-Declarer': declarer,
  'p-char-NullPlayer': nullPlayer,
  'p-dealer': dealer,
  'p-friend': oldFriend,
};

/** Portrait of a character; unknown characters get the Alleinspieler. */
export function characterPortraitId(character: string): string {
  const id = `p-char-${character}`;
  return id in PORTRAITS ? id : 'p-char-Declarer';
}

/** Who speaks in an encounter, by encounter id. */
const ENCOUNTER_FACES: Record<string, string[]> = {
  stammtisch_dispute: ['p-opp-1', 'p-opp-2'],
  card_dealer: ['p-dealer'],
  old_friend: ['p-friend'],
  last_round: ['p-wirt'],
  spilled_beer: ['p-opp-1'],
  crown_inn: ['p-wirt'],
  conjurer: ['p-dealer'],
  prize_skat: ['p-opp-1', 'p-opp-2'],
  guest_from_altenburg: ['p-friend'],
};

/** Portrait ids for an encounter; unknown ones get the Wirt. */
export function encounterFaces(encounterId: string): string[] {
  return ENCOUNTER_FACES[encounterId] ?? ['p-wirt'];
}

export function peopleSymbols(): string {
  return Object.entries(PORTRAITS)
    .map(([id, draw]) => `<symbol id="${id}" viewBox="0 0 100 100">${draw()}</symbol>`)
    .join('');
}

/** Symbol id of a boss portrait. */
export function bossPortraitId(bossId: string): string {
  const id = `p-boss-${bossId}`;
  return id in PORTRAITS ? id : 'p-boss';
}

/** A portrait as inline SVG; the frame comes from CSS. */
export function portrait(id: string, className = ''): string {
  return `<span class="portrait ${className}" aria-hidden="true"><svg viewBox="0 0 100 100"><use href="#${id}"/></svg></span>`;
}

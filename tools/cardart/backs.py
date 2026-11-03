"""Card backs of the opponents (ADR 0015), approved on 02.11.2026, and of
the characters' editions (DESIGN 9.3, 03.11.2026: Drückeberger,
Schneiderin, Bruderherz; the Distel stays in web/src/ui/backs.ts).

    python3 tools/cardart/backs.py            # writes web/src/ui/backs.gen.ts
    python3 tools/cardart/backs.py --check    # fails if the committed file is stale


Same box and family layout as the Distel (web/src/ui/backs.ts, 120 x 180):
a rim, a dark frame, four panels of 42 x 70 and a cross band of 12. Panel
motifs are pixel art on cells of 3 units (14 x 23 cells per panel); the
lower panels are the upper ones turned over, as on the Distel. Each back
has a "small" picture for the opponents' fans: rim, frame and one big icon.
Stammgaeste use a calmer layout without the cross band (a lattice)."""

PANEL_W, PANEL_H, BAND = 42, 70, 12
LEFT, TOP = 10, 12
RIGHT = LEFT + PANEL_W + 2 + BAND + 2
BOTTOM = TOP + PANEL_H + 2 + BAND + 2
CELL = 3


def runs(rows, ch, x, y, u):
    d = []
    for j, row in enumerate(rows):
        i = 0
        while i < len(row):
            if row[i] != ch:
                i += 1
                continue
            k = i
            while k < len(row) and row[k] == ch:
                k += 1
            d.append(f'M{x + i * u} {y + j * u}h{(k - i) * u}v{u}h{-(k - i) * u}z')
            i = k
    return ''.join(d)


def grid(rows, x, y, cols, u=CELL):
    out = ''
    for ch, fill in cols.items():
        d = runs(rows, ch, x, y, u)
        if d:
            out += f'<path d="{d}" fill="{fill}"/>'
    return out


def flip(rows):
    return rows[::-1]


def mirror(rows):
    return [r[::-1] for r in rows]


def rim(c_rim, c_frame, c_line=None):
    out = f'<rect width="120" height="180" fill="{c_rim}"/>'
    if c_line:
        out += f'<path d="M2 2h116v1h-116zM2 177h116v1h-116zM2 3h1v174h-1zM117 3h1v174h-1z" fill="{c_line}"/>'
    out += (f'<rect x="8" y="10" width="104" height="160" fill="{c_frame}"/>'
            f'<path d="M8 10h2v2h-2zM110 10h2v2h-2zM8 168h2v2h-2zM110 168h2v2h-2z" fill="{c_rim}"/>')
    return out


def band(x, y, length, vertical, tile, cols):
    """A cross-band stretch: a tile of 4 x n cells repeated along it."""
    n = len(tile)
    rows = []
    k = 0
    for c in range(length // CELL):
        rows.append(tile[c % n])
    if vertical:
        return grid(rows, x, y, cols)
    rows_h = [''.join(r[i] for r in rows) for i in range(4)]
    return grid(rows_h, x, y, cols)


def family(spec):
    """Rim, frame, four panels and the cross band, like the Distel."""
    c = spec['cols']
    up = spec['panel']
    ground = spec['ground']
    out = rim(spec['rim'], spec['frame'], spec.get('rimline'))
    for px, py, rows in ((LEFT, TOP, up), (RIGHT, TOP, spec.get('panel_r', up)),
                         (LEFT, BOTTOM, flip(up)), (RIGHT, BOTTOM, flip(spec.get('panel_r', up)))):
        out += f'<rect x="{px}" y="{py}" width="{PANEL_W}" height="{PANEL_H}" fill="{spec.get("ground_at", {}).get((px, py), ground)}"/>'
        cc = dict(c)
        if 'cols_at' in spec and (px, py) in spec['cols_at']:
            cc = spec['cols_at'][(px, py)]
        out += grid(rows, px, py + 0.5, cc)
    vx = LEFT + PANEL_W + 2
    hy = TOP + PANEL_H + 2
    t, tc = spec['band']
    out += band(vx, TOP, PANEL_H + 2, True, t, tc) + band(vx, hy + BAND, PANEL_H + 2, True, t, tc)
    out += band(LEFT, hy, PANEL_W + 2, False, t, tc) + band(vx + BAND, hy, PANEL_W + 2, False, t, tc)
    out += f'<rect x="{vx}" y="{hy}" width="{BAND}" height="{BAND}" fill="{spec["cross"][0]}"/>'
    out += f'<rect x="{vx + 3}" y="{hy + 3}" width="6" height="6" fill="{spec["cross"][1]}"/>'
    return out


def small(spec):
    """Tiny version: rim, frame, ground, the band as a plain cross, one big icon."""
    icon = spec['icon']
    w, h = len(icon[0]), len(icon)
    u = min(84 // w, 120 // h)
    x0, y0 = 60 - w * u / 2, 90 - h * u / 2
    out = rim(spec['rim'], spec['frame'])
    out += f'<rect x="14" y="16" width="92" height="148" fill="{spec["ground"]}"/>'
    if spec.get('band'):
        out += f'<rect x="54" y="16" width="12" height="148" fill="{spec["cross"][0]}"/><rect x="14" y="84" width="92" height="12" fill="{spec["cross"][0]}"/>'
    out += grid(icon, x0, y0, spec['icon_cols'], u)
    return out


# ------------------------------------------------------------ colours (no suit colours)
CREAM = '#f1e6cc'
INK = '#1c1a1a'
GOLDW = '#c9a24a'      # warm tavern gold
GOLDD = '#8a6a22'
WINE = '#6e2232'
WINE_D = '#43121d'
PETROL = '#21585c'
PETROL_D = '#123638'
SLATE = '#3a4048'
SLATE_D = '#22262c'
MAUVE = '#a86a86'
MAUVE_D = '#5e3448'
OLIVE = '#7a6e44'
OLIVE_D = '#3e3720'
CRIMSON = '#8c1d2e'
GREY = '#cfc8b8'
AMBER = '#e2a33a'
WHITE = '#fbf6e8'

# ------------------------------------------------------------ Stammgaeste: calm lattice


def stammgast_full():
    out = rim(CREAM, PETROL_D)
    out += f'<rect x="12" y="14" width="96" height="152" fill="{PETROL}"/>'
    d = ''
    for j in range(0, 19):
        for i in range(0, 12):
            if (i + j) % 2 == 0:
                x, y = 12 + i * 8, 14 + j * 8
                d += f'M{x + 4} {y}l4 4l-4 4l-4 -4z'
    out += f'<path d="{d}" fill="{PETROL_D}"/>'
    dots = ''.join(f'M{12 + i * 8 - 1} {14 + j * 8 - 1}h2v2h-2z' for j in range(1, 19) for i in range(1, 12)
                   if (i + j) % 2 == 0)
    out += f'<path d="{dots}" fill="{GOLDW}"/>'
    # a small tavern medallion: a beer mug in a cream diamond
    out += f'<path d="M60 72l18 18l-18 18l-18 -18z" fill="{CREAM}"/><path d="M60 75l15 15l-15 15l-15 -15z" fill="{PETROL_D}"/>'
    mug = ["..www.", ".wwwww", ".ggggg.", ".gggggg", ".ggggg.g", ".ggggg.g", ".gggggg", ".ggggg.", ".ggggg."]
    out += grid(mug, 50, 79, {'w': WHITE, 'g': GOLDW}, 2.5)
    return out


def stammgast_small():
    out = rim(CREAM, PETROL_D)
    out += f'<rect x="14" y="16" width="92" height="148" fill="{PETROL}"/>'
    out += f'<path d="M60 60l30 30l-30 30l-30 -30z" fill="{CREAM}"/><path d="M60 70l20 20l-20 20l-20 -20z" fill="{GOLDW}"/>'
    return out


# ------------------------------------------------------------ boss motifs (14 x 23 cells)
CAT = ["..............",
       "..............",
       "...k.....k....",
       "...kk...kk....",
       "...kkkkkkk....",
       "...kakkkak....",
       "...kkkkkkk....",
       "....kkpkk.....",
       ".....kkk......",
       "....kkkkk.....",
       "...kkkkkkk....",
       "...kkkkkkk....",
       "..kkkkkkkkk...",
       "..kkkkkkkkk...",
       "..kkkkkkkkk..k",
       "..kkkkkkkkk..k",
       "..kkkkkkkkk.k.",
       "..kkkkkkkkkk..",
       "...kk.kk.kk...",
       "..............",
       "..............",
       "..............",
       ".............."]
PETER_R = ["..............",
           "..............",
           "....kkkkkk....",
           "...kwwwwwwk...",
           "...kwkwwkwk...",
           "...kwwwwwwk...",
           "...kwwrrwwk...",
           "....kwwwwk....",
           ".....kkkk.....",
           "....kkkkkk....",
           "...kkwkkwkk...",
           "...kkkkkkkk...",
           "..kkkwkkwkkk..",
           "..kk.kkkk.kk..",
           "..k..kkkk..k..",
           ".....kk.kk....",
           ".....kk.kk....",
           ".....kk.kk....",
           "....kkk.kkk...",
           "..............",
           "..............",
           "..............",
           ".............."]
DOILY = ["..............",
         "....w..w..w...",
         "...www.ww.www.",
         "..w.mmmmmmm.w.",
         ".www.m.w.m.www",
         "..m.mwwwwwm.m.",
         ".wmwmw.m.wmwmw",
         "..mwm.mmm.mwm.",
         ".wmw.mm.mm.wmw",
         "..mwm.mmm.mwm.",
         ".wmwmw.m.wmwmw",
         "..m.mwwwwwm.m.",
         ".www.m.w.m.www",
         "..w.mmmmmmm.w.",
         "...www.ww.www.",
         "....w..w..w...",
         "..............",
         ".......n......",
         "...ooo.n......",
         "..ooooon......",
         "..ooonoo......",
         "..oonooo......",
         "...ooo........"]
ARROWS = ["..............",
          "..............",
          "......y.......",
          "......yy......",
          "yyyyyyyyy.....",
          "yyyyyyyyyy....",
          "yyyyyyyyy.....",
          "......yy......",
          "......y.......",
          "..............",
          ".......w......",
          "......ww......",
          ".....wwwwwwwww",
          "....wwwwwwwwww",
          ".....wwwwwwwww",
          "......ww......",
          ".......w......",
          "..............",
          "......y.......",
          "......yy......",
          "yyyyyyyyy.....",
          "yyyyyyyyyy....",
          "yyyyyyyyy....."]
RAMSCH = ["..............",
          "..w......oo...",
          ".www....o..o..",
          ".www.....oo...",
          ".www....oooo..",
          ".www....o..o..",
          "..y.....oooo..",
          "..............",
          "..............",
          "...gggg.......",
          "..g....g...yy.",
          "..gggggg..y..y",
          "..gggggg...yy.",
          "...gggg....y..",
          "...........yyy",
          "...........y..",
          "..............",
          "..o...........",
          "..ooo.....www.",
          "..ooooo...w.w.",
          "..ooooooo.www.",
          "..ooooooo.....",
          ".............."]
SWORDS = ["..............",
          "..w.........w.",
          "...w.......w..",
          "....w.....w...",
          ".....w...w....",
          "......w.w.....",
          ".......w......",
          "......w.w.....",
          ".....w...w....",
          "...yyw...wyy..",
          "....y.....y...",
          "...o.......o..",
          "..o.........o.",
          ".y...........y",
          "..............",
          "..............",
          "...yyyyyyyy...",
          "...yk.kyyyy...",
          "...ykkyyyyy...",
          "...yk.kyyyy...",
          "....yyyyyy....",
          ".....yyyy.....",
          "......yy......"]
STAR = ["..............",
        "..............",
        "......y.......",
        "......y.......",
        ".....yyy......",
        "yyyyyyyyyyyyy.",
        ".yyyyyyyyyyy..",
        "...yyyyyyy....",
        "...yyyyyyy....",
        "..yyy...yyy...",
        "..yy.....yy...",
        ".y.........y..",
        "..............",
        "rrrr....rrrr..",
        "rrrrrrrrrrrrrr",
        "..rrrrrrrr..rr",
        "..............",
        "kkkk....kkkk..",
        "kkkkkkkkkkkkkk",
        "..kkkkkkkk..kk",
        "..............",
        "..............",
        ".............."]
LAPWING = ["..............",
           "........kk....",
           ".......k......",
           "......kkk.....",
           ".....kkkkk....",
           ".....kwkkkkk..",
           ".....kkkk.....",
           "....kkkkk.....",
           "...kkkkkkw....",
           "..kkkkkkwww...",
           ".kkkkkkwwww...",
           "kkk.kkwwwww...",
           "......ww......",
           "......w.w.....",
           "..............",
           "..r.......r...",
           "...r.....r....",
           "....r...r.....",
           ".....r.r......",
           "......r.......",
           ".....r.r......",
           "....r...r.....",
           "...r.....r...."]
TWIN = ["..............",
        "..............",
        ".....kkkk.....",
        "....kssssk....",
        "....kskssk....",
        "....kssssk....",
        "....kssrsk....",
        ".....kssk.....",
        "....kkkkkk....",
        "...kddddddk...",
        "..kdddwdddd...",
        "..kddwwwddk...",
        "..kdddwdddk...",
        "..kddddddk....",
        "...kddddk.....",
        "....kddk......",
        "....kddk......",
        "....kk.kk.....",
        "....kk.kk.....",
        "...kkk.kkk....",
        "..............",
        "..............",
        ".............."]
CROWN = ["..............",
         "..............",
         "..y...y...y...",
         "..yy.yyy.yy...",
         "..yyyyyyyyy...",
         "..ywyywyywy...",
         "..yyyyyyyyy...",
         "..............",
         "..............",
         ".y...........y",
         ".yy.........yy",
         "..yy.......yy.",
         "...yy.....yy..",
         "....yy...yy...",
         ".....yy.yy....",
         "......yyy.....",
         ".......y......",
         "..............",
         "....yyyyyy....",
         "...y......y...",
         "...y.yyyy.y...",
         "...y......y...",
         "....yyyyyy...."]

TRI = ["wk..", "kw..", "..wk", "..kw"]       # checks, Schwarzer Peter
CHEV = ["w..w", ".ww.", "....", "w..w"]
WAVE = ["r..r", ".rr.", "....", "...."]
DOTS = ["w...", "....", "..w.", "...."]
STRIPE = ["yyyy", "....", "kkkk", "...."]
ARROWB = ["..y.", ".yy.", "yyy.", ".yy."]
CHK = ["wkwk", "kwkw", "wkwk", "kwkw"]

BOSSES = {
    'schwarzer_peter': dict(
        name='Schwarzer Peter', desc='schwarze Katze und der „Schwarze Peter“ auf Grau, Schachbrett-Kreuz',
        rim='#2b2b2e', frame=INK, ground=GREY, panel=CAT, panel_r=PETER_R,
        cols={'k': INK, 'a': AMBER, 'p': '#c98a8a', 'w': WHITE, 'r': CRIMSON},
        band=(CHK, {'w': WHITE, 'k': INK}), cross=(INK, AMBER),
        icon=CAT[2:19], icon_cols={'k': INK, 'a': AMBER, 'p': '#c98a8a'}),
    'oma': dict(
        name='Oma', desc='Häkeldeckchen und Wollknäuel mit Stricknadel, Mauve auf Creme, Spitzenband',
        rim=MAUVE, frame=MAUVE_D, ground=CREAM, panel=DOILY,
        cols={'w': WHITE, 'm': MAUVE, 'o': MAUVE_D, 'n': GOLDD},
        band=(DOTS, {'w': WHITE}), cross=(MAUVE_D, CREAM),
        icon=DOILY[1:16], icon_cols={'w': WHITE, 'm': MAUVE}),
    'schieber': dict(
        name='Schieber', desc='Schiebepfeile nach links und rechts, Gold und Weiß auf Petrol',
        rim=PETROL, frame=PETROL_D, ground='#2f7376', panel=ARROWS, panel_r=mirror(ARROWS),
        cols={'y': GOLDW, 'w': WHITE},
        band=(ARROWB, {'y': GOLDW}), cross=(PETROL_D, GOLDW),
        icon=ARROWS[2:17], icon_cols={'y': GOLDW, 'w': WHITE}),
    'ramscher': dict(
        name='Ramscher', desc='Trödel: Flasche, Kanne, Topf, Schlüssel, Schuh durcheinander auf Oliv',
        rim=OLIVE, frame=OLIVE_D, ground='#b8ab84', panel=RAMSCH, panel_r=mirror(RAMSCH),
        cols={'w': WHITE, 'o': OLIVE_D, 'g': '#5e6a6a', 'y': GOLDD},
        band=(STRIPE, {'y': GOLDD, 'k': OLIVE_D}), cross=(OLIVE_D, GOLDW),
        icon=RAMSCH[1:16], icon_cols={'w': WHITE, 'o': OLIVE_D, 'g': '#5e6a6a', 'y': GOLDD}),
    'kontra_hans': dict(
        name='Kontra-Hans', desc='gekreuzte Säbel über einem goldenen K-Schild, Gold auf Ochsenblut, schwarz-goldenes Band',
        rim=GOLDW, frame=WINE_D, ground=WINE, panel=SWORDS,
        cols={'w': WHITE, 'y': GOLDW, 'o': GOLDD, 'k': INK},
        band=(STRIPE, {'y': GOLDW, 'k': INK}), cross=(INK, GOLDW),
        icon=SWORDS[1:14], icon_cols={'w': WHITE, 'y': GOLDW, 'o': GOLDD}),
    'revolutionaer': dict(
        name='Revolutionär', desc='goldener Stern über wehenden Fahnenstreifen, Gold auf Schwarz und Karmin',
        rim=CRIMSON, frame=INK, ground='#2a1a1c', panel=STAR,
        cols={'y': GOLDW, 'r': CRIMSON, 'k': '#5a2a30'},
        band=(WAVE, {'r': CRIMSON}), cross=(CRIMSON, GOLDW),
        icon=STAR[2:12], icon_cols={'y': GOLDW}),
    'kibitz_killer': dict(
        name='Kiebitz-Mörder', desc='Kiebitz mit Federschopf über einem roten Kreuz, Schiefer, kein Fenster',
        rim=SLATE, frame=SLATE_D, ground='#9aa0a6', panel=LAPWING, panel_r=mirror(LAPWING),
        cols={'k': INK, 'w': WHITE, 'r': CRIMSON},
        band=(CHEV, {'w': '#9aa0a6'}), cross=(SLATE_D, CRIMSON),
        icon=LAPWING[1:23], icon_cols={'k': INK, 'w': WHITE, 'r': CRIMSON}),
    'zwillinge': dict(
        name='Zwillinge', desc='zwei gleiche Figuren gespiegelt, links hell auf dunkel, rechts dunkel auf hell',
        rim=GOLDW, frame=PETROL_D, ground=PETROL, panel=TWIN, panel_r=mirror(TWIN),
        cols={'k': INK, 's': '#f3c9a0', 'r': CRIMSON, 'd': CREAM, 'w': PETROL},
        band=(CHK, {'w': CREAM, 'k': PETROL_D}), cross=(PETROL_D, GOLDW),
        icon=TWIN[2:20], icon_cols={'k': INK, 's': '#f3c9a0', 'r': CRIMSON, 'd': GOLDW, 'w': PETROL}),
    'der_alte': dict(
        name='Der Alte (Endboss)', desc='Krone über Lorbeer und Siegelring, Gold auf Schwarz, goldener Rand',
        rim=GOLDW, frame=INK, ground='#141414', panel=CROWN, rimline=GOLDD,
        cols={'y': GOLDW, 'w': WHITE},
        band=(STRIPE, {'y': GOLDW, 'k': '#2a2a2a'}), cross=(GOLDW, INK),
        icon=CROWN[2:17], icon_cols={'y': GOLDW, 'w': WHITE}),
}
# Zwillinge: the right panels invert ground and figure colours
z = BOSSES['zwillinge']
z['ground_at'] = {(RIGHT, TOP): CREAM, (RIGHT, BOTTOM): CREAM}
z['cols_at'] = {(RIGHT, TOP): {'k': CREAM, 's': '#f3c9a0', 'r': CRIMSON, 'd': PETROL, 'w': CREAM},
                (RIGHT, BOTTOM): {'k': CREAM, 's': '#f3c9a0', 'r': CRIMSON, 'd': PETROL, 'w': CREAM}}


# ------------------------------------------------------------ characters (03.11.2026)
# Der Drueckeberger: a perforated stamp with a posthorn, a round postmark
# with wavy cancel lines, on kraft paper.
STAMP = ["..............",
         ".w.w.w.w.w.w..",
         ".wwwwwwwwwww..",
         "wwgggggggggww.",
         ".wggyyyygggw..",
         "wwgyggggyggww.",
         ".wgyggyyyyyw..",
         "wwgyggggyggww.",
         ".wggyyyygggw..",
         "wwgggggggggww.",
         ".wgwwwgwwwgw..",
         ".wwwwwwwwwww..",
         ".w.w.w.w.w.w..",
         "..............",
         "..kkkk........",
         ".k....k..kk...",
         "k.kkkk.k.k..kk",
         "k......k..kk..",
         "k.kkkk.k.k..kk",
         ".k....k...kk..",
         "..kkkk...k..kk",
         "..............",
         ".............."]
# Die Schneiderin: open scissors over a yellow tape measure and its case,
# on aubergine cloth with a seam.
SHEARS = ["..............",
          ".w.........w..",
          "..w.......w...",
          "...w.....w....",
          "....w...w.....",
          ".....w.w......",
          "......y.......",
          ".....r.r......",
          "....r...r.....",
          "..rrr...rrr...",
          ".r...r.r...r..",
          "..rrr...rrr...",
          "..............",
          "s.s.s.s.s.s.s.",
          "yyyyyyyyyyyyyy",
          "kykykykykykyky",
          "yyyyyyyyyyyyyy",
          "..............",
          "....oooooo....",
          "...oyyyyyyo...",
          "...oyykkyyo...",
          "...oyyyyyyo...",
          "....oooooo...."]
# Bruderherz: one big red heart.
HEART = ["..rrr...rrr..",
         ".rrrrr.rrrrr.",
         "rrwwrrrrrrrrr",
         "rrwrrrrrrrrrr",
         "rrrrrrrrrrrrr",
         ".rrrrrrrrrrr.",
         "..rrrrrrrrr..",
         "...rrrrrrr...",
         "....rrrrr....",
         ".....rrr.....",
         "......r......"]
HEART_RED = '#d02a2a'
HEART_D = '#7a1420'
BLUSH = '#e7b9b0'
KRAFT = '#c7b089'
KRAFT_D = '#6b5636'
AUBERGINE = '#4a3a5a'
AUBERGINE_D = '#2a1f36'

CHARACTERS = {
    'shirker': dict(
        name='Drückeberger', desc='Briefmarke mit Posthorn und Poststempel mit Wellenlinien auf Packpapier',
        rim=KRAFT, frame=KRAFT_D, ground=KRAFT, panel=STAMP, panel_r=mirror(STAMP),
        cols={'w': WHITE, 'g': PETROL, 'y': GOLDW, 'k': WINE_D},
        band=(DOTS, {'w': WINE_D}), cross=(KRAFT_D, GOLDW),
        icon=STAMP[0:13], icon_cols={'w': WHITE, 'g': PETROL, 'y': GOLDW}),
    'tailoress': dict(
        name='Schneiderin', desc='offene Schere über einem gelben Maßband und seiner Dose, Naht auf Aubergine',
        rim=GOLDW, frame=AUBERGINE_D, ground=AUBERGINE, panel=SHEARS, panel_r=mirror(SHEARS),
        cols={'w': GREY, 'y': AMBER, 'r': CRIMSON, 'k': INK, 'o': GOLDD, 's': CREAM},
        band=(STRIPE, {'y': AMBER, 'k': INK}), cross=(AUBERGINE_D, AMBER),
        icon=SHEARS[0:12], icon_cols={'w': WHITE, 'y': AMBER, 'r': CRIMSON}),
}


def heart_full():
    """Bruderherz: cream ground with a lattice of small hearts, one big heart."""
    out = rim(HEART_RED, HEART_D)
    out += f'<rect x="12" y="14" width="96" height="152" fill="{CREAM}"/>'
    small_heart = [".r.r.", "rrrrr", ".rrr.", "..r.."]
    for j in range(0, 8):
        for i in range(0, 6):
            if (i + j) % 2 == 0:
                out += grid(small_heart, 15 + i * 16, 18 + j * 19, {'r': BLUSH}, 2)
    out += grid(HEART, 60 - 13 * 3.5, 90 - 11 * 3.5, {'r': HEART_D}, 7)
    out += grid(HEART, 60 - 13 * 3, 90 - 11 * 3, {'r': HEART_RED, 'w': WHITE}, 6)
    return out


def heart_small():
    out = rim(HEART_RED, HEART_D)
    out += f'<rect x="14" y="16" width="92" height="148" fill="{CREAM}"/>'
    out += grid(HEART, 60 - 13 * 3, 90 - 11 * 3, {'r': HEART_RED, 'w': WHITE}, 6)
    return out


# ------------------------------------------------------------ emit
# Back ids as the content names them (core/content/bosses.ron, guests.ron).
IDS = {
    'black_peter': 'schwarzer_peter', 'grandma': 'oma', 'the_pusher': 'schieber', 'ramscher': 'ramscher',
    'kontra_hans': 'kontra_hans', 'revolutionary': 'revolutionaer', 'kibitz_killer': 'kibitz_killer',
    'twins': 'zwillinge', 'the_old_man': 'der_alte',
}


def ts():
    lines = ['// Generated by tools/cardart/backs.py (the card backs of the opponents and the',
             '// characters). Do not edit;',
             '// change the generator and run it again (tools/cardart/README.md).',
             '//',
             '// Inner SVG of each back in the card\'s 120 x 180 box: the full picture and',
             '// the small one for tiny backs (the opponents\' fans).',
             '',
             'export const BACK_ART: Record<string, { full: string; small: string }> = {',
             f'  regulars: {{ full: {stammgast_full()!r}, small: {stammgast_small()!r} }},']
    for back, key in IDS.items():
        b = BOSSES[key]
        lines.append(f'  {back}: {{ full: {family(b)!r}, small: {small(b)!r} }},')
    for back, b in CHARACTERS.items():
        lines.append(f'  {back}: {{ full: {family(b)!r}, small: {small(b)!r} }},')
    lines.append(f'  heart_brother: {{ full: {heart_full()!r}, small: {heart_small()!r} }},')
    lines += ['};', '']
    return '\n'.join(lines)


if __name__ == '__main__':
    import os
    import sys
    out = os.path.join(os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__)))),
                       'web', 'src', 'ui', 'backs.gen.ts')
    text = ts()
    if '--check' in sys.argv:
        with open(out, encoding='utf-8') as fh:
            if fh.read() != text:
                sys.exit(f'{out} is stale: run python3 tools/cardart/backs.py')
        print('up to date')
    else:
        with open(out, 'w', encoding='utf-8') as fh:
            fh.write(text)
        print(f'wrote {out}')

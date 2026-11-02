"""Effekte v3 in the Pixelstil: the card modifications of effects v2
(DESIGN 7.2) as approved on the board "Effekte v3" (02.11.2026).

Grammar: FRAME = what (ring 1-2), MARK TOP RIGHT = when (none = always),
BAND BOTTOM CENTRE = how much (a colour band of fixed height with white
glyphs). Curses are anthracite and have no corner sign. Rang has no frame,
only stacked chevrons in the band. Untot and Wandelkarte change the picture
(untot.py, wild.py) and have no frame, so a special card can carry one
normal modification on top.

Reference only: the web app draws the same pixels at run time in
web/src/ui/cards.ts, because frames, marks and bands combine per card and
the band text comes from the card's value."""

from fx import rects, shape, draw, halo, DIGITS
from fx2 import ring
from pix import GW, GH, BG, SIGN

GOLD, GOLD_L, GOLD_D = '#d98e0b', '#ffe08a', '#8a5a0a'
WINE, WINE_L = '#8e2430', '#d9a2a8'
PURPLE = '#6a3fa8'
TEAL = '#16707a'
PLAY, PLAY_L = '#d2552a', '#f6b48e'
ANTH, ANTH_L = '#33363b', '#9a9ea4'
STAMP, STAMP_L = '#2f7d6a', '#bfe3d6'
SUITC = {'Eichel': '#8b4a1c', 'Gruen': '#2e9a3c', 'Rot': '#e0281e', 'Schellen': '#f7c21a'}

# ------------------------------------------------------------ glyphs (5 high)
G = dict(DIGITS)
G.update({
    'coin': ['.###.', '##.##', '#.#.#', '##.##', '.###.'],
    'skull': ['.###.', '#####', '#.#.#', '#####', '.#.#.'],
    'wild': ['##.##', '##.##', '.....', '##.##', '##.##'],
    'step': ['....#', '...##', '..###', '.####', '#####'],
    'chev': ['.....', '..#..', '.#.#.', '#...#', '.....'],
    'play': ['#..', '##.', '###', '##.', '#..'],
    'rang1': ['...', '.#.', '#.#', '...', '...'],
    'rang2': ['.#.', '#.#', '...', '.#.', '#.#'],
    'rang3': ['...........', '.#...#...#.', '#.#.#.#.#.#', '...........', '...........'],
})


def glyph_cells(item, x, y):
    """item: '+4', ('chev', n), ('sign', suit), 'coin', 'skull', 'wild', 'step'.
    Returns (list of (cells, colour or None), width)."""
    if isinstance(item, tuple) and item[0] == 'chev':
        out = set()
        for k in range(item[1]):
            out |= shape(G['chev'], x + k * 6, y)
        return [(out, None)], item[1] * 6 - 1
    if isinstance(item, tuple) and item[0] == 'sign':
        rows = SIGN[item[1]]
        cells = {(x + i, y + j + (5 - len(rows)) // 2) for j, r in enumerate(rows) for i, c in enumerate(r) if c != '.'}
        return [(cells, None)], 3
    if item == 'wild':
        # rainbow diamond on a white tile with a black edge (reads on any band colour)
        tile = rects(f'{x - 1} {y - 1} 7 7')
        dia = [('..r..', '#e0281e'), ('.yyy.', '#f7c21a'), ('ggggg', '#2e9a3c'), ('.bbb.', '#2a63c4'), ('..o..', '#8b4a1c')]
        out = [(tile - rects(f'{x} {y} 5 5'), '#1c1a1a'), (rects(f'{x} {y} 5 5'), '#ffffff')]
        for j, (row, c) in enumerate(dia):
            out.append(({(x + i, y + j) for i, ch in enumerate(row) if ch != '.'}, c))
        return out, 5
    if item == 'eye':
        return [(shape(['..###..', '.#...#.', '#..#..#', '.#...#.', '..###..'], x, y), None)], 7
    if item == 'coin':
        return [(shape(G['coin'], x, y), '#f7c21a'), ({(x + 2, y + 2)}, '#8a5a0a')], 5
    if item in G:
        return [(shape(G[item], x, y), None)], len(G[item][0])
    # a number string
    parts, w = [], 0
    cells = set()
    for ch in item:
        g = G[ch]
        cells |= shape(g, x + w, y)
        w += len(g[0]) + 1
    return [(cells, None)], w - 1


def band(items, col, p, fg='#ffffff', y=41, x_min=3):
    """A colour band bottom centre with stepped ends; glyphs in white."""
    widths = [glyph_cells(it, 0, 0)[1] for it in items]
    cw = sum(widths) + 2 * (len(items) - 1)
    pad = max(0, 7 - cw)            # every band is at least as wide as "+3"
    x0 = max(x_min + 2, min((GW - cw - pad) // 2, 23 - cw - pad))
    bx0, bx1 = x0 - 2, x0 + cw + pad + 1
    x0 += pad // 2
    body = rects(f'{bx0} {y} {bx1 - bx0 + 1} 8') | rects(f'{bx0 - 1} {y + 1} 1 6') | rects(f'{bx1 + 1} {y + 1} 1 6')
    out = draw(halo(body) - body, p) + draw(body, col) + draw(rects(f'{bx0} {y + 7} {bx1 - bx0 + 1} 1'), shade(col))
    x = x0
    for it, w in zip(items, widths):
        for cells, c in glyph_cells(it, x, y + 1 + 1)[0]:
            out += draw(cells, c or fg)
        x += w + 2
    return out


def shade(c):
    r, g, b = (int(c[i:i + 2], 16) for i in (1, 3, 5))
    return '#%02x%02x%02x' % (int(r * .62), int(g * .62), int(b * .62))


# ------------------------------------------------------------ frames
def fr_gold(c=GOLD, l=GOLD_L):
    return draw(ring(1), c) + draw(ring(2), l)


def fr_dashed(col):
    return draw({(x, y) for x, y in ring(1) if ((x + y) // 2) % 2 == 0}, col)


def fr_stamp_v2():
    """Gutschein V2: a light stamp margin with a dashed tear line and
    white perforation dots, like a sheet of stamps."""
    margin = ring(0, False) | ring(1, False)
    dots = {(x, y) for x, y in ring(0, False) if (x + y) % 2 == 0}
    tear = {(x, y) for x, y in ring(2) if ((x + y) // 2) % 2 == 0}
    return draw(margin, STAMP_L) + draw(dots, '#ffffff') + draw(tear, STAMP)


# ------------------------------------------------------------ marks top right (when)
FINISH = ['#.#.#', '.#.#.', '#.#.#', '.#.#.']
PLAYM = ['#..', '##.', '###', '##.', '#..']


def mark(col, sign, light='#ffffff'):
    cells = set()
    for y in range(10):
        cells |= {(x, y) for x in range(23 + y, GW)}
    x = 28 if len(sign[0]) == 5 else 29
    return draw(halo(cells) - cells, BG) + draw(cells, col) + draw(shape(sign, x, 1), light)


SEE = f'<rect width="{GW}" height="{GH + 0.5}" fill="{BG}" opacity=".55"></rect>'


def torn_deep():
    """Irregular tears of varied depth (1 to 4 pixels) into all four edges."""
    bites = set()
    rims = set()
    spec = [('l', 16, 3, 2), ('l', 29, 4, 3), ('r', 19, 2, 2), ('r', 30, 4, 2), ('t', 13, 3, 3), ('t', 22, 1, 2),
            ('b', 15, 4, 3), ('b', 8, 2, 2), ('l', 9 + 13, 1, 1)]
    for sd, pos, depth, width in spec:
        for d in range(depth):
            w = max(1, width - d // 2)
            for k in range(-w // 2, w - w // 2):
                if sd == 'l':
                    c = (d, pos + k + (d % 2))
                elif sd == 'r':
                    c = (GW - 1 - d, pos + k - (d % 2))
                elif sd == 't':
                    c = (pos + k + (d % 2), d)
                else:
                    c = (pos + k - (d % 2), GH - 1 - d)
                bites.add(c)
    for x, y in bites:
        for dx, dy in ((1, 0), (-1, 0), (0, 1), (0, -1)):
            q = (x + dx, y + dy)
            if q not in bites and 0 <= q[0] < GW and 0 <= q[1] < GH:
                rims.add(q)
    return draw(bites, '#1b1e21') + draw(rims, '#8a7a60')


# ------------------------------------------------------------ the effects
def augen(v):
    if v.startswith('-'):
        return c_ramponiert(v)
    return lambda p, s: fr_gold() + band([v], GOLD, p)


def rang(n, v=1):
    """Rang: no frame; chevrons inside the standard band (same height as all bands)."""
    return lambda p, s: band(['rang%d' % n], WINE, p)


def trumpf(v='+3'):
    return lambda p, s: fr_dashed(PURPLE) + band([v], PURPLE, p)


def letzter():
    return lambda p, s: fr_dashed(TEAL) + mark(TEAL, FINISH) + band(['+1', 'step'], TEAL, p)


def erster():
    return lambda p, s: fr_gold(PLAY, PLAY_L) + mark(PLAY, PLAYM) + band(['x2'], PLAY, p)


def gutschein(kind='coin'):
    fr = fr_stamp_v2
    items = {'coin': ['coin', '+3'], 'skull': ['skull'], 'wild': ['wild']}[kind]
    return lambda p, s: fr() + band(items, STAMP, p)


def c_durchsichtig():
    def f(p, s):
        return SEE + band(['eye'], ANTH, p)
    return f


def c_ramponiert(v='-2'):
    return lambda p, s: fr_gold(ANTH, ANTH_L) + torn_deep() + band([v], ANTH, p)


def c_fleckig(v='-3'):
    def f(p, s):
        blobs = rects('17 13 6 2, 16 15 8 3, 17 18 6 2') | rects('7 27 4 2, 6 29 6 2, 7 31 3 1') | rects('20 31 5 2, 19 33 7 3')
        return (fr_dashed(ANTH) + draw(blobs, '#8a5a22', 0.82) + draw(rects('18 14 2 1, 8 28 1 1, 21 32 2 1'), '#5e3a12', 0.85)
                + band([v], ANTH, p))
    return f

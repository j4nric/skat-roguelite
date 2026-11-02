"""Proposals for three card modifications in the Pixelstil (canvas only).

A card face is built like the web app does (web/src/ui/cards.ts): paper,
picture, washes ("under"), corner index, marks ("over") with a one-pixel
paper halo. Grid 33 x 49.5, one art pixel = 2 x 2 CSS px at 66 x 99."""

from pix import PAL, BG, GW, GH, GLYPH, SIGN, INK

W, H = GW, GH + 0.5
CURSE_RED = '#bd3b32'

# ------------------------------------------------------------ cells


def rects(spec):
    out = set()
    for r in spec.split(','):
        x, y, w, h = (int(v) for v in r.split())
        for j in range(h):
            for i in range(w):
                out.add((x + i, y + j))
    return out


def shape(rows, x, y, ch='#'):
    return {(x + i, y + j) for j, row in enumerate(rows) for i, c in enumerate(row) if c == ch}


def scale(cells, k, x0=0, y0=0):
    return {(x0 + x * k + i, y0 + y * k + j) for x, y in cells for i in range(k) for j in range(k)}


def draw(cells, fill, opacity=None):
    rows = {}
    for x, y in cells:
        if 0 <= x < GW and 0 <= y < GH:
            rows.setdefault(y, []).append(x)
    d = []
    for y in sorted(rows):
        xs = sorted(rows[y])
        i = 0
        while i < len(xs):
            k = i
            while k + 1 < len(xs) and xs[k + 1] == xs[k] + 1:
                k += 1
            n = k - i + 1
            d.append(f'M{xs[i]} {y}h{n}v1h-{n}z')
            i = k + 1
    op = f' opacity="{opacity}"' if opacity is not None else ''
    return f'<path d="{"".join(d)}" fill="{fill}"{op}></path>' if d else ''


def halo(cells):
    ring = set()
    for x, y in cells:
        for dy in (-1, 0, 1):
            for dx in (-1, 0, 1):
                ring.add((x + dx, y + dy))
    return ring


def marked(parts, paper=BG):
    allc = set()
    for cells, _ in parts:
        allc |= cells
    return draw(halo(allc), paper) + ''.join(draw(c, f) for c, f in parts)


DIGITS = {
    '0': ['###', '#.#', '#.#', '#.#', '###'], '1': ['#', '#', '#', '#', '#'],
    '2': ['###', '..#', '###', '#..', '###'], '3': ['###', '..#', '###', '..#', '###'],
    '4': ['#.#', '#.#', '###', '..#', '..#'], '5': ['###', '#..', '###', '..#', '###'],
    '6': ['###', '#..', '###', '#.#', '###'], '7': ['###', '..#', '.#.', '.#.', '.#.'],
    '8': ['###', '#.#', '###', '#.#', '###'], '9': ['###', '#.#', '###', '..#', '###'],
    '+': ['...', '.#.', '###', '.#.', '...'], '-': ['...', '...', '###', '...', '...'],
    'x': ['...', '#.#', '.#.', '#.#', '...'],
}


def number_cells(text, x, y):
    cells = set()
    for ch in text:
        g = DIGITS[ch]
        cells |= shape(g, x, y)
        x += len(g[0]) + 1
    return cells, x - 1


def number_width(text):
    return sum(len(DIGITS[c][0]) for c in text) + len(text) - 1


CURSE_FRAME = draw(rects('0 0 6 1, 8 0 2 1, 12 0 9 1, 23 0 2 1, 27 0 6 1, 0 1 1 6, 0 9 1 2, 0 13 1 8, 0 23 1 3, '
                         '0 28 1 8, 0 38 1 2, 0 42 1 7, 32 1 1 6, 32 9 1 2, 32 13 1 8, 32 23 1 3, 32 28 1 8, 32 38 1 2, '
                         '32 42 1 7, 0 48 6 1, 8 48 2 1, 12 48 9 1, 23 48 2 1, 27 48 6 1'), CURSE_RED)


# ------------------------------------------------------------ base faces

def split_grid(g):
    """Picture cells per colour, and the corner index cells (drawn on top)."""
    import numpy as np
    pic, idx = {}, {}
    rank, suit = g.rank, g.suit
    from pix import Grid
    ig = Grid()
    ig.index(rank, suit)
    for y in range(GH):
        for x in range(GW):
            c = g.g[y, x]
            if c == '.':
                continue
            if ig.g[y, x] != '.':
                idx.setdefault(PAL[c], set()).add((x, y))
            else:
                pic.setdefault(PAL[c], set()).add((x, y))
    return pic, idx


def face_svg(g, mod=None, w=66, h=99, label=''):
    """mod: dict with optional keys paper, under(paper), over(paper),
    outline (CSS colour around the card), curse (bool)."""
    mod = mod or {}
    paper = mod.get('paper', BG)
    pic, idx = split_grid(g)
    body = f'<rect x="0" y="0" width="{W}" height="{H}" fill="{paper}"></rect>'
    body += ''.join(draw(c, col) for col, c in pic.items())
    if 'under' in mod:
        body += mod['under'](paper)
    if 'over' in mod:
        body += mod['over'](paper)
    # the corner index stays free and on top of every mark
    body += ''.join(draw(c, col) for col, c in idx.items())
    if mod.get('curse'):
        body += CURSE_FRAME
    return (f'<svg viewBox="0 0 {W} {H}" width="{w}" height="{h}" shape-rendering="crispEdges" role="img" '
            f'aria-label="{label}" style="display: block">' + body + '</svg>')


# ------------------------------------------------------------ A: Augen +4 (fat)
GOLD, GOLD_D, GOLD_L = '#d98e0b', '#8a5a0a', '#ffe08a'
INKC = '#1c1a1a'


def num_centre(text, y, fill, paper, extra=()):
    wdt = number_width(text)
    x = (GW - wdt) // 2
    cells, _ = number_cells(text, x, y)
    return marked([(cells, fill)] + list(extra), paper)


def a1(p):
    """A1 Augen-Muenze: a gold coin plaque at the bottom with +4 in black."""
    coin = rects('11 41 11 7') - rects('11 41 1 1, 21 41 1 1, 11 47 1 1, 21 47 1 1')
    rim = coin - rects('12 42 9 5')
    cells, _ = number_cells('+4', 13, 42)
    return marked([(coin, GOLD), (rim, GOLD_D)], p) + draw(cells, INKC)


def a2(p):
    """A2 Goldrahmen innen: a stepped 2-pixel gold frame inside the edge, +4 in gold."""
    fr = rects(f'1 1 {GW - 2} 1, 1 1 1 {GH - 2}, {GW - 2} 1 1 {GH - 2}, 1 {GH - 2} {GW - 2} 1')
    fr2 = rects(f'2 2 {GW - 4} 1, 2 2 1 {GH - 4}, {GW - 3} 2 1 {GH - 4}, 2 {GH - 3} {GW - 4} 1')
    fr -= rects('1 1 6 12, 26 36 6 12')
    fr2 -= rects('1 1 6 12, 26 36 6 12')
    return draw(fr, GOLD) + draw(fr2, GOLD_L) + num_centre('+4', 42, GOLD_D, p)


def a3(p):
    """A3 Augenpunkte: four gold diamonds in a row, one per Auge, no number."""
    d = ['.#.', '###', '.#.']
    cells = set()
    for k in range(4):
        cells |= shape(d, 8 + k * 4, 43)
    return marked([(cells, GOLD), (shape(['.#.'], 8, 43) | shape(['.#.'], 12, 43) | shape(['.#.'], 16, 43) |
                                    shape(['.#.'], 20, 43), GOLD_L)], p)


def a4(p):
    """A4 Augen-Banner: a gold ribbon across the bottom, +4 in white."""
    rib = rects('8 42 17 7') | rects('6 43 2 5, 25 43 2 5') | rects('5 44 1 3, 27 44 1 3')
    rib -= rects('5 45 1 1, 27 45 1 1')
    dark = rects('8 48 17 1')
    cells, _ = number_cells('+4', 13, 43)
    return marked([(rib, GOLD), (dark, GOLD_D)], p) + draw(cells, '#ffffff')


def a5(p):
    """A5 Glanz: four gold sparkles on the card and a small +4."""
    sp = ['.#.', '###', '.#.']
    cells = set()
    for x, y in ((8, 14), (25, 13), (6, 32), (24, 30)):
        cells |= shape(sp, x, y)
        cells |= {(x + 1, y - 1), (x + 1, y + 3), (x - 1, y + 1), (x + 3, y + 1)}
    return marked([(cells, GOLD)], p) + num_centre('+4', 42, GOLD_D, p)


A = [('A1', 'Augen-Münze: goldene Plakette unten mit „+4“', {'over': a1}),
     ('A2', 'Goldrahmen in der Karte, „+4“ unten in Gold', {'over': a2}),
     ('A3', 'Augenpunkte: vier goldene Rauten unten, eine je Auge', {'over': a3}),
     ('A4', 'Augen-Banner: goldenes Band unten, „+4“ weiß', {'over': a4}),
     ('A5', 'Glanz: vier Funkel über dem Bild, „+4“ unten', {'over': a5})]

# ------------------------------------------------------------ B: Trumpf, Grundwert +3 (stamp)
PURPLE, PURPLE_L, PURPLE_D = '#6a3fa8', '#cbb6ea', '#3c2266'


def suit_cells(suit, x, y):
    rows = SIGN[suit]
    return {(x + i, y + j) for j, r in enumerate(rows) for i, c in enumerate(r) if c != '.'}


def b1(suit):
    def f(p):
        """B1 Stempelring: a purple ink ring at the bottom with the suit sign and +3."""
        ring = rects('9 41 15 1, 9 48 15 1, 8 42 1 6, 24 42 1 6')
        sig = suit_cells(suit, 10, 43)
        cells, _ = number_cells('+3', 16, 43)
        return marked([(ring, PURPLE), (sig, PURPLE), (cells, PURPLE)], p)
    return f


def b2(suit):
    def f(p):
        """B2 Lesezeichen: a purple bookmark from the top edge, suit sign in it, +3 below."""
        bm = rects('25 0 5 14') - {(27, 13)} - {(25, 13), (29, 13)}
        bm -= {(26, 13), (28, 13)}
        sig = suit_cells(suit, 26, 5)
        return (draw(bm, PURPLE) + draw(sig, '#ffffff') + num_centre('+3', 42, PURPLE, p))
    return f


def b3(suit):
    def f(p):
        """B3 Trumpf-Band: a purple band across the bottom with suit sign and +3 in white."""
        band = rects('6 41 21 8')
        sig = suit_cells(suit, 9, 43)
        cells, _ = number_cells('+3', 16, 43)
        return draw(band, PURPLE) + draw(rects('6 41 21 1'), PURPLE_D) + draw(sig, '#ffffff') + draw(cells, '#ffffff')
    return f


def b4(suit):
    def f(p):
        """B4 Briefmarke: a perforated purple stamp edge inside the card, +3 below."""
        e = set()
        for x in range(1, GW - 1):
            if x % 2:
                e.add((x, 1))
                e.add((x, GH - 2))
        for y in range(1, GH - 1):
            if y % 2:
                e.add((1, y))
                e.add((GW - 2, y))
        e -= rects('0 0 7 13, 26 35 7 14')
        return draw(e, PURPLE) + num_centre('+3', 42, PURPLE, p)
    return f


def b5(suit):
    def f(p):
        """B5 Grundwert-Pfeil: a purple up arrow over three steps beside +3."""
        arrow = shape(['..#..', '.###.', '#.#.#', '..#..', '..#..'], 8, 42)
        cells, _ = number_cells('+3', 15, 42)
        sig = suit_cells(suit, 23, 43)
        return marked([(arrow, PURPLE), (cells, PURPLE), (sig, PURPLE)], p)
    return f


B = [('B1', 'Stempelring: lila Stempel unten mit Farbzeichen und „+3“', lambda su, f=b1: {'over': f(su)}),
     ('B2', 'Lesezeichen: lila Band von oben mit Farbzeichen, „+3“ unten', lambda su, f=b2: {'over': f(su)}),
     ('B3', 'Trumpf-Band: lila Streifen unten, Farbzeichen und „+3“ weiß', lambda su, f=b3: {'over': f(su)}),
     ('B4', 'Briefmarke: lila Zähnung am Rand, „+3“ unten', lambda su, f=b4: {'over': f(su)}),
     ('B5', 'Pfeil nach oben, „+3“ und Farbzeichen unten in Lila', lambda su, f=b5: {'over': f(su)})]

# ------------------------------------------------------------ C: Schwarze Katze (curse)
CAT, CAT_EYE = '#1d1a18', '#f7c21a'


def c1(p):
    """C1 Katze sitzt unten links und schaut ueber den Rand."""
    cat = shape(['#...#.....',
                 '##.##.....',
                 '#####.....',
                 '#.#.#.....',
                 '#####...#.',
                 '.###...#..',
                 '#####..#..',
                 '######.#..',
                 '#######...'], 1, 40)
    eyes = {(2, 43), (4, 43)}
    return marked([(cat, CAT)], p) + draw(eyes, CAT_EYE) + num_centre('-3', 42, CURSE_RED, p)


def c2(p):
    """C2 Katzenaugen: two yellow eyes in a dark wash over the picture."""
    wash = rects('0 0 33 49')
    eyes = shape(['.###.......###.',
                  '##.##.....##.##',
                  '.###.......###.'], 9, 20)
    pup = {(11, 21), (21, 21)}
    return draw(wash, '#2a2522', 0.22) + draw(eyes, CAT_EYE) + draw(pup, CAT) + num_centre('-3', 42, CURSE_RED, p)


def c3(p):
    """C3 Katze laeuft quer ueber den unteren Rand."""
    cat = shape(['#.#.................',
                 '###...............#.',
                 '#.##..............#.',
                 '############.....#..',
                 '.###########....#...',
                 '.############..#....',
                 '..##########.##.....',
                 '..#.#....#.#........',
                 '..#.#....#.#........'], 4, 38)
    return marked([(cat, CAT)], p) + draw({(5, 40)}, CAT_EYE)


def c4(p):
    """C4 Pfotenspur quer ueber die Karte, -3 unten."""
    paw = ['#.#.#', '.....', '.###.', '.###.']
    cells = set()
    for k, (x, y) in enumerate(((5, 33), (11, 26), (15, 19), (21, 13))):
        cells |= shape(paw, x, y)
    return marked([(cells, CAT)], p) + num_centre('-3', 42, CURSE_RED, p)


def c5(p):
    """C5 Katzenkopf: a big faint black cat head over the picture, -3 below."""
    head = shape(['#...#', '##.##', '#####', '#.#.#', '#####', '.###.'], 0, 0)
    big = scale(head, 4, 6, 12)
    eyes = scale({(1, 3), (3, 3)}, 4, 6, 12)
    return draw(big - eyes, '#1d1a18', 0.3) + draw(eyes, CAT_EYE, 0.55) + num_centre('-3', 42, CURSE_RED, p)


C = [('C1', 'Katze sitzt unten links und schaut über den Rand, „−3“', {'over': c1, 'curse': True}),
     ('C2', 'Katzenaugen: gelbe Augen über abgedunkeltem Bild, „−3“', {'over': c2, 'curse': True}),
     ('C3', 'Katze läuft quer über den unteren Rand (ohne Zahl)', {'over': c3, 'curse': True}),
     ('C4', 'Pfotenspur quer über die Karte, „−3“', {'over': c4, 'curse': True}),
     ('C5', 'Katzenkopf als großes, blasses Wasserzeichen über dem Bild, „−3“', {'over': c5, 'curse': True})]

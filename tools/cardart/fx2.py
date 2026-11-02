"""Frame language for all card modifications (proposal, canvas only).

Rings counted from the card edge on the 33 x 49 art grid:
  ring 0  (outermost pixel)  state:  Fluch (red, broken)  /  Untot (green, ragged)
  ring 1-2                   family: Augen (gold double), Trumpf (perforation),
                                     Sicht (dashed), Glas (light blue + glint),
                                     Wandel (four suit colours), Ehrenunter (blue
                                     with dots), Heimweh (wine stitches)
  ring 3                     a second family frame, as a single line
Inner rings stop at the two index corners (top left, bottom right).
Positional marks: Rang chevrons under the top index, Letzter-Stich flag top
right, corner sign bottom left (5 x 5), printed number bottom centre."""

from fx import (rects, shape, scale, draw, halo, marked, number_cells, number_width, face_svg, CURSE_FRAME, CURSE_RED,
                c1 as cat_c1)
from pix import GW, GH, BG

TL = (7, 13)      # index notch top left: x < 7 and y < 13
BR = (26, 36)     # index notch bottom right: x >= 26 and y >= 36


def ring(k, notch=True):
    out = set()
    for x in range(k, GW - k):
        out.add((x, k))
        out.add((x, GH - 1 - k))
    for y in range(k, GH - k):
        out.add((k, y))
        out.add((GW - 1 - k, y))
    if notch:
        out = {(x, y) for x, y in out if not (x < TL[0] and y < TL[1]) and not (x >= BR[0] and y >= BR[1])}
    return out


def side(cells, which):
    """Filter ring cells to a side: 't', 'b', 'l', 'r'."""
    f = {'t': lambda x, y: y <= 3, 'b': lambda x, y: y >= GH - 4, 'l': lambda x, y: x <= 3 and 3 < y < GH - 4,
         'r': lambda x, y: x >= GW - 4 and 3 < y < GH - 4}[which]
    return {c for c in cells if f(*c)}


# ------------------------------------------------------------ frames by family
def f_augen(col, lt, k=1):
    """Augen: a double gold frame (outer colour, inner light)."""
    if k == 1:
        return draw(ring(1), col) + draw(ring(2), lt)
    return draw(ring(k), col)


def f_trumpf(col, k=1):
    """Trumpf: the perforated edge of a postage stamp."""
    cells = {(x, y) for x, y in ring(k) if (x + y) % 2 == 0}
    return draw(cells, col)


def f_sicht(col, k=1):
    """Sicht: a dashed line, 2 on 2 off."""
    cells = {(x, y) for x, y in ring(k) if ((x + y) // 2) % 2 == 0}
    return draw(cells, col)


def f_glas(k=1):
    out = draw(ring(k), '#7fc4e8')
    if k == 1:
        out += draw(ring(2), '#d6eefa')
        glint = {(5 + i, 35 - i) for i in range(12)} | {(8 + i, 37 - i) for i in range(8)}
        out += draw(glint, '#ffffff')
    return out


def f_wandel(k=1):
    r = ring(k)
    return (draw(side(r, 't') | {c for c in r if c[1] < GH // 2 and c[0] <= 3}, '#8b4a1c') +
            draw({c for c in r if c[1] < GH // 2 and c[0] >= GW - 4}, '#2e9a3c') +
            draw({c for c in r if c[1] >= GH // 2 and c[0] <= 3} | side(r, 'b'), '#e0281e') +
            draw({c for c in r if c[1] >= GH // 2 and c[0] >= GW - 4}, '#f7c21a'))


def f_ehren(k=1):
    out = draw(ring(k), '#1f5a8c')
    if k == 1:
        out += draw({(x, y) for x, y in ring(2) if (x + y) % 3 == 0}, '#1f5a8c')
    return out


def f_heim(k=1):
    return draw({(x, y) for x, y in ring(k) if (x + y) % 3 != 2}, '#8e2430')


def f_untot():
    """Untot: a ragged zombie-green edge on ring 0, with drips inward."""
    r = ring(0, notch=False)
    cells = {(x, y) for x, y in r if (x * 7 + y * 3) % 5 != 0}
    drips = set()
    for x, y in r:
        if (x * 5 + y * 11) % 13 == 0:
            if y == 0:
                drips |= {(x, 1), (x, 2)}
            elif x == 0:
                drips.add((1, y))
            elif x == GW - 1:
                drips.add((GW - 2, y))
    return draw(cells, '#5e7a3c') + draw(drips, '#9cb86e')


def f_curse_untot():
    """Fluch and Untot on one card: ring 0 alternates red and green segments."""
    r = ring(0, notch=False)
    red = {(x, y) for x, y in r if ((x + y) // 4) % 2 == 0}
    return draw(red, CURSE_RED) + draw(r - red, '#5e7a3c')


# ------------------------------------------------------------ marks
CHEVRON = ['...#...', '..###..', '.##.##.', '##...##']


def chevrons(n, col, p):
    return marked([(shape(CHEVRON, 0, 13 + i * 5), col) for i in range(n)], p)


def flag(col, sign):
    cells = set()
    for y in range(13):
        cells |= {(x, y) for x in range(20 + y, GW)}
    return draw(cells, col) + draw(shape(sign, 27, 1), '#ffffff')


def corner(rows, col, p):
    return marked([(shape(rows, 2, 42), col)], p)


def number(text, col, p, x=None):
    wdt = number_width(text)
    x = (GW - wdt) // 2 if x is None else x
    cells, _ = number_cells(text, x, 42)
    return marked([(cells, col)], p)


def numbers2(t1, c1, t2, c2, p):
    """Two printed numbers side by side, centred as a pair."""
    w1, w2 = number_width(t1), number_width(t2)
    x = (GW - (w1 + w2 + 3)) // 2
    return number(t1, c1, p, x) + number(t2, c2, p, x + w1 + 3)


VEIL = f'<rect width="{GW}" height="{GH + 0.5}" fill="{BG}" opacity=".7"></rect>'
EYE = lambda p: marked([(shape(['.###.', '#####', '.###.'], 2, 43), '#2f86c0'), ({(4, 44)}, '#0b0c0e')], p)  # noqa: E731
SKULL = ['.###.', '#####', '#.#.#', '#####', '.#.#.']

GOLD, GOLD_L, GOLD_D = '#d98e0b', '#ffe08a', '#8a5a0a'
PURPLE = '#6a3fa8'

# id -> (family, layers(p), description); layers draw frames + marks; numbers bottom centre
MODS = {
    'fat': ('Augen', lambda p: f_augen(GOLD, GOLD_L) + number('+4', GOLD_D, p), 'Goldrahmen doppelt, „+4“ in Gold'),
    'gold_rim': ('Augen', lambda p: f_augen('#f7c21a', '#fff3b0') + draw(ring(3), '#f7c21a') + number('+8', GOLD_D, p),
                 'Goldrahmen dreifach (hell), „+8“'),
    'lead_weight': ('Augen', lambda p: f_augen('#4a4f57', '#a9adb3') + number('+10', '#4a4f57', p),
                    'Bleirahmen doppelt (grau), „+10“'),
    'marked': ('Rang', lambda p: chevrons(1, '#8e2430', p), 'kein Rahmen: ein Winkel unter dem Eckzeichen'),
    'double_marked': ('Rang', lambda p: chevrons(2, '#8e2430', p), 'kein Rahmen: zwei Winkel'),
    'dressed_up': ('Rang', lambda p: chevrons(2, '#b0336e', p) + corner(['#...#', '##.##', '#####', '##.##', '#...#'],
                                                                         '#b0336e', p),
                   'zwei Winkel in Pink, Schleife unten links'),
    'stamp': ('Trumpf', lambda p: f_trumpf(PURPLE) + number('+3', PURPLE, p), 'Zähnung lila, „+3“'),
    'crown': ('Trumpf', lambda p: f_trumpf('#c08a12') + corner(['#.#.#', '#####', '#####', '.....', '.....'], '#c08a12', p)
              + number('x2', '#c08a12', p), 'Zähnung ocker, Krone unten links, „×2“'),
    'seal': ('Trumpf', lambda p: f_trumpf('#a3263a') + corner(['.###.', '##.##', '#...#', '##.##', '.###.'], '#a3263a', p)
             + number('+1', '#a3263a', p), 'Zähnung weinrot, Siegel unten links, „+1“ Stufe'),
    'horseshoe': ('Trumpf', lambda p: f_trumpf('#4f6272') + corner(['#...#', '#...#', '#...#', '.###.', '.....'], '#4f6272', p)
                  + number('56', '#4f6272', p), 'Zähnung schiefer, Hufeisen, „56“'),
    'tape_measure': ('Trumpf', lambda p: f_trumpf('#b07d0c') + corner(['#####', '#.#.#', '#####', '.....', '.....'], '#b07d0c', p)
                     + number('80', '#b07d0c', p), 'Zähnung ocker, Maßband, „80“'),
    'last_trick': ('Letzter Stich', lambda p: flag('#16707a', ['#....', '###..', '####.', '#....', '#....']),
                   'kein Rahmen: Wimpel oben rechts'),
    'lucky_penny': ('Letzter Stich', lambda p: flag('#b06a2c', ['.###.', '#.#.#', '#.#.#', '#.#.#', '.###.']),
                    'Wimpel kupfer mit Pfennig'),
    'tail_light': ('Letzter Stich', lambda p: flag('#b3202a', ['.###.', '#####', '#####', '.###.', '#.#.#']),
                   'Wimpel rot mit Lampe'),
    'transparent': ('Sicht', lambda p: f_sicht('#2f86c0') + EYE(p), 'gestrichelt blau, Auge unten links'),
    'invisible': ('Sicht', lambda p: VEIL + f_sicht('#7a7f8a'), 'gestrichelt grau, Bild verblasst'),
    'dummy': ('Sicht', lambda p: VEIL + f_sicht('#7a7f8a') + EYE(p) + draw(scale(shape(
        ['.###.', '#...#', '...#.', '..#..', '..#..', '.....', '..#..'], 0, 0), 2, 12, 17), '#7d6a50'),
        'gestrichelt grau, verblasst, Fragezeichen, Auge'),
    'honorary_jack': ('Ehrenunter', lambda p: f_ehren() + corner(['#...#', '.#.#.', '.###.', '.###.', '..#..'], '#1f5a8c', p),
                      'blauer Rahmen mit Punktlinie, Unter-Zeichen'),
    'wild_card': ('Wandel', lambda p: f_wandel() + corner(['##.##', '##.##', '.....', '##.##', '##.##'], '#2e7d4f', p),
                  'Rahmen in den vier Farben, Viererzeichen'),
    'glass_ace': ('Glas', lambda p: f_glas() + number('x2', '#2f86c0', p), 'Glasrahmen hellblau mit Glanz, „×2“'),
    'homesick': ('Heimweh', lambda p: f_heim() + corner(['..#..', '.###.', '#####', '.#.#.', '.###.'], '#8e2430', p),
                 'Steppstich weinrot, Haus unten links'),
    'beer_stain': ('Fluch', lambda p: marked([(rects('19 15 6 2, 18 17 9 3, 19 20 7 2, 21 22 3 1'), '#c98a2a')], p)
                   + number('-5', CURSE_RED, p) + CURSE_FRAME, 'roter Bruchrahmen, Fleck, „−5“'),
    'cracked': ('Fluch', lambda p: marked([(rects('13 0 1 5, 14 5 1 3, 12 8 2 1, 11 9 1 5, 12 14 1 1, 13 15 1 4, 14 19 1 4, '
                                                  '13 23 1 3, 12 26 1 5, 13 31 2 1, 15 32 1 5, 14 37 1 4, 13 41 1 8'),
                                           '#1d1a18')], p) + CURSE_FRAME, 'roter Bruchrahmen, Riss'),
    'black_cat': ('Fluch', lambda p: cat_c1(p) + CURSE_FRAME, 'roter Bruchrahmen, Katze unten links, „−3“'),
    'battered': ('Fluch', lambda p: draw(rects('0 44 1 1, 0 45 2 1, 0 46 3 1, 0 47 4 1, 0 48 5 1'), '#1b1e21')
                 + draw(rects('30 22 3 1, 29 23 4 1, 30 24 3 1'), '#1b1e21') + CURSE_FRAME,
                 'roter Bruchrahmen, Ecke abgerissen, Kerbe'),
    'untot': ('Untot', lambda p: f_untot() + marked([(shape(SKULL, 2, 42), '#3c5226')], p),
              'zerfetzter grüner Rand, Totenkopf unten links'),
}

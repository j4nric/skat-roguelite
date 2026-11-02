"""The four Daus in the flat pixel style. Each is built around the shape
linked to its suit, with the suit sign (twice the pip size) on it:

  Eichel    Dreieck: a triangular shield, point down, under a barred helm
            crested with antlers; brown and green mantling with leaf tips;
            crossed boar spears
  Gruen     Quadrat: a square shield under a helm with a plume of three
            feathers; crossed lances with pennants; green-gold mantling;
            a rosette below
  Rot       Herz: a heart-shaped shield under a large crown; a red mantle
            lined with ermine spreading like a pavilion; crossed sceptres;
            a gold banner below
  Schellen  Kreis: a round blue medallion with a gold rim, a sunburst of
            gold and red rays, a crown on top, crossed maces, coins
"""

import math
from pix import Grid, BIG, GW, GH


def mask_fn(fn):
    return {(x, y) for y in range(GH) for x in range(GW) if fn(x, y)}


def paint(g, pts, col):
    for x, y in pts:
        g.px(x, y, col)


def border(pts):
    """Pixels of a set whose 4-neighbourhood leaves the set."""
    return {(x, y) for x, y in pts if any((x + dx, y + dy) not in pts for dx, dy in ((1, 0), (-1, 0), (0, 1), (0, -1)))}


def line(g, x0, y0, x1, y1, col, w=1):
    n = max(abs(x1 - x0), abs(y1 - y0))
    for i in range(n + 1):
        x = round(x0 + (x1 - x0) * i / n)
        y = round(y0 + (y1 - y0) * i / n)
        for k in range(w):
            g.px(x + k, y, col)


def emblem(g, inside, ring=('y', 'k'), field='w'):
    """Shield of a given shape: gold rim, black step, field."""
    outer = border(inside)
    inner = border(inside - outer)
    paint(g, inside, field)
    paint(g, inner, ring[1])
    paint(g, outer, ring[0])


def mantle(g, top, bot, col, lin, cut=None, x0=4, x1=28, ermine=False, grow=2, rimc=('y', 'k')):
    """Mantling as a stepped cape behind the emblem, in the same invisible
    frame language as the figures: stair chamfers with a gold rim, the
    cloth in bold chevron folds of outer colour and lining."""
    from pix import frame_mask
    from cards2 import rim
    m = frame_mask(top, bot, x0=x0, x1=x1, steps=3)

    def pat(x, y):
        if cut and cut(x, y):
            return '.'
        dx = abs(x - 16)
        t = ((y - top) - dx) % 6
        if t == 0:
            return 'k'
        if t >= 4:
            if ermine:
                return 'k' if (x % 3 == 1 and y % 2 == 0) else 'w'
            return lin
        return col
    g.fill(m, pat)
    rim(g, m, *rimc)
    return m


def tongues(x, y, bot, step=3):
    """Cut the mantle's lower edge into stair tongues."""
    return y > bot - 2 * (abs(x - 16) % step)


def corner_safe(x, y):
    return (x <= 5 and y <= 12) or (x >= 27 and y >= 35)


# ------------------------------------------------------------------ Rot
def daus_rot():
    g = Grid()
    line(g, 4, 44, 26, 12, 'y')
    line(g, 26, 34, 6, 12, 'y')
    for x, y in ((26, 11), (6, 11)):
        g.stamp([".y.", "yry", ".y."], x - 1, y - 2)
    mantle(g, 12, 41, 'r', 'y', cut=corner_safe, rimc=('y', 'k'))

    def heart(x, y):
        u = (x - 16) / 11.5
        v = -(y - 27.5) / 10.5
        return (u * u + v * v - 1) ** 3 - u * u * v ** 3 <= 0
    emblem(g, mask_fn(heart))
    g.big(BIG['Rot'], 9, 22)
    # crown
    g.stamp(["y...y...y...y",
             "yy..y..yyy.yy",
             "yyy.yy.yyyyyy",
             "yyyyyyyyyyyyy",
             "yrybyyryybyry",
             "kkkkkkkkkkkkk"], 10, 7)
    # banner
    g.stamp(["kyyyyyyyyyyyyyyyk",
             "yykkkkkkkkkkkkkyy",
             "kyyyyyyyyyyyyyyyk",
             "k..............k."], 8, 41)
    g.index('A', 'Rot')
    return g


# ------------------------------------------------------------------ Eichel
def daus_eichel():
    g = Grid()
    # crossed boar spears
    line(g, 4, 45, 26, 10, 'h')
    line(g, 26, 34, 6, 10, 'h')
    for x, y in ((26, 10), (6, 10)):
        g.stamp([".k.", "kwk", "kwk", ".k.", "kkk"], x - 1, y - 4)
    mantle(g, 13, 40, 'o', 'g', cut=corner_safe)

    def tri(x, y):
        return 15 <= y <= 45 and abs(x - 16) <= min(11.5, (45 - y) * 0.45 + 0.4)
    emblem(g, mask_fn(tri))
    # the acorn: stalk, wide checkered cap, narrower nut (blue on the Eichel cards)
    g.big(["...k...", "..fjf..", ".fjfjf.", "fjfjfjf", ".jjjjj.", ".jjjjj.", ".jjjjj.", "..jjj..", "...j..."], 9, 16)
    # barred helm with antlers
    g.stamp(["h.h.....h.h",
             "hh.h...h.hh",
             ".hhh...hhh.",
             "...hh.hh...",
             "...kkkkk...",
             "..kwwwwwk..",
             "..kwkwkwk..",
             "..kwkwkwk..",
             "..kkkkkkk..",
             ".kyyyyyyyk."], 11, 3)
    g.index('A', 'Eichel')
    return g


# ------------------------------------------------------------------ Gruen
def daus_gruen():
    g = Grid()
    # crossed lances with pennants
    line(g, 5, 45, 27, 9, 'o')
    line(g, 26, 34, 5, 9, 'o')
    mantle(g, 13, 40, 'g', 'y', cut=corner_safe, rimc=('y', 'k'))
    g.stamp(["k...", "kgg.", "kggg", "kyyy", "kgg.", "k..."], 27, 8)
    g.stamp(["...k", ".ggk", "gggk", "yyyk", ".ggk", "...k"], 2, 8)

    def sq(x, y):
        return 7 <= x <= 25 and 16 <= y <= 34 and not (abs(x - 16) >= 8 and (y <= 17 or y >= 33))
    emblem(g, mask_fn(sq), field='y')
    g.big(BIG['Gruen'], 9, 18)
    # helm with three feathers
    g.stamp(["..g...y...r..",
             "..gg..y..rr..",
             "...gg.y.rr...",
             "....ggyrr....",
             ".....kkk.....",
             "....kwwwk....",
             "....kkkkk....",
             "....kwkwk....",
             "...kkkkkkk...",
             "...kyyyyyk..."], 10, 2)
    # rosette below the shield
    g.stamp([".r.r.", "rryrr", ".ryr.", "rryrr", ".r.r."], 14, 37)
    g.index('A', 'Gruen')
    return g


# ------------------------------------------------------------------ Schellen
def daus_schellen():
    g = Grid()
    # crossed maces
    line(g, 4, 44, 26, 12, 'k')
    line(g, 26, 34, 6, 12, 'k')
    for x, y in ((26, 11), (6, 11)):
        g.stamp([".m.", "mkm", ".m."], x - 1, y - 2)
    # sunburst behind the medallion
    rays = set()
    for y in range(9, 46):
        for x in range(3, 30):
            if corner_safe(x, y):
                continue
            dx, dy = x - 16, y - 27.5
            r = math.hypot(dx, dy)
            a = math.atan2(dy, dx)
            ray = int((a + math.pi) / (2 * math.pi) * 12 + 0.5) % 2
            lim = 15.5 if ray else 13
            if r <= lim:
                g.px(x, y, 'y' if ray else 'r')
                rays.add((x, y))
    for x, y in border(rays):
        g.px(x, y, 'k')

    def circ(x, y):
        return math.hypot(x - 16, y - 27.5) <= 10.6
    pts = mask_fn(circ)
    emblem(g, pts, field='b')
    rim_pts = border(pts)
    for x, y in rim_pts:
        if (x + y) % 3 == 0:
            g.px(x, y, 'k')
    g.big(BIG['Schellen'], 9, 21)
    # crown and coins in metal gold: lemon is the suit, not the metal
    g.stamp(["m...m...m...m",
             "mm..m..mmm.mm",
             "mmmmmmmmmmmmm",
             "mkmwmmkmmwmkm",
             "kkkkkkkkkkkkk"], 10, 10)
    for x, y in ((6, 40), (24, 41)):
        g.stamp([".mm.", "mkkm", ".mm."], x, y)
    g.index('A', 'Schellen')
    return g


ACE = {'Eichel': daus_eichel, 'Gruen': daus_gruen, 'Rot': daus_rot, 'Schellen': daus_schellen}

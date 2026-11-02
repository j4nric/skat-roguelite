"""Bold flat pixel style ("Pixelstil"): a 33 x 49.5 art grid, one art pixel
= 2 x 2 CSS px at the 66 x 99 hand size. Flat colours from a small strict
palette, geometric garment patterns, an invisible frame that cuts the
figure with straight edges and stair-step chamfers."""

import numpy as np

GW, GH = 33, 49          # art grid (the card is 33 x 49.5, the last half row stays white)
# Suit colours (F3, chosen 02.11.2026): every suit's pictures use only its
# own core colour (with its own shades) plus neutrals: black, white, skin,
# greys, hair and wood brown, and a muted metal gold for crowns, chains and
# sceptres. The Wandelkarte (wild.py) keeps its rainbow.
PAL = {
    # core colours
    'r': '#e0281e',      # Rot: signal red
    'g': '#2e9a3c',      # Gruen: leaf green
    'j': '#4f86c6',      # Eichel: jeans blue
    'y': '#f5d52a',      # Schellen: lemon yellow
    # own shades
    'f': '#24507f',      # Eichel dark blue: acorn cap, index ink (8.2:1 on the paper, 7.1:1 tinted)
    't': '#a9c6e8',      # Eichel light blue (where the loden green was)
    'u': '#a87800',      # Schellen dark amber (where the red slashes were)
    'a': '#8a5f00',      # Schellen index ink (5.5:1 on the paper, 5.1:1 tinted)
    'v': '#1f7a30',      # Gruen index ink (5.3:1 on the paper, 4.6:1 tinted)
    'q': '#c0221a',      # Rot index ink (5.9:1 on the paper, 4.9:1 tinted)
    # neutrals
    'm': '#c39a3a',      # metal gold, all suits (crowns, chains, rims)
    'o': '#8b4a1c',      # brown: hair, wood, leather
    'k': '#1c1a1a',      # black
    'd': '#454b53',      # anthracite
    'i': '#8d939b',      # grey
    'w': '#ffffff',      # white
    's': '#f3c9a0',      # skin
    'p': '#c4685c',      # lips
    'h': '#8b4a1c',      # hair and wood of the Eichel cards (becomes 'o', see SUIT_MAP)
    'b': '#2a63c4',      # royal blue, only in the Wandelkarte's rainbow
}
BG = '#fdfdfb'

# ------------------------------------------------------------ pixel letters
GLYPH = {
    'K': ["x.x", "x.x", "xx.", "x.x", "x.x"],
    'O': ["xxx", "x.x", "x.x", "x.x", "xxx"],
    'U': ["x.x", "x.x", "x.x", "x.x", "xxx"],
    'A': [".x.", "x.x", "xxx", "x.x", "x.x"],
    '7': ["xxx", "..x", ".x.", ".x.", ".x."],
    '8': ["xxx", "x.x", "xxx", "x.x", "xxx"],
    '9': ["xxx", "x.x", "xxx", "..x", "xxx"],
    '10': ["x.xxx", "x.x.x", "x.x.x", "x.x.x", "x.xxx"],
}
HEART3 = ["x.x", "xxx", ".x."]
# chunky index signs (3 wide), already coloured
SIGN = {
    'Rot': ["r.r", "rrr", ".r."],
    'Gruen': [".g.", "ggg", "ggg", ".o."],
    'Eichel': [".k.", "fff", "jjj", ".j."],
    'Schellen': [".k.", "yyy", "uuu", "yky"],
}
# Ink of the rank letter per suit: a dark shade of the suit's own colour,
# at least 4.5:1 on the paper and on the suit's tinted paper ("Kartenfarbe
# tönen"). Never black: black read as a fifth suit.
INK = {'Rot': 'q', 'Gruen': 'v', 'Eichel': 'f', 'Schellen': 'a'}
# held / pip signs, 7 wide
BIG = {
    'Rot': [".rr.rr.", "rrrrrrr", "rrrrrrr", ".rrrrr.", "..rrr..", "...r..."],
    'Gruen': ["...g...", "..ggg..", ".ggggg.", "ggggggg", "gggkggg", ".ggkgg.", "...o..."],
    # the acorn: a stalk, a wide checkered cap over a narrower nut, so it
    # does not read as a heart at game size
    'Eichel': ["...k...", ".fjfjf.", "fjfjfjf", ".jjjjj.", ".jjjjj.", ".jjjjj.", "..jjj..", "...j..."],
    'Schellen': ["...k...", "..yyy..", ".yyyyy.", "yyyyyyy", "uuuuuuu", "yyykyyy", ".yykyy."],
}
MID = {
    'Rot': ["rr.rr", "rrrrr", "rrrrr", ".rrr.", "..r.."],
    'Gruen': ["..g..", ".ggg.", "ggggg", "ggkgg", "..o.."],
    'Eichel': ["..k..", "fffff", ".jjj.", ".jjj.", "..j.."],
    'Schellen': ["..k..", ".yyy.", "yyyyy", "uuuuu", "yykyy"],
}
HEART5 = [".x.x.", "xxxxx", "xxxxx", ".xxx.", "..x.."]
HEART7 = [".xx.xx.", "xxxxxxx", "xxxxxxx", ".xxxxx.", "..xxx..", "...x..."]


# Per suit, the keys the drawings use that are not the suit's own colour
# or a neutral, and what they become. The drawings use 'y' for gold
# everywhere; it stays lemon only on Schellen (except crowns, chains and
# sceptres, which cards2/aces2 draw in metal gold 'm' there). 'h' marks
# hair and wood on the Eichel cards, whose brown 'o' becomes the blue.
SUIT_MAP = {
    'Rot': {'y': 'm', 'b': 'i', 'g': 'i', 'h': 'o'},
    'Gruen': {'y': 'm', 'r': 'w', 'b': 'w', 'h': 'o'},
    'Eichel': {'o': 'j', 'h': 'o', 'g': 't', 'r': 'f', 'y': 'm', 'b': 'f'},
    'Schellen': {'r': 'u', 'b': 'd', 'g': 'd', 'h': 'o'},
}


def suit_colours(grid, suit):
    """Recolour a finished face to its suit's colours (SUIT_MAP). Red pixels
    beside skin or beard in the head area are lips and become 'p' on every
    suit."""
    m = SUIT_MAP.get(suit)
    if m is None:
        return
    g = grid.g
    lips = [(x, y) for y in range(17) for x in range(1, GW - 1) if g[y, x] == 'r' and
            (g[y, x - 1] in 'sh' or g[y, x + 1] in 'sh')]
    out = g.copy()
    for y in range(GH):
        for x in range(GW):
            c = g[y, x]
            if c in m:
                out[y, x] = m[c]
    for x, y in lips:
        out[y, x] = 'p'
    grid.g = out


class Grid:
    def __init__(self):
        self.g = np.full((GH, GW), '.', dtype='<U1')

    def px(self, x, y, c):
        if 0 <= x < GW and 0 <= y < GH and c != '.':
            self.g[y, x] = c

    def hline(self, x0, x1, y, c):
        for x in range(x0, x1 + 1):
            self.px(x, y, c)

    def rect(self, x0, y0, x1, y1, c):
        for y in range(y0, y1 + 1):
            self.hline(x0, x1, y, c)

    def stamp(self, rows, x0, y0, cmap=None, flip=False):
        for j, row in enumerate(rows):
            n = len(row)
            for i, ch in enumerate(row):
                if ch == '.':
                    continue
                c = cmap.get(ch, ch) if cmap else ch
                self.px(x0 + (n - 1 - i if flip else i), y0 + j, c)

    def fill(self, mask, fn):
        """mask: dict y -> (x0, x1); fn(x, y) -> colour key or '.'."""
        for y, (x0, x1) in mask.items():
            for x in range(x0, x1 + 1):
                c = fn(x, y)
                if c and c != '.':
                    self.px(x, y, c)

    def index(self, rank, suit='Rot'):
        g = GLYPH[rank]
        w = len(g[0])
        col = INK[suit]
        sg = SIGN[suit]
        self.stamp(g, 2, 2, {'x': col})
        self.stamp(sg, 2, 8)
        # bottom right, upright: sign above the rank
        self.stamp(sg, GW - 5, GH - 8 - len(sg))
        self.stamp(g, GW - 2 - w, GH - 7, {'x': col})
        # The index is the last thing every face stamps: its suit colours now.
        suit_colours(self, suit)

    def big(self, rows, x0, y0, k=2):
        """Stamp a sprite scaled by k (chunky blocks)."""
        for j, row in enumerate(rows):
            for i, ch in enumerate(row):
                if ch != '.':
                    for dy in range(k):
                        for dx in range(k):
                            self.px(x0 + i * k + dx, y0 + j * k + dy, ch)

    def svg(self, w=66, h=99, label=''):
        parts = [f'<rect x="0" y="0" width="{GW}" height="{GH + 0.5}" fill="{BG}"></rect>']
        for key, col in PAL.items():
            d = []
            for y in range(GH):
                x = 0
                row = self.g[y]
                while x < GW:
                    if row[x] == key:
                        k = x
                        while k + 1 < GW and row[k + 1] == key:
                            k += 1
                        d.append(f'M{x} {y}h{k - x + 1}v1h-{k - x + 1}z')
                        x = k + 1
                    else:
                        x += 1
            if d:
                parts.append(f'<path d="{"".join(d)}" fill="{col}"></path>')
        return (f'<svg viewBox="0 0 {GW} {GH + 0.5}" width="{w}" height="{h}" shape-rendering="crispEdges" role="img" '
                f'aria-label="{label}" style="display: block">' + ''.join(parts) + '</svg>')

    def png(self, path, s=10):
        from PIL import Image
        im = Image.new('RGB', (GW * s, int((GH + 0.5) * s)), BG)
        px = im.load()
        for y in range(GH):
            for x in range(GW):
                c = self.g[y, x]
                if c != '.':
                    rgb = tuple(int(PAL[c][i:i + 2], 16) for i in (1, 3, 5))
                    for j in range(s):
                        for i in range(s):
                            px[x * s + i, y * s + j] = rgb
        return im


def frame_mask(top, bot, x0=6, x1=26, steps=3):
    """Hexagon-like invisible frame: full width from top+steps to bot-steps,
    stair-step chamfers of 2 px per row at both ends."""
    m = {}
    for y in range(top, bot + 1):
        k = min(y - top, bot - y, steps)
        m[y] = (x0 + 2 * (steps - k), x1 - 2 * (steps - k))
    return m


def edge_of(mask):
    """Outer stair edge pixels of a frame mask (for the yellow rim)."""
    out = set()
    ys = sorted(mask)
    for i, y in enumerate(ys):
        x0, x1 = mask[y]
        prev = mask.get(y - 1)
        nxt = mask.get(y + 1)
        for x in range(x0, x1 + 1):
            above = prev is None or not (prev[0] <= x <= prev[1])
            below = nxt is None or not (nxt[0] <= x <= nxt[1])
            if above or below:
                out.add((x, y))
    return out

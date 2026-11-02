"""Undead variants ("Untot") of the picture cards in the Pixelstil.

Same grid, poses and frames as the living cards; a transform recolours the
figure and turns faces and hands to bone, rot or mist:
  skelett  bone skull and hands, faded and tattered garments, cobweb
  geist    the whole figure in pale ghost blues, glowing rim, fading hem
  zombie   green-grey skin, rotten muted colours, torn holes, stitches
Marker at game size: a small skull sign at the bottom left corner."""

import numpy as np
from pix import Grid, PAL, BG, GW, GH, GLYPH, SIGN, INK
import cards2
import aces2

EXTRA = {
    'B': '#ece4c8', 'D': '#a89c78',               # bone, bone shadow
    'Z': '#9cb86e', 'X': '#5e7a3c',               # zombie skin, dark
    'C': '#bff0ff',                               # ghost glow
    'P': '#7a6a8a',                               # web grey-violet
    'E': '#ff4a2a',                               # zombie eye
}
GHOST = {'k': '#3b5f7a', 'r': '#86c6da', 'y': '#e2f6f8', 'g': '#a6dccd', 'b': '#5d9cc0', 'o': '#7aa2b2',
         'w': '#f4fcfe', 's': '#d4f0f4'}
ROT = {'r': '#9e4038', 'y': '#bc9c4a', 'g': '#557c40', 'b': '#4a628a', 'o': '#6b4a2a', 'w': '#dcd6c0', 'k': '#1c1a1a',
       's': '#9cb86e'}
FADE = {'r': '#c25a4e', 'y': '#e2c46a', 'g': '#5e9a58', 'b': '#5a7cb8', 'o': '#8a6040', 'w': '#ffffff', 'k': '#1c1a1a',
        's': '#ece4c8'}
SKULL = ['.###.', '#####', '#.#.#', '#####', '.#.#.']


def base(suit, rank):
    if rank == 'A':
        g = aces2.ACE[suit]()
    else:
        g = cards2.FIG[rank](suit)
    g.suit, g.rank = suit, rank
    return g


def index_mask(g):
    ig = Grid()
    ig.index(g.rank, g.suit)
    return ig.g != '.'


FACE_BOX = {'K': (13, 8, 20, 12), 'O': (13, 5, 19, 13), 'U': (11, 6, 20, 11)}


def face_cells(g):
    if g.rank not in FACE_BOX:
        return set()
    x0, y0, x1, y1 = FACE_BOX[g.rank]
    return {(x, y) for y in range(y0, y1 + 1) for x in range(x0, x1 + 1) if g.g[y, x] in 'sk' and
            any(g.g[yy, xx] == 's' for yy in range(max(0, y - 1), y + 2) for xx in range(x - 1, x + 2))}


def skull_face(g, bone='B', shade='D'):
    """Turn the face into a skull: bone, black sockets, nose hole, teeth."""
    if g.rank not in FACE_BOX:
        return
    x0, y0, x1, y1 = FACE_BOX[g.rank]
    eye = mouth = None
    for y in range(y0, y1 + 1):
        for x in range(x0, x1 + 1):
            c = g.g[y, x]
            if c == 's':
                g.g[y, x] = bone
            elif c == 'r' and mouth is None:
                mouth = (x, y)
    # sockets: find the old eye (a single k pixel surrounded by face)
    for y in range(y0, y1 + 1):
        for x in range(x0, x1 + 1):
            if g.g[y, x] == 'k' and g.g[y, x - 1] == bone and g.g[y, x + 1] == bone and eye is None:
                eye = (x, y)
    if eye:
        ex, ey = eye
        for dx, dy in ((0, 0), (1, 0), (0, 1), (1, 1)):
            g.g[ey + dy, ex + dx] = 'k'
        g.g[ey + 2, ex + 2] = 'k'
    if mouth:
        mx, my = mouth
        for dx in range(-1, 3):
            if g.g[my, mx + dx] in (bone, 'r'):
                g.g[my, mx + dx] = 'k' if (mx + dx) % 2 else bone


def hands_to(g, col, keep=set()):
    for y in range(GH):
        for x in range(GW):
            if g.g[y, x] == 's' and (x, y) not in keep:
                g.g[y, x] = col


def tatter(g, imask, strength=5):
    """Fray the outer edge and punch small holes into the garments."""
    out = g.g.copy()
    for y in range(GH):
        for x in range(GW):
            c = g.g[y, x]
            if c == '.' or imask[y, x] or y < 14:
                continue
            edge = any(0 <= y + dy < GH and 0 <= x + dx < GW and g.g[y + dy, x + dx] == '.'
                       for dx, dy in ((1, 0), (-1, 0), (0, 1)))
            h = (x * 7 + y * 13) % strength
            if edge and h == 0:
                out[y, x] = '.'
            elif not edge and (x * 5 + y * 11) % 23 == 0 and c not in 'sBZk':
                out[y, x] = 'k'
    g.g = out


def cobweb(g, x0=24, y0=0):
    rows = ["PPPPPPPPP",
            ".P..P...P",
            "..PP.P.P.",
            "...P.PP..",
            "....PP.P.",
            ".....P.P.",
            "......PP.",
            ".......P."]
    for j, r in enumerate(rows):
        for i, ch in enumerate(r):
            if ch == 'P' and g.g[y0 + j, x0 + i] == '.':
                g.g[y0 + j, x0 + i] = 'P'


def marker(g, col='k'):
    for j, r in enumerate(SKULL):
        for i, ch in enumerate(r):
            if ch == '#':
                g.g[42 + j, 2 + i] = col
    # halo around the marker
    return g


def skelett(suit, rank):
    g = base(suit, rank)
    im = index_mask(g)
    pal = dict(PAL)
    pal.update(FADE)
    pal.update(EXTRA)
    if rank == 'A':
        # bones crossed behind, a skull over the crest
        for k in range(14):
            for x, y in ((6 + k, 30 + k), (26 - k, 30 + k)):
                if g.g[y, x] == '.':
                    g.g[y, x] = 'B'
        sk = ["..BBBBB..", ".BBBBBBB.", "BBkkBkkBB", "BBkkBkkBB", "BBBBkBBBB", ".BBBBBBB.", "..BkBkB.."]
        for j, r in enumerate(sk):
            for i, ch in enumerate(r):
                if ch != '.':
                    g.g[3 + j, 12 + i] = ch
    else:
        skull_face(g)
        hands_to(g, 'B')
    tatter(g, im, 6)
    cobweb(g)
    marker(g, 'k')
    return g, pal


def geist(suit, rank):
    g = base(suit, rank)
    im = index_mask(g)
    pal = dict(PAL)
    if rank != 'A':
        skull_face(g, 's', 's')
    # glow ring and fading hem
    out = g.g.copy()
    for y in range(GH):
        for x in range(GW):
            if g.g[y, x] == '.' and not im[y, x]:
                if any(0 <= y + dy < GH and 0 <= x + dx < GW and g.g[y + dy, x + dx] != '.' and not im[y + dy, x + dx]
                       for dx in (-1, 0, 1) for dy in (-1, 0, 1)):
                    out[y, x] = 'C'
            elif not im[y, x] and y >= 38 and (x + y) % 2 == 0:
                out[y, x] = 'C' if y < 42 else '.'
    g.g = out
    # ghost colours only for the picture: rename picture pixels to lower-case ghost keys
    gp = {}
    for k, v in GHOST.items():
        gp['g' + k] = v
    pic = np.vectorize(lambda c: c)(g.g)
    for y in range(GH):
        for x in range(GW):
            c = g.g[y, x]
            if not im[y, x] and c in GHOST:
                g.g[y, x] = {'k': '1', 'r': '2', 'y': '3', 'g': '4', 'b': '5', 'o': '6', 'w': '7', 's': '8'}[c]
    pal.update({'1': GHOST['k'], '2': GHOST['r'], '3': GHOST['y'], '4': GHOST['g'], '5': GHOST['b'], '6': GHOST['o'],
                '7': GHOST['w'], '8': GHOST['s']})
    pal.update(EXTRA)
    marker(g, '1')
    return g, pal


def zombie(suit, rank):
    g = base(suit, rank)
    im = index_mask(g)
    pal = dict(PAL)
    if rank == 'A':
        # green slime dripping from the emblem
        for x in range(6, 27, 3):
            top = next((y for y in range(10, 45) if g.g[y, x] != '.' and not im[y, x]), None)
            if top is None:
                continue
            bot = max(y for y in range(10, 46) if g.g[y, x] != '.' and not im[y, x])
            for k in range(1, 2 + (x % 4)):
                if bot + k < GH and g.g[bot + k, x] == '.':
                    g.g[bot + k, x] = 'X'
    else:
        x0, y0, x1, y1 = FACE_BOX[rank]
        for y in range(y0, y1 + 1):
            for x in range(x0, x1 + 1):
                if g.g[y, x] == 's':
                    g.g[y, x] = 'Z'
                elif g.g[y, x] == 'r':
                    g.g[y, x] = 'X'
        # a red glowing eye
        for y in range(y0, y1 + 1):
            for x in range(x0 + 1, x1):
                if g.g[y, x] == 'k' and g.g[y, x - 1] == 'Z' and g.g[y, x + 1] == 'Z':
                    g.g[y, x] = 'E'
        # stitches across the cheek
        for (x, y) in ((x0 + 1, y0 + 3), (x0 + 2, y0 + 2), (x0 + 3, y0 + 3)):
            if g.g[y, x] == 'Z':
                g.g[y, x] = 'k'
        hands_to(g, 'Z')
    tatter(g, im, 4)
    for y in range(GH):
        for x in range(GW):
            c = g.g[y, x]
            if not im[y, x] and c in ROT and c not in 'k':
                g.g[y, x] = {'r': '2', 'y': '3', 'g': '4', 'b': '5', 'o': '6', 'w': '7', 's': 'Z'}[c]
    pal.update({'2': ROT['r'], '3': ROT['y'], '4': ROT['g'], '5': ROT['b'], '6': ROT['o'], '7': ROT['w']})
    pal.update(EXTRA)
    if not getattr(g, 'no_marker', False) and globals().get('ZOMBIE_GRID_MARKER', True):
        marker(g, 'X')
    return g, pal


STYLES = {'skelett': skelett, 'geist': geist, 'zombie': zombie}


def svg(g, pal, w=66, h=99, label='', extra=''):
    parts = [f'<rect x="0" y="0" width="{GW}" height="{GH + 0.5}" fill="{BG}"></rect>']
    keys = sorted(set(g.g.flatten()) - {'.'})
    for key in keys:
        col = pal.get(key)
        if not col:
            continue
        d = []
        for y in range(GH):
            row = g.g[y]
            x = 0
            while x < GW:
                if row[x] == key:
                    k = x
                    while k + 1 < GW and row[k + 1] == key:
                        k += 1
                    d.append(f'M{x} {y}h{k - x + 1}v1h-{k - x + 1}z')
                    x = k + 1
                else:
                    x += 1
        parts.append(f'<path d="{"".join(d)}" fill="{col}"></path>')
    return (f'<svg viewBox="0 0 {GW} {GH + 0.5}" width="{w}" height="{h}" shape-rendering="crispEdges" role="img" '
            f'aria-label="{label}" style="display: block">' + ''.join(parts) + extra + '</svg>')


def zombie_card(suit, rank):
    """Final U3 card: zombie art, ragged green edge, skull marker (fx2 layers)."""
    import fx2
    global ZOMBIE_GRID_MARKER
    ZOMBIE_GRID_MARKER = False
    try:
        if rank in ('A', 'K', 'O', 'U'):
            g, pal = zombie(suit, rank)
        else:
            g = cards2.number(suit, int(rank))
            g.suit, g.rank = suit, str(rank)
            pal = dict(PAL)
            pal.update(EXTRA)
    finally:
        ZOMBIE_GRID_MARKER = True
    return g, pal, fx2.MODS['untot'][1](BG)

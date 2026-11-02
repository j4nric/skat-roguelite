"""Wandelkarte: eight suit-independent cards (A, K, O, U, 10-7) with a
joker vibe: harlequin diamonds, jester caps with bells, a marotte, and a
rainbow diamond as the sign. Registers the pseudo-suit 'Wild' in the pix
and cards2 tables (same poses and frames as every suit).

Two looks (DESIGN 7.2, decided by the user on 03.11.2026):
- open (`card`): before the game starts the fool wears a harlequin in the
  four suit colours (Rot 'r', Schellen 'y', Gruen 'g', Eichel 'j'), the
  corner index shows the rank and four tiny suit pips;
- settled (`settled`): a palette swap of the same drawing, costume and cap
  only in shades of the new suit, and the normal corner index of that
  suit, exactly like an ordinary card."""

import math
import pix
import cards2
import aces2
from pix import Grid, GW, GH

# ------------------------------------------------------------ signs
pix.BIG['Wild'] = ["...r...", "..rrr..", ".yyyyy.", "ggggggg", ".jjjjj.", "..ooo..", "...o..."]
pix.MID['Wild'] = ["..r..", ".yyy.", "ggggg", ".jjj.", "..o.."]
# the corner index of the open card: four tiny suit pips (Eichel, Gruen / Rot, Schellen)
pix.SIGN['Wild'] = ["jjgg", "jjgg", "rryy", "rryy"]
pix.INK['Wild'] = 'k'

HQ = 'rygb'


def harle(x, y, size=3, line=True):
    """Harlequin diamonds in four colours with black seams."""
    u = (x + y) // size
    v = (x - y) // size
    if line and ((x + y) % size == 0 or (x - y) % size == 0):
        return 'k'
    return HQ[(u + 2 * v) % 4]


def ruff(x, y):
    return 'k' if (x % 2 == 0 and y % 2 == 1) else 'w'


def k_pat_wild(x, y):
    if y <= 15:
        return ruff(x, y)
    side = x <= 9 or x >= 23
    if side and y <= 33:
        if y in (24, 25):
            return 'y' if x % 2 else 'k'
        return 'r' if (y // 3) % 2 else 'b'
    if y in (27, 29):
        return 'k'
    if y == 28:
        return 'y' if x % 2 else 'r'
    if y < 27:
        return harle(x, y, 4)
    return harle(x, y, 5)


def o_pat_wild(x, y):
    if y <= 16:
        return ruff(x, y)
    side = x <= 10 or x >= 22
    if side and y <= 27:
        return 'g' if (y // 2) % 2 else 'y'
    if y <= 27:
        return harle(x, y, 4)
    if y == 28:
        return 'y'
    if y == 29:
        return 'k'
    if y >= 42:
        return 'r' if (x // 2) % 2 else 'b'
    return harle(x, y, 5)


def u_pat_wild(x, y):
    if y <= 14:
        return ruff(x, y)
    side = x <= 10 or x >= 22
    if side:
        if y in (22, 23):
            return 'k'
        return 'b' if x <= 10 and y < 22 else ('r' if x <= 10 else ('y' if y < 22 else 'g'))
    if y in (26, 28):
        return 'k'
    if y == 27:
        return 'y'
    return harle(x, y, 4)


def legs_wild(g):
    for y in range(35, 45):
        for x in range(10, 15):
            g.px(x, y, 'r' if (y // 2) % 2 else 'g')
        for x in range(18, 23):
            g.px(x, y, 'y' if (y // 2) % 2 else 'b')
    # bells at the toes
    g.px(8, 44, 'y')
    g.px(24, 44, 'y')


cards2.KOENIG['Wild'] = dict(
    pat=k_pat_wild, rim=('y', 'k'), hair='k', beard='k',
    crown=(12, 3, ["r...g...b", "yy.yyy.yy", "yyyyyyyyy", "yryygyyby", "kkkkkkkkk"]),
    right=["yry.", "rkr.", ".s..", ".k..", ".k..", ".k..", ".k..", ".k..", ".k..", ".k..", ".k..", ".k..", ".ss.", ".ss.",
           ".k..", ".k.."],
    left=["ss.", "ss.", ".k.", ".y.", "yyy", ".k."])

cards2.OBERIN['Wild'] = dict(
    pat=o_pat_wild, rim=('y', 'k'),
    head=["y.....y", "rr...bb", ".rrrbbb", "kssssk.", "ksksskk", "kssssk.", "ksrssk.", "kssssk.", "kk.sskk", "kk...kk",
          "yk...ky"],
    sleeve=["ss", "ss", "yy", "gg", "yy", "gg", "yy", "gg", "yy"])

cards2.UNTER['Wild'] = dict(
    pat=u_pat_wild, rim=('y', 'k'), legs=legs_wild,
    head=["y..y...y", "g..r..bb", "gg.rr.b.", "ggrrrbbb", ".kkssss.", ".ksssks.", ".kssssss", ".kkssrs.", "..ssss.."],
    pole=[".y..", "yry.", ".y..", "..k.", "..k.", "..k.", "..k.", "..k.", "..k.", "..k.", ".ss.", ".ss.", "..k.", "..k.",
          "..k.", "..k.", "..k.", "..k.", "..k.", "..k.", "..k."])


def daus_wild():
    g = Grid()
    aces2.mantle(g, 13, 40, 'r', 'y', cut=aces2.corner_safe, rimc=('y', 'k'))
    # repaint the mantle in harlequin diamonds
    for y in range(13, 41):
        for x in range(4, 29):
            if g.g[y, x] in ('r', 'y') and not aces2.corner_safe(x, y):
                g.g[y, x] = harle(x, y, 4)

    def rhomb(x, y):
        return abs(x - 16) / 11.5 + abs(y - 28.5) / 14.5 <= 1
    aces2.emblem(g, aces2.mask_fn(rhomb))
    g.big(pix.BIG['Wild'], 9, 22)
    # jester cap with three bells
    g.stamp(["y.....y.....y",
             "rr....y....bb",
             ".rr..ggg..bb.",
             "..rrrgggbbb..",
             "..kkkkkkkkk..",
             "..kyyyyyyyk.."], 10, 6)
    g.index('A', 'Wild')
    return g


aces2.ACE['Wild'] = daus_wild


def card(rank):
    """The open Wandelkarte: the harlequin in the four suit colours."""
    if rank == 'A':
        g = daus_wild()
    elif rank in ('K', 'O', 'U'):
        g = cards2.FIG[rank]('Wild')
    else:
        g = cards2.number('Wild', int(rank))
    # The drawings use royal blue 'b' for the fourth colour: Eichel's jeans blue.
    g.g[g.g == 'b'] = 'j'
    g.suit, g.rank = 'Wild', str(rank)
    return g


# Shades of the settled costume per suit: (core, light, dark).
TONES = {
    'Eichel': ('j', 't', 'f'),
    'Gruen': ('g', 'l', 'v'),
    'Rot': ('r', 'n', 'q'),
    'Schellen': ('y', 'c', 'u'),
}


def settled(suit, rank):
    """The Wandelkarte settled into `suit`: the open drawing with every
    harlequin colour swapped for a shade of that suit (neighbouring
    diamonds keep apart by shade and the black seams), and the suit's
    normal corner index instead of the four pips."""
    g = card(rank)
    open_idx = Grid()
    open_idx.index(str(rank), 'Wild')
    core, light, dark = TONES[suit]
    swap = {'r': core, 'y': light, 'g': dark, 'j': core}
    src = g.g
    out = src.copy()
    for y in range(GH):
        for x in range(GW):
            if open_idx.g[y, x] != '.':
                out[y, x] = '.'
                continue
            c = src[y, x]
            if c == 'r' and y < 17 and 0 < x < GW - 1 and (src[y, x - 1] == 's' or src[y, x + 1] == 's'):
                out[y, x] = 'p'      # lips stay lips
            elif c in swap:
                out[y, x] = swap[c]
    idx = Grid()
    idx.index(str(rank), suit)
    out[idx.g != '.'] = idx.g[idx.g != '.']
    g.g = out
    g.suit, g.rank = suit, str(rank)
    return g

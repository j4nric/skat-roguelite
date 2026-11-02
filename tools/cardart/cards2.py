"""All four suits in the flat pixel style. One pose per rank (the same
invisible frame, head, hands and prop positions for every suit); the suits
differ in headwear, garment patterns, sleeves, props and colours:

  Eichel    Jagdhof: brown and loden green, fur, chevrons, Tyrolean hats,
            boar spear, hunting horn, dirndl with apron
  Gruen     gotischer Waldhof: green and gold, diamond lattice, dagged
            edges, lily crown and sceptre, hennin, hood, longbow
  Rot       Koenigshof: red, gold, blue, ermine, diamond medallion, sword,
            sceptre
  Schellen  Buerger und Landsknechte: gold with red slashes and stripes,
            fur collar, chain of office, bonnet, ruff, mace, halberd
"""

from pix import Grid, frame_mask, edge_of, BIG, MID, GW, GH


def rim(g, mask, outer='y', inner='k'):
    e = edge_of(mask)
    for (x, y) in e:
        g.px(x, y, outer)
    for (x, y) in e:
        for dy in (-1, 1):
            yy = y + dy
            if yy in mask and mask[yy][0] <= x <= mask[yy][1] and (x, yy) not in e:
                g.px(x, yy, inner)


def ermine(x, y):
    return 'k' if (x % 3 == 1 and y % 2 == 0) else 'w'


def fur(x, y):
    return 'o' if ((x + y) % 3 == 0) else 'w'


def recolor(rows, mp):
    return [''.join(mp.get(c, c) for c in r) for r in rows]


# ------------------------------------------------------------ Koenig pose
KHEAD = ["ssssssHH",
         "sesssHHH",
         "sssssHHH",
         "smsssHH.",
         "BBBBBBB."]


def k_pat_rot(x, y):
    if y <= 15:
        return ermine(x, y)
    side = x <= 9 or x >= 23
    if side and y <= 33:
        if y in (22, 23):
            return 'b'
        if y in (21, 24):
            return 'k'
        if y >= 31:
            return ermine(x, y)
        return 'r'
    if y in (27, 29):
        return 'k'
    if y == 28:
        return 'b' if 15 <= x <= 17 else 'y'
    if y < 27:
        if 15 <= x <= 17:
            return 'k' if (x + y) % 3 == 0 else 'r'
        if x in (14, 18):
            return 'k'
        if (x - y) % 4 == 0:
            return 'b'
        if (x + y) % 4 == 0 and y % 2:
            return 'k'
        return 'w'
    d = abs(x - 16) + abs(y - 36)
    if d <= 1:
        return 'y'
    if d <= 3:
        return 'b'
    if d <= 4:
        return 'k'
    if d <= 6:
        return 'r'
    if d == 7:
        return 'y'
    z = (y + abs(x - 16)) // 2 % 3
    return 'rbk'[z] if not side else 'ryk'[z]


def k_pat_eichel(x, y):
    if y <= 15:
        return fur(x, y)
    side = x <= 9 or x >= 23
    if side and y <= 33:
        if y in (22, 23):
            return 'g'
        if y in (21, 24):
            return 'k'
        if y >= 31:
            return fur(x, y)
        return 'o'
    if y in (27, 29):
        return 'k'
    if y == 28:
        return 'k' if x == 16 else 'y'
    if y < 27:
        if x in (11, 21):
            return fur(x, y)
        if x in (12, 20):
            return 'k'
        # loden panel with antler chevrons
        return 'y' if (y - abs(x - 16)) % 4 == 0 else ('o' if (y - abs(x - 16)) % 4 == 2 else 'g')
    # lower robe: a downward triangle medallion on brown and green checks
    t = 41 - y - abs(x - 16)
    if t >= 0 and y >= 30:
        if t <= 0:
            return 'k'
        if t <= 1:
            return 'y'
        if t <= 4:
            return 'g'
        return 'o'
    return 'o' if (x // 2 + y // 2) % 2 else 'g'


def k_pat_gruen(x, y):
    if y <= 15:
        return 'y' if y == 14 else 'g'
    side = x <= 9 or x >= 23
    if side:
        # long bag sleeves, leaf-dagged at the bottom
        if y >= 34 + (x % 2):
            return 'y' if y == 34 + (x % 2) else '.'
        if y in (20, 21):
            return 'y'
        return 'g'
    if y in (27, 29):
        return 'k'
    if y == 28:
        return 'r' if x == 16 else 'y'
    # diamond lattice, gold on green
    if (x + y) % 6 == 0 or (x - y) % 6 == 0:
        return 'y'
    if (x + y) % 6 == 3 and (x - y) % 6 == 3:
        return 'r' if y < 27 else 'w'
    return 'g'


def k_pat_schellen(x, y):
    if y <= 18:
        # broad fur shawl collar
        return 'k' if (x % 3 == 0 and y % 2 == 1) else 'w'
    side = x <= 9 or x >= 23
    if side and y <= 33:
        if y in (25, 26):
            return 'k'
        return 'r' if x % 2 == 0 else 'y'
    if y in (27, 29):
        return 'k'
    if y == 28:
        return 'y'
    if y < 27:
        # chain of office on striped puffs
        if abs(x - 16) == 22 - y or (abs(x - 16) <= 1 and y in (23, 24)):
            return 'k' if (x + y) % 2 else 'm'
        return 'r' if x % 2 == 0 else 'y'
    if x in (12, 20):
        return 'k' if y % 2 else 'w'
    if 13 <= x <= 19:
        return 'r' if x % 2 == 0 else 'y'
    return 'k' if x in (8, 24) and y % 3 else 'y'


KOENIG = {
    'Rot': dict(pat=k_pat_rot, rim=('y', 'k'), hair='k', beard='k',
                crown=(12, 3, ["y...y...y", "yy.yyy.yy", "yyyyyyyyy", "ybyyryyby", "rrrrrrrrr"]),
                right=[".b..", "bb..", "bb..", "bb..", "bb..", "bb..", "bb..", "bb..", "bb..", "bb..", "yyyy",
                       ".k..", ".ss.", ".ss."],
                left=["ss", "ss", "y.", "y.", "y.", "y.", "y.", "yy", "yy", "r."]),
    'Eichel': dict(pat=k_pat_eichel, rim=('y', 'k'), hair='h', beard='h',
                   crown=(12, 3, ["y...y...y", "yy.yyy.yy", "yyyyyyyyy", "ygyyryygy", "owowowowo"]),
                   right=["..k.", ".kwk", ".kwk", ".kwk", "..k.", "kkkk", "..h.", "..h.", "..h.", "..h.", "..h.",
                          "..h.", ".ss.", ".ss.", "..h.", "..h.", "..h.", "..h."],
                   left=["ss.", "ss.", "y..", "yy.", "yyy", ".yy", "..k"]),
    'Gruen': dict(pat=k_pat_gruen, rim=('y', 'k'), hair='y', beard='y',
                  crown=(12, 2, [".y..y..y.", "yyy.y.yyy", ".y.yyy.y.", "yyyyyyyyy", "gygrgygyg", "........."]),
                  right=["..y..", ".yyy.", "y.y.y", "..y..", "..y..", "..y..", "..y..", "..y..", "..y..", "..y..",
                         "..y..", "..y..", ".ss..", ".ss..", "..y..", "..y.."],
                  left=[".y.", "yry", "yyy", "ss.", "ss."]),
    'Schellen': dict(pat=k_pat_schellen, rim=('k', 'y'), hair='k', beard='k',
                     crown=(11, 3, ["...m.m.m...", "...mmmmm...", "...mkmkm...", "rrrrrrrrrrr", ".rrrrrrrrr."]),
                     right=[".mmm", ".mkm", ".mmm", "..k.", "..k.", "..k.", "..k.", "..k.", "..k.", "..k.", "..k.",
                            "..k.", ".ss.", ".ss.", "..k.", "..k."],
                     left=["ss.", "ss.", ".k.", "ooo", "oyo", "ooo"]),
}


def koenig(suit):
    s = KOENIG[suit]
    g = Grid()
    m = frame_mask(13, 44)
    g.fill(m, s['pat'])
    rim(g, m, *s['rim'])
    g.hline(12, 14, 45, 'k')
    g.hline(18, 20, 45, 'k')
    x, y, rows = s['crown']
    head = recolor(KHEAD, {'H': s['hair'], 'B': s['beard'], 'e': 'k', 'm': 'r'})
    g.stamp(head, 13, 8)
    g.stamp(rows, x, y)
    right = s['right']
    g.stamp(right, 26, 5 if len(right[0]) == 4 else 5)
    g.stamp(s['left'], 4 if len(s['left'][0]) <= 2 else 3, 30)
    g.index('K', suit)
    return g


# ------------------------------------------------------------ Oberin pose
def o_mask():
    m = {14: (12, 20), 15: (10, 22)}
    for y in range(16, 30):
        m[y] = (8, 24)
    m[30] = (7, 25)
    for y in range(31, 42):
        m[y] = (6, 26)
    m[42], m[43], m[44] = (7, 25), (9, 23), (11, 21)
    return m


def o_pat_rot(x, y):
    if y <= 16:
        return 'k' if (x in (11, 21) or y == 16 and x in (12, 20)) else ('y' if y == 14 else 'w')
    side = x <= 10 or x >= 22
    if side and y <= 27:
        if y in (21, 22):
            return 'y'
        if y == 23:
            return 'k'
        return 'r' if y < 21 else 'b'
    if y <= 27:
        if 15 <= x <= 17 and y >= 18:
            return 'y' if y % 2 else 'k'
        u, v = (x - 16) % 6, (y - 17) % 6
        d = abs(u - 3) + abs(v - 3)
        if y <= 17:
            return 'w'
        return 'y' if d == 0 else ('b' if d <= 1 else ('k' if d == 2 else 'r'))
    if y == 28:
        return 'y'
    if y == 29:
        return 'k'
    z = (y + abs((x % 6) - 3)) % 8
    return 'rrbbyykr'[z] if not (15 <= x <= 17) else ('w' if (y % 3) else 'k')


def o_pat_eichel(x, y):
    if y <= 16:
        return 'w' if y < 16 else 'k'
    side = x <= 10 or x >= 22
    if side and y <= 27:
        # white puffed blouse sleeves, green cuffs
        if y >= 25:
            return 'g'
        if y == 24:
            return 'k'
        return 'w' if (x + y) % 3 else 'k' if y in (19, 21) else 'w'
    if y <= 27:
        # laced bodice
        if 14 <= x <= 18:
            return 'w' if abs(x - 16) == (y % 4) % 3 else 'k'
        return 'o'
    if y == 28:
        return 'g'
    if y == 29:
        return 'k'
    # apron with checks over a brown skirt, a green zigzag hem
    if y >= 40:
        return 'g' if (y + abs((x % 4) - 2)) % 3 else 'y'
    if 11 <= x <= 21 and y <= 38:
        if x in (11, 21) or y == 38:
            return 'k'
        return 'g' if (x // 2 + y // 2) % 2 else 'w'
    return 'o' if x % 3 else 'k'


def o_pat_gruen(x, y):
    if y <= 16:
        return 'g'
    side = x <= 10 or x >= 22
    if side and y <= 27:
        if y >= 25:
            return 'y'
        return 'g'
    if y <= 21:
        # V neckline
        if abs(x - 16) <= 21 - y:
            return 's' if abs(x - 16) < 21 - y else 'y'
        return 'g'
    if y in (22, 23):
        return 'y' if y == 22 else 'k'
    # high-waisted gown: gold lattice on green
    if (x + y) % 6 == 0 or (x - y) % 6 == 0:
        return 'y'
    if y >= 41:
        return 'y' if (x + y) % 2 else 'g'
    return 'g'


def o_pat_schellen(x, y):
    if y <= 17:
        # millstone ruff
        if y in (14, 17) or x in (12, 20) and y == 15:
            return 'k'
        return 'k' if (x % 2 == 0 and y == 16) else 'w'
    side = x <= 10 or x >= 22
    if side and y <= 27:
        if y in (26, 27):
            return 'w'
        return 'r' if y % 2 else 'y'
    if y <= 27:
        if 14 <= x <= 18:
            return 'k' if x in (14, 18) else ('y' if y % 3 == 0 else 'k')
        return 'r' if x % 2 == 0 else 'y'
    if y == 28:
        return 'k'
    if y == 29:
        return 'y'
    # gown: gold with red slashes, black front panel with keys and purse
    if 14 <= x <= 18:
        if (x, y) in ((15, 32), (15, 33), (15, 34), (16, 33), (17, 36), (17, 37)):
            return 'w'
        if 15 <= x <= 17 and 38 <= y <= 40:
            return 'o'
        return 'k'
    if y >= 42:
        return 'r'
    return 'r' if (x % 3 == 0 and (y // 3) % 2 == 0) else 'y'


OBERIN = {
    'Rot': dict(pat=o_pat_rot, rim=('y', 'k'),
                head=["y.y.y.y", "yyyyyyy", "kkkkkkk", "kssssk.", "ksksskk", "kssssk.", "ksrssk.", "kssssk.",
                      "kk.sskk", "kk...kk", "kk...kk"],
                sleeve=["ss", "ss", "yy", "rr", "rr", "rr", "rr", "rr", "rr"]),
    'Eichel': dict(pat=o_pat_eichel, rim=('y', 'k'),
                   head=["..g.y..", ".ggg.y.", "ggggggg", "yyyyyyy", "yssssy.", "ysksssy", "yssssy.", "ysrssy.",
                         "yssssy.", "y.sss.y", "y.....y", "r.....r"],
                   sleeve=["ss", "ss", "ss", "ss", "ss", "ww", "ww", "ww", "ww"]),
    'Gruen': dict(pat=o_pat_gruen, rim=('y', 'k'),
                  head=["....g..", "...gg..", "..ggy..", ".gyggw.", "yyyyyyw", ".sssskw", ".ksssk.w", ".sssssw",
                        ".srsss.w", ".sssss..", "..sss..w"],
                  sleeve=["ss", "ss", "yy", "gg", "gg", "gg", "gg", "gg", "gg"]),
    'Schellen': dict(pat=o_pat_schellen, rim=('k', 'y'),
                     head=["..kkkk.", ".kwwwwk", "kwwwwwk", "kwssssk", "kwskssk", "kwssssk", "kwsrssk", ".kssssk",
                           "..ssss.", "..ss...", "..ss..."],
                     sleeve=["ss", "ss", "ww", "rr", "yy", "rr", "yy", "rr", "yy"]),
}


def oberin(suit):
    s = OBERIN[suit]
    g = Grid()
    m = o_mask()
    g.fill(m, s['pat'])
    rim(g, m, *s['rim'])
    g.hline(12, 14, 45, 'k')
    g.hline(18, 20, 45, 'k')
    g.stamp(s['head'], 13, 3)
    g.stamp(BIG[suit], 23, 1)
    g.stamp(s['sleeve'], 25, 9)
    g.stamp(["ss", "ss"], 6, 26)
    g.index('O', suit)
    return g


# ------------------------------------------------------------ Unter pose
def u_pat_rot(x, y):
    if y <= 14:
        return 'w'
    side = x <= 10 or x >= 22
    if side:
        if y in (20, 21):
            return 'y'
        if y == 22:
            return 'k'
        if y == 28:
            return 'w'
        return 'b' if y < 20 else 'r'
    if y in (26, 28):
        return 'k'
    if y == 27:
        return 'y'
    if y > 28:
        return 'r' if ((x // 2 + y // 2) % 2) else 'b'
    return 'r' if ((x // 2 + y // 2) % 2) else 'y'


def u_pat_eichel(x, y):
    if y <= 14:
        return 'w'
    side = x <= 10 or x >= 22
    if side:
        if y >= 27:
            return 'g' if y <= 28 else 'o'
        return 'o'
    if y in (26, 28):
        return 'k'
    if y == 27:
        return 'y' if x == 16 else 'o'
    if y > 28:
        # leather breeches with braces
        return 'g' if x in (13, 19) else ('k' if y == 34 else 'h')
    # loden lapels in a V, horn buttons
    if abs(x - 16) <= (y - 14) // 2 + 1 and abs(x - 16) >= (y - 14) // 2 - 1:
        return 'g'
    if abs(x - 16) < (y - 14) // 2 - 1:
        return 'y' if (x == 16 and y % 3 == 0) else 'o'
    return 'w' if abs(x - 16) <= 1 else 'o'


def u_pat_gruen(x, y):
    if y <= 19:
        # hood cape, dagged
        if y == 19:
            return 'y' if x % 2 else '.'
        return 'g' if y < 18 else ('y' if x % 2 == 0 else 'g')
    side = x <= 10 or x >= 22
    if side:
        return 'y' if y in (27, 28) else 'g'
    if y in (26, 28):
        return 'k'
    if y == 27:
        return 'y'
    if y > 28:
        return 'g' if y < 33 else ('y' if (x + y) % 2 else '.')
    return 'y' if (x + y) % 6 == 0 or (x - y) % 6 == 0 else 'g'


def u_pat_schellen(x, y):
    if y <= 14:
        return 'w'
    side = x <= 10 or x >= 22
    if side:
        if y in (22, 23):
            return 'k'
        return 'r' if (x + y // 2) % 2 else 'y'
    if y in (26, 28):
        return 'k'
    if y == 27:
        return 'y'
    if y > 28:
        return 'r' if x % 2 == 0 else 'y'
    if (x % 4 == 1 and y % 4 in (0, 1)):
        return 'r'
    return 'y'


def legs_rot(g):
    for y in range(35, 45):
        for x in range(10, 15):
            g.px(x, y, 'r' if (y // 2) % 2 else 'y')
        for x in range(18, 23):
            g.px(x, y, 'b')
    g.hline(10, 14, 39, 'k')
    g.hline(18, 22, 39, 'k')


def legs_eichel(g):
    for y in range(35, 45):
        for x in list(range(10, 15)) + list(range(18, 23)):
            g.px(x, y, 'g' if y in (37, 38) else ('h' if y <= 36 else ('w' if y < 43 or x % 5 else 'k')))
    g.hline(10, 14, 35, 'k')
    g.hline(18, 22, 35, 'k')


def legs_gruen(g):
    for y in range(35, 45):
        for x in range(10, 15):
            g.px(x, y, 'g')
        for x in range(18, 23):
            g.px(x, y, 'd')
    g.px(15, 44, 'k')
    g.px(23, 44, 'k')


def legs_schellen(g):
    for y in range(35, 45):
        for x in range(10, 15):
            g.px(x, y, 'r' if (y // 2) % 2 else 'y')
        for x in range(18, 23):
            g.px(x, y, 'b')
    for x in range(9, 24):
        if x not in (15, 16, 17):
            g.px(x, 35, 'r' if x % 2 else 'y')
            g.px(x, 36, 'k')


UNTER = {
    'Rot': dict(pat=u_pat_rot, rim=('y', 'k'), legs=legs_rot,
                head=["..yy....", ".yy.....", "rrrrrrrr", "rrrrrrrr", ".kkssss.", ".ksssks.", ".kssssss", ".kkssrs.",
                      "..ssss.."],
                pole=["..b.", ".bbb", "bbbk", "bbkk", ".bk.", "..k.", "..k.", "..k.", "..k.", "..k.", ".ss.", ".ss.",
                      "..k.", "..k.", "..k.", "..k.", "..k.", "..k.", "..k.", "..k.", "..k."]),
    'Eichel': dict(pat=u_pat_eichel, rim=('y', 'k'), legs=legs_eichel,
                   head=["..kk....", "...kg...", "..ggggg.", ".rrrrrr.", "gggggggg", ".hhssss.", ".hsssks.", ".hhssss",
                         ".hhhrhh.", "..hhhh.."],
                   pole=["..k.", ".kwk", ".kwk", ".kwk", "..k.", "kkkk", "..h.", "..h.", "..h.", "..h.", ".ss.", ".ss.",
                         "..h.", "..h.", "..h.", "..h.", "..h.", "..h.", "..h.", "..h.", "..h."]),
    'Gruen': dict(pat=u_pat_gruen, rim=('y', 'k'), legs=legs_gruen,
                  head=["g.......", "gg.gggg.", ".gggggggg", "gggssssg", "ggsssksg", "ggssssss", "ggsssrsg", "gggssssg",
                        ".gggggg."],
                  pole=["..o.", "...o", "...o", "..ko", "..ko", "..k.o", "..k.o", "..k.o", "..k.o", "..k.o", ".ss.o",
                        ".ss.o", "..k.o", "..k.o", "..k.o", "..k.o", "..ko", "..ko", "...o", "...o", "..o."]),
    'Schellen': dict(pat=u_pat_schellen, rim=('k', 'y'), legs=legs_schellen,
                     head=["...w.y.w...", "..wwyyww...", "rrrrrrrrrrr", ".rryrryrr..", "..kkssss.", "..ksssks.",
                           "..kssssss", "..kkkkkk.", "...kkkk.."],
                     pole=[".wk.", "kwwk", "wwwk", ".kk.", "..k.", "..k.", "..k.", "..k.", "..k.", "..k.", ".ss.",
                           ".ss.", "..k.", "..k.", "..k.", "..k.", "..k.", "..k.", "..k.", "..k.", "..k."]),
}


def unter(suit):
    s = UNTER[suit]
    g = Grid()
    m = frame_mask(13, 34, x0=7, x1=25, steps=2)
    g.fill(m, s['pat'])
    rim(g, m, *s['rim'])
    s['legs'](g)
    g.hline(9, 14, 45, 'k')
    g.hline(18, 23, 45, 'k')
    g.hline(10, 14, 44, 'k')
    g.hline(18, 22, 44, 'k')
    hx = 10 if suit == 'Schellen' else 12
    g.stamp(s['head'], hx, 3)
    g.stamp(s['pole'], 25, 2)
    g.stamp(["ss", "ss"], 5, 28)
    g.stamp(MID[suit], 3, 31)
    g.index('U', suit)
    return g


def number(suit, n):
    g = Grid()
    big = BIG[suit]
    mid = MID[suit]
    if n == 10:
        pts = [(c, y) for c in (8, 20) for y in (8, 15, 22, 29, 36)]
        for x, y in pts:
            g.stamp(mid, x, y)
    else:
        if n == 7:
            pts = [(c, y) for c in (7, 19) for y in (9, 22, 35)] + [(13, 15)]
        elif n == 8:
            pts = [(c, y) for c in (7, 19) for y in (8, 17, 26, 35)]
        else:
            pts = [(c, y) for c in (7, 19) for y in (8, 17, 26, 35)] + [(13, 21)]
        for x, y in pts:
            g.stamp(big, x, y)
    g.index(str(n), suit)
    return g


FIG = {'K': koenig, 'O': oberin, 'U': unter}

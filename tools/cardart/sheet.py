"""Render the deck as one PNG for a quick look (needs Pillow).

    python3 tools/cardart/sheet.py /tmp/deck.png [scale]

Rows: the 32 faces (Eichel, Gruen, Rot, Schellen; columns Daus, Koenig,
Oberin, Unter, 10 to 7), the same 32 as "Untot", the eight open
Wandelkarte faces and the 32 settled ones. Drawn from the same grids and palette as the
TypeScript module emit.py writes (corner index included).
"""

import os
import sys

sys.dont_write_bytecode = True
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

from PIL import Image  # noqa: E402

import untot  # noqa: E402
import wild  # noqa: E402
from emit import RANKS, SUITS, UNDEAD_KEYS, face, rows, undead_face  # noqa: E402
from pix import BG, GH, GW, PAL  # noqa: E402

ORDER = ['A', 'K', 'Q', 'J', 'T', '9', '8', '7']
PALETTE = dict(PAL)
PALETTE.update(untot.ROT_PAL)
PALETTE.update({UNDEAD_KEYS[k]: v for k, v in untot.EXTRA.items() if k in UNDEAD_KEYS})


def image(grid_rows, s):
    im = Image.new('RGB', (GW * s, int((GH + 0.5) * s)), BG)
    px = im.load()
    for y, row in enumerate(grid_rows):
        for x, c in enumerate(row):
            if c == '.':
                continue
            rgb = tuple(int(PALETTE[c.lower()][i:i + 2], 16) for i in (1, 3, 5))
            for j in range(s):
                for i in range(s):
                    px[x * s + i, y * s + j] = rgb
    return im


if __name__ == '__main__':
    out = sys.argv[1] if len(sys.argv) > 1 else 'deck.png'
    s = int(sys.argv[2]) if len(sys.argv) > 2 else 6
    grid = [[image(rows(suit, RANKS[r], face(suit, RANKS[r])), s) for r in ORDER] for suit in SUITS.values()]
    grid += [[image(rows(suit, RANKS[r], undead_face(suit, RANKS[r])), s) for r in ORDER] for suit in SUITS.values()]
    grid += [[image(rows('Wild', RANKS[r], wild.card(RANKS[r])), s) for r in ORDER]]
    grid += [[image(rows(suit, RANKS[r], wild.settled(suit, RANKS[r])), s) for r in ORDER] for suit in SUITS.values()]
    w, h = grid[0][0].size
    sheet = Image.new('RGB', (8 * (w + 8) + 8, len(grid) * (h + 8) + 8), (31, 77, 43))
    for j, row in enumerate(grid):
        for i, im in enumerate(row):
            sheet.paste(im, (8 + i * (w + 8), 8 + j * (h + 8)))
    sheet.save(out)
    print(f'wrote {out}')

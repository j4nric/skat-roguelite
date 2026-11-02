"""Render all 32 faces as one PNG for a quick look (needs Pillow).

    python3 tools/cardart/sheet.py /tmp/deck.png [scale]

Rows: Eichel, Gruen, Rot, Schellen; columns: Daus, Koenig, Oberin, Unter,
10, 9, 8, 7. Same grids as the TypeScript module emit.py writes.
"""

import os
import sys

sys.dont_write_bytecode = True
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

from PIL import Image  # noqa: E402

from emit import RANKS, SUITS, face  # noqa: E402

ORDER = ['A', 'K', 'Q', 'J', 'T', '9', '8', '7']

if __name__ == '__main__':
    out = sys.argv[1] if len(sys.argv) > 1 else 'deck.png'
    s = int(sys.argv[2]) if len(sys.argv) > 2 else 6
    ims = [[face(suit, RANKS[r]).png(None, s) for r in ORDER] for suit in SUITS.values()]
    w, h = ims[0][0].size
    sheet = Image.new('RGB', (8 * (w + 8) + 8, 4 * (h + 8) + 8), (31, 77, 43))
    for j, row in enumerate(ims):
        for i, im in enumerate(row):
            sheet.paste(im, (8 + i * (w + 8), 8 + j * (h + 8)))
    sheet.save(out)
    print(f'wrote {out}')

#!/usr/bin/env python3
"""Build the pixel fonts of the web app from Kenney's CC0 font pack.

    python3 scripts/make-fonts.py <dir with the Kenney TTFs>

Needs fontTools and brotli (pip install fonttools brotli). The input is the
Kenney Fonts pack (https://kenney.nl/assets/kenney-fonts, CC0), files
"Kenney Mini.ttf" and "Kenney Mini Square.ttf" (also mirrored in
github.com/ereborstudios/kenney-fonts). Output: WOFF2 files in
src/assets/fonts/, reduced to the characters of German texts.

Fixes on top of the originals (CC0 allows any change):
- the lowercase umlauts (ä, ö, ü) reuse the capital glyphs in the pack;
  they are rebuilt from the lowercase letter plus the dots of the capital,
  moved down to sit one pixel above the x-height,
- the lowercase l of Kenney Mini is a small capital L at x-height, which
  makes "links" read like "unks"; it gets the plain stem of the capital I,
- the minus sign (U+2212) and the en dash (U+2013) are missing; they get
  the hyphen's outline.
"""

import os
import sys

from fontTools import subset
from fontTools.pens.recordingPen import DecomposingRecordingPen
from fontTools.pens.transformPen import TransformPen
from fontTools.pens.ttGlyphPen import TTGlyphPen
from fontTools.pens.boundsPen import BoundsPen
from fontTools.ttLib import TTFont

FONTS = {
    "Kenney Mini.ttf": "kenney-mini.woff2",
    "Kenney Mini Square.ttf": "kenney-mini-square.woff2",
}

UMLAUTS = {"adieresis": ("a", "Adieresis", "A"), "odieresis": ("o", "Odieresis", "O"), "udieresis": ("u", "Udieresis", "U")}

# Latin-1 plus the punctuation the texts use.
UNICODES = list(range(0x20, 0x7F)) + list(range(0xA0, 0x100)) + [0x2013, 0x2014, 0x2018, 0x2019, 0x201A, 0x201C, 0x201D, 0x201E, 0x2022, 0x2026, 0x2212, 0x20AC]


def contours(font, name):
    """The decomposed outline of a glyph as a list of contours (pen calls)."""
    pen = DecomposingRecordingPen(font.getGlyphSet())
    font.getGlyphSet()[name].draw(pen)
    out, cur = [], []
    for op, args in pen.value:
        cur.append((op, args))
        if op in ("closePath", "endPath"):
            out.append(cur)
            cur = []
    return out


def bounds(font, name):
    pen = BoundsPen(font.getGlyphSet())
    font.getGlyphSet()[name].draw(pen)
    return pen.bounds


def replay(pen, calls):
    for op, args in calls:
        getattr(pen, op)(*args)


def contour_ymin(calls):
    ys = [pt[1] for _, args in calls for pt in args if pt is not None]
    return min(ys) if ys else 0


def fix_umlauts(font):
    glyf = font["glyf"]
    for target, (lower, upper_uml, upper) in UMLAUTS.items():
        cap_top = bounds(font, upper)[3]
        x_top = bounds(font, lower)[3]
        dots = [c for c in contours(font, upper_uml) if contour_ymin(c) >= cap_top]
        if not dots:
            continue
        pen = TTGlyphPen(font.getGlyphSet())
        replay(pen, [call for c in contours(font, lower) for call in c])
        shifted = TransformPen(pen, (1, 0, 0, 1, 0, x_top - cap_top))
        for c in dots:
            replay(shifted, c)
        glyf[target] = pen.glyph()
        font["hmtx"][target] = font["hmtx"][lower]


def fix_l(font):
    """A small-capital l (lower than the capitals) becomes the stem of I."""
    if bounds(font, "l")[3] >= bounds(font, "I")[3]:
        return
    pen = TTGlyphPen(font.getGlyphSet())
    replay(pen, [call for c in contours(font, "I") for call in c])
    font["glyf"]["l"] = pen.glyph()
    font["hmtx"]["l"] = font["hmtx"]["I"]


def add_dashes(font):
    for table in font["cmap"].tables:
        if not table.isUnicode():
            continue
        hyphen = table.cmap.get(0x2D)
        if hyphen:
            table.cmap.setdefault(0x2212, hyphen)
            table.cmap.setdefault(0x2013, hyphen)


def main():
    if len(sys.argv) != 2:
        sys.exit(__doc__)
    src = sys.argv[1]
    out_dir = os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "src", "assets", "fonts")
    for name, out in FONTS.items():
        font = TTFont(os.path.join(src, name))
        fix_umlauts(font)
        fix_l(font)
        add_dashes(font)
        options = subset.Options()
        options.flavor = "woff2"
        options.layout_features = []
        options.name_IDs = ["*"]
        sub = subset.Subsetter(options)
        sub.populate(unicodes=UNICODES)
        sub.subset(font)
        font.flavor = "woff2"
        path = os.path.join(out_dir, out)
        font.save(path)
        print(f"{path}: {os.path.getsize(path)} bytes")


if __name__ == "__main__":
    main()

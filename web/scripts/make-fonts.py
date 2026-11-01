#!/usr/bin/env python3
"""Build the web font of the app: VT323 (Peter Hull, SIL OFL 1.1) as WOFF2.

    python3 scripts/make-fonts.py [path to VT323-Regular.ttf]

Needs fontTools and brotli (pip install fonttools brotli). Without an
argument the TTF is downloaded from the Google Fonts repository:

    https://raw.githubusercontent.com/google/fonts/main/ofl/vt323/VT323-Regular.ttf
    https://raw.githubusercontent.com/google/fonts/main/ofl/vt323/OFL.txt

Both files are checked against the SHA-256 below, so the output stays the
same as long as the source does not change. Output: src/assets/fonts/
vt323.woff2, reduced to Latin-1 plus the punctuation of German texts, and
the licence next to it as OFL-VT323.txt (the OFL asks for the licence to
travel with the font). The outlines are not changed; the font declares no
Reserved Font Name, so the subset keeps the name VT323.
"""

import hashlib
import os
import shutil
import sys
import tempfile
import urllib.request

from fontTools import subset
from fontTools.ttLib import TTFont

BASE = "https://raw.githubusercontent.com/google/fonts/main/ofl/vt323/"
SOURCES = {
    "VT323-Regular.ttf": "cf4de751ada78ceac033dbe16a687742939995b77bc2a052ae17a4957958594d",
    "OFL.txt": "27d9af34210253e7ca1251fbace86c6f65b40031d6ce1a75493a1b2093631298",
}

# Latin-1 plus the punctuation the texts use (de.ts and core/content/*.ron):
# en and em dash, quotes „“ ‚‘ ’ ”, bullet, ellipsis, minus, euro.
UNICODES = list(range(0x20, 0x7F)) + list(range(0xA0, 0x100)) + [
    0x2013, 0x2014, 0x2018, 0x2019, 0x201A, 0x201C, 0x201D, 0x201E, 0x2022, 0x2026, 0x2212, 0x20AC,
]


def sha256(path):
    with open(path, "rb") as f:
        return hashlib.sha256(f.read()).hexdigest()


def fetch(name, into):
    path = os.path.join(into, name)
    with urllib.request.urlopen(BASE + name) as r, open(path, "wb") as f:
        shutil.copyfileobj(r, f)
    return path


def main():
    if len(sys.argv) > 2:
        sys.exit(__doc__)
    out_dir = os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "src", "assets", "fonts")
    with tempfile.TemporaryDirectory() as tmp:
        if len(sys.argv) == 2:
            ttf = sys.argv[1]
            ofl = os.path.join(os.path.dirname(os.path.abspath(ttf)), "OFL.txt")
            if not os.path.exists(ofl):
                ofl = fetch("OFL.txt", tmp)
        else:
            ttf = fetch("VT323-Regular.ttf", tmp)
            ofl = fetch("OFL.txt", tmp)
        for path, name in ((ttf, "VT323-Regular.ttf"), (ofl, "OFL.txt")):
            if sha256(path) != SOURCES[name]:
                print(f"warning: {name} differs from the known version (SHA-256 {sha256(path)})", file=sys.stderr)

        # Keep the timestamp of the source, so equal input gives equal bytes.
        font = TTFont(ttf, recalcTimestamp=False)
        cmap = font.getBestCmap()
        missing = [f"U+{u:04X}" for u in UNICODES if u not in cmap]
        if missing:
            print(f"warning: VT323 has no glyph for {', '.join(missing)}", file=sys.stderr)
        options = subset.Options()
        options.flavor = "woff2"
        options.layout_features = []
        options.name_IDs = ["*"]
        sub = subset.Subsetter(options)
        sub.populate(unicodes=UNICODES)
        sub.subset(font)
        font.flavor = "woff2"
        path = os.path.join(out_dir, "vt323.woff2")
        font.save(path)
        print(f"{path}: {os.path.getsize(path)} bytes")
        shutil.copyfile(ofl, os.path.join(out_dir, "OFL-VT323.txt"))


if __name__ == "__main__":
    main()

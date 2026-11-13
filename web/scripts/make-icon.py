#!/usr/bin/env python3
"""Draw the app icon from the game's own pixel card art:

    python3 scripts/make-icon.py        (npm run icon)

Needs Pillow (pip install pillow). Source: the Wandel-König
(ART_WILD['K']) in src/ui/pixelcards.gen.ts, as a portrait: crown, face,
sceptre and the upper mosaic robe, without the card's frame and without
the corner index, on the card's own paper that runs to every edge. Every
art pixel becomes a square of whole screen pixels (nearest neighbour), so
the icon stays crisp; nothing is added to the art, only paper around it.

Output:

- public/icon-512.png (x16), public/icon-192.png (x6): the 32 x 32 portrait.
- public/apple-touch-icon.png: 180 px, the portrait at x6 less one art
  pixel of paper left and right and the bottom row of the robe, which
  runs off the edge anyway (180 is no multiple of 32; the crown keeps
  its margin).
- public/icon-maskable-512.png: a wider window of the same art (42 x 42 at
  x12), so the portrait sits in the middle 75 % and Android's mask (the
  safe zone is the middle 80 % circle) never cuts the crown or the face.
  The art below the portrait is the card's own robe and hand.
- public/favicon.svg: the head alone (16 x 16: crown, face, collar).
- Then scripts/make-ios-assets.py draws the iOS icon (1024, from
  icon-512.png at x2) and the launch image.

All images are RGB without alpha; the App Store refuses an icon with one.
"""

import importlib.util
import re
from pathlib import Path

from PIL import Image

HERE = Path(__file__).resolve().parent
WEB = HERE.parent
PUBLIC = WEB / "public"
SOURCE = WEB / "src" / "ui" / "pixelcards.gen.ts"
FIGURE = "K"  # ART_WILD key: the Wandel-König

# Windows on the art (column, row, width, height). The art is 33 x 49;
# cells outside it are paper.
PORTRAIT = (2, 1, 32, 32)  # crown on row 3, the figure (cols 6..29) centred
MASKABLE = (-3, -4, 42, 42)  # the portrait plus 5 cells on every side
FAVICON = (9, 2, 16, 16)  # crown, face, collar
# The left hand (cols 3..5 from row 30) holds an orb below the portrait;
# the portrait would cut it to a stray patch, so there it stays paper.
CUT_IN_PORTRAIT = {(x, y) for x in range(3, 6) for y in range(30, 33)}


def hexrgb(value: str) -> tuple[int, int, int]:
    value = value.lstrip("#")
    return tuple(int(value[i : i + 2], 16) for i in (0, 2, 4))


def load_art() -> tuple[list[str], dict[str, tuple[int, int, int]], tuple[int, int, int]]:
    text = SOURCE.read_text(encoding="utf-8")
    paper = hexrgb(re.search(r"ART_PAPER = '(#[0-9a-f]{6})'", text).group(1))
    block = text.split("export const ART_PALETTE")[1].split("};")[0]
    palette = {k: hexrgb(v) for k, v in re.findall(r"^\s+'?([a-z0-9])'?: '(#[0-9a-f]{6})',", block, re.M)}
    wild = text.split("export const ART_WILD")[1].split("\n};")[0]
    match = re.search(rf"^  '?{FIGURE}'?: \[\n((?:    '.*',\n)+)  \]", wild, re.M)
    rows = re.findall(r"'(.*)'", match.group(1))
    return rows, palette, paper


def window(art, box, background, cut=frozenset()) -> Image.Image:
    """One image pixel per art pixel. Paper ('.'), the corner index
    (capitals) and cells outside the art take the background."""
    rows, palette, _ = art
    x0, y0, w, h = box
    img = Image.new("RGB", (w, h), background)
    px = img.load()
    for y in range(h):
        for x in range(w):
            ax, ay = x0 + x, y0 + y
            if (ax, ay) in cut or not (0 <= ay < len(rows) and 0 <= ax < len(rows[ay])):
                continue
            key = rows[ay][ax]
            if key in palette:  # lower case only: the picture
                px[x, y] = palette[key]
    return img


def scaled(img: Image.Image, factor: int) -> Image.Image:
    return img.resize((img.width * factor, img.height * factor), Image.NEAREST)


def portrait(art, background) -> Image.Image:
    return window(art, PORTRAIT, background, CUT_IN_PORTRAIT)


def favicon_svg(img: Image.Image, background) -> str:
    """The background as one rectangle, then runs of equal colour per row
    as rectangles; crisp at any size."""
    px = img.load()
    parts = {}
    for y in range(img.height):
        x = 0
        while x < img.width:
            colour = px[x, y]
            end = x
            while end < img.width and px[end, y] == colour:
                end += 1
            if colour != background:
                parts.setdefault(colour, []).append(f"M{x} {y}h{end - x}v1h-{end - x}z")
            x = end
    bg = f'<rect width="{img.width}" height="{img.height}" fill="#{background[0]:02x}{background[1]:02x}{background[2]:02x}"/>'
    paths = bg + "".join(
        f'<path d="{"".join(d)}" fill="#{c[0]:02x}{c[1]:02x}{c[2]:02x}"/>' for c, d in parts.items()
    )
    return (
        f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {img.width} {img.height}" '
        f'shape-rendering="crispEdges">{paths}</svg>\n'
    )


def main() -> None:
    art = load_art()
    paper = art[2]
    master = portrait(art, paper)

    scaled(master, 16).save(PUBLIC / "icon-512.png", optimize=True)
    scaled(master, 6).save(PUBLIC / "icon-192.png", optimize=True)
    scaled(master, 6).crop((6, 0, 186, 180)).save(PUBLIC / "apple-touch-icon.png", optimize=True)

    wide = scaled(window(art, MASKABLE, paper), 12)  # 504 x 504
    maskable = Image.new("RGB", (512, 512), paper)
    maskable.paste(wide, (4, 8))  # flush with the bottom: the robe runs off the edge
    maskable.save(PUBLIC / "icon-maskable-512.png", optimize=True)

    (PUBLIC / "favicon.svg").write_text(favicon_svg(window(art, FAVICON, paper), paper), encoding="utf-8")
    print(f"icons written to {PUBLIC}")

    spec = importlib.util.spec_from_file_location("make_ios_assets", HERE / "make-ios-assets.py")
    ios = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(ios)
    ios.main()


if __name__ == "__main__":
    main()

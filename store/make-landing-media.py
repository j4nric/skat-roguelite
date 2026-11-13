#!/usr/bin/env python3
"""Small web versions of the store assets for the landing page (web/start.html,
web/en/start.html) and the full-size key art for the press kit:

    python3 store/make-landing-media.py PREVIEW_DIR HEADER_PNG

PREVIEW_DIR holds preview-<de|en>.mp4 and poster-<de|en>.png from
make-preview.mjs (store/preview/, not in the repo); HEADER_PNG is the wide
key art (3840 x 1646, the fan of four Unter). The screenshots come from
store/screenshots/ (in the repo). Needs ffmpeg and Pillow.

Output (all committed):

- web/public/landing/hero-<lang>.mp4: the preview video at 432 px wide,
  H.264 without sound, +faststart, about 1 MB. A muted loop; the landing
  page plays it inline.
- web/public/landing/poster-<lang>.webp: its poster frame (the stamp 342).
- web/public/landing/<lang>-<shot>.webp: four store screenshots without
  their caption line (the page sets its own headings), 480 px wide.
- web/public/landing/header.webp: the key art at 1600 px for the page.
- store/art/header-3840x1646.png: the key art in full size for the press
  kit, reduced to its own 256 colours (the art has about 2000, the
  difference is invisible; half the bytes).

The press kit's screenshots and icon are not copied here: the build takes
them from store/screenshots/ and the iOS icon (web/presskit/plugin.ts).
"""

import subprocess
import sys
from pathlib import Path

from PIL import Image

ROOT = Path(__file__).resolve().parent.parent
LANDING = ROOT / "web" / "public" / "landing"
SHOTS = ROOT / "store" / "screenshots"
ART = ROOT / "store" / "art"

# Screenshots on the landing page, by the store's number.
PICKED = {"table": "01-table", "score": "02-score", "tresen": "04-tresen", "map": "06-map"}
# The store screenshots are 1320 x 2868; the caption ends above this row,
# the app's frame starts just below it.
CAPTION_CUT = 620
VIDEO_WIDTH = 432
SHOT_WIDTH = 480
HEADER_WIDTH = 1600


def video(src: Path, out: Path) -> None:
    subprocess.run(
        [
            "ffmpeg", "-v", "error", "-y", "-i", str(src), "-an",
            "-vf", f"scale={VIDEO_WIDTH}:-2:flags=lanczos",
            "-c:v", "libx264", "-profile:v", "high", "-pix_fmt", "yuv420p",
            "-crf", "33", "-preset", "veryslow", "-movflags", "+faststart",
            str(out),
        ],
        check=True,
    )


def webp(im: Image.Image, width: int, out: Path, quality: int = 85) -> None:
    height = round(im.height * width / im.width)
    im.convert("RGB").resize((width, height), Image.Resampling.LANCZOS).save(out, "WEBP", quality=quality, method=6)


def main() -> None:
    if len(sys.argv) != 3:
        sys.exit(__doc__)
    previews, header = Path(sys.argv[1]), Path(sys.argv[2])
    LANDING.mkdir(parents=True, exist_ok=True)
    ART.mkdir(parents=True, exist_ok=True)
    for lang in ("de", "en"):
        video(previews / f"preview-{lang}.mp4", LANDING / f"hero-{lang}.mp4")
        webp(Image.open(previews / f"poster-{lang}.png"), VIDEO_WIDTH, LANDING / f"poster-{lang}.webp", 80)
        for name, file in PICKED.items():
            im = Image.open(SHOTS / lang / f"{file}.png")
            webp(im.crop((0, CAPTION_CUT, im.width, im.height)), SHOT_WIDTH, LANDING / f"{lang}-{name}.webp")
    art = Image.open(header).convert("RGB")
    webp(art, HEADER_WIDTH, LANDING / "header.webp", 82)
    art.quantize(256, method=Image.Quantize.MEDIANCUT, dither=Image.Dither.NONE).save(ART / "header-3840x1646.png", optimize=True)
    for path in sorted([*LANDING.iterdir(), *ART.iterdir()]):
        print(f"{path.relative_to(ROOT)}: {path.stat().st_size // 1024} kB")


if __name__ == "__main__":
    main()

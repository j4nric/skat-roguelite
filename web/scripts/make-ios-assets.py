#!/usr/bin/env python3
"""Draw the icon and the launch image of the iOS app (ADR 0026) from the
pixel art of the web app:

    python3 scripts/make-ios-assets.py

Needs Pillow (pip install pillow). Source: public/icon-maskable-512.png,
the card fan on the beer mat that fills its square (the PWA's maskable
icon). Every pixel is scaled by a whole number with nearest neighbour, so
the art stays crisp.

Output (in ios/App/App/Assets.xcassets/):

- AppIcon.appiconset/AppIcon-1024.png: 1024 x 1024, no alpha (the App
  Store refuses an icon with an alpha channel); iOS rounds the corners.
- Splash.imageset/splash-2732x2732.png: the icon at 1024 px in the middle
  of the app's dark background (#1b1e21). LaunchScreen.storyboard shows
  it with "aspect fill", so on a phone the icon is about a third of the
  width, and the web view starts on the same colour.
"""

from pathlib import Path

from PIL import Image

HERE = Path(__file__).resolve().parent
WEB = HERE.parent
ASSETS = WEB.parent / "ios" / "App" / "App" / "Assets.xcassets"
SOURCE = WEB / "public" / "icon-maskable-512.png"
BACKGROUND = (0x1B, 0x1E, 0x21)
SPLASH = 2732


def main() -> None:
    art = Image.open(SOURCE).convert("RGBA")
    flat = Image.new("RGB", art.size, BACKGROUND)
    flat.paste(art, mask=art.getchannel("A"))
    icon = flat.resize((1024, 1024), Image.NEAREST)

    icon_dir = ASSETS / "AppIcon.appiconset"
    for old in icon_dir.glob("*.png"):
        old.unlink()
    icon.save(icon_dir / "AppIcon-1024.png", optimize=True)

    splash_dir = ASSETS / "Splash.imageset"
    for old in splash_dir.glob("*.png"):
        old.unlink()
    splash = Image.new("RGB", (SPLASH, SPLASH), BACKGROUND)
    offset = (SPLASH - icon.width) // 2
    splash.paste(icon, (offset, offset))
    splash.save(splash_dir / "splash-2732x2732.png", optimize=True)
    print(f"icon and launch image written to {ASSETS}")


if __name__ == "__main__":
    main()

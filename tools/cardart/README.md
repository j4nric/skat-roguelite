# Kartenbilder „Pixelstil“

Das Blatt der Web-App (32 Karten, deutsches Blatt) ist eigene Pixelgrafik und wird von diesen Skripten erzeugt. Die Web-App liest nur das erzeugte Modul `web/src/ui/pixelcards.gen.ts`; beim Bauen läuft kein Python.

- `pix.py`: Raster (33 × 49,5 Bildpunkte, ein Bildpunkt = 2 × 2 CSS-Pixel bei 66 × 99), Palette mit neun Farben (Ocker nur für die Eckzeichen von Schellen), Pixelbuchstaben und Farbzeichen, Eckzeichen oben links und unten rechts (aufrecht).
- `cards2.py`: König, Oberin und Unter je Farbe (gleiche Pose je Rang, eigener Hof je Farbe), Zahlkarten 10 bis 7.
- `aces2.py`: die vier Daus.
- `emit.py`: schreibt das TypeScript-Modul. Je Karte eine Zeichenkette pro Zeile: Kleinbuchstabe = Bild, Großbuchstabe = Eckzeichen (gleiche Palette), Punkt = Papier. Die Web-App trennt beide Ebenen (`web/src/ui/art.ts`), damit Markierungen der Kartenmodifikationen zwischen Bild und Eckzeichen liegen können.
- `sheet.py`: alle 32 Karten als PNG zum Anschauen.

## Neu erzeugen

Braucht Python 3 mit `numpy` (für `sheet.py` zusätzlich Pillow).

```sh
python3 tools/cardart/emit.py           # schreibt web/src/ui/pixelcards.gen.ts
python3 tools/cardart/emit.py --check   # meldet, ob die Datei veraltet ist
python3 tools/cardart/sheet.py /tmp/deck.png 6
```

Die erzeugte Datei wird mit eingecheckt. Nach einer Änderung an den Bildern die Oberfläche mit Playwright prüfen (Handy hochkant und Desktop), siehe `CLAUDE.md`.

Lizenz: selbst gezeichnet für dieses Projekt, siehe `web/LICENSES.md`.

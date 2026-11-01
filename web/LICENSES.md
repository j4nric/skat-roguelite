# Lizenzen und Quellen der Web-App

Erlaubt sind nur Assets unter CC0 oder CC-BY (DESIGN 16, CLAUDE.md). Jede fremde Quelle steht hier und im Abspann.

## Grafik, Schrift, Klang

| Was | Datei | Autor | Lizenz |
|---|---|---|---|
| Kartenbilder des deutschen Blatts im Pixelstil (Farbzeichen als 8 × 8 Pixel, Zahlkarten, Daus, König, Ober, Unter, Kartenrücken, Blanko, Markierungen der Kartenmodifikationen) | `src/ui/art.ts`, `src/ui/cards.ts` | selbst gezeichnet für dieses Projekt nach der Stilprobe `docs/design/stil-a4-mockup.dc.html` | Teil dieses Projekts |
| Figuren (Stammgäste, Wirt, Bosse), Gegenstände am Tresen; beim Start zu Pixelbildern verkleinert (`src/ui/pixelate.ts`) | `src/ui/people.ts`, `src/ui/goods.ts` | selbst gezeichnet für dieses Projekt | Teil dieses Projekts |
| Favicon | `public/favicon.svg` | selbst gezeichnet | Teil dieses Projekts |
| Pixelschriften „Kenney Mini“ (Text) und „Kenney Mini Square“ (Beschriftungen, Kartenecken), auf Latin-1 reduziert, WOFF2; Kleinbuchstaben ä, ö, ü und l nachgebessert, Minus und Halbgeviertstrich ergänzt | `src/assets/fonts/kenney-mini.woff2`, `src/assets/fonts/kenney-mini-square.woff2` | Kenney (www.kenney.nl), Paket „Kenney Fonts“ (https://kenney.nl/assets/kenney-fonts), bezogen über den Spiegel github.com/ereborstudios/kenney-fonts | CC0 1.0, Text in `src/assets/fonts/Kenney-License.txt` |
| Klänge: Karte legen, Stich, Kreide-Tick, Gläserklirren (Relikt), Groschen, Klopfen (Ergebnis), Sieg und Niederlage (Akkordeon) | `src/assets/sounds/*.mp3` | selbst synthetisiert mit `scripts/make-sounds.mjs` (Rauschen, Sinus-Teiltöne, kleine Akkordeon-Synthese), keine fremden Aufnahmen | Teil dieses Projekts |

Die Schriften baut `scripts/make-fonts.py` (braucht fontTools und brotli) aus den TTF-Dateien des Kenney-Pakets: Sie werden auf die Zeichen für deutsche Texte reduziert, die Kleinbuchstaben ä, ö und ü bekommen eigene Punkte (im Paket sind es die Großbuchstaben), das kleine l bekommt einen vollen Strich, Minus und Halbgeviertstrich werden ergänzt. CC0 erlaubt jede Änderung; die Namensnennung ist freiwillig und steht trotzdem hier und im Abspann. Die Klänge lassen sich mit `node scripts/make-sounds.mjs` (braucht ffmpeg) jederzeit neu erzeugen.

Dieselben Quellen nennt der Abspann im Spiel (Startseite und Menü, Texte in `src/i18n/de.ts`).

Die App lädt zur Laufzeit nichts von fremden Servern.

## Code, der mit ausgeliefert wird

Der Spielkern (`skat_core`) läuft als WebAssembly. Darin stecken diese Rust-Crates, dazu der von `wasm-bindgen` erzeugte JavaScript-Kleber:

| Crate | Version | Lizenz |
|---|---|---|
| wasm-bindgen (inkl. erzeugtem JS-Kleber) | 0.2.130 | MIT OR Apache-2.0 |
| serde, serde_core | 1.0.229 | MIT OR Apache-2.0 |
| serde_json | 1.0.154 | MIT OR Apache-2.0 |
| ron | 0.8.1 | MIT OR Apache-2.0 |
| base64 | 0.21.7 | MIT OR Apache-2.0 |
| bitflags | 2.13.2 | MIT OR Apache-2.0 |
| itoa | 1.0.18 | MIT OR Apache-2.0 |
| memchr | 2.8.3 | Unlicense OR MIT |
| zmij | 1.0.23 | MIT |
| cfg-if | 1.0.5 | MIT OR Apache-2.0 |

## Werkzeuge (nur beim Bauen, nicht ausgeliefert)

| Paket | Version | Lizenz |
|---|---|---|
| Vite | 8.3.5 | MIT |
| TypeScript | 7.0.2 | Apache-2.0 |
| wasm-bindgen-cli | 0.2.130 | MIT OR Apache-2.0 |
| Playwright (Smoke-Test, vorinstalliert, keine Abhängigkeit des Projekts) | – | Apache-2.0 |

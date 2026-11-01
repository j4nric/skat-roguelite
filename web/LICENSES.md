# Lizenzen und Quellen der Web-App

Erlaubt sind Assets unter freien Lizenzen: CC0, CC-BY und SIL OFL (DESIGN 16, CLAUDE.md). Jede fremde Quelle steht hier und im Abspann.

## Grafik, Schrift, Klang

| Was | Datei | Autor | Lizenz |
|---|---|---|---|
| Kartenbilder des deutschen Blatts im Pixelstil (Farbzeichen als 8 × 8 Pixel, Zahlkarten, Daus, König, Ober, Unter, Blanko, Markierungen der Kartenmodifikationen) | `src/ui/art.ts`, `src/ui/cards.ts` | selbst gezeichnet für dieses Projekt nach der Stilprobe `docs/design/stil-a4-mockup.dc.html` | Teil dieses Projekts |
| Kartenrückseite „Distel“ (Standard): Pixelgrafik nach dem Vorbild der traditionellen Altenburger Distel-Rückseite (Ockerrand, Kreuzband aus Dreiecken, Disteln, Kornblumen und Ranken); kein fremdes Asset, kein Foto oder Scan übernommen oder nachgezeichnet, Motiv und Raster werden im Code erzeugt | `src/ui/backs.ts` | selbst gezeichnet für dieses Projekt | Teil dieses Projekts |
| Figuren (Stammgäste, Wirt, Bosse), Gegenstände am Tresen; beim Start zu Pixelbildern verkleinert (`src/ui/pixelate.ts`) | `src/ui/people.ts`, `src/ui/goods.ts` | selbst gezeichnet für dieses Projekt | Teil dieses Projekts |
| Favicon | `public/favicon.svg` | selbst gezeichnet | Teil dieses Projekts |
| Pixelschrift „VT323“ für alle Texte und die Kartenecken, auf Latin-1 und die Satzzeichen deutscher Texte reduziert, WOFF2, Umrisse unverändert | `src/assets/fonts/vt323.woff2` | Peter Hull (The VT323 Project Authors), über Google Fonts: https://github.com/google/fonts/tree/main/ofl/vt323 | SIL Open Font License 1.1, Text in `src/assets/fonts/OFL-VT323.txt` |
| Klänge: Karte legen, Stich, Kreide-Tick, Gläserklirren (Relikt), Groschen, Klopfen (Ergebnis), Sieg und Niederlage (Akkordeon) | `src/assets/sounds/*.mp3` | selbst synthetisiert mit `scripts/make-sounds.mjs` (Rauschen, Sinus-Teiltöne, kleine Akkordeon-Synthese), keine fremden Aufnahmen | Teil dieses Projekts |

Die Schrift baut `scripts/make-fonts.py` (braucht fontTools und brotli): Es lädt `VT323-Regular.ttf` und `OFL.txt` aus dem Google-Fonts-Repository (https://raw.githubusercontent.com/google/fonts/main/ofl/vt323/), prüft beide gegen eine SHA-256-Summe, reduziert die Schrift auf die Zeichen deutscher Texte und legt die Lizenz daneben. Die OFL erlaubt das Reduzieren und Weitergeben mit der App, solange die Lizenz beiliegt und die Schrift nicht allein verkauft wird; VT323 hat keinen reservierten Schriftnamen. Die Klänge lassen sich mit `node scripts/make-sounds.mjs` (braucht ffmpeg) jederzeit neu erzeugen.

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

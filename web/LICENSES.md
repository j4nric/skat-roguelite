# Lizenzen und Quellen der Web-App

Erlaubt sind nur Assets unter CC0 oder CC-BY (DESIGN 16), für Schriften zusätzlich die SIL Open Font License. Jede fremde Quelle steht hier und später im Abspann.

## Grafik, Schrift, Klang

| Was | Datei | Autor | Lizenz |
|---|---|---|---|
| Kartenbilder des deutschen Blatts (Farbzeichen, Zahlkarten, Daus, König, Ober, Unter, Kartenrücken, Blanko, Markierungen der Kartenmodifikationen) | `src/ui/art.ts`, `src/ui/cards.ts` | selbst gezeichnet für dieses Projekt | Teil dieses Projekts |
| Figuren (Stammgäste, Wirt, Bosse), Gegenstände am Tresen, Regal | `src/ui/people.ts`, `src/ui/goods.ts` | selbst gezeichnet für dieses Projekt | Teil dieses Projekts |
| Holz-, Filz- und Kreide-Texturen | `src/style.css` (SVG-Rauschfilter) | selbst erstellt | Teil dieses Projekts |
| Favicon | `public/favicon.svg` | selbst gezeichnet | Teil dieses Projekts |
| Schrift „Grenze Gotisch“ (Anzeige-Schrift), auf Latin-1 reduziert, WOFF2 | `src/assets/fonts/grenze-gotisch.woff2` | Omnibus-Type, The Grenze Gotisch Project Authors (https://github.com/Omnibus-Type/Grenze-Gotisch), bezogen über github.com/google/fonts | SIL Open Font License 1.1, Text in `src/assets/fonts/OFL-GrenzeGotisch.txt` |
| Schrift „Alegreya Sans“ (Textschrift, Regular und Bold), auf Latin-1 reduziert, WOFF2 | `src/assets/fonts/alegreya-sans-*.woff2` | Juan Pablo del Peral, HT Fonts (Huerta Tipográfica), The Alegreya Sans Project Authors (https://github.com/huertatipografica/Alegreya-Sans), bezogen über github.com/google/fonts | SIL Open Font License 1.1, Text in `src/assets/fonts/OFL-AlegreyaSans.txt` |
| Klänge: Karte legen, Stich, Kreide-Tick, Gläserklirren (Relikt), Groschen, Klopfen (Ergebnis), Sieg und Niederlage (Akkordeon) | `src/assets/sounds/*.mp3` | selbst synthetisiert mit `scripts/make-sounds.mjs` (Rauschen, Sinus-Teiltöne, kleine Akkordeon-Synthese), keine fremden Aufnahmen | Teil dieses Projekts |

Die Schriften wurden mit `pyftsubset` (fontTools) auf die Zeichen für deutsche Texte reduziert. Das ist nach OFL erlaubt. Beide Schriften haben keinen reservierten Schriftnamen (Reserved Font Name), die reduzierten Fassungen dürfen ihren Namen also behalten. Die Lizenztexte liegen neben den Schriftdateien. Die Klänge lassen sich mit `node scripts/make-sounds.mjs` (braucht ffmpeg) jederzeit neu erzeugen.

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

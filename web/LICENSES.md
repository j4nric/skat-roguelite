# Lizenzen und Quellen der Web-App

Erlaubt sind nur Assets unter CC0 oder CC-BY (DESIGN 16). Jede fremde Quelle steht hier und später im Abspann.

## Grafik, Schrift, Klang

| Was | Quelle | Lizenz |
|---|---|---|
| Kartenbilder des deutschen Blatts (Eichel, Grün, Rot, Schellen, Kartenrücken, Blanko) | Selbst gezeichnet als SVG in `src/ui/cards.ts` und `src/style.css` | Teil dieses Projekts |
| Favicon | Selbst gezeichnet, `public/favicon.svg` | Teil dieses Projekts |
| Schrift | Systemschrift des Geräts (`system-ui`), nichts wird geladen | – |
| Klänge | keine (kommen in M3) | – |

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

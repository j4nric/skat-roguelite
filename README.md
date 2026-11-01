# Stammtisch

Skat als Roguelike-Deckbuilder (Repo-Arbeitsname: skat-roguelite). Läuft im Browser: https://stammtisch-62-83-23-90.sslip.io Du bist immer Alleinspieler, spielst gegen zwei KI-Gegner und veränderst zwischen den Runden das Blatt, mit dem alle drei spielen.

- Design: [`docs/DESIGN.md`](docs/DESIGN.md)
- Begriffe: [`docs/GLOSSARY.md`](docs/GLOSSARY.md)
- Entscheidungen: [`docs/adr/`](docs/adr/)
- Deployment: [`deploy/README.md`](deploy/README.md)
- Cockpit (Entwicklungsstand, offene Fragen): https://claude.ai/artifact/X1K7udk5bFayH8gGSdMUzB

## Aufbau

| Pfad | Inhalt |
|---|---|
| `core/` | Rust-Crate `skat_core`: Regeln, Spielzustand, KI, Run-Logik. Keine Engine-Abhängigkeit. |
| `sim/` | Headless-Simulator und Balancing-CLI (`cargo run -p skat_sim`). |
| `wasm/` | wasm-bindgen-Schicht, die den Kern im Browser verfügbar macht (ab M2). |
| `web/` | Web-App: Oberfläche des Spiels (ab M2, bis dahin Platzhalter). |
| `deploy/` | Docker Compose und Caddy für den vServer. |

## Befehle

```sh
cargo test                                  # alle Tests
cargo clippy --all-targets -- -D warnings   # Lint
cargo run -p skat_sim -- --seed 42 --games 10000
```

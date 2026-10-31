# Skat Roguelike

Skat als Roguelike-Deckbuilder. Du bist immer Alleinspieler, spielst gegen zwei KI-Gegner und veränderst zwischen den Runden das Blatt, mit dem alle drei spielen.

- Design: [`docs/DESIGN.md`](docs/DESIGN.md)
- Begriffe: [`docs/GLOSSARY.md`](docs/GLOSSARY.md)
- Entscheidungen: [`docs/adr/`](docs/adr/)

## Aufbau

| Pfad | Inhalt |
|---|---|
| `core/` | Rust-Crate `skat_core`: Regeln, Spielzustand, KI, Run-Logik. Keine Engine-Abhängigkeit. |
| `sim/` | Headless-Simulator und Balancing-CLI (`cargo run -p skat_sim`). |
| `bridge/` | GDExtension-Bridge zu Godot 4 (ab M3). |
| `godot/` | Godot-4-Projekt (ab M3). |

## Befehle

```sh
cargo test                                  # alle Tests
cargo clippy --all-targets -- -D warnings   # Lint
cargo run -p skat_sim -- --seed 42 --games 10000
```

# Skat Roguelike

Skat als Roguelike-Deckbuilder. Rust-Kern, Godot-4-Frontend. Doku auf Deutsch, Code und Kommentare auf Englisch.

## Pflichtlektüre

- `docs/DESIGN.md`: Spielkonzept, Designsäulen, Architektur, Roadmap. Bei Unsicherheit über eine Mechanik hier nachsehen, nicht raten.
- `docs/GLOSSARY.md`: Verbindliche Zuordnung deutscher Skat-Begriffe zu englischen Bezeichnern. Neue Begriffe zuerst hier eintragen.
- `docs/adr/`: Begründungen für Grundentscheidungen. Grundentscheidung ändern heißt neues ADR schreiben.

## Befehle

```sh
cargo test                                   # alle Tests
cargo clippy --all-targets -- -D warnings    # muss sauber sein
cargo fmt                                    # vor jedem Commit
cargo run -p skat_sim -- --seed 42 --games 10000
```

## Architekturregeln

- `core/` kennt Godot nicht. Keine Engine-Typen, keine Zeit, kein Rendering.
- Alles im Kern ist deterministisch. Zufall nur über den seeded RNG des Kerns, nie `rand::thread_rng` oder Systemzeit.
- Zustandsänderungen erzeugen Ereignisse. Frontend und Simulator sind Konsumenten.
- Content (Relikte, Kartenmodifikationen, Bosse) ist Daten in `core/content/*.ron`, nicht Code.
- Jede neue Mechanik wird im Simulator gemessen, bevor sie ins Godot-Projekt kommt.
- `unsafe` ist im Workspace verboten.

## Arbeitsweise

- Kleine Commits mit sprechender Nachricht. Commit-Nachrichten auf Englisch.
- Tests gehören zum Feature, nicht in einen späteren Commit.
- Golden-Vektoren für die Spielwertformel: `core/tests/game_value_vectors.rs`. Bei Regelfragen gilt die Skatordnung, bei Roguelike-Fragen das Design-Dokument.
- Spezialisierte Agenten in `.claude/agents/`. Regelfragen an `skat-rules`, Kernlogik an `core-dev`, Zahlen an `balance`, Relikte an `content`.

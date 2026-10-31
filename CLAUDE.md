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

## Git-Workflow

- `main` ist geschützt gedacht: nie direkt pushen. Jede Änderung auf einem Branch (`feat/…`, `fix/…`, `docs/…`), dann PR gegen `main`.
- Claude erstellt den PR und mergt ihn selbst (Squash), sobald CI grün ist. Der Nutzer liest mit, muss aber nicht freigeben.
- Push auf `main` löst den Deploy auf den vServer aus (`.github/workflows/deploy.yml`, siehe `deploy/README.md`, ADR 0007).
- Kleine Commits mit sprechender Nachricht. Commit-Nachrichten auf Englisch.
- Tests gehören zum Feature, nicht in einen späteren Commit.
- Golden-Vektoren für die Spielwertformel: `core/tests/game_value_vectors.rs`. Bei Regelfragen gilt die Skatordnung, bei Roguelike-Fragen das Design-Dokument.
- Spezialisierte Agenten in `.claude/agents/`. Regelfragen an `skat-rules`, Kernlogik an `core-dev`, Zahlen an `balance`, Relikte an `content`.

## Cockpit (Artefakt)

Der Nutzer verfolgt die Entwicklung mobil über das Cockpit-Artefakt:
https://claude.ai/artifact/X1K7udk5bFayH8gGSdMUzB

Es ist eine Seite mit Datenbank (`ArtifactData`). Pflicht nach jedem Arbeitsschritt:

- **Verlauf**: Eintrag in `log` (`ts`, `kind`: commit | pr | merge | decision | note, `title`, `body`, optional `url`). Ids sortierbar: `JJJJMMTT-HHMM-stichwort`.
- **Meilensteine**: `milestones/M0..M5` mit `status` (done | active | planned) und kurzer `note` aktuell halten.
- **Fragen**: Jede Entscheidung, die der Nutzer treffen soll, kommt als Dokument in `questions` (`order`, `area`, `title`, `context`, `recommendation`, `options`, `status: open`). Nicht im Chat fragen, wenn es im Cockpit beantwortet werden kann.
- **Antworten lesen**: Zu Beginn jeder Session `questions` mit `status == answered` und `claudeAck != true` abfragen, die Antwort umsetzen, dann `claudeAck: true` setzen.
- **meta/project**: `branch`, `lastCommit`, `openPr`, `server`, `updatedAt` nach jedem Merge aktualisieren.

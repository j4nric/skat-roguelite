# KI-Genauigkeit: Augen mit Vorzeichen, Knotenbudget je Zug, schnellere Schätzung

- **Datum**: 09.11.2026
- **Branch**: `feat/ai-accuracy` (von `main` 307b245)
- **Agent**: `core-dev`
- **Art**: Nur KI (`core/src/ai/solver.rs`, `core/src/ai/pimc.rs`, `SeatView` in `core/src/game.rs`). Keine Zielwerte, keine Relikte, kein Content.
- **Anlass**: Code-Review, Befunde C6, C5, C1, C20 und P9.

## Befehle

```sh
./target/release/sim --seed 42 --games 2000 --defenders pimc:8:3        # und pimc:12:3
./target/release/sim --seed 42 --runs 2000 --character null             # alle fünf, Heuristik-Spieler
./target/release/sim --seed 42 --runs 2000 --card-mod C7:battered --card-mod S7:battered --card-mod H7:battered --card-mod D7:battered
./target/release/sim --seed 42 --runs 1000 --bot pimc:12:3 --character declarer   # alle fünf, PIMC-Spieler
cargo test --release -p skat_core bench_augen -- --ignored --nocapture             # PIMC_SAMPLES=12
cargo test --release -p skat_core bench_node_limit -- --ignored --nocapture        # BENCH_DEPTH=0 oder 3
```

## 1. Was sich geändert hat

Je Schritt ein Commit, nach jedem gemessen.

| Schritt | Befund | Änderung |
|---|---|---|
| C6 | Löser und PIMC sahen weder Augen unter 0 („Ramponiert“) noch Extra-Augen aus Karteneffekten („Erster Stich“, `Game::bonus_points`). | Der Löser rechnet mit Augen mit Vorzeichen; seine Schranken sind die Summen der lebenden Augen unter und über 0. `SeatView::seat_points` trägt die Augen je Platz mit Vorzeichen und Extra-Augen; PIMC nimmt daraus Gewinn- und Schneider-Linie. |
| C5 | Das Knotenlimit (6000) galt für alle Züge einer Suche zusammen; späte Züge in Kartenreihenfolge bekamen nur einen Stich und die Schätzung. | Jeder Zug an der Wurzel bekommt das ganze Limit. Gemessen gegen gleichmäßiges Aufteilen (siehe 2). |
| C1 | An der Stichgrenze hinter dem Limit kam die Schätzung vor der Transpositionstabelle und wurde nie gespeichert. | Erst die Tabelle, bei einem Fehlgriff die Schätzung als exakter Wert hinein. |
| C20 | Gleichwertige Karten wurden nur mit der zuletzt behaltenen verglichen. | Vergleich mit allen behaltenen, die Trenner einmal je Knoten. |
| P9 | Die gierige Schätzung baute je Karte Tupel und den Stich neu. | Vorberechnete ganzzahlige Schlüssel, gleiche Reihenfolge; verhaltensgleich (Test gegen die Tupel-Fassung, Simulator gibt dieselben Spiele aus). |

## 2. Knotenbudget: je Zug oder aufgeteilt (C5)

`bench_augen`, Alleinspieler-Heuristik gegen PIMC 12:3, je 600 Spiele; Knoten je Entscheidung (deterministisch):

| Variante | Normales Blatt gewonnen | Ramponierte Siebener | Erster Stich doppelt | Knoten Mittel | p99 | Maximum |
|---|---|---|---|---|---|---|
| gemeinsam (vorher, Stand C6) | 35,2 % | 19,7 % | 40,7 % | 9 242 | 50 513 | 60 026 |
| je Zug (gewählt) | 35,0 % | 19,3 % | 40,7 % | 9 544 | 53 414 | 74 216 |
| aufgeteilt | 37,8 % | 19,8 % | 41,7 % | 8 667 | 47 211 | 58 656 |

`bench_node_limit` (Tiefe 3 wie im Spiel, 300 Stellungen mit 10 Stichen, Antworten mit Limit gegen ohne): gemeinsam 0,3 bis 1,0 % falsche Antworten bei den Zügen 4 bis 10, je Zug 0 %, aufgeteilt 0,7 bis 2,3 % bei allen Zügen.

Aufteilen schwächt die Verteidiger (+2,6 Punkte für den Alleinspieler) und rechnet ungenauer als vorher. Je Zug rechnet bei Tiefe 3 so genau wie ohne Limit und kostet 3 % mehr Knoten im Mittel; die größte Entscheidung wächst um ein Viertel (mit ramponierten Siebenern von 60 000 auf 92 000 Knoten). `DECISION_NODES` begrenzt die Entscheidung weiter zwischen zwei Stichproben.

## 3. Verteidiger und Rechenzeit

Alleinspieler-Heuristik gegen PIMC-Verteidiger, `--games 2000`, normales Blatt (dort ändern C6 nichts, es gibt keine Augen unter 0):

| Verteidiger | vorher gewonnen | nachher gewonnen |
|---|---|---|
| PIMC 8:3 | 38,9 % | 39,0 % |
| PIMC 12:3 | 38,1 % | 38,5 % |

Blätter mit Augen-Effekten (`bench_augen`, 600 Spiele, Spiele des Alleinspielers gewonnen):

| Blatt | Verteidiger | vorher | nach C6 | Endstand |
|---|---|---|---|---|
| Ramponierte Siebener (−5) | PIMC 8:3 | 23,8 % | 22,7 % | – |
| Erster Stich doppelt | PIMC 8:3 | 46,8 % | 43,3 % | – |
| Ramponierte Siebener (−5) | PIMC 12:3 | 22,5 % | 19,7 % | 19,3 % |
| Erster Stich doppelt | PIMC 12:3 | 42,2 % | 40,7 % | 40,7 % |

Vorher spielten die Verteidiger auf allen drei Blättern Karte für Karte gleich; sie sahen den Unterschied nicht.

Zeit je PIMC-Entscheidung (Spielemodus, 2000 Spiele, nativ, `main` und Endstand direkt nacheinander, je zwei Läufe gemittelt):

| Verteidiger | Mittel | p95 | p99 | Maximum |
|---|---|---|---|---|
| 8:3 vorher | 1,38 ms | 6,63 ms | 11,4 ms | 31 bis 44 ms |
| 8:3 nachher | 1,16 ms (−16 %) | 5,50 ms (−17 %) | 10,4 ms (−9 %) | 28 bis 33 ms |
| 12:3 vorher | 2,09 ms | 10,2 ms | 16,0 ms | 35 bis 45 ms |
| 12:3 nachher | 1,64 ms (−22 %) | 7,96 ms (−22 %) | 12,1 ms (−24 %) | 33 bis 34 ms |

Die Zeit gewinnt P9 (die gierige Schätzung war der heiße Pfad); C5 kostet ein paar Prozent, C1 und C20 sind auf dem normalen Blatt fast neutral. Die Maxima schwanken mit der Last der Maschine.

## 4. Runs

Heuristik-Spieler, `--runs 2000`, Seed 42 (Stadtteil 2 / Stadtteil 3 / gewonnen):

| Charakter | vorher | nachher |
|---|---|---|
| Alleinspieler | 34,9 / 10,7 / 2,3 % | 34,5 / 10,3 / 2,1 % |
| Nullspieler | 34,7 / 13,4 / 3,2 % | 34,7 / 13,3 / 2,7 % |
| Drückeberger | 34,4 / 10,2 / 1,6 % | 34,0 / 10,0 / 1,5 % |
| Schneiderin | 34,1 / 10,7 / 2,1 % | 34,0 / 10,1 / 1,9 % |
| Bruderherz | 32,5 / 9,4 / 1,4 % | 32,3 / 9,3 / 1,7 % |
| Alleinspieler mit 4 ramponierten Siebenern | 31,2 / 8,5 / 1,7 % | 31,3 / 8,6 / 1,5 % |

PIMC-Spieler (`--bot pimc:12:3`, Maßstab der Kurven), `--runs 1000`, Seed 42:

| Charakter | vorher | nachher |
|---|---|---|
| Alleinspieler | 42,4 / 15,2 / 3,8 % | 43,3 / 15,7 / 4,1 % |
| Nullspieler | 37,4 / 14,8 / 3,9 % | 37,5 / 15,4 / 4,2 % |
| Drückeberger | 38,9 / 14,2 / 3,2 % | 39,9 / 14,3 / 3,4 % |
| Schneiderin | 36,7 / 15,4 / 3,3 % | 37,5 / 14,4 / 3,2 % |
| Bruderherz | 38,7 / 16,4 / 4,1 % | 39,1 / 15,9 / 3,8 % |

Standardfehler bei 1000 Runs etwa 1,5 / 1,1 / 0,6 Punkte, bei 2000 Runs 1,1 / 0,7 / 0,3.

## 5. Lesart

- **Augen mit Vorzeichen (C6)** wirken nur, wo Augen unter 0 oder Extra-Augen im Spiel sind. Dort sind die Verteidiger deutlich stärker: 1 bis 3 Punkte weniger Siege für den Alleinspieler. Im Run sind solche Spiele selten (Ramponiert hängt nur an der Ausgabe des Spielers, 15 bis 16 % der Runs tragen es am Ende, im Mittel auf 0,18 Karten je Run), deshalb bewegen sich die Run-Zahlen kaum.
- **Knotenbudget je Zug (C5)** macht die Suche bei Tiefe 3 exakt und frei von der Kartenreihenfolge. Auf dem normalen Blatt ist der Effekt auf die Siegquote im Rauschen.
- **Keine Run-Größe verschiebt sich um mehr als 1 Punkt**, bei beiden Spielern und allen Charakteren, alles innerhalb von ein bis zwei Standardfehlern. Mit dem PIMC-Spieler liegen die Stadtteil-2-Quoten eher etwas höher (+0,1 bis +1,0); er rechnet mit denselben Verbesserungen, auch mit den Augen seiner eigenen ramponierten Karten.
- **Rechenzeit**: Die Entscheidungen der Verteidiger sind im Mittel 16 bis 22 % schneller, p95 und p99 ebenso; die Maxima liegen nicht höher als vorher. Auf dem Handy (wasm, Hauptthread) bleibt das Budget gewahrt.
- **Keine Balancing-Runde nötig.** Die Kurven von Runde 6 gelten weiter.

## 6. Offen

- `run/bot.rs` `schneider_percent` rechnet den Augen-Pool ohne Vorzeichen; seit C6 zählt der Löser Augen unter 0 mit. Mit ramponierten Karten in der Hand schätzt der Bot Schneider dadurch etwas zu vorsichtig. Eine Zeile (`signed_points` statt `card_points`), sobald die Aufteilung von `run/bot.rs` gemergt ist.
- Spätere Extra-Augen (ein Effekt, der erst nach dem nächsten Stich greift) kennt PIMC nicht; nur die bisher vergebenen zählen.

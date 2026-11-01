# Große Testrunde: Charaktere, Einsatz-Leiter, Passen, Schnäpse, Freischaltungen

**Stand vor der Umstellung auf 3 Spiele je Serie.** Die Run-Struktur wird umgebaut (Serie mit 3 Spielen, „Einmischen“ statt Passen, Ansagen auch nach dem Skat). Alle Zahlen hier gelten für Spiellimit 8 und 6 Pässe; die Zielwert-Kalibrierung unten ist damit überholt und wurde **nicht** übernommen (Content unverändert). Sie bleibt als Befund stehen: wie weit die Charaktere auseinanderliegen und wie flach die Erfolgskurve seit Null für alle ist.

- **Datum**: 01.11.2026
- **Branch**: `feat/testround` (Stand `main` b9da72b, nach PR #36)
- **Agent**: `balance`
- **Maßstab**: PIMC-Spieler (`--bot pimc:12:3`), Gegner je Abschnitt aus `run.ron`, alle Relikte freigeschaltet, Flüche nur angekündigt.
- **Seeds**: 42, 43, 44. Ist-Stand 3 × 1000 Runs je Charakter und Länge (Standardfehler für „Abschnitt 2 erreicht“ rund 0,9 Punkte). Kalibrierte Kurven 3 × 3000 Runs (rund 0,5 Punkte).
- **Simulator neu** (`sim/`, getestet): `--consumable <id>` (jeder Run startet mit diesem Verbrauchsgegenstand, jeder Tresen bietet nur ihn an, gratis, ohne Relikte; Vergleich gegen `--no-shop`); Zeile `consumables (bought/used)` je Verbrauchsgegenstand; `--curve` spielt jetzt auch bei Charakteren mit eigenen Zielwerten jede Serie bis zum Spiellimit (vorher brach der Nullspieler bei seinem ersten Zielwert ab und seine Kurve war falsch); die Tabelle `series` druckt die Zielwerte des Charakters statt immer die aus `run.ron`.

## 1. Ist-Stand je Charakter (Content von `main`, ohne Änderung)

```sh
cargo run --release -p skat_sim -- --seed 42 --runs 1000 --bot pimc:12:3 --character declarer|null|grand|onlooker|landlady [--short]
# dasselbe mit --seed 43 und --seed 44
```

Ganzer Abend (3 × 1000 Runs):

| Charakter | Abschnitt 2 | 3 | 4 | 5 | Run gewonnen | gehalten je Abschnitt |
|---|---|---|---|---|---|---|
| Alleinspieler | 51,3 % | 28,3 % | 16,5 % | 10,1 % | 5,83 % | 51 / 55 / 58 / 61 / 58 % |
| Nullspieler | 33,3 % | 14,2 % | 5,4 % | 2,4 % | 1,17 % | 33 / 43 / 38 / 44 / 49 % |
| Grandspieler | 57,9 % | 35,0 % | 21,1 % | 13,7 % | 8,57 % | 58 / 61 / 60 / 65 / 62 % |
| Zaungast | 58,4 % | 36,8 % | 23,1 % | 15,4 % | 9,93 % | 58 / 63 / 63 / 67 / 64 % |
| Wirtin | 55,5 % | 33,7 % | 19,3 % | 11,1 % | 5,90 % | 56 / 61 / 57 / 58 / 53 % |

Kurzer Abend (3 × 1000 Runs; Abschnitt 1 und 2 sind dieselben Runs wie im ganzen Abend, im dritten sitzt der Alte):

| Charakter | Abschnitt 2 | 3 | Run gewonnen | gehalten je Abschnitt |
|---|---|---|---|---|
| Alleinspieler | 51,3 % | 28,3 % | 17,13 % | 51 / 55 / 61 % |
| Nullspieler | 33,3 % | 14,2 % | 5,63 % | 33 / 43 / 40 % |
| Grandspieler | 57,9 % | 35,0 % | 21,63 % | 58 / 61 / 62 % |
| Zaungast | 58,4 % | 36,8 % | 23,13 % | 58 / 63 / 63 % |
| Wirtin | 55,5 % | 33,7 % | 19,90 % | 56 / 61 / 59 % |

Einzelne Seeds, ganzer Abend (Abschnitt 2 / Run gewonnen): Alleinspieler 51,1 / 50,8 / 51,9 % und 6,5 / 5,5 / 5,5 %; Nullspieler 34,1 / 33,2 / 32,7 % und 1,0 / 1,5 / 1,0 %; Grandspieler 57,2 / 57,6 / 58,8 % und 8,7 / 8,7 / 8,3 %; Zaungast 58,2 / 58,5 / 58,6 % und 10,0 / 9,2 / 10,6 %; Wirtin 55,1 / 54,5 / 57,0 % und 4,8 / 6,3 / 6,6 %.

Spiele, Null und Pässe (ganzer Abend, je Run und je Serie):

| Charakter | Spiele je Run | Pässe je Run | Spiele je Serie | Pässe je Serie | Pässe je Spiel | Null-Anteil | Null gewonnen | Spiele gewonnen |
|---|---|---|---|---|---|---|---|---|
| Alleinspieler | 11,5 | 8,9 | 2,14 | 1,67 | 0,78 | 32,2 % | 55,2 % | 68,5 % |
| Nullspieler | 8,4 | 8,2 | 2,35 | 2,29 | 0,97 | 29,3 % | 74,2 % | 60,3 % |
| Grandspieler | 12,6 | 9,9 | 2,09 | 1,65 | 0,79 | 32,7 % | 55,7 % | 73,5 % |
| Zaungast | 13,2 | 9,5 | 2,11 | 1,52 | 0,72 | 32,8 % | 56,3 % | 71,3 % |
| Wirtin | 12,0 | 9,6 | 2,10 | 1,68 | 0,80 | 33,1 % | 55,1 % | 70,5 % |

Kurzer Abend: dieselben Quoten, 8,0 bis 11,0 Spiele und 7,8 bis 8,4 Pässe je Run.

**Befund**: Gegen das M5-Zielbild (Abschnitt 2 gut ein Drittel, Run gewonnen 1 bis 2 %) ist nur der Nullspieler im Rahmen. Der Alleinspieler ist seit Null für alle und der einen Null-Wertung mit Stilbonus deutlich zu stark (M5: 36,3 % und 1,3 %), die drei neuen Charaktere noch stärker; der Zaungast gewinnt trotz zwei Pässen weniger am häufigsten. Im kurzen Abend gewinnt der Alleinspieler jeden sechsten Run.

**Erfolgskurve einer Serie, Abschnitt 1** (`--curve --runs 2000 --seed 42`, P(Serie gewonnen) bei Zielwert 1 / 36 / 72 / 120):

| Charakter | 1 | 36 | 72 | 120 | Punkte je Spiel |
|---|---|---|---|---|---|
| Alleinspieler | 90,9 % | 84,8 % | 75,3 % | 58,8 % | 23,8 |
| Nullspieler | 81,8 % | 73,0 % | 61,8 % | 49,5 % | 18,8 |
| Grandspieler | 93,5 % | 87,8 % | 80,3 % | 66,3 % | 32,1 |
| Zaungast | 93,2 % | 87,7 % | 78,5 % | 63,6 % | 25,6 |
| Wirtin | 91,0 % | 85,0 % | 75,3 % | 58,8 % | 23,9 |

In M5 lag der Alleinspieler bei 85 / 76 / 57 / 36 % (bei 1 / 36 / 72 / 110). Die Kurve ist flach geworden: Ein Zielwert beißt erst ab etwa zwei gewonnenen Spielen, große Zielwert-Schritte bewegen die Quote wenig. Die Erkundung brauchte deshalb rund doppelt so hohe Zielwerte.

## 2. Kalibrierung (überholt, nicht übernommen)

Mit dem Umbau auf 3 Spiele je Serie verliert diese Kurve ihre Grundlage. Sie zeigt, welche Abstände zwischen den Charakteren die neue Kalibrierung ausgleichen muss.

Kurve des Alleinspielers (`run.ron`), Erkundung 2 × 1000 Runs (Seeds 42, 43):

| Kandidat | Abschnitt 1 bis 5 | Abschnitt 2 | 3 | 4 | 5 | gewonnen | gehalten je Abschnitt |
|---|---|---|---|---|---|---|---|
| A | 60/68/84, 72/80/90, 84/96/100, 96/108/120, 108/120/132 | 39,9 % | 16,8 % | 8,4 % | 3,5 % | 1,50 % | 40 / 42 / 50 / 41 / 43 % |
| B | 66/76/92, 84/92/104, 100/112/120, 116/128/144, 132/144/160 | 36,1 % | 12,9 % | 5,5 % | 1,8 % | 0,45 % | 36 / 36 / 42 / 33 / 25 % |
| C | 64/72/88, 72/80/90, 90/100/110, 96/104/112, 104/116/124 | 37,8 % | 15,9 % | 7,8 % | 3,1 % | 1,30 % | 38 / 42 / 49 / 40 / 42 % |
| D | 62/70/86, 74/82/92, 88/98/108, 96/106/116, 104/116/128 | 38,6 % | 15,2 % | 7,5 % | 3,0 % | 1,25 % | 39 / 39 / 50 / 40 / 42 % |
| E (gewählt, Reizwerte) | 63/72/88, 72/80/90, 90/100/110, 96/100/110, 100/110/120 | 38,0 % | 16,0 % | 7,8 % | 3,2 % | 1,35 % | 38 / 42 / 48 / 41 / 42 % |

Andere Charaktere: eigene `targets` als Kurve E mal Faktor, auf Reizwerte gerundet. Erkundung (2 × 1000): Grandspieler × 1,2 38,1 % / 1,10 %, × 1,35 33,7 % / 0,70 %; Zaungast × 1,1 40,1 % / 1,35 %, × 1,2 35,7 % / 0,70 % (1 Seed); Wirtin × 1 41,9 % / 1,60 %, × 1,1 35,4 % / 1,00 % (1 Seed). Nullspieler: Abschnitt 1 von 36/44/54 auf 27/36/46, Rest wie gehabt.

Ergebnis mit diesen Kurven (3 × 3000 Runs, Seeds 42 bis 44, ganzer Abend):

| Charakter | Kurve | Abschnitt 2 | 3 | 4 | 5 | Run gewonnen | gehalten je Abschnitt |
|---|---|---|---|---|---|---|---|
| Alleinspieler | E | 38,8 % | 16,5 % | 7,6 % | 3,5 % | 1,60 % | 39 / 43 / 46 / 46 / 46 % |
| Nullspieler | eigene, Abschnitt 1 27/36/46 | 37,8 % | 15,7 % | 6,6 % | 3,1 % | 1,39 % | 38 / 41 / 42 / 47 / 45 % |
| Grandspieler | E × 1,2 | 38,4 % | 16,4 % | 6,6 % | 3,0 % | 1,20 % | 38 / 43 / 40 / 46 / 39 % |
| Zaungast | E × 1,15 | 38,4 % | 16,0 % | 6,8 % | 3,0 % | 1,11 % | 38 / 42 / 42 / 45 / 37 % |
| Wirtin | E × 1,05 | 40,0 % | 17,5 % | 6,5 % | 2,6 % | 0,90 % | 40 / 44 / 38 / 40 / 35 % |
| Wirtin | Abschnitt 1–2 etwa × 1,07, danach E | 39,1 % | 17,2 % | 7,0 % | 2,9 % | 1,14 % | 39 / 44 / 41 / 41 / 39 % |

Mit diesen Kurven spielte ein Run im Mittel 2,6 Spiele und 2,0 Pässe je Serie (vorher 2,1 und 1,7), ein voller Run also rund 39 Spiele und 30 Pässe, grob 60 Minuten: höhere Zielwerte verlängern den Run.

Lehren für die Kalibrierung nach dem Umbau:

1. **Relative Stärke**: Grandspieler braucht rund 20 % höhere Zielwerte als der Alleinspieler, Zaungast rund 15 %, Wirtin vorne rund 7 % und hinten keine (sie ist früh stark, mit vier Plätzen spät schwach). Der Nullspieler liegt etwa 4 Punkte unter dem Alleinspieler bei Abschnitt 2.
2. **Schwere Bosse**: Schwarzer Peter (41 bis 47 %), Schieber (43 bis 61 %) und Oma (49 bis 55 %) gewinnt der Alleinspieler mit Kurve E deutlich seltener als die übrigen (64 bis 85 %). Der Schieber ist schwerer als in M5 (58 / 70 %). Vorschlag zum Nachmessen nach dem Umbau: Schieber-Faktor 1,0 → 0,8.
3. Abschnitt 4 (PIMC 12:3) ist bei gleicher Kurve spürbar schwerer als Abschnitt 3 (PIMC 8:3).

## 3. Einsatz-Leiter

```sh
cargo run --release -p skat_sim -- --seed 42 --runs 1000 --bot pimc:12:3 --stake N [--character null]
# N = 0, 2, 5, 10; Seeds 42 bis 44
```

Gemessen mit Kurve E (Abschnitt 2), nicht mit dem Content von `main`; die Wirkung je Auge ist relativ zu lesen. 3 × 1000 Runs, gleiche Karten je Seed:

| Einsatz | Gewinn ab | Abschnitt 2 | 3 | Run gewonnen | Farbe/Grand gewonnen | Null gewonnen | Null-Anteil | Punkte je Spiel |
|---|---|---|---|---|---|---|---|---|
| Alleinspieler 0 | 61 | 37,8 % | 16,1 % | 1,63 % | 75,3 % | 55,7 % | 33,2 % | 28,9 |
| Alleinspieler 2 | 63 | 35,1 % | 13,6 % | 1,23 % | 72,1 % | 55,3 % | 33,3 % | 24,8 |
| Alleinspieler 5 | 66 | 30,2 % | 10,3 % | 0,53 % | 67,8 % | 55,7 % | 33,4 % | 20,3 |
| Alleinspieler 10 | 71 | 23,1 % | 6,6 % | 0,30 % | 59,9 % | 55,8 % | 33,8 % | 12,2 |
| Nullspieler 0 | 61 | 37,6 % | 15,9 % | 1,23 % | 55,0 % | 74,3 % | 29,3 % | 29,5 |
| Nullspieler 10 | 71 | 28,4 % | 9,2 % | 0,37 % | 39,1 % | 74,5 % | 29,2 % | 16,1 |

- **Ein Auge kostet** in den ersten Stufen rund 1,4 Punkte „Abschnitt 2 erreicht“ und 1,5 Punkte Gewinnquote bei Farbspiel und Grand; „Run gewonnen“ fällt je Auge um rund 20 % (relativ), bei Einsatz 5 auf ein Drittel, bei 10 auf ein Fünftel. Die Leiter ist damit steil: Schon Einsatz 3 bis 4 halbiert die Siegchance.
- **Null wird relativ stärker**: Der Bot ist für den Einsatz blind (Ansage und Passen kennen die Schwelle nicht, `suit_expected_score` rechnet mit 61), deshalb bleibt der Null-Anteil gleich. Die Gewinnquote von Null bleibt aber konstant, die von Farbe und Grand fällt: Bei Einsatz 10 gewinnt der Alleinspieler Farbspiele kaum noch öfter als Null (59,9 gegen 55,8 %), der Nullspieler verliert weniger als der Alleinspieler (Abschnitt 2 −24 % gegen −39 % relativ) und liegt bei Einsatz 10 vorn. Ein Mensch, der den Einsatz kennt, wird ab etwa Einsatz 8 bis 10 deutlich mehr Null spielen. **Flag**: Bei hoher Einsatz-Stufe dominiert Null; nach dem Umbau neu messen, sobald der Bot den Einsatz in seiner Wahl berücksichtigt (an `core-dev`: `suit_expected_score` und `SUIT_WIN_PERCENT`/`GRAND_WIN_PERCENT` mit der Schwelle `61 + n` rechnen).

## 4. Passen

Siehe Tabelle in Abschnitt 1. Der Bot passt **weniger als er spielt**: 0,72 (Zaungast) bis 0,80 Pässe je gespieltem Spiel, nur der Nullspieler liegt mit 0,97 knapp an der Grenze von einem Pass je Spiel. In M5 (vor Null für alle) waren es 1,5 (2,84 Pässe und 1,87 Spiele je Serie): Null für alle hat dem Bot viele Blätter spielbar gemacht. Je Serie sind es 1,5 bis 2,3 Pässe. Kein Vorschlag nötig; die Umstellung auf „Einmischen“ ersetzt das Passen ohnehin.

## 5. Schnäpse

Die Einzelmessung (`--no-shop` gegen `--consumable <id>`, 3 × 1000 Runs) wurde wegen des Umbaus abgebrochen, bevor die Schnäpse liefen; das Werkzeug ist im Simulator. Vorhanden sind Kauf und Nutzung in den Runs mit Tresen (Kurven aus Abschnitt 2, 3 × 3000 Runs je Charakter, gekauft / benutzt):

| Gegenstand | Preis | Alleinspieler | Nullspieler | Grandspieler | Zaungast | Wirtin |
|---|---|---|---|---|---|---|
| Kurzer (`shot`) | 2 | 1237 / 1197 | 1311 / 1263 | 1164 / 1122 | 1205 / 1155 | 652 / 629 |
| Doppelkorn (`double_grain`) | 3 | 710 / 668 | 768 / 566 | 626 / 589 | 643 / 613 | 609 / 575 |
| Kräuter (`herbal`) | 2 | 2411 / 54 | 2546 / 343 | 2293 / 53 | 2432 / 936 | 1944 / 264 |
| Magenbitter (`bitters`) | 2 | 1593 / 473 | 1686 / 1586 | 1468 / 233 | 1576 / 344 | 1037 / 176 |
| Kreide (`chalk`) | 3 | 645 / 593 | 738 / 714 | 534 / 479 | 628 / 570 | 534 / 472 |
| Ein Bier (`beer`, Vergleich) | 2 | 2293 / 46 | 2436 / 333 | 2197 / 68 | 2369 / 940 | 1916 / 1 |

Befunde (Bot-Verhalten, keine Wirkungsmessung):

- **Kräuter und Ein Bier sind für den Bot nutzlos, werden aber am meisten gekauft**: Er kauft je Run rund 0,27 von jedem und braucht davon 2 % (Alleinspieler), weil er mit 6 Pässen fast nie ausgeht; nur der Zaungast (4 Pässe) nutzt sie zu 38 %. Das sind rund 1 Groschen je Run verschwendet. Ursache ist der Kaufwert 3 in `CONSUMABLE_VALUES` (`core/src/run/bot.rs`), nicht der Preis. Vorschlag an `core-dev`: Kaufwert von `beer`/`herbal` an die verbleibenden Pässe koppeln oder auf 1 senken.
- **Kurzer**: Wird gekauft und benutzt, der Bot nutzt die Information aber nicht. Platzhalter für Menschen, für den Bot ohne Wirkung.
- **Magenbitter** wird oft gekauft, aber vom Alleinspieler nur zu 30 % benutzt (er spielt selten schwache Blätter); der Nullspieler nutzt ihn fast immer (seine Farbspiele ohne Unter gelten als schwach).
- **Doppelkorn und Kreide** benutzt der Alleinspieler zu über 90 %, der Nullspieler Doppelkorn nur zu 74 % (nie vor einem Null).
- Preise und Seltenheit bleiben Platzhalter. Messung nach dem Umbau mit `skat_sim --runs 1000 --bot pimc:12:3 --consumable <id>` gegen `--no-shop` (gleiche Seeds). Teilmessung ohne Tresen mit Kurve E (2 × 1000 Runs, Seeds 42, 43): Abschnitt 2 erreicht 32,6 %, 3 7,8 %.

## 6. Freischaltungen

```sh
cargo run --release -p skat_sim -- --seed 42 --runs 200 --unlocks --bot pimc:12:3
```

20 Folgen mit frischem Meta-Zustand, höchstens 200 Runs, der Bot spielt Alleinspieler (und nach `tour_won` jeden zweiten Run Nullspieler). Gemessen mit Kurve E, also mit einem Alleinspieler auf dem Zielbild (Abschnitt 2 39 %, gewonnen 1,6 %); mit dem Content von `main` (51 %, 5,8 %) kämen alle früher.

| Freischaltung | Belohnung | erreicht | Runs bis dahin (Mittel) | min | max |
|---|---|---|---|---|---|
| `grand_master` (Grand Schneider ab 96) | Grandspieler | 20/20 | 3,1 | 1 | 11 |
| `full_till` (20 Groschen nach einer Serie) | Wirtin | 20/20 | 5,1 | 1 | 19 |
| `beat_kibitz_killer` | Zaungast | 20/20 | 9,2 | 2 | 22 |
| `tour_won` (ganzer Abend, Alleinspieler) | Nullspieler | 19/20 | 73,3 | 11 | 185 |

Übrige: `early_finish` 1,2; `second_evening` 1,9; `grand_schneider` 2,1; `big_game` 2,5; `hand_won` 4,8; `apprenticeship` 5,0; `boss_without_pass` 5,5; `big_win` 5,7; `third_evening` 6,2; `hearts_schneider` 6,8; `beat_kontra_hans` 10,6; `fourth_evening` 13,3; `grand_hand` 18,0; `beat_black_peter` 19,1; `beat_grandma` 20,4.

Befund: Der Grandspieler kommt nach drei Runs fast so früh wie ein einfacher Grand Schneider (2,1); 96 Punkte sind mit Null für alle und hohen Grand-Werten keine echte Hürde. Wenn er ein späteres Ziel sein soll: `min_value` 144 (wie `big_win`) oder Grand Hand Schneider, dann nachmessen. Wirtin und Zaungast liegen im Bereich der übrigen Relikt-Freischaltungen. `tour_won` nur im ganzen Abend: im Mittel nach 73 Runs, eine von 20 Folgen nicht in 200 Runs.

## 7. Offene Punkte

- Kalibrierung nach dem Umbau neu: Zielwerte je Charakter (Abstände aus Abschnitt 2 als Startwert), Boss-Faktoren (Schieber, Schwarzer Peter), Einsatz-Leiter.
- Bot einsatzblind (Abschnitt 3) und Kaufwerte von Bier/Kräuter (Abschnitt 5): an `core-dev`.
- Schnaps-Einzelwirkung messen (`--consumable`).
- `grand_master` zu leicht (Abschnitt 6).
- In `docs/DESIGN.md` steht in Abschnitt 11 eine verwaiste Marke `@@BOSS12@@` (Zeile 696).

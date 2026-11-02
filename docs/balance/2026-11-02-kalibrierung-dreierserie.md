# Kalibrierung für Serien mit 3 Spielen: Zielwerte je Charakter, Bosse, Einsatz, Freischaltungen

- **Datum**: 02.11.2026
- **Branch**: `feat/calibrate`. Ist-Stand und Erkundung auf `main` 1dace9c (nach PR #43 bis #45: 3 Spiele je Serie, zweimal einmischen je Spiel, Ansagen nach dem Skat, ADR 0013), Endmessung nach dem Merge von `main` 4255f6e (PR #46, Relikte Welle 2a, 43 Relikte).
- **Agent**: `balance`
- **Maßstab**: PIMC-Spieler (`--bot pimc:12:3`), Gegner je Abschnitt aus `run.ron`, alle Relikte freigeschaltet. Zielbild (M5, DESIGN 5.2): Abschnitt 2 in 33 bis 40 % der Runs, ganzer Abend gewonnen 1 bis 2 %, für **jeden** Charakter; Boss-Serien spürbar schwerer als normale Serien, aber kein Boss eine Wand.
- **Seeds**: 42, 43, 44, je 800 Runs (2400 je Charakter und Länge, Standardfehler für „Abschnitt 2 erreicht“ rund 1,0 Punkte, für „Run gewonnen“ bei 1,5 % rund 0,25 Punkte). Erkundung mit 2 × 500 bis 2 × 750 Runs.
- **Werkzeug**: kein neuer Simulator-Code. Für Kurven mit anderen Zielwerten wurde `run.ron` nur im Arbeitsbaum geändert (die Zielwerte, die `--curve` meldet, kommen aus `run.ron`).

## 1. Ist-Stand (vorläufige Kurve, `main` 1dace9c)

Alleinspieler, Grandspieler, Zaungast und Wirtin 24/30/36, 30/36/42, 36/42/48, 42/48/54, 48/54/60; der Nullspieler dieselbe Kurve eine Zeile später.

```sh
cargo run --release -p skat_sim -- --seed 42 --runs 800 --bot pimc:12:3 --character declarer|null|grand|onlooker|landlady [--short]
# dasselbe mit --seed 43 und --seed 44
```

Ganzer Abend (3 × 800 Runs):

| Charakter | Abschnitt 2 | 3 | 4 | 5 | Run gewonnen | gehalten je Abschnitt |
|---|---|---|---|---|---|---|
| Alleinspieler | 43,0 % | 17,4 % | 7,4 % | 3,5 % | 1,92 % | 43 / 41 / 42 / 47 / 55 % |
| Nullspieler | 31,7 % | 10,7 % | 3,3 % | 0,9 % | 0,42 % | 32 / 34 / 31 / 26 / 48 % |
| Grandspieler | 49,5 % | 22,7 % | 10,2 % | 5,5 % | 2,92 % | 50 / 46 / 45 / 54 / 53 % |
| Zaungast | 43,6 % | 17,2 % | 6,0 % | 2,7 % | 1,29 % | 44 / 39 / 35 / 45 / 48 % |
| Wirtin | 49,0 % | 24,3 % | 11,8 % | 5,4 % | 2,96 % | 49 / 50 / 49 / 46 / 55 % |

Kurzer Abend (Abschnitt 1 und 2 sind dieselben Runs): Run gewonnen Alleinspieler 7,71 %, Nullspieler 3,29 %, Grandspieler 10,29 %, Zaungast 6,42 %, Wirtin 12,83 %.

Boss-Serie gewonnen (ganzer Abend, alle Abschnitte, 230 bis 460 Serien je Boss):

| Boss | Faktor | Alleinspieler | Nullspieler | Grandspieler | Zaungast | Wirtin |
|---|---|---|---|---|---|---|
| Schwarzer Peter | 0,5 (Null 0,2) | 40 % | 37 % | 47 % | 38 % | 45 % |
| Oma | 0,3 | 52 % | – | – | – | – |
| Schieber | 1,0 (Null 0,4) | 63 % | 56 % | 60 % | 54 % | 65 % |
| Ramscher | 1,25 | 76 % | 74 % | 70 % | 74 % | 71 % |
| Kontra-Hans | 1,5 | 74 % | 61 % | 80 % | 73 % | 80 % |
| Zwillinge | 1,0 | 74 % | 71 % | 78 % | 75 % | 76 % |
| Revolutionär | 1,0 | 80 % | 68 % | 83 % | 79 % | 82 % |
| Kiebitz-Mörder | 1,0 | 79 % | 72 % | 81 % | 75 % | 85 % |

Normale Serien gewann der Alleinspieler zu 77 bis 88 %, die Boss-Serie in Abschnitt 1 bis 4 zu 66 bis 72 %.

**Befund**: Alleinspieler und Zaungast liegen etwas zu leicht (43 %), Grandspieler und Wirtin deutlich (49 %, fast 3 % Siege), der Nullspieler zu schwer (32 %, 0,4 %). Die Abstände aus der Testrunde vom 01.11. gelten nicht mehr: Der Zaungast ist mit nur einem Einmischen (statt zwei Pässen weniger) nicht mehr stärker als der Alleinspieler, die Wirtin ist vorne deutlich stärker (+6 Punkte statt +4). Revolutionär und Kiebitz-Mörder sind so leicht wie eine normale Serie.

## 2. Erfolgskurve einer Serie

```sh
cargo run --release -p skat_sim -- --seed 42 --runs 1000 --bot pimc:12:3 --curve --character <c> [--defenders pimc:12:3]
```

P(Serie gewonnen) ohne Relikte, gegen Heuristik-Gegner (Abschnitt 1), 1000 Serien, Seed 42:

| Charakter | 1 | 24 | 36 | 48 | 60 | 72 | 96 | 120 |
|---|---|---|---|---|---|---|---|---|
| Alleinspieler | 86,0 % | 81,5 % | 77,2 % | 73,3 % | 66,9 % | 61,8 % | 50,7 % | 42,4 % |
| Nullspieler | 76,6 % | 71,6 % | 66,2 % | 61,3 % | 57,4 % | 52,9 % | 45,2 % | 38,0 % |
| Grandspieler | 89,6 % | 83,8 % | 80,5 % | 76,7 % | 73,6 % | 70,2 % | 61,0 % | 50,8 % |
| Zaungast | 85,3 % | 81,0 % | 76,3 % | 72,2 % | 66,3 % | 60,6 % | 48,3 % | 40,1 % |
| Wirtin | 87,0 % | 83,1 % | 78,0 % | 74,1 % | 68,3 % | 62,5 % | 51,1 % | 43,4 % |

Gegen PIMC 12:3 liegt jede Zeile 4 bis 7 Punkte tiefer (Alleinspieler 80,7 / 76,0 / 70,6 / 66,1 / 60,7 %). Die Kurve ist flach: rund 0,4 Punkte je Zielwert-Punkt, beim Grandspieler 0,3. Der Nullspieler hat die niedrigste Decke: Auch bei Zielwert 1 gewinnt er eine Serie nur zu 76 % (fein gemessen, Seed 43, 1500 Serien: 76,3 % bei 1, 70,7 % bei 23, 61,2 % bei 48).

Bosse (Alleinspieler, Heuristik-Gegner, 300 bis 600 Serien): Schwarzer Peter 50,0 % bei 1, 49,0 % bei 24, 41,7 % bei 36; Oma 49,7 / 45,0 / 40,0 %; Schieber 61,7 / 59,2 / 53,7 %. Nullspieler: Schwarzer Peter 36,8 / 34,0 %, Schieber 45,5 / 43,5 %. Beim Schwarzen Peter und der Oma liegt die Boss-Serie im Run schon an der Obergrenze; ein kleinerer Faktor brächte höchstens 1 bis 2 Punkte.

## 3. Kalibrierung

Vorgehen: Der Abschnitt 1 bestimmt „Abschnitt 2 erreicht“; mit der Steigung der Kurve ergibt sich, wie weit die Zielwerte dort steigen müssen (Alleinspieler rund +9, Grandspieler rund +24, Wirtin rund +17). Danach hält jeder Abschnitt dank der Relikte mehr, die Kurve darf nur flach weiter steigen. Gerundet auf Reizwerte, keine Zeile unter der vorigen.

Erkundung (`main` 1dace9c, ganzer Abend):

| Runde | Charakter | Kurve (Abschnitt 1 bis 5) | Runs | Abschnitt 2 | Run gewonnen | gehalten je Abschnitt |
|---|---|---|---|---|---|---|
| 1 | Alleinspieler | 33/40/45, 35/40/46, 36/44/48, 40/46/54, 45/50/60 | 2 × 500 | 38,2 % | 1,60 % | 38 / 39 / 43 / 46 / 53 % |
| 1 | Nullspieler | 18/20/23, 20/23/27, 23/27/30, 27/30/35, 30/35/40 | 2 × 500 | 39,0 % | 1,50 % | 39 / 38 / 43 / 42 / 56 % |
| 1 | Grandspieler | 48/54/60, 48/54/60, 48/55/63, 50/60/66, 54/63/72 | 2 × 500 | 37,6 % | 1,30 % | 38 / 33 / 44 / 47 / 50 % |
| 1 | Zaungast | wie Alleinspieler | 2 × 500 | 37,6 % | 1,10 % | 38 / 39 / 37 / 41 / 50 % |
| 1 | Wirtin | 40/46/54, 40/46/54, 44/48/55, 46/54/60, 50/60/66 | 2 × 500 | 38,9 % | 0,60 % | 39 / 46 / 34 / 36 / 27 % |
| 2 | Zaungast | 33/40/45, 35/40/46, 35/40/46, 36/44/48, 40/46/54 | 2 × 750 | 38,2 % | 1,13 % | 38 / 36 / 38 / 46 / 47 % |
| 2 | Wirtin | 40/46/54 dreimal, 44/48/55, 46/54/60 | 2 × 750 | 39,7 % | 1,20 % | 40 / 42 / 37 / 44 / 44 % |
| 3 | Alleinspieler | Runde 1 | 3 × 800 | 37,1 % | 1,46 % | 37 / 38 / 43 / 46 / 52 % |
| 3 | Nullspieler | Runde 1 | 3 × 800 | 36,8 % | 0,88 % | 37 / 39 / 36 / 35 / 48 % |
| 3 | Grandspieler | Runde 1 | 3 × 800 | 37,1 % | 1,00 % | 37 / 34 / 40 / 47 / 42 % |
| 3 | Zaungast | Runde 2 | 3 × 800 | 37,7 % | 1,04 % | 38 / 37 / 37 / 44 / 45 % |
| 3 | Wirtin | Runde 2 | 3 × 800 | 38,9 % | 1,50 % | 39 / 41 / 43 / 49 / 44 % |
| 4 | Nullspieler | 18/20/23, 20/23/27 zweimal, 23/27/30 zweimal | 3 × 800 | 36,8 % | 0,96 % | 37 / 39 / 40 / 36 / 46 % |
| 4 | Grandspieler | 48/54/60 dreimal, 48/55/63, 50/60/66 | 3 × 800 | 37,1 % | 1,12 % | 37 / 34 / 41 / 49 / 44 % |
| 4 | Zaungast | 33/40/45 zweimal, 35/40/46 zweimal, 36/44/48 | 2 × 800 | 38,1 % | 1,19 % | 38 / 37 / 36 / 46 / 51 % |

In allen Runden mit Revolutionär und Kiebitz-Mörder 1,5 (Nullspieler 1,0), siehe Abschnitt 5. Runde 4 wurde für den Zaungast nach dem Merge abgebrochen (zwei von drei Seeds).

### Ergebnis (nach dem Merge von `main` 4255f6e, 43 Relikte)

Gewählte Kurven (Serie 1 / Serie 2 / Boss-Serie vor dem Boss-Faktor):

| Abschnitt | Gegner | Alleinspieler (`run.ron`) | Nullspieler | Grandspieler | Zaungast | Wirtin |
|---|---|---|---|---|---|---|
| 1 | Heuristik | 33 / 40 / 45 | 18 / 20 / 23 | 48 / 54 / 60 | 33 / 40 / 45 | 40 / 46 / 54 |
| 2 | PIMC 6:2 | 35 / 40 / 46 | 20 / 23 / 27 | 48 / 54 / 60 | 33 / 40 / 45 | 40 / 46 / 54 |
| 3 | PIMC 8:3 | 36 / 44 / 48 | 20 / 23 / 27 | 48 / 54 / 60 | 35 / 40 / 46 | 40 / 46 / 54 |
| 4 | PIMC 12:3 | 40 / 46 / 54 | 23 / 27 / 30 | 48 / 55 / 63 | 35 / 40 / 46 | 44 / 48 / 55 |
| 5 | PIMC 16:3 | 45 / 50 / 60 | 23 / 27 / 30 | 50 / 60 / 66 | 36 / 44 / 48 | 46 / 54 / 60 |

Ganzer Abend (3 × 800 Runs, Seeds 42 bis 44):

| Charakter | Abschnitt 2 | 3 | 4 | 5 | Run gewonnen | gehalten je Abschnitt |
|---|---|---|---|---|---|---|
| Alleinspieler | 37,3 % | 13,8 % | 5,7 % | 2,7 % | 1,42 % | 37 / 37 / 41 / 47 / 53 % |
| Nullspieler | 36,9 % | 14,5 % | 5,6 % | 1,9 % | 0,58 % | 37 / 39 / 38 / 34 / 30 % |
| Grandspieler | 36,3 % | 13,2 % | 4,8 % | 2,2 % | 1,21 % | 36 / 36 / 36 / 47 / 54 % |
| Zaungast | 37,2 % | 13,9 % | 5,1 % | 2,5 % | 1,29 % | 37 / 37 / 37 / 49 / 52 % |
| Wirtin | 39,9 % | 17,0 % | 7,8 % | 3,6 % | 1,58 % | 40 / 42 / 46 / 47 / 44 % |

Kurzer Abend (3 × 800 Runs; Abschnitt 1 und 2 dieselben Runs, im dritten der Alte):

| Charakter | Abschnitt 2 | 3 | Run gewonnen | gehalten je Abschnitt |
|---|---|---|---|---|
| Alleinspieler | 37,3 % | 13,8 % | 6,00 % | 37 / 37 / 44 % |
| Nullspieler | 36,9 % | 14,5 % | 5,96 % | 37 / 39 / 41 % |
| Grandspieler | 36,3 % | 13,2 % | 4,92 % | 36 / 36 / 37 % |
| Zaungast | 37,2 % | 13,9 % | 5,54 % | 37 / 37 / 40 % |
| Wirtin | 39,9 % | 17,0 % | 7,54 % | 40 / 42 / 44 % |

Einzelne Seeds, ganzer Abend (Abschnitt 2 / Run gewonnen): Alleinspieler 36,2 / 38,1 / 37,6 % und 1,25 / 1,25 / 1,75 %; Nullspieler 38,5 / 38,6 / 33,6 % und 0,75 / 0,62 / 0,38 %; Grandspieler 37,5 / 36,8 / 34,8 % und 1,25 / 1,25 / 1,12 %; Zaungast 38,5 / 37,4 / 35,6 % und 1,50 / 1,38 / 1,00 %; Wirtin 38,6 / 42,5 / 38,6 % und 2,00 / 1,38 / 1,38 %.

Je Run (ganzer Abend) 6,3 bis 7,0 Spiele und 3,4 (Zaungast) bis 5,9 Einmischen, Null-Anteil 32 bis 34 % (Nullspieler 26 %). Normale Serien 70 bis 89 %, Boss-Serien in Abschnitt 1 bis 4 60 bis 69 %.

**Befunde**:

1. Alle fünf Charaktere erreichen Abschnitt 2 in 36 bis 40 % der Runs. „Run gewonnen“ liegt für Alleinspieler, Grandspieler, Zaungast und Wirtin bei 1,2 bis 1,6 %, im Zielband.
2. **Der Nullspieler gewinnt nur 0,6 %** (vor dem Merge mit derselben Kurve 0,96 %; der Unterschied liegt knapp im Rauschen, beide unter 1 %). Ab Abschnitt 4 hält er nur 30 bis 38 %; seine Kurve liegt dort schon bei 23 bis 30 (ein gewonnenes Null ist 23), tiefer bringt wenig (0,3 Punkte je Zielwert-Punkt). Messung als Vorschlag, **nicht übernommen**: ein Einmischen mehr (`rule: (redeals: 1)`, 2 × 800 Runs, Seeds 42 und 43, nach dem Merge) hebt Abschnitt 2 von 38,6 auf 40,0 % und „Run gewonnen“ von 0,69 auf 1,12 %; dann wären die vorderen Zielwerte etwas zu heben. Das ändert seine Regel (DESIGN 9.2) und ist eine Design-Entscheidung.
3. Abstände: Grandspieler vorne rund 1,4-mal der Alleinspieler, Wirtin rund 1,2-mal, Zaungast vorne gleich. Alle außer dem Alleinspieler werden hinten flacher: Zaungast (ein Einmischen) und Wirtin (vier Plätze) werden gegen die PIMC-Gegner schwächer, der Grandspieler hält sich, aber sein Abschnitt 2 ist mit 36 % gehalten der schwerste.
4. Der kurze Abend landet bei 5 bis 7,5 % gewonnen (vorher 3 bis 13 %), für alle ähnlich. Leichter als der ganze Abend, wie in DESIGN 5.3 gewollt.

## 4. Einsatz-Leiter

```sh
cargo run --release -p skat_sim -- --seed 42 --runs 500 --bot pimc:12:3 --stake 2|5
# Seeds 42 bis 44; Einsatz 0 aus der Endmessung (3 × 800)
```

Alleinspieler, Kurve aus Abschnitt 3, nach dem Merge. Der Bot rechnet seit PR #43 bei Ansage und Einmischen mit der Schwelle `61 + n`.

| Einsatz | Gewinn ab | Runs | Abschnitt 2 | 3 | Run gewonnen | Farbe/Grand gewonnen | Null gewonnen | Null-Anteil | Punkte je Spiel |
|---|---|---|---|---|---|---|---|---|---|
| 0 | 61 | 3 × 800 | 37,3 % | 13,8 % | 1,42 % | 72,1 % | 54,4 % | 32,6 % | 22,2 |
| 2 | 63 | 3 × 500 | 35,1 % | 12,6 % | 0,80 % | 69,6 % | 54,1 % | 33,6 % | 18,9 |
| 5 | 66 | 3 × 500 | 31,2 % | 10,5 % | 0,73 % | 65,9 % | 53,1 % | 35,8 % | 16,2 |

Vor dem Merge (gleiche Kurve, 3 × 500): Einsatz 2 35,8 % und 1,47 %, Einsatz 5 31,5 % und 0,93 %. „Run gewonnen“ ist bei 1500 Runs auf rund ±0,25 Punkte genau; beide Messungen zusammen: Einsatz 2 rund 1,1 %, Einsatz 5 rund 0,8 %.

- **Ein Auge kostet rund 1,2 Punkte „Abschnitt 2“** und 1,2 Punkte Gewinnquote bei Farbe und Grand. Die Leiter ist flacher als in der Testrunde vom 01.11. (dort bei Einsatz 5 30,2 % statt 37,8 %, Siege auf ein Drittel); Einsatz 5 kostet jetzt rund die Hälfte der Siege.
- **Null**: Der einsatzbewusste Bot spielt bei Einsatz 5 mehr Null (35,8 % statt 32,6 %), die Null-Quote bleibt. Null dominiert bei Einsatz 5 nicht. Höhere Stufen (10 und mehr) nicht gemessen.
- Kein Änderungsvorschlag: Die Leiter ist endlos und soll steigen; ein Auge je Sieg wirkt spürbar, aber nicht abrupt.

## 5. Bosse

Endmessung, Boss-Serie gewonnen im ganzen Abend (alle Abschnitte, 250 bis 375 Serien je Boss; der Alte 24 bis 62):

| Boss | Faktor | Alleinspieler | Nullspieler | Grandspieler | Zaungast | Wirtin |
|---|---|---|---|---|---|---|
| Schwarzer Peter | 0,5 (Null 0,2) | 44 % | 35 % | 42 % | 41 % | 47 % |
| Oma | 0,3 (nur Alleinspieler) | 53 % | – | – | – | – |
| Schieber | 1,0 (Null 0,4) | 51 % | 53 % | 55 % | 54 % | 60 % |
| Ramscher | 1,25 | 69 % | 75 % | 63 % | 73 % | 69 % |
| Kontra-Hans | 1,5 | 77 % | 67 % | 71 % | 72 % | 71 % |
| Revolutionär | **1,0 → 1,5** (Null 1,0) | 71 % | 75 % | 69 % | 68 % | 73 % |
| Kiebitz-Mörder | **1,0 → 1,5** (Null 1,0) | 68 % | 77 % | 69 % | 69 % | 73 % |
| Zwillinge | 1,0 | 73 % | 76 % | 71 % | 73 % | 74 % |
| Der Alte (Endboss) | 1,4 | 68 % | 58 % | 81 % | 82 % | 61 % |

Im kurzen Abend (der Alte in Abschnitt 3, 190 bis 272 Serien): Alleinspieler 67 %, Nullspieler 68 %, Grandspieler 62 %, Zaungast 68 %, Wirtin 67 %.

- **Revolutionär und Kiebitz-Mörder 1,0 → 1,5**: Mit 1,0 gewann der Bot sie so oft wie eine normale Serie (79 bis 85 % außer beim Nullspieler); jetzt liegen sie mit 68 bis 74 % beim Kontra-Hans. Der Nullspieler behält 1,0 (`character_factors`), er gewann sie schon so nur zu 68 und 72 %.
- **Schieber bleibt 1,0**. In der Testrunde vom 01.11. war 0,8 vorgeschlagen; mit 3 Spielen liegt er bei 51 bis 60 % gleichauf mit der Oma (53 %), schwer, aber keine Wand. Nach dem Merge ist er beim Alleinspieler um 12 Punkte gefallen (63 → 51 %, je rund 270 Serien, also gut zwei Standardfehler); im Auge behalten.
- **Schwarzer Peter bleibt der Ausreißer** (35 bis 48 %). Mit 3 Spielen ist die Obergrenze 50 % (Nullspieler 37 %): Er verlangt mindestens einen Schneider-Sieg und kein verlorenes Spiel. Kein Faktor ändert das. Er ist der schwerste Boss, mit 41 bis 47 % aber keine Wand; beim Nullspieler (35 %) am Rand. Wenn er nicht mehr der Ausreißer sein soll, braucht er eine andere Regel (Vorschlag an `core-dev`/Design: gewonnene Spiele ohne Schneider zählen halb statt null).
- Oma, Ramscher, Kontra-Hans, Zwillinge unverändert.

## 6. Freischaltungen

```sh
cargo run --release -p skat_sim -- --seed 42 --runs 200 --unlocks --bot pimc:12:3
```

20 Folgen mit frischem Meta-Zustand, höchstens 200 Runs, der Bot spielt Alleinspieler (nach `tour_won` jeden zweiten Run Nullspieler). Nach dem Merge, Kurven aus Abschnitt 3:

| Freischaltung | Belohnung | erreicht | Runs bis dahin (Mittel) | min | max |
|---|---|---|---|---|---|
| `early_finish` (Serie im ersten Spiel) | Jungfrau | 20/20 | 1,3 | 1 | 3 |
| `grand_schneider` | Durchmarsch | 20/20 | 2,3 | 1 | 6 |
| `second_evening` | Der Wirt | 20/20 | 2,4 | 1 | 11 |
| `big_game` | Kontra | 20/20 | 3,1 | 1 | 14 |
| `boss_without_pass` (ohne Einmischen) | Der Schiedsrichter | 20/20 | 4,0 | 1 | 13 |
| `apprenticeship` | Lehrgeld | 20/20 | 5,0 | 5 | 5 |
| `hand_won` | Ouvert-Bonus | 20/20 | 6,5 | 1 | 24 |
| `third_evening` | Tournee | 20/20 | 6,5 | 1 | 21 |
| `hearts_schneider` | Der Schneider | 20/20 | 8,4 | 1 | 24 |
| `beat_kibitz_killer` | **Zaungast** | 20/20 | 9,2 | 1 | 27 |
| `beat_kontra_hans` | Auf dem Trockenen | 20/20 | 10,1 | 1 | 34 |
| `big_win` | Glückssträhne | 20/20 | 11,1 | 1 | 37 |
| `fourth_evening` | Der Stammgast | 20/20 | 11,6 | 1 | 51 |
| `grand_master` (Grand Schneider ab 144) | **Grandspieler** | 20/20 | 13,1 | 1 | 37 |
| `grand_hand` | Bierdeckel | 20/20 | 19,6 | 1 | 61 |
| `beat_grandma` | Die Oma | 20/20 | 22,6 | 1 | 83 |
| `beat_black_peter` | Schneiderbock | 20/20 | 24,4 | 1 | 63 |
| `full_till` (20 Groschen nach einer Serie) | **Wirtin** | 20/20 | 34,8 | 4 | 152 |
| `tour_won` (ganzer Abend, Alleinspieler) | **Nullspieler** | 18/20 | 48,8 | 1 | 190 |

Vor dem Merge (gleiche Kurven): `grand_master` 15,0, `full_till` 36,7, `beat_kibitz_killer` 10,8, `tour_won` 60,4 (18/20), sonst innerhalb von 1 bis 9 Runs gleich.

- **`grand_master` mit 144** kommt jetzt nach rund 13 Runs (vorher mit 96 nach 3): ein Ziel aus dem Mittelfeld, wie gewünscht.
- **`full_till`** ist mit 35 Runs im Mittel (bis 152) die späteste Charakter-Freischaltung vor dem Nullspieler. Mit Serien zu 3 Spielen gibt es weniger Groschen für übrige Spiele; 20 Groschen verlangen langes Sparen. Kein Eingriff (Platzhalter für `content`, falls die Wirtin früher kommen soll: 15 Groschen).
- `early_finish` (1,3) und `boss_without_pass` (4,0) sind leichte Einstiegsziele.
- `tour_won`: 18 von 20 Folgen in 200 Runs, im Mittel nach 49 Runs (Run gewonnen 1,4 %).

## 7. Geänderte Werte

- `core/content/run.ron`: `targets` 33/40/45, 35/40/46, 36/44/48, 40/46/54, 45/50/60 (vorher 24/30/36 bis 48/54/60).
- `core/content/characters.ron`: eigene `targets` für Nullspieler, Grandspieler, Zaungast und Wirtin (Tabelle in Abschnitt 3).
- `core/content/bosses.ron`: Revolutionär und Kiebitz-Mörder `target_factor` 1,0 → 1,5, `character_factors: {NullPlayer: 1.0}`.
- `core/src/run/content_tests.rs`: Test auf die neue Kurve.
- `docs/DESIGN.md`: 5.2 (endgültige Tabelle), 5.4 (Einsatz gemessen), 9.3 (Freischaltungen neu gemessen), 9.4 bis 9.6, 11 (Boss-Stand), 17.

## 8. Offene Punkte

- **Nullspieler unter 1 %** (Abschnitt 3, Befund 2): Entscheidung, ob er ein Einmischen mehr bekommt (gemessen +0,4 bis +0,5 Punkte „Run gewonnen“), oder ob 0,6 bis 1 % für den Spezialisten genügt.
- **Schwarzer Peter**: Obergrenze 50 % mit 3 Spielen, Regel statt Faktor (an `core-dev`/Design).
- **Schieber** nach Welle 2a beim Alleinspieler bei 51 %: bei der nächsten Relikt-Welle mitmessen.
- Der Bot ist der Maßstab, Menschen spielen Hand, Ouvert und Informations-Relikte stärker (DESIGN 17); die Statistik echter Runs gegen diese Zahlen halten.
- Schnaps-Einzelwirkung (`--consumable`), Preise von Einmischen-Gegenständen und der Alte mit Einmischen sind weiter offen.

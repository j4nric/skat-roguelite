# Große Runde Alleinspieler: steile Kneipen, zwei Leben, Bosse nachgestellt

- **Datum**: 03.11.2026
- **Branch**: `feat/balance-2026-11-03`, Endstand 9a54767 (auf `main` 908b04c, nach der kleinen Runde `docs/balance/2026-11-03-kleine-runde.md`)
- **Agent**: `balance`
- **Maßstab**: PIMC-Spieler (`--bot pimc:12:3`), Gegner je Stadtteil aus `run.ron`, alles freigeschaltet, Einsatz 0, nur der **Alleinspieler**.
- **Seeds**: 42, 43, 44. Piloten je 3 × 200 Runs (Standardfehler „gewonnen“ bei 5 % rund 0,9 Punkte, „Stadtteil 2“ rund 2 Punkte), große Runde je 3 × 2000 Runs (6000 Runs: „Stadtteil 2“ ±0,6, „gewonnen“ ±0,3, je Boss ±1,4 Punkte).
- **Zielbild** (Koordinator, 03.11.2026): Stadtteil 2 in 35 bis 40 % der Runs, Stadtteil 3 in 12 bis 18 %, 3 bis 5 % gewonnen, ein Viertel bis ein Drittel der verlorenen Runs endet in normalen Kneipen.

## Was geändert wurde

Regeln (`core-dev`, vor dieser Runde, Vorschläge der kleinen Runde): ein verlorener Ramsch zählt in der Kneipe des Ramschers einfach (`SingleLoss`); eine gewonnene Boss-Kneipe zahlt, als blieben zwei Spiele übrig, und das doppelt; Skatgott höchstens +3; Bot-Kaufwert Kräuter und Doppelkorn 1; neue Simulator-Kennzahlen und `--district`.

Zahlen (`balance`, diese Runde):

| Was | vorher | jetzt | Ort |
|---|---|---|---|
| Leben | Start 3, höchstens 5 | **Start 2**, höchstens 4 | `run.ron` `lives` |
| Groschen je übrigem Spiel | 3 | 2 | `run.ron` `economy.per_game_left` |
| Zielwerte Stadtteil 1 | 33 / 36 / 38 / 40 / Boss 45 | 45 / 55 / 65 / 75 / Boss 60 | `run.ron` `targets` |
| Zielwerte Stadtteil 2 | 40 / 42 / 44 / 46 / Boss 52 | 90 / 105 / 120 / 135 / Boss 100 | |
| Zielwerte Stadtteil 3 | 46 / 48 / 50 / 52 / Boss 60 | 145 / 170 / 195 / 205 / Boss 140 | |
| Boss-Faktoren | Schieber 0,8, Oma 0,7, Zwillinge 0,85, Peter 0,5, Ramscher 1,0, Alter 1,0 | Schieber 0,5, Oma 1,0, Zwillinge 1,2, Peter 0,7, Ramscher 1,0, Alter 1,0 | `bosses.ron` |
| Preise | Skatgott 9, Ein Auge zudrücken 8, Schieben 8, Schenken 8, Schutzengel 8, Feierabend 5 | 12, 7, 6, 6, 6, 4 | `house_rules.ron` |
| Mali der Spelunke | alle Gewicht 1 | Kein Einmischen, Klebrige Karten, Schnapsverbot 2; Kein Skat 1 | `tour.ron` |

**Abweichung vom Auftrag**: Start mit **2 Leben** statt 3 war nicht vorgegeben. Ohne ihn erreichte keine Kurve das Zielbild (Piloten unten): Mit 3 Leben endeten höchstens 13 % der verlorenen Runs in normalen Kneipen, auch mit Zielwerten bis 165. Die Boss-Zielwerte stiegen mit, weil die Boss-Kneipen sonst neben den steilen Kneipen zu leicht wurden.

Folgen in Tests (nur Zahlen): `core/src/run/content_tests.rs` (Kurve), `core/src/run/edition_tests.rs` (Hashes der ersten Kneipe für Seeds 42 und 99; die Kneipe endet später, die Karten sind gleich), `wasm/src/lib.rs` (Startleben 2), `web/e2e/smoke.mjs` (zwei Herzen in der Kopfzeile; der Playwright-Lauf steht aus).

## Befehle

```sh
cargo build --release -p skat_sim
./target/release/sim --seed 42 --runs 2000 --bot pimc:12:3     # dasselbe mit 43, 44
# Piloten: run.ron nur im Arbeitsbaum geändert, je Kurve eigenes Binary
./target/release/sim --seed 42 --runs 200 --bot pimc:12:3       # 43, 44
# Ramscher ohne Relikte, Stadtteil 1 und 3
./target/release/sim --seed 42 --runs 600 --bot pimc:12:3 --curve --boss ramscher --district 1
./target/release/sim --seed 42 --runs 600 --bot pimc:12:3 --curve --boss ramscher --district 3
# ein Relikt ab Spielbeginn, ohne Tresen (Seeds 42, 43, je 200)
./target/release/sim --seed 42 --runs 200 --bot pimc:12:3 --no-shop [--relic <id>]
./target/release/sim --seed 42 --runs 40 --unlocks
```

## 1. Piloten der Kurve

Alle mit den Zahlen 1 bis 5 oben (Leben höchstens 4, 2 Groschen je übrigem Spiel, Faktoren, Preise, Mali), je 3 × 200 Runs. „Kneipen-Ende“ ist der Anteil der verlorenen Runs, die in einer normalen Kneipe oder Spelunke enden.

| Kandidat | Zielwerte (Stadtteil 1 · 2 · 3, Boss zuletzt) | Startleben | Stadtteil 2 | Stadtteil 3 | gewonnen | Kneipen-Ende |
|---|---|---|---|---|---|---|
| A: alte Kurve | 33–40 / 45 · 40–46 / 52 · 46–52 / 60 | 3 | 58,2 % | 33,2 % | 18,8 % | 2,1 % |
| M: halber Anstieg | 36 / 41 / 45 / 49 / 45 · 48 / 53 / 58 / 63 / 52 · 59 / 66 / 73 / 80 / 60 | 3 | 58,0 % | 33,2 % | 15,3 % | 3,9 % |
| S: Vorschlag der kleinen Runde | 40 / 46 / 52 / 58 / 45 · 56 / 64 / 72 / 80 / 52 · 72 / 84 / 96 / 108 / 60 | 3 | 57,8 % | 30,5 % | 15,7 % | 5,5 % |
| X: steiler | 45 / 55 / 65 / 75 / 45 · 75 / 90 / 105 / 120 / 52 · 105 / 125 / 145 / 165 / 60 | 3 | 56,5 % | 29,0 % | 13,0 % | 13,4 % |
| L: S mit 2 Leben | wie S | 2 | 47,2 % | 25,3 % | 14,3 % | 26,1 % |
| XL: X mit 2 Leben | wie X | 2 | 44,0 % | 22,7 % | 12,2 % | 35,7 % |
| Y1 | 45 / 55 / 65 / 75 / 45 · 85 / 100 / 115 / 130 / 65 · 125 / 145 / 165 / 185 / 80 | 2 | 44,0 % | 21,2 % | 10,8 % | 37,4 % |
| Y2 | 50 / 60 / 70 / 80 / 50 · 95 / 115 / 135 / 155 / 75 · 145 / 170 / 195 / 220 / 95 | 2 | 41,3 % | 17,2 % | 7,0 % | 42,7 % |
| Z1 | wie Y2, Bosse 55 / 90 / 130 | 2 | 40,2 % | 15,8 % | 5,5 % | 41,4 % |
| **Z2** | 45 / 55 / 65 / 75 / **60** · 90 / 105 / 120 / 135 / **100** · 145 / 170 / 195 / 220 / **140** | 2 | 40,2 % | 16,0 % | 4,8 % | 35,7 % |

- Weder M noch S erreicht das Zielbild: Beide bewegen „gewonnen“ nur von 19 auf 15 bis 16 % und lassen 94 bis 96 % der Runs beim Boss enden. Die Kurve einer Kneipe ist im Run noch flacher als ohne Relikte, weil ein früher Sieg mit Relikten groß ausfällt (in Stadtteil 3 bringt ein im ersten Spiel geschaffter Sieg im Mittel 92 Punkte, jeder fünfte über 150).
- Der stärkste Hebel für „Kneipen-Ende“ sind die **Startleben** (X → XL: 13 → 36 %), für „gewonnen“ die **späten Stadtteile** (Zielwerte und Boss-Zielwerte in Stadtteil 2 und 3).
- **Gewählt**: Z2, mit der letzten Kneipe von Stadtteil 3 220 → 205 geglättet (in Z2 gewann der Bot sie nur zu 23 %).

## 2. Ergebnis der großen Runde

| Kennzahl | kleine Runde (alt, 3 × 500) | große Runde (3 × 2000) | je Seed 42 / 43 / 44 | Zielbild |
|---|---|---|---|---|
| Stadtteil 2 erreicht | 57,4 % | **42,3 %** | 43,9 / 42,0 / 41,0 | 35 bis 40 % |
| Stadtteil 3 erreicht | 34,5 % | **17,4 %** | 18,4 / 16,9 / 17,0 | 12 bis 18 % |
| Run gewonnen | 17,0 % | **5,2 %** | 6,0 / 4,7 / 4,8 | 3 bis 5 % |
| verlorene Runs in normalen Kneipen | 2,6 % | **36,1 %** | 35,4 / 36,0 / 37,0 | 25 bis 33 % |
| gehalten je Stadtteil | 57 / 60 / 49 % | 42 / 41 / 30 % | | |
| normale Kneipen gewonnen | 77 bis 86 % | 38 bis 74 % | | |
| Spiele je Run | 19,1 | 17,4 | | |

Alles am oberen Rand des Zielbilds: „Stadtteil 2“ 2 Punkte darüber, „Kneipen-Ende“ 3 Punkte darüber, „Stadtteil 3“ und „gewonnen“ darin. Nächster kleiner Schritt, falls gewünscht (nicht gemessen): Boss-Zielwert in Stadtteil 1 60 → 70 (geschätzt −2 Punkte „Stadtteil 2“, mehr Ende beim Boss).

### Je Kneipe

Summe der drei Seeds; „Leben vorher“ und „Relikte“ beim Betreten, „Groschen“ je gewonnener Kneipe.

| Kneipe | Zielwert | Versuche | gewonnen | Spiele | Einmischen | Leben vorher | Relikte | Groschen je gewonnener |
|---|---|---|---|---|---|---|---|---|
| 1-1 | 45 | 6000 | 73,7 % | 1,91 | 1,36 | 2,00 | 1,00 | 5,97 |
| 1-2 | 55 | 6000 | 71,8 % | 2,01 | 1,44 | 1,82 | 1,97 | 5,88 |
| 1-3 | 65 | 5631 | 69,7 % | 2,08 | 1,57 | 1,83 | 2,59 | 5,99 |
| 1-4 | 75 | 5020 | 64,5 % | 2,25 | 1,65 | 1,83 | 3,36 | 5,63 |
| 1-Boss | 60 | 4481 | 56,6 % | 3,00 | 2,02 | 1,88 | 3,98 | 14,61 |
| 2-1 | 90 | 2140 | 66,1 % | 2,23 | 1,79 | 2,80 | 4,98 | 5,52 |
| 2-2 | 105 | 2111 | 61,8 % | 2,36 | 1,91 | 2,68 | 5,00 | 5,36 |
| 2-3 | 120 | 2108 | 57,6 % | 2,43 | 1,95 | 2,54 | 5,00 | 5,25 |
| 2-4 | 135 | 2047 | 54,9 % | 2,52 | 2,01 | 2,40 | 5,00 | 5,04 |
| 2-Boss | 100 | 2179 | 48,0 % | 3,00 | 2,28 | 2,42 | 5,00 | 14,59 |
| 3-1 | 145 | 846 | 53,4 % | 2,55 | 2,08 | 3,26 | 5,00 | 5,27 |
| 3-2 | 170 | 854 | 45,5 % | 2,65 | 2,26 | 3,01 | 5,00 | 4,99 |
| 3-3 | 195 | 877 | 43,2 % | 2,69 | 2,23 | 2,67 | 5,00 | 4,74 |
| 3-4 | 205 | 855 | 37,9 % | 2,76 | 2,29 | 2,43 | 5,00 | 4,59 |
| 3-Boss | 140 | 871 | 35,7 % | 3,00 | 2,03 | 2,32 | 5,00 | 14,49 |
| 1-4 Spelunke | 94 | 61 | 45,9 % | 2,57 | 2,00 | 3,03 | 3,94 | 5,50 |
| 2-1 Spelunke | 113 | 399 | 47,6 % | 2,48 | 1,59 | 3,35 | 4,91 | 6,46 |
| 2-2 Spelunke | 131 | 428 | 46,9 % | 2,56 | 1,77 | 3,39 | 4,99 | 5,34 |
| 2-3 Spelunke | 150 | 350 | 38,3 % | 2,69 | 1,75 | 3,41 | 5,00 | 5,12 |
| 2-4 Spelunke | 169 | 292 | 38,7 % | 2,72 | 1,98 | 3,39 | 5,00 | 4,86 |
| 3-1 Spelunke | 181 | 200 | 35,5 % | 2,80 | 2,02 | 3,67 | 5,00 | 5,35 |
| 3-2 Spelunke | 213 | 192 | 29,7 % | 2,81 | 1,93 | 3,53 | 5,00 | 5,11 |
| 3-3 Spelunke | 244 | 150 | 22,0 % | 2,88 | 1,96 | 3,52 | 5,00 | 4,73 |
| 3-4 Spelunke | 256 | 114 | 20,2 % | 2,92 | 2,11 | 3,46 | 5,00 | 4,17 |

### Je Stadtteil

| Stadtteil | Runs | Leben zu Beginn | Leben vor dem Boss | Relikte zu Beginn | Relikte vor dem Boss | verlorene Kneipen je Run | Ende in Kneipe | Ende in Spelunke | Ende beim Boss |
|---|---|---|---|---|---|---|---|---|---|
| 1 | 6000 | 2,00 | 1,88 | 1,00 | 3,98 | 1,13 | 1519 | 0 | 1942 |
| 2 | 2539 | 2,89 | 2,42 | 4,97 | 5,00 | 1,65 | 360 | 0 | 1133 |
| 3 | 1046 | 3,34 | 2,32 | 5,00 | 5,00 | 2,26 | 175 | 0 | 560 |

### Bosse

Boss-Kneipe gewonnen im Run (6000 Runs, Zielwerte 60 / 100 / 140 vor dem Faktor):

| Boss | Faktor | alle | Stadtteil 1 | Stadtteil 2 und 3 | je Seed 42 / 43 / 44 | kleine Runde (alte Faktoren) |
|---|---|---|---|---|---|---|
| Der Ramscher | 1,0 | 51,1 % (1306) | 56,6 % | 39,6 % | 50,4 / 53,7 / 49,3 | 35,5 % |
| Der Schieber | 0,5 | 51,5 % (1366) | 50,1 % | 54,1 % | 53,2 / 48,8 / 52,1 | 52,1 % |
| Der Schwarze Peter | 0,7 | 51,2 % (1316) | 51,9 % | 49,9 % | 52,7 / 51,9 / 49,0 | 63,9 % |
| Die Oma | 1,0 | 57,0 % (1368) | 59,5 % | 51,5 % | 58,3 / 54,8 / 58,2 | 72,4 % |
| Die Zwillinge | 1,2 | 58,3 % (1304) | 65,4 % | 44,3 % | 60,6 / 57,9 / 56,5 | 71,8 % |
| Der Alte (Endboss) | 1,0 | 35,7 % (871) | – | 35,7 % | 38,9 / 32,9 / 35,0 | 49,4 % |

**Ramscher mit einfacher Wertung, ohne Relikte** (`--curve --boss ramscher`, 600 Boss-Kneipen, Seed 42): Stadtteil 1 bei Zielwert 60 58,5 % (Obergrenze 72,7 %), Stadtteil 3 bei 140 33,7 % (Obergrenze 70,3 %). Im Run 57 und 40 %. Er liegt damit bei den anderen Bossen; **sein Faktor bleibt 1,0**.

Die Spanne der Bosse ist von 37 Punkten (35 bis 72 %) auf 7 Punkte (51 bis 58 %) geschrumpft, der Endboss liegt bewusst darunter.

### Wirtschaft

Je Run (6000 Runs):

| Einnahmen | Groschen | | Ausgaben | Groschen | Anteil |
|---|---|---|---|---|---|
| Startkapital | 4,00 | | Relikte | 22,28 | 60,4 % |
| Auszahlungen der Kneipen | 31,56 (vorher 55,75) | | Zinken (Auslage) | 7,04 | 19,1 % |
| Verkäufe | 2,39 | | Schnäpse | 6,83 | 18,5 % |
| aus Inhalten (Relikte, Flüche, Begegnungen) | 2,44 | | Kartenmaler | 0,46 | 1,3 % |
| | | | Neuauslage | 0,18 | 0,5 % |
| | | | Karte reinigen | 0,12 | 0,3 % |
| | | | Zinken neu auslosen | 0,00 | 0,0 % |
| | | | **Summe** | **36,92** (vorher 64,60) | |

- Gewonnene normale Kneipe 4,6 bis 6,0 Groschen, gewonnene **Boss-Kneipe 14,6** (vorher 6,45). 4,0 Tresen-Besuche je Run.
- Relikte beim Boss 1 im Mittel 3,98, ab Stadtteil 2 voll (4,97). Vorher waren die Plätze schon beim Betreten von Stadtteil 2 zu 93 % voll; jetzt füllt sich der letzte Platz erst rund um den ersten Boss.
- Schnäpse gekauft / getrunken: Kräuter 2761 / 1641, Doppelkorn 1436 / 811 (beide weiter rund 40 % ungetrunken, trotz Kaufwert 1), Wegbier 3588 / 3202, Kurzer 3339 / 3252, Blue Curaçao 1803 / 1634, Konterbier 1681 / 1462.
- Häufigste Relikt-Käufe: Ein Auge zudrücken 2149, Schieben 1876, Schenken 1163, Alte Oberin 1151, Luschenkönig 1134, Kleinvieh 1130, Feierabend 1078. Skatgott 61 (bei 12 Groschen fast nie bezahlbar), Ehrenvorsitz 11.

## 3. Einzelne Relikte ohne Tresen

`--no-shop --relic <id>`, Seeds 42 und 43, je 200 Runs (400 je Zeile, gepaarte Karten; Differenzen unter rund 4 Punkten sind Rauschen). Ohne Tresen ist der Abend jetzt hart: „gewonnen“ 0 % ohne Relikt.

| Relikt | Preis jetzt | Stadtteil 2 | Δ | Stadtteil 3 | Δ | gewonnen | Δ |
|---|---|---|---|---|---|---|---|
| ohne (nur Zähler) | – | 34,2 % | | 6,2 % | | 0,0 % | |
| Skatgott (höchstens +3) | 12 | 45,5 % | +11,2 | 20,0 % | **+13,8** | 7,0 % | **+7,0** |
| **Schutzengel** | **6** | 49,0 % | **+14,8** | 15,2 % | +9,0 | 2,8 % | +2,8 |
| Drei drücken | 9 | 45,2 % | +11,0 | 11,2 % | +5,0 | 1,2 % | +1,2 |
| Kleinvieh | 9 | 44,0 % | +9,8 | 10,0 % | +3,8 | 1,8 % | +1,8 |
| Alte Oberin | 9 | 41,8 % | +7,5 | 10,0 % | +3,8 | 1,2 % | +1,2 |
| Ein Auge zudrücken | 7 | 40,5 % | +6,2 | 8,5 % | +2,2 | 0,5 % | +0,5 |
| Schenken | 6 | 36,0 % | +1,8 | 9,5 % | +3,2 | 0,8 % | +0,8 |
| Feierabend | 4 | 34,2 % | ±0,0 | 6,5 % | +0,2 | 0,5 % | +0,5 |

- **Schutzengel**: Mit zwei Startleben und Kneipen, die Leben kosten, ist er vom wirkungslosen (+0,4) zum zweitstärksten Relikt geworden; der neue Preis 6 ist zu niedrig. Vorschlag: zurück auf 8 oder 9.
- **Skatgott** bleibt trotz Deckel +3 der Ausreißer (+14 auf „Stadtteil 3“), ist mit 12 Groschen und Legendär aber im Run selten (61 Käufe in 6000 Runs).
- **Feierabend** bleibt ohne Wirkung für den Bot, auch bei 4 Groschen.

## 4. Freischaltungen

`--unlocks --runs 40` (Heuristik-Bot, 20 Folgen): alle 20 erreichbar. Nullspieler (Run gewinnen) 14/20 nach 15,9 Runs (vorher 18/20 nach 8,7), Schneiderin 8/20 nach 21,4, Bruderherz 10/20 nach 15,7, Drückeberger 18/20 nach 7,3. Zweiter und Dritter Abend (Kneipe 3 und 5 erreichen) weiter nach 1 bis 2 Runs.

## 5. Offen für später

1. **Andere Charaktere ungemessen**: Der Nullspieler behält seine Zielwerte 18 bis 30 und dürfte mit zwei Leben und steilen Gegnern der anderen Charaktere jetzt viel zu leicht sein; Drückeberger, Schneiderin und Bruderherz spielen die neue Kurve ungemessen.
2. **Schutzengel** Preis 6 → 8 oder 9 (oben).
3. **Spelunke** mit Zielwert × 1,25 auf der steilen Kurve: in Stadtteil 3 nur 20 bis 36 % gewonnen. Vorschlag `spelunke.target_factor` 1,25 → 1,1. Die Bot-Regel „Spelunke nur mit 3 Leben und 3 Relikten“ (an `core-dev`) passt nicht mehr zu zwei Startleben.
4. **Zwillinge und Ramscher** fallen nach Stadtteil 1 stark ab (65 → 44 %, 57 → 40 %); der Alte liegt mit 36 % deutlich unter den anderen. Eigene Faktoren je Stadtteil gibt es nicht; erst mit Menschen prüfen.
5. **Feierabend** wirkt beim Bot nicht (5-%-Regel verfehlt); Kräuter und Doppelkorn bleiben zu 40 % ungetrunken.
6. Am oberen Rand des Zielbilds: Boss-Zielwert Stadtteil 1 60 → 70 als nächster kleiner Schritt.

## Schlussfolgerung

Die Kneipentour liegt für den guten Spieler im Zielbild (42 / 17 / 5,2 %, ein gutes Drittel der Verluste in normalen Kneipen statt 3 %). Dafür brauchte es eine viel steilere Kurve als vorgeschlagen (bis 205 statt 108) und zwei Startleben; die Bosse liegen nach Ramscher-Regel und Faktoren eng beieinander (51 bis 58 %, der Alte 36 %).

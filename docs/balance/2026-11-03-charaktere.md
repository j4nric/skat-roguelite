# Charaktere auf der steilen Kurve: Nullspieler, Drückeberger, Schneiderin, Bruderherz

- **Datum**: 03.11.2026
- **Branch**: `feat/balance-characters` (von `main` 69c1775, nach der großen Runde `docs/balance/2026-11-03-grosse-runde.md`)
- **Agent**: `balance`
- **Maßstab**: wie in der großen Runde: PIMC-Spieler (`--bot pimc:12:3`), Gegner je Stadtteil aus `run.ron`, alles freigeschaltet, Einsatz 0. Vergleich immer mit dem Alleinspieler auf denselben Seeds.
- **Seeds**: 42, 43, 44. Ist-Stand und Endmessung je 3 × 300 Runs (900 Runs: Standardfehler „Stadtteil 2“ rund 1,6 Punkte, „Stadtteil 3“ rund 1,3, „gewonnen“ rund 0,8, je Boss rund 3,5), Piloten je 3 × 200.
- **Zielbild** (wie für den Alleinspieler, 03.11.2026): Stadtteil 2 in 35 bis 40 % der Runs, Stadtteil 3 in 12 bis 18 %, 3 bis 5 % gewonnen, ein Viertel bis ein Drittel der verlorenen Runs endet in normalen Kneipen.

## Befehle

```sh
cargo build --release -p skat_sim
./target/release/sim --seed 42 --runs 300 --bot pimc:12:3 --character null   # 43, 44; ebenso declarer, shirker, tailoress, heart_brother
# Piloten: characters.ron / bosses.ron nur im Arbeitsbaum geändert, je Stand eigenes Binary
./target/release/sim --seed 42 --runs 200 --bot pimc:12:3 --character null   # 43, 44
```

Neu im Simulator (nur Zählung, keine Regel): Spiele je Spielart (Anteil, Gewinnquote, Schneider unter den gewonnenen, Handspiele, Punkte je Spiel, Spielwert der gewonnenen) und eingelöste und zurückgekehrte Gutscheine je Run (`sim/src/runs.rs`, `render_kinds`).

## 1. Ist-Stand

3 × 300 Runs, Inhalte wie auf `main` 69c1775. „Kneipen-Ende“: Anteil der verlorenen Runs, die in einer normalen Kneipe enden (Spelunken kamen in keinem verlorenen Run als Ende vor).

| Charakter | Zielwerte | Stadtteil 2 | Stadtteil 3 | gewonnen | Kneipen-Ende | Spiele je Run | je Seed 42 / 43 / 44 (St. 2 · St. 3 · gewonnen) |
|---|---|---|---|---|---|---|---|
| Alleinspieler (Maßstab) | `run.ron` | 40,6 % | 16,1 % | 6,0 % | 35,6 % | 17,0 | 41,3 · 17,0 · 6,0 / 42,7 · 15,0 · 5,7 / 37,7 · 16,3 · 6,3 |
| **Nullspieler** | eigene, 18 bis 30 | **59,9 %** | **39,3 %** | **29,7 %** | 18,8 % | 18,0 | 60,7 · 39,3 · 27,7 / 62,0 · 42,0 · 33,0 / 57,0 · 36,7 · 28,3 |
| Drückeberger | `run.ron` | 44,6 % | 17,3 % | 5,3 % | 31,5 % | 17,7 | 43,3 · 17,0 · 4,0 / 50,3 · 20,7 · 7,0 / 40,0 · 14,3 · 5,0 |
| **Schneiderin** | `run.ron` | **47,2 %** | **20,9 %** | **7,9 %** | 30,8 % | 18,4 | 49,7 · 21,3 · 7,3 / 50,3 · 21,7 · 8,7 / 41,7 · 19,7 · 7,7 |
| Bruderherz | `run.ron` | 36,3 % | 15,3 % | 4,6 % | 38,8 % | 16,4 | 35,7 · 14,0 · 3,0 / 38,0 · 15,7 · 3,3 / 35,3 · 16,3 · 7,3 |

- **Nullspieler** viel zu leicht, wie erwartet: Er gewinnt jede normale Kneipe zu 81 bis 89 %, 30 % der Runs.
- **Schneiderin** deutlich zu leicht (gegen den Alleinspieler +6,6 / +4,8 / +1,9 Punkte).
- **Drückeberger** knapp über dem Alleinspieler (+4,0 / +1,2 / −0,7), innerhalb von rund zwei Standardfehlern; nicht klar daneben, keine Änderung.
- **Bruderherz** im Zielbild (−4,3 / −0,8 / −1,4 gegen den Alleinspieler); keine Änderung.

### Bosse im Ist-Stand

Boss-Kneipe gewonnen im Run (alle Stadtteile; 180 bis 290 Versuche je Zelle, ±3,5 Punkte):

| Boss | Alleinspieler | Nullspieler (alte Faktoren) | Drückeberger | Schneiderin | Bruderherz |
|---|---|---|---|---|---|
| Der Ramscher | 43,5 % | **87,7 %** | 46,4 % | 53,1 % | 47,2 % |
| Der Schieber | 47,9 % | **34,0 %** (0,35) | 53,1 % | 49,6 % | 40,3 % |
| Der Schwarze Peter | 47,7 % | 67,7 % (0,2) | 45,9 % | 58,2 % | 47,8 % |
| Die Oma | 62,1 % | 75,3 % (0,9) | 59,8 % | 62,4 % | 63,2 % |
| Die Zwillinge | 57,1 % | 72,9 % | 58,5 % | 56,8 % | 55,0 % |
| Der Alte | 43,9 % | 75,4 % | 36,4 % | 43,6 % | 35,7 % |

## 2. Prüfungen je Charakter

### Drückeberger: Marken

Je Run (900 Runs):

| Kennzahl | Drückeberger | Alleinspieler |
|---|---|---|
| Pfandmarken eingelöst | 4,65 (9,74 Groschen) | 0,16 (0,48 Groschen, aus Zinken) |
| Totenmarken eingelöst | 1,14 | 0,02 |
| Narrenmarken eingelöst | 1,10 | 0,02 |
| Marken zurückgekehrt | 6,90 | 0 |
| Groschen aus Inhalten (Marken, Relikte, Begegnungen) | 11,5 | 2,4 |
| Groschen für Zinken / Schnäpse | 11,1 / 9,3 | 6,8 / 6,7 |
| Tresen-Besuche | 4,9 | 4,0 |
| Handspiele (Farbe) | 9 bis 12 % | 7 bis 11 % |

Fast jede eingelöste Marke kehrt zurück (6,90 von 6,89). Die Pfandmarken bringen rund **+9 Groschen je Run**, gut ein Fünftel mehr Einnahmen als beim Alleinspieler; er gibt sie für Zinken und Schnäpse aus (die Relikt-Plätze sind ohnehin voll). Im Ergebnis liegt er trotzdem nur knapp über dem Alleinspieler: Der Skatpfleger statt des Zählers und „Hand ohne Multiplikator“ gleichen das aus. **Nicht kaputt.** Auffällig: Der Bot spielt mit ihm genauso oft Hand wie mit dem Alleinspieler, obwohl Hand bei ihm nichts bringt (an `core-dev`, unten).

### Schneiderin: Schneider

| Kennzahl | Schneiderin | Alleinspieler |
|---|---|---|
| gewonnene Farbspiele und Grands mit Schneider | 50,3 % | 39,4 % |
| davon ohne Schneider (−1 Multiplikator) | 49,7 % | 60,6 % |
| Grand: mit Schneider unter den gewonnenen | 57,2 % | 45,4 % |
| Spielwert eines gewonnenen Grands | 90,5 | 81,0 |
| Punkte je Spiel (alle) | 36,6 | 31,3 |

Aufdruck +3 auf den Zehnern und Stichfest heben die Schneider-Quote um 11 Punkte auf die Hälfte; damit zählt ihre Regel im Mittel klar für sie (+2 bei jeder zweiten, −1 bei der anderen Hälfte). Der Bot spielt dabei nicht auf Schneider, er spielt wie immer.

### Bruderherz: Rot und Flüche

| Kennzahl | Bruderherz | Bruderherz ohne Flüche (Pilot, 3 × 200) | Alleinspieler |
|---|---|---|---|
| Anteil Rot-Spiele (an allen Spielen) | 8,7 % | 9,2 % | 8,7 % |
| Rot: Spielwert eines gewonnenen / Punkte je Spiel | 54,6 / +22,2 | 55,7 / +27,4 | 38,5 / +15,9 |
| Eichel: Punkte je Spiel | +12,7 | +18,8 | +16,7 |
| Schellen: Punkte je Spiel | +8,8 | +15,0 | +13,0 |
| Grand: Punkte je Spiel | +42,9 | +43,3 | +45,1 |
| Stadtteil 2 / 3 / gewonnen | 36,3 / 15,3 / 4,6 % | 42,2 / 18,5 / 6,5 % | 40,6 / 16,1 / 6,0 % |

- Der Bot sagt **Rot nicht öfter an** als beim Alleinspieler (8,7 %): Er bewertet das Blatt ohne die Modifikationen. Ein Rot-Spiel bringt bei ihm 42 % mehr, er nutzt es nicht. Ein Spieler, der auf Rot spielt, käme weiter (an `core-dev`).
- **Flüche**: kosten ihn 5,9 Punkte „Stadtteil 2“, 3,2 „Stadtteil 3“ und 1,9 „gewonnen“ (gegen den Pilot ohne Flüche, Siegel und Winkel bleiben); Eichel und Schellen bringen je Spiel 6 Punkte weniger. Die 5-%-Regel ist erfüllt.
- Kartenmaler und Reinigen 0 Groschen (er darf seine Flüche nicht reinigen); Zinken 6,1 Groschen je Run (Alleinspieler 6,8).

## 3. Piloten

Je 3 × 200 Runs; St. = Stadtteil, Zielwerte je Stadtteil vier Kneipen / Boss.

| Kandidat | Zielwerte | Boss-Faktoren Nullspieler | St. 2 | St. 3 | gewonnen | Kneipen-Ende |
|---|---|---|---|---|---|---|
| Nullspieler N1 | wie der Alleinspieler | Schieber 0,35, Oma 0,9, Peter 0,2 | 46,7 % | 23,0 % | 9,5 % | 33,1 % |
| Nullspieler N2 | 50 / 60 / 70 / 85 / 65 · 100 / 115 / 130 / 150 / 110 · 160 / 185 / 215 / 225 / 155 (× 1,1) | Schieber 0,2, Ramscher 2,0 | 40,5 % | 19,3 % | 7,2 % | 34,3 % |
| Schneiderin T1 | 50 / 65 / 75 / 85 / 70 · 105 / 120 / 140 / 155 / 115 · 165 / 195 / 225 / 235 / 160 (× 1,15) | – | 40,8 % | 16,0 % | 4,2 % | 35,3 % |
| Schneiderin T2 | wie T1, 1-4 90, Boss 1 75 | – | 39,3 % | 15,3 % | 3,8 % | 35,4 % |

N2 war in Stadtteil 3 und bei „gewonnen“ noch zu leicht, deshalb in der Endmessung Stadtteil 2 × 1,2 und Stadtteil 3 × 1,25 (N3). T2 lag im Zielbild und ging unverändert in die Endmessung.

Bosse beim Nullspieler in den Piloten:

| Boss | Ist (alte Kurve) | N1 | N2 | Endmessung |
|---|---|---|---|---|
| Der Ramscher | 87,7 % (1,0) | 86,5 % (1,0) | 71,5 % (2,0) | 65,7 % (2,0): St. 1 74,8, ab St. 2 44,4 |
| Der Schieber | 34,0 % (0,35) | 28,9 % (0,35) | 31,9 % (0,2) | 31,6 % (0,2) |
| Der Schwarze Peter | 67,7 % (0,2) | 56,5 % | 53,6 % | 51,6 % |
| Die Oma | 75,3 % (0,9) | 61,1 % | 57,4 % | 61,2 % |
| Die Zwillinge | 72,9 % (1,2) | 62,8 % | 56,3 % | 58,5 % |
| Der Alte | 75,4 % (1,0) | 46,3 % | 43,0 % | 43,3 % |

- **Schieber**: Der Nullspieler gewinnt ihn zu 29 bis 34 %, ob der Zielwert 8 oder 21 ist. Der Faktor wirkt kaum; schwer macht ihn, dass der Bot unter „nur Hand“ Null Hand nur mit höchstens zwei offenen Karten spielt und sonst Farbe ohne Unter. 0,35 → 0,2 bleibt als kleine Hilfe.
- **Ramscher**: Mit abgestuften Untern hat der Nullspieler im Ramsch keinen Trumpf und gewinnt 83 % seiner Ramsch-Spiele. 1,0 → 2,0 senkt die Boss-Kneipe von 87 auf 66 %, in Stadtteil 1 bleibt sie mit 75 % die leichteste.
- **Peter 0,2, Oma 0,9** passen zur neuen Kurve (52 und 61 %, Alleinspieler 48 und 62 %).

## 4. Endmessung und Vergleich

3 × 300 Runs, Seeds 42 bis 44. Drückeberger und Bruderherz unverändert (ihre Werte aus Abschnitt 1, die Änderungen betreffen sie nicht).

| Charakter | | Stadtteil 2 | Stadtteil 3 | gewonnen | Kneipen-Ende | Spiele je Run |
|---|---|---|---|---|---|---|
| Alleinspieler | Maßstab | 40,6 % | 16,1 % | 6,0 % | 35,6 % | 17,0 |
| Nullspieler | vorher | 59,9 % | 39,3 % | 29,7 % | 18,8 % | 18,0 |
| | **nachher** | **38,7 %** | **17,6 %** | **6,1 %** | **39,6 %** | 16,9 |
| | je Seed 42 / 43 / 44 | 41,0 / 40,3 / 34,7 | 18,0 / 18,7 / 16,0 | 7,3 / 6,7 / 4,3 | | |
| Schneiderin | vorher | 47,2 % | 20,9 % | 7,9 % | 30,8 % | 18,4 |
| | **nachher** | **39,6 %** | **16,1 %** | **4,4 %** | **37,1 %** | 17,1 |
| | je Seed 42 / 43 / 44 | 40,7 / 41,7 / 36,3 | 15,0 / 15,0 / 18,3 | 3,7 / 3,7 / 6,0 | | |
| Drückeberger | unverändert | 44,6 % | 17,3 % | 5,3 % | 31,5 % | 17,7 |
| Bruderherz | unverändert | 36,3 % | 15,3 % | 4,6 % | 38,8 % | 16,4 |
| Zielbild | | 35 bis 40 % | 12 bis 18 % | 3 bis 5 % | 25 bis 33 % | |

Je Kneipe nachher (gewonnen, Summe der drei Seeds):

| Kneipe | Nullspieler Zielwert | gewonnen | Schneiderin Zielwert | gewonnen | Alleinspieler Zielwert | gewonnen |
|---|---|---|---|---|---|---|
| 1-1 bis 1-4 | 50 / 65 / 75 / 85 | 70 / 70 / 63 / 64 % | 50 / 65 / 75 / 90 | 74 / 72 / 66 / 65 % | 45 / 55 / 65 / 75 | 75 / 74 / 69 / 66 % |
| 1-Boss | 70 | 55 % | 75 | 53 % | 60 | 54 % |
| 2-1 bis 2-4 | 110 / 125 / 145 / 160 | 64 / 64 / 57 / 55 % | 105 / 120 / 140 / 155 | 63 / 59 / 59 / 51 % | 90 / 105 / 120 / 135 | 66 / 61 / 59 / 53 % |
| 2-Boss | 120 | 52 % | 115 | 48 % | 100 | 46 % |
| 3-1 bis 3-4 | 180 / 210 / 245 / 255 | 49 / 40 / 35 / 30 % | 165 / 195 / 225 / 235 | 51 / 41 / 38 / 30 % | 145 / 170 / 195 / 205 | 62 / 50 / 45 / 37 % |
| 3-Boss | 175 | 43 % | 160 | 36 % | 140 | 44 % |

Bosse nachher (Schneiderin): Ramscher 45,7 %, Schieber 47,2 %, Peter 54,1 %, Oma 59,9 %, Zwillinge 51,1 %, der Alte 36,4 %.

Alle fünf Charaktere liegen jetzt innerhalb von rund 5 Punkten „Stadtteil 2“ beieinander (36 bis 45 %), „gewonnen“ 4,4 bis 6,1 %. „Gewonnen“ liegt beim Nullspieler (6,1 %) wie beim Alleinspieler (6,0 %) auf diesen Seeds etwas über 5 % (die große Runde mit 6000 Runs maß beim Alleinspieler 5,2 %). „Kneipen-Ende“ liegt bei allen außer dem Drückeberger über einem Drittel (37 bis 40 %), wie beim Alleinspieler.

## 5. Was geändert wurde

| Was | vorher | jetzt | Ort |
|---|---|---|---|
| Zielwerte Nullspieler | 18 / 20 / 20 / 23 / 23 · 20 / 23 / 23 / 27 / 27 · 23 / 27 / 27 / 30 / 30 | **50 / 65 / 75 / 85 / 70 · 110 / 125 / 145 / 160 / 120 · 180 / 210 / 245 / 255 / 175** (Alleinspieler × 1,15 / 1,2 / 1,25) | `characters.ron` |
| Zielwerte Schneiderin | wie der Alleinspieler | **50 / 65 / 75 / 90 / 75 · 105 / 120 / 140 / 155 / 115 · 165 / 195 / 225 / 235 / 160** (rund × 1,15) | `characters.ron` |
| Schieber, Faktor Nullspieler | 0,35 | 0,2 | `bosses.ron` |
| Ramscher, Faktor Nullspieler | – (1,0) | 2,0 | `bosses.ron` |
| Drückeberger, Bruderherz | Kurve des Alleinspielers | unverändert | |

Tests (nur Zahlen): `core/src/run/content_tests.rs` hält die Kurven von Nullspieler und Schneiderin fest und dass Drückeberger und Bruderherz die von `run.ron` spielen.

## 6. Offen und an `core-dev`

1. **Bot und Ausgaben**: Die Ansage-Bewertung des Bots (`suit_expected_score`, `null_expected_score`) kennt die Modifikationen der Ausgabe nicht. Bruderherz sagt deshalb nicht öfter Rot an (8,7 % wie der Alleinspieler, obwohl ein Rot-Spiel bei ihm 42 % mehr bringt); der Drückeberger spielt so oft Hand wie der Alleinspieler, obwohl Hand bei ihm nichts zählt. Beide Charaktere sind für einen menschlichen Spieler, der seine Regel nutzt, vermutlich leichter als gemessen.
2. **Nullspieler spielt verlustreiche Farbspiele**: 32 % seiner Spiele sind Farbspiele ohne Unter, gewonnen zu 49 bis 58 %, im Mittel −9 bis −27 Punkte je Spiel (Null +60). Die Gewinnquoten `SUIT_WIN_PERCENT` gelten für Blätter mit Untern. Wird das behoben, wird er stärker und seine Kurve muss nachgemessen werden.
3. **Nullspieler beim Schieber** 32 %, unabhängig vom Faktor (Bot-Regel `NULL_HAND_MAX_EXPOSED` 2). Wenn das so bleiben soll, ist der Schieber für ihn der eigentliche Gegner; sonst die Bot-Regel prüfen.
4. **Nullspieler beim Ramscher**: auch mit Faktor 2,0 in Stadtteil 1 75 %. Höher als 2 erlaubt die Leitlinie in `core/src/run/tests.rs` für `target_factor` nicht; ob das auch für `character_factors` gelten soll, ist offen. Alternative: den Ramscher für den Nullspieler aus dem Pool nehmen (`characters`), eine Inhaltsentscheidung.
5. **Drückeberger**: rund +9 Groschen je Run aus zurückkehrenden Pfandmarken, im Ergebnis aber nur knapp über dem Alleinspieler. Keine Änderung; wenn der Bot Hand meidet (Punkt 1), neu messen. Mögliche Stellschraube dann: Pfandmarke 2 → 1 Groschen.
6. **Kneipen-Ende** bei Nullspieler, Schneiderin und Bruderherz 37 bis 40 %, etwas über dem Zielbild (wie beim Alleinspieler, 36 %). Gemeinsamer Hebel, falls gewünscht: die Bosse etwas schwerer, nicht die Kurven.
7. Stichprobe: 900 Runs je Charakter; eine große Runde (3 × 2000) für die neuen Kurven steht aus.

## Schlussfolgerung

Mit der steilen Kurve des Alleinspielers lagen Drückeberger und Bruderherz ohne Änderung im Zielbild, die Schneiderin rund 15 % zu leicht und der Nullspieler mit seinen alten Zielwerten weit daneben (30 % gewonnen). Nach eigenen Kurven (Nullspieler 1,15 bis 1,25 × Alleinspieler, Schneiderin rund 1,15 ×) und zwei Boss-Faktoren für den Nullspieler liegen alle fünf Charaktere bei 36 bis 45 % „Stadtteil 2“, 15 bis 18 % „Stadtteil 3“ und 4 bis 6 % „gewonnen“. Die Charakter-Regeln wirken messbar (Schneider 50 statt 39 %, Flüche −6 Punkte „Stadtteil 2“, +9 Groschen je Run aus Marken), aber der Bot spielt sie nicht aus.

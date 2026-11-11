# Neue Charaktere: Der Zocker, Die Ramschtante und der Nachtboss Der Totengräber

- **Datum**: 10.11.2026 (Messungen bis 11.11.2026)
- **Branch**: `feat/zocker-oma-nightboss`
- **Agent**: `content`
- **Art**: Zwei neue Charaktere mit eigener Kurve (`characters.ron`), ein Boss nur für die lange Nacht (`bosses.ron`, `run.ron` `night_bosses`), zwei Startrelikte (`house_rules.ron`), zwei Freischaltungen (`unlocks.ron`). Kurven, Ausgabe und Regel des Zockers wurden in dieser Runde gestimmt; die übrigen Charaktere sind unverändert.
- **Zielbild Tag** (je Charakter): Stadtteil 2 35 bis 40 %, Stadtteil 3 12 bis 18 %, gewonnen 3 bis 5 %, höchstens 20 % der Runs enden in den ersten beiden Kneipen.
- **Maßstab**: wie Runde 6. PIMC-Spieler (`--bot pimc:12:3`), mit Tresen. Iterationen 3 × 300 Runs (Seeds 42 bis 44), Endprüfung 7 × 300 Runs (Seeds 42 bis 48, 2100 Runs; Standardfehler rund 1,1 / 0,8 / 0,4 Punkte für Stadtteil 2 / 3 / gewonnen). Nacht: je 3 × 700 Runs (Seeds 42 bis 44), jeder gewonnene Run geht in die Nacht.
- **„Ende 1-1/1-2“**: Anteil der Runs, die in Kneipe 1-1 oder 1-2 enden (Versuche × Verlustquote der beiden Kneipen).

## Befehle

```sh
cargo build --release -p skat_sim
./target/release/sim --seed 42 --runs 300 --bot pimc:12:3 --character gambler       # Seeds 42 bis 48
./target/release/sim --seed 42 --runs 300 --bot pimc:12:3 --character ramsch_aunt   # Seeds 42 bis 48
# Nacht: nur der Totengräber in den Boss-Kneipen der Nacht, zum Vergleich der Alte und die normale Ziehung
./target/release/sim --seed 42 --runs 700 --bot pimc:12:3 --character declarer --night --night-boss gravedigger   # null, gambler, ramsch_aunt; Seeds 42 bis 44
./target/release/sim --seed 42 --runs 700 --bot pimc:12:3 --character declarer --night --night-boss the_old_man
./target/release/sim --seed 42 --runs 700 --bot pimc:12:3 --character declarer --night
```

## 1. Entwurf

### Der Zocker (`Gambler`, englisch „The Gambler“)

Setzt alles auf eine Karte. Hebel: **Stufen** (Hand) und **Grundwert** über die Trumpflänge (Wandelkarten).

- **Regel**: Ein gewonnenes Handspiel zählt zwei Stufen für Hand statt einer (`OnScore(when: All([Won, HandGame, Not(RamschGame)]), then: [AddLevels(1)])`, auch Null Hand). Den Skat nimmt er nur einmal je Kneipe auf (`SkatPickupsPerPub(1)`, danach `RunError::SkatUsedUp`); danach ist jedes Spiel der Kneipe Hand.
- **Ausgabe**: Eichel- und Grün-Sieben Wandelkarten (sie werden meist Trumpf), die Schellen-Daus Durchsichtig.
- **Startrelikt**: Knobelbecher (`dice_cup`): Ein verlorenes Handspiel zählt halb.
- **Freischaltung**: Hoch gepokert (`big_hand`): ein Handspiel mit mindestens 96 Punkten gewinnen; gibt auch den Knobelbecher frei.

### Die Ramschtante (`RamschAunt`, englisch „Aunt Ramsch“)

Hat schon jeden Ramsch gesehen. Hebel: **Grundwert** (die Augen des Verlierers im Ramsch) und **Stufen** (Jungfrau, Spitzendeckchen).

- **Regel**: Vor dem Skat darf sie statt einer Ansage Ramsch spielen (`OfferRamsch`), gewertet wie beim Ramscher: Wer die meisten Augen hat, verliert; hat sie nicht die meisten, zählen die Augen des Verlierers für sie, +1 Stufe Jungfrau. Verloren zählt er einfach.
- **Ausgabe**: Daus und Zehnen abgegriffen (Ramponiert 5): weniger Augen im Ramsch, schwächer in Farbspiel und Grand.
- **Startrelikt**: Spitzendeckchen (`lace_doily`, nur für sie): Ein gewonnener Ramsch mit höchstens zwei Stichen +1 Stufe.
- **Freischaltung**: Ausgeramscht (`beat_ramscher`): den Boss Der Ramscher schlagen.

### Der Totengräber (`gravedigger`, englisch „The Gravedigger“, Figuren „The Gravedigger“ und „The Widow“)

Der erste Boss nur der langen Nacht (`night_bosses` in `run.ron`). Hebel: **Augen**. Die gedrückten Karten zählen in seiner Kneipe keine Augen (`MultiplySkatPoints(0)`, nicht im Handspiel und im Null). Figuren: Der Totengräber (Neunen Rang 2), Die Witwe (Daus Augen +4).

## 2. Ausgangslage und Schritte

**Vorher** ist der Stand des ersten Commits (0420627): beide ohne eigene Kurve (die Kurve aus `run.ron`, 15 / 30 / 45 / 50 / 85 · 55 / 65 / 75 / 85 / 95 · 115 / 130 / 145 / 160 / 115), der Zocker mit vier durchsichtigen Daus und der Hand-Stufe bei der Ansage (sie verdoppelte auch den Verlust), der Bot hob die Skat-Aufnahme für einen Null Hand mit bis zu zwei offenen Karten auf (`NULL_HAND_MAX_EXPOSED`).

Zwischen dem ersten Commit und dieser Runde hat ein erster Durchgang (durch einen Neustart des Containers unterbrochen, ohne Protokoll) beim Zocker drei Dinge geändert und eine erste Kurve gesetzt: nur noch die Schellen-Daus durchsichtig, die Hand-Stufe nur bei Gewinn, der Bot hebt die Aufnahme nur für einen Null Hand mit höchstens einer offenen Karte auf (`HAND_KEEP_NULL_EXPOSED = 1`; Null Hand gewinnt mit einer offenen Karte 76 %, mit zwei 54 %). Die Wirkung der drei Änderungen einzeln ist nicht gemessen, nur zusammen mit der Kurve.

| Schritt | Charakter | Kurve (Stadtteil 1 · 2 · 3, je vier Kneipen und Boss) | Runs | Stadtteil 2 | Stadtteil 3 | gewonnen | Ende 1-1/1-2 |
|---|---|---|---|---|---|---|---|
| vorher | Zocker | `run.ron` | 900 | 25,0 % | 6,8 % | 1,7 % | 30,9 % |
| v1 | | 5 / 15 / 40 / 45 / 80 · 45 / 55 / 60 / 70 / 85 · 95 / 105 / 115 / 125 / 100 | 900 | 38,3 % | 14,1 % | 3,6 % | 17,6 % |
| vorher | Ramschtante | `run.ron` | 900 | 71,3 % | 48,3 % | 22,3 % | 6,6 % |
| v1 | | 35 / 60 / 100 / 115 / 200 · 120 / 140 / 160 / 180 / 180 · 200 / 220 / 240 / 260 / 190 | 900 | 47,6 % | 18,1 % | 3,9 % | 10,4 % |
| v2 | | **40 / 70 / 115 / 130 / 240** · … · **190 / 210 / 230 / 250 / 180** | 900 | 38,4 % | 15,3 % | 3,7 % | 11,5 % |
| v2 | | (dieselbe) | 2100 | 39,9 % | 14,1 % | 3,0 % | 11,0 % |
| v3 | | **40 / 70 / 115 / 130 / 250** · … · **175 / 195 / 210 / 230 / 165** | 2100 | 38,5 % | 13,6 % | 3,1 % | 11,0 % |
| v4 | | **40 / 75 / 125 / 145 / 225** · … · **165 / 180 / 195 / 210 / 160** | 2100 | 40,5 % | 14,4 % | 4,4 % | 11,4 % |
| v5 | | 40 / 75 / 125 / 145 / **235** · … · … | 2100 | 39,2 % | 13,9 % | 4,2 % | 11,4 % |

Lesart (gleiche Seeds, also gepaart):

- **Ramschtante, Boss 1 schwerer** (v2 → v3, 240 → 250, Stadtteil 3 − 7 %): Stadtteil 2 − 1,4, gewonnen kaum (+ 0,1). Ihre Runs endeten zu 43 % beim Boss (Boss 1 gewann sie nur zu 53 %), die normalen Kneipen in Stadtteil 1 zu 86 bis 97 %.
- **Last vom Boss in die Kneipen** (v4: 1-2 bis 1-4 + 5 bis + 15, Boss 250 → 225; Stadtteil 3 − 6 %): Stadtteil 2 + 2,0, gewonnen + 1,3. Stadtteil 3 hält jetzt 30 % statt 23 %.
- **Boss 1 + 10** (v5): Stadtteil 2 − 1,3, Stadtteil 3 − 0,5, gewonnen − 0,2.

## 3. Endprüfung (7 × 300 Runs, Seeds 42 bis 48)

| Charakter | vorher (900) | nachher (2100) | Ende 1-1/1-2 vorher / nachher |
|---|---|---|---|
| Zocker | 25,0 / 6,8 / 1,7 % | **38,8 / 15,2 / 4,0 %** | 30,9 / **18,1 %** |
| Ramschtante | 71,3 / 48,3 / 22,3 % | **39,2 / 13,9 / 4,2 %** | 6,6 / **11,4 %** |

(Stadtteil 2 / Stadtteil 3 / gewonnen.) Beide liegen in allen vier Größen im Zielbild.

Einzelheiten (Seed 42, 600 Runs, Endstand):

| Charakter | verlorene Runs normal / Spelunke / Boss | Boss 1 / 2 / 3 gewonnen | Spiele gewonnen | Punkte je Spiel | Anteil Ramsch | Anteil Null |
|---|---|---|---|---|---|---|
| Zocker | 58,7 / 7,4 / 33,9 % | 64 / 56 / 53 % | 65,2 % | 44,9 | 5,5 % | 19,2 % |
| Ramschtante | 52,6 / 6,8 / 40,6 % | 55 / 68 / 57 % | 79,2 % | 92,4 | 62,2 % | 14,5 % |

- **Zocker**: Seine Kurve liegt überall unter der von `run.ron`, weil er in den meisten Spielen ohne Skat spielt: 45 Punkte je Spiel; 74 bis 87 % seiner Farbspiele und 64 % seiner Grands sind Hand, ein Null nur zu 12 %. Der Ramsch (5,5 %) ist der Ramsch, wenn alle passen.
- **Ramschtante**: Sie spielt in 62 % ihrer Spiele Ramsch und gewinnt ihn zu 86 % (im Mittel 148 Punkte je gewonnenem Ramsch). Ihre Kurve liegt deshalb weit über der von `run.ron`, Boss 1 am höchsten (235), weil sie in Stadtteil 1 mit wenig Relikten schon hohe Werte schreibt. Der Anteil der Runs, die beim Boss enden, ist mit 41 % höher als bei den anderen (30 bis 33 %): Die normalen Kneipen schafft sie leicht, die Boss-Kneipe mit drei vollen Spielen seltener; am schwersten sind für sie in Stadtteil 1 der Ramscher und der Schwarze Peter (je 47 %).

## 4. Der Totengräber in der Nacht

Je 3 × 700 Runs, Seeds 42 bis 44; mit `--night-boss gravedigger` ziehen alle Boss-Kneipen der Nacht nur ihn. Zum Vergleich der Alleinspieler mit der normalen Ziehung (alle sieben Bosse der Nacht) und nur mit dem Alten.

| Charakter | Faktor | Nächte | Boss-Kneipen gewonnen | Nacht erreicht Stadtteil 5 / 6 / 7 / 8+ | gewonnene Kneipen je Nacht |
|---|---|---|---|---|---|
| Alleinspieler | 1,0 | 80 | 186 / 202 (92 %) | 74 / 56 / 39 / 29 % | 13,6 |
| Alleinspieler | 1,4 | 80 | 182 / 201 (91 %) | 74 / 56 / 39 / 27 % | 13,4 |
| Alleinspieler | **2,5** | 80 | **156 / 189 (83 %)** | 68 / 50 / 32 / 19 % | 12,2 |
| Nullspieler | 1,0 | 79 | 206 / 219 (94 %) | 72 / 58 / 47 / 37 % | 14,8 |
| Nullspieler | 1,4 | 79 | 183 / 207 (88 %) | 68 / 53 / 39 / 32 % | 13,7 |
| Nullspieler | **2,5** | 79 | **143 / 180 (79 %)** | 61 / 43 / 28 / 20 % | 11,5 |
| Zocker | 1,0 | 72 | 132 / 158 (84 %) | 71 / 47 / 29 / 21 % | 11,4 |
| Zocker | 1,4 | 72 | 127 / 156 (81 %) | 71 / 47 / 28 / 17 % | 11,1 |
| Zocker | **2,5** | 72 | **117 / 151 (77 %)** | 64 / 42 / 26 / 17 % | 10,5 |
| Ramschtante | 1,0 | 82 | 143 / 163 (88 %) | 62 / 40 / 32 / 23 % | 10,8 |
| Ramschtante | 1,4 | 82 | 139 / 161 (86 %) | 62 / 40 / 30 / 22 % | 10,6 |
| Ramschtante | **2,5** | 82 | **102 / 142 (72 %)** | 54 / 33 / 16 / 12 % | 8,8 |
| Alleinspieler, nur der Alte (× 1,2) | – | 80 | 109 / 155 (70 %) | 55 / 35 / 19 / 10 % | 9,7 |
| Alleinspieler, normale Ziehung (Totengräber × 1,0) | – | 80 | – | 61 / 40 / 28 / 16 % | 10,6 |

Boss-Kneipen der Nacht in der normalen Ziehung (Alleinspieler, 80 Nächte): Schwarzer Peter 85 %, Zwillinge 87 %, Schieber 83 %, Oma 81 %, Alter 58 %, Ramscher 33 %, Totengräber (× 1,0) 19 / 21.

**Lesart**:

- Mit 1,0 war der Totengräber der leichteste Boss der Nacht (92 % beim Alleinspieler, 94 % beim Nullspieler, der ohnehin kaum drückt, was zählt). Seine Regel nimmt dem Alleinspieler die Augen des Skats; das wiegt gegen den Build der Nacht wenig (acht Relikte, Spielwert eines gewonnenen Spiels im Median 120, Zielwert einer Kneipe in Stadtteil 4 im Median 82).
- Der Faktor bewegt in der Nacht wenig: 1,4 kostet 1 bis 6 Punkte, erst 2,5 bringt ihn auf 72 bis 83 % (Alleinspieler 83, Nullspieler 79, Zocker 77, Ramschtante 72 %), in die Nähe der mittleren Bosse (Oma, Schieber, Schwarzer Peter 81 bis 85 % beim Alleinspieler) und über den Alten (58 bis 70 %). Die Ramschtante trifft er am stärksten (72 %); warum, ist nicht aufgeschlüsselt (Kandidat für `character_factors`, falls es sich bestätigt). **Gesetzt: 2,5.** Zwischen 2,5 und den anderen Faktoren liegt kein Widerspruch: Der Totengräber kommt nie am Tag, der Faktor gilt nur auf den Zielwerten der Nacht.
- Ein schwererer Totengräber als der Alte wäre gegen die Absicht: Er soll Abwechslung in die Ziehung bringen, kein neues Ende.

## 5. Geänderte Zahlen

| Was | vorher | nachher |
|---|---|---|
| Zocker Kurve | `run.ron` | 5 / 15 / 40 / 45 / Boss 80 · 45 / 55 / 60 / 70 / Boss 85 · 95 / 105 / 115 / 125 / Boss 100 |
| Zocker Durchsichtig | alle vier Daus | nur die Schellen-Daus |
| Zocker Hand-Stufe | bei der Ansage (auch verloren) | nur gewonnen (`OnScore` mit `Won`) |
| Bot: Skat aufheben für Null Hand | bis 2 offene Karten | bis 1 (`HAND_KEEP_NULL_EXPOSED`) |
| Ramschtante Kurve | `run.ron` | 40 / 75 / 125 / 145 / Boss 235 · 120 / 140 / 160 / 180 / Boss 180 · 165 / 180 / 195 / 210 / Boss 160 |
| Totengräber `target_factor` | 1,0 | 2,5 |

Unverändert: alle anderen Charaktere, Bosse des Tages, Relikte, Bot-Konstanten `RAMSCH_MARGIN` (50 Punkte) und `HAND_KEEP_MIN_PERCENT` (75 %), beide aus dem ersten Durchgang, einzeln nicht gemessen.

## 6. Offen und Nebenbefunde

1. **Ramschtante beim Boss**: 41 % ihrer verlorenen Runs enden beim Boss (andere 30 bis 33 %). Ein eigener Faktor für den Ramscher (`character_factors`, ADR 0012) oder ein leichterer Boss 1 mit schwereren Kneipen wäre der nächste Hebel.
2. **Bot-Konstanten des Zockers und der Ramschtante** einzeln messen (`HAND_KEEP_MIN_PERCENT`, `HAND_KEEP_NULL_EXPOSED`, `RAMSCH_MARGIN`).
3. **Nacht** von Drückeberger, Schneiderin und Bruderherz gegen den Totengräber ungemessen; die lange Nacht der beiden neuen Charaktere nutzt das Wachstum aus `run.ron` (kein eigenes `night_growth`).
4. **Freischaltungen** (`./target/release/sim --seed 42 --runs 150 --unlocks --bot pimc:12:3`, 20 Folgen): `beat_ramscher` (Ramschtante) in 20 von 20 Folgen nach im Mittel 10,4 Runs (1 bis 33), `big_hand` (Zocker, Knobelbecher) 20 von 20 nach 13,8 Runs (1 bis 35). Beide liegen im Mittelfeld neben `grand_hand` (12,7) und `beat_black_peter` (11,4), leichter als Drückeberger (18,4), Nullspieler (38,7), Bruderherz (58,8) und Schneiderin (66,8). Kein Eingriff.

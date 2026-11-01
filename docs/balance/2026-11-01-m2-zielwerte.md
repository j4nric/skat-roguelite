# M2: Zielwerte, Spiellimit, Pässe und Wirtschaft

- **Datum**: 01.11.2026
- **Branch**: `feat/m2-balance` (Stand `main` mit M2-Kern, `bf77a87`)
- **Agent**: `balance`
- **Seeds**: 42, 43, 44. Jede Zahl ist der Mittelwert über die drei Seeds, wenn nicht anders angegeben.
- **Maßstab**: `RunBot` (`core/src/run/bot.rs`) ohne Änderungen, Standard-Charakter, keine Upgrades zu Beginn.

## Befehle

Offizielle Messung (vorher und nachher):

```sh
cargo run --release -p skat_sim -- --seed 42 --runs 1000
cargo run --release -p skat_sim -- --seed 43 --runs 1000
cargo run --release -p skat_sim -- --seed 44 --runs 1000
```

Für alles, was der Simulator noch nicht kann (Bot ohne Einkauf, einzelne Hausregel ab Spielbeginn, Pool ohne eine Hausregel, Erfolgskurven einer Serie, Bosse nach Abschnitt), lief ein Messprogramm außerhalb des Repos, das `skat_core` als Bibliothek nutzt. Es ändert keine Regel- und keine Bot-Logik:

- **Bot ohne Einkauf** (`nobuy`): `RunBot` für alle Spielentscheidungen, am Tresen immer `LeaveShop`.
- **Hausregel ab Spielbeginn** (`inject`): Der frisch gestartete Run wird serialisiert, die Hausregel und ein „Neues Blatt“ werden eingetragen, dann wird sofort neu gegeben, damit auch passive Effekte im ersten Spiel gelten. Die Vergleichsgruppe ohne Hausregel durchläuft dasselbe Protokoll.
- **Pool ohne X**: Hausregel X wird vor dem Run aus dem Content entfernt, der Bot kauft normal.
- **Erfolgskurve**: Eine einzelne Serie mit unerreichbarem Zielwert, der Bot spielt sie bis zum Spiellimit. Da der Bot den Zielwert nie berücksichtigt, ist P(Serie gewonnen bei Zielwert T) = P(höchster Zwischenstand ≥ T). So liefert ein Lauf die Kurve für alle Zielwerte.

Diese Modi sollten als Optionen in `skat_sim` kommen (Auftrag an `core-dev`, siehe unten), damit die Messungen im Repo reproduzierbar sind. Bis dahin sind sie hier so beschrieben, dass sie nachgebaut werden können.

Stichprobengröße: 3 × 1000 Runs für die Haupttabellen, 3 × 3000 Runs bzw. 3 × 3000 Serien für Hausregeln und Kurven. Standardfehler bei 3000 Runs: etwa 0,8 Prozentpunkte für „Abschnitt 2 erreicht“, bei 9000 Runs etwa 0,5.

## Kernbefund: Der Bot hat eine harte Decke

Der Auftrag nannte als Ziel für den Bot: Serie 1-1 in 85 bis 90 % der Runs, 50 bis 65 % erreichen Abschnitt 2. **Das ist mit dem aktuellen `RunBot` mit keinem Zielwert erreichbar.** Selbst wenn der Zielwert 1 ist (jeder Pluspunkt gewinnt), gewinnt der Bot eine normale Serie nur so oft:

| Spiellimit | Pässe | P(Serie ≥ 1) | P(≥ 18) | P(≥ 30) | P(≥ 60) |
|---|---|---|---|---|---|
| 3 | 1 | 61,2 | – | 48,6 | 27,7 |
| 4 | 2 (vorher) | 67,1 | – | 54,6 | 34,5 |
| 4 | 3 | 69,1 | – | 54,9 | 33,3 |
| 5 | 2 | 71,1 | – | 57,5 | 38,6 |
| **5** | **3 (neu)** | **72,2** | **70,3** | **58,9** | **39,7** |
| 6 | 3 | 74,3 | – | 62,0 | 42,5 |
| 8 | 4 | 77,2 | – | 66,4 | 48,9 |
| 10 | 10 | 79,6 | – | 69,6 | 53,8 |

Mit allen 15 Zielwerten auf 1 (Spiellimit 5, 3 Pässe, Bot kauft) erreichen 30,5 % Abschnitt 2, 11,1 % Abschnitt 3 und 3,8 % Abschnitt 4. Das ist die Decke für jede Zielwertkurve.

Ursache ist die Spielstärke, nicht der Zielwert. Ein vom Bot gespieltes Spiel bringt im Mittel negative Punkte, weil verlorene Spiele doppelt zählen. Erwartungswert nach Blatt bei der Ansage (Bot spielt jedes Blatt, 3 × 100 Runs à 200 Spiele):

| Blatt (10 Karten vor dem Skat) | Spiele | Gewonnen % | Punkte je Spiel |
|---|---|---|---|
| Farbspiel, 4 Trümpfe | 17 420 | 28,8 | −47,7 |
| Farbspiel, 5 Trümpfe, 0 oder 1 Ass | 15 238 | 43,8 | −22,2 |
| Farbspiel, 5 Trümpfe, 2 Asse | 3 906 | 63,4 | −4,6 |
| Farbspiel, 5 Trümpfe, 3+ Asse | 943 | 83,9 | +18,7 |
| Farbspiel, 6 Trümpfe | 7 252 | 76,9 | +12,6 |
| Farbspiel, 7 Trümpfe | 1 256 | 92,4 | +32,7 |
| Grand, 2 Buben, 0 oder 1 Ass | 761 | 15,0 | −100,6 |
| Grand, 2 Buben, 2+ Asse | 5 344 | 74,2 | +21,6 |
| Grand, 3 Buben | 1 062 | 91,6 | +68,2 |

`RunBot` spielt jedes Farbspiel ab 5 Trümpfen. Genau diese Blätter haben einen negativen Erwartungswert. Im Run gewinnt der Bot 58 bis 59 % seiner Spiele und macht im Mittel −9 Punkte pro Spiel. Eine Serie gewinnt er praktisch nur, wenn das erste gespielte Spiel gewonnen wird.

**Folge für die Kalibrierung**: Die Ziele aus dem Auftrag wurden auf die Decke des Bots umgerechnet:

| Kennzahl | Ziel laut Auftrag | Decke (alle Zielwerte 1) | Bot-Ziel für M2 |
|---|---|---|---|
| Serie 1-1 gewonnen | 85–90 % | 72,1 % | ≥ 95 % der Decke, also ≥ 68 % |
| Abschnitt 2 erreicht | 50–65 % | 30,5 % | 80–90 % der Decke, also 24–28 % |
| Abschnitt 3 erreicht | 15–25 % | 11,1 % | 5–8 % |
| Abschnitt 4 und mehr | selten | 3,8 % | ≤ 1,5 % |
| Boss in Abschnitt 1 | ≥ 15 % | – | ≥ 15 % je Boss |
| Bot ohne Relikte | klar schlechter | – | Abschnitt 3 höchstens halb so oft |

## Geänderte Werte

`core/content/run.ron`:

| Wert | Vorher | Nachher |
|---|---|---|
| Zielwerte Abschnitt 1 | 60 / 100 / 150 | **18 / 20 / 24** |
| Zielwerte Abschnitt 2 | 200 / 300 / 450 | **27 / 33 / 36** |
| Zielwerte Abschnitt 3 | 600 / 900 / 1400 | **44 / 54 / 63** |
| Zielwerte Abschnitt 4 | 2000 / 3000 / 5000 | **77 / 90 / 110** |
| Zielwerte Abschnitt 5 | 8000 / 12000 / 20000 | **132 / 154 / 180** |
| Spiellimit | 4 | **5** |
| Pässe | 2 | **3** |
| Serie-Grundbetrag | 4 | **3** |
| Bonus pro übrigem Spiel | 1 | 1 |
| Zinsen | 1 pro 5, max. 5 | 1 pro 5, max. 5 |
| Neuauslage | 2 | 2 |
| Startkapital | 4 (Platzhalter) | 4 (gemessen, bleibt) |

Die Zielwerte sind Reizwerte (18, 20, 24, 27, 33, 36, 44, 54, 63, 77, 90, 110, 132, 154, 180). 18 ist der kleinste mögliche Wert eines gewonnenen Spiels, Serie 1-1 heißt also: ein Spiel netto gewinnen. Die Kurve wächst um etwa den Faktor 1,6 bis 1,7 pro Abschnitt.

`core/content/house_rules.ron`, nur Preise:

| Hausregel | Vorher | Nachher | Grund |
|---|---|---|---|
| Bockrunde | 6 | **8** | stärkste Hausregel (siehe unten) |
| Alte Dame | 5 | **7** | zweitstärkste Hausregel |
| Jungfrau | 6 | **5** | Einfluss unter bzw. knapp an 5 % |
| Durchmarsch | 7 | **5** | Einfluss unter 5 % außer im Spätspiel |
| Spitze | 6 | **5** | kein messbarer Einfluss |

Alle anderen Preise bleiben (Kontra 5, Schenken 7, Ouvert-Bonus 5, Schieben 5, Tournee 8). Da der Bot immer die teuerste bezahlbare Hausregel kauft, bringt die Preisordnung „stark = teuer“ ihn nebenbei zu besseren Käufen.

Tests, die die alten Platzhalter festschrieben, lesen die Werte jetzt aus dem Content (`core/src/run/tests.rs`). `core/src/run/content_tests.rs` prüft die neuen Zielwerte von Abschnitt 1 und 5.

## Ergebnis vorher und nachher

`skat_sim --runs 1000`, Seeds 42, 43, 44, Mittelwert:

| Kennzahl | Vorher | Nachher | Seeds nachher |
|---|---|---|---|
| Abschnitt 2 erreicht | 0,6 % | **26,2 %** | 26,1 / 27,6 / 24,8 |
| Abschnitt 3 erreicht | 0,0 % | **6,7 %** | 5,3 / 7,4 / 7,5 |
| Abschnitt 4 erreicht | 0,0 % | **1,2 %** | 1,3 / 1,2 / 1,1 |
| Abschnitt 5 erreicht | 0,0 % | 0,07 % | 0,1 / 0,1 / 0,0 |
| Serie 1-1 | 34,4 % | **69,7 %** | 70,2 / 70,8 / 68,1 |
| Serie 1-2 | 18,3 % | 70,7 % | |
| Boss-Serie 1 | 10,1 % | 53,1 % | |
| Serie 2-1 / 2-2 / Boss | 4,8 / – / – | 71,7 / 69,7 / 51,4 | |
| Serie 3-1 / 3-2 / Boss | – | 67,9 / 61,9 / 44,1 (n ≈ 50) | |
| Gewonnene Spiele | 58,6 % | 58,7 % | |
| Groschen verdient je Run | 2,2 | 11,8 | |
| Hausregeln beim Start von Abschnitt 2 / 3 | – | 3,1 / 5,0 | |
| Geben je Serie (Spiele und Pässe) | 3,6 | 3,05 | |

Bosse (Messprogramm, gleiche Seeds, gewonnene Boss-Serien):

| Boss | Abschnitt 1 vorher | Abschnitt 1 nachher | Abschnitt 2+ nachher |
|---|---|---|---|
| Die Oma | 1,6 % | **33,3 %** | 29,8 % |
| Der Schieber | 14,9 % | **55,2 %** | 50,2 % |
| Der Kontra-Hans | 14,8 % | **70,9 %** | 67,3 % |

Bot mit und ohne Relikte (Messprogramm, 3 × 3000 Runs, nachher):

| Kennzahl | Bot kauft | Bot kauft nie | Unterschied |
|---|---|---|---|
| Abschnitt 2 erreicht | 26,9 % | 21,2 % | −21 % |
| Abschnitt 3 erreicht | 7,3 % | 3,0 % | −59 % |
| Abschnitt 4 erreicht | 1,3 % | 0,10 % | −92 % |
| Mittlerer Abschnitt | 1,36 | 1,24 | |
| Serie 2-1 / 2-2 / Boss | 69,8 / 70,9 / 54,5 | 59,8 / 54,4 / 42,9 | |
| Serie 3-1 / 3-2 / Boss | 66,2 / 62,4 / 42,7 | 45,7 / 45,0 / 16,8 | |

Abgleich mit dem Bot-Ziel: Serie 1-1 bei 97 % der Decke, Abschnitt 2 bei 86 % der Decke, Abschnitt 3 bei 6,7 %, Abschnitt 4 bei 1,2 %, jeder Boss in Abschnitt 1 deutlich über 15 %. Ohne Relikte erreicht der Bot Abschnitt 3 weniger als halb so oft. Relikte werden ab Abschnitt 2 spürbar nötig (Serie 2-2: 71 % mit, 54 % ohne).

Zur Laufzeit: Eine Serie braucht im Mittel 3,05-mal Geben, davon sind 40 % Pässe. Ein vollständiger Run mit 15 Serien hat damit etwa 46-mal Geben, davon etwa 28 gespielte Spiele. Bei 90 Sekunden pro Spiel und wenigen Sekunden pro Pass sind das etwa 45 Minuten plus Tresen. Das passt zum Ziel von 45 bis 60 Minuten.

Hypothetisch, nur zur Einordnung des Auftrags an `core-dev`: Mit einer strengeren Pass-Regel (Farbspiel erst ab 6 Trümpfen oder 5 Trümpfen mit 2 Assen, Grand erst mit 2 Buben und 2 Assen) erreicht der Bot mit denselben Zielwerten 28,8 % Abschnitt 2 und 7,6 % Abschnitt 3. Die Decke hebt das kaum, weil der Bot dann öfter ohne Pässe ein schwaches Blatt spielen muss.

## Hausregeln

### Einzeln ab Spielbeginn (Bot kauft nie, 3 × 3000 Runs)

Vergleich gegen denselben Bot ohne Hausregel. Kriterium aus dem Agenten-Auftrag: mindestens 5 % Einfluss auf eine Kernmetrik.

| Hausregel | Abschn. 2 erreicht | rel. | Abschn. 3 erreicht | Serie 1-1 | Gewonnene Spiele | Punkte je Spiel |
|---|---|---|---|---|---|---|
| keine | 20,90 | – | 2,81 | 70,3 | 57,2 | −8,6 |
| Bockrunde | 28,89 | **+38 %** | 5,58 | 77,0 | 56,4 | −14,0 |
| Alte Dame | 27,80 | **+33 %** | 5,06 | 75,7 | 62,5 | −2,6 |
| Schenken | 25,96 | **+24 %** | 4,84 | 73,8 | 59,3 | −2,9 |
| Tournee | 22,21 | +6 % | 3,63 | 70,3 | 57,1 | −10,8 |
| Jungfrau | 21,88 | +5 % | 3,27 | 70,8 | 57,1 | −7,7 |
| Durchmarsch | 21,17 | +1 % | 2,88 | 70,7 | 57,1 | −6,8 |
| Schieben | 20,94 | 0 % | 3,00 | 70,3 | 57,2 | −8,5 |
| Spitze | 20,91 | 0 % | 2,81 | 70,3 | 57,2 | −8,5 |
| Ouvert-Bonus | 20,90 | 0 % | 2,81 | 70,3 | 57,2 | −8,6 |
| Kontra | 20,18 | −3 % | 3,01 | 69,2 | 57,2 | −19,1 |

### Erfolgskurve einer Serie mit der Hausregel (3 × 3000 Serien)

Zeigt den Einfluss bei höheren Zielwerten (Spätspiel). Tournee wirkt erst nach gewonnenen Serien und fehlt hier.

| Hausregel | P(≥ 18) | P(≥ 24) | P(≥ 36) | P(≥ 63) | P(≥ 110) | P(≥ 180) |
|---|---|---|---|---|---|---|
| keine | 70,3 | 64,8 | 56,2 | 39,2 | 17,6 | 3,7 |
| Bockrunde | 77,0 | 72,1 | 64,3 | 46,4 | 22,6 | 5,8 |
| Schenken | 73,8 | 69,5 | 61,2 | 45,5 | 23,6 | 7,2 |
| Kontra | 69,2 | 66,5 | 61,8 | 47,5 | **27,4** | **10,4** |
| Alte Dame | 75,7 | 70,3 | 62,0 | 44,6 | 20,6 | 4,4 |
| Jungfrau | 70,8 | 66,2 | 57,5 | 40,8 | 19,0 | 4,3 |
| Durchmarsch | 70,7 | 65,2 | 56,8 | 40,1 | 19,1 | 6,0 |
| Spitze | 70,3 | 64,9 | 56,2 | 39,2 | 17,7 | 3,8 |
| Schieben | 70,3 | 64,7 | 56,2 | 39,3 | 17,5 | 3,7 |
| Ouvert-Bonus | 70,3 | 64,8 | 56,2 | 39,2 | 17,6 | 3,7 |

### Im Kontext (Bot kauft, Pool ohne die Hausregel, 3 × 3000 Runs)

Voller Pool: 26,92 % Abschnitt 2, 7,26 % Abschnitt 3. Ohne Bockrunde 25,28 / 5,80, ohne Alte Dame 25,72 / 6,43, ohne Schenken 26,63 / 6,83. Ohne jede andere Hausregel steigen die Werte leicht (27,0 bis 27,6 / 7,4 bis 7,8): Wirkungslose Hausregeln im Pool belegen Plätze und Groschen. Die Unterschiede liegen dort bei 1 bis 1,5 Standardfehlern.

### Bewertung

| Hausregel | Urteil | Vorschlag |
|---|---|---|
| Bockrunde | stark, obere Grenze. Allein fast an der Decke (28,9 von 30,5 % Abschnitt 2). | Preis 8 (umgesetzt). Wert beobachten, in M5 mit stärkerem Bot neu messen. |
| Alte Dame | stark. Wirkt für beide Seiten (ADR 0005), hilft aber dem Alleinspieler, weil die Gewinnschwelle bei 61 bleibt, während die Augensumme auf 128 steigt. | Preis 7 (umgesetzt). Schwellenfrage an `skat-rules`, siehe unten. |
| Schenken | gut, +24 %. | bleibt bei 7. |
| Tournee | wirkt wie gedacht als Skalierung: +6 % Abschnitt 2, +29 % Abschnitt 3, Abschnitt 4 fast dreimal so oft. | bleibt bei 8. |
| Kontra | Varianz-Relikt: früh leicht schädlich (−3 %), bei hohen Zielwerten stark (+56 % bei 110, ×2,8 bei 180). Für den Bot im Kontext eher schädlich. | Preis bleibt 5. Als bewusstes Spätspiel-Relikt beschreiben. |
| Jungfrau | knapp an der 5-%-Grenze (+4,7 % Abschnitt 2). | Bereich 61–65 auf **61–70** erweitern: gemessen +8,3 % Abschnitt 2 (61–75: +12 %). Nicht umgesetzt, weil der Content-Test `virgin_counts_61_to_65_as_schneider` den Bereich festschreibt; Auftrag an `content`. |
| Durchmarsch | unter 5 %, nur im Spätspiel spürbar (+9 % bei 110). Schwarz ist zu selten (0,8 % der Spiele). Grundwert ×5 statt ×3 gemessen: nur +1,5 %. | Bedingung lockern, z. B. „Schneider“ statt „alle Stiche“ mit ×2. Auftrag an `content`. |
| Spitze | kein Einfluss (0,0 %). Auch mit 3 Stufen statt 1 nur +0,3 %. Letzter Stich mit der Karo-7 kommt praktisch nicht vor. | Bedingung ändern, z. B. „letzter Stich mit einer Sieben“. Auftrag an `content`. |
| Ouvert-Bonus | kein Einfluss, weil der Bot nie Ouvert ansagt (Messverzerrung). | Erst messbar, wenn der Bot Ouvert erwägt. Auftrag an `core-dev`. |
| Schieben | kein Einfluss. Kein Bot-Fehler, sondern Mechanik: Der Skat wird neu gezogen, bevor ihn jemand gesehen hat, und das eigene Blatt bleibt gleich. Das ist für jeden Spieler erwartungsneutral. | Mechanik ändern: Skat vor der Entscheidung ansehen lassen oder das eigene Blatt neu ziehen. Auftrag an `core-dev` und `content`. |

### Kartenmodifikationen (Stichprobe)

Der Bot kauft keine Verbrauchsgegenstände, Kartenmodifikationen kommen in Bot-Runs also nie vor. Stichprobe mit einer Modifikation auf einer festen Karte ab Spielbeginn, Erfolgskurve einer Serie, 3 × 3000 Serien:

| Modifikation | P(≥ 18) | P(≥ 63) | P(≥ 110) | Gewonnene Spiele |
|---|---|---|---|---|
| keine | 70,3 | 39,2 | 17,6 | 61,0 |
| Fett auf Kreuz-Ass | 72,8 | 41,5 | 18,7 | 63,5 |
| Fett auf Karo-10 | 72,2 | 40,9 | 18,4 | 63,0 |
| Stempel auf Kreuz-Ass | 70,3 | 41,1 | 19,2 | 61,0 |
| Spitze auf Kreuz-Ass | 70,2 | 39,7 | 17,9 | 61,0 |
| Gezinkt auf Karo-7 oder Kreuz-Bube | 70,3 | 39,2 | 17,6 | 61,0 |
| Durchsichtig auf Kreuz-Ass | 70,3 | 39,2 | 17,6 | 61,0 |

Beide Seiten (ADR 0005): „Fett“ zählt für jeden, der die Karte nimmt, hilft aber netto dem Alleinspieler (+2,5 Prozentpunkte gewonnene Spiele), aus demselben Grund wie die Alte Dame. „Gezinkt“ und „Durchsichtig“ wirken über Rang und Information, die die Heuristik nicht nutzt. Eine belastbare Messung braucht einen Bot, der Verbrauchsgegenstände kauft und Modifikationen gezielt setzt.

## Schlussfolgerungen

1. Die alten Zielwerte waren für die Spielstärke des Bots um eine Größenordnung zu hoch und wuchsen exponentiell, obwohl keine M2-Hausregel exponentiell skaliert. Die neue Kurve wächst um den Faktor 1,6 bis 1,7 pro Abschnitt.
2. Mit den neuen Werten liegt der Bot bei 86 bis 97 % seiner Decke in Abschnitt 1, Bosse sind schlagbar, Relikte werden ab Abschnitt 2 nötig.
3. Die Ziele aus dem Auftrag (85–90 % Serie 1-1, 50–65 % Abschnitt 2) sind mit dem aktuellen Bot nicht erreichbar. Das ist ein Befund über den Bot, nicht über die Zielwerte.
4. Für Menschen wird Abschnitt 1 mit diesen Werten leicht sein. Das ist gewollt (Bot deutlich schwächer als ein Mensch), muss aber in M5 mit stärkerer KI und Testspielen neu kalibriert werden.
5. Vier der zehn Hausregeln haben keinen messbaren Einfluss (Spitze, Ouvert-Bonus, Schieben, Durchmarsch), eine liegt an der Grenze (Jungfrau). Drei sind stark (Bockrunde, Alte Dame, Schenken).

## Aufträge an `core-dev`

1. **Pass-Regel von `RunBot`**: Farbspiele mit 5 Trümpfen und höchstens einem Ass haben einen negativen Erwartungswert (Tabelle oben). Vorschlag: Farbspiel ab 6 Trümpfen oder 5 Trümpfen mit 2 Assen, Grand ab 2 Buben mit 2 Assen oder ab 3 Buben. Dazu Pässe nicht verbrauchen, wenn das Restlimit sie nicht mehr erlaubt. Danach die Zielwerte neu messen.
2. **Spielstärke**: Die eigentliche Decke ist die Abspielstärke (Farbspiel mit 5 Trümpfen nur 49,5 % gewonnen). Das gehört zu PIMC in M5. Bis dahin bleiben alle Zielwerte Bot-Werte.
3. **Tresen-Policy von `RunBot`**: kauft nur die teuerste Hausregel, nie Verbrauchsgegenstände, nie Neuauslage, verkauft nie. Kartenmodifikationen, Verbrauchsgegenstände, Zinsen und Neuauslage sind damit im Run nicht messbar. Vorschlag: eine zweite Policy, die nach einer Wertetabelle kauft und „Ein Bier“ und „Neues Blatt“ nutzt.
4. **Ansagen**: Der Bot sagt nie Ouvert oder Schneider an. Der Ouvert-Bonus ist deshalb nicht messbar.
5. **Schieben**: Die Mechanik ist erwartungsneutral (siehe oben). Regeländerung mit `content` abstimmen.
6. **Boss-Zielwert-Faktor**: Bosse unterscheiden sich stark, teilen aber einen Zielwert. Der Kontra-Hans ist für den Bot so leicht wie eine normale Serie (70,9 % gegen 70,7 % in Serie 1-2). Die Oma ist später kaum schlagbar: Boss-Serie 3 (63) mit drei starken Hausregeln 16 %, Abschnitt 4 (110) 2 %, Abschnitt 5 (180) 0 %. Vorschlag: ein Feld `target_factor` in `bosses.ron` (Startwerte zum Messen: Oma 0,6, Schieber 1,0, Kontra-Hans 1,4).
7. **Simulator-Optionen**: `--no-shop`, `--relic <id>` (ab Spielbeginn, mit passiven Effekten im ersten Spiel), `--without <id>`, `--boss <id>` und eine Kurven-Ausgabe P(Serie ≥ T). Dann sind alle Messungen dieser Datei mit `skat_sim` reproduzierbar.

## Frage an `skat-rules`

Die Gewinnschwelle (61) und die Schneider-Schwellen (90 und 30) sind absolute Zahlen. Jede Augen-Änderung, die für beide Seiten gilt (Alte Dame, Fett), hilft damit netto dem Alleinspieler. Soll die Schwelle mit der Augensumme mitwachsen (mehr als die Hälfte aller Augen), oder ist die Asymmetrie gewollt?

## Offene Punkte für M5

- Zielwerte mit PIMC-KI und Testspielen neu kalibrieren. Abschnitt 4 und 5 sind mit dem Bot kaum gemessen (weniger als 40 Versuche je Serie in 3000 Runs) und gelten als Vorschlag.
- Die Wirtschaft füllt alle fünf Plätze bis Abschnitt 3 (3,1 Hausregeln beim Start von Abschnitt 2, 5,0 bei Abschnitt 3). Ein Mensch gewinnt Serien schneller und verdient mehr. Es braucht Ausgaben für das Spätspiel: Figuren (6 bis 10), Kartenmodifikationen am Tresen, Neuauslagen.
- Zinsen und Neuauslage sind ungemessen, weil der Bot nie spart und nie neu auslegt.
- Eine exponentielle Zielwertkurve wie im ersten Entwurf braucht Hausregeln, die multiplikativ skalieren (M4).
- Nach den Content-Änderungen an Jungfrau, Durchmarsch, Spitze, Schieben und Ouvert-Bonus alle Hausregeln neu messen.

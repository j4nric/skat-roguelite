# Kleine Runde Alleinspieler: Kneipentour, Bosse v2, Gegenstände, Zinken

- **Datum**: 03.11.2026
- **Branch**: `feat/balance-2026-11-03` auf `main` 908b04c (nach #65 Gegenstands-Review, #68 neue Charaktere, #69 Zinken, #70 Bosse v2, #71 Spezialkarten)
- **Agent**: `balance`
- **Art**: reine Messung, **nichts geändert**. Erste Messung seit der Kneipentour (ADR 0016) und den Bossen v2 (ADR 0017). Alle Zahlen in `run.ron`, `bosses.ron`, `house_rules.ron` und `consumables.ron` sind der Stand vom 02./03.11.2026 (nach Augenmaß gesetzt).
- **Maßstab**: PIMC-Spieler (`--bot pimc:12:3`) als guter Spieler, Ansage, Tresen, Routenwahl und Begegnungen vom `RunBot`, Gegner je Stadtteil aus `run.ron` (Heuristik, PIMC 8:3, PIMC 12:3), alles freigeschaltet, Einsatz 0, nur der **Alleinspieler**.
- **Seeds**: 42, 43, 44, je 500 Runs (1500 Runs). Standardfehler bei 1500 Runs: „Stadtteil 2 erreicht“ rund 1,3 Punkte, „Run gewonnen“ rund 1,0 Punkte; je Boss (450 bis 520 Boss-Kneipen) rund 2,2 Punkte.
- **Laufzeit**: rund 1,2 bis 1,7 s je Run und Prozess (drei Prozesse parallel), eine Runde mit 3 × 500 Runs etwa 13 Minuten.

## Befehle

```sh
cargo build --release -p skat_sim
# ganze Abende (Kopfzahlen, Kneipen, Bosse, Relikte)
./target/release/sim --seed 42 --runs 500 --bot pimc:12:3      # dasselbe mit 43, 44
# Erfolgskurve einer Kneipe ohne Relikte (Stadtteil 1, Heuristik-Gegner; mit --defenders pimc:12:3 wie Stadtteil 3)
./target/release/sim --seed 42 --runs 2000 --bot pimc:12:3 --curve
./target/release/sim --seed 42 --runs 800 --bot pimc:12:3 --curve --defenders pimc:12:3
# Boss-Kneipe ohne Relikte, alle drei Spiele, Stand nach dem dritten (Stadtteil 1)
./target/release/sim --seed 42 --runs 800 --bot pimc:12:3 --curve --boss <the_pusher|grandma|twins|ramscher|black_peter|the_old_man>
# ein Relikt ab Spielbeginn, ohne Tresen (je Seed 42 und 43, 250 Runs)
./target/release/sim --seed 42 --runs 250 --bot pimc:12:3 --no-shop [--relic <id>]
# Freischaltungen (Heuristik-Bot, 20 Folgen mit frischem Meta-Zustand)
./target/release/sim --seed 42 --runs 40 --unlocks
```

**Werkzeug für die Details.** Der Simulator meldet weder Leben, Spiele je Kneipe, Groschen je Kneipe noch die Aufteilung der Ausgaben, und seine Kneipentabelle zählt Spelunken unter der Kneipe, die sie ersetzen. Für diese Runde lief deshalb ein Wegwerf-Auswerter außerhalb des Repos (Scratchpad, gleiche Schleife wie `sim/src/runs.rs`: `RunBot`, PIMC-Kartenspiel, Untot-Regel), der je Run die Ereignisse mitschreibt. Er ist identisch mit `sim`: Seed 42 mit 500 Runs ergibt in beiden 59,2 / 34,8 / 17,4 % (Stadtteil 2, 3, gewonnen) und dieselben Kneipen- und Boss-Zahlen. Vorschlag an `core-dev`: diese Kennzahlen in `sim` aufnehmen (siehe Abschnitt 7).

## 1. Ausgang der Runs

| Seed | Stadtteil 2 erreicht | Stadtteil 3 erreicht | Run gewonnen |
|---|---|---|---|
| 42 | 59,2 % | 34,8 % | 17,4 % |
| 43 | 58,0 % | 35,6 % | 17,4 % |
| 44 | 55,0 % | 33,0 % | 16,2 % |
| **alle (1500)** | **57,4 %** | **34,5 %** | **17,0 %** |

Gehalten je Stadtteil (Anteil der Runs, die ihn erreichen und den nächsten erreichen bzw. gewinnen): **57,4 / 60,0 / 49,3 %**.

Wo die Runs enden (Anteil an allen 1500 Runs):

| Stadtteil | in normalen Kneipen und Spelunken (Leben aus) | in der Boss-Kneipe |
|---|---|---|
| 1 | 1,7 % (25) | 40,9 % (614) |
| 2 | 0,4 % (6) | 22,5 % (338) |
| 3 | 0,1 % (1) | 17,4 % (261) |
| **Summe** | **2,1 %** | **80,9 %** |

**97,4 % aller verlorenen Runs enden in einer Boss-Kneipe.** Ein ganzer gewonnener Run spielt im Mittel 29,4 Spiele und mischt 22,8-mal ein (alle Runs: 19,1 Spiele, 14,6 Einmischen); bei rund 90 Sekunden je Spiel liegt das im Zielbild von 45 bis 60 Minuten.

Zum Vergleich ohne Tresen (`--no-shop`, Seeds 42 und 43, je 250): Stadtteil 2 50,4 %, Stadtteil 3 21,6 %, gewonnen 8,2 %. Der Tresen bringt also nur +7 Punkte „Stadtteil 2“ und +9 Punkte „gewonnen“; den Großteil schafft das Kartenspiel selbst.

## 2. Je Kneipe

Ganze Abende, 1500 Runs. „Leben vorher“ ist der Stand beim Betreten, „Relikte“ die Zahl der Relikte beim Betreten (der Zähler als Startrelikt zählt mit), „Groschen“ die Auszahlung einer gewonnenen Kneipe. Zielwert der Spelunke: die ersetzte Kneipe × 1,25 ohne Wegbier (mit Wegbier 3 weniger).

| Kneipe | Zielwert | Versuche | gewonnen | Spiele je Kneipe | Einmischen je Kneipe | Leben vorher | Groschen je gewonnener | Relikte | Null-Anteil |
|---|---|---|---|---|---|---|---|---|---|
| 1-1 | 33 | 1500 | 77,1 % | 1,79 | 1,29 | 3,00 | 7,73 | 1,00 | 36,1 % |
| 1-2 | 36 | 1500 | 78,9 % | 1,73 | 1,29 | 2,83 | 7,96 | 2,16 | 35,8 % |
| 1-3 | 38 | 1398 | 80,6 % | 1,75 | 1,30 | 2,69 | 7,79 | 3,04 | 35,9 % |
| 1-3 Spelunke | 48 | 102 | 63,7 % | 2,09 | 1,52 | 3,10 | 7,82 | 3,74 | 32,9 % |
| 1-4 | 40 | 1306 | 80,1 % | 1,76 | 1,37 | 2,53 | 7,79 | 3,78 | 37,6 % |
| 1-4 Spelunke | 50 | 185 | 72,4 % | 1,94 | 1,37 | 3,14 | 7,84 | 4,27 | 30,7 % |
| 1-Boss | 45 × Faktor | 1475 | 58,4 % | 3,00 | 2,08 | 2,53 | 6,27 | 4,43 | 24,8 % |
| 2-1 | 40 | 650 | 82,8 % | 1,70 | 1,24 | 3,61 | 7,98 | 4,91 | 38,1 % |
| 2-1 Spelunke | 50 | 211 | 68,7 % | 1,97 | 1,41 | 3,70 | 8,00 | 4,92 | 30,4 % |
| 2-2 | 42 | 667 | 80,2 % | 1,73 | 1,39 | 3,48 | 7,90 | 4,99 | 36,9 % |
| 2-2 Spelunke | 53 | 194 | 71,1 % | 1,98 | 1,60 | 3,89 | 7,90 | 5,00 | 30,7 % |
| 2-3 | 44 | 687 | 85,6 % | 1,65 | 1,30 | 3,43 | 7,94 | 5,00 | 38,4 % |
| 2-3 Spelunke | 55 | 174 | 71,3 % | 1,88 | 1,44 | 3,82 | 8,27 | 4,99 | 33,9 % |
| 2-4 | 46 | 683 | 84,8 % | 1,64 | 1,42 | 3,34 | 8,02 | 5,00 | 36,9 % |
| 2-4 Spelunke | 58 | 177 | 76,3 % | 1,90 | 1,61 | 3,95 | 7,85 | 5,00 | 32,3 % |
| 2-Boss | 52 × Faktor | 855 | 60,5 % | 3,00 | 2,37 | 3,45 | 6,53 | 5,00 | 25,6 % |
| 3-1 | 46 | 387 | 86,3 % | 1,63 | 1,33 | 4,30 | 8,09 | 5,00 | 35,5 % |
| 3-1 Spelunke | 58 | 130 | 76,2 % | 1,85 | 1,67 | 4,45 | 8,13 | 5,00 | 27,4 % |
| 3-2 | 48 | 394 | 85,3 % | 1,58 | 1,51 | 4,30 | 8,38 | 5,00 | 36,7 % |
| 3-2 Spelunke | 60 | 123 | 75,6 % | 1,83 | 1,59 | 4,29 | 8,51 | 5,00 | 31,1 % |
| 3-3 | 50 | 403 | 82,9 % | 1,65 | 1,36 | 4,20 | 8,31 | 5,00 | 38,8 % |
| 3-3 Spelunke | 63 | 114 | 78,9 % | 1,88 | 1,34 | 4,33 | 8,03 | 5,00 | 31,3 % |
| 3-4 | 52 | 395 | 86,3 % | 1,70 | 1,51 | 4,11 | 8,01 | 5,00 | 37,5 % |
| 3-4 Spelunke | 65 | 122 | 75,4 % | 1,89 | 1,79 | 4,39 | 8,10 | 5,00 | 25,1 % |
| 3-Boss | 60 × Faktor | 516 | 49,4 % | 3,00 | 2,13 | 4,10 | 6,92 | 5,00 | 19,3 % |

Ausgang einer normalen Kneipe (ohne Spelunke):

| Stadtteil | im 1. Spiel geschafft | im 2. | im 3. | verfehlt | verlorene Kneipen je Run im Stadtteil |
|---|---|---|---|---|---|
| 1 | 54,2 % | 15,9 % | 9,0 % | 20,9 % | 0,85 |
| 2 | 57,7 % | 16,6 % | 9,0 % | 16,6 % | 0,77 |
| 3 | 59,0 % | 18,2 % | 8,0 % | 14,8 % | 0,68 |

**Leben im Verlauf**: beim Start 3,00, vor Boss 1 im Mittel 2,53, Start Stadtteil 2 3,63, vor Boss 2 3,45, Start Stadtteil 3 4,34, vor dem Alten 4,10. Nur 14,7 % aller Runs stehen je bei einem Leben. Die Leben **steigen** über den Abend: +1 je Boss, dazu Rast und Konterbier, die gelten, sobald ein Leben unter dem Maximum 5 fehlt, also vom ersten Schritt an.

**Spelunke**: in 13,3 % der normalen Kneipen gewählt (der Bot geht nur mit 3 Leben und 3 Relikten hinein), 8 bis 17 Punkte schwerer als die Kneipe, die sie ersetzt. Je Malus: Kein Skat 59,3 % (361), Kein Einmischen 71,8 % (387), Klebrige Karten 77,0 % (379), Schnapsverbot 81,7 % (405).

**Erfolgskurve einer Kneipe ohne Relikte** (`--curve`, P(Kneipe geschafft) bei Zielwert):

| Gegner | 1 | 33 | 40 | 46 | 52 | 60 |
|---|---|---|---|---|---|---|
| Heuristik (Stadtteil 1), 2000 Kneipen | 85,2 % | 77,2 % | 75,2 % | 74,0 % | 69,0 % | 67,0 % |
| PIMC 12:3 (wie Stadtteil 3), 800 Kneipen | 79,1 % | 70,4 % | 68,2 % | 65,6 % | 61,0 % | 59,5 % |

Die Kurve ist flach (rund 0,3 Punkte je Zielwert-Punkt). In Stadtteil 3 gewinnt der Bot mit fünf Relikten seine Kneipen bei 46 bis 52 zu 83 bis 86 %, ohne Relikte gegen dieselben Gegner zu 61 bis 66 %: Die Relikte tragen dort rund 20 Punkte.

## 3. Bosse

Boss-Kneipe gewonnen im ganzen Abend (Relikte wie sie der Bot hat) und in der Kurve ohne Relikte (Stadtteil 1, Heuristik-Gegner, 800 Boss-Kneipen, Seed 42; „bei Zielwert“ ist 45 × Faktor, beim Alten 60):

| Boss | Faktor | im Run, alle | Stadtteil 1 | Stadtteil 2 | je Seed 42 / 43 / 44 | Kurve bei Zielwert | Obergrenze (Zielwert 1) |
|---|---|---|---|---|---|---|---|
| Der Ramscher | 1,0 | **35,5 %** (454) | 35,7 % | 35,0 % | 40,8 / 34,9 / 29,0 | 45,8 % | 56,8 % |
| Der Alte (Endboss, Stadtteil 3) | 1,0 | 49,4 % (516) | – | – | 50,3 / 48,9 / 49,1 | 48,9 % | 63,5 % |
| Der Schieber | 0,8 | 52,1 % (489) | 51,6 % | 53,0 % | 53,0 / 55,9 / 48,4 | 39,8 % | 50,6 % |
| Der Schwarze Peter | 0,5 | 63,9 % (463) | 62,5 % | 66,5 % | 61,4 / 69,4 / 61,4 | etwa 54 % | 64,8 % |
| Die Zwillinge | 0,85 | 71,8 % (468) | 72,8 % | 70,2 % | 72,7 / 71,7 / 70,9 | 69,5 % | 76,4 % |
| Die Oma | 0,7 | 72,4 % (456) | 70,1 % | 76,4 % | 76,2 / 65,6 / 75,9 | etwa 60 % | 68,4 % |

Normale Kneipen zum Vergleich: 77 bis 86 % im Run, Obergrenze ohne Relikte 85 %.

Boss-Kurven von Zielwert 1 bis 60 (ohne Relikte, Stadtteil 1):

| Boss | 1 | 33 | 40 | 45 | 52 | 60 |
|---|---|---|---|---|---|---|
| Schieber | 50,6 % | 40,2 % | 38,4 % | 36,6 % | 34,8 % | 32,5 % |
| Ramscher | 56,8 % | 49,1 % | 46,8 % | 45,8 % | 44,4 % | 42,8 % |
| Der Alte | 63,5 % | 56,4 % | 54,2 % | 52,5 % | 51,1 % | 48,9 % |
| Schwarzer Peter | 64,8 % | 51,4 % | 48,5 % | 45,5 % | 42,8 % | 40,4 % |
| Oma | 68,4 % | 60,2 % | 57,6 % | 56,1 % | 53,2 % | 50,1 % |
| Zwillinge | 76,4 % | 70,5 % | 68,8 % | 67,5 % | 64,9 % | 63,1 % |

Alle Boss-Kurven sind noch flacher als die normale (0,15 bis 0,25 Punkte je Zielwert-Punkt). **Der Faktor stellt nur fein nach; wie schwer ein Boss ist, entscheidet seine Regel.**

**Der Ramscher im Einzelnen.** Ein gewonnener Ramsch bringt im Mittel +74, ein verlorener −156 (doppelt). Im Run gewinnt der Bot 63,2 % seiner Ramsch-Spiele, ohne Relikte 69,3 %: Was der Spieler sich aufgebaut hat (Aufdrucke, Winkel, Alte Oberin, Kleinvieh), arbeitet im Ramsch gegen ihn. Praktisch muss er alle drei Ramsch-Spiele gewinnen (Verteilung der gewonnenen Spiele im Run: 0 / 1 / 2 / 3 = 5 / 26 / 44 / 25 %). Gegenrechnung aus denselben Spielen (je 600 Boss-Kneipen ohne Relikte, Seed 42, Stadtteil 1 und 3):

| Variante | Stadtteil 1 (Zielwert 45) | Stadtteil 3 (Zielwert 60) |
|---|---|---|
| heute (verloren doppelt) | 44,8 % | 36,7 % |
| verlorener Ramsch zählt einfach | 64,2 % | 55,7 % |
| einfach und Faktor 0,5 | 70,5 % | 66,0 % |

Der Schwarze Peter ist mit „halb statt null“ (02.11.2026) kein Hindernis mehr (64 % im Run; in den Serien mit 3 Spielen vor ADR 0017 44 %). Die Oma ist mit 16 Flüchen für den Bot leicht: Er spielt sie wie eine normale Kneipe mit etwas weniger Augen.

## 4. Wirtschaft

Je Run (1500 Runs):

| Einnahmen | Groschen | | Ausgaben | Groschen | Anteil |
|---|---|---|---|---|---|
| Startkapital | 4,00 | | Relikte | 33,59 | 52,0 % |
| Auszahlungen der Kneipen | 55,75 | | Zinken (Auslage) | 17,60 | 27,2 % |
| sonstige (Relikte, Flüche, Begegnungen, Kasse) | 4,82 | | Schnäpse | 11,53 | 17,9 % |
| Verkäufe (Rest) | rund 3,8 | | Kartenmaler | 1,20 | 1,9 % |
| | | | Neuauslage | 0,54 | 0,8 % |
| | | | Karte reinigen | 0,14 | 0,2 % |
| | | | Zinken neu auslosen | 0,00 | 0,0 % |
| | | | **Summe** | **64,60** | |
| Bestand am Ende | 3,80 | | | | |

- **Groschen je Kneipe**: eine gewonnene normale Kneipe zahlt 7,93 (meist im ersten Spiel geschafft: 3 + 2 × 3 + Zinsen), eine gewonnene **Boss-Kneipe nur 6,45**, obwohl doppelt: Sie spielt immer alle drei Spiele, der Bonus je übrigem Spiel fällt weg. Je Kneipe (auch verlorene) 5,83. Je Stadtteil, der erreicht wird, 28 bis 31 Groschen.
- **Tresen**: 6,8 Besuche je Run.
- **Relikte beim Betreten**: Stadtteil 1 1,00 (der Zähler), vor Boss 1 4,43, Stadtteil 2 **4,91** (93 % der Runs mit allen fünf Plätzen belegt), Stadtteil 3 5,00. Ab Stadtteil 2 fließt das Geld in Zinken und Schnäpse; das Boss-Relikt ist dann immer ein Tausch.
- **Zinken je Run**: Aufdruck 1,82, Winkel 1,68, Siegel 0,83, Pfandmarke 0,76, Eselsohr 0,63, Kerbe 0,52, Untot 0,07, Wandelkarte 0,05; Kartenmaler (Tresen und Werkstatt) 0,48 Karten je Run. Neu auslosen nimmt der Bot nie, reinigen fast nie (er hat selten einen Fluch).
- **Schnäpse** (gekauft / getrunken, 1500 Runs): Kräuter 1387 / 772, Wegbier 1348 / 1202, Kurzer 1219 / 1152, Konterbier 766 / 548, Blue Curaçao 696 / 570, Doppelkorn 695 / 349, Schwenker 0 / 0. Kräuter und Doppelkorn verfallen zur Hälfte ungetrunken.

## 5. Gegenstände

**Gekauft** (1500 Runs, Käufe; dazu „gewonnen“ = Boss-Relikt oder Spelunken-Beute): Ein Auge zudrücken 739, Schieben 586, Kleinvieh 535 (+177), Alte Oberin 534 (+200), Luschenkönig 398, Schenken 395 (+113), Allrounder 324 (+119), Mit Vieren 276, Skatpfleger 269, Jungfrau 263, Der Schneider 261, Feierabend 258, Stichfest 250, Monatskarte 245, Drei drücken 232 (+90), Spitze 220, Nullachtfuffzehn 199, Dicke Sau 184, Der Wirt 181, Kiebitz 142, Stammtischkasse 131, Deckel anschreiben 130, Schutzengel 120 (+43), Spickzettel 89, Glückssträhne 86, Tournee 84, Spelunkenkenner 76, Skatgott 58 (+17), Bierdeckel 47, Stammfarbe 16, Ehrenvorsitz 7.

**Ein Relikt ab Spielbeginn, ohne Tresen** (`--no-shop --relic <id>`, Seeds 42 und 43, je 250 Runs, also 500 je Zeile; gleiche Karten wie ohne Relikt; Standardfehler einer Differenz bei gepaarten Seeds grob 2 Punkte, Unterschiede unter 3 Punkten sind Rauschen):

| Relikt | Preis | Stadtteil 2 | Δ | Stadtteil 3 | Δ | gewonnen | Δ |
|---|---|---|---|---|---|---|---|
| ohne (nur Zähler) | – | 50,4 % | | 21,6 % | | 8,2 % | |
| **Skatgott** (Legendär) | 9 | 58,2 % | +7,8 | 33,6 % | **+12,0** | 20,4 % | **+12,2** |
| Drei drücken (Selten) | 9 | 57,0 % | +6,6 | 27,6 % | +6,0 | 10,2 % | +2,0 |
| Alte Oberin | 9 | 56,2 % | +5,8 | 26,8 % | +5,2 | 12,2 % | +4,0 |
| Kleinvieh | 9 | 56,0 % | +5,6 | 24,4 % | +2,8 | 10,6 % | +2,4 |
| Ein Auge zudrücken | 8 | 54,0 % | +3,6 | 24,6 % | +3,0 | 9,6 % | +1,4 |
| Jungfrau | 7 | 53,8 % | +3,4 | 24,2 % | +2,6 | 9,4 % | +1,2 |
| Schieben | 8 | 53,2 % | +2,8 | 22,6 % | +1,0 | 8,4 % | +0,2 |
| Spickzettel | 5 | 51,6 % | +1,2 | 20,0 % | −1,6 | 7,6 % | −0,6 |
| Luschenkönig | 5 | 51,2 % | +0,8 | 21,8 % | +0,2 | 6,8 % | −1,4 |
| Schenken | 8 | 51,0 % | +0,6 | 21,6 % | ±0,0 | 8,6 % | +0,4 |
| Schutzengel | 8 | 50,8 % | +0,4 | 21,8 % | +0,2 | 8,0 % | −0,2 |
| Feierabend | 5 | 50,4 % | ±0,0 | 19,8 % | −1,8 | 6,6 % | −1,6 |

- **Ausreißer nach oben: Skatgott.** Allein verdoppelt er fast die Siegquote (8 → 20 %). Er ist Legendär und kam im Run nur in 75 von 1500 Runs vor; wer ihn hatte, gewann zu 35 % (alle: 17 %).
- **Ohne Wirkung beim Bot**: Schutzengel (gilt seit ADR 0017 nicht in der Boss-Kneipe, und dort enden 97 % der Runs), Feierabend (+4 Groschen, wenn ohnehin früh geschafft), Schenken (der Bot kommt selten in die Lage), Luschenkönig, Spickzettel (Menschen-Relikt). Schieben kauft der Bot am zweithäufigsten (586), es bewegt aber kaum etwas: Zwei Einmischen je Spiel reichen ihm fast immer.
- **Unterschätzt vom Bot**: Drei drücken (+6,6 / +6,0) und die Alte Oberin sind die stärksten nach dem Skatgott; Ein Auge zudrücken (der häufigste Kauf) liegt eine Stufe darunter.
- **Beide Seiten (ADR 0005)**: Alte Oberin und Kleinvieh gelten „auch bei den Gegnern“ und bleiben trotzdem klar positiv. Im Ramsch dreht sich jede Augen-Hilfe gegen den Spieler (Abschnitt 3).

## 6. Fehler, Sackgassen, Freischaltungen

- **Keine Abstürze, kein Hängen**: 1500 ganze Runs, 26 × 250 Runs ohne Tresen, 7 × 800 Boss-Kurven, 2 × 600 einzelne Boss-Kneipen ohne einen Fehler oder unmöglichen Zustand.
- **Schwenker** wird nie gekauft (Kaufwert 0, so gewollt), **Zinken neu auslosen** nie genutzt (Bot-Regel, nicht Daten).
- **Freischaltungen** (`--unlocks --runs 40`, Heuristik-Bot, 20 Folgen): alle 20 erreichbar. Nach einem Run: Zweiter Abend, Dritter Abend, Früh fertig; im Mittel nach 1 bis 7 Runs die meisten Relikte; Drückeberger (21 Augen gedrückt) 19/20 nach 10,7 Runs; Bruderherz (dreimal dieselbe Trumpffarbe) 9/20 nach 21 Runs; Schneiderin (drei Schneider in einer Kneipe) 4/20 nach 22,8 Runs; **Nullspieler (Run gewinnen) 18/20 nach 8,7 Runs** (früher rund 1700 Runs im ganzen Abend). Zweiter und Dritter Abend (Kneipe 3 und 5 erreichen) sind mit der Kneipentour Geschenke.

## 7. Schmerzpunkte, nach Gewicht

Zielbild: Für die Kneipentour ist keins festgelegt (5.0.4). **Vorschlag** (nicht gemessen, zur Entscheidung): Der PIMC-Spieler erreicht Stadtteil 2 in 35 bis 40 % der Runs und Stadtteil 3 in 12 bis 18 % und gewinnt 3 bis 5 % der Abende; ein Viertel bis ein Drittel der verlorenen Runs endet in normalen Kneipen (Leben aus). Heute: 57 / 35 / 17 %, 2 % in normalen Kneipen.

Alle Zahlen unten sind **Vorschläge ohne Messung**; die große Runde misst sie.

### 1. Normale Kneipen und Leben beißen nicht: Der Abend sind drei Münzwürfe

97 % der verlorenen Runs enden beim Boss. Normale Kneipen gewinnt der Bot zu 77 bis 86 %, mehr als die Hälfte schon im ersten Spiel, und die Leben steigen über den Abend (3,0 → 4,3). Die Kurve der Kneipe ist flach (0,3 Punkte je Zielwert-Punkt), die Relikte tragen hinten rund 20 Punkte; deshalb muss die Kurve **steil** steigen, um zu wirken.

**Vorschlag `run.ron`, `targets`** (normale Kneipen steiler, Boss-Zielwerte bleiben):

| Stadtteil | heute | Vorschlag |
|---|---|---|
| 1 | 33 / 36 / 38 / 40 / Boss 45 | 40 / 46 / 52 / 58 / Boss 45 |
| 2 | 40 / 42 / 44 / 46 / Boss 52 | 56 / 64 / 72 / 80 / Boss 52 |
| 3 | 46 / 48 / 50 / 52 / Boss 60 | 72 / 84 / 96 / 108 / Boss 60 |

Erwartet (aus den Kurven hochgerechnet): Stadtteil 1 rund −4 Punkte je Kneipe, Stadtteil 2 rund −8, Stadtteil 3 rund −12 bis −15; damit etwa 1,2 bis 1,5 verlorene Kneipen je Stadtteil statt 0,7 bis 0,85. Dazu **`lives: (start: 3, max: 4, boss_bonus: 1)`** statt max 5, damit Rast und Konterbier den Vorrat nicht über den Abend aufbauen (heute 4,3 Leben in Stadtteil 3). Die große Runde soll zwei Kurven vergleichen (diese und eine mittlere mit halbem Anstieg).

### 2. Der Ramscher ist eine Wand

35,5 % im Run (Seeds 29 bis 41 %), die anderen Bosse 49 bis 72 %. Ohne Relikte 45,8 %, Obergrenze 56,8 %; der Faktor hilft kaum (von 45 auf 1 nur +11 Punkte). Grund: Ein verlorener Ramsch kostet doppelt (−156 gegen +74), praktisch müssen alle drei gewonnen werden; und der Aufbau des Spielers wirkt im Ramsch gegen ihn (Spiele gewonnen 69 → 63 %).

**Vorschlag (Regel, offene Frage 17 a, an `core-dev` und den Nutzer)**: In der Kneipe des Ramschers zählt ein verlorener Ramsch einfach. Gegengerechnet ohne Relikte 64 % (Stadtteil 1) und 56 % (Stadtteil 3) statt 45 und 37 %; im Run mit dem Abzug durch die Relikte geschätzt 50 bis 55 %. Faktor bleibt 1,0. Rein mit Zahlen (Faktor 1,0 → 0,5) wären es geschätzt nur +3 bis +4 Punkte.

### 3. Die Bosse liegen 23 Punkte auseinander

Oma 72 %, Zwillinge 72 %, Schwarzer Peter 64 % gegen Schieber 52 % und den Alten 49 %. Boss 1 entscheidet 41 % aller Runs; ob dort der Schieber oder die Oma sitzt, ist Glück.

**Vorschlag `bosses.ron`, `target_factor`** (Alleinspieler; mit Steigung 0,15 bis 0,25 je Punkt):

| Boss | heute | Vorschlag | erwartet im Run |
|---|---|---|---|
| Die Oma | 0,7 | 1,0 | rund −4 Punkte (72 → 68 %) |
| Die Zwillinge | 0,85 | 1,2 | rund −4 Punkte (72 → 68 %) |
| Der Schwarze Peter | 0,5 | 0,7 | rund −2 Punkte (64 → 62 %) |
| Der Schieber | 0,8 | 0,5 | rund +3 Punkte (52 → 55 %) |
| Der Ramscher | 1,0 | 1,0 (Regel aus Punkt 2) | |
| Der Alte | 1,0 | 1,0 (Endboss darf der schwerste sein) | |

Mehr als ±5 Punkte geben die Faktoren nicht her. Der Schieber hat ohne Relikte eine Obergrenze von 51 % (der Bot spielt Hand schwach); ob ein Mensch ihn leichter findet, zeigen echte Runs.

### 4. Relikt-Plätze sind in Stadtteil 2 voll, die Boss-Belohnung zahlt weniger als eine Kneipe

93 % der Runs betreten Stadtteil 2 mit fünf Relikten; danach gibt es für Relikt-Groschen nur noch Tausch, Zinken und Schnäpse. Eine gewonnene Boss-Kneipe zahlt 6,45 Groschen, eine im ersten Spiel geschaffte Kneipe 7,93: „doppelt“ ist weniger, weil der Bonus je übrigem Spiel fehlt.

**Vorschlag `run.ron`, `economy.per_game_left` 3 → 2** (rund −8 Groschen je Run, −15 %; die Plätze füllen sich etwa einen Stadtteil später). Für die Boss-Kneipe an `core-dev`: die doppelte Auszahlung so rechnen, als blieben zwei Spiele übrig (wären heute rund 2 × (3 + 6 + Zinsen) = 19 bis 20 Groschen statt 6,45), oder ein fester Boss-Bonus; die Zahl dafür misst `balance` danach.

### 5. Skatgott dominiert, mehrere Relikte wirken nicht

Skatgott +12 Punkte „gewonnen“ allein, alle anderen höchstens +4. Schutzengel, Feierabend, Schenken, Luschenkönig, Spickzettel liegen im Rauschen (5-%-Regel verfehlt, beim Bot).

**Vorschläge `house_rules.ron`**:

- **Skatgott**: Preis 9 → 12 (Legendär, darf teuer sein); Wirkung an `content`: der Multiplikator wächst je Schneider-Sieg höchstens bis +3.
- **Drei drücken** bleibt 9, **Alte Oberin** und **Kleinvieh** bleiben 9 (gemessen am stärksten nach dem Skatgott).
- **Ein Auge zudrücken** 8 → 7 und **Schieben** 8 → 6 (+3,6 und +2,8, der Preis liegt eine Stufe über der Wirkung).
- **Schenken** 8 → 6, **Schutzengel** 8 → 6 (wirkt nur in normalen Kneipen, die heute kaum Leben kosten; mit Punkt 1 neu messen), **Feierabend** 5 → 4.

### 6. Kleinere Punkte

- **Spelunke mit „Kein Skat“** ist mit 59 % rund 13 bis 23 Punkte schwerer als die anderen Mali (72 bis 82 %). Vorschlag: in `tour.ron` die drei anderen Mali auf `weight: 2`, Kein Skat bleibt 1 (halb so oft).
- **Kräuter und Doppelkorn** verfallen zur Hälfte (Bot-Kaufwert, keine Datenfrage); an `core-dev`: Kaufwert 2 → 1.
- **Nullspieler** ist mit der Kneipentour nach 8,7 Runs frei (Heuristik-Bot), früher nach rund 1700; die Kneipen 3 und 5 (Zweiter, Dritter Abend) sind nach dem ersten Run erreicht. Mit Punkt 1 bis 3 neu messen.
- **Simulator** (an `core-dev`): Leben je Kneipe, Spiele je Kneipe, Ende in normaler oder Boss-Kneipe, Ausgaben nach Art und Relikte je Stadtteil ausgeben; Spelunken in der Kneipentabelle getrennt zählen; `--curve --boss` und `--curve` für einen gewählten Stadtteil (heute immer Stadtteil 1, mit `--defenders` nur die Gegner).

## Schlussfolgerung

Die Kneipentour ist für den guten Spieler viel zu leicht (17 % gewonnen statt eines Ziels von wenigen Prozent) und dabei eintönig: Die normalen Kneipen sind Durchgänge, die Leben ein Polster, das wächst, und der Abend entscheidet sich an drei Boss-Kneipen, unter denen der Ramscher eine Wand und Oma und Zwillinge Durchgänge sind. Die Hebel in Reihenfolge: steilere Kurve der normalen Kneipen mit höchstens 4 Leben, die Ramscher-Regel, ausgeglichene Boss-Faktoren, weniger Groschen je übrigem Spiel, dann die Relikt-Preise.

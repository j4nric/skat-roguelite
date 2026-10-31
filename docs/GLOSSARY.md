# Glossar: Skat-Begriffe und ihre Bezeichner im Code

Die Domäne ist deutsch, der Code ist englisch. Diese Tabelle ist verbindlich. Wer einen neuen Begriff braucht, trägt ihn hier ein, bevor er ihn im Code verwendet. Keine Synonyme (kein `unter` neben `jack`, kein `points` neben `card_points`).

## Karten

| Deutsch | Bezeichner | Anmerkung |
|---|---|---|
| Blatt (das ganze Spiel, 32 Karten) | `deck` | Im Roguelike: das geteilte, modifizierbare Blatt |
| Blatt (die Karten eines Spielers) | `hand` | |
| Karte | `Card` | |
| Farbe | `Suit` | |
| Karo, Herz, Pik, Kreuz | `Diamonds`, `Hearts`, `Spades`, `Clubs` | Reihenfolge aufsteigend nach Grundwert |
| Wert / Bild | `Rank` | |
| Sieben, Acht, Neun, Zehn | `Seven`, `Eight`, `Nine`, `Ten` | |
| Bube | `Jack` | nie `Unter`, nie `Wenzel` |
| Dame | `Queen` | nie `Ober` |
| König | `King` | |
| Ass | `Ace` | |
| Augen | `card_points` | Summe im Blatt: 120 |
| Lusche (7, 8, 9) | `blank` | Karte ohne Augen |
| Blanko-Karte (Roguelike) | `Blank` als eigener Kartentyp | Ohne Farbe, ohne Augen, verliert immer. Ersetzt entfernte Karten. |
| Volle (Ass, Zehn) | `high_card` | |
| Trumpf | `trump` | |
| Skat (die zwei Karten) | `skat` | nie `talon`, siehe unten |
| Talon (Roguelike: Karten über 32, nicht im Spiel) | `talon` | nur im Roguelike-Kontext |
| Gezinkte Karte / Kartenmodifikation | `CardMod` | |

## Spielablauf

| Deutsch | Bezeichner | Anmerkung |
|---|---|---|
| Geben | `deal` | |
| Vorhand, Mittelhand, Hinterhand | `Forehand`, `Middlehand`, `Rearhand` | Sitzposition relativ zum Geber |
| Alleinspieler | `declarer` | Immer der menschliche Spieler |
| Gegenspieler | `defender` | Die zwei KI-Spieler |
| Reizen | entfällt | Ersetzt durch Ansage, siehe ADR 0006 |
| Ansage | `Declaration` | Spielart plus Risikostufe |
| Spielart | `GameType` | `Suit(Suit)`, `Grand`, `Null` |
| Farbspiel | `GameType::Suit` | |
| Hand (Skat nicht aufnehmen) | `hand` (bool) | |
| Skat aufnehmen | `pick_up_skat` | |
| Drücken | `discard` | Die zwei Karten, die in den Skat gelegt werden |
| Ausspielen | `lead` | Erste Karte eines Stichs |
| Bedienen | `follow_suit` | |
| Stechen | `ruff` / `trump` (verb) | Mit Trumpf übernehmen |
| Abwerfen | `slough` | Nicht bedienen können und Nicht-Trumpf legen |
| Schmieren | `smear` | Volle in den Stich des Partners geben |
| Stich | `Trick` | |
| Stichgewinner | `trick_winner` | |
| Passen / Einpassen | `pass` | Im Roguelike: ein Spiel auslassen |

## Wertung

| Deutsch | Bezeichner | Anmerkung |
|---|---|---|
| Spielwert | `game_value` | Grundwert × Gewinnstufen |
| Grundwert | `base_value` | Karo 9, Herz 10, Pik 11, Kreuz 12, Grand 24 |
| Gewinnstufen | `levels` | Spitzen + 1 + Hand + Schneider + … |
| Spitzen / Matadore | `matadors` | `with: bool` für „mit“ oder „ohne“ |
| mit / ohne | `with` / `without` | |
| Schneider | `schneider` | Gegner unter 31 bzw. Alleinspieler ab 90 Augen |
| Schwarz | `schwarz` | Alle Stiche |
| angesagt | `_declared` | `schneider_declared`, `schwarz_declared` |
| Ouvert | `ouvert` | Offen spielen |
| Gewonnen | `won` | Ab 61 Augen |
| Verloren | `lost` | Zählt doppelt negativ |
| Null, Null Hand, Null Ouvert, Null Ouvert Hand | `null_value(hand, ouvert)` | 23, 35, 46, 59 |

## Roguelike

| Deutsch | Bezeichner | Anmerkung |
|---|---|---|
| Run / Tournee | `Run` | |
| Abschnitt / Skatabend | `Section` | Ein Ante |
| Serie | `Series` | Ein Blind: Zielwert plus Spiellimit |
| Zielwert | `target` | |
| Spiellimit | `game_limit` | |
| Boss-Serie | `BossSeries` | |
| Hausregel | `HouseRule` | Passives Relikt, Joker-Äquivalent |
| Stammtisch-Figur | `Regular` | Passives Relikt mit Persönlichkeit |
| Verbrauchsgegenstand | `Consumable` | |
| Relikt (Oberbegriff) | `Relic` | HouseRule und Regular |
| Tresen | `Shop` | |
| Groschen | `coins` | Währung |
| Ereignis / Rast | `Event` | |
| Charakter | `Character` | Alleinspieler, Nullspieler |
| Risikostufe | `RiskLevel` | Normal, Hand, SchneiderDeclared, SchwarzDeclared, Ouvert |

## Kartennotation

Zwei Zeichen, Farbe dann Wert. Wird in Tests, Logs und Datendateien verwendet.

```
Farben:  D = Karo   H = Herz   S = Pik   C = Kreuz
Werte:   7 8 9 T J Q K A
Beispiel: CJ = Kreuz-Bube, DT = Karo-Zehn, HA = Herz-Ass
```

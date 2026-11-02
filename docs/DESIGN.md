# Stammtisch – Design-Dokument

Stand: 2026-11-01 · Status: Entwurf v0.3 · Sprache des Projekts: Deutsch (Code und Kommentare: Englisch) · Arbeitstitel bis 31.10.2026: „Skat Roguelike“

Dieses Dokument hält die Grundentscheidungen fest und beschreibt das Spiel so weit, dass ein erster spielbarer Prototyp daraus gebaut werden kann. Alles unter „Offen“ ist noch nicht entschieden. Alles andere gilt, bis es hier geändert wird.

---

## 1. Pitch

**Stammtisch: Skat als Roguelike-Deckbuilder.** Du bist immer der Alleinspieler. Über einen Run hinweg spielst du Skat gegen zwei KI-Gegenspieler, musst in jeder Runde einen Punkte-Zielwert erreichen und veränderst zwischen den Runden das Blatt, mit dem alle drei spielen. Die Spielwertformel des echten Skats (Grundwert × Gewinnstufen) ist die Score-Engine, Hausregeln sind die Relikte, die Kneipe ist der Shop.

Referenzen: Balatro (Score-Formel, Run-Struktur, Shop), Inscryption (Gegner als Figuren, Regelbrüche), Slay the Spire (Charaktere mit eigenem Kartenpool).

Darstellung: deutsches Blatt (Eichel, Grün, Rot, Schellen; Unter, Oberin, Daus), siehe ADR 0010 und 0014. Dieses Dokument verwendet weiter die französischen Begriffe, weil sie im Code und in der Skatordnung stehen. Sprache des Spiels: Deutsch. Zielgerät: Handy hochkant zuerst.

---

## 2. Feste Grundentscheidungen

| # | Entscheidung | Begründung |
|---|---|---|
| 1 | **Der Spieler ist immer Alleinspieler.** Verteidigen gibt es nicht. | Hält den Kern einfach, jede Partie hat denselben Spannungsbogen. |
| 2 | **Null ist für jeden ansagbar**, der Nullspieler ist ein freischaltbarer Spezialist (ADR 0012, ersetzt 0003). | „Farbe, Grand oder Null?“ ist die spannendste Entscheidung im Skat. Der Spezialist bekommt Startblatt und Relikte für Null, darf aber auch Farbe und Grand spielen. |
| 3 | **Tech-Stack: Rust für den Spielkern, Web-App für die Präsentation.** | Rust-Kern ist headless simulierbar und testbar. Im Browser läuft er als WebAssembly (ADR 0009, ersetzt den ursprünglichen Godot-Plan). |
| 4 | **Regeln dürfen gebogen werden, wenn es dem Spiel dient.** | Authentizität ist Geschmack, kein Dogma. Skatspieler sollen das Spiel wiedererkennen, nicht nachprüfen. |

---

## 3. Designsäulen

1. **Die Formel ist das Spiel.** Jede Entscheidung des Spielers muss sich an einem Faktor der Spielwertformel festmachen lassen. Wenn ein Relikt an keiner Stelle der Formel dreht, gehört es nicht ins Spiel.
2. **Das Blatt ist geteilt.** Deine Karten können bei den Gegnern landen. Deckbau ist ein Risiko, kein reiner Vorteil. Das ist die einzige Idee, die dieses Spiel von Balatro unterscheidet, und sie wird verteidigt.
3. **Zählen lohnt sich.** Skat lebt vom Mitzählen. Jedes System muss Spielern, die mitzählen, einen Vorteil geben, und darf nie durch Zufall diesen Vorteil zunichtemachen.
4. **Kurze Partien, lange Runs.** Eine Partie dauert unter zwei Minuten. Ein Run dauert 40 bis 60 Minuten.
5. **Stammtisch, nicht Turnier.** Ton, Figuren und Sprache kommen aus der Kneipe. Lokalkolorit ist erwünscht.

---

## 4. Der Skat-Kern

### 4.1 Was unverändert bleibt

- **32 Karten**: 7, 8, 9, 10, Bube, Dame, König, Ass in Karo, Herz, Pik, Kreuz.
- **Augen**: Ass 11, Zehn 10, König 4, Dame 3, Bube 2, 7/8/9 null. Summe 120.
- **Verteilung**: 10 Karten pro Spieler, 2 Karten im Skat.
- **Trumpfordnung im Farbspiel**: Kreuz-Bube, Pik-Bube, Herz-Bube, Karo-Bube, dann Ass, 10, K, D, 9, 8, 7 der Trumpffarbe.
- **Grand**: nur die vier Buben sind Trumpf.
- **Stichregeln**: Bedienpflicht. Wer nicht bedienen kann, darf stechen oder abwerfen. Der Stichgewinner spielt aus.
- **Gewinnbedingung**: Der Alleinspieler braucht 61 Augen (Stiche plus Skat). 90 Augen sind Schneider. Alle Stiche sind Schwarz.
- **Grundwerte**: Karo 9, Herz 10, Pik 11, Kreuz 12, Grand 24.
- **Spitzen**: „Mit“ oder „ohne“ wird ab Kreuz-Bube lückenlos gezählt. Im Farbspiel zählen Trumpfkarten der Farbe mit.

### 4.2 Die Spielwertformel als Score-Engine

```
Spielwert = Grundwert × Gewinnstufen

Gewinnstufen = Spitzen
             + 1  (Spiel)
             + 1  Hand
             + 1  Schneider
             + 1  Schneider angesagt
             + 1  Schwarz
             + 1  Schwarz angesagt
             + 1  Ouvert
```

In der Sprache des Genres: **Grundwert sind die Chips, Gewinnstufen sind der Mult.** Alle Modifikationen im Spiel greifen an genau einer dieser Stellen an:

| Hebel | Beispiele für Upgrades |
|---|---|
| Grundwert erhöhen | Kreuz zählt 15 statt 12. Grand zählt 30. |
| Spitzen manipulieren | Ein fünfter Bube im Blatt. Karo-Bube zählt als zweite Spitze. |
| Gewinnstufen hinzufügen | Neue Stufen durch Hausregeln: „Spitze“ (letzter Stich mit einer Sieben), „Kontra“, „Bock“. |
| Augen verändern | Pik-Ass zählt 15. Alle 10er zählen 11. Siebener zählen 1. |
| Schwellen senken | Schneider ab 85 Augen. Gewinn ab 58 Augen. |

### 4.3 Was gebogen wird

**Reizen entfällt.** Stattdessen gibt es die **Ansage**. Die Reihenfolge folgt der Skatordnung (ADR 0011): Erst entscheidet der Spieler, ob er den Skat aufnimmt oder Hand spielt; nach dem Aufnehmen drückt er zwei Karten. Danach erklärt er:

1. Spielart (Farbe oder Grand; beim Null-Charakter Null-Varianten).
2. Risikostufe: *Normal*, *Hand* (Skat bleibt liegen), *Schneider angesagt*, *Schwarz angesagt*, *Ouvert* (offen spielen). Seit ADR 0013 (Spieltest vom 01.11.2026, gegen die Skatordnung) gehen Schneider, Schwarz und Ouvert auch nach dem Aufnehmen: Im Farbspiel und Grand stehen dann *Normal*, *Schneider angesagt*, *Schwarz angesagt* und *Ouvert* zur Wahl, beim Null *Normal* und *Ouvert*. Die Stufe *Hand* zählt nur, wenn aus der Hand gespielt wird; eine Ansage nach dem Skat bringt ihre eigenen Stufen ohne Hand (Grand mit 1, Schneider angesagt und erreicht: 4 Stufen statt 5 aus der Hand). Im Kern: `Declaration::with_hand`, im JSON `after_skat: true`.

Jede Stufe erhöht die Gewinnstufen, aber ein Fehlschlag auf einer angesagten Stufe gilt als verloren. Das ersetzt den Push-your-luck-Charakter des Reizens durch eine einzige, klare Entscheidung.

**Einmischen statt Passen** (Spieltest vom 01.11.2026, Entscheidung des Nutzers): Gefällt dem Spieler sein Blatt nicht, mischt er es vor der Ansage ein: Alle Hände und der Skat werden neu gegeben, das kostet kein Spiel der Serie. Bis zu zweimal pro Spiel (`redeals` in `run.ron`), danach muss eines der Blätter gespielt werden. Nach dem Aufnehmen des Skats und im Ramsch geht es nicht mehr. Einpassen (ein Spiel auslassen, das ein Spiel vom Limit kostete) gibt es nicht mehr. Einzelheiten in 5.1.

**Verlorene Spiele kosten das Doppelte.** Wie im echten Skat wird ein verlorenes Spiel mit dem doppelten Spielwert negativ gewertet. Das ist der wichtigste Risiko-Hebel des gesamten Spiels und bleibt.

**Kein Überreizen.** Da es kein Reizen gibt, gibt es auch keinen Überreiz-Fehler. Die Spitzen werden nach dem Spiel mit dem tatsächlichen Blatt inklusive Skat gezählt.

**Ramsch, Bock, Kontra, Schieberamsch, Revolution, Schenken** sind keine Standardregeln, sondern Relikte, Boss-Mechaniken oder Ereignisse (siehe Abschnitt 8).

### 4.4 Regelklärungen für das veränderbare Blatt (M2)

Diese Regeln gibt es im echten Skat nicht, weil das Blatt dort nie verändert wird. Entschieden von Claude am 01.11.2026 im Rahmen der Autonomie-Regel.

- **Spitzen zählen nur über Trümpfe, die im Spiel sind.** Ist der Kreuz-Bube entfernt, beginnt die Zählung beim nächsten vorhandenen Trumpf. Sonst brächte jedes Entfernen eines Buben allen ein Gratis-„ohne“.
- **Blankos** bedienen nie und gewinnen nie. Wird eine Blanko ausgespielt, bestimmt die erste normale Karte die Farbe. Drei Blankos gehen an den Ausspieler.
- **Gezinkte Karten** steigen eine Stufe. Bei gleicher Stärke behält die früher gespielte Karte den Stich.
- **Schneider** wird immer an der verlierenden Seite gemessen (höchstens 30 Augen). Relikte wie „Jungfrau“ legen zusätzlich Bereiche der Alleinspieler-Augen fest, die ein gewonnenes Spiel zum Schneider machen.
- **Kartenmodifikationen wirken für jeden, der die Karte spielt** (ADR 0005), auch die Modifikation „Spitze“. Die gleichnamige Hausregel verlangt dagegen den Alleinspieler.
- **Bockrunde** wird bei der Ansage verbraucht, nicht beim Geben. So verschenkt Einmischen den Bock nicht.
- **Schenken**: Die restlichen Stiche gehen an den Alleinspieler. Das Spiel ist damit Schneider, und Schwarz, falls die Gegner noch keinen Stich hatten.

Ergänzt in M5 (entschieden von Claude am 01.11.2026):

- **Bube ehrenhalber** (Kartenmodifikation `honorary_jack`, Effekt `HonoraryJack`): Im Farbspiel ist die Karte Trumpf und steht eine halbe Stufe unter dem Karo-Buben: Sie schlägt das Trumpf-Ass und alles darunter und verliert gegen jeden Buben, egal in welcher Reihenfolge gespielt wird. Sie bedient Trumpf und nicht mehr ihre gedruckte Farbe; wird Trumpf ausgespielt, muss sie bedient werden. Liegt sie auf einer Karte der Trumpffarbe, steigt diese Karte über das Ass. Mit Rangbonus („Gezinkt“) steigt sie ganze Stufen von dort: einmal gezinkt schlägt sie den Karo-Buben und verliert gegen den Herz-Buben. Ein gezinktes Trumpf-Ass erreicht weiter den Karo-Buben (Gleichstand, der frühere behält den Stich) und schlägt damit den Buben ehrenhalber. Zwei Buben ehrenhalber im selben Stich: Der frühere behält ihn. Die **Spitzen zählen sie nicht mit**: Gezählt wird weiter die gedruckte Trumpfordnung; ein Bube ehrenhalber ist keine zusätzliche Spitze, und eine Trumpfkarte mit der Modifikation zählt an ihrer gedruckten Stelle. Im Grand, im Null und im Ramsch bleibt die Karte, was gedruckt ist, ebenso ein echter Bube. Im Code: `rules::trump_order_key` (Stärke, dann 0 für den Buben ehrenhalber, 1 für gedruckte Trümpfe), `is_trump` und `follows_lead` kennen ihn; die Gegner-KI sieht über `trump_strength` die Stärke des Karo-Buben.
- **Ramponiert** (Fluch `battered`, Effekt `NoDiscard`): Die Karte darf nicht gedrückt werden, solange zwei andere Karten gedrückt werden können; sonst so wenige ramponierte wie möglich (`rules::valid_discard`, Fehler `DiscardForbidden`). Das gilt auch für die Karte, die beim Skattausch der Oma oder des Opas in den Skat geht.
- **Mehrere Skattausch-Figuren**: Jede Figur mit `SeriesSkatSwap` gewährt ihren eigenen Tausch, einmal pro Serie (Oma: ein Unter, Opa: ein Daus). Sie können im selben Handspiel nacheinander genutzt werden. Die Sicht nennt die Karten, die herauskommen können (`skat_swap_cards`), die erste nimmt `TakeFromSkat` ohne `take`.

Rest abkürzen (entschieden von Claude am 01.11.2026, auf Wunsch des Nutzers):

- **Wann:** Der Spieler ist am Zug, und die restlichen Karten können an der Wertung nichts mehr ändern, egal wie alle drei sie spielen (auch er selbst, nicht nur bei bestem Spiel). Dann bietet der Tisch „Rest abkürzen“ an (`GameView::can_settle`, Aktion `RunAction::SettleGame`).
- **Entschieden heißt:** (1) Kein Hook, der während des Spiels feuert (`OnCardPlayed`, `OnTrickEnd`, z. B. Schenken, Revolutionär, Auf dem Trockenen), kann noch feuern. (2) Das Ergebnis nach den Regeln (gewonnen, Schneider und Schwarz beider Seiten, Grundwert, Gewinnstufen, Spielwert) ist für jede Augen- und Stichzahl gleich, die die Reststiche noch bringen können, mit den geltenden Schwellen (Gewinnschwelle, „Jungfrau“). Null: Der Alleinspieler bekommt in keiner Fortsetzung einen Stich (gewonnen, mit gleichem Stilbonus: Siebenen, nie bedient, Kurzschluss) oder in jeder (verloren). (3) Jeder `OnScore`-Hook feuert in allen Fortsetzungen oder in keiner, auch die des letzten Stichs (Spitze, Glückspfennig, Schlusslicht). Ramsch wird nie abgekürzt.
- **Wie gemessen:** Schranken (kleinster und größter Wert) der Reststiche aus drei Quellen, billig zuerst: aus den Karten allein; für Null ein Test auf den Händen (der Alleinspieler spielt nicht aus, kein Gegner hält eine Blanko, in jeder Farbe mit einer gewinnfähigen Karte des Alleinspielers ist jede Gegnerkarte höher); in den letzten vier Stichen eine vollständige Suche über alle legalen Karten aller Sitze mit offenen Händen (`core/src/endgame.rs`, höchstens 2000 Stichgrenzen, sonst nicht angeboten).
- **Was passiert:** Der Kern spielt die Reststiche mit fester Regel (jeder Sitz seine erste legale Karte) über das normale Kartenspiel aus. Ereignisse, Hooks, Wertung, Groschen und Statistik sind dieselben wie beim Ausspielen. Die Oberfläche lässt die Karten kurz verschwinden und zeigt dann die Abrechnung.

---

## 5. Run-Struktur

```
Run (eine Tournee)
└── Abschnitt (ein Skatabend in einer Kneipe), 3 bis 5 pro Run
    ├── Runde 1: Serie      (Zielwert, 3 Spiele, gleiche zwei Gäste)
    ├── Runde 2: Serie      (höherer Zielwert)
    ├── Runde 3: Boss-Serie (Gegner mit Sonderregel)
    └── dazwischen: Tresen (Shop), Ereignisse, Rast
```

| Ebene | Skat-Begriff | Genre-Gegenstück | Dauer |
|---|---|---|---|
| Ein Spiel | Skat, Ansage, 10 Stiche, Abrechnung | Eine Hand | ~90 Sekunden |
| Eine Serie | 3 Spiele mit Punkteziel gegen dieselben zwei Gäste, je Spiel bis zu zweimal einmischen | Ein Blind | 3 bis 5 Minuten |
| Ein Abschnitt | Skatabend in einer Kneipe | Ein Ante | 15 bis 20 Minuten |
| Der Run | Tournee durch 3 bis 5 Kneipen | Der Run | 45 bis 60 Minuten |

### 5.1 Die Serie

- Jede Serie hat einen **Zielwert** (siehe 5.2) und ein **Spiellimit**: seit dem 01.11.2026 **3 Spiele** gegen dieselben zwei Stammgäste („3 Spiele bilden eine Serie, 2 Serien münden in eine Boss-Serie“, Entscheidung des Nutzers nach dem Spieltest). Vorher: M5 8 Spiele mit bis zu 6 Pässen, M2 5 und 3 (5.2).
- Gewonnene Spiele addieren ihren Spielwert. Verlorene Spiele ziehen den doppelten Spielwert ab.
- **Einmischen** (`RunAction::Redeal`): Vor der Ansage, solange der Skat unberührt ist, gibt der Spieler das Blatt zurück und es wird neu gegeben. Das kostet kein Spiel und keine Punkte, die Vorhand bleibt. Jedes Spiel erlaubt zwei (`redeals` in `run.ron`), danach muss eines der Blätter gespielt werden. Der Zaungast hat eins weniger (9.5), die Hausregel Schieben gibt eins mehr pro Spiel (8.1), Ein Bier und Kräuter geben eins für den Rest der Serie, das nach denen des Spiels verbraucht wird (8.3), „Neues Blatt“ mischt einmal, ohne eins zu verbrauchen, und der Schnaps „Reue“ nimmt das letzte Einmischen zurück. Die Kopfzeile zeigt, wie oft der Spieler im laufenden Spiel noch einmischen darf (eigene plus die der Serie, `SeriesView::redeals_left`). Der Ramsch des Ramschers wird nie eingemischt. Das Einmischen ersetzt das Einpassen, das ein Spiel vom Limit kostete: Wer seine Pässe verbraucht hatte, musste schlechte Blätter spielen und verlor faktisch den Run.
- Wer das Limit erreicht, ohne den Zielwert zu schaffen, verliert den Run.
- Übrige Spiele am Ende einer gewonnenen Serie werden in Groschen umgewandelt (Zinsmechanik wie in Balatro). Mit 3 Spielen sind das 0 bis 2.
- **Alte Spielstände** laden: Eine Serie, die mitten in den alten 8 Spielen gespeichert wurde, spielt mit ihrem gespeicherten Spiellimit zu Ende; ab der nächsten Serie gelten 3 Spiele. Übrige Pässe verfallen, Einmischen gilt sofort nach der neuen Regel (das laufende Spiel hat seine zwei). Gespeicherte Statistik zählt eingepasste Spiele als Einmischen. Alte Namen (`Pass`, `passes`, `passes_used`, `games_passed`, `PassesAdded`, `ExtraPasses`, `SeriesRedeal`, `series_redeals`, `without_pass`) werden weiter gelesen.
- **Im Code**: `Series::redeals_in_game` (verbraucht im laufenden Spiel, mit jedem neuen Spiel 0), `Series::extra_redeals` (Ein Bier, Kräuter), `Series::redeals_used` (alle der Serie, auch „Neues Blatt“), `Run::redeals_left(content)`; Fehler `NoRedealLeft`.

### 5.2 Zielwert-Progression

**Vorläufig seit 01.11.2026 (3 Spiele je Serie, zweimal einmischen je Spiel).** Mit drei Spielen statt acht war die M5-Kurve (36/44/54 …) viel zu hoch. Die Kurve unten ist eine **erste grobe Setzung** aus einer kurzen Messung; die volle Kalibrierung folgt in einer späteren Testrunde (`balance`). Ziel: Der PIMC-Spieler gewinnt normale Serien zu etwa 75 bis 85 %, Boss-Serien zu etwa 60 bis 70 %.

| Abschnitt | Gegner | Alleinspieler (`run.ron`) | Nullspieler (`characters.ron`) |
|---|---|---|---|
| 1 | Heuristik | 24 / 30 / 36 | 24 / 30 / 36 |
| 2 | PIMC 6:2 | 30 / 36 / 42 | 24 / 30 / 36 |
| 3 | PIMC 8:3 | 36 / 42 / 48 | 30 / 36 / 42 |
| 4 | PIMC 12:3 | 42 / 48 / 54 | 36 / 42 / 48 |
| 5 | PIMC 16:3 | 48 / 54 / 60 | 42 / 48 / 54 |

Die Charaktere ohne eigene Zielwerte (Grandspieler, Zaungast, Wirtin) nehmen die des Alleinspielers; der Nullspieler behält sein Muster (eine Zeile später) mit den neuen Werten. Boss-Faktoren (11) unverändert.

Gewählt mit der Erfolgskurve einer Serie ohne Relikte (`--curve`, 1500 Serien, Seed 42, `--bot pimc:12:3`, P(Serie gewonnen) bei Zielwert 24 / 36 / 48 / 60): gegen Heuristik-Gegner 80 / 76 / 72 / 67 %, PIMC 6:2 78 / 73 / 69 / 64 %, PIMC 12:3 76 / 71 / 66 / 61 %, PIMC 16:3 75 / 70 / 66 / 60 %. Die Decke (Zielwert 1) liegt bei 80 bis 85 %; die Kurve ist flach, weil drei Spiele mit doppelt zählenden Niederlagen viel Zufall tragen. Relikte heben die Quoten im Run. Dann im ganzen Run (2 × 500 Runs, Seeds 42 und 43, `--bot pimc:12:3`, Alleinspieler):

| Serie | 1 | 2 | Boss |
|---|---|---|---|
| Abschnitt 1 | 80,4 % | 79,0 % | 69,3 % |
| Abschnitt 2 | 76,8 % | 79,3 % | 67,6 % |
| Abschnitt 3 | 79,6 % | 77,1 % | 67,6 % |
| Abschnitt 4 | 84,0 % | 77,8 % | 75,5 % |
| Abschnitt 5 | 81,1 % | 93,3 % | 75,0 % |

Abschnitt 2 / 3 / 4 / 5 erreicht 44,0 / 18,1 / 7,5 / 3,7 %, Run gewonnen 2,1 %. Ab Abschnitt 4 sind die Stichproben klein (26 bis 75 Serien). Zum Vergleich `main` 07141fd (8 Spiele, 6 Pässe, M5-Kurve, gleiche Messung): normale Serien 81 bis 94 %, Boss 67 bis 83 %, erreicht 50,7 / 27,5 / 16,1 / 10,1 %, Run gewonnen 6,4 %. Ein Run spielt jetzt im Mittel 7,4 Spiele und 5,6 Einmischen (0,75 je Spiel). Nullspieler (300 Runs, Seed 42): normale Serien in Abschnitt 1 und 2 71 bis 77 %, Boss 49 bis 65 %, Abschnitt 2 erreicht 35 %; für ihn ist die Kurve noch etwas zu hoch (17).

```sh
cargo run --release -p skat_sim -- --seed 42 --runs 500 --bot pimc:12:3
cargo run --release -p skat_sim -- --seed 42 --runs 1500 --bot pimc:12:3 --curve --defenders pimc:12:3
```

#### M5-Kurve (8 Spiele, 6 Pässe, bis 01.11.2026)

Spiellimit 8, Pässe 6 pro Serie. Die Zielwerte des Alleinspielers sind Reizwerte (`run.ron`), der Nullspieler hat eigene (`characters.ron`, 9.2).

| Abschnitt | Gegner | Alleinspieler | Nullspieler |
|---|---|---|---|
| 1 | Heuristik | 36 / 44 / 54 | 72 / 80 / 88 |
| 2 | PIMC 6:2 | 44 / 48 / 54 | 88 / 99 / 120 |
| 3 | PIMC 8:3 | 48 / 54 / 60 | 110 / 132 / 160 |
| 4 | PIMC 12:3 | 54 / 60 / 66 | 144 / 176 / 216 |
| 5 | PIMC 16:3 | 60 / 66 / 72 | 198 / 240 / 264 |

Kalibriert am 01.11.2026 gegen den PIMC-Spieler (`--bot pimc:12:3`, Kartenspiel per Double-Dummy-Stichproben, Ansage, Passen und Tresen vom `RunBot`), Messung in `docs/balance/2026-11-01-m5-v1.md`. Zielbild: Ein guter Spieler erreicht Abschnitt 2 in etwa einem Drittel der Runs und gewinnt selten, aber messbar; beide Charaktere liegen gleichauf. Ergebnis (3 × 3000 Runs, Seeds 42 bis 44):

| Abschnitt erreicht | 2 | 3 | 4 | 5 | Run gewonnen |
|---|---|---|---|---|---|
| Alleinspieler | 36,3 % | 14,5 % | 6,5 % | 3,0 % | 1,3 % |
| Nullspieler | 37,0 % | 16,9 % | 8,1 % | 3,9 % | 1,6 % |

```sh
cargo run --release -p skat_sim -- --seed 42 --runs 3000 --bot pimc:12:3
cargo run --release -p skat_sim -- --seed 42 --runs 3000 --bot pimc:12:3 --character null
# dasselbe mit --seed 43 und --seed 44
```

Warum 8 Spiele statt 5: Gegen PIMC-Gegner gewinnt der PIMC-Spieler mit 5 Spielen und 3 Pässen auch bei Zielwert 1 nur rund 75 % einer Serie, weil ein verlorenes Spiel doppelt zählt. Über 15 Serien ist der Run dann fast nur Pech, die Kurve wirkt kaum (zwei flache Kurven bis 72 und bis 60 lagen innerhalb des Rauschens, 38,1 und 36,5 % Abschnitt 2) und Relikte bewegen wenig. Mit 8 Spielen und 6 Pässen liegt die Decke bei 85 %, die Zielwerte beißen wieder, und die Schwierigkeit kommt wie vorgesehen aus Kurve und Bossen (10.3). Der Spieler wartet länger auf ein gutes Blatt, spielt je Serie aber nur rund zwei Spiele. Ein voller Run sind damit etwa 28 Spiele und 40 Pässe, grob 50 Minuten.

Zur Form: Abschnitt 1 (Heuristik-Gegner) verlangt etwa zwei gewonnene Spiele. Ab Abschnitt 2 spielen die Gegner PIMC und werden je Abschnitt stärker, deshalb steigt die Kurve danach nur um 6 je Stufe. Jede normale Serie gewinnt der Bot zu 76 bis 82 %, jede Boss-Serie zu 63 bis 72 %; ein Abschnitt hält damit 40 bis 46 % der Runs, die ihn erreichen. Ohne multiplikative Relikte wächst ein gewonnenes Spiel kaum über 72; eine steilere Kurve wäre ab Abschnitt 4 unschlagbar. Null-Spielwerte kommen in Stufen von 23; für den Nullspieler ist alles zwischen zwei Stufen (etwa 72 bis 92) gleich schwer (9.2).

Vorher (M2, Spiellimit 5, Pässe 3, Kurve 18 bis 180, `main` 3eb9884): gegen den PIMC-Spieler 38,7 / 14,3 / 4,1 / 0,6 % erreicht, Run gewonnen 0,07 % (6 von 9000), Nullspieler 27,7 / 7,7 / 2,3 / 0,7 %, 0,10 %. Die M2-Messung mit dem Heuristik-Bot steht in `docs/balance/2026-11-01-m2-zielwerte.md`.

### 5.3 Kurzer Abend (M7)

Entschieden über das Cockpit (16, „Run-Länge“): 5 Abschnitte als Standard, dazu ein kurzer Abend mit 3 Abschnitten für unterwegs, Freischaltungen in beiden. Der Spieler wählt auf dem Startbildschirm zwischen Charakter und Startknopf „Ganzer Abend (5 Abschnitte)“ oder „Kurzer Abend (3 Abschnitte)“; die Wahl bleibt im Browser (`stammtisch.length`, nur Anzeige). Die Kopfzeile zeigt „Abschnitt x/3“, die Kreidetafel drei Zeilen.

- **Daten**: `run.ron` nennt mit `short_sections` die Zeilen der Kurve (Zielwerte, Gegner, beim Nullspieler seine eigenen Zielwerte aus `characters.ron`), die ein kurzer Abend spielt. Gewählt ist `[0, 1, 2]`: Abschnitt 1 bis 3 mit Zielwerten und Gegnern wie im ganzen Abend, in der Boss-Serie des dritten sitzt statt eines gezogenen Bosses der Endboss „Der Alte“. Im Kern: `RunLength::Full | Short` in `RunStart` und `Run` (ältere Spielstände laden als `Full`), `RunConfig::sections`, `RunConfig::curve_row`, in der Sicht `sections` und `length`.
- **Freischaltungen und Statistik** zählen in beiden. „Abschnitt 4 erreichen“ (Der Stammgast) geht nur im ganzen Abend, „Run gewinnen“ (Nullspieler) auch mit einem gewonnenen kurzen Abend. `RunWon` und `RunLost` tragen die Länge; der Meta-Zustand zählt Runs, Siege und den weitesten Abschnitt je Länge (`MetaStats::lengths`), damit das Vereinsbuch sie später getrennt zeigen kann. Ältere Meta-Stände laden; ihre Runs stehen nur in den Summen.
- **Seeds teilen**: Der Link eines kurzen Abends endet auf `-kurz` (`#seed-4343-alleinspieler-kurz`), Links ohne Zusatz sind wie bisher ganze Abende. Der Tages-Seed gilt für die gewählte Länge. Die Liste der letzten Runs merkt sich die Länge, „Nochmal“ spielt sie wieder so.
- **Simulator**: `--short`.

Gewählt mit dem Simulator. Kandidaten (je 3 × 1000 Runs, Seeds 42 bis 44, `--bot pimc:12:3 --short`, `short_sections` jeweils gesetzt):

| Kandidat | Charakter | Abschnitt 2 erreicht | Abschnitt 3 erreicht | Run gewonnen | gehalten je Abschnitt |
|---|---|---|---|---|---|
| Zeilen 1, 3, 5 (`[0, 2, 4]`) | Alleinspieler | 37,1 % | 12,3 % | 4,8 % | 37 / 33 / 39 % |
| Zeilen 1, 3, 5 (`[0, 2, 4]`) | Nullspieler | 35,8 % | 13,1 % | 3,6 % | 36 / 37 / 27 % |
| Zeilen 1, 2, 3 (`[0, 1, 2]`) | Alleinspieler | 37,1 % | 14,5 % | 6,5 % | 37 / 39 / 45 % |

Mit den Zeilen 1, 3, 5 kommen die Zielwerte und die PIMC-Stärke der späten Abschnitte schon nach ein oder zwei Tresen; ab dem zweiten Abschnitt ist jeder härter als im ganzen Abend (33 bis 39 % gehalten statt 40 bis 46 %, der Nullspieler im letzten nur 27 %). Mit 1 bis 3 hält jeder Abschnitt so viel wie im ganzen Abend, und der Endboss im dritten ist so schwer wie ein gezogener Boss dort. Der kurze Abend ist damit pro Abschnitt gleich schwer und als Ganzes leichter, weil zwei Abschnitte fehlen. Das ist gewollt: Er ist für unterwegs, etwa drei Fünftel eines Runs (rund 17 Spiele, 30 Minuten).

Ergebnis (3 × 3000 Runs, Seeds 42 bis 44, mit den Flüchen aus 7.2):

| Länge | Charakter | Abschnitt 2 | 3 | 4 | 5 | Run gewonnen | gehalten je Abschnitt |
|---|---|---|---|---|---|---|---|
| ganzer Abend | Alleinspieler | 36,5 % | 14,8 % | 6,9 % | 3,2 % | 1,54 % | 36 / 41 / 46 / 47 / 48 % |
| kurzer Abend | Alleinspieler | 36,5 % | 14,8 % | – | – | 6,84 % | 36 / 41 / 46 % |
| ganzer Abend | Nullspieler | 36,8 % | 16,6 % | 7,9 % | 3,9 % | 1,71 % | 37 / 45 / 48 / 49 / 44 % |
| kurzer Abend | Nullspieler | 36,8 % | 16,6 % | – | – | 7,21 % | 37 / 45 / 43 % |

Bis Abschnitt 3 sind die Runs gleich (gleiche Seeds, gleiche Karten, gleiche Kurve); im dritten Abschnitt gewinnt der Alleinspieler die Boss-Serie gegen den Alten zu 71 % wie gegen einen gezogenen Boss, der Nullspieler zu 67 % statt 74 %.

```sh
cargo run --release -p skat_sim -- --seed 42 --runs 3000 --bot pimc:12:3 --short
cargo run --release -p skat_sim -- --seed 42 --runs 3000 --bot pimc:12:3 --short --character null
# dasselbe mit --seed 43 und --seed 44; ohne --short der ganze Abend
```

Freischaltungen im kurzen Abend (`skat_sim --seed 42 --runs 300 --unlocks --short`, Heuristik-Bot, 20 Folgen mit frischem Meta-Zustand): alle erreichbar bis auf `fourth_evening`. `tour_won` (Nullspieler) kam nach 24,9 Runs im Mittel (2 bis 61), im ganzen Abend nach rund 1700 (9.3). Seit dem 01.11.2026 zählt ein gewonnener kurzer Abend nicht mehr für `tour_won` (9.3); im kurzen Abend ist es damit nicht erreichbar. Siehe 17.

### 5.4 Einsatz-Leiter (M7)

Entschieden über das Cockpit (16, „Schwierigkeit“): Jeder gewonnene Run hebt die Gewinngrenze des nächsten um ein Auge (61, 62, 63 …). Schneider rückt nicht mit. Die Leiter ist endlos. Umgesetzt von Claude am 01.11.2026, die Einzelheiten im Rahmen der Autonomie-Regel:

- **Regel**: Bei Einsatz n gewinnt der Alleinspieler ein Farbspiel oder einen Grand ab `61 + n` Augen. Schneider bleibt, wo er ist: Die Gegner sind mit höchstens 30 Augen Schneider (90 für den Alleinspieler), ein verlorenes Spiel ist für den Alleinspieler mit höchstens 30 Augen Schneider. Ab Einsatz 30 ist damit jedes gewonnene Farbspiel Schneider, ab Einsatz 60 reichen ohne zusätzliche Augen (Kartenmodifikationen wie „Fett“, Hausregeln wie „Kleinvieh“) auch alle 120 nicht mehr. Das ist gewollt: Die Leiter ist endlos, oben ist das Farbspiel kaum noch zu gewinnen.
- **Null und Ramsch** kennen keine Augengrenze und bleiben gleich. Je höher der Einsatz, desto stärker wird Null im Vergleich, für den Nullspieler wie für den Alleinspieler (ADR 0012). Vorerst so gelassen; bei Bedarf misst es der Simulator (`--stake`).
- **Relikte, die die Gewinnschwelle senken** („Ein Auge zudrücken“ 58, „Hufeisen“ 56), sprechen vom Spiel ohne Einsatz und senken die erhöhte Grenze um denselben Betrag: Gewinn ab `Wert + n` (Hufeisen bei Einsatz 2: ab 58). Es zählt weiter die niedrigste Schwelle. Bereiche für Schneider aus Sicht des Alleinspielers („Jungfrau“ 61 bis 70) bleiben, wie sie sind, und gelten nur für gewonnene Spiele; bei Einsatz 5 also 66 bis 70. Im Code: `Thresholds::stake`, `Thresholds::with_stake`, `Thresholds::lower_win`; `won_by_lowered_threshold` misst gegen `61 + n`.
- **Freischaltung**: Der Meta-Zustand merkt sich den höchsten freigeschalteten Einsatz je Charakter und je Länge (`MetaState::stakes`, `max_stake`). Ein gewonnener Run mit Einsatz n schaltet n + 1 frei; ein Sieg weiter unten oder ein verlorener Run nimmt nichts zurück. Ganzer und kurzer Abend steigen getrennt, weil ein kurzer Abend leichter zu gewinnen ist (5.3); sonst wäre der ganze Abend über den kurzen hochzuziehen. Ältere Meta-Stände laden mit Einsatz 0 überall.
- **Start**: Der Spieler wählt vor dem Run den Einsatz zwischen 0 und dem höchsten freigeschalteten, voreingestellt ist der höchste. `RunStart::stake`, `Run::stake` (ältere Spielstände: 0); `MetaState::run_start_staked` lehnt einen höheren ab (`MetaError::StakeLocked`). `RunWon` und `RunLost` tragen den Einsatz (ältere Ereignisse: 0).
- **Sicht**: `RunView::stake` und `RunView::win_threshold` (`61 + n` vor Relikten), im Spiel `GameView::win_threshold` mit den Relikten. Die Wertvorschau rechnet den knappsten Sieg mit dieser Schwelle. In der Sicht des Meta-Zustands nennt jeder Charakter `max_stakes` je Länge.
- **Web-App**: Auf dem Startbildschirm unter „Ganzer/Kurzer Abend“ eine Zeile mit Pfeilen, „Einsatz 3 · Gewinn ab 64 Augen“. Sie erscheint erst, wenn irgendein Charakter einen Run gewonnen hat; bis dahin ist alles Einsatz 0. Ein Wechsel von Charakter oder Länge stellt wieder den höchsten ein. Die Kopfzeile zeigt bei Einsatz > 0 „ab 64“, die Spielzeile bei Farbspiel und Grand die Schwelle des Spiels („ab 64“), sobald sie von 61 abweicht (auch durch Relikte). Die Regelkarte erklärt die Leiter.
- **Seeds teilen**: Ein Link mit Einsatz endet auf `-einsatz<n>` (`#seed-4343-alleinspieler-kurz-einsatz2`), Links ohne sind Einsatz 0. Wer den Einsatz noch nicht erreicht hat, spielt den Seed mit seinem höchsten (wie ein gesperrter Charakter als Alleinspieler gespielt wird); das Angebot sagt es. Der Tages-Seed gilt für den gewählten Einsatz, die Liste der letzten Runs merkt ihn sich, „Nochmal“ spielt ihn wieder (höchstens mit dem freigeschalteten).
- **Simulator**: `--stake N`, auch mit `--unlocks` (dann ohne Prüfung der Freischaltung).

Nicht gemessen: Wie stark ein Auge die Siegquote senkt, ist noch nicht kalibriert (siehe 17).

---

## 6. Ein Spiel im Detail

1. **Geben.** Das aktuelle Blatt (32 Karten plus Modifikationen) wird gemischt. 10/10/10/2.
2. **Blatt ansehen.** Der Spieler sieht seine 10 Karten. Relikte, die Informationen geben (Kiebitz), wirken jetzt.
3. **Skat oder Hand.** Skat aufnehmen und 2 Karten drücken, oder Hand spielen. Gedrückte Karten zählen zu den Augen des Alleinspielers. Oder vorher einmischen: neu geben lassen, bis zu zweimal pro Spiel (5.1).
4. **Ansage.** Spielart und Risikostufe wählen (ADR 0011, 0013).
5. **10 Stiche.** Vorhand spielt aus. Die Gegner-KI spielt gemeinsam gegen den Spieler.
6. **Abrechnung.** Augen zählen, Spitzen bestimmen, Gewinnstufen summieren, Spielwert berechnen, Serie-Konto aktualisieren.
7. **Triggerphase.** Relikte mit Nachwirkung (z. B. „Bock“ läuft weiter) werden aktualisiert.

Die Rollen Vorhand, Mittelhand, Hinterhand rotieren pro Spiel. Vorhand zu sein ist ein echter Vorteil im Skat und bleibt als Rhythmus erhalten.

---

## 7. Deckbau

### 7.1 Das geteilte Blatt

Es gibt genau **ein Blatt** pro Run. Alle drei Spieler spielen daraus. Jede Änderung am Blatt betrifft potenziell auch die Gegner. Das ist Absicht und das zentrale Spannungsmoment des Deckbaus.

Das Blatt darf wachsen und schrumpfen, aber die Verteilung bleibt 10/10/10/2. Bei mehr als 32 Karten bleibt der Rest **im Talon** und kommt nicht ins Spiel. Bei weniger als 32 Karten (durch Zerstörung) wird mit **Blanko-Karten** aufgefüllt, die keine Farbe und keine Augen haben und immer verlieren. Ein dünnes Blatt ist damit eine echte Strategie: Weniger Luschen heißt mehr Augen auf der Hand.

### 7.2 Kartenmodifikationen (gezinkte Karten)

Modifikationen hängen an einer konkreten Karte und gelten, **egal wer sie spielt**. Ausnahme: Modifikationen mit dem Zusatz *„nur in deiner Hand“*, die seltener und teurer sind.

| Kategorie | Beispiel | Hebel in der Formel |
|---|---|---|
| Augen | „Fett“: +4 Augen | Gewinnschwelle |
| Rang | „Gezinkt“: zählt eine Stufe höher | Stichgewinn |
| Trumpf | „Bube ehrenhalber“: diese Karte ist im Farbspiel Trumpf unter den Buben | Spitzen |
| Grundwert | „Stempel“: Wird diese Karte Trumpffarbe, Grundwert +3 | Grundwert |
| Stufe | „Spitze“: Gewinnt diese Karte den letzten Stich, +1 Gewinnstufe | Gewinnstufen |
| Information | „Durchsichtig“: Der Spieler sieht, wer diese Karte hält | Zählen |
| Negativ | „Ramponiert“: Diese Karte darf nicht gedrückt werden | Fluch, gibt Geld |

**Umgesetzt in M4: 20 Modifikationen, in M5: 22** (`card_mods.ron`, Preise Platzhalter für `balance`). Zufällige Modifikationen (Stempelkissen, Kartengeber) ziehen gleichverteilt aus dem Pool des Charakters, seit M7 ohne Flüche, gesperrte nicht (Belohnung `CardMod` einer Freischaltung, 9.3). Ereignisse können eine bestimmte Modifikation vergeben (13).

| Modifikation | Effekt | Hebel | Seltenheit | Pool |
|---|---|---|---|---|
| Fett | +4 Augen | Augen | Common | Alleinspieler |
| Goldrand | +8 Augen | Augen | Rare | Alleinspieler |
| Gezinkt | eine Stufe höher | Stichgewinn | Common | Alleinspieler |
| Doppelt gezinkt | zwei Stufen höher | Stichgewinn | Uncommon | Alleinspieler |
| Stempel | Ist ihre Farbe Trumpf: Grundwert +3 | Grundwert | Uncommon | Alleinspieler |
| Krone | Ist ihre Farbe Trumpf: Grundwert ×2 | Grundwert (multiplikativ) | Rare | Alleinspieler |
| Siegel | Ist ihre Farbe Trumpf: +1 Gewinnstufe, gewonnen wie verloren | Gewinnstufen | Uncommon | Alleinspieler |
| Hufeisen | Ist ihre Farbe Trumpf: Gewinn ab 56 Augen | Gewinnschwelle | Uncommon | Alleinspieler |
| Maßband | Ist ihre Farbe Trumpf: 80 bis 89 Augen sind Schneider | Schneider | Uncommon | Alleinspieler |
| Spitze | Gewinnt sie den letzten Stich: +1 Gewinnstufe | Gewinnstufen | Uncommon | Alleinspieler |
| Glückspfennig | Gewonnenes Spiel, dessen letzten Stich sie macht: Spielwert ×2 | Spielwert (multiplikativ) | Rare | Alleinspieler |
| Durchsichtig | Du siehst, wer sie hält | Zählen | Common | alle |
| Bierfleck (Fluch) | −5 Augen, dafür siehst du, wer sie hält; seit M7 beim Erhalt 3 Groschen | Augen (Fluch), Zählen, Groschen | Common | alle |
| Angeknackst (Fluch) | gewinnt nie; beim Erhalt 3 Groschen | Stichgewinn (Fluch), Groschen | Common | Alleinspieler |
| Schwarze Katze (Fluch) | Ist ihre Farbe Trumpf: Grundwert −3; beim Erhalt 3 Groschen | Grundwert (Fluch), Groschen | Common | Alleinspieler |
| Ramponiert (Fluch, M5) | darf nicht gedrückt werden; beim Erhalt 3 Groschen | Drücken (Fluch), Groschen | Common | alle |
| Bube ehrenhalber (M5) | im Farbspiel Trumpf unter dem Karo-Buben, keine Spitze (4.4) | Stichgewinn, Bedienpflicht | Rare | Alleinspieler |
| Unsichtbar | gewinnt nie, bedient aber | Stichgewinn | Uncommon | Nullspieler |
| Attrappe | gewinnt nie, und du siehst, wer sie hält | Stichgewinn, Zählen | Rare | Nullspieler |
| Bleigewicht | +10 Augen (Kurzschluss) | Augen | Common | Nullspieler |
| Schlusslicht | Gewonnenes Null, dessen letzten Stich sie macht: +2 Gewinnstufen | Gewinnstufen | Uncommon | Nullspieler |
| Aufgebrezelt | zwei Stufen höher: beim Gegner gut, bei dir gefährlich | Stichgewinn | Uncommon | Nullspieler |

Flüche, entschieden von Claude am 01.11.2026: Flüche sind Modifikationen mit einem klaren Nachteil, markiert mit `curse: true` (in der Sicht `CardModInfo.curse`, im Katalog `card_mods.<id>.curse`). Sie kommen über Zufallsziehungen und das Ereignis „Umgekipptes Bier“ ins Blatt. Seit M5 zahlen sie beim Erhalt Groschen (Auslöser `OnAttach`, Effekt `GainCoins`) statt eines Nebenvorteils, weil das klarer ist: Angeknackst und Schwarze Katze verloren ihren Ausgleich (+5 Augen, eine Stufe höher) und geben 3 Groschen, Ramponiert ist neu. Der Bierfleck behält den Fleck, an dem man die Karte erkennt, und zahlt nichts; er kommt über ein Ereignis, bei dem man für 1 Groschen die Karte wählt, Geld beim Erhalt wäre dort ein Gewinn. Auf einer Lusche kostet er nichts (Augen fallen nie unter 0). Die Beträge sind Platzhalter für `balance`.

Wirkt sie in diesem Spiel? Entschieden von Claude am 01.11.2026: Nach der Ansage sagt der Kern für jede sichtbare Karte, ob ihre bedingten Modifikationen in diesem Spiel wirken (`CardView.mod_status`, je Modifikation `Active`, `Inactive` oder `Open`; Berechnung `card_mod_status`). Bedingt sind Hooks im Spiel mit Bedingung (Stempel, Krone, Siegel, Hufeisen, Maßband, Schwarze Katze: „Ist ihre Farbe Trumpf“; Spitze, Glückspfennig, Schlusslicht: letzter Stich) und „Unter ehrenhalber“ (nur im Farbspiel). Es zählt nur, was der Spieler weiß: die Ansage, Flags und Zähler, die Stiche auf dem Tisch. „Letzter Stich“ bleibt offen, bis die Karte in einem früheren Stich gefallen ist oder der zehnte gespielt ist; Spitzen und das Ergebnis bleiben offen. Vor der Ansage und für Modifikationen ohne Bedingung („Fett“) gibt es keinen Status. Die Infokarte der Web-App zeigt je Modifikation eine Zeile: „Trumpf ist Rot: wirkt“, „Grand: wirkt nicht in diesem Spiel“, „…: entscheidet sich im Spiel“.

**M7: Flüche nur angekündigt** (entschieden über das Cockpit, 16). Zufallsziehungen ziehen nie einen Fluch: Stempelkissen, Kartengeber und alte Spielstände, deren Kartengeber die Modifikation erst bei der Wahl zieht, nehmen nur Modifikationen ohne `curse` (`Run::card_mod_pool`). Ein Fluch kommt nur über ein Ereignis, das ihn ankündigt: Der Ereignistext muss „Fluch“ sagen (beim Laden geprüft, sonst `ContentError`), und die Optionen nennen den Fluch, bevor der Spieler wählt. Jeder Fluch zahlt beim Erhalt Groschen, auch der Bierfleck jetzt 3 (vorher nichts; damit die Wahl „Fleck auf eine Lusche“ kein Gewinn wird, kostet sie beim Umgekippten Bier 4 statt 1 Groschen, netto wie vorher −1). Quellen sind „Umgekipptes Bier“ (Bierfleck) und das neue Ereignis „Der Falschspieler“ (13), das einen zufälligen Fluch aus dem Pool des Charakters zieht (`MarkCard { curse: true }`); so bleiben Angeknackst, Schwarze Katze und Ramponiert im Spiel. Tests: `core/src/run/m7_tests.rs` (300 Seeds je Charakter für Stempelkissen und Kartengeber, 80 ganze Bot-Runs mit einem Ereignis nach jeder Serie: jeder Fluch im Blatt kam von einem Ereignis mit „Fluch“ im Text und zahlte).

Wirkung im Run (3 × 3000 Runs, `--bot pimc:12:3`, Seeds 42 bis 44, Befehle wie in 5.2), vorher `main` be6aff5:

| Abschnitt erreicht | 2 | 3 | 4 | 5 | Run gewonnen |
|---|---|---|---|---|---|
| Alleinspieler vorher | 36,3 % | 14,5 % | 6,5 % | 3,0 % | 1,32 % |
| Alleinspieler M7 | 36,5 % | 14,8 % | 6,9 % | 3,2 % | 1,54 % |
| Nullspieler vorher | 37,0 % | 16,9 % | 8,1 % | 3,9 % | 1,61 % |
| Nullspieler M7 | 36,8 % | 16,6 % | 7,9 % | 3,9 % | 1,71 % |

Alles im Rauschen (bei 9000 Runs etwa ±0,5 Prozentpunkte auf Abschnitt 2, ±0,15 auf den Sieg). Der Bot nimmt jede Gratis-Karte, also jeden angekündigten Fluch, und kassiert dafür; dass er keine zufälligen Flüche mehr zieht, gleicht das in etwa aus. Ein Run trifft im Mittel 0,06 Falschspieler.

### 7.3 Karten hinzufügen und entfernen

- **Entfernen**: Luschen (7, 8, 9) rauswerfen ist der naheliegende Pfad. Jede entfernte Karte wird durch eine Blanko ersetzt (siehe 7.1). Blankos können später ebenfalls entfernt werden, dann wird das Blatt wirklich kleiner und es gibt weniger Stiche. *Offen: ob Stiche unter 10 sinnvoll sind.*
- **Hinzufügen**: Fünfter Bube (Trumpf, eigene Spitze), Doppel-Ass, „Sechs“ als neue Lusche mit Sondereffekt, Joker-Karten mit wählbarer Farbe beim Ausspielen.
- **Kopieren**: Eine bestehende Karte inklusive Modifikationen duplizieren.

### 7.4 Das Risiko sichtbar machen

Das UI zeigt bei jeder Blatt-Änderung die Wahrscheinlichkeit, dass die Karte in der eigenen Hand oder im Skat landet (12 von 32 Karten, also 37,5 % im Grundblatt). Spieler sollen dieses Risiko spüren, nicht nachrechnen müssen.

Der Kern rechnet das aus (`DealOdds`, Feld `deal_odds` der Sicht, wasm-Funktion `dealOdds`). Gemischt wird immer das ganze Blatt einschließlich Blankos und Talon, gegeben werden die ersten 32 Karten. Deshalb hat jede Karte dieselbe Chance, und sie hängt nur an der Blattgröße: 12 von n Karten. Eine entfernte Karte wird zur Blanko und ändert nichts, jede Karte über 32 senkt die Chance aller (33 Karten: 36,4 %).

---

## 8. Relikte und Gegenstände

### 8.1 Hausregeln (passive Relikte, Joker-Äquivalent)

Hausregeln sind regionale Skat-Varianten, als passive Modifikatoren. Fünf Plätze im Grundzustand.

| Hausregel | Effekt | Hebel |
|---|---|---|
| **Bockrunde** | Nach jedem verlorenen Spiel: nächstes Spiel zählt doppelt | Grundwert ×2 |
| **Kontra** | Gegner dürfen vor dem ersten Stich Kontra sagen (Spielwert ×2). Du darfst Re sagen (×4). KI sagt Kontra, wenn sie dein Blatt für schwach hält. | Mult, Risiko |
| **Spitze** | Letzter Stich mit einer Sieben gibt +1 Gewinnstufe | Gewinnstufen |
| **Schenken** | Haben die Gegner nach 5 Stichen unter 15 Augen, geben sie auf. Spiel gilt als Schneider. | Tempo, Schneider |
| **Ouvert-Bonus** | Ouvert zählt +2 statt +1 | Gewinnstufen |
| **Schieben** | In jedem Spiel einmal mehr einmischen (seit dem Einmischen, 01.11.2026; vorher einmal pro Serie neu geben lassen) | Spitzen, Risikostufe |
| **Jungfrau** | Gewinnst du mit 61 bis 70 Augen, zählt das Spiel trotzdem als Schneider | Schwelle |
| **Alte Dame** | Jede Dame zählt 5 Augen | Augen |
| **Tournee** | Jede Serie, die du mit Spielen übrig gewinnst, gibt +1 Grundwert für den Rest des Runs | Skalierung |
| **Durchmarsch** | Gewinnst du Schneider, zählt der Spielwert doppelt | Spielwert, Schneider |
| **Blindflug** (Null) | Gewonnenes Null aus der Hand: +1 Gewinnstufe | Gewinnstufen |
| **Kleinvieh** | Jede Sieben und Acht zählt 1 Auge | Augen |
| **Ein Auge zudrücken** | Gewinn ab 58 Augen | Schwelle |
| **Lehrgeld** | Jedes verlorene Spiel: +2 Grundwert für den Rest des Runs | Grundwert (Zähler) |
| **Bierdeckel** | Jedes gewonnene Handspiel: +1 Grundwert für den Rest des Runs | Grundwert (Zähler) |
| **Auf dem Trockenen** | Gegner nach drei Stichen ohne Auge: +1 Gewinnstufe | Gewinnstufen |
| **Schneiderbock** | Nach einem gewonnenen Schneider zählt das nächste Spiel doppelt | Grundwert ×2 |
| **Glückssträhne** | Ab dem dritten Sieg in Folge zählt jedes gewonnene Spiel doppelt, bis eines verloren geht | Spielwert ×2 |
| **Nullachtfuffzehn** (Null) | Jedes Null: Grundwert +8, gewonnen wie verloren | Grundwert |
| **Doppelter Boden** (Null) | Gewonnenes Null mit Skat: +1 Gewinnstufe | Gewinnstufen |
| **Dicke Sau** (Null) | Jedes Daus zählt 16 Augen, die Gegner kommen schneller in den Kurzschluss | Augen |
| **Glashaus** (Null) | Gewonnener Null Ouvert zählt doppelt | Spielwert ×2 |
| **Kerbholz** (Null) | Jedes gewonnene Null: +2 Grundwert für den Rest des Runs | Grundwert (Zähler) |
| **Skatpfleger** (M8) | Die Augen der gedrückten Karten zählen doppelt | Augen |
| **Altes Blatt** (M8) | Jede gedrückte Karte gibt bis zum Ende der Serie +1 Grundwert | Grundwert (Zähler der Serie) |
| **Allrounder** (M8) | Je verschiedener Spielart in einer Serie +3 Grundwert | Grundwert (Zähler der Serie) |
| **Stammfarbe** (M8) | Dreimal hintereinander dieselbe Trumpffarbe: ×1,5 | Spielwert ×1,5 |
| **Wechselbalg** (M8) | +1 Stufe, wenn du eine andere Spielart ansagst als im Spiel davor | Gewinnstufen |
| **Bubenparade** (M8) | Jeder Unter, den du ausspielst: +1 Grundwert | Grundwert |
| **Luschenkönig** (M8) | Gewinnt eine Sieben einen Stich: +1 Stufe | Gewinnstufen |
| **Stichfest** (M8) | Dein erster Stich zählt für die Schneider-Grenze doppelt | Schneider |

Geändert am 01.11.2026 nach der M2-Messung (`docs/balance/2026-11-01-m2-zielwerte.md`), weil vier Hausregeln keinen messbaren Einfluss hatten (Designsäule 1): Spitze verlangte die Karo-7 und kam praktisch nie vor, Jungfrau lag knapp unter 5 % Einfluss, Durchmarsch hing an Schwarz (0,8 % der Spiele), Schieben war erwartungsneutral. Nachmessung mit `skat_sim --runs 3000 --no-shop --relic <id>` (Seeds 42 bis 44, Abschnitt 2 erreicht, ohne Hausregel 24,4 %): Durchmarsch +11 %, Schieben +16 %, Jungfrau +6 %, Spitze weiter unter 1 %. Preise unverändert.

**Erweitert in M4 auf 31 Hausregeln und Figuren** (14 neu, entschieden von Claude am 01.11.2026). Vorgaben: jede an genau einem Hebel, fünf nur für den Nullspieler (Feld `characters`), vier multiplikativ fürs Spätspiel (Schneiderbock, Glückssträhne, Stammgast, Glashaus), drei Zähler, die über den Run wachsen (Lehrgeld, Bierdeckel, Kerbholz). Preise sind Platzhalter für `balance`. Einzeln gemessen (`skat_sim --seed 42 --runs 3000 --no-shop --relic <id>`, Abschnitt 2 / 3 / 4 erreicht; ohne Hausregel Alleinspieler 29,0 / 5,9 / 0,6 %, Nullspieler 27,0 / 7,3 / 1,6 %):

| Hausregel | Alleinspieler | Nullspieler | Anmerkung |
|---|---|---|---|
| Kleinvieh | 36,6 / 9,8 / 1,0 | 27,5 / 7,6 / 1,8 | mit den Neunen (+12 Augen) doppelt so stark wie die Alte Dame, deshalb nur Sieben und Acht |
| Ein Auge zudrücken | 33,7 / 8,3 / 1,2 | – | |
| Lehrgeld | 30,1 / 6,8 / 0,9 | 28,0 / 8,1 / 2,2 | mit +1 je Spiel kaum messbar, deshalb +2 |
| Bierdeckel | 29,0 / 5,8 / 0,6 | 27,1 / 7,3 / 1,7 | der Bot spielt fast nie Hand; für Menschen gedacht |
| Auf dem Trockenen | 29,8 / 6,3 / 0,8 | – | nach vier Stichen zu selten, deshalb drei |
| Schneiderbock | 29,1 / 6,6 / 1,1 | – | wirkt erst spät |
| Glückssträhne | 30,2 / 8,6 / 2,1 | 27,0 / 7,3 / 1,7 | Abschnitt 4 dreimal so oft |
| Der Schneider | 31,0 / 7,0 / 1,0 | – | |
| Der Stammgast | 29,0 / 9,0 / 2,8 | 27,0 / 8,2 / 2,6 | wirkt erst ab der vierten Serie, dann am stärksten |
| Nullachtfuffzehn | – | 28,2 / 8,0 / 2,0 | |
| Doppelter Boden | – | 28,4 / 8,4 / 2,3 | |
| Dicke Sau | – | 27,5 / 7,6 / 2,0 | |
| Glashaus | – | 27,0 / 7,3 / 1,7 | der Bot spielt Null Ouvert nur ohne offene Karte; Partner des Angebers |
| Kerbholz | – | 27,8 / 7,9 / 2,1 | mit +2 je Null |
| Mit Vieren (M5) | 29,3 / 6,1 / 0,7 | – | mit oder ohne 4 und mehr: +1 Gewinnstufe; Bedingung `MatadorsAtLeast` |
| Siebenschläfer (M5) | – | 28,4 / 8,6 / 2,4 | gewonnenes Null mit unterlaufener Sieben: Grundwert +5; Bedingung `SevenDucked` |
| Saubere Weste (M5) | – | 27,0 / 7,2 / 1,7 | gewonnenes Null, nie bedient: +2 Gewinnstufen; Bedingung `NeverFollowedSuit`. Der Bot erreicht das praktisch nie, für Menschen gedacht (Fehlfarben drücken) |
| Der Opa (M5) | 29,3 / 5,8 / 0,5 | – | der Bot spielt fast nie Hand |

M5-Werte gemessen mit `skat_sim --seed 42 --runs 3000 --no-shop --relic <id>` gegen 29,2 / 5,9 / 0,5 % (Alleinspieler) und 27,0 / 7,2 / 1,7 % (Nullspieler) ohne Hausregel.

Die Null-Hausregeln wirken im Bot schwächer, weil der Nullspieler-Bot vor allem an seinen Händen scheitert (43 % gewonnene Nullspiele). Im vollen Run mit Tresen (3 × 3000 Runs, Seeds 42 bis 44) ändert sich gegenüber den 17 Hausregeln aus M4 wenig: Alleinspieler Abschnitt 2 / 3 / 4 erreicht 33,1 / 11,3 / 3,3 % vorher, 33,0 / 11,5 / 3,2 % nachher, Nullspieler 28,6 / 8,6 / 2,7 % und 28,3 / 8,3 / 2,6 %. Der volle Pool verdünnt die starken Hausregeln (Bockrunde, Alte Dame), die Multiplikatoren sind Rare und kommen selten an den Tresen. Für die Kalibrierung der Kurve in M5 (`balance`) heißt das: Die Multiplikatoren existieren jetzt, ihre Seltenheit und ihre Preise sind die Stellschrauben. Umgesetzt in M5: Preise und Seltenheit nach Wirkung, siehe 12.

Gesperrt sind sieben der neuen (Stammgast, Lehrgeld, Glückssträhne, Schneiderbock, Auf dem Trockenen, Bierdeckel, Der Schneider), siehe 9.3. Die Null-Hausregeln sind frei, weil der Nullspieler selbst schon eine Freischaltung ist.

**Schieben: Entscheidung für „neu geben lassen“.** Zur Wahl standen zwei Varianten:

1. Nach dem Aufnehmen die zwei gedrückten Karten gegen zwei zufällige ungesehene Karten tauschen.
2. Einmal pro Serie vor der Ansage alle Hände und den Skat neu geben lassen, ohne dass ein Spiel verbraucht wird.

Variante 1 geht im geteilten Blatt nicht sauber: Ungesehen sind die Gegnerhände und der Talon. Der Talon ist im Grundblatt leer, also müssten die gedrückten Karten an die Gegner gehen, deren Blätter sich nach dem Geben ändern. Kommt eine Talonkarte ins Spiel, ändern sich die Karten im Spiel und damit die Spitzen-Zählung (4.4) und die aktiven Kartenmodifikationen. Offen bliebe auch, ob die neuen Karten zum Skat oder zur Hand gehören und welche zwölf Karten die Spitzen bestimmen, und im Handspiel gäbe es die Regel gar nicht. Variante 2 ist regelklar: Sie ist genau das Neugeben von „Neues Blatt“, nur einmal pro Serie und dauerhaft. Ihr Hebel sind Spitzen und Risikostufe: Der Spieler wirft ein Blatt ohne Spitzen und ohne genug Trümpfe weg und sucht eines, das ein Handspiel oder Schneider angesagt trägt, ohne wie beim Einpassen ein Spiel vom Limit zu verlieren. Anders als das alte Schieben ist das nicht erwartungsneutral, weil der Spieler sein Blatt kennt, bevor er entscheidet.

**Seit dem Einmischen (01.11.2026)** ist Neugeben eine Grundregel (5.1, zweimal pro Spiel). Schieben bleibt als Hausregel und gibt in jedem Spiel ein Einmischen mehr (drei statt zwei; Effekt `ExtraRedeal`, jede Quelle eins, liest `SeriesRedeal` alter Stände). Gewählt statt „einmal pro Serie“, weil ein dauerhaftes Relikt mit derselben Einheit wie die Grundregel (pro Spiel) am klarsten ist. Damit ist Schieben deutlich stärker als vorher (bis zu drei zusätzliche Blätter pro Serie statt eines); Preis 8 bleibt, bis `balance` misst.

**Welle 2a (M8, umgesetzt am 02.11.2026).** Acht Hausregeln aus der Abstimmung im Cockpit (alle „ja“), Texte wie abgestimmt, Daten in `house_rules.ron`, je ein eigenes Bierdeckel-Motiv. Auslegungen, entschieden von Claude am 02.11.2026:

- **Spielart** heißt bei Allrounder und Wechselbalg wie in der Skatordnung Farbspiel, Grand oder Null. Ein Wechsel der Trumpffarbe (Kreuz nach Pik) ist keine andere Spielart; dafür gibt es die Stammfarbe. Der Ramsch des Ramschers zählt bei keiner der acht mit und unterbricht nichts: Er wird nicht angesagt, und ein Zuschlag auf Grundwert oder Stufen würde dort nur den Verlust vergrößern.
- **Skatpfleger** (`skat_keeper`, Effekt `MultiplySkatPoints(2)` bei der Ansage): nur nach dem Aufnehmen, im Handspiel ist nichts gedrückt (auch der Tausch der Oma oder des Opas nicht). Die zwei Skatkarten zählen in diesem Spiel doppelte Augen (`Game::map_skat`), für das Gewinnen und gegen die eigene Schneider-Grenze. Den Gegnern fehlt nichts, ihr Schneider bleibt bei höchstens 30 ihrer Augen; das Blatt hat dann mehr als 120 Augen, wie mit der Alten Dame. Zeile „Augen +n“ und Ereignis `SkatPointsRaised`.
- **Altes Blatt** (`old_deck`, Zähler einer Serie): +1 Grundwert je gedrückter Karte, ab dem Spiel, in dem gedrückt wird, bis zum Ende der Serie, gewonnen wie verloren. Mit Skat in jedem Spiel +2, +4, +6. Nach der gewonnenen Serie auf null (`ResetCounter`, Ereignis `CounterReset`); eine verlorene Serie beendet den Run.
- **Allrounder** (`all_rounder`, Zähler einer Serie, `AddBaseValuePerCounter`): +3 Grundwert je Spielart, die in der Serie bisher angesagt wurde, das laufende Spiel eingeschlossen. Das erste Spiel einer Serie bringt also immer +3, mit Farbe, Grand und Null in einer Serie das dritte +9. Mehr als drei Spielarten gibt es nicht, also passt die Regel genau zu den 3 Spielen einer Serie.
- **Stammfarbe** (`home_suit`, Effekt `ScaleGameValue(1.5)`): ab dem dritten Farbspiel hintereinander mit derselben Trumpffarbe und bei jedem weiteren, **über Seriengrenzen hinweg** (sonst müsste jede Serie ganz in einer Farbe gespielt werden). Grand, Null oder eine andere Trumpffarbe beginnen die Folge neu, Einmischen zählt nicht. Der Faktor gilt gewonnen wie verloren (wie die Bockrunde), der Spielwert wird am Ende halb aufgerundet. Dafür gibt es jetzt gebrochene Spielwert-Faktoren: `Change::Scale`, und `ScoreBreakdown::multiplier` ist ein `Factor` (JSON Dezimalzahl, die Abrechnung zeigt „×1,5“).
- **Wechselbalg** (`changeling`): +1 Stufe, wenn das angesagte Spiel eine andere Spielart hat als das Spiel davor, über Seriengrenzen hinweg, gewonnen wie verloren. Das erste Spiel mit ihm hat kein Spiel davor.
- **Bubenparade** (`jack_parade`): „ausspielen“ im Sinn von „auf den Tisch legen“, nicht nur „einen Stich eröffnen“ (Glossar: `lead`): +1 Grundwert für jeden Unter, den der Alleinspieler spielt, gewonnen wie verloren, außer im Ramsch. Gewählt, weil die Regel so am Blatt hängt und nicht an einer Spielweise, die Unter zum Anspielen statt zum Stechen verbraucht. Karten, die beim Schenken nicht mehr gespielt werden, zählen nicht. Unter der Gegner zählen nicht.
- **Luschenkönig** (`blank_king`, Bedingung `TrickWinnerMatches`): +1 Stufe für jeden Stich, den eine Sieben des Alleinspielers gewinnt (eine Trumpf-Sieben, die sticht, oder eine Sieben, die keiner bedient oder sticht). Sieben der Gegner zählen nicht, der Text meint den Spieler. Nicht im Null (dort verliert sein Stich) und nicht im Ramsch.
- **Stichfest** (`watertight`, Bedingung `FirstTrickOfDeclarer`, Effekt `SchneiderBonusFromTrick`): Die Augen des ersten Stichs, den der Alleinspieler macht, zählen nur für die Schneider-Grenze ein zweites Mal. Gewinnt er, sind die Gegner mit höchstens 30 plus diesen Augen Schneider (`Thresholds::schneider_bonus`, gemessen an der verlierenden Seite wie Schneider selbst, 4.4). Gewinnen wird nicht leichter, ein angesagter Schneider aber schon; die eigene Schneider-Grenze des Alleinspielers bleibt. Zeile „zählt als Schneider“, wenn erst der Zuschlag das Spiel zum Schneider macht.
- **Rest abkürzen** (4.4) wartet, solange einer dieser Hooks noch feuern kann: bei der Bubenparade, bis der Spieler keinen Unter mehr hält, beim Luschenkönig, bis er keine Sieben mehr hält, beim Stichfest bis zu seinem ersten Stich.

Einzeln gemessen wie in M5 (`skat_sim --runs 500 --no-shop --bot pimc:12:3 --relic <id>`, Seeds 42 bis 44, Alleinspieler, 3 Spiele je Serie; ohne Relikt Abschnitt 2 / 3 / 4 erreicht 40,5 / 11,7 / 3,4 %), Bericht `docs/balance/2026-11-02-relikte-welle2.md`. Seltenheit und Preise sind Platzhalter für `balance`:

| Hausregel | Abschnitt 2 / 3 / 4 | Δ Abschnitt 3 | Seltenheit, Preis | Anmerkung |
|---|---|---|---|---|
| Allrounder | 47,1 / 15,5 / 4,8 | +3,7 | Uncommon, 8 | ein Farbspiel bringt schon +3, der Bot spielt oft auch Null |
| Altes Blatt | 45,6 / 15,1 / 4,7 | +3,4 | Uncommon, 7 | wirkt fast in jedem Spiel |
| Stichfest | 43,9 / 14,7 / 4,7 | +3,0 | Uncommon, 7 | jedes zehnte Spiel wird durch den Zuschlag Schneider |
| Skatpfleger | 43,1 / 14,2 / 3,6 | +2,5 | Uncommon, 7 | der Bot drückt Karten mit wenig Augen; Menschen holen mehr heraus |
| Bubenparade | 43,5 / 13,5 / 3,8 | +1,8 | Common, 6 | |
| Luschenkönig | 42,5 / 13,1 / 3,5 | +1,3 | Common, 5 | wie Auf dem Trockenen |
| Wechselbalg | 42,9 / 12,2 / 3,6 | +0,5 | Common, 5 | beim Bot −1,4 Punkte je Spiel: er wechselt meist ins Null, und die Stufe zählt verloren doppelt |
| Stammfarbe | 40,5 / 11,8 / 3,4 | +0,1 | Rare, 6 | für Menschen; der Bot spielt fast nie dreimal dieselbe Trumpffarbe |


### 8.2 Stammtisch-Figuren (passive Relikte mit Persönlichkeit)

| Figur | Effekt |
|---|---|
| **Der Kiebitz** | Zeigt dir vor der Ansage eine zufällige Karte eines Gegners |
| **Der Wirt** | Tresen-Preise −20 % |
| **Die Oma** | Einmal pro Serie darfst du einen gedrückten Buben zurücknehmen |
| **Der Schiedsrichter** | Gegner dürfen nicht schummeln (hebt Boss-Mechaniken teilweise auf) |
| **Der Angeber** | +1 Gewinnstufe bei Ouvert, aber du musst immer Ouvert spielen |
| **Der Zähler** | Zeigt dir laufend die Augen, die noch im Spiel sind |
| **Der Stammgast** | Zählt die Serien, die du mit ihm gewinnst. Ab der vierten zählt jedes gewonnene Spiel doppelt |
| **Der Schneider** | Nimmt großzügig Maß: Gewinnst du mit 80 bis 89 Augen, sind die Gegner Schneider |

Umgesetzt in M4 (`house_rules.ron`, `kind: Regular`, Preise 6 bis 8 als Platzhalter für `balance`). Auslegungen, entschieden von Claude am 01.11.2026:

- **Kiebitz** (`kibitzer`, Effekt `PeekDefenderCards(1)`): eine zufällige Karte eines Gegners und wer sie hält, gezogen mit dem Zufall des Spiels (das Geben bleibt gleich). Sie steht in der Sicht unter `revealed` und kommt als Ereignis `CardPeeked`, sonst nichts.
- **Wirt** (`landlord`, `ShopDiscount(20)`): gilt für Relikte, Verbrauchsgegenstände und die Neuauslage, gerundet, mindestens 1 Groschen. Verkauft wird weiter zum halben vollen Preis, also kein Gewinn durch Kaufen und Verkaufen.
- **Oma** (`granny`, `SeriesSkatSwap`): Ein gedrückter Bube lässt sich im echten Skat nicht sinnvoll zurücknehmen, weil der Spieler beim Drücken alles sieht. Umgesetzt als: Einmal pro Serie darf der Spieler in einem Handspiel vor seiner ersten Karte einen Buben aus dem Skat nehmen und dafür eine Karte drücken; das Spiel bleibt Hand. Ob ein Bube im Skat liegt, verrät die Oma (`can_take_from_skat`), seit M5 auch welcher (`skat_swap_cards`). Hebel: Hand plus Trumpf. Nur Alleinspieler.
- **Opa** (`grandpa`, M5, `SeriesSkatSwap` für Daus): wie die Oma, aber ein Daus aus dem Skat. Oma und Opa zusammen geben zwei Täusche pro Serie, je einen (4.4).
- **Schiedsrichter** (`referee`, `NoCheating`): schaltet die Boss-Regeln ab, die als Schummeln markiert sind: Zwillinge und Revolutionär.
- **Angeber** (`show_off`, `RequireOuvert` plus eine Stufe bei Ouvert): Für den Alleinspieler hieße „immer Ouvert“ immer Schwarz angesagt, das wäre Selbstmord. Deshalb nur im Pool des Nullspielers (Null Ouvert und Null Ouvert Hand).
- **Stammgast** (`regular_guest`, Zähler `regular` plus `MultiplyGameValue(2)` ab 4): Hebel Spielwert. Der Multiplikator fürs Spätspiel, wer ihn früh kauft, wartet zwei Abende. Nur gewonnene Spiele zählen doppelt.
- **Schneider** (`tailor`, `SchneiderFromDeclarerPoints(80, 89)`): Hebel Schneider, ein Wortspiel mit dem Handwerk. Nur Alleinspieler. Er kam statt des zuerst geplanten Opa, den der Kern in M4 nicht neben der Oma tragen konnte; seit M5 gibt es beide.
- **Zähler** (`card_counter`, `ShowCount`): zeigt Augen beider Seiten aus den Stichen (plus den Skat, sobald der Spieler ihn kennt) und die Summe der Karten, die der Spieler noch nicht gesehen hat. Der Talon zählt dazu, weil niemand weiß, welche Karten dort liegen; so verrät der Zähler nie mehr als ein perfekter Mitzähler weiß.

### 8.3 Verbrauchsgegenstände

Einmal nutzbare Karten, maximal zwei auf der Hand.

- **Frisches Deck**: Entferne eine Karte dauerhaft.
- **Stempelkissen**: Verpasse einer Karte eine Modifikation.
- **Ein Bier**: Einmal mehr einmischen, in einem beliebigen Spiel dieser Serie (seit 01.11.2026; vorher +1 Pass).
- **Runde Schnaps**: Beide Gegner spielen das nächste Spiel mit offenem Blatt.
- **Neues Blatt**: Sofort neu mischen und geben, ohne dass es ein Einmischen kostet.

Gewählt am 01.11.2026 mit dem Einmischen: Ein Bier und Kräuter wirken **für die Serie** (ein Vorrat, der erst nach den eigenen Einmischen des Spiels verbraucht wird und mit der Serie verfällt), wie vorher der Pass „in dieser Serie“; so bleibt der Text fast gleich und der Schnaps lässt sich früh trinken. Neues Blatt wirkt **sofort**, ohne ein Einmischen zu verbrauchen. Beide wirken damit fast gleich (ein Blatt mehr pro Kauf); Preise und Seltenheit (Bier 2, Kräuter 2, Neues Blatt 4) sind für `balance` neu zu prüfen.

**Schnäpse** (entschieden über das Cockpit am 01.11.2026, umgesetzt von Claude am 01.11.2026). Fünf Verbrauchsgegenstände mit `category: Schnapps` in `consumables.ron`, damit Tresen und Oberfläche sie als „Schnaps“ beschriften. Sie belegen dieselben zwei Plätze, kommen am Tresen wie die anderen und sind dort auch zu verkaufen. Seltenheit und Preise sind **Platzhalter für `balance`**, nicht gemessen: Kurzer 2 (Common), Doppelkorn 3 (Uncommon), Kräuter 2 (Common), Magenbitter 2 (Common), Kreide 3 (Uncommon).

| Schnaps | Id | Wirkung | Wann |
|---|---|---|---|
| **Kurzer** | `shot` | Zeigt die beiden Skatkarten. Die Wahl zwischen Skat und Hand bleibt offen. | Nur mit zehn Karten vor „Skat oder Hand“ (`Phase::Declaring`), einmal pro Spiel. Nicht gegen den Kiebitz-Mörder (Information ist aus). |
| **Doppelkorn** | `double_grain` | Das nächste angesagte Spiel zählt doppelt, gewonnen wie verloren: Zeile `Spielwert ×2` mit der Herkunft `Source::Consumable`. | Am Tresen oder vor der Ansage des liegenden Spiels. Verbraucht bei der nächsten Ansage wie „Letzte Runde“ und die Bockrunde; Einmischen verschenkt ihn nicht. Zwei Doppelkorn stapeln (×4). |
| **Kräuter** | `herbal` | Einmal mehr einmischen in dieser Serie (dieselbe Aktion wie „Ein Bier“, `ExtraRedeals(1)`). | Während einer Serie. |
| **Reue** | `regret` | Nimmt das letzte Einmischen zurück: Das Blatt davor liegt wieder auf dem Tisch, genau wie es war (Hände, Skat, Kiebitz, Kreide), und wird gespielt. Das Einmischen bleibt verbraucht. Neu am 01.11.2026, Common, 2 Groschen (Platzhalter). | Nur mit zehn Karten vor „Skat oder Hand“, gleich nach einem Einmischen (oder „Neues Blatt“) desselben Spiels; nur einmal zurück, nicht zweimal hintereinander (`NoRedealToUndo`). |
| **Magenbitter** | `bitters` | Geht das nächste angesagte Spiel verloren, zählt es einfach statt doppelt: Zeile `LossFactor = 1` (neuer Hebel, nur bei verlorenen Spielen in der Abrechnung). | Wie Doppelkorn. Verbraucht vom nächsten angesagten Spiel, gewonnen wie verloren; gewonnen wirkt er nicht. Ein zweiter, solange einer wartet, wird abgelehnt. |
| **Kreide** | `chalk` | Malt eine Karte der eigenen Hand bis zum Spielende in eine andere Farbe um. Die Karte ist für dieses Spiel diese Karte: Trumpf, Bedienen und Spitzen richten sich nach der neuen Farbe, Augen und Modifikationen bleiben. | Vor der Ansage (vor „Skat oder Hand“, beim Drücken oder bei der Ansage nach dem Skat). Nie ein Unter (er ist Trumpf durch seinen Rang) und keine Blanko. |

Auslegungen, entschieden von Claude am 01.11.2026:

- **Magenbitter wird vom nächsten angesagten Spiel verbraucht, nicht erst vom nächsten verlorenen.** So ist er eine Entscheidung vor einem riskanten Spiel (Grand Hand, Schneider angesagt) statt einer Versicherung, die man kauft und vergisst. Wer ihn trinkt und gewinnt, hat ihn umsonst getrunken.
- **Kreide ist vorübergehend** (Entscheidung des Nutzers): Das Blatt des Runs behält die gedruckte Karte, mit dem nächsten Geben ist sie wieder, was gedruckt ist. Einmischen (auch „Neues Blatt“) beendet das Spiel und damit auch die Kreide; „Reue“ bringt das alte Blatt samt Kreide zurück. Im Kern ersetzt `Game::chalk` die Karte überall, wo das Spiel sie führt (Hand, die zwölf Karten des Alleinspielers, die Karten im Spiel, das Blatt), Ereignis `CardChalked`. Doppelte Karten (zwei Grün-Zehnen) sind wie beim veränderbaren Blatt erlaubt: Bei gleicher Stärke behält die früher gespielte den Stich (4.4). Die Spitzen zählen über die umgemalte Karte, weil sie für dieses Spiel diese Karte ist; ein gemalter Trumpf-Daus kann damit eine Spitze mehr bringen.
- **Doppelkorn und Magenbitter** wirken auf die nächste Ansage im Sinne von `after_declaration`, wie „Letzte Runde“: Auch der Ramsch des Ramschers zählt als angesagtes Spiel. Während des Spiels (nach der Ansage) sind sie nicht nutzbar, damit niemand glaubt, sie wirkten noch aufs laufende Spiel.
- **Kurzer** gilt als Information: Gegen den Kiebitz-Mörder ist er nicht nutzbar. Die Wertvorschau zählt den gesehenen Skat als bekannt.
- **Bot** (`run/bot.rs`, einfache Regeln, nicht kalibriert): Kurzer direkt vor dem Aufnehmen des Skats (der Bot nutzt das Gesehene nicht), nach dem Skat Kreide auf die billigste Beikarte in die Trumpffarbe eines Farbspiels, dann Doppelkorn vor einem nicht schwachen und Magenbitter vor einem schwachen Spiel, nie vor einem Null; Kräuter wie Ein Bier. Reue nutzt der Bot nie (Kaufwert 0).
- **Reue** (Name „Reue“ gewählt: kurz und sagt, was er tut). Im Kern hält `ActiveGame::previous` das Spiel vor dem letzten Neugeben vollständig (es wird nicht neu gezogen, also deterministisch und auch nach Speichern und Laden gleich); `ConsumableAction::UndoRedeal` setzt es wieder ein, Ereignis `RedealUndone`. Was das Geben selbst ausgelöst hat (Zähler, Flaggen von `OnDeal`), bleibt; heute hat kein Inhalt so einen Hook. Der Zufallsstrom läuft weiter: Wer nach der Reue noch einmal einmischt, bekommt ein neues Blatt, nicht das zurückgenommene. Kaufwerte in `CONSUMABLE_VALUES` sind Platzhalter.

---

## 9. Charaktere

Jeder Charakter hat ein eigenes Startblatt, Startrelikte (oder einen Start-Schnaps) und genau eine eigene Regel; eine eigene Zielwertkurve ist möglich. Fünf Charaktere: der Alleinspieler (frei), der Nullspieler, der Grandspieler, der Zaungast und die Wirtin (je über eine Freischaltung, 9.3).

### 9.1 Der Alleinspieler (Standard)

Normales 32er-Blatt. Spielt Farbe, Grand und seit ADR 0012 auch Null. Null zählt seit dem 01.11.2026 für alle Charaktere nach der Formel mit Stilbonus (9.2); vorher galt beim Alleinspieler die Skatordnung (23, 35, 46, 59). Startrelikt: **Der Zähler**. Dieser Charakter ist der Maßstab für alles Balancing.

Charaktere sind Daten in `core/content/characters.ron`: Startblatt als Abweichung vom 32er-Blatt (`remove`, `add`, seit dem 01.11.2026 auch `mods`: Kartenmodifikationen ab Start), Startrelikte, Start-Verbrauchsgegenstände (`start_consumables`), optional eigene Zielwerte, Null-Wertung, die Tresen-Gewichte pro Spielart (`focus_weights`, siehe 9.2) und die eigene Regel (`rule`, `CharacterRule`). Jeder Charakter darf jedes Spiel ansagen (ADR 0012, ersetzt 0003); im Code unterscheidet kein Zweig mehr die Charaktere.

**Die eigene Regel** (`rule`) hat zwei Teile, beide Daten: `passive` und `hooks` sind dieselbe Hook-API wie bei Relikten (Abschnitt 8) und wirken wie ein Relikt, das der Charakter immer hat, ohne Platz zu belegen; in der Wertung steht die Zeile mit der Quelle `Source::Character`. `passes`, `relic_slots` und `payout_factor` ändern die Run-Struktur aus `run.ron`: Pässe pro Serie und Relikt-Plätze als Zuschlag (negativ: weniger, nie unter 0), die Groschen einer gewonnenen Serie als Faktor (`Payout::bonus` ist der Zuschlag). Die Einsatz-Leiter (5.4) gilt pro Charakter von selbst (`MetaState::stakes` ist nach Charakter geschlüsselt).

### 9.2 Der Nullspieler

Seit ADR 0012 ein Null-Spezialist, der auch Farbe und Grand ansagen darf (siehe „Null für alle“ am Ende dieses Abschnitts). Ursprünglicher Entwurf (ADR 0003): Spielt ausschließlich Null. Rangfolge 7, 8, 9, 10, B, D, K, A ohne Trumpf. Ziel: keinen einzigen Stich machen.

**Die Formel muss hier neu gebaut werden**, denn Null hat im echten Skat feste Werte (23, 35, 46, 59) statt Grundwert × Stufen. Vorschlag:

```
Spielwert = 23 × Risikostufen × Stilbonus

Risikostufen: Normal 1, Hand 2, Ouvert 3, Ouvert Hand 4
Stilbonus:    +1 je Stich, den du mit einer 7 „unterlaufen“ hast
              +2 wenn du nie bedienen musstest
              +3 „Kurzschluss“: Gegner haben nach 5 Stichen alle Augen
```

Das Startblatt des Nullspielers hat keine Buben, dafür vier zusätzliche Siebener. Seine Upgrades drehen die Logik um: Karten werden „leiser“ statt stärker, Luschen sind Gold, Asse sind Flüche. Der Tresen bietet andere Modifikationen (z. B. „Unsichtbar“: diese Karte kann nie einen Stich gewinnen).

Der Nullspieler wird nach dem Alleinspieler gebaut und erst freigeschaltet, wenn ein Run mit dem Alleinspieler gewonnen wurde.

**Umgesetzt in M4.** Die Ansage ist ein Enum (`Declaration`: Farbspiel, Grand, Null, Ramsch), Null hat eigene Risikostufen (`NullRisk`: Normal, Hand, Ouvert, Ouvert Hand). Ungültige Kombinationen wie Null mit Schneider angesagt gibt es nicht; im JSON bleibt die Form `{game, risk}`. Null Ouvert ohne Hand (46) ist damit erstmals darstellbar. Ein Null endet wie in der Skatordnung mit dem ersten Stich des Alleinspielers. Bei Null Ouvert sehen die Gegner die Hand ab dem ersten Ausspielen. Startrelikt: **Der Kiebitz**. Pool: Hausregeln, die für Null wirken (Bockrunde, Kontra, Ouvert-Bonus auch für Null Ouvert, Schieben, Tournee), die Null-Hausregel „Blindflug“, die Figuren außer der Oma, der Angeber nur hier, die Modifikation „Unsichtbar“ (`NeverWins`: kann nie stechen, bedient aber). Bosse ohne Wirkung auf Null (Oma, Schwarzer Peter) zieht der Nullspieler nicht.

**Null-Wertung, entschieden am 01.11.2026 mit dem Simulator: die Formel.** Beide Varianten sind Daten (`null_scoring` in `characters.ron`) und laufen durch dieselbe Formel Grundwert × Gewinnstufen, damit Relikte auf Grundwert, Stufen und Spielwert auch für Null wirken:

- `Fixed`: Grundwert 23, 35, 46 oder 59 (Skatordnung), eine Stufe.
- `Formula`: Grundwert 23 × (Risikostufe + Stilbonus). Die Risikostufe zerfällt in Spiel 1, Hand +1, Ouvert +2. Stilbonus nur bei gewonnenem Spiel: +1 je Stich, in dem der Alleinspieler eine Sieben gespielt und den Stich nicht bekommen hat; +2, wenn er nie bedienen musste; +3 „Kurzschluss“, wenn die Gegner nach fünf Stichen mindestens 61 Augen haben. „Alle Augen“ aus dem Vorschlag passt nicht in 15 Karten, deshalb die Mehrheit. Verloren: −2 × 23 × Risikostufe, ohne Stilbonus.

Messung mit dem Nullspieler-Bot (`skat_sim --runs 1000 --character null [--null-scoring fixed]`, Seeds 42 bis 44, alles freigeschaltet), Alleinspieler zum Vergleich:

| Kennzahl | Alleinspieler | Null `Fixed` | Null `Formula` |
|---|---|---|---|
| Abschnitt 2 erreicht | 31,8 % | 7,0 % | 28,4 % |
| Abschnitt 3 erreicht | 10,8 % | 0,1 % | 8,4 % |
| Abschnitt 4 erreicht | 3,5 % | 0,0 % | 2,8 % |
| Serie 1-1 | 70,1 % | 45,4 % | 67,5 % |
| Gewonnene Spiele | 65,7 % | 42,1 % | 43,0 % |
| Punkte je Spiel | +1,3 | −20,3 | +22,2 |

Mit den festen Werten verliert der Nullspieler fast jeden Run: Ein gewonnenes Null bringt 23, ein verlorenes kostet 46, und im Null-Blatt mit acht Siebenern gewinnt der Bot nur 42 % seiner Nullspiele. Mit der Formel kommt er so weit wie der Alleinspieler, obwohl er seltener gewinnt: Der Stilbonus (vor allem unterlaufene Siebener, das Null-Blatt hat acht) hebt ein gewonnenes Null weit über 23, ein verlorenes kostet weiter nur 46 je Risikostufe. Dazu hat die Formel Raum zum Skalieren: Grundwert-Relikte (Tournee) wirken auf 23, Stufen-Relikte (Ouvert-Bonus, Blindflug, Angeber) auf eine Summe von 1 bis 4 plus Stil, statt wie bei `Fixed` mit einer einzigen Stufe jedes Mal den Spielwert zu verdoppeln. Der Nullspieler nutzt deshalb die Zielwerte des Alleinspielers (`targets: None`). Der Bot spielt ein Null, wenn höchstens vier Karten „offen“ sind (klassische Null-Regel: die k-te kleinste Karte einer Farbe ist sicher, wenn höchstens k−1 kleinere Karten draußen sind), ohne offene Karte Null Ouvert Hand.

**M5: eigene Zielwerte** (bis ADR 0012, 5.2, `targets` in `characters.ron`): Mit 8 Spielen pro Serie und Relikten (Bockrunde, Siebenschläfer, Lehrgeld) bringt ein gewonnenes Null meist 92 bis über 200 Punkte; mit Zielwerten nah an denen des Alleinspielers (36 / 46 / 66 in Abschnitt 1, 99 bis 198 in Abschnitt 4 und 5) gewann der Bot jede Serie zu 76 bis 86 % und kam in 3,8 % der Runs durch (Alleinspieler 1,4 %, je 2 × 1000 Runs). Der Bot spielt seit M5 bis zu fünf offene Karten (`NULL_MAX_EXPOSED` in `core/src/run/bot.rs`): mit 4 erreichte er Abschnitt 3 in 20,1 %, mit 5 in 25,1 % (vorherige M5-Messung, 3 × 600 Runs); er steht für einen guten Spieler.

**Null für alle (ADR 0012, umgesetzt in M6).** Jeder Charakter darf Farbspiel, Grand und Null ansagen, in der Reihenfolge der Skatordnung (ADR 0011): aus der Hand Null Hand und Null Ouvert Hand, nach dem Skat Null und Null Ouvert. Was die Charaktere unterscheidet, sind nur noch Daten:

- **Ansage**: Der Dialog zeigt jedem Charakter sechs Spielarten (Eichel, Grün, Rot, Schellen, Grand, Null); bei Null die Null-Stufen dieser Phase mit Wert (`DeclareOptions.null_risks`). `null_style` sagt, ob ein gewonnenes Null einen Stilbonus bekommt. Beide Charaktere werten Null mit der Formel oben (entschieden am 01.11.2026: eine Null-Wertung für alle, damit dasselbe Spiel überall gleich zählt; `Fixed` bleibt als Datenoption). Ramsch gibt es weiter nur beim Ramscher.
- **Der Nullspieler bleibt Spezialist**: Startblatt ohne Unter mit acht Siebenern, Kiebitz, Formel-Wertung, und am Tresen öfter Null-Relikte. Er darf auch Farbe und Grand; ohne Unter ist jedes seiner Trumpfspiele „ohne 4“.
- **Gemeinsamer Pool**: Jedes Relikt und jede Modifikation liegt bei jedem Charakter (die Null-Hausregeln, „Unsichtbar“, „Bleigewicht“, „Schlusslicht“, „Aufgebrezelt“, „Attrappe“ auch beim Alleinspieler). Das neue Feld `focus` (`GameFocus`: `All`, `Trump` für Farbspiel und Grand, `Null`) sagt, für welche Spielart ein Inhalt gemacht ist. `focus_weights` in `characters.ron` gewichtet damit Tresen, Stempelkissen und Ereignisse in Prozent der Seltenheit: Alleinspieler Trump 100, Null 50; Nullspieler Trump 50, Null 200; `All` immer 100. Kein Code-Zweig pro Charakter. Relikte für Trumpfspiele, die im Null Unsinn täten (Spitze, Schenken, Auf dem Trockenen), schließen Null in ihren Bedingungen aus (`Not(NullGame)`). Der Angeber verlangt Farbe oder Grand Ouvert Hand, sonst Null Ouvert.
- **Bot** (`core/src/run/bot.rs`): Aus der Hand spielt er Null Ouvert Hand ohne offene Karte, Farbe oder Grand Hand mit acht Trümpfen oder einer Ansage; sonst nimmt er den Skat auf. Mit zwölf Karten vergleicht er das Farbspiel oder den Grand der Heuristik mit Null (die zwei offensten Karten gedrückt, Null Ouvert ohne offene Karte) nach dem erwarteten Ergebnis ohne Relikte (`suit_expected_score`, `null_expected_score`; Gewinnquoten `SUIT_WIN_PERCENT`, `GRAND_WIN_PERCENT`, `NULL_WIN_PERCENT`, gemessen mit dem PIMC-Spieler). Verloren zählt doppelt. Ein Blatt, das für Farbe und Grand schwach ist und mehr als fünf offene Karten hat, wirft er weg. Unter dem Schieber (nur Hand) spielt er Null Hand nur mit höchstens zwei offenen Karten (`NULL_HAND_MAX_EXPOSED`), die er ohne Skat nicht loswird; beim Schwarzen Peter nie Null, weil ein gewonnenes Null nie Schneider ist und nichts zählt.
- **Alte Spielstände** laden weiter: Die neuen Felder (`focus`, `focus_weights`, `character_factors`) haben Standardwerte, der Charakter im Spielstand bleibt gültig. `RunError::NullNotAvailable` bleibt als Fehlercode stehen, kommt aber nicht mehr vor.

**Messung:** Vorläufig (3 × 3000 Runs, `--bot pimc:12:3`, vor dem Zusammenführen mit dem kurzen Abend): Der Nullspieler erreicht Abschnitt 2 in 34 % der Runs (vorher 37 %), Abschnitt 3 in 14 %. Der Alleinspieler spielt etwa jedes fünfte Spiel als Null. Die endgültige Messung folgt in der großen Testrunde (Entscheidung vom 01.11.2026: Messungen vorerst pausiert, später gesammelt).

### 9.3 Freischaltungen über Runs hinweg

Entschieden am 01.11.2026: Neben dem Nullspieler werden auch Relikte freigeschaltet.

- Ein neuer Spielstand kennt etwa die Hälfte der Hausregeln und Figuren. Die übrigen erscheinen erst nach einer Freischaltung im Tresen.
- Freischaltungen hängen an Taten am Tisch, nicht an Spielzeit: zum Beispiel einen Grand Schneider gewinnen, Abschnitt 3 erreichen, einen Boss ohne Einmischen schlagen.
- Jede Freischaltung ist an genau einer Stelle als Daten definiert (Bedingung und Belohnung) und wird im Simulator auf Erreichbarkeit geprüft.
- Gespeichert wird im Browser des Spielers: laufender Run, Freischaltungen, Statistik der Runs. Kein Konto, kein Server-Speicher.

**Umgesetzt in M4.** Freischaltungen stehen in `core/content/unlocks.ron` (Bedingung und Belohnung). Jedes Relikt und jeder Charakter, der dort als Belohnung steht, ist bis dahin gesperrt. Der Meta-Zustand (`MetaState`: verdiente Freischaltungen, Statistik) ist ein eigenes, serialisierbares Objekt neben dem Run; er lernt aus den Ereignissen des Runs (auch aus den bereinigten Spieler-Ereignissen) und erzeugt den Start eines Runs mit den gesperrten Relikten. Anfangs gesperrt sind 15 von 31 Hausregeln und Figuren (Durchmarsch, Tournee, Jungfrau, Ouvert-Bonus, Kontra, Wirt, Oma, Schiedsrichter, dazu aus M4 Stammgast, Lehrgeld, Glückssträhne, Schneiderbock, Auf dem Trockenen, Bierdeckel, Der Schneider) und der Nullspieler. Die sieben Hausregeln und Figuren nur für den Nullspieler sind nicht gesperrt.

Seit M5 kann eine Freischaltung auch eine Kartenmodifikation belohnen (`Reward::CardMod`): Bis dahin ziehen Stempelkissen und Ereignisse sie nicht zufällig. Noch nutzt kein Inhalt das. Die Sicht der Freischaltungen nennt jede Belohnung mit Art (Hausregel, Figur, Modifikation, Charakter), Name und Seltenheit (`reward_info`).

**Gewonnene Runs nur im ganzen Abend** (entschieden vom Nutzer am 01.11.2026): Freischaltungen, deren Bedingung ein gewonnener Run ist (`WinRun`, heute nur `tour_won`), zählen nur einen gewonnenen ganzen Abend, weil ein kurzer Abend viel leichter zu gewinnen ist (5.3). Alle anderen Bedingungen (Abschnitt erreichen, Spiele, Bosse, Serien, beendete Runs) zählen in beiden Längen. In den Daten: `WinRun(character: Some(Declarer), length: Full)`; ohne `length` gilt `Full`. Der Text der Freischaltung sagt es („Gewinne einen ganzen Abend mit dem Alleinspieler.“). Die Einsatz-Leiter (5.4) ist davon getrennt und steigt weiter je Länge. Meta-Stände speichern nur die verdienten Freischaltungen und laden unverändert; wer den Nullspieler schon über einen kurzen Abend freigeschaltet hat, behält ihn.

Erreichbarkeit (`skat_sim --seed 42 --runs 3000 --unlocks`: 20 Folgen mit frischem Meta-Zustand, der Bot spielt Alleinspieler und nach der Freischaltung jeden zweiten Run Nullspieler):

| Freischaltung | Bedingung | Belohnung | Runs bis dahin (Mittel) |
|---|---|---|---|
| `early_finish` | Serie mit drei Spielen übrig gewinnen | Jungfrau | 1,8 |
| `second_evening` | Abschnitt 2 erreichen | Der Wirt | 3,9 |
| `big_game` | Spiel mit mindestens 96 Punkten gewinnen | Kontra | 4,0 |
| `grand_schneider` | Grand Schneider gewinnen | Durchmarsch | 4,0 |
| `boss_without_pass` | Boss ohne Pass schlagen | Der Schiedsrichter | 6,7 |
| `third_evening` | Abschnitt 3 erreichen | Tournee | 8,0 |
| `hand_won` | Handspiel gewinnen | Ouvert-Bonus | 12,7 |
| `beat_grandma` | Boss Die Oma schlagen | Die Oma | 25,4 |
| `tour_won` | Run mit dem Alleinspieler gewinnen | Nullspieler | 1265 (nur 12 von 20 Folgen in 3000 Runs) |

Nach den M4-Inhalten (gleiche Messung, alle 16 Freischaltungen):

| Freischaltung | Bedingung | Belohnung | Runs bis dahin (Mittel) |
|---|---|---|---|
| `early_finish` | Serie mit drei Spielen übrig gewinnen | Jungfrau | 1,6 |
| `grand_schneider` | Grand Schneider gewinnen | Durchmarsch | 2,0 |
| `second_evening` | Abschnitt 2 erreichen | Der Wirt | 2,2 |
| `big_game` | Spiel mit mindestens 96 Punkten gewinnen | Kontra | 2,5 |
| `boss_without_pass` | Boss ohne Pass schlagen | Der Schiedsrichter | 4,0 |
| `apprenticeship` | Fünf Runs zu Ende spielen | Lehrgeld | 5,0 |
| `hearts_schneider` | Ein Spiel in Rot mit Schneider gewinnen | Der Schneider | 5,2 |
| `third_evening` | Abschnitt 3 erreichen | Tournee | 6,0 |
| `big_win` | Spiel mit mindestens 144 Punkten gewinnen | Glückssträhne | 6,8 |
| `beat_kontra_hans` | Boss Der Kontra-Hans schlagen | Auf dem Trockenen | 9,1 |
| `hand_won` | Handspiel gewinnen | Ouvert-Bonus | 15,9 |
| `beat_grandma` | Boss Die Oma schlagen | Die Oma | 18,5 |
| `beat_black_peter` | Boss Der Schwarze Peter schlagen | Schneiderbock | 25,3 |
| `fourth_evening` | Abschnitt 4 erreichen | Der Stammgast | 34,0 |
| `grand_hand` | Grand Hand gewinnen | Bierdeckel | 35,3 |
| `tour_won` | Run mit dem Alleinspieler gewinnen | Nullspieler | 1677 (8 von 20 Folgen in 3000 Runs) |

Seit den Serien mit 3 Spielen (01.11.2026): `early_finish` verlangt eine Serie gleich im ersten Spiel (`WinSeriesWithGamesLeft(2)`, mit drei übrigen wäre sie unerreichbar), `boss_without_pass` heißt „Ohne Einmischen“ (`BeatBoss(without_redeal: true)`, Id unverändert). Beide nicht neu gemessen.

Verworfen: „Schneider angesagt gewinnen“ für den Schneider (577 Runs, der Bot sagt fast nie an) und „Serie mit vier Spielen übrig“ für Auf dem Trockenen (1,7 Runs, in Serie 1-1 reicht ein Spiel).

**Freischaltungen der neuen Charaktere** (01.11.2026, zählen in beiden Run-Längen, noch nicht im Simulator gemessen):

| Freischaltung | Bedingung | Belohnung |
|---|---|---|
| `grand_master` („Großer Grand“) | Grand Schneider gewinnen, der mindestens 144 Punkte zählt (`WinGame(game: Grand, schneider: true, min_value: 144)`: zum Beispiel mit 4, Spiel 5, Schneider 6; bis 01.11.2026 96, im Testlauf nach rund drei Runs erreicht, zu leicht) | Der Grandspieler |
| `beat_kibitz_killer` („Blinde Kuh“) | Boss Der Kiebitz-Mörder schlagen | Der Zaungast |
| `full_till` („Volle Kasse“) | Nach einer gewonnenen Serie mindestens 20 Groschen haben (neue Bedingung `SeriesWonWithCoins(20)`, gemessen nach der Auszahlung) | Die Wirtin |

Ein einfacher Grand Schneider ist schon die Freischaltung `grand_schneider` (im Mittel nach 2 Runs, zu leicht); 96 Punkte verlangen eine Stufe mehr. Der Kiebitz-Mörder nimmt jede Information, wer ihn schlägt, verdient den Zaungast, der ohne ihn immer mitschaut. 20 Groschen verlangen Sparen über Zinsen und übrige Spiele.

Alle Freischaltungen sind erreichbar. Die letzte ist für den Bot fast unerreichbar, weil er Abschnitt 5 kaum schafft; für Menschen ist sie das Ziel des ersten Runs, wie in 9.2 entschieden.

**Kartenrückseiten**: Standard ist die Distel-Rückseite (nach Altenburger Vorbild, eigene Pixelgrafik). Weitere Rückseiten werden später über bestimmte Erfolge freischaltbar (Freischaltung als Kosmetik, ohne Spielwirkung). Die Web-App führt die Rückseiten als kleine Liste von Entwürfen (`web/src/ui/backs.ts`, `cardBack(className, design)`); die Wahl ist eine reine Anzeige-Einstellung und berührt den Kern nicht.

### 9.4 Der Grandspieler

Entschieden vom Nutzer am 01.11.2026. Lebt für den Grand.

- **Regel**: Ein Grand zählt eine Gewinnstufe mehr, ein Farbspiel eine weniger, gewonnen wie verloren. Daten: zwei `OnDeclare`-Hooks mit `AddLevels(1)` bei `GameIs(Grand)` und `AddLevels(-1)` bei `SuitGame`. Null und Ramsch bleiben unverändert. Nie unter eine Stufe: Die Wertung rechnet `Stufen = max(1, Gewinnstufen nach Regel + Σ Stufen-Zuschläge)` (4.2); ein Farbspiel hat nach der Skatordnung mindestens zwei Stufen (mit oder ohne 1, Spiel 1), nach dem Abzug also mindestens eine, und kommen weitere Abzüge dazu, hält die Formel bei 1.
- **Startblatt**: das 32er-Blatt, der Eichel-Daus (`CA`) mit Goldrand (`gold_rim`, +8 Augen). Ursprünglich war ein gezinkter Eichel-Unter geplant; der ist aber ohnehin der höchste Trumpf, die Modifikation hätte kaum gewirkt (geändert am 01.11.2026). Startrelikte: keine.
- **Tresen**: `focus_weights` Trump 120, Null 50.
- **Zielwerte**: die des Alleinspielers, noch nicht kalibriert (17).

### 9.5 Der Zaungast

Entschieden vom Nutzer am 01.11.2026 (als „Kiebitz“; der Name ist an die Figur „Der Kiebitz“ vergeben, deshalb Zaungast). Der Zaungast schaut mit.

- **Regel**: Er sieht das ganze Blatt des linken Gegners (Platz 1, spielt nach ihm), jedes Spiel von der Ausgabe an, dafür darf er nur einmal pro Spiel einmischen statt zweimal (seit 01.11.2026; vorher zwei Pässe weniger pro Serie; nie unter 0). Daten: `passive: [RevealDefenderHand(1)]`, `redeals: -1`. `RevealDefenderHand` ist eine Informations-Wirkung wie der Kiebitz: Der Kiebitz-Mörder schaltet sie ab. Die Karten stehen wie die des Kiebitz in `ActiveGame::peeked` (die Sicht nennt sie `RevealSource::Onlooker`, in Anzeige-Reihenfolge), deshalb kennen sie auch Wertvorschau und Bot. Hat er zusätzlich den Kiebitz, zieht dieser nur Karten, die er noch nicht sieht.
- **Anzeige**: Am linken Gegner liegt sein Blatt als kleiner Fächer mit Auge; ausgespielte Karten verschwinden daraus. Kein Flüster-Hinweis pro Karte.
- **Start**: Standardblatt, keine Startrelikte (die Regel ist die Information).
- **Zielwerte**: die des Alleinspielers, noch nicht kalibriert (17).

### 9.6 Die Wirtin

Entschieden vom Nutzer am 01.11.2026 (als „Wirt“; der Name ist an die Figur „Der Wirt“ vergeben, deshalb Wirtin). Rechnet gut.

- **Regel**: Jede gewonnene Serie zahlt doppelt so viele Groschen (Grundbetrag, übrige Spiele und Zinsen, die Verdopplung steht als eigene Zeile „Die Wirtin“ in der Auszahlung), dafür hat sie vier statt fünf Relikt-Plätze. Daten: `payout_factor: 2`, `relic_slots: -1`. Groschen aus Relikten nach der Serie (`OnSeriesWon`) verdoppeln sich nicht.
- **Start**: Standardblatt, keine Startrelikte, ein Kräuter (einmal mehr einmischen in einer Serie) im Platz für Verbrauchsgegenstände (`start_consumables`).
- **Zielwerte**: die des Alleinspielers, noch nicht kalibriert (17).

### 9.7 Spätere Kandidaten (nicht vor v1)

- **Der Ramscher**: Spielt Ramsch, gewinnt mit den wenigsten Augen. Eigenes Scoring.
- **Der Schieber**: Sieht immer den Skat vor der Ansage, aber alle Spiele sind Hand.

---

## 10. Gegner und KI

### 10.1 Anforderungen

- Die KI muss **legal** und **plausibel** spielen, nicht optimal. Spieler sollen Fehler der KI nicht als Bug erleben, aber auch nicht das Gefühl haben, gegen Zufall zu spielen.
- Die KI muss **zu zweit koordinieren**: Schmieren, wenn der Partner den Stich hat; stechen, wenn der Partner ihn nicht bekommt; Farbe ausspielen, in der der Alleinspieler blank ist.
- Die KI muss **deterministisch** sein bei gleichem Seed, damit Runs teilbar und Bugs reproduzierbar sind.
- Die KI muss im **Simulator** tausende Spiele pro Sekunde schaffen.

### 10.2 Ansatz

**Stufe 1 (v1): Regelbasierte Heuristik.** Prioritätsliste pro Situation (Ausspielen, Bedienen, Stechen, Abwerfen) mit Gegenspieler-Konventionen aus der Skat-Literatur. Schätzung: zwei bis drei Tage Arbeit, gut testbar.

**Stufe 2 (nach v1): Perfect Information Monte Carlo.** Für jede Entscheidung werden die unbekannten Karten mehrfach zufällig verteilt und jeder Zug simuliert. Deutlich stärker, in Rust schnell genug für Echtzeit. Die Stärke lässt sich über die Zahl der Samples skalieren.

Umgesetzt in M5 (`core/src/ai/`): `sample` verteilt die unbekannten Karten passend zu allem, was der Sitz weiß (Fehlfarben, Ouvert- und Partnerhand, der Skat nach dem Aufnehmen, beim Spieler Durchsichtig und Kiebitz); die Gegenspieler geben dem Alleinspieler dabei bevorzugt ein Blatt, das zu seiner Ansage passt. `solver` ist eine Double-Dummy-Suche (Alpha-Beta, Transpositionstabelle, gleichwertige Karten zusammengefasst). Farbspiel und Grand werden als Schwellenfragen gesucht (erreicht der Alleinspieler 61 Augen, danach Schneider), Null als „bekommt der Alleinspieler einen Stich“, Ramsch exakt in den letzten fünf Stichen. Hinter `depth` Stichen schätzt ein schnelles Ausspielen mit offenen Karten den Rest. Gleichstände entscheidet die Heuristik. Die Stichproben kommen aus einem eigenen Zufallsstrom (Seed des Runs, Spielnummer, Zahl der gespielten Karten, Sitz).

### 10.3 Schwierigkeit

Schwierigkeit steigt **nicht** primär über klügere KI, sondern über die Zielwertkurve und über Boss-Regeln. Die KI darf aber zwischen den Abschnitten von Stufe 1 auf Stufe 2 wechseln, mit mehr Samples je später im Run.

Stand M5 (`run.ron`, `opponents`): Abschnitt 1 Heuristik, Abschnitt 2 PIMC mit 6 Stichproben und Tiefe 2, Abschnitt 3 mit 8 und Tiefe 3, Abschnitt 4 mit 12, Abschnitt 5 mit 16 Stichproben (Tiefe 3). Messung: `docs/balance/2026-11-01-m5-pimc.md`.

### 10.4 Stammgäste

Entschieden am 01.11.2026 (Cockpit): Die normalen Gegner sind wechselnde Stammgäste mit Namen und Pixelgesicht, ohne eigene Regeln; Regeln bleiben den Bossen. Umgesetzt in M6 (`core/content/guests.ron`, 14 Gäste, im Code `Guest`):

- Zu Beginn jeder Serie setzen sich zwei verschiedene Gäste (Sitz 1 links, Sitz 2 rechts). Gezogen wird aus einem eigenen Zufallsstrom, der nur vom Seed des Runs und der Serie abhängt; kein anderer Strom rückt dadurch weiter, Simulator-Ergebnisse bleiben gleich. Die beiden Gäste der vorigen Serie setzen aus, so kommt nie dasselbe Paar zweimal hintereinander.
- In der Boss-Serie sitzt links der Boss, rechts ein Stammgast.
- Spielstände von vorher bekommen die Gäste der laufenden Serie aus dem Seed, ab der nächsten Serie wie oben.
- Jeder Gast hat drei kurze Sprüche (Begrüßung, eigener Stich, Stich des Spielers). Die Oberfläche zeigt selten einen davon als kleine Sprechblase in seinem Kasten: beim ersten Spiel einer Serie und höchstens etwa jedes zweite Spiel nach einem Stich ab 10 Augen. Nie zusammen mit einem Hinweis des Wirts.
- Kein Einfluss auf Spiel und KI.

---

## 11. Bosse

Bosse sind Gegnerpaare mit einer Sonderregel, die für die ganze Boss-Serie gilt. Die Regel wird vor der Serie angezeigt, damit der Spieler sein Blatt und seine Ansagen anpassen kann.

Der Boss eines Abschnitts wird beim Start des Abschnitts gezogen (Ereignis `BossAnnounced`) und ist ab dann in der Sicht lesbar (`upcoming_boss`, `next_series`). So kann der Spieler schon am Tresen vor Serie 2 auf den Boss hin einkaufen. Jeder Boss hat einen Zielwert-Faktor (`target_factor` in `bosses.ron`, Standard 1,0), der den Zielwert seiner Boss-Serie skaliert, weil die Bosse unterschiedlich schwer sind. Seit ADR 0012 kann ein Boss für einzelne Charaktere einen eigenen Faktor haben (`character_factors`, etwa `{NullPlayer: 0.4}`); die Sicht zeigt den Faktor des Charakters, der spielt. Die Werte setzt `balance`; messen lässt sich das mit `skat_sim --curve --boss <id>`.

| Boss | Regel |
|---|---|
| **Die Zwillinge** | Die Gegner sehen gegenseitig ihre Blätter |
| **Der Schieber** | Jedes Spiel ist ein Hand-Spiel |
| **Der Kontra-Hans** | Kontra ist immer aktiv, auch ohne Hausregel |
| **Die Oma** | Alle Buben werden vor dem Geben aus dem Blatt genommen |
| **Der Ramscher** | Ein Ramsch pro Serie ist Pflicht: Wer die meisten Augen hat, verliert |
| **Der Revolutionär** | Die Gegner tauschen nach dem 5. Stich ihre Blätter |
| **Der Schwarze Peter** | Gewinnen zählt nur bei Schneider |
| **Der Kiebitz-Mörder** | Alle Informations-Relikte sind deaktiviert |

Endboss nach Abschnitt 5: **Der Alte**, der jede der obigen Regeln nacheinander auflegt, eine pro Spiel.

**Umgesetzt in M4** (`bosses.ron`), Auslegungen von Claude am 01.11.2026:

- **Zwillinge** (`DefendersSeeEachOther`): Die Gegner-KI kennt die Hand des Partners. Gilt als Schummeln.
- **Ramscher** (`RamschOnce`): Das erste Spiel der Boss-Serie ist ein Ramsch: niemand sagt an, nicht passbar, Buben sind Trumpf wie im Grand, jeder spielt für sich, der Skat geht an den Gewinner des letzten Stichs. Eigene Wertung: Grundwert = Augen dessen mit den meisten Augen, eine Stufe, +1 „Jungfrau“, wenn ein Spieler keinen Stich hat. Hat der Spieler nicht die meisten Augen (bei Gleichstand verliert er), gewinnt er den Spielwert, sonst −2 × Spielwert. Seit M5 nennen Abrechnung (`ScoreBreakdown.seats`) und Spielende (`GameFinished.seats`) Augen und Stiche jedes Platzes, die Sicht die Stiche pro Platz (`seat_tricks`) und mit dem Zähler die Augen pro Platz (`count.seats`), damit die Oberfläche zeigen kann, wer die meisten Augen hat.
- **Revolutionär** (`SwapDefenderHands` nach dem 5. Stich): Die Gegner tauschen die Hände. Gilt als Schummeln.
- **Schwarzer Peter** (`MultiplyGameValue(0)`, wenn gewonnen ohne Schneider). Ein gewonnenes Null ist nie Schneider und zählt hier nichts. Bis ADR 0012 nur Alleinspieler, seitdem bei beiden Charakteren.
- **Kiebitz-Mörder** (`DisableInformation`): Kiebitz, Zähler und „Durchsichtig“ sind aus.
- **Der Alte** (`rotation` in `bosses.ron`): beim n-ten Blatt der Serie die Regel des n-ten Bosses der Liste (Schieber, Zwillinge, Kontra-Hans, Revolutionär, Ramscher, Oma, Schwarzer Peter, Kiebitz-Mörder). Gezählt werden gespielte Spiele plus Einmischen (`games_played + redeals_used`), wie vorher die Pässe mitzählten; mit nur 3 Spielen kämen sonst nur die ersten drei Regeln vor. Einmischen bringt also die nächste Regel, „Reue“ die des alten Blatts zurück. Bosse außerhalb des Charakter-Pools werden übersprungen. Er ist immer der Boss des letzten Abschnitts (`final_boss` in `run.ron`).

Zielwert-Faktoren nach der M4-Messung (`skat_sim --seed 42 --runs 3000 --curve --boss <id>`, P(Serie gewonnen) bei Zielwert 24 / 63 / 110, normale Serie 67 / 41 / 18,5):

| Boss | ohne Faktor | Faktor | Grund |
|---|---|---|---|
| Die Oma | 28 / 7,4 / 0,7 | **0,5** | ab Abschnitt 4 unschlagbar; ohne Buben gewinnt der Bot höchstens 35 % |
| Der Schwarze Peter | 40 / 19,6 / 6,6 | **0,6** | Obergrenze 41 %, später kaum schlagbar |
| Der Ramscher | 67 / 59 / 37,5 | **1,5** | der Ramsch bringt viel; 1,5 trifft die normale Kurve von 24 bis 110 |
| Der Alte | 71 / 55 / 35 (bei 180: 12,9 gegen 3,6) | **1,4** | als Endboss viel leichter als eine normale Serie |
| Kontra-Hans | 70 / 52 / 32,5 | 1,0 | unverändert (Wert von `balance`), im Spätspiel leichter: Vorschlag 1,4 an `balance` |
| Schieber, Zwillinge, Revolutionär, Kiebitz-Mörder | wie normal oder schwerer | 1,0 | Zwillinge, Revolutionär und Kiebitz-Mörder wirken über Information, die der Bot kaum nutzt; für ihn sind sie so schwer wie eine normale Serie |

**M5-Stand** (8 Spiele, Kurve aus 5.2, PIMC-Spieler; Boss-Serie gewonnen im Run, 3 × 3000 Runs, Abschnitt 1 / ab Abschnitt 2, normale Serien 76 bis 82 %):

| Boss | Faktor | Alleinspieler | Nullspieler | Grund |
|---|---|---|---|---|
| Die Oma | 0,5 → **0,3** | 44 / 50 | – | Obergrenze ohne Buben 42 % (Kurve, Zielwert 1); mit 0,4 im Run 37 bis 44 %. 0,3 legt sie an die Obergrenze |
| Der Schwarze Peter | 0,6 → **0,5** | 52 / 59 | – | Obergrenze 53 %; 0,5 hält sie bis 72 über 40 % (Kurve) |
| Der Schieber | 1,0 | 58 / 70 | nicht im Pool | siehe unten |
| Der Kontra-Hans | 1,0 → **1,5** | 66 / 74 | 70 / 74 | im Run sonst leichter als eine normale Serie (89 bis 91 % gegen 85 %) |
| Der Ramscher | 1,5 → **1,25** | 66 / 71 | 69 / 74 | flache Kurve (73 % bei 1, 65 % bei 72); mit 1,5 nur 60 bis 63 % |
| Die Zwillinge | 1,0 | 71 / 69 | 76 / 70 | |
| Der Revolutionär | 1,0 | 73 / 84 | 75 / 80 | wirkt über Information, die der Bot kaum nutzt |
| Der Kiebitz-Mörder | 1,0 | 74 / 81 | 77 / 77 | dito; für Menschen schwerer |
| Der Alte (Endboss) | 1,4 | – / 69 | – / 67 | als Endboss etwa so schwer wie eine Boss-Serie |

**Der Schieber und der Nullspieler**: Null Hand ohne Skat gewinnt der Nullspieler-Bot höchstens in 24 % der Boss-Serien (Kurve mit Zielwert 1, gegen 76 bis 80 % bei den anderen Bossen); vorher waren es 21 % gegen 67 %. Ein Zielwert-Faktor hilft da nicht. Entscheidung M5: Der Schieber steht nur noch im Pool des Alleinspielers (`characters: [Declarer]`, auch im Alten übersprungen). Vorschlag für später, sobald der Bot es kann: Unter dem Schieber sollte der Nullspieler-Bot nur Blätter mit höchstens ein bis zwei offenen Karten spielen (heute bis fünf, die er ohne Skat nicht loswird); danach neu messen und den Schieber mit eigenem Faktor für den Nullspieler zurückholen. Faktoren pro Charakter brauchen ein neues Feld (`core-dev`).

Seit ADR 0012 gibt es Faktoren pro Charakter (`character_factors`) und der Schieber ist wieder bei allen Charakteren; neu gemessen wird nach der Umstellung auf drei Spiele je Serie.

---

## 12. Wirtschaft

- **Währung**: Groschen. Thematisch Zehntelpfennig, wie am Kneipentisch.
- **Einnahmen**: Grundbetrag pro gewonnener Serie, Bonus pro übrigem Spiel, Zinsen auf Guthaben (gedeckelt), Verkauf von Relikten.
- **Ausgaben am Tresen**: Hausregeln, Figuren, Verbrauchsgegenstände, Kartenmodifikationen, Neuauslage.
- **Richtwerte (M2, kalibriert)**: Startkapital 4, Serie-Grundbetrag 3, Bonus pro übrigem Spiel 1, Zinsen 1 pro 5 Groschen bis maximal 5, Neuauslage 2. Hausregeln kosten 5 bis 8: starke 7 bis 8 (Bockrunde 8, Alte Dame 7, Schenken 7, Tournee 8), schwache und noch nicht wirksame 5. Figuren 6 bis 10, Verbrauchsgegenstände 2 bis 4. Messung und Begründung in `docs/balance/2026-11-01-m2-zielwerte.md`. Der Run-Bot hat damit beim Start von Abschnitt 2 im Mittel drei Hausregeln, ab Abschnitt 3 alle fünf Plätze belegt. Fürs Spätspiel fehlen noch Ausgaben (offen für M4 und M5).
- **M5 (PIMC-Spieler, 8 Spiele)**: Grundbeträge unverändert. Relikte kosten 4 bis 9, gestaffelt nach der gemessenen Wirkung allein ab Spielbeginn ohne Tresen (`skat_sim --runs 500 --no-shop --bot pimc:12:3 --relic <id>`, Seeds 42 bis 44, Veränderung von „Abschnitt 3 erreicht“, der stärkere Charakter zählt): ab +6,5 Prozentpunkten 9 (Bockrunde, Alte Dame, Kleinvieh; Stammgast wegen +6,6 auf Abschnitt 4), ab +4,5 8 (Schenken, Ein Auge zudrücken, Siebenschläfer, Schieben), ab +3 7, ab +1,5 6, darunter 5. Grenzfälle eine Stufe tiefer (Durchmarsch +4,5: 7; Kontra +1,6, beim Nullspieler −0,9: 5). Relikte, die vor allem Menschen nutzen (Hand, Ouvert, Information) oder nur gegen einzelne Bosse wirken, 4 bis 6. Hausregeln und Figuren folgen derselben Staffel. Tabelle in `docs/balance/2026-11-01-m5-v1.md`. Die Schnäpse (8.3) kosten 2 bis 3 als Platzhalter, ohne Messung.
- **Seltenheit folgt der Stärke (M5)**: Vorher lagen die drei stärksten Relikte beider Charaktere in Common und kamen in 20 bis 21 % aller Angebote. Bockrunde, Alte Dame, Kleinvieh und Siebenschläfer sind jetzt Uncommon, Spitze, Mit Vieren, Schiedsrichter und Saubere Weste dafür Common; Anteil der drei stärksten am Angebot 11 % (Alleinspieler) und 14 % (Nullspieler). Wirkung im Run (2 × 1000 Runs): Abschnitt 2 / Run gewonnen 36,8 / 1,45 % → 35,9 / 1,0 % (Alleinspieler), Nullspieler 47,5 / 3,8 → 46,5 / 3,9 %, im Rahmen des Rauschens. Gewählt, weil ein seltener Fund stärker sein soll als ein häufiger, nicht wegen der Zahl.
- Ein Run bringt dem Bot im Mittel 23 (Alleinspieler) bis 26 Groschen (Nullspieler), fast alles wird ausgegeben.
- **Serien mit 3 Spielen (01.11.2026)**: Der Bonus pro übrigem Spiel (1 Groschen) bringt jetzt 0 bis 2 statt bis zu 6. Grundbetrag, Zinsen und Preise unverändert; sie misst `balance` mit der vollen Kalibrierung neu. Kaufwerte des Bots (`CONSUMABLE_VALUES`): Neues Blatt 2, Ein Bier und Kräuter 1 (vorher 4, 3, 3), weil sie nur helfen, wenn die zwei Einmischen eines Spiels verbraucht sind: Der Bot mischt etwa jedes zweite Blatt ein (0,75 Einmischen je Spiel, 5.2), drei schwache Blätter hintereinander kommen etwa in jedem achten Spiel. Die Testrunde (`docs/balance/2026-11-01-testrunde.md`) hatte gezeigt, dass er sie mit 6 Pässen kaufte und kaum brauchte.
- **Welle 2a (02.11.2026, 8.1)**: Mit 3 Spielen je Serie fallen alle Wirkungen kleiner aus (Alte Dame +5,3 statt +7,1 auf „Abschnitt 3 erreicht“). Die Preise der acht neuen Hausregeln folgen der M5-Staffel auf drei Viertel verkleinert (ab +4,9 Punkten 9, ab +3,4 8, ab +2,3 7, ab +1,1 6, darunter 5; Grenzfälle eine Stufe tiefer), Relikte für Menschen 6. Kaufwerte des Bots in `RELIC_VALUES`, hochgerechnet auf den M5-Maßstab. Die Preise der älteren Relikte bleiben, bis `balance` alles mit 3 Spielen neu misst.
- **Spielwahl und Einsatz** (Bot, 01.11.2026): `suit_expected_score` rechnet mit der Gewinnschwelle des Spiels (Einsatz und Relikte): Über 61 schrumpft die gemessene Gewinnquote mit den übrigen Augen, `(120 − Schwelle) / 59` davon (grobes Modell, nicht gemessen). So wählt der Bot bei hohem Einsatz öfter Null, das keine Augen zählt.

---

## 13. Ereignisse (Rast)

Zwischen Serien kann statt des Tresens ein Ereignis kommen. Beispiele:

- **Stammtisch-Streit**: Wähle eine Hausregel, die für den Rest des Abends für alle gilt.
- **Der Kartengeber**: Lass eine zufällige Karte zinken oder zahle 3 Groschen und wähle selbst.
- **Alter Bekannter**: Tausche ein Relikt gegen ein zufälliges anderes.
- **Letzte Runde**: Spiel ein einziges Spiel um den doppelten Einsatz.

**Umgesetzt in M4** (`core/content/encounters.ron`, Begriff im Code `Encounter`, weil `Event` die Zustandsereignisse sind). Nach einer gewonnenen Serie (nicht nach der letzten) kommt mit der Wahrscheinlichkeit `encounter_chance` aus `run.ron` (Platzhalter 0,2 für `balance`) ein Ereignis statt des Tresens, gezogen nach `weight`. Es bietet Optionen an (`EncounterOption`, immer zuletzt „Weitergehen“), der Spieler wählt mit `RunAction::ChooseEncounter`, danach beginnt sofort die nächste Serie.

- **Stammtisch-Streit**: drei Hausregeln zur Wahl (freigeschaltet, nicht im Besitz, für den Charakter). Die gewählte gilt bis zum Ende des Abschnitts („Abend“) ohne Platz zu belegen.
- **Der Kartengeber**: „zinken“ heißt eine Kartenmodifikation anbringen. Kostenlos auf eine zufällige Karte, für 3 Groschen auf eine gewählte; die Modifikation ist zufällig aus dem Pool des Charakters, seit M7 nie ein Fluch. Seit M5 wird sie gezogen, wenn das Ereignis aufgeht, und beide Optionen nennen sie (`EncounterOption.card_mod`, in der Sicht `card_mod` mit Name und Fluch-Markierung). Der Spieler entscheidet also mit Wissen; zufällig ist nur noch die Karte.
- **Alter Bekannter**: Ein gewähltes eigenes Relikt wird gegen ein anderes getauscht. Seit M5 wird das neue Relikt gezogen, wenn das Ereignis aufgeht, und jede Option nennt es (`SwapRelic.new`, in der Sicht `new_relic`). Bei Spielständen von vorher ist es weiter zufällig, und die Option sagt das (`random_gain`).
- **Letzte Runde**: Das nächste angesagte Spiel zählt doppelt, gewonnen wie verloren (Spielwert ×2, Herkunft `Source::Encounter`). Wie die Bockrunde wird es bei der Ansage verbraucht, Einmischen verschenkt es nicht.

Gemessen (3 × 1000 Runs): etwa 0,45 Ereignisse pro Run; der Bot nimmt Hausregel und Gratis-Karte immer, tauscht schwache Relikte und lehnt die Letzte Runde ab.

**Erweitert in M4 um fünf Ereignisse** mit denselben vier Formen (entschieden von Claude am 01.11.2026):

- **Umgekipptes Bier** (`spilled_beer`, alle): Eine zufällige Karte bekommt den Fluch „Bierfleck“ (−5 Augen, dafür siehst du, wer sie hält), für 1 Groschen eine gewählte. Wer weitergeht, bleibt sauber; wer wählt, legt den Fleck auf eine Lusche und hat nur den Ausgleich. Seit M7 zahlt der Bierfleck beim Erhalt 3 Groschen, die gewählte Karte kostet 4 (siehe unten).
- **Gasthof zur Krone** (`crown_inn`, Alleinspieler): „Krone“ auf eine zufällige Karte, für 4 Groschen auf eine gewählte.
- **Der Taschenspieler** (`conjurer`, Nullspieler): „Attrappe“ auf eine zufällige Karte, für 3 Groschen auf eine gewählte.
- **Preisskat** (`prize_skat`, alle): Das nächste Spiel zählt dreifach, gewonnen wie verloren.
- **Gast aus Altenburg** (`guest_from_altenburg`, alle): wie der Stammtisch-Streit, aber nur zwei Hausregeln zur Wahl.

Damit trifft jeder Charakter acht Ereignisse (seit M7 neun, siehe unten). Der Bot nimmt jede Gratis-Karte, also auch den Bierfleck; den Preisskat lehnt er wie die Letzte Runde ab.

**M7: Flüche nur angekündigt** (7.2). Ein Ereignis, das einen Fluch geben kann, sagt „Fluch“ im Text (beim Laden geprüft), nennt den Fluch in beiden Optionen und zahlt: Jeder Fluch gibt beim Erhalt 3 Groschen (`OnAttach`, `GainCoins`). Der Kartengeber zieht nie einen Fluch.

- **Umgekipptes Bier** (`spilled_beer`, alle): „Eine zufällige Karte kriegt einen Bierfleck, einen Fluch … Du kassierst 3 Groschen dafür. Für 4 Groschen wischst du ihn auf eine Karte deiner Wahl.“
- **Der Falschspieler** (`card_sharp`, alle, neu): zahlt 3 Groschen, wenn er eine zufällige Karte „präparieren“ darf; der Fluch wird beim Aufgehen zufällig aus dem Pool des Charakters gezogen (Alleinspieler: Bierfleck, Angeknackst, Schwarze Katze, Ramponiert; Nullspieler: Bierfleck, Ramponiert) und in beiden Optionen genannt. Die Karte selbst wählen kostet 4 Groschen. Daten: `MarkCard(price: 4, curse: true)`.

---

## 14. Technik

### 14.1 Architektur

```
skat-roguelite/
├── core/            Rust-Crate „skat_core“: Regeln, Zustand, KI, Run-Logik, Content
│   ├── cards        Karten, Blatt, Modifikationen
│   ├── rules        Stichregeln, Trumpfordnung, Spielwert
│   ├── game         Zustandsautomat für ein Spiel
│   ├── run          Serie, Abschnitt, Tresen, Relikte, Ereignisse
│   ├── ai           Heuristik, später PIMC
│   └── content      Relikte, Bosse, Modifikationen als Daten (RON, eingebettet)
├── sim/             Rust-Binary: headless Simulator und Balancing-CLI
├── wasm/            Crate „skat_wasm“: wasm-bindgen-Schicht, exponiert skat_core dem Browser
└── web/             Web-App (TypeScript, Vite): Tisch, Hand, Tresen, Animation, Speichern
```

### 14.2 Grundsätze

- **Der Kern kennt das Frontend nicht.** Alles im Kern ist reine Logik, ohne Rendering, ohne Zeit, ohne Threads und ohne Dateizugriff zur Laufzeit, damit er als WebAssembly läuft. Web-App und Simulator sind Clients des Kerns.
- **Alles ist deterministisch.** Jeder Run hat einen Seed. Gleicher Seed, gleiche Eingaben, gleiches Ergebnis. Der Kern verwendet einen eigenen seeded RNG, nie den Systemzufall.
- **Der Kern ist ereignisbasiert.** Jede Zustandsänderung erzeugt ein Ereignis (Karte gespielt, Stich beendet, Spielwert berechnet). Die Web-App nutzt diese Ereignisse für Animation. Der Simulator ignoriert sie.
- **Content ist Daten.** Relikte, Modifikationen, Bosse und Ereignisse liegen in RON-Dateien und werden zur Laufzeit geladen. Effekte werden über eine kleine Menge von Hooks (`on_deal`, `on_declare`, `on_card_played`, `on_trick_end`, `on_score`) ausgedrückt.
- **Der Simulator ist Pflicht, nicht Luxus.** Jede neue Mechanik wird im Simulator über mindestens 10 000 Spiele gegen die Basis gemessen, bevor sie in die Oberfläche kommt.

### 14.3 Simulator-Metriken

- Gewinnquote pro Spielart und Risikostufe ohne Upgrades (Baseline)
- Durchschnittlicher Spielwert pro Spiel
- Wahrscheinlichkeit, eine Serie mit Zielwert X in N Spielen zu schaffen
- Einfluss jeder einzelnen Modifikation und jeder Hausregel auf die obigen Werte
- Laufzeit pro Spiel (Ziel: unter 1 ms mit Heuristik-KI)

---

## 15. Roadmap

| Meilenstein | Inhalt | Ergebnis |
|---|---|---|
| **M0** ✔ | Dieses Dokument | Entscheidungen festgehalten |
| **M1 Kern** ✔ | Karten, Geben, Stichregeln, Farbspiel und Grand, Augen, Spitzen, Spielwert, Heuristik-KI, Simulator-CLI | `cargo run -p sim` spielt 10 000 Partien und gibt Baseline-Statistik aus |
| **M2 Run-Loop** | Serie, Zielwerte, Pässe, Groschen, Tresen, 10 Hausregeln, 5 Modifikationen, 3 Bosse. Kern als WebAssembly, einfache Web-Oberfläche mit Platzhaltergrafik | Ein vollständiger Run ist im Browser spielbar, auch am Handy |
| **M3 Web-UI** | Deutsches Blatt als Vektorgrafik, Tisch, Hand, Stiche, Ansage-Dialog, Tresen, Run-Übersicht, Wertungs-Aufschlüsselung, Animation, Klänge, Speichern im Browser | Ein Run fühlt sich wie ein fertiges Spiel an |
| **M4 Content** | Nullspieler, restliche Bosse, Ereignisse, 30 Hausregeln, 20 Modifikationen, Endboss, freischaltbare Relikte | Inhaltlich vollständige v1 |
| **M5 Balance** | Simulator-gestützte Kalibrierung der Zielwertkurve, PIMC-KI, Seeds teilen, Statistik | Spielbare v1 |
| **M6 Feinschliff** | Oberfläche Stil A4 ohne Scrollen, große Abrechnung am Spielende, Null für alle (ADR 0012), geführter erster Abend, Stammgäste mit Namen und Pixelgesicht, leise Chiptune-Musik, Spielstand-Export | v1 zum Teilen |
| **M7 Langzeit** | Kurzer Abend (3 Abschnitte, umgesetzt 5.3), Endlos-Modus nach dem Endboss, Einsatz-Leiter (jeder Run-Sieg hebt die Gewinngrenze um ein Auge), angekündigte Flüche statt zufälliger (umgesetzt 7.2, 13) | Grund, wiederzukommen |
| **M8 Inhalte** | Schnäpse, drei neue Charaktere (Grandspieler, Zaungast, Wirtin), Bierdeckel-Grafiken, Infokarte, große Testrunde, Welle 2 (Relikte: Skat-Hebel, Bau-Richtungen, Spiel-Relikte; Welle 2a mit acht Hausregeln umgesetzt, 8.1), Welle 3 (Wandelkarte, Glasdaus, Heimweh, Stufe Legendär) | Ein Run mit vielen Bau-Richtungen |

Ab M1 wird jeder Meilenstein von einem Papierprototyp begleitet: echtes Skatblatt, Zettel für Relikte, zwei menschliche Gegner. Was auf Papier keinen Spaß macht, wird nicht gebaut.

---

## 16. Entschieden am 31.10.2026 (über das Cockpit)

| Frage | Entscheidung |
|---|---|
| Stiche unter 10 | Nein. Entfernte Karten werden durch Blankos ersetzt, es bleibt bei 10 Stichen. Siehe ADR 0005. |
| Gegner-Persönlichkeit | Standardgegner bleiben anonym. Nur Bosse haben Namen und Gesichter. Neu bewerten in M4. |
| Vorhand-Rotation | Rotation bleibt fest. Beeinflussbar nur über Relikte (z. B. „Ewige Vorhand“, teuer). |
| Name des Spiels | **Stammtisch**. Repo-Name bleibt vorerst `skat-roguelite`. |
| Server-Zielplattform | Browser. Erreichbar unter https://stammtisch-62-83-23-90.sslip.io hinter dem Caddy der Haushalt-App, siehe ADR 0007 und 0008. |

Entschieden am 01.11.2026 (über das Cockpit):

| Frage | Entscheidung |
|---|---|
| Frontend | Web-App mit Rust-Kern als WebAssembly statt Godot, ADR 0009. |
| Kartenbild | Deutsches Blatt, ADR 0010. |
| Zielgerät | Handy hochkant zuerst. |
| Sprache | Nur Deutsch für v1. Texte kommen aus einer Übersetzungsdatei. |
| Assets | CC0, CC-BY und SIL OFL, mit Namensnennung. Jede Quelle steht in `web/LICENSES.md` und im Abspann. |
| Schrift | VT323 (S4 aus der Schriftauswahl), SIL OFL |
| Fortschritt | Laufender Run, Statistik und Freischaltungen im Browser. Freigeschaltet werden der Nullspieler und Relikte (9.3). |
| Rückfragen | Nur bei Grundentscheidungen (ADRs). Alles andere entscheidet Claude und dokumentiert es. |
| Null-Wertung | Wird in M4 mit dem Simulator entschieden (9.2). Entschieden: Formel, siehe 9.2. |

Entschieden am 01.11.2026 nach dem ersten Spieltest:

| Frage | Entscheidung |
|---|---|
| Ablauf der Ansage | Erst Skat aufnehmen oder Hand spielen, dann drücken, dann ansagen, wie in der Skatordnung (ADR 0011). |
| Eckzeichen des Asses | „A“ statt „D“ für Daus. |
| Stil der Oberfläche | Stil A „Spielhölle“ in der Farbwelt A4 „Kupfer und Schiefer“: Pixelschriften, Schiefergrau mit Kupfer und Mint, harte Pixelschatten. Stilproben: https://claude.ai/artifact/6K2j1apvyPeMveR4pRmTid |
| Layout am Tisch | Kein Scrollen. Die Oberfläche füllt genau die sichtbare Höhe, auch als PWA ohne Adressleiste. Hand in zwei Reihen à fünf Karten. Die Wertung ist im Spiel nur eine kleine Zeile. |

Entschieden am 01.11.2026 über das Cockpit (Designfragen nach dem Spieltest):

| Frage | Entscheidung |
|---|---|
| Einstieg | Geführter erster Abend: Der Wirt erklärt in kurzen Sprechblasen, wenn etwas zum ersten Mal passiert. Dazu eine Regelkarte im Menü. Skat selbst wird nicht von Grund auf gelehrt. |
| Run-Länge | 5 Abschnitte als Standard, dazu ein „kurzer Abend“ mit 3 Abschnitten. Freischaltungen in beiden (M7). Umgesetzt: 5.3, Abschnitt 1 bis 3 mit dem Alten am Ende. |
| Schwierigkeit | Endlose Einsatz-Leiter: Jeder gewonnene Run hebt die Gewinngrenze des nächsten um ein Auge (61, 62, 63 …). Schneider rückt nicht mit. Wirkung wird vorher im Simulator gemessen (M7). Umgesetzt: 5.4, je Charakter und Länge getrennt. |
| Nach dem Endboss | Endlos-Modus: Zielwerte wachsen weiter, nur für den Highscore (M7). |
| Flüche | Nur angekündigt (z. B. bei Ereignissen) und mit Groschen bezahlt, nie aus Zufallsziehungen (M7). Umgesetzt: 7.2 und 13, Umgekipptes Bier und Der Falschspieler. |
| Normale Gegner | Wechselnde Stammgäste mit Namen und Pixelgesicht, ohne eigene Regeln. Regeln bleiben den Bossen (M6). |
| Relikte nur für Menschen | Behalten, selten und günstig (Mit Vieren, Der Opa, Saubere Weste, Bierdeckel, Glashaus, Ouvert-Bonus). |
| Musik | Leise Chiptune-Schleifen für Tisch, Tresen und Boss, getrennt von den Klängen abschaltbar (M6). |
| Wertung | Im Spiel eine kleine Zeile, nach dem letzten Stich eine große, überspringbare Abrechnung (M6). |
| Null | Für jeden ansagbar, Nullspieler bleibt als Spezialist (ADR 0012, M6). |
| Domain | sslip.io bleibt vorerst. Spielstand-Export als Text und dauerhafter Browserspeicher kommen trotzdem (M6). |
| KI-Tempo | Am Handy flüssig, kein Hintergrund-Thread nötig. |
| Planung | M6 Feinschliff und M7 Langzeit. Der Weckruf läuft bis M7. |

Entschieden am 01.11.2026 im Chat (Problemliste):

| Frage | Entscheidung |
|---|---|
| Grand | Bleibt 24. Gesteuert wird über Bosse und Relikte, nach der großen Testrunde neu bewerten. |
| Null-Wertung | Eine für alle: die Formel mit Stilbonus (9.2). |
| Endlos-Modus | Zurückgestellt, bis es Relikte mit echtem Wachstum gibt. Die Einsatz-Leiter kommt zuerst. |
| Charaktere | Zwei bis drei neue Charaktere als Startdecks mit je einer eigenen Regel (Konzepte im Cockpit unter Ideen). |
| Passen | Im Spieltest prüfen, ob es zu oft vorkommt. Ersetzt am 01.11.2026 durch Einmischen (siehe unten). |
| Freischaltungen | Run-Sieg-Freischaltungen nur im ganzen Abend. |
| Tempo | Simulator-Messungen und der große Browser-Test sind vorerst pausiert; danach eine große Testrunde. |

Entschieden am 01.11.2026 nach dem Spieltest (Nutzer, umgesetzt von Claude):

| Frage | Entscheidung |
|---|---|
| Passen | Ersetzt durch **Einmischen**: bis zu zweimal pro Spiel neu geben lassen, kostet kein Spiel; danach muss gespielt werden (5.1). Relikte und Schnäpse: Schieben +1 pro Spiel, Ein Bier und Kräuter +1 für die Serie, Neues Blatt sofort ohne Einmischen, Zaungast eins weniger pro Spiel, neuer Schnaps „Reue“ nimmt das letzte Einmischen zurück (8.1, 8.3, 9.5). |
| Serie | 3 Spiele je Serie gegen dieselben zwei Gäste, 2 Serien und eine Boss-Serie je Abschnitt (5.1). Zielwerte vorläufig neu gesetzt (5.2). |
| Ansagen nach dem Skat | Schneider, Schwarz angesagt und Ouvert auch nach dem Aufnehmen, ohne Hand-Stufe (ADR 0013, 4.3). Der Bot sagt nach dem Skat mit denselben Regeln an wie aus der Hand. |
| Freischaltungen | `early_finish` heißt jetzt „Serie gleich im ersten Spiel gewinnen“, `boss_without_pass` „Boss ohne Einmischen“, `grand_master` verlangt 144 statt 96 Punkte (9.3). |

## 17. Offen

- **Zielwerte für 3 Spiele kalibrieren** (01.11.2026, 5.2): Die Kurve ist eine grobe Setzung aus 2 × 500 Runs. Volle Messung mit 3 × 3000 Runs je Charakter, kurzer Abend, Boss-Faktoren (`--curve --boss`), Einsatz-Leiter. Der Nullspieler liegt darunter (Abschnitt 2: 35 % gegen 44 %). Ebenso Preise von Schieben, Ein Bier, Kräuter, Neues Blatt und Reue (8.1, 8.3), die Erreichbarkeit von `early_finish`, `boss_without_pass` und `grand_master` (`--unlocks`) und ob Einmischen beim Alten (Regel je Blatt) zu leicht macht.

- **Neue Charaktere kalibrieren** (01.11.2026, 9.4 bis 9.6): Grandspieler, Zaungast und Wirtin spielen mit den Zielwerten des Alleinspielers, ohne Simulator-Messung (Nutzerentscheidung: erst bauen, später gesammelt messen). Zu messen: `skat_sim --runs 3000 --bot pimc:12:3 --character grand|onlooker|landlady`, Abschnitt erreicht gegen den Alleinspieler, dann `targets` oder die Regel anpassen. Der Bot nutzt die Regeln nur teilweise: Er sieht das Blatt des Zaungasts in den PIMC-Stichproben, wählt aber Grand nicht bewusst öfter. Ebenso die Erreichbarkeit der drei Freischaltungen (`--unlocks`) messen.

- **Null-Formel**: entschieden in M4 (9.2). Offen bleibt das Feintuning der Stilboni mit Testspielen.
- **Zielwert-Faktoren pro Charakter**: erledigt mit ADR 0012 (`character_factors` in `bosses.ron`, 11). Schieber und Schwarzer Peter sind wieder bei beiden Charakteren. Offen: Die Oma bleibt beim Nullspieler draußen, weil sein Blatt keine Unter hat und sie ihm nichts nimmt (ADR 0012 sieht sie wieder bei jedem vor). Sobald er Unter ins Blatt bekommen kann, mit `--curve --boss grandma --character null` neu messen.
- **Null für alle mit Menschen prüfen** (ADR 0012): Die Gewinnquoten hinter der Bot-Wahl Farbe, Grand oder Null (`SUIT_WIN_PERCENT`, `GRAND_WIN_PERCENT`, `NULL_WIN_PERCENT` in `core/src/run/bot.rs`) sind Bot-Messungen ohne Relikte. Ob Menschen mit dem Alleinspieler öfter Null spielen und ob die Tresen-Gewichte (`focus_weights`) den Nullspieler klar genug vom Alleinspieler trennen, zeigt die Statistik echter Runs.
- **M5-Kurve mit Menschen prüfen**: Die Kurve (5.2) ist gegen den PIMC-Spieler gesetzt. Ein Mensch nutzt Hand, Ouvert und Informations-Relikte, die der Bot kaum spielt, und dürfte stärker sein. Testspiele und die Statistik der Web-App (Abschnitt erreicht) gegen die Bot-Zahlen halten.
- **Relikte ohne Bot-Wirkung**: Spitze, Mit Vieren, Schiedsrichter, Bierdeckel, Ouvert-Bonus, Oma, Opa, Glashaus, Blindflug, Saubere Weste und Zähler bewegen beim Bot „Abschnitt 3 erreicht“ um weniger als 5 % (relativ). Sie sind für Menschen gedacht; ob sie die 5-%-Regel erfüllen, zeigt erst die Statistik echter Runs.
- **Ereignis-Wahrscheinlichkeit, Figurenpreise**: Platzhalter, `balance`.
- **Welle 2a im vollen Run** (02.11.2026, 8.1): Preise und Seltenheit der acht neuen Hausregeln sind aus der Einzelmessung ohne Tresen gesetzt (`docs/balance/2026-11-02-relikte-welle2.md`), Platzhalter für `balance`. Offen: Wirkung im vollen Run mit Tresen (3 × 3000), beim Nullspieler, und ob der Bot den Skatpfleger beim Drücken nutzen soll (er drückt weiter Karten mit wenig Augen, ein Mensch drückt Zehnen). Stammfarbe, Wechselbalg und Allrounder hängen an der Spielwahl; der Bot wählt das Spiel ohne Blick auf die Relikte, ein Mensch kann gezielt wechseln oder bei einer Farbe bleiben. Zu entscheiden: ob der Wechselbalg nur bei gewonnenen Spielen zählt (jetzt wie jede Stufe auch verloren, wie der Text sagt; beim Bot kostet er so Punkte je Spiel).
- **Nullspieler über den kurzen Abend** (M7, 5.3): „Run gewinnen“ zählt auch im kurzen Abend, der Nullspieler ist damit nach rund 25 statt rund 1700 Runs (Heuristik-Bot) frei. Entschieden ist „Freischaltungen in beiden“; soll der Nullspieler ein Ziel des ganzen Abends bleiben, bekäme die Bedingung `WinRun` eine Länge (Daten in `unlocks.ron`). Ebenso „Abschnitt 4 erreichen“: nur im ganzen Abend möglich.
- **Weitere Kartenrückseiten** (9.3): Standard ist die Distel-Rückseite (nach Altenburger Vorbild, eigene Pixelgrafik). Weitere Rückseiten werden später über bestimmte Erfolge freischaltbar (Freischaltung als Kosmetik, ohne Spielwirkung). Offen: welche Erfolge. Die Bedingung gehört wie jede Freischaltung in den Kern (`unlocks.ron`, eine neue Belohnungsart für Kosmetik); die Web-App bietet nur die verdienten Rückseiten zur Wahl an.
- **Fluchbeträge** (M7, 7.2): alle Flüche zahlen 3 Groschen, die gewählte Karte kostet bei Bier und Falschspieler 4. Platzhalter für `balance`. Die Option „Zufällige Karte“ zeigt in der Web-App „gratis“, die 3 Groschen stehen nur im Text des Fluchs.
- **Einsatz-Leiter messen** (M7, 5.4): Die Wirkung je Auge ist nicht kalibriert (`skat_sim --runs 3000 --bot pimc:12:3 --stake N` für einige N). Offen auch, ob Null bei hohem Einsatz zu stark wird. Die Texte von „Ein Auge zudrücken“ und „Hufeisen“ sagen seit der Einsatz-Leiter „drei/fünf Augen weniger“.

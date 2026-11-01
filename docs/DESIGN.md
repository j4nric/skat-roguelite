# Stammtisch – Design-Dokument

Stand: 2026-11-01 · Status: Entwurf v0.3 · Sprache des Projekts: Deutsch (Code und Kommentare: Englisch) · Arbeitstitel bis 31.10.2026: „Skat Roguelike“

Dieses Dokument hält die Grundentscheidungen fest und beschreibt das Spiel so weit, dass ein erster spielbarer Prototyp daraus gebaut werden kann. Alles unter „Offen“ ist noch nicht entschieden. Alles andere gilt, bis es hier geändert wird.

---

## 1. Pitch

**Stammtisch: Skat als Roguelike-Deckbuilder.** Du bist immer der Alleinspieler. Über einen Run hinweg spielst du Skat gegen zwei KI-Gegenspieler, musst in jeder Runde einen Punkte-Zielwert erreichen und veränderst zwischen den Runden das Blatt, mit dem alle drei spielen. Die Spielwertformel des echten Skats (Grundwert × Gewinnstufen) ist die Score-Engine, Hausregeln sind die Relikte, die Kneipe ist der Shop.

Referenzen: Balatro (Score-Formel, Run-Struktur, Shop), Inscryption (Gegner als Figuren, Regelbrüche), Slay the Spire (Charaktere mit eigenem Kartenpool).

Darstellung: deutsches Blatt (Eichel, Grün, Rot, Schellen; Unter, Ober, Daus), siehe ADR 0010. Dieses Dokument verwendet weiter die französischen Begriffe, weil sie im Code und in der Skatordnung stehen. Sprache des Spiels: Deutsch. Zielgerät: Handy hochkant zuerst.

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
2. Risikostufe: *Normal*, *Hand* (Skat bleibt liegen), *Schneider angesagt*, *Schwarz angesagt*, *Ouvert* (offen spielen). Schneider, Schwarz und Ouvert ansagen geht nur beim Handspiel; nach dem Aufnehmen bleibt im Farbspiel und Grand nur *Normal*, beim Null *Normal* und *Ouvert*.

Jede Stufe erhöht die Gewinnstufen, aber ein Fehlschlag auf einer angesagten Stufe gilt als verloren. Das ersetzt den Push-your-luck-Charakter des Reizens durch eine einzige, klare Entscheidung.

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
- **Bockrunde** wird bei der Ansage verbraucht, nicht beim Geben. So verschenkt ein Pass den Bock nicht.
- **Schenken**: Die restlichen Stiche gehen an den Alleinspieler. Das Spiel ist damit Schneider, und Schwarz, falls die Gegner noch keinen Stich hatten.

Ergänzt in M5 (entschieden von Claude am 01.11.2026):

- **Bube ehrenhalber** (Kartenmodifikation `honorary_jack`, Effekt `HonoraryJack`): Im Farbspiel ist die Karte Trumpf und steht eine halbe Stufe unter dem Karo-Buben: Sie schlägt das Trumpf-Ass und alles darunter und verliert gegen jeden Buben, egal in welcher Reihenfolge gespielt wird. Sie bedient Trumpf und nicht mehr ihre gedruckte Farbe; wird Trumpf ausgespielt, muss sie bedient werden. Liegt sie auf einer Karte der Trumpffarbe, steigt diese Karte über das Ass. Mit Rangbonus („Gezinkt“) steigt sie ganze Stufen von dort: einmal gezinkt schlägt sie den Karo-Buben und verliert gegen den Herz-Buben. Ein gezinktes Trumpf-Ass erreicht weiter den Karo-Buben (Gleichstand, der frühere behält den Stich) und schlägt damit den Buben ehrenhalber. Zwei Buben ehrenhalber im selben Stich: Der frühere behält ihn. Die **Spitzen zählen sie nicht mit**: Gezählt wird weiter die gedruckte Trumpfordnung; ein Bube ehrenhalber ist keine zusätzliche Spitze, und eine Trumpfkarte mit der Modifikation zählt an ihrer gedruckten Stelle. Im Grand, im Null und im Ramsch bleibt die Karte, was gedruckt ist, ebenso ein echter Bube. Im Code: `rules::trump_order_key` (Stärke, dann 0 für den Buben ehrenhalber, 1 für gedruckte Trümpfe), `is_trump` und `follows_lead` kennen ihn; die Gegner-KI sieht über `trump_strength` die Stärke des Karo-Buben.
- **Ramponiert** (Fluch `battered`, Effekt `NoDiscard`): Die Karte darf nicht gedrückt werden, solange zwei andere Karten gedrückt werden können; sonst so wenige ramponierte wie möglich (`rules::valid_discard`, Fehler `DiscardForbidden`). Das gilt auch für die Karte, die beim Skattausch der Oma oder des Opas in den Skat geht.
- **Mehrere Skattausch-Figuren**: Jede Figur mit `SeriesSkatSwap` gewährt ihren eigenen Tausch, einmal pro Serie (Oma: ein Unter, Opa: ein Daus). Sie können im selben Handspiel nacheinander genutzt werden. Die Sicht nennt die Karten, die herauskommen können (`skat_swap_cards`), die erste nimmt `TakeFromSkat` ohne `take`.

---

## 5. Run-Struktur

```
Run (eine Tournee)
└── Abschnitt (ein Skatabend in einer Kneipe), 3 bis 5 pro Run
    ├── Runde 1: Serie      (Zielwert, bis zu N Spiele)
    ├── Runde 2: Serie      (höherer Zielwert)
    ├── Runde 3: Boss-Serie (Gegner mit Sonderregel)
    └── dazwischen: Tresen (Shop), Ereignisse, Rast
```

| Ebene | Skat-Begriff | Genre-Gegenstück | Dauer |
|---|---|---|---|
| Ein Spiel | Skat, Ansage, 10 Stiche, Abrechnung | Eine Hand | ~90 Sekunden |
| Eine Serie | 3 bis 5 Spiele mit Punkteziel | Ein Blind | 5 bis 8 Minuten |
| Ein Abschnitt | Skatabend in einer Kneipe | Ein Ante | 15 bis 20 Minuten |
| Der Run | Tournee durch 3 bis 5 Kneipen | Der Run | 45 bis 60 Minuten |

### 5.1 Die Serie

- Jede Serie hat einen **Zielwert** (siehe 5.2) und ein **Spiellimit** (in M2: 5 Spiele, davon bis zu 3 Pässe).
- Gewonnene Spiele addieren ihren Spielwert. Verlorene Spiele ziehen den doppelten Spielwert ab.
- Der Spieler darf ein Spiel **einpassen** (nicht spielen). Das kostet ein Spiel vom Limit, aber keine Punkte. Zahl der erlaubten Pässe ist begrenzt und durch Relikte erweiterbar.
- Wer das Limit erreicht, ohne den Zielwert zu schaffen, verliert den Run.
- Übrige Spiele am Ende einer gewonnenen Serie werden in Groschen umgewandelt (Zinsmechanik wie in Balatro).

### 5.2 Zielwert-Progression (M2, kalibriert)

Spiellimit 5, Pässe 3 pro Serie. Die Zielwerte sind Reizwerte.

| Abschnitt | Serie 1 | Serie 2 | Boss |
|---|---|---|---|
| 1 | 18 | 20 | 24 |
| 2 | 27 | 33 | 36 |
| 3 | 44 | 54 | 63 |
| 4 | 77 | 90 | 110 |
| 5 | 132 | 154 | 180 |

Kalibriert am 01.11.2026 mit dem Simulator gegen den Run-Bot, Messung und Begründung in `docs/balance/2026-11-01-m2-zielwerte.md`. Mit diesen Werten gewinnt der Bot Serie 1-1 in 70 % der Runs, 26 % erreichen Abschnitt 2, 7 % Abschnitt 3, 1 % Abschnitt 4. Das liegt nah an der Decke des Bots: Selbst mit Zielwert 1 überall erreicht er nur 31 % Abschnitt 2.

Zur Einordnung: 18 ist der kleinste Wert eines gewonnenen Spiels, Serie 1-1 heißt also „ein Spiel netto gewinnen“. Ein durchschnittlicher Grand ohne Upgrades liegt bei 48 bis 72 Punkten. Die Kurve wächst um den Faktor 1,6 bis 1,7 pro Abschnitt, weil keine M2-Hausregel multiplikativ skaliert. Der erste Entwurf (60 bis 20 000) setzte das voraus. Abschnitt 4 und 5 sind mit dem Bot kaum gemessen und gelten als Vorschlag. In M5 wird die Kurve mit PIMC-KI und Testspielen neu kalibriert. Für Menschen ist Abschnitt 1 bis dahin bewusst leicht.

---

## 6. Ein Spiel im Detail

1. **Geben.** Das aktuelle Blatt (32 Karten plus Modifikationen) wird gemischt. 10/10/10/2.
2. **Blatt ansehen.** Der Spieler sieht seine 10 Karten. Relikte, die Informationen geben (Kiebitz), wirken jetzt.
3. **Skat oder Hand.** Skat aufnehmen und 2 Karten drücken, oder Hand spielen. Gedrückte Karten zählen zu den Augen des Alleinspielers.
4. **Ansage.** Spielart und Risikostufe wählen (ADR 0011).
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

**Umgesetzt in M4: 20 Modifikationen, in M5: 22** (`card_mods.ron`, Preise Platzhalter für `balance`). Zufällige Modifikationen (Stempelkissen, Kartengeber) ziehen gleichverteilt aus dem Pool des Charakters, Flüche eingeschlossen, gesperrte nicht (Belohnung `CardMod` einer Freischaltung, 9.3). Ereignisse können eine bestimmte Modifikation vergeben (13).

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
| Bierfleck (Fluch) | −5 Augen, dafür siehst du, wer sie hält | Augen (Fluch), Zählen | Common | alle |
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
| **Schieben** | Einmal pro Serie vor der Ansage neu geben lassen (alle Hände und Skat), ohne dass ein Spiel verbraucht wird | Spitzen, Risikostufe |
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

Die Null-Hausregeln wirken im Bot schwächer, weil der Nullspieler-Bot vor allem an seinen Händen scheitert (43 % gewonnene Nullspiele). Im vollen Run mit Tresen (3 × 3000 Runs, Seeds 42 bis 44) ändert sich gegenüber den 17 Hausregeln aus M4 wenig: Alleinspieler Abschnitt 2 / 3 / 4 erreicht 33,1 / 11,3 / 3,3 % vorher, 33,0 / 11,5 / 3,2 % nachher, Nullspieler 28,6 / 8,6 / 2,7 % und 28,3 / 8,3 / 2,6 %. Der volle Pool verdünnt die starken Hausregeln (Bockrunde, Alte Dame), die Multiplikatoren sind Rare und kommen selten an den Tresen. Für die Kalibrierung der Kurve in M5 (`balance`) heißt das: Die Multiplikatoren existieren jetzt, ihre Seltenheit und ihre Preise sind die Stellschrauben.

Gesperrt sind sieben der neuen (Stammgast, Lehrgeld, Glückssträhne, Schneiderbock, Auf dem Trockenen, Bierdeckel, Der Schneider), siehe 9.3. Die Null-Hausregeln sind frei, weil der Nullspieler selbst schon eine Freischaltung ist.

**Schieben: Entscheidung für „neu geben lassen“.** Zur Wahl standen zwei Varianten:

1. Nach dem Aufnehmen die zwei gedrückten Karten gegen zwei zufällige ungesehene Karten tauschen.
2. Einmal pro Serie vor der Ansage alle Hände und den Skat neu geben lassen, ohne dass ein Spiel verbraucht wird.

Variante 1 geht im geteilten Blatt nicht sauber: Ungesehen sind die Gegnerhände und der Talon. Der Talon ist im Grundblatt leer, also müssten die gedrückten Karten an die Gegner gehen, deren Blätter sich nach dem Geben ändern. Kommt eine Talonkarte ins Spiel, ändern sich die Karten im Spiel und damit die Spitzen-Zählung (4.4) und die aktiven Kartenmodifikationen. Offen bliebe auch, ob die neuen Karten zum Skat oder zur Hand gehören und welche zwölf Karten die Spitzen bestimmen, und im Handspiel gäbe es die Regel gar nicht. Variante 2 ist regelklar: Sie ist genau das Neugeben von „Neues Blatt“, nur einmal pro Serie und dauerhaft. Ihr Hebel sind Spitzen und Risikostufe: Der Spieler wirft ein Blatt ohne Spitzen und ohne genug Trümpfe weg und sucht eines, das ein Handspiel oder Schneider angesagt trägt, ohne wie beim Einpassen ein Spiel vom Limit zu verlieren. Anders als das alte Schieben ist das nicht erwartungsneutral, weil der Spieler sein Blatt kennt, bevor er entscheidet.

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
- **Ein Bier**: +1 Pass in dieser Serie.
- **Runde Schnaps**: Beide Gegner spielen das nächste Spiel mit offenem Blatt.
- **Neues Blatt**: Mischen und neu geben, ohne dass ein Spiel verbraucht wird.

---

## 9. Charaktere

Jeder Charakter hat ein eigenes Startblatt, ein Startrelikt und eigene Zielwertkurve.

### 9.1 Der Alleinspieler (Standard)

Normales 32er-Blatt. Spielt Farbe und Grand. Startrelikt: **Der Zähler**. Dieser Charakter ist der Maßstab für alles Balancing.

Charaktere sind Daten in `core/content/characters.ron` (Startblatt als Abweichung vom 32er-Blatt, Startrelikte, optional eigene Zielwerte, Null-Wertung). Welche Spiele ein Charakter ansagen darf, steht im Code (`Character::Declarer`, `Character::NullPlayer`).

### 9.2 Der Nullspieler

Spielt ausschließlich Null. Rangfolge 7, 8, 9, 10, B, D, K, A ohne Trumpf. Ziel: keinen einzigen Stich machen.

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

### 9.3 Freischaltungen über Runs hinweg

Entschieden am 01.11.2026: Neben dem Nullspieler werden auch Relikte freigeschaltet.

- Ein neuer Spielstand kennt etwa die Hälfte der Hausregeln und Figuren. Die übrigen erscheinen erst nach einer Freischaltung im Tresen.
- Freischaltungen hängen an Taten am Tisch, nicht an Spielzeit: zum Beispiel einen Grand Schneider gewinnen, Abschnitt 3 erreichen, einen Boss ohne Pass schlagen.
- Jede Freischaltung ist an genau einer Stelle als Daten definiert (Bedingung und Belohnung) und wird im Simulator auf Erreichbarkeit geprüft.
- Gespeichert wird im Browser des Spielers: laufender Run, Freischaltungen, Statistik der Runs. Kein Konto, kein Server-Speicher.

**Umgesetzt in M4.** Freischaltungen stehen in `core/content/unlocks.ron` (Bedingung und Belohnung). Jedes Relikt und jeder Charakter, der dort als Belohnung steht, ist bis dahin gesperrt. Der Meta-Zustand (`MetaState`: verdiente Freischaltungen, Statistik) ist ein eigenes, serialisierbares Objekt neben dem Run; er lernt aus den Ereignissen des Runs (auch aus den bereinigten Spieler-Ereignissen) und erzeugt den Start eines Runs mit den gesperrten Relikten. Anfangs gesperrt sind 15 von 31 Hausregeln und Figuren (Durchmarsch, Tournee, Jungfrau, Ouvert-Bonus, Kontra, Wirt, Oma, Schiedsrichter, dazu aus M4 Stammgast, Lehrgeld, Glückssträhne, Schneiderbock, Auf dem Trockenen, Bierdeckel, Der Schneider) und der Nullspieler. Die sieben Hausregeln und Figuren nur für den Nullspieler sind nicht gesperrt.

Seit M5 kann eine Freischaltung auch eine Kartenmodifikation belohnen (`Reward::CardMod`): Bis dahin ziehen Stempelkissen und Ereignisse sie nicht zufällig. Noch nutzt kein Inhalt das. Die Sicht der Freischaltungen nennt jede Belohnung mit Art (Hausregel, Figur, Modifikation, Charakter), Name und Seltenheit (`reward_info`).

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

Verworfen: „Schneider angesagt gewinnen“ für den Schneider (577 Runs, der Bot sagt fast nie an) und „Serie mit vier Spielen übrig“ für Auf dem Trockenen (1,7 Runs, in Serie 1-1 reicht ein Spiel).

Alle Freischaltungen sind erreichbar. Die letzte ist für den Bot fast unerreichbar, weil er Abschnitt 5 kaum schafft; für Menschen ist sie das Ziel des ersten Runs, wie in 9.2 entschieden.

### 9.4 Spätere Kandidaten (nicht vor v1)

- **Der Grandspieler**: Nur Grand, Grundwert 24, aber Spitzen zählen doppelt.
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

---

## 11. Bosse

Bosse sind Gegnerpaare mit einer Sonderregel, die für die ganze Boss-Serie gilt. Die Regel wird vor der Serie angezeigt, damit der Spieler sein Blatt und seine Ansagen anpassen kann.

Der Boss eines Abschnitts wird beim Start des Abschnitts gezogen (Ereignis `BossAnnounced`) und ist ab dann in der Sicht lesbar (`upcoming_boss`, `next_series`). So kann der Spieler schon am Tresen vor Serie 2 auf den Boss hin einkaufen. Jeder Boss hat einen Zielwert-Faktor (`target_factor` in `bosses.ron`, Standard 1,0), der den Zielwert seiner Boss-Serie skaliert, weil die Bosse unterschiedlich schwer sind. Die Werte setzt `balance`; messen lässt sich das mit `skat_sim --curve --boss <id>`.

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
- **Schwarzer Peter** (`MultiplyGameValue(0)`, wenn gewonnen ohne Schneider). Nur Alleinspieler.
- **Kiebitz-Mörder** (`DisableInformation`): Kiebitz, Zähler und „Durchsichtig“ sind aus.
- **Der Alte** (`rotation` in `bosses.ron`): in Spiel n die Regel des n-ten Bosses der Liste (Schieber, Zwillinge, Kontra-Hans, Revolutionär, Ramscher, Oma, Schwarzer Peter, Kiebitz-Mörder), Pässe zählen mit. Bosse außerhalb des Charakter-Pools werden übersprungen. Er ist immer der Boss des letzten Abschnitts (`final_boss` in `run.ron`).

Zielwert-Faktoren nach der M4-Messung (`skat_sim --seed 42 --runs 3000 --curve --boss <id>`, P(Serie gewonnen) bei Zielwert 24 / 63 / 110, normale Serie 67 / 41 / 18,5):

| Boss | ohne Faktor | Faktor | Grund |
|---|---|---|---|
| Die Oma | 28 / 7,4 / 0,7 | **0,5** | ab Abschnitt 4 unschlagbar; ohne Buben gewinnt der Bot höchstens 35 % |
| Der Schwarze Peter | 40 / 19,6 / 6,6 | **0,6** | Obergrenze 41 %, später kaum schlagbar |
| Der Ramscher | 67 / 59 / 37,5 | **1,5** | der Ramsch bringt viel; 1,5 trifft die normale Kurve von 24 bis 110 |
| Der Alte | 71 / 55 / 35 (bei 180: 12,9 gegen 3,6) | **1,4** | als Endboss viel leichter als eine normale Serie |
| Kontra-Hans | 70 / 52 / 32,5 | 1,0 | unverändert (Wert von `balance`), im Spätspiel leichter: Vorschlag 1,4 an `balance` |
| Schieber, Zwillinge, Revolutionär, Kiebitz-Mörder | wie normal oder schwerer | 1,0 | Zwillinge, Revolutionär und Kiebitz-Mörder wirken über Information, die der Bot kaum nutzt; für ihn sind sie so schwer wie eine normale Serie |

Die Faktoren gelten für beide Charaktere. Für den Nullspieler ist der Schieber fast unschlagbar (21 % statt 67 % in Abschnitt 1, Null Hand ohne Skat), der Ramscher dagegen normal; Faktoren pro Charakter sind offen (17).

---

## 12. Wirtschaft

- **Währung**: Groschen. Thematisch Zehntelpfennig, wie am Kneipentisch.
- **Einnahmen**: Grundbetrag pro gewonnener Serie, Bonus pro übrigem Spiel, Zinsen auf Guthaben (gedeckelt), Verkauf von Relikten.
- **Ausgaben am Tresen**: Hausregeln, Figuren, Verbrauchsgegenstände, Kartenmodifikationen, Neuauslage.
- **Richtwerte (M2, kalibriert)**: Startkapital 4, Serie-Grundbetrag 3, Bonus pro übrigem Spiel 1, Zinsen 1 pro 5 Groschen bis maximal 5, Neuauslage 2. Hausregeln kosten 5 bis 8: starke 7 bis 8 (Bockrunde 8, Alte Dame 7, Schenken 7, Tournee 8), schwache und noch nicht wirksame 5. Figuren 6 bis 10, Verbrauchsgegenstände 2 bis 4. Messung und Begründung in `docs/balance/2026-11-01-m2-zielwerte.md`. Der Run-Bot hat damit beim Start von Abschnitt 2 im Mittel drei Hausregeln, ab Abschnitt 3 alle fünf Plätze belegt. Fürs Spätspiel fehlen noch Ausgaben (offen für M4 und M5).

---

## 13. Ereignisse (Rast)

Zwischen Serien kann statt des Tresens ein Ereignis kommen. Beispiele:

- **Stammtisch-Streit**: Wähle eine Hausregel, die für den Rest des Abends für alle gilt.
- **Der Kartengeber**: Lass eine zufällige Karte zinken oder zahle 3 Groschen und wähle selbst.
- **Alter Bekannter**: Tausche ein Relikt gegen ein zufälliges anderes.
- **Letzte Runde**: Spiel ein einziges Spiel um den doppelten Einsatz.

**Umgesetzt in M4** (`core/content/encounters.ron`, Begriff im Code `Encounter`, weil `Event` die Zustandsereignisse sind). Nach einer gewonnenen Serie (nicht nach der letzten) kommt mit der Wahrscheinlichkeit `encounter_chance` aus `run.ron` (Platzhalter 0,2 für `balance`) ein Ereignis statt des Tresens, gezogen nach `weight`. Es bietet Optionen an (`EncounterOption`, immer zuletzt „Weitergehen“), der Spieler wählt mit `RunAction::ChooseEncounter`, danach beginnt sofort die nächste Serie.

- **Stammtisch-Streit**: drei Hausregeln zur Wahl (freigeschaltet, nicht im Besitz, für den Charakter). Die gewählte gilt bis zum Ende des Abschnitts („Abend“) ohne Platz zu belegen.
- **Der Kartengeber**: „zinken“ heißt eine Kartenmodifikation anbringen. Kostenlos auf eine zufällige Karte, für 3 Groschen auf eine gewählte; die Modifikation ist zufällig aus dem Pool des Charakters. Seit M5 wird sie gezogen, wenn das Ereignis aufgeht, und beide Optionen nennen sie (`EncounterOption.card_mod`, in der Sicht `card_mod` mit Name und Fluch-Markierung). Der Spieler entscheidet also mit Wissen; zufällig ist nur noch die Karte.
- **Alter Bekannter**: Ein gewähltes eigenes Relikt wird gegen ein anderes getauscht. Seit M5 wird das neue Relikt gezogen, wenn das Ereignis aufgeht, und jede Option nennt es (`SwapRelic.new`, in der Sicht `new_relic`). Bei Spielständen von vorher ist es weiter zufällig, und die Option sagt das (`random_gain`).
- **Letzte Runde**: Das nächste angesagte Spiel zählt doppelt, gewonnen wie verloren (Spielwert ×2, Herkunft `Source::Encounter`). Wie die Bockrunde wird es bei der Ansage verbraucht, Einpassen verschenkt es nicht.

Gemessen (3 × 1000 Runs): etwa 0,45 Ereignisse pro Run; der Bot nimmt Hausregel und Gratis-Karte immer, tauscht schwache Relikte und lehnt die Letzte Runde ab.

**Erweitert in M4 um fünf Ereignisse** mit denselben vier Formen (entschieden von Claude am 01.11.2026):

- **Umgekipptes Bier** (`spilled_beer`, alle): Eine zufällige Karte bekommt den Fluch „Bierfleck“ (−5 Augen, dafür siehst du, wer sie hält), für 1 Groschen eine gewählte. Wer weitergeht, bleibt sauber; wer wählt, legt den Fleck auf eine Lusche und hat nur den Ausgleich.
- **Gasthof zur Krone** (`crown_inn`, Alleinspieler): „Krone“ auf eine zufällige Karte, für 4 Groschen auf eine gewählte.
- **Der Taschenspieler** (`conjurer`, Nullspieler): „Attrappe“ auf eine zufällige Karte, für 3 Groschen auf eine gewählte.
- **Preisskat** (`prize_skat`, alle): Das nächste Spiel zählt dreifach, gewonnen wie verloren.
- **Gast aus Altenburg** (`guest_from_altenburg`, alle): wie der Stammtisch-Streit, aber nur zwei Hausregeln zur Wahl.

Damit trifft jeder Charakter acht Ereignisse. Der Bot nimmt jede Gratis-Karte, also auch den Bierfleck; den Preisskat lehnt er wie die Letzte Runde ab.

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
| **M7 Langzeit** | Kurzer Abend (3 Abschnitte), Endlos-Modus nach dem Endboss, Einsatz-Leiter (jeder Run-Sieg hebt die Gewinngrenze um ein Auge), angekündigte Flüche statt zufälliger | Grund, wiederzukommen |

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
| Assets | CC0 und CC-BY mit Namensnennung. Jede Quelle steht in `web/LICENSES.md` und im Abspann. |
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
| Run-Länge | 5 Abschnitte als Standard, dazu ein „kurzer Abend“ mit 3 Abschnitten. Freischaltungen in beiden (M7). |
| Schwierigkeit | Endlose Einsatz-Leiter: Jeder gewonnene Run hebt die Gewinngrenze des nächsten um ein Auge (61, 62, 63 …). Schneider rückt nicht mit. Wirkung wird vorher im Simulator gemessen (M7). |
| Nach dem Endboss | Endlos-Modus: Zielwerte wachsen weiter, nur für den Highscore (M7). |
| Flüche | Nur angekündigt (z. B. bei Ereignissen) und mit Groschen bezahlt, nie aus Zufallsziehungen (M7). |
| Normale Gegner | Wechselnde Stammgäste mit Namen und Pixelgesicht, ohne eigene Regeln. Regeln bleiben den Bossen (M6). |
| Relikte nur für Menschen | Behalten, selten und günstig (Mit Vieren, Der Opa, Saubere Weste, Bierdeckel, Glashaus, Ouvert-Bonus). |
| Musik | Leise Chiptune-Schleifen für Tisch, Tresen und Boss, getrennt von den Klängen abschaltbar (M6). |
| Wertung | Im Spiel eine kleine Zeile, nach dem letzten Stich eine große, überspringbare Abrechnung (M6). |
| Null | Für jeden ansagbar, Nullspieler bleibt als Spezialist (ADR 0012, M6). |
| Domain | sslip.io bleibt vorerst. Spielstand-Export als Text und dauerhafter Browserspeicher kommen trotzdem (M6). |
| KI-Tempo | Am Handy flüssig, kein Hintergrund-Thread nötig. |
| Planung | M6 Feinschliff und M7 Langzeit. Der Weckruf läuft bis M7. |

## 17. Offen

- **Null-Formel**: entschieden in M4 (9.2). Offen bleibt das Feintuning der Stilboni mit Testspielen.
- **Zielwert-Faktoren pro Charakter**: Bosse wirken auf den Nullspieler anders (Schieber, Ramscher, 11). Erledigt sich weitgehend mit ADR 0012, weil der Nullspieler dann auch Farbe und Grand spielen darf; nach dem Umbau nachmessen.
- **Ereignis-Wahrscheinlichkeit, Figurenpreise**: Platzhalter, `balance`.

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
| 2 | **Das Null-Spiel ist ein eigener Charakter**, nicht eine Option im Standard-Run. | Null ist mechanisch ein anderes Spiel. Als Charakter kann es eigene Karten, Relikte und Ziele bekommen. |
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

**Reizen entfällt.** Stattdessen gibt es die **Ansage**. Vor dem Aufnehmen des Skats erklärt der Spieler:

1. Spielart (Farbe oder Grand; beim Null-Charakter Null-Varianten).
2. Risikostufe: *Normal*, *Hand* (Skat bleibt liegen), *Schneider angesagt*, *Schwarz angesagt*, *Ouvert* (offen spielen).

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
| Ein Spiel | Ansage, Skat, 10 Stiche, Abrechnung | Eine Hand | ~90 Sekunden |
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
3. **Ansage.** Spielart und Risikostufe wählen. Bei *Hand* wird der Skat nicht aufgenommen.
4. **Skat aufnehmen und drücken.** 2 Karten aufnehmen, 2 Karten drücken. Gedrückte Karten zählen zu den Augen des Alleinspielers.
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

Geändert am 01.11.2026 nach der M2-Messung (`docs/balance/2026-11-01-m2-zielwerte.md`), weil vier Hausregeln keinen messbaren Einfluss hatten (Designsäule 1): Spitze verlangte die Karo-7 und kam praktisch nie vor, Jungfrau lag knapp unter 5 % Einfluss, Durchmarsch hing an Schwarz (0,8 % der Spiele), Schieben war erwartungsneutral. Nachmessung mit `skat_sim --runs 3000 --no-shop --relic <id>` (Seeds 42 bis 44, Abschnitt 2 erreicht, ohne Hausregel 24,4 %): Durchmarsch +11 %, Schieben +16 %, Jungfrau +6 %, Spitze weiter unter 1 %. Preise unverändert.

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

### 9.3 Freischaltungen über Runs hinweg

Entschieden am 01.11.2026: Neben dem Nullspieler werden auch Relikte freigeschaltet.

- Ein neuer Spielstand kennt etwa die Hälfte der Hausregeln und Figuren. Die übrigen erscheinen erst nach einer Freischaltung im Tresen.
- Freischaltungen hängen an Taten am Tisch, nicht an Spielzeit: zum Beispiel einen Grand Schneider gewinnen, Abschnitt 3 erreichen, einen Boss ohne Pass schlagen.
- Jede Freischaltung ist an genau einer Stelle als Daten definiert (Bedingung und Belohnung) und wird im Simulator auf Erreichbarkeit geprüft.
- Gespeichert wird im Browser des Spielers: laufender Run, Freischaltungen, Statistik der Runs. Kein Konto, kein Server-Speicher.

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

### 10.3 Schwierigkeit

Schwierigkeit steigt **nicht** primär über klügere KI, sondern über die Zielwertkurve und über Boss-Regeln. Die KI darf aber zwischen den Abschnitten von Stufe 1 auf Stufe 2 wechseln, mit mehr Samples je später im Run.

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
| Null-Wertung | Wird in M4 mit dem Simulator entschieden (9.2). |

## 17. Offen

- **Null-Formel**: Der Vorschlag in 9.2 ist ein erster Wurf und braucht den Simulator.

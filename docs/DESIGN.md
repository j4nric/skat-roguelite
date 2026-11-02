# Stammtisch – Design-Dokument

Stand: 2026-11-02 · Status: Entwurf v0.5 (eigene Ausgaben, ADR 0015; Effekte v2, 7.2) · Sprache des Projekts: Deutsch (Code und Kommentare: Englisch) · Arbeitstitel bis 31.10.2026: „Skat Roguelike“

Dieses Dokument hält die Grundentscheidungen fest und beschreibt das Spiel so weit, dass ein erster spielbarer Prototyp daraus gebaut werden kann. Alles unter „Offen“ ist noch nicht entschieden. Alles andere gilt, bis es hier geändert wird.

---

## 1. Pitch

**Stammtisch: Skat als Roguelike-Deckbuilder.** Du bist immer der Alleinspieler. Über einen Run hinweg spielst du Skat gegen zwei KI-Gegenspieler, musst in jeder Runde einen Punkte-Zielwert erreichen und veränderst zwischen den Runden deine Ausgabe des Blatts: Gegeben wird ein Skatblatt für alle drei, aber jeder hält seine eigene Ausgabe jeder Karte (ADR 0015). Die Spielwertformel des echten Skats (Grundwert × Gewinnstufen) ist die Score-Engine, Hausregeln sind die Relikte, die Kneipe ist der Shop.

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
2. **Das Blatt ist geteilt, die Ausgaben nicht** (ADR 0015, ersetzt „Deine Karten können bei den Gegnern landen“). Alle drei spielen mit denselben 32 Karten, aber jeder mit seiner eigenen Ausgabe. Jede Karte, die ein Gegner bekommt, fehlt dir: Deine Aufwertung schläft in diesem Spiel, und was du ausspielst, kann im Stich des Gegners landen. Deckbau ist ein Risiko, kein reiner Vorteil. Das unterscheidet dieses Spiel von Balatro und wird verteidigt.
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

- **Spitzen zählen nur über Trümpfe, die im Spiel sind.** Ist der Kreuz-Bube entfernt, beginnt die Zählung beim nächsten vorhandenen Trumpf. Sonst brächte jedes Entfernen eines Buben allen ein Gratis-„ohne“. Seit ADR 0015 wird keine Karte mehr entfernt; die Regel gilt weiter für Unter, die ein Karteneffekt zur Blanko macht (Boss-Regel der Oma). Die Spitzen zählen über die zwölf Karten des Alleinspielers **in seiner Ausgabe**: Ist ein Unter für ihn keine Unter (Nullspieler, 9.2), zählt er nicht, wie eine umgemalte Karte der Kreide (8.3).
- **Blankos** bedienen nie und gewinnen nie. Wird eine Blanko ausgespielt, bestimmt die erste normale Karte die Farbe. Drei Blankos gehen an den Ausspieler. Seit ADR 0015 entstehen sie nur noch durch Karteneffekte (Boss-Regel der Oma, ein zerbrochener Glasdaus, 7.2), nie durch ein kleineres Blatt.
- **Gezinkte Karten** (seit Effekte v2: Rang, 7.2) steigen eine Stufe je Rang. Bei gleicher Stärke behält die früher gespielte Karte den Stich.
- **Schneider** wird immer an der verlierenden Seite gemessen (höchstens 30 Augen). Relikte wie „Jungfrau“ legen zusätzlich Bereiche der Alleinspieler-Augen fest, die ein gewonnenes Spiel zum Schneider machen.
- ~~**Kartenmodifikationen wirken für jeden, der die Karte spielt** (ADR 0005), auch die Modifikation „Spitze“.~~ Seit ADR 0015: **Kartenmodifikationen wirken nur in der Ausgabe ihres Halters** (7.1). Die Modifikation „Spitze“ des Spielers zählt nur, wenn er die Karte spielt; die gleichnamige Hausregel verlangt weiter den Alleinspieler.
- **Bockrunde** wird bei der Ansage verbraucht, nicht beim Geben. So verschenkt Einmischen den Bock nicht.
- **Schenken**: Die restlichen Stiche gehen an den Alleinspieler. Das Spiel ist damit Schneider, und Schwarz, falls die Gegner noch keinen Stich hatten.

Ergänzt in M5 (entschieden von Claude am 01.11.2026):

- **Bube ehrenhalber** (bis Effekte v2 die Kartenmodifikation `honorary_jack`; seit 02.11.2026 gestrichen, die Wirkung `HonoraryJack` ruht in der Hook-API, 7.2): Im Farbspiel ist die Karte Trumpf und steht eine halbe Stufe unter dem Karo-Buben: Sie schlägt das Trumpf-Ass und alles darunter und verliert gegen jeden Buben, egal in welcher Reihenfolge gespielt wird. Sie bedient Trumpf und nicht mehr ihre gedruckte Farbe; wird Trumpf ausgespielt, muss sie bedient werden. Liegt sie auf einer Karte der Trumpffarbe, steigt diese Karte über das Ass. Mit Rangbonus („Gezinkt“) steigt sie ganze Stufen von dort: einmal gezinkt schlägt sie den Karo-Buben und verliert gegen den Herz-Buben. Ein gezinktes Trumpf-Ass erreicht weiter den Karo-Buben (Gleichstand, der frühere behält den Stich) und schlägt damit den Buben ehrenhalber. Zwei Buben ehrenhalber im selben Stich: Der frühere behält ihn. Die **Spitzen zählen sie nicht mit**: Gezählt wird weiter die gedruckte Trumpfordnung; ein Bube ehrenhalber ist keine zusätzliche Spitze, und eine Trumpfkarte mit der Modifikation zählt an ihrer gedruckten Stelle. Im Grand, im Null und im Ramsch bleibt die Karte, was gedruckt ist, ebenso ein echter Bube. Im Code: `rules::trump_order_key` (Stärke, dann 0 für den Buben ehrenhalber, 1 für gedruckte Trümpfe), `is_trump` und `follows_lead` kennen ihn; die Gegner-KI sieht über `trump_strength` die Stärke des Karo-Buben.
- **Nicht drücken** (bis Effekte v2 der Fluch „Ramponiert“, seit 02.11.2026 heißt Ramponiert −X Augen, 7.2; die Wirkung `NoDiscard` ruht in der Hook-API): Die Karte darf nicht gedrückt werden, solange zwei andere Karten gedrückt werden können; sonst so wenige ramponierte wie möglich (`rules::valid_discard`, Fehler `DiscardForbidden`). Das gilt auch für die Karte, die beim Skattausch der Oma oder des Opas in den Skat geht.
- **Mehrere Skattausch-Figuren**: Jede Figur mit `SeriesSkatSwap` gewährt ihren eigenen Tausch, einmal pro Serie (Oma: ein Unter, Opa: ein Daus). Sie können im selben Handspiel nacheinander genutzt werden. Die Sicht nennt die Karten, die herauskommen können (`skat_swap_cards`), die erste nimmt `TakeFromSkat` ohne `take`.

Rest abkürzen (entschieden von Claude am 01.11.2026, auf Wunsch des Nutzers):

- **Wann:** Der Spieler ist am Zug, und die restlichen Karten können an der Wertung nichts mehr ändern, egal wie alle drei sie spielen (auch er selbst, nicht nur bei bestem Spiel). Dann bietet der Tisch „Rest abkürzen“ an (`GameView::can_settle`, Aktion `RunAction::SettleGame`).
- **Entschieden heißt:** (1) Kein Hook, der während des Spiels feuert (`OnCardPlayed`, `OnTrickEnd`, z. B. Schenken, Revolutionär, Auf dem Trockenen), kann noch feuern. (2) Das Ergebnis nach den Regeln (gewonnen, Schneider und Schwarz beider Seiten, Grundwert, Gewinnstufen, Spielwert) ist für jede Augen- und Stichzahl gleich, die die Reststiche noch bringen können, mit den geltenden Schwellen (Gewinnschwelle, „Jungfrau“). Null: Der Alleinspieler bekommt in keiner Fortsetzung einen Stich (gewonnen, mit gleichem Stilbonus: Siebenen, nie bedient, Kurzschluss) oder in jeder (verloren). (3) Jeder `OnScore`-Hook feuert in allen Fortsetzungen oder in keiner, auch die des letzten Stichs (Spitze, Glückspfennig, Schlusslicht). Ramsch wird nie abgekürzt.
- **Wie gemessen:** Schranken (kleinster und größter Wert) der Reststiche aus drei Quellen, billig zuerst: aus den Karten allein; für Null ein Test auf den Händen (der Alleinspieler spielt nicht aus, kein Gegner hält eine Blanko, in jeder Farbe mit einer gewinnfähigen Karte des Alleinspielers ist jede Gegnerkarte höher); in den letzten vier Stichen eine vollständige Suche über alle legalen Karten aller Sitze mit offenen Händen (`core/src/endgame.rs`, höchstens 2000 Stichgrenzen, sonst nicht angeboten).
- **Was passiert:** Der Kern spielt die Reststiche mit fester Regel (jeder Sitz seine erste legale Karte) über das normale Kartenspiel aus. Ereignisse, Hooks, Wertung, Groschen und Statistik sind dieselben wie beim Ausspielen. Die Oberfläche lässt die Karten kurz verschwinden und zeigt dann die Abrechnung.

---

## 5. Run-Struktur

**Seit ADR 0016 (02.11.2026, Entscheidung des Nutzers) ist ein Abend eine Kneipentour.** Abschnitte und Serien sind ersetzt; was in 5.1 bis 5.3 über Serien steht, gilt für die Kneipe weiter, wo nichts anderes gesagt ist (drei Spiele, Einmischen, Auszahlung), und ist sonst Geschichte. Die Kalibrierung in 5.2 galt der alten Struktur; alle Zahlen der Kneipentour sind vorläufig (5.0.4).

```
Run (eine Kneipentour durch eine erfundene Stadt)
└── Stadtteil (3 pro Abend, z. B. Altstadt, Bahnhofsviertel, Hafen)
    ├── Kneipe 1 bis 4   (Zielwert, bis zu 3 Spiele, zwei Stammgäste)
    │     dazwischen: Routenwahl, drei Wege mit je einer Station
    │     (Tresen, Begegnung, Werkstatt, Rast), manchmal eine Spelunke
    └── Boss-Kneipe      (Boss mit Sonderregel; verloren: noch einmal)
```

| Ebene | Skat-Begriff | Genre-Gegenstück | Dauer |
|---|---|---|---|
| Ein Spiel | Skat, Ansage, 10 Stiche, Abrechnung | Eine Hand | ~90 Sekunden |
| Eine Kneipe | bis zu 3 Spiele mit Punkteziel gegen zwei Stammgäste, je Spiel bis zu zweimal einmischen | Ein Blind | 2 bis 5 Minuten |
| Ein Stadtteil | vier Kneipen und die Boss-Kneipe | Ein Ante | 15 bis 20 Minuten |
| Der Run | Kneipentour durch drei Stadtteile (15 Kneipen) | Der Run | 45 bis 60 Minuten |

### 5.0 Die Kneipentour (ADR 0016)

- **Kneipe**: Zielwert in Augen (Reizwerte wie bisher die Serie), bis zu drei Spiele gegen zwei Stammgäste. Wer den Zielwert erreicht, ist sofort fertig. Belohnung: Grundbetrag (`series_reward`, 3) plus je übrigem Spiel `per_game_left` (vorläufig 3 statt 1, damit sich eine früh geschaffte Kneipe lohnt), dazu Zinsen wie bisher (die Auszahlung, `Payout`). Im Code bleibt der Typ `Series` (Glossar).
- **Verfehlt** (Zielwert nach dem dritten Spiel nicht erreicht): ein Leben weg, weiter zur Routenwahl ohne Belohnung (`PubLost`, `LivesChanged`). Hooks nach einer gewonnenen Kneipe (`OnSeriesWon`) feuern nicht.
- **Leben** (`lives` in `run.ron`): Start 3, höchstens 5, eine geschaffte Boss-Kneipe +1. Mit dem letzten Leben ist der Run verloren (`RunLost`). Die Web-App zeigt sie als Herzen.
- **Boss-Kneipe**: letzte Kneipe eines Stadtteils, der Boss-Tisch wie bisher die Boss-Serie (Boss links, ein Stammgast rechts, Boss-Ausgabe, 11). Der Boss wird beim Start des Stadtteils gezogen und ist ab dann sichtbar. Verloren: ein Leben weg und **sofort noch einmal** gegen denselben Boss, mit neuen Spielen und neuen Stammgästen, ohne Routenwahl dazwischen (Auslegung von Claude; „Boss-Kneipe verloren: Leben weg, Boss wiederholen“). Im letzten Stadtteil sitzt der Endboss „Der Alte“; ihn zu schlagen gewinnt den Run.
- **Zweite Boss-Figur (Sekundant)**: Im Cockpit offen, ob der Boss einen Sekundanten mit derselben Ausgabe auf dem zweiten Platz bekommt. Bis dahin bleibt ein Boss-Platz (`BOSS_SEAT`); das Datenmodell ist offen dafür: Wer an welchem Platz sitzt, entscheidet allein `Run::owner_of_seat`, die Boss-Ausgabe hängt am Besitzer (`Owner::Boss`), nicht am Platz. Ein Sekundant wäre ein zweiter Platz mit `Owner::Boss` (oder ein eigener Besitzer mit eigener Ausgabe) und eine Liste statt eines Bosses in `Series::boss`.
- **Hausregeln des Stammtisch-Streits** gelten bis zum Ende des Stadtteils (bisher des Abschnitts).
- **Einmischen, Schnäpse, Relikte „einmal pro Serie“** zählen je Kneipe.

#### 5.0.1 Routenwahl und Stationen

Nach jeder Kneipe, die den Run nicht beendet, wählt der Spieler einen von **drei Wegen** zur nächsten Kneipe (`RunPhase::Route`, `RunAction::ChooseRoute`). Vorher sichtbar: die nächste Kneipe mit Namen, Stadtteil, Zielwert, Spielen und den Stammgästen am Tisch (in der Boss-Kneipe der Boss, sobald bekannt).

- **Verkehrsmittel**: Zu Fuß, Bahn, Fahrrad. Seit der Nachentscheidung des Nutzers vom 02.11.2026 **verhalten sich alle drei gleich**: Jeder Weg führt über genau eine Station, kostet nichts extra, und die Kategorie der Station wird **je Weg gewürfelt** und angezeigt. Das Verkehrsmittel ist Darstellung (Weg auf der Karte, eine dekorative Fahrzeit `minutes`). Der erste Entwurf (Bahn kostet 1 Groschen und zeigt das genaue Ereignis, Fahrrad zwei Stationen mit einer Überraschung) ist verworfen.
- **Stationen** (`StopCategory`): **Tresen** (der Laden wie bisher), **Begegnung** (die Ereignisse aus 13), **Upgrade** („Werkstatt“, selten: eine Modifikation auf eine gewählte Karte oder eine Karte ganz säubern; dazu der Kartengeber, der Gasthof zur Krone und der Taschenspieler, die Karten aufwerten) und **Rast** (Imbissbude: ein Leben zurück, nur gezogen, solange ein Leben fehlt). Gewichte in `run.ron` (`route.weights`, vorläufig Tresen 40, Begegnung 35, Upgrade 12, Rast 8). Unter den drei Wegen ist immer einer mit Tresen. Das genaue Ereignis ist beim Wählen gezogen, aber verdeckt; die Kategorie ist sichtbar. Jedes Ereignis trägt seine Kategorie als Daten (`category` in `encounters.ron`).
- **Ein Ereignis ohne Angebot** (etwa „Alter Bekannter“ ohne Relikt) wird nicht auf einen Weg gelegt; geht eines unterwegs doch leer aus, geht es direkt in die Kneipe.
- **Zufall**: eigener Generator des Runs für Wege, Spelunken und Mali (`tour_rng`); die Karten der Spiele ändern sich dadurch nicht.

#### 5.0.2 Die Spelunke

Mit der Wahrscheinlichkeit `spelunke.chance` (vorläufig 0,3) führt **einer** der drei Wege statt zur nächsten Kneipe in eine **Spelunke**, nie vor einer Boss-Kneipe. Die Spelunke ist **ein eigener Ort**: eigener Name aus eigener Wortliste (`city.spelunken` in `tour.ron`), eigener Knoten (`d0-p2-s`); sie steht in der Reihenfolge des Stadtteils für die Kneipe, die sie ersetzt (diese gilt als übergangen, `PubState::Skipped`).

- **Zielwert**: der der ersetzten Kneipe × `spelunke.target_factor` (vorläufig 1,25).
- **Malus**: einer, zufällig nach Gewicht, vor der Wahl angezeigt (Antwort-Vorschlag der offenen Cockpit-Frage „ein zufälliger Malus, vorher sichtbar“, umgesetzt als Voreinstellung). Mali sind Daten (`maluses` in `tour.ron`): **Kein Einmischen** (`no_redeal`), **Klebrige Karten** (zwei zufällige Karten der Ausgabe ohne Modifikation sind ramponiert, Augen −5, nur für diese Kneipe, ohne die Groschen eines Fluchs; `curse`), **Kein Skat** (jedes Spiel Hand, `passive: [RequireHand]`, nicht für den Nullspieler) und **Schnapsverbot** (`lock_schnapps`, Fehler `ConsumablesLocked`). Tischregeln wirken als Quelle `Source::Malus` wie eine Boss-Regel.
- **Belohnung**: die normale Auszahlung, dazu die **Beute**: ein Relikt, gezogen beim Gewinnen, in einen freien Platz oder für den halben Preis in Groschen (Ereignis `spelunke_loot`). Verloren kostet sie ein Leben wie jede Kneipe.

#### 5.0.3 Die Stadt und ihre Namen

Stadtteile und Kneipen heißen nach einem **Namensgenerator** mit deutschen Wortlisten in `tour.ron` („Zum Goldenen Hirschen“, „Eckkneipe bei Uschi“, „Bahnhofsklause“, Boss-Kneipen wie „Vereinsheim Gut Blatt“), je Run aus dem Seed, ohne einen anderen Zufallsstrom zu berühren. Namen sind Anzeige. Die **Kartengeometrie** (Straßen, Lage) gehört der Web-App: Die Sicht liefert stabile Knoten-Ids (`d<Stadtteil>-p<Kneipe>`, Spelunke mit `-s`, Wege `…-r<n>`), je Stadtteil und für die Stadt einen Seed (`map_seed`) und den ganzen Plan des Abends (`TourView`: Stadtteile, Kneipen mit Zustand gewonnen, verloren, aktuell, kommend, übergangen, Spelunken, Boss des laufenden Stadtteils), damit die Web-App eine Übersicht und später eine Pixel-Stadtkarte zeichnen kann.

#### 5.0.4 Vorläufige Zahlen der Kneipentour

Gesetzt am 02.11.2026 nach Augenmaß, **ohne Messung** (die Simulationen ruhen auf Wunsch des Nutzers). Grundlage: Die 15 Zielwerte der alten Kurve (5 Abschnitte × 3 Serien, 5.2) laufen über dieselbe Spanne auf 15 Kneipen. Mit Leben ist eine einzelne Kneipe weniger tödlich; ob die Kurve steiler werden muss, zeigt erst die Messung.

| Größe | Wert | Ort |
|---|---|---|
| Stadtteile, Kneipen je Stadtteil | 3, 4 + Boss-Kneipe | `targets` in `run.ron`, `PUBS_PER_DISTRICT` |
| Zielwerte Alleinspieler | 33 / 36 / 38 / 40 / Boss 45 · 40 / 42 / 44 / 46 / Boss 52 · 46 / 48 / 50 / 52 / Boss 60 | `run.ron` |
| Zielwerte Nullspieler | 18 / 20 / 20 / 23 / 23 · 20 / 23 / 23 / 27 / 27 · 23 / 27 / 27 / 30 / 30 | `characters.ron` |
| Zielwerte Grandspieler | 48 / 50 / 52 / 54 / 60 · 52 / 54 / 55 / 57 / 60 · 55 / 57 / 59 / 60 / 66 | `characters.ron` |
| Zielwerte Zaungast | 33 / 35 / 37 / 40 / 45 · 36 / 38 / 40 / 42 / 46 · 40 / 42 / 44 / 44 / 48 | `characters.ron` |
| Zielwerte Wirtin | 40 / 42 / 44 / 46 / 54 · 44 / 46 / 47 / 48 / 55 · 48 / 50 / 52 / 54 / 60 | `characters.ron` |
| Boss-Faktoren | unverändert (11) | `bosses.ron` |
| Gegner je Stadtteil | Heuristik, PIMC 8:3, PIMC 12:3 (die alten Abschnitte 1, 3, 4) | `opponents` |
| Leben | Start 3, höchstens 5, Boss +1 | `lives` |
| Grundbetrag, je übrigem Spiel | 3, 3 (bisher 1) | `economy` |
| Gewichte der Stationen | Tresen 40, Begegnung 35, Upgrade 12, Rast 8 (Rast nur mit fehlendem Leben); immer ein Tresen | `route.weights` |
| Spelunke | Chance 0,3, Zielwert × 1,25, ein Malus (alle Gewicht 1) | `spelunke`, `tour.ron` |
| Rast | +1 Leben | `encounters.ron` (`snack_bar`) |
| Bot-Route | Tresen ab 8 Groschen, Upgrade mit verfluchter Karte, Rast mit fehlendem Leben; Spelunke nur mit 3 Leben und 3 Relikten | `RunBot::route_action` |

Zu messen, sobald der Simulator wieder läuft: Stadtteil 2 und 3 erreicht und Run gewonnen je Charakter (Zielbild neu festzulegen, bisher: ein Drittel erreicht die zweite Stufe, 1 bis 2 % gewinnen), mittlere Leben je Stadtteil, Anteil verlorener Kneipen, Wert der Spelunke (Gewinnquote mit und ohne Malus), Groschen je Run mit dem höheren Bonus je Spiel, Erreichbarkeit der umgeschriebenen Freischaltungen.

### 5.1 Die Serie (seit ADR 0016: die Kneipe)

Gilt für die Kneipe weiter: Zielwert, drei Spiele, Einmischen, Auszahlung. Überholt: „Wer das Limit erreicht, ohne den Zielwert zu schaffen, verliert den Run“ (seit ADR 0016 kostet es ein Leben, 5.0).


- Jede Serie hat einen **Zielwert** (siehe 5.2) und ein **Spiellimit**: seit dem 01.11.2026 **3 Spiele** gegen dieselben zwei Stammgäste („3 Spiele bilden eine Serie, 2 Serien münden in eine Boss-Serie“, Entscheidung des Nutzers nach dem Spieltest). Vorher: M5 8 Spiele mit bis zu 6 Pässen, M2 5 und 3 (5.2).
- Gewonnene Spiele addieren ihren Spielwert. Verlorene Spiele ziehen den doppelten Spielwert ab.
- **Einmischen** (`RunAction::Redeal`): Vor der Ansage, solange der Skat unberührt ist, gibt der Spieler das Blatt zurück und es wird neu gegeben. Das kostet kein Spiel und keine Punkte, die Vorhand bleibt. Jedes Spiel erlaubt zwei (`redeals` in `run.ron`), danach muss eines der Blätter gespielt werden. Der Zaungast hat eins weniger (9.5), die Hausregel Schieben gibt eins mehr pro Spiel (8.1), Ein Bier und Kräuter geben eins für den Rest der Serie, das nach denen des Spiels verbraucht wird (8.3), „Neues Blatt“ mischt einmal, ohne eins zu verbrauchen, und der Schnaps „Reue“ nimmt das letzte Einmischen zurück. Die Kopfzeile zeigt, wie oft der Spieler im laufenden Spiel noch einmischen darf (eigene plus die der Serie, `SeriesView::redeals_left`). Der Ramsch des Ramschers wird nie eingemischt. Das Einmischen ersetzt das Einpassen, das ein Spiel vom Limit kostete: Wer seine Pässe verbraucht hatte, musste schlechte Blätter spielen und verlor faktisch den Run.
- Wer das Limit erreicht, ohne den Zielwert zu schaffen, verliert den Run.
- Übrige Spiele am Ende einer gewonnenen Serie werden in Groschen umgewandelt (Zinsmechanik wie in Balatro). Mit 3 Spielen sind das 0 bis 2.
- **Alte Spielstände** laden: Eine Serie, die mitten in den alten 8 Spielen gespeichert wurde, spielt mit ihrem gespeicherten Spiellimit zu Ende; ab der nächsten Serie gelten 3 Spiele. Übrige Pässe verfallen, Einmischen gilt sofort nach der neuen Regel (das laufende Spiel hat seine zwei). Gespeicherte Statistik zählt eingepasste Spiele als Einmischen. Alte Namen (`Pass`, `passes`, `passes_used`, `games_passed`, `PassesAdded`, `ExtraPasses`, `SeriesRedeal`, `series_redeals`, `without_pass`) werden weiter gelesen.
- **Im Code**: `Series::redeals_in_game` (verbraucht im laufenden Spiel, mit jedem neuen Spiel 0), `Series::extra_redeals` (Ein Bier, Kräuter), `Series::redeals_used` (alle der Serie, auch „Neues Blatt“), `Run::redeals_left(content)`; Fehler `NoRedealLeft`.

### 5.2 Zielwert-Progression (Abschnitte und Serien, bis ADR 0016)

Die folgende Kalibrierung galt der alten Struktur aus fünf Abschnitten. Die vorläufigen Zielwerte der Kneipentour stehen in 5.0.4.


**Kalibriert am 02.11.2026 für Serien mit 3 Spielen** (zweimal einmischen je Spiel, Ansagen nach dem Skat, Relikte bis Welle 2a), Messung in `docs/balance/2026-11-02-kalibrierung-dreierserie.md`. Zielbild wie in M5: Der PIMC-Spieler (`--bot pimc:12:3`) erreicht Abschnitt 2 in gut einem Drittel der Runs und gewinnt 1 bis 2 % der ganzen Abende, **mit jedem Charakter**. Deshalb hat jeder Charakter außer dem Alleinspieler eigene Zielwerte (`targets` in `characters.ron`), alle Reizwerte. Je Zelle: Serie 1 / Serie 2 / Boss-Serie, die Boss-Serie vor dem Boss-Faktor (11).

| Abschnitt | Gegner | Alleinspieler (`run.ron`) | Nullspieler | Grandspieler | Zaungast | Wirtin |
|---|---|---|---|---|---|---|
| 1 | Heuristik | 33 / 40 / 45 | 18 / 20 / 23 | 48 / 54 / 60 | 33 / 40 / 45 | 40 / 46 / 54 |
| 2 | PIMC 6:2 | 35 / 40 / 46 | 20 / 23 / 27 | 48 / 54 / 60 | 33 / 40 / 45 | 40 / 46 / 54 |
| 3 | PIMC 8:3 | 36 / 44 / 48 | 20 / 23 / 27 | 48 / 54 / 60 | 35 / 40 / 46 | 40 / 46 / 54 |
| 4 | PIMC 12:3 | 40 / 46 / 54 | 23 / 27 / 30 | 48 / 55 / 63 | 35 / 40 / 46 | 44 / 48 / 55 |
| 5 | PIMC 16:3 | 45 / 50 / 60 | 23 / 27 / 30 | 50 / 60 / 66 | 36 / 44 / 48 | 46 / 54 / 60 |

Ergebnis (3 × 800 Runs je Charakter und Länge, Seeds 42 bis 44, Standardfehler für „Abschnitt 2“ rund 1 Punkt; im kurzen Abend sind Abschnitt 1 und 2 dieselben Runs):

| Charakter | Abschnitt 2 | 3 | 4 | 5 | Run gewonnen | kurzer Abend gewonnen | gehalten je Abschnitt |
|---|---|---|---|---|---|---|---|
| Alleinspieler | 37,3 % | 13,8 % | 5,7 % | 2,7 % | 1,42 % | 6,00 % | 37 / 37 / 41 / 47 / 53 % |
| Nullspieler | 36,9 % | 14,5 % | 5,6 % | 1,9 % | 0,58 % | 5,96 % | 37 / 39 / 38 / 34 / 30 % |
| Grandspieler | 36,3 % | 13,2 % | 4,8 % | 2,2 % | 1,21 % | 4,92 % | 36 / 36 / 36 / 47 / 54 % |
| Zaungast | 37,2 % | 13,9 % | 5,1 % | 2,5 % | 1,29 % | 5,54 % | 37 / 37 / 37 / 49 / 52 % |
| Wirtin | 39,9 % | 17,0 % | 7,8 % | 3,6 % | 1,58 % | 7,54 % | 40 / 42 / 46 / 47 / 44 % |

Zur Form:

- **Abschnitt 1 zieht an, danach steigt die Kurve flach.** Eine Serie mit drei Spielen hat eine flache Erfolgskurve: Der Alleinspieler gewinnt sie ohne Relikte gegen Heuristik-Gegner bei Zielwert 1 zu 86 %, bei 24 zu 82 %, bei 60 zu 67 % (rund 0,4 Punkte je Zielwert-Punkt). Mit den Relikten wird der Run nach hinten leichter; eine Kurve, die wie bisher um 6 je Abschnitt steigt, machte Abschnitt 1 zu leicht oder den Rest zu schwer. Gegen die stärkeren PIMC-Gegner ab Abschnitt 2 hält dieselbe Zahl mehr.
- **Abstände zwischen den Charakteren**: Der Grandspieler braucht vorne rund 1,4-mal so viel (seine Kurve ist flacher, ein Grand bringt viel), die Wirtin vorne rund 1,2-mal (doppelte Groschen kaufen früh Relikte) und hinten kaum mehr (vier Plätze). Der Zaungast fängt wie der Alleinspieler an, wird aber gegen die PIMC-Gegner mit nur einem Einmischen schwächer und steigt deshalb flacher. Der Nullspieler gewinnt eine Serie ohne Unter auch bei Zielwert 1 nur zu 76 % (Alleinspieler 86 %); er bekommt rund die Hälfte, 23 ist ein einfaches gewonnenes Null. Seine Kurve liegt ab Abschnitt 3 fast am Boden, sein „Run gewonnen“ bleibt trotzdem unter 1 % (17). Deshalb mischt er seit 02.11.2026 dreimal statt zweimal ein (`rule: (redeals: 1)`): gemessen (2 × 800 Runs, Seeds 42 und 43) hebt das ihn von 38,6 auf 40,0 % (Abschnitt 2) und von 0,69 auf 1,12 % (Run gewonnen).
- **Boss-Serien** gewinnt der Bot in Abschnitt 1 bis 4 zu 60 bis 69 %, normale Serien zu 70 bis 89 %. Je Boss siehe 11.
- **Länge**: Ein ganzer Abend spielt im Mittel 6,3 bis 7,0 Spiele und 3,4 bis 5,9 Einmischen je Run; der kurze Abend gewinnt 5 bis 7,5 % (er ist leichter, weil zwei Abschnitte fehlen, 5.3).

```sh
cargo run --release -p skat_sim -- --seed 42 --runs 800 --bot pimc:12:3 --character declarer|null|grand|onlooker|landlady [--short]
# dasselbe mit --seed 43 und --seed 44
```

#### Vorläufige Kurve (01.11.2026 bis 02.11.2026)

Gleich nach der Umstellung auf 3 Spiele: Alleinspieler 24/30/36, 30/36/42, 36/42/48, 42/48/54, 48/54/60, Grandspieler, Zaungast und Wirtin dieselbe, der Nullspieler dieselbe eine Zeile später. Gemessen (3 × 800 Runs, `main` 1dace9c): Abschnitt 2 / Run gewonnen Alleinspieler 43,0 / 1,92 %, Nullspieler 31,7 / 0,42 %, Grandspieler 49,5 / 2,92 %, Zaungast 43,6 / 1,29 %, Wirtin 49,0 / 2,96 %; kurzer Abend gewonnen 7,7 / 3,3 / 10,3 / 6,4 / 12,8 %.

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

### 5.3 Kurzer Abend (M7, entfällt seit ADR 0016)

**Entfällt vorerst** (ADR 0016): Es gibt nur noch den ganzen Abend mit drei Stadtteilen. Der Startbildschirm zeigt keine Wahl mehr; alte Links mit `-kurz` spielen den Seed als ganzen Abend (das Angebot sagt es). `RunLength::Short` bleibt im Kern, damit alte Meta-Stände mit Statistik und Einsätzen des kurzen Abends laden; ein Run kann so nicht starten (`InvalidStart`, in der Web-Schicht `InvalidLength`), der Simulator lehnt `--short` ab. Das Folgende ist Geschichte.


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

Seit ADR 0016 gibt es eine Länge: Die Leiter steigt je Charakter für den einen Abend (`MetaState::stakes[character][Full]`). Einsätze, die im kurzen Abend freigeschaltet wurden, bleiben gespeichert, wirken aber nicht mehr.


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

Gemessen am 02.11.2026 (Alleinspieler, Zielwerte aus 5.2, der Bot rechnet bei Ansage und Einmischen mit der Schwelle `61 + n`; 3 × 800 Runs bei Einsatz 0, 3 × 500 bei 2 und 5): Abschnitt 2 erreicht 37,3 / 35,1 / 31,2 %, Run gewonnen 1,42 / 0,80 / 0,73 %, Farbspiel und Grand gewonnen 72,1 / 69,6 / 65,9 %, Null-Anteil 32,6 / 33,6 / 35,8 %. Ein Auge kostet rund 1,2 Punkte „Abschnitt 2“; die Leiter ist flacher als vor dem einsatzbewussten Bot (dort halbierte Einsatz 5 den Abschnitt-2-Wert fast). Null wird bei hohem Einsatz häufiger, dominiert bei 5 aber nicht. Messung in `docs/balance/2026-11-02-kalibrierung-dreierserie.md`.

---

## 6. Ein Spiel im Detail

1. **Geben.** Das Skatblatt (32 verschiedene Karten) wird gemischt, 10/10/10/2. Jede Karte zeigt die Ausgabe ihres Halters: In der Hand des Spielers und im Skat seine, bei einem Stammgast die normale, beim Boss die Boss-Ausgabe (ADR 0015, 7.1).
2. **Blatt ansehen.** Der Spieler sieht seine 10 Karten. Relikte, die Informationen geben (Kiebitz), wirken jetzt.
3. **Skat oder Hand.** Skat aufnehmen und 2 Karten drücken, oder Hand spielen. Gedrückte Karten zählen zu den Augen des Alleinspielers. Oder vorher einmischen: neu geben lassen, bis zu zweimal pro Spiel (5.1).
4. **Ansage.** Spielart und Risikostufe wählen (ADR 0011, 0013).
5. **10 Stiche.** Vorhand spielt aus. Die Gegner-KI spielt gemeinsam gegen den Spieler.
6. **Abrechnung.** Augen zählen, Spitzen bestimmen, Gewinnstufen summieren, Spielwert berechnen, Serie-Konto aktualisieren.
7. **Triggerphase.** Relikte mit Nachwirkung (z. B. „Bock“ läuft weiter) werden aktualisiert.

Die Rollen Vorhand, Mittelhand, Hinterhand rotieren pro Spiel. Vorhand zu sein ist ein echter Vorteil im Skat und bleibt als Rhythmus erhalten.

---

## 7. Deckbau

### 7.1 Ein Blatt, drei Ausgaben (ADR 0015)

**Bis 02.11.2026 (ADR 0005):** Es gab genau ein Blatt pro Run, alle drei Spieler spielten daraus, jede Modifikation und jede hinzugefügte oder entfernte Karte betraf alle. Bei mehr als 32 Karten blieb der Rest im Talon, bei weniger füllten Blanko-Karten auf.

**Seit ADR 0015 (Entscheidung des Nutzers vom 02.11.2026):** Gegeben wird ein normales Skatblatt aus 32 verschiedenen Karten mit 120 Augen. Jeder Spieler besitzt seine eigene **Ausgabe** jeder Karte: dieselbe Karte, aber mit seinen Modifikationen. Wer eine Karte bekommt, hält seine Version. Der Spieler baut über den Run nur an seiner Ausgabe; die Gegner haben ihre eigene (Stammgäste die normale, 10.4; Bosse eine Boss-Ausgabe, 11). Keine Karte verlässt das Blatt, keine kommt hinzu, es gibt keinen Talon und kein Blatt über oder unter 32 Karten.

Regeln, entschieden vom Nutzer (1 bis 3) und ausgelegt von Claude am 02.11.2026 (4 bis 9):

1. **Keine Karten hinzufügen oder entfernen.** Was bisher das Blatt veränderte, ist ein Karteneffekt (7.3). Charaktere unterscheiden sich durch Modifikationen auf ihrer Ausgabe, nicht durch eine andere Zusammensetzung (9).
2. **Der Skat gehört dem Alleinspieler.** Die zwei Skatkarten sind immer Ausgaben des Spielers, gegeben wie gedrückt, auch im Handspiel (ihre Augen zählen für ihn wie immer).
3. **Gegnerkarten haben eine eigene Rückseite.** Die Rückseite zeigt den Besitzer der Ausgabe: die Distel (oder eine freigeschaltete Rückseite, 9.3) für die Karten des Spielers und den Skat, eine Gegner-Rückseite für die Hände der Gegner, auf Wunsch je Gast oder Boss eine eigene.
4. **Eine Karte zeigt immer die Ausgabe ihres Halters.** Wechselt sie den Halter, wechselt ihre Ausgabe mit: die Karte, die „Drei drücken“ vom linken Gegner holt, ist in der Hand des Spielers seine Ausgabe, die zurückgegebene wieder die des Gegners (8.1); beim Revolutionär tauschen die Gegner Karten, die Ausgaben bleiben bei den Plätzen (11); Heimweh holte die Karte vor dem Spiel in die Hand (gestrichen mit Effekte v2, 7.2).
5. **Im Stich bleibt eine Karte in der Ausgabe dessen, der sie gespielt hat.** Ihre Augen zählen für den, der den Stich macht. Eine Karte des Spielers mit Augen +4, die ein Gegner sticht, bringt dem Gegner die vier Augen. Das ist das verbliebene Risiko beim Ausspielen.
6. **Der Skat bleibt auch im Ramsch die Ausgabe des Spielers**, wenn er an den Gewinner des letzten Stichs geht; er wird nie gespielt und wechselt deshalb keinen Halter.
7. **Hausregeln und Boss-Regeln sind Regeln des Tisches** und gelten für jede Ausgabe (Alte Dame, Kleinvieh, Dicke Sau zählen die Augen jeder Karte; die Oma macht jeden Unter zur Blanko). Nur Kartenmodifikationen hängen an einer Ausgabe.
8. **Die Ausgaben sind öffentlich, die Hände nicht.** Der Spieler sieht vor der Boss-Serie die Boss-Ausgabe neben der Regel, die Gegner-KI kennt die Ausgabe des Spielers (10.5). Wer welche Karte hält, bleibt unbekannt wie im Skat.
9. **Aufwertungen schlafen.** Im Mittel hält der Spieler 12 von 32 Karten (Hand und Skat, 37,5 %). Liegt eine Karte bei einem Gegner, spielt in diesem Spiel dessen Ausgabe; die Aufwertung des Spielers wirkt dann nicht. Das ist das Risiko des Deckbaus (Säule 2).

Im Kern (umgesetzt in M9, 18): Die Ausgabe des Spielers ersetzt das Blatt des Runs (`Run::edition`, Typ `Edition`, je Karte die Modifikationen); die Ausgaben der Gegner ergeben sich aus dem, der am Platz sitzt (Boss links, `BOSS_SEAT`, oder Gast), und werden nicht gespeichert. Gegeben werden die 32 Karten in der Reihenfolge und mit dem Zufallsstrom von bisher (`Game::deal_editions`), dann wird jede Karte mit der Ausgabe ihres Halters aufgelöst; jede aufgelöste Karte nennt den Besitzer ihrer Ausgabe (`DealtCard::owner`: Spieler, Stammgast, Boss). Wechselt eine Karte den Halter, löst eine einzige Funktion sie neu auf (`Game::change_holder`, Ereignis `CardChangedHands`): Drei drücken (gezogene und zurückgegebene Karte), Revolutionär, Schieben (ohne Einzelereignisse, das Ergebnis steht in `SkatRedrawn`). Untot wechselt keinen Halter: Karte und Ersatz bleiben beim Spieler. Kartenmodifikationen feuern nur in der Ausgabe ihres Besitzers (`card_mod_sources` mit `Editions`).

### 7.2 Kartenmodifikationen: Effekte v2

**Entschieden vom Nutzer am 02.11.2026 („Effekte v2“), maßgeblich ab hier.** Das System aus M4 bis M8 (26 Modifikationen, mehrere je Karte, Rahmensprache mit Ringen und Abzeichen) ist ersetzt durch ein kleines System mit wenigen, steigerbaren Modifikationen. Geschichte in einem Satz: Fett, Goldrand, Gezinkt, Stempel, Krone und ihre Verwandten gehen in Augen, Rang und Trumpf mit Werten auf, Spitze heißt jetzt Letzter Stich, die Schwarze Katze heißt Fleckig, Ramponiert bekommt eine neue Bedeutung, alles andere ist gestrichen (Tabelle „Umrechnung“ unten). Daten in `core/content/card_mods.ron`, Preise, Werte und Seltenheit sind Platzhalter für `balance`.

**Grundregeln**

1. **Eine Modifikation je Karte.** Jede Karte trägt höchstens eine Modifikation. Nur Spezialkarten (Untot, Wandelkarte) dürfen zusätzlich eine normale Modifikation tragen.
2. **Steigerung.** Kommt dieselbe Modifikation noch einmal auf dieselbe Karte, steigt sie: Die Werte werden addiert (Augen +3 und Augen +5 ergeben Augen +8). Rang steigt höchstens bis 2 (`max` in den Daten; bis 02.11.2026 bis 3, gesenkt nach Antwort im Cockpit). Alte Spielstände mit Rang 3 kappt der Kern beim Laden auf 2 (`deck::RANK_MAX`, Version 3 der Modifikationen in `Edition`). Nicht steigerbar sind die Moment-Modifikationen (Letzter Stich, Erster Stich: höchstens einmal, ein zweites Mal gibt `AtMaximum`) und die Spezial-Gutscheine (ohne Wert, `AlreadyModified`); der Gutschein mit Groschen addiert, Augen und Trumpf haben keine Obergrenze (entschieden vom Lead am 02.11.2026).
3. **Ersetzen.** Eine andere Modifikation ersetzt die bisherige. Die Oberfläche fragt vorher nach; der Kern ersetzt ohne Rückfrage und meldet das Ersetzte (`DeckChange::Modified.replaced`).
4. **Ein Fluch ist die eine Modifikation.** Eine verfluchte Karte kann weder gesteigert noch ersetzt werden (Fehler `DeckError::Cursed`, die Karte fehlt in jeder Zielauswahl). Nur Frisches Deck entfernt den Fluch (heute, indem es die Karte entfernt; mit dem Umbau auf eigene Ausgaben durch Reinigen, 7.3). Ein Fluch, der auf eine Karte mit normaler Modifikation kommt, ersetzt sie.
5. **Augen und Ramponiert sind ein Wert** (Ergänzung des Nutzers vom 02.11.2026). Eine Augen-Modifikation mit negativem Wert ist der Fluch Ramponiert: ab +1 heißt sie Augen, ab −1 Ramponiert, mit allen Regeln eines Fluchs (sperrt die Karte, nur Frisches Deck entfernt sie). Kommt ein Wert zu einer Karte mit Augen dazu (etwa Ramponiert −5 vom Umgekippten Bier auf Augen +3), wird addiert, und das Vorzeichen der Summe entscheidet: −2 ist Ramponiert, +1 bleibt Augen, 0 nimmt die Modifikation ab. Auf eine ramponierte Karte kommt nichts mehr dazu. Im Kern zwei Ids (`points`, `battered`), `Deck::add_mod` rechnet mit dem Vorzeichen.
6. **Heimweh ist gestrichen** (vorerst).
7. **Für wen sie wirken.** Seit M9 (eigene Ausgaben, 7.1) wirkt jede Modifikation nur in der Ausgabe ihres Halters: Eine Karte des Spielers mit Augen +4 zählt +4, solange er sie hält, spielt oder im Skat hat; hält ein Gegner die Karte, spielt dessen Ausgabe. Trumpf und Fleckig wirken streng nur, wenn der Spieler die Karte hält (Antwort im Cockpit: streng; der Schalter `TRUMP_MODS_NEED_PLAYER` in `core/src/content/engine.rs` bleibt, ist mit den Ausgaben aber ohne Wirkung).

**Visuelle Grammatik** (die Bilder werden getrennt neu gezeichnet; bis dahin zeigt die Web-App einfache Platzhalter auf dem vorhandenen Rahmen-Code):

| Ort auf der Karte | Bedeutung |
|---|---|
| Rahmen | **was** betroffen ist (Augen, Rang, Trumpf, Stich, Gutschein) |
| Markierung oben rechts | **wann** sie wirkt (keine Markierung: immer) |
| Schrift unten Mitte, auf einem Farbklecks-Band | **wie viel** (+5, −3, ×2, +1 Stufe) |
| Feld unten links, quadratisch | nur für Flüche und negative Effekte |
| ganzes Kartenbild | Spezialkarten ändern das Grunddesign (Untot, Wandelkarte) |

**Die Modifikationen**

| Gruppe | Modifikation (`id`) | Wirkung | Werte (Platzhalter) | Hebel | Bild |
|---|---|---|---|---|---|
| Rand | Augen (`points`) | ±X Augen | Tresen +3, +5; zufällig +4 | Augen | Rahmen Augen, Zahl im Band |
| Rand | Rang (`rank`) | sticht 1 oder 2 Stufen höher | +1, Steigerung bis 2 | Stichgewinn | Rahmen Rang, Winkel nach oben statt Zahl |
| Rand | Trumpf (`trump_bonus`) | Grundwert +X, wenn die Farbe der Karte Trumpf ist und der Spieler sie hält | Tresen +2, +4; zufällig +3 | Grundwert | Rahmen Trumpf, Zahl im Band |
| Moment | Letzter Stich (`last_trick`) | +1 Gewinnstufe, wenn diese Karte den letzten Stich macht | +1, nicht steigerbar | Gewinnstufen | Markierung oben rechts, Text unten Mitte |
| Moment | Erster Stich (`first_trick`) | ×2 Augen dieses Stichs, wenn diese Karte den ersten Stich gewinnt | ×2, nicht steigerbar | Augen | Markierung oben rechts, Text unten Mitte |
| Gutschein | Gutschein (`voucher_coins`) | wenn gedrückt: +X Groschen | +3, Steigerung addiert | Groschen | Zackenrand, keine Markierung |
| Gutschein | Gutschein Untot (`voucher_undead`) | wenn gedrückt: die Karte wird Untot | – | Spezialkarte | Zackenrand |
| Gutschein | Gutschein Wandelkarte (`voucher_wild`) | wenn gedrückt: die Karte wird Wandelkarte | – | Spezialkarte | Zackenrand |
| Fluch | Durchsichtig (`transparent`) | das Wissen hängt an der Karte (siehe unten) | – | Zählen | Feld unten links |
| Fluch | Ramponiert (`battered`) | −X Augen, auch unter 0: Augen mit negativem Wert (Grundregel 5) | −5 | Augen | Feld unten links, Zahl im Band |
| Fluch | Fleckig (`stained`) | Grundwert −X, wenn die Farbe der Karte Trumpf ist und der Spieler sie hält | −3 | Grundwert | Feld unten links, Flecken leicht durchscheinend über der Karte |
| Spezial | Untot (`undead`) | wie bisher (unten) | – | Stichgewinn, Augen | eigenes Kartenbild |
| Spezial | Wandelkarte (`wild_card`) | bei Spielbeginn: nimmt die Farbe an, von der der Spieler die meisten Karten hält (ohne Unter; unten) | – | Stichgewinn, Bedienpflicht | eigenes Kartenbild (Harlekin in vier Farben, solange sie offen ist) |

**Auslegungen**, entschieden von Claude am 02.11.2026 im Rahmen der Entscheidung des Nutzers:

- **Werte in den Daten.** Eine Modifikation nennt ihre Wirkung je Einheit (`AddCardPoints(1)`, `RankBonus(1)`, `AddBaseValue(1)`, `AddLevels(1)`, `GainCoins(1)`, `MultiplyTrickPoints(1)`), die Karte trägt den Wert (`DeckCard::mod_values`, ohne Eintrag 1), der Kern vervielfacht beim Wirken (`Effect::scaled`). `values` an der Modifikation sind die angebotenen Werte, der erste gilt für Zufallsziehungen; `max` begrenzt die Steigerung. Texte tragen den Wert als Platzhalter (`{n}` Betrag, `{+n}` mit Vorzeichen).
- **Augen unter 0.** Augen einer Karte dürfen unter 0 fallen (Ramponiert auf einer Lusche: −5). Wer den Stich macht, verliert die Augen. Die Grenzen 61, 90 und 30 bleiben fest, die Gesamtaugen können unter 120 liegen; die Augen einer Seite fallen nie unter 0. Im Kern hat die Karte 0 Augen und eine Schuld (`DealtCard::point_debt`), die die Stichzählung abzieht. Die KI sieht nur die 0.
- **Erster Stich.** Gewinnt die Karte den ersten Stich des Spiels, zählen dessen Augen für den Gewinner doppelt (der Zuschlag liegt in `Game::bonus_points`, Wirkung `MultiplyTrickPoints`, Bedingung `ThisCardWinsFirstTrick`). Im geteilten Blatt auch für einen Gegner, der damit sticht.
- **Letzter Stich** ist die alte „Spitze“ ohne „egal, wer sie spielt“ im Text; „Rest abkürzen“ wartet wie bisher, bis entschieden ist, wer den letzten Stich macht.
- **Trumpf und Fleckig, „hält“:** die Karte liegt in seiner Hand oder im Skat (der Skat gehört dem Alleinspieler, ADR 0015) oder er hat sie in einen Stich gespielt. Geprüft bei der Ansage (`ThisCardIsTrumpSuit` mit dem Schalter oben).
- **Gutschein, „wenn gedrückt“.** Wirkt außerhalb des laufenden Spiels: Landet die Karte beim Drücken im Skat, wird der Gutschein sofort eingelöst, egal wie das Spiel ausgeht, und fällt danach von der Karte (Auslöser `OnDiscard`, Ereignis `VoucherRedeemed`). Bei „Drei drücken“ löst die Karte, die an den Gegner zurückgeht, nichts ein. Im Handspiel wird nichts gedrückt, der Gutschein bleibt. Die Spezial-Gutscheine machen die Karte zur Spezialkarte (Wirkung `TurnInto`); eine andere Spezialkarte auf derselben Karte wird ersetzt.
- **Zufallsziehungen** (Stempelkissen, Kartengeber) ziehen aus Rand, Moment und Gutschein, gewichtet nach `focus_weights` wie bisher, nie einen Fluch und nie direkt eine Spezialkarte (die kommt über den Gutschein); der Wert ist der erste aus `values`. Liegt auf der Karte schon dieselbe Modifikation, steigt sie; eine andere wird ersetzt.
- **Tresen.** Konkrete Werte gibt es als Verbrauchsgegenstände (`ApplyCardMod`): Augen +3, Augen +5, Trumpf +2, Trumpf +4, Rang +1. Platzhalter (12, 17).
- **Durchsichtig (Fluch).** Das Wissen hängt an der Karte. Liegt sie beim Spieler (Hand oder Skat), kennen die KI-Gegner sie: Die Stichproben der PIMC-Gegner legen sie in seine Hand oder in den Skat (`opponents::defender_hints`); die Heuristik nutzt das nicht. Liegt sie bei einem Gegner, sieht der Spieler, wer sie hat, und seit M9 auch die Ausgabe des Gegners auf dieser Karte (die aufgedeckte Karte in `GameView::revealed` ist die Karte des Halters mit dessen Modifikationen und Rückseite; im Kern merkt sich das Spiel die Karten, die die Ausgabe des Spielers markiert, `ActiveGame::player_marked`). Eine Boss-Ausgabe kann Durchsichtig auch tragen (Schwarzer Peter): Dann sieht der Spieler die Karte, solange der Boss sie hält. Der Kiebitz-Mörder schaltet alle Richtungen ab.
- **Flüche** kommen weiter nur über Ereignisse, die sie ankündigen (Umgekipptes Bier: Ramponiert; Falschspieler: ein zufälliger Fluch), und zahlen beim Erhalt 3 Groschen (Platzhalter), einmal, nicht mal dem Wert („beim Erhalt“ wird nicht vervielfacht). Gezahlt wird nur, wenn die Karte danach wirklich den Fluch trägt (Ramponiert −5 auf Augen +8 ergibt Augen +3 und zahlt nichts).
- **Ruhende Wirkungen.** Der Hook-API bleiben `NeverWins`, `HonoraryJack` und `NoDiscard` erhalten (Regeln in 4.4), kein Inhalt nutzt sie. Gestrichen samt Code sind `MultiplyCardPoints`, `BreakThisCard`, `ThisCardBeatenByDefender` und `Homesick` (Glasdaus, Heimweh).

**Umrechnung alter Spielstände** (beim Laden, `Deck` ohne `mods_version: 2`, `core/src/deck.rs`):

| Alt | Neu |
|---|---|
| Fett | Augen +4 |
| Goldrand | Augen +8 |
| Bleigewicht | Augen +10 |
| Glasdaus | Augen +11 |
| Gezinkt | Rang 1 |
| Doppelt gezinkt, Aufgebrezelt | Rang 2 |
| Stempel | Trumpf +3 |
| Krone | Trumpf +6 |
| Spitze, Siegel, Glückspfennig, Schlusslicht | Letzter Stich |
| Bierfleck | Ramponiert −5 |
| Schwarze Katze | Fleckig −3 |
| Durchsichtig | bleibt |
| Untot, Wandelkarte | bleiben (Spezialkarten) |
| alles andere (Hufeisen, Maßband, Angeknackst, Ramponiert alt, Unter ehrenhalber, Unsichtbar, Attrappe, Heimweh) | fällt weg |

Bleiben auf einer Karte mehrere normale Modifikationen übrig, werden gleiche zusammengelegt (Werte addiert, Rang höchstens 2, bis 02.11.2026 3), dann gilt: ein Fluch vor allem anderen (Ramponiert vor Fleckig vor Durchsichtig), sonst die stärkste nach der festen Reihenfolge Rang, Trumpf, Letzter Stich, Augen. Spezialkarten bleiben daneben. Offene Angebote alter Spielstände (Ereignis mit einer alten Modifikation) werden genauso umgerechnet; was wegfällt, wird beim Wählen neu gezogen. Tests in `core/src/run/effects_v2_tests.rs`.

**Untot (umgesetzt am 02.11.2026, Idee des Nutzers, unverändert).** Text: „Karte wird gespielt, dann aber wieder auf die Hand genommen und eine zufällige Karte abgeworfen. Einmal pro Spiel, nur im Farbspiel und Grand.“ (`undead`, Effekt `Undead`). Seit Effekte v2 eine Spezialkarte: Sie kommt über den Gutschein Untot und darf zusätzlich eine normale Modifikation tragen. Auslegung, entschieden von Claude am 02.11.2026:

- **Regel:** Spielt der Spieler die Karte und ist der Stich voll, wird der Stich nach den normalen Regeln mit ihr entschieden, und die `OnTrickEnd`-Hooks feuern (Erster Stich, Luschenkönig, Schenken, Revolutionär sehen den Stich, wie er gespielt wurde). Danach steigt sie zurück in seine Hand, und eine andere Karte seiner Hand, gezogen mit dem Zufall des Spiels, nimmt ihren Platz im Stich ein (`Game::raise`, Ereignis `CardRose { seat, card, substitute }`). Der Stich behält seinen Gewinner; wer ihn macht, bekommt die Augen der Ersatzkarte, nicht die der untoten Karte. Die Ersatzkarte gilt als gespielt: Sie liegt im Stich, für Zählen, Augen und alles, was später auf die Stiche schaut. So bleibt jede Zahl stimmig: Jede Hand gibt pro Stich eine Karte ab, zehn Stiche mit je drei Karten, jede Karte endet in genau einem Stich.
- **Einmal pro Spiel** je Karte (`ActiveGame::risen`), nicht in jedem Stich: Eine Karte, die jeden Stich macht und immer wiederkommt, wäre ein Daus, der das ganze Spiel gewinnt. Die Variante „in jedem Stich“ ist nicht gemessen (siehe 17).
- **Nicht als letzte Karte:** Ist sie die letzte Karte seiner Hand, bleibt sie im Stich. Deshalb steigt sie nie im zehnten Stich, und Letzter Stich bleibt unberührt.
- **Nur im Farbspiel und Grand** (`focus: Trump`). Im Null verliert der Spieler mit dem ersten Stich; eine Ersatzkarte würde dort die offene Hand (Ouvert) und die Stilboni nach Zufall ändern. Im Ramsch spielt jeder für sich, und die Augen der Ersatzkarte entschieden den Verlierer nach Zufall. In beiden ist sie eine normale Karte.
- **Nur in der Ausgabe des Spielers** (ADR 0015): Gegner haben ihre eigene Ausgabe der Karte.
- **Spitzen** bleiben, wie angesagt. **Schneider und Schwarz** zählen wie immer über Augen und Stiche; die Stichzahl ändert sich nie, die Augen um die Differenz der beiden Karten.
- **Zufall:** Beim Geben wird aus dem Zufall des Spiels ein eigener Strom abgezweigt, wenn eine untote Karte im Spiel ist (`ActiveGame::undead_rng`). Spielstände von vorher leiten ihn aus dem Seed ab.
- **Rest abkürzen** wartet, solange eine Karte der Hand noch steigen kann. Eine untote Wandelkarte kommt als die festgelegte Karte zurück und bleibt bis zum Spielende so. **Drei drücken, Kreide** wirken vor dem ersten Stich und ändern nichts an der Regel.
- **KI:** Die Suche kennt das Steigen nicht; sie spielt die Karte wie jede andere. Bot (`RunBot::undead_card`): Spielt er als Letzter zum Stich und nimmt eine untote Karte, die noch steigen kann, den Stich, spielt er sie. Sicht: `CardView::can_rise`.

Tests: `core/src/run/undead_tests.rs`, je Regelkante einer.

**Wandelkarte** (`wild_card`, Effekt `Wild`, seit Effekte v2 eine Spezialkarte über den Gutschein Wandelkarte). Regel neu entschieden vom Nutzer am 03.11.2026, Einzelheiten von Claude; sie ersetzt das Benennen beim Ausspielen (bis dahin `RunAction::PlayAs`, `rules::wild_suits`, Ereignis `WildCardNamed`, „zwingt nie zum Bedienen“). Text: „Bei Spielbeginn nimmt sie die Farbe an, von der du die meisten Karten hältst (ohne Unter). Bei Gleichstand gilt Eichel, Grün, Rot, Schellen.“

- **Wann:** im Moment des Spielbeginns, also nach Aufnehmen und Drücken mit der Ansage, im Handspiel mit der Ansage, auch im Null und im Ramsch (`Game::declare`). Bis dahin ist sie ihre gedruckte Karte, auch bei der Wahl der Drückkarten. Eine gedrückte Wandelkarte (und eine im Skat eines Handspiels) bleibt gedruckt.
- **Welche Farbe:** die Farbe, von der der Spieler in seinen zehn Karten die meisten hält, gezählt nach gedruckter Farbe, ohne alle Unter und ohne die Wandelkarte selbst (`rules::settled_wild_suit`). Gleichstand: Eichel vor Grün vor Rot vor Schellen. Die Taktik dahinter: die zweitlängste Farbe drücken, damit die Wandelkarte in die Farbe kommt, die Trumpf werden soll. Mehrere Wandelkarten (selten, höchstens eine je Ausgabe kommt später) zählen alle nicht mit und nehmen alle dieselbe Farbe an.
- **Was sich ändert:** Rang, Augen und die übrigen Modifikationen bleiben; die Karte ist bis zum Spielende die Karte dieser Farbe mit demselben Rang (in Hand, Deck und allen Ausgaben). Doppelte Karten sind erlaubt wie bei der Kreide (4.4). Ab dann gilt die normale Bedienpflicht für die neue Farbe; nichts wird beim Ausspielen benannt, und sie befreit nicht vom Bedienen. Eine Wandelkarte, die ein Unter ist, bleibt der Unter, unverändert. Die Spitzen bleiben, wie angesagt (gezählt über die gedruckten Karten). In der Hand eines Gegners ist sie die gedruckte Karte.
- **Ereignis und Sicht:** `Event::WildCardSettled { old, new }` (auch für den Spieler, `PlayerGameEvent::WildCardSettled`), je Wandelkarte eines, direkt nach `Declared`; die Web-App kann es animieren. `DealtCard::wild` heißt seitdem „noch offen“ und fällt beim Festlegen ab. Sicht: `CardView::unsettled` (noch offen, vor Spielbeginn: der Harlekin in vier Farben), `CardView::named` (gedruckte Farbe einer festgelegten Wandelkarte, `ActiveGame::named`).
- **Untot:** Eine untote Wandelkarte (nur aus alten Spielständen, die beide Spezialkarten behalten) steigt als die festgelegte Karte zurück.
- **Bot:** Nach dem Festlegen ist sie eine Karte wie jede andere; Suche und Bot sehen die festgelegte Karte (`opponents::tame_game` und `RunBot::wild_suit` gestrichen). Beim Drücken und bei der Ansage rechnet der Bot noch nicht voraus, welche Farbe sie annimmt.
- **Spielstände:** Ohne neue Version. Ein laufendes Spiel vor der Ansage legt sie bei der Ansage fest. Ein Spiel, das vor dem 03.11.2026 schon lief und noch eine Wandelkarte in der Hand hat, behält sie als gedruckte Karte (sie bedient dann normal); die Sicht zeigt sie nicht mehr als offen. Die Aktion `PlayAs` gibt es nicht mehr; alte Ereignisse `WildCardNamed` werden nicht mehr gelesen.

**Spezialkarten im Run** (entschieden vom Nutzer am 03.11.2026, noch nicht umgesetzt): (A) Ein Boss hat Spezialkarten in seiner Ausgabe; beim ersten Erhalt gibt es einen Hinweis, und die Wirkung wird deutlich animiert. (B) Die Boss-Belohnung „1 aus 3“ enthält immer eine Spezialkarte, selten auch die Tresen-Auslage; höchstens ein Untot und eine Wandelkarte je Ausgabe. Eigene Relikte für Spezialkarten gibt es nicht.

**Wirkt sie in diesem Spiel?** (01.11.2026, gilt weiter): Nach der Ansage sagt der Kern für jede sichtbare Karte, ob ihre bedingten Modifikationen in diesem Spiel wirken (`CardView.mod_status`: `Active`, `Inactive`, `Open`; `card_mod_status`). Bedingt sind Trumpf und Fleckig („Ist ihre Farbe Trumpf“) und die Moment-Modifikationen (erster und letzter Stich). Es zählt nur, was der Spieler weiß.

### 7.3 Karten hinzufügen und entfernen (entfällt seit ADR 0015)

Bis 02.11.2026 geplant und teils umgesetzt: Luschen entfernen (eine Blanko tritt an ihre Stelle), Fünfter Bube, Doppel-Ass, „Sechs“, Joker, Kopieren einer Karte samt Modifikationen; ein Talon über 32 Karten.

Seit ADR 0015 verlässt keine Karte das Blatt und keine kommt hinzu. Was davon bleibt, ist ein Karteneffekt auf der Ausgabe des Spielers (entschieden von Claude am 02.11.2026, Frisches Deck als Frage im Cockpit):

| Bisher | Mit eigener Ausgabe |
|---|---|
| Frisches Deck: eine Karte entfernen, eine Blanko tritt an ihre Stelle | **Reinigen** (Antwort im Cockpit, umgesetzt in M9): „Reinigt eine Karte deiner Ausgabe: Alle Effekte fallen ab, auch Flüche.“ Auch Spezialkarten und Abgestuft fallen ab (Aktion `CleanseCard`, `Edition::cleanse`, Ereignis `DeckChange::Cleansed`; eine Karte ohne Modifikation ist kein Ziel, Fehler `NothingToCleanse`). Der Bot kauft es nur, wenn eine Karte einen Fluch trägt, und reinigt sie sofort |
| Luschen entfernen, damit mehr Augen auf die Hand kommen | entfällt; der Hebel Augen läuft über die Modifikation Augen und Hausregeln |
| Fünfter Bube | entfällt (Unter ehrenhalber ist mit Effekte v2 gestrichen) |
| Doppel-Ass, Joker | Kreide (8.3) und Wandelkarte (7.2): eine eigene Karte wird für ein Spiel zu einer anderen; doppelte Karten im Stich wie in 4.4 |
| Kopieren | Idee für `content`: „Abklatsch“, die Modifikationen einer eigenen Karte auf eine zweite übertragen |
| Startblatt des Nullspielers (ohne Unter, vier Siebener mehr) | Modifikation auf seinen Untern (9.2) |
| Glasdaus zerbricht und verlässt das Blatt | entfällt, der Glasdaus ist mit Effekte v2 gestrichen (7.2) |

Blankos gibt es weiter, aber nur als Wirkung auf eine Karte: Die Oma macht für ihre Serie jeden Unter zur Blanko (Tischregel, jede Ausgabe), Scherben gibt es nicht mehr (Glasdaus gestrichen, 7.2). Ein Stich bleibt immer einer von zehn.

### 7.4 Das Risiko sichtbar machen

Bis ADR 0015 zeigte das UI bei jeder Blatt-Änderung die Wahrscheinlichkeit, dass die Karte in der eigenen Hand oder im Skat landet (12 von n Karten, 37,5 % im Grundblatt, weniger mit Talon). Der Kern rechnet das aus (`DealOdds`, Feld `deal_odds` der Sicht, wasm-Funktion `dealOdds`).

Seit ADR 0015 ist das Blatt immer 32 Karten groß, die Chance also immer 12 von 32: „Deine Ausgabe spielt in 37,5 % der Spiele (Hand oder Skat). Sonst hält ein Gegner die Karte in seiner eigenen.“ `DealOdds` bleibt als Sicht, rechnet seit M9 nur noch mit 32 (`deal_odds()` ohne Blattgröße, kein Talon). Das Risiko, das der Spieler spüren soll, ist nicht mehr „hilft dem Gegner“, sondern „schläft“ (7.1, Regel 9); die Anzeige beim Stempelkissen sagt das in einem Satz.

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

**Welle 2b (M8, umgesetzt am 02.11.2026).** Drei Hausregeln aus derselben Abstimmung, die neue Handgriffe am Tisch brauchen. Namen und Texte wie abgestimmt; was der kurze Text offenlässt, steht als zweiter Absatz auf der Infokarte (`de.relicRules` in der Web-App). Auslegungen, entschieden von Claude am 02.11.2026:

- **Drei drücken** (`three_discard`, Effekt `DiscardThree`, Rare): „Einmal pro Serie: Du nimmst drei Karten aus einem größeren Talon und drückst drei.“ Ein Skat aus drei Karten geht mit 32 Karten nicht sauber: 10 + 10 + 10 + 3 sind 33. Erwogen und verworfen: (1) eine 33. Karte aus dem Talon (7.1), im Grundblatt also eine Blanko, die zufällig bei irgendwem landet; das Relikt wäre im Grundblatt fast wirkungslos oder ein Zufallsgeschenk, und der Skat hat im ganzen Kern genau zwei Plätze (Ereignisse, Sichten, KI-Stichproben); (2) dem Alleinspieler eine Karte weniger geben, dann spielt er neun Stiche. Gewählt ist die nächste Variante, die die Regel biegt und die Hände bei zehn lässt: **Beim ersten Aufnehmen der Serie kommt zu den zwei Skatkarten eine zufällige Karte des linken Gegners** (Platz 1, der nach dir spielt). Der Spieler hält 13 Karten und drückt drei. **Die Karte mit den wenigsten Augen bekommt der linke Gegner zurück**, die zwei anderen liegen im Skat und zählen wie immer für den Alleinspieler (`rules::returned_discard`; bei gleichen Augen eine Blanko zuerst, dann der niedrigere gedruckte Rang 7, 8, 9, dann die niedrigere Farbe Schellen, Rot, Grün, Eichel). Fest statt frei gewählt, weil so die Oberfläche nur drei gleiche Plätze braucht und der Spieler trotzdem steuert: Wer eine bestimmte Lusche zurückgeben will, legt sie zu zwei Karten mit mehr Augen. Hebel: Augen im Skat und Spitzen. Die Spitzen zählen über die zehn behaltenen Karten und den Skat (eine gezogene Eichel-Unter zählt, wenn sie bleibt). Die gezogene Karte wird mit einem eigenen Zufall aus dem Seed des Spiels bestimmt (das Geben und alle anderen Ziehungen bleiben gleich), also deterministisch und auch nach Speichern und Laden dieselbe. Der Gegner weiß, was er zurückbekam; der Spieler sieht es in der Sicht beim Gegner (`RevealSource::Returned`), Wertvorschau und Zähler wissen es. Nur nach dem Aufnehmen: Im Handspiel, im Ramsch und gegen den Schieber (nur Hand) wirkt es nicht. Ramponiert gilt für drei wie für zwei (`valid_discard_of`). Altes Blatt zählt weiter +2, weil zwei Karten in den Skat gedrückt werden; die dritte geht zurück. „Selten und teuer“: Rare und gesperrt hinter der neuen Freischaltung „Stammkunde“ (zehn Runs zu Ende spielen, `ten_runs`), damit die Anfangsauswahl gleich bleibt und der Anteil gesperrter Relikte bei einem Drittel. **Einmal pro Serie** (Entscheidung des Leads am 02.11.2026 nach der Messung): In jedem Spiel brachte die Regel +18 Prozentpunkte auf „Abschnitt 3 erreicht“, dreimal das bisher stärkste Relikt. Jetzt wirkt sie nur beim ersten Aufnehmen des Skats in jeder Serie (`Series::discard_three_used`, mit jeder neuen Serie zurück; ein Handspiel verbraucht sie nicht). Der Spieler sieht, ob sie noch frei ist: „1×/Serie“ am Bierdeckel (verblasst, wenn genutzt), Zustand auf der Infokarte, „Drei drücken (1×/Serie)“ über den drei Plätzen (Sicht `discard_three_ready`).
- **Spickzettel** (`crib_sheet`, Effekt `RevealSkatCard(1)` bei der Ansage, Bedingung Handspiel und kein Ramsch): Nach der Ansage eines Handspiels (auch Null Hand) zeigt er die erste der zwei Skatkarten (`ActiveGame::skat_shown`, Ereignis `SkatCardShown`). Reine Information (Kiebitz-Mörder schaltet sie ab), die Augen zählen wie immer. Die Karte liegt am Tisch offen neben dem Stich, die andere verdeckt, solange sie im Skat liegt (Oma und Opa können sie noch herausnehmen). Wertvorschau und Zähler zählen sie als bekannt: Liegt dort ein Unter, zeigt die Vorschau die Spitze mehr. Nach dem Aufnehmen wirkt er nicht, da kennt der Spieler den Skat ohnehin.
- **Lockvogel** (`decoy`, Effekt `DemandLead(1)`): Einmal pro Spiel nennt der Spieler, wenn er am Zug ist, Trumpf oder eine Farbe (`rules::LeadSuit`; Farbe ohne die Karten, die in diesem Spiel Trumpf sind, im Farbspiel ohne die Trumpffarbe, im Null kein Trumpf). Der nächste Gegenspieler, der einen Stich eröffnet, muss sie ausspielen, wenn er eine solche Karte hat; sonst spielt er frei aus (`rules::legal_leads`). Die Forderung wird von diesem Ausspiel verbraucht, erfüllt oder nicht, und gilt nur für Gegenspieler (spielt der Spieler selbst aus, wartet sie). „Wenn du am Zug bist“ statt „wenn ein Gegner ausspielen soll“, weil der Tisch die Gegner ohne Pause spielen lässt; den ersten Stich eines Gegners in Vorhand kann der Lockvogel deshalb nicht lenken. Hebel Stichgewinn: Der Gegner spielt in das eigene Daus oder den höchsten Trumpf (Zählen lohnt sich, Designsäule 3). Die Gegner-KI hält sich daran über die legalen Karten; der PIMC-Gegner sucht dann nur unter ihnen (seine Suche ordnet die Ergebnisse jetzt nach Karten zu). Eine noch wartende Forderung kennt die Suche für spätere Stiche nicht. Im Ramsch erlaubt (dort ist jeder Gegner). Der Bot nennt auf seinem Zug eine Fehlfarbe, deren höchste noch ausstehende Karte er hält, solange die Gegner mindestens zwei Karten der Farbe haben können (die mit den meisten offenen Augen), sonst Trumpf unter derselben Bedingung; im Null eine Farbe, in der seine niedrigste Karte unter allen offenen liegt.

Einzeln gemessen wie Welle 2a (`skat_sim --runs 500 --no-shop --bot pimc:12:3 --relic <id>`, Seeds 42 bis 44; Lockvogel und Spickzettel auf den Zielwerten vor PR #47, ohne Relikt 40,6 / 11,9 / 3,5 %; Drei drücken auf den neuen), Bericht `docs/balance/2026-11-02-relikte-welle2.md`. Seltenheit und Preise sind Platzhalter für `balance`:

| Hausregel | Abschnitt 2 / 3 / 4 | Δ Abschnitt 3 | Seltenheit, Preis | Anmerkung |
|---|---|---|---|---|
| Drei drücken (1×/Serie) | 46,3 / 16,8 / 5,5 | +7,9 | Rare, 9, gesperrt | gemessen auf den neuen Zielwerten (PR #47, ohne Relikt 34,1 / 8,9 / 2,7); in jedem Spiel waren es +18,0 (alte Zielwerte), die Variante „kleinste Karte des Gegners“ +26,4. Eine Karte wechselt die Seite |
| Lockvogel | 42,8 / 13,0 / 3,7 | +1,1 | Uncommon, 6 | wie der Kiebitz; der Bot fordert fast immer, Menschen zählen gezielter |
| Spickzettel | 40,8 / 12,3 / 3,5 | +0,3 | Common, 4 | nur Handspiele (beim Bot 6 % der Spiele); für Menschen |

**Hausregeln mit eigener Ausgabe (ADR 0015, ausgelegt von Claude am 02.11.2026).** Hausregeln sind Regeln des Tisches und gelten für jede Ausgabe (7.1, Regel 7): Alte Dame, Kleinvieh und Dicke Sau zählen die Augen jeder Karte, wer sie auch hält. Skatpfleger zählt die Skatkarten in der Ausgabe des Spielers. **Drei drücken**: Die Karte, die vom linken Gegner kommt, ist in der Hand des Spielers seine Ausgabe (mit seinen Modifikationen); die Karte, die zurückgeht, ist beim Gegner wieder dessen Ausgabe. „Die wenigsten Augen“ (`rules::returned_discard`) misst an den drei Karten, wie der Spieler sie hält, also in seiner Ausgabe.



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

- **Kiebitz** (`kibitzer`, Effekt `PeekDefenderCards(1)`): eine zufällige Karte eines Gegners und wer sie hält, gezogen mit dem Zufall des Spiels (das Geben bleibt gleich). Sie steht in der Sicht unter `revealed` und kommt als Ereignis `CardPeeked`, sonst nichts. Seit ADR 0015 zeigt er die Karte in der Ausgabe des Gegners (beim Boss mit dessen Modifikationen); ebenso der Zaungast (9.5).
- **Wirt** (`landlord`, `ShopDiscount(20)`): gilt für Relikte, Verbrauchsgegenstände und die Neuauslage, gerundet, mindestens 1 Groschen. Verkauft wird weiter zum halben vollen Preis, also kein Gewinn durch Kaufen und Verkaufen.
- **Oma** (`granny`, `SeriesSkatSwap`): Ein gedrückter Bube lässt sich im echten Skat nicht sinnvoll zurücknehmen, weil der Spieler beim Drücken alles sieht. Umgesetzt als: Einmal pro Serie darf der Spieler in einem Handspiel vor seiner ersten Karte einen Buben aus dem Skat nehmen und dafür eine Karte drücken; das Spiel bleibt Hand. Ob ein Bube im Skat liegt, verrät die Oma (`can_take_from_skat`), seit M5 auch welcher (`skat_swap_cards`). Hebel: Hand plus Trumpf. Nur Alleinspieler.
- **Opa** (`grandpa`, M5, `SeriesSkatSwap` für Daus): wie die Oma, aber ein Daus aus dem Skat. Oma und Opa zusammen geben zwei Täusche pro Serie, je einen (4.4).
- **Schiedsrichter** (`referee`, `NoCheating`): schaltet die Boss-Regeln ab, die als Schummeln markiert sind: Zwillinge und Revolutionär.
- **Angeber** (`show_off`, `RequireOuvert` plus eine Stufe bei Ouvert): Für den Alleinspieler hieße „immer Ouvert“ immer Schwarz angesagt, das wäre Selbstmord. Deshalb nur im Pool des Nullspielers (Null Ouvert und Null Ouvert Hand).
- **Stammgast** (`regular_guest`, Zähler `regular` plus `MultiplyGameValue(2)` ab 4): Hebel Spielwert. Der Multiplikator fürs Spätspiel, wer ihn früh kauft, wartet zwei Abende. Nur gewonnene Spiele zählen doppelt.
- **Schneider** (`tailor`, `SchneiderFromDeclarerPoints(80, 89)`): Hebel Schneider, ein Wortspiel mit dem Handwerk. Nur Alleinspieler. Er kam statt des zuerst geplanten Opa, den der Kern in M4 nicht neben der Oma tragen konnte; seit M5 gibt es beide.
- **Zähler** (`card_counter`, `ShowCount`): zeigt Augen beider Seiten aus den Stichen (plus den Skat, sobald der Spieler ihn kennt) und die Summe der Karten, die der Spieler noch nicht gesehen hat. Der Talon zählt dazu, weil niemand weiß, welche Karten dort liegen; so verrät der Zähler nie mehr als ein perfekter Mitzähler weiß. Seit ADR 0015 gibt es keinen Talon; jede ungesehene Karte liegt bei einem der beiden Gegner. Hat der Boss eine Ausgabe mit Augen-Modifikationen, hängt die Summe davon ab, wer die Karte hält: Der Zähler zeigt dann den kleinsten und den größten möglichen Wert („noch 34 bis 38“), damit er nicht mehr verrät als ein Mitzähler weiß.

### 8.3 Verbrauchsgegenstände

Einmal nutzbare Karten, maximal zwei auf der Hand.

- **Frisches Deck**: Entferne eine Karte dauerhaft. Seit ADR 0015 nicht mehr möglich; neue Wirkung offen, empfohlen „eine Karte deiner Ausgabe ist wieder wie neu, alle Modifikationen fallen ab“ (7.3).
- **Stempelkissen**: Verpasse einer Karte eine zufällige Modifikation (seit ADR 0015: einer Karte deiner Ausgabe; die Wahl hat immer 32 Karten). Seit Effekte v2 nach den Regeln aus 7.2: dieselbe steigt, eine andere ersetzt, verfluchte Karten gehen nicht.
- **Augen +3, Augen +5, Trumpf +2, Trumpf +4, Rang +1** (Effekte v2, `ApplyCardMod`): die Modifikation mit diesem Wert auf eine gewählte Karte, Preise 2 bis 4, Platzhalter für `balance` (7.2).
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
- **Kreide ist vorübergehend** (Entscheidung des Nutzers). Seit ADR 0015 malt sie die Ausgabe des Spielers um, nur für dieses Spiel, und nur, solange er die Karte hält oder gespielt hat: Das Blatt des Runs behält die gedruckte Karte, mit dem nächsten Geben ist sie wieder, was gedruckt ist. Einmischen (auch „Neues Blatt“) beendet das Spiel und damit auch die Kreide; „Reue“ bringt das alte Blatt samt Kreide zurück. Im Kern ersetzt `Game::chalk` die Karte überall, wo das Spiel sie führt (Hand, die zwölf Karten des Alleinspielers, die Karten im Spiel, das Blatt), Ereignis `CardChalked`. Doppelte Karten (zwei Grün-Zehnen) sind wie beim veränderbaren Blatt erlaubt: Bei gleicher Stärke behält die früher gespielte den Stich (4.4). Die Spitzen zählen über die umgemalte Karte, weil sie für dieses Spiel diese Karte ist; ein gemalter Trumpf-Daus kann damit eine Spitze mehr bringen.
- **Doppelkorn und Magenbitter** wirken auf die nächste Ansage im Sinne von `after_declaration`, wie „Letzte Runde“: Auch der Ramsch des Ramschers zählt als angesagtes Spiel. Während des Spiels (nach der Ansage) sind sie nicht nutzbar, damit niemand glaubt, sie wirkten noch aufs laufende Spiel.
- **Kurzer** gilt als Information: Gegen den Kiebitz-Mörder ist er nicht nutzbar. Die Wertvorschau zählt den gesehenen Skat als bekannt.
- **Bot** (`run/bot.rs`, einfache Regeln, nicht kalibriert): Kurzer direkt vor dem Aufnehmen des Skats (der Bot nutzt das Gesehene nicht), nach dem Skat Kreide auf die billigste Beikarte in die Trumpffarbe eines Farbspiels, dann Doppelkorn vor einem nicht schwachen und Magenbitter vor einem schwachen Spiel, nie vor einem Null; Kräuter wie Ein Bier. Reue nutzt der Bot nie (Kaufwert 0).
- **Reue** (Name „Reue“ gewählt: kurz und sagt, was er tut). Im Kern hält `ActiveGame::previous` das Spiel vor dem letzten Neugeben vollständig (es wird nicht neu gezogen, also deterministisch und auch nach Speichern und Laden gleich); `ConsumableAction::UndoRedeal` setzt es wieder ein, Ereignis `RedealUndone`. Was das Geben selbst ausgelöst hat (Zähler, Flaggen von `OnDeal`), bleibt; heute hat kein Inhalt so einen Hook. Der Zufallsstrom läuft weiter: Wer nach der Reue noch einmal einmischt, bekommt ein neues Blatt, nicht das zurückgenommene. Kaufwerte in `CONSUMABLE_VALUES` sind Platzhalter.

### 8.4 Stufe Legendär (M8 Welle 3)

Eine vierte Seltenheit für Relikte, die den Run prägen (`Rarity::Legendary`, Anzeige „legendär“). Sie bestimmt nur, wie selten das Relikt am Tresen liegt, nicht den Preis (entschieden am 02.11.2026: Preise nach der gemessenen Wirkung, Staffel aus Welle 2a). Am Tresen weit seltener als Rare: Gewicht `legendary` in `run.ron` als Dezimalzahl neben Common 6, Uncommon 3, Rare 1, Standard 0,3 (ohne Eintrag in `run.ron`, `RarityWeights::percent_weight`). Ein bestimmtes legendäres Relikt liegt damit beim Alleinspieler in etwa 0,6 % der Auslagen. Auch der Stammtisch-Streit und der Alte Bekannte ziehen mit diesem Gewicht. Auf dem Bierdeckel ein goldener Ring mit vier Funken. Preise Platzhalter für `balance`. Entschieden am 02.11.2026:

| Relikt | Effekt | Hebel | Preis |
|---|---|---|---|
| **Skatgott** (`skat_god`) | Jedes gewonnene Spiel mit Schneider: bis zum Run-Ende +1 Stufe auf jedes gewonnene Spiel | Gewinnstufen (Zähler `skat_god`) | 9 |
| **Ehrenvorsitz** (`honorary_chair`) | Du bist immer Vorhand | Stichgewinn (Vorhand) | 6 |

- **Skatgott**: Der Zähler steigt nach jedem gewonnenen Spiel mit Schneider (auch Schneider durch Jungfrau, Schenken, Stichfest oder Schwarz). Die Stufen gelten nur für gewonnene Spiele, jede Spielart (Farbspiel, Grand, Null), nie der Ramsch (`AddLevelsPerCounter` bei der Wertung; der Stufen-Hook steht vor dem Zähler, ein Schneiderspiel zählt seinen eigenen Punkt also nicht). Entschieden am 02.11.2026: „auf alles“ heißt „auf jede Spielart“; die erste Fassung zählte auch verlorene Spiele und brachte dem Bot nur +1,3 bei weniger Punkten je Spiel. Abgestimmter Text ergänzt zu „… +1 Stufe auf jedes gewonnene Spiel“.
- **Ehrenvorsitz**: Der Spieler spielt in jedem Spiel den ersten Stich aus (`AlwaysForehand`); die Gäste sitzen weiter als Mittel- und Hinterhand. Das Geben bleibt gleich, nur die Vorhand wechselt nicht mehr.

Gemessen (`docs/balance/2026-11-02-welle3.md`, wie Welle 2a): Skatgott +4,7 auf „Abschnitt 3 erreicht“ (Abschnitt 4 +3,7, Punkte je Spiel +7,8), so stark wie die Alte Dame und im Spätspiel stärker; Ehrenvorsitz −0,4 (im Rauschen), ein Relikt für Menschen.

---

## 9. Charaktere

Jeder Charakter hat ein eigenes Grunddeck (seit ADR 0015: seine Ausgabe des Skatblatts mit Modifikationen ab Start, nie eine andere Zusammensetzung, 9.8), Startrelikte (oder einen Start-Schnaps) und genau eine eigene Regel; eine eigene Zielwertkurve ist möglich. Fünf Charaktere: der Alleinspieler (frei), der Nullspieler, der Grandspieler, der Zaungast und die Wirtin (je über eine Freischaltung, 9.3).

### 9.1 Der Alleinspieler (Standard)

Normales 32er-Blatt. Spielt Farbe, Grand und seit ADR 0012 auch Null. Null zählt seit dem 01.11.2026 für alle Charaktere nach der Formel mit Stilbonus (9.2); vorher galt beim Alleinspieler die Skatordnung (23, 35, 46, 59). Startrelikt: **Der Zähler**. Dieser Charakter ist der Maßstab für alles Balancing.

Charaktere sind Daten in `core/content/characters.ron`: Startblatt als Abweichung vom 32er-Blatt (`remove`, `add`, seit dem 01.11.2026 auch `mods`: Kartenmodifikationen ab Start; seit ADR 0015 nur noch `mods`, der Ladevorgang lehnt `remove` und `add` ab), Startrelikte, Start-Verbrauchsgegenstände (`start_consumables`), optional eigene Zielwerte, Null-Wertung, die Tresen-Gewichte pro Spielart (`focus_weights`, siehe 9.2) und die eigene Regel (`rule`, `CharacterRule`). Jeder Charakter darf jedes Spiel ansagen (ADR 0012, ersetzt 0003); im Code unterscheidet kein Zweig mehr die Charaktere.

**Die eigene Regel** (`rule`) hat zwei Teile, beide Daten: `passive` und `hooks` sind dieselbe Hook-API wie bei Relikten (Abschnitt 8) und wirken wie ein Relikt, das der Charakter immer hat, ohne Platz zu belegen; in der Wertung steht die Zeile mit der Quelle `Source::Character`. `passes`, `relic_slots` und `payout_factor` ändern die Run-Struktur aus `run.ron`: Pässe pro Serie und Relikt-Plätze als Zuschlag (negativ: weniger, nie unter 0), die Groschen einer gewonnenen Serie als Faktor (`Payout::bonus` ist der Zuschlag). Die Einsatz-Leiter (5.4) gilt pro Charakter von selbst (`MetaState::stakes` ist nach Charakter geschlüsselt).

### 9.2 Der Nullspieler

Seit ADR 0012 ein Null-Spezialist, der auch Farbe und Grand ansagen darf (siehe „Null für alle“ am Ende dieses Abschnitts). Seit dem 02.11.2026 mischt er dreimal statt zweimal pro Spiel ein, weil sein Blatt ohne Unter sonst zu selten einen ganzen Abend trägt (5.2). Ursprünglicher Entwurf (ADR 0003): Spielt ausschließlich Null. Rangfolge 7, 8, 9, 10, B, D, K, A ohne Trumpf. Ziel: keinen einzigen Stich machen.

**Die Formel muss hier neu gebaut werden**, denn Null hat im echten Skat feste Werte (23, 35, 46, 59) statt Grundwert × Stufen. Vorschlag:

```
Spielwert = 23 × Risikostufen × Stilbonus

Risikostufen: Normal 1, Hand 2, Ouvert 3, Ouvert Hand 4
Stilbonus:    +1 je Stich, den du mit einer 7 „unterlaufen“ hast
              +2 wenn du nie bedienen musstest
              +3 „Kurzschluss“: Gegner haben nach 5 Stichen alle Augen
```

Das Startblatt des Nullspielers hat keine Buben, dafür vier zusätzliche Siebener (bis ADR 0015; jetzt eine Modifikation auf seinen Untern, 9.8). Seine Upgrades drehen die Logik um: Karten werden „leiser“ statt stärker, Luschen sind Gold, Asse sind Flüche. Der Tresen bietet andere Modifikationen (z. B. „Unsichtbar“: diese Karte kann nie einen Stich gewinnen).

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

Mit den festen Werten verliert der Nullspieler fast jeden Run: Ein gewonnenes Null bringt 23, ein verlorenes kostet 46, und im Null-Blatt mit acht Siebenern gewinnt der Bot nur 42 % seiner Nullspiele. Mit der Formel kommt er so weit wie der Alleinspieler, obwohl er seltener gewinnt: Der Stilbonus (vor allem unterlaufene Siebener, das Null-Blatt hat acht) hebt ein gewonnenes Null weit über 23, ein verlorenes kostet weiter nur 46 je Risikostufe. Dazu hat die Formel Raum zum Skalieren: Grundwert-Relikte (Tournee) wirken auf 23, Stufen-Relikte (Ouvert-Bonus, Blindflug, Angeber) auf eine Summe von 1 bis 4 plus Stil, statt wie bei `Fixed` mit einer einzigen Stufe jedes Mal den Spielwert zu verdoppeln. Der Nullspieler nutzte deshalb die Zielwerte des Alleinspielers (`targets: None`); seit den Serien mit 3 Spielen hat er eigene, rund die Hälfte (5.2, 02.11.2026). Der Bot spielt ein Null, wenn höchstens vier Karten „offen“ sind (klassische Null-Regel: die k-te kleinste Karte einer Farbe ist sicher, wenn höchstens k−1 kleinere Karten draußen sind), ohne offene Karte Null Ouvert Hand.

**M5: eigene Zielwerte** (bis ADR 0012, 5.2, `targets` in `characters.ron`): Mit 8 Spielen pro Serie und Relikten (Bockrunde, Siebenschläfer, Lehrgeld) bringt ein gewonnenes Null meist 92 bis über 200 Punkte; mit Zielwerten nah an denen des Alleinspielers (36 / 46 / 66 in Abschnitt 1, 99 bis 198 in Abschnitt 4 und 5) gewann der Bot jede Serie zu 76 bis 86 % und kam in 3,8 % der Runs durch (Alleinspieler 1,4 %, je 2 × 1000 Runs). Der Bot spielt seit M5 bis zu fünf offene Karten (`NULL_MAX_EXPOSED` in `core/src/run/bot.rs`): mit 4 erreichte er Abschnitt 3 in 20,1 %, mit 5 in 25,1 % (vorherige M5-Messung, 3 × 600 Runs); er steht für einen guten Spieler.

**Null für alle (ADR 0012, umgesetzt in M6).** Jeder Charakter darf Farbspiel, Grand und Null ansagen, in der Reihenfolge der Skatordnung (ADR 0011): aus der Hand Null Hand und Null Ouvert Hand, nach dem Skat Null und Null Ouvert. Was die Charaktere unterscheidet, sind nur noch Daten:

- **Ansage**: Der Dialog zeigt jedem Charakter sechs Spielarten (Eichel, Grün, Rot, Schellen, Grand, Null); bei Null die Null-Stufen dieser Phase mit Wert (`DeclareOptions.null_risks`). `null_style` sagt, ob ein gewonnenes Null einen Stilbonus bekommt. Beide Charaktere werten Null mit der Formel oben (entschieden am 01.11.2026: eine Null-Wertung für alle, damit dasselbe Spiel überall gleich zählt; `Fixed` bleibt als Datenoption). Ramsch gibt es weiter nur beim Ramscher.
- **Der Nullspieler bleibt Spezialist**: Startblatt ohne Unter mit acht Siebenern (seit ADR 0015 ersetzt, 9.8), Kiebitz, Formel-Wertung, und am Tresen öfter Null-Relikte. Er darf auch Farbe und Grand; ohne Unter ist jedes seiner Trumpfspiele „ohne 4“.
- **Gemeinsamer Pool**: Jedes Relikt und jede Modifikation liegt bei jedem Charakter (die Null-Hausregeln auch beim Alleinspieler; die Null-Modifikationen sind mit Effekte v2 gestrichen, 7.2). Das neue Feld `focus` (`GameFocus`: `All`, `Trump` für Farbspiel und Grand, `Null`) sagt, für welche Spielart ein Inhalt gemacht ist. `focus_weights` in `characters.ron` gewichtet damit Tresen, Stempelkissen und Ereignisse in Prozent der Seltenheit: Alleinspieler Trump 100, Null 50; Nullspieler Trump 50, Null 200; `All` immer 100. Kein Code-Zweig pro Charakter. Relikte für Trumpfspiele, die im Null Unsinn täten (Spitze, Schenken, Auf dem Trockenen), schließen Null in ihren Bedingungen aus (`Not(NullGame)`). Der Angeber verlangt Farbe oder Grand Ouvert Hand, sonst Null Ouvert.
- **Bot** (`core/src/run/bot.rs`): Aus der Hand spielt er Null Ouvert Hand ohne offene Karte, Farbe oder Grand Hand mit acht Trümpfen oder einer Ansage; sonst nimmt er den Skat auf. Mit zwölf Karten vergleicht er das Farbspiel oder den Grand der Heuristik mit Null (die zwei offensten Karten gedrückt, Null Ouvert ohne offene Karte) nach dem erwarteten Ergebnis ohne Relikte (`suit_expected_score`, `null_expected_score`; Gewinnquoten `SUIT_WIN_PERCENT`, `GRAND_WIN_PERCENT`, `NULL_WIN_PERCENT`, gemessen mit dem PIMC-Spieler). Verloren zählt doppelt. Ein Blatt, das für Farbe und Grand schwach ist und mehr als fünf offene Karten hat, wirft er weg. Unter dem Schieber (nur Hand) spielt er Null Hand nur mit höchstens zwei offenen Karten (`NULL_HAND_MAX_EXPOSED`), die er ohne Skat nicht loswird; beim Schwarzen Peter nie Null, weil ein gewonnenes Null nie Schneider ist und nichts zählt.
- **Alte Spielstände** laden weiter: Die neuen Felder (`focus`, `focus_weights`, `character_factors`) haben Standardwerte, der Charakter im Spielstand bleibt gültig. `RunError::NullNotAvailable` bleibt als Fehlercode stehen, kommt aber nicht mehr vor.

**Messung:** Vorläufig (3 × 3000 Runs, `--bot pimc:12:3`, vor dem Zusammenführen mit dem kurzen Abend): Der Nullspieler erreicht Abschnitt 2 in 34 % der Runs (vorher 37 %), Abschnitt 3 in 14 %. Der Alleinspieler spielt etwa jedes fünfte Spiel als Null. Die endgültige Messung folgt in der großen Testrunde (Entscheidung vom 01.11.2026: Messungen vorerst pausiert, später gesammelt).

### 9.3 Freischaltungen über Runs hinweg

**Seit ADR 0016 (Kneipentour)**: Bedingungen an Abschnitten sind umgeschrieben, die Ids bleiben (verdiente Freischaltungen gelten weiter). Neu ist die Bedingung `ReachPub(n)` (n-te Kneipe des Abends, über die Stadtteile gezählt). Vorläufig, nicht gemessen:

| Id | bisher | jetzt |
|---|---|---|
| `second_evening` („Kneipenbummel“, bisher „Zweiter Abend“) | Erreiche Abschnitt 2 | Erreiche die dritte Kneipe des Abends (`ReachPub(3)`) |
| `third_evening` („Vor dem Boss“, bisher „Dritter Abend“) | Erreiche Abschnitt 3 | Erreiche die Boss-Kneipe im ersten Stadtteil (`ReachPub(5)`) |
| `fourth_evening` („Neues Viertel“, bisher „Vierter Abend“) | Erreiche Abschnitt 4 | Erreiche Stadtteil 2 (`ReachSection(2)`) |
| `early_finish`, `boss_without_pass`, `full_till` | „Serie“ | „Kneipe“ (Bedingungen unverändert) |

`full_till` (15 Groschen nach einer gewonnenen Kneipe) dürfte mit dem höheren Bonus je übrigem Spiel (3 statt 1) leichter werden; zu messen. `tour_won` verlangt weiter einen gewonnenen ganzen Abend (es gibt nur noch den).

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

Seit den Serien mit 3 Spielen (01.11.2026): `early_finish` verlangt eine Serie gleich im ersten Spiel (`WinSeriesWithGamesLeft(2)`, mit drei übrigen wäre sie unerreichbar), `boss_without_pass` heißt „Ohne Einmischen“ (`BeatBoss(without_redeal: true)`, Id unverändert).

Neu gemessen am 02.11.2026 (Serien mit 3 Spielen, Zielwerte aus 5.2, `skat_sim --seed 42 --runs 200 --unlocks --bot pimc:12:3`, 20 Folgen, Mittel der Runs bis dahin): `early_finish` 1,3; `grand_schneider` 2,3; `second_evening` 2,4; `big_game` 3,1; `boss_without_pass` 4,0; `apprenticeship` 5,0; `hand_won` 6,5; `third_evening` 6,5; `hearts_schneider` 8,4; `beat_kibitz_killer` (Zaungast) 9,2; `beat_kontra_hans` 10,1; `big_win` 11,1; `fourth_evening` 11,6; `grand_master` (Grandspieler, 144) 13,1 (1 bis 37); `grand_hand` 19,6; `beat_grandma` 22,6; `beat_black_peter` 24,4; `full_till` (Wirtin) 34,8 (4 bis 152); `tour_won` (Nullspieler) 48,8, 18 von 20 Folgen in 200 Runs. Alle erreichbar; `grand_master` mit 144 liegt jetzt im Mittelfeld statt bei drei Runs, `early_finish` und `boss_without_pass` sind leichte Einstiegsziele.

Verworfen: „Schneider angesagt gewinnen“ für den Schneider (577 Runs, der Bot sagt fast nie an) und „Serie mit vier Spielen übrig“ für Auf dem Trockenen (1,7 Runs, in Serie 1-1 reicht ein Spiel).

**Freischaltungen der neuen Charaktere** (01.11.2026, zählen in beiden Run-Längen, noch nicht im Simulator gemessen):

| Freischaltung | Bedingung | Belohnung |
|---|---|---|
| `grand_master` („Großer Grand“) | Grand Schneider gewinnen, der mindestens 144 Punkte zählt (`WinGame(game: Grand, schneider: true, min_value: 144)`: zum Beispiel mit 4, Spiel 5, Schneider 6; bis 01.11.2026 96, im Testlauf nach rund drei Runs erreicht, zu leicht) | Der Grandspieler |
| `beat_kibitz_killer` („Blinde Kuh“) | Boss Der Kiebitz-Mörder schlagen | Der Zaungast |
| `full_till` („Volle Kasse“) | Nach einer gewonnenen Serie mindestens 15 Groschen haben (Bedingung `SeriesWonWithCoins(15)`, gemessen nach der Auszahlung; bis 02.11.2026 20, gesenkt nach Antwort im Cockpit) | Die Wirtin |

Ein einfacher Grand Schneider ist schon die Freischaltung `grand_schneider` (im Mittel nach 2 Runs, zu leicht); 96 Punkte verlangen eine Stufe mehr. Der Kiebitz-Mörder nimmt jede Information, wer ihn schlägt, verdient den Zaungast, der ohne ihn immer mitschaut. 15 Groschen verlangen Sparen über Zinsen und übrige Spiele (die Messung oben, 34,8 Runs, galt für 20).

**Stammkunde** (02.11.2026, Welle 2b): zehn Runs zu Ende spielen (`RunsFinished(10)`, `ten_runs`) schaltet „Drei drücken“ frei (8.1). Damit sind 16 von 46 Hausregeln und Figuren gesperrt. Nicht im Simulator gemessen; zehn beendete Runs sind wie „Lehrjahre“ (fünf) sicher erreichbar.

Alle Freischaltungen sind erreichbar. Die letzte ist für den Bot fast unerreichbar, weil er Abschnitt 5 kaum schafft; für Menschen ist sie das Ziel des ersten Runs, wie in 9.2 entschieden.

**Kartenrückseiten**: Standard ist die Distel-Rückseite (nach Altenburger Vorbild, eigene Pixelgrafik). Seit ADR 0015 zeigt die Rückseite, wessen Ausgabe eine Karte ist: Die Distel (oder die gewählte) gehört dem Spieler und dem Skat; die Hände der Gegner zeigen eine Gegner-Rückseite, die als Daten am Gast oder Boss hängt (`back` in `guests.ron` und `bosses.ron`, ohne Angabe eine gemeinsame Rückseite der Stammgäste). Die Gegner-Rückseiten sind keine Freischaltung und nicht wählbar. Weitere Rückseiten werden später über bestimmte Erfolge freischaltbar (Freischaltung als Kosmetik, ohne Spielwirkung). Die Web-App führt die Rückseiten als kleine Liste von Entwürfen (`web/src/ui/backs.ts`, `cardBack(className, design)`); die Wahl ist eine reine Anzeige-Einstellung und berührt den Kern nicht.

### 9.4 Der Grandspieler

Entschieden vom Nutzer am 01.11.2026. Lebt für den Grand.

- **Regel**: Ein Grand zählt eine Gewinnstufe mehr, ein Farbspiel eine weniger, gewonnen wie verloren. Daten: zwei `OnDeclare`-Hooks mit `AddLevels(1)` bei `GameIs(Grand)` und `AddLevels(-1)` bei `SuitGame`. Null und Ramsch bleiben unverändert. Nie unter eine Stufe: Die Wertung rechnet `Stufen = max(1, Gewinnstufen nach Regel + Σ Stufen-Zuschläge)` (4.2); ein Farbspiel hat nach der Skatordnung mindestens zwei Stufen (mit oder ohne 1, Spiel 1), nach dem Abzug also mindestens eine, und kommen weitere Abzüge dazu, hält die Formel bei 1.
- **Startblatt**: das 32er-Blatt, der Eichel-Daus (`CA`) mit Augen +8 (`points`, bis Effekte v2 „Goldrand“). Seit ADR 0015 nur in seiner Ausgabe (9.8). Ursprünglich war ein gezinkter Eichel-Unter geplant; der ist aber ohnehin der höchste Trumpf, die Modifikation hätte kaum gewirkt (geändert am 01.11.2026). Startrelikte: keine.
- **Tresen**: `focus_weights` Trump 120, Null 50.
- **Zielwerte**: eigene seit 02.11.2026, vorne rund 1,4-mal der Alleinspieler, hinten flach (5.2).

### 9.5 Der Zaungast

Entschieden vom Nutzer am 01.11.2026 (als „Kiebitz“; der Name ist an die Figur „Der Kiebitz“ vergeben, deshalb Zaungast). Der Zaungast schaut mit.

- **Regel**: Er sieht das ganze Blatt des linken Gegners (Platz 1, spielt nach ihm), jedes Spiel von der Ausgabe an, dafür darf er nur einmal pro Spiel einmischen statt zweimal (seit 01.11.2026; vorher zwei Pässe weniger pro Serie; nie unter 0). Daten: `passive: [RevealDefenderHand(1)]`, `redeals: -1`. `RevealDefenderHand` ist eine Informations-Wirkung wie der Kiebitz: Der Kiebitz-Mörder schaltet sie ab. Die Karten stehen wie die des Kiebitz in `ActiveGame::peeked` (die Sicht nennt sie `RevealSource::Onlooker`, in Anzeige-Reihenfolge), deshalb kennen sie auch Wertvorschau und Bot. Hat er zusätzlich den Kiebitz, zieht dieser nur Karten, die er noch nicht sieht.
- **Anzeige**: Am linken Gegner liegt sein Blatt als kleiner Fächer mit Auge; ausgespielte Karten verschwinden daraus. Kein Flüster-Hinweis pro Karte.
- **Start**: Standardblatt, keine Startrelikte (die Regel ist die Information).
- **Zielwerte**: eigene seit 02.11.2026, Abschnitt 1 wie der Alleinspieler, danach flacher (5.2).

### 9.6 Die Wirtin

Entschieden vom Nutzer am 01.11.2026 (als „Wirt“; der Name ist an die Figur „Der Wirt“ vergeben, deshalb Wirtin). Rechnet gut.

- **Regel**: Jede gewonnene Serie zahlt doppelt so viele Groschen (Grundbetrag, übrige Spiele und Zinsen, die Verdopplung steht als eigene Zeile „Die Wirtin“ in der Auszahlung), dafür hat sie vier statt fünf Relikt-Plätze. Daten: `payout_factor: 2`, `relic_slots: -1`. Groschen aus Relikten nach der Serie (`OnSeriesWon`) verdoppeln sich nicht.
- **Start**: Standardblatt, keine Startrelikte, ein Kräuter (einmal mehr einmischen in einer Serie) im Platz für Verbrauchsgegenstände (`start_consumables`).
- **Zielwerte**: eigene seit 02.11.2026, vorne rund 1,2-mal der Alleinspieler, hinten kaum höher (5.2).

### 9.7 Spätere Kandidaten (nicht vor v1)

- **Der Ramscher**: Spielt Ramsch, gewinnt mit den wenigsten Augen. Eigenes Scoring.
- **Der Schieber**: Sieht immer den Skat vor der Ansage, aber alle Spiele sind Hand.

### 9.8 Grunddecks als Ausgaben (ADR 0015)

Seit ADR 0015 unterscheidet sich das Grunddeck eines Charakters nur durch Modifikationen auf seiner Ausgabe. Ziel der Vorschläge (Claude, 02.11.2026): die Eigenart jedes Charakters erhalten und seine gemessene Stärke (5.2) so wenig wie möglich verschieben. Gemessen wird erst, wenn die Simulationen wieder laufen (18.4).

| Charakter | Grunddeck bis ADR 0015 | Ausgabe ab ADR 0015 | Erwartete Verschiebung |
|---|---|---|---|
| Alleinspieler | 32er-Blatt | normale Ausgabe, keine Modifikationen | keine (Maßstab) |
| Nullspieler | ohne Unter, vier Siebener mehr | seine vier Unter tragen „Abgestuft“ (Antwort im Cockpit, umgesetzt in M9, Auslegung unten) | im Null etwas leichter, im Farbspiel etwas schwerer, siehe unten |
| Grandspieler | Eichel-Daus mit Goldrand | Eichel-Daus mit Augen +8 (seit Effekte v2 statt Goldrand), aber nur in seiner Ausgabe | stärker: Die Gegner bekommen die +8 nicht mehr, wenn sie den Daus halten (20 von 32 Spielen); bei Bedarf hebt `balance` seine Zielwerte |
| Zaungast | 32er-Blatt | normale Ausgabe | keine (seine Eigenart ist die Regel) |
| Wirtin | 32er-Blatt | normale Ausgabe | keine (ihre Eigenart ist die Regel) |

**Der Nullspieler: Ersatz für „ohne Unter, vier Siebener mehr“.** Bisher hatte niemand am Tisch einen Unter, und es gab jede Sieben doppelt. Für seine eigene Hand war das ein Blatt aus 32 Karten mit acht Siebenern; für die Gegner hieß es auch: keine Trümpfe aus Untern und mehr niedrige Karten. Varianten:

- **A (empfohlen): neue Modifikation „Abgestuft“ (`demoted`, nur auf Untern)**: „In deiner Hand ist dieser Unter eine Sieben seiner Farbe: kein Trumpf, keine Spitze, keine Augen.“ Seine Hand zieht damit genau wie bisher aus 32 Karten mit acht Karten, die für ihn Siebenen sind; Spitzen zählt er „ohne“ wie bisher (4.4). Anders als bisher hält ein Gegner, der einen Unter bekommt, einen echten Unter: Im Null ist das eine mittlere Karte, die eher sticht (leichter für ihn), im Farbspiel ein Trumpf gegen ihn (schwerer). Eine Sieben kann doppelt im Stich liegen; bei Gleichstand behält die früher gespielte ihn (4.4). Neue Wirkung im Kern (`Demote`), wie die Kreide eine Karte, die für ihr Spiel eine andere ist.
- **B: „Unsichtbar“ auf seinen vier Untern** (mit Effekte v2 gestrichen, bräuchte eine eigene Modifikation): Die Unter gewinnen nie, bedienen aber. Im Null stärker als eine Sieben (eine Sieben kann einen Stich machen, wenn keiner bedient). Im Farbspiel bleiben sie Trumpf, und weil die Spitzen über die gedruckte Trumpfordnung zählen, zählte er „mit“ Spitzen über Trümpfe, die nie stechen; das müsste eine Ausnahme abfangen. Stärker als bisher.
- **C: Normale Unter, dafür Null-Werkzeuge auf hohen Karten**, etwa „Attrappe“ auf allen vier Daus (mit Effekte v2 gestrichen). Einfach; sein Farbspiel wird aber normal, er wäre ein Alleinspieler mit Null-Vorteil und verlöre seine Eigenart „kein Trumpfspieler“.

**Umgesetzt (M9, Antwort des Nutzers, Auslegung von Claude am 02.11.2026): „Abgestuft“ (`demoted`, Wirkung `Demote`) ist eine Spezialkarte, die ein Fluch ist (`ModKind::SpecialCurse`), genau das Gegenstück zu Rang („sticht Stufen tiefer“).** Nicht als Wert neben Rang wie Augen und Ramponiert, sondern eine eigene Art, damit sie den Platz der Spezialkarte hält und daneben eine normale Modifikation tragen kann.

- **Auf einem Unter:** Er ist in keinem Spiel mehr ein Unter, sondern die niedrigste Karte seiner Farbe, unter der Sieben: im Farbspiel einer anderen Farbe und im Grand eine Farbkarte, im Farbspiel seiner Farbe der niedrigste Trumpf, im Null unter der Sieben. Keine Spitze (er fehlt in der Trumpfordnung wie eine Karte, die die Kreide umgemalt hat), seine Augen sind 0 wie bei einer Sieben (die 2 des Unters fallen weg; Hausregeln, die Augen setzen, wirken vorher, Abgestuft setzt danach auf 0). Er ist keine Sieben: Er bringt im Null keinen Stilbonus „Sieben unterlaufen“. Im Code `rules::is_demoted_jack`, `trump_strength` (Stärke 0), `suit_strength` (0), `matadors_in` überspringt ihn.
- **Auf jeder anderen Karte:** eine Stufe tiefer in ihrer Art (Trumpf oder Farbe), wie Rang −1; bei Gleichstand behält die früher gespielte Karte den Stich. Mit Rang daneben heben sich die Stufen auf.
- **Fluch:** Sie wird nie ersetzt (keine andere Spezialkarte, Fehler `Cursed`), nie zufällig gezogen, zahlt keine Groschen. Eine normale Modifikation darf daneben liegen (etwa Augen +3: der abgestufte Unter zählt dann 3). Frisches Deck nimmt sie mit allem anderen ab.
- **Grunddeck:** das normale Blatt, Abgestuft auf seinen vier Untern (`deck.mods` in `characters.ron`). Hält ein Gegner einen Unter, ist es ein echter Unter. Null-Wertung, Zielwerte, Einmischen und Kiebitz bleiben.

Mit A bleibt die Oma weiter außerhalb seines Pools (sie macht auch seine abgestuften Unter zur Blanko, im Null ein Geschenk, 17). Seine drei Einmischen, Kiebitz, Formel-Wertung und `focus_weights` bleiben.

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
- **Ausgabe** (ADR 0015, umgesetzt in M9): Stammgäste spielen die normale Ausgabe ohne Modifikationen (Antwort im Cockpit). Ihre Karten zeigen eine gemeinsame Gegner-Rückseite (`back` in `guests.ron`, Standard `opponent`; Web: `web/src/ui/backs.ts`).

### 10.5 Was die KI über Ausgaben weiß (ADR 0015)

Ausgelegt von Claude am 02.11.2026:

- **Eigene Ausgabe und die des Partners**: Jeder Gegner kennt seine eigene Ausgabe und die seines Partners (Stammgäste normal, die Boss-Ausgabe ist ohnehin öffentlich). Seine Handkarten sind mit seiner Ausgabe aufgelöst, die Heuristik liest Augen, Rangbonus und „gewinnt nie“ wie bisher an der Karte.
- **Ausgabe des Spielers**: Die KI kennt sie, wie der Spieler die Boss-Ausgabe kennt (7.1, Regel 8). Sie weiß also, dass der Eichel-Daus des Spielers Augen +8 trägt, falls er ihn hält, aber nicht, ob er ihn hält (Ausnahme seit Effekte v2: Durchsichtig verrät ihr, dass er die Karte hält, 7.2). Die Stichproben (`ai::sample`) verteilen die unbekannten Karten wie bisher und lösen jede Karte mit der Ausgabe des Platzes auf, auf den sie fällt; der Skat ist immer die Ausgabe des Spielers. Was der Spieler ausspielt, liegt offen mit seinen Modifikationen im Stich.
- **Wechsel des Halters**: Revolutionär und Drei drücken (bis Effekte v2 auch Heimweh) ändern die Ausgabe einer Karte. Die Sicht eines Platzes (`SeatView`) zeigt jede bekannte Karte in der Ausgabe ihres jetzigen Halters; die Suche löst nach jedem Wechsel neu auf.
- **Wandelkarte, Untot**: weil es sie nur in der Ausgabe des Spielers gibt, entfällt das Abnehmen beim Geben (`tame_defender_wild_cards` gestrichen) und der Sonderweg für die Gegner (`defender_card` spielt das Spiel, wie es ist). Nur der Bot für den Platz des Spielers rechnet weiter ohne Wandelkarte (`opponents::tame_game`). Seit dem 03.11.2026 entfällt auch das: Die Wandelkarte wird bei Spielbeginn festgelegt (7.2).

Umgesetzt in M9: `SeatView::editions` trägt die 32 Karten in der Ausgabe jedes Platzes, `ai::sample` löst jede verteilte Karte mit dem Platz auf, auf den sie fällt (der Skat mit dem Spieler), bekannte Karten sind schon in der Ausgabe ihres Halters. Karten, die sich die KI merkt (Kiebitz, Durchsichtig, die zurückgegebene Karte), werden nach einem Wechsel des Halters über `Game::current` neu gelesen. Die Heuristik zählt einen abgestuften Unter nicht als Unter.
- **Kosten**: Das Auflösen einer Karte ist ein Nachschlagen je Karte und Stichprobe; die Suche bleibt gleich teuer.

---

## 11. Bosse

**Seit ADR 0016** sitzt der Boss in der Boss-Kneipe am Ende jedes Stadtteils (drei je Abend, die letzte der Endboss). Was hier „Boss-Serie“ heißt, ist die Boss-Kneipe; verloren wird sie wiederholt (5.0). Die Messungen unten galten den fünf Abschnitten.

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
| **Der Schwarze Peter** | Gewinnen ohne Schneider zählt nur halb |
| **Der Kiebitz-Mörder** | Alle Informations-Relikte sind deaktiviert |

Endboss nach Abschnitt 5: **Der Alte**, der jede der obigen Regeln nacheinander auflegt, eine pro Spiel.

**Boss-Ausgaben (ADR 0015, Vorschlag von Claude am 02.11.2026).** Jeder Boss hat neben seiner Regel eine eigene Ausgabe: Die Karten, die er bekommt, tragen seine Modifikationen. Leitlinie: ein Thema, eine Gruppe von vier Karten (einer Gruppe gleichen Rangs; er hält im Mittel 1,25 davon), Modifikationen aus Effekte v2 (7.2), eine je Karte. So bleibt eine Boss-Ausgabe auf einen Blick lesbar und verschiebt die Boss-Faktoren wenig. Sie steht vor der Boss-Serie neben der Regel (`upcoming_boss`), seine Karten zeigen seine Rückseite. Nur der Boss-Platz (links) hat sie, der Stammgast rechts spielt die normale Ausgabe. Wie stark die Ausgaben sein sollen, ist eine Frage im Cockpit; die Tabelle ist die milde Stufe.

| Boss | Ausgabe | Thema | Hebel | Wirkung für den Spieler |
|---|---|---|---|---|
| Der Schieber | Siebenen „Augen +4“ | Schmuggelware: Luschen, die doch etwas wert sind | Augen | im Mittel rund +5 Augen bei den Gegnern; jedes Handspiel wird knapper |
| Der Kontra-Hans | Könige „Letzter Stich“ (macht der König den letzten Stich, +1 Stufe) | Hans legt einen drauf | Gewinnstufen | der Einsatz steigt, gewonnen wie verloren, wenn er den letzten Stich mit einem König macht; passt zum immer erlaubten Kontra |
| Die Oma | Oberinnen „Augen +3“ (6 statt 3 Augen; Antwort im Cockpit: +3) | die alte Oberin, wie die Hausregel „Alte Oberin“ | Augen | rund +4 Augen; zur Regel (keine Unter) kommt Gewicht in den Stichen |
| Die Zwillinge | Achten „Rang 2“ (zwei Stufen höher) | die Acht sieht aus wie zwei Nullen übereinander | Stichgewinn | eine Lusche, die so hoch steht wie die Oberin (bei Gleichstand die frühere); Zählen wird schwerer, nicht unmöglich |
| Der Ramscher | Daus „Augen −6“ (Ramponiert; „Unsichtbar“ ist mit Effekte v2 gestrichen; Vorschlag angenommen vom Lead am 02.11.2026) | er lässt laufen | Augen | im Ramsch zählen seine Daus kaum (schwerer für den Spieler, der die Augen meiden will), im Farbspiel und Grand bringen sie den Gegnern weniger (leichter) |
| Der Revolutionär | Zehnen „Rang 1“ (gleich hoch wie der Daus, bei Gleichstand die frühere) | die Zehn stürzt den Daus | Stichgewinn | nach dem Tausch nach dem fünften Stich gehen die Zehnen an den Stammgast und sind dort normal: Die Ausgabe bleibt beim Platz (7.1, Regel 4), der Tausch kostet ihn also etwas |
| Der Schwarze Peter | Eichel- und Grün-Unter „Durchsichtig“ (Ruß an den Fingern; seit Effekte v2 ein Fluch, beim Boss bleibt es ein Erkennungszeichen) | der schwarze Unter, den keiner haben will | Zählen, für den Spieler | der Spieler sieht, ob der Peter die schwarzen Unter hält; der schwerste Boss (11, 17) bekommt eine Hilfe statt einer Härte |
| Der Kiebitz-Mörder | keine Modifikationen, nur eine eigene Rückseite | er nimmt Information, er gibt keine | – | keine; seine Regel schaltet ohnehin Informations-Wirkungen ab, auch die des Peters |
| Der Alte (Endboss) | die Ausgabe des Bosses, dessen Regel gerade gilt, eine pro Blatt | wie seine Regel | wie dort | wechselt mit dem Blatt wie die Regel; vor der Serie zeigt die Sicht die Reihenfolge |

**Umgesetzt in M9** (Antwort im Cockpit: milde Stufe, die Tabelle oben). Daten: `edition` in `bosses.ron` (Liste aus Kartenfilter, Modifikation, Wert), `back` (nur der Kiebitz-Mörder hat eine eigene, `kibitz_killer`; alle anderen die Gegner-Rückseite). Der Boss sitzt links (`BOSS_SEAT`); nur seine Karten tragen die Ausgabe, auch beim Tausch des Revolutionärs bleibt sie am Platz. Beim Alten gilt die Ausgabe des Bosses, dessen Regel das Blatt gerade auflegt (`ActiveGame::boss_edition`), ein rotierender Boss darf keine eigene haben (Ladefehler). Die Sicht nennt die Ausgabe neben der Regel (`BossInfo::edition`, `back`), die Web-App zeigt sie im Boss-Dialog, am Tresen und auf der Infokarte („Seine Ausgabe: Siebenen: Augen +4“), Karten des Bosses im Stich mit seinen Modifikationen. Die Augen-Modifikationen des Bosses stehen in der Abrechnung (`point_changes`); beim Ramscher fällt die Augensumme damit unter 120.

Auslegungen dazu: Die Oma macht weiter jeden Unter zur Blanko, in jeder Ausgabe (Tischregel, 7.1 Regel 7). Die Boss-Ausgabe ist Daten (`edition` in `bosses.ron`, Liste aus Kartenfilter und Modifikation, `back` für die Rückseite); neue Wirkungen braucht keine der milden Ausgaben. Ein Boss-Faktor (`target_factor`) gilt bis zur Messung weiter; die Ausgaben von Schieber, Oma, Zwillingen und Revolutionär machen ihre Serie etwas schwerer, die des Peters etwas leichter.

**Umgesetzt in M4** (`bosses.ron`), Auslegungen von Claude am 01.11.2026:

- **Zwillinge** (`DefendersSeeEachOther`): Die Gegner-KI kennt die Hand des Partners. Gilt als Schummeln.
- **Ramscher** (`RamschOnce`): Das erste Spiel der Boss-Serie ist ein Ramsch: niemand sagt an, nicht passbar, Buben sind Trumpf wie im Grand, jeder spielt für sich, der Skat geht an den Gewinner des letzten Stichs. Eigene Wertung: Grundwert = Augen dessen mit den meisten Augen, eine Stufe, +1 „Jungfrau“, wenn ein Spieler keinen Stich hat. Hat der Spieler nicht die meisten Augen (bei Gleichstand verliert er), gewinnt er den Spielwert, sonst −2 × Spielwert. Seit M5 nennen Abrechnung (`ScoreBreakdown.seats`) und Spielende (`GameFinished.seats`) Augen und Stiche jedes Platzes, die Sicht die Stiche pro Platz (`seat_tricks`) und mit dem Zähler die Augen pro Platz (`count.seats`), damit die Oberfläche zeigen kann, wer die meisten Augen hat.
- **Revolutionär** (`SwapDefenderHands` nach dem 5. Stich): Die Gegner tauschen die Hände. Gilt als Schummeln.
- **Schwarzer Peter** (`ScaleGameValue(0.5)`, wenn gewonnen ohne Schneider). Ein gewonnenes Null ist nie Schneider und zählt hier halb. Bis ADR 0012 nur Alleinspieler, seitdem bei beiden Charakteren. Bis 02.11.2026 zählte ein Sieg ohne Schneider gar nichts (`MultiplyGameValue(0)`); seitdem halb (Antwort im Cockpit). Gerundet wird wie bei jedem Faktor auf dem Spielwert am Ende kaufmännisch (halb auf): Karo mit 2 (27) zählt 14, Null (23) zählt 12, Grand mit 2 (72) zählt 36 (Golden-Vektor `black_peter_half_value_vectors`). Siege mit Schneider oder Schwarz zählen voll, verlorene Spiele wie immer. **Die Faktoren (0,5, Nullspieler 0,2) stammen aus Messungen mit der alten Regel und müssen neu gemessen werden**, sobald die Simulationen wieder laufen.
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

**Stand Serien mit 3 Spielen** (02.11.2026, Zielwerte aus 5.2; Boss-Serie gewonnen im Run, alle Abschnitte, 3 × 800 ganze Abende je Charakter, je Boss 260 bis 370 Serien; normale Serien 70 bis 89 %). Messung in `docs/balance/2026-11-02-kalibrierung-dreierserie.md`.

| Boss | Faktor | Alleinspieler | Nullspieler | Grandspieler | Zaungast | Wirtin |
|---|---|---|---|---|---|---|
| Der Schwarze Peter | 0,5 (Nullspieler 0,2) | 44 % | 35 % | 42 % | 41 % | 47 % |
| Die Oma | 0,3 (nur Alleinspieler) | 53 % | – | – | – | – |
| Der Schieber | 1,0 (Nullspieler 0,4) | 51 % | 53 % | 55 % | 54 % | 60 % |
| Der Ramscher | 1,25 | 69 % | 75 % | 63 % | 73 % | 69 % |
| Der Kontra-Hans | 1,5 | 77 % | 67 % | 71 % | 72 % | 71 % |
| Der Revolutionär | 1,0 → **1,5** (Nullspieler 1,0) | 71 % | 75 % | 69 % | 68 % | 73 % |
| Der Kiebitz-Mörder | 1,0 → **1,5** (Nullspieler 1,0) | 68 % | 77 % | 69 % | 69 % | 73 % |
| Die Zwillinge | 1,0 | 73 % | 76 % | 71 % | 73 % | 74 % |
| Der Alte (Endboss) | 1,4 | 68 % | 58 % | 81 % | 82 % | 61 % |

- **Revolutionär und Kiebitz-Mörder 1,0 → 1,5**: Mit 1,0 gewann der Bot sie so oft wie eine normale Serie (Alleinspieler 80 und 79 %, Grandspieler 83 und 81 %, Wirtin 82 und 85 %); mit 1,5 liegen sie wie der Kontra-Hans. Der Nullspieler gewann sie schon mit 1,0 nur zu 68 und 72 % und behält 1,0.
- **Schieber bleibt 1,0** (vorgeschlagen war 0,8): Er liegt mit 51 bis 60 % bei der Oma, schwer, aber keine Wand.
- **Schwarzer Peter bleibt der schwerste Boss** (35 bis 47 %), und kein Faktor hilft: Mit 3 Spielen gewinnt der Bot die Serie auch bei Zielwert 1 nur zu 50 % (Alleinspieler, `--curve --boss black_peter`, bei 24 noch 49 %), der Nullspieler zu 37 %. Die Grenze ist die Regel selbst (ein Schneider-Sieg, kein verlorenes Spiel). Seit 02.11.2026 zählen Siege ohne Schneider halb statt null (oben); Faktor und Tabellenzeile gelten für die alte Regel, neu messen.
- Der Alte hat je Charakter nur 24 bis 62 Serien; seine Spanne ist Rauschen.

---

## 12. Wirtschaft

**Seit ADR 0016**: Groschen je gewonnener Kneipe (Grundbetrag 3, je übrigem Spiel vorläufig 3 statt 1, Zinsen wie bisher); eine verlorene Kneipe zahlt nichts. Der Tresen kommt nur noch als Station eines Weges (5.0.1), nicht mehr nach jeder Serie; die Spelunke bringt ein Relikt oder seinen halben Preis. Alles vorläufig (5.0.4).

- **Währung**: Groschen. Thematisch Zehntelpfennig, wie am Kneipentisch.
- **Einnahmen**: Grundbetrag pro gewonnener Serie, Bonus pro übrigem Spiel, Zinsen auf Guthaben (gedeckelt), Verkauf von Relikten.
- **Ausgaben am Tresen**: Hausregeln, Figuren, Verbrauchsgegenstände, Kartenmodifikationen, Neuauslage.
- **Richtwerte (M2, kalibriert)**: Startkapital 4, Serie-Grundbetrag 3, Bonus pro übrigem Spiel 1, Zinsen 1 pro 5 Groschen bis maximal 5, Neuauslage 2. Hausregeln kosten 5 bis 8: starke 7 bis 8 (Bockrunde 8, Alte Dame 7, Schenken 7, Tournee 8), schwache und noch nicht wirksame 5. Figuren 6 bis 10, Verbrauchsgegenstände 2 bis 4. Messung und Begründung in `docs/balance/2026-11-01-m2-zielwerte.md`. Der Run-Bot hat damit beim Start von Abschnitt 2 im Mittel drei Hausregeln, ab Abschnitt 3 alle fünf Plätze belegt. Fürs Spätspiel fehlen noch Ausgaben (offen für M4 und M5).
- **M5 (PIMC-Spieler, 8 Spiele)**: Grundbeträge unverändert. Relikte kosten 4 bis 9, gestaffelt nach der gemessenen Wirkung allein ab Spielbeginn ohne Tresen (`skat_sim --runs 500 --no-shop --bot pimc:12:3 --relic <id>`, Seeds 42 bis 44, Veränderung von „Abschnitt 3 erreicht“, der stärkere Charakter zählt): ab +6,5 Prozentpunkten 9 (Bockrunde, Alte Dame, Kleinvieh; Stammgast wegen +6,6 auf Abschnitt 4), ab +4,5 8 (Schenken, Ein Auge zudrücken, Siebenschläfer, Schieben), ab +3 7, ab +1,5 6, darunter 5. Grenzfälle eine Stufe tiefer (Durchmarsch +4,5: 7; Kontra +1,6, beim Nullspieler −0,9: 5). Relikte, die vor allem Menschen nutzen (Hand, Ouvert, Information) oder nur gegen einzelne Bosse wirken, 4 bis 6. Hausregeln und Figuren folgen derselben Staffel. Tabelle in `docs/balance/2026-11-01-m5-v1.md`. Die Schnäpse (8.3) kosten 2 bis 3 als Platzhalter, ohne Messung.
- **Seltenheit folgt der Stärke (M5)**: Vorher lagen die drei stärksten Relikte beider Charaktere in Common und kamen in 20 bis 21 % aller Angebote. Bockrunde, Alte Dame, Kleinvieh und Siebenschläfer sind jetzt Uncommon, Spitze, Mit Vieren, Schiedsrichter und Saubere Weste dafür Common; Anteil der drei stärksten am Angebot 11 % (Alleinspieler) und 14 % (Nullspieler). Wirkung im Run (2 × 1000 Runs): Abschnitt 2 / Run gewonnen 36,8 / 1,45 % → 35,9 / 1,0 % (Alleinspieler), Nullspieler 47,5 / 3,8 → 46,5 / 3,9 %, im Rahmen des Rauschens. Gewählt, weil ein seltener Fund stärker sein soll als ein häufiger, nicht wegen der Zahl.
- Ein Run bringt dem Bot im Mittel 23 (Alleinspieler) bis 26 Groschen (Nullspieler), fast alles wird ausgegeben.
- **Serien mit 3 Spielen (01.11.2026)**: Der Bonus pro übrigem Spiel (1 Groschen) bringt jetzt 0 bis 2 statt bis zu 6. Grundbetrag, Zinsen und Preise unverändert; sie misst `balance` mit der vollen Kalibrierung neu. Kaufwerte des Bots (`CONSUMABLE_VALUES`): Neues Blatt 2, Ein Bier und Kräuter 1 (vorher 4, 3, 3), weil sie nur helfen, wenn die zwei Einmischen eines Spiels verbraucht sind: Der Bot mischt etwa jedes zweite Blatt ein (0,75 Einmischen je Spiel, 5.2), drei schwache Blätter hintereinander kommen etwa in jedem achten Spiel. Die Testrunde (`docs/balance/2026-11-01-testrunde.md`) hatte gezeigt, dass er sie mit 6 Pässen kaufte und kaum brauchte.
- **Welle 3 (02.11.2026, 7.2 und 8.4)**: Neue Seltenheit Legendär mit dem Tresen-Gewicht 0,3 (Rare 1); die Seltenheit setzt nicht den Preis. Preise nach der Staffel aus Welle 2a: Skatgott 9 (+4,7, im Spätspiel stärker als alles gemessene), Ehrenvorsitz 6 (ohne Wirkung beim Bot, ein Relikt für Menschen). Kaufwerte des Bots in `LEGENDARY_VALUES` (9 und 1). Wandelkarte (Rare, 5), Glasdaus (Uncommon, 4) und Heimweh (Uncommon, 4) hatten wie alle Modifikationen keinen eigenen Tresen-Preis; sie kamen über Stempelkissen und Ereignisse (Glasdaus und Heimweh seit Effekte v2 gestrichen). Alles Platzhalter für `balance`, Bericht `docs/balance/2026-11-02-welle3.md`.
- **Welle 2a (02.11.2026, 8.1)**: Mit 3 Spielen je Serie fallen alle Wirkungen kleiner aus (Alte Dame +5,3 statt +7,1 auf „Abschnitt 3 erreicht“). Die Preise der acht neuen Hausregeln folgen der M5-Staffel auf drei Viertel verkleinert (ab +4,9 Punkten 9, ab +3,4 8, ab +2,3 7, ab +1,1 6, darunter 5; Grenzfälle eine Stufe tiefer), Relikte für Menschen 6. Kaufwerte des Bots in `RELIC_VALUES`, hochgerechnet auf den M5-Maßstab. Die Preise der älteren Relikte bleiben, bis `balance` alles mit 3 Spielen neu misst.
- **Welle 2b (02.11.2026, 8.1)**: Lockvogel 6 und Spickzettel 4 nach der Staffel der Welle 2a (Spickzettel billiger als der Kiebitz, weil er nur im Handspiel wirkt). Drei drücken wirkt seit der Entscheidung des Leads nur einmal pro Serie und liegt damit bei +7,9 (auf den Zielwerten aus PR #47), etwas über der Alten Dame: Rare, 9 Groschen (oberste Stufe der Staffel), gesperrt hinter „Stammkunde“ (9.3). Kaufwerte des Bots: 10, 3, 1.
- **Effekte v2 (02.11.2026, 7.2)**: Modifikationen mit konkreten Werten gibt es am Tresen als Verbrauchsgegenstände (Augen +3 für 2, Augen +5 für 3, Trumpf +2 für 3, Trumpf +4 für 4, Rang +1 für 3 Groschen), alles Platzhalter ohne Messung. Kaufwerte des Bots in `CONSUMABLE_VALUES` (1 bis 2, auch Platzhalter); er setzt sie wie das Stempelkissen auf einen Daus ohne Modifikation. Flüche zahlen weiter 3 Groschen beim Erhalt, der Gutschein bringt +3 beim Drücken.
- **Spielwahl und Einsatz** (Bot, 01.11.2026): `suit_expected_score` rechnet mit der Gewinnschwelle des Spiels (Einsatz und Relikte): Über 61 schrumpft die gemessene Gewinnquote mit den übrigen Augen, `(120 − Schwelle) / 59` davon (grobes Modell, nicht gemessen). So wählt der Bot bei hohem Einsatz öfter Null, das keine Augen zählt.

---

## 13. Ereignisse (Rast)

**Seit ADR 0016** sind Ereignisse Stationen auf einem Weg zwischen zwei Kneipen (5.0.1) und tragen eine Kategorie: Begegnung (Stammtisch-Streit, Alter Bekannter, Letzte Runde, Umgekipptes Bier, Preisskat, Falschspieler, Gast aus Altenburg), Upgrade (Kartengeber, Gasthof zur Krone, Taschenspieler, neu die Kartenwerkstatt: eine Modifikation auf eine gewählte Karte oder eine Karte säubern, beides gratis) und Rast (neu die Imbissbude: ein Leben zurück). Die Wahrscheinlichkeit `encounter_chance` entfällt. Die Beute der Spelunke ist ein Ereignis, das nie auf einem Weg liegt (`spelunke_loot`, Gewicht 0). Der Stammtisch-Streit gilt bis zum Ende des Stadtteils.

Zwischen Serien kann statt des Tresens ein Ereignis kommen. Beispiele:

- **Stammtisch-Streit**: Wähle eine Hausregel, die für den Rest des Abends für alle gilt.
- **Der Kartengeber**: Lass eine zufällige Karte zinken oder zahle 3 Groschen und wähle selbst.
- **Alter Bekannter**: Tausche ein Relikt gegen ein zufälliges anderes.
- **Letzte Runde**: Spiel ein einziges Spiel um den doppelten Einsatz.

**Umgesetzt in M4** (`core/content/encounters.ron`, Begriff im Code `Encounter`, weil `Event` die Zustandsereignisse sind). Nach einer gewonnenen Serie (nicht nach der letzten) kommt mit der Wahrscheinlichkeit `encounter_chance` aus `run.ron` (Platzhalter 0,2 für `balance`) ein Ereignis statt des Tresens, gezogen nach `weight`. Es bietet Optionen an (`EncounterOption`, immer zuletzt „Weitergehen“), der Spieler wählt mit `RunAction::ChooseEncounter`, danach beginnt sofort die nächste Serie.

- **Stammtisch-Streit**: drei Hausregeln zur Wahl (freigeschaltet, nicht im Besitz, für den Charakter). Die gewählte gilt bis zum Ende des Abschnitts („Abend“) ohne Platz zu belegen.
- **Der Kartengeber**: „zinken“ heißt eine Kartenmodifikation anbringen (seit ADR 0015 immer auf einer Karte der Ausgabe des Spielers; das gilt für jedes Ereignis, das eine Karte markiert, auch für Flüche). Kostenlos auf eine zufällige Karte, für 3 Groschen auf eine gewählte; die Modifikation ist zufällig aus dem Pool des Charakters, seit M7 nie ein Fluch. Seit M5 wird sie gezogen, wenn das Ereignis aufgeht, und beide Optionen nennen sie (`EncounterOption.card_mod`, in der Sicht `card_mod` mit Name und Fluch-Markierung). Der Spieler entscheidet also mit Wissen; zufällig ist nur noch die Karte.
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

**Effekte v2 (02.11.2026, 7.2).** Die Ereignisse vergeben die neuen Modifikationen mit Wert (`MarkCard.value`, sonst der erste Wert aus `values`); verfluchte Karten sind nie Ziel, eine Karte mit anderer Modifikation verliert sie (Ersetzen), eine mit derselben steigt:

- **Umgekipptes Bier**: Ramponiert −5 statt Bierfleck (Fluch).
- **Gasthof zur Krone**: Trumpf +4 statt Krone.
- **Der Taschenspieler** (Nullspieler): Gutschein Wandelkarte statt Attrappe (die Wandelkarte nimmt bei Spielbeginn die längste Farbe an, im Null ein Werkzeug).
- **Der Falschspieler**: zieht aus Durchsichtig, Ramponiert und Fleckig (Fleckig nur mit Gewicht für Farbspiel und Grand).
- **Der Kartengeber**: zieht aus Rand, Moment und Gutschein (7.2).

---

## 14. Technik

### 14.1 Architektur

```
skat-roguelite/
├── core/            Rust-Crate „skat_core“: Regeln, Zustand, KI, Run-Logik, Content
│   ├── cards        Karten, Ausgaben (ADR 0015), Modifikationen
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
| **M8 Inhalte** ✔ | Schnäpse, drei neue Charaktere (Grandspieler, Zaungast, Wirtin), Bierdeckel-Grafiken, Infokarte, große Testrunde, Welle 2 (Relikte: Skat-Hebel, Bau-Richtungen, Spiel-Relikte; Welle 2a mit acht Hausregeln und Welle 2b mit drei umgesetzt, 8.1), Welle 3 (Wandelkarte, Glasdaus, Heimweh, Stufe Legendär mit Skatgott und Ehrenvorsitz; umgesetzt 7.2, 8.4) | Ein Run mit vielen Bau-Richtungen |
| **M10 Kneipentour** | ADR 0016: drei Stadtteile mit je vier Kneipen und einer Boss-Kneipe, Leben, Routenwahl, Spelunke, Namensgenerator, kurzer Abend entfällt, laufende Spielstände verworfen (Kern, Inhalte, Bot, einfache Routenwahl im Web umgesetzt am 02.11.2026; Stadtkarte getrennt im Entwurf), Messung offen (5.0.4) | Ein Abend mit Weg und zweiten Chancen |
| **M9 Eigene Ausgaben** | ADR 0015: Ausgabe des Spielers statt geteiltem Blatt, Grunddecks als Modifikationen, Boss-Ausgaben, Gegner-Rückseiten, alte laufende Spielstände verworfen (Kern, Inhalte, KI, Web umgesetzt am 02.11.2026), Nachmessung offen (18) | Jeder spielt mit seinen eigenen Karten |

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
| Serie | 3 Spiele je Serie gegen dieselben zwei Gäste, 2 Serien und eine Boss-Serie je Abschnitt (5.1). Zielwerte am 02.11.2026 neu kalibriert, je Charakter (5.2). |
| Ansagen nach dem Skat | Schneider, Schwarz angesagt und Ouvert auch nach dem Aufnehmen, ohne Hand-Stufe (ADR 0013, 4.3). Der Bot sagt nach dem Skat mit denselben Regeln an wie aus der Hand. |
| Freischaltungen | `early_finish` heißt jetzt „Serie gleich im ersten Spiel gewinnen“, `boss_without_pass` „Boss ohne Einmischen“, `grand_master` verlangt 144 statt 96 Punkte (9.3). |

Entschieden am 02.11.2026 (Nutzer):

| Frage | Entscheidung |
|---|---|
| Blatt-Modell | **Eigene Ausgabe jeder Karte** (ADR 0015, ersetzt 0005 teilweise): ein normales Skatblatt für alle, jeder hält seine eigene Ausgabe; keine Karten hinzufügen oder entfernen, Grunddecks als Modifikationen; der Skat gehört dem Spieler; Gegnerkarten mit eigener Rückseite (7.1, 9.8, 11, 18). Simulationen bleiben pausiert, gemessen wird später. |
| Kartenmodifikationen | **Effekte v2** (7.2): höchstens eine Modifikation je Karte (Spezialkarten zusätzlich eine), Steigerung durch dieselbe, Ersetzen durch eine andere, Flüche sperren die Karte; Rand (Augen, Rang, Trumpf), Moment (Letzter Stich, Erster Stich), Gutschein (wenn gedrückt), Flüche (Durchsichtig, Ramponiert, Fleckig), Spezialkarten (Untot, Wandelkarte). Visuelle Grammatik: Rahmen = was, oben rechts = wann, unten Mitte = wie viel, unten links = Fluch. 18 alte Modifikationen und Heimweh gestrichen, alte Spielstände werden umgerechnet. |

## 17. Offen

- **Kneipentour (ADR 0016)**: alle Zahlen vorläufig und ungemessen (5.0.4): Zielwerte je Kneipe und Charakter, Leben (3/5/+1), Bonus je übrigem Spiel (3), Gewichte der Stationen, Spelunke (0,3 und × 1,25), Gegnerstärke je Stadtteil, umgeschriebene Freischaltungen. Auslegungen von Claude: Eine verlorene Boss-Kneipe wird sofort wiederholt, ohne Routenwahl dazwischen (also auch ohne Tresen); die Spelunke kommt nie vor einer Boss-Kneipe; unter drei Wegen ist immer ein Tresen; Rast nur mit fehlendem Leben; der Malus „Kein Skat“ trifft den Nullspieler nicht. Offen im Cockpit: Sekundant des Bosses (Datenmodell vorbereitet, 5.0), die Form des Malus (umgesetzt: einer, zufällig, vorher sichtbar). Die Stadtkarte (Pixel-Stadtplan) wird getrennt entworfen; die Sicht liefert dafür den ganzen Plan, Knoten-Ids und Seeds (5.0.3).

- **Effekte v2** (02.11.2026, 7.2): Werte, Preise und Seltenheit sind Platzhalter, nichts gemessen (Simulationen pausiert). Entschieden: Moment-Modifikationen und Spezial-Gutscheine höchstens einmal, Groschen-Gutschein, Augen und Trumpf ohne Obergrenze, Trumpf und Fleckig streng (Cockpit). Mit M9 erledigt: Frisches Deck reinigt (7.3), die Sicht zeigt die Ausgabe des Gegners auf einer durchsichtigen Karte (7.2).
- **Eigene Ausgaben (ADR 0015)**: beantwortet im Cockpit und umgesetzt in M9 (02.11.2026): Trumpf und Fleckig streng, Boss-Ausgaben mild, Stammgäste normal, Frisches Deck reinigt, Nullspieler mit Abgestuft, laufende Spielstände verworfen. Offen: alle Messungen (18.4, Simulationen pausiert); ob die Gleichstands-Regel bei Abgestuft auf Daus und Zehn (gleiche Stärke, die frühere gewinnt) für Menschen verständlich ist; ob die Web-App vor der Boss-Serie zusätzlich die Reihenfolge der Ausgaben des Alten zeigen soll (heute nur die Ausgabe der Regel im laufenden Blatt).

- **Zielwerte für 3 Spiele**: erledigt am 02.11.2026 (5.2, je Charakter eigene Kurve, Boss-Faktoren 11, Einsatz-Leiter 5.4, Freischaltungen 9.3). Offen daraus: (a) **Nullspieler gewinnt nur 0,6 % der ganzen Abende** (Ziel 1 bis 2 %); seine Kurve liegt ab Abschnitt 3 fast am Boden, Zielwerte bewegen dort kaum noch etwas. Entschieden am 02.11.2026: ein Einmischen mehr (`rule: (redeals: 1)` in `characters.ron`, 0,69 → 1,12 % bei gleichen Seeds), siehe 9.2. (b) **Der Schwarze Peter** bleibt mit 35 bis 48 % der schwerste Boss; mit 3 Spielen liegt seine Obergrenze bei 50 % (Nullspieler 37 %), kein Faktor hilft. Entschieden am 02.11.2026 (Cockpit): gewonnene Spiele ohne Schneider zählen halb statt null (11); die Faktoren neu messen, wenn die Simulationen wieder laufen. (c) Preise von Schieben, Ein Bier, Kräuter, Neues Blatt und Reue (8.1, 8.3) und ob Einmischen beim Alten (Regel je Blatt) zu leicht macht, sind noch nicht gemessen.

- **Null-Formel**: entschieden in M4 (9.2). Offen bleibt das Feintuning der Stilboni mit Testspielen.
- **Zielwert-Faktoren pro Charakter**: erledigt mit ADR 0012 (`character_factors` in `bosses.ron`, 11). Schieber und Schwarzer Peter sind wieder bei beiden Charakteren. Offen: Die Oma bleibt beim Nullspieler draußen, weil sein Blatt keine Unter hat und sie ihm nichts nimmt (ADR 0012 sieht sie wieder bei jedem vor). Sobald er Unter ins Blatt bekommen kann, mit `--curve --boss grandma --character null` neu messen. Mit ADR 0015 (Variante A, 9.8) halten die Gegner wieder echte Unter, die Oma nimmt sie ihnen; neu messen, wenn die Simulationen wieder laufen.
- **Null für alle mit Menschen prüfen** (ADR 0012): Die Gewinnquoten hinter der Bot-Wahl Farbe, Grand oder Null (`SUIT_WIN_PERCENT`, `GRAND_WIN_PERCENT`, `NULL_WIN_PERCENT` in `core/src/run/bot.rs`) sind Bot-Messungen ohne Relikte. Ob Menschen mit dem Alleinspieler öfter Null spielen und ob die Tresen-Gewichte (`focus_weights`) den Nullspieler klar genug vom Alleinspieler trennen, zeigt die Statistik echter Runs.
- **M5-Kurve mit Menschen prüfen**: Die Kurve (5.2) ist gegen den PIMC-Spieler gesetzt. Ein Mensch nutzt Hand, Ouvert und Informations-Relikte, die der Bot kaum spielt, und dürfte stärker sein. Testspiele und die Statistik der Web-App (Abschnitt erreicht) gegen die Bot-Zahlen halten.
- **Relikte ohne Bot-Wirkung**: Spitze, Mit Vieren, Schiedsrichter, Bierdeckel, Ouvert-Bonus, Oma, Opa, Glashaus, Blindflug, Saubere Weste und Zähler bewegen beim Bot „Abschnitt 3 erreicht“ um weniger als 5 % (relativ). Sie sind für Menschen gedacht; ob sie die 5-%-Regel erfüllen, zeigt erst die Statistik echter Runs.
- **Ereignis-Wahrscheinlichkeit, Figurenpreise**: Platzhalter, `balance`.
- **Welle 2a im vollen Run** (02.11.2026, 8.1): Preise und Seltenheit der acht neuen Hausregeln sind aus der Einzelmessung ohne Tresen gesetzt (`docs/balance/2026-11-02-relikte-welle2.md`), Platzhalter für `balance`. Offen: Wirkung im vollen Run mit Tresen (3 × 3000), beim Nullspieler, und ob der Bot den Skatpfleger beim Drücken nutzen soll (er drückt weiter Karten mit wenig Augen, ein Mensch drückt Zehnen). Stammfarbe, Wechselbalg und Allrounder hängen an der Spielwahl; der Bot wählt das Spiel ohne Blick auf die Relikte, ein Mensch kann gezielt wechseln oder bei einer Farbe bleiben. Zu entscheiden: ob der Wechselbalg nur bei gewonnenen Spielen zählt (jetzt wie jede Stufe auch verloren, wie der Text sagt; beim Bot kostet er so Punkte je Spiel).
- ~~**Drei drücken zähmen?**~~ Entschieden am 02.11.2026 (Lead): einmal pro Serie, beim ersten Aufnehmen des Skats (8.1). Gemessen +7,9 Prozentpunkte auf „Abschnitt 3 erreicht“ statt +18,0 in jedem Spiel; nicht weiter abgestimmt. Offen bleibt nur die Wirkung im vollen Run mit Tresen und beim Nullspieler.
- **Nullspieler über den kurzen Abend** (M7, 5.3): „Run gewinnen“ zählt auch im kurzen Abend, der Nullspieler ist damit nach rund 25 statt rund 1700 Runs (Heuristik-Bot) frei. Entschieden ist „Freischaltungen in beiden“; soll der Nullspieler ein Ziel des ganzen Abends bleiben, bekäme die Bedingung `WinRun` eine Länge (Daten in `unlocks.ron`). Ebenso „Abschnitt 4 erreichen“: nur im ganzen Abend möglich.
- **Weitere Kartenrückseiten** (9.3): Standard ist die Distel-Rückseite (nach Altenburger Vorbild, eigene Pixelgrafik). Weitere Rückseiten werden später über bestimmte Erfolge freischaltbar (Freischaltung als Kosmetik, ohne Spielwirkung). Offen: welche Erfolge. Die Bedingung gehört wie jede Freischaltung in den Kern (`unlocks.ron`, eine neue Belohnungsart für Kosmetik); die Web-App bietet nur die verdienten Rückseiten zur Wahl an.
- **Welle 3** (02.11.2026, 7.2, 8.4, `docs/balance/2026-11-02-welle3.md`): Abschnitt 3: Skatgott +4,7, Heimweh +0,5, Wandelkarte +0,2, Glasdaus +0,1, Ehrenvorsitz −0,4 (Alte Dame +4,5). Entschieden am 02.11.2026: Skatgott nur auf gewonnene Spiele, Preise nach Wirkung statt nach Stufe (8.4, 12). Der Glasdaus bringt früh Punkte und kostet spät (er zerbricht in 70 % der Runs); ob er beim Gegner (22 Augen für die Gegner) zu oft schadet, zeigen echte Runs (seit ADR 0015 hinfällig: Beim Gegner gibt es ihn nicht mehr). Heimweh und Wandelkarte nutzt nur ein Mensch richtig (bewusst drücken, gezielt benennen). Nicht gemessen: voller Run mit Tresen, Nullspieler.
- **Untot** (02.11.2026, 7.2; seit Effekte v2 Spezialkarte über den Gutschein): Messung im Simulator steht aus (auf Wunsch des Nutzers zurückgestellt, das Blatt-Konzept kann sich noch ändern). Zu messen wie Welle 3 (`--card-mod CK:undead`, PIMC 12:3, Alleinspieler, 3 × 500 Runs), dazu die Variante „in jedem Stich“; Seltenheit und Preis bis dahin Platzhalter. Die Oberfläche ist seit dem 02.11.2026 eingebaut (Wunsch des Nutzers: Simulationen pausiert).
- **Fluchbeträge** (M7, 7.2): alle Flüche zahlen 3 Groschen, die gewählte Karte kostet bei Bier und Falschspieler 4. Platzhalter für `balance`. Die Option „Zufällige Karte“ zeigt in der Web-App „gratis“, die 3 Groschen stehen nur im Text des Fluchs.
- **Einsatz-Leiter**: gemessen am 02.11.2026 (5.4): ein Auge kostet rund 1,2 Punkte „Abschnitt 2“, Null wird bei Einsatz 5 etwas häufiger (35,8 % statt 32,6 %), dominiert aber nicht. Höhere Stufen (ab 10) noch nicht mit dem einsatzbewussten Bot gemessen. Die Texte von „Ein Auge zudrücken“ und „Hufeisen“ sagen seit der Einsatz-Leiter „drei/fünf Augen weniger“.

---

## 18. Umbau auf eigene Ausgaben (ADR 0015)

Plan von Claude, 02.11.2026. Seit Effekte v2 (7.2) sind Glasdaus, Scherben und Heimweh aus dem Plan gefallen; was hier über sie steht, ist überholt. **Umgesetzt in M9 (Zweig `feat/editions`, 02.11.2026): Schritte 1 bis 4 aus 18.5; Schritt 5 (Messung) wartet, bis die Simulationen wieder laufen.** Abweichungen vom Plan stehen in 18.3 und 18.5. Der Zweig `feat/undead` (Modifikation „Untot“, Kartenbilder in `web/src/ui/cards.ts`) wird vorher zusammengeführt, damit sich `card_mods.ron`, `DealtCard` und die Kartenbilder nicht doppelt ändern.

### 18.1 Datenmodell im Kern

- **Ausgabe** (`Edition`): je Karte des Skatblatts (32, in der Reihenfolge von `standard_deck`) die Liste der Modifikationen. Die Ausgabe des Spielers ersetzt das Blatt des Runs (`Run::edition` statt `Run::deck`); jede Karte behält eine feste Id (heute `DeckCard::id`), damit Ereignisse, Heimweh und die Oberfläche weiter über Ids sprechen.
- **Ausgaben der Gegner** werden nicht gespeichert, sondern beim Geben aus dem Platz abgeleitet: Stammgast normal, Boss aus `bosses.ron` (`edition`), beim Alten die des Bosses, dessen Regel gilt.
- **Geben**: Gemischt und gegeben werden die 32 Karten wie heute (gleicher Zufallsstrom, gleiche Reihenfolge, damit Seeds dieselben Karten geben). Danach wird jede Karte mit der Ausgabe ihres Halters aufgelöst (`resolve_card(card, edition)` statt `resolve_deck`); `DealtCard` bekommt den Besitzer der Ausgabe (`owner`, für Sicht, Rückseite und Hooks). Der Skat wird mit der Ausgabe des Spielers aufgelöst.
- **Wechsel des Halters**: eine Funktion im Spielautomaten, die eine Karte von einem Platz zum anderen bewegt und neu auflöst, mit Ereignis (`CardChangedHands`); genutzt von Drei drücken, Revolutionär und Heimweh. Gespielte Karten werden nie neu aufgelöst.
- **Hooks**: `card_mod_sources` nimmt nur noch Karten in der Ausgabe ihres Halters (Spieler: Hand, Skat, gespielte Karten; Boss: seine Hand und seine gespielten Karten), mit `this_card` und dem Besitzer. `ThisCardBeatenByDefender` (Glasdaus) prüft zusätzlich, dass der Spieler die Karte gespielt hat.
- **Entfällt**: `DeckSpec::remove` und `add` (Ladefehler), `Deck::remove`, `Deck::add`, `Deck::copy`, `DeckChange::Added`, `FilledBlank`, `ReplacedByBlank` (nur noch zum Lesen alter Ereignisse), der Talon in `Game` und in den Sichten, `DealOdds` nach Blattgröße. `CardKind::Blank` bleibt (Oma, Scherben). Die Sonderwege für Wandelkarte und Untot beim Geben entfallen.
- **Neu im Content** (je nach Antwort): Wirkung `Demote` und Modifikation `demoted` (Nullspieler), Modifikation `shards` (Scherben) mit der Wirkung „in dieser Ausgabe eine Blanko“, Glasdaus `BreakThisCard` → ersetzt sich durch `shards`, Verbrauchsgegenstand Frisches Deck mit Aktion `CleanseCard`, Felder `edition` und `back` in `bosses.ron`, `back` in `guests.ron`.
- **Glossar**: Ausgabe `Edition`, Halter `holder`, Besitzer einer Ausgabe `owner`, Scherben `shards`, Abgestuft `demoted`, Rückseite `back` (eingetragen am 02.11.2026, mit M9 auf den Stand des Codes gebracht).

### 18.2 Inhalte und Texte

- `characters.ron`: Nullspieler nach der Antwort (9.8), Texte ohne „keine Unter, vier Siebener mehr“.
- `card_mods.ron`: Texte ohne „egal, wer sie spielt“ (Spitze, Glückspfennig, Unsichtbar, Angeknackst, Attrappe, Unter ehrenhalber); Schlusslicht und Aufgebrezelt neu (7.2); Glasdaus „… zerbricht, wenn ein Gegner sie sticht, und bleibt als Scherben“.
- `bosses.ron`: Ausgaben nach der Antwort (11), die Boss-Vorschau nennt sie.
- `consumables.ron`: Frisches Deck neu (7.3), Stempelkissen „einer Karte deiner Ausgabe“.
- `encounters.ron`: Texte „eine Karte deiner Ausgabe“.
- Web: Übersetzungsdatei und Regelkarte („Jeder spielt mit seiner eigenen Ausgabe“), Wirt-Hinweis beim ersten Boss mit Ausgabe.

### 18.3 Spielstände

Laufende Runs liegen im Browser (9.3). **Entschieden im Cockpit: verwerfen statt umrechnen.** Umgesetzt: Spielstand-Version 2 (`SAVE_VERSION` in `wasm/src/lib.rs`); ein laufender Run der Version 1 wird beim Laden mit dem Fehler `OutdatedSave` abgelehnt (die Version wird vor dem Run gelesen, ein alter Run muss nicht mehr passen), die Web-App löscht ihn und sagt einmal „Neue Regeln: Jeder spielt jetzt mit seiner eigenen Ausgabe der Karten. Dein laufender Abend wurde beendet; Freischaltungen, Statistik und Einsätze bleiben.“ Der Meta-Stand ist ein eigener Spielstand und bleibt. Die Fixtures der Smoke-Tests (`web/e2e/fixtures`) wurden mit einem Werkzeug im Kern auf Version 2 gehoben (`cargo test -p skat_core regenerate_fixtures -- --ignored`: Blatt wird Ausgabe, das laufende Spiel wird Karte für Karte nach Halter aufgelöst), damit ihre Szenen bleiben; `discard-battered.json` und `honorary-jack.json` bleiben Version 1 und prüfen das Verwerfen.

Der ursprüngliche Vorschlag (überholt):

- Ein Blatt aus genau 32 verschiedenen Karten ohne Blankos wird 1:1 die Ausgabe des Spielers (Ids und Modifikationen bleiben). Das sind fast alle Runs: Alleinspieler, Grandspieler, Zaungast, Wirtin ohne Frisches Deck und ohne zerbrochenen Glasdaus.
- Fehlt eine Karte (durch eine Blanko ersetzt), bekommt sie in der Ausgabe: beim Daus „Scherben“ (das war fast immer ein Glasdaus), sonst keine Modifikation (Frisches Deck). Modifikationen auf Blankos gibt es nicht.
- Nullspieler: Doppelte Siebenen werden zusammengelegt (Modifikationen der zweiten gehen auf die erste, soweit sie dort nicht schon liegen; was nicht passt, verfällt), die vier fehlenden Unter bekommen die neue Modifikation des Nullspielers.
- Karten über 32 (heute erzeugt das kein Inhalt) verfallen samt Modifikationen.
- Ein Spiel, das mitten im Stich gespeichert wurde, spielt im alten Modell zu Ende (die aufgelösten Karten stehen im Spielstand); ab dem nächsten Geben gilt die Ausgabe. Alte Ereignisse (`DeckChanged` mit `ReplacedByBlank`) werden weiter gelesen. Meta-Stände sind nicht betroffen.
- Tests: Spielstände aus `main` vor dem Umbau als Fixtures (je Charakter, mit zerbrochenem Glasdaus, mit Frischem Deck), Laden, Umrechnen, ein Spiel weiterspielen.

### 18.4 Simulator

Die Simulationen sind pausiert (16). Was der Umbau ungültig macht und später gemessen werden muss, in dieser Reihenfolge:

1. Gleichheit: Alleinspieler ohne Relikte vor und nach dem Umbau mit denselben Seeds; die Karten sind dieselben, die Ergebnisse müssen gleich sein (ohne Modifikationen gibt es keinen Unterschied). Das ist der Test, dass das Geben unverändert ist.
2. Charaktere: Nullspieler (neues Grunddeck) und Grandspieler (Goldrand nur noch bei ihm) gegen ihre Zielwerte (5.2).
3. Bosse: jede Boss-Ausgabe mit `--curve --boss <id>` und im Run, Faktoren anpassen (11).
4. Modifikationen: einzeln wie Welle 3, vor allem die Modifikationen auf die Trumpffarbe (wenn streng) und Glasdaus mit Scherben.
5. Ganze Kalibrierung (3 × 800 Runs je Charakter) wie in 5.2.

Der Simulator braucht dafür keine neuen Schalter; `--boss` und `--character` reichen. Der `RunBot` muss die neue Wirkung von Frisches Deck kaufen und einsetzen können (Kaufwert in `CONSUMABLE_VALUES`).

### 18.5 Schritte

1. ✔ **Kern: Datenmodell**: `Edition`, Auflösen nach Halter (`Game::deal_editions`, `DealtCard::owner`), Wechsel des Halters (`Game::change_holder`, `CardChangedHands`), Hooks nach Besitzer, Skat als Ausgabe des Spielers, Talon, `Deck::add/remove/copy`, `DeckSpec::remove/add` (Ladefehler) und Blattgröße entfernt; statt Umrechnung alter Spielstände das Verwerfen (18.3). Golden-Vektoren unverändert. Tests in `core/src/run/edition_tests.rs` und `core/src/game.rs`. **Gleichheitstest** (18.4.1): `plain_deck_deals_and_plays_as_before_editions` spielt mit dem RunBot die erste Serie von Abschnitt 1 (Heuristik) und 2 (PIMC) für die Seeds 1, 7, 42 und 99 und vergleicht Blätter, jede gespielte Karte, Stiche und Wertungen mit Prüfsummen, die auf `main` vor dem Umbau (53b9d29) aufgenommen wurden: gleich.
2. ✔ **Inhalte**: Nullspieler (Abgestuft, 9.2), Boss-Ausgaben mild (11), Frisches Deck reinigt (7.3), Rückseiten-Ids, Texte.
3. ✔ **KI**: Stichproben lösen nach Platz auf (10.5), Sonderweg in `defender_card` entfällt, Heuristik und Bot kennen Abgestuft; der Bot kauft Frisches Deck bei einem Fluch.
4. ✔ **Web**: Rückseite je Besitzer (Gegner-Rückseite, Kiebitz-Mörder eigene, Skat und Hand Distel), Boss-Ausgabe im Boss-Dialog, am Tresen und auf der Infokarte, Karten der Gegner im Stich mit ihrer Ausgabe, Infokarte nennt die Ausgabe und Abgestuft, `DealOdds`-Satz, Hinweis beim Verwerfen alter Runs. Smoke-Test mit Playwright (Handy hochkant, Desktop).
5. **Messung** (`balance`, wenn die Simulationen wieder laufen): 18.4, danach Zielwerte und Faktoren.

### 18.6 Risiken

- **Gleiche Seeds, andere Runs**: Wer die Karten anders mischt, verliert die Vergleichbarkeit aller Messungen und geteilten Seeds. Gegenmittel: die 32 Karten in der heutigen Reihenfolge geben und den Gleichheitstest (18.4.1) vor allem anderen bauen.
- **Ausgabe am falschen Platz**: Eine Karte, die den Halter wechselt und nicht neu aufgelöst wird, trägt fremde Modifikationen. Gegenmittel: eine einzige Funktion für den Wechsel, eine Prüfung im Test, dass jede Karte in jeder Hand die Ausgabe ihres Halters zeigt.
- **Kein Ausgleich mehr über die Gegner**: Augen- und Rang-Modifikationen werden stärker, die Modifikationen auf die Trumpffarbe schwächer; bis zur Messung stimmen Zielwerte, Preise und Boss-Faktoren nicht. Gegenmittel: Messung zuerst nach dem Kern-Schritt, Zahlen in 5.2, 11 und 12 als vorläufig markieren.
- **Säule 2 wird leiser**: Ohne das Risiko „hilft dem Gegner“ ist Deckbau sicherer. Bleibt: die Aufwertung schläft, Augen landen im fremden Stich, Glasdaus bricht. Wenn das zu wenig ist, können Flüche, Boss-Ausgaben oder Ereignisse das Risiko zurückbringen; im Spieltest prüfen.
- **Zusammenführen mit `feat/undead`**: beide ändern `card_mods.ron`, `DealtCard` und `cards.ts`. Gegenmittel: Untot zuerst zusammenführen, dann den Umbau darauf aufsetzen.
- **Spielstände**: Umrechnung mit Annahmen (Blanko beim Daus = Glasdaus). Gegenmittel: Fixtures aus echten Spielständen, die Umrechnung sagt im Verlauf (Ereignis), was sie getan hat.
- **Oberfläche**: zwei Rückseiten am Tisch und Modifikationen auf Gegnerkarten im Stich dürfen das Handy-Layout nicht überladen. Gegenmittel: Boss-Ausgaben mit höchstens einer Gruppe, Kennzeichen wie bei den eigenen Modifikationen.

# Stammtisch – Design-Dokument

Stand: 2026-11-03 · Status: Entwurf v0.5 (eigene Ausgaben, ADR 0015; Effekte v2, 7.2) · Sprache des Projekts: Deutsch (Code und Kommentare: Englisch) · Arbeitstitel bis 31.10.2026: „Skat Roguelike“

Dieses Dokument hält die Grundentscheidungen fest und beschreibt das Spiel so weit, dass ein erster spielbarer Prototyp daraus gebaut werden kann. Alles unter „Offen“ ist noch nicht entschieden. Alles andere gilt, bis es hier geändert wird.

---

## 1. Pitch

**Stammtisch: Skat als Roguelike-Deckbuilder.** Du bist immer der Alleinspieler. Über einen Run hinweg spielst du Skat gegen zwei KI-Gegenspieler, musst in jeder Runde einen Punkte-Zielwert erreichen und veränderst zwischen den Runden deine Ausgabe des Blatts: Gegeben wird ein Skatblatt für alle drei, aber jeder hält seine eigene Ausgabe jeder Karte (ADR 0015). Die Spielwertformel des echten Skats (Grundwert × Multiplikator) ist die Score-Engine, Hausregeln sind die Relikte, die Kneipe ist der Shop.

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
Spielwert = Grundwert × Multiplikator

Multiplikator = Spitzen
             + 1  (Spiel)
             + 1  Hand
             + 1  Schneider
             + 1  Schneider angesagt
             + 1  Schwarz
             + 1  Schwarz angesagt
             + 1  Ouvert
```

In der Sprache des Genres: **Grundwert sind die Chips, der Multiplikator (bis 02.11.2026 „Gewinnstufen“, Code `levels`) ist der Mult.** Alle Modifikationen im Spiel greifen an genau einer dieser Stellen an:

| Hebel | Beispiele für Upgrades |
|---|---|
| Grundwert erhöhen | Kreuz zählt 15 statt 12. Grand zählt 30. |
| Spitzen manipulieren | Ein fünfter Bube im Blatt. Karo-Bube zählt als zweite Spitze. |
| Multiplikator erhöhen | Neue Stufen durch Hausregeln: „Spitze“ (letzter Stich mit einer Sieben), „Kontra“, „Bock“. |
| Augen verändern | Pik-Ass zählt 15. Alle 10er zählen 11. Siebener zählen 1. |
| Schwellen senken | Schneider ab 85 Augen. Gewinn ab 58 Augen. |

### 4.3 Was gebogen wird

**Reizen entfällt.** Stattdessen gibt es die **Ansage**. Die Reihenfolge folgt der Skatordnung (ADR 0011): Erst entscheidet der Spieler, ob er den Skat aufnimmt oder Hand spielt; nach dem Aufnehmen drückt er zwei Karten. Danach erklärt er:

1. Spielart (Farbe oder Grand; beim Null-Charakter Null-Varianten).
2. Risikostufe: *Normal*, *Hand* (Skat bleibt liegen), *Schneider angesagt*, *Schwarz angesagt*, *Ouvert* (offen spielen). Seit ADR 0013 (Spieltest vom 01.11.2026, gegen die Skatordnung) gehen Schneider, Schwarz und Ouvert auch nach dem Aufnehmen: Im Farbspiel und Grand stehen dann *Normal*, *Schneider angesagt*, *Schwarz angesagt* und *Ouvert* zur Wahl, beim Null *Normal* und *Ouvert*. Die Stufe *Hand* zählt nur, wenn aus der Hand gespielt wird; eine Ansage nach dem Skat bringt ihre eigenen Stufen ohne Hand (Grand mit 1, Schneider angesagt und erreicht: 4 Stufen statt 5 aus der Hand). Im Kern: `Declaration::with_hand`, im JSON `after_skat: true`.

Jede Stufe erhöht den Multiplikator, aber ein Fehlschlag auf einer angesagten Stufe gilt als verloren. Das ersetzt den Push-your-luck-Charakter des Reizens durch eine einzige, klare Entscheidung.

**Einmischen statt Passen** (Spieltest vom 01.11.2026, Entscheidung des Nutzers): Gefällt dem Spieler sein Blatt nicht, mischt er es vor der Ansage ein: Alle Hände und der Skat werden neu gegeben, das kostet kein Spiel der Serie. Bis zu zweimal pro Spiel (`redeals` in `run.ron`), danach muss eines der Blätter gespielt werden. Nach dem Aufnehmen des Skats und im Ramsch geht es nicht mehr. Einpassen (ein Spiel auslassen, das ein Spiel vom Limit kostete) gibt es nicht mehr. Einzelheiten in 5.1.

**Verlorene Spiele kosten das Doppelte.** Wie im echten Skat wird ein verlorenes Spiel mit dem doppelten Spielwert negativ gewertet. Das ist der wichtigste Risiko-Hebel des gesamten Spiels und bleibt.

**Kein Überreizen.** Da es kein Reizen gibt, gibt es auch keinen Überreiz-Fehler. Die Spitzen werden nach dem Spiel mit dem tatsächlichen Blatt inklusive Skat gezählt.

**Ramsch, Bock, Kontra, Schieberamsch, Revolution, Schenken** sind keine Standardregeln, sondern Relikte, Boss-Mechaniken oder Ereignisse (siehe Abschnitt 8).

### 4.4 Regelklärungen für das veränderbare Blatt (M2)

Diese Regeln gibt es im echten Skat nicht, weil das Blatt dort nie verändert wird. Entschieden von Claude am 01.11.2026 im Rahmen der Autonomie-Regel.

- **Spitzen zählen nur über Trümpfe, die im Spiel sind.** Ist der Kreuz-Bube entfernt, beginnt die Zählung beim nächsten vorhandenen Trumpf. Sonst brächte jedes Entfernen eines Buben allen ein Gratis-„ohne“. Seit ADR 0015 wird keine Karte mehr entfernt; die Regel gilt weiter für Unter, die ein Karteneffekt zur Blanko macht (Boss-Regel der Oma). Die Spitzen zählen über die zwölf Karten des Alleinspielers **in seiner Ausgabe**: Ist ein Unter für ihn keine Unter (Nullspieler, 9.2), zählt er nicht, wie eine umgemalte Karte der Blue Curaçao (8.3).
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
- **Entschieden heißt:** (1) Kein Hook, der während des Spiels feuert (`OnCardPlayed`, `OnTrickEnd`, z. B. Schenken, Revolutionär, Auf dem Trockenen), kann noch feuern. (2) Das Ergebnis nach den Regeln (gewonnen, Schneider und Schwarz beider Seiten, Grundwert, Multiplikator, Spielwert) ist für jede Augen- und Stichzahl gleich, die die Reststiche noch bringen können, mit den geltenden Schwellen (Gewinnschwelle, „Jungfrau“). Null: Der Alleinspieler bekommt in keiner Fortsetzung einen Stich (gewonnen, mit gleichem Stilbonus: Siebenen, nie bedient, Kurzschluss) oder in jeder (verloren). (3) Jeder `OnScore`-Hook feuert in allen Fortsetzungen oder in keiner, auch die des letzten Stichs (Spitze, Glückspfennig, Schlusslicht). Ramsch wird nie abgekürzt.
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
    └── Boss-Kneipe      (zwei Boss-Spieler mit Sonderregel, alle 3 Spiele;
                          verloren: Run vorbei, ADR 0017)
```

| Ebene | Skat-Begriff | Genre-Gegenstück | Dauer |
|---|---|---|---|
| Ein Spiel | Skat, Ansage, 10 Stiche, Abrechnung | Eine Hand | ~90 Sekunden |
| Eine Kneipe | bis zu 3 Spiele mit Punkteziel gegen zwei Stammgäste, je Spiel bis zu zweimal einmischen | Ein Blind | 2 bis 5 Minuten |
| Ein Stadtteil | vier Kneipen und die Boss-Kneipe | Ein Ante | 15 bis 20 Minuten |
| Der Run | Kneipentour durch drei Stadtteile (15 Kneipen) | Der Run | 45 bis 60 Minuten |

### 5.0 Die Kneipentour (ADR 0016)

- **Kneipe**: Zielwert in Augen (Reizwerte wie bisher die Serie), bis zu drei Spiele gegen zwei Stammgäste. Wer den Zielwert erreicht, ist sofort fertig. Belohnung: Grundbetrag (`series_reward`, 3) plus je übrigem Spiel `per_game_left` (2; ADR 0016 setzte 3, gemessen am 03.11.2026 waren damit die fünf Relikt-Plätze schon zu Beginn von Stadtteil 2 voll), dazu Zinsen wie bisher (die Auszahlung, `Payout`). Im Code bleibt der Typ `Series` (Glossar).
- **Verfehlt** (Zielwert nach dem dritten Spiel nicht erreicht): ein Leben weg, weiter zur Routenwahl ohne Belohnung (`PubLost`, `LivesChanged`). Hooks nach einer gewonnenen Kneipe (`OnSeriesWon`) feuern nicht.
- **Leben** (`lives` in `run.ron`): Start 2, höchstens 4, eine geschaffte Boss-Kneipe +1 (seit 03.11.2026; ADR 0016 setzte Start 3, höchstens 5: Dann endeten nur 2 % der verlorenen Runs in normalen Kneipen, 5.0.4). Mit dem letzten Leben ist der Run verloren (`RunLost`). Leben zählen nur in normalen Kneipen (ADR 0017). Die Web-App zeigt sie als Herzen.
- **Boss-Kneipe (ADR 0017, Entscheidung des Nutzers vom 03.11.2026)**: letzte Kneipe eines Stadtteils. Am Tisch sitzen **zwei Boss-Spieler**, die beiden Figuren des Bosses (links `BOSS_SEAT`, `Owner::Boss`; rechts `PARTNER_SEAT`, `Owner::BossPartner`), jede mit eigenem Namen, eigener Ausgabe und der Rückseite des Bosses (11). Der Boss wird beim Start des Stadtteils gezogen und ist ab dann sichtbar. **Alle drei Spiele werden gespielt**, der Zielwert zählt nach dem dritten (kein frühes Ende). **Verloren ist der Run vorbei** (`RunLost` mit `boss`), egal wie viele Leben übrig sind; der Schutzengel hilft hier nicht. **Gewonnen**: doppelte Auszahlung (`Payout::boss`), gerechnet, als blieben zwei Spiele übrig (`boss_reward.games_left_as` in `run.ron`, seit 03.11.2026; vorher fiel der Bonus je übrigem Spiel weg, und eine gewonnene Boss-Kneipe zahlte mit 6,45 Groschen weniger als eine früh geschaffte Kneipe mit 7,93, `docs/balance/2026-11-03-kleine-runde.md`), ein Leben und ein Relikt nach Wahl aus drei der Seltenheit Ungewöhnlich oder höher (`RunPhase::BossReward`, `RunAction::ChooseBossRelic`, `boss_reward` in `run.ron`); sind alle Plätze belegt, wird ein eigenes Relikt für den halben Preis verkauft, oder der Spieler verzichtet. Im letzten Stadtteil sitzt der Endboss „Der Alte“ mit „Der Alten“; ihn zu schlagen gewinnt den Run (ohne Wahl). Bis ADR 0017 saß ein Boss links und ein Stammgast rechts, eine verlorene Boss-Kneipe kostete ein Leben und wurde sofort wiederholt.
- **Hausregeln des Stammtisch-Streits** gelten bis zum Ende des Stadtteils (bisher des Abschnitts).
- **Einmischen, Schnäpse, Relikte „einmal pro Serie“** zählen je Kneipe.
- **Kneipe verlassen** (seit 03.11.2026, 8.7): Zwischen zwei Spielen, ab dem zweiten, kann der Spieler eine Kneipe aufgeben. Sie zählt als verloren (ein Leben, keine Auszahlung; der Schutzengel verschont auch sie); eine aufgegebene Boss-Kneipe beendet den Run. Vor dem letzten Spiel zeigt die Kopfzeile, was noch nötig ist.

#### 5.0.1 Routenwahl und Stationen

Nach jeder Kneipe, die den Run nicht beendet, wählt der Spieler einen von **drei Wegen** zur nächsten Kneipe (`RunPhase::Route`, `RunAction::ChooseRoute`). Vorher sichtbar: die nächste Kneipe mit Namen, Stadtteil, Zielwert, Spielen und den Stammgästen am Tisch (in der Boss-Kneipe der Boss, sobald bekannt).

- **Verkehrsmittel**: Zu Fuß, Bahn, Fahrrad. Seit der Nachentscheidung des Nutzers vom 02.11.2026 **verhalten sich alle drei gleich**: Jeder Weg führt über genau eine Station, kostet nichts extra, und die Kategorie der Station wird **je Weg gewürfelt** und angezeigt. Das Verkehrsmittel ist Darstellung (Weg auf der Karte, eine dekorative Fahrzeit `minutes`). Der erste Entwurf (Bahn kostet 1 Groschen und zeigt das genaue Ereignis, Fahrrad zwei Stationen mit einer Überraschung) ist verworfen.
- **Stationen** (`StopCategory`): **Tresen** (der Laden wie bisher), **Begegnung** (die Ereignisse aus 13), **Upgrade** („Werkstatt“, selten: seit den Zinken am Tresen (8.6) der **Kartenmaler gratis** – eine gewählte Karte, dann eine von zwei gezogenen Zinken – oder **Karte reinigen** gratis; dazu der Kartengeber, der Gasthof zur Krone und der Taschenspieler, die Karten aufwerten), **Rast** (Imbissbude: ein Leben zurück, nur gezogen, solange ein Leben fehlt) und seit dem 03.11.2026 **Fundgroschen** (`FoundCoins`: 2 Groschen liegen auf dem Weg), **Kiosk** (`Kiosk`: ein kleiner Laden mit zwei Schnäpsen, je 1 Groschen billiger als am Tresen nach dem Rabatt des Wirts, mindestens 1; einer darf gekauft werden, Kaufen ist freiwillig; keine Relikte, keine Zinken, keine Neuauslage, kein Verkauf von Relikten, kein Anschreiben) und **Würfelbude** (`DiceStall`: 2, 4 oder 6 Groschen setzen, mit 50 % kommt der doppelte Einsatz zurück; wer keine 2 Groschen hat, darf nur zuschauen). Jedes Ereignis trägt seine Kategorie als Daten (`category` in `encounters.ron`); Fundgroschen und Würfelbude sind Ereignisse (`found_coins`, `dice_stall`), der Kiosk ist ein Laden (`Shop::kiosk`, `RunEvent::KioskOpened`).
- **Wege und Begegnungen v2 (Entscheidung des Nutzers vom 03.11.2026, Cockpit-Fragen 10 und 11)**: Die drei Wege einer Routenwahl zeigen immer **drei verschiedene Stationsarten**, eine davon der **Tresen** (auf einem zufälligen Weg). Die beiden anderen werden ohne Zurücklegen nach `route.weights` gezogen, vorläufig nach Augenmaß **Begegnung 35, Fundgroschen 30, Kiosk 15, Würfelbude 10, Werkstatt 7, Rast 5** (Rast nur mit fehlendem Leben; der Tresen hat kein Gewicht mehr, bis dahin Tresen 40, Begegnung 35, Upgrade 12, Rast 8 mit „mindestens ein Tresen“). Gezogen wird nur, was gerade etwas zu bieten hat: eine Kategorie mit einem Ereignis, von dem eine Option genommen werden kann, der Kiosk mit einem Schnaps für den Charakter. Die Spelunke bleibt, wie sie ist. Das genaue Ereignis ist beim Wählen gezogen, aber verdeckt; die Kategorie ist sichtbar.
- **Ein Ereignis ohne Angebot** (keine Option, die gerade genommen werden kann) wird nicht auf einen Weg gelegt; geht eines unterwegs doch leer aus, geht es direkt in die Kneipe.
- **Zufall**: eigener Generator des Runs für Wege, Spelunken und Mali (`tour_rng`); die Karten der Spiele ändern sich dadurch nicht.

#### 5.0.2 Die Spelunke

Mit der Wahrscheinlichkeit `spelunke.chance` (vorläufig 0,3) führt **einer** der drei Wege statt zur nächsten Kneipe in eine **Spelunke**, nie vor einer Boss-Kneipe. Die Spelunke ist **ein eigener Ort**: eigener Name aus eigener Wortliste (`city.spelunken` in `tour.ron`), eigener Knoten (`d0-p2-s`); sie steht in der Reihenfolge des Stadtteils für die Kneipe, die sie ersetzt (diese gilt als übergangen, `PubState::Skipped`).

- **Zielwert**: der der ersetzten Kneipe × `spelunke.target_factor` (seit 03.11.2026 vorläufig 1,1, vorher 1,25).
- **Malus**: einer, zufällig nach Gewicht, vor der Wahl angezeigt (Antwort-Vorschlag der offenen Cockpit-Frage „ein zufälliger Malus, vorher sichtbar“, umgesetzt als Voreinstellung). Mali sind Daten (`maluses` in `tour.ron`): **Kein Einmischen** (`no_redeal`), **Klebrige Karten** (zwei zufällige Karten der Ausgabe ohne Modifikation sind ramponiert, Augen −5, nur für diese Kneipe, ohne die Groschen eines Fluchs; `curse`), **Kein Skat** (jedes Spiel Hand, `passive: [RequireHand]`, nicht für den Nullspieler) und **Schnapsverbot** (`lock_schnapps`, Fehler `ConsumablesLocked`). Tischregeln wirken als Quelle `Source::Malus` wie eine Boss-Regel.
- **Belohnung**: die normale Auszahlung, dazu die **Beute**: ein Relikt, gezogen beim Gewinnen, in einen freien Platz oder für den halben Preis in Groschen (Ereignis `spelunke_loot`). Verloren kostet sie ein Leben wie jede Kneipe.

#### 5.0.3 Die Stadt und ihre Namen

Stadtteile und Kneipen heißen nach einem **Namensgenerator** mit deutschen Wortlisten in `tour.ron` („Zum Goldenen Hirschen“, „Eckkneipe bei Uschi“, „Bahnhofsklause“, Boss-Kneipen wie „Vereinsheim Gut Blatt“), je Run aus dem Seed, ohne einen anderen Zufallsstrom zu berühren. Namen sind Anzeige. Die **Kartengeometrie** (Straßen, Lage) gehört der Web-App: Die Sicht liefert stabile Knoten-Ids (`d<Stadtteil>-p<Kneipe>`, Spelunke mit `-s`, Wege `…-r<n>`), je Stadtteil und für die Stadt einen Seed (`map_seed`) und den ganzen Plan des Abends (`TourView`: Stadtteile, Kneipen mit Zustand gewonnen, verloren, aktuell, kommend, übergangen, Spelunken, Boss des laufenden Stadtteils), damit die Web-App eine Übersicht und später eine Pixel-Stadtkarte zeichnen kann.

#### 5.0.4 Zahlen der Kneipentour

Gesetzt am 02.11.2026 nach Augenmaß; **Zielwerte, Leben, Groschen je übrigem Spiel, Boss-Faktoren und Malus-Gewichte gemessen am 03.11.2026** gegen den PIMC-Spieler (`--bot pimc:12:3`, Alleinspieler). Kleine Runde (Ist-Stand, 3 × 500 Runs): `docs/balance/2026-11-03-kleine-runde.md`; Piloten und große Runde (3 × 2000 Runs, Seeds 42 bis 44): `docs/balance/2026-11-03-grosse-runde.md`.

**Zielbild (03.11.2026)**: Der PIMC-Spieler erreicht Stadtteil 2 in 35 bis 40 % der Runs und Stadtteil 3 in 12 bis 18 %, gewinnt 3 bis 5 % der Abende, und ein Viertel bis ein Drittel der verlorenen Runs endet in normalen Kneipen (Leben aus), der Rest beim Boss.

| Kennzahl | vorher (kleine Runde) | jetzt (große Runde) | Ziel |
|---|---|---|---|
| Stadtteil 2 erreicht | 57,4 % | 42,3 % | 35 bis 40 % |
| Stadtteil 3 erreicht | 34,5 % | 17,4 % | 12 bis 18 % |
| Run gewonnen | 17,0 % | 5,2 % | 3 bis 5 % |
| verlorene Runs in normalen Kneipen | 2,6 % | 36,1 % | 25 bis 33 % |
| normale Kneipen gewonnen | 77 bis 86 % | 38 bis 74 % | |
| Boss-Kneipen gewonnen | 35 bis 72 % | 36 bis 58 % | |

**Warum so steil**: Die Erfolgskurve einer Kneipe ist flach (0,3 Punkte Gewinnquote je Zielwert-Punkt: gewonnen wird groß, verloren doppelt), und mit fünf Relikten bringt ein im ersten Spiel geschaffter Sieg in Stadtteil 3 im Mittel 92 Punkte. Die vorgeschlagene Kurve 40 bis 108 (Faktor 1,5 bis 2) bewegte „Run gewonnen“ nur von 19 auf 16 %; erst mit Zielwerten bis gut 200 und zwei Startleben beißen die normalen Kneipen.

| Größe | Wert | Ort |
|---|---|---|
| Stadtteile, Kneipen je Stadtteil | 3, 4 + Boss-Kneipe | `targets` in `run.ron`, `PUBS_PER_DISTRICT` |
| Zielwerte Alleinspieler (seit 03.11.2026, gemessen) | 45 / 55 / 65 / 75 / Boss 60 · 90 / 105 / 120 / 135 / Boss 100 · 145 / 170 / 195 / 205 / Boss 140 (bis dahin 33 / 36 / 38 / 40 / 45 · 40 / 42 / 44 / 46 / 52 · 46 / 48 / 50 / 52 / 60) | `run.ron` |
| Zielwerte Nullspieler (seit 03.11.2026, gemessen) | 50 / 65 / 75 / 85 / Boss 70 · 110 / 125 / 145 / 160 / Boss 120 · 180 / 210 / 245 / 255 / Boss 175, die Kurve des Alleinspielers × 1,15 / 1,2 / 1,25 (bis dahin 18 bis 30: Stadtteil 2 in 60 %, 30 % gewonnen) | `characters.ron` |
| Zielwerte Schneiderin (seit 03.11.2026, gemessen) | 50 / 65 / 75 / 90 / Boss 75 · 105 / 120 / 140 / 155 / Boss 115 · 165 / 195 / 225 / 235 / Boss 160, rund × 1,15 (mit der Kurve des Alleinspielers 47 % Stadtteil 2, 7,9 % gewonnen) | `characters.ron` |
| Zielwerte Drückeberger, Bruderherz | wie der Alleinspieler (`targets` fehlt), gemessen am 03.11.2026 im Zielbild | `characters.ron` |
| Boss-Faktoren | gemessen am 03.11.2026 (11.1): Alleinspieler in der großen Runde, Nullspieler in der Runde der Charaktere (Schieber 0,2, Ramscher 2,0, Oma 0,9, Peter 0,2) | `bosses.ron` |
| Boss-Belohnung | Auszahlung wie mit zwei übrigen Spielen, × 2 (im Mittel 14,6 Groschen), +1 Leben, 1 aus 3 Relikten ab Ungewöhnlich | `economy`, `lives.boss_bonus`, `boss_reward` |
| Gegner je Stadtteil | Heuristik, PIMC 8:3, PIMC 12:3 (die alten Abschnitte 1, 3, 4) | `opponents` |
| Leben | Start 2, höchstens 4, Boss +1 (bis 03.11.2026 3, 5, +1) | `lives` |
| Grundbetrag, je übrigem Spiel | 3, 2 (03.11.2026; ADR 0016 3, davor 1) | `economy` |
| Gewichte der Stationen | seit 03.11.2026 drei verschiedene Arten je Routenwahl, eine der Tresen; die anderen ohne Zurücklegen: Begegnung 35, Fundgroschen 30, Kiosk 15, Würfelbude 10, Werkstatt 7, Rast 5 (Rast nur mit fehlendem Leben), vorläufig. Bis dahin Tresen 40, Begegnung 35, Upgrade 12, Rast 8 | `route.weights`, `route.kiosk` |
| Spelunke | Chance 0,3, Zielwert × 1,1 (bis 03.11.2026 1,25), ein Malus (Gewicht 2, Kein Skat 1, seit 03.11.2026: Kein Skat war 13 bis 23 Punkte schwerer als die anderen) | `spelunke`, `tour.ron` |
| Rast | +1 Leben | `encounters.ron` (`snack_bar`) |
| Bot-Route | Tresen ab 8 Groschen, Upgrade mit verfluchter Karte, Rast mit fehlendem Leben, Fundgroschen 2, Kiosk 1, Würfelbude 0; Spelunke nur mit 3 Leben und 3 Relikten | `RunBot::route_action` |
| Bot-Ereignis (seit 03.11.2026) | eine Option muss genommen werden: die mit dem höchsten Wert in groben Groschen (`RunBot::option_value`, nicht abgestimmt); am Kiosk der wertvollste Schnaps | `RunBot::encounter_action` |
| Bot-Boss-Relikt | das wertvollste in einen freien Platz, bei vollen Plätzen statt des schwächsten, wenn es mindestens `SWAP_MARGIN` mehr wert ist, sonst keins | `RunBot::boss_reward_action` |

Gemessen am 03.11.2026 für den Alleinspieler (große Runde) und für die anderen vier Charaktere (je 3 × 300 Runs, `docs/balance/2026-11-03-charaktere.md`, Tabelle in 9). Noch zu messen: die Gewichte der Stationen und die Spelunke selbst (mit der steilen Kurve in Stadtteil 3 nur noch 20 bis 36 % gewonnen, 17).

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

Die folgende Kalibrierung galt der alten Struktur aus fünf Abschnitten. Die Zielwerte der Kneipentour (gemessen am 03.11.2026) stehen in 5.0.4.


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
# seit 03.11.2026: --character declarer|null|shirker|tailoress|heart_brother
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
6. **Abrechnung.** Augen zählen, Spitzen bestimmen, Multiplikator summieren, Spielwert berechnen, Serie-Konto aktualisieren.
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

1. **Eine Modifikation je Karte.** Jede Karte trägt höchstens eine Modifikation. Nur Spezialkarten (Untot, Wandelkarte) dürfen zusätzlich eine normale Modifikation tragen. Seit dem 03.11.2026 ist höchstens eine Karte je Ausgabe Untot und höchstens eine Wandelkarte (unten, „Spezialkarten im Run“).
2. **Steigerung.** Kommt dieselbe Modifikation noch einmal auf dieselbe Karte, steigt sie: Die Werte werden addiert (Augen +3 und Augen +5 ergeben Augen +8). Rang steigt höchstens bis 2 (`max` in den Daten; bis 02.11.2026 bis 3, gesenkt nach Antwort im Cockpit). Alte Spielstände mit Rang 3 kappt der Kern beim Laden auf 2 (`deck::RANK_MAX`, Version 3 der Modifikationen in `Edition`). Nicht steigerbar sind die Moment-Modifikationen (Letzter Stich, Erster Stich: höchstens einmal, ein zweites Mal gibt `AtMaximum`) und die Spezial-Gutscheine (ohne Wert, `AlreadyModified`); der Gutschein mit Groschen addiert, Augen und Trumpf haben keine Obergrenze (entschieden vom Lead am 02.11.2026).
3. **Ersetzen.** Eine andere Modifikation ersetzt die bisherige. Die Oberfläche fragt vorher nach; der Kern ersetzt ohne Rückfrage und meldet das Ersetzte (`DeckChange::Modified.replaced`).
4. **Ein Fluch ist die eine Modifikation.** Eine verfluchte Karte kann weder gesteigert noch ersetzt werden (Fehler `DeckError::Cursed`, die Karte fehlt in jeder Zielauswahl). Nur Reinigen entfernt den Fluch (seit 02.11.2026 „Karte reinigen“ am Tresen und in der Werkstatt, 8.6; bis dahin der Gegenstand Frisches Deck, 7.3). Ein Fluch, der auf eine Karte mit normaler Modifikation kommt, ersetzt sie.
5. **Augen und Ramponiert sind ein Wert** (Ergänzung des Nutzers vom 02.11.2026). Eine Augen-Modifikation mit negativem Wert ist der Fluch Ramponiert: ab +1 heißt sie Augen, ab −1 Ramponiert, mit allen Regeln eines Fluchs (sperrt die Karte, nur Reinigen entfernt sie). Kommt ein Wert zu einer Karte mit Augen dazu (etwa Ramponiert −5 vom Umgekippten Bier auf Augen +3), wird addiert, und das Vorzeichen der Summe entscheidet: −2 ist Ramponiert, +1 bleibt Augen, 0 nimmt die Modifikation ab. Auf eine ramponierte Karte kommt nichts mehr dazu. Im Kern zwei Ids (`points`, `battered`), `Deck::add_mod` rechnet mit dem Vorzeichen.
6. **Heimweh ist gestrichen** (vorerst).
7. **Für wen sie wirken.** Seit M9 (eigene Ausgaben, 7.1) wirkt jede Modifikation nur in der Ausgabe ihres Halters: Eine Karte des Spielers mit Augen +4 zählt +4, solange er sie hält, spielt oder im Skat hat; hält ein Gegner die Karte, spielt dessen Ausgabe. Trumpf und Fleckig wirken streng nur, wenn der Spieler die Karte hält (Antwort im Cockpit: streng; der Schalter `TRUMP_MODS_NEED_PLAYER` in `core/src/content/engine.rs` bleibt, ist mit den Ausgaben aber ohne Wirkung).

**Visuelle Grammatik** (die Bilder werden getrennt neu gezeichnet; bis dahin zeigt die Web-App einfache Platzhalter auf dem vorhandenen Rahmen-Code):

| Ort auf der Karte | Bedeutung |
|---|---|
| Rahmen | **was** betroffen ist (Augen, Rang, Trumpf, Stich, Gutschein) |
| Markierung oben rechts | **wann** sie wirkt (keine Markierung: immer) |
| Schrift unten Mitte, auf einem Farbklecks-Band | **wie viel** (+5, −3, ×2, +1 Multiplikator) |
| Feld unten links, quadratisch | nur für Flüche und negative Effekte |
| ganzes Kartenbild | Spezialkarten ändern das Grunddesign (Untot, Wandelkarte) |

**Die Modifikationen**

| Gruppe | Modifikation (`id`) | Wirkung | Werte (Platzhalter) | Hebel | Bild |
|---|---|---|---|---|---|
| Rand | Augen (`points`) | ±X Augen | Tresen +3, +5; zufällig +4 | Augen | Rahmen Augen, Zahl im Band |
| Rand | Rang (`rank`) | sticht 1 oder 2 Stufen höher | +1, Steigerung bis 2 | Stichgewinn | Rahmen Rang, Winkel nach oben statt Zahl |
| Rand | Trumpf (`trump_bonus`) | Grundwert +X, wenn die Farbe der Karte Trumpf ist und der Spieler sie hält | Tresen +2, +4; zufällig +3 | Grundwert | Rahmen Trumpf, Zahl im Band |
| Moment | Letzter Stich (`last_trick`) | +1 Multiplikator, wenn diese Karte den letzten Stich macht | +1, nicht steigerbar | Multiplikator | Markierung oben rechts, Text unten Mitte |
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
- **Gutschein, „wenn gedrückt“.** Wirkt außerhalb des laufenden Spiels: Landet die Karte beim Drücken im Skat, wird der Gutschein sofort eingelöst, egal wie das Spiel ausgeht, und fällt danach von der Karte (Auslöser `OnDiscard`, Ereignis `VoucherRedeemed`). Bei „Drei drücken“ löst die Karte, die an den Gegner zurückgeht, nichts ein. Im Handspiel wird nichts gedrückt, der Gutschein bleibt. Die Spezial-Gutscheine machen die Karte zur Spezialkarte (Wirkung `TurnInto`); eine andere Spezialkarte auf derselben Karte wird ersetzt. Ist schon eine andere Karte der Ausgabe diese Spezialkarte, zahlt die Marke statt dessen 3 Groschen (seit 03.11.2026, unten).
- **Zufallsziehungen** (bis 02.11.2026 Stempelkissen; Kartengeber; seit 8.6 auch Auslage und Kartenmaler, dort zusätzlich nach Seltenheit) ziehen aus Rand, Moment und Gutschein, gewichtet nach `focus_weights` wie bisher, nie einen Fluch und nie direkt eine Spezialkarte (die kommt über den Gutschein; einzige Ausnahme seit 03.11.2026 das seltene Spezialkarten-Angebot der Auslage, 8.6) und nie eine Toten- oder Narrenmarke, deren Spezialkarte die Ausgabe nicht mehr bekommen darf; der Wert ist der erste aus `values`. Liegt auf der Karte schon dieselbe Modifikation, steigt sie; eine andere wird ersetzt.
- **Tresen.** Bis 02.11.2026 gab es konkrete Werte als Verbrauchsgegenstände (`ApplyCardMod`: Augen +3, Augen +5, Trumpf +2, Trumpf +4, Rang +1). Seit den Zinken (8.6) verkauft der Tresen Modifikationen nur noch in der Auslage und über den Kartenmaler; die Aktionen ruhen.
- **Anzeigenamen seit dem 02.11.2026** (Entscheidung des Nutzers, 8.6): Augen heißt **Aufdruck**, Rang **Winkel**, Trumpf **Siegel**, Erster Stich **Eselsohr**, Letzter Stich **Kerbe**, Gutschein (Groschen) **Pfandmarke**, Gutschein Untot **Totenmarke**, Gutschein Wandelkarte **Narrenmarke**. Die Tabelle oben nennt die Wirkung; Ids und Bezeichner im Code bleiben. Jeder Text beginnt mit der Wirkung („Erster Stich: Gewinnt diese Karte …“), damit Infokarten sie neben dem Namen zeigen.
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
- **Rest abkürzen** wartet, solange eine Karte der Hand noch steigen kann. Eine untote Wandelkarte kommt als die festgelegte Karte zurück und bleibt bis zum Spielende so. **Drei drücken, Blue Curaçao** wirken vor dem ersten Stich und ändern nichts an der Regel.
- **KI:** Die Suche kennt das Steigen nicht; sie spielt die Karte wie jede andere. Bot (`RunBot::undead_card`): Spielt er als Letzter zum Stich und nimmt eine untote Karte, die noch steigen kann, den Stich, spielt er sie. Sicht: `CardView::can_rise`.

Tests: `core/src/run/undead_tests.rs`, je Regelkante einer.

**Wandelkarte** (`wild_card`, Effekt `Wild`, seit Effekte v2 eine Spezialkarte über den Gutschein Wandelkarte). Regel neu entschieden vom Nutzer am 03.11.2026, Einzelheiten von Claude; sie ersetzt das Benennen beim Ausspielen (bis dahin `RunAction::PlayAs`, `rules::wild_suits`, Ereignis `WildCardNamed`, „zwingt nie zum Bedienen“). Text: „Bei Spielbeginn nimmt sie die Farbe an, von der du die meisten Karten hältst (ohne Unter). Bei Gleichstand gilt Eichel, Grün, Rot, Schellen.“

- **Wann:** im Moment des Spielbeginns, also nach Aufnehmen und Drücken mit der Ansage, im Handspiel mit der Ansage, auch im Null und im Ramsch (`Game::declare`). Bis dahin ist sie ihre gedruckte Karte, auch bei der Wahl der Drückkarten. Eine gedrückte Wandelkarte (und eine im Skat eines Handspiels) bleibt gedruckt.
- **Welche Farbe:** die Farbe, von der der Spieler in seinen zehn Karten die meisten hält, gezählt nach gedruckter Farbe, ohne alle Unter und ohne die Wandelkarte selbst (`rules::settled_wild_suit`). Gleichstand: Eichel vor Grün vor Rot vor Schellen. Die Taktik dahinter: die zweitlängste Farbe drücken, damit die Wandelkarte in die Farbe kommt, die Trumpf werden soll. Mehrere Wandelkarten (selten, höchstens eine je Ausgabe kommt später) zählen alle nicht mit und nehmen alle dieselbe Farbe an.
- **Was sich ändert:** Rang, Augen und die übrigen Modifikationen bleiben; die Karte ist bis zum Spielende die Karte dieser Farbe mit demselben Rang (in Hand, Deck und allen Ausgaben). Doppelte Karten sind erlaubt wie bei der Blue Curaçao (4.4). Ab dann gilt die normale Bedienpflicht für die neue Farbe; nichts wird beim Ausspielen benannt, und sie befreit nicht vom Bedienen. Eine Wandelkarte, die ein Unter ist, bleibt der Unter, unverändert. Die Spitzen bleiben, wie angesagt (gezählt über die gedruckten Karten). In der Hand eines Gegners ist sie die gedruckte Karte.
- **Ereignis und Sicht:** `Event::WildCardSettled { old, new }` (auch für den Spieler, `PlayerGameEvent::WildCardSettled`), je Wandelkarte eines, direkt nach `Declared`; die Web-App kann es animieren. `DealtCard::wild` heißt seitdem „noch offen“ und fällt beim Festlegen ab. Sicht: `CardView::unsettled` (noch offen, vor Spielbeginn: der Harlekin in vier Farben), `CardView::named` (gedruckte Farbe einer festgelegten Wandelkarte, `ActiveGame::named`), `GameView::wild_settled` (alle festgelegten Wandelkarten des Spiels mit Karte und gedruckter Farbe, auch schon gespielte: für die Ausgabe auf der Kreidetafel nach dem Neuladen).
- **Darstellung** (entschieden vom Nutzer am 03.11.2026): Offen trägt der Narr ein Harlekinkostüm in den vier Farben der Farben (Eichel-Blau, Grün, Rot, Schellen-Gelb), im Eckzeichen stehen der Rang und vier kleine Farbfelder. Festgelegt sind Kostüm und Kappe ganz in Tönen der neuen Farbe, das Eckzeichen ist das normale der Farbe (`tools/cardart/wild.py`, Palettentausch). Beim Festlegen laufen die Farben etwa 0,8 Sekunden durch alle vier und bleiben auf der neuen stehen (ohne Bewegung: sofort). Hand, Stich, Infokarte, Ausgabe auf der Kreidetafel und Abrechnung zeigen sie festgelegt, außerhalb eines Spiels (Tresen) offen.
- **Untot:** Eine untote Wandelkarte (nur aus alten Spielständen, die beide Spezialkarten behalten) steigt als die festgelegte Karte zurück.
- **Bot:** Nach dem Festlegen ist sie eine Karte wie jede andere; Suche und Bot sehen die festgelegte Karte (`opponents::tame_game` und `RunBot::wild_suit` gestrichen). Beim Drücken und bei der Ansage rechnet der Bot noch nicht voraus, welche Farbe sie annimmt.
- **Spielstände:** Ohne neue Version. Ein laufendes Spiel vor der Ansage legt sie bei der Ansage fest. Ein Spiel, das vor dem 03.11.2026 schon lief und noch eine Wandelkarte in der Hand hat, behält sie als gedruckte Karte (sie bedient dann normal); die Sicht zeigt sie nicht mehr als offen. Die Aktion `PlayAs` gibt es nicht mehr; alte Ereignisse `WildCardNamed` werden nicht mehr gelesen.

**Spezialkarten im Run** (entschieden vom Nutzer am 03.11.2026, „A + B“): (A) Ein Boss hat Spezialkarten in seiner Ausgabe; beim ersten Erhalt gibt es einen Hinweis, und die Wirkung wird deutlich animiert. (B) Die Boss-Belohnung „1 aus 3“ enthält immer eine Spezialkarte, selten auch die Tresen-Auslage; höchstens ein Untot und eine Wandelkarte je Ausgabe. Eigene Relikte für Spezialkarten gibt es nicht.

Stand der Umsetzung (03.11.2026): Der Boss mit Spezialkarten (Totengräber) ruht, und die Boss-Belohnung ist seit ADR 0017 ein Relikt aus dreien; aus (A) und (B) umgesetzt sind die Obergrenze, das seltene Angebot der Auslage (8.6), der Hinweis beim ersten Erhalt und die Animation. Auslegungen von Claude:

- **Obergrenze überall.** Höchstens eine Karte der Ausgabe ist Untot, höchstens eine Wandelkarte (jede `ModKind::Special`; Abgestuft ist ein Spezialfluch und nicht begrenzt). Der Kern prüft das an einer Stelle, in `Edition::add_mod` (Fehler `DeckError::SpecialTaken`, Code `SpecialTaken`), also für Marken, Ereignisse, Kartenmaler und Auslage gleich. Auf der eigenen Karte gelten die alten Regeln (nochmal dieselbe: `AlreadyModified`; die andere Spezialkarte ersetzt sie nur, wenn diese noch frei ist).
- **Marke über der Obergrenze.** Wird eine Totenmarke (Narrenmarke) eingelöst, während eine andere Karte schon Untot (Wandelkarte) ist, bleibt die Karte, wie sie ist, die Marke fällt ab und zahlt 3 Groschen (`capped_voucher_coins` in `run.ron`, nach Augenmaß wie eine Pfandmarke), Ereignis `VoucherCapped { card, card_mod, special, holder, coins }`, dann `CoinsGained`. Die Oberfläche sagt es in einem Satz („… ist schon Untot. Dafür 3 Groschen.“). Ist die gedrückte Karte selbst schon Untot (die zurückgekehrte Marke des Drückebergers, 9.4), passiert wie bisher nichts, und es wird nichts gezahlt.
- **Angebote halten sich daran.** Eine Spezialkarte ist „frei“, solange keine Karte sie ist und keine Karte die Marke dafür trägt (`Run::special_open`). Auslage, Kartenmaler, Kartengeber, Taschenspieler, Falschspieler und Alte Karten im Schrank bieten weder eine nicht freie Spezialkarte noch ihre Marke an; hat ein Ereignis dann nichts mehr, findet es nicht statt. So landet nichts, was der Spieler kauft, in der Rückfallzahlung.
- **Alte Spielstände** behalten mehrere Untot oder Wandelkarten, die Regeln des Spiels kommen damit zurecht (Tests); eine weitere kommt nicht dazu.
- **Hinweis beim ersten Erhalt.** Bekommt eine Karte der Ausgabe zum ersten Mal in einem Meta-Spielstand eine Spezialkarte (`DeckChanged` mit Untot oder Wandelkarte), zeigt die Web-App einmal ein Blatt „Neu in deiner Ausgabe“: die Karte, drei Sätze Regel, die Obergrenze. Gemerkt im Meta-Spielstand (`MetaState::specials_seen`, ohne neue Version; fehlt in alten Ständen und ist dann leer), nicht im Browser, damit es mit dem Spielstandscode umzieht. Der Kern meldet die neuen in `MetaStep::first_specials`.
- **Animation.** Steigt eine untote Karte auf, fliegt sie in einem höheren Bogen aus dem Stich zurück in die Hand und leuchtet dabei kurz grün (etwa 1,3 Sekunden); mit reduzierter Bewegung steht nur ein grüner Rand still da, solange die Meldung dauert. Die Wandelkarte hat ihren Farbdurchlauf schon.

**Wirkt sie in diesem Spiel?** (01.11.2026, gilt weiter): Nach der Ansage sagt der Kern für jede sichtbare Karte, ob ihre bedingten Modifikationen in diesem Spiel wirken (`CardView.mod_status`: `Active`, `Inactive`, `Open`; `card_mod_status`). Bedingt sind Trumpf und Fleckig („Ist ihre Farbe Trumpf“) und die Moment-Modifikationen (erster und letzter Stich). Es zählt nur, was der Spieler weiß.

### 7.3 Karten hinzufügen und entfernen (entfällt seit ADR 0015)

Bis 02.11.2026 geplant und teils umgesetzt: Luschen entfernen (eine Blanko tritt an ihre Stelle), Fünfter Bube, Doppel-Ass, „Sechs“, Joker, Kopieren einer Karte samt Modifikationen; ein Talon über 32 Karten.

Seit ADR 0015 verlässt keine Karte das Blatt und keine kommt hinzu. Was davon bleibt, ist ein Karteneffekt auf der Ausgabe des Spielers (entschieden von Claude am 02.11.2026, Frisches Deck als Frage im Cockpit):

| Bisher | Mit eigener Ausgabe |
|---|---|
| Frisches Deck: eine Karte entfernen, eine Blanko tritt an ihre Stelle | **Reinigen** (Antwort im Cockpit, umgesetzt in M9): „Reinigt eine Karte deiner Ausgabe: Alle Effekte fallen ab, auch Flüche.“ Auch Spezialkarten und Abgestuft fallen ab (Aktion `CleanseCard`, `Edition::cleanse`, Ereignis `DeckChange::Cleansed`; eine Karte ohne Modifikation ist kein Ziel, Fehler `NothingToCleanse`). Der Bot kauft es nur, wenn eine Karte einen Fluch trägt, und reinigt sie sofort |
| Luschen entfernen, damit mehr Augen auf die Hand kommen | entfällt; der Hebel Augen läuft über die Modifikation Augen und Hausregeln |
| Fünfter Bube | entfällt (Unter ehrenhalber ist mit Effekte v2 gestrichen) |
| Doppel-Ass, Joker | Blue Curaçao (8.3) und Wandelkarte (7.2): eine eigene Karte wird für ein Spiel zu einer anderen; doppelte Karten im Stich wie in 4.4 |
| Kopieren | Idee für `content`: „Abklatsch“, die Modifikationen einer eigenen Karte auf eine zweite übertragen |
| Startblatt des Nullspielers (ohne Unter, vier Siebener mehr) | Modifikation auf seinen Untern (9.2) |
| Glasdaus zerbricht und verlässt das Blatt | entfällt, der Glasdaus ist mit Effekte v2 gestrichen (7.2) |

Blankos gibt es weiter, aber nur als Wirkung auf eine Karte: Die Oma macht für ihre Serie jeden Unter zur Blanko (Tischregel, jede Ausgabe), Scherben gibt es nicht mehr (Glasdaus gestrichen, 7.2). Ein Stich bleibt immer einer von zehn.

### 7.4 Das Risiko sichtbar machen

Bis ADR 0015 zeigte das UI bei jeder Blatt-Änderung die Wahrscheinlichkeit, dass die Karte in der eigenen Hand oder im Skat landet (12 von n Karten, 37,5 % im Grundblatt, weniger mit Talon). Der Kern rechnet das aus (`DealOdds`, Feld `deal_odds` der Sicht, wasm-Funktion `dealOdds`).

Seit ADR 0015 ist das Blatt immer 32 Karten groß, die Chance also immer 12 von 32: „Deine Ausgabe spielt in 37,5 % der Spiele (Hand oder Skat). Sonst hält ein Gegner die Karte in seiner eigenen.“ `DealOdds` bleibt als Sicht, rechnet seit M9 nur noch mit 32 (`deal_odds()` ohne Blattgröße, kein Talon). Das Risiko, das der Spieler spüren soll, ist nicht mehr „hilft dem Gegner“, sondern „schläft“ (7.1, Regel 9); die Anzeige bei der Kartenwahl (bis 02.11.2026 beim Stempelkissen, seit 8.6 Kartenmaler und Karte reinigen) sagt das in einem Satz.

---

## 8. Relikte und Gegenstände

### 8.1 Hausregeln (passive Relikte, Joker-Äquivalent)

Hausregeln sind regionale Skat-Varianten, als passive Modifikatoren. Fünf Plätze im Grundzustand.

**Seit dem Gegenstands-Review vom 02.11.2026 (8.5)** sind Bockrunde, Kontra, Ouvert-Bonus, Durchmarsch, Blindflug, Lehrgeld, Auf dem Trockenen, Schneiderbock, Doppelter Boden, Glashaus, Kerbholz, Siebenschläfer, Saubere Weste, Altes Blatt, Wechselbalg, Bubenparade und Lockvogel entfernt; Drei drücken und Spickzettel sind geändert. Was hier über sie steht, ist Geschichte.

| Hausregel | Effekt | Hebel |
|---|---|---|
| **Bockrunde** | Nach jedem verlorenen Spiel: nächstes Spiel zählt doppelt | Grundwert ×2 |
| **Kontra** | Gegner dürfen vor dem ersten Stich Kontra sagen (Spielwert ×2). Du darfst Re sagen (×4). KI sagt Kontra, wenn sie dein Blatt für schwach hält. | Mult, Risiko |
| **Spitze** | Letzter Stich mit einer Sieben gibt +1 Multiplikator | Multiplikator |
| **Schenken** | Haben die Gegner nach 5 Stichen unter 15 Augen, geben sie auf. Spiel gilt als Schneider. | Tempo, Schneider |
| **Ouvert-Bonus** | Ouvert zählt +2 statt +1 | Multiplikator |
| **Schieben** | In jedem Spiel einmal mehr einmischen (seit dem Einmischen, 01.11.2026; vorher einmal pro Serie neu geben lassen) | Spitzen, Risikostufe |
| **Jungfrau** | Gewinnst du mit 61 bis 70 Augen, zählt das Spiel trotzdem als Schneider | Schwelle |
| **Alte Dame** | Jede Dame zählt 5 Augen | Augen |
| **Tournee** | Jede Serie, die du mit Spielen übrig gewinnst, gibt +1 Grundwert für den Rest des Runs | Skalierung |
| **Durchmarsch** | Gewinnst du Schneider, zählt der Spielwert doppelt | Spielwert, Schneider |
| **Blindflug** (Null) | Gewonnenes Null aus der Hand: +1 Multiplikator | Multiplikator |
| **Kleinvieh** | Jede Sieben und Acht zählt 1 Auge | Augen |
| **Ein Auge zudrücken** | Gewinn ab 58 Augen | Schwelle |
| **Lehrgeld** | Jedes verlorene Spiel: +2 Grundwert für den Rest des Runs | Grundwert (Zähler) |
| **Bierdeckel** | Jedes gewonnene Handspiel: +1 Grundwert für den Rest des Runs | Grundwert (Zähler) |
| **Auf dem Trockenen** | Gegner nach drei Stichen ohne Auge: +1 Multiplikator | Multiplikator |
| **Schneiderbock** | Nach einem gewonnenen Schneider zählt das nächste Spiel doppelt | Grundwert ×2 |
| **Glückssträhne** | Ab dem dritten Sieg in Folge zählt jedes gewonnene Spiel doppelt, bis eines verloren geht | Spielwert ×2 |
| **Nullachtfuffzehn** (Null) | Jedes Null: Grundwert +8, gewonnen wie verloren | Grundwert |
| **Doppelter Boden** (Null) | Gewonnenes Null mit Skat: +1 Multiplikator | Multiplikator |
| **Dicke Sau** (Null) | Jedes Daus zählt 16 Augen, die Gegner kommen schneller in den Kurzschluss | Augen |
| **Glashaus** (Null) | Gewonnener Null Ouvert zählt doppelt | Spielwert ×2 |
| **Kerbholz** (Null) | Jedes gewonnene Null: +2 Grundwert für den Rest des Runs | Grundwert (Zähler) |
| **Skatpfleger** (M8) | Die Augen der gedrückten Karten zählen doppelt | Augen |
| **Altes Blatt** (M8) | Jede gedrückte Karte gibt bis zum Ende der Serie +1 Grundwert | Grundwert (Zähler der Serie) |
| **Allrounder** (M8) | Je verschiedener Spielart in einer Serie +3 Grundwert | Grundwert (Zähler der Serie) |
| **Stammfarbe** (M8) | Dreimal hintereinander dieselbe Trumpffarbe: ×1,5 | Spielwert ×1,5 |
| **Wechselbalg** (M8) | +1 Multiplikator, wenn du eine andere Spielart ansagst als im Spiel davor | Multiplikator |
| **Bubenparade** (M8) | Jeder Unter, den du ausspielst: +1 Grundwert | Grundwert |
| **Luschenkönig** (M8) | Gewinnt eine Sieben einen Stich: +1 Multiplikator | Multiplikator |
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
| Mit Vieren (M5) | 29,3 / 6,1 / 0,7 | – | mit oder ohne 4 und mehr: +1 Multiplikator; Bedingung `MatadorsAtLeast` |
| Siebenschläfer (M5) | – | 28,4 / 8,6 / 2,4 | gewonnenes Null mit unterlaufener Sieben: Grundwert +5; Bedingung `SevenDucked` |
| Saubere Weste (M5) | – | 27,0 / 7,2 / 1,7 | gewonnenes Null, nie bedient: +2 Multiplikator; Bedingung `NeverFollowedSuit`. Der Bot erreicht das praktisch nie, für Menschen gedacht (Fehlfarben drücken) |
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
- **Wechselbalg** (`changeling`): +1 Multiplikator, wenn das angesagte Spiel eine andere Spielart hat als das Spiel davor, über Seriengrenzen hinweg, gewonnen wie verloren. Das erste Spiel mit ihm hat kein Spiel davor.
- **Bubenparade** (`jack_parade`): „ausspielen“ im Sinn von „auf den Tisch legen“, nicht nur „einen Stich eröffnen“ (Glossar: `lead`): +1 Grundwert für jeden Unter, den der Alleinspieler spielt, gewonnen wie verloren, außer im Ramsch. Gewählt, weil die Regel so am Blatt hängt und nicht an einer Spielweise, die Unter zum Anspielen statt zum Stechen verbraucht. Karten, die beim Schenken nicht mehr gespielt werden, zählen nicht. Unter der Gegner zählen nicht.
- **Luschenkönig** (`blank_king`, Bedingung `TrickWinnerMatches`): +1 Multiplikator für jeden Stich, den eine Sieben des Alleinspielers gewinnt (eine Trumpf-Sieben, die sticht, oder eine Sieben, die keiner bedient oder sticht). Sieben der Gegner zählen nicht, der Text meint den Spieler. Nicht im Null (dort verliert sein Stich) und nicht im Ramsch.
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

Einzeln gemessen wie Welle 2a (`skat_sim --runs 500 --no-shop --bot pimc:12:3 --relic <id>`, Seeds 42 bis 44; Spickzettel auf den Zielwerten vor PR #47, ohne Relikt 40,6 / 11,9 / 3,5 %; Drei drücken auf den neuen), Bericht `docs/balance/2026-11-02-relikte-welle2.md`. Seltenheit und Preise sind Platzhalter für `balance`:

| Hausregel | Abschnitt 2 / 3 / 4 | Δ Abschnitt 3 | Seltenheit, Preis | Anmerkung |
|---|---|---|---|---|
| Drei drücken (1×/Serie) | 46,3 / 16,8 / 5,5 | +7,9 | Rare, 9, gesperrt | gemessen auf den neuen Zielwerten (PR #47, ohne Relikt 34,1 / 8,9 / 2,7); in jedem Spiel waren es +18,0 (alte Zielwerte), die Variante „kleinste Karte des Gegners“ +26,4. Eine Karte wechselt die Seite |
| Spickzettel | 40,8 / 12,3 / 3,5 | +0,3 | Common, 4 | nur Handspiele (beim Bot 6 % der Spiele); für Menschen |

**Hausregeln mit eigener Ausgabe (ADR 0015, ausgelegt von Claude am 02.11.2026).** Hausregeln sind Regeln des Tisches und gelten für jede Ausgabe (7.1, Regel 7): Alte Dame, Kleinvieh und Dicke Sau zählen die Augen jeder Karte, wer sie auch hält. Skatpfleger zählt die Skatkarten in der Ausgabe des Spielers. **Drei drücken**: Die Karte, die vom linken Gegner kommt, ist in der Hand des Spielers seine Ausgabe (mit seinen Modifikationen); die Karte, die zurückgeht, ist beim Gegner wieder dessen Ausgabe. „Die wenigsten Augen“ (`rules::returned_discard`) misst an den drei Karten, wie der Spieler sie hält, also in seiner Ausgabe.



### 8.2 Stammtisch-Figuren (passive Relikte mit Persönlichkeit)

**Seit 02.11.2026 (8.5)** sind Die Oma, Der Opa, Der Schiedsrichter, Der Angeber und Der Stammgast entfernt, neu sind Deckel anschreiben und Spelunkenkenner.

| Figur | Effekt |
|---|---|
| **Der Kiebitz** | Zeigt dir vor der Ansage eine zufällige Karte eines Gegners |
| **Der Wirt** | Tresen-Preise −20 % |
| **Die Oma** | Einmal pro Serie darfst du einen gedrückten Buben zurücknehmen |
| **Der Schiedsrichter** | Gegner dürfen nicht schummeln (hebt Boss-Mechaniken teilweise auf) |
| **Der Angeber** | +1 Multiplikator bei Ouvert, aber du musst immer Ouvert spielen |
| **Der Zähler** | Zeigt dir laufend die Augen, die noch im Spiel sind |
| **Der Stammgast** | Zählt die Serien, die du mit ihm gewinnst. Ab der vierten zählt jedes gewonnene Spiel doppelt |
| **Der Schneider** | Nimmt großzügig Maß: Gewinnst du mit 80 bis 89 Augen, sind die Gegner Schneider |

Umgesetzt in M4 (`house_rules.ron`, `kind: Regular`, Preise 6 bis 8 als Platzhalter für `balance`). Auslegungen, entschieden von Claude am 01.11.2026:

- **Kiebitz** (`kibitzer`, Effekt `PeekDefenderCards(1)`): eine zufällige Karte eines Gegners und wer sie hält, gezogen mit dem Zufall des Spiels (das Geben bleibt gleich). Sie steht in der Sicht unter `revealed` und kommt als Ereignis `CardPeeked`, sonst nichts. Seit ADR 0015 zeigt er die Karte in der Ausgabe des Gegners (beim Boss mit dessen Modifikationen); ebenso der Zaungast (9.5).
- **Wirt** (`landlord`, `ShopDiscount(20)`): gilt für Relikte, Verbrauchsgegenstände und die Neuauslage, gerundet, mindestens 1 Groschen. Verkauft wird weiter zum halben vollen Preis, also kein Gewinn durch Kaufen und Verkaufen.
- **Oma** (`granny`, `SeriesSkatSwap`): Ein gedrückter Bube lässt sich im echten Skat nicht sinnvoll zurücknehmen, weil der Spieler beim Drücken alles sieht. Umgesetzt als: Einmal pro Serie darf der Spieler in einem Handspiel vor seiner ersten Karte einen Buben aus dem Skat nehmen und dafür eine Karte drücken; das Spiel bleibt Hand. Ob ein Bube im Skat liegt, verrät die Oma (`can_take_from_skat`), seit M5 auch welcher (`skat_swap_cards`). Hebel: Hand plus Trumpf. Nur Alleinspieler.
- **Opa** (`grandpa`, M5, `SeriesSkatSwap` für Daus): wie die Oma, aber ein Daus aus dem Skat. Oma und Opa zusammen geben zwei Täusche pro Serie, je einen (4.4).
- **Schiedsrichter** (`referee`, `NoCheating`): schaltet die Boss-Regeln ab, die als Schummeln markiert sind: die Zwillinge (bis ADR 0017 auch der Revolutionär).
- **Angeber** (`show_off`, `RequireOuvert` plus eine Stufe bei Ouvert): Für den Alleinspieler hieße „immer Ouvert“ immer Schwarz angesagt, das wäre Selbstmord. Deshalb nur im Pool des Nullspielers (Null Ouvert und Null Ouvert Hand).
- **Stammgast** (`regular_guest`, Zähler `regular` plus `MultiplyGameValue(2)` ab 4): Hebel Spielwert. Der Multiplikator fürs Spätspiel, wer ihn früh kauft, wartet zwei Abende. Nur gewonnene Spiele zählen doppelt.
- **Schneider** (`tailor`, `SchneiderFromDeclarerPoints(80, 89)`): Hebel Schneider, ein Wortspiel mit dem Handwerk. Nur Alleinspieler. Er kam statt des zuerst geplanten Opa, den der Kern in M4 nicht neben der Oma tragen konnte; seit M5 gibt es beide.
- **Zähler** (`card_counter`, `ShowCount`): zeigt Augen beider Seiten aus den Stichen (plus den Skat, sobald der Spieler ihn kennt) und die Summe der Karten, die der Spieler noch nicht gesehen hat. Der Talon zählt dazu, weil niemand weiß, welche Karten dort liegen; so verrät der Zähler nie mehr als ein perfekter Mitzähler weiß. Seit ADR 0015 gibt es keinen Talon; jede ungesehene Karte liegt bei einem der beiden Gegner. Hat der Boss eine Ausgabe mit Augen-Modifikationen, hängt die Summe davon ab, wer die Karte hält: Der Zähler zeigt dann den kleinsten und den größten möglichen Wert („noch 34 bis 38“), damit er nicht mehr verrät als ein Mitzähler weiß.

### 8.3 Verbrauchsgegenstände

Einmal nutzbare Karten, maximal zwei auf der Hand.

**Seit 02.11.2026 (8.5)** sind Ein Bier, Neues Blatt, Magenbitter, Reue, Augen +5 und Trumpf +4 entfernt; Kräuter und Doppelkorn wirken anders, Kreide heißt Blue Curaçao, neu sind Konterbier, Schwenker und Wegbier. Was hier über die entfernten steht, ist Geschichte.

**Seit dem 03.11.2026 (8.7)** gibt es drei Rettungs-Schnäpse für eine Kneipe im Minus: Klarer, Absacker und Eierlikör.

**Seit den Zinken am Tresen (02.11.2026, 8.6)** sind auch Frisches Deck (jetzt der Tresen-Dienst „Karte reinigen“), Stempelkissen, Augen +3, Trumpf +2 und Rang +1 keine Gegenstände mehr; es bleiben nur die Schnäpse.

- **Frisches Deck** (entfernt am 02.11.2026, jetzt „Karte reinigen“, 8.6): Entferne eine Karte dauerhaft. Seit ADR 0015 nicht mehr möglich; ab M9 „eine Karte deiner Ausgabe ist wieder wie neu, alle Modifikationen fallen ab“ (7.3).
- **Stempelkissen** (entfernt am 02.11.2026, 8.6): Verpasse einer Karte eine zufällige Modifikation (seit ADR 0015: einer Karte deiner Ausgabe; die Wahl hat immer 32 Karten). Seit Effekte v2 nach den Regeln aus 7.2: dieselbe steigt, eine andere ersetzt, verfluchte Karten gehen nicht.
- **Augen +3, Augen +5, Trumpf +2, Trumpf +4, Rang +1** (Effekte v2, `ApplyCardMod`; alle entfernt, zuletzt am 02.11.2026 mit 8.6): die Modifikation mit diesem Wert auf eine gewählte Karte, Preise 2 bis 4.
- **Ein Bier**: Einmal mehr einmischen, in einem beliebigen Spiel dieser Serie (seit 01.11.2026; vorher +1 Pass).
- **Runde Schnaps**: Beide Gegner spielen das nächste Spiel mit offenem Blatt.
- **Neues Blatt**: Sofort neu mischen und geben, ohne dass es ein Einmischen kostet.

Gewählt am 01.11.2026 mit dem Einmischen: Ein Bier und Kräuter wirken **für die Serie** (ein Vorrat, der erst nach den eigenen Einmischen des Spiels verbraucht wird und mit der Serie verfällt), wie vorher der Pass „in dieser Serie“; so bleibt der Text fast gleich und der Schnaps lässt sich früh trinken. Neues Blatt wirkt **sofort**, ohne ein Einmischen zu verbrauchen. Beide wirken damit fast gleich (ein Blatt mehr pro Kauf); Preise und Seltenheit (Bier 2, Kräuter 2, Neues Blatt 4) sind für `balance` neu zu prüfen.

**Schnäpse** (entschieden über das Cockpit am 01.11.2026, umgesetzt von Claude am 01.11.2026). Fünf Verbrauchsgegenstände mit `category: Schnapps` in `consumables.ron`, damit Tresen und Oberfläche sie als „Schnaps“ beschriften. Sie belegen dieselben zwei Plätze, kommen am Tresen wie die anderen und sind dort auch zu verkaufen. Seltenheit und Preise sind **Platzhalter für `balance`**, nicht gemessen: Kurzer 2 (Common), Doppelkorn 3 (Uncommon), Kräuter 2 (Common), Magenbitter 2 (Common), Blue Curaçao 3 (Uncommon).

| Schnaps | Id | Wirkung | Wann |
|---|---|---|---|
| **Kurzer** | `shot` | Zeigt die beiden Skatkarten. Die Wahl zwischen Skat und Hand bleibt offen. | Nur mit zehn Karten vor „Skat oder Hand“ (`Phase::Declaring`), einmal pro Spiel. Nicht gegen den Kiebitz-Mörder (Information ist aus). |
| **Doppelkorn** | `double_grain` | Das nächste angesagte Spiel zählt doppelt, gewonnen wie verloren: Zeile `Spielwert ×2` mit der Herkunft `Source::Consumable`. | Am Tresen oder vor der Ansage des liegenden Spiels. Verbraucht bei der nächsten Ansage wie „Letzte Runde“ und die Bockrunde; Einmischen verschenkt ihn nicht. Zwei Doppelkorn stapeln (×4). |
| **Kräuter** | `herbal` | Einmal mehr einmischen in dieser Serie (dieselbe Aktion wie „Ein Bier“, `ExtraRedeals(1)`). | Während einer Serie. |
| **Reue** | `regret` | Nimmt das letzte Einmischen zurück: Das Blatt davor liegt wieder auf dem Tisch, genau wie es war (Hände, Skat, Kiebitz, Blue Curaçao), und wird gespielt. Das Einmischen bleibt verbraucht. Neu am 01.11.2026, Common, 2 Groschen (Platzhalter). | Nur mit zehn Karten vor „Skat oder Hand“, gleich nach einem Einmischen (oder „Neues Blatt“) desselben Spiels; nur einmal zurück, nicht zweimal hintereinander (`NoRedealToUndo`). |
| **Magenbitter** | `bitters` | Geht das nächste angesagte Spiel verloren, zählt es einfach statt doppelt: Zeile `LossFactor = 1` (neuer Hebel, nur bei verlorenen Spielen in der Abrechnung). | Wie Doppelkorn. Verbraucht vom nächsten angesagten Spiel, gewonnen wie verloren; gewonnen wirkt er nicht. Ein zweiter, solange einer wartet, wird abgelehnt. |
| **Blue Curaçao** (bis 02.11.2026 Kreide) | `blue_curacao` (bis 02.11.2026 `chalk`) | Malt eine Karte der eigenen Hand bis zum Spielende in eine andere Farbe um. Die Karte ist für dieses Spiel diese Karte: Trumpf, Bedienen und Spitzen richten sich nach der neuen Farbe, Augen und Modifikationen bleiben. | Vor der Ansage (vor „Skat oder Hand“, beim Drücken oder bei der Ansage nach dem Skat). Nie ein Unter (er ist Trumpf durch seinen Rang) und keine Blanko. |

Auslegungen, entschieden von Claude am 01.11.2026:

- **Magenbitter wird vom nächsten angesagten Spiel verbraucht, nicht erst vom nächsten verlorenen.** So ist er eine Entscheidung vor einem riskanten Spiel (Grand Hand, Schneider angesagt) statt einer Versicherung, die man kauft und vergisst. Wer ihn trinkt und gewinnt, hat ihn umsonst getrunken.
- **Blue Curaçao ist vorübergehend** (Entscheidung des Nutzers). Seit ADR 0015 malt sie die Ausgabe des Spielers um, nur für dieses Spiel, und nur, solange er die Karte hält oder gespielt hat: Das Blatt des Runs behält die gedruckte Karte, mit dem nächsten Geben ist sie wieder, was gedruckt ist. Einmischen (auch „Neues Blatt“) beendet das Spiel und damit auch die Blue Curaçao; „Reue“ bringt das alte Blatt samt Blue Curaçao zurück. Im Kern ersetzt `Game::chalk` die Karte überall, wo das Spiel sie führt (Hand, die zwölf Karten des Alleinspielers, die Karten im Spiel, das Blatt), Ereignis `CardChalked`. Doppelte Karten (zwei Grün-Zehnen) sind wie beim veränderbaren Blatt erlaubt: Bei gleicher Stärke behält die früher gespielte den Stich (4.4). Die Spitzen zählen über die umgemalte Karte, weil sie für dieses Spiel diese Karte ist; ein gemalter Trumpf-Daus kann damit eine Spitze mehr bringen.
- **Doppelkorn und Magenbitter** wirken auf die nächste Ansage im Sinne von `after_declaration`, wie „Letzte Runde“: Auch der Ramsch des Ramschers zählt als angesagtes Spiel. Während des Spiels (nach der Ansage) sind sie nicht nutzbar, damit niemand glaubt, sie wirkten noch aufs laufende Spiel.
- **Kurzer** gilt als Information (bis ADR 0017 gegen den Kiebitz-Mörder nicht nutzbar). Die Wertvorschau zählt den gesehenen Skat als bekannt.
- **Bot** (`run/bot.rs`, einfache Regeln, nicht kalibriert): Kurzer direkt vor dem Aufnehmen des Skats (der Bot nutzt das Gesehene nicht), nach dem Skat Blue Curaçao auf die billigste Beikarte in die Trumpffarbe eines Farbspiels, dann Doppelkorn vor einem nicht schwachen und Magenbitter vor einem schwachen Spiel, nie vor einem Null; Kräuter wie Ein Bier. Reue nutzt der Bot nie (Kaufwert 0).
- **Reue** (Name „Reue“ gewählt: kurz und sagt, was er tut). Im Kern hält `ActiveGame::previous` das Spiel vor dem letzten Neugeben vollständig (es wird nicht neu gezogen, also deterministisch und auch nach Speichern und Laden gleich); `ConsumableAction::UndoRedeal` setzt es wieder ein, Ereignis `RedealUndone`. Was das Geben selbst ausgelöst hat (Zähler, Flaggen von `OnDeal`), bleibt; heute hat kein Inhalt so einen Hook. Der Zufallsstrom läuft weiter: Wer nach der Reue noch einmal einmischt, bekommt ein neues Blatt, nicht das zurückgenommene. Kaufwerte in `CONSUMABLE_VALUES` sind Platzhalter.

### 8.4 Stufe Legendär (M8 Welle 3)

Eine vierte Seltenheit für Relikte, die den Run prägen (`Rarity::Legendary`, Anzeige „legendär“). Sie bestimmt nur, wie selten das Relikt am Tresen liegt, nicht den Preis (entschieden am 02.11.2026: Preise nach der gemessenen Wirkung, Staffel aus Welle 2a). Am Tresen weit seltener als Rare: Gewicht `legendary` in `run.ron` als Dezimalzahl neben Common 6, Uncommon 3, Rare 1, Standard 0,3 (ohne Eintrag in `run.ron`, `RarityWeights::percent_weight`). Ein bestimmtes legendäres Relikt liegt damit beim Alleinspieler in etwa 0,6 % der Auslagen. Auch der Stammtisch-Streit und der Alte Bekannte ziehen mit diesem Gewicht. Auf dem Bierdeckel ein goldener Ring mit vier Funken. Preise Platzhalter für `balance`. Entschieden am 02.11.2026:

| Relikt | Effekt | Hebel | Preis |
|---|---|---|---|
| **Skatgott** (`skat_god`) | Jedes gewonnene Spiel mit Schneider: bis zum Run-Ende +1 Multiplikator auf jedes gewonnene Spiel, höchstens +3 | Multiplikator (Zähler `skat_god`) | 12 (bis 03.11.2026 9) |
| **Ehrenvorsitz** (`honorary_chair`) | Du bist immer Vorhand | Stichgewinn (Vorhand) | 6 |

- **Skatgott**: Der Zähler steigt nach jedem gewonnenen Spiel mit Schneider (auch Schneider durch Jungfrau, Schenken, Stichfest oder Schwarz). Die Stufen gelten nur für gewonnene Spiele, jede Spielart (Farbspiel, Grand, Null), nie der Ramsch (`AddLevelsPerCounter` bei der Wertung; der Stufen-Hook steht vor dem Zähler, ein Schneiderspiel zählt seinen eigenen Punkt also nicht). Entschieden am 02.11.2026: „auf alles“ heißt „auf jede Spielart“; die erste Fassung zählte auch verlorene Spiele und brachte dem Bot nur +1,3 bei weniger Punkten je Spiel. Abgestimmter Text ergänzt zu „… +1 Multiplikator auf jedes gewonnene Spiel“. **Seit 03.11.2026 höchstens +3** (Entscheidung von Claude nach `docs/balance/2026-11-03-kleine-runde.md`: allein +12 Punkte „Run gewonnen“, alle anderen Relikte höchstens +4): Der Zähler steigt nur, solange er unter 3 steht (`Not(CounterAtLeast(counter: "skat_god", value: 3))`, reine Daten).
- **Ehrenvorsitz**: Der Spieler spielt in jedem Spiel den ersten Stich aus (`AlwaysForehand`); die Gäste sitzen weiter als Mittel- und Hinterhand. Das Geben bleibt gleich, nur die Vorhand wechselt nicht mehr.

Gemessen (`docs/balance/2026-11-02-welle3.md`, wie Welle 2a): Skatgott +4,7 auf „Abschnitt 3 erreicht“ (Abschnitt 4 +3,7, Punkte je Spiel +7,8), so stark wie die Alte Dame und im Spätspiel stärker; Ehrenvorsitz −0,4 (im Rauschen), ein Relikt für Menschen.

---

### 8.5 Gegenstands-Review (02.11.2026)

**Entschieden vom Nutzer am 02.11.2026, maßgeblich ab hier.** Was 8.1 bis 8.3 über die unten entfernten Gegenstände sagen, ist Geschichte. Alle Preise und Seltenheiten dieses Abschnitts sind **vorläufig**, nach Augenmaß ohne Messung gesetzt (die Simulationen ruhen); `balance` misst, sobald der Simulator wieder läuft.

**Entfernt.** Relikte: Blindflug, Bockrunde, Wechselbalg, Saubere Weste, Glashaus, Der Opa, Die Oma, Auf dem Trockenen, Bubenparade, Kontra, Durchmarsch, Altes Blatt, Ouvert-Bonus, Der Schiedsrichter, Der Stammgast, Doppelter Boden, Schneiderbock, Siebenschläfer, Der Angeber, Kerbholz, Lehrgeld und (Nachentscheidung) **Lockvogel ganz**: Er wird kein Schnaps; mit ihm fallen Aktion `DemandLead`, Effekt `DemandLead`, die Regeln `lead_demands`, `Game::demand_lead`, `rules::legal_leads` und die Sicht `can_demand_lead`. Verbrauchsgegenstände: Ein Bier, Neues Blatt, Magenbitter, Reue, Augen +5, Trumpf +4. Ruhend in der Hook-API bleiben die Wirkungen, die Bosse oder alte Spielstände noch brauchen oder die keinen Ballast tragen (`EnableKontra` für den Kontra-Hans, `SeriesSkatSwap`, `NoCheating`, `RequireOuvert`, die Aktionen `ExtraRedeals`, `UndoRedeal`, `NextGame(SingleLoss)`); die Definitionen der entfernten Relikte und Gegenstände stehen nur noch für Tests in `core/src/run/legacy_*.ron`. Zum Lesen alter Spielstände bleiben `rules::LeadSuit` und das Spielereignis `LeadDemanded`; die Fehlercodes `MustLeadDemanded`, `InvalidLeadDemand` und `NoLeadDemandLeft` bleiben stehen (kommen nicht mehr vor).

**Spielstände.** Keine neue Version. Beim Laden (`Session::load` in der Web-App) räumt `Run::drop_unknown_content` auf: entfernte Relikte (besessen, für den Stadtteil, gesperrt), Verbrauchsgegenstände (besessen, auf das nächste Spiel wartend), Tresen-Angebote (werden „verkauft“) und Optionen offener Ereignisse, die sie nennen, fallen weg; der Run geht ohne sie weiter. Umbenannte Ids (`RENAMED_CONSUMABLES`: `chalk` → `blue_curacao`) werden vorher umbenannt. Ein laufendes Spiel behält die Regeln, mit denen es gegeben wurde. Freischaltungen gelten weiter (ihre Ids bleiben). Ein laufender Run kennt die Sperrliste seines Starts; die neuen Relikte liegen darin nicht und können dort schon am Tresen liegen.

**Geändert.**

- **Drei drücken**: nur noch im **ersten Spiel jeder Kneipe** (`Run::discard_three_ready` prüft `games_played == 0`). Wird das erste Spiel aus der Hand gespielt, verfällt es für diese Kneipe. Text: „Im ersten Spiel jeder Kneipe: …“, Anzeige „1. Spiel“. Preis 9 unverändert.
- **Spickzettel**: zeigt nach der Ansage eines Handspiels **beide** Skatkarten (`RevealSkatCard(2)`); bis ADR 0017 schaltete der Kiebitz-Mörder ihn ab. Preis 4 → 5.
- **Kräuter**: wirkt wie das frühere Neue Blatt (sofort neu geben, ohne ein Einmischen zu verbrauchen, `ConsumableAction::Redeal`). Preis 2 → 3, Common.
- **Doppelkorn**: Trinken gibt **sofort neu** (`ConsumableAction::RedealNextGame`), kostenlos wie Kräuter, und das neue Spiel zählt doppelt; weitere Einmischen behalten die Wirkung bis zur nächsten Ansage. Nur mit zehn Karten vor „Skat oder Hand“, nicht am Tresen (dort gäbe es nichts neu zu geben). Auslegung von Claude: „löst ein Einmischen aus“ heißt neu geben ohne ein Einmischen zu verbrauchen; sonst wäre er ohne übriges Einmischen nicht trinkbar. Preis 3, Uncommon.
- **Blue Curaçao** (bis 02.11.2026 „Kreide“, Id `blue_curacao`, alte Id `chalk` wird beim Laden umbenannt): Wirkung unverändert.
- **Alte Oberin** (ADR 0014): Name und Text stehen jetzt so in `house_rules.ron`, die Überschreibung in der Web-App entfällt.
- **„Multiplikator“ statt „Stufe“/„Gewinnstufe“** in allen Spieltexten (Content, Übersetzung, Abrechnung, Regelkarte, Glossar). Im Code bleiben `levels`, `AddLevels`, `Lever::Levels`. „Sticht eine Stufe höher“ (Rang) und die Risikostufe bleiben.
- **Anzeige-Gruppe** als Daten: `group` an jedem Relikt (`Value`, `Multiplier`, `Control`, `Tour`, Standard `Value`), in Sicht und Katalog. Die Web-App liest sie, die Form des Bierdeckels folgt noch der Art (Hausregel, Figur); die Form nach Gruppe kommt mit dem neuen Bierdeckel-Entwurf.
- **Kartenwaren** (Augen +3, Trumpf +2, Rang +1, Stempelkissen) blieben zunächst, wie sie sind. Die geplanten Kartenwaren mit Schnapsnamen (Goldwasser, Trumpfbrand, Absacker, Anstich, Pfandbon, Totenschein, Narrenkappe) sind **nicht** gebaut: Kartenverbesserungen bekommen eine eigene Tresen-Mechanik (Nachentscheidung des Nutzers). **Seit 8.6 überholt**: Die Kartenwaren sind entfernt, die Mechanik heißt Zinken.

**Neue Schnäpse** (`consumables.ron`, alle `category: Schnapps`):

| Schnaps | Id | Wirkung | Wann | Seltenheit, Preis (vorläufig) |
|---|---|---|---|---|
| **Konterbier** | `counter_beer` | +1 Leben, höchstens bis zum Maximum (`GainLives(1)`) | außerhalb eines Spiels (Routenwahl, Tresen, Ereignis), nur mit fehlendem Leben (sonst `LivesFull`) | Uncommon, 4 |
| **Schwenker** | `snifter` | Die Stationen der drei Wege werden neu ausgelost (`RerollRoutes`); eine Spelunke bleibt, wo sie ist, ein Tresen bleibt erreichbar | bei der Routenwahl | Common, 2 |
| **Wegbier** | `road_beer` | Nächste Kneipe: ein Einmischen mehr (für die ganze Kneipe) und Zielwert −3 (`NextPub(redeals: 1, target: -3)`, nie unter 1) | zwischen zwei Kneipen (Routenwahl, Tresen, Ereignis) | Common, 3 |

**Neue Relikte** (`house_rules.ron`, Gruppe Tour):

| Relikt | Id | Wirkung (Daten) | Seltenheit, Preis (vorläufig) | Freischaltung |
|---|---|---|---|---|
| **Feierabend** | `closing_time` | Kneipe im ersten Spiel geschafft: +4 Groschen (`OnSeriesWon` mit `GamesLeftAtLeast(2)`, `GainCoins(4)`) | Common, 4 (bis 03.11.2026 5) | frei |
| **Monatskarte** | `season_ticket` | Jede Bahn: +1 Groschen (neuer Auslöser `OnRouteChosen`, Bedingung `TransportIs(Train)`) | Common, 4 | Neues Viertel |
| **Stammtischkasse** | `kitty` | Von jeder Belohnung einer Kneipe 2 Groschen in die Kasse (`StashCoins`, Zähler `kitty`, nach der Auszahlung, höchstens was da ist); beim nächsten Tresen alles plus die Hälfte, abgerundet (neuer Auslöser `OnShopOpened`, `PayOutCounter(percent: 150)`) | Uncommon, 5 | Dickes Ding |
| **Deckel anschreiben** (Figur) | `slate` | Einmal pro Stadtteil darf der Tresen bis 5 Groschen ins Minus verkaufen (`Credit(5)`); spätere Einnahmen zahlen zuerst die Schulden | Uncommon, 5 | Omas Liebling |
| **Schutzengel** | `guardian_angel` | Einmal pro Stadtteil kostet die erste verlorene Kneipe kein Leben (`SpareLife`, Ereignis `LifeSpared`) | Rare, 8 (am 03.11.2026 kurz 6, nach der großen Runde wieder 8) | Ohne Einmischen |
| **Spelunkenkenner** (Figur) | `dive_expert` | In einer Spelunke zwei Mali zur Wahl (`ChooseMalus(2)`, Aktion `ChooseMalus`) | Uncommon, 4 | Lehrjahre |

Auslegungen von Claude:

- **Deckel anschreiben**: „Einmal pro Stadtteil“ heißt: der erste Tresen-Besuch eines Stadtteils, in dem der Spieler ins Minus kauft, öffnet den Deckel; in **diesem** Besuch darf er weiter bis −5 kaufen (auch Neuauslage), spätere Besuche desselben Stadtteils nicht mehr. Der Stadtteil ist der, in den der Weg führt. Im Kern bleiben die Groschen bei 0, die Schulden stehen daneben (`Run::debt`, Sicht `debt`, `credit_left`); jede Einnahme (Auszahlung, Verkauf, Groschen aus Relikten, Ereignissen, Flüchen, Gutscheinen, Kasse, Wette) zahlt zuerst die Schulden (Ereignis `DebtChanged`). Zinsen rechnen mit den Groschen, nicht mit den Schulden. Ereignisse kosten nie auf Pump.
- **Schutzengel**: gilt seit ADR 0017 nicht für die Boss-Kneipe (die verloren den Run beendet); bis dahin wurde sie mit ihm ohne Lebensverlust wiederholt.
- **Spelunkenkenner**: Die zwei Mali werden mit der Route gezogen und angezeigt; gilt der erste, bis der Spieler einen anderen wählt (`RunAction::ChooseMalus { option, malus }` bei der Routenwahl).
- **Stammtischkasse**: Die Kasse zahlt beim nächsten Tresen, egal ob dazwischen eine Kneipe lag; ohne Tresen wächst sie weiter.

**Neue Begegnungen** (`encounters.ron`, Kategorie Begegnung, Werte vorläufig):

| Begegnung | Id | Wahl |
|---|---|---|
| **Der Wirt spendiert** | `landlord_treat` | einer von drei gezogenen Schnäpsen gratis in einen freien Platz, oder 5 Groschen |
| **Falschspieler am Nebentisch** | `card_sharp` (ersetzt den alten Falschspieler, Id bleibt) | für 2 Groschen eine starke Modifikation (eine von Rang +1, Trumpf +4, Augen +6, beim Aufgehen gezogen und genannt) auf eine gewählte Karte; mit 50 % landet stattdessen ein zufälliger Fluch, der wie jeder Fluch 3 Groschen zahlt |
| **Verlorenes Portemonnaie** | `lost_wallet` | behalten: +8 Groschen und Durchsichtig auf einer zufälligen Karte (der Fluch zahlt zusätzlich die üblichen 3); abgeben: +1 Leben |
| **Der Skatlehrer** | `skat_teacher` | Rang +1 auf eine gewählte Karte gratis; im ersten Spiel der nächsten Kneipe ein Einmischen weniger |
| **Die Wette** | `the_bet` | 3 oder 6 Groschen auf das nächste angesagte Spiel: gewonnen mit Schneider zahlt doppelt, sonst weg; der Ramsch zählt nicht, er wartet |
| **Alte Karten im Schrank** | `old_cards` | ein Gutschein (Untot oder Wandelkarte, beim Aufgehen gezogen) auf eine gewählte Karte, ein Fluch (gezogen und genannt) auf eine zufällige andere |
| **Kiebitz am Fenster** | `lookout` | die genauen Stationen der nächsten zwei Routenwahlen sehen, oder ein Einmischen mehr in der nächsten Kneipe |

Auslegungen von Claude: „Nächste Kneipe, ein Einmischen weniger“ (Skatlehrer) heißt **im ersten Spiel** der Kneipe eins weniger (`Series::redeals_withheld`), weil Einmischen je Spiel zählt und drei weniger zu hart wäre; „ein Einmischen mehr“ (Wegbier, Kiebitz) geht wie früher Ein Bier in den Vorrat der Kneipe. Der Plan für die nächste Kneipe (`NextPubPlan`) summiert sich und wird beim Start der Kneipe angewandt (Ereignis `NextPubApplied`); die Routenwahl zeigt ihn und den geänderten Zielwert. Die Wette wird bei der Wertung des nächsten nicht-Ramsch-Spiels entschieden; Null kann nie Schneider sein und verliert sie. Ereignisse mit Fluch sagen „Fluch“ im Text (beim Laden geprüft). Der Geburtstag am Stammtisch kommt später.

**Bot** (`run/bot.rs`, Regeln, nicht kalibriert): Kaufwerte Kräuter 1, Doppelkorn 1 (seit 03.11.2026, vorher 2: rund die Hälfte blieb ungetrunken, `docs/balance/2026-11-03-kleine-runde.md`), Konterbier 2, Wegbier 2, Schwenker 0 (nie gekauft); Feierabend 3, Monatskarte 2, Kasse 3, Deckel 2 (er kauft nie auf Pump), Schutzengel 7, Spelunkenkenner 1. Bei der Routenwahl trinkt er Konterbier mit fehlendem Leben, Wegbier sofort und den Schwenker, wenn kein Weg Upgrade (bei verfluchter Karte) oder Rast (bei höchstens zwei Leben) bietet; als Spelunkenkenner wählt er Schnapsverbot vor Klebrigen Karten, Kein Einmischen, Kein Skat. Doppelkorn nutzt er wie Kräuter, um ein schwaches Blatt ohne übriges Einmischen loszuwerden. Begegnungen: Schnaps vom Wirt nach Kaufwert (sonst die Groschen), Portemonnaie behalten (Wert 4) oder abgeben bei fehlendem Leben, Skatlehrer auf ein Daus, Kiebitz für das Einmischen; Falschspieler, Wette und Alte Karten lehnt er ab.

Tests: `core/src/run/items_tests.rs` (je Gegenstand einer, dazu Entfernung und Laden alter Spielstände), `wasm` lädt die Fixtures mit entfernten Relikten und spielt weiter.

### 8.6 Zinken am Tresen (02.11.2026)

**Entschieden vom Nutzer am 02.11.2026, maßgeblich ab hier.** Kartenmodifikationen sind keine Verbrauchsgegenstände mehr. Der Tresen verkauft sie unter dem Namen **Zinken** (Verb in der Oberfläche: „Karte zinken“, eine einzelne heißt Zinke). Alle Zahlen dieses Abschnitts sind **vorläufig**, nach Augenmaß ohne Messung (die Simulationen ruhen); `balance` misst, sobald der Simulator wieder läuft. Daten: `zinken` in `run.ron` (`ZinkenConfig`), `price` und `shop_value` in `card_mods.ron`; Code `core/src/run/zinken.rs`, Tests `core/src/run/zinken_tests.rs`.

**Namen.** Aufdruck (`points`), Winkel (`rank`), Siegel (`trump_bonus`), Eselsohr (`first_trick`), Kerbe (`last_trick`), Pfandmarke (`voucher_coins`), Totenmarke (`voucher_undead`), Narrenmarke (`voucher_wild`). Flüche und Spezialkarten behalten ihre Namen. Infokarten zeigen immer die Wirkung neben dem Namen: Jeder Text beginnt mit ihr („Eselsohr“ – „Erster Stich: Gewinnt diese Karte den ersten Stich, zählen seine Augen ×2.“).

**Die Auslage.** Jeder Tresen legt **drei Angebote** aus, jedes eine bestimmte Karte der Ausgabe des Spielers mit einer bestimmten Zinke und ihrem Wert, vor dem Kauf sichtbar (die Oberfläche zeigt die Karte, wie sie danach aussieht). Gezogen mit einem eigenen Zufallsstrom des Runs (`zinken_rng`), damit die übrigen Tresen-Angebote, Wege und Spiele gleich bleiben.

- **Nie ein Fluch**, nie eine Zinke, die nicht auf die Karte passt (`only_on`) oder die die Regeln aus 7.2 ablehnen (verfluchte Karte, Winkel schon bei 2). Eine Karte höchstens einmal je Auslage. Eine Spezialkarte nur als seltenes Angebot (unten), nie eine Toten- oder Narrenmarke, deren Spezialkarte die Ausgabe nicht mehr bekommen darf (7.2).
- **Selten eine Spezialkarte direkt** (seit 03.11.2026, Entscheidung des Nutzers „A + B“, Auslegung von Claude): Jedes Angebot ist mit 5 % (`special_percent`, etwa eines von zwanzig) „wird Untot“ oder „wird Wandelkarte“ auf einer bestimmten Karte, solange die Ausgabe diese Spezialkarte noch bekommen darf (`Run::special_offers`). Höchstens eins je Auslage und Spezialkarte, die Marke dafür eingeschlossen, damit ein Kauf kein anderes Angebot derselben Auslage wertlos macht. Nie auf einer Karte, die schon eine Spezialkarte trägt; die Zinke der Karte bleibt daneben (7.2, Grundregel 1), die Kachel sagt „Aufdruck bleibt“. Gezogen mit demselben Zufallsstrom und denselben Kartengewichten (ohne Modifikation 3, sonst 1), gewürfelt nur, wenn eine solche Spezialkarte möglich ist. Preis 6 Groschen (`special_price`), auf Unter und Daus 7, dann wie jedes Angebot über den Wirt und den Preis des Charakters (`Run::card_offer_cost`: `shop_price`, `zinken_price`). Die Kachel trägt ein Band „Selten“ und zeigt die Karte mit ihrem Spezialbild.
- **Karten ohne Modifikation werden bevorzugt**: Gewicht 3 gegen 1 für Karten, die schon etwas tragen (`bare_weight`, `modded_weight`).
- **Gewicht der Zinke**: Seltenheit wie bei den Relikten (gewöhnlich 6, ungewöhnlich 3, selten 1) mal das Gewicht des Charakters für die Spielart (`focus_weights`), **Gutscheine** (Pfandmarke, Totenmarke, Narrenmarke) nur mit der Hälfte (`voucher_percent: 50`).
- **Werte** in der Auslage (`shop_value`, sonst der erste aus `values`): Aufdruck +3, Siegel +2, Winkel +1, Eselsohr ×2, Kerbe +1, Pfandmarke +3.
- **Preis** nach Zinke und Karte: der Preis der Zinke, auf einem Unter oder Daus 1 Groschen mehr (`premium_ranks`, `premium`). Kauf: `RunAction::BuyCardOffer(slot)`, Ereignis `CardOfferBought`, danach `DeckChanged`. Dieselbe Zinke auf der Karte steigt, eine andere wird ersetzt; die Oberfläche fragt vorher wie bisher (Ersetzen-Bestätigung), die Sicht sagt es vorab (`CardOfferView::replaces`, `raises`).
- **Neu auslosen**: die drei Angebote neu, 1 Groschen, jedes weitere Mal am selben Tresen 1 mehr (`reroll_cost`, `reroll_step`; `RunAction::RerollCardOffers`, Ereignisse `CardOffersRerolled`, `CardOffersDrawn`). Die Neuauslage (Relikte und Gegenstände) lässt die Auslage, wie sie ist, und umgekehrt.

**Der Kartenmaler.** Einmal je Tresen: bezahlen, eine Karte wählen (`RunAction::HirePainter(card)`), dann eine von **zwei** gezogenen Zinken nehmen, die auf diese Karte passen (`RunAction::ChoosePaint(i)`). Gleiche Regeln wie die Auslage (nie ein Fluch, Gewichte, Werte), zwei verschiedene. Bis zur Wahl nimmt der Run nichts anderes an (Fehler `PainterPending`); die Wahl übersteht Speichern und Laden. Teurer als ein Angebot der Auslage, weil der Spieler die Karte wählt. Ereignisse `PainterHired`, `PainterChosen`. Kann die Karte keine Zinke nehmen, wird nichts bezahlt (Fehler der Regeln, etwa `CardCursed`, sonst `NoModAvailable`); die Kartenwahl blendet solche Karten mit dem Grund ab.

**Karte reinigen.** Der Tresen-Dienst, der Frisches Deck ersetzt: Einmal je Tresen fallen alle Modifikationen einer gewählten Karte ab, auch Flüche, Spezialkarten und Abgestuft (`RunAction::CleanseCard(card)`, Ereignis `CardCleansed`, danach `DeckChange::Cleansed`; eine Karte ohne Modifikation ist kein Ziel). Auslegung von Claude: einmal je Tresen wie der Kartenmaler, damit ein Tresen nicht alle Flüche auf einmal tilgt.

**Die Werkstatt** (Station Upgrade, 5.0.1) bietet seit den Zinken den **Kartenmaler gratis** (`EncounterChoice::Painter`: Karte wählen, dann eine von zwei Zinken) und **Karte reinigen** gratis; die bisherige „eine gezogene Modifikation auf eine gewählte Karte“ entfällt. Erst die Wahl des Kartenmalers schließt die Begegnung ab.

**Der Wirt** („Alles am Tresen ist 20 % billiger …“) gilt für alles am Tresen: Auslage, Neu auslosen, Kartenmaler und Karte reinigen wie für Relikte, Gegenstände und die Neuauslage (Auslegung von Claude, Text ergänzt). Gerundet wie bisher, mindestens 1 Groschen. **Deckel anschreiben** gilt ebenso. **Bruderherz** (9.6, Regel des Charakters): Nach dem Wirt gilt sein Preis für Zinken auf bekannten Karten (`Run::zinken_price`, halb auf Rot) für jedes Angebot der Auslage und den Kartenmaler (`Run::card_offer_cost`, `Run::painter_cost`; die Sicht nennt den Preis je Karte in `ServiceView::target_prices`). Karte reinigen läuft über `Run::cleanse`: Seine Flüche bleiben (`CurseStays`), bezahlt wird dann nichts.

**Vorläufige Preise** (Groschen, vor dem Wirt):

| Was | Preis |
|---|---|
| Aufdruck +3, Pfandmarke +3 | 2 |
| Winkel +1, Siegel +2, Eselsohr ×2, Kerbe +1 | 3 |
| Totenmarke, Narrenmarke | 4 |
| Untot, Wandelkarte direkt (selten, seit 03.11.2026) | 6 |
| Zuschlag auf Unter und Daus | +1 |
| Neu auslosen | 1, dann 2, 3, … am selben Tresen |
| Kartenmaler | 6 |
| Karte reinigen | 3 |

Begründung: Die Auslage legt die Karte fest, ist also weniger wert als die alten Kartenwaren mit freier Wahl (Augen +3 für 2, Trumpf +2 und Rang +1 für 3); der Kartenmaler mit freier Karte und Wahl aus zwei liegt über dem teuersten Angebot (5); Karte reinigen kostet, was Frisches Deck kostete.

**Entfernt** (Verbrauchsgegenstände): Frisches Deck (`fresh_deck`), Stempelkissen (`ink_pad`), Augen +3 (`points_3`), Trumpf +2 (`trump_2`), Rang +1 (`rank_1`). Die Aktionen `CleanseCard`, `AddCardMod`, `ApplyCardMod` ruhen in der Hook-API; die Definitionen liegen für Tests in `core/src/run/legacy_consumables.ron`.

**Spielstände.** Keine neue Version (wie 8.5): Beim Laden räumt `Run::drop_unknown_content` die entfernten Gegenstände ab (besessen, Tresen-Angebote), dazu Angebote der Auslage und Optionen des Kartenmalers, deren Zinke die Daten nicht mehr kennen. Ein Spielstand von vorher, der am Tresen steht, hat eine leere Auslage; Neu auslosen füllt sie. Der Zinken-Zufallsstrom alter Spielstände kommt aus einer festen Zahl.

**Bot** (`run/bot.rs`, einfache Regeln, nicht kalibriert, damit der Simulator weiterläuft): Wartet der Kartenmaler, nimmt der Bot die wertvollste Zinke (`ZINKEN_VALUES`: Siegel 3, Winkel 3, Aufdruck 2, Eselsohr 2, Kerbe 1, Pfandmarke 1, Toten- und Narrenmarke 0; seit 03.11.2026 für die seltenen Angebote der Auslage Untot 2, Wandelkarte 1). Am Tresen: erst Karte reinigen auf eine verfluchte Karte, dann Relikte und Schnäpse wie bisher, dann das wertvollste Angebot der Auslage auf einer Karte ohne andere Modifikation (nie ersetzen, Steigern ja), dann den Kartenmaler für einen Daus ohne Modifikation, wenn danach noch 4 Groschen bleiben (`PAINTER_RESERVE`). Nie auf Pump. In der Werkstatt reinigt er eine verfluchte Karte, sonst nimmt er den Kartenmaler für einen Daus.

**Oberfläche.** Der Tresen bekommt eine Reihe „Auslage“ mit drei Angeboten: die Karte mit der neuen Zinke in der Rahmensprache gezeichnet, Name, Wirkung und Preis, Kaufen mit Ersetzen-Bestätigung; daneben „Neu auslosen“ mit Preis, der Kartenmaler (bezahlen, Karte wählen, eine von zwei wählen) und Karte reinigen.

Zu messen, sobald der Simulator wieder läuft: Käufe je Tresen (Auslage, Kartenmaler, Reinigen), Groschen je Run, Wirkung auf „Stadtteil 2 erreicht“ gegenüber den alten Kartenwaren, Anteil der Auslagen ohne passendes Angebot für den Bot.

### 8.7 Rettung einer Kneipe (03.11.2026)

**Entschieden vom Nutzer am 03.11.2026 über das Cockpit** (Varianten A, C und D): Nach zwei verlorenen Spielen steht das Konto oft so tief im Minus, dass das dritte Spiel die Kneipe nicht mehr schaffen kann. Das frustriert, weil man ein Spiel zu Ende spielt, das nichts mehr ändert. Dagegen: aufgeben können, sehen, was noch nötig ist, und drei Rettungs-Schnäpse.

**A, „Kneipe verlassen“** (`RunAction::GiveUpPub`, Ereignis `PubGivenUp`, Sicht `RunView::can_give_up`, `Run::can_give_up`):

- Geht **zwischen zwei Spielen** einer Kneipe (`Run::between_games`: das gegebene Spiel ist unberührt, zehn Karten, Skat nicht aufgenommen, nichts angesagt, keine Karte gespielt; ein Ramsch vor seiner ersten Karte) und **ab dem zweiten Spiel** (`games_played >= 1`). Entschieden von Claude: nicht schon vor dem ersten Spiel, weil dann nichts verloren ist, was eine Aufgabe rechtfertigt, und ein Fehlgriff ein Leben kostet.
- Zählt als **verlorene Kneipe**: Auf `PubGivenUp` folgt alles, was eine verfehlte Kneipe bringt (`PubLost`, ein Leben weg, mit dem letzten Leben `RunLost`, sonst die Routenwahl). Keine Auszahlung, `OnSeriesWon` feuert nicht. Eine Spelunke wie eine normale Kneipe.
- **Boss-Kneipe**: Aufgeben beendet den Run (ADR 0017, `RunLost` mit `boss`). Die Oberfläche fragt deutlich nach: „Aufgeben beendet den Run.“
- **Schutzengel**: verschont auch eine aufgegebene Kneipe (entschieden von Claude, wie vom Nutzer vorgeschlagen): Sie ist eine verlorene Kneipe, und der Engel fragt nicht, wie sie verloren ging. Wie sonst einmal pro Stadtteil und nie in der Boss-Kneipe.
- **Bot**: gibt nie auf. Eine verlorene Kneipe kostet ihn dasselbe Leben, ob er das letzte Spiel spielt oder nicht, und gespielt kann ein Wunder noch helfen.

**D, „noch nötig“** (Sicht `RunView::needed`, `NeededView`, `Run::max_game_value`): Vor dem letzten Spiel einer Kneipe (ein Spiel übrig, noch nicht angesagt) rechnet der Kern

- `points`: Zielwert minus Konto, was ein gewonnenes Spiel bringen muss (0 oder weniger, wenn das Ziel schon steht; die Boss-Kneipe spielt alle Spiele);
- `smallest_game`: den kleinsten Spielwert der reinen Formel (ohne Relikte und Faktoren), der reicht, z. B. 88 (Pik mit Multiplikator 8) für 87 nötige Punkte;
- `max_value`: eine **Obergrenze** für ein gewonnenes Spiel ohne Wissen über das Blatt: das höchste Spiel der Formel (Grand und jede Farbe Ouvert, Hand, Schwarz angesagt mit allen Trümpfen als Spitzen; Null Ouvert Hand mit allen Stilboni; im Ramsch der Verlierer mit 120 Augen und Jungfrau) mit **jeder** erhöhenden Zeile aller Relikte, Bosse, Charaktere und Karten, als gälte ihre Bedingung, so oft ihr Auslöser feuern kann (Stichende zehnmal, jede Karte dreißigmal), dazu die wartenden Faktoren (Doppelkorn, Eierlikör, Letzte Runde). Kontra und Re zählen nicht: Der Spieler kann keinen Gegner zum Kontra bringen;
- `reachable`: `points <= max_value`.

Ehrlich heißt hier: „Nicht mehr zu schaffen“ steht nur, wenn wirklich kein gewonnenes Spiel reicht (bis auf Kontra und Re). Die Web-App zeigt in der Kopfzeile „Du brauchst noch 87 Punkte“ oder „Nicht mehr zu schaffen“ und hebt dann „Kneipe verlassen“ hervor; die Texte stehen in der Übersetzungsdatei.

**C, drei Rettungs-Schnäpse** (`consumables.ron`, `category: Schnapps`; Preise und Seltenheit **vorläufig nach Augenmaß, ohne Messung**, für `balance`):

| Schnaps | Id | Wirkung | Wann | Seltenheit, Preis |
|---|---|---|---|---|
| **Klarer** | `clear_spirit` | Steht das Konto der Kneipe unter 0, steht es wieder auf 0 (`ClearAccount`, Ereignis `AccountCleared`) | zwischen zwei Spielen einer Kneipe (`between_games`), nur mit Minus (sonst `ConditionNotMet`) | Common, 3 |
| **Absacker** | `nightcap` | Ein Spiel mehr in dieser Kneipe (`ExtraGames(1)`, Ereignis `GamesAdded`) | zwischen zwei Spielen einer normalen Kneipe oder Spelunke, nie in der Boss-Kneipe (`NotInBossPub`) | Uncommon, 4 |
| **Eierlikör** | `eggnog` | Gewinnst du das nächste angesagte Spiel, zählt es doppelt; verloren wie sonst (`NextGameIfBehind(MultiplyWin(2))`, Zeile `Spielwert ×2` nur beim Sieg, `ActiveGame::win_factors`) | mit zehn Karten vor „Skat oder Hand“, nur wenn das Konto unter der Hälfte des Zielwerts steht (sonst `ConditionNotMet`); verbraucht von der nächsten Ansage, Einmischen verschenkt ihn nicht | Common, 3 |

Auslegungen von Claude: „Zwischen zwei Spielen“ heißt im Kern „das gegebene Spiel ist noch unberührt“, denn nach der Wertung wird sofort neu gegeben; also auch vor dem ersten Spiel, wo Klarer mangels Minus aber nichts tut. Der Eierlikör braucht dagegen zehn Karten vor „Skat oder Hand“ wie der Kurzer, damit er nicht auf ein schon bekanntes Blatt nach dem Skat getrunken wird. Zwei Eierlikör stapeln (×4 beim Sieg), wie zwei Doppelkorn. Die Wertvorschau zählt den Eierlikör mit, weil sie den Wert eines Siegs zeigt.

**Bot** (`run/bot.rs`, Regeln, nicht kalibriert; Kaufwerte Klarer 1, Absacker 2, Eierlikör 1): Vor „Skat oder Hand“ trinkt er Klarer mit Minus, Absacker vor dem letzten Spiel ohne erreichtes Ziel und Eierlikör, sobald er angenommen wird.

Tests: `core/src/run/rescue_tests.rs`. Zu messen, sobald der Simulator wieder läuft: wie oft die drei gekauft und getrunken werden und was sie an „Stadtteil 2 erreicht“ ändern.

---

## 9. Charaktere

Jeder Charakter hat ein eigenes Grunddeck (seit ADR 0015: seine Ausgabe des Skatblatts mit Modifikationen ab Start, nie eine andere Zusammensetzung, 9.8), Startrelikte (oder einen Start-Schnaps) und genau eine eigene Regel; eine eigene Zielwertkurve ist möglich. Fünf Charaktere: der Alleinspieler (frei), der Nullspieler, der Drückeberger, die Schneiderin und Bruderherz (je über eine Freischaltung, 9.3). Am 03.11.2026 hat der Nutzer den Grandspieler, den Zaungast und die Wirtin gestrichen und die drei neuen bestimmt (9.4 bis 9.6).

**Zielwerte je Charakter** (Kneipentour, gemessen am 03.11.2026 gegen den PIMC-Spieler, je 3 × 300 Runs, Seeds 42 bis 44, `docs/balance/2026-11-03-charaktere.md`; Alleinspieler aus der großen Runde mit 3 × 2000 Runs, `docs/balance/2026-11-03-grosse-runde.md`). Je Stadtteil vier Kneipen / Boss-Kneipe vor dem Boss-Faktor. Zielbild für jeden Charakter: Stadtteil 2 in 35 bis 40 %, Stadtteil 3 in 12 bis 18 %, 3 bis 5 % gewonnen, ein Viertel bis ein Drittel der verlorenen Runs endet in normalen Kneipen.

| Charakter | Stadtteil 1 | Stadtteil 2 | Stadtteil 3 | Ort | Stadtteil 2 | Stadtteil 3 | gewonnen | Kneipen-Ende |
|---|---|---|---|---|---|---|---|---|
| Alleinspieler | 45 / 55 / 65 / 75 / 60 | 90 / 105 / 120 / 135 / 100 | 145 / 170 / 195 / 205 / 140 | `run.ron` | 42,3 % (40,6 % auf 3 × 300) | 17,4 % (16,1 %) | 5,2 % (6,0 %) | 36,1 % (35,6 %) |
| Nullspieler | 50 / 65 / 75 / 85 / 70 | 110 / 125 / 145 / 160 / 120 | 180 / 210 / 245 / 255 / 175 | `characters.ron` | 38,7 % (vorher 59,9 %) | 17,6 % (39,3 %) | 6,1 % (29,7 %) | 39,6 % (18,8 %) |
| Drückeberger | wie der Alleinspieler | | | `run.ron` | 44,6 % | 17,3 % | 5,3 % | 31,5 % |
| Schneiderin | 50 / 65 / 75 / 90 / 75 | 105 / 120 / 140 / 155 / 115 | 165 / 195 / 225 / 235 / 160 | `characters.ron` | 39,6 % (vorher 47,2 %) | 16,1 % (20,9 %) | 4,4 % (7,9 %) | 37,1 % (30,8 %) |
| Bruderherz | wie der Alleinspieler | | | `run.ron` | 36,3 % | 15,3 % | 4,6 % | 38,8 % |

Der Bot nutzt die Regeln der neuen Charaktere nicht gezielt (er sagt bei Bruderherz nicht öfter Rot an, meidet beim Drückeberger keine Handspiele); für einen Spieler, der das tut, sind beide vermutlich etwas leichter als gemessen.

### 9.1 Der Alleinspieler (Standard)

Normales 32er-Blatt. Spielt Farbe, Grand und seit ADR 0012 auch Null. Null zählt seit dem 01.11.2026 für alle Charaktere nach der Formel mit Stilbonus (9.2); vorher galt beim Alleinspieler die Skatordnung (23, 35, 46, 59). Startrelikt: **Der Zähler**. Dieser Charakter ist der Maßstab für alles Balancing.

Charaktere sind Daten in `core/content/characters.ron`: Startblatt als Abweichung vom 32er-Blatt (`remove`, `add`, seit dem 01.11.2026 auch `mods`: Kartenmodifikationen ab Start; seit ADR 0015 nur noch `mods`, der Ladevorgang lehnt `remove` und `add` ab), Startrelikte, Start-Verbrauchsgegenstände (`start_consumables`), optional eigene Zielwerte, Null-Wertung, die Tresen-Gewichte pro Spielart (`focus_weights`, siehe 9.2) und die eigene Regel (`rule`, `CharacterRule`). Jeder Charakter darf jedes Spiel ansagen (ADR 0012, ersetzt 0003); im Code unterscheidet kein Zweig mehr die Charaktere.

**Die eigene Regel** (`rule`) hat zwei Teile, beide Daten: `passive` und `hooks` sind dieselbe Hook-API wie bei Relikten (Abschnitt 8) und wirken wie ein Relikt, das der Charakter immer hat, ohne Platz zu belegen; in der Wertung steht die Zeile mit der Quelle `Source::Character`. `passes`, `relic_slots` und `payout_factor` ändern die Run-Struktur aus `run.ron`: Pässe pro Serie und Relikt-Plätze als Zuschlag (negativ: weniger, nie unter 0), die Groschen einer gewonnenen Serie als Faktor (`Payout::bonus` ist der Zuschlag). Seit dem 03.11.2026 kommen drei Schalter dazu, die keine Hook-API brauchen: `vouchers_return` (eingelöste Gutscheine kommen nach der Kneipe zurück, 9.4), `keep_curses` (Flüche der Ausgabe lassen sich nicht reinigen) und `zinken_discount` (Zinken auf passenden Karten billiger, 9.6). Kein Charakter nutzt `relic_slots`, `payout_factor` oder die Wirkung `RevealDefenderHand` mehr; sie bleiben als Datenoption und sind getestet. Dazu `back`: die Kartenrückseite seiner Ausgabe (9.3). Die Einsatz-Leiter (5.4) gilt pro Charakter von selbst (`MetaState::stakes` ist nach Charakter geschlüsselt).

### 9.2 Der Nullspieler

Seit ADR 0012 ein Null-Spezialist, der auch Farbe und Grand ansagen darf (siehe „Null für alle“ am Ende dieses Abschnitts). Seit dem 02.11.2026 mischt er dreimal statt zweimal pro Spiel ein, weil sein Blatt ohne Unter sonst zu selten einen ganzen Abend trägt (5.2). Seit dem 03.11.2026 liegen seine Zielwerte über denen des Alleinspielers (dessen Kurve × 1,15 bis 1,25, Tabelle in 9): Mit den alten 18 bis 30 gewann er 30 % der Runs, mit der Kurve des Alleinspielers noch 9,5 %; ein gewonnenes Null bringt im Run im Mittel 113 Punkte. Ursprünglicher Entwurf (ADR 0003): Spielt ausschließlich Null. Rangfolge 7, 8, 9, 10, B, D, K, A ohne Trumpf. Ziel: keinen einzigen Stich machen.

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

**Null-Wertung, entschieden am 01.11.2026 mit dem Simulator: die Formel.** Beide Varianten sind Daten (`null_scoring` in `characters.ron`) und laufen durch dieselbe Formel Grundwert × Multiplikator, damit Relikte auf Grundwert, Stufen und Spielwert auch für Null wirken:

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
- **Gemeinsamer Pool**: Jedes Relikt und jede Modifikation liegt bei jedem Charakter (die Null-Hausregeln auch beim Alleinspieler; die Null-Modifikationen sind mit Effekte v2 gestrichen, 7.2). Das neue Feld `focus` (`GameFocus`: `All`, `Trump` für Farbspiel und Grand, `Null`) sagt, für welche Spielart ein Inhalt gemacht ist. `focus_weights` in `characters.ron` gewichtet damit Tresen (seit 8.6 auch Auslage und Kartenmaler; bis dahin das Stempelkissen) und Ereignisse in Prozent der Seltenheit: Alleinspieler Trump 100, Null 50; Nullspieler Trump 50, Null 200; `All` immer 100. Kein Code-Zweig pro Charakter. Relikte für Trumpfspiele, die im Null Unsinn täten (Spitze, Schenken, Auf dem Trockenen), schließen Null in ihren Bedingungen aus (`Not(NullGame)`). Der Angeber verlangt Farbe oder Grand Ouvert Hand, sonst Null Ouvert.
- **Bot** (`core/src/run/bot.rs`): Aus der Hand spielt er Null Ouvert Hand ohne offene Karte, Farbe oder Grand Hand mit acht Trümpfen oder einer Ansage; sonst nimmt er den Skat auf. Mit zwölf Karten vergleicht er das Farbspiel oder den Grand der Heuristik mit Null (die zwei offensten Karten gedrückt, Null Ouvert ohne offene Karte) nach dem erwarteten Ergebnis ohne Relikte (`suit_expected_score`, `null_expected_score`; Gewinnquoten `SUIT_WIN_PERCENT`, `GRAND_WIN_PERCENT`, `NULL_WIN_PERCENT`, gemessen mit dem PIMC-Spieler). Verloren zählt doppelt. Ein Blatt, das für Farbe und Grand schwach ist und mehr als fünf offene Karten hat, wirft er weg. Unter dem Schieber (nur Hand) spielt er Null Hand nur mit höchstens zwei offenen Karten (`NULL_HAND_MAX_EXPOSED`), die er ohne Skat nicht loswird; beim Schwarzen Peter nie Null, weil ein gewonnenes Null nie Schneider ist und nichts zählt.
- **Alte Spielstände** laden weiter: Die neuen Felder (`focus`, `focus_weights`, `character_factors`) haben Standardwerte, der Charakter im Spielstand bleibt gültig. `RunError::NullNotAvailable` bleibt als Fehlercode stehen, kommt aber nicht mehr vor.

**Messung:** Vorläufig (3 × 3000 Runs, `--bot pimc:12:3`, vor dem Zusammenführen mit dem kurzen Abend): Der Nullspieler erreicht Abschnitt 2 in 34 % der Runs (vorher 37 %), Abschnitt 3 in 14 %. Der Alleinspieler spielt etwa jedes fünfte Spiel als Null. Die endgültige Messung folgt in der großen Testrunde (Entscheidung vom 01.11.2026: Messungen vorerst pausiert, später gesammelt).

### 9.3 Freischaltungen über Runs hinweg

**Gegenstands-Review (02.11.2026, 8.5)**: Neun Belohnungen zeigten auf entfernte Relikte und zeigen jetzt auf behaltene oder neue; die Ids der Freischaltungen bleiben, wer eine schon verdient hat, hat das neue Relikt frei. Gewählt von Claude nach Thema: Aus der Hand (`hand_won`) → Spickzettel (statt Ouvert-Bonus), Dickes Ding (`big_game`) → Stammtischkasse (statt Kontra), Grand Schneider → Stichfest (statt Durchmarsch), Ohne Einmischen (`boss_without_pass`) → Schutzengel (statt Schiedsrichter), Omas Liebling (`beat_grandma`) → Deckel anschreiben (statt Die Oma), Neues Viertel (`fourth_evening`) → Monatskarte (statt Stammgast), Lehrjahre (`apprenticeship`) → Spelunkenkenner (statt Lehrgeld), Den Peter los (`beat_black_peter`) → Luschenkönig (statt Schneiderbock), Trocken gekontert (`beat_kontra_hans`) → Stammfarbe (statt Auf dem Trockenen; seit ADR 0017 „Doppelt gesehen“, Bedingung: die Zwillinge schlagen, die Id bleibt). Feierabend ist von Anfang an frei. Folge: Gesperrt sind jetzt 16 von 32 Relikten (bisher etwa ein Drittel), darunter vier bisher freie (Spickzettel, Stichfest, Luschenkönig, Stammfarbe); ob das zu viel ist, zeigt die Messung der Erreichbarkeit (`--unlocks`). Die Startrelikte (Zähler, Kiebitz) und die Start-Verbrauchsgegenstände (damals Kräuter der Wirtin) bleiben; seit dem 03.11.2026 starten auch Skatpfleger, Stichfest und Stammfarbe als Startrelikte (9.4 bis 9.6), obwohl Stichfest und Stammfarbe für andere Charaktere gesperrt sein können.

**Seit ADR 0016 (Kneipentour)**: Bedingungen an Abschnitten sind umgeschrieben, die Ids bleiben (verdiente Freischaltungen gelten weiter). Neu ist die Bedingung `ReachPub(n)` (n-te Kneipe des Abends, über die Stadtteile gezählt). Vorläufig, nicht gemessen:

| Id | bisher | jetzt |
|---|---|---|
| `second_evening` („Kneipenbummel“, bisher „Zweiter Abend“) | Erreiche Abschnitt 2 | Erreiche die dritte Kneipe des Abends (`ReachPub(3)`) |
| `third_evening` („Vor dem Boss“, bisher „Dritter Abend“) | Erreiche Abschnitt 3 | Erreiche die Boss-Kneipe im ersten Stadtteil (`ReachPub(5)`) |
| `fourth_evening` („Neues Viertel“, bisher „Vierter Abend“) | Erreiche Abschnitt 4 | Erreiche Stadtteil 2 (`ReachSection(2)`) |
| `early_finish`, `boss_without_pass`, `full_till` | „Serie“ | „Kneipe“ (Bedingungen unverändert) |

`full_till` (15 Groschen nach einer gewonnenen Kneipe, gestrichen am 03.11.2026) dürfte mit dem höheren Bonus je übrigem Spiel (3 statt 1) leichter werden; zu messen. `tour_won` verlangt weiter einen gewonnenen ganzen Abend (es gibt nur noch den).

Entschieden am 01.11.2026: Neben dem Nullspieler werden auch Relikte freigeschaltet.

- Ein neuer Spielstand kennt etwa die Hälfte der Hausregeln und Figuren. Die übrigen erscheinen erst nach einer Freischaltung im Tresen.
- Freischaltungen hängen an Taten am Tisch, nicht an Spielzeit: zum Beispiel einen Grand Schneider gewinnen, Abschnitt 3 erreichen, einen Boss ohne Einmischen schlagen.
- Jede Freischaltung ist an genau einer Stelle als Daten definiert (Bedingung und Belohnung) und wird im Simulator auf Erreichbarkeit geprüft.
- Gespeichert wird im Browser des Spielers: laufender Run, Freischaltungen, Statistik der Runs. Kein Konto, kein Server-Speicher.

**Umgesetzt in M4.** Freischaltungen stehen in `core/content/unlocks.ron` (Bedingung und Belohnung). Jedes Relikt und jeder Charakter, der dort als Belohnung steht, ist bis dahin gesperrt. Der Meta-Zustand (`MetaState`: verdiente Freischaltungen, Statistik) ist ein eigenes, serialisierbares Objekt neben dem Run; er lernt aus den Ereignissen des Runs (auch aus den bereinigten Spieler-Ereignissen) und erzeugt den Start eines Runs mit den gesperrten Relikten. Anfangs gesperrt sind 15 von 31 Hausregeln und Figuren (Durchmarsch, Tournee, Jungfrau, Ouvert-Bonus, Kontra, Wirt, Oma, Schiedsrichter, dazu aus M4 Stammgast, Lehrgeld, Glückssträhne, Schneiderbock, Auf dem Trockenen, Bierdeckel, Der Schneider) und der Nullspieler. Die sieben Hausregeln und Figuren nur für den Nullspieler sind nicht gesperrt.

Seit M5 kann eine Freischaltung auch eine Kartenmodifikation belohnen (`Reward::CardMod`): Bis dahin ziehen Auslage, Kartenmaler (bis 02.11.2026 das Stempelkissen) und Ereignisse sie nicht zufällig. Noch nutzt kein Inhalt das. Die Sicht der Freischaltungen nennt jede Belohnung mit Art (Hausregel, Figur, Modifikation, Charakter), Name und Seltenheit (`reward_info`).

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

**Freischaltungen der neuen Charaktere** (03.11.2026, Bedingungen vom Nutzer, nicht gemessen):

| Freischaltung | Bedingung | Belohnung |
|---|---|---|
| `full_skat` („Voll gedrückt“) | Ein Spiel gewinnen, in dem mindestens 21 Augen gedrückt wurden (`WinGame(min_discarded: 21)`). Gezählt werden die Augen der gedrückten Karten, wie sie im Spiel zählen (mit Aufdruck, Ramponiert und Skatpfleger), aus `RunEvent::GameScored::discarded_points`; ein Handspiel drückt nichts. Nur Farbspiel und Grand wie jedes `WinGame`. | Der Drückeberger |
| `three_schneider` („Maßarbeit“) | Drei Spiele mit Schneider in einer Kneipe gewinnen (`SchneiderWinsInPub(3)`, Farbspiel und Grand; der Zähler fängt mit jeder Kneipe neu an) | Die Schneiderin |
| `true_colours` („Farbe bekennen“) | Drei Farbspiele hintereinander mit derselben Trumpffarbe gewinnen, egal welche (`SameTrumpStreak(3)`). Die Serie läuft über Kneipen hinweg im selben Run; ein verlorenes Spiel, ein Grand, ein Null oder eine andere Trumpffarbe dazwischen fängt neu an, ein neuer Run auch. Ein Ramsch ist keine Ansage: Er zählt nicht und unterbricht nicht (wie bei der Stammfarbe). | Bruderherz |

Der Meta-Zustand hält dafür zwei kleine Zähler (`pub_schneider_wins`, `trump_streak`), die wie `current_section` aus den Ereignissen lernen; auch die bereinigten Spieler-Ereignisse tragen `discarded_points`.

**Bis 03.11.2026** schalteten `grand_master` („Großer Grand“, Grand Schneider mit mindestens 144 Punkten) den Grandspieler frei, `beat_kibitz_killer` („Blinde Kuh“, den Kiebitz-Mörder schlagen) den Zaungast und `full_till` („Volle Kasse“, 15 Groschen nach einer gewonnenen Kneipe) die Wirtin. Die drei Freischaltungen sind gestrichen. Wer sie verdient hat, behält ihre Ids im Meta-Zustand; sie schalten nichts mehr frei und stören nicht. Ein Meta-Stand mit Statistik oder Einsatz-Leiter eines gestrichenen Charakters lädt weiter: Diese Einträge fallen beim Laden weg. Hatte die Web-App einen gestrichenen Charakter als Wahl gespeichert, steht wieder der Alleinspieler da.

**Stammkunde** (02.11.2026, Welle 2b): zehn Runs zu Ende spielen (`RunsFinished(10)`, `ten_runs`) schaltet „Drei drücken“ frei (8.1). Damit sind 16 von 46 Hausregeln und Figuren gesperrt. Nicht im Simulator gemessen; zehn beendete Runs sind wie „Lehrjahre“ (fünf) sicher erreichbar.

Alle Freischaltungen sind erreichbar. Die letzte ist für den Bot fast unerreichbar, weil er Abschnitt 5 kaum schafft; für Menschen ist sie das Ziel des ersten Runs, wie in 9.2 entschieden.

**Kartenrückseiten**: Standard ist die Distel-Rückseite (nach Altenburger Vorbild, eigene Pixelgrafik). Seit dem 03.11.2026 hängt die Rückseite der Ausgabe des Spielers am Charakter (`back` in `characters.ron`, ohne Angabe die Distel; Sicht `CharacterInfo::back` und `CharacterEntry::back`): Alleinspieler und Nullspieler die Distel, der Drückeberger Briefmarke und Poststempel auf Packpapier, die Schneiderin Schere und Maßband auf Aubergine, Bruderherz ein großes rotes Herz auf Creme mit kleinen Herzen. Die drei neuen sind Pixelgrafik im Stil der Gegner-Rückseiten aus `tools/cardart/backs.py`. Seit ADR 0015 zeigt die Rückseite, wessen Ausgabe eine Karte ist: Die Rückseite des Charakters gehört dem Spieler und dem Skat; die Hände der Gegner zeigen eine Gegner-Rückseite, die als Daten am Gast oder Boss hängt (`back` in `guests.ron` und `bosses.ron`, ohne Angabe eine gemeinsame Rückseite der Stammgäste). Die Gegner-Rückseiten sind keine Freischaltung und nicht wählbar. Weitere Rückseiten werden später über bestimmte Erfolge freischaltbar (Freischaltung als Kosmetik, ohne Spielwirkung). Die Web-App führt die Rückseiten als kleine Liste von Entwürfen (`web/src/ui/backs.ts`, `cardBack(className, design)`); die Wahl ist eine reine Anzeige-Einstellung und berührt den Kern nicht.

### 9.4 Der Drückeberger

Entschieden vom Nutzer am 03.11.2026. Drückt sich, wo er kann, und lebt von dem, was er drückt.

- **Ausgabe** (`Shirker`): eine Pfandmarke (`voucher_coins`, 2 Groschen) auf allen vier Siebenern, eine Totenmarke (`voucher_undead`) auf der Schellen-Acht, eine Narrenmarke (`voucher_wild`) auf der Rot-Acht. Die Namen der Marken kommen mit den Zinken (feat/zinken); im Kern heißen sie nach ihrer Id.
- **Regel, Teil 1: Marken kommen zurück.** Jeder Gutschein, der in einer Kneipe von seiner Ausgabe eingelöst wurde, liegt nach dem Ende dieser Kneipe (gewonnen oder verloren) wieder mit seinem Wert auf seiner Karte (`CharacterRule::vouchers_return`, gemerkt in `Series::returning_vouchers`, Ereignis `RunEvent::VoucherReturned` und `DeckChanged`). Das gilt für jeden Gutschein seiner Ausgabe, auch für einen später dazugekommenen. Trägt die Karte inzwischen eine andere normale Modifikation oder einen Fluch, bleibt der Gutschein weg (die Karte hat nur einen Platz, 7.2). Was die Marke aus der Karte gemacht hat, bleibt: Eine Totenmarke liegt danach neben „Untot“, und wird sie wieder gedrückt, passiert nichts weiter, weil die Karte schon Untot ist. Endet der Run mit der Kneipe, kommt nichts mehr zurück, was keine Rolle spielt.
- **Regel, Teil 2: kein Hand-Multiplikator.** Ein Handspiel zählt ohne die Stufe „Hand“: ein Hook `OnDeclare(when: All([HandGame, Not(RamschGame)]), then: [AddLevels(-1)])`, gewonnen wie verloren, auch Null Hand und Null Ouvert Hand. In der Wertung steht „Hand +1“ und die Zeile des Charakters −1. Unter der Spelunke „Kein Skat“ (alles Hand) bekommt er nie Hand, sie bleibt für ihn ziehbar.
- **Startrelikt**: Skatpfleger (gedrückte Augen zählen doppelt).
- **Freischaltung**: `full_skat` (9.3). **Rückseite**: Briefmarke und Poststempel.
- **Zahlen**: Zielwerte wie der Alleinspieler (gemessen am 03.11.2026 im Zielbild, Tabelle in 9), `focus_weights` Trump 100, Null 50 (nach Augenmaß). Gemessen: Er löst je Run 4,7 Pfandmarken (9,7 Groschen), 1,1 Toten- und 1,1 Narrenmarken ein, fast alle kehren zurück (6,9 je Run); das sind gut 9 Groschen mehr als beim Alleinspieler, im Ergebnis liegt er nur knapp darüber.

### 9.5 Die Schneiderin

Entschieden vom Nutzer am 03.11.2026. Nimmt Maß und macht die Gegner Schneider.

- **Ausgabe** (`Tailoress`): Aufdruck (`points`) +3 auf allen vier Zehnern.
- **Regel**: Gewinnt sie ein Farbspiel oder einen Grand mit Schneider, zählt Schneider +2 statt +1 Multiplikator, Schwarz ebenso +2 statt +1. Ein gewonnenes Farbspiel oder ein Grand ohne Schneider zählt 1 Multiplikator weniger, nie unter 1 (die Formel hält den Multiplikator bei mindestens 1, 4.2). Daten: drei `OnScore`-Hooks mit `AddLevels` auf `WonSchneider`, `All([Won, AllTricksWon])` und `All([Won, Not(WonSchneider)])`, je ohne Null und Ramsch.
- **Auslegung (Claude)**: Null ist ausgenommen, weil es dort keinen Schneider gibt; sonst zählte jedes gewonnene Null eine Stufe weniger, und die Schneiderin könnte praktisch kein Null spielen. Verlorene Spiele bleiben unverändert, auch ein verlorenes Spiel, in dem sie selbst Schneider ist.
- **Startrelikt**: Stichfest (erster Stich zählt für die Schneider-Grenze doppelt).
- **Freischaltung**: `three_schneider` (9.3). **Rückseite**: Schere und Maßband auf Stoff.
- **Zahlen**: eigene Zielwerte seit 03.11.2026, rund die Kurve des Alleinspielers × 1,15 (Tabelle in 9); mit dessen Kurve erreichte sie Stadtteil 2 in 47 % und gewann 7,9 % der Runs. Sie gewinnt 50 % ihrer Farbspiele und Grands mit Schneider (Alleinspieler 39 %), die andere Hälfte zählt eine Stufe weniger. `focus_weights` Trump 100, Null 50 (nach Augenmaß).

### 9.6 Bruderherz

Entschieden vom Nutzer am 03.11.2026. Sein Herz schlägt für Rot.

- **Ausgabe** (`HeartBrother`): Rot ist seine Farbe. Siegel (`trump_bonus`) +2 auf Rot-Daus, -Zehn, -König und -Ober, Winkel (`rank`) 1 auf Rot-Neun, -Acht und -Sieben. Die anderen Farben sind verflucht: Fleckig (`stained`) 2 auf Daus und Zehn von Eichel, Grün und Schellen, Ramponiert (`battered`) 2 auf den drei Königen. Unter unverändert. Die Flüche sind Teil der Start-Ausgabe; „beim Erhalt“ feuert nicht, es gibt keine Groschen dafür. Eine verfluchte Karte nimmt keine weitere Modifikation (7.2).
- **Heimatfarbe**: Rot ist durch die Ausgabe festgelegt, nicht durch eine eigene Regel. Die Stammfarbe zählt bei ihm wie bei allen jede Trumpffarbe (Auslegung von Claude: Rot gewinnt sich von selbst, die Stammfarbe soll ihn nicht festnageln).
- **Regel, Teil 1: Flüche bleiben.** Eine Karte seiner Ausgabe mit einem Fluch lässt sich nicht reinigen, weder mit dem Frischen Deck noch in der Werkstatt (`CharacterRule::keep_curses`, Fehler `RunError::CurseStays`, Code `CurseStays`). Das gilt für jeden Fluch seiner Ausgabe, auch für einen, der später dazukommt; Karten ohne Fluch reinigt er normal. Die Liste der Ziele zeigt verfluchte Karten gar nicht erst an, und der Bot zielt nicht auf sie.
- **Regel, Teil 2: Zinken auf Rot kosten die Hälfte.** `CharacterRule::zinken_discount: Some((cards: (suit: Some(Hearts)), percent: 50))`. `Run::zinken_price(karte, preis)` rechnet den Preis einer Zinke, die für Groschen auf eine Karte seiner Wahl kommt: die Hälfte, kaufmännisch gerundet, mindestens 1 Groschen. Heute gilt das für die bezahlten Zinken auf einer gewählten Karte: den Kartengeber und seine Verwandten (`MarkChosenCard` mit Preis) und den Falschspieler (`Gamble`). Die Sicht nennt den Preis je Ziel (`EncounterOptionView::target_costs`), die Web-App zeigt „4 Gr., auf Rot 2 Gr.“. **Für feat/zinken:** Die Auslage und der Kartenmaler am Tresen müssen jeden Preis einer Zinke auf einer bekannten Karte über `Run::zinken_price` berechnen (nach dem Tresen-Rabatt `shop_price` oder davor, dort zu entscheiden); die heutigen Zinken-Schnäpse am Tresen kennen ihre Karte beim Kauf nicht und bleiben unverbilligt.
- **Startrelikt**: Stammfarbe (dreimal hintereinander dieselbe Trumpffarbe ×1,5).
- **Freischaltung**: `true_colours` (9.3). **Rückseite**: ein großes rotes Herz.
- **Zahlen**: Zielwerte wie der Alleinspieler (gemessen am 03.11.2026 im Zielbild, Tabelle in 9), `focus_weights` Trump 100, Null 50 (nach Augenmaß). Seine Ausgabe ist im Rot-Spiel deutlich stärker (Grundwert 10 + 4 × 2 = 18) und in jeder anderen Trumpffarbe schwächer (Grundwert −4). Gemessen: Ein gewonnenes Rot-Spiel bringt 55 statt 39 Punkte, aber der Bot sagt Rot nicht öfter an (8,7 % der Spiele wie beim Alleinspieler). Die Flüche kosten ihn 6 Punkte „Stadtteil 2“ (42 % ohne, 36 % mit).

### 9.6a Bis 03.11.2026: Grandspieler, Zaungast, Wirtin

Gestrichen vom Nutzer am 03.11.2026. Der Grandspieler (Grand +1 Multiplikator, Farbspiel −1, Eichel-Daus mit Augen +8), der Zaungast (sah das Blatt des linken Gegners, einmal weniger einmischen) und die Wirtin (doppelte Groschen, vier Relikt-Plätze, Start mit einem Kräuter) sind aus `characters.ron`, dem `Character`-Enum, der Web-App und dem Simulator entfernt. Ihre Messungen in 5.2 und 11 bleiben als Geschichte stehen.

### 9.7 Spätere Kandidaten (nicht vor v1)

- **Der Ramscher**: Spielt Ramsch, gewinnt mit den wenigsten Augen. Eigenes Scoring.
- **Der Schieber**: Sieht immer den Skat vor der Ansage, aber alle Spiele sind Hand.

### 9.8 Grunddecks als Ausgaben (ADR 0015)

Seit ADR 0015 unterscheidet sich das Grunddeck eines Charakters nur durch Modifikationen auf seiner Ausgabe. Ziel der Vorschläge (Claude, 02.11.2026): die Eigenart jedes Charakters erhalten und seine gemessene Stärke (5.2) so wenig wie möglich verschieben. Gemessen wird erst, wenn die Simulationen wieder laufen (18.4).

| Charakter | Grunddeck bis ADR 0015 | Ausgabe ab ADR 0015 | Erwartete Verschiebung |
|---|---|---|---|
| Alleinspieler | 32er-Blatt | normale Ausgabe, keine Modifikationen | keine (Maßstab) |
| Nullspieler | ohne Unter, vier Siebener mehr | seine vier Unter tragen „Abgestuft“ (Antwort im Cockpit, umgesetzt in M9, Auslegung unten) | im Null etwas leichter, im Farbspiel etwas schwerer, siehe unten |
| Grandspieler (bis 03.11.2026) | Eichel-Daus mit Goldrand | Eichel-Daus mit Augen +8 (seit Effekte v2 statt Goldrand), aber nur in seiner Ausgabe | stärker: Die Gegner bekommen die +8 nicht mehr, wenn sie den Daus halten (20 von 32 Spielen); bei Bedarf hebt `balance` seine Zielwerte |
| Zaungast (bis 03.11.2026) | 32er-Blatt | normale Ausgabe | keine (seine Eigenart ist die Regel) |
| Wirtin (bis 03.11.2026) | 32er-Blatt | normale Ausgabe | keine (ihre Eigenart ist die Regel) |
| Drückeberger, Schneiderin, Bruderherz (seit 03.11.2026) | – | eigene Ausgabe von Anfang an (9.4 bis 9.6) | nicht gemessen |

**Der Nullspieler: Ersatz für „ohne Unter, vier Siebener mehr“.** Bisher hatte niemand am Tisch einen Unter, und es gab jede Sieben doppelt. Für seine eigene Hand war das ein Blatt aus 32 Karten mit acht Siebenern; für die Gegner hieß es auch: keine Trümpfe aus Untern und mehr niedrige Karten. Varianten:

- **A (empfohlen): neue Modifikation „Abgestuft“ (`demoted`, nur auf Untern)**: „In deiner Hand ist dieser Unter eine Sieben seiner Farbe: kein Trumpf, keine Spitze, keine Augen.“ Seine Hand zieht damit genau wie bisher aus 32 Karten mit acht Karten, die für ihn Siebenen sind; Spitzen zählt er „ohne“ wie bisher (4.4). Anders als bisher hält ein Gegner, der einen Unter bekommt, einen echten Unter: Im Null ist das eine mittlere Karte, die eher sticht (leichter für ihn), im Farbspiel ein Trumpf gegen ihn (schwerer). Eine Sieben kann doppelt im Stich liegen; bei Gleichstand behält die früher gespielte ihn (4.4). Neue Wirkung im Kern (`Demote`), wie die Blue Curaçao eine Karte, die für ihr Spiel eine andere ist.
- **B: „Unsichtbar“ auf seinen vier Untern** (mit Effekte v2 gestrichen, bräuchte eine eigene Modifikation): Die Unter gewinnen nie, bedienen aber. Im Null stärker als eine Sieben (eine Sieben kann einen Stich machen, wenn keiner bedient). Im Farbspiel bleiben sie Trumpf, und weil die Spitzen über die gedruckte Trumpfordnung zählen, zählte er „mit“ Spitzen über Trümpfe, die nie stechen; das müsste eine Ausnahme abfangen. Stärker als bisher.
- **C: Normale Unter, dafür Null-Werkzeuge auf hohen Karten**, etwa „Attrappe“ auf allen vier Daus (mit Effekte v2 gestrichen). Einfach; sein Farbspiel wird aber normal, er wäre ein Alleinspieler mit Null-Vorteil und verlöre seine Eigenart „kein Trumpfspieler“.

**Umgesetzt (M9, Antwort des Nutzers, Auslegung von Claude am 02.11.2026): „Abgestuft“ (`demoted`, Wirkung `Demote`) ist eine Spezialkarte, die ein Fluch ist (`ModKind::SpecialCurse`), genau das Gegenstück zu Rang („sticht Stufen tiefer“).** Nicht als Wert neben Rang wie Augen und Ramponiert, sondern eine eigene Art, damit sie den Platz der Spezialkarte hält und daneben eine normale Modifikation tragen kann.

- **Auf einem Unter:** Er ist in keinem Spiel mehr ein Unter, sondern die niedrigste Karte seiner Farbe, unter der Sieben: im Farbspiel einer anderen Farbe und im Grand eine Farbkarte, im Farbspiel seiner Farbe der niedrigste Trumpf, im Null unter der Sieben. Keine Spitze (er fehlt in der Trumpfordnung wie eine Karte, die die Blue Curaçao umgemalt hat), seine Augen sind 0 wie bei einer Sieben (die 2 des Unters fallen weg; Hausregeln, die Augen setzen, wirken vorher, Abgestuft setzt danach auf 0). Er ist keine Sieben: Er bringt im Null keinen Stilbonus „Sieben unterlaufen“. Im Code `rules::is_demoted_jack`, `trump_strength` (Stärke 0), `suit_strength` (0), `matadors_in` überspringt ihn.
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

**Seit ADR 0017 (03.11.2026, Entscheidung des Nutzers, Brief „Bosse v2“)** ist jeder Boss ein **Paar aus zwei Boss-Spielern** an der Boss-Kneipe am Ende jedes Stadtteils (drei je Abend, die letzte der Endboss). In der Boss-Kneipe werden alle drei Spiele gespielt; verloren ist der Run vorbei, gewonnen gibt es doppelte Groschen (gerechnet, als blieben zwei Spiele übrig, `boss_reward.games_left_as`), ein Leben und ein Relikt nach Wahl (5.0). Was weiter unten „Boss-Serie“ heißt, ist die Boss-Kneipe; die Messungen unten galten einem Boss-Platz, dem frühen Ende und den fünf Abschnitten und sind Geschichte.

Die Sonderregel gilt für die ganze Boss-Kneipe und wird vorher angezeigt, damit der Spieler sein Blatt und seine Ansagen anpassen kann. Der Boss eines Stadtteils wird beim Start des Stadtteils gezogen (Ereignis `BossAnnounced`) und ist ab dann in der Sicht lesbar (`upcoming_boss`, `next_series`, Routenwahl), mit beiden Figuren (`BossInfo::figures`). Jeder Boss hat einen Zielwert-Faktor (`target_factor` in `bosses.ron`, Standard 1,0) und kann für einzelne Charaktere einen eigenen haben (`character_factors`, ADR 0012); messen lässt sich das mit `skat_sim --curve --boss <id>` (seit ADR 0017 zählt dabei der Stand nach dem dritten Spiel).

### 11.1 Die Bosse seit ADR 0017

| Boss | Regel | Figuren (links, rechts) | Ausgaben | Thema | Faktor (vorläufig) |
|---|---|---|---|---|---|
| **Der Schieber** | Jedes Spiel ist ein Handspiel (`RequireHand`) | Der Schieber, Die Hehlerin | Siebenen Augen +4; Achten Augen +3 | Schmuggelware: Luschen, die doch etwas wert sind | 0,5 (Nullspieler 0,2) |
| **Die Oma** | Zu Beginn ihrer Kneipe tragen 16 der 32 Karten der Ausgabe des Spielers je einen zufälligen Fluch (Durchsichtig, Ramponiert, Fleckig, Standardwert), nur für diese Kneipe (`CurseForPub`) | Die Oma, Der Opa | Oberinnen Augen +3 (6 statt 3); Könige Augen +2 (6 statt 4) | aus der alten Zeit: die alte Oberin, der alte König | 1,0 (Nullspieler 0,9) |
| **Die Zwillinge** | Die Gegner sehen gegenseitig ihre Blätter und spielen danach (`DefendersSeeEachOther`, Schummeln) | Fritz, Franz | Achten Rang 2; Neunen Rang 2 | sie sehen doppelt: zwei Nullen übereinander, die Neun gleich mit | 1,2 |
| **Der Ramscher** | Jedes Spiel ist ein Ramsch (`Ramsch`, bis ADR 0017 nur das erste); ein verlorener Ramsch zählt einfach statt doppelt gegen den Spieler (`SingleLoss`, seit 03.11.2026) | Der Ramscher, Die Ramscherin | Daus Ramponiert 6; Zehnen Ramponiert 5 | sie lassen laufen | 1,0 (Nullspieler 2,0) |
| **Der Schwarze Peter** | Ein gewonnenes Spiel ohne Schneider zählt halb | Der Schwarze Peter, Die Schwarze Grete | Eichel- und Grün-Unter Durchsichtig; Eichel- und Grün-Oberin Durchsichtig | Ruß an den Fingern: eine Hilfe beim schwersten Boss | 0,7 (Nullspieler 0,2) |
| **Der Alte** (Endboss) | legt die fünf Regeln oben der Reihe nach auf, eine pro Blatt (Schieber, Zwillinge, Ramscher, Oma, Schwarzer Peter) | Der Alte, Die Alte | die Ausgaben der Figuren des Bosses, dessen Regel gerade gilt (links die erste, rechts die zweite) | das alte Paar | 1,0 |

Entfallen mit ADR 0017: **Der Kontra-Hans** (Kontra immer erlaubt), **Der Revolutionär** (Handtausch nach dem fünften Stich) und **Der Kiebitz-Mörder** (Informations-Wirkungen aus). Mit ihnen die Wirkungen `SwapDefenderHands`, `DisableInformation` und `BlankOut` (die alte Regel der Oma); `EnableKontra` bleibt für die Hausregel. Ein Spielstand, der einen von ihnen gezogen hat, trifft einen Ersatz (`RETIRED_BOSSES`: Kontra-Hans und Revolutionär die Zwillinge, Kiebitz-Mörder der Schwarze Peter; keiner verlangt Hand, damit ein wartender Stand spielbar bleibt). **Der Totengräber** (Boss mit Sonderkarten) ruht.

Leitlinien und Auslegungen (Claude, 03.11.2026):

- **Ausgaben**: je Figur eine Gruppe von vier Karten (der Peter und die Grete je zwei), eine Modifikation je Karte, wie die milde Stufe von ADR 0015. Beide Figuren teilen das Thema, unterscheiden sich aber in der Kartengruppe oder im Wert. Beide zeigen die Rückseite des Bosses (`back`; eine Figur darf eine eigene haben, `figures[].back`). Die Figuren des Alten haben keine eigene Ausgabe (Ladefehler sonst), ihre Rückseite ist seine.
- **Die Oma**: Die Flüche zieht der Generator der Route (`tour_rng`), Karte und Fluch je Karte; ein Fluch, der nicht auf die Karte passt (Fleckig nur auf Farbkarten), wird nicht gelegt. Er tritt an die Stelle der normalen Modifikation, eine Spezialkarte (Untot, Wandelkarte, Abgestuft) bleibt. Jede Karte wird vorher gemerkt (`Series::boss_curses`) und am Ende der Kneipe genau so zurückgelegt; keine Groschen, kein `OnAttach`. Unter dem Alten legt sie die Flüche beim ersten Blatt mit ihrer Regel, und sie bleiben bis zum Ende der Kneipe. Seit ADR 0017 trifft sie jeden Charakter (bis dahin nur den Alleinspieler).
- **Der Ramscher**: Jedes Spiel unter seiner Regel ist ein Ramsch. Mit drei Ramsch-Spielen entschied vor allem, ob einer verloren ging (doppelt gegen den Spieler); der Faktor wirkt kaum. **Entschieden von Claude am 03.11.2026 (17 a)**: In seiner Kneipe (und unter seiner Regel beim Alten) zählt ein verlorener Ramsch nur einfach. Das ist eine eigene Wirkung der Boss-Regel (`SingleLoss` in `passive`, dieselbe Zeile `LossFactor` wie beim Magenbitter), kein Sonderfall im Code. Gemessen vorher ohne Relikte 45 % (Stadtteil 1) und 37 % (Stadtteil 3), gegengerechnet mit einfacher Wertung 64 und 56 % (`docs/balance/2026-11-03-kleine-runde.md`, Abschnitt 3). Der Faktor bleibt 1,0.
- **Faktoren des Nullspielers** (gemessen am 03.11.2026 mit seiner neuen Kurve, `docs/balance/2026-11-03-charaktere.md`): Schieber 0,35 → 0,2, Ramscher 1,0 → 2,0, Oma 0,9 und Schwarzer Peter 0,2 bleiben. Boss-Kneipe gewonnen: Ramscher 66 % (bei 1,0 87 %: mit abgestuften Untern hat er im Ramsch keinen Trumpf), Schieber 32 % (fast unabhängig vom Faktor, der Bot spielt unter „nur Hand“ selten Null), Peter 52 %, Oma 61 %, Zwillinge 59 %, der Alte 43 %.
- **Faktoren** (Alleinspieler, gemessen am 03.11.2026, `docs/balance/2026-11-03-kleine-runde.md` und `2026-11-03-grosse-runde.md`): Schieber 0,8 → 0,5, Oma 0,7 → 1,0, Zwillinge 0,85 → 1,2, Schwarzer Peter 0,5 → 0,7, Ramscher und der Alte 1,0. Die Boss-Kurven sind flach (0,15 bis 0,25 Punkte je Zielwert-Punkt, ohne Relikte Obergrenze 51 bis 76 % bei Zielwert 1); wie schwer ein Boss ist, entscheidet seine Regel, der Faktor stellt nur fein nach. Boss-Kneipe gewonnen im Run (Stadtteil 1 / 2 und 3):

| Boss | vorher (kleine Runde) | jetzt (große Runde, Zielwerte 60 / 100 / 140) |
|---|---|---|
| Der Ramscher | 35,5 % (36 / 35) | 51,1 % (57 / 40) |
| Der Schieber | 52,1 % (52 / 53) | 51,5 % (50 / 54) |
| Der Schwarze Peter | 63,9 % (63 / 67) | 51,2 % (52 / 50) |
| Die Oma | 72,4 % (70 / 76) | 57,0 % (60 / 52) |
| Die Zwillinge | 71,8 % (73 / 70) | 58,3 % (65 / 44) |
| Der Alte (Endboss) | 49,4 % | 35,7 % |

  Der Ramscher liegt mit der einfachen Wertung (unten) bei den anderen, sein Faktor bleibt 1,0. Der Alte ist als Endboss mit 36 % der schwerste.
- **Belohnung**: Die Relikt-Wahl zieht der Ereignis-Generator aus dem Relikt-Pool des Charakters (nicht gesperrt, nicht besessen), Seltenheit Ungewöhnlich oder höher (`boss_reward` in `run.ron`); ohne passendes Relikt geht es direkt zur Routenwahl.

### 11.2 Bis ADR 0017

Bis zum 03.11.2026 waren Bosse „Gegnerpaare“ im Sinne von Boss plus Stammgast: Der Boss saß links mit seiner Ausgabe, rechts ein Stammgast mit der normalen. Acht Bosse (dazu Kontra-Hans, Revolutionär, Kiebitz-Mörder), die Oma nahm alle Unter aus dem Spiel, der Ramscher ließ nur das erste Spiel ramschen.

**Boss-Ausgaben (ADR 0015, umgesetzt in M9, milde Stufe)**: Der Schieber Siebenen Augen +4, der Kontra-Hans Könige „Letzter Stich“, die Oma Oberinnen Augen +3, die Zwillinge Achten Rang 2, der Ramscher Daus Ramponiert 6, der Revolutionär Zehnen Rang 1, der Schwarze Peter die schwarzen Unter Durchsichtig, der Kiebitz-Mörder keine, der Alte die des Bosses, dessen Regel galt.

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

**Seit ADR 0016**: Groschen je gewonnener Kneipe (Grundbetrag 3, je übrigem Spiel 2 seit 03.11.2026, vorher 3 und vor ADR 0016 1, Zinsen wie bisher); eine verlorene Kneipe zahlt nichts. Eine gewonnene Boss-Kneipe zahlt seit 03.11.2026 so, als blieben zwei Spiele übrig, und das doppelt (`boss_reward.games_left_as`, 5.0). Der Tresen kommt nur noch als Station eines Weges (5.0.1), nicht mehr nach jeder Serie; die Spelunke bringt ein Relikt oder seinen halben Preis.

**Gemessen am 03.11.2026** (Alleinspieler, PIMC-Spieler, `docs/balance/2026-11-03-grosse-runde.md`): Ein Run bringt im Mittel 31,6 Groschen aus Kneipen (vorher 55,8), dazu 4 Startkapital, 2,4 aus Verkäufen und 2,4 aus Inhalten; ausgegeben werden 36,9, davon Relikte 60 %, Zinken 19 %, Schnäpse 19 %, Kartenmaler 1 %, Neuauslage und Reinigen unter 1 %; Neu auslosen der Zinken nutzt der Bot nie. 4,0 Tresen-Besuche je Run. Eine gewonnene normale Kneipe zahlt 4,6 bis 6,0, eine gewonnene Boss-Kneipe 14,5 (vorher 6,45, weniger als eine Kneipe). Die fünf Relikt-Plätze sind beim Boss 1 im Mittel zu 4,0 belegt, ab Stadtteil 2 voll (vorher schon beim Betreten von Stadtteil 2 zu 93 %). **Preise seit 03.11.2026** nach der Einzelmessung ohne Tresen: Skatgott 9 → 12, Ein Auge zudrücken 8 → 7, Schieben 8 → 6, Schenken 8 → 6, Schutzengel 8 → 6, Feierabend 5 → 4.

**Gegenstands-Review (02.11.2026, 8.5), vorläufig ohne Messung**: Kräuter 3 (Common), Doppelkorn 3 (Uncommon), Konterbier 4 (Uncommon), Schwenker 2 (Common), Wegbier 3 (Common); Spickzettel 5 (bisher 4); Feierabend 5 (Common), Monatskarte 4 (Common), Stammtischkasse 5 (Uncommon), Deckel anschreiben 5 (Uncommon), Schutzengel 8 (Rare), Spelunkenkenner 4 (Uncommon). Neu in der Wirtschaft: Schulden am Tresen (Deckel anschreiben, bis −5, einmal pro Stadtteil), eine Kasse (2 je Kneipe, +50 % beim nächsten Tresen), die Wette (3 oder 6 Groschen) und Groschen aus Begegnungen (Wirt 5, Portemonnaie 8). Der Falschspieler kostet jetzt 2 Groschen, statt 3 zu zahlen.

**Wege und Begegnungen v2 (03.11.2026, 5.0.1 und 13.0), vorläufig ohne Messung**: Neue Einnahmen unterwegs: Fundgroschen 2, die Gratis-Optionen der Ereignisse (Kartengeber 2, Wasser 1, Kiebitzen beim Preisskat 2, Falschspieler auffliegen lassen 4, Finderlohn 4, altes Blatt 4, Kiebitz 2, Wirt 5), ein Relikt zum vollen Preis beim Alten Bekannten, ein Schnaps bei vollen Plätzen sein Tresen-Preis in Groschen. Neue Ausgaben: eine Runde ausgeben 3, aufwischen 2, die Wette auf einen Sieg 2 (zahlt 3), die Würfelbude 2, 4 oder 6 (Erwartungswert 0). Der Kiosk verkauft einen von zwei Schnäpsen 1 Groschen unter dem Tresen-Preis (nach dem Rabatt des Wirts, mindestens 1), ohne Anschreiben.

**Rettungs-Schnäpse (03.11.2026, 8.7), vorläufig ohne Messung**: Klarer 3 (Common), Absacker 4 (Uncommon, wie das Konterbier: ein Spiel mehr ist fast eine zweite Chance), Eierlikör 3 (Common). Kaufwerte des Bots 1, 2, 1.

- **Währung**: Groschen. Thematisch Zehntelpfennig, wie am Kneipentisch.
- **Einnahmen**: Grundbetrag pro gewonnener Serie, Bonus pro übrigem Spiel, Zinsen auf Guthaben (gedeckelt), Verkauf von Relikten.
- **Ausgaben am Tresen**: Hausregeln, Figuren, Verbrauchsgegenstände (seit 02.11.2026 nur Schnäpse), Zinken (Auslage, Neu auslosen, Kartenmaler, Karte reinigen; 8.6), Neuauslage.
- **Zinken (02.11.2026, 8.6), vorläufig ohne Messung**: Auslage 2 bis 4 Groschen (+1 auf Unter und Daus), Neu auslosen 1 (+1 je weiteres Mal am selben Tresen), Kartenmaler 6, Karte reinigen 3. Der Wirt nimmt auf alles 20 %.
- **Richtwerte (M2, kalibriert)**: Startkapital 4, Serie-Grundbetrag 3, Bonus pro übrigem Spiel 1, Zinsen 1 pro 5 Groschen bis maximal 5, Neuauslage 2. Hausregeln kosten 5 bis 8: starke 7 bis 8 (Bockrunde 8, Alte Dame 7, Schenken 7, Tournee 8), schwache und noch nicht wirksame 5. Figuren 6 bis 10, Verbrauchsgegenstände 2 bis 4. Messung und Begründung in `docs/balance/2026-11-01-m2-zielwerte.md`. Der Run-Bot hat damit beim Start von Abschnitt 2 im Mittel drei Hausregeln, ab Abschnitt 3 alle fünf Plätze belegt. Fürs Spätspiel fehlen noch Ausgaben (offen für M4 und M5).
- **M5 (PIMC-Spieler, 8 Spiele)**: Grundbeträge unverändert. Relikte kosten 4 bis 9, gestaffelt nach der gemessenen Wirkung allein ab Spielbeginn ohne Tresen (`skat_sim --runs 500 --no-shop --bot pimc:12:3 --relic <id>`, Seeds 42 bis 44, Veränderung von „Abschnitt 3 erreicht“, der stärkere Charakter zählt): ab +6,5 Prozentpunkten 9 (Bockrunde, Alte Dame, Kleinvieh; Stammgast wegen +6,6 auf Abschnitt 4), ab +4,5 8 (Schenken, Ein Auge zudrücken, Siebenschläfer, Schieben), ab +3 7, ab +1,5 6, darunter 5. Grenzfälle eine Stufe tiefer (Durchmarsch +4,5: 7; Kontra +1,6, beim Nullspieler −0,9: 5). Relikte, die vor allem Menschen nutzen (Hand, Ouvert, Information) oder nur gegen einzelne Bosse wirken, 4 bis 6. Hausregeln und Figuren folgen derselben Staffel. Tabelle in `docs/balance/2026-11-01-m5-v1.md`. Die Schnäpse (8.3) kosten 2 bis 3 als Platzhalter, ohne Messung.
- **Seltenheit folgt der Stärke (M5)**: Vorher lagen die drei stärksten Relikte beider Charaktere in Common und kamen in 20 bis 21 % aller Angebote. Bockrunde, Alte Dame, Kleinvieh und Siebenschläfer sind jetzt Uncommon, Spitze, Mit Vieren, Schiedsrichter und Saubere Weste dafür Common; Anteil der drei stärksten am Angebot 11 % (Alleinspieler) und 14 % (Nullspieler). Wirkung im Run (2 × 1000 Runs): Abschnitt 2 / Run gewonnen 36,8 / 1,45 % → 35,9 / 1,0 % (Alleinspieler), Nullspieler 47,5 / 3,8 → 46,5 / 3,9 %, im Rahmen des Rauschens. Gewählt, weil ein seltener Fund stärker sein soll als ein häufiger, nicht wegen der Zahl.
- Ein Run bringt dem Bot im Mittel 23 (Alleinspieler) bis 26 Groschen (Nullspieler), fast alles wird ausgegeben.
- **Serien mit 3 Spielen (01.11.2026)**: Der Bonus pro übrigem Spiel (1 Groschen) bringt jetzt 0 bis 2 statt bis zu 6. Grundbetrag, Zinsen und Preise unverändert; sie misst `balance` mit der vollen Kalibrierung neu. Kaufwerte des Bots (`CONSUMABLE_VALUES`): Neues Blatt 2, Ein Bier und Kräuter 1 (vorher 4, 3, 3), weil sie nur helfen, wenn die zwei Einmischen eines Spiels verbraucht sind: Der Bot mischt etwa jedes zweite Blatt ein (0,75 Einmischen je Spiel, 5.2), drei schwache Blätter hintereinander kommen etwa in jedem achten Spiel. Die Testrunde (`docs/balance/2026-11-01-testrunde.md`) hatte gezeigt, dass er sie mit 6 Pässen kaufte und kaum brauchte.
- **Welle 3 (02.11.2026, 7.2 und 8.4)**: Neue Seltenheit Legendär mit dem Tresen-Gewicht 0,3 (Rare 1); die Seltenheit setzt nicht den Preis. Preise nach der Staffel aus Welle 2a: Skatgott 9 (+4,7, im Spätspiel stärker als alles gemessene), Ehrenvorsitz 6 (ohne Wirkung beim Bot, ein Relikt für Menschen). Kaufwerte des Bots in `LEGENDARY_VALUES` (9 und 1). Wandelkarte (Rare, 5), Glasdaus (Uncommon, 4) und Heimweh (Uncommon, 4) hatten wie alle Modifikationen keinen eigenen Tresen-Preis; sie kamen über Stempelkissen und Ereignisse (Glasdaus und Heimweh seit Effekte v2 gestrichen). Alles Platzhalter für `balance`, Bericht `docs/balance/2026-11-02-welle3.md`.
- **Welle 2a (02.11.2026, 8.1)**: Mit 3 Spielen je Serie fallen alle Wirkungen kleiner aus (Alte Dame +5,3 statt +7,1 auf „Abschnitt 3 erreicht“). Die Preise der acht neuen Hausregeln folgen der M5-Staffel auf drei Viertel verkleinert (ab +4,9 Punkten 9, ab +3,4 8, ab +2,3 7, ab +1,1 6, darunter 5; Grenzfälle eine Stufe tiefer), Relikte für Menschen 6. Kaufwerte des Bots in `RELIC_VALUES`, hochgerechnet auf den M5-Maßstab. Die Preise der älteren Relikte bleiben, bis `balance` alles mit 3 Spielen neu misst.
- **Welle 2b (02.11.2026, 8.1)**: Spickzettel 4 nach der Staffel der Welle 2a (Spickzettel billiger als der Kiebitz, weil er nur im Handspiel wirkt). Drei drücken wirkt seit der Entscheidung des Leads nur einmal pro Serie und liegt damit bei +7,9 (auf den Zielwerten aus PR #47), etwas über der Alten Dame: Rare, 9 Groschen (oberste Stufe der Staffel), gesperrt hinter „Stammkunde“ (9.3). Kaufwerte des Bots: 10, 3, 1.
- **Effekte v2 (02.11.2026, 7.2)**: Modifikationen mit konkreten Werten gibt es am Tresen als Verbrauchsgegenstände (Augen +3 für 2, Augen +5 für 3, Trumpf +2 für 3, Trumpf +4 für 4, Rang +1 für 3 Groschen), alles Platzhalter ohne Messung. Kaufwerte des Bots in `CONSUMABLE_VALUES` (1 bis 2, auch Platzhalter); er setzt sie wie das Stempelkissen auf einen Daus ohne Modifikation. Seit 8.6 entfernt (Zinken am Tresen). Flüche zahlen weiter 3 Groschen beim Erhalt, der Gutschein bringt +3 beim Drücken.
- **Spielwahl und Einsatz** (Bot, 01.11.2026): `suit_expected_score` rechnet mit der Gewinnschwelle des Spiels (Einsatz und Relikte): Über 61 schrumpft die gemessene Gewinnquote mit den übrigen Augen, `(120 − Schwelle) / 59` davon (grobes Modell, nicht gemessen). So wählt der Bot bei hohem Einsatz öfter Null, das keine Augen zählt.

---

## 13. Ereignisse (Rast)

### 13.0 Wege und Begegnungen v2 (03.11.2026)

**Entscheidung des Nutzers (Cockpit-Fragen 10 und 11)**: Jedes Ereignis bietet **zwei oder drei Optionen**, eine davon **muss** genommen werden; „Weitergehen“ und „Ablehnen“ gibt es nicht mehr. Was unten über „Weitergehen“ steht, ist Geschichte.

**Ereignisse sind Daten.** In `encounters.ron` listet jedes Ereignis seine Optionen (`options`, `EncounterOptionDef`): einen Titel (`label`), einen Preis (`cost`, bei Wette und Würfelbude der Einsatz) und eine kleine Liste von Wirkungen (`effects`, `EncounterEffect`), die nacheinander gelten. Es gibt keine Rust-Variante je Ereignis mehr (bis dahin `EncounterKind` und `EncounterChoice`). Die Wirkungen: Groschen (`Coins`), eine Hausregel des Stadtteils (`EveningRule`), eine Zinke oder ein Fluch auf eine zufällige oder gewählte Karte (`Mark`, mit `card: Random | Chosen`, benannter Zinke, Angeboten zum Ziehen, `curse` und `curse_percent`), ein Relikt tauschen oder verkaufen (`SwapRelic`, `SellRelic(Prozent)`), ein Relikt oder sein Gegenwert (`Relic`, `RelicCoins`, die Beute der Spelunke), das nächste Spiel ×n (`NextGameFactor`), Einmischen in der nächsten Kneipe mehr oder im ersten Spiel weniger (`NextPubRedeals`), beide Skatkarten im nächsten Spiel (`SeeSkat`: wie der Kurzer bei jedem Geben, bis ein Spiel angesagt ist), ein Schnaps (`Schnapps`), Leben (`Lives`), die nächsten Wege (`Scout`), eine Wette (`Bet` auf Schneider oder Sieg, Auszahlung in Prozent des Einsatzes), Würfeln (`Dice`, Gewinnchance und Auszahlung in Prozent), der Kartenmaler und Karte reinigen (`Painter`, `Cleanse`). Was eine Wirkung offen lässt (`None`), wird beim Öffnen mit dem Ereignis-Generator gezogen und vorher gezeigt: Hausregeln und Schnäpse verschieden je Option, ein Relikt für das ganze Ereignis, eine Zinke je Wirkung. Eine Option mit `fallback` („Zuschauen“) gibt es nur, wenn sonst nichts geht.

**Gesperrt statt versteckt.** Eine Option, die gerade nicht geht, bleibt sichtbar und nennt den Grund (`OptionBlock` in der Sicht, `blocked`): zu wenig Groschen, nichts mehr zu ziehen, Spezialkarten-Grenze der Ausgabe (7.2), keine passende Karte, kein Relikt, alle Relikt-Plätze belegt, kein Leben fehlt, eine Wette läuft schon. **Jedes Ereignis hat eine Option, die nichts kostet** (geprüft beim Laden), meist mit Gewinn; nur die Wette setzt in jeder Option und kommt deshalb nur auf einen Weg, wenn der kleinste Einsatz bezahlbar ist. Ein Ereignis, von dem gerade keine Option genommen werden kann, kommt nicht auf einen Weg (5.0.1).

**Schnaps bei vollen Plätzen** (entschieden von Claude): Eine Schnaps-Option bleibt wählbar und zahlt den Tresen-Preis des Schnapses in Groschen (ohne Rabatt), die Sicht sagt es (`coins_instead`). So ist ein Schnaps nie verschenkt, und das Ereignis behält seine Gratis-Option.

**Relikt als Ziel.** Tauschen und Verkaufen fragen nach dem Relikt: Das Ziel von `ChooseEncounter` ist dann der Index des Relikts (Sicht `target: Relic`), sonst eine Karte (`Card`) oder nichts.

**Alte Spielstände**: Ein offenes Ereignis der alten Form (Optionen mit `choice`) behält beim Laden nur seine Id und wird neu geöffnet (`Run::drop_unknown_content`), mit neu gezogenen Angeboten; hat es nichts mehr zu bieten, geht der Weg weiter. Eine laufende Wette der alten Form zahlt wie bisher (`factor`).

| Ereignis | A | B | C |
|---|---|---|---|
| Stammtisch-Streit | Hausregel 1 | Hausregel 2 | Hausregel 3 |
| Der Kartengeber (Upgrade) | Zinke auf zufällige Karte, gratis | gewählte Karte, 3 Gr. | Zuschauen: +2 Gr. |
| Alter Bekannter | Relikt gegen ein gezogenes tauschen | Relikt zum vollen Preis verkaufen | ein Schnaps |
| Letzte Runde | nächstes Spiel ×2 | eine Runde ausgeben (3 Gr.), +1 Einmischen in der nächsten Kneipe | nur Wasser: +1 Gr. |
| Umgekipptes Bier | zufällige Karte ramponiert (Fluch, zahlt 3 Gr.) | gewählte Karte, 4 Gr. | aufwischen, 2 Gr., keine Karte |
| Gasthof zur Krone (Upgrade, Alleinspieler) | Siegel +4 auf zufällige Karte, gratis | gewählte Karte, 4 Gr. | Kronenbier: ein Schnaps |
| Der Taschenspieler (Upgrade, Nullspieler) | Narrenmarke auf zufällige Karte, gratis | gewählte Karte, 3 Gr. | Kartentrick: beide Skatkarten im nächsten Spiel |
| Preisskat | nächstes Spiel ×3 | nächstes Spiel ×2 | kiebitzen: +2 Gr. |
| Gast aus Altenburg | Hausregel 1 | Hausregel 2 | Winkel +1 auf gewählte Karte |
| Der Wirt spendiert | Schnaps 1 | Schnaps 2 | 5 Gr. (bis dahin drei Schnäpse oder 5 Gr.) |
| Falschspieler am Nebentisch | Trick für 2 Gr., jedes zweite Mal ein Fluch | auffliegen lassen: +4 Gr. | gegen ihn spielen: nächstes Spiel ×2 |
| Verlorenes Portemonnaie | behalten: +8 Gr., Durchsichtig auf zufällige Karte (Fluch) | abgeben: +1 Leben | Finderlohn: +4 Gr. |
| Der Skatlehrer | Winkel +1 auf gewählte Karte, im ersten Spiel der nächsten Kneipe einmal weniger einmischen | Theorie: beide Skatkarten im nächsten Spiel | ein Kurzer |
| Die Wette | 3 Gr. auf Schneider (×2 zurück) | 6 Gr. auf Schneider (×2) | 2 Gr. auf Sieg (×1,5, also 3) |
| Alte Karten im Schrank | Totenmarke auf gewählte Karte, ein Fluch auf eine zufällige andere | Narrenmarke ebenso | altes Blatt verkaufen: +4 Gr. |
| Kiebitz am Fenster | die nächsten 2 Wege genau sehen | +1 Einmischen in der nächsten Kneipe | +2 Gr. |
| Fundgroschen (neu, `found_coins`) | Aufheben: +2 Gr. | | |
| Die Würfelbude (neu, `dice_stall`) | 2 setzen | 4 setzen | 6 setzen; mit 50 % doppelt zurück. Ohne 2 Groschen nur „Zuschauen“ |

Dazu unverändert: die Kartenwerkstatt (Kartenmaler gratis oder Karte reinigen), die Imbissbude (+1 Leben) und die Beute der Spelunke (das Relikt oder sein halber Preis, `RelicCoins(50)`). Der Geburtstag (enc-06) ist nicht gebaut. Alle Zahlen **vorläufig**, nach Augenmaß; der Bot wählt mit einem einfachen Wert je Option (`RunBot::option_value`, 5.0.4). Stichprobe nach dem Umbau (`skat_sim --seed 42 --runs 300 --bot pimc:12:3`, keine Kalibrierung): Stadtteil 2 erreicht 40,3 % (vorher etwa 41 %), Stadtteil 3 16,3 %, Run gewonnen 6,3 % (vorher etwa 6 %); je Run 3,5 Tresen-Besuche (vorher 4,0), 0,2 Kiosk-Besuche und 1,5 Fundgroschen, denn der Bot nimmt den Fundgroschen-Weg gern (Wert 2 gegen 1 für eine Begegnung). Beim Kartengeber und bei der Krone nimmt er fast immer die Groschen oder den Schnaps statt der Zinke.

**Gegenstands-Review (02.11.2026, 8.5)**: neu sind Der Wirt spendiert, Verlorenes Portemonnaie, Der Skatlehrer, Die Wette, Alte Karten im Schrank und Kiebitz am Fenster; der Falschspieler heißt jetzt „Falschspieler am Nebentisch“ und verkauft einen starken Kartentrick, der in der Hälfte der Fälle ein Fluch ist (die alte Form, ein Fluch gegen Groschen, entfällt; `MarkCard { curse: true }` bleibt in den Daten möglich). Alle übrigen Stationen bleiben. Werte und Gewichte (je 1) vorläufig.

**Seit ADR 0016** sind Ereignisse Stationen auf einem Weg zwischen zwei Kneipen (5.0.1) und tragen eine Kategorie: Begegnung (Stammtisch-Streit, Alter Bekannter, Letzte Runde, Umgekipptes Bier, Preisskat, Falschspieler, Gast aus Altenburg), Upgrade (Kartengeber, Gasthof zur Krone, Taschenspieler, neu die Kartenwerkstatt: eine Modifikation auf eine gewählte Karte oder eine Karte säubern, beides gratis; seit 8.6 statt der Modifikation der Kartenmaler gratis, eine von zwei Zinken) und Rast (neu die Imbissbude: ein Leben zurück). Die Wahrscheinlichkeit `encounter_chance` entfällt. Die Beute der Spelunke ist ein Ereignis, das nie auf einem Weg liegt (`spelunke_loot`, Gewicht 0). Der Stammtisch-Streit gilt bis zum Ende des Stadtteils.

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
- Seit 03.11.2026 im ganzen Abend (`--runs N`): je Kneipe Spiele und Einmischen, Leben und Relikte beim Betreten, Groschen je gewonnener Kneipe, Spelunken in eigenen Zeilen; je Stadtteil Leben und Relikte zu Beginn und vor dem Boss, verlorene Kneipen je Run, wo die verlorenen Runs enden (normale Kneipe, Spelunke, Boss-Kneipe); Groschen nach Herkunft und Ausgaben nach Art (Relikte, Schnäpse, Zinken, Kartenmaler, reinigen, neu auslosen, Neuauslage). `--curve` und `--curve --boss <id>` messen mit `--district 1|2|3` im gewählten Stadtteil (dessen Gegner aus `run.ron`).

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

- **Bosse v2 (ADR 0017, 03.11.2026)**: Faktoren des Alleinspielers gemessen am 03.11.2026 (11.1), Boss-Kneipen jetzt 51 bis 58 %, der Alte 36 %; die des Nullspielers und der übrigen Charaktere noch nach Augenmaß. Offen: die Zwillinge fallen von 65 % in Stadtteil 1 auf 44 % danach (PIMC-Gegner, die sich gegenseitig sehen), der Ramscher von 57 auf 40 %; ob der Alte als Endboss mit 36 % zu schwer ist, zeigen echte Runs. (a) ~~**Der Ramscher**: ob ein verlorener Ramsch in seiner Kneipe nur einfach zählen soll.~~ Entschieden von Claude am 03.11.2026: ja, einfach (`SingleLoss`, 11.1), Faktor bleibt 1,0; im ganzen Abend neu messen. Offen zum Entscheiden: (b) **Die Oma unter dem Alten**: Ihre Flüche bleiben ab ihrem Blatt bis zum Ende der Kneipe (Auslegung von Claude), nicht nur für ein Blatt. (c) Eigene Rückseiten je Figur (heute teilen beide die des Bosses) und weitere Bosse als Daten, zuerst der Totengräber (ruht). (d) Der Bot wählt das Boss-Relikt nach seiner Werttabelle (`relic_value`), nicht nach der Seltenheit.

- **Neue Charaktere** (03.11.2026, 9.4 bis 9.6): Drückeberger, Schneiderin und Bruderherz spielen mit den Zielwerten des Alleinspielers und seinen Tresen-Gewichten, nichts gemessen (Simulationen ruhen). Zu messen, sobald sie wieder laufen (`--character shirker|tailoress|heart_brother`): ob die zurückkommenden Marken des Drückebergers zu viele Groschen bringen; ob die Schneiderin mit −1 ohne Schneider überhaupt durch Stadtteil 1 kommt; wie stark Bruderherz im Rot-Spiel ist; Erreichbarkeit der drei Freischaltungen (`--unlocks`, der Bot drückt selten 21 Augen). Für feat/zinken: Auslage und Kartenmaler müssen `Run::zinken_price` nutzen (9.6).

- **Gegenstands-Review** (02.11.2026, 8.5): Preise von Skatgott, Ein Auge zudrücken, Schieben, Schenken, Schutzengel und Feierabend am 03.11.2026 nach Messung gesetzt (12). **Offen: Der Schutzengel** ist mit zwei Startleben vom wirkungslosen zum zweitstärksten Relikt geworden (ohne Tresen +14,8 auf „Stadtteil 2“, +9,0 auf „Stadtteil 3“, `docs/balance/2026-11-03-grosse-runde.md`); deshalb seit dem Nachtrag zur großen Runde wieder 8 (ungemessen). Feierabend wirkt beim Bot weiter nicht. Die übrigen Werte nach Augenmaß. Zu messen: Wirkung von Schutzengel, Kasse, Deckel und Feierabend auf „Stadtteil 2/3 erreicht“, Drei drücken nur im ersten Spiel, Spickzettel mit beiden Karten, Doppelkorn mit Neugeben; Erreichbarkeit der Freischaltungen mit jetzt 16 von 32 gesperrten Relikten; ob der Skatlehrer („im ersten Spiel ein Einmischen weniger“) und die Wette (nur Schneider zählt) so gemeint sind. Offen beim Nutzer: der endgültige Name von „Rang +1“ und die neue Tresen-Mechanik für Kartenverbesserungen; die Form des Bierdeckels nach Anzeige-Gruppe.

- **Kneipentour (ADR 0016)**: Zielwerte, Leben (2/4/+1) und Bonus je übrigem Spiel (2) für den Alleinspieler gemessen am 03.11.2026 (5.0.4): Stadtteil 2 42,3 %, Stadtteil 3 17,4 %, gewonnen 5,2 %, 36 % der verlorenen Runs in normalen Kneipen, alles am oberen Rand des Zielbilds. Nächster kleiner Schritt, falls gewünscht: Boss-Zielwert in Stadtteil 1 60 → 70 (weniger Runs in Stadtteil 2, mehr Ende beim Boss). Offen: **alle anderen Charaktere** mit der neuen Kurve und zwei Startleben (der Nullspieler behält seine Zielwerte 18 bis 30 und dürfte jetzt viel zu leicht sein; Drückeberger, Schneiderin, Bruderherz spielen die steile Kurve ungemessen); die **Spelunke** (Zielwert × 1,25 auf der steilen Kurve: in Stadtteil 3 nur 20 bis 36 % gewonnen; seit dem Nachtrag `target_factor` 1,1, ungemessen); die Bot-Regel „Spelunke nur mit 3 Leben“ passt nicht mehr zu zwei Startleben; Gewichte der Stationen, Gegnerstärke je Stadtteil. Auslegungen von Claude: die Spelunke kommt nie vor einer Boss-Kneipe; unter drei Wegen ist immer ein Tresen; Rast nur mit fehlendem Leben; der Malus „Kein Skat“ trifft den Nullspieler nicht. Seit ADR 0017 erledigt: Sekundant des Bosses (zwei Boss-Spieler) und die Wiederholung einer verlorenen Boss-Kneipe (sie beendet jetzt den Run). Offen im Cockpit: die Form des Malus (umgesetzt: einer, zufällig, vorher sichtbar). Die Stadtkarte (Pixel-Stadtplan) wird getrennt entworfen; die Sicht liefert dafür den ganzen Plan, Knoten-Ids und Seeds (5.0.3).

- **Effekte v2** (02.11.2026, 7.2): Werte, Preise und Seltenheit sind Platzhalter, nichts gemessen (Simulationen pausiert). Entschieden: Moment-Modifikationen und Spezial-Gutscheine höchstens einmal, Groschen-Gutschein, Augen und Trumpf ohne Obergrenze, Trumpf und Fleckig streng (Cockpit). Mit M9 erledigt: Frisches Deck reinigt (7.3), die Sicht zeigt die Ausgabe des Gegners auf einer durchsichtigen Karte (7.2).
- **Eigene Ausgaben (ADR 0015)**: beantwortet im Cockpit und umgesetzt in M9 (02.11.2026): Trumpf und Fleckig streng, Boss-Ausgaben mild, Stammgäste normal, Frisches Deck reinigt, Nullspieler mit Abgestuft, laufende Spielstände verworfen. Offen: alle Messungen (18.4, Simulationen pausiert); ob die Gleichstands-Regel bei Abgestuft auf Daus und Zehn (gleiche Stärke, die frühere gewinnt) für Menschen verständlich ist; ob die Web-App vor der Boss-Serie zusätzlich die Reihenfolge der Ausgaben des Alten zeigen soll (heute nur die Ausgabe der Regel im laufenden Blatt).

- **Zielwerte für 3 Spiele**: erledigt am 02.11.2026 (5.2, je Charakter eigene Kurve, Boss-Faktoren 11, Einsatz-Leiter 5.4, Freischaltungen 9.3). Offen daraus: (a) **Nullspieler gewinnt nur 0,6 % der ganzen Abende** (Ziel 1 bis 2 %); seine Kurve liegt ab Abschnitt 3 fast am Boden, Zielwerte bewegen dort kaum noch etwas. Entschieden am 02.11.2026: ein Einmischen mehr (`rule: (redeals: 1)` in `characters.ron`, 0,69 → 1,12 % bei gleichen Seeds), siehe 9.2. (b) **Der Schwarze Peter** bleibt mit 35 bis 48 % der schwerste Boss; mit 3 Spielen liegt seine Obergrenze bei 50 % (Nullspieler 37 %), kein Faktor hilft. Entschieden am 02.11.2026 (Cockpit): gewonnene Spiele ohne Schneider zählen halb statt null (11); die Faktoren neu messen, wenn die Simulationen wieder laufen. (c) Preise von Schieben, Ein Bier, Kräuter, Neues Blatt und Reue (8.1, 8.3) und ob Einmischen beim Alten (Regel je Blatt) zu leicht macht, sind noch nicht gemessen.

- **Null-Formel**: entschieden in M4 (9.2). Offen bleibt das Feintuning der Stilboni mit Testspielen.
- **Zielwert-Faktoren pro Charakter**: erledigt mit ADR 0012 (`character_factors` in `bosses.ron`, 11). Schieber und Schwarzer Peter sind wieder bei beiden Charakteren. Seit ADR 0017 trifft die Oma mit ihren Flüchen jeden Charakter (Nullspieler mit eigenem Faktor 0,9); messen mit `--curve --boss grandma --character null`, wenn die Simulationen wieder laufen.
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

**Gegenstands-Review (02.11.2026)**: keine neue Spielstand-Version; entfernte Relikte und Verbrauchsgegenstände fallen beim Laden weg, umbenannte werden umbenannt (8.5, `Run::drop_unknown_content`). Die Fixtures mit entfernten Relikten (`oma.json`, `kontra.json` und andere mit `bock_round`) bleiben als Ladetest im Kern (`wasm`, `old_saves_still_load_and_play_on`).

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

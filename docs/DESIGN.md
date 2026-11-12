# Stammtisch – Design-Dokument

Stand: 2026-11-03 · Status: Entwurf v0.5 (eigene Ausgaben, ADR 0015; Effekte v2, 7.2) · Sprache des Projekts: Deutsch (Code und Kommentare: Englisch) · Arbeitstitel bis 31.10.2026: „Skat Roguelike“

Dieses Dokument hält die Grundentscheidungen fest und beschreibt das Spiel so weit, dass ein erster spielbarer Prototyp daraus gebaut werden kann. Alles unter „Offen“ ist noch nicht entschieden. Alles andere gilt, bis es hier geändert wird.

---

## 1. Pitch

**Stammtisch: Skat als Roguelike-Deckbuilder.** Du bist immer der Alleinspieler. Über einen Run hinweg spielst du Skat gegen zwei KI-Gegenspieler, musst in jeder Runde einen Punkte-Zielwert erreichen und veränderst zwischen den Runden deine Ausgabe des Blatts: Gegeben wird ein Skatblatt für alle drei, aber jeder hält seine eigene Ausgabe jeder Karte (ADR 0015). Die Spielwertformel des echten Skats (Grundwert × Stufen) ist die Score-Engine, Hausregeln sind die Relikte, die Kneipe ist der Shop.

Referenzen: Balatro (Score-Formel, Run-Struktur, Shop), Inscryption (Gegner als Figuren, Regelbrüche), Slay the Spire (Charaktere mit eigenem Kartenpool).

Darstellung: deutsches Blatt (Eichel, Grün, Rot, Schellen; Unter, Oberin, Daus), siehe ADR 0010 und 0014. Dieses Dokument verwendet weiter die französischen Begriffe, weil sie im Code und in der Skatordnung stehen. Sprache des Spiels: Deutsch, Englisch als zweite Sprache (ADR 0023). Zielgerät: Handy hochkant zuerst.

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
- **Grundwerte**: Karo 9, Herz 10, Pik 11, Kreuz 12, Grand 24. Im Run zählt Grand seit dem 04.11.2026 nur 18 (4.2).
- **Spitzen**: „Mit“ oder „ohne“ wird ab Kreuz-Bube lückenlos gezählt. Im Farbspiel zählen Trumpfkarten der Farbe mit.

### 4.2 Die Spielwertformel als Score-Engine

```
Spielwert = Grundwert × Stufen

Stufen     = Spitzen
             + 1  (Spiel)
             + 1  Hand
             + 1  Schneider
             + 1  Schneider angesagt
             + 1  Schwarz
             + 1  Schwarz angesagt
             + 1  Ouvert
```

**Seit ADR 0018 (04.11.2026): Spielwert = Grundwert × Stufen × Faktor.** Der Faktor beginnt bei ×1 und wächst nur multiplikativ (×1,5 und ×1,5 ergeben ×2,25). Er kommt nicht von selbst, sondern aus Relikten mit festem Faktor unter einer Bedingung (eines je Farbe, eines für Grand, eines für Null), aus wachsenden Relikten (zum Beispiel +×0,1 je gewonnener Kneipe) und aus einzelnen Begegnungen und Boss-Belohnungen. Dauerhafte Faktoren der Relikte gelten nur für gewonnene Spiele; vorübergehende Faktoren, die ausdrücklich auch verloren zählen (Stammfarbe, Bock, Einsatz), bleiben. Zahlen vorläufig bis zur Messung. Grund: Grundwert und Stufen wachsen nur additiv, für einen Endlos-Modus braucht ein Build eine Quelle exponentiellen Wachstums.

**Umsetzung (04.11.2026)**: Im Code ist der Faktor `ScoreBreakdown::multiplier`, `ScoreStep::multiplier` und `ValuePreview::multiplier` (Typ `Factor`, Tausendstel; die Namen bleiben, damit Spielstände laden; die Stufen heißen im Code `levels`). Alle Spielwert-Faktoren (`MultiplyGameValue`, `ScaleGameValue`, neu `ScaleGameValuePerCounter`, Kontra, Re, Schnäpse) werden in `score_parts` multipliziert, gerundet wird einmal am Ende, halb auf; der Goldene Vektor `factor_as_third_stage_vectors` prüft ×1,5 × ×1,5 = ×2,25. Die Wertung zeigt Grundwert × Stufen und die dritte Box Faktor nur, wenn das Spiel einen Faktor ungleich ×1 hat (Entscheidung des Nutzers 04.11.2026, Cockpit q22; bis dahin stand sie immer da, grau bei ×1); ebenso die Wertvorschau über dem Stich; das Zeichen des Faktors (ein „×“ mit Hochzeichen, bernsteinfarben) ist **vorläufig**, bis der Nutzer eins wählt. Die Relikt-Familie „Faktor“ steht in 8.10.

**Grundwerte im Run (Entscheidung des Nutzers vom 04.11.2026, Cockpit q27)**: Grand zählt im Run **18 statt 24**, die Farben bleiben 9, 10, 11, 12, Null zählt nach der Null-Wertung des Charakters (9.2; ob Null angepasst wird, entscheidet die Balancing-Runde). Ein Relikt „Grand zu 24“ kann den alten Wert zurückbringen (Content). Im Kern bleibt die reine Regelfunktion bei der Skatordnung (`GameType::base_value`, `rules::game_value`, Golden-Vektoren unverändert); der Run nimmt seine Grundwerte aus den Daten (`base_values` in `run.ron`, `RunConfig::base_value`) und wertet mit `RunConfig::game_value_parts` (`ValueParts::with_base_value`): Abrechnung, Wertvorschau, „Noch nötig“, die Spielarten der Ansage (`GameOption::base_value`) und die Erwartungswerte des Bots. Regelkarte und Glossar der Web-App sagen Grand 18. **Vorläufig**, bis die Balancing-Runde misst.

In der Sprache des Genres: **Grundwert sind die Chips, die Stufen (Gewinnstufen; vom 02.11. bis 04.11.2026 in Spieltexten „Multiplikator“, Code `levels`) sind der Mult.** Seit 04.11.2026 (Cockpit q23) heißen sie in allen Spieltexten wieder „Stufen“ („+1 Stufe“, „+2 Stufen“), „Faktor“ ist das einzige Wort fürs Malnehmen. Zeichen der Stufen ist eine schlichte Treppe (Band und Wertung, kupferfarben), es ersetzt den Steilpfeil. Alle Modifikationen im Spiel greifen an genau einer dieser Stellen an:

| Hebel | Beispiele für Upgrades |
|---|---|
| Grundwert erhöhen | Kreuz zählt 15 statt 12. Grand zählt 30. |
| Spitzen manipulieren | Ein fünfter Bube im Blatt. Karo-Bube zählt als zweite Spitze. |
| Stufen erhöhen | Neue Stufen durch Hausregeln: „Spitze“ (letzter Stich mit einer Sieben), „Kontra“, „Bock“. |
| Augen verändern | Pik-Ass zählt 15. Alle 10er zählen 11. Siebener zählen 1. |
| Schwellen senken | Schneider ab 85 Augen. Gewinn ab 58 Augen. |

### 4.3 Was gebogen wird

**Reizen entfällt.** Stattdessen gibt es die **Ansage**. Die Reihenfolge folgt der Skatordnung (ADR 0011): Erst entscheidet der Spieler, ob er den Skat aufnimmt oder Hand spielt; nach dem Aufnehmen drückt er zwei Karten. Danach erklärt er:

1. Spielart (Farbe oder Grand; beim Null-Charakter Null-Varianten).
2. Risikostufe: *Normal*, *Hand* (Skat bleibt liegen), *Schneider angesagt*, *Schwarz angesagt*, *Ouvert* (offen spielen). Seit ADR 0013 (Spieltest vom 01.11.2026, gegen die Skatordnung) gehen Schneider, Schwarz und Ouvert auch nach dem Aufnehmen: Im Farbspiel und Grand stehen dann *Normal*, *Schneider angesagt*, *Schwarz angesagt* und *Ouvert* zur Wahl, beim Null *Normal* und *Ouvert*. Die Stufe *Hand* zählt nur, wenn aus der Hand gespielt wird; eine Ansage nach dem Skat bringt ihre eigenen Stufen ohne Hand (Grand mit 1, Schneider angesagt und erreicht: 4 Stufen statt 5 aus der Hand). Im Kern: `Declaration::with_hand`, im JSON `after_skat: true`.

Jede Stufe zählt +1, aber ein Fehlschlag auf einer angesagten Stufe gilt als verloren. Das ersetzt den Push-your-luck-Charakter des Reizens durch eine einzige, klare Entscheidung.

**Einmischen statt Passen** (Spieltest vom 01.11.2026, Entscheidung des Nutzers): Gefällt dem Spieler sein Blatt nicht, mischt er es vor der Ansage ein: Alle Hände und der Skat werden neu gegeben, das kostet kein Spiel der Serie. Bis zu zweimal pro Spiel (`redeals` in `run.ron`), danach muss eines der Blätter gespielt werden. Nach dem Aufnehmen des Skats und im Ramsch geht es nicht mehr. Einpassen (ein Spiel auslassen, das ein Spiel vom Limit kostete) gibt es nicht mehr. Einzelheiten in 5.1.

**Verlorene Spiele zählen einfach** (Entscheidung des Nutzers vom 04.11.2026, gegen die Skatordnung). Bis dahin wurde wie im echten Skat ein verlorenes Spiel mit dem doppelten Spielwert negativ gewertet; nur der Ramscher (11.1) und früher der Magenbitter machten es einfach. Seit dem 04.11.2026 geht ein verlorenes Spiel mit **−1 × Spielwert** vom Konto der Kneipe ab, für jede Spielart (Farbe, Grand, Null, Ramsch, mit Kontra und Re), jeden Charakter, jede Kneipe (Boss-Kneipe und Spelunke eingeschlossen). Im Kern: Die reine Regelfunktion `rules::game_value` bleibt bei der Skatordnung (`score` = −2 × Spielwert, geprüft mit den Golden-Vektoren in `core/tests/game_value_vectors.rs`); die Wertung des Runs (`content::score_parts`, `ScoreBreakdown::score`) rechnet mit `RUN_LOSS_FACTOR = 1`. Die Wirkungen `SingleLoss` (Boss-Regel, Schnaps) ruhen, sie ändern nichts mehr und erzeugen keine Zeile. Der Bot rechnet seine Erwartungswerte (`suit_expected_score`, `null_expected_score`) mit demselben Faktor. Die Abrechnung sagt bei einem verlorenen Spiel „Verloren zählt einfach“. Der Risiko-Hebel bleibt das verlorene Spiel selbst; ob die Zielwerte danach zu leicht sind, wird gemessen, sobald die Simulationen wieder laufen (17).

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
- **Entschieden heißt:** (1) Kein Hook, der während des Spiels feuert (`OnCardPlayed`, `OnTrickEnd`, z. B. Schenken, Revolutionär, Auf dem Trockenen), kann noch feuern. (2) Das Ergebnis nach den Regeln (gewonnen, Schneider und Schwarz beider Seiten, Grundwert, Stufen, Spielwert) ist für jede Augen- und Stichzahl gleich, die die Reststiche noch bringen können, mit den geltenden Schwellen (Gewinnschwelle, „Jungfrau“). Null: Der Alleinspieler bekommt in keiner Fortsetzung einen Stich (gewonnen, mit gleichem Stilbonus: Siebenen, nie bedient, Kurzschluss) oder in jeder (verloren). (3) Jeder `OnScore`-Hook feuert in allen Fortsetzungen oder in keiner, auch die des letzten Stichs (Spitze, Glückspfennig, Schlusslicht). Ramsch wird nie abgekürzt.
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
- **Verfehlt** (Zielwert nach dem dritten Spiel nicht erreicht): **seit ADR 0019 (04.11.2026, Entscheidung des Nutzers) ist der Run verloren** (`PubLost`, `RunLost`), in jeder Kneipe, auch einer Spelunke. Hooks nach einer gewonnenen Kneipe (`OnSeriesWon`) feuern nicht. Nur der **Schutzengel** (8.5, `SecondChance`) lässt eine verlorene Kneipe einmal im Run von vorn spielen (Ereignis `SecondChance`, dann `SeriesStarted` derselben Kneipe): dieselbe Kneipe oder Spelunke mit ihrem Malus, derselbe Boss (nicht neu gezogen), frisches Konto, neue Karten, der Zielwert ohne die unterwegs geplanten Änderungen (Wegbier); die Hausregeln des Abends bleiben; der Engel ist danach weg. Bis ADR 0019 kostete eine verfehlte Kneipe ein Leben, und es ging ohne Belohnung zur Routenwahl.
- **Leben** (bis ADR 0019): Start 2, höchstens 4, eine geschaffte Boss-Kneipe +1 (03.11.2026; ADR 0016 setzte Start 3, höchstens 5). Entfallen am 04.11.2026 mit Rast, Konterbier und den Leben aus Begegnungen; alte Spielstände lesen `lives` und übergehen es (ADR 0019). Die Web-App zeigt keine Herzen mehr.
- **Boss-Kneipe (ADR 0017, Entscheidung des Nutzers vom 03.11.2026)**: letzte Kneipe eines Stadtteils. Am Tisch sitzen **zwei Boss-Spieler**, die beiden Figuren des Bosses (links `BOSS_SEAT`, `Owner::Boss`; rechts `PARTNER_SEAT`, `Owner::BossPartner`), jede mit eigenem Namen, eigener Ausgabe und der Rückseite des Bosses (11). Der Boss wird beim Start des Stadtteils gezogen und ist ab dann sichtbar. **Alle drei Spiele werden gespielt**, der Zielwert zählt nach dem dritten (kein frühes Ende). **Verloren ist der Run vorbei** (`RunLost` mit `boss`); seit ADR 0019 lässt der Schutzengel auch die Boss-Kneipe einmal neu spielen (bis dahin half er hier nicht). **Gewonnen**: doppelte Auszahlung (`Payout::boss`), gerechnet, als blieben zwei Spiele übrig (`boss_reward.games_left_as` in `run.ron`, seit 03.11.2026; vorher fiel der Bonus je übrigem Spiel weg, und eine gewonnene Boss-Kneipe zahlte mit 6,45 Groschen weniger als eine früh geschaffte Kneipe mit 7,93, `docs/balance/2026-11-03-kleine-runde.md`), bis ADR 0019 ein Leben, und ein Relikt nach Wahl aus drei der Seltenheit Ungewöhnlich oder höher (`RunPhase::BossReward`, `RunAction::ChooseBossRelic`, `boss_reward` in `run.ron`); bei vollen Plätzen (seit 05.11.2026 acht, 8.0; vom 03.11. bis 05.11.2026 unbegrenzt) ersetzt es ein eigenes Relikt, das für den halben Preis verkauft wird (`replace`, `BossRewardView::slots_full`). Der Spieler darf auch verzichten. Im letzten Stadtteil sitzt der Endboss „Der Alte“ mit „Der Alten“; ihn zu schlagen gewinnt den Run (ohne Wahl). Bis ADR 0017 saß ein Boss links und ein Stammgast rechts, eine verlorene Boss-Kneipe kostete ein Leben und wurde sofort wiederholt.
- **Kartenbelohnung (Entscheidung des Nutzers vom 04.11.2026, Cockpit q24; feste Paare seit dem Playtest vom 05.11.2026, Entscheidung des Nutzers, ersetzt Variante A)**: Nach jeder gewonnenen **normalen Kneipe und Spelunke** (nicht nach der Boss-Kneipe, die gibt ihre Relikt-Wahl) bietet der Wirt **drei Angebote** an, jedes eine **zufällige Karte des Decks mit einer Zinke** (`RewardOffer { card, card_mod, value }`); eines nimmt der Spieler **gratis** (`RunPhase::CardReward`, `RunAction::TakeCardReward { offer }`). Trägt die Karte dieselbe Zinke, steigt ihr Wert (bis zum Höchstwert), eine andere wird ersetzt, wie in der Auslage. **Überspringen** gibt 2 Groschen (`RunAction::SkipCardReward`, `card_reward.skip_coins`). Die Wahl kommt nach dem Schnaps der Kneipe und vor der Beute einer Spelunke und der Routenwahl. Wie gezogen wird, steht in 8.6. Oberfläche: nach dem Ergebnis der Kneipe ein Blatt mit den drei Karten, jede so gezeichnet, wie sie mit der Zinke aussieht, mit Name, „auf …“ (dazu „steigert …“, „ersetzt …“ oder „… bleibt“) und Text; Antippen öffnet „Zinke anbringen?“ mit der Karte jetzt und danach („Anbringen“, „Anderes Angebot“), keine Kartenwahl mehr. Bis 05.11.2026 (Variante A): drei Zinken, die Karte wählte der Spieler. Zahlen **vorläufig**.
- **Hausregeln des Stammtisch-Streits** gelten bis zum Ende des Stadtteils (bisher des Abschnitts).
- **Einmischen, Schnäpse, Relikte „einmal pro Serie“** zählen je Kneipe.
- **Kneipe verlassen** (seit 03.11.2026, 8.7): Zwischen zwei Spielen, ab dem zweiten, kann der Spieler eine Kneipe aufgeben. Sie zählt als verloren (keine Auszahlung; seit ADR 0019 ist der Run vorbei, außer der Schutzengel lässt sie neu spielen). Vor dem letzten Spiel zeigt die Kopfzeile, was noch nötig ist.

#### 5.0.1 Routenwahl und Stationen

Nach jeder Kneipe, die den Run nicht beendet, wählt der Spieler einen von **drei Wegen** zur nächsten Kneipe (`RunPhase::Route`, `RunAction::ChooseRoute`). Vorher sichtbar: die nächste Kneipe mit Namen, Stadtteil, Zielwert, Spielen und den Stammgästen am Tisch (in der Boss-Kneipe der Boss, sobald bekannt).

- **Verkehrsmittel**: Zu Fuß, Bahn, Fahrrad. Seit der Nachentscheidung des Nutzers vom 02.11.2026 **verhalten sich alle drei gleich**: Jeder Weg führt über genau eine Station, kostet nichts extra, und die Kategorie der Station wird **je Weg gewürfelt** und angezeigt. Das Verkehrsmittel ist Darstellung (Weg auf der Karte, eine dekorative Fahrzeit `minutes`). Der erste Entwurf (Bahn kostet 1 Groschen und zeigt das genaue Ereignis, Fahrrad zwei Stationen mit einer Überraschung) ist verworfen.
- **Stationen** (`StopCategory`): **Tresen** (der Laden wie bisher), **Begegnung** (die Ereignisse aus 13), **Upgrade** („Werkstatt“, selten: seit den Zinken am Tresen (8.6) der **Kartenmaler gratis** – eine gewählte Karte, dann eine von zwei gezogenen Zinken – oder **Karte reinigen** gratis; dazu der Kartengeber, der Gasthof zur Krone und der Taschenspieler, die Karten aufwerten), **Rast** (Imbissbude; seit ADR 0019: Currywurst für ein Einmischen mehr in der nächsten Kneipe oder eine Karte sauber waschen wie „Karte reinigen“, immer gezogen; bis dahin ein Leben zurück, nur gezogen, solange ein Leben fehlte) und seit dem 03.11.2026 **Fundgroschen** (`FoundCoins`: 2 Groschen liegen auf dem Weg), **Kiosk** (`Kiosk`: ein kleiner Laden mit zwei Schnäpsen, je 1 Groschen billiger als am Tresen nach dem Rabatt des Wirts, mindestens 1; einer darf gekauft werden, Kaufen ist freiwillig; keine Relikte, keine Zinken, keine Neuauslage, kein Verkauf von Relikten, kein Anschreiben) und **Würfelbude** (`DiceStall`: 2, 4 oder 6 Groschen setzen, mit 50 % kommt der doppelte Einsatz zurück; wer keine 2 Groschen hat, darf nur zuschauen). Jedes Ereignis trägt seine Kategorie als Daten (`category` in `encounters.ron`); Fundgroschen und Würfelbude sind Ereignisse (`found_coins`, `dice_stall`), der Kiosk ist ein Laden (`Shop::kiosk`, `RunEvent::KioskOpened`).
- **Wege und Begegnungen v2 (Entscheidung des Nutzers vom 03.11.2026, Cockpit-Fragen 10 und 11)**: Die drei Wege einer Routenwahl zeigen immer **drei verschiedene Stationsarten**, eine davon der **Tresen** (auf einem zufälligen Weg). Die beiden anderen werden ohne Zurücklegen nach `route.weights` gezogen, vorläufig nach Augenmaß **Begegnung 35, Fundgroschen 30, Kiosk 15, Würfelbude 10, Werkstatt 7, Rast 5** (Rast bis ADR 0019 nur mit fehlendem Leben; der Tresen hat kein Gewicht mehr, bis dahin Tresen 40, Begegnung 35, Upgrade 12, Rast 8 mit „mindestens ein Tresen“). Gezogen wird nur, was gerade etwas zu bieten hat: eine Kategorie mit einem Ereignis, von dem eine Option genommen werden kann, der Kiosk mit einem Schnaps für den Charakter. Die Spelunke bleibt, wie sie ist. Das genaue Ereignis ist beim Wählen gezogen, aber verdeckt; die Kategorie ist sichtbar.
- **Ein Ereignis ohne Angebot** (keine Option, die gerade genommen werden kann) wird nicht auf einen Weg gelegt; geht eines unterwegs doch leer aus, geht es direkt in die Kneipe.
- **Zufall**: eigener Generator des Runs für Wege, Spelunken und Mali (`tour_rng`); die Karten der Spiele ändern sich dadurch nicht.

#### 5.0.2 Die Spelunke

Mit der Wahrscheinlichkeit `spelunke.chance` (vorläufig 0,3) führt **einer** der drei Wege statt zur nächsten Kneipe in eine **Spelunke**, nie vor einer Boss-Kneipe. Die Spelunke ist **ein eigener Ort**: eigener Name aus eigener Wortliste (`city.spelunken` in `tour.ron`), eigener Knoten (`d0-p2-s`); sie steht in der Reihenfolge des Stadtteils für die Kneipe, die sie ersetzt (diese gilt als übergangen, `PubState::Skipped`).

- **Zielwert**: der der ersetzten Kneipe × `spelunke.target_factor` (seit 03.11.2026 vorläufig 1,1, vorher 1,25).
- **Malus**: einer, zufällig nach Gewicht, vor der Wahl angezeigt (Antwort-Vorschlag der offenen Cockpit-Frage „ein zufälliger Malus, vorher sichtbar“, umgesetzt als Voreinstellung). Mali sind Daten (`maluses` in `tour.ron`): **Kein Einmischen** (`no_redeal`), **Klebrige Karten** (zwei zufällige Karten der Ausgabe ohne Modifikation sind ramponiert, Augen −5, nur für diese Kneipe, ohne die Groschen eines Fluchs; `curse`), **Kein Skat** (jedes Spiel Hand, `passive: [RequireHand]`, nicht für den Nullspieler) und **Schnapsverbot** (`lock_schnapps`, Fehler `ConsumablesLocked`). Tischregeln wirken als Quelle `Source::Malus` wie eine Boss-Regel.
- **Belohnung**: die normale Auszahlung, dazu die **Beute**: ein Relikt, gezogen beim Gewinnen, zu den eigenen oder für den halben Preis in Groschen (Ereignis `spelunke_loot`; die Wahl bleibt, auch wenn seit dem 03.11.2026 immer Platz ist). Verloren beendet sie den Run wie jede Kneipe (ADR 0019; bis dahin ein Leben).

#### 5.0.3 Die Stadt und ihre Namen

Stadtteile und Kneipen heißen nach einem **Namensgenerator** mit deutschen Wortlisten in `tour.ron` („Zum Goldenen Hirschen“, „Eckkneipe bei Uschi“, „Bahnhofsklause“, Boss-Kneipen wie „Vereinsheim Gut Blatt“), je Run aus dem Seed, ohne einen anderen Zufallsstrom zu berühren. Namen sind Anzeige. Die **Kartengeometrie** (Straßen, Lage) gehört der Web-App: Die Sicht liefert stabile Knoten-Ids (`d<Stadtteil>-p<Kneipe>`, Spelunke mit `-s`, Wege `…-r<n>`), je Stadtteil und für die Stadt einen Seed (`map_seed`) und den ganzen Plan des Abends (`TourView`: Stadtteile, Kneipen mit Zustand gewonnen, verloren, aktuell, kommend, übergangen, Spelunken, Boss des laufenden Stadtteils), damit die Web-App eine Übersicht und später eine Pixel-Stadtkarte zeichnen kann.

#### 5.0.4 Zahlen der Kneipentour

Gesetzt am 02.11.2026 nach Augenmaß; **Zielwerte, Leben (seit ADR 0019 entfallen), Groschen je übrigem Spiel, Boss-Faktoren und Malus-Gewichte gemessen am 03.11.2026** gegen den PIMC-Spieler (`--bot pimc:12:3`, Alleinspieler). **Zielwerte und Spelunken-Faktor ohne Leben neu gesetzt am 04.11.2026** (Runde 1 von 3, `docs/balance/2026-11-04-runde1.md`; Runde 2 mit dem Zielbild „Mischung“ und einem Einmischen mehr in Stadtteil 1: `docs/balance/2026-11-04-runde2.md`). Kleine Runde (Ist-Stand, 3 × 500 Runs): `docs/balance/2026-11-03-kleine-runde.md`; Piloten und große Runde (3 × 2000 Runs, Seeds 42 bis 44): `docs/balance/2026-11-03-grosse-runde.md`.

**Zielbild (03.11.2026)**: Der PIMC-Spieler erreicht Stadtteil 2 in 35 bis 40 % der Runs und Stadtteil 3 in 12 bis 18 %, gewinnt 3 bis 5 % der Abende, und ein Viertel bis ein Drittel der verlorenen Runs endet in normalen Kneipen (Leben aus), der Rest beim Boss. **Seit ADR 0019 (keine Leben)** gilt der Teil „verlorene Runs in normalen Kneipen“ nicht mehr. Vorschlag aus Runde 1 (04.11.2026, Augenmaß, nicht entschieden): Stadtteil 2 / 3 / gewonnen wie oben; von den verlorenen Runs enden 55 bis 70 % in normalen Kneipen, 25 bis 40 % beim Boss, höchstens 10 % in einer Spelunke; höchstens ein Viertel aller Runs endet in den ersten beiden Kneipen.

**Zielbild „Mischung“ (Entscheidung des Nutzers, 04.11.2026)**, je Charakter gegen den PIMC-Spieler: Stadtteil 2 in 35 bis 40 %, Stadtteil 3 in 12 bis 18 %, Run gewonnen 3 bis 5 %. Höchstens 20 % aller Runs enden in den ersten beiden Kneipen, vor allem über **ein Einmischen mehr in Stadtteil 1** (`district_redeals: [1]` in `run.ron`, 5.1), nicht über noch niedrigere Zielwerte dort. Von den verlorenen Runs enden 55 bis 65 % in normalen Kneipen, 30 bis 40 % beim Boss, höchstens 10 % in einer Spelunke.

**Runde 5** (05.11.2026, `docs/balance/2026-11-05-runde5.md`; acht Relikt-Plätze, Kartenbelohnung als feste Paare, Auslage mit drei Angeboten, einmal Neu auslosen je Besuch, Spezialkarten ohne Grenze). Mit acht Plätzen hielt Stadtteil 3 nur noch 5 bis 14 % (gewonnen 0,9 bis 2,4 %): Die Kurven von Stadtteil 3 und der Alte × 1,5 waren für 20 bis 36 Relikte gesetzt. Danach Stadtteil 3 rund − 20 bis − 35 %, der Alte × 1,2, Spelunke × 0,85, Nullspieler und Bruderherz in Stadtteil 2 schwerer (gemessen, Alleinspieler 3 × 500, andere 3 × 300):

| Charakter | Stadtteil 2 | Stadtteil 3 | gewonnen | Ende 1-1/1-2 | verlorene Runs normal / Sp. / Boss |
|---|---|---|---|---|---|
| Alleinspieler | 39,9 % | 13,7 % | 3,5 % | 19,4 % | 59,9 / 8,6 / 31,4 % |
| Nullspieler | 37,4 % | 17,3 % | 4,8 % | 18,7 % | 60,0 / 9,2 / 30,8 % |
| Drückeberger | 38,7 % | 15,8 % | 3,6 % | 19,9 % | 57,8 / 9,2 / 32,9 % |
| Schneiderin | 36,9 % | 16,0 % | 3,4 % | 19,6 % | 60,2 / 9,1 / 30,7 % |
| Bruderherz | 38,8 % | 17,2 % | 3,1 % | 20,2 % | 58,5 / 9,9 / 31,7 % |

**Runde 3** (05.11.2026, `docs/balance/2026-11-05-runde3.md`; Kartenbelohnung, Grand 18, Einmischen des Bots nach optimalem Stoppen, Kurve vor SR3 in 5.0.4 unten; SR3 Alleinspieler 3 × 500, andere je 3 × 300):

| Charakter | Stadtteil 2 | Stadtteil 3 | gewonnen | Ende 1-1/1-2 | verlorene Runs normal / Sp. / Boss |
|---|---|---|---|---|---|
| Alleinspieler | 39,8 % | 15,7 % | 6,3 % | 19,9 % | 57,2 / 9,6 / 33,2 % |
| Nullspieler | 47,9 % | 18,9 % | 8,6 % | 16,3 % | 58,0 / 11,3 / 30,7 % |
| Drückeberger | 42,0 % | 18,9 % | 11,3 % | 19,3 % | 56,5 / 11,3 / 32,2 % |
| Schneiderin | 48,4 % | 20,2 % | 7,9 % | 16,0 % | 56,2 / 12,7 / 31,1 % |
| Bruderherz | 35,2 % | 10,8 % | 3,1 % | 21,2 % | 62,4 / 10,2 / 27,4 % |

Danach (Phase C, ungemessen bis SR4): Stadtteil 3 und der Alte schwerer für alle, Nullspieler, Drückeberger und Schneiderin deutlich schwerer, Bruderherz in Stadtteil 2 leichter, Ramscher leichter, Zwillinge und Schieber schwerer, Spelunke × 0,95.

**Runde 2** (04.11.2026, Alleinspieler, 3 × 500 Runs, Seeds 42 bis 44; Einmischen + 1 in Stadtteil 1, Kurve unten, Spelunke × 1,0, der Alte × 1,3):

| Kennzahl | vorher (Kurve aus Runde 1) | nachher | Ziel |
|---|---|---|---|
| Stadtteil 2 erreicht | 33,6 % | 34,7 % | 35 bis 40 % |
| Stadtteil 3 erreicht | 11,4 % | 11,9 % | 12 bis 18 % |
| Run gewonnen | 4,8 % | 3,5 % | 3 bis 5 % |
| Ende in Kneipe 1-1 oder 1-2 | 30,5 % | 27,9 % | höchstens 20 % |
| verlorene Runs: normal / Spelunke / Boss | 71,8 / 7,2 / 21,0 % | 67,0 / 7,5 / 25,5 % | 55–65 / ≤ 10 / 30–40 % |
| gehalten Stadtteil 1 / 2 / 3 | 34 / 34 / 42 % | 35 / 34 / 30 % | rund 37 / 40 / 27 % |
| Boss-Kneipen gewonnen (Stadtteil 1 / 2 / 3) | 69 / 68 / 72 % | 65 / 64 / 67 % | |

Das Einmischen mehr hebt die erste Kneipe nur um 1 bis 3 Punkte (83,9 statt 82,5 %): Der Bot mischt nur Blätter ein, die er für schwach hält, und nutzt das dritte in rund jedem zehnten Spiel. Eine Kneipe ohne Relikte kommt auch mit Zielwert 1 nicht über 92 bis 93 %; 20 % in den ersten beiden Kneipen bräuchten dort rund 90 % je Kneipe. Das Ziel ist mit Daten allein nicht erreichbar (Cockpit-Frage, 17).

**Runde 1 ohne Leben** (04.11.2026, Alleinspieler, 3 × 300 Runs, Seeds 42 bis 44; „verloren zählt einfach“, Faktor-Relikte):

| Kennzahl | Kurve vom 03.11. ohne Leben | neue Kurve | Ziel |
|---|---|---|---|
| Stadtteil 2 erreicht | 29,0 % | 33,4 % | 35 bis 40 % |
| Stadtteil 3 erreicht | 5,9 % | 12,2 % | 12 bis 18 % |
| Run gewonnen | 1,9 % | 5,1 % | 3 bis 5 % |
| Ende normale Kneipe / Spelunke / Boss (alle Runs) | 72,6 / 9,2 / 16,3 % | 65,7 / 7,6 / 21,7 % | |
| Ende in Kneipe 1-1 oder 1-2 | 34,4 % | 29,6 % | höchstens 25 % |
| normale Kneipen gewonnen | 71 bis 85 % | 81 bis 88 % | |
| Spelunken gewonnen (Stadtteil 2) | 59 % | 77 % | |
| Boss-Kneipen gewonnen (Stadtteil 1 / 2 / 3) | 67 / 65 / 77 % | 67 / 70 / 73 % | |

Die Erfolgskurve einer Kneipe ohne Relikte ist mit einfacher Wertung steiler (rund 0,45 Punkte je Zielwert-Punkt zwischen 45 und 120), ihre Obergrenze bei Zielwert 1 liegt bei 88 bis 92 %. Offen für Runde 2: Stadtteil 1 hält mit 33 % noch zu wenig, Stadtteil 3 mit 42 % zu viel (17).

Bis Runde 1 (03.11.2026, mit zwei Startleben):

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
| Zielwerte Alleinspieler (Runde 5, 05.11.2026) | **15 / 30 / 45 / 50 / Boss 85 · 55 / 65 / 75 / 85 / Boss 95 · 115 / 130 / 145 / 160 / Boss 115** (der Alte × 1,2: 138); mit acht Relikt-Plätzen gemessen 39,9 / 13,7 / 3,5 % (`docs/balance/2026-11-05-runde5.md`; mit Stadtteil 3 = 160 / 180 / 200 / 215 / Boss 140 und dem Alten × 1,5 39,9 / 13,4 / 1,5 %). Runde 3: Stadtteil 3 160 / 180 / 200 / 215 / Boss 140 (der Alte × 1,5: 210). Gemessen in SR3 mit Stadtteil 3 = 145 / 165 / 185 / 200 und dem Alten × 1,3: 39,8 / 15,7 / 6,3 %, 19,9 % in den ersten beiden Kneipen, verlorene Runs 57,2 / 9,6 / 33,2 % (`docs/balance/2026-11-05-runde3.md`); Stadtteil 3 + 15 und der Alte × 1,5 danach, ungemessen. Vor SR3 alle Kurven verschoben: 1-1 − 20 (höchstens bis 10), 1-2 − 15, 1-3, 1-4 und die normalen Kneipen in Stadtteil 2 − 5, alle Bosse + 10. Runde 2 (04.11.2026, gemessen): 35 / 45 / 50 / 55 / Boss 75 · 60 / 70 / 80 / 90 / Boss 85 · 145 / 165 / 185 / 200 / Boss 130. Runde 1: 35 / 45 / 55 / 60 / 60 · 70 / 80 / 90 / 100 / 80 · 135 / 155 / 175 / 190 / 130 (03.11.2026: 45 / 55 / 65 / 75 / 60 · 90 / 105 / 120 / 135 / 100 · 145 / 170 / 195 / 205 / 140; bis dahin 33 / 36 / 38 / 40 / 45 · 40 / 42 / 44 / 46 / 52 · 46 / 48 / 50 / 52 / 60) | `run.ron` |
| Zielwerte Nullspieler (seit 04.11.2026, Runde 2) | **Runde 6 (07.11.2026, nach den neuen Verteidigern und Relikten v4): 30 / 45 / 80 / 85 / Boss 120 · 95 / 110 / 120 / 130 / Boss 150 · 225 / 240 / 265 / 280 / Boss 195** (38,2 / 15,6 / 3,9 %, 5 × 300 Runs; vorher 38,4 / 17,2 / 5,8 %, `docs/balance/2026-11-07-runde6.md`). Runde 5 (05.11.2026, acht Plätze): 30 / 45 / 80 / 85 / Boss 120 · 90 / 105 / 115 / 125 / Boss 150 · 200 / 215 / 235 / 250 / Boss 175 (37,4 / 17,3 / 4,8 %; vorher 36,6 / 17,0 / 2,4 %). Nach SR4 (05.11.2026): 35 / 50 / 80 / 85 / Boss 120 · 85 / 100 / 110 / 120 / Boss 140 · 260 / 280 / 305 / 320 / Boss 185; SR4 mit Boss 2 = 125 und Stadtteil 3 = 210 / 230 / 255 / 270 / 155: 35,9 / 15,9 / 8,2 %, Stadtteil 3 hielt 52 %. Runde 3 (nach SR3): 35 / 50 / 80 / 85 / Boss 120 · 85 / 100 / 110 / 120 / Boss 125 · 210 / 230 / 255 / 270 / Boss 155; SR3 mit 20 / 35 / 50 / 55 / 85 · 70 / 85 / 95 / 105 / 110 · 175 / 195 / 220 / 235 / 155: 47,9 / 18,9 / 8,6 %. Runde 2: 40 / 50 / 55 / 60 / Boss 75 · 75 / 90 / 100 / 110 / Boss 100 · 175 / 195 / 220 / 235 / Boss 145; gemessen mit Stadtteil 3 = 165 / 185 / 210 / 225 (37,5 / 14,7 / 6,5 %), danach Stadtteil 3 + 10 ungemessen. Runde 1: 40 / 50 / 60 / 65 / 65 · 75 / 90 / 100 / 110 / 90 · 150 / 170 / 195 / 210 / 145 (33,4 / 14,6 / 7,3 %; 03.11.2026: 50 / 65 / 75 / 85 / 70 · 110 / 125 / 145 / 160 / 120 · 180 / 210 / 245 / 255 / 175, ohne Leben Stadtteil 2 in 26,9 %; bis dahin 18 bis 30) | `characters.ron` |
| Zielwerte Schneiderin (seit 04.11.2026, Runde 2, gemessen) | **Runde 6 (07.11.2026, nach den neuen Verteidigern und Relikten v4): 20 / 35 / 60 / 65 / Boss 100 · 60 / 75 / 85 / 95 / Boss 105 · 135 / 150 / 165 / 180 / Boss 120** (37,3 / 14,7 / 3,7 %, 5 × 300 Runs; vorher 34,7 / 15,2 / 4,4 %, `docs/balance/2026-11-07-runde6.md`). Runde 5 (05.11.2026, acht Plätze): 20 / 35 / 65 / 70 / Boss 110 · 60 / 75 / 85 / 95 / Boss 105 · 135 / 150 / 165 / 180 / Boss 120 (36,9 / 16,0 / 3,4 %; vorher 36,8 / 15,4 / 1,1 %). Runde 3 (05.11.2026, nach SR3): 25 / 40 / 65 / 70 / Boss 110 · 60 / 75 / 85 / 95 / Boss 105 · 180 / 200 / 225 / 240 / Boss 155; SR3 mit 15 / 30 / 45 / 50 / 85 · 60 / 75 / 85 / 95 / 105 · 160 / 180 / 205 / 220 / 155: 48,4 / 20,2 / 7,9 %. Runde 2: 35 / 45 / 50 / 55 / Boss 75 · 65 / 80 / 90 / 100 / Boss 95 · 160 / 180 / 205 / 220 / Boss 145 (38,3 / 14,7 / 4,8 %). Runde 1: 35 / 45 / 55 / 60 / 60 · 75 / 90 / 100 / 110 / 90 · 150 / 170 / 195 / 210 / 145 (36,7 / 12,3 / 5,4 %; 03.11.2026: 50 / 65 / 75 / 90 / 75 · 105 / 120 / 140 / 155 / 115 · 165 / 195 / 225 / 235 / 160, ohne Leben Stadtteil 2 in 22,4 %) | `characters.ron` |
| Zielwerte Bruderherz (seit 04.11.2026, Runde 2) | **Runde 6 (07.11.2026, nach den neuen Verteidigern und Relikten v4): 10 / 20 / 35 / 40 / Boss 75 · 40 / 50 / 55 / 65 / Boss 90 · 95 / 105 / 120 / 130 / Boss 100** (40,5 / 15,7 / 3,9 %, 5 × 300 Runs; vorher 39,2 / 14,0 / 2,3 %, `docs/balance/2026-11-07-runde6.md`). Runde 5 (05.11.2026, acht Plätze): 10 / 20 / 35 / 40 / Boss 75 · 45 / 55 / 60 / 70 / Boss 90 · 110 / 120 / 135 / 145 / Boss 105 (38,8 / 17,2 / 3,1 %; vorher 38,8 / 18,3 / 0,9 %). Runde 3 (05.11.2026, nach SR3): 10 / 20 / 35 / 40 / Boss 75 · 40 / 50 / 55 / 65 / Boss 80 · 140 / 155 / 175 / 185 / Boss 130; SR3 mit 10 / 20 / 40 / 45 / 75 · 50 / 60 / 65 / 75 / 90 · 150 / 165 / 185 / 195 / 130: 35,2 / 10,8 / 3,1 %. Runde 2: 25 / 35 / 45 / 50 / Boss 65 · 55 / 65 / 70 / 80 / Boss 80 · 150 / 165 / 185 / 195 / Boss 120; gemessen mit Stadtteil 3 = 135 / 150 / 170 / 180 (34,2 / 13,5 / 6,5 %), danach Stadtteil 3 + 15 ungemessen. Runde 1: 30 / 40 / 50 / 55 / 55 · 65 / 75 / 80 / 90 / 75 · 125 / 140 / 160 / 170 / 120 (30,1 / 9,9 / 4,7 %) | `characters.ron` |
| Zielwerte Drückeberger (seit 04.11.2026, Runde 2, gemessen) | **Runde 5 (05.11.2026, acht Plätze): 20 / 35 / 55 / 60 / Boss 100 · 70 / 80 / 90 / 100 / Boss 105 · 130 / 145 / 160 / 175 / Boss 125** (38,7 / 15,8 / 3,6 %; vorher 38,7 / 15,6 / 1,2 %). Nachtrag Runde 4 (05.11.2026): Stadtteil 3 205 / 225 / 245 / 260 / Boss 155 (3 × 300: 37,6 / 14,9 / 6,0 %; mit Stadtteil 3 aus Runde 3 37,8 / 14,8 / 6,4 %, beide mit der Spelunke ab fünf Relikten, `docs/balance/2026-11-05-runde4.md`). Runde 3 (05.11.2026, nach SR3): Stadtteil 3 195 / 215 / 235 / 250 / Boss 140, in SR4 37,4 / 13,1 / 5,4 %; SR3 mit 20 / 35 / 50 / 55 / 90 · 65 / 75 / 85 / 95 / 100 · 155 / 175 / 195 / 210 / 140: 42,0 / 18,9 / 11,3 %. Runde 2: eigene: 40 / 50 / 55 / 60 / Boss 80 · 70 / 80 / 90 / 100 / Boss 90 · 155 / 175 / 195 / 210 / Boss 130 (38,2 / 15,0 / 5,7 %); mit der Kurve des Alleinspielers aus Runde 1 39,3 / 17,2 / 8,9 % | `characters.ron` |
| Einmischen je Spiel | 2, in Stadtteil 1 eins mehr (seit Runde 2, 04.11.2026, Entscheidung des Nutzers); der Nullspieler je eins mehr | `redeals`, `district_redeals` in `run.ron` |
| Boss-Faktoren | Runde 5 (05.11.2026, acht Plätze): der Alte 1,5 → 1,2 (mit 1,5 gewannen die Charaktere 22 bis 55 %, nachher 47 bis 58 %). Runde 3 (05.11.2026, nach SR3, ungemessen): Ramscher 1,0 → 0,7 (Nullspieler 2,0 → 1,6; in Stadtteil 1 45 bis 55 % gegen 64 bis 85 % bei den anderen), Zwillinge 1,2 → 1,35, Schieber 0,5 → 0,6, Schwarzer Peter beim Nullspieler 0,2 → 0,4 (84 %), der Alte 1,3 → 1,5 (75 %). Davor gemessen am 03.11.2026 (11.1): Alleinspieler in der großen Runde, Nullspieler in der Runde der Charaktere (Schieber 0,2, Ramscher 2,0, Oma 0,9, Peter 0,2) | `bosses.ron` |
| Kartenbelohnung (seit 04.11.2026, vorläufig) | nach jeder gewonnenen normalen Kneipe und Spelunke 1 aus 3 Angeboten (Karte mit Zinke, seit 05.11.2026; vorher Zinke, Karte nach Wahl) gratis, Seltenheit gewöhnlich 60, ungewöhnlich 30, selten 10, Marken 50 %, Spezialkarte 5 % je Angebot, Zinken der eigenen Bau-Richtung 150 % (seit 05.11.2026); Überspringen 2 Groschen | `card_reward` |
| Boss-Belohnung | Auszahlung wie mit zwei übrigen Spielen, × 2 (im Mittel 14,6 Groschen), 1 aus 3 Relikten ab Ungewöhnlich (bis ADR 0019 auch +1 Leben) | `economy`, `boss_reward` |
| Gegner je Stadtteil | Heuristik, PIMC 8:3, PIMC 12:3 (die alten Abschnitte 1, 3, 4) | `opponents` |
| Leben | entfallen seit ADR 0019 (bis dahin Start 2, höchstens 4, Boss +1; bis 03.11.2026 3, 5, +1) | – |
| Grundbetrag, je übrigem Spiel | 3, 2 (03.11.2026; ADR 0016 3, davor 1) | `economy` |
| Gewichte der Stationen | seit 03.11.2026 drei verschiedene Arten je Routenwahl, eine der Tresen; die anderen ohne Zurücklegen: Begegnung 35, Fundgroschen 30, Kiosk 15, Würfelbude 10, Werkstatt 7, Rast 5 (bis ADR 0019 nur mit fehlendem Leben), vorläufig. Bis dahin Tresen 40, Begegnung 35, Upgrade 12, Rast 8 | `route.weights`, `route.kiosk` |
| Spelunke | Chance 0,3, Zielwert × 0,85 (seit Runde 5, 05.11.2026: mit acht Plätzen und × 0,9 endeten 9,9 bis 10,5 % der verlorenen Runs in Spelunken, nachher 8,6 bis 9,9 %). Davor × 0,9 (seit SR4, 05.11.2026: 8,6 bis 13,2 % der verlorenen Runs endeten mit × 0,95 in Spelunken, Drückeberger 13,2 %; `docs/balance/2026-11-05-runde4.md`). Davor × 0,95 (Runde 3, 05.11.2026: in SR3 74 bis 82 % gewonnen gegen 85 bis 90 % in normalen Kneipen, 9,6 bis 12,7 % der verlorenen Runs). Davor × 1,0 (seit Runde 2, 04.11.2026: mit × 0,9 gewann der Bot sie so oft wie normale Kneipen, 76 bis 80 % in Stadtteil 2, und nur 7 % der verlorenen Runs endeten dort; nachher 69 bis 84 %, 7,5 %). Davor × 0,9 (Runde 1, 04.11.2026: ohne Leben mit × 1,1 nur 59 % gewonnen gegen 71 bis 83 % in normalen Kneipen, jeder vierte Run in Stadtteil 2 endete dort; nachher 77 %; bis 03.11.2026 1,25), ein Malus (Gewicht 2, Kein Skat 1, seit 03.11.2026: Kein Skat war 13 bis 23 Punkte schwerer als die anderen) | `spelunke`, `tour.ron` |
| Rast | seit ADR 0019: Currywurst (ein Einmischen mehr in der nächsten Kneipe) oder eine Karte waschen; bis dahin +1 Leben | `encounters.ron` (`snack_bar`) |
| Bot-Route | Tresen ab 8 Groschen, Upgrade mit verfluchter Karte, Rast 2 (3 mit verfluchter Karte), Fundgroschen 2, Kiosk 1, Würfelbude 0; Spelunke seit 05.11.2026 (Nutzer q29 A) nur mit fünf Relikten oder vier und dem Schutzengel (seit ADR 0019 vier oder drei, bis dahin 3 Leben und 3 Relikte) | `RunBot::route_action` |
| Bot-Einmischen (seit 05.11.2026, Messwerkzeug) | schwache Blätter wie bisher (`RunBot::is_weak`, auch mit Schnaps); dazu ein mittleres Blatt mit einem eigenen Einmischen des Spiels, wenn sein Wert mit zehn Karten (`ten_card_score`) unter der Schwelle des optimalen Stoppens für die übrigen Einmischen liegt (`REDEAL_STOP`: 8,8 / 17,6 / 23,6 Punkte bei 1 / 2 / 3). Erste Kneipe (Kurve, 1000 Kneipen) bei Zielwert 35 / 55 / 90: 83,2 / 73,4 / 56,7 → 85,3 / 77,4 / 62,9 %; eine Schwelle nach dem Zielwert der Kneipe brachte nichts | `RunBot::redeal_threshold` |
| Bot-Bau-Richtung (seit 05.11.2026, Messwerkzeug, vorläufig) | Zinken mit einer Richtung (`tags`) eines gehaltenen Relikts zählen dreifach (+ 200 %, gewinnen Gleichstand) in Kartenbelohnung, Auslage und Kartenmaler; ein Relikt-Angebot mit einer Richtung der gehaltenen Relikte oder der Zinken im Deck + 1 Wert (Tresen, Boss-Belohnung). Mit + 50 % hätte Blei (1) nie Rang +1 (3) geschlagen | `ZINKE_DIRECTION_BONUS_PERCENT`, `RELIC_DIRECTION_BONUS` |
| Bot-Ereignis (seit 03.11.2026) | eine Option muss genommen werden: die mit dem höchsten Wert in groben Groschen (`RunBot::option_value`, nicht abgestimmt); am Kiosk der wertvollste Schnaps | `RunBot::encounter_action` |
| Bot-Boss-Relikt und Tausch | das wertvollste in einen freien Platz; bei vollen Plätzen (acht seit 05.11.2026) statt des schwächsten (`relic_value`), wenn es mindestens `SWAP_MARGIN` (3) mehr wert ist, sonst keins; am Tresen genauso mit `ReplaceRelic` (das wertvollste bezahlbare Angebot, der Erlös zählt), in der Beute der Spelunke sonst die Groschen | `RunBot::boss_reward_action`, `shop_action`, `option_value` |

Gemessen am 03.11.2026 für den Alleinspieler (große Runde) und für die anderen vier Charaktere (je 3 × 300 Runs, `docs/balance/2026-11-03-charaktere.md`, Tabelle in 9). Noch zu messen: die Gewichte der Stationen und die Spelunke selbst (mit der steilen Kurve in Stadtteil 3 nur noch 20 bis 36 % gewonnen, 17).

### 5.1 Die Serie (seit ADR 0016: die Kneipe)

Gilt für die Kneipe weiter: Zielwert, drei Spiele, Einmischen, Auszahlung. Überholt: „Wer das Limit erreicht, ohne den Zielwert zu schaffen, verliert den Run“ (seit ADR 0016 kostet es ein Leben, 5.0).


- Jede Serie hat einen **Zielwert** (siehe 5.2) und ein **Spiellimit**: seit dem 01.11.2026 **3 Spiele** gegen dieselben zwei Stammgäste („3 Spiele bilden eine Serie, 2 Serien münden in eine Boss-Serie“, Entscheidung des Nutzers nach dem Spieltest). Vorher: M5 8 Spiele mit bis zu 6 Pässen, M2 5 und 3 (5.2).
- Gewonnene Spiele addieren ihren Spielwert. Verlorene Spiele ziehen seit dem 04.11.2026 den einfachen Spielwert ab (bis dahin den doppelten, 4.3).
- **Einmischen** (`RunAction::Redeal`): Vor der Ansage, solange der Skat unberührt ist, gibt der Spieler das Blatt zurück und es wird neu gegeben. Das kostet kein Spiel und keine Punkte, die Vorhand bleibt. Jedes Spiel erlaubt zwei (`redeals` in `run.ron`), in Stadtteil 1 drei (seit Runde 2, 04.11.2026, Entscheidung des Nutzers: `district_redeals` in `run.ron`, je Stadtteil ein Zuschlag, damit weniger Runs in den ersten beiden Kneipen enden; `Content::redeals_in`), danach muss eines der Blätter gespielt werden. Der Zaungast hat eins weniger (9.5), die Hausregel Schieben gibt eins mehr pro Spiel (8.1), Ein Bier und Kräuter geben eins für den Rest der Serie, das nach denen des Spiels verbraucht wird (8.3), „Neues Blatt“ mischt einmal, ohne eins zu verbrauchen, und der Schnaps „Reue“ nimmt das letzte Einmischen zurück. Die Kopfzeile zeigt, wie oft der Spieler im laufenden Spiel noch einmischen darf (eigene plus die der Serie, `SeriesView::redeals_left`). Der Ramsch des Ramschers wird nie eingemischt. Das Einmischen ersetzt das Einpassen, das ein Spiel vom Limit kostete: Wer seine Pässe verbraucht hatte, musste schlechte Blätter spielen und verlor faktisch den Run.
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

### 5.5 Die lange Nacht (ADR 0020, 05.11.2026)

**Entschieden vom Nutzer am 05.11.2026 (Cockpit q30 bis q33, jeweils Variante A).** Endlos-Modus nach dem Alten. Wachstum und Nachtschwärmer seit der Balancing-Runde vom 05.11.2026 gemessen (unten, `docs/balance/2026-11-05-nacht.md`); Preise, Bot-Werte und Verschärfungen weiter vorläufig.

- **Einstieg**: Wer den Alten schlägt, hat den Run gewonnen; er wird genau wie bisher gezählt (Statistik, Einsatz-Leiter, Freischaltungen). Der Endbildschirm bietet „Weiterziehen in die lange Nacht“ neben „Run beenden“ an (`RunAction::EnterNight`). Weiter geht es mit Stadtteil 4, 5, … mit demselben Deck, denselben Relikten, Groschen und Charakter, bis eine Kneipe verloren geht (der Schutzengel hilft einmal, wenn er noch da ist). Verlieren in der Nacht kostet nur den Rekord: `RunLost { night: true }` zählt keinen Run, der Sieg bleibt.
- **Zielwerte** (seit Runde 5, 05.11.2026, acht Relikt-Plätze): Stadtteil n der Nacht spielt die Zielwerte von Stadtteil 3 (Kneipen und Boss mit dem Faktor des gezogenen Bosses, eine Spelunke × 0,85) mal dem Wachstum aller Stadtteile der Nacht bis n, einmal gerundet. Wachstum je Stadtteil über den vorigen: **Stadtteil 4 × 0,6, Stadtteil 5 × 1,0, Stadtteil 6 × 1,05, Stadtteil 7 × 1,1, Stadtteil 8 × 1,25, ab Stadtteil 9 je × 1,6** (zusammen × 0,6 / 0,6 / 0,63 / 0,69 / 0,87 / 1,39 / 2,2 für Stadtteil 4 bis 10); für den Alleinspieler Stadtteil 4 69 / 78 / 87 / 96 und Boss 69 × Boss-Faktor. Die Nacht beginnt unter Stadtteil 3, weil der Build mit acht Plätzen kaum noch wächst und jeder Stadtteil einen Boss und eine Verschärfung bringt (unten). Der **Nullspieler** hat dasselbe Wachstum (Feld `night_growth` bleibt; bis Runde 5 × 1,3 / 3 / 4,5 / 8 / 2,5). Daten `night: (growth: 1.6, growth_per_district: [0.6, 1.0, 1.05, 1.1, 1.25, 1.6])` in `run.ron` (der letzte Eintrag gilt weiter, `growth` nur ohne Liste) und `night_growth` je Charakter in `characters.ron` (`CharacterDef::night`, `Run::night_config`). Von der Nacht-Runde bis Runde 5 × 1,5 / 5 / 8 / 11, dann × 2,5 (für die Relikt-Flut), davor × 1,6 je Stadtteil. Groschen, Kartenbelohnung, Schnaps und Gegner wie in Stadtteil 3; die Stadt bekommt neue Stadtteile aus dem Namensgenerator.
- **Verschärfungen („Die Sperrstunde rückt näher“)**: Jeder Stadtteil der Nacht bringt eine dauerhafte Verschärfung, gezogen mit dem Generator der Nacht, ohne Wiederholung, bis alle dran waren (dann beginnt die Liste von vorn und eine Regel zählt doppelt). Sie wird beim Öffnen der Routenwahl in den neuen Stadtteil gezogen und auf einem Startblatt angekündigt; sie gilt ab da (ein Tresen auf diesem Weg kostet schon mehr). Daten in `night.ron`:

| Verschärfung | Id | Regel |
|---|---|---|
| **Kurze Geduld** | `fewer_redeals` | ein Einmischen weniger je Spiel, nie unter 0 |
| **Hellwache Gegner** | `sharp_opponents` | die Gegner rechnen mit vier PIMC-Blättern mehr je Zug |
| **Doppelte Bosse** | `double_boss` | die Boss-Kneipe der Nacht legt dazu die Regel eines zweiten, zufälligen Bosses auf (nie den Alten; Ereignis `ExtraBossRules`) |
| **Nachtzuschlag** | `pricier_tresen` | am Tresen und am Kiosk kostet alles mit Preis 1 Groschen mehr |
| **Verhexte Karte** | `cursed_card` | zu Beginn jeder Kneipe trägt eine zufällige Karte des Blatts einen Fluch (Durchsichtig, Ramponiert, Fleckig), nur für diese Kneipe |
| **Spelunkenluft** | `dive_air` | jede normale Kneipe hat einen zufälligen Spelunken-Malus (eine Spelunke behält ihren, die Boss-Kneipe hat keinen) |
| **Zielwert-Sprung** | `target_jump` | alle Zielwerte der Nacht × 1,1, obendrauf |
| **Knausriger Wirt** | `stingy_host` | die Kartenbelohnung bietet ein Angebot weniger an (mindestens eines) |

- **Bosse der Nacht**: die fünf Bosse, der Alte und seit dem 10.11.2026 der Totengräber, der nur nachts kommt (`night_bosses` in `run.ron`, 11.1), zufällig, keiner doppelt, bis alle dran waren. Die Boss-Kneipe der Nacht gibt die doppelte Auszahlung und eine Relikt-Wahl aus den **Nachtschwärmern**, solange einer übrig ist (sonst die normale Wahl).
- **Nachtschwärmer** (Relikte nur der Nacht, `night: true`; auch am Tresen der Nacht; Bierdeckel: Faktor-Sechseck mit eigenem Motiv):

| Relikt | Id | Effekt | Seltenheit, Preis | Bot-Wert |
|---|---|---|---|---|
| **Nachteule** | `night_owl` | +×0,1 je Verschärfung auf jedes gewonnene Spiel, höchstens ×1,5 (bis 05.11.2026 +×0,15 ohne Grenze) | Legendary, 10 | 6 |
| **Kehraus** | `sweep_out` | ×2 auf ein gewonnenes drittes Spiel einer Kneipe | Rare, 8 | 3 |
| **Lumpensammler** | `rag_collector` | +×0,03 je Karte mit Zinke im Deck auf jedes gewonnene Spiel, höchstens ×2 (bis 05.11.2026 +×0,05) | Legendary, 10 | 6 |
| **Mondschein** | `moonlight` | jede gewonnene Kneipe +×0,1 auf jedes gewonnene Spiel, für den Rest des Runs, höchstens ×2 (bis 05.11.2026 +×0,2 ohne Grenze) | Rare, 9 | 6 |
| **Nachtschicht** | `night_shift` | +×0,05 je Karte mit einer Zinke deiner Richtung (einer Richtung deiner Relikte), höchstens ×2 (bis 05.11.2026 +×0,1) | Legendary, 10 | 5 |
| **Morgengrauen** | `dawn` | ×1,5 auf ein gewonnenes erstes Spiel einer Kneipe | Rare, 8 | 4 |

- **Rekord und Bestenliste**: Der Meta-Zustand hält je Charakter den tiefsten Stadtteil der Nacht, die meisten Kneipen einer Nacht und das beste Spiel (auch aus normalen Runs; `CharacterStats`). Die Startseite zeigt sie als „Bestenliste“. Der Rekord wird schon während der Nacht fortgeschrieben, auch wenn der Spieler die Nacht verlässt, ohne zu verlieren.
- **Freischaltungen** an den Meilensteinen (9.3): Stadtteil 5 „Tief in der Nacht“ (Rückseite Mondsichel), Stadtteil 7 „Nach der Sperrstunde“ (Rückseite Sternenhimmel und Kehraus für den normalen Run), Stadtteil 10 „Bis zum Morgengrauen“ (Rückseite Laterne und Morgengrauen). Rückseiten sind eine neue, kosmetische Belohnungsart (`Reward::CardBack`); verdiente wählt der Spieler im Menü („Rückseite“).
- **Auslegungen (Claude, 05.11.2026)**: Beim Einstieg in die Nacht gibt es keine Relikt-Wahl für den Alten (wie bisher). Ein Kehraus- oder Morgengrauen-Zähler bleibt bei einem Schutzengel stehen wie beim Rabattheft. Die Nachteule zählt alle Verschärfungen der Nacht, auch die vor ihrem Kauf (Zähler `verschaerfungen` des Runs). Spelunkenluft legt keinen Malus in eine Boss-Kneipe. Doppelte Bosse ziehen die weitere Regel beim Start des Stadtteils (sichtbar wie der Boss); unter dem Alten bleibt seine eigene Reihe.

**Plausibilität (05.11.2026, nicht abgestimmt)**: `skat_sim --seed 42 --runs 20 --night` (Heuristik): kein Run gewonnen, also keine Nacht. `--runs 1000 --night`: 30 gewonnene Runs, alle in der Nacht; 16 enden in Stadtteil 4, die anderen 14 laufen bis Stadtteil 11 bis 27 (Mittel 45,8 Kneipen je Nacht, Faktor des letzten gewonnenen Spiels im Mittel ×5973). Gewählt wurden vor allem Lumpensammler (9), Mondschein (8) und Nachteule (6). Linear wachsende Faktor-Relikte (Zapfhahn, Mondschein, Nachteule, Lumpensammler) multiplizieren sich zu einem Polynom hohen Grades, das 1,6 je Stadtteil erst nach rund 20 Stadtteilen einholt. Für die Balancing-Runde (17).

**Balancing-Runde (05.11.2026, `docs/balance/2026-11-05-nacht.md`)**: PIMC-Spieler (`--bot pimc:12:3 --night`), Alleinspieler 3 × 700 Runs (Seeds 42 bis 44, je 119 Nächte), Nullspieler 3 × 400 (71 Nächte). Ursache des Durchlaufens ist weniger das Polynom der Faktor-Relikte als die **Relikt-Flut**: Ein gewonnener Run hält rund 36 Relikte; in Stadtteil 4 und 5 kauft der Bot je rund zehn weitere, ab Stadtteil 6 ist der Pool (rund 62) leer. Der Spielwert eines gewonnenen Spiels steigt so von Stadtteil 4 auf 6 um das Siebzigfache (Median 326 → 22 676), danach nur noch um × 1,2 bis 1,5 je Stadtteil. Ein festes oder gleichmäßig steigendes Wachstum endet entweder alle Nächte nach der Flut an einer Wand (Stadtteil 8) oder lässt sie weit laufen; das Wachstum folgt deshalb der Flut (× 5, × 8, × 11) und wird danach flach (× 2,5), damit außergewöhnliche Builds einen Schwanz bis Stadtteil 10 haben. Dazu Obergrenzen für alle wachsenden Faktoren (8.10 und oben): **Zapfhahn und Heiße Hand höchstens × 3** (im normalen Run dieselben Runs und Siege, Punkte je Spiel 56,3 → 56,2 in einem von drei Seeds), die Nachtschwärmer kleiner und höchstens × 2 bzw. × 1,5 (Effekt-Feld `max` an `ScaleGameValuePerCounter` und `ScaleGameValuePerCard`).

| Ende der Nacht in Stadtteil | 4 | 5 | 6 | 7 | 8 bis 11 | ab 12 | Median | tiefster |
|---|---|---|---|---|---|---|---|---|
| Alleinspieler vorher (× 1,6) | 25,2 % | 3,4 % | 0,8 % | 0,0 % | 0,8 % | 69,7 % | 22 | 30 |
| Alleinspieler nachher | 24,4 % | 13,4 % | 11,8 % | 20,2 % | 30,3 % | 0,0 % | 7 | 10 |
| Nullspieler vorher (× 1,6) | 40,8 % | 11,3 % | 1,4 % | 0,0 % | 0,0 % | 46,5 % | 5 | 27 |
| Nullspieler nachher | 33,8 % | 18,3 % | 4,2 % | 16,9 % | 26,8 % | 0,0 % | 5 | 9 |

Alleinspieler nachher: Stadtteil 10 erreichen 3,4 % der Nächte; gewonnene Kneipen je Nacht 71,7 → 14,2; Faktor des letzten gewonnenen Spiels im Mittel × 7368 → × 63 (Median × 26). Die Verschärfung, unter der eine Nacht endet, ist ungefähr gleich verteilt (12 bis 20 je 119 Nächte); keine sticht heraus, ihre Stärke bleibt.

**Runde 5 (05.11.2026, acht Relikt-Plätze, `docs/balance/2026-11-05-runde5.md`)**: Ohne die Flut endeten mit dem Wachstum oben 88 % der Nächte in Stadtteil 4 (32 Nächte). Der Build wächst in der Nacht kaum (Median Grundwert 21 bis 24, Faktor × 1); eine Kneipe kommt nicht über rund 90 %, und mehr als die Hälfte der Nächte endet beim Boss (44 von 79, der Ramscher allein 20). Deshalb beginnt die Nacht bei × 0,6, bleibt bis Stadtteil 7 fast flach und wird ab Stadtteil 8 steil. Alleinspieler 3 × 700 (79 Nächte), Nullspieler 3 × 400 (63 Nächte):

| Ende der Nacht in Stadtteil | 4 | 5 bis 7 | 8 bis 11 | ab 12 | Median | tiefster |
|---|---|---|---|---|---|---|
| Alleinspieler mit dem alten Wachstum | 87,5 % | 12,5 % | 0,0 % | 0,0 % | 4 | 5 |
| Alleinspieler nachher | 43,0 % | 46,8 % | 10,1 % | 0,0 % | 5 | 11 |
| Nullspieler nachher | 25,4 % | 57,1 % | 17,5 % | 0,0 % | 5 | 9 |

Stadtteil 10 erreichen 2,5 % der Nächte des Alleinspielers. Der Anteil, der schon in Stadtteil 4 endet, hängt an den Bossen, kaum am Wachstum (× 1,0 statt × 0,6: 56 statt 43 %).

### 5.6 Die Wochen-Herausforderung (10.11.2026)

Aufbauend auf dem Tages-Seed und der Tafel des Tages (14.4): Jede ISO-Woche (Montag bis Sonntag, Woche 1 hält den ersten Donnerstag des Jahres) hat **einen Run für alle**, mit festem Seed und einer **Sonderregel der Woche**.

- **Woche**: ISO-Jahr und -Woche als Zahl `JJJJWW` (202646 ist Woche 46 von 2026, `WeekId`). Die App nimmt die Woche des lokalen Datums, der Server die der UTC-Uhr und lässt eine Woche Abstand zu (ein Run über Sonntag Mitternacht).
- **Seed**: folgt aus der Woche allein (`WeekId::seed`, gemischt, damit er nie wie ein Tages-Seed aussieht). Charakter und Einsatz wählt der Spieler wie beim Tages-Seed, außer die Regel legt den Charakter fest (dann Einsatz 0, auch wenn der Charakter noch gesperrt ist).
- **Sonderregel**: Daten in `core/content/weekly.ron`, etwa acht Regeln. Sie kommen in Runden: Jede Runde ist die ganze Liste in einer Reihenfolge, die aus der Nummer der Runde folgt (`pick_rule`); keine Regel kommt zweimal, bevor alle dran waren. Eine Regel baut nur aus vorhandenen Mechaniken (`WeeklyRuleDef`): fester Charakter (`character`), Startrelikte (`start_relics`), Startausgabe wie beim Charakter (`deck`, ein Startfluch), Groschen zum Start (`coins`), Verschärfungen der langen Nacht für den ganzen Run (`rules`, `NightRule`; `DoubleBoss` gilt nur nachts und ist ein Ladefehler) und Tischregeln wie bei einem Boss (`passive`, `hooks`, Quelle `Source::Weekly`, feuert direkt nach dem Charakter). Was sie zufällig zieht (Malus je Kneipe, Flüche für eine Kneipe), kommt aus einem eigenen Generator; alle anderen Ziehungen bleiben wie ohne Regel. In der Nacht stapeln sich die Verschärfungen der Regel mit den gezogenen (`Run::rules_in_force`).
- **Kern**: `Run::new_weekly(content, week, start)` setzt Seed und Regel (`RunStart::weekly_rule`, `RunStart::week`), die Sicht trägt `RunView::weekly`; vor dem Start nennt `week_info` Woche, Seed, Regel und festen Charakter. Web: `weekInfo(date)` und `CoreRun.createWeekly`.
- **Tafel der Woche**: wie die Tafel des Tages (gleicher Code auf dem Server, `server/src/board.rs`, Tabelle `weekly_results`, Migration 2). Es zählt der **erste beendete Wochen-Run** je Konto; jeder weitere ist Übung: Er zählt im Vereinsbuch, nicht auf der Tafel (der Server behält das erste Ergebnis und antwortet `stored: false`). Der Server kennt weder Seed noch Regel.
- **Oberfläche**: Auf dem Startbildschirm unter der Tafel des Tages ein Block „Herausforderung der Woche“ mit Kalenderwoche, Restzeit bis Montag 0 Uhr, Name und Text der Regel, dem gewerteten Ergebnis und den Knöpfen „Wochen-Run als …“ (danach „Üben als …“) und „Tafel der Woche“ (eigenes Blatt mit der Tafel). Im Run nennt das Menü die Regel; der Endbildschirm bietet keinen Seed-Link an, weil der Seed allein nicht die Herausforderung ist.
- **Simulator**: `--weekly-rule <id>` legt die Regel auf jeden Run der Messung (auf den üblichen Seeds), `--weekly <JJJJWW>` nimmt die Regel dieser Woche.

**Gemessen** (10./11.11.2026, `docs/balance/2026-11-10-wochenregeln.md`): Standard-Bot, 3000 Runs je Regel auf denselben Seeds, Grundlage der Alleinspieler ohne Regel (Nullabend: der Nullspieler). Grenze: gewonnen etwa zwischen der Hälfte und dem Doppelten der Grundlage. Nachgestellt wurden drei Regeln: Teufelspakt war viel zu leicht (5,3 % gewonnen, der Schutzengel ist eine zweite Chance) und bekam Zielwerte +30 %; Spelunkenluft war zu schwer (0,7 %) und startet jetzt mit dem Spelunkenkenner statt mit 6 Groschen; Grand zählt doppelt war zu leicht (4,3 %) und lässt dafür Farbspiele nur drei Viertel zählen.

| Regel | Stadtteil 2 | Stadtteil 3 | gewonnen |
|---|---|---|---|
| Grundlage Alleinspieler | 35,4 % | 10,6 % | 2,1 % |
| Grand zählt doppelt | 38,9 % | 13,0 % | 3,5 % |
| Spelunkenluft | 36,3 % | 13,2 % | 1,9 % |
| Hellwache Gegner | 35,1 % | 9,8 % | 1,6 % |
| Kurze Geduld | 32,5 % | 9,6 % | 1,7 % |
| Teufelspakt | 49,9 % | 13,7 % | 1,7 % |
| Nullabend (Grundlage Nullspieler: gleich) | 34,1 % | 12,4 % | 2,6 % |
| Erbonkel | 35,3 % | 10,4 % | 1,6 % |
| Spendierhosen | 39,9 % | 12,3 % | 2,7 % |

## 6. Ein Spiel im Detail

1. **Geben.** Das Skatblatt (32 verschiedene Karten) wird gemischt, 10/10/10/2. Jede Karte zeigt die Ausgabe ihres Halters: In der Hand des Spielers und im Skat seine, bei einem Stammgast die normale, beim Boss die Boss-Ausgabe (ADR 0015, 7.1).
2. **Blatt ansehen.** Der Spieler sieht seine 10 Karten. Relikte, die Informationen geben (Kiebitz), wirken jetzt.
3. **Skat oder Hand.** Skat aufnehmen und 2 Karten drücken, oder Hand spielen. Gedrückte Karten zählen zu den Augen des Alleinspielers. Oder vorher einmischen: neu geben lassen, bis zu zweimal pro Spiel (5.1).
4. **Ansage.** Spielart und Risikostufe wählen (ADR 0011, 0013).
5. **10 Stiche.** Vorhand spielt aus. Die Gegner-KI spielt gemeinsam gegen den Spieler.
6. **Abrechnung.** Augen zählen, Spitzen bestimmen, Stufen summieren, Spielwert berechnen, Serie-Konto aktualisieren.
7. **Triggerphase.** Relikte mit Nachwirkung (z. B. „Bock“ läuft weiter) werden aktualisiert.

Die Rollen Vorhand, Mittelhand, Hinterhand rotieren pro Spiel. Vorhand zu sein ist ein echter Vorteil im Skat und bleibt als Rhythmus erhalten.

---

## 7. Deckbau

### 7.1 Ein Blatt, drei Ausgaben (ADR 0015)

**Bis 02.11.2026 (ADR 0005):** Es gab genau ein Blatt pro Run, alle drei Spieler spielten daraus, jede Modifikation und jede hinzugefügte oder entfernte Karte betraf alle. Bei mehr als 32 Karten blieb der Rest im Talon, bei weniger füllten Blanko-Karten auf.

**Begriff (04.11.2026):** In Spieltexten heißt die Ausgabe des Spielers **Deck** („in deinem Deck“), die der Gegner „Deck des Gegners“ und „Boss-Deck“ (Wunsch des Nutzers: „Ausgabe“ war nicht verständlich). Das Design-Dokument und der Code (`Edition`) bleiben bei Ausgabe; gemeint ist dasselbe.

**Seit ADR 0015 (Entscheidung des Nutzers vom 02.11.2026):** Gegeben wird ein normales Skatblatt aus 32 verschiedenen Karten mit 120 Augen. Jeder Spieler besitzt seine eigene **Ausgabe** jeder Karte: dieselbe Karte, aber mit seinen Modifikationen. Wer eine Karte bekommt, hält seine Version. Der Spieler baut über den Run nur an seiner Ausgabe; die Gegner haben ihre eigene (Stammgäste die normale, 10.4; Bosse eine Boss-Ausgabe, 11). Keine Karte verlässt das Blatt, keine kommt hinzu, es gibt keinen Talon und kein Blatt über oder unter 32 Karten.

Regeln, entschieden vom Nutzer (1 bis 3) und ausgelegt von Claude am 02.11.2026 (4 bis 9):

1. **Keine Karten hinzufügen oder entfernen.** Was bisher das Blatt veränderte, ist ein Karteneffekt (7.3). Charaktere unterscheiden sich durch Modifikationen auf ihrer Ausgabe, nicht durch eine andere Zusammensetzung (9).
2. **Der Skat gehört dem Alleinspieler.** Die zwei Skatkarten sind immer Ausgaben des Spielers, gegeben wie gedrückt, auch im Handspiel (ihre Augen zählen für ihn wie immer).
3. **Gegnerkarten haben eine eigene Rückseite.** Die Rückseite zeigt den Besitzer der Ausgabe: die Distel (oder eine freigeschaltete Rückseite, 9.3) für die Karten des Spielers und den Skat, eine Gegner-Rückseite für die Hände der Gegner, auf Wunsch je Gast oder Boss eine eigene.
4. **Eine Karte zeigt immer die Ausgabe ihres Halters.** Wechselt sie den Halter, wechselt ihre Ausgabe mit: die Karte, die „Drei drücken“ vom linken Gegner holt, ist in der Hand des Spielers seine Ausgabe, die zurückgegebene wieder die des Gegners (8.1); beim Revolutionär tauschen die Gegner Karten, die Ausgaben bleiben bei den Plätzen (11); Heimweh holte die Karte vor dem Spiel in die Hand (gestrichen mit Effekte v2, 7.2).
5. **Im Stich bleibt eine Karte in der Ausgabe dessen, der sie gespielt hat.** Ihre Augen zählen für den, der den Stich macht. Eine Karte des Spielers mit Augen +4, die ein Gegner sticht, bringt dem Gegner die vier Augen. Das ist das verbliebene Risiko beim Ausspielen.
6. **Der Skat bleibt auch im Ramsch die Ausgabe des Spielers**, wenn er an den Gewinner des letzten Stichs geht; er wird nie gespielt und wechselt deshalb keinen Halter.
7. **Boss-Regeln sind Regeln des Tisches** und gelten für jede Ausgabe (die Oma macht jeden Unter zur Blanko, die Augen-Regeln eines Bosses zählen jede Karte). Hausregeln wirken wie Regeln des Tisches, mit einer Ausnahme seit dem 04.11.2026 (Entscheidung des Nutzers): **Augen-Relikte zählen nur die eigenen Karten.** Alte Oberin, Kleinvieh und Dicke Sau (`SetCardPoints` aus einem Relikt oder der Regel des Charakters) ändern nur Karten der Ausgabe des Spielers, wo sie auch landen: Seine Oberin zählt 5, auch im Stich eines Gegners (wie eine Zinke Aufdruck, Regel 5), die Oberin eines Gegners 3 (`engine::owns_card_points`). Kartenmodifikationen hängen ohnehin an einer Ausgabe.
8. **Die Ausgaben sind öffentlich, die Hände nicht.** Der Spieler sieht vor der Boss-Serie die Boss-Ausgabe neben der Regel, die Gegner-KI kennt die Ausgabe des Spielers (10.5). Wer welche Karte hält, bleibt unbekannt wie im Skat.
9. **Aufwertungen schlafen.** Im Mittel hält der Spieler 12 von 32 Karten (Hand und Skat, 37,5 %). Liegt eine Karte bei einem Gegner, spielt in diesem Spiel dessen Ausgabe; die Aufwertung des Spielers wirkt dann nicht. Das ist das Risiko des Deckbaus (Säule 2).

Im Kern (umgesetzt in M9, 18): Die Ausgabe des Spielers ersetzt das Blatt des Runs (`Run::edition`, Typ `Edition`, je Karte die Modifikationen); die Ausgaben der Gegner ergeben sich aus dem, der am Platz sitzt (Boss links, `BOSS_SEAT`, oder Gast), und werden nicht gespeichert. Gegeben werden die 32 Karten in der Reihenfolge und mit dem Zufallsstrom von bisher (`Game::deal_editions`), dann wird jede Karte mit der Ausgabe ihres Halters aufgelöst; jede aufgelöste Karte nennt den Besitzer ihrer Ausgabe (`DealtCard::owner`: Spieler, Stammgast, Boss). Wechselt eine Karte den Halter, löst eine einzige Funktion sie neu auf (`Game::change_holder`, Ereignis `CardChangedHands`): Drei drücken (gezogene und zurückgegebene Karte), Revolutionär, Schieben (ohne Einzelereignisse, das Ergebnis steht in `SkatRedrawn`). Untot wechselt keinen Halter: Karte und Ersatz bleiben beim Spieler. Kartenmodifikationen feuern nur in der Ausgabe ihres Besitzers (`card_mod_sources` mit `Editions`).

### 7.2 Kartenmodifikationen: Effekte v2

**Entschieden vom Nutzer am 02.11.2026 („Effekte v2“), maßgeblich ab hier.** Das System aus M4 bis M8 (26 Modifikationen, mehrere je Karte, Rahmensprache mit Ringen und Abzeichen) ist ersetzt durch ein kleines System mit wenigen, steigerbaren Modifikationen. Geschichte in einem Satz: Fett, Goldrand, Gezinkt, Stempel, Krone und ihre Verwandten gehen in Augen, Rang und Trumpf mit Werten auf, Spitze heißt jetzt Letzter Stich, die Schwarze Katze heißt Fleckig, Ramponiert bekommt eine neue Bedeutung, alles andere ist gestrichen (Tabelle „Umrechnung“ unten). Daten in `core/content/card_mods.ron`, Preise, Werte und Seltenheit sind Platzhalter für `balance`.

**Grundregeln**

1. **Eine Modifikation je Karte.** Jede Karte trägt höchstens eine Modifikation. Nur Spezialkarten (Untot, Wandelkarte, Doppelgänger) dürfen zusätzlich eine normale Modifikation tragen. Seit dem 05.11.2026 (Entscheidung des Nutzers nach dem Playtest) darf eine Ausgabe **beliebig viele** Spezialkarten haben; vom 03.11. bis 05.11.2026 war höchstens eine Karte je Ausgabe Untot und höchstens eine Wandelkarte (unten, „Spezialkarten im Run“).
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
| Schrift unten Mitte, auf einem Farbklecks-Band | **wie viel** (+5, −3, ×2, +1 Stufe) |
| Feld unten links, quadratisch | nur für Flüche und negative Effekte |
| ganzes Kartenbild | Spezialkarten ändern das Grunddesign (Untot, Wandelkarte) |

**Die Modifikationen**

| Gruppe | Modifikation (`id`) | Wirkung | Werte (Platzhalter) | Hebel | Bild |
|---|---|---|---|---|---|
| Rand | Augen (`points`) | ±X Augen | Tresen +3, +5; zufällig +4 | Augen | Rahmen Augen, Zahl im Band |
| Rand | Rang (`rank`, „Winkel“) | seit 03.11.2026: wird die nächsthöhere Karte ihrer Farbe (7, 8, 9, Oberin, König, 10, Daus), behält ihre Augen; nicht auf Unter und Daus (unten, „Zinken v2“) | +1, Steigerung bis 2 (nicht über den Daus) | Stichgewinn | seit 04.11.2026 (Winkel v2): gedrucktes Bild bleibt, violetter Kopierstift malt darüber (Zusatz-Auge, Figurskizze, großes Daus-Auge), gedruckter Rang im Eckzeichen dick zugekritzelt, neuer Rang in Handschrift daneben |
| Rand | Blei (`lead`) | im Null die niedrigste Karte ihrer Farbe, unter der Sieben | – | Stichgewinn (Null) | bleigrauer Doppelrahmen, Senklot im Band (BL-A) |
| Rand | Goldkante (`gilt_edge`) | +X Groschen je Stich, den der Spieler mit ihr macht | +1, Steigerung addiert | Groschen | Goldschnitt mit Glanzpunkten, Band „+1“ mit Münze (GK-A) |
| Rand | Trumpf (`trump_bonus`) | Grundwert +X, wenn die Farbe der Karte Trumpf ist und der Spieler sie hält | Tresen +2, +4; zufällig +3 | Grundwert | Rahmen Trumpf, Zahl im Band |
| Moment | Letzter Stich (`last_trick`) | +1 Stufe, wenn diese Karte den letzten Stich macht | +1, nicht steigerbar | Stufen | Markierung oben rechts, Text unten Mitte |
| Moment | Erster Stich (`first_trick`) | ×2 Augen dieses Stichs, wenn diese Karte den ersten Stich gewinnt | ×2, nicht steigerbar | Augen | Markierung oben rechts, Text unten Mitte |
| Moment | Fadenkreuz (`crosshair`) | +1 Stufe, wenn diese Karte einen Daus eines Gegners sticht (je solchem Stich) | +1, nicht steigerbar | Stufen | Zielrote Ecke mit weißem Fadenkreuz, Doppelrahmen, Band „+1“ mit Stufen-Treppe (FK-A) |
| Moment | Schneiderkreide (`tailors_chalk`) | +1 Stufe, wenn der Spieler mit Schneider gewinnt und diese Karte gespielt hat | +1, nicht steigerbar | Stufen | heller Rand mit Steppnaht, Schere in der Ecke, Band „+1“ mit Stufen-Treppe (SK-B) |
| Gutschein | Gutschein (`voucher_coins`) | wenn gedrückt oder im Handspiel: +X Groschen | +3, Steigerung addiert | Groschen | Zackenrand, keine Markierung |
| Gutschein | Gutschein Untot (`voucher_undead`) | wenn gedrückt oder im Handspiel: die Karte wird Untot | – | Spezialkarte | Zackenrand |
| Gutschein | Gutschein Wandelkarte (`voucher_wild`) | wenn gedrückt oder im Handspiel: die Karte wird Wandelkarte | – | Spezialkarte | Zackenrand |
| Fluch | Durchsichtig (`transparent`) | das Wissen hängt an der Karte (siehe unten) | – | Zählen | Feld unten links |
| Fluch | Ramponiert (`battered`) | −X Augen, auch unter 0: Augen mit negativem Wert (Grundregel 5) | −5 | Augen | Feld unten links, Zahl im Band |
| Fluch | Fleckig (`stained`) | Grundwert −X, wenn die Farbe der Karte Trumpf ist und der Spieler sie hält | −3 | Grundwert | Feld unten links, Flecken leicht durchscheinend über der Karte |
| Spezial | Untot (`undead`) | seit 03.11.2026: kehrt als die Karte zurück, die den Stich gemacht hat; der Spieler wählt 3 Karten, eine davon geht in den Stich (unten) | – | Stichgewinn, Augen | eigenes Kartenbild |
| Spezial | Doppelgänger (`doppelganger`) | seit 07.11.2026 „Spiegelbild“: als letzte Karte eines Stichs einmal je Spiel eine Kopie der höchsten Karte darin, gewinnt den Stich mit deren Augen (unten) | – | Stichgewinn, Augen | Spiegelbild: rechte Hälfte violett auf Lavendel (DG-A) |
| Spezial | Wandelkarte (`wild_card`) | bei Spielbeginn: nimmt die Farbe an, von der der Spieler die meisten Karten hält (ohne Unter; unten) | – | Stichgewinn, Bedienpflicht | eigenes Kartenbild (Harlekin in vier Farben, solange sie offen ist) |

**Auslegungen**, entschieden von Claude am 02.11.2026 im Rahmen der Entscheidung des Nutzers:

- **Werte in den Daten.** Eine Modifikation nennt ihre Wirkung je Einheit (`AddCardPoints(1)`, `RankBonus(1)`, `AddBaseValue(1)`, `AddLevels(1)`, `GainCoins(1)`, `MultiplyTrickPoints(1)`), die Karte trägt den Wert (`DeckCard::mod_values`, ohne Eintrag 1), der Kern vervielfacht beim Wirken (`Effect::scaled`). `values` an der Modifikation sind die angebotenen Werte, der erste gilt für Zufallsziehungen; `max` begrenzt die Steigerung. Texte tragen den Wert als Platzhalter (`{n}` Betrag, `{+n}` mit Vorzeichen).
- **Augen unter 0.** Augen einer Karte dürfen unter 0 fallen (Ramponiert auf einer Lusche: −5). Wer den Stich macht, verliert die Augen. Die Grenzen 61, 90 und 30 bleiben fest, die Gesamtaugen können unter 120 liegen; die Augen einer Seite fallen nie unter 0. Im Kern hat die Karte 0 Augen und eine Schuld (`DealtCard::point_debt`), die die Stichzählung abzieht. Die KI sieht nur die 0.
- **Erster Stich.** Gewinnt die Karte den ersten Stich des Spiels, zählen dessen Augen für den Gewinner doppelt (der Zuschlag liegt in `Game::bonus_points`, Wirkung `MultiplyTrickPoints`, Bedingung `ThisCardWinsFirstTrick`). Im geteilten Blatt auch für einen Gegner, der damit sticht.
- **Letzter Stich** ist die alte „Spitze“ ohne „egal, wer sie spielt“ im Text; „Rest abkürzen“ wartet wie bisher, bis entschieden ist, wer den letzten Stich macht.
- **Trumpf und Fleckig, „hält“:** die Karte liegt in seiner Hand oder im Skat (der Skat gehört dem Alleinspieler, ADR 0015) oder er hat sie in einen Stich gespielt. Geprüft bei der Ansage (`ThisCardIsTrumpSuit` mit dem Schalter oben).
- **Gutschein, „wenn gedrückt“.** Wirkt außerhalb des laufenden Spiels: Landet die Karte beim Drücken im Skat, wird der Gutschein sofort eingelöst, egal wie das Spiel ausgeht, und fällt danach von der Karte (Auslöser `OnDiscard`, Ereignis `VoucherRedeemed`). Bei „Drei drücken“ löst die Karte, die an den Gegner zurückgeht, nichts ein. Im Handspiel wird nichts gedrückt; dafür gilt seit dem 04.11.2026 die nächste Regel. Die Spezial-Gutscheine machen die Karte zur Spezialkarte (Wirkung `TurnInto`); eine andere Spezialkarte auf derselben Karte wird ersetzt. Ist schon eine andere Karte der Ausgabe diese Spezialkarte, zahlt die Marke statt dessen 3 Groschen (seit 03.11.2026, unten).
- **Marke im Handspiel** (Entscheidung des Nutzers vom 04.11.2026: Handspiele sollen nicht bestraft werden). Sagt der Spieler ein Handspiel an (jedes: Farbspiel, Grand, Null Hand, Null Ouvert Hand, auch unter dem Schieber oder der Spelunke „Kein Skat“; nicht der Ramsch), wird **eine** zufällige Marke auf einer Karte seiner Hand sofort eingelöst, genau wie gedrückt: Die Pfandmarke zahlt ihre Groschen, Toten- und Narrenmarke machen die Karte zur Spezialkarte, mit derselben Obergrenze und derselben Rückfallzahlung (`VoucherCapped`). Danach ist die Marke weg; beim Drückeberger kommt sie nach der Kneipe zurück wie eine gedrückte (9.4, auch die Ausnahme: Hat sie die Karte verwandelt, bleibt sie weg). Trägt keine Handkarte eine Marke, passiert nichts. Die Karte bleibt auf der Hand und wird normal gespielt. Gezogen wird mit einem eigenen Zufall aus dem Seed des Spiels (`HAND_VOUCHER_SEED`, gleichverteilt über die Marken der Hand, nach Karten-Id sortiert), also deterministisch, ohne das Geben oder andere Ziehungen zu verschieben. Ereignis `VoucherRedeemed { via_hand: true }` (`Run::redeem_hand_voucher`), dann wie gedrückt `DeckChanged`, `CoinsGained` oder `VoucherCapped`. **Zeitpunkt (Auslegung von Claude):** unmittelbar vor dem Spielbeginn, also vor `Declared`. So wirkt die neue Spezialkarte schon in diesem Spiel, wie eine Spezialkarte, die von Anfang an auf der Karte lag: Eine Wandelkarte nimmt mit der Ansage die Farbe an, wie die Regel „bei Spielbeginn“ sagt (unten), eine Untote Karte kann in diesem Spiel aufstehen. Das ist die einheitliche Lesart, weil eine gedrückte Karte ihre Spezialkarte in diesem Spiel nie nutzen könnte, eine Handkarte aber schon zu Spielbeginn verwandelt ist. Der Bot entscheidet über Hand wie bisher; die Marke wiegt neben dem Risiko eines Handspiels wenig und wird erst gemessen, wenn die Simulationen wieder laufen. Die Oberfläche sagt es im Ansage-Dialog und auf der Infokarte „Hand spielen“ („Eine zufällige Marke auf deiner Hand wird sofort eingelöst.“), sobald eine Handkarte eine Marke trägt, und zeigt beim Einlösen dieselbe Animation wie beim Drücken (Marke reißt ab, Münze oder Karte wendet).
- **Zufallsziehungen** (bis 02.11.2026 Stempelkissen; Kartengeber; seit 8.6 auch Auslage und Kartenmaler, dort zusätzlich nach Seltenheit) ziehen aus Rand, Moment und Gutschein, gewichtet nach `focus_weights` wie bisher, nie einen Fluch und nie direkt eine Spezialkarte (die kommt über den Gutschein; einzige Ausnahme seit 03.11.2026 das seltene Spezialkarten-Angebot der Auslage, 8.6) und nie eine Toten- oder Narrenmarke, deren Spezialkarte die Ausgabe nicht mehr bekommen darf; der Wert ist der erste aus `values`. Liegt auf der Karte schon dieselbe Modifikation, steigt sie; eine andere wird ersetzt.
- **Tresen.** Bis 02.11.2026 gab es konkrete Werte als Verbrauchsgegenstände (`ApplyCardMod`: Augen +3, Augen +5, Trumpf +2, Trumpf +4, Rang +1). Seit den Zinken (8.6) verkauft der Tresen Modifikationen nur noch in der Auslage und über den Kartenmaler; die Aktionen ruhen.
- **Anzeigenamen seit dem 02.11.2026** (Entscheidung des Nutzers, 8.6): Augen heißt **Aufdruck**, Rang **Winkel**, Trumpf **Siegel**, Erster Stich **Eselsohr**, Letzter Stich **Kerbe**, Gutschein (Groschen) **Pfandmarke**, Gutschein Untot **Totenmarke**, Gutschein Wandelkarte **Narrenmarke**. Die Tabelle oben nennt die Wirkung; Ids und Bezeichner im Code bleiben. Jeder Text beginnt mit der Wirkung („Erster Stich: Gewinnt diese Karte …“), damit Infokarten sie neben dem Namen zeigen.
- **Durchsichtig (Fluch).** Das Wissen hängt an der **Ausgabe, die den Fluch trägt** (ADR 0015). Liegt die durchsichtige Karte des Spielers bei ihm (Hand oder Skat), kennen die KI-Gegner sie: Die Stichproben der PIMC-Gegner legen sie in seine Hand oder in den Skat (`opponents::defender_hints`). Seit 07.11.2026 (Playtest) spielen die Gegner dann auch in Stadtteil 1 PIMC statt der Heuristik, die das Wissen nicht verwerten kann (10.3). Eine Gegner-Ausgabe kann Durchsichtig auch tragen (Schwarzer Peter, die Hausregel Glasauge, 8.1): Dann sieht der Spieler die Karte mit ihrem Halter, solange der Gegner sie hält (`GameView::revealed`, die Karte des Halters mit dessen Modifikationen und Rückseite). **Bugfix vom 04.11.2026 (Playtest):** Bis dahin zeigte ein Durchsichtig auf der Karte des Spielers auch die gleiche Karte eines Gegners (seit M9 sogar mit dessen Ausgabe), weil der Kern sich die Karten-Ids merkte, die die Ausgabe des Spielers markiert (`ActiveGame::player_marked`), und jede Kopie mit dieser Id aufdeckte. Seit ADR 0015 ist das falsch: Jeder spielt seine eigene Ausgabe, der Fluch auf der Karte des Spielers sagt nur, dass die Gegner *seine* Karte sehen. Heute zählt nur die Markierung der Kopie im Spiel (`DealtCard::revealed`, `Run::revealed_holders`); `player_marked` ruht und wird nur noch gelesen. Der Kiebitz-Mörder schaltet alle Richtungen ab.
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

**Untot (umgesetzt am 02.11.2026, Idee des Nutzers; umgebaut am 03.11.2026, siehe „Zinken v2“ unten: zurück als die Karte, die den Stich gemacht hat, Wahl aus drei Karten; die folgenden Punkte gelten weiter, wo der Umbau nichts anderes sagt).** Text: „Karte wird gespielt, dann aber wieder auf die Hand genommen und eine zufällige Karte abgeworfen. Einmal pro Spiel, nur im Farbspiel und Grand.“ (`undead`, Effekt `Undead`). Seit Effekte v2 eine Spezialkarte: Sie kommt über den Gutschein Untot und darf zusätzlich eine normale Modifikation tragen. Auslegung, entschieden von Claude am 02.11.2026:

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

**Zinken v2 (Entscheidungen des Nutzers vom 03.11.2026, Brett „Zinken v2“, Cockpit q15 bis q17).** Vier neue Zinken, Winkel mit neuer Bedeutung, Untot umgebaut, eine neue Spezialkarte. Werte und Preise **vorläufig** nach Augenmaß (die Simulationen ruhen). Auslegungen von Claude:

- **Winkel, neue Bedeutung** (`rank`, Effekt `Promote`, ersetzt `RankBonus`): Die Karte **wird** je Stufe die nächsthöhere Karte ihrer Farbe, in der Reihe 7, 8, 9, Oberin, König, 10, Daus, in jedem Spiel (auch im Null, wo sie dort nach der Null-Reihe sticht: eine Neun als Oberin sticht Zehn und Unter), und behält ihre eigenen Augen (Antwort C auf q15). Der Kern ändert beim Auflösen der Ausgabe ihre Art (`DealtCard::kind`) und merkt sich die Stufen (`DealtCard::promoted`, `Rank::promoted`). Unter steigen nie und werden nie erreicht (eine Neun wird Oberin), ein Daus kann nicht steigen: Winkel passt auf keinen Unter und keinen Daus (`CardModDef::fits`), eine Zehn steigt höchstens einmal (`CardModDef::max_on`). Doppelte Karten sind erlaubt wie bei der Wandelkarte; bei Gleichstand behält die früher gespielte den Stich. Spitzen zählen die Karte, wie sie ist (ein Trumpf-König mit Winkel ist die Trumpf-Zehn). **Alte Spielstände** (Version 4 der Modifikationen, `deck::migrate_winkel`): Winkel behalten ihren Wert mit der neuen Bedeutung, höchstens so viele Stufen, wie die Karte steigen kann; ein Winkel auf einem Unter oder Daus wird Aufdruck mit 3 Augen je Stufe (`WINKEL_TO_AUGEN`). Die Boss-Ausgaben (Fritz, Franz) und Bruderherz behalten ihre Winkel, die jetzt die höhere Karte bedeuten. Bild seit dem 04.11.2026 (Entscheidung des Nutzers, Brett „Winkel v2“: WK-A mit der Tinte von WK-B und dem Durchstreichen von WK-C): Das gedruckte Bild bleibt, ein violetter Kopierstift malt darüber. Steigt die Zahl (7→8, 8→9, König→10), kommt ein von Hand gemaltes Auge der Farbe dazu; wird es eine Bildkarte, wird die Figur skizziert (Oberin mit Dutt, König mit Krone und Bart); der Daus bekommt ein großes gemaltes Auge. Die Skizzen sind mit Papier ausgemalt. Im Eckzeichen (oben links und unten rechts) ist der gedruckte Rang dick zugekritzelt und der neue in einer runden Handschrift daneben geschrieben; zwei Stufen sind zwei Schichten.
- **Blei** (`lead`, Effekt `NullLowest`, `DealtCard::leaden`): im Null die niedrigste Karte ihrer Farbe, unter der Sieben (`rules::suit_strength`); zwei Bleikarten derselben Farbe: die früher gespielte behält den Stich. Sonst die gedruckte Karte, ihre Augen bleiben. Ohne Wert. Fokus Null.
- **Goldkante** (`gilt_edge`, Bedingung `ThisCardWinsTrick` bei Stichende, Wirkung `GainCoins`): je Stich, den diese Karte macht, +1 Groschen sofort (eine untote Karte kann zweimal stechen). Eine Karte des Spielers wirkt nur in seiner Ausgabe, also nur, wenn er sie spielt.
- **Fadenkreuz** (`crosshair`, Bedingung `ThisCardBeatsDefenderCard((rank: Some(Ace)))`): sticht diese Karte einen Daus, den ein Gegner in den Stich gespielt hat, +1 Stufe für dieses Spiel, je solchem Stich. Der eigene Daus zählt nicht. **Vorerst aus dem Angebot genommen** (Playtest 12.11.2026): `retired: true` in `card_mods.ron`; Auslage, Kartenmaler, Kartenbelohnung, Begegnungen und Zufallsziehungen bieten sie nicht mehr an (`CardModDef::is_offered`). Die Id bleibt ladbar: Karten in alten Spielständen behalten das Fadenkreuz samt Wirkung, ein schon ausgelegtes Angebot bleibt kaufbar.
- **Schneiderkreide** (`tailors_chalk`, Bedingung `All([WonSchneider, ThisCardPlayed])` bei der Wertung): gewinnt der Spieler mit Schneider (auch Schwarz, angesagt oder gespielt) und hat die Karte in einen Stich gespielt, +1 Stufe. Im Skat zählt sie nicht. „Rest abkürzen“ wartet, solange Goldkante, Fadenkreuz oder Schneiderkreide noch an einer Karte der Hand hängen.
- **Untot, Umbau** (Cockpit q16): Ist der Stich voll, kehrt die Karte als **Kopie der Karte zurück, die den Stich gemacht hat** (`PlayingCard::risen_as`: Art, Augen und Regel-Eigenschaften der Gewinnerkarte, Kennung, Halter und Sichtbarkeit der eigenen; ihre eigenen Zinken wirken über die Kennung weiter). Hat sie den Stich selbst gemacht, kommt sie unverändert zurück. Auch ein Unter ist möglich: ein zusätzlicher Trumpf, aber nie eine Spitze, die Spitzen bleiben, wie angesagt (die zwölf Karten des Alleinspielers behalten die gedruckte Karte). Die Ersatzkarte: Der Spieler **wählt 3 Karten** seiner Hand, eine davon wird mit dem Zufall des Spiels gezogen (`undead_rng`, nach Kennung sortiert, die Reihenfolge der Wahl ändert nichts) und liegt an ihrer Stelle im Stich. Bis zur Wahl wartet der Run (`ActiveGame::pending_rise`, `Run::rise_pending`, Ereignis `RunEvent::RiseChoice { card, becomes, pick }`, Sicht `GameView::rise_choice`, Aktion `RunAction::ChooseRiseCards([a, b, c])`, Fehler `RisePending`, `InvalidRiseCards`); kein Gegner zieht, nichts anderes geht. Hat er nur noch 3 oder weniger andere Karten, sind alle die Kandidaten und sie steigt sofort (bei einer ist es diese). Ereignis `CardRose` mit `risen` (der Karte in der Hand). Bot: die drei schwächsten Karten (wenigste Augen, dann niedrigste Stärke). Web-App: Blatt „Wähle 3 Karten – eine davon geht in den Stich“ mit der Karte, als die sie zurückkommt, danach der grüne Bogen zurück in die Hand mit dem neuen Bild. Seit dem Playtest vom 05.11.2026 sieht eine Untot-Karte bis zum Aufstehen aus wie die gedruckte Karte mit einem grünen Band und Totenkopf oben; das Zombiebild trägt nur die Karte, die zurückgekommen ist (`CardView::risen`, aus `ActiveGame::risen`). Beim Aufstehen liegt die gespielte Karte groß über dem Tisch und zerbröckelt von oben nach unten in die Zombiekarte, als die sie zurückkommt (Tempo, reduzierte Bewegung und Antippen wie bei allen Szenen).
- **Doppelgänger** (`doppelganger`, Spezialkarte, Effekt `Doppelganger`, Cockpit q17; seit 07.11.2026 „Spiegelbild“, Entscheidung des Nutzers, Cockpit q37 Option A): seit 05.11.2026 wie Untot und Wandelkarte ohne Obergrenze je Ausgabe (bis dahin höchstens einer), auch im seltenen Spezialangebot der Auslage. Er bedient als seine gedruckte Karte (nur spielbar, wenn sie es wäre). Spielt der Spieler ihn als **letzte (dritte) Karte eines Stichs**, wird er eine **Kopie der höchsten Karte im Stich** (Bild, Rang, Augen und Modifikationen dieser Karte, seine eigene Identität: Instanz, Besitzer; `PlayingCard::risen_as` wie beim Untot) und **gewinnt den Stich**, auch gegen die kopierte Karte (er kommt später und ist so festgelegt). Der Stich zählt die Augen der Kopie statt seiner eigenen: ein Stich aus Daus, Sieben und gespiegeltem Daus bringt 22 Augen; die 120 Augen eines Spiels können so überschritten werden. **Einmal je Spiel**, gleich wie viele Doppelgänger die Ausgabe hat (der erste, der spiegelt, verbraucht es; ein weiterer spielt dann als er selbst). Als erste oder zweite Karte, nach dem Spiegeln, **im Null** (dort wäre ein Stich für den Spieler schlecht; Auslegung von Claude: einfach seine gedruckte Karte) und **im Ramsch** spielt er als er selbst. Die Kopie ersetzt die Karte in den Karten des Spiels und den Ausgaben wie ein aufgestandener Untot; die zwölf Karten des Alleinspielers bleiben, wie gegeben, die Spitzen also, wie angesagt. Nur Karten des Spielers spiegeln. Code: `Run::can_mirror` (noch verfügbar), `Run::mirrors_now` (spiegelt, wenn jetzt gespielt), `Game::play_mirrored` (Fehler `GameError::NotLastCard`, wenn der Stich nicht zwei Karten hat), Ereignis `DoppelgangerMirrored { seat, card, copied, copy }`, dann `CardPlayed` mit der Kopie und `TrickWon`; Sicht `CardView::can_mirror`. „Rest abkürzen“ wartet, solange ein Doppelgänger noch spiegeln kann. Die Obergrenze des Spielwerts (`Run::max_game_value`, „Elle“) rechnet mit einem Doppelgänger in der Ausgabe mit 120 Augen plus denen der reichsten Karte im Spiel. **Bis 07.11.2026** nahm er dem nächsten Gegner eine Karte gleichen Rangs ab (Ereignis `DoppelgangerSwapped`, Zufall `doppel_rng`); das Ereignis bleibt, damit mitten im Spiel gespeicherte Spielstände laden, der Zufallsstrom wird beim Laden übergangen. **Bot** (`RunBot::mirror_card`, einfache Regel, nicht abgestimmt): spielt als Letzter einen Doppelgänger, der spiegelt, wenn der Stich mit der Kopie mindestens 15 Augen wert ist (`MIRROR_MIN_AUGEN`: ein Daus oder eine Zehn, kopiert) oder er die einzige spielbare Karte ist; sonst hält er ihn zurück (`RunBot::keep_for_mirror`: aus den spielbaren Karten genommen, solange eine andere geht; auch für die PIMC-Suche des Spielers im Simulator). Die Suche selbst kennt das Spiegeln nicht. **Web-App**: die Karte blitzt im violetten Spiegelbild, ein Pixelschimmer läuft darüber, „Gespiegelt!“, Hinweis „Spiegelbild: … wird zu … und gewinnt den Stich“; dann fliegt die Kopie in den Stich, als kopierte Karte im Spiegelbild-Look (DG-A). Die Infokarte nennt einen Doppelgänger, der noch spiegeln kann. Tempo, reduzierte Bewegung und Antippen wie bei allen Szenen.
- **Bot-Werte** (`ZINKEN_VALUES`): Goldkante 2, Fadenkreuz 1, Schneiderkreide 1, Blei 1, Doppelgänger 1.

Tests: `core/src/run/zinken_v2_tests.rs`, `core/src/run/undead_tests.rs`.

**Wandelkarte** (`wild_card`, Effekt `Wild`, seit Effekte v2 eine Spezialkarte über den Gutschein Wandelkarte). Regel neu entschieden vom Nutzer am 03.11.2026, Einzelheiten von Claude; sie ersetzt das Benennen beim Ausspielen (bis dahin `RunAction::PlayAs`, `rules::wild_suits`, Ereignis `WildCardNamed`, „zwingt nie zum Bedienen“). Text: „Bei Spielbeginn nimmt sie die Farbe an, von der du die meisten Karten hältst (ohne Unter). Bei Gleichstand gilt Eichel, Grün, Rot, Schellen.“

- **Wann:** im Moment des Spielbeginns, also nach Aufnehmen und Drücken mit der Ansage, im Handspiel mit der Ansage, auch im Null und im Ramsch (`Game::declare`). Bis dahin ist sie ihre gedruckte Karte, auch bei der Wahl der Drückkarten. Eine gedrückte Wandelkarte (und eine im Skat eines Handspiels) bleibt gedruckt. Macht die Narrenmarke im Handspiel eine Handkarte zur Wandelkarte (seit 04.11.2026, oben), geschieht das unmittelbar vor der Ansage; die Karte nimmt dann mit der Hand ihre Farbe an.
- **Welche Farbe:** die Farbe, von der der Spieler in seinen zehn Karten die meisten hält, gezählt nach gedruckter Farbe, ohne alle Unter und ohne die Wandelkarte selbst (`rules::settled_wild_suit`). Gleichstand: Eichel vor Grün vor Rot vor Schellen. Die Taktik dahinter: die zweitlängste Farbe drücken, damit die Wandelkarte in die Farbe kommt, die Trumpf werden soll. Mehrere Wandelkarten (seit 05.11.2026 ohne Obergrenze) zählen alle nicht mit und nehmen alle dieselbe Farbe an.
- **Was sich ändert:** Rang, Augen und die übrigen Modifikationen bleiben; die Karte ist bis zum Spielende die Karte dieser Farbe mit demselben Rang (in Hand, Deck und allen Ausgaben). Doppelte Karten sind erlaubt wie bei der Blue Curaçao (4.4). Ab dann gilt die normale Bedienpflicht für die neue Farbe; nichts wird beim Ausspielen benannt, und sie befreit nicht vom Bedienen. Eine Wandelkarte, die ein Unter ist, bleibt der Unter, unverändert. Die Spitzen bleiben, wie angesagt (gezählt über die gedruckten Karten). In der Hand eines Gegners ist sie die gedruckte Karte.
- **Ereignis und Sicht:** `Event::WildCardSettled { old, new }` (auch für den Spieler, `PlayerGameEvent::WildCardSettled`), je Wandelkarte eines, direkt nach `Declared`; die Web-App kann es animieren. `DealtCard::wild` heißt seitdem „noch offen“ und fällt beim Festlegen ab. Sicht: `CardView::unsettled` (noch offen, vor Spielbeginn: der Harlekin in vier Farben), `CardView::named` (gedruckte Farbe einer festgelegten Wandelkarte, `ActiveGame::named`), `GameView::wild_settled` (alle festgelegten Wandelkarten des Spiels mit Karte und gedruckter Farbe, auch schon gespielte: für die Ausgabe auf der Kreidetafel nach dem Neuladen).
- **Darstellung** (entschieden vom Nutzer am 03.11.2026): Offen trägt der Narr ein Harlekinkostüm in den vier Farben der Farben (Eichel-Blau, Grün, Rot, Schellen-Gelb), im Eckzeichen stehen der Rang und vier kleine Farbfelder. Festgelegt sind Kostüm und Kappe ganz in Tönen der neuen Farbe, das Eckzeichen ist das normale der Farbe (`tools/cardart/wild.py`, Palettentausch). Beim Festlegen laufen die Farben etwa 0,8 Sekunden durch alle vier und bleiben auf der neuen stehen (ohne Bewegung: sofort). Hand, Stich, Infokarte, Ausgabe auf der Kreidetafel und Abrechnung zeigen sie festgelegt, außerhalb eines Spiels (Tresen) offen.
- **Untot:** Eine untote Wandelkarte (nur aus alten Spielständen, die beide Spezialkarten behalten) steigt als die festgelegte Karte zurück.
- **Bot:** Nach dem Festlegen ist sie eine Karte wie jede andere; Suche und Bot sehen die festgelegte Karte (`opponents::tame_game` und `RunBot::wild_suit` gestrichen). Beim Drücken und bei der Ansage rechnet der Bot noch nicht voraus, welche Farbe sie annimmt.
- **Spielstände:** Ohne neue Version. Ein laufendes Spiel vor der Ansage legt sie bei der Ansage fest. Ein Spiel, das vor dem 03.11.2026 schon lief und noch eine Wandelkarte in der Hand hat, behält sie als gedruckte Karte (sie bedient dann normal); die Sicht zeigt sie nicht mehr als offen. Die Aktion `PlayAs` gibt es nicht mehr; alte Ereignisse `WildCardNamed` werden nicht mehr gelesen.

**Spezialkarten im Run** (entschieden vom Nutzer am 03.11.2026, „A + B“): (A) Ein Boss hat Spezialkarten in seiner Ausgabe; beim ersten Erhalt gibt es einen Hinweis, und die Wirkung wird deutlich animiert. (B) Die Boss-Belohnung „1 aus 3“ enthält immer eine Spezialkarte, selten auch die Tresen-Auslage; höchstens ein Untot und eine Wandelkarte je Ausgabe (diese Obergrenze hat der Nutzer am 05.11.2026 gestrichen, unten). Eigene Relikte für Spezialkarten gibt es nicht.

Stand der Umsetzung (03.11.2026): Der Boss mit Spezialkarten (Totengräber) ruht, und die Boss-Belohnung ist seit ADR 0017 ein Relikt aus dreien; aus (A) und (B) umgesetzt sind die Obergrenze, das seltene Angebot der Auslage (8.6), der Hinweis beim ersten Erhalt und die Animation. Auslegungen von Claude:

- **Keine Obergrenze (seit 05.11.2026, Entscheidung des Nutzers nach dem Playtest).** Eine Ausgabe darf beliebig viele Untot, Wandelkarten und Doppelgänger haben. `Edition::add_mod` prüft keine andere Karte mehr; auf der eigenen Karte gelten die alten Regeln (nochmal dieselbe: `AlreadyModified`; eine andere Spezialkarte ersetzt sie). Eine Toten- oder Narrenmarke verwandelt ihre Karte immer; ist die gedrückte Karte selbst schon diese Spezialkarte, passiert nichts. Auslage, Kartenbelohnung, Kartenmaler und Ereignisse bieten Spezialkarten und ihre Marken an, egal was die Ausgabe schon hat. Mehrere Wandelkarten nehmen alle dieselbe Farbe an (oben).
- **Bis 05.11.2026 (entfallen):** höchstens eine Karte je Spezialkarte (Fehler `DeckError::SpecialTaken`, Code `SpecialTaken`); eine Marke über der Obergrenze fiel ab und zahlte 3 Groschen (`capped_voucher_coins`, Ereignis `VoucherCapped`); Angebote prüften, ob die Spezialkarte noch „frei“ war (`Run::special_open`, `offer_open`, Sperrgrund `SpecialCardCap`). **Spielstände** brauchen dafür nichts: Felder und Ereignisse der Obergrenze wurden nie gespeichert, ältere Stände mit mehreren Spezialkarten waren schon erlaubt.
- **Hinweis beim ersten Erhalt.** Bekommt eine Karte der Ausgabe zum ersten Mal in einem Meta-Spielstand eine Spezialkarte (`DeckChanged` mit Untot oder Wandelkarte), zeigt die Web-App einmal ein Blatt „Neu in deiner Ausgabe“: die Karte, drei Sätze Regel (seit 05.11.2026 mit dem Satz, dass es beliebig viele sein dürfen, statt der Obergrenze). Gemerkt im Meta-Spielstand (`MetaState::specials_seen`, ohne neue Version; fehlt in alten Ständen und ist dann leer), nicht im Browser, damit es mit dem Spielstandscode umzieht. Der Kern meldet die neuen in `MetaStep::first_specials`.
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

### 8.0 Relikt-Plätze (05.11.2026)

**Entschieden vom Nutzer am 05.11.2026 (Cockpit q34 A): acht Relikt-Plätze**, Hausregeln und Figuren zusammen. Vom 03.11. bis 05.11.2026 waren die Plätze unbegrenzt (davor fünf); ein gewonnener Run hielt dann rund 36 Relikte, und die Relikt-Flut bestimmte die lange Nacht (`docs/balance/2026-11-05-nacht.md`). Mehr Plätze nur über seltene Relikte oder Charaktere; keiner nutzt das bisher, die Daten lassen es zu.

- **Daten**: `relic_slots: Some(8)` in `run.ron` (`Option<u32>`, ohne Angabe keine Grenze). Ein Charakter kann eine eigene Obergrenze setzen (`CharacterRule::relic_slots: Some(n)`), es gilt die kleinere. `Content::relic_slots`, `Content::relic_slot_free`. Abendregeln (Stammtisch-Streit) brauchen keinen Platz.
- **Volle Plätze: tauschen.** Wo ein Relikt dazukommt, ersetzt es bei vollen Plätzen ein gehaltenes; das alte wird für den halben Preis verkauft (wie beim Verkauf am Tresen, `RelicSold`, die Stammtischkasse zahlt vorher aus). Überall derselbe Ablauf: erst das neue Relikt, dann das, das Platz macht.
  - **Tresen**: Statt „Kaufen“ steht „Tauschen“; ein Blatt listet die gehaltenen Relikte mit ihrem halben Preis. `RunAction::ReplaceRelic { slot, replace }` verkauft und kauft in einem Schritt (alles oder nichts); der Erlös zählt für den Preis. Sicht `RelicOffer::replace` (die gehaltenen Relikte, die das Angebot jetzt ersetzen kann). Verkaufen allein bleibt möglich.
  - **Boss-Belohnung** (auch die Nachtschwärmer der Nacht): wie seit ADR 0017, `ChooseBossRelic { offer, replace }`, Hinweis „Alle Plätze sind belegt“.
  - **Begegnungen mit einem Relikt** (Beute der Spelunke): Bei vollen Plätzen braucht „Mitnehmen“ ein gehaltenes Relikt als Ziel (`Run::option_needs_relic`), dasselbe Auswahlblatt wie beim Alten Bekannten. Die Wahl „Auszahlen lassen“ bleibt.
  - **Start-Relikte**: höchstens acht (`RunError::NoFreeSlot`).
- **Bot** (Messwerkzeug): Bei vollen Plätzen ersetzt er sein schwächstes Relikt (`relic_value`) nur, wenn das neue mindestens `SWAP_MARGIN` (3) mehr wert ist, am Tresen mit `ReplaceRelic`, bei der Boss-Belohnung und in der Begegnung genauso; sonst nimmt er die Groschen oder lässt es.
- **Oberfläche**: Die Relikt-Leiste im Kopf zeigt die gehaltenen Relikte und dahinter einen Zähler der Plätze („1/8“); freie Plätze werden seit dem 11.11.2026 nicht mehr gezeichnet (16.3). „Deine Relikte (n/8)“ am Tresen.
- **Spielstände**: Keine neue Version. Ein alter Spielstand mit mehr als acht Relikten behält sie; ein neues kommt dann nur noch im Tausch dazu.
- **Stammtischkasse**: Verlässt das Relikt den Spieler (Verkauf am Tresen, Tausch, Tausch oder Verkauf beim Alten Bekannten (13.0), Ersatz bei der Boss-Belohnung, Ende einer Abendregel), wird zuerst ausgezahlt, was in der Kasse liegt, so wie es ist (ohne die Hälfte obendrauf); Ereignis `StashPaidOut`, in der Oberfläche „Stammtischkasse: N Groschen ausgezahlt“. Allgemein: Jeder Topf, in den das Relikt mit `StashCoins` einzahlt und in den kein anderes gehaltenes Relikt mehr einzahlt.

### 8.1 Hausregeln (passive Relikte, Joker-Äquivalent)

Hausregeln sind regionale Skat-Varianten, als passive Modifikatoren. Bis 03.11.2026 fünf Plätze im Grundzustand, bis 05.11.2026 unbegrenzt, seitdem acht (8.0). Seit dem 03.11.2026 gibt es den Pechvogel (+1 Grundwert je verfluchter Karte) und am Tresen die Hehlerware (8.8).

**Seit dem Gegenstands-Review vom 02.11.2026 (8.5)** sind Bockrunde, Kontra, Ouvert-Bonus, Durchmarsch, Blindflug, Lehrgeld, Auf dem Trockenen, Schneiderbock, Doppelter Boden, Glashaus, Kerbholz, Siebenschläfer, Saubere Weste, Altes Blatt, Wechselbalg, Bubenparade und Lockvogel entfernt; Drei drücken und Spickzettel sind geändert. Was hier über sie steht, ist Geschichte. **Seit den Relikten v4 (07.11.2026, 8.12)** wirken Treue Seele, Spickzettel, Bierdeckel und Spelunkenkenner neu; ihre älteren Beschreibungen in 8.1, 8.5 und 8.11 sind ebenfalls Geschichte.

| Hausregel | Effekt | Hebel |
|---|---|---|
| **Bockrunde** | Nach jedem verlorenen Spiel: nächstes Spiel zählt doppelt | Grundwert ×2 |
| **Kontra** | Gegner dürfen vor dem ersten Stich Kontra sagen (Spielwert ×2). Du darfst Re sagen (×4). KI sagt Kontra, wenn sie dein Blatt für schwach hält. | Mult, Risiko |
| **Spitze** | Letzter Stich mit einer Sieben gibt +1 Stufe | Stufen |
| **Schenken** | Haben die Gegner nach 5 Stichen unter 15 Augen, geben sie auf. Spiel gilt als Schneider. | Tempo, Schneider |
| **Ouvert-Bonus** | Ouvert zählt +2 statt +1 | Stufen |
| **Schieben** | In jedem Spiel einmal mehr einmischen (seit dem Einmischen, 01.11.2026; vorher einmal pro Serie neu geben lassen) | Spitzen, Risikostufe |
| **Jungfrau** | Gewinnst du mit 61 bis 70 Augen, zählt das Spiel trotzdem als Schneider | Schwelle |
| **Alte Dame** | Deine Damen zählen 5 Augen (seit 04.11.2026 nur die eigenen, 7.1 Regel 7) | Augen |
| **Tournee** | Jede Serie, die du mit Spielen übrig gewinnst, gibt +1 Grundwert für den Rest des Runs | Skalierung |
| **Durchmarsch** | Gewinnst du Schneider, zählt der Spielwert doppelt | Spielwert, Schneider |
| **Blindflug** (Null) | Gewonnenes Null aus der Hand: +1 Stufe | Stufen |
| **Kleinvieh** | Deine Sieben und Achten zählen 1 Auge (seit 04.11.2026 nur die eigenen) | Augen |
| **Ein Auge zudrücken** | Gewinn ab 58 Augen | Schwelle |
| **Lehrgeld** | Jedes verlorene Spiel: +2 Grundwert für den Rest des Runs | Grundwert (Zähler) |
| **Bierdeckel** | Jedes gewonnene Handspiel: +1 Grundwert für den Rest des Runs | Grundwert (Zähler) |
| **Auf dem Trockenen** | Gegner nach drei Stichen ohne Auge: +1 Stufe | Stufen |
| **Schneiderbock** | Nach einem gewonnenen Schneider zählt das nächste Spiel doppelt | Grundwert ×2 |
| **Glückssträhne** | Ab dem dritten Sieg in Folge zählt jedes gewonnene Spiel doppelt, bis eines verloren geht | Spielwert ×2 |
| **Nullachtfuffzehn** (Null) | Jedes Null: Grundwert +8, gewonnen wie verloren | Grundwert |
| **Doppelter Boden** (Null) | Gewonnenes Null mit Skat: +1 Stufe | Stufen |
| **Dicke Sau** (Null) | Deine Daus zählen 16 Augen (seit 04.11.2026 nur die eigenen); wirfst du sie im Null ab, kommen die Gegner schneller in den Kurzschluss | Augen |
| **Glashaus** (Null) | Gewonnener Null Ouvert zählt doppelt | Spielwert ×2 |
| **Kerbholz** (Null) | Jedes gewonnene Null: +2 Grundwert für den Rest des Runs | Grundwert (Zähler) |
| **Skatpfleger** (M8) | Die Augen der gedrückten Karten zählen doppelt | Augen |
| **Altes Blatt** (M8) | Jede gedrückte Karte gibt bis zum Ende der Serie +1 Grundwert | Grundwert (Zähler der Serie) |
| **Allrounder** (M8) | Je verschiedener Spielart in einer Serie +3 Grundwert | Grundwert (Zähler der Serie) |
| **Stammfarbe** (M8) | Dreimal hintereinander dieselbe Trumpffarbe: ×1,5 | Spielwert ×1,5 |
| **Wechselbalg** (M8) | +1 Stufe, wenn du eine andere Spielart ansagst als im Spiel davor | Stufen |
| **Bubenparade** (M8) | Jeder Unter, den du ausspielst: +1 Grundwert | Grundwert |
| **Luschenkönig** (M8) | Gewinnt eine Sieben einen Stich: +1 Stufe | Stufen |
| **Stichfest** (M8) | Dein erster Stich zählt für die Schneider-Grenze doppelt | Schneider |
| **Glasauge** (04.11.2026) | Zu Beginn jedes Spiels wird je eine zufällige Karte jedes Gegners durchsichtig, nur für dieses Spiel | Information (Zählen) |

**Glasauge** (`glass_eye`, Wunsch des Nutzers vom 04.11.2026, Ausgestaltung von Claude): Effekt `RevealDefenderCards(1)`, passiv. Bei jedem Geben (auch nach dem Einmischen) zieht der Kern je Gegner eine Karte seiner Hand und macht sie für dieses Spiel durchsichtig (`ActiveGame::transparent_for_game`, Ereignis `CardMadeTransparent { source, card, seat }`). Die Ausgaben bleiben unberührt, die Markierung gilt nur für dieses Spiel. Der Spieler sieht die Karte mit ihrem Halter in der Ausgabe des Gegners, wie eine durchsichtige Karte (Sicht `revealed`, Quelle `CardMod("transparent")`), bis sie gespielt ist; Wertvorschau, Zähler und die Stichproben des PIMC-Spielers (`opponents::player_hints`) zählen sie als bekannt. Gezogen mit eigenem Zufall aus dem Seed des Spiels und der Zahl der Einmischen (`TRANSPARENT_SEED`), unter den Handkarten nach Id sortiert, linker Gegner zuerst; das Geben und alle anderen Ziehungen bleiben gleich. Ungewöhnlich, 6 Groschen, von Anfang an verfügbar, Bot-Wert 3 wie der Kiebitz: alles vorläufig nach Augenmaß, `balance` misst es, sobald die Simulationen wieder laufen. Bierdeckel: ein Auge mit Monokel.

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
| Mit Vieren (M5) | 29,3 / 6,1 / 0,7 | – | mit oder ohne 4 und mehr: +1 Stufe; Bedingung `MatadorsAtLeast` |
| Siebenschläfer (M5) | – | 28,4 / 8,6 / 2,4 | gewonnenes Null mit unterlaufener Sieben: Grundwert +5; Bedingung `SevenDucked` |
| Saubere Weste (M5) | – | 27,0 / 7,2 / 1,7 | gewonnenes Null, nie bedient: +2 Stufen; Bedingung `NeverFollowedSuit`. Der Bot erreicht das praktisch nie, für Menschen gedacht (Fehlfarben drücken) |
| Der Opa (M5) | 29,3 / 5,8 / 0,5 | – | der Bot spielt fast nie Hand |

M5-Werte gemessen mit `skat_sim --seed 42 --runs 3000 --no-shop --relic <id>` gegen 29,2 / 5,9 / 0,5 % (Alleinspieler) und 27,0 / 7,2 / 1,7 % (Nullspieler) ohne Hausregel.

Die Null-Hausregeln wirken im Bot schwächer, weil der Nullspieler-Bot vor allem an seinen Händen scheitert (43 % gewonnene Nullspiele). Im vollen Run mit Tresen (3 × 3000 Runs, Seeds 42 bis 44) ändert sich gegenüber den 17 Hausregeln aus M4 wenig: Alleinspieler Abschnitt 2 / 3 / 4 erreicht 33,1 / 11,3 / 3,3 % vorher, 33,0 / 11,5 / 3,2 % nachher, Nullspieler 28,6 / 8,6 / 2,7 % und 28,3 / 8,3 / 2,6 %. Der volle Pool verdünnt die starken Hausregeln (Bockrunde, Alte Dame), die Stufen-Hausregeln sind Rare und kommen selten an den Tresen. Für die Kalibrierung der Kurve in M5 (`balance`) heißt das: Die Stufen-Hausregeln existieren jetzt, ihre Seltenheit und ihre Preise sind die Stellschrauben. Umgesetzt in M5: Preise und Seltenheit nach Wirkung, siehe 12.

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

Einzeln gemessen wie Welle 2a (`skat_sim --runs 500 --no-shop --bot pimc:12:3 --relic <id>`, Seeds 42 bis 44; Spickzettel auf den Zielwerten vor PR #47, ohne Relikt 40,6 / 11,9 / 3,5 %; Drei drücken auf den neuen), Bericht `docs/balance/2026-11-02-relikte-welle2.md`. Seltenheit und Preise sind Platzhalter für `balance`:

| Hausregel | Abschnitt 2 / 3 / 4 | Δ Abschnitt 3 | Seltenheit, Preis | Anmerkung |
|---|---|---|---|---|
| Drei drücken (1×/Serie) | 46,3 / 16,8 / 5,5 | +7,9 | Rare, 9, gesperrt | gemessen auf den neuen Zielwerten (PR #47, ohne Relikt 34,1 / 8,9 / 2,7); in jedem Spiel waren es +18,0 (alte Zielwerte), die Variante „kleinste Karte des Gegners“ +26,4. Eine Karte wechselt die Seite |
| Spickzettel | 40,8 / 12,3 / 3,5 | +0,3 | Common, 4 | nur Handspiele (beim Bot 6 % der Spiele); für Menschen |

**Hausregeln mit eigener Ausgabe (ADR 0015, ausgelegt von Claude am 02.11.2026).** Hausregeln sind Regeln des Tisches und gelten für jede Ausgabe (7.1, Regel 7): ~~Alte Dame, Kleinvieh und Dicke Sau zählen die Augen jeder Karte, wer sie auch hält.~~ Seit dem 04.11.2026 (Entscheidung des Nutzers) zählen Alte Oberin, Kleinvieh und Dicke Sau nur die Karten der Ausgabe des Spielers, wo sie auch landen; die Messungen unten stammen von vorher und überschätzen sie (etwa 12 von 32 Karten hält der Spieler), neu zu messen, sobald die Simulationen wieder laufen. Skatpfleger zählt die Skatkarten in der Ausgabe des Spielers. **Drei drücken**: Die Karte, die vom linken Gegner kommt, ist in der Hand des Spielers seine Ausgabe (mit seinen Modifikationen); die Karte, die zurückgeht, ist beim Gegner wieder dessen Ausgabe. „Die wenigsten Augen“ (`rules::returned_discard`) misst an den drei Karten, wie der Spieler sie hält, also in seiner Ausgabe.



### 8.2 Stammtisch-Figuren (passive Relikte mit Persönlichkeit)

**Seit 02.11.2026 (8.5)** sind Die Oma, Der Opa, Der Schiedsrichter, Der Angeber und Der Stammgast entfernt, neu sind Deckel anschreiben und Spelunkenkenner.

| Figur | Effekt |
|---|---|
| **Der Kiebitz** | Zeigt dir vor der Ansage eine zufällige Karte eines Gegners |
| **Der Wirt** | Tresen-Preise −20 % |
| **Die Oma** | Einmal pro Serie darfst du einen gedrückten Buben zurücknehmen |
| **Der Schiedsrichter** | Gegner dürfen nicht schummeln (hebt Boss-Mechaniken teilweise auf) |
| **Der Angeber** | +1 Stufe bei Ouvert, aber du musst immer Ouvert spielen |
| **Der Zähler** | Zeigt dir laufend die Augen, die noch im Spiel sind |
| **Der Stammgast** | Zählt die Serien, die du mit ihm gewinnst. Ab der vierten zählt jedes gewonnene Spiel doppelt |
| **Der Schneider** | Nimmt großzügig Maß: Gewinnst du mit 80 bis 89 Augen, sind die Gegner Schneider |

Umgesetzt in M4 (`house_rules.ron`, `kind: Regular`, Preise 6 bis 8 als Platzhalter für `balance`). Auslegungen, entschieden von Claude am 01.11.2026:

- **Kiebitz** (`kibitzer`, Effekt `PeekDefenderCards(1)`): eine zufällige Karte eines Gegners und wer sie hält, gezogen mit dem Zufall des Spiels (das Geben bleibt gleich). Sie steht in der Sicht unter `revealed` und kommt als Ereignis `CardPeeked`, sonst nichts. Seit ADR 0015 zeigt er die Karte in der Ausgabe des Gegners (beim Boss mit dessen Modifikationen); ebenso der Zaungast (9.5).
- **Wirt** (`landlord`, `ShopDiscount(20)`): gilt für Relikte, Verbrauchsgegenstände und Neu auslosen (bis 05.11.2026: die Neuauslage), gerundet, mindestens 1 Groschen. Verkauft wird weiter zum halben vollen Preis, also kein Gewinn durch Kaufen und Verkaufen.
- **Oma** (`granny`, `SeriesSkatSwap`): Ein gedrückter Bube lässt sich im echten Skat nicht sinnvoll zurücknehmen, weil der Spieler beim Drücken alles sieht. Umgesetzt als: Einmal pro Serie darf der Spieler in einem Handspiel vor seiner ersten Karte einen Buben aus dem Skat nehmen und dafür eine Karte drücken; das Spiel bleibt Hand. Ob ein Bube im Skat liegt, verrät die Oma (`can_take_from_skat`), seit M5 auch welcher (`skat_swap_cards`). Hebel: Hand plus Trumpf. Nur Alleinspieler.
- **Opa** (`grandpa`, M5, `SeriesSkatSwap` für Daus): wie die Oma, aber ein Daus aus dem Skat. Oma und Opa zusammen geben zwei Täusche pro Serie, je einen (4.4).
- **Schiedsrichter** (`referee`, `NoCheating`): schaltet die Boss-Regeln ab, die als Schummeln markiert sind: die Zwillinge (bis ADR 0017 auch der Revolutionär).
- **Angeber** (`show_off`, `RequireOuvert` plus eine Stufe bei Ouvert): Für den Alleinspieler hieße „immer Ouvert“ immer Schwarz angesagt, das wäre Selbstmord. Deshalb nur im Pool des Nullspielers (Null Ouvert und Null Ouvert Hand).
- **Stammgast** (`regular_guest`, Zähler `regular` plus `MultiplyGameValue(2)` ab 4): Hebel Spielwert. Der Faktor fürs Spätspiel, wer ihn früh kauft, wartet zwei Abende. Nur gewonnene Spiele zählen doppelt.
- **Schneider** (`tailor`, `SchneiderFromDeclarerPoints(80, 89)`): Hebel Schneider, ein Wortspiel mit dem Handwerk. Nur Alleinspieler. Er kam statt des zuerst geplanten Opa, den der Kern in M4 nicht neben der Oma tragen konnte; seit M5 gibt es beide.
- **Zähler** (`card_counter`, `ShowCount`): zeigt Augen beider Seiten aus den Stichen (plus den Skat, sobald der Spieler ihn kennt) und die Summe der Karten, die der Spieler noch nicht gesehen hat. Der Talon zählt dazu, weil niemand weiß, welche Karten dort liegen; so verrät der Zähler nie mehr als ein perfekter Mitzähler weiß. Seit ADR 0015 gibt es keinen Talon; jede ungesehene Karte liegt bei einem der beiden Gegner. Hat der Boss eine Ausgabe mit Augen-Modifikationen, hängt die Summe davon ab, wer die Karte hält: Der Zähler zeigt dann den kleinsten und den größten möglichen Wert („noch 34 bis 38“), damit er nicht mehr verrät als ein Mitzähler weiß.

### 8.3 Verbrauchsgegenstände

Einmal nutzbare Karten, maximal zwei auf der Hand (seit dem 03.11.2026 drei, 8.9).

**Seit dem 03.11.2026 (8.9)** bringt jede gewonnene Kneipe einen zufälligen Schnaps, und es gibt drei Schnaps-Plätze.

**Seit 02.11.2026 (8.5)** sind Ein Bier, Neues Blatt, Magenbitter, Reue, Augen +5 und Trumpf +4 entfernt; Kräuter und Doppelkorn wirken anders, Kreide heißt Blue Curaçao, neu sind Konterbier, Schwenker und Wegbier. Was hier über die entfernten steht, ist Geschichte.

**Seit dem 03.11.2026 (8.7)** gibt es drei Rettungs-Schnäpse für eine Kneipe im Minus: Klarer, Absacker und Eierlikör.

**Seit dem 03.11.2026 (8.8)** gibt es den Fusel: das nächste Spiel doppelt, dafür ein Fluch für immer.

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
- **Bot** (`run/bot/`, einfache Regeln, nicht kalibriert): Kurzer direkt vor dem Aufnehmen des Skats (der Bot nutzt das Gesehene nicht), nach dem Skat Blue Curaçao auf die billigste Beikarte in die Trumpffarbe eines Farbspiels, dann Doppelkorn vor einem nicht schwachen und Magenbitter vor einem schwachen Spiel, nie vor einem Null; Kräuter wie Ein Bier. Reue nutzt der Bot nie (Kaufwert 0).
- **Reue** (Name „Reue“ gewählt: kurz und sagt, was er tut). Im Kern hält `ActiveGame::previous` das Spiel vor dem letzten Neugeben vollständig (es wird nicht neu gezogen, also deterministisch und auch nach Speichern und Laden gleich); `ConsumableAction::UndoRedeal` setzt es wieder ein, Ereignis `RedealUndone`. Was das Geben selbst ausgelöst hat (Zähler, Flaggen von `OnDeal`), bleibt; heute hat kein Inhalt so einen Hook. Der Zufallsstrom läuft weiter: Wer nach der Reue noch einmal einmischt, bekommt ein neues Blatt, nicht das zurückgenommene. Kaufwerte in `CONSUMABLE_VALUES` sind Platzhalter.

### 8.4 Stufe Legendär (M8 Welle 3)

Eine vierte Seltenheit für Relikte, die den Run prägen (`Rarity::Legendary`, Anzeige „legendär“). Sie bestimmt nur, wie selten das Relikt am Tresen liegt, nicht den Preis (entschieden am 02.11.2026: Preise nach der gemessenen Wirkung, Staffel aus Welle 2a). Am Tresen weit seltener als Rare: Gewicht `legendary` in `run.ron` als Dezimalzahl neben Common 6, Uncommon 3, Rare 1, Standard 0,3 (ohne Eintrag in `run.ron`, `RarityWeights::percent_weight`). Ein bestimmtes legendäres Relikt liegt damit beim Alleinspieler in etwa 0,6 % der Auslagen. Auch der Stammtisch-Streit und der Alte Bekannte ziehen mit diesem Gewicht. Auf dem Bierdeckel ein goldener Ring mit vier Funken. Preise Platzhalter für `balance`. Entschieden am 02.11.2026:

| Relikt | Effekt | Hebel | Preis |
|---|---|---|---|
| **Skatgott** (`skat_god`) | Jedes gewonnene Spiel mit Schneider: bis zum Run-Ende +1 Stufe auf jedes gewonnene Spiel, höchstens +3 | Stufen (Zähler `skat_god`) | 12 (bis 03.11.2026 9) |
| **Ehrenvorsitz** (`honorary_chair`) | Du bist immer Vorhand | Stichgewinn (Vorhand) | 6 |

- **Skatgott**: Der Zähler steigt nach jedem gewonnenen Spiel mit Schneider (auch Schneider durch Jungfrau, Schenken, Stichfest oder Schwarz). Die Stufen gelten nur für gewonnene Spiele, jede Spielart (Farbspiel, Grand, Null), nie der Ramsch (`AddLevelsPerCounter` bei der Wertung; der Stufen-Hook steht vor dem Zähler, ein Schneiderspiel zählt seinen eigenen Punkt also nicht). Entschieden am 02.11.2026: „auf alles“ heißt „auf jede Spielart“; die erste Fassung zählte auch verlorene Spiele und brachte dem Bot nur +1,3 bei weniger Punkten je Spiel. Abgestimmter Text ergänzt zu „… +1 Stufe auf jedes gewonnene Spiel“ (vom 02.11. bis 04.11.2026 „+1 Multiplikator“). **Seit 03.11.2026 höchstens +3** (Entscheidung von Claude nach `docs/balance/2026-11-03-kleine-runde.md`: allein +12 Punkte „Run gewonnen“, alle anderen Relikte höchstens +4): Der Zähler steigt nur, solange er unter 3 steht (`Not(CounterAtLeast(counter: "skat_god", value: 3))`, reine Daten).
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
- **„Multiplikator“ statt „Stufe“/„Gewinnstufe“** in allen Spieltexten (Content, Übersetzung, Abrechnung, Regelkarte, Glossar). Im Code bleiben `levels`, `AddLevels`, `Lever::Levels`. „Sticht eine Stufe höher“ (Rang) und die Risikostufe bleiben. **Seit 04.11.2026 zurückgenommen** (Cockpit q23): Die Spieltexte sagen wieder „Stufen“, siehe 4.2; die Rang-Texte der Zinken sagen dafür „Rang“.
- **Anzeige-Gruppe** als Daten: `group` an jedem Relikt (`Value`, `Multiplier`, `Control`, `Tour`, Standard `Value`), in Sicht und Katalog. Die Web-App liest sie, die Form des Bierdeckels folgt noch der Art (Hausregel, Figur); die Form nach Gruppe kommt mit dem neuen Bierdeckel-Entwurf.
- **Kartenwaren** (Augen +3, Trumpf +2, Rang +1, Stempelkissen) blieben zunächst, wie sie sind. Die geplanten Kartenwaren mit Schnapsnamen (Goldwasser, Trumpfbrand, Absacker, Anstich, Pfandbon, Totenschein, Narrenkappe) sind **nicht** gebaut: Kartenverbesserungen bekommen eine eigene Tresen-Mechanik (Nachentscheidung des Nutzers). **Seit 8.6 überholt**: Die Kartenwaren sind entfernt, die Mechanik heißt Zinken.

**Neue Schnäpse** (`consumables.ron`, alle `category: Schnapps`):

| Schnaps | Id | Wirkung | Wann | Seltenheit, Preis (vorläufig) |
|---|---|---|---|---|
| **Konterbier** | `counter_beer` | **entfällt seit ADR 0019** (gab +1 Leben, `GainLives(1)`); alte Spielstände bekommen seinen Preis zurück (`REFUNDED_CONSUMABLES`, 4 Groschen) | – | bis dahin Uncommon, 4 |
| **Schwenker** | `snifter` | Die Stationen der drei Wege werden neu ausgelost (`RerollRoutes`); eine Spelunke bleibt, wo sie ist, ein Tresen bleibt erreichbar | bei der Routenwahl | Common, 2 |
| **Wegbier** | `road_beer` | Nächste Kneipe: ein Einmischen mehr (für die ganze Kneipe) und Zielwert −3 (`NextPub(redeals: 1, target: -3)`, nie unter 1) | zwischen zwei Kneipen (Routenwahl, Tresen, Ereignis) | Common, 3 |

**Neue Relikte** (`house_rules.ron`, Gruppe Tour):

| Relikt | Id | Wirkung (Daten) | Seltenheit, Preis (vorläufig) | Freischaltung |
|---|---|---|---|---|
| **Feierabend** | `closing_time` | Kneipe im ersten Spiel geschafft: +4 Groschen (`OnSeriesWon` mit `GamesLeftAtLeast(2)`, `GainCoins(4)`) | Common, 4 (bis 03.11.2026 5) | frei |
| **Monatskarte** | `season_ticket` | Jede Bahn: +1 Groschen (neuer Auslöser `OnRouteChosen`, Bedingung `TransportIs(Train)`) | Common, 4 | Neues Viertel |
| **Stammtischkasse** | `kitty` | Von jeder Belohnung einer Kneipe 2 Groschen in die Kasse (`StashCoins`, Zähler `kitty`, nach der Auszahlung, höchstens was da ist); beim nächsten Tresen alles plus die Hälfte, abgerundet (neuer Auslöser `OnShopOpened`, `PayOutCounter(percent: 150)`) | Uncommon, 5 | Dickes Ding |
| **Deckel anschreiben** (Figur) | `slate` | Einmal pro Stadtteil darf der Tresen bis 5 Groschen ins Minus verkaufen (`Credit(5)`); spätere Einnahmen zahlen zuerst die Schulden | Uncommon, 5 | Omas Liebling |
| **Schutzengel** | `guardian_angel` | Seit ADR 0019 die **zweite Chance**: einmal im Run wird eine verlorene Kneipe neu gespielt, auch eine Spelunke, eine aufgegebene und die Boss-Kneipe; danach ist er weg (`SecondChance`, Ereignis `SecondChance`). Bis dahin kostete die erste verlorene Kneipe eines Stadtteils kein Leben (`SpareLife`, der alte Name wird weiter gelesen) | Rare, 12 (seit Runde 3, 05.11.2026: ohne Tresen +34,8 auf „Stadtteil 2“, die stärkste Einzelwirkung, Runde 2; bis dahin 8, am 03.11.2026 kurz 6) | Ohne Einmischen |
| **Spelunkenkenner** (Figur) | `dive_expert` | In einer Spelunke zwei Mali zur Wahl (`ChooseMalus(2)`, Aktion `ChooseMalus`) | Uncommon, 4 | Lehrjahre |

Auslegungen von Claude:

- **Deckel anschreiben**: „Einmal pro Stadtteil“ heißt: der erste Tresen-Besuch eines Stadtteils, in dem der Spieler ins Minus kauft, öffnet den Deckel; in **diesem** Besuch darf er weiter bis −5 kaufen (auch Neu auslosen), spätere Besuche desselben Stadtteils nicht mehr. Der Stadtteil ist der, in den der Weg führt. Im Kern bleiben die Groschen bei 0, die Schulden stehen daneben (`Run::debt`, Sicht `debt`, `credit_left`); jede Einnahme (Auszahlung, Verkauf, Groschen aus Relikten, Ereignissen, Flüchen, Gutscheinen, Kasse, Wette) zahlt zuerst die Schulden (Ereignis `DebtChanged`). Zinsen rechnen mit den Groschen, nicht mit den Schulden. Ereignisse kosten nie auf Pump.
- **Schutzengel**: seit ADR 0019 einmal im Run, in jeder Kneipe einschließlich der Boss-Kneipe, dann verbraucht. Bis ADR 0017 wurde eine verlorene Boss-Kneipe mit ihm ohne Lebensverlust wiederholt, von ADR 0017 bis ADR 0019 half er dort nicht.
- **Spelunkenkenner**: Die zwei Mali werden mit der Route gezogen und angezeigt; gilt der erste, bis der Spieler einen anderen wählt (`RunAction::ChooseMalus { option, malus }` bei der Routenwahl).
- **Stammtischkasse**: Die Kasse zahlt beim nächsten Tresen, egal ob dazwischen eine Kneipe lag; ohne Tresen wächst sie weiter. Seit 03.11.2026 (Entscheidung des Nutzers): Wird die Kasse verkauft oder anders entfernt, zahlt sie vorher aus, was drin liegt, ohne die Hälfte obendrauf (`StashPaidOut`, 8.0).

**Neue Begegnungen** (`encounters.ron`, Kategorie Begegnung, Werte vorläufig):

| Begegnung | Id | Wahl |
|---|---|---|
| **Der Wirt spendiert** | `landlord_treat` | einer von drei gezogenen Schnäpsen gratis in einen freien Platz, oder 5 Groschen |
| **Falschspieler am Nebentisch** | `card_sharp` (ersetzt den alten Falschspieler, Id bleibt) | für 2 Groschen eine starke Modifikation (eine von Rang +1, Trumpf +4, Augen +6, beim Aufgehen gezogen und genannt) auf eine gewählte Karte; mit 50 % landet stattdessen ein zufälliger Fluch, der wie jeder Fluch 3 Groschen zahlt |
| **Verlorenes Portemonnaie** | `lost_wallet` | behalten: +8 Groschen und Durchsichtig auf einer zufälligen Karte (der Fluch zahlt zusätzlich die üblichen 3); abgeben: ein Schnaps (bis ADR 0019 +1 Leben) |
| **Der Skatlehrer** | `skat_teacher` | Rang +1 auf eine gewählte Karte gratis; im ersten Spiel der nächsten Kneipe ein Einmischen weniger |
| **Die Wette** | `the_bet` | 3 oder 6 Groschen auf das nächste angesagte Spiel: gewonnen mit Schneider zahlt doppelt, sonst weg; der Ramsch zählt nicht, er wartet |
| **Alte Karten im Schrank** | `old_cards` | ein Gutschein (Untot oder Wandelkarte, beim Aufgehen gezogen) auf eine gewählte Karte, ein Fluch (gezogen und genannt) auf eine zufällige andere |
| **Kiebitz am Fenster** | `lookout` | die genauen Stationen der nächsten zwei Routenwahlen sehen, oder ein Einmischen mehr in der nächsten Kneipe |

Auslegungen von Claude: „Nächste Kneipe, ein Einmischen weniger“ (Skatlehrer) heißt **im ersten Spiel** der Kneipe eins weniger (`Series::redeals_withheld`), weil Einmischen je Spiel zählt und drei weniger zu hart wäre; „ein Einmischen mehr“ (Wegbier, Kiebitz) geht wie früher Ein Bier in den Vorrat der Kneipe. Der Plan für die nächste Kneipe (`NextPubPlan`) summiert sich und wird beim Start der Kneipe angewandt (Ereignis `NextPubApplied`); die Routenwahl zeigt ihn und den geänderten Zielwert. Die Wette wird bei der Wertung des nächsten nicht-Ramsch-Spiels entschieden; Null kann nie Schneider sein und verliert sie. Ereignisse mit Fluch sagen „Fluch“ im Text (beim Laden geprüft). Der Geburtstag am Stammtisch kommt später.

**Bot** (`run/bot/`, Regeln, nicht kalibriert): Kaufwerte Kräuter 1, Doppelkorn 1 (seit 03.11.2026, vorher 2: rund die Hälfte blieb ungetrunken, `docs/balance/2026-11-03-kleine-runde.md`), Konterbier 2, Wegbier 2, Schwenker 0 (nie gekauft); Feierabend 3, Monatskarte 2, Kasse 3, Deckel 2 (er kauft nie auf Pump), Schutzengel 7, Spelunkenkenner 1. Bei der Routenwahl trinkt er Wegbier sofort (Konterbier bis ADR 0019 mit fehlendem Leben) und den Schwenker, wenn kein Weg Upgrade oder Rast bei verfluchter Karte (bis ADR 0019 auch Rast bei höchstens zwei Leben) bietet; als Spelunkenkenner wählt er Schnapsverbot vor Klebrigen Karten, Kein Einmischen, Kein Skat. Doppelkorn nutzt er wie Kräuter, um ein schwaches Blatt ohne übriges Einmischen loszuwerden. Begegnungen: Schnaps vom Wirt nach Kaufwert (sonst die Groschen), Portemonnaie behalten (Wert 4) oder abgeben bei fehlendem Leben, Skatlehrer auf ein Daus, Kiebitz für das Einmischen; Falschspieler, Wette und Alte Karten lehnt er ab.

Tests: `core/src/run/items_tests.rs` (je Gegenstand einer, dazu Entfernung und Laden alter Spielstände), `wasm` lädt die Fixtures mit entfernten Relikten und spielt weiter.

### 8.6 Zinken am Tresen (02.11.2026)

**Entschieden vom Nutzer am 02.11.2026, maßgeblich ab hier.** Kartenmodifikationen sind keine Verbrauchsgegenstände mehr. Der Tresen verkauft sie unter dem Namen **Zinken** (Verb in der Oberfläche: „Karte zinken“, eine einzelne heißt Zinke). Alle Zahlen dieses Abschnitts sind **vorläufig**, nach Augenmaß ohne Messung (die Simulationen ruhen); `balance` misst, sobald der Simulator wieder läuft. Daten: `zinken` in `run.ron` (`ZinkenConfig`), `price` und `shop_value` in `card_mods.ron`; Code `core/src/run/zinken.rs`, Tests `core/src/run/zinken_tests.rs`.

**Namen.** Aufdruck (`points`), Winkel (`rank`), Siegel (`trump_bonus`), Eselsohr (`first_trick`), Kerbe (`last_trick`), Pfandmarke (`voucher_coins`), Totenmarke (`voucher_undead`), Narrenmarke (`voucher_wild`); seit Zinken v2 (03.11.2026, 7.2) Blei (`lead`), Goldkante (`gilt_edge`), Fadenkreuz (`crosshair`, seit 12.11.2026 vorerst nicht mehr angeboten, 7.2), Schneiderkreide (`tailors_chalk`) in Auslage und Kartenmaler, der Doppelgänger (`doppelganger`) als seltenes Spezialangebot. Flüche und Spezialkarten behalten ihre Namen. Infokarten zeigen immer die Wirkung neben dem Namen: Jeder Text beginnt mit ihr („Eselsohr“ – „Erster Stich: Gewinnt diese Karte den ersten Stich, zählen seine Augen ×2.“).

**Die Auslage.** Jeder Tresen legt **drei Angebote** aus (`zinken.offers`; in Runde 3, 05.11.2026, auf **zwei** gesenkt, weil die Kartenbelohnung rund vier Zinken je Run gratis bringt, `docs/balance/2026-11-05-runde3.md`; seit dem Playtest vom 05.11.2026 wieder **drei**, Entscheidung des Nutzers, vorläufig und ungemessen), jedes eine bestimmte Karte der Ausgabe des Spielers mit einer bestimmten Zinke und ihrem Wert, vor dem Kauf sichtbar (die Oberfläche zeigt die Karte, wie sie danach aussieht). Gezogen mit einem eigenen Zufallsstrom des Runs (`zinken_rng`), damit die übrigen Tresen-Angebote, Wege und Spiele gleich bleiben.

- **Nie ein Fluch** als Zinke (ein Schnäppchen legt seinen Fluch auf eine andere Karte, 8.8), nie eine Zinke, die nicht auf die Karte passt (`only_on`) oder die die Regeln aus 7.2 ablehnen (verfluchte Karte, Winkel schon bei 2). Eine Karte höchstens einmal je Auslage. Eine Spezialkarte nur als seltenes Angebot (unten); Toten- und Narrenmarke seit 05.11.2026 auch, wenn die Ausgabe ihre Spezialkarte schon hat (7.2, keine Obergrenze mehr).
- **Selten eine Spezialkarte direkt** (seit 03.11.2026, Entscheidung des Nutzers „A + B“, Auslegung von Claude): Jedes Angebot ist mit 5 % (`special_percent`, etwa eines von zwanzig) „wird Untot“ oder „wird Wandelkarte“ auf einer bestimmten Karte (`Run::special_offers`; bis 05.11.2026 nur, solange die Ausgabe diese Spezialkarte noch bekommen durfte, und höchstens eins je Auslage und Spezialkarte samt Marke). Nie auf einer Karte, die schon eine Spezialkarte trägt; die Zinke der Karte bleibt daneben (7.2, Grundregel 1), die Kachel sagt „Aufdruck bleibt“. Gezogen mit demselben Zufallsstrom und denselben Kartengewichten (ohne Modifikation 3, sonst 1), gewürfelt nur, wenn eine solche Spezialkarte möglich ist. Preis 6 Groschen (`special_price`), auf Unter und Daus 7, dann wie jedes Angebot über den Wirt und den Preis des Charakters (`Run::card_offer_cost`: `shop_price`, `zinken_price`). Die Kachel trägt ein Band „Selten“ und zeigt die Karte mit ihrem Spezialbild.
- **Darstellung wie die Kartenbelohnung** (seit 05.11.2026, Entscheidung des Nutzers „3 Zinken zur Auswahl im Shop, ähnlich wie gewonnene Kneipe“): Jedes Angebot zeigt die Karte so, wie sie mit der Zinke aussieht, mit „auf …“ und „steigert …“, „ersetzt …“ oder „… bleibt“. Antippen (die Kachel oder „Kaufen“) öffnet immer das Blatt mit der Karte jetzt und danach; „Kaufen · n Gr.“ kauft, „Anderes Angebot“ geht zurück. Steigern und Ersetzen fragen wie bisher („Steigern“, „Ersetzen“). Preise wie bisher.
- **Karten ohne Modifikation werden bevorzugt**: Gewicht 3 gegen 1 für Karten, die schon etwas tragen (`bare_weight`, `modded_weight`).
- **Gewicht der Zinke**: Seltenheit wie bei den Relikten (gewöhnlich 6, ungewöhnlich 3, selten 1) mal das Gewicht des Charakters für die Spielart (`focus_weights`), **Gutscheine** (Pfandmarke, Totenmarke, Narrenmarke) nur mit der Hälfte (`voucher_percent: 50`).
- **Werte** in der Auslage (`shop_value`, sonst der erste aus `values`): Aufdruck +3, Siegel +2, Winkel +1, Eselsohr ×2, Kerbe +1, Pfandmarke +3.
- **Preis** nach Zinke und Karte: der Preis der Zinke, auf einem Unter oder Daus 1 Groschen mehr (`premium_ranks`, `premium`). Kauf: `RunAction::BuyCardOffer(slot)`, Ereignis `CardOfferBought`, danach `DeckChanged`. Dieselbe Zinke auf der Karte steigt, eine andere wird ersetzt; die Oberfläche fragt vorher wie bisher (Ersetzen-Bestätigung), die Sicht sagt es vorab (`CardOfferView::replaces`, `raises`).
- **Neu auslosen** (seit dem Playtest vom 05.11.2026, Entscheidung des Nutzers): **eine** Aktion für den ganzen Tresen, **einmal je Besuch**: Relikte, Schnäpse und die Auslage werden neu gezogen (`RunAction::Reroll`, Ereignisse `ShopRerolled { cost }`, `ShopOpened`, `CardOffersDrawn`), Preis `economy.reroll_cost` = 2 Groschen, vorläufig, über den Wirt; ein zweites Mal gibt `AlreadyUsed` (`Shop::rerolls`, Sicht `ShopView::reroll_used`). Kartenmaler, Karte reinigen und Deckel bleiben, wie sie sind. Mit der Figur **Der Duzfreund** (`first_names`, Gruppe Tour, Ungewöhnlich, 5 Groschen, vorläufig; Wirkung `FreeReroll`, Text „Du bist mit dem Wirt per Du: Neu auslosen am Tresen kostet dich nichts, weiter einmal je Besuch.“) kostet es nichts, bleibt aber einmal je Besuch: Auslegung von Claude, weil beliebig oft gratis den Tresen leer spielen würde (jeder Besuch zöge so lange, bis das gewünschte Relikt kommt). Der Name statt „Der Stammkunde“, weil ein Meilenstein schon so heißt (`unlocks.ron`). **Bis 05.11.2026** gab es zwei Neuauslosungen: die Neuauslage (Relikte und Gegenstände, beliebig oft zu 2 Groschen) und „Neu auslosen“ der Auslage (1 Groschen, jedes weitere Mal am selben Tresen 1 mehr; `RerollCardOffers`, `CardOffersRerolled`, `zinken.reroll_cost`, `reroll_step`, `Shop::card_rerolls`, entfallen; alte Spielstände lesen `card_rerolls` nicht mehr).

**Die Kartenbelohnung** (seit 04.11.2026, 5.0, Code `core/src/run/card_reward.rs`, Tests `card_reward_tests.rs`; feste Paare seit 05.11.2026). Nach einer gewonnenen normalen Kneipe oder Spelunke drei Angebote aus Karte und Zinke, gezogen mit einem eigenen Zufallsstrom (`reward_rng`), damit Karten, Auslage, Wege und Begegnungen gleich bleiben. Je Angebot erst die **Zinke**: mit 5 % (`special_percent`) eine Spezialkarte (Untot, Wandelkarte, Doppelgänger); sonst erst die Seltenheit nach `rarity_weights` (gewöhnlich 60, ungewöhnlich 30, selten 10, nur Klassen mit einer passenden Zinke; selten sind heute nur Toten- und Narrenmarke), dann eine Zinke daraus nach dem Gewicht des Charakters für die Spielart (`focus_weights`), Marken mit `voucher_percent` (50 %) und dem Gewicht der Bau-Richtung (`Run::card_reward_bias`: seit 05.11.2026 wiegt eine Zinke, die eine Bau-Richtung (`tags`) mit einem Relikt des Spielers teilt, `direction_percent` = 150 %, vorläufig; sonst 100 %). Dann die **Karte**: gleich verteilt unter den Karten, die diese Zinke jetzt nehmen (`only_on`, kein Fluch, nicht am Höchstwert; eine Spezialkarte nur auf eine Karte ohne Spezialkarte) und noch in keinem anderen Angebot stehen; Karten mit derselben Zinke sind dabei (sie steigt). **Keine Karte zweimal, keine Zinke zweimal**, solange eine andere Zinke noch eine freie Karte hat; erst dann darf eine Zinke wiederkommen (auf einer anderen Karte). Auslegung von Claude: die Karte gleich verteilt statt nach den Gewichten der Auslage (ohne Modifikation 3, sonst 1), weil der Nutzer „eine zufällige Karte des Decks“ wollte. Dieselben Regeln wie die Auslage: nie ein Fluch, nur für den Charakter und freigeschaltet. Werte wie in der Auslage (`shop_value`). Bis zur Wahl nimmt der Run nichts anderes an (Fehler `CardRewardPending`); die Wahl übersteht Speichern und Laden, alte Spielstände laden (ohne Zufallsstrom: eine feste Zahl). **Spielstände von vor dem 05.11.2026** mit offener Kartenbelohnung (drei Zinken ohne Karte) werden beim Laden mit dem Zufallsstrom des Runs als Paare neu gezogen (`CardReward` liest beide Formen, `Run::drop_unknown_content` zieht neu); derselbe Spielstand gibt immer dieselben Angebote. **Bot**: nimmt das Angebot mit dem höchsten Wert, die Zinke nach `ZINKEN_VALUES` (mit Bau-Richtung, `zinke_score`) weniger der Zinke, die sie ersetzen würde; bei Gleichstand Steigern vor freier Karte vor Ersetzen, dann die Karte mit den meisten Augen, Unter zuletzt; überspringt (2 Groschen), wenn kein Angebot etwas wert ist. **Simulator**: je Run angeboten, genommen, übersprungen und nach Zinke (Zeile `card rewards`).

**Der Kartenmaler.** Einmal je Tresen: bezahlen, eine Karte wählen (`RunAction::HirePainter(card)`), dann eine von **zwei** gezogenen Zinken nehmen, die auf diese Karte passen (`RunAction::ChoosePaint(i)`). Gleiche Regeln wie die Auslage (nie ein Fluch, Gewichte, Werte), zwei verschiedene. Bis zur Wahl nimmt der Run nichts anderes an (Fehler `PainterPending`); die Wahl übersteht Speichern und Laden. Teurer als ein Angebot der Auslage, weil der Spieler die Karte wählt. Ereignisse `PainterHired`, `PainterChosen`. Kann die Karte keine Zinke nehmen, wird nichts bezahlt (Fehler der Regeln, etwa `CardCursed`, sonst `NoModAvailable`); die Kartenwahl blendet solche Karten mit dem Grund ab.

**Karte reinigen.** Der Tresen-Dienst, der Frisches Deck ersetzt: Einmal je Tresen fallen alle Modifikationen einer gewählten Karte ab, auch Flüche, Spezialkarten und Abgestuft (`RunAction::CleanseCard(card)`, Ereignis `CardCleansed`, danach `DeckChange::Cleansed`; eine Karte ohne Modifikation ist kein Ziel). Auslegung von Claude: einmal je Tresen wie der Kartenmaler, damit ein Tresen nicht alle Flüche auf einmal tilgt.

**Die Werkstatt** (Station Upgrade, 5.0.1) bietet seit den Zinken den **Kartenmaler gratis** (`EncounterChoice::Painter`: Karte wählen, dann eine von zwei Zinken) und **Karte reinigen** gratis; die bisherige „eine gezogene Modifikation auf eine gewählte Karte“ entfällt. Erst die Wahl des Kartenmalers schließt die Begegnung ab.

**Der Wirt** („Alles am Tresen ist 20 % billiger …“) gilt für alles am Tresen: Auslage, Neu auslosen, Kartenmaler und Karte reinigen wie für Relikte und Gegenstände (Auslegung von Claude, Text ergänzt). Gerundet wie bisher, mindestens 1 Groschen. **Deckel anschreiben** gilt ebenso. **Bruderherz** (9.6, Regel des Charakters): Nach dem Wirt gilt sein Preis für Zinken auf bekannten Karten (`Run::zinken_price`, halb auf Rot) für jedes Angebot der Auslage und den Kartenmaler (`Run::card_offer_cost`, `Run::painter_cost`; die Sicht nennt den Preis je Karte in `ServiceView::target_prices`). Karte reinigen läuft über `Run::cleanse`: Seine Flüche bleiben (`CurseStays`), bezahlt wird dann nichts.

**Vorläufige Preise** (Groschen, vor dem Wirt):

| Was | Preis |
|---|---|
| Aufdruck +3, Pfandmarke +3 | 2 |
| Winkel +1, Siegel +2, Eselsohr ×2, Kerbe +1 | 3 |
| Blei, Goldkante +1, Schneiderkreide +1 (seit 03.11.2026) | 3 |
| Fadenkreuz +1 (seit 03.11.2026, seit 12.11.2026 nicht im Angebot) | 4 |
| Totenmarke, Narrenmarke | 4 |
| Untot, Wandelkarte direkt (selten, seit 03.11.2026), Doppelgänger direkt (seit 03.11.2026, Zinken v2) | 6 |
| Zuschlag auf Unter und Daus | +1 |
| Neu auslosen (Relikte, Schnäpse und Auslage, einmal je Besuch, seit 05.11.2026; mit dem Duzfreund gratis) | 2 (bis 05.11.2026 Auslage allein 1, dann 2, 3, …; Neuauslage 2) |
| Kartenmaler | 6 |
| Karte reinigen | 3 |

Begründung: Die Auslage legt die Karte fest, ist also weniger wert als die alten Kartenwaren mit freier Wahl (Augen +3 für 2, Trumpf +2 und Rang +1 für 3); der Kartenmaler mit freier Karte und Wahl aus zwei liegt über dem teuersten Angebot (5); Karte reinigen kostet, was Frisches Deck kostete.

**Entfernt** (Verbrauchsgegenstände): Frisches Deck (`fresh_deck`), Stempelkissen (`ink_pad`), Augen +3 (`points_3`), Trumpf +2 (`trump_2`), Rang +1 (`rank_1`). Die Aktionen `CleanseCard`, `AddCardMod`, `ApplyCardMod` ruhen in der Hook-API; die Definitionen liegen für Tests in `core/src/run/legacy_consumables.ron`.

**Spielstände.** Keine neue Version (wie 8.5): Beim Laden räumt `Run::drop_unknown_content` die entfernten Gegenstände ab (besessen, Tresen-Angebote), dazu Angebote der Auslage und Optionen des Kartenmalers, deren Zinke die Daten nicht mehr kennen. Ein Spielstand von vorher, der am Tresen steht, hat eine leere Auslage; Neu auslosen füllt sie. Der Zinken-Zufallsstrom alter Spielstände kommt aus einer festen Zahl.

**Bot** (`run/bot/`, einfache Regeln, nicht kalibriert, damit der Simulator weiterläuft): Wartet der Kartenmaler, nimmt der Bot die wertvollste Zinke (`ZINKEN_VALUES`: Siegel 3, Winkel 3, Aufdruck 2, Eselsohr 2, Kerbe 1, Pfandmarke 1, Toten- und Narrenmarke 0; seit 03.11.2026 für die seltenen Angebote der Auslage Untot 2, Wandelkarte 1). Am Tresen: erst Karte reinigen auf eine verfluchte Karte, dann Relikte und Schnäpse wie bisher, dann das wertvollste Angebot der Auslage auf einer Karte ohne andere Modifikation (nie ersetzen, Steigern ja), dann den Kartenmaler für einen Daus ohne Modifikation, wenn danach noch 4 Groschen bleiben (`PAINTER_RESERVE`). Nie auf Pump. In der Werkstatt reinigt er eine verfluchte Karte, sonst nimmt er den Kartenmaler für einen Daus. Neu auslosen (seit 05.11.2026): einmal je Besuch, wenn nichts Sinnvolles zu kaufen ist und danach noch 6 Groschen bleiben (`REROLL_RESERVE`; bis dahin höchstens zweimal die Neuauslage). Den Duzfreund wertet er mit 2 (`RELIC_VALUES`, ungemessen).

**Oberfläche.** Der Tresen bekommt eine Reihe „Auslage“ mit drei Angeboten: die Karte mit der neuen Zinke in der Rahmensprache gezeichnet, Name, Wirkung und Preis, Kaufen mit Ersetzen-Bestätigung (seit 05.11.2026 immer mit dem Blatt jetzt/danach, wie die Kartenbelohnung); daneben der Kartenmaler (bezahlen, Karte wählen, eine von zwei wählen) und Karte reinigen. „Neu auslosen“ steht seit 05.11.2026 unten in der Leiste des Tresens neben „Weiter“ („Neu auslosen · 2 Gr.“, mit dem Duzfreund „Neu auslosen · gratis“, danach „Schon neu ausgelost“); die eigene Schaltfläche der Auslage entfällt.

Zu messen, sobald der Simulator wieder läuft: Käufe je Tresen (Auslage, Kartenmaler, Reinigen), Groschen je Run, Wirkung auf „Stadtteil 2 erreicht“ gegenüber den alten Kartenwaren, Anteil der Auslagen ohne passendes Angebot für den Bot.

### 8.7 Rettung einer Kneipe (03.11.2026)

**Entschieden vom Nutzer am 03.11.2026 über das Cockpit** (Varianten A, C und D): Nach zwei verlorenen Spielen steht das Konto oft so tief im Minus, dass das dritte Spiel die Kneipe nicht mehr schaffen kann. Das frustriert, weil man ein Spiel zu Ende spielt, das nichts mehr ändert. Dagegen: aufgeben können, sehen, was noch nötig ist, und drei Rettungs-Schnäpse.

**A, „Kneipe verlassen“** (`RunAction::GiveUpPub`, Ereignis `PubGivenUp`, Sicht `RunView::can_give_up`, `Run::can_give_up`):

- Geht **zwischen zwei Spielen** einer Kneipe (`Run::between_games`: das gegebene Spiel ist unberührt, zehn Karten, Skat nicht aufgenommen, nichts angesagt, keine Karte gespielt; ein Ramsch vor seiner ersten Karte) und **ab dem zweiten Spiel** (`games_played >= 1`). Entschieden von Claude: nicht schon vor dem ersten Spiel, weil dann nichts verloren ist, was eine Aufgabe rechtfertigt, und ein Fehlgriff ein Leben kostet.
- Zählt als **verlorene Kneipe**: Auf `PubGivenUp` folgt alles, was eine verfehlte Kneipe bringt (`PubLost`, seit ADR 0019 `RunLost` oder mit dem Schutzengel `SecondChance`). Keine Auszahlung, `OnSeriesWon` feuert nicht. Eine Spelunke wie eine normale Kneipe.
- **Boss-Kneipe**: Aufgeben beendet den Run (ADR 0017, `RunLost` mit `boss`). Die Oberfläche fragt deutlich nach: „Aufgeben beendet den Run.“
- **Schutzengel**: gilt auch für eine aufgegebene Kneipe (entschieden von Claude, wie vom Nutzer vorgeschlagen): Sie ist eine verlorene Kneipe, und der Engel fragt nicht, wie sie verloren ging. Seit ADR 0019 einmal im Run, auch in der Boss-Kneipe.
- **Bot**: gibt nie auf. Eine verlorene Kneipe beendet den Run (ADR 0019), ob er das letzte Spiel spielt oder nicht, und gespielt kann ein Wunder noch helfen.

**D, „noch nötig“** (Sicht `RunView::needed`, `NeededView`, `Run::max_game_value`): Vor dem letzten Spiel einer Kneipe (ein Spiel übrig, noch nicht angesagt) rechnet der Kern

- `points`: Zielwert minus Konto, was ein gewonnenes Spiel bringen muss (0 oder weniger, wenn das Ziel schon steht; die Boss-Kneipe spielt alle Spiele);
- `smallest_game`: den kleinsten Spielwert der reinen Formel (ohne Relikte und Faktoren), der reicht, z. B. 88 (Pik mit 8 Stufen) für 87 nötige Punkte;
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

**Bot** (`run/bot/`, Regeln, nicht kalibriert; Kaufwerte Klarer 1, Absacker 2, Eierlikör 1): Vor „Skat oder Hand“ trinkt er Klarer mit Minus, Absacker vor dem letzten Spiel ohne erreichtes Ziel und Eierlikör, sobald er angenommen wird.

Tests: `core/src/run/rescue_tests.rs`. Zu messen, sobald der Simulator wieder läuft: wie oft die drei gekauft und getrunken werden und was sie an „Stadtteil 2 erreicht“ ändern.

### 8.8 Flüche als Handel (03.11.2026)

**Entschieden vom Nutzer am 03.11.2026, maßgeblich ab hier.** Ein Fluch ist der Preis eines guten Geschäfts. Drei Angebote zahlen mit einem, dazu ein Relikt, das Flüche belohnt. Alle Zahlen sind **vorläufig**, nach Augenmaß ohne Messung; `balance` misst. Code `core/src/run/trade.rs`, Tests `core/src/run/trade_tests.rs`, Daten `trade` in `run.ron` (`TradeConfig`).

**Der Fluch eines Handels** (`Run::trade_curse`): ein dauerhafter, zufälliger Fluch (Durchsichtig, Ramponiert, Fleckig; nie Abgestuft) auf einer zufälligen Karte der Ausgabe, die noch keinen Fluch trägt (`Run::trade_curse_targets`). Erst die Karte (gleich verteilt unter allen, die einen Fluch nehmen können), dann der Fluch nach seinem Gewicht unter denen, die auf sie passen (`only_on`, Gewicht des Charakters je Spielart wie bei den Ereignissen). Es gelten die Regeln aus 7.2: Der Fluch ersetzt die normale Zinke der Karte, eine Spezialkarte (Untot, Wandelkarte) bleibt daneben. **Er zahlt keine Groschen** (Entscheidung des Leads am 03.11.2026): Der Fluch ist der Preis des Geschäfts, 3 Groschen zurück machten das Schnäppchen zur Gelddruckmaschine. Im Kern feuert der Fluch seine `OnAttach`-Hooks ohne `GainCoins` (`Run::attach_mod_paid` mit `paid: false`). Flüche aus anderen Quellen (Ereignisse) zahlen weiter 3 Groschen; ihr Text sagt jetzt „Beim Erhalt 3 Groschen (nicht bei einem Handel).“ Ereignis `TradeCurse { card, curse, cause }` vor dem `DeckChanged`, die Oberfläche sagt es als Meldung („Schnäppchen: Grün-Ober ist jetzt verflucht (Ramponiert).“). Ein Handel wird nur angeboten, solange eine Karte den Fluch nehmen kann; ist beim Kauf keine mehr frei, fällt der Fluch weg. Bruderherz kann diese Flüche wie seine eigenen nicht reinigen.

**Eigener Zufallsstrom** (`Run::trade_rng`): ob ein Angebot ein Handel ist, welche Karte, welcher Fluch. So bleiben Relikte, Schnäpse, Auslage, Wege und Spiele mit und ohne Handel gleich (Test `trades_leave_the_other_draws_alone`). Alte Spielstände bekommen ihn aus einer festen Zahl.

| Was | Id | Wirkung | Zahlen (vorläufig) |
|---|---|---|---|
| **Pechvogel** (Relikt, Gruppe Wert) | `jinx` | +1 Grundwert für jede verfluchte Karte der Ausgabe, gezählt bei der Ansage (`OnDeclare`, nicht im Ramsch; Effekt `AddBaseValuePerCursedCard(1)`, `Run::cursed_cards`) | Rare, 9 Groschen |
| **Fusel** (Schnaps) | `rotgut` | Das nächste angesagte Spiel zählt doppelt, gewonnen wie verloren (wie der Doppelkorn), und eine zufällige Karte ohne Fluch bekommt einen Fluch für immer (`NextGameCursed(Multiply(2))`) | Common, 2 Groschen |
| **Schnäppchen** (Auslage) | `CardOffer::bargain` | Ein Angebot der Auslage zum halben Preis (abgerundet, mindestens 1), dafür ein Fluch auf einer anderen Karte, wenn möglich auf keiner, die die Auslage noch anbietet | etwa jedes fünfte Angebot (`bargain_percent: 20`), nie eine Spezialkarte |
| **Hehlerware** (Tresen) | `Shop::fence` | Eines der drei Relikte zum halben Preis (abgerundet, mindestens 1), dafür beim Kauf ein Fluch auf einer zufälligen Karte | etwa jede vierte Auslage der Relikte (`fence_percent: 25`), auch nach der Neuauslage neu gewürfelt |

Auslegungen von Claude:

- **Pechvogel zählt jeden Fluch** (`ModKind::Curse` und `SpecialCurse`), also auch Abgestuft und die Flüche einer Kneipe (Die Oma, Spelunke), solange sie liegen: Er zählt, was gerade verflucht ist. Damit bringt er dem Nullspieler ab Start +4 (seine vier abgestuften Unter) und Bruderherz +9 (sechs Fleckig, drei Ramponiert). Darum Rare und 9 Groschen, die oberste Stufe der Staffel. Im Null zählt der Grundwert wie bei Acht-Fünfzehn mit. Der Bot reinigt keine Flüche mehr, solange er ihn hält.
- **Fusel** wird wie Kurzer und Eierlikör vor „Skat oder Hand“ getrunken (zehn Karten, Skat unberührt), nicht am Tresen und nicht nach dem Aufnehmen. Anders als der Doppelkorn gibt er nicht neu: Er darf auf ein bekanntes Blatt, der Fluch ist der Preis dafür. Der Fluch liegt sofort auf der Karte; was er beim Geben tut (Augen, Durchsichtig), gilt ab dem nächsten Spiel, seine Hooks (Fleckig bei der Ansage) und der Wegfall der ersetzten Zinke schon jetzt. Kann keine Karte mehr einen Fluch nehmen, lehnt der Run ihn ab (`NoModAvailable`).
- **Preis**: Der halbe Preis gilt vor dem Wirt und dem Preis des Charakters (Bruderherz: halb auf Rot); die Kachel zeigt den vollen Preis durchgestrichen daneben (`was` in der Sicht).
- **Bot** (`run/bot/`, nicht gemessen): Pechvogel Kaufwert 4, Fusel 1 (vor „Skat oder Hand“ bei einem nicht schwachen Blatt, wenn kein anderer Schnaps auf das Spiel wartet). Hehlerware und Schnäppchen zählen 1 weniger als dasselbe Angebot ohne Fluch (`TRADE_PENALTY`), mit Pechvogel voll.

**Oberfläche.** Die Relikte am Tresen liegen seit dem 03.11.2026 als **Reihe aus drei Kacheln** wie die Auslage der Zinken (3 × 1, auch hochkant am Handy): Bierdeckel, Name, Text, Preis, Kaufen; die ganze Kachel öffnet die Infokarte. Schnäppchen und Hehlerware tragen ein rotes Band („Schnäppchen“, „Hehlerware“), einen gestrichelten roten Rand, den vollen Preis durchgestrichen und die Zeile „Schnäppchen: dafür ein Fluch auf einer anderen Karte, ohne Groschen“ bzw. „Hehlerware: dafür ein Fluch auf einer zufälligen Karte, ohne Groschen“; die Infokarte der Hehlerware erklärt den Handel. Motive: Pechvogel eine schwarze Katze auf dem runden Wert-Deckel, Fusel eine schmutzige Flasche ohne Etikett.

**Spielstände.** Keine neue Version: `Shop::fence`, `CardOffer::bargain` und `trade_rng` fehlen in alten Spielständen und kommen leer bzw. aus einer festen Zahl.

### 8.9 Schnaps-Leiste (03.11.2026)

**Entschieden vom Nutzer am 03.11.2026, maßgeblich ab hier.** Schnäpse sollen öfter getrunken werden: Es gibt sie nach jeder gewonnenen Kneipe, und sie liegen in einer Leiste oben, aus der man sie auf ihr Ziel zieht. Code `core/src/run/schnapps_reward.rs`, Tests `core/src/run/schnapps_reward_tests.rs`.

**Ein Schnaps nach jeder gewonnenen Kneipe.** Jede gewonnene Kneipe, die den Run nicht beendet (normale Kneipe, Spelunke, Boss-Kneipe), gibt einen zufälligen Schnaps, nach Seltenheit gewichtet wie am Tresen aus den Schnäpsen des Charakters (`Run::schnapps_pool`, dieselbe Auswahl wie Kiosk und „Der Wirt spendiert“; Schnäpse werden nicht freigeschaltet, alle sind offen). Er kommt nach der Auszahlung und vor der Relikt-Wahl der Boss-Kneipe, der Beute der Spelunke und der Routenwahl; die kommen wie bisher dazu. Die letzte Boss-Kneipe gewinnt den Run und gibt keinen. Ist ein Platz frei, liegt er sofort dort (Ereignis `SchnappsRewarded { slots_full: false }`). Sind alle Plätze voll, wählt der Spieler wie bei der Beute der Spelunke: **neu nehmen und einen abgeben** (der abgegebene ist ohne Groschen weg, der neue nimmt seinen Platz) oder **für den halben Preis verkaufen** (`RunAction::ChooseSchnappsReward { give_up }`, Sicht `RunView::schnapps_reward`). Bis dahin geht nichts anderes (Fehler `SchnappsRewardPending`), auch kein Schnaps: Erst trinken, um Platz zu machen, geht nicht, sonst hätte die Wahl eine dritte Antwort, die die Oberfläche erklären müsste.

**Eigener Zufallsstrom** (`Run::schnapps_rng`, nach allen anderen vom Seed abgeleitet): Deals, Tresen, Begegnungen, Wege und Zinken bleiben mit und ohne den Schnaps gleich (Test `the_reward_leaves_every_other_draw_alone`). Alte Spielstände bekommen ihn aus einer festen Zahl.

**Drei Schnaps-Plätze** (`consumable_slots: 3` in `run.ron`, vorher zwei).

**Ziel und „jetzt trinkbar“ für die Oberfläche.** Jeder Schnaps nennt in `consumables.ron` sein Ziel (`target`, `ConsumableTarget`, ohne Angabe `Self`), nur für das Ziehen und Ablegen in der Oberfläche:

| Ziel | Schnäpse |
|---|---|
| `Skat` | Kurzer |
| `Table` (Tisch) | Kräuter, Doppelkorn, Fusel |
| `Card` (eine Karte der Hand) | Blue Curaçao |
| `Self` (der Spieler trinkt selbst) | Klarer, Absacker, Eierlikör, Wegbier |
| `Map` (Stadtplan) | Schwenker |

Ob ein Schnaps gerade geht, entscheidet allein der Kern: `OwnedConsumable::usable_now` (bis 03.11.2026 `can_use`) probiert die echte Aktion `UseConsumable` auf einer Kopie des Runs (`Run::accepts`), dieselbe Prüfung, die der Run beim Trinken macht; für Blue Curaçao mit jeder Karte der Hand (die passenden in `targets`). Damit stimmen Phase („Vor der Wahl zwischen Skat und Hand“, „Zwischen zwei Spielen“, „Bei der Wahl des Weges“), Bedingungen (Klarer nur im Minus, Konterbier nur mit fehlendem Leben), Schnapsverbot der Spelunke und eine offene Wahl immer mit dem überein, was der Kern annimmt.

**Oberfläche** (Entscheidungen des Nutzers vom 03.11.2026; `web/src/ui/hud.ts`, `web/src/ui/drink.ts`, `web/src/ui/mouth.ts`). Die Leiste im Kopf zeigt die Relikte nur als Bierdeckel (ohne Namen, ein Tipp öffnet die Infokarte); sie scrollt in sich, die Seite nie seitwärts. Daneben, hinter einem Trenner, die drei Schnaps-Plätze: leere gestrichelt (seit dem 11.11.2026 schmal, 16.3; ein belegter Platz bleibt ein volles Tippziel), ein Schnaps grau mit Sanduhr, solange `usable_now` falsch ist. Schnäpse erscheinen überall ohne Etikett und ohne Deckel: ihr 16-×-16-Motiv in ganzen Pixeln vergrößert, dazu ein kleiner Punkt in der Farbe der Seltenheit (Leiste, Tresen, Kiosk, Infokarte, Begegnungen, Belohnungen). Getrunken wird durch Ziehen auf das Ziel; nur gültige Ziele leuchten, anderswo losgelassen springt das Glas zurück. Ersatz ohne Ziehen: Tippen hebt den Schnaps an, Tippen aufs Ziel gießt, nochmal Tippen auf ihn legt ihn zurück. Skat: Tropfen auf den Skat, er dreht sich um. Tisch: Kräuter und Doppelkorn spülen die Karten mit einer grünen Welle weg, bevor neu gegeben wird; ein Faktor fürs nächste Spiel knallt als „×2“-Schild auf den Filz. Karte: Blue Curaçao färbt die Karte blau, dann die Farbwahl. Selbst: ein übergroßer Pixel-Mund im Comic-Stil („M-A“) fährt beim Ziehen von unten hoch; abgelegt kippt das Glas hinein, „Ahh!“ und ein kurzer Hinweis auf die Wirkung. Stadtplan: der Schwenker lost die Wege neu aus, die Stationen drehen sich wie Walzen eines Spielautomaten. Alles im eigenen Layer, abbrechbar, nach Tempo und ohne Bewegung bei `prefers-reduced-motion`; die Aktion ist immer `UseConsumable`, wie über „Trinken“ auf der Infokarte. Bei vollen Plätzen zeigt ein Blatt den neuen Schnaps groß, die drei eigenen zum Abgeben und „Verkaufen für N Gr.“.

**Bot** (`RunBot::schnapps_reward_action`, nicht gemessen): Bei vollen Plätzen nimmt er den neuen für den schwächsten eigenen (Kaufwert `CONSUMABLE_VALUES`, der erste bei Gleichstand), wenn er mehr wert ist, sonst verkauft er ihn.

**Spielstände.** Keine neue Version: `schnapps_rng` und `schnapps_reward` fehlen in alten Spielständen und kommen aus einer festen Zahl bzw. leer. Ein wartender Schnaps, den der Inhalt nicht mehr kennt, fällt beim Laden weg.

**Messung** (Alleinspieler, PIMC-Spieler, `cargo run --release -p skat_sim -- --seed 42 --runs 300 --bot pimc:12:3`): gegen `main` vor der Änderung (3af0889, gleicher Seed):

| Größe | vorher | nachher |
|---|---|---|
| Stadtteil 2 erreicht | 40,0 % | 48,0 % (+8,0) |
| Stadtteil 3 erreicht | 14,3 % | 19,7 % (+5,4) |
| Run gewonnen | 5,3 % | 8,3 % (+3,0) |
| Runs enden in normalen Kneipen | 36,7 % | 31,7 % (−5,0) |
| Schnäpse getrunken je Run | 1,6 | 5,6 |
| Schnäpse gekauft je Run | 1,6 | 1,5 |
| Groschen für Schnäpse je Run | 4,17 | 3,96 |

Der Bot trinkt dreimal so viel, kauft aber kaum weniger. Alle drei Quoten liegen jetzt über den Zielen aus `run.ron` (Stadtteil 2 in 35 bis 40 %, Stadtteil 3 in 12 bis 18 %, 3 bis 5 % gewonnen): **`balance` muss nachziehen** (Zielwerte, Schnaps-Preise oder die Belohnung selbst). Die Zahlen der Spiele in diesem Abschnitt sind nicht geändert.

---

### 8.10 Relikt-Familie Faktor (ADR 0018, 04.11.2026)

Relikte der Anzeige-Gruppe `Factor` (Bierdeckel: ein bernsteinfarbenes Sechseck, vorläufig). Ihre Faktoren gelten **nur für gewonnene Spiele** (Bedingung `Won` beim `OnScore`) und werden mit allen anderen Faktoren multipliziert. Seltenheit, Preise, Faktoren und Bot-Werte **vorläufig** nach Augenmaß, bis die Messung sie festlegt.

| Relikt | Id | Effekt | Seltenheit, Preis | Bot-Wert |
|---|---|---|---|---|
| **Eichelhäher** | `jay` | ×1,5, wenn du ein Eichel-Spiel gewinnst | Rare, 5 (bis 04.11.2026: 7) | 3 |
| **Laubfrosch** | `tree_frog` | ×1,5, wenn du ein Grün-Spiel gewinnst | Rare, 5 (bis 04.11.2026: 7) | 3 |
| **Rotkehlchen** | `robin` | ×1,5, wenn du ein Rot-Spiel gewinnst | Rare, 5 (bis 04.11.2026: 7) | 3 |
| **Schellenbaum** | `bell_tree` | ×1,5, wenn du ein Schellen-Spiel gewinnst | Rare, 5 (bis 04.11.2026: 7) | 3 |
| **Der Grandseigneur** (Figur) | `grand_seigneur` | ×2, wenn du einen Grand gewinnst | Legendary, 9 | 4 |
| **Schwarze Null** (Null) | `black_zero` | ×1,5, wenn du ein Null-Spiel gewinnst | Rare, 9 (seit SR4, 05.11.2026: beim Nullspieler + 19,3 / + 20,7; bis dahin 7) | 2 |
| **Zapfhahn** | `tap` | Jede gewonnene Kneipe +×0,1 auf jedes gewonnene Spiel, für den Rest des Runs, höchstens ×3 (seit 05.11.2026 wegen der langen Nacht, 5.5; Zähler `tap`, `ScaleGameValuePerCounter` mit `max`) | Legendary, 10 (seit Runde 3, 05.11.2026; bis dahin Rare, 8) | 6 |
| **Heiße Hand** | `hot_hand` | Jedes gewonnene Spiel in Folge +×0,15 auf die nächsten gewonnenen (seit 05.11.2026, Nutzer q28 B; bis dahin +×0,2), höchstens ×3 (seit der Nacht-Runde 05.11.2026); ein verlorenes setzt auf ×1 zurück (Zähler `hot_hand`) | Rare, 9 (seit 05.11.2026 mit +×0,15, ohne Tresen + 8,3 / + 4,7; SR4: 10; Runde 3: 8; bis dahin 7) | 5 |

- **Farb-Relikte** (Wunsch des Nutzers): `GameIs(Suit(…))` und `Won`. Namen mit Stammtisch-Klang nach den deutschen Farben (ADR 0010): Eichelhäher, Laubfrosch (Grün), Rotkehlchen, Schellenbaum.
- **Zapfhahn**: Der Zähler steigt nach jeder gewonnenen Kneipe (`OnSeriesWon`, auch Spelunke und Boss-Kneipe); er wirkt ab dem nächsten gewonnenen Spiel. Faktor 1 + 0,1 × Zähler.
- **Heiße Hand**: Der Faktor zählt die Siege *vor* diesem Spiel (der Faktor-Hook steht vor dem Zähler): ×1, ×1,15, ×1,3 …; die Vorschau beim Spielen zeigt so genau den Faktor, den ein Sieg bekommt. Ein verlorenes Spiel (auch ein Ramsch) setzt den Zähler zurück, Einmischen nicht.
- **Zähler** liegen wie alle Run-Zähler im Spielstand (`counters`) und stehen in der Relikt-Leiste.
- **Gemessen in Runde 1** (04.11.2026, `docs/balance/2026-11-04-runde1.md`, alle fünf Charaktere): Gewonnene Spiele haben im Mittel ×1,15 bis ×1,25 (Alleinspieler nachher ×1,25), ein Fünftel hat einen Faktor über 1; der höchste Faktor je Run im Mittel ×1,4 bis ×1,7, der höchste überhaupt ×13,2 (Heiße Hand mit Farb-Relikt und Glückssträhne). 15 bis 27 % der Runs enden mit einem Faktor-Relikt, gekauft wird jedes in 4 bis 6 % der Runs (Grandseigneur und Schwarze Null beim Alleinspieler je 1,6 %). Keines dominiert; Seltenheit, Preise und Faktoren bleiben. Einzelmessung ohne Tresen in Runde 2.
- **Gemessen in Runde 3** (05.11.2026, `docs/balance/2026-11-05-runde3.md`, ohne Tresen, Alleinspieler, 400 Runs): Zapfhahn + 12,2 / + 15,0 auf „Stadtteil 2 / 3“ (Pflicht, deshalb Legendär), Grandseigneur + 12,8 / + 9,2, Heiße Hand + 9,5 / + 6,0, Schwarze Null + 4,0 / + 1,0 (beim Nullspieler + 7,0 / + 5,3), Goldene Schere + 3,5 / + 3,0, Farb-Vögel + 1,0 bis + 2,2 / − 0,5 bis + 1,0.
- **Glückssträhne** (×2 ab dem dritten Sieg in Folge) war schon ein Faktor-Relikt und bleibt in der Gruppe Stufen; Stammfarbe, Bock, Einsatz und Schnäpse zählen weiter auch verloren (ADR 0018 Punkt 4).

### 8.11 Bau-Richtungen (04.11.2026)

**Entschieden vom Nutzer am 04.11.2026 (Cockpit q26, Variante A, „alle Änderungen übernehmen“):** Fünf Bau-Richtungen mit echten Synergien. Bisher waren Relikte vereinzelte Boni; jetzt bekommt jede Richtung zwei bis drei **Verbinder-Relikte**, die andere Teile des Builds zählen, und die bestehenden Relikte, Zinken und Schnäpse tragen ihre Richtung als Daten (`tags` in `house_rules.ron`, `card_mods.ron`, `consumables.ron`; `BuildTag`). Die Tags stehen im Katalog der Web-App (`catalog().relics[id].tags`, ebenso `card_mods`, `consumables`); die Kartenbelohnung darf Angebote danach gewichten. Seltenheit, Preise und Bot-Werte **vorläufig** nach Augenmaß, ohne Messung. „Zinke“ heißt in den Texten jede Modifikation, die kein Fluch ist (auch Marken und Spezialkarten).

| Richtung (Tag) | Bestehende Teile mit Tag | Neue Verbinder |
|---|---|---|
| Farbtreue (`OneSuit`) | Stammfarbe, Eichelhäher, Laubfrosch, Rotkehlchen, Schellenbaum; Siegel; Blue Curaçao | Vereinsfarben, Treue Seele |
| Markenhändler (`Vouchers`) | Skatpfleger, Drei drücken; Pfand-, Toten-, Narrenmarke | Rabattheft, Leergut, Sammelalbum |
| Stichjäger (`Tricks`) | Luschenkönig, Spitze; Goldkante, Fadenkreuz, Kerbe, Eselsohr, Winkel | Fingerspitzengefühl, Strichliste, Trinkgeld |
| Schneider (`Schneider`) | Jungfrau, Der Schneider, Stichfest, Schenken, Skatgott; Schneiderkreide, Aufdruck | Goldene Schere, Maßband, Schnittmuster |
| Nullspieler (`Null`) | Nullachtfuffzehn, Dicke Sau, Schwarze Null; Blei | Bleigießen, Senkblei |
| Grand (`Grand`, seit 07.11.2026, 8.12) | Großvaters Grand, Der Grandseigneur | Die Großkopferten, Unterschlupf, Bierruhe, Offenes Visier |

| Relikt | Id | Text | Hebel, was es zählt | Seltenheit, Preis | Bot-Wert |
|---|---|---|---|---|---|
| **Vereinsfarben** | `club_colours` | Farbspiel: +2 Grundwert je Karte deiner Trumpffarbe mit Zinke in deinem Deck (bis Runde 3, 05.11.2026: +1). | Grundwert; Siegel, Aufdruck, Winkel … auf der Trumpffarbe (Unter der Farbe mit, wie beim Siegel), gehalten oder nicht | Uncommon, 6 (bis 04.11.2026: 7) | 3 |
| **Treue Seele** (bis 07.11.2026, seitdem überarbeitet, 8.12) | `true_soul` | Gewinnst du ein Farbspiel: +1 Stufe je Spiel direkt davor in derselben Trumpffarbe, höchstens +3. | Stufen; die Folge der Stammfarbe (Zähler `true_soul`, über Kneipen, Ramsch unterbricht nicht) | Common, 4 (bis 04.11.2026: 5) | 1 |
| **Rabattheft** | `rebate_book` | Jede eingelöste Marke: +1 Stufe auf jedes gewonnene Spiel, bis die Kneipe vorbei ist. | Stufen; eingelöste Marken der Kneipe (Zähler `rebate_book`, zurück nach der gewonnenen Kneipe) | Uncommon, 6 | 2 |
| **Leergut** | `empties` | Jede eingelöste Marke bringt dir 2 Groschen extra. | Groschen; eingelöste Marken | Common, 4 | 2 |
| **Sammelalbum** | `stamp_album` | +1 Grundwert je Marke, die in deinem Deck noch auf einer Karte klebt. | Grundwert; Marken im Deck (zieht gegen die beiden davor: behalten oder einlösen) | Common, 6 (seit Runde 3; bis dahin 5) | 2 |
| **Fingerspitzengefühl** | `fingertips` | Machst du einen Stich mit einer Karte mit Zinke, zählt er 3 Augen mehr. | Augen (auch für Schneider); Stiche mit Zinken | Uncommon, 6 | 4 |
| **Strichliste** | `tally_sheet` | Machst du in einem gewonnenen Spiel 3 Stiche mit Karten mit Zinke: +1 Stufe, bei 5 Stichen +2. | Stufen; Stiche mit Zinken je Spiel (Zähler `tally_sheet`) | Uncommon, 5 (bis 04.11.2026: 6) | 3 |
| **Trinkgeld** | `tip` | Machst du einen Stich mit einer Karte mit Zinke: 1 Groschen, höchstens 3 pro Spiel. | Groschen; Stiche mit Zinken (Zähler `tip`) | Common, 4 | 3 |
| **Goldene Schere** | `golden_shears` | ×1,5, wenn du mit Schneider gewinnst. | Faktor (ADR 0018: nur selten, Rare, Gruppe Faktor) | Rare, 8 | 5 |
| **Maßband** | `tape_measure` | Jedes Spiel, das du mit Schneider gewinnst, gibt +1 Grundwert für den Rest des Runs. | Grundwert; Schneider-Siege (Zähler `tape_measure`) | Rare, 7 | 5 |
| **Schnittmuster** | `sewing_pattern` | Gewinnst du mit Schneider: +1 Grundwert je Karte mit Aufdruck oder Schneiderkreide in deinem Deck. | Grundwert; Aufdruck und Schneiderkreide im Deck (die Schneiderin startet mit vier) | Common, 6 (seit Runde 3; bis dahin 5) | 2 |
| **Bleigießen** | `lead_pouring` | Gewinnst du ein Null: +1 Stufe je Karte mit Blei auf deiner Hand. | Stufen; Blei auf der Hand nach dem Skat (im gewonnenen Null alle unterlaufen) | Uncommon, 8 (seit SR4; bis dahin 6) | 2 |
| **Senkblei** | `plumb_line` | Null: +1 Grundwert je Karte mit Blei in deinem Deck, gewonnen wie verloren (bis SR4, 05.11.2026: +2; damit beim Nullspieler + 10,7 / + 15,3). | Grundwert; Blei im Deck | Common, 5 | 2 |
| **Großvaters Grand** | `grandfathers_grand` | Der Grand zählt wieder 24 wie zu Großvaters Zeiten: Grundwert +6. | Grundwert; Antwort auf den Grand mit 18 (Cockpit q27), kein Tag | Uncommon, 7 (bis 04.11.2026: 6) | 3 |

Auslegungen (Claude, 04.11.2026):

- **Neue Handgriffe der Hook-API**: `CardCount` (welche Karten der Ausgabe zählen: `mods`, `kind`, `trump_suit`, `in_hand`) mit den Wirkungen `AddBaseValuePerCard` und `AddLevelsPerCard` (`Run::count_cards`, Zeile wie beim Pechvogel nur, wenn mehr als null zählt); die Bedingung `TrickWonWithMarkedCard` (der Alleinspieler macht den Stich mit einer Karte seiner Ausgabe mit Zinke, `Run::marked_cards`); die Wirkung `AddTrickPoints(n)` (Augen zum fertigen Stich); der Auslöser `OnVoucherRedeemed` (nach jeder eingelösten Marke, gedrückt oder im Handspiel).
- **Bleigießen** zählt die Blei-Karten auf der Hand nach dem Skat statt der „abgeworfenen“: Im gewonnenen Null wirft der Spieler ohnehin alle ab, und so steht die Zahl schon bei der Ansage fest (Wertvorschau, Rest abkürzen).
- **Rabattheft** endet mit der gewonnenen Kneipe (`ResetCounter` beim `OnSeriesWon`, wie der Allrounder). Spielt der Schutzengel eine verlorene Kneipe neu, bleibt der Zähler stehen.
- **Rest abkürzen** wartet bei Fingerspitzengefühl, Strichliste und Trinkgeld, solange der Spieler eine Karte mit Zinke auf der Hand hat.
- **Großvaters Grand** ist ein fester Zuschlag (+6) auf den Grundwert jedes Grand, gewonnen wie verloren. Er hängt nicht am Grundwert des Grand im Run: Mit 18 (Cockpit q27) zählt der Grand mit ihm 24, solange der Grand noch 24 zählt, 30.
- **Gesperrt** (9.3): Rabattheft hinter „Voll gedrückt“, Goldene Schere hinter „Maßarbeit“, Vereinsfarben hinter „Farbe bekennen“, je neben dem Charakter. So bleibt ein Drittel der Relikte zu Beginn gesperrt (19 von 56); die übrigen neuen sind frei.
- **Infokarte**: Die Kopfzeile nennt die Richtung („Hausregel · ungewöhnlich · Farbtreue“), aus den Tags im Katalog.
- **Bierdeckel**: je ein eigenes Motiv in `web/src/ui/emblems.ts` (vorläufig): Vereinswimpel, treuer Hund, Rabattheft, Leergut-Kasten, Briefmarkenalbum, Zeigefinger, Strichliste, Glas mit Groschen, goldene Schere, Maßband, Schnittmuster, Löffel mit Blei, Senkblei, Großvaters Pfeife.
- **Gemessen in Runde 3** (05.11.2026, `docs/balance/2026-11-05-runde3.md`, ohne Tresen, 300 bis 400 Runs): Beim Alleinspieler wirken die Verbinder ohne ihren Build kaum (Großvaters Grand + 3,2 / + 2,0 auf „Stadtteil 2 / 3“, Maßband − 0,2 / + 2,2, Fingerspitzengefühl, Strichliste, Schnittmuster, Vereinsfarben + 1 bis + 2 auf „Stadtteil 3“, die übrigen nichts). Im passenden Build: Drückeberger mit Rabattheft + 12,3 / + 2,7, mit Sammelalbum + 8,0 / + 2,3; Schneiderin mit Goldener Schere + 7,7 / + 4,0, mit Schnittmuster + 6,3 / + 3,0, mit Maßband + 2,0 / + 2,7; Nullspieler mit Bleigießen und Senkblei nur + 0,7 (kaum Blei im Deck: Der Bot nimmt als Kartenbelohnung fast immer Rang +1). Leergut und Trinkgeld bringen nur Groschen, ohne Tresen nicht messbar. Danach Vereinsfarben +1 → +2 je Karte, Sammelalbum und Schnittmuster 5 → 6.

### 8.12 Relikte v4 (07.11.2026)

**Entschieden vom Nutzer am 07.11.2026:** alle 24 Vorschläge der Relikt-Runde v4 angenommen, 20 neue Relikte und vier Überarbeitungen (Treue Seele, Spickzettel, Bierdeckel, Spelunkenkenner ersetzen ihre alte Wirkung). Neue Bau-Richtung **Grand** (`BuildTag::Grand`), die auch Großvaters Grand und Der Grandseigneur tragen. Daten in `house_rules.ron`, Tests in `core/src/run/relics_v4_tests.rs`. Seltenheit, Preise und Bot-Werte zuerst nach Augenmaß; seit der Balance-Runde v4 (07.11.2026, `docs/balance/2026-11-07-relikte-v4.md`) gemessen, die Tabelle zeigt den Stand danach (Unterschlupf, Leerer Stuhl, Klempner, Abstinenzler, Stich auf Stich, Wackelkontakt, Rabattkönig und Fanblock mit geänderter Wirkung).

| Relikt | Id | Text | Gruppe, Richtung | Seltenheit, Preis | Bot-Wert |
|---|---|---|---|---|---|
| **Treue Seele** (überarbeitet) | `true_soul` | Farbspiel: +1 Grundwert je Karte deiner Trumpffarbe auf deiner Hand (nach dem Skat, ohne Unter). | Wert, Farbtreue | Uncommon, 5 | 4 |
| **Fanblock** | `fan_block` | Farbspiel: +3 Stufen, wenn jede Karte mit Siegel in deinem Deck deine Trumpffarbe hat (mindestens drei). | Stufen, Farbtreue | Uncommon, 5 | 2 |
| **Kahlschlag** | `clear_cut` | Farbspiel: +1 Stufe je Fehlfarbe, von der du nach dem Skat keine Karte mehr hältst, höchstens +2. | Stufen, Farbtreue | Common, 5 | 3 |
| **Der Tankwart** (Figur) | `pump_attendant` | Nach jeder gewonnenen Kneipe klebt der Tankwart eine Pfandmarke auf eine zufällige Karte deines Decks ohne Zinke. | Tour, Markenhändler | Uncommon, 5 | 2 |
| **Rabattkönig** | `rebate_king` | Ab der zweiten eingelösten Marke einer Kneipe: ×1,5 auf jedes gewonnene Spiel, bis die Kneipe vorbei ist. | Faktor, Markenhändler | Rare, 7 | 3 (+2 im Marken-Build) |
| **Doppelt gemoppelt** | `double_stamp` | Jede Marke wird zweimal eingelöst. | Kontrolle, Markenhändler | Rare, 7 | 1 (+4 im Marken-Build) |
| **Sieben auf einen Streich** | `seven_at_one_blow` | Machst du in einem gewonnenen Spiel mindestens sieben Stiche: ×1,5. | Faktor, Stichjäger | Rare, 8 | 5 |
| **Emporkömmling** | `upstart` | Deine Karten mit Winkel zählen die Augen der Karte, die sie geworden sind. | Wert, Stichjäger | Uncommon, 5 | 3 |
| **Stich auf Stich** | `trick_after_trick` | Machst du drei Stiche hintereinander: +1 Stufe, einmal pro Spiel. | Stufen, Stichjäger | Uncommon, 6 | 3 |
| **Große Klappe** | `big_mouth` | Sagst du Schneider an und gewinnst, zählt der Grundwert doppelt. | Stufen, Schneider | Uncommon, 5 | 1 |
| **Elle** | `yardstick` | Gewinnst du mit Schneider: +1 Grundwert je 5 Augen über 90. | Wert, Schneider | Uncommon, 6 | 3 |
| **Der Klempner** (Figur) | `plumber` | Gewinnst du in einer Kneipe ein Null, gießt der Klempner nach der Kneipe Blei auf eine zufällige Karte deines Decks ohne Zinke, höchstens dreimal im Run. | Wert, Nullspieler | Uncommon, 7 | 2 |
| **Wackelkontakt** | `loose_contact` | Null: Der Kurzschluss kommt schon, wenn die Gegner nach fünf Stichen 55 Augen haben (statt 61). | Stufen, Nullspieler | Uncommon, 6 | 2 |
| **Hosen runter** | `cards_on_table` | Spielst du Null Ouvert, liegen auch die Karten der Gegner offen. | Kontrolle, Nullspieler | Uncommon, 4 | 1 |
| **Die Großkopferten** | `big_shots` | Grand: Nach den Untern zählen die Daus als Spitzen weiter (Eichel, Grün, Rot, Schellen). | Stufen, Grand | Uncommon, 5 | 3 |
| **Unterschlupf** | `safe_haven` | Mischst du in einem Spiel zum ersten Mal ein, behältst du deine Unter. Der Rest wird neu gegeben. | Kontrolle, Grand | Rare, 7 | 5 |
| **Bierruhe** | `beer_calm` | Verlierst du einen Grand, zählt er nur halb. | Kontrolle, Grand | Common, 5 | 2 |
| **Spickzettel** (überarbeitet) | `crib_sheet` | Handspiel: +1 Stufe mehr, und nach der Ansage siehst du beide Skatkarten. | Stufen | Common, 4 | 1 |
| **Bierdeckel** (überarbeitet) | `beer_mat` | Jedes Spiel einer Kneipe macht einen Strich: +2 Grundwert je Strich. Nach der Kneipe zahlst du 2 Groschen. | Wert | Uncommon, 6 | 4 |
| **Offenes Visier** | `open_visor` | Ouvert verlangt nur noch Schneider statt Schwarz. Schaffst du trotzdem Schwarz, zählt es. | Stufen, Grand | Rare, 6 | 1 |
| **Leerer Stuhl** | `empty_chair` | +1 Stufe auf jedes gewonnene Spiel, solange mindestens zwei Relikt-Plätze frei sind. | Stufen | Rare, 9 | 2, mit zwei freien Plätzen nach dem Kauf 5 |
| **Der Trödler** (Figur) | `junk_dealer` | Verkaufst oder tauschst du ein Relikt: +1 Grundwert für den Rest des Runs, einmal je Tresen. | Wert | Common, 4 | 2 |
| **Der Abstinenzler** (Figur) | `teetotaller` | Lässt du eine Kartenbelohnung aus: +1 Grundwert für den Rest des Runs. | Wert | Uncommon, 6 | 2 |
| **Spelunkenkenner** (überarbeitet, Figur) | `dive_expert` | Gilt in einer Kneipe ein Malus, zählt jedes gewonnene Spiel dort +2 Stufen. | Stufen | Uncommon, 5 | 3 |

**Neue Handgriffe der Hook-API** (Glossar): Auslöser `OnRelicSold` (Verkauf am Tresen, Ersetzen am Tresen und bei der Boss-Belohnung, Tausch und Verkauf in Begegnungen; das abgegebene Relikt feuert nicht mehr) und `OnCardRewardSkipped` (nur aktives Auslassen); Wirkungen `AttachModToRandomCard`, `AddLevelsPerVoidSuit`, `AddBaseValuePerDeclarerPoints`, `AddLevelsPerFreeRelicSlot`, `RedeemVouchersTwice`, `PromotedCardsTakePoints`, `ShortCircuitThreshold`, `GrandAcesAsMatadors`, `RedealKeeps`, `OuvertRequiresSchneider`; Bedingungen `CardsCounted`, `PubMalusActive`, `DeclarerTricksAtLeast`; `CardCount` mit `all_cards`, `no_jacks`, `off_trump`; `RevealDefenderHand` auch bei der Ansage, für beide Gegner (`ActiveGame::shown_hands`).

Auslegungen (Claude, 07.11.2026):

- **Große Klappe** gilt nur für gewonnene Spiele (Entscheidung des Leads, wie die Faktor-Relikte nach ADR 0018 Punkt 4); der Text sagt es. Schwarz angesagt und Ouvert schließen Schneider angesagt ein.
- **Faktor-Relikte** (Rabattkönig, Sieben auf einen Streich) nur gewonnen. **Bierruhe** ist ein Faktor unter 1 auf verlorene Grands (Gruppe Kontrolle, kein Faktor-Sechseck); ADR 0018 verbietet nur Faktoren über 1 auf Verluste.
- **Sieben auf einen Streich** zählt die Stiche am Ende über `DeclarerTricksAtLeast` statt mit einem Zähler je Stich: geschenkte Stiche (Schenken) zählen mit, und „Rest abkürzen“ kann früher entscheiden.
- **Fanblock** wirkt ab der Ansage (wie jede Stufe der Ansage, gewonnen wie verloren), damit Wertvorschau und Rest abkürzen ihn kennen.
- **Treue Seele** und **Kahlschlag** zählen Karten so, wie der Spieler sie hält (Blue Curaçao, eingefärbte Wandelkarte). Die alten Flaggen `true_soul_*` und der Zähler `true_soul` bleiben in alten Spielständen liegen, ohne Wirkung.
- **Bierdeckel** zählt mit dem neuen Zähler `beer_tab`; der alte `beer_mat` wirkt nicht mehr.
- **Der Tankwart / Der Klempner** ziehen die Karte mit einem eigenen Zufall aus dem Seed des Spiels (`ATTACH_SEED`), nur Karten ohne jede Modifikation (auch ohne Fluch), auf die die Zinke passt; sonst nichts. Das Blei des Klempners wirkt ab dem nächsten Spiel.
- **Doppelt gemoppelt**: Die Marke fällt einmal ab, ein Ereignis `VoucherRedeemed`; ihre `OnDiscard`-Wirkung und alle `OnVoucherRedeemed`-Hooks laufen zweimal. Ein zweites `TurnInto` auf eine schon verwandelte Karte ändert nichts.
- **Emporkömmling**: Die Augen der neuen Karte nach den Relikten, die Augen setzen (Alte Oberin); eigene Augen-Zinken zählen dazu. Nur die Ausgabe des Spielers.
- **Die Großkopferten** zählen im Grand über die zwölf Karten des Alleinspielers in seiner Ausgabe (`rules::matador_order`, `Game::count_grand_aces`), in Wertvorschau und Obergrenze ebenso.
- **Unterschlupf** behält seit der Balance-Runde v4 nur beim ersten Einmischen eines Spiels (`per_game: Some(1)`, `Series::redeals_this_game`; vorher bei jedem, das Hochschaukeln der Unter brachte + 23 / + 14 Punkte auf Stadtteil 2 / 3) (auch Kräuter und Doppelkorn) die Karten, die in der Ausgabe des Spielers Unter sind (ein abgestufter nicht); der Rest wird mit dem Zufall des Gebens neu verteilt (`Game::deal_editions_keeping`, ohne behaltene Karten genau wie bisher).
- **Offenes Visier**: `Thresholds::ouvert_schneider`; die Regelfunktion bleibt bei der Skatordnung (Golden-Vektoren unverändert), der Run biegt sie wie die Gewinnschwelle.
- **Leerer Stuhl**: freie Plätze = `relic_slots` des Charakters minus gehaltene Relikte (Hausregeln des Abends zählen nicht); ohne Platz-Grenze nichts. Seit der Balance-Runde v4 +1 Stufe ab zwei freien Plätzen (`AddLevelsPerFreeRelicSlot` mit `every: 2`, `max: 1`; vorher +1 je Platz, höchstens +3: + 26 / + 22 Punkte).
- **Der Trödler**: einmal je Tresen über die Flagge `junk_dealer_sold`, die beim Öffnen des nächsten Tresens fällt; sonst wäre Kaufen und Verkaufen +1 Grundwert für 2 Groschen.
- **Spelunkenkenner**: `PubMalusActive` (Malus einer Spelunke oder Spelunkenluft der Nacht). `ChooseMalus` nutzt kein Relikt mehr, bleibt lesbar und getestet.
- **Rest abkürzen** wartet bei Stich auf Stich, solange der Alleinspieler Karten hat (wie bei der Strichliste), und bei der Elle, solange die Stufe der Augen über 90 noch nicht feststeht.
- **Gesperrt** (9.3): sieben der neuen als zweite Belohnung bestehender Taten (Grand Schneider → Großkopferten, Grand Hand → Offenes Visier, Voll gedrückt → Doppelt gemoppelt, Maßarbeit → Elle, Farbe bekennen → Fanblock, Stammkunde → Unterschlupf, Glückspilz → Sieben auf einen Streich), damit ein Drittel der Relikte gesperrt bleibt.
- **Bot** (`run/bot/`): Werte in `RELIC_VALUES`; im Marken-Build sind Doppelt gemoppelt (+4) und Rabattkönig (+2) mehr wert, der Leere Stuhl zählt die nach dem Kauf freien Plätze. Mit dem Abstinenzler lässt der Bot jede Kartenbelohnung aus, die nicht mehr als eine Zinke der eigenen Richtung wert ist (`TEETOTALLER_SKIP_WORTH`). Seit der Balance-Runde v4 (`docs/balance/2026-11-07-relikte-v4.md`): Mit Großer Klappe sagt er nach dem Skat Schneider an, mit Offenem Visier spielt er Ouvert, wenn Schneider in mindestens 80 % von zwölf doppelt-offen gelösten Verteilungen der unbekannten Karten gelingt (`schneider_percent`) und der erwartete Wert über dem Spiel ohne Ansage liegt; mit Kahlschlag drückt er auf Fehlfarben (höchstens zwei, auch ein blankes Daus). Ohne diese Relikte spielt er unverändert. Hosen runter misst der Simulator weiter kaum.
- **Simulator**: Zeile `relic score lines` (Spiele mit einer Zeile eines Relikts in der Abrechnung) zeigt, ob ein Relikt wirkt.
- **Bierdeckel-Motive** der neuen Relikte kommen getrennt (`web/src/ui/emblems.ts`).

## 9. Charaktere

Jeder Charakter hat ein eigenes Grunddeck (seit ADR 0015: seine Ausgabe des Skatblatts mit Modifikationen ab Start, nie eine andere Zusammensetzung, 9.8), Startrelikte (oder einen Start-Schnaps) und genau eine eigene Regel; eine eigene Zielwertkurve ist möglich. Sieben Charaktere: der Alleinspieler (frei), der Nullspieler, der Drückeberger, die Schneiderin, Bruderherz und seit dem 10.11.2026 der Zocker und die Ramschtante (je über eine Freischaltung, 9.3, 9.6b, 9.6c). Am 03.11.2026 hat der Nutzer den Grandspieler, den Zaungast und die Wirtin gestrichen und die drei neuen bestimmt (9.4 bis 9.6).

**Seit Runde 1 ohne Leben (04.11.2026)** gelten neue Kurven (5.0.4); ohne Leben mit den Kurven vom 03.11.2026 erreichten Stadtteil 2: Alleinspieler 29,0 %, Nullspieler 26,9 %, Drückeberger 31,6 %, Schneiderin 22,4 %, Bruderherz 20,9 % (je 3 × 150 Runs, Alleinspieler 3 × 300; `docs/balance/2026-11-04-runde1.md`). Gemessen ist danach nur der Alleinspieler (33,4 / 12,2 / 5,1 %); die neuen Kurven von Nullspieler (× 1,1), Schneiderin (Stadtteil 1 wie der Alleinspieler, danach × 1,1) und Bruderherz (× 0,9, jetzt eigene) sind Vorschläge für Runde 2. Die Tabelle unten ist der Stand vom 03.11.2026 mit Leben.

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

**Die eigene Regel** (`rule`) hat zwei Teile, beide Daten: `passive` und `hooks` sind dieselbe Hook-API wie bei Relikten (Abschnitt 8) und wirken wie ein Relikt, das der Charakter immer hat, ohne Platz zu belegen; in der Wertung steht die Zeile mit der Quelle `Source::Character`. `passes`, `relic_slots` und `payout_factor` ändern die Run-Struktur aus `run.ron`: Pässe pro Serie als Zuschlag (negativ: weniger, nie unter 0), Relikt-Plätze seit 03.11.2026 als Obergrenze (`Some(n)`, die kleinere von `run.ron` und Charakter gilt; bis dahin ein Zuschlag), die Groschen einer gewonnenen Serie als Faktor (`Payout::bonus` ist der Zuschlag). Seit dem 03.11.2026 kommen drei Schalter dazu, die keine Hook-API brauchen: `vouchers_return` (eingelöste Gutscheine kommen nach der Kneipe zurück, 9.4), `keep_curses` (Flüche der Ausgabe lassen sich nicht reinigen) und `zinken_discount` (Zinken auf passenden Karten billiger, 9.6). Kein Charakter nutzt `relic_slots`, `payout_factor` oder die Wirkung `RevealDefenderHand` mehr; sie bleiben als Datenoption und sind getestet. Dazu `back`: die Kartenrückseite seiner Ausgabe (9.3). Die Einsatz-Leiter (5.4) gilt pro Charakter von selbst (`MetaState::stakes` ist nach Charakter geschlüsselt).

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

**Null-Wertung, entschieden am 01.11.2026 mit dem Simulator: die Formel.** Beide Varianten sind Daten (`null_scoring` in `characters.ron`) und laufen durch dieselbe Formel Grundwert × Stufen, damit Relikte auf Grundwert, Stufen und Spielwert auch für Null wirken:

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

**M5: eigene Zielwerte** (bis ADR 0012, 5.2, `targets` in `characters.ron`): Mit 8 Spielen pro Serie und Relikten (Bockrunde, Siebenschläfer, Lehrgeld) bringt ein gewonnenes Null meist 92 bis über 200 Punkte; mit Zielwerten nah an denen des Alleinspielers (36 / 46 / 66 in Abschnitt 1, 99 bis 198 in Abschnitt 4 und 5) gewann der Bot jede Serie zu 76 bis 86 % und kam in 3,8 % der Runs durch (Alleinspieler 1,4 %, je 2 × 1000 Runs). Der Bot spielt seit M5 bis zu fünf offene Karten (`NULL_MAX_EXPOSED` in `core/src/run/bot/`): mit 4 erreichte er Abschnitt 3 in 20,1 %, mit 5 in 25,1 % (vorherige M5-Messung, 3 × 600 Runs); er steht für einen guten Spieler.

**Null für alle (ADR 0012, umgesetzt in M6).** Jeder Charakter darf Farbspiel, Grand und Null ansagen, in der Reihenfolge der Skatordnung (ADR 0011): aus der Hand Null Hand und Null Ouvert Hand, nach dem Skat Null und Null Ouvert. Was die Charaktere unterscheidet, sind nur noch Daten:

- **Ansage**: Der Dialog zeigt jedem Charakter sechs Spielarten (Eichel, Grün, Rot, Schellen, Grand, Null); bei Null die Null-Stufen dieser Phase mit Wert (`DeclareOptions.null_risks`). `null_style` sagt, ob ein gewonnenes Null einen Stilbonus bekommt. Beide Charaktere werten Null mit der Formel oben (entschieden am 01.11.2026: eine Null-Wertung für alle, damit dasselbe Spiel überall gleich zählt; `Fixed` bleibt als Datenoption). Ramsch gibt es weiter nur beim Ramscher.
- **Der Nullspieler bleibt Spezialist**: Startblatt ohne Unter mit acht Siebenern (seit ADR 0015 ersetzt, 9.8), Kiebitz, Formel-Wertung, und am Tresen öfter Null-Relikte. Er darf auch Farbe und Grand; ohne Unter ist jedes seiner Trumpfspiele „ohne 4“.
- **Gemeinsamer Pool**: Jedes Relikt und jede Modifikation liegt bei jedem Charakter (die Null-Hausregeln auch beim Alleinspieler; die Null-Modifikationen sind mit Effekte v2 gestrichen, 7.2). Das neue Feld `focus` (`GameFocus`: `All`, `Trump` für Farbspiel und Grand, `Null`) sagt, für welche Spielart ein Inhalt gemacht ist. `focus_weights` in `characters.ron` gewichtet damit Tresen (seit 8.6 auch Auslage und Kartenmaler; bis dahin das Stempelkissen) und Ereignisse in Prozent der Seltenheit: Alleinspieler Trump 100, Null 50; Nullspieler Trump 50, Null 200; `All` immer 100. Kein Code-Zweig pro Charakter. Relikte für Trumpfspiele, die im Null Unsinn täten (Spitze, Schenken, Auf dem Trockenen), schließen Null in ihren Bedingungen aus (`Not(NullGame)`). Der Angeber verlangt Farbe oder Grand Ouvert Hand, sonst Null Ouvert.
- **Bot** (`core/src/run/bot/`): Aus der Hand spielt er Null Ouvert Hand ohne offene Karte, Farbe oder Grand Hand mit acht Trümpfen oder einer Ansage; sonst nimmt er den Skat auf. Mit zwölf Karten vergleicht er das Farbspiel oder den Grand der Heuristik mit Null (die zwei offensten Karten gedrückt, Null Ouvert ohne offene Karte) nach dem erwarteten Ergebnis ohne Relikte (`suit_expected_score`, `null_expected_score`; Gewinnquoten `SUIT_WIN_PERCENT`, `GRAND_WIN_PERCENT`, `NULL_WIN_PERCENT`, gemessen mit dem PIMC-Spieler). Verloren zählt doppelt. Ein Blatt, das für Farbe und Grand schwach ist und mehr als fünf offene Karten hat, wirft er weg. Unter dem Schieber (nur Hand) spielt er Null Hand nur mit höchstens zwei offenen Karten (`NULL_HAND_MAX_EXPOSED`), die er ohne Skat nicht loswird; beim Schwarzen Peter nie Null, weil ein gewonnenes Null nie Schneider ist und nichts zählt.
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
- Gespeichert wird im Browser des Spielers: laufender Run, Freischaltungen, Statistik der Runs. Seit ADR 0021 (07.11.2026) kann der Spieler zusätzlich ein Konto mit Passkey anlegen; dann liegt derselbe Stand auch auf dem Server und gleicht sich zwischen Geräten ab (14.4). Ohne Konto bleibt alles im Browser wie bisher. Seit 08.11.2026 läuft das Spiel auch offline (14.5).

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
| `full_skat` („Voll gedrückt“) | Ein Spiel gewinnen, in dem mindestens 21 Augen gedrückt wurden (`WinGame(min_discarded: 21)`). Gezählt werden die Augen der gedrückten Karten, wie sie im Spiel zählen (mit Aufdruck, Ramponiert und Skatpfleger), aus `RunEvent::GameScored::discarded_points`; ein Handspiel drückt nichts. Nur Farbspiel und Grand wie jedes `WinGame`. | Der Drückeberger, seit 04.11.2026 auch Rabattheft (8.11) |
| `three_schneider` („Maßarbeit“) | Drei Spiele mit Schneider in einer Kneipe gewinnen (`SchneiderWinsInPub(3)`, Farbspiel und Grand; der Zähler fängt mit jeder Kneipe neu an) | Die Schneiderin, seit 04.11.2026 auch Goldene Schere (8.11) |
| `true_colours` („Farbe bekennen“) | Drei Farbspiele hintereinander mit derselben Trumpffarbe gewinnen, egal welche (`SameTrumpStreak(3)`). Die Serie läuft über Kneipen hinweg im selben Run; ein verlorenes Spiel, ein Grand, ein Null oder eine andere Trumpffarbe dazwischen fängt neu an, ein neuer Run auch. Ein Ramsch ist keine Ansage: Er zählt nicht und unterbricht nicht (wie bei der Stammfarbe). | Bruderherz, seit 04.11.2026 auch Vereinsfarben (8.11) |
| `big_hand` („Hoch gepokert“, 10.11.2026) | Ein Handspiel gewinnen, das mindestens 96 Punkte zählt (`WinGame(min_risk: Some(Hand), min_value: 96)`). Gemessen: 20 von 20 Folgen nach im Mittel 13,8 Runs (1 bis 35) | Der Zocker und der Knobelbecher (9.6b) |
| `beat_ramscher` („Ausgeramscht“, 10.11.2026) | Den Boss Der Ramscher schlagen (`BeatBoss`). Gemessen: 20 von 20 Folgen nach im Mittel 10,4 Runs (1 bis 33) | Die Ramschtante (9.6c) |

Der Meta-Zustand hält dafür zwei kleine Zähler (`pub_schneider_wins`, `trump_streak`), die wie `current_section` aus den Ereignissen lernen; auch die bereinigten Spieler-Ereignisse tragen `discarded_points`.

**Bis 03.11.2026** schalteten `grand_master` („Großer Grand“, Grand Schneider mit mindestens 144 Punkten) den Grandspieler frei, `beat_kibitz_killer` („Blinde Kuh“, den Kiebitz-Mörder schlagen) den Zaungast und `full_till` („Volle Kasse“, 15 Groschen nach einer gewonnenen Kneipe) die Wirtin. Die drei Freischaltungen sind gestrichen. Wer sie verdient hat, behält ihre Ids im Meta-Zustand; sie schalten nichts mehr frei und stören nicht. Ein Meta-Stand mit Statistik oder Einsatz-Leiter eines gestrichenen Charakters lädt weiter: Diese Einträge fallen beim Laden weg. Hatte die Web-App einen gestrichenen Charakter als Wahl gespeichert, steht wieder der Alleinspieler da.

**Stammkunde** (02.11.2026, Welle 2b): zehn Runs zu Ende spielen (`RunsFinished(10)`, `ten_runs`) schaltet „Drei drücken“ frei (8.1). Damit sind 16 von 46 Hausregeln und Figuren gesperrt. Nicht im Simulator gemessen; zehn beendete Runs sind wie „Lehrjahre“ (fünf) sicher erreichbar.

Alle Freischaltungen sind erreichbar. Die letzte ist für den Bot fast unerreichbar, weil er Abschnitt 5 kaum schafft; für Menschen ist sie das Ziel des ersten Runs, wie in 9.2 entschieden.

**Kartenrückseiten**: Standard ist die Distel-Rückseite (nach Altenburger Vorbild, eigene Pixelgrafik). Seit dem 03.11.2026 hängt die Rückseite der Ausgabe des Spielers am Charakter (`back` in `characters.ron`, ohne Angabe die Distel; Sicht `CharacterInfo::back` und `CharacterEntry::back`): Alleinspieler und Nullspieler die Distel, der Drückeberger Briefmarke und Poststempel auf Packpapier, die Schneiderin Schere und Maßband auf Aubergine, Bruderherz ein großes rotes Herz auf Creme mit kleinen Herzen. Die drei neuen sind Pixelgrafik im Stil der Gegner-Rückseiten aus `tools/cardart/backs.py`. Seit ADR 0015 zeigt die Rückseite, wessen Ausgabe eine Karte ist: Die Rückseite des Charakters gehört dem Spieler und dem Skat; die Hände der Gegner zeigen eine Gegner-Rückseite, die als Daten am Gast oder Boss hängt (`back` in `guests.ron` und `bosses.ron`, ohne Angabe eine gemeinsame Rückseite der Stammgäste). Die Gegner-Rückseiten sind keine Freischaltung und nicht wählbar. Weitere Rückseiten werden später über bestimmte Erfolge freischaltbar (Freischaltung als Kosmetik, ohne Spielwirkung). Die Web-App führt die Rückseiten als kleine Liste von Entwürfen (`web/src/ui/backs.ts`, `cardBack(className, design)`); die Wahl ist eine reine Anzeige-Einstellung und berührt den Kern nicht.

**Statistik je Charakter und Relikt** (10.11.2026): Der Meta-Zustand zählt zusätzlich je Relikt, wie oft es genommen wurde (gekauft, gefunden, getauscht; Startrelikte nicht), wie oft es am Ende eines Runs gehalten wurde und wie viele dieser Runs gewonnen waren (`MetaStats::relics`), und je Charakter, welche Relikte er am Ende hielt (`MetaStats::character_relics`). Dafür tragen `RunWon` und `RunLost` die Relikte am Ende; ein Run, der in der langen Nacht endet, zählt nicht noch einmal. Ältere Meta-Stände laden (alle neuen Felder mit Standardwert; Test mit einem alten Stand in `core/tests/fixtures/`) und zählen ab dann. Der Kern rechnet das Statistikblatt (`MetaView::records`: Runs, Siege, Quote, weitester Stadtteil, bestes Spiel und die drei liebsten Relikte je Charakter; je Relikt genommen, am Ende, gewonnen, Quote). Das Vereinsbuch zeigt es auf der Seite „Statistik“ und auf der neuen Seite „Relikte“.

### 9.4 Der Drückeberger

Entschieden vom Nutzer am 03.11.2026. Drückt sich, wo er kann, und lebt von dem, was er drückt.

- **Ausgabe** (`Shirker`): eine Pfandmarke (`voucher_coins`, 2 Groschen) auf allen vier Siebenern, eine Totenmarke (`voucher_undead`) auf der Schellen-Acht, eine Narrenmarke (`voucher_wild`) auf der Rot-Acht. Die Namen der Marken kommen mit den Zinken (feat/zinken); im Kern heißen sie nach ihrer Id.
- **Regel, Teil 1: Marken kommen zurück.** Jeder Gutschein, der in einer Kneipe von seiner Ausgabe eingelöst wurde, liegt nach dem Ende dieser Kneipe (gewonnen oder verloren) wieder mit seinem Wert auf seiner Karte (`CharacterRule::vouchers_return`, gemerkt in `Series::returning_vouchers`, Ereignis `RunEvent::VoucherReturned` und `DeckChanged`). Das gilt für jeden Gutschein seiner Ausgabe, auch für einen später dazugekommenen. Trägt die Karte inzwischen eine andere normale Modifikation oder einen Fluch, bleibt der Gutschein weg (die Karte hat nur einen Platz, 7.2). Was die Marke aus der Karte gemacht hat, bleibt, und die Marke ist dann verbraucht: Hat eine Totenmarke (Narrenmarke) ihre Karte Untot (zur Wandelkarte) gemacht, kommt sie nicht zurück (Playtest 04.11.2026; vorher lag sie danach neben „Untot“). Zurück kommt sie nur, wenn sie wegen der Obergrenze Groschen gezahlt hat und die Karte blieb, wie sie war. Seit dem 04.11.2026 löst auch jedes Handspiel eine zufällige Marke der Hand ein (7.2); sie kommt genauso zurück. Endet der Run mit der Kneipe, kommt nichts mehr zurück, was keine Rolle spielt.
- **Regel, Teil 2: keine Hand-Stufe.** Ein Handspiel zählt ohne die Stufe „Hand“: ein Hook `OnDeclare(when: All([HandGame, Not(RamschGame)]), then: [AddLevels(-1)])`, gewonnen wie verloren, auch Null Hand und Null Ouvert Hand. In der Wertung steht „Hand +1“ und die Zeile des Charakters −1. Unter der Spelunke „Kein Skat“ (alles Hand) bekommt er nie Hand, sie bleibt für ihn ziehbar.
- **Startrelikt**: Skatpfleger (gedrückte Augen zählen doppelt).
- **Freischaltung**: `full_skat` (9.3). **Rückseite**: Briefmarke und Poststempel.
- **Zahlen**: Zielwerte wie der Alleinspieler (gemessen am 03.11.2026 im Zielbild, Tabelle in 9), `focus_weights` Trump 100, Null 50 (nach Augenmaß). Gemessen: Er löst je Run 4,7 Pfandmarken (9,7 Groschen), 1,1 Toten- und 1,1 Narrenmarken ein, fast alle kehren zurück (6,9 je Run); das sind gut 9 Groschen mehr als beim Alleinspieler, im Ergebnis liegt er nur knapp darüber.

### 9.5 Die Schneiderin

Entschieden vom Nutzer am 03.11.2026. Nimmt Maß und macht die Gegner Schneider.

- **Ausgabe** (`Tailoress`): Aufdruck (`points`) +3 auf allen vier Zehnern.
- **Regel**: Gewinnt sie ein Farbspiel oder einen Grand mit Schneider, zählt Schneider +2 statt +1 Stufe, Schwarz ebenso +2 statt +1. Ein gewonnenes Farbspiel oder ein Grand ohne Schneider zählt 1 Stufe weniger, nie unter 1 (die Formel hält die Stufen bei mindestens 1, 4.2). Daten: drei `OnScore`-Hooks mit `AddLevels` auf `WonSchneider`, `All([Won, AllTricksWon])` und `All([Won, Not(WonSchneider)])`, je ohne Null und Ramsch.
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

Gestrichen vom Nutzer am 03.11.2026. Der Grandspieler (Grand +1 Stufe, Farbspiel −1, Eichel-Daus mit Augen +8), der Zaungast (sah das Blatt des linken Gegners, einmal weniger einmischen) und die Wirtin (doppelte Groschen, vier Relikt-Plätze, Start mit einem Kräuter) sind aus `characters.ron`, dem `Character`-Enum, der Web-App und dem Simulator entfernt. Ihre Messungen in 5.2 und 11 bleiben als Geschichte stehen.

### 9.6b Der Zocker (10.11.2026)

Vom Nutzer früher vorgeschlagen, umgesetzt am 10.11.2026. Setzt alles auf eine Karte: Hand lohnt sich, der Skat ist knapp. Englisch „The Gambler“.

- **Ausgabe** (`Gambler`): Eichel- und Grün-Sieben sind Wandelkarten (`wild_card`, 7.2: sie nehmen bei Spielbeginn die Farbe an, von der er die meisten Karten hält, meist Trumpf). Die Schellen-Daus ist Durchsichtig (`transparent`): Hält er sie, wissen es die Gegner. Im ersten Entwurf waren alle vier Daus durchsichtig; mit der Kurve aus 9.6b unten blieb das zu teuer, gemessen mit allen Änderungen zusammen (Balance-Notiz vom 10.11.2026).
- **Regel, Teil 1: Hand zählt zwei Stufen.** Ein **gewonnenes** Handspiel (auch Null Hand, kein Ramsch) bekommt +1 Stufe zu der für Hand (`OnScore(when: All([Won, HandGame, Not(RamschGame)]), then: [AddLevels(1)])`, Zeile mit `Source::Character`). Ein verlorenes Handspiel kostet wie bei allen. Im Entwurf hing die Stufe an der Ansage (`OnDeclare`) und verdoppelte auch den Verlust; das kostete ihn zu viel.
- **Regel, Teil 2: Den Skat nur einmal je Kneipe.** `SkatPickupsPerPub(1)` (passiv): Danach ist jedes Spiel der Kneipe Hand; der Versuch, ihn noch einmal aufzunehmen, ist `RunError::SkatUsedUp`. Die Sicht nennt die übrigen Aufnahmen (`DeclareOptions::skat_pickups_left`), die Web-App schreibt „Den Skat darfst du in dieser Kneipe noch 1× aufnehmen.“ Der Boss Der Schieber (alles Hand) bleibt davon unberührt.
- **Startrelikt**: Knobelbecher (`dice_cup`, Ungewöhnlich, 6 Gr.): Ein verlorenes Handspiel zählt halb (× 0,5, aufgerundet), auch Null Hand.
- **Freischaltung**: `big_hand` („Hoch gepokert“): ein Handspiel gewinnen, das mindestens 96 Punkte zählt (`WinGame(min_risk: Some(Hand), min_value: 96)`); schaltet auch den Knobelbecher für alle frei. **Rückseite**: zwei Würfel und Jetons auf grünem Spieltischfilz (`gambler`).
- **Bot**: Er hebt die eine Aufnahme für ein schwaches Blatt auf, wenn die zehn Karten ein Handspiel zu mindestens 75 % gewinnen (`HAND_KEEP_MIN_PERCENT`) oder ein Null Hand mit höchstens einer offenen Karte möglich ist (`HAND_KEEP_NULL_EXPOSED`; Null Hand gewinnt mit einer 76 %, mit zwei 54 %); im letzten Spiel der Kneipe hebt er nichts auf.
- **Zahlen**: eigene Kurve **5 / 15 / 40 / 45 / Boss 80 · 45 / 55 / 60 / 70 / Boss 85 · 95 / 105 / 115 / 125 / Boss 100**, überall unter der des Alleinspielers, weil er in den meisten Spielen ohne Skat spielt (Punkte je Spiel 45). Gemessen 38,8 / 15,2 / 4,0 %, 18,1 % Ende in den ersten beiden Kneipen (7 × 300 Runs; mit dem Entwurf und der Kurve aus `run.ron` 25,0 / 6,8 / 1,7 %, 30,9 %). `docs/balance/2026-11-10-neue-charaktere.md`.

### 9.6c Die Ramschtante (10.11.2026)

Vom Nutzer früher als „Ramsch-Oma“ vorgeschlagen; der Name „Oma“ gehört dem Boss, darum Die Ramschtante. Umgesetzt am 10.11.2026. Englisch „Aunt Ramsch“.

- **Ausgabe** (`RamschAunt`): Ihre vier Daus und vier Zehnen sind abgegriffen: Ramponiert 5 (Daus 6, Zehn 5 Augen). Im Ramsch nimmt sie damit weniger Augen, in Farbspiel und Grand fehlen sie ihr. Die Flüche lassen sich reinigen.
- **Regel: Ramsch statt Ansage.** `OfferRamsch` (passiv): Vor dem Skat darf sie statt einer Ansage Ramsch spielen (`Declare(Ramsch)`, nur vor dem Skat, Sicht `DeclareOptions::ramsch_offered`, Knopf „Ramsch spielen“). Gewertet wie beim Boss Der Ramscher (11.1): Wer die meisten Augen hat, verliert; hat sie nicht die meisten, zählen die Augen des Verlierers für sie als Grundwert, +1 Stufe Jungfrau, wenn einer keinen Stich hat; verloren zählt er einfach wie jedes Spiel. Kein Gutschein, kein Kontra. Der Skat geht an den letzten Stich.
- **Startrelikt**: Spitzendeckchen (`lace_doily`, Ungewöhnlich, 6 Gr., nur für sie): Ein gewonnener Ramsch mit höchstens zwei Stichen zählt +1 Stufe.
- **Freischaltung**: `beat_ramscher` („Ausgeramscht“): den Boss Der Ramscher schlagen. **Rückseite**: Blumen der Kittelschürze und eine Kaffeetasse auf Kornblumenblau (`ramsch_aunt`).
- **Bot**: Er spielt Ramsch, wenn der Ramsch die beste Ansage und das Einmischen um mindestens 50 Punkte schlägt (`RAMSCH_MARGIN`, Hundertstel, `ramsch_danger` schätzt die Gefahr nach Untern und hohen Karten).
- **Zahlen**: Der Ramsch bringt ihr hohe Spielwerte (63 % ihrer Spiele, 86 % gewonnen, im Mittel 148 Punkte je gewonnenem Ramsch, Punkte je Spiel 92). Eigene Kurve **40 / 75 / 125 / 145 / Boss 235 · 120 / 140 / 160 / 180 / Boss 180 · 165 / 180 / 195 / 210 / Boss 160**. Gemessen 39,2 / 13,9 / 4,2 %, 11,4 % Ende in den ersten beiden Kneipen (7 × 300 Runs; mit der Kurve aus `run.ron` 71,3 / 48,3 / 22,3 %). `docs/balance/2026-11-10-neue-charaktere.md`.

### 9.7 Spätere Kandidaten (nicht vor v1)

- ~~**Der Ramscher**: Spielt Ramsch, gewinnt mit den wenigsten Augen. Eigenes Scoring.~~ Seit 10.11.2026 als Die Ramschtante (9.6c).
- **Der Schieber**: Sieht immer den Skat vor der Ansage, aber alle Spiele sind Hand. (Die Hand-Seite steckt seit 10.11.2026 im Zocker, 9.6b.)

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

Umgesetzt in M5 (`core/src/ai/`): `sample` verteilt die unbekannten Karten passend zu allem, was der Sitz weiß (Fehlfarben, Ouvert- und Partnerhand, der Skat nach dem Aufnehmen, beim Spieler Durchsichtig und Kiebitz); die Gegenspieler geben dem Alleinspieler dabei bevorzugt ein Blatt, das zu seiner Ansage passt. `solver` ist eine Double-Dummy-Suche (Alpha-Beta, Transpositionstabelle, gleichwertige Karten zusammengefasst). Farbspiel und Grand werden als Schwellenfragen gesucht (erreicht der Alleinspieler 61 Augen, danach Schneider), mit Schwarz angesagt und damit jedes Ouvert im Farbspiel und Grand als „bekommen die Gegenspieler einen Stich“ (seit 07.11.2026, Playtest), Null als „bekommt der Alleinspieler einen Stich“, Ramsch exakt in den letzten fünf Stichen. Hinter `depth` Stichen schätzt ein schnelles Ausspielen mit offenen Karten den Rest. Gleichstände entscheidet die Heuristik. Die Stichproben kommen aus einem eigenen Zufallsstrom (Seed des Runs, Spielnummer, Zahl der gespielten Karten, Sitz).

### 10.3 Schwierigkeit

Schwierigkeit steigt **nicht** primär über klügere KI, sondern über die Zielwertkurve und über Boss-Regeln. Die KI darf aber zwischen den Abschnitten von Stufe 1 auf Stufe 2 wechseln, mit mehr Samples je später im Run.

**Wissen schlägt Stufe (Playtest vom 07.11.2026):** Die Heuristik kann nicht verwerten, wo eine Karte liegt. Kennen die Gegenspieler Karten des Spielers (Ouvert, Durchsichtig in seiner Hand oder im Skat), spielen sie deshalb auch in Stadtteil 1 PIMC, mit der schwächsten PIMC-Stärke aus `opponents` (`opponents::informed_config`). Die Suchtiefe bleibt auch gegen eine offene Hand bei 3: Bis zum Ende gesucht (Tiefe 0) schneidet das Knotenbudget je Entscheidung die Stichproben ab, und die Gegner eines Null Ouvert fangen den Alleinspieler seltener (gemessen in `core/tests/defender_knowledge.rs`).

Stand M5 (`run.ron`, `opponents`): Abschnitt 1 Heuristik, Abschnitt 2 PIMC mit 6 Stichproben und Tiefe 2, Abschnitt 3 mit 8 und Tiefe 3, Abschnitt 4 mit 12, Abschnitt 5 mit 16 Stichproben (Tiefe 3). Messung: `docs/balance/2026-11-01-m5-pimc.md`.

### 10.4 Stammgäste

Entschieden am 01.11.2026 (Cockpit): Die normalen Gegner sind wechselnde Stammgäste mit Namen und Pixelgesicht, ohne eigene Regeln; Regeln bleiben den Bossen. Umgesetzt in M6 (`core/content/guests.ron`, 14 Gäste, im Code `Guest`):

- Zu Beginn jeder Serie setzen sich zwei verschiedene Gäste (Sitz 1 links, Sitz 2 rechts). Gezogen wird aus einem eigenen Zufallsstrom, der nur vom Seed des Runs und der Serie abhängt; kein anderer Strom rückt dadurch weiter, Simulator-Ergebnisse bleiben gleich. Die beiden Gäste der vorigen Serie setzen aus, so kommt nie dasselbe Paar zweimal hintereinander.
- In der Boss-Serie saß bis ADR 0017 links der Boss, rechts ein Stammgast; seitdem sitzen dort beide Figuren des Bosses.
- Spielstände von vorher bekommen die Gäste der laufenden Serie aus dem Seed, ab der nächsten Serie wie oben.
- **Sprüche** (bis 11.11.2026 drei je Gast: Begrüßung, eigener Stich, Stich des Spielers; die Oberfläche zeigte beim ersten Spiel einer Serie und höchstens etwa jedes zweite Spiel nach einem Stich ab 10 Augen einen davon). Seit dem 11.11.2026 („Sprüche v2“, entschieden von Claude im Rahmen von Säule „Stammtisch, nicht Turnier“) hat jeder Gast für elf Anlässe je zwei bis drei Varianten (`quips` in `guests.ron`, im Kern `Quips` und `QuipSituation`, 25 Sprüche je Gast):

  | Anlass (`quips`) | Wann | Wie oft |
  |---|---|---|
  | `greet` | erstes Spiel der Serie (steht auch auf dem Filz) | immer |
  | `win`, `lose` | Stich ab 10 Augen an den Gast oder an den Spieler | nach mindestens 14 ruhigen Stichen |
  | `smear` | der Gast schmiert: Daus oder Zehn in den Stich seines Partners | nach 9 ruhigen Stichen |
  | `grand`, `null`, `bold` | der Spieler sagt Grand, Null oder Hand, Schneider, Schwarz, Ouvert an (Null geht vor, dann die Risikostufe, dann Grand) | nach 7 ruhigen Stichen, zu 60 % |
  | `ramsch` | der erste Ramsch einer Serie beginnt | wie die Ansage |
  | `won`, `lost` | nach der Abrechnung: die Gäste gewinnen, der Spieler gewinnt | nach 7 ruhigen Stichen, zu 45 % |
  | `schneider` | der Spieler gewinnt mit Schneider oder Schwarz | nach 3 ruhigen Stichen |

  Ruhig heißt: ohne Sprechblase seit so vielen Stichen. Im Mittel kommt so etwa eine Blase je Spiel, nie zwei kurz hintereinander, immer höchstens eine auf dem Tisch, nie zusammen mit einem Hinweis des Wirts; die Begrüßung kommt immer durch. Im Ramsch (Ramschtante) schweigen die Gäste zu Stichen, ihre Stich-Sprüche gelten einem Spiel gegen den Spieler. Nach dem letzten Spiel einer Kneipe gibt es keinen Ergebnis-Spruch (der Tisch ist weg). Welche Variante kommt, entscheidet die Oberfläche mit `Math.random`, nie dieselbe zweimal nacheinander für dieselbe Person und denselben Anlass (`web/src/ui/quips.ts`, Zeitplan `web/src/app/talk.ts`); die Begrüßung ist je Serie fest, damit Blase und Filz dasselbe sagen. Das ist Dekoration: kein Zufallsstrom des Kerns, kein Einfluss auf Simulator und Spiel. Die Blase bleibt 2,8 Sekunden (Tempo-Einstellung, mindestens 1,8), bei reduzierter Bewegung ohne Einblenden, für Bildschirmleser als `status`. Höchstens 40 Zeichen auf Deutsch (Test im Kern), 42 auf Englisch (`check-i18n`). Ein alter Inhalt mit einem einzelnen Spruch je Anlass lädt weiter. Spielstände enthalten nur die Ids der Gäste, die Sprüche kommen immer aus dem Inhalt.
  Den Anlass „Stammgast neben dem Boss“ gibt es nicht: Seit ADR 0017 sitzen in der Boss-Kneipe beide Figuren des Bosses (die haben eigene Sprüche, 11).
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

**Seit ADR 0017 (03.11.2026, Entscheidung des Nutzers, Brief „Bosse v2“)** ist jeder Boss ein **Paar aus zwei Boss-Spielern** an der Boss-Kneipe am Ende jedes Stadtteils (drei je Abend, die letzte der Endboss). In der Boss-Kneipe werden alle drei Spiele gespielt; verloren ist der Run vorbei, gewonnen gibt es doppelte Groschen (gerechnet, als blieben zwei Spiele übrig, `boss_reward.games_left_as`) und ein Relikt nach Wahl (5.0; bis ADR 0019 auch ein Leben). Was weiter unten „Boss-Serie“ heißt, ist die Boss-Kneipe; die Messungen unten galten einem Boss-Platz, dem frühen Ende und den fünf Abschnitten und sind Geschichte.

Die Sonderregel gilt für die ganze Boss-Kneipe und wird vorher angezeigt, damit der Spieler sein Blatt und seine Ansagen anpassen kann. Der Boss eines Stadtteils wird beim Start des Stadtteils gezogen (Ereignis `BossAnnounced`) und ist ab dann in der Sicht lesbar (`upcoming_boss`, `next_series`, Routenwahl), mit beiden Figuren (`BossInfo::figures`). Jeder Boss hat einen Zielwert-Faktor (`target_factor` in `bosses.ron`, Standard 1,0) und kann für einzelne Charaktere einen eigenen haben (`character_factors`, ADR 0012); messen lässt sich das mit `skat_sim --curve --boss <id>` (seit ADR 0017 zählt dabei der Stand nach dem dritten Spiel).

### 11.1 Die Bosse seit ADR 0017

| Boss | Regel | Figuren (links, rechts) | Ausgaben | Thema | Faktor (vorläufig) |
|---|---|---|---|---|---|
| **Der Schieber** | Jedes Spiel ist ein Handspiel (`RequireHand`) | Der Schieber, Die Hehlerin | Siebenen Augen +4; Achten Augen +3 | Schmuggelware: Luschen, die doch etwas wert sind | 0,5 (Nullspieler 0,2) |
| **Die Oma** | Zu Beginn ihrer Kneipe tragen 16 der 32 Karten der Ausgabe des Spielers je einen zufälligen Fluch (Durchsichtig, Ramponiert, Fleckig, Standardwert), nur für diese Kneipe (`CurseForPub`) | Die Oma, Der Opa | Oberinnen Augen +3 (6 statt 3); Könige Augen +2 (6 statt 4) | aus der alten Zeit: die alte Oberin, der alte König | 1,0 (Nullspieler 0,9) |
| **Die Zwillinge** | Die Gegner sehen gegenseitig ihre Blätter und spielen danach (`DefendersSeeEachOther`, Schummeln) | Fritz, Franz | Achten Rang 2; Neunen Rang 2 | sie sehen doppelt: zwei Nullen übereinander, die Neun gleich mit | 1,2 |
| **Der Ramscher** | Jedes Spiel ist ein Ramsch (`Ramsch`, bis ADR 0017 nur das erste); ein verlorener Ramsch zählte vom 03.11. bis 04.11.2026 als einzige Ausnahme einfach (`SingleLoss`), seit 04.11.2026 zählt jedes verlorene Spiel einfach (4.3) | Der Ramscher, Die Ramscherin | Daus Ramponiert 6; Zehnen Ramponiert 5 | sie lassen laufen | 1,0 (Nullspieler 2,0) |
| **Der Schwarze Peter** | Ein gewonnenes Spiel ohne Schneider zählt halb | Der Schwarze Peter, Die Schwarze Grete | Eichel- und Grün-Unter Durchsichtig; Eichel- und Grün-Oberin Durchsichtig | Ruß an den Fingern: eine Hilfe beim schwersten Boss | 0,7 (Nullspieler 0,2) |
| **Der Alte** (Endboss) | legt die fünf Regeln oben der Reihe nach auf, eine pro Blatt (Schieber, Zwillinge, Ramscher, Oma, Schwarzer Peter) | Der Alte, Die Alte | die Ausgaben der Figuren des Bosses, dessen Regel gerade gilt (links die erste, rechts die zweite) | das alte Paar | 1,3 (seit Runde 2, 04.11.2026; vorher 1,0) |
| **Der Totengräber** (nur in der langen Nacht, seit 10.11.2026) | Die gedrückten Karten zählen keine Augen (`MultiplySkatPoints(0)`; nicht im Handspiel und im Null) | Der Totengräber, Die Witwe | Neunen Rang 2; Daus Augen +4 | vom Friedhof: begrabener Skat, Erbstücke der Witwe | 2,5 (in der Nacht gemessen) |

**Sprüche der Figuren** (11.11.2026, mit den Sprüchen v2 der Stammgäste, 10.4): Jede Figur jedes Bosses, auch Der Alte, Die Alte, Der Totengräber und Die Witwe, hat in der Stimme ihres Bosses je zwei Varianten für sieben Anlässe (`figures[].quips` in `bosses.ron`, Sicht `BossFigureInfo::quips`, nur Darstellung): `greet` beim Auftritt (die linke Figur auf dem Boss-Blatt vor der Kneipe, die rechte als Blase beim ersten Spiel), `win` und `lose` bei einem Stich ab 10 Augen, `won` und `lost` nach einem Spiel, `beaten` auf der Relikt-Wahl nach der gewonnenen Boss-Kneipe (beim Alten auf dem Endbildschirm) und `victory` auf dem Endbildschirm, wenn der Boss die Kneipe gewinnt. Zeitplan und Wahl der Variante wie bei den Stammgästen.

Entfallen mit ADR 0017: **Der Kontra-Hans** (Kontra immer erlaubt), **Der Revolutionär** (Handtausch nach dem fünften Stich) und **Der Kiebitz-Mörder** (Informations-Wirkungen aus). Mit ihnen die Wirkungen `SwapDefenderHands`, `DisableInformation` und `BlankOut` (die alte Regel der Oma); `EnableKontra` bleibt für die Hausregel. Ein Spielstand, der einen von ihnen gezogen hat, trifft einen Ersatz (`RETIRED_BOSSES`: Kontra-Hans und Revolutionär die Zwillinge, Kiebitz-Mörder der Schwarze Peter; keiner verlangt Hand, damit ein wartender Stand spielbar bleibt). **Der Totengräber** ruhte als Boss mit Sonderkarten bis zum 10.11.2026; seitdem ist er der Boss der Nacht (unten).

**Der Totengräber** (10.11.2026, der erste Boss nur der langen Nacht, `night_bosses` in `run.ron`; englisch „The Gravedigger“ und „The Widow“): Er begräbt den Skat. Die gedrückten Karten zählen in seiner Kneipe keine Augen (`OnDeclare(when: All([Not(HandGame), Not(NullGame)]), then: [MultiplySkatPoints(0)])`, die Wirkung des Skatpflegers mit 0, sie schlägt ihn auch). Ein Handspiel drückt nichts und behält seinen Skat; im Null zählen Augen nicht. Figuren: Der Totengräber (Neunen Rang 2: die Neun wird König), Die Witwe (Daus Augen +4, 15 statt 11). Er kommt zu den Bossen des Tages und dem Alten in die Ziehung der Nacht (5.5), nie in einen normalen Run. Faktor **2,5**: Der Build der Nacht ist so stark, dass der Alleinspieler mit 1,0 92 % der Kneipen des Totengräbers gewann (Nullspieler 94 %), mehr als gegen jeden anderen Boss der Nacht; mit 2,5 83 % (Nullspieler 79 %), so oft wie gegen die mittleren Bosse (gemessen in der Nacht, `docs/balance/2026-11-10-neue-charaktere.md`).

Leitlinien und Auslegungen (Claude, 03.11.2026):

- **Ausgaben**: je Figur eine Gruppe von vier Karten (der Peter und die Grete je zwei), eine Modifikation je Karte, wie die milde Stufe von ADR 0015. Beide Figuren teilen das Thema, unterscheiden sich aber in der Kartengruppe oder im Wert. Beide zeigen die Rückseite des Bosses (`back`; eine Figur darf eine eigene haben, `figures[].back`). Die Figuren des Alten haben keine eigene Ausgabe (Ladefehler sonst), ihre Rückseite ist seine.
- **Die Oma**: Die Flüche zieht der Generator der Route (`tour_rng`), Karte und Fluch je Karte; ein Fluch, der nicht auf die Karte passt (Fleckig nur auf Farbkarten), wird nicht gelegt. Er tritt an die Stelle der normalen Modifikation, eine Spezialkarte (Untot, Wandelkarte, Abgestuft) bleibt. Jede Karte wird vorher gemerkt (`Series::boss_curses`) und am Ende der Kneipe genau so zurückgelegt; keine Groschen, kein `OnAttach`. Unter dem Alten legt sie die Flüche beim ersten Blatt mit ihrer Regel, und sie bleiben bis zum Ende der Kneipe. Seit ADR 0017 trifft sie jeden Charakter (bis dahin nur den Alleinspieler).
- **Der Ramscher**: Jedes Spiel unter seiner Regel ist ein Ramsch. Mit drei Ramsch-Spielen entschied vor allem, ob einer verloren ging (doppelt gegen den Spieler); der Faktor wirkt kaum. **Entschieden von Claude am 03.11.2026 (17 a)**: In seiner Kneipe (und unter seiner Regel beim Alten) zählt ein verlorener Ramsch nur einfach. Das ist eine eigene Wirkung der Boss-Regel (`SingleLoss` in `passive`, dieselbe Zeile `LossFactor` wie beim Magenbitter), kein Sonderfall im Code. Gemessen vorher ohne Relikte 45 % (Stadtteil 1) und 37 % (Stadtteil 3), gegengerechnet mit einfacher Wertung 64 und 56 % (`docs/balance/2026-11-03-kleine-runde.md`, Abschnitt 3). Der Faktor bleibt 1,0. Seit dem 04.11.2026 zählt jedes verlorene Spiel einfach (4.3); die Wirkung `SingleLoss` ist aus seinen Daten genommen und ruht.
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

  **Runde 1 ohne Leben** (04.11.2026, „verloren zählt einfach“, `docs/balance/2026-11-04-runde1.md`), Alleinspieler mit der neuen Kurve (Boss 60 / 80 / 130), Boss-Kneipe gewonnen in Stadtteil 1 (je 76 bis 97 Kneipen) und ab Stadtteil 2 (26 bis 60): Zwillinge 74 / 74 %, Schieber 73 / 80 %, Oma 71 / 88 %, Schwarzer Peter 67 / 77 %, Ramscher 56 / 52 %, der Alte 77 %. In Stadtteil 1 ist die Reihenfolge bei allen fünf Charakteren ähnlich (Zwillinge am leichtesten, Ramscher am schwersten); der Schieber beim Nullspieler bleibt mit 43 % der Ausreißer. **Faktoren unverändert**; der Alte ist als Endboss mit 77 % eher zu leicht (Runde 2).

  **Runde 2** (04.11.2026, `docs/balance/2026-11-04-runde2.md`): Der Alte gewann vorher beim Alleinspieler 78 % (92 Kneipen), bei den anderen vier Charakteren 87 bis 89 %; Faktor 1,0 → 1,3, nachher 71 % (75) und 74 bis 81 %. Die Boss-Kneipe in Stadtteil 1 hat seit Runde 2 Zielwert 75 statt 60 (5.0.4): Alleinspieler Zwillinge 74 %, Schieber 74 %, Oma 70 %, Schwarzer Peter 65 %, Ramscher 49 %. Der Ramscher bleibt bei allen Charakteren außer dem Nullspieler 15 bis 25 Punkte unter den anderen; seine Kurve ist flach, ein kleinerer Faktor hülfe kaum (Runde 3: Regel statt Zahl?). Die anderen Faktoren unverändert.
- **Belohnung**: Die Relikt-Wahl zieht der Ereignis-Generator aus dem Relikt-Pool des Charakters (nicht gesperrt, nicht besessen), Seltenheit Ungewöhnlich oder höher (`boss_reward` in `run.ron`); ohne passendes Relikt geht es direkt zur Routenwahl. Bei vollen Plätzen (acht seit 05.11.2026, 8.0) ersetzt das gewählte ein eigenes (halber Preis).

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

**Seit ADR 0016**: Groschen je gewonnener Kneipe (Grundbetrag 3, je übrigem Spiel 2 seit 03.11.2026, vorher 3 und vor ADR 0016 1, Zinsen wie bisher); eine verlorene Kneipe zahlt nichts. Eine gewonnene Boss-Kneipe zahlt seit 03.11.2026 so, als blieben zwei Spiele übrig, und das doppelt (`boss_reward.games_left_as`, 5.0). Der Tresen kommt nur noch als Station eines Weges (5.0.1), nicht mehr nach jeder Serie; die Spelunke bringt ein Relikt oder seinen halben Preis. Relikt-Plätze: vom 03.11. bis 05.11.2026 unbegrenzt, seitdem acht (8.0); bei vollen Plätzen tauscht der Tresen (das alte Relikt für den halben Preis), Verkaufen allein bleibt möglich.

**Gemessen am 03.11.2026** (Alleinspieler, PIMC-Spieler, `docs/balance/2026-11-03-grosse-runde.md`): Ein Run bringt im Mittel 31,6 Groschen aus Kneipen (vorher 55,8), dazu 4 Startkapital, 2,4 aus Verkäufen und 2,4 aus Inhalten; ausgegeben werden 36,9, davon Relikte 60 %, Zinken 19 %, Schnäpse 19 %, Kartenmaler 1 %, Neuauslage und Reinigen unter 1 %; Neu auslosen der Zinken nutzt der Bot nie. 4,0 Tresen-Besuche je Run. Eine gewonnene normale Kneipe zahlt 4,6 bis 6,0, eine gewonnene Boss-Kneipe 14,5 (vorher 6,45, weniger als eine Kneipe). Die fünf Relikt-Plätze sind beim Boss 1 im Mittel zu 4,0 belegt, ab Stadtteil 2 voll (vorher schon beim Betreten von Stadtteil 2 zu 93 %). **Preise seit 03.11.2026** nach der Einzelmessung ohne Tresen: Skatgott 9 → 12, Ein Auge zudrücken 8 → 7, Schieben 8 → 6, Schenken 8 → 6, Schutzengel 8 → 6, Feierabend 5 → 4.

**Gegenstands-Review (02.11.2026, 8.5), vorläufig ohne Messung**: Kräuter 3 (Common), Doppelkorn 3 (Uncommon), Konterbier 4 (Uncommon), Schwenker 2 (Common), Wegbier 3 (Common); Spickzettel 5 (bisher 4); Feierabend 5 (Common), Monatskarte 4 (Common), Stammtischkasse 5 (Uncommon), Deckel anschreiben 5 (Uncommon), Schutzengel 8 (Rare), Spelunkenkenner 4 (Uncommon). Neu in der Wirtschaft: Schulden am Tresen (Deckel anschreiben, bis −5, einmal pro Stadtteil), eine Kasse (2 je Kneipe, +50 % beim nächsten Tresen), die Wette (3 oder 6 Groschen) und Groschen aus Begegnungen (Wirt 5, Portemonnaie 8). Der Falschspieler kostet jetzt 2 Groschen, statt 3 zu zahlen.

**Wege und Begegnungen v2 (03.11.2026, 5.0.1 und 13.0), vorläufig ohne Messung**: Neue Einnahmen unterwegs: Fundgroschen 2, die Gratis-Optionen der Ereignisse (Kartengeber 2, Wasser 1, Kiebitzen beim Preisskat 2, Falschspieler auffliegen lassen 4, Finderlohn 4, altes Blatt 4, Kiebitz 2, Wirt 5), ein Relikt zum vollen Preis beim Alten Bekannten, ein Schnaps bei vollen Plätzen sein Tresen-Preis in Groschen. Neue Ausgaben: eine Runde ausgeben 3, aufwischen 2, die Wette auf einen Sieg 2 (zahlt 3), die Würfelbude 2, 4 oder 6 (Erwartungswert 0). Der Kiosk verkauft einen von zwei Schnäpsen 1 Groschen unter dem Tresen-Preis (nach dem Rabatt des Wirts, mindestens 1), ohne Anschreiben.

**Rettungs-Schnäpse (03.11.2026, 8.7), vorläufig ohne Messung**: Klarer 3 (Common), Absacker 4 (Uncommon, wie das Konterbier: ein Spiel mehr ist fast eine zweite Chance), Eierlikör 3 (Common). Kaufwerte des Bots 1, 2, 1.

**Schnaps-Leiste (03.11.2026, 8.9)**: Jede gewonnene Kneipe (außer der letzten) bringt einen Schnaps; bei vollen Plätzen ist er seinen halben Tresen-Preis wert (1 bis 2 Groschen). Drei statt zwei Schnaps-Plätze. Kiosk und Tresen verkaufen weiter Schnäpse; der Bot kauft kaum weniger (1,5 statt 1,6 je Run, gemessen in 8.9).

**Flüche als Handel (03.11.2026, 8.8), vorläufig ohne Messung**: Pechvogel 9 (Rare; Bruderherz hat ab Start +9 Grundwert), Fusel 2 (Common). Schnäppchen (etwa jedes fünfte Angebot der Auslage) und Hehlerware (etwa jede vierte Auslage der Relikte) kosten die Hälfte, abgerundet, mindestens 1; der Fluch eines Handels zahlt keine Groschen (andere Flüche weiter 3). Kaufwerte des Bots: Pechvogel 4, Fusel 1, ein Handel 1 weniger als ohne Fluch.

- **Währung**: Groschen. Thematisch Zehntelpfennig, wie am Kneipentisch.
- **Einnahmen**: Grundbetrag pro gewonnener Serie, Bonus pro übrigem Spiel, Zinsen auf Guthaben (gedeckelt), Verkauf von Relikten.
- **Ausgaben am Tresen**: Hausregeln, Figuren, Verbrauchsgegenstände (seit 02.11.2026 nur Schnäpse), Zinken (Auslage, Kartenmaler, Karte reinigen; 8.6), Neu auslosen (alles, einmal je Besuch, seit 05.11.2026; vorher Neuauslage und Neu auslosen der Auslage getrennt).
- **Zinken (02.11.2026, 8.6), vorläufig ohne Messung**: Auslage 2 bis 4 Groschen (+1 auf Unter und Daus), Neu auslosen 1 (+1 je weiteres Mal am selben Tresen; seit 05.11.2026 eins für den ganzen Tresen zu 2, einmal je Besuch), Kartenmaler 6, Karte reinigen 3. Der Wirt nimmt auf alles 20 %.
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

**Ereignisse sind Daten.** In `encounters.ron` listet jedes Ereignis seine Optionen (`options`, `EncounterOptionDef`): einen Titel (`label`), einen Preis (`cost`, bei Wette und Würfelbude der Einsatz) und eine kleine Liste von Wirkungen (`effects`, `EncounterEffect`), die nacheinander gelten. Es gibt keine Rust-Variante je Ereignis mehr (bis dahin `EncounterKind` und `EncounterChoice`). Die Wirkungen: Groschen (`Coins`), eine Hausregel des Stadtteils (`EveningRule`), eine Zinke oder ein Fluch auf eine zufällige oder gewählte Karte (`Mark`, mit `card: Random | Chosen`, benannter Zinke, Angeboten zum Ziehen, `curse` und `curse_percent`), ein Relikt tauschen oder verkaufen (`SwapRelic`, `SellRelic(Prozent)`), ein Relikt oder sein Gegenwert (`Relic`, `RelicCoins`, die Beute der Spelunke), das nächste Spiel ×n (`NextGameFactor`), Einmischen in der nächsten Kneipe mehr oder im ersten Spiel weniger (`NextPubRedeals`), beide Skatkarten im nächsten Spiel (`SeeSkat`: wie der Kurzer bei jedem Geben, bis ein Spiel angesagt ist), ein Schnaps (`Schnapps`), Leben (`Lives`), die nächsten Wege (`Scout`), eine Wette (`Bet` auf Schneider oder Sieg, Auszahlung in Prozent des Einsatzes), Würfeln (`Dice`, Gewinnchance und Auszahlung in Prozent), der Kartenmaler und Karte reinigen (`Painter`, `Cleanse`), seit 04.11.2026 eine Marke (`Marke`, unten). Was eine Wirkung offen lässt (`None`), wird beim Öffnen mit dem Ereignis-Generator gezogen und vorher gezeigt: Hausregeln und Schnäpse verschieden je Option, ein Relikt für das ganze Ereignis, eine Zinke je Wirkung. Eine Option mit `fallback` („Zuschauen“) gibt es nur, wenn sonst nichts geht.

**Gesperrt statt versteckt.** Eine Option, die gerade nicht geht, bleibt sichtbar und nennt den Grund (`OptionBlock` in der Sicht, `blocked`): zu wenig Groschen, nichts mehr zu ziehen, keine passende Karte (die Spezialkarten-Grenze der Ausgabe entfiel am 05.11.2026, 7.2), kein Relikt, alle Relikt-Plätze belegt (seit 05.11.2026 nur ohne ein Relikt, das Platz machen kann: bei acht vollen Plätzen ersetzt das neue ein gehaltenes, 8.0), eine Wette läuft schon (bis ADR 0019 auch: kein Leben fehlt). **Jedes Ereignis hat eine Option, die nichts kostet** (geprüft beim Laden), meist mit Gewinn; nur die Wette setzt in jeder Option und kommt deshalb nur auf einen Weg, wenn der kleinste Einsatz bezahlbar ist. Ein Ereignis, von dem gerade keine Option genommen werden kann, kommt nicht auf einen Weg (5.0.1).

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
| Verlorenes Portemonnaie | behalten: +8 Gr., Durchsichtig auf zufällige Karte (Fluch) | abgeben: ein Schnaps (bis ADR 0019 +1 Leben) | Finderlohn: +4 Gr. |
| Der Skatlehrer | Winkel +1 auf gewählte Karte, im ersten Spiel der nächsten Kneipe einmal weniger einmischen | Theorie: beide Skatkarten im nächsten Spiel | ein Kurzer |
| Die Wette | 3 Gr. auf Schneider (×2 zurück) | 6 Gr. auf Schneider (×2) | 2 Gr. auf Sieg (×1,5, also 3) |
| Alte Karten im Schrank | Totenmarke auf gewählte Karte, ein Fluch auf eine zufällige andere | Narrenmarke ebenso | altes Blatt verkaufen: +4 Gr. |
| Der Pfandleiher (neu, `pawnbroker`) | Marke aussuchen, 2 Gr.: die gewählte Marke auf eine zufällige Karte | Pfandschein einlösen: +3 Gr. | |
| Die Markensammlerin (neu, `stamp_collector`) | Marke kaufen, 5 Gr.: gewählte Marke auf gewählte Karte | Marke tauschen: ebenso gratis, ein Fluch auf eine zufällige andere Karte | nur schauen: +1 Gr. |
| Kiebitz am Fenster | die nächsten 2 Wege genau sehen | +1 Einmischen in der nächsten Kneipe | +2 Gr. |
| Fundgroschen (neu, `found_coins`) | Aufheben: +2 Gr. | | |
| Die Würfelbude (neu, `dice_stall`) | 2 setzen | 4 setzen | 6 setzen; mit 50 % doppelt zurück. Ohne 2 Groschen nur „Zuschauen“ |

Dazu die Kartenwerkstatt (Kartenmaler gratis oder Karte reinigen, seit 04.11.2026 als dritte Option der Griff in die Markenkiste: eine zufällige Marke auf eine Karte deiner Wahl, gratis), die Imbissbude (seit ADR 0019 Rast ohne Leben: Currywurst, ein Einmischen mehr in der nächsten Kneipe, oder eine Karte waschen; bis dahin +1 Leben) und die Beute der Spelunke (das Relikt oder sein halber Preis, `RelicCoins(50)`). Der Geburtstag (enc-06) ist nicht gebaut. Alle Zahlen **vorläufig**, nach Augenmaß; der Bot wählt mit einem einfachen Wert je Option (`RunBot::option_value`, 5.0.4). Stichprobe nach dem Umbau (`skat_sim --seed 42 --runs 300 --bot pimc:12:3`, keine Kalibrierung): Stadtteil 2 erreicht 40,3 % (vorher etwa 41 %), Stadtteil 3 16,3 %, Run gewonnen 6,3 % (vorher etwa 6 %); je Run 3,5 Tresen-Besuche (vorher 4,0), 0,2 Kiosk-Besuche und 1,5 Fundgroschen, denn der Bot nimmt den Fundgroschen-Weg gern (Wert 2 gegen 1 für eine Begegnung). Beim Kartengeber und bei der Krone nimmt er fast immer die Groschen oder den Schnaps statt der Zinke.

**Mehr Wege zu Marken (Wunsch des Nutzers vom 04.11.2026, Ausgestaltung von Claude).** Neue Wirkung `Marke { card, marke }` (je `Random` oder `Chosen`): eine Marke (jede Zinke der Art Gutschein: Pfandmarke, Totenmarke, Narrenmarke, `Run::marken`) auf eine Karte der Ausgabe des Spielers. Drei Formen in drei Ereignissen:

- **Zufällige Marke auf gewählte Karte** (Kartenwerkstatt, „Griff in die Markenkiste“, gratis): Der Spieler wählt die Karte; die Marke wird erst beim Nehmen gezogen, nach Seltenheit (`rarity_weights` des Tresens: Pfandmarke gewöhnlich, Toten- und Narrenmarke selten), nur unter den Marken, die auf diese Karte dürfen. Bewusst eine Ausnahme von „vorher gezeigt“: Das ist der Reiz der Option, die Sicht nennt die möglichen Marken mit ihrer Chance (`MarkeChoiceView::percent`).
- **Gewählte Marke auf zufällige Karte** (Der Pfandleiher, neu, 2 Groschen): Der Spieler wählt die Marke, die Karte wird mit dem Ereignis-Zufall gezogen, eine ohne Zinke zuerst (wie beim Kartengeber).
- **Gewählte Marke auf gewählte Karte** (Die Markensammlerin, neu): 5 Groschen, oder gratis mit einem Fluch auf einer zufälligen anderen Karte (wie die Alten Karten im Schrank).

Es gelten die Regeln jeder Zinke (7.2): eine normale Zinke je Karte (eine andere wird ersetzt, dieselbe steigt), keine Marke auf eine verfluchte Karte (Toten- und Narrenmarke seit 05.11.2026 ohne Obergrenze der Spezialkarten, 7.2); der Preis einer gewählten Karte läuft über `zinken_price` (Bruderherz). Die gewählte Marke kommt in `RunAction::ChooseEncounter { marke }`; die Sicht trägt je Marke-Option die Marken mit `can_choose`, bei gewählter Karte deren Ziele und Preise (`EncounterOptionView::marken`), das Ereignis `MarkeGiven { card, card_mod, drawn, random_card }` sagt, was wohin kam. Die Oberfläche zeigt zum Wählen der Marke ein kleines Blatt mit den drei Marken (Bild und Text), danach die Kartenauswahl mit der Bestätigung „Zinke anbringen?“ (vorher, nachher). **Bot**: eine Marke wie eine Zinke (1 auf zufälliger, 2 auf gewählter Karte); muss er wählen, nimmt er die wertvollste nach `ZINKEN_VALUES` (die Pfandmarke). Preise und Gewichte (je 1) vorläufig nach Augenmaß, nicht gemessen.

**Gegenstands-Review (02.11.2026, 8.5)**: neu sind Der Wirt spendiert, Verlorenes Portemonnaie, Der Skatlehrer, Die Wette, Alte Karten im Schrank und Kiebitz am Fenster; der Falschspieler heißt jetzt „Falschspieler am Nebentisch“ und verkauft einen starken Kartentrick, der in der Hälfte der Fälle ein Fluch ist (die alte Form, ein Fluch gegen Groschen, entfällt; `MarkCard { curse: true }` bleibt in den Daten möglich). Alle übrigen Stationen bleiben. Werte und Gewichte (je 1) vorläufig.

**Seit ADR 0016** sind Ereignisse Stationen auf einem Weg zwischen zwei Kneipen (5.0.1) und tragen eine Kategorie: Begegnung (Stammtisch-Streit, Alter Bekannter, Letzte Runde, Umgekipptes Bier, Preisskat, Falschspieler, Gast aus Altenburg), Upgrade (Kartengeber, Gasthof zur Krone, Taschenspieler, neu die Kartenwerkstatt: eine Modifikation auf eine gewählte Karte oder eine Karte säubern, beides gratis; seit 8.6 statt der Modifikation der Kartenmaler gratis, eine von zwei Zinken) und Rast (neu die Imbissbude: ein Leben zurück; seit ADR 0019 ein Einmischen mehr oder eine Karte waschen). Die Wahrscheinlichkeit `encounter_chance` entfällt. Die Beute der Spelunke ist ein Ereignis, das nie auf einem Weg liegt (`spelunke_loot`, Gewicht 0). Der Stammtisch-Streit gilt bis zum Ende des Stadtteils.

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
├── server/          Crate „stammtisch_server“: Konten, Spielstand, Tafel des Tages (ADR 0021), kennt skat_core nicht
└── web/             Web-App (TypeScript, Vite): Tisch, Hand, Tresen, Animation, Speichern
```

**Kern im Web Worker (ADR 0022).** Im Browser läuft der Kern nicht auf dem Hauptthread, sondern in einem eigenen Worker (`web/src/core.worker.ts`). Die Web-App spricht nur über `web/src/core.ts` mit ihm: Jeder Aufruf ist eine Nachricht mit Id und gibt ein Promise zurück; der Worker beantwortet eine nach der anderen in der Reihenfolge, in der sie kamen. Der Hauptthread behält die letzte Sicht und den letzten Spielstand als Text jedes Runs und des Meta-Zustands, damit Zeichnen und Speichern beim Verlassen der Seite nicht auf den Worker warten. Während ein Gegner denkt (PIMC), laufen Denkblase, Animationen und Eingaben weiter; Eingaben ins Spiel bleiben wie bisher gesperrt, bis die Gegner gespielt haben. Ohne Worker (oder ohne WebAssembly darin) läuft dieselbe Hülle (`web/src/core-engine.ts`) auf dem Hauptthread hinter denselben Promises. Der Worker ist ein eigener Chunk unter `/assets/` und liegt im Cache des Service Workers (14.5).

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
- Seit 03.11.2026 im ganzen Abend (`--runs N`): je Kneipe Spiele und Einmischen, Relikte beim Betreten, Groschen je gewonnener Kneipe, Spelunken in eigenen Zeilen, seit ADR 0019 die mit dem Schutzengel neu gespielten Kneipen (`again`; bis dahin Leben beim Betreten); je Stadtteil Relikte zu Beginn und vor dem Boss, verlorene Kneipen je Run, neu gespielte Kneipen, wo die verlorenen Runs enden (normale Kneipe, Spelunke, Boss-Kneipe); Groschen nach Herkunft und Ausgaben nach Art (Relikte, Schnäpse, Zinken, Kartenmaler, reinigen, neu auslosen, Neuauslage). Seit ADR 0018 die Zeile `faktor`: mittlerer Faktor der gewonnenen Spiele, Anteil über ×1, höchster Faktor je Run (Mittel) und insgesamt, Anteil der Runs mit einem Faktor-Relikt am Ende und Relikte je Run, Faktor der wachsenden Relikte (Zapfhahn, Heiße Hand) am Ende. `--curve` und `--curve --boss <id>` messen mit `--district 1|2|3` im gewählten Stadtteil (dessen Gegner aus `run.ron`).

### 14.4 Konto und Spielstand auf dem Server (ADR 0021)

Entschieden vom Nutzer im Cockpit (Frage 38, Variante A mit Ergänzung), umgesetzt am 07.11.2026.

- **Freiwillig.** Ohne Konto spielt man wie bisher; der gesicherte Text (`STAMMTISCH1:…`) bleibt der Weg ohne Server.
- **Anmelden** mit einem Passkey (Fingerabdruck, Gesicht, Geräte-PIN), ohne Benutzernamen, E-Mail oder Passwort. Beim Anlegen zeigt die App einmal einen **Wiederherstellungscode** (24 Zeichen, mit Knopf zum Kopieren und dem Hinweis, ihn aufzuschreiben). Damit kommt man ohne Passkey ins Konto (neues Gerät, andere Domain) und legt dann einen neuen Passkey an; der Code wird bei jedem Gebrauch ersetzt.
- **Passkeys verwalten und überall abmelden** (Nachtrag 09.11.2026): Das Konto listet seine Passkeys (angelegt, zuletzt benutzt, synchronisiert) und entfernt einzelne, nie den letzten. „Überall abmelden“ beendet jede Sitzung auf jedem Gerät. Wer sich mit dem Wiederherstellungscode anmeldet oder ihn erneuert, meldet damit alle anderen Geräte ab: Ein gestohlener Code lässt sich so wieder aussperren.
- **Name am Stammtisch**: 1 bis 24 Zeichen, nicht eindeutig, steht auf der Tafel des Tages.
- **Spielstand**: genau der gesicherte Text, für den Server undurchsichtig. Abgleich beim Start (vor dem Lesen des Speichers), nach jeder Kneipe, am Ende des Runs und beim Verlassen der Seite. Hat sich nur eine Seite geändert, gilt sie (mit Hinweis, wenn der Stand des Kontos geladen wird); haben sich beide geändert, wählt der Spieler in einem Dialog, der beide Stände zeigt. Ein älterer Stand überschreibt nie still einen neueren (Versionen, HTTP 409). Kleine Anzeige „gespeichert“, „offline“, „Konflikt“ neben dem Konto.
- **Tafel des Tages**: Am Ende eines Tages-Runs meldet ein angemeldeter Spieler sein Ergebnis; der Startbildschirm zeigt die besten zehn und den eigenen Platz, ohne Konto nur lesend. Das erste Ergebnis je Tag zählt. Selbstauskunft, der Server prüft nichts.
- **Tafel der Woche** (10.11.2026, 5.6): dasselbe für den Wochen-Run (`POST /api/weekly`, `GET /api/weekly/{JJJJWW}`); das erste Ergebnis je Woche zählt, weitere Wochen-Runs sind Übung.
- **Keine Spiellogik auf dem Server**: Er speichert und zählt, er spielt nicht nach. Die Regeln bleiben im Kern im Browser.
- **Datenschutz**: gespeichert werden nur Name, öffentliche Schlüssel der Passkeys, Hash des Wiederherstellungscodes, Spielstand und Tafel-Ergebnisse. Konto löschen (mit Rückfrage) entfernt alles davon sofort.
- **Betrieb**: statisches musl-Binary in einem Alpine-Container neben unserem Caddy, nur über `/api` erreichbar, SQLite in `/opt/skat/data` mit täglicher Sicherung der letzten sieben Tage (`deploy/README.md`). Fällt der Dienst aus, läuft das Spiel weiter, nur Konto und Tafel sind „offline“.

### 14.5 Offline spielen (Service Worker)

Umgesetzt am 08.11.2026. Der Kern läuft als WebAssembly im Browser und der Spielstand liegt in `localStorage`; mit einem Service Worker geht das Spiel deshalb ganz ohne Netz.

- **Was im Cache liegt**: der ganze Build (die Seite, alle gehashten Dateien unter `/assets/` mit dem WebAssembly-Kern, Schriften und Klängen, Icons, Manifest). Die Liste schreibt ein kleines Vite-Plugin (`web/sw/plugin.ts`) bei jedem Build in `dist/sw.js`, dazu eine Version aus dem Inhalt aller Dateien. Neuer Build heißt neuer Worker mit eigenem Cache. Von Hand geschrieben (`web/sw/sw.js`), ohne Workbox.
- **Strategie**: `/assets/*` aus dem Cache (gehashte Namen werden nie alt); die Seite selbst zuerst aus dem Netz mit 3 Sekunden Frist, sonst die Seite aus dem Cache; `/api/*` nie (Konto und Tafel brauchen den Server).
- **Keine alte Seite mit fehlenden Dateien** (der Grund, warum es bis PR #109 keinen Worker gab): Eine Seite aus dem Netz wird nie in den Cache geschrieben. Die Seite im Cache ist immer die, deren Dateien im selben Cache liegen. Ein neuer Worker lädt seinen Build vollständig, bevor er übernimmt; scheitert das, bleibt der alte. Beim Übernehmen löscht er ältere Caches, behält aber die Dateien der Version davor, damit ein noch offener Tab der alten Version weiterläuft, auch wenn der Deploy sie auf dem Server gelöscht hat.
- **Neue Version**: Der neue Worker wartet. Die App zeigt oben eine schmale Leiste „Neue Version verfügbar – neu laden“ (auf dem Handy kurz „Neue Version – neu laden“); die App rückt um ihre Höhe nach unten, die Leiste verdeckt also nichts, und ein × legt sie bis zum nächsten Besuch weg. Sie lädt nie von selbst mitten im Spiel neu. Ein Tipp lässt den Worker übernehmen und lädt die Seite; der laufende Run steht im Speicher und geht weiter. Nach neuen Versionen sieht die App auch, wenn sie wieder in den Vordergrund kommt.
- **Offline**: Das Spiel läuft vollständig. Angemeldet zeigt das Konto „offline“; was sich geändert hat, geht hoch, sobald der Browser wieder online ist oder die App in den Vordergrund kommt. Die Tafel des Tages zeigt einen kurzen Hinweis statt der Liste, die Knöpfe des Kontos (Anmelden, Anlegen, Code) sind mit Hinweis gesperrt.
- **Entwicklung**: `vite dev` hat keinen Worker, nur `vite build` und `vite preview`. Caddy liefert `/sw.js` mit `Cache-Control: no-cache` aus.

### 14.6 Zwei Sprachen (ADR 0023)

Umgesetzt am 10.11.2026. Deutsch ist die Voreinstellung, Englisch die zweite Sprache; das deutsche Blatt bleibt in beiden.

- **Texte**: `web/src/i18n/de.ts` und `web/src/i18n/en.ts` mit denselben Schlüsseln. `en.ts` ist gegen die Form von `de.ts` getypt, `web/scripts/check-i18n.mjs` prüft dazu offene Tabellen, Platzhalter und Listenlängen. Im Web-Code heißt die aktive Sprache weiter `de` (eine lebende Bindung in `web/src/i18n/index.ts`, die `setLanguage` umstellt).
- **Wahl**: beim ersten Start nach der Browsersprache (deutsch nur für deutsche Browser), dann im Menü „Sprache / Language“, gespeichert unter `stammtisch.lang`. Der Wechsel zeichnet alles neu, ohne Neuladen. Englisch liegt in einem eigenen Bundle-Stück und wird nur geladen, wenn es gewählt ist.
- **Inhalte**: Der Kern liefert weiter die deutschen Namen und Texte aus `core/content/*.ron`. `web/src/i18n/localize.ts` ersetzt sie in jeder Sicht nach Content-Id durch die englischen aus `web/src/i18n/en-content.ts` (Relikte, Zinken mit eingesetztem Wert, Schnäpse, Bosse und ihre Spieler, Begegnungen und ihre Möglichkeiten, Freischaltungen, Mali, Verschärfungen, Rückseiten, Stammgäste mit ihren Sprüchen, Stadtteile). Kneipennamen bleiben deutsch.
- **Formate**: Zahlen und Daten über `Intl` in der Sprache (`1,5` und `1.5`, `37,5 %` und `37.5%`), Anführungszeichen „…“ und “…”.
- **Prüfung**: Der Rauchtest läuft deutsch; `SMOKE_ONLY=english` (und am Ende des vollen Laufs) spielt Start, ein ganzes Spiel, Kartenbelohnung, Route, Tresen, Menü mit Sprachwechsel, Regelkarte, Glossar, eine Begegnung und die Einführung auf Englisch, ohne deutsche Oberflächenwörter, abgeschnittene Texte oder seitliches Scrollen.

### 14.7 Klänge und Musik

Klänge seit M6, erweitert am 11.11.2026 (Klänge v2). Alle Klänge sind selbst synthetisiert (`web/scripts/make-sounds.mjs`, Rauschen und Teiltöne aus einem festen Seed, ffmpeg mit libmp3lame), als kleine MP3s im Repo, ohne fremde Aufnahmen (`web/LICENSES.md`). Die Musik sind eigene Chiptune-Schleifen für Tisch, Tresen und Boss (`web/src/songs.ts`, `web/src/music.ts`). Klänge reagieren nur auf Ereignisse und Sichten des Kerns; die Web-App rechnet dafür nichts aus.

**Wann was klingt** (`web/src/audio.ts`, Name der Datei in Klammern):

| Moment | Klang |
|---|---|
| Knopf gedrückt (nicht Karten) | leiser Holz-Tipp (`tap`) |
| Neues Spiel gegeben, Einmischen, Neu geben | Mischen mit Aufstoßen (`shuffle`), dann Karten beim Austeilen (`card`) |
| Spruch eines Stammgasts (Sprechblase) | leises Brummen „hm“, links tiefer als rechts (`quip`, `playQuipSound(seat)`) |
| Skat aufnehmen | zwei Karten vom Filz geschoben (`pickup`) |
| Drücken | zwei gedämpfte Karten auf den Filz (`discard`) |
| Ansage | Fingerknöchel und Kreidestrich (`declare`); Grand, Null und jede Stufe über Normal (Hand, Schneider, Schwarz, Ouvert): Faust auf den Tisch, die Gläser klirren (`bang`) |
| Karte gespielt | Karte schnippen (`card`); ein Unter knallt (`slam`, nach dem aufgedruckten Bild, ohne Trumpfregel) |
| Stich | Stich zusammenschieben (`trick`) |
| Kontra, Re, Fluch eines Bosses | flache Hand auf den Tisch (`kontra`) |
| Ergebnis | Klopfen (`knock`), dann bei Schneider die Schere (`snip`), bei Schwarz der Riss (`schwarz`), dann Sieg oder Niederlage (`win`, `lose`); auch wenn die Abrechnung übersprungen wird |
| Zeile eines Relikts in der Abrechnung | Gläserklirren (`clink`), sonst Kreide-Tick (`tick`) mit steigender Tonhöhe |
| Kaufen am Tresen, Kartenmaler, Reinigen | Münzen und Kassenglocke (`till`); Verkaufen und Auszahlung: Groschen (`coin`) |
| Neue Auslage am Tresen | Mischen (`shuffle`) |
| Relikt, Schnaps oder Hausregel erhalten | Bierdeckel mit kleinem Glanz (`prize`) |
| Schnaps trinken | Einschenken (`pour`), selbst getrunken danach Kippen und Glas absetzen (`gulp`) |
| Würfel (Spelunke) | Würfelbecher und Wurf (`dice`), dann Groschen oder Niederlage |
| Weg auf der Karte gewählt, Wege neu | Kreide (`chalk`) |
| Boss-Kneipe (Bogen des Bosses) | Kneipentür knarrt und fällt zu (`boss`) |
| Freischaltung | Thekenglocke, zweimal steigend (`unlock`) |
| Die lange Nacht beginnt | Kirchturmglocke (`night`) |
| Run gewonnen oder verloren | Sieg oder Niederlage (`win`, `lose`), einmal je Kette |

**Mischung.** Jeder Klang hat eine Grundlautstärke und eine Stufe: Alltag (Karte, Tipp, Tick, Brummen), auffällig (Groschen, Klirren, Ansage, Kontra) und groß (große Ansage, Freischaltung, Boss, Nacht, Sieg, Niederlage). Zwei große Klänge laufen nie gleichzeitig: der spätere wartet, bis der erste weitgehend verklungen ist (höchstens 2,5 s, sonst entfällt er). Unter einem großen Klang werden auffällige auf die Hälfte und alltägliche auf 70 % gesenkt, die Musik auf die Hälfte. Derselbe Klang zweimal binnen 70 ms (alltägliche: 25 ms) erklingt einmal; zwei auffällige binnen 90 ms: der zweite leiser. Wiederkehrende Klänge streuen leicht in Tonhöhe und Lautstärke. Ein sanfter Begrenzer verhindert Übersteuern.

**Einstellungen.** Im Menü unter „Ton“ Klänge und Musik getrennt an und aus, darunter „Lautstärke“ mit je einem Schieber (0 bis 100 % in Fünferschritten, Voreinstellung 80 %, das ist die entworfene Lautstärke; die Kurve ist quadratisch). Gespeichert in `stammtisch.settings` (`soundVolume`, `musicVolume`); ältere Einstellungen ohne die Werte bekommen 80 %. Ton erst nach der ersten Berührung (Browserregel), angehalten, solange die Seite verborgen ist; jeder Fehler bleibt still. Weniger Bewegung schaltet keine Klänge ab.

**Prüfung.** `SMOKE_ONLY=sounds` (und im vollen Lauf): Ein Init-Skript legt `window.__soundLog` an, `audio.ts` trägt dann jeden angefragten Klang mit Ergebnis der Mischung ein. Ein Durchlauf bei 390 × 844 (weniger Bewegung) und 1280 × 800 prüft die Klänge oben, dass zwei große Klänge nie binnen 300 ms starten, und die Schieber auf Deutsch und Englisch.

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
| Server-Zielplattform | Browser. Erreichbar unter https://one-more-skat.de hinter dem Caddy der Haushalt-App, siehe ADR 0007, 0008 und 0024; die alte Adresse stammtisch-62-83-23-90.sslip.io bleibt mit Umzugshinweis (ADR 0025). |

Entschieden am 01.11.2026 (über das Cockpit):

| Frage | Entscheidung |
|---|---|
| Frontend | Web-App mit Rust-Kern als WebAssembly statt Godot, ADR 0009. |
| Kartenbild | Deutsches Blatt, ADR 0010. |
| Zielgerät | Handy hochkant zuerst. |
| Sprache | Nur Deutsch für v1. Texte kommen aus einer Übersetzungsdatei. Seit 10.11.2026 (ADR 0023): Deutsch als Voreinstellung, Englisch als zweite Sprache, Texte aus `de.ts` und `en.ts`. |
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
| Stil der Oberfläche | Stil A „Spielhölle“ in der Farbwelt A4 „Kupfer und Schiefer“: Pixelschriften, Schiefergrau mit Kupfer und Mint, harte Pixelschatten. Seit dem 11.11.2026 ist Elfenbein statt Kupfer die Farbe der Handlung (16.3). Stilproben: https://claude.ai/artifact/6K2j1apvyPeMveR4pRmTid |
| Layout am Tisch | Kein Scrollen. Die Oberfläche füllt genau die sichtbare Höhe, auch als PWA ohne Adressleiste. Hand in zwei Reihen à fünf Karten. Die Wertung ist im Spiel nur eine kleine Zeile. Seit dem 11.11.2026 ist der Tisch ein Ort (16.3). |

Entschieden am 01.11.2026 über das Cockpit (Designfragen nach dem Spieltest):

| Frage | Entscheidung |
|---|---|
| Einstieg | Geführter erster Abend: Der Wirt erklärt in kurzen Sprechblasen, wenn etwas zum ersten Mal passiert. Dazu eine Regelkarte im Menü und ein Glossar der Zinken (seit dem 04.11.2026: nach Rahmensprache gruppiert, Rand, Ecke, Marke, Spezialkarten, Flüche, je mit Beispielkarte und Text aus dem Content, oben die Werte-Icons). Skat selbst wird hier nicht von Grund auf gelehrt; dafür gibt es seit dem 07.11.2026 die Einführung (16.1). |
| Run-Länge | 5 Abschnitte als Standard, dazu ein „kurzer Abend“ mit 3 Abschnitten. Freischaltungen in beiden (M7). Umgesetzt: 5.3, Abschnitt 1 bis 3 mit dem Alten am Ende. |
| Schwierigkeit | Endlose Einsatz-Leiter: Jeder gewonnene Run hebt die Gewinngrenze des nächsten um ein Auge (61, 62, 63 …). Schneider rückt nicht mit. Wirkung wird vorher im Simulator gemessen (M7). Umgesetzt: 5.4, je Charakter und Länge getrennt. |
| Nach dem Endboss | Endlos-Modus: Zielwerte wachsen weiter, nur für den Highscore (M7). Seit ADR 0020 „Die lange Nacht“ (5.5): × 1,6 je Stadtteil, Verschärfungen, Bestenliste. |
| Flüche | Nur angekündigt (z. B. bei Ereignissen) und mit Groschen bezahlt, nie aus Zufallsziehungen (M7). Umgesetzt: 7.2 und 13, Umgekipptes Bier und Der Falschspieler. |
| Normale Gegner | Wechselnde Stammgäste mit Namen und Pixelgesicht, ohne eigene Regeln. Regeln bleiben den Bossen (M6). |
| Relikte nur für Menschen | Behalten, selten und günstig (Mit Vieren, Der Opa, Saubere Weste, Bierdeckel, Glashaus, Ouvert-Bonus). |
| Musik | Leise Chiptune-Schleifen für Tisch, Tresen und Boss, getrennt von den Klängen abschaltbar (M6). |
| Wertung | Im Spiel eine kleine Zeile, nach dem letzten Stich eine große, überspringbare Abrechnung (M6). |
| Null | Für jeden ansagbar, Nullspieler bleibt als Spezialist (ADR 0012, M6). |
| Domain | Eigene Domain one-more-skat.de seit 11.11.2026; die sslip.io-Adresse bleibt ohne Umleitung und hilft beim Umzug (ADR 0025). |
| KI-Tempo | Am Handy flüssig, kein Hintergrund-Thread nötig. |
| Planung | M6 Feinschliff und M7 Langzeit. Der Weckruf läuft bis M7. |

Entschieden am 01.11.2026 im Chat (Problemliste):

| Frage | Entscheidung |
|---|---|
| Grand | Bleibt 24. Gesteuert wird über Bosse und Relikte, nach der großen Testrunde neu bewerten. Seit 04.11.2026 im Run 18 (Cockpit q27, 4.2). |
| Null-Wertung | Eine für alle: die Formel mit Stilbonus (9.2). |
| Endlos-Modus | Zurückgestellt, bis es Relikte mit echtem Wachstum gibt. Die Einsatz-Leiter kommt zuerst. Seit ADR 0020 (05.11.2026) umgesetzt als „Die lange Nacht“ (5.5). |
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

### 16.1 Die Einführung (07.11.2026)

Der geführte erste Abend (Hinweise des Wirts) setzt voraus, dass der Spieler Skat kennt. Für Spieler, die Skat kaum kennen, gibt es seit dem 07.11.2026 die **Einführung**: eine geführte Kneipe mit drei fest gegebenen Spielen. Entschieden von Claude im Rahmen der Autonomie-Regel.

- **Start**: Knopf „Einführung“ auf dem Startbildschirm; einem neuen Spieler (kein Run gespielt, keiner gespeichert) wird sie einmal oben angeboten („Neu am Stammtisch?“, „Nein danke“ blendet das Angebot für immer aus; gemerkt unter `stammtisch.tutorial`).
- **Kern**: `core/content/tutorial.ron` hält Seed, Zielwert (150) und die drei Spiele (Hände, Skat, Vorhand). `Run::new_tutorial` startet einen Run des Alleinspielers mit diesem Seed; in seiner ersten Kneipe gilt der Zielwert der Datei, Einmischen gibt es nicht, die Spiele kommen in der Reihenfolge der Datei, und wie in der Boss-Kneipe werden alle drei gespielt (das Ziel zählt nach dem dritten). Alles andere ist ein normaler Run, deterministisch über den Seed.
- **Lektionen**: (1) Eichel nach dem Skat, mit 3: Ziel, Skat oder Hand, Drücken (beide Schellen, um Schellen frei zu machen), Ansage (Farbe, Grand, Null), Trumpfordnung, Bedienen, Augen und 61, Grundwert × Stufen, Abrechnung mit Schneider und Schwarz. (2) Null nach dem Skat: zwei Daus drücken, keinen Stich machen, Null-Reihenfolge, Null-Wert mit geduckten Siebenen. (3) Grand Hand mit 4: Hand bringt eine Stufe, nur Unter sind Trumpf, „Noch nötig“. Danach Kartenbelohnung (Zinken), Routenwahl, Tresen und Relikte, dann „Echten Abend starten“ oder „Zum Start“.
- **Web-App**: Die Texte und wann sie kommen stehen in `web/src/tutorial.ts` (Darstellung): Jeder Schritt hat eine Situation, einen Text und ein Ziel, auf das der Wirt mit Rahmen und Pfeil zeigt. Infoschritte gehen mit Antippen weiter, Handlungsschritte, sobald der Spieler handelt; ein Schritt, dessen Situation vorbei ist, gilt als erledigt, allgemeine Schritte fangen andere Wege auf (Hand statt Skat, andere Spielart, verlorenes Spiel oder verlorene Kneipe). „Einführung beenden“ steht an jeder Sprechblase. Die normalen Hinweise des Wirts schweigen solange.
- **Kein Run**: Der Einführungs-Run wird nie gespeichert, seine Ereignisse gehen nicht an den Meta-Stand (keine Freischaltungen, keine Statistik, kein Eintrag in der Liste der Runs), und ein laufender gespeicherter Run bleibt unberührt.

### 16.2 Barrierefreiheit (10.11.2026)

Vorbereitung auf die Veröffentlichung im App Store. Entschieden von Claude im Rahmen der Autonomie-Regel; reine Darstellung, der Kern bleibt unberührt.

- **Menü-Zeile „Darstellung“**: ersetzt die Zeile „Kartenfarbe tönen“ und hält drei Schalter, damit das Menü bei 375 × 667 weiter ohne Scrollen passt: „Tönen“ (wie bisher), „Farbenblind“ und „Schrift“. Gespeichert mit den übrigen Einstellungen (`stammtisch.settings`: `colorBlind`, `textSize`), wirksam als Attribute an `<html>` (`data-color-blind`, `data-text-size`), sodass nur CSS umschaltet und nichts neu gezeichnet werden muss.
- **Farbenblind-Hilfe** (aus): Die vier Farben unterscheiden sich sonst vor allem über die Farbe, was bei Rot-Grün-Schwäche wegfällt (Grün, Rot und Schellen werden olivgelb). Mit der Hilfe trägt jede volle Karte unter dem Eckzeichen den Buchstaben ihrer Farbe (E, G, R, S, `de.suitLetters`) hell auf einem dunklen Schild, als Pixelglyphe im Sprite (`cb-*`); die Ecke ist bei allen Bildern frei, nur das große Daus-Zeichen wird leicht überdeckt. Eine offene Wandelkarte hat keinen Buchstaben, eine festgelegte den ihrer neuen Farbe. Blue Curaçao und die Wandelkarte zeigen auf ihrem Schild den Buchstaben statt des Zeichens. Die Seltenheit der Bierdeckel wechselt vom farbigen Rand zu ein bis vier Punkten. Geprüft mit simulierter Deuteranopie und Protanopie (Farbmatrizen nach Machado 2009, im Smoke-Test `a11yPass`); Wege auf dem Stadtplan (Linienart), Stationen (Symbole), Flüche und Zinken (Rahmenform, Band mit Zahl) sowie die Zielleiste (Zahlen) waren schon ohne Farbe lesbar.
- **Schriftgröße** (Normal, Groß, Sehr groß): ein Tipp wechselt zur nächsten Stufe. Die Schriftskala wächst um 1 bis 4 Pixel je Stufe, die kleinen Größen am meisten, die Anzeigezahlen der Abrechnung am wenigsten. Damit der Tisch weiter ohne Scrollen passt, geben die Handkarten beim Ansagen und die Karten auf dem Filz bei „Skat oder Hand“ etwas Höhe ab; das Menü stellt die Beschriftung über die Schalter und scrollt bei Bedarf in sich. Nichts scrollt seitwärts (390 × 844, 360 × 740).
- **Bildschirmleser**: Karten heißen nach den Sichtdaten und `de.ts`, etwa „Rot-Oberin, gezinkt: Aufdruck +3“ (Name der Zinke und die Zahl auf ihrem Band, Flüche mit „(Fluch)“); Stichkarten mit dem Platz („Jupp: …“). Die Kästen der Gegner nennen Position, Kartenzahl und Stiche, der Kopf Groschen, Spiele und Einmischen; die Zielleiste ist ein `meter` mit Text, auch bei negativem Konto. Eine höfliche Live-Region (`role="status"`, `web/src/ui/a11y.ts`) sagt kurz an, was ein Gegner spielt, wer den Stich bekommt und das Ergebnis des Spiels; beim Vorspulen schweigt sie. Jeder Dialog ist nach seiner Überschrift benannt.
- **Tastatur und Fokus**: Öffnet ein Dialog, geht der Fokus hinein (der Name wird vorgelesen); Tab bleibt im obersten Dialog; Escape schließt, wo die App es erlaubt (nicht die Abrechnung und den Boss, die ihren Knopf haben); danach kehrt der Fokus zum Auslöser zurück. Wird derselbe Dialog neu gezeichnet (ein Schalter im Menü), bleibt der Fokus auf demselben Bedienelement. Fokusringe sind 3 px Mint. Scrollende Textflächen (Regelkarte, Glossar, Vereinsbuch) sind per Tab erreichbar.
- **Kontrast**: Kupfer als Schriftfarbe auf den Schieferflächen hatte 4,1 : 1; Text in Kupfer nimmt jetzt das helle Kupfer (`--accent-light`), dazu dunklere Beschriftungen auf dem Stadtplan und hellere „Verkauft“-Felder am Tresen (WCAG 4,5 : 1). Flächen und Knöpfe in Kupfer bleiben. (Stand 10.11.2026; seit dem 11.11.2026 gelten die Farbrollen aus 16.3, Kupfer ist nur noch Dekoration.)
- **Weniger Bewegung**: `prefers-reduced-motion` kürzt jede Animation auf 1 ms und eine Wiederholung (`style.css`), dazu schalten Rauch, Lampe, Blinzeln, Denkblase und die übrigen Endlos-Animationen ab; das Tempo der Wiedergabe sinkt auf ein Viertel. Ein eigener Schalter in der App fehlt noch (eigene Zeile im Menü nötig).

### 16.3 Tisch, Kopf und Farbrollen (11.11.2026)

Nach der Durchsicht der Oberfläche durch den Nutzer: Stil, Palette „Schiefer“, Pixelschrift, Karten und Porträts bleiben; behoben werden die vier größten Schwächen am Tisch. Reine Darstellung (`web/src/ui/table.ts`, `web/src/ui/hud.ts`, `web/src/style.css`), der Kern bleibt unberührt. Tresen und Stadtplan folgen in späteren Schritten, der Startbildschirm in 16.4.

- **Der Tisch ist ein Ort**: eine Tischplatte aus grünem Filz mit Holzkante unter dem Lichtkegel der Lampe, als Pixelgrafik selbst gezeichnet (`web/scripts/gen-table-art.mjs` schreibt `web/src/assets/table/*.svg`: gestufter, geditherter Lichtkegel, Filzflor, Holzkante als Randbild; kein Verlauf, keine Unschärfe). Die beiden Gegner sitzen an der hinteren Kante, links und rechts gespiegelt: Porträt in doppelter Größe seiner 30 Pixel, Name (im Boss-Lokal das Namensschild beider Figuren), Position, Stiche und ein Fächer aus so vielen Rückseiten, wie sie Karten halten. Sprüche und die Denkblase hängen am Platz. Der Stich liegt groß in der Mitte (Karten größer als die Handkarten), als Dreieck zum Spieler hin. Darüber die Zeile des Spiels (Ansage, Stiche, Grundwert × Stufen als flache Anzeigen), in den freien unteren Ecken links „Letzter Stich“, rechts Boss-Regel und Gedrücktes, darunter der Zähler als kleine Tafel. Skat oder Hand, Drücken und die Ansage liegen auf demselben Filz.
- **Kopf in zwei Zeilen, höchstens 110 px über dem Filz** (390 × 844: 110 px, vorher rund 235 px mit den Kästen der Gegner). Eine flache Leiste mit Ort (ein Tipp öffnet die Kreidetafel), Konto und Ziel über ihrem Balken, den übrigen Spielen als Kästchen, dem Einmischen als Zahl an zwei Karten und den Groschen; daneben der Menüknopf. Darunter die Leiste (48 px, damit Relikte und Schnäpse 44-px-Tippziele bleiben): Malus und Nacht-Chips, der Boss des Lokals (am Tisch nur sein Bierdeckel, die Figuren sitzen ja da), die gehaltenen Relikte, in sich scrollend, daneben immer sichtbar der Zähler „1/8“ statt leerer Plätze, dann die Schnaps-Plätze (leere schmal). Die Kästen der Gegner im Kopf entfallen. Mit großer Schrift steht der Ort zweizeilig, der Name des Lokals bleibt auf der Kreidetafel.
- **Farbrollen** (Token in `style.css`, in der ganzen App): **Elfenbein** (`--role-action`, `--ivory` #f1e6c8, dunkle Schrift darauf) für die nächste Handlung, den Hauptknopf und die aktive Wahl; Mint (`--role-gain`) für Gewinn und alles Gute (Kontobalken, Stufen, Faktor und Relikt-Beiträge der Abrechnung samt ihren Icons, wartende Faktoren, Freischaltungen); Messing (`--role-money`, `--brass`) für Groschen und Preise, auch die Münze selbst und die Preisschilder am Tresen; Rot (`--role-danger`) für Gefahr, Flüche, Boss-Regeln, Warnungen und Verlust; Schiefer und `--text` für reine Anzeigen und Hervorhebungen im Text. Text auf dem Filz nimmt `--text` oder `--felt-text` (mindestens 4,9 : 1), nie `--muted`. Kupfer war bis zum 11.11.2026 die Farbe der Handlung (der Nutzer wählte nach der Durchsicht Elfenbein); seit dem 11.11.2026 abends ist auch der Schriftzug „Stammtisch“ mit seiner Leiste (Startbildschirm, Run-Karte) Elfenbein; Kupfer kommt nur noch in der Pixelgrafik vor.
- **Elfenbein auf hellen Flächen**: Auf Papier, Karten und cremefarbenen Sprechblasen hätte Elfenbein keinen Kontrast. Dort gilt: ein elfenbeinfarbener Knopf behält seine dicke dunkle Linie; ein Ring um eine Karte liegt zwischen zwei dunklen Linien (spielbar 2 px, unter Finger oder Zeiger und gewählt 4 px und höher); Schrift ist dort dunkle Tinte, nie Elfenbein. Auf dem Stadtplan, dessen Wege-Kacheln selbst papierhell sind, ist „Los“ die Umkehrung: dunkle Tinte mit Elfenbein-Linie und Elfenbein-Schrift; die Spur des Wegs ist dunkle Tinte. Bei den Spielarten der Ansage treten die nicht gewählten Kacheln einen Ton zurück.
- **Meldungen über dem Kopf**: Kurzmeldungen („Eingemischt und neu gegeben …“), die Hinweise des Wirts und das Band beim Ziehen eines Schnapses liegen im Run als Streifen über der Kopfleiste, nie über den Plätzen, dem Skat, dem Stich oder der Hand (390 × 844, 360 × 740, Desktop). Die längsten Hinweise reichen bei 360 px Breite 3 px auf die Holzkante; mit „Sehr groß“ reichen lange Hinweise bis an die Oberkante der Porträts (ein Tipp schließt sie).
- **Begrüßung nur als Sprechblase**: Bei Skat oder Hand steht die Begrüßung eines Stammgasts nur noch in der Blase an seinem Platz (eine zur Zeit, `web/src/app/talk.ts`), nicht mehr zusätzlich als Zeilen unter der Frage. Die Begrüßung des anderen steht auf seiner Infokarte (langes Drücken auf den Platz).
- **Knopf oder Anzeige**: Was man drücken kann, ist erhaben (helle Ober-, dunkle Unterkante in der dicken Linie, harter Schatten, `--raised`); Anzeigen sind flach (kein Schatten, höchstens eine dünne Linie).
- **Wer dran ist**: „Du bist dran“ steht als elfenbeinfarbenes Band zwischen Filz und Hand, direkt über den Karten. Spielbare Karten stehen ein Stück aus dem Fächer und tragen einen Elfenbein-Ring, nicht spielbare sind abgedunkelt und entsättigt, aber lesbar (nicht durchscheinend). Die Karte unter Finger oder Zeiger und eine gewählte stehen höher und haben den breiteren Ring. Mit „weniger Bewegung“ hebt sich keine Karte, Rahmen und Abdunklung zeigen den Zustand. Der Gegner am Zug hat einen hellen Porträtrahmen.
- **Desktop**: Ab 900 px Breite nimmt der Tisch 760 px statt der Handy-Spalte, der Lichtkegel ist eine Stufe größer gezeichnet. Ein eigenes Desktop-Layout ist ein späterer Schritt.

### 16.4 Startseite (11.11.2026)

Schritt 2 der Überarbeitung (`docs/ui-ueberarbeitung.md`): Die Startseite war eine Textwand mit dem Hauptknopf mitten im Bildschirm, einem prominenten Seed-Feld und gesperrten Figuren mitten im Karussell. Reine Darstellung (`web/src/ui/screens.ts`, `web/src/style.css`), der Kern bleibt unberührt.

- **Die gewählte Figur groß**: Über dem Karussell steht das Porträt der gewählten Figur in dreifacher Größe seiner 30 Pixel (ab 700 px Breite vierfach), darunter Name, Rolle und die ganze Beschreibung, auf zwei Zeilen gefaltet. „mehr“ (44-px-Tippziel, `aria-expanded`) klappt sie auf, „weniger“ wieder zu; passt der Text in zwei Zeilen, entfällt der Knopf. Der aufgeklappte Zustand bleibt beim Wechsel der Figur.
- **Gesperrte Figuren am Ende**: Im Karussell stehen erst alle freien, dann alle gesperrten Figuren, je in der Reihenfolge des Kerns (nur die Anzeige ordnet um). Gesperrte zeigen weiter Schloss und Bedingung.
- **Hauptknopf am unteren Rand**: „Neuer Run als …“ liegt in einer Leiste, die beim Scrollen am unteren Rand stehen bleibt (`position: sticky` am Ende der Seite, über dem Safe-Area-Rand der App). Am Ende der Seite sitzt sie unter der Fußzeile und verdeckt nichts. Mit einem laufenden Run bleibt „Fortsetzen“ elfenbeinfarben oben und der Knopf in der Leiste ist ein normaler Knopf.
- **Zweite Ebene „Weitere Spielarten“**: Tages-Run (mit Tafel), eigener Seed und die Einführung liegen hinter einer erhabenen Zeile mit Pfeil (`aria-expanded`, `aria-controls`), die sie auf- und zuklappt; eine Linie links bindet den aufgeklappten Teil an die Zeile. Zugeklappt bleibt das Seed-Feld in der Seite (versteckt), ein getippter Seed gilt also weiter. Das Angebot „Neu am Stammtisch?“ bleibt für neue Spieler oben.
- **Reihenfolge**: Schild, Meldungen, geteilter Seed, „Neu am Stammtisch?“, Fortsetzen, Figur und Karussell, Einsatz-Leiter, Herausforderung der Woche (immer sichtbar), Weitere Spielarten, Statistik, Bestenliste, Vereinsbuch, Kurzregel, Fußzeile (Konto, Einstellungen, Abspann), Hauptknopf. Keine Funktion fällt weg.
- Geprüft bei 390 × 844, 360 × 740 und 1280 × 800, mit „Groß“, „Sehr groß“ und Englisch (Smoke `startScreenPass`).

### 16.5 Tresen (11.11.2026)

Schritt 3 der Überarbeitung (`docs/ui-ueberarbeitung.md`): Am Tresen lag ein großer Block mit der nächsten Kneipe vor den Waren, die Waren rutschten unter den Falz, Texte waren abgeschnitten, die Kaufen-Knöpfe klein und dunkel. Reine Darstellung (`web/src/ui/shop.ts`, `web/src/style.css`), der Kern bleibt unberührt; was kaufbar ist, sagt weiter der Kern (`can_buy`).

- **Waren zuerst**: Reihenfolge Wirt, Schulden (falls angeschrieben), Theke mit Hausregeln und Figuren, Schnäpsen und Zinken, Tasche. Bei 390 × 844 und 360 × 740 sind ohne Scrollen die erste Warengruppe ganz und „Weiter“ zu sehen, auch mit „Groß“, „Sehr groß“ und Englisch.
- **Die nächste Kneipe als Streifen über „Weiter“**: In der unteren Leiste steht eine Zeile „Als Nächstes · Boss-Kneipe“, darunter der Boss mit kleinem Bierdeckel und das Ziel („Die Zwillinge · Ziel 115“; ohne Boss „Kneipe 3 · Ziel 60“). Ein Tipp klappt darüber die Kreidetafel mit Stadtteil, Spielen, Einmischen, Figuren, Regel und Boss-Decks auf (`aria-expanded`, `aria-controls`), ein zweiter klappt sie zu; aufgeklappt ist sie höchstens 46 % der Höhe und scrollt in sich. Der Streifen gehört zu „Weiter“, weil er sagt, wohin es geht, und bleibt so ohne Scrollen sichtbar. Eine Boss-Kneipe trägt einen roten Rand links, das Wort „Boss“, den Bierdeckel und den Namen in der Gefahr-Rolle (nicht nur Farbe).
- **Kaufen**: Jeder Kaufen-Knopf ist so breit wie seine Ware und mindestens 44 px hoch, erhaben, mit Messing-Linie und Messing-Schrift, weil er Groschen ausgibt. Elfenbein bleibt allein bei „Weiter“: Auf einer Seite mit bis zu acht Waren konkurriert so kein Knopf mit dem Hauptknopf, und die Theke wirkt ruhig. Im Kaufblatt einer Infokarte ist „Kaufen · 2 Gr.“ die Hauptsache und bleibt Elfenbein.
- **Nicht kaufbar** (der Kern sagt nein): Knopf flach mit gestrichelter Kante, grauer Schrift und einem kleinen Vorhängeschloss, das Preisschild gestrichelt als Umriss; lesbar, nicht durchscheinend. Ebenso flach und gestrichelt sind andere gesperrte Knöpfe am Tresen (Neu auslosen, Kartenmaler, Verkaufen).
- **Kein abgeschnittener Text ohne Weg**: Die Kurzfassung auf Bierdeckel und Ware bleibt auf drei bis vier Zeilen gekürzt. Wo sie abgeschnitten ist (gemessen nach dem Zeichnen), steht darunter „mehr“; ein Tipp auf die Ware öffnet die vorhandene Infokarte mit dem ganzen Text und dem Kaufen-Knopf. Die Auslage der Zinken bleibt ungekürzt, ihr Tipp öffnet weiter das Blatt mit der Karte vorher und nachher.
- **Farbrollen**: Preisschilder Messing (wie bisher), alter Preis bei Schnäppchen und Hehlerware durchgestrichen in dunkler Tinte; Fluch-Hinweise der Zinken, Hehlerware und Schnäppchen in der Gefahr-Rolle, „bleibt“ in der Gewinn-Rolle; Schulden als Zeile mit rotem Rand in der Gefahr-Rolle; „Verkauft“ als flache gestrichelte Fläche in der Anzeige-Rolle, niedriger als vorher.
- Geprüft bei 390 × 844, 360 × 740 und 1280 × 800, mit „Groß“, „Sehr groß“ und Englisch, vor dem Boss, mit und ohne Groschen (Smoke `tresenPass`).

### 16.6 Papier-Ebene und Abrechnung (12.11.2026)

Schritt 4 der Überarbeitung (`docs/ui-ueberarbeitung.md`): Der Stadtplan hatte als einziger Bildschirm eine helle Creme-Fläche und wirkte wie aus einem anderen Spiel; in der Abrechnung stand der Spielwert dreimal (Kacheln Grundwert × Stufen, „Spielwert 54“, „+54“). Reine Darstellung (`web/src/ui/score.ts`, `web/src/ui/infocard.ts`, `web/src/style.css`), der Kern bleibt unberührt.

- **Papier als zweite Fläche**: Die Creme des Stadtplans ist jetzt die Fläche für alles, was geschrieben oder gesagt wird: die Abrechnung als Zettel vom Kellnerblock, die Infokarten, die Sprechblasen (Wirt am Tresen, Hinweise und Einführung des Wirts, Sprüche der Stammgäste) und wie bisher die Karten und Beschriftungen des Stadtplans. Schiefer bleibt die Fläche für Tisch, Kopf, Tresen und Menüs.
- **Token** (`style.css`): `--paper` #f1e6cc, `--paper-light`, `--paper-dark`, `--paper-tile`, `--paper-map`; Tinte `--paper-ink` #2a2320 und `--paper-ink-soft`; `--paper-line` für Linien und Kanten (Schmuck, keine Schrift), `--paper-off` für Abgeschaltetes. Die fest notierten Creme-Werte des Stadtplans sind auf diese Token zurückgeführt; das SVG des Plans nennt seine Werte gleich (`citymap.ts`, `route.ts`).
- **Farbrollen auf Papier**: Auf Papier gilt dunkle Tinte. Die Rollen haben dort dunkle Stufen, alle mindestens 4,5 : 1 auf `--paper`, `--paper-light`, `--paper-dark` und `--paper-map` (niedrigster Wert 5,2 : 1): Gewinn `--paper-gain` #2a6346 (statt Mint), Groschen `--paper-money` #6f5320 (statt Messing), Gefahr `--paper-danger` #9e2b22 (statt Hellrot); dazu die beiden Tinten des Plans, Weinrot `--paper-wine` und Petrol `--paper-teal`. Die Klasse `.paper` setzt innerhalb der Fläche die Schiefer-Token (`--text`, `--muted`, `--role-*`, `--panel` …) auf diese Stufen, sodass Inhalte ohne eigene Regeln richtig erscheinen. Flächen, die selbst dunkle Tinte tragen (ein leuchtender Bierdeckel in Mint), behalten die helle Farbe.
- **Elfenbein auf Papier** (Regel aus 16.3): Ein elfenbeinfarbener Knopf auf Papier ist umgekehrt wie „Los“ auf dem Plan: dunkle Fläche, Elfenbein-Linie, Elfenbein-Schrift, darum eine dunkle Linie (Weiter auf dem Zettel, Kaufen auf der Infokarte). Andere Knöpfe auf Papier sind papierhell mit dicker dunkler Linie, erhaben. Ein geöffneter Glossar-Begriff auf der Infokarte ist ebenso umgekehrt, ein geschlossener dunkle Tinte mit gepunkteter Linie.
- **Der Zettel**: oben die Abrisskante des Blocks als Reihe von Löchern, darunter Ergebnis (Gewonnen in Gewinn-, Verloren in Gefahr-Tinte), Spielart, Augen und Stiche (im Ramsch die drei Plätze), die Bierdeckel der Relikte, dann die Rechnung: je Schritt eine linierte Zeile (Grundwert, Spitzen, Stufen, Relikte und Zinken mit Icon und Beitrag). Unter einem Doppelstrich steht die Summe als ein Block: Grundwert × Stufen (× Faktor, nur wenn das Spiel einen hat) und darunter „Spielwert = …“ als Stempel in Gewinn- oder Gefahr-Tinte. **Das ist die einzige Stelle des Spielwerts.** Darunter die Wirkung aufs Konto: „Konto alt ▸ neu“ mit Ziel und Balken (ein Verlust zeigt sich am neuen, kleineren Stand; der Hinweis zu einfach gezählten Verlusten steht klein darunter), dann bei geschaffter Kneipe die Auszahlung in zwei Zeilen (zwei Paare je Zeile), dann „Weiter“.
- **Animation** wie bisher: Die Zeilen erscheinen nacheinander, die Bierdeckel und Kopf-Chips der beteiligten Relikte leuchten auf, Grundwert, Stufen und Faktor zählen in der Summe mit; dann öffnet sich die Spielwert-Zeile, die Zahl zählt hoch und landet als Stempel (der Zettel ruckt), dann läuft das Konto hoch, der Balken füllt sich, die Groschen der Auszahlung fliegen. Ein Tipp überspringt, mit weniger Bewegung steht alles sofort.
- **Ohne Scrollen**: Der Zettel scrollt nie. Bei 390 × 844, 360 × 740 und 1280 × 800, mit „Groß“, „Sehr groß“ und Englisch sind Summe, Konto, Auszahlung und „Weiter“ immer zu sehen, auch beim längsten Fall (Faktor-Spiel mit drei Relikten, neun Zeilen und Auszahlung). Reicht der Platz nicht für alle Zeilen der Rechnung, scrollt nur die Liste der Zeilen in sich (während des Ablaufs folgt sie der neuesten Zeile); ein Schild „weitere Zeilen“ oben oder unten sagt, dass dort Zeilen verborgen sind, und die Liste ist dann per Tab erreichbar. Unter 780 px Höhe werden Überschrift, Summe und Stempel eine Stufe kleiner.
- **Bildschirmleser**: Die Summe wird als ein Satz vorgelesen („Grundwert 9 mal Stufen 15 mal Faktor 2,535 ergibt Spielwert 342“), die Liste der Zeilen heißt „Die Rechnung“.
- Geprüft im Smoke-Test: `paperPass` (Token-Kontraste, längster Zettel in allen Größen, Infokarte, Blasen und Plan auf dem Papier-Token, umgekehrter Elfenbein-Knopf) und `assertScoreSlip` bei jedem durchgespielten Spiel (Papier-Token, Spielwert genau einmal als Zahl, kein Scrollen, „Weiter“ sichtbar, jeder Text auf dem Zettel mindestens 4,5 : 1).

## 17. Offen

- **Kartenbelohnung und Grand 18** (04.11.2026, 5.0, 4.2, 8.6): beide ungemessen. Die nächste Balancing-Runde misst sie zusammen: Wirkung auf „Stadtteil 2 und 3 erreicht“, Anteil Grand und sein Mittelwert, und je Run angeboten, genommen, gesteigert, übersprungen und je Zinke (Simulator-Zeilen `card rewards` und `card reward zinken`). Kurzprüfung mit 20 Runs (Seed 42): im Mittel 3,45 Belohnungen je Run, alle genommen.
- **Playtest-Rückmeldungen vom 05.11.2026** (Entscheidungen des Nutzers, alles vorläufig, ungemessen): Kartenbelohnung als feste Paare aus Karte und Zinke (5.0, 8.6), Auslage wieder drei Angebote, ein „Neu auslosen“ je Tresen-Besuch zu 2 Groschen mit dem Duzfreund (5 Groschen, Ungewöhnlich) und keine Obergrenze der Spezialkarten (7.2). Die nächste Balancing-Runde misst: (a) Kartenbelohnung genommen, übersprungen und gesteigert (feste Karte statt freier Wahl: der Bot überspringt, wenn nur Ersetzen für weniger bleibt) und ihre Wirkung auf Stadtteil 2 und 3; (b) die dritte Zinke der Auslage gegen Runde 3 (dort mit 2 Angeboten 11,3 % statt 14,3 % Stadtteil 3); (c) Neu auslosen: wie oft, Groschen je Run, und ob der Duzfreund Preis und Seltenheit wert ist; (d) Spezialkarten je Deck am Ende eines Runs und die Wirkung mehrerer Untot oder Wandelkarten (die Marken zahlen keine Groschen mehr).
- **Runde 2** (04.11.2026, `docs/balance/2026-11-04-runde2.md`): Zielbild „Mischung“ (5.0.4), ein Einmischen mehr in Stadtteil 1 (`district_redeals`), neue Kurven für alle fünf Charaktere (der Drückeberger hat jetzt eigene), Spelunke × 1,0, der Alte × 1,3. Alleinspieler 34,7 / 11,9 / 3,5 %, Schneiderin 38,3 / 14,7 / 4,8 %, Drückeberger 38,2 / 15,0 / 5,7 %, Nullspieler 37,5 / 14,7 / 6,5 %, Bruderherz 34,2 / 13,5 / 6,5 % (Nullspieler und Bruderherz danach in Stadtteil 3 schwerer, ungemessen). **Nicht erreicht**: (a) höchstens 20 % in den ersten beiden Kneipen (Alleinspieler 27,9 %, andere 22 bis 29 %): Das dritte Einmischen bringt dem Bot nur 1 bis 3 Punkte je Kneipe, und eine Kneipe ohne Relikte liegt auch bei Zielwert 1 nur bei 92 bis 93 %. Hebel außerhalb der Zahlen: ein klügerer Einmisch-Bot (`core-dev`), ein Start-Schutzengel für Stadtteil 1, oder zwei Einmischen mehr (gemessen: Kurve +2,3 Punkte statt +1,2). Frage fürs Cockpit. (b) 30 bis 40 % der verlorenen Runs beim Boss (nachher 25,5 %): dazu müssten normale Kneipen bei rund 88 % liegen, das ist nahe der Obergrenze. Runde 3: Boss in Stadtteil 2 schwerer (85 → 95), Ramscher prüfen.
- **Runde 1 ohne Leben** (04.11.2026, `docs/balance/2026-11-04-runde1.md`): (a) und (d) unten gemessen; Kurve des Alleinspielers, Spelunke × 0,9 und die Kurven von Nullspieler, Schneiderin und Bruderherz neu gesetzt (5.0.4). Alleinspieler danach 33,4 / 12,2 / 5,1 %. **Offen für Runde 2** (3 × 500 / 3 × 300): Stadtteil 1 etwas leichter (Vorschlag 1-4 und Boss 60 → 55), Stadtteil 3 wieder schwerer (Vorschlag 145 / 165 / 185 / 200 / 145; gehalten 42 % statt rund 27 %, der Alte 77 %); die Kurven der anderen Charaktere messen; Spelunke 0,9 oder 1,0; Schutzengel und Faktor-Relikte einzeln ohne Tresen. Ein Drittel der Runs endet in den ersten beiden Kneipen, und die erste Kneipe kommt über den Zielwert kaum über 90 %: Ob dort ein anderer Hebel nötig ist (ein Einmischen mehr in Stadtteil 1, Start-Currywurst), ist eine Design-Frage. An `core-dev`: Die Bot-Regel für die Spelunke (ab vier Relikten) heißt bei 8 Relikten zu Beginn von Stadtteil 2 praktisch „immer“; der Simulator meldet nicht, wie oft die Schutzengel-Wiederholung gewonnen wird, und nicht die Rast-Wahl.
- **Keine Leben und Faktor (ADR 0019 und 0018, 04.11.2026)**: bis Runde 1 nichts gemessen. Die nächste Runde im Simulator muss messen: (a) die Zielkurve ohne Leben (Stadtteil 2 und 3 erreicht, Run gewonnen, wo die Runs enden; die Kurve aus 5.0.4 war für zwei Startleben gesetzt, ohne sie endet jede verlorene Kneipe den Run, also deutlich härter), daraus ein neues Zielbild; (b) den Schutzengel als zweite Chance (Wirkung auf „Stadtteil 2/3 erreicht“, Preis 8 vorläufig, wie oft er greift: Spalte `again`); (c) die neue Rast (Currywurst oder Karte waschen) und das Portemonnaie mit Schnaps; (d) die acht Faktor-Relikte einzeln und im vollen Run (Zeile `faktor` im Simulator: mittlerer und höchster Faktor, Anteil der Runs mit einem Faktor-Relikt, Faktor von Zapfhahn und Heißer Hand am Ende), ihre Preise, Seltenheit und Bot-Werte; (e) die Bot-Regel für die Spelunke (vier Relikte oder drei und der Schutzengel). Das Faktor-Zeichen in der Oberfläche ist vorläufig, der Nutzer wählt.

- **Verloren zählt einfach** (04.11.2026, 4.3): gemessen in Runde 1 zusammen mit „keine Leben“. Die Erfolgskurve einer Kneipe ist steiler (0,45 statt 0,3 Punkte je Zielwert-Punkt), die Boss-Kneipen leichter (56 bis 74 % in Stadtteil 1 statt 51 bis 58 %); zusammen mit dem Wegfall der Leben war die Kurve vom 03.11. trotzdem viel zu schwer (5.0.4).

- **Bosse v2 (ADR 0017, 03.11.2026)**: Faktoren des Alleinspielers gemessen am 03.11.2026 (11.1), Boss-Kneipen jetzt 51 bis 58 %, der Alte 36 %; die des Nullspielers und der übrigen Charaktere noch nach Augenmaß. Offen: die Zwillinge fallen von 65 % in Stadtteil 1 auf 44 % danach (PIMC-Gegner, die sich gegenseitig sehen), der Ramscher von 57 auf 40 %; ob der Alte als Endboss mit 36 % zu schwer ist, zeigen echte Runs. (a) ~~**Der Ramscher**: ob ein verlorener Ramsch in seiner Kneipe nur einfach zählen soll.~~ Entschieden von Claude am 03.11.2026: ja, einfach (`SingleLoss`, 11.1), Faktor bleibt 1,0; im ganzen Abend neu messen. Offen zum Entscheiden: (b) **Die Oma unter dem Alten**: Ihre Flüche bleiben ab ihrem Blatt bis zum Ende der Kneipe (Auslegung von Claude), nicht nur für ein Blatt. (c) Eigene Rückseiten je Figur (heute teilen beide die des Bosses) und weitere Bosse als Daten, zuerst der Totengräber (ruht). (d) Der Bot wählt das Boss-Relikt nach seiner Werttabelle (`relic_value`), nicht nach der Seltenheit.

- **Neue Charaktere** (03.11.2026, 9.4 bis 9.6): Drückeberger, Schneiderin und Bruderherz spielen mit den Zielwerten des Alleinspielers und seinen Tresen-Gewichten, nichts gemessen (Simulationen ruhen). Zu messen, sobald sie wieder laufen (`--character shirker|tailoress|heart_brother`): ob die zurückkommenden Marken des Drückebergers zu viele Groschen bringen; ob die Schneiderin mit −1 ohne Schneider überhaupt durch Stadtteil 1 kommt; wie stark Bruderherz im Rot-Spiel ist; Erreichbarkeit der drei Freischaltungen (`--unlocks`, der Bot drückt selten 21 Augen). Für feat/zinken: Auslage und Kartenmaler müssen `Run::zinken_price` nutzen (9.6).

- **Gegenstands-Review** (02.11.2026, 8.5): Preise von Skatgott, Ein Auge zudrücken, Schieben, Schenken, Schutzengel und Feierabend am 03.11.2026 nach Messung gesetzt (12). **Offen: Der Schutzengel** (seit ADR 0019 die zweite Chance, neu zu messen) war mit zwei Startleben vom wirkungslosen zum zweitstärksten Relikt geworden (ohne Tresen +14,8 auf „Stadtteil 2“, +9,0 auf „Stadtteil 3“, `docs/balance/2026-11-03-grosse-runde.md`); deshalb seit dem Nachtrag zur großen Runde wieder 8 (ungemessen). Feierabend wirkt beim Bot weiter nicht. Die übrigen Werte nach Augenmaß. Zu messen: Wirkung von Schutzengel, Kasse, Deckel und Feierabend auf „Stadtteil 2/3 erreicht“, Drei drücken nur im ersten Spiel, Spickzettel mit beiden Karten, Doppelkorn mit Neugeben; Erreichbarkeit der Freischaltungen mit jetzt 16 von 32 gesperrten Relikten; ob der Skatlehrer („im ersten Spiel ein Einmischen weniger“) und die Wette (nur Schneider zählt) so gemeint sind. Offen beim Nutzer: der endgültige Name von „Rang +1“ und die neue Tresen-Mechanik für Kartenverbesserungen; die Form des Bierdeckels nach Anzeige-Gruppe.

- **Kneipentour (ADR 0016)**: Zielwerte, Leben (2/4/+1) und Bonus je übrigem Spiel (2) für den Alleinspieler gemessen am 03.11.2026 (5.0.4): Stadtteil 2 42,3 %, Stadtteil 3 17,4 %, gewonnen 5,2 %, 36 % der verlorenen Runs in normalen Kneipen, alles am oberen Rand des Zielbilds. Nächster kleiner Schritt, falls gewünscht: Boss-Zielwert in Stadtteil 1 60 → 70 (weniger Runs in Stadtteil 2, mehr Ende beim Boss). Offen: **alle anderen Charaktere** mit der neuen Kurve und zwei Startleben (der Nullspieler behält seine Zielwerte 18 bis 30 und dürfte jetzt viel zu leicht sein; Drückeberger, Schneiderin, Bruderherz spielen die steile Kurve ungemessen); die **Spelunke** (Zielwert × 1,25 auf der steilen Kurve: in Stadtteil 3 nur 20 bis 36 % gewonnen; seit dem Nachtrag `target_factor` 1,1, ungemessen); die Bot-Regel für die Spelunke (seit ADR 0019 vier Relikte oder drei und der Schutzengel, ungemessen); Gewichte der Stationen, Gegnerstärke je Stadtteil. Auslegungen von Claude: die Spelunke kommt nie vor einer Boss-Kneipe; unter drei Wegen ist immer ein Tresen; Rast bis ADR 0019 nur mit fehlendem Leben; der Malus „Kein Skat“ trifft den Nullspieler nicht. Seit ADR 0017 erledigt: Sekundant des Bosses (zwei Boss-Spieler) und die Wiederholung einer verlorenen Boss-Kneipe (sie beendet jetzt den Run). Offen im Cockpit: die Form des Malus (umgesetzt: einer, zufällig, vorher sichtbar). Die Stadtkarte (Pixel-Stadtplan) wird getrennt entworfen; die Sicht liefert dafür den ganzen Plan, Knoten-Ids und Seeds (5.0.3).

- **Effekte v2** (02.11.2026, 7.2): Werte, Preise und Seltenheit sind Platzhalter, nichts gemessen (Simulationen pausiert). Entschieden: Moment-Modifikationen und Spezial-Gutscheine höchstens einmal, Groschen-Gutschein, Augen und Trumpf ohne Obergrenze, Trumpf und Fleckig streng (Cockpit). Mit M9 erledigt: Frisches Deck reinigt (7.3), die Sicht zeigt die Ausgabe des Gegners auf einer durchsichtigen Karte (7.2).
- **Eigene Ausgaben (ADR 0015)**: beantwortet im Cockpit und umgesetzt in M9 (02.11.2026): Trumpf und Fleckig streng, Boss-Ausgaben mild, Stammgäste normal, Frisches Deck reinigt, Nullspieler mit Abgestuft, laufende Spielstände verworfen. Offen: alle Messungen (18.4, Simulationen pausiert); ob die Gleichstands-Regel bei Abgestuft auf Daus und Zehn (gleiche Stärke, die frühere gewinnt) für Menschen verständlich ist; ob die Web-App vor der Boss-Serie zusätzlich die Reihenfolge der Ausgaben des Alten zeigen soll (heute nur die Ausgabe der Regel im laufenden Blatt).

- **Zielwerte für 3 Spiele**: erledigt am 02.11.2026 (5.2, je Charakter eigene Kurve, Boss-Faktoren 11, Einsatz-Leiter 5.4, Freischaltungen 9.3). Offen daraus: (a) **Nullspieler gewinnt nur 0,6 % der ganzen Abende** (Ziel 1 bis 2 %); seine Kurve liegt ab Abschnitt 3 fast am Boden, Zielwerte bewegen dort kaum noch etwas. Entschieden am 02.11.2026: ein Einmischen mehr (`rule: (redeals: 1)` in `characters.ron`, 0,69 → 1,12 % bei gleichen Seeds), siehe 9.2. (b) **Der Schwarze Peter** bleibt mit 35 bis 48 % der schwerste Boss; mit 3 Spielen liegt seine Obergrenze bei 50 % (Nullspieler 37 %), kein Faktor hilft. Entschieden am 02.11.2026 (Cockpit): gewonnene Spiele ohne Schneider zählen halb statt null (11); die Faktoren neu messen, wenn die Simulationen wieder laufen. (c) Preise von Schieben, Ein Bier, Kräuter, Neues Blatt und Reue (8.1, 8.3) und ob Einmischen beim Alten (Regel je Blatt) zu leicht macht, sind noch nicht gemessen.

- **Null-Formel**: entschieden in M4 (9.2). Offen bleibt das Feintuning der Stilboni mit Testspielen.
- **Zielwert-Faktoren pro Charakter**: erledigt mit ADR 0012 (`character_factors` in `bosses.ron`, 11). Schieber und Schwarzer Peter sind wieder bei beiden Charakteren. Seit ADR 0017 trifft die Oma mit ihren Flüchen jeden Charakter (Nullspieler mit eigenem Faktor 0,9); messen mit `--curve --boss grandma --character null`, wenn die Simulationen wieder laufen.
- **Null für alle mit Menschen prüfen** (ADR 0012): Die Gewinnquoten hinter der Bot-Wahl Farbe, Grand oder Null (`SUIT_WIN_PERCENT`, `GRAND_WIN_PERCENT`, `NULL_WIN_PERCENT` in `core/src/run/bot/`) sind Bot-Messungen ohne Relikte. Ob Menschen mit dem Alleinspieler öfter Null spielen und ob die Tresen-Gewichte (`focus_weights`) den Nullspieler klar genug vom Alleinspieler trennen, zeigt die Statistik echter Runs.
- **M5-Kurve mit Menschen prüfen**: Die Kurve (5.2) ist gegen den PIMC-Spieler gesetzt. Ein Mensch nutzt Hand, Ouvert und Informations-Relikte, die der Bot kaum spielt, und dürfte stärker sein. Testspiele und die Statistik der Web-App (Abschnitt erreicht) gegen die Bot-Zahlen halten.
- **Relikte ohne Bot-Wirkung**: Spitze, Mit Vieren, Schiedsrichter, Bierdeckel, Ouvert-Bonus, Oma, Opa, Glashaus, Blindflug, Saubere Weste und Zähler bewegen beim Bot „Abschnitt 3 erreicht“ um weniger als 5 % (relativ). Sie sind für Menschen gedacht; ob sie die 5-%-Regel erfüllen, zeigt erst die Statistik echter Runs.
- **Ereignis-Wahrscheinlichkeit, Figurenpreise**: Platzhalter, `balance`.
- **Welle 2a im vollen Run** (02.11.2026, 8.1): Preise und Seltenheit der acht neuen Hausregeln sind aus der Einzelmessung ohne Tresen gesetzt (`docs/balance/2026-11-02-relikte-welle2.md`), Platzhalter für `balance`. Offen: Wirkung im vollen Run mit Tresen (3 × 3000), beim Nullspieler, und ob der Bot den Skatpfleger beim Drücken nutzen soll (er drückt weiter Karten mit wenig Augen, ein Mensch drückt Zehnen). Stammfarbe, Wechselbalg und Allrounder hängen an der Spielwahl; der Bot wählt das Spiel ohne Blick auf die Relikte, ein Mensch kann gezielt wechseln oder bei einer Farbe bleiben. Zu entscheiden: ob der Wechselbalg nur bei gewonnenen Spielen zählt (jetzt wie jede Stufe auch verloren, wie der Text sagt; beim Bot kostet er so Punkte je Spiel).
- ~~**Drei drücken zähmen?**~~ Entschieden am 02.11.2026 (Lead): einmal pro Serie, beim ersten Aufnehmen des Skats (8.1). Gemessen +7,9 Prozentpunkte auf „Abschnitt 3 erreicht“ statt +18,0 in jedem Spiel; nicht weiter abgestimmt. Offen bleibt nur die Wirkung im vollen Run mit Tresen und beim Nullspieler.
- **Nullspieler über den kurzen Abend** (M7, 5.3): „Run gewinnen“ zählt auch im kurzen Abend, der Nullspieler ist damit nach rund 25 statt rund 1700 Runs (Heuristik-Bot) frei. Entschieden ist „Freischaltungen in beiden“; soll der Nullspieler ein Ziel des ganzen Abends bleiben, bekäme die Bedingung `WinRun` eine Länge (Daten in `unlocks.ron`). Ebenso „Abschnitt 4 erreichen“: nur im ganzen Abend möglich.
- **Die lange Nacht** (ADR 0020, 5.5, 05.11.2026): Seit Runde 5 (acht Relikt-Plätze, `docs/balance/2026-11-05-runde5.md`) neues Wachstum: Alleinspieler endet in Stadtteil 4 / 5 bis 7 / 8 bis 11 / ab 12 in 43 / 47 / 10 / 0 % der Nächte, Nullspieler 25 / 57 / 18 / 0 %. Offen: (a) mehr als die Hälfte der Nächte endet beim Boss, ein Viertel beim Ramscher; ein eigener Boss-Faktor der Nacht oder ein milderer Ramscher in der Nacht senkte den Anteil, der in Stadtteil 4 endet; (b) Nachtschwärmer kommen mit acht Plätzen kaum ins Spiel (der Bot tauscht nur bei + 3 Wert), Preise und Bot-Werte ungemessen; (c) der Bot hortet in der Nacht Groschen (ab Stadtteil 6 im Mittel 77 und mehr), weil er nur noch tauscht; (d) Drückeberger, Schneiderin und Bruderherz in der Nacht ungemessen; (e) Stadtteil 10 (Freischaltung Morgengrauen, Laterne) erreichen 2,5 % der Nächte des Alleinspielers, keine des Nullspielers.
- **Relikte mit acht Plätzen** (Runde 5): Spickzettel, Treue Seele, Bierdeckel und Spelunkenkenner werden in der Hälfte der Fälle wieder verkauft (dominiert), Build-Relikte außerhalb ihres Builds in rund 40 %; der Schutzengel nie. Kein Relikt ist klar Pflicht; Einzelmessungen mit acht Plätzen (Schutzengel, Legendäre) offen.
- ~~**Weitere Kartenrückseiten** (9.3)~~: seit ADR 0020 gibt es die Belohnungsart `Reward::CardBack` und drei Rückseiten der Nacht (5.5); weitere Erfolge für Rückseiten bleiben offen. Bis dahin: Standard ist die Distel-Rückseite (nach Altenburger Vorbild, eigene Pixelgrafik). Weitere Rückseiten werden später über bestimmte Erfolge freischaltbar (Freischaltung als Kosmetik, ohne Spielwirkung). Die Bedingung gehört wie jede Freischaltung in den Kern (`unlocks.ron`, eine neue Belohnungsart für Kosmetik); die Web-App bietet nur die verdienten Rückseiten zur Wahl an.
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

# M8 Welle 2a: acht neue Hausregeln, einzeln gemessen

- **Datum**: 02.11.2026
- **Branch**: `feat/relics-wave2` (auf `main` 1dace9c: 3 Spiele je Serie, Einmischen, Ansagen nach dem Skat)
- **Agent**: `core-dev`
- **Maßstab**: wie in M5 (`docs/balance/2026-11-01-m5-v1.md`): PIMC-Spieler (`--bot pimc:12:3`), Relikt ab Spielbeginn, ohne Tresen (`--no-shop`), Alleinspieler, Gegner je Abschnitt aus `run.ron`. Zielwerte von `main` (die Neukalibrierung läuft auf einem anderen Branch; verglichen wird nur mit und ohne Relikt auf demselben Stand).
- **Seeds**: 42, 43, 44, je 500 Runs, also 1500 Runs je Zeile. Gleicher Seed heißt gleiche Blätter, solange das Relikt nichts ändert; die Unterschiede sind deshalb weniger verrauscht als zwei unabhängige Stichproben. Standardfehler für „Abschnitt 3 erreicht“ bei 12 % etwa 0,8 Punkte je Zeile; die drei Seeds ohne Relikt liegen bei 11,4 / 11,8 / 12,0 %.

## Befehle

```sh
# je Seed 42, 43, 44
cargo run --release -p skat_sim -- --seed 42 --runs 500 --no-shop --bot pimc:12:3
cargo run --release -p skat_sim -- --seed 42 --runs 500 --no-shop --bot pimc:12:3 --relic <id>
# Auslösungen je Spiel (Heuristik-Kartenspiel, nur Abschnitt 1, 400 Runs je Relikt)
cargo test --release -p skat_core --lib wave_two_trigger_statistics -- --ignored --nocapture
```

## Ergebnis

Abschnitt erreicht in Prozent (Veränderung gegen „ohne Relikt“ in Prozentpunkten), Spiele gewonnen in Prozent, Punkte je Spiel im Mittel aller gespielten Spiele (verlorene doppelt negativ).

| Relikt | Abschnitt 2 | Abschnitt 3 | Abschnitt 4 | Δ Abschnitt 3 | Spiele gewonnen | Punkte je Spiel | Δ Punkte |
|---|---|---|---|---|---|---|---|
| ohne Relikt | 40,5 | 11,7 | 3,4 | – | 64,5 | 16,3 | – |
| Allrounder | 47,1 (+6,6) | 15,5 | 4,8 | +3,7 | 63,8 | 17,7 | +1,4 |
| Altes Blatt | 45,6 (+5,1) | 15,1 | 4,7 | +3,4 | 63,9 | 17,2 | +1,0 |
| Stichfest | 43,9 (+3,3) | 14,7 | 4,7 | +3,0 | 64,2 | 17,9 | +1,6 |
| Skatpfleger | 43,1 (+2,6) | 14,2 | 3,6 | +2,5 | 66,6 | 18,4 | +2,1 |
| Bubenparade | 43,5 (+2,9) | 13,5 | 3,8 | +1,8 | 64,1 | 17,7 | +1,4 |
| Luschenkönig | 42,5 (+1,9) | 13,1 | 3,5 | +1,3 | 64,2 | 17,0 | +0,7 |
| Wechselbalg | 42,9 (+2,4) | 12,2 | 3,6 | +0,5 | 64,1 | 14,8 | −1,4 |
| Stammfarbe | 40,5 (±0,0) | 11,8 | 3,4 | +0,1 | 64,5 | 16,2 | ±0,0 |
| Alte Dame (Vergleich) | 46,2 (+5,7) | 17,0 | 5,6 | +5,3 | 68,1 | 20,9 | +4,6 |
| Glückssträhne (Vergleich) | 42,4 (+1,9) | 14,3 | 4,8 | +2,5 | 64,3 | 28,7 | +12,4 |
| Auf dem Trockenen (Vergleich) | 41,8 (+1,3) | 13,2 | 3,7 | +1,5 | 64,3 | 17,1 | +0,8 |

„Abschnitt 3 erreicht“ je Seed (42 / 43 / 44): ohne 11,4 / 11,8 / 12,0; Allrounder 14,0 / 16,0 / 16,4; Altes Blatt 14,4 / 15,8 / 15,2; Stichfest 14,8 / 15,8 / 13,6; Skatpfleger 13,6 / 15,4 / 13,6; Bubenparade 12,4 / 14,2 / 14,0; Luschenkönig 13,6 / 12,8 / 12,8; Wechselbalg 12,6 / 12,2 / 11,8; Stammfarbe 11,2 / 12,2 / 12,0.

### Wie oft sie auslösen

Heuristik-Kartenspiel (schneller, gleiche Ansage-Regeln wie der PIMC-Spieler), nur Abschnitt 1, 400 Runs je Relikt, etwa 1700 Spiele:

| Relikt | Spiele mit Zeile | Wirkung je Spiel im Mittel | wenn ausgelöst |
|---|---|---|---|
| Allrounder | 98,0 % | +3,8 Grundwert | +3,9 |
| Altes Blatt | 93,4 % | +3,0 Grundwert | +3,2 |
| Bubenparade | 76,1 % | +1,4 Grundwert | +1,9 |
| Skatpfleger | 62,4 % | +5,5 Augen | +8,8 |
| Wechselbalg | 47,4 % | +0,5 Stufen | +1 |
| Luschenkönig | 13,3 % | +0,13 Stufen | +1,0 |
| Stichfest | 10,0 % | Schneider durch den Zuschlag | – |
| Stammfarbe | 0,6 % | ×1,5 | – |

## Einordnung

Mit 3 Spielen je Serie erreicht der Bot Abschnitt 3 seltener als in M5 (11,7 statt 14,5 %), deshalb fallen alle Wirkungen in Prozentpunkten kleiner aus. Die drei Vergleichs-Relikte zeigen den Maßstab: Die Alte Dame (Preis 9) bringt jetzt +5,3 statt +7,1, die Glückssträhne (7) +2,5 statt +4,0, Auf dem Trockenen (5) +1,5 statt +1,2. Die M5-Staffel der Preise (ab +6,5 Punkten 9, ab +4,5 8, ab +3 7, ab +1,5 6, darunter 5) entspricht heute grob dem Dreiviertel: ab +4,9 9, ab +3,4 8, ab +2,3 7, ab +1,1 6, darunter 5. Grenzfälle eine Stufe tiefer, wie in M5.

- **Allrounder** ist die stärkste der acht: Schon ein Farbspiel bringt +3 Grundwert, und der Bot spielt in einem Drittel der Spiele Null, also oft zwei Spielarten in einer Serie.
- **Altes Blatt** wirkt fast in jedem Spiel (der Bot nimmt fast immer den Skat auf) und wächst in der Serie: +2, +4, +6.
- **Stichfest** hilft dem Bot über den Schneider: In jedem zehnten Spiel macht der Zuschlag aus einem gewonnenen Spiel ein Schneiderspiel (eine Stufe mehr).
- **Skatpfleger** bringt dem Bot die meisten Punkte je Spiel (+2,1) und mehr gewonnene Spiele (66,6 statt 64,5 %), obwohl der Bot Karten mit wenig Augen drückt (Ø 8,8 Augen im Skat, wenn welche drin sind). Ein Mensch drückt eine blanke Zehn und holt mehr heraus.
- **Bubenparade**: im Mittel 1,9 Unter je Spiel, in dem der Spieler welche hat.
- **Luschenkönig**: Eine Sieben des Alleinspielers gewinnt in jedem siebten bis achten Spiel einen Stich. Ähnlich stark wie Auf dem Trockenen.
- **Wechselbalg** bringt dem Bot weniger Punkte je Spiel (−1,4), aber mehr Runs in Abschnitt 2 (+2,4). Der Bot wechselt meist in ein Null oder aus einem heraus; Null gewinnt er nur in 55 % der Spiele, und eine Stufe mehr kostet verloren doppelt. Die Streuung hilft trotzdem über den Zielwert einer Serie. Für einen Menschen, der bewusst zwischen Farbe und Grand wechselt, ist er besser.
- **Stammfarbe** wirkt beim Bot praktisch nie: Drei Farbspiele hintereinander mit derselben Trumpffarbe kommen in 0,6 % der Spiele vor, weil der Bot seine Trumpffarbe nach dem Blatt wählt. Ein Relikt für Menschen, die eine Farbe sammeln (Kreide, Gezinkt, Bube ehrenhalber).

## Gesetzt (Platzhalter für `balance`)

| Relikt | Seltenheit | Preis | Kaufwert Bot | Begründung |
|---|---|---|---|---|
| Allrounder | Uncommon | 8 | 7 | +3,7, die stärkste der Welle; Uncommon, weil die Seltenheit der Stärke folgt (12) |
| Altes Blatt | Uncommon | 7 | 7 | +3,4, Grenzfall zwischen 7 und 8 |
| Stichfest | Uncommon | 7 | 6 | +3,0 |
| Skatpfleger | Uncommon | 7 | 6 | +2,5; für Menschen stärker |
| Bubenparade | Common | 6 | 4 | +1,8 |
| Luschenkönig | Common | 5 | 4 | +1,3, Grenzfall zwischen 5 und 6, wie Auf dem Trockenen |
| Wechselbalg | Common | 5 | 2 | +0,5, beim Bot weniger Punkte je Spiel |
| Stammfarbe | Rare | 6 | 1 | für Menschen (Multiplikator ×1,5 fürs Spätspiel), beim Bot ohne Wirkung |

Die Kaufwerte (`RELIC_VALUES` in `core/src/run/bot.rs`) folgen der Tabelle dort, mit den Wirkungen auf den M5-Maßstab hochgerechnet (etwa × 1,3). So kauft der Bot Allrounder und Altes Blatt wie Durchmarsch oder Schieben, die Stammfarbe fast nie.

## Offen

- Wirkung im vollen Run mit Tresen und beim Nullspieler (`--character null`) nicht gemessen; die neuen Relikte verdünnen den Pool (43 statt 35 Relikte).
- Ob Wechselbalg nur bei gewonnenen Spielen zählen soll: Dann wäre er für jeden Spieler ein reiner Gewinn. Jetzt zählt er wie jede Stufe (Hand, Ouvert-Bonus) auch verloren, wie der Text sagt.
- Der Bot nutzt den Skatpfleger beim Drücken nicht (er drückt weiter nach Fehlfarben und wenig Augen). Wenn der Bot Menschen abbilden soll, wäre „Zehnen drücken mit Skatpfleger“ eine Regel für `RunBot::discard_for`.

# M8 Welle 2b: Drei drücken, Spickzettel, Lockvogel

- **Datum**: 02.11.2026
- **Branch**: `feat/relics-wave2b` (auf `main` 4255f6e, Welle 2a gemergt)
- **Agent**: `core-dev`
- **Maßstab**: wie Welle 2a: PIMC-Spieler (`--bot pimc:12:3`), Relikt ab Spielbeginn, ohne Tresen, Alleinspieler, Seeds 42, 43, 44 mit je 500 Runs. Regeln wie in DESIGN 8.1 (Welle 2b). Die Zeile „ohne Relikt“ ist auf diesem Branch neu gemessen: 11,9 statt 11,7 % „Abschnitt 3 erreicht“, weil der größere Relikt-Pool (46 statt 43) die Ziehungen der Ereignisse ändert (Stammtisch-Streit, Alter Bekannter). Verglichen wird nur auf demselben Stand.

## Befehle

```sh
# je Seed 42, 43, 44
cargo run --release -p skat_sim -- --seed 42 --runs 500 --no-shop --bot pimc:12:3
cargo run --release -p skat_sim -- --seed 42 --runs 500 --no-shop --bot pimc:12:3 --relic three_discard
cargo run --release -p skat_sim -- --seed 42 --runs 500 --no-shop --bot pimc:12:3 --relic decoy
cargo run --release -p skat_sim -- --seed 42 --runs 500 --no-shop --bot pimc:12:3 --relic crib_sheet
# Nutzung je Spiel (Heuristik-Kartenspiel, nur Abschnitt 1, 400 Runs je Relikt)
cargo test --release -p skat_core --lib wave_two_b_usage -- --ignored --nocapture
```

## Ergebnis

Abschnitt erreicht in Prozent (Veränderung gegen „ohne Relikt“ in Prozentpunkten), Spiele gewonnen in Prozent, Punkte je Spiel im Mittel aller gespielten Spiele.

| Relikt | Abschnitt 2 | Abschnitt 3 | Abschnitt 4 | Δ Abschnitt 3 | Spiele gewonnen | Punkte je Spiel | Δ Punkte |
|---|---|---|---|---|---|---|---|
| ohne Relikt | 40,6 | 11,9 | 3,5 | – | 64,5 | 16,3 | – |
| Drei drücken | 63,3 (+22,7) | 29,9 | 13,7 | +18,0 | 73,6 | 31,4 | +15,1 |
| Lockvogel | 42,8 (+2,2) | 13,0 | 3,7 | +1,1 | 65,2 | 17,0 | +0,7 |
| Spickzettel | 40,8 (+0,2) | 12,3 | 3,5 | +0,3 | 64,6 | 16,5 | +0,1 |
| Kiebitz (Vergleich, Information) | 41,3 (+0,7) | 13,1 | 3,1 | +1,2 | 64,5 | 16,1 | −0,3 |
| Drei drücken, Variante „kleinste Karte des Gegners“ (verworfen) | 67,0 (+26,4) | 38,3 | 19,9 | +26,4 | 73,2 | 41,5 | +25,2 |

„Abschnitt 3 erreicht“ je Seed (42 / 43 / 44): ohne 12,0 / 12,0 / 11,8; Drei drücken 29,0 / 30,0 / 30,8; Lockvogel 11,8 / 14,0 / 13,2; Spickzettel 12,4 / 11,8 / 12,6; Kiebitz 10,8 / 15,6 / 13,0; Variante kleinste Karte 36,8 / 39,6 / 38,6.

Die Variante „kleinste Karte“ ist ein Gegenversuch, nicht im Kern: Statt einer zufälligen Karte gibt der linke Gegner seine Karte mit den wenigsten Augen (gemessen mit einem Probe-Build vor dem letzten Commit, ohne den Hinweis auf die zurückgegebene Karte im PIMC-Spieler; das ändert an der Größenordnung nichts).

### Wie oft sie wirken

Heuristik-Kartenspiel, nur Abschnitt 1, 400 Runs je Relikt, etwa 1700 Spiele:

| Relikt | Spiele, in denen es wirkt | Anmerkung |
|---|---|---|
| Drei drücken | 94,1 % | jedes Spiel mit aufgenommenem Skat |
| Lockvogel | 98,7 % | der Bot fordert fast immer; der nächste Gegner spielt in 45 % der Forderungen die genannte Farbe aus, sonst hat er keine oder das Spiel ist vorher vorbei |
| Spickzettel | 6,0 % | nur Handspiele, die der Bot selten spielt |

## Einordnung

- **Drei drücken** ist mit Abstand das stärkste Relikt des Spiels: +18,0 Punkte auf „Abschnitt 3 erreicht“, dreimal so viel wie die Alte Dame (+5,3 in Welle 2a), und doppelt so viele Punkte je Spiel. Die Wirkung kommt nicht aus dem dritten Skatplatz, sondern daraus, dass eine Karte die Seite wechselt: Der Alleinspieler wählt seine zehn aus 13 Karten, der linke Gegner verliert eine zufällige Karte (in jedem achten Spiel einen Unter) und bekommt die schwächste des Alleinspielers. Der Gegenversuch zeigt, dass jede Variante, die die dritte Karte beim Gegner holt, so stark ist: Nimmt man seine kleinste Karte, wird es noch stärker (+26,4), weil der Alleinspieler dann im Null eine Sieben bekommt und dem Gegner sein bestes Mittel zum Einfangen fehlt. Ein dritter Skatplatz ohne Gegnerkarte ginge nur mit einer 33. Karte (Blanko im Grundblatt), siehe DESIGN 8.1.
- **Lockvogel** hilft dem Bot wenig (+1,1, im Rauschen der drei Seeds), etwa wie der Kiebitz. Er fordert fast in jedem Spiel, meist eine Farbe, deren Daus er hält. Ein Mensch, der mitzählt, setzt ihn gezielter: wenn er weiß, wer welche Farbe noch hat, und um einen Stich mit vielen Augen in seine Volle zu lenken.
- **Spickzettel** wirkt beim Bot praktisch nicht (+0,3), weil er nur in 6 % der Spiele Hand spielt und eine bekannte Skatkarte das Kartenspiel kaum ändert (der PIMC-Spieler legt sie in seinen Stichproben in den Skat). Ein Relikt für Menschen, die Hand spielen; Partner von Bierdeckel, Oma, Opa und Blindflug.

## Gesetzt (Platzhalter für `balance`)

| Relikt | Seltenheit | Preis | Kaufwert Bot | Begründung |
|---|---|---|---|---|
| Drei drücken | Rare | 12 | 10 | „selten und teuer“ (Abstimmung); +18,0 liegt weit über der Staffel (ab +4,9: 9), der Preis deshalb über allen anderen Relikten. Dazu gesperrt hinter der Freischaltung „Stammkunde“ (zehn Runs zu Ende spielen) |
| Lockvogel | Uncommon | 6 | 3 | +1,1, Grenzfall zwischen 5 und 6; für Menschen stärker (Zählen), deshalb 6 |
| Spickzettel | Common | 4 | 1 | Information für Handspiele, beim Bot ohne Wirkung; billiger als der Kiebitz (5), der in jedem Spiel wirkt |

## Offen

- **Drei drücken entscheiden** (Nutzer oder `balance`): Bleibt es bei +18, ist es ein Relikt, um das sich ein Run dreht. Wege, es zu zähmen, ohne den Text zu ändern: (1) Preis weiter hoch (der Bot hat im Mittel 23 bis 26 Groschen pro Run); (2) einmal pro Serie statt in jedem Spiel; (3) die dritte Karte als 33. Karte aus dem Talon, im Grundblatt eine Blanko, die zufällig gegeben wird (der Skat hätte dann wirklich drei Plätze, braucht aber einen Umbau aller Skat-Stellen im Kern und in der KI). Gemessen ist nur die umgesetzte Regel und die Variante „kleinste Karte“.
- Wirkung im vollen Run mit Tresen und beim Nullspieler nicht gemessen (Drei drücken dürfte dort wegen der Sieben im Null noch stärker sein).
- Die Gegner-KI nutzt nicht, was ein Gegner über den Tausch weiß (welche Karte er abgab und zurückbekam); der PIMC-Spieler kennt die zurückgegebene Karte. Eine noch wartende Lockvogel-Forderung kennt die PIMC-Suche nur für den Stich, in dem sie gilt.

# 0020: Die lange Nacht: Endlos-Modus nach dem Alten

Datum: 2026-11-05 · Status: Angenommen (Zahlen vorläufig bis zur Messung) · Ergänzt [0016](0016-kneipentour.md), [0017](0017-bosse-v2.md), [0018](0018-faktor.md) und [0019](0019-keine-leben.md)

## Kontext

Ein Run endet mit dem Alten in Stadtteil 3. Der Faktor (ADR 0018) wurde eingeführt, damit ein gut gebauter Run exponentiell wachsen kann; ein Ort, an dem das zählt, fehlte (Roadmap M7, Endlos-Modus zurückgestellt). Wer den Alten schlägt, hat danach nichts mehr zu tun, und die stärksten Builds werden nie ausgereizt.

Der Nutzer hat am 05.11.2026 im Cockpit (Fragen q30 bis q33) jeweils Variante A gewählt.

## Entscheidung

1. **Einstieg (q30 A).** Wer den Alten schlägt, hat den Run **gewonnen**, und er wird genau wie bisher gezählt (`RunWon`, Statistik, Einsatz-Leiter, Freischaltungen). Danach bietet der Endbildschirm „Weiterziehen in die lange Nacht“ neben „Run beenden“ an (`RunAction::EnterNight`, nur in `RunPhase::Won`). Weiter geht es mit Stadtteil 4, 5, … mit demselben Deck, denselben Relikten, Groschen und demselben Charakter. Die Nacht endet, sobald eine Kneipe verloren geht; der Schutzengel hilft auch hier einmal, wenn er noch da ist. **Verlieren in der Nacht kostet nichts außer dem Rekord**: Der Sieg bleibt gezählt (`RunLost { night: true }` zählt keinen Run).
2. **Skalierung (q31 A).** Stadtteil n der Nacht (4, 5, …) spielt die Zielwerte von Stadtteil 3 (Kneipen und Boss, mit dem Faktor des gezogenen Bosses) × 1,6^(n − 3), einmal gerundet (`night: (growth: 1.6)` in `run.ron`, `NightConfig::grow` rechnet ganzzahlig). Groschen und Belohnungen wie in Stadtteil 3; die Gegner spielen wie in Stadtteil 3 (der letzte Eintrag von `opponents`).
3. **Verschärfungen (q32 A).** „Die Sperrstunde rückt näher“: Jeder Stadtteil der Nacht bringt **eine** dauerhafte Verschärfung, die sich mit den vorigen stapelt. Sie wird mit einem eigenen Generator der Nacht gezogen, keine doppelt, bevor alle dran waren, und vor dem Stadtteil angekündigt (Ereignis `VerschaerfungDrawn` beim Öffnen der Routenwahl in den Stadtteil, Startblatt in der Oberfläche, Chips in der Kopfzeile). Acht Verschärfungen als Daten in `night.ron` (`NightRule`): ein Einmischen weniger (nie unter 0), stärkere Gegner (vier PIMC-Blätter mehr), doppelte Bosse (die Boss-Kneipe der Nacht legt dazu die Regel eines zweiten Bosses auf, nie den Alten), Tresen teurer (+1 Groschen auf alles mit Preis), eine verhexte Karte je Kneipe (ein zufälliger Fluch nur für diese Kneipe, wie bei der Oma), ein Spelunken-Malus in jeder normalen Kneipe, Zielwert-Sprung (+10 % obendrauf), eine Zinke weniger in der Kartenbelohnung. **Bosse der Nacht**: die bestehenden Bosse samt dem Alten, zufällig, keiner doppelt, bevor alle dran waren; neue Bosse kommen später als Daten.
4. **Belohnungen (q33 A).**
   - **Rekord je Charakter** im Meta-Zustand: tiefster Stadtteil der Nacht, meiste Kneipen einer Nacht und bestes Spiel (höchster Spielwert, auch aus normalen Runs). Die Startseite zeigt sie als „Bestenliste“.
   - **Freischaltungen** an den Meilensteinen Stadtteil 5, 7 und 10 (`ReachSection(n)` in `unlocks.ron`): je eine **Kartenrückseite** (neue Belohnungsart `Reward::CardBack`, Kosmetik ohne Spielwirkung: Mondsichel, Sternenhimmel, Laterne), dazu ab Stadtteil 7 und 10 je ein Relikt für den normalen Run (Kehraus, Morgengrauen). Verdiente Rückseiten wählt der Spieler im Menü.
   - **Relikte der Nacht „Nachtschwärmer“**: sechs starke Relikte (`night: true` in `house_rules.ron`), nur in der Nacht angeboten, nach jedem Boss der Nacht als Relikt-Wahl (solange eines übrig ist) und am Tresen der Nacht; fast alle Faktor, nur für gewonnene Spiele (ADR 0018): Nachteule (+×0,15 je Verschärfung), Kehraus (×2 im dritten Spiel einer Kneipe), Lumpensammler (+×0,05 je Karte mit Zinke), Mondschein (+×0,2 je gewonnener Kneipe), Nachtschicht (+×0,1 je Karte mit einer Zinke deiner Richtung), Morgengrauen (×1,5 im ersten Spiel einer Kneipe).
5. **Alle Zahlen sind vorläufig**, bis eine Balancing-Runde sie misst.

## Alternativen

- **Der Sieg zählt erst am Ende der Nacht**: Die Nacht wäre ein Risiko für den Sieg; verworfen (q30), der Sieg über den Alten soll ein Sieg bleiben.
- **Lineare Zielwerte in der Nacht** oder ein Zuschlag je Kneipe: Gegen multiplikative Faktoren würde die Nacht nie enden; ein exponentielles Wachstum je Stadtteil ist das Gegenstück zu ADR 0018.
- **Verschärfungen zur Wahl**: mehr Entscheidungen, aber jeder würde die harmloseste wählen; gezogen und angekündigt ist die Nacht unberechenbarer und trotzdem planbar.
- **Neue Bosse für die Nacht**: später als Daten; vorerst reichen die bestehenden, ohne Wiederholung.
- **Relikte statt Rückseiten an den Meilensteinen**: Rückseiten gab es als Belohnung noch nicht; die neue Art ist klein (eine Variante, drei Bilder aus der bestehenden Pipeline), und Relikte kommen dazu.

## Folgen

- **Kern:** `Run::night` (`NightState`, alte Spielstände laden ohne Nacht), `RunAction::EnterNight`, Ereignisse `NightStarted`, `VerschaerfungDrawn`, `ExtraBossRules`, `NightCursesLaid`, `RunLost::night`; `Run::pub_target` statt `series_target` für die Zielwerte der Stadtteile ab 4; `Series::night_malus`; Inhalte `night.ron` (`Sources::night`, `Content::verschaerfungen`, `Content::card_backs`), `RunConfig::night`, `RelicDef::night`, Effekt `ScaleGameValuePerCard`, `CardCount::direction`, `Reward::CardBack`, Rekord in `CharacterStats` (`best_game`, `night_best_section`, `night_pubs_won`). Der normale Run zieht keine Zufallszahl anders als vorher.
- **Bot:** `RunBot::next_action` hält am gewonnenen Run wie bisher; `RunBot::night_action` zieht immer weiter (nur für Messungen). Bot-Werte der Nacht-Relikte vorläufig.
- **Simulator:** `--night` lässt jeden gewonnenen Run in die Nacht weiterziehen und berichtet erreichte Stadtteile, Kneipen, Faktor am Ende, die Verschärfung, unter der die Nacht endete, und die gewählten Nacht-Relikte.
- **Oberfläche:** Knopf auf dem Endbildschirm, Startblatt je Stadtteil der Nacht, Chips der Verschärfungen in der Kopfzeile (lange drücken erklärt), dunklere Nacht-Palette, Bestenliste auf der Startseite, Rückseiten-Wahl im Menü, Bierdeckel der Nacht-Relikte.
- **Balance:** Die erste Plausibilitätsmessung (DESIGN 5.5) zeigt, dass mit linear wachsenden Faktor-Relikten (Zapfhahn, Mondschein, Nachteule, Lumpensammler) die Hälfte der Nächte bis Stadtteil 20 und weiter läuft; die Balancing-Runde muss Wachstum und Relikt-Werte einstellen.

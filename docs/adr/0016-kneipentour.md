# 0016: Die Kneipentour statt Abschnitten und Serien

Datum: 2026-11-02 · Status: Angenommen · Ersetzt die Run-Struktur aus Abschnitten und Serien (DESIGN 5, 5.1, 5.3 bis 02.11.2026)

## Kontext

Bis hierher war ein Run eine Folge von fünf Abschnitten (Skatabenden) mit je zwei Serien und einer Boss-Serie, dazwischen der Tresen und gelegentlich ein Ereignis (20 %). Eine verfehlte Serie beendete den Run sofort. Daneben gab es einen kurzen Abend mit drei Abschnitten.

Im Spieltest hat sich das als zu starr und zu hart gezeigt:

- **Ein verlorenes Spiel zur falschen Zeit beendet alles.** Mit drei Spielen je Serie hängt der Run an jeder einzelnen Serie; das fühlt sich nach Pech an, nicht nach Entscheidung (gemessen: eine Serie mit drei Spielen hat eine flache Erfolgskurve, DESIGN 5.2).
- **Zwischen den Serien gibt es nichts zu entscheiden.** Der Tresen kommt immer, ein Ereignis zufällig. Der Spieler kann nicht steuern, ob er einkauft, seine Karten verbessert oder ein Risiko sucht.
- **Die Orte haben kein Gesicht.** „Abschnitt 3, Serie 2“ ist eine Tabellenzelle, keine Kneipe.

Der Nutzer hat am 02.11.2026 eine Grundänderung entschieden: die Kneipentour.

## Entscheidung

Ein Abend ist eine **Kneipentour durch eine erfundene Stadt**: drei **Stadtteile** mit je vier **Kneipen** und einer **Boss-Kneipe**, 15 Kneipen insgesamt.

1. **Kneipe**: ein Zielwert (Augen wie bisher die Serie), bis zu drei Spiele gegen zwei Stammgäste; wer den Zielwert früher erreicht, ist sofort fertig. Belohnung: fester Grundbetrag plus ein Bonus je übrigem Spiel (die bisherige Auszahlung, mit höherem Bonus je Spiel).
2. **Leben**: Start mit 3, höchstens 5. Eine verfehlte Kneipe kostet ein Leben, es geht ohne Belohnung zur nächsten. Eine verfehlte Boss-Kneipe kostet ein Leben und wird wiederholt. Eine geschaffte Boss-Kneipe bringt ein Leben. Mit dem letzten Leben ist der Run verloren.
3. **Routenwahl** zwischen zwei Kneipen: drei Wege (Zu Fuß, Bahn, Fahrrad). Jeder führt über genau eine Station, deren Kategorie je Weg gewürfelt und angezeigt wird: Tresen, Begegnung (Ereignis), Upgrade (Werkstatt: Modifikationen anbringen oder Flüche entfernen, selten) oder Rast (ein Leben zurück). Die nächste Kneipe (Name, Zielwert, Gäste) ist vorher sichtbar. Die drei Verkehrsmittel unterscheiden sich nur im Weg und in der gewürfelten Kategorie (Nachentscheidung des Nutzers vom selben Tag; der erste Entwurf gab der Bahn Kosten und ein sichtbares Ereignis und dem Fahrrad zwei Stationen).
4. **Spelunke** (Elite): Manchmal führt einer der drei Wege in eine Spelunke statt in die nächste Kneipe, ein eigener Ort mit eigenem Namen. Sie steht für diese Kneipe, hat einen höheren Zielwert und einen zufälligen Malus, der vorher angezeigt wird; Belohnung zusätzlich ein Relikt. Mali sind Daten (`tour.ron`).
5. **Boss-Kneipe** am Ende jedes Stadtteils: der Boss-Tisch wie bisher die Boss-Serie, ein Boss-Platz. Das Datenmodell ist für eine zweite Boss-Figur (Sekundant) offen, entschieden ist sie nicht (Frage im Cockpit).
6. **Namen**: Stadtteile und Kneipen bekommen deutsche Namen aus einem Namensgenerator mit Wortlisten als Daten, deterministisch je Seed. Die Kartengeometrie ist Darstellung: Der Kern liefert stabile Knoten-Ids und Seeds, die Web-App zeichnet.
7. **Der kurze Abend entfällt** vorerst. Alte Meta-Stände mit Statistik und Einsätzen des kurzen Abends bleiben lesbar. Die Einsatz-Leiter gilt je Charakter für den einen Abend.

Laufende Runs von vorher werden einmalig verworfen (Spielstand-Version 3), der Meta-Zustand bleibt.

## Alternativen

- **Bei Abschnitten und Serien bleiben:** siehe Kontext. Nur Leben einzuführen hätte die Härte gemildert, aber zwischen den Serien bliebe nichts zu entscheiden.
- **Ein verzweigter Karten-Baum wie in Slay the Spire:** viele Knoten, freie Pfade über mehrere Stockwerke. Verworfen: Auf dem Handy hochkant schwer lesbar, viel Planung für einen Abend von 15 Kneipen, und der Skat soll im Mittelpunkt stehen, nicht die Wegfindung. Die Routenwahl nimmt den Kern davon (eine sichtbare Wahl zwischen drei Wegen vor jeder Kneipe) ohne den Baum.
- **Routen mit unterschiedlichen Verkehrsmitteln (Bahn kostet und zeigt mehr, Fahrrad zwei Stationen):** der erste Entwurf. Verworfen vom Nutzer: Die Unterschiede waren schwer zu lesen und lenkten von der eigentlichen Wahl ab (welche Station brauche ich jetzt?).
- **Spelunke als Malus auf der normalen Kneipe:** einfacher, aber der Ort verliert sein Gesicht; die Spelunke ist ein eigener Ort mit eigenem Namen.

## Folgen

- **Im Code** behalten `Series` (eine Kneipe) und `section` (ein Stadtteil) ihre Namen, damit Relikte, Hooks (`OnSeriesWon`) und Freischaltungen weiterlaufen; das Glossar ordnet sie zu. Neu sind `RunPhase::Route`, `RunAction::ChooseRoute`, `core/src/run/tour.rs` (Leben, Routen, Spelunke, Namen, Sicht `TourView`) und `core/content/tour.ron`.
- **Ereignisse** sind Stationen auf einem Weg und tragen eine Kategorie; das zufällige Ereignis statt des Tresens (`encounter_chance`) entfällt.
- **Freischaltungen** an Abschnitten werden umgeschrieben („Abschnitt 4 erreichen“ → „Stadtteil 2 erreichen“), die Ids bleiben.
- **Alle Zahlen sind vorläufig** (Zielwerte je Kneipe, Leben, Gewichte der Stationen, Spelunke, Bonus je übrigem Spiel): gesetzt nach Augenmaß, weil die Simulationen auf Wunsch des Nutzers ruhen. Sie werden gemessen, sobald der Simulator wieder läuft (DESIGN 5.0.4).
- **Die Web-App** bekommt eine einfache Routenwahl; die Karte der Stadt (Pixel-Stadtplan) wird getrennt entworfen und setzt auf die Daten der Sicht auf.

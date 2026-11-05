# Architecture Decision Records

Jede grundlegende Entscheidung bekommt eine Datei. Nummer fortlaufend, nie wiederverwenden. Eine Entscheidung wird nicht geändert, sondern durch eine neue ersetzt, die auf die alte verweist.

Vorlage: [`0000-template.md`](0000-template.md). Neue ADR anlegen mit dem Skill `/adr`.

| Nr. | Titel | Status |
|---|---|---|
| 0001 | ADRs verwenden | Angenommen |
| 0002 | Der Spieler ist immer Alleinspieler | Angenommen |
| 0003 | Null ist ein eigener Charakter | Ersetzt durch 0012 |
| 0004 | Rust-Kern, Godot-Frontend | Teilweise ersetzt durch 0009 |
| 0005 | Ein geteiltes Blatt für alle drei Spieler | Teilweise ersetzt durch 0015 (Modifikationen für alle, Karten hinzufügen und entfernen); die 32 gemeinsamen Karten bleiben |
| 0006 | Reizen wird durch die Ansage ersetzt | Angenommen, Reihenfolge geändert durch 0011 |
| 0007 | Web-Export auf dem vServer, Auto-Deploy über GitHub Actions | Angenommen |
| 0008 | Hinter dem bestehenden Caddy der Haushalt-App | Angenommen |
| 0009 | Web-Frontend mit WebAssembly-Kern statt Godot | Angenommen |
| 0010 | Deutsches Blatt in der Darstellung | Angenommen, Ober heißt Oberin seit 0014 |
| 0011 | Erst der Skat, dann die Ansage | Angenommen, ein Punkt geändert durch 0013 |
| 0012 | Null für alle, Nullspieler als Spezialist | Angenommen, Startblatt des Nullspielers geändert durch 0015 |
| 0013 | Schneider, Schwarz und Ouvert auch nach dem Skat | Angenommen |
| 0014 | Die Oberin statt des Obers | Angenommen |
| 0015 | Eigene Ausgabe jeder Karte | Angenommen, ersetzt 0005 teilweise, ändert 0012 |
| 0016 | Die Kneipentour statt Abschnitten und Serien | Angenommen, Boss-Kneipe geändert durch 0017, Leben entfallen durch 0019 |
| 0017 | Bosse v2: zwei Boss-Spieler, alles oder nichts | Angenommen, ersetzt den Boss-Teil von 0016; Leben (Punkte 3, 4) entfallen durch 0019 |
| 0018 | Der Faktor als dritte Stufe der Wertung | Angenommen, Zahlen vorläufig |
| 0019 | Keine Leben: Eine verlorene Kneipe beendet den Abend | Angenommen, ersetzt den Lebens-Teil von 0016 und 0017, Zahlen vorläufig |
| 0020 | Die lange Nacht: Endlos-Modus nach dem Alten | Angenommen, Zahlen vorläufig |

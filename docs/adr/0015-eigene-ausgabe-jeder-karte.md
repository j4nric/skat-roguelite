# 0015: Eigene Ausgabe jeder Karte

Datum: 2026-11-02 · Status: Angenommen · Ersetzt 0005 teilweise (Modifikationen für alle, Karten hinzufügen und entfernen) · Ändert das Startblatt des Nullspielers aus 0012

## Kontext

ADR 0005 legt ein geteiltes Blatt fest: Jede Kartenmodifikation und jede hinzugefügte oder entfernte Karte betrifft alle drei Spieler. Das war als Kern des Deckbaus gedacht (Designsäule 2, „deine Karten können bei den Gegnern landen“). In M4 bis M8 hat sich gezeigt, dass das Modell an vielen Stellen gegen sich selbst arbeitet:

- **Sonderregeln häufen sich.** Die Wandelkarte gilt nur in der Hand des Spielers, der Kern nimmt sie den Gegnern beim Geben ab. Heimweh holt eine Karte gezielt in seine Hand. Die Modifikation „Untot“ (in Arbeit) wirkt nie für eine Gegnerkarte. Jede neue Modifikation braucht eine eigene Antwort auf „und wenn ein Gegner sie hat?“.
- **Gute Modifikationen sind oft ein Geschenk an die Gegner.** Ein Glasdaus bringt den Gegnern 22 Augen, ein gezinkter Trumpf sticht gegen den Spieler. Die Spannung ist echt, fühlt sich im Spieltest aber eher nach Strafe als nach Entscheidung an, und sie verwässert jede Messung: Jede Modifikation muss auf beiden Seiten gemessen werden.
- **Hinzufügen und Entfernen brechen den Skat.** Ein entfernter Unter macht für alle Spitzen und Trümpfe anders, Blankos und ein Talon über 32 Karten brauchen eigene Regeln (4.4, 7.1, 7.3), das Mitzählen (Designsäule 3) verliert seinen festen Boden: Der Nullspieler spielt mit acht Siebenern und ohne Unter, kein Skatspieler zählt das.
- **Bosse haben kein Blatt.** Ein Boss unterscheidet sich nur durch eine Regel; seine Karten sind dieselben wie die des Spielers.

Der Nutzer hat am 02.11.2026 entschieden.

## Entscheidung

Gegeben wird weiter ein normales Skatblatt: 32 verschiedene Karten, 120 Augen, 10/10/10/2, mitzählbare Trümpfe. **Jeder Spieler besitzt aber seine eigene Ausgabe jeder Karte.** Wer eine Karte bekommt, hält seine Version: der Spieler seine aufgewertete, ein Stammgast seine normale, ein Boss die Boss-Ausgabe. Jeder spielt immer nur mit seinen eigenen Modifikationen.

Dazu entschieden:

1. **Keine Karte verlässt das Blatt, keine kommt hinzu.** Alles, was bisher Karten entfernt oder hinzugefügt hat, wird über Karteneffekte gelöst. Freigespielte Charaktere haben ein Grunddeck, das sich nur durch Modifikationen auf ihrer Ausgabe unterscheidet, nicht durch eine andere Zusammensetzung.
2. **Der Skat gehört dem Alleinspieler.** Die zwei Skatkarten sind immer Karten des Spielers, also seine Ausgaben, gegeben wie gedrückt.
3. **Gegnerkarten haben eine eigene Rückseite.** Jeder Gegner oder Boss kann eine eigene haben; der Spieler behält die Distel-Rückseite.

Daraus folgt eine Grundregel für den Kern (Auslegung von Claude): **Eine Karte zeigt immer die Ausgabe ihres Halters.** Wechselt sie den Halter (Drei drücken, Revolutionär, Heimweh beim Geben), wechselt ihre Ausgabe mit. Eine gespielte Karte bleibt im Stich in der Ausgabe dessen, der sie gespielt hat; ihre Augen zählen für den, der den Stich macht. Hausregeln und Boss-Regeln sind Regeln des Tisches und gelten weiter für jede Ausgabe.

## Alternativen

- **Beim geteilten Blatt bleiben (ADR 0005):** siehe Kontext. Die Spannung „meine Karte landet beim Gegner“ ist das Versprechen von Säule 2, wird aber mit jeder Modifikation teurer zu erklären, zu bauen und zu messen.
- **Drei echte Stapel:** Jeder Spieler hat ein eigenes Deck und zieht seine zehn Karten daraus (wie in Balatro oder Slay the Spire). Verworfen: Dann gibt es nicht mehr 32 verschiedene Karten am Tisch, sondern bis zu drei Eichel-Unter. Trumpfordnung, Spitzen, Bedienen gegen bekannte Restkarten und das Mitzählen (Säule 3) brechen, der Skat ist keine gemeinsame Ressource mehr, und die KI müsste mit drei unbekannten Decks rechnen. Das wäre ein anderes Spiel als Skat.
- **Modifikationen nur in der Hand des Spielers, Hinzufügen und Entfernen bleiben:** Löst die Sonderregeln der Modifikationen, aber nicht das Mitzählen. Ein Blatt mit fünf Untern oder acht Siebenern bleibt unzählbar, und die Gegner spielen weiter mit dem veränderten Blatt des Spielers.
- **Eigene Ausgaben nur für Bosse:** Gibt den Bossen ein Blatt, lässt aber alle Probleme des geteilten Blatts beim Spieler.

## Folgen

- **Säule 2 ändert ihren Inhalt.** „Das Blatt ist geteilt“ heißt jetzt: Die Karten sind geteilt, die Ausgaben nicht. Das Risiko des Deckbaus ist nicht mehr „meine Aufwertung hilft dem Gegner“, sondern „meine Aufwertung schläft, wenn ein Gegner die Karte bekommt“ (20 von 32 Karten sind bei den Gegnern) und „was ich ausspiele, kann im Stich des Gegners landen“ (Augen-Modifikationen zählen für den, der den Stich macht). Der Glasdaus bleibt das Gegenstück: Er bricht in der Ausgabe des Spielers.
- **Bosse bekommen eine Boss-Ausgabe** als Daten in `bosses.ron`, sichtbar vor der Boss-Serie wie die Regel. Stammgäste spielen die normale Ausgabe.
- **Sonderregeln entfallen.** „Nur in deiner Hand“ (Wandelkarte, Untot) ist die allgemeine Regel. Texte mit „egal, wer sie spielt“ werden umgeschrieben. Modifikationen, die nur über die Gegner wirkten (Aufgebrezelt, Schlusslicht), brauchen eine neue Auslegung; Informations-Modifikationen (Durchsichtig) und Modifikationen auf die Trumpffarbe (Stempel, Krone, Siegel, Hufeisen, Maßband, Schwarze Katze) verlieren einen Teil ihrer Wirkung, weil sie nur noch wirken, wenn der Spieler die Karte hält. Fragen dazu stehen im Cockpit, die Auslegungen im Design-Dokument (7.2, 18).
- **Entfernen und Hinzufügen entfallen** samt Talon und Blattgröße über 32: `DeckSpec::remove` und `add`, `Deck::remove`, `Deck::add`, `Deck::copy`, die Wahrscheinlichkeiten nach Blattgröße (7.4). Blankos gibt es nur noch als Karteneffekt (Boss-Regel der Oma, ein zerbrochener Glasdaus). Frisches Deck braucht eine neue Wirkung.
- **Der Nullspieler** bekommt statt „ohne Unter, vier Siebener mehr“ eine Modifikation auf seinen Untern (ändert ADR 0012 in diesem Punkt). Der Grandspieler behält seinen Goldrand-Daus, Zaungast, Wirtin und Alleinspieler die normale Ausgabe.
- **Die KI** kennt ihre eigene Ausgabe, die ihres Partners und die Ausgabe des Spielers (wie der Spieler die Boss-Ausgabe kennt), aber nicht, welche Karten der Spieler hält. Die Stichproben lösen jede verteilte Karte mit der Ausgabe ihres Halters auf.
- **Spielstände** mit verändertem Blatt (Nullspieler, zerbrochener Glasdaus, Frisches Deck) werden beim Laden in eine Ausgabe des Spielers umgerechnet. Geteilte Seeds geben dieselben Karten wie vorher, solange das Blatt des Charakters nicht verändert war.
- **Messungen** (Zielwerte je Charakter, Boss-Faktoren, Modifikationen) gelten nicht mehr und werden nachgeholt, sobald die Simulationen wieder laufen. Bis dahin sind alle Zahlen vorläufig.
- Der Umbau läuft in Schritten (Kern, Inhalte, KI, Web), siehe Design-Dokument 18.

# 0012: Null ist für jeden ansagbar, der Nullspieler wird ein Spezialist

Datum: 2026-11-01 · Status: Angenommen · Ersetzt 0003

## Kontext

ADR 0003 machte Null zu einem eigenen Charakter, der nur Null spielt, und erlaubte dem Alleinspieler nur Farbspiel und Grand. Die Sorge damals: Null passt nicht in die Wertung, und zwei gegensätzliche Deckbau-Richtungen machen das Balancing unmöglich. Seit M4 hat sich beides als lösbar gezeigt, und die Trennung hat eigene Nachteile:

- Die Wertung läuft auch für Null über Grundwert × Gewinnstufen (9.2, Formel 23 × Stufen). Null ist kein Fremdkörper mehr.
- Bei jedem Blatt „Farbe, Grand oder Null?“ zu fragen, ist im echten Skat die spannendste Entscheidung. Der Alleinspieler hat sie nicht.
- Der Nullspieler ist erst nach einem gewonnenen Run frei. Im Simulator gelingt das nur in 8 bis 12 von 20 Folgen, die Null-Inhalte bleiben lange gesperrt.
- Bosse treffen einen Charakter, der nur Null spielt, einseitig: Gegen den Schieber gewinnt er 21 % der Serien statt 67 %.

Der Nutzer hat am 01.11.2026 im Cockpit entschieden.

## Entscheidung

Jeder Charakter darf Null ansagen, wie in der Skatordnung (ADR 0011 gilt für den Ablauf). Die Null-Hausregeln und Null-Modifikationen kommen in den gemeinsamen Pool. Der Nullspieler bleibt als freischaltbarer Spezialist: Startblatt mit vielen Siebenern, Startrelikt Kiebitz, Null-Relikte häufiger am Tresen. Er darf auch Farbspiel und Grand ansagen.

## Alternativen

- Null für alle, Charakter Nullspieler entfällt: verliert eine Freischaltung und einen klar anderen Spielstil zum Einstieg.
- Bleibt wie bisher: siehe Kontext.

## Folgen

- Bau-Spannung im selben Run: Karten „leiser“ machen hilft beim Null und schadet beim Farbspiel. Relikte und Modifikationen brauchen eine klare Angabe, für welche Spielart sie wirken.
- Die Bot-Ansage muss für beide Charaktere zwischen Farbspiel, Grand und Null wählen. Die Zielwertkurve wird danach im Simulator nachgemessen.
- Bosse, die Null bisher nicht trafen (Oma, Schwarzer Peter), zieht wieder jeder Charakter.
- Alte Spielstände laden weiter; der Charakter im Spielstand bleibt gültig.

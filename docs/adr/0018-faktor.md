# 0018: Der Faktor als dritte Stufe der Wertung

Datum: 2026-11-04 · Status: Angenommen (Zahlen vorläufig bis zur Messung) · Ergänzt DESIGN 4.2

## Kontext

Die Wertung ist Grundwert × Multiplikator (DESIGN 4.2). Fast alle Relikte, Zinken und Begegnungen drehen additiv an einer der beiden Größen: +n Grundwert, +n Multiplikator. Zwei additive Größen miteinander multipliziert wachsen höchstens quadratisch. Für einen Endlos-Modus nach dem Alten (Roadmap M7, zurückgestellt) fehlt damit das Wachstum, das ein gut gebauter Run braucht, um mit exponentiell steigenden Zielwerten mitzuhalten.

Im Kern gibt es schon einen Spielwert-Faktor (`ScoreBreakdown::multiplier`, Typ `Factor`, Wirkungen `MultiplyGameValue` und `ScaleGameValue`). Er wird bisher nur vorübergehend oder bedingt genutzt: Glückssträhne (×2), Stammfarbe (×1,5), Einsatz-Begegnungen und Schnäpse fürs nächste Spiel (×2, ×3), Schwarzer Peter (×0,5). In der Oberfläche ist er eine Zeile der Abrechnung, keine eigene Größe.

Gefragt hat der Nutzer am 04.11.2026, ob ein globaler Multiplikator je gewonnener Kneipe, Relikt oder Begegnung das Wachstum bringen soll.

## Entscheidung

1. **Spielwert = Grundwert × Multiplikator × Faktor.** Der Faktor wird eine sichtbare dritte Größe mit eigenem Zeichen und eigener Box in der Wertung. Er beginnt bei ×1.
2. **Faktoren werden multipliziert, nie addiert.** Zwei Quellen mit ×1,5 ergeben ×2,25. Das ist die Quelle des exponentiellen Wachstums.
3. **Kein Automatismus.** Es gibt keinen globalen Faktor, der von selbst je Kneipe, Relikt oder Begegnung steigt. Faktoren kommen nur aus Quellen, die der Spieler findet, wählt und pflegt:
   - **Relikte mit festem Faktor unter einer Bedingung**, darunter eines je Farbe („×1,5, wenn du ein Spiel mit dieser Trumpffarbe gewinnst“), eines für Grand und eines für Null;
   - **wachsende Relikte**, die ihren Faktor im Lauf des Runs erhöhen (zum Beispiel +×0,1 je gewonnener Kneipe) und ihn unter einer Bedingung wieder verlieren können;
   - einzelne Begegnungen und Boss-Belohnungen; die bestehenden vorübergehenden Faktoren (Schnäpse, Einsatz) bleiben.
4. **Dauerhafte Faktoren der Relikte gelten nur für gewonnene Spiele.** Ein verlorenes Spiel zählt −Grundwert × Multiplikator, ohne diese Faktoren, damit ein starker Build nicht im selben Maß exponentiell verliert. Bestehende Faktoren, die ausdrücklich auch verloren gelten (Stammfarbe, Bock, Einsatz), bleiben, wie sie sind.
5. **Gerundet wird einmal am Ende**, halb auf, wie heute.
6. **Alle Zahlen sind vorläufig**, bis eine Runde im Simulator sie festlegt (vom Nutzer am 04.11.2026 so gewünscht).

## Alternativen

- **Globaler Faktor je gewonnener Kneipe** (die Frage des Nutzers): verworfen. Er wirkt wie eine verschobene Zielkurve; die Zielwerte müssten genauso mitwachsen, und das Wachstum hinge an keiner Entscheidung des Spielers.
- **Den Multiplikator multiplikativ machen** (jede Stufe ×1,1 statt +1): verworfen. Es bricht die Skatformel, die das Spiel trägt (Designsäule 1), und jede bestehende Zahl.
- **Nur seltene Zinken mit Faktor:** zu schmal; Zinken wirken pro Karte und Spiel, ein Build braucht eine dauerhafte Quelle. Eine einzelne seltene Zinke mit Faktor bleibt möglich.

## Folgen

- **Kern:** Der vorhandene Faktor wird als eigene Größe geführt (Sicht, Vorschau `ValuePreview`, Abrechnung `ScoreStep`), Faktoren aus Relikten werden multipliziert. Neue Bedingungen für die Farb-Relikte (gewonnen und Trumpffarbe) und ein Zähler je wachsendem Relikt; die Hook-API reicht dafür weitgehend aus.
- **Inhalte:** eine neue Relikt-Familie „Faktor“ in `house_rules.ron`, Seltenheit Selten und Legendär, Preise und Werte nach der Messung.
- **Oberfläche:** ein viertes Wert-Zeichen neben Münze, Würfelauge, Grundstein und Steilpfeil, eine dritte Box „Faktor“ in der Wertung, das Zeichen auch im Glossar und im Band von Zinken.
- **Balance:** Die Zielkurve und die Bot-Werte müssen neu gemessen werden. Der Endlos-Modus kann danach mit exponentiellen Zielwerten (zum Beispiel ×1,6 je Stadtteil) geplant werden.

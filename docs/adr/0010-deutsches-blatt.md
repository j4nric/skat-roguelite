# 0010: Deutsches Blatt in der Darstellung

Datum: 2026-11-01 · Status: Angenommen

## Kontext

Für die Karten standen das vierfarbige Turnierblatt, das französische und das deutsche Blatt zur Wahl. Der Nutzer hat das deutsche Blatt gewählt.

## Entscheidung

Das Spiel zeigt das deutsche Blatt. Die Regeln und der Code bleiben unverändert, nur die Darstellung und die Anzeigetexte ändern sich:

| Code | Französisch | Deutsches Blatt (Anzeige) | Grundwert |
|---|---|---|---|
| `Clubs` | Kreuz | Eichel | 12 |
| `Spades` | Pik | Grün | 11 |
| `Hearts` | Herz | Rot | 10 |
| `Diamonds` | Karo | Schellen | 9 |
| `Ace` | Ass | Daus | 11 Augen |
| `King` | König | König | 4 Augen |
| `Queen` | Dame | Ober | 3 Augen |
| `Jack` | Bube | Unter | 2 Augen |

Die Bilder zeichnet Claude selbst als Vektorgrafik, in der Tradition des Altenburger Skatblatts mit deutschen Farben. Die Ecken tragen große Zeichen für das Handy.

## Alternativen

Vierfarbiges Turnierblatt: auf kleinen Bildschirmen etwas besser lesbar, aber nicht gewählt.

## Folgen

- Code, Tests und Design-Dokument verwenden weiter die französischen Bezeichner und Begriffe (Glossar). Nur Oberfläche und Spieltexte sprechen vom Unter, Ober und Daus.
- Inhalte, deren Namen am französischen Blatt hängen (z. B. „Alte Dame“), bekommen Anzeigenamen im deutschen Blatt (z. B. „Alter Ober“).
- Die Lesbarkeit der vier Farben auf dem Handy muss früh im Browser geprüft werden.

# 0006: Reizen wird durch die Ansage ersetzt

Datum: 2026-10-31 · Status: Angenommen

## Kontext

Das Reizen entscheidet im echten Skat, wer Alleinspieler wird, und erzeugt das Risiko des Überreizens. Da der Spieler immer Alleinspieler ist (ADR 0002), hat das Reizen keine Funktion mehr.

## Entscheidung

Vor dem Aufnehmen des Skats wählt der Spieler Spielart und Risikostufe (Normal, Hand, Schneider angesagt, Schwarz angesagt, Ouvert). Angesagte Stufen erhöhen die Gewinnstufen, ein Fehlschlag auf einer angesagten Stufe gilt als verloren. Überreizen gibt es nicht.

## Alternativen

Reizen gegen die KI beibehalten: erzeugt Situationen, in denen der Spieler nicht spielen darf, was der Prämisse „immer solo“ widerspricht.

## Folgen

Jedes Spiel beginnt mit genau einer Entscheidung. Relikte wie „Kontra“ können das Risiko nachträglich erhöhen, ohne das Reizen zu brauchen.

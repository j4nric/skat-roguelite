---
name: new-relic
description: Neues Relikt (Hausregel, Stammtisch-Figur oder Verbrauchsgegenstand) anlegen, mit Datendatei und Test. Aufruf: /new-relic <typ> <id> "<Beschreibung des Effekts>".
---

Lege ein neues Relikt an. Argumente: `$ARGUMENTS` (Typ: `house_rule`, `regular` oder `consumable`; id in snake_case; Effektbeschreibung in Anführungszeichen).

1. Prüfe, ob die id schon in `core/content/` oder `docs/content/` existiert. Wenn ja, abbrechen und sagen, wo.
2. Formuliere in einem Satz, an welchem Hebel der Spielwertformel der Effekt dreht (Grundwert, Spitzen, Gewinnstufen, Augen, Schwellen, Information). Geht das nicht, abbrechen: Das Relikt verstößt gegen Designsäule 1.
3. Existiert die Hook-API in `core/src/content/`?
   - Ja: RON-Datei in `core/content/<typ>s/<id>.ron` nach der Vorlage in `.claude/agents/content.md` anlegen, plus Test in `core/tests/content/<id>.rs`, der das Relikt lädt und den Effekt in einer konstruierten Situation prüft.
   - Nein: Entwurf in `docs/content/<typ>s/<id>.md` anlegen mit Name, Text, Hebel, benötigten Hooks und Platzhalter-Preis.
4. Bezeichner nach `docs/GLOSSARY.md`, Anzeigetext auf Deutsch im Stammtisch-Ton.
5. `cargo test` ausführen, falls Code entstanden ist.
6. Am Ende: Pfad der Datei, Hebel in einem Satz, Preis als Platzhalter mit Hinweis, dass `balance` ihn festlegt.

---
name: adr
description: Neues Architecture Decision Record in docs/adr/ anlegen. Aufruf: /adr "<Titel>". Nummer wird automatisch vergeben, Index-Tabelle wird aktualisiert.
---

Lege ein neues ADR an. Titel: `$ARGUMENTS`

1. Höchste vorhandene Nummer in `docs/adr/` ermitteln und um eins erhöhen (vierstellig, führende Nullen).
2. Dateiname: `<nummer>-<titel-in-kebab-case-ohne-umlaute>.md`.
3. Inhalt nach `docs/adr/0000-template.md`: Kontext, Entscheidung, Alternativen, Folgen. Datum heute, Status „Vorgeschlagen“, außer der Nutzer sagt „Angenommen“.
4. Falls Kontext und Entscheidung aus dem Gespräch bekannt sind, ausfüllen. Sonst die Abschnitte mit je einer Leitfrage stehen lassen und den Nutzer bitten, sie zu beantworten.
5. Zeile in der Tabelle in `docs/adr/README.md` ergänzen.
6. Falls die Entscheidung einem bestehenden ADR widerspricht: dessen Status auf „Ersetzt durch <nummer>“ setzen und im neuen ADR darauf verweisen.
7. Falls das Design-Dokument betroffen ist, die Stelle nennen, aber nicht ungefragt ändern.

# 0023: Englisch als zweite Sprache

Datum: 2026-11-10 · Status: Angenommen · Ersetzt teilweise DESIGN 16 („Sprache: Nur Deutsch für v1“) und die Regel „Nur Deutsch“ in CLAUDE.md; ergänzt 0010

## Kontext

Das Spiel war nur deutsch (DESIGN 16, Entscheidung vom 01.11.2026). Freunde ohne Deutsch sollen mitspielen können. Der Nutzer hat entschieden: Englisch kommt als zweite Sprache dazu, Deutsch bleibt die Voreinstellung, und das deutsche Blatt bleibt das Gesicht des Spiels, auch auf Englisch (kein französisches Blatt).

## Entscheidung

**Zwei Übersetzungsdateien.** Alle Spieltexte stehen in `web/src/i18n/de.ts` und `web/src/i18n/en.ts`. `en.ts` ist gegen die Form von `de.ts` getypt: Fehlt ein Schlüssel, schlägt `tsc` fehl. `web/scripts/check-i18n.mjs` (Teil von `npm run typecheck`) vergleicht dazu die offenen Tabellen (Fehlertexte und Ähnliches), die Platzhalter (`{n}`, `{name}` …) und die Länge der Listen. Neue Texte kommen immer in beide Dateien.

**Wahl der Sprache.** Beim ersten Start entscheidet die Browsersprache (`navigator.languages`): Deutsch für deutsche Browser, Englisch für alle anderen. Im Menü steht „Sprache / Language“; die Wahl wird wie die anderen Einstellungen im Browser gespeichert (`stammtisch.lang`, mit dem Spielstand-Code übertragbar) und gilt sofort, ohne Neuladen. Englisch ist ein eigenes Bundle-Stück, das nur geladen wird, wenn es gebraucht wird. Zahlen und Daten folgen der Sprache (Dezimalkomma oder -punkt, `Intl`).

**Inhalte des Kerns.** Namen und Texte der Relikte, Zinken, Schnäpse, Bosse, Begegnungen, Freischaltungen und Mali kommen weiter deutsch aus `core/content/*.ron`. Der Kern bleibt unverändert und kennt keine Sprachen. Die Web-Schicht legt die englischen Texte nach Content-Id darüber (`web/src/i18n/en-content.ts`, eingesetzt in `web/src/i18n/localize.ts` an der Grenze zum Kern), wie `de.content` es schon für Anzeigenamen im deutschen Blatt tat. Die Prüfung meldet jede Id der Content-Dateien ohne englischen Eintrag. Kneipennamen und die Vornamen der Stammgäste bleiben deutsch: Es sind Orts- und Personennamen und gehören zur Kneipe (Zusätze wie „Onkel“ oder „vom Bau“ werden übersetzt). Stadtteile bekommen englische Namen.

**Begriffe.** Die verbindliche Zuordnung steht im Glossar („Englische Anzeige“). Farben: Acorns, Leaves, Hearts, Bells. Bilder: Unter, Ober, King, Ace. Grand, Null, Ramsch, Schneider, Schwarz, Hand, Ouvert, Kontra und Re bleiben deutsch wie in der englischen Skat-Literatur; die Regelkarte erklärt sie unter „Skat words“.

## Alternativen

- **Englische Texte in den RON-Dateien** (`name_en`, `text_en`) und eine Sprache als Parameter des Kerns: Der Kern bliebe nicht frei von Anzeige, jede Sicht müsste die Sprache kennen, und jeder neue Text bräuchte eine Änderung im Kern. Verworfen.
- **Text-Ids im Kern statt Texten**: sauberer, aber ein großer Umbau aller Sichten und Content-Dateien. Die Ids der Inhalte sind schon eindeutig; die Web-Schicht kann sie nutzen. Verworfen.
- **Französisches Blatt im englischen Spiel**: vom Nutzer ausgeschlossen.
- **Neu laden beim Sprachwechsel**: einfacher, aber unnötig; die Oberfläche zeichnet ohnehin alles aus dem Zustand neu.

## Folgen

- Jeder neue Spieltext braucht zwei Fassungen. Strenge Typen und die Prüfung machen ein Vergessen sichtbar.
- Neue Inhalte in `core/content/*.ron` brauchen einen englischen Eintrag in `en-content.ts`; die Prüfung schlägt sonst fehl. Ohne Eintrag zeigte das Spiel den deutschen Text.
- Der Bezeichner `de` in der Web-App steht seit diesem ADR für die Texte der **aktiven** Sprache (eine lebende ES-Modul-Bindung). Der Name blieb, damit laufende Zweige ohne Umbenennung zusammengeführt werden können.
- Der Rauchtest läuft weiter deutsch (jeder Browser-Kontext mit `de-DE`), dazu ein englischer Durchlauf (`SMOKE_ONLY=english`).
- Das Web-Manifest bleibt eine Datei; seine Beschreibung nennt beide Sprachen.

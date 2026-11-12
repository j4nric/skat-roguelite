# 0027: Fehlerberichte aus der App, gespeichert vom Konto-Dienst

Datum: 2026-11-12 · Status: Angenommen · Ergänzt 0021 und 0026

## Kontext

Der Nutzer testet auf dem iPhone. Manche Fehler treten nur ab und zu auf (etwa: „Weiter“ im Rundenende reagiert nicht) und lassen sich am Rechner nicht nachstellen. Ohne Konsole und ohne Absturz-Dienst sieht niemand, was die App in dem Moment tat. Gebraucht wird ein Weg, einen Fehler direkt im Spiel zu melden, so dass Claude die Diagnose bekommt, ohne die Grundsätze aus 0021 (der Server speichert nur, liest keinen Spielstand, kein Tracking) und 0026 (iOS-App über denselben Konto-Dienst) aufzugeben.

## Entscheidung

**Diagnose-Puffer im Client** (`web/src/diag.ts`): ein Ringpuffer der letzten 200 Einträge dieses Besuchs. Er notiert Bildschirm, Phase und offenen Dialog bei jeder Änderung (beim Rundenende auch, ob noch eine Wiedergabe auf die Abrechnung wartet), Taps auf Knöpfe mit `data-action` (nur der Name der Aktion und die Test-Id), Taps, nach denen sich 1,5 s lang weder Zustand noch Bildschirm ändern („toter Tap“, nur eine Notiz, kein Eingriff), Drücke auf einen Knopf, aus denen kein Klick wird (mit dem Element, das unter dem Finger lag), abgefangene Fehler (`error`, `unhandledrejection`, `console.error`, `console.warn`), Größe des Bildschirms und der sicheren Ränder, Sichtbarkeit der Seite und Netz. Er lebt im Speicher; eine gekürzte Kopie (120 Einträge, höchstens 40 000 Zeichen) liegt in `localStorage` unter `stammtisch-diag.last`, außerhalb der Spielstand-Schlüssel, damit ein Bericht nach einem Neustart der App die letzte Sitzung noch hat.

**Bericht** („Fehler melden“ im Menü und auf dem Ladefehler-Bildschirm): ein Blatt mit Textfeld, aufklappbarer Liste dessen, was mitgeht, Häkchen „Spielstand mitschicken“ (voreingestellt an; es ist der Text aus „Spielstand sichern“) und „Senden“. Der Bericht ist JSON in einer Zeichenkette (`web/src/report.ts`): Text, Plattform (`web`, `pwa`, `ios`), Build, Sprache, Browserkennung, Einstellungen (Tempo, Ton, Schrift, reduzierte Bewegung), Viewport, Zustand, beide Puffer, Spielstand. Unter 240 KB: Zu viel kürzt erst den alten Puffer, dann den neuen, dann fällt der Spielstand weg (`saveNote`). Danach zeigt die App die Nummer (`B-7K3Q`). Ohne Netz, oder wenn der Server gerade nicht annimmt, wartet der Bericht in `stammtisch-diag.queue` (höchstens zwei) und geht beim nächsten Online-Sein oder Start raus, mit einem kurzen Hinweis samt Nummer. Rückfallweg ist „Als Text kopieren“.

**Server** (`server/src/report.rs`): `POST /api/reports` mit `{platform, version, report}`, ohne Anmeldung; mit Sitzung (Cookie oder Bearer-Token der App) wird die Konto-Id vermerkt. Gleiche Herkunftsprüfung und CORS für die App wie bei allen ändernden Anfragen. Inhalt höchstens 256 KiB, eigener Topf der Begrenzung (5 je Adresse und Stunde), zusammen höchstens 256 MiB (sonst 507). Tabelle `reports` (Migration 4): Nummer (`B-` und vier Zeichen Crockford-Base32, zufällig, bei Kollision eine neue), Zeit, Plattform, Version, Konto, Text. Der Server liest den Text nie. Kein Lesezugriff über die API.

**Datenschutz und Aufbewahrung.** Keine personenbezogenen Daten im Puffer: kein eingegebener Text, kein Name, keine E-Mail, kein Token, keine Passkey-Daten, kein Fragment der Adresse (Umzugs-Code, 0025); lange Zeichenketten, Spielstand-Texte, `Bearer …` und E-Mail-Adressen in Fehlermeldungen werden ersetzt. Berichte werden nach 90 Tagen gelöscht (stündliches Aufräumen), die eines Kontos mit dem Konto (`ON DELETE CASCADE`). IP-Adressen zählen wie bisher nur im Speicher für die Begrenzung.

**Abholen.** Das Binary hat ein Unterkommando `stammtisch-server reports [--since T] [--id B-XXXX] [--limit N]`, das die Datenbank nur lesend öffnet und JSON ausgibt. Der Workflow `.github/workflows/bug-reports.yml` (nur `workflow_dispatch`) ruft es per SSH mit den Deploy-Secrets im Container auf, zeigt im Log eine Kurzfassung je Bericht und lädt das ganze JSON als Artefakt hoch (7 Tage).

## Alternativen

- **Fremder Absturz- oder Analyse-Dienst** (Sentry, Crashlytics): sammelt dauernd und ohne Zutun, Daten bei Dritten, für die App-Datenschutzangaben „Tracking“-nah. Wir wollen nur, was der Spieler bewusst schickt.
- **Bericht per E-Mail** (`mailto:` mit dem Puffer): braucht eine Adresse, ist auf dem Handy umständlich und für große Texte unzuverlässig. Bleibt als „Als Text kopieren“ übrig.
- **GitHub-Issue direkt aus der App**: bräuchte ein Token in der App oder auf dem Server und machte Berichte öffentlich.
- **Lesen über die API mit Admin-Token**: ein weiteres Geheimnis auf dem Server und eine Angriffsfläche; SSH mit den vorhandenen Deploy-Secrets reicht.
- **`sqlite3` auf dem Server**: nicht im Alpine-Container, nicht auf dem Host garantiert; das Binary kann es selbst.
- **Dauerhaftes Logging aller Sitzungen auf dem Server**: Tracking. Der Puffer bleibt auf dem Gerät, bis der Spieler sendet.
- **Tote Taps automatisch beheben** (etwa die Abrechnung schließen): verdeckt den Fehler, den wir finden wollen. Nur notieren.

## Folgen

- Der Konto-Dienst speichert jetzt auch Daten ohne Konto. Das ist bewusst begrenzt (Größe, Rate, Gesamtmenge, 90 Tage) und bleibt undurchsichtig für ihn; ADR 0021 („speichert nur“) gilt weiter.
- Die App-Datenschutzangaben in App Store Connect bekommen Diagnosedaten und Kundensupport-Inhalte (`docs/ios.md`).
- Tote Taps sind eine Heuristik: Eine Animation, die gerade den Bildschirm ändert, kann einen toten Tap verdecken; ein Knopf, dessen Wirkung erst später sichtbar wird, kann fälschlich als tot gelten. Für die Diagnose reicht das.
- Wer den Puffer ändert, achtet auf dieselben Regeln: keine eingegebenen Texte, keine Tokens, nichts aus dem Adress-Fragment.
- Smoke-Test: `reportPass` (`SMOKE_ONLY=report`) mit gestubbtem `/api/reports`, online und offline.

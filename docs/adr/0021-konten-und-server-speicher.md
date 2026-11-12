# 0021: Konten mit Passkey und Spielstand auf dem Server

Datum: 2026-11-07 · Status: Angenommen · Ändert DESIGN 9.3 („Kein Konto, kein Server-Speicher“) und ergänzt 0007 und 0008

## Kontext

Bis hierher lag der Fortschritt nur im Browser (`localStorage`). Der gesicherte Spielstand als Text (`STAMMTISCH1:…`, `web/src/savecode.ts`) hilft beim Umzug, ist aber Handarbeit. Wer auf Handy und Rechner spielt oder den Browser leert, verliert den Stand. Dazu fehlt dem Tages-Seed eine gemeinsame Tafel, und die braucht Namen.

Der Nutzer hat im Cockpit (Frage 38) Variante A gewählt und ergänzt: Anmelden mit **Passkey** (Fingerabdruck, Gesicht, Geräte-PIN), dazu ein **Wiederherstellungscode**, der beim Anlegen einmal angezeigt wird; angemeldet darf man einen **Namen** setzen; der Fortschritt liegt auf dem Server. Ohne Anmeldung bleibt alles wie bisher.

## Entscheidung

**Ein kleiner Rust-Dienst** (`server/`, Binary `stammtisch-server`: axum, SQLite mit gebündelter Bibliothek, webauthn-rs) läuft neben dem statischen Spiel. Unser Caddy leitet `/api/*` an ihn weiter. Er kennt `skat_core` nicht und **prüft nie Spielzustand**: Die Spiellogik bleibt im Browser (Rust-Kern als WebAssembly, ADR 0009). Für den Server ist der Spielstand ein undurchsichtiger Text.

**Anmelden.**
- Konto anlegen = einen auffindbaren Passkey (resident key, Nutzerprüfung verlangt) für die Seite erzeugen. Kein Benutzername, keine E-Mail, kein Passwort. Die Benutzerkennung des Passkeys ist die Konto-Id (UUID).
- Anmelden mit Passkey: Der Browser bietet die Passkeys an, die er für die Seite kennt (auch synchronisierte aus iCloud-Schlüsselbund oder Google-Passwortmanager).
- **Wiederherstellungscode**: 24 Zeichen Crockford-Base32 (120 Bit) in sechs Vierergruppen. Die ersten 8 Zeichen sind eine öffentliche Kennung zum Finden des Kontos, die übrigen 16 (80 Bit) liegen nur als argon2id-Hash vor. Der Code wird beim Anlegen genau einmal angezeigt (mit Knopf zum Kopieren und dem Hinweis, ihn aufzuschreiben). Wer sich damit anmeldet, bekommt sofort einen neuen Code; der alte gilt nicht mehr. Danach bietet die App an, auf diesem Gerät einen Passkey anzulegen. Angemeldet lässt sich der Code jederzeit erneuern.
- Sitzung: zufälliges Token in einem Cookie `HttpOnly; Secure; SameSite=Lax; Path=/api`, 180 Tage, verlängert sich bei Gebrauch. Gespeichert wird nur sein SHA-256. Ändernde Anfragen mit fremdem `Origin` werden abgewiesen. Leichte Begrenzung je Adresse im Speicher (Anmeldung 30, Wiederherstellung 10, Schreiben 120, Lesen 300 je Minute).

**Name.** 1 bis 24 Zeichen nach dem Säubern (Steuerzeichen und unsichtbare Zeichen wie Bidi-Überschreibungen fallen weg, Leerraum wird zu einem Leerzeichen, getrimmt). Nicht eindeutig. Ohne Namen steht „Namenlos“ auf der Tafel.

**Was gespeichert wird.** Je Konto: Name (optional), die öffentlichen Schlüssel der Passkeys mit Zähler, Kennung und Hash des Wiederherstellungscodes, Sitzungen (Hash), **ein Spielstand** und die Ergebnisse auf der Tafel des Tages. Der Spielstand ist genau der gesicherte Text (`STAMMTISCH1:…`), also dieselben Schlüssel, die der Text trägt: laufender Run, Meta-Zustand (Vereinsbuch, Freischaltungen, Statistik), Liste der letzten Runs, Einstellungen, gesehene Hinweise. Höchstens 1 MB, dazu eine Version und die Zeit der letzten Änderung (`updatedAt`). Keine E-Mail, keine IP-Adressen auf Dauer, kein Tracking.

**Abgleich** (`web/src/sync.ts`).
- Beim Start, angemeldet, bevor die App ihren Speicher liest: den Stand des Kontos holen und vergleichen (höchstens 5 Sekunden, sonst „offline“ und weiter mit dem lokalen Stand).
- Hochladen nach jeder gewonnenen oder verlorenen Kneipe, am Ende des Runs und wenn die Seite verlassen oder verborgen wird, jeweils nur wenn sich lokal etwas geändert hat.
- Jedes Hochladen nennt die Version, auf der es aufbaut (`ifVersion`). Ist der Server weiter, antwortet er 409 und überschreibt nichts.
- Wer gewinnt: Hat sich seit dem letzten Abgleich nur eine Seite geändert, gilt sie (sie ist die neuere). Der Stand des Kontos ersetzt den lokalen nur beim Start oder direkt nach dem Anmelden, mit sichtbarem Hinweis („Spielstand aus deinem Konto geladen“). Haben sich beide geändert, oder hat ein Gerät beim ersten Anmelden eigenen Fortschritt, **fragt die App**: Ein Dialog zeigt beide Stände (Zeit, Charakter, Stadtteil und Kneipe, Seed, Vereinsbuch), der neuere ist markiert, der Spieler wählt. Bis dahin wird nichts hochgeladen. Ein älterer lokaler Stand überschreibt nie still einen neueren auf dem Server.
- Ein Stand aus einer neueren Version des Spiels (der Kern kann ihn nicht laden) wird nie überschrieben: Die App zeigt „Seite neu laden“ und lädt nichts hoch.
- Kleine Anzeige des Zustands neben dem Konto (Startbildschirm, Menü): „gespeichert“, „speichert …“, „offline“, „Konflikt“ (antippen öffnet die Wahl).

**Tafel des Tages.** Ein angemeldeter Spieler meldet am Ende eines Tages-Runs sein Ergebnis (Tages-Seed, Charakter, gewonnen, Stadtteil, Kneipe, Einsatz, bestes Spiel); das erste Ergebnis je Konto und Tag zählt. Reihenfolge: gewonnen, dann weiter gekommen (Stadtteil, Kneipe), dann höherer Einsatz, dann bestes Spiel, dann wer zuerst da war. Der Startbildschirm zeigt die besten zehn mit Namen und den eigenen Platz; ohne Konto nur lesend mit einem Hinweis zum Anmelden. Weil der Server keine Spiellogik hat, sind Ergebnisse Selbstauskunft: Wer die App verändert, kann schummeln. Für eine Tafel unter Freunden ist das hingenommen; ein Nachspielen des Runs auf dem Server (der Seed bestimmt ihn) bliebe möglich, braucht aber den Kern auf dem Server und ein eigenes ADR.

**Datenschutz und Löschen.** Konto löschen (Knopf mit Rückfrage in der App, `DELETE /api/account`) entfernt Konto, Passkeys, Sitzungen, Spielstand und Tafel-Ergebnisse sofort. Der Fortschritt im Browser bleibt. Der Passkey bleibt im Passwortmanager des Spielers, öffnet aber nichts mehr.

**Betrieb** (ergänzt 0007 und 0008). Der Dienst wird in GitHub Actions als statisches musl-Binary gebaut, per `rsync` nach `/opt/skat/server/` gebracht und in einem schlichten Alpine-Container gestartet. Er hängt nur im internen Netzwerk `backend`, das allein unser Caddy erreicht; er veröffentlicht keine Ports und hängt nicht im Netzwerk `stammtisch-proxy`. Die Datenbank liegt in `/opt/skat/data/stammtisch.db`. Der Caddy der Haushalt-App braucht keine Änderung: Er reicht schon alles an unseren Caddy weiter. Startet der Dienst nicht, liefert der Deploy das Spiel trotzdem aus; nur `/api` antwortet dann 502 und die App zeigt „offline“.

**Domain.** WebAuthn bindet Passkeys an die RP-Id, hier den vollen Host `stammtisch-62-83-23-90.sslip.io` (Ursprung `https://stammtisch-62-83-23-90.sslip.io`). `sslip.io` steht nicht auf der Public Suffix List, der Host ist also eine gültige RP-Id. Zieht das Spiel auf eine andere Domain um, gelten die alten Passkeys dort nicht mehr. Dann meldet man sich einmal mit dem Wiederherstellungscode an und legt einen neuen Passkey an. Deshalb ist der Code Pflicht und wird bei jedem Gebrauch ersetzt. *(Geändert durch ADR 0025: Umzug auf one-more-skat.de mit beiden Adressen als Relying Party und einem einmaligen Umzugs-Code.)*

## Alternativen

- **Kein Konto, nur der gesicherte Text** (bisher): bleibt als Weg ohne Server, löst aber den Gerätewechsel nicht von selbst und gibt keine Namen für die Tafel.
- **E-Mail mit Anmeldelink**: braucht Mailversand, speichert eine E-Mail-Adresse und ist auf dem Handy umständlicher als ein Fingerabdruck.
- **Benutzername und Passwort**: Passwörter wollen wir nicht speichern; vergessene Passwörter bräuchten wieder E-Mail.
- **Anmelden über Google, Apple oder GitHub**: Fremdabhängigkeit, Datenweitergabe, Registrierung als App bei jedem Anbieter.
- **Nur Wiederherstellungscode ohne Passkey**: ginge, ist aber ein Passwort in anderer Form; als Rückfallweg ist er gut, als täglicher Weg zu umständlich.
- **Spielstand serverseitig prüfen oder zusammenführen**: braucht den Kern auf dem Server und Regeln zum Zusammenführen von Runs, die es nicht gibt. Der Spieler entscheidet bei einem Konflikt selbst.
- **Neueres `updatedAt` gewinnt immer still**: Uhren gehen auseinander, und ein Fehlgriff kostet einen ganzen Abend. Deshalb nur dann still, wenn eine Seite unverändert ist; sonst fragen.
- **Container-Image aus einer Registry oder Bau auf dem Server**: Registry braucht Zugangsdaten und ein weiteres Secret; ein Bau auf dem Server bräuchte Rust dort (ADR 0007 will keine Build-Werkzeuge auf dem Server). Ein statisches Binary in einem Standard-Image ist am einfachsten.
- **PostgreSQL**: für eine Handvoll Spieler ein weiterer Dienst ohne Nutzen. SQLite in einer Datei lässt sich mit einem Kopierbefehl sichern.

## Folgen

- Spielen ohne Konto bleibt unverändert; der gesicherte Text bleibt der Weg ohne Server. Ohne Server zeigt die App nur keine Tafel und beim Konto „offline“.
- Passkeys brauchen eine sichere Herkunft (HTTPS oder `localhost`). Lokal: `cargo run -p stammtisch_server` auf `127.0.0.1:8787`, Vite leitet `/api` dorthin (`npm run dev` und `npm run preview`).
- Die Datenbank ist jetzt Zustand auf dem Server, der gesichert werden muss (`deploy/README.md`).
- Neue Felder im Spielstand brauchen keine Server-Änderung, der Server liest ihn nie. Ändert sich das Format des gesicherten Texts, muss eine ältere App einen neueren Stand erkennen und in Ruhe lassen (heute: Version im Präfix, Fehler `newer`).
- Die Tafel ist Selbstauskunft (siehe oben).
- Bei einem Umzug der Domain gelten Passkeys nicht mehr; der Wiederherstellungscode trägt hinüber.

## Nachtrag (09.11.2026): Härtung nach Review

Ohne die Entscheidung zu ändern, nach einer Durchsicht von Server, Deploy und CI:

- **Passkeys verwalten, überall abmelden.** `GET /api/passkeys` listet die Passkeys des Kontos (angelegt, zuletzt benutzt, ob ein Passwortmanager ihn synchronisiert; einen Namen liefert der Authenticator nicht). `DELETE /api/passkeys/{id}` entfernt einen, nie den letzten (409 `last_passkey`). `POST /api/logout-all` beendet alle Sitzungen des Kontos, auch die eigene. Das Konto-Blatt der App zeigt beides („Passkeys verwalten“, „Überall abmelden“).
- **Wiederherstellungscode.** Anmelden mit dem Code und Erneuern beenden alle anderen Sitzungen des Kontos; wer den Code gestohlen hat, ist danach draußen. Das Ersetzen ist ein Vergleichen-und-Tauschen (`… WHERE recovery_hash = <alter Hash>`): Zwei gleichzeitige Anfragen mit demselben Code lösen ihn höchstens einmal ein.
- **Sitzung.** Wird die Sitzung verlängert (nach der Hälfte der 180 Tage), bekommt der Browser auch ein neues Cookie mit voller Laufzeit.
- **Begrenzungen.** IPv6-Adressen zählen je /64 (ein Anschluss hat meist ein ganzes /64). Höchstens 5 neue Konten je Adresse und Stunde. Höchstens 10 offene Passkey-Vorgänge je Adresse, jeder lebt 2 Minuten; sind alle 10 000 belegt, antwortet der Server 503 `busy`. Höchstens 2 argon2-Berechnungen gleichzeitig. Anfragen höchstens 16 KiB, `/api/save` 1,2 MB, die Passkey-Registrierung 64 KiB.
- **Speicherplatz.** Die Datenbank wächst höchstens auf 2 GiB (`PRAGMA max_page_count`); ist sie voll, antwortet ein Schreiben 507 `storage_full`, der Fortschritt bleibt im Browser. Ein Spielstand muss wie ein gesicherter Text beginnen (`STAMMTISCH<Zahl>:`), jede Formatnummer wird angenommen.
- **Herkunft.** Ändernde Anfragen (POST, PUT, DELETE) brauchen ein `Origin` der Seite; Browser senden es dabei immer, `curl` braucht `-H 'Origin: …'`. Dazu wird `Sec-Fetch-Site: cross-site` und `same-site` abgewiesen: `sslip.io` steht nicht auf der Public Suffix List, jede andere `*.sslip.io`-Seite wäre für den Browser „same site“ und bekäme das SameSite=Lax-Cookie mit.
- **Fehler** kommen immer als JSON (`{"error": "…"}`), auch die, die axum selbst erzeugt (kaputtes JSON, zu großer Inhalt, unbekannter Pfad).
- **Datenbank.** Das Schema ist eine Liste von Migrationen, gezählt mit `PRAGMA user_version` und in einer Transaktion angewandt; eine Datei aus einem neueren Server wird nicht angefasst. `synchronous=NORMAL` mit WAL. Einmal am Tag schreibt der Dienst mit `VACUUM INTO` eine Sicherung nach `data/backup/stammtisch-JJJJMMTT.db` (Datum nach der Uhr des Servers, die letzten sieben bleiben).
- **Betrieb.** Der Container des Dienstes läuft mit schreibgeschütztem Dateisystem (nur `data/` und ein `/tmp` im Speicher sind beschreibbar), ohne Capabilities, mit Healthcheck. Caddy setzt Sicherheits-Header für die ganze Seite, darunter eine Content-Security-Policy (`deploy/README.md`).

## Nachtrag (10.11.2026): Tafel der Woche

Ohne die Entscheidung zu ändern: Zur Tafel des Tages kommt die Tafel der Woche für die Wochen-Herausforderung (DESIGN 5.6). Sie ist gleich gebaut und teilt den Code (`server/src/board.rs`): Tabelle `weekly_results` (Migration 2), Schlüssel die ISO-Woche als `JJJJWW`, eine Woche Abstand zur Uhr des Servers erlaubt, das erste Ergebnis je Konto und Woche zählt, gleiche Prüfung und Begrenzung wie beim Tag. Konto löschen entfernt auch diese Ergebnisse. Der Server kennt weder Seed noch Sonderregel der Woche; beides ist Sache des Kerns im Browser.

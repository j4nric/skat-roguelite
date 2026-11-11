# 0025: Umzug auf one-more-skat.de, alte Adresse bleibt

Datum: 2026-11-11 · Status: Angenommen · Ändert den Abschnitt „Domain“ von 0021, ergänzt 0008 und 0024

## Kontext

Das Spiel hat eine eigene Domain: **https://one-more-skat.de** (`www` leitet um). Die iOS-App (Capacitor, später; Name „Noch 'ne Runde Skat“, Bundle-Id `de.onemoreskat.app`) soll die Passkeys dieser Domain über Associated Domains nutzen. Zwei Dinge hängen aber an der alten Adresse `stammtisch-62-83-23-90.sslip.io` und lassen sich nicht übertragen:

- **Passkeys** sind an die RP-Id gebunden, bisher `stammtisch-62-83-23-90.sslip.io`. Auf der neuen Domain bietet der Browser sie gar nicht an.
- **localStorage** gehört zur Herkunft. Wer ohne Konto spielt, hat seinen ganzen Fortschritt nur dort.

ADR 0021 sah für den Umzug nur den Wiederherstellungscode vor. Den haben viele nicht mehr zur Hand, und Spieler ohne Konto hätten nur den Spielstand-Text. Niemand soll Konto oder Fortschritt verlieren.

## Entscheidung

- **Beide Adressen bleiben.** Die alte Adresse wird **nicht** umgeleitet, sonst käme niemand mehr an ihren localStorage. Kanonisch ist one-more-skat.de: Teilen-Links und Run-Karte nennen sie, auch wenn man auf der alten Adresse spielt.
- **Ein Konto-Dienst, zwei Relying Parties.** `STAMMTISCH_ORIGIN` nennt beide Herkünfte, die neue zuerst. Jeder Host ist eine eigene RP-Id; eine Zeremonie läuft über die RP, zu der das `Origin` der Anfrage gehört. Alte Passkeys gelten weiter auf der alten Adresse, neue entstehen für die Adresse, auf der man gerade ist. Die Passkey-Liste nennt die Adresse (Spalte `rp_id`, alte Passkeys lernen sie beim nächsten Anmelden).
- **Umzug mit Konto: ein einmaliger Umzugs-Code.** Auf der alten Adresse zeigt der Startbildschirm „Wir sind umgezogen“. Angemeldet lädt „Konto mitnehmen“ zuerst den Spielstand hoch und holt dann `POST /api/move/start`: 32 Zufallsbytes (base64url), gespeichert nur als SHA-256, **10 Minuten** gültig, **einmal** einlösbar, ein neuer ersetzt den alten. Der Link `https://one-more-skat.de/#umzug=<Code>` trägt ihn im Fragment, das der Browser nie an einen Server schickt; die neue Adresse nimmt ihn sofort aus der Adresszeile und schickt ihn nur im Inhalt von `POST /api/move/redeem`. So steht er in keiner Anfragezeile und keinem Log. Einlösen meldet **dasselbe Konto** an (ohne andere Sitzungen oder den Wiederherstellungscode anzufassen) und bietet sofort einen Passkey für one-more-skat.de an. Der Spielstand kommt wie nach jedem Anmelden aus dem Konto.
- **Umzug ohne Konto: der Spielstand im Link.** „Spielstand mitnehmen“ legt den gesicherten Text (`STAMMTISCH1:…`) ins Fragment: `#spielstand=<Text>`. Er geht an keinen Server. Die neue Adresse prüft ihn mit dem Kern und fragt wie beim Einfügen, bevor sie etwas ersetzt. Zu große Stände (über 1,5 MB Link) gehen über den Text im Menü.
- **Apple-Datei.** `/.well-known/apple-app-site-association` mit `webcredentials.apps = ["<TEAMID>.de.onemoreskat.app"]`, als JSON ohne Umleitung. Die Team-Id kommt aus der Repository-Variable `APPLE_TEAM_ID`; ohne gültige (zehn Großbuchstaben oder Ziffern) antwortet die Adresse 404. Der Service Worker lässt `/.well-known/` in Ruhe.

Der Server bleibt, was 0021 sagt: Er speichert nur, kennt `skat_core` nicht und liest keinen Spielstand.

## Alternativen

- **Alte Adresse umleiten:** einfach, aber der localStorage der alten Adresse wäre verloren, und mit ihm jeder Fortschritt ohne Konto.
- **Nur der Wiederherstellungscode** (bisher geplant): sicher, aber wer ihn verlegt hat, kommt nicht hinüber.
- **Related Origin Requests** (WebAuthn L3, `/.well-known/webauthn`): Passkeys der alten RP-Id auch auf der neuen Domain. Nicht in jedem Browser, und die alte RP-Id bliebe für immer die des Kontos. Für die iOS-App muss es ohnehin die neue Domain sein.
- **RP-Id gemeinsam (Elterndomain):** sslip.io und one-more-skat.de haben keine gemeinsame Elterndomain.
- **Spielstand ohne Konto über den Server weiterreichen** (kurz zwischenlagern): ginge, aber der Server bekäme Daten ohne Konto, und das Fragment kann es ohne ihn.
- **Umzugs-Code zum Abtippen** (kurz, mit argon2): unnötig, der Link wird angetippt, nicht abgeschrieben; 256 Bit lassen sich nicht erraten, SHA-256 reicht.

## Folgen

- Deploy: `STAMMTISCH_ORIGIN=https://one-more-skat.de,https://stammtisch-62-83-23-90.sslip.io` (steht in `deploy/docker-compose.yml`). Für die Apple-Datei die Repository-Variable `APPLE_TEAM_ID` setzen; sonst nichts Neues auf dem Server. Migration 3 der Datenbank läuft beim Start von selbst.
- Wer sich auf one-more-skat.de mit einem alten Passkey anmelden will, findet ihn nicht; das Konto-Blatt dort sagt, wo er noch gilt.
- Der Umzugs-Code ist für zehn Minuten so viel wert wie eine Sitzung. Er entsteht nur angemeldet und mit unserem `Origin`, und die App warnt, ihn nicht weiterzugeben.
- Wann die alte Adresse abgeschaltet wird, ist offen; vorher sollten die meisten umgezogen sein. Bis dahin steht sie in `deploy/site.caddy`.
- Smoke-Test: `movePass` (`SMOKE_ONLY=move`) bedient beide Hosts über Playwright-Routen aus der Vorschau und prüft beide Wege.

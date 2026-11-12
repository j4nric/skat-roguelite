# iOS-App „Noch 'ne Runde Skat“

Die App ist eine Capacitor-Hülle um genau den Web-Build aus `web/` (ADR 0026). Sie lädt das Spiel aus ihrem Paket, ist also ohne Netz spielbar; das Konto spricht `https://one-more-skat.de/api` mit einem Bearer-Token, Passkeys laufen nativ über iOS für `one-more-skat.de`.

| Was | Wert |
|---|---|
| Name im Store | Noch 'ne Runde Skat |
| Name unter dem Symbol | Runde Skat (`CFBundleDisplayName`, `ios/App/App/*.lproj/InfoPlist.strings`) |
| Bundle-Id | `de.onemoreskat.app` |
| Geräte | iPhone, Hochformat, ab iOS 16 |
| Sprache | Deutsch und Englisch, aus der Systemsprache (wie im Browser; im Menü umstellbar) |
| Verschlüsselung | nur Standard-HTTPS: `ITSAppUsesNonExemptEncryption = NO` |
| Capacitor | 8.6.0 mit Swift Package Manager, Plugins `@capacitor/haptics`, `@capacitor/status-bar` und das eigene `NativeAccount` |

## Aufbau

| Pfad | Inhalt |
|---|---|
| `web/capacitor.config.json` | App-Id, Name, `webDir: dist`, Pfad `../ios`, Hintergrund, Statusleiste |
| `ios/App/App.xcodeproj` | Xcode-Projekt (Ziel `App`, Schema `App`) |
| `ios/App/CapApp-SPM/Package.swift` | Swift-Pakete; schreibt `npx cap sync ios`, nie von Hand |
| `ios/App/App/AppViewController.swift` | die Web-Ansicht, meldet das eigene Plugin an |
| `ios/App/App/NativeAccountPlugin.swift` | Passkeys (`ASAuthorization`) und Sitzungs-Token im Schlüsselbund |
| `ios/App/App/App.entitlements` | Associated Domains `webcredentials:one-more-skat.de` |
| `ios/App/App/Info.plist` | Hochformat, Statusleiste, Sprachen, Verschlüsselung |
| `ios/App/App/Assets.xcassets` | Icon und Startbild aus `web/scripts/make-ios-assets.py` |
| `web/src/platform.ts`, `web/src/native.ts` | Erkennen der App und die JavaScript-Seite des Plugins |

`ios/App/App/public/` (der kopierte Web-Build) und `capacitor.config.json` im App-Ordner entstehen bei `cap sync` und stehen nicht im Repository.

## Lokal

Unter Linux (ohne Xcode) geht alles außer dem eigentlichen Bau:

```sh
cd web
npm ci
npm run build            # Kern als WebAssembly und Vite-Build nach web/dist
npx cap sync ios         # kopiert dist in die App, schreibt Package.swift
npm run ios:assets       # Icon und Startbild neu zeichnen (Pillow)
```

Auf einem Mac danach `open ../ios/App/App.xcodeproj`, im Ziel `App` unter „Signing & Capabilities“ das Team wählen und auf einem iPhone oder im Simulator starten. Passkeys gehen im Simulator nur mit angemeldetem Apple-Konto und erst, wenn die Apple-Datei ausgeliefert wird (siehe unten).

## Workflow `.github/workflows/ios.yml`

| Auslöser | Was passiert |
|---|---|
| Pull Request oder Push auf `main`, der `ios/`, `web/`, `core/`, `wasm/` oder `Cargo.lock` ändert | Web-Build auf Ubuntu, dann auf macOS `cap sync` und ein Release-Bau für „generic iOS“ **ohne Signatur**. Prüft Swift, Projekt, Pakete und dass der Web-Build im Paket liegt. |
| Von Hand: Actions → **iOS** → „Run workflow“ (optional Version, Häkchen „Upload to TestFlight“) | Wie oben, dann Archiv mit **automatischer Signatur über den API-Schlüssel**, Export als App-Store-IPA, Prüfung und **Upload zu TestFlight**. |
| Tag `ios-v<Version>` (z. B. `git tag ios-v1.0.1 && git push origin ios-v1.0.1`) | wie „von Hand“, Version aus dem Tag (`1.0.1`). |

- **Build-Nummer** ist `github.run_number` dieses Workflows (steigt mit jedem Lauf, auch bei PRs; Lücken sind erlaubt).
- **Version** (`CFBundleShortVersionString`) kommt aus dem Tag oder der Eingabe, sonst aus dem Projekt (`MARKETING_VERSION = 1.0`). Für eine neue Version im Store eine höhere Version wählen; Builds derselben Version landen in TestFlight unter ihr.
- Der Schlüssel (`APPSTORE_API_KEY_P8`) wird nur nach `$RUNNER_TEMP` geschrieben und im letzten Schritt gelöscht, auch wenn etwas scheitert. Geheimnisse erscheinen nie im Log; bei einem Fehler lädt der Lauf nur die `xcodebuild`-Logs als Artefakt hoch.
- Nach dem Upload braucht App Store Connect einige Minuten, bis der Build in TestFlight erscheint („Processing“). Interne Tester bekommen ihn danach ohne Prüfung.

Benötigte GitHub-Secrets (sind angelegt): `APPSTORE_API_KEY_ID`, `APPSTORE_API_ISSUER_ID`, `APPSTORE_API_KEY_P8` (ganze .p8 mit BEGIN- und END-Zeile), `APPLE_TEAM_ID`.

## Was in Apple Developer und App Store Connect noch zu tun ist

Einmalig, in dieser Reihenfolge:

1. **Team-Id als Repository-Variable** (nicht nur als Secret): GitHub → Settings → Secrets and variables → Actions → **Variables** → `APPLE_TEAM_ID`. Daraus schreibt der Deploy `/.well-known/apple-app-site-association` (ADR 0025). Prüfen:
   `curl -s https://one-more-skat.de/.well-known/apple-app-site-association` muss `{"webcredentials":{"apps":["<TEAMID>.de.onemoreskat.app"]}}` zeigen. Apple holt die Datei über sein CDN; nach einer Änderung kann es bis zu einem Tag dauern (`curl -s https://app-site-association.cdn-apple.com/a/v1/one-more-skat.de`). Ohne sie gehen in der App keine Passkeys (der Wiederherstellungscode geht trotzdem).
2. **Bundle-Id** `de.onemoreskat.app` mit Capability **Associated Domains** (erledigt). Weitere Capabilities braucht die App nicht.
3. **Signatur.** Der Workflow lässt Xcode Zertifikat und Profile selbst anlegen (`-allowProvisioningUpdates`). Scheitert der Export mit „Cloud signing permission error“ oder „No signing certificate“, fehlt dem API-Schlüssel das Recht auf cloud-verwaltete Verteilungszertifikate: in App Store Connect → Benutzer und Zugriff → Integrationen einen Schlüssel mit Rolle **Admin** anlegen und die drei `APPSTORE_API_*`-Secrets ersetzen. Jeder Lauf legt auf dem frischen Runner ein Entwicklungszertifikat „Created via API“ an; sammeln sich zu viele an, im Developer-Portal unter Certificates die alten widerrufen (schadet der App im Store nicht).
4. **Verträge:** Für eine kostenpflichtige App muss unter „Geschäft“ die **Paid Apps Agreement** mit Bank- und Steuerdaten aktiv sein. TestFlight geht auch vorher.
5. **App-Eintrag** (existiert): Name „Noch 'ne Runde Skat“, Primärsprache Deutsch, Englisch als weitere Lokalisierung; Kategorie **Spiele**, Unterkategorien **Karten** und **Strategie**; Preis unter „Preise und Verfügbarkeit“ (Einmalpreis), **keine** In-App-Käufe anlegen.
6. **Altersfreigabe** (Fragebogen): keine Gewalt, kein echtes Glücksspiel um Geld. Skat ist ein Kartenspiel ohne Einsatz von echtem Geld; „Simuliertes Glücksspiel“ ehrlich mit „selten/schwach“ oder „nein“ beantworten (Einsatz und Punkte sind Spielwerte). Nutzerinhalte: nur der frei wählbare Name auf der Tafel.
7. **Datenschutz-URL und Support-URL** sind Pflicht: Eine Seite auf one-more-skat.de (etwa `/datenschutz.html`) mit dem Inhalt aus dem nächsten Abschnitt, und eine Support-Adresse (Seite oder E-Mail). Gibt es noch nicht (offener Punkt).
8. **Bildschirmfotos**: iPhone 6,9" (1320 × 2868) ist Pflicht, kleinere Größen leitet App Store Connect ab. Am einfachsten aus dem Simulator (iPhone 17 Pro Max) oder mit Playwright bei 440 × 956 CSS-Pixeln und Faktor 3.
9. **TestFlight**: unter TestFlight eine interne Gruppe anlegen und sich selbst eintragen; externe Tester brauchen eine kurze Beta-Prüfung durch Apple.
10. **Hinweise für die Prüfung** (App Review Information): „Kein Konto nötig, das ganze Spiel läuft offline. Ein Konto ist freiwillig (Passkey oder Wiederherstellungscode) und gleicht den Spielstand zwischen Geräten ab; Löschen im Konto-Blatt („Konto löschen“).“ Ein Demo-Konto ist nicht nötig.

## Datenschutz-Angaben („App-Datenschutz“ in App Store Connect)

**Ohne Konto** sammelt die App nichts: Spielstand, Einstellungen und Statistik bleiben auf dem Gerät. Kein Tracking, keine Werbung, keine Analyse- oder Absturz-Dienste, keine fremden Server.

**Mit Konto** (freiwillig, ADR 0021) speichert der Konto-Dienst auf one-more-skat.de:

| Datentyp in App Store Connect | Was genau | Mit der Identität verknüpft | Zweck | Tracking |
|---|---|---|---|---|
| Kennungen → Nutzer-ID | die Konto-Id (zufällige UUID) | ja | App-Funktionalität | nein |
| Kontaktinformationen → Name | der frei gewählte Anzeigename auf der Tafel (optional, kein Klarname nötig) | ja | App-Funktionalität | nein |
| Nutzerinhalte → Spielinhalte (Gameplay Content) | der Spielstand als Text, Ergebnisse auf der Tafel des Tages und der Woche | ja | App-Funktionalität | nein |

Nicht erhoben: E-Mail, Telefonnummer, Standort, Kontakte, Kaufdaten, Diagnosedaten. IP-Adressen werden nur kurz im Arbeitsspeicher für die Begrenzung der Anfragen gezählt und nicht gespeichert. Der Passkey selbst bleibt im Schlüsselbund des Geräts; der Server kennt nur den öffentlichen Schlüssel. „Konto löschen“ im Konto-Blatt der App entfernt sofort alles auf dem Server.

In App Store Connect also: „Ja, wir erheben Daten“, die drei Typen oben, jeweils „verknüpft“, „App-Funktionalität“, „nicht für Tracking“.

## Fehlersuche

- **Passkey-Fehler in der App** („passkey“ oder sofortiger Abbruch): meist fehlt die Apple-Datei oder sie ist noch nicht im CDN (Schritt 1). In den Geräte-Logs (Console.app, Prozess `App`) steht „NativeAccount: passkey failed: …“ mit dem Grund von iOS, etwa „not associated with domain“.
- **Konto „offline“ in der App**, im Browser aber nicht: `STAMMTISCH_APP_ORIGIN` fehlt im Deploy, oder der Server ist älter als ADR 0026. Prüfen: `curl -si -X OPTIONS -H 'Origin: capacitor://localhost' -H 'Access-Control-Request-Method: PUT' https://one-more-skat.de/api/save` muss `204` mit `access-control-allow-origin: capacitor://localhost` liefern.
- **„Package.swift is out of date“** im Workflow: in `web/` `npx cap sync ios` laufen lassen und `ios/App/CapApp-SPM/Package.swift` mit committen (nach jedem Capacitor- oder Plugin-Update).
- **Neue Capacitor-Version**: in `web/package.json` alle `@capacitor/*` exakt anheben, `npm install`, `npx cap sync ios`, committen; der PR-Bau zeigt, ob Xcode zufrieden ist.

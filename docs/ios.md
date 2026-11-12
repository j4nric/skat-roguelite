# iOS-App „Noch 'ne Runde Skat“

Die App ist eine Capacitor-Hülle um genau den Web-Build aus `web/` (ADR 0026). Sie lädt das Spiel aus ihrem Paket, ist also ohne Netz spielbar; das Konto spricht `https://one-more-skat.de/api` mit einem Bearer-Token, Passkeys laufen nativ über iOS für `one-more-skat.de`.

| Was | Wert |
|---|---|
| Name im Store | Noch 'ne Runde Skat |
| Name unter dem Symbol | Noch 'ne Runde? / One more Round? (`CFBundleDisplayName`, `ios/App/App/*.lproj/InfoPlist.strings`) |
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

1. **Team-Id** als Secret `APPLE_TEAM_ID` (angelegt) oder als gleichnamige Repository-Variable; der Deploy nimmt die Variable, sonst das Secret. Daraus schreibt der Deploy `/.well-known/apple-app-site-association` (ADR 0025). Prüfen:
   `curl -s https://one-more-skat.de/.well-known/apple-app-site-association` muss `{"webcredentials":{"apps":["<TEAMID>.de.onemoreskat.app"]}}` zeigen. Apple holt die Datei über sein CDN; nach einer Änderung kann es bis zu einem Tag dauern (`curl -s https://app-site-association.cdn-apple.com/a/v1/one-more-skat.de`). Ohne sie gehen in der App keine Passkeys (der Wiederherstellungscode geht trotzdem).
2. **Bundle-Id** `de.onemoreskat.app` mit Capability **Associated Domains** (erledigt). Weitere Capabilities braucht die App nicht.
3. **Signatur.** Der API-Schlüssel braucht die Rolle **Admin** (mit App Manager scheitert der Export mit „Cloud signing permission error“). Das Team hat kein registriertes Gerät, darum archiviert der Workflow unsigniert, gibt der App im Archiv ihre Berechtigungen per Ad-hoc-Signatur (`codesign`) und lässt erst der Export mit dem cloud-verwalteten Verteilungszertifikat für den App Store signieren. Danach prüft er, dass `webcredentials:one-more-skat.de` in der signierten App steckt.
4. **Verträge:** Für eine kostenpflichtige App muss unter „Geschäft“ die **Paid Apps Agreement** mit Bank- und Steuerdaten aktiv sein. TestFlight geht auch vorher.
5. **App-Eintrag** (existiert): Name „Noch 'ne Runde Skat“, Primärsprache Deutsch, Englisch als weitere Lokalisierung; Kategorie **Spiele**, Unterkategorien **Karten** und **Strategie**; Preis unter „Preise und Verfügbarkeit“ (Einmalpreis), **keine** In-App-Käufe anlegen.
6. **Altersfreigabe** (Fragebogen): keine Gewalt, kein echtes Glücksspiel um Geld. Skat ist ein Kartenspiel ohne Einsatz von echtem Geld; „Simuliertes Glücksspiel“ ehrlich mit „selten/schwach“ oder „nein“ beantworten (Einsatz und Punkte sind Spielwerte). Nutzerinhalte: nur der frei wählbare Name auf der Tafel.
7. **Datenschutz-URL und Support-URL** sind Pflicht: Eine Seite auf one-more-skat.de (etwa `/datenschutz.html`) mit dem Inhalt aus dem nächsten Abschnitt, und eine Support-Adresse (Seite oder E-Mail). Gibt es noch nicht (offener Punkt).
8. **Bildschirmfotos und Vorschauvideo**: iPhone 6,9" (1320 × 2868) ist Pflicht, kleinere Größen leitet App Store Connect ab. Fertige Sätze (Deutsch, Englisch) liegen in `store/screenshots/`; Fotos und das Vorschauvideo (886 × 1920, 28,6 s, Vorschaubild 00:09) erzeugen die Skripte in `store/`, Anleitung, Texte der Zeilen und Formate in `store/README.md`.
9. **TestFlight**: unter TestFlight eine interne Gruppe anlegen und sich selbst eintragen; externe Tester brauchen eine kurze Beta-Prüfung durch Apple.
10. **Hinweise für die Prüfung** (App Review Information): „Kein Konto nötig, das ganze Spiel läuft offline. Ein Konto ist freiwillig (Passkey oder Wiederherstellungscode) und gleicht den Spielstand zwischen Geräten ab; Löschen im Konto-Blatt („Konto löschen“).“ Ein Demo-Konto ist nicht nötig.

## Datenschutz-Angaben („App-Datenschutz“ in App Store Connect)

**Ohne Konto** sammelt die App nichts von selbst: Spielstand, Einstellungen und Statistik bleiben auf dem Gerät. Kein Tracking, keine Werbung, keine Analyse- oder Absturz-Dienste, keine fremden Server. Einzige Ausnahme ist ein Fehlerbericht, den der Spieler selbst abschickt (unten).

**Mit Konto** (freiwillig, ADR 0021) speichert der Konto-Dienst auf one-more-skat.de:

| Datentyp in App Store Connect | Was genau | Mit der Identität verknüpft | Zweck | Tracking |
|---|---|---|---|---|
| Kennungen → Nutzer-ID | die Konto-Id (zufällige UUID) | ja | App-Funktionalität | nein |
| Kontaktinformationen → Name | der frei gewählte Anzeigename auf der Tafel (optional, kein Klarname nötig) | ja | App-Funktionalität | nein |
| Nutzerinhalte → Spielinhalte (Gameplay Content) | der Spielstand als Text, Ergebnisse auf der Tafel des Tages und der Woche | ja | App-Funktionalität | nein |

Nicht erhoben: E-Mail, Telefonnummer, Standort, Kontakte, Kaufdaten. Diagnosedaten nur in einem Fehlerbericht, den der Spieler selbst sendet (unten). IP-Adressen werden nur kurz im Arbeitsspeicher für die Begrenzung der Anfragen gezählt und nicht gespeichert. Der Passkey selbst bleibt im Schlüsselbund des Geräts; der Server kennt nur den öffentlichen Schlüssel. „Konto löschen“ im Konto-Blatt der App entfernt sofort alles auf dem Server.

**Fehlerbericht** („Fehler melden“ im Menü, ADR 0027, freiwillig und nur auf Knopfdruck). Ein Bericht enthält:

- den Text, den der Spieler schreibt,
- die letzten Schritte in der App (bis zu 200 Einträge): welche Bildschirme kamen, welche Knöpfe angetippt wurden (nur ihr interner Name, nie eingegebener Text), Fehlermeldungen der App und Tipps, nach denen sich nichts tat; dasselbe gekürzt aus der letzten Sitzung, die dafür auf dem Gerät liegen bleibt,
- Angaben zum Gerät: Plattform (Browser, installierte Web-App oder iOS-App), Version des Spiels, Browserkennung (User-Agent mit iOS-Version), Bildschirmgröße und sichere Ränder, Sprache, Tempo, Ton, Schriftgröße und ob weniger Bewegung gewünscht ist,
- wenn das Häkchen „Spielstand mitschicken“ gesetzt bleibt: den Spielstand als Text (wie „Spielstand sichern“),
- angemeldet: die Konto-Id (der Server ordnet den Bericht über die Sitzung zu, der Bericht selbst nennt kein Konto).

Nicht enthalten: Anzeigename, E-Mail, Sitzungs-Token, Passkey-Daten, Wiederherstellungs- oder Umzugs-Code, Standort. Der Server speichert den Bericht mit Zeit, Plattform, Version und einer kurzen Nummer (`B-7K3Q`), liest ihn nie aus und löscht ihn nach 90 Tagen, mit dem Konto sofort. Ohne Netz wartet ein Bericht auf dem Gerät (höchstens zwei) und geht beim nächsten Online-Sein raus.

Text für die Datenschutz-Seite: „Wenn du im Menü ‚Fehler melden‘ wählst und auf ‚Senden‘ tippst, schickt die App deinen Text, die letzten Schritte in der App (Bildschirme, angetippte Knöpfe, Fehlermeldungen), Angaben zu Gerät und Einstellungen (Plattform, Spielversion, Browserkennung, Bildschirmgröße, Sprache, Tempo) und, wenn du es nicht abwählst, deinen Spielstand an one-more-skat.de. Bist du angemeldet, wird der Bericht deinem Konto zugeordnet. Wir nutzen ihn nur, um den Fehler zu finden, und löschen ihn nach 90 Tagen.“

In App Store Connect also: „Ja, wir erheben Daten“, die drei Typen oben, jeweils „verknüpft“, „App-Funktionalität“, „nicht für Tracking“. Dazu für den Fehlerbericht **Diagnose → Sonstige Diagnosedaten** und **Nutzerinhalte → Kundensupport** (der Text): jeweils „App-Funktionalität“, „nicht für Tracking“ und „verknüpft“ (angemeldet ist der Bericht dem Konto zugeordnet).

## Fehlersuche

- **Passkey-Fehler in der App** („passkey“ oder sofortiger Abbruch): meist fehlt die Apple-Datei oder sie ist noch nicht im CDN (Schritt 1). In den Geräte-Logs (Console.app, Prozess `App`) steht „NativeAccount: passkey failed: …“ mit dem Grund von iOS, etwa „not associated with domain“.
- **Konto „offline“ in der App**, im Browser aber nicht: `STAMMTISCH_APP_ORIGIN` fehlt im Deploy, oder der Server ist älter als ADR 0026. Prüfen: `curl -si -X OPTIONS -H 'Origin: capacitor://localhost' -H 'Access-Control-Request-Method: PUT' https://one-more-skat.de/api/save` muss `204` mit `access-control-allow-origin: capacitor://localhost` liefern.
- **„Package.swift is out of date“** im Workflow: in `web/` `npx cap sync ios` laufen lassen und `ios/App/CapApp-SPM/Package.swift` mit committen (nach jedem Capacitor- oder Plugin-Update).
- **Neue Capacitor-Version**: in `web/package.json` alle `@capacitor/*` exakt anheben, `npm install`, `npx cap sync ios`, committen; der PR-Bau zeigt, ob Xcode zufrieden ist.

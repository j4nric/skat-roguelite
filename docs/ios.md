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
| Capacitor | 8.6.0 mit Swift Package Manager, Plugins `@capacitor/haptics`, `@capacitor/status-bar` und die eigenen `NativeAccount` und `Store` |
| Preis | kostenlos; Demo = Stadtteil 1 mit Boss, Vollversion als nicht verbrauchbarer In-App-Kauf `de.onemoreskat.app.full` (ADR 0028) |

## Aufbau

| Pfad | Inhalt |
|---|---|
| `web/capacitor.config.json` | App-Id, Name, `webDir: dist`, Pfad `../ios`, Hintergrund, Statusleiste |
| `ios/App/App.xcodeproj` | Xcode-Projekt (Ziel `App`, Schema `App`) |
| `ios/App/CapApp-SPM/Package.swift` | Swift-Pakete; schreibt `npx cap sync ios`, nie von Hand |
| `ios/App/App/AppViewController.swift` | die Web-Ansicht, meldet das eigene Plugin an |
| `ios/App/App/AppDelegate.swift`, `SceneDelegate.swift` | Audio-Sitzung: `.playback` mit `.mixWithOthers` (Klänge auch bei Stummschalter, fremde Musik läuft weiter) |
| `ios/App/App/NativeAccountPlugin.swift` | Passkeys (`ASAuthorization`) und Sitzungs-Token im Schlüsselbund |
| `ios/App/App/StorePlugin.swift` | Vollversion über StoreKit 2: Produkt und Preis, Kauf, Wiederherstellen, Berechtigung, `Transaction.updates` (ADR 0028) |
| `ios/App/App/Products.storekit` | StoreKit-Konfiguration mit dem Produkt, nur für „Run“ in Xcode (im Schema `App`), nicht im Paket |
| `ios/App/App/App.entitlements` | Associated Domains `webcredentials:one-more-skat.de` |
| `ios/App/App/Info.plist` | Hochformat, Statusleiste, Sprachen, Verschlüsselung |
| `ios/App/App/Assets.xcassets` | Icon und Startbild aus `web/scripts/make-ios-assets.py` |
| `web/src/platform.ts`, `web/src/native.ts` | Erkennen der App und die JavaScript-Seite des Plugins |
| `web/src/store.ts`, `web/src/native-store.ts`, `web/src/app/fullversion.ts`, `web/src/ui/fullversion.ts` | Stand der Vollversion, die Grenze der Demo und die Kauf-Seite (ADR 0028) |

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
2. **Bundle-Id** `de.onemoreskat.app` mit Capability **Associated Domains** (erledigt). **In-App Purchase** ist bei jeder App-Id schon eingeschaltet (unter Certificates, Identifiers & Profiles → Identifiers → `de.onemoreskat.app` nur nachsehen, dass der Haken gesetzt ist); dafür gibt es keinen Eintrag in `App.entitlements`. Weitere Capabilities braucht die App nicht.
3. **Signatur.** Der API-Schlüssel braucht die Rolle **Admin** (mit App Manager scheitert der Export mit „Cloud signing permission error“). Das Team hat kein registriertes Gerät, darum archiviert der Workflow unsigniert, gibt der App im Archiv ihre Berechtigungen per Ad-hoc-Signatur (`codesign`) und lässt erst der Export mit dem cloud-verwalteten Verteilungszertifikat für den App Store signieren. Danach prüft er, dass `webcredentials:one-more-skat.de` in der signierten App steckt.
4. **Verträge:** Für In-App-Käufe muss unter „Geschäft“ die **Paid Apps Agreement** mit Bank- und Steuerdaten aktiv sein (auch wenn die App selbst kostenlos ist). Ohne sie liefert StoreKit kein Produkt, die Kauf-Seite zeigt dann „Der Preis lässt sich gerade nicht laden“. TestFlight geht auch vorher.
5. **App-Eintrag** (existiert): Name „Noch 'ne Runde Skat“, Primärsprache Deutsch, Englisch als weitere Lokalisierung; Kategorie **Spiele**, Unterkategorien **Karten** und **Strategie**; Preis unter „Preise und Verfügbarkeit“ **kostenlos** (ADR 0028, ersetzt den Einmalpreis); die Vollversion als In-App-Kauf (Abschnitt „Vollversion“ unten).
6. **Altersfreigabe** (Fragebogen): keine Gewalt, kein echtes Glücksspiel um Geld. Skat ist ein Kartenspiel ohne Einsatz von echtem Geld; „Simuliertes Glücksspiel“ ehrlich mit „selten/schwach“ oder „nein“ beantworten (Einsatz und Punkte sind Spielwerte). Nutzerinhalte: nur der frei wählbare Name auf der Tafel.
7. **URLs** (App-Informationen bzw. Version): **Datenschutz-URL** `https://one-more-skat.de/datenschutz`, **Support-URL** `https://one-more-skat.de/support`, **Marketing-URL** (optional) `https://one-more-skat.de`. Für die englische Lokalisierung gehen auch `https://one-more-skat.de/en/privacy` und `https://one-more-skat.de/en/support`; verbindlich ist die deutsche Datenschutzerklärung. Die Seiten (mit Impressum `/impressum`) sind statisches HTML aus `web/` (`impressum.html`, `datenschutz.html`, `support.html`, `en/*.html`), Caddy liefert sie ohne `.html` aus. In der App öffnen die Links unten auf der Startseite und im Menü Safari.
8. **Bildschirmfotos**: iPhone 6,9" (1320 × 2868) ist Pflicht, kleinere Größen leitet App Store Connect ab. Am einfachsten aus dem Simulator (iPhone 17 Pro Max) oder mit Playwright bei 440 × 956 CSS-Pixeln und Faktor 3.
9. **TestFlight**: unter TestFlight eine interne Gruppe anlegen und sich selbst eintragen; externe Tester brauchen eine kurze Beta-Prüfung durch Apple.
10. **Hinweise für die Prüfung** (App Review Information): „Kein Konto nötig, das ganze Spiel läuft offline. Ein Konto ist freiwillig (Passkey oder Wiederherstellungscode) und gleicht den Spielstand zwischen Geräten ab; Löschen im Konto-Blatt („Konto löschen“). Die App ist kostenlos; der erste Stadtteil mit seinem Boss ist die Demo. Die Kauf-Seite der Vollversion (In-App-Kauf de.onemoreskat.app.full) erscheint nach dem Boss des ersten Stadtteils statt des Weges in Stadtteil 2, außerdem jederzeit über die Zeile ‚Demo · Vollversion freischalten‘ auf der Startseite und im Menü (☰ → ‚Vollversion freischalten‘). ‚Käufe wiederherstellen‘ steht auf der Kauf-Seite und im Menü.“ Ein Demo-Konto ist nicht nötig.

## Vollversion (In-App-Kauf) in App Store Connect

Entscheidung des Nutzers vom 12.11.2026, ADR 0028. Einmalig, in dieser Reihenfolge:

1. **Paid Apps Agreement** aktiv (Schritt 4 oben). Optional, aber empfohlen: **App Store Small Business Program** beantragen (developer.apple.com → Programs → App Store Small Business Program, oder App Store Connect → Verträge). Damit nimmt Apple 15 % statt 30 % Provision, solange der Erlös unter 1 Mio. US-$ im Jahr bleibt. Gilt ab dem Monat nach der Zusage.
2. **App-Preis kostenlos:** App → „Preise und Verfügbarkeit“ → Preis → **0,00 € (kostenlos)**, alle Länder.
3. **In-App-Kauf anlegen:** App → „Monetarisierung“ → **In-App-Käufe** → „+“:
   - Typ **Nicht-verbrauchbar** (Non-Consumable),
   - Referenzname `Vollversion`,
   - Produkt-ID **`de.onemoreskat.app.full`** (genau so, sie steht im Code; nach dem Anlegen nicht mehr änderbar).
4. **Lokalisierung** (unter dem Kauf, „App Store-Lokalisierung“):
   - Deutsch: Anzeigename **Vollversion**, Beschreibung **Alle Stadtteile, alle Bosse, die lange Nacht.**
   - Englisch (UK und US): Anzeigename **Full version**, Beschreibung **Every district, every boss, the long night.**
   (Anzeigename höchstens 35 Zeichen, Beschreibung höchstens 55. Die App zeigt den Namen nicht selbst, nur den Preis; Apple zeigt beides im Kaufdialog.)
5. **Preis:** „Preisplan“ → Basisland **Deutschland**, **0,99 €**; die anderen Länder rechnet Apple um. Der spätere Wechsel auf **2,99 €**: im selben Preisplan „Preisänderung planen“ mit Startdatum, neuer Preis 2,99 €. Der Code muss dafür nicht geändert werden, die App zeigt immer den Preis aus StoreKit.
6. **Verfügbarkeit:** alle Länder, in denen die App erscheint.
7. **Familienfreigabe:** einschalten („Familienfreigabe“ im Kauf). Ein Käufer teilt die Vollversion dann mit seiner Familie; die App erkennt geteilte Käufe über dieselbe Prüfung. Achtung: einmal eingeschaltet lässt sie sich nicht wieder abschalten.
8. **Prüfungsinformationen** des Kaufs: ein **Bildschirmfoto der Kauf-Seite** (Pflicht, z. B. `unlock-wall-de.png` aus dem Smoke-Test, auf 1320 × 2868 oder im Simulator aufgenommen) und als Notiz: „Erscheint nach dem Boss des ersten Stadtteils statt des Weges in Stadtteil 2, und über ‚Demo · Vollversion freischalten‘ auf der Startseite.“
9. **Mit der App einreichen:** Der erste In-App-Kauf muss zusammen mit einer neuen App-Version zur Prüfung. In der Version unter „In-App-Käufe und Abonnements“ den Kauf auswählen, dann die Version einreichen. Status des Kaufs vorher: „Bereit zur Einreichung“.
10. **App-Store-Beschreibung** ergänzen (Abschnitt unten).

### Testen

- **In Xcode (ohne App Store Connect):** Das Schema `App` nutzt beim Starten `ios/App/App/Products.storekit` (Product → Scheme → Edit Scheme → Run → Options → StoreKit Configuration). Käufe sind dann lokal und kostenlos; Xcode → Debug → StoreKit → **Manage Transactions** zeigt sie, dort lassen sich Käufe löschen (wieder Demo), **erstatten** (die App entzieht die Vollversion) und Kaufanfragen bestätigen (in der Datei „Ask to Buy“ einschalten). Der Release-Bau und das Archiv nutzen die Datei nie.
- **TestFlight mit Sandbox:** In App Store Connect → Benutzer und Zugriff → **Sandbox** → Testaccounts einen **Sandbox-Tester** anlegen (eigene, unbenutzte E-Mail-Adresse). Auf dem iPhone unter Einstellungen → App Store → **Sandbox-Account** damit anmelden (nicht das normale Apple-Konto abmelden). Käufe in TestFlight-Builds kosten nichts, auch ohne Sandbox-Account (TestFlight nutzt die Sandbox mit der eigenen Apple-ID). Wiederholen: in den Sandbox-Einstellungen „Kaufverlauf löschen“.
- Ablauf zum Prüfen: neue App, Abend starten, Stadtteil 1 mit Boss gewinnen → Relikt wählen → Kauf-Seite mit Preis → kaufen → „Prost!“ → derselbe Run geht in Stadtteil 2 weiter. Dann App löschen und neu installieren → Startseite zeigt „Demo · …“ kurz, bis StoreKit antwortet, oder → Menü → „Käufe wiederherstellen“.

### App-Store-Beschreibung

Erste Zeile der Beschreibung (DE): **„Kostenlos spielbar: Der erste Stadtteil mit seinem Boss ist frei. Die Vollversion mit allen Stadtteilen, Bossen und der langen Nacht gibt es per In-App-Kauf, einmal und ohne Abo.“**

Englisch: **“Free to play: the first district with its boss is free. The full version with every district, every boss and the long night is a one-time In-App Purchase, no subscription.”**

App Store Connect zeigt „In-App-Käufe“ dann von selbst neben dem Preis „Gratis“. Keine Hinweise auf andere Bezugswege (etwa die kostenlose Web-Version) in App, Beschreibung oder Kauf-Seite: das verbietet Richtlinie 3.1.1 bzw. 3.1.3.

## Datenschutz-Angaben („App-Datenschutz“ in App Store Connect)

**Ohne Konto** sammelt die App nichts von selbst: Spielstand, Einstellungen und Statistik bleiben auf dem Gerät. Kein Tracking, keine Werbung, keine Analyse- oder Absturz-Dienste, keine fremden Server. Einzige Ausnahme ist ein Fehlerbericht, den der Spieler selbst abschickt (unten).

**Mit Konto** (freiwillig, ADR 0021) speichert der Konto-Dienst auf one-more-skat.de:

| Datentyp in App Store Connect | Was genau | Mit der Identität verknüpft | Zweck | Tracking |
|---|---|---|---|---|
| Kennungen → Nutzer-ID | die Konto-Id (zufällige UUID) | ja | App-Funktionalität | nein |
| Kontaktinformationen → Name | der frei gewählte Anzeigename auf der Tafel (optional, kein Klarname nötig) | ja | App-Funktionalität | nein |
| Nutzerinhalte → Spielinhalte (Gameplay Content) | der Spielstand als Text, Ergebnisse auf der Tafel des Tages und der Woche | ja | App-Funktionalität | nein |

Nicht erhoben: E-Mail, Telefonnummer, Standort, Kontakte. Den Kauf der Vollversion wickelt Apple ab; die App prüft ihn nur auf dem Gerät (StoreKit) und schickt nichts davon an unseren Server, außer im Fehlerbericht den Stand „Demo“ oder „Vollversion“ (unten). Diagnosedaten nur in einem Fehlerbericht, den der Spieler selbst sendet (unten). IP-Adressen werden nur kurz im Arbeitsspeicher für die Begrenzung der Anfragen gezählt und nicht gespeichert. Der Passkey selbst bleibt im Schlüsselbund des Geräts; der Server kennt nur den öffentlichen Schlüssel. „Konto löschen“ im Konto-Blatt der App entfernt sofort alles auf dem Server.

**Fehlerbericht** („Fehler melden“ im Menü, ADR 0027, freiwillig und nur auf Knopfdruck). Ein Bericht enthält:

- den Text, den der Spieler schreibt,
- die letzten Schritte in der App (bis zu 200 Einträge): welche Bildschirme kamen, welche Knöpfe angetippt wurden (nur ihr interner Name, nie eingegebener Text), Fehlermeldungen der App und Tipps, nach denen sich nichts tat; dasselbe gekürzt aus der letzten Sitzung, die dafür auf dem Gerät liegen bleibt,
- Angaben zum Gerät: Plattform (Browser, installierte Web-App oder iOS-App), ob die Vollversion freigeschaltet ist (`demo`/`full`, ohne Beleg und ohne Transaktions-Id), Version des Spiels, Browserkennung (User-Agent mit iOS-Version), Bildschirmgröße und sichere Ränder, Sprache, Tempo, Ton, Schriftgröße und ob weniger Bewegung gewünscht ist,
- wenn das Häkchen „Spielstand mitschicken“ gesetzt bleibt: den Spielstand als Text (wie „Spielstand sichern“),
- angemeldet: die Konto-Id (der Server ordnet den Bericht über die Sitzung zu, der Bericht selbst nennt kein Konto).

Nicht enthalten: Anzeigename, E-Mail, Sitzungs-Token, Passkey-Daten, Wiederherstellungs- oder Umzugs-Code, Standort. Der Server speichert den Bericht mit Zeit, Plattform, Version und einer kurzen Nummer (`B-7K3Q`), liest ihn nie aus und löscht ihn nach 90 Tagen, mit dem Konto sofort. Ohne Netz wartet ein Bericht auf dem Gerät (höchstens zwei) und geht beim nächsten Online-Sein raus.

Text für die Datenschutz-Seite: „Wenn du im Menü ‚Fehler melden‘ wählst und auf ‚Senden‘ tippst, schickt die App deinen Text, die letzten Schritte in der App (Bildschirme, angetippte Knöpfe, Fehlermeldungen), Angaben zu Gerät und Einstellungen (Plattform, Spielversion, Browserkennung, Bildschirmgröße, Sprache, Tempo) und, wenn du es nicht abwählst, deinen Spielstand an one-more-skat.de. Bist du angemeldet, wird der Bericht deinem Konto zugeordnet. Wir nutzen ihn nur, um den Fehler zu finden, und löschen ihn nach 90 Tagen.“

In App Store Connect also: „Ja, wir erheben Daten“, die drei Typen oben, jeweils „verknüpft“, „App-Funktionalität“, „nicht für Tracking“. Dazu für den Fehlerbericht **Diagnose → Sonstige Diagnosedaten** und **Nutzerinhalte → Kundensupport** (der Text): jeweils „App-Funktionalität“, „nicht für Tracking“ und „verknüpft“ (angemeldet ist der Bericht dem Konto zugeordnet). Zur Sicherheit dazu **Käufe → Kaufverlauf** (ebenso „App-Funktionalität“, „nicht für Tracking“, „verknüpft“), weil ein Fehlerbericht nennt, ob die Vollversion freigeschaltet ist (ADR 0028).

## Fehlersuche

- **Kauf-Seite ohne Preis** („Der Preis lässt sich gerade nicht laden“), obwohl online: Paid Apps Agreement nicht aktiv, der In-App-Kauf noch nicht „Bereit zur Einreichung“ oder in einem anderen Land nicht verfügbar, oder die Produkt-ID weicht ab (`de.onemoreskat.app.full`). In den Geräte-Logs steht „Store: products failed: …“. In Xcode ohne Sandbox: StoreKit-Konfiguration im Schema prüfen.
- **Vollversion nach Neuinstallation weg:** Menü → „Käufe wiederherstellen“ (`AppStore.sync()`, fragt nach der Apple-ID). Die App fragt StoreKit sonst bei jedem Start von selbst.

- **Passkey-Fehler in der App** („passkey“ oder sofortiger Abbruch): meist fehlt die Apple-Datei oder sie ist noch nicht im CDN (Schritt 1). In den Geräte-Logs (Console.app, Prozess `App`) steht „NativeAccount: passkey failed: …“ mit dem Grund von iOS, etwa „not associated with domain“.
- **Konto „offline“ in der App**, im Browser aber nicht: `STAMMTISCH_APP_ORIGIN` fehlt im Deploy, oder der Server ist älter als ADR 0026. Prüfen: `curl -si -X OPTIONS -H 'Origin: capacitor://localhost' -H 'Access-Control-Request-Method: PUT' https://one-more-skat.de/api/save` muss `204` mit `access-control-allow-origin: capacitor://localhost` liefern.
- **Keine Klänge in der App**, im Browser aber schon (Spieltest 12.11.2026): Capacitors `capacitor://`-Handler beantwortet Mediendateien (mp3) mit einer `URLResponse` ohne HTTP-Status, `fetch` meldet dann Status 0 und `ok: false`, obwohl die Daten vollständig sind. `web/src/audio.ts` nimmt Status 0 mit Inhalt deshalb als geladen. Die Musik ist davon nicht betroffen, sie wird im Browser erzeugt. Smoke: `SMOKE_ONLY=appaudio` spielt die App-Rolle mit diesem Verhalten nach. Die Audio-Sitzung setzt die App selbst (`.playback`, gemischt mit anderen Apps); `navigator.audioSession` setzt die Seite nur im Browser.
- **„Package.swift is out of date“** im Workflow: in `web/` `npx cap sync ios` laufen lassen und `ios/App/CapApp-SPM/Package.swift` mit committen (nach jedem Capacitor- oder Plugin-Update).
- **Neue Capacitor-Version**: in `web/package.json` alle `@capacitor/*` exakt anheben, `npm install`, `npx cap sync ios`, committen; der PR-Bau zeigt, ob Xcode zufrieden ist.

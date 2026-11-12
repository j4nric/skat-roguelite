# 0026: iOS-App als Capacitor-Hülle, Konto über Bearer-Token und native Passkeys

Datum: 2026-11-12 · Status: Angenommen · Ergänzt 0009, 0021 und 0025

## Kontext

Das Spiel soll als „Noch 'ne Runde Skat“ in den App Store (Bundle-Id `de.onemoreskat.app`, Einmalpreis, keine Käufe in der App, keine Werbung, kein Tracking). Die Web-App aus `web/` ist fertig und läuft mit dem Kern als WebAssembly (ADR 0009). Zwei Fragen standen im Raum:

1. **Wie kommt die Web-App in eine iOS-App**, ohne Spiellogik doppelt zu schreiben und ohne gegen die Richtlinie 4.2 („Minimum Functionality“: keine bloße verpackte Website) zu verstoßen?
2. **Wie funktionieren Konto und Passkeys in der App?** Die Web-App ruft `/api` relativ auf, ihr Konto hängt an einem Cookie (`HttpOnly; Secure; SameSite=Lax; Path=/api`), und WebAuthn bindet Passkeys an die RP-Id = Host (ADR 0021, 0025). In der App ist der Ursprung der Seite aber `capacitor://localhost`: Ein relatives `/api` geht ins App-Paket, für `https://one-more-skat.de` ist die App ein fremder Ursprung (cross-site), WKWebView schickt das Cookie dorthin nicht mit (Intelligent Tracking Prevention, SameSite), und WebAuthn verweigert eine RP-Id, die nicht zum Ursprung der Seite passt.

## Entscheidung

**Hülle.** Capacitor 8 (8.6.0, Swift Package Manager statt CocoaPods), das Xcode-Projekt liegt in `ios/`, die Capacitor-Konfiguration in `web/capacitor.config.json`. Die App bündelt genau den Vite-Build (`web/dist`) und lädt ihn lokal: spielbar ohne Netz, der Kern läuft wie im Browser im Web Worker. Kein Fernladen der Website. Nativer Code macht nur Transport und Darstellung; die Spiellogik bleibt im Kern. Kein Service Worker in der App (die Dateien sind im Paket, neue Versionen kommen über den App Store). iPhone, Hochformat, ab iOS 16 (Passkeys). Kein iPad: Das Layout ist fürs Handy hochkant gebaut, ein iPad bräuchte alle Ausrichtungen, eigene Bildschirmfotos und eigene Prüfung; iPhone-Apps laufen auf dem iPad trotzdem im Kompatibilitätsmodus.

**Konto in der App: derselbe Konto-Dienst, ein Bearer-Token statt des Cookies.**
- Die App ruft `https://one-more-skat.de/api` auf (`credentials: 'omit'`).
- Der Server kennt die Herkunft der App als eigene Liste `STAMMTISCH_APP_ORIGIN` (`capacitor://localhost`, exakter Vergleich). Für sie und nur für sie: CORS-Kopfzeilen (ohne `Allow-Credentials`) und eine Antwort auf den Preflight; ein mitgeschicktes Cookie wird verworfen; ein `Set-Cookie` der Sitzung wird zur Kopfzeile `X-Session-Token` (leer heißt abgemeldet). Die App schickt das Token als `Authorization: Bearer …` zurück. Das Token ist dasselbe Zufallstoken wie im Cookie, gespeichert wird weiter nur sein SHA-256; Laufzeit und Verlängerung bleiben gleich.
- Ändernde Anfragen der App kommen immer mit `Sec-Fetch-Site: cross-site`. Sie werden allein über ihr `Origin` zugelassen: Ohne Cookie gibt es nichts, worauf eine gefälschte Anfrage reiten könnte, und das Bearer-Token hat keine andere Seite.
- Die App hält das Token im Schlüsselbund (`kSecAttrAccessibleAfterFirstUnlockThisDeviceOnly`, nur dieses Gerät, keine iCloud-Synchronisierung).

**Passkeys in der App: nativ über iOS, für die RP-Id `one-more-skat.de`.**
- Ein kleines eigenes Plugin (`ios/App/App/NativeAccountPlugin.swift`, JavaScript-Seite `web/src/native.ts`) nimmt die WebAuthn-Optionen des Servers so, wie sie kommen, ruft `ASAuthorizationPlatformPublicKeyCredentialProvider` auf und gibt die Antwort im JSON zurück, das der Server schon von Browsern liest.
- Die App trägt `webcredentials:one-more-skat.de` in den Associated Domains; die Seite nennt die App in `/.well-known/apple-app-site-association` (ADR 0025, `deploy/aasa.sh`). iOS schreibt `https://one-more-skat.de` als Herkunft in `clientDataJSON`, die Prüfung im Server läuft also unverändert über die Haupt-RP.
- Damit sind es **dieselben Passkeys** wie auf one-more-skat.de im Browser: ein Passkey aus dem iCloud-Schlüsselbund gilt in Safari und in der App.

**Feinheiten.** Icon und Startbild entstehen aus dem Pixel-Icon der PWA (`web/scripts/make-ios-assets.py`, ganzzahlig skaliert, ohne Alphakanal). Dunkler Hintergrund `#1b1e21` vom Startbild bis zur Web-Ansicht, helle Statusleiste (`@capacitor/status-bar`, nur Konfiguration). Vibration über `@capacitor/haptics` (Taptic Engine statt `navigator.vibrate`, das iOS nicht kennt). Sprache wie im Browser aus der Systemsprache (`CFBundleLocalizations` de und en). `ITSAppUsesNonExemptEncryption = NO`. Name unter dem Symbol „Runde Skat“ (der volle Name würde abgeschnitten).

**Bau.** `.github/workflows/ios.yml`: bei Änderungen an App oder Web ein Bau ohne Signatur als Prüfung; von Hand oder mit Tag `ios-v*` Archiv mit automatischer Signatur über den App-Store-Connect-API-Schlüssel und Upload zu TestFlight (`docs/ios.md`).

Der Server bleibt, was 0021 sagt: Er speichert nur, kennt `skat_core` nicht und liest keinen Spielstand.

## Alternativen

- **Die Website in einer WebView fernladen** (`server.url`): kein Offline-Spiel, und Apple lehnt bloß verpackte Websites nach 4.2 ab.
- **Eigene native Oberfläche (SwiftUI) mit dem Kern als Bibliothek:** zweite Oberfläche, doppelter Aufwand, widerspricht 0009.
- **Cookie behalten**, etwa mit `SameSite=None` oder über `CapacitorHttp`/`CapacitorCookies` (native Anfragen mit dem Cookie-Speicher der App): `SameSite=None` würde das Cookie auch fremden Seiten im Browser öffnen und die Härtung aus 0021 aufweichen. `CapacitorHttp` ersetzt `fetch` und `XMLHttpRequest` für die ganze Seite, auch für den Kern und die Medien; Fehler dort träfen das Spiel. Ein Token für genau einen Ursprung ist kleiner und prüfbar.
- **Eigene Domain als Ursprung der App** (`server.hostname = one-more-skat.de` mit `iosScheme`): Capacitor lässt `https` als eigenes Schema auf iOS nicht zu, und eine App, die sich als die Website ausgibt, wäre für Cookies und WebAuthn trotzdem nicht dieselbe Herkunft.
- **WebAuthn direkt in der WKWebView:** geht nur für Seiten, deren Ursprung zur RP-Id passt (oder mit der Browser-Berechtigung von Apple, die eine Spiele-App nicht bekommt).
- **Fremdes Passkey-Plugin aus npm:** mehrere kleine Projekte mit wenigen Pflegern; die nötige Logik sind rund 200 Zeilen Swift, die wir lesen und prüfen können. Eigenes Plugin, solange es so klein bleibt.
- **Token im `localStorage` oder in `@capacitor/preferences`:** einfacher, aber iOS darf den Speicher einer WebView bei Platzmangel leeren. Im Schlüsselbund bleibt die Anmeldung erhalten, und der Spielstand kommt dann aus dem Konto zurück.
- **Eigene RP-Id oder eigener Konto-Dienst für die App:** getrennte Passkeys je Plattform, kein Vorteil.
- **fastlane:** bequem, aber Ruby und eine weitere Werkzeugkette; `xcodebuild` mit API-Schlüssel und `altool` reichen.

## Folgen

- Deploy: `STAMMTISCH_APP_ORIGIN=capacitor://localhost` steht in `deploy/docker-compose.yml`. Ohne die Variable lehnt der Server ändernde Anfragen der App mit 403 `origin` ab; die App läuft dann ohne Konto wie offline.
- Passkeys in der App gehen erst, wenn die Apple-Datei mit der Team-Id ausgeliefert wird (Repository-Variable `APPLE_TEAM_ID`, ADR 0025) und Apple sie über sein CDN geholt hat. Bis dahin meldet iOS einen Fehler, und die App bietet den Wiederherstellungscode an.
- Konto, Spielstand und Tafel sind in Browser und App dieselben; der Abgleich (`web/src/sync.ts`) bleibt unverändert.
- Der Spielstand ohne Konto liegt in der App im `localStorage` der WebView, also getrennt vom Browser. Umziehen geht wie bisher über Konto oder gesicherten Text.
- Neue native Fähigkeiten brauchen dieselbe Abwägung: Plugin nur für Darstellung und Transport, nie für Regeln.
- Eine neue Capacitor-Version wird in `web/package.json` (exakt) und mit `npx cap sync ios` in `ios/App/CapApp-SPM/Package.swift` zusammen eingetragen; der Workflow bricht ab, wenn die Datei nicht passt.

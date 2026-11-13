# 0028: iOS-App kostenlos mit Demo, Vollversion per In-App-Kauf

Datum: 2026-11-12 · Status: Angenommen · Ersetzt den Preis-Teil von 0026 („Einmalpreis, keine Käufe in der App“), ergänzt 0026 und 0027

## Kontext

ADR 0026 ging von einer App mit Einmalpreis aus (Cockpit q41). Eine bezahlte App kann vor dem Kauf niemand ausprobieren, und ein Roguelike lebt davon, dass man den ersten Abend anspielt. Der Nutzer hat am 12.11.2026 neu entschieden (Cockpit q48, ersetzt q41):

- Die App ist im Store **kostenlos**. **Demo ist der erste Stadtteil mit seinem Boss**, voll spielbar. Danach kommt eine Kauf-Seite; ohne Kauf geht es nicht in Stadtteil 2.
- **Vollversion** ist ein **nicht verbrauchbarer** In-App-Kauf mit der Produkt-Id `de.onemoreskat.app.full`, Startpreis 0,99 €, später 2,99 €. Der Preis kommt immer aus StoreKit, nie aus dem Code.
- Die **Web-Version** (Browser und PWA) bleibt vollständig und kostenlos. Die Sperre gilt nur in der iOS-App.

## Entscheidung

**StoreKit 2 nativ, ohne Drittanbieter und ohne eigenen Server.** Ein kleines eigenes Capacitor-Plugin (`ios/App/App/StorePlugin.swift`, angemeldet wie `NativeAccountPlugin` in `AppViewController`, JavaScript-Seite `web/src/native-store.ts`) bietet `products()` (lokalisierter Name und Preis), `purchase()` (`purchased`, `cancelled`, `pending`), `restore()` (`AppStore.sync()`) und `entitlement()` (eine verifizierte, nicht widerrufene Transaktion in `Transaction.currentEntitlements`). Ab dem Start hört es auf `Transaction.updates` (Kauf auf einem anderen Gerät, Familienfreigabe, bestätigte Kaufanfrage, Rückerstattung) und meldet `entitlementChanged`. Apple signiert jede Transaktion, StoreKit prüft die Signatur auf dem Gerät und hält die Berechtigungen im Cache: Die Prüfung geht offline. Ein eigener Server würde Belege nur noch einmal prüfen und bekäme Kaufdaten, die er nicht braucht.

**Zustand im Web** (`web/src/store.ts`): nur in der App aktiv (`isNativeApp()`), im Browser immer „voll“. Ein Cache in `localStorage` (`stammtisch-full`, ohne Punkt, also kein Spielstand-Schlüssel: nie in Spielstand-Text, Upload zum Konto oder dem Spielstand eines Fehlerberichts) macht den Start schnell; die Antwort von StoreKit ersetzt ihn, sobald sie kommt, auch wenn sie die Vollversion wieder entzieht.

**Die Grenze liegt nur in der Steuerung der App** (`web/src/app/fullversion.ts`), der Kern bleibt unverändert. Ein Run steht an der Grenze, wenn er in Stadtteil 2 oder weiter ist oder zwischen zwei Kneipen schon dorthin unterwegs ist (`Route`, Tresen, Begegnung mit der nächsten Kneipe in Stadtteil 2). Die Relikt-Wahl des Bosses (`BossReward`) gehört noch zu Stadtteil 1. An der Grenze:

- erscheint statt des Stadtplans die **Kauf-Seite** (nicht schließbar; „Später“ führt zur Startseite),
- schickt die App keine Aktion an den Kern, die den Run weiterbringt (ausgenommen, was eine gewonnene Kneipe von Stadtteil 1 noch zu nehmen lässt: Schnaps-Belohnung, Kartenmaler); ein gegnerischer Zug wird nach dem Laden nicht fortgesetzt,
- bleibt der Run gespeichert, wie er ist. Nach Kauf oder Wiederherstellen schließt die Seite nach einer kurzen Freude und der Run geht genau dort weiter.

**Sonderfälle:**

| Fall | Lösung |
|---|---|
| Tages-Run, Wochen-Herausforderung, geteilter oder wiederholter Seed | dieselbe Grenze; bis zum Ende von Stadtteil 1 spielbar, ein verlorener Run zählt wie immer |
| Einführung (`view.tutorial`) | nie gesperrt (eine geführte Kneipe) |
| Lange Nacht, Einsatz-Leiter | brauchen einen gewonnenen Run, also die Vollversion; nach einer Rückerstattung gilt dieselbe Grenze |
| Fortsetzen eines alten Spielstands, der schon weiter ist | lädt, bleibt gespeichert, zeigt die Kauf-Seite statt weiterzuspielen |
| Spielstand-Text oder Konto aus dem Browser | wie ein alter Spielstand: nichts geht verloren, nichts wird umgangen (der Cache reist nicht mit) |
| Rückerstattung | `Transaction.updates` entzieht die Vollversion; beim nächsten Schritt über die Grenze kommt die Kauf-Seite wieder |
| Kaufanfrage (Ask to Buy) | „wartet auf Zustimmung“; die spätere Zustimmung kommt über `Transaction.updates` und schaltet frei |

**Oberfläche.** Die Kauf-Seite ist ein Zettel auf der Papier-Ebene (DESIGN 16.6) mit dem Wirt: „Noch 'ne Runde?“, was die Vollversion bringt (alle drei Stadtteile mit ihren Bossen bis zum Alten, die lange Nacht, Tages-Run und Wochen-Herausforderung bis zum Ende, alle sieben Figuren und die Einsatz-Leiter), „Einmal kaufen, für immer: kein Abo, keine Werbung“, der Knopf mit dem Preis aus StoreKit, „Käufe wiederherstellen“ und „Später“. Zustände: lädt, kein Preis (offline, mit „Preis nochmal laden“), Kauf läuft, abgebrochen, wartet auf Zustimmung, Fehler, Suche, nichts gefunden, freigeschaltet (Bierglas, „Prost!“). Auf der Startseite der Demo eine dezente Zeile „Demo · Vollversion freischalten“, im Menü der App „Vollversion freischalten“ (nur in der Demo) und immer „Käufe wiederherstellen“ (Pflicht von Apple). Im Browser erscheint nichts davon. Die Seite nennt keine anderen Bezugswege (App-Store-Richtlinie 3.1.1).

**Fehlerberichte** (ADR 0027) nennen den Stand (`web`, `full`, `demo`, mit „cached“, solange StoreKit nicht geantwortet hat) und im Puffer die Wechsel und das Ergebnis eines Kaufs (`purchase cancelled`), nie Beleg oder Transaktions-Id.

**Xcode.** In-App-Käufe brauchen keine eigene Berechtigung in `App.entitlements` (die Fähigkeit ist an jeder App-Id). Für lokale Tests liegt `ios/App/App/Products.storekit` mit dem Produkt im Schema (nur „Run“, nicht im Archiv und nicht im Paket).

## Alternativen

- **Bezahlte App (Einmalpreis, 0026):** niemand kann vor dem Kauf spielen; ersetzt.
- **Demo als eigene App („Lite“):** zwei Einträge, zwei Spielstände, Apple sieht so etwas ungern (4.3).
- **Zeitlich begrenzte Demo** oder **Anzahl Runs:** passt nicht zu einem Spiel mit Seeds und Runs unterschiedlicher Länge; der erste Stadtteil mit Boss ist eine klare, faire Probe.
- **Sperre im Kern** (etwa ein Flag im `RunStart`): Kauf ist kein Spielregel-Thema, der Simulator und das Web sollen ihn nie sehen; widerspricht „Spiellogik nie in wasm/ oder web/“ nicht, weil die Grenze keine Regel ändert, nur die Steuerung anhält.
- **RevenueCat oder ein anderes SDK:** fremder Dienst, Daten bei Dritten, mehr Datenschutzangaben; StoreKit 2 reicht für ein einziges Produkt.
- **Belegprüfung auf dem eigenen Server:** bringt bei einem nicht verbrauchbaren Kauf ohne Konto-Bindung nichts und widerspräche 0021 („speichert nur“).
- **Kauf ans Konto binden** (auch im Web freischalten): das Web ist ohnehin frei; Käufe gehören der Apple-ID, Wiederherstellen und Familienfreigabe regelt Apple.
- **Kauf-Seite schon vor dem Boss:** unterbricht den spannendsten Moment der Demo; die Grenze liegt nach der Boss-Belohnung.

## Folgen

- In App Store Connect: App-Preis auf **kostenlos**, In-App-Kauf anlegen, Paid Apps Agreement aktiv, Prüfung des Kaufs zusammen mit der App (Schritte in `docs/ios.md`).
- Das Web bleibt frei. Wer im Browser spielt, merkt nichts; wer einen Spielstand aus dem Web in die App holt, spielt dort bis zur Grenze.
- Der Preis steht nirgends im Code; ein Preiswechsel (0,99 € → 2,99 €) ist eine Einstellung in App Store Connect.
- Neue Inhalte für die Vollversion brauchen keine Änderung an der Grenze; was nach Stadtteil 1 kommt, ist automatisch Vollversion. Wer die Run-Struktur ändert (`districtReached`), prüft die Grenze mit `unlockPass` (`SMOKE_ONLY=unlock`).
- Ein Risiko für die Prüfung: Apple verlangt, dass „Käufe wiederherstellen“ sichtbar ist und die Demo ohne Kauf sinnvoll spielbar ist (beides erfüllt). Die Hinweise für die Prüfung nennen, wo die Kauf-Seite kommt.

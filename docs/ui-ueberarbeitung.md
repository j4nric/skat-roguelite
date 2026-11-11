# UI-Überarbeitung: Stand und offene Aufgaben

Stand: 11.11.2026, abends. Übergabe-Dokument für die weitere Arbeit an der Oberfläche.

## Ausgangslage und Entscheidungen des Nutzers

- Drei komplett neue Stilrichtungen (Kreide & Filz, Comic-Kneipe, Neon-Nacht) sind **verworfen** (Cockpit q43). Die Leinwand dazu bleibt nur als Archiv: https://claude.ai/artifact/L1e3QdkxprPcgZEyzsqpon
- **Grunddesign und Pixelgrafik bleiben**: Palette „schiefer“, Pixelschrift, vorhandene Karten und Porträts. Verbessert werden die Schwachstellen des heutigen Themes.
- Die **Filzfarbe des neuen Tischs gefällt** (`--felt: #264a3c`).
- **Akzentfarbe ist Elfenbein `#f1e6c8`** statt Orange (Auswahl aus sechs Farben: https://claude.ai/artifact/6zNiW8X4jsqkKAU5NVcRVG, gewählt Nr. 2). Orange passte dem Nutzer nicht zum Filz.
- Der Nutzer will **Vorher-Nachher-Bilder sehen, bevor ein PR geöffnet wird**.

## Die acht Schwachstellen (Prüfung an frischen Screenshots)

1. Der Tisch hat keinen Ort (leere dunkle Fläche, Gegner als Infoboxen).
2. Der Kopf frisst ein Viertel des Bildschirms (vier Leisten, ~230 von 844 px).
3. Orange bedeutet alles; Knöpfe und Anzeigen sehen gleich aus.
4. „Du bist dran“ ist das schwächste Element; alle Handkarten gleich gerahmt.
5. Die Startseite ist eine Textwand; Hauptknopf mitten im Bildschirm; Seed-Feld zu prominent; gesperrte Figuren mitten im Karussell.
6. Am Tresen liegen die Waren unter dem Falz; Texte abgeschnitten; Kaufen-Knöpfe klein.
7. Der Stadtplan hat eine andere Farbsprache (hell, creme, weinrot); „Los“-Knopf wirkte ausgegraut.
8. Der Desktop nutzt die Breite nicht.

Kleiner: In der Abrechnung steht der Spielwert dreimal untereinander; 116 fest notierte Farbwerte und rund sieben Animationszeiten im Stylesheet.

## Schritt 1: Tisch, Kopf, Farbrollen, Zug-Signal — gemergt (PR #132)

- **Branch:** `feat/ui-table-v2` (Worktree `/home/user/skat-ui`), 10 Commits über `main`, gepusht. **Kein PR.** Achtung: `origin/feat/ui-table` ist ein alter, fremder Branch und wird nicht angefasst.
- **Inhalt** (Details in `docs/DESIGN.md` 16.3):
  - Tisch als Ort: Filz, Holzkante, gestuftes Lampenlicht als Pixelgrafik (`web/scripts/gen-table-art.mjs` → `web/src/assets/table/`). Gegner sitzen oben am Tisch (Porträt 2×, Position, Stiche, Fächer mit Kartenzahl), Boss-Figuren genauso. Stichkarten größer als Handkarten.
  - Kopf: eine Leiste (Ort, Konto/Ziel, Spiele als Pips, Einmischen, Groschen) plus Relikt-Zeile mit nur belegten Plätzen und Zähler „1/8“. 110 statt ~235 px bei 390×844.
  - Farbrollen als Tokens in `web/src/style.css`: `--role-action` (Elfenbein, dunkle Tinte), `--role-gain` (Mint), `--role-money` (Messing), `--role-danger` (Rot), `--role-readout`. Knöpfe erhaben, Anzeigen flach.
  - Zug-Signal: Band „Du bist dran“ zwischen Filz und Hand; spielbare Karten heben sich, nicht spielbare dunkeln ab.
  - Elfenbein app-weit: jede frühere Orange-Verwendung einer Rolle zugeordnet; Kupfer nur noch als Dekoration auf dem Kneipenschild. Auf hellen Flächen Ring zwischen zwei dunklen Linien; „Los“ auf dem Stadtplan dunkel mit Elfenbein-Linie.
  - Skat oder Hand: Begrüßung nur noch als Sprechblase. Meldungen, Wirt-Hinweise und Schnaps-Banner als Streifen über der Kopfzeile.
- **Prüfstand:**
  - Vor Elfenbein: voller Smoke-Test grün (Commit `b9b85f1`).
  - Nach Elfenbein (Commit `10ed969`): `typecheck`, `build:web` und voller Smoke-Test grün (neu laufen lassen: `cd web && SMOKE_PORT=<frei> node e2e/smoke.mjs`, ~25 min).
  - Der Nutzer hat die Elfenbein-Bilder erhalten, aber **noch nicht freigegeben**.
- **Nächste Handgriffe für Schritt 1:**
  1. Smoke-Ergebnis prüfen, Elfenbein-Screenshots selbst kritisch ansehen (Tisch, Ansage, Abrechnung, Tresen, Stadtplan, Start; Handy und Desktop).
  2. Freigabe des Nutzers abwarten, Änderungswünsche einarbeiten.
  3. `origin/main` in den Branch mergen (kein Rebase), PR öffnen, bei grüner CI squash-mergen, Cockpit aktualisieren (Verlauf, `meta/project`).
- **Bekannte Restpunkte aus Schritt 1:**
  - Desktop: Tisch ist breiter (760 px ab 900 px Breite), wirkt aber leer; andere Bildschirme bleiben bei 560 px, die Spaltenbreite springt beim Verlassen des Tischs.
  - Gegner sitzen auf dem Filz statt hinter der Tischkante (sonst wäre der Kopf höher als 110 px).
  - Bei ≤720 px Höhe und großer Schrift schrumpfen Porträts auf 1×; bei „sehr groß“ entfallen Fächer, Beschriftungen und Statuszeile im Ansage-Blatt.
  - Bei 360 px bricht der längste englische Gastname auf zwei Zeilen.
  - Desktop: Die Hand liegt immer in zwei Reihen (`cols` in `web/src/ui/table.ts`), auch bei 1280 px. Eine Reihe gehört zu Schritt 5.
  - Während ein Gegner spricht, ersetzt die Sprechblase Name und Position (gewollt, `.opp-main:has(.quip)`), nur für die Dauer des Spruchs.

## Offene Schritte (Reihenfolge wie mit dem Nutzer besprochen)

### Schritt 2: Startseite — gebaut, noch nicht gemergt
- **Branch:** `feat/ui-start` (Worktree `/home/user/skat-ui`), gepusht, **kein PR**. Details in `docs/DESIGN.md` 16.4.
- Porträt groß (3×, ab 700 px 4×), Name, Rolle, Beschreibung auf zwei Zeilen; „mehr“/„weniger“ klappt auf (`aria-expanded`), entfällt, wenn der Text passt.
- Gesperrte Figuren am Ende des Karussells (nur Anzeige).
- „Neuer Run als …“ in einer Leiste fest am unteren Rand (sticky am Seitenende, verdeckt am Ende nichts). Mit laufendem Run bleibt „Fortsetzen“ oben der Hauptknopf.
- „Weitere Spielarten“ (aufklappbar): Tages-Run, Seed, Einführung. Wochen-Herausforderung bleibt sichtbar, darunter Statistik, Bestenliste, Vereinsbuch, Kurzregel, Fußzeile.
- **Prüfstand:** `typecheck`, `build:web`, voller Smoke-Test mit neuem `startScreenPass` (gesperrte am Ende, Hauptknopf sichtbar und unverdeckt ohne Scrollen, beide Aufklapper; 390 × 844, 360 × 740, 1280 × 800, Groß, Sehr groß, Englisch). Smoke tippt Seeds über `fillSeed()`, das die zweite Ebene öffnet.
- **Restpunkte:** Das Schild „Stammtisch“ ist unverändert hoch (~145 px); bei 360 × 740 sieht man ohne Scrollen nur Figur und Karussell. Die Karten im Karussell sind so hoch wie die längste Freischalt-Bedingung. Ein getippter Seed ist zugeklappt nicht sichtbar.

### Schritt 3: Tresen
- Waren zuerst; Vorschau auf die nächste Kneipe als schmaler, aufklappbarer Streifen.
- Voller Text per Tipp auf den Bierdeckel (keine abgeschnittenen Texte).
- Größere Kauf-Flächen (≥ 44 px).
- Preise und Preisschilder in Messing, restliche Farbrollen nachziehen.

### Schritt 4: Papier-Ebene
- Die Creme des Stadtplans als zweite Fläche im ganzen Spiel: Abrechnung als Zettel, Infokarten, Sprechblasen. So gehört der Plan optisch zum Rest.
- Abrechnung entdoppeln (Spielwert nicht dreimal).

### Schritt 5: Desktop
- Tisch in der Mitte, links Relikte und Schnäpse, rechts Stichverlauf und Zähler.
- Einheitliche Spaltenbreite über alle Bildschirme.

### Aufräumen (kann mit jedem Schritt mitlaufen)
- Fest notierte Farbwerte auf Tokens zurückführen.
- Animationszeiten auf wenige Stufen vereinheitlichen.

## Regeln für jeden Schritt

- Reine Darstellung: keine Spiellogik in `web/` oder `wasm/`, Kern unberührt.
- Pixelgrafik selbst gezeichnet, Karten im ganzzahligen Pixelmaß (`--art-step`), keine fremden Assets.
- Passt ohne Scrollen bei 390×844 und 360×740, auch mit Schrift „groß“ und „sehr groß“; Touch-Ziele ≥ 44 px; Textkontrast ≥ 4,5:1; Farbenblind-Hilfe und Bildschirmleser bleiben.
- Texte immer in `web/src/i18n/de.ts` und `en.ts`.
- Vor dem PR: `npm run typecheck`, `npm run build`, voller Smoke-Test; Screenshots Handy und Desktop selbst ansehen; dem Nutzer Vorher-Nachher zeigen.
- Smoke-Prüfungen anpassen, nie löschen. `docs/DESIGN.md` (16.x) mitziehen.
- Ein Branch pro Schritt, PR gegen `main`, Squash-Merge bei grüner CI, Cockpit-Eintrag.

## Hinweis zur CI

Am 11.11.2026 konnten GitHub-Runner stundenlang `*.ubuntu.com` nicht auflösen. Seitdem baut der Konto-Dienst im Container `rust:1.99.0-alpine`; Playwright-Jobs anderer Repos laufen ohne `--with-deps`. Scheitert ein Job an apt, liegt es daran und nicht am Code.

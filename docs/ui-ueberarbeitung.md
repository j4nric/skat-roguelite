# UI-Überarbeitung: Stand und offene Aufgaben

Stand: 13.11.2026 (Schritt 5 gebaut, `main` hineingemergt, geprüft). Übergabe-Dokument für die weitere Arbeit an der Oberfläche.

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
  - Desktop: Tisch ist breiter (760 px ab 900 px Breite), wirkt aber leer; andere Bildschirme bleiben bei 560 px, die Spaltenbreite springt beim Verlassen des Tischs. (Erledigt in Schritt 5.)
  - Gegner sitzen auf dem Filz statt hinter der Tischkante (sonst wäre der Kopf höher als 110 px).
  - Bei ≤720 px Höhe und großer Schrift schrumpfen Porträts auf 1×; bei „sehr groß“ entfallen Fächer, Beschriftungen und Statuszeile im Ansage-Blatt.
  - Bei 360 px bricht der längste englische Gastname auf zwei Zeilen.
  - Desktop: Die Hand liegt immer in zwei Reihen (`cols` in `web/src/ui/table.ts`), auch bei 1280 px. (Erledigt in Schritt 5: eine Reihe ab 900 × 600.)
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

### Schritt 3: Tresen — gebaut, noch nicht gemergt
- **Branch:** `feat/ui-tresen` (Worktree `/home/user/skat-ui`), gepusht, **kein PR**. Details in `docs/DESIGN.md` 16.5.
- Waren zuerst; die nächste Kneipe als Streifen in der unteren Leiste über „Weiter“ (eine Zeile mit Boss und Ziel, Tipp klappt die Kreidetafel darüber auf, `aria-expanded`). Boss mit rotem Rand, Wort „Boss“ und Bierdeckel.
- Kaufen-Knöpfe so breit wie die Ware, ≥ 44 px, Messing (Groschen ausgeben); Elfenbein nur bei „Weiter“. Nicht kaufbar: flach, gestrichelt, Vorhängeschloss.
- Abgeschnittene Kurztexte zeigen „mehr“, ein Tipp auf die Ware öffnet die Infokarte.
- Farbrollen nachgezogen: Fluch-Hinweise, Hehlerware, Schnäppchen, alter Preis, Schulden, Verkauft.
- **Prüfstand:** `typecheck`, `build:web`, voller Smoke-Test mit neuem `tresenPass` (allein: `SMOKE_ONLY=tresen`).
- **Restpunkte:** Die Auslage der Zinken ist so hoch wie ihr längster Text (Winkel), die Kacheln darunter haben viel Leerraum. Der Wirt mit Sprechblase nimmt weiter ~70 px. Der Hinweis „Weitere Waren weiter unten“ liegt über der unteren Warenreihe.

### Schritt 4: Papier-Ebene — gebaut, noch nicht gemergt
- **Branch:** `feat/ui-paper` (Worktree `/home/user/skat-ui`), gepusht, **kein PR**. Details in `docs/DESIGN.md` 16.6.
- Papier-Token (`--paper`, `--paper-ink`, `--paper-line`, Rollen `--paper-gain`/`-money`/`-danger`, alle ≥ 5,2 : 1 auf jedem Papierton); die Creme-Werte des Stadtplans darauf zurückgeführt. Klasse `.paper` setzt innerhalb der Fläche die Schiefer-Token auf ihre Papierstufen.
- Abrechnung als Zettel vom Kellnerblock: linierte Rechnung, darunter eine Summe „Grundwert × Stufen (× Faktor)“ und „Spielwert = …“ als Stempel, die einzige Stelle des Spielwerts; dann „Konto alt ▸ neu“, Auszahlung kompakt, „Weiter“ umgekehrt (dunkel mit Elfenbein-Linie). Der Zettel scrollt nie; bei vielen Zeilen scrollt nur die Rechnung in sich, mit Schild „weitere Zeilen“.
- Infokarten und Sprechblasen (Wirt am Tresen, Hinweise, Einführung, Sprüche) auf Papier; Kaufen auf der Infokarte umgekehrt, Glossar-Begriffe in dunkler Tinte.
- **Prüfstand:** `typecheck`, `build:web`, voller Smoke-Test mit neuem `paperPass` (allein: `SMOKE_ONLY=paper`) und `assertScoreSlip` in jedem `playOut`.
- **Restpunkte:** Der Zettel zeigt nicht, dass es eine Boss-Kneipe war (steht nur im Kopf). Im Ramsch ist der Grundwert die Augenzahl, dort steht dieselbe Zahl als Grundwert und Spielwert (Regel, kein Doppel). Alte Zinken in Spielständen (z. B. `beer_stain` in `ramsch.json`) zeigen ihre Id statt eines Namens (schon vorher so). Andere Blätter (Kontra, Kartenbelohnung, Boss, Menü) bleiben Schiefer.

### Schritt 5: Desktop — gebaut, noch nicht gemergt
- **Branch:** `feat/ui-desktop` (Worktree `/home/user/skat-ui`), gepusht, **kein PR**. Details in `docs/DESIGN.md` 16.8.
- Zwei Schwellen in `web/src/ui/layout.ts`: Desktop ab 900 × 600 (eine Spalte, Hand in einer Reihe), Seitenspalten ab 1100 × 640. Die App entscheidet per `matchMedia` und zeichnet neu, wenn das Fenster eine Schwelle kreuzt.
- Eine Spalte von 720 px (`--desk-col`) für Start, Stadtplan, Tresen, Abrechnung und Tisch; nichts springt beim Wechsel.
- Am Tisch links Relikte (Bierdeckel, Name, Text auf zwei Zeilen), Merker-Chips und Schnaps-Plätze mit Name und Wirkung (Ziehen und Antippen wie in der Leiste); der Kopf ist dann eine Zeile. Rechts „Das Spiel“ (die Zeile über dem Stich), „Letzter Stich“ mit drei kleinen Karten (Tipp legt ihn auf den Filz) und „Der Zähler“.
- Hand in einer Reihe, Karten auf ein ganzes Vielfaches von `--art-step` gerundet, überlappend, wenn die Reihe breiter als der Filz ist. Der Stich wird eine Stufe größer.
- Handy (390 × 844, 360 × 740) unverändert.
- **Prüfstand:** `typecheck`, `build:web`, voller Smoke-Test mit neuem `desktopPass` (allein: `SMOKE_ONLY=desktop`): Spalten vorhanden und ohne Überlappung, Lesereihenfolge, Kurzer aus der linken Spalte auf den Skat gezogen, Hand einreihig in ganzen Kunstpixeln, letzter Stich rechts, gleiche Spalte auf Start/Plan/Tresen/Tisch/Abrechnung (1280 × 800, 1920 × 1080), 1024 × 768 ohne Seitenspalten, Handy mit Kopf 110 px und zwei Reihen. Die Leiste-Prüfungen der anderen Pässe suchen Relikte und Schnäpse im Kopf oder in der linken Spalte.
- **Stand 13.11.2026:** `origin/main` hineingemergt (Merge, kein Rebase; Fehlerberichte, Demo und Kauf-Seite, rechtliche Seiten, Spieltest 12.11. mit Skat-Flug und ohne „Kneipe verlassen“). Einziger Konflikt: `docs/DESIGN.md`, wo `main` 16.7 für den Spieltest vom 12.11. vergeben hatte; der Desktop-Abschnitt ist jetzt **16.8** (Verweise in Code und Doku nachgezogen). `table.ts`, `style.css`, `score.ts`, `app.ts`, `smoke.mjs` und die Übersetzungen ließen sich automatisch zusammenführen.
- **Zusammenspiel mit den neuen Teilen von `main`** (Bilder 1280 × 800, 1920 × 1080, 1024 × 768, 390 × 844, Vorher = `origin/main`, Nachher = Branch): Papier-Zettel der Abrechnung über der Tischspalte, die Seitenspalten abgedunkelt daneben; Tresen mit Boss-Streifen und „Weiter“ in der 720-px-Spalte; Startseite mit Leiste „Neuer Run als …“ unten, Fußzeile mit Impressum, Datenschutz und Hilfe über der Leiste, wenn ganz nach unten gescrollt; Menü mit „Fehler melden“ und dem Fehlerbericht-Blatt über den Seitenspalten; Kauf-Seite der Demo (nur App, mit nachgebildeter Brücke) mittig. Nichts überlappt, keine Konsolenfehler. Handy 390 × 844 (Tisch, Start, Menü, Kauf-Seite, Fußzeile) pixelgleich zu `main`; beim Fehlerbericht-Blatt weicht nur die zufällige Lage des Kartenstapels unter dem Blatt ab.
- **Prüfstand 13.11.2026:** `npm ci`, `npm run wasm`, `typecheck`, `build:web` ohne Warnungen; voller Smoke-Test (`SMOKE_PORT=4699`, ~31 min) mit Exit 0, ohne Wiederholung, darin `desktopPass`, `reportPass`, `unlockPass`, `legalPass`, `scoreTapPass`.
- **Restpunkte:**
  - Der Stichverlauf zeigt nur den letzten Stich: Der Kern gibt in `GameView` nur `last_trick` heraus. Für alle bisherigen Stiche des Spiels bräuchte die Sicht ein Feld wie `tricks: TrickView[]` (Aufgabe für `core-dev`, Kern blieb in diesem Schritt unberührt).
  - Vor dem ersten Stich ist die rechte Spalte fast leer („Noch nicht angesagt“, „Noch kein Stich“).
  - Bei 1920 × 1080 bleibt die Spalte 720 px; der Filz ist hoch und schmal, die Hintergrundfläche links und rechts der Seitenspalten groß. Möglich wäre eine zweite, breitere Stufe ab ~1600 px, dann für alle Bildschirme zugleich.
  - Mit „Sehr groß“ bei 1100 × 640 scrollt die linke Spalte in sich, ohne Hinweis, dass unten noch Schnäpse liegen.
  - Die Abrechnung bleibt 440 px breit über der Tischspalte (wie bisher); die Seitenspalten liegen abgedunkelt daneben.
  - Einmal (von zwei vollen Läufen) meldete die Prüfung des Stadtplans in der Übersicht bei 1280 × 800 „Los“ unter dem Rand; beim zweiten Lauf und einzeln nachgestellt nicht. Verdacht: das Neuzeichnen beim Kreuzen der Schwelle während schneller Größenwechsel.

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

# App-Store-Auftritt: Bildschirmfotos und Vorschauvideo

Alles hier entsteht aus der echten App (`web/dist`) mit Playwright-Chromium, aus den gespeicherten Runs in `web/e2e/fixtures/` (dieselben wie im Smoke-Test). Feste Seeds und Spielstände, Ton und Hinweise aus (für das Video ist der Ton an, nur um die Spielklänge mitzuschreiben), Einführung abgelehnt, Service Worker gesperrt, Uhr fest (16.11.2026). Kein erfundener Inhalt: jedes Bild ist ein Bildschirm der App, darüber nur die Zeilen.

## Neu erzeugen

```sh
cd web
npm run build                              # die App wie ausgeliefert
node ../store/make-screenshots.mjs         # beide Sprachen; oder: de | en [Motiv 1-6]
node ../store/make-preview.mjs             # beide Sprachen mit der gewählten Musik jazz-a; oder: de | en
node ../store/make-preview.mjs de --skip-record   # nur neu schneiden (Aufnahmen aus store/build/)
node ../store/make-preview.mjs de --skip-record --music=jazz-b   # die zweite Fassung (nicht gewählt) zum Vergleich
STORE_MUSIC=1 node ../store/make-preview.mjs --skip-record      # dazu den Soundtrack neu
python3 ../store/grid.py jazz-a                # das Schnittraster einer Musik
python3 ../store/contact.py ../store/preview/preview-de.mp4 /tmp/kontakt.png 1.0   # Kontaktbogen, ein Bild je Sekunde
```

Die Skripte starten selbst `vite preview` (Port 4193 und 4194) oder nehmen `STORE_BASE=http://localhost:…/`. Sie brauchen Playwright (Chromium), `ffmpeg`, Python 3 mit numpy und Pillow. Bei einem Konsolenfehler der App brechen sie ab.

| Datei | Aufgabe |
|---|---|
| `lines.mjs` | alle Zeilen (Fotos und Video), Deutsch und Englisch |
| `lib.mjs` | Vorschau-Server, Browser-Kontext der App (Sprache, Einstellungen, feste Uhr), Spielstand laden |
| `make-screenshots.mjs` | sechs Motive je Sprache nach `screenshots/<de|en>/` (im Repo) |
| `vclock.mjs` | Aufnahme in virtueller Zeit: Playwrights falsche Uhr treibt Timer und `requestAnimationFrame`, jede Web-Animation wird angehalten und von Hand auf die Zeit gesetzt; je Schritt (1/30 s) ein Bild in Gerätepixeln (886 × 1920; die Aufnahme rendert ausdrücklich mit Faktor 2, weil eine CDP-Sitzung Faktor 1 meldet). `Math.random` ist gesät. Schreibt auch mit, welche Spielklänge die App wann startet. |
| `make-preview.mjs` | nimmt die Szenen auf (`build/<lang>/<clip>/`), zeichnet Zeilen und Endtafel, ruft Musik und Schnitt |
| `grid.py` | das Schnittraster: Ziellänge je Szene, auf ganze Schläge des Tempos gerundet; Musik und Schnitt lesen es |
| `make-music.py` | der Soundtrack in zwei Fassungen (`jazz-a`, `jazz-b`), selbst komponiert und synthetisiert (numpy), `build/music-<fassung>.wav` |
| `assemble.py` | Schnitt im Takt der Musik, Zoom, Zeilen, Spielklänge, Lautheit, Kodierung nach `preview/` |
| `contact.py` | Kontaktbogen eines Videos oder einer Aufnahme |

`build/` und `preview/` stehen nicht im Repo (`.gitignore`), die Videos sind zu groß.

## Bildschirmfotos

1320 × 2868 px (iPhone 6,9", Pflichtgröße; kleinere leitet App Store Connect ab), PNG ohne Alphakanal. Oben eine Zeile in VT323 (Elfenbein, harter Schatten) auf dem dunklen Punkt-Hintergrund der App, darunter die App bei 430 × 780 CSS-px und Faktor 3, auf 1180 px Breite eingepasst, mit dunkler Linie und hartem Schatten, ohne Geräterahmen. Die Schriftgröße ist je Sprache für alle sechs gleich (die längste Zeile passt, höchstens 176 px).

| Nr. | Motiv (Spielstand) | Deutsch | Englisch |
|---|---|---|---|
| 1 | Tisch mitten im Stich, beide Stammgäste haben gelegt, „Du bist dran“, Tante Erna mit Spruch (`factor-win.json`, Vorhand getauscht) | Echter Skat. / Jeden Abend anders. | Real Skat. / Different every night. |
| 2 | Abrechnungszettel: Schellen Hand, drei Faktor-Relikte, 9 × 15 × 2,535, Stempel 342 (`factor-win.json`) | Brich die Regeln. / Stapel die Faktoren. | Break the rules. / Stack the multipliers. |
| 3 | Der Eichel-Daus (Untot) steht als Zombie groß im Stich; in der Hand die zweite Untot-Karte, die Wandelkarte und ein Aufdruck +11 (`undead.json`) | Gezinkte Karten. / Untote Karten. | Marked cards. / Undead cards. |
| 4 | Tresen mit Wirt, Hausregeln und Schnäpsen (`zinken.json`; der Hinweis „Weitere Waren weiter unten“ ist nur für das Foto ausgeblendet) | Hausregeln und / Schnäpse vom Wirt. | House rules and the / landlord's schnapps. |
| 5 | Boss-Kneipe Die Zwillinge: Boss-Regel auf dem Filz, Franz sagt Hallo (`boss-tresen.json`) | Bosse mit / eigenen Regeln. | Bosses with / their own rules. |
| 6 | Stadtplan mit Weg zur nächsten Kneipe, Zu Fuß / Bahn / Fahrrad (`route-station.json`) | Zieh durch die / Kneipen der Stadt. | Crawl through / the city's pubs. |

## Vorschauvideo

886 × 1920, 30 fps, H.264 High (Level 4.0, yuv420p), AAC-LC Stereo 44,1 kHz 256 kbit/s, `+faststart`. Lautheit −16 LUFS (zweistufiges `loudnorm`), Spitze unter −1,5 dBTP. Ausgabe `preview/preview-<lang>.mp4` und `poster-<lang>.png` (gewählte Musik `jazz-a`); eine andere Fassung schreibt `preview-<lang>-<fassung>.mp4`. Vorschaubild (poster frame): der Stempel 342 in der Abrechnung.

Gewählt ist `jazz-a` (13.11.2026); `jazz-b` bleibt als Fassung im Skript. Der Schnitt folgt dem Tempo der Musik (`grid.py`):

| Fassung | Tempo | Länge | Stempel |
|---|---|---|---|
| `jazz-a` (gewählt) | 160 BPM, Swing | 27,0 s | Schlag 24 = 9,0 s |
| `jazz-b` | 120 BPM, Blues-Shuffle | 29,0 s | Schlag 18 = 9,0 s |

Die Zeilen erscheinen und verschwinden in Pixelstufen (geordnetes Dithering in 6-px-Blöcken).

| Szene | Inhalt (Aufnahme) | Zeile Deutsch | Zeile Englisch |
|---|---|---|---|
| sign | Startseite mit dem Schild „Noch 'ne Runde Skat?“ | Echter Skat. | Real Skat. |
| trick | Grand, Karten fliegen in den Stich, Stich an Jupp (Seed 64) | Jeden Abend anders. | Different every night. |
| quip | Geben, Tante Erna: „Ich hab Bienenstich dabei, Kind.“, Zoom auf die Blase | – | – |
| ansage | Handspiel ansagen: Schellen Hand, Schild „Schellen“ | – | – |
| score | Abrechnung: Zeilen, Relikte leuchten, Faktor 2,535, Stempel 342 auf dem Schlag | Brich die Regeln. | Break the rules. |
| undead | Untot: der Daus zerbröckelt und steht als Zombie auf | Untote Karten. | Undead cards. |
| doppel | Doppelgänger spiegelt den Schellen-Daus, „Gespiegelt!“ | Doppelgänger. | Doppelgängers. |
| curse | Tresen: Zinke gestempelt, dann Schnäppchen, die Karte zerbröckelt rot | Verfluchte Karten. | Cursed cards. |
| tresen | Tresen: Hausregel und Schnaps gekauft, Groschen, „Plopp!“ | Hausregeln. Schnäpse. / Gezinkte Karten. | House rules. Schnapps. / Marked cards. |
| boss | Boss-Blatt Die Zwillinge mit Fritz' Satz, Zoom | Bosse mit / eigenen Regeln. | Bosses with / their own rules. |
| map | Stadtplan: Bahn gewählt, „Los“, die Bahn fährt den Weg | Zieh durch die Stadt. | Crawl the city. |
| end | Endtafel: Symbol, Schild „Noch 'ne Runde?“, darunter ein Schild in Elfenbein mit dunkler Schrift und 2-px-Kante mit Pixelecken, ohne Schatten: „Kostenlos im App Store“ | – | (One more round? · Free on the App Store) |

Die Endtafel zeigt das bisherige App-Symbol; das Symbol wird neu gestaltet, die endgültigen Videos entstehen danach.

### Soundtrack

Selbst komponiert und in `make-music.py` aus Sinus-Teiltönen und gesätem Rauschen synthetisiert (numpy), keine Aufnahmen, keine fremde Musik. Beide Fassungen folgen demselben Plan: Auftakt in das Thema, ein Schlagzeug-Break, der den Stempel mit der ganzen Band auf den Schlag setzt, ein Shout-Chorus der Bläser für den Boss, eine Steigerung unter dem Stadtplan, ein großer Schlag auf die Endtafel und ein sauberer Schlussakkord. Leichte Stöße der Band und ein Becken auf jedem Schnitt.

- **jazz-a**: Uptempo-Swing in F, 160 BPM, geswingte Achtel. Klarinette als Melodie (ungerade Teiltöne, Atem, Vibrato, Ansatz von unten), Klavier mit wurzellosen Voicings (Terz, Septime, None, Stimmführung zum nächsten Akkord) im Charleston-Rhythmus, Kontrabass im Walking (Grundton auf dem Wechsel, Akkordtöne, chromatischer Schritt in den nächsten Grundton, ab und zu ein Ghost-Triolen-Auftakt), Ride-Becken mit Hi-Hat auf 2 und 4, gefederte Bassdrum, Snare-Ghosts und Comping-Drops; Bläsersatz (Trompeten-Klang, vierstimmig eng unter der Melodie) für Stöße und Shout. Harmonie I–VI–ii–V (Fmaj7 D7 Gm7 C7) mit Am7 D7, Bbmaj7 Bbm6, im Boss-Teil Bbmaj7 Eb9 Am7 D7#9, Schluss F6/9.
- **jazz-b**: Blues-Shuffle in F, 120 BPM. Saxofon-artige Melodie mit Blue Notes, Scoops und Falls, Hammond-artige Orgel (Zugriegel, Perkussion, Rotor-Schwebung), Klavier, Walking-Bass, Shuffle-Schlagzeug mit Backbeat; Bläser für Stöße und Shout. Blues-Harmonik F7 Bb7 D7#9 Gm7 C7, Schluss F13.

Menschliches Spiel: kleine Zufallsabweichungen in Zeit (wenige Millisekunden) und Anschlag, gesät und daher wiederholbar. Kleiner Raumhall (zwei Impulsantworten für Breite), leichte Bus-Kompression, nur die Spitzen weich begrenzt. Die Spielklänge (Karte, Stich, Faust, Ansage, Kreide, Gläser, Stempel-Klopfen, Kasse, Groschen, Boss-Tür, Kreide auf dem Plan …) liegen mit ihren echten Zeitpunkten aus der Aufnahme leiser darüber.

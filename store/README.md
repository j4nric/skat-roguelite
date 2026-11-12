# App-Store-Auftritt: Bildschirmfotos und Vorschauvideo

Alles hier entsteht aus der echten App (`web/dist`) mit Playwright-Chromium, aus den gespeicherten Runs in `web/e2e/fixtures/` (dieselben wie im Smoke-Test). Feste Seeds und Spielstände, Ton und Hinweise aus (für das Video ist der Ton an, nur um die Spielklänge mitzuschreiben), Einführung abgelehnt, Service Worker gesperrt, Uhr fest (16.11.2026). Kein erfundener Inhalt: jedes Bild ist ein Bildschirm der App, darüber nur die Zeilen.

## Neu erzeugen

```sh
cd web
npm run build                              # die App wie ausgeliefert
node ../store/make-screenshots.mjs         # beide Sprachen; oder: de | en [Motiv 1-6]
node ../store/make-preview.mjs             # beide Sprachen; oder: de | en
node ../store/make-preview.mjs --skip-record de   # nur neu schneiden (Aufnahmen aus store/build/)
STORE_MUSIC=1 node ../store/make-preview.mjs --skip-record   # dazu den Soundtrack neu
python3 ../store/contact.py ../store/preview/preview-de.mp4 /tmp/kontakt.png 1.0   # Kontaktbogen, ein Bild je Sekunde
```

Die Skripte starten selbst `vite preview` (Port 4193 und 4194) oder nehmen `STORE_BASE=http://localhost:…/`. Sie brauchen Playwright (Chromium), `ffmpeg`, Python 3 mit numpy und Pillow. Bei einem Konsolenfehler der App brechen sie ab.

| Datei | Aufgabe |
|---|---|
| `lines.mjs` | alle Zeilen (Fotos und Video), Deutsch und Englisch |
| `lib.mjs` | Vorschau-Server, Browser-Kontext der App (Sprache, Einstellungen, feste Uhr), Spielstand laden |
| `make-screenshots.mjs` | sechs Motive je Sprache nach `screenshots/<de|en>/` (im Repo) |
| `vclock.mjs` | Aufnahme in virtueller Zeit: Playwrights falsche Uhr treibt Timer und `requestAnimationFrame`, jede Web-Animation wird angehalten und von Hand auf die Zeit gesetzt; je Schritt (1/30 s) ein Bild in Gerätepixeln (886 × 1920). `Math.random` ist gesät. Schreibt auch mit, welche Spielklänge die App wann startet. |
| `make-preview.mjs` | nimmt die Szenen auf (`build/<lang>/<clip>/`), zeichnet Zeilen und Endtafel, ruft Musik und Schnitt |
| `make-music.py` | der Soundtrack, selbst komponiert und synthetisiert (numpy), `build/music.wav` |
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
| 4 | Tresen mit Wirt, Hausregeln und Schnäpsen (`zinken.json`) | Hausregeln und / Schnäpse vom Wirt. | House rules and the / landlord's schnapps. |
| 5 | Boss-Kneipe Die Zwillinge: Boss-Regel auf dem Filz, Franz sagt Hallo (`boss-tresen.json`) | Bosse mit / eigenen Regeln. | Bosses with / their own rules. |
| 6 | Stadtplan mit Weg zur nächsten Kneipe, Zu Fuß / Bahn / Fahrrad (`route-station.json`) | Zieh durch die / Kneipen der Stadt. | Crawl through / the city's pubs. |

## Vorschauvideo

886 × 1920, 30 fps, H.264 High (Level 4.0, yuv420p), AAC-LC Stereo 44,1 kHz 256 kbit/s, 28,6 s, `+faststart`. Lautheit −16 LUFS (zweistufiges `loudnorm`), Spitze unter −1,5 dBTP. Vorschaubild (poster frame): der Stempel 342 in der Abrechnung, bei 9,4 s (`poster-<lang>.png`; in App Store Connect den Zeitpunkt 00:09 wählen).

Geschnitten wird auf das Raster der Musik (130 BPM, ein Schlag = 0,46 s). Die Zeilen erscheinen und verschwinden in Pixelstufen (geordnetes Dithering in 6-px-Blöcken, vier und drei Bilder).

| Schläge | Zeit | Szene (Aufnahme) | Zeile Deutsch | Zeile Englisch |
|---|---|---|---|---|
| 0–4 | 0,0–1,8 s | Startseite mit dem Schild „Noch 'ne Runde Skat?“ (`sign`) | Echter Skat. | Real Skat. |
| 4–8 | 1,8–3,7 s | Grand, Karten fliegen in den Stich, Stich an Jupp (`trick`, Seed 64) | Jeden Abend anders. | Different every night. |
| 8–13 | 3,7–6,0 s | Geben, Tante Erna: „Ich hab Bienenstich dabei, Kind.“, Zoom auf die Blase (`factor`) | – | – |
| 13–16 | 6,0–7,4 s | Handspiel ansagen: Schellen Hand, Schild „Schellen“ (`factor`) | – | – |
| 16–24 | 7,4–11,1 s | Abrechnung: Zeilen, Relikte leuchten, Faktor 2,535, Stempel 342 auf Schlag 20 (`factor`) | Brich die Regeln. | Break the rules. |
| 24–28 | 11,1–12,9 s | Untot: der Daus zerbröckelt und steht als Zombie auf (`undead`) | Untote Karten. | Undead cards. |
| 28–32 | 12,9–14,8 s | Doppelgänger spiegelt den Schellen-Daus, „Gespiegelt!“ (`doppel`) | Doppelgänger. | Doppelgängers. |
| 32–36 | 14,8–16,6 s | Tresen: Zinke gestempelt, dann Schnäppchen, die Karte zerbröckelt rot (`curse`) | Verfluchte Karten. | Cursed cards. |
| 36–40 | 16,6–18,5 s | Tresen: Hausregel und Schnaps gekauft, Groschen, „Plopp!“ (`tresen`) | Hausregeln. Schnäpse. / Gezinkte Karten. | House rules. Schnapps. / Marked cards. |
| 40–46 | 18,5–21,2 s | Boss-Blatt Die Zwillinge mit Fritz' Satz, Zoom (`boss`) | Bosse mit / eigenen Regeln. | Bosses with / their own rules. |
| 46–52 | 21,2–24,0 s | Stadtplan: Bahn gewählt, „Los“, die Bahn fährt den Weg (`map`) | Zieh durch die Stadt. | Crawl the city. |
| 52–62 | 24,0–28,6 s | Endtafel: Symbol, Schild „Noch 'ne Runde?“, „Jetzt im App Store“ | – | (One more round? · Now on the App Store) |

### Soundtrack

Selbst komponiert und in `make-music.py` aus Sinus-Teiltönen und gesätem Rauschen synthetisiert, keine Aufnahmen, keine fremde Musik. A-Moll, 130 BPM, Harmonie i–VI–III–VII (Am F C G) mit E als Dominante, D-Moll im Boss-Teil. Instrumente: Akkordeon (zwei gegeneinander verstimmte Zungen mit Vibrato, eine Oktave tiefer dazu) für Thema und Offbeat-Akkorde, ein Chiptune-Puls (25 %) als Doppelung, Saw-Bass in Achteln mit Oktavsprung, breite Flächen aus fünf verstimmten Sägezähnen (leicht vom Kick geduckt), Kick, Snare, Hi-Hats, Toms, Crashs, Sub-Schläge, ein Riser. Aufbau: Intro mit Akkordeon-Stößen und Snare-Wirbel, ab Schlag 8 Four-on-the-floor mit Thema, Fill und Crash auf den Stempel (20), Thema eine Oktave höher ab 28, Boss-Teil in halbem Tempo mit Toms (40), Wirbel und Riser ab 46, großer Schlag auf die Endtafel (52) und Schlussakkord auf Schlag 60. Die Spielklänge (Karte, Stich, Faust, Ansage, Kreide, Gläser, Stempel-Klopfen, Kasse, Groschen, Boss-Tür, Kreide auf dem Plan …) liegen mit ihren echten Zeitpunkten aus der Aufnahme leiser darüber.

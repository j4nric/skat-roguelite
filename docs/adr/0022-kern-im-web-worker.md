# 0022: Der Kern läuft in einem Web Worker

Datum: 2026-11-10 · Status: Angenommen · Ergänzt 0009

## Kontext

Seit ADR 0009 läuft der Rust-Kern als WebAssembly im Browser, bisher auf dem Hauptthread neben der Oberfläche. Jede Entscheidung eines Gegners (PIMC-Suche, ab Stadtteil 2) blockiert ihn: nativ etwa 1 bis 35 ms je Karte, auf dem Handy als WebAssembly ein Mehrfaches. Gemessen mit Playwright bei vierfach gedrosselter CPU über zehn Stiche (20 Gegnerkarten): Der Kern rechnete auf dem Hauptthread 356 ms (`run-end.json`, Stadtteil 2) und 970 ms (`old-man.json`, Stadtteil 3), also im Mittel 18 und 49 ms je Gegnerkarte, einzelne über 100 ms. So lange stand die Denkblase still und Taps warteten. Die Frage war, ob die ganze Sitzung in einen Worker zieht (jeder Aufruf wird asynchron) oder nur die teuren Gegnerzüge in einer gespiegelten Sitzung im Worker laufen.

## Entscheidung

- **Die ganze Sitzung lebt in einem eigenen Web Worker** (`web/src/core.worker.ts`). Die Web-App spricht mit ihm über Nachrichten mit fortlaufender Id; jeder Aufruf des Kerns (`web/src/core.ts`, `CoreRun` und `CoreMeta`) gibt ein Promise zurück. Der Worker bearbeitet eine Anfrage nach der anderen in der Reihenfolge, in der sie kamen; die Antworten kommen also in derselben Reihenfolge zurück.
- **Der Hauptthread behält, was der Kern zuletzt sagte**: die Sicht jedes Runs und des Meta-Zustands und ihren Spielstand als Text. Jede Antwort, die den Run ändert, bringt den neuen Spielstand mit. So bleiben `view()` und `save()` synchron, und `persist` beim Verlassen der Seite (`pagehide`) braucht keine Antwort aus dem Worker. Was auf das Speichern wartet (die Karten der Gegner, W6 des Reviews), ist der Spielstand nach einem Schritt zusammen mit dem Meta-Zustand, der dessen Ereignisse schon gesehen hat; beides ändert sich im selben Schritt.
- **Eingaben während ein Gegner denkt**, bleiben gesperrt wie bisher (`ui.busy`). Wo ein Bildschirmwechsel auf den Kern wartet (Startbildschirm, neuer Run, Einführung), gewinnt der spätere; Kartenwähler und Vorschau öffnen sich nur, wenn sich Tisch und Dialog inzwischen nicht geändert haben.
- **Rückfall**: Gibt es keinen Worker, verweigert der Browser einen Modul-Worker oder WebAssembly darin, läuft dieselbe Schicht (`web/src/core-engine.ts`) auf dem Hauptthread, hinter denselben Promises und in derselben Reihenfolge. Dann wartet der Gegnerzug wie bisher, bis die Denkblase gemalt ist.
- Die Seite lädt die Bytes des Kerns (über das Preload im HTML) und übergibt sie dem Worker; der Worker selbst lädt nichts. Er ist ein eigener Chunk unter `/assets/` und liegt damit im Cache des Service Workers (Offline-Spiel, DESIGN 14.5). Die CSP (`deploy/Caddyfile`) bleibt unverändert: `worker-src 'self'` erlaubt den Worker, `'wasm-unsafe-eval'` in `script-src` gilt auch für ihn, weil Caddy denselben Header an sein Skript hängt.
- Der Kern und `wasm/` ändern sich nicht. Im Worker liegt keine Spiellogik, nur die Hülle um `skat_wasm`.

## Alternativen

- **Mischform: Sitzung auf dem Hauptthread, gespiegelte Sitzung im Worker nur für die Gegnerzüge.** Weniger Umbau in der App, aber zwei Kopien des Kerns im Speicher, zwei Instanzen zum Laden, und für jeden Gegnerzug müsste der Run hin- und zurückgereicht (laden, speichern) oder der Kern um „Gegnerkarte vorschlagen, dann anwenden“ erweitert werden. Die Spiegel könnten auseinanderlaufen. Verworfen.
- **`SharedArrayBuffer` und Threads im Kern**: braucht Cross-Origin-Isolation-Header und Threads im Kern; ADR 0009 hält den Kern ohne Threads. Verworfen.
- **Nur die Suche verkürzen**: hilft auf schwachen Handys nicht sicher und kostet Spielstärke. Kein Ersatz.

## Folgen

- Jeder Aufruf des Kerns ist asynchron; neue Aufrufstellen müssen damit rechnen, dass sich Bildschirm oder Run beim Eintreffen der Antwort geändert haben.
- Der Kern rechnet nicht mehr auf dem Hauptthread: Während die Gegner denken, laufen Blase, Animation und Eingaben weiter. Gemessen wie oben: Rechenzeit des Kerns auf dem Hauptthread 0 ms, lange Aufgaben je Stich im Mittel 182 bis 266 ms vorher, 119 bis 161 ms nachher (Stadtteil 2), 255 bis 257 ms vorher, 140 bis 147 ms nachher (Stadtteil 3). Die Zeit von der eigenen Karte bis zum fertigen Stich bleibt gleich (etwa 3,1 s, fast nur Animation). Lange Aufgaben, die bleiben, kommen vom Zeichnen (ein `render` kostet gedrosselt im Median 27 ms, bis 150 ms) und müssen dort angegangen werden.
- Jeder Schritt serialisiert den Run einmal mehr (im Worker, nicht auf dem Hauptthread).
- Stirbt der Worker mitten im Abend, scheitern die folgenden Aufrufe mit einer Meldung; ein Neuladen der Seite setzt beim zuletzt gespeicherten Stand wieder auf.
- Spätere Möglichkeit: die nächste Gegnerkarte schon berechnen, während die vorige noch fliegt.

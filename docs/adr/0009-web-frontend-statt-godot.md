# 0009: Web-Frontend mit WebAssembly-Kern statt Godot

Datum: 2026-11-01 · Status: Angenommen · Ersetzt teilweise 0004 und 0007

## Kontext

ADR 0004 hat Godot 4 als Frontend gewählt, angebunden über GDExtension. ADR 0007 hat den Browser als Zielplattform des Servers festgelegt. Diese Kombination ist heikel: Rust-GDExtensions im Godot-Web-Export brauchen Nightly-Rust, die Emscripten-Toolchain und Export-Templates mit dynamischem Linken. Das gilt als experimentell. Godot ist zudem in der Cloud-Umgebung, in der Claude autonom arbeitet, nicht installiert. Oberflächenänderungen ließen sich dort nicht prüfen.

## Entscheidung

- Der Rust-Kern wird über `wasm-bindgen` zu WebAssembly kompiliert. Der Crate `bridge/` wird durch `wasm/` ersetzt (Crate `skat_wasm`).
- Die Oberfläche ist eine Web-App in `web/`: TypeScript, gebaut mit Vite. Karten und Animationen werden mit Canvas oder DOM/SVG gezeichnet. Die Bibliothek wählt der Frontend-Agent nach Bedarf, sie muss ohne Laufzeit-Abhängigkeit von fremden Servern funktionieren.
- Ziel ist zuerst das Handy hochkant, Desktop nutzt dasselbe Layout mit mehr Rand.
- Claude prüft jede Oberflächenänderung selbst mit Playwright und Chromium in der Cloud-Umgebung (Screenshots, einfache Durchspiel-Tests).

Entschieden vom Nutzer über das Cockpit am 01.11.2026.

## Alternativen

- Godot mit Web-Export trotz Risiko: verworfen wegen experimenteller Toolchain und fehlender Testbarkeit in der Cloud.
- Godot nur für Desktop plus Web-App für den Server: doppelte Oberfläche, doppelter Aufwand.

## Folgen

- Der Kern muss WebAssembly-tauglich bleiben: keine Threads, kein Dateizugriff zur Laufzeit, keine Systemzeit. Content-Daten werden eingebettet.
- Ab M2 ist das Spiel im Browser spielbar, nicht erst ab M3.
- Eine Desktop-App ist später über Tauri möglich, ohne neue Oberfläche.
- Die Cross-Origin-Isolation-Header für Godot-Threads entfallen.

# 0004: Rust-Kern, Godot-Frontend

Datum: 2026-10-31 · Status: Angenommen

## Kontext

Das Spiel braucht einen Simulator, der tausende Partien pro Sekunde spielt, und eine Präsentationsschicht für Desktop und später mobil.

## Entscheidung

Die gesamte Spiellogik liegt im Rust-Crate `skat_core` ohne Engine-Abhängigkeit. Godot 4 ist der Client, angebunden über GDExtension (`gdext`). Der Simulator ist ein zweiter Client desselben Kerns.

## Alternativen

- Alles in GDScript: zu langsam für den Simulator, schlecht testbar.
- Alles in Rust mit eigener Engine (Bevy): mehr Kontrolle, aber deutlich mehr Aufwand für UI, Animation und Export.
- Kern in Rust, UI im Web: möglich, aber Godot liefert Export für alle Plattformen fertig.

## Folgen

Der Kern darf nichts von Godot wissen. Jede Spiellogik, die versehentlich in GDScript entsteht, ist ein Fehler. Der Bridge-Crate bleibt bis M3 leer, damit Builds schnell bleiben.

---
name: sim
description: Simulator laufen lassen und das Ergebnis als Tabelle zusammenfassen. Aufruf mit optionalem Seed und Spielzahl, z. B. /sim 42 10000.
---

Fahre den Headless-Simulator und fasse das Ergebnis zusammen.

1. Argumente lesen: erstes Argument Seed (Standard 42), zweites Spielzahl (Standard 10000). `$ARGUMENTS`
2. Ausführen:
   ```sh
   cargo run --release -p skat_sim -- --seed <seed> --games <games>
   ```
3. Ergebnis als Markdown-Tabelle zusammenfassen: Gewinnquote je Spielart, mittlerer Spielwert, Varianz, Laufzeit pro Spiel. Falls der Simulator noch ein Stub ist, das klar sagen.
4. Wenn ein Vergleichswert in `docs/balance/` existiert, die Differenz zur letzten Messung nennen.
5. Nur Befehl und Tabelle ausgeben, keine Interpretation, es sei denn, sie wird verlangt. Interpretation ist Aufgabe des Agenten `balance`.

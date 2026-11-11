# Sonderregeln der Woche: Gewinnbarkeit

- **Datum**: 10./11.11.2026
- **Branch**: `feat/weekly-and-stats`
- **Frage**: Ist jede Sonderregel der Woche (`core/content/weekly.ron`, DESIGN 5.6) gewinnbar, und keine absurd leicht oder schwer?
- **Maßstab**: Standard-Bot des Simulators (Heuristik für Ansage und Kartenspiel, Verteidiger nach `run.ron`), mit Tresen, 3000 Runs je Regel, Seed 42. Die Regel liegt auf jedem Run (`--weekly-rule <id>`), auf denselben Seeds wie die Grundlage (gepaart). Grundlage ist der Alleinspieler ohne Regel; für den Nullabend der Nullspieler ohne Regel. Standardfehler bei 3000 Runs: rund 0,9 / 0,6 / 0,3 Punkte (Stadtteil 2 / 3 / gewonnen).
- **Grenze**: Gewonnen höchstens etwa das Doppelte und mindestens etwa die Hälfte der Grundlage; sonst nachstellen.

## Befehle

```sh
cargo build --release -p skat_sim
./target/release/sim --seed 42 --runs 3000                       # Grundlage
./target/release/sim --seed 42 --runs 3000 --character null      # Grundlage Nullabend
./target/release/sim --seed 42 --runs 3000 --weekly-rule <id>    # je Regel
```

## Ergebnis

| Regel | Stadtteil 2 | Stadtteil 3 | gewonnen | Urteil |
|---|---|---|---|---|
| Grundlage Alleinspieler | 35,4 % | 10,6 % | 2,1 % | |
| Grundlage Nullspieler | 34,1 % | 12,4 % | 2,6 % | |
| Grand zählt doppelt (nur Grand ×2) | 41,1 % | 15,0 % | 4,3 % | zu leicht |
| **Grand zählt doppelt** (dazu Farbspiel ×0,75) | 38,9 % | 13,0 % | 3,5 % | leicht, behalten |
| Spelunkenluft (Malus je Kneipe, +6 Groschen) | 26,6 % | 7,1 % | 0,7 % | zu schwer |
| Spelunkenluft (Spelunkenkenner und +6 Groschen) | 38,6 % | 13,9 % | 2,5 % | |
| **Spelunkenluft** (Spelunkenkenner) | 36,3 % | 13,2 % | 1,9 % | behalten |
| **Hellwache Gegner** | 35,1 % | 9,8 % | 1,6 % | behalten |
| **Kurze Geduld** (+6 Groschen) | 32,5 % | 9,6 % | 1,7 % | behalten |
| Teufelspakt (Schutzengel, drei Neunen ramponiert) | 60,7 % | 23,2 % | 5,3 % | viel zu leicht |
| Teufelspakt (dazu Zielwerte +20 %) | 54,1 % | 17,1 % | 2,9 % | |
| **Teufelspakt** (dazu Zielwerte +30 %) | 49,9 % | 13,7 % | 1,7 % | behalten |
| **Nullabend** (gegen Grundlage Nullspieler) | 34,1 % | 12,4 % | 2,6 % | gleich, behalten |
| **Erbonkel** (+15 Groschen, Zielwerte +10 %) | 35,3 % | 10,4 % | 1,6 % | behalten |
| **Spendierhosen** (Tournee, Bierdeckel, Tresen +2) | 39,9 % | 12,3 % | 2,7 % | behalten |

Fett der Stand in `weekly.ron`.

## Lesart

- **Teufelspakt**: Der Schutzengel ist für den Bot der stärkste Einzelhebel (ein verlorener Run wird eine zweite Chance); drei ramponierte Neunen kosten fast nichts. Erst Zielwerte +30 % bringen die Gewinnquote unter die Grundlage. Die Regel hat jetzt eine eigene Form: Stadtteil 2 viel öfter (49,9 statt 35,4 %), danach steiler.
- **Spelunkenluft**: Ein Malus in jeder Kneipe kostet zwei Drittel der Siege; Groschen gleichen das nicht aus. Der Spelunkenkenner (+2 Stufen in jeder Kneipe mit Malus) passt zum Thema und hebt sie auf die Grundlage.
- **Grand zählt doppelt** lohnt dem Bot zu sehr (er spielt 24 % Grand); Farbspiele ×0,75 nehmen die Hälfte des Vorteils wieder weg. Sie bleibt die leichteste Woche.
- **Nullabend** ist der Nullspieler ohne weitere Änderung; die Messung ist gleich seiner Grundlage (gleiche Seeds).
- **Hellwache Gegner** wirkt erst ab Stadtteil 2 (dort spielen die Verteidiger PIMC; die Heuristik bleibt die Heuristik), deshalb nur −0,5 Punkte bei den Siegen.
- Keine Regel liegt danach unter 1,6 % oder über 3,5 % gewonnen (Grundlage 2,1 %).

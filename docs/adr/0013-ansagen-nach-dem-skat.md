# 0013: Schneider, Schwarz und Ouvert auch nach dem Skat

Datum: 2026-11-01 · Status: Angenommen · Ändert einen Punkt aus 0011

## Kontext

ADR 0011 folgt der Skatordnung: Schneider angesagt, Schwarz angesagt und Ouvert gibt es bei Farbspiel und Grand nur aus der Hand, nach dem Aufnehmen bleibt nur *Normal* (Null Ouvert ging schon immer). Im Spieltest vom 01.11.2026 wollte der Nutzer diese Ansagen auch ohne Handspiel: Wer den Skat aufnimmt, sieht sein Blatt erst danach vollständig, und gerade dann weiß er, ob es für Schneider oder offen reicht. Die Ansage ist der Push-your-luck-Hebel des Spiels (DESIGN 4.3); mit einem Weg mehr zu ihr wird sie öfter genutzt, ohne dass sie ihr Risiko verliert. Die Regel darf hier gebogen werden, weil das Roguelike kein Reizen hat und der Spieler ohnehin immer Alleinspieler ist.

## Entscheidung

Nach dem Aufnehmen und Drücken darf der Spieler bei Farbspiel und Grand *Normal*, *Schneider angesagt*, *Schwarz angesagt* oder *Ouvert* ansagen. Die Reihenfolge Skat → drücken → ansagen aus 0011 bleibt. Die Stufe *Hand* gibt es nur, wenn aus der Hand gespielt wird; eine Ansage nach dem Skat bringt ihre eigenen Stufen (Schneider angesagt +1, Schwarz angesagt +1, Ouvert +1, dazu Schneider und Schwarz, wenn erreicht), aber keine Hand-Stufe. Ein Fehlschlag auf der angesagten Stufe ist verloren wie aus der Hand. Null bleibt wie gehabt (nach dem Skat Null und Null Ouvert).

## Alternativen

- Bei der Skatordnung bleiben: weniger Ansagen, die Entscheidung nach dem Skat bleibt flach.
- Nach dem Skat nur Ouvert erlauben: halbe Lösung, Schneider angesagt ist der häufigere Wunsch.
- Als Hausregel (Relikt) statt als Grundregel: der Nutzer wollte es als Grundregel.

## Folgen

- Im Kern trägt die Ansage für Farbspiel und Grand, ob aus der Hand gespielt wird (`Declaration::Suit { suit, risk, hand }`, `with_hand`). Im JSON steht nach dem Skat `after_skat: true`; ältere Ansagen ohne das Feld lesen sich wie in der Skatordnung. Die Phase des Spiels entscheidet, die Oberfläche schickt nur `{game, risk}`.
- Bedingungen auf die Risikostufe (`RiskAtLeast`, die Freischaltung „Aus der Hand“) prüfen *Hand* nur am Handspiel, die Ansagen an der Stufe, egal wie gespielt (`Declaration::risk_at_least`).
- Der Angeber (immer offen) ist auch mit Farbspiel oder Grand Ouvert nach dem Skat zufrieden.
- Der Bot sagt nach dem Skat mit denselben vorsichtigen Regeln an wie aus der Hand, gemessen an den zehn Karten, die er behält.
- Neue Golden-Vektoren in `core/tests/game_value_vectors.rs` für Ansagen nach dem Skat.

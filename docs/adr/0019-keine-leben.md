# 0019: Keine Leben: Eine verlorene Kneipe beendet den Abend

Datum: 2026-11-04 · Status: Angenommen (Zahlen vorläufig bis zur Messung) · Ersetzt den Lebens-Teil von [0016](0016-kneipentour.md) (Punkt 2: Leben, Punkt 3: Rast „ein Leben zurück“) und von [0017](0017-bosse-v2.md) (Punkt 3: „Leben zählen nur in normalen Kneipen“, Punkt 4: „ein Leben“ als Boss-Belohnung)

## Kontext

Mit der Kneipentour (ADR 0016) bekam der Run Leben: Start 2, höchstens 4, eine verfehlte Kneipe kostete eins, eine geschaffte Boss-Kneipe brachte eins (ADR 0017), die Rast, das Konterbier und zwei Begegnungen gaben eins zurück. Seit ADR 0017 beendete die Boss-Kneipe den Run ohnehin, egal wie viele Leben übrig waren; Leben zählten nur noch in normalen Kneipen und Spelunken.

Im Spieltest zeigte sich:

- **Leben nahmen den normalen Kneipen das Gewicht.** Eine verfehlte Kneipe war ein Herz weniger, kein Ende; mit Rast und Konterbier ließ sich der Vorrat wieder auffüllen. Die Spannung lag fast nur beim Boss.
- **Leben verwässerten die Stationen.** Rast, Konterbier und zwei Begegnungsoptionen taten nur etwas, solange ein Leben fehlte, und waren sonst gesperrt oder wertlos.
- **Zwei Regeln für eine Niederlage.** Normale Kneipe: ein Leben. Boss-Kneipe: alles vorbei. Das musste die Oberfläche an jeder Stelle erklären.

Der Nutzer hat am 04.11.2026 entschieden: „Eine verlorene Kneipe verliert das Spiel. Keine zusätzlichen Leben in Events mehr. Stattdessen ein seltenes Relikt für 2te Chance.“

## Entscheidung

1. **Keine Leben.** Jede verlorene Kneipe beendet den Run: eine normale Kneipe, eine Spelunke und die Boss-Kneipe gleichermaßen. Das gilt auch für eine aufgegebene Kneipe („Kneipe verlassen“).
2. **Zweite Chance nur über den Schutzengel.** Der Schutzengel (`guardian_angel`, Selten) wird umgebaut: „Einmal im Run: Verlierst du eine Kneipe, spielst du sie noch einmal, auch die Boss-Kneipe. Danach ist der Schutzengel weg.“ Die Kneipe wird von vorn gespielt: dieselbe Kneipe oder Spelunke mit ihrem Malus, derselbe Boss (nicht neu gezogen), frisches Konto, neue Karten, der Zielwert der Kneipe ohne die Änderungen, die unterwegs für sie geplant waren (Wegbier: verbraucht mit dem ersten Versuch). Die Hausregeln des Abends bleiben. Der Engel ist danach verbraucht (aus den Relikten entfernt). Er hilft auch in der Boss-Kneipe (Empfehlung des Auftrags, so umgesetzt): Er ist selten und einmalig, und gerade dort ist eine zweite Chance etwas wert.
3. **Keine Leben mehr in Inhalten.** Die Rast (Imbissbude) bekommt einen neuen Zweck: eine Currywurst für ein Einmischen mehr in der nächsten Kneipe, oder eine Karte am Waschbecken sauber waschen (wie „Karte reinigen“ der Werkstatt). Die Rast liegt damit immer auf einem Weg, nicht mehr nur mit fehlendem Leben. Das Verlorene Portemonnaie gibt beim Abgeben einen Schnaps statt eines Lebens. Das Konterbier entfällt. Die Boss-Belohnung bleibt doppelte Auszahlung und ein Relikt nach Wahl, ohne Leben.
4. **Alte Spielstände laden.** Das Feld `lives` (und der Stadtteil, in dem der alte Schutzengel half) wird gelesen und übergangen; ein laufender Run spielt ohne Leben weiter. Ein Konterbier in der Hand wird mit seinem Preis (4 Groschen) erstattet. Eine offene Begegnung, die noch ein Leben anbietet, bietet stattdessen 3 Groschen. Keine neue Spielstand-Version.
5. **Alle Zahlen sind vorläufig**, bis eine Runde im Simulator die Zielkurve ohne Leben neu misst (Preis des Schutzengels bleibt 8, die Rast-Optionen nach Augenmaß).

## Alternativen

- **Leben behalten, nur aus den Ereignissen nehmen:** die Härte der normalen Kneipen bliebe gering, die doppelte Regel (Kneipe gegen Boss) bliebe.
- **Ein Leben für den ganzen Run:** ist praktisch dasselbe wie ein Schutzengel für alle, aber ohne Entscheidung des Spielers; verworfen zugunsten des Relikts, das man finden, kaufen und gezielt einsetzen kann.
- **Ein zweites, neues Relikt neben dem alten Schutzengel:** zwei Relikte für dieselbe Idee; der Schutzengel hatte schon den Namen dafür.
- **Schutzengel nicht in der Boss-Kneipe:** schlichter, aber er wäre in der wichtigsten Kneipe wertlos; als einmaliges seltenes Relikt darf er dort helfen.

## Folgen

- **Kern:** Entfallen `Run::lives`, `LivesConfig` (`lives` in `run.ron`), `TourView::lives`, `start_lives`, `max_lives`, die Ereignisse `LivesChanged` und `LifeSpared`, `ConsumableAction::GainLives`, `OptionBlock::LivesFull`. Neu: `Effect::SecondChance` (liest den alten Namen `SpareLife`), Ereignis `SecondChance { relic, section, index }`, `Run::replay_pub`, `REFUNDED_CONSUMABLES`, `LEGACY_LIFE_COINS`. `EncounterEffect::Lives` bleibt nur zum Lesen alter Stände; Inhalte dürfen ihn nicht mehr nennen (Ladefehler). Die Fehlercodes `LivesFull` bleiben stehen (kommen nicht mehr vor).
- **Bot:** Spelunke nur mit vier Relikten oder drei und dem Schutzengel (bisher drei Leben und drei Relikte); die Rast ist ihm eine kleine Station (mehr mit einer verfluchten Karte); kein Konterbier.
- **Simulator:** keine Leben-Spalten mehr; neu je Kneipe und Stadtteil die mit dem Schutzengel wiederholten Kneipen; wo die Runs enden (normale Kneipe, Spelunke, Boss) bleibt.
- **Oberfläche:** keine Herzen im Kopf und auf der Kreidetafel, keine Texte über Leben; „Kneipe verlassen“ sagt, dass der Run endet (oder dass der Schutzengel die Kneipe neu spielen lässt); die Rast zeigt ihre neuen Optionen.
- **Balance:** Der Run wird deutlich härter: Bis hierher endete gut ein Drittel der verlorenen Runs in normalen Kneipen, jetzt endet jede verlorene Kneipe den Run. Die Zielkurve (DESIGN 5.0.4) muss neu gemessen werden; das Zielbild „ein Viertel bis ein Drittel der verlorenen Runs in normalen Kneipen“ gilt nicht mehr.

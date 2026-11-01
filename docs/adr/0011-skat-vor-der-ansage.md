# 0011: Erst der Skat, dann die Ansage

Datum: 2026-11-01 · Status: Angenommen · Ändert die Reihenfolge aus 0006 · Ansagen nach dem Skat geändert durch 0013

## Kontext

ADR 0006 ersetzt das Reizen durch eine Ansage und legt sie vor das Aufnehmen des Skats: Der Spieler wählte Spielart und Risikostufe, danach kam der Skat. Im ersten Spieltest fiel das sofort als falsch auf. Nach der Skatordnung entscheidet der Alleinspieler zuerst, ob er den Skat aufnimmt oder Hand spielt, drückt dann zwei Karten und sagt erst danach an. Wer Skat kennt, stolpert über die vertauschte Reihenfolge, und die Ansage verliert ihren Reiz, weil die zwei Karten aus dem Skat die Spielart oft erst entscheiden.

## Entscheidung

Der Ablauf folgt der Skatordnung: Hand oder Skat aufnehmen. Beim Aufnehmen zwei Karten drücken. Dann die Ansage von Spielart und Risikostufe. Wer Hand spielt, sagt sofort an und darf Schneider, Schwarz und Ouvert ansagen. Nach dem Aufnehmen sind nur die Stufen erlaubt, die die Skatordnung erlaubt (bei Farbspiel und Grand kein Schneider oder Schwarz angesagt und kein Ouvert; Null Ouvert bleibt möglich). Das Reizen bleibt abgeschafft (0006 gilt sonst weiter).

## Alternativen

- Ansage vor dem Skat beibehalten: weniger Zustände, aber widerspricht der Skatordnung und dem Gefühl jedes Skatspielers.
- Ansage vor dem Skat als Hausregel anbieten: möglich als späteres Relikt, nicht als Grundregel.

## Folgen

- Der Spielautomat im Kern hat neue Phasen (`Discarding`, `DeclaringAfterSkat`). Alte Spielstände laden weiter; ein Stand, der schon eine Ansage vor dem Drücken hält, spielt in der alten Reihenfolge zu Ende.
- Die KI für den Alleinspieler (Bot im Simulator) wählt die Spielart jetzt nach dem Drücken. Die Messwerte ab M1 verschieben sich; die Tabellen im Design-Dokument werden nachgemessen.
- Geteilte Seeds aus der Zeit davor spielen sich anders aus.

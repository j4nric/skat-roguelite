# 0017: Bosse v2: zwei Boss-Spieler, alles oder nichts

Datum: 2026-11-03 · Status: Angenommen · Ersetzt den Boss-Teil von ADR 0016 (Punkt 2: „Eine verfehlte Boss-Kneipe kostet ein Leben und wird wiederholt“; Punkt 5: ein Boss-Platz, Sekundant offen)

## Kontext

Mit der Kneipentour (ADR 0016) saß der Boss in der Boss-Kneipe am Ende jedes Stadtteils: ein Boss-Platz links, ein Stammgast rechts, eine verlorene Boss-Kneipe kostete ein Leben und wurde sofort wiederholt. Im Spieltest und in der Durchsicht der Bosse zeigte sich:

- **Die Boss-Kneipe hatte kein Gewicht.** Mit drei Leben ließ sich ein Boss mehrmals versuchen; die Wiederholung war eher Wartezeit als Spannung, und die Belohnung (ein Leben) glich den Verlust fast aus.
- **Ein Boss allein am Tisch wirkt halb.** Der Stammgast rechts spielte die normale Ausgabe; der Boss war eine Regel und eine Figur, kein Gegnerpaar. Die Frage nach einem Sekundanten stand im Cockpit offen.
- **Drei Bosse trugen wenig:** Der Kontra-Hans war eine Hausregel mit Gesicht, der Revolutionär und der Kiebitz-Mörder wirkten über Information, die für den Spieler schwer zu sehen war (Tausch nach dem fünften Stich, abgeschaltete Relikte).
- **Die Oma** (alle Unter Blankos) traf nur den Alleinspieler und war so hart, dass ihr Faktor bei 0,3 lag; **der Ramscher** prägte mit einem einzigen Ramsch seine Kneipe kaum.

Der Nutzer hat am 03.11.2026 entschieden (Brief „Bosse v2“).

## Entscheidung

1. **Zwei Boss-Spieler.** Jeder Boss sitzt mit zwei Figuren am Tisch, auf beiden Gegnerplätzen, jede mit eigenem deutschem Namen und eigener Ausgabe zum Thema des Bosses (nicht unbedingt gleich). Beide zeigen die Rückseite des Bosses. Daten: `figures` in `bosses.ron`. Das beantwortet die offene Frage nach dem Sekundanten.
2. **Alle drei Spiele zählen.** In der Boss-Kneipe gibt es kein frühes Ende; der Zielwert wird nach dem dritten Spiel verglichen, einen Bonus je übrigem Spiel gibt es nicht.
3. **Verloren ist der Abend vorbei.** Eine verlorene Boss-Kneipe beendet den Run, egal wie viele Leben übrig sind. Leben zählen nur in normalen Kneipen; der Schutzengel hilft in der Boss-Kneipe nicht.
4. **Gewonnen lohnt es sich:** doppelte Auszahlung, ein Leben (höchstens 5) und ein Relikt nach Wahl aus drei der Seltenheit Ungewöhnlich oder höher. Sind alle Plätze belegt, macht ein eigenes Relikt Platz (für den halben Preis verkauft), oder der Spieler verzichtet. Nach dem letzten Boss ist der Run gewonnen, ohne Wahl.
5. **Fünf Bosse und der Alte.** Kontra-Hans, Revolutionär und Kiebitz-Mörder entfallen. Es bleiben Schieber, Oma, Zwillinge, Ramscher und Schwarzer Peter; der Alte legt ihre fünf Regeln der Reihe nach auf.
6. **Neue Regeln:** Die Oma legt zu Beginn ihrer Kneipe auf die Hälfte der Ausgabe des Spielers (16 von 32 Karten) je einen zufälligen Fluch (Durchsichtig, Ramponiert, Fleckig) für diese Kneipe; ein solcher Fluch tritt an die Stelle der Modifikation, danach ist die Ausgabe genau wie vorher, Groschen gibt es dafür nicht. Beim Ramscher ist jedes Spiel ein Ramsch.
7. **Der Totengräber** (Boss mit Sonderkarten) ruht.

## Alternativen

- **Bei einem Boss-Platz bleiben, nur die Belohnung erhöhen:** verworfen, der Boss bliebe eine Regel mit einem Stammgast daneben.
- **Ein Sekundant mit derselben Ausgabe:** einfacher (eine Ausgabe je Boss), aber das Paar wirkte wie ein Klon. Zwei Ausgaben zum selben Thema geben jeder Figur ein Gesicht und bleiben lesbar.
- **Verlorene Boss-Kneipe kostet zwei Leben:** milder, aber die Boss-Kneipe bliebe wiederholbar; der Nutzer wollte einen echten Höhepunkt.
- **Die drei schwachen Bosse überarbeiten statt streichen:** verschoben. Neue Bosse kommen später als Daten; der Totengräber ist der erste Kandidat.

## Folgen

- **Im Kern** sitzen die Figuren auf `BOSS_SEAT` (links, `Owner::Boss`) und `PARTNER_SEAT` (rechts, neu `Owner::BossPartner`); die Ausgaben hängen am Besitzer (`ActiveGame::boss_edition`, `partner_edition`). Neu: `RunPhase::BossReward`, `RunAction::ChooseBossRelic`, die Ereignisse `BossRewardOffered`, `BossRewardDeclined`, `BossCursesLaid`, `BossCursesLifted`, `RunLost::boss`, `Payout::boss`, die Wirkungen `CurseForPub` und `Ramsch` (alter Name `RamschOnce`), `boss_reward` in `run.ron`. Entfallen: `SwapDefenderHands`, `DisableInformation`, `BlankOut`; das Spielereignis des Handtauschs bleibt nur lesbar, damit alte Spielprotokolle laden.
- **Spielstände:** Ein Stand, der einen entfernten Boss gezogen hat, trifft einen Ersatz (`RETIRED_BOSSES`: Kontra-Hans und Revolutionär → Zwillinge, Kiebitz-Mörder → Schwarzer Peter); ein laufendes Spiel behält seine Karten. Keine neue Spielstand-Version.
- **Freischaltungen:** „Trocken gekontert“ (`beat_kontra_hans`) heißt jetzt „Doppelt gesehen“ und verlangt die Zwillinge; die Id bleibt, die Stammfarbe bleibt erreichbar.
- **Alle Boss-Faktoren sind vorläufig** (nach Augenmaß für drei volle Spiele gegen zwei Boss-Spieler, DESIGN 11), weil die Simulationen ruhen. Zu messen, sobald der Simulator wieder läuft (`skat_sim --curve --boss <id>` misst jetzt den Stand nach dem letzten Spiel).
- **Schwerer wird der Run:** Eine Boss-Kneipe ist ein Alles-oder-nichts; ob die Kurve der normalen Kneipen dafür milder werden muss, zeigt erst die Messung.
- **Die Web-App** zeigt beide Figuren mit Namensschild und Boss-Rückseite, nennt sie vor der Boss-Kneipe (Weg, Boss-Chip, Tresen, Boss-Dialog), hat eine Auswahl für das Boss-Relikt und sagt beim verlorenen Run, dass die Boss-Kneipe ihn beendet hat.

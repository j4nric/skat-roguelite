# Glossar: Skat-Begriffe und ihre Bezeichner im Code

Die Domäne ist deutsch, der Code ist englisch. Diese Tabelle ist verbindlich. Wer einen neuen Begriff braucht, trägt ihn hier ein, bevor er ihn im Code verwendet. Keine Synonyme (kein `unter` neben `jack`, kein `points` neben `card_points`).

## Karten

| Deutsch | Bezeichner | Anmerkung |
|---|---|---|
| Blatt (das ganze Spiel, 32 Karten) | `deck` | Im Roguelike: das geteilte, modifizierbare Blatt |
| Blatt (die Karten eines Spielers) | `hand` | |
| Karte | `Card` | |
| Farbe | `Suit` | |
| Karo, Herz, Pik, Kreuz | `Diamonds`, `Hearts`, `Spades`, `Clubs` | Reihenfolge aufsteigend nach Grundwert |
| Wert / Bild | `Rank` | |
| Sieben, Acht, Neun, Zehn | `Seven`, `Eight`, `Nine`, `Ten` | |
| Bube | `Jack` | im Code nie `Unter`, nie `Wenzel`. In der Anzeige heißt er Unter (ADR 0010). |
| Dame | `Queen` | im Code nie `Ober`. In der Anzeige heißt sie Ober (ADR 0010). |
| König | `King` | |
| Ass | `Ace` | |
| Augen | `card_points` | Summe im Blatt: 120 |
| Lusche (7, 8, 9) | `blank` | Karte ohne Augen |
| Blanko-Karte (Roguelike) | `CardKind::Blank` | Ohne Farbe, ohne Augen, verliert immer, bedient nie, darf immer gespielt werden. Ersetzt entfernte Karten. Notation `XX`. |
| Kartenart (normal oder Blanko) | `CardKind` | `CardKind::Normal(Card)`, `CardKind::Blank` |
| Karteninstanz im Blatt des Runs | `DeckCard` | `id`, `kind`, `mods` |
| Karteninstanz in einem Spiel | `DealtCard` | Mit aufgelösten Augen, Rangbonus und Sichtbarkeit |
| Karte im Spiel (Regel-Sicht) | `PlayingCard` (Trait) | Von `Card` und `DealtCard` implementiert |
| Rangbonus (Gezinkt) | `rank_bonus` | Stufen höher innerhalb von Trumpf oder Farbe; bei Gleichstand behält die frühere Karte den Stich |
| Bube ehrenhalber | `honorary_jack` | `PlayingCard::honorary_jack`, Effekt `HonoraryJack`: im Farbspiel Trumpf eine halbe Stufe unter dem Karo-Buben, keine Spitze (DESIGN 4.4). In der Anzeige „Unter ehrenhalber“ |
| Platz in der Trumpfordnung | `trump_order_key` | Stärke, dann 0 für den Buben ehrenhalber, 1 für gedruckte Trümpfe |
| Darf nicht gedrückt werden (Ramponiert) | `no_discard` | `PlayingCard::no_discard`, Effekt `NoDiscard`, Prüfung `valid_discard`, Fehler `DiscardForbidden`; je Handkarte `discardable` (`rules::discardable`, `CardView::discardable`) |
| Volle (Ass, Zehn) | `high_card` | |
| Trumpf | `trump` | |
| Skat (die zwei Karten) | `skat` | nie `talon`, siehe unten |
| Talon (Roguelike: Karten über 32, nicht im Spiel) | `talon` | nur im Roguelike-Kontext |
| Gezinkte Karte / Kartenmodifikation | `CardMod` | Definition `CardModDef`, Id `CardModId` |
| Im Spiel (die 32 gegebenen Karten) | `cards_in_play` | Talon gehört nicht dazu |

## Spielablauf

| Deutsch | Bezeichner | Anmerkung |
|---|---|---|
| Geben | `deal` | |
| Vorhand, Mittelhand, Hinterhand | `Forehand`, `Middlehand`, `Rearhand` | Sitzposition relativ zum Geber |
| Alleinspieler | `declarer` | Immer der menschliche Spieler |
| Gegenspieler | `defender` | Die zwei KI-Spieler |
| Reizen | entfällt | Ersetzt durch Ansage, siehe ADR 0006 |
| Ansage | `Declaration` | Enum: `Suit { suit, risk }`, `Grand { risk }`, `Null { risk: NullRisk }`, `Ramsch`. Ungültige Kombinationen gibt es nicht; JSON-Form `{game, risk}` |
| Null-Risikostufe | `NullRisk` | `Normal` (23), `Hand` (35), `Ouvert` (46), `OuvertHand` (59); Stufe 1 bis 4 (`step`) |
| Ramsch | `Declaration::Ramsch` | Boss „Der Ramscher“: Buben Trumpf, jeder für sich, die meisten Augen verlieren |
| Jungfrau (Ramsch: Spieler ohne Stich) | `virgin` | Feld in `LevelParts`; nicht verwechseln mit der Hausregel `virgin` |
| Augen und Stiche pro Platz | `SeatTotals` | `card_points`, `tricks` (Index = Platz), `most` (die meisten Augen, im Ramsch die Verlierer); in `ScoreBreakdown::seats`, `GameFinished::seats`; Sicht `seat_tricks`, Zähler `count.seats` |
| Spielart | `GameType` | `Suit(Suit)`, `Grand`, `Null` |
| Farbspiel | `GameType::Suit` | |
| Hand (Skat nicht aufnehmen) | `hand` (bool) | |
| Skat aufnehmen | `pick_up_skat` | |
| Drücken | `discard` | Die zwei Karten, die in den Skat gelegt werden |
| Skattausch (Oma, Opa) | `skat_swap` | `GameRules::skat_swaps` (Liste von Filtern, je einer pro Figur und Serie), `Series::skat_swaps_taken`, Sicht `skat_swap_cards`, Aktion `TakeFromSkat { give, take }` |
| Ausspielen | `lead` | Erste Karte eines Stichs |
| Bedienen | `follow_suit` | |
| Stechen | `ruff` / `trump` (verb) | Mit Trumpf übernehmen |
| Abwerfen | `slough` | Nicht bedienen können und Nicht-Trumpf legen |
| Schmieren | `smear` | Volle in den Stich des Partners geben |
| Stich | `Trick` | |
| Stichgewinner | `trick_winner` | |
| Passen / Einpassen | `pass` | Im Roguelike: ein Spiel auslassen |
| Kontra | `kontra` | Gegenspieler verdoppeln (Hausregel, Boss) |
| Re | `re` | Antwort des Alleinspielers auf Kontra |
| Schenken (Gegner geben auf) | `concede` | Rest der Stiche und Augen geht an den Alleinspieler |
| Schieben (Hausregel: neu geben lassen) | `redeal` | `RunAction::Redeal`, einmal pro Serie vor der Ansage, alle Hände und Skat neu, kostet kein Spiel (DESIGN 8.1). Effekt `SeriesRedeal` |
| Neu geben (Schieben, „Neues Blatt“) | `redeal` | Ereignis `Redealt` |
| Skat neu ziehen (nur Spielebene) | `redraw_skat` | `Game::redraw_skat`: Gegner und Skat neu, eigene Hand bleibt. Von keinem Inhalt mehr genutzt |

## Wertung

| Deutsch | Bezeichner | Anmerkung |
|---|---|---|
| Spielwert | `game_value` | Grundwert × Gewinnstufen |
| Grundwert | `base_value` | Karo 9, Herz 10, Pik 11, Kreuz 12, Grand 24 |
| Gewinnstufen | `levels` | Spitzen + 1 + Hand + Schneider + … |
| Spitzen / Matadore | `matadors` | `with: bool` für „mit“ oder „ohne“ |
| mit / ohne | `with` / `without` | |
| Schneider | `schneider` | Eine Seite mit höchstens 30 Augen (`SCHNEIDER_LIMIT`). Gemessen an der verlierenden Seite, damit es auch bei veränderten Blättern gilt. Schwarz schließt Schneider immer ein. |
| Schwarz | `schwarz` | Alle Stiche |
| angesagt | `_declared` | `schneider_declared`, `schwarz_declared` |
| Ouvert | `ouvert` | Offen spielen |
| Gewonnen | `won` | Ab 61 Augen |
| Gewinnschwelle, Schneidergrenzen | `Thresholds` | `win`, `schneider_limit`, `declarer_schneider` |
| Spitzen im Spiel | `matadors_in` | Gezählt nur über Trümpfe, die im Spiel sind |
| Verloren | `lost` | Zählt doppelt negativ |
| Null, Null Hand, Null Ouvert, Null Ouvert Hand | `null_value(hand, ouvert)` | 23, 35, 46, 59 |
| Null-Wertung im Run | `NullScoring` | `Fixed` (Skatordnung) oder `Formula(NullFormula)`, Daten pro Charakter |
| Stilbonus (Null) | `NullStyle` | `sevens_ducked` (mit einer Sieben unterlaufen), `never_followed` (nie bedienen müssen), `short_circuit` (Kurzschluss). Bedingungen `SevenDucked`, `NeverFollowedSuit` |
| Mindestens mit/ohne n Spitzen | `MatadorsAtLeast { with, count }` | Bedingung ab der Ansage |
| Kann nie stechen (Unsichtbar) | `never_wins` | `PlayingCard::never_wins`, Effekt `NeverWins` |

## Roguelike

| Deutsch | Bezeichner | Anmerkung |
|---|---|---|
| Run / Tournee | `Run` | |
| Abschnitt / Skatabend | `Section` | Ein Ante |
| Serie | `Series` | Ein Blind: Zielwert plus Spiellimit |
| Zielwert | `target` | |
| Zielwert-Faktor (Boss) | `target_factor` | Typ `Factor`, in RON und JSON als Dezimalzahl, intern Tausendstel |
| Boss angekündigt (Start eines Abschnitts) | `BossAnnounced` | Run-Ereignis; Boss des Abschnitts `section_boss` |
| Bevorstehender Boss | `upcoming_boss` | Feld der Sicht |
| Nächste Serie (Vorschau) | `next_series` | Feld der Sicht, `NextSeriesView` |
| Spiellimit | `game_limit` | |
| Boss-Serie | `BossSeries` | |
| Hausregel | `HouseRule` | Passives Relikt, Joker-Äquivalent |
| Stammtisch-Figur | `Regular` | Passives Relikt mit Persönlichkeit |
| Stammgast (normaler Gegner am Tisch) | `Guest` | Name, Gesicht (`look`) und Sprüche (`quips`: `greet`, `win`, `lose`), keine Regeln. Daten `GuestDef` in `guests.ron`, Id `GuestId`; zwei pro Serie in `Run::guests`, Sicht `SeriesView::guests` (`GuestInfo`). Nicht `Regular`, das ist die Stammtisch-Figur |
| Verbrauchsgegenstand | `Consumable` | Definition `ConsumableDef` |
| Boss (Gegnerpaar mit Sonderregel) | `Boss` | Definition `BossDef`, in `Series::boss` |
| Seltenheit | `Rarity` | `Common`, `Uncommon`, `Rare` |
| Preis | `price` | Groschen |
| Verkaufen (halber Preis) | `sell` | |
| Neuauslage (Tresen) | `reroll` | |
| Grundbetrag (gewonnene Serie) | `series_reward` | |
| Zinsen | `interest` | |
| Auszahlung einer Serie | `Payout` | Grundbetrag, Bonus für übrige Spiele, Zinsen |
| Run-Aktion (Entscheidung des Spielers) | `RunAction` | |
| Run-Ereignis | `RunEvent` | |
| Flagge (Relikt-Zustand, z. B. Bock) | `flag` | Pro Run |
| Zähler (Relikt-Zustand, z. B. Tournee) | `counter` | Pro Run |
| Bock | `bock` | Flagge der Bockrunde |
| Relikt (Oberbegriff) | `Relic` | HouseRule und Regular, Definition `RelicDef` |
| Tresen | `Shop` | |
| Groschen | `coins` | Währung |
| Groschen erhalten (Effekt) | `GainCoins` | Ereignis `CoinsGained { source, amount, coins }`; negativ zieht ab, nie unter 0 |
| Beim Erhalt (Kartenmodifikation angebracht) | `OnAttach` | Auslöser nur für Kartenmodifikationen, Stufe `Stage::OnAttach` |
| Fluch (negative Kartenmodifikation) | `curse` | Feld `CardModDef::curse`; Text beginnt mit „Fluch:“ |
| Ereignis / Rast | `Encounter` | Nicht `Event` (das sind die Zustandsereignisse). Definition `EncounterDef`, offen `ActiveEncounter`, Option `EncounterOption`, Wahl `EncounterChoice`, Aktion `RunAction::ChooseEncounter` |
| Hausregel des Abends (Stammtisch-Streit) | `evening_relics` | Gilt bis zum Ende des Abschnitts, ohne Platz |
| Charakter | `Character` | `Declarer` (Alleinspieler), `NullPlayer` (Nullspieler); Daten `CharacterDef` |
| Startblatt (Abweichung vom 32er-Blatt) | `DeckSpec` | `remove`, `add` |
| Freischaltung | `Unlock` | Definition `UnlockDef`, Id `UnlockId`, Bedingung `UnlockCondition`, Belohnung `Reward` |
| Meta-Zustand (über Runs) | `MetaState` | Freischaltungen und Statistik, getrennt vom Run gespeichert |
| Gesperrt | `locked` | Relikte, die noch Belohnung einer Freischaltung sind |
| Pool eines Charakters | `characters` | Feld an Relikten, Modifikationen, Verbrauchsgegenständen, Bossen und Ereignissen; leer = alle |
| Gesperrte Modifikation | `locked_card_mods` | Belohnung `Reward::CardMod` einer Freischaltung, wird bis dahin nicht zufällig gezogen |
| Art und Seltenheit einer Belohnung | `RewardInfo`, `RewardKind` | `HouseRule`, `Regular`, `CardMod`, `Character`; Feld `reward_info` im Meta-View |
| Endboss | `final_boss` | „Der Alte“, Regeln nacheinander: `rotation` |
| Regel des Bosses in diesem Spiel | `boss_rule` | Beim Alten wechselt sie pro Spiel |
| Schummeln (Boss-Regel) | `is_cheat` | Zwillinge, Revolutionär; abgeschaltet durch `NoCheating` |
| Informations-Effekt | `is_information` | Durchsichtig, Kiebitz, Zähler; abgeschaltet durch `DisableInformation` |
| Rabatt am Tresen (Wirt) | `shop_price`, `ShopDiscount` | |
| Kiebitz-Blick | `peek`, `CardPeeked` | |
| Herkunft einer aufgedeckten Karte | `RevealSource` | `CardMod(id)` (Durchsichtig, Bierfleck, Attrappe) oder `Kibitz`; Feld `RevealedCard::source` |
| Risikostufe | `RiskLevel` | Normal, Hand, SchneiderDeclared, SchwarzDeclared, Ouvert |
| Statistik eines Runs | `RunStats` | Gewonnene, verlorene, eingepasste Spiele, Serien, bestes Spiel |
| Verkaufspreis | `sell_price` | Halber Preis |
| Sicht (Lese-Sicht für die Oberfläche) | `RunView` | `Run::view`, alles fertig berechnet; dazu `GameView`, `ShopView`, `CardView` |
| Spieler-Ereignis (ohne verdeckte Karten) | `PlayerEvent` | `RunEvent<PlayerGameEvent>`, aus `RunEvent::for_player`; nur das geht an den Browser |
| Start eines Runs | `RunStart` | Start-Relikte, erste Serie; `Run::new_with` |
| Fehlercode | `RunErrorCode` | Stabiler Name eines `RunError`, `RunError::code` |
| Wahrscheinlichkeit beim Geben | `DealOdds` | Wo eine Karte des Blatts landet (DESIGN 7.4), `Odds` als Bruch und Promille |
| Gegenspieler-Stufe (KI je Abschnitt) | `Opponents` | `Heuristic` oder `Pimc { samples, depth }`, Liste `opponents` in `run.ron` (DESIGN 10.3) |
| Perfect Information Monte Carlo | `PimcPolicy`, `PimcConfig` | KI-Stufe 2 (DESIGN 10.2), `ai::pimc` |
| KI am Zug (ein Gegenspieler denkt) | `ai_to_move` | `Run::ai_to_move`, Sicht `RunView::ai_to_move`; ein KI-Zug `Run::advance_ai` (wasm `step`), nur die Spieleraktion `Run::apply_step`, bis zur nächsten Spielerentscheidung `Run::apply`; Fehler `AiToMove` |
| Stichprobe (eine zufällige, stimmige Verteilung der unbekannten Karten) | `Sample` | `Knowledge::sample`, `ai::sample` |
| Wissen eines Sitzes über die Kartenverteilung | `Knowledge` | Fehlfarben, offene Hände, festgelegte Karten (`Hints::pinned`) |
| Fehlfarbe (Farbe, die ein Sitz nicht mehr hat) | `void` | Aus Nicht-Bedienen erschlossen |
| Double-Dummy-Suche (alle Karten offen) | `ai::solver`, `solve`, `probe_with` | Alpha-Beta mit Transpositionstabelle |
| Suchtiefe in Stichen | `depth` | Dahinter eine Ausspiel-Schätzung (`estimate`) |

## Hook-API und Wertung mit Relikten

| Deutsch | Bezeichner | Anmerkung |
|---|---|---|
| Auslöser (Zeitpunkt) | `Hook`, `Stage` | `OnDeal`, `OnDeclare`, `OnCardPlayed`, `OnTrickEnd`, `OnScore`, `OnSeriesWon`, dazu `Passive` |
| Effekt | `Effect` | Geschlossener `enum`, jeder Effekt nennt seinen Hebel |
| Bedingung | `Condition` | Geschlossener `enum` |
| Kartenfilter | `CardFilter` | Farbe und/oder Wert |
| Hebel der Formel | `Lever` | `BaseValue`, `Levels`, `GameValue`, `CardPoints`, `WinThreshold`, `Schneider` |
| Herkunft eines Zuschlags | `Source` | Relikt, Kartenmodifikation, Boss, Kontra, Re |
| Zeile der Abrechnung | `ScoreLine` | Herkunft, Hebel, Änderung |
| Abrechnung (Spielwert mit allen Zuschlägen) | `ScoreBreakdown` | |
| Gewinnstufen Term für Term | `LevelParts` | |
| Schritt der Abrechnung | `ScoreStep` | Ein Term mit laufendem Grundwert, Stufen und Faktor, für die schrittweise Anzeige |
| Term der Abrechnung | `ScoreTerm` | Grundwert, Spitzen, Spiel, Hand, …, oder eine `ScoreLine` |

## Content-IDs

Content-IDs sind englisch und snake_case. Die deutschen Namen stehen in den RON-Dateien.

| Deutsch | ID |
|---|---|
| Bockrunde | `bock_round` |
| Kontra | `kontra` |
| Spitze (Hausregel, letzter Stich mit einer Sieben) | `last_trick_seven` (nicht verwechseln mit Spitzen = `matadors`) |
| Schenken | `concede` |
| Ouvert-Bonus | `ouvert_bonus` |
| Schieben | `skat_redraw` |
| Jungfrau | `virgin` |
| Alte Dame | `old_lady` |
| Tournee | `tour` |
| Durchmarsch | `march_through` |
| Fett | `fat` |
| Gezinkt | `marked` |
| Stempel | `stamp` |
| Spitze (Kartenmodifikation) | `last_trick` |
| Durchsichtig | `transparent` |
| Der Schieber | `the_pusher` |
| Der Kontra-Hans | `kontra_hans` |
| Die Oma | `grandma` |
| Frisches Deck | `fresh_deck` |
| Stempelkissen | `ink_pad` |
| Ein Bier | `beer` |
| Neues Blatt | `new_deal` |
| Blindflug | `blind_flight` |
| Der Kiebitz | `kibitzer` |
| Der Wirt | `landlord` |
| Die Oma (Figur) | `granny` (nicht verwechseln mit dem Boss `grandma`) |
| Der Schiedsrichter | `referee` |
| Der Angeber | `show_off` |
| Der Zähler (Figur) | `card_counter` (nicht `counter`, das ist der Relikt-Zustand) |
| Unsichtbar | `invisible` |
| Die Zwillinge | `twins` |
| Der Ramscher | `ramscher` |
| Der Revolutionär | `revolutionary` |
| Der Schwarze Peter | `black_peter` |
| Der Kiebitz-Mörder | `kibitz_killer` |
| Der Alte | `the_old_man` |
| Stammtisch-Streit | `stammtisch_dispute` |
| Der Kartengeber | `card_dealer` |
| Alter Bekannter | `old_friend` |
| Letzte Runde | `last_round` |
| Kleinvieh | `every_bit_counts` |
| Ein Auge zudrücken | `blind_eye` |
| Lehrgeld | `tuition` (Zähler `tuition`) |
| Bierdeckel | `beer_mat` (Zähler `beer_mat`) |
| Auf dem Trockenen | `high_and_dry` |
| Schneiderbock | `schneider_bock` (Flagge `schneider_bock`) |
| Glückssträhne | `lucky_streak` (Flaggen `streak_start`, `streak`) |
| Der Schneider (Figur) | `tailor` (nicht verwechseln mit Schneider = `schneider`) |
| Der Stammgast | `regular_guest` (Zähler `regular`) |
| Nullachtfuffzehn | `null_eight_fifteen` |
| Doppelter Boden | `safety_net` |
| Dicke Sau | `fat_sow` |
| Glashaus | `glass_house` |
| Kerbholz | `tally_stick` (Zähler `tally`) |
| Goldrand | `gold_rim` |
| Doppelt gezinkt | `double_marked` |
| Krone | `crown` |
| Siegel | `seal` |
| Hufeisen | `horseshoe` |
| Maßband | `tape_measure` |
| Glückspfennig | `lucky_penny` |
| Bierfleck | `beer_stain` |
| Angeknackst | `cracked` |
| Schwarze Katze | `black_cat` |
| Bleigewicht | `lead_weight` |
| Schlusslicht | `tail_light` |
| Aufgebrezelt | `dressed_up` |
| Attrappe | `dummy` |
| Umgekipptes Bier | `spilled_beer` |
| Gasthof zur Krone | `crown_inn` |
| Der Taschenspieler | `conjurer` |
| Preisskat | `prize_skat` |
| Gast aus Altenburg | `guest_from_altenburg` |
| Ramponiert | `battered` |
| Bube ehrenhalber (Anzeige: Unter ehrenhalber) | `honorary_jack` |
| Der Opa (Figur) | `grandpa` (nicht verwechseln mit `granny`) |
| Mit Vieren | `with_four` |
| Siebenschläfer | `seven_sleeper` |
| Saubere Weste | `clean_slate` |

## Kartennotation

Zwei Zeichen, Farbe dann Wert. Wird in Tests, Logs und Datendateien verwendet.

```
Farben:  D = Karo   H = Herz   S = Pik   C = Kreuz
Werte:   7 8 9 T J Q K A
Beispiel: CJ = Kreuz-Bube, DT = Karo-Zehn, HA = Herz-Ass
Blanko:  XX
```

## Anzeige im deutschen Blatt (ADR 0010)

Nur für Spieltexte und Grafik. Code, Tests und Design-Dokument bleiben bei den Bezeichnern oben.

| Code | Anzeige |
|---|---|
| `Clubs` | Eichel |
| `Spades` | Grün |
| `Hearts` | Rot |
| `Diamonds` | Schellen |
| `Ace` | Daus |
| `Ten` | Zehn |
| `King` | König |
| `Queen` | Ober |
| `Jack` | Unter |
| `Nine`, `Eight`, `Seven` | Neun, Acht, Sieben |

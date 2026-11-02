# Glossar: Skat-Begriffe und ihre Bezeichner im Code

Die Domäne ist deutsch, der Code ist englisch. Diese Tabelle ist verbindlich. Wer einen neuen Begriff braucht, trägt ihn hier ein, bevor er ihn im Code verwendet. Keine Synonyme (kein `unter` neben `jack`, kein `points` neben `card_points`).

## Karten

| Deutsch | Bezeichner | Anmerkung |
|---|---|---|
| Blatt (das ganze Spiel, 32 Karten) | `deck` | Im Roguelike bis ADR 0015 das geteilte, modifizierbare Blatt; seit ADR 0015 immer das normale Skatblatt aus 32 verschiedenen Karten (`Game::deck_cards`: jede Karte in der Ausgabe ihres Halters) |
| Ausgabe (ADR 0015) | `Edition` | Die Version jeder Karte, die ein Spieler besitzt: je Karte die Modifikationen (`core/src/deck.rs`). Ausgabe des Spielers `Run::edition` (ersetzt `Run::deck`, Sicht `RunView::edition`); Gegner: normal (Stammgast) oder Boss-Ausgabe (`edition` in `bosses.ron`, `Edition::with_mods`, `ActiveGame::boss_edition`); `Editions` bündelt die beiden für die Hooks |
| Halter einer Karte | `holder` | Der Platz, der die Karte gerade hält; die Karte zeigt seine Ausgabe. Der Skat gehört dem Alleinspieler. Wechsel nur über `Game::change_holder`, Ereignis `CardChangedHands`; `Game::in_edition_of`, `Game::current` |
| Besitzer einer Ausgabe | `owner` | `DealtCard::owner` (`Owner`: `Player`, `Guest`, `Boss`): wessen Ausgabe die aufgelöste Karte ist, für Sicht, Rückseite und Hooks; `Run::owner_of_seat`, Platz des Bosses `BOSS_SEAT` |
| Rückseite | `back` | Kosmetik; Spieler: gewählte Rückseite (Standard Distel, `web/src/ui/backs.ts`), Gegner: `back` in `guests.ron` (Standard `opponent`, `OPPONENT_BACK`) und `bosses.ron` (Kiebitz-Mörder `kibitz_killer`); Sicht `CardView::back`, `GameView::backs`, `BossInfo::back` |
| Boss-Ausgabe (Zeile) | `EditionEntry` | Kartenfilter und Modifikation, Sicht `BossInfo::edition` |
| Scherben (zerbrochener Glasdaus, ADR 0015) | `shards` | Nicht umgesetzt (der Glasdaus ist mit Effekte v2 gestrichen) |
| Abgestuft (Nullspieler, ADR 0015) | `demoted` | Spezialkarte, die ein Fluch ist (`ModKind::SpecialCurse`), Wirkung `Demote`, `PlayingCard::demoted`, `DealtCard::demoted`: ein Unter ist keine Unter mehr, sondern die niedrigste Karte seiner Farbe ohne Augen (`rules::is_demoted_jack`), jede andere Karte sticht eine Stufe tiefer |
| Reinigen (Frisches Deck, ADR 0015) | `cleanse` | Alle Modifikationen einer Karte der Ausgabe fallen ab: Aktion `ConsumableAction::CleanseCard`, `Edition::cleanse`, `DeckChange::Cleansed`, Fehler `DeckError::NothingToCleanse` |
| Alter Spielstand (vor ADR 0015 oder 0016) | `OutdatedSave` | Fehler beim Laden eines laufenden Runs der Spielstand-Version 1 oder 2; der Run wird verworfen (`SAVE_VERSION` 3 seit ADR 0016) |
| Blatt (die Karten eines Spielers) | `hand` | |
| Karte | `Card` | |
| Farbe | `Suit` | |
| Karo, Herz, Pik, Kreuz | `Diamonds`, `Hearts`, `Spades`, `Clubs` | Reihenfolge aufsteigend nach Grundwert |
| Wert / Bild | `Rank` | |
| Sieben, Acht, Neun, Zehn | `Seven`, `Eight`, `Nine`, `Ten` | |
| Bube | `Jack` | im Code nie `Unter`, nie `Wenzel`. In der Anzeige heißt er Unter (ADR 0010). |
| Dame | `Queen` | im Code nie `Ober`. In der Anzeige heißt sie Oberin (ADR 0010, 0014). |
| König | `King` | |
| Ass | `Ace` | |
| Augen | `card_points` | Summe im Blatt: 120 |
| Lusche (7, 8, 9) | `blank` | Karte ohne Augen |
| Blanko-Karte (Roguelike) | `CardKind::Blank` | Ohne Farbe, ohne Augen, verliert immer, bedient nie, darf immer gespielt werden. Ersetzte bis ADR 0015 entfernte Karten; seit ADR 0015 nur noch als Karteneffekt (Oma, Scherben). Notation `XX`. |
| Kartenart (normal oder Blanko) | `CardKind` | `CardKind::Normal(Card)`, `CardKind::Blank` |
| Karte einer Ausgabe | `DeckCard` | `id`, `kind`, `mods`, `mod_values` (Wert je Modifikation, ohne Eintrag 1; Effekte v2) |
| Karteninstanz in einem Spiel | `DealtCard` | Mit aufgelösten Augen, Rangbonus und Sichtbarkeit, in der Ausgabe ihres Halters (`owner`) |
| Karte im Spiel (Regel-Sicht) | `PlayingCard` (Trait) | Von `Card` und `DealtCard` implementiert |
| Rangbonus (Modifikation Rang, bis Effekte v2 Gezinkt) | `rank_bonus` | Stufen höher innerhalb von Trumpf oder Farbe; bei Gleichstand behält die frühere Karte den Stich |
| Bube ehrenhalber | `honorary_jack` | `PlayingCard::honorary_jack`, Effekt `HonoraryJack`: im Farbspiel Trumpf eine halbe Stufe unter dem Karo-Buben, keine Spitze (DESIGN 4.4). In der Anzeige „Unter ehrenhalber“. Seit Effekte v2 ohne Inhalt, die Wirkung ruht |
| Platz in der Trumpfordnung | `trump_order_key` | Stärke, dann 0 für den Buben ehrenhalber, 1 für gedruckte Trümpfe |
| Darf nicht gedrückt werden (bis Effekte v2 der Fluch Ramponiert, seitdem ruhend) | `no_discard` | `PlayingCard::no_discard`, Effekt `NoDiscard`, Prüfung `valid_discard`, Fehler `DiscardForbidden`; je Handkarte `discardable` (`rules::discardable`, `CardView::discardable`) |
| Volle (Ass, Zehn) | `high_card` | |
| Trumpf | `trump` | |
| Skat (die zwei Karten) | `skat` | nie `talon`, siehe unten |
| Talon (Roguelike: Karten über 32, nicht im Spiel) | `talon` | entfallen mit ADR 0015 (M9), im Code gestrichen |
| Gezinkte Karte / Kartenmodifikation | `CardMod` | Definition `CardModDef`, Id `CardModId` |
| Im Spiel (die 32 gegebenen Karten) | `cards_in_play` | immer alle 32 Karten (kein Talon seit ADR 0015) |
| Wandelkarte (nimmt bei Spielbeginn die längste Farbe der Hand an) | `wild` | `PlayingCard::wild`, `DealtCard::wild` (nur die Karten des Spielers, solange sie noch offen ist), Effekt `Wild`. Farbe: `rules::settled_wild_suit` (meiste Karten nach gedruckter Farbe, ohne Unter und Wandelkarten; Gleichstand Eichel, Grün, Rot, Schellen); Sicht `CardView::unsettled` (noch offen), `CardView::named` (gedruckte Farbe), `ActiveGame::named` |
| festlegen (Wandelkarte: ihre Farbe bei Spielbeginn) | `settle` | `Game::declare` legt fest, `PlayingCard::settle_wild`; Ereignis `WildCardSettled { old, new }` (Spielebene, auch `PlayerGameEvent`) |
| Untot (Karte steigt aus dem Stich zurück in die Hand) | `undead` | Effekt `Undead`, Prüfung `Run::can_rise`, verbraucht `ActiveGame::risen` (einmal pro Spiel), Zufall `ActiveGame::undead_rng`; Spielebene `Game::raise`, Ereignis `CardRose` (`card` zurück in der Hand, `substitute` im Stich); Sicht `CardView::can_rise`; Bot `RunBot::undead_card` |
| Ersatzkarte (Untot: die Karte, die an ihrer Stelle im Stich liegt) | `substitute` | Feld von `CardRose` |
| Nur auf (Modifikation passt nur auf bestimmte Karten) | `only_on` | Feld `CardModDef::only_on` (`CardFilter`), Prüfung `CardModDef::fits` |
| Effekte v2 (System der Kartenmodifikationen seit 02.11.2026) | – | DESIGN 7.2 |
| Art einer Modifikation | `ModKind` | `Edge` (Rand: immer aktiv), `Moment` (Markierung oben rechts), `Voucher` (Gutschein), `Curse` (Fluch), `Special` (Spezialkarte); Feld `CardModDef::kind` |
| Rand-Modifikation | `ModKind::Edge` | Augen, Rang, Trumpf |
| Moment-Modifikation | `ModKind::Moment` | Letzter Stich, Erster Stich |
| Gutschein (wenn gedrückt) | `ModKind::Voucher` | Auslöser `OnDiscard`, Ereignis `VoucherRedeemed { card, card_mod, value }`, Wirkung `TurnInto` (wird Spezialkarte) |
| Spezialkarte | `ModKind::Special` | Untot, Wandelkarte; darf zusätzlich eine normale Modifikation tragen |
| Wert einer Modifikation | `value` | An der Karte `DeckCard::mod_values`, angeboten `CardModDef::values`, Obergrenze `CardModDef::max`, Wirkung je Einheit mal Wert `Effect::scaled` |
| Steigerung (dieselbe Modifikation noch einmal) | `upgrade` | Werte addieren, Rang bis 3; Ergebnis `DeckChange::Modified { value, replaced: None }` |
| Ersetzen (andere Modifikation) | `replace` | `DeckChange::Modified.replaced` nennt die alte |
| Verflucht (Karte kann nicht verändert werden) | `cursed` | Fehler `DeckError::Cursed` |
| Grenze erreicht | `AtMaximum` | Fehler `DeckError::AtMaximum` (Rang 3) |
| Schuld (Augen unter 0) | `point_debt` | `DealtCard::point_debt`, `PlayingCard::point_debt`; zieht der Stich ab, die Augen einer Seite fallen nie unter 0 |
| Zuschlag auf Augen (Erster Stich) | `bonus_points` | `Game::bonus_points`, `Game::add_bonus_points`; Wirkung `MultiplyTrickPoints`, Bedingung `ThisCardWinsFirstTrick` |
| Trumpf-Modifikationen nur beim Spieler | `TRUMP_MODS_NEED_PLAYER` | Schalter in `content::engine` für `ThisCardIsTrumpSuit` (ADR 0015 streng) |
| Version der Modifikationen eines Blatts | `mods_version` | Feld von `Deck`; ohne 2 wird beim Laden umgerechnet (`deck::migrate_legacy_mods`) |

## Spielablauf

| Deutsch | Bezeichner | Anmerkung |
|---|---|---|
| Geben | `deal` | |
| Vorhand, Mittelhand, Hinterhand | `Forehand`, `Middlehand`, `Rearhand` | Sitzposition relativ zum Geber |
| Alleinspieler | `declarer` | Immer der menschliche Spieler |
| Gegenspieler | `defender` | Die zwei KI-Spieler |
| Reizen | entfällt | Ersetzt durch Ansage, siehe ADR 0006 |
| Ansage | `Declaration` | Enum: `Suit { suit, risk, hand }`, `Grand { risk, hand }`, `Null { risk: NullRisk }`, `Ramsch`. Ungültige Kombinationen gibt es nicht; JSON-Form `{game, risk}`, nach dem Skat angesagt zusätzlich `after_skat: true` |
| Angesagt nach dem Skat (ADR 0013) | `after_skat` | Schneider, Schwarz angesagt oder Ouvert ohne Hand: `Declaration::with_hand(false)`, `hand()` ist dann `false`. Stufe prüfen mit `Declaration::risk_at_least` (`Hand` nur aus der Hand), `RiskLevel::is_announcement` |
| Null-Risikostufe | `NullRisk` | `Normal` (23), `Hand` (35), `Ouvert` (46), `OuvertHand` (59); Stufe 1 bis 4 (`step`) |
| Ramsch | `Declaration::Ramsch` | Boss „Der Ramscher“: Buben Trumpf, jeder für sich, die meisten Augen verlieren |
| Jungfrau (Ramsch: Spieler ohne Stich) | `virgin` | Feld in `LevelParts`; nicht verwechseln mit der Hausregel `virgin` |
| Augen und Stiche pro Platz | `SeatTotals` | `card_points`, `tricks` (Index = Platz), `most` (die meisten Augen, im Ramsch die Verlierer); in `ScoreBreakdown::seats`, `GameFinished::seats`; Sicht `seat_tricks`, Zähler `count.seats` |
| Spielart | `GameType` | `Suit(Suit)`, `Grand`, `Null`. In Relikt-Texten (Allrounder, Wechselbalg) wie in der Skatordnung nur Farbspiel, Grand oder Null: Bedingungen `SuitGame`, `GameIs(Grand)`, `NullGame`; ein Wechsel der Trumpffarbe ist keine andere Spielart |
| Farbspiel | `GameType::Suit` | |
| Hand (Skat nicht aufnehmen) | `hand` (bool) | |
| Skat aufnehmen | `pick_up_skat` | |
| Drücken | `discard` | Die zwei Karten, die in den Skat gelegt werden |
| Drei drücken (dritte Karte vom linken Gegner, drei drücken, einmal pro Serie) | `discard_three` | Effekt `DiscardThree`, Regel `GameRules::discard_three`, verbraucht `Series::discard_three_used`, frei `Run::discard_three_ready` (Sicht `discard_three_ready`); `Game::pick_up_skat_drawing`, `Game::discard_three`, `Game::drawn`, `Game::returned`, `Game::discard_count`; Ereignisse `CardDrawn`, `CardReturned`; Aktion `RunAction::DiscardThree`; Sicht `GameView::discard_count`, `CardView::drawn_from`, `RevealSource::Returned`. Welche Karte zurückgeht: `rules::returned_discard`; Ramponiert bei drei: `valid_discard_of`, `discardable_of` |
| Ausspiel vorgeben (Lockvogel) | `lead_demand` | `rules::LeadSuit` (`Trump` oder `Suit(Suit)`, `options`, `matches`), `rules::legal_leads`; Effekt `DemandLead(n)`, Regel `GameRules::lead_demands`, gezählt `ActiveGame::lead_demands_used`; `Game::demand_lead`, `Game::lead_demand`; Ereignis `LeadDemanded`; Aktion `RunAction::DemandLead`; Sicht `can_demand_lead`, `lead_demand_options`, `lead_demand`; Fehler `MustLeadDemanded`, `InvalidLeadDemand`, `NoLeadDemandLeft` |
| Skattausch (Oma, Opa) | `skat_swap` | `GameRules::skat_swaps` (Liste von Filtern, je einer pro Figur und Serie), `Series::skat_swaps_taken`, Sicht `skat_swap_cards`, Aktion `TakeFromSkat { give, take }` |
| Ausspielen | `lead` | Erste Karte eines Stichs |
| Bedienen | `follow_suit` | |
| Stechen | `ruff` / `trump` (verb) | Mit Trumpf übernehmen |
| Abwerfen | `slough` | Nicht bedienen können und Nicht-Trumpf legen |
| Schmieren | `smear` | Volle in den Stich des Partners geben |
| Stich | `Trick` | |
| Stichgewinner | `trick_winner` | |
| Passen / Einpassen | entfällt | Bis 01.11.2026: ein Spiel auslassen, kostete ein Spiel der Serie. Ersetzt durch Einmischen. Alte Namen werden noch gelesen (`Pass`, `passes`, `games_passed`, `passes_used`, `PassesAdded`, `ExtraPasses`, `without_pass`); Fehlercode `NoPassesLeft` bleibt stehen, kommt aber nicht mehr vor |
| Einmischen (Blatt zurückgeben, neu geben) | `redeal` | `RunAction::Redeal`. Bis zu `RunConfig::redeals` (2) mal pro Spiel vor der Ansage, kostet kein Spiel; danach muss eines der Blätter gespielt werden. Pro Spiel: `Content::redeals` (mit `CharacterRule::redeals`) plus `GameRules::extra_redeals` (Effekt `ExtraRedeal`, Schieben); für die Serie: `Series::extra_redeals` (`ConsumableAction::ExtraRedeals`, Ein Bier, Kräuter). Zähler `Series::redeals_in_game`, `Series::redeals_used`, `RunStats::redeals`; übrig `Run::redeals_left`, Sicht `SeriesView::redeals_left`, `DeclareOptions::can_redeal`; Ereignis `Redealt`, `RedealsAdded`; Fehler `NoRedealLeft` |
| Reue (Einmischen zurücknehmen) | `undo_redeal` | Schnaps `regret`, `ConsumableAction::UndoRedeal`: das Blatt vor dem letzten Einmischen kommt zurück (`ActiveGame::previous`), Ereignis `RedealUndone`, Fehler `NoRedealToUndo` |
| Kontra | `kontra` | Gegenspieler verdoppeln (Hausregel, Boss) |
| Re | `re` | Antwort des Alleinspielers auf Kontra |
| Schenken (Gegner geben auf) | `concede` | Rest der Stiche und Augen geht an den Alleinspieler |
| Rest abkürzen (entschiedenes Spiel beenden) | `settle` | `RunAction::SettleGame`, Sicht `can_settle`, `Run::can_settle`; Schranken der Reststiche in `endgame` (`Reach`, `Span`). Nicht verwechseln mit `Run::settle` (ein fertiges Spiel werten) |
| Schieben (Hausregel: einmal mehr einmischen) | `skat_redraw` | Effekt `ExtraRedeal` (liest auch `SeriesRedeal`): ein Einmischen mehr in jedem Spiel (DESIGN 8.1) |
| Neu geben (Einmischen, „Neues Blatt“) | `redeal` | Ereignis `Redealt` |
| Skat neu ziehen (nur Spielebene) | `redraw_skat` | `Game::redraw_skat`: Gegner und Skat neu, eigene Hand bleibt. Von keinem Inhalt mehr genutzt |

## Wertung

| Deutsch | Bezeichner | Anmerkung |
|---|---|---|
| Spielwert | `game_value` | Grundwert × Gewinnstufen |
| Grundwert | `base_value` | Karo 9, Herz 10, Pik 11, Kreuz 12, Grand 24 |
| Gewinnstufen | `levels` | Spitzen + 1 + Hand + Schneider + … |
| Spielwert-Faktor (auch gebrochen, ×1,5) | `multiplier` | `Factor` in `ScoreBreakdown`, `ScoreStep`, `ValuePreview` (JSON Dezimalzahl); ganze Faktoren `Change::Multiply`, gebrochene `Change::Scale` (Effekt `ScaleGameValue`). Der Spielwert wird am Ende halb aufgerundet |
| Spitzen / Matadore | `matadors` | `with: bool` für „mit“ oder „ohne“ |
| mit / ohne | `with` / `without` | |
| Schneider | `schneider` | Eine Seite mit höchstens 30 Augen (`SCHNEIDER_LIMIT`). Gemessen an der verlierenden Seite, damit es auch bei veränderten Blättern gilt. Schwarz schließt Schneider immer ein. |
| Schwarz | `schwarz` | Alle Stiche |
| angesagt | `_declared` | `schneider_declared`, `schwarz_declared` |
| Ouvert | `ouvert` | Offen spielen |
| Gewonnen | `won` | Ab 61 Augen |
| Gewinnschwelle, Schneidergrenzen | `Thresholds` | `win`, `schneider_limit`, `declarer_schneider`, `stake` (Einsatz, `Thresholds::with_stake`, Relikte senken über `lower_win`), `schneider_bonus` |
| Schneider-Zuschlag (Stichfest: erster Stich zählt für die Schneider-Grenze doppelt) | `schneider_bonus` | `Thresholds::schneider_bonus`: Augen, die nur für Schneider ein zweites Mal zählen; gewonnen und Gegner mit höchstens `schneider_limit + schneider_bonus` Augen ist Schneider. Effekt `SchneiderBonusFromTrick`, Quelle `GameRules::schneider_bonus_source`, Zeile `Lever::Schneider` |
| Gedrückte Karten zählen mehrfach (Skatpfleger) | `MultiplySkatPoints` | Effekt bei der Ansage, nur nach dem Aufnehmen; `Game::map_skat` ersetzt die zwei Skatkarten, Ereignis `SkatPointsRaised`, Zeile `Lever::CardPoints` |
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
| Run / Kneipentour (bis ADR 0016 Tournee) | `Run` | Ein Abend: drei Stadtteile |
| Stadtteil (ADR 0016; bis dahin Abschnitt / Skatabend) | `section` | Vier Kneipen und die Boss-Kneipe; im Code weiter `section` (`Series::section`, `RunStart::section`, `Run::sections`, `RunConfig::sections`); Name aus dem Namensgenerator (`DistrictView::name`). Ein Ante |
| Kneipe (ADR 0016; bis dahin Serie) | `Series` | Ein Blind: Zielwert, 3 Spiele, zwei Stammgäste. Der Typ behält den Namen `Series` (Index `Series::index` 0 bis 4, `PUBS_PER_DISTRICT` = 5, die letzte die Boss-Kneipe, `Series::is_boss_pub`); Run-Ereignisse `SeriesStarted`, `SeriesWon`, neu `PubLost`. Verlauf `Run::pub_results` (`PubResult`). Knoten-Id `d<Stadtteil>-p<Kneipe>` (`tour::pub_id`) |
| Boss-Kneipe (bis ADR 0016 Boss-Serie) | `is_boss_pub` | Letzte Kneipe eines Stadtteils; verloren wird sie wiederholt |
| Leben | `lives` | `Run::lives`, `RunConfig::lives` (`LivesConfig`: `start`, `max`, `boss_bonus`), Ereignis `LivesChanged { change, lives }`, Sicht `TourView::lives`, `max_lives`. In der Anzeige Herzen |
| Route, Routenwahl | `route` | `RunPhase::Route`, `tour::RouteChoice` (nächste Kneipe und drei `RouteOption`), Aktion `RunAction::ChooseRoute(i)`, Ereignisse `RouteOpened`, `RouteChosen`; unterwegs `tour::Trip`. Sicht `TourView::route` (`RouteView`, `RouteOptionView`, `minutes` nur Dekoration) |
| Verkehrsmittel (Zu Fuß, Bahn, Fahrrad) | `Transport` (`Walk`, `Train`, `Bike`) | Seit 02.11.2026 alle gleich: je eine Station, nichts kostet extra; nur Darstellung |
| Station (unterwegs) | `RouteStop` | `category` und, außer beim Tresen, das gezogene Ereignis (`encounter`, verdeckt) |
| Stationsart: Tresen, Begegnung, Upgrade (Werkstatt), Rast | `StopCategory` (`Tresen`, `Event`, `Upgrade`, `Rest`) | Je Weg gewürfelt nach `RouteConfig::weights`; Ereignisse tragen ihre Kategorie (`EncounterDef::category`) |
| Spelunke (Elite-Kneipe) | `Spelunke` | Eigener Ort mit eigenem Namen (`city.spelunken`), steht für eine normale Kneipe; Zielwert × `SpelunkeConfig::target_factor`, ein Malus, Beute ein Relikt (Ereignis `spelunke_loot`, `EncounterKind::RelicReward`, Wahl `TakeRelic`, `TakeCoins`). In `Series::spelunke`, `RouteOption::spelunke`; Knoten-Id `d<S>-p<K>-s`, Sicht `SpelunkeNode` |
| Malus (Spelunke) | `MalusDef`, `MalusId` | Daten in `tour.ron`: `passive` (Tischregeln, Quelle `Source::Malus`), `no_redeal`, `lock_schnapps` (Fehler `ConsumablesLocked`), `curse` (`MalusCurse`, Karten nur für diese Kneipe, `Series::temp_curses`) |
| Stadt, Namensgenerator | `CityNames` | Wortlisten in `tour.ron`, `CityNames::generate`, `Run::city`; Karten-Seeds `Run::map_seed`, Sicht `TourView::map_seed`, `DistrictView::map_seed` (keine Geometrie im Kern) |
| Ansicht des Stadtteils (Altstadt, Hafen, Bahnhof) | `DistrictLook` (`OldTown`, `Harbour`, `Station`) | Daten je Stadtteilname in `tour.ron` (`DistrictName::look`); jeder Abend hat einen Stadtteil jeder Ansicht, Reihenfolge per Seed; Sicht `DistrictView::look`. Nur Anzeige |
| Rast (Leben zurück) | `EncounterKind::Rest` | Wahl `EncounterChoice::Heal(n)` |
| Werkstatt (Upgrade) | `EncounterKind::Workshop` | Modifikation auf eine gewählte Karte oder `EncounterChoice::CleanseChosenCard` |
| Run-Länge: ganzer Abend, kurzer Abend | `RunLength::Full`, `RunLength::Short` | Seit ADR 0016 nur noch der ganze Abend; `Short` bleibt für alte Meta-Stände lesbar (`MetaStats::lengths`, `MetaState::stakes`), ein Run kann so nicht mehr starten (`InvalidStart`, in der Web-Schicht `InvalidLength`). Alte Links mit `-kurz` spielen den ganzen Abend |
| Einsatz, Einsatz-Leiter | `stake` | Gewinngrenze `61 + stake` für Farbspiel und Grand (`RunStart::stake`, `Run::stake`). Höchster freigeschalteter Einsatz je Charakter und Länge: `MetaState::stakes`, `MetaState::max_stake`, in der Sicht `CharacterEntry::max_stakes`. Im Link `-einsatz<n>` |
| Zielwert | `target` | |
| Zielwert-Faktor (Boss) | `target_factor` | Typ `Factor`, in RON und JSON als Dezimalzahl, intern Tausendstel |
| Zielwert-Faktor eines Bosses für einen Charakter | `character_factors` | `BTreeMap<Character, Factor>` in `BossDef`, ersetzt `target_factor` für diesen Charakter (`target_factor_for`) |
| Boss angekündigt (Start eines Stadtteils) | `BossAnnounced` | Run-Ereignis; Boss des Stadtteils `section_boss` |
| Bevorstehender Boss | `upcoming_boss` | Feld der Sicht |
| Nächste Kneipe (Vorschau) | `next_series` | Feld der Sicht, `NextSeriesView`; auf der Routenwahl `RouteView::next` (`PubPreview`) |
| Spiellimit | `game_limit` | |
| Hausregel | `HouseRule` | Passives Relikt, Joker-Äquivalent |
| Stammtisch-Figur | `Regular` | Passives Relikt mit Persönlichkeit |
| Stammgast (normaler Gegner am Tisch) | `Guest` | Name, Gesicht (`look`) und Sprüche (`quips`: `greet`, `win`, `lose`), keine Regeln. Daten `GuestDef` in `guests.ron`, Id `GuestId`; zwei pro Serie in `Run::guests`, Sicht `SeriesView::guests` (`GuestInfo`). Nicht `Regular`, das ist die Stammtisch-Figur |
| Verbrauchsgegenstand | `Consumable` | Definition `ConsumableDef` |
| Schnaps (Gruppe von Verbrauchsgegenständen) | `ConsumableCategory::Schnapps` | Feld `category` an `ConsumableDef` und `ConsumableInfo`; sonst `Item` |
| Wirkung aufs nächste angesagte Spiel (Doppelkorn, Magenbitter) | `NextGameEffect` | `Multiply(n)`, `SingleLoss`; Aktion `ConsumableAction::NextGame`, wartend `Run::pending_next_game` |
| Skat ansehen (Kurzer) | `PeekSkat`, `SkatPeeked` | Feld `ActiveGame::skat_peeked`, Sicht `GameView::skat_peek` |
| Skatkarte zeigen (Spickzettel) | `skat_shown` | Effekt `RevealSkatCard(n)` (Information, bei der Ansage eines Handspiels), Feld `ActiveGame::skat_shown`, Run-Ereignis `SkatCardShown`, Sicht `GameView::skat_shown` |
| Umgemalte Karte (Kreide) | `chalk`, `ChalkCard`, `CardChalked` | `Game::chalk`, Feld `ActiveGame::chalked` (Id und gedruckte Farbe), Sicht `CardView::chalked`; Fehler `CannotChalk` |
| Faktor eines verlorenen Spiels | `Lever::LossFactor` | 2 nach den Run-Regeln, Magenbitter setzt 1 |
| Boss (Gegnerpaar mit Sonderregel) | `Boss` | Definition `BossDef`, in `Series::boss` |
| Seltenheit | `Rarity` | `Common`, `Uncommon`, `Rare`, `Legendary` (Legendär, M8) |
| Gewicht der Stufe Legendär | `RarityWeights::legendary` | Dezimalzahl (Standard 0,3), `RarityWeights::percent_weight` |
| Immer Vorhand (Ehrenvorsitz) | `AlwaysForehand` | Effekt, nur passiv |
| Stufen je Zählerpunkt (Skatgott) | `AddLevelsPerCounter` | `{ counter, per }`, wie `AddBaseValuePerCounter` für Gewinnstufen |
| Preis | `price` | Groschen |
| Verkaufen (halber Preis) | `sell` | |
| Neuauslage (Tresen) | `reroll` | |
| Grundbetrag (gewonnene Kneipe) | `series_reward` | |
| Zinsen | `interest` | |
| Auszahlung einer Kneipe | `Payout` | Grundbetrag, Bonus für übrige Spiele (`per_game_left`), Zinsen; eine verlorene Kneipe zahlt nichts |
| Run-Aktion (Entscheidung des Spielers) | `RunAction` | |
| Run-Ereignis | `RunEvent` | |
| Flagge (Relikt-Zustand, z. B. Bock) | `flag` | Pro Run |
| Zähler (Relikt-Zustand, z. B. Tournee) | `counter` | Pro Run. Zähler einer Serie setzt `ResetCounter` nach der gewonnenen Serie zurück (Ereignis `CounterReset`); `AddBaseValuePerCounter { counter, per }` gibt `per` Grundwert je Punkt |
| Bock | `bock` | Flagge der Bockrunde |
| Relikt (Oberbegriff) | `Relic` | HouseRule und Regular, Definition `RelicDef` |
| Tresen | `Shop` | |
| Groschen | `coins` | Währung |
| Groschen erhalten (Effekt) | `GainCoins` | Ereignis `CoinsGained { source, amount, coins }`; negativ zieht ab, nie unter 0 |
| Beim Erhalt (Kartenmodifikation angebracht) | `OnAttach` | Auslöser nur für Kartenmodifikationen, Stufe `Stage::OnAttach` |
| Wenn gedrückt (Gutschein) | `OnDiscard` | Auslöser nur für Kartenmodifikationen, Stufe `Stage::OnDiscard`; die Karte landet beim Drücken im Skat |
| Fluch (negative Kartenmodifikation) | `curse` | `ModKind::Curse`, Prüfung `CardModDef::is_curse` (in Sicht und Katalog weiter `curse`); Text beginnt mit „Fluch:“. Sperrt die Karte für jede weitere Modifikation (Effekte v2). Nie aus Zufallsziehungen, nur über Ereignisse, deren Text „Fluch“ sagt (`MarkCard { curse: true }` zieht einen Fluch); zahlt beim Erhalt Groschen (M7) |
| Ereignis / Rast | `Encounter` | Station unterwegs (ADR 0016). Nicht `Event` (das sind die Zustandsereignisse). Definition `EncounterDef`, offen `ActiveEncounter`, Option `EncounterOption`, Wahl `EncounterChoice`, Aktion `RunAction::ChooseEncounter` |
| Hausregel des Stadtteils (Stammtisch-Streit, bis ADR 0016 des Abends) | `evening_relics` | Gilt bis zum Ende des Stadtteils, ohne Platz |
| Charakter | `Character` | `Declarer` (Alleinspieler), `NullPlayer` (Nullspieler), `GrandPlayer` (Grandspieler), `Onlooker` (Zaungast, nicht die Figur „Der Kiebitz“ `kibitzer`), `Landlady` (Wirtin, nicht die Figur „Der Wirt“ `landlord`); Daten `CharacterDef` |
| Eigene Regel eines Charakters | `CharacterRule` | Feld `rule` in `CharacterDef`: `passive`, `hooks` (Hook-API wie ein Relikt ohne Platz, Quelle `Source::Character`), `redeals` (Einmischen pro Spiel) und `relic_slots` (Zuschlag auf `run.ron`, nie unter 0), `payout_factor`. Werte eines Runs: `Content::redeals`, `Content::relic_slots`, `Content::payout_factor` |
| Zuschlag der Auszahlung (Wirtin) | `Payout::bonus` | Was `payout_factor` über die einfache Auszahlung hinaus zahlt |
| Blatt eines Gegners sehen (Zaungast) | `RevealDefenderHand(seat)` | Informations-Wirkung, Platz 1 (links) oder 2; Feld `ActiveGame::shown_hand`, Sicht `RevealSource::Onlooker` |
| Start-Verbrauchsgegenstände | `start_consumables` | Feld in `CharacterDef` |
| Nach einer Kneipe mindestens n Groschen | `SeriesWonWithCoins(n)` | Freischaltungs-Bedingung, gemessen nach der Auszahlung |
| n-te Kneipe des Abends erreichen | `ReachPub(n)` | Freischaltungs-Bedingung (ADR 0016), gezählt über die Stadtteile: `Stadtteil × 5 + Kneipe + 1` |
| Startblatt / Grunddeck (Abweichung vom 32er-Blatt) | `DeckSpec` | seit ADR 0015 nur `mods` (Kartenmodifikationen der Ausgabe des Charakters); `remove` und `add` sind ein Ladefehler |
| Spielart, für die ein Relikt gemacht ist | `GameFocus` | `All`, `Trump` (Farbspiel und Grand), `Null`; Feld `focus` an Relikten und Modifikationen (ADR 0012) |
| Tresen-Gewicht pro Spielart | `FocusWeights` | `trump`, `null` in Prozent der Seltenheit; Feld `focus_weights` in `CharacterDef` |
| Freischaltung | `Unlock` | Definition `UnlockDef`, Id `UnlockId`, Bedingung `UnlockCondition`, Belohnung `Reward` |
| Meta-Zustand (über Runs) | `MetaState` | Freischaltungen und Statistik, getrennt vom Run gespeichert |
| Gesperrt | `locked` | Relikte, die noch Belohnung einer Freischaltung sind |
| Pool eines Charakters | `characters` | Feld an Relikten, Modifikationen, Verbrauchsgegenständen, Bossen und Ereignissen; leer = alle |
| Gesperrte Modifikation | `locked_card_mods` | Belohnung `Reward::CardMod` einer Freischaltung, wird bis dahin nicht zufällig gezogen |
| Art und Seltenheit einer Belohnung | `RewardInfo`, `RewardKind` | `HouseRule`, `Regular`, `CardMod`, `Character`; Feld `reward_info` im Meta-View |
| Endboss | `final_boss` | „Der Alte“, Regeln nacheinander: `rotation` |
| Regel des Bosses in diesem Spiel | `boss_rule` | Beim Alten wechselt sie pro Spiel |
| Schummeln (Boss-Regel) | `is_cheat` | Zwillinge, Revolutionär; abgeschaltet durch `NoCheating` |
| Informations-Effekt | `is_information` | Durchsichtig, Kiebitz, Zähler, Spickzettel; abgeschaltet durch `DisableInformation` |
| Rabatt am Tresen (Wirt) | `shop_price`, `ShopDiscount` | |
| Kiebitz-Blick | `peek`, `CardPeeked` | |
| Herkunft einer aufgedeckten Karte | `RevealSource` | `CardMod(id)` (Durchsichtig) oder `Kibitz`; Feld `RevealedCard::source` |
| Risikostufe | `RiskLevel` | Normal, Hand, SchneiderDeclared, SchwarzDeclared, Ouvert; die drei Ansagen auch nach dem Skat (ADR 0013) |
| Statistik eines Runs | `RunStats` | Gewonnene und verlorene Spiele, Einmischen (`redeals`), Serien, bestes Spiel |
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
| Stich gewonnen mit (Luschenkönig) | `TrickWinnerMatches` | `filter`, `by_declarer`: der eben beendete Stich, nur bei Stichende |
| Erster Stich des Alleinspielers (Stichfest) | `FirstTrickOfDeclarer` | Der eben beendete Stich ist der erste, den der Alleinspieler macht |
| Kartenfilter | `CardFilter` | Farbe und/oder Wert |
| Hebel der Formel | `Lever` | `BaseValue`, `Levels`, `GameValue`, `CardPoints`, `WinThreshold`, `Schneider`, `LossFactor` |
| Herkunft eines Zuschlags | `Source` | Relikt, Kartenmodifikation, Boss, Kontra, Re, Ereignis, Verbrauchsgegenstand (`Consumable`) |
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
| Augen (Modifikation) | `points` (nicht verwechseln mit Augen = `card_points`) |
| Rang (Modifikation) | `rank` |
| Trumpf (Modifikation) | `trump_bonus` |
| Letzter Stich (Modifikation, bis Effekte v2 „Spitze“) | `last_trick` |
| Erster Stich (Modifikation) | `first_trick` |
| Gutschein | `voucher_coins` |
| Gutschein Untot | `voucher_undead` |
| Gutschein Wandelkarte | `voucher_wild` |
| Durchsichtig (Fluch seit Effekte v2) | `transparent` |
| Fleckig (Fluch, bis Effekte v2 „Schwarze Katze“) | `stained` |
| Augen +3, Augen +5 (Verbrauchsgegenstand) | `points_3`, `points_5` |
| Trumpf +2, Trumpf +4 (Verbrauchsgegenstand) | `trump_2`, `trump_4` |
| Rang +1 (Verbrauchsgegenstand) | `rank_1` |
| Gestrichen mit Effekte v2 (alte Ids, nur noch beim Umrechnen) | `fat`, `marked`, `stamp`, `invisible`, `gold_rim`, `double_marked`, `crown`, `seal`, `horseshoe`, `tape_measure`, `lucky_penny`, `beer_stain`, `cracked`, `black_cat`, `lead_weight`, `tail_light`, `dressed_up`, `dummy`, `honorary_jack`, `glass_ace`, `homesick` |
| Der Schieber | `the_pusher` |
| Der Kontra-Hans | `kontra_hans` |
| Die Oma | `grandma` |
| Frisches Deck | `fresh_deck` |
| Stempelkissen | `ink_pad` |
| Ein Bier | `beer` |
| Neues Blatt | `new_deal` |
| Kurzer | `shot` |
| Doppelkorn | `double_grain` |
| Kräuter | `herbal` |
| Magenbitter | `bitters` |
| Kreide | `chalk` |
| Reue | `regret` |
| Blindflug | `blind_flight` |
| Der Kiebitz | `kibitzer` |
| Der Wirt | `landlord` |
| Die Oma (Figur) | `granny` (nicht verwechseln mit dem Boss `grandma`) |
| Der Schiedsrichter | `referee` |
| Der Angeber | `show_off` |
| Der Zähler (Figur) | `card_counter` (nicht `counter`, das ist der Relikt-Zustand) |
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
| Umgekipptes Bier | `spilled_beer` |
| Gasthof zur Krone | `crown_inn` |
| Der Taschenspieler | `conjurer` |
| Preisskat | `prize_skat` |
| Gast aus Altenburg | `guest_from_altenburg` |
| Ramponiert (Fluch, seit Effekte v2 −X Augen: Augen mit negativem Wert, Konstante `deck::RAMPONIERT` neben `deck::AUGEN`) | `battered` |
| Der Opa (Figur) | `grandpa` (nicht verwechseln mit `granny`) |
| Mit Vieren | `with_four` |
| Siebenschläfer | `seven_sleeper` |
| Saubere Weste | `clean_slate` |
| Skatpfleger | `skat_keeper` |
| Altes Blatt | `old_deck` (Zähler `old_deck`) |
| Allrounder | `all_rounder` (Zähler `all_rounder`, Flaggen `all_rounder_suit`, `_grand`, `_null`) |
| Stammfarbe | `home_suit` (Zähler `home_suit`, Flaggen `home_suit_clubs`, `_spades`, `_hearts`, `_diamonds`) |
| Wechselbalg | `changeling` (Flaggen `changeling_suit`, `_grand`, `_null`) |
| Bubenparade | `jack_parade` |
| Luschenkönig | `blank_king` (Lusche = `blank`, nicht Blanko) |
| Stichfest | `watertight` |
| Drei drücken | `three_discard` (Freischaltung `ten_runs`, „Stammkunde“) |
| Spickzettel | `crib_sheet` |
| Lockvogel | `decoy` |
| Wandelkarte | `wild_card` |
| Untot | `undead` |
| Skatgott | `skat_god` (Zähler `skat_god`) |
| Ehrenvorsitz | `honorary_chair` |

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
| `Queen` | Oberin |
| `Jack` | Unter |
| `Nine`, `Eight`, `Seven` | Neun, Acht, Sieben |

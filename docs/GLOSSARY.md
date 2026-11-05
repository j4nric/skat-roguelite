# Glossar: Skat-Begriffe und ihre Bezeichner im Code

Die Domäne ist deutsch, der Code ist englisch. Diese Tabelle ist verbindlich. Wer einen neuen Begriff braucht, trägt ihn hier ein, bevor er ihn im Code verwendet. Keine Synonyme (kein `unter` neben `jack`, kein `points` neben `card_points`).

## Karten

| Deutsch | Bezeichner | Anmerkung |
|---|---|---|
| Blatt (das ganze Spiel, 32 Karten) | `deck` | Im Roguelike bis ADR 0015 das geteilte, modifizierbare Blatt; seit ADR 0015 immer das normale Skatblatt aus 32 verschiedenen Karten (`Game::deck_cards`: jede Karte in der Ausgabe ihres Halters) |
| Deck (in Spieltexten seit 04.11.2026; ADR 0015 und Doku: Ausgabe) | `Edition` | In der Anzeige heißt die Ausgabe des Spielers „Deck“ ("in deinem Deck"), die der Gegner „Deck des Gegners“ und „Boss-Deck“; „Ausgabe“ kommt in Spieltexten nicht mehr vor (Wunsch des Nutzers, 04.11.2026). Die Version jeder Karte, die ein Spieler besitzt: je Karte die Modifikationen (`core/src/deck.rs`). Ausgabe des Spielers `Run::edition` (ersetzt `Run::deck`, Sicht `RunView::edition`); Gegner: normal (Stammgast) oder Boss-Ausgabe je Figur (`figures[].edition` in `bosses.ron`, `Edition::with_mods`, `ActiveGame::boss_edition` links, `ActiveGame::partner_edition` rechts, ADR 0017); `Editions` bündelt sie für die Hooks (`player`, `boss`, `partner`) |
| Halter einer Karte | `holder` | Der Platz, der die Karte gerade hält; die Karte zeigt seine Ausgabe. Der Skat gehört dem Alleinspieler. Wechsel nur über `Game::change_holder`, Ereignis `CardChangedHands`; `Game::in_edition_of`, `Game::current` |
| Besitzer einer Ausgabe | `owner` | `DealtCard::owner` (`Owner`: `Player`, `Guest`, `Boss`, seit ADR 0017 `BossPartner`): wessen Ausgabe die aufgelöste Karte ist, für Sicht, Rückseite und Hooks; `Run::owner_of_seat`, Plätze der Boss-Spieler `BOSS_SEAT` (links) und `PARTNER_SEAT` (rechts) |
| Rückseite | `back` | Kosmetik; Spieler: die Rückseite seines Charakters (`CharacterDef::back`, Standard Distel, `web/src/ui/backs.ts`, `setPlayerBack`), Gegner: `back` in `guests.ron` (Standard `opponent`, `OPPONENT_BACK`) und `bosses.ron` (je Boss eine, eine Figur darf eine eigene haben: `BossFigure::back`); Sicht `CardView::back`, `GameView::backs`, `BossInfo::back` |
| Boss-Ausgabe (Zeile) | `EditionEntry` | Kartenfilter und Modifikation, Sicht `BossFigureInfo::edition` |
| Boss-Spieler, Figur eines Bosses (ADR 0017; bis dahin offen als „Sekundant“) | `BossFigure` | Zwei je Boss (`BOSS_FIGURES`), `figures` in `bosses.ron`: Name, Ausgabe, optional Rückseite; die erste links (`Owner::Boss`), die zweite rechts (`Owner::BossPartner`). Sicht `BossInfo::figures` (`BossFigureInfo`) |
| Boss-Belohnung, Relikt-Wahl (ADR 0017) | `BossReward` | Nach einer gewonnenen Boss-Kneipe (nicht der letzten): `RunPhase::BossReward`, `RunAction::ChooseBossRelic { offer, replace }`, Ereignisse `BossRewardOffered`, `BossRewardDeclined`, Sicht `RunView::boss_reward` (`BossRewardView`), Daten `boss_reward` in `run.ron` (`BossRewardConfig`); dazu doppelte Auszahlung `Payout::boss` |
| Kartenbelohnung (Zinke aufs Haus, 04.11.2026; feste Paare seit 05.11.2026) | `CardReward` | Nach einer gewonnenen normalen Kneipe oder Spelunke: drei Angebote aus je einer Karte des Decks mit einer Zinke (`RewardOffer { card, card_mod, value }`), eines gratis oder Überspringen für Groschen. `RunPhase::CardReward`, `Run::card_reward` (`CardReward`), `Run::card_reward_open`; Aktionen `RunAction::TakeCardReward { offer }` (bis 05.11.2026 mit `card`), `RunAction::SkipCardReward`; Ereignisse `CardRewardOffered`, `CardRewardTaken`, `CardRewardSkipped`; Fehler `CardRewardPending`; Sicht `RunView::card_reward` (`CardRewardView`, `CardRewardOfferView` mit `card`, `result`, `can_take`, `replaces`, `raises`); Daten `card_reward` in `run.ron` (`CardRewardConfig`); Modul `core/src/run/card_reward.rs`. Gewicht einer Bau-Richtung: `Run::card_reward_bias` |
| Fluch für die Kneipe (Die Oma, ADR 0017) | `CurseForPub` | Wirkung `CurseForPub { cards, curses }`; gemerkte Karten `Series::boss_curses`, Ereignisse `BossCursesLaid`, `BossCursesLifted`; `Edition::lay_temp_curse`, `Edition::restore` |
| Ersatz für einen entfernten Boss (Spielstände) | `RETIRED_BOSSES` | Kontra-Hans und Revolutionär → Zwillinge, Kiebitz-Mörder → Schwarzer Peter (`Run::drop_unknown_content`) |
| Scherben (zerbrochener Glasdaus, ADR 0015) | `shards` | Nicht umgesetzt (der Glasdaus ist mit Effekte v2 gestrichen) |
| Abgestuft (Nullspieler, ADR 0015) | `demoted` | Spezialkarte, die ein Fluch ist (`ModKind::SpecialCurse`), Wirkung `Demote`, `PlayingCard::demoted`, `DealtCard::demoted`: ein Unter ist keine Unter mehr, sondern die niedrigste Karte seiner Farbe ohne Augen (`rules::is_demoted_jack`), jede andere Karte sticht eine Stufe tiefer |
| Reinigen (ADR 0015; bis 02.11.2026 der Gegenstand Frisches Deck) | `cleanse` | Alle Modifikationen einer Karte der Ausgabe fallen ab: `Edition::cleanse`, `DeckChange::Cleansed`, Fehler `DeckError::NothingToCleanse`; am Tresen „Karte reinigen“, in der Werkstatt `CleanseChosenCard`. Die Aktion `ConsumableAction::CleanseCard` ruht (kein Inhalt nutzt sie) |
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
| Rangbonus (bis Effekte v2 Gezinkt, bis 03.11.2026 Winkel) | `rank_bonus` | Stufen höher innerhalb von Trumpf oder Farbe; bei Gleichstand behält die frühere Karte den Stich. Kein Inhalt nutzt ihn mehr (Effekt `RankBonus` ruht) |
| Winkel: steigen (die Karte wird die nächsthöhere ihrer Farbe, seit 03.11.2026) | `promote` | Effekt `Promote`, `Rank::promoted`, `Rank::promotion_room`, `DealtCard::promoted` (Stufen), `CardModDef::max_on`; Umrechnung alter Spielstände `deck::migrate_winkel`, `WINKEL_TO_AUGEN` |
| Blei (im Null die niedrigste Karte ihrer Farbe) | `leaden` | Effekt `NullLowest`, `PlayingCard::leaden`, `DealtCard::leaden` |
| Doppelgänger (Spezialkarte: tauscht beim Spielen mit einer Karte gleichen Rangs eines Gegners) | `doppelganger` | Effekt `Doppelganger`, `Game::play_swapped`, Ereignisse `DoppelgangerSwapped { seat, card, from, stolen, given }`, `RunEvent::DoppelgangerUnmatched`; Zufall `ActiveGame::doppel_rng` |
| Gegenstück (Doppelgänger: die Karte gleichen Rangs, die er nimmt) | `stolen` | Feld von `DoppelgangerSwapped`; der Gegner bekommt `given` |
| Bube ehrenhalber | `honorary_jack` | `PlayingCard::honorary_jack`, Effekt `HonoraryJack`: im Farbspiel Trumpf eine halbe Stufe unter dem Karo-Buben, keine Spitze (DESIGN 4.4). In der Anzeige „Unter ehrenhalber“. Seit Effekte v2 ohne Inhalt, die Wirkung ruht |
| Platz in der Trumpfordnung | `trump_order_key` | Stärke, dann 0 für den Buben ehrenhalber, 1 für gedruckte Trümpfe |
| Darf nicht gedrückt werden (bis Effekte v2 der Fluch Ramponiert, seitdem ruhend) | `no_discard` | `PlayingCard::no_discard`, Effekt `NoDiscard`, Prüfung `valid_discard`, Fehler `DiscardForbidden`; je Handkarte `discardable` (`rules::discardable`, `CardView::discardable`) |
| Volle (Ass, Zehn) | `high_card` | |
| Trumpf | `trump` | |
| Skat (die zwei Karten) | `skat` | nie `talon`, siehe unten |
| Talon (Roguelike: Karten über 32, nicht im Spiel) | `talon` | entfallen mit ADR 0015 (M9), im Code gestrichen |
| Gezinkte Karte / Kartenmodifikation | `CardMod` | Definition `CardModDef`, Id `CardModId` |
| Zinken (Kategorie am Tresen, „Karte zinken“, seit 02.11.2026) | `zinken` | Die Kartenmodifikationen, wie der Tresen sie verkauft (Auslage, Kartenmaler); Modul `core/src/run/zinken.rs`, Einstellungen `ZinkenConfig` (`zinken` in `run.ron`); eine einzelne heißt Zinke |
| Auslage (Zinken am Tresen) | `CardOffer`, `Shop::card_offers` | Drei Angebote, je Karte der Ausgabe und Zinke mit Wert; Aktion `RunAction::BuyCardOffer(slot)`, Ereignisse `CardOffersDrawn`, `CardOfferBought`; Sicht `ShopView::card_offers` (`CardOfferView` mit `card`, `result`, `replaces`, `raises`) |
| Wert in der Auslage | `CardModDef::shop_value`, `offer_value` | Ohne Angabe der erste aus `values` |
| Neu auslosen (Tresen, seit 05.11.2026: Relikte, Schnäpse und Auslage, einmal je Besuch) | `reroll` | `RunAction::Reroll`, Preis `Economy::reroll_cost` über `Run::reroll_cost` (0 mit `FreeReroll`), Ereignis `ShopRerolled { cost }`, Zähler `Shop::rerolls`, Sicht `ShopView::reroll_used`; ein zweites Mal `AlreadyUsed`. Bis 05.11.2026 getrennt: Neuauslage (`Reroll`, beliebig oft) und Neu auslosen der Auslage (`RerollCardOffers`, `CardOffersRerolled`, `reroll_step`, `Shop::card_rerolls`, entfallen) |
| Kartenmaler | `painter` | Einmal je Tresen: `RunAction::HirePainter(card)`, dann eine von zwei Zinken `RunAction::ChoosePaint(i)`; wartend `Run::painter` (`PainterChoice`), Sicht `RunView::painter` (`PainterView`), `ShopView::painter` (`ServiceView`); Ereignisse `PainterHired`, `PainterChosen`; Fehler `PainterPending`, `AlreadyUsed`. In der Werkstatt gratis (Wirkung `EncounterEffect::Painter`) |
| Karte reinigen (Tresen-Dienst) | `CleanseCard` | Einmal je Tresen, `RunAction::CleanseCard(card)`, Ereignis `CardCleansed`, Sicht `ShopView::cleanse` |
| Im Spiel (die 32 gegebenen Karten) | `cards_in_play` | immer alle 32 Karten (kein Talon seit ADR 0015) |
| Wandelkarte (nimmt bei Spielbeginn die längste Farbe der Hand an) | `wild` | `PlayingCard::wild`, `DealtCard::wild` (nur die Karten des Spielers, solange sie noch offen ist), Effekt `Wild`. Farbe: `rules::settled_wild_suit` (meiste Karten nach gedruckter Farbe, ohne Unter und Wandelkarten; Gleichstand Eichel, Grün, Rot, Schellen); Sicht `CardView::unsettled` (noch offen), `CardView::named` (gedruckte Farbe), `ActiveGame::named` |
| festlegen (Wandelkarte: ihre Farbe bei Spielbeginn) | `settle` | `Game::declare` legt fest, `PlayingCard::settle_wild`; Ereignis `WildCardSettled { old, new }` (Spielebene, auch `PlayerGameEvent`) |
| Untot (Karte steigt aus dem Stich zurück in die Hand) | `undead` | Effekt `Undead`, Prüfung `Run::can_rise`, verbraucht `ActiveGame::risen` (einmal pro Spiel), Zufall `ActiveGame::undead_rng`; Spielebene `Game::raise`, Ereignis `CardRose` (`card` zurück in der Hand, `substitute` im Stich); Sicht `CardView::can_rise`; Bot `RunBot::undead_card` |
| Ersatzkarte (Untot: die Karte, die an ihrer Stelle im Stich liegt) | `substitute` | Feld von `CardRose` |
| Untot: zurück als die Karte, die den Stich gemacht hat (seit 03.11.2026) | `risen` | `PlayingCard::risen_as`, Feld `CardRose::risen` |
| Untot: Wahl der drei Karten | `rise_choice` | `UNDEAD_PICK` (3), wartend `ActiveGame::pending_rise`, `Run::rise_pending`; Ereignis `RunEvent::RiseChoice { card, becomes, pick }`; Sicht `GameView::rise_choice` (`RiseChoiceView`); Aktion `RunAction::ChooseRiseCards([a, b, c])`; Fehler `RisePending`, `InvalidRiseCards`; Bot `RunBot::rise_cards` |
| Diese Karte sticht (Bedingungen der Zinken v2) | `ThisCardWinsTrick`, `ThisCardBeatsDefenderCard`, `ThisCardPlayed` | Goldkante, Fadenkreuz (Filter: Daus eines Gegners), Schneiderkreide (gespielt) |
| Nur auf (Modifikation passt nur auf bestimmte Karten) | `only_on` | Feld `CardModDef::only_on` (`CardFilter`), Prüfung `CardModDef::fits` |
| Effekte v2 (System der Kartenmodifikationen seit 02.11.2026) | – | DESIGN 7.2 |
| Art einer Modifikation | `ModKind` | `Edge` (Rand: immer aktiv), `Moment` (Markierung oben rechts), `Voucher` (Gutschein), `Curse` (Fluch), `Special` (Spezialkarte); Feld `CardModDef::kind` |
| Rand-Modifikation | `ModKind::Edge` | Augen, Rang (Winkel), Trumpf, Blei, Goldkante |
| Moment-Modifikation | `ModKind::Moment` | Letzter Stich, Erster Stich, Fadenkreuz, Schneiderkreide |
| Gutschein (wenn gedrückt) | `ModKind::Voucher` | Auslöser `OnDiscard`, Ereignis `VoucherRedeemed { card, card_mod, value, via_hand }`, Wirkung `TurnInto` (wird Spezialkarte) |
| Marke im Handspiel (eine zufällige Marke der Hand wird bei der Ansage eingelöst, 04.11.2026) | `redeem_hand_voucher` | `Run::redeem_hand_voucher`, eigener Zufall `HAND_VOUCHER_SEED`, Ereignis `VoucherRedeemed { via_hand: true }`; Spielebene `Game::replace_declarer_card` (die Karte wird vor der Ansage Wandelkarte) |
| Spezialkarte | `ModKind::Special` | Untot, Wandelkarte, Doppelgänger; darf zusätzlich eine normale Modifikation tragen. Seit 05.11.2026 beliebig viele je Ausgabe (Entscheidung des Nutzers; vom 03.11. bis 05.11.2026 höchstens eine Karte je Spezialkarte, Fehler `SpecialTaken` und Rückfallzahlung `VoucherCapped`, beide entfallen). Erste Karte mit ihr: `Edition::holder_of`. Ziel einer Marke oder Spezialkarte: `zinken::special_target` |
| Spezialkarte in der Auslage („wird untot“, „wird Wandelkarte“) | `Run::special_offers` | Ein `CardOffer` mit `undead` oder `wild_card`; Anteil `ZinkenConfig::special_percent`, Preis `special_price` |
| Flüche als Handel (03.11.2026) | `trade` | Ein Fluch als Preis eines günstigen Angebots: Modul `core/src/run/trade.rs`, Einstellungen `TradeConfig` (`trade` in `run.ron`), eigener Zufallsstrom `Run::trade_rng`; Ereignis `RunEvent::TradeCurse { card, curse, cause }` mit `CurseCause` (`Bargain`, `Fence`, `Schnapps`); Ziele `Run::trade_curse_targets` |
| Schnäppchen (Auslage) | `bargain` | `CardOffer::bargain` (der Preis ohne Handel), halber Preis `TradeConfig::half`, Anteil `TradeConfig::bargain_percent`; Sicht `CardOfferView::bargain`, `was` |
| Hehlerware (Tresen) | `fence` | Ein Relikt des Tresens zum halben Preis: `Shop::fence` (Platz), Anteil `TradeConfig::fence_percent`, Preis `Run::relic_offer_cost`, `Run::is_fence`; Sicht `RelicOffer::fence`, `was`; Ereignis `ShopOpened { fence }` |
| Pechvogel (Relikt) | `jinx` | +1 Grundwert je verfluchter Karte der Ausgabe: Effekt `AddBaseValuePerCursedCard`, Zählung `Run::cursed_cards` |
| Bau-Richtung (Tag eines Inhalts, 04.11.2026) | `BuildTag`, Feld `tags` | An Relikten, Kartenmodifikationen und Schnäpsen, im Katalog der Web-App: `OneSuit` (Farbtreue), `Vouchers` (Markenhändler), `Tricks` (Stichjäger), `Schneider`, `Null` (Nullspieler) |
| Verbinder-Relikt (zählt Teile einer Bau-Richtung) | – | Zählt Karten der Ausgabe mit `CardCount` (`mods`, `kind`, `trump_suit`, `in_hand`, seit ADR 0020 `direction`): Effekte `AddBaseValuePerCard`, `AddLevelsPerCard`, `ScaleGameValuePerCard`, Zählung `Run::count_cards` |
| Karte mit Zinke (jede Modifikation, die kein Fluch ist) | `marked` | `Run::marked_cards`, `HookContext::marked_cards`, `content::is_marked`; Bedingung `TrickWonWithMarkedCard` (der Alleinspieler macht den Stich mit einer Karte seiner Ausgabe mit Zinke) |
| Marke eingelöst (Auslöser für Relikte) | `OnVoucherRedeemed` | `Stage::OnVoucherRedeemed`, `Hook::OnVoucherRedeemed`; feuert nach jedem `VoucherRedeemed` (gedrückt oder im Handspiel), nur Run-Zustand (Zähler, Groschen) |
| Augen zum Stich (Zuschlag) | `AddTrickPoints` | Wirkung auf den Stich, der gerade fertig ist, wie `MultiplyTrickPoints` über `Game::add_bonus_points` (Fingerspitzengefühl) |
| Fusel (Schnaps) | `rotgut` | `ConsumableAction::NextGameCursed(Multiply(2))`: nächstes angesagtes Spiel ×2 und ein dauerhafter Fluch auf einer zufälligen Karte ohne Fluch |
| Hinweis zur ersten Spezialkarte | `MetaState::specials_seen` | Im Meta-Spielstand; `MetaState::new_specials`, in der Web-App `MetaStep::first_specials` und das Blatt „Neu in deiner Ausgabe“ |
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
| Drei drücken (dritte Karte vom linken Gegner, drei drücken, im ersten Spiel jeder Kneipe seit 02.11.2026) | `discard_three` | Effekt `DiscardThree`, Regel `GameRules::discard_three`, verbraucht `Series::discard_three_used`, frei `Run::discard_three_ready` (Sicht `discard_three_ready`); `Game::pick_up_skat_drawing`, `Game::discard_three`, `Game::drawn`, `Game::returned`, `Game::discard_count`; Ereignisse `CardDrawn`, `CardReturned`; Aktion `RunAction::DiscardThree`; Sicht `GameView::discard_count`, `CardView::drawn_from`, `RevealSource::Returned`. Welche Karte zurückgeht: `rules::returned_discard`; Ramponiert bei drei: `valid_discard_of`, `discardable_of` |
| Ausspiel vorgeben (Lockvogel, entfernt am 02.11.2026) | `LeadSuit` | Nur noch zum Lesen alter Spielstände: `rules::LeadSuit`, Spielereignis `LeadDemanded`; Fehlercodes `MustLeadDemanded`, `InvalidLeadDemand`, `NoLeadDemandLeft` bleiben stehen, kommen aber nicht mehr vor |
| Skattausch (Oma, Opa; ruhend seit 02.11.2026, kein Inhalt nutzt ihn) | `skat_swap` | `GameRules::skat_swaps` (Liste von Filtern, je einer pro Figur und Serie), `Series::skat_swaps_taken`, Sicht `skat_swap_cards`, Aktion `TakeFromSkat { give, take }` |
| Ausspielen | `lead` | Erste Karte eines Stichs |
| Bedienen | `follow_suit` | |
| Stechen | `ruff` / `trump` (verb) | Mit Trumpf übernehmen |
| Abwerfen | `slough` | Nicht bedienen können und Nicht-Trumpf legen |
| Schmieren | `smear` | Volle in den Stich des Partners geben |
| Stich | `Trick` | |
| Stichgewinner | `trick_winner` | |
| Passen / Einpassen | entfällt | Bis 01.11.2026: ein Spiel auslassen, kostete ein Spiel der Serie. Ersetzt durch Einmischen. Alte Namen werden noch gelesen (`Pass`, `passes`, `games_passed`, `passes_used`, `PassesAdded`, `ExtraPasses`, `without_pass`); Fehlercode `NoPassesLeft` bleibt stehen, kommt aber nicht mehr vor |
| Einmischen (Blatt zurückgeben, neu geben) | `redeal` | `RunAction::Redeal`. Bis zu `RunConfig::redeals` (2) mal pro Spiel vor der Ansage, kostet kein Spiel; danach muss eines der Blätter gespielt werden. Pro Spiel: `Content::redeals` (mit `CharacterRule::redeals`) plus `GameRules::extra_redeals` (Effekt `ExtraRedeal`, Schieben); für die Kneipe: `Series::extra_redeals` (Wegbier, Kiebitz am Fenster, ruhend `ConsumableAction::ExtraRedeals`), weniger im ersten Spiel: `Series::redeals_withheld` (Skatlehrer). Zähler `Series::redeals_in_game`, `Series::redeals_used`, `RunStats::redeals`; übrig `Run::redeals_left`, Sicht `SeriesView::redeals_left`, `DeclareOptions::can_redeal`; Ereignis `Redealt`, `RedealsAdded`; Fehler `NoRedealLeft` |
| Reue (Einmischen zurücknehmen, entfernt am 02.11.2026, Aktion ruhend) | `undo_redeal` | Früher Schnaps `regret`, `ConsumableAction::UndoRedeal`: das Blatt vor dem letzten Einmischen kommt zurück (`ActiveGame::previous`), Ereignis `RedealUndone`, Fehler `NoRedealToUndo` |
| Re | `re` | Antwort des Alleinspielers auf Kontra |
| Schenken (Gegner geben auf) | `concede` | Rest der Stiche und Augen geht an den Alleinspieler |
| Rest abkürzen (entschiedenes Spiel beenden) | `settle` | `RunAction::SettleGame`, Sicht `can_settle`, `Run::can_settle`; Schranken der Reststiche in `endgame` (`Reach`, `Span`). Nicht verwechseln mit `Run::settle` (ein fertiges Spiel werten) |
| Schieben (Hausregel: einmal mehr einmischen) | `skat_redraw` | Effekt `ExtraRedeal` (liest auch `SeriesRedeal`): ein Einmischen mehr in jedem Spiel (DESIGN 8.1) |
| Neu geben (Einmischen, Kräuter, Doppelkorn) | `redeal` | Ereignis `Redealt`; `ConsumableAction::Redeal` (Kräuter), `ConsumableAction::RedealNextGame` (Doppelkorn: neu geben und das nächste angesagte Spiel ändern) |
| Skat neu ziehen (nur Spielebene) | `redraw_skat` | `Game::redraw_skat`: Gegner und Skat neu, eigene Hand bleibt. Von keinem Inhalt mehr genutzt |

## Wertung

| Deutsch | Bezeichner | Anmerkung |
|---|---|---|
| Spielwert | `game_value` | Grundwert × Stufen × Faktor |
| Grundwert | `base_value` | Karo 9, Herz 10, Pik 11, Kreuz 12, Grand 24 (Skatordnung, `GameType::base_value`); im Run seit 04.11.2026 Grand 18 (`RunConfig::base_value`, Daten `base_values` in `run.ron`, `BaseValues`; Wertung über `RunConfig::game_value_parts`, `ValueParts::with_base_value`) |
| Stufen (Gewinnstufen; 02.11. bis 04.11.2026 „Multiplikator“) | `levels` | Spitzen + 1 + Hand + Schneider + … |
| Faktor (Spielwert-Faktor, auch gebrochen, ×1,5; seit ADR 0018 die dritte Größe: Grundwert × Stufen × Faktor; das einzige Wort für Malnehmen in Spieltexten) | `multiplier` | `Factor` in `ScoreBreakdown`, `ScoreStep`, `ValuePreview` (JSON Dezimalzahl); ganze Faktoren `Change::Multiply`, gebrochene `Change::Scale` (Effekte `ScaleGameValue`, `ScaleGameValuePerCounter`, seit ADR 0020 `ScaleGameValuePerCard`). Faktoren werden multipliziert, der Spielwert wird am Ende halb aufgerundet. Anzeige-Gruppe der Relikte `RelicGroup::Factor` |
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
| Verloren | `lost` | Zählt im Run einfach negativ (`RUN_LOSS_FACTOR = 1`, seit 04.11.2026); nach Skatordnung doppelt (`rules::game_value`) |
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
| Boss-Kneipe (bis ADR 0016 Boss-Serie) | `is_boss_pub` | Letzte Kneipe eines Stadtteils; seit ADR 0017 zwei Boss-Spieler, alle drei Spiele, verloren endet der Run (`RunLost::boss`); bis dahin wurde sie verloren wiederholt |
| Leben (bis ADR 0019) | `lives` | entfallen am 04.11.2026: alte Spielstände lesen `lives` und übergehen es; `LivesConfig`, `LivesChanged`, `TourView::lives` gibt es nicht mehr |
| Route, Routenwahl | `route` | `RunPhase::Route`, `tour::RouteChoice` (nächste Kneipe und drei `RouteOption`), Aktion `RunAction::ChooseRoute(i)`, Ereignisse `RouteOpened`, `RouteChosen`; unterwegs `tour::Trip`. Sicht `TourView::route` (`RouteView`, `RouteOptionView`, `minutes` nur Dekoration) |
| Verkehrsmittel (Zu Fuß, Bahn, Fahrrad) | `Transport` (`Walk`, `Train`, `Bike`) | Seit 02.11.2026 alle gleich: je eine Station, nichts kostet extra; nur Darstellung |
| Station (unterwegs) | `RouteStop` | `category` und, außer beim Tresen, das gezogene Ereignis (`encounter`, verdeckt) |
| Stationsart: Tresen, Begegnung, Upgrade (Werkstatt), Rast, Fundgroschen, Kiosk, Würfelbude | `StopCategory` (`Tresen`, `Event`, `Upgrade`, `Rest`, `FoundCoins`, `Kiosk`, `DiceStall`) | Seit 03.11.2026 drei verschiedene je Routenwahl, eine der Tresen; die anderen nach `RouteConfig::weights` ohne Zurücklegen; Ereignisse tragen ihre Kategorie (`EncounterDef::category`) |
| Spelunke (Elite-Kneipe) | `Spelunke` | Eigener Ort mit eigenem Namen (`city.spelunken`), steht für eine normale Kneipe; Zielwert × `SpelunkeConfig::target_factor`, ein Malus, Beute ein Relikt (Ereignis `spelunke_loot`, `SpelunkeConfig::loot`, Wirkungen `Relic`, `RelicCoins`). In `Series::spelunke`, `RouteOption::spelunke`; Knoten-Id `d<S>-p<K>-s`, Sicht `SpelunkeNode` |
| Malus (Spelunke) | `MalusDef`, `MalusId` | Daten in `tour.ron`: `passive` (Tischregeln, Quelle `Source::Malus`), `no_redeal`, `lock_schnapps` (Fehler `ConsumablesLocked`), `curse` (`MalusCurse`, Karten nur für diese Kneipe, `Series::temp_curses`) |
| Die lange Nacht (Endlos-Modus nach dem Alten, ADR 0020) | `night` | `Run::night` (`night::NightState`: `verschaerfungen`, `bosses`, `extra_bosses`, `pubs_won`, eigener Generator), `Run::in_night`, Aktion `RunAction::EnterNight` (nur in `RunPhase::Won`), Ereignisse `NightStarted`, `VerschaerfungDrawn`, `ExtraBossRules`, `NightCursesLaid`, `RunLost { night: true }`; Sicht `RunView::night` (`NightView`), `RunView::can_enter_night`; Wachstum `night` in `run.ron` (`NightConfig::growth`, `NightConfig::grow`), Zielwert `Run::pub_target`; Modul `core/src/run/night.rs`. Stadtteil 4, 5, … heißen im Code weiter `section` (3, 4, …) |
| Verschärfung (der Nacht, „Die Sperrstunde rückt näher“) | `Verschaerfung` | Daten in `night.ron` (`VerschaerfungDef`, `VerschaerfungId`, Regel `NightRule`: `FewerRedeals`, `StrongerOpponents`, `DoubleBoss`, `PricierTresen`, `CursedCards`, `MalusEveryPub`, `TargetJump`, `FewerCardOffers`); Zähler `verschaerfungen` (`VERSCHAERFUNGEN_COUNTER`); Sicht `VerschaerfungInfo` |
| Doppelte Bosse (Verschärfung) | `DoubleBoss`, `extra_bosses` | Weitere Boss-Regeln der Boss-Kneipe der Nacht als Quellen `Source::Boss` (`Run::extra_boss_rules`) |
| Nachtschwärmer (Relikte der Nacht) | `RelicDef::night` | Nur in der Nacht angeboten (Boss-Wahl der Nacht, Tresen der Nacht); im normalen Run nur, wenn eine Freischaltung sie nennt und verdient ist (`Content::relic_unlockable`) |
| Bestenliste (je Charakter) | `CharacterStats::best_game`, `night_best_section`, `night_pubs_won` | Im Meta-Zustand; tiefster Stadtteil der Nacht (ab 1: 4 = Stadtteil 4), meiste Kneipen einer Nacht, bestes Spiel |
| Kartenrückseite als Belohnung (Kosmetik) | `Reward::CardBack(id)` | Daten `card_backs` in `night.ron` (`CardBackDef`), verdiente in `MetaState::card_backs`, Sicht `MetaView::card_backs`, Art `RewardKind::CardBack`; Bilder in `tools/cardart/backs.py` (`moon`, `stars`, `lantern`) |
| Stadt, Namensgenerator | `CityNames` | Wortlisten in `tour.ron`, `CityNames::generate`, `Run::city`; Karten-Seeds `Run::map_seed`, Sicht `TourView::map_seed`, `DistrictView::map_seed` (keine Geometrie im Kern) |
| Ansicht des Stadtteils (Altstadt, Hafen, Bahnhof) | `DistrictLook` (`OldTown`, `Harbour`, `Station`) | Daten je Stadtteilname in `tour.ron` (`DistrictName::look`); jeder Abend hat einen Stadtteil jeder Ansicht, Reihenfolge per Seed; Sicht `DistrictView::look`. Nur Anzeige |
| Rast (Imbissbude) | `snack_bar` | seit ADR 0019 `NextPubRedeals(1)` (Currywurst) oder `Cleanse` (Karte waschen); bis dahin `EncounterEffect::Lives(n)`, das nur noch alte Spielstände lesen (als `LEGACY_LIFE_COINS` Groschen) |
| Werkstatt (Upgrade) | `workshop` | Seit den Zinken (02.11.2026) der Kartenmaler gratis (`EncounterEffect::Painter`) oder Karte reinigen (`EncounterEffect::Cleanse`); seit 04.11.2026 auch der Griff in die Markenkiste (`EncounterEffect::Marke`) |
| Marke aus einer Begegnung (zufällige oder gewählte Marke auf zufälliger oder gewählter Karte) | `EncounterEffect::Marke { card, marke }` | `card`, `marke`: `CardPick::Random` oder `Chosen`; die gewählte Marke in `RunAction::ChooseEncounter { marke }`; Sicht `EncounterOptionView::marken` (`MarkeChoiceView`); Ereignis `MarkeGiven`; alle Marken `Run::marken` |
| Fundgroschen (Station, 03.11.2026) | `StopCategory::FoundCoins`, Ereignis `found_coins` | 2 Groschen auf dem Weg |
| Kiosk (Station, 03.11.2026) | `StopCategory::Kiosk`, `Shop::kiosk` | Kein Ereignis, ein Laden: `KioskConfig` (`route.kiosk` in `run.ron`), `Run::at_kiosk`, `Run::consumable_price`, Ereignis `KioskOpened`, Sicht `ShopView::kiosk` |
| Würfelbude (Station, 03.11.2026) | `StopCategory::DiceStall`, Ereignis `dice_stall` | Wirkung `EncounterEffect::Dice`, Ereignis `DiceThrown`; „Zuschauen“ ist eine Rückfall-Option (`fallback`) |
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
| Schnaps-Platz | `consumable_slots` | Seit 03.11.2026 drei (vorher zwei), `RunConfig::consumable_slots` in `run.ron`, Sicht `RunView::consumable_slots` |
| Ziel eines Schnapses (Schnaps-Leiste, 03.11.2026) | `ConsumableTarget` | Worauf die Oberfläche ihn gießt: `Skat`, `Table` (Tisch), `Card` (Handkarte), `Player` (der Spieler selbst, serialisiert `Self`), `Map` (Stadtplan). Feld `target` an `ConsumableDef` (Standard `Self`) und `ConsumableInfo`; nur Darstellung |
| Jetzt trinkbar | `usable_now` | Feld an `OwnedConsumable` (bis 03.11.2026 `can_use`): ob der Kern `UseConsumable` jetzt annähme, mit der echten Aktion auf einer Kopie geprüft (`Run::accepts`) |
| Schnaps nach gewonnener Kneipe (03.11.2026) | `schnapps_reward` | Modul `core/src/run/schnapps_reward.rs`: `Run::reward_schnapps` (eigener Zufallsstrom `schnapps_rng`), bei vollen Plätzen wartend `Run::schnapps_reward`, Aktion `RunAction::ChooseSchnappsReward { give_up }` (einen abgeben oder verkaufen), Ereignisse `SchnappsRewarded { consumable, slots_full }`, `SchnappsRewardTaken { consumable, given_up }`, beim Verkauf `ConsumableSold`; Sicht `RunView::schnapps_reward` (`SchnappsRewardView`); Fehler `SchnappsRewardPending` |
| Wirkung aufs nächste angesagte Spiel (Doppelkorn) | `NextGameEffect` | `Multiply(n)`, `SingleLoss` (ruhend, früher Magenbitter); Aktionen `ConsumableAction::RedealNextGame`, `ConsumableAction::NextGame` (ruhend), wartend `Run::pending_next_game` |
| Skat ansehen (Kurzer) | `PeekSkat`, `SkatPeeked` | Feld `ActiveGame::skat_peeked`, Sicht `GameView::skat_peek` |
| Skatkarte zeigen (Spickzettel) | `skat_shown` | Effekt `RevealSkatCard(n)` (Information, bei der Ansage eines Handspiels), Feld `ActiveGame::skat_shown`, Run-Ereignis `SkatCardShown`, Sicht `GameView::skat_shown` |
| Umgefärbte Karte (Blue Curaçao, bis 02.11.2026 Kreide) | `chalk`, `ChalkCard`, `CardChalked` | `Game::chalk`, Feld `ActiveGame::chalked` (Id und gedruckte Farbe), Sicht `CardView::chalked`; Fehler `CannotChalk` |
| Faktor eines verlorenen Spiels | `Lever::LossFactor`, `content::RUN_LOSS_FACTOR` | seit 04.11.2026 1 nach den Run-Regeln (verloren zählt einfach; die Skatordnung und `rules::game_value` sagen 2); `NextGameEffect::SingleLoss` und `Effect::SingleLoss` ruhen |
| Boss (Gegnerpaar mit Sonderregel) | `Boss` | Definition `BossDef`, in `Series::boss` |
| Seltenheit | `Rarity` | `Common`, `Uncommon`, `Rare`, `Legendary` (Legendär, M8) |
| Gewicht der Stufe Legendär | `RarityWeights::legendary` | Dezimalzahl (Standard 0,3), `RarityWeights::percent_weight` |
| Immer Vorhand (Ehrenvorsitz) | `AlwaysForehand` | Effekt, nur passiv |
| Stufen je Zählerpunkt (Skatgott) | `AddLevelsPerCounter` | `{ counter, per }`, wie `AddBaseValuePerCounter` für die Stufen |
| Preis | `price` | Groschen |
| Verkaufen (halber Preis) | `sell` | |
| Neuauslage (Tresen, bis 05.11.2026) | `reroll` | Relikte und Gegenstände neu, die Auslage blieb; seit 05.11.2026 in „Neu auslosen“ aufgegangen |
| Der Duzfreund (Figur: Neu auslosen gratis, 05.11.2026) | `first_names` | Wirkung `Effect::FreeReroll`, weiter einmal je Besuch |
| Grundbetrag (gewonnene Kneipe) | `series_reward` | |
| Zinsen | `interest` | |
| Auszahlung einer Kneipe | `Payout` | Grundbetrag, Bonus für übrige Spiele (`per_game_left`), Zinsen; eine verlorene Kneipe zahlt nichts |
| Run-Aktion (Entscheidung des Spielers) | `RunAction` | |
| Run-Ereignis | `RunEvent` | |
| Flagge (Relikt-Zustand, z. B. Bock) | `flag` | Pro Run |
| Zähler (Relikt-Zustand, z. B. Tournee) | `counter` | Pro Run. Zähler einer Serie setzt `ResetCounter` nach der gewonnenen Serie zurück (Ereignis `CounterReset`); `AddBaseValuePerCounter { counter, per }` gibt `per` Grundwert je Punkt |
| Relikt (Oberbegriff) | `Relic` | HouseRule und Regular, Definition `RelicDef` |
| Relikt-Plätze | `relic_slots` | Seit 05.11.2026 acht (Hausregeln und Figuren zusammen, Entscheidung des Nutzers q34 A; 03.11. bis 05.11.2026 unbegrenzt, davor fünf): `RunConfig::relic_slots: Option<u32>` in `run.ron` (`Some(8)`; ohne Angabe keine Grenze), Obergrenze eines Charakters `CharacterRule::relic_slots`, es gilt die kleinere; `Content::relic_slots` (`None` = unbegrenzt), `Content::relic_slot_free`; Sicht `RunView::relic_slots` (`null` = unbegrenzt) |
| Relikt tauschen (volle Plätze) | `ReplaceRelic`, `replace` | Ein neues Relikt ersetzt ein gehaltenes, das für den halben Preis verkauft wird (`sell_price`, Ereignis `RelicSold`). Tresen: `RunAction::ReplaceRelic { slot, replace }` (alles oder nichts), Sicht `RelicOffer::replace` (Indizes, die es jetzt ersetzen kann), Oberfläche „Tauschen“; Boss-Belohnung (auch der Nacht): `ChooseBossRelic { offer, replace }`, Sicht `BossRewardView::slots_full`; Begegnung mit Relikt (Beute der Spelunke): `ChooseEncounter { target }` mit dem Index des gehaltenen Relikts, `Run::option_needs_relic`, `EncounterOption::gains_relic` |
| Kasse auszahlen, wenn das Relikt geht (Stammtischkasse) | `StashPaidOut` | Ereignis `StashPaidOut { relic, amount, coins }`: Verkauf, Tausch, Ersatz oder Ende einer Abendregel zahlt den `StashCoins`-Topf des Relikts aus, wie er ist |
| Tresen | `Shop` | |
| Groschen | `coins` | Währung |
| Groschen erhalten (Effekt) | `GainCoins` | Ereignis `CoinsGained { source, amount, coins }`; negativ zieht ab, nie unter 0 |
| Beim Erhalt (Kartenmodifikation angebracht) | `OnAttach` | Auslöser nur für Kartenmodifikationen, Stufe `Stage::OnAttach` |
| Wenn gedrückt (Gutschein) | `OnDiscard` | Auslöser nur für Kartenmodifikationen, Stufe `Stage::OnDiscard`; die Karte landet beim Drücken im Skat |
| Fluch (negative Kartenmodifikation) | `curse` | `ModKind::Curse`, Prüfung `CardModDef::is_curse` (in Sicht und Katalog weiter `curse`); Text beginnt mit „Fluch:“. Sperrt die Karte für jede weitere Modifikation (Effekte v2). Nie aus Zufallsziehungen, nur über Ereignisse, deren Text „Fluch“ sagt (`MarkCard { curse: true }` zieht einen Fluch), und seit 03.11.2026 als Preis eines Handels (Schnäppchen, Hehlerware, Fusel, `TradeCurse`); zahlt beim Erhalt Groschen (M7), nicht als Preis eines Handels (`Run::attach_mod_paid` mit `paid: false`) |
| Ereignis / Rast | `Encounter` | Station unterwegs (ADR 0016). Nicht `Event` (das sind die Zustandsereignisse). Definition `EncounterDef` mit Optionen `EncounterOptionDef` (`label`, `cost`, `effects`, `fallback`), Wirkung `EncounterEffect`, offen `ActiveEncounter` mit `EncounterOption`, Aktion `RunAction::ChooseEncounter` (Ziel: Karte oder Index eines Relikts), Ereignis `EncounterResolved { option }`. Seit 03.11.2026 ohne „Weitergehen“ (bis dahin `EncounterKind`, `EncounterChoice`) |
| Option gesperrt (Grund) | `OptionBlock` | `NotEnoughCoins`, `NothingLeft`, `SpecialCardCap`, `NoFittingCard`, `NoRelic`, `RelicSlotsFull`, `LivesFull`, `BetOpen`, `NotNeeded`; `Run::encounter_option_block`, Sicht `EncounterOptionView::blocked` |
| Zinke auf zufällige oder gewählte Karte (Ereignis) | `CardPick` (`Random`, `Chosen`) | Feld `card` der Wirkung `EncounterEffect::Mark` |
| Beide Skatkarten sehen (Kartentrick, Theorie) | `SeeSkat`, `Run::peek_next_skat` | Wie der Kurzer bei jedem Geben bis zur Ansage; Ereignisse `SkatPeekPlanned`, `SkatPeeked` |
| Hausregel des Stadtteils (Stammtisch-Streit, bis ADR 0016 des Abends) | `evening_relics` | Gilt bis zum Ende des Stadtteils, ohne Platz |
| Charakter | `Character` | `Declarer` (Alleinspieler), `NullPlayer` (Nullspieler), `Shirker` (Drückeberger), `Tailoress` (Schneiderin, nicht die Figur „Der Schneider“ `tailor` und nicht Schneider `schneider`), `HeartBrother` (Bruderherz); Daten `CharacterDef`, Id als Text `Character::as_str`, gelesen mit `Character::from_id`. Bis 03.11.2026 auch `GrandPlayer` (Grandspieler), `Onlooker` (Zaungast), `Landlady` (Wirtin); alte Meta-Stände lassen deren Einträge fallen |
| Eigene Regel eines Charakters | `CharacterRule` | Feld `rule` in `CharacterDef`: `passive`, `hooks` (Hook-API wie ein Relikt ohne Platz, Quelle `Source::Character`), `redeals` (Einmischen pro Spiel) und `relic_slots` (Obergrenze `Some(n)`, die kleinere mit `run.ron` gilt; bis 03.11.2026 ein Zuschlag), `payout_factor`, `vouchers_return`, `keep_curses`, `zinken_discount`. Werte eines Runs: `Content::redeals`, `Content::relic_slots`, `Content::payout_factor` |
| Zuschlag der Auszahlung (bis 03.11.2026 die Wirtin) | `Payout::bonus` | Was `payout_factor` über die einfache Auszahlung hinaus zahlt |
| Blatt eines Gegners sehen (bis 03.11.2026 der Zaungast) | `RevealDefenderHand(seat)` | Informations-Wirkung, Platz 1 (links) oder 2; Feld `ActiveGame::shown_hand`, Sicht `RevealSource::Onlooker` |
| Start-Verbrauchsgegenstände | `start_consumables` | Feld in `CharacterDef` |
| Eingelöste Gutscheine kommen nach der Kneipe zurück (Drückeberger) | `vouchers_return` | Feld in `CharacterRule`; gemerkt in `Series::returning_vouchers`, Ereignis `RunEvent::VoucherReturned { card, card_mod, value }` |
| Flüche lassen sich nicht reinigen (Bruderherz) | `keep_curses` | Feld in `CharacterRule`; Fehler `RunError::CurseStays` (Code `CurseStays`), Prüfung `Run::card_is_cursed` |
| Zinken-Rabatt (Bruderherz: Rot zum halben Preis) | `zinken_discount` | `Option<ZinkenDiscount { cards: CardFilter, percent }>` in `CharacterRule`; Preis `Run::zinken_price`, Sicht `EncounterOptionView::target_costs` |
| Rückseite eines Charakters | `back` | Feld in `CharacterDef` (ohne Angabe `PLAYER_BACK`, die Distel); `shirker`, `tailoress`, `heart_brother`; Sicht `CharacterInfo::back`, `CharacterEntry::back` |
| Gedrückte Augen eines Spiels | `discarded_points` | Feld in `RunEvent::GameScored`, 0 im Handspiel; Freischaltung `WinGame(min_discarded)` |
| Schneider-Siege in einer Kneipe | `SchneiderWinsInPub(n)` | Freischaltungs-Bedingung, Zähler `MetaState::pub_schneider_wins` |
| Gleiche Trumpffarbe hintereinander gewonnen | `SameTrumpStreak(n)` | Freischaltungs-Bedingung, über Kneipen im selben Run; Zähler `MetaState::trump_streak` |
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
| Schummeln (Boss-Regel) | `is_cheat` | Zwillinge (bis ADR 0017 auch Revolutionär); abgeschaltet durch `NoCheating` |
| Informations-Effekt | `is_information` | Durchsichtig, Kiebitz, Zähler, Spickzettel; abgeschaltet durch `DisableInformation` |
| Rabatt am Tresen (Wirt) | `shop_price`, `ShopDiscount` | |
| Kiebitz-Blick | `peek`, `CardPeeked` | |
| Für ein Spiel durchsichtig (Glasauge) | `RevealDefenderCards`, `transparent_for_game`, `CardMadeTransparent` | nur Karten der Gegner, nie die Ausgabe |
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
| Auslöser (Zeitpunkt) | `Hook`, `Stage` | `OnDeal`, `OnDeclare`, `OnCardPlayed`, `OnTrickEnd`, `OnScore`, `OnSeriesWon`, `OnRouteChosen` (Weg gewählt, Bedingung `TransportIs`), `OnShopOpened` (Tresen geöffnet), dazu `Passive` |
| Stufen (Stufe, Gewinnstufe) | `levels` | Spieltexte sagen seit 04.11.2026 wieder „Stufen“ („+1 Stufe“, „+2 Stufen“, Box „Stufen“, Formel „Grundwert × Stufen × Faktor“); vom 02.11. bis 04.11.2026 hieß es „Multiplikator“. Im Code bleiben `levels`, `AddLevels`, `Lever::Levels`, `LevelParts`. Zeichen: eine Treppe aus drei Stufen (Band-Glyphe und großes Werte-Icon `levels`, kupferfarben). Nicht verwechseln mit dem Faktor (`multiplier`, ×1,5), der Risikostufe und dem Rang einer Karte (Winkel: „sticht einen Rang höher“) |
| Anzeige-Gruppe eines Relikts | `RelicGroup` | `Value` (Wert), `Multiplier` (Stufen), `Control` (Kontrolle), `Tour`; Feld `group` in `house_rules.ron`, Standard `Value`; Sicht `RelicInfo::group`, Katalog `group`. Nur Anzeige (Form des Bierdeckels) |
| Plan für die nächste Kneipe | `NextPubPlan` | `redeals`, `target`; Wegbier, Skatlehrer, Kiebitz am Fenster; `Run::next_pub_plan`, Sicht `next_pub`; Ereignisse `NextPubPlanned`, `NextPubApplied`; Aktion `ConsumableAction::NextPub` |
| Anschreiben (Schulden am Tresen) | `debt`, `Credit` | Effekt `Credit(n)` (Deckel anschreiben), `Run::debt`, Sicht `debt`, `credit_left`, `Shop::on_credit`; Ereignis `DebtChanged`; Einnahmen zahlen zuerst die Schulden |
| Kasse (Topf aus Groschen) | `StashCoins`, `PayOutCounter` | Stammtischkasse: Zähler `kitty`; Ereignis `CoinsStashed` |
| Zweite Chance | `SecondChance` | Schutzengel seit ADR 0019: einmal im Run eine verlorene Kneipe neu spielen (`Run::replay_pub`), Ereignis `SecondChance { relic, section, index }`. Alter Name `SpareLife` (einmal pro Stadtteil kein Leben) wird gelesen |
| Malus wählen | `ChooseMalus` | Effekt `ChooseMalus(n)` (Spelunkenkenner), `Spelunke::malus_options`, Aktion `RunAction::ChooseMalus { option, malus }`, Ereignis `MalusChosen` |
| Wege neu auslosen | `RerollRoutes` | Schwenker; Ereignis `RoutesRerolled` |
| Leben trinken (bis ADR 0019) | `GainLives` | Konterbier, entfallen; Spielstände bekommen den Preis zurück (`REFUNDED_CONSUMABLES`). Fehlercode `LivesFull` bleibt stehen |
| Kneipe verlassen (aufgeben) | `GiveUpPub` | `RunAction::GiveUpPub`, Ereignis `PubGivenUp` (dann `PubLost`), `Run::can_give_up`, Sicht `RunView::can_give_up`; nur `Run::between_games` und ab dem zweiten Spiel (DESIGN 8.7) |
| Zwischen zwei Spielen | `between_games` | `Run::between_games`: das gegebene Spiel ist unberührt (zehn Karten, Skat zu, nichts angesagt, keine Karte gespielt) |
| Noch nötig | `needed` | Sicht `RunView::needed` (`NeededView`: `points`, `smallest_game`, `max_value`, `reachable`) vor dem letzten Spiel; Obergrenze `Run::max_game_value` |
| Konto auf null (Klarer) | `ClearAccount` | Ereignis `AccountCleared`; Fehler `ConditionNotMet` |
| Ein Spiel mehr (Absacker) | `ExtraGames` | Ereignis `GamesAdded`; Fehler `NotInBossPub` |
| Nächstes Spiel, wenn zurück (Eierlikör) | `NextGameIfBehind` | Konto unter der Hälfte des Zielwerts; Wirkung `NextGameEffect::MultiplyWin` (nur gewonnen), `ActiveGame::win_factors`; Fehler `ConditionNotMet` |
| Wette | `Wager` | Die Wette: `Run::wager`, Ereignisse `WagerPlaced`, `WagerSettled`; worauf: `BetOn` (`Schneider`, `Win`), Auszahlung `Wager::percent` |
| Wege auskundschaften | `scout_legs` | Kiebitz am Fenster: `EncounterEffect::Scout`, Sicht `StopView::encounter`; Ereignis `StopsRevealed` |
| Spielerei (Falschspieler am Nebentisch) | `Gamble` | `EncounterEffect::Mark` mit `curse_percent`, Ereignis `GambleResolved` |
| Entfernte Inhalte beim Laden | `drop_unknown_content` | `Run::drop_unknown_content` (wasm `Session::load`), Umbenennungen `RENAMED_CONSUMABLES` |
| Effekt | `Effect` | Geschlossener `enum`, jeder Effekt nennt seinen Hebel |
| Bedingung | `Condition` | Geschlossener `enum` |
| Stich gewonnen mit (Luschenkönig) | `TrickWinnerMatches` | `filter`, `by_declarer`: der eben beendete Stich, nur bei Stichende |
| Erster Stich des Alleinspielers (Stichfest) | `FirstTrickOfDeclarer` | Der eben beendete Stich ist der erste, den der Alleinspieler macht |
| Kartenfilter | `CardFilter` | Farbe und/oder Wert |
| Hebel der Formel | `Lever` | `BaseValue`, `Levels`, `GameValue`, `CardPoints`, `WinThreshold`, `Schneider`, `LossFactor` |
| Herkunft eines Zuschlags | `Source` | Relikt, Kartenmodifikation, Boss, Kontra, Re, Ereignis, Verbrauchsgegenstand (`Consumable`) |
| Zeile der Abrechnung | `ScoreLine` | Herkunft, Hebel, Änderung |
| Abrechnung (Spielwert mit allen Zuschlägen) | `ScoreBreakdown` | |
| Stufen Term für Term | `LevelParts` | |
| Schritt der Abrechnung | `ScoreStep` | Ein Term mit laufendem Grundwert, Stufen und Faktor, für die schrittweise Anzeige |
| Term der Abrechnung | `ScoreTerm` | Grundwert, Spitzen, Spiel, Hand, …, oder eine `ScoreLine` |

## Content-IDs

Content-IDs sind englisch und snake_case. Die deutschen Namen stehen in den RON-Dateien.

| Deutsch | ID |
|---|---|
| Spitze (Hausregel, letzter Stich mit einer Sieben) | `last_trick_seven` (nicht verwechseln mit Spitzen = `matadors`) |
| Schenken | `concede` |
| Schieben | `skat_redraw` |
| Jungfrau | `virgin` |
| Alte Oberin (bis 02.11.2026 im Code „Alte Dame“) | `old_lady` |
| Tournee | `tour` |
| Aufdruck (Zinke Augen; bis 02.11.2026 „Augen“) | `points` (nicht verwechseln mit Augen = `card_points`) |
| Winkel (Zinke Rang; bis 02.11.2026 „Rang“; seit 03.11.2026: wird die höhere Karte) | `rank` |
| Blei | `lead` |
| Goldkante | `gilt_edge` |
| Fadenkreuz | `crosshair` |
| Schneiderkreide | `tailors_chalk` |
| Doppelgänger | `doppelganger` |
| Siegel (Zinke Trumpf; bis 02.11.2026 „Trumpf“) | `trump_bonus` |
| Kerbe (Zinke Letzter Stich; bis Effekte v2 „Spitze“, bis 02.11.2026 „Letzter Stich“) | `last_trick` |
| Eselsohr (Zinke Erster Stich; bis 02.11.2026 „Erster Stich“) | `first_trick` |
| Pfandmarke (Gutschein Groschen; bis 02.11.2026 „Gutschein“) | `voucher_coins` |
| Totenmarke (bis 02.11.2026 „Gutschein Untot“) | `voucher_undead` |
| Narrenmarke (bis 02.11.2026 „Gutschein Wandelkarte“) | `voucher_wild` |
| Durchsichtig (Fluch seit Effekte v2) | `transparent` |
| Fleckig (Fluch, bis Effekte v2 „Schwarze Katze“) | `stained` |
| Gestrichen mit Effekte v2 (alte Ids, nur noch beim Umrechnen) | `fat`, `marked`, `stamp`, `invisible`, `gold_rim`, `double_marked`, `crown`, `seal`, `horseshoe`, `tape_measure`, `lucky_penny`, `beer_stain`, `cracked`, `black_cat`, `lead_weight`, `tail_light`, `dressed_up`, `dummy`, `honorary_jack`, `glass_ace`, `homesick` |
| Der Schieber (mit Die Hehlerin) | `the_pusher` |
| Der Kontra-Hans (entfernt, ADR 0017) | `kontra_hans` |
| Die Oma (Boss, mit Der Opa; bis 02.11.2026 auch eine Figur `granny`) | `grandma` |
| Kurzer | `shot` |
| Doppelkorn | `double_grain` |
| Kräuter | `herbal` |
| Blue Curaçao (bis 02.11.2026 Kreide) | `blue_curacao` (alte Id `chalk` wird beim Laden umbenannt) |
| Der Kiebitz | `kibitzer` |
| Der Wirt | `landlord` |
| Der Zähler (Figur) | `card_counter` (nicht `counter`, das ist der Relikt-Zustand) |
| Die Zwillinge (Fritz und Franz) | `twins` |
| Der Ramscher (mit Die Ramscherin; jedes Spiel ein Ramsch, Wirkung `Ramsch`, alter Name `RamschOnce`; seit 04.11.2026 zählt jedes verlorene Spiel einfach) | `ramscher` |
| Der Revolutionär (entfernt, ADR 0017) | `revolutionary` |
| Der Schwarze Peter (mit Die Schwarze Grete) | `black_peter` |
| Der Kiebitz-Mörder (entfernt, ADR 0017) | `kibitz_killer` |
| Der Alte (mit Die Alte) | `the_old_man` |
| Stammtisch-Streit | `stammtisch_dispute` |
| Der Kartengeber | `card_dealer` |
| Alter Bekannter | `old_friend` |
| Letzte Runde | `last_round` |
| Kleinvieh | `every_bit_counts` |
| Ein Auge zudrücken | `blind_eye` |
| Glasauge (Hausregel: je Gegner eine Karte für ein Spiel durchsichtig; Effekt `RevealDefenderCards`, Ereignis `CardMadeTransparent`, `ActiveGame::transparent_for_game`) | `glass_eye` |
| Eichelhäher (Faktor ×1,5 auf gewonnene Eichel-Spiele, ADR 0018) | `jay` |
| Laubfrosch (Faktor ×1,5 auf gewonnene Grün-Spiele) | `tree_frog` |
| Rotkehlchen (Faktor ×1,5 auf gewonnene Rot-Spiele) | `robin` |
| Schellenbaum (Faktor ×1,5 auf gewonnene Schellen-Spiele) | `bell_tree` |
| Der Grandseigneur (Figur, Faktor ×2 auf gewonnene Grands) | `grand_seigneur` |
| Schwarze Null (Faktor ×1,5 auf gewonnene Null-Spiele) | `black_zero` |
| Zapfhahn (+×0,1 je gewonnener Kneipe; Zähler `tap`) | `tap` |
| Heiße Hand (+×0,15 je Sieg in Folge, zurück auf ×1 nach einer Niederlage; Zähler `hot_hand`) | `hot_hand` |
| Bierdeckel | `beer_mat` (Zähler `beer_mat`) |
| Glückssträhne | `lucky_streak` (Flaggen `streak_start`, `streak`) |
| Der Schneider (Figur) | `tailor` (nicht verwechseln mit Schneider = `schneider`) |
| Nullachtfuffzehn | `null_eight_fifteen` |
| Dicke Sau | `fat_sow` |
| Umgekipptes Bier | `spilled_beer` |
| Gasthof zur Krone | `crown_inn` |
| Der Taschenspieler | `conjurer` |
| Preisskat | `prize_skat` |
| Gast aus Altenburg | `guest_from_altenburg` |
| Ramponiert (Fluch, seit Effekte v2 −X Augen: Augen mit negativem Wert, Konstante `deck::RAMPONIERT` neben `deck::AUGEN`) | `battered` |
| Mit Vieren | `with_four` |
| Skatpfleger | `skat_keeper` |
| Allrounder | `all_rounder` (Zähler `all_rounder`, Flaggen `all_rounder_suit`, `_grand`, `_null`) |
| Stammfarbe | `home_suit` (Zähler `home_suit`, Flaggen `home_suit_clubs`, `_spades`, `_hearts`, `_diamonds`) |
| Luschenkönig | `blank_king` (Lusche = `blank`, nicht Blanko) |
| Stichfest | `watertight` |
| Drei drücken | `three_discard` (Freischaltung `ten_runs`, „Stammkunde“) |
| Spickzettel | `crib_sheet` |
| Wandelkarte | `wild_card` |
| Untot | `undead` |
| Skatgott | `skat_god` (Zähler `skat_god`) |
| Ehrenvorsitz | `honorary_chair` |
| Konterbier (entfallen mit ADR 0019) | `counter_beer` |
| Schwenker | `snifter` |
| Wegbier | `road_beer` |
| Klarer | `clear_spirit` |
| Absacker | `nightcap` |
| Eierlikör | `eggnog` |
| Feierabend | `closing_time` |
| Monatskarte | `season_ticket` |
| Stammtischkasse | `kitty` (Zähler `kitty`) |
| Deckel anschreiben (Figur) | `slate` |
| Schutzengel (seit ADR 0019 die zweite Chance) | `guardian_angel` |
| Spelunkenkenner (Figur) | `dive_expert` |
| Nachteule (Nacht) | `night_owl` (Zähler `verschaerfungen`) |
| Kehraus (Nacht) | `sweep_out` (Zähler `sweep_out`) |
| Lumpensammler (Nacht) | `rag_collector` |
| Mondschein (Nacht) | `moonlight` (Zähler `moonlight`) |
| Nachtschicht (Nacht) | `night_shift` (`CardCount::direction`) |
| Morgengrauen (Nacht) | `dawn` (Flagge `dawn`) |
| Kurze Geduld, Hellwache Gegner, Doppelte Bosse, Nachtzuschlag, Verhexte Karte, Spelunkenluft, Zielwert-Sprung, Knausriger Wirt (Verschärfungen) | `fewer_redeals`, `sharp_opponents`, `double_boss`, `pricier_tresen`, `cursed_card`, `dive_air`, `target_jump`, `stingy_host` |
| Tief in der Nacht, Nach der Sperrstunde, Bis zum Morgengrauen (Freischaltungen) | `night_district_5`, `night_district_7`, `night_district_10` |
| Der Wirt spendiert | `landlord_treat` |
| Falschspieler am Nebentisch (bis 02.11.2026 „Der Falschspieler“) | `card_sharp` |
| Verlorenes Portemonnaie | `lost_wallet` |
| Der Skatlehrer | `skat_teacher` |
| Die Wette | `the_bet` |
| Alte Karten im Schrank | `old_cards` |
| Der Pfandleiher | `pawnbroker` |
| Die Markensammlerin | `stamp_collector` |
| Kiebitz am Fenster | `lookout` |
| Fundgroschen | `found_coins` |
| Die Würfelbude | `dice_stall` |
| Entfernt im Gegenstands-Review (02.11.2026; alte Ids fallen beim Laden weg, Definitionen nur noch für Tests in `core/src/run/legacy_*.ron`) | Relikte `bock_round`, `kontra`, `ouvert_bonus`, `march_through`, `blind_flight`, `granny`, `grandpa`, `referee`, `show_off`, `regular_guest`, `tuition`, `high_and_dry`, `schneider_bock`, `safety_net`, `glass_house`, `tally_stick`, `seven_sleeper`, `clean_slate`, `old_deck`, `changeling`, `jack_parade`, `decoy`; Verbrauchsgegenstände `beer`, `new_deal`, `bitters`, `regret`, `points_5`, `trump_4` |
| Entfernt mit den Zinken am Tresen (02.11.2026; alte Ids fallen beim Laden weg, Definitionen nur noch für Tests in `core/src/run/legacy_consumables.ron`) | Verbrauchsgegenstände `fresh_deck` (Frisches Deck, jetzt „Karte reinigen“), `ink_pad` (Stempelkissen), `points_3` (Augen +3), `trump_2` (Trumpf +2), `rank_1` (Rang +1) |

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

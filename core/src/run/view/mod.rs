//! Read-only view of a run for frontends (the web app via `skat_wasm`).
//!
//! Everything the table, the Ansage dialog, the Tresen and the score
//! screen show comes from here, already decided by the core: the hand in
//! display order, which cards are legal, which declarations are allowed,
//! prices, what can be bought, sold or used. A frontend only renders it.
//!
//! The view never changes the run. Whether an action is allowed is found
//! out by trying it on a copy of the run, so the view can never disagree
//! with [`Run::apply`].

//!
//! The data types are in `types`; the parts of the view are built in
//! `game` (the table), `shop` (the Tresen), `encounter`, `tour` (the city
//! map and the route choice) and `night` (the long night).

mod encounter;
mod game;
mod night;
mod shop;
mod tour;
mod types;

pub use types::{
    BossFigureInfo, BossInfo, BossRelicOffer, BossRewardView, CardModInfo, CardOfferView,
    CardRewardOfferView, CardRewardView, CardView, CharacterInfo, ConsumableInfo, ConsumableOffer,
    CountView, DeclareOptions, EditionEntry, EncounterEffectView, EncounterOptionView,
    EncounterView, GameOption, GameView, MarkeChoiceView, NeededView, NextSeriesView,
    NullRiskOption, OptionTarget, OwnedConsumable, OwnedRelic, PainterOptionView, PainterView,
    PendingNextGame, PlayedCard, RelicInfo, RelicOffer, RevealSource, RevealedCard, RiseChoiceView,
    RiskOption, RunView, SchnappsRewardView, ScoreView, SeriesView, ServiceView, SettledWild,
    ShopView, TablePhase, TrickView, ValuePreview,
};

use super::guests::GuestInfo;
use super::{
    EncounterOption, Kontra, OptionBlock, PUBS_PER_DISTRICT, Run, RunAction, RunPhase, RunStats,
    sell_price,
};
use crate::content::{
    BossId, CardModId, ConsumableAction, ConsumableCategory, ConsumableId, ConsumableTarget,
    Content, Factor, ModKind, Rarity, RelicId, RelicKind,
};

impl Run {
    fn relic_info(content: &Content, id: &RelicId) -> RelicInfo {
        match content.relic(id) {
            Some(def) => RelicInfo {
                id: id.clone(),
                kind: def.kind,
                name: def.name.clone(),
                text: def.text.clone(),
                rarity: def.rarity,
                price: def.price,
                group: def.group,
            },
            None => RelicInfo {
                id: id.clone(),
                kind: RelicKind::HouseRule,
                name: id.to_string(),
                text: String::new(),
                rarity: Rarity::Common,
                price: 0,
                group: crate::content::RelicGroup::Value,
            },
        }
    }

    fn card_mod_info(content: &Content, id: &CardModId, value: Option<i32>) -> CardModInfo {
        match content.card_mod(id) {
            Some(def) => {
                let value = def
                    .has_value()
                    .then(|| value.unwrap_or_else(|| def.default_value()));
                CardModInfo {
                    id: id.clone(),
                    name: def.name.clone(),
                    text: def.text_for(value.unwrap_or(1)),
                    rarity: def.rarity,
                    curse: def.is_curse(),
                    kind: def.kind,
                    value,
                }
            }
            None => CardModInfo {
                id: id.clone(),
                name: id.to_string(),
                text: String::new(),
                rarity: Rarity::Common,
                curse: false,
                kind: ModKind::Edge,
                value,
            },
        }
    }

    fn consumable_info(content: &Content, id: &ConsumableId) -> ConsumableInfo {
        match content.consumable(id) {
            Some(def) => ConsumableInfo {
                id: id.clone(),
                name: def.name.clone(),
                text: def.text.clone(),
                rarity: def.rarity,
                price: def.price,
                needs_target: matches!(
                    def.action,
                    ConsumableAction::CleanseCard
                        | ConsumableAction::AddCardMod(_)
                        | ConsumableAction::ApplyCardMod { .. }
                ),
                needs_suit: def.action == ConsumableAction::ChalkCard,
                category: def.category,
                target: def.target,
            },
            None => ConsumableInfo {
                id: id.clone(),
                name: id.to_string(),
                text: String::new(),
                rarity: Rarity::Common,
                price: 0,
                needs_target: false,
                needs_suit: false,
                category: ConsumableCategory::Item,
                target: ConsumableTarget::default(),
            },
        }
    }

    /// A boss as the run's character meets it (with its Zielwert factor
    /// for that character).
    pub(super) fn boss_info(&self, content: &Content, id: &BossId) -> BossInfo {
        match content.boss(id) {
            Some(def) => {
                let back = def
                    .back
                    .clone()
                    .unwrap_or_else(|| crate::content::OPPONENT_BACK.to_owned());
                BossInfo {
                    id: id.clone(),
                    name: def.name.clone(),
                    text: def.text.clone(),
                    target_factor: def.target_factor_for(self.character),
                    figures: def
                        .figures
                        .iter()
                        .map(|f| BossFigureInfo {
                            name: f.name.clone(),
                            edition: f
                                .edition
                                .iter()
                                .map(|(filter, card_mod, value)| EditionEntry {
                                    filter: *filter,
                                    card_mod: Run::card_mod_info(content, card_mod, Some(*value)),
                                })
                                .collect(),
                            back: f.back.clone().unwrap_or_else(|| back.clone()),
                        })
                        .collect(),
                    back,
                }
            }
            None => BossInfo {
                id: id.clone(),
                name: id.to_string(),
                text: String::new(),
                target_factor: Factor::ONE,
                figures: Vec::new(),
                back: crate::content::OPPONENT_BACK.to_owned(),
            },
        }
    }

    /// The pub after the current one: where the way leads (a Spelunke's
    /// target with its factor), else the regular next pub. A new
    /// district's boss is not drawn yet, so its first pub shows none.
    fn next_series_view(&self, content: &Content) -> Option<NextSeriesView> {
        if matches!(self.phase, RunPhase::Won | RunPhase::Lost) {
            return None;
        }
        let (section, index, spelunke) = match (&self.trip, &self.route) {
            (Some(trip), _) => (trip.section, trip.index, trip.spelunke.is_some()),
            (None, Some(route)) => (route.section, route.index, false),
            (None, None) => {
                let (s, i) = self.next_series_position();
                (s, i, false)
            }
        };
        if !self.district_exists(content, section) {
            return None;
        }
        let is_boss = index + 1 == PUBS_PER_DISTRICT;
        let boss = if is_boss && section == self.series.section {
            self.section_boss.as_ref()
        } else {
            None
        };
        let mut target = self.pub_target(content, section, index, boss)?;
        if spelunke {
            target = content.run.spelunke.target_factor.apply(target);
        }
        Some(NextSeriesView {
            section,
            index,
            is_boss,
            target,
            game_limit: content.run.game_limit,
            redeals: content.redeals_in(self.character, section),
            boss: boss.map(|id| self.boss_info(content, id)),
        })
    }

    /// The read-only view of the run for a frontend.
    #[must_use]
    pub fn view(&self, content: &Content) -> RunView {
        let series = &self.series;
        let boss = series.boss.as_ref().map(|id| self.boss_info(content, id));
        let boss_ahead = match self.phase {
            RunPhase::Game => true,
            RunPhase::Shop | RunPhase::Encounter | RunPhase::Route | RunPhase::CardReward => {
                !series.is_boss_pub()
            }
            RunPhase::BossReward | RunPhase::Won | RunPhase::Lost => false,
        };
        let upcoming_boss = self
            .section_boss
            .as_ref()
            .filter(|_| boss_ahead)
            .map(|id| self.boss_info(content, id));
        let relics = self
            .relics
            .iter()
            .enumerate()
            .map(|(index, id)| {
                let relic = Run::relic_info(content, id);
                OwnedRelic {
                    index,
                    sell_price: sell_price(relic.price),
                    relic,
                    can_sell: self.accepts(content, RunAction::SellRelic(index)),
                }
            })
            .collect();
        let consumables = self
            .consumables
            .iter()
            .enumerate()
            .map(|(slot, id)| self.owned_consumable(content, slot, id))
            .collect();
        let character = content.character(self.character);
        RunView {
            seed: self.seed.to_string(),
            tutorial: self.tutorial,
            character: CharacterInfo {
                character: self.character,
                name: character.map_or_else(|| self.character.to_string(), |c| c.name.clone()),
                text: character.map_or_else(String::new, |c| c.text.clone()),
                back: character
                    .map_or(crate::content::PLAYER_BACK, |c| c.back())
                    .to_owned(),
            },
            phase: self.phase,
            ai_to_move: self.ai_to_move(),
            series: SeriesView {
                section: series.section,
                index: series.index,
                is_boss: series.is_boss_pub(),
                target: series.target,
                score: series.score,
                games_played: series.games_played,
                game_limit: series.game_limit,
                games_left: series.games_left(),
                redeals_left: self.redeals_left(content),
                redeals_used: series.redeals_used,
                boss,
                guests: self.guest_infos(content),
            },
            upcoming_boss,
            next_series: self.next_series_view(content),
            deal_odds: crate::deck::deal_odds(),
            sections: self.sections(content),
            length: self.length,
            tour: self.tour_view(content),
            stake: self.stake,
            win_threshold: crate::rules::WIN_THRESHOLD.saturating_add(self.stake),
            coins: self.coins,
            relics,
            relic_slots: content.relic_slots(self.character),
            consumables,
            consumable_slots: content.run.consumable_slots,
            flags: self.flags.iter().cloned().collect(),
            counters: self.counters.clone(),
            game: self.game_view(content),
            shop: self.shop_view(content),
            encounter: self.encounter_view(content),
            boss_reward: self.boss_reward_view(content),
            card_reward: self.card_reward().map(|reward| CardRewardView {
                offers: reward
                    .offers
                    .iter()
                    .enumerate()
                    .filter_map(|(index, offer)| {
                        let card = self.edition.get(offer.card)?.clone();
                        let (result, replaces, raises) =
                            self.mod_result(content, offer.card, &offer.card_mod, offer.value);
                        Some(CardRewardOfferView {
                            index,
                            result: result.unwrap_or_else(|| card.clone()),
                            card,
                            card_mod: Run::card_mod_info(
                                content,
                                &offer.card_mod,
                                Some(offer.value),
                            ),
                            can_take: self.card_reward_open(content, index),
                            replaces,
                            raises,
                        })
                    })
                    .collect(),
                skip_coins: content.run.card_reward.as_ref().map_or(0, |c| c.skip_coins),
            }),
            schnapps_reward: self.schnapps_reward().map(|id| {
                let consumable = Run::consumable_info(content, id);
                SchnappsRewardView {
                    sell_price: sell_price(consumable.price),
                    consumable,
                }
            }),
            painter: self.painter_view(content),
            evening_relics: self
                .evening_relics
                .iter()
                .map(|id| Run::relic_info(content, id))
                .collect(),
            pending_factor: self.pending_factor(),
            pending_next_game: self
                .pending_next_game
                .iter()
                .map(|(consumable, effect)| PendingNextGame {
                    consumable: consumable.clone(),
                    effect: *effect,
                })
                .collect(),
            next_pub: self.next_pub,
            debt: self.debt,
            credit_left: self.credit_left(content),
            wager: self.wager.clone(),
            scout_legs: self.scout_legs,
            last_score: self.last_score.as_ref().map(|b| ScoreView {
                breakdown: b.clone(),
                steps: b.steps(),
            }),
            can_give_up: self.accepts(content, RunAction::GiveUpPub),
            needed: self.needed_view(content),
            edition: self.edition.cards().to_vec(),
            stats: self.stats,
            night: self.night_view(content),
            weekly: self.weekly_view(content),
            can_enter_night: self.phase == RunPhase::Won && self.night.is_none(),
        }
    }

    /// The open relic pick of a won boss pub.
    fn boss_reward_view(&self, content: &Content) -> Option<BossRewardView> {
        let reward = self.boss_reward.as_ref()?;
        Some(BossRewardView {
            offers: reward
                .relics
                .iter()
                .enumerate()
                .map(|(index, id)| BossRelicOffer {
                    index,
                    relic: Run::relic_info(content, id),
                })
                .collect(),
            slots_full: !content.relic_slot_free(self.character, self.relics.len()),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::super::bot::RunBot;
    use super::game::display_key;
    use super::*;
    use crate::content::Character;
    use crate::rules::{Declaration, GameType, NullRisk, RiskLevel};

    fn content() -> Content {
        Content::load().expect("embedded content is valid")
    }

    #[test]
    fn fresh_run_offers_every_declaration_and_a_sorted_hand() {
        let content = content();
        let (run, _) = Run::new(&content, 3).expect("start");
        let view = run.view(&content);
        assert_eq!(view.phase, RunPhase::Game);
        assert_eq!(view.seed, "3");
        assert_eq!(view.series.target, run.series().target);
        let game = view.game.expect("game");
        assert_eq!(game.phase, TablePhase::Declaring);
        assert_eq!(game.hand.len(), 10);
        assert_eq!(game.hand_sizes, vec![10, 10, 10]);
        assert!(!game.your_turn);
        assert!(game.hand.iter().all(|c| !c.legal && !c.from_skat));
        let keys: Vec<_> = game
            .hand
            .iter()
            .map(|c| display_key(GameType::Grand, &c.card))
            .collect();
        let mut sorted = keys.clone();
        sorted.sort();
        assert_eq!(keys, sorted, "hand in display order");
        let options = game.declare.expect("options");
        // Every Spielart for every character (ADR 0012): Null last. The
        // Alleinspieler counts Null like the Skatordnung, without style
        // bonus.
        assert_eq!(options.games.len(), 6);
        assert!(options.games.iter().all(|g| g.allowed));
        assert!(options.risks.iter().all(|r| r.allowed));
        assert_eq!(options.games[0].base_value, 12);
        // Grand 18 in the run (04.11.2026, Cockpit q27).
        assert_eq!(options.games[4].base_value, 18);
        assert_eq!(options.games[5].game, GameType::Null);
        assert_eq!(options.games[5].base_value, 23);
        let null_risks: Vec<(NullRisk, u64)> = options
            .null_risks
            .iter()
            .map(|r| (r.risk, r.value))
            .collect();
        assert_eq!(
            null_risks,
            vec![(NullRisk::Hand, 46), (NullRisk::OuvertHand, 92)]
        );
        assert!(options.null_style);
        assert!(options.can_redeal);
        assert_eq!(
            options.redeals_left,
            content.redeals_in(Character::Declarer, 0)
        );
        assert!(options.can_pick_up_skat);
        assert!(!options.hand_required);
        // Before the skat only the hand levels (ADR 0011).
        assert!(options.risks.iter().all(|r| r.risk != RiskLevel::Normal));
        assert_eq!(options.risks.len(), 4);
    }

    #[test]
    fn view_agrees_with_the_run_through_a_whole_run() {
        let content = content();
        let (mut run, _) = Run::new(&content, 21).expect("start");
        let mut saw_discard = false;
        let mut saw_declare = [false; 2];
        let mut saw_play = false;
        let mut saw_shop = false;
        let mut saw_score = false;
        for _ in 0..2_000 {
            let view = run.view(&content);
            if let Some(game) = &view.game {
                let legal: Vec<u32> = game
                    .hand
                    .iter()
                    .filter(|c| c.legal)
                    .map(|c| c.card.id)
                    .collect();
                if game.phase == TablePhase::Discarding {
                    saw_discard = true;
                    // Ten and the skat; with "Drei drücken" (a boss relic
                    // may bring it, ADR 0017) one card more.
                    assert_eq!(game.hand.len(), 10 + game.discard_count);
                    assert!(game.hand.iter().filter(|c| c.from_skat).count() >= 2);
                }
                if let Some(options) = &game.declare {
                    saw_declare[usize::from(game.phase == TablePhase::DeclaringAfterSkat)] = true;
                    assert_eq!(
                        options.can_pick_up_skat,
                        run.accepts(&content, RunAction::PickUpSkat)
                    );
                    for risk in &options.risks {
                        let grand = Declaration::grand(risk.risk);
                        assert_eq!(
                            risk.allowed,
                            run.accepts(&content, RunAction::Declare(grand))
                        );
                    }
                    for risk in &options.null_risks {
                        let null = Declaration::null(risk.risk);
                        assert_eq!(
                            risk.allowed,
                            run.accepts(&content, RunAction::Declare(null))
                        );
                    }
                }
                let previewed = game.phase == TablePhase::Playing
                    && game.declaration.is_some_and(|d| !d.is_ramsch());
                assert_eq!(game.value_preview.is_some(), previewed);
                if let Some(p) = game.value_preview {
                    assert_eq!(p.game_value, p.multiplier.scale(p.base_value * p.levels));
                }
                if game.your_turn {
                    saw_play = true;
                    assert!(!legal.is_empty());
                    for card in &game.hand {
                        assert_eq!(
                            card.legal,
                            run.accepts(&content, RunAction::Play(card.card.id)),
                            "legal flag matches the run"
                        );
                    }
                } else {
                    assert!(legal.is_empty());
                }
            }
            if let Some(shop) = &view.shop {
                saw_shop = true;
                for offer in shop.relics.iter().flatten() {
                    // The price after discounts ("Der Wirt").
                    let affordable = offer.price <= view.coins;
                    let free = view
                        .relic_slots
                        .is_none_or(|slots| view.relics.len() < slots as usize);
                    assert_eq!(offer.can_buy, affordable && free);
                }
            }
            if let Some(score) = &view.last_score {
                saw_score = true;
                let last = score.steps.last().expect("steps");
                assert_eq!(last.base_value, score.breakdown.base_value);
                assert_eq!(last.levels, score.breakdown.levels);
                assert_eq!(last.multiplier, score.breakdown.multiplier);
            }
            let Some(action) = RunBot.next_action(&run, &content) else {
                break;
            };
            run.apply(&content, action).expect("bot action");
        }
        assert!(saw_discard && saw_play && saw_score, "covered the table");
        assert_eq!(saw_declare, [true, true], "both Ansage dialogs");
        assert!(saw_shop || run.phase() == RunPhase::Lost);
        let stats = run.stats();
        assert!(stats.games_won + stats.games_lost + stats.redeals > 0);
    }

    #[test]
    fn consumables_report_their_targets() {
        // Frisches Deck is a legacy item since the Zinken (DESIGN 8.6).
        let content = crate::run::legacy_content();
        let (mut run, _) = Run::new(&content, 5).expect("start");
        run.consumables = vec![ConsumableId::new("fresh_deck"), ConsumableId::new("herbal")];
        let view = run.view(&content);
        // In a game: Kräuter works, the fresh deck only at the Tresen.
        assert!(!view.consumables[0].usable_now);
        assert!(view.consumables[0].consumable.needs_target);
        assert!(view.consumables[1].usable_now);
        assert!(!view.consumables[1].can_sell, "selling only at the Tresen");
        run.game = None;
        run.phase = RunPhase::Shop;
        run.shop = Some(run.draw_offer(&content, 0));
        // Frisches Deck cleanses a card that carries a modification.
        let view = run.view(&content);
        assert!(!view.consumables[0].usable_now);
        run.edition
            .add_mod(&content, 7, &CardModId::new("points"), 3)
            .expect("Augen");
        let view = run.view(&content);
        assert!(view.consumables[0].usable_now);
        assert_eq!(view.consumables[0].targets, vec![7]);
        assert!(!view.consumables[1].usable_now);
        assert!(view.consumables[1].can_sell);
        assert_eq!(view.consumables[1].sell_price, 1);
        assert!(view.game.is_none());
        assert!(view.shop.is_some());
    }
}

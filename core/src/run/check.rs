//! Checking an action without carrying it out ([`Run::check`]).
//!
//! [`Run::dispatch`] carries an action out on a copy of the run; a
//! failing action leaves the copy behind. To ask whether an action would
//! be accepted, [`Run::check`] first runs [`Run::precheck`]: the checks
//! each action makes before it changes anything, without copying the run.
//! Only an action that passes them is tried on a copy, unless its checks
//! are complete ([`Precheck::Accepted`]: selling, the Reroll, the Tresen
//! services, a consumable on a card). So `check` gives exactly the result
//! of trying the action, much cheaper when it fails, which is what the
//! view asks most (every card for a target, every declaration while the
//! cards are played).
//!
//! [`Run::precheck`] mirrors the first checks of each action in the order
//! the action makes them; where an action keeps them in a function of its
//! own, the precheck calls the same function. Tests play whole runs and
//! compare it with trying every candidate action, and the view built with
//! and without it.

use super::{Run, RunAction, RunError, RunPhase};
use crate::content::{CardModDef, ConsumableAction, Content};
use crate::deck::{CardKind, DeckError};

/// What [`Run::precheck`] found when no check failed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Precheck {
    /// The action may still fail later: try it.
    Try,
    /// Every check passed and nothing after them fails.
    Accepted,
}

impl Run {
    /// Whether the run would accept `action` now ([`Run::check`]).
    #[must_use]
    pub fn accepts(&self, content: &Content, action: RunAction) -> bool {
        self.check(content, action).is_ok()
    }

    /// Whether `action` would be accepted now, and if not, why: the error
    /// [`Run::apply_step`] would return, without changing the run. Cheap
    /// when [`Run::precheck`] already rules the action out; otherwise the
    /// action is tried on a copy.
    pub fn check(&self, content: &Content, action: RunAction) -> Result<(), RunError> {
        #[cfg(test)]
        let skip = tests::TRIAL_ONLY.with(std::cell::Cell::get);
        #[cfg(not(test))]
        let skip = false;
        if !skip && self.precheck(content, &action)? == Precheck::Accepted {
            return Ok(());
        }
        let mut probe = self.clone();
        probe.dispatch(content, action, &mut Vec::new())
    }

    /// The checks `action` makes before it changes anything, in the same
    /// order: an error here is the error of trying it. For a few actions
    /// whose checks are complete (selling, the Reroll, a Tresen service, a
    /// consumable on a card) [`Precheck::Accepted`] says it succeeds.
    pub(super) fn precheck(
        &self,
        content: &Content,
        action: &RunAction,
    ) -> Result<Precheck, RunError> {
        self.precheck_prefix(content, action)?;
        Ok(match action {
            RunAction::HirePainter(card) => {
                self.check_paint(content, *card)?;
                // He draws at least one choice from what fits.
                if content.run.shop.zinken.painter_choices > 0 {
                    Precheck::Accepted
                } else {
                    Precheck::Try
                }
            }
            RunAction::CleanseCard(card) => {
                self.check_cleanse(content, *card)?;
                Precheck::Accepted
            }
            // Past their first checks these cannot fail.
            RunAction::SellRelic(_) | RunAction::SellConsumable(_) | RunAction::Reroll => {
                Precheck::Accepted
            }
            RunAction::UseConsumable {
                slot,
                target: Some(card),
                ..
            } => self.complete_consumable(content, *slot, *card)?,
            _ => Precheck::Try,
        })
    }

    /// The rest of the checks of a consumable used on deck card `card`
    /// in the shop, after [`Run::precheck_consumable`]: complete for
    /// cleansing and attaching a modification.
    fn complete_consumable(
        &self,
        content: &Content,
        slot: usize,
        card: u32,
    ) -> Result<Precheck, RunError> {
        let (_, def) = self.consumable_at(content, slot)?;
        match &def.action {
            ConsumableAction::CleanseCard => self.check_cleanse(content, card)?,
            ConsumableAction::ApplyCardMod { card_mod, value } => {
                self.check_attach(content, card, card_mod, *value)?;
            }
            ConsumableAction::AddCardMod(card_mod) => {
                let deck_card = self.edition.get(card).ok_or(DeckError::UnknownCard(card))?;
                if deck_card.kind == CardKind::Blank {
                    return Err(DeckError::BlankCannotBeModified.into());
                }
                match card_mod {
                    Some(m) => {
                        let value = content.card_mod(m).map_or(1, CardModDef::default_value);
                        self.check_attach(content, card, m, value)?;
                    }
                    // A drawn modification fits: the pool holds only those.
                    None => {
                        if self.card_mod_pool_for(content, deck_card).is_empty() {
                            return Err(RunError::NoModAvailable);
                        }
                    }
                }
            }
            _ => return Ok(Precheck::Try),
        }
        Ok(Precheck::Accepted)
    }

    /// The first checks of `action`, see [`Run::precheck`].
    fn precheck_prefix(&self, content: &Content, action: &RunAction) -> Result<(), RunError> {
        self.check_pending(action)?;
        match action {
            RunAction::Redeal => self.declaring_ref().map(drop),
            RunAction::PickUpSkat => {
                if self.declaring_ref()?.rules.require_hand {
                    return Err(RunError::HandRequired);
                }
                Ok(())
            }
            RunAction::Declare(declaration) => self.check_declare(*declaration).map(drop),
            RunAction::AnswerKontra { .. } => {
                if !self.active_ref()?.awaiting_re {
                    return Err(RunError::NoKontra);
                }
                Ok(())
            }
            RunAction::Discard(_) | RunAction::DiscardThree(_) | RunAction::Play(_) => {
                if self.active_ref()?.awaiting_re {
                    return Err(RunError::AwaitingKontraAnswer);
                }
                Ok(())
            }
            RunAction::TakeFromSkat { .. } => {
                if self.active_ref()?.awaiting_re {
                    return Err(RunError::AwaitingKontraAnswer);
                }
                if self.open_skat_swaps().is_empty() {
                    return Err(RunError::NoSkatSwapLeft);
                }
                Ok(())
            }
            RunAction::ChooseRiseCards(_) => {
                self.rise_pending().ok_or(RunError::InvalidRiseCards)?;
                self.active_ref().map(drop)
            }
            RunAction::UseConsumable { slot, target, suit } => {
                self.precheck_consumable(content, *slot, *target, *suit)
            }
            RunAction::ChooseMalus { option, .. } => {
                if self.phase != RunPhase::Route {
                    return Err(RunError::WrongPhase);
                }
                self.route
                    .as_ref()
                    .and_then(|r| r.options.get(*option))
                    .and_then(|o| o.spelunke.as_ref())
                    .ok_or(RunError::InvalidSlot)
                    .map(drop)
            }
            RunAction::BuyRelic(slot) => {
                let shop = self.shop_ref()?;
                shop.relics
                    .get(*slot)
                    .and_then(Option::as_ref)
                    .ok_or(RunError::InvalidSlot)
                    .map(drop)
            }
            RunAction::ReplaceRelic { slot, replace } => {
                let shop = self.shop_ref()?;
                shop.relics
                    .get(*slot)
                    .and_then(Option::as_ref)
                    .ok_or(RunError::InvalidSlot)?;
                // Then the sale of the relic it replaces.
                self.tresen_ref()?;
                self.relics
                    .get(*replace)
                    .ok_or(RunError::InvalidSlot)
                    .map(drop)
            }
            RunAction::BuyConsumable(slot) => {
                let shop = self.shop_ref()?;
                shop.consumables
                    .get(*slot)
                    .and_then(Option::as_ref)
                    .ok_or(RunError::InvalidSlot)
                    .map(drop)
            }
            RunAction::SellRelic(index) => {
                self.tresen_ref()?;
                self.relics
                    .get(*index)
                    .ok_or(RunError::InvalidSlot)
                    .map(drop)
            }
            RunAction::SellConsumable(index) => {
                self.shop_ref()?;
                self.consumables
                    .get(*index)
                    .ok_or(RunError::InvalidSlot)
                    .map(drop)
            }
            RunAction::Reroll => {
                if self.tresen_ref()?.rerolls > 0 {
                    return Err(RunError::AlreadyUsed);
                }
                if !self.can_pay(content, self.reroll_cost(content)) {
                    return Err(RunError::NotEnoughCoins);
                }
                Ok(())
            }
            RunAction::BuyCardOffer(slot) => {
                let shop = self.shop_ref()?;
                let offer = shop
                    .card_offers
                    .get(*slot)
                    .and_then(Option::as_ref)
                    .ok_or(RunError::InvalidSlot)?;
                if !self.can_pay(content, self.card_offer_cost(content, offer)) {
                    return Err(RunError::NotEnoughCoins);
                }
                Ok(())
            }
            RunAction::HirePainter(card) => {
                if self.tresen_ref()?.painter_used {
                    return Err(RunError::AlreadyUsed);
                }
                if !self.can_pay(content, self.painter_cost(content, *card)) {
                    return Err(RunError::NotEnoughCoins);
                }
                Ok(())
            }
            RunAction::ChoosePaint(option) => {
                let painter = self.painter.as_ref().ok_or(RunError::WrongPhase)?;
                painter
                    .options
                    .get(*option)
                    .ok_or(RunError::InvalidSlot)
                    .map(drop)
            }
            RunAction::CleanseCard(_) => {
                if self.tresen_ref()?.cleansed {
                    return Err(RunError::AlreadyUsed);
                }
                let price = self.shop_price(content, content.run.shop.zinken.cleanse_price);
                if !self.can_pay(content, price) {
                    return Err(RunError::NotEnoughCoins);
                }
                Ok(())
            }
            RunAction::LeaveShop => self.shop_ref().map(drop),
            RunAction::GiveUpPub => {
                if !self.can_give_up() {
                    return Err(RunError::WrongPhase);
                }
                Ok(())
            }
            RunAction::ChooseEncounter {
                option,
                target,
                marke,
            } => self
                .encounter_choice(content, *option, *target, marke.as_ref())
                .map(drop),
            RunAction::ChooseRoute(option) => self.route_choice(*option).map(drop),
            RunAction::ChooseBossRelic { offer, replace } => {
                self.precheck_boss_relic(content, *offer, *replace)
            }
            RunAction::ChooseSchnappsReward { give_up } => {
                self.schnapps_reward.as_ref().ok_or(RunError::WrongPhase)?;
                match give_up {
                    Some(slot) => self
                        .consumables
                        .get(*slot)
                        .ok_or(RunError::InvalidSlot)
                        .map(drop),
                    None => Ok(()),
                }
            }
            RunAction::TakeCardReward { offer } => {
                if self.phase != RunPhase::CardReward {
                    return Err(RunError::WrongPhase);
                }
                let reward = self.card_reward.as_ref().ok_or(RunError::WrongPhase)?;
                reward
                    .offers
                    .get(*offer)
                    .ok_or(RunError::InvalidSlot)
                    .map(drop)
            }
            RunAction::SkipCardReward => {
                if self.phase != RunPhase::CardReward || self.card_reward.is_none() {
                    return Err(RunError::WrongPhase);
                }
                Ok(())
            }
            RunAction::EnterNight => {
                if self.phase != RunPhase::Won || self.night.is_some() {
                    return Err(RunError::WrongPhase);
                }
                Ok(())
            }
            RunAction::SettleGame => Ok(()),
        }
    }

    /// [`Run::precheck`] for [`RunAction::UseConsumable`]: the checks of
    /// [`Run::use_consumable`] before it changes anything.
    fn precheck_consumable(
        &self,
        content: &Content,
        slot: usize,
        target: Option<u32>,
        suit: Option<crate::cards::Suit>,
    ) -> Result<(), RunError> {
        let (_, def) = self.consumable_at(content, slot)?;
        match &def.action {
            ConsumableAction::CleanseCard
            | ConsumableAction::AddCardMod(_)
            | ConsumableAction::ApplyCardMod { .. } => {
                if self.phase != RunPhase::Shop {
                    return Err(RunError::WrongPhase);
                }
                target.ok_or(RunError::TargetRequired).map(drop)
            }
            ConsumableAction::ExtraRedeals(_) => {
                if self.phase != RunPhase::Game {
                    return Err(RunError::WrongPhase);
                }
                Ok(())
            }
            ConsumableAction::PeekSkat => {
                if self.declaring_ref()?.skat_peeked {
                    return Err(RunError::WrongPhase);
                }
                Ok(())
            }
            ConsumableAction::ChalkCard => {
                target.ok_or(RunError::TargetRequired)?;
                suit.ok_or(RunError::TargetRequired)?;
                self.active_ref().map(drop)
            }
            ConsumableAction::RerollRoutes => {
                if self.phase != RunPhase::Route {
                    return Err(RunError::WrongPhase);
                }
                Ok(())
            }
            ConsumableAction::NextPub { .. } => {
                if !matches!(
                    self.phase,
                    RunPhase::Shop | RunPhase::Encounter | RunPhase::Route
                ) {
                    return Err(RunError::WrongPhase);
                }
                Ok(())
            }
            ConsumableAction::ClearAccount => {
                if !self.between_games() {
                    return Err(RunError::WrongPhase);
                }
                if self.series.score >= 0 {
                    return Err(RunError::ConditionNotMet);
                }
                Ok(())
            }
            ConsumableAction::ExtraGames(_) => {
                if !self.between_games() {
                    return Err(RunError::WrongPhase);
                }
                if self.series.is_boss_pub() {
                    return Err(RunError::NotInBossPub);
                }
                Ok(())
            }
            ConsumableAction::NextGameIfBehind(_) => {
                self.declaring_ref()?;
                if !self.behind() {
                    return Err(RunError::ConditionNotMet);
                }
                Ok(())
            }
            ConsumableAction::NextGameCursed(_) => self.declaring_ref().map(drop),
            ConsumableAction::Redeal
            | ConsumableAction::UndoRedeal
            | ConsumableAction::NextGame(_)
            | ConsumableAction::RedealNextGame(_) => Ok(()),
        }
    }

    /// [`Run::precheck`] for [`RunAction::ChooseBossRelic`]: the checks
    /// of [`Run::choose_boss_relic`] before it changes anything.
    fn precheck_boss_relic(
        &self,
        content: &Content,
        offer: Option<usize>,
        replace: Option<usize>,
    ) -> Result<(), RunError> {
        if self.phase != RunPhase::BossReward {
            return Err(RunError::WrongPhase);
        }
        let reward = self.boss_reward.as_ref().ok_or(RunError::WrongPhase)?;
        match offer {
            None if replace.is_some() => Err(RunError::InvalidSlot),
            None => Ok(()),
            Some(i) => {
                reward.relics.get(i).ok_or(RunError::InvalidSlot)?;
                match replace {
                    Some(index) => self
                        .relics
                        .get(index)
                        .ok_or(RunError::InvalidSlot)
                        .map(drop),
                    None if !content.relic_slot_free(self.character, self.relics.len()) => {
                        Err(RunError::NoFreeSlot)
                    }
                    None => Ok(()),
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::bot::RunBot;
    use super::super::content_tests::content;
    use super::*;
    use crate::cards::Suit;
    use crate::game::Seat;
    use crate::rules::{Declaration, RiskLevel};

    std::thread_local! {
        /// Tests: [`Run::check`] without the precheck, only by trying.
        pub(super) static TRIAL_ONLY: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
    }

    /// Every kind of action with a few arguments around what the run
    /// holds now.
    fn candidates(run: &Run, content: &Content) -> Vec<RunAction> {
        let hand: Vec<u32> = run
            .game()
            .map(|a| a.game.hand(Seat::DECLARER).iter().map(|c| c.id).collect())
            .unwrap_or_default();
        let first = |n: usize| hand.get(n).copied().unwrap_or(99);
        let cards: Vec<Option<u32>> = std::iter::once(None)
            .chain((0..32).step_by(3).map(Some))
            .chain(hand.iter().copied().map(Some))
            .collect();
        let marke = run.marken(content).first().map(|m| m.id.clone());
        let mut out = vec![
            RunAction::Redeal,
            RunAction::PickUpSkat,
            RunAction::Declare(Declaration::grand(RiskLevel::Normal)),
            RunAction::Declare(Declaration::grand(RiskLevel::Hand)),
            RunAction::Declare(Declaration::null(crate::rules::NullRisk::Normal)),
            RunAction::Declare(Declaration::null(crate::rules::NullRisk::Hand)),
            RunAction::Declare(Declaration::suit(Suit::Hearts, RiskLevel::Normal)),
            RunAction::Declare(Declaration::Ramsch),
            RunAction::AnswerKontra { re: true },
            RunAction::AnswerKontra { re: false },
            RunAction::Discard([first(0), first(1)]),
            RunAction::DiscardThree([first(0), first(1), first(2)]),
            RunAction::Play(99),
            RunAction::ChooseRiseCards([first(0), first(1), first(2)]),
            RunAction::Reroll,
            RunAction::LeaveShop,
            RunAction::GiveUpPub,
            RunAction::TakeFromSkat {
                give: first(0),
                take: None,
            },
            RunAction::SettleGame,
            RunAction::SkipCardReward,
            RunAction::EnterNight,
            RunAction::ChooseSchnappsReward { give_up: None },
            RunAction::ChooseSchnappsReward { give_up: Some(0) },
            RunAction::ChooseSchnappsReward { give_up: Some(9) },
        ];
        out.extend(hand.iter().map(|&id| RunAction::Play(id)));
        for slot in 0..=run.consumables().len() {
            for &target in &cards {
                for suit in [None, Some(Suit::Clubs)] {
                    out.push(RunAction::UseConsumable { slot, target, suit });
                }
            }
        }
        for i in 0..4 {
            out.extend([
                RunAction::ChooseMalus {
                    option: i,
                    malus: 0,
                },
                RunAction::BuyRelic(i),
                RunAction::BuyConsumable(i),
                RunAction::BuyCardOffer(i),
                RunAction::ChoosePaint(i),
                RunAction::ChooseRoute(i),
                RunAction::TakeCardReward { offer: i },
                RunAction::ChooseBossRelic {
                    offer: Some(i),
                    replace: None,
                },
                RunAction::ChooseBossRelic {
                    offer: Some(i),
                    replace: Some(0),
                },
            ]);
            for replace in 0..3 {
                out.push(RunAction::ReplaceRelic { slot: i, replace });
            }
            for target in cards.iter().copied().chain([Some(0), Some(1), Some(2)]) {
                for marke in [None, marke.clone()] {
                    out.push(RunAction::ChooseEncounter {
                        option: i,
                        target,
                        marke,
                    });
                }
            }
        }
        out.push(RunAction::ChooseBossRelic {
            offer: None,
            replace: None,
        });
        out.push(RunAction::ChooseBossRelic {
            offer: None,
            replace: Some(0),
        });
        for index in 0..=run.relics().len() {
            out.push(RunAction::SellRelic(index));
        }
        for index in 0..=run.consumables().len() {
            out.push(RunAction::SellConsumable(index));
        }
        for card in [0, 7, 15, 31, 99] {
            out.push(RunAction::HirePainter(card));
            out.push(RunAction::CleanseCard(card));
        }
        out
    }

    #[test]
    fn precheck_errors_are_the_errors_of_trying_through_whole_runs() {
        let content = content();
        let mut checked = 0usize;
        let mut ruled_out = 0usize;
        let mut accepted = 0usize;
        let mut phases = std::collections::BTreeSet::new();
        for seed in [3, 21] {
            let (mut run, _) = Run::new(&content, seed).expect("start");
            for step in 0..3_000 {
                phases.insert(format!("{:?}", run.phase()));
                // Every step away from the table, every fifth at it.
                if run.phase() == RunPhase::Game && step % 5 != 0 {
                    let Some(action) = RunBot.next_action(&run, &content) else {
                        break;
                    };
                    run.apply(&content, action).expect("bot action");
                    continue;
                }
                for action in candidates(&run, &content) {
                    let mut probe = run.clone();
                    let tried = probe.dispatch(&content, action.clone(), &mut Vec::new());
                    match run.precheck(&content, &action) {
                        Err(e) => {
                            assert_eq!(tried, Err(e), "{action:?}");
                            ruled_out += 1;
                        }
                        Ok(Precheck::Accepted) => {
                            assert_eq!(tried, Ok(()), "{action:?}");
                            accepted += 1;
                        }
                        Ok(Precheck::Try) => {}
                    }
                    assert_eq!(run.check(&content, action.clone()), tried, "{action:?}");
                    checked += 1;
                }
                let Some(action) = RunBot.next_action(&run, &content) else {
                    break;
                };
                run.apply(&content, action).expect("bot action");
            }
        }
        // Most candidates fail; the precheck catches nearly all of them.
        assert!(ruled_out * 10 > checked * 8, "{ruled_out} of {checked}");
        assert!(accepted > 0, "complete checks were met");
        for phase in ["Game", "Shop", "Encounter", "Route", "BossReward"] {
            assert!(phases.contains(phase), "{phase} in {phases:?}");
        }
    }

    #[test]
    fn the_view_is_the_same_with_and_without_the_precheck() {
        let content = content();
        for seed in [5, 21] {
            let (mut run, _) = Run::new(&content, seed).expect("start");
            for _ in 0..3_000 {
                let fast = serde_json::to_string(&run.view(&content)).expect("view");
                TRIAL_ONLY.with(|t| t.set(true));
                let tried = serde_json::to_string(&run.view(&content)).expect("view");
                TRIAL_ONLY.with(|t| t.set(false));
                assert_eq!(fast, tried);
                let Some(action) = RunBot.next_action(&run, &content) else {
                    break;
                };
                run.apply(&content, action).expect("bot action");
            }
        }
    }

    /// Measurement, not a check: the time of building the view through a
    /// whole run with and without the precheck
    /// (`cargo test -p skat_core --lib view_cost -- --ignored --nocapture`).
    #[test]
    #[ignore = "measurement"]
    fn view_cost() {
        let content = content();
        let mut states = Vec::new();
        for seed in [5, 21] {
            let (mut run, _) = Run::new(&content, seed).expect("start");
            for _ in 0..3_000 {
                states.push(run.clone());
                let Some(action) = RunBot.next_action(&run, &content) else {
                    break;
                };
                run.apply(&content, action).expect("bot action");
            }
        }
        let mut times = [std::time::Duration::ZERO; 2];
        for (i, trial) in [false, true].into_iter().enumerate() {
            TRIAL_ONLY.with(|t| t.set(trial));
            let start = std::time::Instant::now();
            for _ in 0..10 {
                for run in &states {
                    let _ = run.view(&content);
                }
            }
            times[i] = start.elapsed();
        }
        TRIAL_ONLY.with(|t| t.set(false));
        let views = f64::from(u32::try_from(states.len() * 10).unwrap_or(1));
        println!(
            "{} states: with precheck {:.1} us, by trying only {:.1} us per view",
            states.len(),
            times[0].as_secs_f64() * 1e6 / views,
            times[1].as_secs_f64() * 1e6 / views
        );
    }
}

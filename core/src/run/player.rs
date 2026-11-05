//! What the player may know: the run events without hidden cards.
//!
//! [`RunEvent`] carries the full game events of the core, including every
//! hand at the deal and the skat. The run and the simulator need them; a
//! frontend must not get them, or the player could read the defenders'
//! hands and the untouched skat out of the browser. [`RunEvent::for_player`]
//! turns an event into a [`PlayerEvent`] that holds only what the player
//! sees at the table: his own cards, the sizes of the other hands and every
//! card once it is played. The talon order never leaves the core.
//!
//! Everything else in the run events is public table knowledge and passes
//! unchanged.

use serde::{Deserialize, Serialize};

use super::RunEvent;
use crate::deck::DealtCard;
use crate::game::{CompletedTrick, Event, Seat};
use crate::rules::{Declaration, GameValue, LeadSuit, Matadors, Outcome, SeatTotals};

/// A run event as the player may see it.
pub type PlayerEvent = RunEvent<PlayerGameEvent>;

/// A game event as the player may see it. Variant names match
/// [`Event`], so a frontend can read both the same way. Every variant is
/// an object in JSON.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlayerGameEvent {
    /// Cards were dealt: the player's own ten cards, the others only as
    /// counts.
    Dealt {
        /// Seat that will lead the first trick.
        forehand: Seat,
        /// The player's hand.
        hand: Vec<DealtCard>,
        /// Cards per seat (index = seat).
        hand_sizes: [usize; 3],
    },
    /// The defenders' hands and the skat were dealt again; the player's
    /// hand is unchanged and nothing else is visible.
    SkatRedrawn {
        /// Cards per seat (index = seat).
        hand_sizes: [usize; 3],
    },
    /// The player announced the game.
    Declared {
        /// The declaration.
        declaration: Declaration,
    },
    /// The player picked up the skat and now knows it.
    SkatPickedUp {
        /// The two cards that joined the hand.
        skat: [DealtCard; 2],
    },
    /// The player discarded two cards.
    Discarded {
        /// The discarded cards.
        cards: [DealtCard; 2],
    },
    /// A card was played (open on the table).
    CardPlayed {
        /// Who played.
        seat: Seat,
        /// What was played.
        card: DealtCard,
    },
    /// A trick was completed.
    TrickWon {
        /// The completed trick.
        trick: CompletedTrick<DealtCard>,
        /// Augen in the trick.
        card_points: u32,
    },
    /// The defenders gave up.
    Conceded {
        /// Tricks that were not played.
        tricks: u32,
        /// Augen still in the hands.
        card_points: u32,
    },
    /// The defenders swapped their hands; the player sees only that.
    DefenderHandsSwapped {
        /// Cards per seat (index = seat).
        hand_sizes: [usize; 3],
    },
    /// The player took a card from the untouched skat ("Die Oma"): he now
    /// holds `took` and put `gave` into the skat. The other skat card stays
    /// hidden.
    SkatSwapped {
        /// The card taken into the hand.
        took: DealtCard,
        /// The card put into the skat.
        gave: DealtCard,
    },
    /// With the skat the player drew a card from a defender ("Drei
    /// drücken"); it is in his hand now.
    CardDrawn {
        /// The defender who held it.
        seat: Seat,
        /// The card.
        card: DealtCard,
    },
    /// The lowest of the player's three discards went back to the defender
    /// ("Drei drücken").
    CardReturned {
        /// The defender who gets it.
        seat: Seat,
        /// The card.
        card: DealtCard,
    },
    /// The player named what the next defender who leads must lead
    /// ("Lockvogel").
    LeadDemanded {
        /// What must be led.
        suit: LeadSuit,
    },
    /// The player repainted a card of his hand for this game ("Blue Curaçao").
    CardChalked {
        /// The card as it was.
        old: DealtCard,
        /// The card as it is now.
        new: DealtCard,
    },
    /// A Wandelkarte of the player took its suit as the game started
    /// (DESIGN 7.2); `new` is the card from now on.
    WildCardSettled {
        /// The card as printed.
        old: DealtCard,
        /// The card with its settled suit.
        new: DealtCard,
    },
    /// An "Untot" card of the player rose from the last trick back into his
    /// hand; `substitute` from his hand lies in the trick now.
    CardRose {
        /// Who played the card (the player).
        seat: Seat,
        /// The card as it was played.
        card: DealtCard,
        /// The card in the trick now.
        substitute: DealtCard,
        /// The card as it came back into the hand: a copy of the card that
        /// won the trick (03.11.2026). `None` in logs from before.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        risen: Option<DealtCard>,
    },
    /// "Doppelgänger" (DESIGN 7.2): `seat` played `card`, `stolen` from
    /// the hand of `from` lies in the trick instead, and `from` holds
    /// `given` now. Every card of it is open on the table.
    DoppelgangerSwapped {
        /// Who played the Doppelgänger.
        seat: Seat,
        /// The Doppelgänger as it was played.
        card: DealtCard,
        /// The robbed seat.
        from: Seat,
        /// The card taken from him into the trick.
        stolen: DealtCard,
        /// The Doppelgänger in his hand, in his edition.
        given: DealtCard,
    },
    /// A card changed its holder and shows the new holder's edition (ADR
    /// 0015). The card only when the player knows it: it came to him, or
    /// it was his (the card given back with "Drei drücken"). A swap
    /// between the defenders shows no card.
    CardChangedHands {
        /// The new holder.
        seat: Seat,
        /// The card in the new holder's edition, if the player knows it.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        card: Option<DealtCard>,
    },
    /// The game is scored by the rules.
    GameFinished {
        /// Trick outcome.
        outcome: Outcome,
        /// Spitzen.
        matadors: Matadors,
        /// The game value by the rules.
        value: GameValue,
        /// Augen and tricks per seat.
        #[serde(default)]
        seats: SeatTotals,
    },
}

impl PlayerGameEvent {
    /// The player's view of a game event.
    #[must_use]
    pub fn from_event(event: Event<DealtCard>) -> PlayerGameEvent {
        let sizes = |hands: &[Vec<DealtCard>; 3]| [hands[0].len(), hands[1].len(), hands[2].len()];
        match event {
            Event::Dealt {
                forehand, hands, ..
            } => PlayerGameEvent::Dealt {
                forehand,
                hand_sizes: sizes(&hands),
                hand: hands[Seat::DECLARER.index()].clone(),
            },
            Event::SkatRedrawn { hands, .. } => PlayerGameEvent::SkatRedrawn {
                hand_sizes: sizes(&hands),
            },
            Event::Declared { declaration } => PlayerGameEvent::Declared { declaration },
            Event::SkatPickedUp { skat } => PlayerGameEvent::SkatPickedUp { skat },
            Event::Discarded { cards } => PlayerGameEvent::Discarded { cards },
            Event::CardPlayed { seat, card } => PlayerGameEvent::CardPlayed { seat, card },
            Event::TrickWon { trick, card_points } => {
                PlayerGameEvent::TrickWon { trick, card_points }
            }
            Event::Conceded {
                tricks,
                card_points,
            } => PlayerGameEvent::Conceded {
                tricks,
                card_points,
            },
            Event::DefenderHandsSwapped { hands } => PlayerGameEvent::DefenderHandsSwapped {
                hand_sizes: sizes(&hands),
            },
            Event::SkatSwapped { took, gave } => PlayerGameEvent::SkatSwapped { took, gave },
            Event::CardChalked { old, new } => PlayerGameEvent::CardChalked { old, new },
            Event::WildCardSettled { old, new } => PlayerGameEvent::WildCardSettled { old, new },
            Event::CardDrawn { seat, card } => PlayerGameEvent::CardDrawn { seat, card },
            Event::CardReturned { seat, card } => PlayerGameEvent::CardReturned { seat, card },
            Event::CardChangedHands { seat, old, new } => PlayerGameEvent::CardChangedHands {
                seat,
                card: (seat.is_declarer() || old.owner == crate::deck::Owner::Player)
                    .then_some(new),
            },
            Event::LeadDemanded { suit } => PlayerGameEvent::LeadDemanded { suit },
            Event::CardRose {
                seat,
                card,
                substitute,
                risen,
            } => PlayerGameEvent::CardRose {
                seat,
                card,
                substitute,
                risen,
            },
            Event::DoppelgangerSwapped {
                seat,
                card,
                from,
                stolen,
                given,
            } => PlayerGameEvent::DoppelgangerSwapped {
                seat,
                card,
                from,
                stolen,
                given,
            },
            Event::GameFinished {
                outcome,
                matadors,
                value,
                seats,
            } => PlayerGameEvent::GameFinished {
                outcome,
                matadors,
                value,
                seats,
            },
        }
    }
}

impl RunEvent {
    /// This event as the player may see it: game events lose the hidden
    /// cards, everything else is unchanged.
    #[must_use]
    pub fn for_player(self) -> PlayerEvent {
        match self {
            RunEvent::Game(event) => RunEvent::Game(PlayerGameEvent::from_event(event)),
            RunEvent::DoppelgangerUnmatched { card } => RunEvent::DoppelgangerUnmatched { card },
            RunEvent::RiseChoice {
                card,
                becomes,
                pick,
            } => RunEvent::RiseChoice {
                card,
                becomes,
                pick,
            },
            RunEvent::BossAnnounced { section, boss } => RunEvent::BossAnnounced { section, boss },
            RunEvent::SeriesStarted {
                section,
                index,
                target,
                game_limit,
                redeals,
                boss,
                malus,
            } => RunEvent::SeriesStarted {
                section,
                index,
                target,
                game_limit,
                redeals,
                boss,
                malus,
            },
            RunEvent::GameDealt { number, forehand } => RunEvent::GameDealt { number, forehand },
            RunEvent::Kontra { seat } => RunEvent::Kontra { seat },
            RunEvent::Re => RunEvent::Re,
            RunEvent::FlagSet(flag) => RunEvent::FlagSet(flag),
            RunEvent::FlagCleared(flag) => RunEvent::FlagCleared(flag),
            RunEvent::CounterRaised { counter, value } => {
                RunEvent::CounterRaised { counter, value }
            }
            RunEvent::CounterReset { counter } => RunEvent::CounterReset { counter },
            RunEvent::SkatPointsRaised {
                source,
                card_points,
            } => RunEvent::SkatPointsRaised {
                source,
                card_points,
            },
            RunEvent::BonusPoints {
                source,
                seat,
                card_points,
            } => RunEvent::BonusPoints {
                source,
                seat,
                card_points,
            },
            RunEvent::CoinsGained {
                source,
                amount,
                coins,
            } => RunEvent::CoinsGained {
                source,
                amount,
                coins,
            },
            RunEvent::GameScored {
                breakdown,
                series_score,
                discarded_points,
            } => RunEvent::GameScored {
                breakdown,
                series_score,
                discarded_points,
            },
            RunEvent::SeriesWon {
                section,
                index,
                payout,
                coins,
                boss,
                redeals_used,
                games_left,
            } => RunEvent::SeriesWon {
                section,
                index,
                payout,
                coins,
                boss,
                redeals_used,
                games_left,
            },
            RunEvent::ShopOpened {
                relics,
                consumables,
                fence,
            } => RunEvent::ShopOpened {
                relics,
                consumables,
                fence,
            },
            RunEvent::RelicBought { relic, price } => RunEvent::RelicBought { relic, price },
            RunEvent::ConsumableBought { consumable, price } => {
                RunEvent::ConsumableBought { consumable, price }
            }
            RunEvent::RelicSold { relic, price } => RunEvent::RelicSold { relic, price },
            RunEvent::ConsumableSold { consumable, price } => {
                RunEvent::ConsumableSold { consumable, price }
            }
            RunEvent::ShopRerolled { cost } => RunEvent::ShopRerolled { cost },
            RunEvent::CardOffersDrawn { offers } => RunEvent::CardOffersDrawn { offers },
            RunEvent::CardOfferBought {
                slot,
                card,
                card_mod,
                value,
                price,
            } => RunEvent::CardOfferBought {
                slot,
                card,
                card_mod,
                value,
                price,
            },
            RunEvent::PainterHired {
                card,
                options,
                price,
            } => RunEvent::PainterHired {
                card,
                options,
                price,
            },
            RunEvent::PainterChosen {
                card,
                card_mod,
                value,
            } => RunEvent::PainterChosen {
                card,
                card_mod,
                value,
            },
            RunEvent::CardCleansed { card, price } => RunEvent::CardCleansed { card, price },
            RunEvent::TradeCurse { card, curse, cause } => {
                RunEvent::TradeCurse { card, curse, cause }
            }
            RunEvent::ShopLeft => RunEvent::ShopLeft,
            RunEvent::DebtChanged { debt } => RunEvent::DebtChanged { debt },
            RunEvent::SecondChance {
                relic,
                section,
                index,
            } => RunEvent::SecondChance {
                relic,
                section,
                index,
            },
            RunEvent::CoinsStashed {
                source,
                amount,
                pot,
                coins,
            } => RunEvent::CoinsStashed {
                source,
                amount,
                pot,
                coins,
            },
            RunEvent::StashPaidOut {
                relic,
                amount,
                coins,
            } => RunEvent::StashPaidOut {
                relic,
                amount,
                coins,
            },
            RunEvent::RoutesRerolled => RunEvent::RoutesRerolled,
            RunEvent::PubGivenUp {
                section,
                index,
                score,
            } => RunEvent::PubGivenUp {
                section,
                index,
                score,
            },
            RunEvent::AccountCleared { before, score } => {
                RunEvent::AccountCleared { before, score }
            }
            RunEvent::GamesAdded {
                games,
                game_limit,
                games_left,
            } => RunEvent::GamesAdded {
                games,
                game_limit,
                games_left,
            },
            RunEvent::NextPubPlanned { redeals, target } => {
                RunEvent::NextPubPlanned { redeals, target }
            }
            RunEvent::NextPubApplied { redeals, target } => {
                RunEvent::NextPubApplied { redeals, target }
            }
            RunEvent::WagerPlaced { stake } => RunEvent::WagerPlaced { stake },
            RunEvent::WagerSettled { stake, won, payout } => {
                RunEvent::WagerSettled { stake, won, payout }
            }
            RunEvent::StopsRevealed { legs } => RunEvent::StopsRevealed { legs },
            RunEvent::MalusChosen { option, malus } => RunEvent::MalusChosen { option, malus },
            RunEvent::GambleResolved { curse } => RunEvent::GambleResolved { curse },
            RunEvent::ConsumableGained { consumable } => RunEvent::ConsumableGained { consumable },
            RunEvent::ConsumableUsed { consumable } => RunEvent::ConsumableUsed { consumable },
            RunEvent::DeckChanged(change) => RunEvent::DeckChanged(change),
            RunEvent::RedealsAdded {
                redeals,
                redeals_left,
            } => RunEvent::RedealsAdded {
                redeals,
                redeals_left,
            },
            RunEvent::Redealt => RunEvent::Redealt,
            RunEvent::RedealUndone => RunEvent::RedealUndone,
            RunEvent::VoucherRedeemed {
                card,
                card_mod,
                value,
                via_hand,
            } => RunEvent::VoucherRedeemed {
                card,
                card_mod,
                value,
                via_hand,
            },
            RunEvent::VoucherReturned {
                card,
                card_mod,
                value,
            } => RunEvent::VoucherReturned {
                card,
                card_mod,
                value,
            },
            RunEvent::CardPeeked { card, seat } => RunEvent::CardPeeked { card, seat },
            RunEvent::MarkeGiven {
                card,
                card_mod,
                drawn,
                random_card,
            } => RunEvent::MarkeGiven {
                card,
                card_mod,
                drawn,
                random_card,
            },
            RunEvent::CardMadeTransparent { source, card, seat } => {
                RunEvent::CardMadeTransparent { source, card, seat }
            }
            RunEvent::RamschDealt => RunEvent::RamschDealt,
            RunEvent::EncounterOpened { encounter, options } => {
                RunEvent::EncounterOpened { encounter, options }
            }
            RunEvent::EncounterResolved { encounter, option } => {
                RunEvent::EncounterResolved { encounter, option }
            }
            RunEvent::DiceThrown { stake, won, payout } => {
                RunEvent::DiceThrown { stake, won, payout }
            }
            RunEvent::SkatPeekPlanned => RunEvent::SkatPeekPlanned,
            RunEvent::KioskOpened { consumables } => RunEvent::KioskOpened { consumables },
            RunEvent::EveningRuleAdded(relic) => RunEvent::EveningRuleAdded(relic),
            RunEvent::EveningRulesEnded => RunEvent::EveningRulesEnded,
            RunEvent::RelicSwapped { old, new } => RunEvent::RelicSwapped { old, new },
            RunEvent::NextGameDoubled { factor } => RunEvent::NextGameDoubled { factor },
            RunEvent::SkatPeeked { skat } => RunEvent::SkatPeeked { skat },
            RunEvent::SkatCardShown { source, card } => RunEvent::SkatCardShown { source, card },
            RunEvent::RouteOpened { section, index } => RunEvent::RouteOpened { section, index },
            RunEvent::RouteChosen {
                option,
                transport,
                spelunke,
            } => RunEvent::RouteChosen {
                option,
                transport,
                spelunke,
            },
            RunEvent::PubLost {
                section,
                index,
                boss,
            } => RunEvent::PubLost {
                section,
                index,
                boss,
            },
            RunEvent::RelicGained { relic } => RunEvent::RelicGained { relic },
            RunEvent::RunWon {
                character,
                length,
                stake,
            } => RunEvent::RunWon {
                character,
                length,
                stake,
            },
            RunEvent::RunLost {
                section,
                index,
                character,
                length,
                stake,
                boss,
                night,
            } => RunEvent::RunLost {
                section,
                index,
                character,
                length,
                stake,
                boss,
                night,
            },
            RunEvent::NightStarted { character } => RunEvent::NightStarted { character },
            RunEvent::VerschaerfungDrawn {
                section,
                verschaerfung,
            } => RunEvent::VerschaerfungDrawn {
                section,
                verschaerfung,
            },
            RunEvent::ExtraBossRules { section, bosses } => {
                RunEvent::ExtraBossRules { section, bosses }
            }
            RunEvent::NightCursesLaid { cards } => RunEvent::NightCursesLaid { cards },
            RunEvent::BossCursesLaid { boss, cards } => RunEvent::BossCursesLaid { boss, cards },
            RunEvent::BossCursesLifted { cards } => RunEvent::BossCursesLifted { cards },
            RunEvent::BossRewardOffered { relics } => RunEvent::BossRewardOffered { relics },
            RunEvent::BossRewardDeclined => RunEvent::BossRewardDeclined,
            RunEvent::CardRewardOffered { offers } => RunEvent::CardRewardOffered { offers },
            RunEvent::CardRewardTaken {
                offer,
                card,
                card_mod,
                value,
            } => RunEvent::CardRewardTaken {
                offer,
                card,
                card_mod,
                value,
            },
            RunEvent::CardRewardSkipped { coins } => RunEvent::CardRewardSkipped { coins },
            RunEvent::SchnappsRewarded {
                consumable,
                slots_full,
            } => RunEvent::SchnappsRewarded {
                consumable,
                slots_full,
            },
            RunEvent::SchnappsRewardTaken {
                consumable,
                given_up,
            } => RunEvent::SchnappsRewardTaken {
                consumable,
                given_up,
            },
        }
    }
}

/// A list of run events as the player may see them, in the same order.
#[must_use]
pub fn player_events(events: Vec<RunEvent>) -> Vec<PlayerEvent> {
    events.into_iter().map(RunEvent::for_player).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::content::Content;
    use crate::run::{Run, RunAction};

    /// Instance ids of every card the player must not know: the defenders'
    /// hands and the untouched skat.
    fn hidden_ids(run: &Run) -> Vec<u32> {
        let game = &run.game().expect("game").game;
        let mut ids: Vec<u32> = Seat::DEFENDERS
            .iter()
            .flat_map(|&s| game.hand(s).iter().map(|c| c.id))
            .collect();
        ids.extend(game.skat().iter().map(|c| c.id));
        ids
    }

    #[test]
    fn dealt_shows_only_the_own_hand() {
        let content = Content::load().expect("content");
        let (run, events) = Run::new(&content, 17).expect("start");
        let hidden = hidden_ids(&run);
        let own: Vec<DealtCard> = run.game().expect("game").game.hand(Seat::DECLARER).to_vec();
        let player = player_events(events);
        let dealt = player
            .iter()
            .find_map(|e| match e {
                RunEvent::Game(PlayerGameEvent::Dealt {
                    hand, hand_sizes, ..
                }) => Some((hand.clone(), *hand_sizes)),
                _ => None,
            })
            .expect("dealt");
        assert_eq!(dealt.0, own);
        assert_eq!(dealt.1, [10, 10, 10]);
        let text = ron::to_string(&player).expect("ron");
        assert!(text.contains(&format!("id:{},", own[0].id)));
        for id in hidden {
            assert!(
                !text.contains(&format!("id:{id},")),
                "card {id} leaks into {text}"
            );
        }
    }

    #[test]
    fn played_cards_and_the_picked_up_skat_stay_visible() {
        let content = Content::load().expect("content");
        let (mut run, _) = Run::new(&content, 17).expect("start");
        let skat = run.game().expect("game").game.skat();
        let events = run.apply(&content, RunAction::PickUpSkat).expect("pick up");
        let player = player_events(events);
        assert_eq!(
            player,
            vec![RunEvent::Game(PlayerGameEvent::SkatPickedUp { skat })]
        );
        let hand = run.game().expect("game").game.hand(Seat::DECLARER);
        let discard = [hand[0].id, hand[1].id];
        let events = run
            .apply(&content, RunAction::Discard(discard))
            .expect("discard");
        let events = player_events(events);
        assert!(matches!(
            events[..],
            [RunEvent::Game(PlayerGameEvent::Discarded { .. })]
        ));
        let grand = Declaration::new(
            crate::rules::GameType::Grand,
            crate::rules::RiskLevel::Normal,
        )
        .expect("suit or grand");
        let events = run
            .apply(&content, RunAction::Declare(grand))
            .expect("declare");
        assert!(
            player_events(events).contains(&RunEvent::Game(PlayerGameEvent::Declared {
                declaration: grand
            }))
        );
    }
}

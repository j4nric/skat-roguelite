//! A simple run policy for the simulator and for tests.
//!
//! Games: the declarer heuristic of M1 for the game choice, the discard and
//! the play, plus run decisions on top:
//!
//! - **Hand or skat** (Skatordnung, ADR 0011): with ten cards the bot
//!   declares a hand game only for a very strong hand (eight trumps, or an
//!   announcement below); otherwise it picks up the skat, chooses the game
//!   for all twelve cards ([`DeclarerHeuristic::choose_declaration`]),
//!   discards for that game and declares it.
//! - **Passing** ([`RunBot::is_weak`]): a suit game needs six trumps, or
//!   five trumps and two aces; a grand needs two jacks and two aces. A
//!   weaker hand is thrown in, cheapest first: a redeal ("Schieben"), the
//!   consumable "Neues Blatt", a pass, and with no pass left "Ein Bier"
//!   for one more. The last game of a series is never passed, because a
//!   pass there loses the series for sure. The balancing measurement of
//!   01.11.2026 showed that the weaker hands lose points on average
//!   (`docs/balance/2026-11-01-m2-zielwerte.md`).
//! - **Announcements** ([`RunBot::announcement`]): Schneider angesagt or
//!   Ouvert only for hands that make it almost surely (conservative rules,
//!   measured against the defender heuristic, see the function), Re on
//!   Kontra only with a very strong hand ([`RunBot::says_re`]).
//!
//! Tresen ([`RunBot::shop_action`]): one action per call, in this order:
//! use "Stempelkissen" on an ace, sell the weakest relic for a clearly
//! better one when all slots are full, buy the most valuable relic, buy the
//! most valuable consumable, draw a new offer when nothing sensible can be
//! bought and Groschen are plenty, leave. Values come from
//! [`relic_value`] and [`consumable_value`].
//!
//! Nullspieler ([`RunBot::null_declaration`]): a hand with more than
//! [`NULL_MAX_EXPOSED`] exposed cards (classic Null rule, see
//! [`crate::ai::null_exposure`]) is thrown in like a weak hand; a hand
//! without exposed card is played Ouvert Hand, otherwise the skat is picked
//! up, the two most exposed cards go into it, and the game is Null Ouvert
//! if no exposed card is left, else Null.
//!
//! Encounters ([`RunBot::encounter_action`]): the most valuable house rule
//! of a Stammtisch-Streit, the free card of the Kartengeber, a swap for a
//! relic worth at least [`SWAP_MARGIN`] more (or, in saves where the new
//! relic is unknown, of a relic worth less than an average one), never the
//! double stakes of "Letzte Runde".
//!
//! Figures "Die Oma" and "Der Opa": in a hand game the bot takes the card
//! the run offers from the skat whenever it may, giving away its cheapest
//! side card. It never discards a card that must not be discarded
//! ("Ramponiert") while it has others.
//!
//! The bot never looks at the Zielwert or the series account. The
//! simulator's success curve (`--curve`) relies on that.
//!
//! It lives in the core (not in `sim/`) so that core tests can drive whole
//! runs with it; a frontend could use it for an "auto play" button.

use super::{EncounterChoice, Run, RunAction, RunPhase};
use crate::ai::{DeclarerHeuristic, Policy, null_unsafe_cards, trump_count};
use crate::cards::{Card, CardKind, PlayingCard, Rank, Suit};
use crate::content::{ConsumableAction, ConsumableId, Content, RelicId};
use crate::deck::DealtCard;
use crate::game::{Phase, Seat};
use crate::rules::{Declaration, GameType, NullRisk, RiskLevel};

/// How much the bot wants each relic, roughly by the measured effect of
/// the relic alone (`docs/balance/2026-11-01-m2-zielwerte.md`, "Einzeln ab
/// Spielbeginn"). The bot buys the highest value it can afford and swaps a
/// relic for one that is worth at least [`SWAP_MARGIN`] more. Relics not
/// listed are worth [`DEFAULT_RELIC_VALUE`].
///
/// | Relic | Value | Why |
/// |---|---|---|
/// | `bock_round` | 10 | strongest early (+38 % section 2) |
/// | `old_lady` | 9 | +33 % section 2 |
/// | `concede` | 8 | +24 % section 2, Schneider for free |
/// | `march_through` | 7 | Schneider counts double; pairs with `concede` |
/// | `tour` | 6 | weak early, scales late (+29 % section 3) |
/// | `virgin` | 6 | 61 to 70 Augen count as Schneider |
/// | `skat_redraw` | 6 | one free redeal per series, a pass that costs no game |
/// | `kontra` | 3 | only helps against high Zielwerte, hurts early |
/// | `last_trick_seven` | 2 | rare: last trick with a seven |
/// | `ouvert_bonus` | 1 | the bot almost never plays Ouvert |
/// | `landlord` | 5 | cheaper Tresen |
/// | `blind_flight` | 4 | Null Hand +1 level |
/// | `granny` | 4 | a jack from the skat in hand games |
/// | `referee` | 2 | only against two bosses |
/// | `show_off` | 2 | Ouvert forced: strong but risky |
/// | `kibitzer` | 1 | information the bot does not use |
/// | `card_counter` | 1 | information the bot does not use |
/// | `every_bit_counts` | 9 | +8 Augen in the deck, like the Alte Dame |
/// | `blind_eye` | 7 | wins from 58 Augen |
/// | `tailor` | 6 | 80 to 89 Augen count as Schneider |
/// | `lucky_streak` | 8 | won games count double from the third in a row; alone: section 4 in 2.1 % instead of 0.6 % |
/// | `regular_guest` | 7 | doubles won games after four series; alone: section 4 in 2.8 % instead of 0.6 % |
/// | `tuition` | 4 | +2 Grundwert per lost game, small effect for the bot |
/// | `high_and_dry` | 3 | +1 level when the defenders are dry after three tricks, rare |
/// | `schneider_bock` | 4 | Bock after a won Schneider |
/// | `beer_mat` | 2 | the bot rarely plays hand |
/// | `null_eight_fifteen` | 7 | Null: +8 Grundwert on every Null |
/// | `tally_stick` | 6 | Null: +2 Grundwert per won Null |
/// | `safety_net` | 6 | Null: +1 level for a won Null with the skat, the bot's usual Null |
/// | `fat_sow` | 4 | Null: aces 16 Augen, more Kurzschluss |
/// | `glass_house` | 2 | Null: the bot rarely plays Null Ouvert |
/// | `grandpa` | 4 | a Daus from the skat in hand games, like `granny` |
/// | `with_four` | 3 | +1 level with mit or ohne 4, about one game in ten |
/// | `seven_sleeper` | 6 | Null: +5 Grundwert, the Null deck has eight sevens |
/// | `clean_slate` | 3 | Null: +2 levels when never following suit, rare |
pub const RELIC_VALUES: [(&str, u32); 35] = [
    ("bock_round", 10),
    ("old_lady", 9),
    ("concede", 8),
    ("march_through", 7),
    ("tour", 6),
    ("virgin", 6),
    ("skat_redraw", 6),
    ("landlord", 5),
    ("blind_flight", 4),
    ("granny", 4),
    ("kontra", 3),
    ("last_trick_seven", 2),
    ("referee", 2),
    ("show_off", 2),
    ("ouvert_bonus", 1),
    ("kibitzer", 1),
    ("card_counter", 1),
    ("every_bit_counts", 9),
    ("blind_eye", 7),
    ("tailor", 6),
    ("lucky_streak", 8),
    ("regular_guest", 7),
    ("tuition", 4),
    ("high_and_dry", 3),
    ("schneider_bock", 4),
    ("beer_mat", 2),
    ("null_eight_fifteen", 7),
    ("tally_stick", 6),
    ("safety_net", 6),
    ("fat_sow", 4),
    ("glass_house", 2),
    ("grandpa", 4),
    ("with_four", 3),
    ("seven_sleeper", 6),
    ("clean_slate", 3),
];

/// Value of a relic that is not in [`RELIC_VALUES`].
pub const DEFAULT_RELIC_VALUE: u32 = 3;

/// A relic is swapped only for one worth at least this much more.
pub const SWAP_MARGIN: u32 = 3;

/// How much the bot wants each consumable. Not listed: 0 (never bought).
///
/// | Consumable | Value | Use |
/// |---|---|---|
/// | `new_deal` | 4 | throws a weak hand in without using up a game |
/// | `beer` | 3 | one more pass when the passes are used up |
/// | `ink_pad` | 2 | a random modification on an ace; "Fett" and "Stempel" help the declarer |
/// | `fresh_deck` | 0 | a blank helps defenders as much as the declarer (it may always be played), so the bot does not remove cards |
pub const CONSUMABLE_VALUES: [(&str, u32); 4] = [
    ("new_deal", 4),
    ("beer", 3),
    ("ink_pad", 2),
    ("fresh_deck", 0),
];

/// A Neuauslage needs at least this many Groschen left over after paying
/// for it, enough for a cheap relic from the new offer.
pub const REROLL_RESERVE: u32 = 6;

/// At most this many Neuauslagen per Tresen visit.
pub const MAX_REROLLS: u32 = 2;

/// The bot's value of a relic.
#[must_use]
pub fn relic_value(id: &RelicId) -> u32 {
    RELIC_VALUES
        .iter()
        .find(|(name, _)| *name == id.as_str())
        .map_or(DEFAULT_RELIC_VALUE, |&(_, v)| v)
}

/// The bot's value of a consumable.
#[must_use]
pub fn consumable_value(id: &ConsumableId) -> u32 {
    CONSUMABLE_VALUES
        .iter()
        .find(|(name, _)| *name == id.as_str())
        .map_or(0, |&(_, v)| v)
}

/// The Nullspieler plays a hand with at most this many exposed cards
/// (before the skat). Measured with the Null formula (DESIGN 9.2, 3 × 1000
/// runs): with 2 the bot reaches section 2 in 15 %, with 4 in 28 %, with 5
/// in 35 %; 4 keeps the Nullspieler close to the Alleinspieler. Hands with
/// 3 or 4 exposed cards win 60 % and 45 % of their games.
pub const NULL_MAX_EXPOSED: usize = 4;

/// The simple run policy.
#[derive(Debug, Clone, Copy, Default)]
pub struct RunBot;

fn count_rank<C: PlayingCard>(hand: &[C], rank: Rank) -> usize {
    hand.iter()
        .filter(|c| c.face().is_some_and(|f| f.rank == rank))
        .count()
}

/// Whether the hand holds this printed card.
fn holds<C: PlayingCard>(hand: &[C], card: Card) -> bool {
    hand.iter().any(|c| c.face() == Some(card))
}

/// Number of trumps the hand holds in an unbroken run from the top of the
/// trump order (Kreuz-Bube downwards), i.e. Spitzen "mit".
fn top_trumps<C: PlayingCard>(hand: &[C], game: GameType) -> usize {
    game.trump_order()
        .into_iter()
        .take_while(|&card| holds(hand, card))
        .count()
}

/// Non-trump cards of the hand that may lose a trick: everything but
/// aces and tens backed by their ace.
fn side_losers<C: PlayingCard>(hand: &[C], game: GameType) -> usize {
    hand.iter()
        .filter(|&&c| !crate::rules::is_trump(game, c))
        .filter(|c| match c.face() {
            Some(f) if f.rank == Rank::Ace => false,
            Some(f) if f.rank == Rank::Ten => !holds(hand, Card::new(f.suit, Rank::Ace)),
            _ => true,
        })
        .count()
}

impl RunBot {
    /// Whether a hand is too weak to play (pass rule of DESIGN 10 and the
    /// balancing report): a suit game with fewer than six trumps unless it
    /// has five trumps and two aces; a grand without two jacks and two
    /// aces.
    #[must_use]
    pub fn is_weak(hand: &[DealtCard], declaration: Declaration) -> bool {
        let aces = count_rank(hand, Rank::Ace);
        match declaration.game() {
            GameType::Grand => count_rank(hand, Rank::Jack) < 2 || aces < 2,
            game => {
                let trumps = trump_count(game, hand);
                !(trumps >= 6 || (trumps == 5 && aces >= 2))
            }
        }
    }

    /// The highest Risikostufe the bot dares with this ten-card hand.
    ///
    /// Measured on 2 000 000 deals against the defender heuristic (DESIGN
    /// 14.3): an announcement only pays off when the hand has at most one
    /// side card that may lose a trick ("loser": anything but an ace or a
    /// ten backed by its ace). The rules take exactly the hand shapes whose
    /// mean score beat the plain declaration:
    ///
    /// - **Ouvert** (Hand, Schneider and Schwarz angesagt, open): no loser
    ///   and a suit game with eight trumps and the Kreuz-Bube, or a grand
    ///   with all four jacks. Won 100 % in the sample; about 5 hands in a
    ///   million.
    /// - **Schneider angesagt** (a hand game): a suit game without loser,
    ///   seven trumps and the Kreuz-Bube; or with one loser and eight
    ///   trumps from the top three jacks, or nine trumps from the top two.
    ///   A grand with three jacks and at most one loser. Won 92 to 100 %;
    ///   about 1 hand in 8 000.
    /// - Otherwise the risk of the declarer heuristic (Hand from eight
    ///   trumps).
    #[must_use]
    pub fn announcement(hand: &[DealtCard], declaration: Declaration) -> RiskLevel {
        let game = declaration.game();
        let trumps = trump_count(game, hand);
        let top = top_trumps(hand, game);
        let losers = side_losers(hand, game);
        let (ouvert, schneider) = match game {
            GameType::Suit(_) => (
                losers == 0 && trumps >= 8 && top >= 1,
                (losers == 0 && trumps >= 7 && top >= 1)
                    || (losers == 1 && ((trumps >= 8 && top >= 3) || (trumps >= 9 && top >= 2))),
            ),
            GameType::Grand => (
                losers == 0 && top >= 4,
                losers <= 1 && count_rank(hand, Rank::Jack) >= 3,
            ),
            GameType::Null => (false, false),
        };
        if ouvert {
            RiskLevel::Ouvert
        } else if schneider {
            RiskLevel::SchneiderDeclared
        } else {
            declaration.risk().unwrap_or(RiskLevel::Normal)
        }
    }

    /// Whether to answer Kontra with Re: only with a very strong hand. A
    /// suit game with seven trumps, or six trumps and two aces; a grand
    /// with three jacks and three aces, or all four jacks and two aces.
    #[must_use]
    pub fn says_re(hand: &[DealtCard], game: GameType) -> bool {
        let aces = count_rank(hand, Rank::Ace);
        match game {
            GameType::Grand => {
                let jacks = count_rank(hand, Rank::Jack);
                (jacks >= 3 && aces >= 3) || (jacks == 4 && aces >= 2)
            }
            game => {
                let trumps = trump_count(game, hand);
                trumps >= 7 || (trumps >= 6 && aces >= 2)
            }
        }
    }

    /// The next action, or `None` once the run is over.
    #[must_use]
    pub fn next_action(&self, run: &Run, content: &Content) -> Option<RunAction> {
        match run.phase() {
            RunPhase::Won | RunPhase::Lost => None,
            RunPhase::Shop => Some(self.shop_action(run, content)),
            RunPhase::Encounter => Some(self.encounter_action(run)),
            RunPhase::Game => self.game_action(run, content),
        }
    }

    /// The encounter policy, see the module docs.
    #[must_use]
    pub fn encounter_action(&self, run: &Run) -> RunAction {
        let Some(encounter) = run.encounter() else {
            return RunAction::LeaveShop;
        };
        let leave = encounter.options.len().saturating_sub(1);
        let choose = |option| RunAction::ChooseEncounter {
            option,
            target: None,
        };
        let mut best: Option<(u32, usize)> = None;
        for (i, option) in encounter.options.iter().enumerate() {
            if option.cost > run.coins() {
                continue;
            }
            let value = match &option.choice {
                EncounterChoice::TakeRule(id) => relic_value(id) + 1,
                EncounterChoice::MarkRandomCard => 1,
                // The relic received is known: swap for a clearly better one.
                EncounterChoice::SwapRelic {
                    relic,
                    new: Some(new),
                    ..
                } if relic_value(new) >= relic_value(relic) + SWAP_MARGIN => {
                    relic_value(new) - relic_value(relic)
                }
                EncounterChoice::SwapRelic {
                    relic, new: None, ..
                } if relic_value(relic) < DEFAULT_RELIC_VALUE => {
                    DEFAULT_RELIC_VALUE - relic_value(relic)
                }
                _ => 0,
            };
            if value > 0 && best.is_none_or(|(v, _)| value > v) {
                best = Some((value, i));
            }
        }
        choose(best.map_or(leave, |(_, i)| i))
    }

    /// The Null declaration of the Nullspieler for a ten-card hand, or
    /// `None` if the hand is too weak (more than [`NULL_MAX_EXPOSED`]
    /// exposed cards).
    #[must_use]
    pub fn null_declaration(hand: &[DealtCard], deck: &[DealtCard]) -> Option<NullRisk> {
        match null_unsafe_cards(hand, deck) {
            0 => Some(NullRisk::OuvertHand),
            1..=NULL_MAX_EXPOSED => Some(NullRisk::Normal),
            _ => None,
        }
    }

    /// Slot of an owned consumable with this action, if usable now.
    fn usable(
        run: &Run,
        content: &Content,
        wanted: impl Fn(&ConsumableAction) -> bool,
        target: Option<u32>,
    ) -> Option<RunAction> {
        run.consumables()
            .iter()
            .enumerate()
            .filter(|(_, id)| content.consumable(id).is_some_and(|d| wanted(&d.action)))
            .map(|(slot, _)| RunAction::UseConsumable { slot, target })
            .find(|action| run.accepts(content, action.clone()))
    }

    /// The Tresen policy: one action per call, see the module docs.
    #[must_use]
    pub fn shop_action(&self, run: &Run, content: &Content) -> RunAction {
        let Some(shop) = run.shop() else {
            return RunAction::LeaveShop;
        };
        let coins = run.coins();
        let price = |p: u32| run.shop_price(content, p);
        // 1. Stempelkissen: on an ace, Kreuz first, the least modified one.
        for target in Self::ace_targets(run) {
            if let Some(action) = Self::usable(
                run,
                content,
                |a| matches!(a, ConsumableAction::AddCardMod(_)),
                Some(target),
            ) {
                return action;
            }
        }
        let slots = usize::try_from(content.run.relic_slots).unwrap_or(0);
        let free_slot = run.relics().len() < slots;
        // Relic offers the bot wants: (value, price, slot).
        let offers: Vec<(u32, u32, usize)> = shop
            .relics
            .iter()
            .enumerate()
            .filter_map(|(slot, id)| {
                let id = id.as_ref()?;
                let def = content.relic(id)?;
                Some((relic_value(id), price(def.price), slot))
            })
            .filter(|&(value, _, _)| value > 0)
            .collect();
        // 2. All slots full: sell the weakest relic for a clearly better one.
        if !free_slot {
            let weakest = run
                .relics()
                .iter()
                .enumerate()
                .map(|(index, id)| {
                    let refund = content.relic(id).map_or(0, |d| super::sell_price(d.price));
                    (relic_value(id), index, refund)
                })
                .min_by_key(|&(value, index, _)| (value, std::cmp::Reverse(index)));
            if let Some((weak_value, index, refund)) = weakest {
                let better = offers.iter().any(|&(value, price, _)| {
                    value >= weak_value + SWAP_MARGIN && price <= coins + refund
                });
                if better {
                    return RunAction::SellRelic(index);
                }
            }
        }
        // 3. Buy the most valuable relic.
        if free_slot {
            let best = offers
                .iter()
                .filter(|&&(_, price, _)| price <= coins)
                .max_by_key(|&&(value, price, slot)| (value, price, std::cmp::Reverse(slot)));
            if let Some(&(_, _, slot)) = best {
                return RunAction::BuyRelic(slot);
            }
        }
        // 4. Buy the most valuable consumable.
        let consumable_slot =
            run.consumables().len() < usize::try_from(content.run.consumable_slots).unwrap_or(0);
        if consumable_slot {
            let best = shop
                .consumables
                .iter()
                .enumerate()
                .filter_map(|(slot, id)| {
                    let id = id.as_ref()?;
                    let def = content.consumable(id)?;
                    let value = consumable_value(id);
                    let cost = price(def.price);
                    (value > 0 && cost <= coins).then_some((value, cost, slot))
                })
                .max_by_key(|&(value, price, slot)| (value, price, std::cmp::Reverse(slot)));
            if let Some((_, _, slot)) = best {
                return RunAction::BuyConsumable(slot);
            }
        }
        // 5. Nothing sensible to buy: a new offer, if Groschen are plenty.
        let cost = price(content.run.economy.reroll_cost);
        if shop.rerolls < MAX_REROLLS && coins >= cost + REROLL_RESERVE {
            return RunAction::Reroll;
        }
        RunAction::LeaveShop
    }

    /// Deck ids of the aces, Kreuz first, the least modified first.
    fn ace_targets(run: &Run) -> Vec<u32> {
        let mut aces: Vec<(usize, Suit, u32)> = run
            .deck()
            .cards()
            .iter()
            .filter_map(|c| match c.kind {
                CardKind::Normal(card) if card.rank == Rank::Ace => {
                    Some((c.mods.len(), card.suit, c.id))
                }
                _ => None,
            })
            .collect();
        aces.sort_by_key(|&(mods, suit, id)| (mods, std::cmp::Reverse(suit), id));
        aces.into_iter().map(|(_, _, id)| id).collect()
    }

    fn game_action(&self, run: &Run, content: &Content) -> Option<RunAction> {
        let active = run.game()?;
        let game = &active.game;
        let declarer = DeclarerHeuristic;
        if active.awaiting_re {
            let declaration = game.declaration()?;
            let re = RunBot::says_re(game.hand(Seat::DECLARER), declaration.game());
            return Some(RunAction::AnswerKontra { re });
        }
        match game.phase() {
            Phase::Declaring if run.character().plays_null() => {
                let hand = game.hand(Seat::DECLARER);
                let risk = RunBot::null_declaration(hand, game.deck_cards());
                if risk.is_none() {
                    if let Some(action) = Self::throw_in(run, content) {
                        return Some(action);
                    }
                }
                let mut risk = risk.unwrap_or(NullRisk::Normal);
                if active.rules.require_hand && !risk.hand() {
                    risk = if risk.ouvert() {
                        NullRisk::OuvertHand
                    } else {
                        NullRisk::Hand
                    };
                }
                if risk.hand() {
                    if active.rules.require_ouvert {
                        risk = NullRisk::OuvertHand;
                    }
                    Some(RunAction::Declare(Declaration::null(risk)))
                } else {
                    Some(RunAction::PickUpSkat)
                }
            }
            Phase::Declaring => {
                let hand = game.hand(Seat::DECLARER);
                let mut declaration = declarer.choose_declaration(hand);
                if RunBot::is_weak(hand, declaration) {
                    if let Some(action) = Self::throw_in(run, content) {
                        return Some(action);
                    }
                }
                declaration = declaration.with_risk(RunBot::announcement(hand, declaration));
                if active.rules.require_hand && !declaration.hand() {
                    declaration = declaration.with_risk(RiskLevel::Hand);
                }
                if declaration.hand() {
                    Some(RunAction::Declare(declaration))
                } else {
                    Some(RunAction::PickUpSkat)
                }
            }
            Phase::Discarding => {
                // Saves from before ADR 0011 declared before the skat.
                let declaration = game
                    .declaration()
                    .unwrap_or_else(|| Self::game_after_skat(run));
                let [a, b] =
                    Self::discard_for(game.hand(Seat::DECLARER), declaration, game.deck_cards())?;
                Some(RunAction::Discard([a.id, b.id]))
            }
            Phase::DeclaringAfterSkat => {
                let mut declaration = Self::game_after_skat(run);
                if declaration.is_null() {
                    let hand = game.hand(Seat::DECLARER);
                    let skat = game.skat();
                    let rest: Vec<DealtCard> = game
                        .deck_cards()
                        .iter()
                        .copied()
                        .filter(|c| !skat.contains(c))
                        .collect();
                    if active.rules.require_ouvert || null_unsafe_cards(hand, &rest) == 0 {
                        declaration = Declaration::null(NullRisk::Ouvert);
                    }
                }
                Some(RunAction::Declare(declaration))
            }
            Phase::Playing => {
                if !active.rules.skat_swaps.is_empty() {
                    if let Some(action) = Self::take_from_skat(run, content) {
                        return Some(action);
                    }
                }
                let view = game.view(Seat::DECLARER)?;
                let card = declarer.choose(&view)?;
                Some(RunAction::Play(card.id))
            }
            Phase::Finished(_) => None,
        }
    }

    /// The game the bot plays after picking up the skat, without hand
    /// levels: the Nullspieler Null, the Alleinspieler the game the declarer
    /// heuristic picks for all twelve cards
    /// ([`DeclarerHeuristic::choose_game_after_skat`] over the ten dealt
    /// cards plus the skat, also once discarded, so that discard and
    /// declaration agree).
    fn game_after_skat(run: &Run) -> Declaration {
        if run.character().plays_null() {
            return Declaration::null(NullRisk::Normal);
        }
        let cards = run.game().map_or(&[][..], |a| a.game.declarer_cards());
        DeclarerHeuristic.choose_game_after_skat(cards)
    }

    /// The two discards for `declaration` from the twelve cards: the declarer
    /// heuristic's choice (the most exposed cards in a Null), never a card
    /// that must not be discarded while others are left.
    #[must_use]
    pub fn discard_for(
        cards: &[DealtCard],
        declaration: Declaration,
        deck: &[DealtCard],
    ) -> Option<[DealtCard; 2]> {
        let declarer = DeclarerHeuristic;
        Self::discard(cards, |cards| {
            if declaration.is_null() {
                declarer.choose_null_discard(cards, deck)
            } else {
                declarer.choose_discard(cards, declaration.game())
            }
        })
    }

    /// The two discards: chosen by `choose` among the cards that may be
    /// discarded ("Ramponiert" stays in the hand); only if fewer than two
    /// are left, the rest is filled from the others.
    fn discard(
        hand: &[DealtCard],
        choose: impl Fn(&[DealtCard]) -> Option<[DealtCard; 2]>,
    ) -> Option<[DealtCard; 2]> {
        let free: Vec<DealtCard> = hand.iter().copied().filter(|c| !c.no_discard()).collect();
        if free.len() >= 2 {
            return choose(&free);
        }
        let mut chosen = free;
        let bound: Vec<DealtCard> = hand.iter().copied().filter(|c| c.no_discard()).collect();
        let fill = choose(&bound).map_or_else(|| bound.clone(), |pair| pair.to_vec());
        chosen.extend(fill.into_iter().take(2 - chosen.len()));
        match chosen[..] {
            [a, b] => Some([a, b]),
            _ => None,
        }
    }

    /// Skat swaps ("Die Oma", "Der Opa"): give the cheapest side card for
    /// the card the run offers first.
    fn take_from_skat(run: &Run, content: &Content) -> Option<RunAction> {
        let active = run.game()?;
        let game = &active.game;
        let trump = game.declaration()?.game();
        let give = game
            .hand(Seat::DECLARER)
            .iter()
            .copied()
            .filter(|&c| !crate::rules::is_trump(trump, c) && !c.no_discard())
            .min_by_key(|c| (c.card_points(), c.face().map(|f| f.rank)))?;
        let action = RunAction::TakeFromSkat {
            give: give.id,
            take: None,
        };
        run.accepts(content, action.clone()).then_some(action)
    }

    /// The cheapest way to throw a weak hand in, if any: redeal, "Neues
    /// Blatt", pass, "Ein Bier" for one more pass. Never on the last game.
    fn throw_in(run: &Run, content: &Content) -> Option<RunAction> {
        if run.redeals_left() > 0 {
            return Some(RunAction::Redeal);
        }
        if let Some(action) = Self::usable(
            run,
            content,
            |a| matches!(a, ConsumableAction::Redeal),
            None,
        ) {
            return Some(action);
        }
        let series = run.series();
        if series.games_left() <= 1 {
            return None;
        }
        if series.passes_left > 0 {
            return Some(RunAction::Pass);
        }
        Self::usable(
            run,
            content,
            |a| matches!(a, ConsumableAction::ExtraPasses(_)),
            None,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cards::CardKind;

    fn hand(cards: &str) -> Vec<DealtCard> {
        cards
            .split_whitespace()
            .zip(0u32..)
            .map(|(s, id)| DealtCard::plain(id, s.parse::<CardKind>().expect("card")))
            .collect()
    }

    fn suit(s: Suit) -> Declaration {
        Declaration::suit(s, RiskLevel::Normal)
    }

    #[test]
    fn pass_rule_follows_trumps_and_aces() {
        let hearts = suit(Suit::Hearts);
        let five_two_aces = hand("CJ HA HT HK H9 SA ST S9 D7 D8");
        assert!(!RunBot::is_weak(&five_two_aces, hearts));
        let five_one_ace = hand("CJ HA HT HK H9 SK ST S9 D7 D8");
        assert!(RunBot::is_weak(&five_one_ace, hearts));
        let six = hand("CJ SJ HT HK H9 H8 ST S9 D7 D8");
        assert!(!RunBot::is_weak(&six, hearts));
        let grand = Declaration::grand(RiskLevel::Normal);
        assert!(!RunBot::is_weak(
            &hand("CJ DJ CA SA S9 S8 H7 H8 D7 D8"),
            grand
        ));
        assert!(RunBot::is_weak(
            &hand("CJ SJ HJ CA S9 S8 H7 H8 D7 D8"),
            grand
        ));
    }

    #[test]
    fn announcements_are_conservative() {
        let clubs = suit(Suit::Clubs);
        // Eight top trumps and two aces: Ouvert.
        let ouvert = hand("CJ SJ HJ DJ CA CT CK CQ HA SA");
        assert_eq!(RunBot::announcement(&ouvert, clubs), RiskLevel::Ouvert);
        // Seven trumps with the top three jacks and standing aces: Schneider.
        let schneider = hand("CJ SJ HJ CA CT CK C9 HA SA ST");
        assert_eq!(
            RunBot::announcement(&schneider, clubs),
            RiskLevel::SchneiderDeclared
        );
        // A bare king outside: no announcement.
        let bare = hand("CJ SJ HJ CA CT CK C9 HA SA SK");
        assert_eq!(RunBot::announcement(&bare, clubs), RiskLevel::Normal);
        // Eight trumps from the top three jacks and one loser: Schneider.
        let one_loser = hand("CJ SJ HJ CA CT CK C9 C8 SA SK");
        assert_eq!(
            RunBot::announcement(&one_loser, clubs),
            RiskLevel::SchneiderDeclared
        );
        let grand = Declaration::grand(RiskLevel::Normal);
        let jacks = hand("CJ SJ HJ DJ CA CT SA ST HA DA");
        assert_eq!(RunBot::announcement(&jacks, grand), RiskLevel::Ouvert);
        let three = hand("CJ SJ HJ CA CT SA ST HA DA D7");
        assert_eq!(
            RunBot::announcement(&three, grand),
            RiskLevel::SchneiderDeclared
        );
        let two_losers = hand("CJ SJ HJ CA CT SA ST HA D8 D7");
        assert_eq!(RunBot::announcement(&two_losers, grand), RiskLevel::Normal);
        assert!(RunBot::says_re(&jacks, GameType::Grand));
        assert!(!RunBot::says_re(&bare[..8], GameType::Grand));
    }

    fn open_shop(content: &Content, coins: u32) -> Run {
        let (mut run, _) = Run::new(content, 9).expect("start");
        run.coins = coins;
        run.game = None;
        run.phase = RunPhase::Shop;
        run.shop = Some(run.draw_offer(content, 0));
        run
    }

    fn offer(run: &mut Run, relics: &[&str], consumables: &[&str]) {
        let shop = run.shop.as_mut().expect("shop");
        shop.relics = relics.iter().map(|r| Some(RelicId::new(r))).collect();
        shop.consumables = consumables
            .iter()
            .map(|c| Some(ConsumableId::new(c)))
            .collect();
    }

    #[test]
    fn shop_swaps_the_weakest_relic_for_a_clearly_better_one() {
        let content = Content::load().expect("content");
        let mut run = open_shop(&content, 5);
        run.relics = ["kontra", "ouvert_bonus", "tour", "virgin", "concede"]
            .iter()
            .map(|r| RelicId::new(r))
            .collect();
        offer(&mut run, &["bock_round"], &[]);
        // Bock costs 8; selling Ouvert-Bonus brings 2: 7 is not enough.
        assert_eq!(RunBot.shop_action(&run, &content), RunAction::LeaveShop);
        run.coins = 6;
        assert_eq!(RunBot.shop_action(&run, &content), RunAction::SellRelic(1));
        run.apply(&content, RunAction::SellRelic(1)).expect("sell");
        assert_eq!(RunBot.shop_action(&run, &content), RunAction::BuyRelic(0));
        // Not clearly better: Tournee (6) for Kontra (3) is, Kontra for
        // Ouvert-Bonus is not.
        let mut run = open_shop(&content, 50);
        run.relics = ["ouvert_bonus", "bock_round", "old_lady", "concede", "tour"]
            .iter()
            .map(|r| RelicId::new(r))
            .collect();
        offer(&mut run, &["kontra"], &[]);
        assert_ne!(RunBot.shop_action(&run, &content), RunAction::SellRelic(0));
    }

    #[test]
    fn shop_buys_consumables_and_puts_the_ink_pad_on_an_ace() {
        let content = Content::load().expect("content");
        let mut run = open_shop(&content, 10);
        offer(&mut run, &[], &["ink_pad", "beer"]);
        assert_eq!(
            RunBot.shop_action(&run, &content),
            RunAction::BuyConsumable(1)
        );
        run.apply(&content, RunAction::BuyConsumable(1))
            .expect("beer");
        assert_eq!(
            RunBot.shop_action(&run, &content),
            RunAction::BuyConsumable(0)
        );
        run.apply(&content, RunAction::BuyConsumable(0))
            .expect("ink");
        let kreuz_ass = 31;
        assert_eq!(
            RunBot.shop_action(&run, &content),
            RunAction::UseConsumable {
                slot: 1,
                target: Some(kreuz_ass)
            }
        );
        // "Frisches Deck" is never bought.
        let mut run = open_shop(&content, 10);
        offer(&mut run, &[], &["fresh_deck"]);
        assert_ne!(
            RunBot.shop_action(&run, &content),
            RunAction::BuyConsumable(0)
        );
    }

    #[test]
    fn shop_rerolls_only_with_plenty_of_groschen() {
        let content = Content::load().expect("content");
        let cost = content.run.economy.reroll_cost;
        let mut run = open_shop(&content, cost + REROLL_RESERVE);
        offer(&mut run, &[], &[]);
        assert_eq!(RunBot.shop_action(&run, &content), RunAction::Reroll);
        run.coins -= 1;
        assert_eq!(RunBot.shop_action(&run, &content), RunAction::LeaveShop);
        run.coins += 1;
        run.shop.as_mut().expect("shop").rerolls = MAX_REROLLS;
        assert_eq!(RunBot.shop_action(&run, &content), RunAction::LeaveShop);
    }

    #[test]
    fn weak_hands_take_the_cheapest_way_out() {
        let content = Content::load().expect("content");
        let mut seed = 0;
        let mut run = loop {
            let (run, _) = Run::new(&content, seed).expect("start");
            let hand = run.game().expect("game").game.hand(Seat::DECLARER);
            if RunBot::is_weak(hand, DeclarerHeuristic.choose_declaration(hand)) {
                break run;
            }
            seed += 1;
        };
        run.consumables = vec![ConsumableId::new("beer"), ConsumableId::new("new_deal")];
        let next = |run: &Run| RunBot.next_action(run, &content).expect("action");
        assert_eq!(
            next(&run),
            RunAction::UseConsumable {
                slot: 1,
                target: None
            }
        );
        run.consumables.pop();
        assert_eq!(next(&run), RunAction::Pass);
        run.series.passes_left = 0;
        assert_eq!(
            next(&run),
            RunAction::UseConsumable {
                slot: 0,
                target: None
            }
        );
        // The last game is played, whatever the hand: with the skat.
        run.series.games_played = run.series.game_limit - 1;
        assert_eq!(next(&run), RunAction::PickUpSkat);
    }

    /// Hand only for a very strong hand; otherwise pick up the skat, then
    /// discard and declare the game chosen for all twelve cards.
    #[test]
    fn the_bot_sees_the_skat_before_it_chooses_the_game() {
        let content = Content::load().expect("content");
        let (mut hand_games, mut skat_games) = (0, 0);
        for seed in 0..60 {
            let (mut run, _) = Run::new(&content, seed).expect("start");
            let next = |run: &Run| RunBot.next_action(run, &content).expect("action");
            match next(&run) {
                RunAction::Declare(d) => {
                    assert!(d.hand(), "seed {seed}: {d:?} before the skat");
                    hand_games += 1;
                }
                RunAction::PickUpSkat => {
                    let twelve = run.game().expect("game").game.declarer_cards().to_vec();
                    let planned = DeclarerHeuristic.choose_game_after_skat(&twelve);
                    run.apply(&content, RunAction::PickUpSkat).expect("pick up");
                    let RunAction::Discard(ids) = next(&run) else {
                        panic!("seed {seed}: no discard after picking up");
                    };
                    let hand = run.game().expect("game").game.hand(Seat::DECLARER);
                    let expected = RunBot::discard_for(hand, planned, &[]).expect("discard");
                    assert_eq!(ids, [expected[0].id, expected[1].id], "seed {seed}");
                    run.apply(&content, RunAction::Discard(ids))
                        .expect("discard");
                    assert_eq!(next(&run), RunAction::Declare(planned), "seed {seed}");
                    assert!(!planned.hand());
                    run.apply(&content, RunAction::Declare(planned))
                        .expect("declare");
                    skat_games += 1;
                }
                _ => {} // thrown in
            }
        }
        assert!(
            skat_games > 10 && hand_games < skat_games,
            "{hand_games} {skat_games}"
        );
    }

    #[test]
    fn the_null_bot_picks_up_and_plays_ouvert_when_nothing_is_exposed() {
        let content = Content::load().expect("content");
        let start = super::super::RunStart {
            character: crate::content::Character::NullPlayer,
            ..super::super::RunStart::default()
        };
        let mut seen = [0; 4];
        for seed in 0..80 {
            let (mut run, _) = Run::new_with(&content, seed, &start).expect("start");
            let next = |run: &Run| RunBot.next_action(run, &content).expect("action");
            match next(&run) {
                RunAction::Declare(d) => {
                    assert_eq!(d, Declaration::null(NullRisk::OuvertHand));
                    seen[0] += 1;
                }
                RunAction::PickUpSkat => {
                    run.apply(&content, RunAction::PickUpSkat).expect("pick up");
                    let action = next(&run);
                    run.apply(&content, action).expect("discard");
                    let RunAction::Declare(d) = next(&run) else {
                        panic!("seed {seed}: no declaration after the discard");
                    };
                    let game = &run.game().expect("game").game;
                    let skat = game.skat();
                    let rest: Vec<DealtCard> = game
                        .deck_cards()
                        .iter()
                        .copied()
                        .filter(|c| !skat.contains(c))
                        .collect();
                    let exposed = null_unsafe_cards(game.hand(Seat::DECLARER), &rest);
                    let expected = if exposed == 0 {
                        NullRisk::Ouvert
                    } else {
                        NullRisk::Normal
                    };
                    assert_eq!(d, Declaration::null(expected), "seed {seed}");
                    seen[if exposed == 0 { 1 } else { 2 }] += 1;
                }
                _ => seen[3] += 1,
            }
        }
        assert!(seen[2] > 0, "{seen:?}");
    }

    #[test]
    fn value_tables_know_every_content_item() {
        let content = Content::load().expect("content");
        for id in content.relics.keys() {
            assert!(
                RELIC_VALUES.iter().any(|(name, _)| *name == id.as_str()),
                "relic {id} has no bot value"
            );
        }
        for id in content.consumables.keys() {
            assert!(
                CONSUMABLE_VALUES
                    .iter()
                    .any(|(name, _)| *name == id.as_str()),
                "consumable {id} has no bot value"
            );
        }
    }
}

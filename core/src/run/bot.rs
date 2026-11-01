//! A simple run policy for the simulator and for tests.
//!
//! Games: the declarer heuristic of M1 for the card play and the suit game
//! or grand, plus run decisions on top. Every character may declare every
//! game (ADR 0012), so the bot chooses between suit game, grand and Null
//! for both:
//!
//! - **Hand or skat** (Skatordnung, ADR 0011): with ten cards the bot
//!   declares a hand game only for a very strong hand: Null Ouvert Hand
//!   without an exposed card (classic Null rule, see
//!   [`crate::ai::null_exposure`]), a suit game or grand with eight trumps
//!   or an announcement below. Otherwise it picks up the skat and chooses
//!   with twelve cards (`RunBot::after_skat`): the suit game or grand of
//!   [`DeclarerHeuristic::choose_game_after_skat`] or Null (the two most
//!   exposed cards discarded, Null Ouvert if none is left), whichever has
//!   the better expected score ([`suit_expected_score`],
//!   [`null_expected_score`], win rates measured with the PIMC player). Null
//!   is left out where a won Null counts nothing ("Der Schwarze Peter").
//! - **Passing** ([`RunBot::is_weak`]): a suit game needs six trumps, or
//!   five trumps and two aces; a grand needs two jacks and two aces; Null
//!   at most [`NULL_MAX_EXPOSED`] exposed cards. A hand weak for both is
//!   thrown in, cheapest first: a redeal ("Schieben"), the consumable
//!   "Neues Blatt", a pass, and with no pass left "Ein Bier" for one more.
//!   The last game of a series is never passed, because a pass there loses
//!   the series for sure. The balancing measurement of 01.11.2026 showed
//!   that the weaker hands lose points on average
//!   (`docs/balance/2026-11-01-m2-zielwerte.md`).
//! - **Forced hand** ("Der Schieber", `RunBot::forced_hand`): Null Hand
//!   only with at most [`NULL_HAND_MAX_EXPOSED`] exposed cards, else the
//!   suit game or grand with Hand; the better expected score if both fit.
//! - **Always open** ("Der Angeber"): an open suit game or grand only with
//!   an Ouvert hand, otherwise Null Ouvert (Hand without exposed card).
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
//! Encounters ([`RunBot::encounter_action`]): the most valuable house rule
//! of a Stammtisch-Streit, the free card of the Kartengeber, a swap for a
//! relic worth at least [`SWAP_MARGIN`] more (or, in saves where the new
//! relic is unknown, of a relic worth less than an average one), never the
//! double stakes of "Letzte Runde".
//!
//! Schnäpse (simple rules, not tuned): "Kurzer" right before picking up the
//! skat (the bot does not use what it sees); after the skat, "Kreide"
//! paints the cheapest side card (never a jack) into the trump suit of a
//! suit game, then "Doppelkorn" before a game that is not weak
//! ([`RunBot::is_weak`]) and "Magenbitter" before a weak one, never before
//! a Null. "Kräuter" counts as "Ein Bier".
//!
//! Figures "Die Oma" and "Der Opa": in a hand game the bot takes the card
//! the run offers from the skat whenever it may (never in a Null), giving
//! away its cheapest side card. It never discards a card that must not be discarded
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
use crate::content::{ConsumableAction, ConsumableId, Content, NextGameEffect, RelicId};
use crate::deck::DealtCard;
use crate::game::{Phase, Seat};
use crate::rules::{Declaration, GameType, NullRisk, NullScoring, RiskLevel};

/// How much the bot wants each relic, by the measured effect of the relic
/// alone (`docs/balance/2026-11-01-m5-v1.md`: PIMC card play, relic from the
/// start, no Tresen, 3 × 500 runs; change of "section 3 reached" in
/// percentage points for the Alleinspieler unless noted). The bot buys the
/// highest value it can afford and swaps a relic for one that is worth at
/// least [`SWAP_MARGIN`] more; value 0 is never bought. One table serves
/// both characters. Relics not listed are worth [`DEFAULT_RELIC_VALUE`].
///
/// | Relic | Value | Measured |
/// |---|---|---|
/// | `every_bit_counts` | 10 | +8.3 section 3 (Nullspieler +0.8) |
/// | `bock_round` | 10 | +7.1 (Nullspieler +7.7) |
/// | `old_lady` | 10 | +7.1 |
/// | `regular_guest` | 8 | +5.6, section 4 +6.7: the late multiplier |
/// | `concede` | 8 | +5.1 |
/// | `blind_eye` | 8 | +4.9 |
/// | `seven_sleeper` | 8 | Null: +5.5 |
/// | `skat_redraw` | 7 | +3.5 (Nullspieler +4.7) |
/// | `march_through` | 7 | +4.5 |
/// | `safety_net` | 7 | Null: +4.3 |
/// | `lucky_streak` | 6 | +4.0, section 4 +4.0 (Nullspieler +0.1) |
/// | `tuition` | 6 | +2.3 (Nullspieler +4.2) |
/// | `virgin` | 6 | +3.2 |
/// | `tour` | 6 | +3.1, grows over the run |
/// | `tailor` | 6 | +3.1 |
/// | `landlord` | 5 | cheaper Tresen, measured in runs with the Tresen |
/// | `null_eight_fifteen` | 5 | Null: +2.9 |
/// | `tally_stick` | 5 | Null: +2.7 |
/// | `fat_sow` | 4 | Null: +1.7 |
/// | `kontra` | 3 | +1.6, variance: helps late, hurts the Nullspieler (-0.9) |
/// | `kibitzer` | 3 | +1.3, PIMC samples use the peeked card |
/// | `high_and_dry` | 3 | +1.2 |
/// | `schneider_bock` | 3 | +1.1 |
/// | `granny` | 2 | +0.5, the bot rarely plays hand |
/// | `grandpa` | 2 | +0.3, the bot rarely plays hand |
/// | `last_trick_seven` | 2 | +0.4 |
/// | `with_four` | 2 | +0.1 |
/// | `blind_flight` | 2 | Null: +0.3, the bot rarely plays Null Hand |
/// | `referee` | 1 | +0.2, only against two bosses |
/// | `beer_mat` | 1 | +0.4, the bot rarely plays hand |
/// | `ouvert_bonus` | 1 | 0, the bot almost never plays Ouvert |
/// | `glass_house` | 1 | Null: +0.3, the bot rarely plays Null Ouvert |
/// | `clean_slate` | 1 | Null: +0.1, the bot almost never avoids following suit |
/// | `card_counter` | 1 | information the PIMC bot already has |
/// | `show_off` | 0 | Null: -15.9, forced Ouvert ruins the bot: never bought |
pub const RELIC_VALUES: [(&str, u32); 35] = [
    ("every_bit_counts", 10),
    ("bock_round", 10),
    ("old_lady", 10),
    ("regular_guest", 8),
    ("concede", 8),
    ("blind_eye", 8),
    ("seven_sleeper", 8),
    ("skat_redraw", 7),
    ("march_through", 7),
    ("safety_net", 7),
    ("lucky_streak", 6),
    ("tuition", 6),
    ("virgin", 6),
    ("tour", 6),
    ("tailor", 6),
    ("landlord", 5),
    ("null_eight_fifteen", 5),
    ("tally_stick", 5),
    ("fat_sow", 4),
    ("kontra", 3),
    ("kibitzer", 3),
    ("high_and_dry", 3),
    ("schneider_bock", 3),
    ("granny", 2),
    ("grandpa", 2),
    ("last_trick_seven", 2),
    ("with_four", 2),
    ("blind_flight", 2),
    ("referee", 1),
    ("beer_mat", 1),
    ("ouvert_bonus", 1),
    ("glass_house", 1),
    ("clean_slate", 1),
    ("card_counter", 1),
    ("show_off", 0),
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
/// | `herbal` | 3 | "Kräuter": like "Ein Bier" |
/// | `double_grain` | 2 | "Doppelkorn": before a game that is not weak |
/// | `bitters` | 2 | "Magenbitter": before a weak game |
/// | `chalk` | 2 | "Kreide": the cheapest side card into trump |
/// | `shot` | 1 | "Kurzer": before picking up the skat; the bot does not use what it sees |
///
/// The Schnäpse are placeholders, not measured.
pub const CONSUMABLE_VALUES: [(&str, u32); 9] = [
    ("new_deal", 4),
    ("beer", 3),
    ("herbal", 3),
    ("ink_pad", 2),
    ("double_grain", 2),
    ("bitters", 2),
    ("chalk", 2),
    ("shot", 1),
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
/// in 35 %. Hands with 3 or 4 exposed cards win 60 % and 45 % of their
/// games. M5 (`docs/balance/2026-11-01-m5-v1.md`, PIMC card play, 8 games
/// and 6 passes per series): section 3 in 20.1 % with 4, 25.1 % with 5 and
/// 24.8 % with 6 (6 plays a third more games per run). 5 is the stronger
/// policy, and the bot stands in for a good player.
pub const NULL_MAX_EXPOSED: usize = 5;

/// Under forced hand play ("Der Schieber") the bot declares Null Hand only
/// with at most this many exposed cards: it cannot discard them. Null after
/// the skat wins 74 % with one and 51 % with two exposed cards left, 28 %
/// with three ([`NULL_WIN_PERCENT`]).
pub const NULL_HAND_MAX_EXPOSED: usize = 2;

/// Percent of Null games the bot wins by exposed cards in the ten cards it
/// plays (index; 0 is played Ouvert). Measured with the PIMC player
/// (`--bot pimc:12:3`, 01.11.2026), Null after the skat: the Nullspieler
/// with eight sevens won 94, 74, 51, 28, 19 and 7 % for 0 to 5 (600 runs,
/// seed 99, 5300 games), the Alleinspieler with the normal deck 99, 79, 58,
/// 41, 33 and 21 % (300 runs, seed 42, 1100 games). The table is the mean.
/// More than 7: 0.
pub const NULL_WIN_PERCENT: [i64; 8] = [96, 76, 54, 34, 26, 14, 7, 3];

/// Percent of suit games the bot wins, by trumps (index, up to 8) and aces
/// (up to 3) of the ten cards it plays, in a deck with all eleven trumps
/// (the Alleinspieler, 600 runs, seed 99, 2516 suit games); hands with
/// fewer than five trumps are rare (the bot passes them) and rounded. A
/// deck with fewer trumps counts each missing one as held (the
/// Nullspieler's suits have eight: with four or five he won 92 %).
pub const SUIT_WIN_PERCENT: [[i64; 4]; 9] = [
    [0, 0, 5, 10],
    [0, 0, 5, 10],
    [0, 5, 10, 15],
    [0, 5, 15, 20],
    [5, 20, 44, 50],
    [5, 23, 52, 85],
    [55, 74, 81, 93],
    [94, 94, 97, 98],
    [98, 98, 98, 98],
];

/// Trumps of a suit game in the standard deck: four jacks and seven cards.
const SUIT_TRUMPS: usize = 11;

/// Percent of grands the bot wins by jacks (index) and aces (up to 3), same
/// measurement (1650 grands, almost all with two jacks and two aces or
/// more).
pub const GRAND_WIN_PERCENT: [[i64; 4]; 5] = [
    [0, 0, 2, 5],
    [0, 2, 5, 10],
    [5, 20, 67, 89],
    [20, 60, 93, 98],
    [90, 98, 98, 98],
];

/// Expected style bonus of a won Null in hundredths of a Gewinnstufe:
/// a ducked seven per seven held, plus this for "nie bedient" and
/// "Kurzschluss". Measured: a won Null with the skat counts on average
/// 23 × (1.3 + sevens) (Nullspieler, same measurement).
const NULL_STYLE_EXTRA: i64 = 30;

fn percent(table: &[i64], index: usize) -> i64 {
    table.get(index).copied().unwrap_or(0)
}

/// Expected score of a Null with these ten cards, in hundredths of a point
/// (won: the value; lost: minus twice the value, as in the series
/// account), without relics. `exposed` counts the exposed cards
/// ([`null_unsafe_cards`]). The Null formula (DESIGN 9.2) counts the
/// expected style bonus, each seven held as one ducked trick.
#[must_use]
pub fn null_expected_score(
    hand: &[DealtCard],
    exposed: usize,
    declaration: Declaration,
    scoring: &NullScoring,
) -> i64 {
    let Some(risk) = declaration.null_risk() else {
        return 0;
    };
    let win = percent(&NULL_WIN_PERCENT, exposed);
    let (won, lost) = match scoring {
        NullScoring::Fixed => {
            let value = i64::from(crate::rules::null_value(risk.hand(), risk.ouvert())) * 100;
            (value, 2 * value)
        }
        NullScoring::Formula(f) => {
            let base = i64::from(f.base_value);
            let step = i64::from(risk.step()) * 100;
            let sevens = i64::try_from(count_rank(hand, Rank::Seven)).unwrap_or(0);
            let style = sevens * i64::from(f.per_seven_ducked) * 100 + NULL_STYLE_EXTRA;
            (base * (step + style), 2 * base * step)
        }
    };
    (win * won - (100 - win) * lost) / 100
}

/// Expected score of a suit game or grand with these ten cards, in
/// hundredths of a point, without relics: the Spielwert with the Spitzen
/// of `all_cards` (the ten plus the skat, which counts for the Spitzen),
/// won with [`SUIT_WIN_PERCENT`] or [`GRAND_WIN_PERCENT`], lost twice.
/// `deck` is every card of the game (for trumps missing from the deck).
#[must_use]
pub fn suit_expected_score(
    hand: &[DealtCard],
    all_cards: &[DealtCard],
    deck: &[DealtCard],
    declaration: Declaration,
) -> i64 {
    let game = declaration.game();
    let Some(base) = game.base_value() else {
        return 0;
    };
    let aces = count_rank(hand, Rank::Ace).min(3);
    let win = match game {
        GameType::Grand => GRAND_WIN_PERCENT
            .get(count_rank(hand, Rank::Jack).min(4))
            .map_or(0, |row| percent(row, aces)),
        _ => {
            let missing = SUIT_TRUMPS.saturating_sub(trump_count(game, deck));
            SUIT_WIN_PERCENT
                .get((trump_count(game, hand) + missing).min(8))
                .map_or(0, |row| percent(row, aces))
        }
    };
    let faces: Vec<Card> = all_cards.iter().filter_map(|c| c.face()).collect();
    let matadors = crate::rules::matadors(game, &faces).count;
    let levels = matadors + 1 + u32::from(declaration.hand());
    let value = i64::from(base) * i64::from(levels) * 100;
    (win * value - (100 - win) * 2 * value) / 100
}

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
            .map(|(slot, _)| RunAction::UseConsumable {
                slot,
                target,
                suit: None,
            })
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
            Phase::Declaring => {
                let action = Self::before_skat(run, content)?;
                if action == RunAction::PickUpSkat {
                    if let Some(peek) = Self::usable(
                        run,
                        content,
                        |a| matches!(a, ConsumableAction::PeekSkat),
                        None,
                    ) {
                        return Some(peek);
                    }
                }
                Some(action)
            }
            Phase::Discarding => {
                // Saves from before ADR 0011 declared before the skat.
                if let Some(declaration) = game.declaration() {
                    let [a, b] = Self::discard_for(
                        game.hand(Seat::DECLARER),
                        declaration,
                        game.deck_cards(),
                    )?;
                    return Some(RunAction::Discard([a.id, b.id]));
                }
                let (_, [a, b]) = Self::after_skat(run, content)?;
                Some(RunAction::Discard([a.id, b.id]))
            }
            Phase::DeclaringAfterSkat => {
                let (declaration, _) = Self::after_skat(run, content)?;
                if let Some(action) = Self::schnapps_before_declaring(run, content, declaration) {
                    return Some(action);
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

    /// The decision with ten cards (Skatordnung, ADR 0011), for every
    /// character (ADR 0012):
    ///
    /// - Null Ouvert Hand without an exposed card.
    /// - A hand game of the declarer heuristic (eight trumps, or an
    ///   announcement from [`RunBot::announcement`]).
    /// - Throw the hand in ([`RunBot::throw_in`]) if it is weak for a suit
    ///   game or grand ([`RunBot::is_weak`]) and has more than
    ///   [`NULL_MAX_EXPOSED`] exposed cards for Null.
    /// - Otherwise pick up the skat; the game is chosen with twelve cards
    ///   ([`RunBot::after_skat`]).
    ///
    /// "Der Schieber" (hand games only) and "Der Angeber" (always open)
    /// narrow the choice, see [`RunBot::forced_hand`].
    fn before_skat(run: &Run, content: &Content) -> Option<RunAction> {
        let active = run.game()?;
        let game = &active.game;
        let hand = game.hand(Seat::DECLARER);
        let deck = game.deck_cards();
        let exposed = null_unsafe_cards(hand, deck);
        let suit = DeclarerHeuristic.choose_declaration(hand);
        let suit = suit.with_risk(RunBot::announcement(hand, suit));
        let rules = &active.rules;
        if rules.require_hand {
            return Self::forced_hand(run, content);
        }
        let null_counts = Self::null_counts(run, content, None);
        let exposed = if null_counts { exposed } else { usize::MAX };
        if exposed == 0 {
            return Some(RunAction::Declare(Declaration::null(NullRisk::OuvertHand)));
        }
        if rules.require_ouvert {
            if suit.ouvert() {
                return Some(RunAction::Declare(suit));
            }
        } else if suit.hand() {
            return Some(RunAction::Declare(suit));
        }
        let weak = if rules.require_ouvert {
            true
        } else {
            RunBot::is_weak(hand, suit)
        };
        if weak && exposed > NULL_MAX_EXPOSED {
            if let Some(action) = Self::throw_in(run, content) {
                return Some(action);
            }
        }
        Some(RunAction::PickUpSkat)
    }

    /// A hand game is forced ("Der Schieber"): the declaration with the
    /// better expected score of the heuristic's suit game or grand with Hand
    /// and Null Hand (Null Ouvert Hand without an exposed card). Without the
    /// skat the bot cannot discard its exposed cards, so it takes Null Hand
    /// only with at most [`NULL_HAND_MAX_EXPOSED`] of them. A weak hand is
    /// thrown in if it can be; "Der Angeber" leaves Null Ouvert Hand or an
    /// open suit game.
    fn forced_hand(run: &Run, content: &Content) -> Option<RunAction> {
        let active = run.game()?;
        let game = &active.game;
        let hand = game.hand(Seat::DECLARER);
        let exposed = null_unsafe_cards(hand, game.deck_cards());
        let scoring = run.null_scoring(content);
        let mut suit = DeclarerHeuristic.choose_declaration(hand);
        suit = suit.with_risk(RunBot::announcement(hand, suit).max(RiskLevel::Hand));
        let null = if exposed == 0 || active.rules.require_ouvert {
            Declaration::null(NullRisk::OuvertHand)
        } else {
            Declaration::null(NullRisk::Hand)
        };
        let null_ev = null_expected_score(hand, exposed, null, &scoring);
        let suit_ev = suit_expected_score(hand, hand, game.deck_cards(), suit);
        let null_fits = exposed <= NULL_HAND_MAX_EXPOSED && Self::null_counts(run, content, None);
        let suit_fits =
            !RunBot::is_weak(hand, suit) && (!active.rules.require_ouvert || suit.ouvert());
        if !null_fits && !suit_fits {
            if let Some(action) = Self::throw_in(run, content) {
                return Some(action);
            }
        }
        let take_null = if active.rules.require_ouvert {
            !suit.ouvert() || null_ev > suit_ev
        } else {
            match (null_fits, suit_fits) {
                (true, false) => true,
                (false, true) => false,
                _ => null_ev > suit_ev,
            }
        };
        Some(RunAction::Declare(if take_null { null } else { suit }))
    }

    /// The game the bot declares after picking up the skat, with the two
    /// cards it discards for it, from the twelve cards (also once discarded,
    /// so that discard and declaration agree). For every character (ADR
    /// 0012) the better expected score of two candidates
    /// ([`suit_expected_score`], [`null_expected_score`]):
    ///
    /// - the suit game or grand the declarer heuristic picks for the twelve
    ///   cards ([`DeclarerHeuristic::choose_game_after_skat`]), discarding
    ///   for it;
    /// - Null, discarding the two most exposed cards, as Null Ouvert if no
    ///   exposed card is left.
    ///
    /// With "Der Angeber" only Null Ouvert can be played open after the
    /// skat.
    fn after_skat(run: &Run, content: &Content) -> Option<(Declaration, [DealtCard; 2])> {
        let active = run.game()?;
        let game = &active.game;
        let cards = game.declarer_cards();
        let deck = game.deck_cards();
        let null_discard = Self::discard_for(cards, Declaration::null(NullRisk::Normal), deck)?;
        let null_hand: Vec<DealtCard> = cards
            .iter()
            .copied()
            .filter(|c| !null_discard.contains(c))
            .collect();
        let rest: Vec<DealtCard> = deck
            .iter()
            .copied()
            .filter(|c| !null_discard.contains(c))
            .collect();
        let exposed = null_unsafe_cards(&null_hand, &rest);
        let null = if exposed == 0 || active.rules.require_ouvert {
            Declaration::null(NullRisk::Ouvert)
        } else {
            Declaration::null(NullRisk::Normal)
        };
        if active.rules.require_ouvert {
            return Some((null, null_discard));
        }
        let suit = DeclarerHeuristic.choose_game_after_skat(cards);
        let suit_discard = Self::discard_for(cards, suit, deck)?;
        if !Self::null_counts(run, content, Some(null_discard)) {
            return Some((suit, suit_discard));
        }
        let suit_hand: Vec<DealtCard> = cards
            .iter()
            .copied()
            .filter(|c| !suit_discard.contains(c))
            .collect();
        let scoring = run.null_scoring(content);
        let null_ev = null_expected_score(&null_hand, exposed, null, &scoring);
        let suit_ev = suit_expected_score(&suit_hand, cards, deck, suit);
        if null_ev > suit_ev {
            Some((null, null_discard))
        } else {
            Some((suit, suit_discard))
        }
    }

    /// Whether a won Null would count anything here: declare Null on a copy
    /// of the run (after `discard` while discarding) and read the value
    /// preview, which holds every hook certain for a win. "Der Schwarze
    /// Peter" (a won game counts only with Schneider) makes it 0. If the
    /// probe cannot declare Null, it counts.
    fn null_counts(run: &Run, content: &Content, discard: Option<[DealtCard; 2]>) -> bool {
        let Some(active) = run.game() else {
            return true;
        };
        let ouvert = active.rules.require_ouvert;
        let mut probe = run.clone();
        let risk = match active.game.phase() {
            Phase::Declaring if ouvert => NullRisk::OuvertHand,
            Phase::Declaring => NullRisk::Hand,
            Phase::Discarding => {
                let Some([a, b]) = discard else {
                    return true;
                };
                if probe
                    .apply(content, RunAction::Discard([a.id, b.id]))
                    .is_err()
                {
                    return true;
                }
                if ouvert {
                    NullRisk::Ouvert
                } else {
                    NullRisk::Normal
                }
            }
            Phase::DeclaringAfterSkat if ouvert => NullRisk::Ouvert,
            Phase::DeclaringAfterSkat => NullRisk::Normal,
            Phase::Playing | Phase::Finished(_) => return true,
        };
        if probe
            .apply(content, RunAction::Declare(Declaration::null(risk)))
            .is_err()
        {
            return true;
        }
        probe
            .value_preview(content)
            .is_none_or(|p| p.game_value > 0)
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
        // A jack or an ace from the skat only hurts a Null.
        if trump == GameType::Null {
            return None;
        }
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

    /// Schnäpse right before the declaration after the skat, see the
    /// module docs: "Kreide" on the cheapest side card of a suit game,
    /// then "Doppelkorn" (not weak) or "Magenbitter" (weak).
    fn schnapps_before_declaring(
        run: &Run,
        content: &Content,
        declaration: Declaration,
    ) -> Option<RunAction> {
        if declaration.is_null() {
            return None;
        }
        let hand = run.game()?.game.hand(Seat::DECLARER);
        if let GameType::Suit(trump) = declaration.game() {
            let side = hand
                .iter()
                .copied()
                .filter(|&c| !crate::rules::is_trump(declaration.game(), c))
                .filter(|c| c.face().is_some_and(|f| f.rank != Rank::Jack))
                .min_by_key(|c| (c.card_points(), c.face().map(|f| f.rank), c.id));
            if let Some(card) = side {
                let chalk = run
                    .consumables()
                    .iter()
                    .enumerate()
                    .filter(|(_, id)| {
                        content
                            .consumable(id)
                            .is_some_and(|d| d.action == ConsumableAction::ChalkCard)
                    })
                    .map(|(slot, _)| RunAction::UseConsumable {
                        slot,
                        target: Some(card.id),
                        suit: Some(trump),
                    })
                    .find(|action| run.accepts(content, action.clone()));
                if chalk.is_some() {
                    return chalk;
                }
            }
        }
        // One "Doppelkorn" or "Magenbitter" per game.
        if !run.pending_next_game().is_empty() {
            return None;
        }
        let wanted = if RunBot::is_weak(hand, declaration) {
            NextGameEffect::SingleLoss
        } else {
            NextGameEffect::Multiply(2)
        };
        Self::usable(
            run,
            content,
            |a| match (a, wanted) {
                (
                    ConsumableAction::NextGame(NextGameEffect::Multiply(_)),
                    NextGameEffect::Multiply(_),
                ) => true,
                (ConsumableAction::NextGame(e), w) => *e == w,
                _ => false,
            },
            None,
        )
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
        let mut run = open_shop(&content, 6);
        run.relics = ["kontra", "ouvert_bonus", "tour", "virgin", "concede"]
            .iter()
            .map(|r| RelicId::new(r))
            .collect();
        offer(&mut run, &["bock_round"], &[]);
        // Bock costs 9; selling Ouvert-Bonus brings 2: 8 is not enough.
        assert_eq!(RunBot.shop_action(&run, &content), RunAction::LeaveShop);
        run.coins = 7;
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
                target: Some(kreuz_ass),
                suit: None
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
            let game = &run.game().expect("game").game;
            let hand = game.hand(Seat::DECLARER);
            if RunBot::is_weak(hand, DeclarerHeuristic.choose_declaration(hand))
                && null_unsafe_cards(hand, game.deck_cards()) > NULL_MAX_EXPOSED
            {
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
                target: None,
                suit: None
            }
        );
        run.consumables.pop();
        assert_eq!(next(&run), RunAction::Pass);
        run.series.passes_left = 0;
        assert_eq!(
            next(&run),
            RunAction::UseConsumable {
                slot: 0,
                target: None,
                suit: None
            }
        );
        // The last game is played, whatever the hand: with the skat.
        run.series.games_played = run.series.game_limit - 1;
        assert_eq!(next(&run), RunAction::PickUpSkat);
    }

    /// Hand only for a very strong hand; otherwise pick up the skat, then
    /// discard and declare the game chosen for all twelve cards: the same
    /// choice before and after the discard. Both characters play suit
    /// games, grand and Null (ADR 0012); Null after the skat is Ouvert
    /// exactly when no exposed card is left.
    #[test]
    fn the_bot_sees_the_skat_before_it_chooses_the_game() {
        let content = Content::load().expect("content");
        for character in crate::content::Character::ALL {
            let start = super::super::RunStart {
                character,
                ..super::super::RunStart::default()
            };
            let (mut hand_games, mut skat_games, mut nulls) = (0, 0, 0);
            for seed in 0..80 {
                let (mut run, _) = Run::new_with(&content, seed, &start).expect("start");
                let next = |run: &Run| RunBot.next_action(run, &content).expect("action");
                match next(&run) {
                    RunAction::Declare(d) => {
                        assert!(d.hand(), "seed {seed}: {d:?} before the skat");
                        hand_games += 1;
                    }
                    RunAction::PickUpSkat => {
                        run.apply(&content, RunAction::PickUpSkat).expect("pick up");
                        let (planned, discard) =
                            RunBot::after_skat(&run, &content).expect("choice");
                        let ids = [discard[0].id, discard[1].id];
                        assert_eq!(next(&run), RunAction::Discard(ids), "seed {seed}");
                        run.apply(&content, RunAction::Discard(ids))
                            .expect("discard");
                        assert_eq!(next(&run), RunAction::Declare(planned), "seed {seed}");
                        assert!(!planned.hand());
                        if planned.is_null() {
                            let game = &run.game().expect("game").game;
                            let skat = game.skat();
                            let rest: Vec<DealtCard> = game
                                .deck_cards()
                                .iter()
                                .copied()
                                .filter(|c| !skat.contains(c))
                                .collect();
                            let exposed = null_unsafe_cards(game.hand(Seat::DECLARER), &rest);
                            assert_eq!(planned.ouvert(), exposed == 0, "seed {seed}");
                            nulls += 1;
                        }
                        run.apply(&content, RunAction::Declare(planned))
                            .expect("declare");
                        skat_games += 1;
                    }
                    _ => {} // thrown in
                }
            }
            assert!(
                skat_games > 10 && hand_games < skat_games && nulls > 0 && nulls < skat_games,
                "{character}: {hand_games} {skat_games} {nulls}"
            );
        }
    }

    /// The expected scores behind the choice: Null with one exposed card
    /// and three sevens beats a suit game with five trumps; a suit game
    /// with seven trumps beats Null with three exposed cards.
    #[test]
    fn expected_scores_compare_null_with_suit_games() {
        let formula = NullScoring::Formula(crate::rules::NullFormula {
            base_value: 23,
            per_seven_ducked: 1,
            never_followed: 2,
            short_circuit: 3,
            short_circuit_tricks: 5,
            short_circuit_points: 61,
        });
        let null = Declaration::null(NullRisk::Normal);
        let sevens = hand("C7 S7 H7 C8 S8 H9 D8 D9 DT DQ");
        // 76 % of 23 × 4.3, minus 24 % of 46.
        assert_eq!(null_expected_score(&sevens, 1, null, &formula), 6412);
        assert!(null_expected_score(&sevens, 4, null, &formula) < 0);
        // Fixed: 76 % of 23, minus 24 % of 46.
        assert_eq!(
            null_expected_score(&sevens, 1, null, &NullScoring::Fixed),
            644
        );
        let hearts = suit(Suit::Hearts);
        let deck: Vec<DealtCard> = crate::deck::Deck::standard()
            .cards()
            .iter()
            .filter_map(|c| match c.kind {
                CardKind::Normal(card) => Some(DealtCard::plain(c.id, CardKind::Normal(card))),
                CardKind::Blank => None,
            })
            .collect();
        let five = hand("HJ HA HT H9 H8 SA S9 C8 D7 D8");
        let seven = hand("CJ HJ HA HT HK H9 H8 SA S9 C8");
        // Five trumps and two aces, ohne 2: 52 % of 10 × 3, minus 48 % of 60.
        assert_eq!(suit_expected_score(&five, &five, &deck, hearts), -1320);
        assert!(suit_expected_score(&seven, &seven, &deck, hearts) > 0);
        assert!(
            null_expected_score(&sevens, 1, null, &formula)
                > suit_expected_score(&five, &five, &deck, hearts)
        );
        assert!(
            suit_expected_score(&seven, &seven, &deck, hearts)
                > null_expected_score(&sevens, 3, null, &formula)
        );
    }

    /// "Der Schieber": Null Hand only with at most two exposed cards.
    #[test]
    fn forced_hand_takes_null_only_with_few_exposed_cards() {
        let content = Content::load().expect("content");
        for character in crate::content::Character::ALL {
            let start = super::super::RunStart {
                character,
                ..super::super::RunStart::default()
            };
            for seed in 0..80 {
                let (mut run, _) = Run::new_with(&content, seed, &start).expect("start");
                run.game.as_mut().expect("game").rules.require_hand = true;
                let game = &run.game().expect("game").game;
                let exposed = null_unsafe_cards(game.hand(Seat::DECLARER), game.deck_cards());
                if let Some(RunAction::Declare(d)) = RunBot.next_action(&run, &content) {
                    assert!(d.hand(), "seed {seed}");
                    if d.is_null() {
                        assert!(exposed <= NULL_HAND_MAX_EXPOSED, "seed {seed}: {exposed}");
                    }
                }
            }
        }
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

//! The bot at the table: Einmischen, hand or skat, the choice of game,
//! announcements, discarding, Schnäpse before declaring, the card play
//! with "Spiegel" and "Untot".

use crate::ai::{DeclarerHeuristic, Policy, null_unsafe_cards, trump_count};
use crate::cards::{Card, PlayingCard, Rank, Suit};
use crate::content::{ConsumableAction, Content, NextGameEffect};
use crate::deck::DealtCard;
use crate::game::{Phase, Seat};
use crate::rules::{Declaration, GameType, NullRisk, RiskLevel};

use super::{
    BOLD_MIN_PERCENT, NULL_HAND_MAX_EXPOSED, NULL_MAX_EXPOSED, Run, RunAction, RunBot, holds_relic,
    null_expected_score, ramsch_expected_score, schneider_percent, suit_expected_score,
    suit_win_percent, ten_card_score,
};

/// Optimal-stopping values of [`ten_card_score`] over random deals of the
/// standard deck (hundredths; 200 000 deals, Grand 18, Null formula of the
/// Alleinspieler): index `k − 1` is the expected best of `k` deals taken
/// with this rule, so a hand with `k` Einmischen left is thrown in below
/// index `k − 1` (`redeal_probe` test prints them).
pub const REDEAL_STOP: [i64; 5] = [877, 1759, 2355, 2824, 3203];

/// "Kahlschlag": +1 level per void side suit after the skat, at most this
/// many ([`RunBot::void_discard`]).
const CLEAR_CUT_MAX_VOIDS: usize = 2;

/// How much a Ramsch must beat the other choices before "Die
/// Ramschtante" declares it ([`RunBot::ramsch_choice`], hundredths): the
/// other choices leave the skat out, so they are worth a little more than
/// estimated. Set by measurement (docs/balance/2026-11-10-neue-charaktere.md).
pub const RAMSCH_MARGIN: i64 = 5000;

/// "Der Zocker" keeps his one pickup of the skat in a pub for a later
/// game when the ten cards win a hand game at least this often
/// ([`RunBot::keep_the_skat`], percent). Set by measurement
/// (docs/balance/2026-11-10-neue-charaktere.md).
pub const HAND_KEEP_MIN_PERCENT: i64 = 75;

/// The same for a Null Hand: at most this many exposed cards
/// ([`RunBot::keep_the_skat`]; Null wins 76 % with one, 54 % with two).
pub const HAND_KEEP_NULL_EXPOSED: usize = 1;

/// Relic ids the game decisions look at (relics v4).
const CLEAR_CUT: &str = "clear_cut";

/// The ten cards as they play once declared: an open Wandelkarte takes the
/// suit it will settle on ([`crate::rules::settled_wild_suit`], a jack
/// keeps its own), so the bot counts it in its choice of game.
fn settled_hand(hand: &[DealtCard]) -> Vec<DealtCard> {
    if !hand.iter().any(|c| c.wild()) {
        return hand.to_vec();
    }
    let suit = crate::rules::settled_wild_suit(hand);
    hand.iter()
        .map(|&c| {
            if !c.wild() {
                return c;
            }
            match c.face() {
                Some(face) if face.rank == Rank::Jack => c.settle_wild(face.suit),
                _ => c.settle_wild(suit),
            }
        })
        .collect()
}

/// "Große Klappe": a won game with Schneider announced counts the Grundwert
/// twice.
const BIG_MOUTH: &str = "big_mouth";

/// "Offenes Visier": Ouvert asks for Schneider only.
const OPEN_VISOR: &str = "open_visor";

pub(super) fn count_rank<C: PlayingCard>(hand: &[C], rank: Rank) -> usize {
    // A demoted Unter ("Abgestuft") is no Unter.
    hand.iter()
        .filter(|c| c.face().is_some_and(|f| f.rank == rank))
        .filter(|c| !crate::rules::is_demoted_jack(**c))
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

    pub(super) fn game_action(&self, run: &Run, content: &Content) -> Option<RunAction> {
        let active = run.game()?;
        let game = &active.game;
        let declarer = DeclarerHeuristic;
        if active.awaiting_re {
            let declaration = game.declaration()?;
            let re = RunBot::says_re(game.hand(Seat::DECLARER), declaration.game());
            return Some(RunAction::AnswerKontra { re });
        }
        if let Some(cards) = Self::rise_cards(run) {
            return Some(RunAction::ChooseRiseCards(cards));
        }
        match game.phase() {
            Phase::Declaring => {
                if let Some(rescue) = Self::rescue_schnapps(run, content) {
                    return Some(rescue);
                }
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
                let (_, discard) = Self::after_skat(run, content)?;
                Some(Self::discard_action(&discard)?)
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
                if let Some(card) = Self::undead_card(run, content) {
                    return Some(RunAction::Play(card.id));
                }
                if let Some(card) = Self::mirror_card(run, content) {
                    return Some(RunAction::Play(card.id));
                }
                // A Wandelkarte has its settled suit by now: a card like
                // any other. A Doppelgänger waits for its mirror.
                let mut view = game.view(Seat::DECLARER)?;
                Self::keep_for_mirror(run, content, &mut view.legal_moves);
                let chosen = declarer.choose(&view)?;
                Some(RunAction::Play(chosen.id))
            }
            Phase::Finished(_) => None,
        }
    }

    /// The three cards the bot offers for a rising "Untot" card
    /// ([`Run::rise_pending`]; simple rule, not tuned): the weakest of his
    /// hand, fewest Augen first, then the lowest strength in the game, then
    /// the id. `None` if nothing waits.
    #[must_use]
    pub fn rise_cards(run: &Run) -> Option<[u32; 3]> {
        run.rise_pending()?;
        let game = &run.game()?.game;
        let declaration = game.declaration()?;
        let mut hand: Vec<DealtCard> = game.hand(Seat::DECLARER).to_vec();
        hand.sort_by_key(|&c| {
            (
                c.signed_points(),
                crate::ai::solver::card_strength(declaration.game(), c),
                c.id,
            )
        });
        match hand.as_slice() {
            [a, b, c, ..] => Some([a.id, b.id, c.id]),
            _ => None,
        }
    }

    /// An "Untot" card the bot plays now (simple rule, not tuned): playing
    /// last to the trick, a card of the hand that may rise
    /// ([`Run::can_rise`]), is legal and takes the trick. It comes back,
    /// and the substitute's Augen go to the player. `None` otherwise.
    #[must_use]
    pub fn undead_card(run: &Run, content: &Content) -> Option<DealtCard> {
        let active = run.game()?;
        let game = &active.game;
        let declaration = game.declaration()?;
        let trick = game.current_trick();
        if trick.len() != 2 || active.awaiting_re {
            return None;
        }
        game.legal_moves(Seat::DECLARER)
            .into_iter()
            .filter(|&c| run.can_rise(content, c))
            .find(|&c| {
                let mut cards = trick.to_vec();
                cards.push(c);
                crate::rules::winning_index(declaration.game(), &cards) == Some(2)
            })
    }

    /// The least Augen a trick must be worth with the copy before the bot
    /// mirrors a "Doppelgänger" into it ([`RunBot::mirror_card`]; simple
    /// rule, not tuned): a Daus or a Zehn in the trick, copied.
    pub const MIRROR_MIN_AUGEN: i64 = 15;

    /// A "Doppelgänger" the bot plays now (simple rule, not tuned): playing
    /// last to the trick, a card of the hand that mirrors
    /// ([`Run::mirrors_now`]) when the trick with the copy is worth at
    /// least [`RunBot::MIRROR_MIN_AUGEN`] Augen, or when it is the only
    /// legal card. `None` otherwise.
    #[must_use]
    pub fn mirror_card(run: &Run, content: &Content) -> Option<DealtCard> {
        let active = run.game()?;
        let game = &active.game;
        let declaration = game.declaration()?;
        let trick = game.current_trick();
        if trick.len() != 2 || active.awaiting_re {
            return None;
        }
        let legal = game.legal_moves(Seat::DECLARER);
        let card = legal
            .iter()
            .copied()
            .find(|&c| run.mirrors_now(content, c))?;
        if legal.len() == 1 {
            return Some(card);
        }
        let copied = trick[crate::rules::winning_index(declaration.game(), trick)?];
        let worth: i64 =
            trick.iter().map(|c| c.signed_points()).sum::<i64>() + copied.signed_points();
        (worth >= Self::MIRROR_MIN_AUGEN).then_some(card)
    }

    /// Keep a "Doppelgänger" whose mirror is still to come
    /// ([`Run::can_mirror`]) out of `legal` while another card may be
    /// played: the bot plays it only through [`RunBot::mirror_card`] or
    /// when nothing else is legal. The search knows nothing of the mirror.
    pub fn keep_for_mirror(run: &Run, content: &Content, legal: &mut Vec<DealtCard>) {
        if legal.iter().any(|&c| !run.can_mirror(content, c)) {
            legal.retain(|&c| !run.can_mirror(content, c));
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
    /// - Use one of the game's own Einmischen on a mediocre hand
    ///   ([`RunBot::redeal_threshold`], since 05.11.2026).
    /// - Otherwise pick up the skat; the game is chosen with twelve cards
    ///   ([`RunBot::after_skat`]).
    ///
    /// "Der Schieber" (hand games only) and "Der Angeber" (always open)
    /// narrow the choice, see [`RunBot::forced_hand`].
    fn before_skat(run: &Run, content: &Content) -> Option<RunAction> {
        let active = run.game()?;
        let game = &active.game;
        let settled = settled_hand(game.hand(Seat::DECLARER));
        let hand = settled.as_slice();
        let deck = game.deck_cards();
        let exposed = null_unsafe_cards(hand, deck);
        let suit = DeclarerHeuristic.choose_declaration(hand);
        let suit = suit.with_risk(RunBot::announcement(hand, suit));
        let rules = &active.rules;
        if let Some(action) = Self::ramsch_choice(run, content, hand, deck) {
            return Some(action);
        }
        if rules.require_hand || run.skat_used_up() {
            return Self::forced_hand(run, content);
        }
        // "Fusel" on a hand that is not weak, one next-game Schnaps per game.
        if !rules.require_ouvert
            && run.pending_next_game().is_empty()
            && !RunBot::is_weak(hand, suit)
        {
            if let Some(action) = Self::usable(
                run,
                content,
                |a| matches!(a, ConsumableAction::NextGameCursed(_)),
                None,
            ) {
                return Some(action);
            }
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
        if !rules.require_ouvert && Self::keep_the_skat(run, hand, suit, exposed) {
            return Self::forced_hand(run, content);
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
        if Self::redeal_mediocre(run, content, hand, deck, null_counts) {
            return Some(RunAction::Redeal);
        }
        Some(RunAction::PickUpSkat)
    }

    /// "Die Ramschtante" ([`crate::content::Effect::OfferRamsch`]): a
    /// Ramsch when its expected score ([`ramsch_expected_score`]) beats by
    /// [`RAMSCH_MARGIN`] both the best game of the ten cards
    /// ([`ten_card_score`], which leaves the skat out) and, with an
    /// Einmischen of the game left, what a new deal is worth
    /// ([`RunBot::redeal_threshold`]). `None` where no Ramsch is offered.
    fn ramsch_choice(
        run: &Run,
        content: &Content,
        hand: &[DealtCard],
        deck: &[DealtCard],
    ) -> Option<RunAction> {
        let action = RunAction::Declare(Declaration::Ramsch);
        if !run.ramsch_offered() || !run.accepts(content, action.clone()) {
            return None;
        }
        let ramsch = ramsch_expected_score(hand);
        let scoring = run.null_scoring(content);
        let grand = content.run.base_value(GameType::Grand).unwrap_or(0);
        let null_counts = Self::null_counts(run, content, None);
        let game = ten_card_score(hand, deck, grand, null_counts.then_some(&scoring));
        let left = run.redeals_left(content);
        let redeal = if left > 0 && run.accepts(content, RunAction::Redeal) {
            Self::redeal_threshold(left)
        } else {
            i64::MIN
        };
        (ramsch > game.max(redeal).saturating_add(RAMSCH_MARGIN)).then_some(action)
    }

    /// "Der Zocker" ([`crate::content::Effect::SkatPickupsPerPub`]): while
    /// the skat may still be picked up but not for every game left, a hand
    /// that wins a hand game at least [`HAND_KEEP_MIN_PERCENT`] of the time
    /// ([`suit_win_percent`]), or a Null Hand with at most
    /// [`HAND_KEEP_NULL_EXPOSED`] exposed cards, is played from
    /// the hand ([`RunBot::forced_hand`]) and the pickup kept for a weak
    /// one. In the last game of the pub nothing is kept.
    fn keep_the_skat(run: &Run, hand: &[DealtCard], suit: Declaration, exposed: usize) -> bool {
        let Some(left) = run.skat_pickups_left() else {
            return false;
        };
        let deck = run.game().map_or(&[][..], |a| a.game.deck_cards());
        left > 0
            && left < run.series().games_left()
            && (suit_win_percent(hand, deck, suit.game()) >= HAND_KEEP_MIN_PERCENT
                || exposed <= HAND_KEEP_NULL_EXPOSED)
    }

    /// Whether to spend one of the game's own Einmischen on a hand that is
    /// not weak but mediocre: its [`ten_card_score`] is below
    /// [`RunBot::redeal_threshold`] for the Einmischen left. Only real
    /// Einmischen, no Schnaps.
    fn redeal_mediocre(
        run: &Run,
        content: &Content,
        hand: &[DealtCard],
        deck: &[DealtCard],
        null_counts: bool,
    ) -> bool {
        let left = run.redeals_left(content);
        if left == 0 || !run.accepts(content, RunAction::Redeal) {
            return false;
        }
        let scoring = run.null_scoring(content);
        let grand = content.run.base_value(GameType::Grand).unwrap_or(0);
        let score = ten_card_score(hand, deck, grand, null_counts.then_some(&scoring));
        score < Self::redeal_threshold(left)
    }

    /// The [`ten_card_score`] (hundredths) below which the bot throws a
    /// deal in with `left` Einmischen: the optimal stopping rule on the
    /// score of a random deal ([`REDEAL_STOP`]: with `k` left, redeal when
    /// the hand is worth less than the expected best of `k − 1` more
    /// deals). Measurement tool: it stands in for a sensible human, who
    /// throws in mediocre hands while Einmischen are left (round 2 found
    /// the bot used the third one rarely).
    ///
    /// Measured on 05.11.2026 (curve of the first pub, 1000 pubs, seed 42,
    /// two Einmischen): the first pub at Zielwert 35 / 55 / 90 / 145 went
    /// from 83,2 / 73,4 / 56,7 / 30,9 % (only weak hands thrown in) to
    /// 85,3 / 77,4 / 62,9 / 36,9 %. The thresholds × 0,75, × 1,25, × 1,5
    /// and × 2 were equal or worse, also at high Zielwerte, and keeping
    /// every hand with a table win chance of 70, 80 or 90 % or more was
    /// worse too; so the rule does not look at the pub's Zielwert (which
    /// also keeps `--curve`, played without one, equal to the runs).
    #[must_use]
    pub fn redeal_threshold(left: u32) -> i64 {
        let index = usize::try_from(left.saturating_sub(1)).unwrap_or(usize::MAX);
        REDEAL_STOP
            .get(index)
            .or(REDEAL_STOP.last())
            .copied()
            .unwrap_or(0)
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
        let settled = settled_hand(game.hand(Seat::DECLARER));
        let hand = settled.as_slice();
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
        let suit_ev = suit_expected_score(
            hand,
            hand,
            game.deck_cards(),
            suit,
            content.run.base_value(suit.game()).unwrap_or(0),
            active.rules.thresholds.win,
        );
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
    /// The suit game or grand carries the announcement the ten cards kept
    /// allow ([`RunBot::announcement`], ADR 0013: also after the skat,
    /// without Hand). With "Der Angeber" it is played only if that is
    /// Ouvert, otherwise Null Ouvert.
    pub(super) fn after_skat(
        run: &Run,
        content: &Content,
    ) -> Option<(Declaration, Vec<DealtCard>)> {
        let active = run.game()?;
        let game = &active.game;
        let cards = game.declarer_cards();
        let deck = game.deck_cards();
        let count = game.discard_count().max(2);
        let null_discard =
            Self::discard_n(cards, Declaration::null(NullRisk::Normal), deck, count)?;
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
        let suit = DeclarerHeuristic.choose_game_after_skat(cards);
        let voids = holds_relic(run, CLEAR_CUT) && matches!(suit.game(), GameType::Suit(_));
        let suit_discard = if voids {
            Self::void_discard_n(cards, suit, deck, count)?
        } else {
            Self::discard_n(cards, suit, deck, count)?
        };
        let suit_hand: Vec<DealtCard> = cards
            .iter()
            .copied()
            .filter(|c| !suit_discard.contains(c))
            .collect();
        let suit = suit
            .with_risk(RunBot::announcement(&suit_hand, suit))
            .with_hand(false);
        let suit = Self::bold_announcement(run, suit, &suit_hand, cards, deck);
        if active.rules.require_ouvert {
            if suit.ouvert() {
                return Some((suit, suit_discard));
            }
            return Some((null, null_discard));
        }
        if !Self::null_counts(run, content, Some(&null_discard)) {
            return Some((suit, suit_discard));
        }
        let scoring = run.null_scoring(content);
        let null_ev = null_expected_score(&null_hand, exposed, null, &scoring);
        let base = content.run.base_value(suit.game()).unwrap_or(0);
        let suit_ev = suit_expected_score(
            &suit_hand,
            cards,
            deck,
            suit,
            base,
            active.rules.thresholds.win,
        );
        if null_ev > suit_ev {
            Some((null, null_discard))
        } else {
            Some((suit, suit_discard))
        }
    }

    /// Schneider angesagt or Ouvert after the skat for a relic that pays
    /// for it (relics v4, 07.11.2026): with "Große Klappe" (`big_mouth`, a
    /// won game with Schneider announced counts the Grundwert twice) and
    /// "Offenes Visier" (`open_visor`, Ouvert asks for Schneider only).
    /// Without either relic, for a weak hand ([`RunBot::is_weak`]) or a
    /// Null, `suit` is returned unchanged.
    ///
    /// The chance of Schneider is [`schneider_percent`]: the unseen cards
    /// dealt [`BOLD_SAMPLES`](super::BOLD_SAMPLES) times, each deal solved double-dummy. Below
    /// [`BOLD_MIN_PERCENT`] nothing is announced. Above it the bot takes the
    /// best expected Gewinnstufen (hundredths) of a simple model that
    /// counts the game as won whenever Schneider is not announced: plain
    /// `Spitzen + 1` plus the chance of Schneider; announced `k` levels
    /// (Spitzen, Spiel, Schneider, angesagt; Ouvert one more) won with
    /// chance `p`, doubled with "Große Klappe", lost `k` otherwise. With
    /// "Der Angeber" (always open) only Ouvert counts.
    fn bold_announcement(
        run: &Run,
        suit: Declaration,
        kept: &[DealtCard],
        cards: &[DealtCard],
        deck: &[DealtCard],
    ) -> Declaration {
        let Some(active) = run.game() else {
            return suit;
        };
        let rules = &active.rules;
        let big_mouth = holds_relic(run, BIG_MOUTH);
        let visor = holds_relic(run, OPEN_VISOR) && rules.thresholds.ouvert_schneider;
        let risk = suit.risk().unwrap_or(RiskLevel::Normal);
        if suit.is_null()
            || !(big_mouth || visor)
            || risk >= RiskLevel::Ouvert
            || !Self::may_make_schneider(kept, suit.game())
        {
            return suit;
        }
        let p = schneider_percent(
            suit.game(),
            kept,
            cards,
            deck,
            active.game.forehand(),
            rules.thresholds.schneider_limit,
            run.seed(),
        );
        if p < BOLD_MIN_PERCENT {
            return suit;
        }
        if rules.require_ouvert {
            return if visor {
                suit.with_risk(RiskLevel::Ouvert).with_hand(false)
            } else {
                suit
            };
        }
        let faces: Vec<Card> = cards.iter().filter_map(|c| c.face()).collect();
        let spitzen = i64::from(crate::rules::matadors(suit.game(), &faces).count);
        let double = if big_mouth { 2 } else { 1 };
        let announced = |levels: i64| p * levels * double - (100 - p) * levels;
        let mut best = (
            match risk {
                RiskLevel::SchneiderDeclared => announced(spitzen + 3),
                _ => 100 * (spitzen + 1) + p,
            },
            risk,
        );
        if big_mouth && risk < RiskLevel::SchneiderDeclared {
            let ev = announced(spitzen + 3);
            if ev > best.0 {
                best = (ev, RiskLevel::SchneiderDeclared);
            }
        }
        if visor {
            let ev = announced(spitzen + 4);
            if ev > best.0 {
                best = (ev, RiskLevel::Ouvert);
            }
        }
        if best.1 == risk {
            suit
        } else {
            suit.with_risk(best.1).with_hand(false)
        }
    }

    /// Cheap filter before [`schneider_percent`]: a suit game with six
    /// trumps or more, a grand with two jacks or more, and at most two side
    /// cards that may lose a trick.
    fn may_make_schneider(kept: &[DealtCard], game: GameType) -> bool {
        let strong = match game {
            GameType::Grand => count_rank(kept, Rank::Jack) >= 2,
            GameType::Suit(_) => trump_count(game, kept) >= 6,
            GameType::Null => false,
        };
        strong && side_losers(kept, game) <= 2
    }

    /// `count` discards for a suit game with "Kahlschlag" (`clear_cut`, +1
    /// level per side suit without a card after the skat, at most +2): the
    /// pair of [`RunBot::void_discard`], then for a third discard ("Drei
    /// drücken") the plain rule of [`RunBot::discard_n`].
    fn void_discard_n(
        cards: &[DealtCard],
        declaration: Declaration,
        deck: &[DealtCard],
        count: usize,
    ) -> Option<Vec<DealtCard>> {
        let pair = Self::discard(cards, |free| Self::void_discard(free, declaration, deck))?;
        let mut chosen = pair.to_vec();
        if count > 2 {
            let left: Vec<DealtCard> = cards
                .iter()
                .copied()
                .filter(|c| !chosen.contains(c))
                .collect();
            chosen.extend(Self::discard_n(&left, declaration, deck, count - 2)?);
        }
        Some(chosen)
    }

    /// The two discards that leave the most side suits void (counted up to
    /// the two "Kahlschlag" pays for) in a suit game: the plain choice of
    /// [`RunBot::discard_for`] if it leaves as many, else among the pairs
    /// without a trump that do, the one with the most Augen (they count for
    /// the declarer in the skat), the first such pair in hand order on a
    /// tie. Other games: the plain choice.
    pub(super) fn void_discard(
        cards: &[DealtCard],
        declaration: Declaration,
        deck: &[DealtCard],
    ) -> Option<[DealtCard; 2]> {
        let plain = Self::discard_for(cards, declaration, deck)?;
        let GameType::Suit(trump) = declaration.game() else {
            return Some(plain);
        };
        let game = declaration.game();
        let voids_without = |pair: [DealtCard; 2]| -> usize {
            Suit::ALL
                .iter()
                .filter(|&&suit| suit != trump)
                .filter(|&&suit| {
                    !cards.iter().any(|c| {
                        !pair.contains(c)
                            && !crate::rules::is_trump(game, *c)
                            && c.face().is_some_and(|f| f.suit == suit)
                    })
                })
                .count()
                .min(CLEAR_CUT_MAX_VOIDS)
        };
        let plain_voids = voids_without(plain);
        let side: Vec<DealtCard> = cards
            .iter()
            .copied()
            .filter(|&c| !crate::rules::is_trump(game, c))
            .collect();
        let mut best: Option<(usize, u32, [DealtCard; 2])> = None;
        for (i, &a) in side.iter().enumerate() {
            for &b in &side[i + 1..] {
                let pair = [a, b];
                let key = (voids_without(pair), a.card_points() + b.card_points());
                if best.is_none_or(|(v, p, _)| key > (v, p)) {
                    best = Some((key.0, key.1, pair));
                }
            }
        }
        match best {
            Some((voids, _, pair)) if voids > plain_voids => Some(pair),
            _ => Some(plain),
        }
    }

    /// Whether a won Null would count anything here: declare Null on a copy
    /// of the run (after `discard` while discarding) and read the value
    /// preview, which holds every hook certain for a win. A factor 0 on
    /// the Spielwert makes it 0 ("Der Schwarze Peter" only halves it since
    /// 02.11.2026). If the probe cannot declare Null, it counts.
    fn null_counts(run: &Run, content: &Content, discard: Option<&[DealtCard]>) -> bool {
        let Some(active) = run.game() else {
            return true;
        };
        let ouvert = active.rules.require_ouvert;
        let mut probe = run.clone();
        let risk = match active.game.phase() {
            Phase::Declaring if ouvert => NullRisk::OuvertHand,
            Phase::Declaring => NullRisk::Hand,
            Phase::Discarding => {
                let Some(action) = discard.and_then(Self::discard_action) else {
                    return true;
                };
                if probe.apply(content, action).is_err() {
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
            .apply_step(content, RunAction::Declare(Declaration::null(risk)))
            .is_err()
        {
            return true;
        }
        probe
            .value_preview(content)
            .is_none_or(|p| p.game_value > 0)
    }

    /// The action that discards `cards`: two, or three after drawing a card
    /// with the skat ("Drei drücken").
    fn discard_action(cards: &[DealtCard]) -> Option<RunAction> {
        match *cards {
            [a, b] => Some(RunAction::Discard([a.id, b.id])),
            [a, b, c] => Some(RunAction::DiscardThree([a.id, b.id, c.id])),
            _ => None,
        }
    }

    /// `count` discards for `declaration` (two, or three with "Drei
    /// drücken"): the two of [`RunBot::discard_for`], then for a third the
    /// first of the two it would discard from the cards left.
    fn discard_n(
        cards: &[DealtCard],
        declaration: Declaration,
        deck: &[DealtCard],
        count: usize,
    ) -> Option<Vec<DealtCard>> {
        let mut chosen: Vec<DealtCard> = Vec::with_capacity(count);
        let mut left = cards.to_vec();
        while chosen.len() < count {
            let pair = Self::discard_for(&left, declaration, deck)?;
            let take = (count - chosen.len()).min(2);
            for card in pair.into_iter().take(take) {
                left.retain(|&c| c != card);
                chosen.push(card);
            }
        }
        Some(chosen)
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
    /// module docs: "Blue Curaçao" on the cheapest side card of a suit game,
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

    /// The rescue Schnäpse between two games: "Klarer" with the account
    /// below 0, "Absacker" before the last game of a pub short of its
    /// target, "Eierlikör" whenever the run accepts it (account below half
    /// the target). The run refuses each when it would do nothing, so none
    /// of them is drunk twice for the same reason.
    fn rescue_schnapps(run: &Run, content: &Content) -> Option<RunAction> {
        let series = run.series();
        if series.score < 0 {
            if let Some(action) = Self::usable(
                run,
                content,
                |a| matches!(a, ConsumableAction::ClearAccount),
                None,
            ) {
                return Some(action);
            }
        }
        if series.games_left() == 1 && !series.target_reached() {
            if let Some(action) = Self::usable(
                run,
                content,
                |a| matches!(a, ConsumableAction::ExtraGames(_)),
                None,
            ) {
                return Some(action);
            }
        }
        Self::usable(
            run,
            content,
            |a| matches!(a, ConsumableAction::NextGameIfBehind(_)),
            None,
        )
    }

    /// The cheapest way to throw a weak hand in, if any: an Einmischen
    /// left, "Kräuter", "Doppelkorn" (the new game counts double),
    /// "Ein Bier" of older content for one more Einmischen. Einmischen
    /// costs no game, so the last game of a series is no exception.
    fn throw_in(run: &Run, content: &Content) -> Option<RunAction> {
        if run.redeals_left(content) > 0 && run.accepts(content, RunAction::Redeal) {
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
        if let Some(action) = Self::usable(
            run,
            content,
            |a| matches!(a, ConsumableAction::RedealNextGame(_)),
            None,
        ) {
            return Some(action);
        }
        Self::usable(
            run,
            content,
            |a| matches!(a, ConsumableAction::ExtraRedeals(_)),
            None,
        )
    }
}

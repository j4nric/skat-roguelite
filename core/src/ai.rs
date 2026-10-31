//! Heuristic AI (design document, section 10.2, stage 1).
//!
//! Two policies: the two defenders play a coordinated priority list, the
//! declarer heuristic exists for the simulator and for boss tests. Both see
//! only what [`SeatView`] exposes: own hand, the cards on the table and the
//! cards played so far. Null games are not supported by either policy
//! (ADR 0003: Null is a character of its own, later milestone).

use crate::cards::{Card, Rank, Suit};
use crate::game::{Seat, SeatView, seat_at};
use crate::rules::{self, Declaration, GameType, RiskLevel};

/// A card-choosing strategy for one seat.
pub trait Policy {
    /// Choose a card from `view.legal_moves`. Returns `None` only if there
    /// is no legal move at all (not this seat's turn or empty hand).
    fn choose(&self, view: &SeatView<'_>) -> Option<Card>;
}

// ---------------------------------------------------------------------------
// Shared helpers
// ---------------------------------------------------------------------------

/// Points first, then rank strength: the key for "the lowest card".
fn low_key(game: GameType, card: Card) -> (u32, u8) {
    (card.card_points(), strength(game, card))
}

/// Comparable strength within the card's kind (trump or suit).
fn strength(game: GameType, card: Card) -> u8 {
    rules::trump_strength(game, card).unwrap_or_else(|| card.rank.suit_game_strength())
}

fn lowest(game: GameType, cards: &[Card]) -> Option<Card> {
    cards.iter().copied().min_by_key(|&c| low_key(game, c))
}

/// Highest Augen, ties broken by the weaker card.
fn highest_points(game: GameType, cards: &[Card]) -> Option<Card> {
    cards
        .iter()
        .copied()
        .max_by_key(|&c| (c.card_points(), u8::MAX - strength(game, c)))
}

/// Weakest card by strength (used to "take cheaply").
fn weakest(game: GameType, cards: &[Card]) -> Option<Card> {
    cards.iter().copied().min_by_key(|&c| strength(game, c))
}

fn strongest(game: GameType, cards: &[Card]) -> Option<Card> {
    cards.iter().copied().max_by_key(|&c| strength(game, c))
}

/// Whether no unseen card of the same kind can beat `card`.
fn is_sure(game: GameType, unseen: &[Card], card: Card) -> bool {
    !unseen
        .iter()
        .any(|&u| rules::follows_lead(game, card, u) && rules::beats(game, card, card, u))
}

/// Number of unseen trumps.
fn unseen_trumps(game: GameType, unseen: &[Card]) -> usize {
    unseen.iter().filter(|&&c| rules::is_trump(game, c)).count()
}

/// Cards of `hand` that are not trump, grouped by suit (deterministic order).
fn non_trump_suits(game: GameType, hand: &[Card]) -> Vec<(Suit, Vec<Card>)> {
    Suit::ALL
        .iter()
        .filter_map(|&suit| {
            let cards: Vec<Card> = hand
                .iter()
                .copied()
                .filter(|&c| c.suit == suit && !rules::is_trump(game, c))
                .collect();
            if cards.is_empty() {
                None
            } else {
                Some((suit, cards))
            }
        })
        .collect()
}

/// Suits (and trump) in which `seat` is known to be void because it failed
/// to follow a lead earlier in the game. Trump is reported as `None`.
fn known_voids(view: &SeatView<'_>, seat: Seat) -> Vec<Option<Suit>> {
    let game = view.declaration.game;
    let mut voids = Vec::new();
    let current = (view.trick_lead, view.trick);
    let completed = view.tricks.iter().map(|t| (t.lead, &t.cards[..]));
    for (lead_seat, cards) in completed.chain(std::iter::once(current)) {
        let Some(&lead) = cards.first() else {
            continue;
        };
        for (i, &card) in cards.iter().enumerate().skip(1) {
            if seat_at(lead_seat, i) == seat && !rules::follows_lead(game, lead, card) {
                let kind = if rules::is_trump(game, lead) {
                    None
                } else {
                    Some(lead.suit)
                };
                if !voids.contains(&kind) {
                    voids.push(kind);
                }
            }
        }
    }
    voids
}

fn trick_points(trick: &[Card]) -> u32 {
    trick.iter().map(|c| c.card_points()).sum()
}

/// Cards from `legal` that would currently win the trick.
fn beating(game: GameType, trick: &[Card], legal: &[Card]) -> Vec<Card> {
    let (Some(&lead), Some(idx)) = (trick.first(), rules::winning_index(game, trick)) else {
        return legal.to_vec();
    };
    let holder = trick[idx];
    legal
        .iter()
        .copied()
        .filter(|&c| rules::beats(game, lead, holder, c))
        .collect()
}

/// Lowest card, preferring non-trumps (slough without wasting trump).
fn slough(game: GameType, legal: &[Card]) -> Option<Card> {
    let non_trump: Vec<Card> = legal
        .iter()
        .copied()
        .filter(|&c| !rules::is_trump(game, c))
        .collect();
    if non_trump.is_empty() {
        lowest(game, legal)
    } else {
        lowest(game, &non_trump)
    }
}

/// Highest Augen, preferring non-trumps.
fn smear(game: GameType, legal: &[Card]) -> Option<Card> {
    let non_trump: Vec<Card> = legal
        .iter()
        .copied()
        .filter(|&c| !rules::is_trump(game, c))
        .collect();
    if non_trump.is_empty() {
        highest_points(game, legal)
    } else {
        highest_points(game, &non_trump)
    }
}

/// Lead from a suit: the top card if it is sure, otherwise the lowest.
fn lead_from_suit(game: GameType, unseen: &[Card], cards: &[Card]) -> Option<Card> {
    let top = strongest(game, cards)?;
    if is_sure(game, unseen, top) {
        Some(top)
    } else {
        lowest(game, cards)
    }
}

// ---------------------------------------------------------------------------
// Defenders
// ---------------------------------------------------------------------------

/// Coordinated defender heuristic (both defender seats use the same one).
#[derive(Debug, Clone, Copy, Default)]
pub struct DefenderHeuristic;

impl DefenderHeuristic {
    fn lead(&self, view: &SeatView<'_>, unseen: &[Card]) -> Option<Card> {
        let game = view.declaration.game;
        let legal = &view.legal_moves;
        let suits = non_trump_suits(game, legal);
        if suits.is_empty() {
            // Only trumps left: lead the top one.
            return strongest(game, legal);
        }
        let declarer_voids = known_voids(view, Seat::DECLARER);
        // Long suit in which the declarer is void: lead low, it costs nothing
        // and forces a trump or hands the trick to the partner.
        if let Some((_, cards)) = suits
            .iter()
            .filter(|(suit, _)| declarer_voids.contains(&Some(*suit)))
            .max_by_key(|(suit, cards)| (cards.len(), *suit))
        {
            return lowest(game, cards);
        }
        // Otherwise the shortest suit; prefer one where we hold the top card.
        let (_, cards) = suits.iter().min_by_key(|(suit, cards)| {
            let has_sure = strongest(game, cards).is_some_and(|c| is_sure(game, unseen, c));
            (cards.len(), !has_sure, *suit)
        })?;
        lead_from_suit(game, unseen, cards)
    }

    fn follow(&self, view: &SeatView<'_>, unseen: &[Card]) -> Option<Card> {
        let game = view.declaration.game;
        let legal = &view.legal_moves;
        let trick = view.trick;
        let lead = *trick.first()?;
        let winner_idx = rules::winning_index(game, trick)?;
        let winner_seat = seat_at(view.trick_lead, winner_idx);
        let holder = trick[winner_idx];
        let last = trick.len() == 2;
        let points = trick_points(trick);
        let can_follow = legal.iter().any(|&c| rules::follows_lead(game, lead, c));
        let beating = beating(game, trick, legal);

        if !winner_seat.is_declarer() {
            // Partner holds the trick.
            let declarer_voids = known_voids(view, Seat::DECLARER);
            let declarer_can_ruff = !rules::is_trump(game, lead)
                && declarer_voids.contains(&Some(lead.suit))
                && unseen_trumps(game, unseen) > 0;
            let partner_sure = last || (is_sure(game, unseen, holder) && !declarer_can_ruff);
            if partner_sure {
                return smear(game, legal);
            }
            // Partner may be overtaken by the declarer: win it ourselves with
            // a sure card if we have one, otherwise stay low.
            let sure: Vec<Card> = beating
                .iter()
                .copied()
                .filter(|&c| is_sure(game, unseen, c))
                .collect();
            if let Some(c) = weakest(game, &sure) {
                return Some(c);
            }
            return slough(game, legal);
        }

        // Declarer holds the trick.
        if can_follow {
            if !beating.is_empty() {
                if last {
                    return weakest(game, &beating);
                }
                let sure: Vec<Card> = beating
                    .iter()
                    .copied()
                    .filter(|&c| is_sure(game, unseen, c))
                    .collect();
                if let Some(c) = weakest(game, &sure) {
                    return Some(c);
                }
                if points >= 10 {
                    return weakest(game, &beating);
                }
            }
            return lowest(game, legal);
        }

        // Cannot follow: ruff if the trick is worth it, otherwise slough.
        if !beating.is_empty() && (points >= 10 || (last && points > 0)) {
            return weakest(game, &beating);
        }
        slough(game, legal)
    }
}

impl Policy for DefenderHeuristic {
    fn choose(&self, view: &SeatView<'_>) -> Option<Card> {
        if view.legal_moves.is_empty() {
            return None;
        }
        let unseen = view.unseen_cards();
        let choice = if view.trick.is_empty() {
            self.lead(view, &unseen)
        } else {
            self.follow(view, &unseen)
        };
        choice.or_else(|| view.legal_moves.first().copied())
    }
}

// ---------------------------------------------------------------------------
// Declarer
// ---------------------------------------------------------------------------

/// Declarer heuristic: game choice, discard and play. Used by the simulator
/// and by boss tests; never chooses Null.
#[derive(Debug, Clone, Copy, Default)]
pub struct DeclarerHeuristic;

/// Trumps of `game` held in `hand`.
fn trump_count(game: GameType, hand: &[Card]) -> usize {
    hand.iter().filter(|&&c| rules::is_trump(game, c)).count()
}

impl DeclarerHeuristic {
    /// Choose the game for a ten-card hand.
    ///
    /// Grand with at least two jacks and two aces; otherwise the suit with
    /// the most trumps if that makes at least five; otherwise the game with
    /// the highest estimated value (base value × (matadors + 1) × trumps as
    /// a crude proxy for the chance to win). Hand only with eight or more
    /// trumps, risk otherwise `Normal`.
    #[must_use]
    pub fn choose_declaration(&self, hand: &[Card]) -> Declaration {
        let jacks = hand.iter().filter(|c| c.is_jack()).count();
        let aces = hand.iter().filter(|c| c.rank == Rank::Ace).count();
        let game = if jacks >= 2 && aces >= 2 {
            GameType::Grand
        } else {
            let suit_games = Suit::ALL.iter().map(|&s| GameType::Suit(s));
            // Ties go to the higher base value (later suit in Suit::ALL).
            let best_suit = suit_games
                .clone()
                .max_by_key(|&g| (trump_count(g, hand), g.base_value()))
                .unwrap_or(GameType::Suit(Suit::Clubs));
            if trump_count(best_suit, hand) >= 5 {
                best_suit
            } else {
                suit_games
                    .chain(std::iter::once(GameType::Grand))
                    .max_by_key(|&g| {
                        let base = g.base_value().unwrap_or(0);
                        let levels = rules::matadors(g, hand).count + 1;
                        let trumps = u32::try_from(trump_count(g, hand)).unwrap_or(0);
                        (base * levels * trumps, base)
                    })
                    .unwrap_or(GameType::Grand)
            }
        };
        let risk = if trump_count(game, hand) >= 8 {
            RiskLevel::Hand
        } else {
            RiskLevel::Normal
        };
        Declaration::new(game, risk)
    }

    /// Choose the two discards from the twelve cards. Never discards aces
    /// or trumps if avoidable; prefers to clear short non-trump suits and
    /// cards with few Augen.
    #[must_use]
    pub fn choose_discard(&self, cards: &[Card], game: GameType) -> Option<[Card; 2]> {
        let suits = non_trump_suits(game, cards);
        let mut candidates: Vec<(usize, u32, u8, Card)> = Vec::new();
        for (_, suit_cards) in &suits {
            for &c in suit_cards {
                if c.rank != Rank::Ace {
                    candidates.push((suit_cards.len(), c.card_points(), strength(game, c), c));
                }
            }
        }
        candidates.sort();
        let mut chosen: Vec<Card> = candidates.iter().map(|&(_, _, _, c)| c).take(2).collect();
        if chosen.len() < 2 {
            // Fallback: lowest remaining cards of any kind.
            let mut rest: Vec<Card> = cards
                .iter()
                .copied()
                .filter(|c| !chosen.contains(c))
                .collect();
            rest.sort_by_key(|&c| (rules::is_trump(game, c), low_key(game, c)));
            chosen.extend(rest.into_iter().take(2 - chosen.len()));
        }
        match chosen[..] {
            [a, b] => Some([a, b]),
            _ => None,
        }
    }

    fn lead(&self, view: &SeatView<'_>, unseen: &[Card]) -> Option<Card> {
        let game = view.declaration.game;
        let legal = &view.legal_moves;
        let trumps: Vec<Card> = legal
            .iter()
            .copied()
            .filter(|&c| rules::is_trump(game, c))
            .collect();
        // Draw trumps while the defenders may still hold some.
        if !trumps.is_empty() && unseen_trumps(game, unseen) > 0 {
            let top = strongest(game, &trumps)?;
            return if is_sure(game, unseen, top) {
                Some(top)
            } else {
                weakest(game, &trumps)
            };
        }
        let suits = non_trump_suits(game, legal);
        // Sure winners first (aces, bare tens once the ace is gone).
        if let Some(c) = suits
            .iter()
            .filter_map(|(_, cards)| strongest(game, cards))
            .filter(|&c| is_sure(game, unseen, c))
            .max_by_key(|&c| c.card_points())
        {
            return Some(c);
        }
        // Then the longest suit, low.
        if let Some((_, cards)) = suits
            .iter()
            .max_by_key(|(suit, cards)| (cards.len(), *suit))
        {
            return lowest(game, cards);
        }
        strongest(game, legal)
    }

    fn follow(&self, view: &SeatView<'_>, unseen: &[Card]) -> Option<Card> {
        let game = view.declaration.game;
        let legal = &view.legal_moves;
        let trick = view.trick;
        let lead = *trick.first()?;
        let last = trick.len() == 2;
        let points = trick_points(trick);
        let can_follow = legal.iter().any(|&c| rules::follows_lead(game, lead, c));
        let beating = beating(game, trick, legal);
        if !beating.is_empty() {
            if last {
                return weakest(game, &beating);
            }
            let sure: Vec<Card> = beating
                .iter()
                .copied()
                .filter(|&c| is_sure(game, unseen, c))
                .collect();
            if let Some(c) = weakest(game, &sure) {
                return Some(c);
            }
            if points >= 10 || can_follow {
                return weakest(game, &beating);
            }
        }
        if can_follow {
            lowest(game, legal)
        } else {
            slough(game, legal)
        }
    }
}

impl Policy for DeclarerHeuristic {
    fn choose(&self, view: &SeatView<'_>) -> Option<Card> {
        if view.legal_moves.is_empty() {
            return None;
        }
        let unseen = view.unseen_cards();
        let choice = if view.trick.is_empty() {
            self.lead(view, &unseen)
        } else {
            self.follow(view, &unseen)
        };
        choice.or_else(|| view.legal_moves.first().copied())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cards::parse_cards;
    use crate::game::Game;
    use crate::rng::Rng;

    fn c(s: &str) -> Card {
        s.parse().expect("test card")
    }

    #[test]
    fn declaration_grand_with_two_jacks_and_two_aces() {
        let hand = parse_cards("CJ SJ CA SA HT HK D7 D8 S9 C7").expect("hand");
        let d = DeclarerHeuristic.choose_declaration(&hand);
        assert_eq!(d.game, GameType::Grand);
        assert_eq!(d.risk, RiskLevel::Normal);
    }

    #[test]
    fn declaration_longest_suit_and_hand_with_eight_trumps() {
        let hand = parse_cards("CJ HJ HA HT HK HQ H9 H8 D7 S9").expect("hand");
        let d = DeclarerHeuristic.choose_declaration(&hand);
        assert_eq!(d.game, GameType::Suit(Suit::Hearts));
        assert_eq!(d.risk, RiskLevel::Hand);
        let hand = parse_cards("CJ HA HT HK HQ D7 D8 S9 C7 C8").expect("hand");
        let d = DeclarerHeuristic.choose_declaration(&hand);
        assert_eq!(d.game, GameType::Suit(Suit::Hearts));
        assert_eq!(d.risk, RiskLevel::Normal);
    }

    #[test]
    fn declaration_never_null() {
        let mut rng = Rng::from_seed(3);
        for _ in 0..200 {
            let game = Game::deal(&mut rng, Seat::DECLARER);
            let d = DeclarerHeuristic.choose_declaration(game.hand(Seat::DECLARER));
            assert_ne!(d.game, GameType::Null);
        }
    }

    #[test]
    fn discard_avoids_aces_and_trumps_and_clears_short_suits() {
        let cards = parse_cards("CJ SJ HA HT HK HQ SA ST DT C7 D8 S9").expect("cards");
        let game = GameType::Suit(Suit::Hearts);
        let discard = DeclarerHeuristic
            .choose_discard(&cards, game)
            .expect("two cards");
        // C7 is a singleton, D8 and DT form the two-card diamond suit.
        assert_eq!(discard, [c("C7"), c("D8")]);
        let all_trumps = parse_cards("CJ SJ HJ DJ HA HT HK HQ H9 H8 H7 SA").expect("cards");
        let discard = DeclarerHeuristic
            .choose_discard(&all_trumps, game)
            .expect("two cards");
        assert_eq!(discard, [c("SA"), c("H7")]);
    }

    fn play_out(seed: u64) -> Game {
        let mut rng = Rng::from_seed(seed);
        let mut game = Game::deal(&mut rng, Seat::DECLARER);
        let declarer = DeclarerHeuristic;
        let defender = DefenderHeuristic;
        let decl = declarer.choose_declaration(game.hand(Seat::DECLARER));
        game.declare(decl).expect("declare");
        if game.phase() == crate::game::Phase::Discarding {
            let discard = declarer
                .choose_discard(game.hand(Seat::DECLARER), decl.game)
                .expect("discard");
            game.discard(discard).expect("discard");
        }
        while let Some(seat) = game.current_seat() {
            let view = game.view(seat).expect("playing");
            let card = if seat.is_declarer() {
                declarer.choose(&view)
            } else {
                defender.choose(&view)
            }
            .expect("a legal move");
            game.play(seat, card).expect("legal move");
        }
        game
    }

    #[test]
    fn heuristics_always_produce_legal_moves() {
        for seed in 0..50 {
            let game = play_out(seed);
            assert_eq!(game.tricks().len(), 10);
        }
    }

    #[test]
    fn declarer_ruffs_cheaply_as_last_player() {
        // Seat 1 leads SA, seat 2 smears the HT, the declarer is last.
        let hands = [
            parse_cards("CJ SJ HJ DJ CA CT CK CQ C9 C8").expect("hand"),
            parse_cards("SA ST SK SQ S9 S8 S7 D7 D8 D9").expect("hand"),
            parse_cards("HA HT HK HQ H9 H8 H7 DA DT DK").expect("hand"),
        ];
        let mut game = Game::with_cards(hands, [c("C7"), c("DQ")], Seat::ALL[1]).expect("deal");
        game.declare(Declaration::new(
            GameType::Suit(Suit::Clubs),
            RiskLevel::Hand,
        ))
        .expect("declare");
        game.play(Seat::ALL[1], c("SA")).expect("lead");
        game.play(Seat::ALL[2], c("HT")).expect("smear");
        let view = game.view(Seat::DECLARER).expect("view");
        // Declarer follows: cannot follow spades, trick worth 21 -> ruff cheaply.
        assert_eq!(DeclarerHeuristic.choose(&view), Some(c("C8")));
    }

    #[test]
    fn defender_sloughs_low_when_declarer_holds() {
        let hands = [
            parse_cards("CJ SJ HJ DJ CA CT CK CQ C9 C8").expect("hand"),
            parse_cards("SA ST SK SQ S9 S8 S7 D7 D8 D9").expect("hand"),
            parse_cards("HA HT HK HQ H9 H8 H7 DA DT DK").expect("hand"),
        ];
        let mut game = Game::with_cards(hands, [c("C7"), c("DQ")], Seat::DECLARER).expect("deal");
        game.declare(Declaration::new(
            GameType::Suit(Suit::Clubs),
            RiskLevel::Hand,
        ))
        .expect("declare");
        game.play(Seat::DECLARER, c("CJ")).expect("lead");
        let view = game.view(Seat::ALL[1]).expect("view");
        assert_eq!(DefenderHeuristic.choose(&view), Some(c("D7")));
    }
}

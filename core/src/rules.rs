//! Rule definitions that are pure data: game types, base values, trump order.
//!
//! Trick resolution, matador counting and the game value formula itself are
//! implemented in milestone M1. The golden test vectors for the formula live
//! in `core/tests/game_value_vectors.rs`.

use crate::cards::{Card, Rank, Suit};

/// The game the declarer (Alleinspieler) announces.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GameType {
    /// Farbspiel: jacks plus one suit are trump.
    Suit(Suit),
    /// Grand: only the jacks are trump.
    Grand,
    /// Null: no trumps, the declarer must not win a single trick.
    Null,
}

impl GameType {
    /// Grundwert for suit and grand games. Null games use fixed values
    /// (see [`null_value`]) and return `None` here.
    #[must_use]
    pub const fn base_value(self) -> Option<u32> {
        match self {
            GameType::Suit(suit) => Some(suit.base_value()),
            GameType::Grand => Some(24),
            GameType::Null => None,
        }
    }

    /// The trumps of this game, strongest first. Empty for Null.
    #[must_use]
    pub fn trump_order(self) -> Vec<Card> {
        let jacks = [Suit::Clubs, Suit::Spades, Suit::Hearts, Suit::Diamonds]
            .into_iter()
            .map(|suit| Card::new(suit, Rank::Jack));
        match self {
            GameType::Grand => jacks.collect(),
            GameType::Suit(trump) => jacks
                .chain(
                    [
                        Rank::Ace,
                        Rank::Ten,
                        Rank::King,
                        Rank::Queen,
                        Rank::Nine,
                        Rank::Eight,
                        Rank::Seven,
                    ]
                    .into_iter()
                    .map(|rank| Card::new(trump, rank)),
                )
                .collect(),
            GameType::Null => Vec::new(),
        }
    }
}

/// Fixed Spielwert of a Null game.
#[must_use]
pub const fn null_value(hand: bool, ouvert: bool) -> u32 {
    match (hand, ouvert) {
        (false, false) => 23,
        (true, false) => 35,
        (false, true) => 46,
        (true, true) => 59,
    }
}

/// Augen the declarer needs to win a suit or grand game.
pub const WIN_THRESHOLD: u32 = 61;
/// Augen from which the opponents are Schneider.
pub const SCHNEIDER_THRESHOLD: u32 = 90;
/// Augen in the whole deck.
pub const TOTAL_CARD_POINTS: u32 = 120;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn suit_game_has_eleven_trumps_starting_with_clubs_jack() {
        let order = GameType::Suit(Suit::Hearts).trump_order();
        assert_eq!(order.len(), 11);
        assert_eq!(order[0].to_string(), "CJ");
        assert_eq!(order[4].to_string(), "HA");
        assert_eq!(order[10].to_string(), "H7");
    }

    #[test]
    fn grand_has_only_jacks() {
        let order = GameType::Grand.trump_order();
        assert_eq!(order.len(), 4);
        assert!(order.iter().all(|c| c.is_jack()));
    }

    #[test]
    fn null_has_no_trumps_and_fixed_values() {
        assert!(GameType::Null.trump_order().is_empty());
        assert_eq!(GameType::Null.base_value(), None);
        assert_eq!(null_value(false, false), 23);
        assert_eq!(null_value(true, true), 59);
    }
}

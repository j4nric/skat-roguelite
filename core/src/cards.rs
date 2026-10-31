//! Cards, suits, ranks and the 32-card Skat deck.
//!
//! Card notation used in tests, logs and data files: two characters,
//! suit then rank, e.g. `CJ` = Kreuz-Bube, `DT` = Karo-Zehn.
//! Suits: `D` Diamonds/Karo, `H` Hearts/Herz, `S` Spades/Pik, `C` Clubs/Kreuz.
//! Ranks: `7 8 9 T J Q K A`.

use std::fmt;
use std::str::FromStr;

/// The four suits, ordered by their base value in a suit game (Karo lowest).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Suit {
    /// Karo
    Diamonds,
    /// Herz
    Hearts,
    /// Pik
    Spades,
    /// Kreuz
    Clubs,
}

impl Suit {
    /// All suits, ascending by base value.
    pub const ALL: [Suit; 4] = [Suit::Diamonds, Suit::Hearts, Suit::Spades, Suit::Clubs];

    /// Grundwert of a suit game with this suit as trump.
    #[must_use]
    pub const fn base_value(self) -> u32 {
        match self {
            Suit::Diamonds => 9,
            Suit::Hearts => 10,
            Suit::Spades => 11,
            Suit::Clubs => 12,
        }
    }

    /// One-letter symbol used in the card notation.
    #[must_use]
    pub const fn symbol(self) -> char {
        match self {
            Suit::Diamonds => 'D',
            Suit::Hearts => 'H',
            Suit::Spades => 'S',
            Suit::Clubs => 'C',
        }
    }

    fn from_symbol(c: char) -> Option<Suit> {
        match c.to_ascii_uppercase() {
            'D' => Some(Suit::Diamonds),
            'H' => Some(Suit::Hearts),
            'S' => Some(Suit::Spades),
            'C' => Some(Suit::Clubs),
            _ => None,
        }
    }
}

/// The eight ranks, in natural (Null game) order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Rank {
    /// Sieben
    Seven,
    /// Acht
    Eight,
    /// Neun
    Nine,
    /// Zehn
    Ten,
    /// Bube
    Jack,
    /// Dame
    Queen,
    /// König
    King,
    /// Ass
    Ace,
}

impl Rank {
    /// All ranks in natural order.
    pub const ALL: [Rank; 8] = [
        Rank::Seven,
        Rank::Eight,
        Rank::Nine,
        Rank::Ten,
        Rank::Jack,
        Rank::Queen,
        Rank::King,
        Rank::Ace,
    ];

    /// Augen: the card points of this rank. The whole deck sums to 120.
    #[must_use]
    pub const fn card_points(self) -> u32 {
        match self {
            Rank::Ace => 11,
            Rank::Ten => 10,
            Rank::King => 4,
            Rank::Queen => 3,
            Rank::Jack => 2,
            Rank::Seven | Rank::Eight | Rank::Nine => 0,
        }
    }

    /// Strength inside a suit when following suit in a suit or grand game
    /// (Ass > Zehn > König > Dame > Neun > Acht > Sieben). Jacks are trumps
    /// there and never compared by this value.
    #[must_use]
    pub const fn suit_game_strength(self) -> u8 {
        match self {
            Rank::Ace => 7,
            Rank::Ten => 6,
            Rank::King => 5,
            Rank::Queen => 4,
            Rank::Nine => 3,
            Rank::Eight => 2,
            Rank::Seven => 1,
            Rank::Jack => 0,
        }
    }

    /// One-character symbol used in the card notation.
    #[must_use]
    pub const fn symbol(self) -> char {
        match self {
            Rank::Seven => '7',
            Rank::Eight => '8',
            Rank::Nine => '9',
            Rank::Ten => 'T',
            Rank::Jack => 'J',
            Rank::Queen => 'Q',
            Rank::King => 'K',
            Rank::Ace => 'A',
        }
    }

    fn from_symbol(c: char) -> Option<Rank> {
        match c.to_ascii_uppercase() {
            '7' => Some(Rank::Seven),
            '8' => Some(Rank::Eight),
            '9' => Some(Rank::Nine),
            'T' => Some(Rank::Ten),
            'J' => Some(Rank::Jack),
            'Q' => Some(Rank::Queen),
            'K' => Some(Rank::King),
            'A' => Some(Rank::Ace),
            _ => None,
        }
    }
}

/// A playing card. Card modifications (gezinkte Karten) are not part of this
/// type; they are attached to card instances in the deck (milestone M2).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Card {
    /// Farbe
    pub suit: Suit,
    /// Wert
    pub rank: Rank,
}

impl Card {
    /// Create a card.
    #[must_use]
    pub const fn new(suit: Suit, rank: Rank) -> Card {
        Card { suit, rank }
    }

    /// Augen of this card.
    #[must_use]
    pub const fn card_points(self) -> u32 {
        self.rank.card_points()
    }

    /// Whether this card is a Bube.
    #[must_use]
    pub const fn is_jack(self) -> bool {
        matches!(self.rank, Rank::Jack)
    }
}

impl fmt::Display for Card {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}", self.suit.symbol(), self.rank.symbol())
    }
}

/// Error returned when parsing a card from its two-character notation fails.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseCardError(pub String);

impl fmt::Display for ParseCardError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid card notation: {:?}", self.0)
    }
}

impl std::error::Error for ParseCardError {}

impl FromStr for Card {
    type Err = ParseCardError;

    fn from_str(s: &str) -> Result<Card, ParseCardError> {
        let mut chars = s.trim().chars();
        let (Some(s_char), Some(r_char), None) = (chars.next(), chars.next(), chars.next()) else {
            return Err(ParseCardError(s.to_string()));
        };
        match (Suit::from_symbol(s_char), Rank::from_symbol(r_char)) {
            (Some(suit), Some(rank)) => Ok(Card::new(suit, rank)),
            _ => Err(ParseCardError(s.to_string())),
        }
    }
}

/// The standard 32-card deck in a fixed order (Karo-7 … Kreuz-Ass).
#[must_use]
pub fn standard_deck() -> Vec<Card> {
    Suit::ALL
        .iter()
        .flat_map(|&suit| Rank::ALL.iter().map(move |&rank| Card::new(suit, rank)))
        .collect()
}

/// Parse a whitespace-separated list of cards, e.g. `"CJ SJ HA D7"`.
pub fn parse_cards(s: &str) -> Result<Vec<Card>, ParseCardError> {
    s.split_whitespace().map(str::parse).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deck_has_32_distinct_cards() {
        let deck = standard_deck();
        assert_eq!(deck.len(), 32);
        let mut sorted = deck.clone();
        sorted.sort();
        sorted.dedup();
        assert_eq!(sorted.len(), 32);
    }

    #[test]
    fn deck_has_120_card_points() {
        let total: u32 = standard_deck().iter().map(|c| c.card_points()).sum();
        assert_eq!(total, 120);
    }

    #[test]
    fn deck_has_four_jacks() {
        assert_eq!(standard_deck().iter().filter(|c| c.is_jack()).count(), 4);
    }

    #[test]
    fn notation_round_trips() {
        for card in standard_deck() {
            let text = card.to_string();
            assert_eq!(text.len(), 2);
            assert_eq!(text.parse::<Card>().unwrap(), card);
        }
    }

    #[test]
    fn notation_rejects_garbage() {
        assert!("".parse::<Card>().is_err());
        assert!("C".parse::<Card>().is_err());
        assert!("CJX".parse::<Card>().is_err());
        assert!("XJ".parse::<Card>().is_err());
        assert!("C1".parse::<Card>().is_err());
    }

    #[test]
    fn parse_cards_handles_lists() {
        let cards = parse_cards("CJ sj  HA\tD7").unwrap();
        assert_eq!(cards.len(), 4);
        assert_eq!(cards[1], Card::new(Suit::Spades, Rank::Jack));
    }
}

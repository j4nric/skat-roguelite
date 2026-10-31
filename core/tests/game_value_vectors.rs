//! Golden vectors for the Spielwert (game value) formula.
//!
//! Each vector lists the twelve cards the declarer ends up with (hand plus
//! skat), because matadors are counted over all twelve. The expected values
//! were worked out by hand from the Skatordnung.
//!
//! Status: the vectors are checked for internal consistency only. Milestone
//! M1 replaces `expected_from_vector` with a call into `skat_core::rules` and
//! these vectors become the acceptance test for the formula.

use skat_core::cards::Suit;
use skat_core::cards::parse_cards;
use skat_core::rules::{GameType, null_value};

struct Vector {
    name: &'static str,
    game: GameType,
    /// Hand plus skat, 12 cards.
    cards: &'static str,
    hand: bool,
    schneider: bool,
    schneider_declared: bool,
    schwarz: bool,
    schwarz_declared: bool,
    ouvert: bool,
    /// `true` = "mit", `false` = "ohne".
    with: bool,
    matadors: u32,
    levels: u32,
    value: u32,
}

const VECTORS: &[Vector] = &[
    Vector {
        name: "grand mit 3, Spiel 4",
        game: GameType::Grand,
        cards: "CJ SJ HJ CA CT SA ST HA HT DA D7 D8",
        hand: false,
        schneider: false,
        schneider_declared: false,
        schwarz: false,
        schwarz_declared: false,
        ouvert: false,
        with: true,
        matadors: 3,
        levels: 4,
        value: 96,
    },
    Vector {
        name: "kreuz mit 2, Spiel 3, Hand 4, Schneider 5",
        game: GameType::Suit(Suit::Clubs),
        cards: "CJ SJ CA CT CK CQ C9 C8 SA ST HA DA",
        hand: true,
        schneider: true,
        schneider_declared: false,
        schwarz: false,
        schwarz_declared: false,
        ouvert: false,
        with: true,
        matadors: 2,
        levels: 5,
        value: 60,
    },
    Vector {
        name: "herz ohne 2, Spiel 3",
        game: GameType::Suit(Suit::Hearts),
        cards: "HJ DJ HA HT HK HQ SA ST CA CT DA DT",
        hand: false,
        schneider: false,
        schneider_declared: false,
        schwarz: false,
        schwarz_declared: false,
        ouvert: false,
        with: false,
        matadors: 2,
        levels: 3,
        value: 30,
    },
    Vector {
        name: "karo mit 6, Spiel 7 (Karo-König fehlt)",
        game: GameType::Suit(Suit::Diamonds),
        cards: "CJ SJ HJ DJ DA DT DQ D9 CA SA HA H7",
        hand: false,
        schneider: false,
        schneider_declared: false,
        schwarz: false,
        schwarz_declared: false,
        ouvert: false,
        with: true,
        matadors: 6,
        levels: 7,
        value: 63,
    },
    Vector {
        name: "karo ohne 11, Spiel 12 (kein einziger Trumpf)",
        game: GameType::Suit(Suit::Diamonds),
        cards: "CA CT CK CQ SA ST SK SQ HA HT HK HQ",
        hand: false,
        schneider: false,
        schneider_declared: false,
        schwarz: false,
        schwarz_declared: false,
        ouvert: false,
        with: false,
        matadors: 11,
        levels: 12,
        value: 108,
    },
    Vector {
        name: "grand ouvert ohne 4: alle Stufen",
        game: GameType::Grand,
        cards: "CA CT CK SA ST SK HA HT HK DA DT DK",
        hand: true,
        schneider: true,
        schneider_declared: true,
        schwarz: true,
        schwarz_declared: true,
        ouvert: true,
        with: false,
        matadors: 4,
        levels: 11,
        value: 264,
    },
    Vector {
        name: "pik mit 1, Spiel 2",
        game: GameType::Suit(Suit::Spades),
        cards: "CJ HJ SA ST SK S7 CA CT HA HT DA DT",
        hand: false,
        schneider: false,
        schneider_declared: false,
        schwarz: false,
        schwarz_declared: false,
        ouvert: false,
        with: true,
        matadors: 1,
        levels: 2,
        value: 22,
    },
    Vector {
        name: "null",
        game: GameType::Null,
        cards: "D7 D8 D9 H7 H8 H9 S7 S8 S9 C7 C8 C9",
        hand: false,
        schneider: false,
        schneider_declared: false,
        schwarz: false,
        schwarz_declared: false,
        ouvert: false,
        with: false,
        matadors: 0,
        levels: 0,
        value: 23,
    },
    Vector {
        name: "null hand",
        game: GameType::Null,
        cards: "D7 D8 D9 H7 H8 H9 S7 S8 S9 C7 C8 C9",
        hand: true,
        schneider: false,
        schneider_declared: false,
        schwarz: false,
        schwarz_declared: false,
        ouvert: false,
        with: false,
        matadors: 0,
        levels: 0,
        value: 35,
    },
    Vector {
        name: "null ouvert",
        game: GameType::Null,
        cards: "D7 D8 D9 H7 H8 H9 S7 S8 S9 C7 C8 C9",
        hand: false,
        schneider: false,
        schneider_declared: false,
        schwarz: false,
        schwarz_declared: false,
        ouvert: true,
        with: false,
        matadors: 0,
        levels: 0,
        value: 46,
    },
    Vector {
        name: "null ouvert hand",
        game: GameType::Null,
        cards: "D7 D8 D9 H7 H8 H9 S7 S8 S9 C7 C8 C9",
        hand: true,
        schneider: false,
        schneider_declared: false,
        schwarz: false,
        schwarz_declared: false,
        ouvert: true,
        with: false,
        matadors: 0,
        levels: 0,
        value: 59,
    },
];

/// Reference computation straight from the formula, using the matador count
/// stated in the vector. M1 swaps this for the real implementation that
/// derives the matadors from the cards.
fn expected_from_vector(v: &Vector) -> u32 {
    match v.game.base_value() {
        None => null_value(v.hand, v.ouvert),
        Some(base) => {
            let levels = v.matadors
                + 1
                + u32::from(v.hand)
                + u32::from(v.schneider)
                + u32::from(v.schneider_declared)
                + u32::from(v.schwarz)
                + u32::from(v.schwarz_declared)
                + u32::from(v.ouvert);
            base * levels
        }
    }
}

/// Count matadors from the cards, mirroring the Skatordnung: starting at the
/// strongest trump, count how many consecutive trumps are held ("mit") or
/// missing ("ohne").
fn matadors_from_cards(game: GameType, cards: &str) -> (bool, u32) {
    let held = parse_cards(cards).expect("vector cards parse");
    let order = game.trump_order();
    let Some(top) = order.first() else {
        return (false, 0);
    };
    let with = held.contains(top);
    let count = order
        .iter()
        .take_while(|c| held.contains(c) == with)
        .count();
    (with, u32::try_from(count).expect("fits"))
}

#[test]
fn vectors_have_twelve_distinct_cards() {
    for v in VECTORS {
        let mut cards = parse_cards(v.cards).unwrap_or_else(|e| panic!("{}: {e}", v.name));
        assert_eq!(cards.len(), 12, "{}", v.name);
        cards.sort();
        cards.dedup();
        assert_eq!(cards.len(), 12, "{}: duplicate card", v.name);
    }
}

#[test]
fn vectors_matadors_match_their_cards() {
    for v in VECTORS {
        let (with, count) = matadors_from_cards(v.game, v.cards);
        assert_eq!((with, count), (v.with, v.matadors), "{}", v.name);
    }
}

#[test]
fn vectors_are_internally_consistent() {
    for v in VECTORS {
        assert_eq!(expected_from_vector(v), v.value, "{}", v.name);
        if v.game.base_value().is_some() {
            assert_eq!(
                v.value % v.levels,
                0,
                "{}: value must be base × levels",
                v.name
            );
        }
    }
}

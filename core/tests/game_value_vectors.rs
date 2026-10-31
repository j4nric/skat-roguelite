//! Golden vectors for the Spielwert (game value) formula.
//!
//! Each vector lists the twelve cards the declarer ends up with (hand plus
//! skat), because matadors are counted over all twelve. The expected values
//! were worked out by hand from the Skatordnung.
//!
//! These vectors are the acceptance test for `skat_core::rules::matadors`
//! and `skat_core::rules::game_value`. Each vector is turned into a
//! declaration and an outcome that reaches exactly the listed levels.

use skat_core::cards::Suit;
use skat_core::cards::parse_cards;
use skat_core::rules::{
    Declaration, GameType, Outcome, RiskLevel, SCHNEIDER_THRESHOLD, TOTAL_CARD_POINTS,
    WIN_THRESHOLD, game_value, matadors, null_game_value,
};

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

/// Risk level that produces exactly the declared flags of the vector.
/// Null Ouvert without Hand (46) has no `RiskLevel` yet (see the open
/// question in the M1 report); those vectors go through `null_game_value`.
fn risk_from_vector(v: &Vector) -> Option<RiskLevel> {
    match (v.hand, v.schneider_declared, v.schwarz_declared, v.ouvert) {
        (false, false, false, false) => Some(RiskLevel::Normal),
        (true, false, false, false) => Some(RiskLevel::Hand),
        (true, true, false, false) => Some(RiskLevel::SchneiderDeclared),
        (true, true, true, false) => Some(RiskLevel::SchwarzDeclared),
        (true, true, true, true) => Some(RiskLevel::Ouvert),
        _ => None,
    }
}

/// An outcome in which the declarer wins and reaches exactly the Schneider
/// and Schwarz flags of the vector. Null vectors are won: no trick.
fn outcome_from_vector(v: &Vector) -> Outcome {
    if v.game == GameType::Null {
        return Outcome {
            declarer_card_points: 0,
            defender_card_points: TOTAL_CARD_POINTS,
            declarer_tricks: 0,
            defender_tricks: 10,
        };
    }
    let (declarer_card_points, declarer_tricks) = if v.schwarz {
        (TOTAL_CARD_POINTS, 10)
    } else if v.schneider {
        (SCHNEIDER_THRESHOLD, 9)
    } else {
        (WIN_THRESHOLD, 6)
    };
    Outcome {
        declarer_card_points,
        defender_card_points: TOTAL_CARD_POINTS - declarer_card_points,
        declarer_tricks,
        defender_tricks: 10 - declarer_tricks,
    }
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
        let cards = parse_cards(v.cards).unwrap_or_else(|e| panic!("{}: {e}", v.name));
        let m = matadors(v.game, &cards);
        assert_eq!((m.with, m.count), (v.with, v.matadors), "{}", v.name);
    }
}

#[test]
fn vectors_match_game_value() {
    for v in VECTORS {
        let cards = parse_cards(v.cards).unwrap_or_else(|e| panic!("{}: {e}", v.name));
        let outcome = outcome_from_vector(v);
        let value = match risk_from_vector(v) {
            Some(risk) => game_value(
                Declaration::new(v.game, risk),
                matadors(v.game, &cards),
                &outcome,
            ),
            None => {
                assert_eq!(
                    v.game,
                    GameType::Null,
                    "{}: unrepresentable risk level",
                    v.name
                );
                null_game_value(v.hand, v.ouvert, &outcome)
            }
        };
        assert!(value.won, "{}", v.name);
        assert_eq!(value.levels, v.levels, "{}: levels", v.name);
        assert_eq!(value.value, v.value, "{}: value", v.name);
        assert_eq!(
            value.score,
            i32::try_from(v.value).expect("fits"),
            "{}: score",
            v.name
        );
        assert_eq!(value.schneider, v.schneider, "{}: schneider", v.name);
        assert_eq!(value.schwarz, v.schwarz, "{}: schwarz", v.name);
    }
}

#[test]
fn vectors_lost_score_double_negative() {
    for v in VECTORS {
        let Some(risk) = risk_from_vector(v) else {
            continue;
        };
        if v.schneider != v.schneider_declared || v.schwarz != v.schwarz_declared {
            // Reached-but-undeclared levels disappear in a loss; those
            // vectors are covered by the won case only.
            continue;
        }
        let cards = parse_cards(v.cards).unwrap_or_else(|e| panic!("{}: {e}", v.name));
        // One trick with 60 Augen for the declarer: lost without being Schneider.
        let lost = Outcome {
            declarer_card_points: 60,
            defender_card_points: 60,
            declarer_tricks: 1,
            defender_tricks: 9,
        };
        let value = game_value(
            Declaration::new(v.game, risk),
            matadors(v.game, &cards),
            &lost,
        );
        assert!(!value.won, "{}", v.name);
        assert_eq!(value.value, v.value, "{}: lost value", v.name);
        assert_eq!(
            value.score,
            -2 * i32::try_from(v.value).expect("fits"),
            "{}: lost score",
            v.name
        );
    }
}

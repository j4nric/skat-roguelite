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
    Declaration, GameType, Matadors, NullRisk, Outcome, RiskLevel, SCHNEIDER_THRESHOLD,
    TOTAL_CARD_POINTS, WIN_THRESHOLD, game_value, matadors, null_game_value,
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
        name: "kreuz mit 4, Spiel 5, Hand 6, Schneider 7, Schneider angesagt 8",
        game: GameType::Suit(Suit::Clubs),
        cards: "CJ SJ HJ DJ CT CK CQ C9 SA ST HA DA",
        hand: true,
        schneider: true,
        schneider_declared: true,
        schwarz: false,
        schwarz_declared: false,
        ouvert: false,
        with: true,
        matadors: 4,
        levels: 8,
        value: 96,
    },
    Vector {
        name: "herz ohne 1, Spiel 2, Hand 3, Schneider 4, angesagt 5, Schwarz 6, angesagt 7",
        game: GameType::Suit(Suit::Hearts),
        cards: "SJ HJ HA HT HK HQ H9 SA ST CA CT DA",
        hand: true,
        schneider: true,
        schneider_declared: true,
        schwarz: true,
        schwarz_declared: true,
        ouvert: false,
        with: false,
        matadors: 1,
        levels: 7,
        value: 70,
    },
    Vector {
        name: "grand hand mit 4, Spiel 5, Hand 6, Schneider 7, Schwarz 8 (unangesagt)",
        game: GameType::Grand,
        cards: "CJ SJ HJ DJ CA CT SA ST HA HT DA DT",
        hand: true,
        schneider: true,
        schneider_declared: false,
        schwarz: true,
        schwarz_declared: false,
        ouvert: false,
        with: true,
        matadors: 4,
        levels: 8,
        value: 192,
    },
    Vector {
        name: "pik ouvert mit 11: Spitzen 11 plus sieben Stufen",
        game: GameType::Suit(Suit::Spades),
        cards: "CJ SJ HJ DJ SA ST SK SQ S9 S8 S7 HA",
        hand: true,
        schneider: true,
        schneider_declared: true,
        schwarz: true,
        schwarz_declared: true,
        ouvert: true,
        with: true,
        matadors: 11,
        levels: 18,
        value: 198,
    },
    // ADR 0013: Schneider, Schwarz angesagt and Ouvert after picking up the
    // skat. Every level counts except Hand.
    Vector {
        name: "grand mit 1, Spiel 2, Schneider 3, angesagt 4 (nach dem Skat)",
        game: GameType::Grand,
        cards: "CJ HJ DJ CA CT SA ST HA HT DA D7 D8",
        hand: false,
        schneider: true,
        schneider_declared: true,
        schwarz: false,
        schwarz_declared: false,
        ouvert: false,
        with: true,
        matadors: 1,
        levels: 4,
        value: 96,
    },
    Vector {
        name: "pik ohne 1, Spiel 2, Schneider 3, angesagt 4, Schwarz 5, angesagt 6, Ouvert 7 (nach dem Skat)",
        game: GameType::Suit(Suit::Spades),
        cards: "SJ HJ DJ SA ST SK SQ S9 S8 S7 HA DA",
        hand: false,
        schneider: true,
        schneider_declared: true,
        schwarz: true,
        schwarz_declared: true,
        ouvert: true,
        with: false,
        matadors: 1,
        levels: 7,
        value: 77,
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

/// The declaration that produces exactly the declared flags of the vector.
/// Null vectors use the Null risk levels (`NullRisk`), which also cover
/// Null Ouvert without Hand (46).
fn declaration_from_vector(v: &Vector) -> Option<Declaration> {
    if v.game == GameType::Null {
        if v.schneider_declared || v.schwarz_declared {
            return None;
        }
        let risk = match (v.hand, v.ouvert) {
            (false, false) => NullRisk::Normal,
            (true, false) => NullRisk::Hand,
            (false, true) => NullRisk::Ouvert,
            (true, true) => NullRisk::OuvertHand,
        };
        return Some(Declaration::null(risk));
    }
    let declaration = Declaration::new(v.game, risk_from_vector(v)?)?.with_hand(v.hand);
    (declaration.hand() == v.hand).then_some(declaration)
}

/// Risk level that produces exactly the declared flags of a suit or grand
/// vector. The announcements may come from the hand or, since ADR 0013,
/// after the skat ([`Declaration::with_hand`]).
fn risk_from_vector(v: &Vector) -> Option<RiskLevel> {
    match (v.hand, v.schneider_declared, v.schwarz_declared, v.ouvert) {
        (false, false, false, false) => Some(RiskLevel::Normal),
        (true, false, false, false) => Some(RiskLevel::Hand),
        (_, true, false, false) => Some(RiskLevel::SchneiderDeclared),
        (_, true, true, false) => Some(RiskLevel::SchwarzDeclared),
        (_, true, true, true) => Some(RiskLevel::Ouvert),
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
        let declaration = declaration_from_vector(v)
            .unwrap_or_else(|| panic!("{}: unrepresentable declaration", v.name));
        let value = game_value(declaration, matadors(v.game, &cards), &outcome);
        if v.game == GameType::Null {
            assert_eq!(value, null_game_value(v.hand, v.ouvert, &outcome));
        }
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
        let Some(declaration) = declaration_from_vector(v) else {
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
        let value = game_value(declaration, matadors(v.game, &cards), &lost);
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

// ---------------------------------------------------------------------------
// Lost games and boundary cases (Skatordnung, Spielwert / verlorene Spiele)
// ---------------------------------------------------------------------------

struct LostOrBoundary {
    name: &'static str,
    game: GameType,
    risk: RiskLevel,
    matadors: Matadors,
    outcome: Outcome,
    won: bool,
    schneider: bool,
    schwarz: bool,
    levels: u32,
    score: i32,
}

const MIT_2: Matadors = Matadors {
    with: true,
    count: 2,
};

const OHNE_3: Matadors = Matadors {
    with: false,
    count: 3,
};

/// Outcome with all 120 Augen distributed, defenders get the rest.
const fn split(declarer_card_points: u32, declarer_tricks: u32) -> Outcome {
    Outcome {
        declarer_card_points,
        defender_card_points: TOTAL_CARD_POINTS - declarer_card_points,
        declarer_tricks,
        defender_tricks: 10 - declarer_tricks,
    }
}

const BOUNDARY: &[LostOrBoundary] = &[
    LostOrBoundary {
        name: "Schneider: Gegner genau 30 Augen",
        game: GameType::Suit(Suit::Hearts),
        risk: RiskLevel::Normal,
        matadors: MIT_2,
        outcome: split(90, 7),
        won: true,
        schneider: true,
        schwarz: false,
        levels: 4,
        score: 40,
    },
    LostOrBoundary {
        name: "kein Schneider: Gegner 31 Augen",
        game: GameType::Suit(Suit::Hearts),
        risk: RiskLevel::Normal,
        matadors: MIT_2,
        outcome: split(89, 9),
        won: true,
        schneider: false,
        schwarz: false,
        levels: 3,
        score: 30,
    },
    LostOrBoundary {
        name: "kein Schwarz: Gegner haben einen Stich mit 0 Augen",
        game: GameType::Grand,
        risk: RiskLevel::Normal,
        matadors: MIT_2,
        outcome: split(120, 9),
        won: true,
        schneider: true,
        schwarz: false,
        levels: 4,
        score: 96,
    },
    LostOrBoundary {
        name: "Schwarz angesagt, Gegner haben Stich mit 0 Augen: verloren",
        game: GameType::Grand,
        risk: RiskLevel::SchwarzDeclared,
        matadors: MIT_2,
        outcome: split(120, 9),
        won: false,
        schneider: true,
        schwarz: false,
        // mit 2, Spiel 3, Hand 4, Schneider 5, angesagt 6, Schwarz 7, angesagt 8
        levels: 8,
        score: -384,
    },
    LostOrBoundary {
        name: "Ouvert (Schwarz angesagt), Gegner haben Stich mit 0 Augen: verloren",
        game: GameType::Suit(Suit::Clubs),
        risk: RiskLevel::Ouvert,
        matadors: MIT_2,
        outcome: split(120, 9),
        won: false,
        schneider: true,
        schwarz: false,
        levels: 9,
        score: -216,
    },
    LostOrBoundary {
        name: "Schneider angesagt, unangesagter Schwarz erreicht",
        game: GameType::Suit(Suit::Diamonds),
        risk: RiskLevel::SchneiderDeclared,
        matadors: MIT_2,
        outcome: split(120, 10),
        won: true,
        schneider: true,
        schwarz: true,
        // mit 2, Spiel 3, Hand 4, Schneider 5, angesagt 6, Schwarz 7
        levels: 7,
        score: 63,
    },
    LostOrBoundary {
        name: "Hand verloren: Hand zählt mit",
        game: GameType::Suit(Suit::Spades),
        risk: RiskLevel::Hand,
        matadors: MIT_2,
        outcome: split(60, 4),
        won: false,
        schneider: false,
        schwarz: false,
        levels: 4,
        score: -88,
    },
    LostOrBoundary {
        name: "ohne 3 verloren: Spitzen zählen auch im verlorenen Spiel",
        game: GameType::Grand,
        risk: RiskLevel::Normal,
        matadors: OHNE_3,
        outcome: split(55, 4),
        won: false,
        schneider: false,
        schwarz: false,
        levels: 4,
        score: -192,
    },
    LostOrBoundary {
        name: "Alleinspieler Schneider: genau 30 Augen",
        game: GameType::Suit(Suit::Hearts),
        risk: RiskLevel::Hand,
        matadors: MIT_2,
        outcome: split(30, 3),
        won: false,
        schneider: false,
        schwarz: false,
        // mit 2, Spiel 3, Hand 4, Schneider 5
        levels: 5,
        score: -100,
    },
    LostOrBoundary {
        name: "Alleinspieler Schneider: 0 Augen, aber ein Stich (nicht schwarz)",
        game: GameType::Suit(Suit::Hearts),
        risk: RiskLevel::Normal,
        matadors: MIT_2,
        outcome: split(0, 1),
        won: false,
        schneider: false,
        schwarz: false,
        levels: 4,
        score: -80,
    },
    LostOrBoundary {
        name: "Alleinspieler schwarz trotz Augen im Skat (Skat ist kein Stich)",
        game: GameType::Suit(Suit::Hearts),
        risk: RiskLevel::Normal,
        matadors: MIT_2,
        outcome: split(21, 0),
        won: false,
        schneider: false,
        schwarz: false,
        // mit 2, Spiel 3, Schneider 4, Schwarz 5
        levels: 5,
        score: -100,
    },
    LostOrBoundary {
        name: "Schneider angesagt, Alleinspieler selbst Schneider: Stufe nicht doppelt",
        game: GameType::Grand,
        risk: RiskLevel::SchneiderDeclared,
        matadors: MIT_2,
        outcome: split(25, 2),
        won: false,
        schneider: false,
        schwarz: false,
        // mit 2, Spiel 3, Hand 4, Schneider 5, angesagt 6
        levels: 6,
        score: -288,
    },
    LostOrBoundary {
        name: "Schwarz angesagt, Alleinspieler selbst schwarz: Stufen nicht doppelt",
        game: GameType::Grand,
        risk: RiskLevel::SchwarzDeclared,
        matadors: MIT_2,
        outcome: split(0, 0),
        won: false,
        schneider: false,
        schwarz: false,
        levels: 8,
        score: -384,
    },
];

#[test]
fn boundary_and_lost_vectors_match_game_value() {
    for v in BOUNDARY {
        let declaration = Declaration::new(v.game, v.risk).expect("suit or grand");
        let value = game_value(declaration, v.matadors, &v.outcome);
        assert_eq!(value.won, v.won, "{}: won", v.name);
        assert_eq!(value.schneider, v.schneider, "{}: schneider", v.name);
        assert_eq!(value.schwarz, v.schwarz, "{}: schwarz", v.name);
        assert_eq!(value.levels, v.levels, "{}: levels", v.name);
        let base = v.game.base_value().expect("suit or grand");
        assert_eq!(value.value, base * v.levels, "{}: value", v.name);
        assert_eq!(value.score, v.score, "{}: score", v.name);
    }
}

// Skatordnung: Schwarz always includes Schneider. If the defenders took no
// trick they have 0 Augen and are Schneider, whatever the deck total is.
// With the standard deck (120 Augen) this case cannot occur, but the
// roguelike deck can lose Augen (Blanko-Karten, DESIGN 7.1), and `Outcome`
// already admits it. `game_value` decides Schneider by
// `declarer_card_points >= 90` and ignores `defender_card_points`.
#[test]
fn schwarz_implies_schneider_with_reduced_deck() {
    let outcome = Outcome {
        declarer_card_points: 85,
        defender_card_points: 0,
        declarer_tricks: 10,
        defender_tricks: 0,
    };
    let decl = Declaration::grand(RiskLevel::Normal);
    let value = game_value(decl, MIT_2, &outcome);
    assert!(value.won);
    assert!(value.schwarz);
    assert!(value.schneider, "Schwarz schließt Schneider ein");
    // mit 2, Spiel 3, Schneider 4, Schwarz 5
    assert_eq!(value.levels, 5);
}

// Skatordnung: a declarer without a trick is Schwarz and therefore also
// Schneider. With card mods (e.g. "Fett", DESIGN 7.2) the skat can hold
// more than 30 Augen; the Schneider level must still count.
#[test]
fn declarer_schwarz_implies_declarer_schneider() {
    let outcome = Outcome {
        declarer_card_points: 31,
        defender_card_points: 97,
        declarer_tricks: 0,
        defender_tricks: 10,
    };
    let decl = Declaration::grand(RiskLevel::Normal);
    let value = game_value(decl, MIT_2, &outcome);
    assert!(!value.won);
    // mit 2, Spiel 3, Schneider 4, Schwarz 5
    assert_eq!(value.levels, 5);
}

/// Wave 2 relics that reach into the formula (DESIGN 8.1), worked out by
/// hand: "Stichfest" moves only the Schneider line, "Stammfarbe" scales
/// the Spielwert by 1.5 and rounds half up at the end.
#[test]
fn relic_schneider_bonus_and_fractional_factor_vectors() {
    use skat_core::content::{Change, Factor, Lever, RelicId, ScoreLine, Source, score_game};
    use skat_core::rules::{Thresholds, game_value_parts};

    let mit_2 = Matadors {
        with: true,
        count: 2,
    };
    let outcome = |declarer: u32, tricks: u32| Outcome {
        declarer_card_points: declarer,
        defender_card_points: TOTAL_CARD_POINTS - declarer,
        declarer_tricks: tricks,
        defender_tricks: 10 - tricks,
    };
    // Kreuz mit 2, first trick 14 Augen: 76 Augen (defenders 44) are
    // Schneider, 12 × (2 + 1 + 1) = 48; 75 Augen are not, 12 × 3 = 36.
    let kreuz = Declaration::suit(Suit::Clubs, RiskLevel::Normal);
    let stichfest = Thresholds {
        schneider_bonus: 14,
        ..Thresholds::default()
    };
    let parts = game_value_parts(kreuz, mit_2, &outcome(76, 7), &stichfest);
    assert_eq!(
        (parts.won, parts.schneider, parts.value.value),
        (true, true, 48)
    );
    let parts = game_value_parts(kreuz, mit_2, &outcome(75, 7), &stichfest);
    assert_eq!(
        (parts.won, parts.schneider, parts.value.value),
        (true, false, 36)
    );
    // 60 Augen stay lost: −2 × 36.
    let parts = game_value_parts(kreuz, mit_2, &outcome(60, 5), &stichfest);
    assert_eq!((parts.won, parts.value.score), (false, -72));

    // Karo mit 2, Spiel 3: 9 × 3 = 27, × 1.5 = 40.5 → 41; lost −82.
    let karo = Declaration::suit(Suit::Diamonds, RiskLevel::Normal);
    let scale = vec![ScoreLine {
        source: Source::Relic(RelicId::new("home_suit")),
        lever: Lever::GameValue,
        change: Change::Scale(Factor::from_permille(1500)),
    }];
    let won = score_game(
        karo,
        mit_2,
        &outcome(70, 6),
        &Thresholds::default(),
        scale.clone(),
    );
    assert_eq!((won.value, won.score), (41, 41));
    let lost = score_game(karo, mit_2, &outcome(50, 4), &Thresholds::default(), scale);
    assert_eq!((lost.value, lost.score), (41, -82));
}

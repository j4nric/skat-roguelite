//! Rules of a single Skat game: game types, base values, trump order, trick
//! resolution, following suit, matadors and the game value formula.
//!
//! The golden test vectors for the formula live in
//! `core/tests/game_value_vectors.rs`.

use std::fmt;

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

    /// Human readable name used by the simulator and logs.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            GameType::Suit(Suit::Diamonds) => "Diamonds",
            GameType::Suit(Suit::Hearts) => "Hearts",
            GameType::Suit(Suit::Spades) => "Spades",
            GameType::Suit(Suit::Clubs) => "Clubs",
            GameType::Grand => "Grand",
            GameType::Null => "Null",
        }
    }
}

impl fmt::Display for GameType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
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

// ---------------------------------------------------------------------------
// Trick resolution
// ---------------------------------------------------------------------------

/// Whether `card` is a trump in `game`. In a suit game the jacks belong to
/// the trump, not to their printed suit.
#[must_use]
pub const fn is_trump(game: GameType, card: Card) -> bool {
    match game {
        GameType::Suit(trump) => card.is_jack() || card.suit as u8 == trump as u8,
        GameType::Grand => card.is_jack(),
        GameType::Null => false,
    }
}

/// Position of a jack inside the jack order (Kreuz highest).
const fn jack_strength(suit: Suit) -> u8 {
    match suit {
        Suit::Clubs => 4,
        Suit::Spades => 3,
        Suit::Hearts => 2,
        Suit::Diamonds => 1,
    }
}

/// Strength of a trump card; higher wins. `None` if the card is no trump.
#[must_use]
pub const fn trump_strength(game: GameType, card: Card) -> Option<u8> {
    if !is_trump(game, card) {
        return None;
    }
    if card.is_jack() {
        // Jacks sit above the seven suit trumps (strength 1..=7).
        Some(7 + jack_strength(card.suit))
    } else {
        Some(card.rank.suit_game_strength())
    }
}

/// Whether `card` follows the lead card, i.e. is of the same kind: both
/// trump, or both non-trump of the same suit. This is the Bedienpflicht
/// test; jacks follow a trump lead in suit and grand games, never a suit.
#[must_use]
pub const fn follows_lead(game: GameType, lead: Card, card: Card) -> bool {
    match (is_trump(game, lead), is_trump(game, card)) {
        (true, true) => true,
        (false, false) => card.suit as u8 == lead.suit as u8,
        _ => false,
    }
}

/// Whether `challenger`, played after `holder` in a trick led with `lead`,
/// takes the trick away from `holder`.
///
/// Trump beats non-trump, higher trump beats lower trump, and among
/// non-trumps only a card of the led suit can win, the higher rank taking
/// it. In Null games the natural rank order applies.
#[must_use]
pub fn beats(game: GameType, lead: Card, holder: Card, challenger: Card) -> bool {
    match (
        trump_strength(game, holder),
        trump_strength(game, challenger),
    ) {
        (_, None) if !follows_lead(game, lead, challenger) => false,
        (Some(h), Some(c)) => c > h,
        (None, Some(_)) => true,
        (Some(_), None) => false,
        (None, None) => {
            if !follows_lead(game, lead, holder) {
                return true;
            }
            match game {
                GameType::Null => challenger.rank > holder.rank,
                _ => challenger.rank.suit_game_strength() > holder.rank.suit_game_strength(),
            }
        }
    }
}

/// Index of the card currently winning a (possibly incomplete) trick.
/// `cards[0]` is the lead. `None` for an empty trick.
#[must_use]
pub fn winning_index(game: GameType, cards: &[Card]) -> Option<usize> {
    let lead = *cards.first()?;
    let mut best = 0;
    for (i, &card) in cards.iter().enumerate().skip(1) {
        if beats(game, lead, cards[best], card) {
            best = i;
        }
    }
    Some(best)
}

/// Seat index (0..3) that wins a complete trick. `cards[0]` was played by
/// `lead_seat`, `cards[1]` by the next seat and so on.
#[must_use]
pub fn trick_winner(game: GameType, lead_seat: usize, cards: [Card; 3]) -> usize {
    // A three-card slice always has a winning index.
    let offset = winning_index(game, &cards).unwrap_or(0);
    (lead_seat + offset) % 3
}

/// The cards from `hand` that may legally be played. With no lead every
/// card is allowed. Otherwise the lead must be followed (trump with trump,
/// a suit with that suit); whoever cannot follow may play anything.
#[must_use]
pub fn legal_moves(game: GameType, hand: &[Card], lead: Option<Card>) -> Vec<Card> {
    let Some(lead) = lead else {
        return hand.to_vec();
    };
    let following: Vec<Card> = hand
        .iter()
        .copied()
        .filter(|&c| follows_lead(game, lead, c))
        .collect();
    if following.is_empty() {
        hand.to_vec()
    } else {
        following
    }
}

// ---------------------------------------------------------------------------
// Matadors
// ---------------------------------------------------------------------------

/// Spitzen: how many consecutive top trumps the declarer holds ("mit") or
/// lacks ("ohne"), counted from the Kreuz-Bube downwards.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Matadors {
    /// `true` = "mit", `false` = "ohne".
    pub with: bool,
    /// Number of matadors (0 only for Null games).
    pub count: u32,
}

/// Count the matadors over the declarer's twelve cards (hand plus skat).
/// Null games have no trumps and report `with: false, count: 0`.
#[must_use]
pub fn matadors(game: GameType, declarer_cards: &[Card]) -> Matadors {
    let order = game.trump_order();
    let Some(top) = order.first() else {
        return Matadors {
            with: false,
            count: 0,
        };
    };
    let with = declarer_cards.contains(top);
    let count = order
        .iter()
        .take_while(|c| declarer_cards.contains(c) == with)
        .count();
    Matadors {
        with,
        // At most eleven trumps exist, so this always fits.
        count: u32::try_from(count).unwrap_or(u32::MAX),
    }
}

// ---------------------------------------------------------------------------
// Declaration and game value
// ---------------------------------------------------------------------------

/// Risikostufe of a declaration. The levels are cumulative as in real Skat:
/// every level above `Normal` is a hand game, `SchwarzDeclared` includes
/// `SchneiderDeclared`, and `Ouvert` includes all of them.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum RiskLevel {
    /// Skat is picked up, no announcement.
    Normal,
    /// Skat stays on the table.
    Hand,
    /// Hand, Schneider announced.
    SchneiderDeclared,
    /// Hand, Schneider and Schwarz announced.
    SchwarzDeclared,
    /// Hand, Schneider and Schwarz announced, cards face up.
    Ouvert,
}

impl RiskLevel {
    /// All risk levels, lowest first.
    pub const ALL: [RiskLevel; 5] = [
        RiskLevel::Normal,
        RiskLevel::Hand,
        RiskLevel::SchneiderDeclared,
        RiskLevel::SchwarzDeclared,
        RiskLevel::Ouvert,
    ];

    /// Human readable name.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            RiskLevel::Normal => "Normal",
            RiskLevel::Hand => "Hand",
            RiskLevel::SchneiderDeclared => "SchneiderDeclared",
            RiskLevel::SchwarzDeclared => "SchwarzDeclared",
            RiskLevel::Ouvert => "Ouvert",
        }
    }
}

/// Ansage: game type plus risk level, chosen before the skat is touched.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Declaration {
    /// Spielart.
    pub game: GameType,
    /// Risikostufe.
    pub risk: RiskLevel,
}

impl Declaration {
    /// Create a declaration.
    #[must_use]
    pub const fn new(game: GameType, risk: RiskLevel) -> Declaration {
        Declaration { game, risk }
    }

    /// Whether the skat stays on the table.
    #[must_use]
    pub fn hand(self) -> bool {
        self.risk >= RiskLevel::Hand
    }

    /// Whether Schneider was announced.
    #[must_use]
    pub fn schneider_declared(self) -> bool {
        self.risk >= RiskLevel::SchneiderDeclared
    }

    /// Whether Schwarz was announced.
    #[must_use]
    pub fn schwarz_declared(self) -> bool {
        self.risk >= RiskLevel::SchwarzDeclared
    }

    /// Whether the declarer plays with open cards.
    #[must_use]
    pub fn ouvert(self) -> bool {
        self.risk >= RiskLevel::Ouvert
    }
}

/// Result of the ten tricks, as needed for scoring.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Outcome {
    /// Augen of the declarer, including the skat (discarded or untouched).
    pub declarer_card_points: u32,
    /// Augen of both defenders together.
    pub defender_card_points: u32,
    /// Tricks won by the declarer.
    pub declarer_tricks: u32,
    /// Tricks won by the defenders together.
    pub defender_tricks: u32,
}

/// Spielwert of a finished game.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GameValue {
    /// Gewinnstufen (0 for Null games, whose value is fixed).
    pub levels: u32,
    /// Grundwert × Gewinnstufen, or the fixed Null value. Always positive.
    pub value: u32,
    /// Whether the declarer won.
    pub won: bool,
    /// Whether the declarer made the defenders Schneider (suit and grand).
    pub schneider: bool,
    /// Whether the declarer made the defenders Schwarz (suit and grand).
    pub schwarz: bool,
    /// Signed result: `value` if won, `-2 × value` if lost.
    pub score: i32,
}

impl GameValue {
    fn from_value(levels: u32, value: u32, won: bool, schneider: bool, schwarz: bool) -> GameValue {
        // Game values are far below i32::MAX; saturate instead of panicking.
        let signed = i32::try_from(value).unwrap_or(i32::MAX);
        let score = if won {
            signed
        } else {
            signed.saturating_mul(-2)
        };
        GameValue {
            levels,
            value,
            won,
            schneider,
            schwarz,
            score,
        }
    }
}

/// Value of a Null game. Won if the declarer took no trick.
#[must_use]
pub fn null_game_value(hand: bool, ouvert: bool, outcome: &Outcome) -> GameValue {
    let won = outcome.declarer_tricks == 0;
    GameValue::from_value(0, null_value(hand, ouvert), won, false, false)
}

/// Compute the Spielwert from the declaration, the matadors (over hand plus
/// skat) and the outcome of the tricks.
///
/// Suit and grand games: won from 61 Augen; Schneider from 90 Augen;
/// Schwarz when the defenders took no trick. A declared level that was not
/// reached loses the game. Each declared level still counts in the levels,
/// as does Hand. A lost game scores `-2 × value`. If the declarer loses and
/// is Schneider (at most 30 Augen) or Schwarz (no trick) himself, those
/// levels are added as in the Skatordnung.
#[must_use]
pub fn game_value(declaration: Declaration, matadors: Matadors, outcome: &Outcome) -> GameValue {
    let Some(base) = declaration.game.base_value() else {
        return null_game_value(declaration.hand(), declaration.ouvert(), outcome);
    };
    let hand = declaration.hand();
    let schneider_declared = declaration.schneider_declared();
    let schwarz_declared = declaration.schwarz_declared();
    let ouvert = declaration.ouvert();

    let schneider = outcome.declarer_card_points >= SCHNEIDER_THRESHOLD;
    let schwarz = outcome.defender_tricks == 0;
    let won = outcome.declarer_card_points >= WIN_THRESHOLD
        && (!schneider_declared || schneider)
        && (!schwarz_declared || schwarz);

    let declarer_schneider =
        !won && outcome.declarer_card_points <= TOTAL_CARD_POINTS - SCHNEIDER_THRESHOLD;
    let declarer_schwarz = !won && outcome.declarer_tricks == 0;

    let schneider_level = schneider || schneider_declared || declarer_schneider;
    let schwarz_level = schwarz || schwarz_declared || declarer_schwarz;

    let levels = matadors.count
        + 1
        + u32::from(hand)
        + u32::from(schneider_level)
        + u32::from(schneider_declared)
        + u32::from(schwarz_level)
        + u32::from(schwarz_declared)
        + u32::from(ouvert);
    GameValue::from_value(levels, base * levels, won, schneider, schwarz)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cards::parse_cards;

    fn c(s: &str) -> Card {
        s.parse().expect("test card")
    }

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

    #[test]
    fn trump_strength_matches_trump_order() {
        for game in [GameType::Suit(Suit::Spades), GameType::Grand] {
            let order = game.trump_order();
            for pair in order.windows(2) {
                assert!(trump_strength(game, pair[0]) > trump_strength(game, pair[1]));
            }
        }
        assert_eq!(trump_strength(GameType::Grand, c("SA")), None);
        assert_eq!(trump_strength(GameType::Null, c("CJ")), None);
    }

    // Trick rule: trump beats a card of the led suit.
    #[test]
    fn trump_beats_led_suit() {
        let game = GameType::Suit(Suit::Hearts);
        assert!(beats(game, c("SA"), c("SA"), c("H7")));
        assert!(!beats(game, c("SA"), c("H7"), c("SA")));
    }

    // Trick rule: the jack is the highest trump, above the trump ace.
    #[test]
    fn jack_beats_trump_ace() {
        let game = GameType::Suit(Suit::Hearts);
        assert!(beats(game, c("HA"), c("HA"), c("DJ")));
        assert!(!beats(game, c("DJ"), c("DJ"), c("HA")));
        assert!(beats(game, c("DJ"), c("DJ"), c("CJ")));
    }

    // Trick rule: without trump the highest card of the led suit wins and
    // a card of another suit never wins.
    #[test]
    fn led_suit_wins_without_trump() {
        let game = GameType::Suit(Suit::Clubs);
        assert!(beats(game, c("S7"), c("S7"), c("ST")));
        assert!(!beats(game, c("S7"), c("ST"), c("SK")));
        assert!(!beats(game, c("S7"), c("S7"), c("HA")));
        // Ten beats king in suit games, ace beats ten.
        assert!(beats(game, c("SK"), c("SK"), c("ST")));
        assert!(beats(game, c("ST"), c("ST"), c("SA")));
    }

    // Trick rule: blanks (Luschen) rank 7 < 8 < 9.
    #[test]
    fn blank_against_blank_by_rank() {
        let game = GameType::Grand;
        assert!(beats(game, c("D7"), c("D7"), c("D8")));
        assert!(beats(game, c("D7"), c("D8"), c("D9")));
        assert!(!beats(game, c("D7"), c("D9"), c("D8")));
    }

    // Trick rule: in a suit game the jack is a trump, not a card of its suit.
    #[test]
    fn jack_is_not_part_of_its_suit() {
        let game = GameType::Suit(Suit::Hearts);
        // Spade jack on a spade lead ruffs, it does not merely follow.
        assert!(beats(game, c("SA"), c("SA"), c("SJ")));
        assert!(!follows_lead(game, c("SA"), c("SJ")));
        assert!(follows_lead(game, c("HA"), c("SJ")));
        assert!(follows_lead(game, c("SJ"), c("H7")));
    }

    // Null: natural order, no trumps.
    #[test]
    fn null_uses_natural_rank_order() {
        let game = GameType::Null;
        assert!(beats(game, c("H9"), c("H9"), c("HT")));
        assert!(beats(game, c("HT"), c("HT"), c("HJ")));
        assert!(!beats(game, c("H7"), c("H7"), c("CJ")));
    }

    #[test]
    fn trick_winner_maps_offset_to_seat() {
        let game = GameType::Suit(Suit::Diamonds);
        // Seat 2 leads S7, seat 0 plays SA, seat 1 ruffs with D7.
        assert_eq!(trick_winner(game, 2, [c("S7"), c("SA"), c("D7")]), 1);
        assert_eq!(trick_winner(game, 1, [c("CA"), c("C7"), c("C8")]), 1);
        assert_eq!(trick_winner(game, 0, [c("H7"), c("H8"), c("H9")]), 2);
    }

    // Bedienpflicht: a jack must not be used to follow a suit lead; it
    // must follow a trump lead.
    #[test]
    fn following_suit_rules_for_jacks() {
        let game = GameType::Suit(Suit::Clubs);
        let hand = parse_cards("SJ HA H7 CK").expect("hand");
        // Hearts led: only the hearts may be played.
        let legal = legal_moves(game, &hand, Some(c("HT")));
        assert_eq!(legal, parse_cards("HA H7").expect("cards"));
        // Spades led, no spade in hand (SJ is trump): anything goes.
        let legal = legal_moves(game, &hand, Some(c("S8")));
        assert_eq!(legal.len(), 4);
        // Trump led: jack and club king must follow.
        let legal = legal_moves(game, &hand, Some(c("C7")));
        assert_eq!(legal, parse_cards("SJ CK").expect("cards"));
        // Jack led: same.
        let legal = legal_moves(game, &hand, Some(c("DJ")));
        assert_eq!(legal, parse_cards("SJ CK").expect("cards"));
        // No lead: everything.
        assert_eq!(legal_moves(game, &hand, None).len(), 4);
    }

    #[test]
    fn grand_jack_lead_must_be_followed_with_jack() {
        let hand = parse_cards("HJ CA C7").expect("hand");
        let legal = legal_moves(GameType::Grand, &hand, Some(c("DJ")));
        assert_eq!(legal, vec![c("HJ")]);
        let legal = legal_moves(GameType::Grand, &hand, Some(c("CK")));
        assert_eq!(legal, parse_cards("CA C7").expect("cards"));
    }

    fn outcome(declarer_card_points: u32, declarer_tricks: u32) -> Outcome {
        Outcome {
            declarer_card_points,
            defender_card_points: TOTAL_CARD_POINTS - declarer_card_points,
            declarer_tricks,
            defender_tricks: 10 - declarer_tricks,
        }
    }

    const MIT_2: Matadors = Matadors {
        with: true,
        count: 2,
    };

    #[test]
    fn lost_game_scores_double_negative() {
        let decl = Declaration::new(GameType::Suit(Suit::Clubs), RiskLevel::Normal);
        let lost = game_value(decl, MIT_2, &outcome(60, 5));
        assert!(!lost.won);
        assert_eq!(lost.levels, 3);
        assert_eq!(lost.value, 36);
        assert_eq!(lost.score, -72);
        let won = game_value(decl, MIT_2, &outcome(61, 5));
        assert!(won.won);
        assert_eq!(won.score, 36);
    }

    #[test]
    fn schneider_and_schwarz_add_levels_when_reached() {
        let decl = Declaration::new(GameType::Suit(Suit::Hearts), RiskLevel::Normal);
        let v = game_value(decl, MIT_2, &outcome(90, 8));
        assert!(v.won && v.schneider && !v.schwarz);
        assert_eq!(v.levels, 4);
        assert_eq!(v.value, 40);
        let v = game_value(decl, MIT_2, &outcome(120, 10));
        assert!(v.won && v.schneider && v.schwarz);
        assert_eq!(v.levels, 5);
        assert_eq!(v.value, 50);
        let v = game_value(decl, MIT_2, &outcome(89, 9));
        assert!(!v.schneider);
        assert_eq!(v.levels, 3);
    }

    // A declared level that is not reached loses the game, and the declared
    // levels still count (doubled as a loss).
    #[test]
    fn declared_level_not_reached_is_lost() {
        let decl = Declaration::new(GameType::Grand, RiskLevel::SchneiderDeclared);
        let v = game_value(decl, MIT_2, &outcome(85, 7));
        assert!(!v.won);
        // mit 2, Spiel 3, Hand 4, Schneider 5, Schneider angesagt 6.
        assert_eq!(v.levels, 6);
        assert_eq!(v.value, 144);
        assert_eq!(v.score, -288);
        let v = game_value(decl, MIT_2, &outcome(90, 7));
        assert!(v.won);
        assert_eq!(v.score, 144);

        let decl = Declaration::new(GameType::Grand, RiskLevel::SchwarzDeclared);
        let v = game_value(decl, MIT_2, &outcome(110, 9));
        assert!(!v.won);
        // mit 2, Spiel 3, Hand 4, Schneider 5, Schneider angesagt 6,
        // Schwarz 7, Schwarz angesagt 8.
        assert_eq!(v.levels, 8);
        let v = game_value(decl, MIT_2, &outcome(120, 10));
        assert!(v.won);
        assert_eq!(v.levels, 8);

        let decl = Declaration::new(GameType::Suit(Suit::Clubs), RiskLevel::Ouvert);
        let v = game_value(decl, MIT_2, &outcome(120, 10));
        assert!(v.won);
        assert_eq!(v.levels, 9);
        assert_eq!(v.value, 108);
        let v = game_value(decl, MIT_2, &outcome(118, 9));
        assert!(!v.won);
        assert_eq!(v.score, -216);
    }

    // Skatordnung: a declarer who loses Schneider or Schwarz pays those
    // levels too.
    #[test]
    fn declarer_losing_schneider_or_schwarz_pays_extra_levels() {
        let decl = Declaration::new(GameType::Suit(Suit::Diamonds), RiskLevel::Normal);
        let v = game_value(decl, MIT_2, &outcome(30, 2));
        assert!(!v.won && !v.schneider);
        assert_eq!(v.levels, 4);
        assert_eq!(v.score, -72);
        let v = game_value(decl, MIT_2, &outcome(31, 2));
        assert_eq!(v.levels, 3);
        let v = game_value(decl, MIT_2, &outcome(0, 0));
        assert_eq!(v.levels, 5);
        assert_eq!(v.score, -90);
    }

    #[test]
    fn null_game_won_without_tricks() {
        let decl = Declaration::new(GameType::Null, RiskLevel::Normal);
        let v = game_value(
            decl,
            matadors(GameType::Null, &[]),
            &Outcome {
                declarer_tricks: 0,
                defender_tricks: 10,
                ..Outcome::default()
            },
        );
        assert!(v.won);
        assert_eq!((v.levels, v.value, v.score), (0, 23, 23));
        let v = game_value(
            Declaration::new(GameType::Null, RiskLevel::Hand),
            matadors(GameType::Null, &[]),
            &Outcome {
                declarer_tricks: 1,
                defender_tricks: 9,
                ..Outcome::default()
            },
        );
        assert!(!v.won);
        assert_eq!(v.score, -70);
        assert_eq!(null_game_value(false, true, &Outcome::default()).value, 46);
    }

    #[test]
    fn risk_levels_are_cumulative() {
        let d = |risk| Declaration::new(GameType::Grand, risk);
        assert!(!d(RiskLevel::Normal).hand());
        assert!(d(RiskLevel::Hand).hand());
        assert!(!d(RiskLevel::Hand).schneider_declared());
        assert!(d(RiskLevel::SchneiderDeclared).hand());
        assert!(d(RiskLevel::SchwarzDeclared).schneider_declared());
        assert!(d(RiskLevel::Ouvert).schwarz_declared());
        assert!(!d(RiskLevel::SchwarzDeclared).ouvert());
    }
}

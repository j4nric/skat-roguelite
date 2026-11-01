//! Rules of a single Skat game: game types, base values, trump order, trick
//! resolution, following suit, matadors and the game value formula.
//!
//! The golden test vectors for the formula live in
//! `core/tests/game_value_vectors.rs`.

use std::fmt;

use serde::{Deserialize, Serialize};

use crate::cards::{Card, PlayingCard, Rank, Suit};

/// The game the declarer (Alleinspieler) announces.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
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
/// Augen from which the opponents are Schneider in a standard 120-Augen deck.
pub const SCHNEIDER_THRESHOLD: u32 = 90;
/// A side with at most this many Augen is Schneider (Skatordnung).
pub const SCHNEIDER_LIMIT: u32 = 30;
/// Augen in the whole deck.
pub const TOTAL_CARD_POINTS: u32 = 120;

// ---------------------------------------------------------------------------
// Trick resolution
// ---------------------------------------------------------------------------

/// Whether a printed card is a trump in `game`. In a suit game the jacks
/// belong to the trump, not to their printed suit.
const fn face_is_trump(game: GameType, card: Card) -> bool {
    match game {
        GameType::Suit(trump) => card.is_jack() || card.suit as u8 == trump as u8,
        GameType::Grand => card.is_jack(),
        GameType::Null => false,
    }
}

/// Whether `card` is a trump in `game`. In a suit game the jacks belong to
/// the trump, not to their printed suit. Blanks are never trump.
#[must_use]
pub fn is_trump<C: PlayingCard>(game: GameType, card: C) -> bool {
    card.face().is_some_and(|face| face_is_trump(game, face))
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
///
/// Modified cards ("Gezinkt") count [`PlayingCard::rank_bonus`] steps
/// higher. A modified card that reaches the strength of another card does
/// not beat it: on equal strength the card played earlier keeps the trick.
#[must_use]
pub fn trump_strength<C: PlayingCard>(game: GameType, card: C) -> Option<u8> {
    let face = card.face()?;
    if !face_is_trump(game, face) {
        return None;
    }
    let base = if face.is_jack() {
        // Jacks sit above the seven suit trumps (strength 1..=7).
        7 + jack_strength(face.suit)
    } else {
        face.rank.suit_game_strength()
    };
    Some(base.saturating_add(card.rank_bonus()))
}

/// Strength of a non-trump card within its suit, including the rank bonus.
/// Null games use the natural rank order. Blanks have strength 0.
#[must_use]
pub fn suit_strength<C: PlayingCard>(game: GameType, card: C) -> u8 {
    let Some(face) = card.face() else {
        return 0;
    };
    let base = match game {
        GameType::Null => face.rank as u8 + 1,
        _ => face.rank.suit_game_strength(),
    };
    base.saturating_add(card.rank_bonus())
}

/// Whether `card` follows the lead card, i.e. is of the same kind: both
/// trump, or both non-trump of the same suit. This is the Bedienpflicht
/// test; jacks follow a trump lead in suit and grand games, never a suit.
/// A blank never follows and can never be followed.
#[must_use]
pub fn follows_lead<C: PlayingCard>(game: GameType, lead: C, card: C) -> bool {
    let (Some(lead), Some(card)) = (lead.face(), card.face()) else {
        return false;
    };
    match (face_is_trump(game, lead), face_is_trump(game, card)) {
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
/// it. In Null games the natural rank order applies. A blank never takes a
/// trick, and every other card takes it from a blank.
#[must_use]
pub fn beats<C: PlayingCard>(game: GameType, lead: C, holder: C, challenger: C) -> bool {
    if challenger.is_blank() {
        return false;
    }
    if holder.is_blank() {
        return true;
    }
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
            suit_strength(game, challenger) > suit_strength(game, holder)
        }
    }
}

/// The card that decides which suit must be followed: the first card of
/// the trick that is not a blank. When a blank is led, the next normal
/// card takes over this role. `None` if the trick holds only blanks.
#[must_use]
pub fn effective_lead<C: PlayingCard>(cards: &[C]) -> Option<C> {
    cards.iter().copied().find(|c| !c.is_blank())
}

/// Index of the card currently winning a (possibly incomplete) trick.
/// `cards[0]` is the lead. `None` for an empty trick. A trick of blanks
/// only goes to the lead.
#[must_use]
pub fn winning_index<C: PlayingCard>(game: GameType, cards: &[C]) -> Option<usize> {
    let first = *cards.first()?;
    let lead = effective_lead(cards).unwrap_or(first);
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
pub fn trick_winner<C: PlayingCard>(game: GameType, lead_seat: usize, cards: [C; 3]) -> usize {
    // A three-card slice always has a winning index.
    let offset = winning_index(game, &cards).unwrap_or(0);
    (lead_seat + offset) % 3
}

/// The cards from `hand` that may legally be played. `lead` is the
/// [`effective_lead`] of the current trick. With no lead every card is
/// allowed. Otherwise the lead must be followed (trump with trump, a suit
/// with that suit); whoever cannot follow may play anything. Blanks may
/// always be played.
#[must_use]
pub fn legal_moves<C: PlayingCard>(game: GameType, hand: &[C], lead: Option<C>) -> Vec<C> {
    let Some(lead) = lead else {
        return hand.to_vec();
    };
    let can_follow = hand.iter().any(|&c| follows_lead(game, lead, c));
    if can_follow {
        hand.iter()
            .copied()
            .filter(|&c| c.is_blank() || follows_lead(game, lead, c))
            .collect()
    } else {
        hand.to_vec()
    }
}

// ---------------------------------------------------------------------------
// Matadors
// ---------------------------------------------------------------------------

/// Spitzen: how many consecutive top trumps the declarer holds ("mit") or
/// lacks ("ohne"), counted from the Kreuz-Bube downwards.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
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

/// Spitzen over card instances, counted only along the trumps that are in
/// play in this game (`in_play`: hands plus skat).
///
/// Roguelike rule: a trump that was removed from the deck, turned into a
/// blank (boss "Die Oma") or lies in the talon does not exist in this game
/// and is skipped. Otherwise removing the Kreuz-Bube would hand every game
/// a free "ohne" level. With the plain 32-card deck every trump is in play
/// and the result equals [`matadors`].
#[must_use]
pub fn matadors_in<C: PlayingCard>(
    game: GameType,
    declarer_cards: &[C],
    in_play: &[C],
) -> Matadors {
    let present: Vec<Card> = in_play.iter().filter_map(|c| c.face()).collect();
    let held: Vec<Card> = declarer_cards.iter().filter_map(|c| c.face()).collect();
    let order: Vec<Card> = game
        .trump_order()
        .into_iter()
        .filter(|c| present.contains(c))
        .collect();
    let Some(top) = order.first() else {
        return Matadors {
            with: false,
            count: 0,
        };
    };
    let with = held.contains(top);
    let count = order
        .iter()
        .take_while(|c| held.contains(c) == with)
        .count();
    Matadors {
        with,
        count: u32::try_from(count).unwrap_or(u32::MAX),
    }
}

// ---------------------------------------------------------------------------
// Declaration and game value
// ---------------------------------------------------------------------------

/// Risikostufe of a declaration. The levels are cumulative as in real Skat:
/// every level above `Normal` is a hand game, `SchwarzDeclared` includes
/// `SchneiderDeclared`, and `Ouvert` includes all of them.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
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

/// Thresholds that decide won, Schneider and Schwarz. The default is the
/// Skatordnung; relics move them (design document 4.2, "Schwellen senken").
///
/// Schneider stays measured on the losing side: the defenders are Schneider
/// with at most `schneider_limit` Augen, the declarer with at most
/// [`SCHNEIDER_LIMIT`]. Relics that speak from the declarer's point of view
/// ("Schneider ab 85 Augen", "Jungfrau") never move that limit. They add
/// ranges of declarer Augen in `declarer_schneider` that count as Schneider
/// for a game the declarer wins on points. This keeps the rule meaningful
/// when card modifications change the deck's total Augen, and Schwarz
/// always implies Schneider.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Thresholds {
    /// Augen the declarer needs to win (61).
    pub win: u32,
    /// The defenders are Schneider with at most this many Augen (30).
    pub schneider_limit: u32,
    /// Inclusive ranges of declarer Augen that also count as Schneider.
    pub declarer_schneider: Vec<(u32, u32)>,
}

impl Default for Thresholds {
    fn default() -> Thresholds {
        Thresholds {
            win: WIN_THRESHOLD,
            schneider_limit: SCHNEIDER_LIMIT,
            declarer_schneider: Vec::new(),
        }
    }
}

/// The Gewinnstufen of a suit or grand game, one field per term of the
/// formula, so that a frontend can show where each level comes from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub struct LevelParts {
    /// Spitzen (mit or ohne).
    pub matadors: u32,
    /// +1 for the game itself.
    pub game: u32,
    /// +1 Hand.
    pub hand: u32,
    /// +1 Schneider.
    pub schneider: u32,
    /// +1 Schneider angesagt.
    pub schneider_declared: u32,
    /// +1 Schwarz.
    pub schwarz: u32,
    /// +1 Schwarz angesagt.
    pub schwarz_declared: u32,
    /// +1 Ouvert.
    pub ouvert: u32,
}

impl LevelParts {
    /// Sum of all parts: the Gewinnstufen.
    #[must_use]
    pub const fn total(&self) -> u32 {
        self.matadors
            + self.game
            + self.hand
            + self.schneider
            + self.schneider_declared
            + self.schwarz
            + self.schwarz_declared
            + self.ouvert
    }
}

/// Every intermediate result of the game value formula.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ValueParts {
    /// Grundwert of the game type (0 for Null).
    pub base_value: u32,
    /// Gewinnstufen term by term (all zero for Null).
    pub levels: LevelParts,
    /// Whether the declarer won.
    pub won: bool,
    /// Whether the defenders are Schneider (suit and grand).
    pub schneider: bool,
    /// Whether Schneider was reached only through a declarer range of
    /// [`Thresholds::declarer_schneider`].
    pub schneider_by_declarer_points: bool,
    /// Whether the defenders are Schwarz (suit and grand).
    pub schwarz: bool,
    /// Whether the declarer won on points with fewer Augen than the
    /// Skatordnung requires (a lowered win threshold made the difference).
    pub won_by_lowered_threshold: bool,
    /// The resulting game value without any relic multipliers.
    pub value: GameValue,
}

/// Compute the game value with explicit thresholds and return every
/// intermediate result. [`game_value`] is this function with the default
/// thresholds.
#[must_use]
pub fn game_value_parts(
    declaration: Declaration,
    matadors: Matadors,
    outcome: &Outcome,
    thresholds: &Thresholds,
) -> ValueParts {
    let Some(base) = declaration.game.base_value() else {
        let value = null_game_value(declaration.hand(), declaration.ouvert(), outcome);
        return ValueParts {
            base_value: 0,
            levels: LevelParts::default(),
            won: value.won,
            schneider: false,
            schneider_by_declarer_points: false,
            schwarz: false,
            won_by_lowered_threshold: false,
            value,
        };
    };
    let hand = declaration.hand();
    let schneider_declared = declaration.schneider_declared();
    let schwarz_declared = declaration.schwarz_declared();
    let ouvert = declaration.ouvert();
    let points = outcome.declarer_card_points;
    let won_on_points = points >= thresholds.win;

    let schwarz = outcome.defender_tricks == 0;
    let losing_side_schneider =
        schwarz || outcome.defender_card_points <= thresholds.schneider_limit;
    let declarer_range_schneider = won_on_points
        && thresholds
            .declarer_schneider
            .iter()
            .any(|&(min, max)| (min..=max).contains(&points));
    let schneider = losing_side_schneider || declarer_range_schneider;
    let won = won_on_points && (!schneider_declared || schneider) && (!schwarz_declared || schwarz);

    let declarer_schwarz = !won && outcome.declarer_tricks == 0;
    let declarer_schneider = declarer_schwarz || (!won && points <= SCHNEIDER_LIMIT);

    let schneider_level = schneider || schneider_declared || declarer_schneider;
    let schwarz_level = schwarz || schwarz_declared || declarer_schwarz;

    let levels = LevelParts {
        matadors: matadors.count,
        game: 1,
        hand: u32::from(hand),
        schneider: u32::from(schneider_level),
        schneider_declared: u32::from(schneider_declared),
        schwarz: u32::from(schwarz_level),
        schwarz_declared: u32::from(schwarz_declared),
        ouvert: u32::from(ouvert),
    };
    let total = levels.total();
    ValueParts {
        base_value: base,
        levels,
        won,
        schneider,
        schneider_by_declarer_points: declarer_range_schneider && !losing_side_schneider,
        schwarz,
        won_by_lowered_threshold: won && points < WIN_THRESHOLD,
        value: GameValue::from_value(total, base * total, won, schneider, schwarz),
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
/// Suit and grand games: won from 61 Augen; the defenders are Schneider
/// with at most 30 Augen and Schwarz when they took no trick. Schwarz always
/// implies Schneider, also when card modifications change the deck's total
/// Augen (Skatordnung: Schneider is measured on the losing side). A declared level that was not
/// reached loses the game. Each declared level still counts in the levels,
/// as does Hand. A lost game scores `-2 × value`. If the declarer loses and
/// is Schneider (at most 30 Augen) or Schwarz (no trick) himself, those
/// levels are added as in the Skatordnung.
#[must_use]
pub fn game_value(declaration: Declaration, matadors: Matadors, outcome: &Outcome) -> GameValue {
    game_value_parts(declaration, matadors, outcome, &Thresholds::default()).value
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

    // -----------------------------------------------------------------------
    // Roguelike: blanks, modified cards, matadors in play, thresholds
    // -----------------------------------------------------------------------

    use crate::cards::CardKind;
    use crate::deck::DealtCard;

    fn d(id: u32, s: &str) -> DealtCard {
        DealtCard::plain(id, s.parse::<CardKind>().expect("card kind"))
    }

    #[test]
    fn blank_never_follows_and_never_wins() {
        let game = GameType::Suit(Suit::Hearts);
        let blank = d(40, "XX");
        let ace = d(1, "SA");
        assert!(!follows_lead(game, ace, blank));
        assert!(!is_trump(game, blank));
        assert!(!beats(game, ace, d(2, "S7"), blank));
        // Seat 0 leads S7, seat 1 plays a blank, seat 2 sloughs a club.
        assert_eq!(trick_winner(game, 0, [d(2, "S7"), blank, d(3, "C8")]), 0);
    }

    #[test]
    fn after_a_blank_lead_the_first_normal_card_sets_the_suit() {
        let game = GameType::Grand;
        let cards = [d(40, "XX"), d(1, "S7"), d(2, "SA")];
        assert_eq!(effective_lead(&cards), Some(d(1, "S7")));
        assert_eq!(winning_index(game, &cards), Some(2));
        // A club does not follow the spade that took over the lead.
        assert_eq!(
            winning_index(game, &[d(40, "XX"), d(1, "S7"), d(3, "CA")]),
            Some(1)
        );
        // Three blanks: the lead keeps the trick.
        let blanks = [d(40, "XX"), d(41, "XX"), d(42, "XX")];
        assert_eq!(trick_winner(game, 2, blanks), 2);
    }

    #[test]
    fn blanks_may_always_be_played() {
        let game = GameType::Suit(Suit::Clubs);
        let hand = vec![d(1, "HA"), d(40, "XX"), d(2, "SK")];
        // Hearts led: the heart or the blank.
        assert_eq!(
            legal_moves(game, &hand, Some(d(3, "H7"))),
            vec![d(1, "HA"), d(40, "XX")]
        );
        // Trump led, no trump in hand: anything.
        assert_eq!(legal_moves(game, &hand, Some(d(4, "CJ"))).len(), 3);
    }

    #[test]
    fn rank_bonus_ties_keep_the_earlier_card() {
        let game = GameType::Grand;
        let mut king = d(1, "SK");
        king.rank_bonus = 1;
        let ten = d(2, "ST");
        assert!(
            !beats(game, king, king, ten),
            "equal strength: holder keeps"
        );
        assert!(!beats(game, ten, ten, king));
        let mut jack = d(3, "DJ");
        jack.rank_bonus = 1;
        assert_eq!(trump_strength(game, jack), trump_strength(game, d(4, "HJ")));
    }

    #[test]
    fn matadors_in_play_skip_missing_trumps() {
        let game = GameType::Suit(Suit::Hearts);
        let full: Vec<Card> = crate::cards::standard_deck();
        let held = parse_cards("SJ HJ DJ HA").expect("cards");
        // With the Kreuz-Bube in play: "ohne 1".
        assert_eq!(
            matadors_in(game, &held, &full),
            Matadors {
                with: false,
                count: 1
            }
        );
        assert_eq!(matadors_in(game, &held, &full), matadors(game, &held));
        // Kreuz-Bube removed from the deck: "mit 4".
        let without: Vec<Card> = full.into_iter().filter(|&x| x != c("CJ")).collect();
        assert_eq!(
            matadors_in(game, &held, &without),
            Matadors {
                with: true,
                count: 4
            }
        );
        // No trump in play at all (grand without jacks).
        let no_jacks: Vec<Card> = crate::cards::standard_deck()
            .into_iter()
            .filter(|x| !x.is_jack())
            .collect();
        assert_eq!(matadors_in(GameType::Grand, &held, &no_jacks).count, 0);
    }

    #[test]
    fn lowered_win_threshold_and_declarer_schneider_ranges() {
        let decl = Declaration::new(GameType::Suit(Suit::Clubs), RiskLevel::Normal);
        let thresholds = Thresholds {
            win: 58,
            schneider_limit: SCHNEIDER_LIMIT,
            declarer_schneider: vec![(85, u32::MAX)],
        };
        let parts = game_value_parts(decl, MIT_2, &outcome(59, 5), &thresholds);
        assert!(parts.won && parts.won_by_lowered_threshold);
        assert!(!parts.schneider);
        let parts = game_value_parts(decl, MIT_2, &outcome(86, 7), &thresholds);
        assert!(parts.schneider && parts.schneider_by_declarer_points);
        assert_eq!(parts.levels.schneider, 1);
        assert_eq!(parts.value.levels, 4);
        // Measured on the losing side as before: 90 Augen are Schneider by
        // the rules, not by the range.
        let parts = game_value_parts(decl, MIT_2, &outcome(90, 8), &thresholds);
        assert!(parts.schneider && !parts.schneider_by_declarer_points);
        // Default thresholds give exactly the M1 result.
        for points in [0, 30, 60, 61, 89, 90, 120] {
            let o = outcome(points, points / 12);
            assert_eq!(
                game_value_parts(decl, MIT_2, &o, &Thresholds::default()).value,
                game_value(decl, MIT_2, &o)
            );
        }
    }
}

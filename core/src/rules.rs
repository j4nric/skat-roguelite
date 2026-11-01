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
/// the trump, not to their printed suit, and so does a "Bube ehrenhalber"
/// ([`PlayingCard::honorary_jack`]). Blanks are never trump.
#[must_use]
pub fn is_trump<C: PlayingCard>(game: GameType, card: C) -> bool {
    card.face()
        .is_some_and(|face| face_is_trump(game, face) || acts_as_honorary_jack(game, card))
}

/// Whether `card` plays as a "Bube ehrenhalber" in `game` (DESIGN 4.4):
/// only in a suit game, and a printed jack stays the jack it is.
#[must_use]
pub fn acts_as_honorary_jack<C: PlayingCard>(game: GameType, card: C) -> bool {
    matches!(game, GameType::Suit(_))
        && card.honorary_jack()
        && card.face().is_some_and(|face| !face.is_jack())
}

/// Strength of a "Bube ehrenhalber" before any rank bonus: the strength of
/// the Karo-Bube. It loses the tie against every printed card of the same
/// strength (see [`trump_strength`]).
const HONORARY_JACK_STRENGTH: u8 = 7 + 1;

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
///
/// A "Bube ehrenhalber" ([`PlayingCard::honorary_jack`], suit games only)
/// sits half a step below the Karo-Bube: it has the Karo-Bube's strength
/// but loses to every printed card of equal strength, whichever came
/// first, and beats every card below (the trump ace included). With a rank
/// bonus it climbs whole steps from there. The exact order is
/// [`trump_order_key`]; this strength is what the AI compares.
#[must_use]
pub fn trump_strength<C: PlayingCard>(game: GameType, card: C) -> Option<u8> {
    let face = card.face()?;
    let base = if acts_as_honorary_jack(game, card) {
        HONORARY_JACK_STRENGTH
    } else if !face_is_trump(game, face) {
        return None;
    } else if face.is_jack() {
        // Jacks sit above the seven suit trumps (strength 1..=7).
        7 + jack_strength(face.suit)
    } else {
        face.rank.suit_game_strength()
    };
    Some(base.saturating_add(card.rank_bonus()))
}

/// Exact place of a trump in the trump order, higher wins: the
/// [`trump_strength`] and then 0 for a "Bube ehrenhalber", 1 for every
/// printed trump. `None` if the card is no trump.
#[must_use]
pub fn trump_order_key<C: PlayingCard>(game: GameType, card: C) -> Option<(u8, u8)> {
    let strength = trump_strength(game, card)?;
    Some((strength, u8::from(!acts_as_honorary_jack(game, card))))
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
/// A "Bube ehrenhalber" behaves like a jack in a suit game: it follows a
/// trump lead and never its printed suit. A blank never follows and can
/// never be followed.
#[must_use]
pub fn follows_lead<C: PlayingCard>(game: GameType, lead: C, card: C) -> bool {
    let (Some(lead_face), Some(card_face)) = (lead.face(), card.face()) else {
        return false;
    };
    match (is_trump(game, lead), is_trump(game, card)) {
        (true, true) => true,
        (false, false) => card_face.suit == lead_face.suit,
        _ => false,
    }
}

/// Whether `challenger`, played after `holder` in a trick led with `lead`,
/// takes the trick away from `holder`.
///
/// Trump beats non-trump, higher trump beats lower trump, and among
/// non-trumps only a card of the led suit can win, the higher rank taking
/// it. In Null games the natural rank order applies. A blank never takes a
/// trick, and every other card takes it from a blank. The same holds for a
/// card that can never win ("Unsichtbar", [`PlayingCard::never_wins`]),
/// except that it still has its suit and follows suit.
#[must_use]
pub fn beats<C: PlayingCard>(game: GameType, lead: C, holder: C, challenger: C) -> bool {
    if challenger.is_blank() || challenger.never_wins() {
        return false;
    }
    if holder.is_blank() || holder.never_wins() {
        return true;
    }
    match (
        trump_order_key(game, holder),
        trump_order_key(game, challenger),
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

/// Whether the declarer may put `cards` from his twelve cards (`hand`)
/// into the skat. A card that must not be discarded ("Ramponiert",
/// [`PlayingCard::no_discard`]) may only go when fewer than two other
/// cards are left, and then as few of them as possible.
#[must_use]
pub fn valid_discard<C: PlayingCard>(hand: &[C], cards: [C; 2]) -> bool {
    let free = hand.iter().filter(|c| !c.no_discard()).count();
    let forced = cards.iter().filter(|c| c.no_discard()).count();
    forced <= 2usize.saturating_sub(free)
}

/// Whether `card` from the declarer's twelve cards (`hand`) can go into the
/// skat together with some other card of the hand ([`valid_discard`]): a
/// free card always, a card that must not be discarded only when fewer than
/// two free cards are left.
#[must_use]
pub fn discardable<C: PlayingCard>(hand: &[C], card: C) -> bool {
    hand.contains(&card)
        && hand
            .iter()
            .any(|&other| other != card && valid_discard(hand, [card, other]))
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

/// Risikostufe of a Null game (Nullspieler, DESIGN 9.2). Null has its own
/// levels: Schneider and Schwarz do not exist, and Ouvert does not imply
/// Hand.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum NullRisk {
    /// Skat is picked up (Null, 23).
    Normal,
    /// Skat stays on the table (Null Hand, 35).
    Hand,
    /// Skat is picked up, cards face up (Null Ouvert, 46).
    Ouvert,
    /// Skat stays on the table, cards face up (Null Ouvert Hand, 59).
    OuvertHand,
}

impl NullRisk {
    /// All Null risk levels, lowest first.
    pub const ALL: [NullRisk; 4] = [
        NullRisk::Normal,
        NullRisk::Hand,
        NullRisk::Ouvert,
        NullRisk::OuvertHand,
    ];

    /// Whether the skat stays on the table.
    #[must_use]
    pub const fn hand(self) -> bool {
        matches!(self, NullRisk::Hand | NullRisk::OuvertHand)
    }

    /// Whether the declarer plays with open cards.
    #[must_use]
    pub const fn ouvert(self) -> bool {
        matches!(self, NullRisk::Ouvert | NullRisk::OuvertHand)
    }

    /// The Risikostufe as a number for the Null formula of DESIGN 9.2:
    /// Normal 1, Hand 2, Ouvert 3, Ouvert Hand 4.
    #[must_use]
    pub const fn step(self) -> u32 {
        match self {
            NullRisk::Normal => 1,
            NullRisk::Hand => 2,
            NullRisk::Ouvert => 3,
            NullRisk::OuvertHand => 4,
        }
    }

    /// Human readable name.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            NullRisk::Normal => "Normal",
            NullRisk::Hand => "Hand",
            NullRisk::Ouvert => "Ouvert",
            NullRisk::OuvertHand => "OuvertHand",
        }
    }
}

/// Ansage: what the declarer announces before the skat is touched.
///
/// An enum, so that no invalid combination exists: suit and grand games
/// carry a [`RiskLevel`], Null games a [`NullRisk`], and Ramsch (boss "Der
/// Ramscher") carries nothing. In JSON and RON a declaration keeps the
/// shape `{"game": …, "risk": …}`: `{"game": {"Suit": "Hearts"}, "risk":
/// "Hand"}`, `{"game": "Null", "risk": "OuvertHand"}`, `{"game": "Ramsch",
/// "risk": "Normal"}` (a missing risk reads as `Normal`).
/// Combinations that do not exist (Null with Schneider angesagt, a suit
/// game with `OuvertHand`) are rejected when reading.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(try_from = "DeclarationRepr", into = "DeclarationRepr")]
pub enum Declaration {
    /// Farbspiel.
    Suit {
        /// The trump suit.
        suit: Suit,
        /// Risikostufe.
        risk: RiskLevel,
    },
    /// Grand.
    Grand {
        /// Risikostufe.
        risk: RiskLevel,
    },
    /// Null (only the Nullspieler, ADR 0003).
    Null {
        /// Null-Risikostufe.
        risk: NullRisk,
    },
    /// Ramsch (boss "Der Ramscher"): nobody declares, the jacks are trump
    /// as in a grand, everyone plays for himself and the one with the most
    /// Augen loses.
    Ramsch,
}

impl Declaration {
    /// A suit game.
    #[must_use]
    pub const fn suit(suit: Suit, risk: RiskLevel) -> Declaration {
        Declaration::Suit { suit, risk }
    }

    /// A grand.
    #[must_use]
    pub const fn grand(risk: RiskLevel) -> Declaration {
        Declaration::Grand { risk }
    }

    /// A Null game.
    #[must_use]
    pub const fn null(risk: NullRisk) -> Declaration {
        Declaration::Null { risk }
    }

    /// A suit game or grand from its [`GameType`]. `None` for
    /// [`GameType::Null`], which has its own risk levels
    /// ([`Declaration::null`]).
    #[must_use]
    pub const fn new(game: GameType, risk: RiskLevel) -> Option<Declaration> {
        match game {
            GameType::Suit(suit) => Some(Declaration::Suit { suit, risk }),
            GameType::Grand => Some(Declaration::Grand { risk }),
            GameType::Null => None,
        }
    }

    /// The Spielart that decides the trick rules. Ramsch plays like a grand.
    #[must_use]
    pub const fn game(self) -> GameType {
        match self {
            Declaration::Suit { suit, .. } => GameType::Suit(suit),
            Declaration::Grand { .. } | Declaration::Ramsch => GameType::Grand,
            Declaration::Null { .. } => GameType::Null,
        }
    }

    /// The Risikostufe of a suit game or grand; `None` for Null and Ramsch.
    #[must_use]
    pub const fn risk(self) -> Option<RiskLevel> {
        match self {
            Declaration::Suit { risk, .. } | Declaration::Grand { risk } => Some(risk),
            Declaration::Null { .. } | Declaration::Ramsch => None,
        }
    }

    /// The Null-Risikostufe; `None` unless this is a Null game.
    #[must_use]
    pub const fn null_risk(self) -> Option<NullRisk> {
        match self {
            Declaration::Null { risk } => Some(risk),
            _ => None,
        }
    }

    /// The same Spielart with another Risikostufe. Null and Ramsch stay
    /// unchanged.
    #[must_use]
    pub const fn with_risk(self, risk: RiskLevel) -> Declaration {
        match self {
            Declaration::Suit { suit, .. } => Declaration::Suit { suit, risk },
            Declaration::Grand { .. } => Declaration::Grand { risk },
            other => other,
        }
    }

    /// Whether this is a Null game.
    #[must_use]
    pub const fn is_null(self) -> bool {
        matches!(self, Declaration::Null { .. })
    }

    /// Whether this is a Ramsch.
    #[must_use]
    pub const fn is_ramsch(self) -> bool {
        matches!(self, Declaration::Ramsch)
    }

    /// Whether the skat stays on the table. A Ramsch never picks it up.
    #[must_use]
    pub const fn hand(self) -> bool {
        match self {
            Declaration::Suit { risk, .. } | Declaration::Grand { risk } => {
                risk as u8 >= RiskLevel::Hand as u8
            }
            Declaration::Null { risk } => risk.hand(),
            Declaration::Ramsch => true,
        }
    }

    /// Whether Schneider was announced.
    #[must_use]
    pub const fn schneider_declared(self) -> bool {
        match self.risk() {
            Some(risk) => risk as u8 >= RiskLevel::SchneiderDeclared as u8,
            None => false,
        }
    }

    /// Whether Schwarz was announced.
    #[must_use]
    pub const fn schwarz_declared(self) -> bool {
        match self.risk() {
            Some(risk) => risk as u8 >= RiskLevel::SchwarzDeclared as u8,
            None => false,
        }
    }

    /// Whether the declarer plays with open cards.
    #[must_use]
    pub const fn ouvert(self) -> bool {
        match self {
            Declaration::Suit { risk, .. } | Declaration::Grand { risk } => {
                risk as u8 >= RiskLevel::Ouvert as u8
            }
            Declaration::Null { risk } => risk.ouvert(),
            Declaration::Ramsch => false,
        }
    }
}

/// Spielart as written in a declaration: [`GameType`] plus Ramsch.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
enum GameName {
    Suit(Suit),
    Grand,
    Null,
    Ramsch,
}

/// Every risk name a declaration may carry: the [`RiskLevel`]s and the
/// [`NullRisk`]s share their names except `OuvertHand`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
enum RiskName {
    #[default]
    Normal,
    Hand,
    SchneiderDeclared,
    SchwarzDeclared,
    Ouvert,
    OuvertHand,
}

/// Serialized form of a [`Declaration`].
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
struct DeclarationRepr {
    game: GameName,
    #[serde(default)]
    risk: RiskName,
}

impl From<Declaration> for DeclarationRepr {
    fn from(d: Declaration) -> DeclarationRepr {
        let trump = |risk: RiskLevel| match risk {
            RiskLevel::Normal => RiskName::Normal,
            RiskLevel::Hand => RiskName::Hand,
            RiskLevel::SchneiderDeclared => RiskName::SchneiderDeclared,
            RiskLevel::SchwarzDeclared => RiskName::SchwarzDeclared,
            RiskLevel::Ouvert => RiskName::Ouvert,
        };
        match d {
            Declaration::Suit { suit, risk } => DeclarationRepr {
                game: GameName::Suit(suit),
                risk: trump(risk),
            },
            Declaration::Grand { risk } => DeclarationRepr {
                game: GameName::Grand,
                risk: trump(risk),
            },
            Declaration::Null { risk } => DeclarationRepr {
                game: GameName::Null,
                risk: match risk {
                    NullRisk::Normal => RiskName::Normal,
                    NullRisk::Hand => RiskName::Hand,
                    NullRisk::Ouvert => RiskName::Ouvert,
                    NullRisk::OuvertHand => RiskName::OuvertHand,
                },
            },
            Declaration::Ramsch => DeclarationRepr {
                game: GameName::Ramsch,
                risk: RiskName::Normal,
            },
        }
    }
}

impl TryFrom<DeclarationRepr> for Declaration {
    type Error = String;

    fn try_from(repr: DeclarationRepr) -> Result<Declaration, String> {
        let risk = repr.risk;
        let trump = match risk {
            RiskName::Normal => Some(RiskLevel::Normal),
            RiskName::Hand => Some(RiskLevel::Hand),
            RiskName::SchneiderDeclared => Some(RiskLevel::SchneiderDeclared),
            RiskName::SchwarzDeclared => Some(RiskLevel::SchwarzDeclared),
            RiskName::Ouvert => Some(RiskLevel::Ouvert),
            RiskName::OuvertHand => None,
        };
        let invalid = || format!("{risk:?} is no risk level of {:?}", repr.game);
        match repr.game {
            GameName::Suit(suit) => Ok(Declaration::Suit {
                suit,
                risk: trump.ok_or_else(invalid)?,
            }),
            GameName::Grand => Ok(Declaration::Grand {
                risk: trump.ok_or_else(invalid)?,
            }),
            GameName::Null => {
                let risk = match risk {
                    RiskName::Normal => NullRisk::Normal,
                    RiskName::Hand => NullRisk::Hand,
                    RiskName::Ouvert => NullRisk::Ouvert,
                    RiskName::OuvertHand => NullRisk::OuvertHand,
                    RiskName::SchneiderDeclared | RiskName::SchwarzDeclared => {
                        return Err(invalid());
                    }
                };
                Ok(Declaration::Null { risk })
            }
            GameName::Ramsch => match risk {
                RiskName::Normal => Ok(Declaration::Ramsch),
                _ => Err("a Ramsch has no risk level".into()),
            },
        }
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

/// Augen and tricks per seat (index = seat, 0 is the player), for games
/// where the sides do not tell enough: in a Ramsch everyone plays for
/// himself and the one with the most Augen loses.
///
/// Serialized with [`SeatTotals::most`] as `most`, so that a frontend does
/// not compare Augen itself; `most` is ignored when reading and always
/// recomputed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(into = "SeatTotalsRepr", from = "SeatTotalsRepr")]
pub struct SeatTotals {
    /// Augen per seat: tricks, plus the skat for the seat it counts for
    /// (the declarer; in a Ramsch the winner of the last trick), plus
    /// conceded Augen.
    pub card_points: [u32; 3],
    /// Tricks per seat, conceded tricks included.
    pub tricks: [u32; 3],
}

impl SeatTotals {
    /// Which seats have the most Augen (index = seat; several on a tie):
    /// in a Ramsch the losers. The player loses a Ramsch exactly when his
    /// entry is `true` ([`ramsch_value_parts`]).
    #[must_use]
    pub fn most(&self) -> [bool; 3] {
        let max = self.card_points.iter().copied().max().unwrap_or(0);
        self.card_points.map(|p| p == max)
    }
}

/// Serialized form of [`SeatTotals`]: the totals plus who has the most.
#[derive(Serialize, Deserialize)]
struct SeatTotalsRepr {
    card_points: [u32; 3],
    tricks: [u32; 3],
    #[serde(default)]
    most: [bool; 3],
}

impl From<SeatTotals> for SeatTotalsRepr {
    fn from(t: SeatTotals) -> SeatTotalsRepr {
        SeatTotalsRepr {
            card_points: t.card_points,
            tricks: t.tricks,
            most: t.most(),
        }
    }
}

impl From<SeatTotalsRepr> for SeatTotals {
    fn from(t: SeatTotalsRepr) -> SeatTotals {
        SeatTotals {
            card_points: t.card_points,
            tricks: t.tricks,
        }
    }
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
///
/// The stake (Einsatz-Leiter, DESIGN 9.3) raises the win threshold of a
/// run by one Auge per step (61, 62, 63 …); Schneider does not move with
/// it. Relics that lower the threshold lower the raised one by the same
/// amount ([`Thresholds::lower_win`]).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Thresholds {
    /// Augen the declarer needs to win (61, plus the stake).
    pub win: u32,
    /// The defenders are Schneider with at most this many Augen (30).
    pub schneider_limit: u32,
    /// Inclusive ranges of declarer Augen that also count as Schneider.
    pub declarer_schneider: Vec<(u32, u32)>,
    /// Stake of the run: Augen added to [`WIN_THRESHOLD`] before relics.
    #[serde(default)]
    pub stake: u32,
}

impl Default for Thresholds {
    fn default() -> Thresholds {
        Thresholds::with_stake(0)
    }
}

impl Thresholds {
    /// The Skatordnung thresholds with the win threshold raised by
    /// `stake` Augen (`61 + stake`). Schneider stays where it is.
    #[must_use]
    pub const fn with_stake(stake: u32) -> Thresholds {
        Thresholds {
            win: WIN_THRESHOLD.saturating_add(stake),
            schneider_limit: SCHNEIDER_LIMIT,
            declarer_schneider: Vec::new(),
            stake,
        }
    }

    /// Augen needed to win before any relic lowers it: `61 + stake`.
    #[must_use]
    pub const fn unlowered_win(&self) -> u32 {
        WIN_THRESHOLD.saturating_add(self.stake)
    }

    /// Apply a relic that lets the declarer win "from `relic_win` Augen".
    /// Content speaks of the unstaked game, so the relic lowers the
    /// staked threshold by the same amount: `relic_win + stake` (Hufeisen
    /// 56 at stake 2 wins from 58). The lowest threshold counts. Returns
    /// whether the threshold went down.
    pub fn lower_win(&mut self, relic_win: u32) -> bool {
        let win = relic_win.saturating_add(self.stake);
        if win < self.win {
            self.win = win;
            true
        } else {
            false
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
    /// +1 Ouvert (Null formula: +2, see [`NullFormula`]).
    pub ouvert: u32,
    /// Null formula: style bonus for tricks the declarer ducked with a
    /// seven.
    #[serde(default)]
    pub sevens_ducked: u32,
    /// Null formula: style bonus, the declarer never had to follow suit.
    #[serde(default)]
    pub never_followed: u32,
    /// Null formula: style bonus "Kurzschluss".
    #[serde(default)]
    pub short_circuit: u32,
    /// Ramsch: a seat took no trick (Jungfrau).
    #[serde(default)]
    pub virgin: u32,
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
            + self.sevens_ducked
            + self.never_followed
            + self.short_circuit
            + self.virgin
    }
}

/// Every intermediate result of the game value formula.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ValueParts {
    /// Grundwert of the game type. 0 for a Null game scored by the
    /// Skatordnung ([`game_value_parts`]); the run scores Null with
    /// [`null_value_parts`], which gives a Grundwert.
    pub base_value: u32,
    /// Gewinnstufen term by term (all zero for a Null game by the
    /// Skatordnung).
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
    if declaration.is_ramsch() {
        return ramsch_from_outcome(outcome);
    }
    let Some(base) = declaration.game().base_value() else {
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
        ..LevelParts::default()
    };
    let total = levels.total();
    ValueParts {
        base_value: base,
        levels,
        won,
        schneider,
        schneider_by_declarer_points: declarer_range_schneider && !losing_side_schneider,
        schwarz,
        won_by_lowered_threshold: won && points < thresholds.unlowered_win(),
        value: GameValue::from_value(total, base * total, won, schneider, schwarz),
    }
}

/// Value of a Null game. Won if the declarer took no trick.
#[must_use]
pub fn null_game_value(hand: bool, ouvert: bool, outcome: &Outcome) -> GameValue {
    let won = outcome.declarer_tricks == 0;
    GameValue::from_value(0, null_value(hand, ouvert), won, false, false)
}

// ---------------------------------------------------------------------------
// Null in the run (Nullspieler) and Ramsch (boss "Der Ramscher")
// ---------------------------------------------------------------------------

/// How the run scores a Null game of the Nullspieler (DESIGN 9.2). Chosen
/// as data per character in `characters.ron`.
///
/// Both variants go through the same formula as suit games (Grundwert ×
/// Gewinnstufen), so that relics on Grundwert, Gewinnstufen and Spielwert
/// work for Null as well.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NullScoring {
    /// The fixed values of the Skatordnung (23, 35, 46, 59) as Grundwert,
    /// one Gewinnstufe ("Spiel").
    Fixed,
    /// Grundwert × (Risikostufe + Stilbonus), see [`NullFormula`].
    Formula(NullFormula),
}

/// The Null formula of DESIGN 9.2: `Grundwert × (Risikostufe +
/// Stilbonus)`. The Risikostufe ([`NullRisk::step`]) is split into the
/// terms Spiel (1), Hand (+1) and Ouvert (+2). The style bonus only counts
/// for a won game.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NullFormula {
    /// Grundwert (23).
    pub base_value: u32,
    /// Levels per trick in which the declarer played a seven and did not
    /// take the trick ("mit einer 7 unterlaufen").
    pub per_seven_ducked: u32,
    /// Levels if the declarer never had to follow suit to another seat's
    /// lead.
    pub never_followed: u32,
    /// Levels for a "Kurzschluss": the defenders hold at least
    /// `short_circuit_points` Augen after `short_circuit_tricks` tricks.
    pub short_circuit: u32,
    /// Tricks after which the Kurzschluss is checked (5).
    pub short_circuit_tricks: u32,
    /// Augen the defenders need after those tricks (the majority of the
    /// deck's Augen; "alle Augen" cannot fit into 15 cards).
    pub short_circuit_points: u32,
}

/// Style of a Null game as counted by [`crate::game::Game::null_style`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub struct NullStyle {
    /// Tricks the declarer did not take although he played a seven in
    /// them.
    pub sevens_ducked: u32,
    /// Whether the declarer never followed suit to another seat's lead.
    pub never_followed: bool,
    /// Augen the defenders held after the Kurzschluss tricks (`None` if
    /// fewer tricks were played).
    pub defender_points_after: Option<u32>,
}

/// Value of a Null game in the run. Won if the declarer took no trick.
/// A lost game scores `-2 × value` without any style bonus.
#[must_use]
pub fn null_value_parts(
    risk: NullRisk,
    scoring: &NullScoring,
    outcome: &Outcome,
    style: &NullStyle,
) -> ValueParts {
    let won = outcome.declarer_tricks == 0;
    let (base, levels) = match scoring {
        NullScoring::Fixed => (
            null_value(risk.hand(), risk.ouvert()),
            LevelParts {
                game: 1,
                ..LevelParts::default()
            },
        ),
        NullScoring::Formula(f) => {
            let short_circuit = style
                .defender_points_after
                .is_some_and(|p| p >= f.short_circuit_points);
            let levels = LevelParts {
                game: 1,
                hand: u32::from(risk.hand()),
                ouvert: if risk.ouvert() { 2 } else { 0 },
                sevens_ducked: if won {
                    style.sevens_ducked.saturating_mul(f.per_seven_ducked)
                } else {
                    0
                },
                never_followed: if won && style.never_followed {
                    f.never_followed
                } else {
                    0
                },
                short_circuit: if won && short_circuit {
                    f.short_circuit
                } else {
                    0
                },
                ..LevelParts::default()
            };
            (f.base_value.max(1), levels)
        }
    };
    let total = levels.total();
    ValueParts {
        base_value: base,
        levels,
        won,
        schneider: false,
        schneider_by_declarer_points: false,
        schwarz: false,
        won_by_lowered_threshold: false,
        value: GameValue::from_value(total, base.saturating_mul(total), won, false, false),
    }
}

/// Value of a Ramsch for the player (seat 0), from the Augen and tricks of
/// every seat (the skat already added to the seat that took the last
/// trick).
///
/// The seat with the most Augen loses; the player loses also when he
/// shares the most. Spielwert = Augen of the loser (Grundwert) × Gewinnstufen
/// (Spiel 1, +1 Jungfrau if a seat took no trick). Not the loser: won,
/// `+value`; the loser: `-2 × value`.
#[must_use]
pub fn ramsch_value_parts(points: [u32; 3], tricks: [u32; 3]) -> ValueParts {
    let most = points.iter().copied().max().unwrap_or(0);
    let won = points[0] < most;
    let levels = LevelParts {
        game: 1,
        virgin: u32::from(tricks.contains(&0)),
        ..LevelParts::default()
    };
    let base = most.max(1);
    let total = levels.total();
    ValueParts {
        base_value: base,
        levels,
        won,
        schneider: false,
        schneider_by_declarer_points: false,
        schwarz: false,
        won_by_lowered_threshold: false,
        value: GameValue::from_value(total, base.saturating_mul(total), won, false, false),
    }
}

/// A Ramsch judged from an [`Outcome`] alone, which knows only the sum of
/// both defenders: their Augen count as split evenly. The run and the game
/// judge a Ramsch with every seat's Augen ([`ramsch_value_parts`]).
fn ramsch_from_outcome(outcome: &Outcome) -> ValueParts {
    let defenders = outcome.defender_card_points;
    let tricks = [
        outcome.declarer_tricks,
        outcome.defender_tricks / 2,
        outcome.defender_tricks - outcome.defender_tricks / 2,
    ];
    ramsch_value_parts(
        [
            outcome.declarer_card_points,
            defenders / 2,
            defenders - defenders / 2,
        ],
        tricks,
    )
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
        let decl = Declaration::suit(Suit::Clubs, RiskLevel::Normal);
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
        let decl = Declaration::suit(Suit::Hearts, RiskLevel::Normal);
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
        let decl = Declaration::grand(RiskLevel::SchneiderDeclared);
        let v = game_value(decl, MIT_2, &outcome(85, 7));
        assert!(!v.won);
        // mit 2, Spiel 3, Hand 4, Schneider 5, Schneider angesagt 6.
        assert_eq!(v.levels, 6);
        assert_eq!(v.value, 144);
        assert_eq!(v.score, -288);
        let v = game_value(decl, MIT_2, &outcome(90, 7));
        assert!(v.won);
        assert_eq!(v.score, 144);

        let decl = Declaration::grand(RiskLevel::SchwarzDeclared);
        let v = game_value(decl, MIT_2, &outcome(110, 9));
        assert!(!v.won);
        // mit 2, Spiel 3, Hand 4, Schneider 5, Schneider angesagt 6,
        // Schwarz 7, Schwarz angesagt 8.
        assert_eq!(v.levels, 8);
        let v = game_value(decl, MIT_2, &outcome(120, 10));
        assert!(v.won);
        assert_eq!(v.levels, 8);

        let decl = Declaration::suit(Suit::Clubs, RiskLevel::Ouvert);
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
        let decl = Declaration::suit(Suit::Diamonds, RiskLevel::Normal);
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
        let decl = Declaration::null(NullRisk::Normal);
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
            Declaration::null(NullRisk::Hand),
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
        let d = |risk| Declaration::grand(risk);
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
        let decl = Declaration::suit(Suit::Clubs, RiskLevel::Normal);
        let thresholds = Thresholds {
            win: 58,
            schneider_limit: SCHNEIDER_LIMIT,
            declarer_schneider: vec![(85, u32::MAX)],
            stake: 0,
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

    #[test]
    fn the_stake_raises_the_win_threshold_but_not_schneider() {
        let decl = Declaration::suit(Suit::Clubs, RiskLevel::Normal);
        let staked = Thresholds::with_stake(2);
        assert_eq!(staked.win, 63);
        assert_eq!(staked.unlowered_win(), 63);
        assert!(!game_value_parts(decl, MIT_2, &outcome(62, 5), &staked).won);
        let parts = game_value_parts(decl, MIT_2, &outcome(63, 5), &staked);
        assert!(parts.won && !parts.won_by_lowered_threshold && !parts.schneider);
        // Schneider stays at 90 (defenders at most 30).
        assert!(!game_value_parts(decl, MIT_2, &outcome(89, 7), &staked).schneider);
        assert!(game_value_parts(decl, MIT_2, &outcome(90, 8), &staked).schneider);
        // A lost game is still Schneider for the declarer at 30 or less.
        let lost = game_value_parts(decl, MIT_2, &outcome(30, 2), &staked);
        assert!(!lost.won && lost.levels.schneider == 1);
        // Grand alike.
        let grand = Declaration::grand(RiskLevel::Normal);
        assert!(!game_value_parts(grand, MIT_2, &outcome(62, 5), &staked).won);
        assert!(game_value_parts(grand, MIT_2, &outcome(63, 5), &staked).won);
    }

    #[test]
    fn relics_lower_the_staked_threshold_by_their_own_amount() {
        let decl = Declaration::suit(Suit::Clubs, RiskLevel::Normal);
        // Hufeisen (56, five below 61) at stake 2: 58.
        let mut staked = Thresholds::with_stake(2);
        assert!(staked.lower_win(56));
        assert_eq!(staked.win, 58);
        // A weaker relic (58) does not raise it again.
        assert!(!staked.lower_win(58));
        assert_eq!(staked.win, 58);
        let parts = game_value_parts(decl, MIT_2, &outcome(60, 5), &staked);
        assert!(parts.won && parts.won_by_lowered_threshold);
        assert!(!game_value_parts(decl, MIT_2, &outcome(57, 5), &staked).won);
        // Without a stake the relic is what it says.
        let mut plain = Thresholds::default();
        assert!(plain.lower_win(56));
        assert_eq!(plain.win, 56);
        // A save from before the stake reads as stake 0.
        let old = "(win:61,schneider_limit:30,declarer_schneider:[])";
        assert_eq!(
            ron::from_str::<Thresholds>(old).expect("old"),
            Thresholds::default()
        );
    }

    // -----------------------------------------------------------------------
    // M4: Null risk levels, the Null formula, Ramsch
    // -----------------------------------------------------------------------

    #[test]
    fn declarations_keep_their_shape_and_reject_invalid_combinations() {
        let all = [
            Declaration::suit(Suit::Hearts, RiskLevel::Hand),
            Declaration::grand(RiskLevel::Ouvert),
            Declaration::null(NullRisk::Ouvert),
            Declaration::null(NullRisk::OuvertHand),
            Declaration::Ramsch,
        ];
        for d in all {
            let text = ron::to_string(&d).expect("ron");
            assert_eq!(
                ron::from_str::<Declaration>(&text).expect("back"),
                d,
                "{text}"
            );
        }
        assert_eq!(
            ron::to_string(&Declaration::grand(RiskLevel::Hand)).expect("ron"),
            "(game:Grand,risk:Hand)"
        );
        assert_eq!(
            ron::to_string(&Declaration::Ramsch).expect("ron"),
            "(game:Ramsch,risk:Normal)"
        );
        for bad in [
            "(game:Null,risk:SchneiderDeclared)",
            "(game:Suit(Hearts),risk:OuvertHand)",
            "(game:Ramsch,risk:Hand)",
        ] {
            assert!(ron::from_str::<Declaration>(bad).is_err(), "{bad}");
        }
        assert_eq!(Declaration::new(GameType::Null, RiskLevel::Normal), None);
        // Null Ouvert is open without being a hand game.
        let ouvert = Declaration::null(NullRisk::Ouvert);
        assert!(ouvert.ouvert() && !ouvert.hand());
        assert!(!ouvert.schneider_declared());
        assert_eq!(ouvert.game(), GameType::Null);
        assert_eq!(Declaration::Ramsch.game(), GameType::Grand);
        assert!(Declaration::Ramsch.hand());
        let steps: Vec<u32> = NullRisk::ALL.iter().map(|r| r.step()).collect();
        assert_eq!(steps, vec![1, 2, 3, 4]);
    }

    fn formula() -> NullFormula {
        NullFormula {
            base_value: 23,
            per_seven_ducked: 1,
            never_followed: 2,
            short_circuit: 3,
            short_circuit_tricks: 5,
            short_circuit_points: 61,
        }
    }

    #[test]
    fn null_values_by_the_skatordnung_and_by_the_formula() {
        let won = Outcome {
            declarer_tricks: 0,
            defender_tricks: 10,
            ..Outcome::default()
        };
        let lost = Outcome {
            declarer_tricks: 1,
            defender_tricks: 3,
            ..Outcome::default()
        };
        let plain = NullStyle::default();
        for (risk, value) in NullRisk::ALL.into_iter().zip([23, 35, 46, 59]) {
            let p = null_value_parts(risk, &NullScoring::Fixed, &won, &plain);
            assert_eq!(
                (p.base_value, p.value.value, p.value.score),
                (value, value, value as i32)
            );
            let p = null_value_parts(risk, &NullScoring::Formula(formula()), &won, &plain);
            assert_eq!(p.levels.total(), risk.step());
            assert_eq!(p.value.value, 23 * risk.step());
        }
        let style = NullStyle {
            sevens_ducked: 3,
            never_followed: true,
            defender_points_after: Some(70),
        };
        let scoring = NullScoring::Formula(formula());
        let p = null_value_parts(NullRisk::Normal, &scoring, &won, &style);
        // Spiel 1 + 3 sevens + 2 never followed + 3 Kurzschluss.
        assert_eq!(p.levels.total(), 9);
        assert_eq!(p.value.value, 23 * 9);
        // Lost: no style bonus, −2 × 23 × Risikostufe.
        let p = null_value_parts(NullRisk::Hand, &scoring, &lost, &style);
        assert!(!p.won);
        assert_eq!(p.value.score, -2 * 23 * 2);
        let short = NullStyle {
            defender_points_after: Some(60),
            ..NullStyle::default()
        };
        assert_eq!(
            null_value_parts(NullRisk::Normal, &scoring, &won, &short)
                .levels
                .short_circuit,
            0
        );
    }

    #[test]
    fn ramsch_is_lost_by_the_most_augen() {
        let p = ramsch_value_parts([30, 50, 40], [3, 4, 3]);
        assert!(p.won);
        assert_eq!((p.base_value, p.levels.total(), p.value.score), (50, 1, 50));
        let p = ramsch_value_parts([55, 50, 15], [5, 5, 0]);
        assert!(!p.won);
        // Jungfrau: a seat without a trick doubles.
        assert_eq!(p.levels.virgin, 1);
        assert_eq!(p.value.score, -2 * 55 * 2);
        // A shared most counts as lost for the player.
        assert!(!ramsch_value_parts([50, 50, 20], [4, 4, 2]).won);
    }

    #[test]
    fn a_card_that_never_wins_follows_suit_but_loses() {
        let game = GameType::Grand;
        let mut ace = d(1, "SA");
        ace.never_wins = true;
        let seven = d(2, "S7");
        assert!(follows_lead(game, seven, ace));
        assert!(!beats(game, seven, seven, ace));
        assert!(beats(game, ace, ace, seven));
        assert_eq!(trick_winner(game, 0, [ace, seven, d(3, "S8")]), 2);
    }

    // -----------------------------------------------------------------------
    // M5: "Bube ehrenhalber" and "Ramponiert"
    // -----------------------------------------------------------------------

    fn honorary(id: u32, s: &str) -> DealtCard {
        DealtCard {
            honorary_jack: true,
            ..d(id, s)
        }
    }

    // Trumpfordnung: the honorary jack sits between the Karo-Bube and the
    // trump ace, whichever card comes first.
    #[test]
    fn the_honorary_jack_ranks_below_the_jacks_and_above_the_trump_ace() {
        let hearts = GameType::Suit(Suit::Hearts);
        let sk = honorary(1, "SK");
        let (ha, dj) = (d(2, "HA"), d(3, "DJ"));
        assert!(is_trump(hearts, sk));
        assert!(beats(hearts, ha, ha, sk), "beats the trump ace");
        assert!(!beats(hearts, sk, sk, ha));
        assert!(beats(hearts, sk, sk, dj), "loses to the Karo-Bube");
        assert!(!beats(hearts, dj, dj, sk));
        // A trick: S7 led, the honorary SK ruffs, the trump ace comes too
        // late, the Karo-Bube takes it.
        assert_eq!(trick_winner(hearts, 0, [d(4, "S7"), sk, ha]), 1);
        assert_eq!(trick_winner(hearts, 0, [d(4, "S7"), sk, dj]), 2);
        // Of the trump suit itself: climbs above the ace.
        let h7 = honorary(5, "H7");
        assert!(beats(hearts, ha, ha, h7));
        // With a rank bonus a whole step higher: above the Karo-Bube,
        // below the Herz-Bube.
        let mut marked = honorary(6, "SK");
        marked.rank_bonus = 1;
        assert!(beats(hearts, dj, dj, marked));
        assert!(!beats(hearts, d(7, "HJ"), d(7, "HJ"), marked));
        // A gezinkt trump ace keeps reaching the Karo-Bube (tie) and beats
        // the honorary jack.
        let mut ace = d(8, "HA");
        ace.rank_bonus = 1;
        assert!(beats(hearts, sk, sk, ace));
        assert!(!beats(hearts, dj, dj, ace));
        // Two honorary jacks: the earlier keeps the trick.
        let other = honorary(9, "CK");
        assert!(!beats(hearts, sk, sk, other));
        assert!(!beats(hearts, other, other, sk));
    }

    // Grand, Null and Ramsch keep the printed card; a printed jack stays
    // the jack it is.
    #[test]
    fn the_honorary_jack_is_trump_only_in_suit_games() {
        let sk = honorary(1, "SK");
        assert!(!is_trump(GameType::Grand, sk));
        assert!(!is_trump(GameType::Null, sk));
        assert!(follows_lead(GameType::Grand, d(2, "S7"), sk));
        assert!(beats(GameType::Grand, d(3, "S7"), d(3, "S7"), sk));
        let cj = honorary(4, "CJ");
        let hearts = GameType::Suit(Suit::Hearts);
        assert_eq!(trump_order_key(hearts, cj), Some((11, 1)));
        assert_eq!(trump_order_key(hearts, sk), Some((8, 0)));
        assert_eq!(trump_order_key(hearts, d(5, "DJ")), Some((8, 1)));
    }

    // Bedienpflicht: the honorary jack follows trump leads, never its
    // printed suit, and a trump lead forces it out.
    #[test]
    fn the_honorary_jack_follows_trump_not_its_printed_suit() {
        let hearts = GameType::Suit(Suit::Hearts);
        let sk = honorary(1, "SK");
        let hand = vec![sk, d(2, "S8"), d(3, "C7")];
        // Spades led: only the S8 follows; the honorary SK is trump.
        assert_eq!(
            legal_moves(hearts, &hand, Some(d(4, "SA"))),
            vec![d(2, "S8")]
        );
        // Trump led: the honorary SK must follow.
        assert_eq!(legal_moves(hearts, &hand, Some(d(5, "H7"))), vec![sk]);
        assert_eq!(legal_moves(hearts, &hand, Some(d(6, "DJ"))), vec![sk]);
        // Led itself, it asks for trump.
        assert!(follows_lead(hearts, sk, d(7, "H9")));
        assert!(!follows_lead(hearts, sk, d(8, "S9")));
    }

    // Spitzen are counted along the printed trump order: the honorary jack
    // is no extra Spitze and breaks no run.
    #[test]
    fn the_honorary_jack_is_no_spitze() {
        let hearts = GameType::Suit(Suit::Hearts);
        let full: Vec<DealtCard> = crate::cards::standard_deck()
            .into_iter()
            .zip(0u32..)
            .map(|(c, id)| DealtCard::plain(id, CardKind::Normal(c)))
            .collect();
        let held = vec![d(100, "CJ"), d(101, "SJ"), honorary(102, "SK")];
        assert_eq!(
            matadors_in(hearts, &held, &full),
            Matadors {
                with: true,
                count: 2
            }
        );
    }

    #[test]
    fn battered_cards_go_into_the_skat_only_if_nothing_else_can() {
        let battered = |id, s| DealtCard {
            no_discard: true,
            ..d(id, s)
        };
        let (b1, b2) = (battered(1, "S7"), battered(2, "S8"));
        let (c, e) = (d(3, "C7"), d(4, "C8"));
        assert!(valid_discard(&[b1, b2, c, e], [c, e]));
        assert!(!valid_discard(&[b1, b2, c, e], [b1, c]));
        // One free card: it must go, plus one battered card.
        assert!(valid_discard(&[b1, b2, c], [b1, c]));
        assert!(!valid_discard(&[b1, b2, c], [b1, b2]));
        // Only battered cards: any two.
        assert!(valid_discard(&[b1, b2], [b1, b2]));
        // Per card: a battered card only when fewer than two free cards
        // are left.
        assert!(!discardable(&[b1, b2, c, e], b1));
        assert!(discardable(&[b1, b2, c, e], c));
        assert!(discardable(&[b1, b2, c], b1));
        assert!(discardable(&[b1, b2, c], c));
        assert!(discardable(&[b1, b2], b2));
        assert!(!discardable(&[b1, c], e), "not in the hand");
    }

    #[test]
    fn seat_totals_name_who_has_the_most() {
        let totals = SeatTotals {
            card_points: [40, 40, 40],
            tricks: [3, 3, 4],
        };
        assert_eq!(totals.most(), [true, true, true]);
        let totals = SeatTotals {
            card_points: [30, 52, 38],
            tricks: [3, 4, 3],
        };
        assert_eq!(totals.most(), [false, true, false]);
        let text = ron::to_string(&totals).expect("ron");
        assert!(text.contains("most:(false,true,false)"), "{text}");
        let back: SeatTotals = ron::from_str(&text).expect("ron");
        assert_eq!(back, totals);
        // Saves without `most`, or with a wrong one, read the same.
        let old: SeatTotals =
            ron::from_str("(card_points:(30,52,38),tricks:(3,4,3))").expect("old");
        assert_eq!(old, totals);
        let wrong: SeatTotals =
            ron::from_str("(card_points:(30,52,38),tricks:(3,4,3),most:(true,true,true))")
                .expect("wrong");
        assert_eq!(wrong, totals);
    }
}

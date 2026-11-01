//! The shared deck of a run (ADR 0005) as card instances.
//!
//! Every card of the run's deck is a [`DeckCard`]: a stable instance id, a
//! [`CardKind`] (normal card or Blanko-Karte) and the card modifications
//! attached to it. The deck never drops below 32 cards: removing a card at
//! 32 replaces it with a blank, so the deal stays 10/10/10/2 and there are
//! always ten tricks (design document 16). Adding a card fills a blank if
//! there is one; otherwise the deck grows beyond 32 and for every game the
//! cards beyond the first 32 after shuffling stay in the talon.
//!
//! For one game the deck is resolved into [`DealtCard`]s: copies that carry
//! the effective Augen, rank bonus and visibility after all modifications.
//! The resolution itself lives in [`crate::content`], because it applies
//! content effects.

use std::fmt;

use serde::{Deserialize, Serialize};

pub use crate::cards::CardKind;
use crate::cards::{Card, PlayingCard, standard_deck};
use crate::content::{CardModId, DeckSpec};
use crate::game::{CARDS_IN_PLAY, HAND_SIZE};

/// One card instance of the run's deck.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DeckCard {
    /// Stable instance id, unique within the run.
    pub id: u32,
    /// Normal card or blank.
    pub kind: CardKind,
    /// Card modifications attached to this instance, in the order applied.
    pub mods: Vec<CardModId>,
}

/// A card instance as dealt into one game, with every modification already
/// resolved. Cheap to copy; the game state machine and the AI work on it.
///
/// Ordering: by kind (so hands sort like plain cards, blanks last), then by
/// instance id.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct DealtCard {
    /// Normal card or blank (a boss may blank out cards for one game).
    pub kind: CardKind,
    /// Instance id of the [`DeckCard`].
    pub id: u32,
    /// Effective Augen.
    pub card_points: u32,
    /// Effective rank bonus ("Gezinkt").
    pub rank_bonus: u8,
    /// Whether the player sees who holds this card ("Durchsichtig").
    pub revealed: bool,
    /// Whether this card can never take a trick ("Unsichtbar").
    #[serde(default)]
    pub never_wins: bool,
    /// Whether this card is a trump below the Karo-Bube in suit games
    /// ("Bube ehrenhalber").
    #[serde(default)]
    pub honorary_jack: bool,
    /// Whether this card must not be discarded ("Ramponiert").
    #[serde(default)]
    pub no_discard: bool,
}

impl DealtCard {
    /// A dealt card without modifications.
    #[must_use]
    pub fn plain(id: u32, kind: CardKind) -> DealtCard {
        DealtCard {
            kind,
            id,
            card_points: kind.face().map_or(0, |c| c.card_points()),
            rank_bonus: 0,
            revealed: false,
            never_wins: false,
            honorary_jack: false,
            no_discard: false,
        }
    }
}

impl PlayingCard for DealtCard {
    fn kind(self) -> CardKind {
        self.kind
    }

    fn card_points(self) -> u32 {
        self.card_points
    }

    fn rank_bonus(self) -> u8 {
        self.rank_bonus
    }

    fn never_wins(self) -> bool {
        self.never_wins
    }

    fn honorary_jack(self) -> bool {
        self.honorary_jack
    }

    fn no_discard(self) -> bool {
        self.no_discard
    }
}

impl fmt::Display for DealtCard {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.kind.fmt(f)
    }
}

/// A probability as an exact fraction plus its rounded value in
/// thousandths, so a frontend can show "37,5 %" without computing anything.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Odds {
    /// Favourable cases.
    pub numerator: u32,
    /// All cases (never 0).
    pub denominator: u32,
    /// The probability in thousandths, rounded half up.
    pub permille: u32,
}

impl Odds {
    /// `numerator / denominator`; a zero denominator gives 0.
    #[must_use]
    pub fn new(numerator: u32, denominator: u32) -> Odds {
        let denominator = denominator.max(1);
        let permille =
            (u64::from(numerator) * 1000 + u64::from(denominator) / 2) / u64::from(denominator);
        Odds {
            numerator,
            denominator,
            permille: u32::try_from(permille).unwrap_or(u32::MAX),
        }
    }
}

/// Where one card of the deck lands when a game is dealt (DESIGN 7.4).
///
/// Every game shuffles the whole deck, blanks and talon included, and
/// deals the first 32 cards 10/10/10/2; the rest stays in the talon. So
/// every card instance has the same chance for every place, and the odds
/// depend on the deck size only: in the standard deck a card reaches the
/// player's hand or the skat with 12 / 32 = 37.5 %.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DealOdds {
    /// Cards in the deck, blanks included.
    pub deck_size: u32,
    /// In the player's ten cards.
    pub hand: Odds,
    /// In the skat.
    pub skat: Odds,
    /// In the player's hand or the skat: the cards the player gets to use.
    pub hand_or_skat: Odds,
    /// With one of the two defenders.
    pub defenders: Odds,
    /// In the talon (not dealt in this game).
    pub talon: Odds,
}

/// The odds for any one card of a deck with `deck_size` cards. Fewer than
/// 32 cards cannot happen (blanks fill up), so such sizes count as 32.
#[must_use]
pub fn deal_odds(deck_size: usize) -> DealOdds {
    let n = u32::try_from(deck_size.max(CARDS_IN_PLAY)).unwrap_or(u32::MAX);
    let hand = u32::try_from(HAND_SIZE).unwrap_or(10);
    let in_play = u32::try_from(CARDS_IN_PLAY).unwrap_or(32);
    let skat = in_play - 3 * hand;
    DealOdds {
        deck_size: n,
        hand: Odds::new(hand, n),
        skat: Odds::new(skat, n),
        hand_or_skat: Odds::new(hand + skat, n),
        defenders: Odds::new(2 * hand, n),
        talon: Odds::new(n - in_play, n),
    }
}

/// Why a deck operation was rejected.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeckError {
    /// No card with this instance id.
    UnknownCard(u32),
    /// A blank cannot be removed while the deck holds only 32 cards: that
    /// would leave fewer than ten tricks.
    WouldShrinkBelowMinimum,
    /// Blanks carry no modifications.
    BlankCannotBeModified,
    /// The card already carries this modification.
    AlreadyModified(CardModId),
    /// The deck holds no instance of this printed card.
    MissingCard(Card),
}

impl fmt::Display for DeckError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DeckError::UnknownCard(id) => write!(f, "no card with id {id}"),
            DeckError::WouldShrinkBelowMinimum => f.write_str("the deck cannot shrink below 32"),
            DeckError::BlankCannotBeModified => f.write_str("blanks cannot be modified"),
            DeckError::AlreadyModified(m) => write!(f, "card already carries {m}"),
            DeckError::MissingCard(c) => write!(f, "no {c} in the deck"),
        }
    }
}

impl std::error::Error for DeckError {}

/// What a deck operation did, for events and the UI.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeckChange {
    /// A card was replaced by a new blank.
    ReplacedByBlank {
        /// The removed card.
        removed: DeckCard,
        /// Id of the new blank.
        blank: u32,
    },
    /// A card left the deck without replacement (deck above 32).
    Removed {
        /// The removed card.
        removed: DeckCard,
    },
    /// A card was added and took the place of a blank.
    FilledBlank {
        /// Id of the new card.
        card: u32,
        /// Id of the blank that left the deck.
        blank: u32,
    },
    /// A card was added on top of 32 or more cards.
    Added {
        /// Id of the new card.
        card: u32,
    },
    /// A modification was attached to a card.
    Modified {
        /// Id of the card.
        card: u32,
        /// The new modification.
        card_mod: CardModId,
    },
}

/// The run's shared deck.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Deck {
    cards: Vec<DeckCard>,
    next_id: u32,
}

impl Default for Deck {
    fn default() -> Deck {
        Deck::standard()
    }
}

impl Deck {
    /// The standard 32-card deck, ids 0..32 in [`standard_deck`] order.
    #[must_use]
    pub fn standard() -> Deck {
        let cards: Vec<DeckCard> = standard_deck()
            .into_iter()
            .zip(0u32..)
            .map(|(card, id)| DeckCard {
                id,
                kind: CardKind::Normal(card),
                mods: Vec::new(),
            })
            .collect();
        Deck { next_id: 32, cards }
    }

    /// All cards, in id order.
    #[must_use]
    pub fn cards(&self) -> &[DeckCard] {
        &self.cards
    }

    /// Number of cards (at least 32).
    #[must_use]
    pub fn len(&self) -> usize {
        self.cards.len()
    }

    /// Always `false`: a deck holds at least 32 cards.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.cards.is_empty()
    }

    /// Number of blanks.
    #[must_use]
    pub fn blanks(&self) -> usize {
        self.cards
            .iter()
            .filter(|c| c.kind == CardKind::Blank)
            .count()
    }

    /// The card with this instance id.
    #[must_use]
    pub fn get(&self, id: u32) -> Option<&DeckCard> {
        self.cards.iter().find(|c| c.id == id)
    }

    fn take_id(&mut self) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    /// Remove a card. At 32 cards it is replaced by a new blank; above 32
    /// it simply leaves the deck. A blank can only be removed above 32.
    pub fn remove(&mut self, id: u32) -> Result<DeckChange, DeckError> {
        let index = self
            .cards
            .iter()
            .position(|c| c.id == id)
            .ok_or(DeckError::UnknownCard(id))?;
        if self.cards.len() > CARDS_IN_PLAY {
            let removed = self.cards.remove(index);
            return Ok(DeckChange::Removed { removed });
        }
        if self.cards[index].kind == CardKind::Blank {
            return Err(DeckError::WouldShrinkBelowMinimum);
        }
        let removed = self.cards.remove(index);
        let blank = self.take_id();
        self.cards.push(DeckCard {
            id: blank,
            kind: CardKind::Blank,
            mods: Vec::new(),
        });
        Ok(DeckChange::ReplacedByBlank { removed, blank })
    }

    /// Add a card. It takes the place of the oldest blank if there is one;
    /// otherwise the deck grows and the extra cards rotate through the
    /// talon from game to game.
    pub fn add(&mut self, kind: CardKind, mods: Vec<CardModId>) -> DeckChange {
        let card = self.take_id();
        let blank = self
            .cards
            .iter()
            .position(|c| c.kind == CardKind::Blank && kind != CardKind::Blank);
        self.cards.push(DeckCard {
            id: card,
            kind,
            mods,
        });
        match blank {
            Some(index) => {
                let blank = self.cards.remove(index).id;
                DeckChange::FilledBlank { card, blank }
            }
            None => DeckChange::Added { card },
        }
    }

    /// Build a character's start deck from this one: remove one instance
    /// of every card in `spec.remove` (the first by id), then add every
    /// card of `spec.add` (filling the blanks the removals left).
    pub fn apply_spec(&mut self, spec: &DeckSpec) -> Result<Vec<DeckChange>, DeckError> {
        let mut changes = Vec::new();
        for &card in &spec.remove {
            let id = self
                .cards
                .iter()
                .find(|c| c.kind == CardKind::Normal(card))
                .map(|c| c.id)
                .ok_or(DeckError::MissingCard(card))?;
            changes.push(self.remove(id)?);
        }
        for &card in &spec.add {
            changes.push(self.add(CardKind::Normal(card), Vec::new()));
        }
        Ok(changes)
    }

    /// Where any one card of this deck lands when a game is dealt.
    #[must_use]
    pub fn deal_odds(&self) -> DealOdds {
        deal_odds(self.len())
    }

    /// Copy a card including its modifications ("Kopieren").
    pub fn copy(&mut self, id: u32) -> Result<DeckChange, DeckError> {
        let original = self.get(id).ok_or(DeckError::UnknownCard(id))?.clone();
        Ok(self.add(original.kind, original.mods))
    }

    /// Attach a modification to a normal card. Each modification at most
    /// once per card.
    pub fn add_mod(&mut self, id: u32, card_mod: CardModId) -> Result<DeckChange, DeckError> {
        let card = self
            .cards
            .iter_mut()
            .find(|c| c.id == id)
            .ok_or(DeckError::UnknownCard(id))?;
        if card.kind == CardKind::Blank {
            return Err(DeckError::BlankCannotBeModified);
        }
        if card.mods.contains(&card_mod) {
            return Err(DeckError::AlreadyModified(card_mod));
        }
        card.mods.push(card_mod.clone());
        Ok(DeckChange::Modified { card: id, card_mod })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cards::Card;

    fn c(s: &str) -> Card {
        s.parse().expect("test card")
    }

    #[test]
    fn standard_deck_has_32_cards_with_ids_in_order() {
        let deck = Deck::standard();
        assert_eq!(deck.len(), 32);
        assert_eq!(deck.blanks(), 0);
        assert_eq!(deck.get(0).map(|d| d.kind), Some(CardKind::Normal(c("D7"))));
        assert_eq!(
            deck.get(31).map(|d| d.kind),
            Some(CardKind::Normal(c("CA")))
        );
    }

    #[test]
    fn removing_at_32_leaves_a_blank() {
        let mut deck = Deck::standard();
        let change = deck.remove(0).expect("remove");
        assert!(matches!(
            change,
            DeckChange::ReplacedByBlank { blank: 32, .. }
        ));
        assert_eq!(deck.len(), 32);
        assert_eq!(deck.blanks(), 1);
        assert_eq!(deck.get(0), None);
        // A blank cannot be removed at 32.
        assert_eq!(deck.remove(32), Err(DeckError::WouldShrinkBelowMinimum));
        assert_eq!(deck.remove(99), Err(DeckError::UnknownCard(99)));
    }

    #[test]
    fn adding_fills_a_blank_first_then_grows_into_the_talon() {
        let mut deck = Deck::standard();
        deck.remove(0).expect("remove");
        let change = deck.add(CardKind::Normal(c("CJ")), Vec::new());
        assert_eq!(
            change,
            DeckChange::FilledBlank {
                card: 33,
                blank: 32
            }
        );
        assert_eq!(deck.len(), 32);
        assert_eq!(deck.blanks(), 0);
        let change = deck.add(CardKind::Normal(c("CA")), Vec::new());
        assert_eq!(change, DeckChange::Added { card: 34 });
        assert_eq!(deck.len(), 33);
        // Above 32 a removal shrinks the deck instead of leaving a blank.
        assert!(matches!(deck.remove(34), Ok(DeckChange::Removed { .. })));
        assert_eq!(deck.len(), 32);
    }

    #[test]
    fn modifications_attach_once_and_never_to_blanks() {
        let mut deck = Deck::standard();
        let fat = CardModId::new("fat");
        deck.add_mod(5, fat.clone()).expect("mod");
        assert_eq!(
            deck.add_mod(5, fat.clone()),
            Err(DeckError::AlreadyModified(fat.clone()))
        );
        deck.remove(6).expect("remove");
        assert_eq!(deck.add_mod(32, fat), Err(DeckError::BlankCannotBeModified));
        let copy = deck.copy(5).expect("copy");
        assert_eq!(
            copy,
            DeckChange::FilledBlank {
                card: 33,
                blank: 32
            }
        );
        assert_eq!(deck.get(33).map(|d| d.mods.len()), Some(1));
    }

    #[test]
    fn deal_odds_follow_the_deck_size() {
        let odds = Deck::standard().deal_odds();
        assert_eq!(odds.deck_size, 32);
        assert_eq!(odds.hand_or_skat, Odds::new(12, 32));
        assert_eq!(odds.hand_or_skat.permille, 375);
        assert_eq!(odds.hand.permille, 313);
        assert_eq!(odds.skat.permille, 63);
        assert_eq!(odds.defenders.permille, 625);
        assert_eq!(odds.talon.numerator, 0);
        // A removed card leaves a blank: the odds stay.
        let mut deck = Deck::standard();
        deck.remove(0).expect("remove");
        assert_eq!(deck.deal_odds(), Deck::standard().deal_odds());
        // A card beyond 32 lowers everyone's odds and fills the talon.
        deck.add(CardKind::Normal(c("CJ")), Vec::new());
        deck.add(CardKind::Normal(c("CA")), Vec::new());
        let odds = deck.deal_odds();
        assert_eq!(odds.deck_size, 33);
        assert_eq!(odds.hand_or_skat.permille, 364);
        assert_eq!(odds.talon, Odds::new(1, 33));
        let sum = odds.hand.numerator
            + odds.skat.numerator
            + odds.defenders.numerator
            + odds.talon.numerator;
        assert_eq!(sum, odds.deck_size);
        assert_eq!(deal_odds(0).deck_size, 32);
    }

    #[test]
    fn dealt_card_follows_blank_rules() {
        let blank = DealtCard::plain(40, CardKind::Blank);
        assert!(blank.is_blank());
        assert_eq!(PlayingCard::card_points(blank), 0);
        let ace = DealtCard::plain(7, CardKind::Normal(c("DA")));
        assert_eq!(PlayingCard::card_points(ace), 11);
        assert!(ace < blank, "blanks sort last");
        assert_eq!(blank.to_string(), "XX");
    }
}

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

use std::collections::BTreeMap;
use std::fmt;

use serde::{Deserialize, Deserializer, Serialize};

pub use crate::cards::CardKind;
use crate::cards::{Card, PlayingCard, Suit, standard_deck};
use crate::content::{CardModId, Content, DeckSpec, ModKind};
use crate::game::{CARDS_IN_PLAY, HAND_SIZE};

/// One card instance of the run's deck.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DeckCard {
    /// Stable instance id, unique within the run.
    pub id: u32,
    /// Normal card or blank.
    pub kind: CardKind,
    /// Card modifications attached to this instance, in the order applied:
    /// at most one besides a special card (effects v2, DESIGN 7.2).
    pub mods: Vec<CardModId>,
    /// The value of each modification that has one (Augen +5: 5, Rang 2:
    /// 2). A modification without entry counts 1.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub mod_values: BTreeMap<CardModId, i32>,
}

impl DeckCard {
    /// The value of a modification on this card (1 without entry).
    #[must_use]
    pub fn mod_value(&self, card_mod: &CardModId) -> i32 {
        self.mod_values.get(card_mod).copied().unwrap_or(1)
    }

    fn new(id: u32, kind: CardKind) -> DeckCard {
        DeckCard {
            id,
            kind,
            mods: Vec::new(),
            mod_values: BTreeMap::new(),
        }
    }

    fn drop_mod(&mut self, card_mod: &CardModId) {
        self.mods.retain(|m| m != card_mod);
        self.mod_values.remove(card_mod);
    }
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
    /// Whether this card must not be discarded (dormant since effects v2).
    #[serde(default)]
    pub no_discard: bool,
    /// Augen below zero ("Ramponiert", "Augen −X" on a low card, DESIGN
    /// 7.2): `card_points` is 0 then and the trick that takes the card
    /// loses this many Augen.
    #[serde(default, skip_serializing_if = "is_zero")]
    pub point_debt: u32,
    /// Whether this is a "Wandelkarte" in the player's hand: it never
    /// obliges him to follow suit, and he names its suit when he plays it.
    /// Only the copies the player holds (hand and skat) carry it.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub wild: bool,
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
            point_debt: 0,
            wild: false,
        }
    }
}

#[allow(clippy::trivially_copy_pass_by_ref)]
fn is_zero(n: &u32) -> bool {
    *n == 0
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

    fn point_debt(self) -> u32 {
        self.point_debt
    }

    fn wild(self) -> bool {
        self.wild
    }

    fn with_suit(self, suit: Suit) -> DealtCard {
        match self.kind {
            CardKind::Normal(card) => DealtCard {
                kind: CardKind::Normal(Card::new(suit, card.rank)),
                ..self
            },
            CardKind::Blank => self,
        }
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
    /// The card already carries this modification and it cannot be
    /// raised (it has no value).
    AlreadyModified(CardModId),
    /// The card carries a curse: no other modification, no raise (DESIGN
    /// 7.2).
    Cursed(CardModId),
    /// The modification is at its highest value already ("Rang" 3).
    AtMaximum(CardModId),
    /// No such card modification in the content.
    UnknownMod(CardModId),
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
            DeckError::Cursed(m) => write!(f, "card is cursed with {m}"),
            DeckError::AtMaximum(m) => write!(f, "{m} is at its highest value"),
            DeckError::UnknownMod(m) => write!(f, "unknown card mod {m}"),
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
    /// A modification was attached to a card, raised or put in place of
    /// another one (effects v2, DESIGN 7.2).
    Modified {
        /// Id of the card.
        card: u32,
        /// The new modification.
        card_mod: CardModId,
        /// Its value on the card now (after a raise: the sum).
        #[serde(default = "one")]
        value: i32,
        /// The modification it replaced, if any.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        replaced: Option<CardModId>,
    },
    /// A modification fell off a card (a voucher once redeemed).
    Unmodified {
        /// Id of the card.
        card: u32,
        /// The modification that fell off.
        card_mod: CardModId,
    },
}

/// The modification "Augen" (DESIGN 7.2): one signed value with
/// [`RAMPONIERT`].
pub const AUGEN: &str = "points";

/// The curse "Ramponiert": Augen with a negative value (DESIGN 7.2).
pub const RAMPONIERT: &str = "battered";

const fn one() -> i32 {
    1
}

/// The version of the card modifications in a saved deck: 2 is effects v2
/// (DESIGN 7.2). A deck without it is converted when it is loaded
/// ([`migrate_legacy_mods`]).
pub const MODS_VERSION: u32 = 2;

/// The run's shared deck.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Deck {
    cards: Vec<DeckCard>,
    next_id: u32,
    /// [`MODS_VERSION`] once the card modifications are effects v2.
    mods_version: u32,
}

impl<'de> Deserialize<'de> for Deck {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Deck, D::Error> {
        #[derive(Deserialize)]
        struct Saved {
            cards: Vec<DeckCard>,
            next_id: u32,
            #[serde(default)]
            mods_version: u32,
        }
        let saved = Saved::deserialize(deserializer)?;
        let mut deck = Deck {
            cards: saved.cards,
            next_id: saved.next_id,
            mods_version: saved.mods_version,
        };
        if deck.mods_version < MODS_VERSION {
            for card in &mut deck.cards {
                migrate_legacy_mods(card);
            }
            deck.mods_version = MODS_VERSION;
        }
        Ok(deck)
    }
}

/// What an old card modification (before effects v2, 02.11.2026) becomes:
/// a new id with its value, or `None` if it is dropped (DESIGN 7.2,
/// "Umrechnung").
#[must_use]
pub fn legacy_mod(id: &str) -> Option<(&'static str, i32)> {
    Some(match id {
        "fat" => ("points", 4),
        "gold_rim" => ("points", 8),
        "lead_weight" => ("points", 10),
        "glass_ace" => ("points", 11),
        "marked" => ("rank", 1),
        "double_marked" | "dressed_up" => ("rank", 2),
        "stamp" => ("trump_bonus", 3),
        "crown" => ("trump_bonus", 6),
        "last_trick" | "seal" | "lucky_penny" | "tail_light" => ("last_trick", 1),
        "beer_stain" => ("battered", 5),
        "black_cat" => ("stained", 3),
        "transparent" => ("transparent", 1),
        "undead" => ("undead", 1),
        "wild_card" => ("wild_card", 1),
        _ => return None,
    })
}

/// Which of the converted modifications a card keeps, first wins: a curse
/// before anything else (Ramponiert, Fleckig, Durchsichtig), then the
/// strongest by a fixed order (Rang, Trumpf, Letzter Stich, Augen).
pub const LEGACY_KEEP_ORDER: [&str; 7] = [
    "battered",
    "stained",
    "transparent",
    "rank",
    "trump_bonus",
    "last_trick",
    "points",
];

/// The special cards among the converted modifications; they stay next to
/// the one kept.
const LEGACY_SPECIALS: [&str; 2] = ["undead", "wild_card"];

/// The converted modifications that carry a value.
const LEGACY_VALUED: [&str; 6] = [
    "points",
    "rank",
    "trump_bonus",
    "last_trick",
    "battered",
    "stained",
];

/// Convert the modifications of a card saved before effects v2 (DESIGN
/// 7.2): map every old id ([`legacy_mod`]), add up equal ones (Rang at most
/// 3), keep one by [`LEGACY_KEEP_ORDER`] and the special cards.
pub fn migrate_legacy_mods(card: &mut DeckCard) {
    let mut merged: BTreeMap<&'static str, i32> = BTreeMap::new();
    for id in &card.mods {
        if let Some((new, value)) = legacy_mod(id.as_str()) {
            let sum = merged.entry(new).or_insert(0);
            *sum = sum.saturating_add(value);
        }
    }
    if let Some(rank) = merged.get_mut("rank") {
        *rank = (*rank).min(3);
    }
    let kept = LEGACY_KEEP_ORDER
        .iter()
        .find(|id| merged.contains_key(*id))
        .copied();
    card.mods.clear();
    card.mod_values.clear();
    if card.kind == CardKind::Blank {
        return;
    }
    let specials = LEGACY_SPECIALS.iter().filter(|id| merged.contains_key(*id));
    for id in specials.copied().chain(kept) {
        let card_mod = CardModId::new(id);
        if LEGACY_VALUED.contains(&id) {
            let value = merged.get(id).copied().unwrap_or(1);
            card.mod_values.insert(card_mod.clone(), value);
        }
        card.mods.push(card_mod);
    }
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
            .map(|(card, id)| DeckCard::new(id, CardKind::Normal(card)))
            .collect();
        Deck {
            next_id: 32,
            cards,
            mods_version: MODS_VERSION,
        }
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
        self.cards.push(DeckCard::new(blank, CardKind::Blank));
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
        let mut new = DeckCard::new(card, kind);
        new.mods = mods;
        self.cards.push(new);
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
    /// card of `spec.add` (filling the blanks the removals left), then
    /// attach every modification of `spec.mods` to the first instance (by
    /// id) of its card, with its value ([`Deck::add_mod`]).
    pub fn apply_spec(
        &mut self,
        spec: &DeckSpec,
        content: &Content,
    ) -> Result<Vec<DeckChange>, DeckError> {
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
        for (card, card_mod, value) in &spec.mods {
            let id = self
                .cards
                .iter()
                .filter(|c| c.kind == CardKind::Normal(*card))
                .map(|c| c.id)
                .min()
                .ok_or(DeckError::MissingCard(*card))?;
            changes.push(self.add_mod(content, id, card_mod, *value)?);
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
        let change = self.add(original.kind, original.mods);
        if let Some(copy) = self.cards.iter_mut().max_by_key(|c| c.id) {
            copy.mod_values = original.mod_values;
        }
        Ok(change)
    }

    /// Attach a modification with `value` to a normal card by the rules
    /// of effects v2 (DESIGN 7.2):
    ///
    /// - a card carries at most one modification, plus one special card
    ///   (Untot, Wandelkarte); a new special card replaces another one;
    /// - the same modification again raises it: the values add up, at most
    ///   to [`crate::content::CardModDef::max`]; one without value cannot
    ///   be raised ([`DeckError::AlreadyModified`]);
    /// - another modification replaces the one on the card (named in
    ///   [`DeckChange::Modified`] as `replaced`);
    /// - a cursed card takes nothing but a special card
    ///   ([`DeckError::Cursed`]);
    /// - Augen and Ramponiert are one signed value ([`AUGEN`],
    ///   [`RAMPONIERT`]): Augen with a negative value is Ramponiert, a
    ///   curse; Ramponiert on a card with Augen adds up, and the sign of
    ///   the sum decides which of the two the card carries (0: none).
    pub fn add_mod(
        &mut self,
        content: &Content,
        id: u32,
        card_mod: &CardModId,
        value: i32,
    ) -> Result<DeckChange, DeckError> {
        if card_mod.as_str() == AUGEN || card_mod.as_str() == RAMPONIERT {
            let signed = if card_mod.as_str() == AUGEN {
                value
            } else {
                -value
            };
            return self.add_signed_points(content, id, signed);
        }
        let def = content
            .card_mod(card_mod)
            .ok_or_else(|| DeckError::UnknownMod(card_mod.clone()))?;
        let card = self
            .cards
            .iter_mut()
            .find(|c| c.id == id)
            .ok_or(DeckError::UnknownCard(id))?;
        if card.kind == CardKind::Blank {
            return Err(DeckError::BlankCannotBeModified);
        }
        let kind_of = |m: &CardModId| content.card_mod(m).map(|d| d.kind);
        let special = |m: &CardModId| kind_of(m) == Some(ModKind::Special);
        if !def.is_special() {
            if let Some(curse) = card
                .mods
                .iter()
                .find(|m| kind_of(m) == Some(ModKind::Curse))
            {
                return Err(DeckError::Cursed(curse.clone()));
            }
        }
        // The modification in the slot the new one goes to.
        let current = card
            .mods
            .iter()
            .find(|m| special(m) == def.is_special())
            .cloned();
        let limit = |v: i32| def.max.map_or(v, |max| v.min(max));
        let mut replaced = None;
        let new_value = match current {
            Some(old) if old == *card_mod => {
                if !def.has_value() {
                    return Err(DeckError::AlreadyModified(card_mod.clone()));
                }
                let old_value = card.mod_value(card_mod);
                let sum = limit(old_value.saturating_add(value));
                if sum == old_value {
                    return Err(DeckError::AtMaximum(card_mod.clone()));
                }
                sum
            }
            Some(old) => {
                card.drop_mod(&old);
                replaced = Some(old);
                card.mods.push(card_mod.clone());
                limit(value)
            }
            None => {
                card.mods.push(card_mod.clone());
                limit(value)
            }
        };
        if def.has_value() {
            card.mod_values.insert(card_mod.clone(), new_value);
        }
        Ok(DeckChange::Modified {
            card: id,
            card_mod: card_mod.clone(),
            value: new_value,
            replaced,
        })
    }

    /// [`Deck::add_mod`] for the signed Augen value (Augen ≥ +1,
    /// Ramponiert ≤ −1, DESIGN 7.2).
    fn add_signed_points(
        &mut self,
        content: &Content,
        id: u32,
        signed: i32,
    ) -> Result<DeckChange, DeckError> {
        let (augen, ramponiert) = (CardModId::new(AUGEN), CardModId::new(RAMPONIERT));
        if content.card_mod(&augen).is_none() || content.card_mod(&ramponiert).is_none() {
            return Err(DeckError::UnknownMod(augen));
        }
        let card = self
            .cards
            .iter_mut()
            .find(|c| c.id == id)
            .ok_or(DeckError::UnknownCard(id))?;
        if card.kind == CardKind::Blank {
            return Err(DeckError::BlankCannotBeModified);
        }
        let kind_of = |m: &CardModId| content.card_mod(m).map(|d| d.kind);
        if let Some(curse) = card
            .mods
            .iter()
            .find(|m| kind_of(m) == Some(ModKind::Curse))
        {
            return Err(DeckError::Cursed(curse.clone()));
        }
        let normal = card
            .mods
            .iter()
            .find(|m| kind_of(m) != Some(ModKind::Special))
            .cloned();
        let (base, replaced) = match normal {
            Some(m) if m == augen => (card.mod_value(&augen), None),
            Some(m) => (0, Some(m)),
            None => (0, None),
        };
        if let Some(old) = &replaced {
            card.drop_mod(old);
        }
        card.drop_mod(&augen);
        let sum = base.saturating_add(signed);
        let (new, value) = match sum.cmp(&0) {
            std::cmp::Ordering::Greater => (augen.clone(), sum),
            std::cmp::Ordering::Less => (ramponiert, -sum),
            std::cmp::Ordering::Equal => {
                return Ok(DeckChange::Unmodified {
                    card: id,
                    card_mod: augen,
                });
            }
        };
        card.mods.push(new.clone());
        card.mod_values.insert(new.clone(), value);
        let replaced = replaced.or_else(|| (base > 0 && new != augen).then_some(augen));
        Ok(DeckChange::Modified {
            card: id,
            card_mod: new,
            value,
            replaced,
        })
    }

    /// Take a modification off a card (a redeemed voucher). Nothing if the
    /// card does not carry it.
    pub fn remove_mod(&mut self, id: u32, card_mod: &CardModId) -> Option<DeckChange> {
        let card = self.cards.iter_mut().find(|c| c.id == id)?;
        if !card.mods.contains(card_mod) {
            return None;
        }
        card.drop_mod(card_mod);
        Some(DeckChange::Unmodified {
            card: id,
            card_mod: card_mod.clone(),
        })
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
    fn modifications_never_go_on_blanks_and_copy_with_their_values() {
        let content = Content::load().expect("content");
        let mut deck = Deck::standard();
        let points = CardModId::new(AUGEN);
        deck.add_mod(&content, 5, &points, 4).expect("mod");
        deck.remove(6).expect("remove");
        assert_eq!(
            deck.add_mod(&content, 32, &points, 4),
            Err(DeckError::BlankCannotBeModified)
        );
        let copy = deck.copy(5).expect("copy");
        assert_eq!(
            copy,
            DeckChange::FilledBlank {
                card: 33,
                blank: 32
            }
        );
        assert_eq!(deck.get(33).map(|d| d.mod_value(&points)), Some(4));
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

//! The player's edition of the deck (ADR 0015) as card instances.
//!
//! Every game deals the normal Skat deck: 32 different cards, ids 0..32 in
//! [`standard_deck`] order. Every player owns his own edition of each card
//! ([`Edition`]): the same card with his modifications. The player builds
//! his edition over the run ([`DeckCard`] per card); the defenders' editions
//! are derived from who sits at the table (a Stammgast plays the plain
//! edition, a boss his boss edition) and never stored. No card leaves the
//! deck and none is added: there is no talon and no blank from removing.
//!
//! For one game every dealt card is resolved into a [`DealtCard`] with the
//! edition of the seat that holds it ([`Owner`]): the effective Augen, rank
//! bonus and visibility after its modifications. The resolution itself
//! lives in [`crate::content`], because it applies content effects.

use std::collections::BTreeMap;
use std::fmt;

use serde::{Deserialize, Deserializer, Serialize};

pub use crate::cards::CardKind;
use crate::cards::{Card, PlayingCard, Suit, standard_deck};
use crate::content::{CardModId, Content, DeckSpec, ModKind};
use crate::game::{CARDS_IN_PLAY, HAND_SIZE};

/// One card of an edition.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DeckCard {
    /// Stable instance id: the card's index in [`standard_deck`], the same
    /// in every edition.
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

/// Whose edition a dealt card is (ADR 0015): the player's, a Stammgast's
/// (the plain edition) or the boss's. For the view, the card back and the
/// hooks: a card's modifications act only in its owner's edition.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Default, Serialize, Deserialize,
)]
pub enum Owner {
    /// The player's edition ([`Edition`] of the run), also the skat.
    #[default]
    Player,
    /// A Stammgast: the plain edition.
    Guest,
    /// The boss: his boss edition (`edition` in `bosses.ron`).
    Boss,
}

/// A card instance as dealt into one game, with every modification already
/// resolved in the edition of its holder. Cheap to copy; the game state
/// machine and the AI work on it.
///
/// Ordering: by kind (so hands sort like plain cards, blanks last), then by
/// instance id.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct DealtCard {
    /// Normal card or blank (a boss may blank out cards for one game).
    pub kind: CardKind,
    /// Instance id of the [`DeckCard`]: the same in every edition.
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
    /// Whether this is a "Wandelkarte" of the player whose suit is not
    /// settled yet ([`PlayingCard::wild`]). Only the player's copies carry
    /// it; it goes off when the game starts and the card takes its suit
    /// ([`PlayingCard::settle_wild`]).
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub wild: bool,
    /// "Abgestuft" (DESIGN 9.2): see [`PlayingCard::demoted`].
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub demoted: bool,
    /// Whose edition this copy is.
    #[serde(default)]
    pub owner: Owner,
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
            demoted: false,
            owner: Owner::Player,
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

    fn demoted(self) -> bool {
        self.demoted
    }

    fn instance(self) -> Option<u32> {
        Some(self.id)
    }

    fn settle_wild(self, suit: Suit) -> DealtCard {
        DealtCard {
            wild: false,
            ..self.with_suit(suit)
        }
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

/// Where one card lands when a game is dealt (DESIGN 7.4).
///
/// Since ADR 0015 the deck is always the 32 cards of the Skat deck, so
/// every card reaches the player's hand or the skat with 12 / 32 = 37.5 %;
/// otherwise a defender holds it in his own edition.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DealOdds {
    /// Cards in the deck: always 32.
    pub deck_size: u32,
    /// In the player's ten cards.
    pub hand: Odds,
    /// In the skat.
    pub skat: Odds,
    /// In the player's hand or the skat: the cards the player gets to use.
    pub hand_or_skat: Odds,
    /// With one of the two defenders.
    pub defenders: Odds,
}

/// The odds for any one card of the deck.
#[must_use]
pub fn deal_odds() -> DealOdds {
    let n = u32::try_from(CARDS_IN_PLAY).unwrap_or(32);
    let hand = u32::try_from(HAND_SIZE).unwrap_or(10);
    let skat = n - 3 * hand;
    DealOdds {
        deck_size: n,
        hand: Odds::new(hand, n),
        skat: Odds::new(skat, n),
        hand_or_skat: Odds::new(hand + skat, n),
        defenders: Odds::new(2 * hand, n),
    }
}

/// Why a deck operation was rejected.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeckError {
    /// No card with this instance id.
    UnknownCard(u32),
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
    /// "Frisches Deck": the card carries no modification.
    NothingToCleanse(u32),
}

impl fmt::Display for DeckError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DeckError::UnknownCard(id) => write!(f, "no card with id {id}"),
            DeckError::BlankCannotBeModified => f.write_str("blanks cannot be modified"),
            DeckError::AlreadyModified(m) => write!(f, "card already carries {m}"),
            DeckError::Cursed(m) => write!(f, "card is cursed with {m}"),
            DeckError::AtMaximum(m) => write!(f, "{m} is at its highest value"),
            DeckError::UnknownMod(m) => write!(f, "unknown card mod {m}"),
            DeckError::MissingCard(c) => write!(f, "no {c} in the deck"),
            DeckError::NothingToCleanse(id) => write!(f, "card {id} carries no modification"),
        }
    }
}

impl std::error::Error for DeckError {}

/// What a deck operation did, for events and the UI.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeckChange {
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
    /// "Frisches Deck": every modification fell off a card, curses
    /// included (DESIGN 7.3).
    Cleansed {
        /// Id of the card.
        card: u32,
        /// The modifications that fell off.
        removed: Vec<CardModId>,
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
/// (DESIGN 7.2), 3 caps "Rang" at [`RANK_MAX`] (02.11.2026). A deck without
/// it is converted when it is loaded ([`migrate_legacy_mods`]), a deck of
/// version 2 gets its Rang capped.
pub const MODS_VERSION: u32 = 3;

/// The highest value of "Rang" (`rank`), the same as `max` in
/// `card_mods.ron` (a test checks it). Saves are migrated without content,
/// so the cap of old saves lives here.
pub const RANK_MAX: i32 = 2;

/// An edition of the 32 cards (ADR 0015): per card its modifications. The
/// run stores the player's ([`crate::run::Run::edition`]); a boss edition
/// is built from `bosses.ron` for every game ([`Edition::with_mods`]).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Edition {
    cards: Vec<DeckCard>,
    next_id: u32,
    /// [`MODS_VERSION`] once the card modifications are effects v2.
    mods_version: u32,
}

impl<'de> Deserialize<'de> for Edition {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Edition, D::Error> {
        #[derive(Deserialize)]
        struct Saved {
            cards: Vec<DeckCard>,
            next_id: u32,
            #[serde(default)]
            mods_version: u32,
        }
        let saved = Saved::deserialize(deserializer)?;
        let mut deck = Edition {
            cards: saved.cards,
            next_id: saved.next_id,
            mods_version: saved.mods_version,
        };
        if deck.mods_version < 2 {
            for card in &mut deck.cards {
                migrate_legacy_mods(card);
            }
        }
        if deck.mods_version < 3 {
            let rank = CardModId::new("rank");
            for card in &mut deck.cards {
                if let Some(value) = card.mod_values.get_mut(&rank) {
                    *value = (*value).min(RANK_MAX);
                }
            }
        }
        deck.mods_version = MODS_VERSION;
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
/// [`RANK_MAX`]), keep one by [`LEGACY_KEEP_ORDER`] and the special cards.
pub fn migrate_legacy_mods(card: &mut DeckCard) {
    let mut merged: BTreeMap<&'static str, i32> = BTreeMap::new();
    for id in &card.mods {
        if let Some((new, value)) = legacy_mod(id.as_str()) {
            let sum = merged.entry(new).or_insert(0);
            *sum = sum.saturating_add(value);
        }
    }
    if let Some(rank) = merged.get_mut("rank") {
        *rank = (*rank).min(RANK_MAX);
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

impl Default for Edition {
    fn default() -> Edition {
        Edition::standard()
    }
}

impl Edition {
    /// The plain edition: the 32 cards without modifications, ids 0..32 in
    /// [`standard_deck`] order.
    #[must_use]
    pub fn standard() -> Edition {
        let cards: Vec<DeckCard> = standard_deck()
            .into_iter()
            .zip(0u32..)
            .map(|(card, id)| DeckCard::new(id, CardKind::Normal(card)))
            .collect();
        Edition {
            next_id: 32,
            cards,
            mods_version: MODS_VERSION,
        }
    }

    /// The plain edition with `mods` on every card that matches their
    /// filter (a boss edition, `edition` in `bosses.ron`). Their "beim
    /// Erhalt" hooks do not fire.
    pub fn with_mods(
        mods: &[(crate::content::CardFilter, CardModId, i32)],
        content: &Content,
    ) -> Result<Edition, DeckError> {
        let mut edition = Edition::standard();
        for (filter, card_mod, value) in mods {
            let ids: Vec<u32> = edition
                .cards
                .iter()
                .filter(|c| c.kind.face().is_some_and(|f| filter.matches(f)))
                .map(|c| c.id)
                .collect();
            for id in ids {
                edition.add_mod(content, id, card_mod, *value)?;
            }
        }
        Ok(edition)
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

    /// The card with this instance id.
    #[must_use]
    pub fn get(&self, id: u32) -> Option<&DeckCard> {
        self.cards.iter().find(|c| c.id == id)
    }

    /// Build a character's start edition from this one: attach every
    /// modification of `spec.mods` to its card, with its value
    /// ([`Edition::add_mod`]).
    pub fn apply_spec(
        &mut self,
        spec: &DeckSpec,
        content: &Content,
    ) -> Result<Vec<DeckChange>, DeckError> {
        let mut changes = Vec::new();
        for (card, card_mod, value) in &spec.mods {
            let id = self
                .cards
                .iter()
                .find(|c| c.kind == CardKind::Normal(*card))
                .map(|c| c.id)
                .ok_or(DeckError::MissingCard(*card))?;
            changes.push(self.add_mod(content, id, card_mod, *value)?);
        }
        Ok(changes)
    }

    /// The id of a printed card in this edition.
    #[must_use]
    pub fn id_of(&self, card: Card) -> Option<u32> {
        self.cards
            .iter()
            .find(|c| c.kind == CardKind::Normal(card))
            .map(|c| c.id)
    }

    /// "Frisches Deck" (DESIGN 7.3): every modification falls off the
    /// card, curses and special cards included.
    pub fn cleanse(&mut self, id: u32) -> Result<DeckChange, DeckError> {
        let card = self
            .cards
            .iter_mut()
            .find(|c| c.id == id)
            .ok_or(DeckError::UnknownCard(id))?;
        if card.mods.is_empty() {
            return Err(DeckError::NothingToCleanse(id));
        }
        let removed = std::mem::take(&mut card.mods);
        card.mod_values.clear();
        Ok(DeckChange::Cleansed { card: id, removed })
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
    ///   ([`DeckError::Cursed`]); a special curse ("Abgestuft") takes a
    ///   normal modification but no other special card;
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
        let special = |m: &CardModId| kind_of(m).is_some_and(ModKind::is_special);
        if def.is_special() {
            // A special curse ("Abgestuft") holds the special slot for good.
            if let Some(curse) = card
                .mods
                .iter()
                .find(|m| kind_of(m) == Some(ModKind::SpecialCurse) && *m != card_mod)
            {
                return Err(DeckError::Cursed(curse.clone()));
            }
        } else {
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

    /// [`Edition::add_mod`] for the signed Augen value (Augen ≥ +1,
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
            .find(|m| !kind_of(m).is_some_and(ModKind::is_special))
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
    fn standard_edition_has_32_cards_with_ids_in_order() {
        let edition = Edition::standard();
        assert_eq!(edition.len(), 32);
        assert_eq!(
            edition.get(0).map(|d| d.kind),
            Some(CardKind::Normal(c("D7")))
        );
        assert_eq!(
            edition.get(31).map(|d| d.kind),
            Some(CardKind::Normal(c("CA")))
        );
        assert_eq!(edition.id_of(c("CA")), Some(31));
    }

    #[test]
    fn cleansing_takes_every_modification_off_curses_included() {
        let content = Content::load().expect("content");
        let mut edition = Edition::standard();
        let battered = CardModId::new(RAMPONIERT);
        edition.add_mod(&content, 5, &battered, 5).expect("curse");
        edition
            .add_mod(&content, 5, &CardModId::new("undead"), 1)
            .expect("special");
        assert_eq!(
            edition.cleanse(5),
            Ok(DeckChange::Cleansed {
                card: 5,
                removed: vec![battered, CardModId::new("undead")],
            })
        );
        assert!(
            edition
                .get(5)
                .is_some_and(|d| d.mods.is_empty() && d.mod_values.is_empty())
        );
        assert_eq!(edition.cleanse(5), Err(DeckError::NothingToCleanse(5)));
        assert_eq!(edition.cleanse(40), Err(DeckError::UnknownCard(40)));
    }

    #[test]
    fn demoted_is_a_special_curse_next_to_one_normal_modification() {
        let content = Content::load().expect("content");
        let mut edition = Edition::standard();
        let jack = edition.id_of(c("CJ")).expect("CJ");
        let demoted = CardModId::new("demoted");
        edition
            .add_mod(&content, jack, &demoted, 1)
            .expect("demoted");
        // A normal modification sits next to it.
        edition
            .add_mod(&content, jack, &CardModId::new(AUGEN), 3)
            .expect("Augen next to the special curse");
        // No other special card replaces it, and it is not raised.
        assert_eq!(
            edition.add_mod(&content, jack, &CardModId::new("undead"), 1),
            Err(DeckError::Cursed(demoted.clone()))
        );
        assert_eq!(
            edition.add_mod(&content, jack, &demoted, 1),
            Err(DeckError::AlreadyModified(demoted.clone()))
        );
        assert_eq!(
            edition.get(jack).map(|d| d.mods.clone()),
            Some(vec![demoted, CardModId::new(AUGEN)])
        );
        // Frisches Deck takes it off.
        assert!(edition.cleanse(jack).is_ok());
    }

    #[test]
    fn boss_editions_put_a_modification_on_every_matching_card() {
        let content = Content::load().expect("content");
        let sevens = crate::content::CardFilter {
            suit: None,
            rank: Some(crate::cards::Rank::Seven),
        };
        let edition =
            Edition::with_mods(&[(sevens, CardModId::new(AUGEN), 4)], &content).expect("edition");
        let modified: Vec<u32> = edition
            .cards()
            .iter()
            .filter(|d| !d.mods.is_empty())
            .map(|d| d.id)
            .collect();
        assert_eq!(modified.len(), 4);
        assert!(modified.iter().all(|&id| edition.get(id).is_some_and(|d| {
            d.kind
                .face()
                .is_some_and(|f| f.rank == crate::cards::Rank::Seven)
                && d.mod_value(&CardModId::new(AUGEN)) == 4
        })));
    }

    #[test]
    fn deal_odds_are_twelve_of_thirty_two() {
        let odds = deal_odds();
        assert_eq!(odds.deck_size, 32);
        assert_eq!(odds.hand_or_skat, Odds::new(12, 32));
        assert_eq!(odds.hand_or_skat.permille, 375);
        assert_eq!(odds.hand.permille, 313);
        assert_eq!(odds.skat.permille, 63);
        assert_eq!(odds.defenders.permille, 625);
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

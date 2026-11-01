//! Run state machine (design document 5, 6 and 12).
//!
//! A run is a tour through sections (Skatabende). Each section has three
//! series: two normal ones and a boss series. A series has a target
//! (Zielwert), a game limit and a number of passes. Won games add their
//! value to the series account, lost games subtract twice their value
//! including all relic modifiers. A series is won the moment the account
//! reaches the target; the run is lost when the game limit is reached
//! below the target. Between series the Tresen (shop) opens.
//!
//! The run is driven by [`RunAction`]s and answers with [`RunEvent`]s. The
//! two defenders are played by the heuristic AI inside the run, so after
//! every action the run waits for the next decision of the player. The
//! whole state is serializable (serde), so a frontend can save and load a
//! run at any point; equal seed and equal actions give equal results.
//!
//! Content effects fire at fixed points ([`Stage`]): passive effects when a
//! game is set up, then `OnDeal`, `OnDeclare`, `OnCardPlayed`,
//! `OnTrickEnd`, `OnScore` and, after a won series, `OnSeriesWon`. Sources
//! fire in a fixed order: boss, relics in purchase order, the house rules
//! of the evening (encounter "Stammtisch-Streit"), card modifications of
//! the cards in play by card id. Conditions see the effects of earlier
//! sources in the same stage. A card modification's `OnAttach` hooks fire
//! when it is attached to a card (curses pay Groschen there).
//!
//! A run is played by one [`Character`] (DESIGN 9): the Alleinspieler
//! declares suit games and grand, the Nullspieler only Null. Between two
//! series an encounter (Ereignis, DESIGN 13) may come instead of the
//! Tresen. What is unlocked across runs lives outside the run, in
//! [`meta::MetaState`]; a run only gets the relics that are still locked
//! ([`RunStart::locked_relics`]).

pub mod bot;
pub mod meta;
pub mod player;
pub mod view;

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use serde::{Deserialize, Serialize};

use crate::ai::DefenderHeuristic;
use crate::cards::PlayingCard;
use crate::content::{
    ActiveSource, BossId, CardFilter, CardModId, Change, Character, ConsumableAction, ConsumableId,
    Content, Effect, EncounterId, EncounterKind, HookContext, Lever, Rarity, RelicId, RelicKind,
    ScoreBreakdown, ScoreLine, Source, Stage, card_mod_sources, evaluate, in_pool, resolve_deck,
    score_parts,
};
use crate::deck::{CardKind, DealtCard, Deck, DeckChange, DeckError};
use crate::game::{Event, Game, GameError, Phase, Seat};
use crate::rng::Rng;
use crate::rules::{
    Declaration, GameType, NullScoring, Thresholds, ValueParts, game_value_parts, null_value_parts,
    ramsch_value_parts, suit_strength, trump_strength,
};

/// Series per section: two normal ones and the boss series.
pub const SERIES_PER_SECTION: u32 = 3;

/// What the run is waiting for.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RunPhase {
    /// A game of the current series is dealt; see [`ActiveGame`].
    Game,
    /// The Tresen is open between two series.
    Shop,
    /// An encounter (Ereignis) came instead of the Tresen; see
    /// [`ActiveEncounter`].
    Encounter,
    /// The last boss series was won.
    Won,
    /// A series ended below its target.
    Lost,
}

/// The current series.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Series {
    /// Section (Abschnitt), counted from 0.
    pub section: u32,
    /// Series within the section: 0, 1 or 2 (boss series).
    pub index: u32,
    /// Zielwert.
    pub target: u64,
    /// Series account.
    pub score: i64,
    /// Games played or passed.
    pub games_played: u32,
    /// Spiellimit.
    pub game_limit: u32,
    /// Passes left.
    pub passes_left: u32,
    /// Boss of a boss series.
    pub boss: Option<BossId>,
    /// Redeals ("Schieben") used in this series.
    #[serde(default)]
    pub redeals_used: u32,
    /// Passes used in this series.
    #[serde(default)]
    pub passes_used: u32,
    /// Skat swaps ("Die Oma") used in this series.
    #[serde(default)]
    pub skat_swaps_used: u32,
    /// The skat swap filters used in this series, one per swap. Saves from
    /// before several skat swaps lack it; then the first
    /// [`Series::skat_swaps_used`] filters count as used.
    #[serde(default)]
    pub skat_swaps_taken: Vec<CardFilter>,
    /// Whether the Ramsch of this series ("Der Ramscher") was dealt.
    #[serde(default)]
    pub ramsch_played: bool,
}

impl Series {
    /// Games left before the limit.
    #[must_use]
    pub const fn games_left(&self) -> u32 {
        self.game_limit.saturating_sub(self.games_played)
    }

    /// Whether the account reached the target.
    #[must_use]
    pub fn target_reached(&self) -> bool {
        i128::from(self.score) >= i128::from(self.target)
    }
}

/// Kontra state of one game.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Kontra {
    /// Nobody doubled.
    None,
    /// A defender said Kontra (×2).
    Kontra(Seat),
    /// The declarer answered Re (×4 in total).
    Re(Seat),
}

/// Rules in force for one game, collected from passive effects and hooks.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct GameRules {
    /// Thresholds for won and Schneider.
    pub thresholds: Thresholds,
    /// Source of the lowest win threshold, if lowered.
    pub win_threshold_source: Option<Source>,
    /// Sources of declarer-side Schneider ranges.
    pub schneider_sources: Vec<(Source, (u32, u32))>,
    /// Every game must be a hand game.
    pub require_hand: bool,
    /// Who allows Kontra in this game, if anyone.
    pub kontra_source: Option<Source>,
    /// Redeals ("Schieben") the content grants per series. Whether one is
    /// left is [`Series::redeals_used`] against this.
    pub series_redeals: u32,
    /// Every game must be declared Ouvert ("Der Angeber").
    pub require_ouvert: bool,
    /// Cards of the defenders the player sees before declaring ("Der
    /// Kiebitz").
    pub peek: u32,
    /// The player sees the count of Augen ("Der Zähler").
    pub show_count: bool,
    /// Skat swaps the content grants, one filter per granting effect
    /// ("Die Oma": jacks, "Der Opa": aces): each lets the player take a
    /// matching card from the skat once per series. Also read from the
    /// single filter `skat_swap` of older saves.
    #[serde(alias = "skat_swap", deserialize_with = "filters_or_one")]
    pub skat_swaps: Vec<CardFilter>,
    /// This game is the series' Ramsch ("Der Ramscher").
    pub ramsch: bool,
    /// Information effects are off ("Der Kiebitz-Mörder").
    pub no_information: bool,
    /// Boss rules that cheat are off ("Der Schiedsrichter").
    pub no_cheating: bool,
}

/// Read a list of card filters, or the single optional filter of saves
/// from before several skat swaps (`null`/`None` or one filter). Works for
/// JSON and RON alike, so no untagged enum (RON cannot buffer those).
fn filters_or_one<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<Vec<CardFilter>, D::Error> {
    struct Filters;

    impl<'de> serde::de::Visitor<'de> for Filters {
        type Value = Vec<CardFilter>;

        fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str("a list of card filters, one card filter or nothing")
        }

        fn visit_seq<A: serde::de::SeqAccess<'de>>(
            self,
            mut seq: A,
        ) -> Result<Vec<CardFilter>, A::Error> {
            let mut filters = Vec::new();
            while let Some(filter) = seq.next_element::<CardFilter>()? {
                filters.push(filter);
            }
            Ok(filters)
        }

        fn visit_map<A: serde::de::MapAccess<'de>>(
            self,
            map: A,
        ) -> Result<Vec<CardFilter>, A::Error> {
            CardFilter::deserialize(serde::de::value::MapAccessDeserializer::new(map))
                .map(|f| vec![f])
        }

        fn visit_some<D2: serde::Deserializer<'de>>(
            self,
            deserializer: D2,
        ) -> Result<Vec<CardFilter>, D2::Error> {
            deserializer.deserialize_any(Filters)
        }

        fn visit_none<E: serde::de::Error>(self) -> Result<Vec<CardFilter>, E> {
            Ok(Vec::new())
        }

        fn visit_unit<E: serde::de::Error>(self) -> Result<Vec<CardFilter>, E> {
            Ok(Vec::new())
        }
    }

    deserializer.deserialize_any(Filters)
}

/// The game in progress and everything content attached to it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActiveGame {
    /// The game itself.
    pub game: Game<DealtCard>,
    /// Rules in force.
    pub rules: GameRules,
    /// Score modifications collected so far.
    pub lines: Vec<ScoreLine>,
    /// Augen changes per card and source from the deck resolution.
    pub point_changes: Vec<(u32, Source, i32)>,
    /// Kontra state.
    pub kontra: Kontra,
    /// Kontra was said and the declarer has to answer (Re or not) before
    /// the first card.
    pub awaiting_re: bool,
    /// Defender cards the player was shown ("Der Kiebitz") and who held
    /// them at the deal.
    #[serde(default)]
    pub peeked: Vec<(DealtCard, Seat)>,
    /// The boss whose rule is on in this game: the series' boss, or for the
    /// Endboss "Der Alte" the rule of this game.
    #[serde(default)]
    pub boss_rule: Option<BossId>,
}

/// One choice of an encounter.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EncounterChoice {
    /// Stammtisch-Streit: this house rule applies until the end of the
    /// section.
    TakeRule(RelicId),
    /// Der Kartengeber: a random card gets a modification.
    MarkRandomCard,
    /// Der Kartengeber: the player chooses the card (target needed).
    MarkChosenCard,
    /// Alter Bekannter: the owned relic at `index` is swapped for a random
    /// other one.
    SwapRelic {
        /// Index into the owned relics.
        index: usize,
        /// The relic given away.
        relic: RelicId,
        /// The relic received, drawn when the encounter opened. `None`
        /// (saves from before): drawn at random when chosen.
        #[serde(default)]
        new: Option<RelicId>,
    },
    /// Letzte Runde: the next game declared counts this many times.
    DoubleNextGame(u32),
    /// Walk on without taking anything.
    Leave,
}

/// An option of an open encounter: the choice and its price.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EncounterOption {
    /// What it does.
    pub choice: EncounterChoice,
    /// Groschen it costs.
    pub cost: u32,
    /// For [`EncounterChoice::MarkRandomCard`] and
    /// [`EncounterChoice::MarkChosenCard`]: the modification the card gets,
    /// named by the encounter or drawn when it opened. `None` (saves from
    /// before): drawn at random when chosen.
    #[serde(default)]
    pub card_mod: Option<CardModId>,
}

/// The encounter (Ereignis) between two series, while open.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActiveEncounter {
    /// Which encounter.
    pub id: EncounterId,
    /// Its options, drawn when it opened; the last is always
    /// [`EncounterChoice::Leave`].
    pub options: Vec<EncounterOption>,
}

/// The Tresen between two series. Bought items leave an empty slot.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Shop {
    /// Relics on offer.
    pub relics: Vec<Option<RelicId>>,
    /// Consumables on offer.
    pub consumables: Vec<Option<ConsumableId>>,
    /// Neuauslagen bought during this visit.
    pub rerolls: u32,
}

/// Groschen paid for a won series. Serialized with its sum as `total`, so
/// that a frontend does not add anything up itself.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(into = "PayoutRepr", from = "PayoutRepr")]
pub struct Payout {
    /// Grundbetrag.
    pub base: u32,
    /// Bonus for games left.
    pub games_left: u32,
    /// Zinsen on the Groschen held before the payout.
    pub interest: u32,
}

impl Payout {
    /// Sum of all parts.
    #[must_use]
    pub const fn total(&self) -> u32 {
        self.base + self.games_left + self.interest
    }
}

/// Serialized form of [`Payout`]: the parts plus their sum. The sum is
/// ignored when reading; it is always recomputed.
#[derive(Serialize, Deserialize)]
struct PayoutRepr {
    base: u32,
    games_left: u32,
    interest: u32,
    #[serde(default)]
    total: u32,
}

impl From<Payout> for PayoutRepr {
    fn from(p: Payout) -> PayoutRepr {
        PayoutRepr {
            base: p.base,
            games_left: p.games_left,
            interest: p.interest,
            total: p.total(),
        }
    }
}

impl From<PayoutRepr> for Payout {
    fn from(p: PayoutRepr) -> Payout {
        Payout {
            base: p.base,
            games_left: p.games_left,
            interest: p.interest,
        }
    }
}

/// Statistics of a run, for the run summary. Bookkeeping only; nothing in
/// the run reads them.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct RunStats {
    /// Games won.
    pub games_won: u32,
    /// Games lost.
    pub games_lost: u32,
    /// Games passed (eingepasst).
    pub games_passed: u32,
    /// Series won.
    pub series_won: u32,
    /// Highest Spielwert of a won game.
    pub best_game: u64,
}

/// Groschen paid when selling a relic or consumable: half its price.
#[must_use]
pub const fn sell_price(price: u32) -> u32 {
    price / 2
}

/// A decision of the player.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RunAction {
    /// Einpassen: skip the dealt game. Costs a game of the limit and a
    /// pass, no points.
    Pass,
    /// Have all hands and the skat dealt again before declaring, without
    /// using up a game (house rule "Schieben", once per series). Also read
    /// under its old name `RedrawSkat`.
    #[serde(alias = "RedrawSkat")]
    Redeal,
    /// Announce the game.
    Declare(Declaration),
    /// Answer a Kontra: `true` = Re.
    AnswerKontra {
        /// Whether to say Re.
        re: bool,
    },
    /// Discard two cards (instance ids) after picking up the skat.
    Discard([u32; 2]),
    /// Play a card (instance id).
    Play(u32),
    /// Use a consumable from the given slot, with a target card if needed.
    UseConsumable {
        /// Slot of the consumable.
        slot: usize,
        /// Target card instance id for deck changes.
        target: Option<u32>,
    },
    /// Buy the relic in this Tresen slot.
    BuyRelic(usize),
    /// Buy the consumable in this Tresen slot.
    BuyConsumable(usize),
    /// Sell an owned relic (index into the owned relics) for half its price.
    SellRelic(usize),
    /// Sell an owned consumable for half its price.
    SellConsumable(usize),
    /// Neuauslage: draw a new offer.
    Reroll,
    /// Leave the Tresen and start the next series.
    LeaveShop,
    /// "Die Oma" (and every other skat swap): in a hand game before the
    /// first card, take a matching card from the skat and put this card
    /// (instance id) in its place.
    TakeFromSkat {
        /// The card from the hand that goes into the skat.
        give: u32,
        /// The skat card to take (instance id, one of
        /// [`view::GameView::skat_swap_cards`]); `None` takes the first of
        /// them.
        #[serde(default)]
        take: Option<u32>,
    },
    /// Choose an option of the open encounter, with a target card if the
    /// option needs one; then the next series starts.
    ChooseEncounter {
        /// Index into [`ActiveEncounter::options`].
        option: usize,
        /// Deck card id for [`EncounterChoice::MarkChosenCard`].
        target: Option<u32>,
    },
}

/// Something that happened in the run, in order of occurrence.
///
/// `G` is the type of the game events: the full [`Event`] for the run
/// itself and the simulator, [`player::PlayerGameEvent`] for what the
/// player may see (see [`player::PlayerEvent`]).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RunEvent<G = Event<DealtCard>> {
    /// A new section (Skatabend) began and its boss was drawn. The boss is
    /// known from here on, so the player can prepare for it (DESIGN 11).
    BossAnnounced {
        /// Section, from 0.
        section: u32,
        /// The boss of this section's boss series.
        boss: BossId,
    },
    /// A new series started.
    SeriesStarted {
        /// Section, from 0.
        section: u32,
        /// Series within the section (2 = boss series).
        index: u32,
        /// Zielwert.
        target: u64,
        /// Spiellimit.
        game_limit: u32,
        /// Passes.
        passes: u32,
        /// Boss of a boss series.
        boss: Option<BossId>,
    },
    /// A game was dealt.
    GameDealt {
        /// Number of the game in the series, from 1.
        number: u32,
        /// Seat that leads.
        forehand: Seat,
    },
    /// An event of the game in progress.
    Game(G),
    /// The player passed.
    Passed {
        /// Passes left.
        passes_left: u32,
    },
    /// A defender said Kontra.
    Kontra {
        /// Who said it.
        seat: Seat,
    },
    /// The declarer answered Re.
    Re,
    /// A flag was set by content.
    FlagSet(String),
    /// A flag was cleared by content.
    FlagCleared(String),
    /// A counter was raised by content.
    CounterRaised {
        /// Counter name.
        counter: String,
        /// New value.
        value: u32,
    },
    /// Content paid (or took) Groschen ("Ramponiert" when attached).
    CoinsGained {
        /// Who paid.
        source: Source,
        /// Groschen gained; negative when taken.
        amount: i32,
        /// Groschen after the change.
        coins: u32,
    },
    /// A finished game was scored.
    GameScored {
        /// Every term and modification.
        breakdown: ScoreBreakdown,
        /// Series account after this game.
        series_score: i64,
    },
    /// The series was won.
    SeriesWon {
        /// Section.
        section: u32,
        /// Series within the section.
        index: u32,
        /// Groschen earned.
        payout: Payout,
        /// Groschen after the payout.
        coins: u32,
        /// The boss of a boss series.
        #[serde(default)]
        boss: Option<BossId>,
        /// Passes used in the series.
        #[serde(default)]
        passes_used: u32,
        /// Games left when the series was won.
        #[serde(default)]
        games_left: u32,
    },
    /// The Tresen opened.
    ShopOpened {
        /// Relics on offer.
        relics: Vec<Option<RelicId>>,
        /// Consumables on offer.
        consumables: Vec<Option<ConsumableId>>,
    },
    /// A relic was bought.
    RelicBought {
        /// The relic.
        relic: RelicId,
        /// Price paid.
        price: u32,
    },
    /// A consumable was bought.
    ConsumableBought {
        /// The consumable.
        consumable: ConsumableId,
        /// Price paid.
        price: u32,
    },
    /// A relic was sold.
    RelicSold {
        /// The relic.
        relic: RelicId,
        /// Groschen received.
        price: u32,
    },
    /// A consumable was sold.
    ConsumableSold {
        /// The consumable.
        consumable: ConsumableId,
        /// Groschen received.
        price: u32,
    },
    /// A new offer was drawn.
    ShopRerolled {
        /// Price paid.
        cost: u32,
    },
    /// The player left the Tresen.
    ShopLeft,
    /// A consumable was used.
    ConsumableUsed {
        /// The consumable.
        consumable: ConsumableId,
    },
    /// The deck changed.
    DeckChanged(DeckChange),
    /// Passes were added to the series.
    PassesAdded {
        /// Passes added.
        passes: u32,
        /// Passes left now.
        passes_left: u32,
    },
    /// The game was thrown in and dealt again (consumable "Neues Blatt" or
    /// house rule "Schieben").
    Redealt,
    /// "Der Kiebitz" showed the player a defender's card.
    CardPeeked {
        /// The card.
        card: DealtCard,
        /// Who holds it.
        seat: Seat,
    },
    /// The game is the Ramsch of the series ("Der Ramscher"); nobody
    /// declares, it is played at once.
    RamschDealt,
    /// An encounter opened instead of the Tresen.
    EncounterOpened {
        /// Which encounter.
        encounter: EncounterId,
        /// Its options.
        options: Vec<EncounterOption>,
    },
    /// The player chose an option of the encounter.
    EncounterResolved {
        /// Which encounter.
        encounter: EncounterId,
        /// The choice.
        choice: EncounterChoice,
    },
    /// A house rule applies until the end of the section.
    EveningRuleAdded(RelicId),
    /// The house rules of the evening ended with the section.
    EveningRulesEnded,
    /// An owned relic was swapped for another.
    RelicSwapped {
        /// The relic given away.
        old: RelicId,
        /// The relic received.
        new: RelicId,
    },
    /// The next game declared counts `factor` times ("Letzte Runde").
    NextGameDoubled {
        /// Factor on the Spielwert.
        factor: u32,
    },
    /// The run was won.
    RunWon {
        /// Who won it.
        #[serde(default)]
        character: Character,
    },
    /// The run was lost.
    RunLost {
        /// Section of the lost series.
        section: u32,
        /// Series within the section.
        index: u32,
        /// Who lost it.
        #[serde(default)]
        character: Character,
    },
}

/// Why an action was rejected. The run is unchanged.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RunError {
    /// The action does not fit the current phase.
    WrongPhase,
    /// The game rejected the action.
    Game(GameError<DealtCard>),
    /// No passes left in this series.
    NoPassesLeft,
    /// The boss demands a hand game.
    HandRequired,
    /// The Alleinspieler does not play Null (ADR 0003).
    NullNotAvailable,
    /// No redeal ("Schieben") left in this series.
    NoRedealLeft,
    /// Kontra must be answered first.
    AwaitingKontraAnswer,
    /// There is no Kontra to answer.
    NoKontra,
    /// The declarer does not hold this card instance.
    UnknownCard(u32),
    /// No item in this slot.
    InvalidSlot,
    /// Not enough Groschen.
    NotEnoughCoins,
    /// All slots are taken.
    NoFreeSlot,
    /// The deck rejected the change.
    Deck(DeckError),
    /// This consumable needs a target card.
    TargetRequired,
    /// No card modification can be added to this card.
    NoModAvailable,
    /// The content does not know this id.
    UnknownContent(String),
    /// A [`RunStart`] names a section or series that does not exist.
    InvalidStart,
    /// The character cannot declare this game (the Nullspieler only Null,
    /// nobody Ramsch).
    GameNotAvailable,
    /// "Der Angeber" demands an Ouvert game.
    OuvertRequired,
    /// No skat swap ("Die Oma") left in this series, or none granted.
    NoSkatSwapLeft,
    /// No matching card lies in the skat.
    NothingToTake,
    /// The Ramsch of the series cannot be passed or dealt again.
    RamschRequired,
}

/// Stable code of a [`RunError`], for frontends: serialized as its name
/// (`"NotEnoughCoins"`), never renamed once released. A frontend maps the
/// code to its own text; the English message of [`RunError`]'s `Display`
/// is for logs only.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum RunErrorCode {
    /// The action does not fit the current phase of the run.
    WrongPhase,
    /// No passes left in this series.
    NoPassesLeft,
    /// The boss demands a hand game.
    HandRequired,
    /// The Alleinspieler does not play Null.
    NullNotAvailable,
    /// No redeal ("Schieben") left.
    NoRedealLeft,
    /// Kontra must be answered first.
    AwaitingKontraAnswer,
    /// There is no Kontra to answer.
    NoKontra,
    /// The declarer does not hold this card.
    UnknownCard,
    /// No item in this slot.
    InvalidSlot,
    /// Not enough Groschen.
    NotEnoughCoins,
    /// All slots are taken.
    NoFreeSlot,
    /// The consumable needs a target card.
    TargetRequired,
    /// No card modification can be added to this card.
    NoModAvailable,
    /// The content does not know this id.
    UnknownContent,
    /// The start of the run names a section or series that does not exist.
    InvalidStart,
    /// The game rejected the action in its current phase.
    GameWrongPhase,
    /// It is another seat's turn.
    NotYourTurn,
    /// The card is not in the hand.
    CardNotInHand,
    /// The card does not follow suit although the hand could.
    MustFollowSuit,
    /// The two discards are not two distinct cards of the hand.
    InvalidDiscard,
    /// The cards do not form a legal deal.
    InvalidDeal,
    /// A trick is in progress.
    TrickInProgress,
    /// The deck has no card with this id.
    UnknownDeckCard,
    /// The deck cannot shrink below 32 cards.
    DeckAtMinimum,
    /// Blanks carry no modifications.
    BlankCannotBeModified,
    /// The card already carries this modification.
    AlreadyModified,
    /// The card is not in the deck.
    MissingCard,
    /// The character cannot declare this game.
    GameNotAvailable,
    /// An Ouvert game is required.
    OuvertRequired,
    /// No skat swap left.
    NoSkatSwapLeft,
    /// No matching card in the skat.
    NothingToTake,
    /// The skat cannot be swapped with now.
    InvalidSkatSwap,
    /// The Ramsch cannot be passed or dealt again.
    RamschRequired,
    /// The card must not be discarded ("Ramponiert").
    DiscardForbidden,
}

impl RunErrorCode {
    /// The code as text, equal to its serialized form.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            RunErrorCode::WrongPhase => "WrongPhase",
            RunErrorCode::NoPassesLeft => "NoPassesLeft",
            RunErrorCode::HandRequired => "HandRequired",
            RunErrorCode::NullNotAvailable => "NullNotAvailable",
            RunErrorCode::NoRedealLeft => "NoRedealLeft",
            RunErrorCode::AwaitingKontraAnswer => "AwaitingKontraAnswer",
            RunErrorCode::NoKontra => "NoKontra",
            RunErrorCode::UnknownCard => "UnknownCard",
            RunErrorCode::InvalidSlot => "InvalidSlot",
            RunErrorCode::NotEnoughCoins => "NotEnoughCoins",
            RunErrorCode::NoFreeSlot => "NoFreeSlot",
            RunErrorCode::TargetRequired => "TargetRequired",
            RunErrorCode::NoModAvailable => "NoModAvailable",
            RunErrorCode::UnknownContent => "UnknownContent",
            RunErrorCode::InvalidStart => "InvalidStart",
            RunErrorCode::GameWrongPhase => "GameWrongPhase",
            RunErrorCode::NotYourTurn => "NotYourTurn",
            RunErrorCode::CardNotInHand => "CardNotInHand",
            RunErrorCode::MustFollowSuit => "MustFollowSuit",
            RunErrorCode::InvalidDiscard => "InvalidDiscard",
            RunErrorCode::InvalidDeal => "InvalidDeal",
            RunErrorCode::TrickInProgress => "TrickInProgress",
            RunErrorCode::UnknownDeckCard => "UnknownDeckCard",
            RunErrorCode::DeckAtMinimum => "DeckAtMinimum",
            RunErrorCode::BlankCannotBeModified => "BlankCannotBeModified",
            RunErrorCode::AlreadyModified => "AlreadyModified",
            RunErrorCode::MissingCard => "MissingCard",
            RunErrorCode::GameNotAvailable => "GameNotAvailable",
            RunErrorCode::OuvertRequired => "OuvertRequired",
            RunErrorCode::NoSkatSwapLeft => "NoSkatSwapLeft",
            RunErrorCode::NothingToTake => "NothingToTake",
            RunErrorCode::InvalidSkatSwap => "InvalidSkatSwap",
            RunErrorCode::RamschRequired => "RamschRequired",
            RunErrorCode::DiscardForbidden => "DiscardForbidden",
        }
    }
}

impl RunError {
    /// The stable code of this error.
    #[must_use]
    pub const fn code(&self) -> RunErrorCode {
        match self {
            RunError::WrongPhase => RunErrorCode::WrongPhase,
            RunError::Game(e) => match e {
                GameError::WrongPhase { .. } => RunErrorCode::GameWrongPhase,
                GameError::NotYourTurn { .. } => RunErrorCode::NotYourTurn,
                GameError::CardNotInHand { .. } => RunErrorCode::CardNotInHand,
                GameError::MustFollowSuit { .. } => RunErrorCode::MustFollowSuit,
                GameError::InvalidDiscard { .. } => RunErrorCode::InvalidDiscard,
                GameError::InvalidDeal => RunErrorCode::InvalidDeal,
                GameError::TrickInProgress => RunErrorCode::TrickInProgress,
                GameError::InvalidSkatSwap => RunErrorCode::InvalidSkatSwap,
                GameError::DiscardForbidden { .. } => RunErrorCode::DiscardForbidden,
            },
            RunError::NoPassesLeft => RunErrorCode::NoPassesLeft,
            RunError::HandRequired => RunErrorCode::HandRequired,
            RunError::NullNotAvailable => RunErrorCode::NullNotAvailable,
            RunError::NoRedealLeft => RunErrorCode::NoRedealLeft,
            RunError::AwaitingKontraAnswer => RunErrorCode::AwaitingKontraAnswer,
            RunError::NoKontra => RunErrorCode::NoKontra,
            RunError::UnknownCard(_) => RunErrorCode::UnknownCard,
            RunError::InvalidSlot => RunErrorCode::InvalidSlot,
            RunError::NotEnoughCoins => RunErrorCode::NotEnoughCoins,
            RunError::NoFreeSlot => RunErrorCode::NoFreeSlot,
            RunError::Deck(e) => match e {
                DeckError::UnknownCard(_) => RunErrorCode::UnknownDeckCard,
                DeckError::WouldShrinkBelowMinimum => RunErrorCode::DeckAtMinimum,
                DeckError::BlankCannotBeModified => RunErrorCode::BlankCannotBeModified,
                DeckError::AlreadyModified(_) => RunErrorCode::AlreadyModified,
                DeckError::MissingCard(_) => RunErrorCode::MissingCard,
            },
            RunError::TargetRequired => RunErrorCode::TargetRequired,
            RunError::NoModAvailable => RunErrorCode::NoModAvailable,
            RunError::UnknownContent(_) => RunErrorCode::UnknownContent,
            RunError::InvalidStart => RunErrorCode::InvalidStart,
            RunError::GameNotAvailable => RunErrorCode::GameNotAvailable,
            RunError::OuvertRequired => RunErrorCode::OuvertRequired,
            RunError::NoSkatSwapLeft => RunErrorCode::NoSkatSwapLeft,
            RunError::NothingToTake => RunErrorCode::NothingToTake,
            RunError::RamschRequired => RunErrorCode::RamschRequired,
        }
    }
}

impl fmt::Display for RunError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RunError::WrongPhase => f.write_str("the action does not fit the current phase"),
            RunError::Game(e) => write!(f, "game: {e}"),
            RunError::NoPassesLeft => f.write_str("no passes left in this series"),
            RunError::HandRequired => f.write_str("the boss demands a hand game"),
            RunError::NullNotAvailable => f.write_str("the declarer does not play Null"),
            RunError::NoRedealLeft => f.write_str("no redeal left in this series"),
            RunError::AwaitingKontraAnswer => f.write_str("Kontra must be answered first"),
            RunError::NoKontra => f.write_str("there is no Kontra to answer"),
            RunError::UnknownCard(id) => write!(f, "no card with id {id} in hand"),
            RunError::InvalidSlot => f.write_str("no item in this slot"),
            RunError::NotEnoughCoins => f.write_str("not enough Groschen"),
            RunError::NoFreeSlot => f.write_str("all slots are taken"),
            RunError::Deck(e) => write!(f, "deck: {e}"),
            RunError::TargetRequired => f.write_str("this consumable needs a target card"),
            RunError::NoModAvailable => f.write_str("no modification can be added to this card"),
            RunError::UnknownContent(id) => write!(f, "unknown content {id}"),
            RunError::InvalidStart => f.write_str("the run cannot start at this series"),
            RunError::GameNotAvailable => f.write_str("this character cannot declare that game"),
            RunError::OuvertRequired => f.write_str("an Ouvert game is required"),
            RunError::NoSkatSwapLeft => f.write_str("no skat swap left in this series"),
            RunError::NothingToTake => f.write_str("no matching card lies in the skat"),
            RunError::RamschRequired => f.write_str("the Ramsch must be played"),
        }
    }
}

impl std::error::Error for RunError {}

impl From<GameError<DealtCard>> for RunError {
    fn from(e: GameError<DealtCard>) -> RunError {
        RunError::Game(e)
    }
}

impl From<DeckError> for RunError {
    fn from(e: DeckError) -> RunError {
        RunError::Deck(e)
    }
}

/// How a run starts. The default is a fresh run of the Alleinspieler with
/// nothing locked, first series of the first section.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct RunStart {
    /// Relics owned from the start besides the character's start relics
    /// (the simulator's `--relic`), in this order after them. Their passive
    /// effects already apply to the first game.
    pub relics: Vec<RelicId>,
    /// Section of the first series, from 0.
    pub section: u32,
    /// Series of the first series within its section (2 = boss series).
    pub index: u32,
    /// Who plays the run.
    #[serde(default)]
    pub character: Character,
    /// Relics that are not unlocked yet ([`meta::MetaState`]): never on
    /// offer, never drawn by an encounter.
    #[serde(default)]
    pub locked_relics: Vec<RelicId>,
    /// Card modifications that are not unlocked yet: never drawn at
    /// random (a consumable or encounter that names one still gives it).
    #[serde(default)]
    pub locked_card_mods: Vec<CardModId>,
}

/// A run. Create with [`Run::new`], drive with [`Run::apply`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Run {
    seed: u64,
    /// Deals: one fork per game, so shopping never changes the cards.
    deal_rng: Rng,
    /// Tresen offers.
    shop_rng: Rng,
    /// Bosses, random card modifications.
    misc_rng: Rng,
    phase: RunPhase,
    series: Series,
    coins: u32,
    deck: Deck,
    relics: Vec<RelicId>,
    consumables: Vec<ConsumableId>,
    flags: BTreeSet<String>,
    counters: BTreeMap<String, u32>,
    /// Games dealt in the run; drives the fixed forehand rotation.
    games_dealt: u32,
    game: Option<ActiveGame>,
    shop: Option<Shop>,
    last_score: Option<ScoreBreakdown>,
    /// Statistics (absent in saves from before they existed).
    #[serde(default)]
    stats: RunStats,
    /// Boss of the current section, drawn when the section begins (absent
    /// in saves from before; then it is drawn at the next series start).
    #[serde(default)]
    section_boss: Option<BossId>,
    /// Who plays the run.
    #[serde(default)]
    character: Character,
    /// Relics that are not unlocked: never on offer.
    #[serde(default)]
    locked: BTreeSet<RelicId>,
    /// Card modifications that are not unlocked: never drawn at random.
    #[serde(default)]
    locked_card_mods: BTreeSet<CardModId>,
    /// Encounters and what they draw.
    #[serde(default = "Run::default_encounter_rng")]
    encounter_rng: Rng,
    /// The open encounter.
    #[serde(default)]
    encounter: Option<ActiveEncounter>,
    /// House rules of the evening (encounter "Stammtisch-Streit"): active
    /// until the section ends, without a slot.
    #[serde(default)]
    evening_relics: Vec<RelicId>,
    /// "Letzte Runde": the next game declared counts this many times.
    #[serde(default)]
    pending_factor: Option<(EncounterId, u32)>,
}

impl Run {
    /// Start a run: the first series begins and its first game is dealt.
    pub fn new(content: &Content, seed: u64) -> Result<(Run, Vec<RunEvent>), RunError> {
        Run::new_with(content, seed, &RunStart::default())
    }

    /// Start a run as `start` describes. The deals depend on the seed only,
    /// so a run with start relics sees the same cards as one without. The
    /// character's start relics come first, then `start.relics`; a relic
    /// named twice is owned once.
    pub fn new_with(
        content: &Content,
        seed: u64,
        start: &RunStart,
    ) -> Result<(Run, Vec<RunEvent>), RunError> {
        let sections = u32::try_from(content.run.targets.len()).unwrap_or(0);
        if start.section >= sections || start.index >= SERIES_PER_SECTION {
            return Err(RunError::InvalidStart);
        }
        let character = content
            .character(start.character)
            .ok_or_else(|| RunError::UnknownContent(start.character.to_string()))?;
        let mut relics: Vec<RelicId> = Vec::new();
        for relic in character.start_relics.iter().chain(&start.relics) {
            if content.relic(relic).is_none() {
                return Err(RunError::UnknownContent(relic.to_string()));
            }
            if !relics.contains(relic) {
                relics.push(relic.clone());
            }
        }
        if relics.len() > usize::try_from(content.run.relic_slots).unwrap_or(0) {
            return Err(RunError::NoFreeSlot);
        }
        let mut deck = Deck::standard();
        deck.apply_spec(&character.deck)?;
        let mut master = Rng::from_seed(seed);
        let mut run = Run {
            seed,
            deal_rng: master.fork(),
            shop_rng: master.fork(),
            misc_rng: master.fork(),
            phase: RunPhase::Game,
            series: Series {
                section: 0,
                index: 0,
                target: 0,
                score: 0,
                games_played: 0,
                game_limit: 0,
                passes_left: 0,
                boss: None,
                redeals_used: 0,
                passes_used: 0,
                skat_swaps_used: 0,
                skat_swaps_taken: Vec::new(),
                ramsch_played: false,
            },
            coins: content.run.economy.starting_coins,
            deck,
            relics,
            consumables: Vec::new(),
            flags: BTreeSet::new(),
            counters: BTreeMap::new(),
            games_dealt: 0,
            game: None,
            shop: None,
            last_score: None,
            stats: RunStats::default(),
            section_boss: None,
            character: start.character,
            locked: start.locked_relics.iter().cloned().collect(),
            locked_card_mods: start.locked_card_mods.iter().cloned().collect(),
            encounter_rng: master.fork(),
            encounter: None,
            evening_relics: Vec::new(),
            pending_factor: None,
        };
        let mut events = Vec::new();
        run.start_series(content, start.section, start.index, &mut events)?;
        Ok((run, events))
    }

    /// Encounter generator of saves from before encounters existed:
    /// derived from nothing but a constant, so loading stays
    /// deterministic.
    fn default_encounter_rng() -> Rng {
        Rng::from_seed(0x5354_414D_4D54_4953)
    }

    // -----------------------------------------------------------------------
    // Read access
    // -----------------------------------------------------------------------

    /// Who plays the run.
    #[must_use]
    pub const fn character(&self) -> Character {
        self.character
    }

    /// Relics that are not unlocked in this run.
    #[must_use]
    pub const fn locked_relics(&self) -> &BTreeSet<RelicId> {
        &self.locked
    }

    /// Card modifications that are not unlocked in this run.
    #[must_use]
    pub const fn locked_card_mods(&self) -> &BTreeSet<CardModId> {
        &self.locked_card_mods
    }

    /// The open encounter.
    #[must_use]
    pub const fn encounter(&self) -> Option<&ActiveEncounter> {
        self.encounter.as_ref()
    }

    /// House rules of the evening: active until the section ends.
    #[must_use]
    pub fn evening_relics(&self) -> &[RelicId] {
        &self.evening_relics
    }

    /// The factor waiting for the next declared game ("Letzte Runde").
    #[must_use]
    pub fn pending_factor(&self) -> Option<u32> {
        self.pending_factor.as_ref().map(|(_, f)| *f)
    }

    /// The run's seed.
    #[must_use]
    pub const fn seed(&self) -> u64 {
        self.seed
    }

    /// What the run waits for.
    #[must_use]
    pub const fn phase(&self) -> RunPhase {
        self.phase
    }

    /// The current series.
    #[must_use]
    pub const fn series(&self) -> &Series {
        &self.series
    }

    /// Groschen.
    #[must_use]
    pub const fn coins(&self) -> u32 {
        self.coins
    }

    /// The shared deck.
    #[must_use]
    pub const fn deck(&self) -> &Deck {
        &self.deck
    }

    /// Owned relics in purchase order.
    #[must_use]
    pub fn relics(&self) -> &[RelicId] {
        &self.relics
    }

    /// Owned consumables.
    #[must_use]
    pub fn consumables(&self) -> &[ConsumableId] {
        &self.consumables
    }

    /// Run flags set by content.
    #[must_use]
    pub const fn flags(&self) -> &BTreeSet<String> {
        &self.flags
    }

    /// Run counters raised by content.
    #[must_use]
    pub const fn counters(&self) -> &BTreeMap<String, u32> {
        &self.counters
    }

    /// The game in progress.
    #[must_use]
    pub const fn game(&self) -> Option<&ActiveGame> {
        self.game.as_ref()
    }

    /// The Tresen, while open.
    #[must_use]
    pub const fn shop(&self) -> Option<&Shop> {
        self.shop.as_ref()
    }

    /// Boss of the current section, known from the start of the section.
    #[must_use]
    pub const fn section_boss(&self) -> Option<&BossId> {
        self.section_boss.as_ref()
    }

    /// Breakdown of the last scored game.
    #[must_use]
    pub const fn last_score(&self) -> Option<&ScoreBreakdown> {
        self.last_score.as_ref()
    }

    /// Statistics of the run so far.
    #[must_use]
    pub const fn stats(&self) -> &RunStats {
        &self.stats
    }

    /// Cards marked "Durchsichtig" in the current game and who holds them
    /// (`None`: in the skat or already played).
    #[must_use]
    pub fn revealed_holders(&self) -> Vec<(DealtCard, Option<Seat>)> {
        let Some(active) = &self.game else {
            return Vec::new();
        };
        active
            .game
            .cards_in_play()
            .iter()
            .filter(|c| c.revealed)
            .map(|&c| (c, active.game.holder_of(c)))
            .collect()
    }

    // -----------------------------------------------------------------------
    // Actions
    // -----------------------------------------------------------------------

    /// Apply a player decision. On error nothing changed.
    pub fn apply(
        &mut self,
        content: &Content,
        action: RunAction,
    ) -> Result<Vec<RunEvent>, RunError> {
        let mut events = Vec::new();
        // Work on a copy so that a failing action leaves the run untouched.
        let mut next = self.clone();
        next.dispatch(content, action, &mut events)?;
        *self = next;
        Ok(events)
    }

    fn dispatch(
        &mut self,
        content: &Content,
        action: RunAction,
        events: &mut Vec<RunEvent>,
    ) -> Result<(), RunError> {
        match action {
            RunAction::Pass => self.pass(content, events),
            RunAction::Redeal => self.redeal_series(content, events),
            RunAction::Declare(declaration) => self.declare(content, declaration, events),
            RunAction::AnswerKontra { re } => self.answer_kontra(content, re, events),
            RunAction::Discard(ids) => self.discard(content, ids, events),
            RunAction::Play(id) => self.play(content, id, events),
            RunAction::UseConsumable { slot, target } => {
                self.use_consumable(content, slot, target, events)
            }
            RunAction::BuyRelic(slot) => self.buy_relic(content, slot, events),
            RunAction::BuyConsumable(slot) => self.buy_consumable(content, slot, events),
            RunAction::SellRelic(index) => self.sell_relic(content, index, events),
            RunAction::SellConsumable(index) => self.sell_consumable(content, index, events),
            RunAction::Reroll => self.reroll(content, events),
            RunAction::LeaveShop => self.leave_shop(content, events),
            RunAction::TakeFromSkat { give, take } => self.take_from_skat(give, take, events),
            RunAction::ChooseEncounter { option, target } => {
                self.choose_encounter(content, option, target, events)
            }
        }
    }

    fn active_mut(&mut self) -> Result<&mut ActiveGame, RunError> {
        if self.phase != RunPhase::Game {
            return Err(RunError::WrongPhase);
        }
        self.game.as_mut().ok_or(RunError::WrongPhase)
    }

    fn declaring(&mut self) -> Result<&mut ActiveGame, RunError> {
        let active = self.active_mut()?;
        if active.rules.ramsch {
            return Err(RunError::RamschRequired);
        }
        if active.game.phase() != Phase::Declaring {
            return Err(RunError::WrongPhase);
        }
        Ok(active)
    }

    fn pass(&mut self, content: &Content, events: &mut Vec<RunEvent>) -> Result<(), RunError> {
        self.declaring()?;
        if self.series.passes_left == 0 {
            return Err(RunError::NoPassesLeft);
        }
        self.series.passes_left -= 1;
        self.series.passes_used += 1;
        self.series.games_played += 1;
        self.stats.games_passed += 1;
        events.push(RunEvent::Passed {
            passes_left: self.series.passes_left,
        });
        self.after_game(content, events)
    }

    /// House rule "Schieben": deal the game again, once per series.
    fn redeal_series(
        &mut self,
        content: &Content,
        events: &mut Vec<RunEvent>,
    ) -> Result<(), RunError> {
        self.declaring()?;
        if self.redeals_left() == 0 {
            return Err(RunError::NoRedealLeft);
        }
        self.series.redeals_used += 1;
        self.redeal(content, events)
    }

    /// Throw the dealt game in and deal it again before the declaration,
    /// without using up a game. The forehand stays.
    fn redeal(&mut self, content: &Content, events: &mut Vec<RunEvent>) -> Result<(), RunError> {
        self.declaring()?;
        events.push(RunEvent::Redealt);
        self.deal(content, false, events)
    }

    fn declare(
        &mut self,
        content: &Content,
        declaration: Declaration,
        events: &mut Vec<RunEvent>,
    ) -> Result<(), RunError> {
        let character = self.character;
        let active = self.declaring()?;
        match (character.plays_null(), declaration) {
            (_, Declaration::Ramsch) => return Err(RunError::GameNotAvailable),
            (false, Declaration::Null { .. }) => return Err(RunError::NullNotAvailable),
            (true, Declaration::Suit { .. } | Declaration::Grand { .. }) => {
                return Err(RunError::GameNotAvailable);
            }
            _ => {}
        }
        if active.rules.require_hand && !declaration.hand() {
            return Err(RunError::HandRequired);
        }
        if active.rules.require_ouvert && !declaration.ouvert() {
            return Err(RunError::OuvertRequired);
        }
        let game_events = active.game.declare(declaration)?;
        events.extend(game_events.into_iter().map(RunEvent::Game));
        self.after_declaration(content, events);
        if let Some(active) = self.game.as_mut() {
            if active.rules.kontra_source.is_some() {
                let kontra_seat = Seat::DEFENDERS.into_iter().find(|&seat| {
                    DefenderHeuristic.says_kontra(active.game.hand(seat), declaration.game())
                });
                if let Some(seat) = kontra_seat {
                    active.kontra = Kontra::Kontra(seat);
                    active.awaiting_re = true;
                    events.push(RunEvent::Kontra { seat });
                }
            }
        }
        self.advance(content, events)
    }

    /// After any declaration (also the Ramsch): fire `OnDeclare` and use
    /// up the factor of "Letzte Runde".
    fn after_declaration(&mut self, content: &Content, events: &mut Vec<RunEvent>) {
        self.fire(content, Stage::OnDeclare, None, None, events);
        if let Some((encounter, factor)) = self.pending_factor.take() {
            if let Some(active) = self.game.as_mut() {
                active.lines.push(ScoreLine {
                    source: Source::Encounter(encounter),
                    lever: Lever::GameValue,
                    change: Change::Multiply(factor),
                });
            }
        }
    }

    /// Skat swaps ("Die Oma", "Der Opa") still open in this series: the
    /// granted filters minus those used, in the order they were granted.
    fn open_skat_swaps(&self) -> Vec<CardFilter> {
        let Some(active) = &self.game else {
            return Vec::new();
        };
        let mut open = active.rules.skat_swaps.clone();
        let taken = &self.series.skat_swaps_taken;
        for filter in taken {
            if let Some(i) = open.iter().position(|f| f == filter) {
                open.remove(i);
            }
        }
        // Saves from before the list: the first swaps count as used.
        let untracked = usize::try_from(self.series.skat_swaps_used)
            .unwrap_or(usize::MAX)
            .saturating_sub(taken.len());
        open.drain(..untracked.min(open.len()));
        open
    }

    /// Skat cards an open skat swap may take now, the preferred one first
    /// (the strongest: jacks by the jack order, then by rank). Empty
    /// outside a hand game before the player's first card.
    #[must_use]
    pub fn skat_swap_cards(&self) -> Vec<DealtCard> {
        let open = self.open_skat_swaps();
        let Some(active) = &self.game else {
            return Vec::new();
        };
        let game = &active.game;
        let ready = game.phase() == Phase::Playing
            && game
                .declaration()
                .is_some_and(|d| d.hand() && !d.is_ramsch())
            && game.hand(Seat::DECLARER).len() == crate::game::HAND_SIZE
            && !active.awaiting_re;
        if open.is_empty() || !ready {
            return Vec::new();
        }
        let mut cards: Vec<DealtCard> = game
            .skat()
            .into_iter()
            .filter(|c| c.face().is_some_and(|f| open.iter().any(|o| o.matches(f))))
            .collect();
        cards.sort_by_key(|c| {
            std::cmp::Reverse((
                trump_strength(GameType::Grand, *c),
                suit_strength(GameType::Grand, *c),
                c.id,
            ))
        });
        cards
    }

    /// Skat swap ("Die Oma", "Der Opa"): take a matching card from the
    /// untouched skat of a hand game (`take`, or the preferred one) and put
    /// `give` in its place. Each granted filter works once per series.
    fn take_from_skat(
        &mut self,
        give: u32,
        take: Option<u32>,
        events: &mut Vec<RunEvent>,
    ) -> Result<(), RunError> {
        let open = self.open_skat_swaps();
        let candidates = self.skat_swap_cards();
        let active = self.active_mut()?;
        if active.awaiting_re {
            return Err(RunError::AwaitingKontraAnswer);
        }
        if open.is_empty() {
            return Err(RunError::NoSkatSwapLeft);
        }
        let gave = Run::declarer_card(active, give)?;
        let took = match take {
            Some(id) => candidates.iter().copied().find(|c| c.id == id),
            None => candidates.first().copied(),
        }
        .ok_or(RunError::NothingToTake)?;
        let filter = open
            .iter()
            .copied()
            .find(|f| took.face().is_some_and(|face| f.matches(face)))
            .ok_or(RunError::NothingToTake)?;
        let game_events = active.game.swap_with_skat(took, gave)?;
        events.extend(game_events.into_iter().map(RunEvent::Game));
        self.series.skat_swaps_used += 1;
        self.series.skat_swaps_taken.push(filter);
        Ok(())
    }

    fn answer_kontra(
        &mut self,
        content: &Content,
        re: bool,
        events: &mut Vec<RunEvent>,
    ) -> Result<(), RunError> {
        let active = self.active_mut()?;
        if !active.awaiting_re {
            return Err(RunError::NoKontra);
        }
        active.awaiting_re = false;
        if re {
            if let Kontra::Kontra(seat) = active.kontra {
                active.kontra = Kontra::Re(seat);
                events.push(RunEvent::Re);
            }
        }
        self.advance(content, events)
    }

    /// Find a card of the declarer's hand by instance id.
    fn declarer_card(active: &ActiveGame, id: u32) -> Result<DealtCard, RunError> {
        active
            .game
            .hand(Seat::DECLARER)
            .iter()
            .copied()
            .find(|c| c.id == id)
            .ok_or(RunError::UnknownCard(id))
    }

    fn discard(
        &mut self,
        content: &Content,
        ids: [u32; 2],
        events: &mut Vec<RunEvent>,
    ) -> Result<(), RunError> {
        let active = self.active_mut()?;
        if active.awaiting_re {
            return Err(RunError::AwaitingKontraAnswer);
        }
        let cards = [
            Run::declarer_card(active, ids[0])?,
            Run::declarer_card(active, ids[1])?,
        ];
        let game_events = active.game.discard(cards)?;
        events.extend(game_events.into_iter().map(RunEvent::Game));
        self.advance(content, events)
    }

    fn play(
        &mut self,
        content: &Content,
        id: u32,
        events: &mut Vec<RunEvent>,
    ) -> Result<(), RunError> {
        let active = self.active_mut()?;
        if active.awaiting_re {
            return Err(RunError::AwaitingKontraAnswer);
        }
        let card = Run::declarer_card(active, id)?;
        self.play_card(content, Seat::DECLARER, card, events)?;
        self.advance(content, events)
    }

    /// Play one card and fire the in-play hooks.
    fn play_card(
        &mut self,
        content: &Content,
        seat: Seat,
        card: DealtCard,
        events: &mut Vec<RunEvent>,
    ) -> Result<(), RunError> {
        let active = self.active_mut()?;
        let game_events = active.game.play(seat, card)?;
        let trick_done = game_events
            .iter()
            .any(|e| matches!(e, Event::TrickWon { .. }));
        events.extend(game_events.into_iter().map(RunEvent::Game));
        self.fire(
            content,
            Stage::OnCardPlayed,
            Some((seat, card)),
            None,
            events,
        );
        if trick_done {
            self.fire(content, Stage::OnTrickEnd, None, None, events);
        }
        Ok(())
    }

    /// Let the defenders play until it is the declarer's turn, then score a
    /// finished game.
    fn advance(&mut self, content: &Content, events: &mut Vec<RunEvent>) -> Result<(), RunError> {
        loop {
            let Some(active) = self.game.as_ref() else {
                return Ok(());
            };
            match active.game.phase() {
                Phase::Finished(_) => return self.finish_game(content, events),
                Phase::Playing if !active.awaiting_re => {
                    let Some(seat) = active.game.current_seat() else {
                        return Ok(());
                    };
                    if seat.is_declarer() {
                        return Ok(());
                    }
                    let view = active.game.view(seat).ok_or(RunError::WrongPhase)?;
                    let card = crate::ai::Policy::choose(&DefenderHeuristic, &view)
                        .ok_or(RunError::WrongPhase)?;
                    self.play_card(content, seat, card, events)?;
                }
                _ => return Ok(()),
            }
        }
    }

    // -----------------------------------------------------------------------
    // Content hooks
    // -----------------------------------------------------------------------

    /// The boss whose rule is on in the current game: the series' boss, or
    /// for a boss with a rotation (Endboss "Der Alte") the rule of this
    /// game, by the number of games dealt in the series. Bosses not in the
    /// character's pool are skipped.
    fn boss_rule(&self, content: &Content) -> Option<BossId> {
        let def = content.boss(self.series.boss.as_ref()?)?;
        if def.rotation.is_empty() {
            return Some(def.id.clone());
        }
        let rules: Vec<&BossId> = def
            .rotation
            .iter()
            .filter(|id| {
                content
                    .boss(id)
                    .is_some_and(|b| in_pool(&b.characters, self.character))
            })
            .collect();
        let n = u32::try_from(rules.len()).ok().filter(|&n| n > 0)?;
        let i = usize::try_from(self.series.games_played % n).unwrap_or(0);
        rules.get(i).map(|id| (*id).clone())
    }

    /// Boss, relics and house rules of the evening, plus the card
    /// modifications in play if a game runs.
    fn sources<'c>(&self, content: &'c Content, with_card_mods: bool) -> Vec<ActiveSource<'c>> {
        let mut sources = Vec::new();
        if let Some(def) = self.boss_rule(content).and_then(|b| content.boss(&b)) {
            sources.push(ActiveSource {
                source: Source::Boss(def.id.clone()),
                passive: &def.passive,
                hooks: &def.hooks,
                this_card: None,
            });
        }
        let relics = self.relics.iter().chain(&self.evening_relics);
        for def in relics.filter_map(|r| content.relic(r)) {
            sources.push(ActiveSource {
                source: Source::Relic(def.id.clone()),
                passive: &def.passive,
                hooks: &def.hooks,
                this_card: None,
            });
        }
        if with_card_mods {
            if let Some(active) = &self.game {
                sources.extend(card_mod_sources(&self.deck, content, &active.game));
            }
        }
        sources
    }

    /// Fire every hook of `stage` whose condition holds.
    fn fire(
        &mut self,
        content: &Content,
        stage: Stage,
        played: Option<(Seat, DealtCard)>,
        result: Option<&ValueParts>,
        events: &mut Vec<RunEvent>,
    ) {
        let games_left = self.series.games_left();
        let sources = self.sources(content, stage.in_game());
        for source in &sources {
            for hook in source.hooks.iter().filter(|h| h.stage() == stage) {
                let holds = {
                    let ctx = HookContext {
                        flags: &self.flags,
                        counters: &self.counters,
                        game: self.game.as_ref().map(|a| &a.game),
                        played,
                        result,
                        games_left,
                    };
                    evaluate(hook.when(), &ctx, source.this_card)
                };
                if holds {
                    for effect in hook.then() {
                        self.apply_effect(&source.source, effect, events);
                    }
                }
            }
        }
    }

    /// Apply one effect. Card-level effects were applied when the deck was
    /// resolved and are ignored here.
    fn apply_effect(&mut self, source: &Source, effect: &Effect, events: &mut Vec<RunEvent>) {
        let line = |lever, change| ScoreLine {
            source: source.clone(),
            lever,
            change,
        };
        match effect {
            Effect::SetFlag(flag) => {
                if self.flags.insert(flag.clone()) {
                    events.push(RunEvent::FlagSet(flag.clone()));
                }
                return;
            }
            Effect::ClearFlag(flag) => {
                if self.flags.remove(flag) {
                    events.push(RunEvent::FlagCleared(flag.clone()));
                }
                return;
            }
            Effect::IncrementCounter(counter) => {
                let value = self.counters.entry(counter.clone()).or_insert(0);
                *value = value.saturating_add(1);
                events.push(RunEvent::CounterRaised {
                    counter: counter.clone(),
                    value: *value,
                });
                return;
            }
            Effect::GainCoins(amount) => {
                if *amount != 0 {
                    self.coins = if *amount > 0 {
                        self.coins.saturating_add(amount.unsigned_abs())
                    } else {
                        self.coins.saturating_sub(amount.unsigned_abs())
                    };
                    events.push(RunEvent::CoinsGained {
                        source: source.clone(),
                        amount: *amount,
                        coins: self.coins,
                    });
                }
                return;
            }
            _ => {}
        }
        let counter_value = |name: &str| self.counters.get(name).copied().unwrap_or(0);
        let from_counter = match effect {
            Effect::AddBaseValueFromCounter(name) => counter_value(name),
            _ => 0,
        };
        let ramsch_played = self.series.ramsch_played;
        let Some(active) = self.game.as_mut() else {
            return;
        };
        if effect.is_cheat() && matches!(source, Source::Boss(_)) && active.rules.no_cheating {
            return;
        }
        match effect {
            Effect::AddBaseValue(n) => active
                .lines
                .push(line(Lever::BaseValue, Change::Add(i64::from(*n)))),
            Effect::MultiplyBaseValue(n) => active
                .lines
                .push(line(Lever::BaseValue, Change::Multiply(*n))),
            Effect::AddBaseValueFromCounter(_) => {
                if from_counter > 0 {
                    active
                        .lines
                        .push(line(Lever::BaseValue, Change::Add(i64::from(from_counter))));
                }
            }
            Effect::AddLevels(n) => active
                .lines
                .push(line(Lever::Levels, Change::Add(i64::from(*n)))),
            Effect::MultiplyGameValue(n) => active
                .lines
                .push(line(Lever::GameValue, Change::Multiply(*n))),
            Effect::WinThreshold(n) => {
                if *n < active.rules.thresholds.win {
                    active.rules.thresholds.win = *n;
                    active.rules.win_threshold_source = Some(source.clone());
                }
            }
            Effect::SchneiderFromDeclarerPoints { min, max } => {
                active
                    .rules
                    .thresholds
                    .declarer_schneider
                    .push((*min, *max));
                active
                    .rules
                    .schneider_sources
                    .push((source.clone(), (*min, *max)));
            }
            Effect::RequireHand => active.rules.require_hand = true,
            Effect::EnableKontra => {
                if active.rules.kontra_source.is_none() {
                    active.rules.kontra_source = Some(source.clone());
                }
            }
            Effect::SeriesRedeal => {
                active.rules.series_redeals = active.rules.series_redeals.max(1)
            }
            Effect::Concede => {
                if active.game.phase() == Phase::Playing && active.game.current_trick().is_empty() {
                    if let Ok(game_events) = active.game.concede() {
                        events.extend(game_events.into_iter().map(RunEvent::Game));
                    }
                }
            }
            Effect::PeekDefenderCards(n) => {
                if !active.rules.no_information {
                    active.rules.peek = active.rules.peek.saturating_add(*n);
                }
            }
            Effect::ShowCount => active.rules.show_count = !active.rules.no_information,
            Effect::SeriesSkatSwap(filter) => active.rules.skat_swaps.push(*filter),
            Effect::NoCheating => active.rules.no_cheating = true,
            Effect::RequireOuvert => active.rules.require_ouvert = true,
            Effect::DefendersSeeEachOther => active.game.open_defender_hands(),
            Effect::SwapDefenderHands => {
                if active.game.phase() == Phase::Playing && active.game.current_trick().is_empty() {
                    if let Ok(game_events) = active.game.swap_defender_hands() {
                        events.extend(game_events.into_iter().map(RunEvent::Game));
                    }
                }
            }
            Effect::RamschOnce => active.rules.ramsch = !ramsch_played,
            Effect::DisableInformation => {
                active.rules.no_information = true;
                active.rules.peek = 0;
                active.rules.show_count = false;
            }
            Effect::SetCardPoints { .. }
            | Effect::AddCardPoints(_)
            | Effect::RankBonus(_)
            | Effect::RevealHolder
            | Effect::NeverWins
            | Effect::HonoraryJack
            | Effect::NoDiscard
            | Effect::ShopDiscount(_)
            | Effect::BlankOut(_)
            | Effect::SetFlag(_)
            | Effect::ClearFlag(_)
            | Effect::IncrementCounter(_)
            | Effect::GainCoins(_) => {}
        }
    }

    // -----------------------------------------------------------------------
    // Series flow
    // -----------------------------------------------------------------------

    fn start_series(
        &mut self,
        content: &Content,
        section: u32,
        index: u32,
        events: &mut Vec<RunEvent>,
    ) -> Result<(), RunError> {
        let config = &content.run;
        if usize::try_from(section).unwrap_or(usize::MAX) >= config.targets.len() {
            return Err(RunError::WrongPhase);
        }
        if index == 0 && !self.evening_relics.is_empty() {
            self.evening_relics.clear();
            events.push(RunEvent::EveningRulesEnded);
        }
        if index == 0 || self.section_boss.is_none() {
            let last = usize::try_from(section).unwrap_or(0) + 1 == config.targets.len();
            self.section_boss = match (&config.final_boss, last) {
                (Some(boss), true) => Some(boss.clone()),
                _ => {
                    let pool: Vec<&BossId> = config
                        .bosses
                        .iter()
                        .filter(|id| {
                            content
                                .boss(id)
                                .is_some_and(|b| in_pool(&b.characters, self.character))
                        })
                        .collect();
                    let pick = self
                        .misc_rng
                        .gen_range(0..u64::try_from(pool.len()).unwrap_or(1));
                    pool.get(usize::try_from(pick).unwrap_or(0))
                        .map(|id| (*id).clone())
                }
            };
            if let Some(boss) = &self.section_boss {
                events.push(RunEvent::BossAnnounced {
                    section,
                    boss: boss.clone(),
                });
            }
        }
        let boss = if index + 1 == SERIES_PER_SECTION {
            self.section_boss.clone()
        } else {
            None
        };
        let target = series_target(content, self.character, section, index, boss.as_ref())
            .ok_or(RunError::WrongPhase)?;
        self.series = Series {
            section,
            index,
            target,
            score: 0,
            games_played: 0,
            game_limit: config.game_limit,
            passes_left: config.passes,
            boss: boss.clone(),
            redeals_used: 0,
            passes_used: 0,
            skat_swaps_used: 0,
            skat_swaps_taken: Vec::new(),
            ramsch_played: false,
        };
        self.phase = RunPhase::Game;
        self.shop = None;
        self.encounter = None;
        events.push(RunEvent::SeriesStarted {
            section,
            index,
            target,
            game_limit: config.game_limit,
            passes: config.passes,
            boss,
        });
        self.deal(content, true, events)
    }

    /// Set up and deal a game. `rotate` advances the forehand (not for a
    /// redeal of the same game).
    fn deal(
        &mut self,
        content: &Content,
        rotate: bool,
        events: &mut Vec<RunEvent>,
    ) -> Result<(), RunError> {
        let rotation = if rotate {
            self.games_dealt
        } else {
            self.games_dealt.saturating_sub(1)
        };
        let forehand =
            Seat::from_index(usize::try_from(rotation % 3).unwrap_or(0)).unwrap_or(Seat::DECLARER);
        if rotate {
            self.games_dealt += 1;
        }
        let global = self.sources(content, false);
        let resolved = resolve_deck(&self.deck, content, &global);
        let mut rng = self.deal_rng.fork();
        let game = Game::deal_from(&mut rng, resolved.cards, forehand)?;
        self.set_up_game(content, game, resolved.point_changes, events);
        self.peek(&mut rng, events);
        if self.game.as_ref().is_some_and(|a| a.rules.ramsch) {
            return self.start_ramsch(content, events);
        }
        Ok(())
    }

    /// "Der Kiebitz": show the player random cards of the defenders, drawn
    /// with the game's own generator (so the deal stays the same).
    fn peek(&mut self, rng: &mut Rng, events: &mut Vec<RunEvent>) {
        let Some(active) = self.game.as_mut() else {
            return;
        };
        let mut pool: Vec<(DealtCard, Seat)> = Seat::DEFENDERS
            .iter()
            .flat_map(|&seat| active.game.hand(seat).iter().map(move |&c| (c, seat)))
            .collect();
        for _ in 0..active.rules.peek {
            if pool.is_empty() {
                break;
            }
            let pick = rng.gen_range(0..u64::try_from(pool.len()).unwrap_or(1));
            let (card, seat) = pool.remove(usize::try_from(pick).unwrap_or(0));
            active.peeked.push((card, seat));
            events.push(RunEvent::CardPeeked { card, seat });
        }
    }

    /// "Der Ramscher": the dealt game is the series' Ramsch. Nobody
    /// declares; it is played at once.
    fn start_ramsch(
        &mut self,
        content: &Content,
        events: &mut Vec<RunEvent>,
    ) -> Result<(), RunError> {
        self.series.ramsch_played = true;
        let active = self.active_mut()?;
        let game_events = active.game.declare(Declaration::Ramsch)?;
        events.push(RunEvent::RamschDealt);
        events.extend(game_events.into_iter().map(RunEvent::Game));
        self.after_declaration(content, events);
        self.advance(content, events)
    }

    /// Install a freshly dealt game, apply passive effects and fire
    /// `OnDeal`.
    fn set_up_game(
        &mut self,
        content: &Content,
        game: Game<DealtCard>,
        point_changes: Vec<(u32, Source, i32)>,
        events: &mut Vec<RunEvent>,
    ) {
        let forehand = game.forehand();
        let dealt_events: Vec<Event<DealtCard>> = game.log().to_vec();
        let global = self.sources(content, false);
        let has = |wanted: &Effect| global.iter().any(|s| s.passive.contains(wanted));
        let rules = GameRules {
            no_cheating: has(&Effect::NoCheating),
            no_information: has(&Effect::DisableInformation),
            ..GameRules::default()
        };
        let boss_rule = self.boss_rule(content);
        self.game = Some(ActiveGame {
            game,
            rules,
            lines: Vec::new(),
            point_changes,
            kontra: Kontra::None,
            awaiting_re: false,
            peeked: Vec::new(),
            boss_rule,
        });
        events.push(RunEvent::GameDealt {
            number: self.series.games_played + 1,
            forehand,
        });
        events.extend(dealt_events.into_iter().map(RunEvent::Game));
        // Passive effects of boss, relics and card mods in play.
        let sources = self.sources(content, true);
        for source in &sources {
            for effect in source.passive {
                self.apply_effect(&source.source, effect, events);
            }
        }
        self.fire(content, Stage::OnDeal, None, None, events);
    }

    /// Score the finished game and move on.
    fn finish_game(
        &mut self,
        content: &Content,
        events: &mut Vec<RunEvent>,
    ) -> Result<(), RunError> {
        let Some(active) = self.game.as_ref() else {
            return Ok(());
        };
        let game = &active.game;
        let declaration = game.declaration().ok_or(RunError::WrongPhase)?;
        let outcome = game.outcome();
        let matadors = game.matadors().ok_or(RunError::WrongPhase)?;
        let parts = match declaration {
            Declaration::Null { risk } => {
                let scoring = self.null_scoring(content);
                let tricks = match &scoring {
                    NullScoring::Formula(f) => usize::try_from(f.short_circuit_tricks).unwrap_or(0),
                    NullScoring::Fixed => 0,
                };
                null_value_parts(risk, &scoring, &outcome, &game.null_style(tricks))
            }
            Declaration::Ramsch => ramsch_value_parts(game.seat_card_points(), game.seat_tricks()),
            Declaration::Suit { .. } | Declaration::Grand { .. } => {
                game_value_parts(declaration, matadors, &outcome, &active.rules.thresholds)
            }
        };
        self.fire(content, Stage::OnScore, None, Some(&parts), events);

        let Some(active) = self.game.as_ref() else {
            return Ok(());
        };
        let mut lines = declarer_point_lines(active);
        if parts.won_by_lowered_threshold {
            if let Some(source) = &active.rules.win_threshold_source {
                lines.push(ScoreLine {
                    source: source.clone(),
                    lever: Lever::WinThreshold,
                    change: Change::Set(i64::from(active.rules.thresholds.win)),
                });
            }
        }
        if parts.schneider_by_declarer_points {
            let points = outcome.declarer_card_points;
            if let Some((source, _)) = active
                .rules
                .schneider_sources
                .iter()
                .find(|(_, (min, max))| (*min..=*max).contains(&points))
            {
                lines.push(ScoreLine {
                    source: source.clone(),
                    lever: Lever::Schneider,
                    change: Change::Set(1),
                });
            }
        }
        lines.extend(active.lines.iter().cloned());
        match active.kontra {
            Kontra::None => {}
            Kontra::Kontra(_) => lines.push(kontra_line(Source::Kontra)),
            Kontra::Re(_) => {
                lines.push(kontra_line(Source::Kontra));
                lines.push(kontra_line(Source::Re));
            }
        }
        let mut breakdown = score_parts(
            declaration,
            matadors,
            &outcome,
            &active.rules.thresholds,
            parts,
            lines,
        );
        breakdown.seats = Some(active.game.seat_totals());
        self.series.score = self.series.score.saturating_add(breakdown.score);
        self.series.games_played += 1;
        if breakdown.won {
            self.stats.games_won += 1;
            self.stats.best_game = self.stats.best_game.max(breakdown.value);
        } else {
            self.stats.games_lost += 1;
        }
        events.push(RunEvent::GameScored {
            breakdown: breakdown.clone(),
            series_score: self.series.score,
        });
        self.last_score = Some(breakdown);
        self.after_game(content, events)
    }

    /// After a game was scored or passed: series won, run lost or next deal.
    fn after_game(
        &mut self,
        content: &Content,
        events: &mut Vec<RunEvent>,
    ) -> Result<(), RunError> {
        if self.series.target_reached() {
            return self.series_won(content, events);
        }
        if self.series.games_left() == 0 {
            self.game = None;
            self.phase = RunPhase::Lost;
            events.push(RunEvent::RunLost {
                section: self.series.section,
                index: self.series.index,
                character: self.character,
            });
            return Ok(());
        }
        self.deal(content, true, events)
    }

    fn series_won(
        &mut self,
        content: &Content,
        events: &mut Vec<RunEvent>,
    ) -> Result<(), RunError> {
        self.game = None;
        let economy = &content.run.economy;
        let games_left = self.series.games_left();
        let interest = self
            .coins
            .checked_div(economy.interest_step)
            .unwrap_or(0)
            .min(economy.interest_max);
        let payout = Payout {
            base: economy.series_reward,
            games_left: games_left * economy.per_game_left,
            interest,
        };
        self.coins = self.coins.saturating_add(payout.total());
        self.stats.series_won += 1;
        events.push(RunEvent::SeriesWon {
            section: self.series.section,
            index: self.series.index,
            payout,
            coins: self.coins,
            boss: self.series.boss.clone(),
            passes_used: self.series.passes_used,
            games_left,
        });
        self.fire(content, Stage::OnSeriesWon, None, None, events);
        let last_section = u32::try_from(content.run.targets.len()).unwrap_or(1) - 1;
        if self.series.index + 1 == SERIES_PER_SECTION && self.series.section >= last_section {
            self.phase = RunPhase::Won;
            events.push(RunEvent::RunWon {
                character: self.character,
            });
            return Ok(());
        }
        if self.open_encounter(content, events) {
            return Ok(());
        }
        self.phase = RunPhase::Shop;
        let shop = self.draw_offer(content, 0);
        events.push(RunEvent::ShopOpened {
            relics: shop.relics.clone(),
            consumables: shop.consumables.clone(),
        });
        self.shop = Some(shop);
        Ok(())
    }

    // -----------------------------------------------------------------------
    // Tresen
    // -----------------------------------------------------------------------

    /// Relics that may be offered or drawn: for the character, unlocked,
    /// neither owned nor on the table for the evening; only house rules if
    /// `house_rules_only`. In id order.
    fn relic_pool(&self, content: &Content, house_rules_only: bool) -> Vec<(RelicId, Rarity)> {
        content
            .relics
            .values()
            .filter(|r| in_pool(&r.characters, self.character))
            .filter(|r| !self.locked.contains(&r.id))
            .filter(|r| !self.relics.contains(&r.id) && !self.evening_relics.contains(&r.id))
            .filter(|r| !house_rules_only || r.kind == RelicKind::HouseRule)
            .map(|r| (r.id.clone(), r.rarity))
            .collect()
    }

    /// Draw a deterministic offer from the content pool. Relics the player
    /// owns, locked relics and items for other characters are not offered;
    /// no item twice in one offer. Rarer items are drawn less often
    /// (weights in `run.ron`).
    fn draw_offer(&mut self, content: &Content, rerolls: u32) -> Shop {
        let weights = &content.run.shop.rarity_weights;
        let relic_pool = self.relic_pool(content, false);
        let consumable_pool: Vec<(ConsumableId, Rarity)> = content
            .consumables
            .values()
            .filter(|c| in_pool(&c.characters, self.character))
            .map(|c| (c.id.clone(), c.rarity))
            .collect();
        let relics = draw_weighted(
            &mut self.shop_rng,
            relic_pool,
            content.run.shop.relic_offers,
            |r| weights.weight(r),
        );
        let consumables = draw_weighted(
            &mut self.shop_rng,
            consumable_pool,
            content.run.shop.consumable_offers,
            |r| weights.weight(r),
        );
        Shop {
            relics: relics.into_iter().map(Some).collect(),
            consumables: consumables.into_iter().map(Some).collect(),
            rerolls,
        }
    }

    /// Percent off at the Tresen from the owned relics ("Der Wirt"), at
    /// most 90.
    fn shop_discount(&self, content: &Content) -> u32 {
        let total: u32 = self
            .relics
            .iter()
            .chain(&self.evening_relics)
            .filter_map(|r| content.relic(r))
            .flat_map(|def| def.passive.iter())
            .map(|e| match e {
                Effect::ShopDiscount(p) => *p,
                _ => 0,
            })
            .sum();
        total.min(90)
    }

    /// What an item with this price costs the player now: the price minus
    /// the Tresen discount, rounded to the nearest Groschen, at least 1
    /// for a priced item. Selling always pays half the full price.
    #[must_use]
    pub fn shop_price(&self, content: &Content, price: u32) -> u32 {
        let discount = self.shop_discount(content);
        if price == 0 || discount == 0 {
            return price;
        }
        let paid = (u64::from(price) * u64::from(100 - discount) + 50) / 100;
        u32::try_from(paid).unwrap_or(price).max(1)
    }

    /// How the character's Null games are scored (Fixed if the data says
    /// nothing).
    fn null_scoring(&self, content: &Content) -> NullScoring {
        content
            .character(self.character)
            .and_then(|c| c.null_scoring.clone())
            .unwrap_or(NullScoring::Fixed)
    }

    fn shop_mut(&mut self) -> Result<&mut Shop, RunError> {
        if self.phase != RunPhase::Shop {
            return Err(RunError::WrongPhase);
        }
        self.shop.as_mut().ok_or(RunError::WrongPhase)
    }

    fn buy_relic(
        &mut self,
        content: &Content,
        slot: usize,
        events: &mut Vec<RunEvent>,
    ) -> Result<(), RunError> {
        let coins = self.coins;
        let owned = self.relics.len();
        let shop = self.shop_mut()?;
        let id = shop
            .relics
            .get(slot)
            .cloned()
            .flatten()
            .ok_or(RunError::InvalidSlot)?;
        let def = content
            .relic(&id)
            .ok_or_else(|| RunError::UnknownContent(id.to_string()))?;
        let price = self.shop_price(content, def.price);
        if price > coins {
            return Err(RunError::NotEnoughCoins);
        }
        if owned >= usize::try_from(content.run.relic_slots).unwrap_or(0) {
            return Err(RunError::NoFreeSlot);
        }
        if let Some(entry) = self.shop_mut()?.relics.get_mut(slot) {
            *entry = None;
        }
        self.coins -= price;
        self.relics.push(id.clone());
        events.push(RunEvent::RelicBought { relic: id, price });
        Ok(())
    }

    fn buy_consumable(
        &mut self,
        content: &Content,
        slot: usize,
        events: &mut Vec<RunEvent>,
    ) -> Result<(), RunError> {
        let coins = self.coins;
        let owned = self.consumables.len();
        let shop = self.shop_mut()?;
        let id = shop
            .consumables
            .get(slot)
            .cloned()
            .flatten()
            .ok_or(RunError::InvalidSlot)?;
        let def = content
            .consumable(&id)
            .ok_or_else(|| RunError::UnknownContent(id.to_string()))?;
        let price = self.shop_price(content, def.price);
        if price > coins {
            return Err(RunError::NotEnoughCoins);
        }
        if owned >= usize::try_from(content.run.consumable_slots).unwrap_or(0) {
            return Err(RunError::NoFreeSlot);
        }
        if let Some(entry) = self.shop_mut()?.consumables.get_mut(slot) {
            *entry = None;
        }
        self.coins -= price;
        self.consumables.push(id.clone());
        events.push(RunEvent::ConsumableBought {
            consumable: id,
            price,
        });
        Ok(())
    }

    fn sell_relic(
        &mut self,
        content: &Content,
        index: usize,
        events: &mut Vec<RunEvent>,
    ) -> Result<(), RunError> {
        self.shop_mut()?;
        let id = self
            .relics
            .get(index)
            .cloned()
            .ok_or(RunError::InvalidSlot)?;
        let price = content.relic(&id).map_or(0, |d| sell_price(d.price));
        self.relics.remove(index);
        self.coins = self.coins.saturating_add(price);
        events.push(RunEvent::RelicSold { relic: id, price });
        Ok(())
    }

    fn sell_consumable(
        &mut self,
        content: &Content,
        index: usize,
        events: &mut Vec<RunEvent>,
    ) -> Result<(), RunError> {
        self.shop_mut()?;
        let id = self
            .consumables
            .get(index)
            .cloned()
            .ok_or(RunError::InvalidSlot)?;
        let price = content.consumable(&id).map_or(0, |d| sell_price(d.price));
        self.consumables.remove(index);
        self.coins = self.coins.saturating_add(price);
        events.push(RunEvent::ConsumableSold {
            consumable: id,
            price,
        });
        Ok(())
    }

    fn reroll(&mut self, content: &Content, events: &mut Vec<RunEvent>) -> Result<(), RunError> {
        let rerolls = self.shop_mut()?.rerolls;
        let cost = self.shop_price(content, content.run.economy.reroll_cost);
        if cost > self.coins {
            return Err(RunError::NotEnoughCoins);
        }
        self.coins -= cost;
        let shop = self.draw_offer(content, rerolls + 1);
        events.push(RunEvent::ShopRerolled { cost });
        events.push(RunEvent::ShopOpened {
            relics: shop.relics.clone(),
            consumables: shop.consumables.clone(),
        });
        self.shop = Some(shop);
        Ok(())
    }

    fn leave_shop(
        &mut self,
        content: &Content,
        events: &mut Vec<RunEvent>,
    ) -> Result<(), RunError> {
        self.shop_mut()?;
        events.push(RunEvent::ShopLeft);
        let (section, index) = self.next_series_position();
        self.start_series(content, section, index, events)
    }

    // -----------------------------------------------------------------------
    // Consumables
    // -----------------------------------------------------------------------

    fn use_consumable(
        &mut self,
        content: &Content,
        slot: usize,
        target: Option<u32>,
        events: &mut Vec<RunEvent>,
    ) -> Result<(), RunError> {
        let id = self
            .consumables
            .get(slot)
            .cloned()
            .ok_or(RunError::InvalidSlot)?;
        let def = content
            .consumable(&id)
            .ok_or_else(|| RunError::UnknownContent(id.to_string()))?;
        match &def.action {
            ConsumableAction::RemoveCard => {
                if self.phase != RunPhase::Shop {
                    return Err(RunError::WrongPhase);
                }
                let target = target.ok_or(RunError::TargetRequired)?;
                let change = self.deck.remove(target)?;
                events.push(RunEvent::DeckChanged(change));
            }
            ConsumableAction::AddCardMod(card_mod) => {
                if self.phase != RunPhase::Shop {
                    return Err(RunError::WrongPhase);
                }
                let target = target.ok_or(RunError::TargetRequired)?;
                let card = self
                    .deck
                    .get(target)
                    .ok_or(DeckError::UnknownCard(target))?;
                if card.kind == CardKind::Blank {
                    return Err(DeckError::BlankCannotBeModified.into());
                }
                let card_mod = match card_mod {
                    Some(m) => m.clone(),
                    None => {
                        let options = self.card_mod_pool(content, &card.mods);
                        let pick = self
                            .misc_rng
                            .gen_range(0..u64::try_from(options.len()).unwrap_or(0));
                        options
                            .get(usize::try_from(pick).unwrap_or(usize::MAX))
                            .cloned()
                            .ok_or(RunError::NoModAvailable)?
                    }
                };
                self.attach_mod(content, target, card_mod, events)?;
            }
            ConsumableAction::ExtraPasses(n) => {
                if self.phase != RunPhase::Game {
                    return Err(RunError::WrongPhase);
                }
                self.series.passes_left = self.series.passes_left.saturating_add(*n);
                events.push(RunEvent::PassesAdded {
                    passes: *n,
                    passes_left: self.series.passes_left,
                });
            }
            ConsumableAction::Redeal => self.redeal(content, events)?,
        }
        self.consumables.remove(slot);
        events.push(RunEvent::ConsumableUsed { consumable: id });
        Ok(())
    }

    /// Card modifications a random draw may give a card that already
    /// carries `mods`: for the character, unlocked, not on the card yet, in
    /// id order.
    fn card_mod_pool(&self, content: &Content, mods: &[CardModId]) -> Vec<CardModId> {
        content
            .card_mods
            .values()
            .filter(|m| in_pool(&m.characters, self.character) && !mods.contains(&m.id))
            .filter(|m| !self.locked_card_mods.contains(&m.id))
            .map(|m| m.id.clone())
            .collect()
    }

    /// Attach a card modification to a deck card and fire its `OnAttach`
    /// hooks ("Ramponiert" pays Groschen).
    fn attach_mod(
        &mut self,
        content: &Content,
        card: u32,
        card_mod: CardModId,
        events: &mut Vec<RunEvent>,
    ) -> Result<(), RunError> {
        let change = self.deck.add_mod(card, card_mod.clone())?;
        events.push(RunEvent::DeckChanged(change));
        let Some(def) = content.card_mod(&card_mod) else {
            return Ok(());
        };
        let source = Source::CardMod { card_mod, card };
        for hook in def.hooks.iter().filter(|h| h.stage() == Stage::OnAttach) {
            let holds = {
                let ctx = HookContext {
                    flags: &self.flags,
                    counters: &self.counters,
                    game: None,
                    played: None,
                    result: None,
                    games_left: self.series.games_left(),
                };
                evaluate(hook.when(), &ctx, None)
            };
            if holds {
                for effect in hook.then() {
                    self.apply_effect(&source, effect, events);
                }
            }
        }
        Ok(())
    }

    // -----------------------------------------------------------------------
    // Encounters (Ereignisse, DESIGN 13)
    // -----------------------------------------------------------------------

    /// Roll whether an encounter comes instead of the Tresen and open it.
    /// Returns `false` (the Tresen opens) if none comes or the drawn one has
    /// nothing to offer.
    fn open_encounter(&mut self, content: &Content, events: &mut Vec<RunEvent>) -> bool {
        let chance = u64::from(content.run.encounter_chance.permille());
        if chance == 0 || self.encounter_rng.gen_range(0..1000) >= chance {
            return false;
        }
        let pool: Vec<(&EncounterId, u32)> = content
            .encounters
            .values()
            .filter(|e| in_pool(&e.characters, self.character) && e.weight > 0)
            .map(|e| (&e.id, e.weight))
            .collect();
        let total: u64 = pool.iter().map(|(_, w)| u64::from(*w)).sum();
        if total == 0 {
            return false;
        }
        let mut roll = self.encounter_rng.gen_range(0..total);
        let mut chosen = None;
        for (id, weight) in pool {
            if roll < u64::from(weight) {
                chosen = Some(id.clone());
                break;
            }
            roll -= u64::from(weight);
        }
        let Some(def) = chosen.and_then(|id| content.encounter(&id)) else {
            return false;
        };
        let mut options: Vec<EncounterOption> = Vec::new();
        let free = |choice| EncounterOption {
            choice,
            cost: 0,
            card_mod: None,
        };
        match &def.kind {
            EncounterKind::EveningRule { offers } => {
                let pool = self.relic_pool(content, true);
                let weights = &content.run.shop.rarity_weights;
                let drawn = draw_weighted(&mut self.encounter_rng, pool, *offers, |r| {
                    weights.weight(r)
                });
                options.extend(
                    drawn
                        .into_iter()
                        .map(|id| free(EncounterChoice::TakeRule(id))),
                );
            }
            EncounterKind::MarkCard { price, card_mod } => {
                // The modification is known before the choice: named by the
                // encounter or drawn now from the character's pool.
                let card_mod = match card_mod {
                    Some(id) => Some(id.clone()),
                    None => {
                        let pool = self.card_mod_pool(content, &[]);
                        let pick = self
                            .encounter_rng
                            .gen_range(0..u64::try_from(pool.len()).unwrap_or(0));
                        pool.get(usize::try_from(pick).unwrap_or(usize::MAX))
                            .cloned()
                    }
                };
                if let Some(card_mod) = card_mod {
                    options.push(EncounterOption {
                        choice: EncounterChoice::MarkRandomCard,
                        cost: 0,
                        card_mod: Some(card_mod.clone()),
                    });
                    options.push(EncounterOption {
                        choice: EncounterChoice::MarkChosenCard,
                        cost: *price,
                        card_mod: Some(card_mod),
                    });
                }
            }
            EncounterKind::SwapRelic => {
                // The relic received is drawn now, so every option names it.
                let weights = &content.run.shop.rarity_weights;
                let pool = self.relic_pool(content, false);
                let new =
                    draw_weighted(&mut self.encounter_rng, pool, 1, |r| weights.weight(r)).pop();
                if let Some(new) = new {
                    options.extend(self.relics.iter().enumerate().map(|(index, relic)| {
                        free(EncounterChoice::SwapRelic {
                            index,
                            relic: relic.clone(),
                            new: Some(new.clone()),
                        })
                    }));
                }
            }
            EncounterKind::DoubleNextGame { factor } => {
                options.push(free(EncounterChoice::DoubleNextGame(*factor)));
            }
        }
        if options.is_empty() {
            return false;
        }
        options.push(free(EncounterChoice::Leave));
        self.phase = RunPhase::Encounter;
        events.push(RunEvent::EncounterOpened {
            encounter: def.id.clone(),
            options: options.clone(),
        });
        self.encounter = Some(ActiveEncounter {
            id: def.id.clone(),
            options,
        });
        true
    }

    /// The modification an encounter puts on `card`: the one it names, or
    /// a random one from the character's pool.
    fn encounter_mod(
        &mut self,
        content: &Content,
        named: Option<&CardModId>,
        card: u32,
    ) -> Result<CardModId, RunError> {
        let deck_card = self.deck.get(card).ok_or(DeckError::UnknownCard(card))?;
        if deck_card.kind == CardKind::Blank {
            return Err(DeckError::BlankCannotBeModified.into());
        }
        if let Some(id) = named {
            return Ok(id.clone());
        }
        let options = self.card_mod_pool(content, &deck_card.mods);
        let pick = self
            .encounter_rng
            .gen_range(0..u64::try_from(options.len()).unwrap_or(0));
        options
            .get(usize::try_from(pick).unwrap_or(usize::MAX))
            .cloned()
            .ok_or(RunError::NoModAvailable)
    }

    fn choose_encounter(
        &mut self,
        content: &Content,
        option: usize,
        target: Option<u32>,
        events: &mut Vec<RunEvent>,
    ) -> Result<(), RunError> {
        if self.phase != RunPhase::Encounter {
            return Err(RunError::WrongPhase);
        }
        let encounter = self.encounter.clone().ok_or(RunError::WrongPhase)?;
        let def = content
            .encounter(&encounter.id)
            .ok_or_else(|| RunError::UnknownContent(encounter.id.to_string()))?;
        let chosen = encounter
            .options
            .get(option)
            .cloned()
            .ok_or(RunError::InvalidSlot)?;
        if chosen.cost > self.coins {
            return Err(RunError::NotEnoughCoins);
        }
        let named = match &def.kind {
            EncounterKind::MarkCard { card_mod, .. } => {
                chosen.card_mod.as_ref().or(card_mod.as_ref())
            }
            _ => None,
        };
        match &chosen.choice {
            EncounterChoice::TakeRule(id) => {
                self.evening_relics.push(id.clone());
                events.push(RunEvent::EveningRuleAdded(id.clone()));
            }
            EncounterChoice::MarkRandomCard => {
                let cards: Vec<u32> = self
                    .deck
                    .cards()
                    .iter()
                    .filter(|c| c.kind != CardKind::Blank)
                    .filter(|c| named.is_none_or(|m| !c.mods.contains(m)))
                    .filter(|c| named.is_some() || !self.card_mod_pool(content, &c.mods).is_empty())
                    .map(|c| c.id)
                    .collect();
                let pick = self
                    .encounter_rng
                    .gen_range(0..u64::try_from(cards.len()).unwrap_or(0));
                let card = *cards
                    .get(usize::try_from(pick).unwrap_or(usize::MAX))
                    .ok_or(RunError::NoModAvailable)?;
                let card_mod = self.encounter_mod(content, named, card)?;
                self.attach_mod(content, card, card_mod, events)?;
            }
            EncounterChoice::MarkChosenCard => {
                let card = target.ok_or(RunError::TargetRequired)?;
                let card_mod = self.encounter_mod(content, named, card)?;
                self.attach_mod(content, card, card_mod, events)?;
            }
            EncounterChoice::SwapRelic { index, relic, new } => {
                if self.relics.get(*index) != Some(relic) {
                    return Err(RunError::InvalidSlot);
                }
                let new = match new {
                    Some(new) => new.clone(),
                    None => {
                        let weights = &content.run.shop.rarity_weights;
                        let pool = self.relic_pool(content, false);
                        draw_weighted(&mut self.encounter_rng, pool, 1, |r| weights.weight(r))
                            .pop()
                            .ok_or(RunError::NoModAvailable)?
                    }
                };
                self.relics[*index] = new.clone();
                events.push(RunEvent::RelicSwapped {
                    old: relic.clone(),
                    new,
                });
            }
            EncounterChoice::DoubleNextGame(factor) => {
                self.pending_factor = Some((encounter.id.clone(), *factor));
                events.push(RunEvent::NextGameDoubled { factor: *factor });
            }
            EncounterChoice::Leave => {}
        }
        self.coins -= chosen.cost;
        events.push(RunEvent::EncounterResolved {
            encounter: encounter.id.clone(),
            choice: chosen.choice,
        });
        self.encounter = None;
        let (section, index) = self.next_series_position();
        self.start_series(content, section, index, events)
    }

    /// Section and index of the series after the current one.
    fn next_series_position(&self) -> (u32, u32) {
        if self.series.index + 1 >= SERIES_PER_SECTION {
            (self.series.section + 1, 0)
        } else {
            (self.series.section, self.series.index + 1)
        }
    }

    /// Whether the declarer must declare a hand game in the current game.
    #[must_use]
    pub fn requires_hand(&self) -> bool {
        self.game.as_ref().is_some_and(|a| a.rules.require_hand)
    }

    /// Whether Kontra waits for the declarer's answer.
    #[must_use]
    pub fn awaiting_kontra_answer(&self) -> bool {
        self.game.as_ref().is_some_and(|a| a.awaiting_re)
    }

    /// Redeals ("Schieben") left in the current series.
    #[must_use]
    pub fn redeals_left(&self) -> u32 {
        self.game.as_ref().map_or(0, |a| {
            a.rules
                .series_redeals
                .saturating_sub(self.series.redeals_used)
        })
    }
}

/// Zielwert of a series: the character's value (from `characters.ron`, or
/// `run.ron` if the character has none), for a boss series times the
/// boss's target factor. `None` if the section does not exist.
#[must_use]
pub fn series_target(
    content: &Content,
    character: Character,
    section: u32,
    index: u32,
    boss: Option<&BossId>,
) -> Option<u64> {
    let curve = content
        .character(character)
        .and_then(|c| c.targets.as_ref())
        .unwrap_or(&content.run.targets);
    let targets = curve.get(usize::try_from(section).unwrap_or(usize::MAX))?;
    let base = match index {
        0 => targets.0,
        1 => targets.1,
        _ => targets.2,
    };
    let factor = boss
        .and_then(|b| content.boss(b))
        .map_or(crate::content::Factor::ONE, |def| def.target_factor);
    Some(factor.apply(base))
}

const fn kontra_line(source: Source) -> ScoreLine {
    ScoreLine {
        source,
        lever: Lever::GameValue,
        change: Change::Multiply(2),
    }
}

/// Augen changes from relics and card modifications on the cards that
/// count for the declarer (skat, won tricks, conceded hands), one line per
/// source.
fn declarer_point_lines(active: &ActiveGame) -> Vec<ScoreLine> {
    let game = &active.game;
    let mut ids: Vec<u32> = game.skat().iter().map(|c| c.id).collect();
    for trick in game.tricks().iter().filter(|t| t.winner.is_declarer()) {
        ids.extend(trick.cards.iter().map(|c| c.id));
    }
    if game.conceded() {
        for seat in Seat::ALL {
            ids.extend(game.hand(seat).iter().map(|c| c.id));
        }
    }
    let mut per_source: BTreeMap<Source, i64> = BTreeMap::new();
    for (card, source, delta) in &active.point_changes {
        if ids.contains(card) {
            *per_source.entry(source.clone()).or_insert(0) += i64::from(*delta);
        }
    }
    per_source
        .into_iter()
        .filter(|(_, delta)| *delta != 0)
        .map(|(source, delta)| ScoreLine {
            source,
            lever: Lever::CardPoints,
            change: Change::Add(delta),
        })
        .collect()
}

/// Draw up to `count` distinct items, weighted by rarity, in a fixed pool
/// order.
fn draw_weighted<T: Clone>(
    rng: &mut Rng,
    mut pool: Vec<(T, Rarity)>,
    count: u32,
    weight: impl Fn(Rarity) -> u32,
) -> Vec<T> {
    let mut drawn = Vec::new();
    for _ in 0..count {
        let total: u64 = pool.iter().map(|(_, r)| u64::from(weight(*r))).sum();
        if total == 0 {
            break;
        }
        let mut roll = rng.gen_range(0..total);
        let mut chosen = None;
        for (i, (_, rarity)) in pool.iter().enumerate() {
            let w = u64::from(weight(*rarity));
            if roll < w {
                chosen = Some(i);
                break;
            }
            roll -= w;
        }
        let Some(i) = chosen else {
            break;
        };
        drawn.push(pool.remove(i).0);
    }
    drawn
}

#[cfg(test)]
mod content_tests;
#[cfg(test)]
mod m4_tests;
#[cfg(test)]
mod m5_tests;
#[cfg(test)]
mod tests;

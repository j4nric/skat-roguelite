//! Run state machine (design document 5, 6 and 12).
//!
//! A run is a tour through sections (Skatabende). Each section has three
//! series: two normal ones and a boss series. A series has a target
//! (Zielwert) and a game limit. Before declaring, the player may have each
//! game dealt again a few times (einmischen, [`RunAction::Redeal`]) without
//! using up a game; then one of the hands must be played. Won games add
//! their value to the series account, lost games subtract twice their value
//! including all relic modifiers. A series is won the moment the account
//! reaches the target; the run is lost when the game limit is reached
//! below the target. Between series the Tresen (shop) opens.
//!
//! The run is driven by [`RunAction`]s and answers with [`RunEvent`]s. The
//! two defenders are played by the AI inside the run (heuristic or PIMC by
//! section, see [`opponents`]). [`Run::apply`] plays them until the next
//! decision of the player (simulator, bot, tests). A frontend that wants to
//! show each defender's move on its own drives the run step by step:
//! [`Run::apply_step`] only carries out the player's action, then while
//! [`Run::ai_to_move`] names a seat, [`Run::advance_ai`] plays exactly one
//! card of the AI. Both ways give the same events in the same order. The
//! whole state is serializable (serde), so a frontend can save and load a
//! run at any point, also while the AI is to move; equal seed and equal
//! actions give equal results.
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
pub mod guests;
pub mod meta;
pub mod opponents;
pub mod player;
pub mod view;

mod settle;

mod value_preview;

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use serde::{Deserialize, Serialize};

use crate::ai::DefenderHeuristic;
use crate::cards::{Card, PlayingCard, Rank, Suit};
use crate::content::{
    ActiveSource, BossId, CardFilter, CardModId, Change, Character, ConsumableAction, ConsumableId,
    Content, Effect, EncounterId, EncounterKind, GameFocus, HookContext, Lever, NextGameEffect,
    RelicId, RelicKind, RunLength, ScoreBreakdown, ScoreLine, Source, Stage, card_mod_sources,
    evaluate, in_pool, resolve_deck, score_parts,
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
    /// Games played (until 01.11.2026 also games passed).
    pub games_played: u32,
    /// Spiellimit. A save from before the Einmischen rule keeps the limit
    /// of its series (8) until that series ends.
    pub game_limit: u32,
    /// Boss of a boss series.
    pub boss: Option<BossId>,
    /// Einmischen in this series, every redeal of a game counted (in saves
    /// from before: the redeals of "Schieben"). Drives the rule rotation of
    /// "Der Alte" and "Boss ohne Einmischen".
    #[serde(default)]
    pub redeals_used: u32,
    /// Einmischen of the current game's own allowance used so far
    /// ([`Run::redeals_left`]); back to 0 with every new game.
    #[serde(default)]
    pub redeals_in_game: u32,
    /// Einmischen left for the rest of the series from consumables ("Ein
    /// Bier", "Kräuter"), used once the game's own are gone.
    #[serde(default)]
    pub extra_redeals: u32,
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
    /// Einmischen the content adds to every game ("Schieben", one per
    /// [`Effect::ExtraRedeal`]), on top of `run.ron` and the character
    /// ([`Content::redeals`]). Also read under its old name
    /// `series_redeals`.
    #[serde(alias = "series_redeals")]
    pub extra_redeals: u32,
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
    /// Source of the Schneider bonus in the thresholds ("Stichfest").
    pub schneider_bonus_source: Option<Source>,
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
    /// The player looked at the untouched skat ("Kurzer").
    #[serde(default)]
    pub skat_peeked: bool,
    /// Cards repainted for this game ("Kreide"): instance id and printed
    /// suit.
    #[serde(default)]
    pub chalked: Vec<(u32, Suit)>,
    /// Lost, this game counts once instead of twice ("Magenbitter"): who
    /// granted it.
    #[serde(default)]
    pub single_loss: Option<Source>,
    /// The defender whose whole hand the player sees ("Der Zaungast"):
    /// his cards are in `peeked` with this seat.
    #[serde(default)]
    pub shown_hand: Option<Seat>,
    /// The deal before the last Einmischen of this game, kept for "Reue"
    /// ([`ConsumableAction::UndoRedeal`]); its own `previous` is always
    /// empty. `None` before the first Einmischen and after an undo.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub previous: Option<Box<ActiveGame>>,
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
    /// Extra from the character's rule ("Die Wirtin": the three parts
    /// once more); 0 for most characters.
    pub bonus: u32,
}

impl Payout {
    /// Sum of all parts.
    #[must_use]
    pub const fn total(&self) -> u32 {
        self.base + self.games_left + self.interest + self.bonus
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
    bonus: u32,
    #[serde(default)]
    total: u32,
}

impl From<Payout> for PayoutRepr {
    fn from(p: Payout) -> PayoutRepr {
        PayoutRepr {
            base: p.base,
            games_left: p.games_left,
            interest: p.interest,
            bonus: p.bonus,
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
            bonus: p.bonus,
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
    /// Einmischen (games dealt again before the declaration). Also read
    /// under its old name `games_passed` (games passed until 01.11.2026).
    #[serde(alias = "games_passed")]
    pub redeals: u32,
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
    /// Einmischen: throw the dealt game in and have all hands and the skat
    /// dealt again before declaring, without using up a game. Each game
    /// allows [`Content::redeals`] of them (plus "Schieben"), then the
    /// series' extras ("Ein Bier"); see [`Run::redeals_left`]. Also read
    /// under the old names `RedrawSkat` and `Pass` (Einpassen, which cost a
    /// game until 01.11.2026).
    #[serde(alias = "RedrawSkat", alias = "Pass")]
    Redeal,
    /// Pick up the skat instead of declaring a hand game (Skatordnung, ADR
    /// 0011). Then two cards are discarded ([`RunAction::Discard`]) and the
    /// game is declared ([`RunAction::Declare`]). Einmischen is no longer
    /// possible after it.
    PickUpSkat,
    /// Announce the game. With ten cards, before the skat is touched, only a
    /// hand game (Hand, Schneider or Schwarz angesagt, Ouvert; Null Hand,
    /// Null Ouvert Hand). After picking up the skat and discarding, only a
    /// game without the Hand level: `Normal`, Schneider or Schwarz angesagt,
    /// Ouvert (ADR 0013); Null or Null Ouvert. The phase decides whether an
    /// announcement is played from the hand ([`Declaration::with_hand`]).
    Declare(Declaration),
    /// Answer a Kontra: `true` = Re.
    AnswerKontra {
        /// Whether to say Re.
        re: bool,
    },
    /// Discard two cards (instance ids) after picking up the skat; the
    /// declaration follows.
    Discard([u32; 2]),
    /// Play a card (instance id).
    Play(u32),
    /// Use a consumable from the given slot, with a target card and a suit
    /// if needed.
    UseConsumable {
        /// Slot of the consumable.
        slot: usize,
        /// Target card instance id: a deck card for deck changes, a card of
        /// the hand for "Kreide".
        target: Option<u32>,
        /// The new suit of the target ("Kreide").
        #[serde(default)]
        suit: Option<Suit>,
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
    /// "Rest abkürzen": end the game in progress when the remaining cards
    /// can no longer change its score ([`view::GameView::can_settle`],
    /// [`Run::can_settle`]). The remaining cards are played out by a fixed
    /// rule and the game is scored as if played. Fails with
    /// [`RunError::NotDecided`] otherwise.
    SettleGame,
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
        /// Einmischen per game of the character, without relics. Also read
        /// under its old name `passes` (passes per series).
        #[serde(alias = "passes")]
        redeals: u32,
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
    /// A counter went back to zero ("Altes Blatt" after a won series).
    CounterReset {
        /// Counter name.
        counter: String,
    },
    /// Content made the discarded skat count more Augen for the declarer
    /// ("Skatpfleger").
    SkatPointsRaised {
        /// Who did it.
        source: Source,
        /// Augen the declarer counts more (fewer if negative).
        card_points: i32,
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
        /// Einmischen in the series. Also read under its old name
        /// `passes_used`.
        #[serde(default, alias = "passes_used")]
        redeals_used: u32,
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
    /// Einmischen were added to the series ("Ein Bier", "Kräuter"). Also
    /// read under its old name `PassesAdded`.
    #[serde(alias = "PassesAdded")]
    RedealsAdded {
        /// Einmischen added.
        #[serde(alias = "passes")]
        redeals: u32,
        /// Einmischen left now ([`Run::redeals_left`]).
        #[serde(alias = "passes_left")]
        redeals_left: u32,
    },
    /// The game was thrown in and dealt again: Einmischen, or the
    /// consumable "Neues Blatt".
    Redealt,
    /// "Reue": the last Einmischen was taken back; the deal before it is
    /// on the table again.
    RedealUndone,
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
    /// The player looked at the untouched skat ("Kurzer").
    SkatPeeked {
        /// The two skat cards.
        skat: [DealtCard; 2],
    },
    /// The run was won.
    RunWon {
        /// Who won it.
        #[serde(default)]
        character: Character,
        /// Ganzer or kurzer Abend.
        #[serde(default)]
        length: RunLength,
        /// Stake of the run (Einsatz-Leiter).
        #[serde(default)]
        stake: u32,
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
        /// Ganzer or kurzer Abend.
        #[serde(default)]
        length: RunLength,
        /// Stake of the run (Einsatz-Leiter).
        #[serde(default)]
        stake: u32,
    },
}

/// Why an action was rejected. The run is unchanged.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RunError {
    /// The action does not fit the current phase.
    WrongPhase,
    /// The game rejected the action.
    Game(GameError<DealtCard>),
    /// No passes left in this series. Never returned since Einmischen
    /// replaced passing (01.11.2026); kept so that the error codes stay
    /// stable.
    NoPassesLeft,
    /// The boss demands a hand game.
    HandRequired,
    /// Null is not available. Never returned since ADR 0012 (every
    /// character may declare Null); kept so that the error codes stay
    /// stable.
    NullNotAvailable,
    /// No Einmischen left in this game.
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
    /// The Ramsch of the series cannot be dealt again.
    RamschRequired,
    /// A defender is to move: advance the AI ([`Run::advance_ai`]) before
    /// the player acts again.
    AiToMove,
    /// "Kreide" cannot repaint this card into this suit: a jack, a blank,
    /// or the suit it already has.
    CannotChalk,
    /// The rest of the game can still change its score
    /// ([`RunAction::SettleGame`]).
    NotDecided,
    /// "Reue": there is no Einmischen of this game to take back.
    NoRedealToUndo,
}

/// Stable code of a [`RunError`], for frontends: serialized as its name
/// (`"NotEnoughCoins"`), never renamed once released. A frontend maps the
/// code to its own text; the English message of [`RunError`]'s `Display`
/// is for logs only.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum RunErrorCode {
    /// The action does not fit the current phase of the run.
    WrongPhase,
    /// No passes left (never since Einmischen replaced passing, kept for
    /// stable codes).
    NoPassesLeft,
    /// The boss demands a hand game.
    HandRequired,
    /// Null is not available (never since ADR 0012, kept for stable codes).
    NullNotAvailable,
    /// No Einmischen left in this game.
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
    /// The declaration does not fit the skat: before it is picked up only
    /// hand games, after it only games without hand levels.
    InvalidDeclaration,
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
    /// The Ramsch cannot be dealt again.
    RamschRequired,
    /// The card must not be discarded ("Ramponiert").
    DiscardForbidden,
    /// A defender is to move; the player has to wait.
    AiToMove,
    /// "Kreide" cannot repaint this card into this suit.
    CannotChalk,
    /// The rest of the game can still change its score; it cannot be
    /// settled ([`RunAction::SettleGame`]).
    NotDecided,
    /// "Reue": no Einmischen of this game to take back.
    NoRedealToUndo,
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
            RunErrorCode::InvalidDeclaration => "InvalidDeclaration",
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
            RunErrorCode::AiToMove => "AiToMove",
            RunErrorCode::CannotChalk => "CannotChalk",
            RunErrorCode::NotDecided => "NotDecided",
            RunErrorCode::NoRedealToUndo => "NoRedealToUndo",
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
                GameError::InvalidDeclaration { .. } => RunErrorCode::InvalidDeclaration,
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
            RunError::AiToMove => RunErrorCode::AiToMove,
            RunError::CannotChalk => RunErrorCode::CannotChalk,
            RunError::NotDecided => RunErrorCode::NotDecided,
            RunError::NoRedealToUndo => RunErrorCode::NoRedealToUndo,
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
            RunError::NoRedealLeft => f.write_str("no Einmischen left in this game"),
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
            RunError::AiToMove => f.write_str("a defender is to move"),
            RunError::CannotChalk => f.write_str("this card cannot be repainted into this suit"),
            RunError::NotDecided => f.write_str("the rest of the game is not decided"),
            RunError::NoRedealToUndo => f.write_str("no Einmischen to take back"),
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
    /// Ganzer or kurzer Abend (DESIGN 5.3).
    #[serde(default)]
    pub length: RunLength,
    /// Einsatz-Leiter (DESIGN 9.3): suit games and Grand are won from
    /// `61 + stake` Augen. 0 is the normal game.
    #[serde(default)]
    pub stake: u32,
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
    /// Consumables used for the next declared game ("Doppelkorn",
    /// "Magenbitter"), in the order used.
    #[serde(default)]
    pending_next_game: Vec<(ConsumableId, NextGameEffect)>,
    /// Stammgäste of the current series, seat 1 first ([`guests`]). Empty
    /// in saves from before; then they are derived from the seed.
    #[serde(default)]
    guests: Vec<crate::content::GuestId>,
    /// Ganzer or kurzer Abend (saves from before: ganzer).
    #[serde(default)]
    length: RunLength,
    /// Stake of the Einsatz-Leiter (saves from before: 0).
    #[serde(default)]
    stake: u32,
}

impl Run {
    /// Start a run: the first series begins and its first game is dealt.
    /// Should the defenders move first (a Ramsch from a start relic), they
    /// have played when this returns, as after [`Run::apply`].
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
        let sections = content.run.sections(start.length);
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
        if relics.len() > usize::try_from(content.relic_slots(start.character)).unwrap_or(0) {
            return Err(RunError::NoFreeSlot);
        }
        for consumable in &character.start_consumables {
            if content.consumable(consumable).is_none() {
                return Err(RunError::UnknownContent(consumable.to_string()));
            }
        }
        if character.start_consumables.len()
            > usize::try_from(content.run.consumable_slots).unwrap_or(0)
        {
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
                boss: None,
                redeals_used: 0,
                redeals_in_game: 0,
                extra_redeals: 0,
                skat_swaps_used: 0,
                skat_swaps_taken: Vec::new(),
                ramsch_played: false,
            },
            coins: content.run.economy.starting_coins,
            deck,
            relics,
            consumables: character.start_consumables.clone(),
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
            pending_next_game: Vec::new(),
            guests: Vec::new(),
            length: start.length,
            stake: start.stake,
        };
        let mut events = Vec::new();
        run.start_series(content, start.section, start.index, &mut events)?;
        run.play_ai(content, &mut events)?;
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

    /// Ganzer or kurzer Abend.
    #[must_use]
    pub const fn length(&self) -> RunLength {
        self.length
    }

    /// Stake of the Einsatz-Leiter (DESIGN 9.3): suit games and Grand are
    /// won from `61 + stake` Augen before relics.
    #[must_use]
    pub const fn stake(&self) -> u32 {
        self.stake
    }

    /// Number of sections of this run (5, or 3 for a short evening).
    #[must_use]
    pub fn sections(&self, content: &Content) -> u32 {
        content.run.sections(self.length)
    }

    /// The row of the curve (Zielwerte, opponents) the current section
    /// plays: the section itself, or for a short evening the row
    /// `run.ron` names for it.
    #[must_use]
    pub fn curve_row(&self, content: &Content) -> u32 {
        content
            .run
            .curve_row(self.length, self.series.section)
            .unwrap_or(self.series.section)
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

    /// Consumables waiting for the next declared game ("Doppelkorn",
    /// "Magenbitter"), in the order used.
    #[must_use]
    pub fn pending_next_game(&self) -> &[(ConsumableId, NextGameEffect)] {
        &self.pending_next_game
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

    /// Apply a player decision and let the defenders play until the
    /// player decides again: [`Run::apply_step`] followed by
    /// [`Run::advance_ai`] while [`Run::ai_to_move`] names a seat, with all
    /// events in one list. For the simulator, the bot and tests. On error
    /// nothing changed.
    pub fn apply(
        &mut self,
        content: &Content,
        action: RunAction,
    ) -> Result<Vec<RunEvent>, RunError> {
        let mut events = Vec::new();
        // Work on a copy so that a failing action leaves the run untouched.
        let mut next = self.clone();
        next.dispatch(content, action, &mut events)?;
        next.play_ai(content, &mut events)?;
        *self = next;
        Ok(events)
    }

    /// Apply a player decision without letting the defenders play: a
    /// finished game is scored and the next one dealt, but if a defender
    /// is to move afterwards the run waits in that state
    /// ([`Run::ai_to_move`]) for [`Run::advance_ai`]. While a defender is
    /// to move every action fails with [`RunError::AiToMove`]. On error
    /// nothing changed.
    pub fn apply_step(
        &mut self,
        content: &Content,
        action: RunAction,
    ) -> Result<Vec<RunEvent>, RunError> {
        let mut events = Vec::new();
        let mut next = self.clone();
        next.dispatch(content, action, &mut events)?;
        *self = next;
        Ok(events)
    }

    /// The defender who is to play the next card, if the run waits for
    /// the AI rather than for the player: in a game being played, no Kontra
    /// waiting for an answer, a defender's turn.
    #[must_use]
    pub fn ai_to_move(&self) -> Option<Seat> {
        if self.phase != RunPhase::Game {
            return None;
        }
        let active = self.game.as_ref()?;
        if active.awaiting_re || active.game.phase() != Phase::Playing {
            return None;
        }
        active
            .game
            .current_seat()
            .filter(|seat| !seat.is_declarer())
    }

    /// Let the AI play exactly one card of the defender who is to move
    /// ([`Run::ai_to_move`]). A game finished by it is scored and the next
    /// one dealt, as after a card of the player. Fails with
    /// [`RunError::WrongPhase`] if no defender is to move; then nothing
    /// changed.
    pub fn advance_ai(&mut self, content: &Content) -> Result<Vec<RunEvent>, RunError> {
        let mut events = Vec::new();
        let mut next = self.clone();
        next.ai_move(content, &mut events)?;
        *self = next;
        Ok(events)
    }

    /// Let the AI play until the player decides again (nothing if no
    /// defender is to move), e.g. after loading a save made while the AI
    /// was to move. On error nothing changed.
    pub fn advance_ai_all(&mut self, content: &Content) -> Result<Vec<RunEvent>, RunError> {
        let mut events = Vec::new();
        let mut next = self.clone();
        next.play_ai(content, &mut events)?;
        *self = next;
        Ok(events)
    }

    /// One card of the defender who is to move, then settle the game.
    fn ai_move(&mut self, content: &Content, events: &mut Vec<RunEvent>) -> Result<(), RunError> {
        let seat = self.ai_to_move().ok_or(RunError::WrongPhase)?;
        let active = self.game.as_ref().ok_or(RunError::WrongPhase)?;
        let view = active.game.view(seat).ok_or(RunError::WrongPhase)?;
        let card =
            opponents::defender_card(self, content, active, &view).ok_or(RunError::WrongPhase)?;
        self.play_card(content, seat, card, events)?;
        self.settle(content, events)
    }

    /// Let the defenders play until the player decides again.
    fn play_ai(&mut self, content: &Content, events: &mut Vec<RunEvent>) -> Result<(), RunError> {
        while self.ai_to_move().is_some() {
            self.ai_move(content, events)?;
        }
        Ok(())
    }

    fn dispatch(
        &mut self,
        content: &Content,
        action: RunAction,
        events: &mut Vec<RunEvent>,
    ) -> Result<(), RunError> {
        if self.ai_to_move().is_some() {
            return Err(RunError::AiToMove);
        }
        match action {
            RunAction::Redeal => self.redeal_action(content, events),
            RunAction::PickUpSkat => self.pick_up_skat(events),
            RunAction::Declare(declaration) => self.declare(content, declaration, events),
            RunAction::AnswerKontra { re } => self.answer_kontra(content, re, events),
            RunAction::Discard(ids) => self.discard(content, ids, events),
            RunAction::Play(id) => self.play(content, id, events),
            RunAction::UseConsumable { slot, target, suit } => {
                self.use_consumable(content, slot, target, suit, events)
            }
            RunAction::BuyRelic(slot) => self.buy_relic(content, slot, events),
            RunAction::BuyConsumable(slot) => self.buy_consumable(content, slot, events),
            RunAction::SellRelic(index) => self.sell_relic(content, index, events),
            RunAction::SellConsumable(index) => self.sell_consumable(content, index, events),
            RunAction::Reroll => self.reroll(content, events),
            RunAction::LeaveShop => self.leave_shop(content, events),
            RunAction::TakeFromSkat { give, take } => self.take_from_skat(give, take, events),
            RunAction::SettleGame => self.settle_game(content, events),
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

    /// Einmischen ([`RunAction::Redeal`]): one of the game's own first,
    /// then one of the series' extras ("Ein Bier").
    fn redeal_action(
        &mut self,
        content: &Content,
        events: &mut Vec<RunEvent>,
    ) -> Result<(), RunError> {
        self.declaring()?;
        let own_left = self
            .redeal_allowance(content)
            .saturating_sub(self.series.redeals_in_game);
        if own_left > 0 {
            self.series.redeals_in_game += 1;
        } else if self.series.extra_redeals > 0 {
            self.series.extra_redeals -= 1;
        } else {
            return Err(RunError::NoRedealLeft);
        }
        self.redeal(content, events)
    }

    /// Throw the dealt game in and deal it again before the declaration,
    /// without using up a game (Einmischen, "Neues Blatt"). The forehand
    /// stays. The deal thrown in is kept for "Reue"; every redeal counts
    /// for the series (rule rotation of "Der Alte").
    fn redeal(&mut self, content: &Content, events: &mut Vec<RunEvent>) -> Result<(), RunError> {
        self.declaring()?;
        let previous = self.game.take().map(|mut active| {
            active.previous = None;
            Box::new(active)
        });
        self.series.redeals_used += 1;
        self.stats.redeals += 1;
        events.push(RunEvent::Redealt);
        self.deal(content, false, events)?;
        if let Some(active) = self.game.as_mut() {
            active.previous = previous;
        }
        Ok(())
    }

    /// "Reue": the deal before the last Einmischen comes back as it was
    /// (hands, skat, what the player was shown). The Einmischen stays used.
    fn undo_redeal(&mut self, events: &mut Vec<RunEvent>) -> Result<(), RunError> {
        let active = self.declaring()?;
        let previous = active.previous.take().ok_or(RunError::NoRedealToUndo)?;
        self.game = Some(*previous);
        events.push(RunEvent::RedealUndone);
        Ok(())
    }

    /// Pick up the skat (ADR 0011). Not if the boss demands a hand game.
    /// "Der Angeber" allows it: Null Ouvert can be played open after the
    /// skat (ADR 0012, every character may declare Null).
    fn pick_up_skat(&mut self, events: &mut Vec<RunEvent>) -> Result<(), RunError> {
        let active = self.declaring()?;
        if active.rules.require_hand {
            return Err(RunError::HandRequired);
        }
        let game_events = active.game.pick_up_skat()?;
        events.extend(game_events.into_iter().map(RunEvent::Game));
        Ok(())
    }

    fn declare(
        &mut self,
        content: &Content,
        declaration: Declaration,
        events: &mut Vec<RunEvent>,
    ) -> Result<(), RunError> {
        let active = self.active_mut()?;
        if active.rules.ramsch {
            return Err(RunError::RamschRequired);
        }
        if !matches!(
            active.game.phase(),
            Phase::Declaring | Phase::DeclaringAfterSkat
        ) {
            return Err(RunError::WrongPhase);
        }
        // Every character may declare every game (ADR 0012); the Ramsch
        // only comes from "Der Ramscher".
        if declaration == Declaration::Ramsch {
            return Err(RunError::GameNotAvailable);
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
        self.settle(content, events)
    }

    /// After any declaration (also the Ramsch): fire `OnDeclare` and use
    /// up the factor of "Letzte Runde" and the consumables waiting for the
    /// next game ("Doppelkorn", "Magenbitter").
    fn after_declaration(&mut self, content: &Content, events: &mut Vec<RunEvent>) {
        self.fire(content, Stage::OnDeclare, None, None, events);
        let pending = std::mem::take(&mut self.pending_next_game);
        let Some(active) = self.game.as_mut() else {
            return;
        };
        if let Some((encounter, factor)) = self.pending_factor.take() {
            active.lines.push(ScoreLine {
                source: Source::Encounter(encounter),
                lever: Lever::GameValue,
                change: Change::Multiply(factor),
            });
        }
        for (id, effect) in pending {
            let source = Source::Consumable(id);
            match effect {
                NextGameEffect::Multiply(factor) => active.lines.push(ScoreLine {
                    source,
                    lever: Lever::GameValue,
                    change: Change::Multiply(factor),
                }),
                NextGameEffect::SingleLoss => {
                    active.single_loss.get_or_insert(source);
                }
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
        self.settle(content, events)
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
        self.settle(content, events)
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
        self.settle(content, events)
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

    /// Score a finished game (which deals the next one or ends the
    /// series). The defenders do not play here: whether one is to move is
    /// [`Run::ai_to_move`].
    fn settle(&mut self, content: &Content, events: &mut Vec<RunEvent>) -> Result<(), RunError> {
        match self.game.as_ref().map(|a| a.game.phase()) {
            Some(Phase::Finished(_)) => self.finish_game(content, events),
            _ => Ok(()),
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
        // Every deal counts, also one dealt again (Einmischen), as passes
        // did before.
        let deals = self
            .series
            .games_played
            .saturating_add(self.series.redeals_used);
        let i = usize::try_from(deals % n).unwrap_or(0);
        rules.get(i).map(|id| (*id).clone())
    }

    /// The character's own rule, boss, relics and house rules of the
    /// evening, plus the card modifications in play if a game runs.
    fn sources<'c>(&self, content: &'c Content, with_card_mods: bool) -> Vec<ActiveSource<'c>> {
        let mut sources = Vec::new();
        if let Some(def) = content.character(self.character) {
            let rule = &def.rule;
            if !rule.passive.is_empty() || !rule.hooks.is_empty() {
                sources.push(ActiveSource {
                    source: Source::Character(self.character),
                    passive: &rule.passive,
                    hooks: &rule.hooks,
                    this_card: None,
                });
            }
        }
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
            Effect::ResetCounter(counter) => {
                if self.counters.remove(counter).is_some_and(|v| v > 0) {
                    events.push(RunEvent::CounterReset {
                        counter: counter.clone(),
                    });
                }
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
        let line = score_line(source, effect, &self.counters);
        let ramsch_played = self.series.ramsch_played;
        let Some(active) = self.game.as_mut() else {
            return;
        };
        if effect.is_cheat() && matches!(source, Source::Boss(_)) && active.rules.no_cheating {
            return;
        }
        match effect {
            Effect::AddBaseValue(_)
            | Effect::MultiplyBaseValue(_)
            | Effect::AddBaseValueFromCounter(_)
            | Effect::AddBaseValuePerCounter { .. }
            | Effect::AddLevels(_)
            | Effect::MultiplyGameValue(_)
            | Effect::ScaleGameValue(_) => active.lines.extend(line),
            Effect::MultiplySkatPoints(factor) => {
                let factor = *factor;
                let scaled = |c: DealtCard| DealtCard {
                    card_points: c.card_points.saturating_mul(factor),
                    ..c
                };
                // Nothing in a hand game: no card was discarded.
                if let Ok(old) = active.game.map_skat(scaled) {
                    let mut added: i64 = 0;
                    for card in old {
                        let delta =
                            i64::from(scaled(card).card_points) - i64::from(card.card_points);
                        if delta != 0 {
                            let delta = i32::try_from(delta).unwrap_or(0);
                            active.point_changes.push((card.id, source.clone(), delta));
                            added += i64::from(delta);
                        }
                    }
                    if added != 0 {
                        events.push(RunEvent::SkatPointsRaised {
                            source: source.clone(),
                            card_points: i32::try_from(added).unwrap_or(0),
                        });
                    }
                }
            }
            Effect::SchneiderBonusFromTrick => {
                let points = active.game.tricks().last().map_or(0, |t| t.card_points());
                if points > 0 {
                    let thresholds = &mut active.rules.thresholds;
                    thresholds.schneider_bonus = thresholds.schneider_bonus.saturating_add(points);
                    active
                        .rules
                        .schneider_bonus_source
                        .get_or_insert_with(|| source.clone());
                }
            }
            Effect::WinThreshold(n) => {
                if active.rules.thresholds.lower_win(*n) {
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
            Effect::ExtraRedeal => {
                active.rules.extra_redeals = active.rules.extra_redeals.saturating_add(1);
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
            Effect::RevealDefenderHand(index) => {
                let seat = Seat::from_index(usize::from(*index)).filter(|s| !s.is_declarer());
                if let (Some(seat), false) = (seat, active.rules.no_information) {
                    active.shown_hand = Some(seat);
                    for &card in active.game.hand(seat) {
                        if !active.peeked.iter().any(|(c, _)| c.id == card.id) {
                            active.peeked.push((card, seat));
                            events.push(RunEvent::CardPeeked { card, seat });
                        }
                    }
                }
            }
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
            | Effect::ResetCounter(_)
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
        let sections = config.sections(self.length);
        if section >= sections {
            return Err(RunError::WrongPhase);
        }
        if index == 0 && !self.evening_relics.is_empty() {
            self.evening_relics.clear();
            events.push(RunEvent::EveningRulesEnded);
        }
        if index == 0 || self.section_boss.is_none() {
            let last = section + 1 == sections;
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
        let row = content
            .run
            .curve_row(self.length, section)
            .ok_or(RunError::WrongPhase)?;
        let target = series_target(content, self.character, row, index, boss.as_ref())
            .ok_or(RunError::WrongPhase)?;
        self.series = Series {
            section,
            index,
            target,
            score: 0,
            games_played: 0,
            game_limit: config.game_limit,
            boss: boss.clone(),
            redeals_used: 0,
            redeals_in_game: 0,
            extra_redeals: 0,
            skat_swaps_used: 0,
            skat_swaps_taken: Vec::new(),
            ramsch_played: false,
        };
        self.phase = RunPhase::Game;
        self.shop = None;
        self.encounter = None;
        self.seat_guests(content, section, index);
        events.push(RunEvent::SeriesStarted {
            section,
            index,
            target,
            game_limit: config.game_limit,
            redeals: content.redeals(self.character),
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
            self.series.redeals_in_game = 0;
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
        // Cards the player already sees ("Der Zaungast") are not drawn.
        let shown = active.peeked.clone();
        let mut pool: Vec<(DealtCard, Seat)> = Seat::DEFENDERS
            .iter()
            .flat_map(|&seat| active.game.hand(seat).iter().map(move |&c| (c, seat)))
            .filter(|(c, _)| !shown.iter().any(|(s, _)| s.id == c.id))
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
        self.settle(content, events)
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
            thresholds: Thresholds::with_stake(self.stake),
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
            skat_peeked: false,
            chalked: Vec::new(),
            single_loss: None,
            shown_hand: None,
            previous: None,
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
        if parts.schneider && active.rules.thresholds.schneider_bonus > 0 {
            if let Some(source) = &active.rules.schneider_bonus_source {
                // A line only if the bonus made the difference.
                let without = Thresholds {
                    schneider_bonus: 0,
                    ..active.rules.thresholds.clone()
                };
                if !game_value_parts(declaration, matadors, &outcome, &without).schneider {
                    lines.push(ScoreLine {
                        source: source.clone(),
                        lever: Lever::Schneider,
                        change: Change::Set(1),
                    });
                }
            }
        }
        lines.extend(active.lines.iter().cloned());
        lines.extend(kontra_lines(active.kontra));
        if !parts.won {
            if let Some(source) = &active.single_loss {
                lines.push(ScoreLine {
                    source: source.clone(),
                    lever: Lever::LossFactor,
                    change: Change::Set(1),
                });
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

    /// After a game was scored: series won, run lost or next deal.
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
                length: self.length,
                stake: self.stake,
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
        let mut payout = Payout {
            base: economy.series_reward,
            games_left: games_left * economy.per_game_left,
            interest,
            bonus: 0,
        };
        // "Die Wirtin": the parts count `payout_factor` times.
        payout.bonus = payout
            .total()
            .saturating_mul(content.payout_factor(self.character) - 1);
        self.coins = self.coins.saturating_add(payout.total());
        self.stats.series_won += 1;
        events.push(RunEvent::SeriesWon {
            section: self.series.section,
            index: self.series.index,
            payout,
            coins: self.coins,
            boss: self.series.boss.clone(),
            redeals_used: self.series.redeals_used,
            games_left,
        });
        self.fire(content, Stage::OnSeriesWon, None, None, events);
        let last_section = self.sections(content).saturating_sub(1);
        if self.series.index + 1 == SERIES_PER_SECTION && self.series.section >= last_section {
            self.phase = RunPhase::Won;
            events.push(RunEvent::RunWon {
                character: self.character,
                length: self.length,
                stake: self.stake,
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

    /// Relics that may be offered or drawn, with their draw weight: for the
    /// character, unlocked, neither owned nor on the table for the evening;
    /// only house rules if `house_rules_only`. In id order. The weight is
    /// the rarity weight of `run.ron` times the character's weight for the
    /// relic's Spielart ([`Run::focus_weight`]).
    fn relic_pool(&self, content: &Content, house_rules_only: bool) -> Vec<(RelicId, u32)> {
        let weights = &content.run.shop.rarity_weights;
        content
            .relics
            .values()
            .filter(|r| in_pool(&r.characters, self.character))
            .filter(|r| !self.locked.contains(&r.id))
            .filter(|r| !self.relics.contains(&r.id) && !self.evening_relics.contains(&r.id))
            .filter(|r| !house_rules_only || r.kind == RelicKind::HouseRule)
            .map(|r| {
                let weight = weights
                    .weight(r.rarity)
                    .saturating_mul(self.focus_weight(content, r.focus));
                (r.id.clone(), weight)
            })
            .collect()
    }

    /// The character's Tresen weight for items made for this Spielart, in
    /// percent (ADR 0012: the Nullspieler meets Null relics more often).
    fn focus_weight(&self, content: &Content, focus: GameFocus) -> u32 {
        content
            .character(self.character)
            .map_or(100, |c| c.focus_weights.percent(focus))
    }

    /// Draw a deterministic offer from the content pool. Relics the player
    /// owns, locked relics and items for other characters are not offered;
    /// no item twice in one offer. Rarer items are drawn less often
    /// (weights in `run.ron`).
    fn draw_offer(&mut self, content: &Content, rerolls: u32) -> Shop {
        let weights = &content.run.shop.rarity_weights;
        let relic_pool = self.relic_pool(content, false);
        let consumable_pool: Vec<(ConsumableId, u32)> = content
            .consumables
            .values()
            .filter(|c| in_pool(&c.characters, self.character))
            .map(|c| (c.id.clone(), weights.weight(c.rarity)))
            .collect();
        let relics = draw_weighted(
            &mut self.shop_rng,
            relic_pool,
            content.run.shop.relic_offers,
        );
        let consumables = draw_weighted(
            &mut self.shop_rng,
            consumable_pool,
            content.run.shop.consumable_offers,
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
        if owned >= usize::try_from(content.relic_slots(self.character)).unwrap_or(0) {
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
        suit: Option<Suit>,
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
                        draw_weighted(&mut self.misc_rng, options, 1)
                            .pop()
                            .ok_or(RunError::NoModAvailable)?
                    }
                };
                self.attach_mod(content, target, card_mod, events)?;
            }
            ConsumableAction::ExtraRedeals(n) => {
                if self.phase != RunPhase::Game {
                    return Err(RunError::WrongPhase);
                }
                self.series.extra_redeals = self.series.extra_redeals.saturating_add(*n);
                events.push(RunEvent::RedealsAdded {
                    redeals: *n,
                    redeals_left: self.redeals_left(content),
                });
            }
            ConsumableAction::Redeal => self.redeal(content, events)?,
            ConsumableAction::UndoRedeal => self.undo_redeal(events)?,
            ConsumableAction::PeekSkat => self.peek_skat(events)?,
            ConsumableAction::NextGame(effect) => {
                let before_declaration = self.phase == RunPhase::Shop
                    || self.declaring().is_ok()
                    || self.game.as_ref().is_some_and(|a| {
                        matches!(
                            a.game.phase(),
                            Phase::Discarding | Phase::DeclaringAfterSkat
                        ) && a.game.declaration().is_none()
                    });
                if !before_declaration {
                    return Err(RunError::WrongPhase);
                }
                let duplicate = *effect == NextGameEffect::SingleLoss
                    && self.pending_next_game.iter().any(|(_, e)| e == effect);
                if duplicate {
                    return Err(RunError::WrongPhase);
                }
                self.pending_next_game.push((id.clone(), *effect));
            }
            ConsumableAction::ChalkCard => {
                let target = target.ok_or(RunError::TargetRequired)?;
                let suit = suit.ok_or(RunError::TargetRequired)?;
                self.chalk(target, suit, events)?;
            }
        }
        self.consumables.remove(slot);
        events.push(RunEvent::ConsumableUsed { consumable: id });
        Ok(())
    }

    /// "Kurzer": show the player the untouched skat while he still holds
    /// ten cards and has not declared. Once per game; not against "Der
    /// Kiebitz-Mörder" (information is off).
    fn peek_skat(&mut self, events: &mut Vec<RunEvent>) -> Result<(), RunError> {
        let active = self.declaring()?;
        if active.skat_peeked || active.rules.no_information {
            return Err(RunError::WrongPhase);
        }
        active.skat_peeked = true;
        events.push(RunEvent::SkatPeeked {
            skat: active.game.skat(),
        });
        Ok(())
    }

    /// "Kreide": repaint the card `id` of the player's hand into `suit`
    /// until the game ends. Only before the declaration, never a jack (it
    /// is trump by its rank) or a blank, and only into another suit.
    fn chalk(&mut self, id: u32, suit: Suit, events: &mut Vec<RunEvent>) -> Result<(), RunError> {
        let active = self.active_mut()?;
        if active.rules.ramsch || active.game.declaration().is_some() {
            return Err(RunError::WrongPhase);
        }
        if active.awaiting_re {
            return Err(RunError::AwaitingKontraAnswer);
        }
        let old = Run::declarer_card(active, id)?;
        let face = old.face().ok_or(RunError::CannotChalk)?;
        if face.rank == Rank::Jack || face.suit == suit {
            return Err(RunError::CannotChalk);
        }
        let new = DealtCard {
            kind: CardKind::Normal(Card::new(suit, face.rank)),
            ..old
        };
        let game_events = active.game.chalk(old, new)?;
        if !active.chalked.iter().any(|(c, _)| *c == id) {
            active.chalked.push((id, face.suit));
        }
        events.extend(game_events.into_iter().map(RunEvent::Game));
        Ok(())
    }

    /// Card modifications a random draw may give a card that already
    /// carries `mods`, with their draw weight: for the character, unlocked,
    /// not on the card yet, no curse (DESIGN 7.2: curses only where
    /// announced), in id order. Rarity does not count here; the weight is
    /// the character's weight for the modification's Spielart
    /// ([`Run::focus_weight`]), and modifications of weight 0 are left out.
    fn card_mod_pool(&self, content: &Content, mods: &[CardModId]) -> Vec<(CardModId, u32)> {
        self.mod_pool(content, mods, false)
    }

    /// Like [`Run::card_mod_pool`], but only curses (`curse`) or only
    /// modifications without drawback.
    fn mod_pool(
        &self,
        content: &Content,
        mods: &[CardModId],
        curse: bool,
    ) -> Vec<(CardModId, u32)> {
        content
            .card_mods
            .values()
            .filter(|m| m.curse == curse)
            .filter(|m| in_pool(&m.characters, self.character) && !mods.contains(&m.id))
            .filter(|m| !self.locked_card_mods.contains(&m.id))
            .map(|m| (m.id.clone(), self.focus_weight(content, m.focus)))
            .filter(|&(_, weight)| weight > 0)
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
                let drawn = draw_weighted(&mut self.encounter_rng, pool, *offers);
                options.extend(
                    drawn
                        .into_iter()
                        .map(|id| free(EncounterChoice::TakeRule(id))),
                );
            }
            EncounterKind::MarkCard {
                price,
                card_mod,
                curse,
            } => {
                // The modification is known before the choice: named by the
                // encounter or drawn now from the character's pool (a curse
                // only if the encounter announces one).
                let card_mod = match card_mod {
                    Some(id) => Some(id.clone()),
                    None => {
                        let pool = self.mod_pool(content, &[], *curse);
                        draw_weighted(&mut self.encounter_rng, pool, 1).pop()
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
                let pool = self.relic_pool(content, false);
                let new = draw_weighted(&mut self.encounter_rng, pool, 1).pop();
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
    /// (saves from before the draw at opening) a random one from the
    /// character's pool, never a curse.
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
        draw_weighted(&mut self.encounter_rng, options, 1)
            .pop()
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
                        let pool = self.relic_pool(content, false);
                        draw_weighted(&mut self.encounter_rng, pool, 1)
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

    /// Einmischen each game of the series allows, for the game in
    /// progress: `run.ron` and the character ([`Content::redeals`]) plus
    /// one per "Schieben" in force ([`GameRules::extra_redeals`]).
    fn redeal_allowance(&self, content: &Content) -> u32 {
        let extra = self.game.as_ref().map_or(0, |a| a.rules.extra_redeals);
        content.redeals(self.character).saturating_add(extra)
    }

    /// Einmischen left now: what the game in progress still allows of its
    /// own, plus the extras of the series ("Ein Bier", "Kräuter"). 0
    /// without a game. Whether one may be used right now (only with ten
    /// cards before the skat, never in the Ramsch) is
    /// [`Run::accepts`]`(`[`RunAction::Redeal`]`)`.
    #[must_use]
    pub fn redeals_left(&self, content: &Content) -> u32 {
        if self.game.is_none() {
            return 0;
        }
        self.redeal_allowance(content)
            .saturating_sub(self.series.redeals_in_game)
            .saturating_add(self.series.extra_redeals)
    }
}

/// Zielwert of a series: the character's value (from `characters.ron`, or
/// `run.ron` if the character has none), for a boss series times the
/// boss's target factor. `section` is the row of the curve
/// ([`crate::content::RunConfig::curve_row`]; for a full run the section
/// itself). `None` if the row does not exist.
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
        .map_or(crate::content::Factor::ONE, |def| {
            def.target_factor_for(character)
        });
    Some(factor.apply(base))
}

/// The line a score effect (Grundwert, Gewinnstufen, Spielwert) adds, with
/// the run counters as they are now. `None` for every other effect and for
/// a counter that is still zero.
fn score_line(
    source: &Source,
    effect: &Effect,
    counters: &BTreeMap<String, u32>,
) -> Option<ScoreLine> {
    let (lever, change) = match effect {
        Effect::AddBaseValue(n) => (Lever::BaseValue, Change::Add(i64::from(*n))),
        Effect::MultiplyBaseValue(n) => (Lever::BaseValue, Change::Multiply(*n)),
        Effect::AddBaseValueFromCounter(name) => {
            let value = counters.get(name).copied().unwrap_or(0);
            if value == 0 {
                return None;
            }
            (Lever::BaseValue, Change::Add(i64::from(value)))
        }
        Effect::AddBaseValuePerCounter { counter, per } => {
            let value = counters.get(counter).copied().unwrap_or(0);
            let add = i64::from(value) * i64::from(*per);
            if add == 0 {
                return None;
            }
            (Lever::BaseValue, Change::Add(add))
        }
        Effect::AddLevels(n) => (Lever::Levels, Change::Add(i64::from(*n))),
        Effect::MultiplyGameValue(n) => (Lever::GameValue, Change::Multiply(*n)),
        Effect::ScaleGameValue(f) => (Lever::GameValue, Change::Scale(*f)),
        _ => return None,
    };
    Some(ScoreLine {
        source: source.clone(),
        lever,
        change,
    })
}

/// The Spielwert factors of Kontra and Re.
fn kontra_lines(kontra: Kontra) -> Vec<ScoreLine> {
    match kontra {
        Kontra::None => Vec::new(),
        Kontra::Kontra(_) => vec![kontra_line(Source::Kontra)],
        Kontra::Re(_) => vec![kontra_line(Source::Kontra), kontra_line(Source::Re)],
    }
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

/// Draw up to `count` distinct items by their weight (items of weight 0
/// never), in a fixed pool order.
fn draw_weighted<T: Clone>(rng: &mut Rng, mut pool: Vec<(T, u32)>, count: u32) -> Vec<T> {
    let mut drawn = Vec::new();
    for _ in 0..count {
        let total: u64 = pool.iter().map(|(_, w)| u64::from(*w)).sum();
        if total == 0 {
            break;
        }
        let mut roll = rng.gen_range(0..total);
        let mut chosen = None;
        for (i, (_, weight)) in pool.iter().enumerate() {
            let w = u64::from(*weight);
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
mod character_tests;
#[cfg(test)]
mod content_tests;
#[cfg(test)]
mod m4_tests;
#[cfg(test)]
mod m5_tests;
#[cfg(test)]
mod m7_tests;
#[cfg(test)]
mod schnapps_tests;
#[cfg(test)]
mod settle_tests;
#[cfg(test)]
mod skat_order_tests;
#[cfg(test)]
mod stake_tests;
#[cfg(test)]
mod step_tests;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod wave2_tests;

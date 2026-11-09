//! Run state machine (design document 5, 6 and 12).
//!
//! A run is a Kneipentour (ADR 0016) through a city: three districts
//! (Stadtteile, `section` in the code), each with four pubs (Kneipen) and a
//! boss pub. A pub is played as a [`Series`] (the name of the structure
//! before ADR 0016): a target (Zielwert) and a game limit of three games
//! against two Stammgäste. Before declaring, the player may have each game
//! dealt again a few times (einmischen, [`RunAction::Redeal`]) without
//! using up a game; then one of the hands must be played. Won games add
//! their value to the pub's account, lost games subtract their value once
//! (since 04.11.2026, [`crate::content::RUN_LOSS_FACTOR`]; the Skatordnung
//! says twice) including all relic modifiers. A pub is won the moment the account
//! reaches the target; a boss pub plays all its games and counts after the
//! last (ADR 0017). Missing the target after the last game loses the
//! run, in every pub (ADR 0019: no lives); only the "Schutzengel" has the
//! pub played once more ([`crate::content::Effect::SecondChance`]). A won
//! boss pub pays double and gives a pick of relics ([`tour::BossReward`]). Two
//! boss players sit at its table, one per defender seat ([`BOSS_SEAT`],
//! [`PARTNER_SEAT`]). Between two pubs the player
//! chooses a route ([`tour`]): three ways (Verkehrsmittel), each leading
//! over one or two stops (Tresen, encounters), sometimes into a Spelunke
//! instead of the next pub.
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
//! `OnTrickEnd`, `OnScore` and, after a won pub, `OnSeriesWon`. Sources
//! fire in a fixed order: character, boss, the malus of a Spelunke, relics
//! in purchase order, the house rules of the district (encounter
//! "Stammtisch-Streit"), card modifications of
//! the cards in play by card id. Conditions see the effects of earlier
//! sources in the same stage. A card modification's `OnAttach` hooks fire
//! when it is attached to a card (curses pay Groschen there).
//!
//! A run is played by one [`Character`] (DESIGN 9): the Alleinspieler
//! declares suit games and grand, the Nullspieler only Null. What is
//! unlocked across runs lives outside the run, in
//! [`meta::MetaState`]; a run only gets the relics that are still locked
//! ([`RunStart::locked_relics`]).

pub mod bot;
mod card_reward;
mod check;
mod encounters;
pub mod guests;
pub mod meta;
pub mod night;
pub mod opponents;
pub mod player;
mod schnapps_reward;
pub mod tour;
pub mod trade;
pub mod view;
pub mod zinken;

mod settle;

mod action;
mod error;
mod event;
mod migrate;
mod state;
mod value_preview;
pub use action::RunAction;
pub use error::{RunError, RunErrorCode};
pub use event::RunEvent;
pub use migrate::{REFUNDED_CONSUMABLES, RENAMED_CONSUMABLES, RETIRED_BOSSES};
use state::is_zero;
pub use state::{
    ActiveEncounter, ActiveGame, BOSS_SEAT, EncounterOption, GameRules, Kontra, NextPubPlan,
    OptionBlock, PARTNER_SEAT, Payout, RunPhase, RunStats, Series, Shop, Wager, sell_price,
};

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use crate::ai::DefenderHeuristic;
use crate::cards::{Card, PlayingCard, Rank, Suit};
use crate::content::{
    ActiveSource, BetOn, BossId, CardFilter, CardModDef, CardModId, Change, Character,
    ConsumableAction, ConsumableId, Content, Counted, Editions, Effect, EffectKind, EncounterId,
    GameFocus, Hook, HookContext, Lever, NextGameEffect, RelicId, RelicKind, RunLength,
    ScoreBreakdown, ScoreLine, Source, Stage, card_mod_sources, evaluate, in_pool, resolve_edition,
    score_parts,
};
use crate::deck::{CardKind, DealtCard, DeckChange, DeckError, Edition, Owner};
use crate::game::{Event, Game, Phase, Seat};
use crate::rng::Rng;
use crate::rules::{
    Declaration, GameType, NullScoring, Thresholds, ValueParts, game_value_parts, null_value_parts,
    ramsch_value_parts, suit_strength, trump_strength,
};

/// Pubs per district (Stadtteil, ADR 0016): four Kneipen and the
/// Boss-Kneipe last.
pub const PUBS_PER_DISTRICT: u32 = 5;

/// Mixed into the game's seed for the card "Drei drücken" draws, so that
/// the draw is independent of the PIMC samples of the same game.
const DRAW_SEED: u64 = 0xD8E1_D8E1_0000_0003;

/// Salt of the draw of the voucher a hand game redeems
/// ([`Run::redeem_hand_voucher`]): its own generator from the game's seed,
/// so the deal and every other draw stay the same.
const HAND_VOUCHER_SEED: u64 = 0x4A4E_D7E1_0000_0004;

/// Salt of the draw of the defender cards "Glasauge" makes "Durchsichtig"
/// for a game ([`Effect::RevealDefenderCards`]).
const TRANSPARENT_SEED: u64 = 0x61A5_5E7E_0000_0005;

/// Salt of the generator that picks the card of
/// [`Effect::AttachModToRandomCard`] (relics v4).
const ATTACH_SEED: u64 = 0xA77A_C4ED_0000_0006;

/// "Untot" (DESIGN 7.2, 03.11.2026): the player picks this many cards of
/// his hand, one of them is drawn to take the rising card's place in the
/// trick ([`RunAction::ChooseRiseCards`]).
pub const UNDEAD_PICK: usize = 3;

/// The editions of the two boss players of a game, left and right; both
/// `None` outside a boss pub.
type BossEditions = [Option<Edition>; 2];

/// How a run starts. The default is a fresh run of the Alleinspieler with
/// nothing locked, first series of the first section.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct RunStart {
    /// Relics owned from the start besides the character's start relics
    /// (the simulator's `--relic`), in this order after them. Their passive
    /// effects already apply to the first game.
    pub relics: Vec<RelicId>,
    /// District of the first pub, from 0.
    pub section: u32,
    /// The first pub within its district (the last is the boss pub).
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
    /// Ganzer Abend; `Short` is rejected since ADR 0016
    /// ([`RunError::InvalidStart`]).
    #[serde(default)]
    pub length: RunLength,
    /// Einsatz-Leiter (DESIGN 9.3): suit games and Grand are won from
    /// `61 + stake` Augen. 0 is the normal game.
    #[serde(default)]
    pub stake: u32,
    /// The Einführung (DESIGN 16.1): the first pub has the Zielwert and
    /// the fixed deals of `tutorial.ron` and no Einmischen. Use
    /// [`Run::new_tutorial`], which also takes the tutorial's seed.
    #[serde(default)]
    pub tutorial: bool,
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
    /// The player's edition of the 32 cards (ADR 0015).
    edition: Edition,
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
    /// An encounter showed the player both skat cards of every deal until
    /// the next declaration ([`EncounterEffect::SeeSkat`]).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    peek_next_skat: Option<EncounterId>,
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
    // `lives` (Leben, ADR 0016) and `spared_section` ("Schutzengel" once
    // per Stadtteil) of saves from before ADR 0019 are read and ignored.
    /// Routes, Spelunken, maluses (ADR 0016).
    #[serde(default = "Run::default_tour_rng")]
    tour_rng: Rng,
    /// The open route choice.
    #[serde(default)]
    route: Option<tour::RouteChoice>,
    /// The route taken: stops still ahead and the pub it leads to.
    #[serde(default)]
    trip: Option<tour::Trip>,
    /// Every pub played so far, in order.
    #[serde(default)]
    pub_results: Vec<tour::PubResult>,
    /// Changes waiting for the next pub.
    #[serde(default, skip_serializing_if = "NextPubPlan::is_empty")]
    next_pub: NextPubPlan,
    /// Groschen owed at the Tresen ("Deckel anschreiben"); later income
    /// pays it first.
    #[serde(default, skip_serializing_if = "is_zero")]
    debt: u32,
    /// The district in which the Tresen sold on credit.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    credit_section: Option<u32>,
    /// The bet on the next declared game ("Die Wette").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    wager: Option<Wager>,
    /// Route choices still to come whose exact stops are shown ("Kiebitz am
    /// Fenster").
    #[serde(default, skip_serializing_if = "is_zero")]
    scout_legs: u32,
    /// The relic pick of a won boss pub, while open (ADR 0017).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    boss_reward: Option<tour::BossReward>,
    /// The Auslage and the Kartenmaler (Zinken, DESIGN 8.6).
    #[serde(default = "Run::default_zinken_rng")]
    zinken_rng: Rng,
    /// The Kartenmaler's choice, waiting for the player.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    painter: Option<zinken::PainterChoice>,
    /// Curses as trade (DESIGN 8.6): whether an offer is a "Schnäppchen"
    /// or "Hehlerware", and which card a trade curses with which curse.
    /// Its own generator, so that the other draws stay as they were.
    #[serde(default = "Run::default_trade_rng")]
    trade_rng: Rng,
    /// The Schnaps after a won pub (Schnaps-Leiste, DESIGN 8.9). Its own
    /// generator, so that every other draw stays as it was.
    #[serde(default = "Run::default_schnapps_rng")]
    schnapps_rng: Rng,
    /// The Schnaps of a won pub while every slot is full: the player
    /// takes it for one of his or sells it ([`RunAction::ChooseSchnappsReward`]).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    schnapps_reward: Option<ConsumableId>,
    /// The card reward after a won pub (DESIGN 5.0). Its own generator,
    /// so that every other draw stays as it was.
    #[serde(default = "Run::default_reward_rng")]
    reward_rng: Rng,
    /// The card reward of a won pub, while open ([`RunPhase::CardReward`]).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    card_reward: Option<card_reward::CardReward>,
    /// The long night after der Alte (ADR 0020), once the player continued.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    night: Option<night::NightState>,
    /// The Einführung (DESIGN 16.1, [`RunStart::tutorial`]).
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    tutorial: bool,
}

impl Run {
    /// Start the Einführung (DESIGN 16.1): a run of the Alleinspieler with
    /// the seed of `tutorial.ron` whose first pub deals the fixed games of
    /// that file, with its Zielwert and without Einmischen. It needs no
    /// meta state; the web app keeps it out of the statistics and saves.
    pub fn new_tutorial(content: &Content) -> Result<(Run, Vec<RunEvent>), RunError> {
        Run::new_with(
            content,
            content.tutorial.seed,
            &RunStart {
                tutorial: true,
                ..RunStart::default()
            },
        )
    }

    /// Whether this run is the Einführung.
    #[must_use]
    pub fn is_tutorial(&self) -> bool {
        self.tutorial
    }

    /// Whether the pub in progress is the guided pub of the Einführung
    /// (the first pub of the run).
    fn in_tutorial_pub(&self) -> bool {
        self.tutorial && self.series.section == 0 && self.series.index == 0
    }

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
        let sections = content.run.sections();
        if start.section >= sections
            || start.index >= PUBS_PER_DISTRICT
            || start.length == RunLength::Short
        {
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
        if !content.relic_fits(start.character, relics.len()) {
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
        let mut edition = Edition::standard();
        edition.apply_spec(&character.deck, content)?;
        let mut master = Rng::from_seed(seed);
        let mut run = Run {
            seed,
            deal_rng: master.fork(),
            shop_rng: master.fork(),
            misc_rng: master.fork(),
            phase: RunPhase::Game,
            series: Series::default(),
            coins: content.run.economy.starting_coins,
            edition,
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
            tour_rng: master.fork(),
            route: None,
            trip: None,
            pub_results: Vec::new(),
            next_pub: NextPubPlan::default(),
            debt: 0,
            credit_section: None,
            wager: None,
            peek_next_skat: None,
            scout_legs: 0,
            boss_reward: None,
            zinken_rng: master.fork(),
            painter: None,
            trade_rng: master.fork(),
            schnapps_rng: master.fork(),
            schnapps_reward: None,
            reward_rng: master.fork(),
            card_reward: None,
            night: None,
            tutorial: start.tutorial,
        };
        let mut events = Vec::new();
        run.start_pub(content, start.section, start.index, None, &mut events)?;
        run.play_ai(content, &mut events)?;
        Ok((run, events))
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

    /// Number of districts (Stadtteile) of this run.
    #[must_use]
    pub fn sections(&self, content: &Content) -> u32 {
        content.run.sections()
    }

    /// The row of the curve (Zielwerte, opponents) the current district
    /// plays: since ADR 0016 the district itself.
    #[must_use]
    pub const fn curve_row(&self) -> u32 {
        self.series.section
    }

    /// Relics that are not unlocked in this run.
    #[must_use]
    pub const fn locked_relics(&self) -> &BTreeSet<RelicId> {
        &self.locked
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

    /// The player's edition of the 32 cards (ADR 0015).
    #[must_use]
    pub const fn edition(&self) -> &Edition {
        &self.edition
    }

    /// The editions of the game in progress: the player's and the boss
    /// edition at the table, if any.
    #[must_use]
    pub fn editions(&self) -> Editions<'_> {
        Editions {
            player: &self.edition,
            boss: self.game.as_ref().and_then(|a| a.boss_edition.as_ref()),
            partner: self.game.as_ref().and_then(|a| a.partner_edition.as_ref()),
        }
    }

    /// Whose edition the seat plays (ADR 0015): the player's, one of the
    /// two boss players' (both defender seats in a boss pub, ADR 0017) or
    /// a Stammgast's.
    #[must_use]
    pub fn owner_of_seat(&self, seat: Seat) -> Owner {
        if seat.is_declarer() {
            Owner::Player
        } else if self.series.boss.is_none() {
            Owner::Guest
        } else if seat == BOSS_SEAT {
            Owner::Boss
        } else {
            Owner::BossPartner
        }
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
    /// (`None`: in the skat or already played), each as its holder holds
    /// it. The knowledge hangs on the copy (DESIGN 7.2, ADR 0015): a card
    /// is shown only if the copy in play is marked, i.e. its owner's
    /// edition (the player's own, a boss card such as the black Unter of
    /// "Der Schwarze Peter") or a mark for this game ("Glasauge"). A
    /// "Durchsichtig" on the player's copy never shows a defender's copy
    /// of the same card (fixed 04.11.2026).
    #[must_use]
    pub fn revealed_holders(&self) -> Vec<(DealtCard, Option<Seat>)> {
        let Some(active) = &self.game else {
            return Vec::new();
        };
        // "Glasauge" marks defender cards for this game by id: a card of
        // the defenders' editions, never shown while the player holds it.
        let for_game = |c: &DealtCard| {
            active.transparent_for_game.contains(&c.id)
                && active.game.holder_of(*c) != Some(Seat::DECLARER)
        };
        active
            .game
            .cards_in_play()
            .iter()
            .map(|&c| active.game.current(c))
            .filter(|c| c.revealed || for_game(c))
            .map(|c| (c, active.game.holder_of(c)))
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
        if active.awaiting_re
            || active.pending_rise.is_some()
            || active.game.phase() != Phase::Playing
        {
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

    /// Carry out `action`. Only ever called on a copy of the run
    /// ([`Run::apply`], [`Run::apply_step`], [`Run::check`]): on an error
    /// the copy and `events` are dropped, so an action may change the run
    /// and log events before it fails. Actions therefore mutate `self`
    /// directly instead of cloning the run once more.
    fn dispatch(
        &mut self,
        content: &Content,
        action: RunAction,
        events: &mut Vec<RunEvent>,
    ) -> Result<(), RunError> {
        self.check_pending(&action)?;
        match action {
            RunAction::Redeal => self.redeal_action(content, events),
            RunAction::PickUpSkat => self.pick_up_skat(events),
            RunAction::Declare(declaration) => self.declare(content, declaration, events),
            RunAction::AnswerKontra { re } => self.answer_kontra(content, re, events),
            RunAction::Discard(ids) => self.discard(content, ids, events),
            RunAction::DiscardThree(ids) => self.discard_three(content, ids, events),
            RunAction::Play(id) => self.play(content, id, events),
            RunAction::UseConsumable { slot, target, suit } => {
                self.use_consumable(content, slot, target, suit, events)
            }
            RunAction::BuyRelic(slot) => self.buy_relic(content, slot, events),
            RunAction::BuyConsumable(slot) => self.buy_consumable(content, slot, events),
            RunAction::SellRelic(index) => self.sell_relic(content, index, events),
            RunAction::ReplaceRelic { slot, replace } => {
                self.replace_relic(content, slot, replace, events)
            }
            RunAction::SellConsumable(index) => self.sell_consumable(content, index, events),
            RunAction::Reroll => self.reroll(content, events),
            RunAction::BuyCardOffer(slot) => self.buy_card_offer(content, slot, events),
            RunAction::HirePainter(card) => self.hire_painter(content, card, events),
            RunAction::ChoosePaint(option) => self.choose_paint(content, option, events),
            RunAction::ChooseRiseCards(ids) => self.choose_rise_cards(content, ids, events),
            RunAction::CleanseCard(card) => self.cleanse_at_tresen(content, card, events),
            RunAction::LeaveShop => self.leave_shop(content, events),
            RunAction::GiveUpPub => self.give_up_pub(content, events),
            RunAction::TakeFromSkat { give, take } => self.take_from_skat(give, take, events),
            RunAction::SettleGame => self.settle_game(content, events),
            RunAction::ChooseEncounter {
                option,
                target,
                marke,
            } => self.choose_encounter(content, option, target, marke.as_ref(), events),
            RunAction::ChooseRoute(option) => self.choose_route(content, option, events),
            RunAction::ChooseBossRelic { offer, replace } => {
                self.choose_boss_relic(content, offer, replace, events)
            }
            RunAction::ChooseMalus { option, malus } => {
                self.choose_malus(content, option, malus, events)
            }
            RunAction::ChooseSchnappsReward { give_up } => {
                self.choose_schnapps_reward(content, give_up, events)
            }
            RunAction::TakeCardReward { offer } => self.take_card_reward(content, offer, events),
            RunAction::SkipCardReward => self.skip_card_reward(content, events),
            RunAction::EnterNight => self.enter_night(content, events),
        }
    }

    /// What [`Run::dispatch`] checks before any action: a defender to
    /// move, or a choice that must come first.
    fn check_pending(&self, action: &RunAction) -> Result<(), RunError> {
        if self.ai_to_move().is_some() {
            return Err(RunError::AiToMove);
        }
        if self.painter.is_some() && !matches!(action, RunAction::ChoosePaint(_)) {
            return Err(RunError::PainterPending);
        }
        if self.rise_pending().is_some() && !matches!(action, RunAction::ChooseRiseCards(_)) {
            return Err(RunError::RisePending);
        }
        if self.schnapps_reward.is_some()
            && !matches!(action, RunAction::ChooseSchnappsReward { .. })
        {
            return Err(RunError::SchnappsRewardPending);
        }
        if self.phase == RunPhase::CardReward
            && !matches!(
                action,
                RunAction::TakeCardReward { .. }
                    | RunAction::SkipCardReward
                    | RunAction::ChooseSchnappsReward { .. }
            )
        {
            return Err(RunError::CardRewardPending);
        }
        Ok(())
    }

    /// The game in progress, in the game phase.
    fn active_ref(&self) -> Result<&ActiveGame, RunError> {
        if self.phase != RunPhase::Game {
            return Err(RunError::WrongPhase);
        }
        self.game.as_ref().ok_or(RunError::WrongPhase)
    }

    fn active_mut(&mut self) -> Result<&mut ActiveGame, RunError> {
        self.active_ref()?;
        self.game.as_mut().ok_or(RunError::WrongPhase)
    }

    /// The game in progress before the skat is touched, no Ramsch.
    fn declaring_ref(&self) -> Result<&ActiveGame, RunError> {
        let active = self.active_ref()?;
        if active.rules.ramsch {
            return Err(RunError::RamschRequired);
        }
        if active.game.phase() != Phase::Declaring {
            return Err(RunError::WrongPhase);
        }
        Ok(active)
    }

    fn declaring(&mut self) -> Result<&mut ActiveGame, RunError> {
        self.declaring_ref()?;
        self.game.as_mut().ok_or(RunError::WrongPhase)
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
        } else if self.series.extra_redeals > 0 && !self.malus(content).is_some_and(|m| m.no_redeal)
        {
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
        let keep = self.kept_at_redeal(content);
        self.series.redeals_this_game += 1;
        let previous = self.game.take().map(|mut active| {
            active.previous = None;
            Box::new(active)
        });
        self.series.redeals_used += 1;
        self.stats.redeals += 1;
        events.push(RunEvent::Redealt);
        self.deal_keeping(content, false, &keep, events)?;
        if let Some(active) = self.game.as_mut() {
            active.previous = previous;
        }
        Ok(())
    }

    /// The cards of the player's hand an Einmischen leaves him
    /// ([`Effect::RedealKeeps`], "Unterschlupf"): his own cards that match a
    /// filter as he holds them; a Unter his edition demotes is no Unter.
    /// A filter with `per_game` counts only at the first that many
    /// Einmischen of the game ([`Series::redeals_this_game`]).
    fn kept_at_redeal(&self, content: &Content) -> Vec<u32> {
        let Some(active) = self.game.as_ref() else {
            return Vec::new();
        };
        let done = self.series.redeals_this_game;
        let filters: Vec<CardFilter> = self
            .sources(content, false)
            .iter()
            .flat_map(|s| s.passive.iter())
            .filter_map(|e| match e {
                Effect::RedealKeeps { filter, per_game } => {
                    per_game.is_none_or(|n| done < n).then_some(*filter)
                }
                _ => None,
            })
            .collect();
        if filters.is_empty() {
            return Vec::new();
        }
        active
            .game
            .hand(Seat::DECLARER)
            .iter()
            .filter(|c| c.owner == Owner::Player && !crate::rules::is_demoted_jack(**c))
            .filter(|c| {
                c.face()
                    .is_some_and(|f| filters.iter().any(|filter| filter.matches(f)))
            })
            .map(|c| c.id)
            .collect()
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
    ///
    /// "Drei drücken": at the first pickup of the series the third card is
    /// a random card of the left defender, drawn with a generator of its
    /// own from the game's seed, so the deal and every other draw stay the
    /// same.
    fn pick_up_skat(&mut self, events: &mut Vec<RunEvent>) -> Result<(), RunError> {
        let mut draw_rng = Rng::from_seed(opponents::game_seed(self) ^ DRAW_SEED);
        let draw = self.discard_three_ready();
        let active = self.declaring()?;
        if active.rules.require_hand {
            return Err(RunError::HandRequired);
        }
        let game_events = if draw {
            let from = Seat::DEFENDERS[0];
            let hand = active.game.hand(from);
            let pick = draw_rng.gen_range(0..u64::try_from(hand.len()).unwrap_or(1));
            let card = hand
                .get(usize::try_from(pick).unwrap_or(0))
                .copied()
                .ok_or(RunError::WrongPhase)?;
            let game_events = active.game.pick_up_skat_drawing(from, card)?;
            self.series.discard_three_used = true;
            game_events
        } else {
            active.game.pick_up_skat()?
        };
        events.extend(game_events.into_iter().map(RunEvent::Game));
        Ok(())
    }

    /// Whether "Drei drücken" draws a card at the next pickup of the
    /// skat: granted in this game, the first game of the pub (since
    /// 02.11.2026), and not yet used in it.
    #[must_use]
    pub fn discard_three_ready(&self) -> bool {
        !self.series.discard_three_used
            && self.series.games_played == 0
            && self
                .game
                .as_ref()
                .is_some_and(|a| a.rules.discard_three.is_some())
    }

    /// What [`Run::declare`] checks before it changes anything.
    fn check_declare(&self, declaration: Declaration) -> Result<&ActiveGame, RunError> {
        let active = self.active_ref()?;
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
        Ok(active)
    }

    fn declare(
        &mut self,
        content: &Content,
        declaration: Declaration,
        events: &mut Vec<RunEvent>,
    ) -> Result<(), RunError> {
        let active = self.check_declare(declaration)?;
        // Before the skat a declaration is always a hand game: it redeems a
        // voucher of the hand (DESIGN 7.2), before the game starts.
        if active.game.phase() == Phase::Declaring {
            self.redeem_hand_voucher(content, events);
        }
        let active = self.active_mut()?;
        let game_events = active.game.declare(declaration)?;
        active.note_settled(&game_events);
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
        self.peek_next_skat = None;
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
                NextGameEffect::MultiplyWin(factor) => active.win_factors.push((source, factor)),
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
        active.note_settled(&game_events);
        events.extend(game_events.into_iter().map(RunEvent::Game));
        self.redeem_vouchers(content, events);
        self.settle(content, events)
    }

    /// "Drei drücken": discard three of the thirteen cards.
    fn discard_three(
        &mut self,
        content: &Content,
        ids: [u32; 3],
        events: &mut Vec<RunEvent>,
    ) -> Result<(), RunError> {
        let active = self.active_mut()?;
        if active.awaiting_re {
            return Err(RunError::AwaitingKontraAnswer);
        }
        let cards = [
            Run::declarer_card(active, ids[0])?,
            Run::declarer_card(active, ids[1])?,
            Run::declarer_card(active, ids[2])?,
        ];
        let game_events = active.game.discard_three(cards)?;
        active.note_settled(&game_events);
        events.extend(game_events.into_iter().map(RunEvent::Game));
        self.redeem_vouchers(content, events);
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
        if self.mirrors_now(content, card) {
            let active = self.active_mut()?;
            let game_events = active.game.play_mirrored(Seat::DECLARER, card)?;
            let copy = game_events
                .iter()
                .find_map(|e| match e {
                    Event::DoppelgangerMirrored { copy, .. } => Some(*copy),
                    _ => None,
                })
                .unwrap_or(card);
            let trick_done = game_events
                .iter()
                .any(|e| matches!(e, Event::TrickWon { .. }));
            events.extend(game_events.into_iter().map(RunEvent::Game));
            self.after_card(content, Seat::DECLARER, copy, trick_done, events);
            return self.settle(content, events);
        }
        self.play_card(content, Seat::DECLARER, card, events)?;
        self.settle(content, events)
    }

    /// Whether the player's `card` is a "Doppelgänger" whose mirror is
    /// still to come in this game ("Spiegelbild", DESIGN 7.2, since
    /// 07.11.2026): it is in his hand, the game is a suit game or Grand
    /// (not Null, not a Ramsch), and no Doppelgänger mirrored in this game
    /// yet (once per game, whatever the number of Doppelgänger). Played as
    /// the last card of a trick it would mirror ([`Run::mirrors_now`]).
    /// `false` outside a game.
    #[must_use]
    pub fn can_mirror(&self, content: &Content, card: DealtCard) -> bool {
        let Some(active) = self.game.as_ref() else {
            return false;
        };
        let game = &active.game;
        let trump_game = game
            .declaration()
            .is_some_and(|d| !d.is_null() && !d.is_ramsch());
        // The scan of the log last: most cards are no Doppelgänger.
        trump_game
            && !card.is_blank()
            && game.hand(Seat::DECLARER).contains(&card)
            && self.has_card_effect(content, card, &Effect::Doppelganger)
            && !game
                .log()
                .iter()
                .any(|e| matches!(e, Event::DoppelgangerMirrored { .. }))
    }

    /// Whether the player's `card` mirrors if he plays it now: it may
    /// mirror ([`Run::can_mirror`]), it is his turn as the last card of
    /// the trick and it is a legal play as printed. Otherwise a
    /// Doppelgänger plays as its printed card.
    #[must_use]
    pub fn mirrors_now(&self, content: &Content, card: DealtCard) -> bool {
        let Some(active) = self.game.as_ref() else {
            return false;
        };
        let game = &active.game;
        game.current_trick().len() == 2
            && game.current_seat() == Some(Seat::DECLARER)
            && game.legal_moves(Seat::DECLARER).contains(&card)
            && self.can_mirror(content, card)
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
        self.after_card(content, seat, card, trick_done, events);
        Ok(())
    }

    /// Fire the hooks after a card: `OnCardPlayed`, and `OnTrickEnd` if it
    /// completed a trick.
    fn after_card(
        &mut self,
        content: &Content,
        seat: Seat,
        card: DealtCard,
        trick_done: bool,
        events: &mut Vec<RunEvent>,
    ) {
        self.fire(
            content,
            Stage::OnCardPlayed,
            Some((seat, card)),
            None,
            events,
        );
        if trick_done {
            self.fire(content, Stage::OnTrickEnd, None, None, events);
            self.raise_undead(content, events);
        }
    }

    /// Whether the player's `card` would rise if he played it now and the
    /// trick were complete ("Untot", [`Effect::Undead`]): it carries the
    /// modification, did not rise in this game yet, the game is a suit game
    /// or Grand, and it is not his last card. `false` outside a game.
    #[must_use]
    pub fn can_rise(&self, content: &Content, card: DealtCard) -> bool {
        let Some(active) = self.game.as_ref() else {
            return false;
        };
        let game = &active.game;
        let trump_game = game
            .declaration()
            .is_some_and(|d| !d.is_null() && !d.is_ramsch());
        let hand = game.hand(Seat::DECLARER);
        trump_game
            && !card.is_blank()
            && hand.len() > 1
            && hand.contains(&card)
            && !active.risen.contains(&card.id)
            && self.has_card_effect(content, card, &Effect::Undead)
    }

    /// "Untot" (DESIGN 7.2, reworked 03.11.2026): after the `OnTrickEnd`
    /// hooks of a completed trick, the player's card in it rises back into
    /// his hand if it may ([`Run::can_rise`], judged as if it were still in
    /// the hand), as a copy of the card that won the trick
    /// ([`Game::raise`]). One of [`UNDEAD_PICK`] cards of his hand, drawn
    /// with the game's generator, takes its place in the trick: with more
    /// cards in his hand he picks them first, and the run waits
    /// ([`Run::rise_pending`], [`RunEvent::RiseChoice`]); with as many or
    /// fewer, all of them are the candidates and it rises at once.
    fn raise_undead(&mut self, content: &Content, events: &mut Vec<RunEvent>) {
        let Some(active) = self.game.as_ref() else {
            return;
        };
        let game = &active.game;
        if game.phase() != Phase::Playing || !game.current_trick().is_empty() {
            return;
        }
        let Some(trick) = game.tricks().last() else {
            return;
        };
        let Some(card) = (0..3)
            .find(|&i| trick.seat_of(i).is_declarer())
            .map(|i| trick.cards[i])
        else {
            return;
        };
        let trump_game = game
            .declaration()
            .is_some_and(|d| !d.is_null() && !d.is_ramsch());
        let hand = game.hand(Seat::DECLARER).to_vec();
        if !trump_game
            || card.is_blank()
            || hand.is_empty()
            || active.risen.contains(&card.id)
            || !self.has_card_effect(content, card, &Effect::Undead)
        {
            return;
        }
        if hand.len() > UNDEAD_PICK {
            let winning = trick.winning_card();
            let becomes = if winning == card {
                card
            } else {
                card.risen_as(winning)
            };
            if let Some(active) = self.game.as_mut() {
                active.pending_rise = Some(card.id);
            }
            events.push(RunEvent::RiseChoice {
                card,
                becomes,
                pick: UNDEAD_PICK,
            });
            return;
        }
        self.rise_with(card, &hand, events);
    }

    /// Let the "Untot" `card` of the last trick rise, one of `candidates`
    /// (cards of the player's hand) drawn with the game's generator taking
    /// its place.
    fn rise_with(&mut self, card: DealtCard, candidates: &[DealtCard], events: &mut Vec<RunEvent>) {
        let fallback = opponents::game_seed(self) ^ 0x0DEA_D0DE_AD00;
        let Some(active) = self.game.as_mut() else {
            return;
        };
        let rng = active
            .undead_rng
            .get_or_insert_with(|| Rng::from_seed(fallback));
        let roll = rng.gen_range(0..u64::try_from(candidates.len()).unwrap_or(1));
        let Some(&substitute) = usize::try_from(roll).ok().and_then(|i| candidates.get(i)) else {
            return;
        };
        if let Ok(game_events) = active.game.raise(Seat::DECLARER, card, substitute) {
            active.risen.push(card.id);
            active.pending_rise = None;
            events.extend(game_events.into_iter().map(RunEvent::Game));
        }
    }

    /// The "Untot" card waiting for the player's three cards
    /// ([`RunAction::ChooseRiseCards`]), as it lies in the last trick.
    #[must_use]
    pub fn rise_pending(&self) -> Option<DealtCard> {
        let active = self.game.as_ref()?;
        let id = active.pending_rise?;
        active
            .game
            .tricks()
            .last()
            .and_then(|t| t.cards.iter().copied().find(|c| c.id == id))
    }

    /// [`RunAction::ChooseRiseCards`]: three different cards of the hand,
    /// sorted by id so the draw does not depend on their order; one of
    /// them is drawn and the waiting card rises.
    fn choose_rise_cards(
        &mut self,
        content: &Content,
        ids: [u32; UNDEAD_PICK],
        events: &mut Vec<RunEvent>,
    ) -> Result<(), RunError> {
        let card = self.rise_pending().ok_or(RunError::InvalidRiseCards)?;
        let active = self.active_mut()?;
        let mut sorted = ids;
        sorted.sort_unstable();
        if sorted.windows(2).any(|w| w[0] == w[1]) {
            return Err(RunError::InvalidRiseCards);
        }
        let hand = active.game.hand(Seat::DECLARER);
        let candidates: Vec<DealtCard> = sorted
            .iter()
            .filter_map(|id| hand.iter().copied().find(|c| c.id == *id))
            .collect();
        if candidates.len() != UNDEAD_PICK {
            return Err(RunError::InvalidRiseCards);
        }
        self.rise_with(card, &candidates, events);
        self.settle(content, events)
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
                    scale: 1,
                });
            }
        }
        let bosses = self
            .boss_rule(content)
            .into_iter()
            .chain(self.extra_boss_rules().iter().cloned());
        for def in bosses.filter_map(|b| content.boss(&b)) {
            sources.push(ActiveSource {
                source: Source::Boss(def.id.clone()),
                passive: &def.passive,
                hooks: &def.hooks,
                this_card: None,
                scale: 1,
            });
        }
        if let Some(def) = self.malus(content) {
            sources.push(ActiveSource {
                source: Source::Malus(def.id.clone()),
                passive: &def.passive,
                hooks: &[],
                this_card: None,
                scale: 1,
            });
        }
        for def in self.held_relic_defs(content) {
            sources.push(ActiveSource {
                source: Source::Relic(def.id.clone()),
                passive: &def.passive,
                hooks: &def.hooks,
                this_card: None,
                scale: 1,
            });
        }
        if with_card_mods {
            if let Some(active) = &self.game {
                sources.extend(card_mod_sources(self.editions(), content, &active.game));
            }
        }
        sources
    }

    /// Whether any of [`Run::sources`] has a hook of `stage`, without
    /// building them.
    fn has_hook(&self, content: &Content, stage: Stage, with_card_mods: bool) -> bool {
        let of = |hooks: &[Hook]| hooks.iter().any(|h| h.stage() == stage);
        if content
            .character(self.character)
            .is_some_and(|def| of(&def.rule.hooks))
        {
            return true;
        }
        let bosses = self.boss_rule(content);
        let extra = self.extra_boss_rules();
        if bosses
            .iter()
            .chain(extra)
            .filter_map(|b| content.boss(b))
            .any(|def| of(&def.hooks))
        {
            return true;
        }
        if self.held_relic_defs(content).any(|def| of(&def.hooks)) {
            return true;
        }
        if !with_card_mods {
            return false;
        }
        let Some(active) = &self.game else {
            return false;
        };
        let editions = self.editions();
        active.game.cards_in_play().iter().any(|card| {
            !card.is_blank()
                && editions
                    .of(card.owner)
                    .and_then(|e| e.get(card.id))
                    .is_some_and(|deck_card| {
                        deck_card
                            .mods
                            .iter()
                            .filter_map(|m| content.card_mod(m))
                            .any(|def| of(&def.hooks))
                    })
        })
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
        self.fire_with(content, stage, played, result, None, events);
    }

    /// [`Run::fire`] with the means of transport of the route just taken
    /// (on route chosen).
    fn fire_with(
        &mut self,
        content: &Content,
        stage: Stage,
        played: Option<(Seat, DealtCard)>,
        result: Option<&ValueParts>,
        transport: Option<tour::Transport>,
        events: &mut Vec<RunEvent>,
    ) {
        let with_card_mods = stage.in_game();
        if !self.has_hook(content, stage, with_card_mods) {
            return;
        }
        let games_left = self.series.games_left();
        let sources = self.sources(content, with_card_mods);
        // Only conditions on marked cards look at them.
        let marked = if sources
            .iter()
            .flat_map(|s| s.hooks)
            .any(|h| h.stage() == stage && h.when().uses_marked_cards())
        {
            self.marked_cards(content)
        } else {
            BTreeSet::new()
        };
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
                        transport,
                        marked_cards: Some(&marked),
                        run: Some((self, content)),
                    };
                    evaluate(hook.when(), &ctx, source.this_card)
                };
                if holds {
                    for effect in hook.then() {
                        self.apply_effect(
                            content,
                            &source.source,
                            &effect.scaled(source.scale),
                            events,
                        );
                    }
                }
            }
        }
    }

    /// Apply one effect. Card-level effects were applied when the deck was
    /// resolved and are ignored here.
    fn apply_effect(
        &mut self,
        content: &Content,
        source: &Source,
        effect: &Effect,
        events: &mut Vec<RunEvent>,
    ) {
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
            // Needs the content: applied by `redeem_vouchers`.
            Effect::TurnInto(_) => return,
            Effect::AttachModToRandomCard {
                card_mod,
                value,
                only_unmarked,
            } => {
                self.attach_to_random_card(content, card_mod, *value, *only_unmarked, events);
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
                    self.settle_debt(events);
                }
                return;
            }
            Effect::StashCoins { counter, amount } => {
                let moved = (*amount).min(self.coins);
                if moved > 0 {
                    self.coins -= moved;
                    let pot = self.counters.entry(counter.clone()).or_insert(0);
                    *pot = pot.saturating_add(moved);
                    let pot = *pot;
                    events.push(RunEvent::CoinsStashed {
                        source: source.clone(),
                        amount: moved,
                        pot,
                        coins: self.coins,
                    });
                }
                return;
            }
            Effect::PayOutCounter { counter, percent } => {
                let pot = self.counters.remove(counter).unwrap_or(0);
                if pot > 0 {
                    events.push(RunEvent::CounterReset {
                        counter: counter.clone(),
                    });
                    let paid = u32::try_from(u64::from(pot) * u64::from(*percent) / 100)
                        .unwrap_or(u32::MAX);
                    if paid > 0 {
                        self.coins = self.coins.saturating_add(paid);
                        events.push(RunEvent::CoinsGained {
                            source: source.clone(),
                            amount: i32::try_from(paid).unwrap_or(i32::MAX),
                            coins: self.coins,
                        });
                        self.settle_debt(events);
                    }
                }
                return;
            }
            _ => {}
        }
        let line = if effect.kind() == EffectKind::ScoreLine {
            score_line(
                source,
                effect,
                &self.counters,
                self.count_for(content, effect),
            )
        } else {
            None
        };
        let Some(active) = self.game.as_mut() else {
            return;
        };
        if effect.is_cheat() && matches!(source, Source::Boss(_)) && active.rules.no_cheating {
            return;
        }
        if effect.kind() == EffectKind::ScoreLine {
            active.lines.extend(line);
            return;
        }
        match effect {
            Effect::AddTrickPoints(extra) => {
                if let Some(trick) = active.game.tricks().last() {
                    let winner = trick.winner;
                    if *extra > 0 {
                        active.game.add_bonus_points(winner, *extra);
                        events.push(RunEvent::BonusPoints {
                            source: source.clone(),
                            seat: winner,
                            card_points: *extra,
                        });
                    }
                }
            }
            Effect::MultiplyTrickPoints(factor) => {
                if let Some(trick) = active.game.tricks().last() {
                    let points = u32::try_from(trick.signed_points().max(0)).unwrap_or(0);
                    let extra = points.saturating_mul(factor.saturating_sub(1));
                    if extra > 0 {
                        let winner = trick.winner;
                        active.game.add_bonus_points(winner, extra);
                        events.push(RunEvent::BonusPoints {
                            source: source.clone(),
                            seat: winner,
                            card_points: extra,
                        });
                    }
                }
            }
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
                active.rules.peek = active.rules.peek.saturating_add(*n);
            }
            Effect::RevealDefenderCards(n) => {
                active.rules.transparent_per_defender =
                    active.rules.transparent_per_defender.saturating_add(*n);
                active
                    .rules
                    .transparent_source
                    .get_or_insert_with(|| source.clone());
            }
            Effect::ShowCount => active.rules.show_count = true,
            Effect::RevealDefenderHand(index) => {
                let seat = Seat::from_index(usize::from(*index)).filter(|s| !s.is_declarer());
                if let Some(seat) = seat {
                    active.shown_hand.get_or_insert(seat);
                    if !active.shown_hands.contains(&seat) {
                        active.shown_hands.push(seat);
                    }
                    for &card in active.game.hand(seat) {
                        if !active.peeked.iter().any(|(c, _)| c.id == card.id) {
                            active.peeked.push((card, seat));
                            events.push(RunEvent::CardPeeked { card, seat });
                        }
                    }
                }
            }
            Effect::SeriesSkatSwap(filter) => active.rules.skat_swaps.push(*filter),
            Effect::DiscardThree => {
                active
                    .rules
                    .discard_three
                    .get_or_insert_with(|| source.clone());
            }
            Effect::RevealSkatCard(n) => {
                let untouched = active
                    .game
                    .declaration()
                    .is_some_and(|d| d.hand() && !d.is_ramsch());
                if untouched {
                    let fresh: Vec<DealtCard> = active
                        .game
                        .skat()
                        .into_iter()
                        .filter(|c| !active.skat_shown.iter().any(|s| s.id == c.id))
                        .take(usize::try_from(*n).unwrap_or(usize::MAX))
                        .collect();
                    for card in fresh {
                        active.skat_shown.push(card);
                        events.push(RunEvent::SkatCardShown {
                            source: source.clone(),
                            card,
                        });
                    }
                }
            }
            Effect::NoCheating => active.rules.no_cheating = true,
            Effect::GrandAcesAsMatadors => active.game.count_grand_aces(),
            Effect::OuvertRequiresSchneider => active.rules.thresholds.ouvert_schneider = true,
            Effect::RequireOuvert => active.rules.require_ouvert = true,
            Effect::DefendersSeeEachOther => active.game.open_defender_hands(),
            Effect::Ramsch => active.rules.ramsch = true,
            Effect::SingleLoss => {
                active.single_loss.get_or_insert_with(|| source.clone());
            }
            // Card-level effects (applied when the deck is resolved) and
            // effects of the run outside the game score ([`Effect::kind`]).
            _ => {}
        }
    }

    // -----------------------------------------------------------------------
    // Series flow
    // -----------------------------------------------------------------------

    /// Start pub `index` of district `section`, a Spelunke if `spelunke`
    /// is set (ADR 0016), and deal its first game.
    fn start_pub(
        &mut self,
        content: &Content,
        section: u32,
        index: u32,
        spelunke: Option<tour::Spelunke>,
        events: &mut Vec<RunEvent>,
    ) -> Result<(), RunError> {
        self.open_pub(content, section, index, spelunke, false, events)
    }

    /// Play a lost pub again from the start ("Schutzengel", ADR 0019): the
    /// same pub (or Spelunke with its malus) and boss, a fresh account and
    /// a fresh deal; the house rules of the evening stay, the boss is not
    /// drawn again. Its Zielwert is the pub's own, without changes planned
    /// for it on the way ("Wegbier": they were used up by the first try).
    fn replay_pub(
        &mut self,
        content: &Content,
        section: u32,
        index: u32,
        spelunke: Option<tour::Spelunke>,
        events: &mut Vec<RunEvent>,
    ) -> Result<(), RunError> {
        self.open_pub(content, section, index, spelunke, true, events)
    }

    /// [`Run::start_pub`], or with `replay` [`Run::replay_pub`].
    fn open_pub(
        &mut self,
        content: &Content,
        section: u32,
        index: u32,
        spelunke: Option<tour::Spelunke>,
        replay: bool,
        events: &mut Vec<RunEvent>,
    ) -> Result<(), RunError> {
        let config = &content.run;
        let sections = config.sections();
        if !self.district_exists(content, section) || index >= PUBS_PER_DISTRICT {
            return Err(RunError::WrongPhase);
        }
        if index == 0 && !replay && !self.evening_relics.is_empty() {
            for relic in std::mem::take(&mut self.evening_relics) {
                self.pay_out_stash(content, &relic, events);
            }
            events.push(RunEvent::EveningRulesEnded);
        }
        if (index == 0 && !replay) || self.section_boss.is_none() {
            let last = section + 1 == sections;
            self.section_boss = match (&config.final_boss, last) {
                _ if section >= sections => self.draw_night_boss(content),
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
            let extra = self
                .night
                .as_ref()
                .map(|n| n.extra_bosses.clone())
                .unwrap_or_default();
            if section >= sections && !extra.is_empty() {
                events.push(RunEvent::ExtraBossRules {
                    section,
                    bosses: extra,
                });
            }
        }
        let boss = if index + 1 == PUBS_PER_DISTRICT {
            self.section_boss.clone()
        } else {
            None
        };
        let mut target = self
            .pub_target(content, section, index, boss.as_ref())
            .ok_or(RunError::WrongPhase)?;
        if spelunke.is_some() {
            target = config.spelunke.target_factor.apply(target);
        }
        let plan = std::mem::take(&mut self.next_pub);
        target = plan.apply_to(target);
        if self.tutorial && section == 0 && index == 0 {
            target = content.tutorial.target;
        }
        let redeals_withheld = plan.redeals.min(0).unsigned_abs();
        let malus = spelunke.as_ref().map(|s| s.malus.clone());
        self.series = Series {
            section,
            index,
            target,
            game_limit: config.game_limit,
            boss: boss.clone(),
            spelunke,
            redeals_withheld,
            ..Series::default()
        };
        self.series.night_malus = self.draw_night_malus(content, boss.is_some());
        let malus = malus.or_else(|| self.series.night_malus.clone());
        self.phase = RunPhase::Game;
        self.shop = None;
        self.encounter = None;
        self.route = None;
        self.trip = None;
        self.series.extra_redeals = u32::try_from(plan.redeals.max(0)).unwrap_or(0);
        self.seat_guests(content, section, index);
        if !plan.is_empty() {
            events.push(RunEvent::NextPubApplied {
                redeals: plan.redeals,
                target: plan.target,
            });
        }
        events.push(RunEvent::SeriesStarted {
            section,
            index,
            target,
            game_limit: config.game_limit,
            redeals: content.redeals_in(self.character, section),
            boss,
            malus,
        });
        self.curse_for_pub(content, events);
        self.lay_night_curses(content, events);
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
        self.deal_keeping(content, rotate, &[], events)
    }

    /// [`Run::deal`], the player keeping the cards with the ids in `keep`
    /// in his hand ("Unterschlupf" at an Einmischen).
    fn deal_keeping(
        &mut self,
        content: &Content,
        rotate: bool,
        keep: &[u32],
        events: &mut Vec<RunEvent>,
    ) -> Result<(), RunError> {
        let rotation = if rotate {
            self.games_dealt
        } else {
            self.games_dealt.saturating_sub(1)
        };
        self.lay_boss_curses(content, events);
        let global = self.sources(content, false);
        // "Ehrenvorsitz": the player always leads; the rotation runs on.
        let always_forehand = global
            .iter()
            .any(|s| s.passive.contains(&Effect::AlwaysForehand));
        let forehand = if always_forehand {
            Seat::DECLARER
        } else {
            Seat::from_index(usize::try_from(rotation % 3).unwrap_or(0)).unwrap_or(Seat::DECLARER)
        };
        if rotate {
            self.games_dealt += 1;
            self.series.redeals_in_game = 0;
            self.series.redeals_this_game = 0;
        }
        let (resolved, boss_editions) = self.resolve_editions(content, &global);
        let mut rng = self.deal_rng.fork();
        // The Einführung deals its fixed games (DESIGN 16.1); the generator
        // is forked all the same, so everything drawn after stays put.
        let fixed = self
            .in_tutorial_pub()
            .then(|| {
                content
                    .tutorial
                    .deals
                    .get(self.series.games_played as usize)
            })
            .flatten();
        let game = if let Some(deal) = fixed {
            let kinds = |cards: &[Card]| -> Vec<CardKind> {
                cards.iter().map(|&c| CardKind::Normal(c)).collect()
            };
            let skat = [deal.skat[0], deal.skat[1]].map(CardKind::Normal);
            fixed_game(
                &resolved,
                [
                    &kinds(&deal.player),
                    &kinds(&deal.left),
                    &kinds(&deal.right),
                ],
                skat,
                Seat::from_index(deal.forehand).unwrap_or(Seat::DECLARER),
            )?
        } else {
            Game::deal_editions_keeping(
                &mut rng,
                resolved.clone().map(|r| r.cards),
                forehand,
                keep,
            )?
        };
        self.install_game(content, game, &resolved, boss_editions, events);
        self.peek(&mut rng, events);
        // "Untot": the substitutes come from the game's generator, after
        // everything the deal drew from it.
        let undead_in_play = self.game.as_ref().is_some_and(|active| {
            active
                .game
                .cards_in_play()
                .iter()
                .any(|c| !c.is_blank() && self.has_card_effect(content, *c, &Effect::Undead))
        });
        if let (true, Some(active)) = (undead_in_play, self.game.as_mut()) {
            active.undead_rng = Some(rng.fork());
        }
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
        // Cards the player already sees (`RevealDefenderHand`) are not drawn.
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
        active.note_settled(&game_events);
        events.push(RunEvent::RamschDealt);
        events.extend(game_events.into_iter().map(RunEvent::Game));
        self.after_declaration(content, events);
        self.settle(content, events)
    }

    /// ADR 0015: every seat owns an edition of the 32 cards. Resolve the
    /// edition of each seat for the next game (index = seat; the skat goes
    /// with the player's), plus the boss edition at the table.
    fn resolve_editions(
        &self,
        content: &Content,
        global: &[ActiveSource<'_>],
    ) -> ([crate::content::ResolvedDeck; 3], BossEditions) {
        let boss_editions = self.boss_editions(content);
        let plain = Edition::standard();
        let resolved = Seat::ALL.map(|seat| {
            let owner = self.owner_of_seat(seat);
            let edition = match owner {
                Owner::Player => &self.edition,
                Owner::Boss => boss_editions[0].as_ref().unwrap_or(&plain),
                Owner::BossPartner => boss_editions[1].as_ref().unwrap_or(&plain),
                Owner::Guest => &plain,
            };
            resolve_edition(edition, owner, content, global)
        });
        (resolved, boss_editions)
    }

    /// Install a game dealt from [`Run::resolve_editions`]: the Augen
    /// changes of the copies actually dealt go to the breakdown, the boss
    /// edition and the player's "Durchsichtig" cards to the game.
    fn install_game(
        &mut self,
        content: &Content,
        game: Game<DealtCard>,
        resolved: &[crate::content::ResolvedDeck; 3],
        boss_editions: BossEditions,
        events: &mut Vec<RunEvent>,
    ) {
        let mut point_changes: Vec<(u32, Source, i32)> = Vec::new();
        for card in game.cards_in_play() {
            let seat = game.holder_of(*card).unwrap_or(Seat::DECLARER);
            point_changes.extend(
                resolved[seat.index()]
                    .point_changes
                    .iter()
                    .filter(|(id, _, _)| *id == card.id)
                    .cloned(),
            );
        }
        point_changes.sort_by_key(|(id, _, _)| *id);
        self.set_up_game(content, game, point_changes, events);
        if let Some(active) = self.game.as_mut() {
            let [boss, partner] = boss_editions;
            active.boss_edition = boss;
            active.partner_edition = partner;
        }
        self.mark_defender_cards(events);
    }

    /// "Glasauge" ([`Effect::RevealDefenderCards`]): make random cards of
    /// each defender's hand "Durchsichtig" for this game, drawn with an
    /// own generator from the game's seed and the Einmischen of the game
    /// ([`TRANSPARENT_SEED`]), so the deal and every other draw stay the
    /// same. Candidates in the order of the hand sorted by id; the left
    /// defender first.
    fn mark_defender_cards(&mut self, events: &mut Vec<RunEvent>) {
        let seed = opponents::game_seed(self)
            ^ TRANSPARENT_SEED
            ^ u64::from(self.series.redeals_in_game).wrapping_mul(0x9E37_79B9_7F4A_7C15);
        let Some(active) = self.game.as_mut() else {
            return;
        };
        let n = active.rules.transparent_per_defender;
        let Some(source) = active.rules.transparent_source.clone() else {
            return;
        };
        if n == 0 {
            return;
        }
        let mut rng = Rng::from_seed(seed);
        for seat in Seat::DEFENDERS {
            let mut pool: Vec<DealtCard> = active
                .game
                .hand(seat)
                .iter()
                .filter(|c| !c.revealed)
                .copied()
                .collect();
            pool.sort_by_key(|c| c.id);
            for _ in 0..n {
                if pool.is_empty() {
                    break;
                }
                let len = u64::try_from(pool.len()).unwrap_or(1);
                let pick = usize::try_from(rng.gen_range(0..len)).unwrap_or(0);
                let card = pool.remove(pick.min(pool.len() - 1));
                active.transparent_for_game.push(card.id);
                events.push(RunEvent::CardMadeTransparent {
                    source: source.clone(),
                    card,
                    seat,
                });
            }
        }
    }

    /// Tests: replace the current game by a constructed deal, every card
    /// in the edition of the seat that gets it (the skat in the player's),
    /// with all content effects of a normal deal.
    #[cfg(test)]
    pub(crate) fn deal_fixed(
        &mut self,
        content: &Content,
        hands: [&[CardKind]; 3],
        skat: [CardKind; 2],
        forehand: Seat,
        events: &mut Vec<RunEvent>,
    ) {
        let global = self.sources(content, false);
        let (resolved, boss_editions) = self.resolve_editions(content, &global);
        let game = fixed_game(&resolved, hands, skat, forehand).expect("valid deal");
        self.install_game(content, game, &resolved, boss_editions, events);
    }

    /// Fixture tool: a run saved before ADR 0015 continues with editions
    /// (its game in progress resolved card by card by the holder).
    #[cfg(test)]
    pub(crate) fn adopt_editions(&mut self, content: &Content) {
        let global = self.sources(content, false);
        let (resolved, boss_editions) = self.resolve_editions(content, &global);
        let editions = resolved.map(|r| r.cards);
        let mut next = self.game.as_mut();
        while let Some(active) = next {
            active.game.adopt_editions(editions.clone());
            active.boss_edition.clone_from(&boss_editions[0]);
            active.partner_edition.clone_from(&boss_editions[1]);
            next = active.previous.as_deref_mut();
        }
    }

    /// The editions of the two boss players (ADR 0017): the figures of
    /// the boss whose rule is on (for "Der Alte" the boss of this game's
    /// rule), left and right, in a boss pub only.
    fn boss_editions(&self, content: &Content) -> BossEditions {
        let figures = self
            .series
            .boss
            .as_ref()
            .and_then(|_| self.boss_rule(content))
            .and_then(|b| content.boss(&b))
            .map(|def| def.figures.as_slice())
            .unwrap_or_default();
        // Validated when the content loads.
        [0, 1].map(|i| {
            figures
                .get(i)
                .and_then(|f| Edition::with_mods(&f.edition, content).ok())
        })
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
        // Boss, relics and the rest: the same before and after the game
        // is installed; the card modifications in play follow below.
        let mut sources = self.sources(content, false);
        let has = |wanted: &Effect| sources.iter().any(|s| s.passive.contains(wanted));
        let rules = GameRules {
            thresholds: Thresholds::with_stake(self.stake),
            no_cheating: has(&Effect::NoCheating),
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
            transparent_for_game: Vec::new(),
            boss_rule,
            skat_peeked: false,
            chalked: Vec::new(),
            single_loss: None,
            win_factors: Vec::new(),
            shown_hand: None,
            shown_hands: Vec::new(),
            previous: None,
            named: Vec::new(),
            skat_shown: Vec::new(),
            risen: Vec::new(),
            pending_rise: None,
            undead_rng: None,
            boss_edition: None,
            partner_edition: None,
            player_marked: Vec::new(),
        });
        events.push(RunEvent::GameDealt {
            number: self.series.games_played + 1,
            forehand,
        });
        events.extend(dealt_events.into_iter().map(RunEvent::Game));
        // Passive effects of boss, relics and card mods in play.
        if let Some(active) = &self.game {
            sources.extend(card_mod_sources(self.editions(), content, &active.game));
        }
        for source in &sources {
            for effect in source.passive {
                self.apply_effect(content, &source.source, effect, events);
            }
        }
        self.fire(content, Stage::OnDeal, None, None, events);
        // An encounter's "Kartentrick": the skat shown as with the Kurzer.
        if self.peek_next_skat.is_some() {
            if let Some(active) = self.game.as_mut() {
                if !active.rules.ramsch && active.game.phase() == Phase::Declaring {
                    active.skat_peeked = true;
                    events.push(RunEvent::SkatPeeked {
                        skat: active.game.skat(),
                    });
                }
            }
        }
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
            Declaration::Suit { .. } | Declaration::Grand { .. } => content.run.game_value_parts(
                declaration,
                matadors,
                &outcome,
                &active.rules.thresholds,
            ),
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
        // A lost game counts single since 04.11.2026 (`RUN_LOSS_FACTOR`):
        // `single_loss` ("Magenbitter", the Ramscher) changes nothing any
        // more and adds no line.
        if parts.won {
            lines.extend(win_factor_lines(active));
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
        let discarded_points = if active.game.skat_picked_up() {
            active.game.skat().iter().map(|c| c.card_points).sum()
        } else {
            0
        };
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
            discarded_points,
        });
        if !declaration.is_ramsch() {
            if let Some(wager) = self.wager.take() {
                let won = breakdown.won && (wager.on == BetOn::Win || breakdown.schneider);
                let payout = if won { wager.payout() } else { 0 };
                events.push(RunEvent::WagerSettled {
                    stake: wager.stake,
                    won,
                    payout,
                });
                if payout > 0 {
                    self.coins = self.coins.saturating_add(payout);
                    events.push(RunEvent::CoinsGained {
                        source: Source::Encounter(wager.encounter),
                        amount: i32::try_from(payout).unwrap_or(i32::MAX),
                        coins: self.coins,
                    });
                    self.settle_debt(events);
                }
            }
        }
        self.last_score = Some(breakdown);
        self.after_game(content, events)
    }

    /// Whether a modification on the player's card has this passive
    /// effect: only a card in the player's edition (ADR 0015).
    fn has_card_effect(&self, content: &Content, card: DealtCard, effect: &Effect) -> bool {
        if card.owner != Owner::Player {
            return false;
        }
        self.edition.get(card.id).is_some_and(|card| {
            card.mods
                .iter()
                .filter_map(|m| content.card_mod(m))
                .any(|def| def.passive.contains(effect))
        })
    }

    /// After a game was scored: pub won, pub lost or next deal.
    fn after_game(
        &mut self,
        content: &Content,
        events: &mut Vec<RunEvent>,
    ) -> Result<(), RunError> {
        // A boss pub plays all its games (ADR 0017), and so does the
        // Einführung (DESIGN 16.1: every lesson is dealt); the target
        // counts after the last.
        let plays_all = self.series.is_boss_pub() || self.in_tutorial_pub();
        let early = !plays_all || self.series.games_left() == 0;
        if early && self.series.target_reached() {
            return self.series_won(content, events);
        }
        if self.series.games_left() == 0 {
            self.game = None;
            return self.pub_lost(content, events);
        }
        self.deal(content, true, events)
    }

    /// Whether the run stands between two games of a pub: the dealt game
    /// is untouched (ten cards, skat not picked up, nothing declared and
    /// no card played; a Ramsch before its first card). Klarer, Absacker
    /// and "Kneipe verlassen" go only now.
    #[must_use]
    pub fn between_games(&self) -> bool {
        if self.phase != RunPhase::Game {
            return false;
        }
        let Some(active) = self.game.as_ref() else {
            return false;
        };
        let game = &active.game;
        let undeclared = match game.declaration() {
            None => game.phase() == Phase::Declaring,
            Some(d) => d.is_ramsch(),
        };
        undeclared
            && !game.skat_picked_up()
            && game.tricks().is_empty()
            && game.current_trick().is_empty()
            && !active.awaiting_re
    }

    /// Whether the player may give up the pub now
    /// ([`RunAction::GiveUpPub`]): between two games, once its first game
    /// was played.
    #[must_use]
    pub fn can_give_up(&self) -> bool {
        self.series.games_played >= 1 && self.between_games()
    }

    /// "Kneipe verlassen": the pub ends as lost ([`Run::can_give_up`]).
    fn give_up_pub(
        &mut self,
        content: &Content,
        events: &mut Vec<RunEvent>,
    ) -> Result<(), RunError> {
        if !self.can_give_up() {
            return Err(RunError::WrongPhase);
        }
        self.game = None;
        events.push(RunEvent::PubGivenUp {
            section: self.series.section,
            index: self.series.index,
            score: self.series.score,
        });
        self.pub_lost(content, events)
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
        // A boss pub plays all its games (ADR 0017) and pays as if
        // `boss_reward.games_left_as` were left.
        let paid_left = if self.series.is_boss_pub() {
            games_left.max(content.run.boss_reward.games_left_as)
        } else {
            games_left
        };
        let mut payout = Payout {
            base: economy.series_reward,
            games_left: paid_left * economy.per_game_left,
            interest,
            bonus: 0,
            boss: 0,
        };
        // `payout_factor`: the parts count `payout_factor` times.
        payout.bonus = payout
            .total()
            .saturating_mul(content.payout_factor(self.character) - 1);
        // A won boss pub pays double (ADR 0017).
        if self.series.is_boss_pub() {
            payout.boss = payout.total();
        }
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
        self.settle_debt(events);
        self.fire(content, Stage::OnSeriesWon, None, None, events);
        self.pub_won(content, events)
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
            // The relics of the night (ADR 0020): in the long night always;
            // in a normal run only one an unlock opened.
            .filter(|r| {
                if r.night {
                    self.night.is_some()
                        || (content.relic_unlockable(&r.id) && !self.locked.contains(&r.id))
                } else {
                    !self.locked.contains(&r.id)
                }
            })
            .filter(|r| !self.relics.contains(&r.id) && !self.evening_relics.contains(&r.id))
            .filter(|r| !house_rules_only || r.kind == RelicKind::HouseRule)
            .map(|r| {
                let weight = weights.percent_weight(r.rarity, self.focus_weight(content, r.focus));
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
    /// (weights in `run.ron`). One relic may be "Hehlerware"
    /// ([`Run::draw_fence`]).
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
        let fence = self.draw_fence(content, relics.len());
        Shop {
            fence,
            relics: relics.into_iter().map(Some).collect(),
            consumables: consumables.into_iter().map(Some).collect(),
            rerolls,
            on_credit: false,
            card_offers: Vec::new(),
            painter_used: false,
            cleansed: false,
            kiosk: false,
        }
    }

    /// The owned relics and the evening's house rules, in that order.
    fn held_relics(&self) -> impl Iterator<Item = &RelicId> {
        self.relics.iter().chain(&self.evening_relics)
    }

    /// [`Run::held_relics`] the content knows.
    fn held_relic_defs<'c>(
        &self,
        content: &'c Content,
    ) -> impl Iterator<Item = &'c crate::content::RelicDef> {
        self.held_relics().filter_map(|r| content.relic(r))
    }

    /// The passive effects of [`Run::held_relic_defs`].
    fn held_passives<'c>(&self, content: &'c Content) -> impl Iterator<Item = &'c Effect> {
        self.held_relic_defs(content)
            .flat_map(|def| def.passive.iter())
    }

    /// The passive effects of the owned relics and the evening's house
    /// rules.
    fn relic_passives<'c>(&self, content: &'c Content) -> Vec<&'c Effect> {
        self.held_passives(content).collect()
    }

    /// The credit limit at the Tresen ("Deckel anschreiben"): the largest
    /// granted, 0 without.
    fn credit_limit(&self, content: &Content) -> u32 {
        self.relic_passives(content)
            .into_iter()
            .map(|e| match e {
                Effect::Credit(n) => *n,
                _ => 0,
            })
            .max()
            .unwrap_or(0)
    }

    /// The district the Tresen and the way belong to: where the way
    /// leads, else the current pub's.
    fn way_section(&self) -> u32 {
        self.trip
            .as_ref()
            .map_or(self.series.section, |t| t.section)
    }

    /// Whether the player can pay `price` at the Tresen now: from his
    /// Groschen, or on credit ("Deckel anschreiben", once per district,
    /// down to the limit).
    fn can_pay(&self, content: &Content, price: u32) -> bool {
        if price <= self.coins {
            return true;
        }
        let limit = self.credit_limit(content);
        let open = self.shop.as_ref().is_some_and(|s| s.on_credit)
            || self.credit_section != Some(self.way_section());
        open && limit > 0 && self.debt.saturating_add(price - self.coins) <= limit
    }

    /// Pay `price` at the Tresen, on credit for what the Groschen do not
    /// cover. Check [`Run::can_pay`] first.
    fn pay(&mut self, price: u32, events: &mut Vec<RunEvent>) {
        if price <= self.coins {
            self.coins -= price;
            return;
        }
        let short = price - self.coins;
        self.coins = 0;
        self.debt = self.debt.saturating_add(short);
        self.credit_section = Some(self.way_section());
        if let Some(shop) = self.shop.as_mut() {
            shop.on_credit = true;
        }
        events.push(RunEvent::DebtChanged { debt: self.debt });
    }

    /// The player gave `relic` up (sold, replaced, swapped): its pots are
    /// paid out ([`Run::pay_out_stash`]) and the `OnRelicSold` hooks of the
    /// relics still held fire ("Der Trödler").
    fn relic_given_up(&mut self, content: &Content, relic: &RelicId, events: &mut Vec<RunEvent>) {
        self.pay_out_stash(content, relic, events);
        self.fire(content, Stage::OnRelicSold, None, None, events);
    }

    /// `relic` has just left the player: every pot it stashes into
    /// ([`Effect::StashCoins`]) that no relic still held stashes into is
    /// paid out as it is, without a bonus, and emptied
    /// ([`RunEvent::StashPaidOut`]).
    fn pay_out_stash(&mut self, content: &Content, relic: &RelicId, events: &mut Vec<RunEvent>) {
        let stashes = |id: &RelicId| -> Vec<String> {
            content.relic(id).map_or_else(Vec::new, |def| {
                def.hooks
                    .iter()
                    .flat_map(Hook::then)
                    .chain(&def.passive)
                    .filter_map(|e| match e {
                        Effect::StashCoins { counter, .. } => Some(counter.clone()),
                        _ => None,
                    })
                    .collect()
            })
        };
        let still_held: BTreeSet<String> = self.held_relics().flat_map(stashes).collect();
        let mut amount: u32 = 0;
        for counter in stashes(relic) {
            if still_held.contains(&counter) {
                continue;
            }
            amount = amount.saturating_add(self.counters.remove(&counter).unwrap_or(0));
        }
        if amount > 0 {
            self.coins = self.coins.saturating_add(amount);
            events.push(RunEvent::StashPaidOut {
                relic: relic.clone(),
                amount,
                coins: self.coins,
            });
            self.settle_debt(events);
        }
    }

    /// Income pays the debt first ("Deckel anschreiben").
    fn settle_debt(&mut self, events: &mut Vec<RunEvent>) {
        let paid = self.debt.min(self.coins);
        if paid > 0 {
            self.debt -= paid;
            self.coins -= paid;
            events.push(RunEvent::DebtChanged { debt: self.debt });
        }
    }

    /// Groschen owed at the Tresen ("Deckel anschreiben").
    #[must_use]
    pub const fn debt(&self) -> u32 {
        self.debt
    }

    /// Changes waiting for the next pub.
    #[must_use]
    pub const fn next_pub_plan(&self) -> NextPubPlan {
        self.next_pub
    }

    /// The bet on the next declared game ("Die Wette").
    #[must_use]
    pub const fn wager(&self) -> Option<&Wager> {
        self.wager.as_ref()
    }

    /// Route choices still to come whose exact stops are shown.
    #[must_use]
    pub const fn scout_legs(&self) -> u32 {
        self.scout_legs
    }

    /// Percent off at the Tresen from the owned relics ("Der Wirt"), at
    /// most 90.
    fn shop_discount(&self, content: &Content) -> u32 {
        let total: u32 = self
            .held_passives(content)
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
        if price == 0 {
            return 0;
        }
        // "Nachtzuschlag" (long night, ADR 0020): on top of everything.
        let surcharge = self.night_surcharge(content);
        let discount = self.shop_discount(content);
        if discount == 0 {
            return price.saturating_add(surcharge);
        }
        let paid = (u64::from(price) * u64::from(100 - discount) + 50) / 100;
        u32::try_from(paid)
            .unwrap_or(price)
            .max(1)
            .saturating_add(surcharge)
    }

    /// What a Zinke (a card modification bought for Groschen on a card of
    /// the player's choice) with this price costs on the card with this
    /// instance id: the character's [`crate::content::ZinkenDiscount`] for
    /// matching cards ("Bruderherz": half on Rot), rounded to the nearest
    /// Groschen, at least 1 for a priced Zinke. The Tresen discount is
    /// not part of it ([`Run::shop_price`]). Every offer of a Zinke on a
    /// known card must charge this price.
    #[must_use]
    pub fn zinken_price(&self, content: &Content, card: u32, price: u32) -> u32 {
        let Some(discount) = content
            .character(self.character)
            .and_then(|c| c.rule.zinken_discount)
        else {
            return price;
        };
        let matches = self
            .edition
            .get(card)
            .and_then(|c| c.kind.face())
            .is_some_and(|c| discount.cards.matches(c));
        if price == 0 || !matches {
            return price;
        }
        let percent = 100 - discount.percent.min(100);
        let paid = (u64::from(price) * u64::from(percent) + 50) / 100;
        u32::try_from(paid).unwrap_or(price).max(1)
    }

    /// Cleanse a card of the player's edition ("Frisches Deck",
    /// Werkstatt): every modification falls off. A character with
    /// [`crate::content::CharacterRule::keep_curses`] ("Bruderherz")
    /// cannot cleanse a card that carries a curse ([`RunError::CurseStays`]).
    fn cleanse(&mut self, content: &Content, card: u32) -> Result<DeckChange, RunError> {
        self.check_cleanse(content, card)?;
        Ok(self.edition.cleanse(card)?)
    }

    /// Whether [`Run::cleanse`] would cleanse `card`, and if not, why.
    fn check_cleanse(&self, content: &Content, card: u32) -> Result<(), RunError> {
        let keep = content
            .character(self.character)
            .is_some_and(|c| c.rule.keep_curses);
        if keep && self.card_is_cursed(content, card) {
            return Err(RunError::CurseStays);
        }
        let deck_card = self.edition.get(card).ok_or(DeckError::UnknownCard(card))?;
        if deck_card.mods.is_empty() {
            return Err(DeckError::NothingToCleanse(card).into());
        }
        Ok(())
    }

    /// How many cards of the player's edition carry a curse now (a special
    /// curse such as "Abgestuft" and a curse laid for one pub included):
    /// what "Pechvogel" counts ([`Effect::AddBaseValuePerCursedCard`]).
    #[must_use]
    pub fn cursed_cards(&self, content: &Content) -> u32 {
        let count = self
            .edition
            .cards()
            .iter()
            .filter(|c| self.card_is_cursed(content, c.id))
            .count();
        u32::try_from(count).unwrap_or(u32::MAX)
    }

    /// What an effect that counts cards counts now: [`Run::cursed_cards`]
    /// for `AddBaseValuePerCursedCard`, [`Run::count_cards`] for
    /// `AddBaseValuePerCard` and `AddLevelsPerCard`; 0 for every other
    /// effect (not counted).
    fn count_for(&self, content: &Content, effect: &Effect) -> u32 {
        match effect.counted() {
            Some(Counted::CursedCards) => self.cursed_cards(content),
            Some(Counted::Cards(count)) => self.count_cards(content, count),
            Some(Counted::VoidSuits) => self.void_suits(),
            Some(Counted::DeclarerPointsAbove { above, step }) => {
                let points = self
                    .game
                    .as_ref()
                    .map_or(0, |a| a.game.outcome().declarer_card_points);
                points.saturating_sub(above) / step.max(1)
            }
            Some(Counted::FreeRelicSlots { every }) => {
                self.free_relic_slots(content) / every.max(1)
            }
            None => 0,
        }
    }

    /// Relic slots the player leaves free ("Leerer Stuhl"): the slots of
    /// his character less the relics he holds (house rules of the evening
    /// do not count); 0 without a slot limit.
    #[must_use]
    pub fn free_relic_slots(&self, content: &Content) -> u32 {
        content.relic_slots(self.character).map_or(0, |slots| {
            slots.saturating_sub(u32::try_from(self.relics.len()).unwrap_or(u32::MAX))
        })
    }

    /// The suits besides the trump suit of the declared suit game of which
    /// the player holds no card in the ten he plays ("Kahlschlag"): his hand
    /// and the cards he played, as he holds them (repainted by "Blue
    /// Curaçao", a settled Wandelkarte). Trumps (the Unter, "Bube
    /// ehrenhalber") and blanks belong to no suit. 0 outside a suit game.
    #[must_use]
    pub fn void_suits(&self) -> u32 {
        let Some(game) = self.game.as_ref().map(|a| &a.game) else {
            return 0;
        };
        let Some(GameType::Suit(trump)) = game
            .declaration()
            .filter(|d| !d.is_ramsch())
            .map(Declaration::game)
        else {
            return 0;
        };
        let game_type = GameType::Suit(trump);
        let held = player_cards(game);
        let n = Suit::ALL
            .iter()
            .filter(|&&suit| suit != trump)
            .filter(|&&suit| {
                !held.iter().any(|c| {
                    !crate::rules::is_trump(game_type, *c)
                        && c.face().is_some_and(|f| f.suit == suit)
                })
            })
            .count();
        u32::try_from(n).unwrap_or(0)
    }

    /// How many cards of the player's edition `count` counts now (the
    /// connector relics of the build directions, 04.11.2026): those that
    /// carry a matching modification, narrowed to the trump suit of the
    /// declared suit game and to the cards he holds in the game in
    /// progress if asked.
    #[must_use]
    pub fn count_cards(&self, content: &Content, count: &crate::content::CardCount) -> u32 {
        let game = self.game.as_ref().map(|a| &a.game);
        // The build directions of the relics the player holds.
        let directions: Vec<crate::content::BuildTag> = if count.direction {
            self.held_relic_defs(content)
                .flat_map(|r| r.tags.iter().copied())
                .collect()
        } else {
            Vec::new()
        };
        let trump = game
            .and_then(Game::declaration)
            .filter(|d| !d.is_ramsch())
            .and_then(|d| match d.game() {
                GameType::Suit(suit) => Some(suit),
                _ => None,
            });
        let held = game.map(player_cards).unwrap_or_default();
        let n = self
            .edition
            .cards()
            .iter()
            .filter(|c| {
                // With `in_hand` the card as the player holds it.
                let face = if count.in_hand {
                    let Some(copy) = held.iter().find(|h| h.id == c.id) else {
                        return false;
                    };
                    copy.face()
                } else {
                    c.kind.face()
                };
                if count.trump_suit {
                    let Some(suit) = trump else {
                        return false;
                    };
                    if face.is_none_or(|f| f.suit != suit) {
                        return false;
                    }
                }
                if count.off_trump {
                    let Some(suit) = trump else {
                        return false;
                    };
                    if face.is_none_or(|f| f.suit == suit || f.is_jack()) {
                        return false;
                    }
                }
                if count.no_jacks && face.is_some_and(|f| f.is_jack()) {
                    return false;
                }
                if count.all_cards {
                    return true;
                }
                c.mods
                    .iter()
                    .filter_map(|m| content.card_mod(m))
                    .any(|def| {
                        let named = if count.mods.is_empty() {
                            !def.is_curse()
                        } else {
                            count.mods.contains(&def.id)
                        };
                        named
                            && count.kind.is_none_or(|k| def.kind == k)
                            && (!count.direction || def.tags.iter().any(|t| directions.contains(t)))
                    })
            })
            .count();
        u32::try_from(n).unwrap_or(u32::MAX)
    }

    /// Ids of the cards of the player's edition that carry a Zinke: any
    /// modification that is no curse
    /// ([`crate::content::Condition::TrickWonWithMarkedCard`]).
    #[must_use]
    pub fn marked_cards(&self, content: &Content) -> BTreeSet<u32> {
        self.edition
            .cards()
            .iter()
            .filter(|c| {
                c.mods
                    .iter()
                    .filter_map(|m| content.card_mod(m))
                    .any(|def| !def.is_curse())
            })
            .map(|c| c.id)
            .collect()
    }

    /// Whether a card of the player's edition carries a curse (a special
    /// curse such as "Abgestuft" included).
    #[must_use]
    pub fn card_is_cursed(&self, content: &Content, card: u32) -> bool {
        self.edition.get(card).is_some_and(|c| {
            c.mods
                .iter()
                .any(|m| content.card_mod(m).is_some_and(CardModDef::is_curse))
        })
    }

    /// How the character's Null games are scored (Fixed if the data says
    /// nothing), with the Kurzschluss lowered by
    /// [`Effect::ShortCircuitThreshold`] ("Wackelkontakt": the lowest
    /// value of the character's rule, the relics and the evening rules).
    fn null_scoring(&self, content: &Content) -> NullScoring {
        let mut scoring = content
            .character(self.character)
            .and_then(|c| c.null_scoring.clone())
            .unwrap_or(NullScoring::Fixed);
        if let NullScoring::Formula(formula) = &mut scoring {
            let rule = content
                .character(self.character)
                .map(|c| c.rule.passive.as_slice())
                .unwrap_or_default();
            let relics = self.held_passives(content);
            for effect in rule.iter().chain(relics) {
                if let Effect::ShortCircuitThreshold(points) = effect {
                    formula.short_circuit_points = formula.short_circuit_points.min(*points);
                }
            }
        }
        scoring
    }

    fn shop_mut(&mut self) -> Result<&mut Shop, RunError> {
        self.shop_ref()?;
        self.shop.as_mut().ok_or(RunError::WrongPhase)
    }

    /// The open shop, read only.
    fn shop_ref(&self) -> Result<&Shop, RunError> {
        if self.phase != RunPhase::Shop {
            return Err(RunError::WrongPhase);
        }
        self.shop.as_ref().ok_or(RunError::WrongPhase)
    }

    /// The open shop if it is the Tresen (not a Kiosk).
    fn tresen_mut(&mut self) -> Result<&mut Shop, RunError> {
        self.tresen_ref()?;
        self.shop.as_mut().ok_or(RunError::WrongPhase)
    }

    /// [`Run::tresen_mut`], read only.
    fn tresen_ref(&self) -> Result<&Shop, RunError> {
        let shop = self.shop_ref()?;
        if shop.kiosk {
            return Err(RunError::WrongPhase);
        }
        Ok(shop)
    }

    /// Whether the open shop is a Kiosk.
    #[must_use]
    pub fn at_kiosk(&self) -> bool {
        self.phase == RunPhase::Shop && self.shop.as_ref().is_some_and(|s| s.kiosk)
    }

    /// What a consumable with this price costs in the open shop: the
    /// Tresen price ([`Run::shop_price`]), at a Kiosk the Kiosk's discount
    /// less, at least 1.
    #[must_use]
    pub fn consumable_price(&self, content: &Content, price: u32) -> u32 {
        let paid = self.shop_price(content, price);
        if self.shop.as_ref().is_some_and(|s| s.kiosk) {
            paid.saturating_sub(content.run.route.kiosk.discount).max(1)
        } else {
            paid
        }
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
        let price = self.relic_offer_cost(content, slot, def.price);
        if price > coins && !self.can_pay(content, price) {
            return Err(RunError::NotEnoughCoins);
        }
        if !content.relic_slot_free(self.character, owned) {
            return Err(RunError::NoFreeSlot);
        }
        let fence = self.is_fence(slot);
        if let Some(entry) = self.shop_mut()?.relics.get_mut(slot) {
            *entry = None;
        }
        self.pay(price, events);
        self.relics.push(id.clone());
        events.push(RunEvent::RelicBought { relic: id, price });
        if fence {
            self.trade_curse(content, &[], trade::CurseCause::Fence, events)?;
        }
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
        let price = self.consumable_price(content, def.price);
        let kiosk = self.at_kiosk();
        if price > coins && (kiosk || !self.can_pay(content, price)) {
            return Err(RunError::NotEnoughCoins);
        }
        if owned >= usize::try_from(content.run.consumable_slots).unwrap_or(0) {
            return Err(RunError::NoFreeSlot);
        }
        let shop = self.shop_mut()?;
        if kiosk {
            // One Schnaps per Kiosk: the other offers go.
            shop.consumables.iter_mut().for_each(|c| *c = None);
        } else if let Some(entry) = shop.consumables.get_mut(slot) {
            *entry = None;
        }
        self.pay(price, events);
        self.consumables.push(id.clone());
        events.push(RunEvent::ConsumableBought {
            consumable: id,
            price,
        });
        Ok(())
    }

    /// [`RunAction::ReplaceRelic`]: sell `replace`, then buy `slot`, on a
    /// copy, so a failed purchase leaves the run as it was.
    fn replace_relic(
        &mut self,
        content: &Content,
        slot: usize,
        replace: usize,
        events: &mut Vec<RunEvent>,
    ) -> Result<(), RunError> {
        self.shop_mut()?
            .relics
            .get(slot)
            .cloned()
            .flatten()
            .ok_or(RunError::InvalidSlot)?;
        // On an error the caller drops this copy of the run ([`Run::dispatch`]).
        self.sell_relic(content, replace, events)?;
        self.buy_relic(content, slot, events)
    }

    fn sell_relic(
        &mut self,
        content: &Content,
        index: usize,
        events: &mut Vec<RunEvent>,
    ) -> Result<(), RunError> {
        self.tresen_mut()?;
        let id = self
            .relics
            .get(index)
            .cloned()
            .ok_or(RunError::InvalidSlot)?;
        let price = content.relic(&id).map_or(0, |d| sell_price(d.price));
        self.relics.remove(index);
        self.relic_given_up(content, &id, events);
        self.coins = self.coins.saturating_add(price);
        events.push(RunEvent::RelicSold { relic: id, price });
        self.settle_debt(events);
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
        self.settle_debt(events);
        Ok(())
    }

    /// What "Neu auslosen" costs now: [`crate::content::Economy::reroll_cost`]
    /// after the Tresen discount ("Der Wirt"), nothing with "Der
    /// Stammkunde" ([`Effect::FreeReroll`]).
    #[must_use]
    pub fn reroll_cost(&self, content: &Content) -> u32 {
        let free = self
            .held_passives(content)
            .any(|e| *e == Effect::FreeReroll);
        if free {
            0
        } else {
            self.shop_price(content, content.run.economy.reroll_cost)
        }
    }

    /// "Neu auslosen" (DESIGN 8.6, since 05.11.2026): relics, Schnäpse and
    /// the Auslage drawn again, once per Tresen visit.
    fn reroll(&mut self, content: &Content, events: &mut Vec<RunEvent>) -> Result<(), RunError> {
        let rerolls = self.tresen_mut()?.rerolls;
        if rerolls > 0 {
            return Err(RunError::AlreadyUsed);
        }
        let cost = self.reroll_cost(content);
        if !self.can_pay(content, cost) {
            return Err(RunError::NotEnoughCoins);
        }
        self.pay(cost, events);
        let old = self.shop.take();
        let mut shop = self.draw_offer(content, rerolls + 1);
        if let Some(old) = old {
            shop.on_credit = old.on_credit;
            shop.painter_used = old.painter_used;
            shop.cleansed = old.cleansed;
        }
        events.push(RunEvent::ShopRerolled { cost });
        events.push(RunEvent::ShopOpened {
            relics: shop.relics.clone(),
            consumables: shop.consumables.clone(),
            fence: shop.fence,
        });
        self.shop = Some(shop);
        self.draw_card_offers(content, events);
        Ok(())
    }

    fn leave_shop(
        &mut self,
        content: &Content,
        events: &mut Vec<RunEvent>,
    ) -> Result<(), RunError> {
        self.shop_mut()?;
        events.push(RunEvent::ShopLeft);
        self.shop = None;
        self.continue_trip(content, events)
    }

    // -----------------------------------------------------------------------
    // Consumables
    // -----------------------------------------------------------------------

    /// The consumable in `slot` and its definition, if it may be used at
    /// all now (not locked by a Spelunke malus): what
    /// [`Run::use_consumable`] checks first.
    fn consumable_at<'c>(
        &self,
        content: &'c Content,
        slot: usize,
    ) -> Result<(&ConsumableId, &'c crate::content::ConsumableDef), RunError> {
        let id = self.consumables.get(slot).ok_or(RunError::InvalidSlot)?;
        let def = content
            .consumable(id)
            .ok_or_else(|| RunError::UnknownContent(id.to_string()))?;
        if def.category == crate::content::ConsumableCategory::Schnapps
            && self.phase == RunPhase::Game
            && self.malus(content).is_some_and(|m| m.lock_schnapps)
        {
            return Err(RunError::ConsumablesLocked);
        }
        Ok((id, def))
    }

    fn use_consumable(
        &mut self,
        content: &Content,
        slot: usize,
        target: Option<u32>,
        suit: Option<Suit>,
        events: &mut Vec<RunEvent>,
    ) -> Result<(), RunError> {
        let (id, def) = self.consumable_at(content, slot)?;
        let id = id.clone();
        match &def.action {
            ConsumableAction::CleanseCard => {
                if self.phase != RunPhase::Shop {
                    return Err(RunError::WrongPhase);
                }
                let target = target.ok_or(RunError::TargetRequired)?;
                let change = self.cleanse(content, target)?;
                events.push(RunEvent::DeckChanged(change));
            }
            ConsumableAction::AddCardMod(card_mod) => {
                if self.phase != RunPhase::Shop {
                    return Err(RunError::WrongPhase);
                }
                let target = target.ok_or(RunError::TargetRequired)?;
                let card = self
                    .edition
                    .get(target)
                    .ok_or(DeckError::UnknownCard(target))?;
                if card.kind == CardKind::Blank {
                    return Err(DeckError::BlankCannotBeModified.into());
                }
                let card_mod = match card_mod {
                    Some(m) => m.clone(),
                    None => {
                        let options = self.card_mod_pool_for(content, card);
                        draw_weighted(&mut self.misc_rng, options, 1)
                            .pop()
                            .ok_or(RunError::NoModAvailable)?
                    }
                };
                let value = content.card_mod(&card_mod).map_or(1, |d| d.default_value());
                self.attach_mod(content, target, &card_mod, value, events)?;
            }
            ConsumableAction::ApplyCardMod { card_mod, value } => {
                if self.phase != RunPhase::Shop {
                    return Err(RunError::WrongPhase);
                }
                let target = target.ok_or(RunError::TargetRequired)?;
                self.attach_mod(content, target, card_mod, *value, events)?;
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
            ConsumableAction::RedealNextGame(effect) => {
                self.redeal(content, events)?;
                self.pending_next_game.push((id.clone(), *effect));
            }
            ConsumableAction::RerollRoutes => {
                if self.phase != RunPhase::Route {
                    return Err(RunError::WrongPhase);
                }
                self.reroll_routes(content);
                events.push(RunEvent::RoutesRerolled);
            }
            ConsumableAction::NextPub { redeals, target } => {
                if !matches!(
                    self.phase,
                    RunPhase::Shop | RunPhase::Encounter | RunPhase::Route
                ) {
                    return Err(RunError::WrongPhase);
                }
                self.plan_next_pub(*redeals, *target, events);
            }
            ConsumableAction::ClearAccount => {
                if !self.between_games() {
                    return Err(RunError::WrongPhase);
                }
                if self.series.score >= 0 {
                    return Err(RunError::ConditionNotMet);
                }
                let before = self.series.score;
                self.series.score = 0;
                events.push(RunEvent::AccountCleared { before, score: 0 });
            }
            ConsumableAction::ExtraGames(n) => {
                if !self.between_games() {
                    return Err(RunError::WrongPhase);
                }
                if self.series.is_boss_pub() {
                    return Err(RunError::NotInBossPub);
                }
                self.series.game_limit = self.series.game_limit.saturating_add(*n);
                events.push(RunEvent::GamesAdded {
                    games: *n,
                    game_limit: self.series.game_limit,
                    games_left: self.series.games_left(),
                });
            }
            ConsumableAction::NextGameIfBehind(effect) => {
                self.declaring()?;
                if !self.behind() {
                    return Err(RunError::ConditionNotMet);
                }
                self.pending_next_game.push((id.clone(), *effect));
            }
            ConsumableAction::NextGameCursed(effect) => {
                self.declaring()?;
                if self.trade_curse_targets(content, &[]).is_empty() {
                    return Err(RunError::NoModAvailable);
                }
                self.pending_next_game.push((id.clone(), *effect));
                self.trade_curse(content, &[], trade::CurseCause::Schnapps, events)?;
            }
        }
        self.consumables.remove(slot);
        events.push(RunEvent::ConsumableUsed { consumable: id });
        Ok(())
    }

    /// Whether the pub account is below half its Zielwert ("Eierlikör").
    fn behind(&self) -> bool {
        i128::from(self.series.score).saturating_mul(2) < i128::from(self.series.target)
    }

    /// Add to the plan for the next pub.
    fn plan_next_pub(&mut self, redeals: i32, target: i32, events: &mut Vec<RunEvent>) {
        self.next_pub.redeals = self.next_pub.redeals.saturating_add(redeals);
        self.next_pub.target = self.next_pub.target.saturating_add(target);
        events.push(RunEvent::NextPubPlanned {
            redeals: self.next_pub.redeals,
            target: self.next_pub.target,
        });
    }

    /// "Kurzer": show the player the untouched skat while he still holds
    /// ten cards and has not declared. Once per game.
    fn peek_skat(&mut self, events: &mut Vec<RunEvent>) -> Result<(), RunError> {
        let active = self.declaring()?;
        if active.skat_peeked {
            return Err(RunError::WrongPhase);
        }
        active.skat_peeked = true;
        events.push(RunEvent::SkatPeeked {
            skat: active.game.skat(),
        });
        Ok(())
    }

    /// "Blue Curaçao": repaint the card `id` of the player's hand into `suit`
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

    /// The card modifications a random draw may give one deck card
    /// ([`Run::mod_pool`] without curses): only modifications that
    /// may go on it now ([`crate::content::CardModDef::only_on`], and by
    /// the rules of [`Deck::add_mod`]: not on a cursed card, not past the
    /// limit).
    fn card_mod_pool_for(
        &self,
        content: &Content,
        card: &crate::deck::DeckCard,
    ) -> Vec<(CardModId, u32)> {
        self.mod_candidates(content, false)
            .into_iter()
            .filter(|(id, _)| self.may_attach(content, card.id, id))
            .collect()
    }

    /// Whether `card_mod` with its first value may go on the deck card
    /// `card` now. Tried on a copy of the deck.
    fn may_attach(&self, content: &Content, card: u32, card_mod: &CardModId) -> bool {
        let Some(def) = content.card_mod(card_mod) else {
            return false;
        };
        let fits = self.edition.get(card).is_some_and(|c| def.fits(c.kind));
        fits && self
            .edition
            .probe_card(card)
            .add_mod(content, card, card_mod, def.default_value())
            .is_ok()
    }

    /// Card modifications a random draw may give a card (`curse` false;
    /// `true`: only curses, for draws that announce one), with their draw
    /// weight: for the character, unlocked, no curse unless asked for (DESIGN 7.2:
    /// curses only where announced) and no special card (that comes with a
    /// voucher), in id order. Rarity does not count here; the weight is the
    /// character's weight for the modification's Spielart
    /// ([`Run::focus_weight`]), and modifications of weight 0 are left out.
    fn mod_pool(&self, content: &Content, curse: bool) -> Vec<(CardModId, u32)> {
        // Only what some card of the deck may still take.
        self.mod_candidates(content, curse)
            .into_iter()
            .filter(|(m, _)| {
                self.edition
                    .cards()
                    .iter()
                    .any(|c| self.may_attach(content, c.id, m))
            })
            .collect()
    }

    /// [`Run::mod_pool`] before asking whether some deck card may take a
    /// modification: for one deck card filter these with
    /// [`Run::may_attach`] directly (what fits that card fits some card).
    fn mod_candidates(&self, content: &Content, curse: bool) -> Vec<(CardModId, u32)> {
        content
            .card_mods
            .values()
            .filter(|m| m.is_curse() == curse && !m.is_special())
            .filter(|m| in_pool(&m.characters, self.character))
            .filter(|m| !self.locked_card_mods.contains(&m.id))
            .map(|m| (m.id.clone(), self.focus_weight(content, m.focus)))
            .filter(|&(_, weight)| weight > 0)
            .collect()
    }

    /// [`Effect::AttachModToRandomCard`] ("Der Tankwart", "Der Klempner"):
    /// put `card_mod` with `value` on a random card of the player's edition
    /// that may take it (with `only_unmarked` only cards without any
    /// modification), drawn with a generator of its own from the game's
    /// seed and the modification, so every other draw stays the same.
    /// Candidates in id order. Nothing if none fits.
    fn attach_to_random_card(
        &mut self,
        content: &Content,
        card_mod: &CardModId,
        value: i32,
        only_unmarked: bool,
        events: &mut Vec<RunEvent>,
    ) {
        let mut pool: Vec<u32> = self
            .edition
            .cards()
            .iter()
            .filter(|c| !only_unmarked || c.mods.is_empty())
            .map(|c| c.id)
            .filter(|&id| self.may_attach(content, id, card_mod))
            .collect();
        if pool.is_empty() {
            return;
        }
        pool.sort_unstable();
        let salt = card_mod
            .as_str()
            .bytes()
            .fold(0xCBF2_9CE4_8422_2325_u64, |h, b| {
                (h ^ u64::from(b)).wrapping_mul(0x0100_0000_01B3)
            });
        let mut rng = Rng::from_seed(opponents::game_seed(self) ^ ATTACH_SEED ^ salt);
        let len = u64::try_from(pool.len()).unwrap_or(1);
        let pick = usize::try_from(rng.gen_range(0..len)).unwrap_or(0);
        let card = pool[pick.min(pool.len() - 1)];
        // May only fail on content errors, which validation rules out.
        let _ = self.attach_mod(content, card, card_mod, value, events);
    }

    /// Attach a card modification with `value` to a deck card by the
    /// rules of effects v2 ([`Deck::add_mod`]) and fire the `OnAttach`
    /// hooks of what the card carries then (a curse pays Groschen, not
    /// scaled by the value).
    fn attach_mod(
        &mut self,
        content: &Content,
        card: u32,
        card_mod: &CardModId,
        value: i32,
        events: &mut Vec<RunEvent>,
    ) -> Result<(), RunError> {
        self.attach_mod_paid(content, card, card_mod, value, true, events)
    }

    /// Whether [`Run::attach_mod`] would attach `card_mod` with `value` to
    /// `card`, and if not, why: the same checks, the edition's tried on a
    /// copy of the one card. The `OnAttach` hooks that follow never fail.
    fn check_attach(
        &self,
        content: &Content,
        card: u32,
        card_mod: &CardModId,
        value: i32,
    ) -> Result<(), RunError> {
        if let (Some(def), Some(deck_card)) = (content.card_mod(card_mod), self.edition.get(card)) {
            if deck_card.kind != CardKind::Blank && !def.fits(deck_card.kind) {
                return Err(RunError::NoModAvailable);
            }
        }
        self.edition
            .probe_card(card)
            .add_mod(content, card, card_mod, value)?;
        Ok(())
    }

    /// [`Run::attach_mod`]; with `paid` false the `OnAttach` hooks pay no
    /// Groschen (a curse as the price of a trade, [`trade`], decided on
    /// 03.11.2026: the curse is the price, it pays nothing back).
    fn attach_mod_paid(
        &mut self,
        content: &Content,
        card: u32,
        card_mod: &CardModId,
        value: i32,
        paid: bool,
        events: &mut Vec<RunEvent>,
    ) -> Result<(), RunError> {
        if let (Some(def), Some(deck_card)) = (content.card_mod(card_mod), self.edition.get(card)) {
            if deck_card.kind != CardKind::Blank && !def.fits(deck_card.kind) {
                return Err(RunError::NoModAvailable);
            }
        }
        let change = self.edition.add_mod(content, card, card_mod, value)?;
        // What the card carries now: Augen and Ramponiert are one signed
        // value, so the result may be the other one, or nothing.
        let landed = match &change {
            DeckChange::Modified { card_mod, .. } => Some(card_mod.clone()),
            _ => None,
        };
        events.push(RunEvent::DeckChanged(change));
        let Some((card_mod, def)) = landed.and_then(|m| content.card_mod(&m).map(|d| (m, d)))
        else {
            return Ok(());
        };
        let source = Source::CardMod {
            card_mod: card_mod.clone(),
            card,
        };
        for hook in def.hooks.iter().filter(|h| h.stage() == Stage::OnAttach) {
            let holds = {
                let ctx = HookContext {
                    flags: &self.flags,
                    counters: &self.counters,
                    game: None,
                    played: None,
                    result: None,
                    games_left: self.series.games_left(),
                    transport: None,
                    marked_cards: None,
                    run: Some((self, content)),
                };
                evaluate(hook.when(), &ctx, None)
            };
            // "Beim Erhalt" is not scaled: a curse pays its Groschen once,
            // never as the price of a trade.
            if holds {
                for effect in hook.then() {
                    if !paid && matches!(effect, Effect::GainCoins(_)) {
                        continue;
                    }
                    self.apply_effect(content, &source, effect, events);
                }
            }
        }
        Ok(())
    }

    /// Redeem the vouchers of the cards that lie in the skat after the
    /// discard ("wenn gedrückt", DESIGN 7.2): their `OnDiscard` hooks fire
    /// at once, scaled by their value, whatever becomes of the game, and
    /// the voucher falls off the card. `TurnInto` puts the special card on
    /// the card (from the next game on). A hand game discards nothing; it
    /// redeems one voucher of the hand instead ([`Run::redeem_hand_voucher`]).
    fn redeem_vouchers(&mut self, content: &Content, events: &mut Vec<RunEvent>) {
        let Some(active) = self.game.as_ref() else {
            return;
        };
        if !active.game.skat_picked_up() {
            return;
        }
        let skat: Vec<u32> = active.game.skat().iter().map(|c| c.id).collect();
        for card in skat {
            for (card_mod, value) in self.vouchers_on(content, card) {
                self.redeem_voucher(content, card, &card_mod, value, false, events);
            }
        }
    }

    /// The vouchers on `card` in the player's edition with their values.
    fn vouchers_on(&self, content: &Content, card: u32) -> Vec<(CardModId, i32)> {
        let Some(deck_card) = self.edition.get(card) else {
            return Vec::new();
        };
        deck_card
            .mods
            .iter()
            .filter(|m| {
                content
                    .card_mod(m)
                    .is_some_and(|d| d.kind == crate::content::ModKind::Voucher)
            })
            .map(|m| (m.clone(), deck_card.mod_value(m)))
            .collect()
    }

    /// A hand game is declared (decided by the user on 04.11.2026, DESIGN
    /// 7.2): one voucher on a card of the player's hand, drawn with a
    /// generator of its own from the game's seed, is redeemed at once as if
    /// its card had been discarded. The card stays in the hand. Called just
    /// before the game starts, so a Narrenmarke's Wandelkarte takes its suit
    /// with the rest of the hand ("bei Spielbeginn") and an Untot may rise
    /// in this very game. Nothing happens without a voucher in the hand.
    fn redeem_hand_voucher(&mut self, content: &Content, events: &mut Vec<RunEvent>) {
        let Some(active) = self.game.as_ref() else {
            return;
        };
        let held: Vec<DealtCard> = active
            .game
            .hand(Seat::DECLARER)
            .iter()
            .copied()
            .filter(|c| c.owner == Owner::Player)
            .collect();
        let mut candidates: Vec<(DealtCard, CardModId, i32)> = held
            .iter()
            .flat_map(|&card| {
                self.vouchers_on(content, card.id)
                    .into_iter()
                    .map(move |(m, v)| (card, m, v))
            })
            .collect();
        if candidates.is_empty() {
            return;
        }
        candidates.sort_by_key(|(card, m, _)| (card.id, m.clone()));
        let mut rng = Rng::from_seed(opponents::game_seed(self) ^ HAND_VOUCHER_SEED);
        let len = u64::try_from(candidates.len()).unwrap_or(1);
        let pick = usize::try_from(rng.gen_range(0..len)).unwrap_or(0);
        let (card, card_mod, value) = candidates.swap_remove(pick.min(candidates.len() - 1));
        self.redeem_voucher(content, card.id, &card_mod, value, true, events);
        // The game's copy of the card follows its edition: a Wandelkarte
        // now (or no longer, should Untot have replaced it).
        let wild = self.edition.get(card.id).is_some_and(|d| {
            d.mods
                .iter()
                .filter_map(|m| content.card_mod(m))
                .any(|def| def.passive.contains(&Effect::Wild))
        });
        if wild != card.wild {
            if let Some(active) = self.game.as_mut() {
                let new = DealtCard { wild, ..card };
                // Before the declaration the card is in the hand: cannot fail.
                let _ = active.game.replace_declarer_card(card, new);
            }
        }
    }

    /// Redeem one voucher on `card`: [`RunEvent::VoucherRedeemed`], the
    /// voucher falls off, its `OnDiscard` hooks fire scaled by `value`.
    /// "Der Drückeberger" remembers it for after the pub.
    fn redeem_voucher(
        &mut self,
        content: &Content,
        card: u32,
        card_mod: &CardModId,
        value: i32,
        via_hand: bool,
        events: &mut Vec<RunEvent>,
    ) {
        let Some(def) = content.card_mod(card_mod) else {
            return;
        };
        events.push(RunEvent::VoucherRedeemed {
            card,
            card_mod: card_mod.clone(),
            value,
            via_hand,
        });
        let vouchers_return = content
            .character(self.character)
            .is_some_and(|c| c.rule.vouchers_return);
        if vouchers_return {
            self.series
                .returning_vouchers
                .push((card, card_mod.clone(), value));
        }
        let source = Source::CardMod {
            card_mod: card_mod.clone(),
            card,
        };
        let hooks: Vec<Hook> = def
            .hooks
            .iter()
            .filter(|h| h.stage() == Stage::OnDiscard)
            .cloned()
            .collect();
        if let Some(change) = self.edition.remove_mod(card, card_mod) {
            events.push(RunEvent::DeckChanged(change));
        }
        // "Doppelt gemoppelt": everything a Marke does happens twice.
        let twice = self
            .held_passives(content)
            .any(|e| *e == Effect::RedeemVouchersTwice);
        for _ in 0..if twice { 2 } else { 1 } {
            self.fire_voucher(content, card, &source, &hooks, value, events);
        }
    }

    /// The hooks of one redeemed Marke: its `OnDiscard` hooks, scaled by
    /// its value, then every `OnVoucherRedeemed` hook.
    fn fire_voucher(
        &mut self,
        content: &Content,
        card: u32,
        source: &Source,
        hooks: &[Hook],
        value: i32,
        events: &mut Vec<RunEvent>,
    ) {
        for hook in hooks {
            let holds = {
                let ctx = HookContext {
                    flags: &self.flags,
                    counters: &self.counters,
                    game: None,
                    played: None,
                    result: None,
                    games_left: self.series.games_left(),
                    transport: None,
                    marked_cards: None,
                    run: Some((self, content)),
                };
                evaluate(hook.when(), &ctx, None)
            };
            if !holds {
                continue;
            }
            for effect in hook.then() {
                if let Effect::TurnInto(special) = effect {
                    self.turn_into(content, card, special, events);
                } else {
                    self.apply_effect(content, source, &effect.scaled(value), events);
                }
            }
        }
        // Relics that count redeemed Marken ("Markenheft", "Leergut").
        self.fire(content, Stage::OnVoucherRedeemed, None, None, events);
    }

    /// A redeemed voucher makes `card` the special card `special`
    /// (`TurnInto`). Since 05.11.2026 there is no cap on special cards
    /// (DESIGN 7.2): it always turns the card. A card that is the special
    /// card already stays so (the Drückeberger's returned Totenmarke,
    /// DESIGN 9.4).
    fn turn_into(
        &mut self,
        content: &Content,
        card: u32,
        special: &CardModId,
        events: &mut Vec<RunEvent>,
    ) {
        if let Ok(change) = self.edition.add_mod(content, card, special, 1) {
            events.push(RunEvent::DeckChanged(change));
        }
    }

    // -----------------------------------------------------------------------
    // Encounters (Ereignisse, DESIGN 13)
    // -----------------------------------------------------------------------

    /// Tests: at the route choice, walk to the next pub over the Tresen
    /// alone (as the Tresen came after every series before ADR 0016).
    #[cfg(test)]
    pub(crate) fn walk_to_tresen(&mut self, content: &Content) -> Vec<RunEvent> {
        self.walk(content, true)
    }

    /// Tests: at the route choice, walk straight to the next pub.
    #[cfg(test)]
    pub(crate) fn walk_to_pub(&mut self, content: &Content) -> Vec<RunEvent> {
        self.walk(content, false)
    }

    #[cfg(test)]
    fn walk(&mut self, content: &Content, tresen: bool) -> Vec<RunEvent> {
        let route = self.route.clone().expect("route choice");
        let mut events = Vec::new();
        self.route = None;
        self.trip = Some(tour::Trip {
            section: route.section,
            index: route.index,
            transport: tour::Transport::Walk,
            stops: if tresen {
                vec![tour::RouteStop {
                    category: crate::content::StopCategory::Tresen,
                    encounter: None,
                }]
            } else {
                Vec::new()
            },
            spelunke: None,
        });
        self.continue_trip(content, &mut events).expect("walk");
        events
    }

    /// Tests: draw an encounter by weight from every route category (as
    /// the Tresen's stand-in did before ADR 0016) and open it.
    #[cfg(test)]
    pub(crate) fn open_random_encounter(
        &mut self,
        content: &Content,
        events: &mut Vec<RunEvent>,
    ) -> bool {
        let pool: Vec<(EncounterId, u32)> = content
            .encounters
            .values()
            .filter(|e| in_pool(&e.characters, self.character) && e.weight > 0)
            .map(|e| (e.id.clone(), e.weight))
            .collect();
        let Some(id) = draw_weighted(&mut self.encounter_rng, pool, 1).pop() else {
            return false;
        };
        let opened = self.open_encounter(content, &id, events);
        if opened {
            self.phase = RunPhase::Encounter;
        }
        opened
    }

    /// District and index of the pub after the current one in the
    /// regular order.
    fn next_series_position(&self) -> (u32, u32) {
        if self.series.index + 1 >= PUBS_PER_DISTRICT {
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
        if self.malus(content).is_some_and(|m| m.no_redeal) || self.in_tutorial_pub() {
            return 0;
        }
        let extra = self.game.as_ref().map_or(0, |a| a.rules.extra_redeals);
        let withheld = if self.series.games_played == 0 {
            self.series.redeals_withheld
        } else {
            0
        };
        content
            .redeals_in(self.character, self.series.section)
            .saturating_add(extra)
            .saturating_sub(withheld)
            .saturating_sub(self.night_fewer_redeals(content))
    }

    /// Einmischen left now: what the game in progress still allows of its
    /// own, plus the extras of the series ("Ein Bier", "Kräuter"). 0
    /// without a game. Whether one may be used right now (only with ten
    /// cards before the skat, never in the Ramsch) is
    /// [`Run::accepts`]`(`[`RunAction::Redeal`]`)`.
    #[must_use]
    pub fn redeals_left(&self, content: &Content) -> u32 {
        if self.game.is_none()
            || self.malus(content).is_some_and(|m| m.no_redeal)
            || self.in_tutorial_pub()
        {
            return 0;
        }
        self.redeal_allowance(content)
            .saturating_sub(self.series.redeals_in_game)
            .saturating_add(self.series.extra_redeals)
    }
}

/// A game dealt by hand: every card in the edition of the seat that gets
/// it (the skat in the player's), found by its printed face ("Winkel"
/// changes the kind of a dealt card). Fails if a card is not in a deck.
fn fixed_game(
    resolved: &[crate::content::ResolvedDeck; 3],
    hands: [&[CardKind]; 3],
    skat: [CardKind; 2],
    forehand: Seat,
) -> Result<Game<DealtCard>, RunError> {
    let editions = resolved.clone().map(|r| r.cards);
    let printed = Edition::standard();
    let pick = |seat: usize, kind: CardKind| -> Result<DealtCard, RunError> {
        let missing = || RunError::UnknownContent(format!("card {kind} in the deal"));
        let id = printed
            .cards()
            .iter()
            .find(|c| c.kind == kind)
            .ok_or_else(missing)?
            .id;
        editions[seat]
            .iter()
            .find(|c| c.id == id)
            .copied()
            .ok_or_else(missing)
    };
    let mut dealt: [Vec<DealtCard>; 3] = Default::default();
    for (seat, cards) in hands.iter().enumerate() {
        for &kind in *cards {
            dealt[seat].push(pick(seat, kind)?);
        }
    }
    let skat = [pick(0, skat[0])?, pick(0, skat[1])?];
    Ok(Game::with_editions(dealt, skat, editions, forehand)?)
}

/// Zielwert of a pub: the character's value (from `characters.ron`, or
/// `run.ron` if the character has none), for a boss pub times the boss's
/// target factor (a Spelunke's factor comes on top, [`Run`] applies it).
/// `section` is the district, `index` the pub within it. `None` if the
/// row or pub does not exist.
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
    let base = *targets.get(usize::try_from(index).unwrap_or(usize::MAX))?;
    let factor = boss
        .and_then(|b| content.boss(b))
        .map_or(crate::content::Factor::ONE, |def| {
            def.target_factor_for(character)
        });
    Some(factor.apply(base))
}

/// The line a score effect (Grundwert, Gewinnstufen, Spielwert) adds, with
/// the run counters as they are now and `cards` the cards an effect counts
/// in the player's edition ([`Run::count_for`]: cursed cards, or cards of a
/// [`crate::content::CardCount`]). `None` for every other effect and for a counter (or
/// count of cards) that is still zero.
fn score_line(
    source: &Source,
    effect: &Effect,
    counters: &BTreeMap<String, u32>,
    cards: u32,
) -> Option<ScoreLine> {
    let (lever, change) = effect.score_change(counters, cards)?;
    Some(ScoreLine {
        source: source.clone(),
        lever,
        change,
    })
}

/// The ten cards the player plays in `game`, as he holds them: his hand
/// and the cards he played (completed tricks and the trick on the table),
/// never the skat.
fn player_cards(game: &Game<DealtCard>) -> Vec<DealtCard> {
    let mut cards: Vec<DealtCard> = game.hand(Seat::DECLARER).to_vec();
    for trick in game.tricks() {
        for i in 0..3 {
            if trick.seat_of(i).is_declarer() {
                cards.push(trick.cards[i]);
            }
        }
    }
    let lead = game.trick_lead();
    for (i, card) in game.current_trick().iter().enumerate() {
        if crate::game::seat_at(lead, i).is_declarer() {
            cards.push(*card);
        }
    }
    cards
}

/// The Spielwert factors of a won game ("Eierlikör").
fn win_factor_lines(active: &ActiveGame) -> impl Iterator<Item = ScoreLine> + '_ {
    active.win_factors.iter().map(|(source, factor)| ScoreLine {
        source: source.clone(),
        lever: Lever::GameValue,
        change: Change::Multiply(*factor),
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

/// The value a modification goes on with: the one named, else the first on
/// offer only if it is the named modification (a random draw takes its
/// own first value).
fn mod_value(content: &Content, card_mod: &CardModId, named: Option<i32>) -> i32 {
    named.unwrap_or_else(|| {
        content
            .card_mod(card_mod)
            .map_or(1, CardModDef::default_value)
    })
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

/// Tests only: the content plus the relics and consumables removed by the
/// item review of 02.11.2026 (`run/legacy_*.ron`), so the dormant effects
/// and actions they used stay under test.
#[cfg(test)]
pub(crate) fn legacy_content() -> Content {
    let mut content = Content::load().expect("content");
    let relics: Vec<crate::content::RelicDef> =
        ron::from_str(include_str!("run/legacy_relics.ron")).expect("legacy relics");
    for relic in relics {
        content.relics.insert(relic.id.clone(), relic);
    }
    let items: Vec<crate::content::ConsumableDef> =
        ron::from_str(include_str!("run/legacy_consumables.ron")).expect("legacy consumables");
    for item in items {
        content.consumables.insert(item.id.clone(), item);
    }
    content
}

#[cfg(test)]
mod build_tests;
#[cfg(test)]
mod card_reward_tests;
#[cfg(test)]
mod character_tests;
#[cfg(test)]
mod content_tests;
#[cfg(test)]
mod edition_tests;
#[cfg(test)]
mod effects_v2_tests;
#[cfg(test)]
mod encounters_tests;
#[cfg(test)]
mod factor_tests;
#[cfg(test)]
mod hand_voucher_tests;
#[cfg(test)]
mod items_tests;
#[cfg(test)]
mod m4_tests;
#[cfg(test)]
mod m5_tests;
#[cfg(test)]
mod m7_tests;
#[cfg(test)]
mod marke_encounter_tests;
#[cfg(test)]
mod night_tests;
#[cfg(test)]
mod relic_slots_tests;
#[cfg(test)]
mod relics_v4_tests;
#[cfg(test)]
mod rescue_tests;
#[cfg(test)]
mod schnapps_reward_tests;
#[cfg(test)]
mod schnapps_tests;
#[cfg(test)]
mod settle_tests;
#[cfg(test)]
mod skat_order_tests;
#[cfg(test)]
mod special_tests;
#[cfg(test)]
mod stake_tests;
#[cfg(test)]
mod step_tests;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod tour_tests;
#[cfg(test)]
mod trade_tests;
#[cfg(test)]
mod tutorial_tests;
#[cfg(test)]
mod undead_tests;
#[cfg(test)]
mod wave2_tests;
#[cfg(test)]
mod wave2b_tests;
#[cfg(test)]
mod wave3_tests;
#[cfg(test)]
mod zinken_tests;
#[cfg(test)]
mod zinken_v2_tests;

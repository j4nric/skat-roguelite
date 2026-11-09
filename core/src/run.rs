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
mod deal;
mod error;
mod event;
mod hooks;
mod migrate;
mod scoring;
mod state;
mod table;
mod value_preview;
pub use action::RunAction;
pub use error::{RunError, RunErrorCode};
pub use event::RunEvent;
use hooks::score_line;
pub use migrate::{REFUNDED_CONSUMABLES, RENAMED_CONSUMABLES, RETIRED_BOSSES};
pub use scoring::series_target;
use scoring::{kontra_lines, player_cards, win_factor_lines};
use state::is_zero;
pub use state::{
    ActiveEncounter, ActiveGame, BOSS_SEAT, EncounterOption, GameRules, Kontra, NextPubPlan,
    OptionBlock, PARTNER_SEAT, Payout, RunPhase, RunStats, Series, Shop, Wager, sell_price,
};
pub use table::UNDEAD_PICK;

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use crate::cards::{Card, PlayingCard, Rank, Suit};
use crate::content::{
    BossId, CardModDef, CardModId, Character, ConsumableAction, ConsumableId, Content, Counted,
    Editions, Effect, EncounterId, GameFocus, Hook, HookContext, NextGameEffect, RelicId,
    RelicKind, RunLength, ScoreBreakdown, Source, Stage, evaluate, in_pool,
};
use crate::deck::{CardKind, DealtCard, DeckChange, DeckError, Edition, Owner};
use crate::game::{Game, Phase, Seat};
use crate::rng::Rng;
use crate::rules::{Declaration, GameType, NullScoring};

/// Pubs per district (Stadtteil, ADR 0016): four Kneipen and the
/// Boss-Kneipe last.
pub const PUBS_PER_DISTRICT: u32 = 5;

/// Salt of the draw of the voucher a hand game redeems
/// ([`Run::redeem_hand_voucher`]): its own generator from the game's seed,
/// so the deal and every other draw stay the same.
const HAND_VOUCHER_SEED: u64 = 0x4A4E_D7E1_0000_0004;

/// Salt of the generator that picks the card of
/// [`Effect::AttachModToRandomCard`] (relics v4).
const ATTACH_SEED: u64 = 0xA77A_C4ED_0000_0006;

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

    // -----------------------------------------------------------------------
    // Content hooks
    // -----------------------------------------------------------------------

    // -----------------------------------------------------------------------
    // Series flow
    // -----------------------------------------------------------------------

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

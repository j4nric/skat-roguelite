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
mod card_mods;
mod consumables;
mod counts;
mod deal;
mod error;
mod event;
mod hooks;
mod migrate;
mod scoring;
mod shop;
mod state;
mod table;
mod util;
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
use util::{draw_weighted, mod_value};

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

#[cfg(test)]
use crate::content::in_pool;
use crate::content::{
    BossId, CardModId, Character, ConsumableId, Content, Editions, EncounterId, NextGameEffect,
    RelicId, RunLength, ScoreBreakdown,
};
use crate::deck::{DealtCard, Edition, Owner};
use crate::game::{Phase, Seat};
use crate::rng::Rng;

/// Pubs per district (Stadtteil, ADR 0016): four Kneipen and the
/// Boss-Kneipe last.
pub const PUBS_PER_DISTRICT: u32 = 5;

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

    // -----------------------------------------------------------------------
    // Consumables
    // -----------------------------------------------------------------------

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

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
//! fire in a fixed order: boss, relics in purchase order, card
//! modifications of the cards in play by card id. Conditions see the
//! effects of earlier sources in the same stage.

pub mod bot;

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use serde::{Deserialize, Serialize};

use crate::ai::DefenderHeuristic;
use crate::content::{
    ActiveSource, BossId, Change, ConsumableAction, ConsumableId, Content, Effect, HookContext,
    Lever, Rarity, RelicId, ScoreBreakdown, ScoreLine, Source, Stage, card_mod_sources, evaluate,
    resolve_deck, score_game,
};
use crate::deck::{CardKind, DealtCard, Deck, DeckChange, DeckError};
use crate::game::{Event, Game, GameError, Phase, Seat};
use crate::rng::Rng;
use crate::rules::{Declaration, GameType, Thresholds, ValueParts, game_value_parts};

/// Series per section: two normal ones and the boss series.
pub const SERIES_PER_SECTION: u32 = 3;

/// What the run is waiting for.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RunPhase {
    /// A game of the current series is dealt; see [`ActiveGame`].
    Game,
    /// The Tresen is open between two series.
    Shop,
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
    /// Skat redraws left before the declaration.
    pub redraws_left: u32,
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

/// Groschen paid for a won series.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
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

/// A decision of the player.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RunAction {
    /// Einpassen: skip the dealt game. Costs a game of the limit and a
    /// pass, no points.
    Pass,
    /// Redraw the skat before declaring (house rule "Schieben").
    RedrawSkat,
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
}

/// Something that happened in the run, in order of occurrence.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RunEvent {
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
    Game(Event<DealtCard>),
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
    /// The game was thrown in and dealt again (consumable "Neues Blatt").
    Redealt,
    /// The run was won.
    RunWon,
    /// The run was lost.
    RunLost {
        /// Section of the lost series.
        section: u32,
        /// Series within the section.
        index: u32,
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
    /// No skat redraw available.
    NoRedrawLeft,
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
}

impl fmt::Display for RunError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RunError::Game(e) => write!(f, "game: {e}"),
            RunError::Deck(e) => write!(f, "deck: {e}"),
            RunError::UnknownCard(id) => write!(f, "no card with id {id} in hand"),
            RunError::UnknownContent(id) => write!(f, "unknown content {id}"),
            other => write!(f, "{other:?}"),
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

/// A run. Create with [`Run::new`], drive with [`Run::apply`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Run {
    seed: u64,
    /// Deals: one fork per game, so shopping never changes the cards.
    deal_rng: Rng,
    /// Tresen offers.
    shop_rng: Rng,
    /// Bosses, random card modifications, skat redraws.
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
}

impl Run {
    /// Start a run: the first series begins and its first game is dealt.
    pub fn new(content: &Content, seed: u64) -> Result<(Run, Vec<RunEvent>), RunError> {
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
            },
            coins: content.run.economy.starting_coins,
            deck: Deck::standard(),
            relics: Vec::new(),
            consumables: Vec::new(),
            flags: BTreeSet::new(),
            counters: BTreeMap::new(),
            games_dealt: 0,
            game: None,
            shop: None,
            last_score: None,
        };
        let mut events = Vec::new();
        run.start_series(content, 0, 0, &mut events)?;
        Ok((run, events))
    }

    // -----------------------------------------------------------------------
    // Read access
    // -----------------------------------------------------------------------

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

    /// Breakdown of the last scored game.
    #[must_use]
    pub const fn last_score(&self) -> Option<&ScoreBreakdown> {
        self.last_score.as_ref()
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
            RunAction::RedrawSkat => self.redraw_skat(events),
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
        self.series.games_played += 1;
        events.push(RunEvent::Passed {
            passes_left: self.series.passes_left,
        });
        self.after_game(content, events)
    }

    fn redraw_skat(&mut self, events: &mut Vec<RunEvent>) -> Result<(), RunError> {
        let mut rng = self.misc_rng.fork();
        let active = self.declaring()?;
        if active.rules.redraws_left == 0 {
            return Err(RunError::NoRedrawLeft);
        }
        let game_events = active.game.redraw_skat(&mut rng)?;
        active.rules.redraws_left -= 1;
        events.extend(game_events.into_iter().map(RunEvent::Game));
        Ok(())
    }

    fn declare(
        &mut self,
        content: &Content,
        declaration: Declaration,
        events: &mut Vec<RunEvent>,
    ) -> Result<(), RunError> {
        let active = self.declaring()?;
        if declaration.game == GameType::Null {
            return Err(RunError::NullNotAvailable);
        }
        if active.rules.require_hand && !declaration.hand() {
            return Err(RunError::HandRequired);
        }
        let game_events = active.game.declare(declaration)?;
        events.extend(game_events.into_iter().map(RunEvent::Game));
        self.fire(content, Stage::OnDeclare, None, None, events);
        if let Some(active) = self.game.as_mut() {
            if active.rules.kontra_source.is_some() {
                let kontra_seat = Seat::DEFENDERS.into_iter().find(|&seat| {
                    DefenderHeuristic.says_kontra(active.game.hand(seat), declaration.game)
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

    /// Boss and relics, plus the card modifications in play if a game runs.
    fn sources<'c>(&self, content: &'c Content, with_card_mods: bool) -> Vec<ActiveSource<'c>> {
        let mut sources = Vec::new();
        if let Some(def) = self.series.boss.as_ref().and_then(|b| content.boss(b)) {
            sources.push(ActiveSource {
                source: Source::Boss(def.id.clone()),
                passive: &def.passive,
                hooks: &def.hooks,
                this_card: None,
            });
        }
        for def in self.relics.iter().filter_map(|r| content.relic(r)) {
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
            _ => {}
        }
        let counter_value = |name: &str| self.counters.get(name).copied().unwrap_or(0);
        let from_counter = match effect {
            Effect::AddBaseValueFromCounter(name) => counter_value(name),
            _ => 0,
        };
        let Some(active) = self.game.as_mut() else {
            return;
        };
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
            Effect::SkatRedraw => active.rules.redraws_left = active.rules.redraws_left.max(1),
            Effect::Concede => {
                if active.game.phase() == Phase::Playing && active.game.current_trick().is_empty() {
                    if let Ok(game_events) = active.game.concede() {
                        events.extend(game_events.into_iter().map(RunEvent::Game));
                    }
                }
            }
            Effect::SetCardPoints { .. }
            | Effect::AddCardPoints(_)
            | Effect::RankBonus(_)
            | Effect::RevealHolder
            | Effect::BlankOut(_)
            | Effect::SetFlag(_)
            | Effect::ClearFlag(_)
            | Effect::IncrementCounter(_) => {}
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
        let targets = config
            .targets
            .get(usize::try_from(section).unwrap_or(usize::MAX))
            .ok_or(RunError::WrongPhase)?;
        let target = match index {
            0 => targets.0,
            1 => targets.1,
            _ => targets.2,
        };
        let boss = if index + 1 == SERIES_PER_SECTION {
            let pick = self
                .misc_rng
                .gen_range(0..u64::try_from(config.bosses.len()).unwrap_or(1));
            config
                .bosses
                .get(usize::try_from(pick).unwrap_or(0))
                .cloned()
        } else {
            None
        };
        self.series = Series {
            section,
            index,
            target,
            score: 0,
            games_played: 0,
            game_limit: config.game_limit,
            passes_left: config.passes,
            boss: boss.clone(),
        };
        self.phase = RunPhase::Game;
        self.shop = None;
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
        Ok(())
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
        self.game = Some(ActiveGame {
            game,
            rules: GameRules::default(),
            lines: Vec::new(),
            point_changes,
            kontra: Kontra::None,
            awaiting_re: false,
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
        let parts = game_value_parts(declaration, matadors, &outcome, &active.rules.thresholds);
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
        let breakdown = score_game(
            declaration,
            matadors,
            &outcome,
            &active.rules.thresholds,
            lines,
        );
        self.series.score = self.series.score.saturating_add(breakdown.score);
        self.series.games_played += 1;
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
        events.push(RunEvent::SeriesWon {
            section: self.series.section,
            index: self.series.index,
            payout,
            coins: self.coins,
        });
        self.fire(content, Stage::OnSeriesWon, None, None, events);
        let last_section = u32::try_from(content.run.targets.len()).unwrap_or(1) - 1;
        if self.series.index + 1 == SERIES_PER_SECTION && self.series.section >= last_section {
            self.phase = RunPhase::Won;
            events.push(RunEvent::RunWon);
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

    /// Draw a deterministic offer from the content pool. Relics the player
    /// owns are not offered; no item twice in one offer. Rarer items are
    /// drawn less often (weights in `run.ron`).
    fn draw_offer(&mut self, content: &Content, rerolls: u32) -> Shop {
        let weights = &content.run.shop.rarity_weights;
        let relic_pool: Vec<(RelicId, Rarity)> = content
            .relics
            .values()
            .filter(|r| !self.relics.contains(&r.id))
            .map(|r| (r.id.clone(), r.rarity))
            .collect();
        let consumable_pool: Vec<(ConsumableId, Rarity)> = content
            .consumables
            .values()
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
        if def.price > coins {
            return Err(RunError::NotEnoughCoins);
        }
        if owned >= usize::try_from(content.run.relic_slots).unwrap_or(0) {
            return Err(RunError::NoFreeSlot);
        }
        if let Some(entry) = shop.relics.get_mut(slot) {
            *entry = None;
        }
        self.coins -= def.price;
        self.relics.push(id.clone());
        events.push(RunEvent::RelicBought {
            relic: id,
            price: def.price,
        });
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
        if def.price > coins {
            return Err(RunError::NotEnoughCoins);
        }
        if owned >= usize::try_from(content.run.consumable_slots).unwrap_or(0) {
            return Err(RunError::NoFreeSlot);
        }
        if let Some(entry) = shop.consumables.get_mut(slot) {
            *entry = None;
        }
        self.coins -= def.price;
        self.consumables.push(id.clone());
        events.push(RunEvent::ConsumableBought {
            consumable: id,
            price: def.price,
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
        let price = content.relic(&id).map_or(0, |d| d.price / 2);
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
        let price = content.consumable(&id).map_or(0, |d| d.price / 2);
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
        let cost = content.run.economy.reroll_cost;
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
        let (section, index) = if self.series.index + 1 >= SERIES_PER_SECTION {
            (self.series.section + 1, 0)
        } else {
            (self.series.section, self.series.index + 1)
        };
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
                        let options: Vec<_> = content
                            .card_mods
                            .keys()
                            .filter(|m| !card.mods.contains(m))
                            .cloned()
                            .collect();
                        let pick = self
                            .misc_rng
                            .gen_range(0..u64::try_from(options.len()).unwrap_or(0));
                        options
                            .get(usize::try_from(pick).unwrap_or(usize::MAX))
                            .cloned()
                            .ok_or(RunError::NoModAvailable)?
                    }
                };
                let change = self.deck.add_mod(target, card_mod)?;
                events.push(RunEvent::DeckChanged(change));
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
            ConsumableAction::Redeal => {
                self.declaring()?;
                events.push(RunEvent::Redealt);
                self.deal(content, false, events)?;
            }
        }
        self.consumables.remove(slot);
        events.push(RunEvent::ConsumableUsed { consumable: id });
        Ok(())
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

    /// Skat redraws left in the current game.
    #[must_use]
    pub fn redraws_left(&self) -> u32 {
        self.game.as_ref().map_or(0, |a| a.rules.redraws_left)
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
mod tests;

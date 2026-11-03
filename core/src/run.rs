//! Run state machine (design document 5, 6 and 12).
//!
//! A run is a Kneipentour (ADR 0016) through a city: three districts
//! (Stadtteile, `section` in the code), each with four pubs (Kneipen) and a
//! boss pub. A pub is played as a [`Series`] (the name of the structure
//! before ADR 0016): a target (Zielwert) and a game limit of three games
//! against two Stammgäste. Before declaring, the player may have each game
//! dealt again a few times (einmischen, [`RunAction::Redeal`]) without
//! using up a game; then one of the hands must be played. Won games add
//! their value to the pub's account, lost games subtract twice their value
//! including all relic modifiers. A pub is won the moment the account
//! reaches the target; a boss pub plays all its games and counts after the
//! last (ADR 0017). Missing the target after the last game costs a life
//! (Leben) and the pub is left without reward; the run is lost with the
//! last life, and with any lost boss pub (ADR 0017). A won boss pub pays
//! double, gives a life and a pick of relics ([`tour::BossReward`]). Two
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
mod encounters;
pub mod guests;
pub mod meta;
pub mod opponents;
pub mod player;
mod schnapps_reward;
pub mod tour;
pub mod trade;
pub mod view;
pub mod zinken;

mod settle;

mod value_preview;

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use serde::{Deserialize, Serialize};

use crate::ai::DefenderHeuristic;
use crate::cards::{Card, PlayingCard, Rank, Suit};
use crate::content::{
    ActiveSource, BetOn, BossId, CardFilter, CardModDef, CardModId, Change, Character,
    ConsumableAction, ConsumableId, Content, Editions, Effect, EncounterEffect, EncounterId,
    GameFocus, Hook, HookContext, Lever, NextGameEffect, RelicId, RelicKind, RunLength,
    ScoreBreakdown, ScoreLine, Source, Stage, card_mod_sources, evaluate, in_pool, resolve_edition,
    score_parts,
};
use crate::deck::{CardKind, DealtCard, DeckChange, DeckError, Edition, Owner};
use crate::game::{Event, Game, GameError, Phase, Seat};
use crate::rng::Rng;
use crate::rules::{
    Declaration, GameType, NullScoring, Thresholds, ValueParts, game_value_parts, null_value_parts,
    ramsch_value_parts, suit_strength, trump_strength,
};

/// Pubs per district (Stadtteil, ADR 0016): four Kneipen and the
/// Boss-Kneipe last.
pub const PUBS_PER_DISTRICT: u32 = 5;

/// Old name of [`PUBS_PER_DISTRICT`] (series per section before ADR 0016).
pub const SERIES_PER_SECTION: u32 = PUBS_PER_DISTRICT;

/// Mixed into the game's seed for the card "Drei drücken" draws, so that
/// the draw is independent of the PIMC samples of the same game.
const DRAW_SEED: u64 = 0xD8E1_D8E1_0000_0003;

/// Consumables renamed by the item review of 02.11.2026: old id, new id.
/// A loaded run reads the old ids as the new ones
/// ([`Run::drop_unknown_content`]).
pub const RENAMED_CONSUMABLES: [(&str, &str); 1] = [("chalk", "blue_curacao")];

/// Bosses removed by ADR 0017 and the boss a loaded run meets instead
/// ([`Run::drop_unknown_content`]): a save that drew one keeps its place
/// in the tour and meets the stand-in. None of the stand-ins forces a hand
/// game, so a save waiting at "Skat oder Hand" stays playable as it was.
pub const RETIRED_BOSSES: [(&str, &str); 3] = [
    ("kontra_hans", "twins"),
    ("revolutionary", "twins"),
    ("kibitz_killer", "black_peter"),
];

/// What the run is waiting for.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RunPhase {
    /// A game of the current pub is dealt; see [`ActiveGame`].
    Game,
    /// The Tresen is open, a stop on the way to the next pub.
    Shop,
    /// An encounter (Ereignis) is open, a stop on the way, or the loot of
    /// a won Spelunke; see [`ActiveEncounter`].
    Encounter,
    /// The player chooses the way to the next pub ([`tour::RouteChoice`]).
    Route,
    /// A boss pub was won: the player picks one of the relics on offer or
    /// none ([`tour::BossReward`], ADR 0017).
    BossReward,
    /// The last boss pub was won.
    Won,
    /// The last life was lost.
    Lost,
}

/// The current pub (Kneipe; the type keeps its name from before ADR 0016,
/// when a section had three series).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Series {
    /// District (Stadtteil; Abschnitt before ADR 0016), counted from 0.
    pub section: u32,
    /// Pub within the district, from 0; the last
    /// ([`PUBS_PER_DISTRICT`] − 1) is the boss pub.
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
    /// Whether a Ramsch ("Der Ramscher") was dealt in this pub. Since ADR
    /// 0017 every game under his rule is one; bookkeeping only.
    #[serde(default)]
    pub ramsch_played: bool,
    /// Whether "Drei drücken" drew its card in this series (once per
    /// series, at the first pickup of the skat).
    #[serde(default)]
    pub discard_three_used: bool,
    /// The Spelunke this pub is, if it replaced a normal pub (ADR 0016).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub spelunke: Option<tour::Spelunke>,
    /// Cards of the player's edition cursed for this pub only (Spelunke
    /// malus), with the curse that landed; taken off when the pub ends.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub temp_curses: Vec<(u32, CardModId)>,
    /// Cards of the player's edition a boss cursed for this pub ("Die
    /// Oma", [`Effect::CurseForPub`]), each as it was before; put back
    /// when the pub ends.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub boss_curses: Vec<crate::deck::DeckCard>,
    /// Whether the boss's curses of this pub were laid (once per pub).
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub boss_cursed: bool,
    /// Vouchers redeemed in this pub (card, voucher, value) that come
    /// back when it ends ("Der Drückeberger",
    /// [`crate::content::CharacterRule::vouchers_return`]).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub returning_vouchers: Vec<(u32, CardModId, i32)>,
    /// Einmischen the first game of this pub allows fewer ("Der
    /// Skatlehrer", [`NextPubPlan`]).
    #[serde(default, skip_serializing_if = "is_zero")]
    pub redeals_withheld: u32,
}

/// Changes waiting for the next pub ("Wegbier", "Der Skatlehrer",
/// "Kiebitz am Fenster"), applied when it starts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct NextPubPlan {
    /// Einmischen more for the whole pub; fewer if negative (then the
    /// first game allows that many fewer).
    pub redeals: i32,
    /// Change of the Zielwert (never below 1).
    pub target: i32,
}

impl NextPubPlan {
    /// Whether nothing is planned.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.redeals == 0 && self.target == 0
    }

    /// A Zielwert with the planned change, at least 1.
    #[must_use]
    pub fn apply_to(&self, target: u64) -> u64 {
        let changed = i64::try_from(target)
            .unwrap_or(i64::MAX)
            .saturating_add(i64::from(self.target));
        u64::try_from(changed.max(1)).unwrap_or(1)
    }
}

/// A bet on the next declared game ("Die Wette").
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Wager {
    /// The encounter that took it.
    pub encounter: EncounterId,
    /// Groschen staked (already paid).
    pub stake: u32,
    /// Saves from before 03.11.2026: paid back this many times if won
    /// with Schneider (`percent` is 0 then).
    #[serde(default)]
    pub factor: u32,
    /// What it is on.
    #[serde(default)]
    pub on: BetOn,
    /// Paid back in percent of the stake if it goes as bet.
    #[serde(default)]
    pub percent: u32,
}

impl Wager {
    /// Groschen paid back if it goes as bet.
    #[must_use]
    pub fn payout(&self) -> u32 {
        if self.percent == 0 {
            return self.stake.saturating_mul(self.factor);
        }
        let paid = u64::from(self.stake) * u64::from(self.percent) / 100;
        u32::try_from(paid).unwrap_or(u32::MAX)
    }
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

    /// Whether this is the boss pub of its district.
    #[must_use]
    pub const fn is_boss_pub(&self) -> bool {
        self.index + 1 == PUBS_PER_DISTRICT
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
    /// Boss rules that cheat are off ("Der Schiedsrichter").
    pub no_cheating: bool,
    /// Source of the Schneider bonus in the thresholds ("Stichfest").
    pub schneider_bonus_source: Option<Source>,
    /// Who lets the player draw a third card with the skat and discard
    /// three ("Drei drücken").
    pub discard_three: Option<Source>,
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
    /// Cards repainted for this game ("Blue Curaçao"): instance id and printed
    /// suit.
    #[serde(default)]
    pub chalked: Vec<(u32, Suit)>,
    /// Lost, this game counts once instead of twice ("Magenbitter"): who
    /// granted it.
    #[serde(default)]
    pub single_loss: Option<Source>,
    /// Won, this game counts that many times more ("Eierlikör",
    /// [`NextGameEffect::MultiplyWin`]): who granted it and the factor.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub win_factors: Vec<(Source, u32)>,
    /// The defender whose whole hand the player sees (`RevealDefenderHand`):
    /// his cards are in `peeked` with this seat.
    #[serde(default)]
    pub shown_hand: Option<Seat>,
    /// The deal before the last Einmischen of this game, kept for "Reue"
    /// ([`ConsumableAction::UndoRedeal`]); its own `previous` is always
    /// empty. `None` before the first Einmischen and after an undo.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub previous: Option<Box<ActiveGame>>,
    /// Wandelkarten of the player settled as this game started (DESIGN
    /// 7.2, [`Event::WildCardSettled`]): instance id and the printed suit
    /// (the card has its settled suit now). Saves from before 03.11.2026:
    /// the Wandelkarten the player named as he played them.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub named: Vec<(u32, Suit)>,
    /// Cards of the untouched skat the player was shown after declaring a
    /// hand game ("Spickzettel").
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub skat_shown: Vec<DealtCard>,
    /// "Untot" cards that rose in this game (instance ids): each rises
    /// once per game ([`Effect::Undead`]).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub risen: Vec<u32>,
    /// Generator that draws the substitute of a rising "Untot" card, forked
    /// from the game's own generator at the deal if an "Untot" card is in
    /// play. Saves from before it derive one from the run's seed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub undead_rng: Option<Rng>,
    /// The edition of the boss player on the left ([`BOSS_SEAT`]): the
    /// first figure of the boss whose rule is on (ADR 0015, ADR 0017,
    /// DESIGN 11). `None` outside a boss pub.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub boss_edition: Option<Edition>,
    /// The edition of the boss player on the right ([`PARTNER_SEAT`]):
    /// the second figure (ADR 0017). `None` outside a boss pub and in
    /// saves from before, when a Stammgast sat there.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub partner_edition: Option<Edition>,
    /// Cards the player's edition marks "Durchsichtig" in this game (ids):
    /// shown with their holder and his
    /// edition also while a defender holds them.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub player_marked: Vec<u32>,
}

impl ActiveGame {
    /// Remember the printed suit of every Wandelkarte the game settled in
    /// `events` ([`Event::WildCardSettled`]), for [`Self::named`].
    fn note_settled(&mut self, events: &[Event<DealtCard>]) {
        for event in events {
            if let Event::WildCardSettled { old, .. } = event {
                if let Some(face) = old.face() {
                    if !self.named.iter().any(|(id, _)| *id == old.id) {
                        self.named.push((old.id, face.suit));
                    }
                }
            }
        }
    }
}

/// The seat of the first boss player in a boss pub: left of the player
/// (DESIGN 10.4).
pub const BOSS_SEAT: Seat = Seat::DEFENDERS[0];

/// The seat of the second boss player in a boss pub (ADR 0017): right of
/// the player.
pub const PARTNER_SEAT: Seat = Seat::DEFENDERS[1];

/// The editions of the two boss players of a game, left and right; both
/// `None` outside a boss pub.
type BossEditions = [Option<Edition>; 2];

/// Whether a counter is zero (fields left out of saves while unused).
#[allow(clippy::trivially_copy_pass_by_ref)]
const fn is_zero(n: &u32) -> bool {
    *n == 0
}

/// An option of an open encounter: its price and its effects with every
/// draw filled in (DESIGN 13, "Wege und Begegnungen v2"). Title and text
/// come from the content ([`crate::content::EncounterOptionDef`], same
/// index).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EncounterOption {
    /// Groschen it costs (the stake of a bet or a throw of the dice).
    pub cost: u32,
    /// What it does, in order.
    pub effects: Vec<EncounterEffect>,
    /// Offered only when no other option can be taken.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub fallback: bool,
}

impl EncounterOption {
    /// Whether it needs a card of the player's edition as target.
    #[must_use]
    pub fn needs_card(&self) -> bool {
        self.effects.iter().any(EncounterEffect::needs_card)
    }

    /// Whether it needs an owned relic (its index) as target.
    #[must_use]
    pub fn needs_relic(&self) -> bool {
        self.effects.iter().any(EncounterEffect::needs_relic)
    }
}

/// Why an option of the open encounter cannot be taken now (shown with
/// the option; stable names for the frontend's texts).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OptionBlock {
    /// Not enough Groschen.
    NotEnoughCoins,
    /// Nothing was left to draw (no house rule, relic, Schnaps or
    /// modification for the character).
    NothingLeft,
    /// The edition holds as many special cards of this kind as it may
    /// (DESIGN 7.2).
    SpecialCardCap,
    /// No card of the edition can take it.
    NoFittingCard,
    /// No relic owned.
    NoRelic,
    /// Every relic slot is taken.
    RelicSlotsFull,
    /// No life is missing.
    LivesFull,
    /// A bet is open already.
    BetOpen,
    /// A fallback: another option can be taken.
    NotNeeded,
}

impl OptionBlock {
    /// The error [`RunAction::ChooseEncounter`] returns for it.
    #[must_use]
    pub const fn error(self) -> RunError {
        match self {
            OptionBlock::NotEnoughCoins => RunError::NotEnoughCoins,
            OptionBlock::RelicSlotsFull => RunError::NoFreeSlot,
            OptionBlock::LivesFull => RunError::LivesFull,
            OptionBlock::NothingLeft | OptionBlock::SpecialCardCap | OptionBlock::NoFittingCard => {
                RunError::NoModAvailable
            }
            OptionBlock::NoRelic => RunError::InvalidSlot,
            OptionBlock::BetOpen | OptionBlock::NotNeeded => RunError::ConditionNotMet,
        }
    }
}

/// The encounter (Ereignis) on the way, while open.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ActiveEncounter {
    /// Which encounter.
    pub id: EncounterId,
    /// Its options, drawn when it opened; one must be taken. Empty only
    /// for an encounter of a save from before 03.11.2026, which is opened
    /// anew when loaded ([`Run::drop_unknown_content`]).
    pub options: Vec<EncounterOption>,
    /// The option taken while the Kartenmaler waits.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub chosen: Option<usize>,
}

impl<'de> Deserialize<'de> for ActiveEncounter {
    /// Reads the current form; an encounter of the old form (options with
    /// a `choice`, before 03.11.2026) keeps only its id.
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        struct Current {
            id: EncounterId,
            options: Vec<EncounterOption>,
            #[serde(default)]
            chosen: Option<usize>,
        }
        #[derive(Deserialize)]
        struct Old {
            id: EncounterId,
        }
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum Either {
            Current(Current),
            Old(Old),
        }
        Ok(match Either::deserialize(d)? {
            Either::Current(c) => ActiveEncounter {
                id: c.id,
                options: c.options,
                chosen: c.chosen,
            },
            Either::Old(o) => ActiveEncounter {
                id: o.id,
                options: Vec::new(),
                chosen: None,
            },
        })
    }
}

/// The Tresen between two series. Bought items leave an empty slot.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Shop {
    /// Relics on offer.
    pub relics: Vec<Option<RelicId>>,
    /// Consumables on offer.
    pub consumables: Vec<Option<ConsumableId>>,
    /// Neuauslagen bought during this visit.
    pub rerolls: u32,
    /// This visit sold on credit ("Deckel anschreiben"): it may go on
    /// down to the limit.
    #[serde(default)]
    pub on_credit: bool,
    /// The Auslage (Zinken, DESIGN 8.6): concrete modifications on concrete
    /// cards. Empty in saves from before.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub card_offers: Vec<Option<zinken::CardOffer>>,
    /// "Neu auslosen" of the Auslage bought during this visit.
    #[serde(default, skip_serializing_if = "is_zero")]
    pub card_rerolls: u32,
    /// The Kartenmaler worked at this visit (once per Tresen).
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub painter_used: bool,
    /// "Karte reinigen" was used at this visit (once per Tresen).
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub cleansed: bool,
    /// A Kiosk, not the Tresen (03.11.2026): only Schnäpse, each cheaper
    /// ([`crate::content::KioskConfig`]), one may be bought; no Neuauslage,
    /// no Zinken, no selling of relics, no credit.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub kiosk: bool,
    /// The slot of the relic offered as "Hehlerware" (curses as trade,
    /// DESIGN 8.6): half price, and a curse on a random card when bought.
    /// `None` in saves from before.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fence: Option<usize>,
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
    /// Extra from the character's rule (`payout_factor`: the three parts
    /// once more); 0 for most characters.
    pub bonus: u32,
    /// A won boss pub pays double (ADR 0017): everything above once more;
    /// 0 elsewhere.
    pub boss: u32,
}

impl Payout {
    /// Sum of all parts.
    #[must_use]
    pub const fn total(&self) -> u32 {
        self.base + self.games_left + self.interest + self.bonus + self.boss
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
    #[serde(default, skip_serializing_if = "is_zero")]
    boss: u32,
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
            boss: p.boss,
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
            boss: p.boss,
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
    /// Pubs won (series before ADR 0016).
    pub series_won: u32,
    /// Highest Spielwert of a won game.
    pub best_game: u64,
    /// Pubs lost (each cost a life).
    #[serde(default)]
    pub pubs_lost: u32,
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
    /// 0011). Then two cards are discarded ([`RunAction::Discard`]; with
    /// "Drei drücken" a card of the left defender joins and three go,
    /// [`RunAction::DiscardThree`]) and the game is declared
    /// ([`RunAction::Declare`]). Einmischen is no longer possible after it.
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
    /// Discard three cards (instance ids) after picking up the skat with
    /// "Drei drücken" ([`view::GameView::discard_count`] is 3): the lowest
    /// goes back to the defender the third card came from
    /// ([`crate::rules::returned_discard`]), the other two into the skat.
    DiscardThree([u32; 3]),
    /// Play a card (instance id).
    Play(u32),
    /// Use a consumable from the given slot, with a target card and a suit
    /// if needed.
    UseConsumable {
        /// Slot of the consumable.
        slot: usize,
        /// Target card instance id: a deck card for deck changes, a card of
        /// the hand for "Blue Curaçao".
        target: Option<u32>,
        /// The new suit of the target ("Blue Curaçao").
        #[serde(default)]
        suit: Option<Suit>,
    },
    /// "Spelunkenkenner": at the route choice, pick the malus of the
    /// Spelunke on route `option` from its offers
    /// ([`tour::Spelunke::malus_options`]).
    ChooseMalus {
        /// Index of the route.
        option: usize,
        /// Index into the malus offers.
        malus: usize,
    },
    /// Buy the relic in this Tresen slot.
    BuyRelic(usize),
    /// Buy the consumable in this Tresen slot.
    BuyConsumable(usize),
    /// Sell an owned relic (index into the owned relics) for half its price.
    SellRelic(usize),
    /// Sell an owned consumable for half its price.
    SellConsumable(usize),
    /// Neuauslage: draw a new offer of relics and consumables (the
    /// Auslage stays).
    Reroll,
    /// Buy the offer of the Auslage in this slot (Zinken, DESIGN 8.6): its
    /// modification with its value goes on its card. The same modification
    /// on the card is raised, another one replaced.
    BuyCardOffer(usize),
    /// "Neu auslosen": draw the Auslage again. The price rises with every
    /// further time at the same Tresen.
    RerollCardOffers,
    /// Hire the Kartenmaler (once per Tresen) for this card of the player's
    /// edition (instance id): he draws modifications that fit it, the
    /// player takes one ([`RunAction::ChoosePaint`]).
    HirePainter(u32),
    /// Take this option of the Kartenmaler's choice ([`Run::painter`]), at
    /// the Tresen or the Werkstatt.
    ChoosePaint(usize),
    /// "Karte reinigen" (once per Tresen): every modification falls off
    /// this card of the player's edition (instance id), curses included.
    CleanseCard(u32),
    /// Leave the Tresen and start the next series.
    LeaveShop,
    /// "Kneipe verlassen": give up the current pub between two games, from
    /// its second game on ([`view::RunView::can_give_up`]). It counts as
    /// lost: a life goes ("Schutzengel" may spare it), nothing is paid,
    /// `OnSeriesWon` does not fire; in a boss pub the run ends (ADR 0017).
    GiveUpPub,
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
    /// option needs one; then the way goes on (next stop, the route
    /// choice or the next pub).
    ChooseEncounter {
        /// Index into [`ActiveEncounter::options`].
        option: usize,
        /// Deck card id for [`EncounterChoice::MarkChosenCard`] and
        /// [`EncounterChoice::CleanseChosenCard`].
        target: Option<u32>,
    },
    /// Take the route at this index of the open route choice
    /// ([`tour::RouteChoice::options`]).
    ChooseRoute(usize),
    /// The relic pick of a won boss pub (ADR 0017): take the relic at
    /// `offer` of [`tour::BossReward::relics`], or none (`None`). With
    /// `replace` the owned relic at that index is sold for half its price
    /// first and the new one takes a slot (needed when every slot is
    /// full). Then the route choice.
    ChooseBossRelic {
        /// Index into the relics on offer; `None` takes none.
        offer: Option<usize>,
        /// Index of an owned relic to sell for it.
        #[serde(default)]
        replace: Option<usize>,
    },
    /// The Schnaps of a won pub while every Schnaps slot is full
    /// ([`view::RunView::schnapps_reward`], DESIGN 8.9): with `give_up`
    /// it takes the slot of that owned Schnaps, which is gone without
    /// Groschen; without, it is sold for half its price. Nothing else
    /// goes before this choice ([`RunError::SchnappsRewardPending`]).
    ChooseSchnappsReward {
        /// Slot of the owned Schnaps to give up for it; `None` sells it.
        #[serde(default)]
        give_up: Option<usize>,
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
    /// A new pub started (the event keeps its name from the series).
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
        /// The malus of a Spelunke.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        malus: Option<crate::content::MalusId>,
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
    /// A card effect gave a seat extra Augen ("Erster Stich": the Augen of
    /// the first trick once more, DESIGN 7.2).
    BonusPoints {
        /// Who did it.
        source: Source,
        /// The seat that gets them.
        seat: Seat,
        /// Extra Augen.
        card_points: u32,
    },
    /// Content paid (or took) Groschen (a curse when attached, a voucher).
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
        /// Augen of the cards the player discarded into the skat (with
        /// their modifications); 0 in a hand game.
        #[serde(default, skip_serializing_if = "is_zero")]
        discarded_points: u32,
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
    /// A Kiosk opened on the way (03.11.2026).
    KioskOpened {
        /// Schnäpse on offer.
        consumables: Vec<Option<ConsumableId>>,
    },
    /// The Tresen opened.
    ShopOpened {
        /// Relics on offer.
        relics: Vec<Option<RelicId>>,
        /// Consumables on offer.
        consumables: Vec<Option<ConsumableId>>,
        /// The slot of the relic offered as "Hehlerware", if any.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        fence: Option<usize>,
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
    /// The Auslage was drawn (Zinken, DESIGN 8.6): when the Tresen opens
    /// and after "Neu auslosen".
    CardOffersDrawn {
        /// The offers, by slot.
        offers: Vec<Option<zinken::CardOffer>>,
    },
    /// An offer of the Auslage was bought. The `DeckChanged` that follows
    /// shows the card.
    CardOfferBought {
        /// Slot of the Auslage.
        slot: usize,
        /// The card (instance id).
        card: u32,
        /// The modification.
        card_mod: CardModId,
        /// Its value.
        value: i32,
        /// Price paid.
        price: u32,
    },
    /// "Neu auslosen" was paid; `CardOffersDrawn` follows.
    CardOffersRerolled {
        /// Price paid.
        cost: u32,
    },
    /// The Kartenmaler took a card and drew the modifications to choose
    /// from (at the Tresen, or free at the Werkstatt).
    PainterHired {
        /// The card (instance id).
        card: u32,
        /// The modifications with their values.
        options: Vec<(CardModId, i32)>,
        /// Price paid.
        price: u32,
    },
    /// The player took a modification of the Kartenmaler. The
    /// `DeckChanged` that follows shows the card.
    PainterChosen {
        /// The card (instance id).
        card: u32,
        /// The modification.
        card_mod: CardModId,
        /// Its value.
        value: i32,
    },
    /// A trade put a permanent curse on a card of the player's edition
    /// (curses as trade, DESIGN 8.6): a "Schnäppchen" of the Auslage, a
    /// relic of the "Hehlerware", the Schnaps "Fusel". The `DeckChanged`
    /// that follows shows the card; a curse that pays when attached says
    /// so after it.
    TradeCurse {
        /// The card (instance id).
        card: u32,
        /// The curse.
        curse: CardModId,
        /// What the curse was the price of.
        cause: trade::CurseCause,
    },
    /// "Karte reinigen" at the Tresen was paid. The `DeckChanged` that
    /// follows shows the card.
    CardCleansed {
        /// The card (instance id).
        card: u32,
        /// Price paid.
        price: u32,
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
    /// A voucher was redeemed: its card went into the skat by discarding
    /// ("wenn gedrückt", DESIGN 7.2). Its effects follow as their own
    /// events (`CoinsGained`, `DeckChanged`), then it falls off the card.
    VoucherRedeemed {
        /// Instance id of the card.
        card: u32,
        /// The voucher.
        card_mod: CardModId,
        /// Its value.
        value: i32,
    },
    /// A Totenmarke or Narrenmarke was redeemed while another card of the
    /// edition is that special card already (at most one Untot and one
    /// Wandelkarte, DESIGN 7.2): the card stays as it is, the voucher pays
    /// Groschen instead (`CoinsGained` follows).
    VoucherCapped {
        /// Instance id of the card that went into the skat.
        card: u32,
        /// The voucher.
        card_mod: CardModId,
        /// The special card it would have made.
        special: CardModId,
        /// The card that is that special card.
        holder: u32,
        /// Groschen paid instead.
        coins: u32,
    },
    /// "Der Drückeberger" (DESIGN 9.4): a voucher redeemed in the pub
    /// that just ended is back on its card.
    VoucherReturned {
        /// Instance id of the card.
        card: u32,
        /// The voucher.
        card_mod: CardModId,
        /// Its value.
        value: i32,
    },
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
        /// Index of the option taken.
        option: usize,
    },
    /// The dice fell at the Würfelbude.
    DiceThrown {
        /// Groschen staked (already paid).
        stake: u32,
        /// Won.
        won: bool,
        /// Groschen paid out (0 if lost).
        payout: u32,
    },
    /// The player will see both skat cards in the next game.
    SkatPeekPlanned,
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
    /// After a hand declaration the player was shown a card of the
    /// untouched skat ("Spickzettel").
    SkatCardShown {
        /// Who showed it.
        source: Source,
        /// The skat card.
        card: DealtCard,
    },
    /// The route choice to the next pub opened (ADR 0016).
    RouteOpened {
        /// District of the next pub.
        section: u32,
        /// The next pub within it.
        index: u32,
    },
    /// The player took a route.
    RouteChosen {
        /// Index of the route.
        option: usize,
        /// How he goes.
        transport: tour::Transport,
        /// Whether it leads into a Spelunke.
        spelunke: bool,
    },
    /// A pub ended below its target after its last game (ADR 0016).
    PubLost {
        /// District.
        section: u32,
        /// Pub within the district.
        index: u32,
        /// The boss of a boss pub (it is played again).
        #[serde(default)]
        boss: Option<BossId>,
    },
    /// The player gave up the pub ("Kneipe verlassen",
    /// [`RunAction::GiveUpPub`]). It counts as lost: `PubLost` and what a
    /// lost pub brings follow (a life, or the end of the run in a boss
    /// pub).
    PubGivenUp {
        /// District.
        section: u32,
        /// Pub within the district.
        index: u32,
        /// Account when given up.
        score: i64,
    },
    /// A pub account below 0 was set to 0 ("Klarer").
    AccountCleared {
        /// Account before.
        before: i64,
        /// Account now (0).
        score: i64,
    },
    /// The Spiellimit of the pub rose ("Absacker").
    GamesAdded {
        /// Games added.
        games: u32,
        /// Spiellimit now.
        game_limit: u32,
        /// Games left now.
        games_left: u32,
    },
    /// Lives changed (ADR 0016).
    LivesChanged {
        /// Gained (positive) or lost (negative).
        change: i32,
        /// Lives now.
        lives: u32,
    },
    /// A relic was gained without paying (loot of a Spelunke).
    RelicGained {
        /// The relic.
        relic: RelicId,
    },
    /// The Groschen owed at the Tresen changed ("Deckel anschreiben"):
    /// credit taken, or paid back from income.
    DebtChanged {
        /// Groschen owed now.
        debt: u32,
    },
    /// A lost pub cost no life ("Schutzengel").
    LifeSpared {
        /// Who spared it.
        source: Source,
    },
    /// Content put Groschen into a pot ("Stammtischkasse").
    CoinsStashed {
        /// Who did it.
        source: Source,
        /// Groschen moved.
        amount: u32,
        /// Groschen in the pot now.
        pot: u32,
        /// Groschen left to the player.
        coins: u32,
    },
    /// A relic that stashes Groschen ("Stammtischkasse") left the player
    /// (sold, swapped, replaced or an evening rule ended): its pot was paid
    /// out first, as it was, and emptied (DESIGN 8.7).
    StashPaidOut {
        /// The relic that left.
        relic: RelicId,
        /// Groschen paid out.
        amount: u32,
        /// Groschen of the player now.
        coins: u32,
    },
    /// The stops of the open route choice were drawn again ("Schwenker").
    RoutesRerolled,
    /// The plan for the next pub changed ("Wegbier", "Der Skatlehrer",
    /// "Kiebitz am Fenster"); the whole plan now.
    NextPubPlanned {
        /// Einmischen more (fewer if negative).
        redeals: i32,
        /// Change of the Zielwert.
        target: i32,
    },
    /// The plan for the next pub took effect as it started.
    NextPubApplied {
        /// Einmischen more (fewer if negative: in the first game).
        redeals: i32,
        /// Change of the Zielwert.
        target: i32,
    },
    /// A bet on the next declared game was placed ("Die Wette").
    WagerPlaced {
        /// Groschen staked.
        stake: u32,
    },
    /// The bet was decided by a scored game.
    WagerSettled {
        /// Groschen staked.
        stake: u32,
        /// Won with Schneider.
        won: bool,
        /// Groschen paid out (0 if lost).
        payout: u32,
    },
    /// The exact stops of the next route choices are shown ("Kiebitz am
    /// Fenster").
    StopsRevealed {
        /// Route choices still shown.
        legs: u32,
    },
    /// The malus of a Spelunke was chosen ("Spelunkenkenner").
    MalusChosen {
        /// Index of the route.
        option: usize,
        /// The malus.
        malus: crate::content::MalusId,
    },
    /// "Falschspieler am Nebentisch": whether the trick was a curse.
    GambleResolved {
        /// A curse landed instead of the offer.
        curse: bool,
    },
    /// A consumable was gained without paying ("Der Wirt spendiert").
    ConsumableGained {
        /// The consumable.
        consumable: ConsumableId,
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
    /// The run was lost: the last life went.
    RunLost {
        /// District of the pub that cost the last life, or of the boss pub
        /// that was lost (ADR 0017).
        section: u32,
        /// Pub within the district.
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
        /// The boss whose pub was lost: a lost boss pub ends the run
        /// whatever lives are left (ADR 0017). `None` when the last life
        /// went in a normal pub.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        boss: Option<BossId>,
    },
    /// "Die Oma" (ADR 0017): cards of the player's edition carry a curse
    /// until the pub ends; each in place of its normal modification.
    BossCursesLaid {
        /// The boss whose rule laid them.
        boss: BossId,
        /// The cards (ids) with their curse, in id order.
        cards: Vec<(u32, CardModId)>,
    },
    /// The pub ended: the cards a boss cursed are as before again.
    BossCursesLifted {
        /// The cards (ids), in id order.
        cards: Vec<u32>,
    },
    /// A won boss pub (not the last): relics to pick one from (ADR 0017),
    /// see [`BossReward`].
    BossRewardOffered {
        /// The relics on offer.
        relics: Vec<RelicId>,
    },
    /// The relic pick of a won boss pub was passed without a relic.
    BossRewardDeclined,
    /// A won pub gave a random Schnaps (DESIGN 8.9). With a free slot it
    /// is in the slots now; with every slot full the player chooses
    /// ([`RunAction::ChooseSchnappsReward`]).
    SchnappsRewarded {
        /// The Schnaps.
        consumable: ConsumableId,
        /// Every slot was full: the choice is open.
        slots_full: bool,
    },
    /// The Schnaps of a won pub took the slot of an owned one, which is
    /// gone. Sold instead: [`RunEvent::ConsumableSold`].
    SchnappsRewardTaken {
        /// The new Schnaps.
        consumable: ConsumableId,
        /// The Schnaps given up for it.
        given_up: ConsumableId,
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
    /// "Blue Curaçao" cannot repaint this card into this suit: a jack, a blank,
    /// or the suit it already has.
    CannotChalk,
    /// The rest of the game can still change its score
    /// ([`RunAction::SettleGame`]).
    NotDecided,
    /// "Reue": there is no Einmischen of this game to take back.
    NoRedealToUndo,
    /// "Lockvogel": no lead can be demanded in this game. Never returned
    /// since the relic was removed (02.11.2026); kept so that the error
    /// codes stay stable.
    NoLeadDemandLeft,
    /// A Spelunke's malus locks the Schnäpse in this pub (ADR 0016).
    ConsumablesLocked,
    /// No life is missing ("Konterbier").
    LivesFull,
    /// The Kartenmaler waits for the player's choice
    /// ([`RunAction::ChoosePaint`]); nothing else goes before it.
    PainterPending,
    /// The Kartenmaler or "Karte reinigen" was used at this Tresen already.
    AlreadyUsed,
    /// "Bruderherz": a curse on his edition cannot be cleansed.
    CurseStays,
    /// The consumable does nothing now: "Klarer" with an account of 0 or
    /// more, "Eierlikör" with an account of half the Zielwert or more.
    ConditionNotMet,
    /// Not in a boss pub ("Absacker").
    NotInBossPub,
    /// The Schnaps of a won pub waits for the player's choice
    /// ([`RunAction::ChooseSchnappsReward`]); nothing else goes before it.
    SchnappsRewardPending,
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
    /// The deck cannot shrink below 32 cards (no longer produced since
    /// ADR 0015: no card leaves the deck).
    DeckAtMinimum,
    /// "Frisches Deck": the card carries no modification.
    NothingToCleanse,
    /// Blanks carry no modifications.
    BlankCannotBeModified,
    /// The card already carries this modification and it cannot be
    /// raised.
    AlreadyModified,
    /// The card carries a curse and takes no modification (DESIGN 7.2).
    CardCursed,
    /// The modification on the card is at its highest value.
    ModAtMaximum,
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
    /// "Blue Curaçao" cannot repaint this card into this suit.
    CannotChalk,
    /// The rest of the game can still change its score; it cannot be
    /// settled ([`RunAction::SettleGame`]).
    NotDecided,
    /// "Reue": no Einmischen of this game to take back.
    NoRedealToUndo,
    /// "Lockvogel" (removed on 02.11.2026, never returned, kept for stable
    /// codes): the lead did not meet the demand.
    MustLeadDemanded,
    /// "Lockvogel" (never returned, kept for stable codes): no lead could
    /// be demanded.
    InvalidLeadDemand,
    /// "Lockvogel" (never returned, kept for stable codes): none left.
    NoLeadDemandLeft,
    /// A Spelunke's malus locks the Schnäpse.
    ConsumablesLocked,
    /// No life is missing.
    LivesFull,
    /// The Kartenmaler waits for the player's choice.
    PainterPending,
    /// The Kartenmaler or "Karte reinigen" was used at this Tresen already.
    AlreadyUsed,
    /// The character's curses cannot be cleansed ("Bruderherz").
    CurseStays,
    /// Another card of the edition is this special card already (at most
    /// one Untot and one Wandelkarte, DESIGN 7.2).
    SpecialTaken,
    /// The consumable does nothing now ("Klarer" without a minus,
    /// "Eierlikör" without being behind).
    ConditionNotMet,
    /// Not in a boss pub ("Absacker").
    NotInBossPub,
    /// The Schnaps of a won pub waits for the player's choice
    /// ([`RunAction::ChooseSchnappsReward`]); nothing else goes before it.
    SchnappsRewardPending,
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
            RunErrorCode::NothingToCleanse => "NothingToCleanse",
            RunErrorCode::BlankCannotBeModified => "BlankCannotBeModified",
            RunErrorCode::AlreadyModified => "AlreadyModified",
            RunErrorCode::CardCursed => "CardCursed",
            RunErrorCode::ModAtMaximum => "ModAtMaximum",
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
            RunErrorCode::MustLeadDemanded => "MustLeadDemanded",
            RunErrorCode::InvalidLeadDemand => "InvalidLeadDemand",
            RunErrorCode::NoLeadDemandLeft => "NoLeadDemandLeft",
            RunErrorCode::ConsumablesLocked => "ConsumablesLocked",
            RunErrorCode::LivesFull => "LivesFull",
            RunErrorCode::PainterPending => "PainterPending",
            RunErrorCode::AlreadyUsed => "AlreadyUsed",
            RunErrorCode::CurseStays => "CurseStays",
            RunErrorCode::SpecialTaken => "SpecialTaken",
            RunErrorCode::ConditionNotMet => "ConditionNotMet",
            RunErrorCode::NotInBossPub => "NotInBossPub",
            RunErrorCode::SchnappsRewardPending => "SchnappsRewardPending",
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
                DeckError::NothingToCleanse(_) => RunErrorCode::NothingToCleanse,
                DeckError::BlankCannotBeModified => RunErrorCode::BlankCannotBeModified,
                DeckError::AlreadyModified(_) => RunErrorCode::AlreadyModified,
                DeckError::Cursed(_) => RunErrorCode::CardCursed,
                DeckError::AtMaximum(_) => RunErrorCode::ModAtMaximum,
                DeckError::UnknownMod(_) => RunErrorCode::UnknownContent,
                DeckError::MissingCard(_) => RunErrorCode::MissingCard,
                DeckError::SpecialTaken(_) => RunErrorCode::SpecialTaken,
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
            RunError::NoLeadDemandLeft => RunErrorCode::NoLeadDemandLeft,
            RunError::ConsumablesLocked => RunErrorCode::ConsumablesLocked,
            RunError::LivesFull => RunErrorCode::LivesFull,
            RunError::PainterPending => RunErrorCode::PainterPending,
            RunError::AlreadyUsed => RunErrorCode::AlreadyUsed,
            RunError::CurseStays => RunErrorCode::CurseStays,
            RunError::ConditionNotMet => RunErrorCode::ConditionNotMet,
            RunError::NotInBossPub => RunErrorCode::NotInBossPub,
            RunError::SchnappsRewardPending => RunErrorCode::SchnappsRewardPending,
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
            RunError::NoLeadDemandLeft => f.write_str("no lead can be demanded in this game"),
            RunError::ConsumablesLocked => f.write_str("the Schnäpse are locked in this pub"),
            RunError::LivesFull => f.write_str("no life is missing"),
            RunError::PainterPending => f.write_str("the Kartenmaler waits for a choice"),
            RunError::AlreadyUsed => f.write_str("already used at this Tresen"),
            RunError::CurseStays => f.write_str("the character's curses cannot be cleansed"),
            RunError::ConditionNotMet => f.write_str("the consumable would do nothing now"),
            RunError::NotInBossPub => f.write_str("not in a boss pub"),
            RunError::SchnappsRewardPending => {
                f.write_str("the Schnaps of the won pub waits for a choice")
            }
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
    /// Lives (Leben, ADR 0016).
    #[serde(default = "Run::default_lives")]
    lives: u32,
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
    /// The district in which "Schutzengel" spared a life.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    spared_section: Option<u32>,
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
                discard_three_used: false,
                spelunke: None,
                temp_curses: Vec::new(),
                boss_curses: Vec::new(),
                boss_cursed: false,
                returning_vouchers: Vec::new(),
                redeals_withheld: 0,
            },
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
            lives: content.run.lives.start,
            tour_rng: master.fork(),
            route: None,
            trip: None,
            pub_results: Vec::new(),
            next_pub: NextPubPlan::default(),
            debt: 0,
            credit_section: None,
            spared_section: None,
            wager: None,
            peek_next_skat: None,
            scout_legs: 0,
            boss_reward: None,
            zinken_rng: master.fork(),
            painter: None,
            trade_rng: master.fork(),
            schnapps_rng: master.fork(),
            schnapps_reward: None,
        };
        let mut events = Vec::new();
        run.start_series(content, start.section, start.index, &mut events)?;
        run.play_ai(content, &mut events)?;
        Ok((run, events))
    }

    /// Drop what the content no longer knows from a loaded run (relics and
    /// consumables removed by the item review of 02.11.2026, DESIGN 18.3;
    /// renamed consumables, [`RENAMED_CONSUMABLES`], are renamed first):
    /// owned and evening relics, consumables, Tresen offers, locked
    /// relics, consumables waiting for the next game and encounter options
    /// that name them. Returns the ids dropped, in order, for a note.
    /// Bosses removed by ADR 0017 are replaced by their stand-in
    /// ([`RETIRED_BOSSES`]; a game in progress keeps its deal). Nothing
    /// else changes; a run without such ids stays as it is.
    pub fn drop_unknown_content(&mut self, content: &Content) -> Vec<String> {
        // Renamed consumables keep their place.
        let rename = |id: &mut ConsumableId| {
            if let Some((_, new)) = RENAMED_CONSUMABLES
                .iter()
                .find(|(old, _)| *old == id.as_str())
            {
                *id = ConsumableId::new(new);
            }
        };
        self.consumables.iter_mut().for_each(rename);
        self.pending_next_game
            .iter_mut()
            .for_each(|(id, _)| rename(id));
        if let Some(shop) = self.shop.as_mut() {
            shop.consumables.iter_mut().flatten().for_each(rename);
        }
        if let Some(encounter) = self.encounter.as_mut() {
            for option in &mut encounter.options {
                for effect in &mut option.effects {
                    if let EncounterEffect::Schnapps(Some(id)) = effect {
                        rename(id);
                    }
                }
            }
        }
        // Removed bosses (ADR 0017) give way to their stand-in.
        let retire = |id: &mut BossId| {
            if let Some((_, new)) = RETIRED_BOSSES.iter().find(|(old, _)| *old == id.as_str()) {
                *id = BossId::new(new);
            }
        };
        self.section_boss.iter_mut().for_each(retire);
        self.series.boss.iter_mut().for_each(retire);
        let mut next = self.game.as_mut();
        while let Some(active) = next {
            active.boss_rule.iter_mut().for_each(retire);
            next = active.previous.as_deref_mut();
        }
        let mut dropped: Vec<String> = Vec::new();
        let keep_relic = |id: &RelicId, dropped: &mut Vec<String>| {
            let known = content.relic(id).is_some();
            if !known {
                dropped.push(id.to_string());
            }
            known
        };
        self.relics.retain(|r| keep_relic(r, &mut dropped));
        self.evening_relics.retain(|r| keep_relic(r, &mut dropped));
        self.locked.retain(|r| content.relic(r).is_some());
        let keep_item = |id: &ConsumableId, dropped: &mut Vec<String>| {
            let known = content.consumable(id).is_some();
            if !known {
                dropped.push(id.to_string());
            }
            known
        };
        self.consumables.retain(|c| keep_item(c, &mut dropped));
        self.pending_next_game
            .retain(|(c, _)| keep_item(c, &mut dropped));
        if let Some(shop) = self.shop.as_mut() {
            for offer in &mut shop.relics {
                if offer.as_ref().is_some_and(|r| content.relic(r).is_none()) {
                    *offer = None;
                }
            }
            for offer in &mut shop.consumables {
                if offer
                    .as_ref()
                    .is_some_and(|c| content.consumable(c).is_none())
                {
                    *offer = None;
                }
            }
        }
        if let Some(id) = self.schnapps_reward.as_mut() {
            rename(id);
        }
        // A pending Schnaps the content no longer knows: nothing to choose.
        if self
            .schnapps_reward
            .as_ref()
            .is_some_and(|c| !keep_item(c, &mut dropped))
        {
            self.schnapps_reward = None;
        }
        self.drop_unknown_zinken(content);
        self.renew_encounter(content);
        dropped
    }

    /// Encounter generator of saves from before encounters existed:
    /// derived from nothing but a constant, so loading stays
    /// deterministic.
    fn default_encounter_rng() -> Rng {
        Rng::from_seed(0x5354_414D_4D54_4953)
    }

    /// Lives of a save from before the Kneipentour.
    const fn default_lives() -> u32 {
        3
    }

    /// Route generator of saves from before the Kneipentour.
    fn default_tour_rng() -> Rng {
        Rng::from_seed(0x4B4E_4549_5045_4E54)
    }

    /// Trade generator of saves from before curses as trade ("HANDEL").
    fn default_trade_rng() -> Rng {
        Rng::from_seed(0x4841_4E44_454C_0000)
    }

    /// Schnaps generator of saves from before the Schnaps-Leiste
    /// ("SCHNAPS").
    fn default_schnapps_rng() -> Rng {
        Rng::from_seed(0x5343_484E_4150_5300)
    }

    /// Zinken generator of saves from before the Auslage ("ZINKEN").
    fn default_zinken_rng() -> Rng {
        Rng::from_seed(0x5A49_4E4B_454E_0000)
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
    pub const fn curve_row(&self, _content: &Content) -> u32 {
        self.series.section
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
    /// it. The knowledge hangs on the card (DESIGN 7.2): a card is shown
    /// if its holder's copy is marked (the player's own, or a boss card
    /// such as the black Unter of "Der Schwarze Peter"), and also if the
    /// player's edition marks it while a defender holds it: then the
    /// player sees who holds it and that defender's edition of the card.
    #[must_use]
    pub fn revealed_holders(&self) -> Vec<(DealtCard, Option<Seat>)> {
        let Some(active) = &self.game else {
            return Vec::new();
        };
        let marked_by_player = |c: &DealtCard| active.player_marked.contains(&c.id);
        active
            .game
            .cards_in_play()
            .iter()
            .map(|&c| active.game.current(c))
            .filter(|c| c.revealed || marked_by_player(c))
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
        if self.painter.is_some() && !matches!(action, RunAction::ChoosePaint(_)) {
            return Err(RunError::PainterPending);
        }
        if self.schnapps_reward.is_some()
            && !matches!(action, RunAction::ChooseSchnappsReward { .. })
        {
            return Err(RunError::SchnappsRewardPending);
        }
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
            RunAction::SellConsumable(index) => self.sell_consumable(content, index, events),
            RunAction::Reroll => self.reroll(content, events),
            RunAction::BuyCardOffer(slot) => self.buy_card_offer(content, slot, events),
            RunAction::RerollCardOffers => self.reroll_card_offers(content, events),
            RunAction::HirePainter(card) => self.hire_painter(content, card, events),
            RunAction::ChoosePaint(option) => self.choose_paint(content, option, events),
            RunAction::CleanseCard(card) => self.cleanse_at_tresen(content, card, events),
            RunAction::LeaveShop => self.leave_shop(content, events),
            RunAction::GiveUpPub => self.give_up_pub(content, events),
            RunAction::TakeFromSkat { give, take } => self.take_from_skat(give, take, events),
            RunAction::SettleGame => self.settle_game(content, events),
            RunAction::ChooseEncounter { option, target } => {
                self.choose_encounter(content, option, target, events)
            }
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

    /// "Untot" (DESIGN 7.2): after the `OnTrickEnd` hooks of a completed
    /// trick, the player's card in it rises back into his hand if it may
    /// ([`Run::can_rise`], judged as if it were still in the hand), and a
    /// card of his hand drawn with the game's generator takes its place in
    /// the trick.
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
        let fallback = opponents::game_seed(self) ^ 0x0DEA_D0DE_AD00;
        let Some(active) = self.game.as_mut() else {
            return;
        };
        let rng = active
            .undead_rng
            .get_or_insert_with(|| Rng::from_seed(fallback));
        let roll = rng.gen_range(0..u64::try_from(hand.len()).unwrap_or(1));
        let Some(&substitute) = usize::try_from(roll).ok().and_then(|i| hand.get(i)) else {
            return;
        };
        if let Ok(game_events) = active.game.raise(Seat::DECLARER, card, substitute) {
            active.risen.push(card.id);
            events.extend(game_events.into_iter().map(RunEvent::Game));
        }
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
        if let Some(def) = self.boss_rule(content).and_then(|b| content.boss(&b)) {
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
        let relics = self.relics.iter().chain(&self.evening_relics);
        for def in relics.filter_map(|r| content.relic(r)) {
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
                        transport,
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
        let line = score_line(
            source,
            effect,
            &self.counters,
            self.cursed_for(content, effect),
        );
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
            | Effect::AddBaseValuePerCursedCard(_)
            | Effect::AddLevelsPerCounter { .. }
            | Effect::AddLevels(_)
            | Effect::MultiplyGameValue(_)
            | Effect::ScaleGameValue(_) => active.lines.extend(line),
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
            Effect::ShowCount => active.rules.show_count = true,
            Effect::RevealDefenderHand(index) => {
                let seat = Seat::from_index(usize::from(*index)).filter(|s| !s.is_declarer());
                if let Some(seat) = seat {
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
            Effect::RequireOuvert => active.rules.require_ouvert = true,
            Effect::DefendersSeeEachOther => active.game.open_defender_hands(),
            Effect::Ramsch => active.rules.ramsch = true,
            Effect::SingleLoss => {
                active.single_loss.get_or_insert_with(|| source.clone());
            }
            Effect::SetCardPoints { .. }
            | Effect::AddCardPoints(_)
            | Effect::RankBonus(_)
            | Effect::RevealHolder
            | Effect::NeverWins
            | Effect::HonoraryJack
            | Effect::NoDiscard
            | Effect::Wild
            | Effect::Demote
            | Effect::TurnInto(_)
            | Effect::Undead
            | Effect::AlwaysForehand
            | Effect::ShopDiscount(_)
            | Effect::CurseForPub { .. }
            | Effect::SetFlag(_)
            | Effect::ClearFlag(_)
            | Effect::IncrementCounter(_)
            | Effect::ResetCounter(_)
            | Effect::GainCoins(_)
            | Effect::StashCoins { .. }
            | Effect::PayOutCounter { .. }
            | Effect::Credit(_)
            | Effect::SpareLife
            | Effect::ChooseMalus(_) => {}
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
        self.start_pub(content, section, index, None, events)
    }

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
        let config = &content.run;
        let sections = config.sections();
        if section >= sections || index >= PUBS_PER_DISTRICT {
            return Err(RunError::WrongPhase);
        }
        if index == 0 && !self.evening_relics.is_empty() {
            for relic in std::mem::take(&mut self.evening_relics) {
                self.pay_out_stash(content, &relic, events);
            }
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
        let boss = if index + 1 == PUBS_PER_DISTRICT {
            self.section_boss.clone()
        } else {
            None
        };
        let mut target = series_target(content, self.character, section, index, boss.as_ref())
            .ok_or(RunError::WrongPhase)?;
        if spelunke.is_some() {
            target = config.spelunke.target_factor.apply(target);
        }
        let plan = std::mem::take(&mut self.next_pub);
        target = plan.apply_to(target);
        let redeals_withheld = plan.redeals.min(0).unsigned_abs();
        let malus = spelunke.as_ref().map(|s| s.malus.clone());
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
            discard_three_used: false,
            spelunke,
            temp_curses: Vec::new(),
            boss_curses: Vec::new(),
            boss_cursed: false,
            returning_vouchers: Vec::new(),
            redeals_withheld,
        };
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
            redeals: content.redeals(self.character),
            boss,
            malus,
        });
        self.curse_for_pub(content, events);
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
        }
        let (resolved, boss_editions) = self.resolve_editions(content, &global);
        let mut rng = self.deal_rng.fork();
        let game = Game::deal_editions(&mut rng, resolved.clone().map(|r| r.cards), forehand)?;
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
        let player_marked = resolved[Seat::DECLARER.index()]
            .cards
            .iter()
            .filter(|c| c.revealed)
            .map(|c| c.id)
            .collect();
        self.set_up_game(content, game, point_changes, events);
        if let Some(active) = self.game.as_mut() {
            let [boss, partner] = boss_editions;
            active.boss_edition = boss;
            active.partner_edition = partner;
            active.player_marked = player_marked;
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
        let editions = resolved.clone().map(|r| r.cards);
        let pick = |seat: usize, kind: CardKind| -> DealtCard {
            *editions[seat]
                .iter()
                .find(|c| c.kind == kind)
                .expect("card in the deck")
        };
        let hands = [0, 1, 2].map(|seat| hands[seat].iter().map(|&k| pick(seat, k)).collect());
        let skat = skat.map(|k| pick(0, k));
        let game = Game::with_editions(hands, skat, editions, forehand).expect("valid deal");
        self.install_game(content, game, &resolved, boss_editions, events);
    }

    /// Fixture tool: a run saved before ADR 0015 continues with editions
    /// (its game in progress resolved card by card by the holder).
    #[cfg(test)]
    pub(crate) fn adopt_editions(&mut self, content: &Content) {
        let global = self.sources(content, false);
        let (resolved, boss_editions) = self.resolve_editions(content, &global);
        let player_marked: Vec<u32> = resolved[0]
            .cards
            .iter()
            .filter(|c| c.revealed)
            .map(|c| c.id)
            .collect();
        let editions = resolved.map(|r| r.cards);
        let mut next = self.game.as_mut();
        while let Some(active) = next {
            active.game.adopt_editions(editions.clone());
            active.boss_edition.clone_from(&boss_editions[0]);
            active.partner_edition.clone_from(&boss_editions[1]);
            active.player_marked.clone_from(&player_marked);
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
        let global = self.sources(content, false);
        let has = |wanted: &Effect| global.iter().any(|s| s.passive.contains(wanted));
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
            boss_rule,
            skat_peeked: false,
            chalked: Vec::new(),
            single_loss: None,
            win_factors: Vec::new(),
            shown_hand: None,
            previous: None,
            named: Vec::new(),
            skat_shown: Vec::new(),
            risen: Vec::new(),
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
        let sources = self.sources(content, true);
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
        } else {
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
        // A boss pub plays all its games (ADR 0017); the target counts
        // after the last.
        let early = !self.series.is_boss_pub() || self.series.games_left() == 0;
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
            .filter(|r| !self.locked.contains(&r.id))
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
            card_rerolls: 0,
            painter_used: false,
            cleansed: false,
            kiosk: false,
        }
    }

    /// The passive effects of the owned relics and the evening's house
    /// rules.
    fn relic_passives<'c>(&self, content: &'c Content) -> Vec<&'c Effect> {
        self.relics
            .iter()
            .chain(&self.evening_relics)
            .filter_map(|r| content.relic(r))
            .flat_map(|def| def.passive.iter())
            .collect()
    }

    /// The Source of the first owned relic with a passive effect matching
    /// `wanted`.
    fn relic_with(&self, content: &Content, wanted: impl Fn(&Effect) -> bool) -> Option<Source> {
        self.relics
            .iter()
            .chain(&self.evening_relics)
            .filter_map(|r| content.relic(r))
            .find(|def| def.passive.iter().any(&wanted))
            .map(|def| Source::Relic(def.id.clone()))
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

    /// Income pays the debt first ("Deckel anschreiben").
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
        let still_held: BTreeSet<String> = self
            .relics
            .iter()
            .chain(&self.evening_relics)
            .flat_map(stashes)
            .collect();
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
        let keep = content
            .character(self.character)
            .is_some_and(|c| c.rule.keep_curses);
        if keep && self.card_is_cursed(content, card) {
            return Err(RunError::CurseStays);
        }
        Ok(self.edition.cleanse(card)?)
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

    /// [`Run::cursed_cards`] for an effect that counts them, else 0 (not
    /// counted).
    fn cursed_for(&self, content: &Content, effect: &Effect) -> u32 {
        if matches!(effect, Effect::AddBaseValuePerCursedCard(_)) {
            self.cursed_cards(content)
        } else {
            0
        }
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

    /// The open shop if it is the Tresen (not a Kiosk).
    fn tresen_mut(&mut self) -> Result<&mut Shop, RunError> {
        let shop = self.shop_mut()?;
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
        self.pay_out_stash(content, &id, events);
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

    fn reroll(&mut self, content: &Content, events: &mut Vec<RunEvent>) -> Result<(), RunError> {
        let rerolls = self.tresen_mut()?.rerolls;
        let cost = self.shop_price(content, content.run.economy.reroll_cost);
        if !self.can_pay(content, cost) {
            return Err(RunError::NotEnoughCoins);
        }
        self.pay(cost, events);
        let old = self.shop.take();
        let mut shop = self.draw_offer(content, rerolls + 1);
        if let Some(old) = old {
            shop.on_credit = old.on_credit;
            shop.card_offers = old.card_offers;
            shop.card_rerolls = old.card_rerolls;
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
        if def.category == crate::content::ConsumableCategory::Schnapps
            && self.phase == RunPhase::Game
            && self.malus(content).is_some_and(|m| m.lock_schnapps)
        {
            return Err(RunError::ConsumablesLocked);
        }
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
            ConsumableAction::GainLives(n) => {
                if !matches!(
                    self.phase,
                    RunPhase::Shop | RunPhase::Encounter | RunPhase::Route
                ) {
                    return Err(RunError::WrongPhase);
                }
                if self.lives >= content.run.lives.max {
                    return Err(RunError::LivesFull);
                }
                self.gain_lives(content, *n, events);
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

    /// Card modifications a random draw may give a card, with their draw
    /// weight: for the character, unlocked, no curse (DESIGN 7.2: curses
    /// only where announced) and no special card (that comes with a
    /// voucher), in id order. Rarity does not count here; the weight is the
    /// character's weight for the modification's Spielart
    /// ([`Run::focus_weight`]), and modifications of weight 0 are left out.
    fn card_mod_pool(&self, content: &Content) -> Vec<(CardModId, u32)> {
        self.mod_pool(content, false)
    }

    /// [`Run::card_mod_pool`] for one deck card: only modifications that
    /// may go on it now ([`crate::content::CardModDef::only_on`], and by
    /// the rules of [`Deck::add_mod`]: not on a cursed card, not past the
    /// limit).
    fn card_mod_pool_for(
        &self,
        content: &Content,
        card: &crate::deck::DeckCard,
    ) -> Vec<(CardModId, u32)> {
        self.card_mod_pool(content)
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
            .clone()
            .add_mod(content, card, card_mod, def.default_value())
            .is_ok()
    }

    /// Like [`Run::card_mod_pool`], but only curses (`curse`) or only
    /// modifications without drawback.
    fn mod_pool(&self, content: &Content, curse: bool) -> Vec<(CardModId, u32)> {
        content
            .card_mods
            .values()
            .filter(|m| m.is_curse() == curse && !m.is_special())
            .filter(|m| in_pool(&m.characters, self.character))
            .filter(|m| !self.locked_card_mods.contains(&m.id))
            .filter(|m| self.offer_open(content, m))
            // Only what some card of the deck may still take.
            .filter(|m| {
                self.edition
                    .cards()
                    .iter()
                    .any(|c| self.may_attach(content, c.id, &m.id))
            })
            .map(|m| (m.id.clone(), self.focus_weight(content, m.focus)))
            .filter(|&(_, weight)| weight > 0)
            .collect()
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
    /// the card (from the next game on). A hand game discards nothing.
    fn redeem_vouchers(&mut self, content: &Content, events: &mut Vec<RunEvent>) {
        let Some(active) = self.game.as_ref() else {
            return;
        };
        if !active.game.skat_picked_up() {
            return;
        }
        let skat: Vec<u32> = active.game.skat().iter().map(|c| c.id).collect();
        let vouchers_return = content
            .character(self.character)
            .is_some_and(|c| c.rule.vouchers_return);
        for card in skat {
            let Some(deck_card) = self.edition.get(card) else {
                continue;
            };
            let vouchers: Vec<(CardModId, i32)> = deck_card
                .mods
                .iter()
                .filter(|m| {
                    content
                        .card_mod(m)
                        .is_some_and(|d| d.kind == crate::content::ModKind::Voucher)
                })
                .map(|m| (m.clone(), deck_card.mod_value(m)))
                .collect();
            for (card_mod, value) in vouchers {
                let Some(def) = content.card_mod(&card_mod) else {
                    continue;
                };
                events.push(RunEvent::VoucherRedeemed {
                    card,
                    card_mod: card_mod.clone(),
                    value,
                });
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
                if let Some(change) = self.edition.remove_mod(card, &card_mod) {
                    events.push(RunEvent::DeckChanged(change));
                }
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
                        };
                        evaluate(hook.when(), &ctx, None)
                    };
                    if !holds {
                        continue;
                    }
                    for effect in hook.then() {
                        if let Effect::TurnInto(special) = effect {
                            self.turn_into(content, card, &card_mod, special, &source, events);
                        } else {
                            self.apply_effect(content, &source, &effect.scaled(value), events);
                        }
                    }
                }
            }
        }
    }

    /// A redeemed voucher makes `card` the special card `special`
    /// (`TurnInto`). If another card is that special card already (at most
    /// one Untot and one Wandelkarte, DESIGN 7.2), the card stays as it is
    /// and the voucher pays [`crate::content::ZinkenConfig::capped_voucher_coins`]
    /// instead. A card that is the special card already stays so, nothing
    /// paid (the Drückeberger's returned Totenmarke, DESIGN 9.4).
    fn turn_into(
        &mut self,
        content: &Content,
        card: u32,
        voucher: &CardModId,
        special: &CardModId,
        source: &Source,
        events: &mut Vec<RunEvent>,
    ) {
        match self.edition.add_mod(content, card, special, 1) {
            Ok(change) => events.push(RunEvent::DeckChanged(change)),
            Err(DeckError::SpecialTaken(_)) => {
                let coins = content.run.shop.zinken.capped_voucher_coins;
                events.push(RunEvent::VoucherCapped {
                    card,
                    card_mod: voucher.clone(),
                    special: special.clone(),
                    holder: self.edition.holder_of(special).unwrap_or(card),
                    coins,
                });
                let amount = i32::try_from(coins).unwrap_or(i32::MAX);
                self.apply_effect(content, source, &Effect::GainCoins(amount), events);
            }
            Err(_) => {}
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
        if self.malus(content).is_some_and(|m| m.no_redeal) {
            return 0;
        }
        let extra = self.game.as_ref().map_or(0, |a| a.rules.extra_redeals);
        let withheld = if self.series.games_played == 0 {
            self.series.redeals_withheld
        } else {
            0
        };
        content
            .redeals(self.character)
            .saturating_add(extra)
            .saturating_sub(withheld)
    }

    /// Einmischen left now: what the game in progress still allows of its
    /// own, plus the extras of the series ("Ein Bier", "Kräuter"). 0
    /// without a game. Whether one may be used right now (only with ten
    /// cards before the skat, never in the Ramsch) is
    /// [`Run::accepts`]`(`[`RunAction::Redeal`]`)`.
    #[must_use]
    pub fn redeals_left(&self, content: &Content) -> u32 {
        if self.game.is_none() || self.malus(content).is_some_and(|m| m.no_redeal) {
            return 0;
        }
        self.redeal_allowance(content)
            .saturating_sub(self.series.redeals_in_game)
            .saturating_add(self.series.extra_redeals)
    }
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
/// the run counters as they are now and `cursed_cards` cursed cards in the
/// player's edition ([`Run::cursed_for`]). `None` for every other effect
/// and for a counter (or count of cursed cards) that is still zero.
fn score_line(
    source: &Source,
    effect: &Effect,
    counters: &BTreeMap<String, u32>,
    cursed_cards: u32,
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
        Effect::AddBaseValuePerCursedCard(per) => {
            let add = i64::from(cursed_cards) * i64::from(*per);
            if add == 0 {
                return None;
            }
            (Lever::BaseValue, Change::Add(add))
        }
        Effect::AddLevels(n) => (Lever::Levels, Change::Add(i64::from(*n))),
        Effect::AddLevelsPerCounter { counter, per } => {
            let value = counters.get(counter).copied().unwrap_or(0);
            let add = i64::from(value) * i64::from(*per);
            if add == 0 {
                return None;
            }
            (Lever::Levels, Change::Add(add))
        }
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
mod items_tests;
#[cfg(test)]
mod m4_tests;
#[cfg(test)]
mod m5_tests;
#[cfg(test)]
mod m7_tests;
#[cfg(test)]
mod relic_slots_tests;
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
mod undead_tests;
#[cfg(test)]
mod wave2_tests;
#[cfg(test)]
mod wave2b_tests;
#[cfg(test)]
mod wave3_tests;
#[cfg(test)]
mod zinken_tests;

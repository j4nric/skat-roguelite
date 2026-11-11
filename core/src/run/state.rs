//! The state types of a run besides [`Run`](super::Run) itself: the phase, the pub
//! being played ([`Series`]), the game in progress ([`ActiveGame`]), the
//! open encounter, the Tresen ([`Shop`]), the payout of a won pub and the
//! run statistics. All of them are part of a save.

use std::fmt;

use serde::{Deserialize, Serialize};

use crate::cards::{PlayingCard, Suit};
use crate::content::{
    BetOn, BossId, CardFilter, CardModId, ConsumableId, EncounterEffect, EncounterId, RelicId,
    ScoreLine, Source,
};
use crate::deck::{DealtCard, Edition};
use crate::game::{Event, Game, Seat};
use crate::rng::Rng;
use crate::rules::Thresholds;

use super::{PUBS_PER_DISTRICT, RunError, tour, zinken};

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
    /// A normal pub or a Spelunke was won: the innkeeper offers Zinken,
    /// the player takes one on a card of his choice or skips
    /// ([`card_reward::CardReward`](super::card_reward::CardReward), DESIGN 5.0). Added on 04.11.2026,
    /// after the other phases, so saves keep their phase.
    CardReward,
    /// The last boss pub was won.
    Won,
    /// A pub was lost (ADR 0019: any lost pub ends the run).
    Lost,
}

/// The current pub (Kneipe; the type keeps its name from before ADR 0016,
/// when a section had three series).
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
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
    /// ([`Run::redeals_left`](super::Run::redeals_left)); back to 0 with every new game.
    #[serde(default)]
    pub redeals_in_game: u32,
    /// Every Einmischen of the current game (own, extra or "Neues Blatt");
    /// back to 0 with every new game. "Unterschlupf" keeps cards only at
    /// the first ones ([`crate::content::Effect::RedealKeeps`]).
    #[serde(default)]
    pub redeals_this_game: u32,
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
    /// How often the skat was picked up in this pub ("Der Zocker",
    /// [`GameRules::skat_pickups`]).
    #[serde(default, skip_serializing_if = "is_zero")]
    pub skat_pickups: u32,
    /// The Spelunke this pub is, if it replaced a normal pub (ADR 0016).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub spelunke: Option<tour::Spelunke>,
    /// Cards of the player's edition cursed for this pub only (Spelunke
    /// malus), with the curse that landed; taken off when the pub ends.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub temp_curses: Vec<(u32, CardModId)>,
    /// Cards of the player's edition a boss cursed for this pub ("Die
    /// Oma", [`Effect::CurseForPub`](crate::content::Effect::CurseForPub)), each as it was before; put back
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
    /// The Spelunke malus of a normal pub in the long night
    /// ([`crate::content::NightRule::MalusEveryPub`], ADR 0020).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub night_malus: Option<crate::content::MalusId>,
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
    /// [`Effect::ExtraRedeal`](crate::content::Effect::ExtraRedeal)), on top of `run.ron` and the character
    /// ([`Content::redeals`](crate::content::Content::redeals)). Also read under its old name
    /// `series_redeals`.
    #[serde(alias = "series_redeals")]
    pub extra_redeals: u32,
    /// Every game must be declared Ouvert ("Der Angeber").
    pub require_ouvert: bool,
    /// Cards of the defenders the player sees before declaring ("Der
    /// Kiebitz").
    pub peek: u32,
    /// Cards of each defender made "Durchsichtig" for this game when it is
    /// dealt ("Glasauge", [`Effect::RevealDefenderCards`](crate::content::Effect::RevealDefenderCards)), and the first
    /// source that granted it.
    pub transparent_per_defender: u32,
    /// Who granted [`Self::transparent_per_defender`].
    pub transparent_source: Option<Source>,
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
    /// The player may declare a Ramsch before the skat ("Die
    /// Ramschtante", [`Effect::OfferRamsch`](crate::content::Effect::OfferRamsch)).
    pub ramsch_offered: bool,
    /// How often the skat may be picked up in this pub ("Der Zocker",
    /// [`Effect::SkatPickupsPerPub`](crate::content::Effect::SkatPickupsPerPub)); `None`: as often as wanted.
    pub skat_pickups: Option<u32>,
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
    /// Defender cards (ids) "Durchsichtig" for this game only
    /// ("Glasauge", [`Effect::RevealDefenderCards`](crate::content::Effect::RevealDefenderCards)): drawn from the
    /// defenders' hands at the deal, never a card of the player's edition.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub transparent_for_game: Vec<u32>,
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
    /// granted it. Dormant since 04.11.2026: every lost game counts once
    /// ([`crate::content::RUN_LOSS_FACTOR`]); kept for old saves.
    #[serde(default)]
    pub single_loss: Option<Source>,
    /// Won, this game counts that many times more ("Eierlikör",
    /// [`NextGameEffect::MultiplyWin`](crate::content::NextGameEffect::MultiplyWin)): who granted it and the factor.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub win_factors: Vec<(Source, u32)>,
    /// The defender whose whole hand the player sees (`RevealDefenderHand`):
    /// his cards are in `peeked` with this seat.
    #[serde(default)]
    pub shown_hand: Option<Seat>,
    /// Every defender whose whole hand the player sees: both with "Hosen
    /// runter" (relic, Null Ouvert). Saves from before 07.11.2026 have
    /// only [`ActiveGame::shown_hand`].
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub shown_hands: Vec<Seat>,
    /// The deal before the last Einmischen of this game, kept for "Reue"
    /// ([`ConsumableAction::UndoRedeal`](crate::content::ConsumableAction::UndoRedeal)); its own `previous` is always
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
    /// once per game ([`Effect::Undead`](crate::content::Effect::Undead)).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub risen: Vec<u32>,
    /// An "Untot" card (instance id) that lies in the last trick and waits
    /// for the player's three cards ([`RunAction::ChooseRiseCards`](super::RunAction::ChooseRiseCards)) before
    /// it rises. `None` otherwise and in saves from before 03.11.2026.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pending_rise: Option<u32>,
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
    /// Until 04.11.2026: cards the player's edition marks "Durchsichtig"
    /// in this game (ids), which showed a defender's own copy of the same
    /// card to the player (a bug since ADR 0015, DESIGN 7.2). No longer
    /// filled or read; kept so that old saves load.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub player_marked: Vec<u32>,
}

impl ActiveGame {
    /// Whether the player sees the whole hand of `seat`
    /// (`RevealDefenderHand`).
    #[must_use]
    pub fn shows_hand(&self, seat: Seat) -> bool {
        self.shown_hand == Some(seat) || self.shown_hands.contains(&seat)
    }

    /// Remember the printed suit of every Wandelkarte the game settled in
    /// `events` ([`Event::WildCardSettled`]), for [`Self::named`].
    pub(super) fn note_settled(&mut self, events: &[Event<DealtCard>]) {
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

/// Whether a counter is zero (fields left out of saves while unused).
#[allow(clippy::trivially_copy_pass_by_ref)]
pub(super) const fn is_zero(n: &u32) -> bool {
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

    /// Whether it gives a named relic ([`EncounterEffect::Relic`]); with
    /// every relic slot full it then needs an owned relic to replace
    /// ([`Run::option_needs_relic`](super::Run::option_needs_relic)).
    #[must_use]
    pub fn gains_relic(&self) -> bool {
        self.effects
            .iter()
            .any(|e| matches!(e, EncounterEffect::Relic(Some(_))))
    }

    /// Whether it needs a Marke of the player's choice.
    #[must_use]
    pub fn needs_marke(&self) -> bool {
        self.effects.iter().any(EncounterEffect::needs_marke)
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
    /// No card of the edition can take it.
    NoFittingCard,
    /// No relic owned.
    NoRelic,
    /// Every relic slot is taken.
    RelicSlotsFull,
    /// A bet is open already.
    BetOpen,
    /// A fallback: another option can be taken.
    NotNeeded,
}

impl OptionBlock {
    /// The error [`RunAction::ChooseEncounter`](super::RunAction::ChooseEncounter) returns for it.
    #[must_use]
    pub const fn error(self) -> RunError {
        match self {
            OptionBlock::NotEnoughCoins => RunError::NotEnoughCoins,
            OptionBlock::RelicSlotsFull => RunError::NoFreeSlot,
            OptionBlock::NothingLeft | OptionBlock::NoFittingCard => RunError::NoModAvailable,
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
    /// anew when loaded ([`Run::drop_unknown_content`](super::Run::drop_unknown_content)).
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
    /// "Neu auslosen" during this visit (once per visit since 05.11.2026;
    /// saves from before may hold more).
    pub rerolls: u32,
    /// This visit sold on credit ("Deckel anschreiben"): it may go on
    /// down to the limit.
    #[serde(default)]
    pub on_credit: bool,
    /// The Auslage (Zinken, DESIGN 8.6): concrete modifications on concrete
    /// cards. Empty in saves from before.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub card_offers: Vec<Option<zinken::CardOffer>>,
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
    /// Pubs lost (until ADR 0019 each cost a life; since then the run ends
    /// with one, unless the "Schutzengel" has it played again).
    #[serde(default)]
    pub pubs_lost: u32,
}

/// Groschen paid when selling a relic or consumable: half its price.
#[must_use]
pub const fn sell_price(price: u32) -> u32 {
    price / 2
}

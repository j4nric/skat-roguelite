//! Content of "Die lange Nacht" (ADR 0020, DESIGN 5.5): the Verschärfungen
//! of the endless mode after der Alte, from `night.ron`.
//!
//! Each night district (Stadtteil 4, 5, …) adds one Verschärfung ("Die
//! Sperrstunde rückt näher"), drawn at random without repeat until the
//! list is exhausted. They stay for the rest of the night and stack. What
//! a Verschärfung does is data: one [`NightRule`] each. The growth of the
//! Zielwerte is in `run.ron` ([`NightConfig`]).

use std::fmt;

use serde::{Deserialize, Serialize};

use super::{CardModId, Factor};

content_id!(
    /// Id of a Verschärfung of the long night ([`VerschaerfungDef`]).
    VerschaerfungId
);

/// What a Verschärfung does. Every rule stacks: drawn twice (after the
/// list was exhausted once), it counts twice.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum NightRule {
    /// This many Einmischen less per game (never below 0).
    FewerRedeals(u32),
    /// The opponents' PIMC takes this many more deals per decision (the
    /// heuristic stays the heuristic).
    StrongerOpponents(u32),
    /// The night boss pub applies two boss rules: the drawn boss and
    /// another random one.
    DoubleBoss,
    /// Everything at the Tresen and the Kiosk costs this many Groschen
    /// more (only what costs something).
    PricierTresen(u32),
    /// At the start of every pub this many random cards of the player's
    /// edition carry one of these curses, for that pub only (like the
    /// Oma).
    CursedCards {
        /// Cards per pub.
        cards: u32,
        /// The curses to draw from.
        curses: Vec<CardModId>,
    },
    /// Every normal pub gets a random Spelunke malus (a Spelunke keeps its
    /// own).
    MalusEveryPub,
    /// Every Zielwert of the night times this factor (on top of the
    /// growth).
    TargetJump(Factor),
    /// The card reward after a won pub offers this many Zinken less (at
    /// least one).
    FewerCardOffers(u32),
}

/// A Verschärfung of the long night.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VerschaerfungDef {
    /// Unique id.
    pub id: VerschaerfungId,
    /// German name.
    pub name: String,
    /// German rule text.
    pub text: String,
    /// What it does.
    pub rule: NightRule,
}

/// A card back an unlock may give ([`super::Reward::CardBack`]): cosmetic,
/// no effect on play. The picture belongs to the frontend.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CardBackDef {
    /// Id, also the frontend's design key.
    pub id: String,
    /// German name.
    pub name: String,
    /// German description.
    pub text: String,
}

/// `night.ron`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NightContent {
    /// The Verschärfungen, drawn without repeat until all were drawn.
    pub verschaerfungen: Vec<VerschaerfungDef>,
    /// Card backs the night's unlocks give.
    #[serde(default)]
    pub card_backs: Vec<CardBackDef>,
}

/// The long night in `run.ron` (ADR 0020). VORLÄUFIG.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct NightConfig {
    /// Night district n (Stadtteil 4, 5, …) plays the Zielwerte of the
    /// last district times `growth^(n − 3)`, rounded.
    pub growth: Factor,
}

impl Default for NightConfig {
    fn default() -> NightConfig {
        NightConfig {
            growth: Factor::from_permille(1600),
        }
    }
}

impl NightConfig {
    /// `value × growth^steps`, rounded half up once at the end. The power
    /// is kept in millionths with integer arithmetic, so it is the same
    /// on every platform. Saturates.
    #[must_use]
    pub fn grow(&self, value: u64, steps: u32) -> u64 {
        let mut micro: u128 = 1_000_000;
        let growth = u128::from(self.growth.permille());
        for _ in 0..steps {
            micro = (micro.saturating_mul(growth) + 500) / 1000;
        }
        let scaled = (u128::from(value).saturating_mul(micro) + 500_000) / 1_000_000;
        u64::try_from(scaled).unwrap_or(u64::MAX)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn growth_is_a_rounded_power() {
        let night = NightConfig::default();
        assert_eq!(night.grow(100, 0), 100);
        assert_eq!(night.grow(100, 1), 160);
        assert_eq!(night.grow(100, 2), 256);
        assert_eq!(night.grow(215, 1), 344);
        // 140 × 1.6³ = 573.44
        assert_eq!(night.grow(140, 3), 573);
        assert_eq!(night.grow(u64::MAX, 40), u64::MAX);
    }
}

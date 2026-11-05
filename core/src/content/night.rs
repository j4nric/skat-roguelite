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
    /// Growth of the first night district (Stadtteil 4) over the last
    /// district of the run.
    pub growth: Factor,
    /// "Die Sperrstunde rückt näher": every further night district grows
    /// by this much more than the one before (district 4 × `growth`,
    /// district 5 × `growth + step`, district 6 × `growth + 2 × step`, …),
    /// so the Zielwerte outgrow any factor that rises by a fixed step.
    /// 0 (the default) is a plain power.
    #[serde(default = "Factor::zero")]
    pub growth_step: Factor,
}

impl Default for NightConfig {
    fn default() -> NightConfig {
        NightConfig {
            growth: Factor::from_permille(1600),
            growth_step: Factor::zero(),
        }
    }
}

impl NightConfig {
    /// Growth of night step `step` (1 = the first night district).
    #[must_use]
    pub fn growth_of(&self, step: u32) -> Factor {
        let extra = u64::from(self.growth_step.permille()) * u64::from(step.saturating_sub(1));
        let permille = u64::from(self.growth.permille()).saturating_add(extra);
        Factor::from_permille(u32::try_from(permille).unwrap_or(u32::MAX))
    }

    /// `value` times the growth of the first `steps` night districts
    /// ([`NightConfig::growth_of`]), rounded half up once at the end. The
    /// product is kept in millionths with integer arithmetic, so it is the
    /// same on every platform. Saturates.
    #[must_use]
    pub fn grow(&self, value: u64, steps: u32) -> u64 {
        let mut micro: u128 = 1_000_000;
        for step in 1..=steps {
            let growth = u128::from(self.growth_of(step).permille());
            micro = micro.saturating_mul(growth).saturating_add(500) / 1000;
        }
        let scaled = u128::from(value)
            .saturating_mul(micro)
            .saturating_add(500_000)
            / 1_000_000;
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

    #[test]
    fn growth_can_accelerate_per_district() {
        let night = NightConfig {
            growth: Factor::from_permille(1500),
            growth_step: Factor::from_permille(100),
        };
        assert_eq!(night.growth_of(1).permille(), 1500);
        assert_eq!(night.growth_of(3).permille(), 1700);
        assert_eq!(night.grow(100, 1), 150);
        // 100 × 1.5 × 1.6 = 240
        assert_eq!(night.grow(100, 2), 240);
        // 100 × 1.5 × 1.6 × 1.7 = 408
        assert_eq!(night.grow(100, 3), 408);
        assert_eq!(night.grow(u64::MAX, 60), u64::MAX);
    }

    #[test]
    fn growth_step_defaults_to_a_plain_power() {
        let night: NightConfig = ron::from_str("(growth: 1.6)").expect("ron");
        assert_eq!(night, NightConfig::default());
    }
}

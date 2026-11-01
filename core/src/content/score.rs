//! Score breakdown: the game value by the rules plus every relic
//! modification, line by line, so that a frontend can show which relic
//! contributed how much (the Balatro effect).
//!
//! Formula with modifiers:
//!
//! ```text
//! Grundwert  = max(1, Grundwert der Spielart + Σ Grundwert-Zuschläge) × Π Grundwert-Faktoren
//! Stufen     = max(1, Gewinnstufen nach Regel + Σ Stufen-Zuschläge)
//! Spielwert  = Grundwert × Stufen × Π Spielwert-Faktoren (Kontra, Re, …)
//! Ergebnis   = Spielwert, verloren −2 × Spielwert
//! ```
//!
//! Additions come before multiplications, as in Balatro. Lines on the
//! levers Augen, Gewinnschwelle and Schneider are informational: they
//! already shaped the outcome and the rules result.

use serde::{Deserialize, Serialize};

use super::{BossId, CardModId, RelicId};
use crate::rules::{
    Declaration, LevelParts, Matadors, Outcome, Thresholds, ValueParts, game_value_parts,
};

/// Where a modification comes from.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Source {
    /// A house rule or figure the player owns.
    Relic(RelicId),
    /// A card modification on a card instance.
    CardMod {
        /// The modification.
        card_mod: CardModId,
        /// The card instance id.
        card: u32,
    },
    /// The boss of the current series.
    Boss(BossId),
    /// The defenders said Kontra.
    Kontra,
    /// The declarer answered Re.
    Re,
}

/// The lever of the game value formula a line turns.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Lever {
    /// Grundwert.
    BaseValue,
    /// Gewinnstufen.
    Levels,
    /// Spielwert as a whole.
    GameValue,
    /// Augen of the declarer (informational).
    CardPoints,
    /// Gewinnschwelle (informational).
    WinThreshold,
    /// Schneider (informational).
    Schneider,
}

/// How a line changes its lever.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Change {
    /// Add (or subtract) an amount.
    Add(i64),
    /// Multiply by a factor.
    Multiply(u32),
    /// Set to a value.
    Set(i64),
}

/// One modification of the score.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ScoreLine {
    /// Origin.
    pub source: Source,
    /// Lever.
    pub lever: Lever,
    /// Change.
    pub change: Change,
}

/// The full account of one game's score.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScoreBreakdown {
    /// The declaration.
    pub declaration: Declaration,
    /// Augen and tricks.
    pub outcome: Outcome,
    /// Spitzen.
    pub matadors: Matadors,
    /// Thresholds in force.
    pub thresholds: Thresholds,
    /// Whether the declarer won.
    pub won: bool,
    /// Whether the defenders are Schneider.
    pub schneider: bool,
    /// Whether the defenders are Schwarz.
    pub schwarz: bool,
    /// Grundwert of the Spielart before modifiers.
    pub rule_base_value: u32,
    /// Gewinnstufen by the rules, term by term.
    pub rule_levels: LevelParts,
    /// Every modification, in the order it was applied.
    pub lines: Vec<ScoreLine>,
    /// Grundwert after modifiers.
    pub base_value: u64,
    /// Gewinnstufen after modifiers.
    pub levels: u64,
    /// Product of all game value factors.
    pub multiplier: u64,
    /// Spielwert after modifiers.
    pub value: u64,
    /// Signed result for the series account: `value` or `-2 × value`.
    pub score: i64,
}

impl ScoreBreakdown {
    /// Lines of one source.
    pub fn lines_of<'a>(&'a self, source: &'a Source) -> impl Iterator<Item = &'a ScoreLine> {
        self.lines.iter().filter(move |l| &l.source == source)
    }
}

/// Score a finished game. `lines` are the modifications collected by the
/// run's hooks; the result by the rules comes from `thresholds`.
#[must_use]
pub fn score_game(
    declaration: Declaration,
    matadors: Matadors,
    outcome: &Outcome,
    thresholds: &Thresholds,
    lines: Vec<ScoreLine>,
) -> ScoreBreakdown {
    let parts: ValueParts = game_value_parts(declaration, matadors, outcome, thresholds);
    let mut base_add: i64 = 0;
    let mut base_mul: u64 = 1;
    let mut level_add: i64 = 0;
    let mut value_mul: u64 = 1;
    for line in &lines {
        match (line.lever, line.change) {
            (Lever::BaseValue, Change::Add(n)) => base_add = base_add.saturating_add(n),
            (Lever::BaseValue, Change::Multiply(n)) => {
                base_mul = base_mul.saturating_mul(u64::from(n))
            }
            (Lever::Levels, Change::Add(n)) => level_add = level_add.saturating_add(n),
            (Lever::GameValue, Change::Multiply(n)) => {
                value_mul = value_mul.saturating_mul(u64::from(n));
            }
            _ => {}
        }
    }
    let (base_value, levels) = if parts.base_value == 0 {
        // Null: fixed value, no levels.
        (u64::from(parts.value.value), 1)
    } else {
        let base = (i64::from(parts.base_value) + base_add).max(1);
        let levels = (i64::from(parts.levels.total()) + level_add).max(1);
        (
            u64::try_from(base).unwrap_or(1).saturating_mul(base_mul),
            u64::try_from(levels).unwrap_or(1),
        )
    };
    let value = base_value.saturating_mul(levels).saturating_mul(value_mul);
    let signed = i64::try_from(value).unwrap_or(i64::MAX);
    let score = if parts.won {
        signed
    } else {
        signed.saturating_mul(-2)
    };
    ScoreBreakdown {
        declaration,
        outcome: *outcome,
        matadors,
        thresholds: thresholds.clone(),
        won: parts.won,
        schneider: parts.schneider,
        schwarz: parts.schwarz,
        rule_base_value: parts.base_value,
        rule_levels: parts.levels,
        lines,
        base_value,
        levels,
        multiplier: value_mul,
        value,
        score,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cards::Suit;
    use crate::rules::{GameType, RiskLevel, game_value};

    fn outcome(declarer: u32, declarer_tricks: u32) -> Outcome {
        Outcome {
            declarer_card_points: declarer,
            defender_card_points: 120 - declarer,
            declarer_tricks,
            defender_tricks: 10 - declarer_tricks,
        }
    }

    const MIT_2: Matadors = Matadors {
        with: true,
        count: 2,
    };

    #[test]
    fn without_lines_equals_the_rules() {
        for risk in RiskLevel::ALL {
            for (points, tricks) in [(30, 2), (60, 4), (61, 5), (90, 8), (120, 10)] {
                let decl = Declaration::new(GameType::Suit(Suit::Spades), risk);
                let o = outcome(points, tricks);
                let rules = game_value(decl, MIT_2, &o);
                let b = score_game(decl, MIT_2, &o, &Thresholds::default(), Vec::new());
                assert_eq!(b.value, u64::from(rules.value));
                assert_eq!(b.score, i64::from(rules.score));
                assert_eq!(b.won, rules.won);
            }
        }
    }

    #[test]
    fn additions_come_before_multiplications() {
        let decl = Declaration::new(GameType::Suit(Suit::Clubs), RiskLevel::Normal);
        let relic = Source::Relic(RelicId::new("x"));
        let line = |lever, change| ScoreLine {
            source: relic.clone(),
            lever,
            change,
        };
        let b = score_game(
            decl,
            MIT_2,
            &outcome(70, 6),
            &Thresholds::default(),
            vec![
                line(Lever::BaseValue, Change::Multiply(2)),
                line(Lever::BaseValue, Change::Add(3)),
                line(Lever::Levels, Change::Add(1)),
                line(Lever::GameValue, Change::Multiply(2)),
            ],
        );
        // (12 + 3) × 2 = 30 base, 3 + 1 = 4 levels, × 2.
        assert_eq!((b.base_value, b.levels, b.multiplier), (30, 4, 2));
        assert_eq!(b.value, 240);
        assert_eq!(b.score, 240);
        assert_eq!(b.lines_of(&relic).count(), 4);
        // Lost: −2 × value including modifiers.
        let b = score_game(
            decl,
            MIT_2,
            &outcome(50, 4),
            &Thresholds::default(),
            vec![line(Lever::GameValue, Change::Multiply(2))],
        );
        assert_eq!(b.score, -2 * 36 * 2);
    }
}

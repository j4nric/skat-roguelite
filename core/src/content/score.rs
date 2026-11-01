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
    Declaration, GameType, LevelParts, Matadors, Outcome, Thresholds, ValueParts, game_value_parts,
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

    /// The breakdown as a sequence of steps in the order of the formula,
    /// each with the running Grundwert, Gewinnstufen and Spielwert factor,
    /// so that a frontend can reveal the score term by term without doing
    /// any arithmetic itself.
    ///
    /// Order: informational lines (Augen, Gewinnschwelle, Schneider), the
    /// Grundwert of the Spielart, Grundwert additions, Grundwert factors,
    /// the Gewinnstufen by the rules term by term, Gewinnstufen additions,
    /// Spielwert factors. The last step carries exactly
    /// [`ScoreBreakdown::base_value`], [`ScoreBreakdown::levels`] and
    /// [`ScoreBreakdown::multiplier`].
    #[must_use]
    pub fn steps(&self) -> Vec<ScoreStep> {
        let mut steps = Vec::new();
        let mut base: u64 = 0;
        let mut levels: u64 = 0;
        let mut multiplier: u64 = 1;
        let mut push = |term: ScoreTerm, base: u64, levels: u64, multiplier: u64| {
            steps.push(ScoreStep {
                term,
                base_value: base,
                levels,
                multiplier,
            });
        };
        let informational = |lever: Lever| {
            matches!(
                lever,
                Lever::CardPoints | Lever::WinThreshold | Lever::Schneider
            )
        };
        for line in self.lines.iter().filter(|l| informational(l.lever)) {
            push(ScoreTerm::Line(line.clone()), base, levels, multiplier);
        }
        if self.rule_base_value == 0 {
            // Null: fixed value, no levels and no Grundwert modifiers.
            push(
                ScoreTerm::NullValue(self.base_value),
                self.base_value,
                self.levels,
                multiplier,
            );
            base = self.base_value;
            levels = self.levels;
        } else {
            base = u64::from(self.rule_base_value);
            push(
                ScoreTerm::BaseValue {
                    game: self.declaration.game,
                    value: self.rule_base_value,
                },
                base,
                levels,
                multiplier,
            );
            let mut base_sum = i64::from(self.rule_base_value);
            for line in self.lines.iter().filter(|l| l.lever == Lever::BaseValue) {
                if let Change::Add(n) = line.change {
                    base_sum = base_sum.saturating_add(n);
                    base = u64::try_from(base_sum.max(1)).unwrap_or(1);
                    push(ScoreTerm::Line(line.clone()), base, levels, multiplier);
                }
            }
            for line in self.lines.iter().filter(|l| l.lever == Lever::BaseValue) {
                if let Change::Multiply(n) = line.change {
                    base = base.saturating_mul(u64::from(n));
                    push(ScoreTerm::Line(line.clone()), base, levels, multiplier);
                }
            }
            let parts = self.rule_levels;
            let terms = [
                (
                    ScoreTerm::Matadors {
                        with: self.matadors.with,
                        count: parts.matadors,
                    },
                    parts.matadors,
                ),
                (ScoreTerm::Game, parts.game),
                (ScoreTerm::Hand, parts.hand),
                (ScoreTerm::Schneider, parts.schneider),
                (ScoreTerm::SchneiderDeclared, parts.schneider_declared),
                (ScoreTerm::Schwarz, parts.schwarz),
                (ScoreTerm::SchwarzDeclared, parts.schwarz_declared),
                (ScoreTerm::Ouvert, parts.ouvert),
            ];
            for (term, amount) in terms {
                if amount > 0 {
                    levels = levels.saturating_add(u64::from(amount));
                    push(term, base, levels, multiplier);
                }
            }
            let mut level_sum = i64::from(parts.total());
            for line in self.lines.iter().filter(|l| l.lever == Lever::Levels) {
                if let Change::Add(n) = line.change {
                    level_sum = level_sum.saturating_add(n);
                    levels = u64::try_from(level_sum.max(1)).unwrap_or(1);
                    push(ScoreTerm::Line(line.clone()), base, levels, multiplier);
                }
            }
        }
        for line in self.lines.iter().filter(|l| l.lever == Lever::GameValue) {
            if let Change::Multiply(n) = line.change {
                multiplier = multiplier.saturating_mul(u64::from(n));
                push(ScoreTerm::Line(line.clone()), base, levels, multiplier);
            }
        }
        steps
    }
}

/// One term of the score breakdown, as revealed step by step.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScoreTerm {
    /// Grundwert of the Spielart.
    BaseValue {
        /// The Spielart.
        game: GameType,
        /// Its Grundwert.
        value: u32,
    },
    /// Fixed value of a Null game.
    NullValue(u64),
    /// Spitzen, mit or ohne.
    Matadors {
        /// `true` = mit.
        with: bool,
        /// Number of Spitzen.
        count: u32,
    },
    /// +1 for the game.
    Game,
    /// +1 Hand.
    Hand,
    /// +1 Schneider.
    Schneider,
    /// +1 Schneider angesagt.
    SchneiderDeclared,
    /// +1 Schwarz.
    Schwarz,
    /// +1 Schwarz angesagt.
    SchwarzDeclared,
    /// +1 Ouvert.
    Ouvert,
    /// A modification by a relic, card modification, boss, Kontra or Re.
    Line(ScoreLine),
}

/// One step of the score breakdown with the running totals after it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScoreStep {
    /// What this step adds.
    pub term: ScoreTerm,
    /// Grundwert so far.
    pub base_value: u64,
    /// Gewinnstufen so far.
    pub levels: u64,
    /// Product of the Spielwert factors so far.
    pub multiplier: u64,
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
    fn steps_end_with_the_breakdown_totals() {
        let decl = Declaration::new(GameType::Suit(Suit::Clubs), RiskLevel::Hand);
        let relic = Source::Relic(RelicId::new("x"));
        let line = |lever, change| ScoreLine {
            source: relic.clone(),
            lever,
            change,
        };
        let b = score_game(
            decl,
            MIT_2,
            &outcome(95, 9),
            &Thresholds::default(),
            vec![
                line(Lever::CardPoints, Change::Add(4)),
                line(Lever::GameValue, Change::Multiply(2)),
                line(Lever::BaseValue, Change::Multiply(2)),
                line(Lever::BaseValue, Change::Add(3)),
                line(Lever::Levels, Change::Add(1)),
            ],
        );
        let steps = b.steps();
        let last = steps.last().expect("steps");
        assert_eq!(
            (last.base_value, last.levels, last.multiplier),
            (b.base_value, b.levels, b.multiplier)
        );
        let terms: Vec<&ScoreTerm> = steps.iter().map(|s| &s.term).collect();
        assert!(matches!(terms[0], ScoreTerm::Line(l) if l.lever == Lever::CardPoints));
        assert!(matches!(
            terms[1],
            ScoreTerm::BaseValue {
                value: 12,
                game: GameType::Suit(Suit::Clubs)
            }
        ));
        // 12 + 3 = 15, then × 2 = 30.
        assert_eq!(steps[2].base_value, 15);
        assert_eq!(steps[3].base_value, 30);
        // mit 2, Spiel, Hand, Schneider, then the relic level.
        assert!(matches!(
            terms[4],
            ScoreTerm::Matadors {
                with: true,
                count: 2
            }
        ));
        assert_eq!(steps[4].levels, 2);
        assert!(matches!(terms[5], ScoreTerm::Game));
        assert!(matches!(terms[6], ScoreTerm::Hand));
        assert!(matches!(terms[7], ScoreTerm::Schneider));
        assert_eq!(steps[8].levels, 6);
        assert!(matches!(terms[9], ScoreTerm::Line(l) if l.lever == Lever::GameValue));
        assert_eq!(steps.len(), 10);
        // Without lines the steps are the rule terms only.
        let plain = score_game(decl, MIT_2, &outcome(70, 6), &Thresholds::default(), vec![]);
        let last = plain.steps().last().cloned().expect("steps");
        assert_eq!((last.base_value, last.levels, last.multiplier), (12, 4, 1));
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

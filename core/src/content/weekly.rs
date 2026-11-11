//! Die Wochen-Herausforderung (DESIGN 5.6): every ISO week has one run
//! for everyone, a fixed seed and one Sonderregel der Woche, drawn from
//! `weekly.ron`.
//!
//! A week is named by its ISO year and week as one number `JJJJWW`
//! ([`WeekId`], 202646 is week 46 of 2026). The seed and the rule follow
//! from that number alone: the rules are dealt in rounds, each round a
//! shuffle of the whole list, so no rule comes twice before every rule had
//! its week. What a rule does is data ([`WeeklyRuleDef`]): the run
//! mechanics it reuses (Verschärfungen of the long night, start relics, a
//! start deck, a fixed character, table rules like a boss's).

use std::fmt;

use serde::{Deserialize, Serialize};

use super::{Character, DeckSpec, Effect, Hook, NightRule, RelicId};
use crate::rng::{Rng, mix64};

content_id!(
    /// Id of a Sonderregel der Woche ([`WeeklyRuleDef`]).
    WeeklyRuleId
);

/// A Sonderregel der Woche (`weekly.ron`). Every part is optional and
/// defaults to "no change".
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WeeklyRuleDef {
    /// Unique id.
    pub id: WeeklyRuleId,
    /// German name.
    pub name: String,
    /// German rule text.
    pub text: String,
    /// Everyone plays this character this week, whatever was picked (and
    /// whether it is unlocked or not).
    #[serde(default)]
    pub character: Option<Character>,
    /// Relics owned from the start, after the character's own.
    #[serde(default)]
    pub start_relics: Vec<RelicId>,
    /// Card modifications on the start edition, after the character's
    /// own (a start curse, a start Zinke). Like a character's start deck,
    /// their "beim Erhalt" hooks do not fire.
    #[serde(default)]
    pub deck: DeckSpec,
    /// Groschen at the start, added to (or taken from) `run.ron`'s.
    #[serde(default)]
    pub coins: i32,
    /// Verschärfungen of the long night (ADR 0020) in force for the whole
    /// run; in the night they stack with the drawn ones.
    /// [`NightRule::DoubleBoss`] works in the night only and is refused.
    #[serde(default)]
    pub rules: Vec<NightRule>,
    /// Table rules for every game, like a boss's (source
    /// [`super::Source::Weekly`]).
    #[serde(default)]
    pub passive: Vec<Effect>,
    /// Hooks for every game, like a boss's.
    #[serde(default)]
    pub hooks: Vec<Hook>,
}

/// An ISO week (ISO 8601: weeks start on Monday, week 1 holds the first
/// Thursday of the year), the key of the weekly challenge.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(try_from = "u32", into = "u32")]
pub struct WeekId {
    year: u32,
    week: u32,
}

/// Salt of the weekly seeds ("WOCHE").
const WEEK_SEED_SALT: u64 = 0x574F_4348_4500_0000;
/// Salt of the order of the rules.
const WEEK_RULE_SALT: u64 = 0x5245_4745_4C00_0000;

/// Days since 1970-01-01 of a civil date (proleptic Gregorian).
const fn days_from_civil(y: i64, m: i64, d: i64) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = y.div_euclid(400);
    let yoe = y - era * 400;
    let mp = (m + 9) % 12;
    let doy = (153 * mp + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

/// Weekday of a day number, 0 = Monday.
const fn weekday(days: i64) -> i64 {
    // 1970-01-01 was a Thursday.
    (days + 3).rem_euclid(7)
}

/// The Monday of ISO week 1 of `year`, as a day number.
const fn week_one_monday(year: i64) -> i64 {
    let jan4 = days_from_civil(year, 1, 4);
    jan4 - weekday(jan4)
}

/// How many ISO weeks `year` has: 52 or 53.
#[must_use]
pub const fn weeks_in_year(year: u32) -> u32 {
    let y = year as i64;
    if week_one_monday(y + 1) - week_one_monday(y) == 53 * 7 {
        53
    } else {
        52
    }
}

impl WeekId {
    /// Years a week may lie in.
    pub const YEARS: std::ops::RangeInclusive<u32> = 2000..=2999;

    /// The week, if it exists (year 2000 to 2999, week 1 to 52 or 53).
    #[must_use]
    pub fn new(year: u32, week: u32) -> Option<WeekId> {
        (Self::YEARS.contains(&year) && (1..=weeks_in_year(year)).contains(&week))
            .then_some(WeekId { year, week })
    }

    /// The week of a number `JJJJWW` (202646).
    #[must_use]
    pub fn from_number(number: u32) -> Option<WeekId> {
        WeekId::new(number / 100, number % 100)
    }

    /// The ISO week of a day number (days since 1970-01-01).
    #[must_use]
    pub fn of_day(days: i64) -> Option<WeekId> {
        // The Thursday of the same week names its year.
        let thursday = days - weekday(days) + 3;
        let z = thursday + 719_468;
        let era = z.div_euclid(146_097);
        let doe = z - era * 146_097;
        let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
        let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
        let mp = (5 * doy + 2) / 153;
        let year = yoe + era * 400 + i64::from(mp >= 10);
        let week = (days - week_one_monday(year)).div_euclid(7) + 1;
        WeekId::new(u32::try_from(year).ok()?, u32::try_from(week).ok()?)
    }

    /// The number `JJJJWW`.
    #[must_use]
    pub const fn number(self) -> u32 {
        self.year * 100 + self.week
    }

    /// ISO year.
    #[must_use]
    pub const fn year(self) -> u32 {
        self.year
    }

    /// ISO week, from 1.
    #[must_use]
    pub const fn week(self) -> u32 {
        self.week
    }

    /// The Monday of the week, as a day number (days since 1970-01-01).
    #[must_use]
    pub const fn monday(self) -> i64 {
        week_one_monday(self.year as i64) + (self.week as i64 - 1) * 7
    }

    /// Weeks since ISO week 1 of 2000 (0 there).
    #[must_use]
    pub const fn index(self) -> u64 {
        ((self.monday() - week_one_monday(2000)) / 7) as u64
    }

    /// The seed of the week's run: the same for everyone, never a daily
    /// seed's pattern.
    #[must_use]
    pub const fn seed(self) -> u64 {
        mix64(self.number() as u64 ^ WEEK_SEED_SALT)
    }
}

impl TryFrom<u32> for WeekId {
    type Error = String;

    fn try_from(number: u32) -> Result<WeekId, String> {
        WeekId::from_number(number).ok_or_else(|| format!("no ISO week {number}"))
    }
}

impl From<WeekId> for u32 {
    fn from(week: WeekId) -> u32 {
        week.number()
    }
}

impl fmt::Display for WeekId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{:02}", self.year, self.week)
    }
}

/// The rule of `week` among `rules` (in id order): round `index / n` of
/// the rules is a shuffle of all `n`, seeded by the round, and the week
/// takes its place in it. `None` if there are no rules.
#[must_use]
pub fn pick_rule<T>(rules: &[T], week: WeekId) -> Option<&T> {
    let n = u64::try_from(rules.len()).ok().filter(|&n| n > 0)?;
    let round = week.index() / n;
    let mut order: Vec<usize> = (0..rules.len()).collect();
    Rng::from_seed(round ^ WEEK_RULE_SALT).shuffle(&mut order);
    let at = usize::try_from(week.index() % n).ok()?;
    order.get(at).and_then(|&i| rules.get(i))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn iso_weeks_follow_the_calendar() {
        // 2026-11-10 (a Tuesday) lies in week 46.
        let day = days_from_civil(2026, 11, 10);
        let week = WeekId::of_day(day).expect("week");
        assert_eq!(week.number(), 202_646);
        assert_eq!(week.monday(), days_from_civil(2026, 11, 9));
        // 2021-01-03 (Sunday) still belongs to 2020's week 53.
        assert_eq!(
            WeekId::of_day(days_from_civil(2021, 1, 3)).map(WeekId::number),
            Some(202_053)
        );
        // 2024-12-30 (Monday) is week 1 of 2025.
        assert_eq!(
            WeekId::of_day(days_from_civil(2024, 12, 30)).map(WeekId::number),
            Some(202_501)
        );
        assert_eq!(weeks_in_year(2020), 53);
        assert_eq!(weeks_in_year(2026), 53);
        assert_eq!(weeks_in_year(2025), 52);
        assert!(WeekId::from_number(202_553).is_none());
        assert!(WeekId::from_number(202_600).is_none());
        assert!(WeekId::from_number(199_901).is_none());
        assert_eq!(WeekId::from_number(200_001).map(WeekId::index), Some(0));
        assert_eq!(WeekId::from_number(200_002).map(WeekId::index), Some(1));
        // Every day of a week names it, and weeks follow each other.
        let mut last = WeekId::of_day(days_from_civil(2000, 1, 3)).expect("first");
        for d in days_from_civil(2000, 1, 3)..days_from_civil(2040, 1, 1) {
            let w = WeekId::of_day(d).expect("week");
            if w != last {
                assert_eq!(w.index(), last.index() + 1, "{w}");
                assert_eq!(w.monday(), d);
                last = w;
            }
        }
    }

    #[test]
    fn seeds_differ_and_rules_go_round() {
        let a = WeekId::from_number(202_646).expect("a");
        let b = WeekId::from_number(202_647).expect("b");
        assert_ne!(a.seed(), b.seed());
        assert_eq!(a.seed(), WeekId::from_number(202_646).expect("a").seed());
        let rules: Vec<u32> = (0..8).collect();
        // Eight weeks of a round hold every rule once.
        let start = WeekId::from_number(200_001).expect("start");
        for round in 0..20 {
            let mut seen: Vec<u32> = (0..8)
                .map(|i| {
                    let days = start.monday() + (round * 8 + i) * 7;
                    *pick_rule(&rules, WeekId::of_day(days).expect("w")).expect("rule")
                })
                .collect();
            seen.sort_unstable();
            assert_eq!(seen, rules, "round {round}");
        }
        assert!(pick_rule::<u32>(&[], a).is_none());
    }
}

//! Die Wochen-Herausforderung (DESIGN 5.6): a run with the week's seed
//! and its Sonderregel ([`crate::content::WeeklyRuleDef`]).
//!
//! The rule is set when the run starts ([`RunStart::weekly_rule`]): its
//! character, start relics, start deck and Groschen go into the start, its
//! Verschärfungen are in force for the whole run (and stack with the
//! night's), its table rules fire like a boss's for every game. What it
//! draws at random (a malus for every pub, curses for a pub) comes from a
//! generator of its own, so every other draw of the run stays as without.

use serde::{Deserialize, Serialize};

use super::{Run, RunError, RunEvent, RunStart};
use crate::content::{Content, NightRule, WeekId, WeeklyRuleDef, WeeklyRuleId};
use crate::rng::Rng;

/// Salt of the weekly rule's generator ("WOCHE").
const WEEKLY_STREAM: u64 = 0x574F_4348_4500_0001;

/// The Sonderregel in force in a run.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WeeklyState {
    /// The week of a weekly run; `None` for a rule played on another seed
    /// (the simulator's `--weekly-rule`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub week: Option<WeekId>,
    /// The rule.
    pub rule: WeeklyRuleId,
    /// Draws of the rule (maluses, curses) outside the night.
    rng: Rng,
}

impl WeeklyState {
    pub(super) fn new(week: Option<WeekId>, rule: WeeklyRuleId, seed: u64) -> WeeklyState {
        WeeklyState {
            week,
            rule,
            rng: Rng::from_seed(seed ^ WEEKLY_STREAM),
        }
    }
}

/// The Sonderregel as the player sees it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct WeeklyView {
    /// The week as `JJJJWW`; `None` for a rule on another seed.
    pub week: Option<u32>,
    /// Id, also the key for the frontend's texts.
    pub id: WeeklyRuleId,
    /// German name.
    pub name: String,
    /// German rule text.
    pub text: String,
}

/// What a frontend shows of a week before its run starts.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct WeekInfo {
    /// The week as `JJJJWW`.
    pub week: u32,
    /// ISO year.
    pub year: u32,
    /// ISO week, from 1.
    pub iso_week: u32,
    /// The seed of the week's run, as decimal text.
    pub seed: String,
    /// The rule.
    pub rule: WeeklyView,
    /// The character everyone plays this week, if the rule fixes one.
    pub character: Option<crate::content::Character>,
}

/// The week's rule and seed, for the start screen. `None` if the content
/// has no rules.
#[must_use]
pub fn week_info(content: &Content, week: WeekId) -> Option<WeekInfo> {
    let rule = content.rule_of_week(week)?;
    Some(WeekInfo {
        week: week.number(),
        year: week.year(),
        iso_week: week.week(),
        seed: week.seed().to_string(),
        rule: rule_view(rule, Some(week)),
        character: rule.character,
    })
}

fn rule_view(rule: &WeeklyRuleDef, week: Option<WeekId>) -> WeeklyView {
    WeeklyView {
        week: week.map(WeekId::number),
        id: rule.id.clone(),
        name: rule.name.clone(),
        text: rule.text.clone(),
    }
}

impl Run {
    /// Start the run of `week` (Wochen-Herausforderung, DESIGN 5.6): the
    /// week's seed and Sonderregel, the rest of `start` as given
    /// (character, locked relics, stake). A rule with a character plays
    /// that one instead.
    pub fn new_weekly(
        content: &Content,
        week: WeekId,
        start: &RunStart,
    ) -> Result<(Run, Vec<RunEvent>), RunError> {
        let rule = content.rule_of_week(week).ok_or(RunError::InvalidStart)?;
        Run::new_with(
            content,
            week.seed(),
            &RunStart {
                weekly_rule: Some(rule.id.clone()),
                week: Some(week),
                ..start.clone()
            },
        )
    }

    /// The Sonderregel in force, if this is a weekly run.
    #[must_use]
    pub const fn weekly(&self) -> Option<&WeeklyState> {
        self.weekly.as_ref()
    }

    /// The definition of the Sonderregel in force.
    pub(super) fn weekly_def<'c>(&self, content: &'c Content) -> Option<&'c WeeklyRuleDef> {
        content.weekly_rule(&self.weekly.as_ref()?.rule)
    }

    /// The Sonderregel for a frontend.
    #[must_use]
    pub fn weekly_view(&self, content: &Content) -> Option<WeeklyView> {
        let state = self.weekly.as_ref()?;
        Some(rule_view(self.weekly_def(content)?, state.week))
    }

    /// Every Verschärfung in force: the Sonderregel's for the whole run,
    /// then the night's.
    pub(super) fn rules_in_force<'c>(&self, content: &'c Content) -> Vec<&'c NightRule> {
        self.weekly_def(content)
            .into_iter()
            .flat_map(|w| &w.rules)
            .chain(self.night_rules(content))
            .collect()
    }

    /// The generator for what the rules in force draw: the night's in the
    /// night, else the Sonderregel's; `None` without either.
    pub(super) fn rules_rng(&mut self) -> Option<&mut Rng> {
        rules_rng(self.night.as_mut(), self.weekly.as_mut())
    }
}

/// [`Run::rules_rng`] from the two fields, so that the rest of the run
/// stays readable while it is borrowed.
pub(super) fn rules_rng<'a>(
    night: Option<&'a mut super::night::NightState>,
    weekly: Option<&'a mut WeeklyState>,
) -> Option<&'a mut Rng> {
    match (night, weekly) {
        (Some(night), _) => Some(night.rng_mut()),
        (None, weekly) => weekly.map(|w| &mut w.rng),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cards::Card;
    use crate::content::{Character, RelicId, Source};

    fn content() -> Content {
        Content::load().expect("content")
    }

    fn week(n: u32) -> WeekId {
        WeekId::from_number(n).expect("week")
    }

    fn rule_week(content: &Content, id: &str) -> WeekId {
        (0..400)
            .map(|i| WeekId::of_day(week(202_601).monday() + i * 7).expect("w"))
            .find(|w| {
                content
                    .rule_of_week(*w)
                    .is_some_and(|r| r.id.as_str() == id)
            })
            .expect("a week with the rule")
    }

    fn with_rule(content: &Content, id: &str, start: &RunStart) -> Run {
        Run::new_with(
            content,
            42,
            &RunStart {
                weekly_rule: Some(WeeklyRuleId::new(id)),
                ..start.clone()
            },
        )
        .expect("start")
        .0
    }

    #[test]
    fn every_rule_starts_and_the_week_fixes_seed_and_rule() {
        let content = content();
        assert!(content.weekly.len() >= 8, "about eight rules");
        for id in content.weekly.keys() {
            let run = with_rule(&content, id.as_str(), &RunStart::default());
            assert_eq!(run.weekly().map(|w| &w.rule), Some(id));
            assert!(
                run.weekly_view(&content)
                    .is_some_and(|v| !v.text.is_empty())
            );
        }
        let w = week(202_646);
        let (a, _) = Run::new_weekly(&content, w, &RunStart::default()).expect("a");
        let (b, _) = Run::new_weekly(&content, w, &RunStart::default()).expect("b");
        assert_eq!(a, b);
        assert_eq!(a.seed(), w.seed());
        let info = week_info(&content, w).expect("info");
        assert_eq!(info.seed, w.seed().to_string());
        assert_eq!(Some(&info.rule.id), a.weekly().map(|s| &s.rule));
        assert_eq!(a.weekly_view(&content).and_then(|v| v.week), Some(202_646));
    }

    #[test]
    fn a_fixed_character_relics_deck_and_groschen_go_into_the_start() {
        let content = content();
        let null = with_rule(&content, "null_evening", &RunStart::default());
        assert_eq!(null.character(), Character::NullPlayer);
        let w = rule_week(&content, "null_evening");
        let info = week_info(&content, w).expect("info");
        assert_eq!(info.character, Some(Character::NullPlayer));

        let plain = Run::new(&content, 42).expect("plain").0;
        let pact = with_rule(&content, "devils_pact", &RunStart::default());
        assert!(pact.relics().contains(&RelicId::new("guardian_angel")));
        let nine = pact
            .edition()
            .id_of("C9".parse::<Card>().expect("card"))
            .expect("id");
        assert!(!pact.edition().get(nine).expect("card").mods.is_empty());
        assert!(plain.edition().get(nine).expect("card").mods.is_empty());
        // The deals stay those of the seed.
        let hand = |r: &Run| {
            r.game()
                .map(|g| g.game.hand(crate::game::Seat::DECLARER).len())
        };
        assert_eq!(hand(&plain), hand(&pact));

        let uncle = with_rule(&content, "rich_uncle", &RunStart::default());
        assert_eq!(uncle.coins(), plain.coins() + 15);
        assert!(uncle.series().target > plain.series().target);
    }

    #[test]
    fn verschaerfungen_of_the_rule_hold_outside_the_night() {
        let content = content();
        let plain = Run::new(&content, 42).expect("plain").0;
        let short = with_rule(&content, "short_patience", &RunStart::default());
        assert_eq!(
            short.redeals_left(&content) + 1,
            plain.redeals_left(&content)
        );
        let dive = with_rule(&content, "dive_week", &RunStart::default());
        assert!(
            dive.series().night_malus.is_some(),
            "a malus in the first pub"
        );
        assert!(plain.series().night_malus.is_none());
        let awake = with_rule(&content, "wide_awake", &RunStart::default());
        assert_eq!(awake.night_extra_samples(&content), 4);
    }

    #[test]
    fn grand_counts_double_by_the_rule() {
        let content = content();
        let run = with_rule(&content, "grand_double", &RunStart::default());
        let sources = run.sources(&content, false);
        assert!(
            sources
                .iter()
                .any(|s| s.source == Source::Weekly(WeeklyRuleId::new("grand_double")))
        );
        // A weekly run saves and loads with its rule.
        let json = serde_json::to_string(&run).expect("save");
        let back: Run = serde_json::from_str(&json).expect("load");
        assert_eq!(back, run);
        // A run without a rule saves no `weekly` key.
        let plain = Run::new(&content, 42).expect("plain").0;
        assert!(
            !serde_json::to_string(&plain)
                .expect("save")
                .contains("weekly")
        );
    }
}

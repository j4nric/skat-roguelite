//! "Die lange Nacht" (ADR 0020, DESIGN 5.5): the endless mode after der
//! Alte.
//!
//! A won run stays won ([`RunEvent::RunWon`] as before). From
//! [`RunPhase::Won`] the player may continue
//! ([`super::RunAction::EnterNight`]): Stadtteil 4, 5, … with the same deck,
//! relics, Groschen and character, until a pub is lost (the Schutzengel
//! still helps once). Night district n plays the Zielwerte of the last
//! district times `growth^(n − 3)` ([`crate::content::NightConfig`]), and
//! each night district adds one Verschärfung ([`NightRule`]), drawn with
//! the night's own generator without repeat until the list is exhausted,
//! before the district starts. Night bosses are the bosses of the day (der
//! Alte included) and those only of the night (`night_bosses` in
//! `run.ron`, "Der Totengräber"), without repeat until all were drawn. A won night boss
//! pub offers relics of the night pool ("Nachtschwärmer",
//! [`crate::content::RelicDef::night`]).
//!
//! Everything of the night lives in [`NightState`], so a normal run and its
//! saves are untouched; it draws only from its own generator, so the
//! normal run plays exactly as before.

use serde::{Deserialize, Serialize};

use super::{Run, RunError, RunEvent, RunPhase};
use crate::content::{
    BossId, Content, MalusId, NightRule, VerschaerfungDef, VerschaerfungId, in_pool,
};
use crate::rng::Rng;

/// The run counter of the Verschärfungen drawn, for relics that count them
/// ("Nachteule").
pub const VERSCHAERFUNGEN_COUNTER: &str = "verschaerfungen";

/// Salt of the night's generator ("NACHT").
const NIGHT_STREAM: u64 = 0x4E41_4348_5400_0000;

/// The long night in progress.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NightState {
    /// Verschärfungen in force, in the order drawn (one may come twice
    /// once the list was exhausted).
    pub verschaerfungen: Vec<VerschaerfungId>,
    /// Bosses of the night so far, in order (no repeat until all were
    /// drawn).
    #[serde(default)]
    pub bosses: Vec<BossId>,
    /// Further boss rules of the current night district's boss pub
    /// ([`NightRule::DoubleBoss`]).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub extra_bosses: Vec<BossId>,
    /// Pubs won in the night.
    #[serde(default)]
    pub pubs_won: u32,
    /// The night's own generator: Verschärfungen, bosses, curses, maluses.
    rng: Rng,
}

impl NightState {
    /// The night's generator (also for the Sonderregel's draws in the
    /// night, [`Run::rules_rng`]).
    pub(super) fn rng_mut(&mut self) -> &mut Rng {
        &mut self.rng
    }

    fn new(seed: u64) -> NightState {
        NightState {
            verschaerfungen: Vec::new(),
            bosses: Vec::new(),
            extra_bosses: Vec::new(),
            pubs_won: 0,
            rng: Rng::from_seed(seed ^ NIGHT_STREAM),
        }
    }
}

impl Run {
    /// The long night in progress, if the player continued after der
    /// Alte.
    #[must_use]
    pub const fn night(&self) -> Option<&NightState> {
        self.night.as_ref()
    }

    /// Whether the run is in the long night.
    #[must_use]
    pub const fn in_night(&self) -> bool {
        self.night.is_some()
    }

    /// The Verschärfungen in force, with their definitions, in the order
    /// drawn.
    #[must_use]
    pub fn verschaerfungen<'c>(&self, content: &'c Content) -> Vec<&'c VerschaerfungDef> {
        self.night
            .iter()
            .flat_map(|n| &n.verschaerfungen)
            .filter_map(|id| content.verschaerfungen.get(id))
            .collect()
    }

    /// The rules of the Verschärfungen in force.
    pub(super) fn night_rules<'c>(&self, content: &'c Content) -> Vec<&'c NightRule> {
        self.verschaerfungen(content)
            .into_iter()
            .map(|v| &v.rule)
            .collect()
    }

    /// Sum of a number the rules in force give: the night's and the
    /// Sonderregel der Woche's ([`Run::rules_in_force`]); 0 without
    /// either.
    pub(super) fn night_sum(&self, content: &Content, pick: impl Fn(&NightRule) -> u32) -> u32 {
        self.rules_in_force(content)
            .into_iter()
            .map(pick)
            .fold(0, u32::saturating_add)
    }

    /// How many Verschärfungen of this kind are in force.
    pub(super) fn night_count(
        &self,
        content: &Content,
        wanted: impl Fn(&NightRule) -> bool,
    ) -> u32 {
        self.night_sum(content, |r| u32::from(wanted(r)))
    }

    /// Einmischen per game the night takes away.
    pub(super) fn night_fewer_redeals(&self, content: &Content) -> u32 {
        self.night_sum(content, |r| match r {
            NightRule::FewerRedeals(n) => *n,
            _ => 0,
        })
    }

    /// Groschen the night adds to every price at the Tresen and Kiosk.
    pub(super) fn night_surcharge(&self, content: &Content) -> u32 {
        self.night_sum(content, |r| match r {
            NightRule::PricierTresen(n) => *n,
            _ => 0,
        })
    }

    /// Zinken the night takes off the card reward.
    pub(super) fn night_fewer_offers(&self, content: &Content) -> u32 {
        self.night_sum(content, |r| match r {
            NightRule::FewerCardOffers(n) => *n,
            _ => 0,
        })
    }

    /// Deals the night adds to the opponents' PIMC.
    #[must_use]
    pub fn night_extra_samples(&self, content: &Content) -> u32 {
        self.night_sum(content, |r| match r {
            NightRule::StrongerOpponents(n) => *n,
            _ => 0,
        })
    }

    /// The growth of the long night for the run's character
    /// ([`crate::content::CharacterDef::night`]).
    #[must_use]
    pub fn night_config(&self, content: &Content) -> crate::content::NightConfig {
        content.character(self.character).map_or_else(
            || content.run.night.clone(),
            |c| c.night(&content.run.night),
        )
    }

    /// Zielwert of pub `index` in district `section` with `boss`: the
    /// curve of the run ([`super::series_target`]); in a night district
    /// the last district's Zielwert grown by the night
    /// ([`crate::content::NightConfig::grow`]) and every "Zielwert-Sprung"
    /// on top. `None` if the pub does not exist.
    #[must_use]
    pub fn pub_target(
        &self,
        content: &Content,
        section: u32,
        index: u32,
        boss: Option<&BossId>,
    ) -> Option<u64> {
        let sections = content.run.sections();
        // A district of the run: only the Sonderregel der Woche jumps
        // (DESIGN 5.6); a night district: the night's rules too.
        let (mut target, rules) = if section < sections {
            let target = super::series_target(content, self.character, section, index, boss)?;
            let weekly: Vec<&NightRule> = self
                .weekly_def(content)
                .into_iter()
                .flat_map(|w| &w.rules)
                .collect();
            (target, weekly)
        } else {
            let last = sections.checked_sub(1)?;
            let base = super::series_target(content, self.character, last, index, boss)?;
            let target = self.night_config(content).grow(base, section - last);
            (target, self.rules_in_force(content))
        };
        for rule in rules {
            if let NightRule::TargetJump(factor) = rule {
                target = factor.apply(target);
            }
        }
        Some(target)
    }

    /// Whether district `section` can be played: one of the run's, or one
    /// of the night once it began.
    pub(super) fn district_exists(&self, content: &Content, section: u32) -> bool {
        section < content.run.sections() || self.night.is_some()
    }

    /// Number of districts the city names: the run's, in the night up to
    /// the one being played or chosen next.
    pub(super) fn city_districts(&self, content: &Content) -> u32 {
        let sections = content.run.sections();
        if self.night.is_none() {
            return sections;
        }
        let ahead = self
            .route
            .as_ref()
            .map(|r| r.section)
            .or(self.trip.as_ref().map(|t| t.section))
            .unwrap_or(self.series.section);
        sections.max(ahead.max(self.series.section) + 1)
    }

    /// [`super::RunAction::EnterNight`]: after der Alte, continue into the
    /// long night. The win is recorded already; the route to the first pub
    /// of Stadtteil 4 opens, with its Verschärfung drawn.
    pub(super) fn enter_night(
        &mut self,
        content: &Content,
        events: &mut Vec<RunEvent>,
    ) -> Result<(), RunError> {
        if self.phase != RunPhase::Won || self.night.is_some() {
            return Err(RunError::WrongPhase);
        }
        self.night = Some(NightState::new(self.seed));
        events.push(RunEvent::NightStarted {
            character: self.character,
        });
        self.open_route(content, events)
    }

    /// Before the route into a night district opens: draw its
    /// Verschärfung, one per night district, without repeat until every
    /// one was drawn ([`RunEvent::VerschaerfungDrawn`]).
    pub(super) fn draw_verschaerfung(
        &mut self,
        content: &Content,
        section: u32,
        events: &mut Vec<RunEvent>,
    ) {
        let sections = content.run.sections();
        let Some(night) = self.night.as_mut() else {
            return;
        };
        let wanted = usize::try_from(section.saturating_sub(sections) + 1).unwrap_or(0);
        let mut drawn = 0;
        while night.verschaerfungen.len() < wanted {
            let all: Vec<&VerschaerfungId> = content.verschaerfungen.keys().collect();
            if all.is_empty() {
                return;
            }
            // The round of the list in progress: what was drawn since the
            // list was last exhausted.
            let round = night.verschaerfungen.len() % all.len();
            let recent = &night.verschaerfungen[night.verschaerfungen.len() - round..];
            let fresh: Vec<&VerschaerfungId> = all
                .iter()
                .copied()
                .filter(|id| !recent.contains(id))
                .collect();
            let pick = night.rng.gen_range(0..fresh.len() as u64);
            let id = fresh[usize::try_from(pick).unwrap_or(0)].clone();
            night.verschaerfungen.push(id.clone());
            events.push(RunEvent::VerschaerfungDrawn {
                section,
                verschaerfung: id,
            });
            drawn += 1;
        }
        // The run counter "Nachteule" counts ([`VERSCHAERFUNGEN_COUNTER`]).
        if drawn > 0 {
            let counter = self
                .counters
                .entry(VERSCHAERFUNGEN_COUNTER.to_owned())
                .or_insert(0);
            *counter = counter.saturating_add(drawn);
        }
    }

    /// The boss of a night district: one of the bosses of the run, the
    /// Endboss and the bosses of the night (`night_bosses`), for this
    /// character, without repeat until all were drawn;
    /// with [`NightRule::DoubleBoss`] further boss rules for its pub.
    pub(super) fn draw_night_boss(&mut self, content: &Content) -> Option<BossId> {
        let doubles = self.night_count(content, |r| matches!(r, NightRule::DoubleBoss));
        let character = self.character;
        let night = self.night.as_mut()?;
        let mut pool: Vec<BossId> = content
            .run
            .bosses
            .iter()
            .chain(content.run.final_boss.iter())
            .chain(content.run.night_bosses.iter())
            .filter(|id| {
                content
                    .boss(id)
                    .is_some_and(|b| in_pool(&b.characters, character))
            })
            .cloned()
            .collect();
        pool.dedup();
        if pool.is_empty() {
            return None;
        }
        let round = night.bosses.len() % pool.len();
        let recent = &night.bosses[night.bosses.len() - round..];
        let fresh: Vec<BossId> = pool
            .iter()
            .filter(|b| !recent.contains(b))
            .cloned()
            .collect();
        let pick = night.rng.gen_range(0..fresh.len() as u64);
        let boss = fresh[usize::try_from(pick).unwrap_or(0)].clone();
        night.bosses.push(boss.clone());
        // Further rules: another boss each, not the drawn one, none that
        // rotates rules itself (der Alte).
        let mut others: Vec<BossId> = pool
            .into_iter()
            .filter(|b| *b != boss)
            .filter(|b| content.boss(b).is_some_and(|d| d.rotation.is_empty()))
            .collect();
        night.extra_bosses.clear();
        for _ in 0..doubles {
            if others.is_empty() {
                break;
            }
            let pick = night.rng.gen_range(0..others.len() as u64);
            night
                .extra_bosses
                .push(others.remove(usize::try_from(pick).unwrap_or(0)));
        }
        Some(boss)
    }

    /// Further boss rules of the boss pub in progress (night only).
    pub(super) fn extra_boss_rules(&self) -> &[BossId] {
        match &self.night {
            Some(night) if self.series.boss.is_some() => &night.extra_bosses,
            _ => &[],
        }
    }

    /// [`NightRule::MalusEveryPub`]: a random Spelunke malus for a normal
    /// pub of the night (a Spelunke keeps its own, a boss pub has none).
    pub(super) fn draw_night_malus(
        &mut self,
        content: &Content,
        boss_pub: bool,
    ) -> Option<MalusId> {
        let wanted = self.night_count(content, |r| matches!(r, NightRule::MalusEveryPub));
        if wanted == 0 || boss_pub || self.series.spelunke.is_some() {
            return None;
        }
        let character = self.character;
        let rng = self.rules_rng()?;
        let pool: Vec<(MalusId, u32)> = content
            .maluses
            .values()
            .filter(|m| m.weight > 0 && in_pool(&m.characters, character))
            .map(|m| (m.id.clone(), m.weight))
            .collect();
        super::draw_weighted(rng, pool, 1).pop()
    }

    /// [`NightRule::CursedCards`]: at the start of a pub, random cards of
    /// the player's edition carry a random curse of the rule until the pub
    /// ends (kept like the Oma's, [`super::Series::boss_curses`]).
    pub(super) fn lay_night_curses(&mut self, content: &Content, events: &mut Vec<RunEvent>) {
        let rules: Vec<(u32, Vec<crate::content::CardModId>)> = self
            .rules_in_force(content)
            .into_iter()
            .filter_map(|r| match r {
                NightRule::CursedCards { cards, curses } => Some((*cards, curses.clone())),
                _ => None,
            })
            .collect();
        if rules.is_empty() {
            return;
        }
        let Some(rng) = super::weekly::rules_rng(self.night.as_mut(), self.weekly.as_mut()) else {
            return;
        };
        let mut pool: Vec<u32> = self
            .edition
            .cards()
            .iter()
            .filter(|c| c.kind != crate::deck::CardKind::Blank)
            .filter(|c| !self.series.boss_curses.iter().any(|b| b.id == c.id))
            .map(|c| c.id)
            .collect();
        let mut laid = Vec::new();
        for (cards, curses) in rules {
            for _ in 0..cards {
                if pool.is_empty() {
                    break;
                }
                let pick = rng.gen_range(0..pool.len() as u64);
                let card = pool.remove(usize::try_from(pick).unwrap_or(0));
                let kind = self.edition.get(card).map(|c| c.kind);
                let fitting: Vec<&crate::content::CardModId> = curses
                    .iter()
                    .filter(|m| {
                        content
                            .card_mod(m)
                            .is_some_and(|d| kind.is_some_and(|k| d.fits(k)))
                    })
                    .collect();
                if fitting.is_empty() {
                    continue;
                }
                let which = rng.gen_range(0..fitting.len() as u64);
                let curse = fitting[usize::try_from(which).unwrap_or(0)].clone();
                let value = content
                    .card_mod(&curse)
                    .map_or(1, crate::content::CardModDef::default_value);
                if let Ok(before) = self.edition.lay_temp_curse(content, card, &curse, value) {
                    self.series.boss_curses.push(before);
                    laid.push((card, curse));
                }
            }
        }
        if !laid.is_empty() {
            laid.sort();
            events.push(RunEvent::NightCursesLaid { cards: laid });
        }
    }
}

/// A Verschärfung as the player sees it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct VerschaerfungInfo {
    /// Id, also the key for the frontend's texts.
    pub id: VerschaerfungId,
    /// German name.
    pub name: String,
    /// German rule text.
    pub text: String,
    /// The night district it was drawn for, from 1 (4 = Stadtteil 4).
    pub district: u32,
}

/// The long night for a frontend (ADR 0020).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct NightView {
    /// The night district being played or chosen next, from 1 (4 =
    /// Stadtteil 4).
    pub district: u32,
    /// The Verschärfungen in force, in the order drawn.
    pub verschaerfungen: Vec<VerschaerfungInfo>,
    /// The Verschärfung of the district ahead while its first pub is
    /// still to come (route choice into a new night district, the way
    /// there): what the district start sheet shows.
    pub announced: Option<VerschaerfungInfo>,
    /// Further boss rules of the night boss pub of this district
    /// ("Doppelte Bosse").
    pub extra_bosses: Vec<super::view::BossInfo>,
    /// Pubs won in the night.
    pub pubs_won: u32,
    /// Factor of the night's Zielwerte against Stadtteil 3 in this
    /// district, in thousandths (growth and "Zielwert-Sprung").
    pub target_permille: u64,
}

//! What lasts across runs (DESIGN 9.3): unlocks (Freischaltungen) and the
//! statistics of finished runs.
//!
//! The meta state lives next to the run, never inside it: a frontend
//! stores it apart from the running run (in the browser, `localStorage`).
//! It learns from the run's events ([`MetaState::observe`]), so it works
//! with the full [`RunEvent`]s of the simulator as well as with the
//! [`super::player::PlayerEvent`]s the browser gets; it only reads events
//! that are public anyway.
//!
//! Unlocks are data (`unlocks.ron`): one condition, its rewards. A relic
//! or character named as a reward is locked until the unlock is earned.
//! [`MetaState::run_start`] turns the state into a [`RunStart`] with the
//! locked relics, so the Tresen only offers what is unlocked.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use serde::{Deserialize, Serialize};

use super::{RunEvent, RunStart};
use crate::content::{
    CardModId, Character, Content, Rarity, RelicId, RelicKind, Reward, ScoreBreakdown,
    UnlockCondition, UnlockId,
};
use crate::rules::Declaration;

/// Version of the serialized meta state. A frontend rejects other versions.
pub const META_VERSION: u32 = 1;

/// Statistics of one character.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct CharacterStats {
    /// Runs finished (won or lost).
    pub runs: u32,
    /// Runs won.
    pub won: u32,
    /// Highest section reached (from 1), 0 before the first run.
    pub best_section: u32,
}

/// Statistics over all finished runs.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct MetaStats {
    /// Runs finished (won or lost).
    pub runs_finished: u32,
    /// Runs won.
    pub runs_won: u32,
    /// Highest section reached (from 1).
    pub best_section: u32,
    /// Games won.
    pub games_won: u32,
    /// Games lost.
    pub games_lost: u32,
    /// Highest Spielwert of a won game.
    pub best_game: u64,
    /// Per character.
    pub characters: BTreeMap<Character, CharacterStats>,
}

/// The meta state: unlocks earned and statistics.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct MetaState {
    /// Format version ([`META_VERSION`]).
    #[serde(default = "meta_version")]
    pub version: u32,
    /// Unlocks earned.
    #[serde(default)]
    pub unlocked: BTreeSet<UnlockId>,
    /// Statistics.
    #[serde(default)]
    pub stats: MetaStats,
    /// Section reached by the run in progress (from 1), for the
    /// statistics when it ends.
    #[serde(default)]
    current_section: u32,
}

const fn meta_version() -> u32 {
    META_VERSION
}

/// Why a run cannot start from this meta state.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MetaError {
    /// The character is not unlocked yet.
    CharacterLocked(Character),
    /// The content has no such character.
    UnknownCharacter(Character),
}

impl fmt::Display for MetaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MetaError::CharacterLocked(c) => write!(f, "character {c} is locked"),
            MetaError::UnknownCharacter(c) => write!(f, "unknown character {c}"),
        }
    }
}

impl std::error::Error for MetaError {}

impl MetaError {
    /// Stable code for frontends: `CharacterLocked` or `UnknownCharacter`.
    #[must_use]
    pub const fn code(&self) -> &'static str {
        match self {
            MetaError::CharacterLocked(_) => "CharacterLocked",
            MetaError::UnknownCharacter(_) => "UnknownCharacter",
        }
    }
}

/// A character as the menu shows it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CharacterEntry {
    /// Which character.
    pub character: Character,
    /// German name.
    pub name: String,
    /// German description.
    pub text: String,
    /// Whether it can be chosen.
    pub unlocked: bool,
    /// The unlock that opens it, if it is locked.
    pub unlocked_by: Option<UnlockId>,
}

/// What kind of thing a reward is, for the menu.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
pub enum RewardKind {
    /// A house rule (Hausregel).
    HouseRule,
    /// A Stammtisch figure (Figur).
    Regular,
    /// A card modification (Kartenmodifikation).
    CardMod,
    /// A character.
    Character,
}

/// A reward as the menu shows it: what it is, its name and rarity.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RewardInfo {
    /// The reward itself.
    pub reward: Reward,
    /// House rule, figure, card modification or character.
    pub kind: RewardKind,
    /// German name from the content.
    pub name: String,
    /// German text from the content.
    pub text: String,
    /// Rarity of a relic or card modification; `None` for a character.
    pub rarity: Option<Rarity>,
}

/// An unlock as the menu shows it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct UnlockEntry {
    /// Id.
    pub id: UnlockId,
    /// German name.
    pub name: String,
    /// German description of the deed.
    pub text: String,
    /// Whether it was earned.
    pub unlocked: bool,
    /// What it opens.
    pub rewards: Vec<Reward>,
    /// The rewards with kind, name and rarity, in the same order.
    pub reward_info: Vec<RewardInfo>,
}

/// Everything a frontend shows of the meta state.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct MetaView {
    /// Characters, unlocked or not.
    pub characters: Vec<CharacterEntry>,
    /// Every unlock in id order.
    pub unlocks: Vec<UnlockEntry>,
    /// Relics still locked.
    pub locked_relics: Vec<RelicId>,
    /// Card modifications still locked.
    pub locked_card_mods: Vec<CardModId>,
    /// Statistics.
    pub stats: MetaStats,
}

impl MetaState {
    /// A new player: nothing earned.
    #[must_use]
    pub fn new() -> MetaState {
        MetaState {
            version: META_VERSION,
            ..MetaState::default()
        }
    }

    /// Everything unlocked (simulator, tests).
    #[must_use]
    pub fn all_unlocked(content: &Content) -> MetaState {
        MetaState {
            unlocked: content.unlocks.keys().cloned().collect(),
            ..MetaState::new()
        }
    }

    /// Whether this unlock was earned.
    #[must_use]
    pub fn is_unlocked(&self, id: &UnlockId) -> bool {
        self.unlocked.contains(id)
    }

    /// The unlock that rewards `reward`, if any.
    fn unlock_for<'c>(content: &'c Content, reward: &Reward) -> Option<&'c UnlockId> {
        content
            .unlocks
            .values()
            .find(|u| u.rewards.contains(reward))
            .map(|u| &u.id)
    }

    /// Relics that are a reward of an unlock not yet earned, in id order.
    #[must_use]
    pub fn locked_relics(&self, content: &Content) -> Vec<RelicId> {
        let mut locked: Vec<RelicId> = content
            .unlocks
            .values()
            .filter(|u| !self.unlocked.contains(&u.id))
            .flat_map(|u| u.rewards.iter())
            .filter_map(|r| match r {
                Reward::Relic(id) => Some(id.clone()),
                Reward::Character(_) | Reward::CardMod(_) => None,
            })
            .collect();
        locked.sort();
        locked.dedup();
        locked
    }

    /// Card modifications that are a reward of an unlock not yet earned,
    /// in id order.
    #[must_use]
    pub fn locked_card_mods(&self, content: &Content) -> Vec<CardModId> {
        let mut locked: Vec<CardModId> = content
            .unlocks
            .values()
            .filter(|u| !self.unlocked.contains(&u.id))
            .flat_map(|u| u.rewards.iter())
            .filter_map(|r| match r {
                Reward::CardMod(id) => Some(id.clone()),
                Reward::Relic(_) | Reward::Character(_) => None,
            })
            .collect();
        locked.sort();
        locked.dedup();
        locked
    }

    /// Whether `character` can be chosen: it exists and is no reward of an
    /// unlock not yet earned.
    #[must_use]
    pub fn character_unlocked(&self, content: &Content, character: Character) -> bool {
        content.character(character).is_some()
            && Self::unlock_for(content, &Reward::Character(character))
                .is_none_or(|id| self.unlocked.contains(id))
    }

    /// The start of a run with `character`: its locked relics stay off the
    /// Tresen.
    pub fn run_start(
        &self,
        content: &Content,
        character: Character,
    ) -> Result<RunStart, MetaError> {
        if content.character(character).is_none() {
            return Err(MetaError::UnknownCharacter(character));
        }
        if !self.character_unlocked(content, character) {
            return Err(MetaError::CharacterLocked(character));
        }
        Ok(RunStart {
            character,
            locked_relics: self.locked_relics(content),
            locked_card_mods: self.locked_card_mods(content),
            ..RunStart::default()
        })
    }

    /// Learn from run events: earn unlocks and keep the statistics. Returns
    /// the unlocks earned by these events, in id order. Feed every event of
    /// a run exactly once, in order (the events of [`super::Run::new`]
    /// included).
    pub fn observe<G>(&mut self, content: &Content, events: &[RunEvent<G>]) -> Vec<UnlockId> {
        let mut earned = BTreeSet::new();
        for event in events {
            self.count(event);
            for def in content.unlocks.values() {
                if self.unlocked.contains(&def.id) || earned.contains(&def.id) {
                    continue;
                }
                if self.met(&def.condition, event) {
                    earned.insert(def.id.clone());
                }
            }
        }
        self.unlocked.extend(earned.iter().cloned());
        earned.into_iter().collect()
    }

    /// Statistics of one event.
    fn count<G>(&mut self, event: &RunEvent<G>) {
        let stats = &mut self.stats;
        match event {
            RunEvent::SeriesStarted { section, .. } => {
                self.current_section = section + 1;
                stats.best_section = stats.best_section.max(section + 1);
            }
            RunEvent::GameScored { breakdown, .. } => {
                if breakdown.won {
                    stats.games_won += 1;
                    stats.best_game = stats.best_game.max(breakdown.value);
                } else {
                    stats.games_lost += 1;
                }
            }
            RunEvent::RunWon { character } => {
                stats.runs_finished += 1;
                stats.runs_won += 1;
                let entry = stats.characters.entry(*character).or_default();
                entry.runs += 1;
                entry.won += 1;
                entry.best_section = entry.best_section.max(self.current_section);
                stats.best_section = stats.best_section.max(self.current_section);
                self.current_section = 0;
            }
            RunEvent::RunLost {
                section, character, ..
            } => {
                stats.runs_finished += 1;
                let entry = stats.characters.entry(*character).or_default();
                entry.runs += 1;
                entry.best_section = entry.best_section.max(section + 1);
                stats.best_section = stats.best_section.max(section + 1);
                self.current_section = 0;
            }
            _ => {}
        }
    }

    /// Whether `event` fulfils `condition`.
    fn met<G>(&self, condition: &UnlockCondition, event: &RunEvent<G>) -> bool {
        match (condition, event) {
            (
                UnlockCondition::WinGame {
                    game,
                    schneider,
                    min_risk,
                    min_value,
                },
                RunEvent::GameScored { breakdown, .. },
            ) => won_game(breakdown, *game, *schneider, *min_risk, *min_value),
            (UnlockCondition::WinNull(min), RunEvent::GameScored { breakdown, .. }) => {
                breakdown.won && breakdown.declaration.null_risk().is_some_and(|r| r >= *min)
            }
            (UnlockCondition::ReachSection(n), RunEvent::SeriesStarted { section, .. }) => {
                section + 1 >= *n
            }
            (
                UnlockCondition::BeatBoss { boss, without_pass },
                RunEvent::SeriesWon {
                    boss: Some(beaten),
                    passes_used,
                    ..
                },
            ) => boss.as_ref().is_none_or(|b| b == beaten) && (!without_pass || *passes_used == 0),
            (
                UnlockCondition::WinSeriesWithGamesLeft(n),
                RunEvent::SeriesWon { games_left, .. },
            ) => games_left >= n,
            (UnlockCondition::WinRun(who), RunEvent::RunWon { character }) => {
                who.is_none_or(|c| c == *character)
            }
            (
                UnlockCondition::RunsFinished(n),
                RunEvent::RunWon { .. } | RunEvent::RunLost { .. },
            ) => self.stats.runs_finished >= *n,
            _ => false,
        }
    }

    /// The meta state for a frontend.
    #[must_use]
    pub fn view(&self, content: &Content) -> MetaView {
        MetaView {
            characters: content
                .characters
                .values()
                .map(|def| {
                    let unlocked = self.character_unlocked(content, def.character);
                    CharacterEntry {
                        character: def.character,
                        name: def.name.clone(),
                        text: def.text.clone(),
                        unlocked,
                        unlocked_by: if unlocked {
                            None
                        } else {
                            Self::unlock_for(content, &Reward::Character(def.character)).cloned()
                        },
                    }
                })
                .collect(),
            unlocks: content
                .unlocks
                .values()
                .map(|def| UnlockEntry {
                    id: def.id.clone(),
                    name: def.name.clone(),
                    text: def.text.clone(),
                    unlocked: self.unlocked.contains(&def.id),
                    rewards: def.rewards.clone(),
                    reward_info: def
                        .rewards
                        .iter()
                        .map(|r| reward_info(content, r))
                        .collect(),
                })
                .collect(),
            locked_relics: self.locked_relics(content),
            locked_card_mods: self.locked_card_mods(content),
            stats: self.stats.clone(),
        }
    }
}

/// Kind, name and rarity of a reward from the content.
fn reward_info(content: &Content, reward: &Reward) -> RewardInfo {
    let (kind, name, text, rarity) = match reward {
        Reward::Relic(id) => content.relic(id).map_or_else(
            || (RewardKind::HouseRule, id.to_string(), String::new(), None),
            |def| {
                let kind = match def.kind {
                    RelicKind::HouseRule => RewardKind::HouseRule,
                    RelicKind::Regular => RewardKind::Regular,
                };
                (kind, def.name.clone(), def.text.clone(), Some(def.rarity))
            },
        ),
        Reward::CardMod(id) => content.card_mod(id).map_or_else(
            || (RewardKind::CardMod, id.to_string(), String::new(), None),
            |def| {
                (
                    RewardKind::CardMod,
                    def.name.clone(),
                    def.text.clone(),
                    Some(def.rarity),
                )
            },
        ),
        Reward::Character(c) => content.character(*c).map_or_else(
            || (RewardKind::Character, c.to_string(), String::new(), None),
            |def| {
                (
                    RewardKind::Character,
                    def.name.clone(),
                    def.text.clone(),
                    None,
                )
            },
        ),
    };
    RewardInfo {
        reward: reward.clone(),
        kind,
        name,
        text,
        rarity,
    }
}

/// Whether a scored game is a won suit game or grand with these
/// properties.
fn won_game(
    breakdown: &ScoreBreakdown,
    game: Option<crate::rules::GameType>,
    schneider: bool,
    min_risk: Option<crate::rules::RiskLevel>,
    min_value: u64,
) -> bool {
    let declaration = breakdown.declaration;
    let trump_game = matches!(
        declaration,
        Declaration::Suit { .. } | Declaration::Grand { .. }
    );
    breakdown.won
        && trump_game
        && game.is_none_or(|g| declaration.game() == g)
        && (!schneider || breakdown.schneider)
        && min_risk.is_none_or(|m| declaration.risk().is_some_and(|r| r >= m))
        && breakdown.value >= min_value
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::run::bot::RunBot;
    use crate::run::{Run, RunPhase};

    fn content() -> Content {
        Content::load().expect("content")
    }

    #[test]
    fn a_new_player_has_half_the_relics_locked_and_no_null_player() {
        let content = content();
        let meta = MetaState::new();
        let locked = meta.locked_relics(&content);
        assert!(!locked.is_empty());
        let relics = content
            .relics
            .values()
            .filter(|r| r.characters.is_empty() || r.characters.contains(&Character::Declarer))
            .count();
        // About half of the house rules and figures (DESIGN 9.3).
        assert!(locked.len() * 3 >= relics, "{locked:?}");
        assert!(locked.len() * 3 <= relics * 2, "{locked:?}");
        assert!(meta.character_unlocked(&content, Character::Declarer));
        assert!(!meta.character_unlocked(&content, Character::NullPlayer));
        assert_eq!(
            meta.run_start(&content, Character::NullPlayer),
            Err(MetaError::CharacterLocked(Character::NullPlayer))
        );
        let start = meta
            .run_start(&content, Character::Declarer)
            .expect("declarer");
        assert_eq!(start.locked_relics, locked);
        let all = MetaState::all_unlocked(&content);
        assert!(all.locked_relics(&content).is_empty());
        assert!(all.character_unlocked(&content, Character::NullPlayer));
    }

    #[test]
    fn the_tresen_offers_no_locked_relic() {
        let content = content();
        let meta = MetaState::new();
        let locked = meta.locked_relics(&content);
        let start = meta
            .run_start(&content, Character::Declarer)
            .expect("start");
        for seed in 0..40 {
            let (mut run, _) = Run::new_with(&content, seed, &start).expect("run");
            for _ in 0..3_000 {
                if let Some(shop) = run.shop() {
                    for relic in shop.relics.iter().flatten() {
                        assert!(!locked.contains(relic), "{relic} is locked");
                    }
                }
                let Some(action) = RunBot.next_action(&run, &content) else {
                    break;
                };
                run.apply(&content, action).expect("bot");
            }
        }
    }

    #[test]
    fn events_earn_unlocks_and_count_runs() {
        let content = content();
        let mut meta = MetaState::new();
        let start = meta
            .run_start(&content, Character::Declarer)
            .expect("start");
        let mut earned = Vec::new();
        let mut runs = 0;
        for seed in 0..30 {
            let (mut run, mut events) = Run::new_with(&content, seed, &start).expect("run");
            while let Some(action) = RunBot.next_action(&run, &content) {
                events.extend(run.apply(&content, action).expect("bot"));
            }
            assert!(matches!(run.phase(), RunPhase::Won | RunPhase::Lost));
            // Player events give the same result as the full events.
            let mut twin = meta.clone();
            let player = crate::run::player::player_events(events.clone());
            let a = meta.observe(&content, &events);
            let b = twin.observe(&content, &player);
            assert_eq!(a, b);
            assert_eq!(meta, twin);
            earned.extend(a);
            runs += 1;
        }
        assert_eq!(meta.stats.runs_finished, runs);
        let declarer = meta.stats.characters[&Character::Declarer];
        assert_eq!(declarer.runs, runs);
        assert!(meta.stats.best_section >= 1);
        assert!(
            earned.contains(&UnlockId::new("second_evening")),
            "30 runs reach section 2: {earned:?}"
        );
        assert!(meta.is_unlocked(&UnlockId::new("second_evening")));
        assert!(
            !meta
                .locked_relics(&content)
                .contains(&RelicId::new("landlord"))
        );
        // Serializable and stable.
        let text = ron::to_string(&meta).expect("ron");
        let back: MetaState = ron::from_str(&text).expect("ron");
        assert_eq!(back, meta);
        let view = meta.view(&content);
        assert_eq!(view.unlocks.len(), content.unlocks.len());
        // The Nullspieler is unlocked by a won run. Since the v1 balancing
        // (docs/balance/2026-11-01-m5-v1.md) the bot may win one of the 30
        // runs, so the view must match whichever happened.
        let tour_won = meta.is_unlocked(&UnlockId::new("tour_won"));
        let expected = (!tour_won).then(|| UnlockId::new("tour_won"));
        assert!(
            view.characters
                .iter()
                .any(|c| c.character == Character::NullPlayer
                    && c.unlocked == tour_won
                    && c.unlocked_by == expected)
        );
    }

    #[test]
    fn winning_a_run_unlocks_the_null_player() {
        let content = content();
        let mut meta = MetaState::new();
        let events: Vec<RunEvent> = vec![RunEvent::RunWon {
            character: Character::Declarer,
        }];
        let earned = meta.observe(&content, &events);
        assert!(earned.contains(&UnlockId::new("tour_won")));
        assert!(meta.character_unlocked(&content, Character::NullPlayer));
        assert!(meta.run_start(&content, Character::NullPlayer).is_ok());
        assert_eq!(meta.stats.runs_won, 1);
    }

    #[test]
    fn the_end_of_a_run_raises_the_best_section() {
        let content = content();
        // A meta state that missed the series starts (fed from the end of
        // a run only) still counts the section the run ended in.
        let mut meta = MetaState::new();
        let events: Vec<RunEvent> = vec![RunEvent::RunLost {
            section: 3,
            index: 1,
            character: Character::Declarer,
        }];
        meta.observe(&content, &events);
        assert_eq!(meta.stats.best_section, 4);
        assert_eq!(meta.stats.characters[&Character::Declarer].best_section, 4);
        let mut meta = MetaState::new();
        meta.current_section = 5;
        let events: Vec<RunEvent> = vec![RunEvent::RunWon {
            character: Character::Declarer,
        }];
        meta.observe(&content, &events);
        assert_eq!(meta.stats.best_section, 5);
    }
}

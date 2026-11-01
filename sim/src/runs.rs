//! Runs mode: complete runs with the simple run policy.
//!
//! Statistics: section reached, win rate per series, Groschen, and for
//! every relic how often it was bought and how runs with it went.
//!
//! Options (all reproducible with the same seed, see [`Options`]):
//! `--no-shop`, `--relic <id>` (repeatable), `--without <id>`
//! (repeatable), `--boss <id>`, `--character declarer|null`,
//! `--null-scoring fixed|formula`, `--curve` (success curve of a single
//! series instead of whole runs, see [`curve`]) and `--unlocks`
//! (reachability of every unlock, see [`unlocks`]).
//!
//! Runs start with everything unlocked (no relic is locked), so that the
//! measurement does not depend on a player's progress. `--unlocks` plays
//! with a growing meta state instead.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::process::ExitCode;
use std::time::Instant;

use skat_core::ai::pimc::Opponents;
use skat_core::content::{BossId, Character, Content, EncounterId, RelicId, UnlockId};
use skat_core::rng::Rng;
use skat_core::rules::{NullFormula, NullScoring};
use skat_core::run::bot::RunBot;
use skat_core::run::meta::MetaState;
use skat_core::run::{
    EncounterChoice, Run, RunAction, RunEvent, RunPhase, RunStart, SERIES_PER_SECTION,
};

/// Measurement options of the runs mode.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Options {
    /// The bot never buys anything: it leaves every Tresen at once.
    pub no_shop: bool,
    /// Relics owned from the start of every run, with their passive
    /// effects already in the first game ([`RunStart::relics`]).
    pub relics: Vec<RelicId>,
    /// Relics taken out of the content: never offered, so never bought.
    pub without: Vec<RelicId>,
    /// The only boss a boss series may draw.
    pub boss: Option<BossId>,
    /// Measure the success curve of one series instead of whole runs.
    pub curve: bool,
    /// Who plays.
    pub character: Character,
    /// Override of the character's Null scoring.
    pub null_scoring: Option<NullScoring>,
    /// Measure how many runs it takes to earn every unlock.
    pub unlocks: bool,
    /// The player's card play (`--bot`): heuristic or PIMC.
    pub bot: Opponents,
    /// The defenders' card play in every section (`--defenders`), instead
    /// of the per-section list of `run.ron`.
    pub defenders: Option<Opponents>,
}

/// The Null formula of DESIGN 9.2 with the values of `characters.ron`, for
/// `--null-scoring formula`.
#[must_use]
pub fn formula_scoring() -> NullScoring {
    NullScoring::Formula(NullFormula {
        base_value: 23,
        per_seven_ducked: 1,
        never_followed: 2,
        short_circuit: 3,
        short_circuit_tricks: 5,
        short_circuit_points: 61,
    })
}

impl Options {
    /// The content with `--without` and `--boss` applied. Fails on unknown
    /// ids.
    pub fn content(&self, base: &Content) -> Result<Content, String> {
        let mut content = base.clone();
        for id in &self.relics {
            if content.relic(id).is_none() {
                return Err(format!("--relic: unknown relic {id}"));
            }
        }
        for id in &self.without {
            if content.relics.remove(id).is_none() {
                return Err(format!("--without: unknown relic {id}"));
            }
            if self.relics.contains(id) {
                return Err(format!("--without {id} and --relic {id} contradict"));
            }
        }
        if let Some(boss) = &self.boss {
            if content.boss(boss).is_none() {
                return Err(format!("--boss: unknown boss {boss}"));
            }
            content.run.bosses = vec![boss.clone()];
            content.run.final_boss = None;
        }
        if let Some(scoring) = &self.null_scoring {
            let def = content
                .characters
                .get_mut(&Character::NullPlayer)
                .ok_or("--null-scoring: no Nullspieler in the content")?;
            def.null_scoring = Some(scoring.clone());
        }
        if content.character(self.character).is_none() {
            return Err(format!("--character: unknown {}", self.character));
        }
        if let Some(defenders) = self.defenders {
            content.run.opponents = vec![defenders];
        }
        Ok(content)
    }

    /// How every measured run starts.
    fn start(&self) -> RunStart {
        RunStart {
            relics: self.relics.clone(),
            character: self.character,
            ..RunStart::default()
        }
    }

    /// The options as one line for the report header.
    fn describe(&self) -> String {
        let mut parts = Vec::new();
        if self.no_shop {
            parts.push("no-shop".to_string());
        }
        for id in &self.relics {
            parts.push(format!("relic={id}"));
        }
        for id in &self.without {
            parts.push(format!("without={id}"));
        }
        if let Some(boss) = &self.boss {
            parts.push(format!("boss={boss}"));
        }
        if self.character != Character::Declarer {
            parts.push(format!("character={}", self.character));
        }
        match &self.null_scoring {
            Some(NullScoring::Fixed) => parts.push("null-scoring=fixed".into()),
            Some(NullScoring::Formula(_)) => parts.push("null-scoring=formula".into()),
            None => {}
        }
        if let Opponents::Pimc { samples, depth } = self.bot {
            parts.push(format!("bot=pimc:{samples}:{depth}"));
        }
        match self.defenders {
            Some(Opponents::Heuristic) => parts.push("defenders=heuristic".into()),
            Some(Opponents::Pimc { samples, depth }) => {
                parts.push(format!("defenders=pimc:{samples}:{depth}"));
            }
            None => {}
        }
        if parts.is_empty() {
            "standard".to_string()
        } else {
            parts.join(" ")
        }
    }

    /// The bot's next action under these options.
    fn next_action(&self, run: &Run, content: &Content) -> Option<RunAction> {
        if self.no_shop && run.phase() == RunPhase::Shop {
            return Some(RunAction::LeaveShop);
        }
        let action = RunBot.next_action(run, content)?;
        if let (RunAction::Play(_), Some(config)) = (&action, self.bot.pimc()) {
            if let Some(card) = skat_core::run::opponents::player_card(run, config) {
                return Some(RunAction::Play(card.id));
            }
        }
        Some(action)
    }
}

/// Upper bound of actions per run, a guard against a policy that loops.
const MAX_ACTIONS: usize = 1_000_000;

/// How runs with one relic went.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct RelicStat {
    /// Times bought (a relic sold and bought again counts twice).
    pub bought: u64,
    /// Runs in which it was bought at least once.
    pub runs: u64,
    /// Of those, runs won.
    pub runs_won: u64,
    /// Sum of the sections reached (1-based) by those runs.
    pub sections: u64,
}

/// Aggregate over all runs.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RunStats {
    /// Base seed.
    pub seed: u64,
    /// Number of runs.
    pub runs: u32,
    /// Lost runs by section reached (index 0 = section 1).
    pub lost_in_section: Vec<u64>,
    /// Runs won.
    pub won: u64,
    /// Attempts and wins per (section, series).
    pub series: Vec<[(u64, u64); 3]>,
    /// Attempts and wins per boss in section 1.
    pub bosses: BTreeMap<BossId, (u64, u64)>,
    /// Attempts and wins per boss from section 2 on.
    pub bosses_later: BTreeMap<BossId, (u64, u64)>,
    /// Games played (not passed).
    pub games: u64,
    /// Games won.
    pub games_won: u64,
    /// Sum of the series account changes of all games.
    pub points: i64,
    /// Passes.
    pub passes: u64,
    /// Groschen held at the end of each run, summed.
    pub coins_end: u64,
    /// Groschen earned from series, summed.
    pub coins_earned: u64,
    /// Groschen spent at the Tresen, summed.
    pub coins_spent: u64,
    /// Per relic.
    pub relics: BTreeMap<RelicId, RelicStat>,
    /// Encounters opened, by id: (opened, options other than leaving
    /// taken).
    pub encounters: BTreeMap<EncounterId, (u64, u64)>,
    /// Highest Spielwert of a won game, summed over runs.
    pub best_game: u64,
}

/// Play one run to the end and add it to the statistics.
fn play_run(
    content: &Content,
    options: &Options,
    seed: u64,
    stats: &mut RunStats,
) -> Result<(), String> {
    let start = options.start();
    let (mut run, mut events) = Run::new_with(content, seed, &start).map_err(|e| e.to_string())?;
    let mut actions = 0;
    while let Some(action) = options.next_action(&run, content) {
        events.extend(run.apply(content, action).map_err(|e| e.to_string())?);
        actions += 1;
        if actions > MAX_ACTIONS {
            return Err("run does not end".into());
        }
    }
    let sections = content.run.targets.len();
    let mut bought: BTreeSet<RelicId> = BTreeSet::new();
    let mut current_boss: Option<(BossId, u32)> = None;
    for event in &events {
        match event {
            RunEvent::SeriesStarted {
                section,
                index,
                boss,
                ..
            } => {
                if let Some(entry) = stats
                    .series
                    .get_mut(usize::try_from(*section).unwrap_or(usize::MAX))
                    .and_then(|s| s.get_mut(usize::try_from(*index).unwrap_or(usize::MAX)))
                {
                    entry.0 += 1;
                }
                current_boss = boss.clone().map(|b| (b, *section));
                if let Some(boss) = boss {
                    let table = if *section == 0 {
                        &mut stats.bosses
                    } else {
                        &mut stats.bosses_later
                    };
                    table.entry(boss.clone()).or_default().0 += 1;
                }
            }
            RunEvent::SeriesWon {
                section,
                index,
                payout,
                ..
            } => {
                if let Some(entry) = stats
                    .series
                    .get_mut(usize::try_from(*section).unwrap_or(usize::MAX))
                    .and_then(|s| s.get_mut(usize::try_from(*index).unwrap_or(usize::MAX)))
                {
                    entry.1 += 1;
                }
                if let Some((boss, section)) = &current_boss {
                    let table = if *section == 0 {
                        &mut stats.bosses
                    } else {
                        &mut stats.bosses_later
                    };
                    table.entry(boss.clone()).or_default().1 += 1;
                }
                stats.coins_earned += u64::from(payout.total());
            }
            RunEvent::GameScored { breakdown, .. } => {
                stats.games += 1;
                stats.points += breakdown.score;
                if breakdown.won {
                    stats.games_won += 1;
                }
            }
            RunEvent::Passed { .. } => stats.passes += 1,
            RunEvent::RelicBought { relic, price } => {
                stats.relics.entry(relic.clone()).or_default().bought += 1;
                stats.coins_spent += u64::from(*price);
                bought.insert(relic.clone());
            }
            RunEvent::ConsumableBought { price, .. } => stats.coins_spent += u64::from(*price),
            RunEvent::EncounterOpened { encounter, .. } => {
                stats.encounters.entry(encounter.clone()).or_default().0 += 1;
            }
            RunEvent::EncounterResolved { encounter, choice } => {
                if *choice != EncounterChoice::Leave {
                    stats.encounters.entry(encounter.clone()).or_default().1 += 1;
                }
            }
            RunEvent::ShopRerolled { cost } => stats.coins_spent += u64::from(*cost),
            _ => {}
        }
    }
    let won = run.phase() == RunPhase::Won;
    let section = usize::try_from(run.series().section).unwrap_or(0);
    if won {
        stats.won += 1;
    } else if let Some(n) = stats.lost_in_section.get_mut(section.min(sections - 1)) {
        *n += 1;
    }
    stats.coins_end += u64::from(run.coins());
    stats.best_game += run.stats().best_game;
    for relic in bought {
        let entry = stats.relics.entry(relic).or_default();
        entry.runs += 1;
        entry.runs_won += u64::from(won);
        entry.sections += u64::try_from(section + 1).unwrap_or(0);
    }
    Ok(())
}

/// Play `runs` runs. Run seeds come from one master generator, so the same
/// seed gives the same runs with and without options (same deals).
pub fn simulate(
    content: &Content,
    options: &Options,
    seed: u64,
    runs: u32,
) -> Result<RunStats, String> {
    let sections = content.run.targets.len();
    let mut stats = RunStats {
        seed,
        runs,
        lost_in_section: vec![0; sections],
        series: vec![[(0, 0); 3]; sections],
        ..RunStats::default()
    };
    // Every relic appears in the table, also if it was never bought.
    for id in content.relics.keys() {
        stats.relics.insert(id.clone(), RelicStat::default());
    }
    let mut master = Rng::from_seed(seed);
    for i in 0..runs {
        let run_seed = master.next_u64();
        play_run(content, options, run_seed, &mut stats).map_err(|e| format!("run {i}: {e}"))?;
    }
    Ok(stats)
}

fn pct(part: u64, whole: u64) -> f64 {
    if whole == 0 {
        0.0
    } else {
        100.0 * part as f64 / whole as f64
    }
}

fn mean(sum: u64, n: u64) -> f64 {
    if n == 0 { 0.0 } else { sum as f64 / n as f64 }
}

fn mean_signed(sum: i64, n: u64) -> f64 {
    if n == 0 { 0.0 } else { sum as f64 / n as f64 }
}

/// Render the statistics as a text table.
pub fn render(stats: &RunStats, content: &Content, options: &Options, runtime_us: u64) -> String {
    let runs = u64::from(stats.runs);
    let mut out = String::new();
    let _ = writeln!(
        out,
        "skat_sim seed={} runs={} options: {}",
        stats.seed,
        stats.runs,
        options.describe()
    );
    let _ = write!(out, "section reached:");
    for (i, n) in stats.lost_in_section.iter().enumerate() {
        let _ = write!(out, " {}={}", i + 1, n);
    }
    let _ = writeln!(out, " won={}", stats.won);
    // Share of runs that reached a section (lost there or later, or won).
    let _ = write!(out, "reached at least:");
    for section in 1..stats.lost_in_section.len() {
        let later: u64 = stats.lost_in_section[section..].iter().sum::<u64>() + stats.won;
        let _ = write!(out, " {}={:.1}%", section + 1, pct(later, runs));
    }
    out.push('\n');
    let _ = writeln!(
        out,
        "{:<8} {:>7} {:>9} {:>7}",
        "series", "target", "attempts", "win%"
    );
    for (section, row) in stats.series.iter().enumerate() {
        for (index, (attempts, wins)) in row.iter().enumerate() {
            let target = content.run.targets.get(section).map_or(0, |t| match index {
                0 => t.0,
                1 => t.1,
                _ => t.2,
            });
            let label = if index + 1 == usize::try_from(SERIES_PER_SECTION).unwrap_or(3) {
                format!("{}-boss", section + 1)
            } else {
                format!("{}-{}", section + 1, index + 1)
            };
            let _ = writeln!(
                out,
                "{:<8} {:>7} {:>9} {:>7.1}",
                label,
                target,
                attempts,
                pct(*wins, *attempts)
            );
        }
    }
    for (label, table) in [
        ("bosses section 1:", &stats.bosses),
        ("bosses section 2+:", &stats.bosses_later),
    ] {
        let _ = write!(out, "{label}");
        for (boss, (attempts, wins)) in table {
            let _ = write!(
                out,
                " {boss}={wins}/{attempts} ({:.1}%)",
                pct(*wins, *attempts)
            );
        }
        out.push('\n');
    }
    let _ = writeln!(
        out,
        "games: {} won%={:.1} passes={} points per game={:.1}",
        stats.games,
        pct(stats.games_won, stats.games),
        stats.passes,
        mean_signed(stats.points, stats.games)
    );
    let _ = writeln!(out, "best game per run: {:.1}", mean(stats.best_game, runs));
    let _ = write!(out, "encounters (opened/taken):");
    for (id, (opened, taken)) in &stats.encounters {
        let _ = write!(out, " {id}={opened}/{taken}");
    }
    out.push('\n');
    let _ = writeln!(
        out,
        "groschen per run: earned {:.2}, spent {:.2}, held at end {:.2}",
        mean(stats.coins_earned, runs),
        mean(stats.coins_spent, runs),
        mean(stats.coins_end, runs)
    );
    let _ = writeln!(
        out,
        "{:<18} {:>7} {:>6} {:>9} {:>13}",
        "relic", "bought", "runs", "run win%", "mean section"
    );
    for (id, r) in &stats.relics {
        let _ = writeln!(
            out,
            "{:<18} {:>7} {:>6} {:>9.1} {:>13.2}",
            id.as_str(),
            r.bought,
            r.runs,
            pct(r.runs_won, r.runs),
            mean(r.sections, r.runs)
        );
    }
    let per_run = if stats.runs == 0 {
        0.0
    } else {
        runtime_us as f64 / f64::from(stats.runs)
    };
    let _ = writeln!(
        out,
        "runtime: {:.1} ms total, {:.2} ms per run",
        runtime_us as f64 / 1000.0,
        per_run / 1000.0
    );
    out
}

// ---------------------------------------------------------------------------
// Success curve of one series
// ---------------------------------------------------------------------------

/// Success curve of one series (`--curve`): how often the bot wins a
/// series of the game limit and passes in `run.ron`, for every Zielwert.
///
/// Every sample is one series with an unreachable Zielwert, played to the
/// game limit. The bot never looks at the Zielwert or the account, so it
/// plays exactly as it would against any target, and a series with
/// Zielwert T is won exactly when the highest account after a game reaches
/// T. One sample thus gives the result for every T. With `--boss` the
/// series is a boss series with that boss.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Curve {
    /// Base seed.
    pub seed: u64,
    /// Number of series.
    pub series: u32,
    /// Highest account per series (`None`: never above 0 after a game).
    pub best: Vec<Option<i64>>,
    /// Games played (not passed).
    pub games: u64,
    /// Games won.
    pub games_won: u64,
    /// Sum of the account changes of all games.
    pub points: i64,
    /// Passes.
    pub passes: u64,
}

impl Curve {
    /// Share of series won at Zielwert `target`, in percent.
    #[must_use]
    pub fn win_pct(&self, target: u64) -> f64 {
        let won = self
            .best
            .iter()
            .flatten()
            .filter(|&&best| i128::from(best) >= i128::from(target))
            .count();
        pct(u64::try_from(won).unwrap_or(0), u64::from(self.series))
    }
}

/// Zielwerte the curve reports: 1 and every Zielwert of `run.ron`.
fn curve_targets(content: &Content) -> Vec<u64> {
    let mut targets: Vec<u64> = std::iter::once(1)
        .chain(content.run.targets.iter().flat_map(|t| [t.0, t.1, t.2]))
        .collect();
    targets.sort_unstable();
    targets.dedup();
    targets
}

/// Measure the success curve over `series` series.
pub fn curve(
    content: &Content,
    options: &Options,
    seed: u64,
    series: u32,
) -> Result<Curve, String> {
    let mut content = content.clone();
    let unreachable = u64::MAX / 4;
    content
        .run
        .targets
        .fill((unreachable, unreachable, unreachable));
    let start = RunStart {
        index: if options.boss.is_some() {
            SERIES_PER_SECTION - 1
        } else {
            0
        },
        ..options.start()
    };
    let mut result = Curve {
        seed,
        series,
        ..Curve::default()
    };
    let mut master = Rng::from_seed(seed);
    for i in 0..series {
        let run_seed = master.next_u64();
        let (mut run, _) =
            Run::new_with(&content, run_seed, &start).map_err(|e| format!("series {i}: {e}"))?;
        let mut best: Option<i64> = None;
        let mut actions = 0;
        while run.phase() == RunPhase::Game {
            let Some(action) = options.next_action(&run, &content) else {
                break;
            };
            for event in run
                .apply(&content, action)
                .map_err(|e| format!("series {i}: {e}"))?
            {
                match event {
                    RunEvent::GameScored {
                        breakdown,
                        series_score,
                    } => {
                        result.games += 1;
                        result.points += breakdown.score;
                        result.games_won += u64::from(breakdown.won);
                        best = Some(best.map_or(series_score, |b| b.max(series_score)));
                    }
                    RunEvent::Passed { .. } => result.passes += 1,
                    _ => {}
                }
            }
            actions += 1;
            if actions > MAX_ACTIONS {
                return Err(format!("series {i} does not end"));
            }
        }
        result.best.push(best);
    }
    Ok(result)
}

/// Render the success curve.
pub fn render_curve(
    curve: &Curve,
    content: &Content,
    options: &Options,
    runtime_us: u64,
) -> String {
    let mut out = String::new();
    let kind = if options.boss.is_some() {
        "boss series"
    } else {
        "series"
    };
    let _ = writeln!(
        out,
        "skat_sim seed={} curve: {} {kind}, game limit {}, passes {}, options: {}",
        curve.seed,
        curve.series,
        content.run.game_limit,
        content.run.passes,
        options.describe()
    );
    let _ = writeln!(out, "{:>7} {:>7}", "target", "win%");
    for target in curve_targets(content) {
        let _ = writeln!(out, "{:>7} {:>7.1}", target, curve.win_pct(target));
    }
    let n = u64::from(curve.series);
    let _ = writeln!(
        out,
        "games: {} won%={:.1} passes={} points per game={:.1} games per series={:.2}",
        curve.games,
        pct(curve.games_won, curve.games),
        curve.passes,
        mean_signed(curve.points, curve.games),
        mean(curve.games + curve.passes, n)
    );
    let _ = writeln!(out, "runtime: {:.1} ms total", runtime_us as f64 / 1000.0);
    out
}

/// Entry point of the runs mode.
pub fn main(seed: u64, runs: u32, options: &Options) -> ExitCode {
    let content = match Content::load()
        .map_err(|e| e.to_string())
        .and_then(|base| options.content(&base))
    {
        Ok(c) => c,
        Err(e) => {
            eprintln!("error: {e}");
            return ExitCode::FAILURE;
        }
    };
    let start = Instant::now();
    let output = if options.unlocks {
        unlocks(&content, options, seed, runs).map(|u| render_unlocks(&u, &content))
    } else if options.curve {
        curve(&content, options, seed, runs).map(|c| {
            let runtime_us = u64::try_from(start.elapsed().as_micros()).unwrap_or(u64::MAX);
            render_curve(&c, &content, options, runtime_us)
        })
    } else {
        simulate(&content, options, seed, runs).map(|s| {
            let runtime_us = u64::try_from(start.elapsed().as_micros()).unwrap_or(u64::MAX);
            render(&s, &content, options, runtime_us)
        })
    };
    match output {
        Ok(text) => {
            print!("{text}");
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::FAILURE
        }
    }
}

// ---------------------------------------------------------------------------
// Reachability of the unlocks
// ---------------------------------------------------------------------------

/// Sequences of runs played by `--unlocks`, each with a fresh meta state.
pub const UNLOCK_SEQUENCES: u32 = 20;

/// How many runs a fresh player (the bot) needs for each unlock.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Unlocks {
    /// Base seed.
    pub seed: u64,
    /// Sequences played.
    pub sequences: u32,
    /// Most runs per sequence.
    pub max_runs: u32,
    /// Per unlock: the run number (from 1) at which each sequence earned
    /// it; sequences that never earned it are missing.
    pub earned_at: BTreeMap<UnlockId, Vec<u32>>,
}

/// Play [`UNLOCK_SEQUENCES`] sequences of up to `max_runs` runs. Every
/// sequence starts with a fresh [`MetaState`]; each run starts from it
/// (locked relics stay off the Tresen) and its events feed it. The bot
/// plays the Alleinspieler and, once unlocked, every other run the
/// Nullspieler. A sequence stops when everything is unlocked.
pub fn unlocks(
    content: &Content,
    options: &Options,
    seed: u64,
    max_runs: u32,
) -> Result<Unlocks, String> {
    let mut result = Unlocks {
        seed,
        sequences: UNLOCK_SEQUENCES,
        max_runs,
        ..Unlocks::default()
    };
    for id in content.unlocks.keys() {
        result.earned_at.insert(id.clone(), Vec::new());
    }
    let mut master = Rng::from_seed(seed);
    for sequence in 0..UNLOCK_SEQUENCES {
        let mut meta = MetaState::new();
        for n in 1..=max_runs {
            if meta.unlocked.len() == content.unlocks.len() {
                break;
            }
            let character = if n % 2 == 0 && meta.character_unlocked(content, Character::NullPlayer)
            {
                Character::NullPlayer
            } else {
                Character::Declarer
            };
            let mut start = meta
                .run_start(content, character)
                .map_err(|e| e.to_string())?;
            start.relics.clone_from(&options.relics);
            let run_seed = master.next_u64();
            let (mut run, mut events) =
                Run::new_with(content, run_seed, &start).map_err(|e| e.to_string())?;
            let mut actions = 0;
            while let Some(action) = options.next_action(&run, content) {
                events.extend(run.apply(content, action).map_err(|e| e.to_string())?);
                actions += 1;
                if actions > MAX_ACTIONS {
                    return Err(format!("sequence {sequence}: run does not end"));
                }
            }
            for id in meta.observe(content, &events) {
                result.earned_at.entry(id).or_default().push(n);
            }
        }
    }
    Ok(result)
}

/// Render the reachability table.
pub fn render_unlocks(unlocks: &Unlocks, content: &Content) -> String {
    let mut out = String::new();
    let _ = writeln!(
        out,
        "skat_sim seed={} unlocks: {} sequences of at most {} runs, fresh meta state each",
        unlocks.seed, unlocks.sequences, unlocks.max_runs
    );
    let _ = writeln!(
        out,
        "{:<20} {:>8} {:>10} {:>8} {:>8}  reward",
        "unlock", "reached", "mean runs", "min", "max"
    );
    for (id, at) in &unlocks.earned_at {
        let reached = at.len();
        let mean_runs = if reached == 0 {
            0.0
        } else {
            at.iter().map(|&n| f64::from(n)).sum::<f64>() / reached as f64
        };
        let rewards = content
            .unlock(id)
            .map(|u| format!("{:?}", u.rewards))
            .unwrap_or_default();
        let _ = writeln!(
            out,
            "{:<20} {:>5}/{:<2} {:>10.1} {:>8} {:>8}  {rewards}",
            id.as_str(),
            reached,
            unlocks.sequences,
            mean_runs,
            at.iter().min().copied().unwrap_or(0),
            at.iter().max().copied().unwrap_or(0),
        );
    }
    out
}

/// The content with heuristic defenders everywhere: the option tests
/// check plumbing, and PIMC in an unoptimized test build is slow.
#[cfg(test)]
fn heuristic_content() -> Content {
    let mut content = Content::load().expect("content");
    content.run.opponents.clear();
    content
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_seed_gives_identical_run_statistics() {
        let content = heuristic_content();
        let options = Options::default();
        let a = simulate(&content, &options, 42, 30).expect("runs");
        let b = simulate(&content, &options, 42, 30).expect("runs");
        assert_eq!(a, b);
        assert_eq!(
            render(&a, &content, &options, 0),
            render(&b, &content, &options, 0)
        );
        assert_ne!(simulate(&content, &options, 43, 30).expect("runs"), a);
    }

    #[test]
    fn every_run_ends_once() {
        let content = heuristic_content();
        let stats = simulate(&content, &Options::default(), 7, 25).expect("runs");
        let ended: u64 = stats.lost_in_section.iter().sum::<u64>() + stats.won;
        assert_eq!(ended, 25);
        assert_eq!(stats.series[0][0].0, 25, "every run starts with series 1-1");
    }
}

#[cfg(test)]
mod option_tests {
    use super::*;

    fn content() -> Content {
        heuristic_content()
    }

    #[test]
    fn pimc_defenders_and_bot_play_whole_runs_deterministically() {
        let base = Content::load().expect("content");
        let options = Options {
            bot: Opponents::Pimc {
                samples: 2,
                depth: 1,
            },
            defenders: Some(Opponents::Pimc {
                samples: 2,
                depth: 1,
            }),
            ..Options::default()
        };
        let content = options.content(&base).expect("content");
        assert_eq!(content.run.opponents.len(), 1);
        let a = simulate(&content, &options, 4, 3).expect("runs");
        assert_eq!(a, simulate(&content, &options, 4, 3).expect("runs"));
        assert_eq!(a.lost_in_section.iter().sum::<u64>() + a.won, 3);
        let text = render(&a, &content, &options, 0);
        assert!(text.contains("bot=pimc:2:1") && text.contains("defenders=pimc:2:1"));
    }

    #[test]
    fn no_shop_never_buys_and_without_never_offers() {
        let base = content();
        let options = Options {
            no_shop: true,
            ..Options::default()
        };
        let stats = simulate(&base, &options, 3, 40).expect("runs");
        assert_eq!(stats.coins_spent, 0);
        assert!(stats.relics.values().all(|r| r.bought == 0));
        let options = Options {
            without: vec![RelicId::new("bock_round")],
            ..Options::default()
        };
        let content = options.content(&base).expect("content");
        let stats = simulate(&content, &options, 3, 40).expect("runs");
        assert!(!stats.relics.contains_key(&RelicId::new("bock_round")));
        let bad = Options {
            without: vec![RelicId::new("nope")],
            ..Options::default()
        };
        assert!(bad.content(&base).is_err());
        let bad = Options {
            relics: vec![RelicId::new("nope")],
            ..Options::default()
        };
        assert!(bad.content(&base).is_err());
    }

    #[test]
    fn boss_option_draws_only_that_boss() {
        let base = content();
        let options = Options {
            boss: Some(BossId::new("grandma")),
            ..Options::default()
        };
        let content = options.content(&base).expect("content");
        let stats = simulate(&content, &options, 5, 60).expect("runs");
        assert!(!stats.bosses.is_empty());
        assert!(
            stats
                .bosses
                .keys()
                .chain(stats.bosses_later.keys())
                .all(|b| b.as_str() == "grandma")
        );
    }

    #[test]
    fn start_relics_keep_the_deals() {
        let base = content();
        let plain = simulate(&base, &Options::default(), 8, 20).expect("runs");
        let options = Options {
            relics: vec![RelicId::new("ouvert_bonus")],
            no_shop: true,
            ..Options::default()
        };
        let with = simulate(&base, &options, 8, 20).expect("runs");
        assert_eq!(with.runs, plain.runs);
        assert!(render(&with, &base, &options, 0).contains("relic=ouvert_bonus"));
    }

    #[test]
    fn curve_falls_with_the_target_and_is_deterministic() {
        let base = content();
        let options = Options::default();
        let a = curve(&base, &options, 42, 200).expect("curve");
        assert_eq!(a, curve(&base, &options, 42, 200).expect("curve"));
        assert_eq!(a.best.len(), 200);
        let targets = curve_targets(&base);
        assert_eq!(targets[0], 1);
        let pcts: Vec<f64> = targets.iter().map(|&t| a.win_pct(t)).collect();
        assert!(pcts.windows(2).all(|w| w[0] >= w[1]), "{pcts:?}");
        assert!(pcts[0] > 0.0 && pcts[0] < 100.0);
        // The same seed in the runs mode wins series 1-1 exactly when the
        // curve reaches its Zielwert.
        let runs = simulate(&base, &options, 42, 200).expect("runs");
        let won_1_1 = runs.series[0][0].1;
        let first = base.run.targets[0].0;
        let from_curve = a
            .best
            .iter()
            .flatten()
            .filter(|&&b| b >= i64::try_from(first).unwrap_or(i64::MAX))
            .count();
        assert_eq!(u64::try_from(from_curve).unwrap_or(0), won_1_1);
        let boss = Options {
            boss: Some(BossId::new("the_pusher")),
            ..Options::default()
        };
        let content = boss.content(&base).expect("content");
        let b = curve(&content, &boss, 42, 50).expect("boss curve");
        assert_eq!(b.best.len(), 50);
    }

    #[test]
    fn the_null_player_and_the_unlocks_can_be_measured() {
        let base = content();
        let options = Options {
            character: Character::NullPlayer,
            null_scoring: Some(NullScoring::Fixed),
            ..Options::default()
        };
        let content = options.content(&base).expect("content");
        let stats = simulate(&content, &options, 2, 20).expect("runs");
        assert_eq!(stats.lost_in_section.iter().sum::<u64>() + stats.won, 20);
        assert!(render(&stats, &content, &options, 0).contains("character=NullPlayer"));
        let options = Options {
            unlocks: true,
            ..Options::default()
        };
        let a = unlocks(&base, &options, 3, 15).expect("unlocks");
        assert_eq!(a, unlocks(&base, &options, 3, 15).expect("unlocks"));
        assert_eq!(a.earned_at.len(), base.unlocks.len());
        assert!(
            !a.earned_at[&UnlockId::new("second_evening")].is_empty(),
            "section 2 is reached within 15 runs"
        );
        assert!(render_unlocks(&a, &base).contains("tour_won"));
    }
}

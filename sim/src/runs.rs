//! Runs mode: complete runs with the simple run policy.
//!
//! Statistics: section reached, win rate per series, Groschen, and for
//! every relic how often it was bought and how runs with it went.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::process::ExitCode;
use std::time::Instant;

use skat_core::content::{BossId, Content, RelicId};
use skat_core::rng::Rng;
use skat_core::run::bot::RunBot;
use skat_core::run::{Run, RunEvent, RunPhase, SERIES_PER_SECTION};

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
    /// Attempts and wins per boss.
    pub bosses: BTreeMap<BossId, (u64, u64)>,
    /// Games played (not passed).
    pub games: u64,
    /// Games won.
    pub games_won: u64,
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
}

/// Play one run to the end and add it to the statistics.
fn play_run(content: &Content, seed: u64, stats: &mut RunStats) -> Result<(), String> {
    let (mut run, mut events) = Run::new(content, seed).map_err(|e| e.to_string())?;
    let mut actions = 0;
    while let Some(action) = RunBot.next_action(&run, content) {
        events.extend(run.apply(content, action).map_err(|e| e.to_string())?);
        actions += 1;
        if actions > MAX_ACTIONS {
            return Err("run does not end".into());
        }
    }
    let sections = content.run.targets.len();
    let mut bought: BTreeSet<RelicId> = BTreeSet::new();
    let mut current_boss: Option<BossId> = None;
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
                current_boss = boss.clone();
                if let Some(boss) = boss {
                    stats.bosses.entry(boss.clone()).or_default().0 += 1;
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
                if let Some(boss) = &current_boss {
                    stats.bosses.entry(boss.clone()).or_default().1 += 1;
                }
                stats.coins_earned += u64::from(payout.total());
            }
            RunEvent::GameScored { breakdown, .. } => {
                stats.games += 1;
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
    for relic in bought {
        let entry = stats.relics.entry(relic).or_default();
        entry.runs += 1;
        entry.runs_won += u64::from(won);
        entry.sections += u64::try_from(section + 1).unwrap_or(0);
    }
    Ok(())
}

/// Play `runs` runs. Run seeds come from one master generator.
pub fn simulate(content: &Content, seed: u64, runs: u32) -> Result<RunStats, String> {
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
        play_run(content, run_seed, &mut stats).map_err(|e| format!("run {i}: {e}"))?;
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

/// Render the statistics as a text table.
pub fn render(stats: &RunStats, content: &Content, runtime_us: u64) -> String {
    let runs = u64::from(stats.runs);
    let mut out = String::new();
    let _ = writeln!(out, "skat_sim seed={} runs={}", stats.seed, stats.runs);
    let _ = write!(out, "section reached:");
    for (i, n) in stats.lost_in_section.iter().enumerate() {
        let _ = write!(out, " {}={}", i + 1, n);
    }
    let _ = writeln!(out, " won={}", stats.won);
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
    let _ = write!(out, "bosses:");
    for (boss, (attempts, wins)) in &stats.bosses {
        let _ = write!(out, " {boss}={wins}/{attempts}");
    }
    out.push('\n');
    let _ = writeln!(
        out,
        "games: {} won%={:.1} passes={}",
        stats.games,
        pct(stats.games_won, stats.games),
        stats.passes
    );
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

/// Entry point of the runs mode.
pub fn main(seed: u64, runs: u32) -> ExitCode {
    let content = match Content::load() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("error: {e}");
            return ExitCode::FAILURE;
        }
    };
    let start = Instant::now();
    let stats = match simulate(&content, seed, runs) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("error: {e}");
            return ExitCode::FAILURE;
        }
    };
    let runtime_us = u64::try_from(start.elapsed().as_micros()).unwrap_or(u64::MAX);
    print!("{}", render(&stats, &content, runtime_us));
    ExitCode::SUCCESS
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_seed_gives_identical_run_statistics() {
        let content = Content::load().expect("content");
        let a = simulate(&content, 42, 30).expect("runs");
        let b = simulate(&content, 42, 30).expect("runs");
        assert_eq!(a, b);
        assert_eq!(render(&a, &content, 0), render(&b, &content, 0));
        assert_ne!(simulate(&content, 43, 30).expect("runs"), a);
    }

    #[test]
    fn every_run_ends_once() {
        let content = Content::load().expect("content");
        let stats = simulate(&content, 7, 25).expect("runs");
        let ended: u64 = stats.lost_in_section.iter().sum::<u64>() + stats.won;
        assert_eq!(ended, 25);
        assert_eq!(stats.series[0][0].0, 25, "every run starts with series 1-1");
    }
}

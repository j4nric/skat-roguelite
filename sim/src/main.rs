//! Headless simulator.
//!
//! Games mode (milestone M1): plays many games with the heuristic AI on all
//! three seats and prints a baseline statistic.
//! `cargo run --release -p skat_sim -- --seed 42 --games 10000 [--json]`
//!
//! Runs mode (milestone M2): plays complete runs with the simple run policy
//! ([`skat_core::run::bot::RunBot`]) and prints how far runs get.
//! `cargo run --release -p skat_sim -- --seed 42 --runs 1000`
//!
//! Measurement options of the runs mode:
//!
//! - `--no-shop`: the bot never buys anything.
//! - `--relic <id>`: every run starts with this relic (repeatable); its
//!   passive effects already count in the first game. The deals stay the
//!   same as without, so the difference is the relic.
//! - `--without <id>`: the relic is taken out of the pool (repeatable).
//! - `--boss <id>`: boss series draw only this boss.
//! - `--curve`: instead of whole runs, `--runs N` single series (with
//!   `--boss` boss series); prints the share of series won for every
//!   Zielwert of `run.ron`.
//!
//! Everything but the runtime lines is a pure function of the arguments,
//! so two runs with the same arguments print identical statistics.

mod runs;

use std::fmt::Write as _;
use std::process::ExitCode;
use std::time::Instant;

use skat_core::ai::{DeclarerHeuristic, DefenderHeuristic, Policy};
use skat_core::cards::Suit;
use skat_core::content::{BossId, RelicId};
use skat_core::game::{Game, Phase, Seat};
use skat_core::rng::Rng;
use skat_core::rules::{GameType, Matadors, RiskLevel};

struct Args {
    seed: u64,
    games: u32,
    runs: Option<u32>,
    json: bool,
    options: runs::Options,
}

fn parse_args() -> Result<Args, String> {
    let mut args = Args {
        seed: 42,
        games: 1000,
        runs: None,
        json: false,
        options: runs::Options::default(),
    };
    let mut it = std::env::args().skip(1);
    while let Some(flag) = it.next() {
        match flag.as_str() {
            "--json" => args.json = true,
            "--no-shop" => args.options.no_shop = true,
            "--curve" => args.options.curve = true,
            "--relic" | "--without" | "--boss" => {
                let value = it
                    .next()
                    .ok_or_else(|| format!("missing value for {flag}"))?;
                match flag.as_str() {
                    "--relic" => args.options.relics.push(RelicId::new(&value)),
                    "--without" => args.options.without.push(RelicId::new(&value)),
                    _ => args.options.boss = Some(BossId::new(&value)),
                }
            }
            "--seed" | "--games" | "--runs" => {
                let value = it
                    .next()
                    .ok_or_else(|| format!("missing value for {flag}"))?;
                if flag == "--seed" {
                    args.seed = value.parse().map_err(|e| format!("--seed: {e}"))?;
                } else if flag == "--runs" {
                    args.runs = Some(value.parse().map_err(|e| format!("--runs: {e}"))?);
                } else {
                    args.games = value.parse().map_err(|e| format!("--games: {e}"))?;
                }
            }
            other => return Err(format!("unknown flag {other}")),
        }
    }
    if args.runs.is_none() && args.options != runs::Options::default() {
        if args.options.curve {
            args.runs = Some(1000);
        } else {
            return Err("run options need --runs N".into());
        }
    }
    Ok(args)
}

/// The five game types the declarer heuristic may choose, in report order.
const GAME_TYPES: [GameType; 5] = [
    GameType::Suit(Suit::Diamonds),
    GameType::Suit(Suit::Hearts),
    GameType::Suit(Suit::Spades),
    GameType::Suit(Suit::Clubs),
    GameType::Grand,
];

/// Aggregate over a set of games.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
struct Bucket {
    games: u64,
    won: u64,
    schneider: u64,
    schwarz: u64,
    hand: u64,
    /// Sum of the game value over won games.
    value_won: u64,
    /// Sum of the signed score over all games.
    score: i64,
    declarer_card_points: u64,
}

impl Bucket {
    fn add(&mut self, result: &GameResult) {
        self.games += 1;
        self.score += i64::from(result.score);
        self.declarer_card_points += u64::from(result.declarer_card_points);
        if result.hand {
            self.hand += 1;
        }
        if result.won {
            self.won += 1;
            self.value_won += u64::from(result.value);
        }
        if result.schneider {
            self.schneider += 1;
        }
        if result.schwarz {
            self.schwarz += 1;
        }
    }

    fn pct(part: u64, whole: u64) -> f64 {
        if whole == 0 {
            0.0
        } else {
            100.0 * part as f64 / whole as f64
        }
    }

    fn mean(sum: f64, n: u64) -> f64 {
        if n == 0 { 0.0 } else { sum / n as f64 }
    }

    fn win_pct(&self) -> f64 {
        Self::pct(self.won, self.games)
    }
    fn schneider_pct(&self) -> f64 {
        Self::pct(self.schneider, self.games)
    }
    fn schwarz_pct(&self) -> f64 {
        Self::pct(self.schwarz, self.games)
    }
    fn mean_value_won(&self) -> f64 {
        Self::mean(self.value_won as f64, self.won)
    }
    fn mean_score(&self) -> f64 {
        Self::mean(self.score as f64, self.games)
    }
    fn mean_declarer_card_points(&self) -> f64 {
        Self::mean(self.declarer_card_points as f64, self.games)
    }
}

/// Matador histogram: index 0..4 = with 1..4+, 4..8 = without 1..4+.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
struct MatadorHistogram([u64; 8]);

impl MatadorHistogram {
    fn add(&mut self, m: Matadors) {
        let idx = usize::try_from(m.count.clamp(1, 4) - 1).unwrap_or(3);
        self.0[if m.with { idx } else { 4 + idx }] += 1;
    }

    fn label(i: usize) -> String {
        let n = i % 4 + 1;
        let plus = if n == 4 { "+" } else { "" };
        let kind = if i < 4 { "with" } else { "without" };
        format!("{kind} {n}{plus}")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct GameResult {
    game: GameType,
    hand: bool,
    won: bool,
    schneider: bool,
    schwarz: bool,
    value: u32,
    score: i32,
    declarer_card_points: u32,
    matadors: Matadors,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Stats {
    seed: u64,
    games: u32,
    by_game: [Bucket; 5],
    total: Bucket,
    matadors: MatadorHistogram,
}

fn game_index(game: GameType) -> Option<usize> {
    GAME_TYPES.iter().position(|&g| g == game)
}

/// Play one complete game with the heuristics on all seats.
fn play_game(rng: &mut Rng, forehand: Seat) -> Result<GameResult, String> {
    let declarer = DeclarerHeuristic;
    let defender = DefenderHeuristic;
    let mut game = Game::deal(rng, forehand);
    let declaration = declarer.choose_declaration(game.hand(Seat::DECLARER));
    game.declare(declaration).map_err(|e| e.to_string())?;
    if game.phase() == Phase::Discarding {
        let discard = declarer
            .choose_discard(game.hand(Seat::DECLARER), declaration.game)
            .ok_or("no discard")?;
        game.discard(discard).map_err(|e| e.to_string())?;
    }
    while let Some(seat) = game.current_seat() {
        let view = game.view(seat).ok_or("no view while playing")?;
        let card = if seat.is_declarer() {
            declarer.choose(&view)
        } else {
            defender.choose(&view)
        }
        .ok_or("policy returned no card")?;
        game.play(seat, card).map_err(|e| e.to_string())?;
    }
    let value = game.value().ok_or("game did not finish")?;
    let outcome = game.outcome();
    Ok(GameResult {
        game: declaration.game,
        hand: declaration.risk >= RiskLevel::Hand,
        won: value.won,
        schneider: value.schneider,
        schwarz: value.schwarz,
        value: value.value,
        score: value.score,
        declarer_card_points: outcome.declarer_card_points,
        matadors: skat_core::rules::matadors(declaration.game, game.declarer_cards()),
    })
}

fn run(seed: u64, games: u32) -> Result<Stats, String> {
    let mut master = Rng::from_seed(seed);
    let mut stats = Stats {
        seed,
        games,
        by_game: [Bucket::default(); 5],
        total: Bucket::default(),
        matadors: MatadorHistogram::default(),
    };
    for i in 0..games {
        let mut rng = master.fork();
        let forehand =
            Seat::from_index(usize::try_from(i % 3).unwrap_or(0)).unwrap_or(Seat::DECLARER);
        let result = play_game(&mut rng, forehand).map_err(|e| format!("game {i}: {e}"))?;
        let idx = game_index(result.game).ok_or_else(|| format!("game {i}: unexpected type"))?;
        stats.by_game[idx].add(&result);
        stats.total.add(&result);
        stats.matadors.add(result.matadors);
    }
    Ok(stats)
}

fn render_table(stats: &Stats, runtime_us: u64) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "skat_sim seed={} games={}", stats.seed, stats.games);
    let _ = writeln!(
        out,
        "{:<9} {:>6} {:>7} {:>11} {:>9} {:>11} {:>10} {:>6}",
        "game", "n", "win%", "schneider%", "schwarz%", "value(won)", "score", "hand"
    );
    let rows = GAME_TYPES
        .iter()
        .zip(stats.by_game.iter())
        .map(|(g, b)| (g.name(), b))
        .chain(std::iter::once(("total", &stats.total)));
    for (name, b) in rows {
        let _ = writeln!(
            out,
            "{:<9} {:>6} {:>7.1} {:>11.1} {:>9.1} {:>11.1} {:>10.1} {:>6}",
            name,
            b.games,
            b.win_pct(),
            b.schneider_pct(),
            b.schwarz_pct(),
            b.mean_value_won(),
            b.mean_score(),
            b.hand,
        );
    }
    let _ = write!(out, "matadors:");
    for (i, n) in stats.matadors.0.iter().enumerate() {
        let _ = write!(out, " {}={}", MatadorHistogram::label(i), n);
    }
    out.push('\n');
    let _ = writeln!(
        out,
        "mean declarer card points: {:.1}",
        stats.total.mean_declarer_card_points()
    );
    let per_game = if stats.games == 0 {
        0.0
    } else {
        runtime_us as f64 / f64::from(stats.games)
    };
    let _ = writeln!(
        out,
        "runtime: {:.1} ms total, {:.2} us per game",
        runtime_us as f64 / 1000.0,
        per_game
    );
    out
}

fn json_bucket(b: &Bucket) -> String {
    format!(
        concat!(
            "{{\"games\":{},\"won\":{},\"win_pct\":{:.2},\"schneider\":{},\"schwarz\":{},",
            "\"hand\":{},\"mean_value_won\":{:.2},\"mean_score\":{:.2},",
            "\"mean_declarer_card_points\":{:.2}}}"
        ),
        b.games,
        b.won,
        b.win_pct(),
        b.schneider,
        b.schwarz,
        b.hand,
        b.mean_value_won(),
        b.mean_score(),
        b.mean_declarer_card_points(),
    )
}

fn render_json(stats: &Stats, runtime_us: u64) -> String {
    let mut out = String::new();
    let _ = write!(out, "{{\"seed\":{},\"games\":{},", stats.seed, stats.games);
    out.push_str("\"by_game\":{");
    for (i, (g, b)) in GAME_TYPES.iter().zip(stats.by_game.iter()).enumerate() {
        if i > 0 {
            out.push(',');
        }
        let _ = write!(out, "\"{}\":{}", g.name(), json_bucket(b));
    }
    let _ = write!(out, "}},\"total\":{},", json_bucket(&stats.total));
    out.push_str("\"matadors\":{");
    for (i, n) in stats.matadors.0.iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        let _ = write!(
            out,
            "\"{}\":{}",
            MatadorHistogram::label(i)
                .replace(' ', "_")
                .replace('+', "plus"),
            n
        );
    }
    let _ = write!(out, "}},\"runtime_us\":{runtime_us}}}");
    out
}

fn main() -> ExitCode {
    let args = match parse_args() {
        Ok(a) => a,
        Err(e) => {
            eprintln!("error: {e}");
            eprintln!(
                "usage: sim [--seed N] [--games N [--json] | --runs N [--no-shop] [--relic ID]... [--without ID]... [--boss ID] [--curve]]"
            );
            return ExitCode::FAILURE;
        }
    };
    if let Some(runs) = args.runs {
        return runs::main(args.seed, runs, &args.options);
    }
    let start = Instant::now();
    let stats = match run(args.seed, args.games) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("error: {e}");
            return ExitCode::FAILURE;
        }
    };
    let runtime_us = u64::try_from(start.elapsed().as_micros()).unwrap_or(u64::MAX);
    print!("{}", render_table(&stats, runtime_us));
    if args.json {
        println!("{}", render_json(&stats, runtime_us));
    }
    ExitCode::SUCCESS
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_seed_gives_identical_output() {
        let a = run(42, 300).expect("run");
        let b = run(42, 300).expect("run");
        assert_eq!(a, b);
        assert_eq!(render_table(&a, 0), render_table(&b, 0));
        assert_eq!(render_json(&a, 0), render_json(&b, 0));
        assert_ne!(run(43, 300).expect("run"), a);
    }

    #[test]
    fn every_game_is_counted_once() {
        let stats = run(7, 120).expect("run");
        assert_eq!(stats.total.games, 120);
        let per_game: u64 = stats.by_game.iter().map(|b| b.games).sum();
        assert_eq!(per_game, 120);
        let matadors: u64 = stats.matadors.0.iter().sum();
        assert_eq!(matadors, 120);
    }

    #[test]
    fn json_has_balanced_braces() {
        let stats = run(1, 10).expect("run");
        let json = render_json(&stats, 5);
        assert_eq!(
            json.matches('{').count(),
            json.matches('}').count(),
            "{json}"
        );
        assert!(json.starts_with('{') && json.ends_with('}'));
    }
}

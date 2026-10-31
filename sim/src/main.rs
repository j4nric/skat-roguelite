//! Headless simulator. Plays many games with the heuristic AI and prints
//! aggregate statistics. Usage: `cargo run -p skat_sim -- --seed 42 --games 10000`.
//!
//! Milestone M1 fills this in. Until then it validates the deck and the
//! argument parsing so that CI has something to run.

use std::process::ExitCode;

struct Args {
    seed: u64,
    games: u32,
}

fn parse_args() -> Result<Args, String> {
    let mut args = Args {
        seed: 42,
        games: 1000,
    };
    let mut it = std::env::args().skip(1);
    while let Some(flag) = it.next() {
        let value = it
            .next()
            .ok_or_else(|| format!("missing value for {flag}"))?;
        match flag.as_str() {
            "--seed" => args.seed = value.parse().map_err(|e| format!("--seed: {e}"))?,
            "--games" => args.games = value.parse().map_err(|e| format!("--games: {e}"))?,
            other => return Err(format!("unknown flag {other}")),
        }
    }
    Ok(args)
}

fn main() -> ExitCode {
    let args = match parse_args() {
        Ok(a) => a,
        Err(e) => {
            eprintln!("error: {e}");
            eprintln!("usage: sim [--seed N] [--games N]");
            return ExitCode::FAILURE;
        }
    };
    let deck = skat_core::cards::standard_deck();
    let points: u32 = deck.iter().map(|c| c.card_points()).sum();
    println!("skat_sim (stub) seed={} games={}", args.seed, args.games);
    println!("deck: {} cards, {} Augen", deck.len(), points);
    println!("baseline: not implemented yet (M1)");
    ExitCode::SUCCESS
}

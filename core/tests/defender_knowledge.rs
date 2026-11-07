//! Measurement (ignored by default): how much the defenders gain from
//! knowing the declarer's cards, an open Ouvert hand or "Durchsichtig"
//! cards pinned to him. Run in release:
//!
//! ```sh
//! cargo test --release -p skat_core --test defender_knowledge -- --ignored --nocapture
//! ```
//!
//! `KNOWLEDGE_GAMES` sets the number of deals per scenario (default 200).
//! The declarer plays the heuristic in every case, so only the defence
//! changes between the rows.

use std::time::Instant;

use skat_core::ai::pimc::{PimcConfig, PimcPolicy};
use skat_core::ai::sample::Hints;
use skat_core::ai::solver::{self, Goal, Layout, Limits, Table};
use skat_core::ai::{DeclarerHeuristic, DefenderHeuristic, Policy, null_unsafe_cards};
use skat_core::cards::{Card, Rank};
use skat_core::game::{Game, Seat};
use skat_core::rng::Rng;
use skat_core::rules::{Declaration, GameType, NullRisk, RiskLevel};

/// The defence of one row.
#[derive(Debug, Clone, Copy)]
enum Defence {
    Heuristic,
    /// PIMC; `pins` = how many of the declarer's cards it is told.
    Pimc(PimcConfig, Pins),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Pins {
    None,
    /// The cards marked for the deal.
    Marked,
}

fn pimc(samples: u32, depth: u32, pins: Pins) -> Defence {
    Defence::Pimc(PimcConfig { samples, depth }, pins)
}

#[derive(Debug, Default)]
struct Tally {
    games: u32,
    declarer_won: u32,
    decisions: u32,
    micros: u128,
    max_micros: u128,
}

impl Tally {
    fn row(&self, label: &str) -> String {
        let pct = 100.0 * f64::from(self.declarer_won) / f64::from(self.games.max(1));
        let mean = self.micros as f64 / 1000.0 / f64::from(self.decisions.max(1));
        format!(
            "{label:<34} games {:>4}  declarer won {:>5.1}%  ms/decision mean {:>6.2} max {:>7.2}",
            self.games,
            pct,
            mean,
            self.max_micros as f64 / 1000.0
        )
    }
}

/// Play `game` to the end: the declarer by the heuristic, the defenders by
/// `defence` (told `marked` cards of the declarer when it uses pins).
fn play(
    mut game: Game,
    defence: Defence,
    marked: &[Card],
    seed: u64,
    perfect: bool,
    tally: &mut Tally,
) {
    while let Some(seat) = game.current_seat() {
        let view = game.view(seat).expect("view");
        let card = if seat.is_declarer() && perfect {
            perfect_card(&game)
        } else if seat.is_declarer() {
            DeclarerHeuristic.choose(&view)
        } else {
            let start = Instant::now();
            let card = match defence {
                Defence::Heuristic => DefenderHeuristic.choose(&view),
                Defence::Pimc(config, pins) => {
                    let pinned = match pins {
                        Pins::None => Vec::new(),
                        Pins::Marked => marked
                            .iter()
                            .filter(|c| game.hand(Seat::DECLARER).contains(c))
                            .map(|&c| (c, Some(Seat::DECLARER)))
                            .collect(),
                    };
                    let hints = Hints {
                        pinned,
                        defender_swaps: Vec::new(),
                    };
                    PimcPolicy::new(config, seed).choose_with(&view, &hints)
                }
            };
            let us = start.elapsed().as_micros();
            tally.decisions += 1;
            tally.micros += us;
            tally.max_micros = tally.max_micros.max(us);
            card
        }
        .expect("a card");
        game.play(seat, card).expect("legal");
    }
    tally.games += 1;
    if game.value().is_some_and(|v| v.won) {
        tally.declarer_won += 1;
    }
}

/// Double-dummy values of the moves, with a node budget: a rare deal
/// explodes otherwise (behind the budget the solver estimates).
fn dd_solve(game: GameType, goal: Goal, layout: &Layout<Card>) -> Vec<(Card, i32)> {
    let mut table = Table::new(18);
    let limits = Limits {
        depth: 0,
        nodes: 400_000,
    };
    solver::solve_with(game, goal, layout, limits, &mut table).0
}

/// The declarer's best card with every hand known (double dummy): the
/// lowest value of the goal his game is lost by.
fn perfect_card(game: &Game) -> Option<Card> {
    let declaration = game.declaration()?;
    let goal = if declaration.is_null() {
        Goal::NullDeclarerTrick
    } else {
        Goal::DefenderTrick
    };
    let layout = Layout {
        hands: [
            game.hand(Seat::ALL[0]).to_vec(),
            game.hand(Seat::ALL[1]).to_vec(),
            game.hand(Seat::ALL[2]).to_vec(),
        ],
        trick_lead: game.trick_lead(),
        trick: game.current_trick().to_vec(),
        skat_points: 0,
    };
    dd_solve(declaration.game(), goal, &layout)
        .into_iter()
        .min_by_key(|&(_, v)| v)
        .map(|(c, _)| c)
}

/// Whether the defenders win against the double-dummy declarer when they
/// play double dummy too (they could, knowing every card).
fn defence_can_win(game: &Game) -> bool {
    let Some(declaration) = game.declaration() else {
        return false;
    };
    let goal = if declaration.is_null() {
        Goal::NullDeclarerTrick
    } else {
        Goal::DefenderTrick
    };
    let layout = Layout {
        hands: [
            game.hand(Seat::ALL[0]).to_vec(),
            game.hand(Seat::ALL[1]).to_vec(),
            game.hand(Seat::ALL[2]).to_vec(),
        ],
        trick_lead: game.trick_lead(),
        trick: Vec::new(),
        skat_points: 0,
    };
    let values = dd_solve(declaration.game(), goal, &layout);
    let defender_leads = !game.trick_lead().is_declarer();
    let best = if defender_leads {
        values.iter().map(|&(_, v)| v).max()
    } else {
        values.iter().map(|&(_, v)| v).min()
    };
    // Null: the defenders win when the declarer takes a trick (1);
    // Schwarz announced: when they take one (1).
    best == Some(1)
}

fn games() -> u32 {
    std::env::var("KNOWLEDGE_GAMES")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(200)
}

/// Deals (and declares) the first `n` games from seed 0 on that `accept`
/// takes; `accept` declares the game and returns the marked cards.
fn deals(
    n: u32,
    accept: impl Fn(&mut Game, &mut Rng) -> Option<Vec<Card>>,
) -> Vec<(Game, Vec<Card>)> {
    let mut out = Vec::new();
    let mut seed = 0u64;
    while out.len() < n as usize && seed < 2_000_000 {
        let mut rng = Rng::from_seed(seed);
        let forehand = Seat::ALL[usize::try_from(seed % 3).unwrap_or(0)];
        let mut game = Game::deal(&mut rng, forehand);
        if let Some(marked) = accept(&mut game, &mut rng) {
            out.push((game, marked));
        }
        seed += 1;
    }
    out
}

fn measure(title: &str, deals: &[(Game, Vec<Card>)], perfect: bool, rows: &[(&str, Defence)]) {
    println!("\n{title}");
    for (label, defence) in rows {
        let mut tally = Tally::default();
        for (i, (game, marked)) in deals.iter().enumerate() {
            play(
                game.clone(),
                *defence,
                marked,
                7 + i as u64,
                perfect,
                &mut tally,
            );
        }
        println!("{}", tally.row(label));
    }
}

fn jacks(hand: &[Card]) -> usize {
    hand.iter().filter(|c| c.rank == Rank::Jack).count()
}

fn aces(hand: &[Card]) -> usize {
    hand.iter().filter(|c| c.rank == Rank::Ace).count()
}

#[test]
#[ignore = "measurement, run with --ignored in release"]
fn defenders_with_knowledge_of_the_declarers_cards() {
    let n = games();
    let rows_open = [
        ("heuristic", Defence::Heuristic),
        ("pimc 8:3", pimc(8, 3, Pins::None)),
        ("pimc 12:3", pimc(12, 3, Pins::None)),
        ("pimc 8:0", pimc(8, 0, Pins::None)),
        ("pimc 12:0", pimc(12, 0, Pins::None)),
    ];
    // Null Ouvert Hand with at most two exposed cards.
    let null = deals(n, |game, _| {
        let hand = game.hand(Seat::DECLARER).to_vec();
        if null_unsafe_cards(&hand, game.deck_cards()) > 2 {
            return None;
        }
        game.declare(Declaration::null(NullRisk::OuvertHand)).ok()?;
        defence_can_win(game).then(Vec::new)
    });
    measure(
        "Null Ouvert Hand (<= 2 exposed cards, defence wins double dummy), declarer double dummy",
        &null,
        true,
        &rows_open,
    );
    // Grand Ouvert (Hand): four jacks, or three jacks and three aces.
    let grand = deals(n, |game, _| {
        let hand = game.hand(Seat::DECLARER).to_vec();
        let strong = jacks(&hand) == 4 && aces(&hand) >= 2 || jacks(&hand) >= 3 && aces(&hand) >= 3;
        if !strong {
            return None;
        }
        game.declare(Declaration::grand(RiskLevel::Ouvert)).ok()?;
        defence_can_win(game).then(Vec::new)
    });
    measure(
        "Grand Ouvert Hand (strong hands, defence wins double dummy), declarer double dummy",
        &grand,
        true,
        &rows_open,
    );
    // Durchsichtig: the heuristic's game after the skat, three of the
    // declarer's ten cards marked (drawn at random).
    let marked = deals(n, |game, rng| {
        DeclarerHeuristic.declare(game).ok()?;
        if game
            .declaration()
            .is_some_and(|d| d.game() == GameType::Null)
        {
            return None;
        }
        let mut hand = game.hand(Seat::DECLARER).to_vec();
        rng.shuffle(&mut hand);
        hand.truncate(3);
        Some(hand)
    });
    measure(
        "Suit/grand, 3 declarer cards Durchsichtig",
        &marked,
        false,
        &[
            ("heuristic", Defence::Heuristic),
            ("pimc 8:3 without pins", pimc(8, 3, Pins::None)),
            ("pimc 8:3 with pins", pimc(8, 3, Pins::Marked)),
            ("pimc 12:3 without pins", pimc(12, 3, Pins::None)),
            ("pimc 12:3 with pins", pimc(12, 3, Pins::Marked)),
        ],
    );
}

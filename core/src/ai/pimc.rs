//! Perfect Information Monte Carlo (DESIGN 10.2, stage 2).
//!
//! For every decision the unknown cards are dealt [`PimcConfig::samples`]
//! times consistently with what the seat knows ([`Knowledge`]), every
//! legal move is solved with perfect information ([`solver`]) on each
//! deal, and the move with the best mean wins. Ties go to the heuristic's
//! move, so PIMC never looks sillier than stage 1 where it sees no
//! difference.
//!
//! What a move is worth on one deal:
//!
//! - Suit and grand: the declarer's final Augen, plus a bonus for reaching
//!   the win threshold and for Schneider (the defenders count the same
//!   number negated). Thresholds come from [`Hints`]-free
//!   [`PimcPolicy::with_thresholds`] when house rules move them.
//! - Null: whether the declarer takes a trick.
//! - Ramsch: the Augen the two other seats take (every seat for itself,
//!   paranoid: the other two are assumed to play against it).
//!
//! Defenders bias the deal by the declaration: the declarer of a suit game
//! is dealt at least four trumps, of a grand at least two jacks, of a Null
//! at most three unsafe cards (counting the cards he already played), if a
//! few tries find such a deal.
//!
//! Determinism: the samples come from a generator seeded with the game's
//! seed, the number of cards played so far and the seat, never from the
//! caller's streams. Equal seed and equal moves give equal decisions.

use serde::{Deserialize, Serialize};

use super::sample::{Hints, Knowledge, Sample};
use super::solver::{self, Goal, Layout, Limits, Table};
use super::{DeclarerHeuristic, DefenderHeuristic, Policy};
use crate::cards::{PlayingCard, Rank};
use crate::game::{Seat, SeatView};
use crate::rng::Rng;
use crate::rules::{self, GameType, Thresholds};

/// Strength of a PIMC policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PimcConfig {
    /// Deals per decision.
    pub samples: u32,
    /// Tricks searched exactly per deal, the current one included; a
    /// static estimate covers the rest. `0` searches to the end.
    #[serde(default)]
    pub depth: u32,
}

impl Default for PimcConfig {
    fn default() -> PimcConfig {
        PimcConfig {
            samples: 20,
            depth: 0,
        }
    }
}

/// The defenders' policy, as data in `run.ron` (one entry per section).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum Opponents {
    /// Stage 1: the heuristic ([`DefenderHeuristic`]).
    #[default]
    Heuristic,
    /// Stage 2: PIMC with this many deals and this search depth.
    Pimc {
        /// Deals per decision.
        samples: u32,
        /// Tricks searched exactly, `0` = to the end.
        #[serde(default)]
        depth: u32,
    },
}

impl Opponents {
    /// The PIMC configuration, `None` for the heuristic.
    #[must_use]
    pub const fn pimc(self) -> Option<PimcConfig> {
        match self {
            Opponents::Heuristic => None,
            Opponents::Pimc { samples, depth } => Some(PimcConfig { samples, depth }),
        }
    }
}

/// Bonus in Augen for reaching the win threshold.
const WIN_BONUS: i64 = 40;
/// Bonus in Augen for Schneider (either side).
const SCHNEIDER_BONUS: i64 = 20;
/// Weight of one Null trick (the only thing that counts).
const NULL_WEIGHT: i64 = 100;
/// Most cards a Null declarer's ten cards may have that a lower card of
/// the same suit can be led under (see [`super::null_exposure`]).
const NULL_MAX_UNSAFE: usize = 3;
/// Tries to find a deal that fits the declaration before taking any.
const BIAS_TRIES: u32 = 12;
/// Search nodes per probe after which the remaining tricks are estimated
/// (about 3 ms natively): bounds the rare deals that explode.
const NODE_LIMIT: u64 = 6_000;
/// Search nodes per decision after which no further deal is solved (the
/// deals solved so far decide): about 25 ms natively, so the slowest
/// decisions of the strongest stage stay within a phone's budget.
const DECISION_NODES: u64 = 48_000;
/// Tricks left from which a Ramsch is searched (before: heuristic).
const RAMSCH_EXACT_TRICKS: usize = 5;
/// Transposition table size (2^bits slots).
const TABLE_BITS: u32 = 15;

/// PIMC card play for any seat.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PimcPolicy {
    config: PimcConfig,
    seed: u64,
    thresholds: Thresholds,
}

/// What one decision found, for tests and measurements.
#[derive(Debug, Clone, PartialEq)]
pub struct Decision<C> {
    /// The chosen card.
    pub card: C,
    /// Mean value per legal move (same order as the legal moves).
    pub means: Vec<(C, f64)>,
    /// Deals solved.
    pub samples: u32,
    /// Search nodes over all deals.
    pub nodes: u64,
}

/// Mix a seed with the decision's coordinates.
fn decision_seed(seed: u64, played: usize, seat: Seat) -> u64 {
    let mut z = seed ^ (u64::try_from(played).unwrap_or(0) << 8) ^ (seat.index() as u64);
    z = z.wrapping_add(0x9E37_79B9_7F4A_7C15);
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

impl PimcPolicy {
    /// A policy for one game. `seed` should differ per game (the run uses
    /// its seed and the number of games dealt).
    #[must_use]
    pub fn new(config: PimcConfig, seed: u64) -> PimcPolicy {
        PimcPolicy {
            config,
            seed,
            thresholds: Thresholds::default(),
        }
    }

    /// The same policy judging wins and Schneider by `thresholds` (house
    /// rules may lower the win threshold).
    #[must_use]
    pub fn with_thresholds(mut self, thresholds: Thresholds) -> PimcPolicy {
        self.thresholds = thresholds;
        self
    }

    /// The configuration.
    #[must_use]
    pub const fn config(&self) -> PimcConfig {
        self.config
    }

    /// Choose a card with extra knowledge (see [`Hints`]).
    #[must_use]
    pub fn choose_with<C: PlayingCard>(
        &self,
        view: &SeatView<'_, C>,
        hints: &Hints<C>,
    ) -> Option<C> {
        self.decide(view, hints).map(|d| d.card)
    }

    /// The heuristic's move for this seat: the tie breaker and the fallback.
    fn heuristic<C: PlayingCard>(view: &SeatView<'_, C>) -> Option<C> {
        if view.seat.is_declarer() {
            DeclarerHeuristic.choose(view)
        } else {
            DefenderHeuristic.choose(view)
        }
    }

    /// The full decision with the mean value of every legal move.
    #[must_use]
    pub fn decide<C: PlayingCard>(
        &self,
        view: &SeatView<'_, C>,
        hints: &Hints<C>,
    ) -> Option<Decision<C>> {
        let legal = &view.legal_moves;
        let fallback = Self::heuristic(view).or_else(|| legal.first().copied())?;
        let trivial = Decision {
            card: fallback,
            means: Vec::new(),
            samples: 0,
            nodes: 0,
        };
        if legal.len() < 2 || self.config.samples == 0 {
            return Some(trivial);
        }
        // A Ramsch has no threshold to probe; the exact search is cheap
        // only near the end.
        if view.declaration.is_ramsch() && view.hand.len() > RAMSCH_EXACT_TRICKS {
            return Some(trivial);
        }
        let Some(knowledge) = Knowledge::from_view(view, hints) else {
            return Some(trivial);
        };
        let game = view.declaration.game();
        let goal = if view.declaration.is_ramsch() {
            Goal::RamschAvoid(view.seat)
        } else if view.declaration.is_null() {
            Goal::NullDeclarerTrick
        } else {
            Goal::DeclarerPoints
        };
        let played = view.tricks.len() * 3 + view.trick.len();
        let mut rng = Rng::from_seed(decision_seed(self.seed, played, view.seat));
        let declarer_points: u32 = view
            .tricks
            .iter()
            .filter(|t| t.winner.is_declarer())
            .map(|t| t.card_points())
            .sum();
        let played_points: u32 = view
            .tricks
            .iter()
            .map(|t| t.card_points())
            .chain(view.trick.iter().map(|c| c.card_points()))
            .sum();
        let trick_points: u32 = view.trick.iter().map(|c| c.card_points()).sum();
        let declarer_played: Vec<C> = view
            .tricks
            .iter()
            .flat_map(|t| {
                (0..3)
                    .filter(|&i| t.seat_of(i).is_declarer())
                    .map(|i| t.cards[i])
            })
            .chain(
                (0..view.trick.len())
                    .filter(|&i| crate::game::seat_at(view.trick_lead, i).is_declarer())
                    .map(|i| view.trick[i]),
            )
            .collect();
        let bias = !view.seat.is_declarer()
            && !view.declaration.is_ramsch()
            && view.declarer_hand.is_none();
        let mut table = Table::new(TABLE_BITS);
        let mut sums = vec![0i64; legal.len()];
        let mut nodes = 0;
        let mut solved = 0;
        for _ in 0..self.config.samples {
            if solved > 0 && nodes >= DECISION_NODES {
                break;
            }
            let sample = if bias {
                let mut chosen = knowledge.sample(&mut rng);
                let mut tries = 1;
                while !fits_declaration(game, &chosen, &declarer_played, view.deck)
                    && tries < BIAS_TRIES
                {
                    chosen = knowledge.sample(&mut rng);
                    tries += 1;
                }
                chosen
            } else {
                knowledge.sample(&mut rng)
            };
            let skat_points: u32 = sample.skat.iter().map(|c| c.card_points()).sum();
            let layout = Layout {
                hands: sample.hands,
                trick_lead: view.trick_lead,
                trick: view.trick.to_vec(),
                skat_points,
            };
            let live: u32 = layout
                .hands
                .iter()
                .flatten()
                .chain(&layout.trick)
                .map(|c| c.card_points())
                .sum();
            let points = Points {
                base: declarer_points + skat_points,
                live,
                total: played_points - trick_points + live + skat_points,
            };
            let mut utilities = vec![0i64; legal.len()];
            let Some(n) = self.evaluate(
                view.seat,
                game,
                goal,
                &layout,
                points,
                &mut table,
                &mut utilities,
            ) else {
                continue;
            };
            nodes += n;
            for (sum, u) in sums.iter_mut().zip(&utilities) {
                *sum += u;
            }
            solved += 1;
        }
        if solved == 0 {
            return Some(trivial);
        }
        let best = sums.iter().copied().max()?;
        let card = if sums
            .iter()
            .zip(legal)
            .any(|(&s, &c)| s == best && c == fallback)
        {
            fallback
        } else {
            sums.iter()
                .zip(legal)
                .find(|&(&s, _)| s == best)
                .map(|(_, &c)| c)?
        };
        let means = legal
            .iter()
            .zip(&sums)
            .map(|(&c, &s)| (c, s as f64 / f64::from(solved)))
            .collect();
        Some(Decision {
            card,
            means,
            samples: solved,
            nodes,
        })
    }

    /// Solve one deal and write what every legal move is worth to `seat`
    /// into `out`. Returns the nodes searched, `None` if the deal could not
    /// be solved.
    #[allow(clippy::too_many_arguments)]
    fn evaluate<C: PlayingCard>(
        &self,
        seat: Seat,
        game: GameType,
        goal: Goal,
        layout: &Layout<C>,
        points: Points,
        table: &mut Table,
        out: &mut [i64],
    ) -> Option<u64> {
        let limits = Limits {
            depth: self.config.depth,
            nodes: NODE_LIMIT,
        };
        let mut nodes = 0;
        let len = out.len();
        let mut probe = |threshold: i64, table: &mut Table| -> Option<Vec<bool>> {
            if threshold <= 0 {
                return Some(vec![true; len]);
            }
            if threshold > i64::from(points.live) {
                return Some(vec![false; len]);
            }
            let threshold = i32::try_from(threshold).ok()?;
            let (moves, n) = solver::probe_with(game, goal, layout, threshold, limits, table);
            nodes += n;
            (moves.len() == len).then(|| moves.into_iter().map(|(_, r)| r).collect())
        };
        match goal {
            Goal::NullDeclarerTrick => {
                let caught = probe(1, table)?;
                let sign = if seat.is_declarer() { -1 } else { 1 };
                for (u, c) in out.iter_mut().zip(caught) {
                    *u = sign * NULL_WEIGHT * i64::from(c);
                }
            }
            Goal::RamschAvoid(_) => {
                let exact = Limits::default();
                let (values, n) = solver::solve_with(game, goal, layout, exact, table);
                nodes += n;
                if values.len() != out.len() {
                    return None;
                }
                for (u, (_, v)) in out.iter_mut().zip(values) {
                    *u = i64::from(v);
                }
            }
            Goal::DeclarerPoints => {
                let base = i64::from(points.base);
                let total = i64::from(points.total);
                let limit = i64::from(self.thresholds.schneider_limit);
                let wins = probe(i64::from(self.thresholds.win) - base, table)?;
                for (u, &w) in out.iter_mut().zip(&wins) {
                    *u = WIN_BONUS * i64::from(w);
                }
                // When the deal decides nothing about the win, Schneider
                // tells the moves apart.
                if wins.iter().all(|&w| w == wins[0]) {
                    if wins[0] {
                        let schneider = probe(total - limit - base, table)?;
                        for (u, s) in out.iter_mut().zip(schneider) {
                            *u += SCHNEIDER_BONUS * i64::from(s);
                        }
                    } else {
                        let escaped = probe(limit + 1 - base, table)?;
                        for (u, e) in out.iter_mut().zip(escaped) {
                            *u -= SCHNEIDER_BONUS * i64::from(!e);
                        }
                    }
                }
                if !seat.is_declarer() {
                    for u in out.iter_mut() {
                        *u = -*u;
                    }
                }
            }
        }
        Some(nodes)
    }
}

/// Augen that decide a point game on one deal.
#[derive(Debug, Clone, Copy)]
struct Points {
    /// The declarer's Augen so far: tricks and skat.
    base: u32,
    /// Augen still in the hands and on the table.
    live: u32,
    /// All Augen of the game.
    total: u32,
}

/// Whether a deal gives the declarer a hand that fits his declaration.
fn fits_declaration<C: PlayingCard>(
    game: GameType,
    sample: &Sample<C>,
    played: &[C],
    deck: &[C],
) -> bool {
    let cards = sample.hands[0].iter().chain(played);
    match game {
        GameType::Suit(_) => cards.filter(|&&c| rules::is_trump(game, c)).count() >= 4,
        GameType::Grand => {
            cards
                .filter(|c| c.face().is_some_and(|f| f.rank == Rank::Jack))
                .count()
                >= 2
        }
        GameType::Null => {
            let hand: Vec<C> = cards.copied().collect();
            super::null_unsafe_cards(&hand, deck) <= NULL_MAX_UNSAFE
        }
    }
}

impl<C: PlayingCard> Policy<C> for PimcPolicy {
    fn choose(&self, view: &SeatView<'_, C>) -> Option<C> {
        self.choose_with(view, &Hints::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cards::{Card, CardKind, Suit, standard_deck};
    use crate::deck::DealtCard;
    use crate::game::{Game, Phase};
    use crate::rules::{Declaration, NullRisk, RiskLevel};

    fn weak() -> PimcConfig {
        PimcConfig {
            samples: 3,
            depth: 2,
        }
    }

    /// Play a dealt game to the end; `pimc_seats` play PIMC, the others
    /// their heuristic. Every card must be legal.
    fn play_out<C: PlayingCard>(mut game: Game<C>, pimc_seats: &[Seat], seed: u64) -> Game<C> {
        let policy = PimcPolicy::new(weak(), seed);
        while let Some(seat) = game.current_seat() {
            let view = game.view(seat).expect("playing");
            let card = if pimc_seats.contains(&seat) {
                policy.choose(&view)
            } else {
                PimcPolicy::heuristic(&view)
            }
            .expect("a card");
            assert!(view.legal_moves.contains(&card), "{card} is legal");
            game.play(seat, card).expect("legal move");
        }
        game
    }

    fn declare<C: PlayingCard>(game: &mut Game<C>, declaration: Declaration) {
        game.declare(declaration).expect("declare");
        if game.phase() == Phase::Discarding {
            let hand = game.hand(Seat::DECLARER).to_vec();
            let discard = if declaration.is_null() {
                DeclarerHeuristic.choose_null_discard(&hand, game.deck_cards())
            } else {
                DeclarerHeuristic.choose_discard(&hand, declaration.game())
            }
            .expect("discard");
            game.discard(discard).expect("discard");
        }
    }

    #[test]
    fn pimc_plays_only_legal_cards_in_every_game_type() {
        for seed in 0..9u64 {
            let mut rng = Rng::from_seed(seed);
            let mut game = Game::deal(&mut rng, Seat::ALL[usize::try_from(seed % 3).unwrap_or(0)]);
            let declaration = match seed % 3 {
                0 => DeclarerHeuristic.choose_declaration(game.hand(Seat::DECLARER)),
                1 => Declaration::null(NullRisk::Normal),
                _ => Declaration::Ramsch,
            };
            declare(&mut game, declaration);
            let game = play_out(game, &Seat::ALL, seed);
            assert!(game.value().is_some(), "seed {seed} finished");
        }
    }

    /// Blanks, a rank bonus and a card that never wins.
    fn roguelike_deck() -> Vec<DealtCard> {
        standard_deck()
            .into_iter()
            .zip(0u32..)
            .map(|(card, id)| {
                let kind = if card.rank == crate::cards::Rank::Eight {
                    CardKind::Blank
                } else {
                    CardKind::Normal(card)
                };
                let mut dealt = DealtCard::plain(id, kind);
                if card == Card::new(Suit::Spades, crate::cards::Rank::King) {
                    dealt.rank_bonus = 2;
                }
                if card == Card::new(Suit::Hearts, crate::cards::Rank::Ace) {
                    dealt.never_wins = true;
                }
                dealt
            })
            .chain((40..43).map(|id| DealtCard::plain(id, CardKind::Blank)))
            .collect()
    }

    #[test]
    fn pimc_plays_the_roguelike_deck_ouvert_and_with_open_partners() {
        for seed in 0..6u64 {
            let mut rng = Rng::from_seed(seed);
            let mut game = Game::deal_from(&mut rng, roguelike_deck(), Seat::ALL[1]).expect("deal");
            if seed % 2 == 0 {
                game.open_defender_hands();
            }
            let mut declaration = DeclarerHeuristic.choose_declaration(game.hand(Seat::DECLARER));
            if seed % 3 == 0 {
                declaration = declaration.with_risk(RiskLevel::Ouvert);
            }
            declare(&mut game, declaration);
            let game = play_out(game, &Seat::ALL, seed);
            assert!(game.value().is_some());
        }
    }

    #[test]
    fn equal_seed_gives_equal_decisions() {
        let mut rng = Rng::from_seed(21);
        let mut game = Game::deal(&mut rng, Seat::ALL[1]);
        let declaration = DeclarerHeuristic.choose_declaration(game.hand(Seat::DECLARER));
        declare(&mut game, declaration);
        let view = game.view(Seat::ALL[1]).expect("view");
        let config = PimcConfig {
            samples: 6,
            depth: 2,
        };
        let a = PimcPolicy::new(config, 5).decide(&view, &Hints::default());
        let b = PimcPolicy::new(config, 5).decide(&view, &Hints::default());
        assert_eq!(a, b);
        let a = a.expect("decision");
        assert_eq!(a.samples, 6);
        assert_eq!(a.means.len(), view.legal_moves.len());
    }

    #[test]
    fn with_everything_open_pimc_plays_a_double_dummy_best_move() {
        // Ouvert (declarer open) plus "Die Zwillinge" (partner open): a
        // defender knows every card, so each deal is the real one and the
        // chosen move must reach the solver's best value.
        let mut checked = 0;
        for seed in 0..40u64 {
            let mut rng = Rng::from_seed(seed);
            let mut game = Game::deal(&mut rng, Seat::DECLARER);
            game.open_defender_hands();
            let declaration = DeclarerHeuristic
                .choose_declaration(game.hand(Seat::DECLARER))
                .with_risk(RiskLevel::Ouvert);
            declare(&mut game, declaration);
            // Play heuristically until five tricks are left.
            while game.tricks().len() < 5 {
                let seat = game.current_seat().expect("playing");
                let view = game.view(seat).expect("view");
                let card = PimcPolicy::heuristic(&view).expect("card");
                game.play(seat, card).expect("legal");
            }
            let Some(seat) = game.current_seat() else {
                continue;
            };
            if seat.is_declarer() {
                continue;
            }
            let view = game.view(seat).expect("view");
            if view.legal_moves.len() < 2 {
                continue;
            }
            let policy = PimcPolicy::new(
                PimcConfig {
                    samples: 2,
                    depth: 0,
                },
                1,
            );
            let card = policy.choose(&view).expect("card");
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
            let values = solver::solve(declaration.game(), Goal::DeclarerPoints, &layout);
            let base: i32 = game
                .seat_card_points()
                .first()
                .map(|&p| i32::try_from(p).unwrap_or(0))
                .unwrap_or(0);
            let win = |v: i32| base + v >= 61;
            let best_win = values.iter().all(|&(_, v)| win(v));
            let chosen = values
                .iter()
                .find(|(c, _)| *c == card)
                .map(|&(_, v)| v)
                .expect("chosen card is legal");
            // The defenders minimize: if any move stops the declarer, the
            // chosen one does.
            if !best_win {
                assert!(!win(chosen), "seed {seed}: {card} lets the declarer win");
            }
            checked += 1;
        }
        assert!(checked >= 5, "enough positions checked ({checked})");
    }
}

#[cfg(test)]
mod bench {
    use super::*;
    use crate::game::{Game, Phase};

    /// `cargo test --release -p skat_core bench_pimc -- --ignored --nocapture`
    #[test]
    #[ignore = "measurement"]
    fn bench_pimc() {
        let samples: u32 = std::env::var("PIMC_SAMPLES")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(1);
        let depth: u32 = std::env::var("PIMC_DEPTH")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(0);
        let policy = PimcPolicy::new(PimcConfig { samples, depth }, 7);
        let null = std::env::var("PIMC_NULL").is_ok();
        let mut per_trick = [(0u64, 0u64, 0u64, 0u64); 10];
        for seed in 0..40 {
            let mut rng = Rng::from_seed(seed);
            let mut game = Game::deal(&mut rng, Seat::ALL[usize::try_from(seed % 3).unwrap_or(0)]);
            let decl = if null {
                crate::rules::Declaration::null(crate::rules::NullRisk::Hand)
            } else {
                DeclarerHeuristic.choose_declaration(game.hand(Seat::DECLARER))
            };
            game.declare(decl).expect("declare");
            if game.phase() == Phase::Discarding {
                let d = DeclarerHeuristic
                    .choose_discard(game.hand(Seat::DECLARER), decl.game())
                    .expect("discard");
                game.discard(d).expect("discard");
            }
            while let Some(seat) = game.current_seat() {
                let view = game.view(seat).expect("view");
                let trick = view.tricks.len();
                let card = if seat.is_declarer() {
                    DeclarerHeuristic.choose(&view).expect("card")
                } else {
                    let start = std::time::Instant::now();
                    let d = policy.decide(&view, &Hints::default()).expect("decision");
                    let us = u64::try_from(start.elapsed().as_micros()).unwrap_or(0);
                    let e = &mut per_trick[trick];
                    e.0 += 1;
                    e.1 += us;
                    e.2 = e.2.max(us);
                    e.3 += d.nodes;
                    d.card
                };
                game.play(seat, card).expect("legal");
            }
        }
        for (trick, (n, us, max, nodes)) in per_trick.iter().enumerate() {
            if *n > 0 {
                println!(
                    "trick {}: {n} decisions, mean {:.2} ms, max {:.2} ms, mean nodes {}",
                    trick + 1,
                    *us as f64 / *n as f64 / 1000.0,
                    *max as f64 / 1000.0,
                    nodes / n
                );
            }
        }
    }
}

#[cfg(test)]
mod bench_parts {
    use super::*;
    use crate::game::Game;

    #[test]
    #[ignore = "measurement"]
    fn bench_parts() {
        let mut rng = Rng::from_seed(3);
        let mut game = Game::deal(&mut rng, Seat::ALL[1]);
        let decl = DeclarerHeuristic.choose_declaration(game.hand(Seat::DECLARER));
        game.declare(
            crate::rules::Declaration::new(decl.game(), crate::rules::RiskLevel::Hand).expect("d"),
        )
        .expect("declare");
        let view = game.view(Seat::ALL[1]).expect("view");
        let knowledge = Knowledge::from_view(&view, &Hints::default()).expect("k");
        let start = std::time::Instant::now();
        let mut fits = 0;
        for _ in 0..10_000 {
            let s = knowledge.sample(&mut rng);
            fits += usize::from(fits_declaration(
                view.declaration.game(),
                &s,
                &[],
                view.deck,
            ));
        }
        println!(
            "sample: {:.2} us, fits {fits}/10000",
            start.elapsed().as_secs_f64() * 100.0
        );
        let s = knowledge.sample(&mut rng);
        let layout = Layout {
            hands: s.hands,
            trick_lead: view.trick_lead,
            trick: Vec::new(),
            skat_points: 0,
        };
        let mut table = Table::new(TABLE_BITS);
        let start = std::time::Instant::now();
        let mut nodes = 0;
        for _ in 0..1000 {
            nodes += solver::probe_with(
                view.declaration.game(),
                Goal::DeclarerPoints,
                &layout,
                40,
                Limits { depth: 1, nodes: 0 },
                &mut table,
            )
            .1;
        }
        println!(
            "probe depth 1: {:.2} us, nodes {}",
            start.elapsed().as_secs_f64() * 1000.0,
            nodes / 1000
        );
        let start = std::time::Instant::now();
        for _ in 0..1000 {
            let _ = Table::new(TABLE_BITS);
        }
        println!("table: {:.2} us", start.elapsed().as_secs_f64() * 1000.0);
    }
}

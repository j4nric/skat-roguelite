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
//!   number negated). The win threshold and the Schneider limit are the
//!   Skatordnung's unless house rules move them
//!   ([`PimcPolicy::with_thresholds`]). Augen count with their sign (a
//!   card below zero, "Ramponiert", costs its taker) and with the extra
//!   Augen of card effects so far ([`SeatView::seat_points`]).
//! - Schwarz announced (also every suit or grand Ouvert): whether the
//!   defenders take a trick, the only thing that decides the game.
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
use super::solver::{self, Goal, Layout, Limits, Probes, Table};
use super::{DeclarerHeuristic, DefenderHeuristic, Policy};
use crate::cards::{PlayingCard, Rank};
use crate::game::{Seat, SeatView};
use crate::rng::{Rng, mix64};
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
    mix64(seed ^ (u64::try_from(played).unwrap_or(0) << 8) ^ (seat.index() as u64))
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
        } else if view.declaration.schwarz_declared() {
            // Schwarz angesagt and every suit or grand Ouvert: lost with
            // the first defender trick.
            Goal::DefenderTrick
        } else {
            Goal::DeclarerPoints
        };
        let played = view.tricks.len() * 3 + view.trick.len();
        let mut rng = Rng::from_seed(decision_seed(self.seed, played, view.seat));
        // Augen so far with their sign, extra Augen included.
        let [declarer_points, first, second] = view.seat_points;
        let defender_points = first + second;
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
        // One table for every deal of the decision, reused across
        // decisions: each search starts a generation of its own.
        let (sums, nodes, solved) = solver::with_table(TABLE_BITS, |table| {
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
                let skat_points: i64 = sample.skat.iter().map(|c| c.signed_points()).sum();
                let layout = Layout {
                    hands: sample.hands,
                    trick_lead: view.trick_lead,
                    trick: view.trick.to_vec(),
                    skat_points: i32::try_from(skat_points).unwrap_or(0),
                };
                let (mut gain, mut loss) = (0, 0);
                for c in layout.hands.iter().flatten().chain(&layout.trick) {
                    let p = c.signed_points();
                    gain += p.max(0);
                    loss += p.min(0);
                }
                let base = declarer_points + skat_points;
                let points = Points {
                    base,
                    gain,
                    loss,
                    total: base + defender_points + gain + loss,
                };
                let mut utilities = vec![0i64; legal.len()];
                let Some(n) = self.evaluate(
                    view.seat,
                    game,
                    goal,
                    &layout,
                    points,
                    table,
                    legal,
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
            (sums, nodes, solved)
        });
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

    /// Solve one deal and write what every legal move (`legal`, in the
    /// order of `out`) is worth to `seat` into `out`. The search finds the
    /// moves by the rules; the results are matched by card. Returns the
    /// nodes searched, `None` if the deal could not be solved.
    #[allow(clippy::too_many_arguments)]
    fn evaluate<C: PlayingCard>(
        &self,
        seat: Seat,
        game: GameType,
        goal: Goal,
        layout: &Layout<C>,
        points: Points,
        table: &mut Table,
        legal: &[C],
        out: &mut [i64],
    ) -> Option<u64> {
        let limits = Limits {
            depth: self.config.depth,
            nodes: NODE_LIMIT,
        };
        let mut nodes = 0;
        if goal_is_ramsch(goal) {
            let exact = Limits::default();
            let (values, n) = solver::solve_with(game, goal, layout, exact, table);
            nodes += n;
            let values = pick(&values, legal)?;
            for (u, v) in out.iter_mut().zip(values) {
                *u = i64::from(v);
            }
            return Some(nodes);
        }
        let len = out.len();
        // The deal is prepared for the solver once, at the first probe
        // that needs a search; every probe is still a search of its own.
        let mut table = Some(table);
        let mut probes: Option<Option<Probes<'_, C>>> = None;
        // What the rest can bring: in Augen every one below zero up to
        // every positive one; a trick goal counts tricks (a trick of
        // sevens, eights and nines has no Augen).
        let (min, max) = if goal == Goal::DeclarerPoints {
            (points.loss, points.gain)
        } else {
            (0, 1)
        };
        let mut probe = |threshold: i64| -> Option<Vec<bool>> {
            if threshold <= min {
                return Some(vec![true; len]);
            }
            if threshold > max {
                return Some(vec![false; len]);
            }
            let threshold = i32::try_from(threshold).ok()?;
            let prepared = probes.get_or_insert_with(|| {
                table
                    .take()
                    .and_then(|t| Probes::new(game, goal, layout, limits, t))
            });
            let (moves, n) = match prepared {
                Some(p) => p.probe(threshold),
                None => (Vec::new(), 0),
            };
            nodes += n;
            pick(&moves, legal)
        };
        match goal {
            Goal::NullDeclarerTrick => {
                let caught = probe(1)?;
                let sign = if seat.is_declarer() { -1 } else { 1 };
                for (u, c) in out.iter_mut().zip(caught) {
                    *u = sign * NULL_WEIGHT * i64::from(c);
                }
            }
            Goal::DefenderTrick => {
                // Schwarz announced: one defender trick decides the game.
                let caught = probe(1)?;
                let sign = if seat.is_declarer() { -1 } else { 1 };
                for (u, c) in out.iter_mut().zip(caught) {
                    *u = sign * WIN_BONUS * i64::from(c);
                }
            }
            Goal::RamschAvoid(_) => {}
            Goal::DeclarerPoints => {
                let Points { base, total, .. } = points;
                let limit = i64::from(self.thresholds.schneider_limit);
                let wins = probe(i64::from(self.thresholds.win) - base)?;
                for (u, &w) in out.iter_mut().zip(&wins) {
                    *u = WIN_BONUS * i64::from(w);
                }
                // When the deal decides nothing about the win, Schneider
                // tells the moves apart.
                if wins.iter().all(|&w| w == wins[0]) {
                    if wins[0] {
                        let schneider = probe(total - limit - base)?;
                        for (u, s) in out.iter_mut().zip(schneider) {
                            *u += SCHNEIDER_BONUS * i64::from(s);
                        }
                    } else {
                        let escaped = probe(limit + 1 - base)?;
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

/// The search results of the cards in `legal`, in that order; `None` if a
/// card has none.
fn pick<C: PlayingCard, T: Copy>(results: &[(C, T)], legal: &[C]) -> Option<Vec<T>> {
    legal
        .iter()
        .map(|card| results.iter().find(|(c, _)| c == card).map(|&(_, r)| r))
        .collect()
}

/// Whether `goal` is a Ramsch (solved exactly, not probed).
const fn goal_is_ramsch(goal: Goal) -> bool {
    matches!(goal, Goal::RamschAvoid(_))
}

/// Augen that decide a point game on one deal, with their sign (Augen
/// below zero count against the taker) and extra Augen from card effects
/// included.
#[derive(Debug, Clone, Copy)]
struct Points {
    /// The declarer's Augen so far: tricks, extra Augen and skat.
    base: i64,
    /// Positive Augen still in the hands and on the table.
    gain: i64,
    /// Augen below zero still in the hands and on the table (at most 0).
    loss: i64,
    /// All Augen of the game: both sides' so far, the skat and the live
    /// ones.
    total: i64,
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
                .filter(|c| !rules::is_demoted_jack(**c))
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
    use crate::game::Game;
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
        DeclarerHeuristic
            .declare_as(game, declaration)
            .expect("declare");
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
            if declaration.is_null() {
                continue;
            }
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
            // Ouvert is Schwarz announced: one defender trick beats it.
            let values = solver::solve(declaration.game(), Goal::DefenderTrick, &layout);
            let chosen = values
                .iter()
                .find(|(c, _)| *c == card)
                .map(|&(_, v)| v)
                .expect("chosen card is legal");
            // If any move takes a trick, the chosen one does.
            if values.iter().any(|&(_, v)| v == 1) {
                assert_eq!(chosen, 1, "seed {seed}: {card} lets the declarer win");
            }
            checked += 1;
        }
        assert!(checked >= 5, "enough positions checked ({checked})");
    }
}

#[cfg(test)]
mod knowledge_tests {
    use super::*;
    use crate::cards::{Card, Rank, Suit, parse_cards, standard_deck};
    use crate::rules::{Declaration, NullRisk, RiskLevel};

    static NO_EDITIONS: [Vec<Card>; 3] = [Vec::new(), Vec::new(), Vec::new()];

    fn cards(s: &str) -> Vec<Card> {
        let mut cards = parse_cards(s).expect("cards");
        cards.sort();
        cards
    }

    /// The view of seat 1, who leads the next trick of an endgame with
    /// `hands` (the declarer's hand open, the partner's as `partner`
    /// says), `skat` known as `skat_known` says; `deck` holds the live
    /// cards and the skat.
    fn lead_view<'a>(
        declaration: Declaration,
        hands: &'a [Vec<Card>; 3],
        deck: &'a [Card],
        skat: [Card; 2],
        partner: bool,
    ) -> SeatView<'a, Card> {
        SeatView {
            seat: Seat::ALL[1],
            declaration,
            forehand: Seat::ALL[1],
            hand: &hands[1],
            trick_lead: Seat::ALL[1],
            trick: &[],
            tricks: &[],
            seat_points: [0; 3],
            skat: partner.then_some(skat),
            declarer_hand: Some(&hands[0]),
            partner_hand: partner.then_some(&hands[2][..]),
            deck,
            editions: &NO_EDITIONS,
            legal_moves: rules::legal_moves(declaration.game(), &hands[1], None),
        }
    }

    /// A random endgame of `tricks` tricks from `pool`, seat 1 to lead.
    fn endgame(
        rng: &mut Rng,
        pool: &[Card],
        tricks: usize,
    ) -> ([Vec<Card>; 3], [Card; 2], Vec<Card>) {
        let mut deck = pool.to_vec();
        rng.shuffle(&mut deck);
        let mut hands: [Vec<Card>; 3] = [Vec::new(), Vec::new(), Vec::new()];
        for (seat, hand) in hands.iter_mut().enumerate() {
            *hand = deck[seat * tricks..(seat + 1) * tricks].to_vec();
            hand.sort();
        }
        let skat = [deck[3 * tricks], deck[3 * tricks + 1]];
        let mut live = deck[..3 * tricks + 2].to_vec();
        live.sort();
        (hands, skat, live)
    }

    fn defender_values(game: GameType, goal: Goal, hands: &[Vec<Card>; 3]) -> Vec<(Card, i32)> {
        let layout = Layout {
            hands: hands.clone(),
            trick_lead: Seat::ALL[1],
            trick: Vec::new(),
            skat_points: 0,
        };
        solver::solve(game, goal, &layout)
    }

    #[test]
    fn against_an_announced_schwarz_the_defenders_play_for_one_trick() {
        // Grand Ouvert, every card known: whenever a lead takes a trick
        // (double dummy), PIMC leads one. Augen do not matter here; before
        // the trick goal the defenders weighed 61 Augen and fell back to
        // the heuristic.
        let mut rng = Rng::from_seed(31);
        let declaration = Declaration::grand(RiskLevel::Ouvert);
        let policy = PimcPolicy::new(
            PimcConfig {
                samples: 1,
                depth: 0,
            },
            3,
        );
        let mut heuristic_misses = 0;
        let mut checked = 0;
        for _ in 0..1500 {
            let (hands, skat, deck) = endgame(&mut rng, &standard_deck(), 4);
            let values = defender_values(GameType::Grand, Goal::DefenderTrick, &hands);
            if values.iter().all(|&(_, v)| v == values[0].1) {
                continue;
            }
            let view = lead_view(declaration, &hands, &deck, skat, true);
            let value = |card: Card| values.iter().find(|(c, _)| *c == card).map(|&(_, v)| v);
            let card = policy.choose(&view).expect("card");
            assert_eq!(value(card), Some(1), "{card} in {hands:?}");
            let heuristic = DefenderHeuristic.choose(&view).expect("card");
            heuristic_misses += usize::from(value(heuristic) == Some(0));
            checked += 1;
        }
        assert!(checked >= 20, "positions with a choice: {checked}");
        assert!(heuristic_misses > 0, "the heuristic misses some");
    }

    #[test]
    fn grand_ouvert_defender_leads_through_the_declarers_bare_ten() {
        // The declarer shows CJ DJ CT. Seat 1 holds D7 HK C8 and does not
        // know where SJ DQ H8 HQ ST lie (partner or skat). Only the C8
        // can take a trick: the declarer must follow with the CT and the
        // partner, void in clubs, trumps it whenever he holds the SJ. The
        // heuristic leads the HK into the declarer's trumps.
        let hands = [cards("DJ CT CJ"), cards("D7 HK C8"), cards("DQ H8 SJ")];
        let skat = [cards("HQ")[0], cards("ST")[0]];
        let mut deck: Vec<Card> = hands.iter().flatten().copied().chain(skat).collect();
        deck.sort();
        let declaration = Declaration::grand(RiskLevel::Ouvert);
        let view = lead_view(declaration, &hands, &deck, skat, false);
        let c8 = Card::new(Suit::Clubs, Rank::Eight);
        assert_ne!(DefenderHeuristic.choose(&view), Some(c8));
        let config = PimcConfig {
            samples: 8,
            depth: 0,
        };
        assert_eq!(PimcPolicy::new(config, 11).choose(&view), Some(c8));
    }

    #[test]
    fn null_endgame_without_augen_is_still_searched() {
        // Only sevens, eights and nines left: no Augen, but a Null is
        // decided by tricks. PIMC with every card known catches the
        // declarer whenever a lead does (double dummy).
        let low: Vec<Card> = standard_deck()
            .into_iter()
            .filter(|c| matches!(c.rank, Rank::Seven | Rank::Eight | Rank::Nine))
            .collect();
        let declaration = Declaration::null(NullRisk::OuvertHand);
        let policy = PimcPolicy::new(
            PimcConfig {
                samples: 1,
                depth: 0,
            },
            5,
        );
        let mut rng = Rng::from_seed(32);
        let mut checked = 0;
        for _ in 0..300 {
            let (hands, skat, deck) = endgame(&mut rng, &low, 3);
            let values = defender_values(GameType::Null, Goal::NullDeclarerTrick, &hands);
            if values.iter().all(|&(_, v)| v == values[0].1) {
                continue;
            }
            let view = lead_view(declaration, &hands, &deck, skat, true);
            let decision = policy.decide(&view, &Hints::default()).expect("decision");
            let value = values
                .iter()
                .find(|(c, _)| *c == decision.card)
                .map(|&(_, v)| v);
            assert_eq!(value, Some(1), "{} in {hands:?}", decision.card);
            checked += 1;
        }
        assert!(checked >= 20, "positions with a choice: {checked}");
    }
}

#[cfg(test)]
mod bench {
    use super::*;
    use crate::game::Game;

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
            DeclarerHeuristic
                .declare_as(&mut game, decl)
                .expect("declare");
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
mod bench_augen {
    use super::*;
    use crate::cards::{CardKind, standard_deck};
    use crate::deck::DealtCard;
    use crate::game::Game;

    /// The standard deck as dealt cards; `battered` sevens count 5 Augen
    /// below zero ("Ramponiert").
    fn deck(battered: bool) -> Vec<DealtCard> {
        standard_deck()
            .into_iter()
            .zip(0u32..)
            .map(|(card, id)| {
                let mut dealt = DealtCard::plain(id, CardKind::Normal(card));
                if battered && card.rank == Rank::Seven {
                    dealt.card_points = 0;
                    dealt.point_debt = 5;
                }
                dealt
            })
            .collect()
    }

    /// Declarer heuristic against PIMC defenders on decks with Augen below
    /// zero ("Ramponiert" sevens) and with extra Augen ("Erster Stich":
    /// the first trick counts twice for its winner). Prints the declarer's
    /// results and the defenders' decision time per variant.
    ///
    /// `cargo test --release -p skat_core bench_augen -- --ignored --nocapture`
    /// (`PIMC_GAMES`, `PIMC_SAMPLES`, `PIMC_DEPTH` override 600, 8, 3).
    #[test]
    #[ignore = "measurement"]
    fn bench_augen() {
        let env = |name: &str, default: u32| {
            std::env::var(name)
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(default)
        };
        let games = env("PIMC_GAMES", 600);
        let config = PimcConfig {
            samples: env("PIMC_SAMPLES", 8),
            depth: env("PIMC_DEPTH", 3),
        };
        for (name, battered, first_trick) in [
            ("plain", false, false),
            ("battered sevens", true, false),
            ("first trick twice", false, true),
        ] {
            let (mut won, mut schneider, mut point_games) = (0u32, 0u32, 0u32);
            let mut micros: Vec<u64> = Vec::new();
            let mut nodes = 0u64;
            for seed in 0..u64::from(games) {
                let mut rng = Rng::from_seed(seed);
                let forehand = Seat::ALL[usize::try_from(seed % 3).unwrap_or(0)];
                let mut game = Game::deal_from(&mut rng, deck(battered), forehand).expect("deal");
                let declaration = DeclarerHeuristic.declare(&mut game).expect("declare");
                let policy = PimcPolicy::new(config, mix64(seed));
                while let Some(seat) = game.current_seat() {
                    let view = game.view(seat).expect("view");
                    let card = if seat.is_declarer() || view.legal_moves.len() < 2 {
                        PimcPolicy::heuristic(&view).expect("card")
                    } else {
                        let start = std::time::Instant::now();
                        let d = policy.decide(&view, &Hints::default()).expect("decision");
                        micros.push(u64::try_from(start.elapsed().as_micros()).unwrap_or(0));
                        nodes += d.nodes;
                        d.card
                    };
                    let tricks = game.tricks().len();
                    game.play(seat, card).expect("legal");
                    if first_trick && tricks == 0 && game.tricks().len() == 1 {
                        let trick = &game.tricks()[0];
                        let extra = u32::try_from(trick.signed_points().max(0)).unwrap_or(0);
                        let winner = trick.winner;
                        game.add_bonus_points(winner, extra);
                    }
                }
                if declaration.is_null() || declaration.is_ramsch() {
                    continue;
                }
                let value = game.value().expect("finished");
                point_games += 1;
                won += u32::from(value.won);
                schneider += u32::from(value.schneider);
            }
            micros.sort_unstable();
            let n = micros.len().max(1);
            let mean = micros.iter().sum::<u64>() as f64 / n as f64 / 1000.0;
            let p95 = micros[(n * 95 / 100).min(n - 1)] as f64 / 1000.0;
            let max = micros.last().copied().unwrap_or(0) as f64 / 1000.0;
            println!(
                "{name}: suit/grand games {point_games}, declarer won {:.1}%, schneider {:.1}%; \
                 decisions {}, mean {mean:.2} ms, p95 {p95:.2} ms, max {max:.2} ms, mean nodes {}",
                100.0 * f64::from(won) / f64::from(point_games.max(1)),
                100.0 * f64::from(schneider) / f64::from(point_games.max(1)),
                micros.len(),
                nodes / u64::try_from(n).unwrap_or(1),
            );
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

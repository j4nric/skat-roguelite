//! Card play of the defenders in a run (DESIGN 10.3) and a PIMC card play
//! for the player's seat (simulator, `--bot pimc`).
//!
//! `run.ron` lists the defenders' policy per section
//! ([`crate::content::RunConfig::opponents`]): the heuristic, or PIMC with
//! a number of deals and a search depth. A section without entry takes the
//! last one; no entry at all means the heuristic everywhere.
//!
//! The samples of every decision come from a generator derived from the
//! run's seed, the number of games dealt and the cards played so far
//! ([`game_seed`]), so equal seeds and equal actions give equal games.

use super::{ActiveGame, Run};
use crate::ai::pimc::{Opponents, PimcConfig, PimcPolicy};
use crate::ai::sample::Hints;
use crate::ai::{DefenderHeuristic, Policy};
use crate::content::Content;
use crate::deck::DealtCard;
use crate::game::{Event, Seat, SeatView};

/// The defenders' policy in `section` (counted from 0).
#[must_use]
pub fn opponents_in(content: &Content, section: u32) -> Opponents {
    let list = &content.run.opponents;
    usize::try_from(section)
        .ok()
        .and_then(|i| list.get(i))
        .or_else(|| list.last())
        .copied()
        .unwrap_or_default()
}

/// Seed of the PIMC samples of the game in progress: the run's seed mixed
/// with the number of games dealt.
#[must_use]
pub fn game_seed(run: &Run) -> u64 {
    let mut z = run.seed ^ u64::from(run.games_dealt).wrapping_mul(0xD1B5_4A32_D192_ED03);
    z = (z ^ (z >> 29)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z ^ (z >> 32)
}

/// Completed tricks at each swap of the defenders' hands (boss "Der
/// Revolutionär"), read from the game's public log.
#[must_use]
pub fn defender_swaps<C>(log: &[Event<C>]) -> Vec<usize> {
    let mut tricks = 0;
    let mut swaps = Vec::new();
    for event in log {
        match event {
            Event::TrickWon { .. } => tricks += 1,
            Event::DefenderHandsSwapped { .. } => swaps.push(tricks),
            _ => {}
        }
    }
    swaps
}

/// The defender's card in the run's current game.
pub(super) fn defender_card(
    run: &Run,
    content: &Content,
    active: &ActiveGame,
    view: &SeatView<'_, DealtCard>,
) -> Option<DealtCard> {
    match opponents_in(content, run.series.section).pimc() {
        None => DefenderHeuristic.choose(view),
        Some(config) => {
            let hints = Hints {
                pinned: Vec::new(),
                defender_swaps: defender_swaps(active.game.log()),
            };
            PimcPolicy::new(config, game_seed(run))
                .with_thresholds(active.rules.thresholds.clone())
                .choose_with(view, &hints)
        }
    }
}

/// What the player knows beyond his seat's view: "Durchsichtig" cards
/// with their holder, the cards "Der Kiebitz" showed (moved along with a
/// swap of the defenders' hands) and the swaps themselves.
#[must_use]
pub fn player_hints(run: &Run) -> Hints<DealtCard> {
    let Some(active) = run.game() else {
        return Hints::default();
    };
    let swaps = defender_swaps(active.game.log());
    let mut pinned: Vec<(DealtCard, Option<Seat>)> = run
        .revealed_holders()
        .into_iter()
        .filter(|(_, holder)| *holder != Some(Seat::DECLARER))
        .collect();
    for &(card, seat) in &active.peeked {
        let holder = if swaps.len() % 2 == 1 {
            seat.partner().unwrap_or(seat)
        } else {
            seat
        };
        if !pinned.iter().any(|(c, _)| *c == card) {
            pinned.push((card, Some(holder)));
        }
    }
    Hints {
        pinned,
        defender_swaps: swaps,
    }
}

/// A PIMC card for the player's seat in the current game, with everything
/// the player is allowed to know ([`player_hints`]). `None` if the player
/// is not to play.
#[must_use]
pub fn player_card(run: &Run, config: PimcConfig) -> Option<DealtCard> {
    let active = run.game()?;
    let view = active.game.view(Seat::DECLARER)?;
    if view.legal_moves.is_empty() {
        return None;
    }
    PimcPolicy::new(config, game_seed(run) ^ 0x5EA7_0000)
        .with_thresholds(active.rules.thresholds.clone())
        .choose_with(&view, &player_hints(run))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::run::bot::RunBot;
    use crate::run::{RunEvent, RunPhase, RunStart};

    #[test]
    fn run_ron_names_the_opponents_per_section() {
        let content = Content::load().expect("content");
        assert_eq!(opponents_in(&content, 0), Opponents::Heuristic);
        let second = opponents_in(&content, 1)
            .pimc()
            .expect("PIMC from section 2");
        let last = opponents_in(&content, 4).pimc().expect("PIMC in section 5");
        assert!(last.samples > second.samples, "stronger later");
        assert_eq!(opponents_in(&content, 40), opponents_in(&content, 4));
        let mut none = content.clone();
        none.run.opponents.clear();
        assert_eq!(opponents_in(&none, 3), Opponents::Heuristic);
    }

    #[test]
    fn defender_swaps_are_counted_in_completed_tricks() {
        use crate::cards::Card;
        let log: Vec<Event<Card>> = vec![
            Event::DefenderHandsSwapped {
                hands: [Vec::new(), Vec::new(), Vec::new()],
            },
            Event::TrickWon {
                trick: crate::game::CompletedTrick {
                    lead: Seat::DECLARER,
                    cards: ["CJ".parse().expect("card"); 3],
                    winner: Seat::DECLARER,
                },
                card_points: 0,
            },
            Event::DefenderHandsSwapped {
                hands: [Vec::new(), Vec::new(), Vec::new()],
            },
        ];
        assert_eq!(defender_swaps(&log), vec![0, 1]);
    }

    /// Play the first series of section 2 (PIMC defenders) with the bot.
    fn play_series(seed: u64) -> Vec<RunEvent> {
        let content = Content::load().expect("content");
        let start = RunStart {
            section: 1,
            ..RunStart::default()
        };
        let (mut run, mut events) = Run::new_with(&content, seed, &start).expect("run");
        let mut actions = 0;
        while run.phase() == RunPhase::Game && actions < 400 {
            let Some(action) = RunBot.next_action(&run, &content) else {
                break;
            };
            events.extend(run.apply(&content, action).expect("legal action"));
            actions += 1;
        }
        events
    }

    #[test]
    fn pimc_defenders_play_a_series_deterministically() {
        let a = play_series(17);
        let scored = a
            .iter()
            .filter(|e| matches!(e, RunEvent::GameScored { .. }))
            .count();
        assert!(scored >= 1, "a game was played ({scored})");
        assert_eq!(a, play_series(17));
    }
}

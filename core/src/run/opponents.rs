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
use crate::game::{Event, Game, Seat, SeatView};

/// The defenders' policy in `section` (counted from 0; for a short evening
/// the row of the curve, [`Run::curve_row`]).
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
/// Revolutionär", removed by ADR 0017; only logs saved before have one),
/// read from the game's public log.
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

/// PIMC strength of heuristic defenders who know some of the declarer's
/// cards, if `run.ron` names no PIMC section at all.
const INFORMED_FALLBACK: PimcConfig = PimcConfig {
    samples: 8,
    depth: 3,
};

/// The defenders' PIMC configuration in `section` given what they know of
/// the declarer's cards, `None` for the heuristic.
///
/// The heuristic cannot use where a card lies. Whenever the defenders know
/// some of the declarer's cards (an Ouvert hand, "Durchsichtig" cards in
/// his hand or in the skat), they play PIMC instead, with the weakest PIMC
/// strength of `run.ron`, so the knowledge reaches their card play in every
/// section. The search depth stays: searched to the end (depth `0`), the
/// node budget of a decision cuts the deals short, and the defenders of a
/// Null Ouvert catch the declarer less often than with depth 3 (measured
/// in `core/tests/defender_knowledge.rs`).
#[must_use]
pub fn informed_config(
    content: &Content,
    section: u32,
    declarer_open: bool,
    knows_cards: bool,
) -> Option<PimcConfig> {
    opponents_in(content, section).pimc().or_else(|| {
        (declarer_open || knows_cards).then(|| {
            content
                .run
                .opponents
                .iter()
                .find_map(|o| o.pimc())
                .unwrap_or(INFORMED_FALLBACK)
        })
    })
}

/// The defender's card in the run's current game. A defender never holds
/// a Wandelkarte (ADR 0015: it exists only in the player's edition), so it
/// plays the game as it is; the samples resolve every card by the seat it
/// falls to (DESIGN 10.5).
pub(super) fn defender_card(
    run: &Run,
    content: &Content,
    active: &ActiveGame,
    view: &SeatView<'_, DealtCard>,
) -> Option<DealtCard> {
    let hints = defender_hints(active, view);
    let declarer_open = view.declarer_hand.is_some();
    let knows_cards = !hints.pinned.is_empty();
    match informed_config(content, run.curve_row(content), declarer_open, knows_cards) {
        None => DefenderHeuristic.choose(view),
        Some(mut config) => {
            // "Hellwache Gegner" (long night, ADR 0020).
            config.samples = config
                .samples
                .saturating_add(run.night_extra_samples(content));
            PimcPolicy::new(config, game_seed(run))
                .with_thresholds(active.rules.thresholds.clone())
                .choose_with(view, &hints)
        }
    }
}

/// What the defenders know beyond their seat's view: the swaps of their
/// hands and, effects v2 (DESIGN 7.2), every "Durchsichtig" card the
/// player holds, in his hand or in the skat (`DealtCard::revealed`). The
/// cards are taken from the view, so
/// they match the view's copies.
#[must_use]
pub fn defender_hints(active: &ActiveGame, view: &SeatView<'_, DealtCard>) -> Hints<DealtCard> {
    let game = &active.game;
    let as_seen = |card: &DealtCard| view.deck.iter().find(|c| c.id == card.id).copied();
    let mut pinned: Vec<(DealtCard, Option<Seat>)> = game
        .hand(Seat::DECLARER)
        .iter()
        .filter(|c| c.revealed)
        .filter_map(as_seen)
        .map(|c| (c, Some(Seat::DECLARER)))
        .collect();
    pinned.extend(
        game.skat()
            .iter()
            .filter(|c| c.revealed)
            .filter_map(as_seen)
            .map(|c| (c, None)),
    );
    Hints {
        pinned,
        defender_swaps: defender_swaps(game.log()),
    }
}

/// What the player knows beyond his seat's view: "Durchsichtig" cards
/// with their holder, the cards "Der Kiebitz" showed and the card given
/// back with "Drei drücken" (moved along with a swap of the defenders'
/// hands), the skat card "Spickzettel" showed (in no hand) and the swaps
/// themselves.
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
    // The card given back ("Drei drücken") is known like a peeked one.
    let returned = active.game.returned().map(|(seat, card)| (card, seat));
    for &(card, seat) in active.peeked.iter().chain(returned.iter()) {
        // The copy the game holds now: a swap of the defenders' hands
        // changes its edition (ADR 0015).
        let card = active.game.current(card);
        let holder = if swaps.len() % 2 == 1 {
            seat.partner().unwrap_or(seat)
        } else {
            seat
        };
        if !pinned.iter().any(|(c, _)| *c == card) {
            pinned.push((card, Some(holder)));
        }
    }
    // A skat card shown after a hand declaration ("Spickzettel") is in no
    // hand.
    for &card in &active.skat_shown {
        let card = active.game.current(card);
        if !pinned.iter().any(|(c, _)| *c == card) {
            pinned.push((card, None));
        }
    }
    Hints {
        pinned,
        defender_swaps: swaps,
    }
}

/// A PIMC card for the player's seat in the current game, with everything
/// the player is allowed to know ([`player_hints`]). A "Doppelgänger"
/// whose mirror is still to come is kept back while another card is legal
/// ([`crate::run::bot::RunBot::keep_for_mirror`]). `None` if the player
/// is not to play.
#[must_use]
pub fn player_card(run: &Run, content: &Content, config: PimcConfig) -> Option<DealtCard> {
    let active = run.game()?;
    let game = &active.game;
    let mut view = game.view(Seat::DECLARER)?;
    // A Doppelgänger waits for its mirror; the search knows nothing of it.
    crate::run::bot::RunBot::keep_for_mirror(run, content, &mut view.legal_moves);
    if view.legal_moves.is_empty() {
        return None;
    }
    let hints = player_hints(run);
    let chosen = PimcPolicy::new(config, game_seed(run) ^ 0x5EA7_0000)
        .with_thresholds(active.rules.thresholds.clone())
        .choose_with(&view, &hints)?;
    real_card(&active.game, chosen)
}

/// The card of the real game with the id of `chosen`: in the hands, or
/// `chosen` itself.
fn real_card(game: &Game<DealtCard>, chosen: DealtCard) -> Option<DealtCard> {
    Some(
        Seat::ALL
            .iter()
            .flat_map(|&seat| game.hand(seat).iter())
            .copied()
            .find(|c| c.id == chosen.id)
            .unwrap_or(chosen),
    )
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
    fn defenders_who_know_declarer_cards_play_pimc_in_every_section() {
        let content = Content::load().expect("content");
        let weakest = content
            .run
            .opponents
            .iter()
            .find_map(|o| o.pimc())
            .expect("a PIMC section");
        // Section 1: the heuristic, unless the defenders know cards.
        assert_eq!(informed_config(&content, 0, false, false), None);
        assert_eq!(informed_config(&content, 0, false, true), Some(weakest));
        assert_eq!(informed_config(&content, 0, true, false), Some(weakest));
        // Later sections keep their strength.
        let section = opponents_in(&content, 2).pimc().expect("PIMC");
        assert_eq!(informed_config(&content, 2, false, false), Some(section));
        assert_eq!(informed_config(&content, 2, true, true), Some(section));
        // Without any PIMC section in run.ron: the fallback.
        let mut none = content.clone();
        none.run.opponents.clear();
        assert_eq!(informed_config(&none, 0, false, false), None);
        assert_eq!(
            informed_config(&none, 0, false, true),
            Some(INFORMED_FALLBACK)
        );
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

//! A complete random game played by the heuristics: card points add up to
//! 120, ten tricks are played, the events arrive in the right order and the
//! same seed replays the same game.

use skat_core::ai::{DeclarerHeuristic, DefenderHeuristic, Policy};
use skat_core::game::{Event, Game, Phase, Seat};
use skat_core::rng::Rng;
use skat_core::rules::TOTAL_CARD_POINTS;

fn play_seeded(seed: u64) -> Game {
    let mut rng = Rng::from_seed(seed);
    let forehand = Seat::from_index(usize::try_from(seed % 3).expect("small")).expect("seat");
    let mut game = Game::deal(&mut rng, forehand);
    let declarer = DeclarerHeuristic;
    let defender = DefenderHeuristic;
    let declaration = declarer.choose_declaration(game.hand(Seat::DECLARER));
    game.declare(declaration).expect("declare");
    if game.phase() == Phase::Discarding {
        let discard = declarer
            .choose_discard(game.hand(Seat::DECLARER), declaration.game())
            .expect("two discards");
        game.discard(discard).expect("discard");
    }
    while let Some(seat) = game.current_seat() {
        let view = game.view(seat).expect("view while playing");
        let card = if seat.is_declarer() {
            declarer.choose(&view)
        } else {
            defender.choose(&view)
        }
        .expect("legal move");
        game.play(seat, card).expect("legal play");
    }
    game
}

#[test]
fn full_game_accounts_for_all_card_points_and_tricks() {
    for seed in 0..100 {
        let game = play_seeded(seed);
        assert!(matches!(game.phase(), Phase::Finished(_)), "seed {seed}");
        assert_eq!(game.tricks().len(), 10, "seed {seed}");
        for seat in Seat::ALL {
            assert!(game.hand(seat).is_empty(), "seed {seed}");
        }
        let trick_points: u32 = game.tricks().iter().map(|t| t.card_points()).sum();
        let skat_points: u32 = game.skat().iter().map(|c| c.card_points()).sum();
        assert_eq!(trick_points + skat_points, TOTAL_CARD_POINTS, "seed {seed}");
        let outcome = game.outcome();
        assert_eq!(
            outcome.declarer_card_points + outcome.defender_card_points,
            TOTAL_CARD_POINTS,
            "seed {seed}"
        );
        assert_eq!(outcome.declarer_tricks + outcome.defender_tricks, 10);
        // The winner of a trick leads the next one.
        for pair in game.tricks().windows(2) {
            assert_eq!(pair[0].winner, pair[1].lead, "seed {seed}");
        }
        assert_eq!(game.tricks()[0].lead, game.forehand(), "seed {seed}");
    }
}

#[test]
fn events_arrive_in_order() {
    let game = play_seeded(11);
    let log = game.log();
    let hand = game.declaration().expect("declared").hand();
    let mut i = 0;
    assert!(matches!(log[i], Event::Dealt { .. }));
    i += 1;
    assert!(matches!(log[i], Event::Declared { .. }));
    i += 1;
    if !hand {
        assert!(matches!(log[i], Event::SkatPickedUp { .. }));
        i += 1;
        assert!(matches!(log[i], Event::Discarded { .. }));
        i += 1;
    }
    for trick in 0..10 {
        for _ in 0..3 {
            assert!(matches!(log[i], Event::CardPlayed { .. }), "trick {trick}");
            i += 1;
        }
        assert!(matches!(log[i], Event::TrickWon { .. }), "trick {trick}");
        i += 1;
    }
    assert!(matches!(log[i], Event::GameFinished { .. }));
    assert_eq!(log.len(), i + 1);
    let Event::GameFinished { value, .. } = log[i].clone() else {
        panic!("last event must be GameFinished");
    };
    assert_eq!(game.value(), Some(value));
}

#[test]
fn same_seed_replays_the_same_game() {
    for seed in [0, 1, 42, 1234] {
        let a = play_seeded(seed);
        let b = play_seeded(seed);
        assert_eq!(a, b);
        assert_eq!(a.log(), b.log());
    }
    assert_ne!(play_seeded(1).log(), play_seeded(2).log());
}

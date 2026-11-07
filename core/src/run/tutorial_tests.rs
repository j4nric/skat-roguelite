//! Tests of the Einführung (DESIGN 16.1, `tutorial.ron`): a run with a
//! fixed seed whose first pub deals three fixed games, has its own
//! Zielwert and no Einmischen, and plays the lessons as the Wirt explains
//! them.

use super::bot::RunBot;
use super::content_tests::content;
use super::*;
use crate::content::Sources;
use crate::rules::{NullRisk, RiskLevel};

/// The printed faces of the player's hand, sorted.
fn faces(run: &Run, seat: Seat) -> Vec<Card> {
    let mut cards: Vec<Card> = run
        .game()
        .expect("game")
        .game
        .hand(seat)
        .iter()
        .filter_map(|c| c.kind.face())
        .collect();
    cards.sort();
    cards
}

fn sorted(cards: &[Card]) -> Vec<Card> {
    let mut cards = cards.to_vec();
    cards.sort();
    cards
}

fn scored(events: &[RunEvent]) -> Option<ScoreBreakdown> {
    events.iter().find_map(|e| match e {
        RunEvent::GameScored { breakdown, .. } => Some(breakdown.clone()),
        _ => None,
    })
}

/// Pick up the skat and press the cards with these faces.
fn press(run: &mut Run, content: &Content, a: &str, b: &str) -> Vec<RunEvent> {
    let mut events = run.apply(content, RunAction::PickUpSkat).expect("pick up");
    let id = |run: &Run, face: &str| -> u32 {
        let face: Card = face.parse().expect("notation");
        run.game()
            .expect("game")
            .game
            .hand(Seat::DECLARER)
            .iter()
            .find(|c| c.kind.face() == Some(face))
            .expect("card in hand")
            .id
    };
    let ids = [id(run, a), id(run, b)];
    events.extend(run.apply(content, RunAction::Discard(ids)).expect("press"));
    events
}

/// Play the game out: the player's card from `pick` (the bot or the
/// first legal card), until it is scored.
fn play_out(
    run: &mut Run,
    content: &Content,
    mut events: Vec<RunEvent>,
    naive: bool,
) -> ScoreBreakdown {
    loop {
        if let Some(b) = scored(&events) {
            return b;
        }
        let action = if naive {
            let game = &run.game().expect("game").game;
            let card = game.legal_moves(Seat::DECLARER)[0];
            RunAction::Play(card.id)
        } else {
            RunBot.next_action(run, content).expect("game continues")
        };
        events = run.apply(content, action).expect("legal");
    }
}

/// The three lessons as the Wirt suggests them: Eichel after the skat
/// (Schellen 7 and 8 pressed), Null after the skat (Grün- and
/// Schellen-Daus pressed), Grand Hand.
fn play_lessons(naive: bool) -> (Run, Vec<ScoreBreakdown>) {
    let content = content();
    let (mut run, _) = Run::new_tutorial(&content).expect("tutorial");
    let mut scores = Vec::new();

    let mut events = press(&mut run, &content, "D8", "D7");
    events.extend(
        run.apply(
            &content,
            RunAction::Declare(Declaration::Suit {
                suit: Suit::Clubs,
                risk: RiskLevel::Normal,
                hand: false,
            }),
        )
        .expect("declare clubs"),
    );
    scores.push(play_out(&mut run, &content, events, naive));

    let mut events = press(&mut run, &content, "SA", "DA");
    events.extend(
        run.apply(
            &content,
            RunAction::Declare(Declaration::Null {
                risk: NullRisk::Normal,
            }),
        )
        .expect("declare null"),
    );
    scores.push(play_out(&mut run, &content, events, naive));

    let events = run
        .apply(
            &content,
            RunAction::Declare(Declaration::Grand {
                risk: RiskLevel::Hand,
                hand: true,
            }),
        )
        .expect("declare grand hand");
    scores.push(play_out(&mut run, &content, events, naive));
    (run, scores)
}

#[test]
fn tutorial_deals_load_and_cover_the_deck() {
    let content = content();
    let tutorial = &content.tutorial;
    assert_eq!(tutorial.deals.len(), 3, "three games in the tutorial pub");
    assert!(tutorial.target > 0);
    // Content validation rejects a deal with a card twice.
    let text = Sources::embedded().tutorial.replacen("\"CA\"", "\"CJ\"", 1);
    let broken = Content::from_sources(&Sources {
        tutorial: &text,
        ..Sources::embedded()
    });
    assert!(broken.is_err(), "a card twice must be rejected");
}

#[test]
fn tutorial_run_deals_the_fixed_games_with_its_own_target() {
    let content = content();
    let (run, events) = Run::new_tutorial(&content).expect("tutorial");
    let deal = &content.tutorial.deals[0];
    assert!(run.is_tutorial());
    assert_eq!(run.seed, content.tutorial.seed);
    assert_eq!(run.character, Character::Declarer);
    assert_eq!(run.series.target, content.tutorial.target);
    assert_eq!(run.series.game_limit, 3);
    assert_eq!(faces(&run, Seat::DECLARER), sorted(&deal.player));
    assert_eq!(
        faces(&run, Seat::from_index(1).unwrap()),
        sorted(&deal.left)
    );
    assert_eq!(
        faces(&run, Seat::from_index(2).unwrap()),
        sorted(&deal.right)
    );
    assert_eq!(
        run.game().unwrap().game.forehand(),
        Seat::from_index(deal.forehand).unwrap()
    );
    assert!(events.iter().any(|e| matches!(
        e,
        RunEvent::SeriesStarted { target, .. } if *target == content.tutorial.target
    )));
    // No Einmischen: the fixed deal stays.
    assert_eq!(run.redeals_left(&content), 0);
    let mut copy = run.clone();
    assert!(copy.apply(&content, RunAction::Redeal).is_err());
}

#[test]
fn tutorial_is_deterministic() {
    let content = content();
    let (a, _) = Run::new_tutorial(&content).expect("tutorial");
    let (b, _) = Run::new_tutorial(&content).expect("tutorial");
    assert_eq!(a, b);
    let (run_a, scores_a) = play_lessons(false);
    let (run_b, scores_b) = play_lessons(false);
    assert_eq!(run_a, run_b);
    assert_eq!(scores_a, scores_b);
    // Saved and loaded, the run is still the tutorial.
    let json = serde_json::to_string(&a).expect("save");
    let loaded: Run = serde_json::from_str(&json).expect("load");
    assert!(loaded.is_tutorial());
    // A normal run does not carry the flag in its save.
    let (normal, _) = Run::new(&content, 11).expect("run");
    assert!(!serde_json::to_string(&normal).unwrap().contains("tutorial"));
}

#[test]
fn tutorial_lessons_win_the_pub_in_three_games() {
    for naive in [false, true] {
        let (run, scores) = play_lessons(naive);
        let content = content();
        let target = i64::try_from(content.tutorial.target).unwrap();
        assert_eq!(scores.len(), 3);
        for (n, s) in scores.iter().enumerate() {
            assert!(s.won, "game {} lost (naive {naive}): {s:?}", n + 1);
        }
        // Eichel mit 3: 12 × 4 at least; Grand Hand mit 4: 18 × 6 at least.
        assert_eq!(scores[0].base_value, 12);
        assert!(scores[0].levels >= 4);
        assert_eq!(scores[2].base_value, 18);
        assert!(scores[2].levels >= 6);
        // The first two games do not reach the Zielwert, the third does.
        assert!(scores[0].score + scores[1].score < target);
        assert!(scores.iter().map(|s| s.score).sum::<i64>() >= target);
        // The pub is won: the card reward of the Wirt is open.
        assert_eq!(run.phase, RunPhase::CardReward, "naive {naive}");
        // After the tutorial pub, Einmischen is back to normal for the
        // pubs that follow (nothing here deals them yet).
        assert!(run.is_tutorial());
    }
}

#[test]
fn the_third_game_reaches_the_target_even_after_a_lost_first_game() {
    // The Zielwert leaves room for a lesson that went wrong: Eichel lost
    // (one game's value off), Null won, Grand Hand won.
    let content = content();
    let (_, scores) = play_lessons(false);
    let target = i64::try_from(content.tutorial.target).unwrap();
    let worst = -scores[0].score + scores[1].score + scores[2].score;
    assert!(worst >= target, "{worst} < {target}");
}

#[test]
fn tutorial_pub_plays_all_three_games_even_with_the_target_reached() {
    let mut content = content();
    content.tutorial.target = 10;
    let (mut run, _) = Run::new_tutorial(&content).expect("tutorial");
    let events = run
        .apply(
            &content,
            RunAction::Declare(Declaration::Grand {
                risk: RiskLevel::Hand,
                hand: true,
            }),
        )
        .expect("grand hand");
    let first = play_out(&mut run, &content, events, false);
    assert!(first.won && first.score >= 10, "{first:?}");
    // The target is reached, but the next lesson is dealt all the same.
    assert_eq!(run.phase, RunPhase::Game);
    assert_eq!(run.series.games_played, 1);
    assert_eq!(
        faces(&run, Seat::DECLARER),
        sorted(&content.tutorial.deals[1].player)
    );
}

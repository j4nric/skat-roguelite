//! Tests of the stepwise AI: [`Run::apply_step`] carries out only the
//! player's action, [`Run::advance_ai`] one card of a defender. Driven that
//! way a run gives the same events and the same state as with
//! [`Run::apply`], also across saving and loading while the AI is to move.

use super::bot::RunBot;
use super::content_tests::content;
use super::*;
use crate::game::Event;

/// Play a run with the bot through [`Run::apply`] for at most `actions`
/// player actions.
fn whole(content: &Content, seed: u64, start: &RunStart, actions: usize) -> (Run, Vec<RunEvent>) {
    let (mut run, mut events) = Run::new_with(content, seed, start).expect("run");
    for _ in 0..actions {
        let Some(action) = RunBot.next_action(&run, content) else {
            break;
        };
        events.extend(run.apply(content, action).expect("bot action"));
    }
    (run, events)
}

/// The same run, step by step: after every action one AI card at a time,
/// each time saved to RON and loaded again. Returns the run, the events and
/// how many AI cards were played one by one.
fn stepwise(
    content: &Content,
    seed: u64,
    start: &RunStart,
    actions: usize,
) -> (Run, Vec<RunEvent>, usize) {
    let (mut run, mut events) = Run::new_with(content, seed, start).expect("run");
    let mut ai_cards = 0;
    for _ in 0..actions {
        assert_eq!(run.ai_to_move(), None, "the player decides");
        let Some(action) = RunBot.next_action(&run, content) else {
            break;
        };
        events.extend(run.apply_step(content, action).expect("bot action"));
        while let Some(seat) = run.ai_to_move() {
            assert!(!seat.is_declarer());
            let view = run.view(content);
            assert_eq!(view.ai_to_move, Some(seat));
            assert!(view.game.is_some_and(|g| !g.your_turn));
            // Nothing the player does is accepted while the AI is to move.
            let probe = RunBot
                .next_action(&run, content)
                .unwrap_or(RunAction::Redeal);
            assert_eq!(
                run.clone().apply_step(content, probe.clone()),
                Err(RunError::AiToMove)
            );
            assert!(!run.accepts(content, probe));
            // Saved and loaded in the middle of a trick.
            let saved = ron::to_string(&run).expect("save");
            run = ron::from_str(&saved).expect("load");
            let step = run.advance_ai(content).expect("one AI card");
            let played = step
                .iter()
                .filter(|e| matches!(e, RunEvent::Game(Event::CardPlayed { .. })))
                .count();
            assert_eq!(played, 1, "exactly one card: {step:?}");
            assert!(step.iter().any(|e| matches!(
                e,
                RunEvent::Game(Event::CardPlayed { seat: s, .. }) if *s == seat
            )));
            events.extend(step);
            ai_cards += 1;
        }
    }
    (run, events, ai_cards)
}

/// A whole run played with whole actions and step by step ends the same.
/// One test per seed, so they run in parallel.
fn stepwise_play_equals_whole_actions(seed: u64) {
    let content = content();
    let start = RunStart::default();
    let (a, ea) = whole(&content, seed, &start, 3_000);
    let (b, eb, ai_cards) = stepwise(&content, seed, &start, 3_000);
    assert!(ai_cards > 20, "seed {seed}: {ai_cards} AI cards");
    assert_eq!(ea, eb, "seed {seed}: same events");
    assert_eq!(a, b, "seed {seed}: same run");
    assert!(matches!(a.phase(), RunPhase::Won | RunPhase::Lost));
}

#[test]
fn stepwise_play_equals_whole_actions_seed_3() {
    stepwise_play_equals_whole_actions(3);
}

#[test]
fn stepwise_play_equals_whole_actions_seed_7() {
    stepwise_play_equals_whole_actions(7);
}

#[test]
fn stepwise_play_equals_whole_actions_seed_21() {
    stepwise_play_equals_whole_actions(21);
}

#[test]
fn stepwise_play_equals_whole_actions_with_pimc_defenders() {
    let content = content();
    let start = RunStart {
        section: 1,
        ..RunStart::default()
    };
    let (a, ea) = whole(&content, 17, &start, 40);
    let (b, eb, ai_cards) = stepwise(&content, 17, &start, 40);
    assert!(ai_cards > 10, "{ai_cards} AI cards");
    assert_eq!(ea, eb);
    assert_eq!(a, b);
}

#[test]
fn advance_ai_without_a_defender_to_move_changes_nothing() {
    let content = content();
    let (mut run, _) = Run::new(&content, 3).expect("run");
    assert_eq!(run.ai_to_move(), None);
    let before = run.clone();
    assert_eq!(run.advance_ai(&content), Err(RunError::WrongPhase));
    assert_eq!(run.advance_ai_all(&content), Ok(Vec::new()));
    assert_eq!(run, before);
    assert_eq!(RunError::AiToMove.code().as_str(), "AiToMove");
}

#[test]
fn a_ramsch_led_by_a_defender_waits_for_the_ai() {
    let content = content();
    let mut found = false;
    for games in 0..3 {
        let (mut run, _) = Run::new(&content, 11).expect("run");
        run.series.boss = Some(BossId::new("ramscher"));
        run.games_dealt = games;
        let mut events = Vec::new();
        run.deal(&content, true, &mut events).expect("deal");
        assert!(events.contains(&RunEvent::RamschDealt));
        let forehand = run.game().expect("game").game.forehand();
        if forehand.is_declarer() {
            assert_eq!(run.ai_to_move(), None);
            continue;
        }
        found = true;
        assert_eq!(run.ai_to_move(), Some(forehand), "the defender leads");
        let mut whole = run.clone();
        let rest = whole.advance_ai_all(&content).expect("AI");
        let mut steps = Vec::new();
        while run.ai_to_move().is_some() {
            steps.extend(run.advance_ai(&content).expect("AI card"));
        }
        assert_eq!(rest, steps);
        assert_eq!(run, whole);
        assert!(run.view(&content).game.is_some_and(|g| g.your_turn));
    }
    assert!(found, "some deal lets a defender lead");
}

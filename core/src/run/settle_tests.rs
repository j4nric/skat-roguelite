//! Tests of "Rest abkürzen" ([`RunAction::SettleGame`]): settling a
//! decided game scores it exactly as playing it out does, whoever plays
//! which card, and an open game cannot be settled.

use super::bot::RunBot;
use super::content_tests::{content, declare, fixed_deal, relic_run};
use super::*;
use crate::game::TRICKS_PER_GAME;
use crate::rules::{NullRisk, RiskLevel};

/// A Null the declarer cannot lose once a defender leads (all eight
/// sevens of the Nullspieler's deck).
const SAFE_NULL: [&str; 3] = [
    "D7 D7 H7 H7 S7 S7 C7 C7 D8 H8",
    "D9 DT DQ DK DA H9 HT HQ HK HA",
    "S8 S9 ST SQ SK SA C8 C9 CT CQ",
];
const SAFE_NULL_SKAT: &str = "CK CA";

/// Declarer holds all trumps of a diamonds game but the D8 (in the skat)
/// and takes every trick.
const STRONG: [&str; 3] = [
    "CJ SJ HJ DJ DA DT DK DQ D9 D7",
    "HA HT HK HQ H9 H8 H7 SA ST SK",
    "SQ S9 S8 S7 CA CT CK CQ C9 C8",
];
const STRONG_SKAT: &str = "D8 C7";

fn null_run(content: &Content, relics: &[&str]) -> Run {
    let start = RunStart {
        character: Character::NullPlayer,
        ..RunStart::default()
    };
    let (mut run, _) = Run::new_with(content, 11, &start).expect("null run");
    run.relics = relics.iter().map(|r| RelicId::new(r)).collect();
    run
}

fn scored(events: &[RunEvent]) -> Option<ScoreBreakdown> {
    events.iter().find_map(|e| match e {
        RunEvent::GameScored { breakdown, .. } => Some(breakdown.clone()),
        _ => None,
    })
}

/// Everything about the run that the rest of a game may influence,
/// except the Augen and tricks themselves.
#[derive(Debug, PartialEq, Eq)]
struct After {
    score: i64,
    won: bool,
    schneider: bool,
    schwarz: bool,
    value: u64,
    base_value: u64,
    levels: u64,
    multiplier: crate::content::Factor,
    series_score: i64,
    coins: u32,
    flags: BTreeSet<String>,
    counters: BTreeMap<String, u32>,
    stats: RunStats,
    phase: RunPhase,
    next_hand: Option<Vec<DealtCard>>,
}

fn after(run: &Run, breakdown: &ScoreBreakdown) -> After {
    After {
        score: breakdown.score,
        won: breakdown.won,
        schneider: breakdown.schneider,
        schwarz: breakdown.schwarz,
        value: breakdown.value,
        base_value: breakdown.base_value,
        levels: breakdown.levels,
        multiplier: breakdown.multiplier,
        series_score: run.series.score,
        coins: run.coins,
        flags: run.flags.clone(),
        counters: run.counters.clone(),
        stats: run.stats,
        phase: run.phase,
        next_hand: run
            .game
            .as_ref()
            .map(|a| a.game.hand(Seat::DECLARER).to_vec()),
    }
}

/// Play the rest of the game with random legal cards for every seat,
/// through the run's own card play (hooks and all).
fn play_out_randomly(run: &Run, content: &Content, rng: &mut Rng) -> (Run, ScoreBreakdown) {
    let mut run = run.clone();
    let mut events = Vec::new();
    loop {
        if let Some(b) = scored(&events) {
            return (run, b);
        }
        let active = run.game.as_ref().expect("game in progress");
        let seat = active.game.current_seat().expect("someone to play");
        let legal = active.game.legal_moves(seat);
        let card = legal[usize::try_from(rng.gen_range(0..legal.len() as u64)).unwrap_or(0)];
        run.play_card(content, seat, card, &mut events)
            .expect("legal card");
        run.settle(content, &mut events).expect("settle");
    }
}

/// Settle `run` and compare with `lines` random play-outs. Returns the
/// settled breakdown.
fn settle_equals_play(run: &Run, content: &Content, rng: &mut Rng, lines: usize) -> ScoreBreakdown {
    let mut settled = run.clone();
    let events = settled
        .apply(content, RunAction::SettleGame)
        .expect("settle a decided game");
    let breakdown = scored(&events).expect("scored");
    let want = after(&settled, &breakdown);
    // The remaining cards are played as normal events.
    let left: usize = Seat::ALL
        .iter()
        .map(|&s| run.game().map_or(0, |a| a.game.hand(s).len()))
        .sum();
    let played = events
        .iter()
        .filter(|e| matches!(e, RunEvent::Game(Event::CardPlayed { .. })))
        .count();
    if breakdown.won || !breakdown.declaration.is_null() {
        assert_eq!(played, left, "every card is played");
    }
    for _ in 0..lines {
        let (other, b) = play_out_randomly(run, content, rng);
        assert_eq!(
            after(&other, &b),
            want,
            "settled {breakdown:?}, played {b:?}"
        );
    }
    breakdown
}

#[test]
fn a_safe_null_settles_as_won_once_a_defender_leads() {
    let content = content();
    let mut run = null_run(&content, &["seven_sleeper", "clean_slate"]);
    fixed_deal(&mut run, &content, SAFE_NULL, SAFE_NULL_SKAT);
    run.apply(
        &content,
        RunAction::Declare(Declaration::null(NullRisk::Hand)),
    )
    .expect("declare");
    // The declarer leads the first trick: he could still be on top.
    assert!(!run.can_settle(&content));
    let view = run.view(&content).game.expect("game");
    assert!(view.your_turn && !view.can_settle);
    assert_eq!(
        run.clone().apply(&content, RunAction::SettleGame),
        Err(RunError::NotDecided)
    );
    // After that he can no longer take a trick, but the Kurzschluss (the
    // defenders' Augen after five tricks) stays open until the fifth trick.
    loop {
        let action = RunBot.next_action(&run, &content).expect("card");
        run.apply(&content, action).expect("card");
        let tricks = run.game().expect("game").game.tricks().len();
        if run.can_settle(&content) {
            assert!(tricks >= 1, "the declarer led the first trick");
            break;
        }
        assert!(tricks < 5, "decided once the Kurzschluss is");
    }
    assert!(run.view(&content).game.expect("game").can_settle);
    assert!(run.accepts(&content, RunAction::SettleGame));
    let mut rng = Rng::from_seed(3);
    let breakdown = settle_equals_play(&run, &content, &mut rng, 12);
    assert!(breakdown.won);
    assert_eq!(breakdown.outcome.declarer_tricks, 0);
}

#[test]
fn a_schwarz_game_settles_in_the_last_tricks_unless_the_last_trick_counts() {
    let content = content();
    let mut rng = Rng::from_seed(9);
    for relics in [&[][..], &["last_trick_seven"][..]] {
        let mut run = relic_run(&content, relics);
        fixed_deal(&mut run, &content, STRONG, STRONG_SKAT);
        declare(
            &mut run,
            &content,
            Declaration::suit(Suit::Diamonds, RiskLevel::Hand),
        )
        .expect("declare");
        let mut settled_at = None;
        loop {
            let game = run.game().expect("game");
            let left = TRICKS_PER_GAME - game.game.tricks().len();
            if run.can_settle(&content) {
                settled_at = Some(left);
                let b = settle_equals_play(&run, &content, &mut rng, 8);
                assert!(b.won && b.schwarz);
                break;
            }
            let action = RunBot.next_action(&run, &content).expect("card");
            let events = run.apply(&content, action).expect("card");
            if scored(&events).is_some() {
                break;
            }
        }
        if relics.is_empty() {
            // He takes every trick however anyone plays: Schwarz is
            // certain as soon as the search runs.
            assert_eq!(settled_at, Some(settle::SEARCH_TRICKS));
        } else {
            // Which trump takes the last trick is open until he holds
            // only the seven, or the seven is gone.
            assert!(settled_at.is_none_or(|left| left < settle::SEARCH_TRICKS));
        }
    }
}

#[test]
fn settling_scores_like_playing_out_over_whole_runs() {
    let content = content();
    let mut rng = Rng::from_seed(17);
    let mut checked = [0usize; 3];
    let characters = [
        Character::Declarer,
        Character::NullPlayer,
        Character::GrandPlayer,
        Character::Onlooker,
    ];
    for (k, &character) in characters.iter().enumerate() {
        for seed in 0..3u64 {
            let start = RunStart {
                character,
                ..RunStart::default()
            };
            let (mut run, _) = Run::new_with(&content, 100 * k as u64 + seed, &start).expect("run");
            for _ in 0..1_500 {
                if run.can_settle(&content) {
                    let b = settle_equals_play(&run, &content, &mut rng, 3);
                    let kind = match (b.declaration.is_null(), b.won) {
                        (true, true) => 0,
                        (true, false) => 1,
                        _ => 2,
                    };
                    checked[kind] += 1;
                } else if let Some(view) = run.view(&content).game {
                    assert!(!view.can_settle);
                    if view.your_turn {
                        assert!(!run.accepts(&content, RunAction::SettleGame));
                    }
                }
                let Some(action) = RunBot.next_action(&run, &content) else {
                    break;
                };
                run.apply(&content, action).expect("bot action");
            }
        }
    }
    assert!(checked[0] > 5, "won Nulls settled: {checked:?}");
    assert!(
        checked[2] > 20,
        "suit games and grands settled: {checked:?}"
    );
}

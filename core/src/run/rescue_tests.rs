//! Rescue of a pub gone wrong (user decision of 03.11.2026): "Kneipe
//! verlassen", the "noch nötig" line before the last game, and the
//! Schnäpse Klarer, Absacker and Eierlikör.

use super::bot::RunBot;
use super::content_tests::{content, fixed_deal, play};
use super::*;
use crate::cards::Suit;
use crate::content::ScoreLine;
use crate::content::Source;
use crate::content::{ConsumableId, RelicId};
use crate::rules::Declaration;
use crate::rules::{GameType, RiskLevel};

/// Declarer holds all trumps of a diamonds game but the D8 (in the skat).
const STRONG: [&str; 3] = [
    "CJ SJ HJ DJ DA DT DK DQ D9 D7",
    "HA HT HK HQ H9 H8 H7 SA ST SK",
    "SQ S9 S8 S7 CA CT CK CQ C9 C8",
];
const STRONG_SKAT: &str = "D8 C7";

/// Declarer holds only Luschen: he loses any game.
const WEAK: [&str; 3] = [
    "D7 D8 D9 H7 H8 H9 S7 S8 S9 C7",
    "CJ SJ HJ DJ CA CT CK CQ DA DT",
    "DK DQ HA HT HK HQ SA ST SK SQ",
];
const WEAK_SKAT: &str = "C8 C9";

const DIAMONDS: GameType = GameType::Suit(Suit::Diamonds);

fn hand(game: GameType) -> Declaration {
    Declaration::new(game, RiskLevel::Hand).expect("suit or grand")
}

fn start_at(content: &Content, section: u32, index: u32) -> Run {
    let start = RunStart {
        section,
        index,
        ..RunStart::default()
    };
    Run::new_with(content, 5, &start).expect("start").0
}

/// A run in its first pub with these consumables.
fn run_with(content: &Content, items: &[&str]) -> Run {
    let (mut run, _) = Run::new(content, 11).expect("start");
    run.consumables = items.iter().map(|c| ConsumableId::new(c)).collect();
    run
}

fn use_item(slot: usize) -> RunAction {
    RunAction::UseConsumable {
        slot,
        target: None,
        suit: None,
    }
}

fn source(id: &str) -> Source {
    Source::Consumable(ConsumableId::new(id))
}

fn is_line_of(line: &ScoreLine, id: &str) -> bool {
    line.source == source(id)
}

// ---------------------------------------------------------------------------
// Kneipe verlassen
// ---------------------------------------------------------------------------

#[test]
fn giving_up_goes_only_between_games_from_the_second_game_on() {
    let content = content();
    let mut run = start_at(&content, 0, 1);
    assert!(!run.can_give_up(), "not before the first game");
    assert!(!run.view(&content).can_give_up);
    assert_eq!(
        run.apply(&content, RunAction::GiveUpPub),
        Err(RunError::WrongPhase)
    );
    run.series.games_played = 1;
    assert!(run.view(&content).can_give_up);
    // Not once the game is under way.
    run.apply(&content, RunAction::PickUpSkat).expect("pick up");
    assert!(!run.view(&content).can_give_up);
    assert_eq!(
        run.apply(&content, RunAction::GiveUpPub),
        Err(RunError::WrongPhase)
    );
}

#[test]
fn giving_up_a_pub_loses_it_without_reward() {
    let content = content();
    let mut run = start_at(&content, 0, 1);
    run.series.games_played = 1;
    run.series.score = -40;
    let coins = run.coins();
    let events = run.apply(&content, RunAction::GiveUpPub).expect("give up");
    assert_eq!(
        events[0],
        RunEvent::PubGivenUp {
            section: 0,
            index: 1,
            score: -40
        }
    );
    assert!(events.contains(&RunEvent::PubLost {
        section: 0,
        index: 1,
        boss: None
    }));
    assert!(
        !events
            .iter()
            .any(|e| matches!(e, RunEvent::SeriesWon { .. }))
    );
    assert_eq!(run.coins(), coins, "no payout");
    // No lives since ADR 0019: the run is over.
    assert_eq!(run.phase(), RunPhase::Lost);
    assert!(
        events
            .iter()
            .any(|e| matches!(e, RunEvent::RunLost { boss: None, .. }))
    );
    assert_eq!(run.stats().pubs_lost, 1);
    assert!(!run.pub_results()[0].won);
}

#[test]
fn giving_up_a_boss_pub_ends_the_run() {
    let content = content();
    let mut run = start_at(&content, 0, PUBS_PER_DISTRICT - 1);
    let boss = run.series().boss.clone().expect("boss pub");
    run.series.games_played = 1;
    assert!(run.view(&content).can_give_up);
    let events = run.apply(&content, RunAction::GiveUpPub).expect("give up");
    assert_eq!(run.phase(), RunPhase::Lost);
    assert!(events.iter().any(|e| matches!(
        e,
        RunEvent::RunLost { boss: Some(b), .. } if *b == boss
    )));
}

#[test]
fn guardian_angel_plays_a_pub_given_up_again() {
    let content = content();
    let mut run = start_at(&content, 0, 1);
    run.relics.push(RelicId::new("guardian_angel"));
    run.series.games_played = 1;
    let events = run.apply(&content, RunAction::GiveUpPub).expect("give up");
    assert!(events.contains(&RunEvent::SecondChance {
        relic: RelicId::new("guardian_angel"),
        section: 0,
        index: 1,
    }));
    assert_eq!(run.phase(), RunPhase::Game);
    assert_eq!(run.series().index, 1);
    assert_eq!(run.series().games_played, 0);
    assert!(!run.relics().contains(&RelicId::new("guardian_angel")));
}

#[test]
fn a_pub_given_up_on_the_way_to_a_spelunke_counts_like_a_normal_pub() {
    let content = content();
    let mut run = start_at(&content, 0, 1);
    run.series.spelunke = Some(tour::Spelunke {
        name: "Zur Ratte".to_owned(),
        malus: content.maluses.keys().next().expect("malus").clone(),
        malus_options: Vec::new(),
    });
    run.series.games_played = 1;
    let events = run.apply(&content, RunAction::GiveUpPub).expect("give up");
    assert_eq!(run.phase(), RunPhase::Lost);
    assert!(
        events
            .iter()
            .any(|e| matches!(e, RunEvent::RunLost { boss: None, .. }))
    );
}

#[test]
fn the_bot_never_gives_up() {
    let content = content();
    let mut run = start_at(&content, 0, 1);
    run.series.games_played = 2;
    run.series.score = -500;
    assert!(run.can_give_up());
    assert_ne!(
        RunBot.next_action(&run, &content),
        Some(RunAction::GiveUpPub)
    );
}

// ---------------------------------------------------------------------------
// Noch nötig
// ---------------------------------------------------------------------------

#[test]
fn needed_shows_only_before_the_last_game() {
    let content = content();
    let mut run = start_at(&content, 0, 1);
    assert!(run.view(&content).needed.is_none(), "three games left");
    run.series.games_played = 2;
    run.series.target = 100;
    run.series.score = 13;
    let needed = run.view(&content).needed.expect("last game");
    assert_eq!(needed.points, 87);
    // Pik with 8 levels, 11 × 8 = 88, is the smallest plain
    // Spielwert of at least 87.
    assert_eq!(needed.smallest_game, Some(88));
    assert!(needed.reachable);
    // Declared, the value preview takes over.
    run.apply(&content, RunAction::Declare(hand(DIAMONDS)))
        .expect("declare");
    assert!(run.view(&content).needed.is_none());
}

#[test]
fn the_bound_covers_the_highest_games_of_the_formula() {
    let content = content();
    let mut run = start_at(&content, 0, 1);
    run.relics.clear();
    run.evening_relics.clear();
    let max = run.max_game_value(&content);
    // Grand Ouvert with 4 Spitzen: 24 × (4 + 7) = 264; the Null of the
    // character's formula with every style bonus may be higher.
    assert!(max >= 264, "{max}");
    run.series.games_played = 2;
    run.series.score =
        i64::try_from(run.series.target).expect("small") - i64::try_from(max + 1).expect("small");
    let needed = run.view(&content).needed.expect("last game");
    assert_eq!(needed.max_value, max);
    assert!(!needed.reachable, "nicht mehr zu schaffen");
    // A Doppelkorn waiting doubles it.
    run.pending_next_game.push((
        ConsumableId::new("double_grain"),
        NextGameEffect::Multiply(2),
    ));
    assert_eq!(run.max_game_value(&content), 2 * max);
    assert!(run.view(&content).needed.expect("last").reachable);
}

#[test]
fn relic_bonuses_raise_the_bound() {
    let content = content();
    let mut run = start_at(&content, 0, 1);
    run.relics.clear();
    run.evening_relics.clear();
    let plain = run.max_game_value(&content);
    let raising = content
        .relics
        .values()
        .find(|r| {
            r.hooks.iter().any(|h| {
                h.then().iter().any(|e| {
                    matches!(
                        e,
                        crate::content::Effect::AddLevels(n) if *n > 0
                    )
                })
            })
        })
        .expect("a relic adding levels");
    run.relics.push(raising.id.clone());
    assert!(run.max_game_value(&content) > plain, "{}", raising.id);
}

// ---------------------------------------------------------------------------
// Klarer
// ---------------------------------------------------------------------------

#[test]
fn clear_spirit_sets_a_minus_to_zero_between_games() {
    let content = content();
    let mut run = run_with(&content, &["clear_spirit"]);
    assert_eq!(
        run.apply(&content, use_item(0)),
        Err(RunError::ConditionNotMet),
        "no minus"
    );
    assert!(!run.view(&content).consumables[0].usable_now);
    run.series.score = -72;
    assert!(run.view(&content).consumables[0].usable_now);
    let events = run.apply(&content, use_item(0)).expect("drink");
    assert!(events.contains(&RunEvent::AccountCleared {
        before: -72,
        score: 0
    }));
    assert!(events.contains(&RunEvent::ConsumableUsed {
        consumable: ConsumableId::new("clear_spirit")
    }));
    assert_eq!(run.series().score, 0);
    assert!(run.consumables().is_empty());
}

#[test]
fn clear_spirit_does_not_go_mid_game_or_at_the_tresen() {
    let content = content();
    let mut run = run_with(&content, &["clear_spirit"]);
    run.series.score = -10;
    run.apply(&content, RunAction::PickUpSkat).expect("pick up");
    assert_eq!(run.apply(&content, use_item(0)), Err(RunError::WrongPhase));
    let (mut run, _) = Run::new(&content, 3).expect("run");
    run.phase = RunPhase::Shop;
    run.game = None;
    run.consumables = vec![ConsumableId::new("clear_spirit")];
    run.series.score = -10;
    assert_eq!(run.apply(&content, use_item(0)), Err(RunError::WrongPhase));
}

// ---------------------------------------------------------------------------
// Absacker
// ---------------------------------------------------------------------------

#[test]
fn nightcap_adds_a_game_to_a_normal_pub() {
    let content = content();
    let mut run = run_with(&content, &["nightcap"]);
    run.series.games_played = 2;
    let limit = run.series().game_limit;
    let events = run.apply(&content, use_item(0)).expect("drink");
    assert!(events.contains(&RunEvent::GamesAdded {
        games: 1,
        game_limit: limit + 1,
        games_left: 2
    }));
    assert_eq!(run.series().games_left(), 2);
    assert!(
        run.view(&content).needed.is_none(),
        "no longer the last game"
    );
}

#[test]
fn nightcap_is_refused_in_a_boss_pub() {
    let content = content();
    let mut run = start_at(&content, 0, PUBS_PER_DISTRICT - 1);
    run.consumables = vec![ConsumableId::new("nightcap")];
    let err = run.apply(&content, use_item(0)).expect_err("boss pub");
    assert_eq!(err, RunError::NotInBossPub);
    assert_eq!(err.code(), RunErrorCode::NotInBossPub);
    assert!(!run.view(&content).consumables[0].usable_now);
}

// ---------------------------------------------------------------------------
// Eierlikör
// ---------------------------------------------------------------------------

#[test]
fn eggnog_needs_the_account_below_half_the_target() {
    let content = content();
    let mut run = run_with(&content, &["eggnog"]);
    run.series.target = 100;
    run.series.score = 50;
    assert_eq!(
        run.apply(&content, use_item(0)),
        Err(RunError::ConditionNotMet)
    );
    run.series.score = 49;
    run.apply(&content, use_item(0)).expect("behind");
    assert_eq!(
        run.pending_next_game(),
        &[(ConsumableId::new("eggnog"), NextGameEffect::MultiplyWin(2))]
    );
}

#[test]
fn eggnog_doubles_a_won_game() {
    let content = content();
    let mut plain = run_with(&content, &[]);
    fixed_deal(&mut plain, &content, STRONG, STRONG_SKAT);
    let (_, without) = play(&mut plain, &content, hand(DIAMONDS));

    let mut run = run_with(&content, &["eggnog"]);
    run.series.target = 1000;
    fixed_deal(&mut run, &content, STRONG, STRONG_SKAT);
    run.apply(&content, use_item(0)).expect("drink");
    let (_, with) = play(&mut run, &content, hand(DIAMONDS));
    assert!(with.won);
    assert_eq!(with.value, 2 * without.value);
    assert!(with.lines.iter().any(|l| is_line_of(l, "eggnog")));
    assert!(run.pending_next_game().is_empty(), "used up");
}

#[test]
fn eggnog_does_nothing_to_a_lost_game_but_is_used_up() {
    let content = content();
    let mut plain = run_with(&content, &[]);
    fixed_deal(&mut plain, &content, WEAK, WEAK_SKAT);
    let (_, without) = play(&mut plain, &content, hand(DIAMONDS));

    let mut run = run_with(&content, &["eggnog"]);
    run.series.target = 1000;
    fixed_deal(&mut run, &content, WEAK, WEAK_SKAT);
    run.apply(&content, use_item(0)).expect("drink");
    let (_, with) = play(&mut run, &content, hand(DIAMONDS));
    assert!(!with.won);
    assert_eq!(with.score, without.score);
    assert!(!with.lines.iter().any(|l| is_line_of(l, "eggnog")));
    assert!(run.pending_next_game().is_empty(), "used up");
}

#[test]
fn eggnog_is_counted_by_the_value_preview() {
    let content = content();
    let mut run = run_with(&content, &["eggnog"]);
    run.series.target = 1000;
    fixed_deal(&mut run, &content, STRONG, STRONG_SKAT);
    let mut plain = run.clone();
    plain.consumables.clear();
    run.apply(&content, use_item(0)).expect("drink");
    run.apply(&content, RunAction::Declare(hand(DIAMONDS)))
        .expect("declare");
    plain
        .apply(&content, RunAction::Declare(hand(DIAMONDS)))
        .expect("declare");
    let preview = |r: &Run| {
        r.view(&content)
            .game
            .expect("game")
            .value_preview
            .expect("preview")
            .game_value
    };
    assert_eq!(preview(&run), 2 * preview(&plain));
}

// ---------------------------------------------------------------------------
// Bot
// ---------------------------------------------------------------------------

#[test]
fn the_bot_drinks_the_rescue_schnaepse_when_they_help() {
    let content = content();
    let mut run = run_with(&content, &["clear_spirit", "nightcap"]);
    assert_ne!(
        RunBot.next_action(&run, &content),
        Some(use_item(0)),
        "no minus, no Klarer"
    );
    run.series.score = -30;
    assert_eq!(RunBot.next_action(&run, &content), Some(use_item(0)));
    run.apply(&content, use_item(0)).expect("klarer");
    // Absacker only before the last game.
    assert_ne!(RunBot.next_action(&run, &content), Some(use_item(0)));
    run.series.games_played = 2;
    assert_eq!(RunBot.next_action(&run, &content), Some(use_item(0)));
    let mut run = run_with(&content, &["eggnog"]);
    run.series.target = 100;
    assert_eq!(RunBot.next_action(&run, &content), Some(use_item(0)));
}

//! Tests of the Einsatz-Leiter (stake ladder, DESIGN 9.3): every won run
//! raises the win threshold of the next by one Auge.

use super::bot::RunBot;
use super::content_tests::{content, fixed_deal, play, relic_run};
use super::meta::{MetaError, MetaState};
use super::player::player_events;
use super::*;
use crate::cards::Suit;
use crate::rules::{GameType, NullRisk, RiskLevel, WIN_THRESHOLD};

/// Declarer holds all trumps of a diamonds game but the D8 (in the skat):
/// he takes every trick. Instance ids follow the standard deck (DA = 7).
const STRONG: [&str; 3] = [
    "CJ SJ HJ DJ DA DT DK DQ D9 D7",
    "HA HT HK HQ H9 H8 H7 SA ST SK",
    "SQ S9 S8 S7 CA CT CK CQ C9 C8",
];
const STRONG_SKAT: &str = "D8 C7";

fn staked(stake: u32) -> RunStart {
    RunStart {
        stake,
        ..RunStart::default()
    }
}

fn diamonds_hand() -> Declaration {
    Declaration::new(GameType::Suit(Suit::Diamonds), RiskLevel::Hand).expect("suit")
}

#[test]
fn a_staked_run_needs_61_plus_stake_augen() {
    let content = content();
    for stake in [0, 1, 2, 7] {
        let (run, _) = Run::new_with(&content, 3, &staked(stake)).expect("run");
        assert_eq!(run.stake(), stake);
        let win = WIN_THRESHOLD + stake;
        let rules = &run.game().expect("game").rules;
        assert_eq!(rules.thresholds.win, win);
        // Schneider does not move with the stake.
        assert_eq!(rules.thresholds.schneider_limit, 30);
        let view = run.view(&content);
        assert_eq!(view.stake, stake);
        assert_eq!(view.win_threshold, win);
        assert_eq!(view.game.expect("game").win_threshold, win);
    }
    // The stake does not change the cards.
    let (plain, _) = Run::new_with(&content, 3, &staked(0)).expect("run");
    let (high, _) = Run::new_with(&content, 3, &staked(5)).expect("run");
    assert_eq!(
        plain.game().expect("game").game.hand(Seat::DECLARER),
        high.game().expect("game").game.hand(Seat::DECLARER)
    );
}

#[test]
fn relics_lower_the_staked_threshold_by_their_own_amount() {
    let content = content();
    // "Ein Auge zudrücken" (58, three below 61) at stake 2: 60.
    let mut run = relic_run(&content, &["blind_eye"]);
    run.stake = 2;
    let mut events = Vec::new();
    run.deal(&content, true, &mut events).expect("deal");
    let rules = &run.game().expect("game").rules;
    assert_eq!(rules.thresholds.win, 60);
    assert_eq!(
        rules.win_threshold_source,
        Some(Source::Relic(RelicId::new("blind_eye")))
    );
    // Hufeisen (56, five below) on the trump ace at stake 3: 59, and it
    // beats the house rule (61) because it is lower.
    let mut run = relic_run(&content, &["blind_eye"]);
    run.stake = 3;
    run.deck
        .add_mod(7, CardModId::new("horseshoe"))
        .expect("mod");
    fixed_deal(&mut run, &content, STRONG, STRONG_SKAT);
    assert_eq!(run.game().expect("game").rules.thresholds.win, 61);
    run.apply(&content, RunAction::Declare(diamonds_hand()))
        .expect("declare");
    assert_eq!(run.game().expect("game").rules.thresholds.win, 59);
}

#[test]
fn a_scored_game_carries_the_staked_thresholds() {
    let content = content();
    let mut run = relic_run(&content, &[]);
    run.stake = 4;
    fixed_deal(&mut run, &content, STRONG, STRONG_SKAT);
    let (_, breakdown) = play(&mut run, &content, diamonds_hand());
    assert!(breakdown.won);
    assert_eq!(breakdown.thresholds.win, 65);
    assert_eq!(breakdown.thresholds.stake, 4);
    // Far up the ladder all 120 Augen are not enough for a suit game.
    let mut run = relic_run(&content, &[]);
    run.stake = 60;
    fixed_deal(&mut run, &content, STRONG, STRONG_SKAT);
    let (_, breakdown) = play(&mut run, &content, diamonds_hand());
    assert_eq!(breakdown.outcome.declarer_card_points, 120);
    assert!(!breakdown.won);
}

#[test]
fn null_ignores_the_stake() {
    let staked = crate::rules::Thresholds::with_stake(30);
    let plain = crate::rules::Thresholds::default();
    let decl = Declaration::null(NullRisk::Normal);
    let matadors = crate::rules::Matadors {
        with: false,
        count: 0,
    };
    for declarer_tricks in [0, 1] {
        let outcome = crate::rules::Outcome {
            declarer_card_points: 0,
            defender_card_points: 120,
            declarer_tricks,
            defender_tricks: 10 - declarer_tricks,
        };
        assert_eq!(
            crate::rules::game_value_parts(decl, matadors, &outcome, &staked).value,
            crate::rules::game_value_parts(decl, matadors, &outcome, &plain).value
        );
    }
}

#[test]
fn run_won_and_run_lost_carry_the_stake() {
    let content = content();
    let mut seen = 0;
    for seed in 0..6 {
        let (mut run, _) = Run::new_with(&content, seed, &staked(2)).expect("run");
        while let Some(action) = RunBot.next_action(&run, &content) {
            for event in run.apply(&content, action).expect("bot") {
                if let RunEvent::RunLost { stake, .. } | RunEvent::RunWon { stake, .. } = event {
                    assert_eq!(stake, 2);
                    seen += 1;
                }
            }
        }
    }
    assert_eq!(seen, 6);
}

#[test]
fn old_saves_play_at_stake_zero() {
    let content = content();
    let (run, _) = Run::new_with(&content, 4, &staked(2)).expect("run");
    let text = ron::to_string(&run).expect("ron");
    assert!(text.contains(",stake:2"));
    let old = text.replace(",stake:2", "");
    let back: Run = ron::from_str(&old).expect("old save");
    assert_eq!(back.stake(), 0);
    let event: RunEvent = ron::from_str("RunWon(character:Declarer,length:Short)").expect("old");
    assert_eq!(
        event,
        RunEvent::RunWon {
            character: Character::Declarer,
            length: RunLength::Short,
            stake: 0,
        }
    );
}

#[test]
fn a_won_run_unlocks_the_next_stake_per_character_and_length() {
    let content = content();
    let mut meta = MetaState::all_unlocked(&content);
    assert_eq!(meta.max_stake(Character::Declarer, RunLength::Full), 0);
    let won = |character, length, stake| RunEvent::RunWon {
        character,
        length,
        stake,
    };
    meta.observe(&content, &[won(Character::Declarer, RunLength::Short, 0)]);
    assert_eq!(meta.max_stake(Character::Declarer, RunLength::Short), 1);
    assert_eq!(meta.max_stake(Character::Declarer, RunLength::Full), 0);
    assert_eq!(meta.max_stake(Character::NullPlayer, RunLength::Short), 0);
    meta.observe(&content, &[won(Character::Declarer, RunLength::Short, 3)]);
    assert_eq!(meta.max_stake(Character::Declarer, RunLength::Short), 4);
    // A win further down the ladder, or a lost run, takes nothing back.
    meta.observe(
        &content,
        &[
            won(Character::Declarer, RunLength::Short, 1),
            RunEvent::RunLost {
                section: 0,
                index: 0,
                character: Character::Declarer,
                length: RunLength::Short,
                stake: 4,
            },
        ],
    );
    assert_eq!(meta.max_stake(Character::Declarer, RunLength::Short), 4);
    // Player events count the same.
    let mut twin = MetaState::all_unlocked(&content);
    let events: Vec<RunEvent> = vec![won(Character::NullPlayer, RunLength::Full, 0)];
    twin.observe(&content, &player_events(events));
    assert_eq!(twin.max_stake(Character::NullPlayer, RunLength::Full), 1);

    // Starting: anything from 0 to the highest unlocked.
    let start = meta
        .run_start_staked(&content, Character::Declarer, RunLength::Short, 4)
        .expect("stake 4");
    assert_eq!(start.stake, 4);
    assert_eq!(start.length, RunLength::Short);
    assert!(
        meta.run_start_staked(&content, Character::Declarer, RunLength::Short, 2)
            .is_ok()
    );
    let locked = meta.run_start_staked(&content, Character::Declarer, RunLength::Short, 5);
    assert_eq!(locked, Err(MetaError::StakeLocked { stake: 5, max: 4 }));
    assert_eq!(locked.expect_err("locked").code(), "StakeLocked");
    assert_eq!(
        meta.run_start_staked(&content, Character::Declarer, RunLength::Full, 1),
        Err(MetaError::StakeLocked { stake: 1, max: 0 })
    );
    assert_eq!(
        meta.run_start_with(&content, Character::Declarer, RunLength::Short)
            .expect("start")
            .stake,
        0
    );

    // The view lists both lengths for every character.
    let view = meta.view(&content);
    let declarer = view
        .characters
        .iter()
        .find(|c| c.character == Character::Declarer)
        .expect("declarer");
    assert_eq!(declarer.max_stakes.get(&RunLength::Short), Some(&4));
    assert_eq!(declarer.max_stakes.get(&RunLength::Full), Some(&0));

    // Serializable; a meta state from before the ladder loads at 0.
    let text = ron::to_string(&meta).expect("ron");
    assert_eq!(ron::from_str::<MetaState>(&text).expect("back"), meta);
    let old: MetaState = ron::from_str("(version:1,unlocked:[],stats:())").expect("old meta");
    assert_eq!(old.max_stake(Character::Declarer, RunLength::Full), 0);
}

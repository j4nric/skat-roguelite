//! Relics of M8 wave 2a (DESIGN 8.1): one test per relic in a constructed
//! situation, plus the scoring they interact with (Schneider line, the
//! fractional Spielwert factor, the counters of one series).

use super::bot::RunBot;
use super::content_tests::{content, fixed_deal, relic_run};
use super::*;
use crate::cards::Suit;
use crate::content::Factor;
use crate::rules::{NullRisk, RiskLevel};

/// Declarer holds all trumps of a diamonds game but the D8 (in the skat):
/// he wins every trick, the last one with the D7.
const STRONG: [&str; 3] = [
    "CJ SJ HJ DJ DA DT DK DQ D9 D7",
    "HA HT HK HQ H9 H8 H7 SA ST SK",
    "SQ S9 S8 S7 CA CT CK CQ C9 C8",
];
const STRONG_SKAT: &str = "D8 C7";

/// Declarer holds only Luschen, seat 1 all four jacks.
const WEAK: [&str; 3] = [
    "D7 D8 D9 H7 H8 H9 S7 S8 S9 C7",
    "CJ SJ HJ DJ CA CT CK CQ DA DT",
    "DK DQ HA HT HK HQ SA ST SK SQ",
];
const WEAK_SKAT: &str = "C8 C9";

/// Declarer holds ten hearts trumps without the H8 (in the skat) and leads:
/// he takes every trick, the H7 among them.
const HEARTS_SEVEN: [&str; 3] = [
    "CJ SJ HJ DJ HA HT HK HQ H9 H7",
    "DA DT DK DQ D9 D8 D7 SA ST SK",
    "SQ S9 S8 S7 CA CT CK CQ C9 C8",
];
const HEARTS_SEVEN_SKAT: &str = "H8 C7";

/// A grand whose first trick (CJ, HA, DJ) brings the declarer 15 Augen,
/// played by [`WATERTIGHT_PLAY`].
const FIRST_TRICK: [&str; 3] = [
    "CJ SJ HJ CA CT SA ST D7 D8 H7",
    "DA DT DK DQ D9 HA HT HK H9 H8",
    "DJ HQ SK SQ S9 CK CQ C9 C8 C7",
];
const FIRST_TRICK_SKAT: &str = "S8 S7";

fn relic(id: &str) -> Source {
    Source::Relic(RelicId::new(id))
}

fn hand_suit(suit: Suit) -> Declaration {
    Declaration::suit(suit, RiskLevel::Hand)
}

/// Lines of one relic in the game in progress (collected so far).
fn game_lines(run: &Run, id: &str) -> Vec<ScoreLine> {
    let source = relic(id);
    run.game()
        .expect("game")
        .lines
        .iter()
        .filter(|l| l.source == source)
        .cloned()
        .collect()
}

fn breakdown_lines(b: &ScoreBreakdown, id: &str) -> Vec<ScoreLine> {
    b.lines_of(&relic(id)).cloned().collect()
}

/// Deal a fixed hand with a target the series will not reach, so that the
/// run stays in the series.
fn deal(run: &mut Run, content: &Content, hands: [&str; 3], skat: &str) {
    run.series.target = 100_000;
    run.series.game_limit = 100;
    fixed_deal(run, content, hands, skat);
}

/// Win the series and go on to the next one at once (no Tresen).
fn win_series(run: &mut Run, content: &Content) -> Vec<RunEvent> {
    let mut events = Vec::new();
    run.series_won(content, &mut events).expect("won");
    run.shop = None;
    run.encounter = None;
    run.phase = RunPhase::Game;
    events
}

/// Declare a hand game straight away.
fn declare_hand(run: &mut Run, content: &Content, declaration: Declaration) -> Vec<RunEvent> {
    run.apply(content, RunAction::Declare(declaration))
        .expect("declare")
}

/// Pick up the skat, discard the two named cards, declare.
fn declare_after_skat(
    run: &mut Run,
    content: &Content,
    declaration: Declaration,
    discard: [&str; 2],
) -> Vec<RunEvent> {
    let mut events = run.apply(content, RunAction::PickUpSkat).expect("pick up");
    let hand = run.game().expect("game").game.hand(Seat::DECLARER).to_vec();
    let id = |notation: &str| {
        let kind: CardKind = notation.parse().expect("notation");
        hand.iter().find(|c| c.kind == kind).expect("in hand").id
    };
    let ids = [id(discard[0]), id(discard[1])];
    events.extend(
        run.apply(content, RunAction::Discard(ids))
            .expect("discard"),
    );
    events.extend(
        run.apply(content, RunAction::Declare(declaration))
            .expect("declare"),
    );
    events
}

/// Let the bot play the game to its end.
fn bot_to_end(
    run: &mut Run,
    content: &Content,
    mut events: Vec<RunEvent>,
) -> (Vec<RunEvent>, ScoreBreakdown) {
    loop {
        if let Some(b) = scored(&events) {
            return (events, b);
        }
        let action = RunBot.next_action(run, content).expect("game continues");
        events.extend(run.apply(content, action).expect("legal"));
    }
}

fn scored(events: &[RunEvent]) -> Option<ScoreBreakdown> {
    events.iter().find_map(|e| match e {
        RunEvent::GameScored { breakdown, .. } => Some(breakdown.clone()),
        _ => None,
    })
}

/// Play one card of `seat` by notation.
fn play(run: &mut Run, content: &Content, seat: Seat, notation: &str) -> Vec<RunEvent> {
    let kind: CardKind = notation.parse().expect("notation");
    let card = *run
        .game()
        .expect("game")
        .game
        .hand(seat)
        .iter()
        .find(|c| c.kind == kind)
        .expect("in hand");
    let mut events = Vec::new();
    run.play_card(content, seat, card, &mut events)
        .expect("legal");
    events
}

#[test]
fn every_wave_two_relic_loads_and_is_on_offer() {
    let content = content();
    for id in [
        "skat_keeper",
        "old_deck",
        "all_rounder",
        "home_suit",
        "changeling",
        "jack_parade",
        "blank_king",
        "watertight",
    ] {
        let def = content.relic(&RelicId::new(id)).expect(id);
        assert_eq!(def.kind, RelicKind::HouseRule, "{id}");
        assert!(def.price > 0, "{id}");
        assert!(super::bot::relic_value(&def.id) > 0, "the bot buys {id}");
    }
}

#[test]
fn skat_keeper_doubles_the_augen_of_the_discarded_cards() {
    let content = content();
    // Discard DA and DT: 21 Augen in the skat, counted twice.
    let mut run = relic_run(&content, &["skat_keeper"]);
    deal(&mut run, &content, STRONG, STRONG_SKAT);
    let events = declare_after_skat(
        &mut run,
        &content,
        Declaration::suit(Suit::Diamonds, RiskLevel::Normal),
        ["DA", "DT"],
    );
    assert!(events.contains(&RunEvent::SkatPointsRaised {
        source: relic("skat_keeper"),
        card_points: 21,
    }));
    let game = &run.game().expect("game").game;
    assert_eq!(game.outcome().declarer_card_points, 42, "the skat twice");
    assert_eq!(game.outcome().defender_card_points, 0);
    let (_, b) = bot_to_end(&mut run, &content, Vec::new());
    assert_eq!(
        breakdown_lines(&b, "skat_keeper"),
        vec![ScoreLine {
            source: relic("skat_keeper"),
            lever: Lever::CardPoints,
            change: Change::Add(21),
        }]
    );
    // 120 Augen in the cards plus the skat once more.
    assert_eq!(
        b.outcome.declarer_card_points + b.outcome.defender_card_points,
        141
    );
    // A hand game discards nothing.
    let mut run = relic_run(&content, &["skat_keeper"]);
    deal(&mut run, &content, STRONG, STRONG_SKAT);
    let events = declare_hand(&mut run, &content, hand_suit(Suit::Diamonds));
    assert!(
        !events
            .iter()
            .any(|e| matches!(e, RunEvent::SkatPointsRaised { .. }))
    );
    let (_, b) = bot_to_end(&mut run, &content, events);
    assert!(breakdown_lines(&b, "skat_keeper").is_empty());
}

#[test]
fn skat_keeper_adds_the_skat_once_more_for_the_declarer_only() {
    // The declarer holds no Augen at all; he finds two aces in the skat
    // and discards them: 22 Augen, counted twice.
    let content = content();
    let weak = [
        "D7 D8 D9 H7 H8 H9 S7 S8 S9 C7",
        "CJ SJ HJ DJ CT CK CQ DT DK DQ",
        "HA HT HK HQ SA ST SK SQ C8 C9",
    ];
    let mut plain = relic_run(&content, &[]);
    deal(&mut plain, &content, weak, "CA DA");
    let events = declare_after_skat(
        &mut plain,
        &content,
        Declaration::suit(Suit::Clubs, RiskLevel::Normal),
        ["CA", "DA"],
    );
    let (_, without) = bot_to_end(&mut plain, &content, events);
    let mut run = relic_run(&content, &["skat_keeper"]);
    deal(&mut run, &content, weak, "CA DA");
    let events = declare_after_skat(
        &mut run,
        &content,
        Declaration::suit(Suit::Clubs, RiskLevel::Normal),
        ["CA", "DA"],
    );
    let (_, with) = bot_to_end(&mut run, &content, events);
    assert_eq!(
        with.outcome.declarer_card_points,
        without.outcome.declarer_card_points + 22
    );
    assert_eq!(
        with.outcome.defender_card_points,
        without.outcome.defender_card_points
    );
}

#[test]
fn old_deck_counts_discarded_cards_until_the_series_ends() {
    let content = content();
    let mut run = relic_run(&content, &["old_deck"]);
    let line = |n: i64| ScoreLine {
        source: relic("old_deck"),
        lever: Lever::BaseValue,
        change: Change::Add(n),
    };
    // Game 1 with the skat: two cards, +2 already in this game.
    deal(&mut run, &content, STRONG, STRONG_SKAT);
    let events = declare_after_skat(
        &mut run,
        &content,
        Declaration::suit(Suit::Diamonds, RiskLevel::Normal),
        ["C7", "D8"],
    );
    assert!(events.contains(&RunEvent::CounterRaised {
        counter: "old_deck".into(),
        value: 2
    }));
    assert_eq!(game_lines(&run, "old_deck"), vec![line(2)]);
    let (_, b) = bot_to_end(&mut run, &content, events);
    assert_eq!(b.base_value, 9 + 2);
    // Game 2 from the hand: nothing discarded, the +2 stays.
    deal(&mut run, &content, STRONG, STRONG_SKAT);
    declare_hand(&mut run, &content, hand_suit(Suit::Diamonds));
    assert_eq!(game_lines(&run, "old_deck"), vec![line(2)]);
    // Game 3 with the skat again: +4.
    deal(&mut run, &content, STRONG, STRONG_SKAT);
    declare_after_skat(
        &mut run,
        &content,
        Declaration::suit(Suit::Diamonds, RiskLevel::Normal),
        ["C7", "D8"],
    );
    assert_eq!(game_lines(&run, "old_deck"), vec![line(4)]);
    // A won series ends it.
    let events = win_series(&mut run, &content);
    assert!(events.contains(&RunEvent::CounterReset {
        counter: "old_deck".into()
    }));
    assert_eq!(run.counters().get("old_deck"), None);
    deal(&mut run, &content, STRONG, STRONG_SKAT);
    declare_hand(&mut run, &content, hand_suit(Suit::Diamonds));
    assert!(game_lines(&run, "old_deck").is_empty());
}

#[test]
fn all_rounder_pays_three_per_spielart_of_the_series() {
    let content = content();
    let mut run = relic_run(&content, &["all_rounder"]);
    let line = |n: i64| {
        vec![ScoreLine {
            source: relic("all_rounder"),
            lever: Lever::BaseValue,
            change: Change::Add(n),
        }]
    };
    let declare = |run: &mut Run, declaration: Declaration| {
        deal(run, &content, STRONG, STRONG_SKAT);
        declare_hand(run, &content, declaration);
        game_lines(run, "all_rounder")
    };
    // The first game counts its own Spielart.
    assert_eq!(declare(&mut run, hand_suit(Suit::Diamonds)), line(3));
    // Another trump suit is the same Spielart.
    assert_eq!(declare(&mut run, hand_suit(Suit::Clubs)), line(3));
    assert_eq!(
        declare(&mut run, Declaration::grand(RiskLevel::Hand)),
        line(6)
    );
    assert_eq!(
        declare(&mut run, Declaration::null(NullRisk::Hand)),
        line(9)
    );
    assert_eq!(declare(&mut run, hand_suit(Suit::Hearts)), line(9));
    // A new series starts from scratch.
    win_series(&mut run, &content);
    assert_eq!(
        declare(&mut run, Declaration::grand(RiskLevel::Hand)),
        line(3)
    );
    // Played out: the Grundwert of the grand plus 3.
    let (_, b) = bot_to_end(&mut run, &content, Vec::new());
    assert_eq!(b.base_value, 24 + 3);
}

#[test]
fn home_suit_scales_the_third_game_in_a_row_with_the_same_trump() {
    let content = content();
    let mut run = relic_run(&content, &["home_suit"]);
    let scale = vec![ScoreLine {
        source: relic("home_suit"),
        lever: Lever::GameValue,
        change: Change::Scale(Factor::from_permille(1500)),
    }];
    let declare = |run: &mut Run, declaration: Declaration| {
        deal(run, &content, STRONG, STRONG_SKAT);
        declare_hand(run, &content, declaration);
        game_lines(run, "home_suit")
    };
    assert!(declare(&mut run, hand_suit(Suit::Diamonds)).is_empty());
    assert!(declare(&mut run, hand_suit(Suit::Diamonds)).is_empty());
    assert_eq!(declare(&mut run, hand_suit(Suit::Diamonds)), scale);
    // Every further one too, also across a won series.
    win_series(&mut run, &content);
    assert_eq!(declare(&mut run, hand_suit(Suit::Diamonds)), scale);
    // Another trump suit starts over at one.
    assert!(declare(&mut run, hand_suit(Suit::Clubs)).is_empty());
    assert_eq!(run.counters().get("home_suit"), Some(&1));
    assert!(declare(&mut run, hand_suit(Suit::Clubs)).is_empty());
    // A grand ends the streak, so does a Null.
    assert!(declare(&mut run, Declaration::grand(RiskLevel::Hand)).is_empty());
    assert_eq!(run.counters().get("home_suit"), None);
    assert!(declare(&mut run, hand_suit(Suit::Clubs)).is_empty());
    assert!(declare(&mut run, hand_suit(Suit::Clubs)).is_empty());
    assert!(declare(&mut run, Declaration::null(NullRisk::Hand)).is_empty());
    assert!(declare(&mut run, hand_suit(Suit::Clubs)).is_empty());
    // Played out: the Spielwert × 1.5.
    for _ in 0..2 {
        declare(&mut run, hand_suit(Suit::Diamonds));
    }
    deal(&mut run, &content, STRONG, STRONG_SKAT);
    let events = declare_hand(&mut run, &content, hand_suit(Suit::Diamonds));
    let (_, b) = bot_to_end(&mut run, &content, events);
    assert_eq!(b.multiplier, Factor::from_permille(1500));
    assert_eq!(b.value, Factor::from_permille(1500).scale(9 * b.levels));
    assert_eq!(
        b.steps().last().map(|s| s.multiplier),
        Some(Factor::from_permille(1500))
    );
}

#[test]
fn home_suit_rounds_half_up_and_counts_a_loss_too() {
    let content = content();
    let mut run = relic_run(&content, &["home_suit"]);
    for _ in 0..2 {
        deal(&mut run, &content, WEAK, WEAK_SKAT);
        declare_hand(&mut run, &content, hand_suit(Suit::Diamonds));
    }
    deal(&mut run, &content, WEAK, WEAK_SKAT);
    let events = declare_hand(&mut run, &content, hand_suit(Suit::Diamonds));
    let (_, b) = bot_to_end(&mut run, &content, events);
    assert!(!b.won);
    // Lost diamonds Hand: whatever the levels, × 1.5 rounded half up and
    // doubled for the loss.
    let value = (9 * b.levels * 3).div_ceil(2);
    assert_eq!(b.value, value);
    assert_eq!(b.score, -2 * i64::try_from(value).expect("small"));
}

#[test]
fn changeling_adds_a_level_when_the_spielart_changes() {
    let content = content();
    let mut run = relic_run(&content, &["changeling"]);
    let level = vec![ScoreLine {
        source: relic("changeling"),
        lever: Lever::Levels,
        change: Change::Add(1),
    }];
    let declare = |run: &mut Run, declaration: Declaration| {
        deal(run, &content, STRONG, STRONG_SKAT);
        declare_hand(run, &content, declaration);
        game_lines(run, "changeling")
    };
    // No game before.
    assert!(declare(&mut run, hand_suit(Suit::Diamonds)).is_empty());
    // A change of trump suit is no other Spielart.
    assert!(declare(&mut run, hand_suit(Suit::Clubs)).is_empty());
    assert_eq!(
        declare(&mut run, Declaration::grand(RiskLevel::Hand)),
        level
    );
    assert!(declare(&mut run, Declaration::grand(RiskLevel::Hand)).is_empty());
    // Across a won series.
    win_series(&mut run, &content);
    assert_eq!(declare(&mut run, Declaration::null(NullRisk::Hand)), level);
    assert_eq!(declare(&mut run, hand_suit(Suit::Hearts)), level);
    // Played out: one level more than the rules give.
    deal(&mut run, &content, STRONG, STRONG_SKAT);
    let events = declare_hand(&mut run, &content, Declaration::grand(RiskLevel::Hand));
    let (_, b) = bot_to_end(&mut run, &content, events);
    assert_eq!(b.levels, u64::from(b.rule_levels.total()) + 1);
}

#[test]
fn jack_parade_counts_the_jacks_the_declarer_plays() {
    let content = content();
    let mut run = relic_run(&content, &["jack_parade"]);
    deal(&mut run, &content, STRONG, STRONG_SKAT);
    let events = declare_hand(&mut run, &content, hand_suit(Suit::Diamonds));
    let (_, b) = bot_to_end(&mut run, &content, events);
    let lines = breakdown_lines(&b, "jack_parade");
    assert_eq!(lines.len(), 4, "all four jacks");
    assert!(
        lines
            .iter()
            .all(|l| l.lever == Lever::BaseValue && l.change == Change::Add(1))
    );
    assert_eq!(b.base_value, 9 + 4);
    // The defenders' jacks give nothing.
    let mut run = relic_run(&content, &["jack_parade"]);
    deal(&mut run, &content, WEAK, WEAK_SKAT);
    let events = declare_hand(&mut run, &content, hand_suit(Suit::Clubs));
    let (_, b) = bot_to_end(&mut run, &content, events);
    assert!(breakdown_lines(&b, "jack_parade").is_empty());
}

#[test]
fn blank_king_adds_a_level_per_trick_won_by_a_seven_of_the_declarer() {
    let content = content();
    let mut run = relic_run(&content, &["blank_king"]);
    deal(&mut run, &content, HEARTS_SEVEN, HEARTS_SEVEN_SKAT);
    let events = declare_hand(&mut run, &content, hand_suit(Suit::Hearts));
    let (events, b) = bot_to_end(&mut run, &content, events);
    let sevens = events
        .iter()
        .filter(|e| {
            matches!(e, RunEvent::Game(Event::TrickWon { trick, .. })
                if trick.winner.is_declarer()
                    && trick.winning_card().face().is_some_and(|f| f.rank == crate::cards::Rank::Seven))
        })
        .count();
    assert!(sevens >= 1, "the H7 takes a trick");
    let lines = breakdown_lines(&b, "blank_king");
    assert_eq!(lines.len(), sevens);
    assert!(
        lines
            .iter()
            .all(|l| l.lever == Lever::Levels && l.change == Change::Add(1))
    );
    // The defenders' sevens give nothing.
    let mut run = relic_run(&content, &["blank_king"]);
    deal(&mut run, &content, WEAK, WEAK_SKAT);
    let events = declare_hand(&mut run, &content, hand_suit(Suit::Clubs));
    let (_, b) = bot_to_end(&mut run, &content, events);
    assert!(breakdown_lines(&b, "blank_king").is_empty());
}

/// Play a scripted game to its end: `(seat, card)` per card.
fn script(run: &mut Run, content: &Content, cards: &[(usize, &str)]) -> ScoreBreakdown {
    for &(seat, card) in cards {
        play(run, content, Seat::from_index(seat).expect("seat"), card);
    }
    let mut events = Vec::new();
    run.settle(content, &mut events).expect("settle");
    scored(&events).expect("scored")
}

/// Grand Hand mit 3: the first trick (CJ, HA, DJ) brings 15 Augen, the
/// declarer ends with 79, the defenders with 41: Schneider only with the
/// first trick counted twice (41 ≤ 30 + 15).
const WATERTIGHT_PLAY: [(usize, &str); 30] = [
    (0, "CJ"),
    (1, "HA"),
    (2, "DJ"),
    (0, "D7"),
    (1, "DA"),
    (2, "CK"),
    (1, "DT"),
    (2, "CQ"),
    (0, "D8"),
    (1, "HT"),
    (2, "HQ"),
    (0, "H7"),
    (1, "HK"),
    (2, "C7"),
    (0, "SJ"),
    (0, "HJ"),
    (1, "D9"),
    (2, "C8"),
    (0, "CA"),
    (1, "H9"),
    (2, "C9"),
    (0, "CT"),
    (1, "H8"),
    (2, "S9"),
    (0, "SA"),
    (1, "DK"),
    (2, "SQ"),
    (0, "ST"),
    (1, "DQ"),
    (2, "SK"),
];

#[test]
fn watertight_counts_the_first_trick_twice_for_schneider_only() {
    let content = content();
    let mut run = relic_run(&content, &["watertight"]);
    deal(&mut run, &content, FIRST_TRICK, FIRST_TRICK_SKAT);
    declare_hand(&mut run, &content, Declaration::grand(RiskLevel::Hand));
    for &(seat, card) in &WATERTIGHT_PLAY[..3] {
        play(
            &mut run,
            &content,
            Seat::from_index(seat).expect("seat"),
            card,
        );
    }
    let rules = &run.game().expect("game").rules;
    assert_eq!(rules.thresholds.schneider_bonus, 15);
    assert_eq!(rules.schneider_bonus_source, Some(relic("watertight")));
    let b = script(&mut run, &content, &WATERTIGHT_PLAY[3..]);
    // Only the first trick counts, winning does not move.
    assert_eq!(b.thresholds.schneider_bonus, 15);
    assert_eq!(b.thresholds.win, 61);
    assert_eq!(
        (
            b.outcome.declarer_card_points,
            b.outcome.defender_card_points
        ),
        (79, 41)
    );
    assert!(b.won && b.schneider);
    assert_eq!(
        breakdown_lines(&b, "watertight"),
        vec![ScoreLine {
            source: relic("watertight"),
            lever: Lever::Schneider,
            change: Change::Set(1),
        }]
    );
    // The same game without it is no Schneider: one level less.
    let mut plain = relic_run(&content, &[]);
    deal(&mut plain, &content, FIRST_TRICK, FIRST_TRICK_SKAT);
    declare_hand(&mut plain, &content, Declaration::grand(RiskLevel::Hand));
    let without = script(&mut plain, &content, &WATERTIGHT_PLAY);
    assert_eq!(without.outcome, b.outcome);
    assert!(without.won && !without.schneider);
    // Grand Hand mit 3: 3 + 1 + 1 = 5 levels, 6 with Schneider.
    assert_eq!((without.levels, b.levels), (5, 6));
    assert_eq!(b.value, 24 * 6);
    // A Schneider by the rules needs no line.
    let mut run = relic_run(&content, &["watertight"]);
    deal(&mut run, &content, STRONG, STRONG_SKAT);
    let events = declare_hand(&mut run, &content, hand_suit(Suit::Diamonds));
    let (_, b) = bot_to_end(&mut run, &content, events);
    assert!(b.schneider);
    assert!(breakdown_lines(&b, "watertight").is_empty());
}

/// Bot play until the declarer is to move with `left` tricks to go.
fn bot_until(run: &mut Run, content: &Content, left: usize) {
    loop {
        let game = &run.game().expect("game").game;
        if game.tricks().len() + left >= crate::game::TRICKS_PER_GAME
            && game.current_seat() == Some(Seat::DECLARER)
        {
            return;
        }
        let action = RunBot.next_action(run, content).expect("game continues");
        run.apply(content, action).expect("legal");
    }
}

#[test]
fn in_play_hooks_let_the_rest_be_settled_once_they_cannot_fire() {
    // "Rest abkürzen" waits while a hook may still fire. Bubenparade:
    // only the declarer's own jacks; Luschenkönig: only his sevens;
    // Stichfest: only until his first trick.
    let content = content();
    let settle_at = |relics: &[&str], left: usize| {
        let mut run = relic_run(&content, relics);
        deal(&mut run, &content, STRONG, STRONG_SKAT);
        declare_hand(&mut run, &content, hand_suit(Suit::Diamonds));
        bot_until(&mut run, &content, left);
        let jacks = run
            .game()
            .expect("game")
            .game
            .hand(Seat::DECLARER)
            .iter()
            .filter(|c| c.face().is_some_and(|f| f.rank == crate::cards::Rank::Jack))
            .count();
        (run.can_settle(&content), jacks)
    };
    let (plain, jacks) = settle_at(&[], 3);
    assert!(plain, "the strong hand is decided late in the game");
    assert_eq!(jacks, 0, "the bot led its jacks");
    assert_eq!(settle_at(&["jack_parade"], 3).0, plain);
    assert_eq!(settle_at(&["watertight"], 3).0, plain);
    // The D7 is still in his hand: it may take a trick.
    assert!(!settle_at(&["blank_king"], 3).0);
    // Before he played his jacks Bubenparade keeps the game open.
    let mut run = relic_run(&content, &["jack_parade"]);
    deal(&mut run, &content, STRONG, STRONG_SKAT);
    declare_hand(&mut run, &content, hand_suit(Suit::Diamonds));
    assert!(!run.can_settle(&content));
}

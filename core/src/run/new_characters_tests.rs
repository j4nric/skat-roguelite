//! Tests of the characters and the night boss of 10.11.2026 (DESIGN 9.6b,
//! 9.6c, 11.1): Der Zocker (hand counts two levels, the skat once per pub,
//! Knobelbecher), Die Ramschtante (Ramsch instead of a declaration, worn
//! Daus and Zehnen, Spitzendeckchen) and Der Totengräber (discarded cards
//! count no Augen, only in the long night), plus the bot's choices for
//! them.

use super::bot::RunBot;
use super::content_tests::{content, fixed_deal, play};
use super::*;
use crate::cards::{PlayingCard, Suit};
use crate::content::{Change, Factor, Lever, Source};
use crate::rules::{Declaration, GameType, NullRisk, RiskLevel};

/// The player holds the four jacks and six diamonds; the defenders hold no
/// jack and no diamond.
const STRONG: [&str; 3] = [
    "CJ SJ HJ DJ DA DT DK DQ D9 D7",
    "HA HT HK HQ H9 H8 H7 SA ST SK",
    "SQ S9 S8 S7 CA CT CK CQ C9 C8",
];
const STRONG_SKAT: &str = "D8 C7";

/// The player holds only low cards: a safe Ramsch.
const LUSCHEN: [&str; 3] = [
    "C7 C8 C9 S7 S8 S9 H7 H8 H9 D7",
    "CJ SJ CA CT CK CQ SA ST SK SQ",
    "HJ DJ HA HT HK HQ DA DT DK DQ",
];
const LUSCHEN_SKAT: &str = "D8 D9";

/// Low cards and a few Könige and Oberinnen: no Null Ouvert Hand, but a
/// good Ramsch.
const LOW: [&str; 3] = [
    "C7 C8 CK S7 S8 SK H7 H8 HQ DQ",
    "CJ SJ CA CT CQ C9 SA ST SQ S9",
    "HJ DJ HA HT HK H9 DA DT DK D7",
];

fn run_as(content: &Content, character: Character) -> Run {
    let start = RunStart {
        character,
        ..RunStart::default()
    };
    Run::new_with(content, 11, &start).expect("start").0
}

fn id_of(run: &Run, s: &str) -> u32 {
    run.edition()
        .id_of(s.parse().expect("card"))
        .expect("card in the edition")
}

fn mods_of(run: &Run, s: &str) -> Vec<(String, i32)> {
    let card = run.edition().get(id_of(run, s)).expect("card");
    card.mods
        .iter()
        .map(|m| (m.to_string(), card.mod_value(m)))
        .collect()
}

fn hand(game: GameType) -> Declaration {
    Declaration::new(game, RiskLevel::Hand).expect("suit or grand")
}

fn source_lines(b: &ScoreBreakdown, source: &Source) -> Vec<(Lever, Change)> {
    b.lines
        .iter()
        .filter(|l| &l.source == source)
        .map(|l| (l.lever, l.change))
        .collect()
}

fn declare_options(run: &Run, content: &Content) -> super::view::DeclareOptions {
    run.view(content)
        .game
        .and_then(|g| g.declare)
        .expect("declare options")
}

/// Play the game out with the bot after a declaration: every event and
/// the breakdown.
fn finish(
    run: &mut Run,
    content: &Content,
    mut events: Vec<RunEvent>,
) -> (Vec<RunEvent>, ScoreBreakdown) {
    loop {
        if let Some(b) = events.iter().find_map(|e| match e {
            RunEvent::GameScored { breakdown, .. } => Some(breakdown.clone()),
            _ => None,
        }) {
            return (events, b);
        }
        let action = RunBot.next_action(run, content).expect("game continues");
        events.extend(run.apply(content, action).expect("legal"));
    }
}

// ---------------------------------------------------------------------------
// Der Zocker
// ---------------------------------------------------------------------------

#[test]
fn gambler_starts_with_wild_sevens_open_aces_and_the_dice_cup() {
    let content = content();
    let run = run_as(&content, Character::Gambler);
    for seven in ["C7", "S7"] {
        assert_eq!(mods_of(&run, seven), vec![("wild_card".into(), 1)]);
    }
    for ace in ["CA", "SA", "HA", "DA"] {
        assert_eq!(mods_of(&run, ace), vec![("transparent".into(), 1)]);
    }
    assert!(mods_of(&run, "H7").is_empty());
    assert_eq!(run.relics(), &[RelicId::new("dice_cup")]);
    assert_eq!(run.view(&content).character.back, "gambler");
}

#[test]
fn gambler_hand_games_count_two_levels_for_hand() {
    let content = content();
    for declaration in [
        hand(GameType::Suit(Suit::Diamonds)),
        hand(GameType::Grand),
        Declaration::null(NullRisk::Hand),
    ] {
        let mut run = run_as(&content, Character::Gambler);
        fixed_deal(&mut run, &content, STRONG, STRONG_SKAT);
        let (_, b) = play(&mut run, &content, declaration);
        assert_eq!(
            source_lines(&b, &Source::Character(Character::Gambler)),
            vec![(Lever::Levels, Change::Add(1))],
            "{declaration:?}"
        );
    }
    // A won suit game from the hand: Hand +1 and the Zocker's +1.
    let mut run = run_as(&content, Character::Gambler);
    fixed_deal(&mut run, &content, STRONG, STRONG_SKAT);
    let (_, b) = play(&mut run, &content, hand(GameType::Grand));
    assert!(b.won);
    assert_eq!(b.rule_levels.hand, 1);
    assert_eq!(b.levels, u64::from(b.rule_levels.total()) + 1);
    // After the skat nothing is added.
    let mut run = run_as(&content, Character::Gambler);
    fixed_deal(&mut run, &content, STRONG, STRONG_SKAT);
    let normal = Declaration::grand(RiskLevel::Normal);
    let (_, b) = play(&mut run, &content, normal);
    assert!(source_lines(&b, &Source::Character(Character::Gambler)).is_empty());
}

#[test]
fn gambler_picks_up_the_skat_only_once_per_pub() {
    let content = content();
    let mut run = run_as(&content, Character::Gambler);
    run.series.target = 100_000;
    assert_eq!(run.skat_pickups_left(), Some(1));
    assert_eq!(declare_options(&run, &content).skat_pickups_left, Some(1));
    fixed_deal(&mut run, &content, STRONG, STRONG_SKAT);
    let (_, b) = play(&mut run, &content, Declaration::grand(RiskLevel::Normal));
    assert!(b.won);
    // The next game of the same pub: only from the hand.
    assert_eq!(run.phase(), RunPhase::Game);
    assert_eq!(run.series().games_played, 1);
    assert!(run.skat_used_up());
    assert!(!run.accepts(&content, RunAction::PickUpSkat));
    assert_eq!(
        run.apply(&content, RunAction::PickUpSkat),
        Err(RunError::SkatUsedUp)
    );
    let options = declare_options(&run, &content);
    assert!(!options.can_pick_up_skat);
    assert_eq!(options.skat_pickups_left, Some(0));
    assert!(run.accepts(&content, RunAction::Declare(hand(GameType::Grand))));
    // Einmischen is still allowed and keeps the rule.
    if run.accepts(&content, RunAction::Redeal) {
        run.apply(&content, RunAction::Redeal).expect("redeal");
        assert!(run.skat_used_up());
    }
    // A new pub opens the skat again.
    let mut next = run.series.clone();
    next.index += 1;
    run.series = Series {
        section: next.section,
        index: next.index,
        target: next.target,
        game_limit: next.game_limit,
        ..Series::default()
    };
    assert_eq!(run.skat_pickups_left(), Some(1));
}

#[test]
fn other_characters_pick_up_the_skat_in_every_game() {
    let content = content();
    let mut run = run_as(&content, Character::Declarer);
    run.series.target = 100_000;
    assert_eq!(run.skat_pickups_left(), None);
    fixed_deal(&mut run, &content, STRONG, STRONG_SKAT);
    play(&mut run, &content, Declaration::grand(RiskLevel::Normal));
    assert!(!run.skat_used_up());
    assert!(run.accepts(&content, RunAction::PickUpSkat));
    assert_eq!(declare_options(&run, &content).skat_pickups_left, None);
}

#[test]
fn dice_cup_halves_a_lost_hand_game() {
    let content = content();
    let lose = [STRONG[2], STRONG[1], STRONG[0]];
    let lost = |relics: Vec<RelicId>| {
        let start = RunStart {
            relics,
            ..RunStart::default()
        };
        let mut run = Run::new_with(&content, 11, &start).expect("start").0;
        fixed_deal(&mut run, &content, lose, STRONG_SKAT);
        play(&mut run, &content, hand(GameType::Suit(Suit::Hearts))).1
    };
    let plain = lost(Vec::new());
    let cup = lost(vec![RelicId::new("dice_cup")]);
    assert!(!plain.won && !cup.won);
    assert_eq!(
        source_lines(&cup, &Source::Relic(RelicId::new("dice_cup"))),
        vec![(Lever::GameValue, Change::Scale(Factor::from_permille(500)))]
    );
    assert_eq!(cup.value, plain.value.div_ceil(2));
    // A won hand game and a lost game after the skat stay as they are.
    let start = RunStart {
        relics: vec![RelicId::new("dice_cup")],
        ..RunStart::default()
    };
    let mut run = Run::new_with(&content, 11, &start).expect("start").0;
    fixed_deal(&mut run, &content, STRONG, STRONG_SKAT);
    let (_, won) = play(&mut run, &content, hand(GameType::Grand));
    assert!(won.won);
    assert!(source_lines(&won, &Source::Relic(RelicId::new("dice_cup"))).is_empty());
}

// ---------------------------------------------------------------------------
// Die Ramschtante
// ---------------------------------------------------------------------------

#[test]
fn ramsch_aunt_starts_with_worn_aces_and_tens_and_the_doily() {
    let content = content();
    let run = run_as(&content, Character::RamschAunt);
    for card in ["CA", "SA", "HA", "DA", "CT", "ST", "HT", "DT"] {
        assert_eq!(mods_of(&run, card), vec![("battered".into(), 5)], "{card}");
    }
    assert!(mods_of(&run, "CK").is_empty());
    assert_eq!(run.relics(), &[RelicId::new("lace_doily")]);
    assert_eq!(run.view(&content).character.back, "ramsch_aunt");
    // The doily is hers alone.
    let doily = content.relic(&RelicId::new("lace_doily")).expect("relic");
    assert_eq!(doily.characters, vec![Character::RamschAunt]);
}

#[test]
fn ramsch_aunt_may_declare_a_ramsch_before_the_skat() {
    let content = content();
    let mut run = run_as(&content, Character::RamschAunt);
    fixed_deal(&mut run, &content, LUSCHEN, LUSCHEN_SKAT);
    assert!(run.ramsch_offered());
    assert!(declare_options(&run, &content).ramsch_offered);
    let events = run
        .apply(&content, RunAction::Declare(Declaration::Ramsch))
        .expect("Ramsch");
    assert!(run.series().ramsch_played);
    assert_eq!(
        run.game().and_then(|a| a.game.declaration()),
        Some(Declaration::Ramsch)
    );
    let (_, b) = finish(&mut run, &content, events);
    assert!(b.won, "with only Luschen someone else takes the most Augen");
    let seats = b.seats.expect("seats");
    let most = seats.card_points.iter().copied().max().expect("three");
    assert!(seats.card_points[0] < most);
    // Grundwert: the Augen of the loser.
    assert_eq!(b.base_value, u64::from(most));
    // Spitzendeckchen: at most two tricks of hers, +1 level.
    assert!(seats.tricks[0] <= 2);
    assert_eq!(
        source_lines(&b, &Source::Relic(RelicId::new("lace_doily"))),
        vec![(Lever::Levels, Change::Add(1))]
    );
}

#[test]
fn ramsch_is_no_choice_after_the_skat_nor_for_others() {
    let content = content();
    let mut run = run_as(&content, Character::RamschAunt);
    fixed_deal(&mut run, &content, LUSCHEN, LUSCHEN_SKAT);
    run.apply(&content, RunAction::PickUpSkat).expect("skat");
    assert!(!run.ramsch_offered());
    let game = &run.game().expect("game").game;
    let [a, b] = RunBot::discard_for(
        game.hand(Seat::DECLARER),
        Declaration::grand(RiskLevel::Normal),
        game.deck_cards(),
    )
    .expect("discard");
    run.apply(&content, RunAction::Discard([a.id, b.id]))
        .expect("discard");
    assert_eq!(
        run.apply(&content, RunAction::Declare(Declaration::Ramsch)),
        Err(RunError::GameNotAvailable)
    );
    let mut other = run_as(&content, Character::Declarer);
    fixed_deal(&mut other, &content, LUSCHEN, LUSCHEN_SKAT);
    assert!(!other.ramsch_offered());
    assert!(!declare_options(&other, &content).ramsch_offered);
    assert_eq!(
        other.apply(&content, RunAction::Declare(Declaration::Ramsch)),
        Err(RunError::GameNotAvailable)
    );
}

#[test]
fn a_ramsch_of_the_aunt_redeems_no_voucher_and_draws_no_kontra() {
    let content = content();
    let start = RunStart {
        character: Character::RamschAunt,
        relics: vec![RelicId::new("lace_doily")],
        ..RunStart::default()
    };
    let mut run = Run::new_with(&content, 11, &start).expect("start").0;
    let seven = id_of(&run, "C7");
    run.edition
        .add_mod(
            &content,
            seven,
            &crate::content::CardModId::new("voucher_coins"),
            2,
        )
        .expect("voucher");
    fixed_deal(&mut run, &content, LUSCHEN, LUSCHEN_SKAT);
    let coins = run.coins();
    let events = run
        .apply(&content, RunAction::Declare(Declaration::Ramsch))
        .expect("Ramsch");
    assert!(!events.iter().any(|e| matches!(
        e,
        RunEvent::VoucherRedeemed { .. } | RunEvent::Kontra { .. }
    )));
    assert_eq!(run.coins(), coins);
}

// ---------------------------------------------------------------------------
// The bot
// ---------------------------------------------------------------------------

#[test]
fn the_bot_plays_a_ramsch_with_luschen_and_a_game_with_a_strong_hand() {
    let content = content();
    let mut run = run_as(&content, Character::RamschAunt);
    fixed_deal(&mut run, &content, LOW, LUSCHEN_SKAT);
    assert_eq!(
        RunBot.next_action(&run, &content),
        Some(RunAction::Declare(Declaration::Ramsch))
    );
    let mut run = run_as(&content, Character::RamschAunt);
    fixed_deal(&mut run, &content, STRONG, STRONG_SKAT);
    assert_ne!(
        RunBot.next_action(&run, &content),
        Some(RunAction::Declare(Declaration::Ramsch))
    );
    // Without the rule never.
    let mut run = run_as(&content, Character::Declarer);
    fixed_deal(&mut run, &content, LOW, LUSCHEN_SKAT);
    assert_ne!(
        RunBot.next_action(&run, &content),
        Some(RunAction::Declare(Declaration::Ramsch))
    );
}

#[test]
fn ramsch_danger_follows_jacks_and_high_cards() {
    use super::bot::{ramsch_danger, ramsch_expected_score};
    let cards = |s: &str| -> Vec<crate::cards::Card> {
        s.split_whitespace()
            .map(|c| c.parse().expect("card"))
            .collect()
    };
    let luschen = cards(LUSCHEN[0]);
    let jacks = cards(STRONG[0]);
    let heavy = cards("CJ SJ HJ DJ CA SA HA DA CT ST");
    assert_eq!(ramsch_danger(&luschen), 0);
    assert!(ramsch_danger(&jacks) > 14);
    assert!(ramsch_danger(&heavy) > ramsch_danger(&jacks));
    assert!(ramsch_expected_score(&luschen) > ramsch_expected_score(&jacks));
    assert!(ramsch_expected_score(&luschen) > 0);
    assert!(ramsch_expected_score(&heavy) < 0);
    // A bare Daus is worse than one with two low cards to duck with.
    assert!(ramsch_danger(&cards("CA")) > ramsch_danger(&cards("CA C7 C8")));
}

#[test]
fn the_gambler_bot_never_picks_up_a_used_skat_and_keeps_it_for_weak_hands() {
    let content = content();
    // A strong hand in the first game: played from the hand.
    let mut run = run_as(&content, Character::Gambler);
    fixed_deal(&mut run, &content, STRONG, STRONG_SKAT);
    match RunBot.next_action(&run, &content) {
        Some(RunAction::Declare(d)) => assert!(d.hand(), "{d:?}"),
        other => panic!("expected a hand game, got {other:?}"),
    }
    // The skat used up: never PickUpSkat, whatever the hand.
    for seed in 0..40 {
        let start = RunStart {
            character: Character::Gambler,
            ..RunStart::default()
        };
        let (mut run, _) = Run::new_with(&content, seed, &start).expect("start");
        run.series.skat_pickups = 1;
        let action = RunBot.next_action(&run, &content).expect("action");
        assert_ne!(action, RunAction::PickUpSkat, "seed {seed}");
        if let RunAction::Declare(d) = action {
            assert!(d.hand(), "seed {seed}: {d:?}");
        }
    }
}

// ---------------------------------------------------------------------------
// Der Totengräber
// ---------------------------------------------------------------------------

/// Pick up the skat, discard the club Daus and Zehn (21 Augen) and play a
/// grand.
fn discard_full_skat(run: &mut Run, content: &Content) -> (Vec<RunEvent>, ScoreBreakdown) {
    let deal = [
        "CJ SJ HJ DJ DA DT DK DQ D9 D7",
        "HA HT HK HQ H9 H8 H7 SA ST SK",
        "SQ S9 S8 S7 D8 C7 CK CQ C9 C8",
    ];
    fixed_deal(run, content, deal, "CA CT");
    let mut events = run.apply(content, RunAction::PickUpSkat).expect("skat");
    let ca = run
        .game()
        .expect("game")
        .game
        .hand(Seat::DECLARER)
        .iter()
        .find(|c| c.face() == Some("CA".parse().expect("card")))
        .map(|c| c.id)
        .expect("CA");
    let ct = run
        .game()
        .expect("game")
        .game
        .hand(Seat::DECLARER)
        .iter()
        .find(|c| c.face() == Some("CT".parse().expect("card")))
        .map(|c| c.id)
        .expect("CT");
    events.extend(
        run.apply(content, RunAction::Discard([ca, ct]))
            .expect("discard"),
    );
    events.extend(
        run.apply(
            content,
            RunAction::Declare(Declaration::grand(RiskLevel::Normal)),
        )
        .expect("grand"),
    );
    finish(run, content, events)
}

fn discarded(events: &[RunEvent]) -> Option<u32> {
    events.iter().find_map(|e| match e {
        RunEvent::GameScored {
            discarded_points, ..
        } => Some(*discarded_points),
        _ => None,
    })
}

#[test]
fn gravedigger_buries_the_discarded_augen() {
    let content = content();
    let boss = BossId::new("gravedigger");
    let def = content.boss(&boss).expect("boss");
    assert_eq!(def.figures.len(), 2);
    assert_eq!(def.back.as_deref(), Some("gravedigger"));
    // Without him the discard counts 21 Augen.
    let mut run = run_as(&content, Character::Declarer);
    let (events, plain) = discard_full_skat(&mut run, &content);
    assert_eq!(discarded(&events), Some(21));
    let mut run = run_as(&content, Character::Declarer);
    run.series.boss = Some(boss.clone());
    run.series.index = PUBS_PER_DISTRICT - 1;
    let (events, buried) = discard_full_skat(&mut run, &content);
    assert!(events.iter().any(|e| matches!(
        e,
        RunEvent::SkatPointsRaised { source: Source::Boss(b), card_points: -21 } if *b == boss
    )));
    assert_eq!(discarded(&events), Some(0));
    let plain_seat = plain.seats.expect("seats").card_points[0];
    let buried_seat = buried.seats.expect("seats").card_points[0];
    assert_eq!(plain_seat, buried_seat + 21);
}

#[test]
fn gravedigger_leaves_hand_games_and_null_alone() {
    let content = content();
    let boss = BossId::new("gravedigger");
    for declaration in [hand(GameType::Grand), Declaration::null(NullRisk::Hand)] {
        let mut run = run_as(&content, Character::Declarer);
        run.series.boss = Some(boss.clone());
        run.series.index = PUBS_PER_DISTRICT - 1;
        fixed_deal(&mut run, &content, STRONG, STRONG_SKAT);
        let (events, _) = play(&mut run, &content, declaration);
        assert!(
            !events
                .iter()
                .any(|e| matches!(e, RunEvent::SkatPointsRaised { .. })),
            "{declaration:?}"
        );
    }
}

#[test]
fn gravedigger_is_a_boss_of_the_night_only() {
    let content = content();
    let boss = BossId::new("gravedigger");
    assert!(content.run.night_bosses.contains(&boss));
    assert!(!content.run.bosses.contains(&boss));
    assert_ne!(content.run.final_boss.as_ref(), Some(&boss));
    // A boss of the day as well is a load error, an unknown one too.
    let sources = crate::content::Sources::embedded();
    let run = sources.run.replace(
        r#"night_bosses: ["gravedigger"],"#,
        r#"night_bosses: ["twins"],"#,
    );
    assert_ne!(run, sources.run);
    assert!(
        Content::from_sources(&crate::content::Sources {
            run: &run,
            ..sources
        })
        .is_err()
    );
    let run = sources.run.replace(
        r#"night_bosses: ["gravedigger"],"#,
        r#"night_bosses: ["nobody"],"#,
    );
    assert!(
        Content::from_sources(&crate::content::Sources {
            run: &run,
            ..sources
        })
        .is_err()
    );
    // The day never draws him.
    for seed in 0..60 {
        let (run, _) = Run::new_with(&content, seed, &RunStart::default()).expect("start");
        assert_ne!(run.section_boss(), Some(&boss), "seed {seed}");
    }
}

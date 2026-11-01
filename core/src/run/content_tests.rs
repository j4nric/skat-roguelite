//! One test per content item: load the embedded content, put the item into
//! a run and check its effect in a constructed situation.

use super::bot::RunBot;
use super::*;
use crate::cards::PlayingCard;
use crate::cards::Suit;
use crate::content::{CardModId, GameFocus};
use crate::rules::{GameType, Matadors, Outcome, RiskLevel};

pub(super) fn content() -> Content {
    Content::load().expect("embedded content is valid")
}

/// Declarer holds all trumps of a diamonds game but the D8 (in the skat):
/// he wins every trick, the last one with the D7. Instance ids follow the
/// standard deck: D7 = 0, DA = 7, HA = 15, SK = 22, ST = 19.
const STRONG: [&str; 3] = [
    "CJ SJ HJ DJ DA DT DK DQ D9 D7",
    "HA HT HK HQ H9 H8 H7 SA ST SK",
    "SQ S9 S8 S7 CA CT CK CQ C9 C8",
];
const STRONG_SKAT: &str = "D8 C7";

/// Declarer holds only Luschen: he can never reach 61 Augen. Seat 1 holds
/// eight trumps of a clubs game and says Kontra.
const WEAK: [&str; 3] = [
    "D7 D8 D9 H7 H8 H9 S7 S8 S9 C7",
    "CJ SJ HJ DJ CA CT CK CQ DA DT",
    "DK DQ HA HT HK HQ SA ST SK SQ",
];
const WEAK_SKAT: &str = "C8 C9";

/// Declarer holds ten hearts trumps without the H8 (in the skat) and leads:
/// he takes every trick, the last one with the H7.
const HEARTS_SEVEN: [&str; 3] = [
    "CJ SJ HJ DJ HA HT HK HQ H9 H7",
    "DA DT DK DQ D9 D8 D7 SA ST SK",
    "SQ S9 S8 S7 CA CT CK CQ C9 C8",
];
const HEARTS_SEVEN_SKAT: &str = "H8 C7";

pub(super) fn relic_run(content: &Content, relics: &[&str]) -> Run {
    let (mut run, _) = Run::new(content, 11).expect("start");
    run.relics = relics.iter().map(|r| RelicId::new(r)).collect();
    run
}

/// Replace the current game by a constructed deal (declarer is forehand),
/// with all content effects of a normal deal.
pub(super) fn fixed_deal(
    run: &mut Run,
    content: &Content,
    hands: [&str; 3],
    skat: &str,
) -> Vec<RunEvent> {
    let global = run.sources(content, false);
    let resolved = resolve_deck(&run.deck, content, &global);
    let mut pool = resolved.cards.clone();
    let mut take = |notation: &str| -> DealtCard {
        let kind: CardKind = notation.parse().expect("notation");
        let index = pool
            .iter()
            .position(|c| c.kind == kind)
            .expect("card in deck");
        pool.remove(index)
    };
    let parse = |s: &str, take: &mut dyn FnMut(&str) -> DealtCard| -> Vec<DealtCard> {
        s.split_whitespace().map(take).collect()
    };
    let hands = [
        parse(hands[0], &mut take),
        parse(hands[1], &mut take),
        parse(hands[2], &mut take),
    ];
    let skat = parse(skat, &mut take);
    let game = Game::with_talon(hands, [skat[0], skat[1]], pool, Seat::DECLARER).expect("deal");
    let mut events = Vec::new();
    run.set_up_game(content, game, resolved.point_changes, &mut events);
    events
}

/// Declare in the order of the Skatordnung (ADR 0011): a hand game at once,
/// any other game after picking up the skat and discarding as the bot does
/// for this game.
pub(super) fn declare(
    run: &mut Run,
    content: &Content,
    declaration: Declaration,
) -> Result<Vec<RunEvent>, RunError> {
    if declaration.hand() {
        return run.apply(content, RunAction::Declare(declaration));
    }
    let mut events = run.apply(content, RunAction::PickUpSkat)?;
    let game = &run.game().ok_or(RunError::WrongPhase)?.game;
    let [a, b] = RunBot::discard_for(game.hand(Seat::DECLARER), declaration, game.deck_cards())
        .ok_or(RunError::WrongPhase)?;
    events.extend(run.apply(content, RunAction::Discard([a.id, b.id]))?);
    events.extend(run.apply(content, RunAction::Declare(declaration))?);
    Ok(events)
}

/// Declare and let the bot play until the game is scored.
pub(super) fn play(
    run: &mut Run,
    content: &Content,
    declaration: Declaration,
) -> (Vec<RunEvent>, ScoreBreakdown) {
    let mut events = declare(run, content, declaration).expect("declare");
    loop {
        if let Some(breakdown) = events.iter().find_map(|e| match e {
            RunEvent::GameScored { breakdown, .. } => Some(breakdown.clone()),
            _ => None,
        }) {
            return (events, breakdown);
        }
        let action = RunBot.next_action(run, content).expect("game continues");
        events.extend(run.apply(content, action).expect("legal"));
    }
}

fn hand_game(game: GameType) -> Declaration {
    Declaration::new(game, RiskLevel::Hand).expect("suit or grand")
}

const DIAMONDS: GameType = GameType::Suit(Suit::Diamonds);
const CLUBS: GameType = GameType::Suit(Suit::Clubs);
const HEARTS: GameType = GameType::Suit(Suit::Hearts);

fn relic(id: &str) -> Source {
    Source::Relic(RelicId::new(id))
}

fn has_line(b: &ScoreBreakdown, source: &Source, lever: Lever, change: Change) -> bool {
    b.lines
        .iter()
        .any(|l| &l.source == source && l.lever == lever && l.change == change)
}

#[test]
fn every_content_item_loads_with_german_texts() {
    let content = content();
    for id in [
        "bock_round",
        "kontra",
        "last_trick_seven",
        "concede",
        "ouvert_bonus",
        "skat_redraw",
        "virgin",
        "old_lady",
        "tour",
        "march_through",
    ] {
        let def = content.relic(&RelicId::new(id)).expect(id);
        assert!(
            !def.name.is_empty() && !def.text.is_empty() && def.price > 0,
            "{id}"
        );
    }
    for id in ["fat", "marked", "stamp", "last_trick", "transparent"] {
        assert!(content.card_mod(&CardModId::new(id)).is_some(), "{id}");
    }
    for id in ["the_pusher", "kontra_hans", "grandma"] {
        assert!(content.boss(&BossId::new(id)).is_some(), "{id}");
    }
    for id in ["fresh_deck", "ink_pad", "beer", "new_deal"] {
        assert!(content.consumable(&ConsumableId::new(id)).is_some(), "{id}");
    }
    // Calibrated in docs/balance/2026-11-01-m5-v1.md (v1, PIMC player).
    assert_eq!(content.run.targets[0], (36, 44, 54));
    assert_eq!(content.run.targets[4], (60, 66, 72));
    assert_eq!((content.run.game_limit, content.run.passes), (8, 6));
}

// ---------------------------------------------------------------------------
// House rules
// ---------------------------------------------------------------------------

#[test]
fn bock_round_doubles_the_base_value_after_a_lost_game() {
    let content = content();
    let mut run = relic_run(&content, &["bock_round"]);
    fixed_deal(&mut run, &content, WEAK, WEAK_SKAT);
    let (events, lost) = play(&mut run, &content, hand_game(CLUBS));
    assert!(!lost.won);
    assert!(events.contains(&RunEvent::FlagSet("bock".into())));
    assert!(run.flags().contains("bock"));
    // The next game counts double and uses the flag up.
    fixed_deal(&mut run, &content, STRONG, STRONG_SKAT);
    let (events, won) = play(&mut run, &content, hand_game(DIAMONDS));
    assert!(events.contains(&RunEvent::FlagCleared("bock".into())));
    assert!(has_line(
        &won,
        &relic("bock_round"),
        Lever::BaseValue,
        Change::Multiply(2)
    ));
    assert_eq!(won.base_value, 18);
    assert!(!run.flags().contains("bock"));
}

#[test]
fn kontra_doubles_and_re_quadruples() {
    let content = content();
    let mut run = relic_run(&content, &["kontra"]);
    fixed_deal(&mut run, &content, WEAK, WEAK_SKAT);
    let events = run
        .apply(&content, RunAction::Declare(hand_game(CLUBS)))
        .expect("declare");
    assert!(events.contains(&RunEvent::Kontra {
        seat: Seat::DEFENDERS[0]
    }));
    assert!(run.awaiting_kontra_answer());
    assert_eq!(
        run.apply(&content, RunAction::Play(0)),
        Err(RunError::AwaitingKontraAnswer)
    );
    let mut events = run
        .apply(&content, RunAction::AnswerKontra { re: true })
        .expect("re");
    assert!(events.contains(&RunEvent::Re));
    while !events
        .iter()
        .any(|e| matches!(e, RunEvent::GameScored { .. }))
    {
        let action = RunBot.next_action(&run, &content).expect("action");
        events.extend(run.apply(&content, action).expect("legal"));
    }
    let breakdown = run.last_score().expect("scored").clone();
    assert_eq!(breakdown.multiplier, 4);
    assert!(has_line(
        &breakdown,
        &Source::Kontra,
        Lever::GameValue,
        Change::Multiply(2)
    ));
    assert!(has_line(
        &breakdown,
        &Source::Re,
        Lever::GameValue,
        Change::Multiply(2)
    ));
    assert_eq!(
        breakdown.score,
        -2 * i64::try_from(breakdown.base_value * breakdown.levels * 4).expect("small")
    );
    // Without the house rule nobody says Kontra.
    let mut run = relic_run(&content, &[]);
    fixed_deal(&mut run, &content, WEAK, WEAK_SKAT);
    let events = run
        .apply(&content, RunAction::Declare(hand_game(CLUBS)))
        .expect("declare");
    assert!(!events.iter().any(|e| matches!(e, RunEvent::Kontra { .. })));
}

#[test]
fn last_trick_seven_adds_a_level() {
    let content = content();
    let mut run = relic_run(&content, &["last_trick_seven"]);
    fixed_deal(&mut run, &content, STRONG, STRONG_SKAT);
    let (events, b) = play(&mut run, &content, hand_game(DIAMONDS));
    let last = events.iter().rev().find_map(|e| match e {
        RunEvent::Game(Event::TrickWon { trick, .. }) => Some(trick.winning_card()),
        _ => None,
    });
    assert_eq!(last.map(|c| c.to_string()).as_deref(), Some("D7"));
    assert!(has_line(
        &b,
        &relic("last_trick_seven"),
        Lever::Levels,
        Change::Add(1)
    ));
    assert_eq!(b.levels, u64::from(b.rule_levels.total()) + 1);
    // Any seven counts: the H7 takes the last trick of a hearts game.
    let mut run = relic_run(&content, &["last_trick_seven"]);
    fixed_deal(&mut run, &content, HEARTS_SEVEN, HEARTS_SEVEN_SKAT);
    let (events, b) = play(&mut run, &content, hand_game(HEARTS));
    let last = events.iter().rev().find_map(|e| match e {
        RunEvent::Game(Event::TrickWon { trick, .. }) => Some(trick.winning_card()),
        _ => None,
    });
    assert_eq!(last.map(|c| c.to_string()).as_deref(), Some("H7"));
    assert!(has_line(
        &b,
        &relic("last_trick_seven"),
        Lever::Levels,
        Change::Add(1)
    ));
    // Without a seven in the last trick nothing happens.
    let mut run = relic_run(&content, &["last_trick_seven"]);
    fixed_deal(&mut run, &content, WEAK, WEAK_SKAT);
    let (_, b) = play(&mut run, &content, hand_game(CLUBS));
    assert!(b.lines_of(&relic("last_trick_seven")).next().is_none());
}

#[test]
fn concede_ends_the_game_after_five_tricks_as_schneider() {
    let content = content();
    let mut run = relic_run(&content, &["concede"]);
    fixed_deal(&mut run, &content, STRONG, STRONG_SKAT);
    let (events, b) = play(&mut run, &content, hand_game(DIAMONDS));
    let tricks = events
        .iter()
        .filter(|e| matches!(e, RunEvent::Game(Event::TrickWon { .. })))
        .count();
    assert_eq!(tricks, 5);
    assert!(
        events
            .iter()
            .any(|e| matches!(e, RunEvent::Game(Event::Conceded { tricks: 5, .. })))
    );
    assert!(b.won && b.schneider);
    assert_eq!(b.outcome.declarer_card_points, 120);
}

#[test]
fn ouvert_bonus_adds_a_second_ouvert_level() {
    let content = content();
    let mut run = relic_run(&content, &["ouvert_bonus"]);
    fixed_deal(&mut run, &content, STRONG, STRONG_SKAT);
    let (_, b) = play(
        &mut run,
        &content,
        Declaration::new(DIAMONDS, RiskLevel::Ouvert).expect("suit or grand"),
    );
    assert!(b.won && b.schwarz);
    assert_eq!(b.rule_levels.ouvert, 1);
    assert!(has_line(
        &b,
        &relic("ouvert_bonus"),
        Lever::Levels,
        Change::Add(1)
    ));
    // A hand game gets nothing.
    let mut run = relic_run(&content, &["ouvert_bonus"]);
    fixed_deal(&mut run, &content, STRONG, STRONG_SKAT);
    let (_, b) = play(&mut run, &content, hand_game(DIAMONDS));
    assert!(b.lines.is_empty());
}

#[test]
fn skat_redraw_deals_again_once_per_series() {
    let content = content();
    let start = RunStart {
        relics: vec![RelicId::new("skat_redraw")],
        ..RunStart::default()
    };
    let (mut run, _) = Run::new_with(&content, 11, &start).expect("start");
    assert_eq!(run.redeals_left(), 1);
    let before = run.game().expect("game").game.clone();
    let events = run.apply(&content, RunAction::Redeal).expect("redeal");
    assert_eq!(events[0], RunEvent::Redealt);
    assert!(events.contains(&RunEvent::GameDealt {
        number: 1,
        forehand: before.forehand()
    }));
    let after = &run.game().expect("game").game;
    assert_ne!(after.hand(Seat::DECLARER), before.hand(Seat::DECLARER));
    assert_eq!(run.series().games_played, 0, "no game used up");
    assert_eq!(run.series().passes_left, content.run.passes);
    assert_eq!(run.redeals_left(), 0);
    assert_eq!(
        run.apply(&content, RunAction::Redeal),
        Err(RunError::NoRedealLeft)
    );
    // The next game of the same series has none left; a new series has one.
    run.apply(&content, RunAction::Pass).expect("pass");
    assert_eq!(run.redeals_left(), 0);
    run.series.redeals_used = 0;
    assert_eq!(run.redeals_left(), 1);
    // Only before the declaration, and the old action name still works.
    let action: RunAction = ron::from_str("RedrawSkat").expect("old name");
    assert_eq!(action, RunAction::Redeal);
    run.apply(
        &content,
        RunAction::Declare(Declaration::new(CLUBS, RiskLevel::Hand).expect("suit or grand")),
    )
    .expect("declare");
    assert_eq!(
        run.apply(&content, RunAction::Redeal),
        Err(RunError::WrongPhase)
    );
    // Without the house rule there is no redeal.
    let run = relic_run(&content, &[]);
    assert_eq!(run.redeals_left(), 0);
}

#[test]
fn virgin_counts_61_to_70_as_schneider() {
    let content = content();
    let run = relic_run(&content, &["virgin"]);
    let mut run = run;
    let mut events = Vec::new();
    run.deal(&content, true, &mut events).expect("deal");
    let rules = &run.game().expect("game").rules;
    assert_eq!(rules.thresholds.declarer_schneider, vec![(61, 70)]);
    let decl = Declaration::new(CLUBS, RiskLevel::Normal).expect("suit or grand");
    let mit_1 = Matadors {
        with: true,
        count: 1,
    };
    let outcome = |declarer: u32| Outcome {
        declarer_card_points: declarer,
        defender_card_points: 120 - declarer,
        declarer_tricks: 5,
        defender_tricks: 5,
    };
    let b = crate::content::score_game(decl, mit_1, &outcome(63), &rules.thresholds, Vec::new());
    assert!(b.won && b.schneider);
    assert_eq!(b.levels, 3);
    let b = crate::content::score_game(decl, mit_1, &outcome(70), &rules.thresholds, Vec::new());
    assert!(b.won && b.schneider);
    let b = crate::content::score_game(decl, mit_1, &outcome(71), &rules.thresholds, Vec::new());
    assert!(!b.schneider);
    let b = crate::content::score_game(decl, mit_1, &outcome(60), &rules.thresholds, Vec::new());
    assert!(
        !b.won && !b.schneider,
        "a lost game is never made Schneider"
    );
}

#[test]
fn old_lady_makes_every_queen_worth_five() {
    let content = content();
    let mut run = relic_run(&content, &["old_lady"]);
    fixed_deal(&mut run, &content, STRONG, STRONG_SKAT);
    let game = &run.game().expect("game").game;
    for card in game.cards_in_play() {
        if card
            .face()
            .is_some_and(|f| f.rank == crate::cards::Rank::Queen)
        {
            assert_eq!(card.card_points, 5);
        }
    }
    let (_, b) = play(&mut run, &content, hand_game(DIAMONDS));
    assert_eq!(b.outcome.declarer_card_points, 128);
    assert!(has_line(
        &b,
        &relic("old_lady"),
        Lever::CardPoints,
        Change::Add(8)
    ));
}

#[test]
fn tour_raises_the_base_value_for_every_series_won_with_games_left() {
    let content = content();
    let mut run = relic_run(&content, &["tour"]);
    run.series.games_played = 2;
    let mut events = Vec::new();
    run.series_won(&content, &mut events).expect("won");
    assert!(events.contains(&RunEvent::CounterRaised {
        counter: "tour".into(),
        value: 1
    }));
    let mut events = Vec::new();
    run.deal(&content, true, &mut events).expect("deal");
    let lines = &run.game().expect("game").lines;
    assert!(lines.contains(&ScoreLine {
        source: relic("tour"),
        lever: Lever::BaseValue,
        change: Change::Add(1),
    }));
    // A series won with the last game gives nothing.
    run.series.games_played = run.series.game_limit;
    let mut events = Vec::new();
    run.series_won(&content, &mut events).expect("won");
    assert_eq!(run.counters().get("tour"), Some(&1));
}

#[test]
fn march_through_doubles_the_game_value_of_a_won_schneider() {
    let content = content();
    let mut run = relic_run(&content, &["march_through"]);
    fixed_deal(&mut run, &content, STRONG, STRONG_SKAT);
    let (_, b) = play(&mut run, &content, hand_game(DIAMONDS));
    assert!(b.won && b.schneider);
    assert!(has_line(
        &b,
        &relic("march_through"),
        Lever::GameValue,
        Change::Multiply(2)
    ));
    assert_eq!(b.base_value, 9);
    assert_eq!(b.multiplier, 2);
    assert_eq!(b.value, 9 * b.levels * 2);
    // Schneider without Schwarz counts too: Schenken after five tricks.
    let mut run = relic_run(&content, &["march_through", "concede"]);
    fixed_deal(&mut run, &content, STRONG, STRONG_SKAT);
    let (_, b) = play(&mut run, &content, hand_game(DIAMONDS));
    assert!(b.schneider);
    assert_eq!(b.multiplier, 2);
    // A lost game gets nothing.
    let mut run = relic_run(&content, &["march_through"]);
    fixed_deal(&mut run, &content, WEAK, WEAK_SKAT);
    let (_, b) = play(&mut run, &content, hand_game(CLUBS));
    assert!(b.lines_of(&relic("march_through")).next().is_none());
}

// ---------------------------------------------------------------------------
// Card modifications
// ---------------------------------------------------------------------------

fn card_mod_run(content: &Content, card: u32, card_mod: &str) -> Run {
    let mut run = relic_run(content, &[]);
    run.deck
        .add_mod(card, CardModId::new(card_mod))
        .expect("mod");
    run
}

fn mod_source(card_mod: &str, card: u32) -> Source {
    Source::CardMod {
        card_mod: CardModId::new(card_mod),
        card,
    }
}

#[test]
fn fat_adds_four_augen_for_whoever_takes_the_card() {
    let content = content();
    // DA (id 7) is fat and stays with the declarer.
    let mut run = card_mod_run(&content, 7, "fat");
    fixed_deal(&mut run, &content, STRONG, STRONG_SKAT);
    let ace = run
        .game()
        .and_then(|a| a.game.cards_in_play().iter().find(|c| c.id == 7).copied())
        .expect("DA");
    assert_eq!(ace.card_points, 15);
    let (_, b) = play(&mut run, &content, hand_game(DIAMONDS));
    assert_eq!(b.outcome.declarer_card_points, 124);
    assert!(has_line(
        &b,
        &mod_source("fat", 7),
        Lever::CardPoints,
        Change::Add(4)
    ));
}

#[test]
fn marked_card_counts_one_rank_higher() {
    let content = content();
    // SK (id 22) is marked: it now ties with the ST and keeps the trick.
    let run = card_mod_run(&content, 22, "marked");
    let resolved = resolve_deck(&run.deck, &content, &[]);
    let find = |id: u32| *resolved.cards.iter().find(|c| c.id == id).expect("card");
    let king = find(22);
    let ten = find(19);
    let ace = find(23);
    assert_eq!(king.rank_bonus, 1);
    assert!(!crate::rules::beats(GameType::Grand, king, king, ten));
    assert!(crate::rules::beats(GameType::Grand, king, king, ace));
    // Unmarked, the ten takes the king.
    let plain = resolve_deck(&Deck::standard(), &content, &[]);
    let plain_king = plain.cards[22];
    assert!(crate::rules::beats(
        GameType::Grand,
        plain_king,
        plain_king,
        ten
    ));
}

#[test]
fn stamp_adds_three_to_the_base_value_when_its_suit_is_trump() {
    let content = content();
    let mut run = card_mod_run(&content, 7, "stamp");
    fixed_deal(&mut run, &content, STRONG, STRONG_SKAT);
    let (_, b) = play(&mut run, &content, hand_game(DIAMONDS));
    assert!(has_line(
        &b,
        &mod_source("stamp", 7),
        Lever::BaseValue,
        Change::Add(3)
    ));
    assert_eq!(b.base_value, 12);
    // In a grand the stamp does nothing.
    let mut run = card_mod_run(&content, 7, "stamp");
    fixed_deal(&mut run, &content, STRONG, STRONG_SKAT);
    run.apply(&content, RunAction::Declare(hand_game(GameType::Grand)))
        .expect("declare");
    assert!(run.game().expect("game").lines.is_empty());
}

#[test]
fn last_trick_mod_adds_a_level_when_its_card_wins_the_last_trick() {
    let content = content();
    // D7 (id 0) wins the tenth trick.
    let mut run = card_mod_run(&content, 0, "last_trick");
    fixed_deal(&mut run, &content, STRONG, STRONG_SKAT);
    let (_, b) = play(&mut run, &content, hand_game(DIAMONDS));
    assert!(has_line(
        &b,
        &mod_source("last_trick", 0),
        Lever::Levels,
        Change::Add(1)
    ));
    // On a card that does not win the last trick (DA, id 7): nothing.
    let mut run = card_mod_run(&content, 7, "last_trick");
    fixed_deal(&mut run, &content, STRONG, STRONG_SKAT);
    let (_, b) = play(&mut run, &content, hand_game(DIAMONDS));
    assert!(b.lines.is_empty());
}

#[test]
fn transparent_card_shows_its_holder() {
    let content = content();
    // HA (id 15) lies with seat 1.
    let mut run = card_mod_run(&content, 15, "transparent");
    fixed_deal(&mut run, &content, STRONG, STRONG_SKAT);
    let revealed = run.revealed_holders();
    assert_eq!(revealed.len(), 1);
    assert_eq!(revealed[0].0.id, 15);
    assert_eq!(revealed[0].1, Some(Seat::DEFENDERS[0]));
}

// ---------------------------------------------------------------------------
// Bosses
// ---------------------------------------------------------------------------

fn boss_run(content: &Content, boss: &str) -> Run {
    let mut run = relic_run(content, &[]);
    run.series.boss = Some(BossId::new(boss));
    run
}

#[test]
fn the_pusher_forces_hand_games() {
    let content = content();
    let mut run = boss_run(&content, "the_pusher");
    fixed_deal(&mut run, &content, STRONG, STRONG_SKAT);
    assert!(run.requires_hand());
    assert_eq!(
        run.apply(
            &content,
            RunAction::Declare(
                Declaration::new(DIAMONDS, RiskLevel::Normal).expect("suit or grand")
            )
        ),
        Err(RunError::HandRequired)
    );
    let (_, b) = play(&mut run, &content, hand_game(DIAMONDS));
    assert_eq!(b.rule_levels.hand, 1);
}

#[test]
fn kontra_hans_enables_kontra_without_the_house_rule() {
    let content = content();
    let mut run = boss_run(&content, "kontra_hans");
    fixed_deal(&mut run, &content, WEAK, WEAK_SKAT);
    let events = run
        .apply(&content, RunAction::Declare(hand_game(CLUBS)))
        .expect("declare");
    assert!(events.iter().any(|e| matches!(e, RunEvent::Kontra { .. })));
    assert_eq!(
        run.game().and_then(|a| a.rules.kontra_source.clone()),
        Some(Source::Boss(BossId::new("kontra_hans")))
    );
    run.apply(&content, RunAction::AnswerKontra { re: false })
        .expect("no re");
    let mut guard = 0;
    while run.last_score().is_none() && guard < 100 {
        let action = RunBot.next_action(&run, &content).expect("action");
        run.apply(&content, action).expect("legal");
        guard += 1;
    }
    let b = run.last_score().expect("scored");
    assert_eq!(b.multiplier, 2);
}

#[test]
fn grandma_turns_all_jacks_into_blanks() {
    let content = content();
    let mut run = boss_run(&content, "grandma");
    let mut events = Vec::new();
    run.deal(&content, true, &mut events).expect("deal");
    let game = &run.game().expect("game").game;
    let in_play = game.cards_in_play();
    assert_eq!(in_play.iter().filter(|c| c.is_blank()).count(), 4);
    assert!(
        !in_play
            .iter()
            .any(|c| c.face().is_some_and(|f| f.is_jack()))
    );
    // Matadors start below the missing jacks: the declarer holding the
    // trump ace is "mit 1" at least.
    let decl = Declaration::new(DIAMONDS, RiskLevel::Normal).expect("suit or grand");
    let ace = in_play.iter().copied().find(|c| c.id == 7).expect("DA");
    let m = crate::rules::matadors_in(decl.game(), &[ace], in_play);
    assert!(m.with && m.count >= 1);
    // The deck itself keeps its jacks.
    assert_eq!(run.deck().blanks(), 0);
}

#[test]
fn content_validation_rejects_misplaced_effects() {
    let load = |relics: &str| {
        Content::from_sources(&crate::content::Sources {
            relics,
            ..crate::content::Sources::embedded()
        })
    };
    let bad = r#"[(id: "x", name: "X", text: "x", rarity: Common, price: 1,
        hooks: [OnScore(then: [WinThreshold(50)])])]"#;
    assert!(load(bad).is_err());
    let bad = r#"[(id: "x", name: "X", text: "x", rarity: Common, price: 1,
        passive: [AddCardPoints(4)])]"#;
    assert!(load(bad).is_err());
    let bad = r#"[(id: "x", name: "X", text: "x", rarity: Common, price: 1,
        hooks: [OnDeal(when: Won, then: [AddLevels(1)])])]"#;
    assert!(load(bad).is_err());
}

// ---------------------------------------------------------------------------
// M4 content: house rules and figures (declarer side). The Null side is
// tested in `m4_tests`.
// ---------------------------------------------------------------------------

/// Every item added in M4 loads with German texts, a price and the
/// Spielart it is made for; every character meets it (ADR 0012).
#[test]
fn m4_content_items_load_with_texts_prices_and_pools() {
    let content = content();
    let null_only = [
        "null_eight_fifteen",
        "safety_net",
        "fat_sow",
        "glass_house",
        "tally_stick",
    ];
    let declarer_only = ["blind_eye", "high_and_dry", "schneider_bock", "tailor"];
    for id in [
        "every_bit_counts",
        "blind_eye",
        "tuition",
        "beer_mat",
        "high_and_dry",
        "schneider_bock",
        "lucky_streak",
        "tailor",
        "regular_guest",
    ]
    .iter()
    .chain(&null_only)
    {
        let def = content.relic(&RelicId::new(id)).expect(id);
        assert!(
            !def.name.is_empty() && !def.text.is_empty() && def.price > 0,
            "{id}"
        );
        assert!(def.characters.is_empty(), "{id}");
        if null_only.contains(id) {
            assert_eq!(def.focus, GameFocus::Null, "{id}");
        }
        if declarer_only.contains(id) {
            assert_eq!(def.focus, GameFocus::Trump, "{id}");
        }
    }
    assert!(content.relics.len() >= 30);
    assert!(content.card_mods.len() >= 20);
    assert!(content.encounters.len() >= 8);
    let null_mods = content
        .card_mods
        .values()
        .filter(|m| m.focus == GameFocus::Null)
        .count();
    assert!(null_mods >= 5, "{null_mods}");
    let curses: Vec<&str> = content
        .card_mods
        .values()
        .filter(|m| m.curse)
        .map(|m| m.id.as_str())
        .collect();
    assert_eq!(curses, ["battered", "beer_stain", "black_cat", "cracked"]);
    for def in content.card_mods.values() {
        assert_eq!(def.curse, def.text.starts_with("Fluch"), "{}", def.id);
    }
    for def in content.card_mods.values() {
        assert!(!def.name.is_empty() && !def.text.is_empty() && def.price > 0);
    }
}

#[test]
fn every_bit_counts_makes_sevens_and_eights_worth_one() {
    let content = content();
    let mut run = relic_run(&content, &["every_bit_counts"]);
    fixed_deal(&mut run, &content, STRONG, STRONG_SKAT);
    for card in run.game().expect("game").game.cards_in_play() {
        let Some(face) = card.face() else { continue };
        let expected = match face.rank {
            crate::cards::Rank::Seven | crate::cards::Rank::Eight => 1,
            _ => face.rank.card_points(),
        };
        assert_eq!(card.card_points, expected, "{face}");
    }
    // The declarer takes every trick and the skat: all eight cards.
    let (_, b) = play(&mut run, &content, hand_game(DIAMONDS));
    assert_eq!(b.outcome.declarer_card_points, 128);
    assert!(has_line(
        &b,
        &relic("every_bit_counts"),
        Lever::CardPoints,
        Change::Add(8)
    ));
}

/// Score a suit game with `declarer` Augen under the thresholds of the
/// game in progress.
fn score_with_run_thresholds(run: &Run, declarer: u32) -> ScoreBreakdown {
    let thresholds = &run.game().expect("game").rules.thresholds;
    let decl = Declaration::new(CLUBS, RiskLevel::Normal).expect("suit or grand");
    let mit_1 = Matadors {
        with: true,
        count: 1,
    };
    let outcome = Outcome {
        declarer_card_points: declarer,
        defender_card_points: 120 - declarer,
        declarer_tricks: 5,
        defender_tricks: 5,
    };
    crate::content::score_game(decl, mit_1, &outcome, thresholds, Vec::new())
}

#[test]
fn blind_eye_wins_from_58_augen() {
    let content = content();
    let mut run = relic_run(&content, &["blind_eye"]);
    let mut events = Vec::new();
    run.deal(&content, true, &mut events).expect("deal");
    assert_eq!(run.game().expect("game").rules.thresholds.win, 58);
    assert!(score_with_run_thresholds(&run, 58).won);
    assert!(!score_with_run_thresholds(&run, 57).won);
    // Without the house rule 58 Augen lose.
    let mut plain = relic_run(&content, &[]);
    plain.deal(&content, true, &mut events).expect("deal");
    assert!(!score_with_run_thresholds(&plain, 58).won);
}

#[test]
fn tuition_adds_two_base_value_for_every_lost_game() {
    let content = content();
    let mut run = relic_run(&content, &["tuition"]);
    fixed_deal(&mut run, &content, WEAK, WEAK_SKAT);
    let (_, lost) = play(&mut run, &content, hand_game(CLUBS));
    assert!(!lost.won);
    assert_eq!(run.counters().get("tuition"), Some(&2));
    fixed_deal(&mut run, &content, STRONG, STRONG_SKAT);
    let (_, won) = play(&mut run, &content, hand_game(DIAMONDS));
    assert!(has_line(
        &won,
        &relic("tuition"),
        Lever::BaseValue,
        Change::Add(2)
    ));
    assert_eq!(won.base_value, 11);
    // A won game does not raise the counter.
    assert_eq!(run.counters().get("tuition"), Some(&2));
}

#[test]
fn beer_mat_counts_won_hand_games() {
    let content = content();
    let mut run = relic_run(&content, &["beer_mat"]);
    // A lost hand game gives no stroke.
    fixed_deal(&mut run, &content, WEAK, WEAK_SKAT);
    play(&mut run, &content, hand_game(CLUBS));
    assert_eq!(run.counters().get("beer_mat"), None);
    fixed_deal(&mut run, &content, STRONG, STRONG_SKAT);
    let (events, b) = play(&mut run, &content, hand_game(DIAMONDS));
    assert!(b.won);
    assert!(events.contains(&RunEvent::CounterRaised {
        counter: "beer_mat".into(),
        value: 1
    }));
    let mut events = Vec::new();
    run.deal(&content, true, &mut events).expect("deal");
    assert!(run.game().expect("game").lines.contains(&ScoreLine {
        source: relic("beer_mat"),
        lever: Lever::BaseValue,
        change: Change::Add(1),
    }));
}

#[test]
fn high_and_dry_adds_a_level_when_the_defenders_are_dry_after_three_tricks() {
    let content = content();
    let mut run = relic_run(&content, &["high_and_dry"]);
    fixed_deal(&mut run, &content, STRONG, STRONG_SKAT);
    let (_, b) = play(&mut run, &content, hand_game(DIAMONDS));
    assert_eq!(b.outcome.defender_card_points, 0);
    assert!(has_line(
        &b,
        &relic("high_and_dry"),
        Lever::Levels,
        Change::Add(1)
    ));
    // The weak hand gives Augen away early: nothing.
    let mut run = relic_run(&content, &["high_and_dry"]);
    fixed_deal(&mut run, &content, WEAK, WEAK_SKAT);
    let (_, b) = play(&mut run, &content, hand_game(CLUBS));
    assert!(b.lines_of(&relic("high_and_dry")).next().is_none());
}

#[test]
fn schneider_bock_doubles_the_game_after_a_won_schneider() {
    let content = content();
    let mut run = relic_run(&content, &["schneider_bock"]);
    run.series.target = 1_000_000;
    fixed_deal(&mut run, &content, STRONG, STRONG_SKAT);
    let (events, first) = play(&mut run, &content, hand_game(DIAMONDS));
    assert!(first.won && first.schneider);
    assert!(first.lines.is_empty());
    assert!(events.contains(&RunEvent::FlagSet("schneider_bock".into())));
    fixed_deal(&mut run, &content, WEAK, WEAK_SKAT);
    let (events, second) = play(&mut run, &content, hand_game(CLUBS));
    assert!(events.contains(&RunEvent::FlagCleared("schneider_bock".into())));
    assert!(has_line(
        &second,
        &relic("schneider_bock"),
        Lever::BaseValue,
        Change::Multiply(2)
    ));
    assert_eq!(second.base_value, 24);
    // A lost game sets no Bock.
    assert!(!run.flags().contains("schneider_bock"));
}

#[test]
fn lucky_streak_doubles_from_the_third_win_in_a_row_until_a_loss() {
    let content = content();
    let mut run = relic_run(&content, &["lucky_streak"]);
    run.series.target = 1_000_000;
    let mut doubled = Vec::new();
    for _ in 0..3 {
        fixed_deal(&mut run, &content, STRONG, STRONG_SKAT);
        let (_, b) = play(&mut run, &content, hand_game(DIAMONDS));
        assert!(b.won);
        doubled.push(has_line(
            &b,
            &relic("lucky_streak"),
            Lever::GameValue,
            Change::Multiply(2),
        ));
    }
    assert_eq!(doubled, vec![false, false, true]);
    // A loss counts normally and ends the streak.
    fixed_deal(&mut run, &content, WEAK, WEAK_SKAT);
    let (_, lost) = play(&mut run, &content, hand_game(CLUBS));
    assert!(!lost.won && lost.multiplier == 1);
    assert!(!run.flags().contains("streak") && !run.flags().contains("streak_start"));
    fixed_deal(&mut run, &content, STRONG, STRONG_SKAT);
    let (_, b) = play(&mut run, &content, hand_game(DIAMONDS));
    assert_eq!(b.multiplier, 1);
}

#[test]
fn the_tailor_counts_80_to_89_augen_as_schneider() {
    let content = content();
    let mut run = relic_run(&content, &["tailor"]);
    let mut events = Vec::new();
    run.deal(&content, true, &mut events).expect("deal");
    let b = score_with_run_thresholds(&run, 85);
    assert!(b.won && b.schneider);
    assert!(!score_with_run_thresholds(&run, 79).schneider);
    let mut plain = relic_run(&content, &[]);
    plain.deal(&content, true, &mut events).expect("deal");
    assert!(!score_with_run_thresholds(&plain, 85).schneider);
}

#[test]
fn the_regular_guest_doubles_won_games_from_the_fourth_series() {
    let content = content();
    let mut run = relic_run(&content, &["regular_guest"]);
    run.series.games_played = run.series.game_limit;
    let mut events = Vec::new();
    run.series_won(&content, &mut events).expect("won");
    assert!(events.contains(&RunEvent::CounterRaised {
        counter: "regular".into(),
        value: 1
    }));
    // Three series: not yet a Stammgast.
    let mut run = relic_run(&content, &["regular_guest"]);
    run.counters.insert("regular".into(), 3);
    fixed_deal(&mut run, &content, STRONG, STRONG_SKAT);
    let (_, b) = play(&mut run, &content, hand_game(DIAMONDS));
    assert_eq!(b.multiplier, 1);
    // Four: won games count double, lost ones normally.
    let mut run = relic_run(&content, &["regular_guest"]);
    run.counters.insert("regular".into(), 4);
    fixed_deal(&mut run, &content, STRONG, STRONG_SKAT);
    let (_, b) = play(&mut run, &content, hand_game(DIAMONDS));
    assert!(has_line(
        &b,
        &relic("regular_guest"),
        Lever::GameValue,
        Change::Multiply(2)
    ));
    let mut run = relic_run(&content, &["regular_guest"]);
    run.counters.insert("regular".into(), 4);
    fixed_deal(&mut run, &content, WEAK, WEAK_SKAT);
    let (_, b) = play(&mut run, &content, hand_game(CLUBS));
    assert_eq!(b.multiplier, 1);
}

// ---------------------------------------------------------------------------
// M4 content: card modifications (declarer side)
// ---------------------------------------------------------------------------

#[test]
fn gold_rim_adds_eight_augen() {
    let content = content();
    let mut run = card_mod_run(&content, 7, "gold_rim");
    fixed_deal(&mut run, &content, STRONG, STRONG_SKAT);
    let (_, b) = play(&mut run, &content, hand_game(DIAMONDS));
    assert_eq!(b.outcome.declarer_card_points, 128);
    assert!(has_line(
        &b,
        &mod_source("gold_rim", 7),
        Lever::CardPoints,
        Change::Add(8)
    ));
}

#[test]
fn double_marked_card_counts_two_ranks_higher() {
    let content = content();
    // SK (id 22) ties with the SA (id 23): the earlier card keeps it.
    let run = card_mod_run(&content, 22, "double_marked");
    let resolved = resolve_deck(&run.deck, &content, &[]);
    let find = |id: u32| *resolved.cards.iter().find(|c| c.id == id).expect("card");
    let (king, ten, ace) = (find(22), find(19), find(23));
    assert_eq!(king.rank_bonus, 2);
    assert!(!crate::rules::beats(GameType::Grand, king, king, ten));
    assert!(!crate::rules::beats(GameType::Grand, king, king, ace));
    assert!(!crate::rules::beats(GameType::Grand, ace, ace, king));
}

/// Play the STRONG deal as a diamonds hand game with `card_mod` on the DA.
fn play_with_mod_on_trump_ace(content: &Content, card_mod: &str) -> ScoreBreakdown {
    let mut run = card_mod_run(content, 7, card_mod);
    fixed_deal(&mut run, content, STRONG, STRONG_SKAT);
    play(&mut run, content, hand_game(DIAMONDS)).1
}

/// Declare the STRONG deal with `card_mod` on the DA and return the run.
fn declared_with_mod_on_trump_ace(content: &Content, card_mod: &str, game: GameType) -> Run {
    let mut run = card_mod_run(content, 7, card_mod);
    fixed_deal(&mut run, content, STRONG, STRONG_SKAT);
    run.apply(content, RunAction::Declare(hand_game(game)))
        .expect("declare");
    run
}

#[test]
fn crown_doubles_the_base_value_when_its_suit_is_trump() {
    let content = content();
    let b = play_with_mod_on_trump_ace(&content, "crown");
    assert!(has_line(
        &b,
        &mod_source("crown", 7),
        Lever::BaseValue,
        Change::Multiply(2)
    ));
    assert_eq!(b.base_value, 18);
    let run = declared_with_mod_on_trump_ace(&content, "crown", GameType::Grand);
    assert!(run.game().expect("game").lines.is_empty());
}

#[test]
fn seal_adds_a_level_when_its_suit_is_trump() {
    let content = content();
    let b = play_with_mod_on_trump_ace(&content, "seal");
    assert!(has_line(
        &b,
        &mod_source("seal", 7),
        Lever::Levels,
        Change::Add(1)
    ));
    assert_eq!(b.levels, u64::from(b.rule_levels.total()) + 1);
    let run = declared_with_mod_on_trump_ace(&content, "seal", HEARTS);
    assert!(run.game().expect("game").lines.is_empty());
}

#[test]
fn horseshoe_lowers_the_win_threshold_when_its_suit_is_trump() {
    let content = content();
    let run = declared_with_mod_on_trump_ace(&content, "horseshoe", DIAMONDS);
    let rules = &run.game().expect("game").rules;
    assert_eq!(rules.thresholds.win, 56);
    assert_eq!(rules.win_threshold_source, Some(mod_source("horseshoe", 7)));
    let run = declared_with_mod_on_trump_ace(&content, "horseshoe", GameType::Grand);
    assert_eq!(run.game().expect("game").rules.thresholds.win, 61);
}

#[test]
fn tape_measure_makes_80_to_89_augen_schneider_when_its_suit_is_trump() {
    let content = content();
    let run = declared_with_mod_on_trump_ace(&content, "tape_measure", DIAMONDS);
    assert_eq!(
        run.game()
            .expect("game")
            .rules
            .thresholds
            .declarer_schneider,
        vec![(80, 89)]
    );
    let run = declared_with_mod_on_trump_ace(&content, "tape_measure", CLUBS);
    assert!(
        run.game()
            .expect("game")
            .rules
            .thresholds
            .declarer_schneider
            .is_empty()
    );
}

#[test]
fn lucky_penny_doubles_a_won_game_when_its_card_takes_the_last_trick() {
    let content = content();
    // D7 (id 0) wins the tenth trick.
    let mut run = card_mod_run(&content, 0, "lucky_penny");
    fixed_deal(&mut run, &content, STRONG, STRONG_SKAT);
    let (_, b) = play(&mut run, &content, hand_game(DIAMONDS));
    assert!(has_line(
        &b,
        &mod_source("lucky_penny", 0),
        Lever::GameValue,
        Change::Multiply(2)
    ));
    // On the DA (id 7), which does not: nothing.
    let b = play_with_mod_on_trump_ace(&content, "lucky_penny");
    assert_eq!(b.multiplier, 1);
}

#[test]
fn beer_stain_costs_five_augen_but_shows_the_holder() {
    let content = content();
    // HA (id 15) lies with seat 1.
    let mut run = card_mod_run(&content, 15, "beer_stain");
    fixed_deal(&mut run, &content, STRONG, STRONG_SKAT);
    let ace = run
        .game()
        .and_then(|a| a.game.cards_in_play().iter().find(|c| c.id == 15).copied())
        .expect("HA");
    assert_eq!(ace.card_points, 6);
    let revealed = run.revealed_holders();
    assert_eq!(revealed.len(), 1);
    assert_eq!(revealed[0].1, Some(Seat::DEFENDERS[0]));
    let (_, b) = play(&mut run, &content, hand_game(DIAMONDS));
    assert_eq!(b.outcome.declarer_card_points, 115);
    assert!(has_line(
        &b,
        &mod_source("beer_stain", 15),
        Lever::CardPoints,
        Change::Add(-5)
    ));
    // On a seven nothing is lost below zero.
    let run = card_mod_run(&content, 0, "beer_stain");
    let resolved = resolve_deck(&run.deck, &content, &[]);
    assert_eq!(resolved.cards[0].card_points, 0);
}

/// Attach a modification the way the run does (Stempelkissen, Ereignis)
/// and return the events.
fn attach(run: &mut Run, content: &Content, card: u32, card_mod: &str) -> Vec<RunEvent> {
    let mut events = Vec::new();
    run.attach_mod(content, card, CardModId::new(card_mod), &mut events)
        .expect("attach");
    events
}

#[test]
fn a_cracked_card_never_wins_and_pays_three_groschen() {
    let content = content();
    let run = card_mod_run(&content, 22, "cracked");
    let resolved = resolve_deck(&run.deck, &content, &[]);
    let king = resolved.cards[22];
    let seven = resolved.cards[16];
    assert!(king.never_wins);
    assert_eq!(king.card_points, 4);
    // The SK cannot even take a led S7.
    assert!(!crate::rules::beats(GameType::Grand, seven, seven, king));
    let mut run = relic_run(&content, &[]);
    let coins = run.coins();
    let events = attach(&mut run, &content, 22, "cracked");
    assert_eq!(run.coins(), coins + 3);
    assert!(events.contains(&RunEvent::CoinsGained {
        source: mod_source("cracked", 22),
        amount: 3,
        coins: coins + 3,
    }));
}

#[test]
fn black_cat_lowers_the_base_value_when_its_suit_is_trump() {
    let content = content();
    let b = play_with_mod_on_trump_ace(&content, "black_cat");
    assert!(has_line(
        &b,
        &mod_source("black_cat", 7),
        Lever::BaseValue,
        Change::Add(-3)
    ));
    assert_eq!(b.base_value, 6);
    let run = card_mod_run(&content, 7, "black_cat");
    let resolved = resolve_deck(&run.deck, &content, &[]);
    assert_eq!(resolved.cards[7].rank_bonus, 0);
    let mut run = relic_run(&content, &[]);
    let coins = run.coins();
    attach(&mut run, &content, 7, "black_cat");
    assert_eq!(run.coins(), coins + 3);
    let run = declared_with_mod_on_trump_ace(&content, "black_cat", GameType::Grand);
    assert!(run.game().expect("game").lines.is_empty());
}

// ---------------------------------------------------------------------------
// M4 content: unlocks of the new relics
// ---------------------------------------------------------------------------

#[test]
fn half_of_the_new_relics_are_locked_behind_reachable_deeds() {
    use super::meta::MetaState;
    let content = content();
    let locked = MetaState::new().locked_relics(&content);
    for id in [
        "regular_guest",
        "tuition",
        "lucky_streak",
        "schneider_bock",
        "high_and_dry",
        "beer_mat",
        "tailor",
    ] {
        assert!(locked.contains(&RelicId::new(id)), "{id}");
    }
    for id in ["every_bit_counts", "blind_eye", "tally_stick", "safety_net"] {
        assert!(!locked.contains(&RelicId::new(id)), "{id}");
    }
    // Five finished runs earn the Lehrgeld.
    let mut meta = MetaState::new();
    let lost: Vec<RunEvent> = (0..5)
        .map(|_| RunEvent::RunLost {
            section: 0,
            index: 0,
            character: Character::Declarer,
            length: RunLength::Full,
        })
        .collect();
    let earned = meta.observe(&content, &lost);
    assert!(earned.contains(&crate::content::UnlockId::new("apprenticeship")));
    assert!(
        !meta
            .locked_relics(&content)
            .contains(&RelicId::new("tuition"))
    );
}

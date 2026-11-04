//! One test per content item: load the embedded content, put the item into
//! a run and check its effect in a constructed situation.

use super::bot::RunBot;
use super::*;
use crate::cards::PlayingCard;
use crate::cards::Suit;
use crate::content::{CardModId, Factor, GameFocus};
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
    let kinds = |s: &str| -> Vec<CardKind> {
        s.split_whitespace()
            .map(|n| n.parse().expect("notation"))
            .collect()
    };
    let hands = [kinds(hands[0]), kinds(hands[1]), kinds(hands[2])];
    let skat = kinds(skat);
    let mut events = Vec::new();
    run.deal_fixed(
        content,
        [&hands[0], &hands[1], &hands[2]],
        [skat[0], skat[1]],
        Seat::DECLARER,
        &mut events,
    );
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
        "last_trick_seven",
        "concede",
        "skat_redraw",
        "virgin",
        "old_lady",
        "tour",
    ] {
        let def = content.relic(&RelicId::new(id)).expect(id);
        assert!(
            !def.name.is_empty() && !def.text.is_empty() && def.price > 0,
            "{id}"
        );
    }
    for id in [
        "points",
        "rank",
        "trump_bonus",
        "last_trick",
        "first_trick",
        "voucher_coins",
        "voucher_undead",
        "voucher_wild",
        "transparent",
        "battered",
        "stained",
        "wild_card",
        "undead",
    ] {
        assert!(content.card_mod(&CardModId::new(id)).is_some(), "{id}");
    }
    for id in ["the_pusher", "grandma", "twins", "ramscher", "black_peter"] {
        assert!(content.boss(&BossId::new(id)).is_some(), "{id}");
    }
    // ADR 0017: three bosses left the pub.
    for id in ["kontra_hans", "revolutionary", "kibitz_killer"] {
        assert!(content.boss(&BossId::new(id)).is_none(), "{id}");
    }
    for id in ["herbal", "double_grain"] {
        assert!(content.consumable(&ConsumableId::new(id)).is_some(), "{id}");
    }
    // Kneipentour curve (DESIGN 5.0.4, measured 03.11.2026).
    assert_eq!(content.run.targets[0], vec![45, 55, 65, 75, 60]);
    assert_eq!(content.run.targets[2], vec![145, 170, 195, 205, 140]);
    // Own curves of the Nullspieler and the Schneiderin; Drückeberger and
    // Bruderherz play the one of run.ron (measured 03.11.2026, DESIGN 9).
    let targets = |c: Character| content.characters[&c].targets.clone();
    let null = targets(Character::NullPlayer).expect("null targets");
    assert_eq!(null[0], vec![50, 65, 75, 85, 70]);
    assert_eq!(null[2], vec![180, 210, 245, 255, 175]);
    let tailoress = targets(Character::Tailoress).expect("tailoress targets");
    assert_eq!(tailoress[0], vec![50, 65, 75, 90, 75]);
    assert_eq!(tailoress[2], vec![165, 195, 225, 235, 160]);
    assert!(targets(Character::Shirker).is_none());
    assert!(targets(Character::HeartBrother).is_none());
    assert_eq!((content.run.game_limit, content.run.redeals), (3, 2));
}

// ---------------------------------------------------------------------------
// House rules
// ---------------------------------------------------------------------------

#[test]
fn bock_round_doubles_the_base_value_after_a_lost_game() {
    let content = crate::run::legacy_content();
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
    let content = crate::run::legacy_content();
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
    assert_eq!(breakdown.multiplier, Factor::from_int(4));
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
        -i64::try_from(breakdown.base_value * breakdown.levels * 4).expect("small")
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
    let content = crate::run::legacy_content();
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
fn skat_redraw_adds_one_einmischen_to_every_game() {
    let content = content();
    let own = content.redeals(crate::content::Character::Declarer);
    let start = RunStart {
        relics: vec![RelicId::new("skat_redraw")],
        ..RunStart::default()
    };
    let (mut run, _) = Run::new_with(&content, 11, &start).expect("start");
    assert_eq!(run.game().expect("game").rules.extra_redeals, 1);
    assert_eq!(run.redeals_left(&content), own + 1);
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
    assert_eq!(run.redeals_left(&content), own);
    for _ in 0..own {
        run.apply(&content, RunAction::Redeal).expect("redeal");
    }
    assert_eq!(run.redeals_left(&content), 0);
    assert_eq!(
        run.apply(&content, RunAction::Redeal),
        Err(RunError::NoRedealLeft)
    );
    // A new game has all of them again.
    run.series.redeals_in_game = 0;
    assert_eq!(run.redeals_left(&content), own + 1);
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
    // Without the house rule the game has the Einmischen of run.ron.
    let run = relic_run(&content, &[]);
    assert_eq!(run.redeals_left(&content), own);
    // A save from before (`SeriesRedeal`, `series_redeals`) reads the same.
    let effect: crate::content::Effect = ron::from_str("SeriesRedeal").expect("old effect");
    assert_eq!(effect, crate::content::Effect::ExtraRedeal);
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
fn old_lady_makes_the_players_queens_worth_five() {
    // Since 04.11.2026 only the player's own Ober (his edition, ADR 0015):
    // the DQ counts 5, the opponents' Ober keep 3.
    let content = content();
    let mut run = relic_run(&content, &["old_lady"]);
    fixed_deal(&mut run, &content, STRONG, STRONG_SKAT);
    let game = &run.game().expect("game").game;
    let mut queens = 0;
    for card in game.cards_in_play() {
        if card
            .face()
            .is_some_and(|f| f.rank == crate::cards::Rank::Queen)
        {
            queens += 1;
            let own = card.owner == crate::deck::Owner::Player;
            assert_eq!(card.card_points, if own { 5 } else { 3 }, "{card:?}");
        }
    }
    assert_eq!(queens, 4);
    let (_, b) = play(&mut run, &content, hand_game(DIAMONDS));
    assert_eq!(b.outcome.declarer_card_points, 122);
    assert!(has_line(
        &b,
        &relic("old_lady"),
        Lever::CardPoints,
        Change::Add(2)
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
    let content = crate::run::legacy_content();
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
    assert_eq!(b.multiplier, Factor::from_int(2));
    assert_eq!(b.value, 9 * b.levels * 2);
    // Schneider without Schwarz counts too: Schenken after five tricks.
    let mut run = relic_run(&content, &["march_through", "concede"]);
    fixed_deal(&mut run, &content, STRONG, STRONG_SKAT);
    let (_, b) = play(&mut run, &content, hand_game(DIAMONDS));
    assert!(b.schneider);
    assert_eq!(b.multiplier, Factor::from_int(2));
    // A lost game gets nothing.
    let mut run = relic_run(&content, &["march_through"]);
    fixed_deal(&mut run, &content, WEAK, WEAK_SKAT);
    let (_, b) = play(&mut run, &content, hand_game(CLUBS));
    assert!(b.lines_of(&relic("march_through")).next().is_none());
}

// ---------------------------------------------------------------------------
// Card modifications
// ---------------------------------------------------------------------------

/// A run whose deck carries `card_mod` with its first value on `card`.
pub(super) fn card_mod_run(content: &Content, card: u32, card_mod: &str) -> Run {
    let value = content
        .card_mod(&CardModId::new(card_mod))
        .map_or(1, crate::content::CardModDef::default_value);
    card_mod_run_with(content, card, card_mod, value)
}

/// A run whose deck carries `card_mod` with `value` on `card`.
pub(super) fn card_mod_run_with(content: &Content, card: u32, card_mod: &str, value: i32) -> Run {
    let mut run = relic_run(content, &[]);
    run.edition
        .add_mod(content, card, &CardModId::new(card_mod), value)
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
fn points_add_augen_for_whoever_takes_the_card() {
    let content = content();
    // DA (id 7) carries Augen +4 and stays with the declarer.
    let mut run = card_mod_run_with(&content, 7, "points", 4);
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
        &mod_source("points", 7),
        Lever::CardPoints,
        Change::Add(4)
    ));
}

#[test]
fn rank_card_becomes_the_next_higher_card() {
    let content = content();
    // SK (id 22) carries Winkel 1 (03.11.2026): it becomes the S10, keeps
    // its four Augen and ties with the printed S10, which keeps the trick
    // when it came first.
    let run = card_mod_run(&content, 22, "rank");
    let resolved = resolve_edition(&run.edition, crate::deck::Owner::Player, &content, &[]);
    let find = |id: u32| *resolved.cards.iter().find(|c| c.id == id).expect("card");
    let king = find(22);
    let ten = find(19);
    let ace = find(23);
    assert_eq!(king.kind, ten.kind, "it is the Zehn now");
    assert_eq!(king.promoted, 1);
    assert_eq!(king.card_points, 4, "its own Augen");
    assert_eq!(king.rank_bonus, 0);
    assert!(!crate::rules::beats(GameType::Grand, king, king, ten));
    assert!(!crate::rules::beats(GameType::Grand, ten, ten, king));
    assert!(crate::rules::beats(GameType::Grand, king, king, ace));
}

#[test]
fn trump_bonus_adds_to_the_base_value_when_its_suit_is_trump() {
    let content = content();
    let mut run = card_mod_run_with(&content, 7, "trump_bonus", 3);
    fixed_deal(&mut run, &content, STRONG, STRONG_SKAT);
    let (_, b) = play(&mut run, &content, hand_game(DIAMONDS));
    assert!(has_line(
        &b,
        &mod_source("trump_bonus", 7),
        Lever::BaseValue,
        Change::Add(3)
    ));
    assert_eq!(b.base_value, 12);
    // In a grand it does nothing.
    let mut run = card_mod_run(&content, 7, "trump_bonus");
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
fn transparent_card_shows_only_the_marked_copy() {
    let content = content();
    // The player's DA (id 7) is in his hand: listed with him as holder.
    let mut run = card_mod_run(&content, 7, "transparent");
    fixed_deal(&mut run, &content, STRONG, STRONG_SKAT);
    let revealed = run.revealed_holders();
    assert_eq!(revealed.len(), 1);
    assert_eq!(revealed[0].0.id, 7);
    assert_eq!(revealed[0].1, Some(Seat::DECLARER));
    // The player's HA (id 15) is marked, but seat 1 holds his own HA:
    // nothing is shown (ADR 0015, fixed 04.11.2026).
    let mut run = card_mod_run(&content, 15, "transparent");
    fixed_deal(&mut run, &content, STRONG, STRONG_SKAT);
    assert!(run.revealed_holders().is_empty());
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
fn grandma_curses_half_the_edition_for_the_pub_only() {
    let content = content();
    let mut run = boss_run(&content, "grandma");
    // Every card carries "Augen +2", so every curse takes a modification's
    // place; a special card stays.
    let augen = CardModId::new("points");
    for id in 0..32 {
        run.edition.add_mod(&content, id, &augen, 2).expect("augen");
    }
    let wild = CardModId::new("wild_card");
    run.edition.add_mod(&content, 9, &wild, 1).expect("wild");
    let before = run.edition.clone();
    let coins = run.coins();
    let mut events = Vec::new();
    run.deal(&content, true, &mut events).expect("deal");
    let laid = events
        .iter()
        .find_map(|e| match e {
            RunEvent::BossCursesLaid { boss, cards } => Some((boss.clone(), cards.clone())),
            _ => None,
        })
        .expect("curses laid");
    assert_eq!(laid.0, BossId::new("grandma"));
    assert_eq!(laid.1.len(), 16);
    let curses = ["transparent", "battered", "stained"].map(CardModId::new);
    for (id, curse) in &laid.1 {
        assert!(curses.contains(curse), "{curse}");
        let card = run.edition().get(*id).expect("card");
        assert!(card.mods.contains(curse));
        assert!(!card.mods.contains(&augen), "the curse replaces Augen +2");
        assert_eq!(card.mods.contains(&wild), *id == 9);
    }
    let untouched = before
        .cards()
        .iter()
        .filter(|c| run.edition().get(c.id) == Some(c))
        .count();
    assert_eq!(untouched, 16);
    // No Groschen for these curses, and only once per pub.
    assert_eq!(run.coins(), coins);
    let mut again = Vec::new();
    run.deal(&content, true, &mut again).expect("deal");
    assert!(
        !again
            .iter()
            .any(|e| matches!(e, RunEvent::BossCursesLaid { .. }))
    );
    // When the pub ends, the edition is exactly as before.
    let mut ended = Vec::new();
    run.end_pub(&content, &mut ended);
    assert_eq!(run.edition(), &before);
    assert!(
        ended
            .iter()
            .any(|e| matches!(e, RunEvent::BossCursesLifted { cards } if cards.len() == 16))
    );
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
    let null_only = ["null_eight_fifteen", "fat_sow"];
    let declarer_only = ["blind_eye", "tailor"];
    for id in [
        "every_bit_counts",
        "blind_eye",
        "beer_mat",
        "lucky_streak",
        "tailor",
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
    // Item review of 02.11.2026: 22 relics removed, 6 added.
    assert!(content.relics.len() >= 30);
    // Effekte v2 (DESIGN 7.2): 13 modifications, plus "Abgestuft" of the
    // Nullspieler (ADR 0015), plus Zinken v2 (03.11.2026): Blei,
    // Goldkante, Fadenkreuz, Schneiderkreide, Doppelgänger.
    assert_eq!(content.card_mods.len(), 19);
    assert!(content.encounters.len() >= 8);
    let curses: Vec<&str> = content
        .card_mods
        .values()
        .filter(|m| m.is_curse())
        .map(|m| m.id.as_str())
        .collect();
    assert_eq!(curses, ["battered", "demoted", "stained", "transparent"]);
    for def in content.card_mods.values() {
        assert_eq!(def.is_curse(), def.text.starts_with("Fluch"), "{}", def.id);
    }
    for def in content.card_mods.values() {
        assert!(!def.name.is_empty() && !def.text.is_empty() && def.price > 0);
    }
}

#[test]
fn every_bit_counts_makes_the_players_sevens_and_eights_worth_one() {
    let content = content();
    let mut run = relic_run(&content, &["every_bit_counts"]);
    fixed_deal(&mut run, &content, STRONG, STRONG_SKAT);
    for card in run.game().expect("game").game.cards_in_play() {
        let Some(face) = card.face() else { continue };
        let own = card.owner == crate::deck::Owner::Player;
        let expected = match face.rank {
            crate::cards::Rank::Seven | crate::cards::Rank::Eight if own => 1,
            _ => face.rank.card_points(),
        };
        assert_eq!(card.card_points, expected, "{face}");
    }
    // The declarer takes every trick and the skat; his own D7, D8 and C7
    // count 1 each, the opponents' sevens and eights nothing (04.11.2026).
    let (_, b) = play(&mut run, &content, hand_game(DIAMONDS));
    assert_eq!(b.outcome.declarer_card_points, 123);
    assert!(has_line(
        &b,
        &relic("every_bit_counts"),
        Lever::CardPoints,
        Change::Add(3)
    ));
}

#[test]
fn an_own_queen_keeps_its_augen_in_the_opponents_trick() {
    // A player's Ober taken by an opponent still counts 5 (like an
    // Aufdruck); an opponent's Ober taken by the player counts 3.
    let content = content();
    let mut run = relic_run(&content, &["old_lady"]);
    // The player holds the HQ, the left opponent the SQ.
    let hands = [
        "CJ SJ HJ DJ DA DT DK HQ D9 D7",
        "HA HT HK DQ H9 H8 H7 SA ST SK",
        "SQ S9 S8 S7 CA CT CK CQ C9 C8",
    ];
    fixed_deal(&mut run, &content, hands, STRONG_SKAT);
    let game = &run.game().expect("game").game;
    let points = |s: &str| {
        let face: crate::cards::Card = s.parse().expect("card");
        game.cards_in_play()
            .iter()
            .find(|c| c.face() == Some(face))
            .map(|c| (c.owner, c.card_points))
    };
    assert_eq!(points("HQ"), Some((crate::deck::Owner::Player, 5)));
    assert_ne!(points("DQ").map(|p| p.0), Some(crate::deck::Owner::Player));
    assert_eq!(points("DQ").map(|p| p.1), Some(3));
}

#[test]
fn fat_sow_makes_only_the_players_aces_worth_sixteen_in_null() {
    let content = content();
    let mut run = relic_run(&content, &["fat_sow"]);
    fixed_deal(&mut run, &content, STRONG, STRONG_SKAT);
    run.apply(
        &content,
        RunAction::Declare(Declaration::null(crate::rules::NullRisk::Hand)),
    )
    .expect("Null Hand");
    let game = &run.game().expect("game").game;
    let mut aces = 0;
    for card in game.cards_in_play() {
        if card
            .face()
            .is_some_and(|f| f.rank == crate::cards::Rank::Ace)
        {
            aces += 1;
            let own = card.owner == crate::deck::Owner::Player;
            assert_eq!(card.card_points, if own { 16 } else { 11 }, "{card:?}");
        }
    }
    assert_eq!(aces, 4);
}

#[test]
fn boss_card_points_still_count_for_every_edition() {
    use crate::content::{ActiveSource, BossId, CardFilter, owns_card_points, resolve_edition};
    let content = content();
    let edition = Edition::standard();
    let set = [Effect::SetCardPoints {
        filter: CardFilter {
            rank: Some(crate::cards::Rank::Seven),
            ..CardFilter::default()
        },
        points: 4,
    }];
    let source = |source: Source| ActiveSource {
        source,
        passive: &set,
        hooks: &[],
        this_card: None,
        scale: 1,
    };
    let sevens = |owner: crate::deck::Owner, global: &[ActiveSource<'_>]| -> Vec<u32> {
        resolve_edition(&edition, owner, &content, global)
            .cards
            .iter()
            .filter(|c| {
                c.face()
                    .is_some_and(|f| f.rank == crate::cards::Rank::Seven)
            })
            .map(|c| c.card_points)
            .collect()
    };
    let boss = [source(Source::Boss(BossId::new("the_pusher")))];
    let relic = [source(Source::Relic(RelicId::new("every_bit_counts")))];
    for owner in [
        crate::deck::Owner::Player,
        crate::deck::Owner::Guest,
        crate::deck::Owner::Boss,
    ] {
        assert_eq!(sevens(owner, &boss), vec![4; 4], "{owner:?}");
        let own = owner == crate::deck::Owner::Player;
        assert_eq!(sevens(owner, &relic), vec![if own { 4 } else { 0 }; 4]);
        assert_eq!(
            owns_card_points(&Source::Relic(RelicId::new("x")), owner),
            own
        );
    }
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
    let content = crate::run::legacy_content();
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
    let content = crate::run::legacy_content();
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
    let content = crate::run::legacy_content();
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
    run.series.game_limit = 100;
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
    assert!(!lost.won && lost.multiplier == Factor::from_int(1));
    assert!(!run.flags().contains("streak") && !run.flags().contains("streak_start"));
    fixed_deal(&mut run, &content, STRONG, STRONG_SKAT);
    let (_, b) = play(&mut run, &content, hand_game(DIAMONDS));
    assert_eq!(b.multiplier, Factor::from_int(1));
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
    let content = crate::run::legacy_content();
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
    assert_eq!(b.multiplier, Factor::from_int(1));
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
    assert_eq!(b.multiplier, Factor::from_int(1));
}

// ---------------------------------------------------------------------------
// M4 content: card modifications (declarer side)
// ---------------------------------------------------------------------------

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

/// Attach a modification the way the run does (Stempelkissen, Ereignis)
/// and return the events.
pub(super) fn attach(run: &mut Run, content: &Content, card: u32, card_mod: &str) -> Vec<RunEvent> {
    let id = CardModId::new(card_mod);
    let value = content
        .card_mod(&id)
        .map_or(1, crate::content::CardModDef::default_value);
    let mut events = Vec::new();
    run.attach_mod(content, card, &id, value, &mut events)
        .expect("attach");
    events
}

#[test]
fn stained_lowers_the_base_value_when_its_suit_is_trump() {
    let content = content();
    let b = play_with_mod_on_trump_ace(&content, "stained");
    assert!(has_line(
        &b,
        &mod_source("stained", 7),
        Lever::BaseValue,
        Change::Add(-3)
    ));
    assert_eq!(b.base_value, 6);
    let run = card_mod_run(&content, 7, "stained");
    let resolved = resolve_edition(&run.edition, crate::deck::Owner::Player, &content, &[]);
    assert_eq!(resolved.cards[7].rank_bonus, 0);
    let mut run = relic_run(&content, &[]);
    let coins = run.coins();
    attach(&mut run, &content, 7, "stained");
    assert_eq!(run.coins(), coins + 3);
    let run = declared_with_mod_on_trump_ace(&content, "stained", GameType::Grand);
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
    // Since the item review of 02.11.2026 rewards of removed relics point
    // to kept or new ones (unlocks.ron).
    for id in [
        "season_ticket",
        "dive_expert",
        "lucky_streak",
        "blank_king",
        "home_suit",
        "beer_mat",
        "tailor",
    ] {
        assert!(locked.contains(&RelicId::new(id)), "{id}");
    }
    for id in ["every_bit_counts", "blind_eye", "closing_time", "fat_sow"] {
        assert!(!locked.contains(&RelicId::new(id)), "{id}");
    }
    // Five finished runs earn the Spelunkenkenner.
    let mut meta = MetaState::new();
    let lost: Vec<RunEvent> = (0..5)
        .map(|_| RunEvent::RunLost {
            section: 0,
            index: 0,
            character: Character::Declarer,
            length: RunLength::Full,
            stake: 0,
            boss: None,
        })
        .collect();
    let earned = meta.observe(&content, &lost);
    assert!(earned.contains(&crate::content::UnlockId::new("apprenticeship")));
    assert!(
        !meta
            .locked_relics(&content)
            .contains(&RelicId::new("dive_expert"))
    );
}

/// The status of `card_mod` on card `id` in the game in progress.
fn mod_status_of(
    run: &Run,
    content: &Content,
    id: u32,
    card_mod: &str,
) -> Option<crate::content::CardModStatus> {
    let active = run.game().expect("game");
    let card = *active
        .game
        .cards_in_play()
        .iter()
        .find(|c| c.id == id)
        .expect("card in play");
    let ctx = crate::content::HookContext {
        flags: &run.flags,
        counters: &run.counters,
        game: Some(&active.game),
        played: None,
        result: None,
        games_left: 0,
        transport: None,
    };
    let def = content.card_mod(&CardModId::new(card_mod)).expect("mod");
    crate::content::card_mod_status(def, &ctx, card)
}

#[test]
fn the_view_tells_whether_a_trump_mod_works_in_this_game() {
    use crate::content::CardModStatus;
    let content = content();
    let hand_card = |run: &Run, id: u32| {
        run.view(&content)
            .game
            .expect("game")
            .hand
            .into_iter()
            .find(|c| c.card.id == id)
            .expect("in hand")
    };
    // DA (id 7) with Trumpf +3: before the declaration nothing is known.
    let mut run = card_mod_run(&content, 7, "trump_bonus");
    fixed_deal(&mut run, &content, STRONG, STRONG_SKAT);
    assert!(hand_card(&run, 7).mod_status.is_empty());
    let mut grand = run.clone();
    run.apply(&content, RunAction::Declare(hand_game(DIAMONDS)))
        .expect("declare");
    let ace = hand_card(&run, 7);
    assert_eq!(
        ace.mod_status.into_iter().collect::<Vec<_>>(),
        vec![(CardModId::new("trump_bonus"), CardModStatus::Active)]
    );
    grand
        .apply(&content, RunAction::Declare(hand_game(GameType::Grand)))
        .expect("declare");
    assert_eq!(
        hand_card(&grand, 7)
            .mod_status
            .get(&CardModId::new("trump_bonus")),
        Some(&CardModStatus::Inactive)
    );
    // Every trump-suit modification follows the declaration.
    for id in ["trump_bonus", "stained"] {
        let run = declared_with_mod_on_trump_ace(&content, id, DIAMONDS);
        assert_eq!(
            mod_status_of(&run, &content, 7, id),
            Some(CardModStatus::Active),
            "{id}"
        );
        let run = declared_with_mod_on_trump_ace(&content, id, HEARTS);
        assert_eq!(
            mod_status_of(&run, &content, 7, id),
            Some(CardModStatus::Inactive),
            "{id}"
        );
    }
    // Unconditional modifications have no status.
    let run = declared_with_mod_on_trump_ace(&content, "rank", DIAMONDS);
    assert_eq!(mod_status_of(&run, &content, 7, "rank"), None);
}

#[test]
fn a_last_trick_mod_is_open_until_its_card_falls() {
    use crate::content::CardModStatus;
    let content = content();
    // "Letzter Stich" on the DA (id 7): open after the declaration, out
    // once the DA fell in an earlier trick.
    let mut run = declared_with_mod_on_trump_ace(&content, "last_trick", DIAMONDS);
    assert_eq!(
        mod_status_of(&run, &content, 7, "last_trick"),
        Some(CardModStatus::Open)
    );
    loop {
        let fell = run.game().expect("game").game.tricks();
        if fell.iter().any(|t| t.cards.iter().any(|c| c.id == 7)) {
            assert!(fell.len() < 10, "the DA does not take the last trick");
            break;
        }
        let action = RunBot.next_action(&run, &content).expect("game continues");
        run.apply(&content, action).expect("legal");
    }
    assert_eq!(
        mod_status_of(&run, &content, 7, "last_trick"),
        Some(CardModStatus::Inactive)
    );
}

/// Whatever gives a life back says how many lives one can hold at most
/// (playtest 04.11.2026): the head shows no empty hearts above the start,
/// so the maximum is told only where a life is given.
#[test]
fn every_consumable_giving_lives_names_the_maximum() {
    let content = content();
    let max = format!("höchstens {}", content.run.lives.max);
    let mut seen = 0;
    for def in content.consumables.values() {
        if matches!(def.action, crate::content::ConsumableAction::GainLives(_)) {
            seen += 1;
            assert!(
                def.text.contains(&max),
                "{}: \"{}\" does not say \"{max}\"",
                def.id,
                def.text
            );
        }
    }
    assert!(seen > 0, "no consumable gives a life");
}

/// Since 04.11.2026 every lost game counts once against the pub's account
/// (`RUN_LOSS_FACTOR`), for every character, in a normal pub and under
/// every boss rule (the Ramscher's Ramsch: `m4_tests`), with Kontra and Re
/// (`kontra_doubles_and_re_quadruples`) and in the Null (`m4_tests`).
#[test]
fn a_lost_game_counts_once_for_every_character_and_boss() {
    let content = content();
    let mut bosses: Vec<Option<BossId>> = vec![None];
    bosses.extend(
        content
            .bosses
            .keys()
            .filter(|b| b.as_str() != "ramscher")
            .cloned()
            .map(Some),
    );
    for character in crate::content::Character::ALL {
        let start = RunStart {
            character,
            ..RunStart::default()
        };
        for boss in &bosses {
            let (mut run, _) = Run::new_with(&content, 11, &start).expect("start");
            run.series.boss = boss.clone();
            fixed_deal(&mut run, &content, WEAK, WEAK_SKAT);
            let before = run.series.score;
            let (_, b) = play(&mut run, &content, hand_game(CLUBS));
            let name = format!("{character:?} {boss:?}");
            assert!(!b.won, "{name}");
            assert!(
                b.lines.iter().all(|l| l.lever != Lever::LossFactor),
                "{name}"
            );
            let value = i64::try_from(b.value).expect("small");
            assert_eq!(b.score, -value, "{name}");
            assert_eq!(run.series.score, before - value, "{name}");
        }
    }
}

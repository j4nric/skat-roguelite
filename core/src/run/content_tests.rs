//! One test per content item: load the embedded content, put the item into
//! a run and check its effect in a constructed situation.

use super::bot::RunBot;
use super::*;
use crate::cards::PlayingCard;
use crate::cards::Suit;
use crate::content::CardModId;
use crate::rules::{GameType, Matadors, Outcome, RiskLevel};

fn content() -> Content {
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

fn relic_run(content: &Content, relics: &[&str]) -> Run {
    let (mut run, _) = Run::new(content, 11).expect("start");
    run.relics = relics.iter().map(|r| RelicId::new(r)).collect();
    run
}

/// Replace the current game by a constructed deal (declarer is forehand),
/// with all content effects of a normal deal.
fn fixed_deal(run: &mut Run, content: &Content, hands: [&str; 3], skat: &str) -> Vec<RunEvent> {
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

/// Declare and let the bot play until the game is scored.
fn play(
    run: &mut Run,
    content: &Content,
    declaration: Declaration,
) -> (Vec<RunEvent>, ScoreBreakdown) {
    let mut events = run
        .apply(content, RunAction::Declare(declaration))
        .expect("declare");
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
    Declaration::new(game, RiskLevel::Hand)
}

const DIAMONDS: GameType = GameType::Suit(Suit::Diamonds);
const CLUBS: GameType = GameType::Suit(Suit::Clubs);

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
    assert_eq!(content.run.targets[0], (60, 100, 150));
    assert_eq!(content.run.targets[4], (8000, 12000, 20000));
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
    // Without the D7 in the last trick nothing happens.
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
        Declaration::new(DIAMONDS, RiskLevel::Ouvert),
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
fn skat_redraw_deals_the_other_cards_again_once() {
    let content = content();
    let mut run = relic_run(&content, &["skat_redraw"]);
    fixed_deal(&mut run, &content, STRONG, STRONG_SKAT);
    assert_eq!(run.redraws_left(), 1);
    let before = run.game().expect("game").game.clone();
    let events = run.apply(&content, RunAction::RedrawSkat).expect("redraw");
    assert!(matches!(
        events[0],
        RunEvent::Game(Event::SkatRedrawn { .. })
    ));
    let after = &run.game().expect("game").game;
    assert_eq!(after.hand(Seat::DECLARER), before.hand(Seat::DECLARER));
    let others = |g: &Game<DealtCard>| {
        let mut cards: Vec<DealtCard> = g.hand(Seat::DEFENDERS[0]).to_vec();
        cards.extend(g.hand(Seat::DEFENDERS[1]));
        cards.extend(g.skat());
        cards.sort();
        cards
    };
    assert_eq!(others(after), others(&before));
    assert_ne!(after.skat(), before.skat());
    assert_eq!(
        run.apply(&content, RunAction::RedrawSkat),
        Err(RunError::NoRedrawLeft)
    );
}

#[test]
fn virgin_counts_61_to_65_as_schneider() {
    let content = content();
    let run = relic_run(&content, &["virgin"]);
    let mut run = run;
    let mut events = Vec::new();
    run.deal(&content, true, &mut events).expect("deal");
    let rules = &run.game().expect("game").rules;
    assert_eq!(rules.thresholds.declarer_schneider, vec![(61, 65)]);
    let decl = Declaration::new(CLUBS, RiskLevel::Normal);
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
    let b = score_game(decl, mit_1, &outcome(63), &rules.thresholds, Vec::new());
    assert!(b.won && b.schneider);
    assert_eq!(b.levels, 3);
    let b = score_game(decl, mit_1, &outcome(66), &rules.thresholds, Vec::new());
    assert!(!b.schneider);
    let b = score_game(decl, mit_1, &outcome(60), &rules.thresholds, Vec::new());
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
fn march_through_triples_the_base_value_when_all_tricks_are_won() {
    let content = content();
    let mut run = relic_run(&content, &["march_through"]);
    fixed_deal(&mut run, &content, STRONG, STRONG_SKAT);
    let (_, b) = play(&mut run, &content, hand_game(DIAMONDS));
    assert!(has_line(
        &b,
        &relic("march_through"),
        Lever::BaseValue,
        Change::Multiply(3)
    ));
    assert_eq!(b.base_value, 27);
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
            RunAction::Declare(Declaration::new(DIAMONDS, RiskLevel::Normal))
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
    let decl = Declaration::new(DIAMONDS, RiskLevel::Normal);
    let ace = in_play.iter().copied().find(|c| c.id == 7).expect("DA");
    let m = crate::rules::matadors_in(decl.game, &[ace], in_play);
    assert!(m.with && m.count >= 1);
    // The deck itself keeps its jacks.
    assert_eq!(run.deck().blanks(), 0);
}

#[test]
fn content_validation_rejects_misplaced_effects() {
    let [_, card_mods, bosses, consumables, run] = crate::content::SOURCES;
    let bad = r#"[(id: "x", name: "X", text: "x", rarity: Common, price: 1,
        hooks: [OnScore(then: [WinThreshold(50)])])]"#;
    assert!(Content::from_sources(bad, card_mods.1, bosses.1, consumables.1, run.1).is_err());
    let bad = r#"[(id: "x", name: "X", text: "x", rarity: Common, price: 1,
        passive: [AddCardPoints(4)])]"#;
    assert!(Content::from_sources(bad, card_mods.1, bosses.1, consumables.1, run.1).is_err());
    let bad = r#"[(id: "x", name: "X", text: "x", rarity: Common, price: 1,
        hooks: [OnDeal(when: Won, then: [AddLevels(1)])])]"#;
    assert!(Content::from_sources(bad, card_mods.1, bosses.1, consumables.1, run.1).is_err());
}

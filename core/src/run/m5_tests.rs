//! Tests of the M5 core gaps: per-seat Augen (Ramsch), curses with
//! Groschen, "Ramponiert", "Bube ehrenhalber", several skat swaps, Null
//! style and Spitzen conditions, the origin of revealed cards, encounter
//! options that name what they give, and rewards in the meta view.

use super::bot::RunBot;
use super::content_tests::{content, fixed_deal, play, relic_run};
use super::meta::{MetaState, RewardKind};
use super::player::{PlayerGameEvent, player_events};
use super::view::RevealSource;
use super::*;
use crate::cards::{Card, Rank, Suit};
use crate::content::{Condition, EncounterId, Factor, Reward, UnlockId};
use crate::game::GameError;
use crate::rules::{NullRisk, RiskLevel};

/// Instance id of a printed card in the standard deck (D7 = 0 … CA = 31).
fn id(notation: &str) -> u32 {
    let card: Card = notation.parse().expect("card");
    let suit = Suit::ALL
        .iter()
        .position(|&s| s == card.suit)
        .expect("suit");
    let rank = Rank::ALL
        .iter()
        .position(|&r| r == card.rank)
        .expect("rank");
    u32::try_from(suit * 8 + rank).expect("id")
}

fn add_mod(run: &mut Run, notation: &str, card_mod: &str) {
    run.deck
        .add_mod(id(notation), CardModId::new(card_mod))
        .expect("mod");
}

/// The declarer holds CJ, HJ and the diamonds, SJ and HA lie in the skat:
/// mit 3 in a diamonds game.
const MIT_3: [&str; 3] = [
    "CJ HJ DA DT DK DQ D9 D8 D7 H7",
    "HT HK HQ H9 H8 SA ST SK SQ S9",
    "S8 S7 CA CT CK CQ C9 C8 C7 DJ",
];
const MIT_3_SKAT: &str = "SJ HA";

/// A jack and an ace in the skat: one swap for "Die Oma", one for "Der
/// Opa".
const JACK_AND_ACE: [&str; 3] = [
    "SJ HJ DA DT DK DQ D9 D8 D7 H7",
    "HT HK HQ H9 H8 SA ST SK SQ S9",
    "S8 S7 CA CT CK CQ C9 C8 C7 DJ",
];
const JACK_AND_ACE_SKAT: &str = "CJ HA";

/// As in the content tests: the declarer takes every trick.
const STRONG: [&str; 3] = [
    "CJ SJ HJ DJ DA DT DK DQ D9 D7",
    "HA HT HK HQ H9 H8 H7 SA ST SK",
    "SQ S9 S8 S7 CA CT CK CQ C9 C8",
];
const STRONG_SKAT: &str = "D8 C7";

/// A Null deal the declarer cannot lose (all eight sevens).
const SAFE_NULL: [&str; 3] = [
    "D7 D7 H7 H7 S7 S7 C7 C7 D8 H8",
    "D9 DT DQ DK DA H9 HT HQ HK HA",
    "S8 S9 ST SQ SK SA C8 C9 CT CQ",
];
const SAFE_NULL_SKAT: &str = "CK CA";

fn null_run(content: &Content, relics: &[&str]) -> Run {
    let start = RunStart {
        character: Character::NullPlayer,
        ..RunStart::default()
    };
    let (mut run, _) = Run::new_with(content, 11, &start).expect("null run");
    run.relics = relics.iter().map(|r| RelicId::new(r)).collect();
    run
}

fn has_relic_line(b: &ScoreBreakdown, relic: &str, lever: Lever, change: Change) -> bool {
    b.lines_of(&Source::Relic(RelicId::new(relic)))
        .any(|l| l.lever == lever && l.change == change)
}

// ---------------------------------------------------------------------------
// 1. Ramsch: Augen and tricks per seat
// ---------------------------------------------------------------------------

#[test]
fn a_ramsch_reports_augen_and_tricks_per_seat() {
    let content = content();
    let mut run = relic_run(&content, &[]);
    run.series.boss = Some(BossId::new("ramscher"));
    let mut events = Vec::new();
    run.deal(&content, true, &mut events).expect("deal");
    // A defender may lead the Ramsch: let the AI play up to the player.
    run.play_ai(&content, &mut events).expect("AI");
    assert!(events.contains(&RunEvent::RamschDealt));
    let mut seen_tricks = false;
    let breakdown = loop {
        if let Some(b) = events.iter().find_map(|e| match e {
            RunEvent::GameScored { breakdown, .. } => Some(breakdown.clone()),
            _ => None,
        }) {
            break b;
        }
        if let Some(game) = run.view(&content).game {
            let tricks: u32 = game.seat_tricks.iter().sum();
            assert_eq!(tricks as usize, game.tricks_played);
            seen_tricks |= tricks > 0;
        }
        let action = RunBot.next_action(&run, &content).expect("play");
        events.extend(run.apply(&content, action).expect("legal"));
    };
    assert!(seen_tricks);
    assert!(breakdown.declaration.is_ramsch());
    let seats = breakdown.seats.expect("seats");
    assert_eq!(seats.card_points.iter().sum::<u32>(), 120);
    assert_eq!(seats.tricks.iter().sum::<u32>(), 10);
    let most = seats.card_points.iter().copied().max().expect("max");
    assert_eq!(breakdown.won, seats.card_points[0] < most);
    // The player sees the same in the finished game and in the last score.
    let finished = player_events(events).into_iter().find_map(|e| match e {
        RunEvent::Game(PlayerGameEvent::GameFinished { seats, .. }) => Some(seats),
        _ => None,
    });
    assert_eq!(finished, Some(seats));
    let view = run.view(&content);
    assert_eq!(view.last_score.expect("score").breakdown.seats, Some(seats));
}

#[test]
fn the_counter_shows_augen_per_seat() {
    let content = content();
    let mut run = relic_run(&content, &["card_counter"]);
    fixed_deal(&mut run, &content, STRONG, STRONG_SKAT);
    run.apply(
        &content,
        RunAction::Declare(Declaration::suit(Suit::Diamonds, RiskLevel::Hand)),
    )
    .expect("declare");
    for _ in 0..3 {
        let action = RunBot.next_action(&run, &content).expect("play");
        run.apply(&content, action).expect("play");
    }
    let count = run
        .view(&content)
        .game
        .and_then(|g| g.count)
        .expect("count");
    assert_eq!(count.seats[0], count.declarer);
    assert_eq!(count.seats[1] + count.seats[2], count.defenders);
}

// ---------------------------------------------------------------------------
// 2, 7, 8, 12: curses, Groschen, "Ramponiert"
// ---------------------------------------------------------------------------

#[test]
fn gain_coins_is_allowed_outside_passive_and_never_drops_below_zero() {
    for stage in [
        Stage::OnDeal,
        Stage::OnScore,
        Stage::OnSeriesWon,
        Stage::OnAttach,
    ] {
        assert!(Effect::GainCoins(1).allowed_in(stage), "{stage:?}");
    }
    assert!(!Effect::GainCoins(1).allowed_in(Stage::Passive));
    assert!(!Effect::AddLevels(1).allowed_in(Stage::OnAttach));
    let content = content();
    let mut run = relic_run(&content, &[]);
    run.coins = 2;
    let source = Source::Relic(RelicId::new("x"));
    let mut events = Vec::new();
    run.apply_effect(&source, &Effect::GainCoins(-5), &mut events);
    assert_eq!(run.coins(), 0);
    run.apply_effect(&source, &Effect::GainCoins(4), &mut events);
    assert_eq!(run.coins(), 4);
    assert_eq!(
        events,
        vec![
            RunEvent::CoinsGained {
                source: source.clone(),
                amount: -5,
                coins: 0
            },
            RunEvent::CoinsGained {
                source,
                amount: 4,
                coins: 4
            },
        ]
    );
    // OnAttach only on card modifications; GainCoins after a won series
    // on a relic is fine.
    // The embedded relics plus one more.
    let load = |extra: &str| {
        let body = crate::content::Sources::embedded()
            .relics
            .trim_end()
            .trim_end_matches(']')
            .trim_end();
        let sep = if body.ends_with(',') { "" } else { "," };
        let relics = format!("{body}{sep}{extra}]");
        Content::from_sources(&crate::content::Sources {
            relics: &relics,
            ..crate::content::Sources::embedded()
        })
    };
    let bad = r#"(id: "x", name: "X", text: "x", rarity: Common, price: 1,
        hooks: [OnAttach(then: [GainCoins(1)])])"#;
    assert!(load(bad).is_err());
    let good = r#"(id: "x", name: "X", text: "x", rarity: Common, price: 1,
        hooks: [OnSeriesWon(then: [GainCoins(2)]), OnScore(when: Won, then: [GainCoins(1)])])"#;
    assert!(load(good).is_ok(), "{:?}", load(good));
}

#[test]
fn battered_pays_when_attached_and_stays_out_of_the_skat() {
    let content = content();
    // The ink pad (Stempelkissen) names no mod: give the run one that does.
    let mut run = relic_run(&content, &[]);
    run.game = None;
    run.phase = RunPhase::Shop;
    run.shop = Some(run.draw_offer(&content, 0));
    let coins = run.coins();
    let mut events = Vec::new();
    run.attach_mod(&content, id("S7"), CardModId::new("battered"), &mut events)
        .expect("attach");
    assert_eq!(run.coins(), coins + 3);
    assert!(matches!(events[0], RunEvent::DeckChanged(_)));
    assert!(matches!(events[1], RunEvent::CoinsGained { amount: 3, .. }));

    // In a game: the D7 in the hand and the C7 in the skat are battered.
    let mut run = relic_run(&content, &[]);
    add_mod(&mut run, "D7", "battered");
    add_mod(&mut run, "C7", "battered");
    fixed_deal(&mut run, &content, STRONG, STRONG_SKAT);
    run.apply(
        &content,
        RunAction::Declare(Declaration::suit(Suit::Diamonds, RiskLevel::Normal)),
    )
    .expect("declare");
    let view = run.view(&content).game.expect("game");
    let flagged: Vec<u32> = view
        .hand
        .iter()
        .filter(|c| c.card.no_discard)
        .map(|c| c.card.id)
        .collect();
    assert_eq!(flagged, vec![id("D7"), id("C7")], "trumps first");
    for pair in [[id("D7"), id("D9")], [id("C7"), id("D8")]] {
        let err = run
            .clone()
            .apply(&content, RunAction::Discard(pair))
            .expect_err("battered");
        assert!(matches!(
            err,
            RunError::Game(GameError::DiscardForbidden { .. })
        ));
        assert_eq!(err.code(), RunErrorCode::DiscardForbidden);
    }
    assert!(
        run.clone()
            .apply(&content, RunAction::Discard([id("D8"), id("D9")]))
            .is_ok()
    );
    // The bot never pushes a battered card while it has others.
    let action = RunBot.next_action(&run, &content).expect("discard");
    assert!(matches!(action, RunAction::Discard(_)), "{action:?}");
    run.apply(&content, action).expect("bot discard");
    let skat = run.game().expect("game").game.skat();
    assert!(skat.iter().all(|c| !c.no_discard), "{skat:?}");
    assert_eq!(RunErrorCode::DiscardForbidden.as_str(), "DiscardForbidden");
}

#[test]
fn the_grannys_swap_cannot_push_a_battered_card() {
    let content = content();
    let mut run = relic_run(&content, &["granny"]);
    add_mod(&mut run, "H7", "battered");
    fixed_deal(&mut run, &content, JACK_AND_ACE, JACK_AND_ACE_SKAT);
    run.apply(
        &content,
        RunAction::Declare(Declaration::suit(Suit::Diamonds, RiskLevel::Hand)),
    )
    .expect("declare");
    let err = run
        .clone()
        .apply(
            &content,
            RunAction::TakeFromSkat {
                give: id("H7"),
                take: None,
            },
        )
        .expect_err("battered");
    assert_eq!(err.code(), RunErrorCode::DiscardForbidden);
    run.apply(
        &content,
        RunAction::TakeFromSkat {
            give: id("D7"),
            take: None,
        },
    )
    .expect("another card");
}

// ---------------------------------------------------------------------------
// 9: several skat swaps; 6: the view names the card
// ---------------------------------------------------------------------------

#[test]
fn granny_and_grandpa_swap_once_each_and_the_view_names_the_card() {
    let content = content();
    let mut run = relic_run(&content, &["granny", "grandpa"]);
    fixed_deal(&mut run, &content, JACK_AND_ACE, JACK_AND_ACE_SKAT);
    run.apply(
        &content,
        RunAction::Declare(Declaration::suit(Suit::Diamonds, RiskLevel::Hand)),
    )
    .expect("declare");
    let game = run.view(&content).game.expect("game");
    assert!(game.can_take_from_skat);
    let offered: Vec<u32> = game.skat_swap_cards.iter().map(|c| c.card.id).collect();
    assert_eq!(offered, vec![id("CJ"), id("HA")], "the jack first");
    // A card no swap takes.
    assert_eq!(
        run.clone().apply(
            &content,
            RunAction::TakeFromSkat {
                give: id("H7"),
                take: Some(id("D7")),
            },
        ),
        Err(RunError::NothingToTake)
    );
    run.apply(
        &content,
        RunAction::TakeFromSkat {
            give: id("H7"),
            take: None,
        },
    )
    .expect("the jack");
    let game = run.view(&content).game.expect("game");
    let offered: Vec<u32> = game.skat_swap_cards.iter().map(|c| c.card.id).collect();
    assert_eq!(offered, vec![id("HA")], "only the Opa is left");
    run.apply(
        &content,
        RunAction::TakeFromSkat {
            give: id("D7"),
            take: Some(id("HA")),
        },
    )
    .expect("the ace");
    assert_eq!(run.series().skat_swaps_used, 2);
    assert_eq!(run.series().skat_swaps_taken.len(), 2);
    let game = run.view(&content).game.expect("game");
    assert!(!game.can_take_from_skat && game.skat_swap_cards.is_empty());
    assert_eq!(
        run.apply(
            &content,
            RunAction::TakeFromSkat {
                give: id("D8"),
                take: None,
            },
        ),
        Err(RunError::NoSkatSwapLeft)
    );
    let hand = run.game().expect("game").game.hand(Seat::DECLARER).to_vec();
    assert!(hand.iter().any(|c| c.id == id("CJ")));
    assert!(hand.iter().any(|c| c.id == id("HA")));

    // Only the Opa: the jack stays, the ace comes.
    let mut run = relic_run(&content, &["grandpa"]);
    fixed_deal(&mut run, &content, JACK_AND_ACE, JACK_AND_ACE_SKAT);
    run.apply(
        &content,
        RunAction::Declare(Declaration::suit(Suit::Diamonds, RiskLevel::Hand)),
    )
    .expect("declare");
    let game = run.view(&content).game.expect("game");
    let offered: Vec<u32> = game.skat_swap_cards.iter().map(|c| c.card.id).collect();
    assert_eq!(offered, vec![id("HA")]);
    // Saves from before the list: one swap used counts the first filter.
    let mut old = relic_run(&content, &["granny", "grandpa"]);
    fixed_deal(&mut old, &content, JACK_AND_ACE, JACK_AND_ACE_SKAT);
    old.series.skat_swaps_used = 1;
    assert_eq!(
        old.open_skat_swaps(),
        vec![CardFilter {
            suit: None,
            rank: Some(Rank::Ace)
        }]
    );
    // The list survives a save.
    let text = ron::to_string(&old).expect("ron");
    let back: Run = ron::from_str(&text).expect("ron");
    assert_eq!(back, old);
}

// ---------------------------------------------------------------------------
// 10: "Bube ehrenhalber"
// ---------------------------------------------------------------------------

#[test]
fn an_honorary_jack_is_trump_in_the_run_and_sorts_below_the_jacks() {
    let content = content();
    let mut run = relic_run(&content, &[]);
    // The declarer's SK becomes a jack of honour.
    add_mod(&mut run, "H7", "honorary_jack");
    fixed_deal(
        &mut run,
        &content,
        [
            "CJ SJ HJ DJ DA DT DK DQ D9 H7",
            "HA HT HK HQ H9 H8 D7 SA ST SK",
            "SQ S9 S8 S7 CA CT CK CQ C9 C8",
        ],
        "D8 C7",
    );
    let h7 = run
        .game()
        .expect("game")
        .game
        .hand(Seat::DECLARER)
        .iter()
        .find(|c| c.id == id("H7"))
        .copied()
        .expect("H7");
    assert!(h7.honorary_jack);
    run.apply(
        &content,
        RunAction::Declare(Declaration::suit(Suit::Diamonds, RiskLevel::Hand)),
    )
    .expect("declare");
    let game = run.view(&content).game.expect("game");
    let order: Vec<u32> = game.hand.iter().map(|c| c.card.id).collect();
    assert_eq!(
        &order[..5],
        &[id("CJ"), id("SJ"), id("HJ"), id("DJ"), id("H7")]
    );
    assert!(game.hand[4].card.honorary_jack);
    // No Spitze: CJ to DJ, DA to D9 and the D8 in the skat are mit 10
    // along the printed order (the D7 is out); the honorary H7 adds
    // nothing.
    let matadors = run.game().expect("game").game.matadors().expect("matadors");
    assert_eq!(
        matadors,
        crate::rules::Matadors {
            with: true,
            count: 10
        }
    );
    // The bot can play it through.
    let (_, b) = play_on(&mut run, &content);
    assert!(b.won);
}

/// Let the bot finish the current game.
fn play_on(run: &mut Run, content: &Content) -> (Vec<RunEvent>, ScoreBreakdown) {
    let mut events = Vec::new();
    loop {
        if let Some(b) = events.iter().find_map(|e: &RunEvent| match e {
            RunEvent::GameScored { breakdown, .. } => Some(breakdown.clone()),
            _ => None,
        }) {
            return (events, b);
        }
        let action = RunBot.next_action(run, content).expect("play");
        events.extend(run.apply(content, action).expect("legal"));
    }
}

// ---------------------------------------------------------------------------
// 11: conditions; 12: "Mit Vieren", "Siebenschläfer", "Saubere Weste"
// ---------------------------------------------------------------------------

#[test]
fn matadors_at_least_counts_mit_and_ohne() {
    let content = content();
    let cond = |with, count| Condition::MatadorsAtLeast { with, count };
    let check = |run: &Run, c: &Condition| {
        let ctx = HookContext {
            flags: &run.flags,
            counters: &run.counters,
            game: run.game.as_ref().map(|a| &a.game),
            played: None,
            result: None,
            games_left: 0,
        };
        evaluate(c, &ctx, None)
    };
    let mut run = relic_run(&content, &["with_four"]);
    fixed_deal(&mut run, &content, MIT_3, MIT_3_SKAT);
    assert!(!check(&run, &cond(true, 1)), "before the declaration");
    let (_, b) = play(
        &mut run,
        &content,
        Declaration::suit(Suit::Diamonds, RiskLevel::Normal),
    );
    assert_eq!(
        b.matadors,
        crate::rules::Matadors {
            with: true,
            count: 3
        }
    );
    assert!(!has_relic_line(
        &b,
        "with_four",
        Lever::Levels,
        Change::Add(1)
    ));

    let mut run = relic_run(&content, &["with_four"]);
    fixed_deal(&mut run, &content, MIT_3, MIT_3_SKAT);
    run.apply(
        &content,
        RunAction::Declare(Declaration::suit(Suit::Diamonds, RiskLevel::Hand)),
    )
    .expect("declare");
    assert!(check(&run, &cond(true, 3)));
    assert!(!check(&run, &cond(true, 4)));
    assert!(!check(&run, &cond(false, 1)));

    // Mit 11: one level more.
    let mut run = relic_run(&content, &["with_four"]);
    fixed_deal(&mut run, &content, STRONG, STRONG_SKAT);
    let (_, b) = play(
        &mut run,
        &content,
        Declaration::suit(Suit::Diamonds, RiskLevel::Hand),
    );
    assert!(has_relic_line(
        &b,
        "with_four",
        Lever::Levels,
        Change::Add(1)
    ));
    // Ohne 4 in a grand: all four jacks lie with seat 2.
    let mut run = relic_run(&content, &["with_four"]);
    fixed_deal(
        &mut run,
        &content,
        [
            "S8 S7 CA CT CK CQ C9 C8 C7 DA",
            "HT HK HQ H9 H8 SA ST SK SQ S9",
            "CJ SJ HJ DJ DT DK DQ D9 D8 D7",
        ],
        "H7 HA",
    );
    run.apply(
        &content,
        RunAction::Declare(Declaration::grand(RiskLevel::Normal)),
    )
    .expect("declare");
    assert!(check(&run, &cond(false, 4)));
    assert!(!check(&run, &cond(true, 1)));
}

#[test]
fn null_style_conditions_and_house_rules() {
    let content = content();
    let mut run = null_run(&content, &["seven_sleeper", "clean_slate"]);
    fixed_deal(&mut run, &content, SAFE_NULL, SAFE_NULL_SKAT);
    let (_, b) = play(&mut run, &content, Declaration::null(NullRisk::Hand));
    assert!(b.won);
    assert!(b.rule_levels.sevens_ducked > 0);
    assert!(has_relic_line(
        &b,
        "seven_sleeper",
        Lever::BaseValue,
        Change::Add(5)
    ));
    assert_eq!(
        has_relic_line(&b, "clean_slate", Lever::Levels, Change::Add(2)),
        b.rule_levels.never_followed > 0
    );
    // Outside Null both are false.
    let mut run = relic_run(&content, &[]);
    fixed_deal(&mut run, &content, STRONG, STRONG_SKAT);
    run.apply(
        &content,
        RunAction::Declare(Declaration::suit(Suit::Diamonds, RiskLevel::Hand)),
    )
    .expect("declare");
    let ctx = HookContext {
        flags: &run.flags,
        counters: &run.counters,
        game: run.game.as_ref().map(|a| &a.game),
        played: None,
        result: None,
        games_left: 0,
    };
    assert!(!evaluate(&Condition::NeverFollowedSuit, &ctx, None));
    assert!(!evaluate(&Condition::SevenDucked, &ctx, None));
    assert!(Condition::SevenDucked.allowed_in(Stage::OnScore));
    assert!(!Condition::SevenDucked.allowed_in(Stage::OnDeclare));
    assert!(
        Condition::MatadorsAtLeast {
            with: true,
            count: 4
        }
        .allowed_in(Stage::OnDeclare)
    );
}

// ---------------------------------------------------------------------------
// 3: why a card is revealed
// ---------------------------------------------------------------------------

#[test]
fn revealed_cards_name_their_source() {
    let content = content();
    let mut run = relic_run(&content, &[]);
    add_mod(&mut run, "HA", "transparent");
    fixed_deal(&mut run, &content, STRONG, STRONG_SKAT);
    let game = run.view(&content).game.expect("game");
    assert_eq!(game.revealed.len(), 1);
    assert_eq!(
        game.revealed[0].source,
        RevealSource::CardMod(CardModId::new("transparent"))
    );
    let mut run = relic_run(&content, &["kibitzer"]);
    let mut events = Vec::new();
    run.deal(&content, true, &mut events).expect("deal");
    let game = run.view(&content).game.expect("game");
    assert_eq!(game.revealed.len(), 1);
    assert_eq!(game.revealed[0].source, RevealSource::Kibitz);
}

// ---------------------------------------------------------------------------
// 4: encounter options name what they give
// ---------------------------------------------------------------------------

/// Content in which only this encounter comes, after every series.
fn only_encounter(id: &str) -> Content {
    let mut content = content();
    content.run.encounter_chance = Factor::ONE;
    content.encounters.retain(|k, _| k.as_str() == id);
    content
}

#[test]
fn the_card_dealer_names_the_modification_before_the_choice() {
    let content = only_encounter("card_dealer");
    let mut run = relic_run(&content, &[]);
    run.coins = 10;
    let mut events = Vec::new();
    assert!(run.open_encounter(&content, &mut events));
    let encounter = run.encounter().expect("open").clone();
    let named = encounter.options[0].card_mod.clone().expect("named");
    assert_eq!(encounter.options[1].card_mod.as_ref(), Some(&named));
    let view = run.view(&content).encounter.expect("view");
    let info = view.options[0].card_mod.as_ref().expect("info");
    assert_eq!(info.id, named);
    assert!(!info.name.is_empty());
    assert!(!view.options[0].random_gain);
    assert!(view.options.last().expect("leave").card_mod.is_none());
    let target = *view.options[1].targets.first().expect("target");
    run.apply(
        &content,
        RunAction::ChooseEncounter {
            option: 1,
            target: Some(target),
        },
    )
    .expect("chosen");
    assert_eq!(
        run.deck().get(target).map(|c| c.mods.clone()),
        Some(vec![named])
    );
}

#[test]
fn the_old_friend_names_the_relic_received() {
    let content = only_encounter("old_friend");
    let mut run = relic_run(&content, &["kontra", "tour"]);
    let mut events = Vec::new();
    assert!(run.open_encounter(&content, &mut events));
    let encounter = run.encounter().expect("open").clone();
    let EncounterChoice::SwapRelic { new: Some(new), .. } = &encounter.options[1].choice else {
        panic!("swap with a known relic: {:?}", encounter.options);
    };
    let view = run.view(&content).encounter.expect("view");
    assert_eq!(view.options[1].new_relic.as_ref().map(|r| &r.id), Some(new));
    assert!(!view.options[1].random_gain);
    let new = new.clone();
    let events = run
        .apply(
            &content,
            RunAction::ChooseEncounter {
                option: 1,
                target: None,
            },
        )
        .expect("swap");
    assert!(events.contains(&RunEvent::RelicSwapped {
        old: RelicId::new("tour"),
        new: new.clone(),
    }));
    assert_eq!(run.relics(), &[RelicId::new("kontra"), new]);
    // An option of an old save (no relic named) says it is random.
    let mut old = relic_run(&content, &["kontra"]);
    old.game = None;
    old.phase = RunPhase::Encounter;
    old.encounter = Some(ActiveEncounter {
        id: EncounterId::new("old_friend"),
        options: vec![EncounterOption {
            choice: EncounterChoice::SwapRelic {
                index: 0,
                relic: RelicId::new("kontra"),
                new: None,
            },
            cost: 0,
            card_mod: None,
        }],
    });
    let view = old.view(&content).encounter.expect("view");
    assert!(view.options[0].random_gain);
    assert!(view.options[0].new_relic.is_none());
}

// ---------------------------------------------------------------------------
// 5: rewards in the meta view, card modifications as rewards
// ---------------------------------------------------------------------------

#[test]
fn rewards_show_kind_and_rarity_and_card_mods_can_be_locked() {
    let mut content = content();
    let view = MetaState::new().view(&content);
    let tour_won = view
        .unlocks
        .iter()
        .find(|u| u.id == UnlockId::new("tour_won"))
        .expect("tour_won");
    assert_eq!(tour_won.reward_info[0].kind, RewardKind::Character);
    assert!(tour_won.reward_info[0].rarity.is_none());
    let granny = view
        .unlocks
        .iter()
        .flat_map(|u| &u.reward_info)
        .find(|r| r.reward == Reward::Relic(RelicId::new("granny")))
        .expect("granny");
    assert_eq!(granny.kind, RewardKind::Regular);
    assert_eq!(granny.rarity, Some(Rarity::Rare));
    assert_eq!(granny.name, "Die Oma");
    for unlock in &view.unlocks {
        assert_eq!(unlock.rewards.len(), unlock.reward_info.len());
    }
    // A card modification as a reward stays out of random draws.
    let honorary = CardModId::new("honorary_jack");
    content
        .unlocks
        .get_mut(&UnlockId::new("hand_won"))
        .expect("hand_won")
        .rewards
        .push(Reward::CardMod(honorary.clone()));
    let meta = MetaState::new();
    assert_eq!(meta.locked_card_mods(&content), vec![honorary.clone()]);
    let view = meta.view(&content);
    assert_eq!(view.locked_card_mods, vec![honorary.clone()]);
    let start = meta
        .run_start(&content, Character::Declarer)
        .expect("start");
    assert_eq!(start.locked_card_mods, vec![honorary.clone()]);
    let (run, _) = Run::new_with(&content, 1, &start).expect("run");
    assert!(!run.card_mod_pool(&content, &[]).contains(&honorary));
    assert!(
        MetaState::all_unlocked(&content)
            .locked_card_mods(&content)
            .is_empty()
    );
}

// ---------------------------------------------------------------------------
// Robustness: the bot plays whole runs with every new item in play.
// ---------------------------------------------------------------------------

#[test]
fn the_bot_plays_whole_runs_with_the_new_content() {
    let content = content();
    for seed in 0..6 {
        let start = RunStart {
            relics: ["granny", "grandpa", "with_four"]
                .iter()
                .map(|r| RelicId::new(r))
                .collect(),
            ..RunStart::default()
        };
        let (mut run, _) = Run::new_with(&content, seed, &start).expect("run");
        for (card, card_mod) in [
            ("S7", "battered"),
            ("H8", "battered"),
            ("C9", "battered"),
            ("SK", "honorary_jack"),
            ("HA", "honorary_jack"),
            ("D9", "cracked"),
        ] {
            add_mod(&mut run, card, card_mod);
        }
        for _ in 0..3_000 {
            let Some(action) = RunBot.next_action(&run, &content) else {
                break;
            };
            run.apply(&content, action).expect("bot action");
        }
        assert!(matches!(run.phase(), RunPhase::Won | RunPhase::Lost));
    }
}

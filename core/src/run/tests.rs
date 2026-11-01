use super::bot::RunBot;
use super::*;
use crate::cards::{Card, PlayingCard};
use crate::rules::NullRisk;

fn content() -> Content {
    Content::load().expect("embedded content is valid")
}

/// Drive a run with the bot. Returns all events and the number of actions.
fn play_to_end(run: &mut Run, content: &Content, max_actions: usize) -> Vec<RunEvent> {
    let mut events = Vec::new();
    for _ in 0..max_actions {
        let Some(action) = RunBot.next_action(run, content) else {
            break;
        };
        events.extend(run.apply(content, action).expect("bot actions are legal"));
    }
    events
}

#[test]
fn bot_plays_runs_to_the_end() {
    let content = content();
    for seed in 0..20 {
        let (mut run, _) = Run::new(&content, seed).expect("start");
        play_to_end(&mut run, &content, 100_000);
        assert!(
            matches!(run.phase(), RunPhase::Won | RunPhase::Lost),
            "seed {seed}: {:?}",
            run.phase()
        );
    }
}

#[test]
fn same_seed_same_actions_same_run() {
    let content = content();
    let (mut a, ea) = Run::new(&content, 7).expect("start");
    let (mut b, eb) = Run::new(&content, 7).expect("start");
    assert_eq!(ea, eb);
    let ea = play_to_end(&mut a, &content, 100_000);
    let eb = play_to_end(&mut b, &content, 100_000);
    assert_eq!(ea, eb);
    assert_eq!(a, b);
    let (mut c, _) = Run::new(&content, 8).expect("start");
    play_to_end(&mut c, &content, 100_000);
    assert_ne!(a, c);
}

#[test]
fn save_and_load_gives_the_same_result() {
    let content = content();
    for stop_after in [0, 5, 37, 150] {
        let (mut run, _) = Run::new(&content, 1234).expect("start");
        play_to_end(&mut run, &content, stop_after);
        let saved = ron::to_string(&run).expect("serialize");
        let mut loaded: Run = ron::from_str(&saved).expect("deserialize");
        assert_eq!(loaded, run, "stop after {stop_after}");
        let rest_a = play_to_end(&mut run, &content, 100_000);
        let rest_b = play_to_end(&mut loaded, &content, 100_000);
        assert_eq!(rest_a, rest_b);
        assert_eq!(run, loaded);
    }
}

#[test]
fn run_starts_with_first_series_and_a_dealt_game() {
    let content = content();
    let (run, events) = Run::new(&content, 1).expect("start");
    assert_eq!(run.phase(), RunPhase::Game);
    assert_eq!(run.series().target, content.run.targets[0].0);
    assert_eq!(run.series().game_limit, content.run.game_limit);
    assert_eq!(run.series().passes_left, content.run.passes);
    assert_eq!(run.coins(), content.run.economy.starting_coins);
    assert!(matches!(
        &events[0],
        RunEvent::BossAnnounced { section: 0, boss } if content.run.bosses.contains(boss)
    ));
    assert!(matches!(
        events[1],
        RunEvent::SeriesStarted {
            section: 0,
            index: 0,
            target,
            boss: None,
            ..
        } if target == content.run.targets[0].0
    ));
    assert!(
        matches!(events[2], RunEvent::GameDealt { number: 1, forehand } if forehand == Seat::DECLARER)
    );
    let game = &run.game().expect("dealt").game;
    assert_eq!(game.hand(Seat::DECLARER).len(), 10);
    assert_eq!(game.phase(), Phase::Declaring);
}

#[test]
fn forehand_rotates_every_game_and_passes_cost_a_game() {
    let content = content();
    let (mut run, _) = Run::new(&content, 3).expect("start");
    // Two passes, independent of the pass count in run.ron.
    run.series.passes_left = 2;
    let events = run.apply(&content, RunAction::Pass).expect("pass");
    assert!(events.contains(&RunEvent::Passed { passes_left: 1 }));
    assert!(events.contains(&RunEvent::GameDealt {
        number: 2,
        forehand: Seat::DEFENDERS[0]
    }));
    run.apply(&content, RunAction::Pass).expect("pass");
    assert_eq!(run.series().games_played, 2);
    assert_eq!(
        run.game().map(|a| a.game.forehand()),
        Some(Seat::DEFENDERS[1])
    );
    assert_eq!(
        run.apply(&content, RunAction::Pass),
        Err(RunError::NoPassesLeft)
    );
}

#[test]
fn reaching_the_limit_below_target_loses_the_run() {
    let content = content();
    let (mut run, _) = Run::new(&content, 3).expect("start");
    run.series.game_limit = 1;
    let events = run.apply(&content, RunAction::Pass).expect("pass");
    assert_eq!(run.phase(), RunPhase::Lost);
    assert!(events.contains(&RunEvent::RunLost {
        section: 0,
        index: 0,
        character: Character::Declarer,
        length: RunLength::Full,
    }));
    assert_eq!(
        run.apply(&content, RunAction::Pass),
        Err(RunError::WrongPhase)
    );
}

#[test]
fn failing_actions_leave_the_run_untouched() {
    let content = content();
    let (mut run, _) = Run::new(&content, 3).expect("start");
    let before = run.clone();
    let null = Declaration::null(NullRisk::Normal);
    // Null without Hand only after the skat (every character may declare
    // Null, ADR 0012).
    assert_eq!(
        run.apply(&content, RunAction::Declare(null)),
        Err(RunError::Game(GameError::InvalidDeclaration {
            declaration: null
        }))
    );
    let held = run.game().expect("game").game.hand(Seat::DECLARER)[0].id;
    assert_eq!(
        run.apply(&content, RunAction::Play(held)),
        Err(RunError::Game(GameError::WrongPhase {
            phase: Phase::Declaring
        }))
    );
    assert_eq!(
        run.apply(&content, RunAction::LeaveShop),
        Err(RunError::WrongPhase)
    );
    assert_eq!(
        run.apply(&content, RunAction::Redeal),
        Err(RunError::NoRedealLeft)
    );
    assert_eq!(run, before);
}

/// Play the current game to the end with the heuristics.
fn play_current_game(run: &mut Run, content: &Content, declaration: Declaration) -> Vec<RunEvent> {
    let mut events = super::content_tests::declare(run, content, declaration).expect("declare");
    while run.phase() == RunPhase::Game
        && run.game().is_some_and(|a| a.game.value().is_none())
        && !events
            .iter()
            .any(|e| matches!(e, RunEvent::GameScored { .. }))
    {
        let action = RunBot.next_action(run, content).expect("action");
        events.extend(run.apply(content, action).expect("legal"));
    }
    events
}

fn scored(events: &[RunEvent]) -> ScoreBreakdown {
    events
        .iter()
        .find_map(|e| match e {
            RunEvent::GameScored { breakdown, .. } => Some(breakdown.clone()),
            _ => None,
        })
        .expect("a scored game")
}

#[test]
fn series_won_pays_groschen_and_opens_the_shop() {
    let content = content();
    let (mut run, _) = Run::new(&content, 5).expect("start");
    run.series.target = 1;
    run.coins = 12;
    // With a target of 1 the first won game wins the series. Lost games
    // end the series below target, so try fresh runs until one is won.
    let mut won = false;
    for _ in 0..50 {
        let decl = heuristic_declaration(&run);
        let events = play_current_game(&mut run, &content, decl);
        if let Some(RunEvent::SeriesWon { payout, coins, .. }) = events
            .iter()
            .find(|e| matches!(e, RunEvent::SeriesWon { .. }))
        {
            let breakdown = scored(&events);
            assert!(breakdown.won && breakdown.score > 0);
            // 12 Groschen before: interest 2. One game played, the rest
            // of the limit left.
            let economy = &content.run.economy;
            let games_left = content.run.game_limit - 1;
            assert_eq!(payout.base, economy.series_reward);
            assert_eq!(payout.games_left, games_left * economy.per_game_left);
            assert_eq!(payout.interest, 2);
            assert_eq!(*coins, 12 + payout.base + payout.games_left + 2);
            won = true;
            break;
        }
        // Lost or still running: restart the series in place.
        let (fresh, _) = Run::new(&content, 5 + run.games_dealt as u64).expect("start");
        run = fresh;
        run.series.target = 1;
        run.coins = 12;
    }
    assert!(won, "some game must be won");
    assert_eq!(run.phase(), RunPhase::Shop);
    let shop = run.shop().expect("open");
    assert_eq!(shop.relics.len(), 3);
    assert_eq!(shop.consumables.len(), 2);
}

/// The declarer heuristic's declaration for the current hand.
fn heuristic_declaration(run: &Run) -> Declaration {
    let hand = run.game().expect("game").game.hand(Seat::DECLARER);
    crate::ai::DeclarerHeuristic.choose_declaration(hand)
}

fn open_shop(content: &Content, seed: u64, coins: u32) -> Run {
    let (mut run, _) = Run::new(content, seed).expect("start");
    run.coins = coins;
    run.game = None;
    run.phase = RunPhase::Shop;
    run.shop = Some(run.draw_offer(content, 0));
    run
}

#[test]
fn shop_offer_is_deterministic_and_never_repeats_owned_relics() {
    let content = content();
    let a = open_shop(&content, 9, 50);
    let b = open_shop(&content, 9, 50);
    assert_eq!(a.shop(), b.shop());
    let shop = a.shop().expect("shop");
    let mut ids: Vec<_> = shop.relics.iter().flatten().collect();
    ids.sort();
    ids.dedup();
    assert_eq!(ids.len(), 3, "no relic twice in one offer");
    let mut owned = open_shop(&content, 9, 50);
    owned.relics = content.relics.keys().take(8).cloned().collect();
    let offer = owned.draw_offer(&content, 0);
    for id in offer.relics.iter().flatten() {
        assert!(!owned.relics.contains(id));
    }
}

#[test]
fn buy_sell_reroll_and_slots() {
    let content = content();
    let mut run = open_shop(&content, 9, 100);
    // Without the Alleinspieler's start relic.
    run.relics.clear();
    let id = run.shop().and_then(|s| s.relics[0].clone()).expect("offer");
    let price = content.relic(&id).expect("def").price;
    let events = run.apply(&content, RunAction::BuyRelic(0)).expect("buy");
    assert_eq!(
        events,
        vec![RunEvent::RelicBought {
            relic: id.clone(),
            price
        }]
    );
    assert_eq!(run.coins(), 100 - price);
    assert_eq!(run.relics(), std::slice::from_ref(&id));
    assert_eq!(
        run.apply(&content, RunAction::BuyRelic(0)),
        Err(RunError::InvalidSlot)
    );
    // Sell for half the price.
    run.apply(&content, RunAction::SellRelic(0)).expect("sell");
    assert_eq!(run.coins(), 100 - price + price / 2);
    assert!(run.relics().is_empty());
    // Reroll costs 2 Groschen and draws a new offer.
    let coins = run.coins();
    run.apply(&content, RunAction::Reroll).expect("reroll");
    assert_eq!(run.coins(), coins - 2);
    assert_eq!(run.shop().map(|s| s.rerolls), Some(1));
    // Slots: five relics, two consumables.
    run.relics = content.relics.keys().take(5).cloned().collect();
    let free = run
        .shop()
        .and_then(|s| s.relics.iter().position(Option::is_some))
        .expect("offer");
    assert_eq!(
        run.apply(&content, RunAction::BuyRelic(free)),
        Err(RunError::NoFreeSlot)
    );
    run.apply(&content, RunAction::BuyConsumable(0))
        .expect("buy");
    run.apply(&content, RunAction::BuyConsumable(1))
        .expect("buy");
    assert_eq!(run.consumables().len(), 2);
    // Not enough coins.
    let mut poor = open_shop(&content, 9, 0);
    assert_eq!(
        poor.apply(&content, RunAction::BuyRelic(0)),
        Err(RunError::NotEnoughCoins)
    );
    assert_eq!(
        poor.apply(&content, RunAction::Reroll),
        Err(RunError::NotEnoughCoins)
    );
}

#[test]
fn leaving_the_shop_starts_the_next_series() {
    let content = content();
    let mut run = open_shop(&content, 9, 0);
    let events = run.apply(&content, RunAction::LeaveShop).expect("leave");
    assert!(events.contains(&RunEvent::ShopLeft));
    assert!(matches!(
        events[1],
        RunEvent::SeriesStarted {
            section: 0,
            index: 1,
            target,
            ..
        } if target == content.run.targets[0].1
    ));
    assert_eq!(run.phase(), RunPhase::Game);
    // After the boss series the next section begins.
    // The boss series brings the boss drawn at the start of the section.
    let mut run = open_shop(&content, 9, 0);
    let announced = run.section_boss().cloned().expect("drawn at the start");
    run.series.index = 1;
    let events = run.apply(&content, RunAction::LeaveShop).expect("leave");
    let boss = events.iter().find_map(|e| match e {
        RunEvent::SeriesStarted { index: 2, boss, .. } => boss.clone(),
        _ => None,
    });
    assert_eq!(boss, Some(announced));
    assert!(
        !events
            .iter()
            .any(|e| matches!(e, RunEvent::BossAnnounced { .. }))
    );
    // A new section draws its boss first.
    let mut run = open_shop(&content, 9, 0);
    run.series.index = 2;
    let events = run.apply(&content, RunAction::LeaveShop).expect("leave");
    assert!(matches!(
        &events[1],
        RunEvent::BossAnnounced { section: 1, boss } if Some(boss) == run.section_boss()
    ));
    assert!(matches!(
        events[2],
        RunEvent::SeriesStarted {
            section: 1,
            index: 0,
            target,
            boss: None,
            ..
        } if target == content.run.targets[1].0
    ));
}

#[test]
fn consumables_change_the_deck_and_the_series() {
    let content = content();
    let mut run = open_shop(&content, 9, 0);
    run.consumables = vec![
        ConsumableId::new("fresh_deck"),
        ConsumableId::new("ink_pad"),
    ];
    assert_eq!(
        run.apply(
            &content,
            RunAction::UseConsumable {
                slot: 0,
                target: None
            }
        ),
        Err(RunError::TargetRequired)
    );
    run.apply(
        &content,
        RunAction::UseConsumable {
            slot: 0,
            target: Some(0),
        },
    )
    .expect("remove D7");
    assert_eq!(run.deck().blanks(), 1);
    assert_eq!(run.consumables(), &[ConsumableId::new("ink_pad")]);
    run.apply(
        &content,
        RunAction::UseConsumable {
            slot: 0,
            target: Some(31),
        },
    )
    .expect("stamp CA");
    assert_eq!(run.deck().get(31).map(|c| c.mods.len()), Some(1));
    assert!(run.consumables().is_empty());

    // Beer during a series, new deal before declaring.
    let (mut run, _) = Run::new(&content, 4).expect("start");
    run.consumables = vec![ConsumableId::new("beer"), ConsumableId::new("new_deal")];
    run.apply(
        &content,
        RunAction::UseConsumable {
            slot: 0,
            target: None,
        },
    )
    .expect("beer");
    assert_eq!(run.series().passes_left, content.run.passes + 1);
    let before = run.game().map(|a| a.game.hand(Seat::DECLARER).to_vec());
    let events = run
        .apply(
            &content,
            RunAction::UseConsumable {
                slot: 0,
                target: None,
            },
        )
        .expect("new deal");
    assert!(events.contains(&RunEvent::Redealt));
    assert_eq!(run.series().games_played, 0, "a new deal costs no game");
    assert_eq!(run.game().map(|a| a.game.forehand()), Some(Seat::DECLARER));
    assert_ne!(
        run.game().map(|a| a.game.hand(Seat::DECLARER).to_vec()),
        before
    );
    // Deck changes only at the Tresen.
    run.consumables = vec![ConsumableId::new("fresh_deck")];
    assert_eq!(
        run.apply(
            &content,
            RunAction::UseConsumable {
                slot: 0,
                target: Some(3)
            }
        ),
        Err(RunError::WrongPhase)
    );
}

#[test]
fn a_removed_card_is_dealt_as_blank() {
    let content = content();
    let (mut run, _) = Run::new(&content, 4).expect("start");
    run.deck.remove(31).expect("remove CA");
    run.games_dealt = 0;
    let mut events = Vec::new();
    run.deal(&content, true, &mut events).expect("deal");
    let game = &run.game().expect("game").game;
    let blanks = game
        .cards_in_play()
        .iter()
        .filter(|c| c.kind == CardKind::Blank)
        .count();
    assert_eq!(blanks, 1);
    let ace: Card = "CA".parse().expect("card");
    assert!(
        !game
            .cards_in_play()
            .iter()
            .any(|c| c.kind == CardKind::Normal(ace))
    );
    assert_eq!(game.cards_in_play().len(), 32);
}

#[test]
fn the_boss_is_known_from_the_start_of_the_section() {
    let content = content();
    let (run, _) = Run::new(&content, 31).expect("start");
    let boss = run.section_boss().cloned().expect("boss");
    let view = run.view(&content);
    assert_eq!(view.upcoming_boss.as_ref().map(|b| &b.id), Some(&boss));
    let def = content.boss(&boss).expect("def");
    let info = view.upcoming_boss.expect("boss");
    assert_eq!(info.name, def.name);
    assert_eq!(info.text, def.text);
    // The next series is series 2 of section 1, without boss.
    let next = view.next_series.expect("next");
    assert_eq!((next.section, next.index, next.is_boss), (0, 1, false));
    assert_eq!(next.target, content.run.targets[0].1);
    assert_eq!(next.game_limit, content.run.game_limit);
    assert!(next.boss.is_none());
    // At the Tresen before the boss series the preview shows the boss.
    let mut run = open_shop(&content, 31, 0);
    run.series.index = 1;
    let next = run.view(&content).next_series.expect("next");
    assert!(next.is_boss);
    assert_eq!(
        next.target,
        series_target(&content, Character::Declarer, 0, 2, Some(&boss)).expect("target")
    );
    assert_eq!(next.boss.map(|b| b.id), Some(boss));
    // After the boss series the boss is no longer ahead.
    run.series.index = 2;
    let view = run.view(&content);
    assert!(view.upcoming_boss.is_none());
    let next = view.next_series.expect("next");
    assert_eq!((next.section, next.index), (1, 0));
}

/// ADR 0012: a boss may scale the Zielwert differently per character; the
/// view shows the factor of the run's character.
#[test]
fn a_boss_may_have_its_own_factor_for_a_character() {
    let content = Content::load().expect("content");
    let pusher = BossId::new("the_pusher");
    let def = content.boss(&pusher).expect("the_pusher");
    assert_eq!(
        def.target_factor_for(Character::Declarer),
        def.target_factor
    );
    let factor = def.target_factor_for(Character::NullPlayer);
    assert_ne!(factor, def.target_factor);
    let null_base = content.characters[&Character::NullPlayer]
        .targets
        .as_ref()
        .unwrap_or(&content.run.targets)[0]
        .2;
    assert_eq!(
        series_target(&content, Character::NullPlayer, 0, 2, Some(&pusher)),
        Some(factor.apply(null_base))
    );
    // Both bosses that were the Alleinspieler's alone are back for
    // everyone; the Oma stays his (the Nullspieler has no jacks).
    for id in ["the_pusher", "black_peter"] {
        let def = content.boss(&BossId::new(id)).expect(id);
        assert!(crate::content::in_pool(
            &def.characters,
            Character::NullPlayer
        ));
    }
    let grandma = content.boss(&BossId::new("grandma")).expect("grandma");
    assert!(!crate::content::in_pool(
        &grandma.characters,
        Character::NullPlayer
    ));
}

#[test]
fn the_boss_target_factor_scales_the_boss_series() {
    let bosses = r#"[(id: "hard", name: "Hart", text: "x", target_factor: 1.4)]"#;
    let hard_def: Vec<crate::content::BossDef> = ron::from_str(bosses).expect("ron");
    let mut content = Content::load().expect("content");
    let hard = BossId::new("hard");
    content.bosses.insert(hard.clone(), hard_def[0].clone());
    content.run.bosses = vec![hard.clone()];
    content.run.final_boss = None;
    assert_eq!(
        content.boss(&hard).map(|b| b.target_factor),
        Some(crate::content::Factor::from_permille(1400))
    );
    let base = content.run.targets[0].2;
    let expected = (base * 14 + 5) / 10;
    assert_eq!(
        series_target(&content, Character::Declarer, 0, 2, Some(&hard)),
        Some(expected)
    );
    assert_eq!(
        series_target(&content, Character::Declarer, 0, 1, None),
        Some(content.run.targets[0].1)
    );
    let start = RunStart {
        index: 2,
        ..RunStart::default()
    };
    let (run, events) = Run::new_with(&content, 3, &start).expect("start");
    assert_eq!(run.series().target, expected);
    assert_eq!(run.series().boss, Some(hard));
    assert!(events.iter().any(|e| matches!(
        e,
        RunEvent::SeriesStarted { index: 2, target, .. } if *target == expected
    )));
    // Factors are positive and at most 2 (set from measurements, see
    // bosses.ron).
    for def in Content::load().expect("content").bosses.values() {
        assert!(
            (1..=2000).contains(&def.target_factor.permille()),
            "{}",
            def.id
        );
    }
}

#[test]
fn a_run_can_start_with_relics_and_later() {
    let content = content();
    let start = RunStart {
        relics: vec![RelicId::new("old_lady")],
        ..RunStart::default()
    };
    let (with, _) = Run::new_with(&content, 12, &start).expect("start");
    let (without, _) = Run::new(&content, 12).expect("start");
    // The character's start relic first, then the extra one.
    assert_eq!(
        with.relics(),
        &[RelicId::new("card_counter"), RelicId::new("old_lady")]
    );
    // Same cards, but the passive effect already counts in the first game.
    let ids = |run: &Run| -> Vec<u32> {
        run.game()
            .expect("game")
            .game
            .hand(Seat::DECLARER)
            .iter()
            .map(|c| c.id)
            .collect()
    };
    assert_eq!(ids(&with), ids(&without));
    let queens = |run: &Run| -> Vec<u32> {
        run.game()
            .expect("game")
            .game
            .cards_in_play()
            .iter()
            .filter(|c| {
                c.face()
                    .is_some_and(|f| f.rank == crate::cards::Rank::Queen)
            })
            .map(|c| c.card_points)
            .collect()
    };
    assert!(queens(&with).iter().all(|&p| p == 5));
    assert!(queens(&without).iter().all(|&p| p == 3));
    let bad = |start: RunStart| Run::new_with(&content, 1, &start).err();
    assert_eq!(
        bad(RunStart {
            relics: vec![RelicId::new("nope")],
            ..RunStart::default()
        })
        .map(|e| e.code()),
        Some(RunErrorCode::UnknownContent)
    );
    assert_eq!(
        bad(RunStart {
            section: 99,
            ..RunStart::default()
        }),
        Some(RunError::InvalidStart)
    );
    assert_eq!(
        bad(RunStart {
            relics: content.relics.keys().cloned().collect(),
            ..RunStart::default()
        }),
        Some(RunError::NoFreeSlot)
    );
    let (later, _) = Run::new_with(
        &content,
        1,
        &RunStart {
            section: 1,
            index: 1,
            ..RunStart::default()
        },
    )
    .expect("start");
    assert_eq!((later.series().section, later.series().index), (1, 1));
    assert_eq!(later.series().target, content.run.targets[1].1);
    assert!(later.section_boss().is_some());
}

#[test]
fn error_codes_are_stable_names() {
    let cases = [
        (RunError::WrongPhase, "WrongPhase"),
        (RunError::NotEnoughCoins, "NotEnoughCoins"),
        (RunError::UnknownCard(3), "UnknownCard"),
        (RunError::NoRedealLeft, "NoRedealLeft"),
        (
            RunError::Game(GameError::WrongPhase {
                phase: Phase::Declaring,
            }),
            "GameWrongPhase",
        ),
        (
            RunError::Game(GameError::TrickInProgress),
            "TrickInProgress",
        ),
        (
            RunError::Deck(DeckError::WouldShrinkBelowMinimum),
            "DeckAtMinimum",
        ),
        (RunError::InvalidStart, "InvalidStart"),
    ];
    for (error, name) in cases {
        let code = error.code();
        assert_eq!(code.as_str(), name);
        assert_eq!(ron::to_string(&code).expect("ron"), name);
        let text = error.to_string();
        assert!(!text.is_empty() && !text.contains('{'), "{text}");
    }
}

#[test]
fn payout_serializes_its_total() {
    let payout = Payout {
        base: 3,
        games_left: 2,
        interest: 1,
    };
    let text = ron::to_string(&payout).expect("ron");
    assert!(text.contains("total:6"), "{text}");
    let back: Payout = ron::from_str(&text).expect("read");
    assert_eq!(back, payout);
    let old: Payout = ron::from_str("(base:3,games_left:2,interest:1)").expect("old");
    assert_eq!(old.total(), 6);
}

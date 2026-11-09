//! Routes and encounters v2 (decision of the user, 03.11.2026): three
//! different stop categories per route choice, one always the Tresen; the
//! new stops Fundgroschen, Kiosk and Würfelbude; every encounter with two
//! or three options, one of which must be taken.

use super::bot::RunBot;
use super::content_tests::{content, relic_run};
use super::tour::RouteStop;
use super::*;
use crate::content::EncounterEffect;
use crate::content::{ConsumableCategory, EncounterDef, StopCategory};

/// End the current pub as won in its first game.
fn win_pub(run: &mut Run, content: &Content) {
    run.series.games_played = 1;
    run.series.score = i64::try_from(run.series.target).expect("small");
    run.game = None;
    let mut events = Vec::new();
    run.after_game(content, &mut events).expect("after game");
    run.pass_card_reward(content, &mut events);
}

/// A run at the route choice after the first pub.
fn at_route(content: &Content) -> Run {
    let mut run = relic_run(content, &[]);
    win_pub(&mut run, content);
    assert_eq!(run.phase(), RunPhase::Route);
    run
}

/// Take way 0 with this one stop.
fn walk_to(run: &mut Run, content: &Content, stop: RouteStop) -> Vec<RunEvent> {
    let route = run.route.as_mut().expect("route");
    for option in &mut route.options {
        option.stops = vec![stop.clone()];
        option.spelunke = None;
    }
    run.apply(content, RunAction::ChooseRoute(0)).expect("walk")
}

fn stop(category: StopCategory, encounter: Option<&str>) -> RouteStop {
    RouteStop {
        category,
        encounter: encounter.map(EncounterId::new),
    }
}

fn choose(run: &mut Run, content: &Content, option: usize) -> Vec<RunEvent> {
    run.apply(
        content,
        RunAction::ChooseEncounter {
            option,
            target: None,
            marke: None,
        },
    )
    .expect("choose")
}

#[test]
fn every_route_choice_shows_three_categories_one_of_them_the_tresen() {
    let content = content();
    let mut seen = BTreeSet::new();
    let mut tresen_ways = BTreeSet::new();
    for seed in 0..120 {
        let (mut run, _) = Run::new(&content, seed).expect("run");
        win_pub(&mut run, &content);
        for _ in 0..4 {
            let route = run.route().expect("route");
            let categories: Vec<StopCategory> = route
                .options
                .iter()
                .map(|o| {
                    assert_eq!(o.stops.len(), 1);
                    o.stops[0].category
                })
                .collect();
            let distinct: BTreeSet<StopCategory> = categories.iter().copied().collect();
            assert_eq!(distinct.len(), 3, "seed {seed}: {categories:?}");
            let tresen = categories
                .iter()
                .position(|c| *c == StopCategory::Tresen)
                .expect("a Tresen");
            tresen_ways.insert(tresen);
            seen.extend(categories);
            run.reroll_routes(&content);
        }
    }
    // The Tresen lies on any of the three ways, and every category comes.
    assert_eq!(tresen_ways.len(), 3);
    for category in [
        StopCategory::Event,
        StopCategory::Upgrade,
        StopCategory::Rest,
        StopCategory::FoundCoins,
        StopCategory::Kiosk,
        StopCategory::DiceStall,
    ] {
        assert!(seen.contains(&category), "{category:?}");
    }
}

#[test]
fn every_encounter_has_two_or_three_options_and_one_for_nothing() {
    let content = content();
    let single = ["found_coins"];
    for def in content.encounters.values() {
        let regular: Vec<_> = def.options.iter().filter(|o| !o.fallback).collect();
        if single.contains(&def.id.as_str()) {
            assert_eq!(regular.len(), 1, "{}", def.id);
        } else if def.id.as_str() == "spelunke_loot" {
            // The workshop has three since 04.11.2026 (a random Marke).
            assert_eq!(regular.len(), 2, "{}", def.id);
        } else {
            assert!((2..=3).contains(&regular.len()), "{}", def.id);
        }
        let bets = def.options.iter().all(|o| {
            o.effects
                .iter()
                .any(|e| matches!(e, EncounterEffect::Bet { .. }))
        });
        assert!(
            bets || def.options.iter().any(|o| o.cost == 0 || o.fallback),
            "{}",
            def.id
        );
        assert!(
            def.options.iter().all(|o| !o.label.is_empty()),
            "{}",
            def.id
        );
    }
}

#[test]
fn a_fresh_run_can_take_an_option_of_every_encounter_it_meets() {
    let content = content();
    for character in Character::ALL {
        let start = RunStart {
            character,
            ..RunStart::default()
        };
        let (mut run, _) = Run::new_with(&content, 5, &start).expect("run");
        win_pub(&mut run, &content);
        run.coins = 0;
        let met: Vec<&EncounterDef> = content
            .encounters
            .values()
            .filter(|e| e.weight > 0 && crate::content::in_pool(&e.characters, character))
            .collect();
        for def in met {
            let available = run.encounter_available(&content, def);
            // Only the bet stakes every option.
            assert_eq!(
                available,
                def.id.as_str() != "the_bet",
                "{character} {}",
                def.id
            );
            if !available {
                continue;
            }
            let mut probe = run.clone();
            probe.test_encounter(&content, def.id.as_str());
            let action = RunBot.encounter_action(&probe, &content);
            probe.apply(&content, action).expect("the bot takes one");
        }
    }
}

#[test]
fn the_found_coins_give_two_groschen() {
    let content = content();
    let mut run = at_route(&content);
    let coins = run.coins();
    walk_to(
        &mut run,
        &content,
        stop(StopCategory::FoundCoins, Some("found_coins")),
    );
    assert_eq!(run.phase(), RunPhase::Encounter);
    let view = run.view(&content).encounter.expect("view");
    assert_eq!(view.category, StopCategory::FoundCoins);
    assert_eq!(view.options.len(), 1);
    choose(&mut run, &content, 0);
    assert_eq!(run.coins(), coins + 2);
    assert_eq!(run.phase(), RunPhase::Game);
}

#[test]
fn the_dice_stall_doubles_the_stake_or_takes_it() {
    let content = content();
    let (mut won, mut lost) = (0, 0);
    for seed in 0..40 {
        let mut run = at_route(&content);
        run.encounter_rng = crate::rng::Rng::from_seed(seed);
        run.coins = 10;
        walk_to(
            &mut run,
            &content,
            stop(StopCategory::DiceStall, Some("dice_stall")),
        );
        // With Groschen to stake, watching is not offered.
        assert_eq!(
            run.encounter_option_block(&content, 3),
            Some(OptionBlock::NotNeeded)
        );
        let events = choose(&mut run, &content, 1);
        let thrown = events
            .iter()
            .find_map(|e| match e {
                RunEvent::DiceThrown { stake, won, payout } => Some((*stake, *won, *payout)),
                _ => None,
            })
            .expect("thrown");
        match thrown {
            (4, true, 8) => {
                won += 1;
                assert_eq!(run.coins(), 14);
            }
            (4, false, 0) => {
                lost += 1;
                assert_eq!(run.coins(), 6);
            }
            other => panic!("{other:?}"),
        }
    }
    assert!(won > 8 && lost > 8, "{won} won, {lost} lost");
    // Too poor for the smallest stake: only watching.
    let mut run = at_route(&content);
    run.coins = 1;
    walk_to(
        &mut run,
        &content,
        stop(StopCategory::DiceStall, Some("dice_stall")),
    );
    let view = run.view(&content).encounter.expect("view");
    let open: Vec<bool> = view.options.iter().map(|o| o.can_choose).collect();
    assert_eq!(open, [false, false, false, true]);
    assert_eq!(view.options[0].blocked, Some(OptionBlock::NotEnoughCoins));
    assert_eq!(
        RunBot.encounter_action(&run, &content),
        RunAction::ChooseEncounter {
            option: 3,
            target: None,
            marke: None
        }
    );
    choose(&mut run, &content, 3);
    assert_eq!(run.coins(), 1);
}

#[test]
fn the_kiosk_sells_one_of_two_schnapps_cheaper() {
    let content = content();
    let mut run = at_route(&content);
    run.coins = 20;
    run.consumables.clear();
    let events = walk_to(&mut run, &content, stop(StopCategory::Kiosk, None));
    assert!(
        events
            .iter()
            .any(|e| matches!(e, RunEvent::KioskOpened { .. }))
    );
    assert_eq!(run.phase(), RunPhase::Shop);
    assert!(run.at_kiosk());
    let view = run.view(&content).shop.expect("kiosk");
    assert!(view.kiosk);
    assert!(view.relics.is_empty() && view.card_offers.is_empty());
    assert_eq!(view.consumables.len(), 2);
    let offer = view.consumables[0].as_ref().expect("offer");
    let def = content.consumable(&offer.consumable.id).expect("known");
    assert_eq!(def.category, ConsumableCategory::Schnapps);
    let tresen = run.shop_price(&content, def.price);
    assert_eq!(offer.price, tresen.saturating_sub(1).max(1));
    // No Tresen services at the Kiosk.
    for action in [
        RunAction::Reroll,
        RunAction::HirePainter(31),
        RunAction::CleanseCard(31),
    ] {
        assert!(!run.accepts(&content, action.clone()), "{action:?}");
    }
    run.apply(&content, RunAction::BuyConsumable(0))
        .expect("buy");
    assert_eq!(run.coins(), 20 - offer.price);
    assert_eq!(
        run.consumables(),
        std::slice::from_ref(&offer.consumable.id)
    );
    // One per Kiosk.
    assert!(!run.accepts(&content, RunAction::BuyConsumable(1)));
    run.apply(&content, RunAction::LeaveShop).expect("leave");
    assert_eq!(run.phase(), RunPhase::Game);
}

#[test]
fn an_open_encounter_of_an_old_save_is_opened_anew() {
    let content = content();
    let mut run = at_route(&content);
    walk_to(
        &mut run,
        &content,
        stop(StopCategory::Event, Some("stammtisch_dispute")),
    );
    assert_eq!(run.phase(), RunPhase::Encounter);
    // The form before 03.11.2026: options with a choice, "Weitergehen" last.
    let mut save = serde_json::to_value(&run).expect("json");
    save["encounter"] = serde_json::json!({
        "id": "stammtisch_dispute",
        "options": [
            { "choice": { "TakeRule": "blind_eye" }, "cost": 0, "card_mod": null },
            { "choice": "Leave", "cost": 0, "card_mod": null }
        ]
    });
    let mut loaded: Run = serde_json::from_value(save).expect("old save loads");
    assert!(loaded.encounter().expect("kept").options.is_empty());
    loaded.drop_unknown_content(&content);
    let encounter = loaded.encounter().expect("opened anew");
    assert_eq!(encounter.id.as_str(), "stammtisch_dispute");
    assert_eq!(encounter.options.len(), 3);
    assert_eq!(loaded.phase(), RunPhase::Encounter);
    let action = RunBot.encounter_action(&loaded, &content);
    loaded.apply(&content, action).expect("take");
    assert_eq!(loaded.phase(), RunPhase::Game);
    // The old Rast (a life) opens anew as the Rast of ADR 0019.
    let mut save = serde_json::to_value(&run).expect("json");
    save["encounter"] = serde_json::json!({
        "id": "snack_bar",
        "options": [{ "choice": { "Heal": 1 }, "cost": 0 }]
    });
    let mut loaded: Run = serde_json::from_value(save).expect("old save loads");
    loaded.drop_unknown_content(&content);
    let rest = loaded.encounter().expect("opened anew");
    assert_eq!(rest.id.as_str(), "snack_bar");
    assert!(
        rest.options
            .iter()
            .flat_map(|o| &o.effects)
            .all(|e| !matches!(e, EncounterEffect::Lives(_)))
    );
    // A life offered in the form of 03.11.2026 becomes Groschen.
    let mut old = run.clone();
    if let Some(encounter) = old.encounter.as_mut() {
        encounter.options[0].effects = vec![EncounterEffect::Lives(1)];
    }
    let json = serde_json::to_string(&old).expect("json");
    let mut loaded: Run = serde_json::from_str(&json).expect("old save loads");
    loaded.drop_unknown_content(&content);
    assert_eq!(
        loaded.encounter().expect("kept").options[0].effects,
        vec![EncounterEffect::Coins(crate::content::LEGACY_LIFE_COINS)]
    );
    // The new form survives a round trip.
    let json = serde_json::to_string(&run).expect("json");
    let back: Run = serde_json::from_str(&json).expect("load");
    assert_eq!(back, run);
}

#[test]
fn the_card_trick_shows_the_skat_of_every_deal_until_the_declaration() {
    let content = content();
    let start = RunStart {
        character: Character::NullPlayer,
        ..RunStart::default()
    };
    let (mut run, _) = Run::new_with(&content, 3, &start).expect("run");
    win_pub(&mut run, &content);
    run.test_encounter(&content, "conjurer");
    let events = choose(&mut run, &content, 2);
    assert!(events.contains(&RunEvent::SkatPeekPlanned));
    assert!(
        events
            .iter()
            .any(|e| matches!(e, RunEvent::SkatPeeked { .. }))
    );
    // Einmischen: the new deal shows its skat too.
    let events = run.apply(&content, RunAction::Redeal).expect("redeal");
    assert!(
        events
            .iter()
            .any(|e| matches!(e, RunEvent::SkatPeeked { .. }))
    );
    assert!(run.peek_next_skat.is_some());
}

#[test]
fn a_bot_run_meets_the_new_stops_and_never_gets_stuck() {
    let content = content();
    let mut categories = BTreeSet::new();
    for seed in 0..12 {
        let (mut run, _) = Run::new(&content, seed).expect("run");
        let mut steps = 0;
        while let Some(action) = RunBot.next_action(&run, &content) {
            if run.phase() == RunPhase::Route {
                if let Some(route) = run.route() {
                    categories.extend(route.options.iter().map(|o| o.stops[0].category));
                }
            }
            run.apply(&content, action).expect("legal");
            steps += 1;
            assert!(steps < 20_000, "seed {seed} stuck");
        }
    }
    assert!(categories.len() >= 5, "{categories:?}");
}

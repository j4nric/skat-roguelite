//! The Schnaps after a won pub and the Schnaps-Leiste (DESIGN 8.9,
//! decided by the user on 03.11.2026): the reward, the choice with every
//! slot full, its own generator, old saves, the bot, the target kinds and
//! `usable_now`.

use super::bot::RunBot;
use super::tour::Spelunke;
use super::*;
use crate::content::{ConsumableCategory, ConsumableTarget, MalusId, Rarity};

fn content() -> Content {
    Content::load().expect("content")
}

fn start_at(content: &Content, seed: u64, section: u32, index: u32) -> Run {
    let start = RunStart {
        section,
        index,
        ..RunStart::default()
    };
    Run::new_with(content, seed, &start).expect("start").0
}

/// End the current pub as if its games were played, won or lost.
fn end_pub(run: &mut Run, content: &Content, games: u32, won: bool) -> Vec<RunEvent> {
    run.series.games_played = games;
    let target = i64::try_from(run.series.target).expect("small");
    run.series.score = if won { target } else { target - 1 };
    run.game = None;
    let mut events = Vec::new();
    run.after_game(content, &mut events).expect("after game");
    run.pass_card_reward(content, &mut events);
    events
}

fn rewarded(events: &[RunEvent]) -> Vec<(ConsumableId, bool)> {
    events
        .iter()
        .filter_map(|e| match e {
            RunEvent::SchnappsRewarded {
                consumable,
                slots_full,
            } => Some((consumable.clone(), *slots_full)),
            _ => None,
        })
        .collect()
}

fn ids(names: &[&str]) -> Vec<ConsumableId> {
    names.iter().map(|n| ConsumableId::new(n)).collect()
}

const FULL: [&str; 3] = ["shot", "herbal", "eggnog"];

#[test]
fn three_schnaps_slots() {
    assert_eq!(content().run.consumable_slots, 3);
}

#[test]
fn a_won_pub_puts_a_random_schnaps_into_a_free_slot() {
    let content = content();
    let mut run = start_at(&content, 5, 0, 0);
    run.consumables.clear();
    let events = end_pub(&mut run, &content, 1, true);
    let got = rewarded(&events);
    assert_eq!(got.len(), 1, "{events:?}");
    let (id, full) = &got[0];
    assert!(!full);
    let def = content.consumable(id).expect("known");
    assert_eq!(def.category, ConsumableCategory::Schnapps);
    assert_eq!(run.consumables(), std::slice::from_ref(id));
    assert_eq!(run.schnapps_reward(), None);
    assert_eq!(run.phase(), RunPhase::Route);
    assert!(run.view(&content).schnapps_reward.is_none());
    // The Schnaps comes after the pay, before the route choice.
    let won = events
        .iter()
        .position(|e| matches!(e, RunEvent::SeriesWon { .. }))
        .expect("won");
    let schnaps = events
        .iter()
        .position(|e| matches!(e, RunEvent::SchnappsRewarded { .. }))
        .expect("schnaps");
    let route = events
        .iter()
        .position(|e| matches!(e, RunEvent::RouteOpened { .. }))
        .expect("route");
    assert!(won < schnaps && schnaps < route);
}

#[test]
fn a_lost_pub_gives_no_schnaps() {
    let content = content();
    let mut run = start_at(&content, 5, 0, 1);
    run.consumables.clear();
    let events = end_pub(&mut run, &content, 3, false);
    assert!(rewarded(&events).is_empty());
    assert!(run.consumables().is_empty());
}

#[test]
fn the_reward_is_drawn_by_rarity_from_the_characters_schnaepse() {
    let content = content();
    let mut seen: BTreeMap<ConsumableId, u32> = BTreeMap::new();
    for seed in 0..300 {
        let mut run = start_at(&content, seed, 0, 0);
        run.consumables.clear();
        end_pub(&mut run, &content, 1, true);
        *seen.entry(run.consumables()[0].clone()).or_default() += 1;
    }
    let pool = start_at(&content, 0, 0, 0).schnapps_pool(&content);
    for id in seen.keys() {
        assert!(pool.iter().any(|(p, _)| p == id), "{id} not in the pool");
    }
    let common: u32 = seen
        .iter()
        .filter(|(id, _)| content.consumable(id).expect("known").rarity == Rarity::Common)
        .map(|(_, n)| n)
        .sum();
    assert!(common > 150, "commons come more often: {seen:?}");
    assert!(seen.len() >= 8, "most Schnäpse come up: {seen:?}");
}

#[test]
fn with_every_slot_full_the_player_takes_it_for_one_or_sells_it() {
    let content = content();
    let mut run = start_at(&content, 5, 0, 0);
    run.consumables = ids(&FULL);
    let coins_before = run.coins();
    let events = end_pub(&mut run, &content, 1, true);
    let [(id, true)] = &rewarded(&events)[..] else {
        panic!("{events:?}");
    };
    let id = id.clone();
    assert_eq!(run.schnapps_reward(), Some(&id));
    assert_eq!(run.consumables(), ids(&FULL));
    assert_eq!(run.phase(), RunPhase::Route, "the route waits below");
    let view = run.view(&content);
    let reward = view.schnapps_reward.expect("choice");
    assert_eq!(reward.consumable.id, id);
    let price = content.consumable(&id).expect("known").price;
    assert_eq!(reward.sell_price, price / 2);
    // Nothing else goes before the choice, and no Schnaps can be drunk.
    assert_eq!(
        run.apply(&content, RunAction::ChooseRoute(0)),
        Err(RunError::SchnappsRewardPending)
    );
    assert!(view.consumables.iter().all(|c| !c.usable_now));
    assert!(view.consumables.iter().all(|c| !c.can_sell));
    assert!(
        view.tour
            .route
            .expect("route")
            .options
            .iter()
            .all(|o| !o.can_choose)
    );
    assert_eq!(
        run.apply(
            &content,
            RunAction::ChooseSchnappsReward { give_up: Some(3) }
        ),
        Err(RunError::InvalidSlot)
    );
    // Take it for the Kräuter.
    let mut taken = run.clone();
    let events = taken
        .apply(
            &content,
            RunAction::ChooseSchnappsReward { give_up: Some(1) },
        )
        .expect("take");
    assert!(events.contains(&RunEvent::SchnappsRewardTaken {
        consumable: id.clone(),
        given_up: ConsumableId::new("herbal"),
    }));
    assert_eq!(
        taken.consumables(),
        [
            ConsumableId::new("shot"),
            id.clone(),
            ConsumableId::new("eggnog")
        ]
    );
    assert_eq!(taken.schnapps_reward(), None);
    assert_eq!(taken.coins(), run.coins());
    taken
        .apply(&content, RunAction::ChooseRoute(0))
        .expect("on the way");
    // Or sell it for half its price.
    let events = run
        .apply(&content, RunAction::ChooseSchnappsReward { give_up: None })
        .expect("sell");
    assert!(events.contains(&RunEvent::ConsumableSold {
        consumable: id,
        price: price / 2,
    }));
    assert_eq!(run.consumables(), ids(&FULL));
    assert!(run.coins() >= coins_before + price / 2);
    assert_eq!(
        run.apply(&content, RunAction::ChooseSchnappsReward { give_up: None }),
        Err(RunError::WrongPhase),
        "no second choice"
    );
}

#[test]
fn a_boss_pub_gives_a_schnaps_and_the_relic_pick() {
    let content = content();
    let mut run = start_at(&content, 5, 0, 4);
    run.consumables = ids(&FULL);
    let events = end_pub(&mut run, &content, 3, true);
    assert_eq!(rewarded(&events).len(), 1);
    assert_eq!(run.phase(), RunPhase::BossReward);
    assert!(run.boss_reward().is_some());
    assert!(run.schnapps_reward().is_some());
    assert_eq!(
        run.apply(
            &content,
            RunAction::ChooseBossRelic {
                offer: None,
                replace: None
            }
        ),
        Err(RunError::SchnappsRewardPending)
    );
    // The bot answers the Schnaps first, then picks a relic.
    let first = RunBot.next_action(&run, &content).expect("action");
    assert!(matches!(first, RunAction::ChooseSchnappsReward { .. }));
    run.apply(&content, first).expect("schnaps");
    let second = RunBot.next_action(&run, &content).expect("action");
    assert!(matches!(second, RunAction::ChooseBossRelic { .. }));
    run.apply(&content, second).expect("relic");
    assert_eq!(run.phase(), RunPhase::Route);
}

#[test]
fn the_last_boss_pub_wins_the_run_without_a_schnaps() {
    let content = content();
    let last = content.run.sections() - 1;
    let mut run = start_at(&content, 5, last, 4);
    run.consumables.clear();
    let events = end_pub(&mut run, &content, 3, true);
    assert_eq!(run.phase(), RunPhase::Won);
    assert!(rewarded(&events).is_empty());
    assert!(run.consumables().is_empty());
}

#[test]
fn a_spelunke_gives_a_schnaps_and_its_loot() {
    let content = content();
    let mut run = start_at(&content, 5, 0, 0);
    end_pub(&mut run, &content, 1, true);
    run.consumables.clear();
    let route = run.route.as_mut().expect("route");
    route.options[0].stops.clear();
    route.options[0].spelunke = Some(Spelunke {
        name: "Spelunke Zum Test".into(),
        malus: MalusId::new("no_redeal"),
        malus_options: Vec::new(),
    });
    run.apply(&content, RunAction::ChooseRoute(0))
        .expect("spelunke");
    let events = end_pub(&mut run, &content, 2, true);
    assert_eq!(rewarded(&events).len(), 1);
    assert_eq!(run.consumables().len(), 1);
    assert_eq!(run.phase(), RunPhase::Encounter, "the loot follows");
    assert_eq!(
        run.encounter().map(|e| e.id.as_str()),
        Some("spelunke_loot")
    );
}

#[test]
fn the_reward_leaves_every_other_draw_alone() {
    let content = content();
    for seed in 0..10 {
        let mut run = start_at(&content, seed, 0, 0);
        run.consumables.clear();
        let before = run.clone();
        let mut events = Vec::new();
        run.reward_schnapps(&content, &mut events);
        assert_eq!(run.consumables().len(), 1);
        assert_eq!(run.deal_rng, before.deal_rng, "seed {seed}");
        assert_eq!(run.shop_rng, before.shop_rng);
        assert_eq!(run.misc_rng, before.misc_rng);
        assert_eq!(run.encounter_rng, before.encounter_rng);
        assert_eq!(run.tour_rng, before.tour_rng);
        assert_eq!(run.zinken_rng, before.zinken_rng);
        assert_eq!(run.trade_rng, before.trade_rng);
        assert_ne!(run.schnapps_rng, before.schnapps_rng);
    }
    // The deals of a run stay as they were: the first game of the next
    // pub is dealt the same with and without the Schnaps.
    let mut a = start_at(&content, 3, 0, 0);
    let mut b = a.clone();
    a.consumables.clear();
    b.consumables = ids(&FULL);
    end_pub(&mut a, &content, 1, true);
    end_pub(&mut b, &content, 1, true);
    b.apply(&content, RunAction::ChooseSchnappsReward { give_up: None })
        .expect("sell");
    assert_eq!(a.route, b.route);
    a.apply(&content, RunAction::ChooseRoute(0)).expect("a");
    b.apply(&content, RunAction::ChooseRoute(0)).expect("b");
    assert_eq!(a.deal_rng, b.deal_rng);
}

#[test]
fn old_saves_load_without_the_schnaps_generator() {
    let content = content();
    let mut run = start_at(&content, 7, 0, 0);
    run.consumables = ids(&FULL);
    end_pub(&mut run, &content, 1, true);
    let json = serde_json::to_string(&run).expect("save");
    let loaded: Run = serde_json::from_str(&json).expect("load");
    assert_eq!(loaded, run, "the open choice survives a save");
    let mut value: serde_json::Value = serde_json::to_value(&run).expect("value");
    let object = value.as_object_mut().expect("object");
    assert!(object.remove("schnapps_rng").is_some());
    assert!(object.remove("schnapps_reward").is_some());
    let mut old: Run = serde_json::from_value(value).expect("old save");
    assert_eq!(old.schnapps_reward(), None);
    old.apply(&content, RunAction::ChooseRoute(0))
        .expect("plays on");
    // A pending Schnaps the content no longer knows is dropped.
    let mut gone = run.clone();
    gone.schnapps_reward = Some(ConsumableId::new("no_such_schnaps"));
    let dropped = gone.drop_unknown_content(&content);
    assert_eq!(dropped, vec!["no_such_schnaps".to_string()]);
    assert_eq!(gone.schnapps_reward(), None);
}

#[test]
fn the_bot_keeps_the_better_schnaps() {
    let content = content();
    let mut run = start_at(&content, 5, 0, 0);
    // Schwenker (bot value 0) is the weakest: anything worth more takes
    // its slot.
    run.consumables = ids(&["road_beer", "snifter", "nightcap"]);
    run.schnapps_reward = Some(ConsumableId::new("shot"));
    assert_eq!(
        RunBot::schnapps_reward_action(&run),
        Some(RunAction::ChooseSchnappsReward { give_up: Some(1) })
    );
    // Not worth more than the weakest: sold.
    run.consumables = ids(&["road_beer", "road_beer", "nightcap"]);
    assert_eq!(
        RunBot::schnapps_reward_action(&run),
        Some(RunAction::ChooseSchnappsReward { give_up: None })
    );
    run.schnapps_reward = None;
    assert_eq!(RunBot::schnapps_reward_action(&run), None);
}

#[test]
fn every_schnaps_names_where_it_is_poured() {
    let content = content();
    let expected = [
        ("shot", ConsumableTarget::Skat),
        ("herbal", ConsumableTarget::Table),
        ("double_grain", ConsumableTarget::Table),
        ("rotgut", ConsumableTarget::Table),
        ("blue_curacao", ConsumableTarget::Card),
        ("clear_spirit", ConsumableTarget::Player),
        ("nightcap", ConsumableTarget::Player),
        ("eggnog", ConsumableTarget::Player),
        ("road_beer", ConsumableTarget::Player),
        ("snifter", ConsumableTarget::Map),
    ];
    for (id, target) in expected {
        let def = content.consumable(&ConsumableId::new(id)).expect(id);
        assert_eq!(def.target, target, "{id}");
    }
    assert_eq!(
        content.consumables.len(),
        expected.len(),
        "a new Schnaps needs its target here"
    );
    // In the view, with `Self` for the player.
    let mut run = start_at(&content, 5, 0, 0);
    run.consumables = ids(&["counter_beer", "snifter", "blue_curacao"]);
    let json = serde_json::to_value(run.view(&content)).expect("json");
    let targets: Vec<&str> = json["consumables"]
        .as_array()
        .expect("consumables")
        .iter()
        .map(|c| c["consumable"]["target"].as_str().expect("target"))
        .collect();
    assert_eq!(targets, ["Self", "Map", "Card"]);
    assert!(json["consumables"][0]["usable_now"].is_boolean());
    assert!(json["schnapps_reward"].is_null());
}

#[test]
fn usable_now_follows_the_phase() {
    let content = content();
    let mut run = start_at(&content, 5, 0, 0);
    run.consumables = ids(&["snifter", "clear_spirit", "herbal"]);
    // In the game before the declaration: Kräuter yes, Schwenker no,
    // Klarer only in the minus.
    let usable = |run: &Run| -> Vec<bool> {
        run.view(&content)
            .consumables
            .iter()
            .map(|c| c.usable_now)
            .collect()
    };
    assert_eq!(usable(&run), [false, false, true]);
    run.series.score = -5;
    assert_eq!(usable(&run), [false, true, true]);
    // At the route choice: only the Schwenker.
    run.series.score = 0;
    run.consumables = ids(&["snifter", "clear_spirit"]);
    end_pub(&mut run, &content, 1, true);
    if run.schnapps_reward().is_some() {
        run.apply(&content, RunAction::ChooseSchnappsReward { give_up: None })
            .expect("sell");
    }
    assert_eq!(run.phase(), RunPhase::Route);
    let usable = usable(&run);
    assert!(usable[0], "Schwenker at the route choice");
    assert!(!usable[1], "no Klarer outside a pub");
}

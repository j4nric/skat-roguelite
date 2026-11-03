//! The Kneipentour (ADR 0016): pubs won early and lost, lives, the boss
//! pub (ADR 0017: all games, lost ends the run, the relic pick), the route
//! choice per transport, the Spelunke and the names.

use super::tour::{RouteStop, Transport, Trip};
use super::*;
use crate::content::{Factor, MalusId, StopCategory};

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

/// End the current pub as if its games were played: `games` played, the
/// account at the target (won) or below (lost).
fn end_pub(run: &mut Run, content: &Content, games: u32, won: bool) -> Vec<RunEvent> {
    run.series.games_played = games;
    let target = i64::try_from(run.series.target).expect("small");
    run.series.score = if won { target } else { target - 1 };
    run.game = None;
    let mut events = Vec::new();
    run.after_game(content, &mut events).expect("after game");
    events
}

fn lives_changes(events: &[RunEvent]) -> Vec<i32> {
    events
        .iter()
        .filter_map(|e| match e {
            RunEvent::LivesChanged { change, .. } => Some(*change),
            _ => None,
        })
        .collect()
}

#[test]
fn a_pub_won_early_pays_for_the_games_left_and_opens_the_route_choice() {
    let content = content();
    let mut run = start_at(&content, 5, 0, 0);
    let coins = run.coins();
    let events = end_pub(&mut run, &content, 1, true);
    let economy = &content.run.economy;
    let payout = events
        .iter()
        .find_map(|e| match e {
            RunEvent::SeriesWon {
                payout, games_left, ..
            } => Some((*payout, *games_left)),
            _ => None,
        })
        .expect("won");
    assert_eq!(payout.1, 2, "two games left");
    assert_eq!(payout.0.base, economy.series_reward);
    assert_eq!(payout.0.games_left, 2 * economy.per_game_left);
    assert_eq!(run.coins(), coins + payout.0.total());
    assert_eq!(run.phase(), RunPhase::Route);
    assert_eq!(run.lives(), content.run.lives.start, "no life lost");
    let route = run.route().expect("route");
    assert_eq!((route.section, route.index), (0, 1));
    let transports: Vec<Transport> = route.options.iter().map(|o| o.transport).collect();
    assert_eq!(transports, Transport::ALL.to_vec());
    for option in &route.options {
        assert_eq!(option.stops.len(), 1, "one stop per way");
    }
    assert!(
        route
            .options
            .iter()
            .any(|o| o.stops.iter().any(|s| s.category == StopCategory::Tresen)),
        "a Tresen is always within reach"
    );
    assert_eq!(run.pub_results().len(), 1);
    assert!(run.pub_results()[0].won);
}

#[test]
fn a_lost_pub_costs_a_life_and_moves_on_without_reward() {
    let content = content();
    let mut run = start_at(&content, 5, 0, 1);
    let coins = run.coins();
    let events = end_pub(&mut run, &content, 3, false);
    assert!(
        !events
            .iter()
            .any(|e| matches!(e, RunEvent::SeriesWon { .. }))
    );
    assert!(events.contains(&RunEvent::PubLost {
        section: 0,
        index: 1,
        boss: None
    }));
    assert_eq!(lives_changes(&events), vec![-1]);
    assert_eq!(run.lives(), content.run.lives.start - 1);
    assert_eq!(run.coins(), coins, "no reward");
    assert_eq!(run.phase(), RunPhase::Route);
    let route = run.route().expect("route");
    assert_eq!((route.section, route.index), (0, 2), "on to the next pub");
    assert_eq!(run.stats().pubs_lost, 1);
}

#[test]
fn the_last_life_loses_the_run() {
    let content = content();
    let mut run = start_at(&content, 5, 1, 2);
    run.lives = 1;
    let events = end_pub(&mut run, &content, 3, false);
    assert_eq!(run.phase(), RunPhase::Lost);
    assert_eq!(run.lives(), 0);
    assert!(events.iter().any(|e| matches!(
        e,
        RunEvent::RunLost {
            section: 1,
            index: 2,
            ..
        }
    )));
}

#[test]
fn a_lost_boss_pub_ends_the_run_whatever_lives_are_left() {
    let content = content();
    let mut run = start_at(&content, 5, 0, 4);
    let boss = run.series().boss.clone().expect("boss");
    // Not even "Schutzengel" helps in a boss pub (ADR 0017).
    run.relics.push(RelicId::new("guardian_angel"));
    let events = end_pub(&mut run, &content, 3, false);
    assert_eq!(run.phase(), RunPhase::Lost);
    assert_eq!(run.lives(), content.run.lives.start, "lives do not matter");
    assert!(lives_changes(&events).is_empty());
    assert!(
        !events
            .iter()
            .any(|e| matches!(e, RunEvent::LifeSpared { .. }))
    );
    assert!(events.iter().any(|e| matches!(
        e,
        RunEvent::RunLost { section: 0, index: 4, boss: Some(b), .. } if *b == boss
    )));
    assert!(run.route().is_none());
}

#[test]
fn a_boss_pub_plays_all_three_games() {
    let content = content();
    let mut run = start_at(&content, 5, 0, 4);
    // At the target after the first game: the second is dealt anyway.
    let events = end_pub(&mut run, &content, 1, true);
    assert!(
        !events
            .iter()
            .any(|e| matches!(e, RunEvent::SeriesWon { .. }))
    );
    assert_eq!(run.phase(), RunPhase::Game);
    assert!(run.game().is_some());
    assert_eq!(run.series().games_played, 1);
}

/// Win the boss pub of district 1 after three games.
fn win_boss_pub(content: &Content, seed: u64) -> (Run, Vec<RunEvent>) {
    let mut run = start_at(content, seed, 0, 4);
    let events = end_pub(&mut run, content, 3, true);
    (run, events)
}

#[test]
fn a_won_boss_pub_pays_double_gives_a_life_and_a_pick_of_three_relics() {
    let content = content();
    let max = content.run.lives.max;
    let (mut run, events) = win_boss_pub(&content, 5);
    assert_eq!(lives_changes(&events), vec![1]);
    assert_eq!(run.lives(), content.run.lives.start + 1);
    let payout = events
        .iter()
        .find_map(|e| match e {
            RunEvent::SeriesWon { payout, .. } => Some(*payout),
            _ => None,
        })
        .expect("won");
    assert_eq!(payout.games_left, 0, "no games left in a boss pub");
    assert!(payout.boss > 0);
    assert_eq!(payout.total(), 2 * payout.boss, "double");
    assert_eq!(run.phase(), RunPhase::BossReward);
    let offered = events
        .iter()
        .find_map(|e| match e {
            RunEvent::BossRewardOffered { relics } => Some(relics.clone()),
            _ => None,
        })
        .expect("offered");
    assert_eq!(offered.len(), 3);
    for id in &offered {
        let rarity = content.relic(id).expect("relic").rarity;
        assert!(rarity >= crate::content::Rarity::Uncommon, "{id}");
        assert!(!run.relics().contains(id));
    }
    let view = run.view(&content).boss_reward.expect("pick view");
    assert_eq!(view.offers.len(), 3);
    assert!(!view.slots_full);
    // Taking one: into a free slot, then the route to the next district.
    let events = run
        .apply(
            &content,
            RunAction::ChooseBossRelic {
                offer: Some(1),
                replace: None,
            },
        )
        .expect("take");
    assert!(events.contains(&RunEvent::RelicGained {
        relic: offered[1].clone()
    }));
    assert!(run.relics().contains(&offered[1]));
    assert_eq!(run.phase(), RunPhase::Route);
    let route = run.route().expect("route");
    assert_eq!((route.section, route.index), (1, 0), "the next district");
    assert_eq!(
        run.apply(
            &content,
            RunAction::ChooseBossRelic {
                offer: None,
                replace: None
            }
        ),
        Err(RunError::WrongPhase)
    );
    // Lives capped at the maximum.
    let mut run = start_at(&content, 5, 0, 4);
    run.lives = max;
    let events = end_pub(&mut run, &content, 3, true);
    assert!(lives_changes(&events).is_empty());
    assert_eq!(run.lives(), max);
    // The last boss wins the run, without a pick.
    let mut run = start_at(&content, 5, content.run.sections() - 1, 4);
    let events = end_pub(&mut run, &content, 3, true);
    assert_eq!(run.phase(), RunPhase::Won);
    assert!(events.iter().any(|e| matches!(e, RunEvent::RunWon { .. })));
    assert!(run.boss_reward().is_none());
}

#[test]
fn the_boss_pick_with_full_slots_replaces_a_relic_or_is_passed() {
    let content = content();
    let (mut run, _) = win_boss_pub(&content, 6);
    let offered = run.boss_reward().expect("pick").relics.clone();
    let slots = usize::try_from(content.relic_slots(run.character())).expect("slots");
    let fillers: Vec<RelicId> = content
        .relics
        .keys()
        .filter(|id| !offered.contains(id))
        .take(slots)
        .cloned()
        .collect();
    run.relics = fillers.clone();
    assert!(run.view(&content).boss_reward.expect("view").slots_full);
    let take = RunAction::ChooseBossRelic {
        offer: Some(0),
        replace: None,
    };
    assert_eq!(run.apply(&content, take), Err(RunError::NoFreeSlot));
    // Selling the third relic for half its price makes room.
    let mut replaced = run.clone();
    let coins = replaced.coins();
    let price = sell_price(content.relic(&fillers[2]).expect("relic").price);
    let events = replaced
        .apply(
            &content,
            RunAction::ChooseBossRelic {
                offer: Some(0),
                replace: Some(2),
            },
        )
        .expect("replace");
    assert!(events.contains(&RunEvent::RelicSold {
        relic: fillers[2].clone(),
        price
    }));
    assert_eq!(replaced.coins(), coins + price);
    assert_eq!(replaced.relics().len(), slots);
    assert!(replaced.relics().contains(&offered[0]));
    assert!(!replaced.relics().contains(&fillers[2]));
    assert_eq!(replaced.phase(), RunPhase::Route);
    // Or none at all.
    let events = run
        .apply(
            &content,
            RunAction::ChooseBossRelic {
                offer: None,
                replace: None,
            },
        )
        .expect("pass");
    assert!(events.contains(&RunEvent::BossRewardDeclined));
    assert_eq!(run.relics(), fillers.as_slice());
    assert_eq!(run.phase(), RunPhase::Route);
}

#[test]
fn the_bot_picks_a_boss_relic() {
    let content = content();
    let (mut run, _) = win_boss_pub(&content, 7);
    let action = bot::RunBot.next_action(&run, &content).expect("action");
    assert!(matches!(
        action,
        RunAction::ChooseBossRelic {
            offer: Some(_),
            replace: None
        }
    ));
    run.apply(&content, action).expect("legal");
    assert_eq!(run.phase(), RunPhase::Route);
}

/// A run at the route choice to pub 1 with the given stops per transport.
fn at_route(content: &Content, stops: [Vec<RouteStop>; 3]) -> Run {
    let mut run = start_at(content, 5, 0, 0);
    end_pub(&mut run, content, 1, true);
    let route = run.route.as_mut().expect("route");
    for (option, stops) in route.options.iter_mut().zip(stops) {
        option.stops = stops;
        option.spelunke = None;
    }
    run
}

fn stop(category: StopCategory, encounter: Option<&str>) -> RouteStop {
    RouteStop {
        category,
        encounter: encounter.map(EncounterId::new),
    }
}

#[test]
fn every_transport_leads_over_one_shown_stop_for_free() {
    let content = content();
    let event = || stop(StopCategory::Event, Some("last_round"));
    let rest = || stop(StopCategory::Rest, Some("snack_bar"));
    let upgrade = || stop(StopCategory::Upgrade, Some("workshop"));
    let mut run = at_route(&content, [vec![event()], vec![upgrade()], vec![rest()]]);
    let view = run.view(&content).tour.route.expect("route view");
    assert_eq!(view.next.section, 0);
    assert_eq!(view.next.index, 1);
    assert_eq!(view.next.target, content.run.targets[0][1]);
    assert_eq!(view.next.guests.len(), 2, "the guests at the next table");
    assert!(!view.next.name.is_empty());
    let shown: Vec<StopCategory> = view.options.iter().map(|o| o.stops[0].category).collect();
    assert_eq!(
        shown,
        vec![
            StopCategory::Event,
            StopCategory::Upgrade,
            StopCategory::Rest
        ]
    );
    let transports: Vec<Transport> = view.options.iter().map(|o| o.transport).collect();
    assert_eq!(transports, Transport::ALL.to_vec());
    assert!(view.options.iter().all(|o| o.can_choose && o.minutes > 0));
    assert_eq!(view.options[1].id, "d0-p1-r1");
    // Without Groschen every way is open, and none costs anything.
    run.coins = 0;
    let mut train = run.clone();
    train
        .apply(&content, RunAction::ChooseRoute(1))
        .expect("train");
    assert_eq!(train.coins(), 0);
    assert_eq!(train.encounter().map(|e| e.id.as_str()), Some("workshop"));
    // Fahrrad: the Rast, then the pub.
    run.lives = 2;
    let events = run
        .apply(&content, RunAction::ChooseRoute(2))
        .expect("bike");
    assert!(events.iter().any(|e| matches!(
        e,
        RunEvent::RouteChosen {
            transport: Transport::Bike,
            spelunke: false,
            ..
        }
    )));
    assert_eq!(run.encounter().map(|e| e.id.as_str()), Some("snack_bar"));
    let events = run
        .apply(
            &content,
            RunAction::ChooseEncounter {
                option: 0,
                target: None,
            },
        )
        .expect("rest");
    assert_eq!(lives_changes(&events), vec![1]);
    assert_eq!(run.lives(), 3);
    assert!(events.iter().any(|e| matches!(
        e,
        RunEvent::SeriesStarted {
            section: 0,
            index: 1,
            ..
        }
    )));
    assert_eq!(run.phase(), RunPhase::Game);
}

#[test]
fn the_tresen_on_the_way_leads_to_the_pub() {
    let content = content();
    let tresen = || stop(StopCategory::Tresen, None);
    let mut run = at_route(&content, [vec![tresen()], vec![tresen()], vec![tresen()]]);
    run.apply(&content, RunAction::ChooseRoute(0))
        .expect("walk");
    assert_eq!(run.phase(), RunPhase::Shop);
    assert!(run.view(&content).tour.trip.is_some());
    run.apply(&content, RunAction::LeaveShop).expect("leave");
    assert_eq!(run.phase(), RunPhase::Game);
    assert_eq!(run.series().index, 1);
}

/// A run at the route choice whose first way leads into a Spelunke with
/// this malus.
fn spelunke_run(content: &Content, malus: &str) -> Run {
    let tresen = || stop(StopCategory::Tresen, None);
    let mut run = at_route(content, [Vec::new(), vec![tresen()], vec![tresen()]]);
    let route = run.route.as_mut().expect("route");
    route.options[0].spelunke = Some(Spelunke {
        name: "Spelunke Zum Test".into(),
        malus: MalusId::new(malus),
        malus_options: Vec::new(),
    });
    run
}

use super::tour::Spelunke;

#[test]
fn spelunken_come_on_routes_but_never_before_a_boss_pub() {
    let mut content = content();
    content.run.spelunke.chance = Factor::ONE;
    let mut run = start_at(&content, 9, 0, 0);
    end_pub(&mut run, &content, 1, true);
    let route = run.route().expect("route");
    let spelunken = route
        .options
        .iter()
        .filter(|o| o.spelunke.is_some())
        .count();
    assert_eq!(spelunken, 1, "one way leads into a Spelunke");
    let view = run.view(&content).tour.route.expect("view");
    let preview = view
        .options
        .iter()
        .find_map(|o| o.spelunke.clone())
        .expect("preview");
    assert!(preview.malus.is_some(), "the malus is shown in advance");
    assert_eq!(
        preview.target,
        content.run.spelunke.target_factor.apply(view.next.target)
    );
    assert!(preview.id.ends_with("-s"));
    // Before the boss pub never.
    let mut run = start_at(&content, 9, 0, 3);
    end_pub(&mut run, &content, 1, true);
    assert!(
        run.route()
            .expect("route")
            .options
            .iter()
            .all(|o| o.spelunke.is_none())
    );
}

#[test]
fn a_won_spelunke_pays_a_relic_on_top() {
    let content = content();
    let mut run = spelunke_run(&content, "schnapps_locked");
    let regular = content.run.targets[0][1];
    let events = run
        .apply(&content, RunAction::ChooseRoute(0))
        .expect("spelunke");
    assert!(events.iter().any(|e| matches!(
        e,
        RunEvent::SeriesStarted { malus: Some(m), .. } if m.as_str() == "schnapps_locked"
    )));
    assert_eq!(
        run.series().target,
        content.run.spelunke.target_factor.apply(regular)
    );
    assert!(
        run.sources(&content, false)
            .iter()
            .any(|s| s.source == Source::Malus(MalusId::new("schnapps_locked")))
    );
    let coins = run.coins();
    let events = end_pub(&mut run, &content, 2, true);
    assert!(
        events
            .iter()
            .any(|e| matches!(e, RunEvent::SeriesWon { .. }))
    );
    assert!(run.coins() > coins, "the normal reward");
    assert_eq!(run.phase(), RunPhase::Encounter, "the loot");
    let loot = run.encounter().expect("loot").clone();
    assert_eq!(loot.id.as_str(), "spelunke_loot");
    let EncounterChoice::TakeRelic(relic) = loot.options[0].choice.clone() else {
        panic!("a relic first: {:?}", loot.options);
    };
    assert!(matches!(
        loot.options[1].choice,
        EncounterChoice::TakeCoins(_)
    ));
    let events = run
        .apply(
            &content,
            RunAction::ChooseEncounter {
                option: 0,
                target: None,
            },
        )
        .expect("take");
    assert!(events.contains(&RunEvent::RelicGained {
        relic: relic.clone()
    }));
    assert!(run.relics().contains(&relic));
    assert_eq!(run.phase(), RunPhase::Route);
    assert_eq!(run.route().map(|r| r.index), Some(2));
    assert!(run.pub_results()[1].spelunke.is_some());
    // On the map the Spelunke is a place of its own.
    let district = &run.view(&content).tour.districts[0];
    assert_eq!(district.spelunken.len(), 1);
    assert_eq!(district.spelunken[0].id, "d0-p1-s");
    assert_eq!(district.spelunken[0].name, "Spelunke Zum Test");
    assert_eq!(district.spelunken[0].state, super::tour::PubState::Won);
    assert_eq!(district.pubs[1].state, super::tour::PubState::Skipped);
    assert_ne!(district.pubs[1].name, "Spelunke Zum Test");
}

#[test]
fn spelunke_maluses_bite_for_their_pub_only() {
    let content = content();
    // Kein Einmischen.
    let mut run = spelunke_run(&content, "no_redeal");
    run.apply(&content, RunAction::ChooseRoute(0)).expect("go");
    run.series.extra_redeals = 1;
    assert_eq!(run.redeals_left(&content), 0);
    assert_eq!(
        run.clone().apply(&content, RunAction::Redeal),
        Err(RunError::NoRedealLeft)
    );
    // Kein Skat.
    let mut run = spelunke_run(&content, "hand_only");
    run.apply(&content, RunAction::ChooseRoute(0)).expect("go");
    assert!(run.requires_hand());
    // Schnapsverbot.
    let mut run = spelunke_run(&content, "schnapps_locked");
    run.apply(&content, RunAction::ChooseRoute(0)).expect("go");
    run.consumables = vec![ConsumableId::new("double_grain")];
    assert_eq!(
        run.clone().apply(
            &content,
            RunAction::UseConsumable {
                slot: 0,
                target: None,
                suit: None
            }
        ),
        Err(RunError::ConsumablesLocked)
    );
    // Klebrige Karten: two cards cursed for this pub, no Groschen for it.
    let mut run = spelunke_run(&content, "cursed_cards");
    let coins = run.coins();
    let events = run.apply(&content, RunAction::ChooseRoute(0)).expect("go");
    assert_eq!(run.coins(), coins);
    assert!(
        !events
            .iter()
            .any(|e| matches!(e, RunEvent::CoinsGained { .. }))
    );
    let cursed: Vec<u32> = run.series().temp_curses.iter().map(|(c, _)| *c).collect();
    assert_eq!(cursed.len(), 2);
    for card in &cursed {
        assert!(!run.edition().get(*card).expect("card").mods.is_empty());
    }
    end_pub(&mut run, &content, 3, false);
    for card in &cursed {
        assert!(
            run.edition().get(*card).expect("card").mods.is_empty(),
            "clean again after the pub"
        );
    }
    assert_eq!(run.phase(), RunPhase::Route);
}

#[test]
fn the_workshop_cleanses_a_chosen_card() {
    let content = content();
    let mut run = at_route(
        &content,
        [
            vec![stop(StopCategory::Upgrade, Some("workshop"))],
            Vec::new(),
            Vec::new(),
        ],
    );
    let card = run.edition().cards()[3].id;
    run.edition
        .add_mod(&content, card, &CardModId::new("battered"), 5)
        .expect("curse");
    run.apply(&content, RunAction::ChooseRoute(0))
        .expect("walk");
    let encounter = run.encounter().expect("workshop").clone();
    let cleanse = encounter
        .options
        .iter()
        .position(|o| o.choice == EncounterChoice::CleanseChosenCard)
        .expect("cleanse");
    let view = run.view(&content).encounter.expect("view");
    assert!(view.options[cleanse].targets.contains(&card));
    run.apply(
        &content,
        RunAction::ChooseEncounter {
            option: cleanse,
            target: Some(card),
        },
    )
    .expect("cleanse");
    assert!(run.edition().get(card).expect("card").mods.is_empty());
    assert_eq!(run.phase(), RunPhase::Game);
}

#[test]
fn the_city_is_named_by_the_seed() {
    let content = content();
    let a = start_at(&content, 77, 0, 0);
    let b = start_at(&content, 77, 0, 0);
    let c = start_at(&content, 78, 0, 0);
    assert_eq!(a.city(&content), b.city(&content));
    assert_ne!(a.city(&content), c.city(&content));
    let city = a.city(&content);
    assert_eq!(city.len(), 3);
    for (i, district) in city.iter().enumerate() {
        assert!(!district.name.is_empty());
        assert!(
            city[..i].iter().all(|d| d.name != district.name),
            "distinct"
        );
        assert!(
            city[..i].iter().all(|d| d.look != district.look),
            "one district of each look"
        );
        assert_eq!(district.pubs.len(), 5);
        assert!(
            district
                .pubs
                .iter()
                .all(|p| !p.is_empty() && !p.contains('{'))
        );
    }
    let view = a.view(&content).tour;
    assert_eq!(view.lives, content.run.lives.start);
    assert_eq!(view.max_lives, content.run.lives.max);
    assert_eq!(view.districts[0].pubs[0].id, "d0-p0");
    assert_eq!(
        view.districts[0].pubs[0].state,
        super::tour::PubState::Current
    );
    assert!(view.districts[0].pubs[4].is_boss);
    assert!(view.districts[0].boss.is_some(), "the first boss is known");
    assert_eq!(view.map_seed, b.view(&content).tour.map_seed);
    assert_eq!(view.current.expect("current").name, city[0].pubs[0]);
    let looks: Vec<_> = view.districts.iter().map(|d| d.look).collect();
    assert_eq!(
        looks,
        city.iter().map(|d| d.look).collect::<Vec<_>>(),
        "the view shows each district's look"
    );
    let json = serde_json::to_value(&view.districts[0]).expect("json");
    assert!(json["look"].is_string(), "look in the view: {json}");
}

#[test]
fn the_bot_takes_routes_and_plays_whole_tours() {
    let mut content = content();
    content.run.spelunke.chance = Factor::from_permille(500);
    let mut routes = 0;
    let mut spelunken = 0;
    for seed in 0..6 {
        let (mut run, _) = Run::new(&content, seed).expect("run");
        for _ in 0..20_000 {
            let Some(action) = bot::RunBot.next_action(&run, &content) else {
                break;
            };
            let events = run.apply(&content, action).expect("bot");
            for e in &events {
                if let RunEvent::RouteChosen { spelunke, .. } = e {
                    routes += 1;
                    spelunken += usize::from(*spelunke);
                }
            }
        }
        assert!(matches!(run.phase(), RunPhase::Won | RunPhase::Lost));
    }
    assert!(routes >= 6, "{routes} routes");
    let _ = spelunken;
    // The trip type round-trips through a save.
    let trip = Trip {
        section: 0,
        index: 1,
        transport: Transport::Bike,
        stops: vec![stop(StopCategory::Tresen, None)],
        spelunke: None,
    };
    let text = serde_json::to_string(&trip).expect("json");
    assert_eq!(serde_json::from_str::<Trip>(&text).expect("back"), trip);
}

/// Bring the saved runs of `web/e2e/fixtures` from save version 2 to the
/// Kneipentour (version 3, ADR 0016), keeping their scenes: the boss
/// series (old index 2) becomes the boss pub (4), the series before it (1)
/// the pub before the boss pub (3); sections past the last district are
/// the last one. A Tresen or encounter gets a way with no further stop to
/// the pub after it, so leaving starts that pub as before. The web app
/// itself discards older runs (`OutdatedSave`). Run by hand: `cargo test
/// -p skat_core regenerate_tour_fixtures -- --ignored`.
#[test]
#[ignore = "rewrites web/e2e/fixtures"]
fn regenerate_tour_fixtures() {
    let content = content();
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../web/e2e/fixtures");
    let mut names: Vec<_> = std::fs::read_dir(&dir)
        .expect("fixtures")
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|x| x == "json"))
        .collect();
    names.sort();
    for path in names {
        let text = std::fs::read_to_string(&path).expect("read");
        let save: serde_json::Value = serde_json::from_str(&text).expect("json");
        if save["version"] != 2 || !save["run"].is_object() {
            continue;
        }
        let mut run: Run = serde_json::from_value(save["run"].clone()).expect("run");
        let last = content.run.sections() - 1;
        run.series.section = run.series.section.min(last);
        run.series.index = match run.series.index {
            2 => 4,
            1 => 3,
            other => other,
        };
        if matches!(run.phase, RunPhase::Shop | RunPhase::Encounter) {
            let (section, index) = run.next_series_position();
            run.trip = Some(Trip {
                section,
                index,
                transport: Transport::Walk,
                stops: Vec::new(),
                spelunke: None,
            });
        }
        let out = serde_json::json!({ "version": 3, "run": run });
        std::fs::write(&path, serde_json::to_string(&out).expect("json")).expect("write");
        println!("upgraded {}", path.display());
    }
}

#[test]
fn a_save_with_a_removed_boss_meets_its_stand_in() {
    let content = content();
    let mut run = start_at(&content, 5, 0, 4);
    run.section_boss = Some(BossId::new("kontra_hans"));
    run.series.boss = Some(BossId::new("kontra_hans"));
    if let Some(active) = run.game.as_mut() {
        active.boss_rule = Some(BossId::new("kontra_hans"));
    }
    let saved = serde_json::to_string(&run).expect("save");
    let mut loaded: Run = serde_json::from_str(&saved).expect("load");
    loaded.drop_unknown_content(&content);
    let pusher = Some(BossId::new("the_pusher"));
    assert_eq!(loaded.section_boss(), pusher.as_ref());
    assert_eq!(loaded.series().boss, pusher);
    assert_eq!(loaded.game().and_then(|a| a.boss_rule.clone()), pusher);
    for (old, new) in RETIRED_BOSSES {
        assert!(content.boss(&BossId::new(old)).is_none(), "{old}");
        assert!(content.boss(&BossId::new(new)).is_some(), "{new}");
    }
    // The run goes on; the next deal follows the stand-in's rule.
    assert!(loaded.view(&content).series.boss.is_some());
    loaded.deal(&content, true, &mut Vec::new()).expect("deal");
    assert!(loaded.requires_hand());
}

/// A run at the table of the boss pub of district 1 against `boss`, at the
/// player's first decision: pub 4 won, the way taken straight there.
fn at_boss_table(content: &Content, seed: u64, boss: &str) -> Run {
    let mut run = start_at(content, seed, 0, 3);
    run.section_boss = Some(BossId::new(boss));
    end_pub(&mut run, content, 3, true);
    run.walk_to_pub(content);
    let mut events = Vec::new();
    run.play_ai(content, &mut events).expect("AI");
    assert_eq!(run.series().boss, Some(BossId::new(boss)));
    run
}

#[test]
fn the_boss_table_seats_both_figures_with_the_boss_back() {
    let content = content();
    let run = at_boss_table(&content, 11, "twins");
    assert_eq!(run.owner_of_seat(BOSS_SEAT), crate::deck::Owner::Boss);
    assert_eq!(
        run.owner_of_seat(PARTNER_SEAT),
        crate::deck::Owner::BossPartner
    );
    let view = run.view(&content);
    let boss = view.series.boss.expect("boss");
    let names: Vec<&str> = boss.figures.iter().map(|f| f.name.as_str()).collect();
    assert_eq!(names, ["Fritz", "Franz"]);
    let backs = view.game.expect("game").backs;
    assert_eq!(backs[1].as_deref(), Some("twins"));
    assert_eq!(backs[2].as_deref(), Some("twins"));
}

#[test]
fn the_oma_under_the_old_man_curses_once_her_rule_comes() {
    let content = content();
    let mut run = start_at(&content, 5, content.run.sections() - 1, 4);
    let old_man = content.boss(&BossId::new("the_old_man")).expect("old man");
    let oma = old_man
        .rotation
        .iter()
        .position(|b| b.as_str() == "grandma")
        .expect("in the rotation");
    let before = run.edition().clone();
    assert!(!run.series().boss_cursed, "no curse before her rule");
    run.series.games_played = u32::try_from(oma).expect("small");
    run.series.redeals_used = 0;
    run.deal(&content, true, &mut Vec::new()).expect("deal");
    assert!(run.series().boss_cursed);
    assert_eq!(run.series().boss_curses.len(), 16);
    let mut ended = Vec::new();
    run.end_pub(&content, &mut ended);
    assert_eq!(run.edition(), &before);
}

/// Fixture tool for the e2e test (web/e2e/fixtures/boss-*.json): saves at
/// the boss pub of ADR 0017. Run with `cargo test -p skat_core
/// regenerate_boss_fixtures -- --ignored`.
#[test]
#[ignore = "rewrites web/e2e/fixtures"]
fn regenerate_boss_fixtures() {
    let content = content();
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../web/e2e/fixtures");
    let write = |name: &str, run: &Run| {
        let out = serde_json::json!({ "version": 3, "run": run });
        let path = dir.join(name);
        std::fs::write(&path, serde_json::to_string(&out).expect("json")).expect("write");
        println!("wrote {}", path.display());
    };
    // The route to the boss pub: both figures named before it.
    let mut route = start_at(&content, 11, 0, 3);
    route.section_boss = Some(BossId::new("the_pusher"));
    end_pub(&mut route, &content, 3, true);
    assert_eq!(route.phase(), RunPhase::Route);
    write("boss-route.json", &route);
    // At the table of Die Zwillinge.
    write("boss-table.json", &at_boss_table(&content, 11, "twins"));
    // Die Oma's curses on the table.
    write("boss-oma.json", &at_boss_table(&content, 12, "grandma"));
    // The last game of a boss pub that cannot be won any more: lost, the
    // run ends with lives left.
    let mut lost = at_boss_table(&content, 13, "the_pusher");
    lost.series.games_played = 2;
    lost.series.score = -400;
    write("boss-lost.json", &lost);
    // The relic pick after the won boss pub, with a free slot and full.
    let mut won = at_boss_table(&content, 14, "black_peter");
    end_pub(&mut won, &content, 3, true);
    assert_eq!(won.phase(), RunPhase::BossReward);
    write("boss-reward.json", &won);
    let offered = won.boss_reward().expect("pick").relics.clone();
    let slots = usize::try_from(content.relic_slots(won.character())).expect("slots");
    won.relics = content
        .relics
        .values()
        .filter(|r| !offered.contains(&r.id) && !won.locked.contains(&r.id))
        .filter(|r| r.kind == RelicKind::HouseRule)
        .take(slots)
        .map(|r| r.id.clone())
        .collect();
    write("boss-reward-full.json", &won);
}

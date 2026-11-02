//! The Kneipentour (ADR 0016): pubs won early and lost, lives, boss
//! replays, the route choice per transport, the Spelunke and the names.

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
fn a_lost_boss_pub_costs_a_life_and_is_played_again() {
    let content = content();
    let mut run = start_at(&content, 5, 0, 4);
    let boss = run.series().boss.clone().expect("boss");
    let events = end_pub(&mut run, &content, 3, false);
    assert_eq!(lives_changes(&events), vec![-1]);
    assert_eq!(run.phase(), RunPhase::Game, "straight back to the table");
    assert_eq!((run.series().section, run.series().index), (0, 4));
    assert_eq!(run.series().boss.as_ref(), Some(&boss));
    assert_eq!(run.series().games_played, 0);
    assert_eq!(run.series().score, 0);
    assert!(events.iter().any(|e| matches!(
        e,
        RunEvent::SeriesStarted { index: 4, boss: Some(b), .. } if *b == boss
    )));
}

#[test]
fn a_won_boss_pub_gives_a_life_up_to_the_maximum() {
    let content = content();
    let max = content.run.lives.max;
    let mut run = start_at(&content, 5, 0, 4);
    let events = end_pub(&mut run, &content, 2, true);
    assert_eq!(lives_changes(&events), vec![1]);
    assert_eq!(run.lives(), content.run.lives.start + 1);
    assert_eq!(run.phase(), RunPhase::Route);
    let route = run.route().expect("route");
    assert_eq!((route.section, route.index), (1, 0), "the next district");
    // Capped.
    let mut run = start_at(&content, 5, 0, 4);
    run.lives = max;
    let events = end_pub(&mut run, &content, 2, true);
    assert!(lives_changes(&events).is_empty());
    assert_eq!(run.lives(), max);
    // The last boss wins the run.
    let mut run = start_at(&content, 5, content.run.sections() - 1, 4);
    let events = end_pub(&mut run, &content, 2, true);
    assert_eq!(run.phase(), RunPhase::Won);
    assert!(events.iter().any(|e| matches!(e, RunEvent::RunWon { .. })));
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
    for (i, (district, pubs)) in city.iter().enumerate() {
        assert!(!district.is_empty());
        assert!(city[..i].iter().all(|(d, _)| d != district), "distinct");
        assert_eq!(pubs.len(), 5);
        assert!(pubs.iter().all(|p| !p.is_empty() && !p.contains('{')));
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
    assert_eq!(view.current.expect("current").name, city[0].1[0]);
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

//! "Die lange Nacht" (ADR 0020): entering after der Alte, the growth of the
//! Zielwerte, the Verschärfungen (drawn without repeat, stacking, each
//! rule), the night bosses and relics, losing in the night, the record
//! and the night's unlocks.

use super::meta::MetaState;
use super::night::VERSCHAERFUNGEN_COUNTER;
use super::*;
use crate::content::Source;
use crate::content::{MalusId, NightRule, UnlockId, VerschaerfungId};

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

/// End the current pub as if its games were played (see `tour_tests`).
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

/// A run that just beat der Alte, with all its events.
fn won_run(content: &Content, seed: u64) -> (Run, Vec<RunEvent>) {
    let start = RunStart {
        section: 2,
        index: 4,
        ..RunStart::default()
    };
    let (mut run, mut events) = Run::new_with(content, seed, &start).expect("start");
    events.extend(end_pub(&mut run, content, 3, true));
    assert_eq!(run.phase(), RunPhase::Won);
    (run, events)
}

/// Into the night: the route to Stadtteil 4.
fn night_run(content: &Content, seed: u64) -> (Run, Vec<RunEvent>) {
    let (mut run, mut events) = won_run(content, seed);
    events.extend(run.apply(content, RunAction::EnterNight).expect("night"));
    (run, events)
}

/// Take the first way and leave every stop until the next pub is dealt.
fn walk_to_pub(run: &mut Run, content: &Content) -> Vec<RunEvent> {
    let mut events = run
        .apply(content, RunAction::ChooseRoute(0))
        .expect("route");
    for _ in 0..8 {
        if run.phase() == RunPhase::Game {
            return events;
        }
        let action = match run.phase() {
            RunPhase::Shop => RunAction::LeaveShop,
            _ => super::bot::RunBot.next_action(run, content).expect("bot"),
        };
        events.extend(run.apply(content, action).expect("stop"));
    }
    assert_eq!(run.phase(), RunPhase::Game);
    events
}

fn force(run: &mut Run, ids: &[&str]) {
    run.night.as_mut().expect("night").verschaerfungen =
        ids.iter().map(|id| VerschaerfungId::new(id)).collect();
}

#[test]
fn the_win_stays_and_the_night_opens_on_request() {
    let content = content();
    let (run, events) = won_run(&content, 7);
    assert!(events.iter().any(|e| matches!(e, RunEvent::RunWon { .. })));
    assert!(run.view(&content).can_enter_night);
    assert!(run.night().is_none());
    // Not before the run is won.
    let mut early = start_at(&content, 7, 0, 0);
    assert_eq!(
        early.apply(&content, RunAction::EnterNight),
        Err(RunError::WrongPhase)
    );
    let (run, events) = night_run(&content, 7);
    assert!(events.contains(&RunEvent::NightStarted {
        character: Character::Declarer
    }));
    assert!(
        events
            .iter()
            .any(|e| matches!(e, RunEvent::VerschaerfungDrawn { section: 3, .. }))
    );
    assert!(events.contains(&RunEvent::RouteOpened {
        section: 3,
        index: 0
    }));
    assert_eq!(run.phase(), RunPhase::Route);
    let view = run.view(&content);
    assert!(!view.can_enter_night);
    let night = view.night.expect("night view");
    assert_eq!(night.district, 4);
    assert_eq!(night.verschaerfungen.len(), 1);
    assert_eq!(night.announced.as_ref(), night.verschaerfungen.first());
    let jump = night
        .verschaerfungen
        .iter()
        .any(|v| v.id.as_str() == "target_jump");
    let first = u64::from(run.night_config(&content).growth_of(1).permille());
    assert_eq!(
        night.target_permille,
        if jump { first * 11 / 10 } else { first }
    );
    // Only once.
    let mut again = run.clone();
    assert_eq!(
        again.apply(&content, RunAction::EnterNight),
        Err(RunError::WrongPhase)
    );
    // The city names the night district.
    assert_eq!(run.tour_view(&content).districts.len(), 4);
    assert_eq!(
        run.counters().get(VERSCHAERFUNGEN_COUNTER).copied(),
        Some(1)
    );
}

#[test]
fn night_targets_grow_from_the_last_district() {
    let content = content();
    let (mut run, _) = night_run(&content, 11);
    force(&mut run, &["fewer_redeals"]);
    let last = &content.run.targets[2];
    for index in 0..4 {
        let base = last[index as usize];
        assert_eq!(
            run.pub_target(&content, 3, index, None),
            Some(run.night_config(&content).growth_of(1).apply(base))
        );
        assert_eq!(
            run.pub_target(&content, 5, index, None),
            Some(run.night_config(&content).grow(base, 3))
        );
    }
    // The boss with its own factor, then the growth.
    let boss = BossId::new("twins");
    let plain = series_target(&content, Character::Declarer, 2, 4, Some(&boss)).expect("boss");
    assert_eq!(
        run.pub_target(&content, 4, 4, Some(&boss)),
        Some(run.night_config(&content).grow(plain, 2))
    );
    // "Zielwert-Sprung" on top, stacking.
    force(&mut run, &["target_jump", "target_jump"]);
    let grown = run.night_config(&content).grow(last[0], 1);
    let jump = crate::content::Factor::from_permille(1100);
    assert_eq!(
        run.pub_target(&content, 3, 0, None),
        Some(jump.apply(jump.apply(grown)))
    );
    // The normal districts are untouched.
    assert_eq!(run.pub_target(&content, 2, 0, None), Some(last[0]));
}

#[test]
fn the_first_night_pub_plays_its_grown_target() {
    let content = content();
    let (mut run, _) = night_run(&content, 13);
    force(&mut run, &["fewer_redeals"]);
    walk_to_pub(&mut run, &content);
    assert_eq!((run.series().section, run.series().index), (3, 0));
    let base = content.run.targets[2][0];
    assert_eq!(
        run.series().target,
        run.night_config(&content).grow(base, 1)
    );
    // One Einmischen less than in Stadtteil 3.
    assert_eq!(
        run.redeals_left(&content),
        content.redeals_in(Character::Declarer, 3) - 1
    );
    let boss = run.section_boss().cloned().expect("night boss");
    assert_eq!(run.night().expect("night").bosses, vec![boss]);
}

#[test]
fn verschaerfungen_come_one_per_district_without_repeat_until_all_were_drawn() {
    let content = content();
    let (mut run, _) = night_run(&content, 17);
    let all = content.verschaerfungen.len();
    let mut events = Vec::new();
    for section in 4..(3 + 2 * u32::try_from(all).expect("small")) {
        run.draw_verschaerfung(&content, section, &mut events);
    }
    let drawn = &run.night().expect("night").verschaerfungen;
    assert_eq!(drawn.len(), 2 * all);
    let mut first: Vec<_> = drawn[..all].to_vec();
    first.sort();
    first.dedup();
    assert_eq!(first.len(), all, "every one before any twice");
    let mut second: Vec<_> = drawn[all..].to_vec();
    second.sort();
    second.dedup();
    assert_eq!(second.len(), all, "the second round again without repeat");
    assert_eq!(
        run.counters().get(VERSCHAERFUNGEN_COUNTER).copied(),
        Some(u32::try_from(2 * all).expect("small"))
    );
    // Drawing again for a district that has its own does nothing.
    run.draw_verschaerfung(&content, 4, &mut events);
    assert_eq!(run.night().expect("night").verschaerfungen.len(), 2 * all);
    // Equal seeds draw the same.
    let (mut twin, _) = night_run(&content, 17);
    for section in 4..(3 + 2 * u32::try_from(all).expect("small")) {
        twin.draw_verschaerfung(&content, section, &mut Vec::new());
    }
    assert_eq!(twin.night(), run.night());
}

#[test]
fn stacked_rules_add_up() {
    let content = content();
    let (mut run, _) = night_run(&content, 19);
    force(
        &mut run,
        &["pricier_tresen", "pricier_tresen", "stingy_host"],
    );
    assert_eq!(run.shop_price(&content, 5), 7);
    assert_eq!(run.shop_price(&content, 0), 0);
    assert_eq!(run.night_fewer_offers(&content), 1);
    force(&mut run, &["sharp_opponents", "sharp_opponents"]);
    assert_eq!(run.night_extra_samples(&content), 8);
    force(
        &mut run,
        &[
            "fewer_redeals",
            "fewer_redeals",
            "fewer_redeals",
            "fewer_redeals",
        ],
    );
    walk_to_pub(&mut run, &content);
    assert_eq!(run.redeals_left(&content), 0, "never below 0");
}

#[test]
fn the_stingy_host_offers_fewer_zinken() {
    let content = content();
    let (mut run, _) = night_run(&content, 23);
    force(&mut run, &["stingy_host"]);
    walk_to_pub(&mut run, &content);
    run.series.games_played = 1;
    run.series.score = i64::try_from(run.series.target).expect("small");
    run.game = None;
    let mut events = Vec::new();
    run.after_game(&content, &mut events).expect("after game");
    let offers = events
        .iter()
        .find_map(|e| match e {
            RunEvent::CardRewardOffered { offers } => Some(offers.len()),
            _ => None,
        })
        .expect("card reward");
    let config = content.run.card_reward.as_ref().expect("config");
    assert_eq!(offers, usize::try_from(config.offers).expect("small") - 1);
    // The night counts the pub.
    assert_eq!(run.night().expect("night").pubs_won, 1);
}

#[test]
fn every_pub_of_the_night_gets_a_malus_and_a_cursed_card() {
    let content = content();
    let (mut run, _) = night_run(&content, 29);
    force(&mut run, &["dive_air", "cursed_card"]);
    let before = run.edition.clone();
    let events = walk_to_pub(&mut run, &content);
    let malus = run.series.night_malus.clone().expect("malus");
    assert!(content.maluses.contains_key(&malus));
    assert_eq!(
        run.malus(&content).map(|m| m.id.clone()),
        Some(malus.clone())
    );
    assert!(events.iter().any(|e| matches!(
        e,
        RunEvent::SeriesStarted { malus: Some(m), .. } if *m == malus
    )));
    assert_eq!(
        run.tour_view(&content)
            .current
            .and_then(|p| p.malus)
            .map(|m| m.id),
        Some(malus)
    );
    let laid = events
        .iter()
        .find_map(|e| match e {
            RunEvent::NightCursesLaid { cards } => Some(cards.clone()),
            _ => None,
        })
        .expect("a cursed card");
    assert_eq!(laid.len(), 1);
    assert!(run.card_is_cursed(&content, laid[0].0));
    // When the pub ends the card is as before.
    let mut events = Vec::new();
    run.end_pub(&content, &mut events);
    assert_eq!(run.edition, before);
}

#[test]
fn a_night_malus_never_lands_on_a_boss_pub() {
    let content = content();
    let (mut run, _) = night_run(&content, 31);
    force(&mut run, &["dive_air"]);
    run.series.boss = Some(BossId::new("twins"));
    assert_eq!(run.draw_night_malus(&content, true), None);
}

#[test]
fn double_bosses_add_a_second_rule_to_the_night_boss_pub() {
    let content = content();
    let (mut run, _) = night_run(&content, 37);
    force(&mut run, &["double_boss"]);
    let boss = run.draw_night_boss(&content).expect("boss");
    let extra = run.night().expect("night").extra_bosses.clone();
    assert_eq!(extra.len(), 1);
    assert_ne!(extra[0], boss);
    assert!(
        content
            .boss(&extra[0])
            .is_some_and(|b| b.rotation.is_empty()),
        "never der Alte as the second rule"
    );
    // In the boss pub both rules are sources.
    run.series.boss = Some(boss.clone());
    let sources = run.sources(&content, false);
    let bosses: Vec<&Source> = sources
        .iter()
        .map(|s| &s.source)
        .filter(|s| matches!(s, Source::Boss(_)))
        .collect();
    assert_eq!(bosses.len(), 2);
    assert!(bosses.contains(&&Source::Boss(extra[0].clone())));
}

#[test]
fn night_bosses_do_not_repeat_until_all_were_drawn() {
    let content = content();
    let (mut run, _) = night_run(&content, 41);
    force(&mut run, &["fewer_redeals"]);
    let pool = content.run.bosses.len() + usize::from(content.run.final_boss.is_some());
    let drawn: Vec<BossId> = (0..pool)
        .map(|_| run.draw_night_boss(&content).expect("boss"))
        .collect();
    let mut unique = drawn.clone();
    unique.sort();
    unique.dedup();
    assert_eq!(unique.len(), pool);
    assert!(drawn.contains(content.run.final_boss.as_ref().expect("der Alte")));
}

#[test]
fn night_relics_stay_in_the_night_unless_unlocked() {
    let content = content();
    let night: Vec<RelicId> = content
        .relics
        .values()
        .filter(|r| r.night)
        .map(|r| r.id.clone())
        .collect();
    assert_eq!(night.len(), 6);
    let normal = start_at(&content, 3, 0, 0);
    let pool: Vec<RelicId> = normal
        .relic_pool(&content, false)
        .into_iter()
        .map(|(id, _)| id)
        .collect();
    // Unlocked (nothing locked here): only the two with an unlock.
    let open: Vec<&RelicId> = night.iter().filter(|id| pool.contains(id)).collect();
    assert_eq!(
        open,
        vec![&RelicId::new("dawn"), &RelicId::new("sweep_out")],
        "only the unlockable ones"
    );
    // Locked by the meta state: none of them in a normal run.
    let start = MetaState::new()
        .run_start(&content, Character::Declarer)
        .expect("start");
    let locked = Run::new_with(&content, 3, &start).expect("run").0;
    let pool: Vec<RelicId> = locked
        .relic_pool(&content, false)
        .into_iter()
        .map(|(id, _)| id)
        .collect();
    assert!(night.iter().all(|id| !pool.contains(id)));
    // In the night every one, locked or not.
    let mut run = locked;
    run.night = Some(night_run(&content, 3).0.night.expect("night"));
    let pool: Vec<RelicId> = run
        .relic_pool(&content, false)
        .into_iter()
        .map(|(id, _)| id)
        .collect();
    assert!(night.iter().all(|id| pool.contains(id)));
}

#[test]
fn a_won_night_boss_offers_night_relics() {
    let content = content();
    let (mut run, _) = night_run(&content, 43);
    force(&mut run, &["fewer_redeals"]);
    walk_to_pub(&mut run, &content);
    let boss = run.section_boss().cloned().expect("boss");
    run.series.index = 4;
    run.series.boss = Some(boss);
    let events = end_pub(&mut run, &content, 3, true);
    assert_eq!(run.phase(), RunPhase::BossReward, "{events:?}");
    let offers = &run.boss_reward().expect("pick").relics;
    assert!(!offers.is_empty());
    assert!(
        offers
            .iter()
            .all(|id| content.relic(id).is_some_and(|r| r.night))
    );
    // Then the route into the next night district with its Verschärfung.
    let events = run
        .apply(
            &content,
            RunAction::ChooseBossRelic {
                offer: Some(0),
                replace: None,
            },
        )
        .expect("pick");
    assert!(
        events
            .iter()
            .any(|e| matches!(e, RunEvent::VerschaerfungDrawn { section: 4, .. }))
    );
    assert_eq!(run.night().expect("night").verschaerfungen.len(), 2);
    assert_eq!(run.view(&content).night.expect("night").district, 5);
}

#[test]
fn losing_in_the_night_ends_only_the_night() {
    let content = content();
    let (mut run, mut events) = night_run(&content, 47);
    force(&mut run, &["fewer_redeals"]);
    events.extend(walk_to_pub(&mut run, &content));
    events.extend(end_pub(&mut run, &content, 3, false));
    assert_eq!(run.phase(), RunPhase::Lost);
    assert!(events.iter().any(|e| matches!(
        e,
        RunEvent::RunLost {
            section: 3,
            night: true,
            ..
        }
    )));
    let mut meta = MetaState::new();
    meta.observe(&content, &events);
    assert_eq!(meta.stats.runs_won, 1, "the win stays");
    assert_eq!(meta.stats.runs_finished, 1, "counted once");
    let record = meta.stats.characters[&Character::Declarer];
    assert_eq!(record.won, 1);
    assert_eq!(record.runs, 1);
    assert_eq!(record.best_section, 3);
    assert_eq!(record.night_best_section, 4);
    assert_eq!(record.night_pubs_won, 0);
    // The Schutzengel still helps once.
    let (mut run, _) = night_run(&content, 47);
    force(&mut run, &["fewer_redeals"]);
    walk_to_pub(&mut run, &content);
    run.relics.push(RelicId::new("guardian_angel"));
    let events = end_pub(&mut run, &content, 3, false);
    assert!(
        events
            .iter()
            .any(|e| matches!(e, RunEvent::SecondChance { .. }))
    );
    assert_eq!(run.phase(), RunPhase::Game);
}

#[test]
fn the_record_counts_night_pubs_and_the_best_game_per_character() {
    let content = content();
    let (mut run, mut events) = night_run(&content, 53);
    force(&mut run, &["fewer_redeals"]);
    events.extend(walk_to_pub(&mut run, &content));
    events.extend(end_pub(&mut run, &content, 1, true));
    let mut meta = MetaState::new();
    meta.observe(&content, &events);
    let record = meta.stats.characters[&Character::Declarer];
    assert_eq!(record.night_pubs_won, 1);
    assert_eq!(record.night_best_section, 4);
    // Won games in the night count for the character's best game.
    let (mut run, mut events) = night_run(&content, 71);
    force(&mut run, &["fewer_redeals"]);
    events.extend(walk_to_pub(&mut run, &content));
    while run.phase() == RunPhase::Game {
        let action = super::bot::RunBot.next_action(&run, &content).expect("bot");
        events.extend(run.apply(&content, action).expect("play"));
    }
    let best = events
        .iter()
        .filter_map(|e| match e {
            RunEvent::GameScored { breakdown, .. } if breakdown.won => Some(breakdown.value),
            _ => None,
        })
        .max()
        .expect("a won game");
    let mut fresh = MetaState::new();
    fresh.observe(&content, &events);
    assert_eq!(fresh.stats.characters[&Character::Declarer].best_game, best);
    // A later night with fewer pubs keeps the record.
    let (mut run, events) = night_run(&content, 59);
    force(&mut run, &["fewer_redeals"]);
    meta.observe(&content, &events);
    assert_eq!(
        meta.stats.characters[&Character::Declarer].night_pubs_won,
        1
    );
}

#[test]
fn night_milestones_unlock_backs_and_relics() {
    let content = content();
    let mut meta = MetaState::new();
    let started = |section: u32| RunEvent::<crate::game::Event<DealtCard>>::SeriesStarted {
        section,
        index: 0,
        target: 1,
        game_limit: 3,
        redeals: 2,
        boss: None,
        malus: None,
    };
    meta.observe(&content, &[started(3)]);
    assert!(meta.card_backs(&content).is_empty());
    assert_eq!(
        meta.observe(&content, &[started(4)]),
        vec![UnlockId::new("night_district_5")]
    );
    assert_eq!(meta.card_backs(&content), vec!["moon".to_owned()]);
    assert!(
        meta.locked_relics(&content)
            .contains(&RelicId::new("sweep_out"))
    );
    assert_eq!(
        meta.observe(&content, &[started(6)]),
        vec![UnlockId::new("night_district_7")]
    );
    assert!(
        !meta
            .locked_relics(&content)
            .contains(&RelicId::new("sweep_out"))
    );
    assert_eq!(
        meta.observe(&content, &[started(9)]),
        vec![UnlockId::new("night_district_10")]
    );
    assert_eq!(meta.view(&content).card_backs.len(), 3);
}

#[test]
fn a_night_run_saves_and_loads_and_old_saves_have_no_night() {
    let content = content();
    let (mut run, _) = night_run(&content, 61);
    force(&mut run, &["dive_air"]);
    walk_to_pub(&mut run, &content);
    let saved = serde_json::to_string(&run).expect("save");
    let loaded: Run = serde_json::from_str(&saved).expect("load");
    assert_eq!(loaded, run);
    let normal = start_at(&content, 61, 0, 0);
    let saved = serde_json::to_string(&normal).expect("save");
    assert!(!saved.contains("\"night\""));
    assert!(!saved.contains("night_malus"));
    let loaded: Run = serde_json::from_str(&saved).expect("load");
    assert!(loaded.night().is_none());
}

#[test]
fn night_rules_match_the_content() {
    let content = content();
    let rules: Vec<&NightRule> = content.verschaerfungen.values().map(|v| &v.rule).collect();
    assert_eq!(rules.len(), 8);
    assert!(rules.iter().any(|r| matches!(r, NightRule::DoubleBoss)));
    assert!(rules.iter().any(|r| matches!(r, NightRule::MalusEveryPub)));
    let _ = MalusId::new("no_redeal");
}

#[test]
fn the_bot_plays_on_into_the_night_only_when_asked() {
    let content = content();
    let (run, _) = won_run(&content, 67);
    assert_eq!(super::bot::RunBot.next_action(&run, &content), None);
    assert_eq!(
        super::bot::RunBot.night_action(&run),
        Some(RunAction::EnterNight)
    );
    let (night, _) = night_run(&content, 67);
    assert_eq!(super::bot::RunBot.night_action(&night), None);
}

#[test]
fn a_character_may_have_its_own_night_growth() {
    let mut content = content();
    let null = content
        .character(Character::NullPlayer)
        .expect("null")
        .clone();
    let mut own = null.clone();
    own.night_growth = Some(vec![
        crate::content::Factor::from_permille(1200),
        crate::content::Factor::from_permille(2000),
    ]);
    let config = own.night(&content.run.night);
    assert_eq!(config.growth_of(1).permille(), 1200);
    assert_eq!(config.growth_of(5).permille(), 2000);
    let mut plain = null;
    plain.night_growth = None;
    assert_eq!(plain.night(&content.run.night), content.run.night);
    // The run's Zielwerte of the night follow the character's growth.
    content.characters.insert(Character::NullPlayer, own);
    let (mut run, _) = night_run(&content, 7);
    run.character = Character::NullPlayer;
    force(&mut run, &["fewer_redeals"]);
    let base = series_target(&content, Character::NullPlayer, 2, 0, None).expect("base");
    assert_eq!(
        run.pub_target(&content, 4, 0, None),
        Some((base * 24 + 5) / 10)
    );
}

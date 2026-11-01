//! Tests of M7: curses only where announced (DESIGN 7.2) and the short
//! evening (kurzer Abend, DESIGN 5.3).

use super::bot::RunBot;
use super::content_tests::content;
use super::meta::MetaState;
use super::opponents::opponents_in;
use super::*;
use crate::content::{ConsumableId, Factor, UnlockId};
use crate::deck::DeckChange;

fn is_curse(content: &Content, id: &CardModId) -> bool {
    content.card_mod(id).is_some_and(|m| m.curse)
}

/// Modifications attached by these events.
fn attached(events: &[RunEvent]) -> Vec<CardModId> {
    events
        .iter()
        .filter_map(|e| match e {
            RunEvent::DeckChanged(DeckChange::Modified { card_mod, .. }) => Some(card_mod.clone()),
            _ => None,
        })
        .collect()
}

/// A run of `character` at the Tresen with an empty purse.
fn shop_run(content: &Content, seed: u64, character: Character) -> Run {
    let start = RunStart {
        character,
        ..RunStart::default()
    };
    let (mut run, _) = Run::new_with(content, seed, &start).expect("run");
    run.game = None;
    run.phase = RunPhase::Shop;
    run.shop = Some(run.draw_offer(content, 0));
    run.coins = 0;
    run
}

/// The content with only this encounter, which always comes.
fn only_encounter(id: &str) -> Content {
    let mut content = content();
    content.encounters.retain(|k, _| k.as_str() == id);
    assert_eq!(content.encounters.len(), 1, "{id}");
    content.run.encounter_chance = Factor::ONE;
    content
}

// ---------------------------------------------------------------------------
// Curses only where announced
// ---------------------------------------------------------------------------

#[test]
fn the_ink_pad_never_draws_a_curse() {
    let content = content();
    for character in Character::ALL {
        let mut seen = BTreeSet::new();
        for seed in 0..300 {
            let mut run = shop_run(&content, seed, character);
            run.consumables = vec![ConsumableId::new("ink_pad")];
            let cards: Vec<u32> = run
                .deck()
                .cards()
                .iter()
                .filter(|c| c.kind != CardKind::Blank)
                .map(|c| c.id)
                .collect();
            let target = cards[usize::try_from(seed).expect("seed") % cards.len()];
            let events = run
                .apply(
                    &content,
                    RunAction::UseConsumable {
                        slot: 0,
                        target: Some(target),
                        suit: None,
                    },
                )
                .expect("ink pad");
            let mods = attached(&events);
            assert_eq!(mods.len(), 1, "seed {seed}");
            assert!(!is_curse(&content, &mods[0]), "seed {seed}: {}", mods[0]);
            seen.extend(mods);
        }
        // The draw is random, not one fixed modification.
        assert!(seen.len() >= 5, "{character}: {seen:?}");
    }
}

#[test]
fn the_card_dealer_never_draws_a_curse() {
    let content = only_encounter("card_dealer");
    for character in Character::ALL {
        let mut seen = BTreeSet::new();
        for seed in 0..300 {
            let mut run = shop_run(&content, seed, character);
            run.shop = None;
            let mut events = Vec::new();
            assert!(run.open_encounter(&content, &mut events), "seed {seed}");
            let options = &run.encounter().expect("open").options;
            for option in options {
                if let Some(m) = &option.card_mod {
                    assert!(!is_curse(&content, m), "seed {seed}: {m}");
                    seen.insert(m.clone());
                }
            }
            let events = run
                .apply(
                    &content,
                    RunAction::ChooseEncounter {
                        option: 0,
                        target: None,
                    },
                )
                .expect("free card");
            for m in attached(&events) {
                assert!(!is_curse(&content, &m), "seed {seed}: {m}");
            }
        }
        assert!(seen.len() >= 5, "{character}: {seen:?}");
    }
}

#[test]
fn old_card_dealer_options_without_a_drawn_mod_never_give_a_curse() {
    // Saves from before M5 drew the modification only when chosen.
    let content = content();
    for seed in 0..300 {
        let mut run = shop_run(&content, seed, Character::Declarer);
        run.shop = None;
        run.phase = RunPhase::Encounter;
        run.encounter = Some(ActiveEncounter {
            id: EncounterId::new("card_dealer"),
            options: vec![
                EncounterOption {
                    choice: EncounterChoice::MarkRandomCard,
                    cost: 0,
                    card_mod: None,
                },
                EncounterOption {
                    choice: EncounterChoice::Leave,
                    cost: 0,
                    card_mod: None,
                },
            ],
        });
        let events = run
            .apply(
                &content,
                RunAction::ChooseEncounter {
                    option: 0,
                    target: None,
                },
            )
            .expect("free card");
        let mods = attached(&events);
        assert_eq!(mods.len(), 1);
        assert!(!is_curse(&content, &mods[0]), "seed {seed}: {}", mods[0]);
    }
}

/// Over whole runs of the bot with an encounter after every series, every
/// curse that lands in the deck comes from an encounter whose text says
/// "Fluch", and pays.
#[test]
fn in_whole_runs_curses_come_only_from_announcing_encounters() {
    let mut content = content();
    content.run.encounter_chance = Factor::ONE;
    let mut curses = 0;
    for character in Character::ALL {
        for seed in 0..40 {
            let start = RunStart {
                character,
                ..RunStart::default()
            };
            let (mut run, mut events) = Run::new_with(&content, seed, &start).expect("run");
            while let Some(action) = RunBot.next_action(&run, &content) {
                events.extend(run.apply(&content, action).expect("bot"));
            }
            let mut open: Option<EncounterId> = None;
            for (i, event) in events.iter().enumerate() {
                match event {
                    RunEvent::EncounterOpened { encounter, .. } => open = Some(encounter.clone()),
                    RunEvent::EncounterResolved { .. } => open = None,
                    RunEvent::DeckChanged(DeckChange::Modified { card_mod, .. })
                        if is_curse(&content, card_mod) =>
                    {
                        curses += 1;
                        let encounter = open
                            .as_ref()
                            .and_then(|id| content.encounter(id))
                            .unwrap_or_else(|| panic!("seed {seed}: unannounced {card_mod}"));
                        assert!(encounter.text.contains("Fluch"), "{}", encounter.id);
                        assert!(
                            matches!(
                                events.get(i + 1),
                                Some(RunEvent::CoinsGained { amount, .. }) if *amount > 0
                            ),
                            "seed {seed}: {card_mod} did not pay"
                        );
                    }
                    _ => {}
                }
            }
        }
    }
    assert!(curses > 0, "the bot takes announced curses");
}

#[test]
fn every_curse_pays_groschen_when_attached() {
    let content = content();
    let curses: Vec<_> = content.card_mods.values().filter(|m| m.curse).collect();
    assert!(curses.len() >= 4);
    for def in curses {
        assert!(def.text.starts_with("Fluch:"), "{}", def.id);
        let pays = def.hooks.iter().any(|h| {
            h.stage() == Stage::OnAttach
                && h.then()
                    .iter()
                    .any(|e| matches!(e, Effect::GainCoins(n) if *n > 0))
        });
        assert!(pays, "{} pays nothing", def.id);
    }
}

#[test]
fn announced_curses_name_the_curse_and_pay() {
    // Der Falschspieler: the curse is drawn when it opens, both options
    // name it, the random card pays 3 Groschen.
    let content = only_encounter("card_sharp");
    for character in Character::ALL {
        let mut seen = BTreeSet::new();
        for seed in 0..60 {
            let mut run = shop_run(&content, seed, character);
            run.shop = None;
            let mut events = Vec::new();
            assert!(run.open_encounter(&content, &mut events));
            let options = run.encounter().expect("open").options.clone();
            assert_eq!(options.len(), 3);
            let curse = options[0].card_mod.clone().expect("named");
            assert!(is_curse(&content, &curse), "{curse}");
            assert_eq!(options[1].card_mod.as_ref(), Some(&curse));
            assert_eq!(options[1].cost, 4);
            seen.insert(curse.clone());
            let events = run
                .apply(
                    &content,
                    RunAction::ChooseEncounter {
                        option: 0,
                        target: None,
                    },
                )
                .expect("random card");
            assert_eq!(attached(&events), vec![curse]);
            assert_eq!(run.coins(), 3, "seed {seed}");
        }
        // Every curse of the character's pool can come.
        let pool: Vec<CardModId> = shop_run(&content, 0, character)
            .mod_pool(&content, &[], true)
            .into_iter()
            .map(|(id, _)| id)
            .collect();
        assert_eq!(seen.into_iter().collect::<Vec<_>>(), pool, "{character}");
    }
    // Umgekipptes Bier: the Bierfleck on a random card pays 3 Groschen.
    let content = only_encounter("spilled_beer");
    let mut run = shop_run(&content, 5, Character::Declarer);
    run.shop = None;
    let mut events = Vec::new();
    assert!(run.open_encounter(&content, &mut events));
    run.apply(
        &content,
        RunAction::ChooseEncounter {
            option: 0,
            target: None,
        },
    )
    .expect("random card");
    assert_eq!(run.coins(), 3);
    assert_eq!(
        run.deck()
            .cards()
            .iter()
            .filter(|c| c.mods.contains(&CardModId::new("beer_stain")))
            .count(),
        1
    );
}

/// The embedded content with one replacement in `encounters.ron` or
/// `run.ron`.
fn load_with(file: &str, from: &str, to: &str) -> Result<Content, crate::content::ContentError> {
    let embedded = crate::content::Sources::embedded();
    let text = if file == "run" {
        embedded.run
    } else {
        embedded.encounters
    };
    assert!(text.contains(from), "{from}");
    let changed = text.replace(from, to);
    Content::from_sources(&if file == "run" {
        crate::content::Sources {
            run: &changed,
            ..embedded
        }
    } else {
        crate::content::Sources {
            encounters: &changed,
            ..embedded
        }
    })
}

#[test]
fn an_encounter_that_gives_a_curse_must_say_so() {
    let card_sharp = &content().encounters[&EncounterId::new("card_sharp")].text;
    assert!(
        load_with(
            "encounters",
            card_sharp,
            "Ein Fremder zinkt dir eine Karte."
        )
        .is_err()
    );
    let beer = &content().encounters[&EncounterId::new("spilled_beer")].text;
    assert!(load_with("encounters", beer, "Ein Bier kippt über das Blatt.").is_err());
    // A random draw without curse needs no announcement.
    let dealer = &content().encounters[&EncounterId::new("card_dealer")].text;
    assert!(
        load_with(
            "encounters",
            dealer,
            "Der Kartengeber zinkt dir eine Karte."
        )
        .is_ok()
    );
}

// ---------------------------------------------------------------------------
// Short evening
// ---------------------------------------------------------------------------

fn short_start(character: Character) -> RunStart {
    RunStart {
        character,
        length: RunLength::Short,
        ..RunStart::default()
    }
}

#[test]
fn a_short_evening_plays_the_rows_of_run_ron() {
    let content = content();
    let rows = content.run.short_sections.clone();
    assert_eq!(rows.len(), 3, "kurzer Abend: 3 Abschnitte");
    assert_eq!(content.run.sections(RunLength::Short), 3);
    assert_eq!(
        content.run.sections(RunLength::Full),
        u32::try_from(content.run.targets.len()).expect("len")
    );
    for character in Character::ALL {
        for (section, &row) in (0u32..).zip(&rows) {
            let start = RunStart {
                section,
                ..short_start(character)
            };
            let (run, _) = Run::new_with(&content, 3, &start).expect("run");
            assert_eq!(run.length(), RunLength::Short);
            assert_eq!(run.sections(&content), 3);
            assert_eq!(run.curve_row(&content), row);
            assert_eq!(
                run.series().target,
                series_target(&content, character, row, 0, None).expect("target")
            );
            assert_eq!(
                opponents_in(&content, run.curve_row(&content)),
                opponents_in(&content, row)
            );
            let view = run.view(&content);
            assert_eq!(view.sections, 3);
            assert_eq!(view.length, RunLength::Short);
            assert_eq!(view.series.section, section);
        }
    }
    // A short evening has no fourth section.
    let start = RunStart {
        section: 3,
        ..short_start(Character::Declarer)
    };
    assert_eq!(
        Run::new_with(&content, 3, &start).map(|_| ()),
        Err(RunError::InvalidStart)
    );
}

#[test]
fn a_short_evening_ends_with_the_final_boss() {
    let content = content();
    let final_boss = content.run.final_boss.clone().expect("Der Alte");
    let start = RunStart {
        section: 2,
        index: 2,
        ..short_start(Character::Declarer)
    };
    let (mut run, events) = Run::new_with(&content, 8, &start).expect("run");
    assert!(events.contains(&RunEvent::BossAnnounced {
        section: 2,
        boss: final_boss.clone(),
    }));
    assert_eq!(run.series().boss.as_ref(), Some(&final_boss));
    let row = content.run.short_sections[2];
    assert_eq!(
        run.series().target,
        series_target(&content, Character::Declarer, row, 2, Some(&final_boss)).expect("target")
    );
    // The boss series won: the run is won, as a short evening.
    run.series.score = i64::try_from(run.series.target).expect("target");
    run.game = None;
    let mut events = Vec::new();
    run.after_game(&content, &mut events).expect("won");
    assert_eq!(run.phase(), RunPhase::Won);
    assert!(events.contains(&RunEvent::RunWon {
        character: Character::Declarer,
        length: RunLength::Short,
        stake: 0,
    }));
    // In a full run the third section's boss is drawn, not the final boss.
    let start = RunStart {
        section: 2,
        index: 2,
        ..RunStart::default()
    };
    for seed in 0..20 {
        let (run, _) = Run::new_with(&content, seed, &start).expect("run");
        assert_ne!(run.series().boss.as_ref(), Some(&final_boss));
    }
}

#[test]
fn the_bot_plays_short_evenings_to_the_end() {
    let content = content();
    let mut sections = BTreeSet::new();
    for character in Character::ALL {
        for seed in 0..30 {
            let (mut run, _) = Run::new_with(&content, seed, &short_start(character)).expect("run");
            while let Some(action) = RunBot.next_action(&run, &content) {
                let events = run.apply(&content, action).expect("bot");
                for event in &events {
                    if let RunEvent::RunLost { length, .. } | RunEvent::RunWon { length, .. } =
                        event
                    {
                        assert_eq!(*length, RunLength::Short);
                    }
                }
            }
            assert!(matches!(run.phase(), RunPhase::Won | RunPhase::Lost));
            assert!(run.series().section < 3);
            sections.insert(run.series().section);
        }
    }
    assert!(sections.len() >= 2, "{sections:?}");
}

#[test]
fn a_short_evening_counts_for_unlocks_but_not_run_wins_and_its_own_statistics() {
    let content = content();
    let mut meta = MetaState::new();
    let start = meta
        .run_start_with(&content, Character::Declarer, RunLength::Short)
        .expect("start");
    assert_eq!(start.length, RunLength::Short);
    assert_eq!(
        meta.run_start(&content, Character::Declarer)
            .expect("full")
            .length,
        RunLength::Full
    );
    let events: Vec<RunEvent> = vec![
        RunEvent::SeriesStarted {
            section: 2,
            index: 0,
            target: 48,
            game_limit: 8,
            passes: 6,
            boss: None,
        },
        RunEvent::RunWon {
            character: Character::Declarer,
            length: RunLength::Short,
            stake: 0,
        },
    ];
    let earned = meta.observe(&content, &events);
    // A won short evening counts for every unlock except winning a run
    // (DESIGN 9.3): reaching section 3 does, `tour_won` does not.
    assert!(!earned.contains(&UnlockId::new("tour_won")), "{earned:?}");
    assert!(!meta.character_unlocked(&content, Character::NullPlayer));
    assert!(
        earned.contains(&UnlockId::new("third_evening")),
        "{earned:?}"
    );
    assert_eq!(meta.stats.runs_won, 1);
    assert_eq!(meta.stats.lengths[&RunLength::Short].won, 1);
    assert_eq!(meta.stats.lengths[&RunLength::Short].best_section, 3);
    assert!(!meta.stats.lengths.contains_key(&RunLength::Full));
    let events: Vec<RunEvent> = vec![RunEvent::RunLost {
        section: 1,
        index: 2,
        character: Character::Declarer,
        length: RunLength::Full,
        stake: 0,
    }];
    meta.observe(&content, &events);
    assert_eq!(meta.stats.runs_finished, 2);
    assert_eq!(meta.stats.lengths[&RunLength::Full].runs, 1);
    assert_eq!(meta.stats.lengths[&RunLength::Full].won, 0);
    assert_eq!(meta.stats.lengths[&RunLength::Full].best_section, 2);
    // Round trip with the length as a map key.
    let text = ron::to_string(&meta).expect("ron");
    assert!(text.contains("Short:"), "{text}");
    let back: MetaState = ron::from_str(&text).expect("ron");
    assert_eq!(back, meta);
}

#[test]
fn saves_and_events_from_before_the_short_evening_are_full() {
    let content = content();
    // A run saved before the field existed.
    let (run, _) = Run::new(&content, 21).expect("run");
    let text = ron::to_string(&run).expect("ron");
    assert!(text.contains(",length:Full"), "length is saved");
    let old = text.replace(",length:Full", "");
    let back: Run = ron::from_str(&old).expect("old save");
    assert_eq!(back.length(), RunLength::Full);
    assert_eq!(back, run);
    // A meta state and run end events without the length.
    let meta: MetaState =
        ron::from_str("(version:1,unlocked:[],stats:(runs_finished:3))").expect("old meta");
    assert_eq!(meta.stats.runs_finished, 3);
    assert!(meta.stats.lengths.is_empty());
    let event: RunEvent = ron::from_str("RunWon(character:Declarer)").expect("old event");
    assert_eq!(
        event,
        RunEvent::RunWon {
            character: Character::Declarer,
            length: RunLength::Full,
            stake: 0,
        }
    );
}

#[test]
fn short_sections_must_rise_within_the_curve() {
    let list = format!("short_sections: {:?},", content().run.short_sections);
    assert!(load_with("run", &list, "short_sections: [0, 2, 5],").is_err());
    assert!(load_with("run", &list, "short_sections: [2, 1, 4],").is_err());
    // Without a list a short evening is a full one.
    let content = load_with("run", &list, "").expect("no list");
    assert_eq!(
        content.run.sections(RunLength::Short),
        content.run.sections(RunLength::Full)
    );
}

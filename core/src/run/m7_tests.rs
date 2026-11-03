//! Tests of M7: curses only where announced (DESIGN 7.2) and the short
//! evening (kurzer Abend, DESIGN 5.3).

use super::bot::RunBot;
use super::content_tests::content;
use super::meta::MetaState;
use super::*;
use crate::content::ConsumableId;
use crate::deck::DeckChange;

fn is_curse(content: &Content, id: &CardModId) -> bool {
    content
        .card_mod(id)
        .is_some_and(crate::content::CardModDef::is_curse)
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
    content
}

// ---------------------------------------------------------------------------
// Curses only where announced
// ---------------------------------------------------------------------------

#[test]
fn the_ink_pad_never_draws_a_curse() {
    // Removed with the Zinken at the Tresen (DESIGN 8.6); the dormant
    // action stays under test.
    let content = crate::run::legacy_content();
    for character in Character::ALL {
        let mut seen = BTreeSet::new();
        for seed in 0..300 {
            let mut run = shop_run(&content, seed, character);
            run.consumables = vec![ConsumableId::new("ink_pad")];
            let cards: Vec<u32> = run
                .edition()
                .cards()
                .iter()
                .filter(|c| c.kind != CardKind::Blank)
                // A cursed card ("Bruderherz" starts with some) takes none.
                .filter(|c| !run.card_is_cursed(&content, c.id))
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
            assert!(
                run.open_random_encounter(&content, &mut events),
                "seed {seed}"
            );
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
                    value: None,
                },
                EncounterOption {
                    choice: EncounterChoice::Leave,
                    cost: 0,
                    card_mod: None,
                    value: None,
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
    let content = content();
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
            let mut malus_curse = false;
            for (i, event) in events.iter().enumerate() {
                match event {
                    RunEvent::EncounterOpened { encounter, .. } => open = Some(encounter.clone()),
                    RunEvent::EncounterResolved { .. } => open = None,
                    // A Spelunke malus curses cards for its pub (ADR 0016),
                    // announced on the route choice.
                    RunEvent::SeriesStarted { malus: Some(m), .. }
                        if content.maluses[m].curse.is_some() =>
                    {
                        malus_curse = true;
                    }
                    RunEvent::GameDealt { .. } => malus_curse = false,
                    RunEvent::DeckChanged(DeckChange::Modified { card_mod, .. })
                        if is_curse(&content, card_mod) && !malus_curse =>
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
    let curses: Vec<_> = content
        .card_mods
        .values()
        .filter(|m| m.kind == crate::content::ModKind::Curse)
        .collect();
    // "Abgestuft" (a special curse of the Nullspieler's edition) is never
    // attached at the Tresen and pays nothing.
    assert_eq!(curses.len(), 3);
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
    // The Falschspieler (since 02.11.2026 "am Nebentisch", a gamble) is
    // tested in `items_tests`.
    // Umgekipptes Bier: the Bierfleck on a random card pays 3 Groschen.
    let content = only_encounter("spilled_beer");
    let mut run = shop_run(&content, 5, Character::Declarer);
    run.shop = None;
    let mut events = Vec::new();
    assert!(run.open_random_encounter(&content, &mut events));
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
        run.edition()
            .cards()
            .iter()
            .filter(|c| c.mods.contains(&CardModId::new("battered")))
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
// Short evening (dropped with ADR 0016)
// ---------------------------------------------------------------------------

#[test]
fn a_short_evening_cannot_start_and_old_meta_states_keep_it() {
    let content = content();
    let start = RunStart {
        length: RunLength::Short,
        ..RunStart::default()
    };
    assert_eq!(
        Run::new_with(&content, 21, &start).map(|_| ()),
        Err(RunError::InvalidStart)
    );
    // A meta state with short-evening statistics and stakes still loads.
    let meta: MetaState = ron::from_str(
        "(version:1,unlocked:[],stats:(runs_finished:2,lengths:{Short:(runs:2,won:1,best_section:3)}),stakes:{Declarer:{Short:1}})",
    )
    .expect("old meta");
    assert_eq!(meta.stats.lengths[&RunLength::Short].won, 1);
    assert_eq!(meta.max_stake(Character::Declarer, RunLength::Short), 1);
    assert_eq!(meta.max_stake(Character::Declarer, RunLength::Full), 0);
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

//! More ways to get Marken in encounters (04.11.2026, DESIGN 13.0): a
//! random Marke on a chosen card (Kartenwerkstatt), a chosen Marke on a
//! random card (Der Pfandleiher), a chosen Marke on a chosen card (Die
//! Markensammlerin).

use super::bot::RunBot;
use super::content_tests::{content, relic_run};
use super::*;
use crate::content::CardPick;

const MARKEN: [&str; 3] = ["voucher_coins", "voucher_undead", "voucher_wild"];

fn id(s: &str) -> CardModId {
    CardModId::new(s)
}

fn choose(option: usize, target: Option<u32>, marke: Option<&str>) -> RunAction {
    RunAction::ChooseEncounter {
        option,
        target,
        marke: marke.map(id),
    }
}

/// Index of the option with a Marke effect of this kind.
fn marke_option(run: &Run, card: CardPick, marke: CardPick) -> usize {
    run.encounter()
        .expect("encounter")
        .options
        .iter()
        .position(|o| o.effects.contains(&EncounterEffect::Marke { card, marke }))
        .expect("Marke option")
}

/// The Marken on the card, in the edition.
fn marken_on(run: &Run, card: u32) -> Vec<CardModId> {
    run.edition()
        .get(card)
        .expect("card")
        .mods
        .iter()
        .filter(|m| MARKEN.contains(&m.as_str()))
        .cloned()
        .collect()
}

#[test]
fn the_three_marke_options_are_in_three_encounters() {
    let content = content();
    for (encounter, card, marke) in [
        ("workshop", CardPick::Chosen, CardPick::Random),
        ("pawnbroker", CardPick::Random, CardPick::Chosen),
        ("stamp_collector", CardPick::Chosen, CardPick::Chosen),
    ] {
        let def = content
            .encounter(&EncounterId::new(encounter))
            .expect(encounter);
        assert!(
            def.options
                .iter()
                .flat_map(|o| &o.effects)
                .any(|e| *e == EncounterEffect::Marke { card, marke }),
            "{encounter}"
        );
    }
    let run = relic_run(&content, &[]);
    let marken: Vec<&str> = run.marken(&content).iter().map(|m| m.id.as_str()).collect();
    assert_eq!(marken, MARKEN);
}

#[test]
fn a_random_marke_goes_on_the_chosen_card() {
    let content = content();
    let mut run = relic_run(&content, &[]);
    run.test_encounter(&content, "workshop");
    let option = marke_option(&run, CardPick::Chosen, CardPick::Random);
    // The view lists what may come, with its chance by rarity.
    let view = run.view(&content).encounter.expect("view");
    let marken = &view.options[option].marken;
    assert_eq!(marken.len(), 3);
    assert!(marken.iter().all(|m| m.can_choose && m.targets.is_empty()));
    assert_eq!(marken.iter().map(|m| m.percent).sum::<u32>() / 10, 9);
    assert!(
        marken[0].percent > marken[1].percent,
        "Pfandmarke most often"
    );
    assert_eq!(
        run.clone().apply(&content, choose(option, None, None)),
        Err(RunError::TargetRequired)
    );
    let ace = run
        .edition()
        .id_of(Card::new(Suit::Clubs, Rank::Ace))
        .expect("CA");
    let coins = run.coins();
    let events = run
        .apply(&content, choose(option, Some(ace), None))
        .expect("take");
    let given = events
        .iter()
        .find_map(|e| match e {
            RunEvent::MarkeGiven {
                card,
                card_mod,
                drawn,
                random_card,
            } => Some((*card, card_mod.clone(), *drawn, *random_card)),
            _ => None,
        })
        .expect("MarkeGiven");
    assert_eq!(given.0, ace);
    assert!(given.2 && !given.3);
    assert_eq!(marken_on(&run, ace), vec![given.1]);
    assert_eq!(run.coins(), coins, "free in the workshop");
    // Deterministic: the same run draws the same Marke.
    let mut again = relic_run(&content, &[]);
    again.test_encounter(&content, "workshop");
    let events_again = again
        .apply(&content, choose(option, Some(ace), None))
        .expect("take");
    assert_eq!(events, events_again);
}

#[test]
fn a_random_marke_respects_the_special_card_caps() {
    let content = content();
    let mut run = relic_run(&content, &[]);
    // Another card is Untot and another a Wandelkarte: only the
    // Pfandmarke can come (DESIGN 7.2).
    for (card, special) in [(0, "undead"), (1, "wild_card")] {
        run.edition
            .add_mod(&content, card, &id(special), 1)
            .expect("special");
    }
    run.test_encounter(&content, "workshop");
    let option = marke_option(&run, CardPick::Chosen, CardPick::Random);
    let view = run.view(&content).encounter.expect("view");
    let can: Vec<bool> = view.options[option]
        .marken
        .iter()
        .map(|m| m.can_choose)
        .collect();
    assert_eq!(can, [true, false, false]);
    assert_eq!(view.options[option].marken[0].percent, 100);
    for seed in 0..12 {
        let mut probe = run.clone();
        probe.encounter_rng = crate::rng::Rng::from_seed(seed);
        probe
            .apply(&content, choose(option, Some(20), None))
            .expect("take");
        assert_eq!(marken_on(&probe, 20), vec![id("voucher_coins")]);
    }
    // A cursed card takes no Marke.
    let mut run = relic_run(&content, &[]);
    run.edition
        .add_mod(&content, 9, &id("battered"), 5)
        .expect("curse");
    run.test_encounter(&content, "workshop");
    let option = marke_option(&run, CardPick::Chosen, CardPick::Random);
    let view = run.view(&content).encounter.expect("view");
    assert!(!view.options[option].targets.contains(&9));
    assert_eq!(
        run.apply(&content, choose(option, Some(9), None)),
        Err(RunError::NoModAvailable)
    );
}

#[test]
fn a_chosen_marke_goes_on_a_random_card() {
    let content = content();
    let mut run = relic_run(&content, &[]);
    run.coins = 5;
    run.test_encounter(&content, "pawnbroker");
    let option = marke_option(&run, CardPick::Random, CardPick::Chosen);
    let view = run.view(&content).encounter.expect("view");
    let o = &view.options[option];
    assert!(o.can_choose);
    assert_eq!(o.target, super::view::OptionTarget::None);
    assert!(o.marken.iter().all(|m| m.can_choose && m.percent == 0));
    // The Marke must be named.
    assert_eq!(
        run.clone().apply(&content, choose(option, None, None)),
        Err(RunError::TargetRequired)
    );
    // Not a Zinke that is no Marke.
    assert_eq!(
        run.clone()
            .apply(&content, choose(option, None, Some("trump_bonus"))),
        Err(RunError::NoModAvailable)
    );
    let events = run
        .apply(&content, choose(option, None, Some("voucher_undead")))
        .expect("take");
    let card = events
        .iter()
        .find_map(|e| match e {
            RunEvent::MarkeGiven {
                card,
                card_mod,
                drawn: false,
                random_card: true,
            } if card_mod.as_str() == "voucher_undead" => Some(*card),
            _ => None,
        })
        .expect("MarkeGiven");
    assert_eq!(marken_on(&run, card), vec![id("voucher_undead")]);
    assert_eq!(run.coins(), 3, "2 Groschen");
    // With an Untot card the Totenmarke cannot be chosen any more.
    let mut run = relic_run(&content, &[]);
    run.coins = 5;
    run.edition
        .add_mod(&content, 0, &id("undead"), 1)
        .expect("special");
    run.test_encounter(&content, "pawnbroker");
    let view = run.view(&content).encounter.expect("view");
    let can: Vec<bool> = view.options[option]
        .marken
        .iter()
        .map(|m| m.can_choose)
        .collect();
    assert_eq!(can, [true, false, true]);
    assert_eq!(
        run.apply(&content, choose(option, None, Some("voucher_undead"))),
        Err(RunError::NoModAvailable)
    );
    // Too poor: blocked, the other option is free.
    let mut run = relic_run(&content, &[]);
    run.coins = 1;
    run.test_encounter(&content, "pawnbroker");
    assert_eq!(
        run.encounter_option_block(&content, option),
        Some(OptionBlock::NotEnoughCoins)
    );
}

#[test]
fn a_chosen_marke_goes_on_a_chosen_card() {
    let content = content();
    let mut run = relic_run(&content, &[]);
    run.coins = 6;
    run.edition
        .add_mod(&content, 9, &id("battered"), 5)
        .expect("curse");
    run.test_encounter(&content, "stamp_collector");
    let bought = 0;
    let view = run.view(&content).encounter.expect("view");
    let o = &view.options[bought];
    assert_eq!(o.target, super::view::OptionTarget::Card);
    assert_eq!(o.marken.len(), 3);
    for m in &o.marken {
        assert!(m.can_choose);
        assert!(!m.targets.contains(&9), "a cursed card takes no Marke");
        assert_eq!(m.targets.len(), m.target_costs.len());
        assert!(m.target_costs.iter().all(|&c| c == 5));
    }
    assert_eq!(
        run.clone().apply(&content, choose(bought, Some(20), None)),
        Err(RunError::TargetRequired)
    );
    run.apply(&content, choose(bought, Some(20), Some("voucher_wild")))
        .expect("take");
    assert_eq!(marken_on(&run, 20), vec![id("voucher_wild")]);
    assert_eq!(run.coins(), 1);
    // A Pfandmarke on a card with a Pfandmarke rises (one Zinke per card).
    let mut run = relic_run(&content, &[]);
    run.edition
        .add_mod(&content, 20, &id("voucher_coins"), 3)
        .expect("voucher");
    run.test_encounter(&content, "stamp_collector");
    let traded = 1;
    let events = run
        .apply(&content, choose(traded, Some(20), Some("voucher_coins")))
        .expect("trade");
    assert_eq!(
        run.edition()
            .get(20)
            .expect("card")
            .mod_value(&id("voucher_coins")),
        6
    );
    // The trade puts a curse on another card, announced in the text.
    assert!(events.iter().any(|e| matches!(
        e,
        RunEvent::DeckChanged(DeckChange::Modified { card, card_mod, .. })
            if *card != 20 && content.card_mod(card_mod).is_some_and(CardModDef::is_curse)
    )));
}

#[test]
fn the_bot_takes_every_marke_option() {
    let content = content();
    for encounter in ["workshop", "pawnbroker", "stamp_collector"] {
        let mut run = relic_run(&content, &[]);
        run.coins = 10;
        run.test_encounter(&content, encounter);
        let def = content
            .encounter(&EncounterId::new(encounter))
            .expect("def");
        for (i, option) in def.options.iter().enumerate() {
            if !option
                .effects
                .iter()
                .any(|e| matches!(e, EncounterEffect::Marke { .. }))
            {
                continue;
            }
            // Only this option on offer: the bot must find target and Marke.
            let mut probe = run.clone();
            let open = probe.encounter.as_mut().expect("open");
            let only = open.options[i].clone();
            open.options = vec![only];
            let action = RunBot.encounter_action(&probe, &content);
            let RunAction::ChooseEncounter { marke, .. } = &action else {
                panic!("{action:?}");
            };
            let wants_marke = matches!(
                option.effects[0],
                EncounterEffect::Marke {
                    marke: CardPick::Chosen,
                    ..
                }
            );
            assert_eq!(marke.is_some(), wants_marke, "{encounter}");
            if wants_marke {
                assert_eq!(marke.as_ref(), Some(&id("voucher_coins")), "{encounter}");
            }
            probe
                .apply(&content, action)
                .expect("the bot's choice is legal");
            assert_ne!(probe.phase(), RunPhase::Encounter, "{encounter}");
        }
    }
}

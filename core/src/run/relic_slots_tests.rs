//! Unlimited relic slots (decision of 03.11.2026, DESIGN 8.0) and the pot
//! of the "Stammtischkasse" paid out when the relic leaves (DESIGN 8.7).

use super::bot::RunBot;
use super::content_tests::{content, relic_run};
use super::*;

/// `count` owned relics, none of them `except`, the kitty first if asked.
fn many_relics(content: &Content, count: usize, kitty: bool, except: &[RelicId]) -> Vec<RelicId> {
    let kitty_id = RelicId::new("kitty");
    let mut relics: Vec<RelicId> = Vec::new();
    if kitty {
        relics.push(kitty_id.clone());
    }
    relics.extend(
        content
            .relics
            .keys()
            .filter(|id| **id != kitty_id && !except.contains(id))
            .take(count - relics.len())
            .cloned(),
    );
    relics
}

/// A run at the Tresen with `relics` and plenty of Groschen.
fn at_shop(content: &Content, relics: Vec<RelicId>) -> Run {
    let mut run = relic_run(content, &[]);
    run.relics = relics;
    run.coins = 100;
    run.game = None;
    run.phase = RunPhase::Shop;
    run.shop = Some(run.draw_offer(content, 0));
    run
}

/// A run at the relic pick of a won boss pub, offering `offers`.
fn at_boss_reward(run: &mut Run, offers: Vec<RelicId>) {
    run.game = None;
    run.phase = RunPhase::BossReward;
    run.boss_reward = Some(tour::BossReward { relics: offers });
}

fn paid_out(events: &[RunEvent]) -> Option<(RelicId, u32)> {
    events.iter().find_map(|e| match e {
        RunEvent::StashPaidOut { relic, amount, .. } => Some((relic.clone(), *amount)),
        _ => None,
    })
}

#[test]
fn relic_slots_are_unlimited_by_default() {
    let content = content();
    assert_eq!(content.run.relic_slots, None);
    for character in content.characters.keys() {
        assert_eq!(content.relic_slots(*character), None, "{character}");
        assert!(content.relic_slot_free(*character, 1000));
    }
    // The Tresen sells a sixth, seventh, ... relic.
    let mut run = at_shop(&content, many_relics(&content, 8, false, &[]));
    let view = run.view(&content);
    assert_eq!(view.relic_slots, None);
    let offer = view.shop.expect("shop").relics[0].clone().expect("offer");
    assert!(offer.can_buy);
    run.apply(&content, RunAction::BuyRelic(0)).expect("buy");
    assert_eq!(run.relics().len(), 9);
    // Selling stays possible.
    run.apply(&content, RunAction::SellRelic(0)).expect("sell");
    assert_eq!(run.relics().len(), 8);
}

#[test]
fn an_encounter_gives_a_relic_whatever_is_owned() {
    let content = content();
    let mut run = relic_run(&content, &[]);
    let new = RelicId::new("old_lady");
    run.relics = many_relics(&content, 8, false, std::slice::from_ref(&new));
    run.game = None;
    run.phase = RunPhase::Encounter;
    run.encounter = Some(ActiveEncounter {
        id: EncounterId::new("spelunke_loot"),
        options: vec![
            EncounterOption {
                cost: 0,
                effects: vec![EncounterEffect::Relic(Some(new.clone()))],
                fallback: false,
            },
            EncounterOption {
                cost: 0,
                effects: vec![EncounterEffect::Coins(3)],
                fallback: false,
            },
        ],
        chosen: None,
    });
    assert_eq!(
        run.view(&content).encounter.expect("open").options[0].blocked,
        None
    );
    // The bot takes it too: there is always a free slot.
    let action = RunBot.encounter_action(&run, &content);
    assert_eq!(
        action,
        RunAction::ChooseEncounter {
            option: 0,
            target: None
        }
    );
    run.apply(&content, action).expect("take");
    assert_eq!(run.relics().len(), 9);
    assert!(run.relics().contains(&new));
}

#[test]
fn the_boss_pick_never_needs_a_replace_but_still_allows_one() {
    let content = content();
    let offers: Vec<RelicId> = content.relics.keys().rev().take(3).cloned().collect();
    let mut run = relic_run(&content, &[]);
    run.relics = many_relics(&content, 8, false, &offers);
    at_boss_reward(&mut run, offers.clone());
    assert!(!run.view(&content).boss_reward.expect("pick").slots_full);
    // The bot takes its pick without giving anything up.
    let RunAction::ChooseBossRelic { offer, replace } = RunBot::boss_reward_action(&run, &content)
    else {
        panic!("a boss pick");
    };
    assert!(offer.is_some());
    assert_eq!(replace, None);
    let mut taken = run.clone();
    taken
        .apply(
            &content,
            RunAction::ChooseBossRelic {
                offer: Some(0),
                replace: None,
            },
        )
        .expect("take");
    assert_eq!(taken.relics().len(), 9);
    // Replacing is still possible: the old relic is sold for half.
    run.apply(
        &content,
        RunAction::ChooseBossRelic {
            offer: Some(0),
            replace: Some(0),
        },
    )
    .expect("replace");
    assert_eq!(run.relics().len(), 8);
    assert!(run.relics().contains(&offers[0]));
}

#[test]
fn a_character_cap_still_limits_the_relics() {
    let mut content = content();
    content
        .characters
        .get_mut(&Character::Declarer)
        .expect("declarer")
        .rule
        .relic_slots = Some(2);
    let mut run = at_shop(&content, many_relics(&content, 2, false, &[]));
    assert_eq!(run.view(&content).relic_slots, Some(2));
    assert_eq!(
        run.apply(&content, RunAction::BuyRelic(0)),
        Err(RunError::NoFreeSlot)
    ); // An encounter relic is blocked, and the bot does not count on it.
    let new = RelicId::new("old_lady");
    run.relics = many_relics(&content, 2, false, std::slice::from_ref(&new));
    run.shop = None;
    run.phase = RunPhase::Encounter;
    let option = EncounterOption {
        cost: 0,
        effects: vec![EncounterEffect::Relic(Some(new))],
        fallback: false,
    };
    assert_eq!(RunBot::option_value(&run, &content, &option), 0);
    run.encounter = Some(ActiveEncounter {
        id: EncounterId::new("spelunke_loot"),
        options: vec![option],
        chosen: None,
    });
    let view = run.view(&content).encounter.expect("open");
    assert_eq!(view.options[0].blocked, Some(OptionBlock::RelicSlotsFull));
}

#[test]
fn selling_the_kitty_pays_out_its_pot_first() {
    let content = content();
    let mut run = at_shop(&content, many_relics(&content, 3, true, &[]));
    run.counters.insert("kitty".into(), 4);
    let coins = run.coins();
    let price = sell_price(content.relic(&RelicId::new("kitty")).expect("kitty").price);
    let events = run.apply(&content, RunAction::SellRelic(0)).expect("sell");
    assert_eq!(paid_out(&events), Some((RelicId::new("kitty"), 4)));
    let paid = events
        .iter()
        .position(|e| matches!(e, RunEvent::StashPaidOut { .. }));
    let sold = events
        .iter()
        .position(|e| matches!(e, RunEvent::RelicSold { .. }));
    assert!(paid < sold, "the pot first: {events:?}");
    assert_eq!(run.coins(), coins + 4 + price, "as it is, no bonus");
    assert!(!run.counters.contains_key("kitty"), "the pot is empty");
    // Selling another relic pays nothing; an empty pot pays nothing.
    run.counters.insert("kitty".into(), 0);
    let events = run.apply(&content, RunAction::SellRelic(0)).expect("sell");
    assert_eq!(paid_out(&events), None);
}

/// A run at "Der alte Bekannte" owning `relics`, `pot` Groschen in the
/// kitty, with a single option doing `effect`.
fn at_old_friend(content: &Content, relics: &[&str], pot: u32, effect: EncounterEffect) -> Run {
    let mut run = relic_run(content, relics);
    run.counters.insert("kitty".into(), pot);
    run.game = None;
    run.phase = RunPhase::Encounter;
    run.encounter = Some(ActiveEncounter {
        id: EncounterId::new("old_friend"),
        options: vec![EncounterOption {
            cost: 0,
            effects: vec![effect],
            fallback: false,
        }],
        chosen: None,
    });
    run
}

#[test]
fn swapping_selling_or_replacing_the_kitty_pays_out_its_pot() {
    let content = content();
    let kitty = RelicId::new("kitty");
    let pick_kitty = RunAction::ChooseEncounter {
        option: 0,
        target: Some(0),
    };
    // "Der alte Bekannte" swaps it away.
    let swap = EncounterEffect::SwapRelic(Some(RelicId::new("old_lady")));
    let mut run = at_old_friend(&content, &["kitty", "kontra"], 6, swap);
    let coins = run.coins();
    let events = run.apply(&content, pick_kitty.clone()).expect("swap");
    assert_eq!(paid_out(&events), Some((kitty.clone(), 6)));
    assert_eq!(run.coins(), coins + 6);
    assert!(!run.counters.contains_key("kitty"));
    assert_eq!(run.relics()[0], RelicId::new("old_lady"));
    // ... or buys it at full price.
    let mut run = at_old_friend(
        &content,
        &["kitty", "kontra"],
        7,
        EncounterEffect::SellRelic(100),
    );
    let coins = run.coins();
    let price = content.relic(&kitty).expect("kitty").price;
    let events = run.apply(&content, pick_kitty).expect("sell");
    assert_eq!(paid_out(&events), Some((kitty.clone(), 7)));
    assert_eq!(run.coins(), coins + 7 + price);
    assert!(!run.counters.contains_key("kitty"));
    assert!(!run.relics().contains(&kitty));
    // The boss pick replaces it.
    let mut run = relic_run(&content, &["kontra", "kitty"]);
    run.counters.insert("kitty".into(), 5);
    at_boss_reward(&mut run, vec![RelicId::new("old_lady")]);
    let coins = run.coins();
    let price = sell_price(content.relic(&kitty).expect("kitty").price);
    let events = run
        .apply(
            &content,
            RunAction::ChooseBossRelic {
                offer: Some(0),
                replace: Some(1),
            },
        )
        .expect("replace");
    assert_eq!(paid_out(&events), Some((kitty, 5)));
    assert_eq!(run.coins(), coins + 5 + price);
}

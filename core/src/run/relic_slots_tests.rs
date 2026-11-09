//! Eight relic slots (decision of 05.11.2026, DESIGN 8.0; unlimited from
//! 03.11.2026), replacing one when every slot is full, and the pot
//! of the "Stammtischkasse" paid out when the relic leaves (DESIGN 8.7).

use super::bot::RunBot;
use super::content_tests::{content, relic_run};
use super::*;
use crate::content::EncounterEffect;
use crate::run::view::OptionTarget;

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
fn eight_relic_slots_and_the_tresen_replaces_one() {
    let content = content();
    assert_eq!(content.run.relic_slots, Some(8));
    for character in content.characters.keys() {
        assert_eq!(content.relic_slots(*character), Some(8), "{character}");
        assert!(content.relic_slot_free(*character, 7));
        assert!(!content.relic_slot_free(*character, 8));
    }
    // Seven owned: the eighth is bought as usual.
    let mut run = at_shop(&content, many_relics(&content, 7, false, &[]));
    let view = run.view(&content);
    assert_eq!(view.relic_slots, Some(8));
    let offer = view.shop.expect("shop").relics[0].clone().expect("offer");
    assert!(offer.can_buy);
    assert!(offer.replace.is_empty(), "a slot is free");
    run.apply(&content, RunAction::BuyRelic(0)).expect("buy");
    assert_eq!(run.relics().len(), 8);
    // Eight owned: no plain purchase, but in place of an owned one.
    let offer = run.view(&content).shop.expect("shop").relics[1]
        .clone()
        .expect("offer");
    assert!(!offer.can_buy);
    assert_eq!(offer.replace, (0..8).collect::<Vec<_>>());
    assert_eq!(
        run.clone().apply(&content, RunAction::BuyRelic(1)),
        Err(RunError::NoFreeSlot)
    );
    let old = run.relics()[2].clone();
    let refund = sell_price(content.relic(&old).expect("old").price);
    let coins = run.coins();
    let events = run
        .apply(
            &content,
            RunAction::ReplaceRelic {
                slot: 1,
                replace: 2,
            },
        )
        .expect("replace");
    assert_eq!(run.relics().len(), 8);
    assert!(!run.relics().contains(&old));
    assert!(run.relics().contains(&offer.relic.id));
    assert_eq!(run.coins(), coins + refund - offer.price);
    let sold = events
        .iter()
        .position(|e| matches!(e, RunEvent::RelicSold { .. }));
    let bought = events
        .iter()
        .position(|e| matches!(e, RunEvent::RelicBought { .. }));
    assert!(sold.is_some() && sold < bought, "sold first: {events:?}");
    // All or nothing: too poor even with the refund, nothing changes.
    let tap = RelicId::new("tap");
    let mut poor = at_shop(
        &content,
        many_relics(&content, 8, false, std::slice::from_ref(&tap)),
    );
    let shop = poor.shop.as_mut().expect("shop");
    shop.relics = vec![Some(tap)];
    shop.fence = None;
    poor.coins = 0;
    let before = poor.clone();
    let replace = RunAction::ReplaceRelic {
        slot: 0,
        replace: 0,
    };
    assert_eq!(poor.apply(&content, replace), Err(RunError::NotEnoughCoins));
    assert_eq!(poor.relics(), before.relics());
    assert_eq!(poor.coins(), 0);
    assert!(
        poor.view(&content).shop.expect("shop").relics[0]
            .clone()
            .expect("offer")
            .replace
            .is_empty()
    );
    // Without a cap (`relic_slots: None`) any number, as from 03.11.2026.
    let mut unlimited = content.clone();
    unlimited.run.relic_slots = None;
    let mut run = at_shop(&unlimited, many_relics(&unlimited, 8, false, &[]));
    run.apply(&unlimited, RunAction::BuyRelic(0))
        .expect("ninth");
    assert_eq!(run.relics().len(), 9);
}

/// The Spelunke loot offering `new` (or its Groschen).
fn at_loot(run: &mut Run, new: &RelicId) {
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
}

#[test]
fn an_encounter_relic_with_every_slot_full_replaces_one() {
    let content = content();
    let new = RelicId::new("old_lady");
    // A free slot: taken without a target.
    let mut run = relic_run(&content, &[]);
    run.relics = many_relics(&content, 7, false, std::slice::from_ref(&new));
    at_loot(&mut run, &new);
    let view = run.view(&content).encounter.expect("open");
    assert_eq!(view.options[0].blocked, None);
    assert_eq!(view.options[0].target, OptionTarget::None);
    run.apply(
        &content,
        RunAction::ChooseEncounter {
            option: 0,
            target: None,
            marke: None,
        },
    )
    .expect("take");
    assert_eq!(run.relics().len(), 8);
    // Every slot full: an owned relic makes room, sold for half its price.
    let mut run = relic_run(&content, &[]);
    run.relics = many_relics(&content, 8, false, std::slice::from_ref(&new));
    at_loot(&mut run, &new);
    let view = run.view(&content).encounter.expect("open");
    assert_eq!(view.options[0].blocked, None);
    assert_eq!(view.options[0].target, OptionTarget::Relic);
    assert_eq!(view.options[0].targets.len(), 8);
    let take = |target| RunAction::ChooseEncounter {
        option: 0,
        target,
        marke: None,
    };
    assert_eq!(
        run.clone().apply(&content, take(None)),
        Err(RunError::TargetRequired)
    );
    let old = run.relics()[3].clone();
    let coins = run.coins();
    run.apply(&content, take(Some(3))).expect("replace");
    assert_eq!(run.relics().len(), 8);
    assert!(run.relics().contains(&new) && !run.relics().contains(&old));
    let refund = sell_price(content.relic(&old).expect("old").price);
    assert_eq!(run.coins(), coins + refund);
}

#[test]
fn the_bot_replaces_only_for_a_clearly_better_relic() {
    let content = content();
    // Boss pick: the Alte Dame (10) in place of the Spickzettel (1).
    let owned = [
        "tap",
        "crib_sheet",
        "beer_mat",
        "true_soul",
        "dive_expert",
        "hot_hand",
        "tour",
        "virgin",
    ];
    let mut run = relic_run(&content, &owned);
    at_boss_reward(&mut run, vec![RelicId::new("old_lady")]);
    assert!(run.view(&content).boss_reward.expect("pick").slots_full);
    let action = RunBot::boss_reward_action(&run, &content);
    let RunAction::ChooseBossRelic {
        offer: Some(0),
        replace: Some(index),
    } = action
    else {
        panic!("a replace: {action:?}");
    };
    assert_eq!(run.relics()[index], RelicId::new("crib_sheet"));
    run.apply(&content, action).expect("replace");
    assert_eq!(run.relics().len(), 8);
    // The loot: replaced too, the Groschen otherwise.
    let mut run = relic_run(&content, &owned);
    let new = RelicId::new("old_lady");
    at_loot(&mut run, &new);
    let action = RunBot.encounter_action(&run, &content);
    assert!(
        matches!(
            action,
            RunAction::ChooseEncounter {
                option: 0,
                target: Some(_),
                ..
            }
        ),
        "{action:?}"
    );
    run.apply(&content, action).expect("take");
    assert!(run.relics().contains(&new));
    assert!(!run.relics().contains(&RelicId::new("crib_sheet")));
    assert_eq!(run.relics().len(), 8);
    // Not clearly better (the Kiebitz, 3, for the Spickzettel, 1): the
    // Groschen instead.
    let mut run = relic_run(&content, &owned);
    at_loot(&mut run, &RelicId::new("kibitzer"));
    let action = RunBot.encounter_action(&run, &content);
    assert!(
        matches!(action, RunAction::ChooseEncounter { option: 1, .. }),
        "{action:?}"
    );
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
    );
    // An encounter relic needs one to replace; the bot counts the gain.
    let new = RelicId::new("old_lady");
    run.relics = vec![RelicId::new("crib_sheet"), RelicId::new("beer_mat")];
    run.shop = None;
    run.phase = RunPhase::Encounter;
    let option = EncounterOption {
        cost: 0,
        effects: vec![EncounterEffect::Relic(Some(new))],
        fallback: false,
    };
    assert_eq!(RunBot::option_value(&run, &content, &option), 9);
    assert!(run.option_needs_relic(&content, &option));
    run.encounter = Some(ActiveEncounter {
        id: EncounterId::new("spelunke_loot"),
        options: vec![option],
        chosen: None,
    });
    let view = run.view(&content).encounter.expect("open");
    assert_eq!(view.options[0].blocked, None);
    assert_eq!(view.options[0].target, OptionTarget::Relic);
    // No slot at all and none owned: blocked.
    content
        .characters
        .get_mut(&Character::Declarer)
        .expect("declarer")
        .rule
        .relic_slots = Some(0);
    run.relics.clear();
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
        marke: None,
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

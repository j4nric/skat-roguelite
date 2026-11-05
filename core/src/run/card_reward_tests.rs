//! The card reward after a won pub (Kartenbelohnung, DESIGN 5.0, decided
//! by the user on 04.11.2026, Cockpit q24): when it opens, what it offers,
//! taking and raising, skipping, the guard, its own generator, old saves
//! and the bot.

use super::bot::RunBot;
use super::card_reward::CardReward;
use super::tour::Spelunke;
use super::*;
use crate::content::{MalusId, ModKind, Rarity};

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
    if run.schnapps_reward().is_some() {
        events.extend(
            run.apply(content, RunAction::ChooseSchnappsReward { give_up: None })
                .expect("sell"),
        );
    }
    events
}

fn position(events: &[RunEvent], test: impl Fn(&RunEvent) -> bool) -> Option<usize> {
    events.iter().position(test)
}

fn id(name: &str) -> CardModId {
    CardModId::new(name)
}

/// The first card (instance id) of the printed card `name` ("SA", "HJ").
fn card(run: &Run, name: &str) -> u32 {
    let printed: crate::cards::Card = name.parse().expect("card");
    run.edition()
        .cards()
        .iter()
        .find(|c| c.kind == CardKind::Normal(printed))
        .expect("in the edition")
        .id
}

/// Put an open card reward with these offers on the run.
fn offer(run: &mut Run, offers: &[(&str, i32)]) {
    run.phase = RunPhase::CardReward;
    run.game = None;
    run.card_reward = Some(CardReward {
        offers: offers.iter().map(|(m, v)| (id(m), *v)).collect(),
    });
}

#[test]
fn a_won_pub_offers_three_zinken_after_the_schnaps() {
    let content = content();
    let mut run = start_at(&content, 5, 0, 0);
    let events = end_pub(&mut run, &content, 1, true);
    assert_eq!(run.phase(), RunPhase::CardReward);
    let reward = run.card_reward().expect("open").clone();
    assert_eq!(reward.offers.len(), 3);
    let won = position(&events, |e| matches!(e, RunEvent::SeriesWon { .. })).expect("won");
    let schnaps =
        position(&events, |e| matches!(e, RunEvent::SchnappsRewarded { .. })).expect("schnaps");
    let offered =
        position(&events, |e| matches!(e, RunEvent::CardRewardOffered { .. })).expect("offered");
    assert!(won < schnaps && schnaps < offered);
    assert!(
        position(&events, |e| matches!(e, RunEvent::RouteOpened { .. })).is_none(),
        "the route waits for the reward"
    );
    let view = run.view(&content).card_reward.expect("view");
    assert_eq!(view.offers.len(), 3);
    assert_eq!(view.skip_coins, 2);
    for (i, o) in view.offers.iter().enumerate() {
        assert_eq!(o.index, i);
        assert!(!o.targets.is_empty(), "{:?} fits some card", o.card_mod.id);
        assert_eq!(o.targets, run.card_reward_targets(&content, i));
    }
}

#[test]
fn offers_are_different_never_curses_and_rare_about_one_in_ten() {
    let content = content();
    let mut offers = 0u32;
    let mut rare = 0u32;
    let mut specials = 0u32;
    let mut vouchers = 0u32;
    for seed in 0..300 {
        let mut run = start_at(&content, seed, 0, 0);
        end_pub(&mut run, &content, 1, true);
        let reward = run.card_reward().expect("open");
        let mut seen: Vec<&CardModId> = Vec::new();
        for (i, (m, value)) in reward.offers.iter().enumerate() {
            let def = content.card_mod(m).expect("known");
            assert!(!def.is_curse(), "seed {seed}: curse {m}");
            assert!(!seen.contains(&m), "seed {seed}: {m} twice");
            seen.push(m);
            assert_eq!(*value, def.offer_value(), "the Auslage value");
            assert!(!run.card_reward_targets(&content, i).is_empty());
            offers += 1;
            rare += u32::from(def.rarity == Rarity::Rare);
            specials += u32::from(def.kind == ModKind::Special);
            vouchers += u32::from(def.kind == ModKind::Voucher);
        }
        // At most one offer per special card, its Marke included.
        let undead = reward
            .offers
            .iter()
            .filter(|(m, _)| m.as_str() == "undead" || m.as_str() == "voucher_undead")
            .count();
        let wild = reward
            .offers
            .iter()
            .filter(|(m, _)| m.as_str() == "wild_card" || m.as_str() == "voucher_wild")
            .count();
        assert!(undead <= 1 && wild <= 1, "seed {seed}: {reward:?}");
    }
    assert_eq!(offers, 900);
    // Rare: the class (10 %) plus the special cards (5 % per offer).
    let pct = rare * 100 / offers;
    assert!((8..=22).contains(&pct), "rare {pct} %");
    assert!(specials > 0 && specials < 90, "specials {specials}");
    assert!(vouchers < offers / 4, "Marken are rarer: {vouchers}");
}

#[test]
fn taking_puts_the_zinke_on_the_card_then_the_route() {
    let content = content();
    let mut run = start_at(&content, 5, 0, 0);
    end_pub(&mut run, &content, 1, true);
    offer(&mut run, &[("points", 3), ("trump_bonus", 2)]);
    let ace = card(&run, "SA");
    let coins = run.coins();
    let events = run
        .apply(
            &content,
            RunAction::TakeCardReward {
                offer: 1,
                card: ace,
            },
        )
        .expect("take");
    assert!(events.contains(&RunEvent::CardRewardTaken {
        offer: 1,
        card: ace,
        card_mod: id("trump_bonus"),
        value: 2,
    }));
    assert!(events.iter().any(|e| matches!(e, RunEvent::DeckChanged(_))));
    let deck_card = run.edition().get(ace).expect("ace");
    assert!(deck_card.mods.contains(&id("trump_bonus")));
    assert_eq!(deck_card.mod_value(&id("trump_bonus")), 2);
    assert_eq!(run.coins(), coins, "free");
    assert_eq!(run.card_reward(), None);
    assert_eq!(run.phase(), RunPhase::Route);
}

#[test]
fn the_same_zinke_rises_another_is_replaced() {
    let content = content();
    let mut run = start_at(&content, 5, 0, 0);
    end_pub(&mut run, &content, 1, true);
    let ace = card(&run, "SA");
    run.edition
        .add_mod(&content, ace, &id("points"), 3)
        .expect("points");
    offer(&mut run, &[("points", 3)]);
    run.apply(
        &content,
        RunAction::TakeCardReward {
            offer: 0,
            card: ace,
        },
    )
    .expect("raise");
    assert_eq!(
        run.edition()
            .get(ace)
            .expect("ace")
            .mod_value(&id("points")),
        6
    );
    // Another Zinke replaces it.
    offer(&mut run, &[("first_trick", 2)]);
    run.apply(
        &content,
        RunAction::TakeCardReward {
            offer: 0,
            card: ace,
        },
    )
    .expect("replace");
    let mods = &run.edition().get(ace).expect("ace").mods;
    assert!(mods.contains(&id("first_trick")) && !mods.contains(&id("points")));
}

#[test]
fn a_card_that_does_not_take_it_is_refused_and_nothing_changes() {
    let content = content();
    let mut run = start_at(&content, 5, 0, 0);
    end_pub(&mut run, &content, 1, true);
    // Winkel never goes on a Daus.
    offer(&mut run, &[("rank", 1)]);
    let ace = card(&run, "SA");
    assert!(!run.card_reward_targets(&content, 0).contains(&ace));
    let before = run.clone();
    assert!(
        run.apply(
            &content,
            RunAction::TakeCardReward {
                offer: 0,
                card: ace
            }
        )
        .is_err()
    );
    assert_eq!(run, before);
    assert_eq!(
        run.apply(
            &content,
            RunAction::TakeCardReward {
                offer: 5,
                card: ace
            }
        ),
        Err(RunError::InvalidSlot)
    );
    // Nothing else goes before the choice.
    assert_eq!(
        run.apply(&content, RunAction::ChooseRoute(0)),
        Err(RunError::CardRewardPending)
    );
    assert_eq!(
        run.apply(&content, RunAction::SellRelic(0)),
        Err(RunError::CardRewardPending)
    );
    assert_eq!(
        RunError::CardRewardPending.code().as_str(),
        "CardRewardPending"
    );
}

#[test]
fn skipping_pays_two_groschen() {
    let content = content();
    let mut run = start_at(&content, 5, 0, 0);
    end_pub(&mut run, &content, 1, true);
    assert_eq!(run.phase(), RunPhase::CardReward);
    let coins = run.coins();
    let edition = run.edition().clone();
    let events = run
        .apply(&content, RunAction::SkipCardReward)
        .expect("skip");
    assert!(events.contains(&RunEvent::CardRewardSkipped { coins: 2 }));
    assert_eq!(run.coins(), coins + 2);
    assert_eq!(run.edition(), &edition);
    assert_eq!(run.phase(), RunPhase::Route);
    // Only while a reward is open.
    assert_eq!(
        run.apply(&content, RunAction::SkipCardReward),
        Err(RunError::WrongPhase)
    );
}

#[test]
fn a_boss_pub_gives_its_relic_pick_instead() {
    let content = content();
    let mut run = start_at(&content, 5, 0, PUBS_PER_DISTRICT - 1);
    assert!(run.series().is_boss_pub());
    let events = end_pub(&mut run, &content, 3, true);
    assert_eq!(run.phase(), RunPhase::BossReward);
    assert!(position(&events, |e| matches!(e, RunEvent::CardRewardOffered { .. })).is_none());
}

#[test]
fn a_lost_pub_gives_none() {
    let content = content();
    let mut run = start_at(&content, 5, 0, 1);
    let events = end_pub(&mut run, &content, 3, false);
    assert!(position(&events, |e| matches!(e, RunEvent::CardRewardOffered { .. })).is_none());
    assert_eq!(run.card_reward(), None);
}

#[test]
fn a_spelunke_gives_the_reward_then_its_loot() {
    let content = content();
    let mut run = start_at(&content, 5, 0, 0);
    end_pub(&mut run, &content, 1, true);
    run.apply(&content, RunAction::SkipCardReward)
        .expect("skip");
    let route = run.route.as_mut().expect("route");
    route.options[0].stops.clear();
    route.options[0].spelunke = Some(Spelunke {
        name: "Spelunke Zum Test".into(),
        malus: MalusId::new("no_redeal"),
        malus_options: Vec::new(),
    });
    run.apply(&content, RunAction::ChooseRoute(0))
        .expect("spelunke");
    end_pub(&mut run, &content, 2, true);
    assert_eq!(run.phase(), RunPhase::CardReward);
    let ace = card(&run, "HA");
    let offer = (0..run.card_reward().expect("open").offers.len())
        .find(|&i| run.card_reward_targets(&content, i).contains(&ace))
        .unwrap_or(0);
    let target = run.card_reward_targets(&content, offer)[0];
    run.apply(
        &content,
        RunAction::TakeCardReward {
            offer,
            card: target,
        },
    )
    .expect("take");
    assert_eq!(run.phase(), RunPhase::Encounter, "the loot follows");
    assert_eq!(
        run.encounter().map(|e| e.id.as_str()),
        Some("spelunke_loot")
    );
}

#[test]
fn without_a_card_reward_in_the_content_the_route_follows() {
    let mut content = content();
    content.run.card_reward = None;
    let mut run = start_at(&content, 5, 0, 0);
    end_pub(&mut run, &content, 1, true);
    assert_eq!(run.phase(), RunPhase::Route);
}

#[test]
fn the_reward_leaves_every_other_draw_alone() {
    let content = content();
    let mut without = content.clone();
    without.run.card_reward = None;
    for seed in 0..10 {
        let mut a = start_at(&content, seed, 0, 0);
        let mut b = a.clone();
        end_pub(&mut a, &content, 1, true);
        end_pub(&mut b, &without, 1, true);
        assert_ne!(a.reward_rng, b.reward_rng, "seed {seed}");
        let rewarded = a.clone();
        a.apply(&content, RunAction::SkipCardReward).expect("skip");
        assert_eq!(a.deal_rng, b.deal_rng, "seed {seed}");
        assert_eq!(a.shop_rng, b.shop_rng);
        assert_eq!(a.misc_rng, b.misc_rng);
        assert_eq!(a.encounter_rng, b.encounter_rng);
        assert_eq!(a.tour_rng, b.tour_rng);
        assert_eq!(a.zinken_rng, b.zinken_rng);
        assert_eq!(a.trade_rng, b.trade_rng);
        assert_eq!(a.schnapps_rng, b.schnapps_rng);
        assert_eq!(rewarded.reward_rng, a.reward_rng);
    }
}

#[test]
fn old_saves_load_and_the_open_reward_survives_a_save() {
    let content = content();
    let mut run = start_at(&content, 7, 0, 0);
    end_pub(&mut run, &content, 1, true);
    let json = serde_json::to_string(&run).expect("save");
    let loaded: Run = serde_json::from_str(&json).expect("load");
    assert_eq!(loaded, run, "the open reward survives a save");
    // A save from before the card reward: no generator, no reward.
    let mut before = start_at(&content, 7, 0, 0);
    let mut value: serde_json::Value = serde_json::to_value(&before).expect("value");
    let object = value.as_object_mut().expect("object");
    assert!(object.remove("reward_rng").is_some());
    assert!(!object.contains_key("card_reward"));
    let mut old: Run = serde_json::from_value(value).expect("old save");
    assert_eq!(old.card_reward(), None);
    end_pub(&mut old, &content, 1, true);
    end_pub(&mut before, &content, 1, true);
    assert_eq!(old.phase(), RunPhase::CardReward, "the reward comes");
    // A Zinke the content no longer knows is dropped from the offers.
    let mut gone = run.clone();
    if let Some(reward) = gone.card_reward.as_mut() {
        reward.offers.push((id("no_such_zinke"), 1));
    }
    gone.drop_unknown_content(&content);
    assert_eq!(gone.card_reward(), run.card_reward());
}

#[test]
fn the_bot_takes_its_best_zinke_and_skips_only_if_nothing_fits() {
    let content = content();
    let mut run = start_at(&content, 5, 0, 0);
    end_pub(&mut run, &content, 1, true);
    offer(
        &mut run,
        &[("last_trick", 1), ("trump_bonus", 2), ("points", 3)],
    );
    let RunAction::TakeCardReward {
        offer: taken,
        card: target,
    } = RunBot::card_reward_action(&run, &content)
    else {
        panic!("takes one");
    };
    assert_eq!(taken, 1, "Siegel is worth most to the bot");
    let deck_card = run.edition().get(target).expect("card");
    assert!(deck_card.mods.is_empty(), "a bare card");
    assert!(
        matches!(deck_card.kind, CardKind::Normal(c) if c.rank == crate::cards::Rank::Ace),
        "the card with the most Augen"
    );
    // A card with the same Zinke comes first: it rises.
    let ten = card(&run, "CT");
    run.edition
        .add_mod(&content, ten, &id("trump_bonus"), 2)
        .expect("siegel");
    assert_eq!(
        RunBot::card_reward_action(&run, &content),
        RunAction::TakeCardReward {
            offer: 1,
            card: ten
        }
    );
    // Nothing fits any card: skipped.
    offer(&mut run, &[("rank", 1)]);
    let ids: Vec<u32> = run.edition().cards().iter().map(|c| c.id).collect();
    for c in ids {
        let _ = run.edition.add_mod(&content, c, &id("battered"), 1);
    }
    assert!(run.card_reward_targets(&content, 0).is_empty());
    assert_eq!(
        RunBot::card_reward_action(&run, &content),
        RunAction::SkipCardReward
    );
}

#[test]
fn a_whole_run_with_the_bot_takes_rewards() {
    let content = content();
    let (mut run, _) = Run::new(&content, 11).expect("run");
    let mut offered = 0;
    let mut taken = 0;
    for _ in 0..20_000 {
        let Some(action) = RunBot.next_action(&run, &content) else {
            break;
        };
        let events = run.apply(&content, action).expect("bot action");
        for e in &events {
            match e {
                RunEvent::CardRewardOffered { .. } => offered += 1,
                RunEvent::CardRewardTaken { .. } => taken += 1,
                _ => {}
            }
        }
    }
    assert!(offered > 0, "a pub was won");
    assert_eq!(taken, offered, "the bot always finds a card");
}

/// Write `web/e2e/fixtures/card-reward.json`: the card reward after the
/// first pub of seed 5, with three fixed Zinken (Winkel, which no Daus
/// takes, Aufdruck, already on the Grün-Daus so that it rises, and the
/// rare Narrenmarke); the smoke test takes one, goes back to the offers
/// and skips. Run by hand: `cargo test -p skat_core
/// regenerate_card_reward_fixture -- --ignored`.
#[test]
#[ignore = "rewrites web/e2e/fixtures/card-reward.json"]
fn regenerate_card_reward_fixture() {
    let content = content();
    let mut run = start_at(&content, 5, 0, 0);
    end_pub(&mut run, &content, 1, true);
    let ace = card(&run, "SA");
    run.edition
        .add_mod(&content, ace, &id("points"), 3)
        .expect("points");
    offer(&mut run, &[("rank", 1), ("points", 3), ("voucher_wild", 1)]);
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../web/e2e/fixtures/card-reward.json");
    let out = serde_json::json!({ "version": 3, "run": run });
    std::fs::write(&path, serde_json::to_string(&out).expect("json")).expect("write");
    println!("wrote {}", path.display());
}

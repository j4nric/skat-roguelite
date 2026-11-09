//! The card reward after a won pub (Kartenbelohnung, DESIGN 5.0, decided
//! by the user on 04.11.2026; fixed pairs of card and Zinke since
//! 05.11.2026): when it opens, what it offers, taking and raising,
//! skipping, the guard, its own generator, old saves and the bot.

use super::bot::RunBot;
use super::card_reward::{CardReward, RewardOffer};
use super::tour::Spelunke;
use super::*;
use crate::content::{MalusId, ModKind, Rarity};
use crate::deck::CardKind;

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

/// Put an open card reward with these offers (printed card, Zinke, value)
/// on the run.
fn offer(run: &mut Run, offers: &[(&str, &str, i32)]) {
    run.phase = RunPhase::CardReward;
    run.game = None;
    let offers = offers
        .iter()
        .map(|(c, m, v)| RewardOffer {
            card: card(run, c),
            card_mod: id(m),
            value: *v,
        })
        .collect();
    run.card_reward = Some(CardReward::new(offers));
}

fn take(offer: usize) -> RunAction {
    RunAction::TakeCardReward { offer }
}

#[test]
fn a_won_pub_offers_three_cards_with_a_zinke_after_the_schnaps() {
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
        assert!(o.can_take, "{:?} fits its card", o.card_mod.id);
        assert_eq!(o.card.id, reward.offers[i].card);
        assert_ne!(o.result, o.card, "the card with the Zinke");
    }
}

#[test]
fn offers_are_different_never_curses_and_rare_about_one_in_ten() {
    let content = content();
    let mut offers = 0u32;
    let mut rare = 0u32;
    let mut specials = 0u32;
    let mut vouchers = 0u32;
    let mut raised = 0u32;
    for seed in 0..300 {
        let mut run = start_at(&content, seed, 0, 0);
        end_pub(&mut run, &content, 1, true);
        let reward = run.card_reward().expect("open");
        let mut zinken: Vec<&CardModId> = Vec::new();
        let mut cards: Vec<u32> = Vec::new();
        for (i, o) in reward.offers.iter().enumerate() {
            let def = content.card_mod(&o.card_mod).expect("known");
            assert!(!def.is_curse(), "seed {seed}: curse {}", o.card_mod);
            assert!(!zinken.contains(&&o.card_mod), "seed {seed}: Zinke twice");
            assert!(!cards.contains(&o.card), "seed {seed}: card twice");
            zinken.push(&o.card_mod);
            cards.push(o.card);
            assert_eq!(o.value, def.offer_value(), "the Auslage value");
            assert!(run.card_reward_open(&content, i), "seed {seed}: fits");
            offers += 1;
            rare += u32::from(def.rarity == Rarity::Rare);
            specials += u32::from(def.kind == ModKind::Special);
            vouchers += u32::from(def.kind == ModKind::Voucher);
            raised += u32::from(
                run.edition()
                    .get(o.card)
                    .is_some_and(|c| c.mods.contains(&o.card_mod)),
            );
        }
    }
    assert_eq!(offers, 900);
    // Rare: the class (10 %) plus the special cards (5 % per offer).
    let pct = rare * 100 / offers;
    assert!((8..=22).contains(&pct), "rare {pct} %");
    assert!(specials > 0 && specials < 90, "specials {specials}");
    assert!(vouchers < offers / 4, "Marken are rarer: {vouchers}");
    // A fresh edition carries no Zinke yet.
    assert_eq!(raised, 0);
}

#[test]
fn the_card_is_drawn_among_the_cards_the_zinke_fits() {
    let content = content();
    let mut seen = std::collections::BTreeSet::new();
    for seed in 0..200 {
        let mut run = start_at(&content, seed, 0, 0);
        // Every card but the Grün-Daus carries a curse: only it takes a
        // normal Zinke (a special card goes on a cursed card all the same),
        // and no card comes twice.
        let ace = card(&run, "SA");
        let ids: Vec<u32> = run.edition().cards().iter().map(|c| c.id).collect();
        for c in ids.into_iter().filter(|&c| c != ace) {
            run.edition
                .add_mod(&content, c, &id("battered"), 1)
                .expect("curse");
        }
        end_pub(&mut run, &content, 1, true);
        let reward = run.card_reward().expect("open");
        let normal: Vec<&RewardOffer> = reward
            .offers
            .iter()
            .filter(|o| !content.card_mod(&o.card_mod).expect("known").is_special())
            .collect();
        assert!(normal.len() <= 1, "seed {seed}: no card twice");
        assert!(normal.iter().all(|o| o.card == ace), "seed {seed}");
        let mut cards: Vec<u32> = reward.offers.iter().map(|o| o.card).collect();
        cards.sort_unstable();
        cards.dedup();
        assert_eq!(cards.len(), reward.offers.len(), "seed {seed}");
        // A plain deck: the cards vary.
        let mut plain = start_at(&content, seed, 0, 0);
        end_pub(&mut plain, &content, 1, true);
        seen.extend(
            plain
                .card_reward()
                .expect("open")
                .offers
                .iter()
                .map(|o| o.card),
        );
    }
    assert!(seen.len() > 20, "random cards: {}", seen.len());
}

#[test]
fn a_zinke_comes_twice_only_when_no_other_has_a_free_card() {
    // A catalogue with Aufdruck as the only Zinke: three offers of it, on
    // three different cards.
    let mut content = content();
    content.card_mods.retain(|m, def| {
        m.as_str() == "points" || def.is_curse() || def.kind == ModKind::SpecialCurse
    });
    let mut run = start_at(&content, 5, 0, 0);
    end_pub(&mut run, &content, 1, true);
    let reward = run.card_reward().expect("open");
    assert_eq!(reward.offers.len(), 3);
    assert!(reward.offers.iter().all(|o| o.card_mod == id("points")));
    let mut cards: Vec<u32> = reward.offers.iter().map(|o| o.card).collect();
    cards.sort_unstable();
    cards.dedup();
    assert_eq!(cards.len(), 3, "no card twice");
}

#[test]
fn taking_puts_the_zinke_on_its_card_then_the_route() {
    let content = content();
    let mut run = start_at(&content, 5, 0, 0);
    end_pub(&mut run, &content, 1, true);
    offer(&mut run, &[("HA", "points", 3), ("SA", "trump_bonus", 2)]);
    let ace = card(&run, "SA");
    let coins = run.coins();
    let events = run.apply(&content, take(1)).expect("take");
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
    assert!(
        run.edition()
            .get(card(&run, "HA"))
            .expect("HA")
            .mods
            .is_empty()
    );
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
    offer(&mut run, &[("SA", "points", 3), ("SA", "first_trick", 2)]);
    let view = run.view(&content).card_reward.expect("view");
    assert!(view.offers[0].raises && view.offers[0].replaces.is_none());
    assert_eq!(view.offers[0].result.mod_value(&id("points")), 6);
    assert!(!view.offers[1].raises);
    assert_eq!(view.offers[1].replaces, Some(id("points")));
    run.apply(&content, take(0)).expect("raise");
    assert_eq!(
        run.edition()
            .get(ace)
            .expect("ace")
            .mod_value(&id("points")),
        6
    );
    // Another Zinke replaces it.
    offer(&mut run, &[("SA", "first_trick", 2)]);
    run.apply(&content, take(0)).expect("replace");
    let mods = &run.edition().get(ace).expect("ace").mods;
    assert!(mods.contains(&id("first_trick")) && !mods.contains(&id("points")));
}

#[test]
fn an_offer_its_card_does_not_take_is_refused_and_nothing_changes() {
    let content = content();
    let mut run = start_at(&content, 5, 0, 0);
    end_pub(&mut run, &content, 1, true);
    // Winkel never goes on a Daus.
    offer(&mut run, &[("SA", "rank", 1)]);
    assert!(!run.card_reward_open(&content, 0));
    let view = run.view(&content).card_reward.expect("view");
    assert!(!view.offers[0].can_take);
    let before = run.clone();
    assert!(run.apply(&content, take(0)).is_err());
    assert_eq!(run, before);
    assert_eq!(run.apply(&content, take(5)), Err(RunError::InvalidSlot));
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
    run.apply(&content, take(0)).expect("take");
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
        reward.offers.push(RewardOffer {
            card: 3,
            card_mod: id("no_such_zinke"),
            value: 1,
        });
    }
    gone.drop_unknown_content(&content);
    assert_eq!(gone.card_reward(), run.card_reward());
}

#[test]
fn an_open_reward_of_bare_zinken_is_drawn_again_as_pairs() {
    // Saves from before 05.11.2026: the offers were Zinken without cards.
    let content = content();
    let mut run = start_at(&content, 7, 0, 0);
    end_pub(&mut run, &content, 1, true);
    let mut value = serde_json::to_value(&run).expect("value");
    value["card_reward"] = serde_json::json!({
        "offers": [["points", 3], ["rank", 1], ["voucher_wild", 1]]
    });
    let json = serde_json::to_string(&value).expect("json");
    let load = || {
        let mut old: Run = serde_json::from_str(&json).expect("old save");
        old.drop_unknown_content(&content);
        old
    };
    let old = load();
    assert_eq!(old.phase(), RunPhase::CardReward);
    let reward = old.card_reward().expect("open");
    assert_eq!(reward.offers.len(), 3, "drawn again");
    for i in 0..3 {
        assert!(old.card_reward_open(&content, i));
    }
    assert_eq!(load(), old, "the same save gives the same offers");
    // It plays on and saves in the new shape.
    let saved = serde_json::to_string(&old).expect("save");
    let again: Run = serde_json::from_str(&saved).expect("load");
    assert_eq!(again, old);
    let mut old = old;
    old.apply(&content, take(0)).expect("take");
    assert_eq!(old.phase(), RunPhase::Route);
}

#[test]
fn the_bot_takes_its_best_offer_and_skips_if_none_is_worth_it() {
    let content = content();
    let mut run = start_at(&content, 5, 0, 0);
    end_pub(&mut run, &content, 1, true);
    offer(
        &mut run,
        &[
            ("HA", "last_trick", 1),
            ("C7", "trump_bonus", 2),
            ("SA", "points", 3),
        ],
    );
    assert_eq!(
        RunBot::card_reward_action(&run, &content),
        take(1),
        "Siegel is worth most to the bot"
    );
    // With a Siegel on the C7, an Aufdruck there would replace it.
    let seven = card(&run, "C7");
    run.edition
        .add_mod(&content, seven, &id("trump_bonus"), 2)
        .expect("siegel");
    offer(&mut run, &[("C7", "points", 3), ("SA", "last_trick", 1)]);
    assert_eq!(
        RunBot::card_reward_action(&run, &content),
        take(1),
        "never a worse Zinke in place of a better one"
    );
    // The same Zinke rises: worth taking.
    offer(
        &mut run,
        &[("C7", "trump_bonus", 2), ("SA", "trump_bonus", 2)],
    );
    assert_eq!(RunBot::card_reward_action(&run, &content), take(0));
    // Only a replace for less: skipped for the Groschen.
    offer(&mut run, &[("C7", "points", 3)]);
    assert_eq!(
        RunBot::card_reward_action(&run, &content),
        RunAction::SkipCardReward
    );
    // Nothing fits: skipped.
    offer(&mut run, &[("SA", "rank", 1)]);
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
    let mut skipped = 0;
    for _ in 0..20_000 {
        let Some(action) = RunBot.next_action(&run, &content) else {
            break;
        };
        let events = run.apply(&content, action).expect("bot action");
        for e in &events {
            match e {
                RunEvent::CardRewardOffered { .. } => offered += 1,
                RunEvent::CardRewardTaken { .. } => taken += 1,
                RunEvent::CardRewardSkipped { .. } => skipped += 1,
                _ => {}
            }
        }
    }
    assert!(offered > 0, "a pub was won");
    assert!(taken > 0, "the bot takes rewards");
    assert_eq!(taken + skipped, offered);
}

/// Write `web/e2e/fixtures/card-reward.json`: the card reward after the
/// first pub of seed 5 with three fixed pairs: Winkel on the Schellen-König
/// (a bare card), Aufdruck on the Grün-Daus that has one already (it
/// rises) and the rare Narrenmarke on the Herz-König; the smoke test opens
/// the confirm sheets, takes the Aufdruck and skips once. Run by hand:
/// `cargo test -p skat_core regenerate_card_reward_fixture -- --ignored`.
/// The fixture of the old shape (bare Zinken, before 05.11.2026) stays as
/// `card-reward-v1.json` for the save check.
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
    offer(
        &mut run,
        &[
            ("DK", "rank", 1),
            ("SA", "points", 3),
            ("HK", "voucher_wild", 1),
        ],
    );
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../web/e2e/fixtures/card-reward.json");
    let out = serde_json::json!({ "version": 3, "run": run });
    std::fs::write(&path, serde_json::to_string(&out).expect("json")).expect("write");
    println!("wrote {}", path.display());
}

#[test]
fn zinken_of_a_held_relics_direction_weigh_more() {
    let content = content();
    let mut run = start_at(&content, 5, 0, 0);
    let percent = content
        .run
        .card_reward
        .as_ref()
        .expect("card reward")
        .direction_percent;
    assert!(percent > 100, "the direction is favoured");
    let tricks = content.card_mod(&id("gilt_edge")).expect("Goldkante");
    let schneider = content.card_mod(&id("points")).expect("Augen");
    let untagged = content
        .card_mods
        .values()
        .find(|m| m.tags.is_empty())
        .expect("a Zinke without direction");
    assert_eq!(run.card_reward_bias(&content, tricks), 100);
    run.relics.push(RelicId::new("fingertips"));
    assert_eq!(run.card_reward_bias(&content, tricks), percent);
    assert_eq!(run.card_reward_bias(&content, schneider), 100);
    assert_eq!(run.card_reward_bias(&content, untagged), 100);
}

#[test]
fn the_bot_prefers_zinken_and_relics_of_its_direction() {
    use super::bot::{RELIC_DIRECTION_BONUS, relic_score, relic_value, zinke_score, zinken_value};
    let content = content();
    let mut run = start_at(&content, 5, 0, 0);
    end_pub(&mut run, &content, 1, true);
    offer(&mut run, &[("S7", "rank", 1), ("H7", "lead", 1)]);
    let taken = |run: &Run| match RunBot::card_reward_action(run, &content) {
        RunAction::TakeCardReward { offer } => offer,
        other => panic!("takes one: {other:?}"),
    };
    // Without a Null relic Rang +1 is worth more than Blei.
    assert_eq!(taken(&run), 0);
    assert_eq!(
        zinke_score(&run, &content, &id("lead")),
        zinken_value(&id("lead")) * 100
    );
    // With "Senkblei" (Null) Blei wins.
    run.relics.push(RelicId::new("plumb_line"));
    assert!(zinke_score(&run, &content, &id("lead")) > zinke_score(&run, &content, &id("rank")));
    assert_eq!(taken(&run), 1);
    // A relic of a direction the bot has gets a bonus, others do not.
    let lead_pouring = RelicId::new("lead_pouring");
    let shears = RelicId::new("golden_shears");
    assert_eq!(
        relic_score(&run, &content, &lead_pouring),
        relic_value(&lead_pouring) * 100 + RELIC_DIRECTION_BONUS
    );
    assert_eq!(
        relic_score(&run, &content, &shears),
        relic_value(&shears) * 100
    );
    // A Zinke in the deck counts as well: Schneiderkreide for Schneider.
    let ace = card(&run, "CA");
    run.edition
        .add_mod(&content, ace, &id("tailors_chalk"), 1)
        .expect("chalk");
    assert_eq!(
        relic_score(&run, &content, &shears),
        relic_value(&shears) * 100 + RELIC_DIRECTION_BONUS
    );
}

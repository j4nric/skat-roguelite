//! Zinken at the Tresen (DESIGN 8.6): the Auslage, "Neu auslosen", the
//! Kartenmaler, "Karte reinigen", the Werkstatt, old saves.

use super::tour::RouteStop;
use super::zinken::{CardOffer, PainterChoice};
use super::*;
use crate::content::{CardFilter, ModKind, StopCategory};

fn content() -> Content {
    Content::load().expect("content")
}

/// A run at the route choice after a won first pub, with `coins`.
pub(super) fn at_route(content: &Content, seed: u64, coins: u32) -> Run {
    let (mut run, _) = Run::new(content, seed).expect("start");
    run.series.games_played = 1;
    run.series.score = i64::try_from(run.series.target).expect("small");
    run.game = None;
    let mut events = Vec::new();
    run.after_game(content, &mut events).expect("after game");
    run.pass_card_reward(content, &mut events);
    assert_eq!(run.phase(), RunPhase::Route);
    run.coins = coins;
    run
}

/// A run at a real Tresen (opened the way the route opens it).
pub(super) fn tresen(content: &Content, seed: u64, coins: u32) -> (Run, Vec<RunEvent>) {
    let mut run = at_route(content, seed, coins);
    let events = run.walk_to_tresen(content);
    assert_eq!(run.phase(), RunPhase::Shop);
    (run, events)
}

pub(super) fn offers(run: &Run) -> Vec<CardOffer> {
    run.shop()
        .expect("shop")
        .card_offers
        .iter()
        .flatten()
        .cloned()
        .collect()
}

pub(super) fn set_offers(run: &mut Run, offers: Vec<CardOffer>) {
    run.shop.as_mut().expect("shop").card_offers = offers.into_iter().map(Some).collect();
}

fn offer(card: u32, card_mod: &str, value: i32, price: u32) -> CardOffer {
    CardOffer {
        card,
        card_mod: CardModId::new(card_mod),
        value,
        price,
        bargain: None,
    }
}

#[test]
fn the_card_items_are_gone_from_the_tresen() {
    let content = content();
    for id in ["fresh_deck", "ink_pad", "points_3", "trump_2", "rank_1"] {
        assert!(content.consumable(&ConsumableId::new(id)).is_none(), "{id}");
    }
    for def in content.consumables.values() {
        assert!(
            !matches!(
                def.action,
                ConsumableAction::CleanseCard
                    | ConsumableAction::AddCardMod(_)
                    | ConsumableAction::ApplyCardMod { .. }
            ),
            "{}",
            def.id
        );
    }
    // The names of the user (02.11.2026); the text leads with the effect.
    for (id, name, lead) in [
        ("points", "Aufdruck", "Augen:"),
        ("rank", "Winkel", "Rang {+n}:"),
        ("trump_bonus", "Siegel", "Trumpf:"),
        ("first_trick", "Eselsohr", "Erster Stich:"),
        ("last_trick", "Kerbe", "Letzter Stich:"),
        ("voucher_coins", "Pfandmarke", "Gutschein:"),
        ("voucher_undead", "Totenmarke", "Gutschein:"),
        ("voucher_wild", "Narrenmarke", "Gutschein:"),
    ] {
        let def = content.card_mod(&CardModId::new(id)).expect(id);
        assert_eq!(def.name, name);
        assert!(def.text.starts_with(lead), "{id}: {}", def.text);
    }
    let value = |id: &str| {
        content
            .card_mod(&CardModId::new(id))
            .expect(id)
            .offer_value()
    };
    assert_eq!(
        (value("points"), value("trump_bonus"), value("rank")),
        (3, 2, 1)
    );
}

#[test]
fn the_tresen_lays_out_three_concrete_offers() {
    let content = content();
    let zinken = &content.run.shop.zinken;
    for seed in 0..40 {
        let (run, events) = tresen(&content, seed, 10);
        let drawn = events
            .iter()
            .find_map(|e| match e {
                RunEvent::CardOffersDrawn { offers } => Some(offers.clone()),
                _ => None,
            })
            .expect("drawn");
        let offers = offers(&run);
        assert_eq!(offers.len(), 3, "seed {seed}");
        assert_eq!(drawn.into_iter().flatten().collect::<Vec<_>>(), offers);
        let mut cards: Vec<u32> = offers.iter().map(|o| o.card).collect();
        cards.sort_unstable();
        cards.dedup();
        assert_eq!(cards.len(), 3, "one card at most once");
        for o in &offers {
            let def = content.card_mod(&o.card_mod).expect("mod");
            // Never a curse; a special card only as the rare special offer
            // (`special_tests`), at its own price.
            assert!(!def.is_curse(), "{}", o.card_mod);
            assert_ne!(def.kind, ModKind::SpecialCurse, "{}", o.card_mod);
            assert_eq!(o.value, def.offer_value());
            let card = run.edition().get(o.card).expect("card");
            let face = card.kind.face().expect("face");
            let premium = zinken.premium_ranks.contains(&face.rank);
            let base = if def.is_special() {
                zinken.special_price
            } else {
                def.price
            };
            let full = base + if premium { zinken.premium } else { 0 };
            // A "Schnäppchen" (curses as trade) costs half.
            match o.bargain {
                Some(before) => {
                    assert_eq!(before, full);
                    assert_eq!(o.price, crate::content::TradeConfig::half(full));
                }
                None => assert_eq!(o.price, full),
            }
        }
        // Deterministic.
        let (again, _) = tresen(&content, seed, 10);
        assert_eq!(again.shop(), run.shop());
    }
}

#[test]
fn bare_cards_are_preferred_three_to_one() {
    let content = content();
    let (mut bare, mut modded) = (0_u32, 0_u32);
    for seed in 0..200 {
        let mut run = at_route(&content, seed, 10);
        // Every second card carries Aufdruck +1.
        for id in (0..32).step_by(2) {
            run.edition
                .add_mod(&content, id, &CardModId::new("points"), 1)
                .expect("points");
        }
        run.walk_to_tresen(&content);
        for o in offers(&run) {
            if run.edition().get(o.card).is_some_and(|c| c.mods.is_empty()) {
                bare += 1;
            } else {
                modded += 1;
            }
        }
    }
    let share = bare * 100 / (bare + modded);
    // 3:1 with equal halves: 75 %, a little lower as distinct cards are drawn.
    assert!((62..=85).contains(&share), "bare {share} %");
}

#[test]
fn vouchers_are_rarer_and_curses_never_offered() {
    let content = content();
    let run = at_route(&content, 1, 0);
    let pool = run.zinken_pool(&content);
    let weight = |id: &str| {
        pool.iter()
            .find(|(m, _)| m.as_str() == id)
            .map_or(0, |(_, w)| *w)
    };
    assert_eq!(weight("voucher_coins") * 2, weight("points"));
    for (id, _) in &pool {
        let def = content.card_mod(id).expect("mod");
        assert!(!def.is_curse() && def.kind != ModKind::Special, "{id}");
    }
}

#[test]
fn an_offer_never_goes_where_it_does_not_fit() {
    let mut content = content();
    // Eselsohr only on a Daus, for this test.
    content
        .card_mods
        .get_mut(&CardModId::new("first_trick"))
        .expect("first trick")
        .only_on = Some(CardFilter {
        suit: None,
        rank: Some(Rank::Ace),
    });
    for seed in 0..120 {
        let mut run = at_route(&content, seed, 10);
        run.edition
            .add_mod(&content, 3, &CardModId::new("battered"), 5)
            .expect("curse");
        run.edition
            .add_mod(&content, 4, &CardModId::new("rank"), 2)
            .expect("rank 2");
        run.walk_to_tresen(&content);
        for o in offers(&run) {
            assert_ne!(o.card, 3, "never on a cursed card");
            assert!(
                !(o.card == 4 && o.card_mod.as_str() == "rank"),
                "never past the maximum"
            );
            if o.card_mod.as_str() == "first_trick" {
                let face = run.edition().get(o.card).and_then(|c| c.kind.face());
                assert_eq!(face.map(|f| f.rank), Some(Rank::Ace), "only_on");
            }
        }
    }
}

#[test]
fn buying_an_offer_puts_it_on_its_card() {
    let content = content();
    let (mut run, _) = tresen(&content, 3, 10);
    run.edition
        .add_mod(&content, 9, &CardModId::new("rank"), 1)
        .expect("rank");
    set_offers(
        &mut run,
        vec![
            offer(31, "points", 3, 3),
            offer(9, "trump_bonus", 2, 3),
            offer(10, "points", 3, 20),
        ],
    );
    let view = run.view(&content).shop.expect("shop");
    let first = view.card_offers[0].as_ref().expect("offer");
    assert!(first.can_buy && first.replaces.is_none() && !first.raises);
    assert_eq!(first.result.mod_value(&CardModId::new("points")), 3);
    assert_eq!(
        view.card_offers[1]
            .as_ref()
            .and_then(|o| o.replaces.clone()),
        Some(CardModId::new("rank")),
        "Siegel would replace Winkel"
    );
    assert!(
        !view.card_offers[2].as_ref().expect("offer").can_buy,
        "too dear"
    );
    let events = run
        .apply(&content, RunAction::BuyCardOffer(0))
        .expect("buy");
    assert_eq!(run.coins(), 7);
    assert!(matches!(
        &events[0],
        RunEvent::CardOfferBought {
            slot: 0,
            card: 31,
            value: 3,
            price: 3,
            ..
        }
    ));
    assert!(matches!(
        &events[1],
        RunEvent::DeckChanged(DeckChange::Modified {
            card: 31,
            value: 3,
            ..
        })
    ));
    assert_eq!(run.shop().expect("shop").card_offers[0], None);
    assert_eq!(
        run.apply(&content, RunAction::BuyCardOffer(0)),
        Err(RunError::InvalidSlot)
    );
    // Replacing: the core replaces and says so.
    let events = run
        .apply(&content, RunAction::BuyCardOffer(1))
        .expect("replace");
    assert!(events.iter().any(|e| matches!(
        e,
        RunEvent::DeckChanged(DeckChange::Modified { card: 9, replaced: Some(r), .. })
            if r.as_str() == "rank"
    )));
    assert_eq!(
        run.apply(&content, RunAction::BuyCardOffer(2)),
        Err(RunError::NotEnoughCoins)
    );
}

#[test]
fn der_wirt_takes_off_everything_with_zinken() {
    let content = content();
    let (mut run, _) = tresen(&content, 3, 20);
    run.relics.push(RelicId::new("landlord"));
    set_offers(&mut run, vec![offer(31, "points", 3, 5)]);
    let zinken = &content.run.shop.zinken;
    let view = run.view(&content).shop.expect("shop");
    let off = |p: u32| run.shop_price(&content, p);
    assert_eq!(view.card_offers[0].as_ref().map(|o| o.price), Some(4));
    assert_eq!(view.painter.price, off(zinken.painter_price));
    assert!(view.painter.price < zinken.painter_price);
    assert_eq!(view.cleanse.price, off(zinken.cleanse_price));
    assert!(view.cleanse.price < zinken.cleanse_price);
    run.apply(&content, RunAction::BuyCardOffer(0))
        .expect("buy");
    assert_eq!(run.coins(), 16);
    // 20 % off 1 is still 1 Groschen, off 2 too.
    assert_eq!(view.card_reroll_cost, 1);
}

#[test]
fn neu_auslosen_gets_dearer_at_the_same_tresen() {
    let content = content();
    let zinken = &content.run.shop.zinken;
    let (mut run, _) = tresen(&content, 5, 3);
    let before = offers(&run);
    let relics = run.shop().expect("shop").relics.clone();
    let events = run
        .apply(&content, RunAction::RerollCardOffers)
        .expect("reroll");
    assert_eq!(
        events[0],
        RunEvent::CardOffersRerolled {
            cost: zinken.reroll_cost
        }
    );
    assert!(matches!(events[1], RunEvent::CardOffersDrawn { .. }));
    assert_eq!(run.coins(), 3 - zinken.reroll_cost);
    assert_ne!(offers(&run), before);
    assert_eq!(run.shop().expect("shop").relics, relics, "relics stay");
    let second = zinken.reroll_cost + zinken.reroll_step;
    assert_eq!(
        run.view(&content).shop.expect("shop").card_reroll_cost,
        second
    );
    run.apply(&content, RunAction::RerollCardOffers)
        .expect("second");
    assert_eq!(run.coins(), 3 - zinken.reroll_cost - second);
    assert_eq!(
        run.apply(&content, RunAction::RerollCardOffers),
        Err(RunError::NotEnoughCoins)
    );
    // A Neuauslage keeps the Auslage and its price.
    run.coins = 50;
    let auslage = offers(&run);
    run.apply(&content, RunAction::Reroll).expect("Neuauslage");
    assert_eq!(offers(&run), auslage);
    assert_eq!(run.shop().expect("shop").card_rerolls, 2);
    // The next Tresen starts at the first price again.
    run.apply(&content, RunAction::LeaveShop).expect("leave");
}

#[test]
fn the_kartenmaler_offers_two_that_fit_and_waits() {
    let content = content();
    let zinken = &content.run.shop.zinken;
    let (mut run, _) = tresen(&content, 7, 20);
    let card = 31;
    let events = run
        .apply(&content, RunAction::HirePainter(card))
        .expect("hire");
    assert_eq!(run.coins(), 20 - zinken.painter_price);
    let RunEvent::PainterHired {
        card: painted,
        options,
        price,
    } = &events[0]
    else {
        panic!("{events:?}");
    };
    assert_eq!((*painted, *price), (card, zinken.painter_price));
    assert_eq!(
        options.len(),
        usize::try_from(zinken.painter_choices).expect("small")
    );
    assert_ne!(options[0].0, options[1].0, "two different ones");
    for (m, value) in options {
        let def = content.card_mod(m).expect("mod");
        assert!(!def.is_curse() && !def.is_special());
        assert_eq!(*value, def.offer_value());
    }
    // Nothing else before the choice.
    for action in [
        RunAction::LeaveShop,
        RunAction::BuyCardOffer(0),
        RunAction::CleanseCard(card),
    ] {
        assert_eq!(run.apply(&content, action), Err(RunError::PainterPending));
    }
    let view = run.view(&content).painter.expect("painter");
    assert_eq!(view.options.len(), 2);
    assert_eq!(
        run.apply(&content, RunAction::ChoosePaint(2)),
        Err(RunError::InvalidSlot)
    );
    let (chosen, value) = options[1].clone();
    let events = run
        .apply(&content, RunAction::ChoosePaint(1))
        .expect("choose");
    assert_eq!(
        events[0],
        RunEvent::PainterChosen {
            card,
            card_mod: chosen.clone(),
            value
        }
    );
    assert!(run.painter().is_none());
    assert_eq!(
        run.edition().get(card).map(|c| c.mod_value(&chosen)),
        Some(value)
    );
    assert_eq!(run.phase(), RunPhase::Shop);
    // Once per Tresen.
    assert_eq!(
        run.apply(&content, RunAction::HirePainter(30)),
        Err(RunError::AlreadyUsed)
    );
    assert!(run.view(&content).shop.expect("shop").painter.used);
    // A cursed card takes nothing, and nothing is paid.
    let (mut run, _) = tresen(&content, 7, 20);
    run.edition
        .add_mod(&content, 2, &CardModId::new("battered"), 5)
        .expect("curse");
    assert_eq!(
        run.apply(&content, RunAction::HirePainter(2)),
        Err(RunError::Deck(DeckError::Cursed(CardModId::new(
            "battered"
        ))))
    );
    assert!(
        !run.view(&content)
            .shop
            .expect("shop")
            .painter
            .targets
            .contains(&2)
    );
    run.coins = zinken.painter_price - 1;
    assert_eq!(
        run.apply(&content, RunAction::HirePainter(31)),
        Err(RunError::NotEnoughCoins)
    );
}

#[test]
fn karte_reinigen_once_per_tresen() {
    let content = content();
    let zinken = &content.run.shop.zinken;
    let (mut run, _) = tresen(&content, 8, 10);
    assert_eq!(
        run.apply(&content, RunAction::CleanseCard(4)),
        Err(RunError::Deck(DeckError::NothingToCleanse(4)))
    );
    assert_eq!(run.coins(), 10);
    for card in [4, 5] {
        run.edition
            .add_mod(&content, card, &CardModId::new("stained"), 3)
            .expect("curse");
    }
    let view = run.view(&content).shop.expect("shop");
    assert_eq!(view.cleanse.targets, vec![4, 5]);
    let events = run
        .apply(&content, RunAction::CleanseCard(4))
        .expect("cleanse");
    assert_eq!(
        events[0],
        RunEvent::CardCleansed {
            card: 4,
            price: zinken.cleanse_price
        }
    );
    assert!(matches!(
        events[1],
        RunEvent::DeckChanged(DeckChange::Cleansed { card: 4, .. })
    ));
    assert_eq!(run.coins(), 10 - zinken.cleanse_price);
    assert!(run.edition().get(4).is_some_and(|c| c.mods.is_empty()));
    assert_eq!(
        run.apply(&content, RunAction::CleanseCard(5)),
        Err(RunError::AlreadyUsed)
    );
}

#[test]
fn the_werkstatt_has_the_kartenmaler_for_free() {
    let content = content();
    let mut run = at_route(&content, 11, 0);
    let route = run.route.as_mut().expect("route");
    for option in &mut route.options {
        option.stops = vec![RouteStop {
            category: StopCategory::Upgrade,
            encounter: Some(EncounterId::new("workshop")),
        }];
        option.spelunke = None;
    }
    run.apply(&content, RunAction::ChooseRoute(0))
        .expect("walk");
    let encounter = run.encounter().expect("workshop").clone();
    let effects: Vec<&[EncounterEffect]> =
        encounter.options.iter().map(|o| &o.effects[..]).collect();
    assert_eq!(
        effects,
        [
            &[EncounterEffect::Painter][..],
            &[EncounterEffect::Cleanse][..],
            // Since 04.11.2026 also a random Marke on a chosen card.
            &[EncounterEffect::Marke {
                card: crate::content::CardPick::Chosen,
                marke: crate::content::CardPick::Random,
            }][..]
        ]
    );
    let view = run.view(&content).encounter.expect("view");
    assert!(
        view.options[0].target == super::view::OptionTarget::Card
            && view.options[0].targets.contains(&31)
    );
    let events = run
        .apply(
            &content,
            RunAction::ChooseEncounter {
                option: 0,
                target: Some(31),
                marke: None,
            },
        )
        .expect("painter");
    assert!(matches!(
        events[0],
        RunEvent::PainterHired {
            card: 31,
            price: 0,
            ..
        }
    ));
    assert_eq!(run.phase(), RunPhase::Encounter, "waits for the choice");
    assert_eq!(
        run.apply(
            &content,
            RunAction::ChooseEncounter {
                option: 1,
                target: None,
                marke: None
            }
        ),
        Err(RunError::PainterPending)
    );
    let events = run
        .apply(&content, RunAction::ChoosePaint(0))
        .expect("paint");
    assert!(events.contains(&RunEvent::EncounterResolved {
        encounter: EncounterId::new("workshop"),
        option: 0,
    }));
    assert!(run.encounter().is_none() && run.painter().is_none());
    assert_eq!(run.phase(), RunPhase::Game, "on to the next pub");
    assert_eq!(run.edition().get(31).map(|c| c.mods.len()), Some(1));
}

#[test]
fn saves_keep_the_auslage_and_old_saves_lose_the_removed_items() {
    let content = content();
    let (mut run, _) = tresen(&content, 12, 20);
    run.apply(&content, RunAction::HirePainter(31))
        .expect("hire");
    let json = serde_json::to_string(&run).expect("save");
    let loaded: Run = serde_json::from_str(&json).expect("load");
    assert_eq!(loaded, run, "painter and Auslage survive a save");
    // A save from before: no Auslage, no Zinken generator, removed items.
    let mut value: serde_json::Value = serde_json::to_value(&run).expect("value");
    let object = value.as_object_mut().expect("object");
    object.remove("zinken_rng");
    object.remove("painter");
    let shop = object["shop"].as_object_mut().expect("shop");
    shop.remove("card_offers");
    shop.remove("painter_used");
    shop["consumables"] = serde_json::json!(["ink_pad", "herbal"]);
    object["consumables"] = serde_json::json!(["fresh_deck", "points_3"]);
    let mut old: Run = serde_json::from_value(value).expect("old save");
    assert!(old.shop().expect("shop").card_offers.is_empty());
    let dropped = old.drop_unknown_content(&content);
    assert_eq!(dropped, vec!["fresh_deck", "points_3"]);
    assert!(old.consumables().is_empty());
    assert_eq!(
        old.shop().expect("shop").consumables,
        vec![None, Some(ConsumableId::new("herbal"))]
    );
    // The Auslage of a save from before is empty; "Neu auslosen" fills it.
    old.apply(&content, RunAction::RerollCardOffers)
        .expect("reroll");
    assert_eq!(offers(&old).len(), 3);
    // Unknown modifications fall out of the Auslage and the painter.
    let mut run = loaded;
    set_offers(&mut run, vec![offer(1, "glass_ace", 1, 4)]);
    run.painter = Some(PainterChoice {
        card: 31,
        options: vec![(CardModId::new("glass_ace"), 1)],
        encounter: false,
    });
    run.drop_unknown_content(&content);
    assert_eq!(run.shop().expect("shop").card_offers, vec![None]);
    assert!(run.painter().is_none());
}

#[test]
fn the_bot_plays_whole_runs_with_the_zinken() {
    let content = content();
    let mut bought = 0;
    let mut painted = 0;
    for seed in 0..6 {
        let (mut run, _) = Run::new(&content, seed).expect("start");
        let mut steps = 0;
        while let Some(action) = super::bot::RunBot.next_action(&run, &content) {
            let events = run.apply(&content, action.clone()).expect("bot action");
            bought += events
                .iter()
                .filter(|e| matches!(e, RunEvent::CardOfferBought { .. }))
                .count();
            painted += events
                .iter()
                .filter(|e| matches!(e, RunEvent::PainterChosen { .. }))
                .count();
            steps += 1;
            assert!(steps < 5000, "seed {seed}: stuck at {action:?}");
        }
    }
    assert!(bought > 0, "the bot buys from the Auslage");
    let _ = painted;
}

#[test]
fn bruderherz_pays_half_on_rot_and_keeps_his_curses() {
    let content = content();
    let start = RunStart {
        character: Character::HeartBrother,
        ..RunStart::default()
    };
    let (mut run, _) = Run::new_with(&content, 3, &start).expect("start");
    run.series.games_played = 1;
    run.series.score = i64::try_from(run.series.target).expect("small");
    run.game = None;
    let mut events = Vec::new();
    run.after_game(&content, &mut events).expect("after game");
    run.pass_card_reward(&content, &mut events);
    run.coins = 40;
    run.walk_to_tresen(&content);
    // Rot-Zehn (11) and Grün-Zehn (19), both bare for the test.
    for card in [11, 19] {
        let _ = run.edition.cleanse(card);
    }
    set_offers(
        &mut run,
        vec![offer(11, "points", 3, 4), offer(19, "points", 3, 4)],
    );
    let view = run.view(&content).shop.expect("shop");
    let prices: Vec<u32> = view.card_offers.iter().flatten().map(|o| o.price).collect();
    assert_eq!(prices, vec![2, 4], "half on Rot");
    run.apply(&content, RunAction::BuyCardOffer(0))
        .expect("buy");
    assert_eq!(run.coins(), 38);
    // The Kartenmaler: the Wirt first, then half on Rot (6 * 0.8 = 5, half 3).
    run.relics.push(RelicId::new("landlord"));
    let painter = &run.view(&content).shop.expect("shop").painter;
    let at = |card: u32| {
        painter
            .targets
            .iter()
            .position(|&c| c == card)
            .map(|i| painter.target_prices[i])
    };
    assert_eq!(at(19), Some(5));
    assert_eq!(at(10), Some(3));
    run.apply(&content, RunAction::HirePainter(10))
        .expect("hire");
    assert_eq!(run.coins(), 35);
    run.apply(&content, RunAction::ChoosePaint(0))
        .expect("paint");
    // His curses stay: Karte reinigen goes through Run::cleanse.
    let cursed = run
        .edition()
        .cards()
        .iter()
        .find(|c| {
            c.mods
                .iter()
                .any(|m| content.card_mod(m).is_some_and(|d| d.is_curse()))
        })
        .map(|c| c.id)
        .expect("a cursed card in his edition");
    assert_eq!(
        run.apply(&content, RunAction::CleanseCard(cursed)),
        Err(RunError::CurseStays)
    );
    assert_eq!(run.coins(), 35, "nothing paid");
}

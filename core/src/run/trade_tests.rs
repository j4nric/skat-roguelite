//! Curses as trade (DESIGN 8.6, decided by the user on 03.11.2026):
//! "Pechvogel", "Fusel", "Schnäppchen" and "Hehlerware", old saves.

use super::content_tests::{content, fixed_deal, play, relic_run};
use super::trade::CurseCause;
use super::zinken_tests::{offers, tresen};
use super::*;
use crate::cards::Suit;
use crate::content::{Rarity, TradeConfig};
use crate::rules::{GameType, RiskLevel};

/// Declarer holds all trumps of a diamonds game but the D8 (in the skat).
const STRONG: [&str; 3] = [
    "CJ SJ HJ DJ DA DT DK DQ D9 D7",
    "HA HT HK HQ H9 H8 H7 SA ST SK",
    "SQ S9 S8 S7 CA CT CK CQ C9 C8",
];
const STRONG_SKAT: &str = "D8 C7";

fn diamonds_hand() -> Declaration {
    Declaration::new(GameType::Suit(Suit::Diamonds), RiskLevel::Hand).expect("suit")
}

/// Instance id of the printed card `notation` in the player's edition.
fn card_id(run: &Run, notation: &str) -> u32 {
    let kind: CardKind = notation.parse().expect("notation");
    run.edition()
        .cards()
        .iter()
        .find(|c| c.kind == kind)
        .expect("in the edition")
        .id
}

fn curse(run: &mut Run, content: &Content, notation: &str, curse: &str) {
    let id = card_id(run, notation);
    run.edition
        .add_mod(content, id, &CardModId::new(curse), 1)
        .expect("curse");
}

/// Content with every offer a trade.
fn always_trade() -> Content {
    let mut content = content();
    content.run.shop.trade = TradeConfig {
        bargain_percent: 100,
        fence_percent: 100,
    };
    content
}

fn trade_curses(events: &[RunEvent]) -> Vec<(u32, CardModId, CurseCause)> {
    events
        .iter()
        .filter_map(|e| match e {
            RunEvent::TradeCurse { card, curse, cause } => Some((*card, curse.clone(), *cause)),
            _ => None,
        })
        .collect()
}

#[test]
fn the_new_items_load() {
    let content = content();
    let jinx = content.relic(&RelicId::new("jinx")).expect("Pechvogel");
    assert_eq!(jinx.name, "Pechvogel");
    assert_eq!(jinx.rarity, Rarity::Rare);
    assert_eq!(jinx.group, crate::content::RelicGroup::Value);
    let rotgut = content
        .consumable(&ConsumableId::new("rotgut"))
        .expect("Fusel");
    assert_eq!(rotgut.name, "Fusel");
    assert_eq!(
        rotgut.action,
        ConsumableAction::NextGameCursed(NextGameEffect::Multiply(2))
    );
    assert_eq!(
        rotgut.category,
        crate::content::ConsumableCategory::Schnapps
    );
    let trade = &content.run.shop.trade;
    assert!(trade.bargain_percent > 0 && trade.fence_percent > 0);
    assert_eq!(
        [0, 1, 2, 3, 7, 9].map(TradeConfig::half),
        [0, 1, 1, 1, 3, 4]
    );
}

#[test]
fn jinx_adds_one_grundwert_per_cursed_card() {
    let content = content();
    // Without a curse: no line.
    let mut run = relic_run(&content, &["jinx"]);
    assert_eq!(run.cursed_cards(&content), 0);
    fixed_deal(&mut run, &content, STRONG, STRONG_SKAT);
    let (_, plain) = play(&mut run, &content, diamonds_hand());
    assert!(
        !plain
            .lines
            .iter()
            .any(|l| l.source == Source::Relic(RelicId::new("jinx")))
    );
    // Three cursed cards, in hand or not: +3.
    let mut run = relic_run(&content, &["jinx"]);
    curse(&mut run, &content, "C7", "transparent");
    curse(&mut run, &content, "HA", "transparent");
    curse(&mut run, &content, "S7", "transparent");
    assert_eq!(run.cursed_cards(&content), 3);
    fixed_deal(&mut run, &content, STRONG, STRONG_SKAT);
    let (_, cursed) = play(&mut run, &content, diamonds_hand());
    assert!(cursed.lines.contains(&ScoreLine {
        source: Source::Relic(RelicId::new("jinx")),
        lever: Lever::BaseValue,
        change: Change::Add(3),
    }));
    assert_eq!(cursed.base_value, plain.base_value + 3);
    // The preview of the game value knows it too.
    let mut run = relic_run(&content, &["jinx"]);
    curse(&mut run, &content, "C7", "transparent");
    fixed_deal(&mut run, &content, STRONG, STRONG_SKAT);
    let events = run
        .apply(&content, RunAction::Declare(diamonds_hand()))
        .expect("declare");
    assert!(!events.is_empty());
    let preview = run
        .view(&content)
        .game
        .expect("game")
        .value_preview
        .expect("preview");
    assert_eq!(preview.base_value, plain.base_value + 1);
}

#[test]
fn jinx_counts_every_curse_of_the_start_editions() {
    let content = content();
    for (character, cursed) in [
        (Character::Declarer, 0),
        // Abgestuft on the four Unter counts (a special curse).
        (Character::NullPlayer, 4),
        // Bruderherz: six Fleckig, three Ramponiert.
        (Character::HeartBrother, 9),
    ] {
        let start = RunStart {
            character,
            ..RunStart::default()
        };
        let (run, _) = Run::new_with(&content, 3, &start).expect("run");
        assert_eq!(run.cursed_cards(&content), cursed, "{character:?}");
    }
}

#[test]
fn rotgut_doubles_the_next_game_and_curses_a_card_for_good() {
    let content = content();
    let (mut plain, _) = Run::new(&content, 11).expect("start");
    fixed_deal(&mut plain, &content, STRONG, STRONG_SKAT);
    let (_, base) = play(&mut plain, &content, diamonds_hand());

    let (mut run, _) = Run::new(&content, 11).expect("start");
    run.consumables = vec![ConsumableId::new("rotgut")];
    fixed_deal(&mut run, &content, STRONG, STRONG_SKAT);
    let hand_before = run.game().expect("game").game.hand(Seat::DECLARER).to_vec();
    let coins = run.coins();
    let events = run
        .apply(
            &content,
            RunAction::UseConsumable {
                slot: 0,
                target: None,
                suit: None,
            },
        )
        .expect("drink");
    // No new deal, unlike the Doppelkorn.
    assert!(!events.contains(&RunEvent::Redealt));
    assert_eq!(
        run.game().expect("game").game.hand(Seat::DECLARER),
        hand_before
    );
    let curses = trade_curses(&events);
    assert_eq!(curses.len(), 1);
    let (card, curse, cause) = curses[0].clone();
    assert_eq!(cause, CurseCause::Schnapps);
    assert!(content.card_mod(&curse).is_some_and(|d| d.is_curse()));
    assert!(!content.card_mod(&curse).is_some_and(|d| d.is_special()));
    assert!(run.card_is_cursed(&content, card));
    assert_eq!(run.cursed_cards(&content), 1);
    // The curse is the price: it pays nothing (03.11.2026).
    assert_eq!(run.coins(), coins);
    assert!(
        !events
            .iter()
            .any(|e| matches!(e, RunEvent::CoinsGained { .. }))
    );
    assert_eq!(
        run.pending_next_game(),
        &[(ConsumableId::new("rotgut"), NextGameEffect::Multiply(2))]
    );
    let (_, doubled) = play(&mut run, &content, diamonds_hand());
    assert!(doubled.won);
    assert!(doubled.lines.contains(&ScoreLine {
        source: Source::Consumable(ConsumableId::new("rotgut")),
        lever: Lever::GameValue,
        change: Change::Multiply(2),
    }));
    // The curse stays after the game.
    assert!(run.card_is_cursed(&content, card));
    assert!(base.value > 0);
}

#[test]
fn rotgut_only_before_skat_oder_hand_and_only_with_a_card_to_curse() {
    let content = content();
    let drink = RunAction::UseConsumable {
        slot: 0,
        target: None,
        suit: None,
    };
    // After picking up the skat: too late.
    let (mut run, _) = Run::new(&content, 11).expect("start");
    run.consumables = vec![ConsumableId::new("rotgut")];
    run.apply(&content, RunAction::PickUpSkat).expect("pick up");
    assert_eq!(
        run.apply(&content, drink.clone()),
        Err(RunError::WrongPhase)
    );
    // At the Tresen: not there.
    let (mut run, _) = tresen(&content, 5, 10);
    run.consumables = vec![ConsumableId::new("rotgut")];
    assert_eq!(
        run.apply(&content, drink.clone()),
        Err(RunError::WrongPhase)
    );
    // Every card cursed: nothing to pay with.
    let (mut run, _) = Run::new(&content, 11).expect("start");
    run.consumables = vec![ConsumableId::new("rotgut")];
    let ids: Vec<u32> = run.edition().cards().iter().map(|c| c.id).collect();
    for id in ids {
        run.edition
            .add_mod(&content, id, &CardModId::new("transparent"), 1)
            .expect("curse");
    }
    assert!(run.trade_curse_targets(&content, &[]).is_empty());
    assert_eq!(run.apply(&content, drink), Err(RunError::NoModAvailable));
    assert_eq!(run.consumables().len(), 1, "not drunk");
}

#[test]
fn a_bargain_costs_half_and_curses_another_card() {
    let content = always_trade();
    let (mut run, _) = tresen(&content, 7, 30);
    let all = offers(&run);
    let n = usize::try_from(content.run.shop.zinken.offers).expect("small");
    assert_eq!(all.len(), n);
    for o in &all {
        let def = content.card_mod(&o.card_mod).expect("mod");
        if def.is_special() {
            assert_eq!(o.bargain, None, "a special card is never a bargain");
            continue;
        }
        let full = o.bargain.expect("every Zinke a bargain");
        assert_eq!(o.price, TradeConfig::half(full));
    }
    let (slot, offer) = all
        .iter()
        .enumerate()
        .find(|(_, o)| o.bargain.is_some())
        .expect("a bargain");
    let view = run.view(&content).shop.expect("shop");
    let tile = view.card_offers[slot].as_ref().expect("tile");
    assert!(tile.bargain);
    assert_eq!(tile.price, run.card_offer_cost(&content, offer));
    assert!(tile.was.expect("full price") > tile.price);
    let coins = run.coins();
    let events = run
        .apply(&content, RunAction::BuyCardOffer(slot))
        .expect("buy");
    assert_eq!(
        run.coins() + tile.price,
        coins,
        "half price, no Groschen back"
    );
    let curses = trade_curses(&events);
    assert_eq!(curses.len(), 1);
    let (card, _, cause) = curses[0].clone();
    assert_eq!(cause, CurseCause::Bargain);
    assert_ne!(card, offer.card, "on another card");
    let others: Vec<u32> = all.iter().map(|o| o.card).collect();
    assert!(!others.contains(&card), "on no card of the Auslage");
    assert!(
        run.edition()
            .get(offer.card)
            .is_some_and(|c| c.mods.contains(&offer.card_mod)),
        "the Zinke landed"
    );
    assert!(run.card_is_cursed(&content, card));
}

#[test]
fn hehlerware_sells_a_relic_at_half_price_for_a_curse() {
    let content = always_trade();
    let (mut run, events) = tresen(&content, 7, 30);
    let fence = run.shop().expect("shop").fence.expect("Hehlerware");
    assert!(events.iter().any(|e| matches!(
        e,
        RunEvent::ShopOpened { fence: Some(f), .. } if *f == fence
    )));
    let relic = run.shop().expect("shop").relics[fence]
        .clone()
        .expect("relic");
    let full = content.relic(&relic).expect("def").price;
    let view = run.view(&content).shop.expect("shop");
    let tile = view.relics[fence].as_ref().expect("tile");
    assert!(tile.fence);
    assert_eq!(tile.price, TradeConfig::half(full));
    assert_eq!(tile.was, Some(full));
    for (slot, other) in view.relics.iter().enumerate() {
        if slot != fence {
            let other = other.as_ref().expect("offer");
            assert!(!other.fence && other.was.is_none());
            assert_eq!(other.price, other.relic.price);
        }
    }
    let coins = run.coins();
    let events = run
        .apply(&content, RunAction::BuyRelic(fence))
        .expect("buy");
    assert!(run.relics().contains(&relic));
    let curses = trade_curses(&events);
    assert_eq!(curses.len(), 1);
    assert_eq!(curses[0].2, CurseCause::Fence);
    assert!(run.card_is_cursed(&content, curses[0].0));
    let paid = events
        .iter()
        .find_map(|e| match e {
            RunEvent::RelicBought { price, .. } => Some(*price),
            _ => None,
        })
        .expect("bought");
    assert_eq!(paid, TradeConfig::half(full));
    assert_eq!(run.coins() + paid, coins, "no Groschen for the curse");
    // A normal relic of the same Tresen curses nothing.
    let other = (0..3).find(|&s| s != fence).expect("other slot");
    let events = run
        .apply(&content, RunAction::BuyRelic(other))
        .expect("buy");
    assert!(trade_curses(&events).is_empty());
}

#[test]
fn no_trade_without_a_card_to_curse_and_none_by_default_off() {
    let content = always_trade();
    let (mut run, _) = tresen(&content, 7, 30);
    let ids: Vec<u32> = run.edition().cards().iter().map(|c| c.id).collect();
    for id in ids {
        run.edition
            .add_mod(&content, id, &CardModId::new("transparent"), 1)
            .expect("curse");
    }
    // Every card cursed: the Auslage has nothing to offer, the Tresen no
    // Hehlerware.
    let shop = run.draw_offer(&content, 0);
    assert_eq!(shop.fence, None);
    // Content without trade numbers offers none.
    let mut off = content.clone();
    off.run.shop.trade = TradeConfig::default();
    for seed in 0..20 {
        let (run, _) = tresen(&off, seed, 10);
        assert_eq!(run.shop().expect("shop").fence, None);
        assert!(offers(&run).iter().all(|o| o.bargain.is_none()));
    }
}

#[test]
fn the_trades_come_now_and_then_with_the_real_numbers() {
    let content = content();
    let (mut fences, mut bargains, mut offers_seen) = (0, 0, 0);
    for seed in 0..60 {
        let (run, _) = tresen(&content, seed, 10);
        fences += usize::from(run.shop().expect("shop").fence.is_some());
        let all = offers(&run);
        offers_seen += all.len();
        bargains += all.iter().filter(|o| o.bargain.is_some()).count();
    }
    // About one Tresen in four, one offer in five.
    assert!((5..=30).contains(&fences), "{fences}");
    assert!(bargains * 10 > offers_seen && bargains * 3 < offers_seen);
}

#[test]
fn trades_leave_the_other_draws_alone() {
    // The trade has its own generator: with or without trades, a Tresen
    // offers the same relics, Schnäpse and Zinken.
    let on = always_trade();
    let mut off = on.clone();
    off.run.shop.trade = TradeConfig::default();
    for seed in 0..10 {
        let (a, _) = tresen(&on, seed, 10);
        let (b, _) = tresen(&off, seed, 10);
        let (sa, sb) = (a.shop().expect("shop"), b.shop().expect("shop"));
        assert_eq!(sa.relics, sb.relics, "seed {seed}");
        assert_eq!(sa.consumables, sb.consumables, "seed {seed}");
        let cards = |r: &Run| -> Vec<(u32, CardModId)> {
            offers(r)
                .into_iter()
                .map(|o| (o.card, o.card_mod))
                .collect()
        };
        assert_eq!(cards(&a), cards(&b), "seed {seed}");
    }
}

#[test]
fn old_saves_load_without_the_trade() {
    let content = always_trade();
    let (run, _) = tresen(&content, 7, 30);
    let json = serde_json::to_string(&run).expect("save");
    let loaded: Run = serde_json::from_str(&json).expect("load");
    assert_eq!(loaded, run, "Hehlerware and Schnäppchen survive a save");
    let mut value: serde_json::Value = serde_json::to_value(&run).expect("value");
    let object = value.as_object_mut().expect("object");
    assert!(object.remove("trade_rng").is_some());
    let shop = object["shop"].as_object_mut().expect("shop");
    assert!(shop.remove("fence").is_some());
    for offer in shop["card_offers"].as_array_mut().expect("offers") {
        if let Some(offer) = offer.as_object_mut() {
            offer.remove("bargain");
        }
    }
    let mut old: Run = serde_json::from_value(value).expect("old save");
    assert_eq!(old.shop().expect("shop").fence, None);
    assert!(offers(&old).iter().all(|o| o.bargain.is_none()));
    // It plays on: a normal relic, a new Auslage with its own trades.
    old.apply(&content, RunAction::BuyRelic(0)).expect("buy");
    old.apply(&content, RunAction::Reroll).expect("reroll");
    let n = usize::try_from(content.run.shop.zinken.offers).expect("small");
    assert_eq!(offers(&old).len(), n);
}

#[test]
fn the_bot_plays_whole_runs_with_trades() {
    let content = always_trade();
    let mut traded = 0;
    for seed in 0..4 {
        let start = RunStart {
            character: if seed % 2 == 0 {
                Character::Declarer
            } else {
                Character::HeartBrother
            },
            relics: vec![RelicId::new("jinx")],
            ..RunStart::default()
        };
        let (mut run, _) = Run::new_with(&content, seed, &start).expect("start");
        run.consumables = vec![ConsumableId::new("rotgut")];
        let mut steps = 0;
        while let Some(action) = super::bot::RunBot.next_action(&run, &content) {
            let events = run.apply(&content, action).expect("bot action");
            traded += trade_curses(&events).len();
            steps += 1;
            assert!(steps < 50_000, "seed {seed} runs on");
        }
        assert!(matches!(run.phase(), RunPhase::Won | RunPhase::Lost));
    }
    assert!(traded > 0, "the bot takes trades with Pechvogel");
}

#[test]
fn trade_curses_pay_no_groschen_other_curses_do() {
    let content = content();
    let (mut run, _) = Run::new(&content, 11).expect("start");
    run.coins = 5;
    // A curse from elsewhere (an encounter) pays its 3 Groschen.
    let mut events = Vec::new();
    let id = card_id(&run, "C7");
    run.attach_mod(&content, id, &CardModId::new("battered"), 5, &mut events)
        .expect("curse");
    assert_eq!(run.coins(), 8);
    // As the price of a trade it pays nothing (decided on 03.11.2026).
    for cause in [CurseCause::Bargain, CurseCause::Fence, CurseCause::Schnapps] {
        let mut events = Vec::new();
        let (card, _) = run
            .trade_curse(&content, &[], cause, &mut events)
            .expect("trade")
            .expect("a card");
        assert!(run.card_is_cursed(&content, card));
        assert_eq!(run.coins(), 8, "{cause:?}");
        assert!(
            !events
                .iter()
                .any(|e| matches!(e, RunEvent::CoinsGained { .. }))
        );
    }
    // The curses say so.
    for id in ["transparent", "battered", "stained"] {
        let def = content.card_mod(&CardModId::new(id)).expect(id);
        assert!(
            def.text
                .ends_with("Beim Erhalt 3 Groschen (nicht bei einem Handel)."),
            "{id}"
        );
    }
}

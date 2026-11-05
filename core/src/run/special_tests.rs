//! Special cards in the run (DESIGN 7.2, decided by the user on
//! 03.11.2026; since 05.11.2026 without a cap per edition): Marken always
//! turn their card, offers propose special cards the edition has already,
//! the rare special offer of the Auslage, the hint flag of the meta state
//! and saves.

use super::content_tests::{card_mod_run_with, content, fixed_deal};
use super::meta::MetaState;
use super::zinken::{CardOffer, special_target};
use super::zinken_tests::{at_route, offers, set_offers, tresen};
use super::*;
use crate::content::ModKind;
use crate::deck::DeckChange;

/// The declarer's hand of a diamonds game; D9 (id 2) and C7 (id 24) go
/// into the skat. Ids: D9 = 2, C7 = 24, DA = 7, HA = 15.
const STRONG: [&str; 3] = [
    "CJ SJ HJ DJ DA DT DK DQ D9 D7",
    "HA HT HK HQ H9 H8 H7 SA ST SK",
    "SQ S9 S8 S7 CA CT CK CQ C9 C8",
];
const STRONG_SKAT: &str = "D8 C7";

fn id(m: &str) -> CardModId {
    CardModId::new(m)
}

fn mods(run: &Run, card: u32) -> Vec<CardModId> {
    run.edition().get(card).expect("card").mods.clone()
}

/// `run` with the D9 discarded after picking up the skat.
fn discard_d9(run: &mut Run, content: &Content) -> Vec<RunEvent> {
    fixed_deal(run, content, STRONG, STRONG_SKAT);
    run.apply(content, RunAction::PickUpSkat).expect("pick up");
    run.apply(content, RunAction::Discard([2, 24]))
        .expect("discard")
}

// ---------------------------------------------------------------------------
// Vouchers
// ---------------------------------------------------------------------------

#[test]
fn a_totenmarke_turns_its_card_when_another_card_is_untot() {
    let content = content();
    let mut run = card_mod_run_with(&content, 2, "voucher_undead", 1);
    run.edition
        .add_mod(&content, 15, &id("undead"), 1)
        .expect("Untot on HA");
    let coins = run.coins();
    let events = discard_d9(&mut run, &content);
    assert!(events.iter().any(|e| matches!(
        e,
        RunEvent::DeckChanged(DeckChange::Modified { card: 2, card_mod, .. })
            if card_mod == &id("undead")
    )));
    assert_eq!(run.coins(), coins, "nothing paid instead");
    assert_eq!(mods(&run, 2), vec![id("undead")]);
    assert_eq!(mods(&run, 15), vec![id("undead")]);
}

#[test]
fn a_narrenmarke_turns_its_card_when_another_card_is_wandelkarte() {
    let content = content();
    let mut run = card_mod_run_with(&content, 2, "voucher_wild", 1);
    run.edition
        .add_mod(&content, 15, &id("wild_card"), 1)
        .expect("Wandelkarte on HA");
    let coins = run.coins();
    discard_d9(&mut run, &content);
    assert_eq!(run.coins(), coins);
    assert_eq!(mods(&run, 2), vec![id("wild_card")]);
}

#[test]
fn a_totenmarke_turns_its_card_when_untot_is_free() {
    let content = content();
    let mut run = card_mod_run_with(&content, 2, "voucher_undead", 1);
    // The other special card does not block it.
    run.edition
        .add_mod(&content, 15, &id("wild_card"), 1)
        .expect("Wandelkarte on HA");
    let coins = run.coins();
    discard_d9(&mut run, &content);
    assert_eq!(mods(&run, 2), vec![id("undead")]);
    assert_eq!(run.coins(), coins);
}

#[test]
fn a_totenmarke_on_its_own_untot_card_does_nothing() {
    // The Drückeberger's returned Totenmarke (DESIGN 9.4): the card is
    // Untot already, nothing is paid.
    let content = content();
    let mut run = card_mod_run_with(&content, 2, "undead", 1);
    run.edition
        .add_mod(&content, 2, &id("voucher_undead"), 1)
        .expect("voucher next to Untot");
    let coins = run.coins();
    discard_d9(&mut run, &content);
    assert_eq!(mods(&run, 2), vec![id("undead")]);
    assert_eq!(run.coins(), coins);
}

// ---------------------------------------------------------------------------
// No cap on any path (since 05.11.2026)
// ---------------------------------------------------------------------------

#[test]
fn the_special_target_of_vouchers_and_special_cards() {
    let content = content();
    let target = |m: &str| special_target(content.card_mod(&id(m)).expect("mod")).cloned();
    assert_eq!(target("voucher_undead"), Some(id("undead")));
    assert_eq!(target("voucher_wild"), Some(id("wild_card")));
    assert_eq!(target("undead"), Some(id("undead")));
    assert_eq!(target("wild_card"), Some(id("wild_card")));
    for m in ["demoted", "points", "voucher_coins", "battered"] {
        assert_eq!(target(m), None, "{m}");
    }
}

#[test]
fn the_pools_keep_the_marken_next_to_a_special_card() {
    let content = content();
    let mut run = at_route(&content, 3, 0);
    let has = |run: &Run, m: &str| run.zinken_pool(&content).iter().any(|(p, _)| p == &id(m));
    run.edition
        .add_mod(&content, 9, &id("undead"), 1)
        .expect("Untot");
    run.edition
        .add_mod(&content, 10, &id("voucher_wild"), 1)
        .expect("Narrenmarke");
    assert!(has(&run, "voucher_undead") && has(&run, "voucher_wild"));
    // The random draws of the encounters (Kartengeber) as well.
    let pool = run.mod_pool(&content, false);
    assert!(pool.iter().any(|(m, _)| m.as_str() == "voucher_undead"));
}

/// Open an encounter on a walk (as `items_tests::open` does).
fn open(run: &mut Run, content: &Content, encounter: &str) -> bool {
    run.route = None;
    run.trip = Some(tour::Trip {
        section: 0,
        index: 1,
        transport: tour::Transport::Walk,
        stops: Vec::new(),
        spelunke: None,
    });
    let mut events = Vec::new();
    run.open_encounter(content, &EncounterId::new(encounter), &mut events)
}

#[test]
fn old_cards_offer_their_vouchers_next_to_special_cards() {
    let content = content();
    let mut run = at_route(&content, 1, 0);
    run.edition
        .add_mod(&content, 9, &id("undead"), 1)
        .expect("Untot");
    run.edition
        .add_mod(&content, 10, &id("voucher_wild"), 1)
        .expect("Narrenmarke");
    assert!(open(&mut run, &content, "old_cards"));
    for i in 0..3 {
        assert_eq!(run.encounter_option_block(&content, i), None, "option {i}");
    }
}

#[test]
fn the_conjurer_comes_while_another_card_is_wandelkarte() {
    let content = content();
    let mut run = at_route(&content, 1, 0);
    run.edition
        .add_mod(&content, 9, &id("wild_card"), 1)
        .expect("Wandelkarte");
    assert!(open(&mut run, &content, "conjurer"));
    assert_eq!(run.encounter_option_block(&content, 0), None);
}

// ---------------------------------------------------------------------------
// The Auslage
// ---------------------------------------------------------------------------

fn is_special(content: &Content, offer: &CardOffer) -> bool {
    content
        .card_mod(&offer.card_mod)
        .is_some_and(|d| d.kind == ModKind::Special)
}

#[test]
fn the_auslage_rarely_offers_a_special_card_right_on_a_card() {
    let content = content();
    let zinken = &content.run.shop.zinken;
    let (mut total, mut special) = (0_u32, 0_u32);
    for seed in 0..400 {
        let (run, _) = tresen(&content, seed, 10);
        let offers = offers(&run);
        for o in &offers {
            total += 1;
            if !is_special(&content, o) {
                continue;
            }
            special += 1;
            let card = run.edition().get(o.card).expect("card");
            let face = card.kind.face().expect("face");
            let premium = if zinken.premium_ranks.contains(&face.rank) {
                zinken.premium
            } else {
                0
            };
            assert_eq!(o.price, zinken.special_price + premium, "seed {seed}");
            assert_eq!(o.value, 1);
            assert!(
                !card
                    .mods
                    .iter()
                    .any(|m| content.card_mod(m).is_some_and(|d| d.is_special())),
                "never on a card with a special card"
            );
        }
    }
    // About one offer in twenty (5 %).
    let permille = special * 1000 / total;
    assert!((20..=90).contains(&permille), "{special} of {total}");
}

#[test]
fn the_auslage_offers_special_cards_the_edition_has() {
    let content = content();
    let mut seen = 0;
    for seed in 0..300 {
        let mut run = at_route(&content, seed, 10);
        run.edition
            .add_mod(&content, 9, &id("undead"), 1)
            .expect("Untot");
        run.edition
            .add_mod(&content, 12, &id("voucher_wild"), 1)
            .expect("Narrenmarke");
        run.walk_to_tresen(&content);
        seen += offers(&run)
            .iter()
            .filter(|o| {
                let def = content.card_mod(&o.card_mod).expect("mod");
                special_target(def).is_some()
            })
            .count();
    }
    assert!(seen > 0, "Untot and Narrenmarke still come");
}

#[test]
fn a_special_offer_keeps_the_zinke_of_the_card() {
    let content = content();
    let (mut run, _) = tresen(&content, 4, 20);
    run.edition
        .add_mod(&content, 31, &id("points"), 3)
        .expect("Aufdruck");
    set_offers(
        &mut run,
        vec![CardOffer {
            card: 31,
            card_mod: id("undead"),
            value: 1,
            price: 7,
            bargain: None,
        }],
    );
    let view = run.view(&content).shop.expect("shop");
    let tile = view.card_offers[0].as_ref().expect("tile");
    assert_eq!(tile.replaces, None);
    assert_eq!(tile.result.mods, vec![id("points"), id("undead")]);
    let events = run
        .apply(&content, RunAction::BuyCardOffer(0))
        .expect("buy");
    assert!(events.iter().any(|e| matches!(
        e,
        RunEvent::DeckChanged(DeckChange::Modified { card: 31, card_mod, replaced: None, .. })
            if card_mod == &id("undead")
    )));
    assert_eq!(run.coins(), 13);
    assert_eq!(mods(&run, 31), vec![id("points"), id("undead")]);
}

#[test]
fn a_special_offer_goes_through_the_tresen_prices() {
    let content = content();
    let (mut run, _) = tresen(&content, 4, 20);
    let offer = CardOffer {
        card: 31,
        card_mod: id("undead"),
        value: 1,
        price: 7,
        bargain: None,
    };
    let plain = run.card_offer_cost(&content, &offer);
    assert_eq!(plain, 7);
    // "Der Wirt": 20 % off at the Tresen.
    run.relics.push(RelicId::new("landlord"));
    assert!(run.card_offer_cost(&content, &offer) < plain);
}

#[test]
fn a_second_untot_can_be_bought() {
    let content = content();
    let (mut run, _) = tresen(&content, 4, 20);
    run.edition
        .add_mod(&content, 9, &id("undead"), 1)
        .expect("Untot");
    set_offers(
        &mut run,
        vec![CardOffer {
            card: 31,
            card_mod: id("undead"),
            value: 1,
            price: 7,
            bargain: None,
        }],
    );
    let view = run.view(&content).shop.expect("shop");
    assert!(view.card_offers[0].as_ref().expect("tile").can_buy);
    run.apply(&content, RunAction::BuyCardOffer(0))
        .expect("buy");
    assert_eq!(mods(&run, 9), vec![id("undead")]);
    assert_eq!(mods(&run, 31), vec![id("undead")]);
}

// ---------------------------------------------------------------------------
// The hint flag of the meta state
// ---------------------------------------------------------------------------

#[test]
fn the_meta_state_remembers_the_first_special_card() {
    let content = content();
    let mut meta = MetaState::new();
    let got = |card_mod: &str| {
        RunEvent::<crate::game::Event<DealtCard>>::DeckChanged(DeckChange::Modified {
            card: 3,
            card_mod: id(card_mod),
            value: 1,
            replaced: None,
        })
    };
    let before = meta.clone();
    meta.observe(&content, &[got("points"), got("voucher_undead")]);
    assert!(meta.new_specials(&before).is_empty(), "no special card yet");
    // "Abgestuft" is a special curse, no special card to explain.
    meta.observe(&content, &[got("demoted")]);
    assert!(meta.specials_seen.is_empty());
    let before = meta.clone();
    meta.observe(&content, &[got("undead")]);
    assert_eq!(meta.new_specials(&before), vec![id("undead")]);
    // The second time is no news.
    let before = meta.clone();
    meta.observe(&content, &[got("undead"), got("wild_card")]);
    assert_eq!(meta.new_specials(&before), vec![id("wild_card")]);
    // Part of the save.
    let json = serde_json::to_string(&meta).expect("save");
    assert!(json.contains("specials_seen"));
    let loaded: MetaState = serde_json::from_str(&json).expect("load");
    assert_eq!(loaded, meta);
}

#[test]
fn a_meta_save_from_before_has_seen_no_special_card() {
    let fresh = serde_json::to_string(&MetaState::new()).expect("save");
    assert!(!fresh.contains("specials_seen"), "left out while empty");
    let old: MetaState = serde_json::from_str(r#"{"version":1,"unlocked":[]}"#).expect("old");
    assert!(old.specials_seen.is_empty());
}

#[test]
fn a_run_shows_the_hint_once_through_its_events() {
    // The whole way: a voucher redeemed in a real game feeds the meta
    // state, which explains the Untot once.
    let content = content();
    let mut run = card_mod_run_with(&content, 2, "voucher_undead", 1);
    let mut meta = MetaState::new();
    let before = meta.clone();
    let events = discard_d9(&mut run, &content);
    meta.observe(&content, &events);
    assert_eq!(meta.new_specials(&before), vec![id("undead")]);
}

// ---------------------------------------------------------------------------
// Saves
// ---------------------------------------------------------------------------

#[test]
fn a_save_with_two_untot_keeps_both() {
    let content = content();
    let mut run = at_route(&content, 5, 10);
    run.edition
        .add_mod(&content, 9, &id("undead"), 1)
        .expect("Untot");
    // Two Untot (from before the cap of 03.11.2026, or since 05.11.2026).
    let mut value = serde_json::to_value(&run).expect("value");
    let cards = value["edition"]["cards"]
        .as_array_mut()
        .expect("cards of the edition");
    let card = cards.iter_mut().find(|c| c["id"] == 10).expect("card 10");
    card["mods"] = serde_json::json!(["undead"]);
    let mut old: Run = serde_json::from_value(value).expect("old save");
    old.drop_unknown_content(&content);
    assert_eq!(mods(&old, 9), vec![id("undead")]);
    assert_eq!(mods(&old, 10), vec![id("undead")]);
    // A third may come since 05.11.2026 (no cap).
    assert!(
        old.edition
            .clone()
            .add_mod(&content, 11, &id("undead"), 1)
            .is_ok()
    );
    // A save with a special offer in the Auslage loads as it was.
    let (mut run, _) = tresen(&content, 6, 10);
    set_offers(
        &mut run,
        vec![CardOffer {
            card: 31,
            card_mod: id("wild_card"),
            value: 1,
            price: 7,
            bargain: None,
        }],
    );
    let json = serde_json::to_string(&run).expect("save");
    let loaded: Run = serde_json::from_str(&json).expect("load");
    assert_eq!(loaded, run);
}

// ---------------------------------------------------------------------------
// Bot
// ---------------------------------------------------------------------------

#[test]
fn the_bot_values_the_special_offers() {
    assert_eq!(bot::zinken_value(&id("undead")), 2);
    assert_eq!(bot::zinken_value(&id("wild_card")), 1);
    assert_eq!(bot::zinken_value(&id("voucher_undead")), 0);
}

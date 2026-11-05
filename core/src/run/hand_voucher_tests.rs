//! A hand game redeems a voucher (decided by the user on 04.11.2026,
//! DESIGN 7.2): declaring any hand game redeems one random voucher on a
//! card of the player's hand at once, as if the card had been discarded.
//! The card stays in the hand.

use super::content_tests::{card_mod_run_with, content, fixed_deal};
use super::*;
use crate::cards::Suit;
use crate::rules::{GameType, NullRisk, RiskLevel};

/// Declarer holds the four jacks and six diamonds. Ids: D7 = 0, D8 = 1,
/// D9 = 2, DA = 7, HA = 15, C7 = 24.
const STRONG: [&str; 3] = [
    "CJ SJ HJ DJ DA DT DK DQ D9 D7",
    "HA HT HK HQ H9 H8 H7 SA ST SK",
    "SQ S9 S8 S7 CA CT CK CQ C9 C8",
];
const STRONG_SKAT: &str = "D8 C7";

/// As [`STRONG`], but the declarer holds the H7 instead of the D7.
const ONE_HEART: [&str; 3] = [
    "CJ SJ HJ DJ DA DT DK DQ D9 H7",
    "HA HT HK HQ H9 H8 D7 SA ST SK",
    "SQ S9 S8 S7 CA CT CK CQ C9 C8",
];

fn id(m: &str) -> CardModId {
    CardModId::new(m)
}

fn mods(run: &Run, card: u32) -> Vec<CardModId> {
    run.edition().get(card).expect("card").mods.clone()
}

fn id_of(run: &Run, s: &str) -> u32 {
    run.edition()
        .id_of(s.parse().expect("card"))
        .expect("card in the edition")
}

fn diamonds_hand() -> Declaration {
    Declaration::new(GameType::Suit(Suit::Diamonds), RiskLevel::Hand).expect("suit")
}

fn declare(run: &mut Run, content: &Content, declaration: Declaration) -> Vec<RunEvent> {
    run.apply(content, RunAction::Declare(declaration))
        .expect("declare")
}

fn redeemed(events: &[RunEvent]) -> Vec<(u32, CardModId, bool)> {
    events
        .iter()
        .filter_map(|e| match e {
            RunEvent::VoucherRedeemed {
                card,
                card_mod,
                via_hand,
                ..
            } => Some((*card, card_mod.clone(), *via_hand)),
            _ => None,
        })
        .collect()
}

fn in_hand(run: &Run, card: u32) -> Option<DealtCard> {
    run.game()
        .and_then(|a| a.game.hand(Seat::DECLARER).iter().find(|c| c.id == card))
        .copied()
}

#[test]
fn a_hand_game_redeems_a_pfandmarke_in_the_hand() {
    let content = content();
    let mut run = card_mod_run_with(&content, 2, "voucher_coins", 3);
    fixed_deal(&mut run, &content, STRONG, STRONG_SKAT);
    let coins = run.coins();
    let events = declare(&mut run, &content, diamonds_hand());
    assert_eq!(redeemed(&events), vec![(2, id("voucher_coins"), true)]);
    assert!(events.contains(&RunEvent::VoucherRedeemed {
        card: 2,
        card_mod: id("voucher_coins"),
        value: 3,
        via_hand: true,
    }));
    assert_eq!(run.coins(), coins + 3);
    assert!(mods(&run, 2).is_empty(), "the voucher is gone");
    assert!(in_hand(&run, 2).is_some(), "the card stays in the hand");
    // Redeemed before the game starts: the event comes before `Declared`.
    let redeemed_at = events
        .iter()
        .position(|e| matches!(e, RunEvent::VoucherRedeemed { .. }));
    let declared_at = events
        .iter()
        .position(|e| matches!(e, RunEvent::Game(crate::game::Event::Declared { .. })));
    assert!(redeemed_at < declared_at);
}

#[test]
fn a_hand_game_redeems_a_totenmarke_and_the_card_may_rise_at_once() {
    let content = content();
    let mut run = card_mod_run_with(&content, 2, "voucher_undead", 1);
    fixed_deal(&mut run, &content, STRONG, STRONG_SKAT);
    let events = declare(&mut run, &content, diamonds_hand());
    assert_eq!(redeemed(&events), vec![(2, id("voucher_undead"), true)]);
    assert_eq!(mods(&run, 2), vec![id("undead")]);
    let card = in_hand(&run, 2).expect("D9 in the hand");
    assert!(run.can_rise(&content, card), "Untot from this game on");
}

#[test]
fn a_hand_game_redeems_a_narrenmarke_and_the_card_settles_with_the_hand() {
    let content = content();
    let (mut run, _) = Run::new(&content, 11).expect("start");
    let h7 = id_of(&run, "H7");
    run.edition
        .add_mod(&content, h7, &id("voucher_wild"), 1)
        .expect("voucher");
    fixed_deal(&mut run, &content, ONE_HEART, STRONG_SKAT);
    assert!(!in_hand(&run, h7).expect("H7").wild);
    let events = declare(&mut run, &content, diamonds_hand());
    assert_eq!(redeemed(&events), vec![(h7, id("voucher_wild"), true)]);
    assert_eq!(mods(&run, h7), vec![id("wild_card")]);
    // A Wandelkarte now, it takes the longest suit as the game starts.
    assert!(events.iter().any(|e| matches!(
        e,
        RunEvent::Game(crate::game::Event::WildCardSettled { old, .. }) if old.id == h7
    )));
    let card = in_hand(&run, h7).expect("still in the hand");
    assert!(!card.wild);
    assert_eq!(card.face().map(|f| f.suit), Some(Suit::Diamonds));
    let active = run.game().expect("game");
    assert!(active.named.contains(&(h7, Suit::Hearts)));
    // The view shows the settled card with its printed suit.
    let view = run.view(&content);
    let shown = view
        .game
        .as_ref()
        .and_then(|g| g.hand.iter().find(|c| c.card.id == h7))
        .expect("in the view");
    assert_eq!(shown.named, Some(Suit::Hearts));
    assert!(!shown.unsettled);
}

#[test]
fn a_hand_without_a_voucher_redeems_nothing() {
    let content = content();
    // The voucher lies on the C7 in the skat: a hand game leaves it.
    let mut run = card_mod_run_with(&content, 24, "voucher_coins", 3);
    fixed_deal(&mut run, &content, STRONG, STRONG_SKAT);
    let coins = run.coins();
    let events = declare(&mut run, &content, diamonds_hand());
    assert!(redeemed(&events).is_empty());
    assert_eq!(run.coins(), coins);
    assert_eq!(mods(&run, 24), vec![id("voucher_coins")]);
}

#[test]
fn null_hand_and_grand_hand_redeem_too() {
    let content = content();
    for declaration in [
        Declaration::null(NullRisk::Hand),
        Declaration::null(NullRisk::OuvertHand),
        Declaration::new(GameType::Grand, RiskLevel::Hand).expect("grand"),
    ] {
        let mut run = card_mod_run_with(&content, 2, "voucher_coins", 3);
        fixed_deal(&mut run, &content, STRONG, STRONG_SKAT);
        let events = declare(&mut run, &content, declaration);
        assert_eq!(
            redeemed(&events),
            vec![(2, id("voucher_coins"), true)],
            "{declaration:?}"
        );
    }
}

#[test]
fn a_game_after_the_skat_redeems_only_the_discarded_vouchers() {
    let content = content();
    // DA (7) stays in the hand, D9 (2) is discarded.
    let mut run = card_mod_run_with(&content, 7, "voucher_coins", 3);
    run.edition
        .add_mod(&content, 2, &id("voucher_coins"), 3)
        .expect("voucher");
    fixed_deal(&mut run, &content, STRONG, STRONG_SKAT);
    run.apply(&content, RunAction::PickUpSkat).expect("pick up");
    let mut events = run
        .apply(&content, RunAction::Discard([2, 24]))
        .expect("discard");
    let normal = Declaration::new(GameType::Suit(Suit::Diamonds), RiskLevel::Normal).expect("suit");
    events.extend(declare(&mut run, &content, normal));
    assert_eq!(redeemed(&events), vec![(2, id("voucher_coins"), false)]);
    assert_eq!(mods(&run, 7), vec![id("voucher_coins")]);
}

/// A run with a Pfandmarke on every card of the [`STRONG`] hand.
fn many_vouchers(content: &Content, games_dealt: u32) -> Run {
    let (mut run, _) = Run::new(content, 11).expect("start");
    for s in STRONG[0].split_whitespace() {
        let card = id_of(&run, s);
        run.edition
            .add_mod(content, card, &id("voucher_coins"), 1)
            .expect("voucher");
    }
    run.games_dealt = games_dealt;
    fixed_deal(&mut run, content, STRONG, STRONG_SKAT);
    run
}

#[test]
fn exactly_one_random_voucher_fires_and_the_draw_is_deterministic() {
    let content = content();
    let mut picked = std::collections::BTreeSet::new();
    for games_dealt in 0..12 {
        let mut a = many_vouchers(&content, games_dealt);
        let mut b = many_vouchers(&content, games_dealt);
        let coins = a.coins();
        let first = redeemed(&declare(&mut a, &content, diamonds_hand()));
        let second = redeemed(&declare(&mut b, &content, diamonds_hand()));
        assert_eq!(first.len(), 1, "one voucher only");
        assert_eq!(first, second, "same seed, same voucher");
        assert_eq!(a.coins(), coins + 1);
        picked.insert(first[0].0);
    }
    assert!(
        picked.len() > 3,
        "the draw varies with the game: {picked:?}"
    );
}

#[test]
fn a_totenmarke_in_the_hand_turns_its_card_when_another_card_is_untot() {
    // No cap on special cards since 05.11.2026 (DESIGN 7.2).
    let content = content();
    let mut run = card_mod_run_with(&content, 2, "voucher_undead", 1);
    run.edition
        .add_mod(&content, 15, &id("undead"), 1)
        .expect("Untot on HA");
    fixed_deal(&mut run, &content, STRONG, STRONG_SKAT);
    let coins = run.coins();
    declare(&mut run, &content, diamonds_hand());
    assert_eq!(run.coins(), coins, "nothing paid instead");
    assert_eq!(mods(&run, 2), vec![id("undead")]);
    assert_eq!(mods(&run, 15), vec![id("undead")]);
}

#[test]
fn a_narrenmarke_in_the_hand_turns_its_card_when_another_card_is_wandelkarte() {
    let content = content();
    let (mut run, _) = Run::new(&content, 11).expect("start");
    let h7 = id_of(&run, "H7");
    run.edition
        .add_mod(&content, h7, &id("voucher_wild"), 1)
        .expect("voucher");
    run.edition
        .add_mod(&content, 15, &id("wild_card"), 1)
        .expect("Wandelkarte on HA");
    fixed_deal(&mut run, &content, ONE_HEART, STRONG_SKAT);
    let coins = run.coins();
    declare(&mut run, &content, diamonds_hand());
    assert_eq!(run.coins(), coins);
    assert_eq!(
        run.edition().get(h7).expect("H7").mods,
        vec![id("wild_card")]
    );
}

// ---------------------------------------------------------------------------
// Der Drückeberger (DESIGN 9.4)
// ---------------------------------------------------------------------------

fn shirker(content: &Content) -> Run {
    let start = RunStart {
        character: Character::Shirker,
        ..RunStart::default()
    };
    let (run, _) = Run::new_with(content, 11, &start).expect("start");
    run
}

#[test]
fn shirker_voucher_redeemed_by_a_hand_game_comes_back() {
    let content = content();
    let mut run = shirker(&content);
    // The D7 (Pfandmarke 2) is the only voucher in the hand.
    fixed_deal(&mut run, &content, STRONG, STRONG_SKAT);
    let d7 = id_of(&run, "D7");
    let events = declare(&mut run, &content, diamonds_hand());
    assert_eq!(redeemed(&events), vec![(d7, id("voucher_coins"), true)]);
    assert_eq!(
        run.series.returning_vouchers,
        vec![(d7, id("voucher_coins"), 2)]
    );
    let mut events = Vec::new();
    run.pub_lost(&content, &mut events).expect("pub over");
    assert!(events.contains(&RunEvent::VoucherReturned {
        card: d7,
        card_mod: id("voucher_coins"),
        value: 2,
    }));
    assert_eq!(mods(&run, d7), vec![id("voucher_coins")]);
}

#[test]
fn shirker_totenmarke_that_turned_its_card_by_a_hand_game_stays_gone() {
    let content = content();
    let mut run = shirker(&content);
    // The D8 (Totenmarke) is the only voucher in the hand.
    let hands = ["CJ SJ HJ DJ DA DT DK DQ D9 D8", STRONG[1], STRONG[2]];
    fixed_deal(&mut run, &content, hands, "D7 C7");
    let d8 = id_of(&run, "D8");
    let events = declare(&mut run, &content, diamonds_hand());
    assert_eq!(redeemed(&events), vec![(d8, id("voucher_undead"), true)]);
    assert_eq!(mods(&run, d8), vec![id("undead")]);
    let mut events = Vec::new();
    run.pub_lost(&content, &mut events).expect("pub over");
    assert!(
        !events
            .iter()
            .any(|e| matches!(e, RunEvent::VoucherReturned { .. }))
    );
    assert_eq!(mods(&run, d8), vec![id("undead")]);
}

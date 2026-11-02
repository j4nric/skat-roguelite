//! Effects v2 (DESIGN 7.2): one test per card modification, the rules of
//! one modification per card (raise, replace, curses block, Augen and
//! Ramponiert as one signed value), the voucher and the conversion of old
//! saves.

use super::content_tests::{card_mod_run_with, content, fixed_deal, play, relic_run};
use super::opponents::defender_hints;
use super::*;
use crate::content::TRUMP_MODS_NEED_PLAYER;
use crate::deck::{AUGEN, DeckCard, RAMPONIERT, legacy_mod};
use crate::rules::{GameType, RiskLevel};

/// The declarer holds all trumps of a diamonds game but the D8 (in the
/// skat) and wins every trick, the last one with the D7. Ids: D7 = 0,
/// D9 = 2, DA = 7, HA = 15, CJ = 28, C7 = 24.
const STRONG: [&str; 3] = [
    "CJ SJ HJ DJ DA DT DK DQ D9 D7",
    "HA HT HK HQ H9 H8 H7 SA ST SK",
    "SQ S9 S8 S7 CA CT CK CQ C9 C8",
];
const STRONG_SKAT: &str = "D8 C7";

fn hand(game: GameType) -> Declaration {
    Declaration::new(game, RiskLevel::Hand).expect("suit or grand")
}

fn diamonds() -> GameType {
    GameType::Suit(Suit::Diamonds)
}

fn id(m: &str) -> CardModId {
    CardModId::new(m)
}

fn card(deck: &Deck, card: u32) -> DeckCard {
    deck.get(card).expect("card").clone()
}

fn source(card_mod: &str, card: u32) -> Source {
    Source::CardMod {
        card_mod: id(card_mod),
        card,
    }
}

// ---------------------------------------------------------------------------
// The modifications
// ---------------------------------------------------------------------------

#[test]
fn ramponiert_takes_augen_below_zero_from_whoever_takes_the_card() {
    let content = content();
    // The D7 (no Augen) is ramponiert −5 and wins the last trick for the
    // declarer: he has 115 instead of 120.
    let mut run = card_mod_run_with(&content, 0, RAMPONIERT, 5);
    fixed_deal(&mut run, &content, STRONG, STRONG_SKAT);
    let seven = run
        .game()
        .and_then(|a| a.game.cards_in_play().iter().find(|c| c.id == 0).copied())
        .expect("D7");
    assert_eq!((seven.card_points, seven.point_debt), (0, 5));
    let (_, b) = play(&mut run, &content, hand(diamonds()));
    assert_eq!(b.outcome.declarer_card_points, 115);
    assert_eq!(b.outcome.defender_card_points, 0);
}

#[test]
fn rank_rises_to_three_and_no_further() {
    let content = content();
    let mut deck = Deck::standard();
    for expected in 1..=3 {
        let change = deck.add_mod(&content, 22, &id("rank"), 1).expect("rank");
        assert!(matches!(change, DeckChange::Modified { value, .. } if value == expected));
    }
    assert_eq!(
        deck.add_mod(&content, 22, &id("rank"), 1),
        Err(DeckError::AtMaximum(id("rank")))
    );
    let resolved = resolve_deck(&deck, &content, &[]);
    assert_eq!(resolved.cards[22].rank_bonus, 3);
}

#[test]
fn trump_bonus_works_only_on_a_card_the_player_holds() {
    let content = content();
    const { assert!(TRUMP_MODS_NEED_PLAYER, "ADR 0015 strict (DESIGN 7.2)") };
    // The HA (id 15) lies with seat 1: a hearts game gets nothing.
    let mut run = card_mod_run_with(&content, 15, "trump_bonus", 4);
    fixed_deal(&mut run, &content, STRONG, STRONG_SKAT);
    run.apply(
        &content,
        RunAction::Declare(hand(GameType::Suit(Suit::Hearts))),
    )
    .expect("declare");
    assert!(run.game().expect("game").lines.is_empty());
    // On the player's DA in a diamonds game: Grundwert +4.
    let mut run = card_mod_run_with(&content, 7, "trump_bonus", 4);
    fixed_deal(&mut run, &content, STRONG, STRONG_SKAT);
    let (_, b) = play(&mut run, &content, hand(diamonds()));
    assert_eq!(b.base_value, 13);
}

#[test]
fn stained_lowers_the_base_value_only_on_a_card_the_player_holds() {
    let content = content();
    let mut run = card_mod_run_with(&content, 15, "stained", 3);
    fixed_deal(&mut run, &content, STRONG, STRONG_SKAT);
    run.apply(
        &content,
        RunAction::Declare(hand(GameType::Suit(Suit::Hearts))),
    )
    .expect("declare");
    assert!(run.game().expect("game").lines.is_empty());
}

#[test]
fn first_trick_doubles_the_augen_of_the_first_trick_it_wins() {
    let content = content();
    // The CJ (id 28) carries "Erster Stich" ×2; the declarer leads it.
    let mut run = card_mod_run_with(&content, 28, "first_trick", 2);
    fixed_deal(&mut run, &content, STRONG, STRONG_SKAT);
    run.apply(&content, RunAction::Declare(hand(diamonds())))
        .expect("declare");
    let events = run.apply(&content, RunAction::Play(28)).expect("lead");
    let game = &run.game().expect("game").game;
    let first = game.tricks().first().expect("first trick");
    assert_eq!(first.winning_card().id, 28);
    let points = first.card_points();
    assert_eq!(game.bonus_points(), [points, 0, 0]);
    assert!(events.contains(&RunEvent::BonusPoints {
        source: source("first_trick", 28),
        seat: Seat::DECLARER,
        card_points: points,
    }));
    // Raised once more: ×4, three times the Augen on top.
    let mut deck = Deck::standard();
    deck.add_mod(&content, 28, &id("first_trick"), 2)
        .expect("x2");
    deck.add_mod(&content, 28, &id("first_trick"), 2)
        .expect("x4");
    assert_eq!(card(&deck, 28).mod_value(&id("first_trick")), 4);
}

#[test]
fn transparent_cards_of_the_player_are_known_to_the_defenders() {
    let content = content();
    // The DA (id 7) in the player's hand is Durchsichtig.
    let mut run = card_mod_run_with(&content, 7, "transparent", 1);
    fixed_deal(&mut run, &content, STRONG, STRONG_SKAT);
    run.apply(&content, RunAction::Declare(hand(diamonds())))
        .expect("declare");
    let active = run.game().expect("game");
    let view = active.game.view(Seat::DEFENDERS[0]).expect("view");
    let hints = defender_hints(active, &view);
    assert_eq!(hints.pinned.len(), 1);
    assert_eq!(hints.pinned[0].0.id, 7);
    assert_eq!(hints.pinned[0].1, Some(Seat::DECLARER));
    // On a defender's card the player sees who holds it.
    let mut run = card_mod_run_with(&content, 15, "transparent", 1);
    fixed_deal(&mut run, &content, STRONG, STRONG_SKAT);
    assert_eq!(
        run.revealed_holders(),
        vec![(
            *run.game()
                .expect("game")
                .game
                .hand(Seat::DEFENDERS[0])
                .iter()
                .find(|c| c.id == 15)
                .expect("HA"),
            Some(Seat::DEFENDERS[0])
        )]
    );
}

// ---------------------------------------------------------------------------
// Vouchers
// ---------------------------------------------------------------------------

/// A STRONG deal with `card_mod` on the D9 (id 2), the skat picked up and
/// D9 and C7 discarded.
fn discard_voucher(card_mod: &str) -> (Content, Run, Vec<RunEvent>, u32) {
    let content = content();
    let mut run = card_mod_run_with(&content, 2, card_mod, 3);
    fixed_deal(&mut run, &content, STRONG, STRONG_SKAT);
    let coins = run.coins();
    run.apply(&content, RunAction::PickUpSkat).expect("pick up");
    let events = run
        .apply(&content, RunAction::Discard([2, 24]))
        .expect("discard");
    (content, run, events, coins)
}

#[test]
fn a_voucher_pays_when_discarded_and_falls_off() {
    let (_, run, events, coins) = discard_voucher("voucher_coins");
    assert_eq!(run.coins(), coins + 3);
    assert!(events.contains(&RunEvent::VoucherRedeemed {
        card: 2,
        card_mod: id("voucher_coins"),
        value: 3,
    }));
    assert!(card(run.deck(), 2).mods.is_empty());
}

#[test]
fn a_special_voucher_turns_the_card_into_a_special_card() {
    let (_, run, _, _) = discard_voucher("voucher_undead");
    assert_eq!(card(run.deck(), 2).mods, vec![id("undead")]);
    let (_, run, _, _) = discard_voucher("voucher_wild");
    assert_eq!(card(run.deck(), 2).mods, vec![id("wild_card")]);
}

#[test]
fn a_voucher_stays_in_a_hand_game() {
    let content = content();
    let mut run = card_mod_run_with(&content, 2, "voucher_coins", 3);
    fixed_deal(&mut run, &content, STRONG, STRONG_SKAT);
    let coins = run.coins();
    let (events, _) = play(&mut run, &content, hand(diamonds()));
    assert!(
        !events
            .iter()
            .any(|e| matches!(e, RunEvent::VoucherRedeemed { .. }))
    );
    assert_eq!(card(run.deck(), 2).mods, vec![id("voucher_coins")]);
    assert!(run.coins() >= coins);
}

// ---------------------------------------------------------------------------
// One modification per card
// ---------------------------------------------------------------------------

#[test]
fn the_same_modification_raises_another_replaces() {
    let content = content();
    let mut deck = Deck::standard();
    deck.add_mod(&content, 7, &id(AUGEN), 3).expect("+3");
    deck.add_mod(&content, 7, &id(AUGEN), 5).expect("+5");
    assert_eq!(card(&deck, 7).mods, vec![id(AUGEN)]);
    assert_eq!(card(&deck, 7).mod_value(&id(AUGEN)), 8);
    let change = deck
        .add_mod(&content, 7, &id("trump_bonus"), 2)
        .expect("replace");
    assert_eq!(
        change,
        DeckChange::Modified {
            card: 7,
            card_mod: id("trump_bonus"),
            value: 2,
            replaced: Some(id(AUGEN)),
        }
    );
    assert_eq!(card(&deck, 7).mods, vec![id("trump_bonus")]);
    assert!(!card(&deck, 7).mod_values.contains_key(&id(AUGEN)));
    // A value-less modification cannot be raised.
    deck.add_mod(&content, 8, &id("voucher_undead"), 1)
        .expect("voucher");
    assert_eq!(
        deck.add_mod(&content, 8, &id("voucher_undead"), 1),
        Err(DeckError::AlreadyModified(id("voucher_undead")))
    );
}

#[test]
fn a_special_card_carries_one_normal_modification_besides() {
    let content = content();
    let mut deck = Deck::standard();
    deck.add_mod(&content, 7, &id("undead"), 1).expect("undead");
    deck.add_mod(&content, 7, &id(AUGEN), 4).expect("augen");
    deck.add_mod(&content, 7, &id("rank"), 1).expect("rank");
    assert_eq!(card(&deck, 7).mods, vec![id("undead"), id("rank")]);
    // Another special card replaces the first.
    deck.add_mod(&content, 7, &id("wild_card"), 1)
        .expect("wild");
    assert_eq!(card(&deck, 7).mods, vec![id("rank"), id("wild_card")]);
}

#[test]
fn a_curse_blocks_the_card() {
    let content = content();
    let mut deck = Deck::standard();
    // A curse replaces a normal modification.
    deck.add_mod(&content, 7, &id("rank"), 1).expect("rank");
    deck.add_mod(&content, 7, &id("stained"), 3).expect("curse");
    assert_eq!(card(&deck, 7).mods, vec![id("stained")]);
    for (m, v) in [("rank", 1), (AUGEN, 4), ("stained", 3), ("transparent", 1)] {
        assert_eq!(
            deck.add_mod(&content, 7, &id(m), v),
            Err(DeckError::Cursed(id("stained"))),
            "{m}"
        );
    }
    // Every target choice leaves the cursed card out.
    let mut run = relic_run(&content, &[]);
    run.deck
        .add_mod(&content, 7, &id("transparent"), 1)
        .expect("curse");
    assert!(!run.may_attach(&content, 7, &id(AUGEN)));
    assert!(
        run.card_mod_pool_for(&content, &card(&run.deck, 7))
            .is_empty()
    );
}

#[test]
fn augen_and_ramponiert_are_one_signed_value() {
    let content = content();
    let value = |deck: &Deck, m: &str| card(deck, 7).mod_value(&id(m));
    // Ramponiert −5 on Augen +3: −2, the curse.
    let mut deck = Deck::standard();
    deck.add_mod(&content, 7, &id(AUGEN), 3).expect("+3");
    let change = deck.add_mod(&content, 7, &id(RAMPONIERT), 5).expect("−5");
    assert_eq!(
        change,
        DeckChange::Modified {
            card: 7,
            card_mod: id(RAMPONIERT),
            value: 2,
            replaced: Some(id(AUGEN)),
        }
    );
    assert_eq!(card(&deck, 7).mods, vec![id(RAMPONIERT)]);
    assert_eq!(value(&deck, RAMPONIERT), 2);
    // Now cursed: nothing more, also no Augen.
    assert_eq!(
        deck.add_mod(&content, 7, &id(AUGEN), 9),
        Err(DeckError::Cursed(id(RAMPONIERT)))
    );
    // Ramponiert −5 on Augen +8: Augen +3 stays.
    let mut deck = Deck::standard();
    deck.add_mod(&content, 7, &id(AUGEN), 8).expect("+8");
    deck.add_mod(&content, 7, &id(RAMPONIERT), 5).expect("−5");
    assert_eq!(card(&deck, 7).mods, vec![id(AUGEN)]);
    assert_eq!(value(&deck, AUGEN), 3);
    // Exactly zero: the card is bare.
    deck.add_mod(&content, 7, &id(RAMPONIERT), 3).expect("−3");
    assert!(card(&deck, 7).mods.is_empty());
    // Augen with a negative value is Ramponiert.
    deck.add_mod(&content, 7, &id(AUGEN), -2).expect("−2");
    assert_eq!(card(&deck, 7).mods, vec![id(RAMPONIERT)]);
    assert_eq!(value(&deck, RAMPONIERT), 2);
}

#[test]
fn a_curse_pays_once_when_it_lands_and_not_when_it_does_not() {
    let content = content();
    let mut run = relic_run(&content, &[]);
    let coins = run.coins();
    let mut events = Vec::new();
    run.attach_mod(&content, 7, &id(RAMPONIERT), 5, &mut events)
        .expect("curse");
    assert_eq!(run.coins(), coins + 3, "3 Groschen, not 3 × 5");
    // On Augen +8 the result is Augen +3: no curse, no Groschen.
    run.attach_mod(&content, 8, &id(AUGEN), 8, &mut events)
        .expect("augen");
    let coins = run.coins();
    run.attach_mod(&content, 8, &id(RAMPONIERT), 5, &mut events)
        .expect("ramponiert");
    assert_eq!(run.coins(), coins);
}

// ---------------------------------------------------------------------------
// Old saves
// ---------------------------------------------------------------------------

#[test]
fn old_saves_convert_their_card_modifications() {
    let old = r#"(cards:[
        (id:0,kind:Normal("D7"),mods:["fat"]),
        (id:1,kind:Normal("D8"),mods:["gold_rim","fat"]),
        (id:2,kind:Normal("D9"),mods:["marked","double_marked"]),
        (id:3,kind:Normal("DT"),mods:["stamp","beer_stain"]),
        (id:4,kind:Normal("DJ"),mods:["crown","seal","undead"]),
        (id:5,kind:Normal("DQ"),mods:["horseshoe","homesick","glass_ace"]),
        (id:6,kind:Normal("DK"),mods:["black_cat","transparent"]),
        (id:7,kind:Normal("DA"),mods:["honorary_jack","cracked","battered"]),
    ],next_id:32)"#;
    let deck: Deck = ron::from_str(old).expect("old deck loads");
    let mods = |i: u32| {
        let c = card(&deck, i);
        c.mods
            .iter()
            .map(|m| (m.to_string(), c.mod_value(m)))
            .collect::<Vec<_>>()
    };
    let one = |m: &str, v: i32| vec![(m.to_owned(), v)];
    assert_eq!(mods(0), one("points", 4));
    assert_eq!(mods(1), one("points", 12), "equal ones add up");
    assert_eq!(mods(2), one("rank", 3), "rank at most 3");
    assert_eq!(mods(3), one("battered", 5), "a curse wins");
    assert_eq!(
        mods(4),
        vec![("undead".to_owned(), 1), ("trump_bonus".to_owned(), 6)],
        "rank, trump, last trick, augen: trump beats last trick; specials stay"
    );
    assert_eq!(mods(5), one("points", 11), "Glasdaus: Augen +11");
    assert_eq!(mods(6), one("stained", 3), "Fleckig before Durchsichtig");
    assert!(mods(7).is_empty(), "everything else falls away");
    // Saved again, it stays as it is.
    let again: Deck = ron::from_str(&ron::to_string(&deck).expect("ron")).expect("load");
    assert_eq!(again, deck);
    assert_eq!(legacy_mod("tail_light"), Some(("last_trick", 1)));
    assert_eq!(legacy_mod("dummy"), None);
}

#[test]
fn an_open_encounter_of_an_old_save_gives_the_converted_modification() {
    let content = content();
    let mut run = relic_run(&content, &[]);
    run.game = None;
    run.phase = RunPhase::Encounter;
    run.encounter = Some(ActiveEncounter {
        id: EncounterId::new("crown_inn"),
        options: vec![
            EncounterOption {
                choice: EncounterChoice::MarkChosenCard,
                cost: 0,
                card_mod: Some(id("crown")),
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
    run.apply(
        &content,
        RunAction::ChooseEncounter {
            option: 0,
            target: Some(7),
        },
    )
    .expect("chosen");
    assert_eq!(card(run.deck(), 7).mods, vec![id("trump_bonus")]);
    assert_eq!(card(run.deck(), 7).mod_value(&id("trump_bonus")), 6);
}

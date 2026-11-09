//! Build directions (decided by the user on 04.11.2026, Cockpit q26): the
//! tags of the content and one test per connector relic, each in a
//! constructed situation. Also "Großvaters Grand" (Cockpit q27).

use super::content_tests::{
    CLUBS, DIAMONDS, STRONG, STRONG_SKAT, WEAK, WEAK_SKAT, content, fixed_deal, hand_game,
    has_line, play, relic, relic_run,
};
use super::wave3_tests::{bot_to_end, deal_with, defender_plays, hand_clubs, id_of};
use super::*;
use crate::content::Change;
use crate::content::Lever;
use crate::content::{BuildTag, CardCount, Factor, ModKind, Rarity, RelicGroup};
use crate::rules::{GameType, NullRisk};

/// The declarer holds the lowest cards of every suit: a Null he cannot
/// lose.
const LOW: [&str; 3] = [
    "D7 D8 D9 H7 H8 H9 S7 S8 S9 C7",
    "CJ SJ HJ DJ CA CT CK CQ DA DT",
    "DK DQ HA HT HK HQ SA ST SK SQ",
];
const LOW_SKAT: &str = "C8 C9";

/// Player forehand with all four Unter and the clubs top; the left defender
/// has no trump, the right one CQ and C8 (as in `zinken_v2_tests`).
const HANDS: [&str; 3] = [
    "CJ SJ HJ DJ CA CT CK C9 D7 H7",
    "HA HT HK HQ H9 H8 SA ST SK SQ",
    "S9 S8 S7 DA DT DK DQ D9 CQ C8",
];
const SKAT: &str = "D8 C7";

/// A run with these relics, a target it will not reach and room for many
/// games.
fn long_run(content: &Content, relics: &[&str]) -> Run {
    let mut run = relic_run(content, relics);
    run.series.target = 1_000_000;
    run.series.game_limit = 100;
    run
}

/// Put `card_mod` with its first value on the printed card `notation`.
fn put(run: &mut Run, content: &Content, notation: &str, card_mod: &str) {
    let id = CardModId::new(card_mod);
    let value = content
        .card_mod(&id)
        .map_or(1, crate::content::CardModDef::default_value);
    run.edition
        .add_mod(content, id_of(notation), &id, value)
        .expect("mod");
}

/// The run of [`HANDS`] with the relics and Zinken, a clubs hand game
/// declared.
fn declared(relics: &[&str], zinken: &[(&str, &str)]) -> (Content, Run) {
    let content = content();
    let mut run = relic_run(&content, relics);
    for (card, card_mod) in zinken {
        put(&mut run, &content, card, card_mod);
    }
    deal_with(&mut run, &content, HANDS, SKAT, Seat::DECLARER);
    run.apply_step(&content, RunAction::Declare(hand_clubs()))
        .expect("declare");
    (content, run)
}

/// Player leads `lead`, the defenders answer.
fn trick(run: &mut Run, content: &Content, lead: &str, left: &str, right: &str) -> Vec<RunEvent> {
    let mut events = run
        .apply_step(content, RunAction::Play(id_of(lead)))
        .expect("lead");
    events.extend(defender_plays(run, content, Seat::DEFENDERS[0], left));
    events.extend(defender_plays(run, content, Seat::DEFENDERS[1], right));
    events
}

fn coins_from(events: &[RunEvent], id: &str) -> i32 {
    events
        .iter()
        .filter_map(|e| match e {
            RunEvent::CoinsGained { source, amount, .. } if *source == relic(id) => Some(*amount),
            _ => None,
        })
        .sum()
}

fn bonus_from(events: &[RunEvent], id: &str) -> u32 {
    events
        .iter()
        .filter_map(|e| match e {
            RunEvent::BonusPoints {
                source,
                card_points,
                ..
            } if *source == relic(id) => Some(*card_points),
            _ => None,
        })
        .sum()
}

fn tags_of(content: &Content, id: &str) -> Vec<BuildTag> {
    content.relic(&RelicId::new(id)).expect(id).tags.clone()
}

// ------------------------------------------------------------------ Tags

#[test]
fn the_parts_of_every_direction_carry_its_tag() {
    let content = content();
    for (tag, relics, mods) in [
        (
            BuildTag::OneSuit,
            &[
                "home_suit",
                "jay",
                "tree_frog",
                "robin",
                "bell_tree",
                "club_colours",
                "true_soul",
            ][..],
            &["trump_bonus"][..],
        ),
        (
            BuildTag::Vouchers,
            &["skat_keeper", "rebate_book", "empties", "stamp_album"][..],
            &["voucher_coins", "voucher_undead", "voucher_wild"][..],
        ),
        (
            BuildTag::Tricks,
            &["blank_king", "fingertips", "tally_sheet", "tip"][..],
            &["gilt_edge", "crosshair", "last_trick", "first_trick"][..],
        ),
        (
            BuildTag::Schneider,
            &[
                "virgin",
                "tailor",
                "watertight",
                "golden_shears",
                "tape_measure",
                "sewing_pattern",
            ][..],
            &["tailors_chalk"][..],
        ),
        (
            BuildTag::Null,
            &[
                "black_zero",
                "null_eight_fifteen",
                "lead_pouring",
                "plumb_line",
            ][..],
            &["lead"][..],
        ),
    ] {
        for id in relics {
            assert!(tags_of(&content, id).contains(&tag), "{id}");
        }
        for id in mods {
            let def = content.card_mod(&CardModId::new(id)).expect(id);
            assert!(def.tags.contains(&tag), "{id}");
        }
    }
    let curacao = content
        .consumable(&ConsumableId::new("blue_curacao"))
        .expect("Blue Curaçao");
    assert_eq!(curacao.tags, vec![BuildTag::OneSuit]);
    // Untagged items stay untagged.
    assert!(tags_of(&content, "landlord").is_empty());
    // The Grand direction (relics v4, 07.11.2026).
    for id in [
        "grandfathers_grand",
        "grand_seigneur",
        "big_shots",
        "safe_haven",
        "beer_calm",
        "open_visor",
    ] {
        assert_eq!(tags_of(&content, id), vec![BuildTag::Grand], "{id}");
    }
    // The tags read from RON as written.
    let tags: Vec<BuildTag> =
        ron::from_str("[OneSuit, Vouchers, Tricks, Schneider, Null, Grand]").expect("ron");
    assert_eq!(tags.len(), 6);
}

#[test]
fn the_new_relics_speak_of_the_deck_and_the_factor_one_is_rare() {
    let content = content();
    let def = content
        .relic(&RelicId::new("golden_shears"))
        .expect("Goldene Schere");
    assert_eq!(def.group, RelicGroup::Factor);
    assert_eq!(def.rarity, Rarity::Rare);
    assert!(def.text.starts_with('×'));
    // "Ausgabe" is gone from every player-facing text (04.11.2026).
    for def in content.relics.values() {
        assert!(!def.text.contains("Ausgabe"), "{}", def.id);
    }
    for def in content.consumables.values() {
        assert!(!def.text.contains("Ausgabe"), "{}", def.id);
    }
    for def in content.card_mods.values() {
        assert!(!def.text.contains("Ausgabe"), "{}", def.id);
    }
    for def in content.bosses.values() {
        assert!(!def.text.contains("Ausgabe"), "{}", def.id);
    }
    for def in content.encounters.values() {
        assert!(!def.text.contains("Ausgabe"), "{}", def.id);
    }
}

// ------------------------------------------------------------- Farbtreue

#[test]
fn club_colours_counts_the_trump_suit_cards_with_a_zinke_in_the_deck() {
    let content = content();
    let mut run = long_run(&content, &["club_colours"]);
    // Diamonds: Daus and Unter in the hand, the Acht in the skat, all with
    // a Zinke (+2 each); the König with a curse and the Herz-Daus do not
    // count.
    put(&mut run, &content, "DA", "points");
    put(&mut run, &content, "DJ", "rank");
    put(&mut run, &content, "D8", "gilt_edge");
    put(&mut run, &content, "DK", "stained");
    put(&mut run, &content, "HA", "points");
    let count = CardCount {
        trump_suit: true,
        ..CardCount::default()
    };
    assert_eq!(run.count_cards(&content, &count), 0, "no game declared");
    fixed_deal(&mut run, &content, STRONG, STRONG_SKAT);
    let (_, b) = play(&mut run, &content, hand_game(DIAMONDS));
    assert!(has_line(
        &b,
        &relic("club_colours"),
        Lever::BaseValue,
        Change::Add(6)
    ));
    // Another trump suit: none of its cards carries a Zinke.
    fixed_deal(&mut run, &content, STRONG, STRONG_SKAT);
    let (_, b) = play(&mut run, &content, hand_game(CLUBS));
    assert!(b.lines_of(&relic("club_colours")).next().is_none());
    // Grand: no trump suit.
    fixed_deal(&mut run, &content, STRONG, STRONG_SKAT);
    let (_, b) = play(&mut run, &content, hand_game(GameType::Grand));
    assert!(b.lines_of(&relic("club_colours")).next().is_none());
}

#[test]
fn true_soul_adds_the_trump_suit_in_the_hand_to_the_base_value() {
    // Relics v4 (07.11.2026): +1 per card of the trump suit the player
    // holds after the skat, Zinke or not, without the Unter.
    let content = content();
    let mut run = long_run(&content, &["true_soul"]);
    // STRONG in diamonds: DA DT DK DQ D9 D7 in the hand (the D8 in the
    // skat does not count), four Unter left out.
    fixed_deal(&mut run, &content, STRONG, STRONG_SKAT);
    let (_, b) = play(&mut run, &content, hand_game(DIAMONDS));
    assert!(has_line(
        &b,
        &relic("true_soul"),
        Lever::BaseValue,
        Change::Add(6)
    ));
    assert_eq!(b.base_value, 9 + 6);
    // Clubs: no club in the hand.
    fixed_deal(&mut run, &content, STRONG, STRONG_SKAT);
    let (_, b) = play(&mut run, &content, hand_game(CLUBS));
    assert!(b.lines_of(&relic("true_soul")).next().is_none());
    // Grand: no trump suit.
    fixed_deal(&mut run, &content, STRONG, STRONG_SKAT);
    let (_, b) = play(&mut run, &content, hand_game(GameType::Grand));
    assert!(b.lines_of(&relic("true_soul")).next().is_none());
    // Lost games count too (a Grundwert line): WEAK holds one club, C7.
    fixed_deal(&mut run, &content, WEAK, WEAK_SKAT);
    let (_, b) = play(&mut run, &content, hand_game(CLUBS));
    assert!(!b.won);
    assert!(has_line(
        &b,
        &relic("true_soul"),
        Lever::BaseValue,
        Change::Add(1)
    ));
}

// ---------------------------------------------------------- Markenhändler

/// A run with a Pfandmarke (value 3) on the D9 in the hand of [`STRONG`].
fn voucher_run(content: &Content, relics: &[&str]) -> Run {
    let mut run = long_run(content, relics);
    run.edition
        .add_mod(content, id_of("D9"), &CardModId::new("voucher_coins"), 3)
        .expect("Pfandmarke");
    run
}

#[test]
fn rebate_book_adds_a_level_per_redeemed_marke_until_the_pub_is_won() {
    let content = content();
    let mut run = voucher_run(&content, &["rebate_book"]);
    fixed_deal(&mut run, &content, STRONG, STRONG_SKAT);
    // The hand game redeems the Marke in the hand.
    let (events, b) = play(&mut run, &content, hand_game(DIAMONDS));
    assert!(
        events
            .iter()
            .any(|e| matches!(e, RunEvent::VoucherRedeemed { .. }))
    );
    assert_eq!(run.counters().get("rebate_book"), Some(&1));
    assert!(b.won);
    assert!(has_line(
        &b,
        &relic("rebate_book"),
        Lever::Levels,
        Change::Add(1)
    ));
    // The next game of the pub keeps it, without a new Marke.
    fixed_deal(&mut run, &content, STRONG, STRONG_SKAT);
    let (_, b) = play(&mut run, &content, hand_game(DIAMONDS));
    assert!(has_line(
        &b,
        &relic("rebate_book"),
        Lever::Levels,
        Change::Add(1)
    ));
    // A won pub ends it.
    let mut events = Vec::new();
    run.fire(&content, Stage::OnSeriesWon, None, None, &mut events);
    assert_eq!(run.counters().get("rebate_book"), None);
}

#[test]
fn empties_pays_two_groschen_per_redeemed_marke() {
    let content = content();
    let mut run = voucher_run(&content, &["empties"]);
    fixed_deal(&mut run, &content, STRONG, STRONG_SKAT);
    let coins = run.coins();
    let events = run
        .apply(&content, RunAction::Declare(hand_game(DIAMONDS)))
        .expect("declare");
    assert_eq!(coins_from(&events, "empties"), 2);
    // The Pfandmarke pays its 3, Leergut 2 more.
    assert_eq!(run.coins(), coins + 5);
    // Without a Marke in the hand nothing is redeemed, nothing paid.
    let mut run = long_run(&content, &["empties"]);
    fixed_deal(&mut run, &content, STRONG, STRONG_SKAT);
    let events = run
        .apply(&content, RunAction::Declare(hand_game(DIAMONDS)))
        .expect("declare");
    assert_eq!(coins_from(&events, "empties"), 0);
}

#[test]
fn stamp_album_counts_the_marken_still_on_cards() {
    let content = content();
    let mut run = long_run(&content, &["stamp_album"]);
    // Two Marken on cards the defenders hold, a Zinke and a curse that do
    // not count.
    put(&mut run, &content, "HA", "voucher_coins");
    put(&mut run, &content, "SK", "voucher_wild");
    put(&mut run, &content, "CA", "points");
    put(&mut run, &content, "C8", "transparent");
    let count = CardCount {
        kind: Some(ModKind::Voucher),
        ..CardCount::default()
    };
    assert_eq!(run.count_cards(&content, &count), 2);
    fixed_deal(&mut run, &content, STRONG, STRONG_SKAT);
    let (_, b) = play(&mut run, &content, hand_game(DIAMONDS));
    assert!(has_line(
        &b,
        &relic("stamp_album"),
        Lever::BaseValue,
        Change::Add(2)
    ));
    // A redeemed Marke no longer counts: the one in the hand goes before
    // the declaration.
    let mut run = voucher_run(&content, &["stamp_album"]);
    fixed_deal(&mut run, &content, STRONG, STRONG_SKAT);
    let (_, b) = play(&mut run, &content, hand_game(DIAMONDS));
    assert!(b.lines_of(&relic("stamp_album")).next().is_none());
}

// ------------------------------------------------------------- Stichjäger

#[test]
fn fingertips_adds_three_augen_to_a_trick_taken_with_a_zinke() {
    let (content, mut run) = declared(&["fingertips"], &[("CJ", "points")]);
    let events = trick(&mut run, &content, "CJ", "H8", "C8");
    assert_eq!(bonus_from(&events, "fingertips"), 3);
    let active = run.game().expect("game");
    assert_eq!(active.game.bonus_points()[0], 3);
    // A trick taken with a card without a Zinke: nothing.
    let events = trick(&mut run, &content, "SJ", "H9", "CQ");
    assert_eq!(bonus_from(&events, "fingertips"), 0);
    // A Zinke on a card that loses its trick: nothing.
    let (content, mut run) = declared(&["fingertips"], &[("H7", "points")]);
    let events = trick(&mut run, &content, "H7", "HA", "S9");
    assert_eq!(bonus_from(&events, "fingertips"), 0);
}

#[test]
fn tally_sheet_gives_levels_for_three_and_five_tricks_with_a_zinke() {
    let zinken = [("CJ", "points"), ("SJ", "gilt_edge"), ("HJ", "rank")];
    let (content, mut run) = declared(&["tally_sheet"], &zinken);
    trick(&mut run, &content, "CJ", "H8", "C8");
    trick(&mut run, &content, "SJ", "H9", "CQ");
    trick(&mut run, &content, "HJ", "HK", "S7");
    assert_eq!(run.counters().get("tally_sheet"), Some(&3));
    let (_, b) = bot_to_end(&mut run, &content);
    assert!(b.won);
    let lines: Vec<Change> = b
        .lines_of(&relic("tally_sheet"))
        .map(|l| l.change)
        .collect();
    assert_eq!(lines, vec![Change::Add(1)]);
    // Five tricks: +2.
    let zinken = [
        ("CJ", "points"),
        ("SJ", "gilt_edge"),
        ("HJ", "rank"),
        ("DJ", "points"),
        ("CA", "points"),
    ];
    let (content, mut run) = declared(&["tally_sheet"], &zinken);
    trick(&mut run, &content, "CJ", "H8", "C8");
    trick(&mut run, &content, "SJ", "H9", "CQ");
    trick(&mut run, &content, "HJ", "HK", "S7");
    trick(&mut run, &content, "DJ", "HQ", "S8");
    trick(&mut run, &content, "CA", "HT", "S9");
    assert_eq!(run.counters().get("tally_sheet"), Some(&5));
    let (_, b) = bot_to_end(&mut run, &content);
    assert!(b.won);
    assert_eq!(b.lines_of(&relic("tally_sheet")).count(), 2);
    // The next declaration starts the count over.
    deal_with(&mut run, &content, HANDS, SKAT, Seat::DECLARER);
    run.apply_step(&content, RunAction::Declare(hand_clubs()))
        .expect("declare");
    assert_eq!(run.counters().get("tally_sheet"), None);
}

#[test]
fn tip_pays_a_groschen_per_trick_with_a_zinke_three_per_game() {
    let zinken = [
        ("CJ", "points"),
        ("SJ", "points"),
        ("HJ", "points"),
        ("DJ", "points"),
    ];
    let (content, mut run) = declared(&["tip"], &zinken);
    let coins = run.coins();
    let mut paid = 0;
    paid += coins_from(&trick(&mut run, &content, "CJ", "H8", "C8"), "tip");
    paid += coins_from(&trick(&mut run, &content, "SJ", "H9", "CQ"), "tip");
    paid += coins_from(&trick(&mut run, &content, "HJ", "HK", "S7"), "tip");
    paid += coins_from(&trick(&mut run, &content, "DJ", "HQ", "S8"), "tip");
    assert_eq!(paid, 3);
    assert_eq!(run.coins(), coins + 3);
}

#[test]
fn a_trick_with_a_zinke_still_to_come_keeps_the_game_open() {
    // "Rest abkürzen" waits while a card with a Zinke may still take a
    // trick for the Stichjäger relics.
    let (content, run) = declared(&["fingertips"], &[("CA", "points")]);
    let ctx_marked = run.marked_cards(&content);
    assert!(ctx_marked.contains(&id_of("CA")));
    assert!(!ctx_marked.contains(&id_of("CJ")));
    assert!(!run.can_settle(&content));
}

// --------------------------------------------------------------- Schneider

#[test]
fn golden_shears_scales_a_game_won_with_schneider() {
    let content = content();
    let mut run = long_run(&content, &["golden_shears"]);
    fixed_deal(&mut run, &content, STRONG, STRONG_SKAT);
    let (_, b) = play(&mut run, &content, hand_game(DIAMONDS));
    assert!(b.won);
    assert!(has_line(
        &b,
        &relic("golden_shears"),
        Lever::GameValue,
        Change::Scale(Factor::from_permille(1500))
    ));
    assert_eq!(b.multiplier, Factor::from_permille(1500));
    // A lost game: nothing.
    fixed_deal(&mut run, &content, WEAK, WEAK_SKAT);
    let (_, b) = play(&mut run, &content, hand_game(CLUBS));
    assert!(!b.won);
    assert!(b.lines_of(&relic("golden_shears")).next().is_none());
}

#[test]
fn tape_measure_grows_by_one_per_schneider_win() {
    let content = content();
    let mut run = long_run(&content, &["tape_measure"]);
    fixed_deal(&mut run, &content, STRONG, STRONG_SKAT);
    let (_, b) = play(&mut run, &content, hand_game(DIAMONDS));
    assert!(b.won);
    assert!(b.lines_of(&relic("tape_measure")).next().is_none());
    assert_eq!(run.counters().get("tape_measure"), Some(&1));
    fixed_deal(&mut run, &content, STRONG, STRONG_SKAT);
    let (_, b) = play(&mut run, &content, hand_game(DIAMONDS));
    assert!(has_line(
        &b,
        &relic("tape_measure"),
        Lever::BaseValue,
        Change::Add(1)
    ));
    assert_eq!(run.counters().get("tape_measure"), Some(&2));
    // A lost game raises nothing.
    fixed_deal(&mut run, &content, WEAK, WEAK_SKAT);
    play(&mut run, &content, hand_game(CLUBS));
    assert_eq!(run.counters().get("tape_measure"), Some(&2));
}

#[test]
fn sewing_pattern_counts_aufdruck_and_schneiderkreide_on_a_schneider_win() {
    let content = content();
    let mut run = long_run(&content, &["sewing_pattern"]);
    put(&mut run, &content, "HA", "points");
    put(&mut run, &content, "SK", "tailors_chalk");
    put(&mut run, &content, "CA", "gilt_edge");
    fixed_deal(&mut run, &content, STRONG, STRONG_SKAT);
    let (_, b) = play(&mut run, &content, hand_game(DIAMONDS));
    assert!(b.won);
    assert!(has_line(
        &b,
        &relic("sewing_pattern"),
        Lever::BaseValue,
        Change::Add(2)
    ));
    fixed_deal(&mut run, &content, WEAK, WEAK_SKAT);
    let (_, b) = play(&mut run, &content, hand_game(CLUBS));
    assert!(b.lines_of(&relic("sewing_pattern")).next().is_none());
}

// ------------------------------------------------------------ Nullspieler

#[test]
fn lead_pouring_adds_a_level_per_blei_card_in_the_hand_of_a_won_null() {
    let content = content();
    let mut run = long_run(&content, &["lead_pouring"]);
    // Two Blei cards in the hand, one with a defender, one in the skat.
    put(&mut run, &content, "D7", "lead");
    put(&mut run, &content, "H7", "lead");
    put(&mut run, &content, "CA", "lead");
    put(&mut run, &content, "C8", "lead");
    fixed_deal(&mut run, &content, LOW, LOW_SKAT);
    let (_, b) = play(&mut run, &content, Declaration::null(NullRisk::Hand));
    assert!(b.won);
    assert!(has_line(
        &b,
        &relic("lead_pouring"),
        Lever::Levels,
        Change::Add(2)
    ));
    // Not in a suit game.
    fixed_deal(&mut run, &content, STRONG, STRONG_SKAT);
    let (_, b) = play(&mut run, &content, hand_game(DIAMONDS));
    assert!(b.lines_of(&relic("lead_pouring")).next().is_none());
}

#[test]
fn plumb_line_adds_one_grundwert_per_blei_card_in_every_null() {
    let content = content();
    let mut run = long_run(&content, &["plumb_line"]);
    put(&mut run, &content, "D7", "lead");
    put(&mut run, &content, "CA", "lead");
    put(&mut run, &content, "SA", "lead");
    fixed_deal(&mut run, &content, LOW, LOW_SKAT);
    let (_, b) = play(&mut run, &content, Declaration::null(NullRisk::Hand));
    assert!(has_line(
        &b,
        &relic("plumb_line"),
        Lever::BaseValue,
        Change::Add(3)
    ));
    fixed_deal(&mut run, &content, STRONG, STRONG_SKAT);
    let (_, b) = play(&mut run, &content, hand_game(DIAMONDS));
    assert!(b.lines_of(&relic("plumb_line")).next().is_none());
}

// ------------------------------------------------------------------ Grand

#[test]
fn grandfathers_grand_adds_six_grundwert_to_every_grand() {
    let content = content();
    let mut run = long_run(&content, &["grandfathers_grand"]);
    fixed_deal(&mut run, &content, STRONG, STRONG_SKAT);
    let (_, b) = play(&mut run, &content, hand_game(GameType::Grand));
    assert!(has_line(
        &b,
        &relic("grandfathers_grand"),
        Lever::BaseValue,
        Change::Add(6)
    ));
    let mut plain = long_run(&content, &[]);
    fixed_deal(&mut plain, &content, STRONG, STRONG_SKAT);
    let (_, p) = play(&mut plain, &content, hand_game(GameType::Grand));
    assert_eq!(b.base_value, p.base_value + 6);
    fixed_deal(&mut run, &content, STRONG, STRONG_SKAT);
    let (_, b) = play(&mut run, &content, hand_game(DIAMONDS));
    assert!(b.lines_of(&relic("grandfathers_grand")).next().is_none());
}

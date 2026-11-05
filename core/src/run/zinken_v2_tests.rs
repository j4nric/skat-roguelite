//! Zinken v2 (DESIGN 7.2, user decisions of 03.11.2026): Blei, Goldkante,
//! Fadenkreuz, Schneiderkreide, the new Winkel and the Doppelgänger. One
//! test per rule edge; the Untot rework is in `undead_tests.rs`.

use super::content_tests::content;
use super::wave3_tests::{bot_to_end, deal_with, defender_plays, hand_clubs, id_of, with_mod};
use super::*;
use crate::cards::Rank;
use crate::content::{Lever, resolve_edition};
use crate::deck::{AUGEN, DeckError, MODS_VERSION, Owner, WINKEL_TO_AUGEN};
use crate::rules::{NullRisk, beats};

/// Player forehand with all four Unter and the clubs top; the left defender
/// has no trump, the right one CQ and C8.
const HANDS: [&str; 3] = [
    "CJ SJ HJ DJ CA CT CK C9 D7 H7",
    "HA HT HK HQ H9 H8 SA ST SK SQ",
    "S9 S8 S7 DA DT DK DQ D9 CQ C8",
];
const SKAT: &str = "D8 C7";

fn kind(notation: &str) -> CardKind {
    notation.parse().expect("card")
}

fn run_with(card_mods: &[(&str, &str)], hands: [&str; 3], skat: &str) -> (Content, Run) {
    let content = content();
    let (mut run, _) = Run::new(&content, 11).expect("start");
    for (card, card_mod) in card_mods {
        with_mod(&content, &mut run, card, card_mod);
    }
    deal_with(&mut run, &content, hands, skat, Seat::DECLARER);
    (content, run)
}

fn declared(card_mods: &[(&str, &str)]) -> (Content, Run) {
    let (content, mut run) = run_with(card_mods, HANDS, SKAT);
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

fn from_mod(source: &Source, id: &str) -> bool {
    matches!(source, Source::CardMod { card_mod, .. } if card_mod.as_str() == id)
}

fn coins_from(events: &[RunEvent], id: &str) -> i32 {
    events
        .iter()
        .filter_map(|e| match e {
            RunEvent::CoinsGained { source, amount, .. } if from_mod(source, id) => Some(*amount),
            _ => None,
        })
        .sum()
}

fn level_lines(run: &Run, id: &str) -> usize {
    run.game()
        .expect("game")
        .lines
        .iter()
        .filter(|l| from_mod(&l.source, id) && l.lever == Lever::Levels)
        .count()
}

fn dealt(notation: &str) -> DealtCard {
    DealtCard::plain(id_of(notation), kind(notation))
}

// ------------------------------------------------------------------ Blei

#[test]
fn lead_is_the_lowest_card_of_its_suit_in_null_only() {
    let content = content();
    let def = content.card_mod(&CardModId::new("lead")).expect("Blei");
    assert_eq!(def.name, "Blei");
    assert_eq!(def.focus, crate::content::GameFocus::Null);
    let mut edition = Edition::standard();
    edition
        .add_mod(&content, id_of("HA"), &CardModId::new("lead"), 1)
        .expect("Blei");
    let resolved = resolve_edition(&edition, Owner::Player, &content, &[]);
    let ace = *resolved
        .cards
        .iter()
        .find(|c| c.id == id_of("HA"))
        .expect("HA");
    assert!(ace.leaden);
    assert_eq!(ace.card_points, 11, "Augen stay");
    let seven = dealt("H7");
    // Null: the Sieben takes the leaden Daus, the leaden Daus nothing.
    assert!(beats(GameType::Null, ace, ace, seven));
    assert!(!beats(GameType::Null, seven, seven, ace));
    // Another suit game: the Daus as printed.
    assert!(!beats(GameType::Grand, ace, ace, seven));
    assert!(beats(GameType::Grand, seven, seven, ace));
}

// ------------------------------------------------------------- Goldkante

#[test]
fn gilt_edge_pays_a_groschen_for_every_trick_the_card_takes() {
    let (content, mut run) = declared(&[("CJ", "gilt_edge")]);
    let before = run.coins();
    let events = trick(&mut run, &content, "CJ", "H8", "C8");
    assert_eq!(coins_from(&events, "gilt_edge"), 1);
    assert_eq!(run.coins(), before + 1);
    // A card that loses its trick pays nothing.
    let (content, mut run) = declared(&[("H7", "gilt_edge")]);
    let events = trick(&mut run, &content, "H7", "HA", "S9");
    assert_eq!(coins_from(&events, "gilt_edge"), 0);
}

// ------------------------------------------------------------ Fadenkreuz

#[test]
fn crosshair_adds_a_level_when_the_card_beats_a_defenders_daus() {
    let (content, mut run) = declared(&[("CJ", "crosshair")]);
    trick(&mut run, &content, "CJ", "HA", "C8");
    assert_eq!(level_lines(&run, "crosshair"), 1);
    // No Daus in the trick: nothing.
    let (content, mut run) = declared(&[("CJ", "crosshair")]);
    trick(&mut run, &content, "CJ", "H8", "C8");
    assert_eq!(level_lines(&run, "crosshair"), 0);
    // The player's own Daus does not count: the CA wins with no Daus of a
    // defender in the trick.
    let (content, mut run) = declared(&[("CA", "crosshair")]);
    trick(&mut run, &content, "CA", "H8", "C8");
    assert_eq!(level_lines(&run, "crosshair"), 0);
}

// ------------------------------------------------------- Schneiderkreide

#[test]
fn tailors_chalk_adds_a_level_to_a_won_schneider_game_it_was_played_in() {
    // The player holds all ten trumps: Schwarz, so Schneider.
    let hands = [
        "CJ SJ HJ DJ CA CT CK CQ C9 C8",
        "HA HT HK HQ H9 H8 SA ST SK SQ",
        "S9 S8 S7 DA DT DK DQ D9 D8 D7",
    ];
    let score = |card: &str| {
        let (content, mut run) = run_with(&[(card, "tailors_chalk")], hands, "H7 C7");
        run.apply_step(&content, RunAction::Declare(hand_clubs()))
            .expect("declare");
        let (_, breakdown) = bot_to_end(&mut run, &content);
        assert!(breakdown.won && breakdown.schneider);
        breakdown
            .lines
            .iter()
            .filter(|l| from_mod(&l.source, "tailors_chalk"))
            .count()
    };
    assert_eq!(score("CA"), 1, "played and won with Schneider");
    assert_eq!(score("H7"), 0, "in the skat, never played");
}

// ---------------------------------------------------------------- Winkel

#[test]
fn ranks_promote_in_the_order_of_a_suit_and_unter_never() {
    assert_eq!(Rank::Seven.promoted(1), Rank::Eight);
    assert_eq!(Rank::Eight.promoted(2), Rank::Queen, "8, 9, Oberin");
    assert_eq!(Rank::Nine.promoted(1), Rank::Queen, "the Unter is skipped");
    assert_eq!(Rank::King.promoted(1), Rank::Ten);
    assert_eq!(Rank::King.promoted(2), Rank::Ace);
    assert_eq!(Rank::Ten.promoted(2), Rank::Ace, "no further than the Daus");
    assert_eq!(Rank::Ace.promoted(1), Rank::Ace);
    assert_eq!(Rank::Jack.promoted(2), Rank::Jack);
    assert_eq!(Rank::Jack.promotion_room(), 0);
    assert_eq!(Rank::Ace.promotion_room(), 0);
    assert_eq!(Rank::Ten.promotion_room(), 1);
    assert_eq!(Rank::Seven.promotion_room(), 6);
}

#[test]
fn winkel_makes_the_card_the_higher_card_and_keeps_its_augen() {
    let content = content();
    let rank = CardModId::new("rank");
    let mut edition = Edition::standard();
    edition
        .add_mod(&content, id_of("H8"), &rank, 2)
        .expect("Winkel 2");
    edition
        .add_mod(&content, id_of("SK"), &rank, 1)
        .expect("Winkel 1");
    let resolved = resolve_edition(&edition, Owner::Player, &content, &[]);
    let find = |n: &str| {
        *resolved
            .cards
            .iter()
            .find(|c| c.id == id_of(n))
            .expect("card")
    };
    let eight = find("H8");
    assert_eq!(eight.kind, kind("HQ"), "two steps: 8, 9, Oberin");
    assert_eq!(eight.promoted, 2);
    assert_eq!(eight.card_points, 0, "the Acht's Augen");
    let king = find("SK");
    assert_eq!(king.kind, kind("ST"));
    assert_eq!(king.card_points, 4, "the König's Augen");
    // In a suit game it plays as the Oberin: the König takes it, it takes
    // the Neun.
    let hearts = GameType::Suit(crate::cards::Suit::Hearts);
    assert!(beats(hearts, eight, eight, dealt("HK")));
    assert!(beats(hearts, dealt("H9"), dealt("H9"), eight));
}

#[test]
fn winkel_fits_no_unter_and_no_daus_and_a_ten_rises_once() {
    let content = content();
    let rank = CardModId::new("rank");
    let def = content.card_mod(&rank).expect("Winkel");
    assert!(!def.fits(kind("CJ")));
    assert!(!def.fits(kind("HA")));
    assert!(def.fits(kind("HT")));
    let mut edition = Edition::standard();
    edition
        .add_mod(&content, id_of("HT"), &rank, 1)
        .expect("one");
    assert_eq!(
        edition.add_mod(&content, id_of("HT"), &rank, 1),
        Err(DeckError::AtMaximum(rank.clone())),
        "the Daus is the top"
    );
    // Put there anyway (a boss edition, an old save): no effect.
    edition.force_mod(id_of("CJ"), &rank);
    edition.force_mod(id_of("DA"), &rank);
    let resolved = resolve_edition(&edition, Owner::Player, &content, &[]);
    for n in ["CJ", "DA"] {
        let c = resolved
            .cards
            .iter()
            .find(|c| c.id == id_of(n))
            .expect("card");
        assert_eq!(c.kind, kind(n), "{n} stays");
        assert_eq!(c.promoted, 0);
    }
    // The Auslage and the Kartenmaler never offer it there.
    let (run, _) = Run::new(&content, 1).expect("run");
    for n in ["CJ", "SA"] {
        assert!(!run.takes(&content, id_of(n), &rank, 1), "{n}");
    }
}

#[test]
fn winkel_works_in_null_too() {
    let content = content();
    let mut edition = Edition::standard();
    edition
        .add_mod(&content, id_of("H9"), &CardModId::new("rank"), 1)
        .expect("Winkel");
    let resolved = resolve_edition(&edition, Owner::Player, &content, &[]);
    let nine = *resolved
        .cards
        .iter()
        .find(|c| c.id == id_of("H9"))
        .expect("H9");
    assert_eq!(nine.kind, kind("HQ"));
    // Null order 7, 8, 9, 10, Unter, Oberin: it takes the Zehn and the Unter.
    for n in ["HT", "HJ"] {
        assert!(beats(GameType::Null, dealt(n), dealt(n), nine), "{n}");
    }
    assert!(beats(GameType::Null, nine, nine, dealt("HK")));
}

#[test]
fn old_saves_convert_winkel_to_the_new_meaning() {
    let content = content();
    let rank = CardModId::new("rank");
    let mut edition = Edition::standard();
    edition
        .add_mod(&content, id_of("H8"), &rank, 2)
        .expect("H8");
    edition.force_mod(id_of("CJ"), &rank);
    edition.force_mod(id_of("SA"), &rank);
    let text = ron::to_string(&edition).expect("save");
    let old = text.replace(&format!("mods_version:{MODS_VERSION}"), "mods_version:3");
    assert_ne!(old, text, "the version is written");
    let loaded: Edition = ron::from_str(&old).expect("load");
    let get = |e: &Edition, n: &str| e.get(id_of(n)).expect("card").clone();
    assert_eq!(get(&loaded, "H8").mod_value(&rank), 2, "kept");
    for n in ["CJ", "SA"] {
        let c = get(&loaded, n);
        assert!(!c.mods.contains(&rank), "{n}: no Winkel on Unter or Daus");
        assert_eq!(c.mod_value(&CardModId::new(AUGEN)), WINKEL_TO_AUGEN, "{n}");
    }
    let again = ron::to_string(&loaded).expect("save");
    assert!(again.contains(&format!("mods_version:{MODS_VERSION}")));
    // An old Winkel 2 on a Zehn keeps one step, the way to the Daus.
    let mut ten = get(&Edition::standard(), "DT");
    ten.mods.push(rank.clone());
    ten.mod_values.insert(rank.clone(), 2);
    crate::deck::migrate_winkel(&mut ten);
    assert_eq!(ten.mod_value(&rank), 1);
}

// ---------------------------------------------------------- Doppelgänger

const DG: &str = "doppelganger";

fn swapped(events: &[RunEvent]) -> Option<(Seat, DealtCard, DealtCard)> {
    events.iter().find_map(|e| match e {
        RunEvent::Game(Event::DoppelgangerSwapped {
            from,
            stolen,
            given,
            ..
        }) => Some((*from, *stolen, *given)),
        _ => None,
    })
}

#[test]
fn the_doppelganger_steals_from_the_next_opponent_first() {
    // C9: the left defender holds the H9, the right one the S9 and D9.
    let (content, mut run) = declared(&[("C9", DG)]);
    let events = run
        .apply_step(&content, RunAction::Play(id_of("C9")))
        .expect("lead");
    let (from, stolen, given) = swapped(&events).expect("swapped");
    assert_eq!(from, Seat::DEFENDERS[0]);
    assert_eq!(stolen.kind, kind("H9"));
    assert_eq!(given.id, id_of("C9"));
    assert_eq!(given.kind, kind("C9"), "its printed card");
    assert_ne!(
        given.owner,
        Owner::Player,
        "in his edition, no special card"
    );
    let g = &run.game().expect("game").game;
    assert_eq!(
        g.current_trick(),
        &[stolen],
        "the stolen card lies in the trick"
    );
    assert!(g.hand(Seat::DEFENDERS[0]).contains(&given));
    assert!(!g.hand(Seat::DECLARER).iter().any(|c| c.id == id_of("C9")));
    // It lies there as it is: a heart is led, the left defender follows it.
    for card in g.legal_moves(Seat::DEFENDERS[0]) {
        assert_eq!(
            card.face().map(|f| f.suit),
            Some(crate::cards::Suit::Hearts)
        );
    }
    // Then the other opponent: the D7 finds no Sieben on the left.
    let (content, mut run) = declared(&[("D7", DG)]);
    let events = run
        .apply_step(&content, RunAction::Play(id_of("D7")))
        .expect("lead");
    let (from, stolen, _) = swapped(&events).expect("swapped");
    assert_eq!(from, Seat::DEFENDERS[1]);
    assert_eq!(stolen.kind, kind("S7"));
}

#[test]
fn the_doppelganger_must_be_a_legal_play_as_printed() {
    let (content, mut run) = declared(&[("H7", DG)]);
    // The right defender takes the first trick with the DA and leads a
    // trump; the player holds trumps, so the H7 must not go.
    trick(&mut run, &content, "D7", "HA", "DA");
    assert_eq!(
        run.game().expect("game").game.trick_lead(),
        Seat::DEFENDERS[1]
    );
    defender_plays(&mut run, &content, Seat::DEFENDERS[1], "C8");
    assert!(matches!(
        run.apply_step(&content, RunAction::Play(id_of("H7"))),
        Err(RunError::Game(GameError::MustFollowSuit { .. }))
    ));
}

#[test]
fn the_doppelganger_steals_a_unter_and_the_spitzen_stay() {
    let hands = [
        "CJ HJ DJ CA CT CK C9 D7 H7 H9",
        "SJ SA ST SK SQ S9 S8 DA DT DK",
        "HA HT HK HQ H8 S7 DQ D9 CQ C8",
    ];
    let (content, mut run) = run_with(&[("HJ", DG)], hands, SKAT);
    run.apply_step(&content, RunAction::Declare(hand_clubs()))
        .expect("declare");
    let matadors = run.game().expect("game").game.matadors();
    let events = run
        .apply_step(&content, RunAction::Play(id_of("HJ")))
        .expect("lead");
    let (from, stolen, given) = swapped(&events).expect("swapped");
    assert_eq!((from, stolen.kind), (Seat::DEFENDERS[0], kind("SJ")));
    assert_eq!(given.kind, kind("HJ"));
    assert_eq!(run.game().expect("game").game.matadors(), matadors);
}

#[test]
fn without_a_match_the_doppelganger_plays_as_itself() {
    let hands = [
        "CJ SJ HJ DJ CA CT CK C9 D7 H7",
        "HA HT HK HQ H9 H8 SA ST SK SQ",
        "S9 S8 DA DT DK DQ D9 CQ C8 D8",
    ];
    let (content, mut run) = run_with(&[("H7", DG)], hands, "S7 C7");
    run.apply_step(&content, RunAction::Declare(hand_clubs()))
        .expect("declare");
    let events = run
        .apply_step(&content, RunAction::Play(id_of("H7")))
        .expect("lead");
    assert!(swapped(&events).is_none());
    assert!(
        events.iter().any(
            |e| matches!(e, RunEvent::DoppelgangerUnmatched { card } if card.id == id_of("H7"))
        )
    );
    let g = &run.game().expect("game").game;
    assert_eq!(g.current_trick()[0].id, id_of("H7"));
}

#[test]
fn the_doppelganger_works_in_null_and_not_in_a_ramsch() {
    let (content, mut run) = run_with(&[("D7", DG)], HANDS, SKAT);
    run.apply_step(
        &content,
        RunAction::Declare(Declaration::Null {
            risk: NullRisk::Hand,
        }),
    )
    .expect("declare");
    let events = run
        .apply_step(&content, RunAction::Play(id_of("D7")))
        .expect("lead");
    let (_, stolen, _) = swapped(&events).expect("swapped in Null");
    assert_eq!(stolen.kind, kind("S7"));
    let (content, mut run) = run_with(&[("D7", DG)], HANDS, SKAT);
    let active = run.game.as_mut().expect("game");
    active.game.declare(Declaration::Ramsch).expect("ramsch");
    active.rules.ramsch = true;
    let events = run
        .apply_step(&content, RunAction::Play(id_of("D7")))
        .expect("lead");
    assert!(swapped(&events).is_none());
    assert!(
        !events
            .iter()
            .any(|e| matches!(e, RunEvent::DoppelgangerUnmatched { .. }))
    );
}

#[test]
fn a_game_with_a_doppelganger_plays_to_its_end_and_counts_every_card() {
    for seed in 0..6 {
        let content = content();
        let (mut run, _) = Run::new(&content, seed).expect("start");
        with_mod(&content, &mut run, "C9", DG);
        deal_with(&mut run, &content, HANDS, SKAT, Seat::DECLARER);
        run.apply_step(&content, RunAction::Declare(hand_clubs()))
            .expect("declare");
        let (events, breakdown) = bot_to_end(&mut run, &content);
        assert_eq!(
            breakdown.outcome.declarer_card_points + breakdown.outcome.defender_card_points,
            120,
            "{seed}"
        );
        assert_eq!(
            events
                .iter()
                .filter(|e| matches!(e, RunEvent::Game(Event::TrickWon { .. })))
                .count(),
            10
        );
    }
    // No cap on special cards since 05.11.2026: several Doppelgänger.
    let content = content();
    let mut edition = Edition::standard();
    let dg = CardModId::new(DG);
    edition.add_mod(&content, 3, &dg, 1).expect("one");
    edition.add_mod(&content, 4, &dg, 1).expect("two");
}

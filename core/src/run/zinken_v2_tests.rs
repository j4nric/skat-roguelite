//! Zinken v2 (DESIGN 7.2, user decisions of 03.11.2026): Blei, Goldkante,
//! Fadenkreuz, Schneiderkreide, the new Winkel and the Doppelgänger (its
//! mirror since 07.11.2026). One
//! test per rule edge; the Untot rework is in `undead_tests.rs`.

use super::bot::RunBot;
use super::content_tests::content;
use super::player::PlayerGameEvent;
use super::wave3_tests::{
    bot_to_end, card_in, deal_with, defender_plays, hand_clubs, id_of, with_mod,
};
use super::*;
use crate::cards::Rank;
use crate::content::{Lever, resolve_edition};
use crate::deck::{AUGEN, DeckError, MODS_VERSION, Owner, WINKEL_TO_AUGEN};
use crate::game::GameError;
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

/// The mirror of a Doppelgänger in `events`: the card as played, the card
/// it copied and the copy in the trick.
fn mirrored(events: &[RunEvent]) -> Vec<(DealtCard, DealtCard, DealtCard)> {
    events
        .iter()
        .filter_map(|e| match e {
            RunEvent::Game(Event::DoppelgangerMirrored {
                card, copied, copy, ..
            }) => Some((*card, *copied, *copy)),
            _ => None,
        })
        .collect()
}

fn won(events: &[RunEvent]) -> Option<(Seat, u32)> {
    events.iter().find_map(|e| match e {
        RunEvent::Game(Event::TrickWon { trick, card_points }) => {
            Some((trick.winner, *card_points))
        }
        _ => None,
    })
}

/// The left defender leads the first trick, the player plays last.
fn declared_last(card_mods: &[(&str, &str)], declaration: Declaration) -> (Content, Run) {
    let content = content();
    let (mut run, _) = Run::new(&content, 11).expect("start");
    for (card, card_mod) in card_mods {
        with_mod(&content, &mut run, card, card_mod);
    }
    deal_with(&mut run, &content, HANDS, SKAT, Seat::DEFENDERS[0]);
    run.apply_step(&content, RunAction::Declare(declaration))
        .expect("declare");
    (content, run)
}

#[test]
fn the_doppelganger_copies_the_highest_card_as_the_last_card_and_wins() {
    let (content, mut run) = declared_last(&[("H7", DG)], hand_clubs());
    let h7 = card_in(run.game().expect("game").game.hand(Seat::DECLARER), "H7");
    assert!(run.can_mirror(&content, h7));
    defender_plays(&mut run, &content, Seat::DEFENDERS[0], "HA");
    defender_plays(&mut run, &content, Seat::DEFENDERS[1], "S9");
    assert!(run.mirrors_now(&content, h7));
    let events = run
        .apply_step(&content, RunAction::Play(id_of("H7")))
        .expect("last card");
    let [(card, copied, copy)] = mirrored(&events)[..] else {
        panic!("one mirror: {events:?}");
    };
    assert_eq!(card, h7);
    assert_eq!(copied.kind, kind("HA"));
    assert_eq!(copy.kind, kind("HA"), "a copy of the Daus");
    assert_eq!(
        (copy.id, copy.owner),
        (h7.id, Owner::Player),
        "its identity"
    );
    assert_eq!(copy.card_points, 11);
    // It lies in the trick as the copy and wins the tie against the Daus:
    // 11 + 0 + 11 Augen instead of 11 + 0 + 0.
    assert!(events.iter().any(|e| matches!(
        e,
        RunEvent::Game(Event::CardPlayed { seat, card }) if *seat == Seat::DECLARER && *card == copy
    )));
    assert_eq!(won(&events), Some((Seat::DECLARER, 22)));
    let g = &run.game().expect("game").game;
    assert_eq!(g.trick_lead(), Seat::DECLARER);
    assert_eq!(g.trick_card_points()[Seat::DECLARER.index()], 22);
    assert_eq!(g.current(h7), copy, "the deck holds the copy");
    // The player's view of the event.
    let seen = events.into_iter().map(RunEvent::for_player).find(|e| {
        matches!(
            e,
            RunEvent::Game(PlayerGameEvent::DoppelgangerMirrored { .. })
        )
    });
    assert!(seen.is_some());
}

#[test]
fn the_doppelganger_also_copies_a_trump_that_takes_the_trick() {
    // The right defender trumps the Daus with the C8: the Sieben follows
    // hearts, copies the C8 and wins with it (no Augen of its own).
    let (content, mut run) = declared_last(&[("H7", DG)], hand_clubs());
    defender_plays(&mut run, &content, Seat::DEFENDERS[0], "HA");
    defender_plays(&mut run, &content, Seat::DEFENDERS[1], "C8");
    let events = run
        .apply_step(&content, RunAction::Play(id_of("H7")))
        .expect("last card");
    let [(_, copied, copy)] = mirrored(&events)[..] else {
        panic!("one mirror");
    };
    assert_eq!((copied.kind, copy.kind), (kind("C8"), kind("C8")));
    assert_eq!(won(&events), Some((Seat::DECLARER, 11)));
}

#[test]
fn the_doppelganger_plays_as_itself_when_it_is_not_the_last_card() {
    // Led: the player leads the Sieben, the defenders answer.
    let (content, mut run) = declared(&[("H7", DG)]);
    let events = trick(&mut run, &content, "H7", "HA", "S9");
    assert!(mirrored(&events).is_empty());
    assert_eq!(won(&events), Some((Seat::DEFENDERS[0], 11)));
    // Second: the right defender leads, the player follows.
    let content = super::content_tests::content();
    let (mut run, _) = Run::new(&content, 11).expect("start");
    with_mod(&content, &mut run, "D7", DG);
    deal_with(&mut run, &content, HANDS, SKAT, Seat::DEFENDERS[1]);
    run.apply_step(&content, RunAction::Declare(hand_clubs()))
        .expect("declare");
    defender_plays(&mut run, &content, Seat::DEFENDERS[1], "D9");
    let d7 = card_in(run.game().expect("game").game.hand(Seat::DECLARER), "D7");
    assert!(run.can_mirror(&content, d7));
    assert!(!run.mirrors_now(&content, d7), "second, not last");
    let events = run
        .apply_step(&content, RunAction::Play(id_of("D7")))
        .expect("second");
    assert!(mirrored(&events).is_empty());
    assert_eq!(
        run.game().expect("game").game.current_trick()[1].kind,
        kind("D7"),
        "as its printed card"
    );
}

#[test]
fn the_doppelganger_mirrors_once_per_game() {
    // Two Doppelgänger. The left defender holds trumps to win trick 2.
    let hands = [
        "CJ SJ HJ DJ CA CT CK C9 D7 H7",
        "HA HT HK HQ H9 H8 SA ST CQ C8",
        "S9 S8 S7 SK SQ DA DT DK DQ D9",
    ];
    let content = content();
    let (mut run, _) = Run::new(&content, 11).expect("start");
    with_mod(&content, &mut run, "H7", DG);
    with_mod(&content, &mut run, "D7", DG);
    deal_with(&mut run, &content, hands, SKAT, Seat::DEFENDERS[0]);
    run.apply_step(&content, RunAction::Declare(hand_clubs()))
        .expect("declare");
    defender_plays(&mut run, &content, Seat::DEFENDERS[0], "HA");
    defender_plays(&mut run, &content, Seat::DEFENDERS[1], "S9");
    let events = run
        .apply_step(&content, RunAction::Play(id_of("H7")))
        .expect("mirror");
    assert_eq!(mirrored(&events).len(), 1);
    let d7 = card_in(run.game().expect("game").game.hand(Seat::DECLARER), "D7");
    assert!(!run.can_mirror(&content, d7), "used in this game");
    // Trick 2: the left defender takes the C9 with the CQ.
    let events = trick(&mut run, &content, "C9", "CQ", "S8");
    assert_eq!(won(&events).map(|w| w.0), Some(Seat::DEFENDERS[0]));
    // Trick 3: the player is last again, void in spades: the second
    // Doppelgänger plays as itself and the Daus takes the trick.
    defender_plays(&mut run, &content, Seat::DEFENDERS[0], "SA");
    defender_plays(&mut run, &content, Seat::DEFENDERS[1], "S7");
    let events = run
        .apply_step(&content, RunAction::Play(id_of("D7")))
        .expect("as itself");
    assert!(mirrored(&events).is_empty());
    assert_eq!(won(&events), Some((Seat::DEFENDERS[0], 11)));
}

#[test]
fn the_doppelganger_must_be_a_legal_play_as_printed() {
    // Hearts led: the player holds the H7 and must follow with it, so the
    // D7 must not go, not even to mirror.
    let (content, mut run) = declared_last(&[("D7", DG)], hand_clubs());
    defender_plays(&mut run, &content, Seat::DEFENDERS[0], "HA");
    defender_plays(&mut run, &content, Seat::DEFENDERS[1], "S9");
    let d7 = card_in(run.game().expect("game").game.hand(Seat::DECLARER), "D7");
    assert!(!run.mirrors_now(&content, d7));
    assert!(matches!(
        run.apply_step(&content, RunAction::Play(id_of("D7"))),
        Err(RunError::Game(GameError::MustFollowSuit { .. }))
    ));
}

#[test]
fn the_doppelganger_plays_as_itself_in_null_and_in_a_ramsch() {
    let null = Declaration::Null {
        risk: NullRisk::Hand,
    };
    let (content, mut run) = declared_last(&[("H7", DG)], null);
    let h7 = card_in(run.game().expect("game").game.hand(Seat::DECLARER), "H7");
    assert!(!run.can_mirror(&content, h7));
    defender_plays(&mut run, &content, Seat::DEFENDERS[0], "HA");
    defender_plays(&mut run, &content, Seat::DEFENDERS[1], "S9");
    let events = run
        .apply_step(&content, RunAction::Play(id_of("H7")))
        .expect("as itself");
    assert!(mirrored(&events).is_empty());
    assert_eq!(won(&events).map(|w| w.0), Some(Seat::DEFENDERS[0]));
    // Ramsch.
    let content = super::content_tests::content();
    let (mut run, _) = Run::new(&content, 11).expect("start");
    with_mod(&content, &mut run, "H7", DG);
    deal_with(&mut run, &content, HANDS, SKAT, Seat::DEFENDERS[0]);
    let active = run.game.as_mut().expect("game");
    active.game.declare(Declaration::Ramsch).expect("ramsch");
    active.rules.ramsch = true;
    let h7 = card_in(run.game().expect("game").game.hand(Seat::DECLARER), "H7");
    assert!(!run.can_mirror(&content, h7));
    defender_plays(&mut run, &content, Seat::DEFENDERS[0], "HA");
    defender_plays(&mut run, &content, Seat::DEFENDERS[1], "S9");
    let events = run
        .apply_step(&content, RunAction::Play(id_of("H7")))
        .expect("as itself");
    assert!(mirrored(&events).is_empty());
}

#[test]
fn game_play_mirrored_needs_the_last_card_of_a_trick() {
    let c = |s: &str| -> Card { s.parse().expect("card") };
    let parse = |s: &str| crate::cards::parse_cards(s).expect("hand");
    let mut g = Game::with_cards(
        [parse(HANDS[0]), parse(HANDS[1]), parse(HANDS[2])],
        [c("D8"), c("C7")],
        Seat::DEFENDERS[0],
    )
    .expect("deal");
    g.declare(hand_clubs()).expect("declare");
    g.play(Seat::DEFENDERS[0], c("HK")).expect("lead");
    assert_eq!(
        g.play_mirrored(Seat::DEFENDERS[1], c("S9")),
        Err(GameError::NotLastCard { card: c("S9") })
    );
    g.play(Seat::DEFENDERS[1], c("DA")).expect("slough");
    assert!(matches!(
        g.play_mirrored(Seat::DECLARER, c("D7")),
        Err(GameError::MustFollowSuit { .. })
    ));
    let events = g.play_mirrored(Seat::DECLARER, c("H7")).expect("mirror");
    // A plain card has no identity of its own: the copy is the card.
    assert!(matches!(
        events[..],
        [
            Event::DoppelgangerMirrored { .. },
            Event::CardPlayed { .. },
            Event::TrickWon { .. }
        ]
    ));
    let trick = g.tricks()[0];
    assert_eq!(trick.winner, Seat::DECLARER);
    assert_eq!(trick.card_points(), 4 + 11 + 4);
}

#[test]
fn the_bot_mirrors_a_rich_trick_and_keeps_the_doppelganger_otherwise() {
    // Void in spades: the D7 may mirror the SA (11 + 0 + 11 Augen).
    let (content, mut run) = declared_last(&[("D7", DG)], hand_clubs());
    defender_plays(&mut run, &content, Seat::DEFENDERS[0], "SA");
    defender_plays(&mut run, &content, Seat::DEFENDERS[1], "S9");
    let card = RunBot::mirror_card(&run, &content).expect("mirror");
    assert_eq!(card.id, id_of("D7"));
    assert_eq!(
        RunBot.next_action(&run, &content),
        Some(RunAction::Play(id_of("D7")))
    );
    // A poor trick (SQ, S9: 3 + 0 + 3): the bot keeps it, the search
    // never sees it while another card is legal.
    let (content, mut run) = declared_last(&[("D7", DG)], hand_clubs());
    defender_plays(&mut run, &content, Seat::DEFENDERS[0], "SQ");
    defender_plays(&mut run, &content, Seat::DEFENDERS[1], "S9");
    assert_eq!(RunBot::mirror_card(&run, &content), None);
    assert_ne!(
        RunBot.next_action(&run, &content),
        Some(RunAction::Play(id_of("D7")))
    );
    let mut legal = run.game().expect("game").game.legal_moves(Seat::DECLARER);
    RunBot::keep_for_mirror(&run, &content, &mut legal);
    assert!(!legal.iter().any(|c| c.id == id_of("D7")));
    assert!(!legal.is_empty());
}

#[test]
fn the_hand_view_marks_a_doppelganger_that_may_still_mirror() {
    let (content, run) = declared_last(&[("D7", DG)], hand_clubs());
    let view = run.view(&content);
    let json = serde_json::to_string(&view).expect("json");
    assert_eq!(json.matches("\"can_mirror\":true").count(), 1, "{json}");
}

#[test]
fn a_save_from_before_the_mirror_still_loads() {
    let (_, run) = declared(&[("H7", DG)]);
    let mut json: serde_json::Value = serde_json::to_value(&run).expect("save");
    // The generator of the old steal and a logged swap.
    let game = json
        .pointer_mut("/game")
        .and_then(serde_json::Value::as_object_mut)
        .expect("active game");
    game.insert("doppel_rng".into(), serde_json::json!({ "state": 7 }));
    let swap = Event::DoppelgangerSwapped {
        seat: Seat::DECLARER,
        card: dealt("C9"),
        from: Seat::DEFENDERS[0],
        stolen: dealt("H9"),
        given: dealt("C9"),
    };
    json.pointer_mut("/game/game/log")
        .and_then(serde_json::Value::as_array_mut)
        .expect("log")
        .push(serde_json::to_value(&swap).expect("event"));
    let loaded: Run = serde_json::from_value(json).expect("old save");
    assert!(loaded.game().is_some());
}

#[test]
fn a_game_with_a_doppelganger_plays_to_its_end_and_counts_every_card() {
    let mut mirrors = 0;
    for seed in 0..6 {
        for forehand in [Seat::DECLARER, Seat::DEFENDERS[0], Seat::DEFENDERS[1]] {
            let content = content();
            let (mut run, _) = Run::new(&content, seed).expect("start");
            with_mod(&content, &mut run, "C9", DG);
            deal_with(&mut run, &content, HANDS, SKAT, forehand);
            run.apply_step(&content, RunAction::Declare(hand_clubs()))
                .expect("declare");
            let (events, breakdown) = bot_to_end(&mut run, &content);
            // The copy counts the Augen of the card it copied instead of
            // its own.
            let extra: i64 = mirrored(&events)
                .iter()
                .map(|(card, _, copy)| copy.signed_points() - card.signed_points())
                .sum();
            mirrors += mirrored(&events).len();
            let total =
                breakdown.outcome.declarer_card_points + breakdown.outcome.defender_card_points;
            assert_eq!(i64::from(total), 120 + extra, "{seed}");
            assert_eq!(
                events
                    .iter()
                    .filter(|e| matches!(e, RunEvent::Game(Event::TrickWon { .. })))
                    .count(),
                10
            );
        }
    }
    assert!(mirrors > 0, "the bot mirrors in some of these games");
    // No cap on special cards since 05.11.2026: several Doppelgänger.
    let content = content();
    let mut edition = Edition::standard();
    let dg = CardModId::new(DG);
    edition.add_mod(&content, 3, &dg, 1).expect("one");
    edition.add_mod(&content, 4, &dg, 1).expect("two");
}

#[test]
fn the_value_bound_counts_the_augen_a_mirror_can_add() {
    // "Elle": +1 Grundwert per 5 Augen above 90. Without a Doppelgänger
    // the player takes at most 120 Augen, with one a copied Daus more.
    let (content, mut run) = run_with(&[], HANDS, SKAT);
    run.relics.clear();
    run.evening_relics.clear();
    run.relics.push(RelicId::new("yardstick"));
    let plain = run.max_game_value(&content);
    let (content, mut mirrored) = run_with(&[("H7", DG)], HANDS, SKAT);
    mirrored.relics.clear();
    mirrored.evening_relics.clear();
    mirrored.relics.push(RelicId::new("yardstick"));
    assert!(mirrored.max_game_value(&content) > plain);
}

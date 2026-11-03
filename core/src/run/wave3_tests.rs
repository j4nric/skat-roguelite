//! M8 wave 3 (DESIGN 7.2, 8.1): the card modification "Wandelkarte" (the
//! Glasdaus and Heimweh are gone with effects v2) and the legendary relics
//! "Skatgott" and "Ehrenvorsitz". One test per item in a constructed situation, plus the
//! edge cases of their rules.

use super::bot::RunBot;
use super::content_tests::{content, relic_run};
use super::*;
use crate::cards::Suit;
use crate::content::{CardModDef, Factor, Rarity, RarityWeights};
use crate::rules::{self, NullRisk, RiskLevel};

const WILD: &str = "wild_card";

/// Instance id of a printed card in the standard deck (D7 = 0 … CA = 31).
pub(super) fn id_of(notation: &str) -> u32 {
    let card: Card = notation.parse().expect("card");
    let suit = Suit::ALL
        .iter()
        .position(|&s| s == card.suit)
        .expect("suit");
    let rank = [
        Rank::Seven,
        Rank::Eight,
        Rank::Nine,
        Rank::Ten,
        Rank::Jack,
        Rank::Queen,
        Rank::King,
        Rank::Ace,
    ]
    .iter()
    .position(|&r| r == card.rank)
    .expect("rank");
    u32::try_from(suit * 8 + rank).expect("small")
}

pub(super) fn with_mod(content: &Content, run: &mut Run, card: &str, card_mod: &str) {
    let id = CardModId::new(card_mod);
    let value = content.card_mod(&id).map_or(1, CardModDef::default_value);
    run.attach_mod(content, id_of(card), &id, value, &mut Vec::new())
        .expect("attach");
}

/// Several Untot or Wandelkarten on `cards`, as a save from before the cap
/// of the special cards (DESIGN 7.2, 03.11.2026) may hold them: the game
/// rules still cope with them.
pub(super) fn with_old_specials(run: &mut Run, cards: &[&str], card_mod: &str) {
    for card in cards {
        run.edition
            .force_mod(id_of(card), &CardModId::new(card_mod));
    }
}

/// Replace the current game by a constructed deal with `forehand`, with
/// every content effect of a normal deal, and a target the series will not
/// reach.
pub(super) fn deal_with(
    run: &mut Run,
    content: &Content,
    hands: [&str; 3],
    skat: &str,
    forehand: Seat,
) -> Vec<RunEvent> {
    run.series.target = 100_000;
    run.series.game_limit = 100;
    let kinds = |s: &str| -> Vec<CardKind> {
        s.split_whitespace()
            .map(|n| n.parse().expect("notation"))
            .collect()
    };
    let hands = [kinds(hands[0]), kinds(hands[1]), kinds(hands[2])];
    let skat = kinds(skat);
    let mut events = Vec::new();
    run.deal_fixed(
        content,
        [&hands[0], &hands[1], &hands[2]],
        [skat[0], skat[1]],
        forehand,
        &mut events,
    );
    events
}

pub(super) fn card_in(cards: &[DealtCard], notation: &str) -> DealtCard {
    let kind: CardKind = notation.parse().expect("notation");
    *cards.iter().find(|c| c.kind == kind).expect("card held")
}

/// A defender plays `notation` (fires the hooks like the AI would).
pub(super) fn defender_plays(
    run: &mut Run,
    content: &Content,
    seat: Seat,
    notation: &str,
) -> Vec<RunEvent> {
    let card = card_in(run.game().expect("game").game.hand(seat), notation);
    let mut events = Vec::new();
    run.play_card(content, seat, card, &mut events)
        .expect("legal");
    run.settle(content, &mut events).expect("settle");
    events
}

/// Let the bot play the game to its end.
pub(super) fn bot_to_end(run: &mut Run, content: &Content) -> (Vec<RunEvent>, ScoreBreakdown) {
    let mut events = Vec::new();
    loop {
        if let Some(b) = events.iter().find_map(|e| match e {
            RunEvent::GameScored { breakdown, .. } => Some(breakdown.clone()),
            _ => None,
        }) {
            return (events, b);
        }
        if run.ai_to_move().is_some() {
            events.extend(run.advance_ai_all(content).expect("ai"));
            continue;
        }
        let action = RunBot.next_action(run, content).expect("game continues");
        events.extend(run.apply(content, action).expect("legal"));
    }
}

pub(super) fn hand_clubs() -> Declaration {
    Declaration::suit(Suit::Clubs, RiskLevel::Hand)
}

fn dealt(notation: &str, wild: bool) -> DealtCard {
    DealtCard {
        wild,
        ..DealtCard::plain(
            id_of(notation),
            CardKind::Normal(notation.parse().expect("card")),
        )
    }
}

// ---------------------------------------------------------------- content

#[test]
fn wave_three_items_load_with_their_tier() {
    let content = content();
    assert!(content.card_mod(&CardModId::new(WILD)).is_some());
    for id in ["skat_god", "honorary_chair"] {
        let def = content.relic(&RelicId::new(id)).expect("relic");
        assert_eq!(def.rarity, Rarity::Legendary, "{id}");
    }
    // A run.ron without the legendary weight reads 0.3.
    let weights: RarityWeights = ron::from_str("(common: 6, uncommon: 3, rare: 1)").expect("ron");
    assert_eq!(weights.legendary, Factor::from_permille(300));
    assert_eq!(weights.percent_weight(Rarity::Legendary, 100), 30);
    assert_eq!(weights.percent_weight(Rarity::Rare, 100), 100);
    assert_eq!(weights.weight(Rarity::Legendary), 0);
}

#[test]
fn legendary_relics_are_offered_far_less_often_than_rare_ones() {
    let content = content();
    let (run, _) = Run::new(&content, 3).expect("start");
    let pool = run.relic_pool(&content, false);
    let weight = |id: &str| {
        pool.iter()
            .find(|(r, _)| r.as_str() == id)
            .map(|&(_, w)| w)
            .expect("in pool")
    };
    assert_eq!(weight("skat_god"), 30);
    assert_eq!(weight("honorary_chair"), 30);
    assert_eq!(weight("tour"), 100, "a rare one");
    // Over many offers a legendary relic shows up, but rarely.
    let mut seen = 0;
    let mut offers = 0;
    let mut probe = run.clone();
    for _ in 0..3000 {
        let shop = probe.draw_offer(&content, 0);
        offers += 1;
        seen += shop
            .relics
            .iter()
            .flatten()
            .filter(|id| id.as_str() == "skat_god")
            .count();
    }
    let percent = seen * 100 / offers;
    assert!(seen > 0 && percent < 2, "{seen} of {offers} offers");
}

// ---------------------------------------------------------- Wandelkarte

#[test]
fn a_wild_card_settles_into_the_longest_suit_without_unter() {
    // Spades and diamonds two each, hearts only the H7: the Wandelkarte
    // (HA) itself does not count. Spades come first in a tie.
    let hand = [
        dealt("HA", true),
        dealt("S7", false),
        dealt("S8", false),
        dealt("D7", false),
        dealt("D8", false),
        dealt("H7", false),
    ];
    assert_eq!(rules::settled_wild_suit(&hand), Suit::Spades);
    // The Unter do not count: three clubs Unter lose to two spades.
    let hand = [
        dealt("CJ", false),
        dealt("SJ", false),
        dealt("HJ", false),
        dealt("C7", false),
        dealt("S7", false),
        dealt("S8", false),
        dealt("DA", true),
    ];
    assert_eq!(rules::settled_wild_suit(&hand), Suit::Spades);
    // Ties: Eichel, Grün, Rot, Schellen.
    let tie = |a: &str, b: &str| rules::settled_wild_suit(&[dealt(a, false), dealt(b, false)]);
    assert_eq!(tie("D7", "C7"), Suit::Clubs);
    assert_eq!(tie("H7", "S7"), Suit::Spades);
    assert_eq!(tie("D7", "H7"), Suit::Hearts);
    // Nothing to count (only Unter and Wandelkarten): Eichel.
    assert_eq!(
        rules::settled_wild_suit(&[dealt("DJ", false), dealt("D7", true)]),
        Suit::Clubs
    );
    // A longer suit wins over the tie order.
    let hand = [dealt("D7", false), dealt("D8", false), dealt("C7", false)];
    assert_eq!(rules::settled_wild_suit(&hand), Suit::Diamonds);
}

/// The settled card of instance `id` in the player's hand.
fn held(run: &Run, id: u32) -> DealtCard {
    *run.game()
        .expect("game")
        .game
        .hand(Seat::DECLARER)
        .iter()
        .find(|c| c.id == id)
        .expect("held")
}

fn settled(events: &[RunEvent]) -> Vec<(DealtCard, DealtCard)> {
    events
        .iter()
        .filter_map(|e| match e {
            RunEvent::Game(Event::WildCardSettled { old, new }) => Some((*old, *new)),
            _ => None,
        })
        .collect()
}

fn kind(notation: &str) -> CardKind {
    CardKind::Normal(notation.parse().expect("card"))
}

#[test]
fn a_hand_game_settles_the_wild_card_at_the_declaration() {
    let content = content();
    let (mut run, _) = Run::new(&content, 11).expect("start");
    // Seat 1 holds a Wandelkarte as well: there it is the printed card.
    with_old_specials(&mut run, &["HA", "HT"], WILD);
    deal_with(
        &mut run,
        &content,
        [
            "CJ SJ HA CA CT CK SA ST D7 D8",
            "HJ DJ HT HK HQ H9 H8 H7 DA DT",
            "SK SQ S9 S8 S7 DK DQ D9 CQ C9",
        ],
        "C8 C7",
        Seat::DECLARER,
    );
    let game = &run.game().expect("game").game;
    assert!(held(&run, id_of("HA")).wild);
    assert!(!card_in(game.hand(Seat::DEFENDERS[0]), "HT").wild);
    // Before the game starts it is the printed card, shown as open.
    let view = run.view(&content).game.expect("game view");
    let wild = view.hand.iter().find(|c| c.card.id == id_of("HA"));
    let wild = wild.expect("held");
    assert!(wild.unsettled);
    assert_eq!(wild.named, None);
    assert_eq!(wild.card.kind, kind("HA"));
    assert!(
        view.hand
            .iter()
            .filter(|c| c.card.id != id_of("HA"))
            .all(|c| !c.unsettled)
    );
    let events = run
        .apply_step(&content, RunAction::Declare(hand_clubs()))
        .expect("declare");
    // Clubs three (CA CT CK), spades two, diamonds two: a Kreuz-Daus. The
    // event follows the declaration.
    let declared = events
        .iter()
        .position(|e| matches!(e, RunEvent::Game(Event::Declared { .. })));
    let at = events
        .iter()
        .position(|e| matches!(e, RunEvent::Game(Event::WildCardSettled { .. })));
    assert!(declared < at);
    let [(old, new)] = settled(&events)[..] else {
        panic!("one settled card");
    };
    assert_eq!(
        (old.id, old.kind, old.wild),
        (id_of("HA"), kind("HA"), true)
    );
    assert_eq!(
        (new.id, new.kind, new.wild),
        (id_of("HA"), kind("CA"), false)
    );
    assert_eq!(new.card_points, old.card_points);
    assert_eq!(held(&run, id_of("HA")), new);
    assert!(rules::is_trump(GameType::Suit(Suit::Clubs), new));
    let active = run.game().expect("game");
    assert_eq!(active.named, [(id_of("HA"), Suit::Hearts)]);
    // The Spitzen stay as declared: with CJ SJ, without the HJ, two.
    let matadors = active.game.matadors().expect("declared");
    assert_eq!((matadors.with, matadors.count), (true, 2));
    // The view: the settled suit in `kind`, the printed one in `named`.
    let view = run.view(&content).game.expect("game view");
    let wild = view.hand.iter().find(|c| c.card.id == id_of("HA"));
    let wild = wild.expect("held");
    assert!(!wild.unsettled);
    assert_eq!(wild.named, Some(Suit::Hearts));
    assert_eq!(wild.card.kind, kind("CA"));
    assert_eq!(
        view.wild_settled,
        [view::SettledWild {
            card: id_of("HA"),
            kind: kind("CA"),
            printed: Suit::Hearts,
        }]
    );
    // The player sees the event too.
    let seen = player::player_events(events);
    assert!(seen.iter().any(|e| matches!(
        e,
        RunEvent::Game(player::PlayerGameEvent::WildCardSettled { new, .. }) if new.kind == kind("CA")
    )));
    // It plays as the Kreuz-Daus and stays it on the table.
    let events = run
        .apply_step(&content, RunAction::Play(id_of("HA")))
        .expect("plays");
    let played = events.iter().find_map(|e| match e {
        RunEvent::Game(Event::CardPlayed { card, .. }) => Some(*card),
        _ => None,
    });
    assert_eq!(played, Some(new));
}

/// Twelve cards for the player after picking up the skat (ST DT), the
/// Wandelkarte on HA.
fn discard_run(content: &Content) -> Run {
    let (mut run, _) = Run::new(content, 11).expect("start");
    with_mod(content, &mut run, "HA", WILD);
    deal_with(
        &mut run,
        content,
        [
            "CJ HA S7 S8 S9 D7 D8 D9 C7 C8",
            "C9 CT CQ CK CA SJ SQ SK SA HJ",
            "H7 H8 H9 HT HQ HK DJ DQ DK DA",
        ],
        "ST DT",
        Seat::DECLARER,
    );
    run.apply_step(content, RunAction::PickUpSkat)
        .expect("pick up");
    run
}

#[test]
fn discarding_the_second_longest_suit_decides_the_wild_suit() {
    let content = content();
    let grand = Declaration::grand(RiskLevel::Normal);
    // While discarding it is the printed card.
    let mut run = discard_run(&content);
    assert_eq!(held(&run, id_of("HA")).kind, kind("HA"));
    // Two spades away: diamonds four, spades two.
    run.apply_step(&content, RunAction::Discard([id_of("S7"), id_of("S8")]))
        .expect("discard");
    assert!(held(&run, id_of("HA")).wild, "not before the declaration");
    let events = run
        .apply_step(&content, RunAction::Declare(grand))
        .expect("declare");
    assert_eq!(settled(&events).len(), 1);
    assert_eq!(held(&run, id_of("HA")).kind, kind("DA"));
    // Two diamonds away instead: spades four, diamonds two.
    let mut run = discard_run(&content);
    run.apply_step(&content, RunAction::Discard([id_of("D7"), id_of("D8")]))
        .expect("discard");
    run.apply_step(&content, RunAction::Declare(grand))
        .expect("declare");
    assert_eq!(held(&run, id_of("HA")).kind, kind("SA"));
    // A Wandelkarte put into the skat stays the printed card.
    let mut run = discard_run(&content);
    run.apply_step(&content, RunAction::Discard([id_of("HA"), id_of("D8")]))
        .expect("discard");
    let events = run
        .apply_step(&content, RunAction::Declare(grand))
        .expect("declare");
    assert!(settled(&events).is_empty());
    let skat = run.game().expect("game").game.skat();
    assert!(
        skat.iter()
            .any(|c| c.id == id_of("HA") && c.kind == kind("HA"))
    );
}

#[test]
fn a_wild_card_settles_in_null_and_a_wild_unter_stays_as_it_is() {
    let content = content();
    let (mut run, _) = Run::new(&content, 11).expect("start");
    with_old_specials(&mut run, &["H8", "HJ"], WILD);
    deal_with(
        &mut run,
        &content,
        [
            "HJ H8 D7 D8 D9 S7 C7 C8 C9 CT",
            "CJ SJ DJ HA HT HK HQ H9 H7 DA",
            "SA ST SK SQ S9 S8 DT DK DQ CA",
        ],
        "CQ CK",
        Seat::DECLARER,
    );
    let null = Declaration::Null {
        risk: NullRisk::Hand,
    };
    let events = run
        .apply_step(&content, RunAction::Declare(null))
        .expect("declare");
    // Clubs four: the Herz-Acht becomes the Kreuz-Acht, next to the C8.
    let settled = settled(&events);
    assert_eq!(settled.len(), 2, "both Wandelkarten settle");
    assert_eq!(held(&run, id_of("H8")).kind, kind("C8"));
    // The Unter keeps its suit; it is settled all the same.
    let unter = held(&run, id_of("HJ"));
    assert_eq!((unter.kind, unter.wild), (kind("HJ"), false));
    let both: Vec<u32> = run
        .game()
        .expect("game")
        .named
        .iter()
        .map(|(id, _)| *id)
        .collect();
    assert!(both.contains(&id_of("H8")) && both.contains(&id_of("HJ")));
}

#[test]
fn a_settled_wild_card_follows_its_new_suit() {
    let content = content();
    let (mut run, _) = Run::new(&content, 11).expect("start");
    with_mod(&content, &mut run, "HA", WILD);
    deal_with(
        &mut run,
        &content,
        [
            "CJ SJ HA S7 S8 S9 ST D7 D8 H7",
            "HJ DJ SA HK HQ H9 H8 HT DA DT",
            "SK SQ CA CT CK CQ C9 C8 C7 DK",
        ],
        "D9 DQ",
        Seat::DEFENDERS[0],
    );
    let grand = Declaration::grand(RiskLevel::Hand);
    run.apply_step(&content, RunAction::Declare(grand))
        .expect("declare");
    // Spades four: the Wandelkarte is the Grün-Daus now.
    let wild = held(&run, id_of("HA"));
    assert_eq!(wild.kind, kind("SA"));
    defender_plays(&mut run, &content, Seat::DEFENDERS[0], "HK");
    defender_plays(&mut run, &content, Seat::DEFENDERS[1], "DK");
    // A heart is led: only the H7 follows, the former Rot-Daus may not.
    let legal = run.game().expect("game").game.legal_moves(Seat::DECLARER);
    assert_eq!(
        legal.iter().map(|c| c.id).collect::<Vec<_>>(),
        [id_of("H7")]
    );
    let copy = run.clone();
    let err = run.apply_step(&content, RunAction::Play(id_of("HA")));
    assert_eq!(err.map_err(|e| e.code()), Err(RunErrorCode::MustFollowSuit));
    assert_eq!(run, copy);
    run.apply_step(&content, RunAction::Play(id_of("H7")))
        .expect("follows");
    // A spade is led: the Wandelkarte must follow as a spade like the rest.
    let next = run.game().expect("game").game.current_seat();
    assert_eq!(next, Some(Seat::DEFENDERS[0]), "the defender took it");
    defender_plays(&mut run, &content, Seat::DEFENDERS[0], "SA");
    defender_plays(&mut run, &content, Seat::DEFENDERS[1], "SK");
    let legal = run.game().expect("game").game.legal_moves(Seat::DECLARER);
    let spades: Vec<u32> = ["HA", "S7", "S8", "S9", "ST"].map(id_of).to_vec();
    let mut ids: Vec<u32> = legal.iter().map(|c| c.id).collect();
    ids.sort_unstable();
    let mut want = spades;
    want.sort_unstable();
    assert_eq!(ids, want);
    // Nothing is left to settle: the rest may be played out.
    assert!(
        !run.game()
            .expect("game")
            .game
            .hand(Seat::DECLARER)
            .iter()
            .any(|c| c.wild)
    );
}

#[test]
fn an_undead_wild_card_rises_as_its_settled_card() {
    // One card that is both (a deck from before effects v2 keeps both
    // special cards, `migrate_legacy_mods`).
    let mut content = content();
    let def = content
        .card_mods
        .get_mut(&CardModId::new(WILD))
        .expect("wild");
    def.passive.push(Effect::Undead);
    let (mut run, _) = Run::new(&content, 11).expect("start");
    with_mod(&content, &mut run, "D7", WILD);
    deal_with(
        &mut run,
        &content,
        [
            "CJ SJ HJ DJ CA CT CK C9 D7 H7",
            "HA HT HK HQ H9 H8 SA ST SK SQ",
            "S9 S8 S7 DA DT DK DQ D9 CQ C8",
        ],
        "D8 C7",
        Seat::DECLARER,
    );
    run.apply_step(&content, RunAction::Declare(hand_clubs()))
        .expect("declare");
    // Clubs four: the Schellen-Sieben is a Kreuz-Sieben, a trump.
    let wild = held(&run, id_of("D7"));
    assert_eq!(wild.kind, kind("C7"));
    assert!(run.can_rise(&content, wild));
    run.apply_step(&content, RunAction::Play(id_of("D7")))
        .expect("lead");
    defender_plays(&mut run, &content, Seat::DEFENDERS[0], "H8");
    let events = defender_plays(&mut run, &content, Seat::DEFENDERS[1], "C8");
    let rose = events.iter().find_map(|e| match e {
        RunEvent::Game(Event::CardRose { card, .. }) => Some(*card),
        _ => None,
    });
    assert_eq!(rose, Some(wild), "it rises as the Kreuz-Sieben");
    assert_eq!(held(&run, id_of("D7")), wild, "and stays it");
    assert!(!wild.wild);
}

#[test]
fn the_bot_plays_a_game_with_a_settled_wild_card() {
    let content = content();
    let (mut run, _) = Run::new(&content, 11).expect("start");
    with_mod(&content, &mut run, "H7", WILD);
    deal_with(
        &mut run,
        &content,
        [
            "CJ SJ H7 CA CT CK HA HT D7 D8",
            "HJ DJ SA SK HQ H9 H8 ST DA DT",
            "SQ S9 S8 S7 DK DQ D9 CQ C9 HK",
        ],
        "C8 C7",
        Seat::DEFENDERS[0],
    );
    run.apply_step(&content, RunAction::Declare(hand_clubs()))
        .expect("declare");
    // Clubs three, hearts two (HA HT), diamonds two: the Kreuz-Sieben.
    assert_eq!(held(&run, id_of("H7")).kind, kind("C7"));
    let (events, _) = bot_to_end(&mut run, &content);
    let played = events.iter().find_map(|e| match e {
        RunEvent::Game(Event::CardPlayed { card, .. }) if card.id == id_of("H7") => Some(*card),
        _ => None,
    });
    assert_eq!(played.map(|c| c.kind), Some(kind("C7")));
}

// -------------------------------------------------------------- Glasdaus

// --------------------------------------------------------------- Heimweh

fn hand_of(run: &Run) -> &[DealtCard] {
    run.game().expect("game").game.hand(Seat::DECLARER)
}

// -------------------------------------------------------------- Skatgott

#[test]
fn skat_god_adds_a_level_to_every_won_game_after_a_won_schneider() {
    let content = content();
    let mut run = relic_run(&content, &["skat_god"]);
    let strong = [
        "CJ SJ HJ DJ DA DT DK DQ D9 D7",
        "HA HT HK HQ H9 H8 H7 SA ST SK",
        "SQ S9 S8 S7 CA CT CK CQ C9 C8",
    ];
    deal_with(&mut run, &content, strong, "D8 C7", Seat::DECLARER);
    let diamonds = Declaration::suit(Suit::Diamonds, RiskLevel::Hand);
    run.apply(&content, RunAction::Declare(diamonds))
        .expect("declare");
    let (events, breakdown) = bot_to_end(&mut run, &content);
    assert!(breakdown.won && breakdown.schneider);
    let source = Source::Relic(RelicId::new("skat_god"));
    assert_eq!(
        breakdown.lines_of(&source).count(),
        0,
        "not on its own game"
    );
    assert!(events.contains(&RunEvent::CounterRaised {
        counter: "skat_god".into(),
        value: 1,
    }));
    // The next game, lost: no level.
    let weak = [
        "D7 D8 D9 H7 H8 H9 S7 S8 S9 C7",
        "CJ SJ HJ DJ CA CT CK CQ DA DT",
        "DK DQ HA HT HK HQ SA ST SK SQ",
    ];
    deal_with(&mut run, &content, weak, "C8 C9", Seat::DECLARER);
    run.apply(&content, RunAction::Declare(hand_clubs()))
        .expect("declare");
    let (_, breakdown) = bot_to_end(&mut run, &content);
    assert!(!breakdown.won);
    assert_eq!(breakdown.lines_of(&source).count(), 0, "lost: no level");
    // A won game: +1 level from the one Schneider so far; it is Schneider
    // again, so the counter climbs to 2 for the games after it.
    let line = ScoreLine {
        source: source.clone(),
        lever: Lever::Levels,
        change: Change::Add(1),
    };
    deal_with(&mut run, &content, strong, "D8 C7", Seat::DECLARER);
    run.apply(&content, RunAction::Declare(diamonds))
        .expect("declare");
    let (_, breakdown) = bot_to_end(&mut run, &content);
    assert!(breakdown.won);
    assert!(breakdown.lines_of(&source).any(|l| *l == line));
    assert_eq!(run.counters().get("skat_god"), Some(&2));
    // The value preview (if won) counts the two levels at once.
    let preview = |run: &Run| {
        run.view(&content)
            .game
            .and_then(|g| g.value_preview)
            .expect("preview")
            .levels
    };
    deal_with(&mut run, &content, strong, "D8 C7", Seat::DECLARER);
    run.apply(&content, RunAction::Declare(diamonds))
        .expect("declare");
    let mut without = run.clone();
    without.counters.remove("skat_god");
    assert_eq!(preview(&run), preview(&without) + 2);
}

// ---------------------------------------------------------- Ehrenvorsitz

#[test]
fn honorary_chair_makes_the_player_forehand_in_every_game() {
    let content = content();
    let start = RunStart {
        relics: vec![RelicId::new("honorary_chair")],
        ..RunStart::default()
    };
    let (mut with, _) = Run::new_with(&content, 7, &start).expect("start");
    let (without, _) = Run::new(&content, 7).expect("start");
    // The same cards, only the lead moves.
    assert_eq!(hand_of(&with), hand_of(&without));
    let mut forehands = vec![with.game().expect("game").game.forehand()];
    for _ in 0..2000 {
        let Some(action) = RunBot.next_action(&with, &content) else {
            break;
        };
        // An encounter may swap it away (Alter Bekannter).
        if !with.relics().contains(&RelicId::new("honorary_chair")) {
            break;
        }
        let events = with.apply(&content, action).expect("legal");
        forehands.extend(events.iter().filter_map(|e| match e {
            RunEvent::GameDealt { forehand, .. } => Some(*forehand),
            _ => None,
        }));
    }
    assert!(forehands.len() > 6, "{} deals", forehands.len());
    assert!(forehands.iter().all(|&f| f == Seat::DECLARER));
    // Without it the forehand rotates.
    assert_eq!(
        without.game().expect("game").game.forehand(),
        Seat::DECLARER
    );
}

/// Statistics for the balance report (`docs/balance/2026-11-02-welle3.md`):
/// how often the wave 3 items act in whole runs of the bot (heuristic card
/// play, no Tresen, 400 runs each, the item from the start).
#[test]
#[ignore = "statistics for the balance report, not a check"]
fn wave_three_trigger_statistics() {
    let base = content();
    let with_mod = |card: &str, id: &str| {
        let mut c = base.clone();
        if let Some(def) = c.characters.get_mut(&Character::Declarer) {
            def.deck
                .mods
                .push((card.parse().expect("card"), CardModId::new(id), 1));
        }
        c
    };
    let items: [(&str, Content, Vec<RelicId>); 3] = [
        ("wild_card on HA", with_mod("HA", WILD), Vec::new()),
        ("skat_god", base.clone(), vec![RelicId::new("skat_god")]),
        (
            "honorary_chair",
            base.clone(),
            vec![RelicId::new("honorary_chair")],
        ),
    ];
    for (name, content, relics) in items {
        let (mut games, mut named, mut renamed, mut levels) = (0u32, 0u32, 0u32, 0i64);
        let mut counter_end = 0u32;
        let runs = 400u32;
        for seed in 0..u64::from(runs) {
            let start = RunStart {
                relics: relics.clone(),
                ..RunStart::default()
            };
            let (mut run, _) = Run::new_with(&content, seed, &start).expect("run");
            for _ in 0..5000 {
                let action = match run.phase() {
                    RunPhase::Shop => RunAction::LeaveShop,
                    _ => match RunBot.next_action(&run, &content) {
                        Some(a) => a,
                        None => break,
                    },
                };
                for e in run.apply(&content, action).expect("legal") {
                    match e {
                        RunEvent::GameScored { breakdown, .. } => {
                            games += 1;
                            let source = Source::Relic(RelicId::new("skat_god"));
                            for l in breakdown.lines_of(&source) {
                                if let Change::Add(n) = l.change {
                                    levels += n;
                                }
                            }
                        }
                        RunEvent::Game(Event::WildCardSettled { old, new }) => {
                            named += 1;
                            if old.kind != new.kind {
                                renamed += 1;
                            }
                        }
                        _ => {}
                    }
                }
            }
            counter_end += run.counters().get("skat_god").copied().unwrap_or(0);
        }
        let per = |n: u32| f64::from(n) / f64::from(games.max(1));
        println!(
            "{name}: games {games}; named {:.3}/game ({renamed} into another suit); skat_god levels {:.3}/game, counter at the end {:.2}",
            per(named),
            levels as f64 / f64::from(games.max(1)),
            f64::from(counter_end) / f64::from(runs),
        );
    }
}

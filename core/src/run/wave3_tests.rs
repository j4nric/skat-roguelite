//! M8 wave 3 (DESIGN 7.2, 8.1): the card modification "Wandelkarte" (the
//! Glasdaus and Heimweh are gone with effects v2) and the legendary relics
//! "Skatgott" and "Ehrenvorsitz". One test per item in a constructed situation, plus the
//! edge cases of their rules.

use super::bot::RunBot;
use super::content_tests::{content, relic_run};
use super::*;
use crate::cards::Suit;
use crate::content::{CardModDef, Factor, Rarity, RarityWeights};
use crate::rules::{self, RiskLevel};

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
    let global = run.sources(content, false);
    let resolved = resolve_deck(&run.deck, content, &global);
    let mut pool = resolved.cards.clone();
    let mut take = |notation: &str| -> DealtCard {
        let kind: CardKind = notation.parse().expect("notation");
        let index = pool
            .iter()
            .position(|c| c.kind == kind)
            .expect("card in deck");
        pool.remove(index)
    };
    let mut parse = |s: &str| -> Vec<DealtCard> { s.split_whitespace().map(&mut take).collect() };
    let hands = [parse(hands[0]), parse(hands[1]), parse(hands[2])];
    let skat = parse(skat);
    let game = Game::with_talon(hands, [skat[0], skat[1]], pool, forehand).expect("deal");
    let mut events = Vec::new();
    run.set_up_game(content, game, resolved.point_changes, &mut events);
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
fn a_wild_card_never_obliges_and_may_be_named_to_follow() {
    let clubs = GameType::Suit(Suit::Clubs);
    let lead = Some(dealt("SA", false));
    // Another spade in the hand: the Wandelkarte may only follow as a spade.
    let hand = [dealt("HA", true), dealt("S7", false), dealt("H7", false)];
    assert_eq!(
        rules::wild_suits(clubs, &hand, lead, hand[0]),
        [Suit::Spades]
    );
    assert_eq!(rules::legal_moves(clubs, &hand, lead), [hand[0], hand[1]]);
    // No other spade: every suit, and the other cards are free too.
    let hand = [dealt("HA", true), dealt("H7", false), dealt("D7", false)];
    assert_eq!(rules::wild_suits(clubs, &hand, lead, hand[0]).len(), 4);
    assert_eq!(rules::legal_moves(clubs, &hand, lead).len(), 3);
    // A printed spade that is wild does not oblige its holder.
    let hand = [dealt("SK", true), dealt("H7", false)];
    assert!(rules::may_play(clubs, &hand, lead, hand[1]));
    // A Unter stays a trump whatever it is named: it cannot follow a suit.
    let hand = [dealt("DJ", true), dealt("S7", false)];
    assert!(rules::wild_suits(clubs, &hand, lead, hand[0]).is_empty());
    assert_eq!(rules::legal_moves(clubs, &hand, lead), [hand[1]]);
    let trump_lead = Some(dealt("CA", false));
    assert_eq!(
        rules::wild_suits(clubs, &hand, trump_lead, hand[0]).len(),
        4
    );
    // Grand: only Unter are trump, a Daus cannot be named into one.
    let hand = [dealt("HA", true), dealt("DJ", false)];
    let jack_lead = Some(dealt("CJ", false));
    assert!(rules::wild_suits(GameType::Grand, &hand, jack_lead, hand[0]).is_empty());
    // A card that is not wild has no suits to name.
    assert!(rules::wild_suits(clubs, &hand, lead, hand[1]).is_empty());
}

#[test]
fn the_player_names_the_suit_of_a_wild_card_as_he_plays_it() {
    let content = content();
    let (mut run, _) = Run::new(&content, 11).expect("start");
    with_mod(&content, &mut run, "HA", WILD);
    // Seat 1 holds a Wandelkarte as well: there it is the printed card.
    with_mod(&content, &mut run, "HT", WILD);
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
    assert!(card_in(game.hand(Seat::DECLARER), "HA").wild);
    assert!(!card_in(game.hand(Seat::DEFENDERS[0]), "HT").wild);
    run.apply_step(&content, RunAction::Declare(hand_clubs()))
        .expect("declare");
    let view = run.view(&content).game.expect("game view");
    let wild = view
        .hand
        .iter()
        .find(|c| c.card.id == id_of("HA"))
        .expect("held");
    assert_eq!(wild.wild_suits.len(), 4, "he leads: every suit");
    assert!(
        view.hand
            .iter()
            .filter(|c| c.card.id != id_of("HA"))
            .all(|c| c.wild_suits.is_empty())
    );
    let before = run.game().expect("game").game.matadors();
    // Not a Wandelkarte: rejected, nothing changed.
    let copy = run.clone();
    assert!(
        run.apply_step(
            &content,
            RunAction::PlayAs {
                card: id_of("SA"),
                suit: Suit::Hearts
            }
        )
        .is_err()
    );
    assert_eq!(run, copy);
    let events = run
        .apply_step(
            &content,
            RunAction::PlayAs {
                card: id_of("HA"),
                suit: Suit::Clubs,
            },
        )
        .expect("named");
    assert!(events.iter().any(|e| matches!(
        e,
        RunEvent::WildCardNamed { card, suit: Suit::Clubs } if card.id == id_of("HA")
    )));
    let played = events.iter().find_map(|e| match e {
        RunEvent::Game(Event::CardPlayed { card, .. }) => Some(*card),
        _ => None,
    });
    let played = played.expect("played");
    assert_eq!(played.id, id_of("HA"));
    assert_eq!(played.kind, CardKind::Normal("CA".parse().expect("card")));
    let active = run.game().expect("game");
    assert_eq!(active.named, [(id_of("HA"), Suit::Hearts)]);
    // The Spitzen stay as declared.
    assert_eq!(active.game.matadors(), before);
    // A trump lead: seat 1 must follow with a Unter.
    assert!(rules::is_trump(GameType::Suit(Suit::Clubs), played));
    let view = run.view(&content).game.expect("game view");
    let on_table = view.trick.first().expect("on the table");
    assert_eq!(on_table.card.named, Some(Suit::Hearts));
}

#[test]
fn naming_must_follow_when_another_card_can() {
    let content = content();
    let (mut run, _) = Run::new(&content, 11).expect("start");
    with_mod(&content, &mut run, "HA", WILD);
    deal_with(
        &mut run,
        &content,
        [
            "CJ SJ HA CA CT CK S8 ST D7 D8",
            "HJ DJ SA HK HQ H9 H8 H7 DA DT",
            "SK SQ S9 HT S7 DK DQ D9 CQ C9",
        ],
        "C8 C7",
        Seat::DEFENDERS[0],
    );
    run.apply_step(&content, RunAction::Declare(hand_clubs()))
        .expect("declare");
    defender_plays(&mut run, &content, Seat::DEFENDERS[0], "SA");
    defender_plays(&mut run, &content, Seat::DEFENDERS[1], "S7");
    let view = run.view(&content).game.expect("game view");
    let wild = view
        .hand
        .iter()
        .find(|c| c.card.id == id_of("HA"))
        .expect("held");
    assert_eq!(wild.wild_suits, [Suit::Spades]);
    let copy = run.clone();
    let err = run.apply_step(
        &content,
        RunAction::PlayAs {
            card: id_of("HA"),
            suit: Suit::Clubs,
        },
    );
    assert_eq!(
        err.map_err(|e| e.code()),
        Err(RunErrorCode::MustFollowSuit),
        "no ruff while a spade is held"
    );
    assert_eq!(run, copy);
    // The bot names the suit that follows; as a Daus of spades it does not
    // beat the earlier SA (equal strength keeps the trick, 4.4).
    let card = card_in(run.game().expect("game").game.hand(Seat::DECLARER), "HA");
    assert_eq!(RunBot::wild_suit(&run, card), Some(Suit::Spades));
    // Rest abkürzen waits while a Wandelkarte is in the hand.
    assert!(!run.can_settle(&content));
}

#[test]
fn the_bot_names_a_suit_that_takes_the_trick() {
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
    defender_plays(&mut run, &content, Seat::DEFENDERS[0], "SA");
    defender_plays(&mut run, &content, Seat::DEFENDERS[1], "S7");
    // No spade in the hand: the Wandelkarte may be anything; as a trump
    // (the Kreuz-Sieben) it takes the spade Daus.
    let card = card_in(run.game().expect("game").game.hand(Seat::DECLARER), "H7");
    assert_eq!(RunBot::wild_suit(&run, card), Some(Suit::Clubs));
    let events = run
        .apply_step(&content, RunBot::play_action(&run, card))
        .expect("plays");
    let won = events.iter().find_map(|e| match e {
        RunEvent::Game(Event::TrickWon { trick, .. }) => Some(trick.winner),
        _ => None,
    });
    assert_eq!(won, Some(Seat::DECLARER));
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

#[test]
fn the_bot_does_not_throw_a_wild_card_it_cannot_ruff_with() {
    let content = content();
    let (mut run, _) = Run::new(&content, 11).expect("start");
    with_mod(&content, &mut run, "HA", WILD);
    deal_with(
        &mut run,
        &content,
        [
            "CJ SJ HA HT HK HQ DK ST S7 D8",
            "HJ DJ DA DT H9 H8 S8 SA SK S9",
            "SQ H7 DQ D9 D7 CA CT CK CQ C9",
        ],
        "C8 C7",
        Seat::DEFENDERS[0],
    );
    let hearts = Declaration::suit(Suit::Hearts, RiskLevel::Hand);
    run.apply_step(&content, RunAction::Declare(hearts))
        .expect("declare");
    defender_plays(&mut run, &content, Seat::DEFENDERS[0], "DA");
    defender_plays(&mut run, &content, Seat::DEFENDERS[1], "D7");
    // As printed the Rot-Daus is a trump, but the player holds diamonds:
    // named it could only be a Schellen-Daus and lose to the first one.
    let action = RunBot.next_action(&run, &content).expect("plays");
    let hand = hand_of(&run).to_vec();
    let played = match action {
        RunAction::Play(id) | RunAction::PlayAs { card: id, .. } => id,
        other => panic!("{other:?}"),
    };
    let card = hand.iter().find(|c| c.id == played).expect("held");
    assert_eq!(card.face().map(|f| f.suit), Some(Suit::Diamonds), "{card}");
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
                        RunEvent::WildCardNamed { card, suit } => {
                            named += 1;
                            if card.face().is_some_and(|f| f.suit != suit) {
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

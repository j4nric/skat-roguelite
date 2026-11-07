//! Relics of M8 wave 2b (DESIGN 8.1): "Drei drücken", "Spickzettel" and
//! the new interactions they need, their edge cases, the bot
//! and old saves.

use super::bot::RunBot;
use super::content_tests::{content, fixed_deal, relic_run};
use super::view::RevealSource;
use super::*;
use crate::cards::Suit;
use crate::rules::{RiskLevel, discardable_of, returned_discard, valid_discard_of};

/// Declarer holds all trumps of a diamonds game but the D8 (in the skat).
const STRONG: [&str; 3] = [
    "CJ SJ HJ DJ DA DT DK DQ D9 D7",
    "HA HT HK HQ H9 H8 H7 SA ST SK",
    "SQ S9 S8 S7 CA CT CK CQ C9 C8",
];
const STRONG_SKAT: &str = "D8 C7";

/// Declarer holds two jacks; the HJ lies first in the skat.
const JACK_IN_SKAT: [&str; 3] = [
    "CJ SJ CA CT CK CQ C9 C8 C7 DA",
    "HA HT HK HQ H9 H8 H7 SA ST SK",
    "SQ S9 S8 S7 DJ DT DK DQ D9 D8",
];
const JACK_IN_SKAT_SKAT: &str = "HJ D7";

fn relic(id: &str) -> Source {
    Source::Relic(RelicId::new(id))
}

/// Deal a fixed hand with a target the series will not reach.
fn deal(run: &mut Run, content: &Content, hands: [&str; 3], skat: &str) {
    run.series.target = 100_000;
    run.series.game_limit = 100;
    fixed_deal(run, content, hands, skat);
}

fn card(game: &Game<DealtCard>, seat: Seat, notation: &str) -> DealtCard {
    let kind: CardKind = notation.parse().expect("notation");
    game.hand(seat)
        .iter()
        .copied()
        .find(|c| c.kind == kind)
        .unwrap_or_else(|| panic!("{notation} in the hand of {seat}"))
}

fn id(run: &Run, notation: &str) -> u32 {
    card(&run.game().expect("game").game, Seat::DECLARER, notation).id
}

fn scored(events: &[RunEvent]) -> Option<ScoreBreakdown> {
    events.iter().find_map(|e| match e {
        RunEvent::GameScored { breakdown, .. } => Some(breakdown.clone()),
        _ => None,
    })
}

/// Let the bot play the game to its end.
fn bot_to_end(run: &mut Run, content: &Content, mut events: Vec<RunEvent>) -> Vec<RunEvent> {
    while scored(&events).is_none() {
        let action = RunBot.next_action(run, content).expect("game continues");
        events.extend(run.apply(content, action).expect("legal"));
    }
    events
}

fn c(notation: &str) -> Card {
    notation.parse().expect("notation")
}

fn cards(notation: &str) -> Vec<Card> {
    notation.split_whitespace().map(c).collect()
}

// ---------------------------------------------------------------------------
// Drei drücken
// ---------------------------------------------------------------------------

#[test]
fn three_discard_draws_from_the_left_defender_and_gives_the_lowest_back() {
    let content = content();
    let mut run = relic_run(&content, &["three_discard"]);
    deal(&mut run, &content, STRONG, STRONG_SKAT);
    assert_eq!(
        run.game().expect("game").rules.discard_three,
        Some(relic("three_discard"))
    );
    let events = run.apply(&content, RunAction::PickUpSkat).expect("pick up");
    let game = &run.game().expect("game").game;
    let (from, drawn) = game.drawn().expect("drawn");
    assert_eq!(from, Seat::DEFENDERS[0], "the left defender");
    assert!(events.iter().any(|e| matches!(
        e,
        RunEvent::Game(Event::CardDrawn { seat, card }) if *seat == from && *card == drawn
    )));
    assert_eq!(game.hand(Seat::DECLARER).len(), 13);
    assert_eq!(game.hand(from).len(), 9);
    assert_eq!(game.discard_count(), 3);
    let view = run.view(&content).game.expect("game view");
    assert_eq!(view.discard_count, 3);
    assert_eq!(view.hand.len(), 13);
    let marked: Vec<&view::CardView> = view
        .hand
        .iter()
        .filter(|c| c.drawn_from.is_some())
        .collect();
    assert_eq!(marked.len(), 1);
    assert_eq!(marked[0].card.id, drawn.id);
    assert_eq!(view.hand.iter().filter(|c| c.from_skat).count(), 2);

    // Two cards are not enough now.
    let two = RunAction::Discard([id(&run, "D8"), id(&run, "C7")]);
    assert_eq!(
        run.apply(&content, two).err().map(|e| e.code()),
        Some(RunErrorCode::InvalidDiscard)
    );
    // C7 and D8 have no Augen; the seven is lower and goes back.
    let three = [id(&run, "D8"), id(&run, "C7"), id(&run, "D9")];
    let events = run
        .apply(&content, RunAction::DiscardThree(three))
        .expect("discard three");
    let game = &run.game().expect("game").game;
    let back = card(game, from, "C7");
    assert_eq!(game.returned(), Some((from, back)));
    assert!(events.iter().any(|e| matches!(
        e,
        RunEvent::Game(Event::CardReturned { seat, card }) if *seat == from && card.id == back.id
    )));
    assert_eq!(game.hand(Seat::DECLARER).len(), 10);
    assert_eq!(game.hand(from).len(), 10);
    let skat: Vec<String> = game.skat().iter().map(|c| c.kind.to_string()).collect();
    assert_eq!(skat, vec!["D8", "D9"]);
    assert_eq!(game.declarer_cards().len(), 12);
    assert_eq!(game.phase(), Phase::DeclaringAfterSkat);
    // The player knows who holds the card he gave back.
    let view = run.view(&content).game.expect("game view");
    let known = view
        .revealed
        .iter()
        .find(|r| r.card.card.id == back.id)
        .expect("returned card shown");
    assert_eq!(known.source, RevealSource::Returned);
    assert_eq!(known.holder, Some(from));
    // The PIMC player knows it too.
    assert!(
        opponents::player_hints(&run)
            .pinned
            .contains(&(back, Some(from)))
    );
    // And the game plays to its end.
    let events = run
        .apply(
            &content,
            RunAction::Declare(Declaration::suit(Suit::Diamonds, RiskLevel::Normal)),
        )
        .expect("declare");
    let events = bot_to_end(&mut run, &content, events);
    assert!(scored(&events).is_some());
}

#[test]
fn three_discard_counts_spitzen_over_the_cards_kept() {
    // Declarer without the CJ; the left defender holds it.
    let hands = [
        cards("SJ HJ DJ DA DT DK DQ D9 D7 HA"),
        cards("CJ HT HK HQ H9 H8 H7 SA ST SK"),
        cards("SQ S9 S8 S7 CA CT CK CQ C9 C8"),
    ];
    let skat = [c("D8"), c("C7")];
    let kept = |keep_jack: bool| {
        let mut game = Game::with_cards(hands.clone(), skat, Seat::DECLARER).expect("deal");
        game.pick_up_skat_drawing(Seat::DEFENDERS[0], c("CJ"))
            .expect("draw");
        let discard = if keep_jack {
            [c("C7"), c("D8"), c("HA")]
        } else {
            // The jack is the lowest of the three (2 Augen) and goes back.
            [c("CJ"), c("HA"), c("DA")]
        };
        game.discard_three(discard).expect("discard");
        game.declare(Declaration::suit(Suit::Diamonds, RiskLevel::Normal))
            .expect("declare");
        game.matadors().expect("matadors")
    };
    let with = kept(true);
    assert!(with.with && with.count >= 4, "{with:?}");
    let without = kept(false);
    assert!(!without.with && without.count == 1, "{without:?}");
}

#[test]
fn three_discard_rules_for_the_returned_card_and_ramponiert() {
    let low = returned_discard([c("D8"), c("C7"), c("DA")]);
    assert_eq!(low, c("C7"));
    let low = returned_discard([c("H9"), c("S9"), c("CQ")]);
    assert_eq!(low, c("H9"), "equal Augen and rank: the lower suit");
    let low = returned_discard([c("SJ"), c("HQ"), c("ST")]);
    assert_eq!(low, c("SJ"), "a jack has 2 Augen");
    // Thirteen cards, two of them must not be discarded.
    let mut hand: Vec<DealtCard> = (0u32..13)
        .map(|i| DealtCard::plain(i, CardKind::Blank))
        .collect();
    hand[0].no_discard = true;
    hand[1].no_discard = true;
    assert!(valid_discard_of(&hand, &[hand[2], hand[3], hand[4]]));
    assert!(!valid_discard_of(&hand, &[hand[0], hand[3], hand[4]]));
    assert!(!discardable_of(&hand, hand[0], 3));
    assert!(discardable_of(&hand, hand[5], 3));
    // With only two free cards left one bound card must go.
    let bound: Vec<DealtCard> = hand
        .iter()
        .map(|&c| DealtCard {
            no_discard: c.id > 1,
            ..c
        })
        .collect();
    assert!(discardable_of(&bound, bound[5], 3));
    assert!(valid_discard_of(&bound, &[bound[0], bound[1], bound[5]]));
    assert!(!valid_discard_of(&bound, &[bound[0], bound[4], bound[5]]));
}

#[test]
fn three_discard_does_nothing_in_a_hand_game_and_draws_the_same_card_again() {
    let content = content();
    let mut run = relic_run(&content, &["three_discard"]);
    deal(&mut run, &content, STRONG, STRONG_SKAT);
    run.apply(
        &content,
        RunAction::Declare(Declaration::suit(Suit::Diamonds, RiskLevel::Hand)),
    )
    .expect("hand");
    let game = &run.game().expect("game").game;
    assert_eq!(game.drawn(), None);
    assert_eq!(game.hand(Seat::DEFENDERS[0]).len(), 10);
    // A hand game does not use it up.
    assert!(run.discard_three_ready());

    // The draw comes from the game's seed: equal runs draw equal cards.
    let draw = || {
        let mut run = relic_run(&content, &["three_discard"]);
        deal(&mut run, &content, STRONG, STRONG_SKAT);
        run.apply(&content, RunAction::PickUpSkat).expect("pick up");
        run.game().expect("game").game.drawn()
    };
    assert_eq!(draw(), draw());
}

#[test]
fn three_discard_bot_plays_whole_series() {
    let content = content();
    let start = RunStart {
        relics: vec![RelicId::new("three_discard")],
        ..RunStart::default()
    };
    let mut drawn = 0;
    for seed in 0..6 {
        let (mut run, _) = Run::new_with(&content, seed, &start).expect("run");
        while run.series().section == 0 {
            let Some(action) = RunBot.next_action(&run, &content) else {
                break;
            };
            if matches!(action, RunAction::DiscardThree(_)) {
                drawn += 1;
            }
            run.apply(&content, action).expect("legal");
        }
    }
    assert!(drawn > 0, "the bot discarded three");
}

#[test]
fn old_saves_load_without_the_new_fields() {
    let content = content();
    let mut run = relic_run(&content, &[]);
    deal(&mut run, &content, STRONG, STRONG_SKAT);
    let mut text = ron::to_string(&run).expect("ron");
    for field in ["drawn:", "returned:", "skat_shown:"] {
        assert!(!text.contains(field), "{field} left out while unused");
    }
    // Rules of a save from before have neither of the new rules.
    for field in [",discard_three:None", ",discard_three_used:false"] {
        assert!(text.contains(field), "{field}");
        text = text.replacen(field, "", 1);
    }
    let loaded: Run = ron::from_str(&text).expect("old save loads");
    assert_eq!(loaded, run);
}

// ---------------------------------------------------------------------------
// Spickzettel
// ---------------------------------------------------------------------------

#[test]
fn crib_sheet_shows_both_skat_cards_after_a_hand_declaration() {
    let content = content();
    let mut run = relic_run(&content, &["crib_sheet"]);
    deal(&mut run, &content, STRONG, STRONG_SKAT);
    assert!(run.view(&content).game.expect("view").skat_shown.is_empty());
    let events = run
        .apply(
            &content,
            RunAction::Declare(Declaration::suit(Suit::Diamonds, RiskLevel::Hand)),
        )
        .expect("hand");
    let shown: Vec<DealtCard> = events
        .iter()
        .filter_map(|e| match e {
            RunEvent::SkatCardShown { source, card } => {
                assert_eq!(source, &relic("crib_sheet"));
                Some(*card)
            }
            _ => None,
        })
        .collect();
    let skat = run.game().expect("game").game.skat();
    // Since 02.11.2026 both cards.
    assert_eq!(shown, skat.to_vec(), "both skat cards");
    let view = run.view(&content).game.expect("view");
    assert_eq!(view.skat_shown.len(), 2);
    assert_eq!(view.skat_shown[0].card.id, skat[0].id);
    assert_eq!(view.skat_shown[1].card.id, skat[1].id);
    for card in skat {
        assert!(opponents::player_hints(&run).pinned.contains(&(card, None)));
    }
    // The player's event carries it too.
    let public: Vec<_> = events.into_iter().map(RunEvent::for_player).collect();
    assert!(
        public
            .iter()
            .any(|e| matches!(e, RunEvent::SkatCardShown { .. }))
    );
}

#[test]
fn crib_sheet_shows_nothing_after_the_skat() {
    let content = content();
    let mut run = relic_run(&content, &["crib_sheet"]);
    deal(&mut run, &content, STRONG, STRONG_SKAT);
    let events = super::content_tests::declare(
        &mut run,
        &content,
        Declaration::suit(Suit::Diamonds, RiskLevel::Normal),
    )
    .expect("declare");
    assert!(
        !events
            .iter()
            .any(|e| matches!(e, RunEvent::SkatCardShown { .. }))
    );
    assert!(run.game().expect("game").skat_shown.is_empty());
}

#[test]
fn crib_sheet_tells_the_value_preview_about_a_jack_in_the_skat() {
    let content = content();
    let preview = |relics: &[&str]| {
        let mut run = relic_run(&content, relics);
        deal(&mut run, &content, JACK_IN_SKAT, JACK_IN_SKAT_SKAT);
        run.apply(
            &content,
            RunAction::Declare(Declaration::suit(Suit::Clubs, RiskLevel::Hand)),
        )
        .expect("hand");
        run.view(&content)
            .game
            .expect("view")
            .value_preview
            .expect("preview")
    };
    // Without it the HJ may be with a defender: mit 2. With it the player
    // knows it lies in the skat: mit 3, one level more, and since relics v4
    // (07.11.2026) the Spickzettel's own level for the hand game.
    let without = preview(&[]);
    let with = preview(&["crib_sheet"]);
    assert_eq!(with.levels, without.levels + 2, "{with:?} {without:?}");
}

#[test]
fn three_discard_keeps_every_card_once() {
    let content = content();
    let start = RunStart {
        relics: vec![RelicId::new("three_discard")],
        ..RunStart::default()
    };
    let mut checked = 0;
    for seed in 0..8 {
        let (mut run, _) = Run::new_with(&content, seed, &start).expect("run");
        while run.series().section == 0 {
            let Some(action) = RunBot.next_action(&run, &content) else {
                break;
            };
            let before = run.game().map(|a| a.game.clone());
            let events = run.apply(&content, action).expect("legal");
            let Some(game) = before else { continue };
            for e in &events {
                let RunEvent::Game(Event::GameFinished { outcome, .. }) = e else {
                    continue;
                };
                // Effekte v2: "Erster Stich" gives extra Augen.
                let bonus: u32 = game.bonus_points().iter().sum::<u32>()
                    + events
                        .iter()
                        .map(|e| match e {
                            RunEvent::BonusPoints { card_points, .. } => *card_points,
                            _ => 0,
                        })
                        .sum::<u32>();
                // and "Ramponiert" takes Augen, also below 0.
                let signed: i64 = game.cards_in_play().iter().map(|c| c.signed_points()).sum();
                let total = u32::try_from(signed).expect("positive") + bonus;
                let finished = game.drawn().is_some() || !game.tricks().is_empty();
                if finished && game.declaration().is_some_and(|d| !d.is_null()) {
                    let sum = outcome.declarer_card_points + outcome.defender_card_points;
                    // A seat never has fewer than 0 Augen (`seat_card_points`):
                    // with a card below 0 in play the sum may lie above.
                    let below_zero = game.cards_in_play().iter().any(|c| c.signed_points() < 0);
                    if below_zero {
                        assert!(sum >= total, "{sum} < {total}");
                    } else {
                        assert_eq!(sum, total);
                    }
                    checked += 1;
                }
            }
        }
    }
    assert!(checked > 0);
}

#[test]
fn three_discard_works_once_per_series() {
    let content = content();
    let mut run = relic_run(&content, &["three_discard"]);
    deal(&mut run, &content, STRONG, STRONG_SKAT);
    assert!(run.view(&content).game.expect("view").discard_three_ready);
    run.apply(&content, RunAction::PickUpSkat).expect("pick up");
    assert!(run.game().expect("game").game.drawn().is_some());
    assert!(run.series().discard_three_used);
    assert!(!run.discard_three_ready());
    // The next game of the series: an ordinary pickup with two cards.
    deal(&mut run, &content, STRONG, STRONG_SKAT);
    assert!(!run.view(&content).game.expect("view").discard_three_ready);
    run.apply(&content, RunAction::PickUpSkat).expect("pick up");
    let game = &run.game().expect("game").game;
    assert_eq!(game.drawn(), None);
    assert_eq!(game.discard_count(), 2);
    // A new series grants it again.
    let mut events = Vec::new();
    run.start_series(&content, 0, 1, &mut events)
        .expect("series");
    assert!(run.discard_three_ready());
    deal(&mut run, &content, STRONG, STRONG_SKAT);
    run.apply(&content, RunAction::PickUpSkat).expect("pick up");
    assert!(run.game().expect("game").game.drawn().is_some());
}

//! Card modification "Untot" (DESIGN 7.2): a card of the player that rises
//! from a completed trick back into his hand once per game, a random other
//! card of his hand taking its place in the trick. One test per rule edge.

use super::bot::RunBot;
use super::content_tests::content;
use super::wave3_tests::{
    bot_to_end, card_in, deal_with, defender_plays, hand_clubs, id_of, with_mod, with_old_specials,
};
use super::*;
use crate::content::GameFocus;
use crate::rules::NullRisk;

const UNDEAD: &str = "undead";

/// Player forehand with all four Unter and the clubs top; the left defender
/// has no trump, the right one CQ and C8.
const HANDS: [&str; 3] = [
    "CJ SJ HJ DJ CA CT CK C9 D7 H7",
    "HA HT HK HQ H9 H8 SA ST SK SQ",
    "S9 S8 S7 DA DT DK DQ D9 CQ C8",
];
const SKAT: &str = "D8 C7";

fn undead_run(card: &str) -> (Content, Run) {
    let content = content();
    let (mut run, _) = Run::new(&content, 11).expect("start");
    with_mod(&content, &mut run, card, UNDEAD);
    deal_with(&mut run, &content, HANDS, SKAT, Seat::DECLARER);
    (content, run)
}

fn rose(events: &[RunEvent]) -> Option<(DealtCard, DealtCard)> {
    events.iter().find_map(|e| match e {
        RunEvent::Game(Event::CardRose {
            card, substitute, ..
        }) => Some((*card, *substitute)),
        _ => None,
    })
}

fn game(run: &Run) -> &Game<DealtCard> {
    &run.game().expect("game").game
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

#[test]
fn undead_loads_as_a_special_card_for_every_card() {
    let content = content();
    let def = content.card_mod(&CardModId::new(UNDEAD)).expect("undead");
    assert_eq!(def.name, "Untot");
    assert_eq!(def.focus, GameFocus::Trump);
    assert!(def.is_special());
    assert!(def.passive.contains(&Effect::Undead));
    for card in ["CK", "HQ", "SA", "D7", "CJ"] {
        assert!(def.fits(CardKind::Normal(card.parse().expect("card"))));
    }
    assert!(!def.fits(CardKind::Blank));
    // Effekte v2: a special card, it comes with the voucher "Gutschein
    // Untot", never from a random draw.
    let (run, _) = Run::new(&content, 1).expect("run");
    assert!(
        !run.card_mod_pool(&content)
            .iter()
            .any(|(id, _)| id.as_str() == UNDEAD)
    );
}

#[test]
fn an_undead_card_rises_and_a_hand_card_takes_its_place_in_the_trick() {
    let (content, mut run) = undead_run("CJ");
    run.apply_step(&content, RunAction::Declare(hand_clubs()))
        .expect("declare");
    let matadors = game(&run).matadors();
    let events = trick(&mut run, &content, "CJ", "H8", "C8");
    let (card, substitute) = rose(&events).expect("rose");
    assert_eq!(card.id, id_of("CJ"));
    assert_ne!(substitute.id, card.id);
    let g = game(&run);
    // The hand keeps one card less per trick: nine, with the Unter back.
    let hand = g.hand(Seat::DECLARER);
    assert_eq!(hand.len(), 9);
    assert!(hand.iter().any(|c| c.id == id_of("CJ")));
    assert!(!hand.contains(&substitute));
    // The trick keeps its winner and holds the substitute.
    let done = g.tricks()[0];
    assert_eq!(done.winner, Seat::DECLARER);
    assert_eq!(done.cards[0], substitute);
    assert_eq!(g.trick_card_points()[0], substitute.card_points);
    // The rising comes after the trick, and the Spitzen stay as declared.
    let won = events
        .iter()
        .position(|e| matches!(e, RunEvent::Game(Event::TrickWon { .. })));
    let risen = events
        .iter()
        .position(|e| matches!(e, RunEvent::Game(Event::CardRose { .. })));
    assert!(won < risen);
    assert_eq!(g.matadors(), matadors);
    assert_eq!(run.game().expect("game").risen, vec![id_of("CJ")]);
}

#[test]
fn an_undead_card_rises_only_once_per_game() {
    let (content, mut run) = undead_run("CJ");
    run.apply_step(&content, RunAction::Declare(hand_clubs()))
        .expect("declare");
    assert!(rose(&trick(&mut run, &content, "CJ", "H8", "C8")).is_some());
    let view = run.view(&content);
    let cj = view
        .game
        .as_ref()
        .expect("game")
        .hand
        .iter()
        .find(|c| c.card.id == id_of("CJ"))
        .expect("held")
        .can_rise;
    assert!(!cj, "risen once: no more");
    let events = trick(&mut run, &content, "CJ", "H9", "CQ");
    assert_eq!(rose(&events), None);
    assert_eq!(game(&run).hand(Seat::DECLARER).len(), 8);
    assert!(
        game(&run)
            .tricks()
            .iter()
            .flat_map(|t| t.cards)
            .any(|c| c.id == id_of("CJ")),
        "it stays in the second trick"
    );
}

#[test]
fn a_defenders_undead_card_does_not_rise() {
    let (content, mut run) = undead_run("CQ");
    run.apply_step(&content, RunAction::Declare(hand_clubs()))
        .expect("declare");
    let events = trick(&mut run, &content, "CJ", "H8", "CQ");
    assert_eq!(rose(&events), None);
    assert_eq!(game(&run).hand(Seat::DEFENDERS[1]).len(), 9);
}

#[test]
fn a_losing_undead_card_rises_and_the_substitute_goes_to_the_defender() {
    let (content, mut run) = undead_run("H7");
    run.apply_step(&content, RunAction::Declare(hand_clubs()))
        .expect("declare");
    let events = trick(&mut run, &content, "H7", "HA", "S9");
    let (_, substitute) = rose(&events).expect("rose");
    let g = game(&run);
    assert_eq!(g.tricks()[0].winner, Seat::DEFENDERS[0]);
    assert_eq!(
        g.trick_card_points()[1],
        11 + substitute.card_points,
        "Daus plus the substitute, not the Sieben"
    );
    assert_eq!(g.trick_card_points()[0], 0);
}

#[test]
fn the_last_card_stays_and_every_card_ends_in_exactly_one_trick() {
    for seed in 0..12 {
        let content = content();
        let (mut run, _) = Run::new(&content, seed).expect("start");
        // Eight Untot, as a save from before the cap may hold them.
        with_old_specials(
            &mut run,
            &["CJ", "SJ", "HJ", "CA", "HA", "SA", "DA", "D7"],
            UNDEAD,
        );
        deal_with(&mut run, &content, HANDS, SKAT, Seat::DECLARER);
        run.apply_step(&content, RunAction::Declare(hand_clubs()))
            .expect("declare");
        let (events, _) = bot_to_end(&mut run, &content);
        let rises: Vec<u32> = events
            .iter()
            .filter_map(|e| match e {
                RunEvent::Game(Event::CardRose { card, .. }) => Some(card.id),
                _ => None,
            })
            .collect();
        let mut distinct = rises.clone();
        distinct.sort_unstable();
        distinct.dedup();
        assert_eq!(distinct.len(), rises.len(), "each card once ({seed})");
        let tricks: Vec<crate::game::CompletedTrick<DealtCard>> = events
            .iter()
            .filter_map(|e| match e {
                RunEvent::Game(Event::TrickWon { trick, .. }) => Some(*trick),
                _ => None,
            })
            .collect();
        assert_eq!(tricks.len(), 10);
        let finished = events.iter().find_map(|e| match e {
            RunEvent::Game(Event::GameFinished { outcome, .. }) => Some(*outcome),
            _ => None,
        });
        let outcome = finished.expect("finished");
        assert_eq!(
            outcome.declarer_card_points + outcome.defender_card_points,
            120,
            "the Augen of the 30 played cards and the skat ({seed})"
        );
        assert_eq!(outcome.declarer_tricks + outcome.defender_tricks, 10);
    }
}

#[test]
fn every_card_is_in_one_place_at_every_step() {
    let content = content();
    let (mut run, _) = Run::new(&content, 5).expect("start");
    with_old_specials(&mut run, &["CJ", "SJ", "HJ", "DJ", "CA"], UNDEAD);
    deal_with(&mut run, &content, HANDS, SKAT, Seat::DECLARER);
    run.apply_step(&content, RunAction::Declare(hand_clubs()))
        .expect("declare");
    let mut rises = 0;
    loop {
        let g = game(&run);
        let mut cards: Vec<u32> = g
            .tricks()
            .iter()
            .flat_map(|t| t.cards)
            .chain(g.current_trick().iter().copied())
            .chain(Seat::ALL.iter().flat_map(|&s| g.hand(s).iter().copied()))
            .map(|c| c.id)
            .collect();
        assert_eq!(cards.len(), 30);
        cards.sort_unstable();
        cards.dedup();
        assert_eq!(cards.len(), 30, "no card twice, none lost");
        let played = g.tricks().len() * 3 + g.current_trick().len();
        for seat in Seat::ALL {
            let seat_played = g.tricks().len()
                + usize::from(
                    (0..g.current_trick().len())
                        .any(|i| crate::game::seat_at(g.trick_lead(), i) == seat),
                );
            assert_eq!(g.hand(seat).len(), 10 - seat_played, "{played}");
        }
        let events = if run.ai_to_move().is_some() {
            run.advance_ai(&content).expect("ai")
        } else {
            let action = RunBot.next_action(&run, &content).expect("action");
            run.apply_step(&content, action).expect("legal")
        };
        rises += events
            .iter()
            .filter(|e| matches!(e, RunEvent::Game(Event::CardRose { .. })))
            .count();
        if events
            .iter()
            .any(|e| matches!(e, RunEvent::GameScored { .. }))
        {
            break;
        }
    }
    assert!(rises <= 5);
}

#[test]
fn an_undead_card_does_not_rise_in_null() {
    let content = content();
    let (mut run, _) = Run::new(&content, 11).expect("start");
    with_mod(&content, &mut run, "H7", UNDEAD);
    deal_with(&mut run, &content, HANDS, SKAT, Seat::DECLARER);
    run.apply_step(
        &content,
        RunAction::Declare(Declaration::Null {
            risk: NullRisk::Hand,
        }),
    )
    .expect("declare");
    assert!(!run.can_rise(&content, card_in(game(&run).hand(Seat::DECLARER), "H7")));
    let events = trick(&mut run, &content, "H7", "H8", "S9");
    assert_eq!(rose(&events), None);
}

#[test]
fn an_undead_card_does_not_rise_in_a_ramsch() {
    let (content, mut run) = undead_run("CJ");
    let active = run.game.as_mut().expect("game");
    active.game.declare(Declaration::Ramsch).expect("ramsch");
    active.rules.ramsch = true;
    let events = trick(&mut run, &content, "CJ", "H8", "C8");
    assert_eq!(rose(&events), None);
}

/// The player leads `lead`, both defenders throw their first legal card.
fn lead_and_slough(run: &mut Run, content: &Content, lead: &str) -> Vec<RunEvent> {
    let mut events = run
        .apply_step(content, RunAction::Play(id_of(lead)))
        .expect("lead");
    for seat in Seat::DEFENDERS {
        let card = game(run).legal_moves(seat)[0];
        let mut more = Vec::new();
        run.play_card(content, seat, card, &mut more)
            .expect("legal");
        run.settle(content, &mut more).expect("settle");
        events.extend(more);
    }
    events
}

#[test]
fn rest_abkuerzen_waits_while_an_undead_card_can_rise() {
    // The player holds all ten trumps: every trick is his.
    let hands = [
        "CJ SJ HJ DJ CA CT CK CQ C9 C8",
        "HA HT HK HQ H9 H8 SA ST SK SQ",
        "S9 S8 S7 DA DT DK DQ D9 D8 D7",
    ];
    let content = content();
    let setup = |undead: bool| {
        let (mut run, _) = Run::new(&content, 11).expect("start");
        if undead {
            with_mod(&content, &mut run, "C8", UNDEAD);
        }
        deal_with(&mut run, &content, hands, "H7 C7", Seat::DECLARER);
        run.apply_step(&content, RunAction::Declare(hand_clubs()))
            .expect("declare");
        for lead in ["CJ", "SJ", "HJ", "DJ", "CA", "CT"] {
            lead_and_slough(&mut run, &content, lead);
        }
        run
    };
    assert!(setup(false).can_settle(&content), "decided without Untot");
    let mut run = setup(true);
    let c8 = card_in(game(&run).hand(Seat::DECLARER), "C8");
    assert!(run.can_rise(&content, c8));
    assert!(!run.can_settle(&content), "the substitute is not known yet");
    assert!(
        run.apply_step(&content, RunAction::SettleGame).is_err(),
        "not offered"
    );
    // Once it rose, the rest is decided again.
    let events = lead_and_slough(&mut run, &content, "C8");
    assert!(rose(&events).is_some());
    assert!(run.can_settle(&content));
}

#[test]
fn the_view_marks_a_card_that_would_rise() {
    let (content, mut run) = undead_run("CK");
    let marked = |run: &Run| -> Vec<u32> {
        run.view(&content)
            .game
            .expect("game")
            .hand
            .iter()
            .filter(|c| c.can_rise)
            .map(|c| c.card.id)
            .collect()
    };
    assert!(marked(&run).is_empty(), "before the declaration");
    run.apply_step(&content, RunAction::Declare(hand_clubs()))
        .expect("declare");
    assert_eq!(marked(&run), vec![id_of("CK")]);
    let text = ron::to_string(&run.view(&content)).expect("ron");
    assert_eq!(text.matches("can_rise").count(), 1, "left out when false");
}

#[test]
fn rising_is_deterministic_and_drawn_from_the_game_generator() {
    let play = || {
        let (content, mut run) = undead_run("CJ");
        run.apply_step(&content, RunAction::Declare(hand_clubs()))
            .expect("declare");
        rose(&trick(&mut run, &content, "CJ", "H8", "C8")).expect("rose")
    };
    assert_eq!(play(), play());
    // A normal deal forks the generator if an Untot card is in play, and
    // only then.
    let content = content();
    let (mut run, _) = Run::new(&content, 3).expect("start");
    assert!(run.game().expect("game").undead_rng.is_none());
    with_mod(&content, &mut run, "CJ", UNDEAD);
    let mut events = Vec::new();
    run.deal(&content, true, &mut events).expect("deal");
    assert!(run.game().expect("game").undead_rng.is_some());
}

#[test]
fn a_save_with_a_risen_card_loads_and_a_save_without_the_fields_too() {
    let (content, mut run) = undead_run("CJ");
    run.apply_step(&content, RunAction::Declare(hand_clubs()))
        .expect("declare");
    // Before the rising neither `risen` nor `undead_rng` is written, as in
    // saves from before them.
    let old = ron::to_string(&run).expect("save");
    assert!(!old.contains("risen") && !old.contains("undead_rng"));
    let mut loaded: Run = ron::from_str(&old).expect("old save loads");
    let a = trick(&mut run, &content, "CJ", "H8", "C8");
    let b = trick(&mut loaded, &content, "CJ", "H8", "C8");
    assert_eq!(rose(&a), rose(&b), "same substitute after loading");
    let text = ron::to_string(&run).expect("save");
    assert!(text.contains("risen"));
    let back: Run = ron::from_str(&text).expect("load");
    assert_eq!(back, run);
}

#[test]
fn player_events_show_the_rising() {
    let (content, mut run) = undead_run("CJ");
    run.apply_step(&content, RunAction::Declare(hand_clubs()))
        .expect("declare");
    let events = trick(&mut run, &content, "CJ", "H8", "C8");
    let (card, substitute) = rose(&events).expect("rose");
    let seen: Vec<_> = events.into_iter().map(RunEvent::for_player).collect();
    assert!(
        seen.contains(&RunEvent::Game(super::player::PlayerGameEvent::CardRose {
            seat: Seat::DECLARER,
            card,
            substitute,
        }))
    );
}

#[test]
fn the_bot_plays_an_undead_card_last_when_it_takes_the_trick() {
    let content = content();
    let (mut run, _) = Run::new(&content, 11).expect("start");
    with_mod(&content, &mut run, "CA", UNDEAD);
    // The right defender leads, the player plays last.
    deal_with(&mut run, &content, HANDS, SKAT, Seat::DEFENDERS[0]);
    run.apply_step(&content, RunAction::Declare(hand_clubs()))
        .expect("declare");
    defender_plays(&mut run, &content, Seat::DEFENDERS[0], "HA");
    defender_plays(&mut run, &content, Seat::DEFENDERS[1], "S9");
    // Only the H7 is legal: it follows hearts and loses.
    assert_eq!(RunBot::undead_card(&run, &content), None);
    let events = run
        .apply_step(&content, RunAction::Play(id_of("H7")))
        .expect("follow");
    assert!(rose(&events).is_none());
    // Trick 2: the left defender won and leads a spade, the player is void.
    assert_eq!(game(&run).trick_lead(), Seat::DEFENDERS[0]);
    defender_plays(&mut run, &content, Seat::DEFENDERS[0], "SA");
    defender_plays(&mut run, &content, Seat::DEFENDERS[1], "S8");
    let card = RunBot::undead_card(&run, &content).expect("ruff with the Untot");
    assert_eq!(card.id, id_of("CA"));
    assert_eq!(
        RunBot.next_action(&run, &content),
        Some(RunAction::Play(id_of("CA")))
    );
}

#[test]
fn game_raise_checks_its_cards_and_the_trick_boundary() {
    let c = |s: &str| -> Card { s.parse().expect("card") };
    let parse = |s: &str| crate::cards::parse_cards(s).expect("hand");
    let mut g = Game::with_cards(
        [parse(HANDS[0]), parse(HANDS[1]), parse(HANDS[2])],
        [c("D8"), c("C7")],
        Seat::DECLARER,
    )
    .expect("deal");
    g.declare(hand_clubs()).expect("declare");
    assert!(matches!(
        g.raise(Seat::DECLARER, c("CJ"), c("D7")),
        Err(GameError::CardNotInHand { .. })
    ));
    g.play(Seat::DECLARER, c("CJ")).expect("lead");
    assert_eq!(
        g.raise(Seat::DECLARER, c("CJ"), c("D7")),
        Err(GameError::TrickInProgress)
    );
    g.play(Seat::DEFENDERS[0], c("H8")).expect("slough");
    g.play(Seat::DEFENDERS[1], c("C8")).expect("follow");
    assert!(
        g.raise(Seat::DEFENDERS[0], c("CJ"), c("HA")).is_err(),
        "not his card"
    );
    assert!(
        g.raise(Seat::DECLARER, c("CJ"), c("HA")).is_err(),
        "not in his hand"
    );
    let events = g.raise(Seat::DECLARER, c("CA"), c("D7"));
    assert!(events.is_err(), "CA is not in the trick");
    let events = g.raise(Seat::DECLARER, c("CJ"), c("CA")).expect("rise");
    assert_eq!(
        events,
        vec![Event::CardRose {
            seat: Seat::DECLARER,
            card: c("CJ"),
            substitute: c("CA"),
        }]
    );
    assert_eq!(g.trick_card_points()[0], 11);
    assert_eq!(g.tricks()[0].cards, [c("CA"), c("H8"), c("C8")]);
    assert_eq!(g.log().last(), events.last());
}

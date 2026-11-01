//! The order of the Skatordnung (ADR 0011): with ten cards the player
//! declares a hand game or picks up the skat; after picking it up he
//! discards and only then declares, without hand levels.

use super::content_tests::{content, fixed_deal, relic_run};
use super::view::TablePhase;
use super::*;
use crate::cards::Suit;
use crate::rules::{NullRisk, RiskLevel};

/// Declarer holds all trumps of a diamonds game but the D8 (in the skat).
const STRONG: [&str; 3] = [
    "CJ SJ HJ DJ DA DT DK DQ D9 D7",
    "HA HT HK HQ H9 H8 H7 SA ST SK",
    "SQ S9 S8 S7 CA CT CK CQ C9 C8",
];
const STRONG_SKAT: &str = "D8 C7";

/// Declarer holds only Luschen; seat 1 holds eight trumps of a clubs game
/// and says Kontra.
const WEAK: [&str; 3] = [
    "D7 D8 D9 H7 H8 H9 S7 S8 S9 C7",
    "CJ SJ HJ DJ CA CT CK CQ DA DT",
    "DK DQ HA HT HK HQ SA ST SK SQ",
];
const WEAK_SKAT: &str = "C8 C9";

fn diamonds(risk: RiskLevel) -> Declaration {
    Declaration::suit(Suit::Diamonds, risk)
}

/// Instance id of a card in the standard deck.
fn id(run: &Run, notation: &str) -> u32 {
    let kind: crate::cards::CardKind = notation.parse().expect("notation");
    run.deck()
        .cards()
        .iter()
        .find(|c| c.kind == kind)
        .map(|c| c.id)
        .expect("card in deck")
}

#[test]
fn the_skat_comes_before_the_declaration() {
    let content = content();
    let mut run = relic_run(&content, &[]);
    fixed_deal(&mut run, &content, STRONG, STRONG_SKAT);
    // Before the skat: no game without Hand.
    let normal = RunAction::Declare(diamonds(RiskLevel::Normal));
    let err = run
        .clone()
        .apply(&content, normal.clone())
        .expect_err("hand");
    assert_eq!(err.code(), RunErrorCode::InvalidDeclaration);
    assert_eq!(
        RunErrorCode::InvalidDeclaration.as_str(),
        "InvalidDeclaration"
    );

    let events = run.apply(&content, RunAction::PickUpSkat).expect("pick up");
    assert!(matches!(
        events[..],
        [RunEvent::Game(Event::SkatPickedUp { .. })]
    ));
    let view = run.view(&content).game.expect("game");
    assert_eq!(view.phase, TablePhase::Discarding);
    assert_eq!(view.hand.len(), 12);
    assert!(view.declare.is_none());
    assert!(view.skat.is_none());
    // No way back: no pass, no redeal, no second pick-up, no declaration
    // before the discard.
    for action in [
        RunAction::Pass,
        RunAction::Redeal,
        RunAction::PickUpSkat,
        normal.clone(),
    ] {
        assert!(!run.accepts(&content, action.clone()), "{action:?}");
    }

    run.apply(
        &content,
        RunAction::Discard([id(&run, "D8"), id(&run, "C7")]),
    )
    .expect("discard");
    let view = run.view(&content).game.expect("game");
    assert_eq!(view.phase, TablePhase::DeclaringAfterSkat);
    assert_eq!(view.hand.len(), 10);
    assert_eq!(view.skat.as_ref().map(Vec::len), Some(2), "the discards");
    let options = view.declare.expect("options");
    assert_eq!(
        options.risks.iter().map(|r| r.risk).collect::<Vec<_>>(),
        vec![RiskLevel::Normal]
    );
    assert!(options.risks[0].allowed);
    assert!(options.games.iter().all(|g| g.allowed));
    assert!(!options.can_pick_up_skat && !options.can_pass && !options.can_redeal);
    // After the skat: no hand levels.
    for risk in [
        RiskLevel::Hand,
        RiskLevel::SchneiderDeclared,
        RiskLevel::SchwarzDeclared,
        RiskLevel::Ouvert,
    ] {
        let err = run
            .clone()
            .apply(&content, RunAction::Declare(diamonds(risk)))
            .expect_err("no hand level after the skat");
        assert_eq!(err.code(), RunErrorCode::InvalidDeclaration, "{risk:?}");
    }
    let events = run.apply(&content, normal).expect("declare");
    assert!(events.contains(&RunEvent::Game(Event::Declared {
        declaration: diamonds(RiskLevel::Normal)
    })));
    let view = run.view(&content).game.expect("game");
    assert_eq!(view.phase, TablePhase::Playing);
    assert!(view.your_turn);
}

#[test]
fn before_the_skat_the_dialog_offers_hand_levels_and_the_skat() {
    let content = content();
    let mut run = relic_run(&content, &[]);
    fixed_deal(&mut run, &content, STRONG, STRONG_SKAT);
    let options = run
        .view(&content)
        .game
        .expect("game")
        .declare
        .expect("options");
    assert!(options.can_pick_up_skat && options.can_pass);
    assert_eq!(
        options.risks.iter().map(|r| r.risk).collect::<Vec<_>>(),
        vec![
            RiskLevel::Hand,
            RiskLevel::SchneiderDeclared,
            RiskLevel::SchwarzDeclared,
            RiskLevel::Ouvert
        ]
    );
    assert!(options.risks.iter().all(|r| r.allowed));
    assert!(options.games.iter().all(|g| g.allowed));
    // A hand game goes straight to the tricks, the skat stays hidden.
    run.apply(&content, RunAction::Declare(diamonds(RiskLevel::Hand)))
        .expect("hand");
    let view = run.view(&content).game.expect("game");
    assert_eq!(view.phase, TablePhase::Playing);
    assert!(view.skat.is_none());
}

#[test]
fn the_pusher_leaves_only_the_hand_way() {
    let content = content();
    let mut run = relic_run(&content, &[]);
    run.series.boss = Some(BossId::new("the_pusher"));
    fixed_deal(&mut run, &content, STRONG, STRONG_SKAT);
    assert_eq!(
        run.clone().apply(&content, RunAction::PickUpSkat),
        Err(RunError::HandRequired)
    );
    let options = run
        .view(&content)
        .game
        .expect("game")
        .declare
        .expect("options");
    assert!(!options.can_pick_up_skat);
    assert!(options.hand_required);
    assert!(options.risks.iter().all(|r| r.allowed));
    // The bot plays the hand way as well.
    let action = bot::RunBot.next_action(&run, &content).expect("action");
    assert!(
        matches!(action, RunAction::Declare(d) if d.hand()),
        "{action:?}"
    );
}

#[test]
fn the_null_player_picks_up_for_null_and_null_ouvert() {
    let content = content();
    let start = RunStart {
        character: Character::NullPlayer,
        ..RunStart::default()
    };
    let (mut run, _) = Run::new_with(&content, 3, &start).expect("start");
    let options = run
        .view(&content)
        .game
        .expect("game")
        .declare
        .expect("options");
    assert_eq!(
        options
            .null_risks
            .iter()
            .map(|r| r.risk)
            .collect::<Vec<_>>(),
        vec![NullRisk::Hand, NullRisk::OuvertHand]
    );
    assert!(options.can_pick_up_skat);
    run.apply(&content, RunAction::PickUpSkat).expect("pick up");
    let hand = run.game().expect("game").game.hand(Seat::DECLARER);
    let discard = [hand[0].id, hand[1].id];
    run.apply(&content, RunAction::Discard(discard))
        .expect("discard");
    let options = run
        .view(&content)
        .game
        .expect("game")
        .declare
        .expect("options");
    assert_eq!(
        options
            .null_risks
            .iter()
            .map(|r| (r.risk, r.allowed, r.value))
            .collect::<Vec<_>>(),
        vec![(NullRisk::Normal, true, 23), (NullRisk::Ouvert, true, 69)]
    );
    assert!(!run.accepts(
        &content,
        RunAction::Declare(Declaration::null(NullRisk::OuvertHand))
    ));
    run.apply(
        &content,
        RunAction::Declare(Declaration::null(NullRisk::Ouvert)),
    )
    .expect("Null Ouvert after the skat");
}

#[test]
fn the_show_off_lets_the_null_player_pick_up_for_null_ouvert_only() {
    let content = content();
    let start = RunStart {
        character: Character::NullPlayer,
        relics: vec![RelicId::new("show_off")],
        ..RunStart::default()
    };
    let (mut run, _) = Run::new_with(&content, 3, &start).expect("start");
    run.apply(&content, RunAction::PickUpSkat).expect("pick up");
    let hand = run.game().expect("game").game.hand(Seat::DECLARER);
    let discard = [hand[0].id, hand[1].id];
    run.apply(&content, RunAction::Discard(discard))
        .expect("discard");
    let options = run
        .view(&content)
        .game
        .expect("game")
        .declare
        .expect("options");
    let allowed: Vec<(NullRisk, bool)> = options
        .null_risks
        .iter()
        .map(|r| (r.risk, r.allowed))
        .collect();
    assert_eq!(
        allowed,
        vec![(NullRisk::Normal, false), (NullRisk::Ouvert, true)]
    );
}

/// "Bockrunde" works at the real declaration: picking up the skat does not
/// use the Bock up, the declaration after the discard does.
#[test]
fn on_declare_effects_wait_for_the_declaration_after_the_skat() {
    let content = content();
    let mut run = relic_run(&content, &["bock_round"]);
    run.flags.insert("bock".into());
    fixed_deal(&mut run, &content, STRONG, STRONG_SKAT);
    let events = run.apply(&content, RunAction::PickUpSkat).expect("pick up");
    assert!(!events.contains(&RunEvent::FlagCleared("bock".into())));
    let events = run
        .apply(
            &content,
            RunAction::Discard([id(&run, "D8"), id(&run, "C7")]),
        )
        .expect("discard");
    assert!(!events.contains(&RunEvent::FlagCleared("bock".into())));
    assert!(run.flags().contains("bock"));
    let events = run
        .apply(&content, RunAction::Declare(diamonds(RiskLevel::Normal)))
        .expect("declare");
    assert!(events.contains(&RunEvent::FlagCleared("bock".into())));
    assert!(!run.flags().contains("bock"));
}

/// Kontra comes after the declaration, also after picking up the skat.
#[test]
fn kontra_follows_the_declaration_after_the_skat() {
    let content = content();
    let mut run = relic_run(&content, &["kontra"]);
    fixed_deal(&mut run, &content, WEAK, WEAK_SKAT);
    let events = run.apply(&content, RunAction::PickUpSkat).expect("pick up");
    assert!(!events.iter().any(|e| matches!(e, RunEvent::Kontra { .. })));
    run.apply(
        &content,
        RunAction::Discard([id(&run, "C8"), id(&run, "C9")]),
    )
    .expect("discard");
    let clubs = Declaration::suit(Suit::Clubs, RiskLevel::Normal);
    let events = run
        .apply(&content, RunAction::Declare(clubs))
        .expect("declare");
    assert!(events.contains(&RunEvent::Kontra {
        seat: Seat::DEFENDERS[0]
    }));
    let view = run.view(&content).game.expect("game");
    assert!(view.awaiting_kontra_answer && !view.your_turn);
    assert_eq!(view.phase, TablePhase::Playing);
    run.apply(&content, RunAction::AnswerKontra { re: false })
        .expect("answer");
}

/// A run saved in the old phase "Ansage vor Skat" (ten cards, nothing
/// declared) loads into the new decision and plays on.
#[test]
fn a_save_from_before_the_skat_order_plays_on() {
    let content = content();
    let (run, _) = Run::new(&content, 14).expect("start");
    let saved = ron::to_string(&run).expect("save");
    let mut loaded: Run = ron::from_str(&saved).expect("load");
    assert_eq!(loaded, run);
    let view = loaded.view(&content).game.expect("game");
    assert_eq!(view.phase, TablePhase::Declaring);
    assert!(view.declare.is_some_and(|d| d.can_pick_up_skat));
    for _ in 0..400 {
        let Some(action) = bot::RunBot.next_action(&loaded, &content) else {
            break;
        };
        loaded.apply(&content, action).expect("bot action");
    }
    assert!(loaded.stats().games_won + loaded.stats().games_lost > 0);
}

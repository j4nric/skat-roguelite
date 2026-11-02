//! Tests of the Schnäpse (DESIGN 8.3): Kurzer, Doppelkorn, Kräuter,
//! Magenbitter and Kreide.

use super::bot::RunBot;
use super::content_tests::{content, fixed_deal, play};
use super::*;
use crate::cards::Suit;
use crate::content::{ConsumableCategory, ConsumableId, Factor, ScoreTerm};
use crate::rules::{GameType, RiskLevel, is_trump};

/// Declarer holds all trumps of a diamonds game but the D8 (in the skat):
/// he wins every trick.
const STRONG: [&str; 3] = [
    "CJ SJ HJ DJ DA DT DK DQ D9 D7",
    "HA HT HK HQ H9 H8 H7 SA ST SK",
    "SQ S9 S8 S7 CA CT CK CQ C9 C8",
];
const STRONG_SKAT: &str = "D8 C7";

/// Declarer holds only Luschen: he loses any game.
const WEAK: [&str; 3] = [
    "D7 D8 D9 H7 H8 H9 S7 S8 S9 C7",
    "CJ SJ HJ DJ CA CT CK CQ DA DT",
    "DK DQ HA HT HK HQ SA ST SK SQ",
];
const WEAK_SKAT: &str = "C8 C9";

/// Like [`STRONG`], but the declarer's D7 is the H7: one heart to chalk.
const ONE_HEART: [&str; 3] = [
    "CJ SJ HJ DJ DA DT DK DQ D9 H7",
    "D7 HA HT HK HQ H9 H8 SA ST SK",
    "SQ S9 S8 S7 CA CT CK CQ C9 C8",
];

/// Aces and tens, no jack: weak for every game.
const ACES: [&str; 3] = [
    "CA CT HA HT SA ST DA DT SK HK",
    "CJ SJ HJ DJ CK CQ C9 C8 C7 D7",
    "SQ S9 S8 S7 HQ H9 H8 H7 DK DQ",
];
const ACES_SKAT: &str = "D9 D8";

fn hand(game: GameType) -> Declaration {
    Declaration::new(game, RiskLevel::Hand).expect("suit or grand")
}

const DIAMONDS: GameType = GameType::Suit(Suit::Diamonds);
const CLUBS: GameType = GameType::Suit(Suit::Clubs);

/// A fresh run in its first game with these consumables.
fn run_with(content: &Content, items: &[&str]) -> Run {
    let (mut run, _) = Run::new(content, 11).expect("start");
    run.consumables = items.iter().map(|c| ConsumableId::new(c)).collect();
    run
}

fn use_item(slot: usize) -> RunAction {
    RunAction::UseConsumable {
        slot,
        target: None,
        suit: None,
    }
}

fn source(id: &str) -> Source {
    Source::Consumable(ConsumableId::new(id))
}

fn hand_card(run: &Run, notation: &str) -> DealtCard {
    let kind: CardKind = notation.parse().expect("notation");
    run.game()
        .expect("game")
        .game
        .hand(Seat::DECLARER)
        .iter()
        .copied()
        .find(|c| c.kind == kind)
        .expect("in hand")
}

#[test]
fn the_five_schnaepse_load_as_one_group() {
    let content = content();
    for id in ["shot", "double_grain", "herbal", "bitters", "chalk"] {
        let def = content.consumable(&ConsumableId::new(id)).expect("loaded");
        assert_eq!(def.category, ConsumableCategory::Schnapps, "{id}");
        assert!((2..=4).contains(&def.price), "{id}");
        assert!(!def.text.is_empty(), "{id}");
    }
    let items = content
        .consumables
        .values()
        .filter(|d| d.category == ConsumableCategory::Item)
        .count();
    assert_eq!(items, 4, "the old consumables stay items");
    // They come to the Tresen like every other consumable.
    let mut seen = BTreeSet::new();
    for seed in 0..200 {
        let (mut run, _) = Run::new(&content, seed).expect("run");
        let shop = run.draw_offer(&content, 0);
        seen.extend(shop.consumables.into_iter().flatten());
    }
    for id in ["shot", "double_grain", "herbal", "bitters", "chalk"] {
        assert!(seen.contains(&ConsumableId::new(id)), "{id} never offered");
    }
}

#[test]
fn shot_shows_the_skat_before_skat_or_hand_and_keeps_the_choice_open() {
    let content = content();
    let mut run = run_with(&content, &["shot"]);
    fixed_deal(&mut run, &content, STRONG, STRONG_SKAT);
    let view = run.view(&content);
    assert!(view.consumables[0].can_use);
    assert!(view.game.as_ref().expect("game").skat_peek.is_none());
    let events = run.apply(&content, use_item(0)).expect("peek");
    let skat = run.game().expect("game").game.skat();
    assert!(events.contains(&RunEvent::SkatPeeked { skat }));
    assert!(events.contains(&RunEvent::ConsumableUsed {
        consumable: ConsumableId::new("shot")
    }));
    // The player event shows the same two cards.
    assert!(
        super::player::player_events(events).contains(&RunEvent::SkatPeeked { skat }),
        "the player may see the peeked skat"
    );
    let peek = run.view(&content).game.expect("game").skat_peek;
    let ids: Vec<u32> = peek.expect("peeked").iter().map(|c| c.card.id).collect();
    assert_eq!(ids, skat.map(|c| c.id).to_vec());
    // Both ways stay open.
    assert!(run.accepts(&content, RunAction::PickUpSkat));
    assert!(run.accepts(&content, RunAction::Declare(hand(DIAMONDS))));
    // Once the skat is picked up, a second Kurzer has nothing to show.
    run.consumables = vec![ConsumableId::new("shot")];
    run.apply(&content, RunAction::PickUpSkat).expect("pick up");
    assert_eq!(run.apply(&content, use_item(0)), Err(RunError::WrongPhase));
    assert!(run.view(&content).game.expect("game").skat_peek.is_none());
}

#[test]
fn shot_works_only_with_ten_cards_and_not_against_the_kibitz_killer() {
    let content = content();
    // Not at the Tresen.
    let mut run = run_with(&content, &["shot"]);
    run.game = None;
    run.phase = RunPhase::Shop;
    run.shop = Some(run.draw_offer(&content, 0));
    assert_eq!(run.apply(&content, use_item(0)), Err(RunError::WrongPhase));
    // Not after a hand declaration.
    let mut run = run_with(&content, &["shot"]);
    fixed_deal(&mut run, &content, STRONG, STRONG_SKAT);
    run.apply(&content, RunAction::Declare(hand(DIAMONDS)))
        .expect("hand");
    assert_eq!(run.apply(&content, use_item(0)), Err(RunError::WrongPhase));
    // Not against "Der Kiebitz-Mörder" (information is off).
    let mut run = run_with(&content, &["shot"]);
    fixed_deal(&mut run, &content, STRONG, STRONG_SKAT);
    run.game.as_mut().expect("game").rules.no_information = true;
    assert!(!run.view(&content).consumables[0].can_use);
    assert_eq!(run.apply(&content, use_item(0)), Err(RunError::WrongPhase));
}

#[test]
fn double_grain_doubles_the_next_declared_game_won_or_lost() {
    let content = content();
    // Plain game for comparison.
    let mut plain = run_with(&content, &[]);
    fixed_deal(&mut plain, &content, STRONG, STRONG_SKAT);
    let (_, base) = play(&mut plain, &content, hand(DIAMONDS));
    // Drunk before the deal is declared; Einmischen does not waste it.
    let mut run = run_with(&content, &["double_grain"]);
    run.apply(&content, use_item(0)).expect("drink");
    assert_eq!(run.view(&content).pending_next_game.len(), 1);
    run.apply(&content, RunAction::Redeal).expect("einmischen");
    assert_eq!(run.pending_next_game().len(), 1, "Einmischen keeps it");
    fixed_deal(&mut run, &content, STRONG, STRONG_SKAT);
    let (_, won) = play(&mut run, &content, hand(DIAMONDS));
    assert!(won.won);
    assert!(won.lines.contains(&ScoreLine {
        source: source("double_grain"),
        lever: Lever::GameValue,
        change: Change::Multiply(2),
    }));
    assert_eq!(won.value, 2 * base.value);
    assert!(run.pending_next_game().is_empty(), "used up");
    // Lost, it counts double as well: −2 × doubled value.
    let mut run = run_with(&content, &["double_grain"]);
    fixed_deal(&mut run, &content, WEAK, WEAK_SKAT);
    run.apply(&content, use_item(0)).expect("drink");
    let (_, lost) = play(&mut run, &content, hand(CLUBS));
    assert!(!lost.won);
    assert_eq!(lost.multiplier, Factor::from_int(2));
    assert_eq!(lost.score, -2 * i64::try_from(lost.value).expect("value"));
    // At the Tresen it waits for the next game; during play it is too late.
    let mut run = run_with(&content, &["double_grain", "double_grain"]);
    run.game = None;
    run.phase = RunPhase::Shop;
    run.shop = Some(run.draw_offer(&content, 0));
    run.apply(&content, use_item(0)).expect("at the Tresen");
    let mut run = run_with(&content, &["double_grain"]);
    fixed_deal(&mut run, &content, STRONG, STRONG_SKAT);
    run.apply(&content, RunAction::Declare(hand(DIAMONDS)))
        .expect("hand");
    assert_eq!(run.apply(&content, use_item(0)), Err(RunError::WrongPhase));
}

#[test]
fn herbal_adds_one_einmischen_in_the_series() {
    let content = content();
    let mut run = run_with(&content, &["herbal"]);
    let before = run.redeals_left(&content);
    let events = run.apply(&content, use_item(0)).expect("drink");
    assert_eq!(run.redeals_left(&content), before + 1);
    assert!(events.contains(&RunEvent::RedealsAdded {
        redeals: 1,
        redeals_left: before + 1
    }));
    // The game's own Einmischen go first, the extra one stays for the
    // series.
    for _ in 0..before {
        run.apply(&content, RunAction::Redeal).expect("einmischen");
    }
    assert_eq!(run.series().extra_redeals, 1);
    run.apply(&content, RunAction::Redeal)
        .expect("the extra one");
    assert_eq!(run.series().extra_redeals, 0);
    assert_eq!(run.redeals_left(&content), 0);
    // Old saves and events: "PassesAdded" and "ExtraPasses" read as the
    // new names.
    let old: RunEvent = ron::from_str("PassesAdded(passes: 1, passes_left: 3)").expect("old event");
    assert_eq!(
        old,
        RunEvent::RedealsAdded {
            redeals: 1,
            redeals_left: 3
        }
    );
    let old: crate::content::ConsumableAction =
        ron::from_str("ExtraPasses(1)").expect("old action");
    assert_eq!(old, crate::content::ConsumableAction::ExtraRedeals(1));
}

#[test]
fn bitters_makes_the_next_lost_game_count_single() {
    let content = content();
    let mut run = run_with(&content, &["bitters", "bitters"]);
    fixed_deal(&mut run, &content, WEAK, WEAK_SKAT);
    run.apply(&content, use_item(0)).expect("drink");
    // A second one while the first waits would be wasted.
    assert_eq!(run.apply(&content, use_item(0)), Err(RunError::WrongPhase));
    let (_, lost) = play(&mut run, &content, hand(CLUBS));
    assert!(!lost.won);
    let line = ScoreLine {
        source: source("bitters"),
        lever: Lever::LossFactor,
        change: Change::Set(1),
    };
    assert!(lost.lines.contains(&line));
    assert_eq!(lost.score, -i64::try_from(lost.value).expect("value"));
    let last = lost.steps().pop().expect("steps");
    assert_eq!(last.term, ScoreTerm::Line(line));
    assert_eq!(last.multiplier, lost.multiplier);
    // Won, it does nothing and is used up by the game anyway.
    let mut run = run_with(&content, &["bitters"]);
    fixed_deal(&mut run, &content, STRONG, STRONG_SKAT);
    run.apply(&content, use_item(0)).expect("drink");
    let (_, won) = play(&mut run, &content, hand(DIAMONDS));
    assert!(won.won);
    assert!(won.lines.iter().all(|l| l.lever != Lever::LossFactor));
    assert!(run.pending_next_game().is_empty());
}

#[test]
fn chalk_repaints_a_card_of_the_hand_until_the_game_ends() {
    let content = content();
    let mut run = run_with(&content, &["chalk"]);
    fixed_deal(&mut run, &content, ONE_HEART, STRONG_SKAT);
    let heart = hand_card(&run, "H7");
    let view = run.view(&content);
    let item = &view.consumables[0];
    assert!(item.consumable.needs_suit && !item.consumable.needs_target);
    assert_eq!(item.targets.len(), 6, "every card but the four jacks");
    assert!(item.targets.contains(&heart.id));
    // A card and a suit are needed; jacks and the own suit are refused.
    assert_eq!(
        run.apply(&content, use_item(0)),
        Err(RunError::TargetRequired)
    );
    let chalk = |id: u32, suit| RunAction::UseConsumable {
        slot: 0,
        target: Some(id),
        suit: Some(suit),
    };
    let jack = hand_card(&run, "CJ");
    assert_eq!(
        run.apply(&content, chalk(jack.id, Suit::Hearts)),
        Err(RunError::CannotChalk)
    );
    assert_eq!(
        run.apply(&content, chalk(heart.id, Suit::Hearts)),
        Err(RunError::CannotChalk)
    );
    let events = run
        .apply(&content, chalk(heart.id, Suit::Diamonds))
        .expect("chalk");
    let painted = hand_card(&run, "D7");
    assert_eq!(painted.id, heart.id);
    assert!(events.contains(&RunEvent::Game(Event::CardChalked {
        old: heart,
        new: painted
    })));
    let view = run.view(&content);
    let card = view
        .game
        .expect("game")
        .hand
        .into_iter()
        .find(|c| c.card.id == heart.id)
        .expect("in hand");
    assert_eq!(card.chalked, Some(Suit::Hearts));
    // In a diamonds game it is a trump: eleven trumps and no side card,
    // so the declarer has "mit 11".
    assert!(is_trump(DIAMONDS, painted));
    let (_, won) = play(&mut run, &content, hand(DIAMONDS));
    assert!(won.won);
    assert_eq!(won.matadors.count, 11);
    // The deck keeps the printed card; the next game knows nothing of it.
    let printed: CardKind = "H7".parse().expect("notation");
    assert_eq!(run.deck().get(heart.id).expect("deck").kind, printed);
    if let Some(active) = run.game() {
        assert!(active.chalked.is_empty());
    }
}

#[test]
fn chalk_only_before_the_declaration() {
    let content = content();
    let mut run = run_with(&content, &["chalk"]);
    fixed_deal(&mut run, &content, ONE_HEART, STRONG_SKAT);
    let heart = hand_card(&run, "H7");
    run.apply(&content, RunAction::Declare(hand(DIAMONDS)))
        .expect("hand");
    let action = RunAction::UseConsumable {
        slot: 0,
        target: Some(heart.id),
        suit: Some(Suit::Diamonds),
    };
    assert_eq!(run.apply(&content, action), Err(RunError::WrongPhase));
    assert!(!run.view(&content).consumables[0].can_use);
}

#[test]
fn the_bot_drinks_each_schnaps() {
    let content = content();
    // Kurzer right before picking up the skat (a weak hand is picked up
    // when nothing can throw it in: no Einmischen left).
    let mut run = run_with(&content, &["shot"]);
    fixed_deal(&mut run, &content, ACES, ACES_SKAT);
    // No Einmischen left: the hand is played.
    run.series.redeals_in_game = 99;
    assert_eq!(RunBot.next_action(&run, &content), Some(use_item(0)));
    run.apply(&content, use_item(0)).expect("peek");
    assert_eq!(
        RunBot.next_action(&run, &content),
        Some(RunAction::PickUpSkat)
    );
    // A strong game after the skat: Doppelkorn.
    let mut run = run_with(&content, &["double_grain"]);
    fixed_deal(&mut run, &content, ONE_HEART, STRONG_SKAT);
    run.apply(&content, RunAction::PickUpSkat).expect("pick up");
    assert_eq!(drunk_until_play(&mut run, &content), ["double_grain"]);
    // A weak game after the skat: Magenbitter.
    let mut run = run_with(&content, &["bitters"]);
    fixed_deal(&mut run, &content, ACES, ACES_SKAT);
    // No Einmischen left: the hand is played.
    run.series.redeals_in_game = 99;
    assert_eq!(drunk_until_play(&mut run, &content), ["bitters"]);
    // Kreide paints a side card into trump; with it the same hand is no
    // longer weak (five trumps, four aces), so Doppelkorn follows.
    let mut run = run_with(&content, &["chalk", "double_grain"]);
    fixed_deal(&mut run, &content, ACES, ACES_SKAT);
    // No Einmischen left: the hand is played.
    run.series.redeals_in_game = 99;
    assert_eq!(
        drunk_until_play(&mut run, &content),
        ["chalk", "double_grain"]
    );
    let active = run.game().expect("game");
    assert_eq!(active.chalked.len(), 1);
    let trump = active.game.declaration().expect("declared").game();
    let (id, _) = active.chalked[0];
    let card = active
        .game
        .declarer_cards()
        .iter()
        .find(|c| c.id == id)
        .expect("chalked card");
    assert!(is_trump(trump, *card), "painted into trump");
}

/// Let the bot act until the first card; the consumables it used.
fn drunk_until_play(run: &mut Run, content: &Content) -> Vec<String> {
    let mut used = Vec::new();
    for _ in 0..10 {
        let Some(action) = RunBot.next_action(run, content) else {
            break;
        };
        if matches!(action, RunAction::Play(_)) {
            break;
        }
        let events = run.apply(content, action).expect("legal");
        used.extend(events.iter().filter_map(|e| match e {
            RunEvent::ConsumableUsed { consumable } => Some(consumable.as_str().to_owned()),
            _ => None,
        }));
    }
    used
}

#[test]
fn old_saves_without_the_schnaps_fields_still_load() {
    let content = content();
    let (run, _) = Run::new(&content, 13).expect("run");
    let mut text = ron::to_string(&run).expect("ron");
    for field in [
        ",pending_next_game:[]",
        ",skat_peeked:false",
        ",chalked:[]",
        ",single_loss:None",
    ] {
        assert!(text.contains(field), "{field}");
        text = text.replacen(field, "", 1);
    }
    let back: Run = ron::from_str(&text).expect("old save loads");
    assert_eq!(back.series(), run.series());
    // An action without the suit still reads.
    let action: RunAction = ron::from_str("UseConsumable(slot:0,target:None)").expect("action");
    assert_eq!(action, use_item(0));
}

// ---------------------------------------------------------------------------
// Reue (01.11.2026, with the Einmischen rule)
// ---------------------------------------------------------------------------

#[test]
fn regret_brings_back_the_deal_before_the_last_einmischen() {
    let content = content();
    let def = content
        .consumable(&ConsumableId::new("regret"))
        .expect("regret");
    assert_eq!(def.category, ConsumableCategory::Schnapps);
    let mut run = run_with(&content, &["regret"]);
    // Nothing to take back before an Einmischen.
    assert_eq!(
        run.apply(&content, use_item(0)),
        Err(RunError::NoRedealToUndo)
    );
    assert!(!run.view(&content).consumables[0].can_use);
    let first = run.game().expect("game").clone();
    let left = run.redeals_left(&content);
    run.apply(&content, RunAction::Redeal).expect("einmischen");
    let second = run.game().expect("game").game.clone();
    assert_ne!(second.hand(Seat::DECLARER), first.game.hand(Seat::DECLARER));
    assert!(run.view(&content).consumables[0].can_use);
    // A save keeps the deal before.
    let text = ron::to_string(&run).expect("ron");
    let mut run: Run = ron::from_str(&text).expect("load");
    let events = run.apply(&content, use_item(0)).expect("regret");
    assert!(events.contains(&RunEvent::RedealUndone));
    let back = run.game().expect("game");
    assert_eq!(back.game, first.game, "the same deal as before");
    assert_eq!(back.previous, None);
    // The Einmischen stays used; no second undo.
    assert_eq!(run.redeals_left(&content), left - 1);
    run.consumables = vec![ConsumableId::new("regret")];
    assert_eq!(
        run.apply(&content, use_item(0)),
        Err(RunError::NoRedealToUndo)
    );
    // Only with ten cards before the skat.
    run.apply(&content, RunAction::Redeal).expect("einmischen");
    run.apply(&content, RunAction::PickUpSkat).expect("pick up");
    assert_eq!(run.apply(&content, use_item(0)), Err(RunError::WrongPhase));
    // Deterministic: the same actions give the same run.
    let replay = |content: &Content| {
        let mut run = run_with(content, &["regret"]);
        run.apply(content, RunAction::Redeal).expect("einmischen");
        run.apply(content, use_item(0)).expect("regret");
        run.apply(content, RunAction::Redeal).expect("einmischen");
        run
    };
    assert_eq!(replay(&content), replay(&content));
}

#[test]
fn saves_from_before_the_einmischen_rule_load() {
    let content = content();
    let (run, _) = Run::new(&content, 13).expect("run");
    let mut text = ron::to_string(&run).expect("ron");
    // An old series: 8 games, passes left and used, no Einmischen fields.
    for (new, old) in [
        ("game_limit:3,", "game_limit:8,passes_left:4,"),
        ("redeals_used:0,", "redeals_used:0,passes_used:2,"),
        (",redeals_in_game:0", ""),
        (",extra_redeals:0", ""),
        (",redeals:0,", ",games_passed:2,"),
        ("extra_redeals:0", "series_redeals:0"),
    ] {
        assert!(text.contains(new), "{new}");
        text = text.replacen(new, old, 1);
    }
    let back: Run = ron::from_str(&text).expect("old save loads");
    // The series keeps its limit; Einmischen follows the new rule.
    assert_eq!(back.series().game_limit, 8);
    assert_eq!(back.redeals_left(&content), content.run.redeals);
    assert_eq!(back.stats().redeals, 2);
    // Old events read under the new names.
    let won: RunEvent = ron::from_str(
        "SeriesWon(section:0,index:2,payout:(base:3,games_left:0,interest:0),coins:5,boss:None,passes_used:1,games_left:0)",
    )
    .expect("old event");
    assert!(matches!(
        won,
        RunEvent::SeriesWon {
            redeals_used: 1,
            ..
        }
    ));
}

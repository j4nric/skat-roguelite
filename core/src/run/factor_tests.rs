//! The Faktor as the third stage of the score (ADR 0018): the relics of
//! the family "Faktor", their conditions (won games only, the trump suit,
//! Grand, Null), multiplicative stacking, rounding, growing counters and
//! their place in saves and in the value preview.

use super::content_tests::{
    CLUBS, DIAMONDS, STRONG, STRONG_SKAT, WEAK, WEAK_SKAT, content, declare, fixed_deal, hand_game,
    has_line, play, relic, relic_run,
};
use super::*;
use crate::content::per_step_factor;
use crate::content::{Factor, Rarity};
use crate::rules::{GameType, NullRisk};

/// [`STRONG`] with clubs as the long suit: the declarer takes every trick
/// of a clubs game.
const STRONG_CLUBS: [&str; 3] = [
    "CJ SJ HJ DJ CA CT CK CQ C9 C7",
    "HA HT HK HQ H9 H8 H7 SA ST SK",
    "SQ S9 S8 S7 DA DT DK DQ D9 D8",
];
const STRONG_CLUBS_SKAT: &str = "C8 D7";

/// The declarer holds the lowest cards of every suit: a Null he cannot
/// lose.
const LOW: [&str; 3] = [
    "D7 D8 D9 H7 H8 H9 S7 S8 S9 C7",
    "CJ SJ HJ DJ CA CT CK CQ DA DT",
    "DK DQ HA HT HK HQ SA ST SK SQ",
];
const LOW_SKAT: &str = "C8 C9";

fn long_run(content: &Content, relics: &[&str]) -> Run {
    let mut run = relic_run(content, relics);
    run.series.target = 1_000_000;
    run.series.game_limit = 100;
    run
}

fn scale(permille: u32) -> Change {
    Change::Scale(Factor::from_permille(permille))
}

#[test]
fn every_factor_relic_is_rare_or_legendary_in_its_own_group() {
    let content = content();
    for id in [
        "jay",
        "tree_frog",
        "robin",
        "bell_tree",
        "grand_seigneur",
        "black_zero",
        "tap",
        "hot_hand",
    ] {
        let def = content.relic(&RelicId::new(id)).expect(id);
        assert_eq!(def.group, crate::content::RelicGroup::Factor, "{id}");
        assert!(
            matches!(def.rarity, Rarity::Rare | Rarity::Legendary),
            "{id}"
        );
        assert!(def.text.starts_with('×') || def.text.contains("+×"), "{id}");
    }
}

#[test]
fn a_suit_relic_scales_a_won_game_of_its_trump_suit_only() {
    let content = content();
    let mut run = long_run(&content, &["jay", "bell_tree"]);
    fixed_deal(&mut run, &content, STRONG, STRONG_SKAT);
    let (_, b) = play(&mut run, &content, hand_game(DIAMONDS));
    assert!(b.won);
    assert!(has_line(
        &b,
        &relic("bell_tree"),
        Lever::GameValue,
        scale(1500)
    ));
    assert!(b.lines_of(&relic("jay")).next().is_none());
    assert_eq!(b.multiplier, Factor::from_permille(1500));
    // Grundwert × Stufen × 1,5, rounded half up once at the end.
    let plain = b.base_value * b.levels;
    assert_eq!(b.value, (plain * 1500 + 500) / 1000);

    fixed_deal(&mut run, &content, STRONG_CLUBS, STRONG_CLUBS_SKAT);
    let (_, b) = play(&mut run, &content, hand_game(CLUBS));
    assert!(b.won);
    assert!(has_line(&b, &relic("jay"), Lever::GameValue, scale(1500)));
    assert!(b.lines_of(&relic("bell_tree")).next().is_none());
}

#[test]
fn factor_relics_do_not_count_on_a_lost_game() {
    let content = content();
    let mut run = long_run(&content, &["jay", "tap", "hot_hand"]);
    run.counters.insert("tap".into(), 5);
    run.counters.insert("hot_hand".into(), 3);
    fixed_deal(&mut run, &content, WEAK, WEAK_SKAT);
    let (_, b) = play(&mut run, &content, hand_game(CLUBS));
    assert!(!b.won);
    assert_eq!(b.multiplier, Factor::ONE);
    assert!(b.lines.iter().all(|l| l.lever != Lever::GameValue));
    assert_eq!(b.score, -i64::try_from(b.value).expect("small"));
    // The loss resets "Heiße Hand", "Zapfhahn" stays.
    assert_eq!(run.counters().get("hot_hand"), None);
    assert_eq!(run.counters().get("tap"), Some(&5));
}

#[test]
fn factors_multiply_never_add() {
    let content = content();
    // ×1,5 from the suit and ×1,5 from five won pubs: ×2,25, not ×2.
    let mut run = long_run(&content, &["bell_tree", "tap"]);
    run.counters.insert("tap".into(), 5);
    fixed_deal(&mut run, &content, STRONG, STRONG_SKAT);
    let (_, b) = play(&mut run, &content, hand_game(DIAMONDS));
    assert!(has_line(&b, &relic("tap"), Lever::GameValue, scale(1500)));
    assert_eq!(b.multiplier, Factor::from_permille(2250));
    let plain = b.base_value * b.levels;
    assert_eq!(b.value, (plain * 2250 + 500) / 1000);
    // The score steps carry the running factor.
    let steps = b.steps();
    assert_eq!(
        steps.last().map(|s| s.multiplier),
        Some(Factor::from_permille(2250))
    );
}

#[test]
fn the_grand_relic_doubles_a_won_grand_and_the_null_relic_scales_a_won_null() {
    let content = content();
    let mut run = long_run(&content, &["grand_seigneur", "black_zero"]);
    fixed_deal(&mut run, &content, STRONG, STRONG_SKAT);
    let (_, b) = play(&mut run, &content, hand_game(GameType::Grand));
    assert!(b.won);
    assert!(has_line(
        &b,
        &relic("grand_seigneur"),
        Lever::GameValue,
        Change::Multiply(2)
    ));
    assert!(b.lines_of(&relic("black_zero")).next().is_none());
    assert_eq!(b.multiplier, Factor::from_int(2));

    fixed_deal(&mut run, &content, LOW, LOW_SKAT);
    let (_, b) = play(&mut run, &content, Declaration::null(NullRisk::Hand));
    assert!(b.won);
    assert!(has_line(
        &b,
        &relic("black_zero"),
        Lever::GameValue,
        scale(1500)
    ));
    assert!(b.lines_of(&relic("grand_seigneur")).next().is_none());
    let plain = b.base_value * b.levels;
    assert_eq!(b.value, (plain * 1500 + 500) / 1000);
}

#[test]
fn the_tap_grows_with_every_won_pub_for_the_rest_of_the_run() {
    let content = content();
    let mut run = relic_run(&content, &["tap"]);
    for expected in 1..=2u32 {
        run.series.games_played = run.series.game_limit;
        let mut events = Vec::new();
        run.series_won(&content, &mut events).expect("won");
        assert!(events.contains(&RunEvent::CounterRaised {
            counter: "tap".into(),
            value: expected,
        }));
    }
    let mut run = long_run(&content, &["tap"]);
    run.counters.insert("tap".into(), 2);
    fixed_deal(&mut run, &content, STRONG, STRONG_SKAT);
    let (_, b) = play(&mut run, &content, hand_game(DIAMONDS));
    assert!(has_line(&b, &relic("tap"), Lever::GameValue, scale(1200)));
    let plain = b.base_value * b.levels;
    assert_eq!(b.value, (plain * 1200 + 500) / 1000);
}

#[test]
fn the_hot_hand_grows_with_every_win_in_a_row_and_breaks_on_a_loss() {
    let content = content();
    let mut run = long_run(&content, &["hot_hand"]);
    let mut factors = Vec::new();
    for _ in 0..3 {
        fixed_deal(&mut run, &content, STRONG, STRONG_SKAT);
        let (_, b) = play(&mut run, &content, hand_game(DIAMONDS));
        assert!(b.won);
        factors.push(b.multiplier.permille());
    }
    // The factor counts the wins before the game: ×1, ×1,15, ×1,3.
    assert_eq!(factors, vec![1000, 1150, 1300]);
    assert_eq!(run.counters().get("hot_hand"), Some(&3));

    // The counter is part of the save.
    let json = serde_json::to_string(&run).expect("save");
    let loaded: Run = serde_json::from_str(&json).expect("load");
    assert_eq!(loaded.counters().get("hot_hand"), Some(&3));

    fixed_deal(&mut run, &content, WEAK, WEAK_SKAT);
    let (_, lost) = play(&mut run, &content, hand_game(CLUBS));
    assert!(!lost.won && lost.multiplier == Factor::ONE);
    assert_eq!(run.counters().get("hot_hand"), None);
    fixed_deal(&mut run, &content, STRONG, STRONG_SKAT);
    let (_, b) = play(&mut run, &content, hand_game(DIAMONDS));
    assert_eq!(b.multiplier, Factor::ONE);
}

#[test]
fn the_value_preview_shows_the_factor_of_a_win() {
    let content = content();
    let mut run = long_run(&content, &["bell_tree", "hot_hand"]);
    run.counters.insert("hot_hand".into(), 2);
    fixed_deal(&mut run, &content, STRONG, STRONG_SKAT);
    declare(&mut run, &content, hand_game(DIAMONDS)).expect("declare");
    let preview = run
        .view(&content)
        .game
        .expect("game")
        .value_preview
        .expect("preview");
    // ×1,5 × ×1,3.
    assert_eq!(preview.multiplier, Factor::from_permille(1950));
    assert_eq!(
        preview.game_value,
        Factor::from_permille(1950).scale(preview.base_value * preview.levels)
    );
}

/// Write `web/e2e/fixtures/factor-win.json`: the first game of a run with
/// Schellenbaum, Zapfhahn (three pubs) and Heiße Hand (two wins in a row),
/// dealt so that a Schellen Hand takes every trick; the smoke test plays
/// it and checks the Faktor box (×1,5 × ×1,3 × ×1,3). Run by hand: `cargo
/// test -p skat_core regenerate_factor_fixture -- --ignored`.
#[test]
#[ignore = "rewrites web/e2e/fixtures/factor-win.json"]
fn regenerate_factor_fixture() {
    let content = content();
    let mut run = relic_run(&content, &["bell_tree", "tap", "hot_hand"]);
    run.counters.insert("tap".into(), 3);
    run.counters.insert("hot_hand".into(), 2);
    fixed_deal(&mut run, &content, STRONG, STRONG_SKAT);
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../web/e2e/fixtures/factor-win.json");
    let out = serde_json::json!({ "version": 3, "run": run });
    std::fs::write(&path, serde_json::to_string(&out).expect("json")).expect("write");
    println!("wrote {}", path.display());
}

#[test]
fn a_growing_factor_stops_at_its_cap() {
    let per = Factor::from_permille(200);
    let max = Some(Factor::from_permille(3000));
    assert_eq!(per_step_factor(per, 4, max).permille(), 1800);
    assert_eq!(per_step_factor(per, 10, max).permille(), 3000);
    assert_eq!(per_step_factor(per, 25, max).permille(), 3000);
    assert_eq!(per_step_factor(per, 25, None).permille(), 6000);
    // The cap is optional in the data.
    let capped: crate::content::Effect = ron::from_str(
        r#"ScaleGameValuePerCounter(counter: "moonlight", per: 0.2, max: Some(3.0))"#,
    )
    .expect("ron");
    assert!(matches!(
        capped,
        crate::content::Effect::ScaleGameValuePerCounter { max: Some(m), .. } if m.permille() == 3000
    ));
    let open: crate::content::Effect =
        ron::from_str(r#"ScaleGameValuePerCard(count: (), per: 0.05)"#).expect("ron");
    assert!(matches!(
        open,
        crate::content::Effect::ScaleGameValuePerCard { max: None, .. }
    ));
}

#[test]
fn the_growing_relics_stop_at_their_caps() {
    let content = content();
    // Balance round of the night (05.11.2026): Zapfhahn and Heiße Hand
    // at most ×3, Mondschein at most ×2, Nachteule at most ×1,5.
    for (id, counter, value, permille) in [
        ("tap", "tap", 19, 2900),
        ("tap", "tap", 40, 3000),
        ("hot_hand", "hot_hand", 40, 3000),
        ("moonlight", "moonlight", 9, 1900),
        ("moonlight", "moonlight", 30, 2000),
        ("night_owl", "verschaerfungen", 12, 1500),
    ] {
        let mut run = long_run(&content, &[id]);
        run.counters.insert(counter.into(), value);
        fixed_deal(&mut run, &content, STRONG, STRONG_SKAT);
        let (_, b) = play(&mut run, &content, hand_game(DIAMONDS));
        assert!(
            has_line(&b, &relic(id), Lever::GameValue, scale(permille)),
            "{id} at {value}"
        );
    }
}

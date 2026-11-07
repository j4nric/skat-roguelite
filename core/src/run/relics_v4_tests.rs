//! Relics v4 (decided by the user on 07.11.2026): one test per new relic
//! and per new piece of the hook API, each in a constructed situation.
//! The reworked Treue Seele and Bierdeckel are tested in `build_tests` and
//! `content_tests`.

use super::bot::RunBot;
use super::content_tests::{
    CLUBS, DIAMONDS, STRONG, STRONG_SKAT, WEAK, WEAK_SKAT, content, fixed_deal, hand_game, play,
    relic, relic_run,
};
use super::view::RevealSource;
use super::wave3_tests::id_of;
use super::*;
use crate::content::{CardCount, Condition, Factor, HookContext, MalusId, evaluate};
use crate::rules::{
    GameType, Matadors, NullRisk, NullScoring, Outcome, RiskLevel, Thresholds, game_value_parts,
    matador_order,
};

/// Player forehand with all four Unter, the clubs top, D7 and H7.
const HANDS: [&str; 3] = [
    "CJ SJ HJ DJ CA CT CK C9 D7 H7",
    "HA HT HK HQ H9 H8 SA ST SK SQ",
    "S9 S8 S7 DA DT DK DQ D9 CQ C8",
];
const SKAT: &str = "D8 C7";

/// The declarer holds the lowest cards of every suit: a Null he cannot
/// lose.
const LOW: [&str; 3] = [
    "D7 D8 D9 H7 H8 H9 S7 S8 S9 C7",
    "CJ SJ HJ DJ CA CT CK CQ DA DT",
    "DK DQ HA HT HK HQ SA ST SK SQ",
];
const LOW_SKAT: &str = "C8 C9";

/// A run with these relics, a target it will not reach and room for many
/// games.
fn long_run(content: &Content, relics: &[&str]) -> Run {
    let mut run = relic_run(content, relics);
    run.series.target = 1_000_000;
    run.series.game_limit = 100;
    run
}

/// Put `card_mod` with its default value on the printed card `notation`.
fn put(run: &mut Run, content: &Content, notation: &str, card_mod: &str) {
    let id = CardModId::new(card_mod);
    let value = content
        .card_mod(&id)
        .map_or(1, crate::content::CardModDef::default_value);
    run.edition
        .add_mod(content, id_of(notation), &id, value)
        .expect("mod");
}

/// The changes of the lines `id` added to a breakdown.
fn changes(b: &ScoreBreakdown, id: &str) -> Vec<Change> {
    b.lines_of(&relic(id)).map(|l| l.change).collect()
}

/// Cards of the player's edition that carry `card_mod`.
fn with_mod(run: &Run, card_mod: &str) -> Vec<u32> {
    run.edition()
        .cards()
        .iter()
        .filter(|c| c.mods.iter().any(|m| m.as_str() == card_mod))
        .map(|c| c.id)
        .collect()
}

fn null(risk: NullRisk) -> Declaration {
    Declaration::Null { risk }
}

// ------------------------------------------------------------- Farbtreue

#[test]
fn fan_block_needs_three_siegel_on_the_trump_suit_and_none_beside() {
    let content = content();
    let game = |zinken: &[&str]| {
        let mut run = long_run(&content, &["fan_block"]);
        for card in zinken {
            put(&mut run, &content, card, "trump_bonus");
        }
        fixed_deal(&mut run, &content, STRONG, STRONG_SKAT);
        play(&mut run, &content, hand_game(DIAMONDS)).1
    };
    let b = game(&["DA", "DT", "DK"]);
    assert_eq!(changes(&b, "fan_block"), vec![Change::Add(3)]);
    // The Unter of the trump suit counts as the trump suit.
    let b = game(&["DA", "DT", "DJ"]);
    assert_eq!(changes(&b, "fan_block"), vec![Change::Add(3)]);
    // Two are too few, one Siegel beside spoils it.
    assert!(changes(&game(&["DA", "DT"]), "fan_block").is_empty());
    assert!(changes(&game(&["DA", "DT", "DK", "CA"]), "fan_block").is_empty());
    // An Unter of another suit is no card beside the trump suit.
    let b = game(&["DA", "DT", "DK", "CJ"]);
    assert_eq!(changes(&b, "fan_block"), vec![Change::Add(3)]);
}

#[test]
fn clear_cut_counts_the_void_side_suits_at_most_two() {
    let content = content();
    // STRONG in diamonds: no heart, spade or club: three, capped at two.
    let mut run = long_run(&content, &["clear_cut"]);
    fixed_deal(&mut run, &content, STRONG, STRONG_SKAT);
    let (_, b) = play(&mut run, &content, hand_game(DIAMONDS));
    assert_eq!(changes(&b, "clear_cut"), vec![Change::Add(2)]);
    // HANDS in clubs: D7 and H7 left, spades void: one.
    fixed_deal(&mut run, &content, HANDS, SKAT);
    let (_, b) = play(&mut run, &content, hand_game(CLUBS));
    assert_eq!(changes(&b, "clear_cut"), vec![Change::Add(1)]);
    // No suit game, no levels.
    fixed_deal(&mut run, &content, HANDS, SKAT);
    let (_, b) = play(&mut run, &content, hand_game(GameType::Grand));
    assert!(changes(&b, "clear_cut").is_empty());
}

// --------------------------------------------------------- Markenhändler

#[test]
fn pump_attendant_puts_a_pfandmarke_on_a_card_without_zinke() {
    let content = content();
    let mut run = relic_run(&content, &["pump_attendant"]);
    put(&mut run, &content, "DA", "points");
    let before = run.edition().clone();
    let mut again = run.clone();
    let mut events = Vec::new();
    run.fire(&content, Stage::OnSeriesWon, None, None, &mut events);
    let marked = with_mod(&run, "voucher_coins");
    assert_eq!(marked.len(), 1);
    let card = marked[0];
    assert!(before.get(card).expect("card").mods.is_empty());
    assert!(
        events
            .iter()
            .any(|e| matches!(e, RunEvent::DeckChanged(DeckChange::Modified { .. })))
    );
    // The same state draws the same card.
    let mut events = Vec::new();
    again.fire(&content, Stage::OnSeriesWon, None, None, &mut events);
    assert_eq!(with_mod(&again, "voucher_coins"), marked);
    // Nothing without a card free of modifications.
    let mut full = relic_run(&content, &["pump_attendant"]);
    let ids: Vec<u32> = full.edition().cards().iter().map(|c| c.id).collect();
    for id in ids {
        let _ = full
            .edition
            .add_mod(&content, id, &CardModId::new("stained"), 1);
    }
    let bare = full
        .edition()
        .cards()
        .iter()
        .filter(|c| c.mods.is_empty())
        .count();
    let mut events = Vec::new();
    full.fire(&content, Stage::OnSeriesWon, None, None, &mut events);
    assert_eq!(
        with_mod(&full, "voucher_coins").len(),
        usize::from(bare > 0)
    );
}

/// A run with a Pfandmarke (value 3) on the D9 in the hand of [`STRONG`].
fn voucher_run(content: &Content, relics: &[&str]) -> Run {
    let mut run = long_run(content, relics);
    run.edition
        .add_mod(content, id_of("D9"), &CardModId::new("voucher_coins"), 3)
        .expect("Pfandmarke");
    run
}

#[test]
fn rebate_king_scales_won_games_from_the_second_marke_of_the_pub() {
    let content = content();
    let mut run = voucher_run(&content, &["rebate_king"]);
    run.counters.insert("rebate_king".into(), 1);
    // The hand game redeems the second Marke: ×1,5 on this won game.
    fixed_deal(&mut run, &content, STRONG, STRONG_SKAT);
    let (_, b) = play(&mut run, &content, hand_game(DIAMONDS));
    assert_eq!(run.counters().get("rebate_king"), Some(&2));
    assert_eq!(
        changes(&b, "rebate_king"),
        vec![Change::Scale(Factor::from_permille(1500))]
    );
    // Lost games never.
    fixed_deal(&mut run, &content, WEAK, WEAK_SKAT);
    let (_, b) = play(&mut run, &content, hand_game(CLUBS));
    assert!(!b.won);
    assert!(changes(&b, "rebate_king").is_empty());
    // A won pub starts over.
    let mut events = Vec::new();
    run.fire(&content, Stage::OnSeriesWon, None, None, &mut events);
    assert_eq!(run.counters().get("rebate_king"), None);
    // One Marke is not enough.
    let mut run = long_run(&content, &["rebate_king"]);
    run.counters.insert("rebate_king".into(), 1);
    fixed_deal(&mut run, &content, STRONG, STRONG_SKAT);
    let (_, b) = play(&mut run, &content, hand_game(DIAMONDS));
    assert!(changes(&b, "rebate_king").is_empty());
}

#[test]
fn double_stamp_redeems_every_marke_twice() {
    let content = content();
    let redeem = |relics: &[&str]| {
        let mut run = voucher_run(&content, relics);
        fixed_deal(&mut run, &content, STRONG, STRONG_SKAT);
        let coins = run.coins();
        let events = run
            .apply(&content, RunAction::Declare(hand_game(DIAMONDS)))
            .expect("declare");
        let redeemed = events
            .iter()
            .filter(|e| matches!(e, RunEvent::VoucherRedeemed { .. }))
            .count();
        (
            run.coins() - coins,
            run.counters().get("rebate_book").copied(),
            redeemed,
        )
    };
    let (once, book_once, events_once) = redeem(&["rebate_book", "empties"]);
    let (twice, book_twice, events_twice) = redeem(&["rebate_book", "empties", "double_stamp"]);
    assert!(once > 0);
    assert_eq!(twice, 2 * once);
    assert_eq!((book_once, book_twice), (Some(1), Some(2)));
    // One Marke, redeemed once as an event.
    assert_eq!((events_once, events_twice), (1, 1));
    // The Marke is gone either way.
    let mut run = voucher_run(&content, &["double_stamp"]);
    fixed_deal(&mut run, &content, STRONG, STRONG_SKAT);
    run.apply(&content, RunAction::Declare(hand_game(DIAMONDS)))
        .expect("declare");
    assert!(with_mod(&run, "voucher_coins").is_empty());
}

// ------------------------------------------------------------ Stichjäger

#[test]
fn seven_at_one_blow_scales_a_won_game_with_seven_tricks() {
    let content = content();
    let mut run = long_run(&content, &["seven_at_one_blow"]);
    fixed_deal(&mut run, &content, STRONG, STRONG_SKAT);
    let (_, b) = play(&mut run, &content, hand_game(DIAMONDS));
    assert!(b.won && b.outcome.declarer_tricks >= 7);
    assert_eq!(
        changes(&b, "seven_at_one_blow"),
        vec![Change::Scale(Factor::from_permille(1500))]
    );
    fixed_deal(&mut run, &content, WEAK, WEAK_SKAT);
    let (_, b) = play(&mut run, &content, hand_game(CLUBS));
    assert!(changes(&b, "seven_at_one_blow").is_empty());
}

#[test]
fn declarer_tricks_at_least_counts_the_tricks_so_far() {
    let content = content();
    let mut run = long_run(&content, &[]);
    fixed_deal(&mut run, &content, STRONG, STRONG_SKAT);
    run.apply(&content, RunAction::Declare(hand_game(DIAMONDS)))
        .expect("declare");
    // The declarer takes every trick of STRONG: play three.
    while run.game().expect("game").game.tricks().len() < 3 {
        let action = RunBot.next_action(&run, &content).expect("action");
        run.apply(&content, action).expect("legal");
    }
    let game = &run.game().expect("game").game;
    let ctx = HookContext {
        flags: &run.flags,
        counters: &run.counters,
        game: Some(game),
        played: None,
        result: None,
        games_left: 0,
        transport: None,
        marked_cards: None,
        run: None,
    };
    let tricks = u32::try_from(game.tricks().len()).expect("small");
    assert!(evaluate(
        &Condition::DeclarerTricksAtLeast(tricks),
        &ctx,
        None
    ));
    assert!(!evaluate(
        &Condition::DeclarerTricksAtLeast(tricks + 1),
        &ctx,
        None
    ));
}

#[test]
fn upstart_gives_a_card_with_winkel_the_augen_of_its_new_rank() {
    let content = content();
    let points = |relics: &[&str], card: &str| {
        let mut run = relic_run(&content, relics);
        put(&mut run, &content, card, "rank");
        fixed_deal(&mut run, &content, STRONG, STRONG_SKAT);
        let id = id_of(card);
        run.game()
            .expect("game")
            .game
            .hand(Seat::DECLARER)
            .iter()
            .find(|c| c.id == id)
            .expect("in the hand")
            .card_points
    };
    // The König becomes the 10: 4 Augen, 10 with the relic.
    assert_eq!(points(&[], "DK"), 4);
    assert_eq!(points(&["upstart"], "DK"), 10);
    // The Neun becomes the Oberin: 0, 3, and 5 with the Alte Oberin.
    assert_eq!(points(&["upstart"], "D9"), 3);
    assert_eq!(points(&["upstart", "old_lady"], "D9"), 5);
    // A Daus stays a Daus.
    assert_eq!(points(&["upstart"], "DA"), 11);
}

#[test]
fn trick_after_trick_adds_a_level_for_a_run_of_three_once_per_game() {
    let content = content();
    let mut run = long_run(&content, &["trick_after_trick"]);
    fixed_deal(&mut run, &content, STRONG, STRONG_SKAT);
    let (_, b) = play(&mut run, &content, hand_game(DIAMONDS));
    assert_eq!(b.outcome.declarer_tricks, 10);
    assert_eq!(changes(&b, "trick_after_trick"), vec![Change::Add(1)]);
    // The counters start over with the next declaration.
    fixed_deal(&mut run, &content, WEAK, WEAK_SKAT);
    let (_, b) = play(&mut run, &content, hand_game(CLUBS));
    assert!(changes(&b, "trick_after_trick").is_empty());
}

// --------------------------------------------------------------- Schneider

#[test]
fn big_mouth_doubles_the_base_value_of_a_won_announced_schneider() {
    let content = content();
    let mut run = long_run(&content, &["big_mouth"]);
    fixed_deal(&mut run, &content, STRONG, STRONG_SKAT);
    let announced = Declaration::suit(Suit::Diamonds, RiskLevel::SchneiderDeclared);
    let (_, b) = play(&mut run, &content, announced);
    assert!(b.won);
    assert_eq!(changes(&b, "big_mouth"), vec![Change::Multiply(2)]);
    assert_eq!(b.base_value, 18);
    // Not without the announcement.
    fixed_deal(&mut run, &content, STRONG, STRONG_SKAT);
    let (_, b) = play(&mut run, &content, hand_game(DIAMONDS));
    assert!(changes(&b, "big_mouth").is_empty());
    // Won games only.
    fixed_deal(&mut run, &content, WEAK, WEAK_SKAT);
    let lost = Declaration::suit(Suit::Clubs, RiskLevel::SchneiderDeclared);
    let (_, b) = play(&mut run, &content, lost);
    assert!(!b.won);
    assert!(changes(&b, "big_mouth").is_empty());
}

#[test]
fn yardstick_adds_a_base_value_per_five_augen_above_ninety() {
    let content = content();
    let mut run = long_run(&content, &["yardstick"]);
    fixed_deal(&mut run, &content, STRONG, STRONG_SKAT);
    let (_, b) = play(&mut run, &content, hand_game(DIAMONDS));
    assert!(b.schneider);
    let steps = (b.outcome.declarer_card_points - 90) / 5;
    assert_eq!(
        changes(&b, "yardstick"),
        vec![Change::Add(i64::from(steps))]
    );
    // No Schneider win, nothing.
    fixed_deal(&mut run, &content, WEAK, WEAK_SKAT);
    let (_, b) = play(&mut run, &content, hand_game(CLUBS));
    assert!(changes(&b, "yardstick").is_empty());
}

// ------------------------------------------------------------ Nullspieler

#[test]
fn plumber_pours_lead_after_a_pub_with_a_won_null_three_times() {
    let content = content();
    let mut run = long_run(&content, &["plumber"]);
    let won_pub = |run: &mut Run| {
        let mut events = Vec::new();
        run.fire(&content, Stage::OnSeriesWon, None, None, &mut events);
    };
    // Two won Nulls in a pub: one Blei, after the pub.
    for _ in 0..2 {
        fixed_deal(&mut run, &content, LOW, LOW_SKAT);
        let (_, b) = play(&mut run, &content, null(NullRisk::Hand));
        assert!(b.won);
    }
    assert!(with_mod(&run, "lead").is_empty());
    won_pub(&mut run);
    let lead = with_mod(&run, "lead");
    assert_eq!(lead.len(), 1);
    // A pub with a suit game only does not.
    fixed_deal(&mut run, &content, STRONG, STRONG_SKAT);
    play(&mut run, &content, hand_game(DIAMONDS));
    won_pub(&mut run);
    assert_eq!(with_mod(&run, "lead"), lead);
    // At most three times in a run.
    for _ in 0..3 {
        fixed_deal(&mut run, &content, LOW, LOW_SKAT);
        play(&mut run, &content, null(NullRisk::Hand));
        won_pub(&mut run);
    }
    assert_eq!(with_mod(&run, "lead").len(), 3);
}

#[test]
fn loose_contact_lowers_the_short_circuit_to_fifty_five() {
    let content = content();
    let threshold = |run: &Run| match run.null_scoring(&content) {
        NullScoring::Formula(f) => f.short_circuit_points,
        NullScoring::Fixed => 0,
    };
    assert_eq!(threshold(&relic_run(&content, &[])), 61);
    assert_eq!(threshold(&relic_run(&content, &["loose_contact"])), 55);
}

#[test]
fn cards_on_table_opens_both_defender_hands_in_null_ouvert() {
    let content = content();
    let mut run = long_run(&content, &["cards_on_table"]);
    fixed_deal(&mut run, &content, LOW, LOW_SKAT);
    run.apply(&content, RunAction::Declare(null(NullRisk::OuvertHand)))
        .expect("declare");
    let active = run.game().expect("game");
    assert!(Seat::DEFENDERS.iter().all(|&s| active.shows_hand(s)));
    assert_eq!(active.peeked.len(), 20);
    let view = run.view(&content).game.expect("view");
    let open = view
        .revealed
        .iter()
        .filter(|r| r.source == RevealSource::Onlooker)
        .count();
    assert_eq!(open, 20);
    // Null Hand: closed.
    let mut run = long_run(&content, &["cards_on_table"]);
    fixed_deal(&mut run, &content, LOW, LOW_SKAT);
    run.apply(&content, RunAction::Declare(null(NullRisk::Hand)))
        .expect("declare");
    assert!(run.game().expect("game").peeked.is_empty());
}

// ------------------------------------------------------------------- Grand

#[test]
fn big_shots_count_the_daus_as_spitzen_in_a_grand() {
    let content = content();
    let spitzen = |relics: &[&str], game: GameType| {
        let mut run = long_run(&content, relics);
        fixed_deal(&mut run, &content, HANDS, SKAT);
        let (_, b) = play(&mut run, &content, hand_game(game));
        b.matadors
    };
    // CJ SJ HJ DJ and CA held, SA with a defender.
    assert_eq!(
        spitzen(&[], GameType::Grand),
        Matadors {
            with: true,
            count: 4
        }
    );
    assert_eq!(
        spitzen(&["big_shots"], GameType::Grand),
        Matadors {
            with: true,
            count: 5
        }
    );
    // A suit game keeps its trumps.
    assert_eq!(spitzen(&["big_shots"], CLUBS), spitzen(&[], CLUBS),);
    assert_eq!(matador_order(GameType::Grand, true).len(), 8);
    assert_eq!(matador_order(CLUBS, true), CLUBS.trump_order());
}

#[test]
fn safe_haven_keeps_the_unter_at_an_einmischen() {
    let content = content();
    let mut run = relic_run(&content, &["safe_haven"]);
    fixed_deal(&mut run, &content, STRONG, STRONG_SKAT);
    let before: Vec<u32> = run
        .game()
        .expect("game")
        .game
        .hand(Seat::DECLARER)
        .iter()
        .map(|c| c.id)
        .collect();
    run.apply(&content, RunAction::Redeal).expect("redeal");
    let after: Vec<u32> = run
        .game()
        .expect("game")
        .game
        .hand(Seat::DECLARER)
        .iter()
        .map(|c| c.id)
        .collect();
    for jack in ["CJ", "SJ", "HJ", "DJ"] {
        assert!(after.contains(&id_of(jack)), "{jack}");
    }
    assert_ne!(before, after);
    assert_eq!(after.len(), 10);
    // Every card once.
    let game = &run.game().expect("game").game;
    let mut all: Vec<u32> = Seat::ALL
        .iter()
        .flat_map(|&s| game.hand(s).iter().map(|c| c.id))
        .chain(game.skat().iter().map(|c| c.id))
        .collect();
    all.sort_unstable();
    all.dedup();
    assert_eq!(all.len(), 32);
    // Only the first Einmischen of a game keeps (balance v4): the second
    // deals like a plain run.
    assert_eq!(run.series().redeals_this_game, 1);
    let mut plain = run.clone();
    plain.relics.clear();
    run.apply(&content, RunAction::Redeal).expect("redeal");
    plain.apply(&content, RunAction::Redeal).expect("redeal");
    let hand = |run: &Run| -> Vec<u32> {
        run.game()
            .expect("game")
            .game
            .hand(Seat::DECLARER)
            .iter()
            .map(|c| c.id)
            .collect()
    };
    assert_eq!(hand(&run), hand(&plain));
    // The next game starts the count again.
    let mut run = long_run(&content, &["safe_haven"]);
    fixed_deal(&mut run, &content, STRONG, STRONG_SKAT);
    run.apply(&content, RunAction::Redeal).expect("redeal");
    assert_eq!(run.series().redeals_this_game, 1);
    play(&mut run, &content, hand_game(DIAMONDS));
    assert_eq!(run.series().redeals_this_game, 0);
}

#[test]
fn keeping_nothing_deals_as_before() {
    let editions = [0, 1, 2].map(|_| {
        Edition::standard()
            .cards()
            .iter()
            .map(|c| DealtCard::plain(c.id, c.kind))
            .collect::<Vec<_>>()
    });
    let mut a = Rng::from_seed(5);
    let mut b = Rng::from_seed(5);
    let plain = Game::deal_editions(&mut a, editions.clone(), Seat::DECLARER).expect("deal");
    let keeping = Game::deal_editions_keeping(&mut b, editions, Seat::DECLARER, &[]).expect("deal");
    assert_eq!(plain, keeping);
}

#[test]
fn beer_calm_halves_a_lost_grand() {
    let content = content();
    let mut run = long_run(&content, &["beer_calm"]);
    fixed_deal(&mut run, &content, WEAK, WEAK_SKAT);
    let (_, b) = play(&mut run, &content, hand_game(GameType::Grand));
    assert!(!b.won);
    assert_eq!(
        changes(&b, "beer_calm"),
        vec![Change::Scale(Factor::from_permille(500))]
    );
    let mut plain = long_run(&content, &[]);
    fixed_deal(&mut plain, &content, WEAK, WEAK_SKAT);
    let (_, full) = play(&mut plain, &content, hand_game(GameType::Grand));
    assert_eq!(b.score, (full.score - 1) / 2);
    // A won Grand stays as it is.
    fixed_deal(&mut run, &content, STRONG, STRONG_SKAT);
    let (_, b) = play(&mut run, &content, hand_game(GameType::Grand));
    assert!(b.won);
    assert!(changes(&b, "beer_calm").is_empty());
}

#[test]
fn ouvert_with_schneider_wins_with_the_open_visor() {
    let decl = Declaration::suit(Suit::Clubs, RiskLevel::Ouvert);
    let mit_2 = Matadors {
        with: true,
        count: 2,
    };
    // Schneider, not Schwarz: one trick for the defenders.
    let outcome = Outcome {
        declarer_card_points: 110,
        defender_card_points: 10,
        declarer_tricks: 9,
        defender_tricks: 1,
    };
    let plain = game_value_parts(decl, mit_2, &outcome, &Thresholds::default());
    assert!(!plain.won);
    let visor = Thresholds {
        ouvert_schneider: true,
        ..Thresholds::default()
    };
    let parts = game_value_parts(decl, mit_2, &outcome, &visor);
    assert!(parts.won && parts.schneider && !parts.schwarz);
    // Spitzen 2, Spiel, Hand, Schneider, angesagt, Ouvert: no Schwarz.
    assert_eq!(parts.levels.total(), 7);
    assert_eq!(
        (parts.levels.schwarz, parts.levels.schwarz_declared),
        (0, 0)
    );
    // Schwarz made counts once.
    let schwarz = Outcome {
        declarer_card_points: 120,
        defender_card_points: 0,
        declarer_tricks: 10,
        defender_tricks: 0,
    };
    let parts = game_value_parts(decl, mit_2, &schwarz, &visor);
    assert_eq!(parts.levels.total(), 8);
    // No Schneider: lost.
    let short = Outcome {
        declarer_card_points: 80,
        defender_card_points: 40,
        declarer_tricks: 7,
        defender_tricks: 3,
    };
    assert!(!game_value_parts(decl, mit_2, &short, &visor).won);
    // The relic sets the rule in every game.
    let content = content();
    let mut run = relic_run(&content, &["open_visor"]);
    fixed_deal(&mut run, &content, STRONG, STRONG_SKAT);
    assert!(run.game().expect("game").rules.thresholds.ouvert_schneider);
}

// ---------------------------------------------------------- Without tags

#[test]
fn empty_chair_adds_a_level_while_two_slots_are_free() {
    let content = content();
    assert_eq!(content.run.relic_slots, Some(8));
    let mut run = long_run(&content, &["empty_chair"]);
    assert_eq!(run.free_relic_slots(&content), 7);
    fixed_deal(&mut run, &content, STRONG, STRONG_SKAT);
    let (_, b) = play(&mut run, &content, hand_game(DIAMONDS));
    assert_eq!(changes(&b, "empty_chair"), vec![Change::Add(1)]);
    let six = [
        "empty_chair",
        "tour",
        "virgin",
        "concede",
        "skat_redraw",
        "card_counter",
    ];
    let mut run = long_run(&content, &six);
    assert_eq!(run.free_relic_slots(&content), 2);
    fixed_deal(&mut run, &content, STRONG, STRONG_SKAT);
    let (_, b) = play(&mut run, &content, hand_game(DIAMONDS));
    assert_eq!(changes(&b, "empty_chair"), vec![Change::Add(1)]);
    // One free slot is not enough.
    let mut run = long_run(&content, &[&six[..], &["kibitzer"]].concat());
    fixed_deal(&mut run, &content, STRONG, STRONG_SKAT);
    let (_, b) = play(&mut run, &content, hand_game(DIAMONDS));
    assert!(changes(&b, "empty_chair").is_empty());
    // Without a slot limit: nothing.
    let mut unlimited = content.clone();
    unlimited.run.relic_slots = None;
    assert_eq!(run.free_relic_slots(&unlimited), 0);
}

/// A run at the Tresen with these relics.
fn at_tresen(content: &Content, relics: &[&str]) -> Run {
    let mut run = relic_run(content, relics);
    run.game = None;
    run.phase = RunPhase::Shop;
    run.shop = Some(run.draw_offer(content, 0));
    run
}

#[test]
fn junk_dealer_counts_a_sold_relic_once_per_tresen() {
    let content = content();
    let mut run = at_tresen(&content, &["junk_dealer", "tour", "virgin", "concede"]);
    run.apply(&content, RunAction::SellRelic(1)).expect("sell");
    assert_eq!(run.counters().get("junk_dealer"), Some(&1));
    // Once per visit.
    run.apply(&content, RunAction::SellRelic(1)).expect("sell");
    assert_eq!(run.counters().get("junk_dealer"), Some(&1));
    // The next Tresen counts again.
    let mut events = Vec::new();
    run.fire(&content, Stage::OnShopOpened, None, None, &mut events);
    run.apply(&content, RunAction::SellRelic(1)).expect("sell");
    assert_eq!(run.counters().get("junk_dealer"), Some(&2));
    // The Grundwert counts in every game.
    let mut run = long_run(&content, &["junk_dealer"]);
    run.counters.insert("junk_dealer".into(), 2);
    fixed_deal(&mut run, &content, STRONG, STRONG_SKAT);
    let (_, b) = play(&mut run, &content, hand_game(DIAMONDS));
    assert_eq!(changes(&b, "junk_dealer"), vec![Change::Add(2)]);
    // Selling the Trödler himself counts nothing.
    let mut run = at_tresen(&content, &["junk_dealer", "tour"]);
    run.apply(&content, RunAction::SellRelic(0)).expect("sell");
    assert_eq!(run.counters().get("junk_dealer"), None);
}

#[test]
fn teetotaller_counts_skipped_card_rewards() {
    let content = content();
    let start = RunStart {
        relics: vec![RelicId::new("teetotaller")],
        ..RunStart::default()
    };
    let mut run = Run::new_with(&content, 5, &start).expect("start").0;
    run.series.games_played = 1;
    run.series.score = i64::try_from(run.series.target).expect("small");
    run.game = None;
    let mut events = Vec::new();
    run.after_game(&content, &mut events).expect("after game");
    if run.schnapps_reward().is_some() {
        run.apply(&content, RunAction::ChooseSchnappsReward { give_up: None })
            .expect("sell");
    }
    assert_eq!(run.phase(), RunPhase::CardReward);
    run.apply(&content, RunAction::SkipCardReward)
        .expect("skip");
    assert_eq!(run.counters().get("teetotaller"), Some(&1));
    // +1 Grundwert per skip in every game.
    let mut run = long_run(&content, &["teetotaller"]);
    run.counters.insert("teetotaller".into(), 2);
    fixed_deal(&mut run, &content, STRONG, STRONG_SKAT);
    let (_, b) = play(&mut run, &content, hand_game(DIAMONDS));
    assert_eq!(changes(&b, "teetotaller"), vec![Change::Add(2)]);
}

#[test]
fn dive_expert_adds_two_levels_under_a_malus() {
    let content = content();
    let mut run = long_run(&content, &["dive_expert"]);
    fixed_deal(&mut run, &content, STRONG, STRONG_SKAT);
    let (_, b) = play(&mut run, &content, hand_game(DIAMONDS));
    assert!(changes(&b, "dive_expert").is_empty());
    run.series.night_malus = Some(MalusId::new("no_redeal"));
    fixed_deal(&mut run, &content, STRONG, STRONG_SKAT);
    let (_, b) = play(&mut run, &content, hand_game(DIAMONDS));
    assert_eq!(changes(&b, "dive_expert"), vec![Change::Add(2)]);
    // Won games only.
    fixed_deal(&mut run, &content, WEAK, WEAK_SKAT);
    let (_, b) = play(&mut run, &content, hand_game(CLUBS));
    assert!(changes(&b, "dive_expert").is_empty());
}

#[test]
fn crib_sheet_adds_a_level_to_a_hand_game() {
    let content = content();
    let mut run = long_run(&content, &["crib_sheet"]);
    fixed_deal(&mut run, &content, STRONG, STRONG_SKAT);
    let (_, b) = play(&mut run, &content, hand_game(DIAMONDS));
    assert_eq!(changes(&b, "crib_sheet"), vec![Change::Add(1)]);
    // Not after the skat.
    fixed_deal(&mut run, &content, STRONG, STRONG_SKAT);
    let (_, b) = play(
        &mut run,
        &content,
        Declaration::suit(Suit::Diamonds, RiskLevel::Normal),
    );
    assert!(changes(&b, "crib_sheet").is_empty());
}

// ------------------------------------------------------ Hook API pieces

#[test]
fn card_count_reads_all_cards_no_jacks_and_off_trump() {
    let content = content();
    let mut run = long_run(&content, &[]);
    put(&mut run, &content, "CA", "trump_bonus");
    put(&mut run, &content, "SJ", "trump_bonus");
    fixed_deal(&mut run, &content, HANDS, SKAT);
    run.apply(&content, RunAction::Declare(hand_game(CLUBS)))
        .expect("declare");
    let count = |c: CardCount| run.count_cards(&content, &c);
    // Clubs in the hand: CJ SJ HJ DJ (trump Unter only CJ) CA CT CK C9.
    let in_hand = CardCount {
        trump_suit: true,
        in_hand: true,
        all_cards: true,
        ..CardCount::default()
    };
    assert_eq!(count(in_hand.clone()), 5);
    assert_eq!(
        count(CardCount {
            no_jacks: true,
            ..in_hand
        }),
        4
    );
    // Siegel off the trump suit: the SJ is an Unter, so none.
    let off = CardCount {
        mods: vec![CardModId::new("trump_bonus")],
        off_trump: true,
        ..CardCount::default()
    };
    assert_eq!(count(off.clone()), 0);
    let mut run2 = run.clone();
    put(&mut run2, &content, "D7", "trump_bonus");
    assert_eq!(run2.count_cards(&content, &off), 1);
    let ctx_holds = |run: &Run, min: Option<u32>, max: Option<u32>| {
        let ctx = HookContext {
            flags: &run.flags,
            counters: &run.counters,
            game: run.game.as_ref().map(|a| &a.game),
            played: None,
            result: None,
            games_left: 0,
            transport: None,
            marked_cards: None,
            run: Some((run, &content)),
        };
        evaluate(
            &Condition::CardsCounted {
                count: off.clone(),
                min,
                max,
            },
            &ctx,
            None,
        )
    };
    assert!(ctx_holds(&run, None, Some(0)));
    assert!(!ctx_holds(&run2, None, Some(0)));
    assert!(ctx_holds(&run2, Some(1), None));
}

#[test]
fn pub_malus_active_reads_the_pub() {
    let content = content();
    let mut run = relic_run(&content, &[]);
    let holds = |run: &Run| {
        let ctx = HookContext {
            flags: &run.flags,
            counters: &run.counters,
            game: None,
            played: None,
            result: None,
            games_left: 0,
            transport: None,
            marked_cards: None,
            run: Some((run, &content)),
        };
        evaluate(&Condition::PubMalusActive, &ctx, None)
    };
    assert!(!holds(&run));
    run.series.night_malus = Some(MalusId::new("no_redeal"));
    assert!(holds(&run));
}

#[test]
fn the_new_relics_are_in_the_content_with_texts() {
    let content = content();
    for id in [
        "fan_block",
        "clear_cut",
        "pump_attendant",
        "rebate_king",
        "double_stamp",
        "seven_at_one_blow",
        "upstart",
        "trick_after_trick",
        "big_mouth",
        "yardstick",
        "plumber",
        "loose_contact",
        "cards_on_table",
        "big_shots",
        "safe_haven",
        "beer_calm",
        "open_visor",
        "empty_chair",
        "junk_dealer",
        "teetotaller",
    ] {
        let def = content.relic(&RelicId::new(id)).expect(id);
        assert!(
            !def.name.is_empty() && !def.text.is_empty() && def.price > 0,
            "{id}"
        );
        assert!(super::bot::relic_value(&def.id) > 0, "{id}");
    }
}

#[test]
fn the_bot_skips_a_plain_card_reward_with_the_teetotaller() {
    let content = content();
    let start = RunStart::default();
    let mut run = Run::new_with(&content, 5, &start).expect("start").0;
    run.series.games_played = 1;
    run.series.score = i64::try_from(run.series.target).expect("small");
    run.game = None;
    let mut events = Vec::new();
    run.after_game(&content, &mut events).expect("after game");
    if run.schnapps_reward().is_some() {
        run.apply(&content, RunAction::ChooseSchnappsReward { give_up: None })
            .expect("sell");
    }
    assert_eq!(run.phase(), RunPhase::CardReward);
    let without = RunBot::card_reward_action(&run, &content);
    run.relics.push(RelicId::new("teetotaller"));
    let with = RunBot::card_reward_action(&run, &content);
    // Without a build direction every Zinke is plain: it skips.
    assert!(matches!(without, RunAction::TakeCardReward { .. }));
    assert_eq!(with, RunAction::SkipCardReward);
}

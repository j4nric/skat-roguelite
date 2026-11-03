//! Tests of the characters from 03.11.2026 (DESIGN 9.4 to 9.6): Der
//! Drückeberger, Die Schneiderin, Bruderherz. One test per rule, plus
//! their start, unlocks, the generic rule fields (`RevealDefenderHand`,
//! `relic_slots`, `payout_factor`) and old saves of removed characters.

use super::bot::RunBot;
use super::content_tests::{content, fixed_deal, play};
use super::meta::MetaState;
use super::view::RevealSource;
use super::*;
use crate::cards::Suit;
use crate::content::{CardModId, Effect, UnlockId};
use crate::rules::{GameType, NullRisk, RiskLevel};

/// Declarer holds the four jacks and six diamonds; the defenders hold no
/// jack and no diamond, so he takes every trick in diamonds and in grand.
const STRONG: [&str; 3] = [
    "CJ SJ HJ DJ DA DT DK DQ D9 D7",
    "HA HT HK HQ H9 H8 H7 SA ST SK",
    "SQ S9 S8 S7 CA CT CK CQ C9 C8",
];
const STRONG_SKAT: &str = "D8 C7";

fn start(character: Character) -> RunStart {
    RunStart {
        character,
        ..RunStart::default()
    }
}

fn character_run(content: &Content, character: Character) -> Run {
    let (run, _) = Run::new_with(content, 11, &start(character)).expect("start");
    run
}

fn hand(game: GameType) -> Declaration {
    Declaration::new(game, RiskLevel::Hand).expect("suit or grand")
}

fn character_lines(b: &ScoreBreakdown, character: Character) -> Vec<Change> {
    b.lines
        .iter()
        .filter(|l| l.source == Source::Character(character) && l.lever == Lever::Levels)
        .map(|l| l.change)
        .collect()
}

fn card(s: &str) -> crate::cards::Card {
    s.parse().expect("card")
}

fn id_of(run: &Run, s: &str) -> u32 {
    run.edition().id_of(card(s)).expect("card in the edition")
}

fn mods_of(run: &Run, s: &str) -> Vec<(String, i32)> {
    let deck_card = run.edition().get(id_of(run, s)).expect("card");
    deck_card
        .mods
        .iter()
        .map(|m| (m.to_string(), deck_card.mod_value(m)))
        .collect()
}

#[test]
fn every_character_loads_and_the_new_ones_start_locked() {
    let content = content();
    assert_eq!(Character::ALL.len(), 5);
    for character in Character::ALL {
        let def = content.character(character).expect("character in content");
        assert!(!def.name.is_empty() && !def.text.is_empty());
        assert!(!def.text.contains("Stufe"), "{character}: Multiplikator");
        assert_eq!(character.as_str(), character.to_string());
        assert_eq!(Character::from_id(character.as_str()), Some(character));
    }
    for removed in ["GrandPlayer", "Onlooker", "Landlady"] {
        assert_eq!(Character::from_id(removed), None);
    }
    let meta = MetaState::new();
    assert!(meta.character_unlocked(&content, Character::Declarer));
    for (character, unlock) in [
        (Character::Shirker, "full_skat"),
        (Character::Tailoress, "three_schneider"),
        (Character::HeartBrother, "true_colours"),
    ] {
        assert!(!meta.character_unlocked(&content, character));
        let entry = meta
            .view(&content)
            .characters
            .into_iter()
            .find(|c| c.character == character)
            .expect("entry");
        assert_eq!(entry.unlocked_by, Some(UnlockId::new(unlock)));
        let mut earned = MetaState::new();
        earned.unlocked.insert(UnlockId::new(unlock));
        assert!(earned.character_unlocked(&content, character));
    }
}

#[test]
fn each_character_has_its_back() {
    let content = content();
    let meta = MetaState::all_unlocked(&content).view(&content);
    let back = |c: Character| {
        meta.characters
            .iter()
            .find(|e| e.character == c)
            .map(|e| e.back.clone())
            .expect("entry")
    };
    assert_eq!(back(Character::Declarer), "distel");
    assert_eq!(back(Character::NullPlayer), "distel");
    assert_eq!(back(Character::Shirker), "shirker");
    assert_eq!(back(Character::Tailoress), "tailoress");
    assert_eq!(back(Character::HeartBrother), "heart_brother");
    let run = character_run(&content, Character::Tailoress);
    assert_eq!(run.view(&content).character.back, "tailoress");
}

// ---------------------------------------------------------------------------
// Der Drückeberger
// ---------------------------------------------------------------------------

#[test]
fn shirker_starts_with_vouchers_and_the_skat_keeper() {
    let content = content();
    let run = character_run(&content, Character::Shirker);
    for seven in ["C7", "S7", "H7", "D7"] {
        assert_eq!(mods_of(&run, seven), vec![("voucher_coins".into(), 2)]);
    }
    assert_eq!(mods_of(&run, "D8"), vec![("voucher_undead".into(), 1)]);
    assert_eq!(mods_of(&run, "H8"), vec![("voucher_wild".into(), 1)]);
    let marked = run
        .edition()
        .cards()
        .iter()
        .filter(|c| !c.mods.is_empty())
        .count();
    assert_eq!(marked, 6);
    assert_eq!(run.relics(), &[RelicId::new("skat_keeper")]);
}

#[test]
fn shirker_hand_games_get_no_hand_level() {
    let content = content();
    for game in [GameType::Suit(Suit::Diamonds), GameType::Grand] {
        let mut run = character_run(&content, Character::Shirker);
        fixed_deal(&mut run, &content, STRONG, STRONG_SKAT);
        let (_, b) = play(&mut run, &content, hand(game));
        assert!(b.won);
        assert_eq!(b.rule_levels.hand, 1);
        assert_eq!(
            character_lines(&b, Character::Shirker),
            vec![Change::Add(-1)]
        );
        assert_eq!(b.levels, u64::from(b.rule_levels.total()) - 1);
    }
    // Null Hand too; a game after the skat keeps its levels.
    let mut run = character_run(&content, Character::Shirker);
    fixed_deal(&mut run, &content, STRONG, STRONG_SKAT);
    let (_, b) = play(&mut run, &content, Declaration::null(NullRisk::Hand));
    assert_eq!(
        character_lines(&b, Character::Shirker),
        vec![Change::Add(-1)]
    );
    let mut run = character_run(&content, Character::Shirker);
    fixed_deal(&mut run, &content, STRONG, STRONG_SKAT);
    let normal = Declaration::new(GameType::Grand, RiskLevel::Normal).expect("grand");
    let (_, b) = play(&mut run, &content, normal);
    assert!(character_lines(&b, Character::Shirker).is_empty());
}

#[test]
fn shirker_vouchers_come_back_after_the_pub() {
    let content = content();
    let mut run = character_run(&content, Character::Shirker);
    fixed_deal(&mut run, &content, STRONG, STRONG_SKAT);
    let mut events = run.apply(&content, RunAction::PickUpSkat).expect("skat");
    let (d8, c7) = (id_of(&run, "D8"), id_of(&run, "C7"));
    events.extend(
        run.apply(&content, RunAction::Discard([d8, c7]))
            .expect("discard"),
    );
    assert!(events.iter().any(|e| matches!(
        e,
        RunEvent::VoucherRedeemed { card, .. } if *card == c7
    )));
    // Redeemed: the Groschen paid, D8 is Untot now, the vouchers are off.
    assert!(mods_of(&run, "C7").is_empty());
    assert_eq!(mods_of(&run, "D8"), vec![("undead".into(), 1)]);
    assert_eq!(run.series.returning_vouchers.len(), 2);
    // An Aufdruck lands on C7 before the pub ends: that voucher stays gone.
    run.edition
        .add_mod(&content, c7, &CardModId::new("points"), 3)
        .expect("points");
    let mut events = Vec::new();
    run.pub_lost(&content, &mut events).expect("pub over");
    assert!(events.contains(&RunEvent::VoucherReturned {
        card: d8,
        card_mod: CardModId::new("voucher_undead"),
        value: 1,
    }));
    assert!(
        !events
            .iter()
            .any(|e| matches!(e, RunEvent::VoucherReturned { card, .. } if *card == c7))
    );
    assert_eq!(
        mods_of(&run, "D8"),
        vec![("undead".into(), 1), ("voucher_undead".into(), 1)]
    );
    assert_eq!(mods_of(&run, "C7"), vec![("points".into(), 3)]);
    assert!(run.series.returning_vouchers.is_empty());
}

#[test]
fn other_characters_keep_their_vouchers_redeemed() {
    let content = content();
    let mut run = character_run(&content, Character::Declarer);
    let c7 = id_of(&run, "C7");
    run.edition
        .add_mod(&content, c7, &CardModId::new("voucher_coins"), 3)
        .expect("voucher");
    fixed_deal(&mut run, &content, STRONG, STRONG_SKAT);
    run.apply(&content, RunAction::PickUpSkat).expect("skat");
    let d8 = id_of(&run, "D8");
    run.apply(&content, RunAction::Discard([d8, c7]))
        .expect("discard");
    assert!(run.series.returning_vouchers.is_empty());
    let mut events = Vec::new();
    run.pub_lost(&content, &mut events).expect("pub over");
    assert!(mods_of(&run, "C7").is_empty());
}

// ---------------------------------------------------------------------------
// Die Schneiderin
// ---------------------------------------------------------------------------

#[test]
fn tailoress_starts_with_points_on_the_tens_and_watertight() {
    let content = content();
    let run = character_run(&content, Character::Tailoress);
    for ten in ["CT", "ST", "HT", "DT"] {
        assert_eq!(mods_of(&run, ten), vec![("points".into(), 3)]);
    }
    assert_eq!(run.relics(), &[RelicId::new("watertight")]);
}

#[test]
fn tailoress_schneider_and_schwarz_count_two_levels() {
    let content = content();
    let mut run = character_run(&content, Character::Tailoress);
    fixed_deal(&mut run, &content, STRONG, STRONG_SKAT);
    let (_, b) = play(&mut run, &content, hand(GameType::Suit(Suit::Diamonds)));
    assert!(b.won && b.schneider && b.schwarz);
    assert_eq!(
        character_lines(&b, Character::Tailoress),
        vec![Change::Add(1), Change::Add(1)]
    );
    assert_eq!(b.levels, u64::from(b.rule_levels.total()) + 2);
}

#[test]
fn tailoress_win_without_schneider_counts_one_level_less() {
    let content = content();
    // He wins the four jack tricks and his clubs, then loses his four
    // low cards: won, but the defenders keep more than 30 Augen.
    let deal = [
        "CJ SJ HJ DJ CA CT D8 D7 S7 H7",
        "DA DT DK DQ SA ST SK HA HT HK",
        "D9 CK SQ S9 S8 HQ H9 H8 CQ C9",
    ];
    let mut run = character_run(&content, Character::Tailoress);
    fixed_deal(&mut run, &content, deal, "C8 C7");
    let (_, b) = play(&mut run, &content, hand(GameType::Grand));
    assert!(b.won && !b.schneider, "{:?}", b.outcome);
    assert_eq!(
        character_lines(&b, Character::Tailoress),
        vec![Change::Add(-1)]
    );
    // Null has no Schneider: unchanged.
    let mut run = character_run(&content, Character::Tailoress);
    fixed_deal(&mut run, &content, STRONG, STRONG_SKAT);
    let (_, b) = play(&mut run, &content, Declaration::null(NullRisk::Hand));
    assert!(character_lines(&b, Character::Tailoress).is_empty());
}

// ---------------------------------------------------------------------------
// Bruderherz
// ---------------------------------------------------------------------------

#[test]
fn heart_brother_starts_with_strong_hearts_and_cursed_suits() {
    let content = content();
    let run = character_run(&content, Character::HeartBrother);
    for c in ["HA", "HT", "HK", "HQ"] {
        assert_eq!(mods_of(&run, c), vec![("trump_bonus".into(), 2)]);
    }
    for c in ["H9", "H8", "H7"] {
        assert_eq!(mods_of(&run, c), vec![("rank".into(), 1)]);
    }
    for c in ["CA", "CT", "SA", "ST", "DA", "DT"] {
        assert_eq!(mods_of(&run, c), vec![("stained".into(), 2)]);
    }
    for c in ["CK", "SK", "DK"] {
        assert_eq!(mods_of(&run, c), vec![("battered".into(), 2)]);
    }
    for c in ["CJ", "SJ", "HJ", "DJ", "CQ", "C9", "D7"] {
        assert!(mods_of(&run, c).is_empty(), "{c}");
    }
    assert_eq!(run.relics(), &[RelicId::new("home_suit")]);
    // The curses paid nothing: "beim Erhalt" does not fire at the start.
    assert_eq!(run.coins(), content.run.economy.starting_coins);
}

#[test]
fn heart_brother_cannot_cleanse_a_curse() {
    let content = content();
    let mut run = character_run(&content, Character::HeartBrother);
    run.game = None;
    run.phase = RunPhase::Shop;
    run.shop = Some(run.draw_offer(&content, 0));
    run.coins = 20;
    // "Karte reinigen" at the Tresen (since the Zinken, DESIGN 8.6).
    let use_on = RunAction::CleanseCard;
    let ck = id_of(&run, "CK");
    let err = run.clone().apply(&content, use_on(ck));
    assert_eq!(err, Err(RunError::CurseStays));
    assert_eq!(RunError::CurseStays.code().as_str(), "CurseStays");
    // A card without a curse cleanses as usual.
    run.apply(&content, use_on(id_of(&run, "HA")))
        .expect("cleanse HA");
    assert!(mods_of(&run, "HA").is_empty());
    // Other characters cleanse curses.
    let mut plain = character_run(&content, Character::Declarer);
    plain
        .edition
        .add_mod(&content, ck, &CardModId::new("battered"), 2)
        .expect("curse");
    plain.game = None;
    plain.phase = RunPhase::Shop;
    plain.shop = Some(plain.draw_offer(&content, 0));
    plain.coins = 20;
    plain.apply(&content, use_on(ck)).expect("cleanse");
}

#[test]
fn heart_brother_pays_half_for_zinken_on_hearts() {
    let content = content();
    let run = character_run(&content, Character::HeartBrother);
    let (h9, s9) = (id_of(&run, "H9"), id_of(&run, "S9"));
    assert_eq!(run.zinken_price(&content, h9, 4), 2);
    assert_eq!(run.zinken_price(&content, h9, 3), 2);
    assert_eq!(run.zinken_price(&content, h9, 1), 1);
    assert_eq!(run.zinken_price(&content, h9, 0), 0);
    assert_eq!(run.zinken_price(&content, s9, 4), 4);
    let plain = character_run(&content, Character::Declarer);
    assert_eq!(plain.zinken_price(&content, h9, 4), 4);

    // Der Kartengeber: choosing a Rot card costs half, another full.
    let choose = |target| RunAction::ChooseEncounter {
        option: 0,
        target: Some(target),
    };
    let mut run = character_run(&content, Character::HeartBrother);
    run.coins = 2;
    run.set_encounter(
        "card_dealer",
        vec![super::encounters::test_option(
            4,
            vec![EncounterEffect::Mark {
                card: crate::content::CardPick::Chosen,
                card_mod: Some(CardModId::new("points")),
                value: Some(3),
                offers: Vec::new(),
                curse: false,
                curse_percent: 0,
            }],
        )],
    );
    let view = run.view(&content);
    let offer = &view.encounter.expect("encounter").options[0];
    assert!(offer.targets.contains(&h9) && !offer.targets.contains(&s9));
    let at = offer.targets.iter().position(|&c| c == h9).expect("H9");
    assert_eq!(offer.target_costs[at], 2);
    assert_eq!(
        run.clone().apply(&content, choose(s9)),
        Err(RunError::NotEnoughCoins)
    );
    run.apply(&content, choose(h9)).expect("half price");
    assert_eq!(run.coins(), 0);
    assert_eq!(mods_of(&run, "H9"), vec![("points".into(), 3)]);
}

// ---------------------------------------------------------------------------
// Generic rule fields (no character uses them since 03.11.2026)
// ---------------------------------------------------------------------------

#[test]
fn a_rule_may_show_a_defender_hand() {
    let mut content = content();
    let def = content
        .characters
        .get_mut(&Character::Declarer)
        .expect("declarer");
    def.rule.passive.push(Effect::RevealDefenderHand(1));
    let run = character_run(&content, Character::Declarer);
    let left = Seat::DEFENDERS[0];
    assert_eq!(run.game().expect("game").shown_hand, Some(left));
    let game = run.view(&content).game.expect("game");
    let shown = game
        .revealed
        .iter()
        .filter(|r| r.source == RevealSource::Onlooker)
        .count();
    assert_eq!(shown, 10);
}

#[test]
fn a_rule_may_change_slots_redeals_and_payout() {
    let mut content = content();
    let def = content
        .characters
        .get_mut(&Character::Declarer)
        .expect("declarer");
    def.rule.relic_slots = -1;
    def.rule.redeals = -5;
    def.rule.payout_factor = 2;
    assert_eq!(
        content.relic_slots(Character::Declarer),
        content.run.relic_slots - 1
    );
    assert_eq!(content.redeals(Character::Declarer), 0);
    let mut run = character_run(&content, Character::Declarer);
    run.coins = 12;
    run.series.games_played = 3;
    let mut events = Vec::new();
    run.series_won(&content, &mut events).expect("won");
    let payout = events
        .iter()
        .find_map(|e| match e {
            RunEvent::SeriesWon { payout, .. } => Some(*payout),
            _ => None,
        })
        .expect("series won");
    assert_eq!(
        payout.bonus,
        payout.base + payout.games_left + payout.interest
    );
}

#[test]
fn a_character_rule_is_validated() {
    let sources = crate::content::Sources::embedded();
    let load = |from: &str, to: &str| {
        assert!(sources.characters.contains(from));
        let characters = sources.characters.replace(from, to);
        Content::from_sources(&crate::content::Sources {
            characters: &characters,
            ..sources
        })
    };
    let rule = "keep_curses: true,";
    let relics = r#"start_relics: ["watertight"],"#;
    assert!(load(rule, rule).is_ok());
    for (from, bad) in [
        (rule, "keep_curses: true, passive: [RevealDefenderHand(0)],"),
        (rule, "keep_curses: true, passive: [GainCoins(1)],"),
        (rule, "keep_curses: true, payout_factor: 0,"),
        ("percent: 50", "percent: 0"),
        ("percent: 50", "percent: 101"),
        (relics, r#"start_relics: ["nope"],"#),
        (
            relics,
            r#"start_relics: ["watertight"], start_consumables: ["nope"],"#,
        ),
        (
            relics,
            r#"start_relics: ["watertight"], deck: (mods: [("CJ", "nope", 1)]),"#,
        ),
    ] {
        assert!(load(from, bad).is_err(), "{bad} must be rejected");
    }
}

// ---------------------------------------------------------------------------
// Unlocks, stakes and old saves
// ---------------------------------------------------------------------------

fn scored(declaration: Declaration, won: bool, schneider: bool, discarded: u32) -> RunEvent {
    let mut breakdown = crate::content::score_game(
        declaration,
        crate::rules::Matadors {
            with: true,
            count: 1,
        },
        &crate::rules::Outcome {
            declarer_card_points: if won { 95 } else { 40 },
            defender_card_points: if won { 25 } else { 80 },
            declarer_tricks: 8,
            defender_tricks: 2,
        },
        &crate::rules::Thresholds::default(),
        Vec::new(),
    );
    breakdown.won = won;
    breakdown.schneider = schneider;
    RunEvent::GameScored {
        breakdown,
        series_score: 0,
        discarded_points: discarded,
    }
}

fn pub_started(section: u32, index: u32) -> RunEvent {
    RunEvent::SeriesStarted {
        section,
        index,
        target: 30,
        game_limit: 3,
        redeals: 2,
        boss: None,
        malus: None,
    }
}

fn suit(s: Suit) -> Declaration {
    Declaration::new(GameType::Suit(s), RiskLevel::Normal).expect("suit")
}

fn grand() -> Declaration {
    Declaration::new(GameType::Grand, RiskLevel::Normal).expect("grand")
}

#[test]
fn shirker_unlocks_with_21_augen_discarded() {
    let content = content();
    let mut meta = MetaState::new();
    meta.observe(&content, &[scored(grand(), true, false, 20)]);
    meta.observe(&content, &[scored(grand(), false, false, 22)]);
    meta.observe(
        &content,
        &[scored(Declaration::null(NullRisk::Normal), true, false, 22)],
    );
    assert!(!meta.character_unlocked(&content, Character::Shirker));
    let earned = meta.observe(&content, &[scored(grand(), true, false, 21)]);
    assert!(earned.contains(&UnlockId::new("full_skat")), "{earned:?}");
    assert!(meta.character_unlocked(&content, Character::Shirker));
}

#[test]
fn discarded_points_come_from_the_game() {
    let content = content();
    let mut run = character_run(&content, Character::Declarer);
    fixed_deal(
        &mut run,
        &content,
        [
            "CJ SJ HJ DJ DA DT DK DQ D9 D7",
            "HK HQ H9 H8 H7 SA ST SK SQ S9",
            "S8 S7 CK CQ C9 C8 C7 D8 HT CA",
        ],
        "HA CT",
    );
    run.apply(&content, RunAction::PickUpSkat).expect("skat");
    let ids = [id_of(&run, "HA"), id_of(&run, "CT")];
    let mut events = run
        .apply(&content, RunAction::Discard(ids))
        .expect("discard");
    let decl = suit(Suit::Diamonds);
    events.extend(
        run.apply(&content, RunAction::Declare(decl))
            .expect("declare"),
    );
    while !events
        .iter()
        .any(|e| matches!(e, RunEvent::GameScored { .. }))
    {
        let action = RunBot.next_action(&run, &content).expect("action");
        events.extend(run.apply(&content, action).expect("legal"));
    }
    let discarded = events.iter().find_map(|e| match e {
        RunEvent::GameScored {
            discarded_points, ..
        } => Some(*discarded_points),
        _ => None,
    });
    assert_eq!(discarded, Some(21));
    // A hand game discards nothing.
    let mut run = character_run(&content, Character::Declarer);
    fixed_deal(&mut run, &content, STRONG, STRONG_SKAT);
    let (events, _) = play(&mut run, &content, hand(GameType::Grand));
    assert!(events.iter().any(|e| matches!(
        e,
        RunEvent::GameScored {
            discarded_points: 0,
            ..
        }
    )));
}

#[test]
fn tailoress_unlocks_with_three_schneider_wins_in_one_pub() {
    let content = content();
    let mut meta = MetaState::new();
    let win = scored(grand(), true, true, 0);
    meta.observe(&content, &[pub_started(0, 0), win.clone(), win.clone()]);
    // A new pub starts the count over.
    meta.observe(&content, &[pub_started(0, 1), win.clone(), win.clone()]);
    meta.observe(&content, &[scored(grand(), true, false, 0)]);
    assert!(!meta.character_unlocked(&content, Character::Tailoress));
    let earned = meta.observe(&content, &[win]);
    assert!(
        earned.contains(&UnlockId::new("three_schneider")),
        "{earned:?}"
    );
}

#[test]
fn heart_brother_unlocks_with_three_suit_wins_of_one_trump() {
    let content = content();
    let won = |s| scored(suit(s), true, false, 0);
    let mut meta = MetaState::new();
    // A grand, another suit or a lost game in between start over.
    meta.observe(
        &content,
        &[
            pub_started(0, 0),
            won(Suit::Hearts),
            won(Suit::Hearts),
            scored(grand(), true, false, 0),
            won(Suit::Hearts),
            won(Suit::Hearts),
            won(Suit::Clubs),
            won(Suit::Clubs),
            scored(suit(Suit::Clubs), false, false, 0),
            won(Suit::Clubs),
        ],
    );
    assert!(!meta.character_unlocked(&content, Character::HeartBrother));
    // Across pubs and past a Ramsch.
    let ramsch = scored(Declaration::Ramsch, true, false, 0);
    let earned = meta.observe(
        &content,
        &[
            won(Suit::Clubs),
            pub_started(0, 1),
            ramsch,
            won(Suit::Clubs),
        ],
    );
    assert!(
        earned.contains(&UnlockId::new("true_colours")),
        "{earned:?}"
    );
    // A new run starts over.
    let mut meta = MetaState::new();
    meta.observe(&content, &[won(Suit::Spades), won(Suit::Spades)]);
    meta.observe(&content, &[pub_started(0, 0), won(Suit::Spades)]);
    assert!(!meta.character_unlocked(&content, Character::HeartBrother));
}

#[test]
fn the_stake_ladder_climbs_per_new_character() {
    let content = content();
    let mut meta = MetaState::all_unlocked(&content);
    meta.observe::<crate::game::Event<DealtCard>>(
        &content,
        &[RunEvent::RunWon {
            character: Character::Tailoress,
            length: RunLength::Full,
            stake: 0,
        }],
    );
    assert_eq!(meta.max_stake(Character::Tailoress, RunLength::Full), 1);
    assert_eq!(meta.max_stake(Character::Declarer, RunLength::Full), 0);
    assert_eq!(meta.max_stake(Character::Shirker, RunLength::Full), 0);
    let start = meta
        .run_start_staked(&content, Character::Tailoress, RunLength::Full, 1)
        .expect("stake 1 unlocked");
    let (run, _) = Run::new_with(&content, 4, &start).expect("run");
    assert_eq!(run.stake(), 1);
    assert_eq!(run.character(), Character::Tailoress);
}

#[test]
fn an_old_ron_meta_save_with_removed_characters_still_loads() {
    let meta: MetaState = ron::from_str(
        "(version:1,unlocked:[],stats:(runs_finished:1,characters:{GrandPlayer:(runs:1),Declarer:(runs:2)}),stakes:{Onlooker:{Full:1},Landlady:{Full:3},Shirker:{Full:2}})",
    )
    .expect("old meta");
    assert_eq!(meta.stats.characters.len(), 1);
    assert_eq!(meta.stats.characters[&Character::Declarer].runs, 2);
    assert_eq!(meta.stakes.len(), 1);
    assert_eq!(meta.max_stake(Character::Shirker, RunLength::Full), 2);
    // Every character of today reads back as itself.
    for character in Character::ALL {
        let text = ron::to_string(&meta).expect("ron");
        let back: MetaState = ron::from_str(&text).expect("ron");
        assert_eq!(back, meta);
        let json = format!(r#"{{"stakes":{{"{character}":{{"Full":1}}}}}}"#);
        let read: MetaState = serde_json::from_str(&json).expect("json");
        assert_eq!(read.max_stake(character, RunLength::Full), 1, "{character}");
    }
}

#[test]
fn an_old_meta_save_with_removed_characters_still_loads() {
    let content = content();
    let text = r#"{"version":1,"unlocked":["grand_master","full_till","tour_won"],
        "stats":{"runs_finished":3,"characters":{"Declarer":{"runs":2,"won":0,"best_section":2},
        "GrandPlayer":{"runs":1,"won":1,"best_section":3}}},
        "stakes":{"Landlady":{"Full":2},"Declarer":{"Full":1}}}"#;
    let meta: MetaState = serde_json::from_str(text).expect("old save");
    assert_eq!(meta.stats.runs_finished, 3);
    assert_eq!(
        meta.stats.characters.keys().copied().collect::<Vec<_>>(),
        vec![Character::Declarer]
    );
    assert_eq!(meta.max_stake(Character::Declarer, RunLength::Full), 1);
    assert_eq!(meta.stakes.len(), 1);
    // The old unlock ids stay, harmless; the Nullspieler is still open.
    assert!(meta.is_unlocked(&UnlockId::new("grand_master")));
    assert!(meta.character_unlocked(&content, Character::NullPlayer));
    assert!(!meta.character_unlocked(&content, Character::Shirker));
    let view = meta.view(&content);
    assert_eq!(view.characters.len(), 5);
    assert_eq!(view.unlocks.len(), content.unlocks.len());
}

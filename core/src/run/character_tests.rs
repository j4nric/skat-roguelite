//! Tests of the characters from 01.11.2026 (DESIGN 9.4 to 9.6): Der
//! Grandspieler, Der Zaungast, Die Wirtin. One test per rule, plus their
//! start, unlocks and stake ladder.

use super::content_tests::{content, fixed_deal, play};
use super::meta::MetaState;
use super::view::RevealSource;
use super::*;
use crate::cards::Suit;
use crate::content::{CardModId, ConsumableId, UnlockId, score_game};
use crate::rules::{GameType, Matadors, Outcome, RiskLevel};

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

fn has_character_line(b: &ScoreBreakdown, character: Character, change: Change) -> bool {
    b.lines.iter().any(|l| {
        l.source == Source::Character(character) && l.lever == Lever::Levels && l.change == change
    })
}

#[test]
fn every_character_loads_and_the_new_ones_start_locked() {
    let content = content();
    assert_eq!(Character::ALL.len(), 5);
    for character in Character::ALL {
        let def = content.character(character).expect("character in content");
        assert!(!def.name.is_empty() && !def.text.is_empty());
        assert_eq!(character.as_str(), character.to_string());
    }
    let meta = MetaState::new();
    assert!(meta.character_unlocked(&content, Character::Declarer));
    for (character, unlock) in [
        (Character::GrandPlayer, "grand_master"),
        (Character::Onlooker, "beat_kibitz_killer"),
        (Character::Landlady, "full_till"),
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

// ---------------------------------------------------------------------------
// Der Grandspieler
// ---------------------------------------------------------------------------

#[test]
fn grand_player_starts_with_a_gold_rimmed_club_ace() {
    let content = content();
    let run = character_run(&content, Character::GrandPlayer);
    let marked: Vec<_> = run
        .deck()
        .cards()
        .iter()
        .filter(|c| !c.mods.is_empty())
        .collect();
    assert_eq!(marked.len(), 1);
    assert_eq!(marked[0].kind, "CA".parse().expect("card"));
    assert_eq!(marked[0].mods, vec![CardModId::new("gold_rim")]);
    assert_eq!(run.deck().len(), 32);
    assert!(run.relics().is_empty());
}

#[test]
fn grand_player_grand_counts_one_level_more() {
    let content = content();
    let mut run = character_run(&content, Character::GrandPlayer);
    fixed_deal(&mut run, &content, STRONG, STRONG_SKAT);
    let (_, b) = play(&mut run, &content, hand(GameType::Grand));
    assert!(b.won);
    assert!(has_character_line(
        &b,
        Character::GrandPlayer,
        Change::Add(1)
    ));
    assert_eq!(b.levels, u64::from(b.rule_levels.total()) + 1);
}

#[test]
fn grand_player_suit_game_counts_one_level_less() {
    let content = content();
    let mut run = character_run(&content, Character::GrandPlayer);
    fixed_deal(&mut run, &content, STRONG, STRONG_SKAT);
    let (_, b) = play(&mut run, &content, hand(GameType::Suit(Suit::Diamonds)));
    assert!(b.won);
    assert!(has_character_line(
        &b,
        Character::GrandPlayer,
        Change::Add(-1)
    ));
    assert_eq!(b.levels, u64::from(b.rule_levels.total()) - 1);
    // Null keeps its levels.
    let mut run = character_run(&content, Character::GrandPlayer);
    fixed_deal(&mut run, &content, STRONG, STRONG_SKAT);
    let (_, b) = play(
        &mut run,
        &content,
        Declaration::null(crate::rules::NullRisk::Hand),
    );
    assert!(
        !b.lines
            .iter()
            .any(|l| l.source == Source::Character(Character::GrandPlayer))
    );
}

#[test]
fn grand_player_levels_never_drop_below_one() {
    // A suit game "mit 1, Spiel 2" with the rule's −1 and one more −1 from
    // elsewhere: the formula keeps one level (DESIGN 4.2).
    let decl = Declaration::new(GameType::Suit(Suit::Clubs), RiskLevel::Normal).expect("suit");
    let mit_1 = Matadors {
        with: true,
        count: 1,
    };
    let outcome = Outcome {
        declarer_card_points: 70,
        defender_card_points: 50,
        declarer_tricks: 6,
        defender_tricks: 4,
    };
    let line = ScoreLine {
        source: Source::Character(Character::GrandPlayer),
        lever: Lever::Levels,
        change: Change::Add(-1),
    };
    let thresholds = crate::rules::Thresholds::default();
    let b = score_game(decl, mit_1, &outcome, &thresholds, vec![line.clone()]);
    assert_eq!(b.levels, 1);
    assert_eq!(b.value, 12);
    let b = score_game(decl, mit_1, &outcome, &thresholds, vec![line.clone(), line]);
    assert_eq!(b.levels, 1);
}

// ---------------------------------------------------------------------------
// Der Zaungast
// ---------------------------------------------------------------------------

#[test]
fn onlooker_sees_the_left_hand_and_redeals_once_fewer_per_game() {
    let content = content();
    let run = character_run(&content, Character::Onlooker);
    assert_eq!(
        content.redeals(Character::Onlooker),
        content.run.redeals - 1
    );
    assert_eq!(content.redeals(Character::Onlooker), 1);
    assert_eq!(run.redeals_left(&content), content.run.redeals - 1);
    let left = Seat::DEFENDERS[0];
    let active = run.game().expect("game");
    assert_eq!(active.shown_hand, Some(left));
    let mut hand: Vec<u32> = active.game.hand(left).iter().map(|c| c.id).collect();
    hand.sort_unstable();
    assert_eq!(hand.len(), 10);
    let view = run.view(&content);
    let game = view.game.expect("game");
    let mut shown: Vec<u32> = game
        .revealed
        .iter()
        .filter(|r| r.source == RevealSource::Onlooker)
        .map(|r| r.card.card.id)
        .collect();
    shown.sort_unstable();
    assert_eq!(shown, hand);
    assert!(game.revealed.iter().all(|r| r.holder == Some(left)));
    let next = view.next_series.expect("next series");
    assert_eq!(next.redeals, content.run.redeals - 1);
    // The other characters keep every Einmischen and see nothing.
    let plain = character_run(&content, Character::Declarer);
    assert_eq!(plain.redeals_left(&content), content.run.redeals);
    assert!(plain.game().expect("game").peeked.is_empty());
}

#[test]
fn onlooker_redeals_never_drop_below_zero() {
    let mut content = content();
    content.run.redeals = 0;
    assert_eq!(content.redeals(Character::Onlooker), 0);
    assert_eq!(content.redeals(Character::Declarer), 0);
}

#[test]
fn onlooker_is_blind_against_the_kibitz_killer() {
    let content = content();
    let mut run = character_run(&content, Character::Onlooker);
    run.series.boss = Some(BossId::new("kibitz_killer"));
    let mut events = Vec::new();
    run.deal(&content, true, &mut events).expect("deal");
    let active = run.game().expect("game");
    assert!(active.peeked.is_empty());
    assert_eq!(active.shown_hand, None);
}

#[test]
fn onlooker_with_the_kibitz_peeks_at_other_cards() {
    let content = content();
    let mut run = character_run(&content, Character::Onlooker);
    run.relics = vec![RelicId::new("kibitzer")];
    let mut events = Vec::new();
    run.deal(&content, true, &mut events).expect("deal");
    let active = run.game().expect("game");
    let mut ids: Vec<u32> = active.peeked.iter().map(|(c, _)| c.id).collect();
    let count = ids.len();
    assert!(count > 10, "the Kiebitz adds cards to the left hand");
    ids.sort_unstable();
    ids.dedup();
    assert_eq!(ids.len(), count, "no card is shown twice");
    let right = Seat::DEFENDERS[1];
    assert!(
        active.peeked[10..].iter().all(|(_, seat)| *seat == right),
        "the Kiebitz shows the right hand"
    );
}

// ---------------------------------------------------------------------------
// Die Wirtin
// ---------------------------------------------------------------------------

#[test]
fn landlady_starts_with_herbal_and_four_relic_slots() {
    let content = content();
    let run = character_run(&content, Character::Landlady);
    assert_eq!(run.consumables(), &[ConsumableId::new("herbal")]);
    assert_eq!(content.relic_slots(Character::Landlady), 4);
    assert_eq!(
        content.relic_slots(Character::Declarer),
        content.run.relic_slots
    );
    assert_eq!(run.view(&content).relic_slots, 4);
    // A fifth relic does not fit.
    let start = RunStart {
        relics: ["tour", "kontra", "virgin", "landlord", "tuition"]
            .into_iter()
            .map(RelicId::new)
            .collect(),
        ..start(Character::Landlady)
    };
    assert_eq!(
        Run::new_with(&content, 11, &start).map(|_| ()),
        Err(RunError::NoFreeSlot)
    );
}

#[test]
fn landlady_cannot_buy_a_fifth_relic() {
    let content = content();
    let mut run = character_run(&content, Character::Landlady);
    run.relics = ["tour", "kontra", "virgin", "landlord"]
        .into_iter()
        .map(RelicId::new)
        .collect();
    run.coins = 99;
    run.phase = RunPhase::Shop;
    run.game = None;
    run.shop = Some(Shop {
        relics: vec![Some(RelicId::new("tuition"))],
        consumables: Vec::new(),
        rerolls: 0,
    });
    assert_eq!(
        run.apply(&content, RunAction::BuyRelic(0)),
        Err(RunError::NoFreeSlot)
    );
}

#[test]
fn landlady_doubles_the_series_payout() {
    let content = content();
    let economy = &content.run.economy;
    for (character, factor) in [(Character::Declarer, 1), (Character::Landlady, 2)] {
        let mut run = character_run(&content, character);
        run.coins = 12;
        run.series.games_played = 3;
        let games_left = run.series.games_left();
        let mut events = Vec::new();
        run.series_won(&content, &mut events).expect("won");
        let payout = events
            .iter()
            .find_map(|e| match e {
                RunEvent::SeriesWon { payout, .. } => Some(*payout),
                _ => None,
            })
            .expect("series won");
        let plain = economy.series_reward + games_left * economy.per_game_left + 2;
        assert_eq!(payout.base + payout.games_left + payout.interest, plain);
        assert_eq!(payout.bonus, plain * (factor - 1));
        assert_eq!(payout.total(), plain * factor);
        assert_eq!(run.coins(), 12 + plain * factor);
    }
}

// ---------------------------------------------------------------------------
// Unlocks and stakes
// ---------------------------------------------------------------------------

fn series_won(coins: u32, boss: Option<&str>) -> RunEvent {
    RunEvent::SeriesWon {
        section: 0,
        index: 2,
        payout: Payout {
            base: 3,
            games_left: 0,
            interest: 0,
            bonus: 0,
        },
        coins,
        boss: boss.map(BossId::new),
        redeals_used: 1,
        games_left: 0,
    }
}

#[test]
fn landlady_unlocks_with_twenty_groschen_after_a_series() {
    let content = content();
    let mut meta = MetaState::new();
    assert!(meta.observe(&content, &[series_won(19, None)]).is_empty());
    assert_eq!(
        meta.observe(&content, &[series_won(20, None)]),
        vec![UnlockId::new("full_till")]
    );
    assert!(meta.character_unlocked(&content, Character::Landlady));
}

#[test]
fn onlooker_unlocks_by_beating_the_kibitz_killer() {
    let content = content();
    let mut meta = MetaState::new();
    meta.observe(&content, &[series_won(0, Some("grandma"))]);
    assert!(!meta.character_unlocked(&content, Character::Onlooker));
    meta.observe(&content, &[series_won(0, Some("kibitz_killer"))]);
    assert!(meta.character_unlocked(&content, Character::Onlooker));
}

#[test]
fn grand_player_unlocks_with_a_big_grand_schneider() {
    let content = content();
    let grand = Declaration::new(GameType::Grand, RiskLevel::Normal).expect("grand");
    let schneider = Outcome {
        declarer_card_points: 95,
        defender_card_points: 25,
        declarer_tricks: 8,
        defender_tricks: 2,
    };
    let thresholds = crate::rules::Thresholds::default();
    let scored = |with: u32| -> RunEvent {
        RunEvent::GameScored {
            breakdown: score_game(
                grand,
                Matadors {
                    with: true,
                    count: with,
                },
                &schneider,
                &thresholds,
                Vec::new(),
            ),
            series_score: 0,
        }
    };
    let mut meta = MetaState::new();
    // mit 3, Spiel 4, Schneider 5: 120, too small.
    meta.observe(&content, &[scored(3)]);
    assert!(!meta.character_unlocked(&content, Character::GrandPlayer));
    // mit 4, Spiel 5, Schneider 6: 144.
    meta.observe(&content, &[scored(4)]);
    assert!(meta.character_unlocked(&content, Character::GrandPlayer));
}

#[test]
fn the_stake_ladder_climbs_per_new_character() {
    let content = content();
    let mut meta = MetaState::all_unlocked(&content);
    meta.observe::<crate::game::Event<DealtCard>>(
        &content,
        &[RunEvent::RunWon {
            character: Character::Landlady,
            length: RunLength::Full,
            stake: 0,
        }],
    );
    assert_eq!(meta.max_stake(Character::Landlady, RunLength::Full), 1);
    assert_eq!(meta.max_stake(Character::Landlady, RunLength::Short), 0);
    assert_eq!(meta.max_stake(Character::Declarer, RunLength::Full), 0);
    assert_eq!(meta.max_stake(Character::Onlooker, RunLength::Full), 0);
    let start = meta
        .run_start_staked(&content, Character::Landlady, RunLength::Full, 1)
        .expect("stake 1 unlocked");
    let (run, _) = Run::new_with(&content, 4, &start).expect("run");
    assert_eq!(run.stake(), 1);
    assert_eq!(run.character(), Character::Landlady);
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
    let rule = "relic_slots: -1,";
    let consumables = r#"start_consumables: ["herbal"],"#;
    assert!(load(rule, rule).is_ok());
    for (from, bad) in [
        (rule, "relic_slots: -1, passive: [RevealDefenderHand(0)],"),
        (rule, "relic_slots: -1, passive: [GainCoins(1)],"),
        (rule, "relic_slots: -1, payout_factor: 0,"),
        (consumables, r#"start_consumables: ["nope"],"#),
        (
            consumables,
            r#"start_consumables: ["herbal"], deck: (mods: [("CJ", "nope")]),"#,
        ),
    ] {
        assert!(load(from, bad).is_err(), "{bad} must be rejected");
    }
}

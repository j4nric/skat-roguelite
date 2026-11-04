//! Tests of the M4 mechanics: the Nullspieler, the Stammtisch figures, the
//! new bosses, encounters (Ereignisse) and the Null scoring in the run.
//! Unlocks are tested in [`super::meta`].

use super::bot::RunBot;
use super::content_tests::{content, fixed_deal, play, relic_run};
use super::*;
use crate::cards::{Card, Suit};
use crate::content::{Change, EncounterId, Factor, Lever, ScoreLine, ScoreTerm, Source};
use crate::rules::{NullRisk, RiskLevel};

/// A Null deal the declarer cannot lose: he holds the four sevens and his
/// four demoted Unter (the lowest card of each suit, ADR 0015) plus
/// the lowest diamond and heart, so he can always go under.
const SAFE_NULL: [&str; 3] = [
    "D7 DJ H7 HJ S7 SJ C7 CJ D8 H8",
    "D9 DT DQ DK DA H9 HT HQ HK HA",
    "S8 S9 ST SQ SK SA C8 C9 CT CQ",
];
const SAFE_NULL_SKAT: &str = "CK CA";

/// The declarer has two jacks, the other two lie in the skat.
const JACKS_IN_SKAT: [&str; 3] = [
    "SJ HJ DA DT DK DQ D9 D8 D7 HA",
    "HT HK HQ H9 H8 H7 SA ST SK SQ",
    "S9 S8 S7 CA CT CK CQ C9 C8 C7",
];
const JACKS_IN_SKAT_SKAT: &str = "CJ DJ";

/// The declarer holds every diamond trump but the D8 and wins every
/// trick (as in the content tests).
const STRONG: [&str; 3] = [
    "CJ SJ HJ DJ DA DT DK DQ D9 D7",
    "HA HT HK HQ H9 H8 H7 SA ST SK",
    "SQ S9 S8 S7 CA CT CK CQ C9 C8",
];
const STRONG_SKAT: &str = "D8 C7";

fn null_run(content: &Content, relics: &[&str]) -> Run {
    let start = RunStart {
        character: Character::NullPlayer,
        ..RunStart::default()
    };
    let (mut run, _) = Run::new_with(content, 11, &start).expect("null run");
    run.relics = relics.iter().map(|r| RelicId::new(r)).collect();
    run
}

fn printed(run: &Run) -> Vec<Card> {
    run.edition()
        .cards()
        .iter()
        .filter_map(|c| c.kind.face())
        .collect()
}

#[test]
fn the_null_player_starts_with_his_deck_relic_and_plays_every_game() {
    let content = content();
    let (run, events) = Run::new_with(
        &content,
        3,
        &RunStart {
            character: Character::NullPlayer,
            ..RunStart::default()
        },
    )
    .expect("start");
    assert_eq!(run.character(), Character::NullPlayer);
    // ADR 0015: the normal 32 cards, his four Unter "Abgestuft".
    let cards = printed(&run);
    assert_eq!(cards, crate::cards::standard_deck());
    let demoted: Vec<Card> = run
        .edition()
        .cards()
        .iter()
        .filter(|c| c.mods == [CardModId::new("demoted")])
        .filter_map(|c| c.kind.face())
        .collect();
    assert_eq!(demoted.len(), 4);
    assert!(demoted.iter().all(|c| c.is_jack()));
    assert_eq!(run.relics(), &[RelicId::new("kibitzer")]);
    // The Kiebitz shows one defender card at the deal.
    let peeked: Vec<(DealtCard, Seat)> = events
        .iter()
        .filter_map(|e| match e {
            RunEvent::CardPeeked { card, seat } => Some((*card, *seat)),
            _ => None,
        })
        .collect();
    assert_eq!(peeked.len(), 1);
    let (card, seat) = peeked[0];
    assert!(!seat.is_declarer());
    assert!(run.game().expect("game").game.hand(seat).contains(&card));
    let view = run.view(&content);
    assert!(
        view.game
            .expect("game")
            .revealed
            .iter()
            .any(|r| r.card.card.id == card.id && r.holder == Some(seat))
    );
    // Every game but the Ramsch (ADR 0012).
    let mut probe = run.clone();
    assert_eq!(
        probe.apply(&content, RunAction::Declare(Declaration::Ramsch)),
        Err(RunError::GameNotAvailable)
    );
    assert!(
        probe
            .apply(
                &content,
                RunAction::Declare(Declaration::grand(RiskLevel::Hand))
            )
            .is_ok()
    );
    // And the Alleinspieler Null.
    let (mut declarer, _) = Run::new(&content, 3).expect("start");
    assert!(
        declarer
            .apply(
                &content,
                RunAction::Declare(Declaration::null(NullRisk::Hand))
            )
            .is_ok()
    );
}

#[test]
fn a_won_null_is_scored_by_the_formula_with_style_bonus() {
    let content = content();
    let mut run = null_run(&content, &[]);
    fixed_deal(&mut run, &content, SAFE_NULL, SAFE_NULL_SKAT);
    let (_, b) = play(&mut run, &content, Declaration::null(NullRisk::Hand));
    assert!(b.won);
    assert_eq!(b.outcome.declarer_tricks, 0);
    assert_eq!(b.rule_base_value, 23);
    // Spiel 1 + Hand 1, plus one level per trick ducked with a seven.
    assert_eq!(b.rule_levels.game + b.rule_levels.hand, 2);
    assert!(b.rule_levels.sevens_ducked >= 1, "{:?}", b.rule_levels);
    assert_eq!(b.levels, u64::from(b.rule_levels.total()));
    assert_eq!(b.value, 23 * b.levels);
    let steps = b.steps();
    assert!(
        steps
            .iter()
            .any(|s| matches!(s.term, ScoreTerm::SevensDucked(_)))
    );
    assert_eq!(steps.last().map(|s| s.levels), Some(b.levels));
    // The same game with the fixed values of the Skatordnung: 35.
    let mut fixed = content.clone();
    if let Some(def) = fixed.characters.get_mut(&Character::NullPlayer) {
        def.null_scoring = Some(crate::rules::NullScoring::Fixed);
    }
    let mut run = null_run(&fixed, &[]);
    fixed_deal(&mut run, &fixed, SAFE_NULL, SAFE_NULL_SKAT);
    let (_, b) = play(&mut run, &fixed, Declaration::null(NullRisk::Hand));
    assert!(b.won);
    assert_eq!((b.base_value, b.levels, b.value), (35, 1, 35));
}

#[test]
fn a_null_ends_with_the_declarers_first_trick() {
    let content = content();
    let mut run = null_run(&content, &[]);
    // The declarer holds the aces: the defenders lead low and he must take.
    fixed_deal(
        &mut run,
        &content,
        [SAFE_NULL[1], SAFE_NULL[0], SAFE_NULL[2]],
        SAFE_NULL_SKAT,
    );
    let (events, b) = play(&mut run, &content, Declaration::null(NullRisk::Normal));
    assert!(!b.won);
    assert_eq!(b.outcome.declarer_tricks, 1);
    let tricks = events
        .iter()
        .filter(|e| matches!(e, RunEvent::Game(Event::TrickWon { .. })))
        .count();
    assert!(tricks < 10, "the game stops at the first trick taken");
    // Lost: −2 × 23 × Risikostufe, no style bonus.
    assert_eq!(b.score, -2 * 23);
}

#[test]
fn null_ouvert_shows_the_hand() {
    let content = content();
    let mut run = null_run(&content, &[]);
    fixed_deal(&mut run, &content, SAFE_NULL, SAFE_NULL_SKAT);
    run.apply(
        &content,
        RunAction::Declare(Declaration::null(NullRisk::OuvertHand)),
    )
    .expect("declare");
    let action = RunBot.next_action(&run, &content).expect("lead");
    run.apply(&content, action).expect("lead");
    let active = run.game().expect("game");
    // The declarer led; the defenders see his hand.
    let view = active.game.view(Seat::ALL[1]).expect("view");
    assert_eq!(
        view.declarer_hand.map(<[DealtCard]>::len),
        Some(active.game.hand(Seat::DECLARER).len())
    );
}

#[test]
fn the_show_off_demands_ouvert() {
    let content = crate::run::legacy_content();
    let mut run = null_run(&content, &["show_off"]);
    fixed_deal(&mut run, &content, SAFE_NULL, SAFE_NULL_SKAT);
    let mut probe = run.clone();
    assert_eq!(
        probe.apply(
            &content,
            RunAction::Declare(Declaration::null(NullRisk::Hand))
        ),
        Err(RunError::OuvertRequired)
    );
    let (_, b) = play(&mut run, &content, Declaration::null(NullRisk::OuvertHand));
    assert!(b.won);
    assert!(
        b.lines
            .iter()
            .any(|l| l.source == Source::Relic(RelicId::new("show_off"))
                && l.change == Change::Add(1))
    );
}

#[test]
fn the_kibitzer_and_the_counter_show_only_what_is_allowed() {
    let content = content();
    let mut run = relic_run(&content, &["kibitzer", "card_counter"]);
    let mut events = Vec::new();
    run.deal(&content, true, &mut events).expect("deal");
    assert_eq!(
        events
            .iter()
            .filter(|e| matches!(e, RunEvent::CardPeeked { .. }))
            .count(),
        1
    );
    let count = run
        .view(&content)
        .game
        .and_then(|g| g.count)
        .expect("count");
    let hand: u32 = run
        .game()
        .expect("game")
        .game
        .hand(Seat::DECLARER)
        .iter()
        .map(|c| c.card_points)
        .sum();
    assert_eq!((count.declarer, count.defenders), (0, 0));
    assert_eq!(count.unseen + hand, 120, "everything but the own hand");
    // Playing on: the sides plus the unseen cards plus the own hand stay 120.
    fixed_deal(&mut run, &content, STRONG, STRONG_SKAT);
    run.apply(
        &content,
        RunAction::Declare(Declaration::suit(Suit::Diamonds, RiskLevel::Hand)),
    )
    .expect("declare");
    for _ in 0..4 {
        let action = RunBot.next_action(&run, &content).expect("play");
        run.apply(&content, action).expect("play");
    }
    let view = run.view(&content);
    let game = view.game.expect("game");
    let count = game.count.expect("count");
    let hand: u32 = game.hand.iter().map(|c| c.card.card_points).sum();
    assert_eq!(count.declarer + count.defenders + count.unseen + hand, 120);
}

#[test]
fn the_landlord_lowers_tresen_prices() {
    let content = content();
    let mut run = relic_run(&content, &["landlord"]);
    run.coins = 50;
    run.game = None;
    run.phase = RunPhase::Shop;
    run.shop = Some(run.draw_offer(&content, 0));
    assert_eq!(run.shop_price(&content, 8), 6);
    assert_eq!(run.shop_price(&content, 5), 4);
    assert_eq!(run.shop_price(&content, 2), 2);
    let shop = run.view(&content).shop.expect("shop");
    let offer = shop.relics.iter().flatten().next().expect("offer");
    assert_eq!(offer.price, run.shop_price(&content, offer.relic.price));
    let events = run
        .apply(&content, RunAction::BuyRelic(offer.slot))
        .expect("buy");
    assert!(events.contains(&RunEvent::RelicBought {
        relic: offer.relic.id.clone(),
        price: offer.price
    }));
    assert_eq!(run.coins(), 50 - offer.price);
}

#[test]
fn granny_lets_the_player_take_a_jack_from_the_skat_once() {
    let content = crate::run::legacy_content();
    let mut run = relic_run(&content, &["granny"]);
    fixed_deal(&mut run, &content, JACKS_IN_SKAT, JACKS_IN_SKAT_SKAT);
    run.apply(
        &content,
        RunAction::Declare(Declaration::suit(Suit::Diamonds, RiskLevel::Hand)),
    )
    .expect("declare");
    assert!(run.view(&content).game.expect("game").can_take_from_skat);
    let heart_ace = 15;
    let events = run
        .apply(
            &content,
            RunAction::TakeFromSkat {
                give: heart_ace,
                take: None,
            },
        )
        .expect("take");
    assert!(
        events
            .iter()
            .any(|e| matches!(e, RunEvent::Game(Event::SkatSwapped { .. })))
    );
    let game = &run.game().expect("game").game;
    assert!(
        game.hand(Seat::DECLARER)
            .iter()
            .any(|c| c.face() == Some(Card::new(Suit::Clubs, crate::cards::Rank::Jack)))
    );
    assert!(game.skat().iter().any(|c| c.id == heart_ace));
    assert_eq!(
        run.apply(
            &content,
            RunAction::TakeFromSkat {
                give: 7,
                take: None,
            }
        ),
        Err(RunError::NoSkatSwapLeft)
    );
    // Not without the figure, not in a game with the skat picked up.
    let mut run = relic_run(&content, &[]);
    fixed_deal(&mut run, &content, JACKS_IN_SKAT, JACKS_IN_SKAT_SKAT);
    run.apply(
        &content,
        RunAction::Declare(Declaration::suit(Suit::Diamonds, RiskLevel::Hand)),
    )
    .expect("declare");
    assert_eq!(
        run.apply(
            &content,
            RunAction::TakeFromSkat {
                give: heart_ace,
                take: None,
            }
        ),
        Err(RunError::NoSkatSwapLeft)
    );
}

#[test]
fn twins_see_each_other_unless_the_referee_watches() {
    let content = crate::run::legacy_content();
    let mut run = relic_run(&content, &[]);
    run.series.boss = Some(BossId::new("twins"));
    fixed_deal(&mut run, &content, STRONG, STRONG_SKAT);
    assert!(run.game().expect("game").game.defenders_open());
    run.apply(
        &content,
        RunAction::Declare(Declaration::suit(Suit::Diamonds, RiskLevel::Hand)),
    )
    .expect("declare");
    let view = run.game().expect("game").game.view(Seat::ALL[1]);
    assert!(view.is_some_and(|v| v.partner_hand.is_some()));
    let mut run = relic_run(&content, &["referee"]);
    run.series.boss = Some(BossId::new("twins"));
    fixed_deal(&mut run, &content, STRONG, STRONG_SKAT);
    assert!(!run.game().expect("game").game.defenders_open());
}

#[test]
fn the_ramscher_makes_every_game_a_ramsch() {
    let content = content();
    let mut run = relic_run(&content, &[]);
    run.series.boss = Some(BossId::new("ramscher"));
    let mut events = Vec::new();
    run.deal(&content, true, &mut events).expect("deal");
    // A defender may lead the Ramsch: let the AI play up to the player.
    run.play_ai(&content, &mut events).expect("AI");
    assert!(events.contains(&RunEvent::RamschDealt));
    let game = &run.game().expect("game").game;
    assert_eq!(game.declaration(), Some(Declaration::Ramsch));
    assert_eq!(
        run.apply(&content, RunAction::Redeal),
        Err(RunError::RamschRequired)
    );
    let mut scored = None;
    for _ in 0..40 {
        let action = RunBot.next_action(&run, &content).expect("action");
        for event in run.apply(&content, action).expect("legal") {
            if let RunEvent::GameScored { breakdown, .. } = event {
                scored = Some(breakdown);
            }
        }
        if scored.is_some() {
            break;
        }
    }
    let b = scored.expect("scored");
    assert_eq!(b.declaration, Declaration::Ramsch);
    let parts = b.rule_levels;
    assert_eq!(parts.game, 1);
    assert!(b.rule_base_value >= 1);
    assert!(
        b.steps()
            .iter()
            .any(|s| matches!(s.term, ScoreTerm::RamschLoser(_)))
    );
    assert!(run.series().ramsch_played);
    // ADR 0017: the next game is a Ramsch again.
    if run.phase() == RunPhase::Game {
        let game = &run.game().expect("next game").game;
        assert_eq!(game.declaration(), Some(Declaration::Ramsch));
    }
}

#[test]
fn black_peter_pays_won_games_without_schneider_half() {
    let content = content();
    let mut run = relic_run(&content, &[]);
    run.series.boss = Some(BossId::new("black_peter"));
    fixed_deal(&mut run, &content, STRONG, STRONG_SKAT);
    let (_, b) = play(
        &mut run,
        &content,
        Declaration::suit(Suit::Diamonds, RiskLevel::Hand),
    );
    assert!(b.won && b.schneider);
    assert!(b.score > 0, "Schneider counts");
    assert!(
        !b.lines
            .iter()
            .any(|l| l.source == Source::Boss(BossId::new("black_peter"))),
        "a Schneider win counts fully"
    );
    // A won game without Schneider counts half, rounded half up.
    for seed in 0..200 {
        let (mut run, _) = Run::new(&content, seed).expect("run");
        run.series.boss = Some(BossId::new("black_peter"));
        let mut events = Vec::new();
        run.deal(&content, true, &mut events).expect("deal");
        for _ in 0..40 {
            if run.series().boss != Some(BossId::new("black_peter")) {
                break;
            }
            let Some(action) = RunBot.next_action(&run, &content) else {
                break;
            };
            for event in run.apply(&content, action).expect("legal") {
                if let RunEvent::GameScored { breakdown, .. } = event {
                    if breakdown.won && !breakdown.schneider {
                        let half = ScoreLine {
                            source: Source::Boss(BossId::new("black_peter")),
                            lever: Lever::GameValue,
                            change: Change::Scale(Factor::from_permille(500)),
                        };
                        assert!(breakdown.lines.contains(&half), "{breakdown:?}");
                        let raw = breakdown.base_value * breakdown.levels;
                        assert_eq!(breakdown.value, breakdown.multiplier.scale(raw));
                        assert!(breakdown.score > 0, "half, not zero");
                        assert_eq!(breakdown.score, i64::try_from(breakdown.value).unwrap());
                        return;
                    }
                }
            }
        }
    }
    panic!("no won game without Schneider in 200 seeds");
}

#[test]
fn a_lost_ramsch_counts_once_in_the_ramschers_pub() {
    // 03.11.2026 (DESIGN 17a): the boss rule `SingleLoss`, data in
    // bosses.ron, not a branch on the boss id.
    let content = content();
    let ramscher = BossId::new("ramscher");
    let def = content.boss(&ramscher).expect("ramscher");
    assert!(def.passive.contains(&Effect::SingleLoss));
    for seed in 0..200 {
        let (mut run, _) = Run::new(&content, seed).expect("run");
        run.series.boss = Some(ramscher.clone());
        let mut events = Vec::new();
        run.deal(&content, true, &mut events).expect("deal");
        run.play_ai(&content, &mut events).expect("AI");
        for _ in 0..40 {
            let Some(action) = RunBot.next_action(&run, &content) else {
                break;
            };
            for event in run.apply(&content, action).expect("legal") {
                let RunEvent::GameScored { breakdown, .. } = event else {
                    continue;
                };
                assert_eq!(breakdown.declaration, Declaration::Ramsch);
                if breakdown.won {
                    assert!(breakdown.lines.iter().all(|l| l.lever != Lever::LossFactor));
                    continue;
                }
                let once = ScoreLine {
                    source: Source::Boss(ramscher.clone()),
                    lever: Lever::LossFactor,
                    change: Change::Set(1),
                };
                assert!(breakdown.lines.contains(&once), "{breakdown:?}");
                assert_eq!(
                    breakdown.score,
                    -i64::try_from(breakdown.value).expect("small")
                );
                return;
            }
            if run.series().boss.as_ref() != Some(&ramscher) {
                break;
            }
        }
    }
    panic!("no lost Ramsch in 200 seeds");
}

#[test]
fn the_old_man_lays_one_rule_per_game() {
    let content = content();
    let def = content.boss(&BossId::new("the_old_man")).expect("old man");
    let mut run = relic_run(&content, &[]);
    run.series.boss = Some(def.id.clone());
    for (game, expected) in def.rotation.iter().enumerate().take(5) {
        run.series.games_played = u32::try_from(game).expect("small");
        assert_eq!(run.boss_rule(&content).as_ref(), Some(expected));
    }
    run.series.games_played = 0;
    fixed_deal(&mut run, &content, STRONG, STRONG_SKAT);
    assert!(run.requires_hand(), "game 1: the rule of the Schieber");
    let view = run.view(&content).game.expect("game");
    assert_eq!(
        view.boss_rule.map(|b| b.id),
        Some(BossId::new("the_pusher"))
    );
    // The last section ends with the Endboss.
    let start = RunStart {
        section: 2,
        index: 4,
        ..RunStart::default()
    };
    let (run, _) = Run::new_with(&content, 4, &start).expect("start");
    assert_eq!(run.series().boss, Some(BossId::new("the_old_man")));
    assert_eq!(
        run.series().target,
        def.target_factor.apply(content.run.targets[2][4])
    );
    // ADR 0017: five rules, every one also for the Nullspieler.
    assert_eq!(def.rotation.len(), 5);
    let mut null = null_run(&content, &[]);
    null.series.boss = Some(def.id.clone());
    let rules: Vec<Option<BossId>> = (0..5)
        .map(|game| {
            null.series.games_played = game;
            null.boss_rule(&content)
        })
        .collect();
    let expected: Vec<Option<BossId>> = def.rotation.iter().cloned().map(Some).collect();
    assert_eq!(rules, expected);
}

#[test]
fn encounters_on_the_way_must_be_taken_and_lead_to_the_next_pub() {
    let content = content();
    for seed in 0..80 {
        let (mut run, _) = Run::new(&content, seed).expect("run");
        while run.phase() == RunPhase::Game {
            let action = RunBot.next_action(&run, &content).expect("action");
            run.apply(&content, action).expect("legal");
        }
        if run.phase() != RunPhase::Route || run.series().index != 0 {
            continue;
        }
        let Some(way) = run.route().expect("route").options.iter().position(|o| {
            o.stops.len() == 1 && o.stops[0].encounter.is_some() && o.spelunke.is_none()
        }) else {
            continue;
        };
        run.apply(&content, RunAction::ChooseRoute(way))
            .expect("route");
        if run.phase() != RunPhase::Encounter {
            continue;
        }
        // No walking on: every option does something or costs something,
        // and the bot always takes one.
        let encounter = run.encounter().expect("open").clone();
        assert!(
            encounter
                .options
                .iter()
                .all(|o| !o.effects.is_empty() || o.cost > 0 || o.fallback)
        );
        let action = RunBot.encounter_action(&run, &content);
        assert!(matches!(action, RunAction::ChooseEncounter { .. }));
        let mut events = run.apply(&content, action).expect("take");
        while run.phase() == RunPhase::Encounter {
            let action = RunBot.next_action(&run, &content).expect("action");
            events.extend(run.apply(&content, action).expect("paint"));
        }
        assert!(
            events
                .iter()
                .any(|e| matches!(e, RunEvent::EncounterResolved { .. }))
        );
        assert!(
            events
                .iter()
                .any(|e| matches!(e, RunEvent::SeriesStarted { index: 1, .. }))
        );
        assert_eq!(run.phase(), RunPhase::Game);
        return;
    }
    panic!("no encounter in 80 seeds");
}

#[test]
fn encounter_choices_change_the_run() {
    let content = content();
    let choose = |option, target| RunAction::ChooseEncounter { option, target };
    // Stammtisch-Streit: a house rule for the evening, without a slot.
    let mut run = relic_run(&content, &[]);
    run.test_encounter(&content, "stammtisch_dispute").options[0].effects =
        vec![EncounterEffect::EveningRule(Some(RelicId::new(
            "blind_eye",
        )))];
    let events = run.apply(&content, choose(0, None)).expect("take");
    assert!(events.contains(&RunEvent::EveningRuleAdded(RelicId::new("blind_eye"))));
    assert_eq!(run.evening_relics(), &[RelicId::new("blind_eye")]);
    assert!(run.relics().is_empty());
    assert!(
        run.sources(&content, false)
            .iter()
            .any(|s| s.source == Source::Relic(RelicId::new("blind_eye")))
    );
    // The evening ends with the section.
    run.series.index = 4;
    run.test_encounter(&content, "last_round");
    let events = run.apply(&content, choose(2, None)).expect("water");
    assert!(events.contains(&RunEvent::EveningRulesEnded));
    assert!(run.evening_relics().is_empty());

    // Der Kartengeber: pay to choose the card, or watch for 2 Groschen.
    let mut run = relic_run(&content, &[]);
    run.coins = 5;
    run.test_encounter(&content, "card_dealer");
    assert_eq!(
        run.clone().apply(&content, choose(1, None)),
        Err(RunError::TargetRequired)
    );
    run.apply(&content, choose(1, Some(31))).expect("chosen");
    assert_eq!(run.coins(), 2);
    assert_eq!(run.edition().get(31).map(|c| c.mods.len()), Some(1));
    let mut random = relic_run(&content, &[]);
    random.test_encounter(&content, "card_dealer");
    let events = random.apply(&content, choose(0, None)).expect("random");
    assert!(events.iter().any(|e| matches!(e, RunEvent::DeckChanged(_))));
    let mut watch = relic_run(&content, &[]);
    watch.coins = 0;
    watch.test_encounter(&content, "card_dealer");
    assert_eq!(
        watch.encounter_option_block(&content, 1),
        Some(OptionBlock::NotEnoughCoins)
    );
    watch.apply(&content, choose(2, None)).expect("watch");
    assert_eq!(watch.coins(), 2);

    // Alter Bekannter: a relic swapped (target: its index), sold at the
    // full price, or a Schnaps.
    let mut run = relic_run(&content, &["kontra"]);
    run.test_encounter(&content, "old_friend");
    assert_eq!(
        run.clone().apply(&content, choose(0, None)),
        Err(RunError::TargetRequired)
    );
    let events = run.apply(&content, choose(0, Some(0))).expect("swap");
    assert!(
        events
            .iter()
            .any(|e| matches!(e, RunEvent::RelicSwapped { .. }))
    );
    assert_eq!(run.relics().len(), 1);
    assert_ne!(run.relics()[0], RelicId::new("kontra"));
    let mut run = relic_run(&content, &["old_lady"]);
    run.coins = 0;
    run.test_encounter(&content, "old_friend");
    run.apply(&content, choose(1, Some(0))).expect("sell");
    let price = content
        .relic(&RelicId::new("old_lady"))
        .expect("old_lady")
        .price;
    assert_eq!(run.coins(), price);
    assert!(run.relics().is_empty());
    let mut none = relic_run(&content, &[]);
    none.test_encounter(&content, "old_friend");
    assert_eq!(
        none.encounter_option_block(&content, 0),
        Some(OptionBlock::NoRelic)
    );
    assert_eq!(
        none.encounter_option_block(&content, 1),
        Some(OptionBlock::NoRelic)
    );
    assert_eq!(none.encounter_option_block(&content, 2), None);

    // Letzte Runde: the next declared game counts double.
    let mut run = relic_run(&content, &[]);
    run.test_encounter(&content, "last_round");
    run.apply(&content, choose(0, None)).expect("accept");
    assert_eq!(run.pending_factor(), Some(2));
    fixed_deal(&mut run, &content, STRONG, STRONG_SKAT);
    let (_, b) = play(
        &mut run,
        &content,
        Declaration::suit(Suit::Diamonds, RiskLevel::Hand),
    );
    assert!(b.lines.iter().any(
        |l| l.source == Source::Encounter(EncounterId::new("last_round"))
            && l.change == Change::Multiply(2)
    ));
    assert_eq!(run.pending_factor(), None);
    // A round for the table: 3 Groschen, one Einmischen more next pub.
    let mut run = relic_run(&content, &[]);
    run.coins = 3;
    run.test_encounter(&content, "last_round");
    let events = run.apply(&content, choose(1, None)).expect("round");
    assert_eq!(run.coins(), 0);
    assert!(
        events
            .iter()
            .any(|e| matches!(e, RunEvent::NextPubPlanned { redeals: 1, .. }))
    );
}

#[test]
fn locked_relics_are_never_offered_and_null_relics_are_for_everyone() {
    let content = content();
    let start = RunStart {
        locked_relics: vec![RelicId::new("all_rounder"), RelicId::new("old_lady")],
        ..RunStart::default()
    };
    let mut null_offered = false;
    for seed in 0..60 {
        let (mut run, _) = Run::new_with(&content, seed, &start).expect("run");
        run.game = None;
        run.phase = RunPhase::Shop;
        for rerolls in 0..5 {
            let shop = run.draw_offer(&content, rerolls);
            for id in shop.relics.iter().flatten() {
                assert!(!start.locked_relics.contains(id));
                null_offered |= id.as_str() == "fat_sow";
            }
        }
    }
    // ADR 0012: the Alleinspieler meets Null relics too.
    assert!(null_offered);
    // Weights: rarity (Common 6, Uncommon 3) times the character's focus
    // weight in percent (Alleinspieler Null 50; Nullspieler Trump 50, Null
    // 200; every game 100).
    let weight = |pool: &[(RelicId, u32)], id: &str| {
        pool.iter().find(|(r, _)| r.as_str() == id).map(|&(_, w)| w)
    };
    let (declarer, _) = Run::new(&content, 1).expect("run");
    let pool = declarer.relic_pool(&content, false);
    assert_eq!(weight(&pool, "fat_sow"), Some(300));
    assert_eq!(weight(&pool, "old_lady"), Some(300));
    assert_eq!(weight(&pool, "all_rounder"), Some(300));
    let run = null_run(&content, &[]);
    let pool = run.relic_pool(&content, false);
    assert_eq!(weight(&pool, "fat_sow"), Some(1200));
    assert_eq!(weight(&pool, "old_lady"), Some(150));
    assert_eq!(weight(&pool, "all_rounder"), Some(300));
    let mods = run.card_mod_pool(&content);
    let mod_weight = |id: &str| mods.iter().find(|(m, _)| m.as_str() == id).map(|&(_, w)| w);
    assert_eq!(mod_weight("points"), Some(50));
    assert_eq!(mod_weight("voucher_coins"), Some(100));
    // Effekte v2: random draws never give a curse or a special card.
    assert_eq!(mod_weight("transparent"), None);
    assert_eq!(mod_weight("undead"), None);
}

#[test]
fn old_saves_without_the_m4_fields_still_load() {
    let content = content();
    let (run, _) = Run::new(&content, 13).expect("run");
    let mut text = ron::to_string(&run).expect("ron");
    for field in [
        ",character:Declarer",
        ",encounter:None",
        ",evening_relics:[]",
        ",pending_factor:None",
    ] {
        assert!(text.contains(field), "{field}");
        text = text.replacen(field, "", 1);
    }
    let back: Run = ron::from_str(&text).expect("old save loads");
    assert_eq!(back.character(), Character::Declarer);
    assert_eq!(back.series(), run.series());
}

#[test]
fn the_bot_plays_whole_null_runs() {
    let content = content();
    let start = RunStart {
        character: Character::NullPlayer,
        ..RunStart::default()
    };
    let (mut won_games, mut null_games) = (0, 0);
    for seed in 0..15 {
        let (mut run, _) = Run::new_with(&content, seed, &start).expect("run");
        for _ in 0..20_000 {
            let Some(action) = RunBot.next_action(&run, &content) else {
                break;
            };
            for event in run.apply(&content, action).expect("bot") {
                if let RunEvent::GameScored { breakdown, .. } = event {
                    null_games += u32::from(breakdown.declaration.is_null());
                    won_games += u32::from(breakdown.won);
                }
            }
        }
        assert!(matches!(run.phase(), RunPhase::Won | RunPhase::Lost));
    }
    assert!(won_games > 0 && null_games > 0);
}

// ---------------------------------------------------------------------------
// M4 content: Null house rules and modifications, new encounters. One test
// per item in a constructed situation.
// ---------------------------------------------------------------------------

fn null_line(b: &ScoreBreakdown, id: &str, lever: Lever, change: Change) -> bool {
    b.lines.iter().any(|l| {
        l.source == Source::Relic(RelicId::new(id)) && l.lever == lever && l.change == change
    })
}

#[test]
fn null_eight_fifteen_adds_eight_to_every_null() {
    let content = content();
    let mut run = null_run(&content, &["null_eight_fifteen"]);
    fixed_deal(&mut run, &content, SAFE_NULL, SAFE_NULL_SKAT);
    let (_, b) = play(&mut run, &content, Declaration::null(NullRisk::Hand));
    assert!(b.won);
    assert!(null_line(
        &b,
        "null_eight_fifteen",
        Lever::BaseValue,
        Change::Add(8)
    ));
    assert_eq!(b.base_value, 31);
    // Lost too: −2 × 31.
    let mut run = null_run(&content, &["null_eight_fifteen"]);
    fixed_deal(
        &mut run,
        &content,
        [SAFE_NULL[1], SAFE_NULL[0], SAFE_NULL[2]],
        SAFE_NULL_SKAT,
    );
    let (_, b) = play(&mut run, &content, Declaration::null(NullRisk::Normal));
    assert!(!b.won);
    assert_eq!(b.score, -2 * 31);
}

#[test]
fn safety_net_adds_a_level_to_a_won_null_with_the_skat() {
    let content = crate::run::legacy_content();
    let mut run = null_run(&content, &["safety_net"]);
    fixed_deal(&mut run, &content, SAFE_NULL, SAFE_NULL_SKAT);
    let (_, b) = play(&mut run, &content, Declaration::null(NullRisk::Normal));
    assert!(b.won);
    assert!(null_line(&b, "safety_net", Lever::Levels, Change::Add(1)));
    // Hand: nothing (that is the Blindflug's).
    let mut run = null_run(&content, &["safety_net"]);
    fixed_deal(&mut run, &content, SAFE_NULL, SAFE_NULL_SKAT);
    let (_, b) = play(&mut run, &content, Declaration::null(NullRisk::Hand));
    assert!(b.won && b.lines.is_empty());
}

#[test]
fn fat_sow_makes_the_players_aces_worth_sixteen() {
    let content = content();
    let mut run = null_run(&content, &["fat_sow"]);
    fixed_deal(&mut run, &content, SAFE_NULL, SAFE_NULL_SKAT);
    let in_play = run.game().expect("game").game.cards_in_play().to_vec();
    // Since 04.11.2026 only the player's own aces (his edition).
    for card in in_play
        .iter()
        .filter(|c| c.face().is_some_and(|f| f.rank == crate::cards::Rank::Ace))
    {
        let own = card.owner == crate::deck::Owner::Player;
        assert_eq!(card.card_points, if own { 16 } else { 11 }, "{card:?}");
    }
    // Made for Null, but every character meets it (ADR 0012).
    let def = content.relic(&RelicId::new("fat_sow")).expect("fat_sow");
    assert_eq!(def.focus, crate::content::GameFocus::Null);
    assert!(crate::content::in_pool(
        &def.characters,
        Character::Declarer
    ));
}

#[test]
fn glass_house_doubles_a_won_null_ouvert() {
    let content = crate::run::legacy_content();
    let mut run = null_run(&content, &["glass_house"]);
    fixed_deal(&mut run, &content, SAFE_NULL, SAFE_NULL_SKAT);
    let (_, b) = play(&mut run, &content, Declaration::null(NullRisk::OuvertHand));
    assert!(b.won);
    assert!(null_line(
        &b,
        "glass_house",
        Lever::GameValue,
        Change::Multiply(2)
    ));
    assert_eq!(b.value, b.base_value * b.levels * 2);
    let mut run = null_run(&content, &["glass_house"]);
    fixed_deal(&mut run, &content, SAFE_NULL, SAFE_NULL_SKAT);
    let (_, b) = play(&mut run, &content, Declaration::null(NullRisk::Hand));
    assert_eq!(b.multiplier, Factor::from_int(1));
}

#[test]
fn tally_stick_adds_two_base_value_per_won_null() {
    let content = crate::run::legacy_content();
    let mut run = null_run(&content, &["tally_stick"]);
    fixed_deal(&mut run, &content, SAFE_NULL, SAFE_NULL_SKAT);
    let (_, b) = play(&mut run, &content, Declaration::null(NullRisk::Hand));
    assert!(b.won);
    assert_eq!(run.counters().get("tally"), Some(&2));
    let mut events = Vec::new();
    run.deal(&content, true, &mut events).expect("deal");
    assert!(run.game().expect("game").lines.contains(&ScoreLine {
        source: Source::Relic(RelicId::new("tally_stick")),
        lever: Lever::BaseValue,
        change: Change::Add(2),
    }));
}

#[test]
fn m4_encounters_mark_cards_raise_stakes_and_offer_rules() {
    let content = content();
    let choose = |option, target| RunAction::ChooseEncounter { option, target };
    let has_mod = |run: &Run, id: u32, m: &str| {
        run.edition()
            .get(id)
            .is_some_and(|c| c.mods.contains(&CardModId::new(m)))
    };
    let marked = |run: &Run, m: &str| {
        run.edition()
            .cards()
            .iter()
            .filter(|c| c.mods.contains(&CardModId::new(m)))
            .count()
    };
    // Umgekipptes Bier: for 4 Groschen on a chosen card, else at random,
    // or 2 Groschen to mop it up. The curse pays 3 Groschen when attached
    // (DESIGN 7.2).
    let mut run = relic_run(&content, &[]);
    run.coins = 4;
    run.test_encounter(&content, "spilled_beer");
    run.apply(&content, choose(1, Some(16))).expect("chosen");
    assert_eq!(run.coins(), 3);
    assert!(has_mod(&run, 16, "battered"));
    let mut run = relic_run(&content, &[]);
    run.test_encounter(&content, "spilled_beer");
    run.apply(&content, choose(0, None)).expect("random");
    assert_eq!(marked(&run, "battered"), 1);
    let mut run = relic_run(&content, &[]);
    run.coins = 2;
    run.test_encounter(&content, "spilled_beer");
    run.apply(&content, choose(2, None)).expect("mop");
    assert_eq!(run.coins(), 0);
    assert_eq!(marked(&run, "battered"), 0);
    // Gasthof zur Krone (Alleinspieler) and Der Taschenspieler (Nullspieler).
    let mut run = relic_run(&content, &[]);
    run.test_encounter(&content, "crown_inn");
    run.apply(&content, choose(0, None)).expect("crown");
    assert_eq!(marked(&run, "trump_bonus"), 1);
    let mut run = null_run(&content, &[]);
    run.test_encounter(&content, "conjurer");
    run.apply(&content, choose(0, None)).expect("dummy");
    assert_eq!(marked(&run, "voucher_wild"), 1);
    for (id, character) in [
        ("crown_inn", Character::Declarer),
        ("conjurer", Character::NullPlayer),
    ] {
        let def = content.encounter(&EncounterId::new(id)).expect(id);
        assert_eq!(def.characters, vec![character]);
    }
    // Preisskat: the next game counts three times.
    let mut run = relic_run(&content, &[]);
    run.test_encounter(&content, "prize_skat");
    run.apply(&content, choose(0, None)).expect("accept");
    assert_eq!(run.pending_factor(), Some(3));
    fixed_deal(&mut run, &content, STRONG, STRONG_SKAT);
    let (_, b) = play(
        &mut run,
        &content,
        Declaration::suit(Suit::Diamonds, RiskLevel::Hand),
    );
    assert!(b.lines.iter().any(
        |l| l.source == Source::Encounter(EncounterId::new("prize_skat"))
            && l.change == Change::Multiply(3)
    ));
    // Gast aus Altenburg: two different house rules on offer, or a Winkel
    // on a chosen card.
    let mut run = relic_run(&content, &[]);
    run.test_encounter(&content, "guest_from_altenburg");
    let options = run.encounter().expect("open").options.clone();
    assert_eq!(options.len(), 3);
    let rules: Vec<RelicId> = options[..2]
        .iter()
        .map(|o| match &o.effects[..] {
            [EncounterEffect::EveningRule(Some(id))] => id.clone(),
            other => panic!("{other:?}"),
        })
        .collect();
    assert_ne!(rules[0], rules[1]);
    for id in &rules {
        let def = content.relic(id).expect("relic");
        assert_eq!(def.kind, crate::content::RelicKind::HouseRule);
    }
    // "Winkel" fits no Daus since 03.11.2026: the König beside it.
    run.apply(&content, choose(2, Some(30))).expect("winkel");
    assert!(has_mod(&run, 30, "rank"));
}

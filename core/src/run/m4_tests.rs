//! Tests of the M4 mechanics: the Nullspieler, the Stammtisch figures, the
//! new bosses, encounters (Ereignisse) and the Null scoring in the run.
//! Unlocks are tested in [`super::meta`].

use super::bot::RunBot;
use super::content_tests::{content, fixed_deal, play, relic_run};
use super::*;
use crate::cards::{Card, Suit};
use crate::content::{EncounterId, Factor, ScoreTerm};
use crate::rules::{GameType, NullRisk, RiskLevel};

/// A Null deal the declarer cannot lose: he holds all eight sevens plus
/// the lowest diamond and heart, so he can always go under.
const SAFE_NULL: [&str; 3] = [
    "D7 D7 H7 H7 S7 S7 C7 C7 D8 H8",
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
    run.deck()
        .cards()
        .iter()
        .filter_map(|c| c.kind.face())
        .collect()
}

#[test]
fn the_null_player_starts_with_his_deck_relic_and_null_only() {
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
    let cards = printed(&run);
    assert_eq!(
        cards.len(),
        32,
        "no blanks: the sevens fill the jacks' places"
    );
    assert!(cards.iter().all(|c| !c.is_jack()));
    let sevens = cards
        .iter()
        .filter(|c| c.rank == crate::cards::Rank::Seven)
        .count();
    assert_eq!(sevens, 8);
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
    // Only Null.
    let mut probe = run.clone();
    assert_eq!(
        probe.apply(
            &content,
            RunAction::Declare(Declaration::grand(RiskLevel::Normal))
        ),
        Err(RunError::GameNotAvailable)
    );
    assert_eq!(
        probe.apply(&content, RunAction::Declare(Declaration::Ramsch)),
        Err(RunError::GameNotAvailable)
    );
    // And the Alleinspieler never Null.
    let (mut declarer, _) = Run::new(&content, 3).expect("start");
    assert_eq!(
        declarer.apply(
            &content,
            RunAction::Declare(Declaration::null(NullRisk::Normal))
        ),
        Err(RunError::NullNotAvailable)
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
fn null_ouvert_shows_the_hand_and_invisible_cards_never_win() {
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
    // "Unsichtbar" on the Kreuz-Ass makes it never win.
    let content = content.clone();
    let mut run = null_run(&content, &[]);
    let ace = run
        .deck()
        .cards()
        .iter()
        .find(|c| c.kind == CardKind::Normal(Card::new(Suit::Clubs, crate::cards::Rank::Ace)))
        .map(|c| c.id)
        .expect("ace");
    run.deck
        .add_mod(ace, CardModId::new("invisible"))
        .expect("mod");
    let global = run.sources(&content, false);
    let resolved = resolve_deck(&run.deck, &content, &global);
    let card = resolved
        .cards
        .iter()
        .find(|c| c.id == ace)
        .copied()
        .expect("card");
    assert!(card.never_wins);
    let seven = resolved
        .cards
        .iter()
        .find(|c| c.kind == CardKind::Normal(Card::new(Suit::Clubs, crate::cards::Rank::Seven)))
        .copied()
        .expect("seven");
    assert!(crate::rules::beats(GameType::Null, card, card, seven));
    assert!(!crate::rules::beats(GameType::Null, seven, seven, card));
}

#[test]
fn the_show_off_demands_ouvert() {
    let content = content();
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
    // The Kiebitz-Mörder switches both off.
    let mut run = relic_run(&content, &["kibitzer", "card_counter"]);
    run.series.boss = Some(BossId::new("kibitz_killer"));
    let mut events = Vec::new();
    run.deal(&content, true, &mut events).expect("deal");
    assert!(
        !events
            .iter()
            .any(|e| matches!(e, RunEvent::CardPeeked { .. }))
    );
    assert!(run.view(&content).game.and_then(|g| g.count).is_none());
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
    let content = content();
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
    let content = content();
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
fn the_revolutionary_swaps_the_defender_hands_after_five_tricks() {
    let content = content();
    let mut run = relic_run(&content, &[]);
    run.series.boss = Some(BossId::new("revolutionary"));
    fixed_deal(&mut run, &content, STRONG, STRONG_SKAT);
    let (events, b) = play(
        &mut run,
        &content,
        Declaration::suit(Suit::Diamonds, RiskLevel::Hand),
    );
    assert!(b.won);
    let swap = events
        .iter()
        .position(|e| matches!(e, RunEvent::Game(Event::DefenderHandsSwapped { .. })))
        .expect("swapped");
    let tricks_before = events[..swap]
        .iter()
        .filter(|e| matches!(e, RunEvent::Game(Event::TrickWon { .. })))
        .count();
    assert_eq!(tricks_before, 5);
    // The player sees only the hand sizes.
    let player = crate::run::player::player_events(events);
    assert!(player.iter().any(|e| matches!(
        e,
        RunEvent::Game(crate::run::player::PlayerGameEvent::DefenderHandsSwapped {
            hand_sizes: [5, 5, 5]
        })
    )));
}

#[test]
fn the_ramscher_makes_the_first_game_a_ramsch() {
    let content = content();
    let mut run = relic_run(&content, &[]);
    run.series.boss = Some(BossId::new("ramscher"));
    let mut events = Vec::new();
    run.deal(&content, true, &mut events).expect("deal");
    assert!(events.contains(&RunEvent::RamschDealt));
    let game = &run.game().expect("game").game;
    assert_eq!(game.declaration(), Some(Declaration::Ramsch));
    assert_eq!(
        run.apply(&content, RunAction::Pass),
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
    if run.phase() == RunPhase::Game {
        let game = &run.game().expect("next game").game;
        assert_ne!(game.declaration(), Some(Declaration::Ramsch));
    }
}

#[test]
fn black_peter_only_pays_won_schneider_games() {
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
    // A won game without Schneider counts nothing.
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
                        assert_eq!(breakdown.score, 0);
                        assert_eq!(breakdown.multiplier, 0);
                        return;
                    }
                }
            }
        }
    }
    panic!("no won game without Schneider in 200 seeds");
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
        section: 4,
        index: 2,
        ..RunStart::default()
    };
    let (run, _) = Run::new_with(&content, 4, &start).expect("start");
    assert_eq!(run.series().boss, Some(BossId::new("the_old_man")));
    assert_eq!(
        run.series().target,
        def.target_factor.apply(content.run.targets[4].2)
    );
    // The Nullspieler never meets the jack boss "Die Oma" in the rotation.
    let mut null = null_run(&content, &[]);
    null.series.boss = Some(def.id.clone());
    for game in 0..16 {
        null.series.games_played = game;
        assert_ne!(null.boss_rule(&content), Some(BossId::new("grandma")));
    }
}

/// Open an encounter with these options by hand.
fn encounter(run: &mut Run, id: &str, options: Vec<EncounterOption>) {
    run.game = None;
    run.shop = None;
    run.phase = RunPhase::Encounter;
    run.encounter = Some(ActiveEncounter {
        id: EncounterId::new(id),
        options,
    });
}

fn option(choice: EncounterChoice, cost: u32) -> EncounterOption {
    EncounterOption {
        choice,
        cost,
        card_mod: None,
    }
}

#[test]
fn encounters_come_instead_of_the_tresen_and_leave_to_the_next_series() {
    let mut content = content();
    content.run.encounter_chance = Factor::ONE;
    for seed in 0..50 {
        let (mut run, _) = Run::new(&content, seed).expect("run");
        while run.phase() == RunPhase::Game {
            let action = RunBot.next_action(&run, &content).expect("action");
            run.apply(&content, action).expect("legal");
        }
        if run.phase() == RunPhase::Lost {
            continue;
        }
        assert_eq!(run.phase(), RunPhase::Encounter);
        let encounter = run.encounter().expect("open").clone();
        assert_eq!(
            encounter.options.last().map(|o| &o.choice),
            Some(&EncounterChoice::Leave)
        );
        let leave = encounter.options.len() - 1;
        let events = run
            .apply(
                &content,
                RunAction::ChooseEncounter {
                    option: leave,
                    target: None,
                },
            )
            .expect("leave");
        assert!(
            events
                .iter()
                .any(|e| matches!(e, RunEvent::SeriesStarted { index: 1, .. }))
        );
        assert_eq!(run.phase(), RunPhase::Game);
        return;
    }
    panic!("no series won in 50 seeds");
}

#[test]
fn encounter_choices_change_the_run() {
    let content = content();
    // Stammtisch-Streit: a house rule for the evening, without a slot.
    let mut run = relic_run(&content, &[]);
    encounter(
        &mut run,
        "stammtisch_dispute",
        vec![
            option(EncounterChoice::TakeRule(RelicId::new("bock_round")), 0),
            option(EncounterChoice::Leave, 0),
        ],
    );
    let events = run
        .apply(
            &content,
            RunAction::ChooseEncounter {
                option: 0,
                target: None,
            },
        )
        .expect("take");
    assert!(events.contains(&RunEvent::EveningRuleAdded(RelicId::new("bock_round"))));
    assert_eq!(run.evening_relics(), &[RelicId::new("bock_round")]);
    assert!(run.relics().is_empty());
    assert!(
        run.sources(&content, false)
            .iter()
            .any(|s| s.source == Source::Relic(RelicId::new("bock_round")))
    );
    // The evening ends with the section.
    encounter(
        &mut run,
        "last_round",
        vec![option(EncounterChoice::Leave, 0)],
    );
    run.series.index = 2;
    let events = run
        .apply(
            &content,
            RunAction::ChooseEncounter {
                option: 0,
                target: None,
            },
        )
        .expect("leave");
    assert!(events.contains(&RunEvent::EveningRulesEnded));
    assert!(run.evening_relics().is_empty());

    // Der Kartengeber: pay to choose the card.
    let mut run = relic_run(&content, &[]);
    run.coins = 5;
    encounter(
        &mut run,
        "card_dealer",
        vec![
            option(EncounterChoice::MarkRandomCard, 0),
            option(EncounterChoice::MarkChosenCard, 3),
            option(EncounterChoice::Leave, 0),
        ],
    );
    let choose = |option, target| RunAction::ChooseEncounter { option, target };
    assert_eq!(
        run.clone().apply(&content, choose(1, None)),
        Err(RunError::TargetRequired)
    );
    run.apply(&content, choose(1, Some(31))).expect("chosen");
    assert_eq!(run.coins(), 2);
    assert_eq!(run.deck().get(31).map(|c| c.mods.len()), Some(1));
    let mut random = relic_run(&content, &[]);
    encounter(
        &mut random,
        "card_dealer",
        vec![
            option(EncounterChoice::MarkRandomCard, 0),
            option(EncounterChoice::Leave, 0),
        ],
    );
    let events = random.apply(&content, choose(0, None)).expect("random");
    assert!(events.iter().any(|e| matches!(e, RunEvent::DeckChanged(_))));

    // Alter Bekannter: the relic is swapped for another.
    let mut run = relic_run(&content, &["kontra"]);
    encounter(
        &mut run,
        "old_friend",
        vec![
            option(
                EncounterChoice::SwapRelic {
                    index: 0,
                    relic: RelicId::new("kontra"),
                    new: None,
                },
                0,
            ),
            option(EncounterChoice::Leave, 0),
        ],
    );
    let events = run.apply(&content, choose(0, None)).expect("swap");
    assert!(
        events
            .iter()
            .any(|e| matches!(e, RunEvent::RelicSwapped { .. }))
    );
    assert_eq!(run.relics().len(), 1);
    assert_ne!(run.relics()[0], RelicId::new("kontra"));

    // Letzte Runde: the next declared game counts double.
    let mut run = relic_run(&content, &[]);
    encounter(
        &mut run,
        "last_round",
        vec![
            option(EncounterChoice::DoubleNextGame(2), 0),
            option(EncounterChoice::Leave, 0),
        ],
    );
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
}

#[test]
fn locked_relics_and_other_characters_items_are_never_offered() {
    let content = content();
    let start = RunStart {
        locked_relics: vec![RelicId::new("bock_round"), RelicId::new("old_lady")],
        ..RunStart::default()
    };
    for seed in 0..60 {
        let (mut run, _) = Run::new_with(&content, seed, &start).expect("run");
        run.game = None;
        run.phase = RunPhase::Shop;
        for rerolls in 0..5 {
            let shop = run.draw_offer(&content, rerolls);
            for id in shop.relics.iter().flatten() {
                assert!(!start.locked_relics.contains(id));
                assert!(id.as_str() != "blind_flight" && id.as_str() != "show_off");
            }
        }
    }
    // The Nullspieler never gets the Alte Dame or Fett.
    let mut run = null_run(&content, &[]);
    run.game = None;
    run.phase = RunPhase::Shop;
    for rerolls in 0..30 {
        let shop = run.draw_offer(&content, rerolls);
        for id in shop.relics.iter().flatten() {
            let def = content.relic(id).expect("relic");
            assert!(crate::content::in_pool(
                &def.characters,
                Character::NullPlayer
            ));
        }
    }
    let pool = run.card_mod_pool(&content, &[]);
    assert!(pool.contains(&CardModId::new("invisible")));
    assert!(!pool.contains(&CardModId::new("fat")));
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
    let mut won_games = 0;
    for seed in 0..15 {
        let (mut run, _) = Run::new_with(&content, seed, &start).expect("run");
        for _ in 0..20_000 {
            let Some(action) = RunBot.next_action(&run, &content) else {
                break;
            };
            for event in run.apply(&content, action).expect("bot") {
                if let RunEvent::GameScored { breakdown, .. } = event {
                    assert!(breakdown.declaration.is_null() || breakdown.declaration.is_ramsch());
                    won_games += u32::from(breakdown.won);
                }
            }
        }
        assert!(matches!(run.phase(), RunPhase::Won | RunPhase::Lost));
    }
    assert!(won_games > 0);
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
    let content = content();
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
fn fat_sow_makes_every_ace_worth_sixteen() {
    let content = content();
    let mut run = null_run(&content, &["fat_sow"]);
    fixed_deal(&mut run, &content, SAFE_NULL, SAFE_NULL_SKAT);
    let in_play = run.game().expect("game").game.cards_in_play().to_vec();
    let aces: Vec<u32> = in_play
        .iter()
        .filter(|c| c.face().is_some_and(|f| f.rank == crate::cards::Rank::Ace))
        .map(|c| c.card_points)
        .collect();
    assert_eq!(aces, vec![16; 4]);
    // The Alleinspieler never meets it.
    let def = content.relic(&RelicId::new("fat_sow")).expect("fat_sow");
    assert!(!crate::content::in_pool(
        &def.characters,
        Character::Declarer
    ));
}

#[test]
fn glass_house_doubles_a_won_null_ouvert() {
    let content = content();
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
    assert_eq!(b.multiplier, 1);
}

#[test]
fn tally_stick_adds_two_base_value_per_won_null() {
    let content = content();
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

/// Deck id of a printed card in the Nullspieler's deck.
fn null_card(run: &Run, card: &str) -> u32 {
    let kind: CardKind = card.parse().expect("card");
    run.deck()
        .cards()
        .iter()
        .find(|c| c.kind == kind)
        .map(|c| c.id)
        .expect("in deck")
}

fn resolved_card(run: &Run, content: &Content, id: u32) -> DealtCard {
    let global = run.sources(content, false);
    resolve_deck(&run.deck, content, &global)
        .cards
        .into_iter()
        .find(|c| c.id == id)
        .expect("card")
}

#[test]
fn lead_weight_adds_ten_augen() {
    let content = content();
    let mut run = null_run(&content, &[]);
    let ace = null_card(&run, "CA");
    run.deck
        .add_mod(ace, CardModId::new("lead_weight"))
        .expect("mod");
    assert_eq!(resolved_card(&run, &content, ace).card_points, 21);
}

#[test]
fn tail_light_adds_two_levels_when_its_card_takes_the_last_trick_of_a_won_null() {
    let content = content();
    // Find the card that takes the tenth trick of the safe Null.
    let mut probe = null_run(&content, &[]);
    fixed_deal(&mut probe, &content, SAFE_NULL, SAFE_NULL_SKAT);
    let (events, b) = play(&mut probe, &content, Declaration::null(NullRisk::Hand));
    assert!(b.won);
    let last = events
        .iter()
        .rev()
        .find_map(|e| match e {
            RunEvent::Game(Event::TrickWon { trick, .. }) => Some(trick.winning_card().id),
            _ => None,
        })
        .expect("ten tricks");
    let mut run = null_run(&content, &[]);
    run.deck
        .add_mod(last, CardModId::new("tail_light"))
        .expect("mod");
    fixed_deal(&mut run, &content, SAFE_NULL, SAFE_NULL_SKAT);
    let (_, b) = play(&mut run, &content, Declaration::null(NullRisk::Hand));
    assert!(b.lines.iter().any(|l| l.source
        == Source::CardMod {
            card_mod: CardModId::new("tail_light"),
            card: last,
        }
        && l.change == Change::Add(2)));
    // On the declarer's D7 it never takes the last trick of a won Null.
    let mut run = null_run(&content, &[]);
    let seven = null_card(&run, "D7");
    run.deck
        .add_mod(seven, CardModId::new("tail_light"))
        .expect("mod");
    fixed_deal(&mut run, &content, SAFE_NULL, SAFE_NULL_SKAT);
    let (_, b) = play(&mut run, &content, Declaration::null(NullRisk::Hand));
    assert!(b.lines.is_empty());
}

#[test]
fn dressed_up_card_counts_two_ranks_higher_in_null() {
    let content = content();
    let mut run = null_run(&content, &[]);
    let nine = null_card(&run, "C9");
    run.deck
        .add_mod(nine, CardModId::new("dressed_up"))
        .expect("mod");
    let nine = resolved_card(&run, &content, nine);
    assert_eq!(nine.rank_bonus, 2);
    // In Null the order is 7 8 9 10 Unter Ober König Daus: two steps up,
    // the C9 ranks with the Unter (none in this deck) and beats the CT.
    let ten = resolved_card(&run, &content, null_card(&run, "CT"));
    assert!(crate::rules::beats(GameType::Null, ten, ten, nine));
}

#[test]
fn a_dummy_never_wins_and_shows_its_holder() {
    let content = content();
    let mut run = null_run(&content, &[]);
    // CK lies in the skat of the safe Null; take the HA (seat 1).
    let ace = null_card(&run, "HA");
    run.deck.add_mod(ace, CardModId::new("dummy")).expect("mod");
    fixed_deal(&mut run, &content, SAFE_NULL, SAFE_NULL_SKAT);
    let card = resolved_card(&run, &content, ace);
    assert!(card.never_wins && card.revealed);
    let revealed = run.revealed_holders();
    assert_eq!(revealed.len(), 1);
    assert_eq!(revealed[0].0.id, ace);
    assert_eq!(revealed[0].1, Some(Seat::DEFENDERS[0]));
}

#[test]
fn m4_encounters_mark_cards_raise_stakes_and_offer_rules() {
    let content = content();
    let choose = |option, target| RunAction::ChooseEncounter { option, target };
    let has_mod = |run: &Run, id: u32, m: &str| {
        run.deck()
            .get(id)
            .is_some_and(|c| c.mods.contains(&CardModId::new(m)))
    };
    let marked = |run: &Run, m: &str| {
        run.deck()
            .cards()
            .iter()
            .filter(|c| c.mods.contains(&CardModId::new(m)))
            .count()
    };
    // Umgekipptes Bier: for 1 Groschen on a chosen card, else at random.
    let mut run = relic_run(&content, &[]);
    run.coins = 1;
    encounter(
        &mut run,
        "spilled_beer",
        vec![
            option(EncounterChoice::MarkRandomCard, 0),
            option(EncounterChoice::MarkChosenCard, 1),
            option(EncounterChoice::Leave, 0),
        ],
    );
    run.apply(&content, choose(1, Some(16))).expect("chosen");
    assert_eq!(run.coins(), 0);
    assert!(has_mod(&run, 16, "beer_stain"));
    let mut run = relic_run(&content, &[]);
    encounter(
        &mut run,
        "spilled_beer",
        vec![
            option(EncounterChoice::MarkRandomCard, 0),
            option(EncounterChoice::Leave, 0),
        ],
    );
    run.apply(&content, choose(0, None)).expect("random");
    assert_eq!(marked(&run, "beer_stain"), 1);
    // Gasthof zur Krone (Alleinspieler) and Der Taschenspieler (Nullspieler).
    let mut run = relic_run(&content, &[]);
    encounter(
        &mut run,
        "crown_inn",
        vec![
            option(EncounterChoice::MarkRandomCard, 0),
            option(EncounterChoice::Leave, 0),
        ],
    );
    run.apply(&content, choose(0, None)).expect("crown");
    assert_eq!(marked(&run, "crown"), 1);
    let mut run = null_run(&content, &[]);
    encounter(
        &mut run,
        "conjurer",
        vec![
            option(EncounterChoice::MarkRandomCard, 0),
            option(EncounterChoice::Leave, 0),
        ],
    );
    run.apply(&content, choose(0, None)).expect("dummy");
    assert_eq!(marked(&run, "dummy"), 1);
    for (id, character) in [
        ("crown_inn", Character::Declarer),
        ("conjurer", Character::NullPlayer),
    ] {
        let def = content.encounter(&EncounterId::new(id)).expect(id);
        assert_eq!(def.characters, vec![character]);
    }
    // Preisskat: the next game counts three times.
    let mut run = relic_run(&content, &[]);
    encounter(
        &mut run,
        "prize_skat",
        vec![
            option(EncounterChoice::DoubleNextGame(3), 0),
            option(EncounterChoice::Leave, 0),
        ],
    );
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
    // Gast aus Altenburg: two house rules on offer, plus leaving.
    let mut only_guest = content.clone();
    only_guest
        .encounters
        .retain(|id, _| id.as_str() == "guest_from_altenburg");
    only_guest.run.encounter_chance = Factor::ONE;
    let mut run = relic_run(&only_guest, &[]);
    run.game = None;
    let mut events = Vec::new();
    assert!(run.open_encounter(&only_guest, &mut events));
    let options = &run.encounter().expect("open").options;
    assert_eq!(options.len(), 3);
    for o in &options[..2] {
        let EncounterChoice::TakeRule(id) = &o.choice else {
            panic!("{o:?}");
        };
        let def = content.relic(id).expect("relic");
        assert_eq!(def.kind, crate::content::RelicKind::HouseRule);
    }
}

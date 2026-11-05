//! The item review of 02.11.2026 (DESIGN 8, 12, 13): removed items and
//! old saves, the changed relics and Schnäpse, the new Schnäpse, relics
//! and encounters, one test per item.

use super::content_tests::{content, fixed_deal, play, relic_run};
use super::tour::{RouteStop, Transport};
use super::view::OptionTarget;
use super::*;
use crate::cards::Suit;
use crate::content::{ConsumableCategory, Factor, MalusId, RelicGroup, StopCategory};
use crate::rules::RiskLevel;

/// Declarer holds all trumps of a diamonds game but the D8 (in the skat):
/// he wins every trick (Schneider and Schwarz).
const STRONG: [&str; 3] = [
    "CJ SJ HJ DJ DA DT DK DQ D9 D7",
    "HA HT HK HQ H9 H8 H7 SA ST SK",
    "SQ S9 S8 S7 CA CT CK CQ C9 C8",
];
const STRONG_SKAT: &str = "D8 C7";

/// Declarer holds only Luschen: he can never reach 61 Augen.
const WEAK: [&str; 3] = [
    "D7 D8 D9 H7 H8 H9 S7 S8 S9 C7",
    "CJ SJ HJ DJ CA CT CK CQ DA DT",
    "DK DQ HA HT HK HQ SA ST SK SQ",
];
const WEAK_SKAT: &str = "C8 C9";

/// Kreuz-Daus in the standard deck.
const CLUBS_ACE: u32 = 31;
/// The König next to it: "Winkel" fits no Daus (03.11.2026).
const KING_BESIDE: u32 = 30;

fn diamonds_hand() -> Declaration {
    Declaration::suit(Suit::Diamonds, RiskLevel::Hand)
}

fn use_item(slot: usize) -> RunAction {
    RunAction::UseConsumable {
        slot,
        target: None,
        suit: None,
    }
}

/// End the current pub as if its games were played: `games` played, the
/// account at the target (won) or below (lost).
fn end_pub(run: &mut Run, content: &Content, games: u32, won: bool) -> Vec<RunEvent> {
    run.series.games_played = games;
    let target = i64::try_from(run.series.target).expect("small");
    run.series.score = if won { target } else { target - 1 };
    run.game = None;
    let mut events = Vec::new();
    run.after_game(content, &mut events).expect("after game");
    run.pass_card_reward(content, &mut events);
    events
}

/// A run at the route choice after the first pub, won in its first game.
fn at_route(content: &Content, relics: &[&str]) -> Run {
    let mut run = relic_run(content, relics);
    end_pub(&mut run, content, 1, true);
    assert_eq!(run.phase(), RunPhase::Route);
    run
}

/// Set every way of the open route choice to these stops, no Spelunke.
fn set_stops(run: &mut Run, stops: [RouteStop; 3]) {
    let route = run.route.as_mut().expect("route");
    for (option, stop) in route.options.iter_mut().zip(stops) {
        option.stops = vec![stop];
        option.spelunke = None;
    }
}

fn tresen() -> RouteStop {
    RouteStop {
        category: StopCategory::Tresen,
        encounter: None,
    }
}

fn event(id: &str) -> RouteStop {
    RouteStop {
        category: StopCategory::Event,
        encounter: Some(EncounterId::new(id)),
    }
}

fn coins_from(events: &[RunEvent], source: &Source) -> i32 {
    events
        .iter()
        .filter_map(|e| match e {
            RunEvent::CoinsGained {
                source: s, amount, ..
            } if s == source => Some(*amount),
            _ => None,
        })
        .sum()
}

fn relic(id: &str) -> Source {
    Source::Relic(RelicId::new(id))
}

/// Open encounter `id` at the route choice's place (on the way).
fn open(run: &mut Run, content: &Content, id: &str) -> Vec<EncounterOption> {
    run.route = None;
    run.trip = Some(tour::Trip {
        section: 0,
        index: 1,
        transport: Transport::Walk,
        stops: Vec::new(),
        spelunke: None,
    });
    let mut events = Vec::new();
    assert!(
        run.open_encounter(content, &EncounterId::new(id), &mut events),
        "{id} opens"
    );
    run.encounter().expect("open").options.clone()
}

fn choose(run: &mut Run, content: &Content, option: usize, target: Option<u32>) -> Vec<RunEvent> {
    run.apply(
        content,
        RunAction::ChooseEncounter {
            option,
            target,
            marke: None,
        },
    )
    .expect("choose")
}

// ---------------------------------------------------------------------------
// Removed items and old saves
// ---------------------------------------------------------------------------

const REMOVED_RELICS: [&str; 22] = [
    "blind_flight",
    "bock_round",
    "changeling",
    "clean_slate",
    "glass_house",
    "grandpa",
    "granny",
    "high_and_dry",
    "jack_parade",
    "kontra",
    "march_through",
    "old_deck",
    "ouvert_bonus",
    "referee",
    "regular_guest",
    "safety_net",
    "schneider_bock",
    "seven_sleeper",
    "show_off",
    "tally_stick",
    "tuition",
    "decoy",
];

const REMOVED_ITEMS: [&str; 6] = [
    "beer", "new_deal", "bitters", "regret", "points_5", "trump_4",
];

#[test]
fn removed_items_are_gone_and_every_reward_names_a_relic() {
    let content = content();
    for id in REMOVED_RELICS {
        assert!(content.relic(&RelicId::new(id)).is_none(), "{id}");
    }
    for id in REMOVED_ITEMS.into_iter().chain(["lure"]) {
        assert!(content.consumable(&ConsumableId::new(id)).is_none(), "{id}");
    }
    for unlock in content.unlocks.values() {
        for reward in &unlock.rewards {
            if let crate::content::Reward::Relic(id) = reward {
                assert!(content.relic(id).is_some(), "{}: {id}", unlock.id);
            }
        }
    }
    for def in content.characters.values() {
        for id in &def.start_relics {
            assert!(content.relic(id).is_some(), "{id}");
        }
        for id in &def.start_consumables {
            assert!(content.consumable(id).is_some(), "{id}");
        }
    }
    // The bot knows every item.
    for id in content.consumables.keys() {
        assert!(
            super::bot::CONSUMABLE_VALUES
                .iter()
                .any(|(name, _)| *name == id.as_str()),
            "{id}"
        );
    }
}

#[test]
fn an_old_save_drops_the_removed_relics_and_items_on_load() {
    let content = content();
    let mut run = relic_run(&content, &["bock_round", "old_lady", "kontra"]);
    run.evening_relics = vec![RelicId::new("tuition")];
    run.consumables = vec![ConsumableId::new("beer"), ConsumableId::new("herbal")];
    run.pending_next_game = vec![(ConsumableId::new("bitters"), NextGameEffect::SingleLoss)];
    run.locked = [RelicId::new("granny"), RelicId::new("tour")]
        .into_iter()
        .collect();
    // An old save: written with the removed ids, read again.
    let text = serde_json::to_string(&run).expect("json");
    let mut loaded: Run = serde_json::from_str(&text).expect("old save reads");
    let dropped = loaded.drop_unknown_content(&content);
    assert_eq!(
        dropped,
        ["bock_round", "kontra", "tuition", "beer", "bitters"]
    );
    assert_eq!(loaded.relics(), &[RelicId::new("old_lady")]);
    assert!(loaded.evening_relics().is_empty());
    assert_eq!(loaded.consumables(), &[ConsumableId::new("herbal")]);
    assert!(loaded.pending_next_game().is_empty());
    assert_eq!(
        loaded.locked.iter().cloned().collect::<Vec<_>>(),
        vec![RelicId::new("tour")]
    );
    // The view and the bot work on it.
    let view = loaded.view(&content);
    assert_eq!(view.relics.len(), 1);
    assert!(super::bot::RunBot.next_action(&loaded, &content).is_some());
    // A Tresen offer and an encounter option of a removed relic go too.
    let mut run = relic_run(&content, &[]);
    run.game = None;
    run.phase = RunPhase::Shop;
    let mut shop = run.draw_offer(&content, 0);
    shop.relics[0] = Some(RelicId::new("show_off"));
    shop.consumables[0] = Some(ConsumableId::new("regret"));
    run.shop = Some(shop);
    run.drop_unknown_content(&content);
    let shop = run.shop().expect("shop");
    assert_eq!(shop.relics[0], None);
    assert_eq!(shop.consumables[0], None);
    // An encounter option naming a removed relic stays, without it.
    let mut run = relic_run(&content, &["old_lady"]);
    let encounter = run.test_encounter(&content, "stammtisch_dispute");
    encounter.options[0].effects = vec![EncounterEffect::EveningRule(Some(RelicId::new(
        "bock_round",
    )))];
    encounter.options[1].effects =
        vec![EncounterEffect::EveningRule(Some(RelicId::new("show_off")))];
    run.drop_unknown_content(&content);
    let options = &run.encounter().expect("open").options;
    assert_eq!(options.len(), 3);
    assert_eq!(options[1].effects, vec![EncounterEffect::EveningRule(None)]);
    assert_eq!(
        run.encounter_option_block(&content, 1),
        Some(OptionBlock::NothingLeft)
    );
    // A renamed Schnaps keeps its slot: "Kreide" is "Blue Curaçao".
    let mut run = relic_run(&content, &[]);
    run.consumables = vec![ConsumableId::new("chalk"), ConsumableId::new("bitters")];
    assert_eq!(run.drop_unknown_content(&content), ["bitters"]);
    assert_eq!(run.consumables(), &[ConsumableId::new("blue_curacao")]);
    // Nothing to drop: nothing changes.
    let (mut run, _) = Run::new(&content, 3).expect("run");
    let before = run.clone();
    assert!(run.drop_unknown_content(&content).is_empty());
    assert_eq!(run, before);
}

// ---------------------------------------------------------------------------
// Changed relics
// ---------------------------------------------------------------------------

#[test]
fn three_discard_works_only_in_the_first_game_of_a_pub() {
    let content = content();
    let mut run = relic_run(&content, &["three_discard"]);
    fixed_deal(&mut run, &content, STRONG, STRONG_SKAT);
    assert!(run.discard_three_ready(), "first game");
    run.series.games_played = 1;
    assert!(!run.discard_three_ready(), "second game");
    run.series.games_played = 0;
    run.apply(&content, RunAction::PickUpSkat).expect("pick up");
    assert_eq!(
        run.game().expect("game").game.hand(Seat::DECLARER).len(),
        13,
        "a card of the left defender joins"
    );
    // A hand game in the first game lets it lapse for the pub.
    let mut run = relic_run(&content, &["three_discard"]);
    fixed_deal(&mut run, &content, STRONG, STRONG_SKAT);
    play(&mut run, &content, diamonds_hand());
    assert!(run.series.games_played >= 1 || run.phase() != RunPhase::Game);
    assert!(!run.discard_three_ready());
}

#[test]
fn groups_of_the_relics_are_data() {
    let content = content();
    let groups = [
        (
            RelicGroup::Value,
            "all_rounder beer_mat tour null_eight_fifteen every_bit_counts fat_sow old_lady skat_keeper jinx club_colours stamp_album fingertips tape_measure sewing_pattern plumb_line grandfathers_grand",
        ),
        (
            RelicGroup::Multiplier,
            "blank_king last_trick_seven with_four skat_god home_suit lucky_streak blind_eye tailor virgin watertight concede true_soul rebate_book tally_sheet lead_pouring",
        ),
        (
            RelicGroup::Control,
            "skat_redraw three_discard crib_sheet honorary_chair card_counter kibitzer glass_eye",
        ),
        (
            RelicGroup::Tour,
            "landlord closing_time season_ticket kitty slate guardian_angel dive_expert empties tip",
        ),
        (
            RelicGroup::Factor,
            "jay tree_frog robin bell_tree grand_seigneur black_zero tap hot_hand golden_shears night_owl sweep_out rag_collector moonlight night_shift dawn",
        ),
    ];
    let mut named = 0;
    for (group, ids) in groups {
        for id in ids.split_whitespace() {
            let def = content.relic(&RelicId::new(id)).expect(id);
            assert_eq!(def.group, group, "{id}");
            named += 1;
        }
    }
    assert_eq!(named, content.relics.len(), "every relic has its group");
    let run = relic_run(&content, &["kitty"]);
    assert_eq!(run.view(&content).relics[0].relic.group, RelicGroup::Tour);
    // Without the field: Value.
    let def: crate::content::RelicDef =
        ron::from_str(r#"(id: "x", name: "X", text: "X.", rarity: Common, price: 1)"#)
            .expect("relic");
    assert_eq!(def.group, RelicGroup::Value);
}

// ---------------------------------------------------------------------------
// New Schnäpse
// ---------------------------------------------------------------------------

#[test]
fn the_new_schnaepse_are_schnaepse() {
    let content = content();
    for id in ["snifter", "road_beer"] {
        let def = content.consumable(&ConsumableId::new(id)).expect(id);
        assert_eq!(def.category, ConsumableCategory::Schnapps, "{id}");
    }
}

/// "Konterbier" gave a life until ADR 0019; a save that still holds one
/// gets its price back in Groschen, the other Schnäpse stay.
#[test]
fn an_old_counter_beer_pays_its_price_back_when_loaded() {
    let content = content();
    assert!(
        content
            .consumable(&ConsumableId::new("counter_beer"))
            .is_none()
    );
    let mut run = at_route(&content, &[]);
    run.consumables = vec![
        ConsumableId::new("counter_beer"),
        ConsumableId::new("snifter"),
    ];
    let coins = run.coins();
    let json = serde_json::to_string(&run).expect("save");
    let mut loaded: Run = serde_json::from_str(&json).expect("load");
    let dropped = loaded.drop_unknown_content(&content);
    assert_eq!(dropped, vec!["counter_beer".to_string()]);
    assert_eq!(loaded.coins(), coins + 4);
    assert_eq!(loaded.consumables(), &[ConsumableId::new("snifter")]);
}

#[test]
fn snifter_draws_the_stops_of_the_route_choice_again() {
    let content = content();
    let mut changed = false;
    for seed in 0..12 {
        let mut run = at_route(&content, &[]);
        run.tour_rng = crate::rng::Rng::from_seed(seed);
        run.consumables = vec![ConsumableId::new("snifter")];
        let before = run.route().expect("route").clone();
        let events = run.apply(&content, use_item(0)).expect("snifter");
        assert!(events.contains(&RunEvent::RoutesRerolled));
        let after = run.route().expect("route");
        assert_eq!((after.section, after.index), (before.section, before.index));
        assert!(
            after
                .options
                .iter()
                .any(|o| o.stops.iter().any(|s| s.category == StopCategory::Tresen)),
            "the Tresen stays within reach"
        );
        for (a, b) in after.options.iter().zip(&before.options) {
            assert_eq!(a.spelunke, b.spelunke, "Spelunken stay");
        }
        changed |= after.options != before.options;
    }
    assert!(changed, "new stops");
    // Only at the route choice.
    let mut run = relic_run(&content, &[]);
    run.consumables = vec![ConsumableId::new("snifter")];
    assert_eq!(run.apply(&content, use_item(0)), Err(RunError::WrongPhase));
}

#[test]
fn road_beer_lowers_the_next_target_and_adds_an_einmischen() {
    let content = content();
    let mut run = at_route(&content, &[]);
    set_stops(&mut run, [tresen(), tresen(), tresen()]);
    run.consumables = vec![ConsumableId::new("road_beer")];
    let target = run.view(&content).tour.route.expect("route").next.target;
    let events = run.apply(&content, use_item(0)).expect("drink");
    assert!(events.contains(&RunEvent::NextPubPlanned {
        redeals: 1,
        target: -3
    }));
    let view = run.view(&content);
    assert_eq!(
        view.next_pub,
        NextPubPlan {
            redeals: 1,
            target: -3
        }
    );
    assert_eq!(view.tour.route.expect("route").next.target, target - 3);
    run.apply(&content, RunAction::ChooseRoute(0))
        .expect("route");
    let events = run.apply(&content, RunAction::LeaveShop).expect("on");
    assert!(events.contains(&RunEvent::NextPubApplied {
        redeals: 1,
        target: -3
    }));
    assert_eq!(run.series().target, target - 3);
    assert_eq!(run.series().extra_redeals, 1);
    assert_eq!(
        run.redeals_left(&content),
        content.redeals_in(Character::Declarer, run.series().section) + 1
    );
    assert!(run.next_pub_plan().is_empty(), "used up");
    // Not during a game.
    let mut run = relic_run(&content, &[]);
    run.consumables = vec![ConsumableId::new("road_beer")];
    assert_eq!(run.apply(&content, use_item(0)), Err(RunError::WrongPhase));
}

// ---------------------------------------------------------------------------
// New relics
// ---------------------------------------------------------------------------

#[test]
fn closing_time_pays_for_a_pub_won_in_its_first_game() {
    let content = content();
    let mut run = relic_run(&content, &["closing_time"]);
    let events = end_pub(&mut run, &content, 1, true);
    assert_eq!(coins_from(&events, &relic("closing_time")), 4);
    let mut run = relic_run(&content, &["closing_time"]);
    let events = end_pub(&mut run, &content, 2, true);
    assert_eq!(coins_from(&events, &relic("closing_time")), 0);
}

#[test]
fn season_ticket_pays_for_the_train() {
    let content = content();
    let walk = Transport::ALL
        .iter()
        .position(|t| *t == Transport::Walk)
        .expect("walk");
    let train = Transport::ALL
        .iter()
        .position(|t| *t == Transport::Train)
        .expect("train");
    let mut run = at_route(&content, &["season_ticket"]);
    set_stops(&mut run, [tresen(), tresen(), tresen()]);
    let events = run
        .apply(&content, RunAction::ChooseRoute(train))
        .expect("train");
    assert_eq!(coins_from(&events, &relic("season_ticket")), 1);
    let mut run = at_route(&content, &["season_ticket"]);
    set_stops(&mut run, [tresen(), tresen(), tresen()]);
    let events = run
        .apply(&content, RunAction::ChooseRoute(walk))
        .expect("walk");
    assert_eq!(coins_from(&events, &relic("season_ticket")), 0);
}

#[test]
fn kitty_saves_two_of_every_reward_and_pays_half_on_top_at_the_tresen() {
    let content = content();
    let mut run = relic_run(&content, &["kitty"]);
    let coins = run.coins();
    let events = end_pub(&mut run, &content, 1, true);
    let payout = events
        .iter()
        .find_map(|e| match e {
            RunEvent::SeriesWon { payout, .. } => Some(payout.total()),
            _ => None,
        })
        .expect("won");
    assert!(events.iter().any(|e| matches!(
        e,
        RunEvent::CoinsStashed {
            amount: 2,
            pot: 2,
            ..
        }
    )));
    assert_eq!(run.coins(), coins + payout - 2);
    assert_eq!(run.counters.get("kitty"), Some(&2));
    // A second pub fills the pot to 4; the Tresen pays 6.
    run.counters.insert("kitty".into(), 4);
    set_stops(&mut run, [tresen(), tresen(), tresen()]);
    let before = run.coins();
    let events = run
        .apply(&content, RunAction::ChooseRoute(0))
        .expect("route");
    assert_eq!(coins_from(&events, &relic("kitty")), 6);
    assert_eq!(run.coins(), before + 6);
    assert!(!run.counters.contains_key("kitty"), "the pot is empty");
}

#[test]
fn slate_sells_on_credit_once_per_district_and_income_pays_it_back() {
    let content = content();
    let mut run = at_route(&content, &["slate"]);
    set_stops(&mut run, [tresen(), tresen(), tresen()]);
    run.apply(&content, RunAction::ChooseRoute(0))
        .expect("route");
    run.coins = 1;
    let shop = run.shop.as_mut().expect("shop");
    shop.relics[0] = Some(RelicId::new("crib_sheet"));
    shop.relics[1] = Some(RelicId::new("old_lady"));
    let price = run.shop_price(&content, 5);
    assert!(
        run.view(&content).shop.expect("shop").relics[0]
            .as_ref()
            .expect("offer")
            .can_buy
    );
    assert_eq!(run.view(&content).credit_left, 5);
    let events = run
        .apply(&content, RunAction::BuyRelic(0))
        .expect("on credit");
    assert!(events.contains(&RunEvent::DebtChanged { debt: price - 1 }));
    assert_eq!((run.coins(), run.debt()), (0, price - 1));
    // Not beyond the limit.
    assert_eq!(
        run.apply(&content, RunAction::BuyRelic(1)),
        Err(RunError::NotEnoughCoins)
    );
    // Income pays the debt first.
    let events = run.apply(&content, RunAction::SellRelic(1)).expect("sell");
    let refund = sell_price(5);
    assert!(events.contains(&RunEvent::DebtChanged {
        debt: price - 1 - refund
    }));
    assert_eq!(run.coins(), 0);
    // The next Tresen of the same district: no credit.
    run.apply(&content, RunAction::LeaveShop).expect("leave");
    end_pub(&mut run, &content, 3, false);
    assert_eq!(run.view(&content).credit_left, 0);
    // Pub income pays off what is left.
    let mut run2 = relic_run(&content, &["slate"]);
    run2.debt = 2;
    let coins = run2.coins();
    let events = end_pub(&mut run2, &content, 1, true);
    assert!(events.contains(&RunEvent::DebtChanged { debt: 0 }));
    let payout = events
        .iter()
        .find_map(|e| match e {
            RunEvent::SeriesWon { payout, .. } => Some(payout.total()),
            _ => None,
        })
        .expect("won");
    assert_eq!(run2.coins(), coins + payout - 2);
    // Without the relic there is no credit.
    let mut run = at_route(&content, &[]);
    set_stops(&mut run, [tresen(), tresen(), tresen()]);
    run.apply(&content, RunAction::ChooseRoute(0))
        .expect("route");
    run.coins = 0;
    run.shop.as_mut().expect("shop").relics[0] = Some(RelicId::new("crib_sheet"));
    assert_eq!(
        run.apply(&content, RunAction::BuyRelic(0)),
        Err(RunError::NotEnoughCoins)
    );
}

/// ADR 0019: the "Schutzengel" has a lost pub played again, once in the
/// run, and goes; the next lost pub ends the run.
#[test]
fn guardian_angel_plays_a_lost_pub_again_once() {
    let content = content();
    let mut run = relic_run(&content, &["guardian_angel", "kibitzer"]);
    let target = run.series().target;
    let events = end_pub(&mut run, &content, 3, false);
    assert!(events.contains(&RunEvent::SecondChance {
        relic: RelicId::new("guardian_angel"),
        section: 0,
        index: 0,
    }));
    assert!(!events.iter().any(|e| matches!(e, RunEvent::RunLost { .. })));
    // The same pub from the start: fresh account, fresh deal.
    assert_eq!(run.phase(), RunPhase::Game);
    assert_eq!((run.series().section, run.series().index), (0, 0));
    assert_eq!(run.series().target, target);
    assert_eq!((run.series().score, run.series().games_played), (0, 0));
    assert!(run.game().is_some());
    assert_eq!(run.relics(), &[RelicId::new("kibitzer")]);
    assert!(run.pub_results().is_empty());
    // Lost again: the run is over.
    let events = end_pub(&mut run, &content, 3, false);
    assert!(events.iter().any(|e| matches!(e, RunEvent::RunLost { .. })));
    assert_eq!(run.phase(), RunPhase::Lost);
    // Without the angel a lost pub ends the run at once.
    let mut run = relic_run(&content, &[]);
    let events = end_pub(&mut run, &content, 3, false);
    assert!(events.contains(&RunEvent::RunLost {
        section: 0,
        index: 0,
        character: Character::Declarer,
        length: RunLength::Full,
        stake: 0,
        boss: None,
        night: false,
    }));
}

/// The second chance also holds in a boss pub (ADR 0019): the same boss,
/// not drawn again, and the house rules of the evening stay.
#[test]
fn guardian_angel_plays_a_boss_pub_again() {
    let content = content();
    let mut run = relic_run(&content, &["guardian_angel"]);
    run.series.index = PUBS_PER_DISTRICT - 1;
    let boss = run.section_boss.clone().expect("boss drawn");
    run.series.boss = Some(boss.clone());
    run.evening_relics = vec![RelicId::new("kibitzer")];
    let events = end_pub(&mut run, &content, 3, false);
    assert!(
        events
            .iter()
            .any(|e| matches!(e, RunEvent::SecondChance { .. }))
    );
    assert_eq!(run.phase(), RunPhase::Game);
    assert_eq!(run.series().index, PUBS_PER_DISTRICT - 1);
    assert_eq!(run.series().boss, Some(boss.clone()));
    assert_eq!(run.section_boss, Some(boss));
    assert_eq!(run.evening_relics, vec![RelicId::new("kibitzer")]);
    assert!(run.relics().is_empty());
    assert!(
        !events
            .iter()
            .any(|e| matches!(e, RunEvent::BossAnnounced { .. }))
    );
}

#[test]
fn dive_expert_offers_two_maluses_to_choose_from() {
    let mut content = content();
    content.run.spelunke.chance = Factor::ONE;
    let mut run = at_route(&content, &["dive_expert"]);
    // The route choice of the second pub had its Spelunke drawn with the
    // relic owned.
    let (option, spelunke) = run
        .route()
        .expect("route")
        .options
        .iter()
        .enumerate()
        .find_map(|(i, o)| o.spelunke.clone().map(|s| (i, s)))
        .expect("a Spelunke");
    assert_eq!(spelunke.malus_options.len(), 2);
    assert_eq!(spelunke.malus, spelunke.malus_options[0]);
    assert_ne!(spelunke.malus_options[0], spelunke.malus_options[1]);
    let view = run.view(&content).tour.route.expect("route");
    assert_eq!(view.options[option].malus_options.len(), 2);
    let events = run
        .apply(&content, RunAction::ChooseMalus { option, malus: 1 })
        .expect("choose");
    assert!(events.contains(&RunEvent::MalusChosen {
        option,
        malus: spelunke.malus_options[1].clone()
    }));
    let chosen: MalusId = run.route().expect("route").options[option]
        .spelunke
        .as_ref()
        .expect("spelunke")
        .malus
        .clone();
    assert_eq!(chosen, spelunke.malus_options[1]);
    assert_eq!(
        run.apply(&content, RunAction::ChooseMalus { option, malus: 2 }),
        Err(RunError::InvalidSlot)
    );
    // Without the relic: one malus, no choice.
    let run = at_route(&content, &[]);
    let spelunke = run
        .route()
        .expect("route")
        .options
        .iter()
        .find_map(|o| o.spelunke.clone())
        .expect("a Spelunke");
    assert!(spelunke.malus_options.is_empty());
}

// ---------------------------------------------------------------------------
// New encounters
// ---------------------------------------------------------------------------

/// The effects of option `i`.
fn effects(options: &[EncounterOption], i: usize) -> &[EncounterEffect] {
    &options[i].effects
}

#[test]
fn the_landlord_treats_to_a_schnaps_or_groschen() {
    let content = content();
    let mut run = at_route(&content, &[]);
    run.consumables.clear();
    let options = open(&mut run, &content, "landlord_treat");
    assert_eq!(options.len(), 3, "two Schnäpse, Groschen");
    let schnaps = |i: usize| match effects(&options, i) {
        [EncounterEffect::Schnapps(Some(id))] => id.clone(),
        other => panic!("{other:?}"),
    };
    let (first, second) = (schnaps(0), schnaps(1));
    assert_ne!(first, second);
    let def = content.consumable(&first).expect("known");
    assert_eq!(def.category, ConsumableCategory::Schnapps);
    assert_eq!(effects(&options, 2), [EncounterEffect::Coins(5)]);
    let view = run.view(&content).encounter.expect("view");
    assert!(view.options[0].effects[0].consumable.is_some());
    assert_eq!(view.options[0].effects[0].coins_instead, None);
    let events = choose(&mut run, &content, 0, None);
    assert!(events.contains(&RunEvent::ConsumableGained {
        consumable: first.clone()
    }));
    assert_eq!(run.consumables(), &[first]);
    // Or the Groschen.
    let mut run = at_route(&content, &[]);
    let coins = run.coins();
    open(&mut run, &content, "landlord_treat");
    choose(&mut run, &content, 2, None);
    assert_eq!(run.coins(), coins + 5);
    // Every slot full: the Schnaps pays its price instead.
    let mut run = at_route(&content, &[]);
    run.consumables = vec![
        ConsumableId::new("shot"),
        ConsumableId::new("herbal"),
        ConsumableId::new("eggnog"),
    ];
    let coins = run.coins();
    let options = open(&mut run, &content, "landlord_treat");
    let price = match effects(&options, 0) {
        [EncounterEffect::Schnapps(Some(id))] => content.consumable(id).expect("known").price,
        other => panic!("{other:?}"),
    };
    let view = run.view(&content).encounter.expect("view");
    assert_eq!(view.options[0].effects[0].coins_instead, Some(price));
    assert!(view.options[0].can_choose);
    choose(&mut run, &content, 0, None);
    assert_eq!(run.coins(), coins + price);
    assert_eq!(run.consumables().len(), 3);
}

#[test]
fn the_card_sharp_sells_a_strong_trick_that_is_a_curse_half_the_time() {
    let content = content();
    let (mut good, mut cursed) = (0, 0);
    for seed in 0..40 {
        let mut run = at_route(&content, &[]);
        run.encounter_rng = crate::rng::Rng::from_seed(seed);
        run.coins = 5;
        let options = open(&mut run, &content, "card_sharp");
        assert_eq!(options.len(), 3);
        assert_eq!(options[0].cost, 2);
        let offer = match effects(&options, 0) {
            [
                EncounterEffect::Mark {
                    card_mod: Some(m),
                    curse_percent: 50,
                    ..
                },
            ] => m.clone(),
            other => panic!("{other:?}"),
        };
        let view = run.view(&content).encounter.expect("view");
        assert_eq!(view.options[0].target, OptionTarget::Card);
        assert!(view.options[0].targets.contains(&KING_BESIDE));
        let events = choose(&mut run, &content, 0, Some(KING_BESIDE));
        let curse = events
            .iter()
            .find_map(|e| match e {
                RunEvent::GambleResolved { curse } => Some(*curse),
                _ => None,
            })
            .expect("resolved");
        let card = run.edition().get(KING_BESIDE).expect("card");
        if curse {
            cursed += 1;
            assert!(card.mods.iter().any(|m| {
                content
                    .card_mod(m)
                    .is_some_and(crate::content::CardModDef::is_curse)
            }));
            // Paid 2, the curse pays 3.
            assert_eq!(run.coins(), 5 - 2 + 3);
        } else {
            good += 1;
            assert!(card.mods.contains(&offer));
            assert_eq!(run.coins(), 3);
        }
    }
    assert!(good > 5 && cursed > 5, "{good} good, {cursed} cursed");
    // Exposed: 4 Groschen hush money; or played against: double stakes.
    let mut run = at_route(&content, &[]);
    let coins = run.coins();
    open(&mut run, &content, "card_sharp");
    choose(&mut run, &content, 1, None);
    assert_eq!(run.coins(), coins + 4);
    let mut run = at_route(&content, &[]);
    open(&mut run, &content, "card_sharp");
    choose(&mut run, &content, 2, None);
    assert_eq!(run.pending_factor(), Some(2));
}

#[test]
fn the_lost_wallet_pays_with_a_curse_a_schnaps_or_a_finders_fee() {
    let content = content();
    let mut run = at_route(&content, &[]);
    let coins = run.coins();
    let options = open(&mut run, &content, "lost_wallet");
    assert_eq!(options.len(), 3);
    let view = run.view(&content).encounter.expect("view");
    assert!(view.options[0].effects.iter().any(|e| {
        e.card_mod
            .as_ref()
            .is_some_and(|m| m.id.as_str() == "transparent")
    }));
    choose(&mut run, &content, 0, None);
    // 8 from the wallet, 3 for the curse (every curse pays when attached).
    assert_eq!(run.coins(), coins + 8 + 3);
    let transparent = run
        .edition()
        .cards()
        .iter()
        .filter(|c| c.mods.contains(&CardModId::new("transparent")))
        .count();
    assert_eq!(transparent, 1);
    // Handed in: the owner gives a Schnaps (a life until ADR 0019).
    let mut run = at_route(&content, &[]);
    let options = open(&mut run, &content, "lost_wallet");
    assert!(matches!(
        options[1].effects.as_slice(),
        [EncounterEffect::Schnapps(Some(_))]
    ));
    let held = run.consumables().len();
    choose(&mut run, &content, 1, None);
    assert_eq!(run.consumables().len(), held + 1);
    // The finder's fee, no curse.
    let mut run = at_route(&content, &[]);
    open(&mut run, &content, "lost_wallet");
    let coins = run.coins();
    choose(&mut run, &content, 2, None);
    assert_eq!(run.coins(), coins + 4);
}

#[test]
fn the_skat_teacher_raises_a_card_and_costs_an_einmischen_next_pub() {
    let content = content();
    let mut run = at_route(&content, &[]);
    open(&mut run, &content, "skat_teacher");
    assert_eq!(
        run.apply(
            &content,
            RunAction::ChooseEncounter {
                option: 0,
                target: None,
                marke: None
            }
        ),
        Err(RunError::TargetRequired)
    );
    let events = choose(&mut run, &content, 0, Some(KING_BESIDE));
    assert!(events.contains(&RunEvent::NextPubPlanned {
        redeals: -1,
        target: 0
    }));
    assert!(
        run.edition()
            .get(KING_BESIDE)
            .expect("card")
            .mods
            .contains(&CardModId::new("rank"))
    );
    // The next pub started: its first game allows one Einmischen fewer,
    // the second the usual number.
    assert_eq!(run.phase(), RunPhase::Game);
    let own = content.redeals_in(Character::Declarer, run.series().section);
    assert_eq!(run.redeals_left(&content), own - 1);
    run.series.games_played = 1;
    assert_eq!(run.redeals_left(&content), own);
    // The theory: both skat cards in the next game, from every deal until
    // a game is declared.
    let mut run = at_route(&content, &[]);
    open(&mut run, &content, "skat_teacher");
    let events = choose(&mut run, &content, 1, None);
    assert!(
        events
            .iter()
            .any(|e| matches!(e, RunEvent::SkatPeeked { .. }))
    );
    let view = run.view(&content);
    assert!(view.game.expect("game").skat_peek.is_some());
    // Or a Kurzer.
    let mut run = at_route(&content, &[]);
    run.consumables.clear();
    open(&mut run, &content, "skat_teacher");
    choose(&mut run, &content, 2, None);
    assert_eq!(run.consumables(), &[ConsumableId::new("shot")]);
}

#[test]
fn the_bet_pays_for_schneider_or_a_win_and_is_lost_otherwise() {
    let content = content();
    let mut run = at_route(&content, &[]);
    run.coins = 10;
    let options = open(&mut run, &content, "the_bet");
    assert_eq!(options.len(), 3, "two stakes on Schneider, one on a win");
    assert_eq!(options[0].cost, 3);
    let events = choose(&mut run, &content, 0, None);
    assert!(events.contains(&RunEvent::WagerPlaced { stake: 3 }));
    assert_eq!(run.coins(), 7);
    assert_eq!(run.wager().map(|w| w.stake), Some(3));
    // The next pub: a won game with Schneider (and Schwarz).
    fixed_deal(&mut run, &content, STRONG, STRONG_SKAT);
    let (events, b) = play(&mut run, &content, diamonds_hand());
    assert!(b.won && b.schneider);
    assert!(events.contains(&RunEvent::WagerSettled {
        stake: 3,
        won: true,
        payout: 6
    }));
    assert!(run.wager().is_none());
    // Lost: the stake is gone.
    let mut run = at_route(&content, &[]);
    run.coins = 10;
    open(&mut run, &content, "the_bet");
    choose(&mut run, &content, 1, None);
    let coins = run.coins();
    fixed_deal(&mut run, &content, WEAK, WEAK_SKAT);
    let (events, _) = play(&mut run, &content, diamonds_hand());
    assert!(events.contains(&RunEvent::WagerSettled {
        stake: 6,
        won: false,
        payout: 0
    }));
    assert_eq!(run.coins(), coins);
    // On a win: 2 staked, 3 back.
    let mut run = at_route(&content, &[]);
    run.coins = 10;
    open(&mut run, &content, "the_bet");
    choose(&mut run, &content, 2, None);
    fixed_deal(&mut run, &content, STRONG, STRONG_SKAT);
    let (events, _) = play(&mut run, &content, diamonds_hand());
    assert!(events.contains(&RunEvent::WagerSettled {
        stake: 2,
        won: true,
        payout: 3
    }));
    // Not affordable: not chosen.
    let mut run = at_route(&content, &[]);
    run.coins = 2;
    open(&mut run, &content, "the_bet");
    assert_eq!(
        run.apply(
            &content,
            RunAction::ChooseEncounter {
                option: 0,
                target: None,
                marke: None
            }
        ),
        Err(RunError::NotEnoughCoins)
    );
}

#[test]
fn old_cards_give_a_voucher_and_a_curse_or_groschen() {
    let content = content();
    let mut run = at_route(&content, &[]);
    let coins = run.coins();
    let options = open(&mut run, &content, "old_cards");
    let (voucher, curse) = match effects(&options, 0) {
        [
            EncounterEffect::Mark {
                card_mod: Some(voucher),
                ..
            },
            EncounterEffect::Mark {
                card_mod: Some(curse),
                ..
            },
        ] => (voucher.clone(), curse.clone()),
        other => panic!("{other:?}"),
    };
    assert_eq!(voucher.as_str(), "voucher_undead");
    let view = run.view(&content).encounter.expect("view");
    assert_eq!(view.options[0].target, OptionTarget::Card);
    assert_eq!(
        view.options[0].effects[1].card_mod.as_ref().map(|c| &c.id),
        Some(&curse)
    );
    choose(&mut run, &content, 0, Some(CLUBS_ACE));
    assert!(
        run.edition()
            .get(CLUBS_ACE)
            .expect("card")
            .mods
            .contains(&voucher)
    );
    let cursed: Vec<u32> = run
        .edition()
        .cards()
        .iter()
        .filter(|c| c.mods.contains(&curse))
        .map(|c| c.id)
        .collect();
    assert_eq!(cursed.len(), 1);
    assert_ne!(cursed[0], CLUBS_ACE, "another card");
    assert_eq!(run.coins(), coins + 3, "the curse pays");
    // Or the old deck sold.
    let mut run = at_route(&content, &[]);
    let coins = run.coins();
    open(&mut run, &content, "old_cards");
    choose(&mut run, &content, 2, None);
    assert_eq!(run.coins(), coins + 4);
}

#[test]
fn the_lookout_shows_the_next_stops_or_adds_an_einmischen() {
    let content = content();
    let mut run = at_route(&content, &[]);
    open(&mut run, &content, "lookout");
    let events = choose(&mut run, &content, 0, None);
    assert!(events.contains(&RunEvent::StopsRevealed { legs: 2 }));
    // The next pub: the route choice after it shows its encounters.
    assert_eq!(run.phase(), RunPhase::Game);
    end_pub(&mut run, &content, 1, true);
    set_stops(&mut run, [event("last_round"), tresen(), tresen()]);
    let view = run.view(&content).tour.route.expect("route");
    assert_eq!(
        view.options[0].stops[0]
            .encounter
            .as_ref()
            .map(|e| e.id.as_str()),
        Some("last_round")
    );
    assert!(view.options[1].stops[0].encounter.is_none(), "a Tresen");
    run.apply(&content, RunAction::ChooseRoute(1))
        .expect("route");
    assert_eq!(run.scout_legs(), 1);
    // Without the lookout the encounter stays hidden.
    let mut run = at_route(&content, &[]);
    set_stops(&mut run, [event("last_round"), tresen(), tresen()]);
    let view = run.view(&content).tour.route.expect("route");
    assert!(view.options[0].stops[0].encounter.is_none());
    // Or one more Einmischen in the next pub.
    let mut run = at_route(&content, &[]);
    open(&mut run, &content, "lookout");
    choose(&mut run, &content, 1, None);
    assert_eq!(run.series().extra_redeals, 1);
    // Or 2 Groschen.
    let mut run = at_route(&content, &[]);
    let coins = run.coins();
    open(&mut run, &content, "lookout");
    choose(&mut run, &content, 2, None);
    assert_eq!(run.coins(), coins + 2);
}

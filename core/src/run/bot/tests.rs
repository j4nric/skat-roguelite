//! Tests of the run bot: whole runs, the choice of game, the Tresen and
//! the encounters, plus the Einmischen threshold and a balance probe.

use super::*;
use crate::ai::{DeclarerHeuristic, null_unsafe_cards};
use crate::cards::{CardKind, PlayingCard, Suit};
use crate::content::{CardModId, ConsumableId, RelicId};
use crate::deck::DealtCard;
use crate::game::Seat;
use crate::rules::{Declaration, GameType, NullRisk, NullScoring, RiskLevel};

fn hand(cards: &str) -> Vec<DealtCard> {
    cards
        .split_whitespace()
        .zip(0u32..)
        .map(|(s, id)| DealtCard::plain(id, s.parse::<CardKind>().expect("card")))
        .collect()
}

fn suit(s: Suit) -> Declaration {
    Declaration::suit(s, RiskLevel::Normal)
}

#[test]
fn pass_rule_follows_trumps_and_aces() {
    let hearts = suit(Suit::Hearts);
    let five_two_aces = hand("CJ HA HT HK H9 SA ST S9 D7 D8");
    assert!(!RunBot::is_weak(&five_two_aces, hearts));
    let five_one_ace = hand("CJ HA HT HK H9 SK ST S9 D7 D8");
    assert!(RunBot::is_weak(&five_one_ace, hearts));
    let six = hand("CJ SJ HT HK H9 H8 ST S9 D7 D8");
    assert!(!RunBot::is_weak(&six, hearts));
    let grand = Declaration::grand(RiskLevel::Normal);
    assert!(!RunBot::is_weak(
        &hand("CJ DJ CA SA S9 S8 H7 H8 D7 D8"),
        grand
    ));
    assert!(RunBot::is_weak(
        &hand("CJ SJ HJ CA S9 S8 H7 H8 D7 D8"),
        grand
    ));
}

#[test]
fn announcements_are_conservative() {
    let clubs = suit(Suit::Clubs);
    // Eight top trumps and two aces: Ouvert.
    let ouvert = hand("CJ SJ HJ DJ CA CT CK CQ HA SA");
    assert_eq!(RunBot::announcement(&ouvert, clubs), RiskLevel::Ouvert);
    // Seven trumps with the top three jacks and standing aces: Schneider.
    let schneider = hand("CJ SJ HJ CA CT CK C9 HA SA ST");
    assert_eq!(
        RunBot::announcement(&schneider, clubs),
        RiskLevel::SchneiderDeclared
    );
    // A bare king outside: no announcement.
    let bare = hand("CJ SJ HJ CA CT CK C9 HA SA SK");
    assert_eq!(RunBot::announcement(&bare, clubs), RiskLevel::Normal);
    // Eight trumps from the top three jacks and one loser: Schneider.
    let one_loser = hand("CJ SJ HJ CA CT CK C9 C8 SA SK");
    assert_eq!(
        RunBot::announcement(&one_loser, clubs),
        RiskLevel::SchneiderDeclared
    );
    let grand = Declaration::grand(RiskLevel::Normal);
    let jacks = hand("CJ SJ HJ DJ CA CT SA ST HA DA");
    assert_eq!(RunBot::announcement(&jacks, grand), RiskLevel::Ouvert);
    let three = hand("CJ SJ HJ CA CT SA ST HA DA D7");
    assert_eq!(
        RunBot::announcement(&three, grand),
        RiskLevel::SchneiderDeclared
    );
    let two_losers = hand("CJ SJ HJ CA CT SA ST HA D8 D7");
    assert_eq!(RunBot::announcement(&two_losers, grand), RiskLevel::Normal);
    assert!(RunBot::says_re(&jacks, GameType::Grand));
    assert!(!RunBot::says_re(&bare[..8], GameType::Grand));
}

fn open_shop(content: &Content, coins: u32) -> Run {
    let (mut run, _) = Run::new(content, 9).expect("start");
    run.coins = coins;
    run.game = None;
    run.phase = RunPhase::Shop;
    run.shop = Some(run.draw_offer(content, 0));
    run
}

fn offer(run: &mut Run, relics: &[&str], consumables: &[&str]) {
    let shop = run.shop.as_mut().expect("shop");
    shop.relics = relics.iter().map(|r| Some(RelicId::new(r))).collect();
    shop.consumables = consumables
        .iter()
        .map(|c| Some(ConsumableId::new(c)))
        .collect();
}

#[test]
fn shop_swaps_the_weakest_relic_for_a_clearly_better_one() {
    // With every slot full (5 here, 8 in run.ron since 05.11.2026).
    let mut content = Content::load().expect("content");
    content.run.relic_slots = Some(5);
    let mut run = open_shop(&content, 6);
    run.relics = ["tour", "card_counter", "skat_redraw", "virgin", "concede"]
        .iter()
        .map(|r| RelicId::new(r))
        .collect();
    offer(&mut run, &["old_lady"], &[]);
    // Alte Dame costs 9; selling Der Zähler brings 2: 8 is not enough.
    assert_eq!(RunBot.shop_action(&run, &content), RunAction::LeaveShop);
    run.coins = 7;
    let replace = RunAction::ReplaceRelic {
        slot: 0,
        replace: 1,
    };
    assert_eq!(RunBot.shop_action(&run, &content), replace);
    run.apply(&content, replace).expect("replace");
    assert_eq!(run.relics().len(), 5);
    assert!(run.relics().contains(&RelicId::new("old_lady")));
    assert!(!run.relics().contains(&RelicId::new("card_counter")));
    // Not clearly better: the Kiebitz (3) for the Spickzettel (1).
    let mut run = open_shop(&content, 50);
    run.relics = [
        "crib_sheet",
        "every_bit_counts",
        "old_lady",
        "concede",
        "tour",
    ]
    .iter()
    .map(|r| RelicId::new(r))
    .collect();
    offer(&mut run, &["kibitzer"], &[]);
    assert!(!matches!(
        RunBot.shop_action(&run, &content),
        RunAction::SellRelic(_) | RunAction::ReplaceRelic { .. }
    ));
}

#[test]
fn shop_buys_a_schnaps_and_cleanses_a_cursed_card() {
    let content = Content::load().expect("content");
    let mut run = open_shop(&content, 10);
    run.shop.as_mut().expect("shop").card_offers.clear();
    offer(&mut run, &[], &["herbal", "road_beer"]);
    // Wegbier (2) before Kräuter (1).
    assert_eq!(
        RunBot.shop_action(&run, &content),
        RunAction::BuyConsumable(1)
    );
    // "Karte reinigen" with a cursed card, before anything else.
    run.edition
        .add_mod(&content, 9, &CardModId::new("battered"), 5)
        .expect("curse");
    assert_eq!(
        RunBot.shop_action(&run, &content),
        RunAction::CleanseCard(9)
    );
    run.apply(&content, RunAction::CleanseCard(9))
        .expect("cleanse");
    assert!(run.edition().get(9).is_some_and(|c| c.mods.is_empty()));
}

#[test]
fn shop_buys_zinken_on_bare_cards_and_hires_the_painter() {
    let content = Content::load().expect("content");
    let mut run = open_shop(&content, 7);
    offer(&mut run, &[], &[]);
    let kreuz_ass = 31;
    let offer_on = |card: u32, card_mod: &str, price: u32| {
        Some(super::super::zinken::CardOffer {
            card,
            card_mod: CardModId::new(card_mod),
            value: 1,
            price,
            bargain: None,
        })
    };
    run.edition
        .add_mod(&content, 0, &CardModId::new("rank"), 1)
        .expect("rank");
    {
        let shop = run.shop.as_mut().expect("shop");
        // Siegel would replace Winkel on card 0: never; Kerbe (1) on a
        // bare card loses to Aufdruck (2).
        shop.card_offers = vec![
            offer_on(0, "trump_bonus", 3),
            offer_on(5, "last_trick", 3),
            offer_on(kreuz_ass, "points", 3),
        ];
    }
    assert_eq!(
        RunBot.shop_action(&run, &content),
        RunAction::BuyCardOffer(2)
    );
    run.apply(&content, RunAction::BuyCardOffer(2))
        .expect("buy");
    assert_eq!(run.coins(), 4);
    // Kerbe next; then 1 Groschen is left, no Kartenmaler.
    assert_eq!(
        RunBot.shop_action(&run, &content),
        RunAction::BuyCardOffer(1)
    );
    run.shop.as_mut().expect("shop").card_offers.clear();
    run.coins = content.run.shop.zinken.painter_price + PAINTER_RESERVE;
    let hire = RunBot.shop_action(&run, &content);
    let RunAction::HirePainter(card) = hire else {
        panic!("painter: {hire:?}");
    };
    assert!(run.edition().get(card).is_some_and(|c| c.mods.is_empty()));
    run.apply(&content, hire).expect("hire");
    let choice = RunBot.shop_action(&run, &content);
    assert!(matches!(choice, RunAction::ChoosePaint(_)));
    run.apply(&content, choice).expect("paint");
    assert!(run.painter().is_none());
    assert!(run.edition().get(card).is_some_and(|c| c.mods.len() == 1));
    // Once per Tresen.
    assert!(!matches!(
        RunBot.shop_action(&run, &content),
        RunAction::HirePainter(_)
    ));
}

#[test]
fn shop_rerolls_only_with_plenty_of_groschen() {
    let content = Content::load().expect("content");
    let cost = content.run.economy.reroll_cost;
    let mut run = open_shop(&content, cost + REROLL_RESERVE);
    offer(&mut run, &[], &[]);
    assert_eq!(RunBot.shop_action(&run, &content), RunAction::Reroll);
    run.coins -= 1;
    assert_eq!(RunBot.shop_action(&run, &content), RunAction::LeaveShop);
    run.coins += 1;
    // Once per visit.
    run.shop.as_mut().expect("shop").rerolls = 1;
    assert_eq!(RunBot.shop_action(&run, &content), RunAction::LeaveShop);
}

#[test]
fn weak_hands_take_the_cheapest_way_out() {
    let content = Content::load().expect("content");
    let mut seed = 0;
    let mut run = loop {
        let (run, _) = Run::new(&content, seed).expect("start");
        let game = &run.game().expect("game").game;
        let hand = game.hand(Seat::DECLARER);
        if RunBot::is_weak(hand, DeclarerHeuristic.choose_declaration(hand))
            && null_unsafe_cards(hand, game.deck_cards()) > NULL_MAX_EXPOSED
        {
            break run;
        }
        seed += 1;
    };
    run.consumables = vec![
        ConsumableId::new("double_grain"),
        ConsumableId::new("herbal"),
    ];
    let next = |run: &Run| RunBot.next_action(run, &content).expect("action");
    // The game's own Einmischen first.
    assert_eq!(next(&run), RunAction::Redeal);
    run.series.redeals_in_game = 99;
    assert_eq!(
        next(&run),
        RunAction::UseConsumable {
            slot: 1,
            target: None,
            suit: None
        }
    );
    run.consumables.pop();
    assert_eq!(
        next(&run),
        RunAction::UseConsumable {
            slot: 0,
            target: None,
            suit: None
        }
    );
    // Nothing left to throw it in: the hand is played, with the skat.
    run.consumables.clear();
    assert_eq!(next(&run), RunAction::PickUpSkat);
}

/// Hand only for a very strong hand; otherwise pick up the skat, then
/// discard and declare the game chosen for all twelve cards: the same
/// choice before and after the discard. Both characters play suit
/// games, grand and Null (ADR 0012); Null after the skat is Ouvert
/// exactly when no exposed card is left.
#[test]
fn the_bot_sees_the_skat_before_it_chooses_the_game() {
    let content = Content::load().expect("content");
    for character in crate::content::Character::ALL {
        let start = super::super::RunStart {
            character,
            ..super::super::RunStart::default()
        };
        let (mut hand_games, mut skat_games, mut nulls) = (0, 0, 0);
        for seed in 0..80 {
            let (mut run, _) = Run::new_with(&content, seed, &start).expect("start");
            let next = |run: &Run| RunBot.next_action(run, &content).expect("action");
            match next(&run) {
                RunAction::Declare(d) => {
                    assert!(d.hand(), "seed {seed}: {d:?} before the skat");
                    hand_games += 1;
                }
                RunAction::PickUpSkat => {
                    run.apply(&content, RunAction::PickUpSkat).expect("pick up");
                    let (planned, discard) = RunBot::after_skat(&run, &content).expect("choice");
                    let ids = [discard[0].id, discard[1].id];
                    assert_eq!(next(&run), RunAction::Discard(ids), "seed {seed}");
                    run.apply(&content, RunAction::Discard(ids))
                        .expect("discard");
                    assert_eq!(next(&run), RunAction::Declare(planned), "seed {seed}");
                    assert!(!planned.hand());
                    if planned.is_null() {
                        let game = &run.game().expect("game").game;
                        let skat = game.skat();
                        let rest: Vec<DealtCard> = game
                            .deck_cards()
                            .iter()
                            .copied()
                            .filter(|c| !skat.contains(c))
                            .collect();
                        let exposed = null_unsafe_cards(game.hand(Seat::DECLARER), &rest);
                        assert_eq!(planned.ouvert(), exposed == 0, "seed {seed}");
                        nulls += 1;
                    }
                    run.apply(&content, RunAction::Declare(planned))
                        .expect("declare");
                    skat_games += 1;
                }
                _ => {} // thrown in
            }
        }
        assert!(
            skat_games > 10 && hand_games < skat_games && nulls > 0 && nulls < skat_games,
            "{character}: {hand_games} {skat_games} {nulls}"
        );
    }
}

/// The expected scores behind the choice: Null with one exposed card
/// and three sevens beats a suit game with five trumps; a suit game
/// with seven trumps beats Null with three exposed cards.
#[test]
fn expected_scores_compare_null_with_suit_games() {
    let formula = NullScoring::Formula(crate::rules::NullFormula {
        base_value: 23,
        per_seven_ducked: 1,
        never_followed: 2,
        short_circuit: 3,
        short_circuit_tricks: 5,
        short_circuit_points: 61,
    });
    let null = Declaration::null(NullRisk::Normal);
    let sevens = hand("C7 S7 H7 C8 S8 H9 D8 D9 DT DQ");
    // 76 % of 23 × 4.3, minus 24 % of 23 (a lost game counts once
    // since 04.11.2026).
    assert_eq!(null_expected_score(&sevens, 1, null, &formula), 6964);
    assert!(null_expected_score(&sevens, 7, null, &formula) < 0);
    // Fixed: 76 % of 23, minus 24 % of 23.
    assert_eq!(
        null_expected_score(&sevens, 1, null, &NullScoring::Fixed),
        1196
    );
    let hearts = suit(Suit::Hearts);
    let deck: Vec<DealtCard> = crate::deck::Edition::standard()
        .cards()
        .iter()
        .filter_map(|c| match c.kind {
            CardKind::Normal(card) => Some(DealtCard::plain(c.id, CardKind::Normal(card))),
            CardKind::Blank => None,
        })
        .collect();
    let five = hand("HJ HA HT H9 H8 SA S9 C8 D7 D8");
    let seven = hand("CJ HJ HA HT HK H9 H8 SA S9 C8");
    // Five trumps and two aces, ohne 2: 52 % of 10 × 3, minus 48 % of 30.
    assert_eq!(
        suit_expected_score(&five, &five, &deck, hearts, 10, 61),
        120
    );
    assert!(suit_expected_score(&seven, &seven, &deck, hearts, 10, 61) > 0);
    assert!(
        null_expected_score(&sevens, 1, null, &formula)
            > suit_expected_score(&five, &five, &deck, hearts, 10, 61)
    );
    assert!(
        suit_expected_score(&seven, &seven, &deck, hearts, 10, 61)
            > null_expected_score(&sevens, 3, null, &formula)
    );
    // A high stake makes the suit game worse and Null the better
    // choice; a lowered threshold keeps the measured table.
    let staked = suit_expected_score(&seven, &seven, &deck, hearts, 10, 61 + 30);
    assert!(staked < suit_expected_score(&seven, &seven, &deck, hearts, 10, 61));
    assert!(staked < null_expected_score(&sevens, 3, null, &formula));
    assert_eq!(
        suit_expected_score(&five, &five, &deck, hearts, 10, 58),
        suit_expected_score(&five, &five, &deck, hearts, 10, 61)
    );
}

/// "Der Schieber": Null Hand only with at most two exposed cards.
#[test]
fn forced_hand_takes_null_only_with_few_exposed_cards() {
    let content = Content::load().expect("content");
    for character in crate::content::Character::ALL {
        let start = super::super::RunStart {
            character,
            ..super::super::RunStart::default()
        };
        for seed in 0..80 {
            let (mut run, _) = Run::new_with(&content, seed, &start).expect("start");
            run.game.as_mut().expect("game").rules.require_hand = true;
            let game = &run.game().expect("game").game;
            let exposed = null_unsafe_cards(game.hand(Seat::DECLARER), game.deck_cards());
            if let Some(RunAction::Declare(d)) = RunBot.next_action(&run, &content) {
                assert!(d.hand(), "seed {seed}");
                if d.is_null() {
                    assert!(exposed <= NULL_HAND_MAX_EXPOSED, "seed {seed}: {exposed}");
                }
            }
        }
    }
}

#[test]
fn value_tables_know_every_content_item() {
    let content = Content::load().expect("content");
    for id in content.relics.keys() {
        assert!(
            RELIC_VALUES
                .iter()
                .chain(LEGENDARY_VALUES.iter())
                .chain(NIGHT_VALUES.iter())
                .any(|(name, _)| *name == id.as_str()),
            "relic {id} has no bot value"
        );
    }
    for id in content.consumables.keys() {
        assert!(
            CONSUMABLE_VALUES
                .iter()
                .any(|(name, _)| *name == id.as_str()),
            "consumable {id} has no bot value"
        );
    }
}
/// A run with `relics` on a constructed deal (the player forehand), the
/// skat picked up: the bot's planned declaration and discard.
fn plan_after_skat(relics: &[&str], hands: [&str; 3], skat: &str) -> (Declaration, Vec<DealtCard>) {
    use super::super::content_tests::{content, fixed_deal, relic_run};
    let content = content();
    let mut run = relic_run(&content, relics);
    fixed_deal(&mut run, &content, hands, skat);
    run.apply(&content, RunAction::PickUpSkat).expect("pick up");
    RunBot::after_skat(&run, &content).expect("plan")
}

/// A grand mit 4 with the four clubs from the ace and two aces: two
/// side cards may lose a trick by the plain rules, so the bot plays it
/// without announcement; double-dummy the declarer takes every trick.
const SCHNEIDER_HAND: [&str; 3] = [
    "CJ SJ HJ DJ CA CT CK CQ SA HA",
    "C9 C8 HT HK HQ H9 H8 SK SQ S9",
    "C7 H7 ST S8 S7 DA DT DK DQ D9",
];
const SCHNEIDER_SKAT: &str = "D8 D7";

#[test]
fn big_mouth_and_open_visor_make_the_bot_announce() {
    let grand = |risk| Declaration::grand(risk).with_hand(false);
    let (plain, plain_discard) = plan_after_skat(&[], SCHNEIDER_HAND, SCHNEIDER_SKAT);
    assert_eq!(plain, grand(RiskLevel::Normal));
    let (big, big_discard) = plan_after_skat(&["big_mouth"], SCHNEIDER_HAND, SCHNEIDER_SKAT);
    assert_eq!(big, grand(RiskLevel::SchneiderDeclared));
    assert_eq!(big_discard, plain_discard);
    let (visor, _) = plan_after_skat(&["open_visor"], SCHNEIDER_HAND, SCHNEIDER_SKAT);
    assert_eq!(visor, grand(RiskLevel::Ouvert));
    let (both, _) = plan_after_skat(&["big_mouth", "open_visor"], SCHNEIDER_HAND, SCHNEIDER_SKAT);
    assert_eq!(both, grand(RiskLevel::Ouvert));
    // A hand that does not make Schneider stays plain: STRONG of the
    // content tests without two of its trumps.
    let middling = [
        "CJ HJ DA DK DQ D9 D7 HA S7 C8",
        "SJ DJ HT HK HQ H9 H8 H7 SA ST",
        "DT SK SQ S9 S8 CA CT CK CQ C9",
    ];
    let (decl, _) = plan_after_skat(&["big_mouth", "open_visor"], middling, "D8 C7");
    assert!(!decl.schneider_declared(), "{decl:?}");
}

#[test]
fn schneider_percent_is_deterministic_and_reads_the_cards() {
    let deck: Vec<DealtCard> = hand(
        "CJ SJ HJ DJ CA CT CK SA ST H7 HA HT HK HQ H9 H8 SK SQ S9 S8 \
         CQ C9 C8 C7 DA DT DK DQ D9 S7 D8 D7",
    );
    let cards: Vec<DealtCard> = deck[..10].iter().chain(&deck[30..]).copied().collect();
    let kept = &deck[..10];
    let clubs = GameType::Suit(Suit::Clubs);
    let p = schneider_percent(clubs, kept, &cards, &deck, Seat::DECLARER, 30, 7);
    assert!(p >= BOLD_MIN_PERCENT, "{p}");
    assert_eq!(
        p,
        schneider_percent(clubs, kept, &cards, &deck, Seat::DECLARER, 30, 7)
    );
    // A defender's hand in the declarer's seat: no trump, no Schneider.
    let weak: Vec<DealtCard> = deck[10..20].iter().chain(&deck[30..]).copied().collect();
    assert_eq!(
        schneider_percent(clubs, &deck[10..20], &weak, &deck, Seat::DECLARER, 30, 7),
        0
    );
}

#[test]
fn clear_cut_makes_the_bot_discard_for_voids() {
    // Clubs with nine trumps, HA, S7 and from the skat D7 D8: the plain
    // rule never discards an ace and leaves one void suit (spades);
    // with "Kahlschlag" HA and S7 go, hearts and spades are void.
    let hands = [
        "CJ SJ HJ CT CK CQ C9 C8 HA S7",
        "DJ CA C7 HT HK HQ H9 H8 H7 SA",
        "ST SK SQ S9 S8 DA DT DK DQ D9",
    ];
    let faces = |cards: &[DealtCard]| -> Vec<String> {
        let mut v: Vec<String> = cards
            .iter()
            .filter_map(|c| c.face())
            .map(|f| f.to_string())
            .collect();
        v.sort();
        v
    };
    let (plain, plain_discard) = plan_after_skat(&[], hands, "D7 D8");
    assert_eq!(plain.game(), GameType::Suit(Suit::Clubs));
    assert!(!faces(&plain_discard).contains(&"HA".to_string()));
    let (decl, discard) = plan_after_skat(&["clear_cut"], hands, "D7 D8");
    assert_eq!(decl.game(), GameType::Suit(Suit::Clubs));
    assert_eq!(faces(&discard), vec!["HA".to_string(), "S7".to_string()]);
    // Not a suit game: the plain discard.
    let deck = hand("CJ SJ HJ DJ CA C7 HA H7 S7 S8 D7 D8");
    let grand = Declaration::grand(RiskLevel::Normal);
    assert_eq!(
        RunBot::void_discard(&deck, grand, &deck),
        RunBot::discard_for(&deck, grand, &deck)
    );
}

mod redeal_tests {
    use super::super::*;
    use crate::ai::{DeclarerHeuristic, null_unsafe_cards};
    use crate::content::Content;
    use crate::game::Seat;
    use crate::rules::GameType;

    #[test]
    fn the_redeal_threshold_falls_with_fewer_einmischen() {
        assert_eq!(RunBot::redeal_threshold(1), REDEAL_STOP[0]);
        assert_eq!(RunBot::redeal_threshold(3), REDEAL_STOP[2]);
        assert!(RunBot::redeal_threshold(2) < RunBot::redeal_threshold(3));
        // Beyond the table the last value holds.
        assert_eq!(RunBot::redeal_threshold(9), REDEAL_STOP[4]);
    }

    /// A deal that is not weak but scores below the threshold is thrown
    /// in while an Einmischen is left, and played without one.
    #[test]
    fn the_bot_throws_in_a_mediocre_hand_only_with_einmischen_left() {
        let content = Content::load().expect("content");
        let grand = content.run.base_value(GameType::Grand).unwrap_or(0);
        let found = (0..500).find_map(|seed| {
            let (run, _) = Run::new(&content, seed).ok()?;
            let active = run.game()?;
            let hand = active.game.hand(Seat::DECLARER);
            let deck = active.game.deck_cards();
            let suit = DeclarerHeuristic.choose_declaration(hand);
            let scoring = run.null_scoring(&content);
            let score = ten_card_score(hand, deck, grand, Some(&scoring));
            let left = run.redeals_left(&content);
            let mediocre = !RunBot::is_weak(hand, suit)
                && !suit.hand()
                && null_unsafe_cards(hand, deck) > 0
                && score < RunBot::redeal_threshold(left);
            mediocre.then_some(run)
        });
        let mut run = found.expect("a mediocre deal in 500 seeds");
        assert_eq!(RunBot.next_action(&run, &content), Some(RunAction::Redeal));
        run.series.redeals_in_game = 99;
        run.series.extra_redeals = 0;
        assert_eq!(run.redeals_left(&content), 0);
        assert_ne!(RunBot.next_action(&run, &content), Some(RunAction::Redeal));
    }
}

mod redeal_probe {
    use super::super::*;
    use crate::cards::CardKind;
    use crate::content::{Character, Content};
    use crate::deck::DealtCard;
    use crate::rules::NullScoring;

    /// Prints the distribution of [`ten_card_score`] over random deals and
    /// the optimal-stopping values (balance tool, not a check).
    #[test]
    #[ignore = "balance probe, prints numbers"]
    fn ten_card_score_distribution() {
        let content = Content::load().expect("content");
        for character in [Character::Declarer, Character::NullPlayer] {
            let def = content.character(character).expect("character");
            let scoring = def.null_scoring.clone().unwrap_or(NullScoring::Fixed);
            let deck: Vec<DealtCard> = crate::deck::Edition::standard()
                .cards()
                .iter()
                .filter_map(|c| match c.kind {
                    CardKind::Normal(card) => Some(DealtCard::plain(c.id, CardKind::Normal(card))),
                    CardKind::Blank => None,
                })
                .collect();
            let mut rng = crate::rng::Rng::from_seed(7);
            let mut scores = Vec::new();
            for _ in 0..200_000 {
                let mut d = deck.clone();
                rng.shuffle(&mut d);
                scores.push(ten_card_score(&d[..10], &deck, 18, Some(&scoring)));
            }
            scores.sort_unstable();
            let n = scores.len();
            let q: Vec<i64> = [5, 10, 25, 50, 75, 90, 95]
                .iter()
                .map(|p| scores[n * p / 100])
                .collect();
            let mut v = scores.iter().sum::<i64>() / i64::try_from(n).expect("n");
            let mut stops = vec![v];
            for _ in 0..4 {
                let t = v;
                v = scores.iter().map(|&s| s.max(t)).sum::<i64>() / i64::try_from(n).expect("n");
                stops.push(v);
            }
            println!("{character:?} quantiles {q:?} stopping V0.. {stops:?}");
        }
    }
}

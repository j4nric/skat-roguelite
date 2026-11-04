//! Own editions of every card (ADR 0015, DESIGN 7.1, 18): equality with
//! the shared deck for a plain deck, holder resolution, holder changes,
//! boss editions, Frisches Deck, Abgestuft, Durchsichtig.

use super::bot::RunBot;
use super::{Run, RunEvent, RunPhase, RunStart};
use crate::content::Content;
use crate::game::Event;

/// A digest of everything that decides a game: the deals (printed card and
/// id per seat), every card played, every trick and every score.
fn digest(events: &[RunEvent]) -> String {
    let mut out = String::new();
    for event in events {
        match event {
            RunEvent::Game(Event::Dealt { hands, skat, .. }) => {
                out.push('D');
                for hand in hands {
                    for c in hand {
                        out.push_str(&format!("{}:{},", c.kind, c.id));
                    }
                    out.push('|');
                }
                out.push_str(&format!(
                    "{}:{},{}:{};",
                    skat[0].kind, skat[0].id, skat[1].kind, skat[1].id
                ));
            }
            RunEvent::Game(Event::CardPlayed { seat, card }) => {
                out.push_str(&format!("P{}{}:{};", seat.index(), card.kind, card.id));
            }
            RunEvent::Game(Event::TrickWon { trick, card_points }) => {
                out.push_str(&format!("T{}:{card_points};", trick.winner.index()));
            }
            RunEvent::GameScored {
                breakdown,
                series_score,
                ..
            } => {
                out.push_str(&format!(
                    "S{}:{}:{}:{series_score};",
                    breakdown.won, breakdown.value, breakdown.outcome.declarer_card_points
                ));
            }
            _ => {}
        }
    }
    out
}

fn fnv(text: &str) -> u64 {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in text.bytes() {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x0100_0000_01b3);
    }
    hash
}

/// Play the first series of `section` with the bot (Alleinspieler, no
/// card modifications, Stammgäste only).
fn first_series(seed: u64, section: u32) -> String {
    let content = Content::load().expect("content");
    let start = RunStart {
        section,
        ..RunStart::default()
    };
    let (mut run, mut events) = Run::new_with(&content, seed, &start).expect("run");
    let mut actions = 0;
    while run.phase() == RunPhase::Game && run.series.index == 0 && actions < 600 {
        let Some(action) = RunBot.next_action(&run, &content) else {
            break;
        };
        events.extend(run.apply(&content, action).expect("legal action"));
        actions += 1;
    }
    digest(&events)
}

#[test]
fn plain_deck_deals_and_plays_as_before_editions() {
    let mut hashes = Vec::new();
    for seed in [1_u64, 7, 42, 99] {
        for section in [0, 1] {
            hashes.push(fnv(&first_series(seed, section)));
        }
    }
    // Recorded on main before the editions (53b9d29): the same seeds give
    // the same deals, cards, tricks and scores. The second district's
    // hashes were recorded again with the Kneipentour (ADR 0016): its
    // opponents (PIMC 8:3 instead of 6:2) and Zielwerte changed. Seeds 42
    // and 99 again with the steep Zielwerte of 03.11.2026: the pub ends
    // later, the deals are the same. All again on 04.11.2026: a lost game
    // counts once, which changes the scores and the bot's choices.
    assert_eq!(
        hashes,
        vec![
            8_372_553_221_233_023_206,
            12_912_900_704_559_812_833,
            2_311_626_695_843_553_887,
            4_391_168_479_812_042_796,
            8_533_871_236_692_366_623,
            1_970_303_883_358_432_031,
            11_370_352_877_980_222_910,
            2_884_972_791_164_610_454,
        ]
    );
}

// ---------------------------------------------------------------------------
// Holder resolution, boss editions, holder changes
// ---------------------------------------------------------------------------

use super::content_tests::fixed_deal;
use super::{BOSS_SEAT, PARTNER_SEAT};
use crate::cards::{Card, CardKind, PlayingCard, Rank, Suit};
use crate::content::{BossId, CardModId};
use crate::deck::{DealtCard, Owner};
use crate::game::Seat;
use crate::rules::{self, GameType};

fn content() -> Content {
    Content::load().expect("content")
}

fn card(s: &str) -> Card {
    s.parse().expect("card")
}

fn find(cards: &[DealtCard], s: &str) -> DealtCard {
    let kind = CardKind::Normal(card(s));
    *cards.iter().find(|c| c.kind == kind).expect("card held")
}

/// A run whose edition gives every card one Auge more, dealt again.
fn plus_one_run(content: &Content) -> Run {
    let (mut run, _) = Run::new(content, 5).expect("run");
    for id in 0..32 {
        run.edition
            .add_mod(content, id, &CardModId::new("points"), 1)
            .expect("Augen");
    }
    run.games_dealt = 0;
    run.deal(content, true, &mut Vec::new()).expect("deal");
    run
}

#[test]
fn every_dealt_card_shows_the_edition_of_its_holder_and_the_skat_is_the_players() {
    let content = content();
    let run = plus_one_run(&content);
    let game = &run.game().expect("game").game;
    let printed = |c: &DealtCard| c.face().map_or(0, |f| f.rank.card_points());
    for &c in game.hand(Seat::DECLARER).iter().chain(game.skat().iter()) {
        assert_eq!(c.owner, Owner::Player);
        assert_eq!(c.card_points, printed(&c) + 1, "{c}");
    }
    for seat in Seat::DEFENDERS {
        for c in game.hand(seat) {
            assert_eq!(c.owner, Owner::Guest);
            assert_eq!(
                c.card_points,
                printed(c),
                "a guest plays the plain edition: {c}"
            );
        }
    }
    // The view shows the holder's modifications and the back of the owner.
    let view = run.view(&content);
    let table = view.game.expect("game");
    assert!(
        table
            .hand
            .iter()
            .all(|c| c.mods == [CardModId::new("points")])
    );
    assert!(table.hand.iter().all(|c| c.back.is_none()));
    assert_eq!(
        table.backs,
        [
            None,
            Some("regulars".to_owned()),
            Some("regulars".to_owned())
        ]
    );
    // Hooks fire only for the owner's edition: only the player's cards are
    // sources.
    let sources = run.sources(&content, true);
    let card_sources: Vec<u32> = sources
        .iter()
        .filter_map(|s| s.this_card)
        .map(|c| c.id)
        .collect();
    assert_eq!(card_sources.len(), 12);
    assert!(card_sources.iter().all(|id| {
        game.hand(Seat::DECLARER)
            .iter()
            .chain(game.skat().iter())
            .any(|c| c.id == *id)
    }));
}

fn boss_run(content: &Content, boss: &str) -> Run {
    let start = RunStart {
        index: 2,
        ..RunStart::default()
    };
    let (mut run, _) = Run::new_with(content, 21, &start).expect("boss series");
    run.series.boss = Some(BossId::new(boss));
    run.series.ramsch_played = true;
    run
}

#[test]
fn a_boss_edition_applies_only_on_the_boss_seat() {
    let content = content();
    let mut run = boss_run(&content, "the_pusher");
    let mut events = Vec::new();
    // Every seat gets two sevens; the skat holds none.
    fixed_deal(
        &mut run,
        &content,
        [
            "D7 H7 DA DT DK DQ D9 D8 HA HT",
            "S7 C7 HK HQ H9 H8 SA ST SK SQ",
            "DJ HJ SJ CJ S9 S8 CA CT CK CQ",
        ],
        "C9 C8",
    )
    .into_iter()
    .for_each(|e| events.push(e));
    let game = &run.game().expect("game").game;
    assert_eq!(run.owner_of_seat(BOSS_SEAT), Owner::Boss);
    // "Der Schieber": his sevens count four Augen.
    for s in ["S7", "C7"] {
        let c = find(game.hand(BOSS_SEAT), s);
        assert_eq!((c.owner, c.card_points), (Owner::Boss, 4), "{s}");
    }
    for s in ["D7", "H7"] {
        let c = find(game.hand(Seat::DECLARER), s);
        assert_eq!((c.owner, c.card_points), (Owner::Player, 0), "{s}");
    }
    // ADR 0017: the second figure on the right, "Die Hehlerin" with her
    // eights at three Augen more.
    assert_eq!(run.owner_of_seat(PARTNER_SEAT), Owner::BossPartner);
    assert!(
        game.hand(PARTNER_SEAT)
            .iter()
            .all(|c| c.owner == Owner::BossPartner)
    );
    let s8 = find(game.hand(PARTNER_SEAT), "S8");
    assert_eq!(s8.card_points, 3);
    let d8 = find(game.hand(Seat::DECLARER), "D8");
    assert_eq!(d8.card_points, 0);
    // The Augen are in the breakdown under the boss's cards.
    let active = run.game().expect("game");
    assert_eq!(
        active
            .point_changes
            .iter()
            .filter(|(_, s, d)| matches!(s, crate::content::Source::CardMod { .. }) && *d == 4)
            .count(),
        2
    );
    assert_eq!(
        active
            .point_changes
            .iter()
            .filter(|(_, s, d)| matches!(s, crate::content::Source::CardMod { .. }) && *d == 3)
            .count(),
        1
    );
    // The view names the boss edition before and during the series.
    let view = run.view(&content);
    let rule = view.game.expect("game").boss_rule.expect("boss rule");
    let names: Vec<&str> = rule.figures.iter().map(|f| f.name.as_str()).collect();
    assert_eq!(names, ["Der Schieber", "Die Hehlerin"]);
    let edition = &rule.figures[0].edition;
    assert_eq!(edition.len(), 1);
    assert_eq!(edition[0].card_mod.id, CardModId::new("points"));
    assert_eq!(edition[0].filter.rank, Some(Rank::Seven));
    assert_eq!(rule.figures[1].edition[0].filter.rank, Some(Rank::Eight));
    // Both figures show the boss's back.
    assert!(rule.figures.iter().all(|f| f.back == "the_pusher"));
}

#[test]
fn every_boss_figure_but_the_old_couple_has_one_group_of_four() {
    let content = content();
    for boss in content.bosses.values() {
        assert_eq!(
            boss.figures.len(),
            crate::content::BOSS_FIGURES,
            "{}",
            boss.id
        );
        for figure in &boss.figures {
            let edition =
                crate::deck::Edition::with_mods(&figure.edition, &content).expect("edition");
            let modified = edition
                .cards()
                .iter()
                .filter(|c| !c.mods.is_empty())
                .count();
            match boss.id.as_str() {
                "the_old_man" => assert_eq!(modified, 0, "{}", figure.name),
                "black_peter" => assert_eq!(modified, 2, "{}", figure.name),
                _ => assert_eq!(modified, 4, "{}", figure.name),
            }
        }
        // Themed, but not the same: the two editions differ.
        if boss.rotation.is_empty() {
            assert_ne!(
                boss.figures[0].edition, boss.figures[1].edition,
                "{}",
                boss.id
            );
        }
    }
}

#[test]
fn the_old_man_plays_the_edition_of_the_rule_in_force() {
    let content = content();
    let mut run = boss_run(&content, "the_old_man");
    // Game 1: the rule (and edition) of "Der Schieber".
    run.series.games_played = 0;
    run.series.redeals_used = 0;
    run.games_dealt = 0;
    run.deal(&content, true, &mut Vec::new()).expect("deal");
    let active = run.game().expect("game");
    assert_eq!(active.boss_rule, Some(BossId::new("the_pusher")));
    let edition = active.boss_edition.as_ref().expect("boss edition");
    let seven = edition.id_of(card("S7")).expect("S7");
    assert_eq!(edition.get(seven).map(|c| c.mods.len()), Some(1));
    // Game 2: "Die Zwillinge", eights two steps higher.
    run.series.games_played = 1;
    run.deal(&content, true, &mut Vec::new()).expect("deal");
    let active = run.game().expect("game");
    assert_eq!(active.boss_rule, Some(BossId::new("twins")));
    for c in active.game.hand(BOSS_SEAT) {
        let eight = c.face().is_some_and(|f| f.rank == Rank::Eight);
        assert_eq!(c.rank_bonus, if eight { 2 } else { 0 }, "{c}");
    }
    // "Die Alte" on the right plays the second twin: nines two steps up.
    for c in active.game.hand(PARTNER_SEAT) {
        let nine = c.face().is_some_and(|f| f.rank == Rank::Nine);
        assert_eq!(c.rank_bonus, if nine { 2 } else { 0 }, "{c}");
    }
    let view = run.view(&content);
    // Der Alte shows his own back, not the one of the boss whose rule is on.
    let backs = view.game.expect("game").backs;
    assert_eq!(backs[1], Some("the_old_man".to_owned()));
    assert_eq!(backs[2], Some("the_old_man".to_owned()));
}

// ---------------------------------------------------------------------------
// Durchsichtig
// ---------------------------------------------------------------------------

#[test]
fn a_transparent_card_of_the_player_never_shows_the_defenders_copy() {
    // Until 04.11.2026 a "Durchsichtig" on the player's S7 showed the
    // boss's own S7 (holder and edition) to the player. Since ADR 0015 each
    // seat plays its own edition: the curse on the player's copy says
    // "the defenders see this card of yours", nothing about theirs.
    let content = content();
    let mut run = boss_run(&content, "the_pusher");
    let s7 = run.edition.id_of(card("S7")).expect("S7");
    let d7 = run.edition.id_of(card("D7")).expect("D7");
    for id in [s7, d7] {
        run.edition
            .add_mod(&content, id, &CardModId::new("transparent"), 1)
            .expect("curse");
    }
    fixed_deal(
        &mut run,
        &content,
        [
            "D7 H7 DA DT DK DQ D9 D8 HA HT",
            "S7 C7 HK HQ H9 H8 SA ST SK SQ",
            "DJ HJ SJ CJ S9 S8 CA CT CK CQ",
        ],
        "C9 C8",
    );
    let view = run.view(&content);
    let game = view.game.expect("game");
    assert!(
        game.revealed.iter().all(|r| r.card.card.id != s7),
        "the boss's S7 stays hidden: {:?}",
        game.revealed
    );
    assert!(
        game.revealed
            .iter()
            .all(|r| r.holder.is_none_or(|h| h == Seat::DECLARER)),
        "no defender card is shown"
    );
    // The boss's copy carries no mark, the player's D7 does: the
    // defenders know the player holds it.
    let active = run.game().expect("game");
    assert!(!active.game.hand(BOSS_SEAT).iter().any(|c| c.revealed));
    assert!(
        active
            .game
            .hand(Seat::DECLARER)
            .iter()
            .any(|c| c.id == d7 && c.revealed)
    );
    let shown = game
        .revealed
        .iter()
        .find(|r| r.card.card.id == d7)
        .expect("the own Durchsichtig card is listed with its holder");
    assert_eq!(shown.holder, Some(Seat::DECLARER));
    assert_eq!(
        shown.source,
        super::view::RevealSource::CardMod(CardModId::new("transparent"))
    );
}

#[test]
fn black_peter_shows_his_black_unter_only_while_he_holds_them() {
    let content = content();
    let mut run = boss_run(&content, "black_peter");
    fixed_deal(
        &mut run,
        &content,
        [
            "SJ H7 DA DT DK DQ D9 D8 HA HT",
            "CJ C7 HK HQ H9 H8 SA ST SK SQ",
            "DJ HJ D7 S7 S9 S8 CA CT CK CQ",
        ],
        "C9 C8",
    );
    let view = run.view(&content);
    let revealed: Vec<(Card, Option<Seat>)> = view
        .game
        .expect("game")
        .revealed
        .iter()
        .filter_map(|r| r.card.card.face().map(|f| (f, r.holder)))
        .collect();
    // His black Unter while he holds them, the black Oberinnen of "Die
    // Schwarze Grete" on the right (his own Grün-Oberin is plain).
    assert_eq!(
        revealed,
        vec![
            (card("CJ"), Some(BOSS_SEAT)),
            (card("CQ"), Some(PARTNER_SEAT))
        ]
    );
}

// ---------------------------------------------------------------------------
// Abgestuft
// ---------------------------------------------------------------------------

fn dealt(s: &str, demoted: bool) -> DealtCard {
    DealtCard {
        demoted,
        ..DealtCard::plain(0, CardKind::Normal(card(s)))
    }
}

#[test]
fn a_demoted_unter_is_the_lowest_card_of_its_suit() {
    let jack = dealt("DJ", true);
    let seven = dealt("D7", false);
    let diamonds = GameType::Suit(Suit::Diamonds);
    let clubs = GameType::Suit(Suit::Clubs);
    for game in [GameType::Grand, clubs, GameType::Null] {
        assert!(!rules::is_trump(game, jack), "{game}");
        // It follows its suit, and the Seven beats it.
        assert!(rules::follows_lead(game, seven, jack), "{game}");
        assert!(rules::beats(game, jack, jack, seven), "{game}");
        assert!(!rules::beats(game, seven, seven, jack), "{game}");
    }
    // In a suit game of its suit it is the lowest trump.
    assert!(rules::is_trump(diamonds, jack));
    assert!(rules::beats(diamonds, jack, jack, seven));
    assert!(!rules::beats(diamonds, seven, seven, jack));
    // A real Unter of the other side beats it in any trump game.
    assert!(rules::beats(
        GameType::Grand,
        jack,
        jack,
        dealt("SJ", false)
    ));
    assert!(rules::beats(diamonds, jack, jack, dealt("CJ", false)));
    // No Spitze: the declarer with a demoted CJ plays "ohne".
    let mut in_play: Vec<DealtCard> = crate::cards::standard_deck()
        .into_iter()
        .zip(0u32..)
        .map(|(c, id)| DealtCard::plain(id, CardKind::Normal(c)))
        .collect();
    let cj = in_play
        .iter_mut()
        .find(|c| c.kind == CardKind::Normal(card("CJ")))
        .expect("CJ");
    cj.demoted = true;
    let cj = *cj;
    let sj = *in_play
        .iter()
        .find(|c| c.kind == CardKind::Normal(card("SJ")))
        .expect("SJ");
    let m = rules::matadors_in(GameType::Grand, &[cj], &in_play);
    // In Grand three Unter remain in the order.
    assert!(!m.with && m.count == 3, "{m:?}");
    let m = rules::matadors_in(GameType::Grand, &[cj, sj], &in_play);
    assert!(m.with && m.count == 1, "{m:?}");
}

#[test]
fn another_demoted_card_counts_one_step_lower() {
    let ace = dealt("DA", true);
    let ten = dealt("DT", false);
    // Equal strength: the card played first keeps the trick.
    assert!(!rules::beats(GameType::Grand, ace, ace, ten));
    assert!(!rules::beats(GameType::Grand, ten, ten, ace));
    let king = dealt("DK", false);
    assert!(rules::beats(GameType::Grand, king, king, ace));
    // As a trump too.
    let diamonds = GameType::Suit(Suit::Diamonds);
    assert!(!rules::beats(diamonds, ten, ten, ace));
    assert!(!rules::beats(diamonds, ace, ace, dealt("DT", false)));
    // With "Rang" next to it the steps cancel.
    let ranked = DealtCard {
        rank_bonus: 1,
        ..ace
    };
    assert!(rules::beats(GameType::Grand, ten, ten, ranked));
}

#[test]
fn the_null_player_holds_demoted_unter_the_defenders_real_ones() {
    let content = content();
    let start = RunStart {
        character: crate::content::Character::NullPlayer,
        ..RunStart::default()
    };
    let (mut run, _) = Run::new_with(&content, 8, &start).expect("run");
    // An Augen modification may sit next to the special curse.
    let cj = run.edition.id_of(card("CJ")).expect("CJ");
    run.edition
        .add_mod(&content, cj, &CardModId::new("points"), 3)
        .expect("Augen next to Abgestuft");
    fixed_deal(
        &mut run,
        &content,
        [
            "CJ SJ D7 H7 S7 C7 D8 H8 S8 C8",
            "HJ DJ D9 DT DQ DK DA H9 HT HQ",
            "HK HA S9 ST SQ SK SA C9 CT CQ",
        ],
        "CK CA",
    );
    let game = &run.game().expect("game").game;
    let mine = find(game.hand(Seat::DECLARER), "CJ");
    assert!(mine.demoted);
    assert_eq!(mine.card_points, 3, "0 Augen as a demoted Unter, +3 Augen");
    let sj = find(game.hand(Seat::DECLARER), "SJ");
    assert!(sj.demoted && sj.card_points == 0);
    let theirs = find(game.hand(Seat::DEFENDERS[0]), "HJ");
    assert!(!theirs.demoted && theirs.card_points == 2);
    let view = run.view(&content);
    let shown = view
        .game
        .expect("game")
        .hand
        .into_iter()
        .find(|c| c.card.id == cj)
        .expect("in hand");
    assert_eq!(
        shown.mods,
        vec![CardModId::new("demoted"), CardModId::new("points")]
    );
    // Frisches Deck takes both off.
    run.edition.cleanse(cj).expect("cleanse");
    assert!(run.edition.get(cj).is_some_and(|c| c.mods.is_empty()));
}

// ---------------------------------------------------------------------------
// Fixtures of the web smoke test
// ---------------------------------------------------------------------------

/// Bring the saved runs of `web/e2e/fixtures` to the save format of own
/// editions (version 2, ADR 0015): the shared deck becomes the player's
/// edition, the game in progress is resolved card by card by its holder.
/// The web app itself discards older runs (`OutdatedSave`); the fixtures
/// keep their scenes. Run by hand: `cargo test -p skat_core
/// regenerate_fixtures -- --ignored`.
#[test]
#[ignore = "rewrites web/e2e/fixtures"]
fn regenerate_fixtures() {
    let content = content();
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../web/e2e/fixtures");
    let mut names: Vec<_> = std::fs::read_dir(&dir)
        .expect("fixtures")
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|x| x == "json"))
        .collect();
    names.sort();
    for path in names {
        let text = std::fs::read_to_string(&path).expect("read");
        let mut save: serde_json::Value = serde_json::from_str(&text).expect("json");
        // These two stay old saves: they show that such a run is
        // discarded (skat_wasm).
        let kept_old = ["discard-battered.json", "honorary-jack.json"];
        if kept_old.iter().any(|k| path.ends_with(k)) {
            continue;
        }
        if save["version"] != 1 || !save["run"].is_object() {
            continue;
        }
        let run_json = save["run"].as_object_mut().expect("run");
        let deck = run_json.remove("deck").expect("deck");
        run_json.insert("edition".to_owned(), deck);
        let mut run: Run = serde_json::from_value(save["run"].clone()).expect("run");
        run.adopt_editions(&content);
        let out = serde_json::json!({ "version": 2, "run": run });
        std::fs::write(&path, serde_json::to_string(&out).expect("json")).expect("write");
        println!("upgraded {}", path.display());
    }
}

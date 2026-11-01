//! Rule tests against the Internationale Skatordnung (ISkO): trick winner,
//! Bedienpflicht (follow_suit), trump order, Null order, matadors over the
//! declarer's twelve cards in a real game flow, and the skat counting for
//! the declarer.
//!
//! Every test names the rule it checks. Deliberate deviations of the
//! roguelike (DESIGN.md 4.3, ADRs) are not tested as errors here.

use skat_core::cards::{Card, Suit, parse_cards};
use skat_core::game::{Event, Game, Phase, Seat};
use skat_core::rules::{
    Declaration, GameType, Matadors, RiskLevel, beats, game_value, legal_moves, trick_winner,
};

fn c(s: &str) -> Card {
    s.parse().expect("test card")
}

fn cards(s: &str) -> Vec<Card> {
    parse_cards(s).expect("test cards")
}

// ---------------------------------------------------------------------------
// Trick winner and trump order
// ---------------------------------------------------------------------------

// ISkO trump order in a suit game: CJ > SJ > HJ > DJ > A > 10 > K > Q > 9 >
// 8 > 7 of the trump suit. The jack of the trump suit ranks as a jack, not
// as a card of its suit.
#[test]
fn suit_game_trump_suit_jack_ranks_among_jacks_above_trump_ace() {
    let hearts = GameType::Suit(Suit::Hearts);
    // HA led, HJ beats it, DJ does not beat HJ.
    assert_eq!(trick_winner(hearts, 0, [c("HA"), c("HJ"), c("DJ")]), 1);
    // DJ (lowest jack) still beats the trump ace.
    assert_eq!(trick_winner(hearts, 0, [c("HA"), c("HT"), c("DJ")]), 2);
    // Full jack order in one trick each.
    assert_eq!(trick_winner(hearts, 0, [c("DJ"), c("HJ"), c("SJ")]), 2);
    assert_eq!(trick_winner(hearts, 0, [c("CJ"), c("HJ"), c("SJ")]), 0);
}

// ISkO rank order in suit and grand games: Ass > Zehn > König > Dame > 9 >
// 8 > 7, for trumps as well as for plain suits. Ten above king.
#[test]
fn ten_beats_king_in_trump_and_in_plain_suit() {
    let clubs = GameType::Suit(Suit::Clubs);
    // Trump suit.
    assert_eq!(trick_winner(clubs, 0, [c("CK"), c("CT"), c("CQ")]), 1);
    assert_eq!(trick_winner(clubs, 0, [c("CT"), c("CK"), c("CA")]), 2);
    // Plain suit in a suit game.
    assert_eq!(trick_winner(clubs, 0, [c("HK"), c("H9"), c("HT")]), 2);
    // Plain suit in grand.
    assert_eq!(
        trick_winner(GameType::Grand, 0, [c("SK"), c("ST"), c("SQ")]),
        1
    );
}

// ISkO grand: only the four jacks are trump, CJ > SJ > HJ > DJ, and any
// jack beats the ace of the led suit.
#[test]
fn grand_jacks_are_the_only_trumps() {
    let grand = GameType::Grand;
    assert!(beats(grand, c("SA"), c("SA"), c("DJ")));
    assert!(!beats(grand, c("SA"), c("SA"), c("CA")));
    assert_eq!(trick_winner(grand, 1, [c("HJ"), c("DJ"), c("CJ")]), 0);
    assert_eq!(trick_winner(grand, 2, [c("SJ"), c("HJ"), c("DJ")]), 2);
    // Ace of the led suit wins without any jack in the trick.
    assert_eq!(trick_winner(grand, 0, [c("CT"), c("CA"), c("HA")]), 1);
}

// ISkO trick rule: a card that neither follows nor trumps never wins, the
// lead card wins if nobody follows or trumps.
#[test]
fn slough_never_wins_the_trick() {
    let spades = GameType::Suit(Suit::Spades);
    assert_eq!(trick_winner(spades, 1, [c("D7"), c("HA"), c("CA")]), 1);
    assert_eq!(
        trick_winner(GameType::Grand, 2, [c("H7"), c("SA"), c("DA")]),
        2
    );
}

// ISkO Null: no trumps, natural order 7 8 9 10 B D K A in every suit.
#[test]
fn null_rank_order_is_seven_to_ace_without_trumps() {
    let order = cards("S7 S8 S9 ST SJ SQ SK SA");
    for pair in order.windows(2) {
        assert!(
            beats(GameType::Null, pair[0], pair[0], pair[1]),
            "{} must beat {} in Null",
            pair[1],
            pair[0]
        );
        assert!(!beats(GameType::Null, pair[0], pair[1], pair[0]));
    }
    // The jack is a plain card: it does not beat a led ace of its suit and
    // never beats a card of another suit.
    assert_eq!(
        trick_winner(GameType::Null, 0, [c("SA"), c("SJ"), c("CJ")]),
        0
    );
}

// ---------------------------------------------------------------------------
// Bedienpflicht (follow_suit)
// ---------------------------------------------------------------------------

// ISkO Bedienpflicht in a suit game: the jack of the trump suit is a trump,
// it must be played on a trump lead.
#[test]
fn follow_suit_trump_suit_jack_must_follow_trump_lead() {
    let hearts = GameType::Suit(Suit::Hearts);
    let hand = cards("HJ S7 C8");
    assert_eq!(legal_moves(hearts, &hand, Some(c("HA"))), cards("HJ"));
    assert_eq!(legal_moves(hearts, &hand, Some(c("CJ"))), cards("HJ"));
}

// ISkO Bedienpflicht in a suit game: a jack never follows the plain suit
// printed on it. Spade lead in a heart game with SJ as only "spade" means
// the player cannot follow and may play anything.
#[test]
fn follow_suit_jack_does_not_count_as_its_printed_suit() {
    let hearts = GameType::Suit(Suit::Hearts);
    let hand = cards("SJ H7 C8");
    assert_eq!(legal_moves(hearts, &hand, Some(c("SA"))).len(), 3);
    // With a real spade in hand only that spade is legal, not the SJ.
    let hand = cards("SJ S9 H7");
    assert_eq!(legal_moves(hearts, &hand, Some(c("SA"))), cards("S9"));
}

// ISkO Bedienpflicht in grand: jacks form their own trump suit; a suit
// lead is followed only by non-jacks of that suit.
#[test]
fn follow_suit_grand_jack_is_not_part_of_its_suit() {
    let grand = GameType::Grand;
    let hand = cards("CJ CA");
    // Club lead: CA must be played, CJ is not a club.
    assert_eq!(legal_moves(grand, &hand, Some(c("C7"))), cards("CA"));
    // Spade lead, no spade (SJ is trump): free choice including SJ.
    let hand = cards("SJ HA");
    assert_eq!(legal_moves(grand, &hand, Some(c("S7"))).len(), 2);
    // Jack lead: only jacks follow, even with clubs in hand.
    let hand = cards("DJ C7 CA");
    assert_eq!(legal_moves(grand, &hand, Some(c("CJ"))), cards("DJ"));
}

// ISkO Bedienpflicht in Null: no trumps, the jack belongs to its suit.
#[test]
fn follow_suit_null_jack_belongs_to_its_suit() {
    let hand = cards("SJ H7");
    assert_eq!(
        legal_moves(GameType::Null, &hand, Some(c("SA"))),
        cards("SJ")
    );
    // A jack lead in Null asks for the jack's suit, not for jacks.
    let hand = cards("DJ C7");
    assert_eq!(
        legal_moves(GameType::Null, &hand, Some(c("CJ"))),
        cards("C7")
    );
}

// ---------------------------------------------------------------------------
// Matadors and skat in a real game flow
// ---------------------------------------------------------------------------

/// Deal in which the declarer holds HJ but CJ and SJ lie in the skat.
/// Over the ten hand cards the declarer is "ohne 2" in grand, over the
/// twelve cards (hand plus skat) he is "mit 3".
fn deal_with_top_jacks_in_skat() -> Game {
    let hands = [
        cards("HJ CA CT CK SA ST SK HA HT HK"),
        cards("DJ CQ C9 C8 C7 SQ S9 S8 S7 HQ"),
        cards("H9 H8 H7 DA DT DK DQ D9 D8 D7"),
    ];
    Game::with_cards(hands, [c("CJ"), c("SJ")], Seat::DECLARER).expect("valid deal")
}

/// Play every remaining card with the first legal move until the game ends.
fn play_out(game: &mut Game) {
    while let Some(seat) = game.current_seat() {
        let card = *game.legal_moves(seat).first().expect("a legal move");
        game.play(seat, card).expect("legal play");
    }
    assert!(matches!(game.phase(), Phase::Finished(_)));
}

fn finished_matadors(game: &Game) -> Matadors {
    game.log()
        .iter()
        .find_map(|e| match e {
            Event::GameFinished { matadors, .. } => Some(*matadors),
            _ => None,
        })
        .expect("GameFinished event")
}

const MIT_3: Matadors = Matadors {
    with: true,
    count: 3,
};

// ISkO Spitzen: counted over hand plus skat, also in a hand game where the
// skat is never picked up.
#[test]
fn matadors_count_untouched_skat_in_hand_game() {
    let mut game = deal_with_top_jacks_in_skat();
    let decl = Declaration::grand(RiskLevel::Hand);
    game.declare(decl).expect("declare");
    play_out(&mut game);
    assert_eq!(finished_matadors(&game), MIT_3);
    let value = game.value().expect("finished");
    assert_eq!(value, game_value(decl, MIT_3, &game.outcome()));
}

// ISkO Spitzen: counted over hand plus skat, a discarded jack still counts.
#[test]
fn matadors_count_discarded_jack() {
    let mut game = deal_with_top_jacks_in_skat();
    let decl = Declaration::suit(Suit::Hearts, RiskLevel::Normal);
    game.declare(decl).expect("declare");
    // Discard the Kreuz-Bube: the declarer still plays "mit 3".
    game.discard([c("CJ"), c("CA")]).expect("discard");
    play_out(&mut game);
    assert_eq!(finished_matadors(&game), MIT_3);
}

// ISkO: the discarded cards (gedrückte Karten) count for the declarer.
#[test]
fn discarded_card_points_count_for_declarer() {
    let mut game = deal_with_top_jacks_in_skat();
    game.declare(Declaration::grand(RiskLevel::Normal))
        .expect("declare");
    game.discard([c("CA"), c("CT")]).expect("discard");
    // Before any trick the declarer already owns the 21 Augen in the skat.
    assert_eq!(game.outcome().declarer_card_points, 21);
    play_out(&mut game);
    let outcome = game.outcome();
    assert_eq!(
        outcome.declarer_card_points,
        game.trick_card_points()[0] + 21
    );
    assert_eq!(
        outcome.declarer_card_points + outcome.defender_card_points,
        120
    );
}

// ISkO: in a hand game the untouched skat counts for the declarer.
#[test]
fn untouched_skat_counts_for_declarer_in_hand_game() {
    let mut game = deal_with_top_jacks_in_skat();
    game.declare(Declaration::grand(RiskLevel::Hand))
        .expect("declare");
    // CJ + SJ in the skat: 4 Augen.
    assert_eq!(game.outcome().declarer_card_points, 4);
    play_out(&mut game);
    assert_eq!(
        game.outcome().declarer_card_points,
        game.trick_card_points()[0] + 4
    );
}

// ISkO Schwarz: the skat is not a trick. A declarer who takes no trick is
// Schwarz even though the skat gives him Augen.
#[test]
fn skat_is_not_a_trick_for_schwarz() {
    let mut game = deal_with_top_jacks_in_skat();
    game.declare(Declaration::grand(RiskLevel::Hand))
        .expect("declare");
    assert_eq!(game.outcome().declarer_tricks, 0);
    assert!(game.outcome().declarer_card_points > 0);
}

//! The bot's estimates of a hand: win rates by trumps and exposed cards
//! (measured with the PIMC player), the expected score of a suit game,
//! grand or Null, the score of a ten-card hand for Einmischen, and the
//! chance of Schneider by double-dummy samples.

use crate::ai::{null_unsafe_cards, trump_count};
use crate::cards::{Card, PlayingCard, Rank, Suit};
use crate::deck::DealtCard;
use crate::game::Seat;
use crate::rng::mix64;
use crate::rules::{Declaration, GameType, NullRisk, NullScoring, RiskLevel};

use super::count_rank;

/// The Nullspieler plays a hand with at most this many exposed cards
/// (before the skat). Measured with the Null formula (DESIGN 9.2, 3 × 1000
/// runs): with 2 the bot reaches section 2 in 15 %, with 4 in 28 %, with 5
/// in 35 %. Hands with 3 or 4 exposed cards win 60 % and 45 % of their
/// games. M5 (`docs/balance/2026-11-01-m5-v1.md`, PIMC card play, 8 games
/// and 6 passes per series): section 3 in 20.1 % with 4, 25.1 % with 5 and
/// 24.8 % with 6 (6 plays a third more games per run). 5 is the stronger
/// policy, and the bot stands in for a good player.
pub const NULL_MAX_EXPOSED: usize = 5;

/// Under forced hand play ("Der Schieber") the bot declares Null Hand only
/// with at most this many exposed cards: it cannot discard them. Null after
/// the skat wins 74 % with one and 51 % with two exposed cards left, 28 %
/// with three ([`NULL_WIN_PERCENT`]).
pub const NULL_HAND_MAX_EXPOSED: usize = 2;

/// Percent of Null games the bot wins by exposed cards in the ten cards it
/// plays (index; 0 is played Ouvert). Measured with the PIMC player
/// (`--bot pimc:12:3`, 01.11.2026), Null after the skat: the Nullspieler
/// with eight sevens won 94, 74, 51, 28, 19 and 7 % for 0 to 5 (600 runs,
/// seed 99, 5300 games), the Alleinspieler with the normal deck 99, 79, 58,
/// 41, 33 and 21 % (300 runs, seed 42, 1100 games). The table is the mean.
/// More than 7: 0.
pub const NULL_WIN_PERCENT: [i64; 8] = [96, 76, 54, 34, 26, 14, 7, 3];

/// Percent of suit games the bot wins, by trumps (index, up to 8) and aces
/// (up to 3) of the ten cards it plays, in a deck with all eleven trumps
/// (the Alleinspieler, 600 runs, seed 99, 2516 suit games); hands with
/// fewer than five trumps are rare (the bot passes them) and rounded. A
/// deck with fewer trumps counts each missing one as held (the
/// Nullspieler's suits have eight: with four or five he won 92 %).
pub const SUIT_WIN_PERCENT: [[i64; 4]; 9] = [
    [0, 0, 5, 10],
    [0, 0, 5, 10],
    [0, 5, 10, 15],
    [0, 5, 15, 20],
    [5, 20, 44, 50],
    [5, 23, 52, 85],
    [55, 74, 81, 93],
    [94, 94, 97, 98],
    [98, 98, 98, 98],
];

/// Trumps of a suit game in the standard deck: four jacks and seven cards.
const SUIT_TRUMPS: usize = 11;

/// Percent of grands the bot wins by jacks (index) and aces (up to 3), same
/// measurement (1650 grands, almost all with two jacks and two aces or
/// more).
pub const GRAND_WIN_PERCENT: [[i64; 4]; 5] = [
    [0, 0, 2, 5],
    [0, 2, 5, 10],
    [5, 20, 67, 89],
    [20, 60, 93, 98],
    [90, 98, 98, 98],
];

/// Expected style bonus of a won Null in hundredths of a Gewinnstufe:
/// a ducked seven per seven held, plus this for "nie bedient" and
/// "Kurzschluss". Measured: a won Null with the skat counts on average
/// 23 × (1.3 + sevens) (Nullspieler, same measurement).
const NULL_STYLE_EXTRA: i64 = 30;

fn percent(table: &[i64], index: usize) -> i64 {
    table.get(index).copied().unwrap_or(0)
}

/// Expected score of a Null with these ten cards, in hundredths of a point
/// (won: the value; lost: minus the value times
/// [`crate::content::RUN_LOSS_FACTOR`], as in the series account), without relics. `exposed` counts the exposed cards
/// ([`null_unsafe_cards`]). The Null formula (DESIGN 9.2) counts the
/// expected style bonus, each seven held as one ducked trick.
#[must_use]
pub fn null_expected_score(
    hand: &[DealtCard],
    exposed: usize,
    declaration: Declaration,
    scoring: &NullScoring,
) -> i64 {
    let Some(risk) = declaration.null_risk() else {
        return 0;
    };
    let win = percent(&NULL_WIN_PERCENT, exposed);
    let (won, lost) = match scoring {
        NullScoring::Fixed => {
            let value = i64::from(crate::rules::null_value(risk.hand(), risk.ouvert())) * 100;
            (value, crate::content::RUN_LOSS_FACTOR * value)
        }
        NullScoring::Formula(f) => {
            let base = i64::from(f.base_value);
            let step = i64::from(risk.step()) * 100;
            let sevens = i64::try_from(count_rank(hand, Rank::Seven)).unwrap_or(0);
            let style = sevens * i64::from(f.per_seven_ducked) * 100 + NULL_STYLE_EXTRA;
            (
                base * (step + style),
                crate::content::RUN_LOSS_FACTOR * base * step,
            )
        }
    };
    (win * won - (100 - win) * lost) / 100
}

/// Expected score of a suit game or grand with these ten cards, in
/// hundredths of a point, without relics: the Spielwert with the Spitzen
/// of `all_cards` (the ten plus the skat, which counts for the Spitzen),
/// won with [`SUIT_WIN_PERCENT`] or [`GRAND_WIN_PERCENT`], lost
/// [`crate::content::RUN_LOSS_FACTOR`] times (once since 04.11.2026).
/// `deck` is every card of the game (for trumps missing from the deck).
/// `base` is the run's Grundwert of the game
/// ([`crate::content::RunConfig::base_value`], Grand 18); 0 gives 0.
///
/// `win_threshold` is the Augen the game needs (`61 + stake`, lowered by
/// relics; [`crate::rules::Thresholds::win`]). The tables are measured at
/// 61; above it the win chance shrinks with the Augen left over, `(120 −
/// threshold) / 59` of it (a rough model, not measured), so that at high
/// stakes Null, which counts no Augen, wins the comparison more often.
/// Below 61 the tables are kept.
#[must_use]
pub fn suit_expected_score(
    hand: &[DealtCard],
    all_cards: &[DealtCard],
    deck: &[DealtCard],
    declaration: Declaration,
    base: u32,
    win_threshold: u32,
) -> i64 {
    let game = declaration.game();
    if game == GameType::Null || base == 0 {
        return 0;
    }
    let aces = count_rank(hand, Rank::Ace).min(3);
    let win = match game {
        GameType::Grand => GRAND_WIN_PERCENT
            .get(count_rank(hand, Rank::Jack).min(4))
            .map_or(0, |row| percent(row, aces)),
        _ => {
            let missing = SUIT_TRUMPS.saturating_sub(trump_count(game, deck));
            SUIT_WIN_PERCENT
                .get((trump_count(game, hand) + missing).min(8))
                .map_or(0, |row| percent(row, aces))
        }
    };
    let total = i64::from(crate::rules::TOTAL_CARD_POINTS);
    let normal = i64::from(crate::rules::WIN_THRESHOLD);
    let needed = i64::from(win_threshold).max(normal);
    let win = win * (total - needed).max(0) / (total - normal);
    let faces: Vec<Card> = all_cards.iter().filter_map(|c| c.face()).collect();
    let matadors = crate::rules::matadors(game, &faces).count;
    let levels = matadors + 1 + u32::from(declaration.hand());
    let value = i64::from(base) * i64::from(levels) * 100;
    (win * value - (100 - win) * crate::content::RUN_LOSS_FACTOR * value) / 100
}

/// Exposed cards the skat is assumed to take away from a Null before the
/// skat is seen ([`ten_card_score`]): the two the bot discards.
const NULL_SKAT_DISCARDS: usize = 2;

/// Expected score of ten cards before the skat, in hundredths of a point,
/// without relics: the best of the four suit games, the grand
/// ([`suit_expected_score`] with the run's Grundwert `grand_base`) and,
/// with `null`, a Null after the skat with [`NULL_SKAT_DISCARDS`] exposed
/// cards fewer ([`null_expected_score`]). A rough measure of how good a
/// deal is, for the redeal decision ([`RunBot::redeal_threshold`](super::RunBot::redeal_threshold)); the
/// skat is not counted, so every hand is a little better than this.
#[must_use]
pub fn ten_card_score(
    hand: &[DealtCard],
    deck: &[DealtCard],
    grand_base: u32,
    null: Option<&NullScoring>,
) -> i64 {
    let suits = Suit::ALL.iter().map(|&s| {
        let declaration = Declaration::suit(s, RiskLevel::Normal);
        let base = GameType::Suit(s).base_value().unwrap_or(0);
        suit_expected_score(hand, hand, deck, declaration, base, 61)
    });
    let grand = suit_expected_score(
        hand,
        hand,
        deck,
        Declaration::grand(RiskLevel::Normal),
        grand_base,
        61,
    );
    let null = null.map_or(i64::MIN, |scoring| {
        let exposed = null_unsafe_cards(hand, deck).saturating_sub(NULL_SKAT_DISCARDS);
        null_expected_score(hand, exposed, Declaration::null(NullRisk::Normal), scoring)
    });
    suits.chain([grand, null]).max().unwrap_or(0)
}

/// Deals of the unseen cards [`schneider_percent`] solves.
pub const BOLD_SAMPLES: u64 = 12;

/// The least chance of Schneider (percent of [`BOLD_SAMPLES`] deals) for
/// which the bot announces it with "Große Klappe" or plays Ouvert with
/// "Offenes Visier" ([`RunBot::bold_announcement`](super::RunBot::bold_announcement)). High on purpose: the
/// double-dummy deals see every card, the bot in play does not.
pub const BOLD_MIN_PERCENT: i64 = 80;

/// Nodes per deal after which [`schneider_percent`] estimates the rest
/// (keeps the search cheap; the estimate is the solver's static one).
const BOLD_NODES: u64 = 20_000;

/// Percent of [`BOLD_SAMPLES`] deals of the unseen cards on which the
/// declarer makes Schneider in `game` with the ten cards `kept`, solved
/// double-dummy ([`crate::ai::solver`]): the defenders' Augen at most
/// `schneider_limit`, the skat counted for the declarer. `cards` are the
/// twelve cards (kept and skat), `deck` every card of the game, `forehand`
/// leads the first trick. The deals come from a generator seeded with
/// `seed` and the cards of the declarer (in any order), so the same
/// decision gives the same answer in every phase. Stops early once the
/// deals left cannot lift it to [`BOLD_MIN_PERCENT`] (the deals not solved
/// count as missed). 0 if the unseen cards are not two
/// hands of ten ("Drei drücken").
#[must_use]
pub fn schneider_percent(
    game: GameType,
    kept: &[DealtCard],
    cards: &[DealtCard],
    deck: &[DealtCard],
    forehand: Seat,
    schneider_limit: u32,
    seed: u64,
) -> i64 {
    use crate::ai::solver::{Goal, Layout, Limits, probe_with, with_table};
    let unseen: Vec<DealtCard> = deck
        .iter()
        .copied()
        .filter(|c| !cards.contains(c))
        .collect();
    if kept.len() != 10 || unseen.len() != 20 {
        return 0;
    }
    let pool: u32 = kept.iter().chain(&unseen).map(|c| c.card_points()).sum();
    let need = i32::try_from(pool.saturating_sub(schneider_limit)).unwrap_or(i32::MAX);
    let key = cards
        .iter()
        .fold(seed, |acc, c| acc.wrapping_add(mix64(u64::from(c.id))));
    let mut rng = crate::rng::Rng::from_seed(mix64(key));
    let samples = i64::try_from(BOLD_SAMPLES).unwrap_or(1);
    // Misses after which the percent is below BOLD_MIN_PERCENT for sure.
    let allowed = samples - (BOLD_MIN_PERCENT * samples + 99) / 100;
    let limits = Limits {
        depth: 0,
        nodes: BOLD_NODES,
    };
    let made = with_table(16, |table| {
        let (mut made, mut missed) = (0, 0);
        for _ in 0..BOLD_SAMPLES {
            if missed > allowed {
                break;
            }
            let mut shuffled = unseen.clone();
            rng.shuffle(&mut shuffled);
            let (first, second) = shuffled.split_at(10);
            let layout = Layout {
                hands: [kept.to_vec(), first.to_vec(), second.to_vec()],
                trick_lead: forehand,
                trick: Vec::new(),
                skat_points: 0,
            };
            let (moves, _) = probe_with(game, Goal::DeclarerPoints, &layout, need, limits, table);
            let reached = if forehand.is_declarer() {
                moves.iter().any(|&(_, r)| r)
            } else {
                !moves.is_empty() && moves.iter().all(|&(_, r)| r)
            };
            made += i64::from(reached);
            missed += i64::from(!reached);
        }
        made
    });
    made * 100 / samples
}

//! Endgame bounds: what the remaining tricks of a game can still bring,
//! over every legal way all three seats may play them (not only best
//! play). The run uses them to tell whether the rest of a game can still
//! change its score ([`crate::run::RunAction::SettleGame`]).
//!
//! Every quantity is a [`Span`]: a smallest and a largest value such that
//! every line of play stays inside. Three sources, from cheap to dear:
//!
//! - [`loose_reach`]: from the cards alone, as wide as they allow;
//! - [`null_reach`]: a Null the declarer can no longer lose, by a simple
//!   test on the hands;
//! - [`reach`]: exact spans by walking every legal card of every seat
//!   with all hands known, with a memo at trick boundaries (what the
//!   remaining tricks can bring depends only on the cards still held and
//!   on who leads), equivalent cards of one hand searched once (same
//!   kind, Augen and seven-ness, no live card of another seat between
//!   them; off when the winner of the tenth trick is asked for) and a node
//!   limit past which it gives up (`None`), never guesses.
//!
//! The spans are each sound on their own; they do not say which
//! combinations occur together.

use std::collections::HashMap;

use crate::ai::solver::{BLANK, card_kind, card_strength};
use crate::cards::{PlayingCard, Rank};
use crate::game::{Game, Phase, Seat, TRICKS_PER_GAME, seat_at};
use crate::rules::{self, GameType};

/// Smallest and largest value of a quantity over all lines of play.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Span {
    /// Smallest value.
    pub min: u32,
    /// Largest value.
    pub max: u32,
}

impl Span {
    /// A span of one value.
    #[must_use]
    pub const fn exactly(value: u32) -> Span {
        Span {
            min: value,
            max: value,
        }
    }

    /// Whether every line gives the same value.
    #[must_use]
    pub const fn fixed(self) -> bool {
        self.min == self.max
    }

    /// Every value from `min` to `max`, shifted by `base`.
    pub fn values(self, base: u32) -> impl Iterator<Item = u32> {
        (self.min..=self.max).map(move |v| v.saturating_add(base))
    }

    /// The span with `by` added to both ends.
    #[must_use]
    pub fn shift(self, by: u32) -> Span {
        Span {
            min: self.min.saturating_add(by),
            max: self.max.saturating_add(by),
        }
    }

    fn join(self, other: Span) -> Span {
        Span {
            min: self.min.min(other.min),
            max: self.max.max(other.max),
        }
    }
}

/// What to search for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReachOptions {
    /// Collect the possible winners of the tenth trick (card and seat).
    /// Switches off the merging of equivalent cards.
    pub last_trick: bool,
    /// Count the defenders' Augen in the tricks before this one (counted
    /// from the first trick of the game; the Null "Kurzschluss").
    pub early_tricks: usize,
    /// Trick boundaries after which the search gives up.
    pub node_limit: u64,
}

impl Default for ReachOptions {
    fn default() -> ReachOptions {
        ReachOptions {
            last_trick: false,
            early_tricks: 0,
            node_limit: 20_000,
        }
    }
}

/// How the game may end: the winning card and seat of the tenth trick, or
/// `None` for a game that ends earlier (a Null the declarer loses).
pub type LastTrick<C> = Option<(C, Seat)>;

/// Most different last tricks collected before the list counts as "many".
const MAX_LAST: usize = 8;

/// What the remaining tricks (the one on the table included) can bring,
/// counted from now on. See the module docs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Reach<C> {
    /// Augen the declarer takes.
    pub declarer_points: Span,
    /// Augen the defenders take.
    pub defender_points: Span,
    /// Tricks the declarer takes.
    pub declarer_tricks: Span,
    /// Tricks still played (fewer than all in a Null the declarer loses).
    pub tricks: Span,
    /// Tricks in which the declarer follows the suit of another seat's
    /// lead (Null style "never followed").
    pub followed: Span,
    /// Tricks the declarer does not take although he plays a seven in
    /// them (Null style "sevens ducked").
    pub sevens_ducked: Span,
    /// Augen the defenders take in tricks before
    /// [`ReachOptions::early_tricks`].
    pub early_defender_points: Span,
    /// The possible endings ([`LastTrick`]), sorted; `None` if not asked
    /// for or more than a handful.
    pub last_tricks: Option<Vec<LastTrick<C>>>,
    /// Trick boundaries searched.
    pub nodes: u64,
}

/// Bounds of the remaining tricks of `game` over all legal play. `None`
/// outside play or when the search needs more than
/// [`ReachOptions::node_limit`] trick boundaries.
#[must_use]
pub fn reach<C: PlayingCard>(game: &Game<C>, options: ReachOptions) -> Option<Reach<C>> {
    if game.phase() != Phase::Playing {
        return None;
    }
    let declaration = game.declaration()?;
    let mut search = Search::new(game, declaration.game(), declaration.is_null(), options)?;
    let trick: Vec<u8> = game
        .current_trick()
        .iter()
        .map(|c| search.index_of(*c))
        .collect::<Option<_>>()?;
    let mut cards = [0u8; 3];
    cards[..trick.len()].copy_from_slice(&trick);
    let hands = search.hands;
    let done = game.tricks().len();
    let sum = search.trick(hands, game.trick_lead().index(), cards, trick.len(), done)?;
    let last_tricks = match sum.last {
        Last::Set(list) => Some(
            list.iter()
                .map(|&code| search.decode_last(code))
                .collect::<Vec<_>>(),
        ),
        Last::Untracked | Last::Many => None,
    };
    Some(Reach {
        declarer_points: sum.declarer_points,
        defender_points: sum.defender_points,
        declarer_tricks: sum.declarer_tricks,
        tricks: sum.tricks,
        followed: sum.followed,
        sevens_ducked: sum.sevens,
        early_defender_points: sum.early,
        last_tricks: last_tricks.map(|mut l| {
            l.sort();
            l
        }),
        nodes: search.nodes,
    })
}

/// Augen of the cards still in the hands and on the table.
#[must_use]
pub fn live_points<C: PlayingCard>(game: &Game<C>) -> u32 {
    Seat::ALL
        .iter()
        .flat_map(|&s| game.hand(s).iter())
        .chain(game.current_trick().iter())
        .map(|c| c.card_points())
        .sum()
}

/// Tricks not yet complete, the one on the table included.
fn tricks_left<C: PlayingCard>(game: &Game<C>) -> u32 {
    u32::try_from(TRICKS_PER_GAME.saturating_sub(game.tricks().len())).unwrap_or(0)
}

/// Whether a card is a printed seven.
fn is_seven<C: PlayingCard>(card: C) -> bool {
    card.face().is_some_and(|f| f.rank == Rank::Seven)
}

/// The declarer's card in the current trick, if he played one.
fn declarer_on_table<C: PlayingCard>(game: &Game<C>) -> Option<(usize, C)> {
    let offset = (3 - game.trick_lead().index()) % 3;
    game.current_trick().get(offset).map(|&c| (offset, c))
}

/// Sevens the declarer can still play into a trick: in his hand and on
/// the table in the trick in progress.
fn sevens_left<C: PlayingCard>(game: &Game<C>) -> u32 {
    let in_hand = game
        .hand(Seat::DECLARER)
        .iter()
        .filter(|c| is_seven(**c))
        .count();
    let on_table = declarer_on_table(game).is_some_and(|(_, c)| is_seven(c));
    u32::try_from(in_hand).unwrap_or(0) + u32::from(on_table)
}

/// Bounds without any search, from the cards alone: wide, but sound.
/// Every quantity lies in its span in every line of play; the spans are
/// as wide as the cards still in play allow. `None` outside play.
#[must_use]
pub fn loose_reach<C: PlayingCard>(game: &Game<C>, early_tricks: usize) -> Option<Reach<C>> {
    if game.phase() != Phase::Playing {
        return None;
    }
    let null = game.declaration()?.is_null();
    let live = live_points(game);
    let left = tricks_left(game);
    let points = Span { min: 0, max: live };
    Some(Reach {
        declarer_points: points,
        defender_points: points,
        declarer_tricks: Span {
            min: 0,
            max: if null { left.min(1) } else { left },
        },
        tricks: if null {
            Span {
                min: left.min(1),
                max: left,
            }
        } else {
            Span::exactly(left)
        },
        followed: Span { min: 0, max: left },
        sevens_ducked: Span {
            min: 0,
            max: sevens_left(game),
        },
        early_defender_points: if early_tricks <= game.tricks().len() {
            Span::exactly(0)
        } else {
            points
        },
        last_tricks: None,
        nodes: 0,
    })
}

/// Bounds of a Null in which the declarer can no longer take a trick,
/// however anyone plays; `None` if that is not certain from this simple
/// test (or the game is no Null in play). Sound, without search:
///
/// - no defender holds a blank (a blank lead lets the next card choose
///   the suit);
/// - the declarer is not to lead (after his first trick the Null is
///   over, so he leads at most the very first trick);
/// - in the trick on the table, no card he may play (or has played)
///   beats the card winning it so far;
/// - in every suit in which he holds a card that can win, every card of
///   the defenders is higher and can win itself. Then whatever a defender
///   leads beats everything the declarer can follow with, and whatever he
///   throws off never wins.
///
/// Then he loses every remaining trick: his sevens are all ducked, the
/// defenders take all Augen. Whether he still has to follow suit and the
/// Augen of the early tricks are only bounded.
#[must_use]
pub fn null_reach<C: PlayingCard>(game: &Game<C>, early_tricks: usize) -> Option<Reach<C>> {
    if game.phase() != Phase::Playing {
        return None;
    }
    let declaration = game.declaration()?;
    if !declaration.is_null() {
        return None;
    }
    let g = declaration.game();
    let own = game.hand(Seat::DECLARER);
    let defenders: Vec<C> = Seat::DEFENDERS
        .iter()
        .flat_map(|&s| game.hand(s).iter().copied())
        .collect();
    if defenders.iter().any(|c| c.is_blank()) {
        return None;
    }
    let trick = game.current_trick();
    let played = declarer_on_table(game);
    let lead = rules::effective_lead(trick);
    match (played, lead) {
        (Some((offset, _)), _) => {
            if rules::winning_index(g, trick)? == offset {
                return None;
            }
        }
        (None, None) => {
            // Nothing to follow yet: the declarer must not set the suit.
            if game.current_seat()?.is_declarer() {
                return None;
            }
        }
        (None, Some(lead)) => {
            let winner = trick[rules::winning_index(g, trick)?];
            let legal = rules::legal_moves(g, own, Some(lead));
            if legal.iter().any(|&c| rules::beats(g, lead, winner, c)) {
                return None;
            }
        }
    }
    let can_win = |c: &C| !c.is_blank() && !c.never_wins();
    for &mine in own.iter().filter(|c| can_win(c)) {
        let kind = card_kind(g, mine);
        let strength = card_strength(g, mine);
        let threatened = defenders
            .iter()
            .any(|&d| card_kind(g, d) == kind && (!can_win(&d) || card_strength(g, d) <= strength));
        if threatened {
            return None;
        }
    }
    let live = live_points(game);
    let left = tricks_left(game);
    let followed_now = played.is_some_and(|(offset, card)| {
        offset > 0
            && rules::effective_lead(&trick[..offset])
                .is_some_and(|lead| rules::follows_lead(g, lead, card))
    });
    let may_follow = own.iter().any(|&mine| {
        let kind = card_kind(g, mine);
        mine.face().is_some()
            && (defenders.iter().any(|&d| card_kind(g, d) == kind)
                || (played.is_none() && lead.is_some_and(|l| rules::follows_lead(g, l, mine))))
    });
    // Following the lead on the table is forced without a blank to dodge.
    let forced_now = played.is_none()
        && !own.iter().any(|c| c.is_blank())
        && lead.is_some_and(|l| own.iter().any(|&c| rules::follows_lead(g, l, c)));
    let base = u32::from(followed_now);
    Some(Reach {
        declarer_points: Span::exactly(0),
        defender_points: Span::exactly(live),
        declarer_tricks: Span::exactly(0),
        tricks: Span::exactly(left),
        followed: Span {
            min: base + u32::from(forced_now),
            max: if may_follow { base + left } else { base },
        },
        sevens_ducked: Span::exactly(sevens_left(game)),
        early_defender_points: early_points(game, early_tricks),
        last_tricks: None,
        nodes: 0,
    })
}

/// Bounds of the Augen in the tricks before `early_tricks` when the
/// defenders take every remaining trick: the cards on the table plus, per
/// seat, its cheapest and its dearest cards for the tricks still to come
/// before then.
fn early_points<C: PlayingCard>(game: &Game<C>, early_tricks: usize) -> Span {
    let done = game.tricks().len();
    if early_tricks <= done {
        return Span::exactly(0);
    }
    let tricks = early_tricks.min(TRICKS_PER_GAME) - done;
    let table: u32 = game.current_trick().iter().map(|c| c.card_points()).sum();
    let mut span = Span::exactly(table);
    for seat in Seat::ALL {
        let offset = (seat.index() + 3 - game.trick_lead().index()) % 3;
        let on_table = usize::from(offset < game.current_trick().len());
        let count = tricks - on_table;
        let mut points: Vec<u32> = game.hand(seat).iter().map(|c| c.card_points()).collect();
        points.sort_unstable();
        let low: u32 = points.iter().take(count).sum();
        let high: u32 = points.iter().rev().take(count).sum();
        span.min += low;
        span.max += high;
    }
    span
}

/// "Ended before the tenth trick" in the list of last tricks.
const NO_LAST: u16 = u16::MAX;

#[derive(Debug, Clone, PartialEq, Eq)]
enum Last {
    Untracked,
    /// Codes `card index << 2 | seat`, sorted.
    Set(Vec<u16>),
    Many,
}

impl Last {
    fn join(&self, other: &Last) -> Last {
        match (self, other) {
            (Last::Untracked, _) | (_, Last::Untracked) => Last::Untracked,
            (Last::Many, _) | (_, Last::Many) => Last::Many,
            (Last::Set(a), Last::Set(b)) => {
                let mut all = a.clone();
                for code in b {
                    if let Err(at) = all.binary_search(code) {
                        all.insert(at, *code);
                    }
                }
                if all.len() > MAX_LAST {
                    Last::Many
                } else {
                    Last::Set(all)
                }
            }
        }
    }
}

/// Bounds from a position on (see [`Reach`]).
#[derive(Debug, Clone, PartialEq, Eq)]
struct Sum {
    declarer_points: Span,
    defender_points: Span,
    declarer_tricks: Span,
    tricks: Span,
    followed: Span,
    sevens: Span,
    early: Span,
    last: Last,
}

impl Sum {
    fn join(&self, other: &Sum) -> Sum {
        Sum {
            declarer_points: self.declarer_points.join(other.declarer_points),
            defender_points: self.defender_points.join(other.defender_points),
            declarer_tricks: self.declarer_tricks.join(other.declarer_tricks),
            tricks: self.tricks.join(other.tricks),
            followed: self.followed.join(other.followed),
            sevens: self.sevens.join(other.sevens),
            early: self.early.join(other.early),
            last: self.last.join(&other.last),
        }
    }
}

/// What one trick adds.
#[derive(Debug, Clone, Copy)]
struct Gain {
    declarer_points: u32,
    defender_points: u32,
    declarer_trick: u32,
    followed: u32,
    seven: u32,
    early: u32,
}

impl Gain {
    fn leaf(self, last: Last) -> Sum {
        Sum {
            declarer_points: Span::exactly(self.declarer_points),
            defender_points: Span::exactly(self.defender_points),
            declarer_tricks: Span::exactly(self.declarer_trick),
            tricks: Span::exactly(1),
            followed: Span::exactly(self.followed),
            sevens: Span::exactly(self.seven),
            early: Span::exactly(self.early),
            last,
        }
    }

    fn add_to(self, rest: &Sum) -> Sum {
        Sum {
            declarer_points: rest.declarer_points.shift(self.declarer_points),
            defender_points: rest.defender_points.shift(self.defender_points),
            declarer_tricks: rest.declarer_tricks.shift(self.declarer_trick),
            tricks: rest.tricks.shift(1),
            followed: rest.followed.shift(self.followed),
            sevens: rest.sevens.shift(self.seven),
            early: rest.early.shift(self.early),
            last: rest.last.clone(),
        }
    }
}

struct Search<C> {
    game: GameType,
    null: bool,
    cards: Vec<C>,
    kind: Vec<u8>,
    strength: Vec<u8>,
    points: Vec<u32>,
    can_win: u32,
    seven: u32,
    /// Cards per kind (suits 0..4, trump, blank).
    kind_mask: [u32; 6],
    hands: [u32; 3],
    options: ReachOptions,
    memo: HashMap<u64, Sum>,
    nodes: u64,
}

impl<C: PlayingCard> Search<C> {
    fn new(game: &Game<C>, game_type: GameType, null: bool, options: ReachOptions) -> Option<Self> {
        let mut cards: Vec<C> = Seat::ALL
            .iter()
            .flat_map(|&s| game.hand(s).iter().copied())
            .chain(game.current_trick().iter().copied())
            .collect();
        if cards.len() > 32 {
            return None;
        }
        cards.sort();
        let mut s = Search {
            game: game_type,
            null,
            kind: cards.iter().map(|&c| card_kind(game_type, c)).collect(),
            strength: cards.iter().map(|&c| card_strength(game_type, c)).collect(),
            points: cards.iter().map(|c| c.card_points()).collect(),
            can_win: 0,
            seven: 0,
            kind_mask: [0; 6],
            hands: [0; 3],
            cards,
            options,
            memo: HashMap::new(),
            nodes: 0,
        };
        for (i, &c) in s.cards.iter().enumerate() {
            if !c.is_blank() && !c.never_wins() {
                s.can_win |= 1 << i;
            }
            if is_seven(c) {
                s.seven |= 1 << i;
            }
            s.kind_mask[usize::from(s.kind[i]).min(5)] |= 1 << i;
        }
        for seat in Seat::ALL {
            for &c in game.hand(seat) {
                s.hands[seat.index()] |= 1 << s.index_of(c)?;
            }
        }
        if s.hands.iter().any(|h| h.count_ones() > 10) {
            return None;
        }
        Some(s)
    }

    fn index_of(&self, card: C) -> Option<u8> {
        let i = self.cards.binary_search(&card).ok()?;
        u8::try_from(i).ok()
    }

    fn decode_last(&self, code: u16) -> LastTrick<C> {
        if code == NO_LAST {
            return None;
        }
        let card = *self.cards.get(usize::from(code >> 2))?;
        Some((card, Seat::from_index(usize::from(code & 3))?))
    }

    /// Whether cards `a` and `b` of one hand lead to the same results.
    fn equivalent(&self, others: u32, a: usize, b: usize) -> bool {
        let same = |bits: u32| (bits >> a) & 1 == (bits >> b) & 1;
        if self.kind[a] != self.kind[b]
            || self.points[a] != self.points[b]
            || !same(self.seven)
            || !same(self.can_win)
        {
            return false;
        }
        if self.kind[a] == BLANK || self.can_win & (1 << a) == 0 {
            return true;
        }
        let (lo, hi) = (
            self.strength[a].min(self.strength[b]),
            self.strength[a].max(self.strength[b]),
        );
        let mut rest = others & self.can_win;
        while rest != 0 {
            let x = rest.trailing_zeros() as usize;
            rest &= rest - 1;
            if self.kind[x] == self.kind[a] && (lo..=hi).contains(&self.strength[x]) {
                return false;
            }
        }
        true
    }

    /// Legal cards of `seat` after the trick so far, one per class of
    /// equivalent cards. Mirrors [`rules::legal_moves`]: the kind of the
    /// first card that is no blank must be followed if the hand can;
    /// blanks may always be played.
    fn moves(&self, hands: [u32; 3], seat: usize, trick: &[u8]) -> ([u8; 10], usize) {
        let hand = hands[seat];
        let lead = trick
            .iter()
            .map(|&i| self.kind[usize::from(i)])
            .find(|&k| k != BLANK);
        let mut legal = match lead {
            None => hand,
            Some(kind) => {
                let follow = hand & self.kind_mask[usize::from(kind)];
                if follow == 0 {
                    hand
                } else {
                    follow | (hand & self.kind_mask[usize::from(BLANK)])
                }
            }
        };
        let mut on_table = 0u32;
        for &i in trick {
            on_table |= 1 << i;
        }
        let others = (hands[(seat + 1) % 3] | hands[(seat + 2) % 3] | on_table) & !hand;
        let merge = !self.options.last_trick;
        let mut moves = [0u8; 10];
        let mut n = 0;
        while legal != 0 && n < moves.len() {
            let i = legal.trailing_zeros() as usize;
            legal &= legal - 1;
            if merge
                && moves[..n]
                    .iter()
                    .any(|&m| self.equivalent(others, usize::from(m), i))
            {
                continue;
            }
            moves[n] = u8::try_from(i).unwrap_or(0);
            n += 1;
        }
        (moves, n)
    }

    /// Bounds from a trick boundary on.
    fn boundary(&mut self, hands: [u32; 3], leader: usize, done: usize) -> Option<Sum> {
        let key = u64::from(hands[0] | hands[1] | hands[2]) | ((leader as u64) << 32);
        if let Some(sum) = self.memo.get(&key) {
            return Some(sum.clone());
        }
        self.nodes += 1;
        if self.nodes > self.options.node_limit {
            return None;
        }
        let sum = self.trick(hands, leader, [0; 3], 0, done)?;
        self.memo.insert(key, sum.clone());
        Some(sum)
    }

    /// Bounds from a trick with `len` cards on the table on. `done` is the
    /// number of completed tricks before it.
    fn trick(
        &mut self,
        hands: [u32; 3],
        leader: usize,
        mut cards: [u8; 3],
        len: usize,
        done: usize,
    ) -> Option<Sum> {
        if len == 3 {
            return self.complete(hands, leader, cards, done);
        }
        let seat = (leader + len) % 3;
        let mut acc: Option<Sum> = None;
        let (moves, n) = self.moves(hands, seat, &cards[..len]);
        for &m in &moves[..n] {
            let mut next = hands;
            next[seat] &= !(1 << m);
            cards[len] = m;
            let sum = self.trick(next, leader, cards, len + 1, done)?;
            acc = Some(match acc {
                None => sum,
                Some(a) => a.join(&sum),
            });
        }
        acc
    }

    /// A complete trick: what it adds, then the rest of the game.
    fn complete(
        &mut self,
        hands: [u32; 3],
        leader: usize,
        trick: [u8; 3],
        done: usize,
    ) -> Option<Sum> {
        let cards = trick.map(|i| self.cards[usize::from(i)]);
        let winner = rules::trick_winner(self.game, leader, cards);
        let points: u32 = trick.iter().map(|&i| self.points[usize::from(i)]).sum();
        let declarer_wins = winner == 0;
        let offset = (3 - leader) % 3;
        let own = trick[offset];
        let followed = offset > 0
            && rules::effective_lead(&cards[..offset])
                .is_some_and(|lead| rules::follows_lead(self.game, lead, cards[offset]));
        let gain = Gain {
            declarer_points: if declarer_wins { points } else { 0 },
            defender_points: if declarer_wins { 0 } else { points },
            declarer_trick: u32::from(declarer_wins),
            followed: u32::from(followed),
            seven: u32::from(!declarer_wins && self.seven & (1 << own) != 0),
            early: if !declarer_wins && done < self.options.early_tricks {
                points
            } else {
                0
            },
        };
        let ended = (self.null && declarer_wins) || hands[winner] == 0;
        if ended {
            let last = if !self.options.last_trick {
                Last::Untracked
            } else if done + 1 == TRICKS_PER_GAME {
                let offset = (winner + 3 - leader) % 3;
                let seat = seat_at(Seat::from_index(leader)?, offset);
                let code = (u16::from(trick[offset]) << 2) | u16::try_from(seat.index()).ok()?;
                Last::Set(vec![code])
            } else {
                Last::Set(vec![NO_LAST])
            };
            return Some(gain.leaf(last));
        }
        let rest = self.boundary(hands, winner, done + 1)?;
        Some(gain.add_to(&rest))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cards::{CardKind, Suit, parse_cards, standard_deck};
    use crate::deck::DealtCard;
    use crate::rng::Rng;
    use crate::rules::{Declaration, NullRisk, NullStyle, RiskLevel};

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct Line<C> {
        points: [u32; 3],
        tricks: [u32; 3],
        count: usize,
        style: NullStyle,
        last: LastTrick<C>,
    }

    fn line_of<C: PlayingCard>(game: &Game<C>, early: usize) -> Line<C> {
        Line {
            points: game.trick_card_points(),
            tricks: game.seat_tricks(),
            count: game.tricks().len(),
            style: game.null_style(early),
            last: (game.tricks().len() == TRICKS_PER_GAME)
                .then(|| game.tricks().last().map(|t| (t.winning_card(), t.winner)))
                .flatten(),
        }
    }

    /// Every line of play by the rules of [`Game`], no memo, no merging.
    fn brute<C: PlayingCard>(game: &Game<C>, early: usize, out: &mut Vec<Line<C>>) {
        let Some(seat) = game.current_seat() else {
            out.push(line_of(game, early));
            return;
        };
        for card in game.legal_moves(seat) {
            let mut next = game.clone();
            next.play(seat, card).expect("legal");
            brute(&next, early, out);
        }
    }

    fn span(values: impl Iterator<Item = u32>) -> Span {
        let v: Vec<u32> = values.collect();
        Span {
            min: v.iter().copied().min().unwrap_or(0),
            max: v.iter().copied().max().unwrap_or(0),
        }
    }

    /// The standard deck as dealt cards, some of them modified: blanks,
    /// "Gezinkt", "Unsichtbar", "Bube ehrenhalber", more Augen.
    fn modified_deck(rng: &mut Rng) -> Vec<DealtCard> {
        standard_deck()
            .into_iter()
            .zip(0u32..)
            .map(|(card, id)| {
                let mut c = DealtCard::plain(id, CardKind::Normal(card));
                match rng.gen_range(0..16) {
                    0 => c = DealtCard::plain(id, CardKind::Blank),
                    1 => c.rank_bonus = 1,
                    2 => c.never_wins = true,
                    3 => c.honorary_jack = true,
                    4 => c.card_points += 5,
                    _ => {}
                }
                c
            })
            .collect()
    }

    /// A game after `cards` random legal cards. With `low_null` the
    /// declarer gets mostly low cards, so that some Nulls are safe.
    fn random_position<C: PlayingCard>(
        rng: &mut Rng,
        deck: Vec<C>,
        declaration: Declaration,
        cards: usize,
        low_null: bool,
    ) -> Game<C> {
        let forehand = Seat::from_index(rng.gen_range(0..3) as usize).unwrap_or(Seat::DECLARER);
        let mut deck = deck;
        rng.shuffle(&mut deck);
        if low_null {
            let rank = |c: &C| {
                c.face()
                    .map_or(9, |f| rules::suit_strength(GameType::Null, f))
            };
            let noise: Vec<u64> = (0..deck.len()).map(|_| rng.gen_range(0..4)).collect();
            let mut keyed: Vec<(u64, C)> = deck
                .iter()
                .zip(noise)
                .map(|(c, n)| (u64::from(rank(c)) + n, *c))
                .collect();
            keyed.sort_by_key(|(k, _)| *k);
            let low: Vec<C> = keyed.iter().take(10).map(|(_, c)| *c).collect();
            let rest: Vec<C> = deck.iter().copied().filter(|c| !low.contains(c)).collect();
            deck = low.into_iter().chain(rest).collect();
        }
        let hands = [
            deck[0..10].to_vec(),
            deck[10..20].to_vec(),
            deck[20..30].to_vec(),
        ];
        let mut game = Game::with_cards(hands, [deck[30], deck[31]], forehand).expect("deal");
        game.pick_up_skat().expect("pick up");
        let hand = game.hand(Seat::DECLARER).to_vec();
        let mut discard: Vec<C> = hand.clone();
        discard.sort_by_key(|c| std::cmp::Reverse(c.card_points()));
        game.discard([discard[0], discard[1]]).expect("discard");
        game.declare(declaration).expect("declare");
        for _ in 0..cards {
            let Some(seat) = game.current_seat() else {
                break;
            };
            let legal = game.legal_moves(seat);
            let pick = legal[rng.gen_range(0..legal.len() as u64) as usize];
            game.play(seat, pick).expect("legal");
        }
        game
    }

    fn within(outer: Span, inner: Span) -> bool {
        outer.min <= inner.min && inner.max <= outer.max
    }

    fn assert_within<C: PlayingCard>(outer: &Reach<C>, exact: &Reach<C>, what: &str) {
        assert!(
            within(outer.declarer_points, exact.declarer_points),
            "{what}"
        );
        assert!(
            within(outer.defender_points, exact.defender_points),
            "{what}"
        );
        assert!(
            within(outer.declarer_tricks, exact.declarer_tricks),
            "{what}"
        );
        assert!(within(outer.tricks, exact.tricks), "{what}");
        assert!(within(outer.followed, exact.followed), "{what}");
        assert!(within(outer.sevens_ducked, exact.sevens_ducked), "{what}");
        assert!(
            within(outer.early_defender_points, exact.early_defender_points),
            "{what}"
        );
    }

    /// Compare all three sources with brute force. Returns whether
    /// [`null_reach`] found the Null safe.
    fn check<C: PlayingCard>(game: &Game<C>, early: usize) -> bool {
        if game.phase() != Phase::Playing {
            return false;
        }
        let mut lines = Vec::new();
        brute(game, early, &mut lines);
        let now = line_of(game, early);
        let gained = |f: &dyn Fn(&Line<C>) -> u32| span(lines.iter().map(|l| f(l) - f(&now)));
        let mut exact = None;
        for last_trick in [false, true] {
            let options = ReachOptions {
                last_trick,
                early_tricks: early,
                node_limit: 1_000_000,
            };
            let r = reach(game, options).expect("within the limit");
            assert_eq!(r.declarer_points, gained(&|l| l.points[0]));
            assert_eq!(r.defender_points, gained(&|l| l.points[1] + l.points[2]));
            assert_eq!(r.declarer_tricks, gained(&|l| l.tricks[0]));
            assert_eq!(r.tricks, gained(&|l| l.count as u32));
            assert_eq!(r.sevens_ducked, gained(&|l| l.style.sevens_ducked));
            if game.declaration().is_some_and(|d| d.is_null()) && now.style.never_followed {
                let never = || lines.iter().map(|l| u32::from(!l.style.never_followed));
                assert_eq!(span(never()).max, u32::from(r.followed.max > 0));
                assert_eq!(span(never()).min, u32::from(r.followed.min > 0));
            }
            let all_reach_early = lines
                .iter()
                .all(|l| l.style.defender_points_after.is_some());
            if early > game.tricks().len() && all_reach_early {
                let before = now.points[1] + now.points[2];
                let after = lines
                    .iter()
                    .filter_map(|l| l.style.defender_points_after)
                    .map(|p| p - before);
                assert_eq!(r.early_defender_points, span(after));
            }
            if last_trick {
                let mut want: Vec<LastTrick<C>> = lines.iter().map(|l| l.last).collect();
                want.sort();
                want.dedup();
                if want.len() <= MAX_LAST {
                    assert_eq!(r.last_tricks, Some(want));
                } else {
                    assert_eq!(r.last_tricks, None);
                }
            }
            exact = Some(r);
        }
        let exact = exact.expect("searched");
        let loose = loose_reach(game, early).expect("in play");
        assert_within(&loose, &exact, "loose");
        match null_reach(game, early) {
            Some(safe) => {
                assert_within(&safe, &exact, "null");
                assert_eq!(exact.declarer_tricks, Span::exactly(0));
                true
            }
            None => false,
        }
    }

    #[test]
    fn bounds_equal_brute_force_in_late_positions() {
        let declarations = [
            Declaration::suit(Suit::Hearts, RiskLevel::Normal),
            Declaration::grand(RiskLevel::Normal),
            Declaration::null(NullRisk::Normal),
        ];
        let mut rng = Rng::from_seed(5);
        let mut safe = 0;
        for round in 0..120 {
            for &declaration in &declarations {
                // 3 to 4 tricks left, sometimes in the middle of a trick.
                let played = 18 + rng.gen_range(0..4) as usize;
                let low = declaration.is_null() && round % 2 == 0;
                if round % 3 == 0 {
                    let deck = modified_deck(&mut rng);
                    let game = random_position(&mut rng, deck, declaration, played, low);
                    safe += usize::from(check(&game, 8));
                } else {
                    let game = random_position(&mut rng, standard_deck(), declaration, played, low);
                    safe += usize::from(check(&game, 9));
                }
            }
        }
        assert!(safe > 10, "only {safe} safe Nulls");
    }

    #[test]
    fn a_null_with_only_low_cards_left_cannot_be_lost() {
        // The declarer holds the sevens and eights; everything else is
        // higher. He does not lead.
        let c = |s: &str| parse_cards(s).expect("cards");
        let hands = [
            c("C7 S7 H7 D7 C8 S8 H8 D8 C9 S9"),
            c("CA CK CQ CJ CT SA SK SQ SJ ST"),
            c("HA HK HQ HJ HT DA DK DQ DJ DT"),
        ];
        let skat = [c("H9")[0], c("D9")[0]];
        let mut game =
            Game::with_cards(hands, skat, Seat::from_index(1).expect("seat")).expect("deal");
        game.declare(Declaration::null(NullRisk::Hand))
            .expect("declare");
        let r = null_reach(&game, 0).expect("safe");
        assert_eq!(r.declarer_tricks, Span::exactly(0));
        assert_eq!(r.tricks, Span::exactly(10));
        assert_eq!(r.sevens_ducked, Span::exactly(4));
        // Leading himself he could still be the one on top.
        let mut lead = Game::with_cards(
            [
                c("C7 S7 H7 D7 C8 S8 H8 D8 C9 S9"),
                c("CA CK CQ CJ CT SA SK SQ SJ ST"),
                c("HA HK HQ HJ HT DA DK DQ DJ DT"),
            ],
            skat,
            Seat::DECLARER,
        )
        .expect("deal");
        lead.declare(Declaration::null(NullRisk::Hand))
            .expect("declare");
        assert_eq!(null_reach(&lead, 0), None);
        // A defender's low card makes it open again.
        let mut open = Game::with_cards(
            [
                c("C7 S7 H7 D7 C9 S8 H8 D8 H9 S9"),
                c("CA CK CQ CJ CT SA SK SQ SJ ST"),
                c("HA HK HQ HJ HT DA DK DQ DJ C8"),
            ],
            [c("D9")[0], c("DT")[0]],
            Seat::from_index(1).expect("seat"),
        )
        .expect("deal");
        open.declare(Declaration::null(NullRisk::Hand))
            .expect("declare");
        assert_eq!(null_reach(&open, 0), None);
    }
}

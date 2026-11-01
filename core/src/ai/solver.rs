//! Double-dummy solver: the exact result of the remaining tricks when every
//! hand is known (DESIGN 10.2, stage 2).
//!
//! The PIMC policies ([`super::pimc`]) deal the unknown cards and ask this
//! solver how every legal move ends with perfect information. It is an
//! alpha-beta search over single cards with
//!
//! - card indices in a `u32` bit mask per hand (at most 32 cards are live:
//!   three hands plus the cards on the table),
//! - a transposition table at trick boundaries (the value of the remaining
//!   tricks depends only on who holds what and who leads),
//! - equivalent cards searched once (two cards of one hand of the same
//!   kind and Augen with no live card of another seat between them),
//! - move ordering (sure winners and cheap ducks first, the stored best
//!   lead of the transposition table before everything else),
//! - an optional depth limit in tricks with a static estimate behind it.
//!
//! The trick rules mirror [`crate::rules`] exactly, blanks, the rank bonus
//! ("Gezinkt") and cards that never win ("Unsichtbar") included; a test
//! compares both on random tricks.

use crate::cards::PlayingCard;
use crate::game::Seat;
use crate::rules::{self, GameType};

/// Kind of a non-trump card: its suit (0..4). Trumps and blanks follow.
const TRUMP: u8 = 4;
/// Kind of a blank: follows nothing, never wins.
const BLANK: u8 = 5;
/// Most live cards a position may hold.
const MAX_CARDS: usize = 32;
/// "No move" marker in the transposition table.
const NO_MOVE: u8 = u8::MAX;
/// Infinity for the search window.
const INF: i32 = i32::MAX / 4;

/// What the search optimizes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Goal {
    /// Augen the declarer takes from now on (the cards on the table
    /// included). The declarer maximizes, the defenders minimize.
    DeclarerPoints,
    /// Null: 1 if the declarer takes a trick from now on, else 0. The
    /// defenders maximize, the declarer minimizes.
    NullDeclarerTrick,
    /// Ramsch seen from one seat (paranoid): the Augen the two other seats
    /// take from now on, the skat included for the last trick. The seat
    /// maximizes, the two others minimize.
    RamschAvoid(Seat),
}

/// A position with every hand known.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Layout<C> {
    /// Current hands per seat.
    pub hands: [Vec<C>; 3],
    /// Seat that led the current trick (or leads the next one).
    pub trick_lead: Seat,
    /// Cards of the current trick in play order (0 to 2).
    pub trick: Vec<C>,
    /// Augen in the skat; in a Ramsch they go to the winner of the last
    /// trick.
    pub skat_points: u32,
}

/// Search limits.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Limits {
    /// Tricks searched exactly, the current one included; behind them a
    /// static estimate. `0` searches to the end.
    pub depth: u32,
    /// Nodes after which every further trick boundary is estimated
    /// instead of searched, as if the depth were used up. `0` = no limit.
    /// Keeps the worst case of a search bounded, deterministically.
    pub nodes: u64,
}

/// One slot of the transposition table.
#[derive(Debug, Clone, Copy, Default)]
struct Entry {
    hands: [u32; 3],
    tag: u32,
    flag: u8,
    best: u8,
    generation: u32,
    value: i32,
}

const EXACT: u8 = 0;
const LOWER: u8 = 1;
const UPPER: u8 = 2;

/// Transposition table, reused across searches. A new generation
/// invalidates all entries without clearing memory.
#[derive(Debug, Clone)]
pub struct Table {
    entries: Vec<Entry>,
    generation: u32,
}

impl Table {
    /// A table with `2^bits` slots.
    #[must_use]
    pub fn new(bits: u32) -> Table {
        Table {
            entries: vec![Entry::default(); 1 << bits.clamp(4, 24)],
            generation: 0,
        }
    }

    fn next_generation(&mut self) {
        self.generation = self.generation.wrapping_add(1);
        if self.generation == 0 {
            self.entries.fill(Entry::default());
            self.generation = 1;
        }
    }

    fn slot(&self, hands: [u32; 3], tag: u32) -> usize {
        let a = u64::from(hands[0]) | (u64::from(hands[1]) << 32);
        let b = u64::from(hands[2]) | (u64::from(tag) << 32);
        let h = mix(a ^ mix(b));
        (h as usize) & (self.entries.len() - 1)
    }
}

impl Default for Table {
    fn default() -> Table {
        Table::new(16)
    }
}

/// 64-bit finalizer (splitmix64).
fn mix(mut z: u64) -> u64 {
    z = z.wrapping_add(0x9E37_79B9_7F4A_7C15);
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

/// Search state over card indices.
struct Search<'t> {
    kind: [u8; MAX_CARDS],
    strength: [u8; MAX_CARDS],
    points: [i32; MAX_CARDS],
    /// First and last index with the same kind and strength.
    tie_lo: [u8; MAX_CARDS],
    tie_hi: [u8; MAX_CARDS],
    kind_mask: [u32; 6],
    can_win: u32,
    /// Seats whose tricks count for the value.
    gain: u8,
    /// Seats that maximize.
    maxi: u8,
    null: bool,
    ramsch: bool,
    skat_points: i32,
    hands: [u32; 3],
    trick: [u8; 3],
    trick_len: usize,
    leader: usize,
    /// Augen still in the hands and on the table.
    live_points: i32,
    /// Tricks left to search exactly (`u32::MAX` = no limit).
    depth_left: u32,
    table: &'t mut Table,
    nodes: u64,
    /// See [`Limits::nodes`].
    node_limit: u64,
    /// Cutoffs per seat and card (history heuristic for move ordering).
    history: [[u32; MAX_CARDS]; 3],
}

/// Values of the legal moves of the seat to play, one per move in the
/// order of [`rules::legal_moves`]. See [`Goal`] for what a value means.
/// Returns an empty list if nobody is to play or a hand is inconsistent.
#[must_use]
pub fn solve<C: PlayingCard>(game: GameType, goal: Goal, layout: &Layout<C>) -> Vec<(C, i32)> {
    let mut table = Table::default();
    solve_with(game, goal, layout, Limits::default(), &mut table).0
}

/// Like [`solve`] with explicit limits and a reusable table. Also returns
/// the number of nodes searched.
#[must_use]
pub fn solve_with<C: PlayingCard>(
    game: GameType,
    goal: Goal,
    layout: &Layout<C>,
    limits: Limits,
    table: &mut Table,
) -> (Vec<(C, i32)>, u64) {
    let Some((mut search, cards)) = Search::new(game, goal, layout, limits, table) else {
        return (Vec::new(), 0);
    };
    let seat = search.to_play();
    let lead = rules::effective_lead(&layout.trick);
    let legal = rules::legal_moves(game, &layout.hands[seat], lead);
    let index_of = |card: C| cards.iter().position(|&c| c == card);
    // Equivalent moves share one search.
    let mut reps: Vec<(u8, i32)> = Vec::new();
    let mut out = Vec::with_capacity(legal.len());
    for card in legal {
        let Some(i) = index_of(card) else {
            return (Vec::new(), search.nodes);
        };
        let i = u8::try_from(i).unwrap_or(0);
        let known = reps
            .iter()
            .find(|&&(r, _)| search.equivalent(seat, r.min(i), r.max(i)))
            .map(|&(_, v)| v);
        let value = match known {
            Some(v) => v,
            None => {
                let v = search.play(seat, i, -INF, INF);
                reps.push((i, v));
                v
            }
        };
        out.push((card, value));
    }
    (out, search.nodes)
}

/// For every legal move of the seat to play, whether the value reaches
/// `threshold` (a null-window search, much cheaper than the exact value).
/// Returns an empty list if nobody is to play or a hand is inconsistent.
#[must_use]
pub fn probe_with<C: PlayingCard>(
    game: GameType,
    goal: Goal,
    layout: &Layout<C>,
    threshold: i32,
    limits: Limits,
    table: &mut Table,
) -> (Vec<(C, bool)>, u64) {
    let Some((mut search, cards)) = Search::new(game, goal, layout, limits, table) else {
        return (Vec::new(), 0);
    };
    let seat = search.to_play();
    let lead = rules::effective_lead(&layout.trick);
    let legal = rules::legal_moves(game, &layout.hands[seat], lead);
    let mut reps: Vec<(u8, bool)> = Vec::new();
    let mut out = Vec::with_capacity(legal.len());
    for card in legal {
        let Some(i) = cards.iter().position(|&c| c == card) else {
            return (Vec::new(), search.nodes);
        };
        let i = u8::try_from(i).unwrap_or(0);
        let known = reps
            .iter()
            .find(|&&(r, _)| search.equivalent(seat, r.min(i), r.max(i)))
            .map(|&(_, v)| v);
        let reached = match known {
            Some(v) => v,
            None => {
                let v = search.play(seat, i, threshold - 1, threshold) >= threshold;
                reps.push((i, v));
                v
            }
        };
        out.push((card, reached));
    }
    (out, search.nodes)
}

impl<'t> Search<'t> {
    fn new<C: PlayingCard>(
        game: GameType,
        goal: Goal,
        layout: &Layout<C>,
        limits: Limits,
        table: &'t mut Table,
    ) -> Option<(Search<'t>, Vec<C>)> {
        if layout.trick.len() > 2 {
            return None;
        }
        let mut cards: Vec<C> = layout
            .hands
            .iter()
            .flatten()
            .copied()
            .chain(layout.trick.iter().copied())
            .collect();
        if cards.len() > MAX_CARDS {
            return None;
        }
        let key = |c: &C| {
            let kind = card_kind(game, *c);
            let strength = card_strength(game, *c);
            (
                kind,
                std::cmp::Reverse(strength),
                std::cmp::Reverse(c.card_points()),
                *c,
            )
        };
        cards.sort_by_key(key);
        let n = cards.len();
        cards.dedup();
        if cards.len() != n {
            return None;
        }
        table.next_generation();
        let mut s = Search {
            kind: [BLANK; MAX_CARDS],
            strength: [0; MAX_CARDS],
            points: [0; MAX_CARDS],
            tie_lo: [0; MAX_CARDS],
            tie_hi: [0; MAX_CARDS],
            kind_mask: [0; 6],
            can_win: 0,
            gain: 0,
            maxi: 0,
            null: false,
            ramsch: false,
            skat_points: i32::try_from(layout.skat_points).unwrap_or(0),
            hands: [0; 3],
            trick: [0; 3],
            trick_len: layout.trick.len(),
            leader: layout.trick_lead.index(),
            live_points: 0,
            depth_left: if limits.depth == 0 {
                u32::MAX
            } else {
                limits.depth
            },
            table,
            nodes: 0,
            node_limit: limits.nodes,
            history: [[0; MAX_CARDS]; 3],
        };
        for (i, &c) in cards.iter().enumerate() {
            s.kind[i] = card_kind(game, c);
            s.strength[i] = card_strength(game, c);
            s.points[i] = i32::try_from(c.card_points()).unwrap_or(0);
            s.kind_mask[usize::from(s.kind[i])] |= 1 << i;
            if !c.is_blank() && !c.never_wins() {
                s.can_win |= 1 << i;
            }
            s.live_points += s.points[i];
        }
        for i in 0..n {
            let same = |j: usize| s.kind[j] == s.kind[i] && s.strength[j] == s.strength[i];
            let lo = (0..=i).rev().take_while(|&j| same(j)).last().unwrap_or(i);
            let hi = (i..n).take_while(|&j| same(j)).last().unwrap_or(i);
            s.tie_lo[i] = u8::try_from(lo).unwrap_or(0);
            s.tie_hi[i] = u8::try_from(hi).unwrap_or(0);
        }
        let index = |c: &C| cards.binary_search_by_key(&key(c), key).ok();
        for (seat, hand) in layout.hands.iter().enumerate() {
            for c in hand {
                s.hands[seat] |= 1 << index(c)?;
            }
        }
        for (k, c) in layout.trick.iter().enumerate() {
            s.trick[k] = u8::try_from(index(c)?).ok()?;
        }
        match goal {
            Goal::DeclarerPoints => {
                s.gain = 1;
                s.maxi = 1;
            }
            Goal::NullDeclarerTrick => {
                s.null = true;
                s.maxi = 0b110;
            }
            Goal::RamschAvoid(seat) => {
                s.ramsch = true;
                s.maxi = 1 << seat.index();
                s.gain = 0b111 & !s.maxi;
            }
        }
        // Seats that played to the trick in progress hold one card less.
        let to_play = s.to_play();
        let full = s.hands[to_play].count_ones();
        for seat in 0..3 {
            let offset = (seat + 3 - s.leader) % 3;
            let expected = if offset < s.trick_len {
                full.checked_sub(1)?
            } else {
                full
            };
            if s.hands[seat].count_ones() != expected {
                return None;
            }
        }
        Some((s, cards))
    }

    /// Whether the search must estimate at the next trick boundary.
    fn out_of_budget(&self) -> bool {
        self.depth_left == 0 || (self.node_limit > 0 && self.nodes > self.node_limit)
    }

    /// Seat to play.
    fn to_play(&self) -> usize {
        (self.leader + self.trick_len) % 3
    }

    fn live(&self) -> u32 {
        let mut live = self.hands[0] | self.hands[1] | self.hands[2];
        for &c in &self.trick[..self.trick_len] {
            live |= 1 << c;
        }
        live
    }

    /// Whether `challenger` takes the trick from `holder` (mirrors
    /// [`rules::beats`]).
    fn beats(&self, lead_kind: u8, holder: u8, challenger: u8) -> bool {
        if self.can_win & (1 << challenger) == 0 {
            return false;
        }
        if self.can_win & (1 << holder) == 0 {
            return true;
        }
        let (h, c) = (usize::from(holder), usize::from(challenger));
        match (self.kind[h] == TRUMP, self.kind[c] == TRUMP) {
            (true, true) => self.strength[c] > self.strength[h],
            (false, true) => true,
            (true, false) => false,
            (false, false) => {
                if self.kind[c] != lead_kind {
                    false
                } else if self.kind[h] != lead_kind {
                    true
                } else {
                    self.strength[c] > self.strength[h]
                }
            }
        }
    }

    /// Kind of the effective lead of the current trick, `BLANK` if none.
    fn lead_kind(&self) -> u8 {
        self.trick[..self.trick_len]
            .iter()
            .map(|&c| self.kind[usize::from(c)])
            .find(|&k| k != BLANK)
            .unwrap_or(BLANK)
    }

    /// Offset of the card currently winning the trick.
    fn winning_offset(&self) -> usize {
        let lead_kind = self.lead_kind();
        let mut best = 0;
        for i in 1..self.trick_len {
            if self.beats(lead_kind, self.trick[best], self.trick[i]) {
                best = i;
            }
        }
        best
    }

    fn legal(&self, seat: usize) -> u32 {
        let hand = self.hands[seat];
        let lead_kind = self.lead_kind();
        if lead_kind == BLANK {
            return hand;
        }
        let follow = hand & self.kind_mask[usize::from(lead_kind)];
        if follow == 0 {
            hand
        } else {
            follow | (hand & self.kind_mask[usize::from(BLANK)])
        }
    }

    /// Whether cards `a < b` of `seat` are interchangeable.
    fn equivalent(&self, seat: usize, a: u8, b: u8) -> bool {
        if a == b {
            return true;
        }
        let (ia, ib) = (usize::from(a), usize::from(b));
        if self.kind[ia] != self.kind[ib] || self.points[ia] != self.points[ib] {
            return false;
        }
        if self.kind[ia] == BLANK {
            return true;
        }
        let wins_a = self.can_win & (1 << a) != 0;
        let wins_b = self.can_win & (1 << b) != 0;
        if wins_a != wins_b {
            return false;
        }
        if !wins_a {
            return true;
        }
        let lo = u32::from(self.tie_lo[ia]);
        let hi = u32::from(self.tie_hi[ib]);
        let range = range_mask(lo, hi);
        let separators = self.live() & !self.hands[seat] & self.can_win;
        range & separators & self.kind_mask[usize::from(self.kind[ia])] == 0
    }

    /// Whether no live card of another seat beats `card` within its kind.
    fn is_top(&self, seat: usize, card: u8) -> bool {
        let i = usize::from(card);
        if self.can_win & (1 << card) == 0 {
            return false;
        }
        let stronger = range_mask(0, u32::from(self.tie_hi[i])) & !(1 << card);
        let others = (self.hands[(seat + 1) % 3] | self.hands[(seat + 2) % 3]) & self.can_win;
        stronger & others & self.kind_mask[usize::from(self.kind[i])] == 0
    }

    /// Legal moves without equivalent duplicates, best guess first.
    fn ordered_moves(&self, seat: usize, tt_move: u8) -> ([u8; 10], usize) {
        let mut moves = [0u8; 10];
        let mut scores = [0i32; 10];
        let mut n = 0;
        let mut legal = self.legal(seat);
        let mut last_kept: Option<u8> = None;
        let maximize = self.maxi & (1 << seat) != 0;
        let (holder, lead_kind, partner_wins) = if self.trick_len > 0 {
            let off = self.winning_offset();
            let winner = (self.leader + off) % 3;
            let same_side = (self.maxi & (1 << winner) != 0) == maximize;
            (Some(self.trick[off]), self.lead_kind(), same_side)
        } else {
            (None, BLANK, false)
        };
        let last = self.trick_len == 2;
        while legal != 0 && n < 10 {
            let c = u8::try_from(legal.trailing_zeros()).unwrap_or(0);
            legal &= legal - 1;
            if let Some(prev) = last_kept {
                if self.equivalent(seat, prev, c) {
                    continue;
                }
            }
            last_kept = Some(c);
            let i = usize::from(c);
            let pts = self.points[i];
            let strength = i32::from(self.strength[i]);
            let score = if c == tt_move {
                10_000
            } else if self.null {
                let declarer = seat == 0;
                match holder {
                    None => 100 - strength,
                    Some(h) => {
                        let wins = self.beats(lead_kind, h, c);
                        if declarer {
                            if wins { -strength } else { 200 + strength }
                        } else if wins {
                            50 - strength
                        } else {
                            100 + strength
                        }
                    }
                }
            } else {
                match holder {
                    None => {
                        if self.is_top(seat, c) {
                            300 + pts
                        } else {
                            100 - 2 * pts - strength
                        }
                    }
                    Some(h) => {
                        let wins = self.beats(lead_kind, h, c);
                        if partner_wins {
                            if wins {
                                if last { -pts } else { 50 - pts }
                            } else {
                                150 + pts
                            }
                        } else if wins {
                            let sure = last || self.is_top(seat, c);
                            if sure {
                                300 - strength
                            } else {
                                200 + pts - strength
                            }
                        } else {
                            100 - pts
                        }
                    }
                }
            };
            let history = self.history[seat][i];
            let bonus = i32::try_from(32 - history.leading_zeros()).unwrap_or(0);
            moves[n] = c;
            scores[n] = score * 32 + bonus;
            n += 1;
        }
        // Insertion sort, highest score first (stable for equal scores).
        for i in 1..n {
            let mut j = i;
            while j > 0 && scores[j] > scores[j - 1] {
                scores.swap(j, j - 1);
                moves.swap(j, j - 1);
                j -= 1;
            }
        }
        (moves, n)
    }

    /// Static estimate of the remaining tricks behind the depth limit: a
    /// greedy play-out with open cards ([`Search::greedy_card`]).
    fn estimate(&self) -> i32 {
        let mut hands = self.hands;
        let mut leader = self.leader;
        let mut value = 0;
        while hands[leader] != 0 {
            let mut trick = [0u8; 3];
            for k in 0..3 {
                let seat = (leader + k) % 3;
                let Some(card) = self.greedy_card(&hands, seat, &trick[..k], leader) else {
                    return value;
                };
                trick[k] = card;
                hands[seat] &= !(1 << card);
            }
            let winner = (leader + self.winner_of(&trick)) % 3;
            if self.gain & (1 << winner) != 0 {
                value += trick
                    .iter()
                    .map(|&c| self.points[usize::from(c)])
                    .sum::<i32>();
                if self.ramsch && hands[winner] == 0 {
                    value += self.skat_points;
                }
            }
            leader = winner;
        }
        value
    }

    /// Null behind the depth limit: a greedy play-out with open cards
    /// (the declarer ducks as high as he can, the defenders keep him on
    /// top). 1 if the declarer takes a trick.
    fn null_estimate(&self) -> i32 {
        let mut hands = self.hands;
        let mut leader = self.leader;
        while hands[leader] != 0 {
            let mut trick = [0u8; 3];
            for k in 0..3 {
                let seat = (leader + k) % 3;
                let legal = self.legal_of(hands[seat], &trick[..k]);
                let Some(card) = self.null_greedy(legal, seat, &trick[..k], leader) else {
                    return 0;
                };
                trick[k] = card;
                hands[seat] &= !(1 << card);
            }
            let winner = (leader + self.winner_of(&trick)) % 3;
            if winner == 0 {
                return 1;
            }
            leader = winner;
        }
        0
    }

    fn null_greedy(&self, legal: u32, seat: usize, trick: &[u8], leader: usize) -> Option<u8> {
        let mut rest = legal;
        let cards = std::iter::from_fn(move || {
            if rest == 0 {
                return None;
            }
            let c = u8::try_from(rest.trailing_zeros()).unwrap_or(0);
            rest &= rest - 1;
            Some(c)
        });
        let strength = |c: u8| self.strength[usize::from(c)];
        if trick.is_empty() {
            return cards.min_by_key(|&c| strength(c));
        }
        let offset = self.winner_of(trick);
        let holder = trick[offset];
        let lead_kind = self.lead_kind_of(trick);
        let declarer_played = (0..trick.len()).any(|k| (leader + k) % 3 == 0);
        let declarer_wins = (leader + offset) % 3 == 0;
        let under = |c: &u8| !self.beats(lead_kind, holder, *c);
        if seat == 0 || declarer_wins {
            // Duck as high as possible; if nothing ducks, the lowest.
            let all: Vec<u8> = cards.collect();
            return all
                .iter()
                .copied()
                .filter(under)
                .max_by_key(|&c| strength(c))
                .or_else(|| all.iter().copied().min_by_key(|&c| strength(c)));
        }
        if declarer_played {
            cards.max_by_key(|&c| strength(c))
        } else {
            cards.min_by_key(|&c| strength(c))
        }
    }

    /// Kind of the effective lead of `trick`, `BLANK` if none.
    fn lead_kind_of(&self, trick: &[u8]) -> u8 {
        trick
            .iter()
            .map(|&c| self.kind[usize::from(c)])
            .find(|&k| k != BLANK)
            .unwrap_or(BLANK)
    }

    /// Offset of the card winning `trick` so far.
    fn winner_of(&self, trick: &[u8]) -> usize {
        let lead_kind = self.lead_kind_of(trick);
        let mut best = 0;
        for i in 1..trick.len() {
            if self.beats(lead_kind, trick[best], trick[i]) {
                best = i;
            }
        }
        best
    }

    /// Legal cards of `hand` after `trick`.
    fn legal_of(&self, hand: u32, trick: &[u8]) -> u32 {
        let lead_kind = self.lead_kind_of(trick);
        if lead_kind == BLANK {
            return hand;
        }
        let follow = hand & self.kind_mask[usize::from(lead_kind)];
        if follow == 0 {
            hand
        } else {
            follow | (hand & self.kind_mask[usize::from(BLANK)])
        }
    }

    /// Whether a seat of the other side still to play after `trick` (with
    /// `card` added) can take the trick away from the current winner.
    fn overtaken(
        &self,
        hands: &[u32; 3],
        seat: usize,
        trick: &[u8],
        card: u8,
        leader: usize,
    ) -> bool {
        let mut next = [0u8; 3];
        next[..trick.len()].copy_from_slice(trick);
        next[trick.len()] = card;
        let next = &next[..=trick.len()];
        let holder = next[self.winner_of(next)];
        let lead_kind = self.lead_kind_of(next);
        let side = self.maxi & (1 << seat) != 0;
        (next.len()..3).any(|k| {
            let other = (leader + k) % 3;
            if (self.maxi & (1 << other) != 0) == side {
                return false;
            }
            let mut legal = self.legal_of(hands[other], next);
            while legal != 0 {
                let c = u8::try_from(legal.trailing_zeros()).unwrap_or(0);
                legal &= legal - 1;
                if self.beats(lead_kind, holder, c) {
                    return true;
                }
            }
            false
        })
    }

    /// A plausible card with open hands: cash sure winners, smear onto a
    /// partner's sure trick, win cheaply when it holds, else throw the
    /// cheapest card.
    fn greedy_card(
        &self,
        hands: &[u32; 3],
        seat: usize,
        trick: &[u8],
        leader: usize,
    ) -> Option<u8> {
        let legal = self.legal_of(hands[seat], trick);
        if legal == 0 {
            return None;
        }
        let cards = || {
            let mut rest = legal;
            std::iter::from_fn(move || {
                if rest == 0 {
                    return None;
                }
                let c = u8::try_from(rest.trailing_zeros()).unwrap_or(0);
                rest &= rest - 1;
                Some(c)
            })
        };
        let cheapest = || {
            cards().min_by_key(|&c| {
                let i = usize::from(c);
                (self.kind[i] == TRUMP, self.points[i], self.strength[i])
            })
        };
        if trick.is_empty() {
            // Highest-scoring card nobody can take away, else the cheapest.
            let sure = cards()
                .filter(|&c| !self.overtaken(hands, seat, &[], c, leader))
                .max_by_key(|&c| {
                    let i = usize::from(c);
                    (self.kind[i] == TRUMP, self.points[i], self.strength[i])
                });
            return sure.or_else(cheapest);
        }
        let holder_offset = self.winner_of(trick);
        let winner = (leader + holder_offset) % 3;
        let lead_kind = self.lead_kind_of(trick);
        let holder = trick[holder_offset];
        let side = self.maxi & (1 << seat) != 0;
        let partner_wins = (self.maxi & (1 << winner) != 0) == side;
        let mut beating = 0u32;
        for c in cards() {
            if self.beats(lead_kind, holder, c) {
                beating |= 1 << c;
            }
        }
        if partner_wins {
            let losing = cards().filter(|&c| beating & (1 << c) == 0);
            let smear = losing.max_by_key(|&c| {
                let i = usize::from(c);
                (self.kind[i] != TRUMP, self.points[i])
            });
            if let Some(c) = smear {
                if !self.overtaken(hands, seat, trick, c, leader) {
                    return Some(c);
                }
            }
        }
        let safe_win = cards()
            .filter(|&c| beating & (1 << c) != 0)
            .filter(|&c| !self.overtaken(hands, seat, trick, c, leader))
            .min_by_key(|&c| {
                let i = usize::from(c);
                (self.kind[i] == TRUMP, self.strength[i], -self.points[i])
            });
        safe_win.or_else(cheapest)
    }

    /// Value of the remaining tricks from the current state.
    fn search(&mut self, mut alpha: i32, mut beta: i32) -> i32 {
        self.nodes += 1;
        let boundary = self.trick_len == 0;
        if boundary && self.hands[self.leader] == 0 {
            return 0;
        }
        if self.null {
            if alpha >= 1 {
                return 1;
            }
            if beta <= 0 {
                return 0;
            }
            if boundary && self.out_of_budget() {
                return self.null_estimate();
            }
        } else {
            let max = self.live_points + if self.ramsch { self.skat_points } else { 0 };
            if beta <= 0 {
                return 0;
            }
            if alpha >= max {
                return max;
            }
            if boundary && self.out_of_budget() {
                return self.estimate();
            }
        }
        let tag = self.tag();
        let slot = self.table.slot(self.hands, tag);
        let mut tt_move = NO_MOVE;
        let e = self.table.entries[slot];
        if e.generation == self.table.generation && e.hands == self.hands && e.tag == tag {
            match e.flag {
                EXACT => return e.value,
                LOWER => alpha = alpha.max(e.value),
                _ => beta = beta.min(e.value),
            }
            if alpha >= beta {
                return e.value;
            }
            tt_move = e.best;
        }
        let seat = self.to_play();
        let maximize = self.maxi & (1 << seat) != 0;
        let (moves, n) = self.ordered_moves(seat, tt_move);
        let (alpha0, beta0) = (alpha, beta);
        let mut best = if maximize { -INF } else { INF };
        let mut best_move = NO_MOVE;
        for &m in &moves[..n] {
            let v = self.play(seat, m, alpha, beta);
            if maximize {
                if v > best {
                    best = v;
                    best_move = m;
                }
                alpha = alpha.max(v);
            } else {
                if v < best {
                    best = v;
                    best_move = m;
                }
                beta = beta.min(v);
            }
            if alpha >= beta {
                let weight = self.live().count_ones().pow(2);
                let h = &mut self.history[seat][usize::from(m)];
                *h = h.saturating_add(1 + weight);
                break;
            }
        }
        let flag = if best <= alpha0 {
            UPPER
        } else if best >= beta0 {
            LOWER
        } else {
            EXACT
        };
        self.table.entries[slot] = Entry {
            hands: self.hands,
            tag,
            flag,
            best: best_move,
            generation: self.table.generation,
            value: best,
        };
        best
    }

    /// Leader, cards on the table and their order: with the hands the
    /// complete key of a position.
    fn tag(&self) -> u32 {
        let mut tag = u32::try_from(self.leader).unwrap_or(0)
            | (u32::try_from(self.trick_len).unwrap_or(0) << 2);
        for (k, &c) in self.trick[..self.trick_len].iter().enumerate() {
            tag |= (u32::from(c) + 1) << (4 + 8 * k);
        }
        tag
    }

    /// Play card `m` for `seat`, search on, take it back.
    fn play(&mut self, seat: usize, m: u8, alpha: i32, beta: i32) -> i32 {
        self.hands[seat] &= !(1 << m);
        self.trick[self.trick_len] = m;
        self.trick_len += 1;
        let value = if self.trick_len == 3 {
            let winner = (self.leader + self.winning_offset()) % 3;
            let points: i32 = self
                .trick
                .iter()
                .map(|&c| self.points[usize::from(c)])
                .sum();
            let saved_leader = self.leader;
            let saved_trick = self.trick;
            if self.null && winner == 0 {
                1
            } else {
                let mut gained = if self.gain & (1 << winner) != 0 {
                    points
                } else {
                    0
                };
                let last = self.hands[winner] == 0;
                if last && self.ramsch && self.gain & (1 << winner) != 0 {
                    gained += self.skat_points;
                }
                self.trick_len = 0;
                self.leader = winner;
                self.live_points -= points;
                let depth = self.depth_left;
                self.depth_left = depth.saturating_sub(1);
                let v = gained + self.search(alpha - gained, beta - gained);
                self.depth_left = depth;
                self.live_points += points;
                self.leader = saved_leader;
                self.trick = saved_trick;
                self.trick_len = 3;
                v
            }
        } else {
            self.search(alpha, beta)
        };
        self.trick_len -= 1;
        self.hands[seat] |= 1 << m;
        value
    }
}

/// Bits `lo..=hi`.
fn range_mask(lo: u32, hi: u32) -> u32 {
    if lo > hi {
        return 0;
    }
    let upper = if hi >= 31 {
        u32::MAX
    } else {
        (1 << (hi + 1)) - 1
    };
    let lower = (1u32 << lo) - 1;
    upper & !lower
}

/// Kind of a card: suit 0..4 for non-trumps, [`TRUMP`], [`BLANK`].
fn card_kind<C: PlayingCard>(game: GameType, card: C) -> u8 {
    match card.face() {
        None => BLANK,
        Some(_) if rules::is_trump(game, card) => TRUMP,
        Some(face) => face.suit as u8,
    }
}

/// Strength within the kind, 0 for cards that never win. Trumps follow
/// [`rules::trump_order_key`] (a "Bube ehrenhalber" half a step below the
/// printed card of equal strength), so equal values really tie.
fn card_strength<C: PlayingCard>(game: GameType, card: C) -> u8 {
    if card.is_blank() || card.never_wins() {
        return 0;
    }
    match rules::trump_order_key(game, card) {
        Some((strength, printed)) => strength.saturating_mul(2).saturating_add(printed),
        None => rules::suit_strength(game, card),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cards::{Card, CardKind, Suit, parse_cards, standard_deck};
    use crate::deck::DealtCard;
    use crate::game::seat_at;
    use crate::rng::Rng;

    fn c(s: &str) -> Card {
        s.parse().expect("test card")
    }

    fn sorted(s: &str) -> Vec<Card> {
        let mut cards = parse_cards(s).expect("cards");
        cards.sort();
        cards
    }

    /// Plain minimax over the rules of [`crate::rules`], no pruning.
    fn reference<C: PlayingCard>(
        game: GameType,
        goal: Goal,
        hands: &mut [Vec<C>; 3],
        lead: Seat,
        trick: &mut Vec<C>,
        skat_points: u32,
    ) -> i32 {
        let seat = seat_at(lead, trick.len());
        let legal = rules::legal_moves(game, &hands[seat.index()], rules::effective_lead(trick));
        if legal.is_empty() {
            return 0;
        }
        let maximize = match goal {
            Goal::DeclarerPoints => seat.is_declarer(),
            Goal::NullDeclarerTrick => !seat.is_declarer(),
            Goal::RamschAvoid(me) => seat == me,
        };
        let mut best: Option<i32> = None;
        for card in legal {
            hands[seat.index()].retain(|&x| x != card);
            trick.push(card);
            let v = if trick.len() == 3 {
                let cards = [trick[0], trick[1], trick[2]];
                let winner = Seat::from_index(rules::trick_winner(game, lead.index(), cards))
                    .unwrap_or(lead);
                let points: u32 = cards.iter().map(|c| c.card_points()).sum();
                let saved = std::mem::take(trick);
                let last = hands[winner.index()].is_empty();
                let v = match goal {
                    Goal::NullDeclarerTrick if winner.is_declarer() => 1,
                    Goal::NullDeclarerTrick => {
                        reference(game, goal, hands, winner, trick, skat_points)
                    }
                    Goal::DeclarerPoints => {
                        let gained = if winner.is_declarer() { points } else { 0 };
                        i32::try_from(gained).unwrap_or(0)
                            + reference(game, goal, hands, winner, trick, skat_points)
                    }
                    Goal::RamschAvoid(me) => {
                        let mut gained = 0;
                        if winner != me {
                            gained = points + if last { skat_points } else { 0 };
                        }
                        i32::try_from(gained).unwrap_or(0)
                            + reference(game, goal, hands, winner, trick, skat_points)
                    }
                };
                *trick = saved;
                v
            } else {
                reference(game, goal, hands, lead, trick, skat_points)
            };
            trick.pop();
            hands[seat.index()].push(card);
            hands[seat.index()].sort();
            best = Some(match best {
                None => v,
                Some(b) if maximize => b.max(v),
                Some(b) => b.min(v),
            });
        }
        best.unwrap_or(0)
    }

    fn reference_moves<C: PlayingCard>(
        game: GameType,
        goal: Goal,
        layout: &Layout<C>,
    ) -> Vec<(C, i32)> {
        let seat = seat_at(layout.trick_lead, layout.trick.len());
        let legal = rules::legal_moves(
            game,
            &layout.hands[seat.index()],
            rules::effective_lead(&layout.trick),
        );
        legal
            .into_iter()
            .map(|card| {
                let mut hands = layout.hands.clone();
                hands[seat.index()].retain(|&x| x != card);
                let mut trick = layout.trick.clone();
                trick.push(card);
                // Let the reference play the rest: a one-card hand that
                // only holds `card` is forced.
                let mut forced = hands.clone();
                forced[seat.index()].push(card);
                forced[seat.index()].sort();
                let mut only = layout.hands.clone();
                only[seat.index()] = vec![card];
                let _ = only;
                trick.pop();
                // Evaluate by restricting the seat's choice to `card`.
                let mut restricted = forced;
                let value = reference_first(game, goal, &mut restricted, layout, card);
                (card, value)
            })
            .collect()
    }

    /// Reference value after `seat` plays `card` first.
    fn reference_first<C: PlayingCard>(
        game: GameType,
        goal: Goal,
        hands: &mut [Vec<C>; 3],
        layout: &Layout<C>,
        card: C,
    ) -> i32 {
        let lead = layout.trick_lead;
        let seat = seat_at(lead, layout.trick.len());
        let mut trick = layout.trick.clone();
        hands[seat.index()].retain(|&x| x != card);
        trick.push(card);
        if trick.len() < 3 {
            return reference(game, goal, hands, lead, &mut trick, layout.skat_points);
        }
        let cards = [trick[0], trick[1], trick[2]];
        let winner =
            Seat::from_index(rules::trick_winner(game, lead.index(), cards)).unwrap_or(lead);
        let points: u32 = cards.iter().map(|c| c.card_points()).sum();
        let last = hands[winner.index()].is_empty();
        let mut empty = Vec::new();
        let rest = |hands: &mut [Vec<C>; 3], empty: &mut Vec<C>| {
            reference(game, goal, hands, winner, empty, layout.skat_points)
        };
        match goal {
            Goal::NullDeclarerTrick if winner.is_declarer() => 1,
            Goal::NullDeclarerTrick => rest(hands, &mut empty),
            Goal::DeclarerPoints => {
                let gained = if winner.is_declarer() { points } else { 0 };
                i32::try_from(gained).unwrap_or(0) + rest(hands, &mut empty)
            }
            Goal::RamschAvoid(me) => {
                let gained = if winner == me {
                    0
                } else {
                    points + if last { layout.skat_points } else { 0 }
                };
                i32::try_from(gained).unwrap_or(0) + rest(hands, &mut empty)
            }
        }
    }

    /// A random position with `tricks` tricks left and `on_table` cards of
    /// the current trick already played.
    fn random_layout<C: PlayingCard>(
        rng: &mut Rng,
        mut deck: Vec<C>,
        tricks: usize,
        on_table: usize,
    ) -> Layout<C> {
        rng.shuffle(&mut deck);
        let lead = Seat::ALL[usize::try_from(rng.gen_range(0..3)).unwrap_or(0)];
        let mut hands: [Vec<C>; 3] = [Vec::new(), Vec::new(), Vec::new()];
        let mut it = deck.into_iter();
        for (offset, seat) in (0..3).map(|o| (o, seat_at(lead, o))) {
            let n = if offset < on_table {
                tricks - 1
            } else {
                tricks
            };
            hands[seat.index()] = it.by_ref().take(n).collect();
            hands[seat.index()].sort();
        }
        let trick: Vec<C> = it.by_ref().take(on_table).collect();
        let skat_points = it.take(2).map(|c| c.card_points()).sum();
        Layout {
            hands,
            trick_lead: lead,
            trick,
            skat_points,
        }
    }

    fn games() -> [GameType; 3] {
        [
            GameType::Suit(Suit::Hearts),
            GameType::Grand,
            GameType::Null,
        ]
    }

    #[test]
    fn matches_plain_minimax_on_small_endgames() {
        let mut rng = Rng::from_seed(11);
        for round in 0..240 {
            let game = games()[round % 3];
            let tricks = 2 + round % 3;
            let on_table = (round / 3) % 3;
            let layout = random_layout(&mut rng, standard_deck(), tricks, on_table);
            let goals = match game {
                GameType::Null => vec![Goal::NullDeclarerTrick],
                _ => vec![
                    Goal::DeclarerPoints,
                    Goal::RamschAvoid(Seat::ALL[round % 3]),
                ],
            };
            for goal in goals {
                let fast = solve(game, goal, &layout);
                let slow = reference_moves(game, goal, &layout);
                assert_eq!(fast, slow, "round {round} {game} {goal:?} {layout:?}");
            }
        }
    }

    /// A roguelike deck: blanks, a rank bonus, a card that never wins and
    /// one with extra Augen.
    fn odd_deck() -> Vec<DealtCard> {
        standard_deck()
            .into_iter()
            .zip(0u32..)
            .map(|(card, id)| {
                let blank = card.rank == crate::cards::Rank::Seven && card.suit != Suit::Clubs;
                let kind = if blank {
                    CardKind::Blank
                } else {
                    CardKind::Normal(card)
                };
                let mut dealt = DealtCard::plain(id, kind);
                if card == c("SK") || card == c("H9") {
                    dealt.rank_bonus = 2;
                }
                if card == c("DA") || card == c("C8") {
                    dealt.never_wins = true;
                }
                if card == c("HQ") {
                    dealt.card_points = 9;
                }
                if card == c("SQ") || card == c("DA") || card == c("C9") {
                    dealt.honorary_jack = true;
                }
                dealt
            })
            .collect()
    }

    #[test]
    fn matches_plain_minimax_with_blanks_and_modified_cards() {
        let mut rng = Rng::from_seed(12);
        for round in 0..240 {
            let game = games()[round % 3];
            let layout = random_layout(&mut rng, odd_deck(), 2 + round % 3, (round / 3) % 3);
            let goal = if game == GameType::Null {
                Goal::NullDeclarerTrick
            } else {
                Goal::DeclarerPoints
            };
            let fast = solve(game, goal, &layout);
            let slow = reference_moves(game, goal, &layout);
            assert_eq!(fast, slow, "round {round} {game} {layout:?}");
        }
    }

    #[test]
    fn trick_resolution_mirrors_the_rules() {
        let mut rng = Rng::from_seed(13);
        for round in 0..2000 {
            let game = games()[round % 3];
            let layout = random_layout(&mut rng, odd_deck(), 1, 0);
            let mut table = Table::new(4);
            let (mut search, cards) = Search::new(
                game,
                Goal::DeclarerPoints,
                &layout,
                Limits::default(),
                &mut table,
            )
            .expect("valid layout");
            let order: Vec<DealtCard> = (0..3)
                .map(|o| layout.hands[seat_at(layout.trick_lead, o).index()][0])
                .collect();
            for (k, card) in order.iter().enumerate() {
                let i = cards.iter().position(|x| x == card).expect("indexed");
                search.trick[k] = u8::try_from(i).expect("small");
            }
            search.trick_len = 3;
            assert_eq!(
                Some(search.winning_offset()),
                rules::winning_index(game, &order),
                "{game} {order:?}"
            );
        }
    }

    #[test]
    fn hand_checked_three_trick_endgame() {
        // Hearts game, declarer (seat 0) leads, three tricks left.
        // Declarer CJ HA ST, seat 1 SJ SA S7, seat 2 HT SK S8.
        let hands = [sorted("CJ HA ST"), sorted("SJ SA S7"), sorted("HT SK S8")];
        let layout = Layout {
            hands,
            trick_lead: Seat::DECLARER,
            trick: Vec::new(),
            skat_points: 0,
        };
        let values = solve(GameType::Suit(Suit::Hearts), Goal::DeclarerPoints, &layout);
        let value = |card: &str| {
            values
                .iter()
                .find(|(x, _)| *x == c(card))
                .map(|&(_, v)| v)
                .expect("legal")
        };
        // CJ draws SJ and HT (14). The HA then wins a trick the defenders
        // throw S7 and S8 into (11); the ST loses to the SA. 25.
        assert_eq!(value("CJ"), 25);
        // HA runs into the SJ (seat 1 must follow trump) with the HT: 23
        // for the defenders. Seat 1 cashes the SA (seat 2 smears the SK
        // onto it) and leads S7: the CJ ruffs S7 and S8. 2.
        assert_eq!(value("HA"), 2);
        // ST: seat 1 takes it with the SA, seat 2 smears the SK (25 for
        // the defenders). Whatever seat 1 leads next, the declarer gets
        // the SJ trick with the CJ (SJ, HT: 14) and the S7 trick with the
        // HA (S8: 11). Ducking with the S7 instead would hand the declarer
        // the ST, then CJ and HA over SA and SK: 50.
        assert_eq!(value("ST"), 25);
    }

    #[test]
    fn null_declarer_is_caught_with_a_lone_high_card() {
        // Null: declarer holds DA and D7, seat 1 leads the D8; seat 2 has
        // no diamonds. The declarer ducks with D7, then must lead DA... a
        // lead is won by the leader unless someone follows higher.
        let hands = [sorted("DA D7"), sorted("D8 C7"), sorted("H7 H8")];
        let layout = Layout {
            hands,
            trick_lead: Seat::ALL[1],
            trick: Vec::new(),
            skat_points: 0,
        };
        let values = solve(GameType::Null, Goal::NullDeclarerTrick, &layout);
        // Seat 1 leads: D8 forces the declarer under with D7, then the
        // declarer leads DA and wins: caught either way after D8? After
        // D8 D7 H7 seat 1 wins and leads C7: declarer sloughs DA. Safe.
        // Leading C7 first: declarer sloughs DA, then D8 vs D7 is safe.
        assert_eq!(values, vec![(c("D8"), 0), (c("C7"), 0)]);
        // With the D9 instead of the D8 at seat 1 nothing changes; with
        // the declarer holding D9 instead of D7 he must take the D8 lead.
        let hands = [sorted("DA D9"), sorted("D8 C7"), sorted("H7 H8")];
        let layout = Layout { hands, ..layout };
        let values = solve(GameType::Null, Goal::NullDeclarerTrick, &layout);
        assert_eq!(values, vec![(c("D8"), 1), (c("C7"), 1)]);
    }
}

#[cfg(test)]
mod bench {
    use super::*;
    use crate::cards::standard_deck;
    use crate::game::seat_at;
    use crate::rng::Rng;

    /// `cargo test --release -p skat_core bench_solver -- --ignored --nocapture`
    #[test]
    #[ignore = "measurement"]
    fn bench_solver() {
        let mut rng = Rng::from_seed(5);
        let mut table = Table::new(20);
        for tricks in [4usize, 6, 8, 9] {
            let mut total = 0u64;
            let mut max = 0u64;
            let mut probe_total = 0u64;
            let mut probe_max = 0u64;
            let start = std::time::Instant::now();
            let n = 50;
            for _ in 0..n {
                let mut deck = standard_deck();
                rng.shuffle(&mut deck);
                let lead = Seat::ALL[1];
                let mut hands: [Vec<_>; 3] = [Vec::new(), Vec::new(), Vec::new()];
                for o in 0..3 {
                    let seat = seat_at(lead, o);
                    hands[seat.index()] = deck[o * tricks..(o + 1) * tricks].to_vec();
                    hands[seat.index()].sort();
                }
                let layout = Layout {
                    hands,
                    trick_lead: lead,
                    trick: Vec::new(),
                    skat_points: 0,
                };
                let (_, nodes) = solve_with(
                    GameType::Suit(crate::cards::Suit::Clubs),
                    Goal::DeclarerPoints,
                    &layout,
                    Limits::default(),
                    &mut table,
                );
                total += nodes;
                max = max.max(nodes);
                let (_, nodes) = probe_with(
                    GameType::Suit(crate::cards::Suit::Clubs),
                    Goal::DeclarerPoints,
                    &layout,
                    i32::try_from(tricks * 5).unwrap_or(0),
                    Limits::default(),
                    &mut table,
                );
                probe_total += nodes;
                probe_max = probe_max.max(nodes);
            }
            println!(
                "tricks {tricks}: mean nodes {} max {} time per solve {:.3} ms",
                total / n,
                max,
                start.elapsed().as_secs_f64() * 1000.0 / n as f64
            );
            println!("  probe: mean nodes {} max {}", probe_total / n, probe_max);
        }
    }
}

//! Double-dummy solver: the exact result of the remaining tricks when every
//! hand is known (DESIGN 10.2, stage 2).
//!
//! The PIMC policies ([`super::pimc`]) deal the unknown cards and ask this
//! solver how every legal move ends with perfect information. It is an
//! alpha-beta search over single cards with
//!
//! - card indices in a `u32` bit mask per hand (at most 32 cards are live:
//!   three hands plus the cards on the table),
//! - a transposition table probed at every node (the value of the rest
//!   depends only on who holds what, who led and the cards on the table;
//!   within one search the cards still held say who holds what),
//! - the static estimates at the depth or node limit stored in the
//!   transposition table like searched values, and in a cache of their own,
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
use crate::rng::mix64;
use crate::rules::{self, GameType};

/// Kind of a non-trump card: its suit (0..4). Trumps and blanks follow.
const TRUMP: u8 = 4;
/// Kind of a blank: follows nothing, never wins.
pub(crate) const BLANK: u8 = 5;
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
    /// Schwarz announced (Schwarz angesagt, a suit or grand Ouvert): 1 if
    /// a defender takes a trick from now on, else 0. The defenders
    /// maximize, the declarer minimizes.
    DefenderTrick,
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
    /// Augen in the skat with their sign ([`PlayingCard::signed_points`]);
    /// in a Ramsch they go to the winner of the last trick.
    pub skat_points: i32,
}

/// Search limits.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Limits {
    /// Tricks searched exactly, the current one included; behind them a
    /// static estimate. `0` searches to the end.
    pub depth: u32,
    /// Nodes per move of the seat to play after which every further trick
    /// boundary of that move's search is estimated instead of searched, as
    /// if the depth were used up. `0` = no limit. Keeps the worst case of a
    /// search bounded, deterministically. Every move gets the whole limit:
    /// a limit shared by all moves left the moves late in the legal order
    /// with one trick and the estimate once the early ones had used it up.
    pub nodes: u64,
}

/// One slot of the transposition table.
#[derive(Debug, Clone, Copy, Default)]
struct Entry {
    /// [`position_key`] of the position.
    key: u64,
    flag: u8,
    best: u8,
    generation: u32,
    value: i32,
}

/// One slot of the leaf-estimate cache: an estimate of one search
/// generation for the hands and the leader at a trick boundary.
#[derive(Debug, Clone, Copy, Default)]
struct Estimate {
    /// [`position_key`] of the trick boundary.
    key: u64,
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
    /// Leaf estimates behind the depth or node limit, valid in their
    /// generation only (see [`Search::cached_estimate`]).
    estimates: Vec<Estimate>,
}

impl Table {
    /// A table with `2^bits` slots.
    #[must_use]
    pub fn new(bits: u32) -> Table {
        Table {
            entries: vec![Entry::default(); 1 << bits.clamp(4, 24)],
            generation: 0,
            estimates: vec![Estimate::default(); 1 << 12],
        }
    }

    fn next_generation(&mut self) {
        self.generation = self.generation.wrapping_add(1);
        if self.generation == 0 {
            self.entries.fill(Entry::default());
            self.estimates.fill(Estimate::default());
            self.generation = 1;
        }
    }

    fn slot(&self, hands: [u32; 3], tag: u32) -> usize {
        let a = u64::from(hands[0]) | (u64::from(hands[1]) << 32);
        let b = u64::from(hands[2]) | (u64::from(tag) << 32);
        let h = mix64(a ^ mix64(b));
        (h as usize) & (self.entries.len() - 1)
    }
}

/// The key of a position within one search: the cards still in the
/// hands and the [`Search::tag`]. Who holds a live card follows from the
/// deal the search started from (every card has one holder), so the
/// union of the hands says as much as the three hands.
fn position_key(hands: [u32; 3], tag: u32) -> u64 {
    u64::from(hands[0] | hands[1] | hands[2]) | (u64::from(tag) << 32)
}

/// Most spare tables [`with_table`] keeps.
const SPARE_TABLES: usize = 4;

std::thread_local! {
    /// Spare tables of [`with_table`]. Not shared state: a search starts
    /// a new generation, which invalidates whatever an earlier search
    /// left, so results never depend on which table a search gets. The
    /// core runs single-threaded (on `wasm32-unknown-unknown` this is a
    /// plain static); the simulator's threads each get their own.
    static SPARE: std::cell::RefCell<Vec<Table>> = const { std::cell::RefCell::new(Vec::new()) };
}

/// Run `f` with a table of `2^bits` slots, reused from an earlier call
/// instead of allocating one (a table of 15 bits holds about 900 KB).
/// Nested calls get a table of their own.
pub fn with_table<R>(bits: u32, f: impl FnOnce(&mut Table) -> R) -> R {
    let len = 1usize << bits.clamp(4, 24);
    let spare = SPARE.with(|spare| {
        let mut spare = spare.borrow_mut();
        let at = spare.iter().position(|t| t.entries.len() == len)?;
        Some(spare.swap_remove(at))
    });
    let mut table = spare.unwrap_or_else(|| Table::new(bits));
    let result = f(&mut table);
    SPARE.with(|spare| {
        let mut spare = spare.borrow_mut();
        if spare.len() < SPARE_TABLES {
            spare.push(table);
        }
    });
    result
}

impl Default for Table {
    fn default() -> Table {
        Table::new(16)
    }
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
    /// Null card play (move ordering and estimate of a Null).
    null: bool,
    /// Seats whose first trick ends the search with value 1 (Null: the
    /// declarer, Schwarz announced: the defenders); `0` for point goals.
    catch: u8,
    ramsch: bool,
    skat_points: i32,
    hands: [u32; 3],
    trick: [u8; 3],
    trick_len: usize,
    leader: usize,
    /// Augen still in the hands and on the table, the positive ones
    /// (`live_gain`, at least 0) and the ones below zero (`live_loss`, at
    /// most 0) apart: what the rest of the search can add lies between
    /// their sums.
    live_gain: i32,
    live_loss: i32,
    /// Tricks left to search exactly (`u32::MAX` = no limit).
    depth_left: u32,
    table: &'t mut Table,
    nodes: u64,
    /// See [`Limits::nodes`].
    node_limit: u64,
    /// Nodes when the search of the current root move began: the node
    /// limit counts from there.
    move_start: u64,
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
    let Some(roots) = RootMoves::new(&search, &cards, &legal) else {
        return (Vec::new(), 0);
    };
    let values: Vec<i32> = roots
        .distinct
        .iter()
        .map(|&m| {
            search.begin_root_move();
            search.play(seat, m, -INF, INF)
        })
        .collect();
    let out = legal
        .into_iter()
        .zip(&roots.slot)
        .map(|(card, &k)| (card, values[k]))
        .collect();
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
    match Probes::new(game, goal, layout, limits, table) {
        Some(mut probes) => probes.probe(threshold),
        None => (Vec::new(), 0),
    }
}

/// One position prepared for several [`probe_with`] calls with different
/// thresholds: the cards are sorted and indexed once. Every probe is a
/// search of its own (a new table generation, no history carried over),
/// so the results equal separate calls of [`probe_with`].
pub struct Probes<'t, C> {
    search: Search<'t>,
    legal: Vec<C>,
    /// `None` if a legal card is not among the cards of the layout.
    roots: Option<RootMoves>,
    limits: Limits,
    fresh: bool,
}

/// The legal moves of the seat to play at the root of a search, equivalent
/// moves merged: they share one search.
struct RootMoves {
    /// Card indices of the moves searched, in the order of the legal moves.
    distinct: Vec<u8>,
    /// Per legal move the position of its search in `distinct`.
    slot: Vec<usize>,
}

impl RootMoves {
    /// `None` if a legal card is not among `cards`.
    fn new<C: PlayingCard>(search: &Search<'_>, cards: &[C], legal: &[C]) -> Option<RootMoves> {
        let others = search.separators(search.to_play());
        let mut distinct: Vec<u8> = Vec::new();
        let mut slot = Vec::with_capacity(legal.len());
        for &card in legal {
            let i = u8::try_from(cards.iter().position(|&c| c == card)?).ok()?;
            let known = distinct
                .iter()
                .position(|&r| search.equivalent(others, r.min(i), r.max(i)));
            slot.push(known.unwrap_or_else(|| {
                distinct.push(i);
                distinct.len() - 1
            }));
        }
        Some(RootMoves { distinct, slot })
    }
}

impl<'t, C: PlayingCard> Probes<'t, C> {
    /// Prepare `layout`; `None` if nobody is to play or a hand is
    /// inconsistent (then [`probe_with`] returns an empty list).
    #[must_use]
    pub fn new(
        game: GameType,
        goal: Goal,
        layout: &Layout<C>,
        limits: Limits,
        table: &'t mut Table,
    ) -> Option<Probes<'t, C>> {
        let (search, cards) = Search::new(game, goal, layout, limits, table)?;
        let seat = search.to_play();
        let lead = rules::effective_lead(&layout.trick);
        let legal = rules::legal_moves(game, &layout.hands[seat], lead);
        let roots = RootMoves::new(&search, &cards, &legal);
        Some(Probes {
            search,
            legal,
            roots,
            limits,
            fresh: true,
        })
    }

    /// [`probe_with`] for `threshold`.
    pub fn probe(&mut self, threshold: i32) -> (Vec<(C, bool)>, u64) {
        if !self.fresh {
            self.search.restart(self.limits);
        }
        self.fresh = false;
        let Some(roots) = &self.roots else {
            return (Vec::new(), 0);
        };
        let search = &mut self.search;
        let seat = search.to_play();
        let reached: Vec<bool> = roots
            .distinct
            .iter()
            .map(|&m| {
                search.begin_root_move();
                search.play(seat, m, threshold - 1, threshold) >= threshold
            })
            .collect();
        let out = self
            .legal
            .iter()
            .zip(&roots.slot)
            .map(|(&card, &k)| (card, reached[k]))
            .collect();
        (out, search.nodes)
    }
}

impl<'t> Search<'t> {
    fn new<C: PlayingCard>(
        game: GameType,
        goal: Goal,
        layout: &Layout<C>,
        limits: Limits,
        table: &'t mut Table,
    ) -> Option<(Search<'t>, Vec<C>)> {
        // A hand holds at most ten cards ([`Search::ordered_moves`] keeps
        // ten moves).
        if layout.trick.len() > 2 || layout.hands.iter().any(|h| h.len() > 10) {
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
        cards.sort_by_cached_key(key);
        let n = cards.len();
        cards.dedup();
        if cards.len() != n {
            return None;
        }
        let keys: Vec<_> = cards.iter().map(key).collect();
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
            catch: 0,
            ramsch: false,
            skat_points: layout.skat_points,
            hands: [0; 3],
            trick: [0; 3],
            trick_len: layout.trick.len(),
            leader: layout.trick_lead.index(),
            live_gain: 0,
            live_loss: 0,
            depth_left: if limits.depth == 0 {
                u32::MAX
            } else {
                limits.depth
            },
            table,
            nodes: 0,
            node_limit: limits.nodes,
            move_start: 0,
            history: [[0; MAX_CARDS]; 3],
        };
        for (i, &c) in cards.iter().enumerate() {
            s.kind[i] = card_kind(game, c);
            s.strength[i] = card_strength(game, c);
            s.points[i] = i32::try_from(c.signed_points()).unwrap_or(0);
            s.kind_mask[usize::from(s.kind[i])] |= 1 << i;
            if !c.is_blank() && !c.never_wins() {
                s.can_win |= 1 << i;
            }
            s.live_gain += s.points[i].max(0);
            s.live_loss += s.points[i].min(0);
        }
        for i in 0..n {
            let same = |j: usize| s.kind[j] == s.kind[i] && s.strength[j] == s.strength[i];
            let lo = (0..=i).rev().take_while(|&j| same(j)).last().unwrap_or(i);
            let hi = (i..n).take_while(|&j| same(j)).last().unwrap_or(i);
            s.tie_lo[i] = u8::try_from(lo).unwrap_or(0);
            s.tie_hi[i] = u8::try_from(hi).unwrap_or(0);
        }
        let index = |c: &C| keys.binary_search(&key(c)).ok();
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
                s.catch = 0b001;
                s.maxi = 0b110;
            }
            Goal::DefenderTrick => {
                s.catch = 0b110;
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

    /// Start a new search of the same position (see [`Probes`]): a new
    /// table generation, no nodes, no history. The position itself is
    /// back where [`Search::new`] left it after every search.
    fn restart(&mut self, limits: Limits) {
        self.table.next_generation();
        self.nodes = 0;
        self.node_limit = limits.nodes;
        self.move_start = 0;
        self.depth_left = if limits.depth == 0 {
            u32::MAX
        } else {
            limits.depth
        };
        self.history = [[0; MAX_CARDS]; 3];
    }

    /// Start the search of a root move: it gets the whole node limit
    /// ([`Limits::nodes`]) of its own, whatever the moves before it used.
    fn begin_root_move(&mut self) {
        self.move_start = self.nodes;
    }

    /// Whether the search must estimate at the next trick boundary.
    fn out_of_budget(&self) -> bool {
        self.depth_left == 0
            || (self.node_limit > 0 && self.nodes - self.move_start > self.node_limit)
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

    /// The cards that take the trick from `holder` under `lead_kind`, as
    /// a mask: `beats(lead_kind, holder, c)` for every `c` in it.
    fn beaters(&self, lead_kind: u8, holder: u8) -> u32 {
        if self.can_win & (1 << holder) == 0 {
            return self.can_win;
        }
        let h = usize::from(holder);
        let stronger = (1u32 << self.tie_lo[h]) - 1;
        let trumps = self.kind_mask[usize::from(TRUMP)];
        let m = if self.kind[h] == TRUMP {
            trumps & stronger
        } else if self.kind[h] == lead_kind {
            trumps | (self.kind_mask[usize::from(lead_kind)] & stronger)
        } else {
            trumps | self.kind_mask[usize::from(lead_kind)]
        };
        m & self.can_win
    }

    /// Kind of the effective lead of the current trick, `BLANK` if none.
    fn lead_kind(&self) -> u8 {
        self.lead_kind_of(&self.trick[..self.trick_len])
    }

    /// Offset of the card currently winning the trick.
    fn winning_offset(&self) -> usize {
        self.winner_of(&self.trick[..self.trick_len])
    }

    /// Legal cards of `seat` in the current trick.
    fn legal(&self, seat: usize) -> u32 {
        self.legal_of(self.hands[seat], &self.trick[..self.trick_len])
    }

    /// The live cards of the other seats that can win a trick: what
    /// separates two cards of `seat` ([`Search::equivalent`]).
    fn separators(&self, seat: usize) -> u32 {
        self.live() & !self.hands[seat] & self.can_win
    }

    /// Whether cards `a < b` of one seat are interchangeable, given the
    /// [`Search::separators`] of that seat.
    fn equivalent(&self, separators: u32, a: u8, b: u8) -> bool {
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
        let separators = self.separators(seat);
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
            // Kept moves have lower indices; compare with every one, an
            // equivalent card need not be the neighbour.
            if moves[..n]
                .iter()
                .any(|&kept| self.equivalent(separators, kept, c))
            {
                continue;
            }
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

    /// A leaf estimate, from the table's estimate cache when this search
    /// already estimated the same position. Estimates are pure functions
    /// of the hands and the leader at a trick boundary (everything else
    /// they read is fixed for one search), and an entry only counts in
    /// the generation of its search.
    fn cached_estimate(&mut self, f: impl Fn(&Search<'t>) -> i32) -> i32 {
        let leader = u32::try_from(self.leader).unwrap_or(0);
        let generation = self.table.generation;
        let key = position_key(self.hands, leader);
        let a = u64::from(self.hands[0]) | (u64::from(self.hands[1]) << 32);
        let b = u64::from(self.hands[2]) | (u64::from(leader) << 32);
        let slot = (mix64(a ^ mix64(b)) as usize) & (self.table.estimates.len() - 1);
        let e = self.table.estimates[slot];
        if e.generation == generation && e.key == key {
            return e.value;
        }
        let value = f(self);
        self.table.estimates[slot] = Estimate {
            key,
            generation,
            value,
        };
        value
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

    /// Schwarz announced behind the depth limit: the greedy play-out of
    /// [`Search::estimate`]; 1 if a defender takes a trick.
    fn trick_estimate(&self) -> i32 {
        let mut hands = self.hands;
        let mut leader = self.leader;
        while hands[leader] != 0 {
            let mut trick = [0u8; 3];
            for k in 0..3 {
                let seat = (leader + k) % 3;
                let Some(card) = self.greedy_card(&hands, seat, &trick[..k], leader) else {
                    return 0;
                };
                trick[k] = card;
                hands[seat] &= !(1 << card);
            }
            let winner = (leader + self.winner_of(&trick)) % 3;
            if self.catch & (1 << winner) != 0 {
                return 1;
            }
            leader = winner;
        }
        0
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
        if seat == 0 || declarer_wins {
            // Duck as high as possible; if nothing ducks, the lowest.
            let bits = |mut rest: u32| {
                std::iter::from_fn(move || {
                    if rest == 0 {
                        return None;
                    }
                    let c = u8::try_from(rest.trailing_zeros()).unwrap_or(0);
                    rest &= rest - 1;
                    Some(c)
                })
            };
            let under = legal & !self.beaters(lead_kind, holder);
            return bits(under)
                .max_by_key(|&c| strength(c))
                .or_else(|| bits(legal).min_by_key(|&c| strength(c)));
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
            self.legal_of(hands[other], next) & self.beaters(lead_kind, holder) != 0
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
        let beating = legal & self.beaters(lead_kind, holder);
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
        if self.catch != 0 {
            if alpha >= 1 {
                return 1;
            }
            if beta <= 0 {
                return 0;
            }
        } else {
            // The rest adds at least every Augen below zero and at most
            // every positive one (a Ramsch: the skat too).
            let (mut min, mut max) = (self.live_loss, self.live_gain);
            if self.ramsch {
                min += self.skat_points.min(0);
                max += self.skat_points.max(0);
            }
            if beta <= min {
                return min;
            }
            if alpha >= max {
                return max;
            }
        }
        let tag = self.tag();
        let slot = self.table.slot(self.hands, tag);
        let mut tt_move = NO_MOVE;
        let key = position_key(self.hands, tag);
        let e = self.table.entries[slot];
        let hit = e.generation == self.table.generation && e.key == key;
        if hit {
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
        // Behind the depth or node limit: the estimate, stored like a
        // searched value (equal hands at a trick boundary have equal depth
        // left within one search), within a bound a search already proved.
        if boundary && self.out_of_budget() {
            let mut value = if self.catch == 0 {
                self.cached_estimate(Search::estimate)
            } else if self.null {
                self.cached_estimate(Search::null_estimate)
            } else {
                self.cached_estimate(Search::trick_estimate)
            };
            if hit {
                value = if e.flag == LOWER {
                    value.max(e.value)
                } else {
                    value.min(e.value)
                };
            }
            self.table.entries[slot] = Entry {
                key,
                flag: EXACT,
                best: NO_MOVE,
                generation: self.table.generation,
                value,
            };
            return value;
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
            key,
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
            let (mut gain, mut loss) = (0, 0);
            for &c in &self.trick {
                let p = self.points[usize::from(c)];
                gain += p.max(0);
                loss += p.min(0);
            }
            let points = gain + loss;
            let saved_leader = self.leader;
            let saved_trick = self.trick;
            if self.catch & (1 << winner) != 0 {
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
                self.live_gain -= gain;
                self.live_loss -= loss;
                let depth = self.depth_left;
                self.depth_left = depth.saturating_sub(1);
                let v = gained + self.search(alpha - gained, beta - gained);
                self.depth_left = depth;
                self.live_gain += gain;
                self.live_loss += loss;
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
pub(crate) fn card_kind<C: PlayingCard>(game: GameType, card: C) -> u8 {
    match card.face() {
        None => BLANK,
        Some(_) if rules::is_trump(game, card) => TRUMP,
        Some(face) => face.suit as u8,
    }
}

/// Strength within the kind, 0 for cards that never win. Trumps follow
/// [`rules::trump_order_key`] (a "Bube ehrenhalber" half a step below the
/// printed card of equal strength), so equal values really tie.
pub(crate) fn card_strength<C: PlayingCard>(game: GameType, card: C) -> u8 {
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

    /// The Augen of `cards` with their sign.
    fn signed_sum<C: PlayingCard>(cards: &[C]) -> i32 {
        i32::try_from(cards.iter().map(|c| c.signed_points()).sum::<i64>()).expect("small")
    }

    /// Plain minimax over the rules of [`crate::rules`], no pruning.
    fn reference<C: PlayingCard>(
        game: GameType,
        goal: Goal,
        hands: &mut [Vec<C>; 3],
        lead: Seat,
        trick: &mut Vec<C>,
        skat_points: i32,
    ) -> i32 {
        let seat = seat_at(lead, trick.len());
        let legal = rules::legal_moves(game, &hands[seat.index()], rules::effective_lead(trick));
        if legal.is_empty() {
            return 0;
        }
        let maximize = match goal {
            Goal::DeclarerPoints => seat.is_declarer(),
            Goal::NullDeclarerTrick | Goal::DefenderTrick => !seat.is_declarer(),
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
                let points = signed_sum(&cards);
                let saved = std::mem::take(trick);
                let last = hands[winner.index()].is_empty();
                let v = match goal {
                    Goal::NullDeclarerTrick if winner.is_declarer() => 1,
                    Goal::NullDeclarerTrick => {
                        reference(game, goal, hands, winner, trick, skat_points)
                    }
                    Goal::DefenderTrick if !winner.is_declarer() => 1,
                    Goal::DefenderTrick => reference(game, goal, hands, winner, trick, skat_points),
                    Goal::DeclarerPoints => {
                        let gained = if winner.is_declarer() { points } else { 0 };
                        gained + reference(game, goal, hands, winner, trick, skat_points)
                    }
                    Goal::RamschAvoid(me) => {
                        let mut gained = 0;
                        if winner != me {
                            gained = points + if last { skat_points } else { 0 };
                        }
                        gained + reference(game, goal, hands, winner, trick, skat_points)
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
                // The seat plays `card` first, the reference the rest.
                let mut hands = layout.hands.clone();
                let value = reference_first(game, goal, &mut hands, layout, card);
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
        let points = signed_sum(&cards);
        let last = hands[winner.index()].is_empty();
        let mut empty = Vec::new();
        let rest = |hands: &mut [Vec<C>; 3], empty: &mut Vec<C>| {
            reference(game, goal, hands, winner, empty, layout.skat_points)
        };
        match goal {
            Goal::NullDeclarerTrick if winner.is_declarer() => 1,
            Goal::NullDeclarerTrick => rest(hands, &mut empty),
            Goal::DefenderTrick if !winner.is_declarer() => 1,
            Goal::DefenderTrick => rest(hands, &mut empty),
            Goal::DeclarerPoints => {
                let gained = if winner.is_declarer() { points } else { 0 };
                gained + rest(hands, &mut empty)
            }
            Goal::RamschAvoid(me) => {
                let gained = if winner == me {
                    0
                } else {
                    points + if last { layout.skat_points } else { 0 }
                };
                gained + rest(hands, &mut empty)
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
        let skat: Vec<C> = it.take(2).collect();
        let skat_points = signed_sum(&skat);
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
                    Goal::DefenderTrick,
                ],
            };
            for goal in goals {
                let fast = solve(game, goal, &layout);
                let slow = reference_moves(game, goal, &layout);
                assert_eq!(fast, slow, "round {round} {game} {goal:?} {layout:?}");
            }
        }
    }

    /// A roguelike deck: blanks, a rank bonus, a card that never wins, one
    /// with extra Augen and some below zero ("Ramponiert").
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
                // Below zero (a Seven, an Eight, a Unter), a Zehn with
                // fewer Augen.
                if card == c("C7") || card == c("D8") || card == c("SJ") {
                    dealt.card_points = 0;
                    dealt.point_debt = 5;
                }
                if card == c("HT") {
                    dealt.card_points = 5;
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
            let goals = if game == GameType::Null {
                vec![Goal::NullDeclarerTrick]
            } else {
                vec![
                    Goal::DeclarerPoints,
                    Goal::RamschAvoid(Seat::ALL[round % 3]),
                    Goal::DefenderTrick,
                ]
            };
            for goal in goals {
                let fast = solve(game, goal, &layout);
                let slow = reference_moves(game, goal, &layout);
                assert_eq!(fast, slow, "round {round} {game} {goal:?} {layout:?}");
            }
        }
    }

    /// Many cards below zero ("Ramponiert" with 1 to 9 Augen of debt on
    /// every Seven, Eight, Nine and Dame, the Unter partly): the bounds of
    /// the search are no longer 0 and the Augen left.
    fn battered_deck() -> Vec<DealtCard> {
        standard_deck()
            .into_iter()
            .zip(0u32..)
            .map(|(card, id)| {
                let mut dealt = DealtCard::plain(id, CardKind::Normal(card));
                let low = matches!(
                    card.rank,
                    crate::cards::Rank::Seven
                        | crate::cards::Rank::Eight
                        | crate::cards::Rank::Nine
                        | crate::cards::Rank::Queen
                ) || (card.rank == crate::cards::Rank::Jack && id % 2 == 0);
                if low {
                    dealt.card_points = 0;
                    dealt.point_debt = 1 + id % 9;
                }
                dealt
            })
            .collect()
    }

    #[test]
    fn matches_plain_minimax_with_augen_below_zero() {
        let mut rng = Rng::from_seed(14);
        for round in 0..240 {
            let game = games()[round % 2];
            let layout = random_layout(&mut rng, battered_deck(), 2 + round % 3, (round / 2) % 3);
            for goal in [
                Goal::DeclarerPoints,
                Goal::RamschAvoid(Seat::ALL[round % 3]),
            ] {
                let fast = solve(game, goal, &layout);
                let slow = reference_moves(game, goal, &layout);
                assert_eq!(fast, slow, "round {round} {game} {goal:?} {layout:?}");
                // The null window gives the same answer as the exact value
                // for thresholds around and below zero.
                for threshold in [-20, -7, -1, 0, 1, 5, 12, 30] {
                    let mut table = Table::new(10);
                    let (probed, _) = probe_with(
                        game,
                        goal,
                        &layout,
                        threshold,
                        Limits::default(),
                        &mut table,
                    );
                    let expected: Vec<(DealtCard, bool)> =
                        slow.iter().map(|&(c, v)| (c, v >= threshold)).collect();
                    assert_eq!(
                        probed, expected,
                        "round {round} {goal:?} threshold {threshold}"
                    );
                }
            }
        }
    }

    #[test]
    fn every_root_move_gets_the_whole_node_limit() {
        // Depth 3 as PIMC searches, whole deals, a defender to lead: a
        // node limit of 6000 per move is not reached on these deals, so
        // the limited probes equal the unlimited ones. With one limit for
        // all moves, the moves late in the legal order were cut short and
        // some answered wrongly.
        let mut rng = Rng::from_seed(9);
        let mut table = Table::new(15);
        for round in 0..150 {
            let mut deck = standard_deck();
            rng.shuffle(&mut deck);
            let lead = Seat::ALL[1 + round % 2];
            let mut hands: [Vec<Card>; 3] = [Vec::new(), Vec::new(), Vec::new()];
            for o in 0..3 {
                let seat = seat_at(lead, o);
                hands[seat.index()] = deck[o * 10..(o + 1) * 10].to_vec();
                hands[seat.index()].sort();
            }
            let live: u32 = deck[..30].iter().map(|c| c.card_points()).sum();
            let layout = Layout {
                hands,
                trick_lead: lead,
                trick: Vec::new(),
                skat_points: 0,
            };
            let game = GameType::Suit(Suit::ALL[round % 4]);
            let threshold = i32::try_from(live / 2).expect("small");
            let limited = Limits {
                depth: 3,
                nodes: 6000,
            };
            let free = Limits { depth: 3, nodes: 0 };
            let goal = Goal::DeclarerPoints;
            let (a, _) = probe_with(game, goal, &layout, threshold, limited, &mut table);
            let (b, _) = probe_with(game, goal, &layout, threshold, free, &mut table);
            assert_eq!(a, b, "round {round}");
        }
    }

    #[test]
    fn equivalent_cards_merge_with_a_different_card_between_them() {
        // Grand, the declarer leads C9, C8 with 5 extra Augen, C7. C9 and
        // C7 are interchangeable (no club of another seat between them),
        // the C8 between them is not: two moves, not three.
        let dealt = |s: &str, id: u32| DealtCard::plain(id, CardKind::Normal(c(s)));
        let mut c8 = dealt("C8", 1);
        c8.card_points = 5;
        let hands = [
            vec![dealt("C9", 0), c8, dealt("C7", 2)],
            vec![dealt("SA", 3), dealt("SK", 4), dealt("SQ", 5)],
            vec![dealt("HA", 6), dealt("HK", 7), dealt("HQ", 8)],
        ];
        let layout = Layout {
            hands,
            trick_lead: Seat::DECLARER,
            trick: Vec::new(),
            skat_points: 0,
        };
        let mut table = Table::new(4);
        let (search, cards) = Search::new(
            GameType::Grand,
            Goal::DeclarerPoints,
            &layout,
            Limits::default(),
            &mut table,
        )
        .expect("valid layout");
        let (moves, n) = search.ordered_moves(0, NO_MOVE);
        let mut kept: Vec<DealtCard> = moves[..n].iter().map(|&m| cards[usize::from(m)]).collect();
        kept.sort();
        assert_eq!(kept.len(), 2, "{kept:?}");
        assert!(kept.contains(&c8));
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

    /// How the node limit of PIMC's probes errs against the same probe
    /// without it, by the position of the move in the legal order.
    ///
    /// `cargo test --release -p skat_core bench_node_limit -- --ignored --nocapture`
    /// (`BENCH_DEPTH` sets the depth, default 0 = to the end; `BENCH_NODES`
    /// the node limit, default 6000).
    #[test]
    #[ignore = "measurement"]
    fn bench_node_limit() {
        let env = |name: &str, default: u64| {
            std::env::var(name)
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(default)
        };
        let depth = u32::try_from(env("BENCH_DEPTH", 0)).unwrap_or(0);
        let node_limit = env("BENCH_NODES", 6000);
        let mut rng = Rng::from_seed(9);
        let mut table = Table::new(15);
        for tricks in [10usize, 8, 6] {
            let positions = 300;
            // Per move index in the legal order: probes, wrong answers.
            let mut by_index = [(0u64, 0u64); 10];
            let (mut nodes_total, mut nodes_max, mut wrong_choice) = (0u64, 0u64, 0u64);
            let (mut exact_nodes, mut micros) = (0u64, 0u128);
            for round in 0..positions {
                let mut deck = standard_deck();
                rng.shuffle(&mut deck);
                let lead = Seat::ALL[1 + round % 2];
                let mut hands: [Vec<_>; 3] = [Vec::new(), Vec::new(), Vec::new()];
                for o in 0..3 {
                    let seat = seat_at(lead, o);
                    hands[seat.index()] = deck[o * tricks..(o + 1) * tricks].to_vec();
                    hands[seat.index()].sort();
                }
                let live: u32 = deck[..3 * tricks].iter().map(|c| c.card_points()).sum();
                let layout = Layout {
                    hands,
                    trick_lead: lead,
                    trick: Vec::new(),
                    skat_points: 0,
                };
                let game = GameType::Suit(crate::cards::Suit::ALL[round % 4]);
                let threshold = i32::try_from(live / 2).unwrap_or(0);
                let exact = Limits { depth, nodes: 0 };
                let (truth, n) = probe_with(
                    game,
                    Goal::DeclarerPoints,
                    &layout,
                    threshold,
                    exact,
                    &mut table,
                );
                exact_nodes += n;
                let limited = Limits {
                    depth,
                    nodes: node_limit,
                };
                let start = std::time::Instant::now();
                let (got, n) = probe_with(
                    game,
                    Goal::DeclarerPoints,
                    &layout,
                    threshold,
                    limited,
                    &mut table,
                );
                micros += start.elapsed().as_micros();
                nodes_total += n;
                nodes_max = nodes_max.max(n);
                for (k, (t, g)) in truth.iter().zip(&got).enumerate() {
                    by_index[k.min(9)].0 += 1;
                    by_index[k.min(9)].1 += u64::from(t.1 != g.1);
                }
                // The defenders (seat to play) want a move that keeps the
                // declarer below the threshold: the first such move.
                let pick = got.iter().position(|&(_, r)| !r).unwrap_or(0);
                let can = truth.iter().any(|&(_, r)| !r);
                wrong_choice += u64::from(can && truth[pick].1);
            }
            let p = positions as u64;
            println!(
                "tricks {tricks} depth {depth}: nodes per probe mean {} max {} (exact mean {}), \
                 {:.3} ms per probe, wrong choice {:.1}%",
                nodes_total / p,
                nodes_max,
                exact_nodes / p,
                micros as f64 / 1000.0 / p as f64,
                100.0 * wrong_choice as f64 / p as f64,
            );
            let row: Vec<String> = by_index
                .iter()
                .enumerate()
                .filter(|(_, (n, _))| *n > 0)
                .map(|(k, (n, w))| format!("{}:{:.1}%", k + 1, 100.0 * *w as f64 / *n as f64))
                .collect();
            println!("  wrong answers by move index: {}", row.join(" "));
        }
    }

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

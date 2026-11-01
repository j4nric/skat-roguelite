//! Heuristic AI (design document, section 10.2, stage 1).
//!
//! Two policies: the two defenders play a coordinated priority list, the
//! declarer heuristic exists for the simulator, the run bot and for boss
//! tests. Both see only what [`SeatView`] exposes: own hand, the cards on
//! the table, the cards played so far, in an Ouvert game the declarer's
//! open hand and, against the boss "Die Zwillinge", the partner's hand.
//!
//! Both also play Null (Nullspieler, ADR 0003): the declarer ducks under
//! the winning card and sheds his most exposed cards, the defenders lead
//! low and keep the declarer on top. In a Ramsch (boss "Der Ramscher")
//! every seat plays for itself and avoids Augen.
//!
//! Both policies are generic over the card type, so they play the plain
//! 32-card deck as well as the roguelike deck with blanks and modified
//! cards. Blanks are worth nothing and never win, so they are the first
//! cards to slough.
//!
//! Stage 2 (Perfect Information Monte Carlo) lives in [`pimc`], built on
//! the double-dummy [`solver`] and the consistent deals of [`sample`].

pub mod pimc;
pub mod sample;
pub mod solver;

use crate::cards::{Card, PlayingCard, Rank, Suit};
use crate::game::{Game, GameError, Seat, SeatView, seat_at};
use crate::rules::{self, Declaration, GameType, RiskLevel};

/// A card-choosing strategy for one seat.
pub trait Policy<C: PlayingCard = Card> {
    /// Choose a card from `view.legal_moves`. Returns `None` only if there
    /// is no legal move at all (not this seat's turn or empty hand).
    fn choose(&self, view: &SeatView<'_, C>) -> Option<C>;
}

// ---------------------------------------------------------------------------
// Shared helpers
// ---------------------------------------------------------------------------

/// Points first, then rank strength: the key for "the lowest card".
fn low_key<C: PlayingCard>(game: GameType, card: C) -> (u32, u8) {
    (card.card_points(), strength(game, card))
}

/// Comparable strength within the card's kind (trump or suit).
fn strength<C: PlayingCard>(game: GameType, card: C) -> u8 {
    rules::trump_strength(game, card).unwrap_or_else(|| rules::suit_strength(game, card))
}

/// Printed suit of a card, `None` for a blank.
fn suit_of<C: PlayingCard>(card: C) -> Option<Suit> {
    card.face().map(|f| f.suit)
}

/// Whether the card is a printed card of `rank`.
fn has_rank<C: PlayingCard>(card: C, rank: Rank) -> bool {
    card.face().is_some_and(|f| f.rank == rank)
}

fn lowest<C: PlayingCard>(game: GameType, cards: &[C]) -> Option<C> {
    cards.iter().copied().min_by_key(|&c| low_key(game, c))
}

/// Highest Augen, ties broken by the weaker card.
fn highest_points<C: PlayingCard>(game: GameType, cards: &[C]) -> Option<C> {
    cards
        .iter()
        .copied()
        .max_by_key(|&c| (c.card_points(), u8::MAX - strength(game, c)))
}

/// Weakest card by strength (used to "take cheaply").
fn weakest<C: PlayingCard>(game: GameType, cards: &[C]) -> Option<C> {
    cards.iter().copied().min_by_key(|&c| strength(game, c))
}

fn strongest<C: PlayingCard>(game: GameType, cards: &[C]) -> Option<C> {
    cards.iter().copied().max_by_key(|&c| strength(game, c))
}

/// Whether no unseen card of the same kind can beat `card`.
fn is_sure<C: PlayingCard>(game: GameType, unseen: &[C], card: C) -> bool {
    !unseen
        .iter()
        .any(|&u| rules::follows_lead(game, card, u) && rules::beats(game, card, card, u))
}

/// Number of unseen trumps.
fn unseen_trumps<C: PlayingCard>(game: GameType, unseen: &[C]) -> usize {
    unseen.iter().filter(|&&c| rules::is_trump(game, c)).count()
}

/// Cards of `hand` that are not trump, grouped by suit (deterministic order).
/// Blanks belong to no suit.
fn non_trump_suits<C: PlayingCard>(game: GameType, hand: &[C]) -> Vec<(Suit, Vec<C>)> {
    Suit::ALL
        .iter()
        .filter_map(|&suit| {
            let cards: Vec<C> = hand
                .iter()
                .copied()
                .filter(|&c| suit_of(c) == Some(suit) && !rules::is_trump(game, c))
                .collect();
            if cards.is_empty() {
                None
            } else {
                Some((suit, cards))
            }
        })
        .collect()
}

/// Suits (and trump) in which `seat` is known to be void because it failed
/// to follow a lead earlier in the game. Trump is reported as `None`.
/// Blanks may always be played, so playing one reveals nothing. If the
/// declarer plays Ouvert, his voids are read directly from the open hand.
fn known_voids<C: PlayingCard>(view: &SeatView<'_, C>, seat: Seat) -> Vec<Option<Suit>> {
    let game = view.declaration.game();
    if let (true, Some(open)) = (seat.is_declarer(), view.declarer_hand) {
        let kinds = Suit::ALL
            .iter()
            .map(|&s| Some(s))
            .chain(std::iter::once(None));
        return kinds
            .filter(|&kind| {
                !open.iter().any(|&c| match kind {
                    None => rules::is_trump(game, c),
                    Some(suit) => !rules::is_trump(game, c) && suit_of(c) == Some(suit),
                })
            })
            .collect();
    }
    let mut voids = Vec::new();
    let current = (view.trick_lead, view.trick);
    let completed = view.tricks.iter().map(|t| (t.lead, &t.cards[..]));
    for (lead_seat, cards) in completed.chain(std::iter::once(current)) {
        let Some(lead_offset) = cards.iter().position(|c| !c.is_blank()) else {
            continue;
        };
        let lead = cards[lead_offset];
        for (i, &card) in cards.iter().enumerate().skip(lead_offset + 1) {
            if seat_at(lead_seat, i) == seat
                && !card.is_blank()
                && !rules::follows_lead(game, lead, card)
            {
                let kind = if rules::is_trump(game, lead) {
                    None
                } else {
                    suit_of(lead)
                };
                if !voids.contains(&kind) {
                    voids.push(kind);
                }
            }
        }
    }
    voids
}

fn trick_points<C: PlayingCard>(trick: &[C]) -> u32 {
    trick.iter().map(|c| c.card_points()).sum()
}

/// Cards from `legal` that would currently win the trick.
fn beating<C: PlayingCard>(game: GameType, trick: &[C], legal: &[C]) -> Vec<C> {
    let Some(idx) = rules::winning_index(game, trick) else {
        return legal.to_vec();
    };
    let lead = rules::effective_lead(trick).unwrap_or(trick[0]);
    let holder = trick[idx];
    legal
        .iter()
        .copied()
        .filter(|&c| rules::beats(game, lead, holder, c))
        .collect()
}

/// Lowest card, preferring non-trumps (slough without wasting trump).
fn slough<C: PlayingCard>(game: GameType, legal: &[C]) -> Option<C> {
    let non_trump: Vec<C> = legal
        .iter()
        .copied()
        .filter(|&c| !rules::is_trump(game, c))
        .collect();
    if non_trump.is_empty() {
        lowest(game, legal)
    } else {
        lowest(game, &non_trump)
    }
}

/// Highest Augen, preferring non-trumps.
fn smear<C: PlayingCard>(game: GameType, legal: &[C]) -> Option<C> {
    let non_trump: Vec<C> = legal
        .iter()
        .copied()
        .filter(|&c| !rules::is_trump(game, c))
        .collect();
    if non_trump.is_empty() {
        highest_points(game, legal)
    } else {
        highest_points(game, &non_trump)
    }
}

/// Lead from a suit: the top card if it is sure, otherwise the lowest.
fn lead_from_suit<C: PlayingCard>(game: GameType, unseen: &[C], cards: &[C]) -> Option<C> {
    let top = strongest(game, cards)?;
    if is_sure(game, unseen, top) {
        Some(top)
    } else {
        lowest(game, cards)
    }
}

// ---------------------------------------------------------------------------
// Defenders
// ---------------------------------------------------------------------------

/// Coordinated defender heuristic (both defender seats use the same one).
#[derive(Debug, Clone, Copy, Default)]
pub struct DefenderHeuristic;

impl DefenderHeuristic {
    /// Simple hand evaluation for Kontra (house rule "Kontra"): a defender
    /// doubles when he holds at least four trumps of the declared game, or
    /// three trumps and at least three Volle (aces and tens) outside trump.
    #[must_use]
    pub fn says_kontra<C: PlayingCard>(&self, hand: &[C], game: GameType) -> bool {
        let trumps = trump_count(game, hand);
        let high_cards = hand
            .iter()
            .filter(|&&c| {
                !rules::is_trump(game, c) && (has_rank(c, Rank::Ace) || has_rank(c, Rank::Ten))
            })
            .count();
        if game == GameType::Null {
            return Self::says_null_kontra(hand);
        }
        trumps >= 4 || (trumps >= 3 && high_cards >= 3)
    }

    fn lead<C: PlayingCard>(&self, view: &SeatView<'_, C>, unseen: &[C]) -> Option<C> {
        let game = view.declaration.game();
        let legal = &view.legal_moves;
        let suits = non_trump_suits(game, legal);
        if suits.is_empty() {
            // Only trumps (and blanks) left: lead the top one.
            return strongest(game, legal);
        }
        let declarer_voids = known_voids(view, Seat::DECLARER);
        // Long suit in which the declarer is void: lead low, it costs nothing
        // and forces a trump or hands the trick to the partner.
        if let Some((_, cards)) = suits
            .iter()
            .filter(|(suit, _)| declarer_voids.contains(&Some(*suit)))
            .max_by_key(|(suit, cards)| (cards.len(), *suit))
        {
            return lowest(game, cards);
        }
        // Otherwise the shortest suit; prefer one where we hold the top card.
        let (_, cards) = suits.iter().min_by_key(|(suit, cards)| {
            let has_sure = strongest(game, cards).is_some_and(|c| is_sure(game, unseen, c));
            (cards.len(), !has_sure, *suit)
        })?;
        lead_from_suit(game, unseen, cards)
    }

    fn follow<C: PlayingCard>(&self, view: &SeatView<'_, C>, unseen: &[C]) -> Option<C> {
        let game = view.declaration.game();
        let legal = &view.legal_moves;
        let trick = view.trick;
        let winner_idx = rules::winning_index(game, trick)?;
        let winner_seat = seat_at(view.trick_lead, winner_idx);
        let holder = trick[winner_idx];
        let last = trick.len() == 2;
        let points = trick_points(trick);
        let lead = view.lead_card();
        let can_follow =
            lead.is_some_and(|lead| legal.iter().any(|&c| rules::follows_lead(game, lead, c)));
        let beating = beating(game, trick, legal);

        if !winner_seat.is_declarer() {
            // Partner holds the trick.
            let declarer_voids = known_voids(view, Seat::DECLARER);
            let declarer_can_ruff = lead.is_some_and(|lead| {
                !rules::is_trump(game, lead)
                    && declarer_voids.contains(&suit_of(lead))
                    && unseen_trumps(game, unseen) > 0
            });
            let partner_sure =
                last || (!holder.is_blank() && is_sure(game, unseen, holder) && !declarer_can_ruff);
            if partner_sure {
                return smear(game, legal);
            }
            // Partner may be overtaken by the declarer: win it ourselves with
            // a sure card if we have one, otherwise stay low.
            let sure: Vec<C> = beating
                .iter()
                .copied()
                .filter(|&c| is_sure(game, unseen, c))
                .collect();
            if let Some(c) = weakest(game, &sure) {
                return Some(c);
            }
            return slough(game, legal);
        }

        // Declarer holds the trick.
        if can_follow {
            if !beating.is_empty() {
                if last {
                    return weakest(game, &beating);
                }
                let sure: Vec<C> = beating
                    .iter()
                    .copied()
                    .filter(|&c| is_sure(game, unseen, c))
                    .collect();
                if let Some(c) = weakest(game, &sure) {
                    return Some(c);
                }
                if points >= 10 {
                    return weakest(game, &beating);
                }
            }
            return lowest(game, legal);
        }

        // Cannot follow: ruff if the trick is worth it, otherwise slough.
        if !beating.is_empty() && (points >= 10 || (last && points > 0)) {
            return weakest(game, &beating);
        }
        slough(game, legal)
    }
}

impl<C: PlayingCard> Policy<C> for DefenderHeuristic {
    fn choose(&self, view: &SeatView<'_, C>) -> Option<C> {
        if view.legal_moves.is_empty() {
            return None;
        }
        if view.declaration.is_ramsch() {
            return ramsch_play(view).or_else(|| view.legal_moves.first().copied());
        }
        if view.declaration.is_null() {
            return self
                .null_play(view)
                .or_else(|| view.legal_moves.first().copied());
        }
        let unseen = view.unseen_cards();
        let choice = if view.trick.is_empty() {
            self.lead(view, &unseen)
        } else {
            self.follow(view, &unseen)
        };
        choice.or_else(|| view.legal_moves.first().copied())
    }
}

// ---------------------------------------------------------------------------
// Declarer
// ---------------------------------------------------------------------------

/// Declarer heuristic: game choice, discard and play. Used by the simulator
/// and by boss tests; never chooses Null.
#[derive(Debug, Clone, Copy, Default)]
pub struct DeclarerHeuristic;

/// Trumps of `game` held in `hand`.
#[must_use]
pub fn trump_count<C: PlayingCard>(game: GameType, hand: &[C]) -> usize {
    hand.iter().filter(|&&c| rules::is_trump(game, c)).count()
}

impl DeclarerHeuristic {
    /// Choose the game for a ten-card hand.
    ///
    /// Grand with at least two jacks and two aces; otherwise the suit with
    /// the most trumps if that makes at least five; otherwise the game with
    /// the highest estimated value (base value × (matadors + 1) × trumps as
    /// a crude proxy for the chance to win). Hand only with eight or more
    /// trumps, risk otherwise `Normal`.
    #[must_use]
    pub fn choose_declaration<C: PlayingCard>(&self, hand: &[C]) -> Declaration {
        let jacks = hand.iter().filter(|&&c| has_rank(c, Rank::Jack)).count();
        let aces = hand.iter().filter(|&&c| has_rank(c, Rank::Ace)).count();
        let faces: Vec<Card> = hand.iter().filter_map(|c| c.face()).collect();
        let game = if jacks >= 2 && aces >= 2 {
            GameType::Grand
        } else {
            let suit_games = Suit::ALL.iter().map(|&s| GameType::Suit(s));
            // Ties go to the higher base value (later suit in Suit::ALL).
            let best_suit = suit_games
                .clone()
                .max_by_key(|&g| (trump_count(g, hand), g.base_value()))
                .unwrap_or(GameType::Suit(Suit::Clubs));
            if trump_count(best_suit, hand) >= 5 {
                best_suit
            } else {
                suit_games
                    .chain(std::iter::once(GameType::Grand))
                    .max_by_key(|&g| {
                        let base = g.base_value().unwrap_or(0);
                        let levels = rules::matadors(g, &faces).count + 1;
                        let trumps = u32::try_from(trump_count(g, hand)).unwrap_or(0);
                        (base * levels * trumps, base)
                    })
                    .unwrap_or(GameType::Grand)
            }
        };
        let risk = if trump_count(game, hand) >= 8 {
            RiskLevel::Hand
        } else {
            RiskLevel::Normal
        };
        // The candidates above are suit games and the grand only.
        Declaration::new(game, risk).unwrap_or(Declaration::grand(risk))
    }

    /// The game to declare after picking up the skat: the choice of
    /// [`DeclarerHeuristic::choose_declaration`] for all twelve cards, without
    /// hand levels (Risikostufe `Normal`).
    #[must_use]
    pub fn choose_game_after_skat<C: PlayingCard>(&self, cards: &[C]) -> Declaration {
        self.choose_declaration(cards).with_risk(RiskLevel::Normal)
    }

    /// Play the Ansage of a dealt game in the order of the Skatordnung (ADR
    /// 0011), for the simulator and tests: a hand game if
    /// [`DeclarerHeuristic::choose_declaration`] says so for the ten cards;
    /// otherwise pick up the skat, choose the game for all twelve cards
    /// ([`DeclarerHeuristic::choose_game_after_skat`]), discard for it and
    /// declare it. Returns the declaration.
    pub fn declare<C: PlayingCard>(&self, game: &mut Game<C>) -> Result<Declaration, GameError<C>> {
        let first = self.choose_declaration(game.hand(Seat::DECLARER));
        if first.hand() {
            game.declare(first)?;
            return Ok(first);
        }
        game.pick_up_skat()?;
        let declaration = self.choose_game_after_skat(game.hand(Seat::DECLARER));
        self.discard_and_declare(game, declaration)?;
        Ok(declaration)
    }

    /// Declare `declaration` for a dealt game: a hand game (or the Ramsch)
    /// at once; any other game after picking up the skat and discarding with
    /// [`DeclarerHeuristic::choose_discard`] (the most exposed cards in a
    /// Null, [`DeclarerHeuristic::choose_null_discard`]). For tests and
    /// measurements that fix the game.
    pub fn declare_as<C: PlayingCard>(
        &self,
        game: &mut Game<C>,
        declaration: Declaration,
    ) -> Result<(), GameError<C>> {
        if declaration.hand() {
            game.declare(declaration)?;
            return Ok(());
        }
        game.pick_up_skat()?;
        self.discard_and_declare(game, declaration)
    }

    /// Discard for `declaration` from the twelve cards, then declare it.
    fn discard_and_declare<C: PlayingCard>(
        &self,
        game: &mut Game<C>,
        declaration: Declaration,
    ) -> Result<(), GameError<C>> {
        let cards = game.hand(Seat::DECLARER).to_vec();
        let discard = if declaration.is_null() {
            self.choose_null_discard(&cards, game.deck_cards())
        } else {
            self.choose_discard(&cards, declaration.game())
        };
        let Some(discard) = discard else {
            return Err(GameError::WrongPhase {
                phase: game.phase(),
            });
        };
        game.discard(discard)?;
        game.declare(declaration)?;
        Ok(())
    }

    /// Choose the two discards from the twelve cards. Blanks go first (they
    /// are worth nothing and never win); then never aces or trumps if
    /// avoidable, preferring to clear short non-trump suits and cards with
    /// few Augen.
    #[must_use]
    pub fn choose_discard<C: PlayingCard>(&self, cards: &[C], game: GameType) -> Option<[C; 2]> {
        let mut chosen: Vec<C> = cards
            .iter()
            .copied()
            .filter(|c| c.is_blank())
            .take(2)
            .collect();
        let suits = non_trump_suits(game, cards);
        let mut candidates: Vec<(usize, u32, u8, C)> = Vec::new();
        for (_, suit_cards) in &suits {
            for &c in suit_cards {
                if !has_rank(c, Rank::Ace) {
                    candidates.push((suit_cards.len(), c.card_points(), strength(game, c), c));
                }
            }
        }
        candidates.sort();
        let missing = 2 - chosen.len();
        chosen.extend(candidates.iter().map(|&(_, _, _, c)| c).take(missing));
        if chosen.len() < 2 {
            // Fallback: lowest remaining cards of any kind.
            let mut rest: Vec<C> = cards
                .iter()
                .copied()
                .filter(|c| !chosen.contains(c))
                .collect();
            rest.sort_by_key(|&c| (rules::is_trump(game, c), low_key(game, c)));
            let missing = 2 - chosen.len();
            chosen.extend(rest.into_iter().take(missing));
        }
        match chosen[..] {
            [a, b] => Some([a, b]),
            _ => None,
        }
    }

    fn lead<C: PlayingCard>(&self, view: &SeatView<'_, C>, unseen: &[C]) -> Option<C> {
        let game = view.declaration.game();
        let legal = &view.legal_moves;
        let trumps: Vec<C> = legal
            .iter()
            .copied()
            .filter(|&c| rules::is_trump(game, c))
            .collect();
        // Draw trumps while the defenders may still hold some.
        if !trumps.is_empty() && unseen_trumps(game, unseen) > 0 {
            let top = strongest(game, &trumps)?;
            return if is_sure(game, unseen, top) {
                Some(top)
            } else {
                weakest(game, &trumps)
            };
        }
        let suits = non_trump_suits(game, legal);
        // Sure winners first (aces, bare tens once the ace is gone).
        if let Some(c) = suits
            .iter()
            .filter_map(|(_, cards)| strongest(game, cards))
            .filter(|&c| is_sure(game, unseen, c))
            .max_by_key(|&c| c.card_points())
        {
            return Some(c);
        }
        // Then the longest suit, low.
        if let Some((_, cards)) = suits
            .iter()
            .max_by_key(|(suit, cards)| (cards.len(), *suit))
        {
            return lowest(game, cards);
        }
        strongest(game, legal)
    }

    fn follow<C: PlayingCard>(&self, view: &SeatView<'_, C>, unseen: &[C]) -> Option<C> {
        let game = view.declaration.game();
        let legal = &view.legal_moves;
        let trick = view.trick;
        if trick.is_empty() {
            return None;
        }
        let last = trick.len() == 2;
        let points = trick_points(trick);
        let can_follow = view
            .lead_card()
            .is_some_and(|lead| legal.iter().any(|&c| rules::follows_lead(game, lead, c)));
        let beating = beating(game, trick, legal);
        if !beating.is_empty() {
            if last {
                return weakest(game, &beating);
            }
            let sure: Vec<C> = beating
                .iter()
                .copied()
                .filter(|&c| is_sure(game, unseen, c))
                .collect();
            if let Some(c) = weakest(game, &sure) {
                return Some(c);
            }
            if points >= 10 || can_follow {
                return weakest(game, &beating);
            }
        }
        if can_follow {
            lowest(game, legal)
        } else {
            slough(game, legal)
        }
    }
}

impl<C: PlayingCard> Policy<C> for DeclarerHeuristic {
    fn choose(&self, view: &SeatView<'_, C>) -> Option<C> {
        if view.legal_moves.is_empty() {
            return None;
        }
        if view.declaration.is_ramsch() {
            return ramsch_play(view).or_else(|| view.legal_moves.first().copied());
        }
        let unseen = view.unseen_cards();
        if view.declaration.is_null() {
            return self
                .null_play(view, &unseen)
                .or_else(|| view.legal_moves.first().copied());
        }
        let choice = if view.trick.is_empty() {
            self.lead(view, &unseen)
        } else {
            self.follow(view, &unseen)
        };
        choice.or_else(|| view.legal_moves.first().copied())
    }
}

// ---------------------------------------------------------------------------
// Null (Nullspieler) and Ramsch (boss "Der Ramscher")
// ---------------------------------------------------------------------------

/// Strength of a card in a Null game for "who goes under whom": a blank or
/// a card that never wins counts as the lowest of all.
fn null_strength<C: PlayingCard>(card: C) -> u8 {
    if card.is_blank() || card.never_wins() {
        0
    } else {
        rules::suit_strength(GameType::Null, card)
    }
}

/// How exposed a card of the Null declarer is: the unseen cards of its
/// suit that are lower than it, minus the own cards of the suit that are
/// lower (the classic Null rule: the k-th lowest card of a suit is safe if
/// at most k-1 lower cards are out). Blanks and cards that never win are
/// never exposed.
#[must_use]
pub fn null_exposure<C: PlayingCard>(card: C, own: &[C], unseen: &[C]) -> i32 {
    let Some(face) = card.face() else {
        return i32::MIN;
    };
    if card.never_wins() {
        return i32::MIN;
    }
    let strength = null_strength(card);
    let lower = |cards: &[C]| {
        cards
            .iter()
            .filter(|c| {
                c.face().is_some_and(|f| f.suit == face.suit)
                    && !c.never_wins()
                    && null_strength(**c) < strength
            })
            .count()
    };
    let out = i32::try_from(lower(unseen)).unwrap_or(i32::MAX);
    let mine = i32::try_from(lower(own)).unwrap_or(i32::MAX);
    out - mine
}

/// Unsafe cards of a Null hand against every other card of the deck:
/// cards with a positive [`null_exposure`].
#[must_use]
pub fn null_unsafe_cards<C: PlayingCard>(hand: &[C], deck: &[C]) -> usize {
    let unseen: Vec<C> = deck.iter().copied().filter(|c| !hand.contains(c)).collect();
    hand.iter()
        .filter(|&&c| null_exposure(c, hand, &unseen) > 0)
        .count()
}

/// Card of `cards` that does not take the trick so far, highest first
/// (a safe card to get rid of), if any.
fn highest_under<C: PlayingCard>(game: GameType, trick: &[C], cards: &[C]) -> Option<C> {
    let idx = rules::winning_index(game, trick)?;
    let lead = rules::effective_lead(trick).unwrap_or(trick[0]);
    let holder = trick[idx];
    cards
        .iter()
        .copied()
        .filter(|&c| !rules::beats(game, lead, holder, c))
        .max_by_key(|&c| (null_strength(c), c.card_points()))
}

impl DeclarerHeuristic {
    /// Null: discard the two most exposed cards of the twelve.
    #[must_use]
    pub fn choose_null_discard<C: PlayingCard>(&self, cards: &[C], deck: &[C]) -> Option<[C; 2]> {
        let mut hand = cards.to_vec();
        let mut chosen = Vec::new();
        for _ in 0..2 {
            let unseen: Vec<C> = deck
                .iter()
                .copied()
                .filter(|c| !cards.contains(c))
                .collect();
            let worst = hand
                .iter()
                .copied()
                .max_by_key(|&c| (null_exposure(c, &hand, &unseen), null_strength(c)))?;
            hand.retain(|&c| c != worst);
            chosen.push(worst);
        }
        match chosen[..] {
            [a, b] => Some([a, b]),
            _ => None,
        }
    }

    fn null_play<C: PlayingCard>(&self, view: &SeatView<'_, C>, unseen: &[C]) -> Option<C> {
        let game = GameType::Null;
        let legal = &view.legal_moves;
        let trick = view.trick;
        if trick.is_empty() {
            // Lead the card the defenders can least go under: few unseen
            // cards of its suit at or below it, many above.
            return legal.iter().copied().min_by_key(|&c| {
                if c.is_blank() || c.never_wins() {
                    return (i32::MIN, 0, 0);
                }
                let suit = c.face().map(|f| f.suit);
                let same: Vec<&C> = unseen
                    .iter()
                    .filter(|u| u.face().map(|f| f.suit) == suit && !u.never_wins())
                    .collect();
                let under = same
                    .iter()
                    .filter(|u| null_strength(***u) <= null_strength(c))
                    .count();
                let over = same.len() - under;
                (
                    i32::try_from(under).unwrap_or(i32::MAX),
                    -i32::try_from(over).unwrap_or(0),
                    null_strength(c),
                )
            });
        }
        let lead = view.lead_card();
        let can_follow =
            lead.is_some_and(|lead| legal.iter().any(|&c| rules::follows_lead(game, lead, c)));
        if can_follow {
            if let Some(c) = highest_under(game, trick, legal) {
                return Some(c);
            }
            // Every card takes the trick so far.
            let last = trick.len() == 2;
            return if last {
                legal.iter().copied().max_by_key(|&c| null_strength(c))
            } else {
                legal.iter().copied().min_by_key(|&c| null_strength(c))
            };
        }
        // Cannot follow: get rid of the most exposed card.
        legal
            .iter()
            .copied()
            .max_by_key(|&c| (null_exposure(c, view.hand, unseen), null_strength(c)))
    }
}

impl DefenderHeuristic {
    /// Null: a defender doubles with many low cards (sevens and eights),
    /// the cards that force the declarer to take a trick.
    fn says_null_kontra<C: PlayingCard>(hand: &[C]) -> bool {
        let low = hand
            .iter()
            .filter(|&&c| has_rank(c, Rank::Seven) || has_rank(c, Rank::Eight))
            .count();
        low >= 5
    }

    /// Null defence: both defenders want the declarer to take a trick.
    fn null_play<C: PlayingCard>(&self, view: &SeatView<'_, C>) -> Option<C> {
        let game = GameType::Null;
        let legal = &view.legal_moves;
        let trick = view.trick;
        if trick.is_empty() {
            let voids = known_voids(view, Seat::DECLARER);
            let candidates: Vec<C> = legal
                .iter()
                .copied()
                .filter(|c| c.face().is_some_and(|f| !voids.contains(&Some(f.suit))))
                .collect();
            let pool = if candidates.is_empty() {
                legal.clone()
            } else {
                candidates
            };
            // With the declarer's hand open: a suit where all his cards are
            // above our lowest one forces him to take the trick.
            if let Some(open) = view.declarer_hand {
                let forcing = pool.iter().copied().filter(|&c| {
                    let suit = c.face().map(|f| f.suit);
                    let his: Vec<&C> = open
                        .iter()
                        .filter(|d| d.face().map(|f| f.suit) == suit)
                        .collect();
                    !his.is_empty()
                        && his
                            .iter()
                            .all(|d| null_strength(**d) > null_strength(c) && !d.never_wins())
                });
                if let Some(c) = forcing.min_by_key(|&c| null_strength(c)) {
                    return Some(c);
                }
            }
            return pool.iter().copied().min_by_key(|&c| null_strength(c));
        }
        let lead = view.lead_card();
        let can_follow =
            lead.is_some_and(|lead| legal.iter().any(|&c| rules::follows_lead(game, lead, c)));
        let declarer_played = (0..trick.len()).any(|i| seat_at(view.trick_lead, i).is_declarer());
        let dump = || legal.iter().copied().max_by_key(|&c| null_strength(c));
        if !can_follow {
            return dump();
        }
        if declarer_played {
            let idx = rules::winning_index(game, trick)?;
            if seat_at(view.trick_lead, idx).is_declarer() {
                // Keep him on top: our highest card below his.
                return highest_under(game, trick, legal).or_else(dump);
            }
            return dump();
        }
        // The declarer plays after us: keep the bar low.
        legal.iter().copied().min_by_key(|&c| null_strength(c))
    }
}

/// Ramsch: every seat plays for itself and avoids Augen and tricks. Used
/// for all three seats by both heuristics.
fn ramsch_play<C: PlayingCard>(view: &SeatView<'_, C>) -> Option<C> {
    let game = GameType::Grand;
    let legal = &view.legal_moves;
    let trick = view.trick;
    let key = |c: C| (c.card_points(), strength(game, c));
    if trick.is_empty() {
        let non_trump: Vec<C> = legal
            .iter()
            .copied()
            .filter(|&c| !rules::is_trump(game, c))
            .collect();
        let pool = if non_trump.is_empty() {
            legal.clone()
        } else {
            non_trump
        };
        return pool.iter().copied().min_by_key(|&c| key(c));
    }
    let idx = rules::winning_index(game, trick)?;
    let lead = rules::effective_lead(trick).unwrap_or(trick[0]);
    let holder = trick[idx];
    let under: Vec<C> = legal
        .iter()
        .copied()
        .filter(|&c| !rules::beats(game, lead, holder, c))
        .collect();
    if !under.is_empty() {
        // Safe: dump the most Augen (the trick goes to someone else).
        return under.iter().copied().max_by_key(|&c| key(c));
    }
    // Every card takes the trick: as cheaply as possible.
    legal.iter().copied().min_by_key(|&c| key(c))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cards::parse_cards;
    use crate::game::Game;
    use crate::rng::Rng;

    fn c(s: &str) -> Card {
        s.parse().expect("test card")
    }

    #[test]
    fn declaration_grand_with_two_jacks_and_two_aces() {
        let hand = parse_cards("CJ SJ CA SA HT HK D7 D8 S9 C7").expect("hand");
        let d = DeclarerHeuristic.choose_declaration(&hand);
        assert_eq!(d.game(), GameType::Grand);
        assert_eq!(d.risk(), Some(RiskLevel::Normal));
    }

    #[test]
    fn declaration_longest_suit_and_hand_with_eight_trumps() {
        let hand = parse_cards("CJ HJ HA HT HK HQ H9 H8 D7 S9").expect("hand");
        let d = DeclarerHeuristic.choose_declaration(&hand);
        assert_eq!(d.game(), GameType::Suit(Suit::Hearts));
        assert_eq!(d.risk(), Some(RiskLevel::Hand));
        let hand = parse_cards("CJ HA HT HK HQ D7 D8 S9 C7 C8").expect("hand");
        let d = DeclarerHeuristic.choose_declaration(&hand);
        assert_eq!(d.game(), GameType::Suit(Suit::Hearts));
        assert_eq!(d.risk(), Some(RiskLevel::Normal));
    }

    #[test]
    fn declaration_never_null() {
        let mut rng = Rng::from_seed(3);
        for _ in 0..200 {
            let game = Game::deal(&mut rng, Seat::DECLARER);
            let d = DeclarerHeuristic.choose_declaration(game.hand(Seat::DECLARER));
            assert_ne!(d.game(), GameType::Null);
        }
    }

    #[test]
    fn discard_avoids_aces_and_trumps_and_clears_short_suits() {
        let cards = parse_cards("CJ SJ HA HT HK HQ SA ST DT C7 D8 S9").expect("cards");
        let game = GameType::Suit(Suit::Hearts);
        let discard = DeclarerHeuristic
            .choose_discard(&cards, game)
            .expect("two cards");
        // C7 is a singleton, D8 and DT form the two-card diamond suit.
        assert_eq!(discard, [c("C7"), c("D8")]);
        let all_trumps = parse_cards("CJ SJ HJ DJ HA HT HK HQ H9 H8 H7 SA").expect("cards");
        let discard = DeclarerHeuristic
            .choose_discard(&all_trumps, game)
            .expect("two cards");
        assert_eq!(discard, [c("SA"), c("H7")]);
    }

    fn play_out(seed: u64) -> Game {
        let mut rng = Rng::from_seed(seed);
        let mut game = Game::deal(&mut rng, Seat::DECLARER);
        let declarer = DeclarerHeuristic;
        let defender = DefenderHeuristic;
        declarer.declare(&mut game).expect("declare");
        while let Some(seat) = game.current_seat() {
            let view = game.view(seat).expect("playing");
            let card = if seat.is_declarer() {
                declarer.choose(&view)
            } else {
                defender.choose(&view)
            }
            .expect("a legal move");
            game.play(seat, card).expect("legal move");
        }
        game
    }

    #[test]
    fn heuristics_always_produce_legal_moves() {
        for seed in 0..50 {
            let game = play_out(seed);
            assert_eq!(game.tricks().len(), 10);
        }
    }

    #[test]
    fn declarer_ruffs_cheaply_as_last_player() {
        // Seat 1 leads SA, seat 2 smears the HT, the declarer is last.
        let hands = [
            parse_cards("CJ SJ HJ DJ CA CT CK CQ C9 C8").expect("hand"),
            parse_cards("SA ST SK SQ S9 S8 S7 D7 D8 D9").expect("hand"),
            parse_cards("HA HT HK HQ H9 H8 H7 DA DT DK").expect("hand"),
        ];
        let mut game = Game::with_cards(hands, [c("C7"), c("DQ")], Seat::ALL[1]).expect("deal");
        game.declare(Declaration::suit(Suit::Clubs, RiskLevel::Hand))
            .expect("declare");
        game.play(Seat::ALL[1], c("SA")).expect("lead");
        game.play(Seat::ALL[2], c("HT")).expect("smear");
        let view = game.view(Seat::DECLARER).expect("view");
        // Declarer follows: cannot follow spades, trick worth 21 -> ruff cheaply.
        assert_eq!(DeclarerHeuristic.choose(&view), Some(c("C8")));
    }

    #[test]
    fn defender_sloughs_low_when_declarer_holds() {
        let hands = [
            parse_cards("CJ SJ HJ DJ CA CT CK CQ C9 C8").expect("hand"),
            parse_cards("SA ST SK SQ S9 S8 S7 D7 D8 D9").expect("hand"),
            parse_cards("HA HT HK HQ H9 H8 H7 DA DT DK").expect("hand"),
        ];
        let mut game = Game::with_cards(hands, [c("C7"), c("DQ")], Seat::DECLARER).expect("deal");
        game.declare(Declaration::suit(Suit::Clubs, RiskLevel::Hand))
            .expect("declare");
        game.play(Seat::DECLARER, c("CJ")).expect("lead");
        let view = game.view(Seat::ALL[1]).expect("view");
        assert_eq!(DefenderHeuristic.choose(&view), Some(c("D7")));
    }

    /// A deck with blanks and a marked card, as instances.
    fn roguelike_deck() -> Vec<crate::deck::DealtCard> {
        use crate::cards::{CardKind, standard_deck};
        use crate::deck::DealtCard;
        standard_deck()
            .into_iter()
            .zip(0u32..)
            .map(|(card, id)| {
                // Remove all sevens (blanks) and mark the spade king.
                let kind = if card.rank == Rank::Seven {
                    CardKind::Blank
                } else {
                    CardKind::Normal(card)
                };
                let mut dealt = DealtCard::plain(id, kind);
                if card == c("SK") {
                    dealt.rank_bonus = 1;
                }
                dealt
            })
            .collect()
    }

    #[test]
    fn heuristics_play_legal_games_with_blanks_and_ouvert() {
        for seed in 0..60 {
            let mut rng = Rng::from_seed(seed);
            let mut game =
                Game::deal_from(&mut rng, roguelike_deck(), Seat::ALL[(seed % 3) as usize])
                    .expect("deal");
            let declarer = DeclarerHeuristic;
            let mut decl = declarer.choose_declaration(game.hand(Seat::DECLARER));
            if seed % 4 == 0 {
                decl = decl.with_risk(RiskLevel::Ouvert);
            }
            if decl.hand() {
                game.declare(decl).expect("declare");
            } else {
                game.pick_up_skat().expect("pick up");
                let decl = declarer.choose_game_after_skat(game.hand(Seat::DECLARER));
                let discard = declarer
                    .choose_discard(game.hand(Seat::DECLARER), decl.game())
                    .expect("discard");
                let blanks = game
                    .hand(Seat::DECLARER)
                    .iter()
                    .filter(|h| h.is_blank())
                    .count();
                let discarded = discard.iter().filter(|d| d.is_blank()).count();
                assert_eq!(discarded, blanks.min(2), "blanks are discarded first");
                game.discard(discard).expect("discard");
                game.declare(decl).expect("declare");
            }
            while let Some(seat) = game.current_seat() {
                let view = game.view(seat).expect("playing");
                let card = if seat.is_declarer() {
                    declarer.choose(&view)
                } else {
                    DefenderHeuristic.choose(&view)
                }
                .expect("a legal move");
                game.play(seat, card).expect("legal move");
            }
            assert_eq!(game.tricks().len(), 10);
            let o = game.outcome();
            assert_eq!(o.declarer_card_points + o.defender_card_points, 120);
            for trick in game.tricks() {
                if !trick.cards.iter().all(|c| c.is_blank()) {
                    assert!(!trick.winning_card().is_blank(), "a blank never wins");
                }
            }
        }
    }

    #[test]
    fn kontra_needs_a_strong_defender_hand() {
        let game = GameType::Suit(Suit::Hearts);
        let strong = parse_cards("HJ DJ HA HT S7 S8 C7 C8 D7 D8").expect("hand");
        assert!(DefenderHeuristic.says_kontra(&strong, game));
        let three = parse_cards("HJ HA HT SA ST CA S8 C7 D7 D8").expect("hand");
        assert!(DefenderHeuristic.says_kontra(&three, game));
        let weak = parse_cards("HJ HA S7 S8 S9 C7 C8 D7 D8 D9").expect("hand");
        assert!(!DefenderHeuristic.says_kontra(&weak, game));
    }
}

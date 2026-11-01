//! Dealing the unknown cards consistently with what a seat knows.
//!
//! From the [`SeatView`] of one seat, a [`Knowledge`] collects which cards
//! are known where (own hand, an open Ouvert or partner hand, cards the
//! caller pins to a holder, the skat after picking it up) and which
//! suits every other seat is known to lack (it did not follow a lead).
//! [`Knowledge::sample`] then deals the unknown cards to the other hands
//! and to the hidden pile (skat and talon) so that every hand gets its
//! number of cards and no seat receives a card of a suit it showed out of.
//!
//! The deal goes card by card in random order; each card goes to one of
//! its allowed places with a probability proportional to the free room,
//! among the places that keep the rest dealable (Hall's condition over the
//! at most four places). It is not exactly uniform over all consistent
//! deals but close, and it never fails on a consistent position.

use crate::cards::PlayingCard;
use crate::game::{Seat, SeatView, seat_at};
use crate::rng::Rng;
use crate::rules::{self, GameType};

/// The hidden pile: skat (if unknown) and talon.
const HIDDEN: usize = 3;
/// Kind index of trump in the void table.
const TRUMP_KIND: usize = 4;

/// Extra knowledge a caller may add to a [`SeatView`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Hints<C> {
    /// Cards whose place this seat knows: in a seat's hand (`Some`) or not
    /// in any hand (`None`, so in the skat or the talon). Durchsichtig
    /// cards and the cards "Der Kiebitz" showed are the examples. Cards
    /// already played are ignored.
    pub pinned: Vec<(C, Option<Seat>)>,
    /// Completed tricks at each swap of the defenders' hands (boss "Der
    /// Revolutionär"): a void seen before a swap moves to the partner.
    pub defender_swaps: Vec<usize>,
}

impl<C> Default for Hints<C> {
    fn default() -> Hints<C> {
        Hints {
            pinned: Vec::new(),
            defender_swaps: Vec::new(),
        }
    }
}

/// One consistent deal of the unknown cards.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Sample<C> {
    /// Current hands of all three seats.
    pub hands: [Vec<C>; 3],
    /// The skat as this deal has it (known or dealt).
    pub skat: Vec<C>,
}

/// What one seat knows about the location of the cards.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Knowledge<C> {
    /// Cards known to be in each seat's current hand.
    known: [Vec<C>; 3],
    /// The skat, if this seat knows it.
    skat: Option<[C; 2]>,
    /// Cards known to lie in the hidden pile.
    known_hidden: Vec<C>,
    /// Cards to deal.
    unknown: Vec<C>,
    /// Allowed places per unknown card (bits 0..3 seats, bit 3 hidden).
    allowed: Vec<u8>,
    /// Room per place for unknown cards.
    room: [usize; 4],
    /// Voids per seat and kind (four suits, trump).
    voids: [[bool; 5]; 3],
    /// Whether void information had to be dropped to stay consistent.
    relaxed: bool,
}

/// Kind index for the void table, `None` for a blank.
fn kind<C: PlayingCard>(game: GameType, card: C) -> Option<usize> {
    let face = card.face()?;
    if rules::is_trump(game, card) {
        Some(TRUMP_KIND)
    } else {
        Some(face.suit as usize)
    }
}

impl<C: PlayingCard> Knowledge<C> {
    /// Collect what `view.seat` knows. `None` if the view is inconsistent
    /// (more known cards than room), which never happens in a real game.
    #[must_use]
    pub fn from_view(view: &SeatView<'_, C>, hints: &Hints<C>) -> Option<Knowledge<C>> {
        let game = view.declaration.game();
        let me = view.seat.index();
        // Cards held now: equal for all seats that did not play to the
        // trick in progress yet.
        let played_now = |seat: usize| {
            (0..view.trick.len()).any(|i| seat_at(view.trick_lead, i).index() == seat)
        };
        let full = view.hand.len() + usize::from(played_now(me));
        let mut sizes = [0usize; 3];
        for (seat, size) in sizes.iter_mut().enumerate() {
            *size = full.checked_sub(usize::from(played_now(seat)))?;
        }
        let mut known: [Vec<C>; 3] = [Vec::new(), Vec::new(), Vec::new()];
        known[me] = view.hand.to_vec();
        if let (Some(partner), Some(hand)) = (view.seat.partner(), view.partner_hand) {
            known[partner.index()] = hand.to_vec();
        }
        if let Some(open) = view.declarer_hand {
            if me != 0 {
                known[0] = open.to_vec();
            }
        }
        let mut seen: Vec<C> = known.iter().flatten().copied().collect();
        seen.extend(view.tricks.iter().flat_map(|t| t.cards));
        seen.extend(view.trick.iter().copied());
        if let Some(skat) = view.skat {
            seen.extend(skat);
        }
        let mut known_hidden = Vec::new();
        for &(card, place) in &hints.pinned {
            if seen.contains(&card) || !view.deck.contains(&card) {
                continue;
            }
            match place {
                Some(seat) => known[seat.index()].push(card),
                None => known_hidden.push(card),
            }
            seen.push(card);
        }
        let unknown: Vec<C> = view
            .deck
            .iter()
            .copied()
            .filter(|c| !seen.contains(c))
            .collect();
        let mut room = [0usize; 4];
        for seat in 0..3 {
            room[seat] = sizes[seat].checked_sub(known[seat].len())?;
        }
        let dealt: usize = room[..3].iter().sum();
        room[HIDDEN] = unknown.len().checked_sub(dealt)?;
        let mut voids = [[false; 5]; 3];
        let completed = view.tricks.iter().map(|t| (t.lead, &t.cards[..]));
        let current = std::iter::once((view.trick_lead, view.trick));
        for (index, (lead_seat, cards)) in completed.chain(current).enumerate() {
            let Some(first) = cards.iter().position(|c| !c.is_blank()) else {
                continue;
            };
            let lead = cards[first];
            let Some(lead_kind) = kind(game, lead) else {
                continue;
            };
            for (i, &card) in cards.iter().enumerate().skip(first + 1) {
                if card.is_blank() || rules::follows_lead(game, lead, card) {
                    continue;
                }
                let mut seat = seat_at(lead_seat, i);
                let swaps = hints.defender_swaps.iter().filter(|&&p| p > index).count();
                if swaps % 2 == 1 {
                    seat = seat.partner().unwrap_or(seat);
                }
                voids[seat.index()][lead_kind] = true;
            }
        }
        let mut knowledge = Knowledge {
            known,
            skat: view.skat,
            known_hidden,
            allowed: Vec::new(),
            unknown,
            room,
            voids,
            relaxed: false,
        };
        knowledge.allowed = knowledge.allowed_places(game, true);
        if !knowledge.feasible_all() {
            // Contradicting information (a swap the caller did not report):
            // forget the voids rather than refuse to play.
            knowledge.allowed = knowledge.allowed_places(game, false);
            knowledge.relaxed = true;
            if !knowledge.feasible_all() {
                return None;
            }
        }
        Some(knowledge)
    }

    fn allowed_places(&self, game: GameType, use_voids: bool) -> Vec<u8> {
        self.unknown
            .iter()
            .map(|&card| {
                let mut mask = 0u8;
                for seat in 0..3 {
                    let void = use_voids && kind(game, card).is_some_and(|k| self.voids[seat][k]);
                    if self.room[seat] > 0 && !void {
                        mask |= 1 << seat;
                    }
                }
                if self.room[HIDDEN] > 0 {
                    mask |= 1 << HIDDEN;
                }
                mask
            })
            .collect()
    }

    fn feasible_all(&self) -> bool {
        let mut counts = [0usize; 16];
        for &mask in &self.allowed {
            counts[usize::from(mask)] += 1;
        }
        feasible(&counts, &self.room)
    }

    /// Whether `seat` is known to have no card of `card`'s kind.
    #[must_use]
    pub fn is_void(&self, game: GameType, seat: Seat, card: C) -> bool {
        kind(game, card).is_some_and(|k| self.voids[seat.index()][k])
    }

    /// Whether void information was dropped because it contradicted the
    /// rest.
    #[must_use]
    pub fn relaxed(&self) -> bool {
        self.relaxed
    }

    /// Cards whose place is unknown.
    #[must_use]
    pub fn unknown(&self) -> &[C] {
        &self.unknown
    }

    /// Deal the unknown cards once.
    #[must_use]
    pub fn sample(&self, rng: &mut Rng) -> Sample<C> {
        let mut order: Vec<usize> = (0..self.unknown.len()).collect();
        rng.shuffle(&mut order);
        let mut counts = [0usize; 16];
        for &mask in &self.allowed {
            counts[usize::from(mask)] += 1;
        }
        let mut room = self.room;
        let mut hands = self.known.clone();
        let mut hidden = self.known_hidden.clone();
        for i in order {
            let card = self.unknown[i];
            let mask = self.allowed[i];
            counts[usize::from(mask)] -= 1;
            let mut options = [(0usize, 0u64); 4];
            let mut n = 0;
            let mut total = 0u64;
            for place in 0..4 {
                if mask & (1 << place) == 0 || room[place] == 0 {
                    continue;
                }
                room[place] -= 1;
                if feasible(&counts, &room) {
                    let weight = room[place] as u64 + 1;
                    options[n] = (place, weight);
                    n += 1;
                    total += weight;
                }
                room[place] += 1;
            }
            // A consistent start keeps at least one place feasible; fall
            // back to any allowed place with room should that ever fail.
            let place = if n == 0 {
                (0..4)
                    .find(|&p| mask & (1 << p) != 0 && room[p] > 0)
                    .or_else(|| (0..4).find(|&p| room[p] > 0))
                    .unwrap_or(HIDDEN)
            } else {
                let mut roll = rng.gen_range(0..total);
                let mut chosen = options[0].0;
                for &(p, w) in &options[..n] {
                    if roll < w {
                        chosen = p;
                        break;
                    }
                    roll -= w;
                }
                chosen
            };
            room[place] = room[place].saturating_sub(1);
            if place == HIDDEN {
                hidden.push(card);
            } else {
                hands[place].push(card);
            }
        }
        for hand in &mut hands {
            hand.sort();
        }
        let skat = match self.skat {
            Some(skat) => skat.to_vec(),
            None => {
                // The hidden pile is dealt in random order: its first two
                // cards are the skat, the rest the talon.
                rng.shuffle(&mut hidden);
                hidden.into_iter().take(2).collect()
            }
        };
        Sample { hands, skat }
    }
}

/// Hall's condition for dealing cards with the allowed-place masks counted
/// in `counts` into places with `room`: for every set of places, the cards
/// that may only go there fit.
fn feasible(counts: &[usize; 16], room: &[usize; 4]) -> bool {
    for set in 1u8..16 {
        let cap: usize = (0..4)
            .filter(|&p| set & (1 << p) != 0)
            .map(|p| room[p])
            .sum();
        let need: usize = (1u8..16)
            .filter(|&mask| mask & !set == 0)
            .map(|mask| counts[usize::from(mask)])
            .sum();
        if need > cap {
            return false;
        }
    }
    // Cards with no allowed place at all can never be dealt.
    counts[0] == 0
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cards::{Card, Suit, parse_cards};
    use crate::game::Game;
    use crate::rules::{Declaration, RiskLevel};

    fn c(s: &str) -> Card {
        s.parse().expect("test card")
    }

    /// Hearts hand game. Trick 1: the declarer cashes the SA, seat 1 shows
    /// out of spades. Trick 2: C7 led, seat 1 wins with the CA.
    fn game_with_voids() -> Game {
        let hands = [
            parse_cards("CJ SJ HA HT HK SA ST S9 C7 D7").expect("hand"),
            parse_cards("HJ DJ H9 H8 CA CT CK DA DT DK").expect("hand"),
            parse_cards("H7 HQ SK SQ S8 S7 DQ D9 D8 CQ").expect("hand"),
        ];
        let mut game =
            Game::with_cards(hands, [c("C9"), c("C8")], Seat::DECLARER).expect("valid deal");
        game.declare(Declaration::suit(Suit::Hearts, RiskLevel::Hand))
            .expect("declare");
        // Trick 1: SA, seat 1 shows out (no spades), seat 2 follows.
        game.play(Seat::DECLARER, c("SA")).expect("lead");
        game.play(Seat::ALL[1], c("DK")).expect("void");
        game.play(Seat::ALL[2], c("S7")).expect("follow");
        // Trick 2: declarer leads C7, seat 1 follows, seat 2 shows out.
        game.play(Seat::DECLARER, c("C7")).expect("lead");
        game.play(Seat::ALL[1], c("CA")).expect("follow");
        game.play(Seat::ALL[2], c("CQ")).expect("follow");
        game
    }

    #[test]
    fn samples_respect_voids_sizes_and_known_cards() {
        let game = game_with_voids();
        // Seat 1 won trick 2 (CA) and leads now. Seat 2 followed clubs
        // with the CQ, so only seat 1 shows a void (spades).
        let view = game.view(Seat::ALL[1]).expect("view");
        let knowledge = Knowledge::from_view(&view, &Hints::default()).expect("consistent");
        assert!(!knowledge.relaxed());
        let mut rng = Rng::from_seed(1);
        for _ in 0..300 {
            let sample = knowledge.sample(&mut rng);
            assert_eq!(sample.hands[1], game.hand(Seat::ALL[1]));
            assert_eq!(sample.hands[0].len(), 8);
            assert_eq!(sample.hands[2].len(), 8);
            assert_eq!(sample.skat.len(), 2);
            let mut all: Vec<Card> = sample
                .hands
                .iter()
                .flatten()
                .copied()
                .chain(sample.skat.iter().copied())
                .chain(game.tricks().iter().flat_map(|t| t.cards))
                .collect();
            all.sort();
            all.dedup();
            assert_eq!(all.len(), 32, "every card exactly once");
        }
        // The declarer's view: seat 1 has no spades in any deal.
        let mut game = game;
        game.play(Seat::ALL[1], c("CT")).expect("lead");
        let view = game.view(Seat::ALL[2]).expect("view");
        let knowledge = Knowledge::from_view(&view, &Hints::default()).expect("consistent");
        let game_type = view.declaration.game();
        for _ in 0..300 {
            let sample = knowledge.sample(&mut rng);
            assert!(
                sample.hands[1]
                    .iter()
                    .all(|&x| !(suit_is(game_type, x, Suit::Spades))),
                "seat 1 showed out of spades: {:?}",
                sample.hands[1]
            );
            assert_eq!(sample.hands[2], game.hand(Seat::ALL[2]));
        }
    }

    fn suit_is(game: GameType, card: Card, suit: Suit) -> bool {
        !rules::is_trump(game, card) && card.suit == suit
    }

    #[test]
    fn open_hands_and_pinned_cards_are_dealt_where_they_are() {
        let mut game = game_with_voids();
        game.play(Seat::ALL[1], c("CT")).expect("lead");
        let view = game.view(Seat::ALL[2]).expect("view");
        // Pin the HJ to seat 1 and the C9 to the skat.
        let hints = Hints {
            pinned: vec![(c("HJ"), Some(Seat::ALL[1])), (c("C9"), None)],
            defender_swaps: Vec::new(),
        };
        let knowledge = Knowledge::from_view(&view, &hints).expect("consistent");
        let mut rng = Rng::from_seed(2);
        for _ in 0..200 {
            let sample = knowledge.sample(&mut rng);
            assert!(sample.hands[1].contains(&c("HJ")));
            assert!(!sample.hands.iter().flatten().any(|&x| x == c("C9")));
        }
    }

    #[test]
    fn ouvert_declarer_hand_is_known_to_the_defenders() {
        let hands = [
            parse_cards("CJ SJ HA HT HK SA ST S9 C7 D7").expect("hand"),
            parse_cards("HJ DJ H9 H8 CA CT CK DA DT DK").expect("hand"),
            parse_cards("H7 HQ SK SQ S8 S7 DQ D9 D8 CQ").expect("hand"),
        ];
        let mut game =
            Game::with_cards(hands, [c("C9"), c("C8")], Seat::DECLARER).expect("valid deal");
        // Ouvert is a hand game: the skat stays unknown to everybody.
        game.declare(Declaration::suit(Suit::Hearts, RiskLevel::Ouvert))
            .expect("declare");
        game.play(Seat::DECLARER, c("CJ")).expect("lead");
        let view = game.view(Seat::ALL[1]).expect("view");
        let knowledge = Knowledge::from_view(&view, &Hints::default()).expect("consistent");
        let mut rng = Rng::from_seed(3);
        for _ in 0..50 {
            let sample = knowledge.sample(&mut rng);
            assert_eq!(sample.hands[0], game.hand(Seat::DECLARER));
            assert_eq!(sample.hands[2].len(), 10);
        }
    }

    #[test]
    fn a_swap_moves_the_void_to_the_partner() {
        let game = game_with_voids();
        // Seat 1 showed out of spades in trick 1. After a swap at 1 trick
        // the void belongs to seat 2's hand.
        let view = game.view(Seat::ALL[1]).expect("view");
        let swapped = Hints {
            pinned: Vec::new(),
            defender_swaps: vec![1],
        };
        let game_type = view.declaration.game();
        let plain = Knowledge::from_view(&view, &Hints::default()).expect("consistent");
        assert!(plain.is_void(game_type, Seat::ALL[1], c("SK")));
        let after = Knowledge::from_view(&view, &swapped).expect("consistent");
        assert!(after.is_void(game_type, Seat::ALL[2], c("SK")));
        assert!(!after.is_void(game_type, Seat::ALL[1], c("SK")));
        // A swap before the trick changes nothing.
        let before = Hints {
            pinned: Vec::new(),
            defender_swaps: vec![0],
        };
        let early = Knowledge::from_view(&view, &before).expect("consistent");
        assert!(early.is_void(game_type, Seat::ALL[1], c("SK")));
    }
}

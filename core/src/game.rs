//! State machine for a single Skat game, in the order of the Skatordnung
//! (ADR 0011): deal → hand declaration, or pick up the skat → discard →
//! declaration → ten tricks → scored.
//!
//! The game is generic over the card type ([`PlayingCard`]): `Game<Card>`
//! (the default) plays the plain 32-card deck of milestone M1, while
//! `Game<DealtCard>` plays card instances of the roguelike deck: the 32
//! cards, each in the edition of the seat that holds it (ADR 0015, see
//! [`crate::deck`]).
//!
//! Every state change is recorded as an [`Event`]; frontends animate them,
//! the simulator ignores them. Illegal actions return a [`GameError`] and
//! leave the state untouched. Nothing in here panics on bad input.

use std::fmt;

use serde::{Deserialize, Serialize};

use crate::cards::{Card, PlayingCard, Rank, standard_deck};
use crate::rng::Rng;
use crate::rules::{
    self, Declaration, GameValue, LeadSuit, Matadors, NullStyle, Outcome, SeatTotals,
};

/// Cards each seat receives when dealing.
pub const HAND_SIZE: usize = 10;
/// Tricks in a game.
pub const TRICKS_PER_GAME: usize = 10;
/// Cards dealt into one game: three hands plus the skat.
pub const CARDS_IN_PLAY: usize = 3 * HAND_SIZE + 2;

/// One of the three seats at the table. Seat 0 is always the declarer
/// (the human player, see ADR 0002), seats 1 and 2 are the defenders.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Seat(u8);

impl Seat {
    /// The declarer's seat.
    pub const DECLARER: Seat = Seat(0);
    /// All seats in table order.
    pub const ALL: [Seat; 3] = [Seat(0), Seat(1), Seat(2)];
    /// The two defender seats.
    pub const DEFENDERS: [Seat; 2] = [Seat(1), Seat(2)];

    /// Seat with the given index (0..3).
    #[must_use]
    pub const fn from_index(index: usize) -> Option<Seat> {
        match index {
            0 => Some(Seat(0)),
            1 => Some(Seat(1)),
            2 => Some(Seat(2)),
            _ => None,
        }
    }

    /// Index of this seat (0..3).
    #[must_use]
    pub const fn index(self) -> usize {
        self.0 as usize
    }

    /// The seat that plays after this one.
    #[must_use]
    pub const fn next(self) -> Seat {
        Seat((self.0 + 1) % 3)
    }

    /// Whether this is the declarer's seat.
    #[must_use]
    pub const fn is_declarer(self) -> bool {
        self.0 == 0
    }

    /// The defender partner of a defender seat; `None` for the declarer.
    #[must_use]
    pub const fn partner(self) -> Option<Seat> {
        match self.0 {
            1 => Some(Seat(2)),
            2 => Some(Seat(1)),
            _ => None,
        }
    }
}

impl fmt::Display for Seat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "seat {}", self.0)
    }
}

/// Table position relative to the dealer for one game. Forehand leads the
/// first trick.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Position {
    /// Vorhand: leads the first trick.
    Forehand,
    /// Mittelhand.
    Middlehand,
    /// Hinterhand.
    Rearhand,
}

/// Phase of the game. Dealing happens inside [`Game::deal`], so a freshly
/// dealt game starts in `Declaring`.
///
/// The order follows the Skatordnung (ADR 0011): with ten cards the
/// declarer either declares a hand game or picks up the skat; after picking
/// it up he discards two cards and only then declares the game. Saves from
/// before ADR 0011 that wait in `Declaring` simply continue with this
/// choice; a save in `Discarding` that already holds a declaration goes
/// straight to `Playing` after the discard.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Phase {
    /// The declarer looks at ten cards and decides: declare a hand game
    /// ([`Game::declare`]) or pick up the skat ([`Game::pick_up_skat`]).
    Declaring,
    /// The declarer holds twelve cards and must discard two (thirteen and
    /// three after drawing a card with the skat, "Drei drücken").
    Discarding,
    /// The skat is picked up and two cards are discarded: the declarer
    /// announces the game, without hand levels ([`Game::declare`]).
    DeclaringAfterSkat,
    /// Tricks are being played.
    Playing,
    /// All tricks are played (or the defenders conceded) and the game is
    /// scored by the Skatordnung. Relic modifiers are applied on top by the
    /// run (see [`crate::content::ScoreBreakdown`]).
    Finished(GameValue),
}

/// A trick that has been played out.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompletedTrick<C = Card> {
    /// Seat that led.
    pub lead: Seat,
    /// Cards in play order, `cards[0]` by `lead`.
    pub cards: [C; 3],
    /// Seat that won the trick.
    pub winner: Seat,
}

impl<C: PlayingCard> CompletedTrick<C> {
    /// Augen in the trick.
    #[must_use]
    pub fn card_points(&self) -> u32 {
        self.cards.iter().map(|c| c.card_points()).sum()
    }

    /// Augen in the trick with their sign: Augen below zero
    /// ([`PlayingCard::point_debt`]) count against it (DESIGN 7.2).
    #[must_use]
    pub fn signed_points(&self) -> i64 {
        self.cards.iter().map(|c| c.signed_points()).sum()
    }

    /// Seat that played `cards[i]`.
    #[must_use]
    pub fn seat_of(&self, i: usize) -> Seat {
        seat_at(self.lead, i)
    }

    /// The card that won the trick.
    #[must_use]
    pub fn winning_card(&self) -> C {
        let offset = (self.winner.index() + 3 - self.lead.index()) % 3;
        self.cards[offset]
    }
}

/// Seat that plays the card at `offset` in a trick led by `lead`.
#[must_use]
pub fn seat_at(lead: Seat, offset: usize) -> Seat {
    // offset % 3 is always 0..3, so the lookup never fails.
    Seat::from_index((lead.index() + offset) % 3).unwrap_or(lead)
}

/// Something that happened in the game, in order of occurrence.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Event<C = Card> {
    /// Cards were dealt.
    Dealt {
        /// Seat that will lead the first trick.
        forehand: Seat,
        /// Hands per seat.
        hands: [Vec<C>; 3],
        /// The two cards in the skat.
        skat: [C; 2],
    },
    /// The defenders' hands and the skat were dealt again from the cards
    /// the declarer does not hold (house rule "Schieben").
    SkatRedrawn {
        /// Hands per seat; the declarer's hand is unchanged.
        hands: [Vec<C>; 3],
        /// The new skat.
        skat: [C; 2],
    },
    /// The declarer announced the game.
    Declared {
        /// The declaration.
        declaration: Declaration,
    },
    /// The declarer picked up the skat (not in hand games).
    SkatPickedUp {
        /// The two cards that joined the declarer's hand.
        skat: [C; 2],
    },
    /// The declarer discarded two cards.
    Discarded {
        /// The discarded cards (they count for the declarer).
        cards: [C; 2],
    },
    /// A card was played.
    CardPlayed {
        /// Who played.
        seat: Seat,
        /// What was played.
        card: C,
    },
    /// A trick was completed.
    TrickWon {
        /// The completed trick.
        trick: CompletedTrick<C>,
        /// Augen in the trick.
        card_points: u32,
    },
    /// The defenders gave up (house rule "Schenken"). The remaining tricks
    /// and the Augen still in the hands go to the declarer.
    Conceded {
        /// Tricks that were not played.
        tricks: u32,
        /// Augen still in the hands.
        card_points: u32,
    },
    /// The two defenders swapped their hands (boss "Der Revolutionär").
    DefenderHandsSwapped {
        /// Hands per seat after the swap.
        hands: [Vec<C>; 3],
    },
    /// The declarer took a card from the untouched skat into his hand and
    /// put one of his cards in its place (figure "Die Oma").
    SkatSwapped {
        /// The card taken from the skat.
        took: C,
        /// The card put into the skat.
        gave: C,
    },
    /// A card changed its holder and shows the holder's edition now (ADR
    /// 0015): drawn with "Drei drücken", given back, swapped by "Der
    /// Revolutionär" (a new deal by "Schieben" logs only
    /// [`Event::SkatRedrawn`]). Only when the edition changes the card.
    CardChangedHands {
        /// The new holder (the declarer also for the skat).
        seat: Seat,
        /// The card in the edition of its old holder.
        old: C,
        /// The card in the edition of its new holder.
        new: C,
    },
    /// With the skat the declarer drew a card from a defender's hand
    /// (house rule "Drei drücken"); he discards three.
    CardDrawn {
        /// The defender who held it.
        seat: Seat,
        /// The card, now in the declarer's hand.
        card: C,
    },
    /// Of the declarer's three discards the lowest went back to the
    /// defender the third card was drawn from ("Drei drücken",
    /// [`rules::returned_discard`]).
    CardReturned {
        /// The defender who gets it.
        seat: Seat,
        /// The card.
        card: C,
    },
    /// The declarer named what the next defender who leads must lead (house
    /// rule "Lockvogel", removed on 02.11.2026). Never emitted any more;
    /// kept so that the log of an older save still reads.
    LeadDemanded {
        /// What must be led.
        suit: LeadSuit,
    },
    /// A card of the declarer's hand was repainted into another suit for
    /// this game (consumable "Blue Curaçao"): `new` replaces `old` everywhere.
    CardChalked {
        /// The card as it was.
        old: C,
        /// The card as it is now.
        new: C,
    },
    /// A Wandelkarte of the declarer's hand took its suit as the game
    /// started (DESIGN 7.2, [`rules::settled_wild_suit`]): `new` replaces
    /// `old` in the hand and the deck until the end of the game. A Unter
    /// stays as it is (`new` differs from `old` only in no longer being
    /// wild).
    WildCardSettled {
        /// The card as printed, still wild.
        old: C,
        /// The card with its settled suit.
        new: C,
    },
    /// An "Untot" card rose from the last completed trick back into the
    /// hand of the seat that played it, and `substitute` from that hand
    /// took its place in the trick ([`Game::raise`]). The trick winner
    /// stays; the trick's Augen are now those of the substitute.
    CardRose {
        /// Who played the card.
        seat: Seat,
        /// The card that came back into the hand.
        card: C,
        /// The card that lies in the trick now.
        substitute: C,
    },
    /// The game is over and scored: all tricks played, the defenders
    /// conceded, or the Null declarer took a trick.
    GameFinished {
        /// Trick outcome used for scoring.
        outcome: Outcome,
        /// Matadors over the declarer's twelve cards.
        matadors: Matadors,
        /// The resulting game value.
        value: GameValue,
        /// Augen and tricks per seat (who has the most Augen in a Ramsch).
        #[serde(default)]
        seats: SeatTotals,
    },
}

/// Why an action was rejected.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum GameError<C = Card> {
    /// The action is not allowed in the current phase.
    WrongPhase {
        /// What the game is doing right now.
        phase: Phase,
    },
    /// It is another seat's turn.
    NotYourTurn {
        /// The seat that tried to act.
        seat: Seat,
        /// The seat whose turn it is.
        expected: Seat,
    },
    /// The seat does not hold the card.
    CardNotInHand {
        /// The seat that tried to act.
        seat: Seat,
        /// The card it does not hold.
        card: C,
    },
    /// The card does not follow the lead although the seat could.
    MustFollowSuit {
        /// The rejected card.
        card: C,
        /// The lead card of the trick.
        lead: C,
    },
    /// The two discards are not two distinct cards from the hand.
    InvalidDiscard {
        /// The rejected discards.
        cards: [C; 2],
    },
    /// The supplied cards do not form a legal deal (32 distinct cards,
    /// ten per hand).
    InvalidDeal,
    /// The action needs a trick boundary but a trick is in progress.
    TrickInProgress,
    /// The declaration does not fit the skat: before the skat is picked up
    /// only hand games may be declared, after picking it up only games
    /// without hand levels (no Hand, Schneider or Schwarz angesagt, no
    /// Ouvert except Null Ouvert).
    InvalidDeclaration {
        /// The rejected declaration.
        declaration: Declaration,
    },
    /// The card to take is not in the skat, or the skat may not be
    /// touched (no hand game, or play has begun).
    InvalidSkatSwap,
    /// The card must not be discarded ("Ramponiert") while another card
    /// could go into the skat instead.
    DiscardForbidden {
        /// The rejected card.
        card: C,
    },
}

impl<C: fmt::Display> fmt::Display for GameError<C> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GameError::WrongPhase { phase } => write!(f, "action not allowed in phase {phase:?}"),
            GameError::NotYourTurn { seat, expected } => {
                write!(f, "{seat} acted but it is {expected}'s turn")
            }
            GameError::CardNotInHand { seat, card } => write!(f, "{seat} does not hold {card}"),
            GameError::MustFollowSuit { card, lead } => {
                write!(f, "{card} does not follow the lead {lead}")
            }
            GameError::InvalidDiscard { cards } => {
                write!(f, "cannot discard {} {}", cards[0], cards[1])
            }
            GameError::InvalidDeal => f.write_str("invalid deal"),
            GameError::TrickInProgress => f.write_str("a trick is in progress"),
            GameError::InvalidDeclaration { declaration } => {
                write!(f, "{declaration:?} does not fit the skat")
            }
            GameError::InvalidSkatSwap => f.write_str("the skat cannot be swapped with"),
            GameError::DiscardForbidden { card } => write!(f, "{card} must not be discarded"),
        }
    }
}

impl<C: fmt::Debug + fmt::Display> std::error::Error for GameError<C> {}

/// What a seat is allowed to know when choosing a card: its own hand, the
/// declaration, everything played so far and, for the declarer after
/// discarding, the skat. In an Ouvert game the defenders also see the
/// declarer's hand once the first card has been led.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SeatView<'a, C = Card> {
    /// The seat looking at the table.
    pub seat: Seat,
    /// The declaration being played.
    pub declaration: Declaration,
    /// Seat that led the first trick.
    pub forehand: Seat,
    /// Own hand.
    pub hand: &'a [C],
    /// Seat that led the current trick.
    pub trick_lead: Seat,
    /// Cards of the current trick in play order (0 to 2 cards).
    pub trick: &'a [C],
    /// Completed tricks in order.
    pub tricks: &'a [CompletedTrick<C>],
    /// The skat, if this seat knows it (declarer after picking it up).
    pub skat: Option<[C; 2]>,
    /// The declarer's current hand, open on the table in an Ouvert game
    /// from the first lead on. `None` for the declarer's own view and in
    /// all other games.
    pub declarer_hand: Option<&'a [C]>,
    /// The partner's hand, for a defender whose partner plays open (boss
    /// "Die Zwillinge"). `None` otherwise.
    pub partner_hand: Option<&'a [C]>,
    /// Every card of this game, sorted: the 32 dealt cards, each in the
    /// edition of its current holder (the skat in the declarer's).
    pub deck: &'a [C],
    /// The 32 cards in the edition of each seat (index = seat, the skat
    /// goes with the declarer's), or empty lists for a game without
    /// editions. The editions are public (ADR 0015, DESIGN 10.5): a seat
    /// that deals the unknown cards resolves each with the seat it falls
    /// to ([`SeatView::in_edition_of`]).
    pub editions: &'a [Vec<C>; 3],
    /// Cards this seat may play now. Empty if it is not its turn.
    pub legal_moves: Vec<C>,
}

impl<C: PlayingCard> SeatView<'_, C> {
    /// Every card that is no longer unknown to this seat: own hand, all
    /// played cards, the skat if known and an open partner hand.
    #[must_use]
    pub fn seen_cards(&self) -> Vec<C> {
        let mut seen: Vec<C> = self.hand.to_vec();
        if let Some(partner) = self.partner_hand {
            seen.extend(partner.iter().copied());
        }
        seen.extend(self.tricks.iter().flat_map(|t| t.cards.iter().copied()));
        seen.extend(self.trick.iter().copied());
        if let Some(skat) = self.skat {
            seen.extend(skat);
        }
        seen
    }

    /// Cards whose location this seat does not know. The open hand of an
    /// Ouvert declarer still counts as unseen here: those cards can still
    /// beat the seat's cards.
    #[must_use]
    pub fn unseen_cards(&self) -> Vec<C> {
        let seen = self.seen_cards();
        self.deck
            .iter()
            .copied()
            .filter(|c| !seen.contains(c))
            .collect()
    }

    /// The card that must be followed in the current trick, if any.
    #[must_use]
    pub fn lead_card(&self) -> Option<C> {
        rules::effective_lead(self.trick)
    }

    /// `card` in the edition of `seat` ([`Game::in_edition_of`]).
    #[must_use]
    pub fn in_edition_of(&self, card: C, seat: Seat) -> C {
        resolve_in(self.editions, card, seat)
    }
}

/// `card` in the edition of `seat`: the copy with the same instance in
/// `editions[seat]`, or `card` itself without editions.
pub(crate) fn resolve_in<C: PlayingCard>(editions: &[Vec<C>; 3], card: C, seat: Seat) -> C {
    let Some(id) = card.instance() else {
        return card;
    };
    editions[seat.index()]
        .iter()
        .copied()
        .find(|c| c.instance() == Some(id))
        .unwrap_or(card)
}

/// A single Skat game.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Game<C = Card> {
    forehand: Seat,
    hands: [Vec<C>; 3],
    skat: [C; 2],
    /// The 32 cards, sorted, each in the edition of its current holder:
    /// what any seat may expect to see.
    deck: Vec<C>,
    /// The 32 cards in the edition of each seat (ADR 0015), empty for a
    /// game without editions. The skat goes with the declarer's.
    #[serde(default = "no_editions", skip_serializing_if = "no_edition_lists")]
    editions: [Vec<C>; 3],
    /// The 32 dealt cards, sorted: the trumps in play for matadors.
    in_play: Vec<C>,
    /// Declarer's original hand plus skat: the twelve cards for matadors.
    declarer_cards: Vec<C>,
    declaration: Option<Declaration>,
    phase: Phase,
    trick_lead: Seat,
    trick: Vec<C>,
    tricks: Vec<CompletedTrick<C>>,
    card_points: [u32; 3],
    /// Tricks and Augen handed to the declarer by a concession.
    conceded: Option<(u32, u32)>,
    /// The defenders see each other's hands (boss "Die Zwillinge").
    #[serde(default)]
    defenders_open: bool,
    /// The card the declarer drew from a defender with the skat ("Drei
    /// drücken") and who held it.
    #[serde(default = "no_card", skip_serializing_if = "Option::is_none")]
    drawn: Option<(Seat, C)>,
    /// The discard that went back to that defender ("Drei drücken").
    #[serde(default = "no_card", skip_serializing_if = "Option::is_none")]
    returned: Option<(Seat, C)>,
    /// Extra Augen per seat from card effects ("Erster Stich", DESIGN
    /// 7.2).
    #[serde(default, skip_serializing_if = "no_bonus")]
    bonus_points: [u32; 3],
    log: Vec<Event<C>>,
}

#[allow(clippy::trivially_copy_pass_by_ref)]
fn no_bonus(points: &[u32; 3]) -> bool {
    *points == [0; 3]
}

const fn no_editions<C>() -> [Vec<C>; 3] {
    [Vec::new(), Vec::new(), Vec::new()]
}

fn no_edition_lists<C>(editions: &[Vec<C>; 3]) -> bool {
    editions.iter().all(Vec::is_empty)
}

/// Default of the optional card fields of saves from before them (without a
/// `C: Default` bound, which a derived default would ask for).
const fn no_card<C>() -> Option<(Seat, C)> {
    None
}

impl Game<Card> {
    /// Shuffle the standard deck with `rng` and deal 10/10/10/2. The game
    /// starts in [`Phase::Declaring`]; the `Dealt` event is in [`Game::log`].
    #[must_use]
    pub fn deal(rng: &mut Rng, forehand: Seat) -> Game<Card> {
        let mut deck = standard_deck();
        rng.shuffle(&mut deck);
        let hands = [
            deck[0..HAND_SIZE].to_vec(),
            deck[HAND_SIZE..2 * HAND_SIZE].to_vec(),
            deck[2 * HAND_SIZE..3 * HAND_SIZE].to_vec(),
        ];
        let skat = [deck[3 * HAND_SIZE], deck[3 * HAND_SIZE + 1]];
        // A shuffled standard deck is a valid deal by construction.
        Game::build(hands, skat, forehand)
    }
}

impl<C: PlayingCard> Game<C> {
    /// Shuffle `cards` with `rng` and deal 10/10/10/2. Fails unless there
    /// are exactly 32 different cards.
    pub fn deal_from(
        rng: &mut Rng,
        mut cards: Vec<C>,
        forehand: Seat,
    ) -> Result<Game<C>, GameError<C>> {
        if cards.len() != CARDS_IN_PLAY {
            return Err(GameError::InvalidDeal);
        }
        rng.shuffle(&mut cards);
        let hands = [
            cards[0..HAND_SIZE].to_vec(),
            cards[HAND_SIZE..2 * HAND_SIZE].to_vec(),
            cards[2 * HAND_SIZE..3 * HAND_SIZE].to_vec(),
        ];
        let skat = [cards[3 * HAND_SIZE], cards[3 * HAND_SIZE + 1]];
        Game::with_cards(hands, skat, forehand)
    }

    /// Deal the 32 cards with their own edition per seat (ADR 0015):
    /// `editions[seat]` lists the same 32 card instances in the edition of
    /// that seat, in the same order. The order is shuffled with `rng`
    /// exactly as [`Game::deal_from`] shuffles one list, so equal seeds deal
    /// the same cards to the same seats; then every card is taken from the
    /// edition of the seat it falls to, the skat from the declarer's.
    pub fn deal_editions(
        rng: &mut Rng,
        editions: [Vec<C>; 3],
        forehand: Seat,
    ) -> Result<Game<C>, GameError<C>> {
        let same_cards = editions.iter().all(|e| {
            e.len() == CARDS_IN_PLAY
                && e.iter()
                    .zip(&editions[0])
                    .all(|(a, b)| a.instance() == b.instance())
        });
        if !same_cards {
            return Err(GameError::InvalidDeal);
        }
        let mut order: Vec<usize> = (0..CARDS_IN_PLAY).collect();
        rng.shuffle(&mut order);
        let take = |seat: usize, range: std::ops::Range<usize>| -> Vec<C> {
            order[range].iter().map(|&i| editions[seat][i]).collect()
        };
        let hands = [
            take(0, 0..HAND_SIZE),
            take(1, HAND_SIZE..2 * HAND_SIZE),
            take(2, 2 * HAND_SIZE..3 * HAND_SIZE),
        ];
        let skat = [
            editions[0][order[3 * HAND_SIZE]],
            editions[0][order[3 * HAND_SIZE + 1]],
        ];
        let mut game = Game::with_cards(hands, skat, forehand)?;
        game.editions = editions;
        for edition in &mut game.editions {
            edition.sort();
        }
        Ok(game)
    }

    /// Start a game from explicit cards with an edition per seat (tests,
    /// replays): like [`Game::with_cards`], and `editions` lists the 32
    /// cards in the edition of each seat (see [`Game::deal_editions`]).
    /// Every card must be in the edition of the seat that gets it, the
    /// skat in the declarer's.
    pub fn with_editions(
        hands: [Vec<C>; 3],
        skat: [C; 2],
        mut editions: [Vec<C>; 3],
        forehand: Seat,
    ) -> Result<Game<C>, GameError<C>> {
        let fits = |seat: usize, card: &C| editions[seat].contains(card);
        let placed = hands
            .iter()
            .enumerate()
            .all(|(seat, hand)| hand.iter().all(|c| fits(seat, c)))
            && skat.iter().all(|c| fits(0, c));
        if !placed || editions.iter().any(|e| e.len() != CARDS_IN_PLAY) {
            return Err(GameError::InvalidDeal);
        }
        let mut game = Game::with_cards(hands, skat, forehand)?;
        for edition in &mut editions {
            edition.sort();
        }
        game.editions = editions;
        Ok(game)
    }

    /// Start a game from explicit cards (tests, replays, frontends with a
    /// modified deck). Hands are sorted so that equal deals compare equal.
    pub fn with_cards(
        hands: [Vec<C>; 3],
        skat: [C; 2],
        forehand: Seat,
    ) -> Result<Game<C>, GameError<C>> {
        if hands.iter().any(|h| h.len() != HAND_SIZE) {
            return Err(GameError::InvalidDeal);
        }
        let mut all: Vec<C> = hands.iter().flatten().copied().chain(skat).collect();
        let total = all.len();
        all.sort();
        all.dedup();
        if all.len() != total {
            return Err(GameError::InvalidDeal);
        }
        Ok(Game::build(hands, skat, forehand))
    }

    /// Build the initial state from cards that are known to be valid.
    fn build(mut hands: [Vec<C>; 3], skat: [C; 2], forehand: Seat) -> Game<C> {
        for hand in &mut hands {
            hand.sort();
        }
        let mut declarer_cards = hands[0].clone();
        declarer_cards.extend(skat);
        let mut in_play: Vec<C> = hands.iter().flatten().copied().chain(skat).collect();
        in_play.sort();
        let deck = in_play.clone();
        let log = vec![Event::Dealt {
            forehand,
            hands: hands.clone(),
            skat,
        }];
        Game {
            forehand,
            hands,
            skat,
            deck,
            editions: no_editions(),
            in_play,
            declarer_cards,
            declaration: None,
            phase: Phase::Declaring,
            trick_lead: forehand,
            trick: Vec::with_capacity(3),
            tricks: Vec::with_capacity(TRICKS_PER_GAME),
            card_points: [0; 3],
            conceded: None,
            defenders_open: false,
            drawn: None,
            returned: None,
            bonus_points: [0; 3],
            log,
        }
    }

    /// The 32 cards in the edition of each seat (ADR 0015); empty lists
    /// for a game without editions.
    #[must_use]
    pub fn editions(&self) -> &[Vec<C>; 3] {
        &self.editions
    }

    /// `card` in the edition of `seat`: a card always shows the edition of
    /// its holder (ADR 0015). Without editions `card` itself.
    #[must_use]
    pub fn in_edition_of(&self, card: C, seat: Seat) -> C {
        resolve_in(&self.editions, card, seat)
    }

    /// The copy of `card` the game holds now (hand, skat, trick: the
    /// edition of its holder), found by instance; `card` itself if it is
    /// not in play. For cards a run remembers across a change of holder
    /// ("Der Kiebitz", "Durchsichtig").
    #[must_use]
    pub fn current(&self, card: C) -> C {
        match card.instance() {
            Some(id) => self
                .deck
                .iter()
                .copied()
                .find(|c| c.instance() == Some(id))
                .unwrap_or(card),
            None => card,
        }
    }

    /// Tests and fixture tools: give a game dealt without editions (a save
    /// from before ADR 0015) the editions `editions` and every card the
    /// copy of the seat that holds it, played it, or for the skat the
    /// declarer. A card "Blue Curaçao" or a settled Wandelkarte keeps its suit.
    #[cfg(test)]
    pub(crate) fn adopt_editions(&mut self, mut editions: [Vec<C>; 3]) {
        for edition in &mut editions {
            edition.sort();
        }
        self.editions = editions;
        let resolve = |game: &Game<C>, card: C, seat: Seat| -> C {
            let new = game.in_edition_of(card, seat);
            match card.face() {
                Some(face) if new.face() != Some(face) => new.with_suit(face.suit),
                _ => new,
            }
        };
        let mut placed: Vec<(C, C)> = Vec::new();
        for seat in Seat::ALL {
            let hand: Vec<C> = self.hands[seat.index()]
                .iter()
                .map(|&c| resolve(self, c, seat))
                .collect();
            placed.extend(
                self.hands[seat.index()]
                    .iter()
                    .copied()
                    .zip(hand.iter().copied()),
            );
            self.hands[seat.index()] = hand;
        }
        let skat = self.skat.map(|c| resolve(self, c, Seat::DECLARER));
        placed.extend(self.skat.iter().copied().zip(skat));
        self.skat = skat;
        let lead = self.trick_lead;
        let trick: Vec<C> = self
            .trick
            .iter()
            .enumerate()
            .map(|(i, &c)| resolve(self, c, seat_at(lead, i)))
            .collect();
        placed.extend(self.trick.iter().copied().zip(trick.iter().copied()));
        self.trick = trick;
        let mut tricks = self.tricks.clone();
        for (trick, old) in tricks.iter_mut().zip(&self.tricks) {
            for i in 0..3 {
                trick.cards[i] = resolve(self, old.cards[i], old.seat_of(i));
                placed.push((old.cards[i], trick.cards[i]));
            }
        }
        self.tricks = tricks;
        let find = |c: C| -> C {
            placed
                .iter()
                .find(|(old, _)| old.instance() == c.instance())
                .map_or(c, |(_, new)| *new)
        };
        for list in [&mut self.deck, &mut self.in_play, &mut self.declarer_cards] {
            for c in list.iter_mut() {
                *c = find(*c);
            }
            list.sort();
        }
        self.drawn = self.drawn.map(|(s, c)| (s, find(c)));
        self.returned = self.returned.map(|(s, c)| (s, find(c)));
    }

    /// The one place where a card changes its holder (ADR 0015, DESIGN
    /// 18.1): `card` goes to `to` in `to`'s edition. Replaces it in the
    /// deck, the cards in play and the declarer's twelve cards; the caller
    /// moves it between hands. Logs [`Event::CardChangedHands`] if the
    /// edition changes the card. Returns the card as `to` holds it.
    fn change_holder(&mut self, card: C, to: Seat, events: &mut Vec<Event<C>>) -> C {
        let new = self.in_edition_of(card, to);
        if new == card {
            return card;
        }
        for list in [&mut self.deck, &mut self.in_play, &mut self.declarer_cards] {
            for c in list.iter_mut().filter(|c| **c == card) {
                *c = new;
            }
            list.sort();
        }
        events.push(Event::CardChangedHands {
            seat: to,
            old: card,
            new,
        });
        new
    }

    /// Current phase.
    #[must_use]
    pub fn phase(&self) -> Phase {
        self.phase
    }

    /// Alias of [`Game::phase`] for readers who think in states.
    #[must_use]
    pub fn state(&self) -> Phase {
        self.phase
    }

    /// Seat that leads the first trick.
    #[must_use]
    pub fn forehand(&self) -> Seat {
        self.forehand
    }

    /// Table position of a seat in this game.
    #[must_use]
    pub fn position(&self, seat: Seat) -> Position {
        if seat == self.forehand {
            Position::Forehand
        } else if seat == self.forehand.next() {
            Position::Middlehand
        } else {
            Position::Rearhand
        }
    }

    /// Hand of a seat.
    #[must_use]
    pub fn hand(&self, seat: Seat) -> &[C] {
        &self.hands[seat.index()]
    }

    /// The skat: untouched in hand games, the discards otherwise.
    #[must_use]
    pub fn skat(&self) -> [C; 2] {
        self.skat
    }

    /// The 32 dealt cards (hands plus skat), sorted.
    #[must_use]
    pub fn cards_in_play(&self) -> &[C] {
        &self.in_play
    }

    /// The declarer's original ten cards plus the skat: the twelve cards
    /// the Spitzen are counted over. With "Drei drücken" the card drawn
    /// from a defender joins them while discarding (thirteen), and after
    /// the discard they are the ten cards kept plus the skat.
    #[must_use]
    pub fn declarer_cards(&self) -> &[C] {
        &self.declarer_cards
    }

    /// The card the declarer drew from a defender with the skat and who
    /// held it ("Drei drücken"), once drawn.
    #[must_use]
    pub fn drawn(&self) -> Option<(Seat, C)> {
        self.drawn
    }

    /// The discard that went back to a defender ("Drei drücken"), once
    /// discarded.
    #[must_use]
    pub fn returned(&self) -> Option<(Seat, C)> {
        self.returned
    }

    /// Cards the declarer must discard now: two, three after drawing a
    /// card with the skat ("Drei drücken"), none outside
    /// [`Phase::Discarding`].
    #[must_use]
    pub fn discard_count(&self) -> usize {
        match (self.phase, self.drawn) {
            (Phase::Discarding, Some(_)) => 3,
            (Phase::Discarding, None) => 2,
            _ => 0,
        }
    }

    /// The declaration, once made.
    #[must_use]
    pub fn declaration(&self) -> Option<Declaration> {
        self.declaration
    }

    /// Whether the declarer picked up the skat: while discarding, while
    /// declaring after it, and in every game that is not a hand game.
    #[must_use]
    pub fn skat_picked_up(&self) -> bool {
        match self.phase {
            Phase::Discarding | Phase::DeclaringAfterSkat => true,
            Phase::Declaring => false,
            Phase::Playing | Phase::Finished(_) => self.declaration.is_some_and(|d| !d.hand()),
        }
    }

    /// Completed tricks in order.
    #[must_use]
    pub fn tricks(&self) -> &[CompletedTrick<C>] {
        &self.tricks
    }

    /// Cards of the current, incomplete trick in play order.
    #[must_use]
    pub fn current_trick(&self) -> &[C] {
        &self.trick
    }

    /// Seat that led the current trick (or will lead the next one).
    #[must_use]
    pub fn trick_lead(&self) -> Seat {
        self.trick_lead
    }

    /// Augen each seat has collected in tricks (skat not included).
    #[must_use]
    pub fn trick_card_points(&self) -> [u32; 3] {
        self.card_points
    }

    /// Extra Augen per seat from card effects ("Erster Stich").
    #[must_use]
    pub fn bonus_points(&self) -> [u32; 3] {
        self.bonus_points
    }

    /// Give `seat` extra Augen ("Erster Stich": the Augen of the first
    /// trick once more). They count in [`Game::seat_card_points`].
    pub fn add_bonus_points(&mut self, seat: Seat, points: u32) {
        let total = &mut self.bonus_points[seat.index()];
        *total = total.saturating_add(points);
    }

    /// Whether the defenders conceded the game.
    #[must_use]
    pub fn conceded(&self) -> bool {
        self.conceded.is_some()
    }

    /// Seat that currently holds `card`, if it is in a hand.
    #[must_use]
    pub fn holder_of(&self, card: C) -> Option<Seat> {
        Seat::ALL
            .into_iter()
            .find(|seat| self.hands[seat.index()].contains(&card))
    }

    /// The 32 cards of this game, sorted, each in the edition of its
    /// current holder (played cards in the edition of who played them).
    #[must_use]
    pub fn deck_cards(&self) -> &[C] {
        &self.deck
    }

    /// Let the defenders see each other's hands (boss "Die Zwillinge").
    pub fn open_defender_hands(&mut self) {
        self.defenders_open = true;
    }

    /// Whether the defenders see each other's hands.
    #[must_use]
    pub fn defenders_open(&self) -> bool {
        self.defenders_open
    }

    /// All events so far, oldest first.
    #[must_use]
    pub fn log(&self) -> &[Event<C>] {
        &self.log
    }

    /// Seat that has to play next while in [`Phase::Playing`].
    #[must_use]
    pub fn current_seat(&self) -> Option<Seat> {
        match self.phase {
            Phase::Playing => Some(seat_at(self.trick_lead, self.trick.len())),
            _ => None,
        }
    }

    /// The card that must be followed in the current trick, if any.
    fn lead_card(&self) -> Option<C> {
        rules::effective_lead(&self.trick)
    }

    /// Cards `seat` may play right now. Empty unless it is that seat's turn.
    #[must_use]
    pub fn legal_moves(&self, seat: Seat) -> Vec<C> {
        match (self.current_seat(), self.declaration) {
            (Some(current), Some(declaration)) if current == seat => {
                rules::legal_moves(declaration.game(), self.hand(seat), self.lead_card())
            }
            _ => Vec::new(),
        }
    }

    /// What `seat` knows during play. `None` outside [`Phase::Playing`].
    #[must_use]
    pub fn view(&self, seat: Seat) -> Option<SeatView<'_, C>> {
        if self.phase != Phase::Playing {
            return None;
        }
        let declaration = self.declaration?;
        let skat = if seat.is_declarer() && !declaration.hand() {
            Some(self.skat)
        } else {
            None
        };
        let first_lead_done = !self.tricks.is_empty() || !self.trick.is_empty();
        let declarer_hand = if declaration.ouvert() && !seat.is_declarer() && first_lead_done {
            Some(self.hand(Seat::DECLARER))
        } else {
            None
        };
        let partner_hand = match seat.partner() {
            Some(partner) if self.defenders_open => Some(self.hand(partner)),
            _ => None,
        };
        Some(SeatView {
            seat,
            declaration,
            forehand: self.forehand,
            hand: self.hand(seat),
            trick_lead: self.trick_lead,
            trick: &self.trick,
            tricks: &self.tricks,
            skat,
            declarer_hand,
            partner_hand,
            deck: &self.deck,
            editions: &self.editions,
            legal_moves: self.legal_moves(seat),
        })
    }

    /// Deal the defenders' hands and the skat again from all cards the
    /// declarer does not hold. Only before the declaration. Used by the
    /// house rule "Schieben". Every card goes in the edition of its new
    /// holder (the skat in the declarer's).
    pub fn redraw_skat(&mut self, rng: &mut Rng) -> Result<Vec<Event<C>>, GameError<C>> {
        if self.phase != Phase::Declaring {
            return Err(GameError::WrongPhase { phase: self.phase });
        }
        let mut pool: Vec<C> = self.hands[1]
            .iter()
            .chain(self.hands[2].iter())
            .copied()
            .chain(self.skat)
            .collect();
        pool.sort();
        rng.shuffle(&mut pool);
        let mut changes = Vec::new();
        let mut place = |game: &mut Game<C>, card: C, seat: Seat| -> C {
            game.change_holder(card, seat, &mut changes)
        };
        let mut first: Vec<C> = Vec::with_capacity(HAND_SIZE);
        let mut second: Vec<C> = Vec::with_capacity(HAND_SIZE);
        for (i, &card) in pool.iter().enumerate() {
            if i < HAND_SIZE {
                first.push(place(self, card, Seat::DEFENDERS[0]));
            } else if i < 2 * HAND_SIZE {
                second.push(place(self, card, Seat::DEFENDERS[1]));
            }
        }
        let skat = [
            place(self, pool[2 * HAND_SIZE], Seat::DECLARER),
            place(self, pool[2 * HAND_SIZE + 1], Seat::DECLARER),
        ];
        first.sort();
        second.sort();
        self.hands[1] = first;
        self.hands[2] = second;
        self.skat = skat;
        let mut declarer_cards = self.hands[0].clone();
        declarer_cards.extend(skat);
        self.declarer_cards = declarer_cards;
        let mut in_play: Vec<C> = self.hands.iter().flatten().copied().chain(skat).collect();
        in_play.sort();
        self.in_play = in_play;
        // A new deal of hidden cards: the changes of edition are not
        // logged one by one ([`Event::SkatRedrawn`] carries the result).
        drop(changes);
        let events = vec![Event::SkatRedrawn {
            hands: self.hands.clone(),
            skat,
        }];
        self.log.extend(events.iter().cloned());
        Ok(events)
    }

    /// Pick up the skat instead of declaring a hand game: the two cards join
    /// the declarer's hand and the game waits for the discard. Only with ten
    /// cards, before anything is declared.
    pub fn pick_up_skat(&mut self) -> Result<Vec<Event<C>>, GameError<C>> {
        if self.phase != Phase::Declaring {
            return Err(GameError::WrongPhase { phase: self.phase });
        }
        self.hands[Seat::DECLARER.index()].extend(self.skat);
        self.hands[Seat::DECLARER.index()].sort();
        self.phase = Phase::Discarding;
        let events = vec![Event::SkatPickedUp { skat: self.skat }];
        self.log.extend(events.iter().cloned());
        Ok(events)
    }

    /// Pick up the skat and draw `card` from the hand of the defender
    /// `from` as a third card (house rule "Drei drücken", DESIGN 8.1): the
    /// declarer holds thirteen cards and discards three
    /// ([`Game::discard_three`]); the lowest of them goes back to `from`.
    pub fn pick_up_skat_drawing(
        &mut self,
        from: Seat,
        card: C,
    ) -> Result<Vec<Event<C>>, GameError<C>> {
        if self.phase != Phase::Declaring {
            return Err(GameError::WrongPhase { phase: self.phase });
        }
        if from.is_declarer() || !self.hand(from).contains(&card) {
            return Err(GameError::CardNotInHand { seat: from, card });
        }
        let mut events = self.pick_up_skat()?;
        self.hands[from.index()].retain(|&c| c != card);
        let mut changed = Vec::new();
        let card = self.change_holder(card, Seat::DECLARER, &mut changed);
        let hand = &mut self.hands[Seat::DECLARER.index()];
        hand.push(card);
        hand.sort();
        self.declarer_cards.push(card);
        self.declarer_cards.sort();
        self.drawn = Some((from, card));
        changed.push(Event::CardDrawn { seat: from, card });
        self.log.extend(changed.iter().cloned());
        events.extend(changed);
        Ok(events)
    }

    /// Before the declaration: replace the card `old` of the declarer's hand
    /// by `new` for the rest of the game (consumable "Blue Curaçao" repaints its
    /// suit). The card is replaced wherever the game keeps it (hand, the
    /// declarer's twelve cards, the cards in play and the deck), so trump,
    /// following suit and Spitzen see `new`.
    pub fn chalk(&mut self, old: C, new: C) -> Result<Vec<Event<C>>, GameError<C>> {
        if !matches!(
            self.phase,
            Phase::Declaring | Phase::Discarding | Phase::DeclaringAfterSkat
        ) {
            return Err(GameError::WrongPhase { phase: self.phase });
        }
        if !self.hand(Seat::DECLARER).contains(&old) {
            return Err(GameError::CardNotInHand {
                seat: Seat::DECLARER,
                card: old,
            });
        }
        let swap = |cards: &mut Vec<C>| {
            for c in cards.iter_mut().filter(|c| **c == old) {
                *c = new;
            }
            cards.sort();
        };
        swap(&mut self.hands[Seat::DECLARER.index()]);
        swap(&mut self.declarer_cards);
        swap(&mut self.in_play);
        swap(&mut self.deck);
        // The card is repainted in every edition: should it change its
        // holder, it keeps the new suit.
        if let (Some(id), Some(face)) = (old.instance(), new.face()) {
            for edition in &mut self.editions {
                for c in edition.iter_mut().filter(|c| c.instance() == Some(id)) {
                    *c = c.with_suit(face.suit);
                }
                edition.sort();
            }
        }
        // While discarding, the picked-up skat cards are also still named
        // in the skat.
        for c in self.skat.iter_mut().filter(|c| **c == old) {
            *c = new;
        }
        if let Some((_, c)) = self.drawn.as_mut().filter(|(_, c)| *c == old) {
            *c = new;
        }
        let events = vec![Event::CardChalked { old, new }];
        self.log.extend(events.iter().cloned());
        Ok(events)
    }

    /// Replace the two discarded skat cards by `f(card)` everywhere the
    /// game holds them (the skat, the declarer's twelve cards, the cards
    /// in play, the deck), e.g. with more Augen (house rule "Skatpfleger").
    /// Only after the skat was picked up and discarded and before the
    /// first card; the skat counts for the declarer, so his Augen change
    /// and nobody else's. Returns the skat as it was.
    pub fn map_skat(&mut self, f: impl Fn(C) -> C) -> Result<[C; 2], GameError<C>> {
        let discarded = matches!(self.phase, Phase::DeclaringAfterSkat | Phase::Playing)
            && self.skat_picked_up()
            && self.tricks.is_empty()
            && self.trick.is_empty();
        if !discarded {
            return Err(GameError::WrongPhase { phase: self.phase });
        }
        let old = self.skat;
        for card in old {
            let new = f(card);
            let swap = |cards: &mut Vec<C>| {
                for c in cards.iter_mut().filter(|c| **c == card) {
                    *c = new;
                }
                cards.sort();
            };
            swap(&mut self.declarer_cards);
            swap(&mut self.in_play);
            swap(&mut self.deck);
            for c in self.skat.iter_mut().filter(|c| **c == card) {
                *c = new;
            }
        }
        Ok(old)
    }

    /// Announce the game. Before the skat is touched ([`Phase::Declaring`])
    /// only a hand game (or the Ramsch) may be declared; after picking it up
    /// and discarding ([`Phase::DeclaringAfterSkat`]) only a game without
    /// the Hand level: `Normal`, and since ADR 0013 also Schneider angesagt,
    /// Schwarz angesagt and Ouvert (Null: Null and Null Ouvert). The phase
    /// decides whether an announcement is played from the hand
    /// ([`Declaration::with_hand`]). Either way play begins.
    pub fn declare(&mut self, declaration: Declaration) -> Result<Vec<Event<C>>, GameError<C>> {
        let (declaration, fits) = match self.phase {
            Phase::Declaring => {
                let declaration = declaration.with_hand(true);
                (declaration, declaration.hand())
            }
            Phase::DeclaringAfterSkat => {
                let declaration = declaration.with_hand(false);
                (declaration, !declaration.hand())
            }
            phase => return Err(GameError::WrongPhase { phase }),
        };
        if !fits {
            return Err(GameError::InvalidDeclaration { declaration });
        }
        self.declaration = Some(declaration);
        self.phase = Phase::Playing;
        let mut events = vec![Event::Declared { declaration }];
        self.settle_wild_cards(&mut events);
        self.log.extend(events.iter().cloned());
        Ok(events)
    }

    /// The game starts: every Wandelkarte of the declarer's ten cards
    /// ([`PlayingCard::wild`]) takes the suit [`rules::settled_wild_suit`]
    /// of the hand for the rest of the game, a Unter keeps its suit
    /// (DESIGN 7.2). It is replaced in the hand, the deck and every
    /// edition (should it change its holder, it keeps the suit); the
    /// declarer's twelve cards and the cards in play keep the printed
    /// card, so the Spitzen stay as declared. Cards in the skat stay
    /// printed. Logs one [`Event::WildCardSettled`] per card.
    fn settle_wild_cards(&mut self, events: &mut Vec<Event<C>>) {
        let hand = &self.hands[Seat::DECLARER.index()];
        if !hand.iter().any(|c| c.wild()) {
            return;
        }
        let suit = rules::settled_wild_suit(hand);
        let wild: Vec<C> = hand.iter().copied().filter(|c| c.wild()).collect();
        for old in wild {
            let new = match old.face() {
                Some(face) if face.rank == Rank::Jack => old.settle_wild(face.suit),
                _ => old.settle_wild(suit),
            };
            let target = new.face().map(|f| f.suit);
            let swap = |cards: &mut Vec<C>| {
                for c in cards.iter_mut().filter(|c| **c == old) {
                    *c = new;
                }
                cards.sort();
            };
            swap(&mut self.hands[Seat::DECLARER.index()]);
            swap(&mut self.deck);
            if let (Some(id), Some(target)) = (old.instance(), target) {
                for edition in &mut self.editions {
                    for c in edition.iter_mut().filter(|c| c.instance() == Some(id)) {
                        *c = c.settle_wild(target);
                    }
                    edition.sort();
                }
            }
            events.push(Event::WildCardSettled { old, new });
        }
    }

    /// Put two cards from the declarer's twelve back into the skat. Then the
    /// game waits for the declaration ([`Phase::DeclaringAfterSkat`]).
    pub fn discard(&mut self, cards: [C; 2]) -> Result<Vec<Event<C>>, GameError<C>> {
        if self.phase != Phase::Discarding {
            return Err(GameError::WrongPhase { phase: self.phase });
        }
        let hand = &self.hands[Seat::DECLARER.index()];
        // After drawing a third card ("Drei drücken") three go.
        if self.drawn.is_some()
            || cards[0] == cards[1]
            || !hand.contains(&cards[0])
            || !hand.contains(&cards[1])
        {
            return Err(GameError::InvalidDiscard { cards });
        }
        if !rules::valid_discard(hand, cards) {
            let card = if cards[0].no_discard() {
                cards[0]
            } else {
                cards[1]
            };
            return Err(GameError::DiscardForbidden { card });
        }
        self.hands[Seat::DECLARER.index()].retain(|c| !cards.contains(c));
        self.skat = cards;
        // Saves from before ADR 0011 declared before picking up the skat.
        self.phase = if self.declaration.is_some() {
            Phase::Playing
        } else {
            Phase::DeclaringAfterSkat
        };
        let mut events = vec![Event::Discarded { cards }];
        if self.phase == Phase::Playing {
            self.settle_wild_cards(&mut events);
        }
        self.log.extend(events.iter().cloned());
        Ok(events)
    }

    /// Discard three of the thirteen cards after drawing a card with the
    /// skat ("Drei drücken"): the lowest of them
    /// ([`rules::returned_discard`]) goes back to the defender the card
    /// was drawn from, the other two into the skat, where they count for
    /// the declarer. The Spitzen are counted over the ten cards kept plus
    /// the skat. Then the game waits for the declaration.
    pub fn discard_three(&mut self, cards: [C; 3]) -> Result<Vec<Event<C>>, GameError<C>> {
        if self.phase != Phase::Discarding {
            return Err(GameError::WrongPhase { phase: self.phase });
        }
        let invalid = GameError::InvalidDiscard {
            cards: [cards[0], cards[1]],
        };
        let Some((from, _)) = self.drawn else {
            return Err(invalid);
        };
        let hand = &self.hands[Seat::DECLARER.index()];
        let distinct = cards[0] != cards[1] && cards[0] != cards[2] && cards[1] != cards[2];
        if !distinct || cards.iter().any(|c| !hand.contains(c)) {
            return Err(invalid);
        }
        if !rules::valid_discard_of(hand, &cards) {
            let card = cards
                .iter()
                .copied()
                .find(|c| c.no_discard())
                .unwrap_or(cards[0]);
            return Err(GameError::DiscardForbidden { card });
        }
        let back = rules::returned_discard(cards);
        let mut skat = cards.iter().copied().filter(|&c| c != back);
        let (Some(first), Some(second)) = (skat.next(), skat.next()) else {
            return Err(invalid);
        };
        let skat = [first, second];
        self.hands[Seat::DECLARER.index()].retain(|c| !cards.contains(c));
        let mut changed = Vec::new();
        let back = self.change_holder(back, from, &mut changed);
        let defender = &mut self.hands[from.index()];
        defender.push(back);
        defender.sort();
        self.skat = skat;
        let mut declarer_cards = self.hands[Seat::DECLARER.index()].clone();
        declarer_cards.extend(skat);
        declarer_cards.sort();
        self.declarer_cards = declarer_cards;
        self.returned = Some((from, back));
        self.phase = if self.declaration.is_some() {
            Phase::Playing
        } else {
            Phase::DeclaringAfterSkat
        };
        let mut events = vec![Event::Discarded { cards: skat }];
        events.extend(changed);
        events.push(Event::CardReturned {
            seat: from,
            card: back,
        });
        if self.phase == Phase::Playing {
            self.settle_wild_cards(&mut events);
        }
        self.log.extend(events.iter().cloned());
        Ok(events)
    }

    /// Play a card. Returns the events this caused: the card, possibly the
    /// completed trick, possibly the finished game.
    pub fn play(&mut self, seat: Seat, card: C) -> Result<Vec<Event<C>>, GameError<C>> {
        let (Phase::Playing, Some(declaration)) = (self.phase, self.declaration) else {
            return Err(GameError::WrongPhase { phase: self.phase });
        };
        let expected = seat_at(self.trick_lead, self.trick.len());
        if seat != expected {
            return Err(GameError::NotYourTurn { seat, expected });
        }
        if !self.hand(seat).contains(&card) {
            return Err(GameError::CardNotInHand { seat, card });
        }
        if let Some(lead) = self.lead_card() {
            if !rules::may_play(declaration.game(), self.hand(seat), Some(lead), card) {
                return Err(GameError::MustFollowSuit { card, lead });
            }
        }

        self.hands[seat.index()].retain(|&c| c != card);
        self.trick.push(card);
        let mut events = vec![Event::CardPlayed { seat, card }];

        if self.trick.len() == 3 {
            let cards = [self.trick[0], self.trick[1], self.trick[2]];
            let winner_index =
                rules::trick_winner(declaration.game(), self.trick_lead.index(), cards);
            let winner = Seat::from_index(winner_index).unwrap_or(self.trick_lead);
            let trick = CompletedTrick {
                lead: self.trick_lead,
                cards,
                winner,
            };
            let card_points = trick.card_points();
            self.card_points[winner.index()] += card_points;
            self.tricks.push(trick);
            self.trick.clear();
            self.trick_lead = winner;
            events.push(Event::TrickWon { trick, card_points });

            // Null is lost with the declarer's first trick (Skatordnung).
            let null_lost = declaration.is_null() && winner.is_declarer();
            if self.tricks.len() == TRICKS_PER_GAME || null_lost {
                events.push(self.finish(declaration));
            }
        }
        self.log.extend(events.iter().cloned());
        Ok(events)
    }

    /// "Untot" (DESIGN 7.2): `card`, which `seat` played into the last
    /// completed trick, goes back into that seat's hand, and `substitute`
    /// from the hand takes its place in the trick. The trick keeps its
    /// winner (decided with `card`); the winner's Augen change by the
    /// difference. Only while playing, between two tricks; the hand keeps
    /// its size, so every seat still plays one card per trick. Which card
    /// may rise and which substitute is drawn is the run's business.
    pub fn raise(
        &mut self,
        seat: Seat,
        card: C,
        substitute: C,
    ) -> Result<Vec<Event<C>>, GameError<C>> {
        if self.phase != Phase::Playing {
            return Err(GameError::WrongPhase { phase: self.phase });
        }
        if !self.trick.is_empty() {
            return Err(GameError::TrickInProgress);
        }
        if substitute == card || !self.hand(seat).contains(&substitute) {
            return Err(GameError::CardNotInHand {
                seat,
                card: substitute,
            });
        }
        let Some(trick) = self.tricks.last_mut() else {
            return Err(GameError::CardNotInHand { seat, card });
        };
        let Some(offset) = (0..3).find(|&i| trick.seat_of(i) == seat && trick.cards[i] == card)
        else {
            return Err(GameError::CardNotInHand { seat, card });
        };
        trick.cards[offset] = substitute;
        let winner = trick.winner.index();
        let points = &mut self.card_points[winner];
        *points = points.saturating_sub(card.card_points()) + substitute.card_points();
        let hand = &mut self.hands[seat.index()];
        hand.retain(|&c| c != substitute);
        hand.push(card);
        hand.sort();
        let events = vec![Event::CardRose {
            seat,
            card,
            substitute,
        }];
        self.log.extend(events.iter().cloned());
        Ok(events)
    }

    /// A copy of the game with every card passed through `f`: hands, skat,
    /// the editions, the cards in play, the declarer's twelve cards, the trick,
    /// the finished tricks and the cards of "Drei drücken". The log stays as it was. For an AI that should
    /// see the cards differently, never for the game itself.
    #[must_use]
    pub fn map_cards(&self, f: impl Fn(C) -> C) -> Game<C> {
        let list = |cards: &[C]| -> Vec<C> { cards.iter().map(|&c| f(c)).collect() };
        let mut game = self.clone();
        game.hands = [
            list(&self.hands[0]),
            list(&self.hands[1]),
            list(&self.hands[2]),
        ];
        game.skat = [f(self.skat[0]), f(self.skat[1])];
        game.deck = list(&self.deck);
        game.editions = [
            list(&self.editions[0]),
            list(&self.editions[1]),
            list(&self.editions[2]),
        ];
        game.in_play = list(&self.in_play);
        game.declarer_cards = list(&self.declarer_cards);
        game.trick = list(&self.trick);
        for (trick, old) in game.tricks.iter_mut().zip(&self.tricks) {
            trick.cards = [f(old.cards[0]), f(old.cards[1]), f(old.cards[2])];
        }
        game.drawn = self.drawn.map(|(seat, c)| (seat, f(c)));
        game.returned = self.returned.map(|(seat, c)| (seat, f(c)));
        game
    }

    /// The defenders give up between two tricks (house rule "Schenken").
    /// The tricks not yet played and the Augen still in the hands go to the
    /// declarer, and the game is scored.
    pub fn concede(&mut self) -> Result<Vec<Event<C>>, GameError<C>> {
        let (Phase::Playing, Some(declaration)) = (self.phase, self.declaration) else {
            return Err(GameError::WrongPhase { phase: self.phase });
        };
        if !self.trick.is_empty() {
            return Err(GameError::TrickInProgress);
        }
        let tricks = u32::try_from(TRICKS_PER_GAME.saturating_sub(self.tricks.len())).unwrap_or(0);
        let card_points: i64 = self.hands.iter().flatten().map(|c| c.signed_points()).sum();
        let card_points = u32::try_from(card_points.max(0)).unwrap_or(0);
        self.conceded = Some((tricks, card_points));
        let events = vec![
            Event::Conceded {
                tricks,
                card_points,
            },
            self.finish(declaration),
        ];
        self.log.extend(events.iter().cloned());
        Ok(events)
    }

    /// The two defenders swap their hands between two tricks (boss "Der
    /// Revolutionär").
    pub fn swap_defender_hands(&mut self) -> Result<Vec<Event<C>>, GameError<C>> {
        if self.phase != Phase::Playing {
            return Err(GameError::WrongPhase { phase: self.phase });
        }
        if !self.trick.is_empty() {
            return Err(GameError::TrickInProgress);
        }
        self.hands.swap(1, 2);
        // The editions stay with the seats (ADR 0015, DESIGN 7.1 rule 4).
        let mut events = Vec::new();
        for seat in Seat::DEFENDERS {
            let hand = std::mem::take(&mut self.hands[seat.index()]);
            let mut moved: Vec<C> = hand
                .into_iter()
                .map(|card| self.change_holder(card, seat, &mut events))
                .collect();
            moved.sort();
            self.hands[seat.index()] = moved;
        }
        events.push(Event::DefenderHandsSwapped {
            hands: self.hands.clone(),
        });
        self.log.extend(events.iter().cloned());
        Ok(events)
    }

    /// In a hand game before the declarer's first card: take `took` from
    /// the untouched skat into the declarer's hand and put `gave` from the
    /// hand into the skat (figure "Die Oma"). The game stays a hand game;
    /// the Spitzen were counted over hand and skat anyway.
    pub fn swap_with_skat(&mut self, took: C, gave: C) -> Result<Vec<Event<C>>, GameError<C>> {
        let (Phase::Playing, Some(declaration)) = (self.phase, self.declaration) else {
            return Err(GameError::WrongPhase { phase: self.phase });
        };
        let started = self.hand(Seat::DECLARER).len() < HAND_SIZE;
        if !declaration.hand() || declaration.is_ramsch() || started {
            return Err(GameError::InvalidSkatSwap);
        }
        let Some(slot) = self.skat.iter().position(|&c| c == took) else {
            return Err(GameError::InvalidSkatSwap);
        };
        if !self.hand(Seat::DECLARER).contains(&gave) {
            return Err(GameError::CardNotInHand {
                seat: Seat::DECLARER,
                card: gave,
            });
        }
        // What goes into the skat is discarded: "Ramponiert" only if every
        // card of the hand is.
        if gave.no_discard() && self.hand(Seat::DECLARER).iter().any(|c| !c.no_discard()) {
            return Err(GameError::DiscardForbidden { card: gave });
        }
        let hand = &mut self.hands[Seat::DECLARER.index()];
        hand.retain(|&c| c != gave);
        hand.push(took);
        hand.sort();
        self.skat[slot] = gave;
        let events = vec![Event::SkatSwapped { took, gave }];
        self.log.extend(events.iter().cloned());
        Ok(events)
    }

    /// Score the game by the Skatordnung and switch to `Finished`.
    fn finish(&mut self, declaration: Declaration) -> Event<C> {
        let outcome = self.outcome();
        let matadors = self.matadors().unwrap_or(Matadors {
            with: false,
            count: 0,
        });
        let value = if declaration.is_ramsch() {
            rules::ramsch_value_parts(self.seat_card_points(), self.seat_tricks()).value
        } else {
            rules::game_value(declaration, matadors, &outcome)
        };
        self.phase = Phase::Finished(value);
        Event::GameFinished {
            outcome,
            matadors,
            value,
            seats: self.seat_totals(),
        }
    }

    /// Spitzen of the declarer over hand plus skat, counted along the
    /// trumps in play. `None` before the declaration.
    #[must_use]
    pub fn matadors(&self) -> Option<Matadors> {
        let declaration = self.declaration?;
        if declaration.is_ramsch() {
            return Some(Matadors {
                with: false,
                count: 0,
            });
        }
        Some(rules::matadors_in(
            declaration.game(),
            &self.declarer_cards,
            &self.in_play,
        ))
    }

    /// Augen per seat so far: tricks, the skat and conceded Augen. The skat
    /// counts for the declarer; in a Ramsch it goes to the seat that took
    /// the last trick (nobody's until then).
    ///
    /// Augen below zero ([`PlayingCard::point_debt`], DESIGN 7.2) count
    /// against the seat that took them; a seat never has fewer than 0.
    /// Extra Augen from card effects ([`Game::bonus_points`]) count too.
    #[must_use]
    pub fn seat_card_points(&self) -> [u32; 3] {
        let mut points: [i64; 3] = self.bonus_points.map(i64::from);
        for trick in &self.tricks {
            points[trick.winner.index()] += trick.signed_points();
        }
        let skat_points: i64 = self.skat.iter().map(|c| c.signed_points()).sum();
        let ramsch = self.declaration.is_some_and(Declaration::is_ramsch);
        if ramsch {
            if self.tricks.len() == TRICKS_PER_GAME {
                if let Some(last) = self.tricks.last() {
                    points[last.winner.index()] += skat_points;
                }
            }
        } else {
            points[0] += skat_points;
        }
        if let Some((_, conceded)) = self.conceded {
            points[0] += i64::from(conceded);
        }
        points.map(|p| u32::try_from(p.max(0)).unwrap_or(u32::MAX))
    }

    /// Augen and tricks per seat so far ([`Game::seat_card_points`],
    /// [`Game::seat_tricks`]).
    #[must_use]
    pub fn seat_totals(&self) -> SeatTotals {
        SeatTotals {
            card_points: self.seat_card_points(),
            tricks: self.seat_tricks(),
        }
    }

    /// Tricks per seat so far, conceded tricks included.
    #[must_use]
    pub fn seat_tricks(&self) -> [u32; 3] {
        let mut tricks = [0; 3];
        for trick in &self.tricks {
            tricks[trick.winner.index()] += 1;
        }
        if let Some((conceded, _)) = self.conceded {
            tricks[0] += conceded;
        }
        tricks
    }

    /// Style of a Null game so far (DESIGN 9.2, see
    /// [`crate::rules::NullFormula`]): tricks the declarer did not take
    /// although he played a seven in them, whether he never followed suit
    /// to another seat's lead, and the defenders' Augen after
    /// `short_circuit_tricks` tricks.
    #[must_use]
    pub fn null_style(&self, short_circuit_tricks: usize) -> NullStyle {
        let game = self
            .declaration
            .map_or(rules::GameType::Null, Declaration::game);
        let mut style = NullStyle {
            never_followed: true,
            ..NullStyle::default()
        };
        let mut defender_points = 0;
        for (i, trick) in self.tricks.iter().enumerate() {
            let offset = (3 + Seat::DECLARER.index() - trick.lead.index()) % 3;
            let own = trick.cards[offset];
            let seven = own
                .face()
                .is_some_and(|f| f.rank == crate::cards::Rank::Seven);
            if seven && !trick.winner.is_declarer() {
                style.sevens_ducked += 1;
            }
            if offset > 0 {
                if let Some(lead) = rules::effective_lead(&trick.cards[..offset]) {
                    if rules::follows_lead(game, lead, own) {
                        style.never_followed = false;
                    }
                }
            }
            if !trick.winner.is_declarer() {
                defender_points += trick.card_points();
            }
            if i + 1 == short_circuit_tricks {
                style.defender_points_after = Some(defender_points);
            }
        }
        style
    }

    /// Trick outcome so far. The skat counts for the declarer (in a Ramsch
    /// for the seat that took the last trick), and so does everything the
    /// defenders conceded.
    #[must_use]
    pub fn outcome(&self) -> Outcome {
        let points = self.seat_card_points();
        let tricks = self.seat_tricks();
        Outcome {
            declarer_card_points: points[0],
            defender_card_points: points[1] + points[2],
            declarer_tricks: tricks[0],
            defender_tricks: tricks[1] + tricks[2],
        }
    }
    /// The final value once the game is finished.
    #[must_use]
    pub fn value(&self) -> Option<GameValue> {
        match self.phase {
            Phase::Finished(v) => Some(v),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cards::{Suit, parse_cards};
    use crate::rules::{NullRisk, RiskLevel};

    fn c(s: &str) -> Card {
        s.parse().expect("test card")
    }

    fn fixed_game() -> Game {
        let hands = [
            parse_cards("CJ SJ HA HT HK HQ H9 SA ST D7").expect("hand"),
            parse_cards("HJ DJ H8 H7 SK SQ S9 CA CT D8").expect("hand"),
            parse_cards("S8 S7 CK CQ C9 C8 C7 DA DT DK").expect("hand"),
        ];
        let skat = [c("DQ"), c("D9")];
        Game::with_cards(hands, skat, Seat::DECLARER).expect("valid deal")
    }

    #[test]
    fn deal_gives_ten_cards_each_and_two_in_skat() {
        let game = Game::deal(&mut Rng::from_seed(1), Seat(1));
        for seat in Seat::ALL {
            assert_eq!(game.hand(seat).len(), 10);
        }
        assert_eq!(game.phase(), Phase::Declaring);
        assert_eq!(game.declarer_cards().len(), 12);
        assert!(matches!(
            game.log()[0],
            Event::Dealt {
                forehand: Seat(1),
                ..
            }
        ));
        assert_eq!(game.position(Seat(1)), Position::Forehand);
        assert_eq!(game.position(Seat(2)), Position::Middlehand);
        assert_eq!(game.position(Seat(0)), Position::Rearhand);
    }

    #[test]
    fn with_cards_rejects_bad_deals() {
        let hands = [
            parse_cards("CJ SJ HA HT HK HQ H9 SA ST D7").expect("hand"),
            parse_cards("HJ DJ H8 H7 SK SQ S9 CA CT D8").expect("hand"),
            parse_cards("S8 S7 CK CQ C9 C8 C7 DA DT CJ").expect("hand"),
        ];
        assert_eq!(
            Game::with_cards(hands, [c("DQ"), c("D9")], Seat::DECLARER),
            Err(GameError::InvalidDeal)
        );
    }

    #[test]
    fn actions_outside_their_phase_fail() {
        let mut game = fixed_game();
        assert!(matches!(
            game.play(Seat::DECLARER, c("CJ")),
            Err(GameError::WrongPhase { .. })
        ));
        assert!(matches!(
            game.discard([c("D7"), c("DQ")]),
            Err(GameError::WrongPhase { .. })
        ));
        assert_eq!(game.current_seat(), None);
        assert!(game.view(Seat::DECLARER).is_none());
    }

    #[test]
    fn skat_is_picked_up_and_discarded_before_the_declaration() {
        let mut game = fixed_game();
        let decl = Declaration::suit(Suit::Hearts, RiskLevel::Normal);
        // Before the skat only hand games may be declared.
        assert_eq!(
            game.declare(decl),
            Err(GameError::InvalidDeclaration { declaration: decl })
        );
        assert!(!game.skat_picked_up());
        let events = game.pick_up_skat().expect("pick up");
        assert_eq!(events, vec![Event::SkatPickedUp { skat: game.skat() }]);
        assert_eq!(game.phase(), Phase::Discarding);
        assert!(game.skat_picked_up());
        assert_eq!(game.hand(Seat::DECLARER).len(), 12);
        assert!(matches!(
            game.pick_up_skat(),
            Err(GameError::WrongPhase { .. })
        ));
        assert!(matches!(
            game.declare(decl),
            Err(GameError::WrongPhase { .. })
        ));
        assert!(matches!(
            game.discard([c("D7"), c("D7")]),
            Err(GameError::InvalidDiscard { .. })
        ));
        assert!(matches!(
            game.discard([c("D7"), c("CA")]),
            Err(GameError::InvalidDiscard { .. })
        ));
        game.discard([c("D7"), c("DQ")]).expect("discard");
        assert_eq!(game.phase(), Phase::DeclaringAfterSkat);
        assert_eq!(game.declaration(), None);
        assert_eq!(game.skat(), [c("D7"), c("DQ")]);
        assert_eq!(game.hand(Seat::DECLARER).len(), 10);
        assert!(game.hand(Seat::DECLARER).contains(&c("D9")));
        // After the skat no Hand level. Schneider angesagt, Schwarz angesagt
        // and Ouvert are fine since ADR 0013, played without Hand; so is
        // Null Ouvert.
        let hand = Declaration::suit(Suit::Hearts, RiskLevel::Hand);
        assert_eq!(
            game.clone().declare(hand),
            Err(GameError::InvalidDeclaration { declaration: hand })
        );
        for risk in [
            RiskLevel::SchneiderDeclared,
            RiskLevel::SchwarzDeclared,
            RiskLevel::Ouvert,
        ] {
            let mut probe = game.clone();
            let events = probe
                .declare(Declaration::suit(Suit::Hearts, risk))
                .expect("announcement after the skat");
            let declared = Declaration::suit(Suit::Hearts, risk).with_hand(false);
            assert!(!declared.hand());
            assert_eq!(
                events,
                vec![Event::Declared {
                    declaration: declared
                }]
            );
            assert!(probe.skat_picked_up());
        }
        for null in [NullRisk::Hand, NullRisk::OuvertHand] {
            assert!(game.clone().declare(Declaration::null(null)).is_err());
        }
        assert!(
            game.clone()
                .declare(Declaration::null(NullRisk::Ouvert))
                .is_ok()
        );
        let events = game.declare(decl).expect("declare");
        assert_eq!(events, vec![Event::Declared { declaration: decl }]);
        assert_eq!(game.phase(), Phase::Playing);
        assert!(game.skat_picked_up());
        let order: Vec<&str> = game
            .log()
            .iter()
            .map(|e| match e {
                Event::Dealt { .. } => "dealt",
                Event::SkatPickedUp { .. } => "picked up",
                Event::Discarded { .. } => "discarded",
                Event::Declared { .. } => "declared",
                _ => "other",
            })
            .collect();
        assert_eq!(order, ["dealt", "picked up", "discarded", "declared"]);
        assert_eq!(
            game.view(Seat::DECLARER).and_then(|v| v.skat),
            Some([c("D7"), c("DQ")])
        );
        assert_eq!(game.view(Seat(1)).and_then(|v| v.skat), None);
    }

    /// A save from before ADR 0011: declared first, then the skat was picked
    /// up. The discard goes straight to playing.
    #[test]
    fn old_saves_declared_before_the_skat_play_on_after_the_discard() {
        let mut game = fixed_game();
        game.pick_up_skat().expect("pick up");
        let decl = Declaration::suit(Suit::Hearts, RiskLevel::Normal);
        game.declaration = Some(decl);
        let saved = ron::to_string(&game).expect("save");
        let mut loaded: Game = ron::from_str(&saved).expect("load");
        loaded.discard([c("D7"), c("DQ")]).expect("discard");
        assert_eq!(loaded.phase(), Phase::Playing);
        assert_eq!(loaded.declaration(), Some(decl));
        assert!(loaded.skat_picked_up());
        assert_eq!(loaded.current_seat(), Some(Seat::DECLARER));
    }

    #[test]
    fn a_chalked_card_follows_its_new_suit() {
        use crate::cards::CardKind;
        use crate::deck::DealtCard;
        let base = fixed_game();
        let mut id = 0;
        let mut dealt = |cards: &[Card]| -> Vec<DealtCard> {
            cards
                .iter()
                .map(|&card| {
                    id += 1;
                    DealtCard::plain(id, CardKind::Normal(card))
                })
                .collect()
        };
        let hands = [
            dealt(base.hand(Seat(0))),
            dealt(base.hand(Seat(1))),
            dealt(base.hand(Seat(2))),
        ];
        let skat = dealt(&base.skat());
        let mut game = Game::with_cards(hands, [skat[0], skat[1]], Seat(2)).expect("valid deal");
        let find = |game: &Game<DealtCard>, seat: Seat, s: &str| {
            game.hand(seat)
                .iter()
                .copied()
                .find(|d| d.kind == CardKind::Normal(c(s)))
                .expect("held")
        };
        // The declarer's only diamond becomes a spade.
        let old = find(&game, Seat::DECLARER, "D7");
        let new = DealtCard {
            kind: CardKind::Normal(c("S7")),
            ..old
        };
        assert!(matches!(
            game.chalk(find(&game, Seat(1), "D8"), new),
            Err(GameError::CardNotInHand { .. })
        ));
        let events = game.chalk(old, new).expect("blue_curacao");
        assert_eq!(events, vec![Event::CardChalked { old, new }]);
        assert!(game.hand(Seat::DECLARER).contains(&new));
        assert!(game.declarer_cards().contains(&new));
        assert!(game.cards_in_play().contains(&new) && !game.cards_in_play().contains(&old));
        assert!(game.deck_cards().contains(&new));
        game.declare(Declaration::grand(RiskLevel::Hand))
            .expect("declare");
        assert!(matches!(
            game.chalk(new, old),
            Err(GameError::WrongPhase { .. })
        ));
        // A diamond is led: the declarer has none left and may play anything.
        let lead = find(&game, Seat(2), "DA");
        game.play(Seat(2), lead).expect("lead");
        assert_eq!(game.legal_moves(Seat::DECLARER).len(), 10);
    }

    #[test]
    fn declare_hand_skips_discard_and_keeps_skat_for_declarer() {
        let mut game = fixed_game();
        let decl = Declaration::suit(Suit::Hearts, RiskLevel::Hand);
        let events = game.declare(decl).expect("declare");
        assert_eq!(events.len(), 1);
        assert_eq!(game.phase(), Phase::Playing);
        assert!(!game.skat_picked_up());
        assert_eq!(game.hand(Seat::DECLARER).len(), 10);
        assert_eq!(game.outcome().declarer_card_points, 3);
        assert_eq!(game.view(Seat::DECLARER).and_then(|v| v.skat), None);
    }

    #[test]
    fn play_enforces_turn_hand_and_following_suit() {
        let mut game = fixed_game();
        game.declare(Declaration::suit(Suit::Hearts, RiskLevel::Hand))
            .expect("declare");
        assert_eq!(game.current_seat(), Some(Seat::DECLARER));
        assert!(matches!(
            game.play(Seat(1), c("HJ")),
            Err(GameError::NotYourTurn { .. })
        ));
        assert!(matches!(
            game.play(Seat::DECLARER, c("HJ")),
            Err(GameError::CardNotInHand { .. })
        ));
        game.play(Seat::DECLARER, c("SA")).expect("lead");
        assert_eq!(game.current_seat(), Some(Seat(1)));
        // Seat 1 holds spades, so a diamond is illegal.
        assert!(matches!(
            game.play(Seat(1), c("D8")),
            Err(GameError::MustFollowSuit { .. })
        ));
        assert_eq!(
            game.legal_moves(Seat(1)),
            parse_cards("S9 SQ SK").expect("cards")
        );
        assert!(game.legal_moves(Seat(2)).is_empty());
        game.play(Seat(1), c("S9")).expect("follow");
        // Seat 2 has spades too (S7, S8).
        let events = game.play(Seat(2), c("S7")).expect("follow");
        assert_eq!(events.len(), 2);
        assert!(matches!(
            events[1],
            Event::TrickWon {
                trick: CompletedTrick {
                    winner: Seat(0),
                    ..
                },
                card_points: 11
            }
        ));
        assert_eq!(game.tricks().len(), 1);
        assert_eq!(game.trick_lead(), Seat::DECLARER);
    }

    #[test]
    fn seat_helpers() {
        assert_eq!(Seat::from_index(3), None);
        assert_eq!(Seat(2).next(), Seat(0));
        assert_eq!(Seat(1).partner(), Some(Seat(2)));
        assert_eq!(Seat(0).partner(), None);
        assert_eq!(seat_at(Seat(2), 2), Seat(1));
    }

    #[test]
    fn ouvert_shows_the_declarer_hand_to_defenders_after_the_first_lead() {
        let mut game = fixed_game();
        game.declare(Declaration::suit(Suit::Hearts, RiskLevel::Ouvert))
            .expect("declare");
        // Before the first card nothing is open.
        assert_eq!(game.view(Seat(1)).and_then(|v| v.declarer_hand), None);
        game.play(Seat::DECLARER, c("SA")).expect("lead");
        let view = game.view(Seat(1)).expect("view");
        assert_eq!(view.declarer_hand, Some(game.hand(Seat::DECLARER)));
        assert_eq!(view.declarer_hand.map(<[Card]>::len), Some(9));
        // The declarer's own view and non-ouvert games stay closed.
        game.play(Seat(1), c("S9")).expect("follow");
        game.play(Seat(2), c("S7")).expect("follow");
        assert_eq!(
            game.view(Seat::DECLARER).and_then(|v| v.declarer_hand),
            None
        );
        let mut closed = fixed_game();
        closed
            .declare(Declaration::suit(Suit::Hearts, RiskLevel::SchwarzDeclared))
            .expect("declare");
        closed.play(Seat::DECLARER, c("SA")).expect("lead");
        assert_eq!(closed.view(Seat(1)).and_then(|v| v.declarer_hand), None);
    }

    #[test]
    fn deal_from_needs_exactly_32_cards() {
        let mut cards = standard_deck();
        let extra = cards.pop().expect("card");
        let mut rng = Rng::from_seed(4);
        assert_eq!(
            Game::deal_from(&mut rng, cards.clone(), Seat::DECLARER),
            Err(GameError::InvalidDeal)
        );
        cards.push(extra);
        cards.push(extra);
        assert_eq!(
            Game::deal_from(&mut rng, cards.clone(), Seat::DECLARER),
            Err(GameError::InvalidDeal)
        );
        cards.pop();
        let game = Game::deal_from(&mut rng, cards, Seat::DECLARER).expect("deal");
        assert_eq!(game.cards_in_play().len(), 32);
    }

    /// Three editions of the 32 cards: the player's counts one Auge more on
    /// every card, the boss's (seat 1) two, the guest's (seat 2) is plain.
    fn three_editions() -> [Vec<crate::deck::DealtCard>; 3] {
        use crate::cards::CardKind;
        use crate::deck::{DealtCard, Owner};
        let edition = |owner: Owner, extra: u32| -> Vec<DealtCard> {
            standard_deck()
                .into_iter()
                .zip(0u32..)
                .map(|(card, id)| {
                    let plain = DealtCard::plain(id, CardKind::Normal(card));
                    DealtCard {
                        owner,
                        card_points: plain.card_points + extra,
                        ..plain
                    }
                })
                .collect()
        };
        [
            edition(Owner::Player, 1),
            edition(Owner::Boss, 2),
            edition(Owner::Guest, 0),
        ]
    }

    #[test]
    fn editions_deal_like_one_deck_and_resolve_every_card_by_its_holder() {
        use crate::deck::Owner;
        let owners = [Owner::Player, Owner::Boss, Owner::Guest];
        for seed in 0..20 {
            let editions = three_editions();
            let plain = Game::deal_from(
                &mut Rng::from_seed(seed),
                editions[0].clone(),
                Seat::DECLARER,
            )
            .expect("deal");
            let game = Game::deal_editions(&mut Rng::from_seed(seed), editions, Seat::DECLARER)
                .expect("deal");
            for seat in Seat::ALL {
                let ids = |g: &Game<crate::deck::DealtCard>| -> Vec<u32> {
                    g.hand(seat).iter().map(|c| c.id).collect()
                };
                assert_eq!(ids(&game), ids(&plain), "same cards to the same seat");
                assert!(
                    game.hand(seat)
                        .iter()
                        .all(|c| c.owner == owners[seat.index()])
                );
            }
            // The skat is the declarer's (ADR 0015).
            assert_eq!(game.skat().map(|c| c.id), plain.skat().map(|c| c.id));
            assert!(game.skat().iter().all(|c| c.owner == Owner::Player));
            assert_eq!(game.deck_cards().len(), 32);
        }
    }

    #[test]
    fn a_card_that_changes_hands_takes_the_edition_of_its_new_holder() {
        use crate::deck::Owner;
        let mut game =
            Game::deal_editions(&mut Rng::from_seed(3), three_editions(), Seat::DECLARER)
                .expect("deal");
        // "Drei drücken": the drawn card becomes the player's.
        let card = game.hand(Seat(1))[0];
        assert_eq!(card.owner, Owner::Boss);
        let events = game.pick_up_skat_drawing(Seat(1), card).expect("draw");
        let (_, drawn) = game.drawn().expect("drawn");
        assert_eq!((drawn.id, drawn.owner), (card.id, Owner::Player));
        assert_eq!(drawn.card_points, card.card_points - 1);
        assert!(events.contains(&Event::CardChangedHands {
            seat: Seat::DECLARER,
            old: card,
            new: drawn,
        }));
        assert!(game.declarer_cards().contains(&drawn));
        assert!(game.deck_cards().contains(&drawn) && !game.deck_cards().contains(&card));
        // The card given back is the boss's again.
        let hand = game.hand(Seat::DECLARER).to_vec();
        game.discard_three([hand[0], hand[1], hand[2]])
            .expect("discard three");
        let (seat, back) = game.returned().expect("returned");
        assert_eq!((seat, back.owner), (Seat(1), Owner::Boss));
        assert!(game.hand(Seat(1)).contains(&back));
        assert!(game.skat().iter().all(|c| c.owner == Owner::Player));
        game.declare(Declaration::suit(Suit::Hearts, RiskLevel::Normal))
            .expect("declare");
        // "Der Revolutionär": the editions stay with the seats.
        game.swap_defender_hands().expect("swap");
        assert!(game.hand(Seat(1)).iter().all(|c| c.owner == Owner::Boss));
        assert!(game.hand(Seat(2)).iter().all(|c| c.owner == Owner::Guest));
        for seat in Seat::ALL {
            for &c in game.hand(seat) {
                assert_eq!(
                    game.in_edition_of(c, seat),
                    c,
                    "every card in its holder's edition"
                );
                assert!(game.deck_cards().contains(&c));
            }
        }
    }

    #[test]
    fn concede_gives_the_rest_to_the_declarer() {
        let mut game = fixed_game();
        game.declare(Declaration::suit(Suit::Hearts, RiskLevel::Hand))
            .expect("declare");
        game.play(Seat::DECLARER, c("SA")).expect("lead");
        assert_eq!(game.concede(), Err(GameError::TrickInProgress));
        game.play(Seat(1), c("S9")).expect("follow");
        game.play(Seat(2), c("S7")).expect("follow");
        let events = game.concede().expect("concede");
        assert!(matches!(events[0], Event::Conceded { tricks: 9, .. }));
        assert!(game.conceded());
        let outcome = game.outcome();
        assert_eq!(
            outcome.declarer_card_points + outcome.defender_card_points,
            120
        );
        assert_eq!(outcome.declarer_tricks, 10);
        assert!(game.value().is_some_and(|v| v.won && v.schwarz));
    }

    #[test]
    fn redraw_skat_keeps_the_declarer_hand() {
        let mut game = fixed_game();
        let hand = game.hand(Seat::DECLARER).to_vec();
        let mut rng = Rng::from_seed(9);
        game.redraw_skat(&mut rng).expect("redraw");
        assert_eq!(game.hand(Seat::DECLARER), &hand[..]);
        assert_eq!(game.hand(Seat(1)).len(), 10);
        assert_eq!(game.hand(Seat(2)).len(), 10);
        assert_eq!(game.declarer_cards().len(), 12);
        game.declare(Declaration::grand(RiskLevel::Hand))
            .expect("declare");
        assert!(matches!(
            game.redraw_skat(&mut rng),
            Err(GameError::WrongPhase { .. })
        ));
    }
}

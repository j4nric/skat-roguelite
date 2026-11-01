//! State machine for a single Skat game: deal → declare → (discard) → ten
//! tricks → scored.
//!
//! The game is generic over the card type ([`PlayingCard`]): `Game<Card>`
//! (the default) plays the plain 32-card deck of milestone M1, while
//! `Game<DealtCard>` plays card instances of the shared roguelike deck with
//! blanks and modifications (see [`crate::deck`]).
//!
//! Every state change is recorded as an [`Event`]; frontends animate them,
//! the simulator ignores them. Illegal actions return a [`GameError`] and
//! leave the state untouched. Nothing in here panics on bad input.

use std::fmt;

use serde::{Deserialize, Serialize};

use crate::cards::{Card, PlayingCard, standard_deck};
use crate::rng::Rng;
use crate::rules::{self, Declaration, GameValue, Matadors, NullStyle, Outcome, SeatTotals};

/// Cards each seat receives when dealing.
pub const HAND_SIZE: usize = 10;
/// Tricks in a game.
pub const TRICKS_PER_GAME: usize = 10;
/// Cards dealt into one game: three hands plus the skat. A larger deck
/// leaves the rest in the talon.
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Phase {
    /// The declarer looks at ten cards and chooses the declaration.
    Declaring,
    /// The declarer holds twelve cards and must discard two.
    Discarding,
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
    /// Every card that may be in this game: the dealt cards plus the talon,
    /// sorted. The deck is shared and known to all seats, but nobody knows
    /// which cards lie in the talon.
    pub deck: &'a [C],
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
}

/// A single Skat game.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Game<C = Card> {
    forehand: Seat,
    hands: [Vec<C>; 3],
    skat: [C; 2],
    /// Cards that were not dealt (roguelike deck with more than 32 cards).
    talon: Vec<C>,
    /// Dealt cards plus talon, sorted: what any seat may expect to see.
    deck: Vec<C>,
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
    log: Vec<Event<C>>,
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
        Game::build(hands, skat, Vec::new(), forehand)
    }
}

impl<C: PlayingCard> Game<C> {
    /// Shuffle `cards` with `rng` and deal 10/10/10/2. Cards beyond the
    /// first 32 stay in the talon for this game. Fails with fewer than 32
    /// cards or if a card appears twice.
    pub fn deal_from(
        rng: &mut Rng,
        mut cards: Vec<C>,
        forehand: Seat,
    ) -> Result<Game<C>, GameError<C>> {
        if cards.len() < CARDS_IN_PLAY {
            return Err(GameError::InvalidDeal);
        }
        rng.shuffle(&mut cards);
        let talon = cards.split_off(CARDS_IN_PLAY);
        let hands = [
            cards[0..HAND_SIZE].to_vec(),
            cards[HAND_SIZE..2 * HAND_SIZE].to_vec(),
            cards[2 * HAND_SIZE..3 * HAND_SIZE].to_vec(),
        ];
        let skat = [cards[3 * HAND_SIZE], cards[3 * HAND_SIZE + 1]];
        Game::with_talon(hands, skat, talon, forehand)
    }

    /// Start a game from explicit cards (tests, replays, frontends with a
    /// modified deck). Hands are sorted so that equal deals compare equal.
    pub fn with_cards(
        hands: [Vec<C>; 3],
        skat: [C; 2],
        forehand: Seat,
    ) -> Result<Game<C>, GameError<C>> {
        Game::with_talon(hands, skat, Vec::new(), forehand)
    }

    /// Like [`Game::with_cards`], with extra cards in the talon that are
    /// known to exist but are not dealt.
    pub fn with_talon(
        hands: [Vec<C>; 3],
        skat: [C; 2],
        talon: Vec<C>,
        forehand: Seat,
    ) -> Result<Game<C>, GameError<C>> {
        if hands.iter().any(|h| h.len() != HAND_SIZE) {
            return Err(GameError::InvalidDeal);
        }
        let mut all: Vec<C> = hands
            .iter()
            .flatten()
            .copied()
            .chain(skat)
            .chain(talon.iter().copied())
            .collect();
        let total = all.len();
        all.sort();
        all.dedup();
        if all.len() != total {
            return Err(GameError::InvalidDeal);
        }
        Ok(Game::build(hands, skat, talon, forehand))
    }

    /// Build the initial state from cards that are known to be valid.
    fn build(mut hands: [Vec<C>; 3], skat: [C; 2], mut talon: Vec<C>, forehand: Seat) -> Game<C> {
        for hand in &mut hands {
            hand.sort();
        }
        talon.sort();
        let mut declarer_cards = hands[0].clone();
        declarer_cards.extend(skat);
        let mut in_play: Vec<C> = hands.iter().flatten().copied().chain(skat).collect();
        in_play.sort();
        let mut deck = in_play.clone();
        deck.extend(talon.iter().copied());
        deck.sort();
        let log = vec![Event::Dealt {
            forehand,
            hands: hands.clone(),
            skat,
        }];
        Game {
            forehand,
            hands,
            skat,
            talon,
            deck,
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
            log,
        }
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

    /// Cards of the deck that were not dealt into this game.
    #[must_use]
    pub fn talon(&self) -> &[C] {
        &self.talon
    }

    /// The 32 dealt cards (hands plus skat), sorted.
    #[must_use]
    pub fn cards_in_play(&self) -> &[C] {
        &self.in_play
    }

    /// The declarer's original ten cards plus the skat.
    #[must_use]
    pub fn declarer_cards(&self) -> &[C] {
        &self.declarer_cards
    }

    /// The declaration, once made.
    #[must_use]
    pub fn declaration(&self) -> Option<Declaration> {
        self.declaration
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

    /// Every card that may be in this game, the talon included, sorted.
    /// The deck is shared and public; which cards lie in the talon is not.
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
            legal_moves: self.legal_moves(seat),
        })
    }

    /// Deal the defenders' hands and the skat again from all cards the
    /// declarer does not hold (including the talon). Only before the
    /// declaration. Used by the house rule "Schieben".
    pub fn redraw_skat(&mut self, rng: &mut Rng) -> Result<Vec<Event<C>>, GameError<C>> {
        if self.phase != Phase::Declaring {
            return Err(GameError::WrongPhase { phase: self.phase });
        }
        let mut pool: Vec<C> = self.hands[1]
            .iter()
            .chain(self.hands[2].iter())
            .copied()
            .chain(self.skat)
            .chain(self.talon.iter().copied())
            .collect();
        pool.sort();
        rng.shuffle(&mut pool);
        let mut talon = pool.split_off(2 * HAND_SIZE + 2);
        talon.sort();
        let mut first = pool[0..HAND_SIZE].to_vec();
        let mut second = pool[HAND_SIZE..2 * HAND_SIZE].to_vec();
        first.sort();
        second.sort();
        let skat = [pool[2 * HAND_SIZE], pool[2 * HAND_SIZE + 1]];
        self.hands[1] = first;
        self.hands[2] = second;
        self.skat = skat;
        self.talon = talon;
        let mut declarer_cards = self.hands[0].clone();
        declarer_cards.extend(skat);
        self.declarer_cards = declarer_cards;
        let mut in_play: Vec<C> = self.hands.iter().flatten().copied().chain(skat).collect();
        in_play.sort();
        self.in_play = in_play;
        let events = vec![Event::SkatRedrawn {
            hands: self.hands.clone(),
            skat,
        }];
        self.log.extend(events.iter().cloned());
        Ok(events)
    }

    /// Announce the game. Hand games go straight to playing; otherwise the
    /// skat is picked up and the game waits for the discard.
    pub fn declare(&mut self, declaration: Declaration) -> Result<Vec<Event<C>>, GameError<C>> {
        if self.phase != Phase::Declaring {
            return Err(GameError::WrongPhase { phase: self.phase });
        }
        self.declaration = Some(declaration);
        let mut events = vec![Event::Declared { declaration }];
        if declaration.hand() {
            self.phase = Phase::Playing;
        } else {
            self.hands[Seat::DECLARER.index()].extend(self.skat);
            self.hands[Seat::DECLARER.index()].sort();
            events.push(Event::SkatPickedUp { skat: self.skat });
            self.phase = Phase::Discarding;
        }
        self.log.extend(events.iter().cloned());
        Ok(events)
    }

    /// Put two cards from the declarer's twelve back into the skat.
    pub fn discard(&mut self, cards: [C; 2]) -> Result<Vec<Event<C>>, GameError<C>> {
        if self.phase != Phase::Discarding {
            return Err(GameError::WrongPhase { phase: self.phase });
        }
        let hand = &self.hands[Seat::DECLARER.index()];
        if cards[0] == cards[1] || !hand.contains(&cards[0]) || !hand.contains(&cards[1]) {
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
        self.phase = Phase::Playing;
        let events = vec![Event::Discarded { cards }];
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
            if !rules::legal_moves(declaration.game(), self.hand(seat), Some(lead)).contains(&card)
            {
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
        let card_points: u32 = self.hands.iter().flatten().map(|c| c.card_points()).sum();
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
        let events = vec![Event::DefenderHandsSwapped {
            hands: self.hands.clone(),
        }];
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
    #[must_use]
    pub fn seat_card_points(&self) -> [u32; 3] {
        let mut points = self.card_points;
        let skat_points: u32 = self.skat.iter().map(|c| c.card_points()).sum();
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
            points[0] += conceded;
        }
        points
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
    use crate::rules::RiskLevel;

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
    fn declare_normal_picks_up_skat_and_requires_discard() {
        let mut game = fixed_game();
        let decl = Declaration::suit(Suit::Hearts, RiskLevel::Normal);
        let events = game.declare(decl).expect("declare");
        assert_eq!(events.len(), 2);
        assert_eq!(game.phase(), Phase::Discarding);
        assert_eq!(game.hand(Seat::DECLARER).len(), 12);
        assert!(matches!(
            game.discard([c("D7"), c("D7")]),
            Err(GameError::InvalidDiscard { .. })
        ));
        assert!(matches!(
            game.discard([c("D7"), c("CA")]),
            Err(GameError::InvalidDiscard { .. })
        ));
        game.discard([c("D7"), c("DQ")]).expect("discard");
        assert_eq!(game.phase(), Phase::Playing);
        assert_eq!(game.skat(), [c("D7"), c("DQ")]);
        assert_eq!(game.hand(Seat::DECLARER).len(), 10);
        assert!(game.hand(Seat::DECLARER).contains(&c("D9")));
        assert_eq!(
            game.view(Seat::DECLARER).and_then(|v| v.skat),
            Some([c("D7"), c("DQ")])
        );
        assert_eq!(game.view(Seat(1)).and_then(|v| v.skat), None);
    }

    #[test]
    fn declare_hand_skips_discard_and_keeps_skat_for_declarer() {
        let mut game = fixed_game();
        let decl = Declaration::suit(Suit::Hearts, RiskLevel::Hand);
        let events = game.declare(decl).expect("declare");
        assert_eq!(events.len(), 1);
        assert_eq!(game.phase(), Phase::Playing);
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
    fn deal_from_leaves_extra_cards_in_the_talon() {
        let mut cards = standard_deck();
        let extra = cards.pop().expect("card");
        let mut rng = Rng::from_seed(4);
        assert_eq!(
            Game::deal_from(&mut rng, cards.clone(), Seat::DECLARER),
            Err(GameError::InvalidDeal)
        );
        // 32 distinct cards deal 10/10/10/2 with an empty talon.
        cards.push(extra);
        let game = Game::deal_from(&mut rng, cards, Seat::DECLARER).expect("deal");
        assert!(game.talon().is_empty());
        assert_eq!(game.cards_in_play().len(), 32);
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

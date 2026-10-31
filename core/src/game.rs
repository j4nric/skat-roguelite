//! State machine for a single Skat game: deal → declare → (discard) → ten
//! tricks → scored.
//!
//! Every state change is recorded as an [`Event`]; frontends animate them,
//! the simulator ignores them. Illegal actions return a [`GameError`] and
//! leave the state untouched. Nothing in here panics on bad input.

use std::fmt;

use crate::cards::{Card, standard_deck};
use crate::rng::Rng;
use crate::rules::{self, Declaration, GameValue, Matadors, Outcome};

/// Cards each seat receives when dealing.
pub const HAND_SIZE: usize = 10;
/// Tricks in a game.
pub const TRICKS_PER_GAME: usize = 10;

/// One of the three seats at the table. Seat 0 is always the declarer
/// (the human player, see ADR 0002), seats 1 and 2 are the defenders.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Seat(u8);

impl Seat {
    /// The declarer's seat.
    pub const DECLARER: Seat = Seat(0);
    /// All seats in table order.
    pub const ALL: [Seat; 3] = [Seat(0), Seat(1), Seat(2)];

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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    /// The declarer looks at ten cards and chooses the declaration.
    Declaring,
    /// The declarer holds twelve cards and must discard two.
    Discarding,
    /// Tricks are being played.
    Playing,
    /// All tricks are played and the game is scored.
    Finished(GameValue),
}

/// A trick that has been played out.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CompletedTrick {
    /// Seat that led.
    pub lead: Seat,
    /// Cards in play order, `cards[0]` by `lead`.
    pub cards: [Card; 3],
    /// Seat that won the trick.
    pub winner: Seat,
}

impl CompletedTrick {
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
}

/// Seat that plays the card at `offset` in a trick led by `lead`.
#[must_use]
pub fn seat_at(lead: Seat, offset: usize) -> Seat {
    // offset % 3 is always 0..3, so the lookup never fails.
    Seat::from_index((lead.index() + offset) % 3).unwrap_or(lead)
}

/// Something that happened in the game, in order of occurrence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Event {
    /// Cards were dealt.
    Dealt {
        /// Seat that will lead the first trick.
        forehand: Seat,
        /// Hands per seat.
        hands: [Vec<Card>; 3],
        /// The two cards in the skat.
        skat: [Card; 2],
    },
    /// The declarer announced the game.
    Declared {
        /// The declaration.
        declaration: Declaration,
    },
    /// The declarer picked up the skat (not in hand games).
    SkatPickedUp {
        /// The two cards that joined the declarer's hand.
        skat: [Card; 2],
    },
    /// The declarer discarded two cards.
    Discarded {
        /// The discarded cards (they count for the declarer).
        cards: [Card; 2],
    },
    /// A card was played.
    CardPlayed {
        /// Who played.
        seat: Seat,
        /// What was played.
        card: Card,
    },
    /// A trick was completed.
    TrickWon {
        /// The completed trick.
        trick: CompletedTrick,
        /// Augen in the trick.
        card_points: u32,
    },
    /// The last trick was played and the game is scored.
    GameFinished {
        /// Trick outcome used for scoring.
        outcome: Outcome,
        /// Matadors over the declarer's twelve cards.
        matadors: Matadors,
        /// The resulting game value.
        value: GameValue,
    },
}

/// Why an action was rejected.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GameError {
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
        card: Card,
    },
    /// The card does not follow the lead although the seat could.
    MustFollowSuit {
        /// The rejected card.
        card: Card,
        /// The lead card of the trick.
        lead: Card,
    },
    /// The two discards are not two distinct cards from the hand.
    InvalidDiscard {
        /// The rejected discards.
        cards: [Card; 2],
    },
    /// The supplied cards do not form a legal deal (32 distinct cards,
    /// ten per hand).
    InvalidDeal,
}

impl fmt::Display for GameError {
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
        }
    }
}

impl std::error::Error for GameError {}

/// What a seat is allowed to know when choosing a card: its own hand, the
/// declaration, everything played so far and, for the declarer after
/// discarding, the skat.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SeatView<'a> {
    /// The seat looking at the table.
    pub seat: Seat,
    /// The declaration being played.
    pub declaration: Declaration,
    /// Seat that led the first trick.
    pub forehand: Seat,
    /// Own hand.
    pub hand: &'a [Card],
    /// Seat that led the current trick.
    pub trick_lead: Seat,
    /// Cards of the current trick in play order (0 to 2 cards).
    pub trick: &'a [Card],
    /// Completed tricks in order.
    pub tricks: &'a [CompletedTrick],
    /// The skat, if this seat knows it (declarer after picking it up).
    pub skat: Option<[Card; 2]>,
    /// Cards this seat may play now. Empty if it is not its turn.
    pub legal_moves: Vec<Card>,
}

impl SeatView<'_> {
    /// Every card that is no longer unknown to this seat: own hand, all
    /// played cards and the skat if known.
    #[must_use]
    pub fn seen_cards(&self) -> Vec<Card> {
        let mut seen: Vec<Card> = self.hand.to_vec();
        seen.extend(self.tricks.iter().flat_map(|t| t.cards.iter().copied()));
        seen.extend(self.trick.iter().copied());
        if let Some(skat) = self.skat {
            seen.extend(skat);
        }
        seen
    }

    /// Cards whose location this seat does not know.
    #[must_use]
    pub fn unseen_cards(&self) -> Vec<Card> {
        let seen = self.seen_cards();
        standard_deck()
            .into_iter()
            .filter(|c| !seen.contains(c))
            .collect()
    }
}

/// A single Skat game.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Game {
    forehand: Seat,
    hands: [Vec<Card>; 3],
    skat: [Card; 2],
    /// Declarer's original hand plus skat: the twelve cards for matadors.
    declarer_cards: Vec<Card>,
    declaration: Option<Declaration>,
    phase: Phase,
    trick_lead: Seat,
    trick: Vec<Card>,
    tricks: Vec<CompletedTrick>,
    card_points: [u32; 3],
    log: Vec<Event>,
}

impl Game {
    /// Shuffle the standard deck with `rng` and deal 10/10/10/2. The game
    /// starts in [`Phase::Declaring`]; the `Dealt` event is in [`Game::log`].
    #[must_use]
    pub fn deal(rng: &mut Rng, forehand: Seat) -> Game {
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

    /// Start a game from explicit cards (tests, replays, frontends with a
    /// modified deck). Hands are sorted so that equal deals compare equal.
    pub fn with_cards(
        hands: [Vec<Card>; 3],
        skat: [Card; 2],
        forehand: Seat,
    ) -> Result<Game, GameError> {
        if hands.iter().any(|h| h.len() != HAND_SIZE) {
            return Err(GameError::InvalidDeal);
        }
        let mut all: Vec<Card> = hands.iter().flatten().copied().chain(skat).collect();
        all.sort();
        all.dedup();
        if all.len() != 3 * HAND_SIZE + 2 {
            return Err(GameError::InvalidDeal);
        }
        Ok(Game::build(hands, skat, forehand))
    }

    /// Build the initial state from cards that are known to be valid.
    fn build(mut hands: [Vec<Card>; 3], skat: [Card; 2], forehand: Seat) -> Game {
        for hand in &mut hands {
            hand.sort();
        }
        let mut declarer_cards = hands[0].clone();
        declarer_cards.extend(skat);
        let log = vec![Event::Dealt {
            forehand,
            hands: hands.clone(),
            skat,
        }];
        Game {
            forehand,
            hands,
            skat,
            declarer_cards,
            declaration: None,
            phase: Phase::Declaring,
            trick_lead: forehand,
            trick: Vec::with_capacity(3),
            tricks: Vec::with_capacity(TRICKS_PER_GAME),
            card_points: [0; 3],
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
    pub fn hand(&self, seat: Seat) -> &[Card] {
        &self.hands[seat.index()]
    }

    /// The skat: untouched in hand games, the discards otherwise.
    #[must_use]
    pub fn skat(&self) -> [Card; 2] {
        self.skat
    }

    /// The declarer's original ten cards plus the skat.
    #[must_use]
    pub fn declarer_cards(&self) -> &[Card] {
        &self.declarer_cards
    }

    /// The declaration, once made.
    #[must_use]
    pub fn declaration(&self) -> Option<Declaration> {
        self.declaration
    }

    /// Completed tricks in order.
    #[must_use]
    pub fn tricks(&self) -> &[CompletedTrick] {
        &self.tricks
    }

    /// Cards of the current, incomplete trick in play order.
    #[must_use]
    pub fn current_trick(&self) -> &[Card] {
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

    /// All events so far, oldest first.
    #[must_use]
    pub fn log(&self) -> &[Event] {
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

    /// Lead card of the current trick, if any.
    fn lead_card(&self) -> Option<Card> {
        self.trick.first().copied()
    }

    /// Cards `seat` may play right now. Empty unless it is that seat's turn.
    #[must_use]
    pub fn legal_moves(&self, seat: Seat) -> Vec<Card> {
        match (self.current_seat(), self.declaration) {
            (Some(current), Some(declaration)) if current == seat => {
                rules::legal_moves(declaration.game, self.hand(seat), self.lead_card())
            }
            _ => Vec::new(),
        }
    }

    /// What `seat` knows during play. `None` outside [`Phase::Playing`].
    #[must_use]
    pub fn view(&self, seat: Seat) -> Option<SeatView<'_>> {
        if self.phase != Phase::Playing {
            return None;
        }
        let declaration = self.declaration?;
        let skat = if seat.is_declarer() && !declaration.hand() {
            Some(self.skat)
        } else {
            None
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
            legal_moves: self.legal_moves(seat),
        })
    }

    /// Announce the game. Hand games go straight to playing; otherwise the
    /// skat is picked up and the game waits for the discard.
    pub fn declare(&mut self, declaration: Declaration) -> Result<Vec<Event>, GameError> {
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
    pub fn discard(&mut self, cards: [Card; 2]) -> Result<Vec<Event>, GameError> {
        if self.phase != Phase::Discarding {
            return Err(GameError::WrongPhase { phase: self.phase });
        }
        let hand = &self.hands[Seat::DECLARER.index()];
        if cards[0] == cards[1] || !hand.contains(&cards[0]) || !hand.contains(&cards[1]) {
            return Err(GameError::InvalidDiscard { cards });
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
    pub fn play(&mut self, seat: Seat, card: Card) -> Result<Vec<Event>, GameError> {
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
            if !rules::legal_moves(declaration.game, self.hand(seat), Some(lead)).contains(&card) {
                return Err(GameError::MustFollowSuit { card, lead });
            }
        }

        self.hands[seat.index()].retain(|&c| c != card);
        self.trick.push(card);
        let mut events = vec![Event::CardPlayed { seat, card }];

        if self.trick.len() == 3 {
            let cards = [self.trick[0], self.trick[1], self.trick[2]];
            let winner_index =
                rules::trick_winner(declaration.game, self.trick_lead.index(), cards);
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

            if self.tricks.len() == TRICKS_PER_GAME {
                let outcome = self.outcome();
                let matadors = rules::matadors(declaration.game, &self.declarer_cards);
                let value = rules::game_value(declaration, matadors, &outcome);
                self.phase = Phase::Finished(value);
                events.push(Event::GameFinished {
                    outcome,
                    matadors,
                    value,
                });
            }
        }
        self.log.extend(events.iter().cloned());
        Ok(events)
    }

    /// Trick outcome so far. The skat always counts for the declarer.
    #[must_use]
    pub fn outcome(&self) -> Outcome {
        let skat_points: u32 = self.skat.iter().map(|c| c.card_points()).sum();
        let declarer_tricks = self
            .tricks
            .iter()
            .filter(|t| t.winner.is_declarer())
            .count();
        Outcome {
            declarer_card_points: self.card_points[0] + skat_points,
            defender_card_points: self.card_points[1] + self.card_points[2],
            // At most ten tricks exist.
            declarer_tricks: u32::try_from(declarer_tricks).unwrap_or(u32::MAX),
            defender_tricks: u32::try_from(self.tricks.len() - declarer_tricks).unwrap_or(u32::MAX),
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
    use crate::rules::{GameType, RiskLevel};

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
        let decl = Declaration::new(GameType::Suit(Suit::Hearts), RiskLevel::Normal);
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
        let decl = Declaration::new(GameType::Suit(Suit::Hearts), RiskLevel::Hand);
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
        game.declare(Declaration::new(
            GameType::Suit(Suit::Hearts),
            RiskLevel::Hand,
        ))
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
}

//! The game at the table: Einmischen (redeal), picking up the skat,
//! declaring, discarding, Kontra, playing cards (with "Spiegel" and
//! "Untot") and ending a decided game ("Rest abkürzen").

use crate::ai::DefenderHeuristic;
use crate::cards::PlayingCard;
use crate::content::{
    CardFilter, Change, Content, Effect, Lever, NextGameEffect, ScoreLine, Source, Stage,
};
use crate::deck::{DealtCard, Owner};
use crate::game::{Event, Phase, Seat};
use crate::rng::Rng;
use crate::rules::{Declaration, GameType, suit_strength, trump_strength};

use super::{ActiveGame, Kontra, Run, RunError, RunEvent, opponents};

/// Mixed into the game's seed for the card "Drei drücken" draws, so that
/// the draw is independent of the PIMC samples of the same game.
const DRAW_SEED: u64 = 0xD8E1_D8E1_0000_0003;

/// "Untot" (DESIGN 7.2, 03.11.2026): the player picks this many cards of
/// his hand, one of them is drawn to take the rising card's place in the
/// trick ([`RunAction::ChooseRiseCards`](super::RunAction::ChooseRiseCards)).
pub const UNDEAD_PICK: usize = 3;

impl Run {
    /// Einmischen ([`RunAction::Redeal`](super::RunAction::Redeal)): one of the game's own first,
    /// then one of the series' extras ("Ein Bier").
    pub(super) fn redeal_action(
        &mut self,
        content: &Content,
        events: &mut Vec<RunEvent>,
    ) -> Result<(), RunError> {
        self.declaring()?;
        let own_left = self
            .redeal_allowance(content)
            .saturating_sub(self.series.redeals_in_game);
        if own_left > 0 {
            self.series.redeals_in_game += 1;
        } else if self.series.extra_redeals > 0 && !self.malus(content).is_some_and(|m| m.no_redeal)
        {
            self.series.extra_redeals -= 1;
        } else {
            return Err(RunError::NoRedealLeft);
        }
        self.redeal(content, events)
    }

    /// Throw the dealt game in and deal it again before the declaration,
    /// without using up a game (Einmischen, "Neues Blatt"). The forehand
    /// stays. The deal thrown in is kept for "Reue"; every redeal counts
    /// for the series (rule rotation of "Der Alte").
    pub(super) fn redeal(
        &mut self,
        content: &Content,
        events: &mut Vec<RunEvent>,
    ) -> Result<(), RunError> {
        self.declaring()?;
        let keep = self.kept_at_redeal(content);
        self.series.redeals_this_game += 1;
        let previous = self.game.take().map(|mut active| {
            active.previous = None;
            Box::new(active)
        });
        self.series.redeals_used += 1;
        self.stats.redeals += 1;
        events.push(RunEvent::Redealt);
        self.deal_keeping(content, false, &keep, events)?;
        if let Some(active) = self.game.as_mut() {
            active.previous = previous;
        }
        Ok(())
    }

    /// The cards of the player's hand an Einmischen leaves him
    /// ([`Effect::RedealKeeps`], "Unterschlupf"): his own cards that match a
    /// filter as he holds them; a Unter his edition demotes is no Unter.
    /// A filter with `per_game` counts only at the first that many
    /// Einmischen of the game ([`Series::redeals_this_game`](super::Series::redeals_this_game)).
    fn kept_at_redeal(&self, content: &Content) -> Vec<u32> {
        let Some(active) = self.game.as_ref() else {
            return Vec::new();
        };
        let done = self.series.redeals_this_game;
        let filters: Vec<CardFilter> = self
            .sources(content, false)
            .iter()
            .flat_map(|s| s.passive.iter())
            .filter_map(|e| match e {
                Effect::RedealKeeps { filter, per_game } => {
                    per_game.is_none_or(|n| done < n).then_some(*filter)
                }
                _ => None,
            })
            .collect();
        if filters.is_empty() {
            return Vec::new();
        }
        active
            .game
            .hand(Seat::DECLARER)
            .iter()
            .filter(|c| c.owner == Owner::Player && !crate::rules::is_demoted_jack(**c))
            .filter(|c| {
                c.face()
                    .is_some_and(|f| filters.iter().any(|filter| filter.matches(f)))
            })
            .map(|c| c.id)
            .collect()
    }

    /// "Reue": the deal before the last Einmischen comes back as it was
    /// (hands, skat, what the player was shown). The Einmischen stays used.
    pub(super) fn undo_redeal(&mut self, events: &mut Vec<RunEvent>) -> Result<(), RunError> {
        let active = self.declaring()?;
        let previous = active.previous.take().ok_or(RunError::NoRedealToUndo)?;
        self.game = Some(*previous);
        events.push(RunEvent::RedealUndone);
        Ok(())
    }

    /// Pick up the skat (ADR 0011). Not if the boss demands a hand game.
    /// "Der Angeber" allows it: Null Ouvert can be played open after the
    /// skat (ADR 0012, every character may declare Null).
    ///
    /// "Drei drücken": at the first pickup of the series the third card is
    /// a random card of the left defender, drawn with a generator of its
    /// own from the game's seed, so the deal and every other draw stay the
    /// same.
    pub(super) fn pick_up_skat(&mut self, events: &mut Vec<RunEvent>) -> Result<(), RunError> {
        let mut draw_rng = Rng::from_seed(opponents::game_seed(self) ^ DRAW_SEED);
        let draw = self.discard_three_ready();
        let used_up = self.skat_used_up();
        let active = self.declaring()?;
        if active.rules.require_hand {
            return Err(RunError::HandRequired);
        }
        if used_up {
            return Err(RunError::SkatUsedUp);
        }
        let game_events = if draw {
            let from = Seat::DEFENDERS[0];
            let hand = active.game.hand(from);
            let pick = draw_rng.gen_range(0..u64::try_from(hand.len()).unwrap_or(1));
            let card = hand
                .get(usize::try_from(pick).unwrap_or(0))
                .copied()
                .ok_or(RunError::WrongPhase)?;
            let game_events = active.game.pick_up_skat_drawing(from, card)?;
            self.series.discard_three_used = true;
            game_events
        } else {
            active.game.pick_up_skat()?
        };
        self.series.skat_pickups = self.series.skat_pickups.saturating_add(1);
        events.extend(game_events.into_iter().map(RunEvent::Game));
        Ok(())
    }

    /// Whether the skat may not be picked up again in this pub ("Der
    /// Zocker", [`crate::content::Effect::SkatPickupsPerPub`]): every game
    /// left in it is a hand game.
    #[must_use]
    pub fn skat_used_up(&self) -> bool {
        self.skat_pickups_left() == Some(0)
    }

    /// How often the skat may still be picked up in this pub, `None`
    /// without a limit (or without a game).
    #[must_use]
    pub fn skat_pickups_left(&self) -> Option<u32> {
        let limit = self.game.as_ref()?.rules.skat_pickups?;
        Some(limit.saturating_sub(self.series.skat_pickups))
    }

    /// Whether the player may declare a Ramsch instead of a game in this
    /// game ("Die Ramschtante", [`crate::content::Effect::OfferRamsch`]):
    /// before the skat, never in a game that is a Ramsch anyway.
    #[must_use]
    pub fn ramsch_offered(&self) -> bool {
        self.game.as_ref().is_some_and(|a| {
            a.rules.ramsch_offered && !a.rules.ramsch && a.game.phase() == Phase::Declaring
        })
    }

    /// Whether "Drei drücken" draws a card at the next pickup of the
    /// skat: granted in this game, the first game of the pub (since
    /// 02.11.2026), and not yet used in it.
    #[must_use]
    pub fn discard_three_ready(&self) -> bool {
        !self.series.discard_three_used
            && self.series.games_played == 0
            && self
                .game
                .as_ref()
                .is_some_and(|a| a.rules.discard_three.is_some())
    }

    /// What [`Run::declare`] checks before it changes anything.
    pub(super) fn check_declare(&self, declaration: Declaration) -> Result<&ActiveGame, RunError> {
        let active = self.active_ref()?;
        if active.rules.ramsch {
            return Err(RunError::RamschRequired);
        }
        if !matches!(
            active.game.phase(),
            Phase::Declaring | Phase::DeclaringAfterSkat
        ) {
            return Err(RunError::WrongPhase);
        }
        // Every character may declare every game (ADR 0012); the Ramsch
        // comes from "Der Ramscher", or before the skat from a rule that
        // offers it ("Die Ramschtante").
        if declaration == Declaration::Ramsch {
            let offered = active.rules.ramsch_offered && active.game.phase() == Phase::Declaring;
            if !offered {
                return Err(RunError::GameNotAvailable);
            }
            if active.rules.require_ouvert {
                return Err(RunError::OuvertRequired);
            }
            return Ok(active);
        }
        if active.rules.require_hand && !declaration.hand() {
            return Err(RunError::HandRequired);
        }
        if active.rules.require_ouvert && !declaration.ouvert() {
            return Err(RunError::OuvertRequired);
        }
        Ok(active)
    }

    pub(super) fn declare(
        &mut self,
        content: &Content,
        declaration: Declaration,
        events: &mut Vec<RunEvent>,
    ) -> Result<(), RunError> {
        let active = self.check_declare(declaration)?;
        // Before the skat a declaration is always a hand game: it redeems a
        // voucher of the hand (DESIGN 7.2), before the game starts. A
        // Ramsch is nobody's game: no voucher, no Kontra.
        let ramsch = declaration.is_ramsch();
        if active.game.phase() == Phase::Declaring && !ramsch {
            self.redeem_hand_voucher(content, events);
        }
        let active = self.active_mut()?;
        let game_events = active.game.declare(declaration)?;
        active.note_settled(&game_events);
        events.extend(game_events.into_iter().map(RunEvent::Game));
        self.after_declaration(content, events);
        if ramsch {
            self.series.ramsch_played = true;
        }
        if let Some(active) = self.game.as_mut().filter(|_| !ramsch) {
            if active.rules.kontra_source.is_some() {
                let kontra_seat = Seat::DEFENDERS.into_iter().find(|&seat| {
                    DefenderHeuristic.says_kontra(active.game.hand(seat), declaration.game())
                });
                if let Some(seat) = kontra_seat {
                    active.kontra = Kontra::Kontra(seat);
                    active.awaiting_re = true;
                    events.push(RunEvent::Kontra { seat });
                }
            }
        }
        self.settle(content, events)
    }

    /// After any declaration (also the Ramsch): fire `OnDeclare` and use
    /// up the factor of "Letzte Runde" and the consumables waiting for the
    /// next game ("Doppelkorn", "Magenbitter").
    pub(super) fn after_declaration(&mut self, content: &Content, events: &mut Vec<RunEvent>) {
        self.fire(content, Stage::OnDeclare, None, None, events);
        let pending = std::mem::take(&mut self.pending_next_game);
        let Some(active) = self.game.as_mut() else {
            return;
        };
        self.peek_next_skat = None;
        if let Some((encounter, factor)) = self.pending_factor.take() {
            active.lines.push(ScoreLine {
                source: Source::Encounter(encounter),
                lever: Lever::GameValue,
                change: Change::Multiply(factor),
            });
        }
        for (id, effect) in pending {
            let source = Source::Consumable(id);
            match effect {
                NextGameEffect::Multiply(factor) => active.lines.push(ScoreLine {
                    source,
                    lever: Lever::GameValue,
                    change: Change::Multiply(factor),
                }),
                NextGameEffect::SingleLoss => {
                    active.single_loss.get_or_insert(source);
                }
                NextGameEffect::MultiplyWin(factor) => active.win_factors.push((source, factor)),
            }
        }
    }

    /// Skat swaps ("Die Oma", "Der Opa") still open in this series: the
    /// granted filters minus those used, in the order they were granted.
    pub(super) fn open_skat_swaps(&self) -> Vec<CardFilter> {
        let Some(active) = &self.game else {
            return Vec::new();
        };
        let mut open = active.rules.skat_swaps.clone();
        let taken = &self.series.skat_swaps_taken;
        for filter in taken {
            if let Some(i) = open.iter().position(|f| f == filter) {
                open.remove(i);
            }
        }
        // Saves from before the list: the first swaps count as used.
        let untracked = usize::try_from(self.series.skat_swaps_used)
            .unwrap_or(usize::MAX)
            .saturating_sub(taken.len());
        open.drain(..untracked.min(open.len()));
        open
    }

    /// Skat cards an open skat swap may take now, the preferred one first
    /// (the strongest: jacks by the jack order, then by rank). Empty
    /// outside a hand game before the player's first card.
    #[must_use]
    pub fn skat_swap_cards(&self) -> Vec<DealtCard> {
        let open = self.open_skat_swaps();
        let Some(active) = &self.game else {
            return Vec::new();
        };
        let game = &active.game;
        let ready = game.phase() == Phase::Playing
            && game
                .declaration()
                .is_some_and(|d| d.hand() && !d.is_ramsch())
            && game.hand(Seat::DECLARER).len() == crate::game::HAND_SIZE
            && !active.awaiting_re;
        if open.is_empty() || !ready {
            return Vec::new();
        }
        let mut cards: Vec<DealtCard> = game
            .skat()
            .into_iter()
            .filter(|c| c.face().is_some_and(|f| open.iter().any(|o| o.matches(f))))
            .collect();
        cards.sort_by_key(|c| {
            std::cmp::Reverse((
                trump_strength(GameType::Grand, *c),
                suit_strength(GameType::Grand, *c),
                c.id,
            ))
        });
        cards
    }

    /// Skat swap ("Die Oma", "Der Opa"): take a matching card from the
    /// untouched skat of a hand game (`take`, or the preferred one) and put
    /// `give` in its place. Each granted filter works once per series.
    pub(super) fn take_from_skat(
        &mut self,
        give: u32,
        take: Option<u32>,
        events: &mut Vec<RunEvent>,
    ) -> Result<(), RunError> {
        let open = self.open_skat_swaps();
        let candidates = self.skat_swap_cards();
        let active = self.active_mut()?;
        if active.awaiting_re {
            return Err(RunError::AwaitingKontraAnswer);
        }
        if open.is_empty() {
            return Err(RunError::NoSkatSwapLeft);
        }
        let gave = Run::declarer_card(active, give)?;
        let took = match take {
            Some(id) => candidates.iter().copied().find(|c| c.id == id),
            None => candidates.first().copied(),
        }
        .ok_or(RunError::NothingToTake)?;
        let filter = open
            .iter()
            .copied()
            .find(|f| took.face().is_some_and(|face| f.matches(face)))
            .ok_or(RunError::NothingToTake)?;
        let game_events = active.game.swap_with_skat(took, gave)?;
        events.extend(game_events.into_iter().map(RunEvent::Game));
        self.series.skat_swaps_used += 1;
        self.series.skat_swaps_taken.push(filter);
        Ok(())
    }

    pub(super) fn answer_kontra(
        &mut self,
        content: &Content,
        re: bool,
        events: &mut Vec<RunEvent>,
    ) -> Result<(), RunError> {
        let active = self.active_mut()?;
        if !active.awaiting_re {
            return Err(RunError::NoKontra);
        }
        active.awaiting_re = false;
        if re {
            if let Kontra::Kontra(seat) = active.kontra {
                active.kontra = Kontra::Re(seat);
                events.push(RunEvent::Re);
            }
        }
        self.settle(content, events)
    }

    /// Find a card of the declarer's hand by instance id.
    pub(super) fn declarer_card(active: &ActiveGame, id: u32) -> Result<DealtCard, RunError> {
        active
            .game
            .hand(Seat::DECLARER)
            .iter()
            .copied()
            .find(|c| c.id == id)
            .ok_or(RunError::UnknownCard(id))
    }

    pub(super) fn discard(
        &mut self,
        content: &Content,
        ids: [u32; 2],
        events: &mut Vec<RunEvent>,
    ) -> Result<(), RunError> {
        let active = self.active_mut()?;
        if active.awaiting_re {
            return Err(RunError::AwaitingKontraAnswer);
        }
        let cards = [
            Run::declarer_card(active, ids[0])?,
            Run::declarer_card(active, ids[1])?,
        ];
        let game_events = active.game.discard(cards)?;
        active.note_settled(&game_events);
        events.extend(game_events.into_iter().map(RunEvent::Game));
        self.redeem_vouchers(content, events);
        self.settle(content, events)
    }

    /// "Drei drücken": discard three of the thirteen cards.
    pub(super) fn discard_three(
        &mut self,
        content: &Content,
        ids: [u32; 3],
        events: &mut Vec<RunEvent>,
    ) -> Result<(), RunError> {
        let active = self.active_mut()?;
        if active.awaiting_re {
            return Err(RunError::AwaitingKontraAnswer);
        }
        let cards = [
            Run::declarer_card(active, ids[0])?,
            Run::declarer_card(active, ids[1])?,
            Run::declarer_card(active, ids[2])?,
        ];
        let game_events = active.game.discard_three(cards)?;
        active.note_settled(&game_events);
        events.extend(game_events.into_iter().map(RunEvent::Game));
        self.redeem_vouchers(content, events);
        self.settle(content, events)
    }

    pub(super) fn play(
        &mut self,
        content: &Content,
        id: u32,
        events: &mut Vec<RunEvent>,
    ) -> Result<(), RunError> {
        let active = self.active_mut()?;
        if active.awaiting_re {
            return Err(RunError::AwaitingKontraAnswer);
        }
        let card = Run::declarer_card(active, id)?;
        if self.mirrors_now(content, card) {
            let active = self.active_mut()?;
            let game_events = active.game.play_mirrored(Seat::DECLARER, card)?;
            let copy = game_events
                .iter()
                .find_map(|e| match e {
                    Event::DoppelgangerMirrored { copy, .. } => Some(*copy),
                    _ => None,
                })
                .unwrap_or(card);
            let trick_done = game_events
                .iter()
                .any(|e| matches!(e, Event::TrickWon { .. }));
            events.extend(game_events.into_iter().map(RunEvent::Game));
            self.after_card(content, Seat::DECLARER, copy, trick_done, events);
            return self.settle(content, events);
        }
        self.play_card(content, Seat::DECLARER, card, events)?;
        self.settle(content, events)
    }

    /// Whether the player's `card` is a "Doppelgänger" whose mirror is
    /// still to come in this game ("Spiegelbild", DESIGN 7.2, since
    /// 07.11.2026): it is in his hand, the game is a suit game or Grand
    /// (not Null, not a Ramsch), and no Doppelgänger mirrored in this game
    /// yet (once per game, whatever the number of Doppelgänger). Played as
    /// the last card of a trick it would mirror ([`Run::mirrors_now`]).
    /// `false` outside a game.
    #[must_use]
    pub fn can_mirror(&self, content: &Content, card: DealtCard) -> bool {
        let Some(active) = self.game.as_ref() else {
            return false;
        };
        let game = &active.game;
        let trump_game = game
            .declaration()
            .is_some_and(|d| !d.is_null() && !d.is_ramsch());
        // The scan of the log last: most cards are no Doppelgänger.
        trump_game
            && !card.is_blank()
            && game.hand(Seat::DECLARER).contains(&card)
            && self.has_card_effect(content, card, &Effect::Doppelganger)
            && !game
                .log()
                .iter()
                .any(|e| matches!(e, Event::DoppelgangerMirrored { .. }))
    }

    /// Whether the player's `card` mirrors if he plays it now: it may
    /// mirror ([`Run::can_mirror`]), it is his turn as the last card of
    /// the trick and it is a legal play as printed. Otherwise a
    /// Doppelgänger plays as its printed card.
    #[must_use]
    pub fn mirrors_now(&self, content: &Content, card: DealtCard) -> bool {
        let Some(active) = self.game.as_ref() else {
            return false;
        };
        let game = &active.game;
        game.current_trick().len() == 2
            && game.current_seat() == Some(Seat::DECLARER)
            && game.legal_moves(Seat::DECLARER).contains(&card)
            && self.can_mirror(content, card)
    }

    /// Play one card and fire the in-play hooks.
    pub(super) fn play_card(
        &mut self,
        content: &Content,
        seat: Seat,
        card: DealtCard,
        events: &mut Vec<RunEvent>,
    ) -> Result<(), RunError> {
        let active = self.active_mut()?;
        let game_events = active.game.play(seat, card)?;
        let trick_done = game_events
            .iter()
            .any(|e| matches!(e, Event::TrickWon { .. }));
        events.extend(game_events.into_iter().map(RunEvent::Game));
        self.after_card(content, seat, card, trick_done, events);
        Ok(())
    }

    /// Fire the hooks after a card: `OnCardPlayed`, and `OnTrickEnd` if it
    /// completed a trick.
    fn after_card(
        &mut self,
        content: &Content,
        seat: Seat,
        card: DealtCard,
        trick_done: bool,
        events: &mut Vec<RunEvent>,
    ) {
        self.fire(
            content,
            Stage::OnCardPlayed,
            Some((seat, card)),
            None,
            events,
        );
        if trick_done {
            self.fire(content, Stage::OnTrickEnd, None, None, events);
            self.raise_undead(content, events);
        }
    }

    /// Whether the player's `card` would rise if he played it now and the
    /// trick were complete ("Untot", [`Effect::Undead`]): it carries the
    /// modification, did not rise in this game yet, the game is a suit game
    /// or Grand, and it is not his last card. `false` outside a game.
    #[must_use]
    pub fn can_rise(&self, content: &Content, card: DealtCard) -> bool {
        let Some(active) = self.game.as_ref() else {
            return false;
        };
        let game = &active.game;
        let trump_game = game
            .declaration()
            .is_some_and(|d| !d.is_null() && !d.is_ramsch());
        let hand = game.hand(Seat::DECLARER);
        trump_game
            && !card.is_blank()
            && hand.len() > 1
            && hand.contains(&card)
            && !active.risen.contains(&card.id)
            && self.has_card_effect(content, card, &Effect::Undead)
    }

    /// "Untot" (DESIGN 7.2, reworked 03.11.2026): after the `OnTrickEnd`
    /// hooks of a completed trick, the player's card in it rises back into
    /// his hand if it may ([`Run::can_rise`], judged as if it were still in
    /// the hand), as a copy of the card that won the trick
    /// ([`Game::raise`](crate::game::Game::raise)). One of [`UNDEAD_PICK`] cards of his hand, drawn
    /// with the game's generator, takes its place in the trick: with more
    /// cards in his hand he picks them first, and the run waits
    /// ([`Run::rise_pending`], [`RunEvent::RiseChoice`]); with as many or
    /// fewer, all of them are the candidates and it rises at once.
    fn raise_undead(&mut self, content: &Content, events: &mut Vec<RunEvent>) {
        let Some(active) = self.game.as_ref() else {
            return;
        };
        let game = &active.game;
        if game.phase() != Phase::Playing || !game.current_trick().is_empty() {
            return;
        }
        let Some(trick) = game.tricks().last() else {
            return;
        };
        let Some(card) = (0..3)
            .find(|&i| trick.seat_of(i).is_declarer())
            .map(|i| trick.cards[i])
        else {
            return;
        };
        let trump_game = game
            .declaration()
            .is_some_and(|d| !d.is_null() && !d.is_ramsch());
        let hand = game.hand(Seat::DECLARER).to_vec();
        if !trump_game
            || card.is_blank()
            || hand.is_empty()
            || active.risen.contains(&card.id)
            || !self.has_card_effect(content, card, &Effect::Undead)
        {
            return;
        }
        if hand.len() > UNDEAD_PICK {
            let winning = trick.winning_card();
            let becomes = if winning == card {
                card
            } else {
                card.risen_as(winning)
            };
            if let Some(active) = self.game.as_mut() {
                active.pending_rise = Some(card.id);
            }
            events.push(RunEvent::RiseChoice {
                card,
                becomes,
                pick: UNDEAD_PICK,
            });
            return;
        }
        self.rise_with(card, &hand, events);
    }

    /// Let the "Untot" `card` of the last trick rise, one of `candidates`
    /// (cards of the player's hand) drawn with the game's generator taking
    /// its place.
    fn rise_with(&mut self, card: DealtCard, candidates: &[DealtCard], events: &mut Vec<RunEvent>) {
        let fallback = opponents::game_seed(self) ^ 0x0DEA_D0DE_AD00;
        let Some(active) = self.game.as_mut() else {
            return;
        };
        let rng = active
            .undead_rng
            .get_or_insert_with(|| Rng::from_seed(fallback));
        let roll = rng.gen_range(0..u64::try_from(candidates.len()).unwrap_or(1));
        let Some(&substitute) = usize::try_from(roll).ok().and_then(|i| candidates.get(i)) else {
            return;
        };
        if let Ok(game_events) = active.game.raise(Seat::DECLARER, card, substitute) {
            active.risen.push(card.id);
            active.pending_rise = None;
            events.extend(game_events.into_iter().map(RunEvent::Game));
        }
    }

    /// The "Untot" card waiting for the player's three cards
    /// ([`RunAction::ChooseRiseCards`](super::RunAction::ChooseRiseCards)), as it lies in the last trick.
    #[must_use]
    pub fn rise_pending(&self) -> Option<DealtCard> {
        let active = self.game.as_ref()?;
        let id = active.pending_rise?;
        active
            .game
            .tricks()
            .last()
            .and_then(|t| t.cards.iter().copied().find(|c| c.id == id))
    }

    /// [`RunAction::ChooseRiseCards`](super::RunAction::ChooseRiseCards): three different cards of the hand,
    /// sorted by id so the draw does not depend on their order; one of
    /// them is drawn and the waiting card rises.
    pub(super) fn choose_rise_cards(
        &mut self,
        content: &Content,
        ids: [u32; UNDEAD_PICK],
        events: &mut Vec<RunEvent>,
    ) -> Result<(), RunError> {
        let card = self.rise_pending().ok_or(RunError::InvalidRiseCards)?;
        let active = self.active_mut()?;
        let mut sorted = ids;
        sorted.sort_unstable();
        if sorted.windows(2).any(|w| w[0] == w[1]) {
            return Err(RunError::InvalidRiseCards);
        }
        let hand = active.game.hand(Seat::DECLARER);
        let candidates: Vec<DealtCard> = sorted
            .iter()
            .filter_map(|id| hand.iter().copied().find(|c| c.id == *id))
            .collect();
        if candidates.len() != UNDEAD_PICK {
            return Err(RunError::InvalidRiseCards);
        }
        self.rise_with(card, &candidates, events);
        self.settle(content, events)
    }

    /// Score a finished game (which deals the next one or ends the
    /// series). The defenders do not play here: whether one is to move is
    /// [`Run::ai_to_move`].
    pub(super) fn settle(
        &mut self,
        content: &Content,
        events: &mut Vec<RunEvent>,
    ) -> Result<(), RunError> {
        match self.game.as_ref().map(|a| a.game.phase()) {
            Some(Phase::Finished(_)) => self.finish_game(content, events),
            _ => Ok(()),
        }
    }

    /// Whether the declarer must declare a hand game in the current game.
    #[must_use]
    pub fn requires_hand(&self) -> bool {
        self.game.as_ref().is_some_and(|a| a.rules.require_hand)
    }

    /// Whether Kontra waits for the declarer's answer.
    #[must_use]
    pub fn awaiting_kontra_answer(&self) -> bool {
        self.game.as_ref().is_some_and(|a| a.awaiting_re)
    }

    /// Einmischen each game of the series allows, for the game in
    /// progress: `run.ron` and the character ([`Content::redeals`]) plus
    /// one per "Schieben" in force ([`GameRules::extra_redeals`](super::GameRules::extra_redeals)).
    fn redeal_allowance(&self, content: &Content) -> u32 {
        if self.malus(content).is_some_and(|m| m.no_redeal) || self.in_tutorial_pub() {
            return 0;
        }
        let extra = self.game.as_ref().map_or(0, |a| a.rules.extra_redeals);
        let withheld = if self.series.games_played == 0 {
            self.series.redeals_withheld
        } else {
            0
        };
        content
            .redeals_in(self.character, self.series.section)
            .saturating_add(extra)
            .saturating_sub(withheld)
            .saturating_sub(self.night_fewer_redeals(content))
    }

    /// Einmischen left now: what the game in progress still allows of its
    /// own, plus the extras of the series ("Ein Bier", "Kräuter"). 0
    /// without a game. Whether one may be used right now (only with ten
    /// cards before the skat, never in the Ramsch) is
    /// [`Run::accepts`]`(`[`RunAction::Redeal`](super::RunAction::Redeal)`)`.
    #[must_use]
    pub fn redeals_left(&self, content: &Content) -> u32 {
        if self.game.is_none()
            || self.malus(content).is_some_and(|m| m.no_redeal)
            || self.in_tutorial_pub()
        {
            return 0;
        }
        self.redeal_allowance(content)
            .saturating_sub(self.series.redeals_in_game)
            .saturating_add(self.series.extra_redeals)
    }
}

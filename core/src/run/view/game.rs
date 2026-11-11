//! The view of the game at the table: the hand in display order with
//! legal cards and card backs, the Ansage dialog, the count of Augen and
//! tricks, the "Untot" choice and what the player sees of other hands.

use std::collections::BTreeMap;

use crate::cards::{Card, PlayingCard, Suit};
use crate::content::{
    BossId, CardModId, CardModStatus, Content, HookContext, OPPONENT_BACK, card_mod_status,
};
use crate::deck::{DealtCard, Owner};
use crate::game::{Phase, Seat, seat_at};
use crate::rules::{
    Declaration, GameType, NullRisk, NullScoring, RiskLevel, discardable_of, suit_strength,
    trump_order_key,
};

use super::{
    CardView, CountView, DeclareOptions, GameOption, GameView, NullRiskOption, PlayedCard,
    RevealSource, RevealedCard, RiseChoiceView, RiskOption, Run, RunAction, RunPhase, SettledWild,
    TablePhase, TrickView,
};

/// Signed Augen shown as a count: never below 0.
fn non_negative(points: i64) -> u32 {
    u32::try_from(points.max(0)).unwrap_or(u32::MAX)
}

/// The trump games of the Ansage dialog in dialog order; Null follows.
const GAMES: [GameType; 5] = [
    GameType::Suit(Suit::Clubs),
    GameType::Suit(Suit::Spades),
    GameType::Suit(Suit::Hearts),
    GameType::Suit(Suit::Diamonds),
    GameType::Grand,
];

/// Display order of a card in a hand: trumps by strength, then the suits
/// Kreuz, Pik, Herz, Karo by rank, blanks last. Uses the printed card, so a
/// "Gezinkt" card stays in its place; a "Bube ehrenhalber" sits below the
/// jacks in a suit game.
pub(super) fn display_key(game: GameType, card: &DealtCard) -> (u8, u8, u32) {
    let Some(face) = card.face() else {
        return (9, 0, card.id);
    };
    let printed = DealtCard {
        rank_bonus: 0,
        ..*card
    };
    if let Some((strength, exact)) = trump_order_key(game, printed) {
        return (0, u8::MAX - (2 * strength + exact), card.id);
    }
    let suit_rank = match face.suit {
        Suit::Clubs => 1,
        Suit::Spades => 2,
        Suit::Hearts => 3,
        Suit::Diamonds => 4,
    };
    (
        suit_rank,
        u8::MAX - suit_strength::<Card>(game, face),
        card.id,
    )
}

impl Run {
    /// The card back of a seat (ADR 0015): `None` for the player, the
    /// boss's in his seat, a Stammgast's otherwise.
    fn back_of_seat(&self, content: &Content, seat: Seat) -> Option<String> {
        match self.owner_of_seat(seat) {
            Owner::Player => None,
            Owner::Boss => Some(self.boss_back(content, 0)),
            Owner::BossPartner => Some(self.boss_back(content, 1)),
            // Borrowed: this runs for every card of the view.
            Owner::Guest => Some(
                self.seated_guests(content)
                    .iter()
                    .zip(Seat::DEFENDERS)
                    .find(|(_, s)| *s == seat)
                    .and_then(|(id, _)| content.guest(id))
                    .map_or_else(|| OPPONENT_BACK.to_owned(), |g| g.back.clone()),
            ),
        }
    }

    /// The back of a boss player (figure 0 left, 1 right): the figure's
    /// own, else the boss's (for "Der Alte" always his, not that of the
    /// boss whose rule is on).
    fn boss_back(&self, content: &Content, figure: usize) -> String {
        let back = |id: &BossId| {
            content.boss(id).and_then(|b| {
                b.figures
                    .get(figure)
                    .and_then(|f| f.back.clone())
                    .or_else(|| b.back.clone())
            })
        };
        self.series
            .boss
            .as_ref()
            .and_then(back)
            .or_else(|| self.game.as_ref()?.boss_rule.as_ref().and_then(back))
            .unwrap_or_else(|| OPPONENT_BACK.to_owned())
    }

    /// The card back of a card's owner: `None` for the player's own.
    fn back_of(&self, content: &Content, owner: Owner) -> Option<String> {
        match owner {
            Owner::Player => None,
            Owner::Boss => Some(self.boss_back(content, 0)),
            Owner::BossPartner => Some(self.boss_back(content, 1)),
            // The Stammgast's seat: right of the player (seat 2) holds one
            // in every series.
            Owner::Guest => self.back_of_seat(content, Seat::DEFENDERS[1]),
        }
    }

    /// The "Untot" card waiting for the player's three cards, if any.
    fn rise_choice_view(&self, content: &Content) -> Option<RiseChoiceView> {
        let card = self.rise_pending()?;
        let trick = self.game.as_ref()?.game.tricks().last().copied()?;
        let winning = trick.winning_card();
        let becomes = if winning == card {
            card
        } else {
            card.risen_as(winning)
        };
        Some(RiseChoiceView {
            card: self.card_view(content, card),
            becomes: self.card_view(content, becomes),
            pick: super::super::UNDEAD_PICK,
        })
    }

    fn card_view(&self, content: &Content, card: DealtCard) -> CardView {
        let chalked = self.game.as_ref().and_then(|a| {
            a.chalked
                .iter()
                .find(|(id, _)| *id == card.id)
                .map(|&(_, suit)| suit)
        });
        let (mods, mod_values) = self
            .editions()
            .of(card.owner)
            .and_then(|e| e.get(card.id))
            .map(|d| (d.mods.clone(), d.mod_values.clone()))
            .unwrap_or_default();
        let mod_status = self.mod_status(content, card, &mods);
        let named = self.game.as_ref().and_then(|a| {
            a.named
                .iter()
                .find(|(id, _)| *id == card.id)
                .map(|&(_, suit)| suit)
        });
        CardView {
            chalked,
            card,
            mods,
            back: self.back_of(content, card.owner),
            mod_values,
            legal: false,
            from_skat: false,
            drawn_from: None,
            discardable: false,
            mod_status,
            unsettled: card.wild
                && self
                    .game
                    .as_ref()
                    .is_some_and(|a| a.game.declaration().is_none()),
            named,
            can_rise: false,
            can_mirror: false,
            risen: card.owner == Owner::Player
                && self
                    .game
                    .as_ref()
                    .is_some_and(|a| a.risen.contains(&card.id)),
        }
    }

    /// Whether the conditional modifications on `card` work in the game in
    /// progress, from what the player knows ([`card_mod_status`]).
    fn mod_status(
        &self,
        content: &Content,
        card: DealtCard,
        mods: &[CardModId],
    ) -> BTreeMap<CardModId, CardModStatus> {
        let Some(active) = &self.game else {
            return BTreeMap::new();
        };
        let ctx = HookContext {
            flags: &self.flags,
            counters: &self.counters,
            game: Some(&active.game),
            played: None,
            result: None,
            games_left: self.series.games_left(),
            transport: None,
            marked_cards: None,
            run: Some((self, content)),
        };
        mods.iter()
            .filter_map(|id| {
                let def = content.card_mod(id)?;
                card_mod_status(def, &ctx, card).map(|status| (id.clone(), status))
            })
            .collect()
    }

    /// Whether the skat is picked up and discarded: the Ansage without hand
    /// levels.
    fn after_skat(&self) -> bool {
        self.game
            .as_ref()
            .is_some_and(|a| a.game.phase() == Phase::DeclaringAfterSkat)
    }

    /// The Ansage dialog (ADR 0011, 0012, 0013): every character may
    /// declare the suit games, the grand and Null, each with the levels of
    /// this phase (before the skat the hand levels, after it the levels
    /// without Hand: suit and grand `Normal`, Schneider and Schwarz
    /// angesagt, Ouvert; Null `Normal` and `Ouvert`).
    fn declare_options(&self, content: &Content) -> DeclareOptions {
        let after_skat = self.after_skat();
        let skipped = if after_skat {
            RiskLevel::Hand
        } else {
            RiskLevel::Normal
        };
        let risks: Vec<RiskOption> = RiskLevel::ALL
            .into_iter()
            .filter(|&risk| risk != skipped)
            .map(|risk| RiskOption {
                risk,
                allowed: self.accepts(content, RunAction::Declare(Declaration::grand(risk))),
            })
            .collect();
        let lowest = risks
            .iter()
            .find(|r| r.allowed)
            .or_else(|| risks.first())
            .map_or(RiskLevel::Normal, |r| r.risk);
        let scoring = self.null_scoring(content);
        let null_risks: Vec<NullRiskOption> = NullRisk::ALL
            .into_iter()
            .filter(|risk| risk.hand() != after_skat)
            .map(|risk| NullRiskOption {
                risk,
                allowed: self.accepts(content, RunAction::Declare(Declaration::null(risk))),
                value: match &scoring {
                    NullScoring::Fixed => {
                        u64::from(crate::rules::null_value(risk.hand(), risk.ouvert()))
                    }
                    NullScoring::Formula(f) => u64::from(f.base_value) * u64::from(risk.step()),
                },
            })
            .collect();
        let null_base = match &scoring {
            NullScoring::Fixed => crate::rules::null_value(false, false),
            NullScoring::Formula(f) => f.base_value,
        };
        let games = GAMES
            .into_iter()
            .map(|game| GameOption {
                game,
                base_value: content.run.base_value(game).unwrap_or(0),
                allowed: Declaration::new(game, lowest)
                    .is_some_and(|d| self.accepts(content, RunAction::Declare(d))),
            })
            .chain(std::iter::once(GameOption {
                game: GameType::Null,
                base_value: null_base,
                allowed: null_risks.iter().any(|r| r.allowed),
            }))
            .collect();
        let can_redeal = self.accepts(content, RunAction::Redeal);
        DeclareOptions {
            games,
            risks,
            null_risks,
            null_style: matches!(scoring, NullScoring::Formula(_)),
            can_pick_up_skat: self.accepts(content, RunAction::PickUpSkat),
            ouvert_required: self.game.as_ref().is_some_and(|a| a.rules.require_ouvert),
            can_redeal,
            redeals_left: self.redeals_left(content),
            hand_required: self.requires_hand(),
            ramsch_offered: self.ramsch_offered()
                && self.accepts(content, RunAction::Declare(Declaration::Ramsch)),
            skat_pickups_left: self.skat_pickups_left(),
        }
    }

    /// The count of "Der Zähler" from what the player knows.
    fn count_view(&self) -> Option<CountView> {
        let active = self.game.as_ref()?;
        if !active.rules.show_count {
            return None;
        }
        let game = &active.game;
        // With the sign: a "Ramponiert" card counts against its taker.
        let points = game.trick_signed_points();
        let known_skat = game.skat_picked_up() && game.phase() != Phase::Discarding;
        let skat_points: i64 = game.skat().iter().map(|c| c.signed_points()).sum();
        let mut seen: Vec<DealtCard> = game.hand(Seat::DECLARER).to_vec();
        seen.extend(game.tricks().iter().flat_map(|t| t.cards));
        seen.extend(game.current_trick().iter().copied());
        if known_skat {
            seen.extend(game.skat());
        }
        // A skat card shown by "Spickzettel", a card given back ("Drei
        // drücken").
        seen.extend(active.skat_shown.iter().copied());
        seen.extend(game.returned().map(|(_, c)| c));
        let unseen: i64 = game
            .deck_cards()
            .iter()
            .filter(|c| !seen.contains(c))
            .map(|c| c.signed_points())
            .sum();
        let own = points[0] + if known_skat { skat_points } else { 0 };
        let seats = [own, points[1], points[2]].map(non_negative);
        Some(CountView {
            declarer: seats[0],
            defenders: seats[1] + seats[2],
            unseen: non_negative(unseen),
            seats,
        })
    }

    pub(super) fn game_view(&self, content: &Content) -> Option<GameView> {
        if self.phase != RunPhase::Game {
            return None;
        }
        let active = self.game.as_ref()?;
        let game = &active.game;
        let declaration = game.declaration();
        let phase = match game.phase() {
            Phase::Declaring => TablePhase::Declaring,
            Phase::Discarding => TablePhase::Discarding,
            Phase::DeclaringAfterSkat => TablePhase::DeclaringAfterSkat,
            Phase::Playing => TablePhase::Playing,
            Phase::Finished(_) => TablePhase::Finished,
        };
        let order_game = declaration.map_or(GameType::Grand, |d| d.game());
        let legal = if active.awaiting_re {
            Vec::new()
        } else {
            game.legal_moves(Seat::DECLARER)
        };
        let skat = game.skat();
        let held = game.hand(Seat::DECLARER);
        let discarding = phase == TablePhase::Discarding && !active.awaiting_re;
        let discard_count = game.discard_count();
        let drawn = game.drawn().filter(|_| phase == TablePhase::Discarding);
        let mut hand: Vec<DealtCard> = held.to_vec();
        hand.sort_by_key(|c| display_key(order_game, c));
        let hand = hand
            .into_iter()
            .map(|card| CardView {
                legal: legal.contains(&card),
                from_skat: phase == TablePhase::Discarding && skat.contains(&card),
                drawn_from: drawn.filter(|(_, c)| c.id == card.id).map(|(seat, _)| seat),
                discardable: discarding && discardable_of(held, card, discard_count),
                can_rise: self.can_rise(content, card),
                can_mirror: self.can_mirror(content, card),
                ..self.card_view(content, card)
            })
            .collect();
        let trick = game
            .current_trick()
            .iter()
            .enumerate()
            .map(|(i, &card)| PlayedCard {
                seat: seat_at(game.trick_lead(), i),
                card: self.card_view(content, card),
            })
            .collect();
        let last_trick = game.tricks().last().map(|t| TrickView {
            cards: t
                .cards
                .iter()
                .enumerate()
                .map(|(i, &card)| PlayedCard {
                    seat: t.seat_of(i),
                    card: self.card_view(content, card),
                })
                .collect(),
            winner: t.winner,
        });
        let declarer_tricks = game
            .tricks()
            .iter()
            .filter(|t| t.winner.is_declarer())
            .count();
        let known_skat = (game.skat_picked_up() && phase != TablePhase::Discarding)
            .then(|| skat.iter().map(|&c| self.card_view(content, c)).collect());
        let mut revealed: Vec<RevealedCard> = self
            .revealed_holders()
            .into_iter()
            .map(|(card, holder)| RevealedCard {
                source: RevealSource::CardMod(self.reveal_mod(content, card)),
                card: self.card_view(content, card),
                holder,
            })
            .collect();
        // The hand `RevealDefenderHand` shows, in display order, like the player's.
        let mut peeked = active.peeked.clone();
        peeked.sort_by_key(|(card, seat)| {
            if active.shows_hand(*seat) {
                display_key(order_game, card)
            } else {
                (0, 0, 0)
            }
        });
        for &(card, seat) in &peeked {
            // The copy of its holder now (a swap changes the edition).
            let card = game.current(card);
            if !revealed.iter().any(|r| r.card.card.id == card.id) {
                revealed.push(RevealedCard {
                    card: self.card_view(content, card),
                    holder: game.holder_of(card),
                    source: if active.shows_hand(seat) {
                        RevealSource::Onlooker
                    } else {
                        RevealSource::Kibitz
                    },
                });
            }
        }
        // The card given back ("Drei drücken"): the player knows who holds
        // it until it is played.
        if let Some((_, card)) = game.returned() {
            let card = game.current(card);
            if !revealed.iter().any(|r| r.card.card.id == card.id) {
                revealed.push(RevealedCard {
                    card: self.card_view(content, card),
                    holder: game.holder_of(card),
                    source: RevealSource::Returned,
                });
            }
        }
        let skat_swap_cards: Vec<CardView> = self
            .skat_swap_cards()
            .into_iter()
            .map(|c| self.card_view(content, c))
            .collect();
        let can_take_from_skat = !skat_swap_cards.is_empty()
            && game.hand(Seat::DECLARER).iter().any(|c| {
                self.accepts(
                    content,
                    RunAction::TakeFromSkat {
                        give: c.id,
                        take: None,
                    },
                )
            });
        let skat_swap_cards = if can_take_from_skat {
            skat_swap_cards
        } else {
            Vec::new()
        };
        Some(GameView {
            number: self.series.games_played + 1,
            phase,
            forehand: game.forehand(),
            positions: Seat::ALL.iter().map(|&s| game.position(s)).collect(),
            declaration,
            hand,
            hand_sizes: Seat::ALL.iter().map(|&s| game.hand(s).len()).collect(),
            trick_lead: game.trick_lead(),
            trick,
            last_trick,
            tricks_played: game.tricks().len(),
            declarer_tricks,
            defender_tricks: game.tricks().len() - declarer_tricks,
            skat: known_skat,
            your_turn: !active.awaiting_re
                && active.pending_rise.is_none()
                && game.current_seat() == Some(Seat::DECLARER),
            kontra: active.kontra,
            awaiting_kontra_answer: active.awaiting_re,
            kontra_possible: active.rules.kontra_source.is_some(),
            win_threshold: active.rules.thresholds.win,
            declare: matches!(
                phase,
                TablePhase::Declaring | TablePhase::DeclaringAfterSkat
            )
            .then(|| self.declare_options(content)),
            revealed,
            wild_settled: active
                .named
                .iter()
                .filter_map(|&(card, printed)| {
                    let kind = game.deck_cards().iter().find(|c| c.id == card)?.kind;
                    Some(SettledWild {
                        card,
                        kind,
                        printed,
                    })
                })
                .collect(),
            boss_rule: active
                .boss_rule
                .as_ref()
                .map(|id| self.boss_info(content, id)),
            backs: Seat::ALL.map(|seat| self.back_of_seat(content, seat)),
            count: self.count_view(),
            can_take_from_skat,
            skat_swap_cards,
            skat_peek: (active.skat_peeked && !game.skat_picked_up())
                .then(|| skat.iter().map(|&c| self.card_view(content, c)).collect()),
            seat_tricks: game.seat_tricks(),
            value_preview: self.value_preview(content),
            can_settle: self.can_settle(content),
            discard_count,
            skat_shown: active
                .skat_shown
                .iter()
                .filter(|c| skat.contains(c))
                .map(|&c| self.card_view(content, c))
                .collect(),
            discard_three_ready: self.discard_three_ready(),
            rise_choice: self.rise_choice_view(content),
        })
    }

    /// The modification that shows the holder of `card` ("Durchsichtig"):
    /// on the copy's own edition, else "Durchsichtig" (a mark for this game
    /// only, "Glasauge"). Never the player's edition for a defender's copy
    /// (ADR 0015).
    fn reveal_mod(&self, content: &Content, card: DealtCard) -> CardModId {
        let editions = self.editions();
        let reveals = |m: &&CardModId| {
            content
                .card_mod(m)
                .is_some_and(|def| def.passive.contains(&crate::content::Effect::RevealHolder))
        };
        editions
            .of(card.owner)
            .into_iter()
            .filter_map(|e| e.get(card.id))
            .find_map(|d| d.mods.iter().find(reveals))
            .cloned()
            .unwrap_or_else(|| CardModId::new("transparent"))
    }
}

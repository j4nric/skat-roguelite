//! Using a consumable (Schnaps): its target, its effect on the game in
//! progress or the next one, and the plan for the next pub.

use crate::cards::{Card, PlayingCard, Rank, Suit};
use crate::content::{ConsumableAction, ConsumableId, Content, NextGameEffect};
use crate::deck::{CardKind, DealtCard, DeckError};
use crate::game::Phase;

use super::{Run, RunError, RunEvent, RunPhase, draw_weighted, trade};

impl Run {
    /// The consumable in `slot` and its definition, if it may be used at
    /// all now (not locked by a Spelunke malus): what
    /// [`Run::use_consumable`] checks first.
    pub(super) fn consumable_at<'c>(
        &self,
        content: &'c Content,
        slot: usize,
    ) -> Result<(&ConsumableId, &'c crate::content::ConsumableDef), RunError> {
        let id = self.consumables.get(slot).ok_or(RunError::InvalidSlot)?;
        let def = content
            .consumable(id)
            .ok_or_else(|| RunError::UnknownContent(id.to_string()))?;
        if def.category == crate::content::ConsumableCategory::Schnapps
            && self.phase == RunPhase::Game
            && self.malus(content).is_some_and(|m| m.lock_schnapps)
        {
            return Err(RunError::ConsumablesLocked);
        }
        Ok((id, def))
    }

    pub(super) fn use_consumable(
        &mut self,
        content: &Content,
        slot: usize,
        target: Option<u32>,
        suit: Option<Suit>,
        events: &mut Vec<RunEvent>,
    ) -> Result<(), RunError> {
        let (id, def) = self.consumable_at(content, slot)?;
        let id = id.clone();
        match &def.action {
            ConsumableAction::CleanseCard => {
                if self.phase != RunPhase::Shop {
                    return Err(RunError::WrongPhase);
                }
                let target = target.ok_or(RunError::TargetRequired)?;
                let change = self.cleanse(content, target)?;
                events.push(RunEvent::DeckChanged(change));
            }
            ConsumableAction::AddCardMod(card_mod) => {
                if self.phase != RunPhase::Shop {
                    return Err(RunError::WrongPhase);
                }
                let target = target.ok_or(RunError::TargetRequired)?;
                let card = self
                    .edition
                    .get(target)
                    .ok_or(DeckError::UnknownCard(target))?;
                if card.kind == CardKind::Blank {
                    return Err(DeckError::BlankCannotBeModified.into());
                }
                let card_mod = match card_mod {
                    Some(m) => m.clone(),
                    None => {
                        let options = self.card_mod_pool_for(content, card);
                        draw_weighted(&mut self.misc_rng, options, 1)
                            .pop()
                            .ok_or(RunError::NoModAvailable)?
                    }
                };
                let value = content.card_mod(&card_mod).map_or(1, |d| d.default_value());
                self.attach_mod(content, target, &card_mod, value, events)?;
            }
            ConsumableAction::ApplyCardMod { card_mod, value } => {
                if self.phase != RunPhase::Shop {
                    return Err(RunError::WrongPhase);
                }
                let target = target.ok_or(RunError::TargetRequired)?;
                self.attach_mod(content, target, card_mod, *value, events)?;
            }
            ConsumableAction::ExtraRedeals(n) => {
                if self.phase != RunPhase::Game {
                    return Err(RunError::WrongPhase);
                }
                self.series.extra_redeals = self.series.extra_redeals.saturating_add(*n);
                events.push(RunEvent::RedealsAdded {
                    redeals: *n,
                    redeals_left: self.redeals_left(content),
                });
            }
            ConsumableAction::Redeal => self.redeal(content, events)?,
            ConsumableAction::UndoRedeal => self.undo_redeal(events)?,
            ConsumableAction::PeekSkat => self.peek_skat(events)?,
            ConsumableAction::NextGame(effect) => {
                let before_declaration = self.phase == RunPhase::Shop
                    || self.declaring().is_ok()
                    || self.game.as_ref().is_some_and(|a| {
                        matches!(
                            a.game.phase(),
                            Phase::Discarding | Phase::DeclaringAfterSkat
                        ) && a.game.declaration().is_none()
                    });
                if !before_declaration {
                    return Err(RunError::WrongPhase);
                }
                let duplicate = *effect == NextGameEffect::SingleLoss
                    && self.pending_next_game.iter().any(|(_, e)| e == effect);
                if duplicate {
                    return Err(RunError::WrongPhase);
                }
                self.pending_next_game.push((id.clone(), *effect));
            }
            ConsumableAction::ChalkCard => {
                let target = target.ok_or(RunError::TargetRequired)?;
                let suit = suit.ok_or(RunError::TargetRequired)?;
                self.chalk(target, suit, events)?;
            }
            ConsumableAction::RedealNextGame(effect) => {
                self.redeal(content, events)?;
                self.pending_next_game.push((id.clone(), *effect));
            }
            ConsumableAction::RerollRoutes => {
                if self.phase != RunPhase::Route {
                    return Err(RunError::WrongPhase);
                }
                self.reroll_routes(content);
                events.push(RunEvent::RoutesRerolled);
            }
            ConsumableAction::NextPub { redeals, target } => {
                if !matches!(
                    self.phase,
                    RunPhase::Shop | RunPhase::Encounter | RunPhase::Route
                ) {
                    return Err(RunError::WrongPhase);
                }
                self.plan_next_pub(*redeals, *target, events);
            }
            ConsumableAction::ClearAccount => {
                if !self.between_games() {
                    return Err(RunError::WrongPhase);
                }
                if self.series.score >= 0 {
                    return Err(RunError::ConditionNotMet);
                }
                let before = self.series.score;
                self.series.score = 0;
                events.push(RunEvent::AccountCleared { before, score: 0 });
            }
            ConsumableAction::ExtraGames(n) => {
                if !self.between_games() {
                    return Err(RunError::WrongPhase);
                }
                if self.series.is_boss_pub() {
                    return Err(RunError::NotInBossPub);
                }
                self.series.game_limit = self.series.game_limit.saturating_add(*n);
                events.push(RunEvent::GamesAdded {
                    games: *n,
                    game_limit: self.series.game_limit,
                    games_left: self.series.games_left(),
                });
            }
            ConsumableAction::NextGameIfBehind(effect) => {
                self.declaring()?;
                if !self.behind() {
                    return Err(RunError::ConditionNotMet);
                }
                self.pending_next_game.push((id.clone(), *effect));
            }
            ConsumableAction::NextGameCursed(effect) => {
                self.declaring()?;
                if self.trade_curse_targets(content, &[]).is_empty() {
                    return Err(RunError::NoModAvailable);
                }
                self.pending_next_game.push((id.clone(), *effect));
                self.trade_curse(content, &[], trade::CurseCause::Schnapps, events)?;
            }
        }
        self.consumables.remove(slot);
        events.push(RunEvent::ConsumableUsed { consumable: id });
        Ok(())
    }

    /// Whether the pub account is below half its Zielwert ("Eierlikör").
    pub(super) fn behind(&self) -> bool {
        i128::from(self.series.score).saturating_mul(2) < i128::from(self.series.target)
    }

    /// Add to the plan for the next pub.
    pub(super) fn plan_next_pub(&mut self, redeals: i32, target: i32, events: &mut Vec<RunEvent>) {
        self.next_pub.redeals = self.next_pub.redeals.saturating_add(redeals);
        self.next_pub.target = self.next_pub.target.saturating_add(target);
        events.push(RunEvent::NextPubPlanned {
            redeals: self.next_pub.redeals,
            target: self.next_pub.target,
        });
    }

    /// "Kurzer": show the player the untouched skat while he still holds
    /// ten cards and has not declared. Once per game.
    fn peek_skat(&mut self, events: &mut Vec<RunEvent>) -> Result<(), RunError> {
        let active = self.declaring()?;
        if active.skat_peeked {
            return Err(RunError::WrongPhase);
        }
        active.skat_peeked = true;
        events.push(RunEvent::SkatPeeked {
            skat: active.game.skat(),
        });
        Ok(())
    }

    /// "Blue Curaçao": repaint the card `id` of the player's hand into `suit`
    /// until the game ends. Only before the declaration, never a jack (it
    /// is trump by its rank) or a blank, and only into another suit.
    fn chalk(&mut self, id: u32, suit: Suit, events: &mut Vec<RunEvent>) -> Result<(), RunError> {
        let active = self.active_mut()?;
        if active.rules.ramsch || active.game.declaration().is_some() {
            return Err(RunError::WrongPhase);
        }
        if active.awaiting_re {
            return Err(RunError::AwaitingKontraAnswer);
        }
        let old = Run::declarer_card(active, id)?;
        let face = old.face().ok_or(RunError::CannotChalk)?;
        if face.rank == Rank::Jack || face.suit == suit {
            return Err(RunError::CannotChalk);
        }
        let new = DealtCard {
            kind: CardKind::Normal(Card::new(suit, face.rank)),
            ..old
        };
        let game_events = active.game.chalk(old, new)?;
        if !active.chalked.iter().any(|(c, _)| *c == id) {
            active.chalked.push((id, face.suit));
        }
        events.extend(game_events.into_iter().map(RunEvent::Game));
        Ok(())
    }
}

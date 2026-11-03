//! The Schnaps after a won pub (Schnaps-Leiste, DESIGN 8.9, decision of
//! the user on 03.11.2026).
//!
//! Every won pub that does not end the run (normal pub, Spelunke, boss
//! pub) gives one random Schnaps, drawn by rarity like the Tresen from the
//! Schnäpse the character meets ([`Run::schnapps_pool`]), with a generator
//! of its own (`schnapps_rng`), so that deals, offers, routes and
//! encounters stay as they were. With a free slot it goes there at once;
//! with every slot full the player chooses: take it and give up an owned
//! one, or sell it for half its price ([`super::RunAction::ChooseSchnappsReward`]).
//! The choice comes before anything else, also before the relic pick of a
//! boss pub and the loot of a Spelunke, which follow as before.

use super::{Run, RunError, RunEvent, draw_weighted, sell_price};
use crate::content::{ConsumableId, Content};

impl Run {
    /// The Schnaps of a won pub waiting for the player's choice, if every
    /// slot was full ([`super::RunAction::ChooseSchnappsReward`]).
    #[must_use]
    pub fn schnapps_reward(&self) -> Option<&ConsumableId> {
        self.schnapps_reward.as_ref()
    }

    /// Whether a Schnaps slot is free.
    pub(super) fn consumable_slot_free(&self, content: &Content) -> bool {
        self.consumables.len() < usize::try_from(content.run.consumable_slots).unwrap_or(0)
    }

    /// Draw the Schnaps of a won pub: into a free slot, or the choice opens.
    /// Nothing if no Schnaps is in the character's pool.
    pub(super) fn reward_schnapps(&mut self, content: &Content, events: &mut Vec<RunEvent>) {
        let pool = self.schnapps_pool(content);
        let Some(id) = draw_weighted(&mut self.schnapps_rng, pool, 1).pop() else {
            return;
        };
        let slots_full = !self.consumable_slot_free(content);
        events.push(RunEvent::SchnappsRewarded {
            consumable: id.clone(),
            slots_full,
        });
        if slots_full {
            self.schnapps_reward = Some(id);
        } else {
            self.consumables.push(id);
        }
    }

    /// [`super::RunAction::ChooseSchnappsReward`].
    pub(super) fn choose_schnapps_reward(
        &mut self,
        content: &Content,
        give_up: Option<usize>,
        events: &mut Vec<RunEvent>,
    ) -> Result<(), RunError> {
        let id = self.schnapps_reward.clone().ok_or(RunError::WrongPhase)?;
        match give_up {
            Some(slot) => {
                let slot = self
                    .consumables
                    .get_mut(slot)
                    .ok_or(RunError::InvalidSlot)?;
                let given_up = std::mem::replace(slot, id.clone());
                events.push(RunEvent::SchnappsRewardTaken {
                    consumable: id,
                    given_up,
                });
            }
            None => {
                let price = content.consumable(&id).map_or(0, |d| sell_price(d.price));
                self.coins = self.coins.saturating_add(price);
                events.push(RunEvent::ConsumableSold {
                    consumable: id,
                    price,
                });
                self.settle_debt(events);
            }
        }
        self.schnapps_reward = None;
        Ok(())
    }
}

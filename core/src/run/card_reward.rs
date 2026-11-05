//! The card reward after a won pub (Kartenbelohnung, DESIGN 5.0, decision
//! of the user on 04.11.2026, Cockpit q24, variant A).
//!
//! After every won normal pub and Spelunke (not a boss pub, which gives
//! its relic pick) the innkeeper offers [`CardRewardConfig::offers`]
//! different Zinken. The player takes one for free and names the card it
//! goes on ([`super::RunAction::TakeCardReward`]); the same Zinke on that
//! card rises (up to its maximum), another is replaced, as in the Auslage.
//! Or he skips for [`CardRewardConfig::skip_coins`] Groschen
//! ([`super::RunAction::SkipCardReward`]).
//!
//! The offers come from the Zinken catalogue like the Auslage (DESIGN
//! 8.6): never a curse, only what the character meets and has unlocked,
//! only a Zinke some card of the edition takes now, under the cap of the
//! special cards ([`Run::special_open`]), at most one offer per special
//! card (the Marke for it included). Each offer is, with
//! [`CardRewardConfig::special_percent`], a special card (Untot,
//! Wandelkarte, Doppelgänger) while one may still come; otherwise first a
//! rarity class by [`CardRewardConfig::rarity_weights`] (only classes with
//! a Zinke left), then a Zinke of it by the character's weight for its
//! Spielart, Marken at [`CardRewardConfig::voucher_percent`] of that, and
//! the build direction's weight ([`Run::card_reward_bias`]). Values are the
//! Auslage values (`shop_value`). Drawn with the run's own generator
//! (`reward_rng`), so deals, offers, routes and encounters stay as they
//! were.
//!
//! The choice comes after the Schnaps of the won pub and before the loot
//! of a Spelunke and the route choice.
//!
//! [`CardRewardConfig::offers`]: crate::content::CardRewardConfig::offers
//! [`CardRewardConfig::skip_coins`]: crate::content::CardRewardConfig::skip_coins
//! [`CardRewardConfig::special_percent`]: crate::content::CardRewardConfig::special_percent
//! [`CardRewardConfig::rarity_weights`]: crate::content::CardRewardConfig::rarity_weights
//! [`CardRewardConfig::voucher_percent`]: crate::content::CardRewardConfig::voucher_percent

use serde::{Deserialize, Serialize};

use super::zinken::special_target;
use super::{Run, RunError, RunEvent, RunPhase, draw_weighted};
use crate::content::{CardModDef, CardModId, Content, ModKind, Rarity, in_pool};

/// The card reward of a won pub, waiting for the player.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CardReward {
    /// The Zinken on offer with their values, all different.
    pub offers: Vec<(CardModId, i32)>,
}

/// A Zinke that may be offered, with what the draw needs.
struct Candidate {
    id: CardModId,
    rarity: Rarity,
    weight: u32,
    special: Option<CardModId>,
}

impl Run {
    /// The open card reward of a won pub.
    #[must_use]
    pub const fn card_reward(&self) -> Option<&CardReward> {
        self.card_reward.as_ref()
    }

    /// The deck cards (instance ids, in id order) the Zinke at `offer` of
    /// the open card reward may go on now: the card takes it (it fits,
    /// not cursed, not at its maximum), and a special card only on a card
    /// that is no special card yet. Empty without an open reward.
    #[must_use]
    pub fn card_reward_targets(&self, content: &Content, offer: usize) -> Vec<u32> {
        let Some((card_mod, value)) = self.card_reward.as_ref().and_then(|r| r.offers.get(offer))
        else {
            return Vec::new();
        };
        self.edition
            .cards()
            .iter()
            .map(|c| c.id)
            .filter(|&card| self.reward_fits(content, card, card_mod, *value))
            .collect()
    }

    /// Whether `card_mod` with `value` may go on `card` as a card reward.
    fn reward_fits(&self, content: &Content, card: u32, card_mod: &CardModId, value: i32) -> bool {
        let Some(def) = content.card_mod(card_mod) else {
            return false;
        };
        if def.kind == ModKind::Special {
            let has_special = self.edition.get(card).is_some_and(|c| {
                c.mods
                    .iter()
                    .any(|m| content.card_mod(m).is_some_and(CardModDef::is_special))
            });
            if has_special {
                return false;
            }
        }
        self.takes(content, card, card_mod, value)
    }

    /// Whether some card of the edition takes `def` at its offer value.
    fn reward_fits_some(&self, content: &Content, def: &CardModDef) -> bool {
        let value = def.offer_value();
        self.edition
            .cards()
            .iter()
            .any(|c| self.reward_fits(content, c.id, &def.id, value))
    }

    /// The build direction's weight for `def` in the card reward, in
    /// percent (100 = plain): [`CardRewardConfig::direction_percent`] when
    /// the Zinke shares a build tag with a relic the player holds
    /// (DESIGN 8.11), else 100.
    ///
    /// [`CardRewardConfig::direction_percent`]: crate::content::CardRewardConfig::direction_percent
    #[must_use]
    pub fn card_reward_bias(&self, content: &Content, def: &CardModDef) -> u32 {
        let Some(config) = content.run.card_reward.as_ref() else {
            return 100;
        };
        let shares_tag = !def.tags.is_empty()
            && self.relics.iter().any(|id| {
                content
                    .relic(id)
                    .is_some_and(|r| r.tags.iter().any(|t| def.tags.contains(t)))
            });
        if shares_tag {
            config.direction_percent
        } else {
            100
        }
    }

    /// The Zinken and special cards the card reward may offer now, in id
    /// order, weight 0 left out.
    fn reward_candidates(&self, content: &Content) -> (Vec<Candidate>, Vec<Candidate>) {
        let Some(config) = content.run.card_reward.as_ref() else {
            return (Vec::new(), Vec::new());
        };
        let mut zinken = Vec::new();
        let mut specials = Vec::new();
        let defs = content
            .card_mods
            .values()
            .filter(|m| !m.is_curse() && m.kind != ModKind::SpecialCurse)
            .filter(|m| in_pool(&m.characters, self.character))
            .filter(|m| !self.locked_card_mods.contains(&m.id))
            .filter(|m| self.offer_open(content, m))
            .filter(|m| self.reward_fits_some(content, m));
        for def in defs {
            let mut weight = self.focus_weight(content, def.focus);
            if def.kind == ModKind::Voucher {
                weight = weight.saturating_mul(config.voucher_percent) / 100;
            }
            weight = weight.saturating_mul(self.card_reward_bias(content, def)) / 100;
            if weight == 0 {
                continue;
            }
            let candidate = Candidate {
                id: def.id.clone(),
                rarity: def.rarity,
                weight,
                special: special_target(def).cloned(),
            };
            if def.kind == ModKind::Special {
                specials.push(candidate);
            } else {
                zinken.push(candidate);
            }
        }
        (zinken, specials)
    }

    /// Draw the Zinken of a card reward (module docs).
    fn draw_card_reward(&mut self, content: &Content) -> Vec<(CardModId, i32)> {
        let Some(config) = content.run.card_reward.as_ref() else {
            return Vec::new();
        };
        let (mut zinken, mut specials) = self.reward_candidates(content);
        let mut offers = Vec::new();
        for _ in 0..config.offers {
            let special_roll = !specials.is_empty()
                && self.reward_rng.gen_range(0..100) < u64::from(config.special_percent);
            let drawn = if special_roll {
                let pool = specials.iter().map(|c| (c.id.clone(), c.weight)).collect();
                draw_weighted(&mut self.reward_rng, pool, 1).pop()
            } else {
                let classes: Vec<(Rarity, u32)> = [
                    Rarity::Common,
                    Rarity::Uncommon,
                    Rarity::Rare,
                    Rarity::Legendary,
                ]
                .into_iter()
                .filter(|r| zinken.iter().any(|c| c.rarity == *r))
                .map(|r| (r, config.rarity_weights.weight(r)))
                .collect();
                draw_weighted(&mut self.reward_rng, classes, 1)
                    .pop()
                    .and_then(|class| {
                        let pool = zinken
                            .iter()
                            .filter(|c| c.rarity == class)
                            .map(|c| (c.id.clone(), c.weight))
                            .collect();
                        draw_weighted(&mut self.reward_rng, pool, 1).pop()
                    })
            };
            let Some(id) = drawn else {
                break;
            };
            let special = zinken
                .iter()
                .chain(&specials)
                .find(|c| c.id == id)
                .and_then(|c| c.special.clone());
            let gone = |c: &Candidate| c.id == id || (special.is_some() && c.special == special);
            zinken.retain(|c| !gone(c));
            specials.retain(|c| !gone(c));
            let value = content.card_mod(&id).map_or(1, CardModDef::offer_value);
            offers.push((id, value));
        }
        offers
    }

    /// Open the card reward after a won normal pub or Spelunke. Returns
    /// `false` (nothing changed but the generator) if the content has
    /// none or no Zinke fits any card; the way goes on then.
    pub(super) fn open_card_reward(
        &mut self,
        content: &Content,
        events: &mut Vec<RunEvent>,
    ) -> bool {
        let offers = self.draw_card_reward(content);
        if offers.is_empty() {
            return false;
        }
        self.game = None;
        self.phase = RunPhase::CardReward;
        events.push(RunEvent::CardRewardOffered {
            offers: offers.clone(),
        });
        self.card_reward = Some(CardReward { offers });
        true
    }

    /// [`super::RunAction::TakeCardReward`].
    pub(super) fn take_card_reward(
        &mut self,
        content: &Content,
        offer: usize,
        card: u32,
        events: &mut Vec<RunEvent>,
    ) -> Result<(), RunError> {
        if self.phase != RunPhase::CardReward {
            return Err(RunError::WrongPhase);
        }
        let reward = self.card_reward.clone().ok_or(RunError::WrongPhase)?;
        let (card_mod, value) = reward
            .offers
            .get(offer)
            .cloned()
            .ok_or(RunError::InvalidSlot)?;
        if !self.reward_fits(content, card, &card_mod, value) {
            // The reason from the rules (unknown card, cursed, at its
            // maximum, special card taken), else it does not fit.
            self.edition
                .clone()
                .add_mod(content, card, &card_mod, value)?;
            return Err(RunError::NoModAvailable);
        }
        events.push(RunEvent::CardRewardTaken {
            offer,
            card,
            card_mod: card_mod.clone(),
            value,
        });
        self.attach_mod(content, card, &card_mod, value, events)?;
        self.card_reward = None;
        self.after_pub_rewards(content, events)
    }

    /// [`super::RunAction::SkipCardReward`].
    pub(super) fn skip_card_reward(
        &mut self,
        content: &Content,
        events: &mut Vec<RunEvent>,
    ) -> Result<(), RunError> {
        if self.phase != RunPhase::CardReward || self.card_reward.is_none() {
            return Err(RunError::WrongPhase);
        }
        let coins = content.run.card_reward.as_ref().map_or(0, |c| c.skip_coins);
        self.coins = self.coins.saturating_add(coins);
        events.push(RunEvent::CardRewardSkipped { coins });
        self.settle_debt(events);
        self.card_reward = None;
        self.after_pub_rewards(content, events)
    }

    /// Drop what the content no longer knows from the open card reward
    /// (part of [`Run::drop_unknown_content`]). With nothing left only
    /// "Überspringen" goes.
    pub(super) fn drop_unknown_card_reward(&mut self, content: &Content) {
        if let Some(reward) = self.card_reward.as_mut() {
            reward.offers.retain(|(m, _)| content.card_mod(m).is_some());
        }
    }
}

#[cfg(test)]
impl Run {
    /// Tests only: close an open card reward without a Zinke and without
    /// Groschen, as a won pub went on before the card reward (tests of
    /// other features that walk on to the route).
    pub(crate) fn pass_card_reward(&mut self, content: &Content, events: &mut Vec<RunEvent>) {
        if self.phase == RunPhase::CardReward {
            self.card_reward = None;
            self.after_pub_rewards(content, events)
                .expect("the way goes on");
        }
    }
}

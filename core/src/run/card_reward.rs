//! The card reward after a won pub (Kartenbelohnung, DESIGN 5.0; fixed
//! pairs since the user's decision of 05.11.2026, replacing Cockpit q24
//! variant A).
//!
//! After every won normal pub and Spelunke (not a boss pub, which gives
//! its relic pick) the innkeeper offers [`CardRewardConfig::offers`]
//! pairs, each a card of the player's edition with a Zinke
//! ([`RewardOffer`]). The player takes one for free
//! ([`super::RunAction::TakeCardReward`]); the same Zinke on that card
//! rises (up to its maximum), another is replaced, as in the Auslage. Or he
//! skips for [`CardRewardConfig::skip_coins`] Groschen
//! ([`super::RunAction::SkipCardReward`]).
//!
//! The Zinken come from the catalogue like the Auslage (DESIGN 8.6): never
//! a curse, only what the character meets and has unlocked, only a Zinke
//! some card of the edition takes now. Each offer is, with
//! [`CardRewardConfig::special_percent`], a special card (Untot,
//! Wandelkarte, Doppelgänger); otherwise first a rarity class by
//! [`CardRewardConfig::rarity_weights`] (only classes with a Zinke left),
//! then a Zinke of it by the character's weight for its Spielart, Marken
//! at [`CardRewardConfig::voucher_percent`] of that, and the build
//! direction's weight ([`Run::card_reward_bias`]). Then the card: drawn
//! evenly among the cards that take the Zinke now (it fits, not cursed,
//! not at its maximum; a special card only on a card without one) and are
//! in no other offer. No card twice, and no Zinke twice while another one
//! still has a free card. Values are the Auslage values (`shop_value`).
//! Drawn with the run's own generator (`reward_rng`), so deals, offers,
//! routes and encounters stay as they were.
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

use super::{Run, RunError, RunEvent, RunPhase, draw_weighted};
use crate::content::{CardModDef, CardModId, Content, ModKind, Rarity, in_pool};

/// The card reward of a won pub, waiting for the player.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(from = "CardRewardSave")]
pub struct CardReward {
    /// The pairs on offer: no card twice, no Zinke twice if possible.
    pub offers: Vec<RewardOffer>,
    /// Loaded from a save of before 05.11.2026 (Zinken without cards):
    /// drawn again when the run is loaded ([`Run::drop_unknown_content`]).
    #[serde(skip)]
    stale: bool,
}

impl CardReward {
    /// A reward with these offers.
    #[must_use]
    pub const fn new(offers: Vec<RewardOffer>) -> Self {
        Self {
            offers,
            stale: false,
        }
    }
}

/// One offer of the card reward: a card of the player's edition with a
/// Zinke and its value.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RewardOffer {
    /// The card (instance id).
    pub card: u32,
    /// The Zinke.
    pub card_mod: CardModId,
    /// Its value.
    pub value: i32,
}

/// The card reward as saved: since 05.11.2026 pairs, before that bare
/// Zinken with their values.
#[derive(Deserialize)]
struct CardRewardSave {
    offers: Vec<SavedOffer>,
}

/// One offer as saved.
#[derive(Deserialize)]
#[serde(untagged)]
enum SavedOffer {
    /// Since 05.11.2026.
    Pair(RewardOffer),
    /// Before: a Zinke and its value, the card chosen by the player. Read
    /// and dropped: the reward is drawn again.
    Zinke(serde::de::IgnoredAny, serde::de::IgnoredAny),
}

impl From<CardRewardSave> for CardReward {
    fn from(save: CardRewardSave) -> Self {
        let stale = save
            .offers
            .iter()
            .any(|o| matches!(o, SavedOffer::Zinke(..)));
        let offers = save
            .offers
            .into_iter()
            .filter_map(|o| match o {
                SavedOffer::Pair(offer) => Some(offer),
                SavedOffer::Zinke(..) => None,
            })
            .collect();
        Self { offers, stale }
    }
}

/// A Zinke that may be offered, with what the draw needs.
struct Candidate {
    id: CardModId,
    rarity: Rarity,
    weight: u32,
    special: bool,
}

impl Run {
    /// The open card reward of a won pub.
    #[must_use]
    pub const fn card_reward(&self) -> Option<&CardReward> {
        self.card_reward.as_ref()
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
                special: def.kind == ModKind::Special,
            };
            if candidate.special {
                specials.push(candidate);
            } else {
                zinken.push(candidate);
            }
        }
        (zinken, specials)
    }

    /// The cards `card_mod` may go on as a card reward, in id order,
    /// without the cards in `used`.
    fn reward_cards(&self, content: &Content, card_mod: &CardModId, used: &[u32]) -> Vec<u32> {
        let value = content
            .card_mod(card_mod)
            .map_or(1, CardModDef::offer_value);
        self.edition
            .cards()
            .iter()
            .map(|c| c.id)
            .filter(|card| !used.contains(card))
            .filter(|&card| self.reward_fits(content, card, card_mod, value))
            .collect()
    }

    /// Draw one Zinke from `zinken` and `specials` (module docs).
    fn draw_reward_zinke(
        &mut self,
        content: &Content,
        zinken: &[&Candidate],
        specials: &[&Candidate],
    ) -> Option<CardModId> {
        let config = content.run.card_reward.as_ref()?;
        let special_roll = !specials.is_empty()
            && self.reward_rng.gen_range(0..100) < u64::from(config.special_percent);
        if special_roll {
            let pool = specials.iter().map(|c| (c.id.clone(), c.weight)).collect();
            return draw_weighted(&mut self.reward_rng, pool, 1).pop();
        }
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
        let class = draw_weighted(&mut self.reward_rng, classes, 1).pop()?;
        let pool = zinken
            .iter()
            .filter(|c| c.rarity == class)
            .map(|c| (c.id.clone(), c.weight))
            .collect();
        draw_weighted(&mut self.reward_rng, pool, 1).pop()
    }

    /// Draw the offers of a card reward (module docs).
    fn draw_card_reward(&mut self, content: &Content) -> Vec<RewardOffer> {
        let Some(config) = content.run.card_reward.as_ref() else {
            return Vec::new();
        };
        let (zinken, specials) = self.reward_candidates(content);
        let mut offers: Vec<RewardOffer> = Vec::new();
        // "Knausriger Wirt" (long night): fewer offers, at least one.
        let count = config
            .offers
            .saturating_sub(self.night_fewer_offers(content))
            .max(1);
        for _ in 0..count {
            let used: Vec<u32> = offers.iter().map(|o| o.card).collect();
            let open = |c: &&Candidate, fresh: bool| {
                (!fresh || !offers.iter().any(|o| o.card_mod == c.id))
                    && !self.reward_cards(content, &c.id, &used).is_empty()
            };
            // A Zinke not offered yet if one has a free card, else any.
            let mut pick: Option<(Vec<&Candidate>, Vec<&Candidate>)> = None;
            for fresh in [true, false] {
                let z: Vec<&Candidate> = zinken.iter().filter(|c| open(c, fresh)).collect();
                let s: Vec<&Candidate> = specials.iter().filter(|c| open(c, fresh)).collect();
                if !z.is_empty() || !s.is_empty() {
                    pick = Some((z, s));
                    break;
                }
            }
            let Some((z, s)) = pick else {
                break;
            };
            let Some(card_mod) = self.draw_reward_zinke(content, &z, &s) else {
                break;
            };
            let cards = self.reward_cards(content, &card_mod, &used);
            let Ok(len) = u64::try_from(cards.len()) else {
                break;
            };
            let index = usize::try_from(self.reward_rng.gen_range(0..len)).unwrap_or(0);
            let Some(&card) = cards.get(index) else {
                break;
            };
            let value = content
                .card_mod(&card_mod)
                .map_or(1, CardModDef::offer_value);
            offers.push(RewardOffer {
                card,
                card_mod,
                value,
            });
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
        self.card_reward = Some(CardReward::new(offers));
        true
    }

    /// [`super::RunAction::TakeCardReward`].
    pub(super) fn take_card_reward(
        &mut self,
        content: &Content,
        offer: usize,
        events: &mut Vec<RunEvent>,
    ) -> Result<(), RunError> {
        if self.phase != RunPhase::CardReward {
            return Err(RunError::WrongPhase);
        }
        let reward = self.card_reward.clone().ok_or(RunError::WrongPhase)?;
        let RewardOffer {
            card,
            card_mod,
            value,
        } = reward
            .offers
            .get(offer)
            .cloned()
            .ok_or(RunError::InvalidSlot)?;
        if !self.reward_fits(content, card, &card_mod, value) {
            // The reason from the rules (unknown card, cursed, at its
            // maximum), else it does not fit.
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

    /// Whether the offer at `offer` of the open card reward can be taken
    /// now (its card still takes its Zinke).
    #[must_use]
    pub fn card_reward_open(&self, content: &Content, offer: usize) -> bool {
        self.card_reward
            .as_ref()
            .and_then(|r| r.offers.get(offer))
            .is_some_and(|o| self.reward_fits(content, o.card, &o.card_mod, o.value))
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
    /// "Überspringen" goes. An open reward from a save of before
    /// 05.11.2026 (Zinken without cards) is drawn again as pairs with the
    /// run's reward generator, so the same save always gives the same
    /// offers.
    pub(super) fn drop_unknown_card_reward(&mut self, content: &Content) {
        if self.card_reward.as_ref().is_some_and(|r| r.stale) {
            let offers = self.draw_card_reward(content);
            self.card_reward = Some(CardReward::new(offers));
        }
        let edition = &self.edition;
        if let Some(reward) = self.card_reward.as_mut() {
            reward.offers.retain(|o| {
                content.card_mod(&o.card_mod).is_some() && edition.get(o.card).is_some()
            });
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

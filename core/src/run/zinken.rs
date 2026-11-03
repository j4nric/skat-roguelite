//! Zinken at the Tresen (DESIGN 8.6, decided by the user on 02.11.2026).
//!
//! Card modifications are no longer consumables. The Tresen sells them
//! three ways, all paid like everything at the Tresen (the discount of
//! "Der Wirt", credit of "Deckel anschreiben"):
//!
//! - **Auslage** ([`CardOffer`]): always [`ZinkenConfig::offers`] offers,
//!   each a concrete card of the player's edition with a concrete
//!   modification and its value, visible before buying. Drawn with the
//!   run's own Zinken generator; never a curse, never a special card,
//!   never a modification the card cannot take now. Cards without a
//!   modification are preferred ([`ZinkenConfig::bare_weight`] against
//!   [`ZinkenConfig::modded_weight`]), vouchers are rarer
//!   ([`ZinkenConfig::voucher_percent`]). One card at most once per
//!   Auslage.
//! - **Neu auslosen**: the Auslage drawn again; the price rises by
//!   [`ZinkenConfig::reroll_step`] with each further time at the same
//!   Tresen.
//! - **Kartenmaler** (once per Tresen, free at the Werkstatt): the player
//!   pays and names a card, the painter draws
//!   [`ZinkenConfig::painter_choices`] modifications that fit it, the
//!   player takes one ([`PainterChoice`], [`RunAction::ChoosePaint`]).
//!   Until then the run accepts nothing else.
//! - **Karte reinigen** (once per Tresen): every modification off a card,
//!   curses included (what "Frisches Deck" did as a consumable).
//!
//! [`ZinkenConfig::offers`]: crate::content::ZinkenConfig::offers
//! [`ZinkenConfig::bare_weight`]: crate::content::ZinkenConfig::bare_weight
//! [`ZinkenConfig::modded_weight`]: crate::content::ZinkenConfig::modded_weight
//! [`ZinkenConfig::voucher_percent`]: crate::content::ZinkenConfig::voucher_percent
//! [`ZinkenConfig::reroll_step`]: crate::content::ZinkenConfig::reroll_step
//! [`ZinkenConfig::painter_choices`]: crate::content::ZinkenConfig::painter_choices
//! [`RunAction::ChoosePaint`]: super::RunAction::ChoosePaint

use serde::{Deserialize, Serialize};

use super::{EncounterChoice, Run, RunError, RunEvent, RunPhase, draw_weighted};
use crate::content::{CardModDef, CardModId, Content, ModKind, in_pool};
use crate::deck::CardKind;

/// An offer of the Auslage: a card of the player's edition with a card
/// modification and its value.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CardOffer {
    /// The card (instance id).
    pub card: u32,
    /// The modification.
    pub card_mod: CardModId,
    /// Its value.
    pub value: i32,
    /// Price before discounts: the modification's price, more on a card of
    /// [`crate::content::ZinkenConfig::premium_ranks`].
    pub price: u32,
}

/// The Kartenmaler's choice, waiting for the player.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PainterChoice {
    /// The card he paints (instance id).
    pub card: u32,
    /// The modifications with their values, one to take.
    pub options: Vec<(CardModId, i32)>,
    /// Hired at the Werkstatt: taking one resolves the encounter.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub encounter: bool,
}

impl Run {
    /// The Kartenmaler's choice waiting for the player, if any.
    #[must_use]
    pub const fn painter(&self) -> Option<&PainterChoice> {
        self.painter.as_ref()
    }

    /// Modifications the Auslage and the Kartenmaler may draw, with their
    /// weight: no curse, no special card, for the character, unlocked;
    /// rarity times the character's weight for the Spielart, vouchers
    /// [`crate::content::ZinkenConfig::voucher_percent`] of that. In id
    /// order, weight 0 left out.
    pub(super) fn zinken_pool(&self, content: &Content) -> Vec<(CardModId, u32)> {
        let config = &content.run.shop.zinken;
        let rarity = &content.run.shop.rarity_weights;
        content
            .card_mods
            .values()
            .filter(|m| !m.is_curse() && !m.is_special())
            .filter(|m| in_pool(&m.characters, self.character))
            .filter(|m| !self.locked_card_mods.contains(&m.id))
            .map(|m| {
                let mut weight =
                    rarity.percent_weight(m.rarity, self.focus_weight(content, m.focus));
                if m.kind == ModKind::Voucher {
                    weight = weight.saturating_mul(config.voucher_percent) / 100;
                }
                (m.id.clone(), weight)
            })
            .filter(|&(_, weight)| weight > 0)
            .collect()
    }

    /// Whether `card_mod` with `value` may go on the deck card `card` now:
    /// it fits the printed card and the rules of [`crate::deck::Edition`]
    /// take it (not on a cursed card, not past its maximum). Tried on a
    /// copy of the edition.
    fn takes(&self, content: &Content, card: u32, card_mod: &CardModId, value: i32) -> bool {
        let Some(def) = content.card_mod(card_mod) else {
            return false;
        };
        let fits = self
            .edition
            .get(card)
            .is_some_and(|c| c.kind != CardKind::Blank && def.fits(c.kind));
        fits && self
            .edition
            .clone()
            .add_mod(content, card, card_mod, value)
            .is_ok()
    }

    /// The price of an offer before discounts.
    fn card_offer_price(content: &Content, def: &CardModDef, kind: CardKind) -> u32 {
        let config = &content.run.shop.zinken;
        let premium = kind
            .face()
            .is_some_and(|face| config.premium_ranks.contains(&face.rank));
        def.price + if premium { config.premium } else { 0 }
    }

    /// Draw a new Auslage into the open Tresen and say so.
    pub(super) fn draw_card_offers(&mut self, content: &Content, events: &mut Vec<RunEvent>) {
        let config = &content.run.shop.zinken;
        let pool = self.zinken_pool(content);
        let mut pairs: Vec<((u32, CardModId), u32)> = Vec::new();
        for card in self.edition.cards() {
            let card_weight = if card.mods.is_empty() {
                config.bare_weight
            } else {
                config.modded_weight
            };
            for (card_mod, weight) in &pool {
                let value = content
                    .card_mod(card_mod)
                    .map_or(1, CardModDef::offer_value);
                if self.takes(content, card.id, card_mod, value) {
                    pairs.push((
                        (card.id, card_mod.clone()),
                        weight.saturating_mul(card_weight),
                    ));
                }
            }
        }
        let mut offers: Vec<Option<CardOffer>> = Vec::new();
        for _ in 0..config.offers {
            let Some((card, card_mod)) =
                draw_weighted(&mut self.zinken_rng, pairs.clone(), 1).pop()
            else {
                break;
            };
            pairs.retain(|((c, _), _)| *c != card);
            let (Some(def), Some(deck_card)) =
                (content.card_mod(&card_mod), self.edition.get(card))
            else {
                continue;
            };
            offers.push(Some(CardOffer {
                card,
                value: def.offer_value(),
                price: Run::card_offer_price(content, def, deck_card.kind),
                card_mod,
            }));
        }
        if let Some(shop) = self.shop.as_mut() {
            shop.card_offers.clone_from(&offers);
        }
        events.push(RunEvent::CardOffersDrawn { offers });
    }

    /// What an offer of the Auslage costs now: the Tresen discount ("Der
    /// Wirt", [`Run::shop_price`]), then the character's price for Zinken
    /// on this card ([`Run::zinken_price`], "Bruderherz": half on Rot).
    #[must_use]
    pub fn card_offer_cost(&self, content: &Content, offer: &CardOffer) -> u32 {
        let price = self.shop_price(content, offer.price);
        self.zinken_price(content, offer.card, price)
    }

    /// What the Kartenmaler costs now for `card`: like
    /// [`Run::card_offer_cost`].
    #[must_use]
    pub fn painter_cost(&self, content: &Content, card: u32) -> u32 {
        let price = self.shop_price(content, content.run.shop.zinken.painter_price);
        self.zinken_price(content, card, price)
    }

    /// Price of the next "Neu auslosen" before discounts.
    #[must_use]
    pub fn card_reroll_price(&self, content: &Content) -> u32 {
        let config = &content.run.shop.zinken;
        let done = self.shop.as_ref().map_or(0, |s| s.card_rerolls);
        config
            .reroll_cost
            .saturating_add(config.reroll_step.saturating_mul(done))
    }

    pub(super) fn buy_card_offer(
        &mut self,
        content: &Content,
        slot: usize,
        events: &mut Vec<RunEvent>,
    ) -> Result<(), RunError> {
        let offer = self
            .shop_mut()?
            .card_offers
            .get(slot)
            .cloned()
            .flatten()
            .ok_or(RunError::InvalidSlot)?;
        let price = self.card_offer_cost(content, &offer);
        if !self.can_pay(content, price) {
            return Err(RunError::NotEnoughCoins);
        }
        if !self.takes(content, offer.card, &offer.card_mod, offer.value) {
            // The reason from the rules (cursed, at its maximum).
            let mut probe = self.edition.clone();
            probe.add_mod(content, offer.card, &offer.card_mod, offer.value)?;
            return Err(RunError::NoModAvailable);
        }
        if let Some(entry) = self.shop_mut()?.card_offers.get_mut(slot) {
            *entry = None;
        }
        self.pay(price, events);
        events.push(RunEvent::CardOfferBought {
            slot,
            card: offer.card,
            card_mod: offer.card_mod.clone(),
            value: offer.value,
            price,
        });
        self.attach_mod(content, offer.card, &offer.card_mod, offer.value, events)
    }

    pub(super) fn reroll_card_offers(
        &mut self,
        content: &Content,
        events: &mut Vec<RunEvent>,
    ) -> Result<(), RunError> {
        self.shop_mut()?;
        let cost = self.shop_price(content, self.card_reroll_price(content));
        if !self.can_pay(content, cost) {
            return Err(RunError::NotEnoughCoins);
        }
        self.pay(cost, events);
        self.shop_mut()?.card_rerolls += 1;
        events.push(RunEvent::CardOffersRerolled { cost });
        self.draw_card_offers(content, events);
        Ok(())
    }

    /// The modifications the Kartenmaler offers for `card`: up to
    /// [`crate::content::ZinkenConfig::painter_choices`] different ones
    /// that fit it, with their Tresen value.
    fn draw_paints(&mut self, content: &Content, card: u32) -> Vec<(CardModId, i32)> {
        let pool: Vec<(CardModId, u32)> = self
            .zinken_pool(content)
            .into_iter()
            .filter(|(m, _)| {
                let value = content.card_mod(m).map_or(1, CardModDef::offer_value);
                self.takes(content, card, m, value)
            })
            .collect();
        let choices = content.run.shop.zinken.painter_choices;
        draw_weighted(&mut self.zinken_rng, pool, choices)
            .into_iter()
            .map(|m| {
                let value = content.card_mod(&m).map_or(1, CardModDef::offer_value);
                (m, value)
            })
            .collect()
    }

    /// The Kartenmaler takes `card`: draw his choice and wait for the
    /// player. `price` is paid first (0 at the Werkstatt).
    pub(super) fn start_painter(
        &mut self,
        content: &Content,
        card: u32,
        price: u32,
        encounter: bool,
        events: &mut Vec<RunEvent>,
    ) -> Result<(), RunError> {
        let deck_card = self
            .edition
            .get(card)
            .ok_or(crate::deck::DeckError::UnknownCard(card))?;
        if deck_card.kind == CardKind::Blank {
            return Err(crate::deck::DeckError::BlankCannotBeModified.into());
        }
        let options = self.draw_paints(content, card);
        if options.is_empty() {
            // The reason from the rules if they give one (a cursed card).
            if let Some((card_mod, _)) = self.zinken_pool(content).first() {
                let value = content
                    .card_mod(card_mod)
                    .map_or(1, CardModDef::offer_value);
                self.edition
                    .clone()
                    .add_mod(content, card, card_mod, value)?;
            }
            return Err(RunError::NoModAvailable);
        }
        self.pay(price, events);
        events.push(RunEvent::PainterHired {
            card,
            options: options.clone(),
            price,
        });
        self.painter = Some(PainterChoice {
            card,
            options,
            encounter,
        });
        Ok(())
    }

    pub(super) fn hire_painter(
        &mut self,
        content: &Content,
        card: u32,
        events: &mut Vec<RunEvent>,
    ) -> Result<(), RunError> {
        if self.shop_mut()?.painter_used {
            return Err(RunError::AlreadyUsed);
        }
        let price = self.painter_cost(content, card);
        if !self.can_pay(content, price) {
            return Err(RunError::NotEnoughCoins);
        }
        self.start_painter(content, card, price, false, events)?;
        self.shop_mut()?.painter_used = true;
        Ok(())
    }

    pub(super) fn choose_paint(
        &mut self,
        content: &Content,
        option: usize,
        events: &mut Vec<RunEvent>,
    ) -> Result<(), RunError> {
        let painter = self.painter.clone().ok_or(RunError::WrongPhase)?;
        let (card_mod, value) = painter
            .options
            .get(option)
            .cloned()
            .ok_or(RunError::InvalidSlot)?;
        let mut next = self.clone();
        next.painter = None;
        events.push(RunEvent::PainterChosen {
            card: painter.card,
            card_mod: card_mod.clone(),
            value,
        });
        next.attach_mod(content, painter.card, &card_mod, value, events)?;
        if painter.encounter && next.phase == RunPhase::Encounter {
            if let Some(encounter) = next.encounter.take() {
                events.push(RunEvent::EncounterResolved {
                    encounter: encounter.id,
                    choice: EncounterChoice::Painter,
                });
            }
            next.continue_trip(content, events)?;
        }
        *self = next;
        Ok(())
    }

    pub(super) fn cleanse_at_tresen(
        &mut self,
        content: &Content,
        card: u32,
        events: &mut Vec<RunEvent>,
    ) -> Result<(), RunError> {
        if self.shop_mut()?.cleansed {
            return Err(RunError::AlreadyUsed);
        }
        let price = self.shop_price(content, content.run.shop.zinken.cleanse_price);
        if !self.can_pay(content, price) {
            return Err(RunError::NotEnoughCoins);
        }
        let mut next = self.clone();
        let change = next.cleanse(content, card)?;
        self.edition = next.edition;
        self.pay(price, events);
        self.shop_mut()?.cleansed = true;
        events.push(RunEvent::CardCleansed { card, price });
        events.push(RunEvent::DeckChanged(change));
        Ok(())
    }

    /// Drop what the content no longer knows from the Auslage and the
    /// Kartenmaler's choice (part of [`Run::drop_unknown_content`]).
    pub(super) fn drop_unknown_zinken(&mut self, content: &Content) {
        if let Some(shop) = self.shop.as_mut() {
            for offer in &mut shop.card_offers {
                if offer
                    .as_ref()
                    .is_some_and(|o| content.card_mod(&o.card_mod).is_none())
                {
                    *offer = None;
                }
            }
        }
        if let Some(painter) = self.painter.as_mut() {
            painter
                .options
                .retain(|(m, _)| content.card_mod(m).is_some());
            if painter.options.is_empty() {
                self.painter = None;
            }
        }
    }
}

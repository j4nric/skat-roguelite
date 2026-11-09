//! The view of the Tresen: offers with prices and whether they can be
//! bought, the services, the Kartenmaler and the consumables owned.

use crate::cards::{PlayingCard, Suit};
use crate::content::{CardModId, ConsumableId, Content};
use crate::deck::{DealtCard, DeckCard};
use crate::game::Seat;

use super::{
    CardOfferView, ConsumableOffer, OwnedConsumable, PainterOptionView, PainterView, RelicOffer,
    Run, RunAction, ServiceView, ShopView, sell_price,
};

impl Run {
    pub(super) fn owned_consumable(
        &self,
        content: &Content,
        slot: usize,
        id: &ConsumableId,
    ) -> OwnedConsumable {
        let consumable = Run::consumable_info(content, id);
        let (usable_now, targets) = if consumable.needs_suit {
            let hand: Vec<DealtCard> = self
                .game
                .as_ref()
                .map(|a| a.game.hand(Seat::DECLARER).to_vec())
                .unwrap_or_default();
            let targets: Vec<u32> = hand
                .iter()
                .filter(|card| {
                    // Every other suit is as good as any: try one.
                    let own = card.face().map(|f| f.suit);
                    let other = Suit::ALL.into_iter().find(|&s| Some(s) != own);
                    self.accepts(
                        content,
                        RunAction::UseConsumable {
                            slot,
                            target: Some(card.id),
                            suit: other,
                        },
                    )
                })
                .map(|c| c.id)
                .collect();
            (!targets.is_empty(), targets)
        } else if consumable.needs_target {
            let targets: Vec<u32> = self
                .edition
                .cards()
                .iter()
                .map(|c| c.id)
                .filter(|&card| {
                    self.accepts(
                        content,
                        RunAction::UseConsumable {
                            slot,
                            target: Some(card),
                            suit: None,
                        },
                    )
                })
                .collect();
            (!targets.is_empty(), targets)
        } else {
            let action = RunAction::UseConsumable {
                slot,
                target: None,
                suit: None,
            };
            (self.accepts(content, action), Vec::new())
        };
        OwnedConsumable {
            slot,
            sell_price: sell_price(consumable.price),
            consumable,
            can_sell: self.accepts(content, RunAction::SellConsumable(slot)),
            usable_now,
            targets,
        }
    }

    /// How far the Tresen would sell on credit now: the limit minus the
    /// debt, 0 once the credit of this district is used up.
    pub(super) fn credit_left(&self, content: &Content) -> u32 {
        let limit = self.credit_limit(content);
        let open = self.shop.as_ref().is_some_and(|s| s.on_credit)
            || self.credit_section != Some(self.way_section());
        if open {
            limit.saturating_sub(self.debt)
        } else {
            0
        }
    }

    pub(super) fn shop_view(&self, content: &Content) -> Option<ShopView> {
        let shop = self.shop.as_ref()?;
        let slots_full = !shop.kiosk && !content.relic_slot_free(self.character, self.relics.len());
        Some(ShopView {
            relics: shop
                .relics
                .iter()
                .enumerate()
                .map(|(slot, id)| {
                    id.as_ref().map(|id| {
                        let relic = Run::relic_info(content, id);
                        let fence = self.is_fence(slot);
                        RelicOffer {
                            slot,
                            price: self.relic_offer_cost(content, slot, relic.price),
                            was: fence.then(|| self.shop_price(content, relic.price)),
                            fence,
                            relic,
                            can_buy: self.accepts(content, RunAction::BuyRelic(slot)),
                            replace: if slots_full {
                                (0..self.relics.len())
                                    .filter(|&replace| {
                                        self.accepts(
                                            content,
                                            RunAction::ReplaceRelic { slot, replace },
                                        )
                                    })
                                    .collect()
                            } else {
                                Vec::new()
                            },
                        }
                    })
                })
                .collect(),
            consumables: shop
                .consumables
                .iter()
                .enumerate()
                .map(|(slot, id)| {
                    id.as_ref().map(|id| {
                        let consumable = Run::consumable_info(content, id);
                        ConsumableOffer {
                            slot,
                            price: self.consumable_price(content, consumable.price),
                            consumable,
                            can_buy: self.accepts(content, RunAction::BuyConsumable(slot)),
                        }
                    })
                })
                .collect(),
            reroll_cost: self.reroll_cost(content),
            can_reroll: self.accepts(content, RunAction::Reroll),
            rerolls: shop.rerolls,
            reroll_used: shop.rerolls > 0,
            card_offers: shop
                .card_offers
                .iter()
                .enumerate()
                .map(|(slot, offer)| {
                    let offer = offer.as_ref()?;
                    let card = self.edition.get(offer.card)?.clone();
                    let (result, replaces, raises) =
                        self.mod_result(content, offer.card, &offer.card_mod, offer.value);
                    Some(CardOfferView {
                        slot,
                        result: result.unwrap_or_else(|| card.clone()),
                        card,
                        card_mod: Run::card_mod_info(content, &offer.card_mod, Some(offer.value)),
                        price: self.card_offer_cost(content, offer),
                        can_buy: self.accepts(content, RunAction::BuyCardOffer(slot)),
                        replaces,
                        raises,
                        bargain: offer.bargain.is_some(),
                        was: offer.bargain.map(|full| {
                            let full = super::super::zinken::CardOffer {
                                price: full,
                                ..offer.clone()
                            };
                            self.card_offer_cost(content, &full)
                        }),
                    })
                })
                .collect(),
            painter: self.service_view(
                content,
                shop.painter_used,
                content.run.shop.zinken.painter_price,
                RunAction::HirePainter,
            ),
            cleanse: self.service_view(
                content,
                shop.cleansed,
                content.run.shop.zinken.cleanse_price,
                RunAction::CleanseCard,
            ),
            kiosk: shop.kiosk,
            kiosk_discount: if shop.kiosk {
                content.run.route.kiosk.discount
            } else {
                0
            },
        })
    }

    /// A card service of the Tresen: its price and the cards it takes now.
    fn service_view(
        &self,
        content: &Content,
        used: bool,
        price: u32,
        action: fn(u32) -> RunAction,
    ) -> ServiceView {
        let targets: Vec<u32> = if used {
            Vec::new()
        } else {
            self.edition
                .cards()
                .iter()
                .map(|c| c.id)
                .filter(|&card| self.accepts(content, action(card)))
                .collect()
        };
        let painter = action(0) == RunAction::HirePainter(0);
        let target_prices = targets
            .iter()
            .map(|&card| {
                if painter {
                    self.painter_cost(content, card)
                } else {
                    self.shop_price(content, price)
                }
            })
            .collect();
        ServiceView {
            price: self.shop_price(content, price),
            used,
            can_use: !targets.is_empty(),
            targets,
            target_prices,
        }
    }

    /// What `card_mod` with `value` would make of `card`: the card after,
    /// the modification it replaces and whether it raises one. Tried on a
    /// copy of the edition; `None` if the rules refuse it.
    pub(in crate::run) fn mod_result(
        &self,
        content: &Content,
        card: u32,
        card_mod: &CardModId,
        value: i32,
    ) -> (Option<DeckCard>, Option<CardModId>, bool) {
        let before = self
            .edition
            .get(card)
            .map(|c| c.mods.clone())
            .unwrap_or_default();
        let mut edition = self.edition.probe_card(card);
        match edition.add_mod(content, card, card_mod, value) {
            Ok(change) => {
                let replaced = match change {
                    crate::deck::DeckChange::Modified { replaced, .. } => replaced,
                    _ => None,
                };
                let raises = replaced.is_none() && before.contains(card_mod);
                (edition.get(card).cloned(), replaced, raises)
            }
            Err(_) => (None, None, false),
        }
    }

    pub(super) fn painter_view(&self, content: &Content) -> Option<PainterView> {
        let painter = self.painter.as_ref()?;
        let card = self.edition.get(painter.card)?.clone();
        let options = painter
            .options
            .iter()
            .enumerate()
            .map(|(index, (card_mod, value))| {
                let (result, replaces, raises) =
                    self.mod_result(content, painter.card, card_mod, *value);
                PainterOptionView {
                    index,
                    card_mod: Run::card_mod_info(content, card_mod, Some(*value)),
                    result: result.unwrap_or_else(|| card.clone()),
                    replaces,
                    raises,
                }
            })
            .collect();
        Some(PainterView {
            card,
            options,
            encounter: painter.encounter,
        })
    }
}

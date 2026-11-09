//! The Tresen (DESIGN 8): offers, prices, paying (also on credit, "Deckel
//! anschreiben"), buying, selling and rerolling relics and consumables.

use std::collections::BTreeSet;

use crate::content::{
    ConsumableId, Content, Effect, GameFocus, Hook, RelicId, RelicKind, Stage, in_pool,
};
use crate::deck::{DeckChange, DeckError};

use super::{Run, RunError, RunEvent, RunPhase, Shop, draw_weighted, sell_price, trade};

impl Run {
    /// Relics that may be offered or drawn, with their draw weight: for the
    /// character, unlocked, neither owned nor on the table for the evening;
    /// only house rules if `house_rules_only`. In id order. The weight is
    /// the rarity weight of `run.ron` times the character's weight for the
    /// relic's Spielart ([`Run::focus_weight`]).
    pub(super) fn relic_pool(
        &self,
        content: &Content,
        house_rules_only: bool,
    ) -> Vec<(RelicId, u32)> {
        let weights = &content.run.shop.rarity_weights;
        content
            .relics
            .values()
            .filter(|r| in_pool(&r.characters, self.character))
            // The relics of the night (ADR 0020): in the long night always;
            // in a normal run only one an unlock opened.
            .filter(|r| {
                if r.night {
                    self.night.is_some()
                        || (content.relic_unlockable(&r.id) && !self.locked.contains(&r.id))
                } else {
                    !self.locked.contains(&r.id)
                }
            })
            .filter(|r| !self.relics.contains(&r.id) && !self.evening_relics.contains(&r.id))
            .filter(|r| !house_rules_only || r.kind == RelicKind::HouseRule)
            .map(|r| {
                let weight = weights.percent_weight(r.rarity, self.focus_weight(content, r.focus));
                (r.id.clone(), weight)
            })
            .collect()
    }

    /// The character's Tresen weight for items made for this Spielart, in
    /// percent (ADR 0012: the Nullspieler meets Null relics more often).
    pub(super) fn focus_weight(&self, content: &Content, focus: GameFocus) -> u32 {
        content
            .character(self.character)
            .map_or(100, |c| c.focus_weights.percent(focus))
    }

    /// Draw a deterministic offer from the content pool. Relics the player
    /// owns, locked relics and items for other characters are not offered;
    /// no item twice in one offer. Rarer items are drawn less often
    /// (weights in `run.ron`). One relic may be "Hehlerware"
    /// ([`Run::draw_fence`]).
    pub(super) fn draw_offer(&mut self, content: &Content, rerolls: u32) -> Shop {
        let weights = &content.run.shop.rarity_weights;
        let relic_pool = self.relic_pool(content, false);
        let consumable_pool: Vec<(ConsumableId, u32)> = content
            .consumables
            .values()
            .filter(|c| in_pool(&c.characters, self.character))
            .map(|c| (c.id.clone(), weights.weight(c.rarity)))
            .collect();
        let relics = draw_weighted(
            &mut self.shop_rng,
            relic_pool,
            content.run.shop.relic_offers,
        );
        let consumables = draw_weighted(
            &mut self.shop_rng,
            consumable_pool,
            content.run.shop.consumable_offers,
        );
        let fence = self.draw_fence(content, relics.len());
        Shop {
            fence,
            relics: relics.into_iter().map(Some).collect(),
            consumables: consumables.into_iter().map(Some).collect(),
            rerolls,
            on_credit: false,
            card_offers: Vec::new(),
            painter_used: false,
            cleansed: false,
            kiosk: false,
        }
    }

    /// The owned relics and the evening's house rules, in that order.
    pub(super) fn held_relics(&self) -> impl Iterator<Item = &RelicId> {
        self.relics.iter().chain(&self.evening_relics)
    }

    /// [`Run::held_relics`] the content knows.
    pub(super) fn held_relic_defs<'c>(
        &self,
        content: &'c Content,
    ) -> impl Iterator<Item = &'c crate::content::RelicDef> {
        self.held_relics().filter_map(|r| content.relic(r))
    }

    /// The passive effects of [`Run::held_relic_defs`].
    pub(super) fn held_passives<'c>(
        &self,
        content: &'c Content,
    ) -> impl Iterator<Item = &'c Effect> {
        self.held_relic_defs(content)
            .flat_map(|def| def.passive.iter())
    }

    /// The passive effects of the owned relics and the evening's house
    /// rules.
    pub(super) fn relic_passives<'c>(&self, content: &'c Content) -> Vec<&'c Effect> {
        self.held_passives(content).collect()
    }

    /// The credit limit at the Tresen ("Deckel anschreiben"): the largest
    /// granted, 0 without.
    pub(super) fn credit_limit(&self, content: &Content) -> u32 {
        self.relic_passives(content)
            .into_iter()
            .map(|e| match e {
                Effect::Credit(n) => *n,
                _ => 0,
            })
            .max()
            .unwrap_or(0)
    }

    /// The district the Tresen and the way belong to: where the way
    /// leads, else the current pub's.
    pub(super) fn way_section(&self) -> u32 {
        self.trip
            .as_ref()
            .map_or(self.series.section, |t| t.section)
    }

    /// Whether the player can pay `price` at the Tresen now: from his
    /// Groschen, or on credit ("Deckel anschreiben", once per district,
    /// down to the limit).
    pub(super) fn can_pay(&self, content: &Content, price: u32) -> bool {
        if price <= self.coins {
            return true;
        }
        let limit = self.credit_limit(content);
        let open = self.shop.as_ref().is_some_and(|s| s.on_credit)
            || self.credit_section != Some(self.way_section());
        open && limit > 0 && self.debt.saturating_add(price - self.coins) <= limit
    }

    /// Pay `price` at the Tresen, on credit for what the Groschen do not
    /// cover. Check [`Run::can_pay`] first.
    pub(super) fn pay(&mut self, price: u32, events: &mut Vec<RunEvent>) {
        if price <= self.coins {
            self.coins -= price;
            return;
        }
        let short = price - self.coins;
        self.coins = 0;
        self.debt = self.debt.saturating_add(short);
        self.credit_section = Some(self.way_section());
        if let Some(shop) = self.shop.as_mut() {
            shop.on_credit = true;
        }
        events.push(RunEvent::DebtChanged { debt: self.debt });
    }

    /// The player gave `relic` up (sold, replaced, swapped): its pots are
    /// paid out ([`Run::pay_out_stash`]) and the `OnRelicSold` hooks of the
    /// relics still held fire ("Der Trödler").
    pub(super) fn relic_given_up(
        &mut self,
        content: &Content,
        relic: &RelicId,
        events: &mut Vec<RunEvent>,
    ) {
        self.pay_out_stash(content, relic, events);
        self.fire(content, Stage::OnRelicSold, None, None, events);
    }

    /// `relic` has just left the player: every pot it stashes into
    /// ([`Effect::StashCoins`]) that no relic still held stashes into is
    /// paid out as it is, without a bonus, and emptied
    /// ([`RunEvent::StashPaidOut`]).
    pub(super) fn pay_out_stash(
        &mut self,
        content: &Content,
        relic: &RelicId,
        events: &mut Vec<RunEvent>,
    ) {
        let stashes = |id: &RelicId| -> Vec<String> {
            content.relic(id).map_or_else(Vec::new, |def| {
                def.hooks
                    .iter()
                    .flat_map(Hook::then)
                    .chain(&def.passive)
                    .filter_map(|e| match e {
                        Effect::StashCoins { counter, .. } => Some(counter.clone()),
                        _ => None,
                    })
                    .collect()
            })
        };
        let still_held: BTreeSet<String> = self.held_relics().flat_map(stashes).collect();
        let mut amount: u32 = 0;
        for counter in stashes(relic) {
            if still_held.contains(&counter) {
                continue;
            }
            amount = amount.saturating_add(self.counters.remove(&counter).unwrap_or(0));
        }
        if amount > 0 {
            self.coins = self.coins.saturating_add(amount);
            events.push(RunEvent::StashPaidOut {
                relic: relic.clone(),
                amount,
                coins: self.coins,
            });
            self.settle_debt(events);
        }
    }

    /// Income pays the debt first ("Deckel anschreiben").
    pub(super) fn settle_debt(&mut self, events: &mut Vec<RunEvent>) {
        let paid = self.debt.min(self.coins);
        if paid > 0 {
            self.debt -= paid;
            self.coins -= paid;
            events.push(RunEvent::DebtChanged { debt: self.debt });
        }
    }

    /// Percent off at the Tresen from the owned relics ("Der Wirt"), at
    /// most 90.
    fn shop_discount(&self, content: &Content) -> u32 {
        let total: u32 = self
            .held_passives(content)
            .map(|e| match e {
                Effect::ShopDiscount(p) => *p,
                _ => 0,
            })
            .sum();
        total.min(90)
    }

    /// What an item with this price costs the player now: the price minus
    /// the Tresen discount, rounded to the nearest Groschen, at least 1
    /// for a priced item. Selling always pays half the full price.
    #[must_use]
    pub fn shop_price(&self, content: &Content, price: u32) -> u32 {
        if price == 0 {
            return 0;
        }
        // "Nachtzuschlag" (long night, ADR 0020): on top of everything.
        let surcharge = self.night_surcharge(content);
        let discount = self.shop_discount(content);
        if discount == 0 {
            return price.saturating_add(surcharge);
        }
        let paid = (u64::from(price) * u64::from(100 - discount) + 50) / 100;
        u32::try_from(paid)
            .unwrap_or(price)
            .max(1)
            .saturating_add(surcharge)
    }

    /// What a Zinke (a card modification bought for Groschen on a card of
    /// the player's choice) with this price costs on the card with this
    /// instance id: the character's [`crate::content::ZinkenDiscount`] for
    /// matching cards ("Bruderherz": half on Rot), rounded to the nearest
    /// Groschen, at least 1 for a priced Zinke. The Tresen discount is
    /// not part of it ([`Run::shop_price`]). Every offer of a Zinke on a
    /// known card must charge this price.
    #[must_use]
    pub fn zinken_price(&self, content: &Content, card: u32, price: u32) -> u32 {
        let Some(discount) = content
            .character(self.character)
            .and_then(|c| c.rule.zinken_discount)
        else {
            return price;
        };
        let matches = self
            .edition
            .get(card)
            .and_then(|c| c.kind.face())
            .is_some_and(|c| discount.cards.matches(c));
        if price == 0 || !matches {
            return price;
        }
        let percent = 100 - discount.percent.min(100);
        let paid = (u64::from(price) * u64::from(percent) + 50) / 100;
        u32::try_from(paid).unwrap_or(price).max(1)
    }

    /// Cleanse a card of the player's edition ("Frisches Deck",
    /// Werkstatt): every modification falls off. A character with
    /// [`crate::content::CharacterRule::keep_curses`] ("Bruderherz")
    /// cannot cleanse a card that carries a curse ([`RunError::CurseStays`]).
    pub(super) fn cleanse(&mut self, content: &Content, card: u32) -> Result<DeckChange, RunError> {
        self.check_cleanse(content, card)?;
        Ok(self.edition.cleanse(card)?)
    }

    /// Whether [`Run::cleanse`] would cleanse `card`, and if not, why.
    pub(super) fn check_cleanse(&self, content: &Content, card: u32) -> Result<(), RunError> {
        let keep = content
            .character(self.character)
            .is_some_and(|c| c.rule.keep_curses);
        if keep && self.card_is_cursed(content, card) {
            return Err(RunError::CurseStays);
        }
        let deck_card = self.edition.get(card).ok_or(DeckError::UnknownCard(card))?;
        if deck_card.mods.is_empty() {
            return Err(DeckError::NothingToCleanse(card).into());
        }
        Ok(())
    }

    /// Relic slots the player leaves free ("Leerer Stuhl"): the slots of
    /// his character less the relics he holds (house rules of the evening
    /// do not count); 0 without a slot limit.
    #[must_use]
    pub fn free_relic_slots(&self, content: &Content) -> u32 {
        content.relic_slots(self.character).map_or(0, |slots| {
            slots.saturating_sub(u32::try_from(self.relics.len()).unwrap_or(u32::MAX))
        })
    }

    pub(super) fn shop_mut(&mut self) -> Result<&mut Shop, RunError> {
        self.shop_ref()?;
        self.shop.as_mut().ok_or(RunError::WrongPhase)
    }

    /// The open shop, read only.
    pub(super) fn shop_ref(&self) -> Result<&Shop, RunError> {
        if self.phase != RunPhase::Shop {
            return Err(RunError::WrongPhase);
        }
        self.shop.as_ref().ok_or(RunError::WrongPhase)
    }

    /// The open shop if it is the Tresen (not a Kiosk).
    pub(super) fn tresen_mut(&mut self) -> Result<&mut Shop, RunError> {
        self.tresen_ref()?;
        self.shop.as_mut().ok_or(RunError::WrongPhase)
    }

    /// [`Run::tresen_mut`], read only.
    pub(super) fn tresen_ref(&self) -> Result<&Shop, RunError> {
        let shop = self.shop_ref()?;
        if shop.kiosk {
            return Err(RunError::WrongPhase);
        }
        Ok(shop)
    }

    /// Whether the open shop is a Kiosk.
    #[must_use]
    pub fn at_kiosk(&self) -> bool {
        self.phase == RunPhase::Shop && self.shop.as_ref().is_some_and(|s| s.kiosk)
    }

    /// What a consumable with this price costs in the open shop: the
    /// Tresen price ([`Run::shop_price`]), at a Kiosk the Kiosk's discount
    /// less, at least 1.
    #[must_use]
    pub fn consumable_price(&self, content: &Content, price: u32) -> u32 {
        let paid = self.shop_price(content, price);
        if self.shop.as_ref().is_some_and(|s| s.kiosk) {
            paid.saturating_sub(content.run.route.kiosk.discount).max(1)
        } else {
            paid
        }
    }

    pub(super) fn buy_relic(
        &mut self,
        content: &Content,
        slot: usize,
        events: &mut Vec<RunEvent>,
    ) -> Result<(), RunError> {
        let coins = self.coins;
        let owned = self.relics.len();
        let shop = self.shop_mut()?;
        let id = shop
            .relics
            .get(slot)
            .cloned()
            .flatten()
            .ok_or(RunError::InvalidSlot)?;
        let def = content
            .relic(&id)
            .ok_or_else(|| RunError::UnknownContent(id.to_string()))?;
        let price = self.relic_offer_cost(content, slot, def.price);
        if price > coins && !self.can_pay(content, price) {
            return Err(RunError::NotEnoughCoins);
        }
        if !content.relic_slot_free(self.character, owned) {
            return Err(RunError::NoFreeSlot);
        }
        let fence = self.is_fence(slot);
        if let Some(entry) = self.shop_mut()?.relics.get_mut(slot) {
            *entry = None;
        }
        self.pay(price, events);
        self.relics.push(id.clone());
        events.push(RunEvent::RelicBought { relic: id, price });
        if fence {
            self.trade_curse(content, &[], trade::CurseCause::Fence, events)?;
        }
        Ok(())
    }

    pub(super) fn buy_consumable(
        &mut self,
        content: &Content,
        slot: usize,
        events: &mut Vec<RunEvent>,
    ) -> Result<(), RunError> {
        let coins = self.coins;
        let owned = self.consumables.len();
        let shop = self.shop_mut()?;
        let id = shop
            .consumables
            .get(slot)
            .cloned()
            .flatten()
            .ok_or(RunError::InvalidSlot)?;
        let def = content
            .consumable(&id)
            .ok_or_else(|| RunError::UnknownContent(id.to_string()))?;
        let price = self.consumable_price(content, def.price);
        let kiosk = self.at_kiosk();
        if price > coins && (kiosk || !self.can_pay(content, price)) {
            return Err(RunError::NotEnoughCoins);
        }
        if owned >= usize::try_from(content.run.consumable_slots).unwrap_or(0) {
            return Err(RunError::NoFreeSlot);
        }
        let shop = self.shop_mut()?;
        if kiosk {
            // One Schnaps per Kiosk: the other offers go.
            shop.consumables.iter_mut().for_each(|c| *c = None);
        } else if let Some(entry) = shop.consumables.get_mut(slot) {
            *entry = None;
        }
        self.pay(price, events);
        self.consumables.push(id.clone());
        events.push(RunEvent::ConsumableBought {
            consumable: id,
            price,
        });
        Ok(())
    }

    /// [`RunAction::ReplaceRelic`](super::RunAction::ReplaceRelic): sell `replace`, then buy `slot`, on a
    /// copy, so a failed purchase leaves the run as it was.
    pub(super) fn replace_relic(
        &mut self,
        content: &Content,
        slot: usize,
        replace: usize,
        events: &mut Vec<RunEvent>,
    ) -> Result<(), RunError> {
        self.shop_mut()?
            .relics
            .get(slot)
            .cloned()
            .flatten()
            .ok_or(RunError::InvalidSlot)?;
        // On an error the caller drops this copy of the run ([`Run::dispatch`]).
        self.sell_relic(content, replace, events)?;
        self.buy_relic(content, slot, events)
    }

    pub(super) fn sell_relic(
        &mut self,
        content: &Content,
        index: usize,
        events: &mut Vec<RunEvent>,
    ) -> Result<(), RunError> {
        self.tresen_mut()?;
        let id = self
            .relics
            .get(index)
            .cloned()
            .ok_or(RunError::InvalidSlot)?;
        let price = content.relic(&id).map_or(0, |d| sell_price(d.price));
        self.relics.remove(index);
        self.relic_given_up(content, &id, events);
        self.coins = self.coins.saturating_add(price);
        events.push(RunEvent::RelicSold { relic: id, price });
        self.settle_debt(events);
        Ok(())
    }

    pub(super) fn sell_consumable(
        &mut self,
        content: &Content,
        index: usize,
        events: &mut Vec<RunEvent>,
    ) -> Result<(), RunError> {
        self.shop_mut()?;
        let id = self
            .consumables
            .get(index)
            .cloned()
            .ok_or(RunError::InvalidSlot)?;
        let price = content.consumable(&id).map_or(0, |d| sell_price(d.price));
        self.consumables.remove(index);
        self.coins = self.coins.saturating_add(price);
        events.push(RunEvent::ConsumableSold {
            consumable: id,
            price,
        });
        self.settle_debt(events);
        Ok(())
    }

    /// What "Neu auslosen" costs now: [`crate::content::Economy::reroll_cost`]
    /// after the Tresen discount ("Der Wirt"), nothing with "Der
    /// Stammkunde" ([`Effect::FreeReroll`]).
    #[must_use]
    pub fn reroll_cost(&self, content: &Content) -> u32 {
        let free = self
            .held_passives(content)
            .any(|e| *e == Effect::FreeReroll);
        if free {
            0
        } else {
            self.shop_price(content, content.run.economy.reroll_cost)
        }
    }

    /// "Neu auslosen" (DESIGN 8.6, since 05.11.2026): relics, Schnäpse and
    /// the Auslage drawn again, once per Tresen visit.
    pub(super) fn reroll(
        &mut self,
        content: &Content,
        events: &mut Vec<RunEvent>,
    ) -> Result<(), RunError> {
        let rerolls = self.tresen_mut()?.rerolls;
        if rerolls > 0 {
            return Err(RunError::AlreadyUsed);
        }
        let cost = self.reroll_cost(content);
        if !self.can_pay(content, cost) {
            return Err(RunError::NotEnoughCoins);
        }
        self.pay(cost, events);
        let old = self.shop.take();
        let mut shop = self.draw_offer(content, rerolls + 1);
        if let Some(old) = old {
            shop.on_credit = old.on_credit;
            shop.painter_used = old.painter_used;
            shop.cleansed = old.cleansed;
        }
        events.push(RunEvent::ShopRerolled { cost });
        events.push(RunEvent::ShopOpened {
            relics: shop.relics.clone(),
            consumables: shop.consumables.clone(),
            fence: shop.fence,
        });
        self.shop = Some(shop);
        self.draw_card_offers(content, events);
        Ok(())
    }

    pub(super) fn leave_shop(
        &mut self,
        content: &Content,
        events: &mut Vec<RunEvent>,
    ) -> Result<(), RunError> {
        self.shop_mut()?;
        events.push(RunEvent::ShopLeft);
        self.shop = None;
        self.continue_trip(content, events)
    }
}

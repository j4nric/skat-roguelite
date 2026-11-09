//! Curses as trade (decided by the user on 03.11.2026, DESIGN 8.6).
//!
//! A curse is the price of a bargain. Three offers pay with one, each a
//! permanent random curse on a random card of the player's edition that
//! carries none (a curse takes the place of the card's normal
//! modification, a special card keeps its special, DESIGN 7.2). A trade
//! curse pays none of the Groschen a curse pays elsewhere when it lands:
//!
//! - **Schnäppchen** ([`super::zinken::CardOffer::bargain`]): an offer of
//!   the Auslage at half price (rounded down, at least 1); the curse lands
//!   on another card, if possible on none the Auslage offers.
//! - **Hehlerware** ([`super::Shop::fence`]): one relic of the Tresen at
//!   half price; buying it curses a card.
//! - **Fusel** ([`crate::content::ConsumableAction::NextGameCursed`]): the
//!   next declared game counts double, and a card is cursed at once. What
//!   the curse does when the cards are dealt (Augen, Durchsichtig) counts
//!   from the next game on; its hooks (Fleckig) and the Zinke it replaced
//!   count from now on.
//!
//! The relic "Pechvogel" turns the other way: +1 Grundwert per cursed card
//! ([`crate::content::Effect::AddBaseValuePerCursedCard`]).
//!
//! The trade has its own generator (`trade_rng`): whether an offer is a
//! trade, which card and which curse. The draws of the Tresen and the
//! Auslage stay as they were without it. An offer is a trade only while a
//! card can take the curse; should none be left when it is bought, the
//! curse falls away.

use serde::{Deserialize, Serialize};

use super::{Run, RunError, RunEvent, draw_weighted, mod_value};
use crate::content::{CardModId, Content, TradeConfig};
use crate::deck::CardKind;

/// What a trade curse was the price of.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CurseCause {
    /// A "Schnäppchen" of the Auslage.
    Bargain,
    /// A relic of the "Hehlerware".
    Fence,
    /// The Schnaps "Fusel".
    Schnapps,
}

impl Run {
    /// The cards a trade may curse now, `except` left out: no blank, no
    /// card that carries a curse already, and only cards that take at least
    /// one curse of the character's pool ([`Run::mod_pool`]). In id order.
    #[must_use]
    pub fn trade_curse_targets(&self, content: &Content, except: &[u32]) -> Vec<u32> {
        let curses = self.mod_candidates(content, true);
        self.edition
            .cards()
            .iter()
            .filter(|c| c.kind != CardKind::Blank && !except.contains(&c.id))
            .filter(|c| !self.card_is_cursed(content, c.id))
            .filter(|c| {
                curses
                    .iter()
                    .any(|(m, _)| self.may_attach(content, c.id, m))
            })
            .map(|c| c.id)
            .collect()
    }

    /// Put a trade curse on a random card of
    /// [`Run::trade_curse_targets`]: the card drawn evenly, then the curse
    /// by its weight among those that fit it, with its default value. Says
    /// so ([`RunEvent::TradeCurse`]) before the curse lands. It pays no
    /// Groschen: the curse is the price of the deal (decided on
    /// 03.11.2026). Returns the card, `None` if no card can take a curse
    /// (it falls away).
    pub(super) fn trade_curse(
        &mut self,
        content: &Content,
        except: &[u32],
        cause: CurseCause,
        events: &mut Vec<RunEvent>,
    ) -> Result<Option<(u32, CardModId)>, RunError> {
        let targets = self.trade_curse_targets(content, except);
        if targets.is_empty() {
            return Ok(None);
        }
        let pick = self
            .trade_rng
            .gen_range(0..u64::try_from(targets.len()).unwrap_or(1));
        let card = targets[usize::try_from(pick).unwrap_or(0)];
        let pool: Vec<(CardModId, u32)> = self
            .mod_candidates(content, true)
            .into_iter()
            .filter(|(m, _)| self.may_attach(content, card, m))
            .collect();
        let Some(curse) = draw_weighted(&mut self.trade_rng, pool, 1).pop() else {
            return Ok(None);
        };
        events.push(RunEvent::TradeCurse {
            card,
            curse: curse.clone(),
            cause,
        });
        let value = mod_value(content, &curse, None);
        self.attach_mod_paid(content, card, &curse, value, false, events)?;
        Ok(Some((card, curse)))
    }

    /// Whether a drawn relic offer has a "Hehlerware", and in which slot:
    /// with [`TradeConfig::fence_percent`], among `relics` offers, while a
    /// card can take a curse.
    pub(super) fn draw_fence(&mut self, content: &Content, relics: usize) -> Option<usize> {
        let percent = content.run.shop.trade.fence_percent;
        if percent == 0 || relics == 0 {
            return None;
        }
        if self.trade_rng.gen_range(0..100) >= u64::from(percent) {
            return None;
        }
        if self.trade_curse_targets(content, &[]).is_empty() {
            return None;
        }
        let slot = self
            .trade_rng
            .gen_range(0..u64::try_from(relics).unwrap_or(1));
        usize::try_from(slot).ok()
    }

    /// Whether an offer of the Auslage on `card` is a "Schnäppchen": with
    /// [`TradeConfig::bargain_percent`], while another card can take a
    /// curse.
    pub(super) fn draw_bargain(&mut self, content: &Content, card: u32) -> bool {
        let percent = content.run.shop.trade.bargain_percent;
        if percent == 0 || self.trade_rng.gen_range(0..100) >= u64::from(percent) {
            return false;
        }
        !self.trade_curse_targets(content, &[card]).is_empty()
    }

    /// Whether the relic offer in `slot` of the open Tresen is
    /// "Hehlerware".
    #[must_use]
    pub fn is_fence(&self, slot: usize) -> bool {
        self.shop
            .as_ref()
            .is_some_and(|s| !s.kiosk && s.fence == Some(slot))
    }

    /// What the relic offer in `slot` with this `price` costs now: half
    /// for "Hehlerware" ([`TradeConfig::half`]), then the Tresen discount
    /// ([`Run::shop_price`]).
    #[must_use]
    pub fn relic_offer_cost(&self, content: &Content, slot: usize, price: u32) -> u32 {
        let price = if self.is_fence(slot) {
            TradeConfig::half(price)
        } else {
            price
        };
        self.shop_price(content, price)
    }

    /// The bargain's curse after an offer of the Auslage on `card` was
    /// bought: on another card, if possible on none the Auslage still
    /// offers (so that it voids no offer).
    pub(super) fn bargain_curse(
        &mut self,
        content: &Content,
        card: u32,
        events: &mut Vec<RunEvent>,
    ) -> Result<(), RunError> {
        let mut except: Vec<u32> = self
            .shop
            .as_ref()
            .map(|s| s.card_offers.iter().flatten().map(|o| o.card).collect())
            .unwrap_or_default();
        except.push(card);
        if self.trade_curse_targets(content, &except).is_empty() {
            except = vec![card];
        }
        self.trade_curse(content, &except, CurseCause::Bargain, events)?;
        Ok(())
    }
}

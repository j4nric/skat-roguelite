//! Save compatibility: everything a loaded run needs to read a save of an
//! older version. [`Run::drop_unknown_content`] maps renamed, refunded and
//! retired content, and the `default_*_rng` functions give the random
//! streams that saves from before them lack.

use crate::content::{BossId, ConsumableId, Content, EncounterEffect, RelicId};
use crate::rng::Rng;

use super::Run;

/// Consumables renamed by the item review of 02.11.2026: old id, new id.
/// A loaded run reads the old ids as the new ones
/// ([`Run::drop_unknown_content`]).
pub const RENAMED_CONSUMABLES: [(&str, &str); 1] = [("chalk", "blue_curacao")];

/// Consumables removed with their price paid back to a loaded run that
/// still holds them ([`Run::drop_unknown_content`]): "Konterbier" gave a
/// life until ADR 0019 (no lives).
pub const REFUNDED_CONSUMABLES: [(&str, u32); 1] = [("counter_beer", 4)];

/// Bosses removed by ADR 0017 and the boss a loaded run meets instead
/// ([`Run::drop_unknown_content`]): a save that drew one keeps its place
/// in the tour and meets the stand-in. None of the stand-ins forces a hand
/// game, so a save waiting at "Skat oder Hand" stays playable as it was.
pub const RETIRED_BOSSES: [(&str, &str); 3] = [
    ("kontra_hans", "twins"),
    ("revolutionary", "twins"),
    ("kibitz_killer", "black_peter"),
];

impl Run {
    /// Drop what the content no longer knows from a loaded run (relics and
    /// consumables removed by the item review of 02.11.2026, DESIGN 18.3;
    /// renamed consumables, [`RENAMED_CONSUMABLES`], are renamed first):
    /// owned and evening relics, consumables, Tresen offers, locked
    /// relics, consumables waiting for the next game and encounter options
    /// that name them. Returns the ids dropped, in order, for a note.
    /// Bosses removed by ADR 0017 are replaced by their stand-in
    /// ([`RETIRED_BOSSES`]; a game in progress keeps its deal). A held
    /// consumable of [`REFUNDED_CONSUMABLES`] the content no longer knows
    /// pays its price back in Groschen. Nothing else changes; a run
    /// without such ids stays as it is.
    pub fn drop_unknown_content(&mut self, content: &Content) -> Vec<String> {
        // Renamed consumables keep their place.
        let rename = |id: &mut ConsumableId| {
            if let Some((_, new)) = RENAMED_CONSUMABLES
                .iter()
                .find(|(old, _)| *old == id.as_str())
            {
                *id = ConsumableId::new(new);
            }
        };
        self.consumables.iter_mut().for_each(rename);
        self.pending_next_game
            .iter_mut()
            .for_each(|(id, _)| rename(id));
        if let Some(shop) = self.shop.as_mut() {
            shop.consumables.iter_mut().flatten().for_each(rename);
        }
        if let Some(encounter) = self.encounter.as_mut() {
            for option in &mut encounter.options {
                for effect in &mut option.effects {
                    if let EncounterEffect::Schnapps(Some(id)) = effect {
                        rename(id);
                    }
                }
            }
        }
        // Removed bosses (ADR 0017) give way to their stand-in.
        let retire = |id: &mut BossId| {
            if let Some((_, new)) = RETIRED_BOSSES.iter().find(|(old, _)| *old == id.as_str()) {
                *id = BossId::new(new);
            }
        };
        self.section_boss.iter_mut().for_each(retire);
        self.series.boss.iter_mut().for_each(retire);
        let mut next = self.game.as_mut();
        while let Some(active) = next {
            active.boss_rule.iter_mut().for_each(retire);
            next = active.previous.as_deref_mut();
        }
        let mut dropped: Vec<String> = Vec::new();
        let keep_relic = |id: &RelicId, dropped: &mut Vec<String>| {
            let known = content.relic(id).is_some();
            if !known {
                dropped.push(id.to_string());
            }
            known
        };
        self.relics.retain(|r| keep_relic(r, &mut dropped));
        self.evening_relics.retain(|r| keep_relic(r, &mut dropped));
        self.locked.retain(|r| content.relic(r).is_some());
        let keep_item = |id: &ConsumableId, dropped: &mut Vec<String>| {
            let known = content.consumable(id).is_some();
            if !known {
                dropped.push(id.to_string());
            }
            known
        };
        for id in &self.consumables {
            if content.consumable(id).is_some() {
                continue;
            }
            if let Some((_, price)) = REFUNDED_CONSUMABLES
                .iter()
                .find(|(old, _)| *old == id.as_str())
            {
                self.coins = self.coins.saturating_add(*price);
            }
        }
        self.consumables.retain(|c| keep_item(c, &mut dropped));
        self.pending_next_game
            .retain(|(c, _)| keep_item(c, &mut dropped));
        if let Some(shop) = self.shop.as_mut() {
            for offer in &mut shop.relics {
                if offer.as_ref().is_some_and(|r| content.relic(r).is_none()) {
                    *offer = None;
                }
            }
            for offer in &mut shop.consumables {
                if offer
                    .as_ref()
                    .is_some_and(|c| content.consumable(c).is_none())
                {
                    *offer = None;
                }
            }
        }
        if let Some(id) = self.schnapps_reward.as_mut() {
            rename(id);
        }
        // A pending Schnaps the content no longer knows: nothing to choose.
        if self
            .schnapps_reward
            .as_ref()
            .is_some_and(|c| !keep_item(c, &mut dropped))
        {
            self.schnapps_reward = None;
        }
        self.drop_unknown_zinken(content);
        self.drop_unknown_card_reward(content);
        self.drop_unknown_boss_reward(content);
        self.renew_encounter(content);
        dropped
    }

    /// Encounter generator of saves from before encounters existed:
    /// derived from nothing but a constant, so loading stays
    /// deterministic.
    pub(super) fn default_encounter_rng() -> Rng {
        Rng::from_seed(0x5354_414D_4D54_4953)
    }

    /// Route generator of saves from before the Kneipentour.
    pub(super) fn default_tour_rng() -> Rng {
        Rng::from_seed(0x4B4E_4549_5045_4E54)
    }

    /// Trade generator of saves from before curses as trade ("HANDEL").
    pub(super) fn default_trade_rng() -> Rng {
        Rng::from_seed(0x4841_4E44_454C_0000)
    }

    /// Schnaps generator of saves from before the Schnaps-Leiste
    /// ("SCHNAPS").
    pub(super) fn default_schnapps_rng() -> Rng {
        Rng::from_seed(0x5343_484E_4150_5300)
    }

    /// Card reward generator of saves from before it ("KREWARD").
    pub(super) fn default_reward_rng() -> Rng {
        Rng::from_seed(0x4B52_4557_4152_4400)
    }

    /// Zinken generator of saves from before the Auslage ("ZINKEN").
    pub(super) fn default_zinken_rng() -> Rng {
        Rng::from_seed(0x5A49_4E4B_454E_0000)
    }
}

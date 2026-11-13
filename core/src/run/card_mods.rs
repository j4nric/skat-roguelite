//! Card modifications on the player's cards: which may be drawn and
//! attached where, attaching (also paid, also at random), and redeeming
//! Marken (vouchers).

use crate::content::{
    CardModId, Content, Effect, Hook, HookContext, Source, Stage, evaluate, in_pool,
};
use crate::deck::{CardKind, DealtCard, DeckChange, Owner};
use crate::game::Seat;
use crate::rng::Rng;

use super::{Run, RunError, RunEvent, opponents};

/// Salt of the draw of the voucher a hand game redeems
/// ([`Run::redeem_hand_voucher`]): its own generator from the game's seed,
/// so the deal and every other draw stay the same.
const HAND_VOUCHER_SEED: u64 = 0x4A4E_D7E1_0000_0004;

/// Salt of the generator that picks the card of
/// [`Effect::AttachModToRandomCard`] (relics v4).
const ATTACH_SEED: u64 = 0xA77A_C4ED_0000_0006;

impl Run {
    /// The card modifications a random draw may give one deck card
    /// ([`Run::mod_pool`] without curses): only modifications that
    /// may go on it now ([`crate::content::CardModDef::only_on`], and by
    /// the rules of [`Deck::add_mod`](crate::deck::Deck::add_mod): not on a cursed card, not past the
    /// limit).
    pub(super) fn card_mod_pool_for(
        &self,
        content: &Content,
        card: &crate::deck::DeckCard,
    ) -> Vec<(CardModId, u32)> {
        self.mod_candidates(content, false)
            .into_iter()
            .filter(|(id, _)| self.may_attach(content, card.id, id))
            .collect()
    }

    /// Whether `card_mod` with its first value may go on the deck card
    /// `card` now. Tried on a copy of the deck.
    pub(super) fn may_attach(&self, content: &Content, card: u32, card_mod: &CardModId) -> bool {
        let Some(def) = content.card_mod(card_mod) else {
            return false;
        };
        let fits = self.edition.get(card).is_some_and(|c| def.fits(c.kind));
        fits && self
            .edition
            .probe_card(card)
            .add_mod(content, card, card_mod, def.default_value())
            .is_ok()
    }

    /// Card modifications a random draw may give a card (`curse` false;
    /// `true`: only curses, for draws that announce one), with their draw
    /// weight: for the character, unlocked, no curse unless asked for (DESIGN 7.2:
    /// curses only where announced) and no special card (that comes with a
    /// voucher), in id order. Rarity does not count here; the weight is the
    /// character's weight for the modification's Spielart
    /// ([`Run::focus_weight`]), and modifications of weight 0 are left out.
    pub(super) fn mod_pool(&self, content: &Content, curse: bool) -> Vec<(CardModId, u32)> {
        // Only what some card of the deck may still take.
        self.mod_candidates(content, curse)
            .into_iter()
            .filter(|(m, _)| {
                self.edition
                    .cards()
                    .iter()
                    .any(|c| self.may_attach(content, c.id, m))
            })
            .collect()
    }

    /// [`Run::mod_pool`] before asking whether some deck card may take a
    /// modification: for one deck card filter these with
    /// [`Run::may_attach`] directly (what fits that card fits some card).
    pub(super) fn mod_candidates(&self, content: &Content, curse: bool) -> Vec<(CardModId, u32)> {
        content
            .card_mods
            .values()
            .filter(|m| m.is_curse() == curse && !m.is_special())
            .filter(|m| in_pool(&m.characters, self.character))
            .filter(|m| !self.locked_card_mods.contains(&m.id))
            .filter(|m| m.is_offered())
            .map(|m| (m.id.clone(), self.focus_weight(content, m.focus)))
            .filter(|&(_, weight)| weight > 0)
            .collect()
    }

    /// [`Effect::AttachModToRandomCard`] ("Der Tankwart", "Der Klempner"):
    /// put `card_mod` with `value` on a random card of the player's edition
    /// that may take it (with `only_unmarked` only cards without any
    /// modification), drawn with a generator of its own from the game's
    /// seed and the modification, so every other draw stays the same.
    /// Candidates in id order. Nothing if none fits.
    pub(super) fn attach_to_random_card(
        &mut self,
        content: &Content,
        card_mod: &CardModId,
        value: i32,
        only_unmarked: bool,
        events: &mut Vec<RunEvent>,
    ) {
        let mut pool: Vec<u32> = self
            .edition
            .cards()
            .iter()
            .filter(|c| !only_unmarked || c.mods.is_empty())
            .map(|c| c.id)
            .filter(|&id| self.may_attach(content, id, card_mod))
            .collect();
        if pool.is_empty() {
            return;
        }
        pool.sort_unstable();
        let salt = card_mod
            .as_str()
            .bytes()
            .fold(0xCBF2_9CE4_8422_2325_u64, |h, b| {
                (h ^ u64::from(b)).wrapping_mul(0x0100_0000_01B3)
            });
        let mut rng = Rng::from_seed(opponents::game_seed(self) ^ ATTACH_SEED ^ salt);
        let len = u64::try_from(pool.len()).unwrap_or(1);
        let pick = usize::try_from(rng.gen_range(0..len)).unwrap_or(0);
        let card = pool[pick.min(pool.len() - 1)];
        // May only fail on content errors, which validation rules out.
        let _ = self.attach_mod(content, card, card_mod, value, events);
    }

    /// Attach a card modification with `value` to a deck card by the
    /// rules of effects v2 ([`Deck::add_mod`](crate::deck::Deck::add_mod)) and fire the `OnAttach`
    /// hooks of what the card carries then (a curse pays Groschen, not
    /// scaled by the value).
    pub(super) fn attach_mod(
        &mut self,
        content: &Content,
        card: u32,
        card_mod: &CardModId,
        value: i32,
        events: &mut Vec<RunEvent>,
    ) -> Result<(), RunError> {
        self.attach_mod_paid(content, card, card_mod, value, true, events)
    }

    /// Whether [`Run::attach_mod`] would attach `card_mod` with `value` to
    /// `card`, and if not, why: the same checks, the edition's tried on a
    /// copy of the one card. The `OnAttach` hooks that follow never fail.
    pub(super) fn check_attach(
        &self,
        content: &Content,
        card: u32,
        card_mod: &CardModId,
        value: i32,
    ) -> Result<(), RunError> {
        if let (Some(def), Some(deck_card)) = (content.card_mod(card_mod), self.edition.get(card)) {
            if deck_card.kind != CardKind::Blank && !def.fits(deck_card.kind) {
                return Err(RunError::NoModAvailable);
            }
        }
        self.edition
            .probe_card(card)
            .add_mod(content, card, card_mod, value)?;
        Ok(())
    }

    /// [`Run::attach_mod`]; with `paid` false the `OnAttach` hooks pay no
    /// Groschen (a curse as the price of a trade, [`trade`](super::trade), decided on
    /// 03.11.2026: the curse is the price, it pays nothing back).
    pub(super) fn attach_mod_paid(
        &mut self,
        content: &Content,
        card: u32,
        card_mod: &CardModId,
        value: i32,
        paid: bool,
        events: &mut Vec<RunEvent>,
    ) -> Result<(), RunError> {
        if let (Some(def), Some(deck_card)) = (content.card_mod(card_mod), self.edition.get(card)) {
            if deck_card.kind != CardKind::Blank && !def.fits(deck_card.kind) {
                return Err(RunError::NoModAvailable);
            }
        }
        let change = self.edition.add_mod(content, card, card_mod, value)?;
        // What the card carries now: Augen and Ramponiert are one signed
        // value, so the result may be the other one, or nothing.
        let landed = match &change {
            DeckChange::Modified { card_mod, .. } => Some(card_mod.clone()),
            _ => None,
        };
        events.push(RunEvent::DeckChanged(change));
        let Some((card_mod, def)) = landed.and_then(|m| content.card_mod(&m).map(|d| (m, d)))
        else {
            return Ok(());
        };
        let source = Source::CardMod {
            card_mod: card_mod.clone(),
            card,
        };
        for hook in def.hooks.iter().filter(|h| h.stage() == Stage::OnAttach) {
            let holds = {
                let ctx = HookContext {
                    flags: &self.flags,
                    counters: &self.counters,
                    game: None,
                    played: None,
                    result: None,
                    games_left: self.series.games_left(),
                    transport: None,
                    marked_cards: None,
                    run: Some((self, content)),
                };
                evaluate(hook.when(), &ctx, None)
            };
            // "Beim Erhalt" is not scaled: a curse pays its Groschen once,
            // never as the price of a trade.
            if holds {
                for effect in hook.then() {
                    if !paid && matches!(effect, Effect::GainCoins(_)) {
                        continue;
                    }
                    self.apply_effect(content, &source, effect, events);
                }
            }
        }
        Ok(())
    }

    /// Redeem the vouchers of the cards that lie in the skat after the
    /// discard ("wenn gedrückt", DESIGN 7.2): their `OnDiscard` hooks fire
    /// at once, scaled by their value, whatever becomes of the game, and
    /// the voucher falls off the card. `TurnInto` puts the special card on
    /// the card (from the next game on). A hand game discards nothing; it
    /// redeems one voucher of the hand instead ([`Run::redeem_hand_voucher`]).
    pub(super) fn redeem_vouchers(&mut self, content: &Content, events: &mut Vec<RunEvent>) {
        let Some(active) = self.game.as_ref() else {
            return;
        };
        if !active.game.skat_picked_up() {
            return;
        }
        let skat: Vec<u32> = active.game.skat().iter().map(|c| c.id).collect();
        for card in skat {
            for (card_mod, value) in self.vouchers_on(content, card) {
                self.redeem_voucher(content, card, &card_mod, value, false, events);
            }
        }
    }

    /// The vouchers on `card` in the player's edition with their values.
    fn vouchers_on(&self, content: &Content, card: u32) -> Vec<(CardModId, i32)> {
        let Some(deck_card) = self.edition.get(card) else {
            return Vec::new();
        };
        deck_card
            .mods
            .iter()
            .filter(|m| {
                content
                    .card_mod(m)
                    .is_some_and(|d| d.kind == crate::content::ModKind::Voucher)
            })
            .map(|m| (m.clone(), deck_card.mod_value(m)))
            .collect()
    }

    /// A hand game is declared (decided by the user on 04.11.2026, DESIGN
    /// 7.2): one voucher on a card of the player's hand, drawn with a
    /// generator of its own from the game's seed, is redeemed at once as if
    /// its card had been discarded. The card stays in the hand. Called just
    /// before the game starts, so a Narrenmarke's Wandelkarte takes its suit
    /// with the rest of the hand ("bei Spielbeginn") and an Untot may rise
    /// in this very game. Nothing happens without a voucher in the hand.
    pub(super) fn redeem_hand_voucher(&mut self, content: &Content, events: &mut Vec<RunEvent>) {
        let Some(active) = self.game.as_ref() else {
            return;
        };
        let held: Vec<DealtCard> = active
            .game
            .hand(Seat::DECLARER)
            .iter()
            .copied()
            .filter(|c| c.owner == Owner::Player)
            .collect();
        let mut candidates: Vec<(DealtCard, CardModId, i32)> = held
            .iter()
            .flat_map(|&card| {
                self.vouchers_on(content, card.id)
                    .into_iter()
                    .map(move |(m, v)| (card, m, v))
            })
            .collect();
        if candidates.is_empty() {
            return;
        }
        candidates.sort_by_key(|(card, m, _)| (card.id, m.clone()));
        let mut rng = Rng::from_seed(opponents::game_seed(self) ^ HAND_VOUCHER_SEED);
        let len = u64::try_from(candidates.len()).unwrap_or(1);
        let pick = usize::try_from(rng.gen_range(0..len)).unwrap_or(0);
        let (card, card_mod, value) = candidates.swap_remove(pick.min(candidates.len() - 1));
        self.redeem_voucher(content, card.id, &card_mod, value, true, events);
        // The game's copy of the card follows its edition: a Wandelkarte
        // now (or no longer, should Untot have replaced it).
        let wild = self.edition.get(card.id).is_some_and(|d| {
            d.mods
                .iter()
                .filter_map(|m| content.card_mod(m))
                .any(|def| def.passive.contains(&Effect::Wild))
        });
        if wild != card.wild {
            if let Some(active) = self.game.as_mut() {
                let new = DealtCard { wild, ..card };
                // Before the declaration the card is in the hand: cannot fail.
                let _ = active.game.replace_declarer_card(card, new);
            }
        }
    }

    /// Redeem one voucher on `card`: [`RunEvent::VoucherRedeemed`], the
    /// voucher falls off, its `OnDiscard` hooks fire scaled by `value`.
    /// "Der Drückeberger" remembers it for after the pub.
    fn redeem_voucher(
        &mut self,
        content: &Content,
        card: u32,
        card_mod: &CardModId,
        value: i32,
        via_hand: bool,
        events: &mut Vec<RunEvent>,
    ) {
        let Some(def) = content.card_mod(card_mod) else {
            return;
        };
        events.push(RunEvent::VoucherRedeemed {
            card,
            card_mod: card_mod.clone(),
            value,
            via_hand,
        });
        let vouchers_return = content
            .character(self.character)
            .is_some_and(|c| c.rule.vouchers_return);
        if vouchers_return {
            self.series
                .returning_vouchers
                .push((card, card_mod.clone(), value));
        }
        let source = Source::CardMod {
            card_mod: card_mod.clone(),
            card,
        };
        let hooks: Vec<Hook> = def
            .hooks
            .iter()
            .filter(|h| h.stage() == Stage::OnDiscard)
            .cloned()
            .collect();
        if let Some(change) = self.edition.remove_mod(card, card_mod) {
            events.push(RunEvent::DeckChanged(change));
        }
        // "Doppelt gemoppelt": everything a Marke does happens twice.
        let twice = self
            .held_passives(content)
            .any(|e| *e == Effect::RedeemVouchersTwice);
        for _ in 0..if twice { 2 } else { 1 } {
            self.fire_voucher(content, card, &source, &hooks, value, events);
        }
    }

    /// The hooks of one redeemed Marke: its `OnDiscard` hooks, scaled by
    /// its value, then every `OnVoucherRedeemed` hook.
    fn fire_voucher(
        &mut self,
        content: &Content,
        card: u32,
        source: &Source,
        hooks: &[Hook],
        value: i32,
        events: &mut Vec<RunEvent>,
    ) {
        for hook in hooks {
            let holds = {
                let ctx = HookContext {
                    flags: &self.flags,
                    counters: &self.counters,
                    game: None,
                    played: None,
                    result: None,
                    games_left: self.series.games_left(),
                    transport: None,
                    marked_cards: None,
                    run: Some((self, content)),
                };
                evaluate(hook.when(), &ctx, None)
            };
            if !holds {
                continue;
            }
            for effect in hook.then() {
                if let Effect::TurnInto(special) = effect {
                    self.turn_into(content, card, special, events);
                } else {
                    self.apply_effect(content, source, &effect.scaled(value), events);
                }
            }
        }
        // Relics that count redeemed Marken ("Markenheft", "Leergut").
        self.fire(content, Stage::OnVoucherRedeemed, None, None, events);
    }

    /// A redeemed voucher makes `card` the special card `special`
    /// (`TurnInto`). Since 05.11.2026 there is no cap on special cards
    /// (DESIGN 7.2): it always turns the card. A card that is the special
    /// card already stays so (the Drückeberger's returned Totenmarke,
    /// DESIGN 9.4).
    fn turn_into(
        &mut self,
        content: &Content,
        card: u32,
        special: &CardModId,
        events: &mut Vec<RunEvent>,
    ) {
        if let Ok(change) = self.edition.add_mod(content, card, special, 1) {
            events.push(RunEvent::DeckChanged(change));
        }
    }
}

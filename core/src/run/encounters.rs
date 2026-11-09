//! Encounters (Ereignisse, DESIGN 13) as data ("Wege und Begegnungen v2",
//! decision of the user, 03.11.2026).
//!
//! Every encounter lists two or three options in `encounters.ron`
//! ([`crate::content::EncounterOptionDef`]); each option is a price and a
//! small list of [`EncounterEffect`]s. When the encounter opens, whatever
//! the effects leave open (a house rule, a Schnaps, a relic, a card
//! modification) is drawn with the encounter generator, so the player sees
//! it before choosing. One option must be taken; there is no walking on.
//! An option that cannot be taken now names its reason ([`OptionBlock`]);
//! an encounter with no option to take is not opened (the way goes on).

use super::{
    ActiveEncounter, EncounterOption, OptionBlock, Run, RunError, RunEvent, RunPhase, Wager,
    draw_weighted, mod_value, sell_price,
};
use crate::content::{
    CardModDef, CardModId, CardPick, ConsumableCategory, ConsumableId, Content, EncounterDef,
    EncounterEffect, EncounterId, ModKind, Source, in_pool,
};
use crate::deck::CardKind;

/// What an unnamed modification is drawn from: the offers and whether a
/// curse.
type DrawKey = (Vec<(CardModId, i32)>, bool);

/// A drawn modification with its value.
type DrawnMod = (Option<CardModId>, Option<i32>);

/// What the player chose for an option: a card of the edition, an owned
/// relic (index), a Marke.
#[derive(Debug, Clone, Copy)]
struct Target<'a> {
    card: Option<u32>,
    relic: Option<usize>,
    marke: Option<&'a CardModId>,
}

impl Run {
    /// Open the encounter `id` (a stop on the way, or the loot of a
    /// Spelunke): its draws are made now. Returns `false` if no option can
    /// be taken; then the run is unchanged but for the draws.
    pub(super) fn open_encounter(
        &mut self,
        content: &Content,
        id: &EncounterId,
        events: &mut Vec<RunEvent>,
    ) -> bool {
        let Some(def) = content.encounter(id) else {
            return false;
        };
        let options = self.draw_options(content, def);
        let open = (0..options.len()).any(|i| self.option_block(content, &options, i).is_none());
        if !open {
            return false;
        }
        self.phase = RunPhase::Encounter;
        events.push(RunEvent::EncounterOpened {
            encounter: def.id.clone(),
            options: options.clone(),
        });
        self.encounter = Some(ActiveEncounter {
            id: def.id.clone(),
            options,
            chosen: None,
        });
        true
    }

    /// Whether the encounter would have an option to take now (checked
    /// before it is put on a route). Draws nothing: what an option leaves
    /// to the draw counts as there if the pool has it.
    pub(super) fn encounter_available(&self, content: &Content, def: &EncounterDef) -> bool {
        let rules = self.relic_pool(content, true).len();
        let schnapps = self.schnapps_pool(content).len();
        let relics = self.relic_pool(content, false);
        let options: Vec<EncounterOption> = def
            .options
            .iter()
            .map(|o| EncounterOption {
                cost: o.cost,
                effects: o
                    .effects
                    .iter()
                    .map(|e| match e {
                        // Stand-ins for the draws, only to ask the rules.
                        EncounterEffect::EveningRule(None) if rules > 0 => {
                            EncounterEffect::Coins(0)
                        }
                        EncounterEffect::Schnapps(None) if schnapps > 0 => {
                            EncounterEffect::Coins(0)
                        }
                        EncounterEffect::SwapRelic(None) | EncounterEffect::Relic(None)
                            if !relics.is_empty() =>
                        {
                            match e {
                                EncounterEffect::SwapRelic(_) => {
                                    EncounterEffect::SwapRelic(Some(relics[0].0.clone()))
                                }
                                _ => EncounterEffect::Relic(Some(relics[0].0.clone())),
                            }
                        }
                        EncounterEffect::RelicCoins(_) if !relics.is_empty() => {
                            EncounterEffect::Coins(0)
                        }
                        EncounterEffect::Mark {
                            card,
                            card_mod: None,
                            offers,
                            curse,
                            ..
                        } => {
                            let drawn = if offers.is_empty() {
                                self.mod_pool(content, *curse)
                                    .first()
                                    .map(|(m, _)| m.clone())
                            } else {
                                offers
                                    .iter()
                                    .map(|(m, _)| m)
                                    .find(|m| content.card_mod(m).is_some())
                                    .cloned()
                            };
                            EncounterEffect::Mark {
                                card: *card,
                                card_mod: drawn,
                                value: None,
                                offers: Vec::new(),
                                curse: false,
                                curse_percent: 0,
                            }
                        }
                        other => other.clone(),
                    })
                    .collect(),
                fallback: o.fallback,
            })
            .collect();
        (0..options.len()).any(|i| self.option_block(content, &options, i).is_none())
    }

    /// The Schnäpse an encounter may draw, with their rarity weight.
    pub(super) fn schnapps_pool(&self, content: &Content) -> Vec<(ConsumableId, u32)> {
        let weights = &content.run.shop.rarity_weights;
        content
            .consumables
            .values()
            .filter(|c| c.category == ConsumableCategory::Schnapps)
            .filter(|c| in_pool(&c.characters, self.character))
            .map(|c| (c.id.clone(), weights.weight(c.rarity)))
            .collect()
    }

    /// The options of `def` with every draw made: house rules and Schnäpse
    /// distinct across the options, one relic for the whole encounter, a
    /// modification per [`EncounterEffect::Mark`] that names none.
    fn draw_options(&mut self, content: &Content, def: &EncounterDef) -> Vec<EncounterOption> {
        let effects = || def.options.iter().flat_map(|o| &o.effects);
        let count = |wanted: fn(&EncounterEffect) -> bool| {
            u32::try_from(effects().filter(|e| wanted(e)).count()).unwrap_or(0)
        };
        let rules = count(|e| matches!(e, EncounterEffect::EveningRule(None)));
        let pool = self.relic_pool(content, true);
        let mut rules = draw_weighted(&mut self.encounter_rng, pool, rules).into_iter();
        let schnapps = count(|e| matches!(e, EncounterEffect::Schnapps(None)));
        let pool = self.schnapps_pool(content);
        let mut schnapps = draw_weighted(&mut self.encounter_rng, pool, schnapps).into_iter();
        let wants_relic = count(|e| {
            matches!(
                e,
                EncounterEffect::Relic(None)
                    | EncounterEffect::SwapRelic(None)
                    | EncounterEffect::RelicCoins(_)
            )
        }) > 0;
        let relic = if wants_relic {
            let pool = self.relic_pool(content, false);
            draw_weighted(&mut self.encounter_rng, pool, 1).pop()
        } else {
            None
        };
        // One draw per kind of unnamed modification: the Kartengeber names
        // the same Zinke for the random and the chosen card.
        let mut mods: Vec<(DrawKey, DrawnMod)> = Vec::new();
        let mut options = Vec::new();
        for option in &def.options {
            let mut drawn = Vec::new();
            for effect in &option.effects {
                drawn.push(match effect {
                    EncounterEffect::EveningRule(None) => {
                        EncounterEffect::EveningRule(rules.next())
                    }
                    EncounterEffect::Schnapps(None) => EncounterEffect::Schnapps(schnapps.next()),
                    EncounterEffect::Relic(None) => EncounterEffect::Relic(relic.clone()),
                    EncounterEffect::SwapRelic(None) => EncounterEffect::SwapRelic(relic.clone()),
                    EncounterEffect::RelicCoins(percent) => match &relic {
                        Some(id) => {
                            let price = content.relic(id).map_or(0, |r| r.price);
                            let coins = u64::from(price) * u64::from(*percent) / 100;
                            EncounterEffect::Coins(u32::try_from(coins).unwrap_or(0))
                        }
                        None => effect.clone(),
                    },
                    EncounterEffect::Mark {
                        card,
                        card_mod,
                        value,
                        offers,
                        curse,
                        curse_percent,
                    } => {
                        let key = (offers.clone(), *curse);
                        let known = mods.iter().find(|(k, _)| *k == key).map(|(_, m)| m.clone());
                        let (card_mod, value) = match (card_mod, known) {
                            (Some(id), _) => (Some(id.clone()), *value),
                            (None, Some(known)) => known,
                            (None, None) if !offers.is_empty() => {
                                let pool: Vec<((CardModId, i32), u32)> = offers
                                    .iter()
                                    .filter(|(m, _)| content.card_mod(m).is_some())
                                    .map(|o| (o.clone(), 1))
                                    .collect();
                                draw_weighted(&mut self.encounter_rng, pool, 1)
                                    .pop()
                                    .map_or((None, None), |(m, v)| (Some(m), Some(v)))
                            }
                            (None, None) => {
                                let pool = self.mod_pool(content, *curse);
                                (draw_weighted(&mut self.encounter_rng, pool, 1).pop(), None)
                            }
                        };
                        let value = value.or_else(|| {
                            card_mod
                                .as_ref()
                                .and_then(|m| content.card_mod(m))
                                .map(CardModDef::default_value)
                        });
                        if effect_names_none(effect) && !mods.iter().any(|(k, _)| *k == key) {
                            mods.push((key, (card_mod.clone(), value)));
                        }
                        EncounterEffect::Mark {
                            card: *card,
                            card_mod,
                            value,
                            offers: Vec::new(),
                            curse: *curse,
                            curse_percent: *curse_percent,
                        }
                    }
                    other => other.clone(),
                });
            }
            options.push(EncounterOption {
                cost: option.cost,
                effects: drawn,
                fallback: option.fallback,
            });
        }
        options
    }

    /// Whether `option` needs an owned relic (its index) as target now: a
    /// swap or a sale always, a relic gained with every slot full too (the
    /// owned one makes room and is sold for half its price, DESIGN 8.0).
    #[must_use]
    pub fn option_needs_relic(&self, content: &Content, option: &EncounterOption) -> bool {
        option.needs_relic()
            || (option.gains_relic() && !content.relic_slot_free(self.character, self.relics.len()))
    }

    /// Why option `index` of the open encounter cannot be taken now, or
    /// `None` if it can (with a fitting target if it needs one).
    #[must_use]
    pub fn encounter_option_block(&self, content: &Content, index: usize) -> Option<OptionBlock> {
        let encounter = self.encounter.as_ref()?;
        self.option_block(content, &encounter.options, index)
    }

    /// [`Run::encounter_option_block`] for one option among `options`.
    fn option_block(
        &self,
        content: &Content,
        options: &[EncounterOption],
        index: usize,
    ) -> Option<OptionBlock> {
        let option = options.get(index)?;
        if option.fallback {
            let other = options
                .iter()
                .any(|o| !o.fallback && self.single_block(content, o).is_none());
            if other {
                return Some(OptionBlock::NotNeeded);
            }
        }
        self.single_block(content, option)
    }

    /// Cards of the edition the option's chosen-card effects may all take
    /// now ([`CardPick::Chosen`]); every card without such an effect.
    fn fitting_cards(&self, content: &Content, option: &EncounterOption) -> Vec<u32> {
        self.edition
            .cards()
            .iter()
            .filter(|c| c.kind != CardKind::Blank)
            .map(|c| c.id)
            .filter(|&card| {
                option.effects.iter().all(|e| match e {
                    EncounterEffect::Mark {
                        card: CardPick::Chosen,
                        card_mod: Some(m),
                        ..
                    } => self.may_attach(content, card, m),
                    EncounterEffect::Marke {
                        card: CardPick::Chosen,
                        ..
                    } => self.marken_for(content, Some(card)).next().is_some(),
                    EncounterEffect::Cleanse => {
                        self.edition.get(card).is_some_and(|c| !c.mods.is_empty())
                    }
                    _ => true,
                })
            })
            .collect()
    }

    /// Every Marke (card modification of kind [`ModKind::Voucher`]) for the
    /// character, unlocked, in the order of their ids, whether or not one
    /// may go on a card now: what a Marke option shows.
    #[must_use]
    pub fn marken<'c>(&self, content: &'c Content) -> Vec<&'c CardModDef> {
        content
            .card_mods
            .values()
            .filter(|m| m.kind == ModKind::Voucher)
            .filter(|m| in_pool(&m.characters, self.character))
            .filter(|m| !self.locked_card_mods.contains(&m.id))
            .collect()
    }

    /// The Marken that may go on a card now: on `card`, or (`None`) on
    /// some card of the edition.
    fn marken_for<'c>(
        &'c self,
        content: &'c Content,
        card: Option<u32>,
    ) -> impl Iterator<Item = &'c CardModDef> + 'c {
        self.marken(content)
            .into_iter()
            .filter(move |m| match card {
                Some(card) => self.may_attach(content, card, &m.id),
                None => self
                    .edition
                    .cards()
                    .iter()
                    .any(|c| c.kind != CardKind::Blank && self.may_attach(content, c.id, &m.id)),
            })
    }

    /// Whether the Marke `marke` may go on `card` now.
    #[must_use]
    pub fn marke_fits_card(&self, content: &Content, card: u32, marke: &CardModId) -> bool {
        self.marken_for(content, Some(card)).any(|m| &m.id == marke)
    }

    /// Whether the Marke `marke` may be taken for `option` on `card` (the
    /// chosen card, `None` for a random one): a Marke of the content that
    /// fits.
    #[must_use]
    pub fn marke_fits(
        &self,
        content: &Content,
        option: &EncounterOption,
        marke: &CardModId,
        card: Option<u32>,
    ) -> bool {
        option.effects.iter().all(|e| match e {
            EncounterEffect::Marke {
                card: pick,
                marke: CardPick::Chosen,
            } => {
                let on = match pick {
                    CardPick::Chosen => card,
                    CardPick::Random => None,
                };
                self.marken_for(content, on).any(|m| &m.id == marke)
            }
            _ => true,
        })
    }

    /// What the option costs on `card`: a Zinke on a chosen card may cost
    /// less ("Bruderherz", [`Run::zinken_price`]).
    pub(super) fn option_cost(
        &self,
        content: &Content,
        option: &EncounterOption,
        card: Option<u32>,
    ) -> u32 {
        let zinke = option.effects.iter().any(|e| {
            matches!(
                e,
                EncounterEffect::Mark {
                    card: CardPick::Chosen,
                    ..
                } | EncounterEffect::Marke {
                    card: CardPick::Chosen,
                    ..
                }
            )
        });
        match card {
            Some(card) if zinke => self.zinken_price(content, card, option.cost),
            _ => option.cost,
        }
    }

    /// Why one option cannot be taken, by itself.
    fn single_block(&self, content: &Content, option: &EncounterOption) -> Option<OptionBlock> {
        for effect in &option.effects {
            let block = match effect {
                EncounterEffect::EveningRule(None)
                | EncounterEffect::Schnapps(None)
                | EncounterEffect::Relic(None)
                | EncounterEffect::SwapRelic(None)
                | EncounterEffect::RelicCoins(_)
                | EncounterEffect::Mark { card_mod: None, .. } => Some(OptionBlock::NothingLeft),
                EncounterEffect::Mark {
                    card,
                    card_mod: Some(m),
                    ..
                } => match content.card_mod(m) {
                    None => Some(OptionBlock::NothingLeft),
                    Some(_) => {
                        let fits = match card {
                            CardPick::Random => self.edition.cards().iter().any(|c| {
                                c.kind != CardKind::Blank && self.may_attach(content, c.id, m)
                            }),
                            CardPick::Chosen => !self.fitting_cards(content, option).is_empty(),
                        };
                        (!fits).then_some(OptionBlock::NoFittingCard)
                    }
                },
                EncounterEffect::Marke { card, .. } => {
                    if self.marken(content).is_empty() {
                        Some(OptionBlock::NothingLeft)
                    } else {
                        let fits = match card {
                            CardPick::Random => self.marken_for(content, None).next().is_some(),
                            CardPick::Chosen => !self.fitting_cards(content, option).is_empty(),
                        };
                        (!fits).then_some(OptionBlock::NoFittingCard)
                    }
                }
                EncounterEffect::Cleanse => self
                    .fitting_cards(content, option)
                    .is_empty()
                    .then_some(OptionBlock::NoFittingCard),
                EncounterEffect::SwapRelic(Some(_)) | EncounterEffect::SellRelic(_) => {
                    self.relics.is_empty().then_some(OptionBlock::NoRelic)
                }
                // With every slot full an owned relic makes room
                // (`option_needs_relic`); with none owned it cannot.
                EncounterEffect::Relic(Some(_)) => (!content
                    .relic_slot_free(self.character, self.relics.len())
                    && self.relics.is_empty())
                .then_some(OptionBlock::RelicSlotsFull),
                EncounterEffect::Bet { .. } => self.wager.is_some().then_some(OptionBlock::BetOpen),
                _ => None,
            };
            if block.is_some() {
                return block;
            }
        }
        let lowest = if option.needs_card() {
            self.fitting_cards(content, option)
                .into_iter()
                .map(|card| self.option_cost(content, option, Some(card)))
                .min()
                .unwrap_or(option.cost)
        } else {
            option.cost
        };
        (lowest > self.coins).then_some(OptionBlock::NotEnoughCoins)
    }

    /// A random card of the player's edition `card_mod` may go on now, a
    /// bare one first, never one of `except`. Drawn with the encounter
    /// generator.
    fn random_card_for(
        &mut self,
        content: &Content,
        card_mod: &CardModId,
        except: &[u32],
    ) -> Option<u32> {
        let fitting: Vec<&crate::deck::DeckCard> = self
            .edition
            .cards()
            .iter()
            .filter(|c| c.kind != CardKind::Blank && !except.contains(&c.id))
            .filter(|c| self.may_attach(content, c.id, card_mod))
            .collect();
        let bare: Vec<u32> = fitting
            .iter()
            .filter(|c| c.mods.is_empty())
            .map(|c| c.id)
            .collect();
        let cards: Vec<u32> = if bare.is_empty() {
            fitting.iter().map(|c| c.id).collect()
        } else {
            bare
        };
        if cards.is_empty() {
            return None;
        }
        let pick = self
            .encounter_rng
            .gen_range(0..u64::try_from(cards.len()).unwrap_or(1));
        cards.get(usize::try_from(pick).unwrap_or(0)).copied()
    }

    /// [`super::RunAction::ChooseEncounter`]: take option `option`, with a
    /// card of the edition (or, for a relic option, the index of an owned
    /// relic) as `target` if it needs one, and the Marke `marke` if it lets
    /// the player choose one. All or nothing: on an error the run is
    /// unchanged.
    pub(super) fn choose_encounter(
        &mut self,
        content: &Content,
        option: usize,
        target: Option<u32>,
        marke: Option<&CardModId>,
        events: &mut Vec<RunEvent>,
    ) -> Result<(), RunError> {
        if self.phase != RunPhase::Encounter {
            return Err(RunError::WrongPhase);
        }
        let encounter = self.encounter.clone().ok_or(RunError::WrongPhase)?;
        if encounter.chosen.is_some() {
            return Err(RunError::PainterPending);
        }
        let chosen = encounter
            .options
            .get(option)
            .cloned()
            .ok_or(RunError::InvalidSlot)?;
        if let Some(block) = self.option_block(content, &encounter.options, option) {
            return Err(block.error());
        }
        let card = if chosen.needs_card() {
            let card = target.ok_or(RunError::TargetRequired)?;
            let deck_card = self
                .edition
                .get(card)
                .ok_or(crate::deck::DeckError::UnknownCard(card))?;
            if deck_card.kind == CardKind::Blank {
                return Err(crate::deck::DeckError::BlankCannotBeModified.into());
            }
            if !self.fitting_cards(content, &chosen).contains(&card) {
                return Err(RunError::NoModAvailable);
            }
            Some(card)
        } else {
            None
        };
        let relic = if self.option_needs_relic(content, &chosen) {
            let index = usize::try_from(target.ok_or(RunError::TargetRequired)?)
                .map_err(|_| RunError::InvalidSlot)?;
            if index >= self.relics.len() {
                return Err(RunError::InvalidSlot);
            }
            Some(index)
        } else {
            None
        };
        let marke = if chosen.needs_marke() {
            let marke = marke.ok_or(RunError::TargetRequired)?;
            if !self.marke_fits(content, &chosen, marke, card) {
                return Err(RunError::NoModAvailable);
            }
            Some(marke)
        } else {
            None
        };
        let cost = self.option_cost(content, &chosen, card);
        if cost > self.coins {
            return Err(RunError::NotEnoughCoins);
        }
        let mut next = self.clone();
        let mut out = Vec::new();
        next.coins -= cost;
        let target = Target { card, relic, marke };
        let painter = next.apply_option(content, &encounter.id, &chosen, target, cost, &mut out)?;
        if painter {
            let card = card.ok_or(RunError::TargetRequired)?;
            if let Some(open) = next.encounter.as_mut() {
                open.chosen = Some(option);
            }
            next.start_painter(content, card, 0, true, &mut out)?;
        } else {
            next.settle_debt(&mut out);
            out.push(RunEvent::EncounterResolved {
                encounter: encounter.id.clone(),
                option,
            });
            next.encounter = None;
            next.continue_trip(content, &mut out)?;
        }
        *self = next;
        events.extend(out);
        Ok(())
    }

    /// Apply the effects of an option in order. Returns `true` if it ends
    /// with the Kartenmaler (the encounter waits for his choice).
    #[allow(clippy::too_many_lines)]
    fn apply_option(
        &mut self,
        content: &Content,
        id: &EncounterId,
        option: &EncounterOption,
        target: Target<'_>,
        stake: u32,
        events: &mut Vec<RunEvent>,
    ) -> Result<bool, RunError> {
        let Target { card, relic, marke } = target;
        let source = Source::Encounter(id.clone());
        let mut marked: Vec<u32> = Vec::new();
        for effect in &option.effects {
            match effect {
                EncounterEffect::Coins(n) => self.gain_coins(&source, *n, events),
                EncounterEffect::EveningRule(Some(rule)) => {
                    self.evening_relics.push(rule.clone());
                    events.push(RunEvent::EveningRuleAdded(rule.clone()));
                }
                EncounterEffect::Mark {
                    card: pick,
                    card_mod: Some(card_mod),
                    value,
                    curse_percent,
                    ..
                } => {
                    let target = match pick {
                        CardPick::Chosen => card.ok_or(RunError::TargetRequired)?,
                        CardPick::Random => {
                            match self.random_card_for(content, card_mod, &marked) {
                                Some(target) => target,
                                // Nothing left to take it (a curse after
                                // the chosen card): it falls away.
                                None => continue,
                            }
                        }
                    };
                    let curse = if *curse_percent > 0 {
                        let hit = self.encounter_rng.gen_range(0..100) < u64::from(*curse_percent);
                        let curse = if hit {
                            let pool: Vec<(CardModId, u32)> = self
                                .mod_candidates(content, true)
                                .into_iter()
                                .filter(|(m, _)| self.may_attach(content, target, m))
                                .collect();
                            draw_weighted(&mut self.encounter_rng, pool, 1).pop()
                        } else {
                            None
                        };
                        events.push(RunEvent::GambleResolved {
                            curse: curse.is_some(),
                        });
                        curse
                    } else {
                        None
                    };
                    match curse {
                        Some(curse) => {
                            let value = mod_value(content, &curse, None);
                            self.attach_mod(content, target, &curse, value, events)?;
                        }
                        None => {
                            if !self.may_attach(content, target, card_mod) {
                                return Err(RunError::NoModAvailable);
                            }
                            let value = mod_value(content, card_mod, *value);
                            self.attach_mod(content, target, card_mod, value, events)?;
                        }
                    }
                    marked.push(target);
                }
                EncounterEffect::Marke {
                    card: card_pick,
                    marke: marke_pick,
                } => {
                    let on = match card_pick {
                        CardPick::Chosen => Some(card.ok_or(RunError::TargetRequired)?),
                        CardPick::Random => None,
                    };
                    let chosen_marke = match marke_pick {
                        CardPick::Chosen => marke.ok_or(RunError::TargetRequired)?.clone(),
                        CardPick::Random => {
                            let weights = &content.run.shop.rarity_weights;
                            let pool: Vec<(CardModId, u32)> = self
                                .marken_for(content, on)
                                .map(|m| (m.id.clone(), weights.weight(m.rarity)))
                                .collect();
                            draw_weighted(&mut self.encounter_rng, pool, 1)
                                .pop()
                                .ok_or(RunError::NoModAvailable)?
                        }
                    };
                    let target = match on {
                        Some(card) => card,
                        None => self
                            .random_card_for(content, &chosen_marke, &marked)
                            .ok_or(RunError::NoModAvailable)?,
                    };
                    if !self.may_attach(content, target, &chosen_marke) {
                        return Err(RunError::NoModAvailable);
                    }
                    let value = mod_value(content, &chosen_marke, None);
                    events.push(RunEvent::MarkeGiven {
                        card: target,
                        card_mod: chosen_marke.clone(),
                        drawn: *marke_pick == CardPick::Random,
                        random_card: *card_pick == CardPick::Random,
                    });
                    self.attach_mod(content, target, &chosen_marke, value, events)?;
                    marked.push(target);
                }
                EncounterEffect::SwapRelic(Some(new)) => {
                    let index = relic.ok_or(RunError::TargetRequired)?;
                    let old = std::mem::replace(&mut self.relics[index], new.clone());
                    self.relic_given_up(content, &old, events);
                    events.push(RunEvent::RelicSwapped {
                        old,
                        new: new.clone(),
                    });
                }
                EncounterEffect::SellRelic(percent) => {
                    let index = relic.ok_or(RunError::TargetRequired)?;
                    let old = self.relics.remove(index);
                    self.relic_given_up(content, &old, events);
                    let price = content.relic(&old).map_or(0, |d| d.price);
                    let price =
                        u32::try_from(u64::from(price) * u64::from(*percent) / 100).unwrap_or(0);
                    self.coins = self.coins.saturating_add(price);
                    events.push(RunEvent::RelicSold { relic: old, price });
                }
                EncounterEffect::Relic(Some(new)) => {
                    if !content.relic_slot_free(self.character, self.relics.len()) {
                        // Every slot full: the chosen relic makes room,
                        // sold for half its price (DESIGN 8.0).
                        let index = relic.ok_or(RunError::TargetRequired)?;
                        let old = self.relics.remove(index);
                        self.relic_given_up(content, &old, events);
                        let price = content.relic(&old).map_or(0, |d| sell_price(d.price));
                        self.coins = self.coins.saturating_add(price);
                        events.push(RunEvent::RelicSold { relic: old, price });
                        self.settle_debt(events);
                    }
                    self.relics.push(new.clone());
                    events.push(RunEvent::RelicGained { relic: new.clone() });
                }
                EncounterEffect::NextGameFactor(factor) => {
                    self.pending_factor = Some((id.clone(), *factor));
                    events.push(RunEvent::NextGameDoubled { factor: *factor });
                }
                EncounterEffect::NextPubRedeals(n) => self.plan_next_pub(*n, 0, events),
                EncounterEffect::SeeSkat => {
                    self.peek_next_skat = Some(id.clone());
                    events.push(RunEvent::SkatPeekPlanned);
                }
                EncounterEffect::Schnapps(Some(item)) => {
                    let slots = usize::try_from(content.run.consumable_slots).unwrap_or(0);
                    if self.consumables.len() < slots {
                        self.consumables.push(item.clone());
                        events.push(RunEvent::ConsumableGained {
                            consumable: item.clone(),
                        });
                    } else {
                        // Every slot full: its price in Groschen instead.
                        let price = content.consumable(item).map_or(0, |c| c.price);
                        self.gain_coins(&source, price, events);
                    }
                }
                // Converted to Groschen when a save is loaded (ADR 0019).
                EncounterEffect::Lives(n) => self.gain_coins(
                    &source,
                    n.saturating_mul(crate::content::LEGACY_LIFE_COINS),
                    events,
                ),
                EncounterEffect::Scout(legs) => {
                    self.scout_legs = self.scout_legs.max(*legs);
                    events.push(RunEvent::StopsRevealed {
                        legs: self.scout_legs,
                    });
                }
                EncounterEffect::Bet { on, percent } => {
                    if self.wager.is_some() {
                        return Err(RunError::ConditionNotMet);
                    }
                    self.wager = Some(Wager {
                        encounter: id.clone(),
                        stake,
                        factor: percent / 100,
                        on: *on,
                        percent: *percent,
                    });
                    events.push(RunEvent::WagerPlaced { stake });
                }
                EncounterEffect::Dice { chance, percent } => {
                    let won = self.encounter_rng.gen_range(0..100) < u64::from(*chance);
                    let payout = if won {
                        u32::try_from(u64::from(stake) * u64::from(*percent) / 100)
                            .unwrap_or(u32::MAX)
                    } else {
                        0
                    };
                    events.push(RunEvent::DiceThrown { stake, won, payout });
                    if payout > 0 {
                        self.gain_coins(&source, payout, events);
                    }
                }
                EncounterEffect::Painter => return Ok(true),
                EncounterEffect::Cleanse => {
                    let card = card.ok_or(RunError::TargetRequired)?;
                    let change = self.cleanse(content, card)?;
                    events.push(RunEvent::DeckChanged(change));
                }
                EncounterEffect::EveningRule(None)
                | EncounterEffect::Schnapps(None)
                | EncounterEffect::Relic(None)
                | EncounterEffect::SwapRelic(None)
                | EncounterEffect::RelicCoins(_)
                | EncounterEffect::Mark { card_mod: None, .. } => {
                    return Err(RunError::NoModAvailable);
                }
            }
        }
        Ok(false)
    }

    /// Groschen from an encounter.
    fn gain_coins(&mut self, source: &Source, n: u32, events: &mut Vec<RunEvent>) {
        if n == 0 {
            return;
        }
        self.coins = self.coins.saturating_add(n);
        events.push(RunEvent::CoinsGained {
            source: source.clone(),
            amount: i32::try_from(n).unwrap_or(i32::MAX),
            coins: self.coins,
        });
    }

    /// A loaded encounter of the form before 03.11.2026 (no options) is
    /// opened anew; one that has nothing to offer any more is left, and
    /// the way goes on. Ids the content no longer knows leave their option
    /// without what they named (it is shown as not available).
    pub(super) fn renew_encounter(&mut self, content: &Content) {
        let Some(encounter) = self.encounter.as_mut() else {
            return;
        };
        for option in &mut encounter.options {
            for effect in &mut option.effects {
                // No lives since ADR 0019: Groschen instead.
                if let EncounterEffect::Lives(n) = effect {
                    *effect =
                        EncounterEffect::Coins(n.saturating_mul(crate::content::LEGACY_LIFE_COINS));
                }
                let unknown = match effect {
                    EncounterEffect::EveningRule(Some(id))
                    | EncounterEffect::Relic(Some(id))
                    | EncounterEffect::SwapRelic(Some(id)) => content.relic(id).is_none(),
                    EncounterEffect::Schnapps(Some(id)) => content.consumable(id).is_none(),
                    EncounterEffect::Mark {
                        card_mod: Some(id), ..
                    } => content.card_mod(id).is_none(),
                    _ => false,
                };
                if unknown {
                    match effect {
                        EncounterEffect::EveningRule(id)
                        | EncounterEffect::Relic(id)
                        | EncounterEffect::SwapRelic(id) => *id = None,
                        EncounterEffect::Schnapps(id) => *id = None,
                        EncounterEffect::Mark { card_mod, .. } => *card_mod = None,
                        _ => {}
                    }
                }
            }
        }
        if !encounter.options.is_empty() || self.painter.is_some() {
            return;
        }
        let id = encounter.id.clone();
        self.encounter = None;
        let mut events = Vec::new();
        if !self.open_encounter(content, &id, &mut events) {
            // Nothing to offer: on along the way, as if it had been passed.
            let _ = self.continue_trip(content, &mut events);
        }
    }
}

#[cfg(test)]
impl Run {
    /// Tests: open the encounter `id` from the content (draws made, no
    /// check whether an option can be taken) as the only stop on the way
    /// to the next pub.
    pub(crate) fn test_encounter(&mut self, content: &Content, id: &str) -> &mut ActiveEncounter {
        let def = content
            .encounter(&EncounterId::new(id))
            .expect("encounter")
            .clone();
        let options = self.draw_options(content, &def);
        self.set_encounter(id, options)
    }

    /// Tests: open an encounter with these options by hand, as the only
    /// stop on the way to the next pub.
    pub(crate) fn set_encounter(
        &mut self,
        id: &str,
        options: Vec<EncounterOption>,
    ) -> &mut ActiveEncounter {
        self.game = None;
        self.shop = None;
        self.phase = RunPhase::Encounter;
        let (section, index) = self.next_series_position();
        self.trip = Some(super::tour::Trip {
            section,
            index,
            transport: super::tour::Transport::Walk,
            stops: Vec::new(),
            spelunke: None,
        });
        self.encounter.insert(ActiveEncounter {
            id: EncounterId::new(id),
            options,
            chosen: None,
        })
    }
}

/// Tests: an option with these effects.
#[cfg(test)]
pub(crate) fn test_option(cost: u32, effects: Vec<EncounterEffect>) -> EncounterOption {
    EncounterOption {
        cost,
        effects,
        fallback: false,
    }
}

/// Whether a Mark leaves its modification to the draw.
fn effect_names_none(effect: &EncounterEffect) -> bool {
    matches!(effect, EncounterEffect::Mark { card_mod: None, .. })
}

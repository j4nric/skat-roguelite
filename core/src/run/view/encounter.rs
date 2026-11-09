//! The view of an open encounter: its options with price, effects and
//! targets, and the Marken a "Marke" option can put on a card.

use crate::content::{CardModId, CardPick, Content, EncounterEffect, StopCategory};
use crate::deck::CardKind;

use super::{
    EncounterEffectView, EncounterOption, EncounterOptionView, EncounterView, MarkeChoiceView,
    OptionTarget, Run, RunAction,
};

impl Run {
    pub(super) fn encounter_view(&self, content: &Content) -> Option<EncounterView> {
        let encounter = self.encounter.as_ref()?;
        let def = content.encounter(&encounter.id);
        let item_slots = usize::try_from(content.run.consumable_slots).unwrap_or(0);
        let slots_full = self.consumables.len() >= item_slots;
        let options = encounter
            .options
            .iter()
            .enumerate()
            .map(|(index, option)| {
                let effects = option
                    .effects
                    .iter()
                    .map(|effect| {
                        let mut view = EncounterEffectView {
                            effect: effect.clone(),
                            relic: None,
                            card_mod: None,
                            consumable: None,
                            coins_instead: None,
                        };
                        match effect {
                            EncounterEffect::EveningRule(Some(id))
                            | EncounterEffect::Relic(Some(id))
                            | EncounterEffect::SwapRelic(Some(id)) => {
                                view.relic = Some(Run::relic_info(content, id));
                            }
                            EncounterEffect::Mark {
                                card_mod: Some(id),
                                value,
                                ..
                            } => view.card_mod = Some(Run::card_mod_info(content, id, *value)),
                            EncounterEffect::Schnapps(Some(id)) => {
                                view.consumable = Some(Run::consumable_info(content, id));
                                if slots_full {
                                    view.coins_instead =
                                        Some(content.consumable(id).map_or(0, |c| c.price));
                                }
                            }
                            _ => {}
                        }
                        view
                    })
                    .collect();
                let target = if option.needs_card() {
                    OptionTarget::Card
                } else if self.option_needs_relic(content, option) {
                    OptionTarget::Relic
                } else {
                    OptionTarget::None
                };
                let candidates: Vec<u32> = match target {
                    OptionTarget::Card => self.edition.cards().iter().map(|c| c.id).collect(),
                    OptionTarget::Relic => (0..self.relics.len())
                        .filter_map(|i| u32::try_from(i).ok())
                        .collect(),
                    OptionTarget::None => Vec::new(),
                };
                let blocked = self.encounter_option_block(content, index);
                let marken = self.marke_choices(content, index, option, blocked.is_some());
                // With a Marke to choose: a card any Marke may go on.
                let marke_ids: Vec<Option<CardModId>> = if option.needs_marke() {
                    marken.iter().map(|m| Some(m.card_mod.id.clone())).collect()
                } else {
                    vec![None]
                };
                let targets: Vec<u32> = if blocked.is_some() {
                    Vec::new()
                } else {
                    candidates
                        .into_iter()
                        .filter(|&t| {
                            marke_ids.iter().any(|marke| {
                                self.accepts(
                                    content,
                                    RunAction::ChooseEncounter {
                                        option: index,
                                        target: Some(t),
                                        marke: marke.clone(),
                                    },
                                )
                            })
                        })
                        .collect()
                };
                let target_costs: Vec<u32> = targets
                    .iter()
                    .map(|&t| match target {
                        OptionTarget::Card => self.option_cost(content, option, Some(t)),
                        _ => option.cost,
                    })
                    .collect();
                let can_choose = match target {
                    OptionTarget::None => marke_ids.iter().any(|marke| {
                        self.accepts(
                            content,
                            RunAction::ChooseEncounter {
                                option: index,
                                target: None,
                                marke: marke.clone(),
                            },
                        )
                    }),
                    _ => !targets.is_empty(),
                };
                EncounterOptionView {
                    index,
                    label: def
                        .and_then(|d| d.options.get(index))
                        .map_or_else(String::new, |o| o.label.clone()),
                    cost: option.cost,
                    effects,
                    fallback: option.fallback,
                    can_choose,
                    blocked,
                    target,
                    targets,
                    target_costs,
                    marken,
                }
            })
            .collect();
        Some(EncounterView {
            id: encounter.id.clone(),
            name: def.map_or_else(|| encounter.id.to_string(), |d| d.name.clone()),
            text: def.map_or_else(String::new, |d| d.text.clone()),
            category: def.map_or(StopCategory::Event, |d| d.category),
            options,
        })
    }

    /// The Marken of option `index` if it gives one
    /// ([`EncounterEffect::Marke`]), see [`MarkeChoiceView`].
    fn marke_choices(
        &self,
        content: &Content,
        index: usize,
        option: &EncounterOption,
        blocked: bool,
    ) -> Vec<MarkeChoiceView> {
        let Some((card_pick, marke_pick)) = option.effects.iter().find_map(|e| match e {
            EncounterEffect::Marke { card, marke } => Some((*card, *marke)),
            _ => None,
        }) else {
            return Vec::new();
        };
        let marken = self.marken(content);
        let weights = &content.run.shop.rarity_weights;
        let fits_some = |id: &CardModId| {
            self.edition
                .cards()
                .iter()
                .any(|c| c.kind != CardKind::Blank && self.marke_fits_card(content, c.id, id))
        };
        let total: u32 = marken
            .iter()
            .filter(|m| fits_some(&m.id))
            .map(|m| weights.weight(m.rarity))
            .sum();
        marken
            .into_iter()
            .map(|m| {
                let info = Run::card_mod_info(content, &m.id, None);
                let fits = fits_some(&m.id);
                let mut targets = Vec::new();
                let mut target_costs = Vec::new();
                let can_choose = match marke_pick {
                    CardPick::Chosen if !blocked => match card_pick {
                        CardPick::Chosen => {
                            for card in self.edition.cards().iter().map(|c| c.id) {
                                let action = RunAction::ChooseEncounter {
                                    option: index,
                                    target: Some(card),
                                    marke: Some(m.id.clone()),
                                };
                                if self.accepts(content, action) {
                                    targets.push(card);
                                    target_costs.push(self.option_cost(
                                        content,
                                        option,
                                        Some(card),
                                    ));
                                }
                            }
                            !targets.is_empty()
                        }
                        CardPick::Random => self.accepts(
                            content,
                            RunAction::ChooseEncounter {
                                option: index,
                                target: None,
                                marke: Some(m.id.clone()),
                            },
                        ),
                    },
                    CardPick::Chosen => false,
                    CardPick::Random => fits,
                };
                let percent = if marke_pick == CardPick::Random && fits && total > 0 {
                    weights.weight(m.rarity) * 100 / total
                } else {
                    0
                };
                MarkeChoiceView {
                    card_mod: info,
                    can_choose,
                    targets,
                    target_costs,
                    percent,
                }
            })
            .collect()
    }
}

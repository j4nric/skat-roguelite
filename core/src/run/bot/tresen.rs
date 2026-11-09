//! The bot away from the table: the Tresen and the kiosk, encounters,
//! the route choice, the boss relic, the card reward and the Schnaps of a
//! won pub.

use crate::cards::{CardKind, Rank, Suit};
use crate::content::{
    BetOn, CardModDef, CardModId, CardPick, ConsumableAction, Content, Effect, EncounterEffect,
    StopCategory,
};

use super::{
    EncounterOption, PAINTER_RESERVE, REROLL_RESERVE, RICH, Run, RunAction, RunBot, SWAP_MARGIN,
    TEETOTALLER_SKIP_WORTH, TRADE_PENALTY, consumable_value, relic_score, relic_value, zinke_score,
    zinken_value,
};

impl RunBot {
    /// The relic pick of a won boss pub (ADR 0017): the most valuable
    /// relic on offer ([`relic_value`], the first on a tie) into a free
    /// slot (always, unless the slots are capped); with every slot full in
    /// place of the weakest owned relic if it is worth at least
    /// [`SWAP_MARGIN`] more, else none.
    #[must_use]
    pub fn boss_reward_action(run: &Run, content: &Content) -> RunAction {
        let none = RunAction::ChooseBossRelic {
            offer: None,
            replace: None,
        };
        let Some(reward) = run.boss_reward() else {
            return none;
        };
        let mut best: Option<(u32, usize)> = None;
        for (i, id) in reward.relics.iter().enumerate() {
            let value = relic_score(run, content, id);
            if best.is_none_or(|(b, _)| value > b) {
                best = Some((value, i));
            }
        }
        let Some((value, offer)) = best else {
            return none;
        };
        if content.relic_slot_free(run.character(), run.relics().len()) {
            return RunAction::ChooseBossRelic {
                offer: Some(offer),
                replace: None,
            };
        }
        let weakest = run
            .relics()
            .iter()
            .enumerate()
            .min_by_key(|(i, id)| (relic_value(id), *i));
        match weakest {
            Some((index, id)) if value >= (relic_value(id) + SWAP_MARGIN) * 100 => {
                RunAction::ChooseBossRelic {
                    offer: Some(offer),
                    replace: Some(index),
                }
            }
            _ => none,
        }
    }

    /// The card reward of a won pub (DESIGN 5.0, fixed pairs since
    /// 05.11.2026): the offer worth most to the bot. An offer is worth its
    /// Zinke ([`zinke_score`]), less the Zinke it would replace; on a tie
    /// a raise before a bare card before a replace, then the card with the
    /// most Augen, Unter last (trumps anyway), then the first offer.
    /// Skipped for the Groschen if no offer is worth anything.
    #[must_use]
    pub fn card_reward_action(run: &Run, content: &Content) -> RunAction {
        let Some(reward) = run.card_reward() else {
            return RunAction::SkipCardReward;
        };
        let best = reward
            .offers
            .iter()
            .enumerate()
            .filter(|&(offer, _)| run.card_reward_open(content, offer))
            .filter_map(|(offer, o)| {
                let (_, replaces, raises) = run.mod_result(content, o.card, &o.card_mod, o.value);
                let gain = i64::from(zinke_score(run, content, &o.card_mod));
                let lost = replaces
                    .as_ref()
                    .map_or(0, |old| i64::from(zinke_score(run, content, old)));
                let fit = if raises {
                    2
                } else if replaces.is_none() {
                    1
                } else {
                    0
                };
                let (points, jack) = match run.edition().get(o.card)?.kind {
                    CardKind::Normal(card) => (card.card_points(), card.is_jack()),
                    _ => (0, false),
                };
                Some(((gain - lost, fit, !jack, points), offer))
            })
            .max_by_key(|&(key, offer)| (key, std::cmp::Reverse(offer)));
        // "Der Abstinenzler": skipping is worth +2 Grundwert for the rest
        // of the run, more than any Zinke the bot does not build on.
        let skip_worth = if run.relics().iter().any(|r| r.as_str() == "teetotaller") {
            TEETOTALLER_SKIP_WORTH
        } else {
            0
        };
        match best {
            Some(((value, ..), offer)) if value > skip_worth => RunAction::TakeCardReward { offer },
            _ => RunAction::SkipCardReward,
        }
    }

    /// The Schnaps of a won pub with every slot full, if it waits: in
    /// place of the weakest owned Schnaps (the first on a tie) if it is
    /// worth more, else sold.
    #[must_use]
    pub fn schnapps_reward_action(run: &Run) -> Option<RunAction> {
        let new = consumable_value(run.schnapps_reward()?);
        let give_up = run
            .consumables()
            .iter()
            .enumerate()
            .min_by_key(|(i, id)| (consumable_value(id), *i))
            .filter(|(_, id)| new > consumable_value(id))
            .map(|(slot, _)| slot);
        Some(RunAction::ChooseSchnappsReward { give_up })
    }

    /// The route policy, see the module docs: the way with the best
    /// score, the first on a tie.
    #[must_use]
    pub fn route_action(&self, run: &Run, content: &Content) -> RunAction {
        let Some(route) = run.route() else {
            return RunAction::LeaveShop;
        };
        if let Some(action) = Self::route_schnapps(run, content) {
            return action;
        }
        if let Some(action) = Self::malus_choice(run) {
            return action;
        }
        let cursed = Self::cursed_card(run, content).is_some();
        let angel = run.relics().iter().any(|r| {
            content
                .relic(r)
                .is_some_and(|d| d.passive.contains(&Effect::SecondChance))
        });
        let relics = run.relics().len();
        let strong = relics >= 5 || (relics >= 4 && angel);
        let mut best: Option<(i64, usize)> = None;
        for (i, option) in route.options.iter().enumerate() {
            let mut score: i64 = 0;
            for stop in &option.stops {
                score += match stop.category {
                    StopCategory::Tresen if run.coins() >= RICH => 4,
                    StopCategory::Tresen => 1,
                    StopCategory::Upgrade if cursed => 5,
                    StopCategory::Upgrade => 2,
                    StopCategory::Rest if cursed => 3,
                    StopCategory::Rest => 2,
                    StopCategory::Event => 1,
                    StopCategory::FoundCoins => 2,
                    StopCategory::Kiosk => 1,
                    StopCategory::DiceStall => 0,
                };
            }
            if option.spelunke.is_some() {
                score += if strong { 3 } else { -20 };
            }
            if best.is_none_or(|(b, _)| score > b) {
                best = Some((score, i));
            }
        }
        RunAction::ChooseRoute(best.map_or(0, |(_, i)| i))
    }

    /// The encounter policy, see the module docs: the option with the best
    /// value ([`RunBot::option_value`]) among those that can be taken, the
    /// first on a tie. One is always taken.
    #[must_use]
    pub fn encounter_action(&self, run: &Run, content: &Content) -> RunAction {
        if let Some(action) = Self::paint_choice(run, content) {
            return action;
        }
        let Some(encounter) = run.encounter() else {
            return RunAction::LeaveShop;
        };
        let mut best: Option<(i64, RunAction)> = None;
        for (i, option) in encounter.options.iter().enumerate() {
            if run.encounter_option_block(content, i).is_some() {
                continue;
            }
            let Some((target, marke)) = Self::option_choice(run, content, i, option) else {
                continue;
            };
            let value = Self::option_value(run, content, option);
            if best.as_ref().is_none_or(|(v, _)| value > *v) {
                best = Some((
                    value,
                    RunAction::ChooseEncounter {
                        option: i,
                        target,
                        marke,
                    },
                ));
            }
        }
        best.map_or(
            RunAction::ChooseEncounter {
                option: 0,
                target: None,
                marke: None,
            },
            |(_, action)| action,
        )
    }

    /// Target and Marke the bot uses for an option, `None` if none is
    /// accepted. With a Marke to choose (04.11.2026), the most valuable
    /// one by [`zinken_value`] (the Pfandmarke, then the first by id)
    /// that some target accepts, on the target [`RunBot::option_target`]
    /// would pick for it.
    #[allow(clippy::type_complexity)]
    fn option_choice(
        run: &Run,
        content: &Content,
        index: usize,
        option: &EncounterOption,
    ) -> Option<(Option<u32>, Option<CardModId>)> {
        if !option.needs_marke() {
            return Self::option_target(run, content, index, option, None).map(|t| (t, None));
        }
        let mut marken: Vec<CardModId> = run
            .marken(content)
            .into_iter()
            .map(|m| m.id.clone())
            .collect();
        marken.sort_by_key(|m| std::cmp::Reverse(zinken_value(m)));
        marken.into_iter().find_map(|marke| {
            Self::option_target(run, content, index, option, Some(&marke)).map(|t| (t, Some(marke)))
        })
    }

    /// The target the bot uses for an option: `Some(None)` without one,
    /// `None` if no target is accepted. Cleansing aims at a cursed card, a
    /// curse at a low card without modification, everything else at an
    /// ace; a relic option at the weakest relic.
    #[allow(clippy::option_option)]
    fn option_target(
        run: &Run,
        content: &Content,
        index: usize,
        option: &EncounterOption,
        marke: Option<&CardModId>,
    ) -> Option<Option<u32>> {
        let accepted = |target: u32| {
            run.accepts(
                content,
                RunAction::ChooseEncounter {
                    option: index,
                    target: Some(target),
                    marke: marke.cloned(),
                },
            )
        };
        if run.option_needs_relic(content, option) {
            let weakest = run
                .relics()
                .iter()
                .enumerate()
                .min_by_key(|(i, id)| (relic_value(id), *i))
                .and_then(|(i, _)| u32::try_from(i).ok())?;
            return accepted(weakest).then_some(Some(weakest));
        }
        if !option.needs_card() {
            let ok = run.accepts(
                content,
                RunAction::ChooseEncounter {
                    option: index,
                    target: None,
                    marke: marke.cloned(),
                },
            );
            return (ok || marke.is_none()).then_some(None);
        }
        let cleanse = option.effects.contains(&EncounterEffect::Cleanse);
        let curse = option.effects.iter().any(|e| match e {
            EncounterEffect::Mark {
                card: CardPick::Chosen,
                card_mod: Some(m),
                ..
            } => content.card_mod(m).is_some_and(CardModDef::is_curse),
            _ => false,
        });
        let mut candidates: Vec<u32> = if cleanse {
            Self::cursed_card(run, content).into_iter().collect()
        } else if curse {
            Self::low_targets(run)
        } else {
            Self::ace_targets(run)
        };
        if !cleanse {
            candidates.extend(run.edition().cards().iter().map(|c| c.id));
        }
        candidates.into_iter().find(|&c| accepted(c)).map(Some)
    }

    /// Deck ids of sevens, eights and nines without modification.
    fn low_targets(run: &Run) -> Vec<u32> {
        run.edition()
            .cards()
            .iter()
            .filter(|c| c.mods.is_empty())
            .filter_map(|c| match c.kind {
                CardKind::Normal(card)
                    if matches!(card.rank, Rank::Seven | Rank::Eight | Rank::Nine) =>
                {
                    Some(c.id)
                }
                _ => None,
            })
            .collect()
    }

    /// The bot's value of an encounter option, in rough Groschen: what its
    /// effects bring minus what it costs. Simple rules, not tuned: Groschen
    /// count as they are, a house rule or relic as [`relic_value`] + 1, a
    /// Schnaps as [`consumable_value`] + 1 (its price with every slot full),
    /// a life of an old save its Groschen, a Zinke 1 on a random and 2 on a
    /// chosen card, a curse -2, a swap the gain in value when it is at
    /// least [`SWAP_MARGIN`], higher stakes -1 (the bot never seeks them),
    /// a bet its expected payout (Schneider a quarter of the time, a win
    /// three in five), a throw of the dice its expected payout.
    #[must_use]
    pub fn option_value(run: &Run, content: &Content, option: &EncounterOption) -> i64 {
        let free_slot = content.relic_slot_free(run.character(), run.relics().len());
        let item_slot =
            run.consumables().len() < usize::try_from(content.run.consumable_slots).unwrap_or(0);
        let weakest = run.relics().iter().map(relic_value).min().unwrap_or(0);
        let stake = i64::from(option.cost);
        let mut value: i64 = -stake;
        for effect in &option.effects {
            value += match effect {
                EncounterEffect::Coins(n) => i64::from(*n),
                EncounterEffect::EveningRule(Some(id)) => i64::from(relic_value(id)) + 1,
                EncounterEffect::Relic(Some(id)) if free_slot => i64::from(relic_value(id)) + 1,
                // Every slot full: in place of the weakest, only if
                // clearly better (like a swap).
                EncounterEffect::Relic(Some(new)) | EncounterEffect::SwapRelic(Some(new)) => {
                    let gain = i64::from(relic_value(new)) - i64::from(weakest);
                    if gain >= i64::from(SWAP_MARGIN) {
                        gain
                    } else {
                        -1
                    }
                }
                EncounterEffect::Mark {
                    card,
                    card_mod: Some(m),
                    curse_percent,
                    ..
                } => {
                    let good = match card {
                        CardPick::Random => 1,
                        CardPick::Chosen => 2,
                    };
                    let base = if content.card_mod(m).is_some_and(CardModDef::is_curse) {
                        -2
                    } else {
                        good
                    };
                    let p = i64::from(*curse_percent);
                    (base * (100 - p) - 2 * p) / 100
                }
                // A Marke like a Zinke: 1 on a random, 2 on a chosen card.
                EncounterEffect::Marke { card, .. } => match card {
                    CardPick::Random => 1,
                    CardPick::Chosen => 2,
                },
                EncounterEffect::Schnapps(Some(id)) => {
                    if item_slot {
                        i64::from(consumable_value(id)) + 1
                    } else {
                        content.consumable(id).map_or(0, |c| i64::from(c.price))
                    }
                }
                EncounterEffect::Lives(n) => {
                    i64::from(*n) * i64::from(crate::content::LEGACY_LIFE_COINS)
                }
                EncounterEffect::NextPubRedeals(n) => 2 * i64::from(*n),
                EncounterEffect::SeeSkat | EncounterEffect::Scout(_) => 1,
                EncounterEffect::NextGameFactor(_) => -1,
                EncounterEffect::Bet { on, percent } => {
                    let chance = match on {
                        BetOn::Schneider => 25,
                        BetOn::Win => 60,
                    };
                    stake * i64::from(*percent) * chance / 10_000
                }
                EncounterEffect::Dice { chance, percent } => {
                    stake * i64::from(*percent) * i64::from(*chance) / 10_000
                }
                EncounterEffect::Painter => 3,
                EncounterEffect::Cleanse => 5,
                _ => 0,
            };
        }
        value
    }

    /// At a Kiosk: the most valuable Schnaps ([`consumable_value`]) if it
    /// can be bought, else on.
    fn kiosk_action(run: &Run, content: &Content) -> RunAction {
        let Some(shop) = run.shop() else {
            return RunAction::LeaveShop;
        };
        shop.consumables
            .iter()
            .enumerate()
            .filter_map(|(slot, id)| Some((slot, consumable_value(id.as_ref()?))))
            .filter(|&(_, value)| value > 0)
            .filter(|&(slot, _)| run.accepts(content, RunAction::BuyConsumable(slot)))
            .max_by_key(|&(slot, value)| (value, std::cmp::Reverse(slot)))
            .map_or(RunAction::LeaveShop, |(slot, _)| {
                RunAction::BuyConsumable(slot)
            })
    }

    /// Slot of an owned consumable with this action, if usable now.
    pub(super) fn usable(
        run: &Run,
        content: &Content,
        wanted: impl Fn(&ConsumableAction) -> bool,
        target: Option<u32>,
    ) -> Option<RunAction> {
        run.consumables()
            .iter()
            .enumerate()
            .filter(|(_, id)| content.consumable(id).is_some_and(|d| wanted(&d.action)))
            .map(|(slot, _)| RunAction::UseConsumable {
                slot,
                target,
                suit: None,
            })
            .find(|action| run.accepts(content, action.clone()))
    }

    /// The Tresen policy: one action per call, see the module docs.
    #[must_use]
    pub fn shop_action(&self, run: &Run, content: &Content) -> RunAction {
        let Some(shop) = run.shop() else {
            return RunAction::LeaveShop;
        };
        if let Some(action) = Self::paint_choice(run, content) {
            return action;
        }
        if shop.kiosk {
            return Self::kiosk_action(run, content);
        }
        let coins = run.coins();
        let price = |p: u32| run.shop_price(content, p);
        let cursed = Self::cursed_card(run, content);
        let likes_curses = Self::likes_curses(run, content);
        let zinken = &content.run.shop.zinken;
        // 1. "Karte reinigen" on a cursed card (ADR 0015).
        if let Some(target) = cursed {
            let action = RunAction::CleanseCard(target);
            if price(zinken.cleanse_price) <= coins && run.accepts(content, action.clone()) {
                return action;
            }
        }
        let free_slot = content.relic_slot_free(run.character(), run.relics().len());
        // Relic offers the bot wants: (value, price, slot).
        let offers: Vec<(u32, u32, usize)> = shop
            .relics
            .iter()
            .enumerate()
            .filter_map(|(slot, id)| {
                let id = id.as_ref()?;
                let def = content.relic(id)?;
                let score = relic_score(run, content, id);
                let value = if run.is_fence(slot) && !likes_curses {
                    score.saturating_sub(TRADE_PENALTY * 100)
                } else {
                    score
                };
                Some((value, run.relic_offer_cost(content, slot, def.price), slot))
            })
            .filter(|&(value, _, _)| value > 0)
            .collect();
        // 2. All slots full (8 since 05.11.2026): the most valuable offer
        //    in place of the weakest relic, if it is clearly better
        //    ([`SWAP_MARGIN`]) and affordable with the weakest sold.
        if !free_slot {
            let weakest = run
                .relics()
                .iter()
                .enumerate()
                .map(|(index, id)| {
                    let refund = content
                        .relic(id)
                        .map_or(0, |d| super::super::sell_price(d.price));
                    (relic_value(id) * 100, index, refund)
                })
                .min_by_key(|&(value, index, _)| (value, std::cmp::Reverse(index)));
            if let Some((weak_value, replace, refund)) = weakest {
                let best = offers
                    .iter()
                    .filter(|&&(value, price, _)| {
                        value >= weak_value + SWAP_MARGIN * 100 && price <= coins + refund
                    })
                    .max_by_key(|&&(value, price, slot)| (value, price, std::cmp::Reverse(slot)));
                if let Some(&(_, _, slot)) = best {
                    let action = RunAction::ReplaceRelic { slot, replace };
                    if run.accepts(content, action.clone()) {
                        return action;
                    }
                }
            }
        }
        // 3. Buy the most valuable relic.
        if free_slot {
            let best = offers
                .iter()
                .filter(|&&(_, price, _)| price <= coins)
                .max_by_key(|&&(value, price, slot)| (value, price, std::cmp::Reverse(slot)));
            if let Some(&(_, _, slot)) = best {
                return RunAction::BuyRelic(slot);
            }
        }
        // 4. Buy the most valuable consumable.
        let consumable_slot =
            run.consumables().len() < usize::try_from(content.run.consumable_slots).unwrap_or(0);
        if consumable_slot {
            let best = shop
                .consumables
                .iter()
                .enumerate()
                .filter_map(|(slot, id)| {
                    let id = id.as_ref()?;
                    let def = content.consumable(id)?;
                    let value = consumable_value(id);
                    let cost = price(def.price);
                    (value > 0 && cost <= coins).then_some((value, cost, slot))
                })
                .max_by_key(|&(value, price, slot)| (value, price, std::cmp::Reverse(slot)));
            if let Some((_, _, slot)) = best {
                return RunAction::BuyConsumable(slot);
            }
        }
        // 5. The Auslage: the best Zinke on a card that carries nothing
        // else (the same one raised is fine), never replacing.
        let best = shop
            .card_offers
            .iter()
            .enumerate()
            .filter_map(|(slot, offer)| {
                let offer = offer.as_ref()?;
                let card = run.edition().get(offer.card)?;
                let bare = card.mods.iter().all(|m| *m == offer.card_mod);
                let score = zinke_score(run, content, &offer.card_mod);
                let value = if offer.bargain.is_some() && !likes_curses {
                    score.saturating_sub(TRADE_PENALTY * 100)
                } else {
                    score
                };
                let cost = run.card_offer_cost(content, offer);
                (bare && value > 0 && cost <= coins).then_some((value, cost, slot))
            })
            .max_by_key(|&(value, cost, slot)| {
                (value, std::cmp::Reverse(cost), std::cmp::Reverse(slot))
            });
        if let Some((_, _, slot)) = best {
            let action = RunAction::BuyCardOffer(slot);
            if run.accepts(content, action.clone()) {
                return action;
            }
        }
        // 6. The Kartenmaler for an ace without modification, if Groschen
        // are plenty.
        let bare_ace = Self::ace_targets(run).into_iter().find(|&card| {
            run.edition().get(card).is_some_and(|c| c.mods.is_empty())
                && coins >= run.painter_cost(content, card) + PAINTER_RESERVE
                && run.accepts(content, RunAction::HirePainter(card))
        });
        if let Some(card) = bare_ace {
            return RunAction::HirePainter(card);
        }
        // 7. Nothing sensible to buy: "Neu auslosen" (once per visit), if
        // Groschen are plenty.
        let cost = run.reroll_cost(content);
        if shop.rerolls == 0
            && coins >= cost + REROLL_RESERVE
            && run.accepts(content, RunAction::Reroll)
        {
            return RunAction::Reroll;
        }
        RunAction::LeaveShop
    }

    /// The Kartenmaler waits: take his most valuable modification, the
    /// first on a tie.
    fn paint_choice(run: &Run, content: &Content) -> Option<RunAction> {
        let painter = run.painter()?;
        let best = painter
            .options
            .iter()
            .enumerate()
            .max_by_key(|(i, (m, _))| (zinke_score(run, content, m), std::cmp::Reverse(*i)))
            .map_or(0, |(i, _)| i);
        Some(RunAction::ChoosePaint(best))
    }

    /// The first card of the player's edition that carries a curse ("Abgestuft"
    /// stays: it is the Nullspieler's start edition).
    fn cursed_card(run: &Run, content: &Content) -> Option<u32> {
        // "Bruderherz" cannot cleanse his curses: nothing to aim for.
        // With "Pechvogel" a curse is worth Grundwert.
        if content
            .character(run.character())
            .is_some_and(|c| c.rule.keep_curses)
            || Self::likes_curses(run, content)
        {
            return None;
        }
        run.edition()
            .cards()
            .iter()
            .find(|c| {
                c.mods.iter().any(|m| {
                    content
                        .card_mod(m)
                        .is_some_and(|d| d.kind == crate::content::ModKind::Curse)
                })
            })
            .map(|c| c.id)
    }

    /// Whether the bot owns a relic that pays for cursed cards
    /// ("Pechvogel"): then it keeps its curses and takes trade offers at
    /// full value.
    fn likes_curses(run: &Run, content: &Content) -> bool {
        run.relics()
            .iter()
            .filter_map(|r| content.relic(r))
            .flat_map(|d| {
                d.passive
                    .iter()
                    .chain(d.hooks.iter().flat_map(|h| h.then()))
            })
            .any(|e| matches!(e, Effect::AddBaseValuePerCursedCard(n) if *n > 0))
    }

    /// Deck ids of the aces, Kreuz first, the least modified first.
    fn ace_targets(run: &Run) -> Vec<u32> {
        let mut aces: Vec<(usize, Suit, u32)> = run
            .edition()
            .cards()
            .iter()
            .filter_map(|c| match c.kind {
                CardKind::Normal(card) if card.rank == Rank::Ace => {
                    Some((c.mods.len(), card.suit, c.id))
                }
                _ => None,
            })
            .collect();
        aces.sort_by_key(|&(mods, suit, id)| (mods, std::cmp::Reverse(suit), id));
        aces.into_iter().map(|(_, _, id)| id).collect()
    }

    /// Schnäpse at the route choice: "Wegbier" at once, "Schwenker" when
    /// no way has what the bot wants most.
    fn route_schnapps(run: &Run, content: &Content) -> Option<RunAction> {
        if let Some(action) = Self::usable(
            run,
            content,
            |a| matches!(a, ConsumableAction::NextPub { .. }),
            None,
        ) {
            return Some(action);
        }
        let route = run.route()?;
        let has = |category: StopCategory| {
            route
                .options
                .iter()
                .any(|o| o.stops.iter().any(|s| s.category == category))
        };
        let wants_upgrade = Self::cursed_card(run, content).is_some()
            && !has(StopCategory::Upgrade)
            && !has(StopCategory::Rest);
        if wants_upgrade {
            return Self::usable(
                run,
                content,
                |a| matches!(a, ConsumableAction::RerollRoutes),
                None,
            );
        }
        None
    }

    /// "Spelunkenkenner": the malus the bot likes best on a Spelunke way,
    /// if it is not the chosen one yet (Schnapsverbot, then Klebrige
    /// Karten, Kein Einmischen, Kein Skat).
    fn malus_choice(run: &Run) -> Option<RunAction> {
        const LIKED: [&str; 4] = ["schnapps_locked", "cursed_cards", "no_redeal", "hand_only"];
        let route = run.route()?;
        route.options.iter().enumerate().find_map(|(option, o)| {
            let spelunke = o.spelunke.as_ref()?;
            let best = spelunke
                .malus_options
                .iter()
                .enumerate()
                .min_by_key(|(_, m)| LIKED.iter().position(|l| *l == m.as_str()).unwrap_or(9))?;
            (*best.1 != spelunke.malus).then_some(RunAction::ChooseMalus {
                option,
                malus: best.0,
            })
        })
    }
}

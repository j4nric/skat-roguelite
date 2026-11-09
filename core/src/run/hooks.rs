//! Content hooks: which sources fire at a [`Stage`] in which order
//! (character, boss, Spelunke, relics, house rules, card modifications)
//! and what each [`Effect`] does to the run when it fires.

use std::collections::{BTreeMap, BTreeSet};

use crate::cards::PlayingCard;
use crate::content::{
    ActiveSource, BossId, Content, Effect, EffectKind, Hook, HookContext, ScoreLine, Source, Stage,
    card_mod_sources, evaluate, in_pool,
};
use crate::deck::DealtCard;
use crate::game::{Phase, Seat};
use crate::rules::ValueParts;

use super::{Run, RunEvent, tour};

/// The line a score effect (Grundwert, Gewinnstufen, Spielwert) adds, with
/// the run counters as they are now and `cards` the cards an effect counts
/// in the player's edition ([`Run::count_for`]: cursed cards, or cards of a
/// [`crate::content::CardCount`]). `None` for every other effect and for a counter (or
/// count of cards) that is still zero.
pub(super) fn score_line(
    source: &Source,
    effect: &Effect,
    counters: &BTreeMap<String, u32>,
    cards: u32,
) -> Option<ScoreLine> {
    let (lever, change) = effect.score_change(counters, cards)?;
    Some(ScoreLine {
        source: source.clone(),
        lever,
        change,
    })
}

impl Run {
    /// The boss whose rule is on in the current game: the series' boss, or
    /// for a boss with a rotation (Endboss "Der Alte") the rule of this
    /// game, by the number of games dealt in the series. Bosses not in the
    /// character's pool are skipped.
    pub(super) fn boss_rule(&self, content: &Content) -> Option<BossId> {
        let def = content.boss(self.series.boss.as_ref()?)?;
        if def.rotation.is_empty() {
            return Some(def.id.clone());
        }
        let rules: Vec<&BossId> = def
            .rotation
            .iter()
            .filter(|id| {
                content
                    .boss(id)
                    .is_some_and(|b| in_pool(&b.characters, self.character))
            })
            .collect();
        let n = u32::try_from(rules.len()).ok().filter(|&n| n > 0)?;
        // Every deal counts, also one dealt again (Einmischen), as passes
        // did before.
        let deals = self
            .series
            .games_played
            .saturating_add(self.series.redeals_used);
        let i = usize::try_from(deals % n).unwrap_or(0);
        rules.get(i).map(|id| (*id).clone())
    }

    /// The character's own rule, boss, relics and house rules of the
    /// evening, plus the card modifications in play if a game runs.
    pub(super) fn sources<'c>(
        &self,
        content: &'c Content,
        with_card_mods: bool,
    ) -> Vec<ActiveSource<'c>> {
        let mut sources = Vec::new();
        if let Some(def) = content.character(self.character) {
            let rule = &def.rule;
            if !rule.passive.is_empty() || !rule.hooks.is_empty() {
                sources.push(ActiveSource {
                    source: Source::Character(self.character),
                    passive: &rule.passive,
                    hooks: &rule.hooks,
                    this_card: None,
                    scale: 1,
                });
            }
        }
        let bosses = self
            .boss_rule(content)
            .into_iter()
            .chain(self.extra_boss_rules().iter().cloned());
        for def in bosses.filter_map(|b| content.boss(&b)) {
            sources.push(ActiveSource {
                source: Source::Boss(def.id.clone()),
                passive: &def.passive,
                hooks: &def.hooks,
                this_card: None,
                scale: 1,
            });
        }
        if let Some(def) = self.malus(content) {
            sources.push(ActiveSource {
                source: Source::Malus(def.id.clone()),
                passive: &def.passive,
                hooks: &[],
                this_card: None,
                scale: 1,
            });
        }
        for def in self.held_relic_defs(content) {
            sources.push(ActiveSource {
                source: Source::Relic(def.id.clone()),
                passive: &def.passive,
                hooks: &def.hooks,
                this_card: None,
                scale: 1,
            });
        }
        if with_card_mods {
            if let Some(active) = &self.game {
                sources.extend(card_mod_sources(self.editions(), content, &active.game));
            }
        }
        sources
    }

    /// Whether any of [`Run::sources`] has a hook of `stage`, without
    /// building them.
    fn has_hook(&self, content: &Content, stage: Stage, with_card_mods: bool) -> bool {
        let of = |hooks: &[Hook]| hooks.iter().any(|h| h.stage() == stage);
        if content
            .character(self.character)
            .is_some_and(|def| of(&def.rule.hooks))
        {
            return true;
        }
        let bosses = self.boss_rule(content);
        let extra = self.extra_boss_rules();
        if bosses
            .iter()
            .chain(extra)
            .filter_map(|b| content.boss(b))
            .any(|def| of(&def.hooks))
        {
            return true;
        }
        if self.held_relic_defs(content).any(|def| of(&def.hooks)) {
            return true;
        }
        if !with_card_mods {
            return false;
        }
        let Some(active) = &self.game else {
            return false;
        };
        let editions = self.editions();
        active.game.cards_in_play().iter().any(|card| {
            !card.is_blank()
                && editions
                    .of(card.owner)
                    .and_then(|e| e.get(card.id))
                    .is_some_and(|deck_card| {
                        deck_card
                            .mods
                            .iter()
                            .filter_map(|m| content.card_mod(m))
                            .any(|def| of(&def.hooks))
                    })
        })
    }

    /// Fire every hook of `stage` whose condition holds.
    pub(super) fn fire(
        &mut self,
        content: &Content,
        stage: Stage,
        played: Option<(Seat, DealtCard)>,
        result: Option<&ValueParts>,
        events: &mut Vec<RunEvent>,
    ) {
        self.fire_with(content, stage, played, result, None, events);
    }

    /// [`Run::fire`] with the means of transport of the route just taken
    /// (on route chosen).
    pub(super) fn fire_with(
        &mut self,
        content: &Content,
        stage: Stage,
        played: Option<(Seat, DealtCard)>,
        result: Option<&ValueParts>,
        transport: Option<tour::Transport>,
        events: &mut Vec<RunEvent>,
    ) {
        let with_card_mods = stage.in_game();
        if !self.has_hook(content, stage, with_card_mods) {
            return;
        }
        let games_left = self.series.games_left();
        let sources = self.sources(content, with_card_mods);
        // Only conditions on marked cards look at them.
        let marked = if sources
            .iter()
            .flat_map(|s| s.hooks)
            .any(|h| h.stage() == stage && h.when().uses_marked_cards())
        {
            self.marked_cards(content)
        } else {
            BTreeSet::new()
        };
        for source in &sources {
            for hook in source.hooks.iter().filter(|h| h.stage() == stage) {
                let holds = {
                    let ctx = HookContext {
                        flags: &self.flags,
                        counters: &self.counters,
                        game: self.game.as_ref().map(|a| &a.game),
                        played,
                        result,
                        games_left,
                        transport,
                        marked_cards: Some(&marked),
                        run: Some((self, content)),
                    };
                    evaluate(hook.when(), &ctx, source.this_card)
                };
                if holds {
                    for effect in hook.then() {
                        self.apply_effect(
                            content,
                            &source.source,
                            &effect.scaled(source.scale),
                            events,
                        );
                    }
                }
            }
        }
    }

    /// Apply one effect. Card-level effects were applied when the deck was
    /// resolved and are ignored here.
    pub(super) fn apply_effect(
        &mut self,
        content: &Content,
        source: &Source,
        effect: &Effect,
        events: &mut Vec<RunEvent>,
    ) {
        match effect {
            Effect::SetFlag(flag) => {
                if self.flags.insert(flag.clone()) {
                    events.push(RunEvent::FlagSet(flag.clone()));
                }
                return;
            }
            Effect::ClearFlag(flag) => {
                if self.flags.remove(flag) {
                    events.push(RunEvent::FlagCleared(flag.clone()));
                }
                return;
            }
            Effect::IncrementCounter(counter) => {
                let value = self.counters.entry(counter.clone()).or_insert(0);
                *value = value.saturating_add(1);
                events.push(RunEvent::CounterRaised {
                    counter: counter.clone(),
                    value: *value,
                });
                return;
            }
            Effect::ResetCounter(counter) => {
                if self.counters.remove(counter).is_some_and(|v| v > 0) {
                    events.push(RunEvent::CounterReset {
                        counter: counter.clone(),
                    });
                }
                return;
            }
            // Needs the content: applied by `redeem_vouchers`.
            Effect::TurnInto(_) => return,
            Effect::AttachModToRandomCard {
                card_mod,
                value,
                only_unmarked,
            } => {
                self.attach_to_random_card(content, card_mod, *value, *only_unmarked, events);
                return;
            }
            Effect::GainCoins(amount) => {
                if *amount != 0 {
                    self.coins = if *amount > 0 {
                        self.coins.saturating_add(amount.unsigned_abs())
                    } else {
                        self.coins.saturating_sub(amount.unsigned_abs())
                    };
                    events.push(RunEvent::CoinsGained {
                        source: source.clone(),
                        amount: *amount,
                        coins: self.coins,
                    });
                    self.settle_debt(events);
                }
                return;
            }
            Effect::StashCoins { counter, amount } => {
                let moved = (*amount).min(self.coins);
                if moved > 0 {
                    self.coins -= moved;
                    let pot = self.counters.entry(counter.clone()).or_insert(0);
                    *pot = pot.saturating_add(moved);
                    let pot = *pot;
                    events.push(RunEvent::CoinsStashed {
                        source: source.clone(),
                        amount: moved,
                        pot,
                        coins: self.coins,
                    });
                }
                return;
            }
            Effect::PayOutCounter { counter, percent } => {
                let pot = self.counters.remove(counter).unwrap_or(0);
                if pot > 0 {
                    events.push(RunEvent::CounterReset {
                        counter: counter.clone(),
                    });
                    let paid = u32::try_from(u64::from(pot) * u64::from(*percent) / 100)
                        .unwrap_or(u32::MAX);
                    if paid > 0 {
                        self.coins = self.coins.saturating_add(paid);
                        events.push(RunEvent::CoinsGained {
                            source: source.clone(),
                            amount: i32::try_from(paid).unwrap_or(i32::MAX),
                            coins: self.coins,
                        });
                        self.settle_debt(events);
                    }
                }
                return;
            }
            _ => {}
        }
        let line = if effect.kind() == EffectKind::ScoreLine {
            score_line(
                source,
                effect,
                &self.counters,
                self.count_for(content, effect),
            )
        } else {
            None
        };
        let Some(active) = self.game.as_mut() else {
            return;
        };
        if effect.is_cheat() && matches!(source, Source::Boss(_)) && active.rules.no_cheating {
            return;
        }
        if effect.kind() == EffectKind::ScoreLine {
            active.lines.extend(line);
            return;
        }
        match effect {
            Effect::AddTrickPoints(extra) => {
                if let Some(trick) = active.game.tricks().last() {
                    let winner = trick.winner;
                    if *extra > 0 {
                        active.game.add_bonus_points(winner, *extra);
                        events.push(RunEvent::BonusPoints {
                            source: source.clone(),
                            seat: winner,
                            card_points: *extra,
                        });
                    }
                }
            }
            Effect::MultiplyTrickPoints(factor) => {
                if let Some(trick) = active.game.tricks().last() {
                    let points = u32::try_from(trick.signed_points().max(0)).unwrap_or(0);
                    let extra = points.saturating_mul(factor.saturating_sub(1));
                    if extra > 0 {
                        let winner = trick.winner;
                        active.game.add_bonus_points(winner, extra);
                        events.push(RunEvent::BonusPoints {
                            source: source.clone(),
                            seat: winner,
                            card_points: extra,
                        });
                    }
                }
            }
            Effect::MultiplySkatPoints(factor) => {
                let factor = *factor;
                let scaled = |c: DealtCard| DealtCard {
                    card_points: c.card_points.saturating_mul(factor),
                    ..c
                };
                // Nothing in a hand game: no card was discarded.
                if let Ok(old) = active.game.map_skat(scaled) {
                    let mut added: i64 = 0;
                    for card in old {
                        let delta =
                            i64::from(scaled(card).card_points) - i64::from(card.card_points);
                        if delta != 0 {
                            let delta = i32::try_from(delta).unwrap_or(0);
                            active.point_changes.push((card.id, source.clone(), delta));
                            added += i64::from(delta);
                        }
                    }
                    if added != 0 {
                        events.push(RunEvent::SkatPointsRaised {
                            source: source.clone(),
                            card_points: i32::try_from(added).unwrap_or(0),
                        });
                    }
                }
            }
            Effect::SchneiderBonusFromTrick => {
                let points = active.game.tricks().last().map_or(0, |t| t.card_points());
                if points > 0 {
                    let thresholds = &mut active.rules.thresholds;
                    thresholds.schneider_bonus = thresholds.schneider_bonus.saturating_add(points);
                    active
                        .rules
                        .schneider_bonus_source
                        .get_or_insert_with(|| source.clone());
                }
            }
            Effect::WinThreshold(n) => {
                if active.rules.thresholds.lower_win(*n) {
                    active.rules.win_threshold_source = Some(source.clone());
                }
            }
            Effect::SchneiderFromDeclarerPoints { min, max } => {
                active
                    .rules
                    .thresholds
                    .declarer_schneider
                    .push((*min, *max));
                active
                    .rules
                    .schneider_sources
                    .push((source.clone(), (*min, *max)));
            }
            Effect::RequireHand => active.rules.require_hand = true,
            Effect::EnableKontra => {
                if active.rules.kontra_source.is_none() {
                    active.rules.kontra_source = Some(source.clone());
                }
            }
            Effect::ExtraRedeal => {
                active.rules.extra_redeals = active.rules.extra_redeals.saturating_add(1);
            }
            Effect::Concede => {
                if active.game.phase() == Phase::Playing && active.game.current_trick().is_empty() {
                    if let Ok(game_events) = active.game.concede() {
                        events.extend(game_events.into_iter().map(RunEvent::Game));
                    }
                }
            }
            Effect::PeekDefenderCards(n) => {
                active.rules.peek = active.rules.peek.saturating_add(*n);
            }
            Effect::RevealDefenderCards(n) => {
                active.rules.transparent_per_defender =
                    active.rules.transparent_per_defender.saturating_add(*n);
                active
                    .rules
                    .transparent_source
                    .get_or_insert_with(|| source.clone());
            }
            Effect::ShowCount => active.rules.show_count = true,
            Effect::RevealDefenderHand(index) => {
                let seat = Seat::from_index(usize::from(*index)).filter(|s| !s.is_declarer());
                if let Some(seat) = seat {
                    active.shown_hand.get_or_insert(seat);
                    if !active.shown_hands.contains(&seat) {
                        active.shown_hands.push(seat);
                    }
                    for &card in active.game.hand(seat) {
                        if !active.peeked.iter().any(|(c, _)| c.id == card.id) {
                            active.peeked.push((card, seat));
                            events.push(RunEvent::CardPeeked { card, seat });
                        }
                    }
                }
            }
            Effect::SeriesSkatSwap(filter) => active.rules.skat_swaps.push(*filter),
            Effect::DiscardThree => {
                active
                    .rules
                    .discard_three
                    .get_or_insert_with(|| source.clone());
            }
            Effect::RevealSkatCard(n) => {
                let untouched = active
                    .game
                    .declaration()
                    .is_some_and(|d| d.hand() && !d.is_ramsch());
                if untouched {
                    let fresh: Vec<DealtCard> = active
                        .game
                        .skat()
                        .into_iter()
                        .filter(|c| !active.skat_shown.iter().any(|s| s.id == c.id))
                        .take(usize::try_from(*n).unwrap_or(usize::MAX))
                        .collect();
                    for card in fresh {
                        active.skat_shown.push(card);
                        events.push(RunEvent::SkatCardShown {
                            source: source.clone(),
                            card,
                        });
                    }
                }
            }
            Effect::NoCheating => active.rules.no_cheating = true,
            Effect::GrandAcesAsMatadors => active.game.count_grand_aces(),
            Effect::OuvertRequiresSchneider => active.rules.thresholds.ouvert_schneider = true,
            Effect::RequireOuvert => active.rules.require_ouvert = true,
            Effect::DefendersSeeEachOther => active.game.open_defender_hands(),
            Effect::Ramsch => active.rules.ramsch = true,
            Effect::SingleLoss => {
                active.single_loss.get_or_insert_with(|| source.clone());
            }
            // Card-level effects (applied when the deck is resolved) and
            // effects of the run outside the game score ([`Effect::kind`]).
            _ => {}
        }
    }
}

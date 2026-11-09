//! The end of a game and of a pub: the score of a finished game with its
//! lines, the pub's account, winning or losing the pub, and what follows
//! (payout, card and Schnaps reward, the route choice).

use std::collections::BTreeMap;

use crate::content::{
    BetOn, BossId, Change, Character, Content, Effect, Lever, ScoreLine, Source, Stage, score_parts,
};
use crate::deck::{DealtCard, Owner};
use crate::game::{Game, Phase, Seat};
use crate::rules::{
    Declaration, NullScoring, Thresholds, game_value_parts, null_value_parts, ramsch_value_parts,
};

use super::{ActiveGame, Kontra, PUBS_PER_DISTRICT, Payout, Run, RunError, RunEvent, RunPhase};

/// Zielwert of a pub: the character's value (from `characters.ron`, or
/// `run.ron` if the character has none), for a boss pub times the boss's
/// target factor (a Spelunke's factor comes on top, [`Run`] applies it).
/// `section` is the district, `index` the pub within it. `None` if the
/// row or pub does not exist.
#[must_use]
pub fn series_target(
    content: &Content,
    character: Character,
    section: u32,
    index: u32,
    boss: Option<&BossId>,
) -> Option<u64> {
    let curve = content
        .character(character)
        .and_then(|c| c.targets.as_ref())
        .unwrap_or(&content.run.targets);
    let targets = curve.get(usize::try_from(section).unwrap_or(usize::MAX))?;
    let base = *targets.get(usize::try_from(index).unwrap_or(usize::MAX))?;
    let factor = boss
        .and_then(|b| content.boss(b))
        .map_or(crate::content::Factor::ONE, |def| {
            def.target_factor_for(character)
        });
    Some(factor.apply(base))
}

/// The ten cards the player plays in `game`, as he holds them: his hand
/// and the cards he played (completed tricks and the trick on the table),
/// never the skat.
pub(super) fn player_cards(game: &Game<DealtCard>) -> Vec<DealtCard> {
    let mut cards: Vec<DealtCard> = game.hand(Seat::DECLARER).to_vec();
    for trick in game.tricks() {
        for i in 0..3 {
            if trick.seat_of(i).is_declarer() {
                cards.push(trick.cards[i]);
            }
        }
    }
    let lead = game.trick_lead();
    for (i, card) in game.current_trick().iter().enumerate() {
        if crate::game::seat_at(lead, i).is_declarer() {
            cards.push(*card);
        }
    }
    cards
}

/// The Spielwert factors of a won game ("Eierlikör").
pub(super) fn win_factor_lines(active: &ActiveGame) -> impl Iterator<Item = ScoreLine> + '_ {
    active.win_factors.iter().map(|(source, factor)| ScoreLine {
        source: source.clone(),
        lever: Lever::GameValue,
        change: Change::Multiply(*factor),
    })
}

/// The Spielwert factors of Kontra and Re.
pub(super) fn kontra_lines(kontra: Kontra) -> Vec<ScoreLine> {
    match kontra {
        Kontra::None => Vec::new(),
        Kontra::Kontra(_) => vec![kontra_line(Source::Kontra)],
        Kontra::Re(_) => vec![kontra_line(Source::Kontra), kontra_line(Source::Re)],
    }
}

const fn kontra_line(source: Source) -> ScoreLine {
    ScoreLine {
        source,
        lever: Lever::GameValue,
        change: Change::Multiply(2),
    }
}

/// Augen changes from relics and card modifications on the cards that
/// count for the declarer (skat, won tricks, conceded hands), one line per
/// source.
fn declarer_point_lines(active: &ActiveGame) -> Vec<ScoreLine> {
    let game = &active.game;
    let mut ids: Vec<u32> = game.skat().iter().map(|c| c.id).collect();
    for trick in game.tricks().iter().filter(|t| t.winner.is_declarer()) {
        ids.extend(trick.cards.iter().map(|c| c.id));
    }
    if game.conceded() {
        for seat in Seat::ALL {
            ids.extend(game.hand(seat).iter().map(|c| c.id));
        }
    }
    let mut per_source: BTreeMap<Source, i64> = BTreeMap::new();
    for (card, source, delta) in &active.point_changes {
        if ids.contains(card) {
            *per_source.entry(source.clone()).or_insert(0) += i64::from(*delta);
        }
    }
    per_source
        .into_iter()
        .filter(|(_, delta)| *delta != 0)
        .map(|(source, delta)| ScoreLine {
            source,
            lever: Lever::CardPoints,
            change: Change::Add(delta),
        })
        .collect()
}

impl Run {
    /// Score the finished game and move on.
    pub(super) fn finish_game(
        &mut self,
        content: &Content,
        events: &mut Vec<RunEvent>,
    ) -> Result<(), RunError> {
        let Some(active) = self.game.as_ref() else {
            return Ok(());
        };
        let game = &active.game;
        let declaration = game.declaration().ok_or(RunError::WrongPhase)?;
        let outcome = game.outcome();
        let matadors = game.matadors().ok_or(RunError::WrongPhase)?;
        let parts = match declaration {
            Declaration::Null { risk } => {
                let scoring = self.null_scoring(content);
                let tricks = match &scoring {
                    NullScoring::Formula(f) => usize::try_from(f.short_circuit_tricks).unwrap_or(0),
                    NullScoring::Fixed => 0,
                };
                null_value_parts(risk, &scoring, &outcome, &game.null_style(tricks))
            }
            Declaration::Ramsch => ramsch_value_parts(game.seat_card_points(), game.seat_tricks()),
            Declaration::Suit { .. } | Declaration::Grand { .. } => content.run.game_value_parts(
                declaration,
                matadors,
                &outcome,
                &active.rules.thresholds,
            ),
        };
        self.fire(content, Stage::OnScore, None, Some(&parts), events);

        let Some(active) = self.game.as_ref() else {
            return Ok(());
        };
        let mut lines = declarer_point_lines(active);
        if parts.won_by_lowered_threshold {
            if let Some(source) = &active.rules.win_threshold_source {
                lines.push(ScoreLine {
                    source: source.clone(),
                    lever: Lever::WinThreshold,
                    change: Change::Set(i64::from(active.rules.thresholds.win)),
                });
            }
        }
        if parts.schneider_by_declarer_points {
            let points = outcome.declarer_card_points;
            if let Some((source, _)) = active
                .rules
                .schneider_sources
                .iter()
                .find(|(_, (min, max))| (*min..=*max).contains(&points))
            {
                lines.push(ScoreLine {
                    source: source.clone(),
                    lever: Lever::Schneider,
                    change: Change::Set(1),
                });
            }
        }
        if parts.schneider && active.rules.thresholds.schneider_bonus > 0 {
            if let Some(source) = &active.rules.schneider_bonus_source {
                // A line only if the bonus made the difference.
                let without = Thresholds {
                    schneider_bonus: 0,
                    ..active.rules.thresholds.clone()
                };
                if !game_value_parts(declaration, matadors, &outcome, &without).schneider {
                    lines.push(ScoreLine {
                        source: source.clone(),
                        lever: Lever::Schneider,
                        change: Change::Set(1),
                    });
                }
            }
        }
        lines.extend(active.lines.iter().cloned());
        lines.extend(kontra_lines(active.kontra));
        // A lost game counts single since 04.11.2026 (`RUN_LOSS_FACTOR`):
        // `single_loss` ("Magenbitter", the Ramscher) changes nothing any
        // more and adds no line.
        if parts.won {
            lines.extend(win_factor_lines(active));
        }
        let mut breakdown = score_parts(
            declaration,
            matadors,
            &outcome,
            &active.rules.thresholds,
            parts,
            lines,
        );
        breakdown.seats = Some(active.game.seat_totals());
        let discarded_points = if active.game.skat_picked_up() {
            active.game.skat().iter().map(|c| c.card_points).sum()
        } else {
            0
        };
        self.series.score = self.series.score.saturating_add(breakdown.score);
        self.series.games_played += 1;
        if breakdown.won {
            self.stats.games_won += 1;
            self.stats.best_game = self.stats.best_game.max(breakdown.value);
        } else {
            self.stats.games_lost += 1;
        }
        events.push(RunEvent::GameScored {
            breakdown: breakdown.clone(),
            series_score: self.series.score,
            discarded_points,
        });
        if !declaration.is_ramsch() {
            if let Some(wager) = self.wager.take() {
                let won = breakdown.won && (wager.on == BetOn::Win || breakdown.schneider);
                let payout = if won { wager.payout() } else { 0 };
                events.push(RunEvent::WagerSettled {
                    stake: wager.stake,
                    won,
                    payout,
                });
                if payout > 0 {
                    self.coins = self.coins.saturating_add(payout);
                    events.push(RunEvent::CoinsGained {
                        source: Source::Encounter(wager.encounter),
                        amount: i32::try_from(payout).unwrap_or(i32::MAX),
                        coins: self.coins,
                    });
                    self.settle_debt(events);
                }
            }
        }
        self.last_score = Some(breakdown);
        self.after_game(content, events)
    }

    /// Whether a modification on the player's card has this passive
    /// effect: only a card in the player's edition (ADR 0015).
    pub(super) fn has_card_effect(
        &self,
        content: &Content,
        card: DealtCard,
        effect: &Effect,
    ) -> bool {
        if card.owner != Owner::Player {
            return false;
        }
        self.edition.get(card.id).is_some_and(|card| {
            card.mods
                .iter()
                .filter_map(|m| content.card_mod(m))
                .any(|def| def.passive.contains(effect))
        })
    }

    /// After a game was scored: pub won, pub lost or next deal.
    pub(super) fn after_game(
        &mut self,
        content: &Content,
        events: &mut Vec<RunEvent>,
    ) -> Result<(), RunError> {
        // A boss pub plays all its games (ADR 0017), and so does the
        // Einführung (DESIGN 16.1: every lesson is dealt); the target
        // counts after the last.
        let plays_all = self.series.is_boss_pub() || self.in_tutorial_pub();
        let early = !plays_all || self.series.games_left() == 0;
        if early && self.series.target_reached() {
            return self.series_won(content, events);
        }
        if self.series.games_left() == 0 {
            self.game = None;
            return self.pub_lost(content, events);
        }
        self.deal(content, true, events)
    }

    /// Whether the run stands between two games of a pub: the dealt game
    /// is untouched (ten cards, skat not picked up, nothing declared and
    /// no card played; a Ramsch before its first card). Klarer, Absacker
    /// and "Kneipe verlassen" go only now.
    #[must_use]
    pub fn between_games(&self) -> bool {
        if self.phase != RunPhase::Game {
            return false;
        }
        let Some(active) = self.game.as_ref() else {
            return false;
        };
        let game = &active.game;
        let undeclared = match game.declaration() {
            None => game.phase() == Phase::Declaring,
            Some(d) => d.is_ramsch(),
        };
        undeclared
            && !game.skat_picked_up()
            && game.tricks().is_empty()
            && game.current_trick().is_empty()
            && !active.awaiting_re
    }

    /// Whether the player may give up the pub now
    /// ([`RunAction::GiveUpPub`](super::RunAction::GiveUpPub)): between two games, once its first game
    /// was played.
    #[must_use]
    pub fn can_give_up(&self) -> bool {
        self.series.games_played >= 1 && self.between_games()
    }

    /// "Kneipe verlassen": the pub ends as lost ([`Run::can_give_up`]).
    pub(super) fn give_up_pub(
        &mut self,
        content: &Content,
        events: &mut Vec<RunEvent>,
    ) -> Result<(), RunError> {
        if !self.can_give_up() {
            return Err(RunError::WrongPhase);
        }
        self.game = None;
        events.push(RunEvent::PubGivenUp {
            section: self.series.section,
            index: self.series.index,
            score: self.series.score,
        });
        self.pub_lost(content, events)
    }

    pub(super) fn series_won(
        &mut self,
        content: &Content,
        events: &mut Vec<RunEvent>,
    ) -> Result<(), RunError> {
        self.game = None;
        let economy = &content.run.economy;
        let games_left = self.series.games_left();
        let interest = self
            .coins
            .checked_div(economy.interest_step)
            .unwrap_or(0)
            .min(economy.interest_max);
        // A boss pub plays all its games (ADR 0017) and pays as if
        // `boss_reward.games_left_as` were left.
        let paid_left = if self.series.is_boss_pub() {
            games_left.max(content.run.boss_reward.games_left_as)
        } else {
            games_left
        };
        let mut payout = Payout {
            base: economy.series_reward,
            games_left: paid_left * economy.per_game_left,
            interest,
            bonus: 0,
            boss: 0,
        };
        // `payout_factor`: the parts count `payout_factor` times.
        payout.bonus = payout
            .total()
            .saturating_mul(content.payout_factor(self.character) - 1);
        // A won boss pub pays double (ADR 0017).
        if self.series.is_boss_pub() {
            payout.boss = payout.total();
        }
        self.coins = self.coins.saturating_add(payout.total());
        self.stats.series_won += 1;
        events.push(RunEvent::SeriesWon {
            section: self.series.section,
            index: self.series.index,
            payout,
            coins: self.coins,
            boss: self.series.boss.clone(),
            redeals_used: self.series.redeals_used,
            games_left,
        });
        self.settle_debt(events);
        self.fire(content, Stage::OnSeriesWon, None, None, events);
        self.pub_won(content, events)
    }

    /// District and index of the pub after the current one in the
    /// regular order.
    pub(super) fn next_series_position(&self) -> (u32, u32) {
        if self.series.index + 1 >= PUBS_PER_DISTRICT {
            (self.series.section + 1, 0)
        } else {
            (self.series.section, self.series.index + 1)
        }
    }
}

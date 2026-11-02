//! "Rest abkürzen": ending a game whose score can no longer change
//! ([`RunAction::SettleGame`], [`super::view::GameView::can_settle`]).
//!
//! A game is decided when every quantity that reaches the score is the
//! same however all three seats play the remaining cards, the player
//! included (not only with best play):
//!
//! 1. no hook of the character, the boss, a relic, a house rule of the
//!    evening or a card modification in play can still fire while cards
//!    are played (`OnCardPlayed`, `OnTrickEnd`): its condition is already
//!    certain to stay false (e.g. `TricksPlayed(5)` after the fifth trick,
//!    "Schenken" once the defenders hold 15 Augen). Such hooks can move
//!    thresholds, concede, swap hands or collect lines, so the game is not
//!    decided while one may still fire;
//! 2. the result by the rules ([`ValueParts`]: won, Schneider and Schwarz
//!    of either side, Grundwert, Gewinnstufen, Spielwert) is the same for
//!    every Augen and trick count the remaining tricks can bring
//!    ([`crate::endgame`]), with the thresholds in force (Gewinnschwelle,
//!    "Jungfrau" ranges). For a Null: the declarer takes a trick in no
//!    line (won, with the same style bonus in every line: sevens ducked,
//!    never followed, Kurzschluss) or in every line (lost);
//! 3. every `OnScore` hook either fires in every line or in none,
//!    judged on the same bounds plus the possible winners of the tenth
//!    trick ("Spitze", "Glückspfennig", "Schlusslicht").
//!
//! Flags, counters, Kontra and the lines collected so far do not change
//! while the rest is played (1.), so the score, the Groschen, the run
//! statistics and the events for unlocks are the same in every line.
//!
//! The bounds come from three sources, cheapest first: the cards alone,
//! the Null test of [`null_reach`] (any time in a Null), and the exact
//! search of [`reach`], which runs only in the last [`SEARCH_TRICKS`]
//! tricks and gives up after [`SEARCH_NODES`] trick boundaries. A Ramsch
//! is never settled. Settling plays the remaining cards with a fixed rule
//! (every seat its first legal card) through the normal play, so the
//! events, hooks and the scoring are those of a game played out.

use super::{Run, RunError, RunEvent, RunPhase};
use crate::cards::PlayingCard;
use crate::content::{CardFilter, Condition, Content, HookContext, Stage, evaluate};
use crate::deck::DealtCard;
use crate::endgame::{
    LastTrick, Reach, ReachOptions, Span, live_points, loose_reach, null_reach, reach,
};
use crate::game::{Phase, Seat, TRICKS_PER_GAME};
use crate::rules::{
    Declaration, NullScoring, NullStyle, Outcome, ValueParts, game_value_parts, null_value_parts,
};

/// Tricks left (the one on the table included) from which on the exact
/// search runs.
pub const SEARCH_TRICKS: usize = 4;

/// Trick boundaries the exact search may visit before it gives up.
pub const SEARCH_NODES: u64 = 2_000;

/// Tricks in a game, as a count.
const ALL_TRICKS: u32 = TRICKS_PER_GAME as u32;

/// Whether "the declarer never followed suit" can end true, and whether
/// it can end false.
fn never_followed_possible(style: &NullStyle, reach: &Reach<DealtCard>) -> (bool, bool) {
    (
        style.never_followed && reach.followed.min == 0,
        !style.never_followed || reach.followed.max > 0,
    )
}

/// Everything about the end of the game that is certain or bounded now.
struct Ending<'a> {
    /// The result by the rules, the same in every line.
    parts: ValueParts,
    null: bool,
    /// Tricks the declarer has at the end.
    declarer_tricks: Span,
    /// Tricks played at the end.
    tricks: Span,
    /// Augen of the defenders at the end.
    defender_points: Span,
    /// Tricks with a seven the declarer did not take, at the end.
    sevens_ducked: Span,
    /// "The declarer never followed suit" at the end, if certain.
    never_followed: Option<bool>,
    last_tricks: Option<&'a [LastTrick<DealtCard>]>,
}

/// The rules result with the informational flags cleared: lines on the
/// levers Gewinnschwelle and Schneider do not change the score.
fn scoring_parts(mut parts: ValueParts) -> ValueParts {
    parts.won_by_lowered_threshold = false;
    parts.schneider_by_declarer_points = false;
    parts
}

/// The one value all of `parts` share, `None` if they differ.
fn unique(parts: impl Iterator<Item = ValueParts>) -> Option<ValueParts> {
    let mut first: Option<ValueParts> = None;
    for p in parts.map(scoring_parts) {
        match &first {
            None => first = Some(p),
            Some(f) if *f == p => {}
            Some(_) => return None,
        }
    }
    first
}

/// `Some(b)` if `test` gives `b` for every value of `span`.
fn certain(span: Span, test: impl Fn(u32) -> bool) -> Option<bool> {
    let first = test(span.min);
    (span.min..=span.max)
        .all(|v| test(v) == first)
        .then_some(first)
}

/// `Some(b)` if `test` gives `b` for every possible last trick.
fn certain_last(
    last: Option<&[LastTrick<DealtCard>]>,
    test: impl Fn(&LastTrick<DealtCard>) -> bool,
) -> Option<bool> {
    let list = last?;
    let first = test(list.first()?);
    list.iter().all(|l| test(l) == first).then_some(first)
}

/// Kleene "and", "or" over three-valued results.
fn all3(values: impl Iterator<Item = Option<bool>>) -> Option<bool> {
    let mut out = Some(true);
    for v in values {
        match v {
            Some(false) => return Some(false),
            Some(true) => {}
            None => out = None,
        }
    }
    out
}

fn any3(values: impl Iterator<Item = Option<bool>>) -> Option<bool> {
    let mut out = Some(false);
    for v in values {
        match v {
            Some(true) => return Some(true),
            Some(false) => {}
            None => out = None,
        }
    }
    out
}

/// Whether a condition looks at the tenth trick.
fn needs_last_trick(condition: &Condition) -> bool {
    match condition {
        Condition::All(cs) | Condition::Any(cs) => cs.iter().any(needs_last_trick),
        Condition::Not(c) => needs_last_trick(c),
        Condition::LastTrickWonWith { .. }
        | Condition::LastTrickWinnerMatches { .. }
        | Condition::ThisCardWinsLastTrick => true,
        _ => false,
    }
}

/// An `OnScore` condition at the end of every line: `Some` if it is the
/// same in all of them. `ctx` is the game now with the common result.
fn on_score(
    condition: &Condition,
    ctx: &HookContext<'_>,
    this_card: Option<DealtCard>,
    end: &Ending<'_>,
) -> Option<bool> {
    match condition {
        Condition::All(cs) => all3(cs.iter().map(|c| on_score(c, ctx, this_card, end))),
        Condition::Any(cs) => any3(cs.iter().map(|c| on_score(c, ctx, this_card, end))),
        Condition::Not(c) => on_score(c, ctx, this_card, end).map(|b| !b),
        // Fixed by now or by the common result.
        Condition::Always
        | Condition::Flag(_)
        | Condition::CounterAtLeast { .. }
        | Condition::Won
        | Condition::Lost
        | Condition::WonSchneider
        | Condition::GameIs(_)
        | Condition::SuitGame
        | Condition::RiskAtLeast(_)
        | Condition::ThisCardIsTrumpSuit
        | Condition::GamesLeftAtLeast(_)
        | Condition::NullGame
        | Condition::NullRiskAtLeast(_)
        | Condition::RamschGame
        | Condition::HandGame
        | Condition::MatadorsAtLeast { .. }
        | Condition::PlayedCardMatches(_)
        | Condition::PlayedByDeclarer => Some(evaluate(condition, ctx, this_card)),
        Condition::AllTricksWon => certain(end.declarer_tricks, |t| t == ALL_TRICKS),
        Condition::NoTrickTaken => certain(end.declarer_tricks, |t| t == 0),
        Condition::TricksPlayed(n) => certain(end.tricks, |t| t == *n),
        Condition::DefenderPointsBelow(n) => certain(end.defender_points, |p| p < *n),
        Condition::SevenDucked => {
            if end.null {
                certain(end.sevens_ducked, |s| s > 0)
            } else {
                Some(false)
            }
        }
        Condition::NeverFollowedSuit => {
            if end.null {
                end.never_followed
            } else {
                Some(false)
            }
        }
        Condition::LastTrickWonWith { card, by_declarer } => certain_last(end.last_tricks, |l| {
            l.is_some_and(|(c, seat)| {
                c.face() == Some(*card) && (!by_declarer || seat.is_declarer())
            })
        }),
        Condition::LastTrickWinnerMatches {
            filter,
            by_declarer,
        } => certain_last(end.last_tricks, |l| {
            l.is_some_and(|(c, seat)| {
                c.face().is_some_and(|f| filter.matches(f)) && (!by_declarer || seat.is_declarer())
            })
        }),
        Condition::ThisCardWinsLastTrick => certain_last(end.last_tricks, |l| {
            l.is_some_and(|(c, _)| this_card.is_some_and(|t| t.id == c.id))
        }),
        // On trick end only: never true when the game is scored.
        Condition::TrickWinnerMatches { .. } | Condition::FirstTrickOfDeclarer => Some(false),
    }
}

/// Whether a condition of a hook that fires during play (`stage`) is
/// certain to stay false for every card still to come: `Some(false)`.
/// `ctx` is the game now, without a result and without a card played.
fn during_play(
    condition: &Condition,
    stage: Stage,
    ctx: &HookContext<'_>,
    this_card: Option<DealtCard>,
) -> Option<bool> {
    let game = ctx.game?;
    let done = u32::try_from(game.tricks().len()).unwrap_or(0);
    let outcome = game.outcome();
    let null = game.declaration().is_some_and(Declaration::is_null);
    // A card of the filter still to be played: by anyone, or only by the
    // declarer (the cards on the table were played already).
    let to_play = |filter: &CardFilter, declarer_only: bool| {
        let matches = |c: &DealtCard| c.face().is_some_and(|f| filter.matches(f));
        if declarer_only {
            game.hand(Seat::DECLARER).iter().any(matches)
        } else {
            Seat::ALL
                .iter()
                .any(|&seat| game.hand(seat).iter().any(matches))
        }
    };
    match condition {
        // "A card of the declarer that matches" ("Bubenparade"): only his
        // own cards still to come can make it true.
        Condition::All(cs)
            if stage == Stage::OnCardPlayed && cs.contains(&Condition::PlayedByDeclarer) =>
        {
            all3(cs.iter().map(|c| match c {
                Condition::PlayedCardMatches(filter) => (!to_play(filter, true)).then_some(false),
                _ => during_play(c, stage, ctx, this_card),
            }))
        }
        Condition::All(cs) => all3(cs.iter().map(|c| during_play(c, stage, ctx, this_card))),
        Condition::Any(cs) => any3(cs.iter().map(|c| during_play(c, stage, ctx, this_card))),
        Condition::Not(c) => during_play(c, stage, ctx, this_card).map(|b| !b),
        // Fixed while the rest is played; without a result `Won`, `Lost`
        // and `WonSchneider` are false.
        Condition::Always
        | Condition::Flag(_)
        | Condition::CounterAtLeast { .. }
        | Condition::Won
        | Condition::Lost
        | Condition::WonSchneider
        | Condition::GameIs(_)
        | Condition::SuitGame
        | Condition::RiskAtLeast(_)
        | Condition::ThisCardIsTrumpSuit
        | Condition::GamesLeftAtLeast(_)
        | Condition::NullGame
        | Condition::NullRiskAtLeast(_)
        | Condition::RamschGame
        | Condition::HandGame
        | Condition::MatadorsAtLeast { .. } => Some(evaluate(condition, ctx, this_card)),
        Condition::TricksPlayed(n) => {
            // Tricks complete when the hook looks: after each card from
            // now on, or after each trick still to finish.
            let first = if stage == Stage::OnTrickEnd {
                done + 1
            } else {
                done
            };
            let last = u32::try_from(TRICKS_PER_GAME).unwrap_or(10);
            if (first..=last).contains(n) {
                None
            } else {
                Some(false)
            }
        }
        // The defenders' Augen only grow.
        Condition::DefenderPointsBelow(n) => (outcome.defender_card_points >= *n).then_some(false),
        Condition::PlayedCardMatches(filter) => {
            (stage == Stage::OnTrickEnd || !to_play(filter, false)).then_some(false)
        }
        Condition::PlayedByDeclarer => (stage == Stage::OnTrickEnd).then_some(false),
        Condition::NoTrickTaken => (outcome.declarer_tricks > 0).then_some(false),
        Condition::AllTricksWon => (outcome.defender_tricks > 0).then_some(false),
        Condition::SevenDucked | Condition::NeverFollowedSuit => (!null).then_some(false),
        // Only a card still in a hand (or on the table) can win a trick to
        // come: the declarer's own for `by_declarer` (it is his turn, so
        // none of his lies on the table).
        Condition::TrickWinnerMatches {
            filter,
            by_declarer,
        } => {
            let matches = |c: &DealtCard| c.face().is_some_and(|f| filter.matches(f));
            let left = if *by_declarer {
                game.hand(Seat::DECLARER).iter().any(matches)
            } else {
                Seat::ALL
                    .iter()
                    .any(|&seat| game.hand(seat).iter().any(matches))
                    || game.current_trick().iter().any(matches)
            };
            (!left).then_some(false)
        }
        // The declarer's trick count only grows.
        Condition::FirstTrickOfDeclarer => (outcome.declarer_tricks > 0).then_some(false),
        Condition::LastTrickWonWith { .. }
        | Condition::LastTrickWinnerMatches { .. }
        | Condition::ThisCardWinsLastTrick => None,
    }
}

impl Run {
    /// Whether the player may end the game in progress now with
    /// [`super::RunAction::SettleGame`]: it is his turn and the rest can
    /// no longer change anything that counts (see the module docs).
    #[must_use]
    pub fn can_settle(&self, content: &Content) -> bool {
        if self.phase != RunPhase::Game {
            return false;
        }
        let Some(active) = self.game.as_ref() else {
            return false;
        };
        let game = &active.game;
        if active.awaiting_re
            || game.phase() != Phase::Playing
            || game.current_seat() != Some(Seat::DECLARER)
        {
            return false;
        }
        let Some(declaration) = game.declaration() else {
            return false;
        };
        if declaration.is_ramsch() {
            return false;
        }
        let sources = self.sources(content, true);
        let now = HookContext {
            flags: &self.flags,
            counters: &self.counters,
            game: Some(game),
            played: None,
            result: None,
            games_left: self.series.games_left(),
        };
        let mut last_trick = false;
        for source in &sources {
            for hook in source.hooks {
                let stage = hook.stage();
                match stage {
                    Stage::OnCardPlayed | Stage::OnTrickEnd => {
                        if during_play(hook.when(), stage, &now, source.this_card) != Some(false) {
                            return false;
                        }
                    }
                    Stage::OnScore => last_trick |= needs_last_trick(hook.when()),
                    _ => {}
                }
            }
        }
        let scoring = self.null_scoring(content);
        let early = match &scoring {
            NullScoring::Formula(f) if declaration.is_null() => {
                usize::try_from(f.short_circuit_tricks).unwrap_or(0)
            }
            _ => 0,
        };
        let decided = |reach: &Reach<DealtCard>| {
            let Some(end) = self.ending(declaration, &scoring, early, reach) else {
                return false;
            };
            let ctx = HookContext {
                result: Some(&end.parts),
                ..now
            };
            sources.iter().all(|source| {
                source
                    .hooks
                    .iter()
                    .filter(|h| h.stage() == Stage::OnScore)
                    .all(|h| on_score(h.when(), &ctx, source.this_card, &end).is_some())
            })
        };
        if loose_reach(game, early).is_some_and(|r| decided(&r)) {
            return true;
        }
        if null_reach(game, early).is_some_and(|r| decided(&r)) {
            return true;
        }
        if TRICKS_PER_GAME - game.tricks().len() > SEARCH_TRICKS {
            return false;
        }
        let options = ReachOptions {
            last_trick,
            early_tricks: early,
            node_limit: SEARCH_NODES,
        };
        reach(game, options).is_some_and(|r| decided(&r))
    }

    /// The end of the game in every line as far as `reach` tells, with
    /// the one rules result they all share; `None` if it differs.
    fn ending<'a>(
        &self,
        declaration: Declaration,
        scoring: &NullScoring,
        early: usize,
        reach: &'a Reach<DealtCard>,
    ) -> Option<Ending<'a>> {
        let active = self.game.as_ref()?;
        let game = &active.game;
        let now = game.outcome();
        let tricks_done = u32::try_from(game.tricks().len()).unwrap_or(0);
        let style = game.null_style(early);
        let declarer_tricks = reach.declarer_tricks.shift(now.declarer_tricks);
        let (parts, defender_points) = match declaration {
            Declaration::Ramsch => return None,
            Declaration::Null { risk } => {
                let parts = if reach.declarer_tricks.max == 0 {
                    // Won in every line: the style bonus must agree.
                    let (can_true, can_false) = never_followed_possible(&style, reach);
                    let never: Vec<bool> = [can_true, can_false]
                        .into_iter()
                        .zip([true, false])
                        .filter_map(|(possible, value)| possible.then_some(value))
                        .collect();
                    let after: Vec<Option<u32>> = if early == 0 {
                        vec![None]
                    } else if early <= game.tricks().len() {
                        vec![style.defender_points_after]
                    } else {
                        let before = now.defender_card_points;
                        reach
                            .early_defender_points
                            .values(before)
                            .map(Some)
                            .collect()
                    };
                    let won = Outcome {
                        defender_tricks: ALL_TRICKS,
                        ..Outcome::default()
                    };
                    let mut all = Vec::new();
                    for sevens in reach.sevens_ducked.values(style.sevens_ducked) {
                        for &never_followed in &never {
                            for &defender_points_after in &after {
                                let style = NullStyle {
                                    sevens_ducked: sevens,
                                    never_followed,
                                    defender_points_after,
                                };
                                all.push(null_value_parts(risk, scoring, &won, &style));
                            }
                        }
                    }
                    unique(all.into_iter())?
                } else if reach.declarer_tricks.min >= 1 {
                    let lost = Outcome {
                        declarer_tricks: 1,
                        ..Outcome::default()
                    };
                    null_value_parts(risk, scoring, &lost, &NullStyle::default())
                } else {
                    return None;
                };
                (parts, reach.defender_points.shift(now.defender_card_points))
            }
            Declaration::Suit { .. } | Declaration::Grand { .. } => {
                let matadors = game.matadors()?;
                let total = now.declarer_card_points + now.defender_card_points + live_points(game);
                let thresholds = &active.rules.thresholds;
                let points = reach.declarer_points.shift(now.declarer_card_points);
                let mut all = Vec::new();
                for p in points.values(0) {
                    for t in declarer_tricks.values(0) {
                        let outcome = Outcome {
                            declarer_card_points: p,
                            defender_card_points: total.saturating_sub(p),
                            declarer_tricks: t,
                            defender_tricks: ALL_TRICKS.saturating_sub(t),
                        };
                        all.push(game_value_parts(
                            declaration,
                            matadors,
                            &outcome,
                            thresholds,
                        ));
                    }
                }
                let parts = unique(all.into_iter())?;
                let defender_points = Span {
                    min: total.saturating_sub(points.max),
                    max: total.saturating_sub(points.min),
                };
                (parts, defender_points)
            }
        };
        let never_followed = match never_followed_possible(&style, reach) {
            (true, false) => Some(true),
            (false, true) => Some(false),
            _ => None,
        };
        Some(Ending {
            parts,
            null: declaration.is_null(),
            declarer_tricks,
            tricks: reach.tricks.shift(tricks_done),
            defender_points,
            sevens_ducked: reach.sevens_ducked.shift(style.sevens_ducked),
            never_followed,
            last_tricks: reach.last_tricks.as_deref(),
        })
    }

    /// Play the rest of a decided game, every seat its first legal card,
    /// and score it.
    pub(super) fn settle_game(
        &mut self,
        content: &Content,
        events: &mut Vec<RunEvent>,
    ) -> Result<(), RunError> {
        if !self.can_settle(content) {
            return Err(RunError::NotDecided);
        }
        while let Some((seat, card)) = self.game.as_ref().and_then(|active| {
            let seat = active.game.current_seat()?;
            let card = *active.game.legal_moves(seat).first()?;
            Some((seat, card))
        }) {
            self.play_card(content, seat, card, events)?;
        }
        self.settle(content, events)
    }
}

//! Applying content: resolving the deck into dealt cards for one game and
//! evaluating hook conditions. The run ([`crate::run`]) decides when hooks
//! fire and applies their effects to its own state.

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use super::effects::{Condition, Effect, Hook};
use super::score::Source;
use super::{CardModDef, CardModId, Content};
use crate::cards::{CardKind, PlayingCard};
use crate::deck::{DealtCard, Deck};
use crate::game::{Game, Seat, TRICKS_PER_GAME};
use crate::rules::{Declaration, GameType, Matadors, ValueParts, acts_as_honorary_jack};

/// One active content item: where it comes from and what it does. Card
/// modifications carry the card instance they sit on.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActiveSource<'a> {
    /// Origin, for the score breakdown and events.
    pub source: Source,
    /// Always-on effects.
    pub passive: &'a [Effect],
    /// Triggered effects.
    pub hooks: &'a [Hook],
    /// For card modifications: the dealt card it sits on.
    pub this_card: Option<DealtCard>,
}

/// The deck resolved for one game.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedDeck {
    /// Every card of the deck with effective Augen, rank bonus and
    /// visibility, in deck order.
    pub cards: Vec<DealtCard>,
    /// Augen changes per card and source, for the score breakdown:
    /// `(card id, source, delta)`.
    pub point_changes: Vec<(u32, Source, i32)>,
}

/// Resolve the deck for one game.
///
/// Order per card: `BlankOut` from bosses and relics turns matching cards
/// into blanks (their modifications are inactive for this game); then
/// `SetCardPoints` from bosses and relics in source order; then the card's
/// own modifications in the order they were attached (`AddCardPoints`,
/// `RankBonus`, `RevealHolder`, `NeverWins`, `HonoraryJack`, `NoDiscard`). Augen never drop below zero.
/// `RevealHolder` is off while a global source has `DisableInformation`.
#[must_use]
pub fn resolve_deck(deck: &Deck, content: &Content, global: &[ActiveSource<'_>]) -> ResolvedDeck {
    let mut cards = Vec::with_capacity(deck.len());
    let mut point_changes = Vec::new();
    let no_information = global
        .iter()
        .any(|s| s.passive.contains(&Effect::DisableInformation));
    for deck_card in deck.cards() {
        let blanked = deck_card.kind.face().is_some_and(|face| {
            global.iter().any(|s| {
                s.passive
                    .iter()
                    .any(|e| matches!(e, Effect::BlankOut(f) if f.matches(face)))
            })
        });
        let kind = if blanked {
            CardKind::Blank
        } else {
            deck_card.kind
        };
        let mut dealt = DealtCard::plain(deck_card.id, kind);
        let Some(face) = kind.face() else {
            cards.push(dealt);
            continue;
        };
        for source in global {
            for effect in source.passive {
                if let Effect::SetCardPoints { filter, points } = effect {
                    if filter.matches(face) && *points != dealt.card_points {
                        let delta = i64::from(*points) - i64::from(dealt.card_points);
                        point_changes.push((
                            dealt.id,
                            source.source.clone(),
                            i32::try_from(delta).unwrap_or(0),
                        ));
                        dealt.card_points = *points;
                    }
                }
            }
        }
        for mod_id in &deck_card.mods {
            let Some(def) = content.card_mod(mod_id) else {
                continue;
            };
            let source = Source::CardMod {
                card_mod: mod_id.clone(),
                card: deck_card.id,
            };
            for effect in &def.passive {
                match effect {
                    Effect::AddCardPoints(delta) => {
                        let new = i64::from(dealt.card_points) + i64::from(*delta);
                        let new = u32::try_from(new.max(0)).unwrap_or(0);
                        let applied = i64::from(new) - i64::from(dealt.card_points);
                        if applied != 0 {
                            point_changes.push((
                                dealt.id,
                                source.clone(),
                                i32::try_from(applied).unwrap_or(0),
                            ));
                        }
                        dealt.card_points = new;
                    }
                    Effect::RankBonus(bonus) => {
                        dealt.rank_bonus = dealt.rank_bonus.saturating_add(*bonus);
                    }
                    Effect::RevealHolder => dealt.revealed = !no_information,
                    Effect::NeverWins => dealt.never_wins = true,
                    Effect::HonoraryJack => dealt.honorary_jack = true,
                    Effect::NoDiscard => dealt.no_discard = true,
                    _ => {}
                }
            }
        }
        cards.push(dealt);
    }
    ResolvedDeck {
        cards,
        point_changes,
    }
}

/// The card modifications that are active in a game: those on dealt cards
/// that are in play and not blanked out, in card id order.
#[must_use]
pub fn card_mod_sources<'a>(
    deck: &Deck,
    content: &'a Content,
    game: &Game<DealtCard>,
) -> Vec<ActiveSource<'a>> {
    let mut in_play: Vec<DealtCard> = game.cards_in_play().to_vec();
    in_play.sort_by_key(|c| c.id);
    let mut sources = Vec::new();
    for card in in_play {
        if card.is_blank() {
            continue;
        }
        let Some(deck_card) = deck.get(card.id) else {
            continue;
        };
        for mod_id in &deck_card.mods {
            if let Some(def) = content.card_mod(mod_id) {
                sources.push(ActiveSource {
                    source: Source::CardMod {
                        card_mod: CardModId::clone(mod_id),
                        card: card.id,
                    },
                    passive: &def.passive,
                    hooks: &def.hooks,
                    this_card: Some(card),
                });
            }
        }
    }
    sources
}

/// Everything a condition may look at.
#[derive(Debug, Clone, Copy)]
pub struct HookContext<'a> {
    /// Run flags.
    pub flags: &'a BTreeSet<String>,
    /// Run counters.
    pub counters: &'a BTreeMap<String, u32>,
    /// The game in progress, if any.
    pub game: Option<&'a Game<DealtCard>>,
    /// The card just played and who played it (on card played).
    pub played: Option<(Seat, DealtCard)>,
    /// The result by the rules (on score).
    pub result: Option<&'a ValueParts>,
    /// Games left in the series (on series won).
    pub games_left: u32,
}

/// Evaluate a condition. `this_card` is the card a modification sits on.
/// Conditions that lack the information they need (no game, no result)
/// are false.
#[must_use]
pub fn evaluate(
    condition: &Condition,
    ctx: &HookContext<'_>,
    this_card: Option<DealtCard>,
) -> bool {
    let game = ctx.game;
    let declaration = game.and_then(Game::declaration);
    match condition {
        Condition::Always => true,
        Condition::All(cs) => cs.iter().all(|c| evaluate(c, ctx, this_card)),
        Condition::Any(cs) => cs.iter().any(|c| evaluate(c, ctx, this_card)),
        Condition::Not(c) => !evaluate(c, ctx, this_card),
        Condition::Flag(flag) => ctx.flags.contains(flag),
        Condition::CounterAtLeast { counter, value } => {
            ctx.counters.get(counter).copied().unwrap_or(0) >= *value
        }
        Condition::Won => ctx.result.is_some_and(|r| r.won),
        Condition::Lost => ctx.result.is_some_and(|r| !r.won),
        Condition::AllTricksWon => game.is_some_and(|g| {
            g.outcome().defender_tricks == 0
                && g.outcome().declarer_tricks == u32::try_from(TRICKS_PER_GAME).unwrap_or(10)
        }),
        Condition::GameIs(game_type) => {
            declaration.is_some_and(|d| !d.is_ramsch() && d.game() == *game_type)
        }
        Condition::SuitGame => declaration.is_some_and(|d| matches!(d.game(), GameType::Suit(_))),
        Condition::RiskAtLeast(risk) => {
            declaration.is_some_and(|d| d.risk().is_some_and(|r| r >= *risk))
        }
        Condition::LastTrickWonWith { card, by_declarer } => last_trick(game).is_some_and(|t| {
            t.winning_card().face() == Some(*card) && (!by_declarer || t.winner.is_declarer())
        }),
        Condition::LastTrickWinnerMatches {
            filter,
            by_declarer,
        } => last_trick(game).is_some_and(|t| {
            t.winning_card().face().is_some_and(|f| filter.matches(f))
                && (!by_declarer || t.winner.is_declarer())
        }),
        Condition::WonSchneider => ctx.result.is_some_and(|r| r.won && r.schneider),
        Condition::ThisCardWinsLastTrick => match (last_trick(game), this_card) {
            (Some(t), Some(this)) => t.winning_card().id == this.id,
            _ => false,
        },
        Condition::ThisCardIsTrumpSuit => match (declaration, this_card.and_then(|c| c.face())) {
            (Some(d), Some(face)) => d.game() == GameType::Suit(face.suit),
            _ => false,
        },
        Condition::TricksPlayed(n) => {
            game.is_some_and(|g| u32::try_from(g.tricks().len()).unwrap_or(0) == *n)
        }
        Condition::DefenderPointsBelow(n) => {
            game.is_some_and(|g| g.outcome().defender_card_points < *n)
        }
        Condition::PlayedCardMatches(filter) => ctx
            .played
            .and_then(|(_, card)| card.face())
            .is_some_and(|face| filter.matches(face)),
        Condition::PlayedByDeclarer => ctx.played.is_some_and(|(seat, _)| seat.is_declarer()),
        Condition::GamesLeftAtLeast(n) => ctx.games_left >= *n,
        Condition::NullGame => declaration.is_some_and(Declaration::is_null),
        Condition::NullRiskAtLeast(risk) => {
            declaration.is_some_and(|d| d.null_risk().is_some_and(|r| r >= *risk))
        }
        Condition::RamschGame => declaration.is_some_and(Declaration::is_ramsch),
        Condition::HandGame => declaration.is_some_and(Declaration::hand),
        Condition::NoTrickTaken => game.is_some_and(|g| g.seat_tricks()[0] == 0),
        Condition::SevenDucked => {
            null_game(game).is_some_and(|g| g.null_style(0).sevens_ducked > 0)
        }
        Condition::NeverFollowedSuit => {
            null_game(game).is_some_and(|g| g.null_style(0).never_followed)
        }
        Condition::MatadorsAtLeast { with, count } => game
            .and_then(Game::matadors)
            .is_some_and(|m| m.count > 0 && m.with == *with && m.count >= *count),
    }
}

/// Evaluate an `OnScore` condition before the game is over, for a game
/// the declarer would win: `Some(true)` or `Some(false)` when the answer
/// is already certain from what the player knows, `None` when it depends
/// on how the game goes on (the score preview of the run counts only
/// certain hooks).
///
/// `ctx.result` is the result of the smallest win as currently known
/// (Schneider and Schwarz only when declared). `matadors` holds every
/// count of Spitzen that fits what the player knows (one entry once it is
/// certain).
///
/// Certain: `Won` (true), `Lost` (false), flags and counters as they are
/// now, everything fixed by the declaration, `WonSchneider` and
/// `AllTricksWon` when the declaration demands them (false in Null),
/// `NoTrickTaken` in Null (a won Null takes no trick), `MatadorsAtLeast`
/// when every possible count agrees, and the conditions that are always
/// false on score (the card just played). Unknown: everything about the
/// tenth trick, Augen, tricks played and Null style.
#[must_use]
pub fn evaluate_if_won(
    condition: &Condition,
    ctx: &HookContext<'_>,
    this_card: Option<DealtCard>,
    matadors: &[Matadors],
) -> Option<bool> {
    let declaration = ctx.game.and_then(Game::declaration);
    let null = declaration.is_some_and(Declaration::is_null);
    match condition {
        Condition::All(cs) => {
            let mut all = Some(true);
            for c in cs {
                match evaluate_if_won(c, ctx, this_card, matadors) {
                    Some(false) => return Some(false),
                    Some(true) => {}
                    None => all = None,
                }
            }
            all
        }
        Condition::Any(cs) => {
            let mut any = Some(false);
            for c in cs {
                match evaluate_if_won(c, ctx, this_card, matadors) {
                    Some(true) => return Some(true),
                    Some(false) => {}
                    None => any = None,
                }
            }
            any
        }
        Condition::Not(c) => evaluate_if_won(c, ctx, this_card, matadors).map(|b| !b),
        Condition::Won => Some(true),
        Condition::Lost => Some(false),
        Condition::WonSchneider | Condition::AllTricksWon if null => Some(false),
        Condition::WonSchneider => declaration
            .is_some_and(Declaration::schneider_declared)
            .then_some(true),
        Condition::AllTricksWon => declaration
            .is_some_and(Declaration::schwarz_declared)
            .then_some(true),
        Condition::NoTrickTaken if null => Some(true),
        Condition::MatadorsAtLeast { with, count } => {
            let holds = |m: &Matadors| m.count > 0 && m.with == *with && m.count >= *count;
            if matadors.iter().all(holds) {
                Some(true)
            } else if matadors.iter().any(holds) {
                None
            } else {
                Some(false)
            }
        }
        Condition::LastTrickWonWith { .. }
        | Condition::LastTrickWinnerMatches { .. }
        | Condition::ThisCardWinsLastTrick
        | Condition::TricksPlayed(_)
        | Condition::DefenderPointsBelow(_)
        | Condition::NoTrickTaken
        | Condition::SevenDucked
        | Condition::NeverFollowedSuit => None,
        Condition::Always
        | Condition::Flag(_)
        | Condition::CounterAtLeast { .. }
        | Condition::GameIs(_)
        | Condition::SuitGame
        | Condition::RiskAtLeast(_)
        | Condition::ThisCardIsTrumpSuit
        | Condition::PlayedCardMatches(_)
        | Condition::PlayedByDeclarer
        | Condition::GamesLeftAtLeast(_)
        | Condition::NullGame
        | Condition::NullRiskAtLeast(_)
        | Condition::RamschGame
        | Condition::HandGame => Some(evaluate(condition, ctx, this_card)),
    }
}

/// Whether a card modification works in the game in progress, as far as
/// the player can know it (shown in the info card of a card).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CardModStatus {
    /// It works in this game (its condition holds).
    Active,
    /// It cannot work in this game any more.
    Inactive,
    /// It depends on how the game goes (the tenth trick, the result).
    Open,
}

/// The status of a card modification on `this_card` in the game of
/// `ctx`, once the game is declared. `None` before the declaration and
/// for a modification whose effect in the game has no condition ("Fett",
/// "Gezinkt": they always work).
///
/// Conditional are the in-game hooks with a condition (not "beim
/// Erhalt") and the passive "Bube ehrenhalber" (suit games only). The
/// modification is [`CardModStatus::Active`] when one of them holds for
/// certain, [`CardModStatus::Inactive`] when none can hold any more, and
/// [`CardModStatus::Open`] otherwise. Only what the player knows counts:
/// the declaration, flags and counters, and the tricks on the table (see
/// [`evaluate_known`]).
#[must_use]
pub fn card_mod_status(
    def: &CardModDef,
    ctx: &HookContext<'_>,
    this_card: DealtCard,
) -> Option<CardModStatus> {
    let declaration = ctx.game.and_then(Game::declaration)?;
    let mut parts: Vec<Option<bool>> = Vec::new();
    if def.passive.contains(&Effect::HonoraryJack) {
        parts.push(Some(acts_as_honorary_jack(declaration.game(), this_card)));
    }
    for hook in &def.hooks {
        if hook.stage().in_game() && *hook.when() != Condition::Always {
            parts.push(evaluate_known(hook.when(), ctx, Some(this_card)));
        }
    }
    if parts.is_empty() {
        None
    } else if parts.contains(&Some(true)) {
        Some(CardModStatus::Active)
    } else if parts.iter().all(|p| *p == Some(false)) {
        Some(CardModStatus::Inactive)
    } else {
        Some(CardModStatus::Open)
    }
}

/// Evaluate a condition during the game from what the player knows:
/// `Some` when the answer is certain now, `None` when it depends on how
/// the game goes on.
///
/// Certain: everything fixed by the declaration, flags and counters as
/// they are now, and "this card wins the tenth trick" once the tenth
/// trick is played or the card fell in an earlier one. Unknown: the
/// result, the tricks to come, Augen, Null style and the Spitzen (they
/// may hang on the hidden skat).
#[must_use]
pub fn evaluate_known(
    condition: &Condition,
    ctx: &HookContext<'_>,
    this_card: Option<DealtCard>,
) -> Option<bool> {
    match condition {
        Condition::All(cs) => {
            let mut all = Some(true);
            for c in cs {
                match evaluate_known(c, ctx, this_card) {
                    Some(false) => return Some(false),
                    Some(true) => {}
                    None => all = None,
                }
            }
            all
        }
        Condition::Any(cs) => {
            let mut any = Some(false);
            for c in cs {
                match evaluate_known(c, ctx, this_card) {
                    Some(true) => return Some(true),
                    Some(false) => {}
                    None => any = None,
                }
            }
            any
        }
        Condition::Not(c) => evaluate_known(c, ctx, this_card).map(|b| !b),
        Condition::ThisCardWinsLastTrick => {
            let (game, this) = (ctx.game?, this_card?);
            let tricks = game.tricks();
            if tricks.len() == TRICKS_PER_GAME {
                Some(evaluate(condition, ctx, this_card))
            } else if tricks
                .iter()
                .any(|t| t.cards.iter().any(|c| c.id == this.id))
            {
                Some(false)
            } else {
                None
            }
        }
        Condition::Always
        | Condition::Flag(_)
        | Condition::CounterAtLeast { .. }
        | Condition::GameIs(_)
        | Condition::SuitGame
        | Condition::RiskAtLeast(_)
        | Condition::ThisCardIsTrumpSuit
        | Condition::NullGame
        | Condition::NullRiskAtLeast(_)
        | Condition::RamschGame
        | Condition::HandGame => Some(evaluate(condition, ctx, this_card)),
        Condition::Won
        | Condition::Lost
        | Condition::AllTricksWon
        | Condition::LastTrickWonWith { .. }
        | Condition::LastTrickWinnerMatches { .. }
        | Condition::WonSchneider
        | Condition::TricksPlayed(_)
        | Condition::DefenderPointsBelow(_)
        | Condition::PlayedCardMatches(_)
        | Condition::PlayedByDeclarer
        | Condition::GamesLeftAtLeast(_)
        | Condition::NoTrickTaken
        | Condition::SevenDucked
        | Condition::NeverFollowedSuit
        | Condition::MatadorsAtLeast { .. } => None,
    }
}

/// The game, if it is a Null game.
fn null_game(game: Option<&Game<DealtCard>>) -> Option<&Game<DealtCard>> {
    game.filter(|g| g.declaration().is_some_and(Declaration::is_null))
}

/// The tenth trick, if it was played.
fn last_trick(game: Option<&Game<DealtCard>>) -> Option<crate::game::CompletedTrick<DealtCard>> {
    let tricks = game?.tricks();
    if tricks.len() == TRICKS_PER_GAME {
        tricks.last().copied()
    } else {
        None
    }
}

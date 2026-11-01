//! The hook API: when an effect fires ([`Hook`]), under which condition
//! ([`Condition`]) and what it does ([`Effect`]). All three are closed enums
//! with data, written in RON by content authors. There is no scripting.
//!
//! Design pillar 1 ("Die Formel ist das Spiel"): every effect names the
//! lever of the game value formula it turns in its doc comment.

use serde::{Deserialize, Serialize};

use crate::cards::{Card, Rank, Suit};
use crate::rules::{GameType, RiskLevel};

/// The moment at which content acts. `Passive` effects are always on while
/// the content is active; they are applied when a game is set up.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Stage {
    /// Always on (applied when a game is set up, before dealing).
    Passive,
    /// Right after the cards were dealt.
    OnDeal,
    /// Right after the declarer announced the game.
    OnDeclare,
    /// After every card played, by any seat.
    OnCardPlayed,
    /// After every completed trick.
    OnTrickEnd,
    /// After the game was scored by the rules, before relic multipliers.
    OnScore,
    /// After a series was won (no game in progress).
    OnSeriesWon,
}

impl Stage {
    /// Whether a game is in progress at this stage.
    #[must_use]
    pub const fn in_game(self) -> bool {
        !matches!(self, Stage::OnSeriesWon)
    }

    /// Whether this stage comes before the result is known, so that it may
    /// still move thresholds.
    #[must_use]
    pub const fn before_result(self) -> bool {
        matches!(
            self,
            Stage::Passive
                | Stage::OnDeal
                | Stage::OnDeclare
                | Stage::OnCardPlayed
                | Stage::OnTrickEnd
        )
    }
}

/// A triggered effect list. `when` defaults to [`Condition::Always`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Hook {
    /// `on_deal`: after dealing.
    OnDeal {
        /// Condition.
        #[serde(default)]
        when: Condition,
        /// Effects, applied in order.
        then: Vec<Effect>,
    },
    /// `on_declare`: after the declaration.
    OnDeclare {
        /// Condition.
        #[serde(default)]
        when: Condition,
        /// Effects, applied in order.
        then: Vec<Effect>,
    },
    /// `on_card_played`: after every card.
    OnCardPlayed {
        /// Condition.
        #[serde(default)]
        when: Condition,
        /// Effects, applied in order.
        then: Vec<Effect>,
    },
    /// `on_trick_end`: after every trick.
    OnTrickEnd {
        /// Condition.
        #[serde(default)]
        when: Condition,
        /// Effects, applied in order.
        then: Vec<Effect>,
    },
    /// `on_score`: after the game is scored.
    OnScore {
        /// Condition.
        #[serde(default)]
        when: Condition,
        /// Effects, applied in order.
        then: Vec<Effect>,
    },
    /// After a won series.
    OnSeriesWon {
        /// Condition.
        #[serde(default)]
        when: Condition,
        /// Effects, applied in order.
        then: Vec<Effect>,
    },
}

impl Hook {
    /// The stage this hook fires in.
    #[must_use]
    pub const fn stage(&self) -> Stage {
        match self {
            Hook::OnDeal { .. } => Stage::OnDeal,
            Hook::OnDeclare { .. } => Stage::OnDeclare,
            Hook::OnCardPlayed { .. } => Stage::OnCardPlayed,
            Hook::OnTrickEnd { .. } => Stage::OnTrickEnd,
            Hook::OnScore { .. } => Stage::OnScore,
            Hook::OnSeriesWon { .. } => Stage::OnSeriesWon,
        }
    }

    /// The condition.
    #[must_use]
    pub const fn when(&self) -> &Condition {
        match self {
            Hook::OnDeal { when, .. }
            | Hook::OnDeclare { when, .. }
            | Hook::OnCardPlayed { when, .. }
            | Hook::OnTrickEnd { when, .. }
            | Hook::OnScore { when, .. }
            | Hook::OnSeriesWon { when, .. } => when,
        }
    }

    /// The effects.
    #[must_use]
    pub fn then(&self) -> &[Effect] {
        match self {
            Hook::OnDeal { then, .. }
            | Hook::OnDeclare { then, .. }
            | Hook::OnCardPlayed { then, .. }
            | Hook::OnTrickEnd { then, .. }
            | Hook::OnScore { then, .. }
            | Hook::OnSeriesWon { then, .. } => then,
        }
    }
}

/// Selects printed cards by suit and/or rank. An empty filter matches every
/// normal card; blanks never match.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct CardFilter {
    /// Only this suit.
    pub suit: Option<Suit>,
    /// Only this rank.
    pub rank: Option<Rank>,
}

impl CardFilter {
    /// Whether a printed card matches.
    #[must_use]
    pub fn matches(&self, card: Card) -> bool {
        self.suit.is_none_or(|s| s == card.suit) && self.rank.is_none_or(|r| r == card.rank)
    }
}

/// What an effect does. Each variant names its lever of the formula.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Effect {
    /// Lever Grundwert: add to the base value of this game.
    AddBaseValue(i32),
    /// Lever Grundwert: multiply the base value of this game (Bock).
    MultiplyBaseValue(u32),
    /// Lever Grundwert: add the current value of a run counter (Tournee).
    AddBaseValueFromCounter(String),
    /// Lever Gewinnstufen: add levels to this game.
    AddLevels(i32),
    /// Lever Spielwert: multiply the whole game value (Kontra-like).
    MultiplyGameValue(u32),
    /// Lever Augen: every matching card counts this many Augen, for every
    /// seat (the deck is shared). Passive only.
    SetCardPoints {
        /// Which cards.
        filter: CardFilter,
        /// Their new Augen.
        points: u32,
    },
    /// Lever Augen: this card counts more (or fewer) Augen. Card
    /// modifications only, passive only.
    AddCardPoints(i32),
    /// Lever Stichgewinn: this card counts this many steps higher within
    /// its kind. Card modifications only, passive only.
    RankBonus(u8),
    /// Lever Zählen: the player sees who holds this card. Card
    /// modifications only, passive only.
    RevealHolder,
    /// Lever Spitzen: matching cards become blanks for this game (boss
    /// "Die Oma" takes out the jacks). Passive only.
    BlankOut(CardFilter),
    /// Lever Gewinnschwelle: the declarer wins from this many Augen. The
    /// lowest active threshold counts. Before the result only.
    WinThreshold(u32),
    /// Lever Schneider: a won game with declarer Augen in `min..=max`
    /// counts as Schneider. See [`crate::rules::Thresholds`] for why
    /// Schneider stays measured on the losing side. Before the result only.
    SchneiderFromDeclarerPoints {
        /// Lowest declarer Augen.
        min: u32,
        /// Highest declarer Augen.
        max: u32,
    },
    /// Lever Gewinnstufen (Hand): every game must be declared Hand.
    /// Passive or on deal.
    RequireHand,
    /// Lever Spielwert: the defenders may say Kontra (×2) after the
    /// declaration, the declarer may answer Re (×4). Passive or on deal.
    EnableKontra,
    /// Lever Spitzen and Risikostufe (hand quality, house rule
    /// "Schieben"): once per series the declarer may have all hands and the
    /// skat dealt again before declaring, without using up a game. He looks
    /// for a hand with more Spitzen or enough trumps for Hand. Passive or
    /// on deal.
    SeriesRedeal,
    /// Lever Schneider: the defenders give up; the remaining tricks and
    /// Augen go to the declarer. On trick end only.
    Concede,
    /// Run state: set a flag (e.g. "bock").
    SetFlag(String),
    /// Run state: clear a flag.
    ClearFlag(String),
    /// Run state: add one to a counter (e.g. "tournee").
    IncrementCounter(String),
}

impl Effect {
    /// Whether this effect is meaningful in `stage`.
    #[must_use]
    pub fn allowed_in(&self, stage: Stage) -> bool {
        match self {
            Effect::AddBaseValue(_)
            | Effect::MultiplyBaseValue(_)
            | Effect::AddBaseValueFromCounter(_)
            | Effect::AddLevels(_)
            | Effect::MultiplyGameValue(_) => stage.in_game(),
            Effect::SetCardPoints { .. }
            | Effect::AddCardPoints(_)
            | Effect::RankBonus(_)
            | Effect::RevealHolder
            | Effect::BlankOut(_) => stage == Stage::Passive,
            Effect::WinThreshold(_) | Effect::SchneiderFromDeclarerPoints { .. } => {
                stage.before_result()
            }
            Effect::RequireHand | Effect::EnableKontra | Effect::SeriesRedeal => {
                matches!(stage, Stage::Passive | Stage::OnDeal)
            }
            Effect::Concede => stage == Stage::OnTrickEnd,
            Effect::SetFlag(_) | Effect::ClearFlag(_) | Effect::IncrementCounter(_) => {
                stage != Stage::Passive
            }
        }
    }

    /// Whether this effect needs "this card" and so only works on card
    /// modifications.
    #[must_use]
    pub const fn card_mod_only(&self) -> bool {
        matches!(
            self,
            Effect::AddCardPoints(_) | Effect::RankBonus(_) | Effect::RevealHolder
        )
    }
}

/// When a hook fires. Conditions are evaluated against the run (flags,
/// counters), the current game and, for card modifications, "this card".
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum Condition {
    /// Always true.
    #[default]
    Always,
    /// All of the conditions.
    All(Vec<Condition>),
    /// Any of the conditions.
    Any(Vec<Condition>),
    /// The negation.
    Not(Box<Condition>),
    /// A run flag is set.
    Flag(String),
    /// A run counter has at least this value.
    CounterAtLeast {
        /// Counter name.
        counter: String,
        /// Minimum value.
        value: u32,
    },
    /// The declarer won the game. On score only.
    Won,
    /// The declarer lost the game. On score only.
    Lost,
    /// The declarer took every trick (Durchmarsch). On score only.
    AllTricksWon,
    /// The declared game is exactly this one. From the declaration on.
    GameIs(GameType),
    /// Any suit game. From the declaration on.
    SuitGame,
    /// The risk level is at least this one. From the declaration on.
    RiskAtLeast(RiskLevel),
    /// The tenth trick was won with this printed card, optionally only by
    /// the declarer. On score only.
    LastTrickWonWith {
        /// The winning card.
        card: Card,
        /// Whether the declarer must have won it.
        by_declarer: bool,
    },
    /// The tenth trick was won with a printed card that matches the
    /// filter, optionally only by the declarer. On score only.
    LastTrickWinnerMatches {
        /// Which winning cards count.
        filter: CardFilter,
        /// Whether the declarer must have won it.
        by_declarer: bool,
    },
    /// The declarer won and the defenders are Schneider (also through a
    /// declarer range such as "Jungfrau", by a concession or Schwarz). On
    /// score only.
    WonSchneider,
    /// This card won the tenth trick, whoever played it. Card
    /// modifications only, on score only.
    ThisCardWinsLastTrick,
    /// This card's suit is the trump suit of the declared suit game. Card
    /// modifications only, from the declaration on.
    ThisCardIsTrumpSuit,
    /// Exactly this many tricks are complete. During play.
    TricksPlayed(u32),
    /// The defenders together hold fewer Augen than this. During play.
    DefenderPointsBelow(u32),
    /// The card just played matches. On card played only.
    PlayedCardMatches(CardFilter),
    /// The card just played was played by the declarer. On card played
    /// only.
    PlayedByDeclarer,
    /// The won series had at least this many games left. On series won
    /// only.
    GamesLeftAtLeast(u32),
}

impl Condition {
    /// Whether this condition can be evaluated in `stage`.
    #[must_use]
    pub fn allowed_in(&self, stage: Stage) -> bool {
        match self {
            Condition::Always | Condition::Flag(_) | Condition::CounterAtLeast { .. } => true,
            Condition::All(cs) | Condition::Any(cs) => cs.iter().all(|c| c.allowed_in(stage)),
            Condition::Not(c) => c.allowed_in(stage),
            Condition::Won
            | Condition::Lost
            | Condition::AllTricksWon
            | Condition::LastTrickWonWith { .. }
            | Condition::LastTrickWinnerMatches { .. }
            | Condition::WonSchneider
            | Condition::ThisCardWinsLastTrick => stage == Stage::OnScore,
            Condition::GameIs(_)
            | Condition::SuitGame
            | Condition::RiskAtLeast(_)
            | Condition::ThisCardIsTrumpSuit => matches!(
                stage,
                Stage::OnDeclare | Stage::OnCardPlayed | Stage::OnTrickEnd | Stage::OnScore
            ),
            Condition::TricksPlayed(_) | Condition::DefenderPointsBelow(_) => {
                matches!(stage, Stage::OnCardPlayed | Stage::OnTrickEnd)
            }
            Condition::PlayedCardMatches(_) | Condition::PlayedByDeclarer => {
                stage == Stage::OnCardPlayed
            }
            Condition::GamesLeftAtLeast(_) => stage == Stage::OnSeriesWon,
        }
    }

    /// Whether this condition needs "this card".
    #[must_use]
    pub fn card_mod_only(&self) -> bool {
        match self {
            Condition::All(cs) | Condition::Any(cs) => cs.iter().any(Condition::card_mod_only),
            Condition::Not(c) => c.card_mod_only(),
            Condition::ThisCardWinsLastTrick | Condition::ThisCardIsTrumpSuit => true,
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hooks_parse_from_ron() {
        let hook: Hook = ron::from_str(
            r#"OnScore(when: All([Lost, Not(Flag("bock"))]), then: [SetFlag("bock"), AddLevels(1)])"#,
        )
        .expect("ron");
        assert_eq!(hook.stage(), Stage::OnScore);
        assert_eq!(hook.then().len(), 2);
        let hook: Hook = ron::from_str("OnDeal(then: [RequireHand])").expect("ron");
        assert_eq!(hook.when(), &Condition::Always);
        let filter: CardFilter = ron::from_str("(rank: Some(Queen))").expect("ron");
        assert!(filter.matches("HQ".parse().expect("card")));
        assert!(!filter.matches("HK".parse().expect("card")));
        let cond: Condition =
            ron::from_str(r#"LastTrickWonWith(card: "D7", by_declarer: true)"#).expect("ron");
        assert!(cond.allowed_in(Stage::OnScore));
        assert!(!cond.allowed_in(Stage::OnDeal));
    }

    #[test]
    fn stage_rules_reject_misplaced_effects() {
        assert!(Effect::AddLevels(1).allowed_in(Stage::OnScore));
        assert!(!Effect::AddLevels(1).allowed_in(Stage::OnSeriesWon));
        assert!(!Effect::WinThreshold(58).allowed_in(Stage::OnScore));
        assert!(Effect::WinThreshold(58).allowed_in(Stage::Passive));
        assert!(!Effect::AddCardPoints(4).allowed_in(Stage::OnDeal));
        assert!(Effect::Concede.allowed_in(Stage::OnTrickEnd));
        assert!(!Effect::SetFlag("x".into()).allowed_in(Stage::Passive));
        assert!(Effect::RankBonus(1).card_mod_only());
        assert!(Condition::Not(Box::new(Condition::ThisCardIsTrumpSuit)).card_mod_only());
    }
}

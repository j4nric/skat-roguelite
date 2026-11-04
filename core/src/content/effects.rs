//! The hook API: when an effect fires ([`Hook`]), under which condition
//! ([`Condition`]) and what it does ([`Effect`]). All three are closed enums
//! with data, written in RON by content authors. There is no scripting.
//!
//! Design pillar 1 ("Die Formel ist das Spiel"): every effect names the
//! lever of the game value formula it turns in its doc comment.

use serde::{Deserialize, Serialize};

use super::{CardModId, Factor};
use crate::cards::{Card, Rank, Suit};
use crate::rules::{GameType, NullRisk, RiskLevel};

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
    /// When a card modification is attached to a card (no game in
    /// progress). Card modifications only.
    OnAttach,
    /// When the card it sits on lands in the skat by discarding ("wenn
    /// gedrückt", vouchers, DESIGN 7.2). Acts outside the game in
    /// progress: only run state (Groschen, the deck). Card modifications
    /// only.
    OnDiscard,
    /// When the player takes a route to the next pub (no game in
    /// progress, ADR 0016). Condition [`Condition::TransportIs`].
    OnRouteChosen,
    /// When the Tresen opens (no game in progress).
    OnShopOpened,
}

impl Stage {
    /// Whether a game is in progress at this stage.
    #[must_use]
    pub const fn in_game(self) -> bool {
        !matches!(
            self,
            Stage::OnSeriesWon
                | Stage::OnAttach
                | Stage::OnDiscard
                | Stage::OnRouteChosen
                | Stage::OnShopOpened
        )
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
    /// When the card modification is attached to a card ("beim Erhalt").
    /// Card modifications only.
    OnAttach {
        /// Condition.
        #[serde(default)]
        when: Condition,
        /// Effects, applied in order.
        then: Vec<Effect>,
    },
    /// When the card is discarded into the skat ("wenn gedrückt").
    /// Card modifications only.
    OnDiscard {
        /// Condition.
        #[serde(default)]
        when: Condition,
        /// Effects, applied in order.
        then: Vec<Effect>,
    },
    /// When the player takes a route to the next pub.
    OnRouteChosen {
        /// Condition.
        #[serde(default)]
        when: Condition,
        /// Effects, applied in order.
        then: Vec<Effect>,
    },
    /// When the Tresen opens.
    OnShopOpened {
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
            Hook::OnAttach { .. } => Stage::OnAttach,
            Hook::OnDiscard { .. } => Stage::OnDiscard,
            Hook::OnRouteChosen { .. } => Stage::OnRouteChosen,
            Hook::OnShopOpened { .. } => Stage::OnShopOpened,
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
            | Hook::OnSeriesWon { when, .. }
            | Hook::OnAttach { when, .. }
            | Hook::OnDiscard { when, .. }
            | Hook::OnRouteChosen { when, .. }
            | Hook::OnShopOpened { when, .. } => when,
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
            | Hook::OnSeriesWon { then, .. }
            | Hook::OnAttach { then, .. }
            | Hook::OnDiscard { then, .. }
            | Hook::OnRouteChosen { then, .. }
            | Hook::OnShopOpened { then, .. } => then,
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
    /// Lever Spielwert: multiply the whole game value by a factor that
    /// need not be whole, written as a decimal (`ScaleGameValue(1.5)`,
    /// "Stammfarbe"). The Spielwert is rounded half up at the end.
    ScaleGameValue(Factor),
    /// Lever Grundwert: add `per` times the current value of a run counter
    /// ("Allrounder": +3 per Spielart of the series).
    AddBaseValuePerCounter {
        /// Counter name.
        counter: String,
        /// Grundwert per point of the counter.
        per: i32,
    },
    /// Lever Grundwert: add this much per card of the player's edition that
    /// carries a curse when the effect fires, a special curse such as
    /// "Abgestuft" and a curse laid for one pub included ("Pechvogel",
    /// 03.11.2026). Scaled like [`Effect::AddBaseValue`].
    AddBaseValuePerCursedCard(i32),
    /// Lever Gewinnstufen: add `per` levels per point of a run counter
    /// ("Skatgott": +1 per won Schneider game, for the rest of the run).
    AddLevelsPerCounter {
        /// Counter name.
        counter: String,
        /// Gewinnstufen per point of the counter.
        per: i32,
    },
    /// Lever Augen: the two discarded cards in the skat count this many
    /// times their Augen for the declarer ("Skatpfleger": 2). Only in a
    /// game with the skat picked up; nothing in a hand game. On declare
    /// only (the discard is final then).
    MultiplySkatPoints(u32),
    /// Lever Schneider: the Augen of the trick just completed count once
    /// more for the Schneider line, not for winning ("Stichfest": the
    /// declarer's first trick, see [`crate::rules::Thresholds`]
    /// `schneider_bonus`). On trick end only.
    SchneiderBonusFromTrick,
    /// Lever Augen: every matching card counts this many Augen. From a
    /// boss or malus in every edition; from a relic or the character only
    /// in the player's edition (`engine::owns_card_points`, 04.11.2026).
    /// Passive only.
    SetCardPoints {
        /// Which cards.
        filter: CardFilter,
        /// Their new Augen.
        points: u32,
    },
    /// Lever Augen: this card counts more (or fewer) Augen, also below
    /// zero (DESIGN 7.2, [`crate::deck::DealtCard::point_debt`]). Card
    /// modifications only, passive only.
    AddCardPoints(i32),
    /// Lever Stichgewinn: this card counts this many steps higher within
    /// its kind. Card modifications only, passive only. No content uses it
    /// since "Winkel" became [`Effect::Promote`] (03.11.2026).
    RankBonus(u8),
    /// Lever Stichgewinn ("Winkel", DESIGN 7.2, 03.11.2026): this card
    /// becomes the card this many steps higher in its suit (7, 8, 9, Ober,
    /// König, 10, Daus; [`crate::cards::Rank::promoted`]) in every game and
    /// keeps its own Augen. A Unter never rises and is never reached, a
    /// Daus stays. Card modifications only, passive only.
    Promote(u8),
    /// Lever Stichgewinn (Null, "Blei", DESIGN 7.2): in a Null game this
    /// card is the lowest card of its suit, below the Seven
    /// ([`crate::rules::suit_strength`]). Card modifications only, passive
    /// only.
    NullLowest,
    /// Lever Zählen: the player sees who holds this card. Card
    /// modifications only, passive only.
    RevealHolder,
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
    /// "Schieben"): one Einmischen more in every game, so the declarer may
    /// have all hands and the skat dealt again once more before declaring.
    /// He looks for a hand with more Spitzen or enough trumps for Hand.
    /// Each source adds one. Passive or on deal. Also read under its old
    /// name `SeriesRedeal` (once per series).
    #[serde(alias = "SeriesRedeal")]
    ExtraRedeal,
    /// Lever Schneider: the defenders give up; the remaining tricks and
    /// Augen go to the declarer. On trick end only.
    Concede,
    /// Run state: set a flag (e.g. "bock").
    SetFlag(String),
    /// Run state: clear a flag.
    ClearFlag(String),
    /// Run state: add one to a counter (e.g. "tournee").
    IncrementCounter(String),
    /// Run state: set a counter back to zero (counters of one series,
    /// e.g. "Altes Blatt" after a won series).
    ResetCounter(String),
    /// Lever Stichgewinn (Null): this card can never take a trick, whoever
    /// plays it ("Unsichtbar"). It keeps its suit and follows suit. Card
    /// modifications only, passive only.
    NeverWins,
    /// Lever Zählen: before the declaration the player sees this many
    /// random cards of the defenders and who holds them (figure "Der
    /// Kiebitz"). An information effect. Passive or on deal.
    PeekDefenderCards(u32),
    /// Lever Zählen: the player sees the whole hand of one defender, the
    /// seat by index (1: the left neighbour, who plays after him; 2: the
    /// right one), for the whole game (a character rule; "Der Zaungast" until 03.11.2026). An
    /// information effect. Passive or on deal.
    RevealDefenderHand(u8),
    /// Lever Zählen: the player sees the Augen taken so far by each side
    /// and the Augen he has not seen yet (figure "Der Zähler"). An
    /// information effect. Passive only.
    ShowCount,
    /// Lever Groschen: prices at the Tresen (and the Neuauslage) drop by
    /// this many percent, rounded to the nearest Groschen, at least 1
    /// (figure "Der Wirt"). Passive only; several add up to at most 90.
    ShopDiscount(u32),
    /// Lever Spitzen and Hand: once per series, in a hand game before the
    /// first card, the player may take a card matching the filter from the
    /// untouched skat into his hand and put one of his cards in its place.
    /// The game stays a hand game (figure "Die Oma", jacks; "Der Opa",
    /// aces). Whether such a card lies in the skat is what the figure
    /// tells. Several of them add up: each grants its own swap per series.
    /// Passive only.
    SeriesSkatSwap(CardFilter),
    /// Boss rules marked as cheating ([`Effect::is_cheat`]) do not apply
    /// (figure "Der Schiedsrichter"). Passive only.
    NoCheating,
    /// Lever Gewinnstufen (Ouvert): every game must be declared Ouvert
    /// (figure "Der Angeber"). Passive or on deal.
    RequireOuvert,
    /// Lever Stichgewinn: the defenders see each other's hands and play
    /// with that knowledge (boss "Die Zwillinge"). Cheating. Passive or on
    /// deal.
    DefendersSeeEachOther,
    /// Own scoring: every game dealt while this rule is active is a
    /// Ramsch, declared by nobody and not to be passed (boss "Der
    /// Ramscher" since ADR 0017; until then only the first game of the
    /// pub, which is why saves call it `RamschOnce`; see
    /// [`crate::rules::ramsch_value_parts`]). Passive only.
    #[serde(alias = "RamschOnce")]
    Ramsch,
    /// Lever Spielwert: a lost game counts once against the player instead
    /// of twice, as "Magenbitter" did (boss "Der Ramscher" since
    /// 03.11.2026: a lost Ramsch in his pub). Passive or on deal, so it
    /// holds for every game under the rule.
    SingleLoss,
    /// Lever Flüche (boss "Die Oma", ADR 0017): when the first game under
    /// this rule is dealt, `cards` random cards of the player's edition
    /// each get one of `curses` (drawn per card, with the curse's default
    /// value) until the pub ends. A temporary curse takes the place of the
    /// card's normal modification while it lasts; afterwards the edition
    /// is exactly as before. No Groschen, no `OnAttach`. Passive only.
    CurseForPub {
        /// How many cards of the player's edition.
        cards: u32,
        /// The curses drawn from, one per card.
        curses: Vec<CardModId>,
    },
    /// Lever Groschen: the player gains this many Groschen (fewer if
    /// negative, never below 0). In every stage but `Passive`: on score,
    /// after a won series, or on a card modification when it is attached
    /// ("Ramponiert": a curse that pays).
    GainCoins(i32),
    /// Lever Gewinnschwelle (curse "Ramponiert"): this card must not be
    /// discarded while two other cards could go into the skat (see
    /// [`crate::rules::valid_discard`]). Card modifications only, passive
    /// only.
    NoDiscard,
    /// Lever Augen ("Erster Stich", DESIGN 7.2): the Augen of the trick
    /// just completed count this many times in total for the seat that
    /// took it (1: unchanged, 2: twice). The extra goes to
    /// [`crate::game::Game::add_bonus_points`]. On trick end only.
    MultiplyTrickPoints(u32),
    /// Deck ("Gutschein", DESIGN 7.2): this card becomes the named special
    /// card (Untot, Wandelkarte) from the next game on. Card
    /// modifications only, when discarded only.
    TurnInto(CardModId),
    /// Lever Stichgewinn and Bedienpflicht ("Wandelkarte"): in the player's
    /// hand this card takes a suit when the game starts, the one of which
    /// he holds the most cards (see [`crate::rules::settled_wild_suit`]),
    /// and is that card of the same rank until the end of the game. In a
    /// defender's hand it is the printed card. Card modifications only,
    /// passive only.
    Wild,
    /// Lever Stichgewinn ("Abgestuft", DESIGN 9.2): a Unter is no Unter
    /// any more but the lowest card of its suit, with no Augen; every other
    /// card counts one step lower within its kind (see
    /// [`crate::rules::is_demoted_jack`]). Card modifications only, passive
    /// only.
    Demote,
    /// Lever Stichgewinn and Augen ("Untot", DESIGN 7.2): when the player
    /// plays this card in a suit game or Grand and the trick is complete,
    /// the card comes back into his hand once per game and a random other
    /// card of his hand takes its place in the trick (the trick winner
    /// stays, the substitute's Augen count). Not with his last card, not in
    /// Null or Ramsch, not for a defender's copy. Card modifications only,
    /// passive only.
    Undead,
    /// Lever Stichgewinn ("Doppelgänger", DESIGN 7.2, 03.11.2026): when the
    /// player plays this card (a legal play as printed) in a suit game,
    /// Grand or Null, it takes a card of the same rank and another suit
    /// from the next opponent in play order, else from the other one (a
    /// random one of several); that card lies in the trick instead, and the
    /// robbed opponent holds this card for the rest of the game. Without a
    /// match it is played as itself. Card modifications only, passive only.
    Doppelganger,
    /// Lever Stichgewinn (Vorhand): the player leads the first trick of
    /// every game ("Ehrenvorsitz"). Passive only.
    AlwaysForehand,
    /// Lever Stichgewinn and Bedienpflicht ("Bube ehrenhalber"): in a suit
    /// game this card is a trump half a step below the Karo-Bube, follows
    /// trump leads and no longer its printed suit. It is no Spitze. Grand,
    /// Null and Ramsch keep the printed card (DESIGN 4.4). Card
    /// modifications only, passive only.
    HonoraryJack,
    /// Lever Augen and Spitzen ("Drei drücken", DESIGN 8.1): in the first
    /// game of each pub (since 02.11.2026), when he picks up the skat, the
    /// player also draws a
    /// random card from the hand of the left defender, holds thirteen
    /// cards and discards three. The lowest
    /// of the three goes back to that defender
    /// ([`crate::rules::returned_discard`]), the other two into the skat.
    /// Several sources still draw one card. Passive only.
    DiscardThree,
    /// Lever Zählen ("Spickzettel"): in a hand game, after the
    /// declaration, the player sees this many cards of the untouched skat.
    /// An information effect. On declare only; nothing once the skat is
    /// picked up.
    RevealSkatCard(u32),
    /// Lever Groschen (savings, "Stammtischkasse"): move up to `amount`
    /// of the player's Groschen into the run counter `counter` (a pot).
    /// After a won series only, so it takes from the pub's reward.
    StashCoins {
        /// The pot.
        counter: String,
        /// Groschen moved at most.
        amount: u32,
    },
    /// Lever Groschen (savings, "Stammtischkasse"): pay the pot `counter`
    /// out at `percent` (150: half on top, rounded down) and empty it. When
    /// the Tresen opens only.
    PayOutCounter {
        /// The pot.
        counter: String,
        /// Percent of the pot paid.
        percent: u32,
    },
    /// Lever Groschen (credit, "Deckel anschreiben"): once per Stadtteil the
    /// Tresen sells down to minus this many Groschen; later income pays the
    /// debt first. Passive only; the largest limit counts.
    Credit(u32),
    /// Lever Leben ("Schutzengel"): once per Stadtteil the first lost pub
    /// costs no life; never a boss pub, whose loss ends the run (ADR
    /// 0017). Passive only.
    SpareLife,
    /// Lever Spelunke ("Spelunkenkenner"): a Spelunke offers this many
    /// maluses and the player picks one at the route choice. Passive only;
    /// the largest number counts.
    ChooseMalus(u32),
}

impl Effect {
    /// Whether this effect is meaningful in `stage`.
    #[must_use]
    pub fn allowed_in(&self, stage: Stage) -> bool {
        match self {
            Effect::AddBaseValue(_)
            | Effect::MultiplyBaseValue(_)
            | Effect::AddBaseValueFromCounter(_)
            | Effect::AddBaseValuePerCounter { .. }
            | Effect::AddBaseValuePerCursedCard(_)
            | Effect::AddLevelsPerCounter { .. }
            | Effect::AddLevels(_)
            | Effect::MultiplyGameValue(_)
            | Effect::ScaleGameValue(_) => stage.in_game(),
            Effect::MultiplySkatPoints(_) | Effect::RevealSkatCard(_) => stage == Stage::OnDeclare,
            Effect::SchneiderBonusFromTrick | Effect::MultiplyTrickPoints(_) => {
                stage == Stage::OnTrickEnd
            }
            Effect::TurnInto(_) => stage == Stage::OnDiscard,
            Effect::SetCardPoints { .. }
            | Effect::AddCardPoints(_)
            | Effect::RankBonus(_)
            | Effect::Promote(_)
            | Effect::NullLowest
            | Effect::RevealHolder
            | Effect::Wild
            | Effect::Demote
            | Effect::Undead
            | Effect::Doppelganger
            | Effect::AlwaysForehand => stage == Stage::Passive,
            Effect::WinThreshold(_) | Effect::SchneiderFromDeclarerPoints { .. } => {
                stage.before_result()
            }
            Effect::RequireHand
            | Effect::EnableKontra
            | Effect::ExtraRedeal
            | Effect::SingleLoss => {
                matches!(stage, Stage::Passive | Stage::OnDeal)
            }
            Effect::Concede => stage == Stage::OnTrickEnd,
            Effect::SetFlag(_)
            | Effect::ClearFlag(_)
            | Effect::IncrementCounter(_)
            | Effect::ResetCounter(_)
            | Effect::GainCoins(_) => stage != Stage::Passive,
            Effect::NeverWins
            | Effect::NoDiscard
            | Effect::HonoraryJack
            | Effect::DiscardThree
            | Effect::Credit(_)
            | Effect::SpareLife
            | Effect::ChooseMalus(_)
            | Effect::ShowCount
            | Effect::ShopDiscount(_)
            | Effect::SeriesSkatSwap(_)
            | Effect::NoCheating
            | Effect::Ramsch
            | Effect::CurseForPub { .. } => stage == Stage::Passive,
            Effect::PeekDefenderCards(_)
            | Effect::RevealDefenderHand(_)
            | Effect::RequireOuvert
            | Effect::DefendersSeeEachOther => matches!(stage, Stage::Passive | Stage::OnDeal),
            Effect::StashCoins { .. } => stage == Stage::OnSeriesWon,
            Effect::PayOutCounter { .. } => stage == Stage::OnShopOpened,
        }
    }

    /// Whether this effect gives the player information.
    #[must_use]
    pub const fn is_information(&self) -> bool {
        matches!(
            self,
            Effect::RevealHolder
                | Effect::PeekDefenderCards(_)
                | Effect::RevealDefenderHand(_)
                | Effect::ShowCount
                | Effect::RevealSkatCard(_)
        )
    }

    /// Whether this effect is a boss rule that counts as cheating
    /// (switched off by [`Effect::NoCheating`]).
    #[must_use]
    pub const fn is_cheat(&self) -> bool {
        matches!(self, Effect::DefendersSeeEachOther)
    }

    /// Whether this effect needs "this card" and so only works on card
    /// modifications.
    #[must_use]
    pub const fn card_mod_only(&self) -> bool {
        matches!(
            self,
            Effect::AddCardPoints(_)
                | Effect::RankBonus(_)
                | Effect::Promote(_)
                | Effect::NullLowest
                | Effect::RevealHolder
                | Effect::NeverWins
                | Effect::NoDiscard
                | Effect::HonoraryJack
                | Effect::Wild
                | Effect::Demote
                | Effect::TurnInto(_)
                | Effect::Undead
                | Effect::Doppelganger
        )
    }

    /// This effect for a card modification of `value` (DESIGN 7.2): the
    /// effects of a card modification are written per unit of its value
    /// and multiplied here. Scaled: `AddCardPoints`, `RankBonus` and
    /// `Promote` (at most 255), `AddBaseValue`, `AddBaseValuePerCursedCard`, `AddLevels`, `GainCoins` and
    /// `MultiplyTrickPoints` (the factor itself is the value: one unit of
    /// "Erster Stich" with value 2 is ×2). Every other effect stays as it
    /// is. A value of 1 changes nothing.
    #[must_use]
    pub fn scaled(&self, value: i32) -> Effect {
        let times = |n: i32| n.saturating_mul(value);
        match self {
            Effect::AddCardPoints(n) => Effect::AddCardPoints(times(*n)),
            Effect::AddBaseValue(n) => Effect::AddBaseValue(times(*n)),
            Effect::AddBaseValuePerCursedCard(n) => Effect::AddBaseValuePerCursedCard(times(*n)),
            Effect::AddLevels(n) => Effect::AddLevels(times(*n)),
            Effect::GainCoins(n) => Effect::GainCoins(times(*n)),
            Effect::RankBonus(n) => {
                Effect::RankBonus(u8::try_from(times(i32::from(*n)).clamp(0, 255)).unwrap_or(0))
            }
            Effect::Promote(n) => {
                Effect::Promote(u8::try_from(times(i32::from(*n)).clamp(0, 255)).unwrap_or(0))
            }
            Effect::MultiplyTrickPoints(n) => Effect::MultiplyTrickPoints(
                u32::try_from(i64::from(*n) * i64::from(value.max(0))).unwrap_or(u32::MAX),
            ),
            other => other.clone(),
        }
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
    /// The risk level is at least this one: `Hand` only from the hand, an
    /// announcement also after the skat (ADR 0013,
    /// [`crate::rules::Declaration::risk_at_least`]). From the declaration
    /// on.
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
    /// This card's suit is the trump suit of the declared suit game and,
    /// while [`super::engine::TRUMP_MODS_NEED_PLAYER`] is on (ADR 0015
    /// strict, DESIGN 7.2), the player holds the card: in his hand, in the
    /// skat or played by him. Card modifications only, from the
    /// declaration on.
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
    /// The trick just completed was won with a printed card that matches
    /// the filter, optionally only by the declarer ("Luschenkönig": a
    /// seven of the declarer). On trick end only.
    TrickWinnerMatches {
        /// Which winning cards count.
        filter: CardFilter,
        /// Whether the declarer must have won it.
        by_declarer: bool,
    },
    /// The trick just completed is the first one the declarer took
    /// ("Stichfest"). On trick end only.
    FirstTrickOfDeclarer,
    /// The trick just completed is the first trick of the game and this
    /// card won it, whoever played it ("Erster Stich"). Card modifications
    /// only, on trick end only.
    ThisCardWinsFirstTrick,
    /// This card won the trick just completed ("Goldkante"). Card
    /// modifications only, on trick end only.
    ThisCardWinsTrick,
    /// This card won the trick just completed and a defender played a
    /// card matching the filter into it ("Fadenkreuz": a Daus of an
    /// opponent). Card modifications only, on trick end only.
    ThisCardBeatsDefenderCard(CardFilter),
    /// The player played this card into a completed trick of this game
    /// ("Schneiderkreide"). Card modifications only, during play and on
    /// score.
    ThisCardPlayed,
    /// The won series had at least this many games left. On series won
    /// only.
    GamesLeftAtLeast(u32),
    /// The route just taken goes by this means of transport. On route
    /// chosen only.
    TransportIs(crate::run::tour::Transport),
    /// A Null game. From the declaration on.
    NullGame,
    /// A Null game with at least this Null-Risikostufe (order Normal,
    /// Hand, Ouvert, Ouvert Hand). From the declaration on.
    NullRiskAtLeast(NullRisk),
    /// A Ramsch. From the declaration on.
    RamschGame,
    /// The skat stays on the table: Hand and above, Null Hand, Null Ouvert
    /// Hand, Ramsch. From the declaration on.
    HandGame,
    /// The declarer took no trick yet (in a won Null game: none at all).
    /// During play and on score.
    NoTrickTaken,
    /// Null style: a Null game in which the declarer has played a seven
    /// in a trick he did not take, at least once so far ("mit einer Sieben
    /// unterlaufen", see [`crate::rules::NullStyle`]). During play and on
    /// score.
    SevenDucked,
    /// Null style: a Null game in which the declarer has never had to
    /// follow suit to another seat's lead so far. During play and on
    /// score.
    NeverFollowedSuit,
    /// The declarer's Spitzen are "mit" (`with: true`) or "ohne" and at
    /// least `count` (suit games and grand; never in Null or Ramsch). From
    /// the declaration on.
    MatadorsAtLeast {
        /// "mit" or "ohne".
        with: bool,
        /// Minimum number of Spitzen.
        count: u32,
    },
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
            Condition::TrickWinnerMatches { .. }
            | Condition::FirstTrickOfDeclarer
            | Condition::ThisCardWinsFirstTrick
            | Condition::ThisCardWinsTrick
            | Condition::ThisCardBeatsDefenderCard(_) => stage == Stage::OnTrickEnd,
            Condition::ThisCardPlayed => matches!(
                stage,
                Stage::OnCardPlayed | Stage::OnTrickEnd | Stage::OnScore
            ),
            Condition::GamesLeftAtLeast(_) => stage == Stage::OnSeriesWon,
            Condition::TransportIs(_) => stage == Stage::OnRouteChosen,
            Condition::NullGame
            | Condition::NullRiskAtLeast(_)
            | Condition::RamschGame
            | Condition::HandGame => matches!(
                stage,
                Stage::OnDeclare | Stage::OnCardPlayed | Stage::OnTrickEnd | Stage::OnScore
            ),
            Condition::NoTrickTaken | Condition::SevenDucked | Condition::NeverFollowedSuit => {
                matches!(
                    stage,
                    Stage::OnCardPlayed | Stage::OnTrickEnd | Stage::OnScore
                )
            }
            Condition::MatadorsAtLeast { .. } => matches!(
                stage,
                Stage::OnDeclare | Stage::OnCardPlayed | Stage::OnTrickEnd | Stage::OnScore
            ),
        }
    }

    /// Whether this condition needs "this card".
    #[must_use]
    pub fn card_mod_only(&self) -> bool {
        match self {
            Condition::All(cs) | Condition::Any(cs) => cs.iter().any(Condition::card_mod_only),
            Condition::Not(c) => c.card_mod_only(),
            Condition::ThisCardWinsLastTrick
            | Condition::ThisCardIsTrumpSuit
            | Condition::ThisCardWinsFirstTrick
            | Condition::ThisCardWinsTrick
            | Condition::ThisCardBeatsDefenderCard(_)
            | Condition::ThisCardPlayed => true,
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
        assert!(Effect::MultiplySkatPoints(2).allowed_in(Stage::OnDeclare));
        assert!(!Effect::MultiplySkatPoints(2).allowed_in(Stage::Passive));
        assert!(Effect::SchneiderBonusFromTrick.allowed_in(Stage::OnTrickEnd));
        assert!(!Effect::SchneiderBonusFromTrick.allowed_in(Stage::OnScore));
        assert!(!Effect::ResetCounter("x".into()).allowed_in(Stage::Passive));
        assert!(Condition::FirstTrickOfDeclarer.allowed_in(Stage::OnTrickEnd));
        assert!(!Condition::FirstTrickOfDeclarer.allowed_in(Stage::OnScore));
    }

    #[test]
    fn wave_two_effects_parse_from_ron() {
        let hook: Hook = ron::from_str(
            r#"OnDeclare(when: SuitGame, then: [ScaleGameValue(1.5), AddBaseValuePerCounter(counter: "all_rounder", per: 3)])"#,
        )
        .expect("ron");
        assert_eq!(
            hook.then()[0],
            Effect::ScaleGameValue(Factor::from_permille(1500))
        );
        let cond: Condition =
            ron::from_str("TrickWinnerMatches(filter: (rank: Some(Seven)), by_declarer: true)")
                .expect("ron");
        assert!(cond.allowed_in(Stage::OnTrickEnd));
    }
}

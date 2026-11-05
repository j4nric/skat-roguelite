//! Data-driven content and the effect hook API (design document 14.2).
//!
//! House rules, card modifications, bosses, consumables, characters,
//! encounters (Ereignisse), unlocks (Freischaltungen) and the run
//! configuration live as RON files in `core/content/`. They are embedded
//! into the binary with `include_str!` and parsed by [`Content::load`], so
//! the core never touches the file system at runtime (WebAssembly safe).
//!
//! Every content item is validated on load: each effect and condition must
//! be allowed in the stage it is used in, and effects that need "this
//! card" only appear on card modifications.

mod effects;
mod engine;
mod score;

use std::collections::BTreeMap;
use std::fmt;

use serde::{Deserialize, Serialize};

use crate::cards::Card;
use crate::rules::{GameType, NullRisk, NullScoring, RiskLevel};

pub use effects::{CardFilter, Condition, Effect, Hook, Stage};
pub use engine::{
    ActiveSource, CardModStatus, Editions, HookContext, ResolvedDeck, TRUMP_MODS_NEED_PLAYER,
    card_mod_sources, card_mod_status, evaluate, evaluate_if_won, evaluate_known, held_by_player,
    owns_card_points, resolve_edition,
};
pub use score::{
    Change, Lever, RUN_LOSS_FACTOR, ScoreBreakdown, ScoreLine, ScoreStep, ScoreTerm, Source,
    score_game, score_parts,
};

macro_rules! content_id {
    ($(#[$meta:meta])* $name:ident) => {
        $(#[$meta])*
        #[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
        #[serde(transparent)]
        pub struct $name(pub String);

        impl $name {
            /// Create an id from its text.
            #[must_use]
            pub fn new(id: &str) -> $name {
                $name(id.to_string())
            }

            /// The id as text.
            #[must_use]
            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(&self.0)
            }
        }
    };
}

mod tour;

pub use tour::{
    CityDistrict, CityNames, DistrictLook, DistrictName, MalusCurse, MalusDef, MalusId,
    NamePattern, TourContent,
};

content_id!(
    /// Id of a relic (house rule or Stammtisch figure).
    RelicId
);
content_id!(
    /// Id of a card modification (gezinkte Karte).
    CardModId
);
content_id!(
    /// Id of a boss.
    BossId
);
content_id!(
    /// Id of a consumable.
    ConsumableId
);
content_id!(
    /// Id of an encounter (Ereignis on the way between two pubs).
    EncounterId
);
content_id!(
    /// Id of an unlock (Freischaltung).
    UnlockId
);
content_id!(
    /// Id of a Stammgast at the table ([`GuestDef`]).
    GuestId
);

/// What a Stammgast says in the speech bubble (one short line each).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GuestQuips {
    /// When the guest sits down (first game of a series).
    pub greet: String,
    /// When the guest takes a trick.
    pub win: String,
    /// When the player takes a trick.
    pub lose: String,
}

/// A Stammgast (DESIGN 10): a normal opponent with a name and a pixel
/// face, without rules of his own. Two sit at the table each series
/// ([`crate::run::Run::guests`]). Pure presentation: no effect on play or AI.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GuestDef {
    /// Content id.
    pub id: GuestId,
    /// German display name.
    pub name: String,
    /// Id of the face the frontend draws.
    pub look: String,
    /// Speech bubble lines.
    pub quips: GuestQuips,
    /// Card back of his cards (ADR 0015); the shared opponent back
    /// `"opponent"` if not set.
    #[serde(default = "opponent_back")]
    pub back: String,
}

/// The card back every opponent shows unless content names another one.
pub const OPPONENT_BACK: &str = "opponent";

fn opponent_back() -> String {
    OPPONENT_BACK.to_owned()
}

/// Charakter: who the player is in a run (DESIGN 9). Every character may
/// declare every game, suit games, grand and Null (ADR 0012); what sets
/// them apart is data in `characters.ron` ([`CharacterDef`]): start deck,
/// start relics, Zielwerte, Null scoring and which relics the Tresen
/// offers more often.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Default, Serialize, Deserialize,
)]
pub enum Character {
    /// Der Alleinspieler: the standard character and the measure for all
    /// balancing.
    #[default]
    Declarer,
    /// Der Nullspieler: the unlockable Null specialist (ADR 0012, which
    /// replaces ADR 0003). He may declare every game too.
    NullPlayer,
    /// Der Drückeberger (DESIGN 9.4): vouchers on his edition, redeemed
    /// vouchers come back after each pub, hand games get no Hand level.
    Shirker,
    /// Die Schneiderin (DESIGN 9.5): Schneider and Schwarz count two
    /// levels each, a win without Schneider one level less.
    Tailoress,
    /// Bruderherz (DESIGN 9.6): his Rot cards are strong, the other
    /// suits cursed; his curses cannot be cleansed, Zinken on Rot cost
    /// half.
    HeartBrother,
}

impl Character {
    /// All characters in display order.
    pub const ALL: [Character; 5] = [
        Character::Declarer,
        Character::NullPlayer,
        Character::Shirker,
        Character::Tailoress,
        Character::HeartBrother,
    ];

    /// The id as text, equal to its serialized form.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Character::Declarer => "Declarer",
            Character::NullPlayer => "NullPlayer",
            Character::Shirker => "Shirker",
            Character::Tailoress => "Tailoress",
            Character::HeartBrother => "HeartBrother",
        }
    }

    /// The character with this id, `None` for an unknown one (a removed
    /// character of an old save: "GrandPlayer", "Onlooker", "Landlady").
    #[must_use]
    pub fn from_id(id: &str) -> Option<Character> {
        Character::ALL.into_iter().find(|c| c.as_str() == id)
    }
}

impl fmt::Display for Character {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// How long a run is. Since the Kneipentour (ADR 0016) there is only the
/// whole evening (three Stadtteile); the short evening (kurzer Abend,
/// DESIGN 5.3 until 02.11.2026) is dropped for now. The value stays so
/// that old meta states (statistics and stakes per length) still load; a
/// run cannot start as `Short` any more.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Default, Serialize, Deserialize,
)]
pub enum RunLength {
    /// Ganzer Abend: every Stadtteil. Saves and links from before the
    /// short evening are this.
    #[default]
    Full,
    /// Kurzer Abend (until ADR 0016): only read from old meta states.
    Short,
}

impl RunLength {
    /// The id as text, equal to its serialized form.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            RunLength::Full => "Full",
            RunLength::Short => "Short",
        }
    }
}

/// Which Spielarten a relic or card modification is made for (ADR 0012):
/// every game, the trump games (suit games and grand) or Null. It decides
/// how often the Tresen offers the item to a character
/// ([`CharacterDef::focus_weights`]); what the item does stays in its
/// effects and conditions.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Default, Serialize, Deserialize,
)]
pub enum GameFocus {
    /// Every Spielart.
    #[default]
    All,
    /// Suit games and grand (Trumpfspiele).
    Trump,
    /// Null.
    Null,
}

/// How often the Tresen offers items of each [`GameFocus`] to a character,
/// in percent of the rarity weight. Items for every game always count
/// 100 %.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct FocusWeights {
    /// Items for suit games and grand.
    pub trump: u32,
    /// Items for Null.
    pub null: u32,
}

impl Default for FocusWeights {
    fn default() -> FocusWeights {
        FocusWeights {
            trump: 100,
            null: 100,
        }
    }
}

impl fmt::Display for RunLength {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FocusWeights {
    /// Weight of an item with this focus, in percent.
    #[must_use]
    pub const fn percent(&self, focus: GameFocus) -> u32 {
        match focus {
            GameFocus::All => 100,
            GameFocus::Trump => self.trump,
            GameFocus::Null => self.null,
        }
    }
}

/// Which characters a content item is for. Empty means every character.
pub type CharacterPool = Vec<Character>;

/// Whether `character` may meet an item with this pool.
#[must_use]
pub fn in_pool(pool: &[Character], character: Character) -> bool {
    pool.is_empty() || pool.contains(&character)
}

/// How a character's start edition differs from the plain one (ADR
/// 0015): card modifications only. Cards are never removed or added; the
/// old fields `remove` and `add` are a load error.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct DeckSpec {
    /// Card modifications on the start edition: each goes on its printed
    /// card, with its value (effects v2, DESIGN 7.2). Their "beim Erhalt"
    /// hooks do not fire (the card comes modified).
    pub mods: Vec<(Card, CardModId, i32)>,
}

/// The one own rule of a character (DESIGN 9): effects that act like a
/// relic the character always owns, without a slot (source
/// [`Source::Character`]), and changes to the run structure of
/// `run.ron`. Everything defaults to "no change".
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct CharacterRule {
    /// Always-on effects, like a relic's.
    pub passive: Vec<Effect>,
    /// Triggered effects, like a relic's.
    pub hooks: Vec<Hook>,
    /// Einmischen per game added to `run.ron` (negative: fewer, never
    /// below 0). Also read under its old name `passes` (passes per series
    /// until the Einmischen rule of 01.11.2026).
    #[serde(alias = "passes")]
    pub redeals: i32,
    /// A cap on the relics a run with this character may hold, below
    /// `run.ron` if that has one (DESIGN 8.0: unlimited since 03.11.2026).
    /// `None`: no cap of its own.
    pub relic_slots: Option<u32>,
    /// The Groschen of a won series (Grundbetrag, übrige Spiele, Zinsen)
    /// count this many times; the extra is [`crate::run::Payout::bonus`].
    pub payout_factor: u32,
    /// "Der Drückeberger": when a pub ends (won or lost), every voucher
    /// redeemed from the player's edition during it comes back onto its
    /// card ([`crate::run::RunEvent::VoucherReturned`]), unless the card
    /// carries another normal modification or a curse by then.
    pub vouchers_return: bool,
    /// "Bruderherz": a card of the player's edition that carries a curse
    /// cannot be cleansed ([`crate::run::RunErrorCode::CurseStays`]).
    pub keep_curses: bool,
    /// "Bruderherz": Zinken (card modifications bought for Groschen on a
    /// card of the player's choice) on matching cards cost less
    /// ([`crate::run::Run::zinken_price`]).
    pub zinken_discount: Option<ZinkenDiscount>,
}

/// A discount on Zinken for some cards (a character rule, DESIGN 9.6):
/// cards matching `cards` cost `percent` percent less, rounded to the
/// nearest Groschen, at least 1 for a priced Zinke.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZinkenDiscount {
    /// Which printed cards.
    pub cards: CardFilter,
    /// Discount in percent (1 to 100).
    pub percent: u32,
}

impl Default for CharacterRule {
    fn default() -> CharacterRule {
        CharacterRule {
            passive: Vec::new(),
            hooks: Vec::new(),
            redeals: 0,
            relic_slots: None,
            payout_factor: 1,
            vouchers_return: false,
            keep_curses: false,
            zinken_discount: None,
        }
    }
}

/// A character as data (DESIGN 9): start deck, start relics, Zielwerte,
/// Null scoring and the Tresen weights per [`GameFocus`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CharacterDef {
    /// Which character.
    pub character: Character,
    /// German display name.
    pub name: String,
    /// German description.
    pub text: String,
    /// Relics owned from the start (Startrelikt), in this order.
    #[serde(default)]
    pub start_relics: Vec<RelicId>,
    /// Consumables owned from the start (e.g. a Schnaps), in this order.
    #[serde(default)]
    pub start_consumables: Vec<ConsumableId>,
    /// The character's own rule.
    #[serde(default)]
    pub rule: CharacterRule,
    /// Start deck.
    #[serde(default)]
    pub deck: DeckSpec,
    /// Zielwerte per Stadtteil, one per pub (the last the boss pub);
    /// `None` uses `run.ron`. Must have the shape of `run.ron`'s.
    #[serde(default)]
    pub targets: Option<Vec<Vec<u64>>>,
    /// How Null games are scored; `None` scores them like the
    /// Skatordnung ([`NullScoring::Fixed`]).
    #[serde(default)]
    pub null_scoring: Option<NullScoring>,
    /// How often the Tresen (offers, Auslage, Kartenmaler) and the
    /// encounters offer relics and card modifications made for one Spielart (default: as
    /// often as any other).
    #[serde(default)]
    pub focus_weights: FocusWeights,
    /// Card back of the player's edition (ADR 0015, DESIGN 9.3); `None`
    /// is the Distel ([`PLAYER_BACK`]).
    #[serde(default)]
    pub back: Option<String>,
}

/// The card back of the player's edition unless his character names
/// another one: the Distel.
pub const PLAYER_BACK: &str = "distel";

impl CharacterDef {
    /// The card back of this character's edition.
    #[must_use]
    pub fn back(&self) -> &str {
        self.back.as_deref().unwrap_or(PLAYER_BACK)
    }
}

/// Which card a [`EncounterEffect::Mark`] goes on; for
/// [`EncounterEffect::Marke`] also which Marke.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum CardPick {
    /// A random card the modification may go on (a card without
    /// modification first, never one this option marked already).
    #[default]
    Random,
    /// A card of the player's choice (the option needs a target card).
    Chosen,
}

/// What a bet ([`EncounterEffect::Bet`]) is on: the next declared game
/// (the Ramsch does not count).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum BetOn {
    /// Won with the defenders Schneider.
    #[default]
    Schneider,
    /// Won.
    Win,
}

/// One effect of an encounter option (DESIGN 13, "Wege und Begegnungen
/// v2", 03.11.2026). An option is a small list of these, applied in order.
/// What an effect leaves open (`None`) is drawn when the encounter opens,
/// so the player sees it before choosing; the open encounter
/// ([`crate::run::ActiveEncounter`]) holds the effects with every draw
/// filled in.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EncounterEffect {
    /// Groschen for the player.
    Coins(u32),
    /// A house rule that applies until the end of the district without
    /// taking a slot (not owned, unlocked, for the character). `None`:
    /// drawn, never the same as another option's.
    EveningRule(Option<RelicId>),
    /// A card modification on a card of the player's edition.
    Mark {
        /// Which card.
        #[serde(default)]
        card: CardPick,
        /// The modification; `None`: one of `offers`, or (without offers)
        /// drawn from the character's pool, a curse only with `curse`.
        #[serde(default)]
        card_mod: Option<CardModId>,
        /// Its value; `None`: the modification's first value.
        #[serde(default)]
        value: Option<i32>,
        /// Modifications with their value, one drawn when it opens.
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        offers: Vec<(CardModId, i32)>,
        /// Without `card_mod` and `offers`: draw a curse. The encounter's
        /// text must say "Fluch".
        #[serde(default, skip_serializing_if = "std::ops::Not::not")]
        curse: bool,
        /// With this chance (percent) a random curse lands on the card
        /// instead. The encounter's text must say "Fluch".
        #[serde(default, skip_serializing_if = "is_zero")]
        curse_percent: u32,
    },
    /// A Marke (Gutschein: every card modification of kind
    /// [`ModKind::Voucher`], i.e. Pfandmarke, Totenmarke, Narrenmarke) on a
    /// card of the player's edition (04.11.2026, DESIGN 13.0). The card is
    /// random (a bare one first) or the player's choice; the Marke is drawn
    /// by rarity weight when the option is taken (`marke: Random`, among
    /// those that may go on the card) or the player's choice (`Chosen`,
    /// [`crate::run::RunAction::ChooseEncounter`] `marke`). The rules of
    /// every Zinke hold: one normal Zinke per card, the special card caps.
    Marke {
        /// Which card.
        #[serde(default)]
        card: CardPick,
        /// Which Marke: drawn when taken (`Random`) or chosen (`Chosen`).
        #[serde(default)]
        marke: CardPick,
    },
    /// Swap an owned relic (the target, an index into the owned relics)
    /// for this one; `None`: drawn.
    SwapRelic(Option<RelicId>),
    /// Sell an owned relic (the target) for this percentage of its price
    /// (100: the full price).
    SellRelic(u32),
    /// This relic into a free slot; `None`: drawn, the same for every
    /// option of the encounter.
    Relic(Option<RelicId>),
    /// Groschen worth this percentage of the encounter's relic
    /// ([`EncounterEffect::Relic`]); becomes [`EncounterEffect::Coins`]
    /// when it opens.
    RelicCoins(u32),
    /// The next declared game counts this many times, won or lost.
    NextGameFactor(u32),
    /// Einmischen in the next pub: more, or (negative) fewer in its first
    /// game.
    NextPubRedeals(i32),
    /// The player sees both skat cards in the next game, as with the
    /// "Kurzer" (from every deal until a game is declared).
    SeeSkat,
    /// A Schnaps into a free slot; `None`: drawn, never the same as
    /// another option's. With every slot full the player gets its price
    /// in Groschen instead.
    Schnapps(Option<ConsumableId>),
    /// Lives back (ADR 0016). Gone with the lives (ADR 0019): no content
    /// may name it; an encounter open in a save from before offers this
    /// many times [`LEGACY_LIFE_COINS`] Groschen instead
    /// ([`crate::run::Run::drop_unknown_content`]).
    Lives(u32),
    /// The exact stops of the next route choices are shown.
    Scout(u32),
    /// Bet the option's cost on the next declared game: paid back with
    /// `percent` percent (200: double) if it goes as bet, else gone.
    Bet {
        /// What the bet is on.
        on: BetOn,
        /// Payout in percent of the stake.
        percent: u32,
    },
    /// Throw the dice for the option's cost: with `chance` percent the
    /// stake comes back with `percent` percent (200: double), else it is
    /// gone. Drawn with the encounter generator.
    Dice {
        /// Chance to win, in percent.
        chance: u32,
        /// Payout in percent of the stake.
        percent: u32,
    },
    /// The Kartenmaler for free on a card of the player's choice; the
    /// encounter waits for [`crate::run::RunAction::ChoosePaint`]. Always
    /// the last effect of its option.
    Painter,
    /// Every modification off a card of the player's choice.
    Cleanse,
}

impl EncounterEffect {
    /// Whether it needs a card of the player's choice.
    #[must_use]
    pub const fn needs_card(&self) -> bool {
        matches!(
            self,
            EncounterEffect::Mark {
                card: CardPick::Chosen,
                ..
            } | EncounterEffect::Marke {
                card: CardPick::Chosen,
                ..
            } | EncounterEffect::Painter
                | EncounterEffect::Cleanse
        )
    }

    /// Whether it needs a Marke of the player's choice
    /// ([`EncounterEffect::Marke`] with `marke: Chosen`).
    #[must_use]
    pub const fn needs_marke(&self) -> bool {
        matches!(
            self,
            EncounterEffect::Marke {
                marke: CardPick::Chosen,
                ..
            }
        )
    }

    /// Whether it needs an owned relic of the player's choice.
    #[must_use]
    pub const fn needs_relic(&self) -> bool {
        matches!(
            self,
            EncounterEffect::SwapRelic(_) | EncounterEffect::SellRelic(_)
        )
    }
}

/// One option of an encounter as data.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EncounterOptionDef {
    /// German title on its card or button.
    pub label: String,
    /// Groschen it costs (the stake of a bet or a throw of the dice).
    #[serde(default)]
    pub cost: u32,
    /// What it does, in order.
    #[serde(default)]
    pub effects: Vec<EncounterEffect>,
    /// Offered only when no other option can be taken ("Zuschauen" at the
    /// Würfelbude).
    #[serde(default)]
    pub fallback: bool,
}

#[allow(clippy::trivially_copy_pass_by_ref)]
const fn is_zero(n: &u32) -> bool {
    *n == 0
}

/// The kind of a stop on a route between two pubs (ADR 0016): the
/// Tresen, or the category of an encounter. Shown on the route choice.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Default, Serialize, Deserialize,
)]
pub enum StopCategory {
    /// The Tresen (shop).
    Tresen,
    /// Begegnung: an event (Ereignis).
    #[default]
    Event,
    /// Upgrade: modifications on, curses off.
    Upgrade,
    /// Rast (Imbissbude): a rest before the next pub (ADR 0019: since the
    /// lives are gone, one Einmischen more or a card cleansed).
    Rest,
    /// Fundgroschen: a few Groschen on the way (03.11.2026).
    FoundCoins,
    /// Kiosk: a small shop with two Schnäpse, cheaper than at the Tresen
    /// (03.11.2026). No encounter.
    Kiosk,
    /// Würfelbude: a throw of the dice for Groschen (03.11.2026).
    DiceStall,
}

impl StopCategory {
    /// Every category in a fixed order.
    pub const ALL: [StopCategory; 7] = [
        StopCategory::Tresen,
        StopCategory::Event,
        StopCategory::Upgrade,
        StopCategory::Rest,
        StopCategory::FoundCoins,
        StopCategory::Kiosk,
        StopCategory::DiceStall,
    ];

    /// Whether its stop is a shop rather than an encounter.
    #[must_use]
    pub const fn is_shop(self) -> bool {
        matches!(self, StopCategory::Tresen | StopCategory::Kiosk)
    }
}

/// An encounter (Ereignis / Rast, DESIGN 13): a stop on a route between
/// two pubs (ADR 0016).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EncounterDef {
    /// Unique id.
    pub id: EncounterId,
    /// German display name.
    pub name: String,
    /// German description.
    pub text: String,
    /// Its options (two or three on a route since 03.11.2026); one must be
    /// taken, there is no walking on.
    pub options: Vec<EncounterOptionDef>,
    /// Weight when drawn (default 1); 0: never drawn on a route.
    #[serde(default = "one")]
    pub weight: u32,
    /// Characters that meet it (empty: all).
    #[serde(default)]
    pub characters: CharacterPool,
    /// The route category it is drawn for (default: Begegnung). Never
    /// [`StopCategory::Tresen`].
    #[serde(default)]
    pub category: StopCategory,
}

const fn one() -> u32 {
    1
}

const fn one_i32() -> i32 {
    1
}

/// When an unlock (Freischaltung, DESIGN 9.3) is earned. Deeds at the
/// table, not play time. Checked against the run events
/// ([`crate::run::meta::MetaState::observe`]).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum UnlockCondition {
    /// Win a suit game or grand with these properties.
    WinGame {
        /// Only this Spielart (`None`: any suit game or grand).
        #[serde(default)]
        game: Option<GameType>,
        /// Only with the defenders Schneider.
        #[serde(default)]
        schneider: bool,
        /// Only from this Risikostufe on.
        #[serde(default)]
        min_risk: Option<RiskLevel>,
        /// Only with at least this Spielwert (after all modifiers).
        #[serde(default)]
        min_value: u64,
        /// Only with at least this many Augen discarded into the skat, as
        /// they count in the game ([`crate::run::RunEvent::GameScored`]).
        #[serde(default)]
        min_discarded: u32,
    },
    /// Win this many suit games or grands with the defenders Schneider in
    /// one pub.
    SchneiderWinsInPub(u32),
    /// Win this many suit games in a row with the same trump suit (any
    /// suit). The streak runs across pubs within a run; a lost game, a
    /// grand, a Null or another trump suit starts it over, a Ramsch
    /// neither counts nor breaks it.
    SameTrumpStreak(u32),
    /// Win a Null game from this Null-Risikostufe on.
    WinNull(NullRisk),
    /// Reach this section (Stadtteil, from 1).
    ReachSection(u32),
    /// Reach the n-th pub of the evening (from 1, counted over the
    /// Stadtteile: pub p of Stadtteil d is `d × 5 + p + 1`).
    ReachPub(u32),
    /// Win a boss series.
    BeatBoss {
        /// Only this boss (`None`: any).
        #[serde(default)]
        boss: Option<BossId>,
        /// Only without redealing (einmischen) in that series. Also read
        /// under its old name `without_pass`.
        #[serde(default, alias = "without_pass")]
        without_redeal: bool,
    },
    /// Win a series with at least this many games left.
    WinSeriesWithGamesLeft(u32),
    /// Win a series and hold at least this many Groschen after its payout.
    SeriesWonWithCoins(u32),
    /// Win a run. Only a won ganzer Abend counts by default, because a
    /// kurzer Abend is much easier to win (DESIGN 9.3).
    WinRun {
        /// Only with this character (`None`: any).
        #[serde(default)]
        character: Option<Character>,
        /// Only a run of this length (default: [`RunLength::Full`]).
        #[serde(default)]
        length: RunLength,
    },
    /// Finish this many runs (won or lost).
    RunsFinished(u32),
}

/// What an unlock opens.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Reward {
    /// A relic appears at the Tresen.
    Relic(RelicId),
    /// A character can be chosen.
    Character(Character),
    /// A card modification joins the random draws (Auslage, Kartenmaler,
    /// Kartengeber).
    CardMod(CardModId),
}

/// An unlock (Freischaltung, DESIGN 9.3): one condition, its rewards.
/// Every relic or character named as a reward is locked until then.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UnlockDef {
    /// Unique id.
    pub id: UnlockId,
    /// German display name.
    pub name: String,
    /// German description of the deed.
    pub text: String,
    /// The deed.
    pub condition: UnlockCondition,
    /// What it opens.
    pub rewards: Vec<Reward>,
}

/// Seltenheit: how often an item shows up at the Tresen.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Rarity {
    /// Common.
    Common,
    /// Uncommon.
    Uncommon,
    /// Rare.
    Rare,
    /// Legendär (M8 wave 3): far rarer than Rare and expensive, for relics
    /// that change the run for good.
    Legendary,
}

/// Kind of relic.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum RelicKind {
    /// Hausregel.
    #[default]
    HouseRule,
    /// Stammtisch-Figur.
    Regular,
}

/// Display group of a relic (02.11.2026): which lever it turns, for the
/// shape of its beer mat in a frontend. Presentation only.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum RelicGroup {
    /// Wert: Augen, Grundwert, Skat.
    #[default]
    Value,
    /// Stufen (Gewinnstufen): Schneider, Spielwert.
    Multiplier,
    /// Kontrolle: Blatt, Skat, Information, Vorhand.
    Control,
    /// Tour: Groschen, Tresen, Wege.
    Tour,
    /// Faktor (ADR 0018): the Spielwert as a whole, multiplied.
    Factor,
}

/// A relic: house rule or Stammtisch figure.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RelicDef {
    /// Unique id (english, snake_case).
    pub id: RelicId,
    /// House rule or figure.
    #[serde(default)]
    pub kind: RelicKind,
    /// German display name.
    pub name: String,
    /// German description.
    pub text: String,
    /// Rarity.
    pub rarity: Rarity,
    /// Price at the Tresen in Groschen (placeholder, set by `balance`).
    pub price: u32,
    /// Always-on effects.
    #[serde(default)]
    pub passive: Vec<Effect>,
    /// Triggered effects.
    #[serde(default)]
    pub hooks: Vec<Hook>,
    /// The Spielart it is made for (default: every game).
    #[serde(default)]
    pub focus: GameFocus,
    /// Characters that meet it at the Tresen (empty: all).
    #[serde(default)]
    pub characters: CharacterPool,
    /// Display group (default: Value).
    #[serde(default)]
    pub group: RelicGroup,
}

/// The kind of a card modification in the effects v2 system (DESIGN 7.2).
/// It decides how it is drawn, whether it blocks the card and where it is
/// drawn on the card.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ModKind {
    /// Rand: always on, the number on the band (Augen, Rang, Trumpf).
    Edge,
    /// Moment: a mark top right says when it acts (last, first trick).
    Moment,
    /// Gutschein: acts when discarded, then falls off the card.
    Voucher,
    /// Fluch: a drawback; the card takes no other modification any more.
    Curse,
    /// Spezialkarte (Untot, Wandelkarte): changes the card's design and
    /// may sit next to one normal modification.
    Special,
    /// Spezialkarte that is a curse ("Abgestuft", DESIGN 9.2): it holds the
    /// special slot, may sit next to one normal modification, and no
    /// other special card replaces it; only cleansing takes it off ("Karte
    /// reinigen", the Werkstatt).
    SpecialCurse,
}

impl ModKind {
    /// Whether it sits in the special slot (Spezialkarte).
    #[must_use]
    pub fn is_special(self) -> bool {
        matches!(self, ModKind::Special | ModKind::SpecialCurse)
    }

    /// Whether it is a curse.
    #[must_use]
    pub fn is_curse(self) -> bool {
        matches!(self, ModKind::Curse | ModKind::SpecialCurse)
    }
}

/// A card modification (gezinkte Karte). It acts on the card instance it is
/// attached to, whoever holds it (ADR 0005), and only while the card is in
/// play and not blanked out. Effects v2 (DESIGN 7.2): a card carries at
/// most one modification besides a special card; the effects are written
/// per unit of the value the card carries ([`Effect::scaled`]).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CardModDef {
    /// Unique id.
    pub id: CardModId,
    /// German display name.
    pub name: String,
    /// German description.
    pub text: String,
    /// Rarity.
    pub rarity: Rarity,
    /// Price (placeholder).
    pub price: u32,
    /// Always-on effects.
    #[serde(default)]
    pub passive: Vec<Effect>,
    /// Triggered effects.
    #[serde(default)]
    pub hooks: Vec<Hook>,
    /// The Spielart it is made for (default: every game).
    #[serde(default)]
    pub focus: GameFocus,
    /// Characters whose random modifications may draw it (empty: all).
    #[serde(default)]
    pub characters: CharacterPool,
    /// Rand, Moment, Gutschein, Fluch or Spezialkarte.
    pub kind: ModKind,
    /// The values on offer, the first for random draws and when no value
    /// is named. Empty: the modification has no value (it counts as 1)
    /// and cannot be raised.
    #[serde(default)]
    pub values: Vec<i32>,
    /// The highest value a raise may reach ("Rang": 2). `None`: no limit.
    #[serde(default)]
    pub max: Option<i32>,
    /// The value it comes with in the Auslage and from the Kartenmaler
    /// (DESIGN 8.6). `None`: [`CardModDef::default_value`].
    #[serde(default)]
    pub shop_value: Option<i32>,
    /// Only cards whose printed card matches may carry it ("Glasdaus":
    /// only a Daus). `None`: every normal card.
    #[serde(default)]
    pub only_on: Option<CardFilter>,
}

impl CardModDef {
    /// Whether it is a curse (Fluch, DESIGN 7.2).
    #[must_use]
    pub fn is_curse(&self) -> bool {
        self.kind.is_curse()
    }

    /// Whether it is a special card (Untot, Wandelkarte).
    #[must_use]
    pub fn is_special(&self) -> bool {
        self.kind.is_special()
    }

    /// Whether it carries a value that a second copy raises.
    #[must_use]
    pub fn has_value(&self) -> bool {
        !self.values.is_empty()
    }

    /// The value of a random draw: the first on offer, 1 without values.
    #[must_use]
    pub fn default_value(&self) -> i32 {
        self.values.first().copied().unwrap_or(1)
    }

    /// The value at the Tresen (Auslage, Kartenmaler): `shop_value`, else
    /// the value of a random draw.
    #[must_use]
    pub fn offer_value(&self) -> i32 {
        self.shop_value.unwrap_or_else(|| self.default_value())
    }

    /// The text with its value filled in: `{n}` is the amount, `{+n}` the
    /// value with its sign ("+4", "−5").
    #[must_use]
    pub fn text_for(&self, value: i32) -> String {
        let signed = match value.cmp(&0) {
            std::cmp::Ordering::Less => format!("−{}", value.unsigned_abs()),
            std::cmp::Ordering::Equal => "0".to_owned(),
            std::cmp::Ordering::Greater => format!("+{value}"),
        };
        self.text
            .replace("{+n}", &signed)
            .replace("{n}", &value.unsigned_abs().to_string())
    }

    /// Whether this modification may sit on `card` (a normal card; blanks
    /// never carry modifications).
    #[must_use]
    ///
    /// A modification that promotes the card ("Winkel", [`Effect::Promote`])
    /// fits no Unter and no Daus: they cannot rise.
    pub fn fits(&self, card: crate::cards::CardKind) -> bool {
        card.face().is_some_and(|face| {
            self.only_on.is_none_or(|f| f.matches(face))
                && (!self.promotes() || face.rank.promotion_room() > 0)
        })
    }

    /// Whether it promotes the card ("Winkel", [`Effect::Promote`]).
    #[must_use]
    pub fn promotes(&self) -> bool {
        self.passive.iter().any(|e| matches!(e, Effect::Promote(_)))
    }

    /// The highest value it may reach on `card`: [`CardModDef::max`], for
    /// "Winkel" also no more steps than the printed rank can rise (a 10
    /// rises once, to the Daus). `None`: no limit.
    #[must_use]
    pub fn max_on(&self, card: crate::cards::CardKind) -> Option<i32> {
        let room = card
            .face()
            .map(|face| i32::from(face.rank.promotion_room()))
            .filter(|&room| self.promotes() && room > 0);
        match (self.max, room) {
            (Some(max), Some(room)) => Some(max.min(room)),
            (max, room) => max.or(room),
        }
    }
}

/// A factor, written as a decimal number in RON and JSON (`1.0`, `0.6`,
/// `1.4`) and kept in thousandths, so that content stays `Eq` and every
/// computation with it is integer arithmetic (deterministic on every
/// platform, also in WebAssembly).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Factor {
    permille: u32,
}

impl Factor {
    /// The factor 1.0.
    pub const ONE: Factor = Factor { permille: 1000 };

    /// The factor 0.0.
    #[must_use]
    pub const fn zero() -> Factor {
        Factor { permille: 0 }
    }

    /// A factor from thousandths (1400 = 1.4).
    #[must_use]
    pub const fn from_permille(permille: u32) -> Factor {
        Factor { permille }
    }

    /// The factor in thousandths.
    #[must_use]
    pub const fn permille(self) -> u32 {
        self.permille
    }

    /// The whole number `n` as a factor (2 = 2.0). Saturates.
    #[must_use]
    pub const fn from_int(n: u32) -> Factor {
        Factor {
            permille: n.saturating_mul(1000),
        }
    }

    /// `value × factor`, rounded half up; unlike [`Factor::apply`] it
    /// may be 0 (a factor 0, `MultiplyGameValue(0)`). Saturates.
    #[must_use]
    pub fn scale(self, value: u64) -> u64 {
        let scaled = (u128::from(value) * u128::from(self.permille) + 500) / 1000;
        u64::try_from(scaled).unwrap_or(u64::MAX)
    }

    /// The product of two factors, rounded half up to thousandths.
    /// Saturates instead of overflowing.
    #[must_use]
    pub fn times(self, other: Factor) -> Factor {
        let product = (u64::from(self.permille) * u64::from(other.permille) + 500) / 1000;
        Factor {
            permille: u32::try_from(product).unwrap_or(u32::MAX),
        }
    }

    /// `value × factor`, rounded half up, at least 1 for a positive value.
    /// Saturates instead of overflowing.
    #[must_use]
    pub fn apply(self, value: u64) -> u64 {
        if value == 0 {
            return 0;
        }
        let scaled = (u128::from(value) * u128::from(self.permille) + 500) / 1000;
        u64::try_from(scaled).unwrap_or(u64::MAX).max(1)
    }
}

impl Default for Factor {
    fn default() -> Factor {
        Factor::ONE
    }
}

impl Serialize for Factor {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_f64(f64::from(self.permille) / 1000.0)
    }
}

impl<'de> Deserialize<'de> for Factor {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Factor, D::Error> {
        let value = f64::deserialize(deserializer)?;
        if !value.is_finite() || !(0.0..=1000.0).contains(&value) {
            return Err(serde::de::Error::custom(format!(
                "factor {value} must be between 0 and 1000"
            )));
        }
        // In range by the check above, so the cast cannot truncate.
        let permille = (value * 1000.0).round() as u32;
        Ok(Factor { permille })
    }
}

/// Figures of every boss: two boss players, both defender seats (ADR
/// 0017). The first sits left of the player ([`crate::run::BOSS_SEAT`]),
/// the second right.
pub const BOSS_FIGURES: usize = 2;

/// One of the two boss players at the boss table (ADR 0017): a name, an
/// edition and optionally an own back.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BossFigure {
    /// German display name ("Der Schieber", "Sein Kumpan").
    pub name: String,
    /// The figure's boss edition (ADR 0015, DESIGN 11): modifications on
    /// every card that matches the filter, with their value. The cards
    /// this figure holds carry them. Empty for the figures of a rotating
    /// boss ("Der Alte"): they play the editions of the rule in force.
    #[serde(default)]
    pub edition: Vec<(CardFilter, CardModId, i32)>,
    /// Card back of this figure's cards; the boss's back if not set.
    #[serde(default)]
    pub back: Option<String>,
}

/// A boss: the special rule of a boss series.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BossDef {
    /// Unique id.
    pub id: BossId,
    /// German display name.
    pub name: String,
    /// German description of the rule.
    pub text: String,
    /// Zielwert factor of the boss series (default 1.0): bosses differ in
    /// how hard they are, so each scales the Zielwert of its series. Set by
    /// `balance`.
    #[serde(default)]
    pub target_factor: Factor,
    /// Always-on effects during the boss series.
    #[serde(default)]
    pub passive: Vec<Effect>,
    /// Triggered effects during the boss series.
    #[serde(default)]
    pub hooks: Vec<Hook>,
    /// Endboss "Der Alte": the rules of these bosses, one per game in this
    /// order (game 1 the first, …, then from the start again), each with
    /// its passive effects and hooks. Bosses not in the character's pool
    /// are skipped. Empty for a normal boss.
    #[serde(default)]
    pub rotation: Vec<BossId>,
    /// Characters that meet it (empty: all).
    #[serde(default)]
    pub characters: CharacterPool,
    /// Zielwert factors for single characters, instead of `target_factor`
    /// (a boss may hit one character harder than another).
    #[serde(default)]
    pub character_factors: BTreeMap<Character, Factor>,
    /// The two boss players at the table, left and right of the player
    /// (ADR 0017), each with a name and an edition. Exactly
    /// [`BOSS_FIGURES`].
    pub figures: Vec<BossFigure>,
    /// Card back of both figures' cards (a figure may name its own); the
    /// shared opponent back if not set.
    #[serde(default)]
    pub back: Option<String>,
}

impl BossDef {
    /// The Zielwert factor of the boss series for this character.
    #[must_use]
    pub fn target_factor_for(&self, character: Character) -> Factor {
        self.character_factors
            .get(&character)
            .copied()
            .unwrap_or(self.target_factor)
    }
}

/// What a consumable does when used. These are player actions, not hooks.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConsumableAction {
    /// "Frisches Deck" (DESIGN 7.3): every modification of a chosen card
    /// of the player's edition falls off, curses included (at the
    /// Tresen). Saves name it `RemoveCard` (before ADR 0015 it removed
    /// the card).
    #[serde(alias = "RemoveCard")]
    CleanseCard,
    /// Attach a card modification to a chosen card (at the Tresen). `None`
    /// picks one at random with the run's RNG.
    AddCardMod(Option<CardModId>),
    /// Attach this card modification with this value to a chosen card (at
    /// the Tresen, effects v2: "Augen +3", "Trumpf +4", "Rang +1").
    ApplyCardMod {
        /// The modification.
        card_mod: CardModId,
        /// Its value.
        value: i32,
    },
    /// Einmischen more in the current series ("Ein Bier", "Kräuter"): a
    /// pool used once the game's own Einmischen are gone. Also read under
    /// its old name `ExtraPasses`.
    #[serde(alias = "ExtraPasses")]
    ExtraRedeals(u32),
    /// Shuffle and deal again before declaring, without using up an
    /// Einmischen ("Neues Blatt").
    Redeal,
    /// Take the last Einmischen back ("Reue"): the deal before it returns
    /// as it was, and that hand is played. Only with ten cards before the
    /// skat is touched, and only right after a redeal of this game; the
    /// Einmischen stays used.
    UndoRedeal,
    /// Look at the two skat cards before choosing Skat or Hand ("Kurzer").
    /// Only while the ten cards are held and the skat is untouched; the
    /// choice stays open.
    PeekSkat,
    /// Change the next declared game ("Doppelkorn", "Magenbitter"). Used at
    /// the Tresen or before the declaration of the dealt game; used up by
    /// the next declaration (Einmischen does not waste it), like "Letzte
    /// Runde".
    NextGame(NextGameEffect),
    /// Repaint a card of the hand (target, never a jack or a blank) into
    /// another suit until the game ends ("Blue Curaçao"). Only before the
    /// declaration; the card is that card for the whole game: trump,
    /// following suit and Spitzen.
    ChalkCard,
    /// Deal again at once without using up an Einmischen, and change the
    /// next declared game ("Doppelkorn" since 02.11.2026): like
    /// [`ConsumableAction::Redeal`] plus [`ConsumableAction::NextGame`], so
    /// it cannot be drunk on a hand already known. Only with ten cards
    /// before the skat is touched.
    RedealNextGame(NextGameEffect),
    /// Draw the stops of the open route choice again ("Schwenker"): every way
    /// gets a new stop, the Tresen stays within reach, a Spelunke stays
    /// where it is. At the route choice only.
    RerollRoutes,
    /// Change the next pub ("Wegbier"): `redeals` more Einmischen for the
    /// whole pub (fewer if negative: the first game allows that many
    /// fewer) and its Zielwert changed by `target` (never below 1).
    /// Between two pubs only (route choice, Tresen, encounter).
    NextPub {
        /// Einmischen more (fewer if negative).
        #[serde(default)]
        redeals: i32,
        /// Change of the Zielwert.
        #[serde(default)]
        target: i32,
    },
    /// Raise a pub account below 0 to 0 ("Klarer"). Only between two
    /// games of a pub (the dealt game untouched, [`crate::run::Run`]), and
    /// only while the account is below 0 (else `ConditionNotMet`).
    ClearAccount,
    /// More games in the current pub ("Absacker"): the Spiellimit rises by
    /// this many. Only between two games of a normal pub or a Spelunke,
    /// never in a boss pub (`NotInBossPub`).
    ExtraGames(u32),
    /// Change the next declared game like [`ConsumableAction::NextGame`],
    /// but only while the pub account is below half its Zielwert
    /// ("Eierlikör"; else `ConditionNotMet`). Drunk between two games
    /// only: with ten cards before the skat is touched.
    NextGameIfBehind(NextGameEffect),
    /// Change the next declared game like [`ConsumableAction::NextGame`]
    /// and put a permanent curse on a random card of the player's edition
    /// that carries none ("Fusel", 03.11.2026: curses as trade). Drunk
    /// before Skat oder Hand only (ten cards, skat untouched); refused with
    /// no card left to take a curse.
    NextGameCursed(NextGameEffect),
}

/// What a consumable of [`ConsumableAction::NextGame`] does to the next
/// declared game.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NextGameEffect {
    /// The Spielwert counts this many times, won or lost ("Doppelkorn").
    Multiply(u32),
    /// Lost, the game counts once instead of twice ("Magenbitter"). Won,
    /// it does nothing; it is used up by the game either way. Dormant
    /// since 04.11.2026: every lost game counts once
    /// ([`RUN_LOSS_FACTOR`]).
    SingleLoss,
    /// Won, the Spielwert counts this many times; lost, it counts as
    /// usual ("Eierlikör"). Used up by the game either way.
    MultiplyWin(u32),
}

/// Group of a consumable at the Tresen, for labels only.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum ConsumableCategory {
    /// A plain item (Gegenstand).
    #[default]
    Item,
    /// A Schnaps: Kurzer, Doppelkorn, Kräuter, Magenbitter, Blue Curaçao.
    Schnapps,
}

/// A consumable (Verbrauchsgegenstand).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConsumableDef {
    /// Unique id.
    pub id: ConsumableId,
    /// German display name.
    pub name: String,
    /// German description.
    pub text: String,
    /// Rarity.
    pub rarity: Rarity,
    /// Price (placeholder).
    pub price: u32,
    /// What it does.
    pub action: ConsumableAction,
    /// Characters that meet it at the Tresen (empty: all).
    #[serde(default)]
    pub characters: CharacterPool,
    /// Group for labels ("Schnäpse").
    #[serde(default)]
    pub category: ConsumableCategory,
    /// What the player pours it on in the frontend (drag and drop); the
    /// core decides whether it can be used now. Default [`ConsumableTarget::Player`].
    #[serde(default)]
    pub target: ConsumableTarget,
}

/// What a Schnaps is poured on in the frontend (Schnaps-Leiste, DESIGN
/// 8.9). Presentation only: whether it can be used now is the core's
/// decision ([`crate::run::view::OwnedConsumable::usable_now`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum ConsumableTarget {
    /// The skat ("Kurzer").
    Skat,
    /// The table: the deal or the next game ("Kräuter", "Doppelkorn",
    /// "Fusel").
    Table,
    /// A card of the hand ("Blue Curaçao").
    Card,
    /// The player himself, drunk ("Konterbier", "Klarer"); written `Self`.
    #[default]
    #[serde(rename = "Self")]
    Player,
    /// The city map at the route choice ("Schwenker").
    Map,
}

/// Groschen rules (design document 12).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Economy {
    /// Grundbetrag per won series.
    pub series_reward: u32,
    /// Bonus per game left in a won series.
    pub per_game_left: u32,
    /// One Groschen of interest per this many Groschen held.
    pub interest_step: u32,
    /// Maximum interest per series.
    pub interest_max: u32,
    /// Price of a Neuauslage at the Tresen.
    pub reroll_cost: u32,
    /// Groschen at the start of a run.
    pub starting_coins: u32,
}

/// Weights of the rarities when the Tresen draws an offer.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RarityWeights {
    /// Weight of common items.
    pub common: u32,
    /// Weight of uncommon items.
    pub uncommon: u32,
    /// Weight of rare items.
    pub rare: u32,
    /// Weight of legendary relics, as a fraction of the others' unit
    /// (decimal, default 0.3: a third of a rare one). Absent in `run.ron`
    /// before the tier existed.
    #[serde(default = "RarityWeights::default_legendary")]
    pub legendary: Factor,
}

impl RarityWeights {
    /// Weight of legendary relics if `run.ron` names none.
    #[must_use]
    pub const fn default_legendary() -> Factor {
        Factor::from_permille(300)
    }

    /// Weight of one rarity, in whole units. Legendary items have less
    /// than one unit and count 0 here; draws that may offer them use
    /// [`RarityWeights::percent_weight`].
    #[must_use]
    pub const fn weight(&self, rarity: Rarity) -> u32 {
        match rarity {
            Rarity::Common => self.common,
            Rarity::Uncommon => self.uncommon,
            Rarity::Rare => self.rare,
            Rarity::Legendary => 0,
        }
    }

    /// Weight of one rarity times `percent` (the character's weight for
    /// the item's Spielart, 100 = plain), so that the fractional legendary
    /// weight keeps its share: common 6 × 100 = 600, legendary 0.3 × 100 =
    /// 30.
    #[must_use]
    pub fn percent_weight(&self, rarity: Rarity, percent: u32) -> u32 {
        match rarity {
            Rarity::Legendary => {
                let scaled = u64::from(self.legendary.permille()) * u64::from(percent) / 1000;
                u32::try_from(scaled).unwrap_or(u32::MAX)
            }
            _ => self.weight(rarity).saturating_mul(percent),
        }
    }
}

/// Tresen layout.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShopConfig {
    /// Relics on offer per visit.
    pub relic_offers: u32,
    /// Consumables on offer per visit.
    pub consumable_offers: u32,
    /// How likely each rarity is.
    pub rarity_weights: RarityWeights,
    /// Zinken at the Tresen: the Auslage, its reroll, the Kartenmaler and
    /// "Karte reinigen" (DESIGN 8.6).
    pub zinken: ZinkenConfig,
    /// Curses as trade: "Schnäppchen" in the Auslage and "Hehlerware" among
    /// the relics (DESIGN 8.6). Without it neither is offered.
    #[serde(default)]
    pub trade: TradeConfig,
}

/// Curses as trade at the Tresen (decided by the user on 03.11.2026,
/// DESIGN 8.6): an offer at half price (rounded down, at least 1) that
/// puts a curse on a random card of the player's edition. Every number is
/// provisional.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct TradeConfig {
    /// Chance in percent that an offer of the Auslage is a "Schnäppchen":
    /// half price, and a curse on another card.
    #[serde(default)]
    pub bargain_percent: u32,
    /// Chance in percent that a drawn relic offer has one "Hehlerware": a
    /// relic at half price that curses a card when bought.
    #[serde(default)]
    pub fence_percent: u32,
}

impl TradeConfig {
    /// The trade price of an offer: half of `price`, rounded down, at
    /// least 1 (0 stays 0).
    #[must_use]
    pub fn half(price: u32) -> u32 {
        if price == 0 { 0 } else { (price / 2).max(1) }
    }
}

/// Zinken at the Tresen (DESIGN 8.6, decided by the user on 02.11.2026):
/// the Auslage of concrete card modifications on concrete cards, "Neu
/// auslosen", the Kartenmaler and "Karte reinigen". Every number is
/// provisional.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZinkenConfig {
    /// Offers in the Auslage per visit.
    pub offers: u32,
    /// Draw weight of a card without modification.
    pub bare_weight: u32,
    /// Draw weight of a card that carries a modification already.
    pub modded_weight: u32,
    /// Vouchers weigh this many percent of their rarity weight.
    pub voucher_percent: u32,
    /// Ranks whose cards cost `premium` more in the Auslage.
    #[serde(default)]
    pub premium_ranks: Vec<crate::cards::Rank>,
    /// Groschen more for a card of `premium_ranks`.
    #[serde(default)]
    pub premium: u32,
    /// Price of the first "Neu auslosen" at a Tresen.
    pub reroll_cost: u32,
    /// Groschen more for every further "Neu auslosen" at the same Tresen.
    pub reroll_step: u32,
    /// Price of the Kartenmaler.
    pub painter_price: u32,
    /// Modifications the Kartenmaler offers to choose from.
    pub painter_choices: u32,
    /// Price of "Karte reinigen".
    pub cleanse_price: u32,
    /// Chance in percent that an offer of the Auslage is a special card
    /// right on a card ("wird untot", "wird Wandelkarte"), while the
    /// edition may still get one (DESIGN 8.6). At most one per Auslage.
    pub special_percent: u32,
    /// Price of such an offer before discounts (plus `premium` on a card
    /// of `premium_ranks`).
    pub special_price: u32,
    /// Groschen a Totenmarke or Narrenmarke pays when it is redeemed while
    /// another card is that special card already (DESIGN 7.2: at most one
    /// Untot and one Wandelkarte per edition).
    pub capped_voucher_coins: u32,
}

/// Groschen per life of an encounter option from before ADR 0019 (no
/// lives): an open encounter of an old save that offered a life offers
/// these instead.
pub const LEGACY_LIFE_COINS: u32 = 3;

/// Draw weights of the stop categories on a route. The Tresen has none:
/// since 03.11.2026 one of the three ways always leads to it, and the
/// other two draw two different categories by these weights.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StopWeights {
    /// Begegnung.
    pub event: u32,
    /// Upgrade.
    pub upgrade: u32,
    /// Rast.
    pub rest: u32,
    /// Fundgroschen.
    #[serde(default)]
    pub found_coins: u32,
    /// Kiosk.
    #[serde(default)]
    pub kiosk: u32,
    /// Würfelbude.
    #[serde(default)]
    pub dice_stall: u32,
}

impl StopWeights {
    /// The weight of one category (0 for the Tresen).
    #[must_use]
    pub const fn of(&self, category: StopCategory) -> u32 {
        match category {
            StopCategory::Tresen => 0,
            StopCategory::Event => self.event,
            StopCategory::Upgrade => self.upgrade,
            StopCategory::Rest => self.rest,
            StopCategory::FoundCoins => self.found_coins,
            StopCategory::Kiosk => self.kiosk,
            StopCategory::DiceStall => self.dice_stall,
        }
    }
}

/// The Kiosk (03.11.2026): a stop with a few Schnäpse, each cheaper than
/// at the Tresen. One may be bought.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KioskConfig {
    /// Schnäpse on offer.
    pub offers: u32,
    /// Groschen off the Tresen price (after the Tresen discount), at
    /// least 1 Groschen is left.
    pub discount: u32,
}

impl Default for KioskConfig {
    fn default() -> Self {
        KioskConfig {
            offers: 2,
            discount: 1,
        }
    }
}

/// The route choice between two pubs (ADR 0016).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RouteConfig {
    /// How often each stop category is drawn.
    pub weights: StopWeights,
    /// The Kiosk.
    #[serde(default)]
    pub kiosk: KioskConfig,
}

/// The Spelunke (elite pub, ADR 0016).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpelunkeConfig {
    /// Chance that one route of a choice leads to a Spelunke instead of
    /// the next normal pub (0.0 to 1.0).
    pub chance: Factor,
    /// Factor on the Zielwert of the pub it replaces.
    pub target_factor: Factor,
    /// The encounter of its loot (never on a route).
    #[serde(default = "SpelunkeConfig::default_loot")]
    pub loot: EncounterId,
}

impl SpelunkeConfig {
    fn default_loot() -> EncounterId {
        EncounterId::new("spelunke_loot")
    }
}

/// The relic pick of a won boss pub (ADR 0017).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BossRewardConfig {
    /// Relics on offer.
    pub relics: u32,
    /// Lowest rarity on offer.
    pub min_rarity: Rarity,
    /// A won boss pub plays all its games, so none is left; its payout
    /// counts this many games left instead (at least the real number),
    /// before it is doubled. 0: the games really left.
    #[serde(default)]
    pub games_left_as: u32,
}

impl Default for BossRewardConfig {
    fn default() -> BossRewardConfig {
        BossRewardConfig {
            relics: 3,
            min_rarity: Rarity::Uncommon,
            games_left_as: 0,
        }
    }
}

/// Run structure as data: target values, limits, slots, economy, bosses.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RunConfig {
    /// Zielwerte per Stadtteil, one per pub in order; the last of each row
    /// is the boss pub (before the boss's factor). Every row has
    /// [`crate::run::PUBS_PER_DISTRICT`] values.
    pub targets: Vec<Vec<u64>>,
    /// The route choice between two pubs.
    pub route: RouteConfig,
    /// The Spelunke.
    pub spelunke: SpelunkeConfig,
    /// Spiellimit per pub.
    pub game_limit: u32,
    /// Einmischen per game: how often the player may have a game dealt
    /// again before declaring; then one of the hands must be played. Also
    /// read under its old name `passes` (passes per series).
    #[serde(alias = "passes")]
    pub redeals: u32,
    /// Slots for house rules and figures together. `None` (the default
    /// since 03.11.2026): the player may hold any number of relics.
    #[serde(default)]
    pub relic_slots: Option<u32>,
    /// Slots for consumables.
    pub consumable_slots: u32,
    /// Groschen rules.
    pub economy: Economy,
    /// Tresen layout.
    pub shop: ShopConfig,
    /// Bosses a boss series may draw from.
    pub bosses: Vec<BossId>,
    /// Boss of the last Stadtteil's boss pub (Endboss), not drawn.
    #[serde(default)]
    pub final_boss: Option<BossId>,
    /// The relic pick after a won boss pub (ADR 0017).
    #[serde(default)]
    pub boss_reward: BossRewardConfig,
    /// The Grundwerte of the run where they differ from the Skatordnung
    /// (DESIGN 4.2). Without: the Skatordnung.
    #[serde(default)]
    pub base_values: BaseValues,
    /// The card reward after a won pub (DESIGN 5.0, Kartenbelohnung).
    /// Without: none.
    #[serde(default)]
    pub card_reward: Option<CardRewardConfig>,
    /// The defenders' card play per Stadtteil (DESIGN 10.3): heuristic or
    /// PIMC with its strength. Later Stadtteile take the last entry; an
    /// empty list means the heuristic everywhere.
    #[serde(default)]
    pub opponents: Vec<crate::ai::pimc::Opponents>,
}

impl RunConfig {
    /// Number of Stadtteile (sections) of a run.
    #[must_use]
    pub fn sections(&self) -> u32 {
        u32::try_from(self.targets.len()).unwrap_or(0)
    }

    /// The Grundwert of `game` in the run: the Skatordnung
    /// ([`GameType::base_value`]) except where [`RunConfig::base_values`]
    /// sets another (Grand 18 since 04.11.2026). `None` for Null, which
    /// the run scores by the character's [`NullScoring`].
    #[must_use]
    pub fn base_value(&self, game: GameType) -> Option<u32> {
        match game {
            GameType::Grand => Some(self.base_values.grand),
            _ => game.base_value(),
        }
    }

    /// [`crate::rules::game_value_parts`] with the run's Grundwerte
    /// ([`RunConfig::base_value`]) for suit games and Grand; a Null and a
    /// Ramsch as the rules give them.
    #[must_use]
    pub fn game_value_parts(
        &self,
        declaration: crate::rules::Declaration,
        matadors: crate::rules::Matadors,
        outcome: &crate::rules::Outcome,
        thresholds: &crate::rules::Thresholds,
    ) -> crate::rules::ValueParts {
        let parts = crate::rules::game_value_parts(declaration, matadors, outcome, thresholds);
        let own = match declaration {
            crate::rules::Declaration::Suit { .. } | crate::rules::Declaration::Grand { .. } => {
                self.base_value(declaration.game())
            }
            _ => None,
        };
        own.map_or(parts, |base| parts.with_base_value(base))
    }
}

/// Grundwerte of the run that differ from the Skatordnung (DESIGN 4.2).
/// The rules ([`crate::rules::game_value`]) keep the Skatordnung; the run
/// scores its games with these.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct BaseValues {
    /// Grundwert of a Grand (Skatordnung 24; in the run 18 since the
    /// decision of the user on 04.11.2026, Cockpit q27).
    pub grand: u32,
}

impl Default for BaseValues {
    fn default() -> BaseValues {
        BaseValues { grand: 24 }
    }
}

/// The card reward after a won normal pub or Spelunke (DESIGN 5.0,
/// decision of the user on 04.11.2026, Cockpit q24): the innkeeper offers
/// [`CardRewardConfig::offers`] Zinken, the player takes one for free on
/// a card of his choice, or skips for [`CardRewardConfig::skip_coins`].
/// All numbers VORLÄUFIG.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CardRewardConfig {
    /// Zinken on offer, all different.
    pub offers: u32,
    /// Groschen for "Überspringen".
    pub skip_coins: u32,
    /// Weight of each rarity class (drawn first, then a Zinke of it): a
    /// rare Zinke comes with `rare / (common + uncommon + rare)` per
    /// offer, as far as the class has one that fits.
    pub rarity_weights: RarityWeights,
    /// Marken (vouchers) weigh this many percent within their class.
    pub voucher_percent: u32,
    /// Percent per offer for a special card (Untot, Wandelkarte) instead
    /// of a Zinke, while the edition may still get one.
    pub special_percent: u32,
}

/// Why content failed to load.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContentError(pub String);

impl fmt::Display for ContentError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "content error: {}", self.0)
    }
}

impl std::error::Error for ContentError {}

/// The RON texts content is loaded from, one per file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Sources<'a> {
    /// `house_rules.ron`: house rules and figures.
    pub relics: &'a str,
    /// `card_mods.ron`.
    pub card_mods: &'a str,
    /// `bosses.ron`.
    pub bosses: &'a str,
    /// `consumables.ron`.
    pub consumables: &'a str,
    /// `run.ron`.
    pub run: &'a str,
    /// `characters.ron`.
    pub characters: &'a str,
    /// `encounters.ron`.
    pub encounters: &'a str,
    /// `unlocks.ron`.
    pub unlocks: &'a str,
    /// `guests.ron`: the Stammgäste.
    pub guests: &'a str,
    /// `tour.ron`: names of the city and the Spelunke maluses.
    pub tour: &'a str,
}

impl Sources<'static> {
    /// The RON files embedded in the binary.
    #[must_use]
    pub const fn embedded() -> Sources<'static> {
        Sources {
            relics: include_str!("../../content/house_rules.ron"),
            card_mods: include_str!("../../content/card_mods.ron"),
            bosses: include_str!("../../content/bosses.ron"),
            consumables: include_str!("../../content/consumables.ron"),
            run: include_str!("../../content/run.ron"),
            characters: include_str!("../../content/characters.ron"),
            encounters: include_str!("../../content/encounters.ron"),
            unlocks: include_str!("../../content/unlocks.ron"),
            guests: include_str!("../../content/guests.ron"),
            tour: include_str!("../../content/tour.ron"),
        }
    }
}

/// All content of the game, parsed and validated. Maps are ordered by id,
/// so iteration is deterministic.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Content {
    /// House rules and figures.
    pub relics: BTreeMap<RelicId, RelicDef>,
    /// Card modifications.
    pub card_mods: BTreeMap<CardModId, CardModDef>,
    /// Bosses.
    pub bosses: BTreeMap<BossId, BossDef>,
    /// Consumables.
    pub consumables: BTreeMap<ConsumableId, ConsumableDef>,
    /// Run structure.
    pub run: RunConfig,
    /// Characters.
    pub characters: BTreeMap<Character, CharacterDef>,
    /// Encounters (Ereignisse).
    pub encounters: BTreeMap<EncounterId, EncounterDef>,
    /// Unlocks (Freischaltungen).
    pub unlocks: BTreeMap<UnlockId, UnlockDef>,
    /// Stammgäste, the normal opponents at the table.
    pub guests: BTreeMap<GuestId, GuestDef>,
    /// Word lists of the city's name generator (ADR 0016).
    pub city: CityNames,
    /// Spelunke maluses (ADR 0016).
    pub maluses: BTreeMap<MalusId, MalusDef>,
}

fn parse<T: for<'de> Deserialize<'de>>(file: &str, text: &str) -> Result<T, ContentError> {
    ron::from_str(text).map_err(|e| ContentError(format!("{file}: {e}")))
}

fn index<K: Ord + fmt::Display + Clone, V>(
    file: &str,
    items: Vec<V>,
    id: impl Fn(&V) -> &K,
) -> Result<BTreeMap<K, V>, ContentError> {
    let mut map = BTreeMap::new();
    for item in items {
        let key = id(&item).clone();
        if map.contains_key(&key) {
            return Err(ContentError(format!("{file}: duplicate id {key}")));
        }
        map.insert(key, item);
    }
    Ok(map)
}

fn index_characters(
    items: Vec<CharacterDef>,
) -> Result<BTreeMap<Character, CharacterDef>, ContentError> {
    let mut map = BTreeMap::new();
    for item in items {
        if map.contains_key(&item.character) {
            return Err(ContentError(format!(
                "characters: duplicate character {}",
                item.character
            )));
        }
        map.insert(item.character, item);
    }
    Ok(map)
}

fn validate_effects(
    what: &str,
    passive: &[Effect],
    hooks: &[Hook],
    is_card_mod: bool,
) -> Result<(), ContentError> {
    let check = |stage: Stage, effect: &Effect| {
        if !effect.allowed_in(stage) {
            return Err(ContentError(format!(
                "{what}: effect {effect:?} is not allowed in {stage:?}"
            )));
        }
        if effect.card_mod_only() && !is_card_mod {
            return Err(ContentError(format!(
                "{what}: effect {effect:?} only works on card modifications"
            )));
        }
        Ok(())
    };
    let check = |stage: Stage, effect: &Effect| {
        if let Effect::RevealDefenderHand(seat) = effect {
            if !(1..=2).contains(seat) {
                return Err(ContentError(format!(
                    "{what}: RevealDefenderHand needs seat 1 or 2, not {seat}"
                )));
            }
        }
        check(stage, effect)
    };
    for effect in passive {
        check(Stage::Passive, effect)?;
    }
    for hook in hooks {
        let stage = hook.stage();
        if !hook.when().allowed_in(stage) {
            return Err(ContentError(format!(
                "{what}: condition {:?} cannot be evaluated in {stage:?}",
                hook.when()
            )));
        }
        if hook.when().card_mod_only() && !is_card_mod {
            return Err(ContentError(format!(
                "{what}: condition {:?} only works on card modifications",
                hook.when()
            )));
        }
        if matches!(stage, Stage::OnAttach | Stage::OnDiscard) && !is_card_mod {
            return Err(ContentError(format!(
                "{what}: {stage:?} only works on card modifications"
            )));
        }
        for effect in hook.then() {
            check(stage, effect)?;
        }
    }
    Ok(())
}

impl Content {
    /// Parse and validate the embedded content.
    pub fn load() -> Result<Content, ContentError> {
        Content::from_sources(&Sources::embedded())
    }

    /// Parse and validate content from RON texts (tests, modding).
    pub fn from_sources(sources: &Sources<'_>) -> Result<Content, ContentError> {
        let characters: Vec<CharacterDef> = parse("characters", sources.characters)?;
        let tour: TourContent = parse("tour", sources.tour)?;
        let content = Content {
            relics: index(
                "relics",
                parse::<Vec<RelicDef>>("relics", sources.relics)?,
                |r| &r.id,
            )?,
            card_mods: index(
                "card_mods",
                parse::<Vec<CardModDef>>("card_mods", sources.card_mods)?,
                |m| &m.id,
            )?,
            bosses: index(
                "bosses",
                parse::<Vec<BossDef>>("bosses", sources.bosses)?,
                |b| &b.id,
            )?,
            consumables: index(
                "consumables",
                parse::<Vec<ConsumableDef>>("consumables", sources.consumables)?,
                |c| &c.id,
            )?,
            run: parse("run", sources.run)?,
            characters: index_characters(characters)?,
            encounters: index(
                "encounters",
                parse::<Vec<EncounterDef>>("encounters", sources.encounters)?,
                |e| &e.id,
            )?,
            unlocks: index(
                "unlocks",
                parse::<Vec<UnlockDef>>("unlocks", sources.unlocks)?,
                |u| &u.id,
            )?,
            guests: index(
                "guests",
                parse::<Vec<GuestDef>>("guests", sources.guests)?,
                |g| &g.id,
            )?,
            city: tour.city,
            maluses: index("tour", tour.maluses, |m| &m.id)?,
        };
        content.validate()?;
        Ok(content)
    }

    fn validate(&self) -> Result<(), ContentError> {
        for relic in self.relics.values() {
            validate_effects(
                &format!("relic {}", relic.id),
                &relic.passive,
                &relic.hooks,
                false,
            )?;
        }
        for card_mod in self.card_mods.values() {
            validate_effects(
                &format!("card mod {}", card_mod.id),
                &card_mod.passive,
                &card_mod.hooks,
                true,
            )?;
            self.validate_card_mod(card_mod)?;
        }
        for boss in self.bosses.values() {
            validate_effects(
                &format!("boss {}", boss.id),
                &boss.passive,
                &boss.hooks,
                false,
            )?;
        }
        for consumable in self.consumables.values() {
            let factor_zero = matches!(
                consumable.action,
                ConsumableAction::NextGame(
                    NextGameEffect::Multiply(0) | NextGameEffect::MultiplyWin(0)
                ) | ConsumableAction::NextGameIfBehind(
                    NextGameEffect::Multiply(0) | NextGameEffect::MultiplyWin(0)
                ) | ConsumableAction::NextGameCursed(
                    NextGameEffect::Multiply(0) | NextGameEffect::MultiplyWin(0)
                ) | ConsumableAction::ExtraGames(0)
            );
            if factor_zero {
                return Err(ContentError(format!(
                    "consumable {}: factor 0",
                    consumable.id
                )));
            }
            let named = match &consumable.action {
                ConsumableAction::AddCardMod(Some(id))
                | ConsumableAction::ApplyCardMod { card_mod: id, .. } => Some(id),
                _ => None,
            };
            if let Some(id) = named {
                if !self.card_mods.contains_key(id) {
                    return Err(ContentError(format!(
                        "consumable {}: unknown card mod {id}",
                        consumable.id
                    )));
                }
            }
        }
        if self.run.targets.is_empty() {
            return Err(ContentError("run: no sections".into()));
        }
        let pubs = usize::try_from(crate::run::PUBS_PER_DISTRICT).unwrap_or(0);
        if self.run.targets.iter().any(|row| row.len() != pubs) {
            return Err(ContentError(format!(
                "run: every row of targets needs {pubs} pubs"
            )));
        }
        if self.run.game_limit == 0 {
            return Err(ContentError("run: game limit 0".into()));
        }
        if self.run.spelunke.chance > Factor::ONE {
            return Err(ContentError("run: Spelunke chance above 1".into()));
        }
        self.validate_tour()?;
        for boss in &self.run.bosses {
            if !self.bosses.contains_key(boss) {
                return Err(ContentError(format!("run: unknown boss {boss}")));
            }
        }
        if self.run.bosses.is_empty() {
            return Err(ContentError("run: no bosses".into()));
        }
        if let Some(boss) = &self.run.final_boss {
            if !self.bosses.contains_key(boss) {
                return Err(ContentError(format!("run: unknown final boss {boss}")));
            }
        }
        for boss in self.bosses.values() {
            if boss.figures.len() != BOSS_FIGURES {
                return Err(ContentError(format!(
                    "boss {}: needs {BOSS_FIGURES} figures",
                    boss.id
                )));
            }
            for figure in &boss.figures {
                if figure.name.trim().is_empty() {
                    return Err(ContentError(format!(
                        "boss {}: figure without name",
                        boss.id
                    )));
                }
                if !boss.rotation.is_empty() && !figure.edition.is_empty() {
                    return Err(ContentError(format!(
                        "boss {}: a rotating boss plays the editions of the rule in force",
                        boss.id
                    )));
                }
                crate::deck::Edition::with_mods(&figure.edition, self).map_err(|e| {
                    ContentError(format!("boss {} ({}): edition: {e}", boss.id, figure.name))
                })?;
            }
            for sub in &boss.rotation {
                let known = self.bosses.get(sub);
                if known.is_none_or(|b| !b.rotation.is_empty()) {
                    return Err(ContentError(format!(
                        "boss {}: rotation names {sub}, which is unknown or rotates itself",
                        boss.id
                    )));
                }
            }
        }
        self.validate_characters()?;
        self.validate_encounters()?;
        self.validate_guests()?;
        self.validate_unlocks()
    }

    fn validate_guests(&self) -> Result<(), ContentError> {
        for guest in self.guests.values() {
            let q = &guest.quips;
            let texts = [&guest.name, &guest.look, &q.greet, &q.win, &q.lose];
            if guest.id.as_str().is_empty() || texts.iter().any(|t| t.trim().is_empty()) {
                return Err(ContentError(format!(
                    "guest {}: id, name, look and quips must not be empty",
                    guest.id
                )));
            }
        }
        Ok(())
    }

    /// Effects v2 (DESIGN 7.2): a voucher acts when discarded and only
    /// then, it turns a card only into a special card, a curse says
    /// "Fluch:", a limit is not below the first value.
    fn validate_card_mod(&self, def: &CardModDef) -> Result<(), ContentError> {
        let what = format!("card mod {}", def.id);
        let on_discard = def.hooks.iter().any(|h| h.stage() == Stage::OnDiscard);
        if on_discard != (def.kind == ModKind::Voucher) {
            return Err(ContentError(format!(
                "{what}: a voucher and only a voucher acts on discard"
            )));
        }
        for effect in def.hooks.iter().flat_map(Hook::then) {
            if let Effect::TurnInto(target) = effect {
                if !self
                    .card_mods
                    .get(target)
                    .is_some_and(CardModDef::is_special)
                {
                    return Err(ContentError(format!(
                        "{what}: TurnInto needs a special card, not {target}"
                    )));
                }
            }
        }
        if def.is_curse() && !def.text.starts_with("Fluch:") {
            return Err(ContentError(format!(
                "{what}: a curse's text starts with \"Fluch:\""
            )));
        }
        if def.max.is_some_and(|max| max < def.default_value()) {
            return Err(ContentError(format!("{what}: max below the first value")));
        }
        Ok(())
    }

    fn validate_characters(&self) -> Result<(), ContentError> {
        if !self.characters.contains_key(&Character::Declarer) {
            return Err(ContentError(
                "characters: the Alleinspieler is missing".into(),
            ));
        }
        for def in self.characters.values() {
            let what = format!("character {}", def.character);
            for relic in &def.start_relics {
                if !self.relics.contains_key(relic) {
                    return Err(ContentError(format!("{what}: unknown start relic {relic}")));
                }
            }
            if !self.relic_fits(def.character, def.start_relics.len()) {
                return Err(ContentError(format!(
                    "{what}: more start relics than slots"
                )));
            }
            for consumable in &def.start_consumables {
                if !self.consumables.contains_key(consumable) {
                    return Err(ContentError(format!(
                        "{what}: unknown start consumable {consumable}"
                    )));
                }
            }
            if def.start_consumables.len() > usize::try_from(self.run.consumable_slots).unwrap_or(0)
            {
                return Err(ContentError(format!(
                    "{what}: more start consumables than slots"
                )));
            }
            for (card, card_mod, _) in &def.deck.mods {
                if !self.card_mods.contains_key(card_mod) {
                    return Err(ContentError(format!(
                        "{what}: unknown card mod {card_mod} on {card}"
                    )));
                }
            }
            validate_effects(&what, &def.rule.passive, &def.rule.hooks, false)?;
            if let Some(discount) = &def.rule.zinken_discount {
                if discount.percent == 0 || discount.percent > 100 {
                    return Err(ContentError(format!(
                        "{what}: zinken_discount must be 1 to 100 percent"
                    )));
                }
            }
            if def.rule.payout_factor == 0 {
                return Err(ContentError(format!(
                    "{what}: payout_factor must be at least 1"
                )));
            }
            if let Some(targets) = &def.targets {
                let shape = |t: &Vec<Vec<u64>>| t.iter().map(Vec::len).collect::<Vec<_>>();
                if shape(targets) != shape(&self.run.targets) {
                    return Err(ContentError(format!(
                        "{what}: {} sections of Zielwerte, run.ron has {}",
                        targets.len(),
                        self.run.targets.len()
                    )));
                }
            }
            let mut deck = crate::deck::Edition::standard();
            deck.apply_spec(&def.deck, self)
                .map_err(|e| ContentError(format!("{what}: start deck: {e}")))?;
        }
        Ok(())
    }

    fn validate_encounters(&self) -> Result<(), ContentError> {
        for def in self.encounters.values() {
            self.validate_encounter(def)?;
        }
        // Every category a route may draw needs an encounter for everyone
        // (the Kiosk is a shop).
        let weights = &self.run.route.weights;
        for category in StopCategory::ALL {
            if category.is_shop() || weights.of(category) == 0 {
                continue;
            }
            for character in self.characters.keys() {
                let met = self.encounters.values().any(|e| {
                    e.category == category && e.weight > 0 && in_pool(&e.characters, *character)
                });
                if !met {
                    return Err(ContentError(format!(
                        "run: no {category:?} encounter for {character}"
                    )));
                }
            }
        }
        let loot = &self.run.spelunke.loot;
        let has_relic = self.encounters.get(loot).is_some_and(|e| {
            e.options
                .iter()
                .flat_map(|o| &o.effects)
                .any(|f| matches!(f, EncounterEffect::Relic(_)))
        });
        if !has_relic {
            return Err(ContentError(format!(
                "run: the Spelunke's loot {loot} must be an encounter with a relic"
            )));
        }
        Ok(())
    }

    /// One encounter: its options and what they name.
    fn validate_encounter(&self, def: &EncounterDef) -> Result<(), ContentError> {
        let err = |what: String| Err(ContentError(format!("encounter {}: {what}", def.id)));
        if def.category.is_shop() {
            return err(format!("{:?} is no encounter category", def.category));
        }
        if def
            .options
            .iter()
            .flat_map(|o| &o.effects)
            .any(|e| matches!(e, EncounterEffect::Lives(_)))
        {
            return err("there are no lives any more (ADR 0019)".into());
        }
        let regular = def.options.iter().filter(|o| !o.fallback).count();
        if regular == 0 || regular > 3 {
            return err("needs one to three options besides a fallback".into());
        }
        // One option is always there to take: one that costs nothing, or
        // a fallback. Only bets stake every option (they are put on a
        // route only when the smallest stake is affordable).
        let all_bets = def.options.iter().all(|o| {
            o.effects
                .iter()
                .any(|e| matches!(e, EncounterEffect::Bet { .. }))
        });
        if !all_bets && !def.options.iter().any(|o| o.cost == 0 || o.fallback) {
            return err("needs an option that costs nothing".into());
        }
        let has_relic = def
            .options
            .iter()
            .flat_map(|o| &o.effects)
            .any(|e| matches!(e, EncounterEffect::Relic(_)));
        let mut gives_curse = false;
        for option in &def.options {
            let card = option.effects.iter().any(EncounterEffect::needs_card);
            let relic = option.effects.iter().any(EncounterEffect::needs_relic);
            if card && relic {
                return err(format!("{}: a card and a relic target", option.label));
            }
            let painter = option
                .effects
                .iter()
                .position(|e| *e == EncounterEffect::Painter);
            if painter.is_some_and(|i| i + 1 != option.effects.len()) {
                return err(format!("{}: the Kartenmaler must come last", option.label));
            }
            let stakes = option.effects.iter().any(|e| {
                matches!(
                    e,
                    EncounterEffect::Bet { .. } | EncounterEffect::Dice { .. }
                )
            });
            if stakes && option.cost == 0 {
                return err(format!("{}: a stake of 0", option.label));
            }
            for effect in &option.effects {
                match effect {
                    EncounterEffect::Mark {
                        card_mod,
                        offers,
                        curse,
                        curse_percent,
                        ..
                    } => {
                        let mut named: Vec<&CardModId> = offers.iter().map(|(m, _)| m).collect();
                        named.extend(card_mod.iter());
                        for id in named {
                            let Some(m) = self.card_mods.get(id) else {
                                return err(format!("unknown card mod {id}"));
                            };
                            if *curse && !m.is_curse() {
                                return err(format!("curse: true names {id}, which is no curse"));
                            }
                            gives_curse |= m.is_curse();
                        }
                        gives_curse |= (*curse && card_mod.is_none() && offers.is_empty())
                            || *curse_percent > 0;
                    }
                    EncounterEffect::EveningRule(Some(id)) => {
                        if !self
                            .relics
                            .get(id)
                            .is_some_and(|r| r.kind == RelicKind::HouseRule)
                        {
                            return err(format!("{id} is no house rule"));
                        }
                    }
                    EncounterEffect::Relic(Some(id)) | EncounterEffect::SwapRelic(Some(id)) => {
                        if !self.relics.contains_key(id) {
                            return err(format!("unknown relic {id}"));
                        }
                    }
                    EncounterEffect::Schnapps(Some(id)) => {
                        if !self
                            .consumables
                            .get(id)
                            .is_some_and(|c| c.category == ConsumableCategory::Schnapps)
                        {
                            return err(format!("{id} is no Schnaps"));
                        }
                    }
                    EncounterEffect::RelicCoins(_) if !has_relic => {
                        return err("Groschen for a relic, but no relic".into());
                    }
                    _ => {}
                }
            }
        }
        // Curses come only where announced (DESIGN 7.2).
        if gives_curse && !def.text.contains("Fluch") {
            return err("gives a curse, its text must say \"Fluch\"".into());
        }
        Ok(())
    }

    fn validate_tour(&self) -> Result<(), ContentError> {
        let city = &self.city;
        let pubs = self.run.sections() * (crate::run::PUBS_PER_DISTRICT - 1);
        if city.districts.len() < self.targets_rows() || city.pubs.is_empty() {
            return Err(ContentError(
                "tour: needs a district name per Stadtteil and pub patterns".into(),
            ));
        }
        let mut looks: Vec<_> = city.districts.iter().map(|d| d.look).collect();
        looks.sort_unstable();
        looks.dedup();
        if looks.len() < self.targets_rows().min(3) {
            return Err(ContentError(
                "tour: needs a district of a different look per Stadtteil".into(),
            ));
        }
        if city.boss_pubs.is_empty() || city.spelunken.is_empty() || pubs == 0 {
            return Err(ContentError(
                "tour: needs boss pub and Spelunke patterns".into(),
            ));
        }
        for pattern in city
            .pubs
            .iter()
            .chain(&city.boss_pubs)
            .chain(&city.spelunken)
        {
            let mut rest = pattern.pattern.as_str();
            while let Some(start) = rest.find('{') {
                let Some(len) = rest[start..].find('}') else {
                    return Err(ContentError(format!(
                        "tour: unclosed placeholder in {:?}",
                        pattern.pattern
                    )));
                };
                let key = &rest[start + 1..start + len];
                if city.words.get(key).is_none_or(Vec::is_empty) {
                    return Err(ContentError(format!(
                        "tour: {:?} names the empty or unknown list {key}",
                        pattern.pattern
                    )));
                }
                rest = &rest[start + len + 1..];
            }
        }
        if self.maluses.is_empty() {
            return Err(ContentError("tour: no Spelunke maluses".into()));
        }
        for malus in self.maluses.values() {
            validate_effects(&format!("malus {}", malus.id), &malus.passive, &[], false)?;
            if let Some(curse) = &malus.curse {
                if !self
                    .card_mods
                    .get(&curse.card_mod)
                    .is_some_and(CardModDef::is_curse)
                {
                    return Err(ContentError(format!(
                        "malus {}: {} is no curse",
                        malus.id, curse.card_mod
                    )));
                }
            }
            if malus.text.trim().is_empty() || malus.name.trim().is_empty() {
                return Err(ContentError(format!("malus {}: name and text", malus.id)));
            }
        }
        Ok(())
    }

    fn targets_rows(&self) -> usize {
        self.run.targets.len()
    }

    fn validate_unlocks(&self) -> Result<(), ContentError> {
        let mut rewarded: Vec<&Reward> = Vec::new();
        for def in self.unlocks.values() {
            let what = format!("unlock {}", def.id);
            if def.rewards.is_empty() {
                return Err(ContentError(format!("{what}: no reward")));
            }
            for reward in &def.rewards {
                let known = match reward {
                    Reward::Relic(id) => self.relics.contains_key(id),
                    Reward::Character(c) => {
                        *c != Character::Declarer && self.characters.contains_key(c)
                    }
                    Reward::CardMod(id) => self.card_mods.contains_key(id),
                };
                if !known {
                    return Err(ContentError(format!("{what}: unknown reward {reward:?}")));
                }
                if rewarded.contains(&reward) {
                    return Err(ContentError(format!("{what}: {reward:?} rewarded twice")));
                }
                rewarded.push(reward);
            }
            if let UnlockCondition::BeatBoss { boss: Some(b), .. } = &def.condition {
                if !self.bosses.contains_key(b) {
                    return Err(ContentError(format!("{what}: unknown boss {b}")));
                }
            }
        }
        Ok(())
    }

    /// A character by its kind.
    #[must_use]
    pub fn character(&self, character: Character) -> Option<&CharacterDef> {
        self.characters.get(&character)
    }

    /// Relic slots of a run with `character`: the lower of `run.ron` and
    /// the character's cap; `None` when neither sets one (unlimited).
    #[must_use]
    pub fn relic_slots(&self, character: Character) -> Option<u32> {
        let cap = self.character(character).and_then(|c| c.rule.relic_slots);
        match (self.run.relic_slots, cap) {
            (Some(a), Some(b)) => Some(a.min(b)),
            (a, b) => a.or(b),
        }
    }

    /// Whether a run with `character` may hold `count` relics.
    #[must_use]
    pub fn relic_fits(&self, character: Character, count: usize) -> bool {
        self.relic_slots(character)
            .is_none_or(|slots| count <= usize::try_from(slots).unwrap_or(usize::MAX))
    }

    /// Whether a run with `character` holding `owned` relics has room for
    /// one more.
    #[must_use]
    pub fn relic_slot_free(&self, character: Character, owned: usize) -> bool {
        self.relic_fits(character, owned.saturating_add(1))
    }

    /// Einmischen per game of a run with `character`: `run.ron` plus the
    /// character's rule, never below 0. Relics may add more per game
    /// ([`Effect::ExtraRedeal`]).
    #[must_use]
    pub fn redeals(&self, character: Character) -> u32 {
        let delta = self.character(character).map_or(0, |c| c.rule.redeals);
        self.run.redeals.saturating_add_signed(delta)
    }

    /// How many times the Groschen of a won series count for `character`
    /// (at least 1).
    #[must_use]
    pub fn payout_factor(&self, character: Character) -> u32 {
        self.character(character)
            .map_or(1, |c| c.rule.payout_factor)
            .max(1)
    }

    /// An encounter by id.
    #[must_use]
    pub fn encounter(&self, id: &EncounterId) -> Option<&EncounterDef> {
        self.encounters.get(id)
    }

    /// An unlock by id.
    #[must_use]
    pub fn unlock(&self, id: &UnlockId) -> Option<&UnlockDef> {
        self.unlocks.get(id)
    }

    /// A relic by id.
    #[must_use]
    pub fn relic(&self, id: &RelicId) -> Option<&RelicDef> {
        self.relics.get(id)
    }

    /// A card modification by id.
    #[must_use]
    pub fn card_mod(&self, id: &CardModId) -> Option<&CardModDef> {
        self.card_mods.get(id)
    }

    /// A boss by id.
    #[must_use]
    pub fn boss(&self, id: &BossId) -> Option<&BossDef> {
        self.bosses.get(id)
    }

    /// A Stammgast by id.
    #[must_use]
    pub fn guest(&self, id: &GuestId) -> Option<&GuestDef> {
        self.guests.get(id)
    }

    /// A consumable by id.
    #[must_use]
    pub fn consumable(&self, id: &ConsumableId) -> Option<&ConsumableDef> {
        self.consumables.get(id)
    }
}

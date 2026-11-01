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
    ActiveSource, HookContext, ResolvedDeck, card_mod_sources, evaluate, resolve_deck,
};
pub use score::{
    Change, Lever, ScoreBreakdown, ScoreLine, ScoreStep, ScoreTerm, Source, score_game, score_parts,
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
    /// Id of an encounter (Ereignis between two series).
    EncounterId
);
content_id!(
    /// Id of an unlock (Freischaltung).
    UnlockId
);

/// Charakter: who the player is in a run (DESIGN 9). The mechanics differ
/// in code (which games may be declared); everything else about a
/// character is data in `characters.ron` ([`CharacterDef`]).
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Default, Serialize, Deserialize,
)]
pub enum Character {
    /// Der Alleinspieler: suit games and grand. The standard character
    /// and the measure for all balancing.
    #[default]
    Declarer,
    /// Der Nullspieler: Null only (ADR 0003).
    NullPlayer,
}

impl Character {
    /// All characters in display order.
    pub const ALL: [Character; 2] = [Character::Declarer, Character::NullPlayer];

    /// The id as text, equal to its serialized form.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Character::Declarer => "Declarer",
            Character::NullPlayer => "NullPlayer",
        }
    }

    /// Whether this character plays Null (and only Null).
    #[must_use]
    pub const fn plays_null(self) -> bool {
        matches!(self, Character::NullPlayer)
    }
}

impl fmt::Display for Character {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Which characters a content item is for. Empty means every character.
pub type CharacterPool = Vec<Character>;

/// Whether `character` may meet an item with this pool.
#[must_use]
pub fn in_pool(pool: &[Character], character: Character) -> bool {
    pool.is_empty() || pool.contains(&character)
}

/// How a character's start deck differs from the standard 32 cards:
/// `remove` takes printed cards out (each leaves a blank), `add` puts new
/// cards in (filling blanks first).
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct DeckSpec {
    /// Printed cards to remove, one instance each.
    pub remove: Vec<Card>,
    /// Printed cards to add.
    pub add: Vec<Card>,
}

/// A character as data (DESIGN 9): start deck, start relics, Zielwerte and
/// for the Nullspieler the Null scoring.
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
    /// Start deck.
    #[serde(default)]
    pub deck: DeckSpec,
    /// Zielwerte per section; `None` uses `run.ron`. Must have as many
    /// sections as `run.ron`.
    #[serde(default)]
    pub targets: Option<Vec<(u64, u64, u64)>>,
    /// How Null games are scored. Required for a character that plays
    /// Null.
    #[serde(default)]
    pub null_scoring: Option<NullScoring>,
}

/// What an encounter (Ereignis, DESIGN 13) offers. The concrete options
/// are drawn when the encounter opens ([`crate::run::EncounterOption`]).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EncounterKind {
    /// Stammtisch-Streit: choose one of `offers` house rules (not owned,
    /// unlocked, for the character); it applies until the end of the
    /// section (Abend) without taking a slot.
    EveningRule {
        /// House rules on offer.
        offers: u32,
    },
    /// Der Kartengeber: a random card of the deck gets a card modification
    /// for free, or for `price` Groschen the player chooses the card. The
    /// modification is `card_mod`, or random from the character's pool.
    MarkCard {
        /// Price for choosing the card.
        price: u32,
        /// The modification; `None` draws one.
        #[serde(default)]
        card_mod: Option<CardModId>,
    },
    /// Alter Bekannter: swap an owned relic for a random other one (not
    /// owned, unlocked, for the character).
    SwapRelic,
    /// Letzte Runde: the next game declared counts `factor` times, won or
    /// lost.
    DoubleNextGame {
        /// Factor on the Spielwert.
        factor: u32,
    },
}

/// An encounter (Ereignis / Rast, DESIGN 13): may come instead of the
/// Tresen between two series.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EncounterDef {
    /// Unique id.
    pub id: EncounterId,
    /// German display name.
    pub name: String,
    /// German description.
    pub text: String,
    /// What it offers.
    pub kind: EncounterKind,
    /// Weight when drawn (default 1).
    #[serde(default = "one")]
    pub weight: u32,
    /// Characters that meet it (empty: all).
    #[serde(default)]
    pub characters: CharacterPool,
}

const fn one() -> u32 {
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
    },
    /// Win a Null game from this Null-Risikostufe on.
    WinNull(NullRisk),
    /// Reach this section (from 1).
    ReachSection(u32),
    /// Win a boss series.
    BeatBoss {
        /// Only this boss (`None`: any).
        #[serde(default)]
        boss: Option<BossId>,
        /// Only without passing in that series.
        #[serde(default)]
        without_pass: bool,
    },
    /// Win a series with at least this many games left.
    WinSeriesWithGamesLeft(u32),
    /// Win a run.
    WinRun(Option<Character>),
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
    /// A card modification joins the random draws (Stempelkissen,
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
    /// Characters that meet it at the Tresen (empty: all).
    #[serde(default)]
    pub characters: CharacterPool,
}

/// A card modification (gezinkte Karte). It acts on the card instance it is
/// attached to, whoever holds it (ADR 0005), and only while the card is in
/// play and not blanked out.
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
    /// Characters whose random modifications may draw it (empty: all).
    #[serde(default)]
    pub characters: CharacterPool,
    /// A curse (Fluch, DESIGN 7.2): a drawback, usually paid for in
    /// Groschen when attached.
    #[serde(default)]
    pub curse: bool,
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
}

/// What a consumable does when used. These are player actions, not hooks.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConsumableAction {
    /// Remove a chosen card from the deck for good (at the Tresen).
    RemoveCard,
    /// Attach a card modification to a chosen card (at the Tresen). `None`
    /// picks one at random with the run's RNG.
    AddCardMod(Option<CardModId>),
    /// More passes in the current series.
    ExtraPasses(u32),
    /// Shuffle and deal again before declaring, without using up a game.
    Redeal,
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
}

impl RarityWeights {
    /// Weight of one rarity.
    #[must_use]
    pub const fn weight(&self, rarity: Rarity) -> u32 {
        match rarity {
            Rarity::Common => self.common,
            Rarity::Uncommon => self.uncommon,
            Rarity::Rare => self.rare,
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
}

/// Run structure as data: target values, limits, slots, economy, bosses.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RunConfig {
    /// Zielwerte per section: (series 1, series 2, boss series).
    pub targets: Vec<(u64, u64, u64)>,
    /// Spiellimit per series.
    pub game_limit: u32,
    /// Passes per series.
    pub passes: u32,
    /// Slots for house rules and figures together.
    pub relic_slots: u32,
    /// Slots for consumables.
    pub consumable_slots: u32,
    /// Groschen rules.
    pub economy: Economy,
    /// Tresen layout.
    pub shop: ShopConfig,
    /// Bosses a boss series may draw from.
    pub bosses: Vec<BossId>,
    /// Boss of the last section's boss series (Endboss), not drawn.
    #[serde(default)]
    pub final_boss: Option<BossId>,
    /// Chance that an encounter comes instead of the Tresen between two
    /// series (0.0 to 1.0).
    #[serde(default = "Factor::zero")]
    pub encounter_chance: Factor,
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
        if stage == Stage::OnAttach && !is_card_mod {
            return Err(ContentError(format!(
                "{what}: OnAttach only works on card modifications"
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
            if let ConsumableAction::AddCardMod(Some(id)) = &consumable.action {
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
        if self.run.game_limit == 0 {
            return Err(ContentError("run: game limit 0".into()));
        }
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
        if self.run.encounter_chance > Factor::ONE {
            return Err(ContentError("run: encounter chance above 1".into()));
        }
        for boss in self.bosses.values() {
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
        self.validate_unlocks()
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
            if def.start_relics.len() > usize::try_from(self.run.relic_slots).unwrap_or(0) {
                return Err(ContentError(format!(
                    "{what}: more start relics than slots"
                )));
            }
            if let Some(targets) = &def.targets {
                if targets.len() != self.run.targets.len() {
                    return Err(ContentError(format!(
                        "{what}: {} sections of Zielwerte, run.ron has {}",
                        targets.len(),
                        self.run.targets.len()
                    )));
                }
            }
            if def.character.plays_null() && def.null_scoring.is_none() {
                return Err(ContentError(format!(
                    "{what}: plays Null without null_scoring"
                )));
            }
            let mut deck = crate::deck::Deck::standard();
            deck.apply_spec(&def.deck)
                .map_err(|e| ContentError(format!("{what}: start deck: {e}")))?;
        }
        Ok(())
    }

    fn validate_encounters(&self) -> Result<(), ContentError> {
        for def in self.encounters.values() {
            if let EncounterKind::MarkCard {
                card_mod: Some(id), ..
            } = &def.kind
            {
                if !self.card_mods.contains_key(id) {
                    return Err(ContentError(format!(
                        "encounter {}: unknown card mod {id}",
                        def.id
                    )));
                }
            }
        }
        if self.run.encounter_chance > Factor::zero() && self.encounters.is_empty() {
            return Err(ContentError(
                "run: encounter chance without encounters".into(),
            ));
        }
        Ok(())
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

    /// A consumable by id.
    #[must_use]
    pub fn consumable(&self, id: &ConsumableId) -> Option<&ConsumableDef> {
        self.consumables.get(id)
    }
}

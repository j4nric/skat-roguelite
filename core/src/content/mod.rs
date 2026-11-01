//! Data-driven content and the effect hook API (design document 14.2).
//!
//! House rules, card modifications, bosses, consumables and the run
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

pub use effects::{CardFilter, Condition, Effect, Hook, Stage};
pub use engine::{
    ActiveSource, HookContext, ResolvedDeck, card_mod_sources, evaluate, resolve_deck,
};
pub use score::{
    Change, Lever, ScoreBreakdown, ScoreLine, ScoreStep, ScoreTerm, Source, score_game,
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

/// The embedded RON sources, in load order.
pub const SOURCES: [(&str, &str); 5] = [
    (
        "house_rules.ron",
        include_str!("../../content/house_rules.ron"),
    ),
    ("card_mods.ron", include_str!("../../content/card_mods.ron")),
    ("bosses.ron", include_str!("../../content/bosses.ron")),
    (
        "consumables.ron",
        include_str!("../../content/consumables.ron"),
    ),
    ("run.ron", include_str!("../../content/run.ron")),
];

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
        for effect in hook.then() {
            check(stage, effect)?;
        }
    }
    Ok(())
}

impl Content {
    /// Parse and validate the embedded content.
    pub fn load() -> Result<Content, ContentError> {
        let [house_rules, card_mods, bosses, consumables, run] = SOURCES;
        Content::from_sources(house_rules.1, card_mods.1, bosses.1, consumables.1, run.1)
    }

    /// Parse and validate content from RON texts (tests, modding).
    pub fn from_sources(
        relics: &str,
        card_mods: &str,
        bosses: &str,
        consumables: &str,
        run: &str,
    ) -> Result<Content, ContentError> {
        let content = Content {
            relics: index("relics", parse::<Vec<RelicDef>>("relics", relics)?, |r| {
                &r.id
            })?,
            card_mods: index(
                "card_mods",
                parse::<Vec<CardModDef>>("card_mods", card_mods)?,
                |m| &m.id,
            )?,
            bosses: index("bosses", parse::<Vec<BossDef>>("bosses", bosses)?, |b| {
                &b.id
            })?,
            consumables: index(
                "consumables",
                parse::<Vec<ConsumableDef>>("consumables", consumables)?,
                |c| &c.id,
            )?,
            run: parse("run", run)?,
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
        Ok(())
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

//! `skat_wasm`: the WebAssembly shell around `skat_core` for the web app.
//!
//! A thin layer without rules and without state of its own besides the
//! wrapped run: create a run from a seed, load and save it, apply a
//! [`RunAction`] given as JSON and hand back the events plus the new
//! [`RunView`] as JSON. Everything the web app shows is computed by the
//! core (see `skat_core::run::view`).
//!
//! The browser only gets what the player may know: the events go out as
//! [`PlayerEvent`]s (no defender hands, no untouched skat, no talon), the
//! view holds nothing hidden either. The save is the full run state and is
//! only meant to be stored and handed back.
//!
//! Errors are thrown as a JSON text `{"code": "...", "message": "..."}`.
//! `code` is a stable [`RunErrorCode`] name or one of the shell's own
//! codes ([`SHELL_ERROR_CODES`]); `message` is English, for logs only.
//!
//! The meta state across runs (unlocks, statistics, DESIGN 9.3) is a
//! separate object ([`MetaSession`], [`WasmMeta`]) with its own save: the
//! web app feeds it the events of every step and starts new runs from it
//! ([`WasmRun::new_run_with`]).
//!
//! The logic lives in [`Session`] and [`MetaSession`], which are plain
//! Rust and tested natively; [`WasmRun`] and [`WasmMeta`] only adapt them
//! to `wasm-bindgen`.

use std::collections::BTreeMap;
use std::sync::OnceLock;

use serde::{Deserialize, Serialize};
use skat_core::content::{Character, Content, UnlockId};
use skat_core::deck::deal_odds;
use skat_core::run::meta::{META_VERSION, MetaState, MetaView};
use skat_core::run::player::{PlayerEvent, player_events};
use skat_core::run::view::RunView;
use skat_core::run::{Run, RunAction, RunError, RunErrorCode, RunStart};
use wasm_bindgen::prelude::*;

/// Version of the save format. Saves with another version are rejected,
/// so the web app can start a new run instead of crashing.
pub const SAVE_VERSION: u32 = 1;

/// Error codes of the shell itself, next to the [`RunErrorCode`]s of the
/// core: `InvalidSeed` (not a decimal u64), `InvalidAction` (not a
/// `RunAction` in JSON), `InvalidSave` (not a save), `UnsupportedSave`
/// (another save version), `Internal` (content or serialization failed),
/// `InvalidCharacter` (not `"Declarer"` or `"NullPlayer"`),
/// `CharacterLocked` (the meta state has not unlocked it),
/// `UnknownCharacter` (the content has no such character),
/// `InvalidMeta` (not a meta save), `UnsupportedMeta` (another meta
/// version), `InvalidEvents` (not a list of player events).
pub const SHELL_ERROR_CODES: [&str; 11] = [
    "InvalidSeed",
    "InvalidAction",
    "InvalidSave",
    "UnsupportedSave",
    "Internal",
    "InvalidCharacter",
    "CharacterLocked",
    "UnknownCharacter",
    "InvalidMeta",
    "UnsupportedMeta",
    "InvalidEvents",
];

/// An error as thrown to JavaScript.
#[derive(Serialize)]
struct ErrorJson<'a> {
    code: &'a str,
    message: String,
}

/// The JSON text of an error.
fn error_json(code: &str, message: impl std::fmt::Display) -> String {
    let error = ErrorJson {
        code,
        message: message.to_string(),
    };
    serde_json::to_string(&error).unwrap_or_else(|_| format!(r#"{{"code":"{code}","message":""}}"#))
}

fn run_error(e: &RunError) -> String {
    let code: RunErrorCode = e.code();
    error_json(code.as_str(), e)
}

fn internal(e: impl std::fmt::Display) -> String {
    error_json("Internal", e)
}

/// A saved run.
#[derive(Serialize, Deserialize)]
struct Save {
    version: u32,
    run: Run,
}

/// Result of an action: what happened, as the player may see it, and how
/// the table looks now.
#[derive(Serialize)]
struct Step {
    events: Vec<PlayerEvent>,
    view: RunView,
}

/// Name and text of one content item.
#[derive(Serialize)]
struct Entry<'a> {
    name: &'a str,
    text: &'a str,
}

/// Names and texts of all content by id, so the web app can label every
/// source of a score line, also of relics no longer owned.
#[derive(Serialize)]
struct Catalog<'a> {
    relics: BTreeMap<&'a str, Entry<'a>>,
    card_mods: BTreeMap<&'a str, Entry<'a>>,
    bosses: BTreeMap<&'a str, Entry<'a>>,
    consumables: BTreeMap<&'a str, Entry<'a>>,
    characters: BTreeMap<&'a str, Entry<'a>>,
    encounters: BTreeMap<&'a str, Entry<'a>>,
    unlocks: BTreeMap<&'a str, Entry<'a>>,
}

fn entries<'a, T: 'a>(
    items: impl Iterator<Item = &'a T>,
    key: impl Fn(&'a T) -> &'a str,
    name: impl Fn(&'a T) -> &'a str,
    text: impl Fn(&'a T) -> &'a str,
) -> BTreeMap<&'a str, Entry<'a>> {
    items
        .map(|item| {
            (
                key(item),
                Entry {
                    name: name(item),
                    text: text(item),
                },
            )
        })
        .collect()
}

/// The content catalog as JSON.
pub fn catalog_json() -> Result<String, String> {
    let content = content()?;
    let catalog = Catalog {
        relics: content
            .relics
            .values()
            .map(|d| {
                (
                    d.id.as_str(),
                    Entry {
                        name: &d.name,
                        text: &d.text,
                    },
                )
            })
            .collect(),
        card_mods: content
            .card_mods
            .values()
            .map(|d| {
                (
                    d.id.as_str(),
                    Entry {
                        name: &d.name,
                        text: &d.text,
                    },
                )
            })
            .collect(),
        bosses: content
            .bosses
            .values()
            .map(|d| {
                (
                    d.id.as_str(),
                    Entry {
                        name: &d.name,
                        text: &d.text,
                    },
                )
            })
            .collect(),
        consumables: content
            .consumables
            .values()
            .map(|d| {
                (
                    d.id.as_str(),
                    Entry {
                        name: &d.name,
                        text: &d.text,
                    },
                )
            })
            .collect(),
        characters: entries(
            content.characters.values(),
            |d| d.character.as_str(),
            |d| &d.name,
            |d| &d.text,
        ),
        encounters: entries(
            content.encounters.values(),
            |d| d.id.as_str(),
            |d| &d.name,
            |d| &d.text,
        ),
        unlocks: entries(
            content.unlocks.values(),
            |d| d.id.as_str(),
            |d| &d.name,
            |d| &d.text,
        ),
    };
    serde_json::to_string(&catalog).map_err(internal)
}

/// Where any one card lands when a deck of `deck_size` cards is dealt
/// (DESIGN 7.4), as JSON: `{deck_size, hand, skat, hand_or_skat,
/// defenders, talon}`, each odds as `{numerator, denominator, permille}`.
pub fn deal_odds_json(deck_size: u32) -> Result<String, String> {
    let size = usize::try_from(deck_size).map_err(internal)?;
    serde_json::to_string(&deal_odds(size)).map_err(internal)
}

/// The embedded content, parsed once.
fn content() -> Result<&'static Content, String> {
    static CONTENT: OnceLock<Content> = OnceLock::new();
    if let Some(content) = CONTENT.get() {
        return Ok(content);
    }
    let loaded = Content::load().map_err(internal)?;
    Ok(CONTENT.get_or_init(|| loaded))
}

/// A run plus the content it plays with. Plain Rust, usable natively.
#[derive(Debug, Clone)]
pub struct Session {
    content: &'static Content,
    run: Run,
}

fn parse_seed(seed: &str) -> Result<u64, String> {
    seed.trim()
        .parse()
        .map_err(|_| error_json("InvalidSeed", format!("invalid seed {seed:?}")))
}

fn parse_character(character: &str) -> Result<Character, String> {
    serde_json::from_value(serde_json::Value::String(character.trim().to_string())).map_err(|_| {
        error_json(
            "InvalidCharacter",
            format!("invalid character {character:?}"),
        )
    })
}

impl Session {
    /// Start a new run of the Alleinspieler with every relic unlocked (the
    /// web app before it keeps a meta state). `seed` is a decimal u64.
    pub fn new(seed: &str) -> Result<Session, String> {
        let seed = parse_seed(seed)?;
        let content = content()?;
        let (run, _events) = Run::new(content, seed).map_err(|e| run_error(&e))?;
        Ok(Session { content, run })
    }

    /// Start a new run with `character` (`"Declarer"` or `"NullPlayer"`)
    /// from a meta state: the character must be unlocked, locked relics
    /// stay off the Tresen. Returns the session and the events of the
    /// start as player events (feed them to the meta state).
    pub fn new_with(
        seed: &str,
        character: &str,
        meta: &MetaState,
    ) -> Result<(Session, Vec<PlayerEvent>), String> {
        let seed = parse_seed(seed)?;
        let character = parse_character(character)?;
        let content = content()?;
        let start: RunStart = meta
            .run_start(content, character)
            .map_err(|e| error_json(e.code(), &e))?;
        let (run, events) = Run::new_with(content, seed, &start).map_err(|e| run_error(&e))?;
        Ok((Session { content, run }, player_events(events)))
    }

    /// Load a run saved with [`Session::save`].
    pub fn load(state: &str) -> Result<Session, String> {
        let save: Save = serde_json::from_str(state).map_err(|e| error_json("InvalidSave", e))?;
        if save.version != SAVE_VERSION {
            return Err(error_json(
                "UnsupportedSave",
                format!("unsupported save version {}", save.version),
            ));
        }
        Ok(Session {
            content: content()?,
            run: save.run,
        })
    }

    /// The run as JSON, for `localStorage`.
    pub fn save(&self) -> Result<String, String> {
        serde_json::to_string(&Save {
            version: SAVE_VERSION,
            run: self.run.clone(),
        })
        .map_err(internal)
    }

    /// The current view as JSON.
    pub fn view(&self) -> Result<String, String> {
        serde_json::to_string(&self.run.view(self.content)).map_err(internal)
    }

    /// Apply a [`RunAction`] given as JSON. Returns `{ events, view }` as
    /// JSON, the events as [`PlayerEvent`]s. On error the run is unchanged
    /// and the error is the JSON text described in the crate docs.
    pub fn apply(&mut self, action: &str) -> Result<String, String> {
        let action: RunAction =
            serde_json::from_str(action).map_err(|e| error_json("InvalidAction", e))?;
        let events = self
            .run
            .apply(self.content, action)
            .map_err(|e| run_error(&e))?;
        serde_json::to_string(&Step {
            events: player_events(events),
            view: self.run.view(self.content),
        })
        .map_err(internal)
    }

    /// The wrapped run.
    #[must_use]
    pub const fn run(&self) -> &Run {
        &self.run
    }
}

/// A saved meta state.
#[derive(Serialize, Deserialize)]
struct MetaSave {
    version: u32,
    meta: MetaState,
}

/// Result of feeding events to the meta state: the unlocks just earned
/// and the new meta view.
#[derive(Serialize)]
struct MetaStep {
    earned: Vec<UnlockId>,
    view: MetaView,
}

/// The meta state across runs plus the content. Plain Rust, usable
/// natively.
#[derive(Debug, Clone)]
pub struct MetaSession {
    content: &'static Content,
    meta: MetaState,
}

impl MetaSession {
    /// A new player: nothing unlocked.
    pub fn fresh() -> Result<MetaSession, String> {
        Ok(MetaSession {
            content: content()?,
            meta: MetaState::new(),
        })
    }

    /// Load a meta state saved with [`MetaSession::save`].
    pub fn load(state: &str) -> Result<MetaSession, String> {
        let save: MetaSave =
            serde_json::from_str(state).map_err(|e| error_json("InvalidMeta", e))?;
        if save.version != META_VERSION {
            return Err(error_json(
                "UnsupportedMeta",
                format!("unsupported meta version {}", save.version),
            ));
        }
        Ok(MetaSession {
            content: content()?,
            meta: save.meta,
        })
    }

    /// The meta state as JSON, for `localStorage` (apart from the run).
    pub fn save(&self) -> Result<String, String> {
        serde_json::to_string(&MetaSave {
            version: META_VERSION,
            meta: self.meta.clone(),
        })
        .map_err(internal)
    }

    /// The meta view as JSON.
    pub fn view(&self) -> Result<String, String> {
        serde_json::to_string(&self.meta.view(self.content)).map_err(internal)
    }

    /// Feed the player events of a step (the `events` array of
    /// [`Session::apply`], or of the run start) as JSON. Returns
    /// `{ earned, view }` as JSON.
    pub fn observe(&mut self, events: &str) -> Result<String, String> {
        let events: Vec<PlayerEvent> =
            serde_json::from_str(events).map_err(|e| error_json("InvalidEvents", e))?;
        let earned = self.meta.observe(self.content, &events);
        serde_json::to_string(&MetaStep {
            earned,
            view: self.meta.view(self.content),
        })
        .map_err(internal)
    }

    /// The wrapped meta state.
    #[must_use]
    pub const fn meta(&self) -> &MetaState {
        &self.meta
    }
}

/// Names and texts of all content by id, as JSON.
#[wasm_bindgen]
pub fn catalog() -> Result<String, JsError> {
    catalog_json().map_err(|e| JsError::new(&e))
}

/// Where any one card lands when a deck of `deckSize` cards is dealt, as
/// JSON (see [`deal_odds_json`]).
#[wasm_bindgen(js_name = dealOdds)]
pub fn deal_odds_js(deck_size: u32) -> Result<String, JsError> {
    deal_odds_json(deck_size).map_err(|e| JsError::new(&e))
}

/// The run as seen from JavaScript.
#[wasm_bindgen]
pub struct WasmRun {
    session: Session,
    /// Player events of the start, until fetched.
    start: Option<String>,
}

/// The meta state as seen from JavaScript.
#[wasm_bindgen]
pub struct WasmMeta {
    session: MetaSession,
}

#[wasm_bindgen]
impl WasmMeta {
    /// A new player: nothing unlocked.
    pub fn fresh() -> Result<WasmMeta, JsError> {
        MetaSession::fresh()
            .map(|session| WasmMeta { session })
            .map_err(|e| JsError::new(&e))
    }

    /// Load a saved meta state.
    pub fn load(state: &str) -> Result<WasmMeta, JsError> {
        MetaSession::load(state)
            .map(|session| WasmMeta { session })
            .map_err(|e| JsError::new(&e))
    }

    /// Serialize the meta state.
    pub fn save(&self) -> Result<String, JsError> {
        self.session.save().map_err(|e| JsError::new(&e))
    }

    /// The meta view as JSON.
    pub fn view(&self) -> Result<String, JsError> {
        self.session.view().map_err(|e| JsError::new(&e))
    }

    /// Feed player events (JSON array); returns `{ earned, view }`.
    pub fn observe(&mut self, events: &str) -> Result<String, JsError> {
        self.session.observe(events).map_err(|e| JsError::new(&e))
    }
}

#[wasm_bindgen]
impl WasmRun {
    /// Start a new run of the Alleinspieler with a decimal seed, every
    /// relic unlocked.
    #[wasm_bindgen(js_name = newRun)]
    pub fn new_run(seed: &str) -> Result<WasmRun, JsError> {
        Session::new(seed)
            .map(|session| WasmRun {
                session,
                start: None,
            })
            .map_err(|e| JsError::new(&e))
    }

    /// Start a new run with a character (`"Declarer"`, `"NullPlayer"`)
    /// from the meta state: locked relics stay off the Tresen, a locked
    /// character is refused (`CharacterLocked`). Feed
    /// [`WasmRun::start_events`] to the meta state once.
    #[wasm_bindgen(js_name = newRunWith)]
    pub fn new_run_with(seed: &str, character: &str, meta: &WasmMeta) -> Result<WasmRun, JsError> {
        let (session, events) =
            Session::new_with(seed, character, &meta.session.meta).map_err(|e| JsError::new(&e))?;
        let start = serde_json::to_string(&events).map_err(|e| JsError::new(&internal(e)))?;
        Ok(WasmRun {
            session,
            start: Some(start),
        })
    }

    /// The player events of the run start as JSON (once after
    /// [`WasmRun::new_run_with`], then `"[]"`).
    #[wasm_bindgen(js_name = startEvents)]
    pub fn start_events(&mut self) -> String {
        self.start.take().unwrap_or_else(|| "[]".to_string())
    }

    /// Load a saved run.
    pub fn load(state: &str) -> Result<WasmRun, JsError> {
        Session::load(state)
            .map(|session| WasmRun {
                session,
                start: None,
            })
            .map_err(|e| JsError::new(&e))
    }

    /// Serialize the run.
    pub fn save(&self) -> Result<String, JsError> {
        self.session.save().map_err(|e| JsError::new(&e))
    }

    /// The current view as JSON.
    pub fn view(&self) -> Result<String, JsError> {
        self.session.view().map_err(|e| JsError::new(&e))
    }

    /// Apply an action (JSON) and return `{ events, view }` as JSON.
    pub fn apply(&mut self, action: &str) -> Result<String, JsError> {
        self.session.apply(action).map_err(|e| JsError::new(&e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use skat_core::run::RunPhase;
    use skat_core::run::bot::RunBot;

    #[test]
    fn new_view_apply_save_load() {
        let mut session = Session::new("42").expect("new");
        let view: serde_json::Value =
            serde_json::from_str(&session.view().expect("view")).expect("json");
        assert_eq!(view["seed"], "42");
        assert_eq!(view["phase"], "Game");
        assert_eq!(view["game"]["hand"].as_array().map(Vec::len), Some(10));
        assert!(
            view["game"]["hand"][0]["id"].is_u64(),
            "card fields are flattened"
        );

        let step = session
            .apply(r#"{"Declare":{"game":"Grand","risk":"Normal"}}"#)
            .expect("declare");
        let step: serde_json::Value = serde_json::from_str(&step).expect("json");
        assert_eq!(step["view"]["game"]["phase"], "Discarding");
        assert!(step["events"].as_array().is_some_and(|e| !e.is_empty()));

        let saved = session.save().expect("save");
        let loaded = Session::load(&saved).expect("load");
        assert_eq!(loaded.run(), session.run());
        assert!(Session::load(&saved.replace("\"version\":1", "\"version\":99")).is_err());
        assert!(Session::load("garbage").is_err());
        assert!(Session::new("not a number").is_err());
        assert!(session.apply(r#""LeaveShop""#).is_err(), "wrong phase");
        assert!(session.apply("nonsense").is_err());
    }

    /// Instance ids of every card the player must not know right after the
    /// deal: the defenders' hands and the skat.
    fn hidden_ids(session: &Session) -> Vec<u64> {
        use skat_core::game::Seat;
        let game = &session.run().game().expect("game").game;
        let mut ids: Vec<u64> = Seat::DEFENDERS
            .iter()
            .flat_map(|&s| game.hand(s).iter().map(|c| u64::from(c.id)))
            .collect();
        ids.extend(game.skat().iter().map(|c| u64::from(c.id)));
        ids
    }

    /// Every dealt card in a JSON value: objects with `id` and
    /// `card_points` (deck cards in the view carry `mods`, not Augen, and
    /// are public anyway).
    fn dealt_card_ids(value: &serde_json::Value, out: &mut Vec<u64>) {
        match value {
            serde_json::Value::Object(map) => {
                if let (Some(id), true) = (map.get("id"), map.contains_key("card_points")) {
                    out.extend(id.as_u64());
                }
                map.values().for_each(|v| dealt_card_ids(v, out));
            }
            serde_json::Value::Array(items) => items.iter().for_each(|v| dealt_card_ids(v, out)),
            _ => {}
        }
    }

    #[test]
    fn the_player_json_holds_no_hidden_card_after_a_deal() {
        let mut session = Session::new("42").expect("new");
        let step = session.apply(r#""Pass""#).expect("pass");
        let hidden = hidden_ids(&session);
        assert_eq!(hidden.len(), 22);
        let json: serde_json::Value = serde_json::from_str(&step).expect("json");
        let dealt = json["events"]
            .as_array()
            .expect("events")
            .iter()
            .find_map(|e| e["Game"].get("Dealt"))
            .expect("a new deal");
        assert_eq!(dealt["hand"].as_array().map(Vec::len), Some(10));
        assert_eq!(dealt["hand_sizes"], serde_json::json!([10, 10, 10]));
        assert!(dealt.get("hands").is_none() && dealt.get("skat").is_none());
        let mut seen = Vec::new();
        dealt_card_ids(&json, &mut seen);
        assert!(!seen.is_empty());
        for id in &hidden {
            assert!(!seen.contains(id), "hidden card {id} in {step}");
        }
        let view: serde_json::Value =
            serde_json::from_str(&session.view().expect("view")).expect("json");
        let mut seen = Vec::new();
        dealt_card_ids(&view, &mut seen);
        for id in &hidden {
            assert!(!seen.contains(id), "hidden card {id} in the view");
        }
    }

    #[test]
    fn errors_carry_stable_codes() {
        let mut session = Session::new("42").expect("new");
        let code = |e: String| -> String {
            let json: serde_json::Value = serde_json::from_str(&e).expect("error json");
            assert!(json["message"].is_string());
            json["code"].as_str().expect("code").to_string()
        };
        let err = session.apply(r#""LeaveShop""#).expect_err("wrong phase");
        assert_eq!(code(err), "WrongPhase");
        let err = session.apply("nonsense").expect_err("bad json");
        assert_eq!(code(err), "InvalidAction");
        let err = session.apply(r#"{"Play":999}"#).expect_err("not held");
        assert_eq!(code(err), "UnknownCard");
        let err = session.apply(r#""RedrawSkat""#).expect_err("no relic");
        assert_eq!(code(err), "NoRedealLeft");
        assert_eq!(code(Session::new("x").expect_err("seed")), "InvalidSeed");
        assert_eq!(
            code(Session::load("garbage").expect_err("save")),
            "InvalidSave"
        );
        for c in SHELL_ERROR_CODES {
            assert!(!c.is_empty());
        }
    }

    #[test]
    fn view_shows_boss_next_series_and_odds() {
        let session = Session::new("42").expect("new");
        let view: serde_json::Value =
            serde_json::from_str(&session.view().expect("view")).expect("json");
        assert!(view["upcoming_boss"]["id"].is_string());
        assert!(view["upcoming_boss"]["text"].is_string());
        assert_eq!(view["upcoming_boss"]["target_factor"], 1.0);
        assert_eq!(view["next_series"]["index"], 1);
        assert_eq!(view["next_series"]["is_boss"], false);
        assert!(view["next_series"]["target"].is_u64());
        assert_eq!(view["deal_odds"]["hand_or_skat"]["permille"], 375);
        assert_eq!(view["game"]["declare"]["can_redeal"], false);
        let odds: serde_json::Value =
            serde_json::from_str(&deal_odds_json(33).expect("odds")).expect("json");
        assert_eq!(odds["deck_size"], 33);
        assert_eq!(odds["hand_or_skat"]["numerator"], 12);
        assert_eq!(odds["talon"]["numerator"], 1);
    }

    #[test]
    fn catalog_lists_all_content() {
        let catalog: serde_json::Value =
            serde_json::from_str(&catalog_json().expect("catalog")).expect("json");
        assert_eq!(catalog["relics"]["old_lady"]["name"], "Alte Dame");
        assert!(catalog["card_mods"]["fat"]["text"].is_string());
        assert!(catalog["bosses"]["grandma"]["name"].is_string());
        assert!(catalog["consumables"]["beer"]["name"].is_string());
    }

    #[test]
    fn meta_state_starts_runs_and_learns_from_events() {
        let mut meta = MetaSession::fresh().expect("meta");
        let view: serde_json::Value =
            serde_json::from_str(&meta.view().expect("view")).expect("json");
        let null = view["characters"]
            .as_array()
            .expect("characters")
            .iter()
            .find(|c| c["character"] == "NullPlayer")
            .expect("null player");
        assert_eq!(null["unlocked"], false);
        assert_eq!(null["unlocked_by"], "tour_won");
        assert!(
            view["locked_relics"]
                .as_array()
                .is_some_and(|l| !l.is_empty())
        );
        let code = |e: String| -> String {
            let json: serde_json::Value = serde_json::from_str(&e).expect("error json");
            json["code"].as_str().expect("code").to_string()
        };
        assert_eq!(
            code(Session::new_with("1", "NullPlayer", meta.meta()).expect_err("locked")),
            "CharacterLocked"
        );
        assert_eq!(
            code(Session::new_with("1", "Nobody", meta.meta()).expect_err("bad")),
            "InvalidCharacter"
        );
        let (mut session, start) = Session::new_with("5", "Declarer", meta.meta()).expect("run");
        let start = serde_json::to_string(&start).expect("json");
        meta.observe(&start).expect("observe");
        let content = content().expect("content");
        let mut earned = Vec::new();
        for _ in 0..5_000 {
            let Some(action) = RunBot.next_action(session.run(), content) else {
                break;
            };
            let step = session
                .apply(&serde_json::to_string(&action).expect("json"))
                .expect("bot");
            let step: serde_json::Value = serde_json::from_str(&step).expect("json");
            let events = serde_json::to_string(&step["events"]).expect("json");
            let result: serde_json::Value =
                serde_json::from_str(&meta.observe(&events).expect("observe")).expect("json");
            earned.extend(result["earned"].as_array().cloned().unwrap_or_default());
        }
        let stats: serde_json::Value =
            serde_json::from_str(&meta.view().expect("view")).expect("json");
        assert_eq!(stats["stats"]["runs_finished"], 1);
        let saved = meta.save().expect("save");
        let loaded = MetaSession::load(&saved).expect("load");
        assert_eq!(loaded.meta(), meta.meta());
        assert_eq!(
            code(
                MetaSession::load(&saved.replace("\"version\":1", "\"version\":7")).expect_err("v")
            ),
            "UnsupportedMeta"
        );
        assert_eq!(
            code(MetaSession::load("x").expect_err("bad")),
            "InvalidMeta"
        );
        assert_eq!(
            code(meta.observe("nope").expect_err("bad")),
            "InvalidEvents"
        );
        // Everything unlocked: the Nullspieler plays Null through JSON.
        let all = MetaState::all_unlocked(content);
        let (mut null, _) = Session::new_with("8", "NullPlayer", &all).expect("null");
        let view: serde_json::Value =
            serde_json::from_str(&null.view().expect("view")).expect("json");
        assert_eq!(view["character"]["character"], "NullPlayer");
        let options = &view["game"]["declare"];
        assert_eq!(options["games"][0]["game"], "Null");
        assert_eq!(options["null_risks"].as_array().map(Vec::len), Some(4));
        assert_eq!(options["null_risks"][3]["value"], 92);
        assert!(
            null.apply(r#"{"Declare":{"game":"Grand","risk":"Normal"}}"#)
                .is_err()
        );
        assert!(
            null.apply(r#"{"Declare":{"game":"Null","risk":"SchneiderDeclared"}}"#)
                .is_err()
        );
        let step = null
            .apply(r#"{"Declare":{"game":"Null","risk":"OuvertHand"}}"#)
            .expect("null ouvert hand");
        let step: serde_json::Value = serde_json::from_str(&step).expect("json");
        assert_eq!(step["view"]["game"]["declaration"]["game"], "Null");
        assert_eq!(step["view"]["game"]["declaration"]["risk"], "OuvertHand");
        let catalog: serde_json::Value =
            serde_json::from_str(&catalog_json().expect("catalog")).expect("json");
        assert!(catalog["characters"]["NullPlayer"]["name"].is_string());
        assert!(catalog["encounters"]["last_round"]["name"].is_string());
        assert!(catalog["unlocks"]["tour_won"]["text"].is_string());
    }

    #[test]
    fn encounters_are_chosen_through_json() {
        let content = content().expect("content");
        // Find a seed whose first interlude is an encounter.
        for seed in 0..400u64 {
            let mut session = Session::new(&seed.to_string()).expect("run");
            for _ in 0..400 {
                let view: serde_json::Value =
                    serde_json::from_str(&session.view().expect("view")).expect("json");
                if view["phase"] == "Encounter" {
                    let options = view["encounter"]["options"].as_array().expect("options");
                    let leave = options.len() - 1;
                    assert_eq!(options[leave]["choice"], "Leave");
                    let step = session
                        .apply(&format!(
                            r#"{{"ChooseEncounter":{{"option":{leave},"target":null}}}}"#
                        ))
                        .expect("leave");
                    let step: serde_json::Value = serde_json::from_str(&step).expect("json");
                    assert_eq!(step["view"]["phase"], "Game");
                    assert!(
                        step["events"]
                            .as_array()
                            .expect("events")
                            .iter()
                            .any(|e| e.get("EncounterResolved").is_some())
                    );
                    return;
                }
                if view["phase"] == "Shop" || view["phase"] == "Lost" {
                    break;
                }
                let Some(action) = RunBot.next_action(session.run(), content) else {
                    break;
                };
                session
                    .apply(&serde_json::to_string(&action).expect("json"))
                    .expect("bot");
            }
        }
        panic!("no encounter in 400 seeds");
    }

    #[test]
    fn a_whole_run_through_json() {
        let content = content().expect("content");
        let mut session = Session::new("7").expect("new");
        let mut payouts = 0;
        for _ in 0..5_000 {
            let Some(action) = RunBot.next_action(session.run(), content) else {
                break;
            };
            let json = serde_json::to_string(&action).expect("action json");
            let step = session.apply(&json).expect("bot action");
            let step: serde_json::Value = serde_json::from_str(&step).expect("json");
            for event in step["events"].as_array().expect("events") {
                if let Some(payout) = event.get("SeriesWon").map(|w| &w["payout"]) {
                    let sum = ["base", "games_left", "interest"]
                        .iter()
                        .filter_map(|k| payout[k].as_u64())
                        .sum::<u64>();
                    assert_eq!(payout["total"].as_u64(), Some(sum));
                    payouts += 1;
                }
            }
            let saved = session.save().expect("save");
            session = Session::load(&saved).expect("load");
        }
        assert!(payouts > 0, "seed 7 wins a series");
        assert!(matches!(
            session.run().phase(),
            RunPhase::Won | RunPhase::Lost
        ));
    }
}

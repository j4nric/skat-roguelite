//! `skat_wasm`: the WebAssembly shell around `skat_core` for the web app.
//!
//! A thin layer without rules and without state of its own besides the
//! wrapped run: create a run from a seed, load and save it, apply a
//! [`RunAction`] given as JSON and hand back the [`RunEvent`]s plus the new
//! [`RunView`] as JSON. Everything the web app shows is computed by the
//! core (see `skat_core::run::view`).
//!
//! The logic lives in [`Session`], which is plain Rust and tested natively;
//! [`WasmRun`] only adapts it to `wasm-bindgen`.

use std::collections::BTreeMap;
use std::sync::OnceLock;

use serde::{Deserialize, Serialize};
use skat_core::content::Content;
use skat_core::run::view::RunView;
use skat_core::run::{Run, RunAction, RunEvent};
use wasm_bindgen::prelude::*;

/// Version of the save format. Saves with another version are rejected,
/// so the web app can start a new run instead of crashing.
pub const SAVE_VERSION: u32 = 1;

/// A saved run.
#[derive(Serialize, Deserialize)]
struct Save {
    version: u32,
    run: Run,
}

/// Result of an action: what happened and how the table looks now.
#[derive(Serialize)]
struct Step<'a> {
    events: &'a [RunEvent],
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
    };
    serde_json::to_string(&catalog).map_err(|e| e.to_string())
}

/// The embedded content, parsed once.
fn content() -> Result<&'static Content, String> {
    static CONTENT: OnceLock<Content> = OnceLock::new();
    if let Some(content) = CONTENT.get() {
        return Ok(content);
    }
    let loaded = Content::load().map_err(|e| e.to_string())?;
    Ok(CONTENT.get_or_init(|| loaded))
}

/// A run plus the content it plays with. Plain Rust, usable natively.
#[derive(Debug, Clone)]
pub struct Session {
    content: &'static Content,
    run: Run,
}

impl Session {
    /// Start a new run. `seed` is a decimal u64.
    pub fn new(seed: &str) -> Result<Session, String> {
        let seed: u64 = seed
            .trim()
            .parse()
            .map_err(|_| format!("invalid seed {seed:?}"))?;
        let content = content()?;
        let (run, _events) = Run::new(content, seed).map_err(|e| e.to_string())?;
        Ok(Session { content, run })
    }

    /// Load a run saved with [`Session::save`].
    pub fn load(state: &str) -> Result<Session, String> {
        let save: Save = serde_json::from_str(state).map_err(|e| e.to_string())?;
        if save.version != SAVE_VERSION {
            return Err(format!("unsupported save version {}", save.version));
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
        .map_err(|e| e.to_string())
    }

    /// The current view as JSON.
    pub fn view(&self) -> Result<String, String> {
        serde_json::to_string(&self.run.view(self.content)).map_err(|e| e.to_string())
    }

    /// Apply a [`RunAction`] given as JSON. Returns `{ events, view }` as
    /// JSON. On error the run is unchanged.
    pub fn apply(&mut self, action: &str) -> Result<String, String> {
        let action: RunAction = serde_json::from_str(action).map_err(|e| e.to_string())?;
        let events = self
            .run
            .apply(self.content, action)
            .map_err(|e| e.to_string())?;
        serde_json::to_string(&Step {
            events: &events,
            view: self.run.view(self.content),
        })
        .map_err(|e| e.to_string())
    }

    /// The wrapped run.
    #[must_use]
    pub const fn run(&self) -> &Run {
        &self.run
    }
}

/// Names and texts of all content by id, as JSON.
#[wasm_bindgen]
pub fn catalog() -> Result<String, JsError> {
    catalog_json().map_err(|e| JsError::new(&e))
}

/// The run as seen from JavaScript.
#[wasm_bindgen]
pub struct WasmRun {
    session: Session,
}

#[wasm_bindgen]
impl WasmRun {
    /// Start a new run with a decimal seed.
    #[wasm_bindgen(js_name = newRun)]
    pub fn new_run(seed: &str) -> Result<WasmRun, JsError> {
        Session::new(seed)
            .map(|session| WasmRun { session })
            .map_err(|e| JsError::new(&e))
    }

    /// Load a saved run.
    pub fn load(state: &str) -> Result<WasmRun, JsError> {
        Session::load(state)
            .map(|session| WasmRun { session })
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
    fn a_whole_run_through_json() {
        let content = content().expect("content");
        let mut session = Session::new("7").expect("new");
        for _ in 0..5_000 {
            let Some(action) = RunBot.next_action(session.run(), content) else {
                break;
            };
            let json = serde_json::to_string(&action).expect("action json");
            session.apply(&json).expect("bot action");
            let saved = session.save().expect("save");
            session = Session::load(&saved).expect("load");
        }
        assert!(matches!(
            session.run().phase(),
            RunPhase::Won | RunPhase::Lost
        ));
    }
}

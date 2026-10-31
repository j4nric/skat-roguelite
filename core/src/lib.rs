//! `skat_core`: the engine-independent heart of the Skat roguelike.
//!
//! Architecture rules (see `docs/DESIGN.md`, section 14):
//! - No dependency on Godot or any rendering/timing concern.
//! - Fully deterministic: every source of randomness is seeded.
//! - Event based: state changes emit events that a frontend may animate and
//!   a simulator may ignore.
//! - Content (relics, card mods, bosses) lives in data files, not in code.
//!
//! Term mapping German ↔ English: `docs/GLOSSARY.md`.

pub mod cards;
pub mod rules;

/// Game state machine for a single Skat game (deal → declare → tricks → score).
/// Filled in during milestone M1.
pub mod game {}

/// Run structure: series, sections, shop, relics, events. Milestone M2.
pub mod run {}

/// Opponent AI. Heuristic first (M1), perfect-information Monte Carlo later (M5).
pub mod ai {}

/// Data-driven content loading (RON) and the effect hook API. Milestone M2.
pub mod content {}

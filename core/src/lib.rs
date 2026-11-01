//! `skat_core`: the engine-independent heart of the Skat roguelike.
//!
//! Architecture rules (see `docs/DESIGN.md`, section 14):
//! - No dependency on Godot or any rendering/timing concern.
//! - Fully deterministic: every source of randomness is seeded.
//! - Event based: state changes emit events that a frontend may animate and
//!   a simulator may ignore.
//! - Content (relics, card mods, bosses) lives in data files, not in code.
//! - WebAssembly safe: no threads, no file system, no clock. Content is
//!   embedded with `include_str!` and parsed at start.
//!
//! Term mapping German ↔ English: `docs/GLOSSARY.md`.

pub mod ai;
pub mod cards;
pub mod content;
pub mod deck;
pub mod game;
pub mod rng;
pub mod rules;
pub mod run;

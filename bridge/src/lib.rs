//! GDExtension bridge between `skat_core` and the Godot 4 frontend.
//!
//! Intentionally empty until milestone M3. The bridge will:
//! - wrap the core's game and run state in Godot classes,
//! - forward core events as Godot signals,
//! - never contain rules or balancing logic of its own.

#[doc(hidden)]
pub fn core_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

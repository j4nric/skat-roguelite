//! Build script of `skat_core`: the content RON files without their
//! comment lines, for embedding ([`crate::content::Sources::embedded`]).
//!
//! The files in `content/` stay as they are; only the copies in
//! `OUT_DIR` lose every line that is nothing but a `//` comment. The
//! content has no string that spans lines, so no string loses a line; a
//! test parses both and compares.

use std::path::Path;

/// The content files, as `Sources::embedded` includes them.
const FILES: [&str; 12] = [
    "house_rules.ron",
    "card_mods.ron",
    "bosses.ron",
    "consumables.ron",
    "run.ron",
    "characters.ron",
    "encounters.ron",
    "unlocks.ron",
    "guests.ron",
    "tour.ron",
    "night.ron",
    "tutorial.ron",
];

fn main() {
    let Some(out) = std::env::var_os("OUT_DIR") else {
        panic!("cargo sets OUT_DIR for build scripts");
    };
    for name in FILES {
        let source = Path::new("content").join(name);
        println!("cargo:rerun-if-changed={}", source.display());
        let text = std::fs::read_to_string(&source)
            .unwrap_or_else(|e| panic!("read {}: {e}", source.display()));
        let mut stripped = String::with_capacity(text.len());
        for line in text.lines() {
            if !line.trim_start().starts_with("//") {
                stripped.push_str(line);
                stripped.push('\n');
            }
        }
        let target = Path::new(&out).join(name);
        std::fs::write(&target, stripped)
            .unwrap_or_else(|e| panic!("write {}: {e}", target.display()));
    }
    println!("cargo:rerun-if-changed=build.rs");
}

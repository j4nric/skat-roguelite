#!/bin/sh
# Build the Rust core as WebAssembly and generate the JS bindings for the
# web app. Output: web/src/wasm/pkg/ (generated, not in git).
#
# Needs the wasm32-unknown-unknown target and the wasm-bindgen CLI in
# exactly the version of the wasm-bindgen crate (wasm/Cargo.toml):
#   rustup target add wasm32-unknown-unknown
#   cargo install wasm-bindgen-cli --version 0.2.130 --locked
set -eu

WASM_BINDGEN_VERSION="0.2.130"

here="$(cd "$(dirname "$0")" && pwd)"
root="$(cd "$here/../.." && pwd)"
out="$root/web/src/wasm/pkg"

if ! command -v wasm-bindgen >/dev/null 2>&1; then
  echo "wasm-bindgen CLI not found. Install it with:" >&2
  echo "  cargo install wasm-bindgen-cli --version $WASM_BINDGEN_VERSION --locked" >&2
  exit 1
fi
found="$(wasm-bindgen --version | awk '{print $2}')"
if [ "$found" != "$WASM_BINDGEN_VERSION" ]; then
  echo "wasm-bindgen CLI $found found, but the crate uses $WASM_BINDGEN_VERSION." >&2
  echo "  cargo install wasm-bindgen-cli --version $WASM_BINDGEN_VERSION --locked --force" >&2
  exit 1
fi

cd "$root"
cargo build -p skat_wasm --target wasm32-unknown-unknown --profile wasm-release
rm -rf "$out"
wasm-bindgen --target web --remove-name-section --remove-producers-section --out-dir "$out" \
  "$root/target/wasm32-unknown-unknown/wasm-release/skat_wasm.wasm"
echo "wasm bindings written to $out"

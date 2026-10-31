#!/usr/bin/env bash
# Claude Code PreToolUse hook for Bash: before a `git commit`, require that
# formatting, clippy and tests are clean. Exit 2 blocks the command and shows
# stderr to the agent. Any other command passes through untouched.
set -u
cmd=$(jq -r '.tool_input.command // empty' 2>/dev/null)
case "$cmd" in
  *"git commit"*|*"git -c "*"commit"*) ;;
  *) exit 0 ;;
esac

cd "${CLAUDE_PROJECT_DIR:-.}" || exit 0
[ -f Cargo.toml ] || exit 0

fail() { echo "commit blocked: $1" >&2; exit 2; }

cargo fmt --all -- --check >/dev/null 2>&1 \
  || fail "cargo fmt --check failed. Run 'cargo fmt' first."
cargo clippy --all-targets --quiet -- -D warnings 2>/tmp/clippy.err \
  || fail "clippy reported warnings:
$(tail -n 40 /tmp/clippy.err)"
cargo test --quiet 2>/tmp/test.err >/dev/null \
  || fail "tests failed:
$(tail -n 40 /tmp/test.err)"
exit 0

#!/usr/bin/env bash
# Claude Code PreToolUse hook for Bash: before a `git commit`, require that
# formatting, clippy and tests are clean in the worktree being committed.
# Exit 2 blocks the command and shows stderr to the agent. Any other command
# passes through untouched.
#
# The target directory is taken from the command itself (`cd DIR && …` or
# `git -C DIR …`), falling back to the hook's cwd, so commits in a separate
# git worktree are checked there and not in the main checkout.
set -u
input=$(cat)
cmd=$(printf '%s' "$input" | jq -r '.tool_input.command // empty' 2>/dev/null)
case "$cmd" in
  *"git commit"*|*"git -c "*" commit"*|*"git -C "*" commit"*) ;;
  *) exit 0 ;;
esac

base=$(printf '%s' "$input" | jq -r '.cwd // empty' 2>/dev/null)
base=${base:-${CLAUDE_PROJECT_DIR:-.}}
target=$(printf '%s' "$cmd" | grep -oE 'git -C [^ ;&|]+' | head -n1 | cut -d' ' -f3)
[ -n "$target" ] || target=$(printf '%s' "$cmd" | grep -oE '(^|[;&|] *)cd [^ ;&|]+' | head -n1 | sed -E 's/.*cd //')
target=${target:-$base}
case "$target" in /*) ;; *) target="$base/$target" ;; esac

cd "$target" 2>/dev/null || exit 0
root=$(git rev-parse --show-toplevel 2>/dev/null) || exit 0
cd "$root" || exit 0
[ -f Cargo.toml ] || exit 0

fail() { echo "commit blocked (checked in $root): $1" >&2; exit 2; }

cargo fmt --all -- --check >/dev/null 2>&1 \
  || fail "cargo fmt --check failed. Run 'cargo fmt' first."
cargo clippy --all-targets --quiet -- -D warnings 2>/tmp/clippy.err \
  || fail "clippy reported warnings:
$(tail -n 40 /tmp/clippy.err)"
cargo test --quiet 2>/tmp/test.err >/dev/null \
  || fail "tests failed:
$(tail -n 40 /tmp/test.err)"
exit 0

#!/usr/bin/env bash
# Claude Code PostToolUse hook: format a Rust file right after it was edited.
# Receives the tool call as JSON on stdin. Never blocks.
set -u
file=$(jq -r '.tool_input.file_path // empty' 2>/dev/null)
case "$file" in
  *.rs)
    if command -v rustfmt >/dev/null 2>&1; then
      rustfmt --edition 2024 "$file" 2>/dev/null || true
    fi
    ;;
esac
exit 0

#!/usr/bin/env sh
# SessionStart hook: prints every rule in ../rules, which Claude Code adds to
# the session's context. Plugins have no rules component; this stands in for
# one. A failure prints nothing and never blocks the session.
rules_dir="$(dirname -- "$0")/../rules"
for rule in "$rules_dir"/*.md; do
  [ -f "$rule" ] || continue
  cat "$rule"
  printf '\n'
done
exit 0

#!/usr/bin/env sh
# PreToolUse hook on file edits: denies an edit to a `.rs` file until the
# session has loaded the rust skill, and names the skill in the denial so the
# next step is to load it. The transcript is the only record of what the
# session loaded; once it shows the skill, every edit passes. Without jq the
# input cannot be read, so the hook stays out of the way.
command -v jq >/dev/null 2>&1 || exit 0
input=$(cat)
field() { printf '%s' "$input" | jq -r "$1 // empty" 2>/dev/null; }

case "$(field .tool_input.file_path)" in
  *.rs) ;;
  *) exit 0 ;;
esac

transcript=$(field .transcript_path)
if [ -f "$transcript" ] && grep -qF \
  -e '"name":"Skill","input":{"skill":"dev:rust"' \
  -e '"name":"Skill","input":{"skill":"rust"' \
  -e '<command-name>/dev:rust</command-name>' \
  -e '<command-name>/rust</command-name>' "$transcript"; then
  exit 0
fi

printf '%s\n' '{"hookSpecificOutput":{"hookEventName":"PreToolUse","permissionDecision":"deny","permissionDecisionReason":"Rust source: load the dev:rust skill with the Skill tool first, then retry this edit."}}'

# Documentation style

## Describe the present, not the change

**Write durable documentation for a reader who arrives today and has never seen
an earlier version. Describe the system as it is — never relative to a past the
reader never saw.**

Changelog framing — "now", "no longer", "previously", "used to", "instead of",
"we changed", "the old way", "newly", "split out" — references a state that, to
this reader, never existed. It is invisible noise, and it rots as the code moves
on. If a sentence only makes sense to someone who knew the prior version,
rewrite it in plain present tense. The diff belongs in git history and commit
messages, not in docs meant to outlive it.

```
# WRONG — describes the change; meaningless to a first-time reader
`gen-csr.sh` no longer generates the key. It now takes the key as an
argument, so re-running it doesn't re-create the key like it used to.

# CORRECT — describes the current state directly
`gen-csr.sh` builds the CSR from an existing private key passed as an
argument. Rerun it freely to adjust the CSR; the key is only read, never
modified.
```

## Say only what the code cannot

**A comment holds what the code cannot say: why, an invariant no type
enforces, a constraint from outside the code.** Anything else is a second copy
of the code that nothing keeps in sync. The compiler checks the code; nobody
checks the comment.

- **No counts or lists of what the code defines.** "Lint 1 of 4", "the three
  callers", "both workers" and "the two tests that…" turn false the day one is
  added. Name what the members share, or say nothing.
- **No copied values.** "Thirty seconds" beside `const LOOKBACK` is a second
  source of truth. Name the item (in Rust, as an intra-doc link, so a rename
  breaks the doc build), or say why the value is what it is.
- **No restating the name or signature.** Delete `/// Returns the client id.`
  on `fn client_id()`; do not reword it.
- **No narrating callers or other files.** Who calls this, and from where,
  changes without this file changing.
- **Reasoning goes to its home** and the code links it once: a design
  decision to the spec or ADR, a change to the commit message.

When unsure, delete. A missing comment costs a reader a minute; a wrong one
costs a bug.

Both sections apply to: READMEs, `*.md` and `*.org` docs, code comments,
docstrings, and script header blocks. Neither applies to git commit messages,
PR descriptions, or CHANGELOG entries — those exist to describe change and
should.

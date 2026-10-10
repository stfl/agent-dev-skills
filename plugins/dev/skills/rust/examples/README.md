# Skill examples

Every `rust` code block in `SKILL.md` and `references/*.md` appears here, with
elided bodies filled in and nothing else changed. The crate exists so the skill
cannot drift into teaching code that does not compile.

Run after any edit to the skill:

```sh
cargo clippy --all-targets
```

Silence is the pass condition. Anything that fires is either a defect in the
skill or a snippet here that no longer matches the file it mirrors — fix
whichever is wrong, and keep the two in step.

Module names map to their source: `m*` → `modeling.md`, `l*` → `lifecycle.md`,
`b*` → `boundaries.md`, `e*` → `errors.md`, `o*` → `ownership.md`,
`a*` → `async.md`, and the rest are named for what they check.

Agents do not read this directory; the routing table in `SKILL.md` never points
here.

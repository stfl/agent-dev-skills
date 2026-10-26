# Verification: lints, tests, the gate

## Lints as a ratchet

Put them in `Cargo.toml` so every invocation and every CI job agrees. This is the highest-yield thing in this file — do it in the first commit, before there is any code to argue with. In a workspace, define them once under `[workspace.lints]` and put `[lints] workspace = true` in each member.

```toml
[lints.rust]
unsafe_code = "deny"           # deny, not forbid: forbid cannot be overridden anywhere in
                               # the package, tests included, so a counting allocator or an
                               # FFI shim becomes impossible
unreachable_pub = "warn"
missing_debug_implementations = "warn"
# Deliberately no `missing_docs` — see "Documentation" below.

[lints.rustdoc]
broken_intra_doc_links = "warn"   # rot-detection for the links your docs do carry
bare_urls = "warn"

[lints.clippy]
pedantic = { level = "warn", priority = -1 }  # priority, so single lints below override
missing_errors_doc = "allow"                  # what it forces is usually the variant list
missing_panics_doc = "allow"                  # public-item-only; misses the panics you have
doc_markdown = "allow"                        # a hand-maintained proper-noun allowlist
wildcard_enum_match_arm = "warn"              # blind to tuples; `#[expect]` it on foreign enums
unwrap_used = "warn"
expect_used = "warn"                          # the half that catches an invariant nothing checks
indexing_slicing = "warn"                     # see the arena carve-out below
string_slice = "warn"                         # indexing_slicing does NOT cover `&s[a..b]` on a str
allow_attributes_without_reason = "warn"
undocumented_unsafe_blocks = "warn"           # meaningful only with `deny` above, not `forbid`
await_holding_lock = "deny"                   # warn by default; deny is the point, tokio crates only
cognitive_complexity = "warn"                 # restriction; its threshold lives in clippy.toml below
self_named_module_files = "warn"              # restriction; a topic's entry point is x/mod.rs (boundaries.md)
```

Everything in that block is either `restriction` (off by default, enabled one lint at a time) or a deliberate level change on a lint that already warns.

The size and shape lints are the second pass's worklist (`clean-code`, *the second pass*). Their thresholds live in `clippy.toml` beside `Cargo.toml`, and the defaults are loose enough never to fire on the code this skill exists to prevent, so the file is part of setup:

```toml
# clippy.toml — thresholds for the size and shape lints. Every key tunes a lint
# the [lints] block already enables. A hit is a cleanup item for the second
# pass: fix it or ledger it as an issue; #[expect] only with a reason a reviewer
# would accept.
too-many-lines-threshold = 60        # too_many_lines (pedantic); default 100. Past it: extract by decision, not by line.
too-many-arguments-threshold = 5     # too_many_arguments (complexity); default 7. Past it: a parameter struct.
max-struct-bools = 1                 # struct_excessive_bools (pedantic); default 3. Two bools on one thing are an enum.
max-fn-params-bools = 1              # fn_params_excessive_bools (pedantic); default 3. A bool parameter is a flag argument.
cognitive-complexity-threshold = 15  # cognitive_complexity (restriction, enabled above); default 25. Branching over state that wants a match.
excessive-nesting-threshold = 4      # excessive_nesting (complexity); default 0, which never fires. Guard clauses, let-else, or a match.
enum-variant-size-threshold = 200    # large_enum_variant (perf); the default. Box the fat variant, keep the enum.
large-error-threshold = 128          # result_large_err (perf); the default. The error enum sits inside every Result.
```

`unused_self`, already on under `pedantic`, is the pure-function nudge with no threshold: a method that never reads `self` is a free function.

**Setup check**, on entering any crate: `[lints]` in `Cargo.toml` (or `[workspace.lints]` plus `workspace = true` per member), `clippy.toml` present, the gate green. Whatever is missing goes in the first commit, before feature work, and its first run is a worklist, not a failure.

Take clippy's suggestion by default; when you keep the code, `#[expect(lint, reason = "…")]` on the narrowest scope — and only *after* the lint fires, since a pre-emptive `#[expect]` is itself an error under `unfulfilled_lint_expectations`. The suggestions are not type-checked — `needless_pass_by_value` offers `&T` where an owning `'static` task needs the value.

**`pedantic` is not free**, but its cost is not documentation. Under `-D warnings` it pulls in `struct_field_names`, `match_same_arms`, `single_match_else` and `cast_possible_truncation`. `match_same_arms` turns an exhaustive match into a hard error the moment two arms' *values* coincide though their *reasons* differ: merge with an or-pattern that still names every variant, or `#[expect]` it.

## Documentation

**No lint requires a doc comment on any item in an internal or application crate.** The discipline is by hand, and it lives at the module level. Measured on one workspace built three ways: mandating per-item docs produced 571 doc lines of which 158 restated the item name, against 324 lines and 16 restatements when only a module rule applied — same seam usability, a tenth the noise. With no rule at all, module docs stop being written entirely (3 lines across three crates) and recorded invariants drop.

- Every crate root, and every module whose purpose or invariants are not obvious from its name, carries a `//!` header naming what it is for and which invariants it maintains. The crate-root header is the one place a locking discipline, a crash-ordering rule, or a "readers take no lock" contract can be stated, and it is the documentation that earns the most per line.
- Write a `///` only where a signature genuinely under-determines behaviour — a `size` field meaning three things by kind, a handle valid only against particular inputs, a parameter taken by value for a reason — and say the non-obvious thing rather than restating the name. Never document a field whose `#[error]` string or type already renders it.
- Any invariant the spec names — id stability, antisymmetry, a recursion bound, an ordering contract and its tie-break, a version-before-parse rule — belongs in a doc comment on a **public** item, not only in a test name or a `pub(crate)` function. That is where a caller looks, and it is where these gaps have produced data loss.
- A wrong doc is worse than no doc, and rot starts at zero elapsed time: both mandated arms shipped verified-false sentences on day one. When you write a claim, either pin it with a doctest or record it as a choice rather than a fact.
- Refer to a constant, a variant or another item by intra-doc link, never by restating its value or counting its siblings: `broken_intra_doc_links` turns a rename into a failing docs build, and nothing catches "thirty seconds" once `LOOKBACK` changes. The general comment rules, and the workflow for slimming a crate that over-documents, are `doc-style`'s part B.
- If the workspace ever publishes a crate, put `#![warn(missing_docs)]` in **that crate's root** rather than in `[workspace.lints]` — the shape tokio and aya use, because cargo cannot override workspace lints per member.

**`[lints]` is per package, not per target.** `unwrap_used`, `expect_used` and `indexing_slicing` fire inside `#[cfg(test)] mod tests` and in every `tests/*.rs`, and `unwrap_used` also fires on `unwrap_err()`, which is how you assert an error. There is no manifest-level fix. Write one inner attribute at the top of each `mod tests` and each `tests/*.rs`:

```rust
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::indexing_slicing,
         reason = "a failed unwrap or a panicking index is a failing test")]
```

Add `clippy::float_cmp` and the `cast_*` pair in a crate with floats or generated fixtures. Expect a handful more in table-driven and property files and add them as they fire; the ones that recur are `type_complexity` (an ordinary three-tuple case table), `similar_names`, `needless_raw_string_hashes`, `too_many_lines`, `single_match_else`, `format_collect`, `assert_is_empty`, and `wildcard_enum_match_arm` — on *foreign* enums in tests as much as your own.

`main.rs` is neither a test module nor `tests/`, so the setup `unwrap`s that `errors.md` calls fine there still fire. Put `#[expect(clippy::expect_used, reason = "…")]` on those specific lines rather than widening the blanket.

`wildcard_enum_match_arm` cannot be satisfied on a **foreign** `#[non_exhaustive]` enum — `io::ErrorKind`, `sqlx::Error` — where a `_` arm is mandatory and clippy's own suggested fix still ends in one. `#[expect]` it at those matches; it earns its place on your own enums. It is also blind to a match over a *tuple* of enums, where the cross-product is usually too large to write out — a catch-all passes `-D warnings` in silence while a new variant lands in it unnoticed. Take one only when it implements a single uniform policy (default-deny, preserving the whole tuple in the error), and do not claim exhaustiveness in the doc comment: the compiler is not checking what you think it is.

**`indexing_slicing` — but not inside an arena or index-based structure**, where indexing *is* the design and the lint fights every accessor. There, funnel indexing through one `slot`/`get` pair carrying a single `#[expect(clippy::indexing_slicing, reason = "…")]` that states the minting invariant, and leave the lint on everywhere else. It is also wrong for a constant table indexed by `x % LEN`: defensive `get()`-and-fallback replaces a provably-correct line with unreachable error plumbing.

## Tests that check the design

- **Unit tests** in `mod tests` beside the code, for logic with branches.
- **Integration tests** in `tests/`. Write at least one even for a small crate: `tests/*.rs` is a separate crate that sees only `pub` items, so it audits your API surface as a side effect, and a claim about concurrency or atomicity that a unit test reaches by touching a private field has not been tested through anything a user can call. When such a test cannot reach something it obviously should, either the visibility is wrong **or the test was reaching for something it has no business constructing** — decide which before widening `pub`. (`#[non_exhaustive]` on your own types bites there.)
- **Doctests** on public items are documentation the compiler keeps honest.
- **`rstest`** for table-driven cases — parse boundaries, state transitions, error mapping.
- **A crate exposing both a "check it" and a "do it" entry point** — verify/apply, validate/execute, dry-run/run — needs one test asserting they **agree**. A kill table cannot reach this: deleting a check kills the doing test while the checking test stays green because it never ran the check. The structural fix is to put the decision core inside the *reader*, so both verbs get it for free.
- **`proptest`** for round-trips and invariants: `parse(render(x)) == x`, `decode(encode(x)) == x`, "the total never goes negative". These find the cases a table misses.

## Testing a type-level guarantee

A guarantee like "you cannot `send` before `connect`" has no runtime test — the offending program does not exist. Put it in a ` ```compile_fail,E0277 ` doctest, naming the code — but **compile the snippet once and read which code rustc actually emits before pinning one**. A bound that fails through inherent-method resolution is `E0599`, not `E0277`; and when the trait has exactly one applicable impl, inference pins the parameter from that single candidate *before* the bound is checked, giving `E0308` with no `on_unimplemented` text at all (the fix is a second, uninhabited candidate). Verify the pin twice: misspell the type, **and** swap the pinned code for a different one — misspelling alone passes for a snippet failing with the wrong code: same guarantee, no extra dependency, no version-coupled `.stderr` snapshot, and it runs under `cargo test --doc` while doubling as the documentation. A **bare** `compile_fail` passes when the snippet fails for *any* reason — a renamed type, a moved item, a typo in the crate path — so it reports green from the day the API drifts. Pin the code, then verify the test once by misspelling the type in the snippet and confirming the doctest goes red. Reach for `trybuild` only when you must assert on the error *text*. A guarantee you claim in a README and never compile-fail is a claim, not a test.

## State the laws first

Before the kill table, list the algebraic properties your API should obey — symmetry under argument swap, idempotence, an identity element, `f(x, x)`, a round trip in both directions, monotonicity of a "next" function — and write one property test per law. Do this *before* the mutation pass, because a mutation table can only contain claims you already knew you were making. `merge(b, o, t) == merge(b, t, o)` is nine lines and is the difference between a merge tool and a data-loss tool.

An oracle computed from the implementation's own derived values is not an oracle. When a property test bounds a result by a quantity the code produced — a cached interval, a stored window, a `num_buckets()` — the rival that derived that quantity differently passes unchanged. Recompute the bound from the **configured inputs**, and widen the generator to include the cases your own validation rejects: a rate that divides its period evenly tests nothing about rounding direction. A generator narrower than the type is a claim about the generator.

## Break it on purpose

A green suite proves the tests ran, not that they discriminate. For every claim you would make about the design — "the write is atomic", "there are N workers", "the batch ships on age" — delete the code that makes it true and watch a *named* test fail. If none does, the claim is a comment, and comments do not hold.

Make it mechanical, before claiming done: one row per claim, naming the code you deleted and the test that failed. A claim with no row is not one you get to make. `cargo mutants` automates the mechanical part. Expect ten or more rows for anything with a persistence or retry layer, not three.

Deleting code tests that your tests defend what you wrote. It does not test that what you wrote is *right*. For each claim, also write the **rival implementation** — the other reasonable thing a competent engineer would have done at that line — and check that some test distinguishes it from yours. Swapping which value gets returned, which of two orders two reads happen in, which operand the loop consumes: these compile and pass while the behaviour inverts. When no test can tell the two apart, the row documents a choice, not a verified one, and it stays in the table as an open question. A table whose every row makes the code more naive has tested your typing, not your design. Take every counter your run summary prints and set it to a constant — a field that survives the suite is a field with no test.

**Hygiene, in order** — each step has cost a run:

1. Run the gate green, then snapshot `src` to a path **only this process writes** — never a shared `/tmp` path, which has cost two runs their entire `src/`. Snapshotting before the gate makes a mutation left over from an earlier debug run your baseline.
2. `cargo fmt` before the first mutation, so restores diff clean.
3. Run under `--no-fail-fast` — plain `cargo test` stops at the first failing *target*, so a kill gets credited to whichever binary ran first. On lock, condvar or channel code a mutant fails by **hanging**: wrap every run in `timeout`, count exit 124 as a kill, and kill the process group between mutations — `timeout` kills the runner, not a spinning grandchild, and one leaked mutant turns every later 124 into a fake kill.
4. After each restore, `touch` the sources (a restore that preserves mtimes makes cargo run the *mutant* binary against restored sources) and `cmp` each file against the snapshot — `touch` fixes mtimes, it does not prove the restore landed.
5. Read a *dead* mutant too, and confirm the failure names the behaviour in the row rather than something incidental.

**A survivor has six causes, and only the last is a test gap.** Rule out the other five before recording SURVIVED:

- The mutation was a no-op.
- The mutant did not compile — confirm the build succeeded. A mutation that stops a whole test crate from *compiling* is a kill, recorded as a build failure naming the file.
- The killing test aborted the process — a stack overflow, an OOM, a `SIGABRT` — which emits no per-test failure line; key on the runner's summary and exit code.
- The mutation was too weak. Write down the concrete input that would break the *unmutated* code if the claim were false, and confirm the mutant produces it — flipping `>` to `>=` does not test a guard whose failure mode needs `<`.
- The test you believe you wrote is not in the file — a reformat or a failed substitution can silently delete a table row, so re-read the *test*, not only the code.
- The tests really are blind.

Every `unreachable!`, `expect`, `assert!` and `unwrap` you ship in a library path is a claim that some *other* code restricts the input. Deleting the panicking line proves nothing — the mutant just returns garbage. For each one, name the function that makes it unreachable and widen **that** function's admissible set, then *execute* the widened case rather than only planning it. Cheaper than the test: make the function total, so there is no claim to defend.

**Structural claims need structural assertions.** Cheapest and best: add the gauge you would want in production anyway — a busy counter incremented behind a `Drop` guard, with a `fetch_max` high-water mark — and assert the watermark equals the fleet size while the sink is stalled. No test-only plumbing, it discriminates N workers from one, and it survives as a metric.

**Print every variant once.** One test constructs an instance of every error variant and **asserts**, in two halves. `assert!(!err.to_string().contains(&source.to_string()))` catches text doubled between a variant and its `#[source]` — printing it and reading the output does not, because printing cannot fail and no CI run reads `--nocapture`. For the payload half, assert each variant's rendered `Display` contains a rendering of each field it carries; that is the only check for an actionable payload no message ever shows. A variant wrapping a foreign error you cannot construct belongs in an integration test that provokes the real failure instead.

## The gate

One command, same locally and in CI:

```bash
cargo fmt --all --check \
  && cargo clippy --workspace --all-targets -- -D warnings \
  && cargo nextest run --workspace \
  && cargo test --workspace --doc
```

When the crate has features, run clippy **twice** — default and `--no-default-features` — and give an integration test that needs a feature a `[[test]] required-features` entry. A `no_std` claim checked only with `cargo check` on the lib target is a claim about one target out of four.

`--workspace` everywhere is load-bearing: when the workspace root is itself a package, `default-members` is just the root, so a bare `cargo test` compiles none of your members and still exits 0. Keep the `--doc` half too — `nextest` skips doctests, and dropping it silently deletes every `compile_fail` guarantee.

**Worktrees that share one target directory can run a stale binary.** The output's file names do not depend on the worktree's path, and cargo judges a local crate fresh by modification time, so a build in a worktree whose sources are older than the last binary reuses that binary. Give each worktree its own target directory, or `touch` its sources before the build, and check the test count against a number you can derive. Why: a stale binary lists or passes the tests of another commit, and its output does not say so.

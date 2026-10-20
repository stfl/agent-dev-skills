---
name: rust
description: "Design Rust so the compiler enforces the design and hands you the worklist. Load on the first sign the work is Rust, before the first edit: a `Cargo.toml` or `Cargo.lock` in the tree, a `*.rs` file in the diff or about to be written, a `cargo build`, `cargo check`, `cargo test`, `cargo clippy` or `cargo fmt` command, a Rust crate or workspace member by name. Use when writing or refactoring Rust — a crate or module, a type, trait, enum or public API, error handling, tokio tasks and channels, FFI over a C library, state persisted to a file, a background thread's lifecycle, or a crate's lints, tests and mutation gate. Also use for a review of existing Rust, or when a borrow-checker or trait-bound error wants a design fix rather than a `.clone()`."
---

# Rust

Inherits `clean-code`: load it first when it is not in context. This file carries only what Rust adds — the construct that realises each rule, the detectors clippy provides, and the traps the compiler does not catch.

Rust pays for its difficulty in one currency: a design mistake becomes a compile error instead of a 3am page. Spend effort where that conversion actually happens, and write the plain thing everywhere else.

## The loop

1. **Signatures first.** Types, signatures, `todo!()` bodies, bare `cargo check` — not the gate, since `todo!()` bodies emit unused-variable noise by design; grep for `^error`. Where a framework trait or a `serde` attribute is the design, it goes in the skeleton — a handler with a `todo!()` body typechecks only if its return type already implements the response trait. A `-> impl Trait` body needs a concrete stub (`std::iter::empty()`), not `todo!()`, which satisfies no bound. A plain library with neither can skip the pass.
2. **Fill one body at a time**, `cargo check` between.
3. **Green.** The gate in `references/verify.md`, `-D warnings`, everything `--workspace`. Clippy's suggestion is usually right; when you keep the code anyway, `#[expect(lint, reason = "…")]` on the narrowest scope, except in code a macro emits — `expect` warns once the lint stops firing, so a stale suppression reports itself.
4. **Break it on purpose.** Green proves the tests ran, not that they discriminate. Before claiming done, delete the code behind each claim you would make about the design and watch a named test fail. `references/verify.md` has the method and the table it produces.
5. **Leave it cleaner.** `clean-code`'s second pass, in a fresh context, over the diff and one hop out. Rust's worklist is clippy under `verify.md`'s `clippy.toml`: every size or shape hit inside the scope is fixed or ledgered as an issue, in a commit that carries no behaviour change.

## The compiler is a worklist

These moves turn "find everywhere that…" into a list rustc hands you — complete, unlike grep:

- **Change the type, then `cargo check`.** Every affected call site is now an error. That list is the refactor plan.
- **Match your own enums exhaustively, with no `_ =>` arm**, wherever the variant set is small enough that every arm is a real decision. Adding a variant then becomes an error at exactly the sites that must handle it. The shape that slips past review is `match self { Self::X(v) => Some(v), _ => None }` — write the other arms out. When a wide match shares an implementation, push the per-variant part into a closure (`BinOp::Add => arith(a, b, |x, y| x + y)`) rather than collapsing it. A match over a *tuple* of enums is the exception; `references/verify.md` says when a catch-all is allowed there.
- **Add the field or variant before the logic**, to get that list earlier.

## Core and shell

`clean-code`'s decompose-by-decision rule has a Rust-specific force. Decisions are sync functions over values; async is contagious — every `async fn` forces its callers async — so a small async surface leaves most of the program testable with no runtime, no temp dir, and no mock. When a test needs `#[tokio::test]` or a mock to reach a *decision*, that decision leaked into the shell; move it back. The reverse leak is quieter and survives that test: a decision function returning a label the caller must interpret leaves the derivation in the shell, where no pure test reaches it. Hand back what the shell executes — `boundaries.md`, *what a decision hands back*.

## Where to spend

**Default — cheaper than the bug they prevent, no justification needed:**

| Situation | Reach for |
|---|---|
| An ID, unit, or bounded value crossing a function boundary | newtype |
| Two states tracked by a `bool`, or fields only valid together | `enum` |
| Data from outside (JSON, argv, env, DB, socket) | parse into a domain type at the boundary |
| A size, count, depth, or capacity that arrived from outside | bound it before you allocate, recurse, or index on it |
| A counter, id, or capacity loaded from state you persisted | `checked_add`, plus a variant for exhaustion — a value from your own file is still outside input |
| A library's error | `thiserror` — `enum` when callers branch on it, `struct { kind, detail }` when they all do the same thing, plus a classifier method when the branch is in another crate |
| Synchronous, infallible work on the way out | `Drop` guard — async or fallible teardown is `async fn close(self) -> Result<()>` instead |
| Ambient input a test must control — clock, RNG, env, entropy | pass the *value* the behaviour would produce (`now: Date`); a closure only when the caller genuinely varies it |
| A function with a precondition its caller cannot check | a token type only the prover can construct — and the token stays in the signature |
| A permission bit, capability, or privilege word from outside | newtype whose *accessor* returns the neutralised value (`fn permission_bits(self) -> u32 { self.0 & 0o777 }`) |
| A new crate, or an existing one without `[lints]` or `clippy.toml` | `verify.md`'s `[lints]` block and `clippy.toml`, in the first commit — or `[workspace.lints]` once, inherited per member |

**Earn it** — powerful, and costly to read. Take these only when you can name the caller mistake they prevent in one sentence:

typestate · `PhantomData` markers · const generics · sealed traits · GATs · custom `Deref` · declarative macros for API surface · trait hierarchies deeper than one level

Earn a hand-rolled primitive too — a date type, a numeral system, a CSV escaper, a UUID. Writing your own buys a domain-shaped error and one less dependency; it costs the reader arithmetic they must audit. Take it only when the newtype you would wrap around the crate is as long as the thing itself.

When the call is close, a `Vec`, a `match`, and a plain function win.

## Route by situation

Read the file before writing the code, not after the review. A small change inside an existing crate fires one or two rows. Each row names what its file carries — match on that, not on the file name, and skip a row whose subjects you do not have. Two pairs always travel together: `modeling` + `errors` for anything that parses input you did not write, and `lifecycle` + `verify` for anything that writes a file.

| You are | Read |
|---|---|
| modeling values, IDs, or wire/config input | `references/modeling.md` — parse at the boundary, newtypes, enums, bounding untrusted input, recursion |
| writing a parser, decoder, or anything that recurses over input you did not write | as above |
| converting a domain value where the map is partial or many-to-one — wall clock ↔ instant, unicode ↔ bytes, float ↔ decimal | as above |
| designing a **byte format** a caller owns — encode/decode, a version tag, an integrity check | `references/lifecycle.md` § schema version, and `references/modeling.md` — bound declared sizes, canonical encoding |
| **replacing a file on disk** the program later re-reads — save file, vault, lock file | `references/lifecycle.md` — atomic replace, locking, `Drop` guards |
| writing a **proc macro or derive** whose output compiles in someone else's crate | `references/boundaries.md` § Generated code |
| writing a value whose methods must be called in an order | as above |
| running a background `std::thread` that must be signalled and joined on drop | as above |
| designing a trait, a public API, an iterator, or a crate, module or directory layout | `references/boundaries.md` — earn the trait, std traits first, signatures, crate split, a topic's directory |
| writing a decision function whose return the shell must *interpret* — a `ServeMissed` that leaves the shell deriving *which* | `references/boundaries.md` § What a decision hands back |
| building an interpreter, VM, query executor, or a pipeline over a fixed set of node kinds | `references/boundaries.md` § When you do write one |
| choosing error types, propagation, or whether to panic | `references/errors.md` — variants as decisions, classifiers, panic policy, reporting once |
| writing tokio — tasks, channels, shutdown, `select!`, a service that starts and drains | `references/async.md` — actors, channels, cancellation, shutdown, blocking, clocks |
| writing an arena or index-based structure, running work in parallel, or feeling pull toward `clone`, `Arc`, `RefCell`, `unsafe` | `references/ownership.md` — borrow-error moves, arenas, sharing ladder, rayon, FFI |
| wrapping a C library — FFI, a raw handle, `build.rs` + `cc` | `references/ownership.md` § FFI |
| entering a crate — checking `[lints]`, `clippy.toml`, tests, and the gate are in place, or setting them up | `references/verify.md` — lints, `clippy.toml` thresholds, tests, `compile_fail`, mutation, the gate |

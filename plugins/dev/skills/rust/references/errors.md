# Errors

An error type is an API for *deciding what to do next*. Design it from the caller's choices backwards.

## Two layers

- **Library / domain code** returns a `thiserror` enum. The caller can `match`, and adding a variant becomes their compile error.
- **Binaries and the outermost handlers** use `anyhow` (or `color-eyre`) and `.context(...)` — *unless something above the error site branches on which error it was*. An exit code, an HTTP status, or a retry decision is such a branch: keep the `thiserror` enum all the way to `main` and give it a *classifier* — `fn kind(&self) -> ErrorKind`, described below — matched exhaustively with no `_` arm so a new variant forces a deliberate decision. `anyhow` is for programs whose only response to failure is to print it. Nothing below `main`'s neighbourhood returns `anyhow::Result`.

`main() -> anyhow::Result<()>` prints the whole context chain and exits non-zero; `ExitCode` when the code itself carries meaning.

## Variants are decisions, not locations

One variant per thing the caller might do differently. Three variants the caller always handles identically are one variant with a field.

```rust
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum StoreError {
    #[error("no account {0}")]
    NotFound(AccountId),                       // caller 404s
    #[error("account {0} is frozen")]
    Frozen(AccountId),                         // caller 403s
    #[error("storage backend unavailable")]
    Backend(#[from] sqlx::Error),              // caller retries
}
```

- Three variants the caller always handles identically are one variant with a field — or a `struct { kind, detail }`, which keeps the category as a metric dimension without pretending anyone will match on it.
- Carry the identifiers in the variant. An error a human cannot act on is a message that will be re-derived from logs later — but **an error is a value the caller drops**. Never **render** an unbounded caller-supplied payload — keep `Display` bounded to identifiers the type controls, so the message length does not depend on what a socket sent. That is a rule about rendering and size, *not* about ownership: an error that drops the caller's value forces every caller to clone defensively before every call. Own it, render labels only. And a newtype in an `#[error]` string is interpolated with `{}` — `{:?}` on a tuple struct prints the constructor name into the terminal. If the payload has a recursive `Drop`, an expensive `Clone`, or a large footprint, the error is a weapon aimed at whoever handles it. Carry a `Copy` tag naming the shape, a length, an offset, or a pointer to where to look. (`clippy::result_large_err` is the size half of the same concern.)
- Construct carried-identifier errors **lazily**. `x.get(i).ok_or(Err::OutOfBounds { path: self.clone(), .. })` evaluates the struct literal on every iteration, including the successful ones, turning a linear walk quadratic. Use `ok_or_else` / `map_err` / `unwrap_or_else`. `clippy::or_fun_call` does *not* fire on a struct literal, even one containing a method call, so `-D warnings` will not catch it — but only when the payload allocates or clones. A struct literal of `Copy` scalars and `&'static str` costs nothing.
- Each `#[error]` string describes **its own layer only**. The `source` chain prints the rest, and repeating it gives `failed to load config: failed to read file: file not found`. Concretely: never write `{source}` or `{0}` in an `#[error]` string for a field you also mark `#[source]` or `#[from]`.
- `#[from]` when the conversion is unambiguous. When the same foreign error can arrive from four places, `#[source]` plus a `.map_err` that says which one beats a `?` that erases it.
- `#[non_exhaustive]` on the **outer** error a caller propagates, paired with a closed classifier. Leave the **inner taxonomy** exhaustive — the enum whose whole reason to exist is "a distinct variant for every way X can fail" — because that set *is* the promise. Marking the taxonomy `#[non_exhaustive]` without giving it its own classifier makes the distinctions unreachable across the crate boundary, which is the opposite of why you enumerated them. (`tests/*.rs` is a downstream crate: this is also why your own integration tests suddenly want `_` arms.)
- Keep the enum small — it sits inside every `Result` in the crate. Box a fat payload; clippy's `result_large_err` will tell you when.

A classifier is also the only way to get an exhaustive match **across a crate boundary**, since a downstream crate cannot exhaustively match a `#[non_exhaustive]` type at all. Give callers one returning a small enum, never a `bool`: `fn kind(&self) -> ErrorKind` where `ErrorKind` is closed and small — `{ Transient, Permanent }`, `{ User, System }`, whatever the decision actually is. Its body is then an exhaustive `match` a new variant breaks, and the decision site is a `match` the compiler checks. A `-> bool` classifier gets written `matches!(self, Self::X(_))`, which silently defaults every future variant to one answer. When the safe action carries data — bytes to skip, a retry-after, a field to re-request — that payload goes in the enum. Leave *that* enum off `#[non_exhaustive]`: a caller sweeping a new strategy into `_` gets the exact failure the classifier exists to prevent.

The predicate's answer must come from data the error itself validated. An answer derived from a field that failed its own check — a resync offset read from a length the CRC just refused to vouch for — is a confident wrong answer, worse than no predicate. Say in the doc comment which check vouches for it; if none does, return the conservative answer.

**Error as a domain value.** When the failure is something the domain stores, compares, and computes *through* — a spreadsheet `#DIV/0!`, SQL `NULL`, a poisoned cache entry — it belongs in the value type, not the `Result`. `Result` is for failures that leave the computation. The tell: you are writing a propagation rule ("the leftmost error wins") that every operator would re-implement. Put the variant in the value enum and the rule becomes one `match` on a tuple.

## Panic on bugs, `Result` on the world

`Result` is for anything the environment can cause: IO, parsing, a peer, a clock, a full disk. A panic says *this program is wrong* — a violated invariant, an unreachable branch, an index the type system could not bound.

- `unwrap()` in library code is a crash you shipped. In tests and `main`'s setup it is fine and clear.
- `expect("…")` states the **invariant that makes it safe**, not the failure: `expect("regex literal is valid")`, not `expect("failed to compile regex")`.
- `?` on `Option` inside a function returning `Option` is often cleaner than an `unwrap` guarded by an `if let`.
- Integer overflow panics when `overflow-checks` is on (Cargo's dev default) and wraps when off (release default) — a profile setting, so rely on neither: `checked_add` / `saturating_sub` at any boundary where the number came from outside. Same for indexing: `get(i)` returns an `Option` you can propagate.

## Reporting once

Log where you handle, return where you do not. An error that is logged *and* returned appears twice in the log with two different stack positions and no extra information. The top of the request/task/command is the one place that logs, and it logs the whole chain (`{err:?}` on an `anyhow::Error` prints the causes).

Map domain errors to transport codes in the transport layer. `impl From<StoreError> for http::StatusCode` is an orphan-rule error — both types are foreign there. Define a local `ApiError` in the transport crate, `impl From<StoreError> for ApiError`, and let `impl IntoResponse for ApiError` own the status table. Write that table as an exhaustive `match` on the **classifier**, not on the domain enum: the domain enum keeps `#[non_exhaustive]`, the one exhaustive match lives in the defining crate as `fn kind(&self) -> ErrorKind`, and `ErrorKind` is closed — so the transport's table is exhaustive too, and a new domain variant is a compile error in exactly one place. The domain crate does not know it is being served over HTTP.

One carve-out: when a variant's payload **is** the transport value — an upstream status you relay verbatim, a `Retry-After` a peer supplied — that variant is not a category and no classifier can hold it. Then the domain enum's own exhaustive `match` is the table, and the thing to protect is that the match has no `_` arm. Say so at the match site, because it looks like the mistake this section warns about.

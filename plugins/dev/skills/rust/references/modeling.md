# Modeling values

The goal is a type whose every *value* is meaningful. Then `match` is total, tests shrink, and the reviewer stops asking "can this be empty here?".

## Parse at the boundary

Untrusted data — JSON, argv, env, a DB row, a socket frame — is checked exactly once, at the edge, and converted to a type that carries the proof. Inside that line, no function re-checks anything.

Scanning a `&str` by byte index is fine for finding ASCII delimiters. The moment you copy scanned bytes into a `String`, `b as char` and `char::from(b)` are a Latin-1 decode that silently mojibakes every multi-byte character — no error, just a literal that never matches. Push `char`s from `char_indices()`, or slice `&src[start..end]`, which panics loudly on a bad boundary instead of corrupting quietly. Test one non-ASCII literal end to end through the public API.

The proof only holds if the field is private and the module is small enough to audit. `pub struct Email(pub String)` proves nothing.

```rust
pub struct Email(String);            // private field: this module is the invariant boundary

impl Email {
    pub fn parse(raw: &str) -> Result<Self, InvalidEmail> { /* ... */ }
    pub fn as_str(&self) -> &str { &self.0 }
}
```

A declared size is not a promise. Bound a length prefix, batch size, nesting depth, or repeat count against a constant *before* you allocate from it, recurse on it, or index with it — a `Result` cannot catch a stack overflow or an OOM kill. A constant chosen for legitimate data is only half the bound: also reject `declared × MIN_BYTES_PER_ITEM > bytes_remaining` before `with_capacity`, or a 1,000,000-item cap over a 64-byte record still lets a five-byte header reserve 64 MB. Two corollaries: in a zero-copy decoder the buffer that grows is the caller's, so the bound must precede the "need more bytes" return, not follow it; and a depth counter on the **parser** does not bound the depth of the **tree** — a precedence-climbing loop consumes `a + b + c + …` iteratively while adding one level per operator, which every later traversal, including the derived `Drop`, then recurses over. Bounding input length is only a proxy — it caps depth at (bytes / shortest operator), still thousands of frames on whatever stack the later walk happens to run on. Flatten an associative operator — `AND`, `OR`, `+`, concat — into `All(Vec<_>)`, so the precedence loop pushes instead of nesting and a chain of any length has height 1. Keep `Box` plus a depth counter for genuine nesting only, and reach for a handle or arena only when nodes are shared, content-addressed, or serialized. When the recursive type is **foreign** — `serde_json::Value`, a protobuf message — route every internal copy, comparison, overwrite and discard through an explicit worklist, and remember that an assignment is a drop.

When the recursion is a divide-and-conquer whose depth is provably `O(log n)`, none of this applies — assert the bound and move on.

The commonest instance is a line reader: `read_until(b'\n', &mut buf)` allocates the entire remaining input when the input contains no newline. Bound it by binding the `Take` to a local first — chaining `read_until` onto `take` keeps the temporary alive and will not compile — and note that both `by_ref` and `take` are `Read` methods, so `std::io::Read` must be in scope. For record-structured input, an over-long line is an error naming the record, not something to resynchronise past: silently resuming mid-record produces plausible wrong data.

When a validated value exists in order to **match** untrusted input — a path prefix, a domain suffix, a mount point, a CIDR — put the match on the type, beside the constructor, and be suspicious of the obvious string operation. `/api` covers `/api/v1` and does **not** cover `/apiary`; `starts_with` gets that wrong, compiles, and passes every test whose paths do not straddle the boundary. Make the adjacent-but-not-covered case a named test. If the same matching logic appears at a second call site, one of the two is already wrong.

Implement `FromStr` rather than a bare `parse` when the type comes from text: `.parse()` works, and `clap` and `serde` pick it up.

Keep the failure informative — `InvalidEmail { input: String, reason: &'static str }` beats a unit struct, because the error is what the user reads. Derive `thiserror::Error` on it: `serde`'s `try_from` requires the error to implement `Display`.

## Newtype

Wrap any ID, unit, or bounded value that crosses a function boundary. `fn transfer(from: AccountId, to: AccountId, amount: Cents)` cannot be called with the amount in an id position; the `u64, u64, u64` version can, and will be.

A newtype separates values of different *kinds*. It does nothing for two arguments of the same kind — `transfer(to, from, amount)` still compiles. When swapping two same-typed arguments is the bug you fear, the tools are a role wrapper per position (`Payer(AccountId)`) or one struct argument with named fields; both cost every call site, so name the mistake before paying.

Apply that test to *every* newtype. A value only ever read back out to be logged or displayed has not crossed a boundary in the sense that matters — no call site can pass it wrongly, because no call site passes it. Leave it primitive.

- Derive `Debug, Clone, PartialEq, Eq, Hash` by default; add `Copy` when the inner type is `Copy` and small, `PartialOrd, Ord` when ordering means something.
- Expose `as_str` / `as_bytes` / `into_inner`, or implement `AsRef`, in preference to `Deref` — the API guidelines reserve `Deref` for smart pointers, and on a plain wrapper it leaks the entire inner API into method resolution. Never `DerefMut`: it hands out mutable access to the very thing the invariant protects.
- `#[serde(transparent)]` when the wire format is the bare inner value **and** the type has no constructor to route through. When it has one, `try_from`/`from` replaces `transparent` — serde rejects both on one type.
- Conversions *into* the newtype are fallible when they **validate**: `TryFrom<String>`, never `From<String>`, since an infallible `From` hands the invariant back to every caller. They are infallible when they **normalise**: `From<Vec<Tag>> for TagSet` that sorts and de-duplicates *establishes* the invariant rather than checking it, and the serde spelling is `#[serde(from = "Vec<Tag>", into = "Vec<Tag>")]`. Pick by asking whether there is an input the constructor must reject.

Distinct units get distinct types. `Duration`, `Cents`, `Meters`, `Utc` timestamps — mixing them up is the classic production bug, and the type system deletes the entire class.

## Partial and non-injective conversions

When a conversion into your domain is **partial** (some inputs have no image) or **non-injective** (some collapse), three decisions are yours: the resolution for the hole, the canonical representative for the collision, and — the one people miss — **verification in the target**. Do not predict ambiguity in the source; run your own output back through the decoder and compare. A per-segment check ("no segment contains the separator") misses a segment that *forms* one at the join: `a_` + `__` + `b` splits back as `a`, `_b`. A per-character check misses a case mapping that changes length: `ß` uppercases to `SS`, and `to_lowercase`/`to_uppercase` are not involutions. An `encode(&K) -> Option<String>` whose last line is `if decode(&out)? != *k { return None }` costs one round trip per key at construction, and the set you accept becomes the set that round-trips. Ordering is the same problem: enumerating the source in order does not enumerate the target in order, so a `next_after(x)` re-checks `result > x` in the *target*. Make the non-ASCII test use a non-ASCII **key**, not only a value — a value is carried, a key is transformed.

## Enum over flags

Whenever two `bool`s, or an `Option` beside the field that explains it, describe one thing, that thing is an enum. Three `bool`s admit eight states and usually mean four.

```rust
struct Job { pending: bool, started_at: Option<Instant>, output: Option<Vec<u8>>, error: Option<String> }
// ^ sixteen combinations, three of them legal.  Below: three variants, each carrying what it has.
enum Job { Pending, Running { started_at: Instant }, Finished { output: Vec<u8> }, Failed { error: JobError } }
```

Give variants struct fields with names once there are two of them. Reserve tuple variants for one obvious payload.

When a variant's payload has its own invariants, that payload is its own struct — the enum stays a list of situations, not a bag of fields.

## Serde without a hole

`#[derive(Deserialize)]` on a type with invariants is a back door: it builds the value field by field, skipping your constructor. Route deserialization through the parser.

```rust
#[derive(serde::Deserialize)]
#[serde(try_from = "String")]
pub struct Email(String);

impl TryFrom<String> for Email {
    type Error = InvalidEmail;
    fn try_from(v: String) -> Result<Self, Self::Error> { Email::parse(&v) }
}
```

A flat mirror struct is invariant-free by construction, so **every field that was a validated domain type on the way out must be re-parsed on the way in** — including cached extrema, running min/max, counters, and any statistic you compute rather than accept. Before writing `TryFrom<Wire>`, list the domain types the mirror flattened to primitives and check each has a constructor call on the return path.

For config structs, `#[serde(deny_unknown_fields)]` turns a typo in a YAML key from a silently-ignored default into a startup error.

## A precondition as a type

When a function's precondition spans several values — "these cells are independent", "this index came from this arena", "this batch is already sorted" — no single newtype states it. Mint a token whose constructor lives in the module that *proved* the property, keep its field private, and take it as a parameter. The proof then travels in the signature. It only works if the token stays there: destructuring it one line before the call puts the proof back in a doc comment.

## Public types that will grow

- `#[non_exhaustive]` on a `pub` enum or struct that may gain variants or fields: it forces downstream `_ =>` arms and blocks struct literals, so adding one later is not a breaking change. It is transparent inside the defining crate — your own matches still hand you the worklist — but `tests/*.rs` is a *downstream* crate, so your own integration tests will need `_` arms on it. Skip it when the exhaustive set **is** the API and a downstream `_` would be a bug; that exception is common, not rare.
- `#[must_use]` on any type whose only purpose is to be consumed: a guard, a builder, a "did anything change" report. `Result` already carries it.
- Constructors are `TryFrom` / `FromStr` / named `parse`; a `new` that can fail and returns `Option` throws away the reason. Before writing the error type, though, name the caller that reads its fields: when every caller maps failure to one fixed value — a `#REF!`, a `None`, a default — `Option` plus a doc comment naming the reason is the honest constructor, and the `thiserror` type is ceremony whose only consumer is your own unit tests.

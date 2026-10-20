# Boundaries: traits, APIs, modules

## A trait needs two implementors

Write the concrete type first. Introduce a trait when a second real implementation exists, when a third party must plug in, or when it names a contract at a crate seam a second implementation will certainly meet. One trait with one impl is indirection that hides the code from the reader and from `go to definition`, and it multiplies the surface the compiler has to check on your behalf.

"But I need it for tests" is the usual reason, and it is usually the wrong fix. In order of preference:

1. **Move the decision out of the IO** — a sync function over owned data needs no seam at all (see `core and shell` in `SKILL.md`).
2. **Pass the value**, not a source for it: a function needing the time takes `now: DateTime<Utc>`; one needing randomness takes the seed. A `Clock` trait with one implementor is the anti-pattern this rule exists to prevent.
3. **Pass the behaviour**, not an object: `fn retry(op: impl FnMut() -> Result<T, E>)`.
4. **Pass the data**: `impl Read` / `impl Write` / `&mut impl BufRead` covers most "fake the file/socket" cases, and `std` already implements them for `&[u8]` and `Vec<u8>`.
5. **Hit the real thing** — an in-process SQLite, a `wiremock` server, a `tempfile` — which tests the SQL and the wire format too.
6. **A planned second implementation counts.** When the interface shape *is* the artefact being designed, write the trait now. Two governors: name the methods in domain verbs (`claim`, `follow`), not CRUD, so the second implementor is not forced to re-derive your rules; and give it at least one test double, or it is untested indirection.

Before writing a trait method, list the rules its implementors must uphold and check that each rule's inputs appear in the signature. A rule whose inputs are missing — `insert(&self, key, value)` for a rule about expiry, with no `now` — is a rule only your first implementor happens to satisfy.

## What a decision hands back

A module that decides hands its caller either a **classification** to act on or a **plan** to execute. Both are values crossing the same seam; the question is which leaves nothing for the caller to work out.

The test is not how much work follows. A classification whose arms send the caller off to hash a multi-gigabyte file, or to loop probing for an unused filename, is sound. A classification whose arm costs one subtraction leaks, when that subtraction is domain arithmetic.

> A classification is the right return when, given the decider's inputs, **nothing further is computable** — every remaining step is a fresh observation of the world or an effect on it. It is the wrong return when the caller still evaluates an expression over values, or pure functions, the decider had or could have been handed.

```rust
// Sound: nothing more follows from (name, known). Adopting the file needs its
// mtime, quarantining it needs a free name only a probe can find.
fn classify(name: &str, known: Option<Entry>) -> Disposition;
enum Disposition { Keep, Adopt, Quarantine, Sweep }

// Leaky: `ServeMissed` obliges the caller to derive *which* periods, out of a
// cursor and a schedule the decider already holds.
enum Recovery { CloseNow, ServeMissed, Continue }

// Sound: the effect stream, arguments supplied.
fn plan(cursor: Cursor, now: Instant, next_tick: impl Fn(Instant) -> Option<Instant>) -> Plan;
enum Plan { Reopen { at: Instant }, Emit { periods: Vec<Period> } }
```

The usual cause of a leaky classification is **a pure function mistaken for a capability**. A `Schedule` with a `next_after(instant)` method sits in a field and is named like a service, so it reads as IO — but it is a function of (schedule, instant) with no world behind it, and everything derived from it strands on the effectful side of the seam. Rust hides this: a value, a pure function and a capability all arrive as fields of the same struct. Audit them by hand — for each field the decider's branches would touch, mark it **value**, **pure function**, or **capability**. The first two become parameters; only capabilities stay behind. Take the pure one as `impl Fn(..) -> ..` rather than borrowing its owner, and the decider tests with a closure.

Push down what the decider can compute *within the question it answers*, and stop at the executor's vocabulary. A plan naming periods to emit does not also allocate the sequence numbers stamped on them, when monotonic numbering is an invariant of the emitting module rather than an answer to the recovery question. Without that bound the rule keeps pulling, and the executor's invariants land in the decider.

Three moves that fire before the bad version exists: write the shell's verbs first with their real parameter lists (`emit(period)`, `quarantine(path)`, `reopen(at)`) — the plan is those calls with the arguments filled in; run the field audit above; write the pure test first with its expected value stated as what reaches the wire or the disk, since a return type that cannot express the expectation is too shallow and you learn it before a caller exists. Two smells while typing the return type: a variant payload nobody matches on, feeding only a log line, is a **message, not a discriminant** — a `&'static str` field says so; and a variant named as an imperative with its object missing (`ServeMissed` invites "serve *which*?") is a verb whose arguments belong in the type.

Shape the return like the effect stream: a sequence of outputs wants an iteration and one action per input wants a discriminant per input — a case statement facing a sequence grows a loop somewhere it does not belong. Ousterhout's *small interface* slogan misleads here: the leaky enum is the smaller type carrying the larger interface, its obligations living in no type at all.

A reshaped return type that **deletes** cases is evidence the old shape carried a distinction the domain does not have — `Emit { periods: vec![] }` and "leave it alone" are one case, not two. Qualify that before taking it as a win: cases the *effects* never distinguished were noise, but cases the *journal* distinguished are a trade. Two situations performing identical steps can still deserve different log lines, and collapsing them silently costs an operator the sentence that would have explained the morning's behaviour.

### Where a classification wins

A plan is a claim about a world that holds still. Prefer a classification when it does not:

- **A step's arguments depend on the world after the previous step.** Two files colliding in a quarantine directory take `.1` and `.2` suffixes; a precomputed plan would have to model the namespace it is about to mutate.
- **Observations and decisions interleave.** The plan shape assumes every observation precedes every decision; when the next probe depends on the last answer, return a verdict per round.
- **The seam is a channel.** A plan is only as fresh as the snapshot behind it, and the state moves after you compute one — send the *question* across the mailbox and derive the plan inside the task that owns the state (`async.md`).
- **Most combinations of the effects are invalid.** A struct of `Option` fields admits states its own doc forbids; the sum type makes them unrepresentable (`modeling.md`, *enum over flags*).

Materialising a plan is safe when it is a projection of durable state that survives a partial run: execute a prefix, crash, and the next start re-derives the remainder from the same rule.

## Prefer a std trait to a new one

Before defining `trait ToBytes`, check whether `From`, `TryFrom`, `FromStr`, `Display`, `AsRef`, `IntoIterator`, `Iterator`, `Default`, `Extend`, or `io::Read/Write` already means it. Implementing a std trait wires your type into every generic function in the ecosystem for free; a bespoke trait wires it into yours.


## When you do write one

| Situation | Shape |
|---|---|
| One call site, one type per instantiation | generic `impl Trait` argument — the default |
| Heterogeneous set stored together, or a plugin loaded at runtime | `dyn Trait` behind `Box`/`Arc`/`&` |
| Closed, known set of implementations | `enum` + `match` — no vtable, still matchable, and the compiler lists new variants for you |
| Closed set differing in the **state stored per element**, held in a large collection | generic parameter with an associated `State` type — an `enum` costs the union plus a discriminant on *every* element |

When both rows above fire, ask **where the polymorphic value lives**: an `enum` is right on the stack for one call; a generic parameter is right as the per-element payload of a collection, or when no call site is ever generic over the set.

`dyn` needs *dyn compatibility* (rustc's word since 1.83): no generic methods, no `async fn` or `-> impl Trait`, no `Self` outside the receiver, no associated consts. One implementation per type → associated type (`Iterator::Item`); several per type → type parameter (`From<T>`); getting it backwards forces callers to annotate types they should never name. To publish a trait others may *use* but not *implement*, give it a private supertrait (`mod sealed { pub trait Sealed {} }`) — the only mechanism on stable. To add methods to a foreign type, an extension trait with a blanket impl is the orphan-rule escape; keep blanket impls at the leaves, since one `impl<T: Foo> Bar for T` forbids every future hand-written `Bar`.

## Generated code

Code a macro emits compiles under **someone else's** lints, not yours. It cannot know what they deny, so it must be clean under the strict end of what people configure: fully-qualified paths (`::core::result::Result`, never a bare `Result` the caller may have shadowed), no reliance on anything in scope at the call site, and `#[automatically_derived]` on the impls so `missing_docs` and the doc lints do not fire on items the user did not write. `#[expect]` is the wrong tool inside generated code — an expectation that goes unfulfilled in *one* caller's configuration is a hard error there; use `#[allow]` with a reason.

Diagnostics are the deliverable as much as the codegen. A misspelled variant, a duplicate rule, or a malformed attribute must produce an error pointed at the offending span — `syn::Error::new_spanned` — not a panic, and never an error that surfaces from inside the expansion where the user cannot read it.

## Signatures callers can actually satisfy

Take the loosest thing you can use; hand back the most specific thing you have.

- Arguments: `impl AsRef<Path>` for paths, `impl Into<String>` when you will own it, `impl IntoIterator<Item = T>` over `Vec<T>` — unless the function traverses more than once, in which case it must `collect()` internally and you have moved the allocation rather than removed it; take `&[T]` there. (Clippy already catches `&String` and `&Vec<T>`.)
- Returns: a concrete type, or `impl Iterator<Item = T>` to skip the intermediate `Vec`. `Box<dyn Iterator>` only when the type genuinely varies. A named iterator type is worth it when callers must store or name it; `impl Iterator` when they only consume it.
- Laziness is about work, not validation. Validate structure once at the boundary and let the lazy iterator be infallible; `Iterator<Item = Result<T, E>>` over data your own constructor already accepted is a re-check pushed onto every caller — and callers write `filter_map(Result::ok)` and lose the error.
- An iterator that promises laziness must be lazy. A constructor that walks the whole graph makes "stops at the first hit" false, and the doc comment is then advertising an O(1) early exit over a Θ(V) call.
- `#[derive(Clone)]` on a type that only *borrows* — `struct Iter<'a, T> { s: &'a [T] }` — generates `where T: Clone`, which is wrong: the field is `Copy` for every `T`. Hand-write the impl. Same trap for `Debug` and `Default`. This is the commonest way a library accidentally constrains its user's payload type.
- Take `self` when the method consumes, `&mut self` when it mutates, `&self` otherwise — and let that choice carry the meaning rather than a doc comment.
- Push the `if` up and the `for` down: a function taking `Option<T>` only to return early wants to take `T` and let the caller branch. A precondition handled at the call site is a precondition the callee's type can state.
- Lifetimes in a public struct force every caller to thread them. Owning the data, or `Cow<'_, str>`, is usually the kinder API.

## Modules and crates

Visibility is the enforcement mechanism for every invariant in `modeling.md`, so it is a design decision, not bookkeeping: default to `pub(crate)` and let `pub` mean "I will keep this working". A module is the unit that owns an invariant — big enough to enforce it, small enough to read in one sitting.

A directory is how a topic (`clean-code`, *The tree is the table of contents*) reaches the file system, and the topic's parent module is a visibility boundary: privacy reaches a module's descendants, so an item private to `billing` is visible in `billing::invoice` and nowhere else. That parent is the topic's one entry point — what its files share, the trait or the vocabulary types, and the `//!` header that says what the topic is. Inside it, narrow below the crate default: a child only the topic uses is `mod invoice;`, an item only the parent needs is `pub(super)`, and `pub(crate)` is kept for what the rest of the crate calls. Visibility does not choose the group; it checks it. An entry point that keeps nothing private marks a sorted folder, and a member another topic must reach into is cut on the wrong side. Re-export (`pub use`) only at a library's public surface: inside a crate, the path says where the code lives.

Split a workspace crate when there is a reason the compiler can see: a genuinely reusable library, a heavy dependency you want off the critical path, or a build-time win from parallelism. Splitting for tidiness alone buys `pub` on everything that used to be `pub(crate)`.

The layout that pays for itself: one crate holding domain types and pure logic, whose `Cargo.toml` mentions neither `tokio` nor an HTTP client, and thin crates around it that do. If the domain crate ever needs `tokio`, something moved to the wrong side. One exception: a pure core producing *deadlines* for a tokio timer uses `tokio::time::Instant`, or a `start_paused` test compares a virtual deadline against a real `std::time::Instant::now()` and passes for the wrong reason. The rule is about IO, not about which clock the domain measures against.

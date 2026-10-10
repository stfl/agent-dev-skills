# Ownership: when the borrow checker pushes back

A borrow error is a design review. Work the list before reaching for an escape hatch — `.clone()` is move 6 and `Arc<Mutex<_>>` is move 7.

1. **Shorten the borrow.** Compute into a local *before* the mutable borrow starts. `let n = map.len(); map.insert(k, n);` compiles where the one-liner does not.
2. **Split the borrow.** Destructure — `let Self { input, output, .. } = self;` — and rustc tracks the fields independently. `split_at_mut` does the same for slices.
3. **Move instead of borrow.** `Option::take`, `mem::replace`, `mem::take`, or take `self` by value and return the new value.
4. **Replace the reference with an index.** See below.
5. **Copy the small thing out.** Ids, lengths, and enum tags are `Copy`; you rarely needed the reference.

Then, deliberately: clone (say why in a comment when it is in a loop), or share.

## Indices instead of pointers

Any graph, tree with parent links, or object soup that wants `Rc<RefCell<Node>>` should be a `Vec` plus newtype indices. This is how rustc and rust-analyzer are built.

```rust
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct NodeId { arena: ArenaId, index: u32 }        // both fields private

impl Graph {
    #[track_caller]
    #[expect(clippy::indexing_slicing, reason = "index minted by this arena, checked above")]
    fn slot(&self, id: NodeId) -> &Node {
        assert_eq!(id.arena, self.id, "handle from a different arena");
        &self.nodes[id.index as usize]
    }
}
```

You get `Copy` handles, cycles, serialization, `Debug`, and no runtime borrow panics.

The cost is that an id from one arena means nothing in another, and **a bare newtype does not catch that** — `NodeId(3)` indexes any arena happily and returns a real, wrong node. With one arena live, ignore it. With several, tag the handle as above and check it in one place; `Clone` must then mint a *fresh* `ArenaId`, or a copy silently accepts the original's handles once the two diverge. Funnelling every index through one `slot`/`get` pair is also what makes `clippy::indexing_slicing` bearable in the rest of the crate.

## The sharing ladder

Climb only as far as the problem forces:

| | |
|---|---|
| One owner, `&mut` passed down | the default; no sharing at all |
| Several readers, no writers | `&T` — including across threads under scoped or `rayon` parallelism, where `Arc` is pure overhead |
| Several owners, one writer at a time, one thread | `Cell` (whole-value `set`/`replace`; only `get` needs `Copy`) then `RefCell` |
| Several tasks or threads that each need the value | **send it down a channel** — move ownership rather than share it |
| Counters or flags shared by many tasks | `Arc<struct-of-atomics>` — no lock to hold across an await |
| Genuinely shared mutable state | `Arc<Mutex<T>>` for a small, cheap critical section |
| Read-mostly shared state | `Arc<RwLock<T>>`, or swap a whole new `Arc<T>` in via `arc_swap` |

Hold a lock across as few lines as possible, and never across an `.await`. A `std::sync::MutexGuard` held over one blocks a whole worker thread; it is `!Send`, so `tokio::spawn` refuses it, but a current-thread runtime compiles it happily. `clippy::await_holding_lock` is what actually catches this. Clone out, or compute in a block that ends before the await.

Write `Arc::clone(&x)` rather than `x.clone()` for refcount bumps, so the reader can tell a pointer copy from a deep copy at a glance.

## Owning vs borrowing in your own structs

A lifetime parameter on a struct is a promise the caller must keep: *this cannot outlive that buffer*. Make it when zero-copy is the point (a parser over an input slice, a view type). Otherwise own the data — `String`, `Vec<T>`, `Arc<str>` — and spend the allocation.

`Cow<'_, str>` covers the middle case: return borrowed when nothing changed, owned when it did, and let callers stop caring.

A struct that wants to hold both a buffer and a reference into that buffer is not expressible. Store offsets, or an index, and resolve on access.

## Data parallelism

`rayon`'s `par_iter()` borrows `&T` across threads under a scope that outlives them — no `Arc`, no `Mutex`. Call it from `spawn_blocking` inside an async program, prefer computing a fresh generation from a read-only snapshot over mutating shared state under a lock, and assert the *schedule* rather than only the output — `verify.md`'s busy-counter watermark is the cheapest way — **unless the operation is an associative-commutative reduce, whose whole contract is schedule-independence**. There the output *is* the schedule assertion: `assert_eq!(parallel, serial)` proves bit-identity under any merge tree, and a watermark would be testing rayon. Note `fold` yields `T` while `try_reduce` needs `Try`, so the working spelling is `.fold(id, op).map(Ok).try_reduce(id, op)` — the bare `.map(Ok)` reads like a mistake and is not.

## Clone budget

Cloning at setup, at a boundary, or once per request is free in every sense that matters. Cloning inside a loop over a large collection is the thing profiles find. Optimise the second, ignore the first, and never contort a signature to avoid the first.

## FFI

`unsafe` is for FFI and for a hot path you have benchmarked. Confine it to one module, wrap it in a safe API, and put a `// SAFETY:` comment above every block naming the invariant that makes it sound. Reaching for `unsafe` to escape a borrow error means moves 1–5 were skipped. Build the C side from `build.rs` with the `cc` crate so the crate is self-contained, and declare the surface by hand rather than generating it when it is a dozen functions. Edition 2024 requires `unsafe extern "C" { … }`, and `verify.md`'s `unsafe_code = "deny"` is overridden by a module-level `#![expect(unsafe_code, reason = "…")]` — that is what `deny` rather than `forbid` buys you.

- Own the handle as `NonNull<T>`, never `*mut T`: it is niche-optimised and already denies both `Send` and `Sync`, so the only thing left to write is `unsafe impl Send`.
- `Send` and `Sync` are two questions. `Send`: does any call depend on the calling thread — a thread-local, an errno slot, an owning-thread OS handle? `Sync`: can two `&self` calls run at once — audit every `&self` method, including a C accessor that lazily renders through a `const`-cast, which mutates. Prove a denial with a `compile_fail,E0277` doctest asserting `fn assert_sync<T: Sync>()` fails to instantiate.
- Copy a borrowed `const char*` into an owned `String`/`Box<str>` **inside the same `unsafe` block**. Never hand a caller a `&str` borrowed from the callee.
- Out-parameters: `Vec::reserve`, `spare_capacity_mut`, then `assert!(written <= cap)` before `set_len`. An OK status is the callee's word, not a licence to publish uninitialised bytes.
- Two jobs are two types, not one with a mode field: `Encoder`/`Decoder` with `finish(self)` deletes both "wrong direction" and "already finished" error variants outright.

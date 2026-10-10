# Lifecycle: construction, transitions, teardown, persistence

Four tools for "this happens in an order". Pick by *who knows the order*.

| The sequence is | Use |
|---|---|
| fixed and known while writing the code | typestate — states are types |
| decided at runtime by data, IO, or a peer | `enum` state machine — states are values |
| fixed for the half you drive, decided at runtime for the half a peer drives | typestate on the send half, `enum` machine on the receive half — say which is which in the module doc |
| "assemble, then use" | builder |
| "must happen on the way out" | `Drop` guard |

## Typestate

Earns its place when a caller can plausibly call the methods out of order and the consequence is real: committing a transaction twice, reading a stream after closing it, reusing a consumed buffer. Each transition takes `self` by value, so the old state is gone — `fn close(self) -> Conn<Closed>` with a `PhantomData<S>` on the struct.

The bill: the state parameter spreads into every signature that touches the value, the states cannot sit in one `Vec`, and callers cannot hold "a connection in some state" without an enum wrapper anyway. Pay it for a library boundary others depend on. Inside a crate, a runtime `enum` and a returned error usually buy the same safety for a tenth of the reading cost.

Never reach for typestate for state that arrives from outside the program — a protocol peer decides its own state, so yours must be a value you can `match`. When only one direction is yours to drive, split: typestate on what you send, an `enum` on what you receive.

## Enum state machines

If the *phase* is `Copy`, match on it directly — `match (self.phase, event)` — and skip everything below. Derive `Copy`/`Clone` on the phase only, **never on the machine that owns it**: a duplicated machine is a second authority on a state only one thing may hold, and nothing warns when one is made. This is the enum branch's version of "each transition takes `self` by value, so the old state is gone".

When the state owns non-`Copy` data, transition by *replacing* it rather than mutating fields around it. Taking ownership out of `&mut self` needs `mem::replace`:

```rust
fn advance(&mut self, ev: Event) {
    self.state = match (std::mem::replace(&mut self.state, State::Poisoned), ev) {
        (State::Idle, Event::Start) => State::Running { since: Instant::now() },
        (State::Running { since }, Event::Done) => State::Finished { took: since.elapsed() },
        (other, _) => other,
    };
}
```

`mem::replace` is what makes this compile: it hands you the owned state and leaves a stand-in behind. Keep the stand-in unreachable in the happy path, or use `Option` + `take()` for an explicit panic site. Match on `(state, event)` pairs so the compiler lists every combination you have not decided about.

## Builder

Reach for one past three or four constructor arguments, or when most are optional. Inside a crate, `Config { timeout, ..Default::default() }` is a builder with no code — try it first; past that, generate it (`bon`, `derive_builder`) before hand-rolling. A builder is `#[must_use]`; a built-and-dropped builder is always a bug.

## Drop guards

Anything that must happen on the way out — unlock, roll back, flush, restore the terminal, stop the timer, decrement the gauge — belongs to a value whose `Drop` does it. Then early returns, `?`, and panics all clean up, and no reviewer has to check the paths.

```rust
#[must_use = "the span ends when this guard is dropped"]
pub struct Timer { name: &'static str, start: Instant }

impl Drop for Timer {
    fn drop(&mut self) { metrics::histogram!(self.name).record(self.start.elapsed()); }
}
```

Bind it to a named variable: `let _guard = Timer::start("db.query");`. `let _ = Timer::start("db.query");` drops it immediately and silently does nothing. Better than remembering: **hang the guarded operations off the guard** — `guard.put(x)`, not `store.put(x)` — so a dropped guard has no methods left and the mistake stops being expressible.

A worker that sleeps needs waking before it can be joined: a `Condvar` the `Drop` notifies, or `thread::park` with an `unpark`. Signal first, then join — a `Drop` that joins a parked thread hangs. And `std::thread::JoinHandle` is not `#[must_use]` either. Dropping one silently **detaches** the thread — no warning, no error, no lint — so a `Drop` that signals a worker and drops its handle has not joined it. Test that by checking `Arc::strong_count` on state the closure captured, immediately after `drop`, with no sleep: a thread-count snapshot or an "it went away eventually" trend passes under the bug.

## Replacing state on disk

Temp file **in the same directory**, `sync_all` it, rename over the target, `sync_all` the directory. `tempfile::NamedTempFile` is the spelling, and it is both patterns above at once: `Drop` removes the temp file on every early return, and `persist(self)` consumes, so double-commit is unrepresentable.

**An atomic replace is not a lock.** `load → mutate → save` across two invocations still loses updates: both read the old state, both write, the second wins silently and neither is told. Four things, each with its own failure:

- **Lock the whole read-modify-write** with `flock` or `O_EXCL`, or concurrent writers silently drop each other's work.
- **Only mutating paths take the lock.** A read path that takes an exclusive lock makes a file on a read-only mount, or another user's file, *unreadable*, and two concurrent readers evict each other. Take a shared lock or none.
- **Canonicalize the target and copy its mode onto the temp file** — *when the target is a document that already exists*. Otherwise the rename replaces a symlink with a regular file, and a file the user chmodded is reset on every save.
- **When the path is written once and never updated** — content-addressed objects, an append-only log segment — invert both: `persist_noclobber`, not `persist`, so an existing file is never replaced; do not canonicalize and do not copy a mode, because the target must not exist. The schema version then goes *inside* the hashed bytes, since a header outside them changes the identity.
- **When a background sweep must see a consistent whole, invert the lock.** GC, compaction and snapshotting are the *exclusive* holder and take the lock with `try_` so they back off; mutators are the *shared* holders and block; readers still take none. The unit of exclusion is the **transaction**: a caller writing three objects and then a pointer to them holds one guard across all four, or the sweep observes the window where the objects exist and nothing references them.
- **An `O_EXCL` lock file is not crash-safe; an `flock` is.** The kernel releases an `flock` when the holder dies; a stale lock *file* wedges that resource forever. Choose `flock` for anything a crash can interrupt, or write the holder's pid and boot id into the file and define a takeover rule — and say which in the module doc, because "permanently locked" is the failure a reader hits and cannot diagnose.

- **A directory tree inverts the recipe.** Stage in a fresh directory in the *destination's parent*; `rename` onto an existing directory **fails** on Linux rather than replacing it, so the destination must not exist — check and refuse, and say the check is racy. `TempDir::keep()` returns a `PathBuf`, not a `Result` like `NamedTempFile::persist`, so hand-roll a `Staging` guard with a consuming `persist(self, dest)` and a `Drop` that sweeps. Apply restrictive modes in a **final pass immediately before the commit**, never as you build: `remove_dir_all` cannot enter a `0o500` directory the guarded code just created.

Tag the file with a schema version and check it **before** the typed parse. A structural change in a future version otherwise surfaces as `missing field 'tasks'` rather than "this file is newer than this build". Probe with a minimal `#[derive(Deserialize)] struct Probe { version: Option<u32> }`, not a full `serde_json::Value` — the `Value` route materializes the whole document as a generic tree, re-allocates every string, then parses it a second time.

## Drop's two limits

**`Drop` cannot be async and cannot fail.** So for anything with fallible or awaited teardown, expose `async fn close(self) -> Result<()>` as the real path and let `Drop` do a best-effort fallback that logs. And when a type must not be dropped implicitly at all, consuming `self` in `commit`/`abort` plus a `Drop` that warns is as close as Rust gets.

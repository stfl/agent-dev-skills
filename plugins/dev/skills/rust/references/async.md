# Async: tasks, channels, shutdown

With no background task and no channel, only the *Blocking* section below applies — an HTTP handler or a UI loop that merely awaits is not the subject of this file. Async buys concurrency for IO-bound work and nothing else. Keep the decisions sync (see *core and shell* in `SKILL.md`) so this layer stays thin enough to reason about.

## A task owns state; a channel is the way in

When the state is contended and the critical section would await, the default is a task plus a handle — not `Arc<Mutex<State>>`. The state has exactly one owner, so there is no lock to hold, forget, or hold across an await. When reads dominate and the critical section is a few nanoseconds of pure computation, an actor serializes reads against reads and is strictly worse: take `ownership.md`'s sharing ladder instead.


The handle is a newtype over an `mpsc::Sender` — cheap, `Clone`, and the whole public API. Each of its methods sends a command carrying a `oneshot::Sender` for the reply and awaits it; the task owns the state outright and answers in a loop.

The loop returning means every handle was dropped; a send failing means the loop stopped. Both are typed, and neither needs a flag.

Two refinements once the actor carries real state:

- Handle each command in a plain `fn handle(&mut self, cmd: Cmd) -> ControlFlow<()>`. Because it is not `async`, the compiler forbids awaiting mid-transition — so no other message can observe a half-applied change, and `Break` is the fail-stop.
- Put `biased;` first in a `select!` and the first branch becomes the one that cannot be starved — that is the whole decision. A shutdown branch first means cancellation is seen before the next message; a loop racing a deadline against `recv()` puts the *timer* first for bounded latency and a deterministic order — not because tokio would starve it, since the cooperative budget makes `recv()` yield. What genuinely starves a later branch is one that is **permanently ready without consuming budget**: `mpsc::Receiver::recv()` on a closed channel resolves `Ready(None)` on every poll forever, and `CancellationToken::cancelled()` does the same after cancellation. Disable such a branch with a `select!` precondition — `recv(), if !done`. Write down which starvation you chose, and for shutdown also whether it drains the mailbox or preempts it.

## Channels

| | |
|---|---|
| `mpsc` | commands and work queues — **bounded** |
| `oneshot` | the reply half of a request |
| `watch` | latest value only: config reloads, health, "shutdown now" |
| `broadcast` | fan-out events; slow receivers get `Lagged(n)` and you must decide what that means |
| MPMC work queue, N persistent workers | `async-channel` or `flume` — tokio has no MPMC primitive, and `broadcast` is not it: it fans out instead of load-balancing |

Bounded is the whole point: a full `mpsc::channel(64)` makes `send().await` wait, and that backpressure travels all the way to whatever is accepting work. `unbounded_channel` moves the failure from a slow producer to an OOM kill an hour later. Reach for it only when the sender is sync and provably rate-limited.

When a pure core hands the shell a batch to execute, **the batch length is the concurrency** unless you say otherwise. Cap in-flight work with a `Semaphore` sized independently of the batch: a limit that bounds the returned `Vec` bounds allocation, not tasks, and a loop that re-plans immediately when the plan reports truncation reintroduces the unbounded spawn one bounded plan at a time. Say which of the two your limit is, in the limit type's own doc comment.

Limit concurrency without spawning: `stream::iter(jobs).map(run_one).buffer_unordered(16)`, or a `Semaphore` when the limit belongs to a shared resource.

## Spawn structurally

Every spawned task is owned by something. A bare `tokio::spawn` whose `JoinHandle` is dropped is a task nobody can wait for, cancel, or observe panicking.

- `JoinSet` when you need the results, or want them all aborted when the set drops.
- `tokio_util::task::TaskTracker` plus `tokio_util::sync::CancellationToken` for long-lived services: `token.child_token()` per subsystem, `tracker.close(); tracker.wait().await` to drain.
- Propagate a panic: an unawaited task that panics kills only itself, and the service silently degrades.
- Holding the handles is not enough. `let a = h1.await??; let b = h2.await?;` drops `h2` the instant the first `?` fires, detaching a live task with no owner and no way to cancel it. When a function returns `Result` and awaits more than one handle, write the error path first: `.abort()` the survivors, or put the handles in a struct whose `Drop` aborts them.

- Shape the tree by dependency direction: the task owning shared state is the **parent** of the tasks that message it, so "nothing talks to a stopped actor" follows from the shutdown order instead of being intended.
- A loop that returns without having been asked to is a failure, not a completion. Classify the exit rather than letting a dead worker leave the process looking healthy.
- When startup work must finish before anything races — recovery, migration, warmup — return the loop as an inert value the caller places in its tree, rather than spawning inside the constructor.

Shutdown is four phases, not three: stop accepting; wait for the tracker **against a deadline**; cancel the token; wait again against a **second, shorter** deadline; then abort what is left and count it in the report. Cancelling before the first wait means there was no grace period at all, which is the whole point of a drain. Leaving the second wait unbounded is worse — one peer that stops reading its socket makes `SIGTERM` unable to end the process.

A cancellation token reaches a task only at a `select!` the token is a branch of. Every other await in that task — a `write_all`, a `Sink::send`, a `hyper::Connection` you are not selecting on, a library handshake — is uncancellable. Before relying on a token, list the awaits sitting outside its cancellation `select!` and either wrap each in a `timeout` or accept that the task's only bound is `abort`.

A drain returns a report, and the report must say *why* the loop stopped. That fact belongs to the loop that stopped — return it as a value — not to a token queried afterwards. `is_cancelled()` answers "was cancellation requested", a different question from "did this run finish its input"; the two diverge exactly during the drain, and either error makes a supervisor re-ingest completed work or accept a truncated run.

## Cancellation safety

`tokio::select!` drops the losing futures wherever they happen to be. If one of them had consumed half a message, that half is gone.

Safe to hold directly in a `select!` branch: `mpsc::Receiver::recv`, `CancellationToken::cancelled`, `JoinSet::join_next`, `AsyncReadExt::read`. Not safe: `read_exact`, `write_all`, `read_line` (partially-read data is lost — `read_until` keeps it), `mpsc::Sender::send` (you lose your place in the queue *and* the value; the cancel-safe form is `let permit = tx.reserve().await?` in the branch and `permit.send(v)` after it), and anything hand-written that awaits twice with state in between.

A `sleep` is cancel-safe but *not* loop-safe: written inline in a loop's `select!` it is recreated every iteration, so the timer restarts and the deadline never arrives. Pin one outside the loop (`let idle = sleep(d); tokio::pin!(idle);` then `&mut idle`) when the deadline always exists. When it can be *absent*, model it `Option<Instant>` and select on the timer only in the `Some` arm — parking a pinned `Sleep` in the far future is a sentinel standing in for the state the type lacks. `sleep_until` is safe to recreate each iteration, because the `Instant` is absolute. Check the "Cancel safety" section in the docs before putting an unfamiliar future in a branch — and when it is not safe, move the work into a task and select on its handle instead.

Your own `async fn` is a third case the arithmetic above misses: a function that **mutates shared state and then awaits even once** is cancel-unsafe with no `select!` in sight. `tokio::time::timeout`, a dropped `JoinHandle`, and an abandoned `JoinSet` all cancel it identically. If the mutation reserved a resource the future intends to consume, hand the reservation to a `Drop` guard that returns it, or order the await first — and say in the doc comment what a drop between the two costs.

Cancel safety and the bound-your-input rule in `modeling.md` collide for the most ordinary IO task there is: a bounded line reader is a hand-written fn that awaits twice with state in between, so it *cannot* be cancel-safe. Bound it anyway — an OOM is unrecoverable, a lost line is not — put the cancellation check on the outer loop, and name at the `select!` exactly which item can be lost.

## Bounding the outside world

Every await reaching a network, a disk, or another process needs a deadline. For an HTTP client the deadline belongs on the builder — `reqwest::Client::builder().timeout(d).build()` — not at each call site: `Client::new()` has **no** default timeout and waits forever on a server that accepts your request and never answers. Before shipping, grep for `Client::new()`, `::connect(`, and `.send().await` and confirm each sits inside a `timeout` or behind a client that carries one. A retry ladder does not save you: an unbounded first attempt never returns to be retried, and no cancellation token interrupts it.

## Traits with `async fn`

`async fn` in a trait works, but the future it returns carries no `Send` bound, so a generic caller cannot `tokio::spawn` the result. Three ways out, in order:

Write the desugared form — `fn fetch(&self) -> impl Future<Output = Result<T, E>> + Send;` — and implement it with a plain `async fn`; writing the desugared form on both sides trips `clippy::manual_async_fn`. Keep the body lazy: a non-async body returning `ready(...)` does its work at *call* time. `#[async_trait]` remains the answer when you need `dyn Trait`.

Before any of this, re-read `boundaries.md`: most async traits exist to mock one IO call, and are better replaced by a sync core plus a real integration test.

## Blocking

`tokio::io::stdin()` reads on a blocking thread whose read cannot be cancelled, so a pipeline can drain perfectly and the process still hangs at exit while the runtime waits for it. A CLI reading stdin needs an explicit `Runtime` plus `shutdown_timeout(…)` rather than `#[tokio::main]`.

Anything CPU-bound or synchronous running longer than a hundred microseconds — file IO, `serde_json` on a large document, hashing, compression — starves every other task on that worker thread. `spawn_blocking` for one-off blocking calls, `rayon` for parallel CPU work. The symptom of getting this wrong is unrelated requests going slow together.

## Testing it

Make the clock a parameter and the feature is testable with literals: a pure core takes `now` as an argument and never calls `Instant::now()` itself, while the shell supplies it. `#[tokio::test(start_paused = true)]` is the narrower tool — `tokio::time::advance(Duration::from_secs(30)).await` makes a retry-with-backoff test finish instantly — but it moves `Instant` only, so it covers timeouts, `interval` and backoff and does nothing for `SystemTime`, TTLs, or any user-visible timestamp. It also needs tokio's `test-util` feature, which `full` does *not* include.

A service whose deadlines *are* user-visible wall-clock times cannot make `Instant` its domain type and must not call `Utc::now()` in the loop, or a `start_paused` test sleeps on virtual time and compares against real time. Anchor the clocks once — `struct Clock { epoch: DateTime<Utc>, origin: tokio::time::Instant }`, with `now() = epoch + (Instant::now() - origin)` and `deadline_for(t) = origin + (t - epoch)`. Production builds it from `Utc::now()`, a test from a literal: no trait, no mock. And `tokio::time::advance` bumps the clock and yields **once**, so a timer a spawned task has not yet registered gets jumped over; when spawned tasks own the timers, call `tokio::time::sleep` and let `start_paused` auto-advance. Timeout and interval logic should be tested this way rather than with real sleeps.

Instrument across the spawn boundary — a new task does not inherit the current `tracing` span, so pass one in with `.instrument(span)` or the logs from your worker will float free of the request that caused them.

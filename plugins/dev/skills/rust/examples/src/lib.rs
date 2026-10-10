//! Every code block from the `rust` skill, adapted only by filling elided bodies.
#![allow(dead_code, unused_variables, clippy::all)]

// ── modeling.md #1: parse at the boundary ────────────────────────────────────
mod m1 {
    #[derive(Debug, thiserror::Error)]
    #[error("invalid email {input:?}: {reason}")]
    pub struct InvalidEmail {
        pub input: String,
        pub reason: &'static str,
    }

    pub struct Email(String); // private field: this module is the invariant boundary

    impl Email {
        pub fn parse(raw: &str) -> Result<Self, InvalidEmail> {
            if raw.contains('@') {
                Ok(Self(raw.to_owned()))
            } else {
                Err(InvalidEmail { input: raw.to_owned(), reason: "missing @" })
            }
        }
        pub fn as_str(&self) -> &str { &self.0 }
    }
}

// ── modeling.md #2: enum over flags ──────────────────────────────────────────
mod m2 {
    use std::time::{Duration, Instant};
    #[derive(Debug)]
    pub struct JobError;

    // sixteen combinations, three of them legal
    struct JobFlags {
        pending: bool,
        started_at: Option<Instant>,
        output: Option<Vec<u8>>,
        error: Option<String>,
    }

    // Three variants, each carrying exactly what it has.
    enum Job {
        Pending,
        Running { started_at: Instant },
        Finished { output: Vec<u8> },
        Failed { error: JobError },
    }

    fn _use(j: &Job) -> Option<Duration> {
        match j {
            Job::Running { started_at } => Some(started_at.elapsed()),
            _ => None,
        }
    }
}

// ── modeling.md #3: serde without a hole ─────────────────────────────────────
mod m3 {
    use super::m1::{Email as Inner, InvalidEmail};

    #[derive(serde::Deserialize)]
    #[serde(try_from = "String")]
    pub struct Email(String);

    impl TryFrom<String> for Email {
        type Error = InvalidEmail;
        fn try_from(v: String) -> Result<Self, Self::Error> {
            Inner::parse(&v).map(|e| Email(e.as_str().to_owned()))
        }
    }
}

// serde(transparent) on a newtype, as claimed in modeling.md
mod m3b {
    #[derive(serde::Serialize, serde::Deserialize)]
    #[serde(transparent)]
    pub struct UserId(u64);

    #[repr(transparent)]
    pub struct Handle(u32);
}

// ── lifecycle.md #1: typestate ───────────────────────────────────────────────
mod l1 {
    use std::io;
    use std::marker::PhantomData;
    use std::net::TcpStream;

    pub struct Open;
    pub struct Closed;

    pub struct Conn<S> {
        sock: TcpStream,
        _s: PhantomData<S>,
    }

    impl Conn<Open> {
        pub fn send(&mut self, b: &[u8]) -> io::Result<()> {
            use std::io::Write;
            self.sock.write_all(b)
        }
        pub fn close(self) -> Conn<Closed> {
            Conn { sock: self.sock, _s: PhantomData }
        }
    }
}

// ── lifecycle.md #2: enum state machine with mem::replace ────────────────────
mod l2 {
    use std::time::{Duration, Instant};

    pub enum Event { Start, Done }

    pub enum State {
        Idle,
        Running { since: Instant },
        Finished { took: Duration },
        Poisoned,
    }

    pub struct Machine { state: State }

    impl Machine {
        fn advance(&mut self, ev: Event) {
            self.state = match (std::mem::replace(&mut self.state, State::Poisoned), ev) {
                (State::Idle, Event::Start) => State::Running { since: Instant::now() },
                (State::Running { since }, Event::Done) => State::Finished { took: since.elapsed() },
                (other, _) => other,
            };
        }
    }
}

// ── lifecycle.md #3: Drop guard ──────────────────────────────────────────────
mod l3 {
    use std::time::{Duration, Instant};

    fn record(_name: &'static str, _took: Duration) {}

    #[must_use = "the span ends when this guard is dropped"]
    pub struct Timer { name: &'static str, start: Instant }

    impl Timer {
        pub fn start(name: &'static str) -> Self { Self { name, start: Instant::now() } }
    }

    impl Drop for Timer {
        fn drop(&mut self) { record(self.name, self.start.elapsed()); }
    }

    fn caller() {
        let _guard = Timer::start("db.query");
    }
}

// ── boundaries.md #1: sealing ────────────────────────────────────────────────
mod b1 {
    mod sealed { pub trait Sealed {} }

    pub struct Utf8;

    pub trait Encoding: sealed::Sealed {
        fn encode(&self, out: &mut Vec<u8>);
    }

    impl sealed::Sealed for Utf8 {}
    impl Encoding for Utf8 {
        fn encode(&self, out: &mut Vec<u8>) { out.push(0); }
    }
}

// extension trait + blanket impl, as claimed in boundaries.md
mod b2 {
    pub trait ResultExt { fn log_err(self) -> Self; }

    impl<T, E: std::fmt::Debug> ResultExt for Result<T, E> {
        fn log_err(self) -> Self {
            if let Err(e) = &self { eprintln!("{e:?}"); }
            self
        }
    }
}

// ── errors.md #1: error enum (sqlx::Error substituted by io::Error) ──────────
mod e1 {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct AccountId(u64);
    impl std::fmt::Display for AccountId {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { write!(f, "{}", self.0) }
    }

    #[derive(Debug, thiserror::Error)]
    #[non_exhaustive]
    pub enum StoreError {
        #[error("no account {0}")]
        NotFound(AccountId),
        #[error("account {0} is frozen")]
        Frozen(AccountId),
        #[error("storage backend unavailable")]
        Backend(#[from] std::io::Error),
    }

    impl StoreError {
        pub fn is_retryable(&self) -> bool { matches!(self, StoreError::Backend(_)) }
    }
}

// ── ownership.md: tagged arena + single indexing funnel (v2) ────────────────
mod o1 {
    #[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
    pub struct ArenaId(u32);
    #[derive(Debug)]
    pub struct Node;

    #[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
    pub struct NodeId { arena: ArenaId, index: u32 }

    #[derive(Debug)]
    pub struct Graph { id: ArenaId, nodes: Vec<Node> }

    impl Graph {
        #[track_caller]
        #[expect(clippy::indexing_slicing, reason = "index minted by this arena, checked above")]
        fn slot(&self, id: NodeId) -> &Node {
            assert_eq!(id.arena, self.id, "handle from a different arena");
            &self.nodes[id.index as usize]
        }
    }
}

// ownership.md: the borrow-error moves 1 and 2
mod o2 {
    use std::collections::HashMap;

    fn shorten(map: &mut HashMap<String, usize>, k: String) {
        let n = map.len();
        map.insert(k, n);
    }

    struct Pipe { input: Vec<u8>, output: Vec<u8> }
    impl Pipe {
        fn split(&mut self) {
            let Self { input, output, .. } = self;
            output.extend_from_slice(input);
        }
    }
}

// ── async.md #1: actor task + handle ─────────────────────────────────────────
mod a1 {
    use std::collections::HashMap;
    use tokio::sync::{mpsc, oneshot};

    pub type Key = String;
    pub type Value = String;
    #[derive(Debug)]
    pub struct Gone;

    #[derive(Default)]
    pub struct State(HashMap<Key, Value>);
    impl State {
        fn get(&self, k: &Key) -> Option<Value> { self.0.get(k).cloned() }
        fn set(&mut self, k: Key, v: Value) { self.0.insert(k, v); }
    }

    pub enum Cmd {
        Get { key: Key, reply: oneshot::Sender<Option<Value>> },
        Set { key: Key, value: Value },
    }

    #[derive(Clone)]
    pub struct Store(mpsc::Sender<Cmd>);

    impl Store {
        pub async fn get(&self, key: Key) -> Result<Option<Value>, Gone> {
            let (reply, rx) = oneshot::channel();
            self.0.send(Cmd::Get { key, reply }).await.map_err(|_| Gone)?;
            rx.await.map_err(|_| Gone)
        }
    }

    async fn run(mut rx: mpsc::Receiver<Cmd>, mut state: State) {
        while let Some(cmd) = rx.recv().await {
            match cmd {
                Cmd::Get { key, reply } => { let _ = reply.send(state.get(&key)); }
                Cmd::Set { key, value } => state.set(key, value),
            }
        }
    }
}

// async.md: buffer_unordered claim
mod a2 {
    use futures::stream::{self, StreamExt};

    async fn run_one(j: u32) -> u32 { j }

    pub async fn fanout(jobs: Vec<u32>) -> Vec<u32> {
        stream::iter(jobs).map(run_one).buffer_unordered(16).collect().await
    }
}

// ── async.md (v2): declare desugared with + Send, implement with `async fn` ──
mod a3 {
    use std::future::Future;

    pub trait Fetch {
        fn fetch(&self) -> impl Future<Output = Result<Vec<u8>, std::io::Error>> + Send;
    }
    pub struct RealFetch;
    impl Fetch for RealFetch {
        async fn fetch(&self) -> Result<Vec<u8>, std::io::Error> {
            tokio::task::yield_now().await;
            Ok(Vec::new())
        }
    }
    pub fn spawns<T: Fetch + Send + Sync + 'static>(t: T) {
        tokio::spawn(async move { t.fetch().await });
    }

    #[trait_variant::make(Send)]
    pub trait FetchVariant {
        async fn fetch(&self) -> Result<Vec<u8>, std::io::Error>;
    }

    pub async fn shutdown() {
        let token = tokio_util::sync::CancellationToken::new();
        let tracker = tokio_util::task::TaskTracker::new();
        let child = token.child_token();
        tracker.spawn(async move { child.cancelled().await });
        token.cancel();
        tracker.close();
        tracker.wait().await;
    }
}

// ── async.md (v2): cancel-safe send via reserve/permit; stdin shutdown ───────
mod a5 {
    use tokio::sync::{mpsc, Notify};

    pub async fn cancel_safe_send(tx: &mpsc::Sender<u32>, quit: &Notify, v: u32) {
        tokio::select! {
            Ok(permit) = tx.reserve() => { permit.send(v); }
            () = quit.notified() => {}
        }
    }

    pub fn stdin_safe_main() -> std::io::Result<()> {
        let rt = tokio::runtime::Runtime::new()?;
        rt.block_on(async {});
        rt.shutdown_timeout(std::time::Duration::from_millis(100));
        Ok(())
    }
}

// ── verify.md (v3): the test-module blanket, exactly as written ─────────────
#[cfg(test)]
mod blanket_probe {
    #![allow(clippy::unwrap_used, clippy::expect_used, clippy::indexing_slicing,
             reason = "a failed unwrap or a panicking index is a failing test")]
    #[test]
    fn uses_them() {
        let v = vec![1u8, 2, 3];
        assert_eq!(v[0], 1);
        assert_eq!(Some(2u8).unwrap(), 2);
    }
}

// ── ownership.md (v4): FFI handle shape + Send/Sync denial proof ────────────
mod ffi_probe {
    use std::ptr::NonNull;
    #[repr(C)]
    pub struct RcHandle { _private: [u8; 0] }

    #[derive(Debug)]
    pub struct Encoder(NonNull<RcHandle>);
    // NonNull denies both Send and Sync; Send is opted back in deliberately.
    // SAFETY: the C handle owns no thread-local state.
    unsafe impl Send for Encoder {}

    pub fn assert_send<T: Send>() {}
    // `assert_sync::<Encoder>()` would not compile — Encoder stays !Sync.

    // out-parameter shape from the FFI section
    pub fn out_param(cap: usize, written: usize) -> Vec<u8> {
        let mut v: Vec<u8> = Vec::with_capacity(cap);
        let _spare = v.spare_capacity_mut();
        assert!(written <= cap);
        // SAFETY: the callee reported `written` bytes initialised, bounded above.
        unsafe { v.set_len(written) };
        v
    }
}

// ── modeling.md: the bounded line reader (by_ref, so src is not moved) ──────
mod bounded_reader {
    use std::io::{BufRead, Read};   // `by_ref`/`take` are Read methods — both traits must be in scope
    pub fn read_bounded<R: BufRead>(src: &mut R, limit: usize) -> std::io::Result<Vec<u8>> {
        let mut buf = Vec::new();
        let mut limited = src.by_ref().take(limit as u64 + 1);
        limited.read_until(b'\n', &mut buf)?;
        if buf.len() > limit {
            let mut skip = Vec::new();
            src.read_until(b'\n', &mut skip)?;   // src still usable
        }
        Ok(buf)
    }
}

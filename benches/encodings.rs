//! Same machine, five encodings. This is the fair comparison.
//!
//! ```text
//! cargo bench --bench encodings
//! ```
//!
//! Criterion’s default measurement is **wall-clock** (`WallTime` via
//! `std::time::Instant`): nanoseconds per iteration, warmup, outlier
//! detection, 95% CI. `Throughput::Elements` turns that into
//! transitions/sec. We do **not** use a raw `Instant` loop (noisy) and we
//! do **not** claim cachegrind/`perf` cycles (Linux-only, not this crate).
//!
//! **Workload (identical semantics):** two XOR regions (auth × sync).
//! Each tick increments a `u64` and toggles sync Idle↔Busy. That counter
//! is `black_box`’d so LLVM cannot delete the loop. 1 tick = 1 logical
//! transition.
//!
//! | Name | Encoding | Extra cost vs a field write |
//! | --- | --- | --- |
//! | `handwritten_fields` | struct fields, **no** `Runtime` | none — the floor |
//! | `newton_runtime` | same nested ADT, `Runtime::apply` | one virtual call into `Machine::update` (static dispatch) |
//! | `mega_enum` | cartesian product `AnonIdle\|…` | same speed at 2×2; 3 regions × 4 children = 64 variants by hand |
//! | `string_id_set` | `HashSet<&'static str>` current-set | hash + string ids: SCXML/interpreter shape |
//! | `gof_box_dyn` | `Box<dyn State>` per region | heap alloc + vtable: GoF State. Payload is a `u64` so `Box` actually allocates (`Box<ZST>` would not). |
//!
//! **Claim this crate can defend:** nested ADT + `Runtime` sits on the
//! handwritten floor; interpreter/vtable encodings do not. **Do not
//! claim:** “faster than XState” (different language). **Do not claim:**
//! “faster than a `match` you would have written anyway” — that *is*
//! `handwritten_fields`. Newton’s pitch is *same speed, illegal XOR does
//! not compile, snapshot is a triple*.
//!
//! HTML: `target/criterion/report/index.html`.

#![allow(clippy::unit_arg)] // black_box(()) is the sink, not a useless unit

use std::collections::HashSet;
use std::time::Duration;

use criterion::measurement::WallTime;
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use newton_machine::prelude::*;

const BATCH: u64 = 1024;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Auth {
    Anon,
    /// Present so the product space matches the mega-enum (Login is off this path).
    #[allow(dead_code)]
    User,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Sync {
    Idle,
    Busy,
}

#[inline(always)]
fn toggle_sync(s: Sync) -> Sync {
    match s {
        Sync::Idle => Sync::Busy,
        Sync::Busy => Sync::Idle,
    }
}

// --- 0. Handwritten nested ADT, no Runtime --------------------------------

#[derive(Clone, Copy)]
struct Hand {
    auth: Auth,
    sync: Sync,
    n: u64,
}

impl Hand {
    fn new() -> Self {
        Self {
            auth: Auth::Anon,
            sync: Sync::Idle,
            n: 0,
        }
    }

    #[inline(always)]
    fn tick(&mut self) {
        self.n = self.n.wrapping_add(1);
        self.sync = toggle_sync(self.sync);
    }
}

// --- 1. Newton nested ADT + Runtime ---------------------------------------

#[derive(Clone, Copy)]
struct Newton {
    auth: Auth,
    sync: Sync,
}

impl Machine for Newton {
    type Flags = ();
    type Model = u64;
    type Msg = ();
    type Cmd = ();
    type View = (Auth, Sync, u64);
    type History = ();
    type NodeId = ();

    fn init(_: ()) -> Boot<Self> {
        Boot::new(
            Newton {
                auth: Auth::Anon,
                sync: Sync::Idle,
            },
            0,
            (),
            (),
        )
    }

    fn update(&mut self, n: &mut u64, _: &mut (), _: ()) {
        *n = n.wrapping_add(1);
        self.sync = toggle_sync(self.sync);
    }

    fn view(&self, n: &u64) -> (Auth, Sync, u64) {
        (self.auth, self.sync, *n)
    }

    fn in_state(&self, _: ()) -> bool {
        true
    }
}

// --- 2. Mega-enum (cartesian product) -------------------------------------

#[derive(Clone, Copy)]
struct Mega {
    state: MegaState,
    n: u64,
}

#[derive(Clone, Copy)]
enum MegaState {
    AnonIdle,
    AnonBusy,
    UserIdle,
    UserBusy,
}

impl Mega {
    fn new() -> Self {
        Self {
            state: MegaState::AnonIdle,
            n: 0,
        }
    }

    #[inline(always)]
    fn tick(&mut self) {
        self.n = self.n.wrapping_add(1);
        self.state = match self.state {
            MegaState::AnonIdle => MegaState::AnonBusy,
            MegaState::AnonBusy => MegaState::AnonIdle,
            MegaState::UserIdle => MegaState::UserBusy,
            MegaState::UserBusy => MegaState::UserIdle,
        };
    }
}

// --- 3. String-id current-set (interpreter / SCXML-ish) -------------------

struct StringSet {
    set: HashSet<&'static str>,
    n: u64,
}

impl StringSet {
    fn new() -> Self {
        let mut set = HashSet::with_capacity(8);
        set.insert("auth.anon");
        set.insert("sync.idle");
        Self { set, n: 0 }
    }

    fn tick(&mut self) {
        self.n = self.n.wrapping_add(1);
        if self.set.contains("sync.idle") {
            self.set.remove("sync.idle");
            self.set.insert("sync.busy");
        } else {
            self.set.remove("sync.busy");
            self.set.insert("sync.idle");
        }
    }
}

// --- 4. GoF State: Box<dyn> per region ------------------------------------

trait SyncBox {
    fn tick(self: Box<Self>) -> Box<dyn SyncBox>;
    fn n(&self) -> u64;
}

/// Non-ZST on purpose: `Box<ZST>` does not allocate, which would understate
/// GoF. Real state objects hold data; this `u64` forces a heap hit per tick.
struct Idle {
    n: u64,
}
struct Busy {
    n: u64,
}

impl SyncBox for Idle {
    fn tick(self: Box<Self>) -> Box<dyn SyncBox> {
        Box::new(Busy {
            n: self.n.wrapping_add(1),
        })
    }
    fn n(&self) -> u64 {
        self.n
    }
}
impl SyncBox for Busy {
    fn tick(self: Box<Self>) -> Box<dyn SyncBox> {
        Box::new(Idle {
            n: self.n.wrapping_add(1),
        })
    }
    fn n(&self) -> u64 {
        self.n
    }
}

struct Gof {
    auth: Auth,
    sync: Option<Box<dyn SyncBox>>,
}

impl Gof {
    fn new() -> Self {
        Self {
            auth: Auth::Anon,
            sync: Some(Box::new(Idle { n: 0 })),
        }
    }
    fn tick(&mut self) {
        let _keep = self.auth;
        let cur = self.sync.take().expect("sync region");
        self.sync = Some(cur.tick());
    }
    fn n(&self) -> u64 {
        self.sync.as_ref().expect("sync region").n()
    }
}

fn encodings(c: &mut Criterion<WallTime>) {
    let mut g = c.benchmark_group("encoding_tick");
    g.throughput(Throughput::Elements(1));
    g.warm_up_time(Duration::from_millis(500));
    g.measurement_time(Duration::from_secs(3));
    g.sample_size(100);

    g.bench_function(BenchmarkId::from_parameter("handwritten_fields"), |b| {
        let mut h = Hand::new();
        b.iter(|| {
            h.tick();
            black_box(h.n);
            black_box(h.auth);
            black_box(h.sync);
        });
    });

    g.bench_function(BenchmarkId::from_parameter("newton_runtime"), |b| {
        let (mut rt, _) = Runtime::<Newton>::boot(());
        b.iter(|| {
            rt.apply(black_box(()));
            black_box(rt.view());
        });
    });

    g.bench_function(BenchmarkId::from_parameter("mega_enum"), |b| {
        let mut m = Mega::new();
        b.iter(|| {
            m.tick();
            black_box(m.n);
            black_box(m.state);
        });
    });

    g.bench_function(BenchmarkId::from_parameter("string_id_set"), |b| {
        let mut s = StringSet::new();
        b.iter(|| {
            s.tick();
            black_box(s.n);
            black_box(s.set.len());
        });
    });

    g.bench_function(BenchmarkId::from_parameter("gof_box_dyn"), |b| {
        let mut gof = Gof::new();
        b.iter(|| {
            gof.tick();
            black_box(gof.n());
        });
    });

    g.finish();

    // Wall-clock of a 1024-step burst (a “bar loop”). Throughput is
    // transitions/sec across the whole batch.
    let mut g = c.benchmark_group("encoding_batch_1024");
    g.throughput(Throughput::Elements(BATCH));
    g.warm_up_time(Duration::from_millis(500));
    g.measurement_time(Duration::from_secs(3));
    g.sample_size(80);

    g.bench_function("handwritten_fields", |b| {
        let mut h = Hand::new();
        b.iter(|| {
            // Sink per tick so LLVM cannot strength-reduce `n += 1024`.
            for _ in 0..BATCH {
                h.tick();
                black_box(h.n);
            }
            black_box(h.sync);
        });
    });
    g.bench_function("newton_runtime", |b| {
        let (mut rt, _) = Runtime::<Newton>::boot(());
        b.iter(|| {
            for _ in 0..BATCH {
                rt.apply(());
                black_box(*rt.model());
            }
        });
    });
    g.bench_function("mega_enum", |b| {
        let mut m = Mega::new();
        b.iter(|| {
            for _ in 0..BATCH {
                m.tick();
                black_box(m.n);
            }
        });
    });
    g.bench_function("string_id_set", |b| {
        let mut s = StringSet::new();
        b.iter(|| {
            for _ in 0..BATCH {
                s.tick();
                black_box(s.n);
            }
        });
    });
    g.bench_function("gof_box_dyn", |b| {
        let mut gof = Gof::new();
        b.iter(|| {
            for _ in 0..BATCH {
                gof.tick();
                black_box(gof.n());
            }
        });
    });

    g.finish();
}

criterion_group!(benches, encodings);
criterion_main!(benches);

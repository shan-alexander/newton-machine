//! Composition costs: `And<L, R>` vs a handwritten AND struct, and `Cmd::and`.
//!
//! ```text
//! cargo bench --bench compose
//! ```
//!
//! **AND.** Harel orthogonal regions are "concurrent in meaning." The crate
//! offers two encodings:
//!
//! - Handwritten `struct Session { auth, sync }` — one `Model`, one `Msg`
//!   offered in field order. Zero `Clone` of the message.
//! - `And<AuthM, SyncM>` — first-class combinator. Each region owns its
//!   config. The `Msg` is cloned once so both `update`s can take it by
//!   value (law: no shared `&mut` of the sibling).
//!
//! Cross-encoding (mega-enum / HashSet ids / `Box<dyn State>`) lives in
//! `--bench encodings`. This file is *our* combinator vs *our* struct.
//!
//! If `and_combinator_tick` is within a small constant of
//! `handwritten_struct_tick`, pick the combinator for the type-safety
//! (`AndNode` cannot mix regions). If it is a lot slower, your `Msg` is
//! expensive to clone — use a handwritten struct or a cheap `Msg`.
//!
//! **Cmd.** `INLINE_CAP` is 4. `and` of two atoms is the `perform`
//! (exit+enter) case and must not allocate. The fifth atom spills to
//! `Vec` when `alloc` is on. This group shows that split. Without `alloc`
//! the fifth `and` panics (a chart bug); we do not bench panics.
//!
//! **Sub::diff.** The host’s start/stop. Cheap `PartialEq` on `&str`
//! listeners; not a socket benchmark.

use std::time::Duration;

use criterion::measurement::WallTime;
use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};
use newton_machine::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Auth {
    Anon,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Sync {
    Idle,
    Busy,
}

#[derive(Clone, Copy, Debug)]
enum Msg {
    Tick,
}

// --- handwritten product (shared Model) ---

#[derive(Clone, Copy, Debug)]
struct Session {
    auth: Auth,
    sync: Sync,
}

impl Machine for Session {
    type Flags = ();
    type Model = u32;
    type Msg = Msg;
    type Cmd = ();
    type View = (Auth, Sync);
    type History = ();
    type NodeId = &'static str;

    fn init(_: ()) -> Boot<Self> {
        Boot::new(
            Session {
                auth: Auth::Anon,
                sync: Sync::Idle,
            },
            0,
            (),
            (),
        )
    }

    fn update(&mut self, ticks: &mut u32, _: &mut (), msg: Msg) {
        if matches!(msg, Msg::Tick) {
            *ticks += 1;
            self.sync = match self.sync {
                Sync::Idle => Sync::Busy,
                Sync::Busy => Sync::Idle,
            };
        }
    }

    fn view(&self, _: &u32) -> (Auth, Sync) {
        (self.auth, self.sync)
    }

    fn in_state(&self, id: &'static str) -> bool {
        match id {
            "anon" => self.auth == Auth::Anon,
            "idle" => self.sync == Sync::Idle,
            "busy" => self.sync == Sync::Busy,
            _ => false,
        }
    }
}

// --- And combinator (split Model, Msg: Clone) ---

struct AuthM(Auth);

impl Machine for AuthM {
    type Flags = ();
    type Model = ();
    type Msg = Msg;
    type Cmd = ();
    type View = Auth;
    type History = ();
    type NodeId = &'static str;

    fn init(_: ()) -> Boot<Self> {
        Boot::new(AuthM(Auth::Anon), (), (), ())
    }

    fn update(&mut self, _: &mut (), _: &mut (), _: Msg) {}

    fn view(&self, _: &()) -> Auth {
        self.0
    }

    fn in_state(&self, id: &'static str) -> bool {
        id == "anon"
    }
}

struct SyncM(Sync);

impl Machine for SyncM {
    type Flags = ();
    type Model = u32;
    type Msg = Msg;
    type Cmd = ();
    type View = Sync;
    type History = ();
    type NodeId = &'static str;

    fn init(_: ()) -> Boot<Self> {
        Boot::new(SyncM(Sync::Idle), 0, (), ())
    }

    fn update(&mut self, ticks: &mut u32, _: &mut (), msg: Msg) {
        if matches!(msg, Msg::Tick) {
            *ticks += 1;
            self.0 = match self.0 {
                Sync::Idle => Sync::Busy,
                Sync::Busy => Sync::Idle,
            };
        }
    }

    fn view(&self, _: &u32) -> Sync {
        self.0
    }

    fn in_state(&self, id: &'static str) -> bool {
        matches!((id, self.0), ("idle", Sync::Idle) | ("busy", Sync::Busy))
    }
}

type Pair = And<AuthM, SyncM>;

fn and_vs_struct(c: &mut Criterion<WallTime>) {
    let mut g = c.benchmark_group("and");
    g.throughput(Throughput::Elements(1));
    g.warm_up_time(Duration::from_millis(400));
    g.measurement_time(Duration::from_secs(3));
    g.sample_size(100);

    g.bench_function("handwritten_struct_tick", |b| {
        let (mut rt, _) = Runtime::<Session>::boot(());
        b.iter(|| {
            rt.apply(black_box(Msg::Tick));
            black_box(*rt.model());
            black_box(rt.view());
        });
    });

    g.bench_function("and_combinator_tick", |b| {
        let (mut rt, _) = Runtime::<Pair>::boot(((), ()));
        b.iter(|| {
            rt.apply(black_box(Msg::Tick));
            black_box(*rt.model());
            black_box(rt.view());
        });
    });

    g.finish();
}

fn cmd_concat(c: &mut Criterion<WallTime>) {
    let mut g = c.benchmark_group("cmd");
    g.throughput(Throughput::Elements(1));
    g.warm_up_time(Duration::from_millis(400));
    g.measurement_time(Duration::from_secs(2));
    g.sample_size(80);

    g.bench_function("and_2_stack", |b| {
        b.iter(|| black_box(Cmd::single(1u8).and(Cmd::single(2))));
    });
    g.bench_function("and_4_stack", |b| {
        b.iter(|| {
            black_box(
                Cmd::single(1u8)
                    .and(Cmd::single(2))
                    .and(Cmd::single(3))
                    .and(Cmd::single(4)),
            )
        });
    });
    // Fifth atom: heap with default `alloc`. This is the spill, not the
    // common perform(exit, enter) path.
    g.bench_function("and_5_heap", |b| {
        b.iter(|| {
            black_box(
                Cmd::single(1u8)
                    .and(Cmd::single(2))
                    .and(Cmd::single(3))
                    .and(Cmd::single(4))
                    .and(Cmd::single(5)),
            )
        });
    });

    g.finish();
}

fn sub_diff(c: &mut Criterion<WallTime>) {
    let mut g = c.benchmark_group("sub_diff");
    g.throughput(Throughput::Elements(1));
    g.warm_up_time(Duration::from_millis(400));
    g.measurement_time(Duration::from_secs(2));
    g.sample_size(80);

    let none = Sub::<&'static str>::none();
    let one = Sub::single("clock");
    let two = Sub::many(["clock", "feed"]);
    let swap = Sub::many(["feed", "keys"]);

    g.bench_function("none_to_one", |b| {
        b.iter(|| black_box(none.diff(black_box(&one))));
    });
    g.bench_function("unchanged_one", |b| {
        b.iter(|| black_box(one.diff(black_box(&one))));
    });
    g.bench_function("swap_two", |b| {
        b.iter(|| black_box(two.diff(black_box(&swap))));
    });

    g.finish();
}

criterion_group!(benches, and_vs_struct, cmd_concat, sub_diff);
criterion_main!(benches);

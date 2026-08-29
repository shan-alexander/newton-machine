//! Hot path of a live Newton machine: `Runtime::apply`.
//!
//! ```text
//! cargo bench --bench apply
//! ```
//!
//! **What to read this for (no clone required):** the crate's claim is that
//! `apply` is one function, in place, and `step` is the same semantics with
//! owned values for tests/replay. This file puts a number on that split.
//!
//! | Function | What you pay | When to use |
//! | --- | --- | --- |
//! | `Runtime::apply` | mutate the triple in place | live host |
//! | `step` | clone-or-move the triple out and back | tests, journals |
//! | `view` | read-only projection | HUD / telemetry; must stay cheaper than a transition |
//!
//! Do **not** conclude "Newton is faster than XState." This is not a
//! cross-crate shoot-out. It is *our* apply vs *our* step on a tiny XOR
//! that actually calls `perform` (LCA), so you are not timing an empty
//! `match`.
//!
//! Times are Criterion **wall-clock** (`WallTime`). `Throughput::Elements(1)`
//! is transitions/sec. The `view` bench is a sanity floor: if it is in the
//! same band as `apply`, the transition is too cheap to care — or `view`
//! is doing too much.
//!
//! Criterion HTML: `target/criterion/report/index.html`.

#![allow(clippy::unit_arg)] // black_box(()) is the sink, not a useless unit

use std::time::Duration;

use criterion::measurement::WallTime;
use criterion::{black_box, criterion_group, criterion_main, BatchSize, Criterion, Throughput};
use newton_machine::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Node {
    Root,
    Off,
    On,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Chart {
    Off,
    On,
}

impl Chart {
    fn node(self) -> Node {
        match self {
            Chart::Off => Node::Off,
            Chart::On => Node::On,
        }
    }
}

impl Topology for Chart {
    type Node = Node;
    fn parent(node: Node) -> Option<Node> {
        match node {
            Node::Root => None,
            Node::Off | Node::On => Some(Node::Root),
        }
    }
}

impl Transitional for Chart {
    type Ctx = u32;
    type Hist = ();
    type Cmd = Cmd<u8>;

    fn enter(&mut self, node: Node, ctx: &mut u32, _: &mut ()) -> Cmd<u8> {
        if node == Node::On {
            *ctx += 1;
            Cmd::single(1)
        } else {
            Cmd::none()
        }
    }
}

impl Machine for Chart {
    type Flags = ();
    type Model = u32;
    type Msg = ();
    type Cmd = Cmd<u8>;
    type View = bool;
    type History = ();
    type NodeId = Node;

    fn init(_: ()) -> Boot<Self> {
        Boot::new(Chart::Off, 0, (), Cmd::none())
    }

    fn update(&mut self, model: &mut u32, hist: &mut (), _: ()) -> Cmd<u8> {
        let dest = match self {
            Chart::Off => Chart::On,
            Chart::On => Chart::Off,
        };
        let from = self.node();
        let to = dest.node();
        perform(self, from, dest, to, model, hist)
    }

    fn view(&self, _: &u32) -> bool {
        matches!(self, Chart::On)
    }

    fn in_state(&self, id: Node) -> bool {
        self.node() == id || id == Node::Root
    }
}

fn apply_in_place(c: &mut Criterion<WallTime>) {
    let mut group = c.benchmark_group("apply");
    group.throughput(Throughput::Elements(1));
    group.warm_up_time(Duration::from_millis(500));
    group.measurement_time(Duration::from_secs(3));
    group.sample_size(100);

    // Live path. Setup is outside the timed closure except the Runtime
    // itself, which we recycle: toggling Off↔On is the steady state.
    // black_box the Cmd *and* the model so LLVM cannot drop `enter`.
    group.bench_function("runtime_apply_toggle", |b| {
        let (mut rt, _) = Runtime::<Chart>::boot(());
        b.iter(|| {
            let cmd = rt.apply(black_box(()));
            let _ = black_box(cmd);
            black_box(*rt.model());
            black_box(rt.view());
        });
    });

    // Owned TEA path. Each iteration *moves* a fresh triple in; that is
    // the cost of `step` (tests, persist-after-event). Compare to apply.
    group.bench_function("step_owned_toggle", |b| {
        b.iter_batched(
            || {
                let boot = Chart::init(());
                (boot.machine, boot.model, boot.history)
            },
            |(m, model, hist)| {
                let out = step(m, model, hist, black_box(()));
                black_box(out)
            },
            BatchSize::SmallInput,
        );
    });

    // Query. If this is in the same nanosecond band as apply, the
    // transition is too cheap to care — or view is doing too much.
    group.bench_function("runtime_view", |b| {
        let (rt, _) = Runtime::<Chart>::boot(());
        b.iter(|| black_box(rt.view()));
    });

    group.finish();
}

criterion_group!(benches, apply_in_place);
criterion_main!(benches);

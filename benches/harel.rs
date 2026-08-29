//! Harel kinematics: LCA / `perform`, and RTC drain.
//!
//! ```text
//! cargo bench --bench harel
//! ```
//!
//! **Read this if you are choosing Newton vs a string-id chart.** The
//! expensive part of a Harel tool is often "walk the document." Here the
//! parent function is a `match` on a `Copy` enum and the walk is bounded
//! by `MAX_DEPTH` (32) on the stack. These numbers are *that* cost, not
//! I/O, not serde, not a GUI.
//!
//! Groups:
//!
//! 1. **`lca_*`** — ancestor walk only (no exit/enter actions).
//! 2. **`perform_*`** — full transition: exit inner-first, assign, enter
//!    outer-first, concatenating `Cmd` (stack, ≤4 atoms).
//! 3. **`rtc_drain_*`** — internal follow-ups *inside one external step*.
//!    31 is just under the default cap (32). A storm is a *bug*, not a
//!    throughput mode: we do not bench panicking `unwrap_storm`.
//!
//! Sibling vs deep: if `perform_descend` is only a few ns above
//! `perform_sibling`, LCA is not your bottleneck. Your `exit`/`enter`
//! bodies will be.
//!
//! Times are Criterion **wall-clock**. 31 follow-ups is a legal (if rude)
//! step just under the default cap; a storm is a bug, not a throughput
//! mode, so we do not bench panicking `unwrap_storm`.

use std::time::Duration;

use criterion::measurement::WallTime;
use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};
use newton_machine::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
enum Node {
    Root,
    A,
    B,
    B1,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Chart {
    A,
    B { deep: bool },
}

impl Topology for Chart {
    type Node = Node;
    fn parent(node: Node) -> Option<Node> {
        match node {
            Node::Root => None,
            Node::A | Node::B => Some(Node::Root),
            Node::B1 => Some(Node::B),
        }
    }
}

impl Transitional for Chart {
    type Ctx = ();
    type Hist = ();
    type Cmd = Cmd<u8>;

    fn exit(&mut self, node: Node, _: &mut (), _: &mut ()) -> Cmd<u8> {
        Cmd::single(node as u8)
    }

    fn enter(&mut self, node: Node, _: &mut (), _: &mut ()) -> Cmd<u8> {
        Cmd::single(node as u8)
    }
}

fn lca_and_perform(c: &mut Criterion<WallTime>) {
    let mut g = c.benchmark_group("harel");
    g.throughput(Throughput::Elements(1));
    g.warm_up_time(Duration::from_millis(400));
    g.measurement_time(Duration::from_secs(2));
    g.sample_size(80);

    g.bench_function("lca_sibling", |b| {
        b.iter(|| black_box(Chart::lca(Node::A, Node::B)));
    });
    g.bench_function("lca_deep", |b| {
        b.iter(|| black_box(Chart::lca(Node::A, Node::B1)));
    });

    g.bench_function("perform_sibling", |b| {
        b.iter(|| {
            let mut chart = Chart::A;
            let dest = Chart::B { deep: false };
            let cmd = perform(&mut chart, Node::A, dest, Node::B, &mut (), &mut ());
            black_box((chart, cmd))
        });
    });
    g.bench_function("perform_descend", |b| {
        b.iter(|| {
            let mut chart = Chart::A;
            let dest = Chart::B { deep: true };
            let cmd = perform(&mut chart, Node::A, dest, Node::B1, &mut (), &mut ());
            black_box((chart, cmd))
        });
    });
    g.bench_function("perform_ascend", |b| {
        b.iter(|| {
            let mut chart = Chart::B { deep: true };
            let cmd = perform(&mut chart, Node::B1, Chart::A, Node::A, &mut (), &mut ());
            black_box((chart, cmd))
        });
    });

    g.finish();
}

fn rtc_drain(c: &mut Criterion<WallTime>) {
    let mut g = c.benchmark_group("rtc");
    g.throughput(Throughput::Elements(1));
    g.warm_up_time(Duration::from_millis(400));
    g.measurement_time(Duration::from_secs(2));
    g.sample_size(80);

    // One external message, no follow-ups — the simple-machine case
    // (`update -> Cmd`, no rtc). Compare to drain_8 / drain_31.
    g.bench_function("drain_1", |b| {
        b.iter(|| black_box(rtc::<u8, Cmd<u8>, _>(0, |m, _| Cmd::single(m)).unwrap()));
    });

    g.bench_function("drain_8", |b| {
        b.iter(|| {
            black_box(
                rtc::<u8, Cmd<u8>, _>(0, |m, inbox| {
                    if m + 1 < 8 {
                        inbox.push(m + 1);
                    }
                    Cmd::single(m)
                })
                .unwrap(),
            )
        });
    });

    // Default cap is 32. 31 follow-ups is a legal (if rude) step.
    g.bench_function("drain_31", |b| {
        b.iter(|| {
            black_box(
                rtc::<u8, Cmd<u8>, _>(0, |m, inbox| {
                    if m + 1 < 31 {
                        inbox.push(m + 1);
                    }
                    Cmd::single(m)
                })
                .unwrap(),
            )
        });
    });

    g.finish();
}

criterion_group!(benches, lca_and_perform, rtc_drain);
criterion_main!(benches);

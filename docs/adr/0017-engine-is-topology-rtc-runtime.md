---
tags: [engine, rtc, lca]
node_type: adr
---
# 0017 Engine is Topology RTC Runtime

## Status

Accepted

## Context

`0.0.0` shipped laws as types. Implementing the crate required a compiled RTC/LCA engine without an SCXML interpreter and without a proc-macro (see [[docs/adr/0009-no-scxml-interpreter]], [[docs/adr/0010-handwritten-expansion-before-macros]]).

The configuration is author-defined ADTs. The engine cannot walk a document. It can: (1) treat node ids as a tree, (2) run exit/install/enter in Harel order, (3) drain internal messages with a cap, (4) own the Elm triple.

## Decision

Three protocols, not one god object:

1. **TEA** — [`Machine`] + [`Runtime`]. `Self` is the configuration. `Model` is extended state. `History` is the sidecar. `init` returns [`Boot`]. `update` does not perform I/O.
2. **Kinematics** — [`Topology`] (`parent` of each node id) and [`Transitional`] (`exit` / `enter`). [`perform`] exits inner-first, assigns `dest`, enters outer-first. Cost O(depth).
3. **RTC** — [`rtc`] / [`Inbox`] drain follow-ups. Cap 32. [`Storm`] on a loop. One external `Msg` is one step from the caller.

Authors write enums and `match`. They call `perform` from `update`. A macro may later emit this expansion; the expansion remains readable Rust.

`MemoryStore` + `Runtime::persist` / `load` journal the phase space after the step. File backends stay in the host.

Rejected still: SCXML interpreter, thread-per-region, `Box<dyn State>`, I/O in `Drop`.

## Consequences

- `0.1.0` is the engine crate. Not SemVer-stable.
- Orthogonal regions remain fields; `perform` on a leaf uses LCA at the AND parent so `Online` is not exited (see [[docs/edge_cases/cross-region-lca]], `tests/connection.rs`).
- Internal-event storms fail closed with `Storm` instead of hanging ([[docs/edge_cases/internal-event-storms]]).
- `step` / `apply` now take history. Breaking vs 0.0.0, which was the design crate.

## Related

- symbol:perform
- symbol:Runtime
- symbol:rtc
- symbol:Topology
- symbol:Transitional
- [[docs/concepts/least-common-ancestor-transition]]
- [[docs/concepts/run-to-completion]]
- [[docs/plans/v0-crate-roadmap]]

---
tags: [and, composite, orthogonal]
node_type: adr
---
# 0021 And combinator is first-class Harel AND

## Status

Accepted

## Context

XOR illegal states are unrepresentable (`enum`). AND was only a handwritten `struct { auth, sync }` plus a flat `Topology::Node` namespace. `perform(Anonymous → Idle)` type-checked. A `NewtonError::CrossRegion` would be late and train `.unwrap`. Law 1: make it unrepresentable. GoF Composite, not an error enum. YAGNI for a Rete/visitor.

## Decision

- [`And<L, R>`](symbol:And) is two machines, one `Msg` (must be `Clone`), one RTC clock, document order **left then right**. Not threads. See [[docs/adr/0007-virtual-concurrency-not-threads]].
- Models and histories are **split** `(L::Model, R::Model)` and [`AndHistory`](symbol:AndHistory). Shared datamodel stays a handwritten struct with one `Model`.
- [`AndNode::Left`](symbol:AndNode) / `Right` so `in_state` cannot mix regions.
- `And` implements `try_update` (either region may Storm) and `update` via `unwrap_storm`.
- Cross-talk is `Msg` (and each region's own model), never assigning the sibling's `Self`.

## Consequences

- `And<And<A, B>, C>` nests.
- Handwritten `struct Session { auth, sync }` remains valid and is the right grain when both regions share one `ticks: u32`.
- Errors stay for *dynamic* faults (Storm, persist I/O, CmdOverflow). AND illegal is a type error at the combinator boundary.

## Related

- [[docs/concepts/and-node]]
- [[docs/edge_cases/cross-region-lca]]
- [[docs/adr/0002-xor-enums-and-and-structs]]

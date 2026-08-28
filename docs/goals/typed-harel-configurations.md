---
tags: [harel, adt, goals]
node_type: goal
---
# Typed Harel configurations

The live control state of a Newton machine is a **tree-shaped product**: every ancestor on the path, one child per XOR region, all children of an AND node. That set is encoded as nested Rust ADTs, not as a bag of string ids and not as a typestate lattice.

## Goals

- XOR = `enum`. AND = `struct`. Illegal simultaneous children do not compile. See [[docs/adr/0002-xor-enums-and-and-structs]].
- Keep the live configuration `Copy` or cheap `Clone`. Heavy data lives in [[docs/concepts/extended-state]].
- Pattern matching is dispatch. No `Box<dyn State>` (rejected: [[docs/adr/gof-state-trait-objects]]).
- Orthogonal regions are fields, not threads ([[docs/concepts/virtual-concurrency]]).
- Transition cost is O(depth + number of active regions), not O(number of states). No global string table on the hot path.

## Non-goals

- Representing "overlapping" as two parents sharing a child. Harel overlap is a configuration *set*, not a DAG of ownership. See [[docs/concepts/configuration-space]].
- Compiling every orthogonal leaf into a phantom-typed API ([[docs/adr/typestate-lattice]]).

## Related

- [[docs/concepts/xor-region]]
- [[docs/concepts/and-node]]
- [[docs/concepts/least-common-ancestor-transition]]
- tests/connection.rs

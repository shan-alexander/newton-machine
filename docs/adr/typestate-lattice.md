---
tags: [typestate, rejected]
node_type: alternative
---
# Typestate lattice

## Status

Rejected as the core model. Coarse façades remain allowed: [[docs/adr/0008-typestate-is-a-facade]].

## Context

Rust typestate (`Machine<Offline>`) makes illegal transitions a compile error. That is a real superpower for session-typed APIs and protocol clients.

## Why it was considered

- Zero runtime checks for illegal methods.
- Familiar to Rust engineers.
- Works beautifully for linear, shallow machines.

## Why it is not the interior of a Newton machine

- Orthogonal regions are a product. Phantom-typing every leaf is a lattice, not a tree.
- Deep history restore has to name a type that may not have been the last public phase.
- The Elm loop needs a single `Self` to put in a runtime and a snapshot.
- `Machine<S>` cannot be the thing you serialize without type-erasing `S`, which throws the lattice away at the moment you needed it.

Use typestate at the edges. Keep the interior as data.

## Related

- [[docs/concepts/typestate-facade]]
- [[docs/edge_cases/deep-history-type-explosion]]
- [[docs/adr/0002-xor-enums-and-and-structs]]

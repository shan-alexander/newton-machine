---
tags: [harel, adt]
node_type: adr
---
# 0002 XOR enums and AND structs

## Status

Accepted

## Context

Harel configurations are a tree-shaped product: one child per XOR, all children of AND. Runtime graphs of string ids (SCXML) are slow and un-Rusty. A full typestate lattice over orthogonal × history explodes. Statechart code generators for Rust already emit enums and structs; we make that the law.

## Decision

- **XOR region** = `enum`. Exactly one variant is the active child. Illegal simultaneous children do not compile.
- **AND node** = `struct`. Each field is an orthogonal region, all active at once.
- The live configuration *is* `Self` on `Machine`. Heavy data belongs in `Model` ([[docs/concepts/extended-state]]).
- Dispatch is `match`, not vtables. Use `mem::replace` / `mem::take` when rewriting a variant behind `&mut`.
- No string ids on the hot path. `NodeId` exists for `in_state` and diagnostics; prefer a compact enum.

Rejected alternatives: [[docs/adr/scxml-interpreter]], [[docs/adr/typestate-lattice]], [[docs/adr/gof-state-trait-objects]].

## Consequences

- Authors write ordinary Rust ADTs. The crate does not need a document object model.
- Orthogonality is free: a struct of enums is the configuration set.
- Deep hierarchy is nested types. That is readable until it is not; when it is not, split modules, do not introduce string ids.
- `Clone`/`Copy` discipline becomes part of the public contract. See [[docs/goals/typed-harel-configurations]].

## Related

- [[docs/concepts/xor-region]]
- [[docs/concepts/and-node]]
- [[docs/concepts/configuration-space]]
- symbol:Machine

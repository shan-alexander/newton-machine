---
tags: [typestate]
node_type: adr
---
# 0008 Typestate is a facade

## Status

Accepted

## Context

Typestate (`Machine<Offline>` vs `Machine<Online>`) can hide illegal methods at compile time. Applied to every orthogonal leaf and every history restore, the type lattice explodes and the Elm loop cannot even name `Self`.

## Decision

- Typestate is allowed **only** for coarse public phases at the crate's *edges* (methods that must not exist while offline).
- The interior stays a configuration tree so orthogonality and deep history remain data.
- `Machine` itself is not parameterized by a phantom state in 0.0.0.
- Do not typestate `History` restorations.

Rejected: [[docs/adr/typestate-lattice]].

## Consequences

- Callers who need "no `logout()` while offline" can wrap the machine. Core does not.
- `in_state` remains a runtime query for orthogonal leaves, which is honest: those leaves change without changing the public type.

## Related

- [[docs/concepts/typestate-facade]]
- [[docs/edge_cases/deep-history-type-explosion]]
- [[docs/adr/0002-xor-enums-and-and-structs]]

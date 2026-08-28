---
tags: [scxml, rejected]
node_type: alternative
---
# SCXML interpreter

## Status

Rejected for core. See [[docs/adr/0009-no-scxml-interpreter]].

## Context

W3C SCXML plus an interpreter is the standard way to "run Harel" with interchange. Several Rust crates already sit in this neighborhood.

## Why it was considered

- Domain experts already draw charts.
- Multi-language codegen and vendor tools (STATEMATE, itemis CREATE) speak SCXML.
- Runtime fidelity is a selling point if the artifact is a specification.

## Why it is not a Newton machine

- String ids on the hot path.
- Illegal XOR children fail in production, not at compile time.
- The API is a document, not Elm.
- Snapshots are interpreter heaps, often with callbacks.
- Interpreters are the opposite of "efficient and rusty" for an in-process library.

If interchange is ever required, compile a document into ADTs in a *separate* crate. Do not interpret.

## Related

- [[docs/goals/when-to-choose-newton-over-harel]]
- [[docs/concepts/unidirectional-configuration-architecture]]

---
tags: [selection, harel, goals]
node_type: goal
---
# When to choose Newton over Harel

Rule of thumb: if you would be embarrassed to serialize the machine after an event and restore it on another process, you wanted Harel-as-diagram or an actor soup. If that snapshot **is** the product, you wanted a Newton machine.

## Use a Harel chart when the primary artifact is a specification

- Domain experts must read a diagram.
- You need SCXML / STATEMATE / itemis CREATE interchange.
- Code will be generated into more than one language.
- The interesting problem is "what configurations exist and which arrows are legal."
- Runtime fidelity to W3C SCXML matters more than snapshot purity.

## Use a Newton machine when the primary artifact is a running Rust system

- The same `update` must serve live, replay, and test.
- You need a crash-consistent snapshot with no closures inside it.
- Orthogonal concerns must be concurrent in meaning and sequential in fact (one event, several regions, defined order).
- History must be cheap, optional per composite, and persistable without dragging sockets along.
- Other engineers should consume `init` / `update` / `view` / `subscriptions` / `in_state`.
- Invalid XOR children should fail to compile, not fail in production.
- I/O belongs to a host (broker, clock, disk), not to entry actions.

## Use neither

A three-variant enum and a `match` will do. Do not import this crate to feel architectural.

## Related

- [[docs/concepts/newtonian-state-machine]]
- [[docs/concepts/unidirectional-configuration-architecture]]
- [[docs/adr/0009-no-scxml-interpreter]]

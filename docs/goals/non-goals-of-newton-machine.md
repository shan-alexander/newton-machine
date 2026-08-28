---
tags: [non-goals, scope]
node_type: goal
---
# Non-goals of newton-machine

A crate is defined as much by what it refuses as by what it ships. These refusals are family law, not a backlog.

## Non-goals (will not ship in core)

- A generic "any SCXML document at runtime" interpreter. See [[docs/adr/0009-no-scxml-interpreter]] and [[docs/adr/scxml-interpreter]].
- `Box<dyn State>` / GoF State as the core model. See [[docs/adr/gof-state-trait-objects]].
- One OS thread per orthogonal region. See [[docs/adr/0007-virtual-concurrency-not-threads]].
- String state ids on the hot path.
- Closures in the snapshot (`Box<dyn Fn>` history is a persistence dead end).
- I/O inside `update`, entry actions, or `Drop` of a state.
- A typestate lattice over every orthogonal leaf. See [[docs/adr/typestate-lattice]] and [[docs/adr/0008-typestate-is-a-facade]].
- A broker session, FIX engine, or risk gateway. See [[docs/adr/0011-core-crate-is-not-a-broker]].
- Making a three-variant enum and a `match` "use Newton." UCA earns its keep when you have at least two of: hierarchy, orthogonality, history, replay, host-executed effects.

## Still in scope (later versions, not 0.0.0 claims)

- Compiled RTC + LCA transition engine.
- Optional proc-macro whose expansion remains readable Rust.
- Optional `newton-trading` companion with region vocabulary only.

## Related

- [[docs/goals/establish-the-newtonian-family]]
- [[docs/goals/when-to-choose-newton-over-harel]]

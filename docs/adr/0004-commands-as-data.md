---
tags: [cmd, effects]
node_type: adr
---
# 0004 Commands as data

## Status

Accepted

## Context

If entry actions call I/O, snapshots contain lies, tests need the network, and a panicked `impl Machine` can send FIX. Elm's `Cmd` already solved this: `update` returns a description; the runtime executes it and turns the result into another `Msg`.

## Decision

- `Cmd<C>` is an enum: `None`, `Single(C)`, `Batch(Vec<C>)` (batch requires `alloc`). symbol:Cmd
- `Sub<L>` is the same idea for ongoing listeners. symbol:Sub
- `update` never performs I/O. `Drop` of a state variant never performs I/O. See [[docs/edge_cases/i-o-smuggled-in-drop]].
- The host is the only type allowed to talk to a broker, disk, or clock.
- Commands collect during RTC and execute **after** the step finishes.

## Consequences

- A test host records `Cmd`s and never opens a socket. Live and replay stay one `update`.
- The crate cannot place an order even if misused. That is the family test. See [[docs/adr/0011-core-crate-is-not-a-broker]].
- Authors must design `C` as a serializable vocabulary (`Submit`, `Persist`, `Alert`), not as closures.

## Related

- [[docs/concepts/command-as-data]]
- [[docs/concepts/subscriptions]]
- [[docs/goals/effects-never-leave-the-host]]
- [[docs/adr/0012-intent-versus-authority]]

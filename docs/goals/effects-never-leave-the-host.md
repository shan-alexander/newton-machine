---
tags: [cmd, io, goals]
node_type: goal
---
# Effects never leave the host

`update` is a pure step. Entry, exit, and transition actions return [[docs/concepts/command-as-data]]. The host executes I/O and feeds facts back as `Msg`.

This is the conservation law that keeps snapshots honest and the family distinct from "Harel plus callbacks."

## Goals

- Nothing in `newton-machine` can place an order, open a socket, or write a file even if misused. If a panicked engineer can `impl Machine` and accidentally send FIX from an entry action, the family is broken. See [[docs/adr/0011-core-crate-is-not-a-broker]].
- Intent versus authority: the machine emits `Cmd::Submit`; a gateway admits or refuses. See [[docs/adr/0012-intent-versus-authority]] and [[docs/concepts/intent-and-authority]].
- Subscriptions are a function of the current configuration so timers exist only while relevant.
- The same `update` serves live, backtest, and unit test. A test host records commands and never opens a socket.

## Non-goals

- A built-in HTTP, broker, or async runtime.
- Executing commands inside `Drop` of a state variant. See [[docs/edge_cases/i-o-smuggled-in-drop]].
- Making the crate a risk gateway. A Newton machine may lock itself. Only the gateway may lock the wire. See [[docs/concepts/risk-layers]].

## Related

- symbol:Cmd
- symbol:Sub
- [[docs/edge_cases/silence-is-not-a-message]]

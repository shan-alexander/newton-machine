---
tags: [snapshot, serde]
node_type: adr
---
# 0006 Snapshot is the phase space

## Status

Accepted

## Context

Replay and crash restore need a classical point in phase space. Functions, sockets, and view closures are not restorable. Harel tools often snapshot an interpreter heap. Elm apps often forget hierarchy. The Newton machine's persistable triple is the product.

## Decision

- `Snapshot { config, context, history }` is the complete serializable state. symbol:Snapshot
- Command handlers, view functions, and host resources are **not** in the snapshot.
- Optional `serde` derives on these types when the `serde` feature is on.
- A journal keys snapshots by domain identity (e.g. `(symbol, bar_ts)`). The crate does not choose the key.
- Restore is "load snapshot, then host reconciles facts the snapshot cannot know."

## Consequences

- Live and backtest are one `update` applied to the same triple plus a sequence of `Msg`.
- If you cannot serialize after an event, you are not running a Newton machine.
- In-flight commands are not in the snapshot; that is an edge case. See [[docs/edge_cases/persist-versus-in-flight-commands]].

## Related

- [[docs/concepts/phase-space-snapshot]]
- [[docs/adr/0005-history-as-sidecar]]
- [[docs/edge_cases/panic-during-rtc]]

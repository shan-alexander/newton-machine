---
tags: [history, store]
node_type: adr
---
# 0015 Persist after RTC

## Status

Accepted

## Context

Persisting per micro-action (each exit, each orthogonal field) multiplies syscalls and can capture a torn configuration. Wall-clock wants one write after the step. Compute wants history writes only on exit of opted-in composites (in RAM), then one store save.

## Decision

- Hot path mutates RAM only.
- After RTC completes, the host may `HistoryStore::save` the snapshot.
- `HistoryStore::save` takes `&mut self`. File-backed stores may ignore the mut and write to disk.
- `MemoryStore` is the in-session default. Same snapshot type as a file store.
- Persist asynchronously if the host wants; the crate does not spawn tasks.
- Do not persist from entry/exit actions. Those return `Cmd::Persist` if the author wants a hint; the host still decides.

## Consequences

- A panic during RTC leaves the previous snapshot. See [[docs/edge_cases/panic-during-rtc]].
- Commands issued in the step may not have executed when the snapshot is saved. See [[docs/edge_cases/persist-versus-in-flight-commands]].

## Related

- symbol:HistoryStore
- symbol:MemoryStore
- [[docs/concepts/phase-space-snapshot]]
- [[docs/adr/0006-snapshot-is-the-phase-space]]

---
tags: [panic, snapshot]
node_type: edge_case
---
# Panic during RTC

A panic in `update` aborts the step. The previous snapshot is the last honest point. A torn in-RAM configuration must not be saved.

## Trap

Host `catch_unwind`s and persists the half-updated `&mut` machine. Journal now contains a configuration that `init` could never produce.

## Law

- Persist **after** a successful step.
- On panic, discard the in-RAM machine, reload the last snapshot, inject `Msg` that records the panic if you care, do not save the torn state.
- `update` should be written so panics are bugs, not control flow.

## Related

- [[docs/adr/0015-persist-after-rtc]]
- [[docs/adr/0006-snapshot-is-the-phase-space]]
- [[docs/concepts/phase-space-snapshot]]

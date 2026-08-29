---
tags: [runtime, purity, tea]
node_type: adr
---
# 0019 No host mut escape hatches on Runtime

## Status

Accepted

## Context

`Runtime::model_mut` / `history_mut` let the host change extended state and the sidecar **without a `Msg`**. That is a second door. Law 2: `update` is the only accelerator. Clone-to-please-the-borrow-checker is not an excuse: `update` already takes `&mut Model` and `&mut History`.

## Decision

- Remove `model_mut` and `history_mut`.
- Hosts inject facts as `Msg`. Tests send `Msg`. Restore uses [`Runtime::restore`](symbol:Runtime::restore) (replace the whole triple), then reconcile with a `Msg`.
- `model()` / `history()` stay shared borrows (CQS: queries do not mutate).

## Consequences

- More boilerplate (`Msg::SetClock`). That is the product: the journal contains the cause.
- `update(&mut self, &mut model, &mut history, msg)` is unchanged — that mutation is the step, not an escape hatch.

## Related

- [[docs/adr/0003-tea-is-the-only-mutation-protocol]]
- [[docs/concepts/unidirectional-configuration-architecture]]

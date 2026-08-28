---
tags: [history, inertia]
node_type: adr
---
# 0005 History as sidecar

## Status

Accepted

## Context

Harel shallow (H) and deep (H*) history are kinematics. If you store them *inside* live enums, every clone, persist, and `PartialEq` pays for ghosts, and resource-owning states cannot be snapshotted. Newton's first law wants inertia **beside** the live system.

## Decision

- History is a sidecar type (`Machine::History`, `()` if unused).
- Live configuration answers "where am I now?" History answers "where was this composite last time I left it?"
- **Shallow**: store a discriminant (often one byte). Re-enter that child, then that child's default descendant.
- **Deep**: store the subtree config, only when the subtree is `Copy` or trivially `Clone` (enums + small ids). If the subtree owns sockets or buffers, store a path of discriminants, never the resources.
- Opt-in per composite. Default re-entry is the declared initial child.
- History writes happen on exit of opted-in composites, not on every event.

## Consequences

- `HistoryKind::{None, Shallow, Deep}` documents intent. symbol:HistoryKind
- Resource-owning states are an edge case with a hard rule. See [[docs/edge_cases/history-of-resource-owning-states]].
- Restoring deep history of `Book`/`Scale` without restoring the broker socket is correct. Restoring `Order::Working` from history without reconcile is not. See [[docs/edge_cases/restore-without-reconcile]].

## Related

- [[docs/concepts/inertial-history]]
- [[docs/goals/inertial-persistable-history]]
- [[docs/adr/0015-persist-after-rtc]]

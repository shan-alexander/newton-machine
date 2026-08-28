---
tags: [history, snapshot, goals]
node_type: goal
---
# Inertial persistable history

A composite keeps its last configuration in a sidecar until a transition writes a new one. That is Newton's first law applied to control state. History is optional per composite, cheap, and serializable.

## Goals

- Do not keep history inside live enums. Active config answers "where am I now?"; history answers "where was this composite last time I left it?" See [[docs/adr/0005-history-as-sidecar]].
- Shallow history stores a discriminant. Deep history stores a small subtree, and only when that subtree is `Copy` or trivially `Clone`.
- Default re-entry is the declared initial child. Most nodes opt out.
- Persist `{config, context, history}` after RTC completes, never per micro-action. See [[docs/adr/0015-persist-after-rtc]] and [[docs/concepts/phase-space-snapshot]].
- A `HistoryStore` trait swaps in-memory for a file without changing `update`. symbol:HistoryStore

## Non-goals

- Storing sockets, tasks, or `Box<dyn Fn>` in history. See [[docs/edge_cases/history-of-resource-owning-states]].
- Deep history of every composite "just in case."
- Making history the live configuration so equality and clone pay for ghosts.

## Related

- [[docs/concepts/inertial-history]]
- [[docs/edge_cases/default-child-versus-history-miss]]
- [[docs/edge_cases/deep-history-type-explosion]]

---
tags: [history, newton]
node_type: concept
---
# Inertial history

Newton's first law: a composite keeps its last configuration in the sidecar until a transition writes a new one.

| Kind | Store | Re-enter |
| --- | --- | --- |
| None (default) | nothing | declared initial child |
| Shallow (H) | 1-byte discriminant | that child, then its default descendant |
| Deep (H*) | subtree config | that subtree, only if `Copy` / cheap `Clone` |

Opt-in per composite. Most nodes should have no history.

Do not keep history inside live enums. Mixing them makes every clone, persist, and equality check pay for ghosts.

Session vs process: same `History` type. In-memory store for the session; `HistoryStore` for disk. Persist after RTC, not per micro-action.

## Related

- [[docs/adr/0005-history-as-sidecar]]
- [[docs/adr/0015-persist-after-rtc]]
- [[docs/edge_cases/history-of-resource-owning-states]]
- [[docs/edge_cases/default-child-versus-history-miss]]
- symbol:HistoryKind

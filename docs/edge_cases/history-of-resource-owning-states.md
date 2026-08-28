---
tags: [history, drop]
node_type: edge_case
---
# History of resource-owning states

Deep history of a subtree that owns a socket, file, or task cannot be the resource itself. History stores discriminants and small snapshots, never host resources.

## Trap

`Online { stream: TcpStream }` stored in `History`. Snapshot is not serializable. Restore opens a dead fd. `Drop` of the historical copy closes the live stream.

## Law

- If the subtree owns resources, store a **path of discriminants**, never the resources.
- Resources live in the host. The machine holds ids in [[docs/concepts/extended-state]].
- `Drop` of a config variant must not close a socket. See [[docs/edge_cases/i-o-smuggled-in-drop]].

## Related

- [[docs/adr/0005-history-as-sidecar]]
- [[docs/concepts/inertial-history]]
- [[docs/concepts/phase-space-snapshot]]

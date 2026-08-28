---
tags: [lca, orthogonal]
node_type: edge_case
---
# Cross-region LCA

A transition whose source is in one orthogonal region and whose target is in another does not "move a field." The LCA is the AND parent (or above). Exit/enter must respect that or you will leave a sibling region in a ghost child.

## Trap

`Auth::Anonymous --Login--> Sync::Fetching`. Naive code overwrites the wrong field or both. `Session` becomes inconsistent with the type (if you used a flattened enum) or leaves `auth` stale (if you assigned `sync` only).

## Law

- Legal transitions stay inside a region, or they are transitions of a **parent** XOR that rebuilds the AND node.
- Cross-talk between regions is via `Model` (shared numbers) and via parent transitions, not via assigning a sibling field from a child `match`.
- Document order still applies: each region is offered the event; at most one parent transition should fire per step unless you explicitly compose.

## Related

- [[docs/concepts/least-common-ancestor-transition]]
- [[docs/concepts/and-node]]
- [[docs/adr/0007-virtual-concurrency-not-threads]]

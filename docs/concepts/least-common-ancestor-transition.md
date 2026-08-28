---
tags: [harel, lca]
node_type: concept
aliases: [LCA, lca]
---
# Least common ancestor transition

Do not search a global transition table by string id. For a compiled/hand-written tree:

1. Offer the event to the innermost active leaf of each orthogonal region.
2. If that leaf does not consume it, bubble to its XOR parent (`Super`).
3. When a transition fires, compute the **least common ancestor** of source and target in the tree.
4. Exit source → LCA (inner first). Run exit actions. Record history for every opted-in composite you leave.
5. Enter LCA → target (outer first), then default-child descent or history restore.
6. Collect `Cmd`s; do not execute them until the step finishes.

Cost: O(depth + number of active regions), not O(number of states).

`0.1.0` implements this as symbol:perform plus [[docs/concepts/outcome-vocabulary]] for bubbling. See [[docs/adr/0017-engine-is-topology-rtc-runtime]].

## Related

- [[docs/adr/0014-outcome-vocabulary]]
- [[docs/edge_cases/cross-region-lca]]
- [[docs/plans/v0-crate-roadmap]]

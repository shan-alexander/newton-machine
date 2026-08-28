---
tags: [orthogonal, harel]
node_type: concept
---
# Virtual concurrency

Harel "concurrency" for orthogonal regions is **virtual**: one event, several regions, defined order, one clock.

Real threads per region buy you races and buy you nothing for the formalism.

Document order = field order of the AND struct. Authors who need a different order document it; they do not spawn tasks.

If you later need many machines, the unit of parallelism is the **machine**, not the region.

## Related

- [[docs/adr/0007-virtual-concurrency-not-threads]]
- [[docs/concepts/and-node]]
- [[docs/concepts/run-to-completion]]

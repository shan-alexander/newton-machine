---
tags: [orthogonal, rtc]
node_type: adr
---
# 0007 Virtual concurrency not threads

## Status

Accepted

## Context

Harel orthogonal regions are "concurrent in meaning." Real threads per region buy races, `Arc<Mutex>`, and a second clock, and buy nothing for the formalism. The efficient encoding is already a struct of enums.

## Decision

- Orthogonal regions receive the **same** event, in a **fixed document order**, one after another, in the **same** RTC step.
- Document order is field order of the AND struct unless an author documents otherwise.
- The unit of parallelism, if any, is the **machine** (one symbol, one worker), not the region.
- No `Send` bound is required of `update` for orthogonality to work.

## Consequences

- Authors reason about a total order per step. Tests are deterministic.
- A `Bar` can update `Regime` while `Order` is `Working` and `Risk` is `Throttled` without locks.
- Cross-region transitions still go through LCA at the AND parent. See [[docs/edge_cases/cross-region-lca]].

## Related

- [[docs/concepts/virtual-concurrency]]
- [[docs/concepts/and-node]]
- [[docs/concepts/run-to-completion]]

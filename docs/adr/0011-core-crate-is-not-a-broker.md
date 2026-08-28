---
tags: [trading, boundary]
node_type: adr
---
# 0011 Core crate is not a broker

## Status

Accepted

## Context

The motivating domain is live algo trading on equity bars. That domain has brokers, watchdogs, and kill switches. If those leak into core, the family becomes a trading crate and the conservation law (no I/O in `update`) dies.

## Decision

Treat the crate as **laws of motion**, not as a broker.

**Inside `newton-machine`:** `Machine`, `step`/`apply`, RTC/LCA (later), history sidecar, `Cmd`/`Sub` as data, `in_state`, `Snapshot {config, context, history}`, `Outcome`.

**Inside an optional later `newton-trading` or examples, not core:** suggested regions (`Session`, `Risk`, `Order`, `Book`), suggested messages (`Killed`, `FeedStale`, `Resume`, `Fill`, `Reject`), suggested commands (`Submit`, `Cancel`, `CancelAll`, `Flatten`, `Persist`, `Alert`), a `RiskLimits` value type (numbers only). Still no sockets.

**Outside, in the host:** broker session, watchdog, firewall, account/locate/PDT, feed-staleness clock, portfolio aggregator, kill UI, persistence backend.

**Outside the application:** venue cancel-on-disconnect, broker kill, prime-broker caps.

Hard rule: nothing in this crate can place an order even if misused.

## Consequences

- README sentence stands: *A Newton machine may lock itself. Only the gateway may lock the wire. Only the venue can save you when both are dead.*
- Trading engineers copy region vocabulary; they do not get a matching engine from us.

## Related

- [[docs/adr/0012-intent-versus-authority]]
- [[docs/concepts/risk-layers]]
- [[docs/goals/effects-never-leave-the-host]]

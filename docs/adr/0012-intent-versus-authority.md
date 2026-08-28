---
tags: [gateway, risk]
node_type: adr
---
# 0012 Intent versus authority

## Status

Accepted

## Context

A Newton step is RTC. That is a virtue for replay and a liability for a hard kill: `update` does not run if the task is wedged, the bar feed is silent, or the process is dead. Knight-style losses are "the chart was not in the path," not "the chart chose the wrong child." After 2012, pre-trade controls must exist in the order path, independent of the algo that proposed the order.

## Decision

- The machine emits **intent**: `Cmd::Submit`, `Cmd::Cancel`, `Cmd::Flatten`.
- A host **gateway** admits or refuses that intent, and can actuate cancel/flatten without waiting for the next `update`.
- The machine is then told what the world did, via `Fill` / `Reject` / `Killed`.
- Actuate out of band. Reconcile in band. Never invert that.
- Do not give the machine a `Broker` trait to call.

See [[docs/concepts/risk-layers]] for L0–L5.

## Consequences

- L3 (the Risk region) is necessary and not sufficient. L3 without L2 is a diary of what you meant to do.
- Silence is handled at L2, not by waiting for L3 to notice. See [[docs/edge_cases/silence-is-not-a-message]].
- Backtests must run L2 against simulated account state.

## Related

- [[docs/concepts/intent-and-authority]]
- [[docs/adr/0011-core-crate-is-not-a-broker]]
- [[docs/edge_cases/duplicate-acks]]

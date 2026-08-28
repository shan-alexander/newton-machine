---
tags: [risk, feed]
node_type: edge_case
---
# Silence is not a message

A Risk region that only moves on messages cannot protect you from the **absence** of messages. First-law failure mode: a machine that does not see a force stays in `Armed`.

## Trap

Bar feed stalls. No `Msg::Bar` arrives. `update` does not run. L3 never transitions. Orders still go out if L2 also waits for the chart.

## Law

Stale-input fail-closed at **L2**, stale-policy at **L3**.

- No bar / no heartbeat / mark older than N seconds: L2 refuses risk-increasing orders even if the last configuration said `Armed`.
- Separately, the host injects `Msg::FeedStale` so L3 can move to `Throttled` or `Locked` for the journal.
- Do not wait for L3 to "notice" silence.

## Related

- [[docs/concepts/risk-layers]]
- [[docs/adr/0012-intent-versus-authority]]
- [[docs/concepts/run-to-completion]]
- [[docs/concepts/inertial-history]]

---
tags: [harel, rtc]
node_type: concept
aliases: [RTC, rtc]
---
# Run-to-completion

Harel RTC: an external event is fully processed — exits, transition, entries, collected actions — before the next external event is considered.

In a Newton machine:

- One external `Msg` = one RTC step from the caller's view.
- Internal follow-up messages may be drained inside `update` into a tiny `Vec<Msg>` before return. Still one wall-clock step.
- Commands collect; they execute **after** the step.
- Orthogonal regions share that step. They do not each get a thread.

RTC is why replay works. It is also why `update` cannot be the last line of defense: if no `Msg` arrives, no step runs. See [[docs/edge_cases/silence-is-not-a-message]] and [[docs/concepts/risk-layers]].

## Related

- [[docs/adr/0003-tea-is-the-only-mutation-protocol]]
- [[docs/concepts/least-common-ancestor-transition]]
- [[docs/edge_cases/internal-event-storms]]
- [[docs/edge_cases/re-entrancy-of-update]]

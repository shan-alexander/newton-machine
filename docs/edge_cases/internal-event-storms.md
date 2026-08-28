---
tags: [rtc, events]
node_type: edge_case
---
# Internal event storms

Internal follow-up messages drained inside `update` can loop (`A → B → A → …`) and never return. From the caller's view the step never ends.

## Trap

A guard that is always true fires `Transition` that immediately queues the same `Msg`. RTC does not complete. Persist never happens. The host looks wedged — which L1 may treat as a kill, correctly.

## Law

- Bound internal drains (small cap, e.g. 32). Exceeding the cap is a bug: return a `Cmd::Alert` or a typed error to the host, do not spin.
- Prefer `Internal(cmd)` / `Handled` over chaining messages for "do two things."
- `rtc` / `rtc_n` enforce the cap (`Storm`). Authors call them from `update`.

## Related

- [[docs/concepts/run-to-completion]]
- [[docs/plans/v0-crate-roadmap]]
- [[docs/adr/0014-outcome-vocabulary]]

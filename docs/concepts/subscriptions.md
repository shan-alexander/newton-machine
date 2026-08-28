---
tags: [elm, sub]
node_type: concept
---
# Subscriptions

`subscriptions(model)` is a pure function of the current configuration. The host diffs the new `Sub` against the last one and starts or stops listeners.

When a Newton machine is `Locked` or `Offline`, the host should see a smaller `Sub` and drop timers it no longer requested. Elm already got this right; we keep it.

`Sub` never holds closures. `L` is a listener vocabulary (`BarFeed`, `FillFeed`, `Clock`). The runtime maps listeners to `Msg`.

RAII/`Drop` may stop an activity when a state is exited, but the *decision* to start or stop still comes from `subscriptions` or a `Cmd` — not from a hidden side effect inside the variant.

symbol:Sub

## Related

- [[docs/adr/0004-commands-as-data]]
- [[docs/concepts/command-as-data]]
- [[docs/edge_cases/i-o-smuggled-in-drop]]

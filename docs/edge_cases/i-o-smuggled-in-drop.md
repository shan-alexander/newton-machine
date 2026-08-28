---
tags: [drop, effects]
node_type: edge_case
---
# I/O smuggled in Drop

`Drop` on a state variant looks like an exit action. It is not. Exit actions return `Cmd`. `Drop` runs on panic, on history clones, on snapshot copies, and on abandoned tests.

## Trap

`impl Drop for Online { fn drop(&mut self) { broker.cancel_all(); } }`. Cloning the config for a snapshot cancels live orders. History deep-copy cancels live orders. A test assertion clone cancels live orders.

## Law

- `Drop` must not perform I/O and must not talk to a broker.
- RAII may drop *host-owned* activities when the **host** decides a subscription ended, not when a config value is copied.
- The decision to start/stop still comes from `subscriptions(model)` or a `Cmd`.

## Related

- [[docs/adr/0004-commands-as-data]]
- [[docs/concepts/subscriptions]]
- [[docs/edge_cases/history-of-resource-owning-states]]
- [[docs/goals/effects-never-leave-the-host]]

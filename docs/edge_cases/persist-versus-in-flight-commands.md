---
tags: [persist, cmd]
node_type: edge_case
---
# Persist versus in-flight commands

The snapshot does not contain unexecuted `Cmd`s. Saving immediately after `update` returns, before the host runs `Submit`, means restore will not know a live order may exist.

## Trap

Crash after `Cmd::Submit` is returned and before the wire write. Restore thinks `Order::Idle`. Broker has (or does not have) the order. Either way the snapshot lied about the world, which is allowed — the world is not in the snapshot — but the host must reconcile.

## Law

- Snapshot is `{config, context, history}`, not `{…, pending_cmds}`.
- Host reconcile is mandatory after restore: drop-copy, working-order query, then `Msg`s to catch L3 up.
- Optionally the host journals the command log *beside* the snapshot. That log is not core's type.

## Related

- [[docs/edge_cases/restore-without-reconcile]]
- [[docs/adr/0006-snapshot-is-the-phase-space]]
- [[docs/concepts/command-as-data]]

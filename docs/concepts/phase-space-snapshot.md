---
tags: [snapshot, persist]
node_type: concept
---
# Phase-space snapshot

`{config, context, history}` is a complete classical point. Replay is the same trajectory because the snapshot contains no functions.

- **config** — live XOR/AND tree. "Where am I now?"
- **context** — [[docs/concepts/extended-state]]. Numbers and ids.
- **history** — [[docs/concepts/inertial-history]]. "Where was I when I left?"

Command handlers, view functions, and sockets are not in it. Restore on another process, then let the host reconcile facts the snapshot cannot know (broker working orders, account).

symbol:Snapshot

## Related

- [[docs/adr/0006-snapshot-is-the-phase-space]]
- [[docs/adr/0015-persist-after-rtc]]
- [[docs/edge_cases/restore-without-reconcile]]
- [[docs/edge_cases/persist-versus-in-flight-commands]]

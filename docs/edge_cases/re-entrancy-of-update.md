---
tags: [rtc, reentrancy]
node_type: edge_case
---
# Re-entrancy of update

`update` must not call the host in a way that feeds another `Msg` before the current step finishes. That would nest RTC inside RTC and tear the configuration.

## Trap

Entry action executes HTTP synchronously; the response callback calls `apply` on the same machine. Orthogonal fields are half-updated. History records the wrong exit.

## Law

- `update` returns `Cmd`. The host executes after return.
- Internal follow-ups are a `Vec<Msg>` **drained inside** the same call, not a recursive `apply` from the host.
- Hosts must queue inbound messages while a step is running.

## Related

- [[docs/concepts/run-to-completion]]
- [[docs/adr/0003-tea-is-the-only-mutation-protocol]]
- [[docs/adr/0004-commands-as-data]]

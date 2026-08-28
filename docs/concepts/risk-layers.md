---
tags: [trading, gateway]
node_type: concept
---
# Risk layers

A Newton machine is allowed to decide that risk is exhausted. It is not allowed to be the only component that can stop orders from reaching a venue.

```text
human / compliance          L5  policy, lists, manual kill
portfolio supervisor        L4  firm / book / correlation
Newton Risk region          L3  modal policy (Armed / Throttled / Locked)
pre-trade firewall          L2  admit this order? (always-on arithmetic)
watchdog / DMS              L1  is the process alive?
venue / broker primitive    L0  cancel-on-disconnect, cancel-all-after
                │
                ▼
              venue
```

| Layer | Where | Job |
| --- | --- | --- |
| L0 | Venue / broker | Runs when your process does not |
| L1 | Sibling process or (weaker) dedicated task | Heartbeat or kill; must not call `update` |
| L2 | Host gateway on the `Cmd` path | Admit this envelope? Fail-closed on stale marks |
| L3 | Inside the machine | Modal policy; emits `Cmd`; does not send them |
| L4 | Parent machine or service | Cross-symbol caps; not stuffed into each child |
| L5 | Humans / compliance | Must flip L1/L2 without compiling a new chart |

Severity ladder (who may declare):

| Severity | Effect | Who |
| --- | --- | --- |
| Throttle | shrink size / rate | L3, L4 |
| Closing only | admits reduces, refuses increases | L2, L3, L5 |
| Cancel working | pull resting orders | L0, L1, L2 |
| Flatten | market out of position | L1, L2, L5 (L3 may request) |
| Lock | no strategy-originated orders until explicit resume | L2 latches; L3 records |

L3 may *request* flatten. L1/L2 may *perform* flatten and then tell L3.

L3 is necessary and not sufficient. L3 without L2 is a diary of what you meant to do. L2 without L3 cannot express "armed but only scale-downs in Protect."

None of L0–L2, L4, L5 belong in `newton-machine`. L3's *vocabulary* may later live in `newton-trading`. Core only supplies the laws.

## Related

- [[docs/adr/0012-intent-versus-authority]]
- [[docs/concepts/intent-and-authority]]
- [[docs/edge_cases/silence-is-not-a-message]]
- [[docs/adr/0011-core-crate-is-not-a-broker]]

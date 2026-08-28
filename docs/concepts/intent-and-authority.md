---
tags: [gateway, risk]
node_type: concept
---
# Intent and authority

Split:

```text
update  →  Cmd::Submit { .. }     // intent
gateway →  Wire::NewOrder { .. }  // authority
        or Reject { reason }
        or Drop                    // lock already on
```

The machine is allowed to **decide** that risk is exhausted. It is not allowed to be the only component that can stop orders from reaching a venue. Those fail in different ways.

Actuate out of band (gateway/watchdog/venue). Reconcile in band (`Msg::Killed`, `Fill`, `Reject`). Never invert that.

Do not give the machine a `Broker` trait to call. That collapses the firewall into the chart and makes snapshots dishonest again.

## Related

- [[docs/adr/0012-intent-versus-authority]]
- [[docs/concepts/risk-layers]]
- [[docs/adr/0011-core-crate-is-not-a-broker]]

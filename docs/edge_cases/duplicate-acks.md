---
tags: [idempotency, fills]
node_type: edge_case
---
# Duplicate acks

Kill paths race the machine's own `Cmd::Cancel`. Venues retransmit. `CancelAll` and `Flatten` must be safe to fire twice.

## Trap

Second `CancelAck` transitions `Order` out of `Idle` into a nonsense state, or double-counts a flatten in context.

## Law

- Gateway dedupes by `client_id` / venue id.
- The machine treats duplicate acks as `Handled`.
- Remediation commands are idempotent in both L2 and L3.

## Related

- [[docs/adr/0012-intent-versus-authority]]
- [[docs/concepts/outcome-vocabulary]]
- [[docs/concepts/risk-layers]]

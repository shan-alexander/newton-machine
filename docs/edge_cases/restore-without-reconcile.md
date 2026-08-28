---
tags: [snapshot, broker]
node_type: edge_case
---
# Restore without reconcile

Loading `{config, context, history}` and continuing to trade without asking the broker what is actually working is how a restored `Armed` + `Book::Long` double-submits.

## Trap

Process dies after a fill but before persist, or after persist but the fill never reached context. Restore disagrees with the venue.

## Law

- Snapshot restore is necessary and not sufficient.
- Host reconcile (drop-copy / working orders / account) produces `Msg`s. Then `update` catches L3 up. Then, and only then, L2 may unlatch.
- `Order` should not use history. Working orders are facts in context plus live broker state, not ghosts in the chart.

## Related

- [[docs/concepts/phase-space-snapshot]]
- [[docs/adr/0012-intent-versus-authority]]
- [[docs/edge_cases/persist-versus-in-flight-commands]]
- [[docs/concepts/inertial-history]]

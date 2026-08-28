---
tags: [history, typestate]
node_type: edge_case
---
# Deep history type explosion

If you try to typestate every orthogonal leaf *and* remember deep history, the type space is the product of every region times every historical child. That is the lattice we refused.

## Trap

`Machine<Online<SignedIn, Dirty, Working, Armed>>` plus a phantom for last-regime. Every transition is a different type. `step` cannot be a function. Serde cannot name the enum.

## Law

- Interior is data. History is a sidecar of discriminants / small snapshots.
- Typestate only as a coarse façade. See [[docs/adr/0008-typestate-is-a-facade]].

## Related

- [[docs/adr/typestate-lattice]]
- [[docs/concepts/typestate-facade]]
- [[docs/concepts/inertial-history]]

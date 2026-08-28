---
tags: [typestate]
node_type: concept
---
# Typestate facade

Typestate at the **edges**: public methods that only exist in some coarse phases (`Machine<Offline>` cannot `logout()`).

Typestate as the **interior**: forbidden. Orthogonal × deep history is a lattice you cannot name, serialize, or put in an Elm loop.

The interior is a configuration tree (data). The façade may wrap it. 0.0.0 does not even parameterize `Machine` by a phantom state.

## Related

- [[docs/adr/0008-typestate-is-a-facade]]
- [[docs/adr/typestate-lattice]]
- [[docs/edge_cases/deep-history-type-explosion]]

---
tags: [policy, host, bits, chord, fleet]
node_type: adr
---
# 0023 Chord table is host policy

## Status

Accepted

## Context

A pool of simultaneous flags `{A,B,C,D}` can have authored *chords* (sleeves) `{A,B}`, `{A,C,D}`, `{A,B,D}`. That is a **policy table**, not a Harel parent. Putting every conjunction in the chart is the mega-enum [[docs/adr/0002-xor-enums-and-and-structs]] refused. Loading the table from YAML *as topology* is the SCXML/ADR-0012 hole: knobs may be YAML; the chart may not.

A Quant desk already has the exact-key form (`HashMap<bitset, sleeve>`, unauthored → none). They will not accept `perform()` on every score flicker, nor a rustc change per new YAML sleeve.

The missing generic pieces were: project the typed config to a compact key; look up a table without making the table a node; many symbols without `And` of 50 desks; apply only on XOR category change.

## Decision

- [`Bits`](symbol:Bits) is a `u128` projection of the live configuration. [`Machine::project`](symbol:Machine::project) is optional; default empty. Orthogonal XOR children occupy disjoint bits. Not a string intern table.
- [`ChordTable`](symbol:ChordTable) lives in the **host**. `update` does not call it.
  - [`MatchMode::Exact`](symbol:MatchMode): pool equals row. Unauthored → miss. (Quant sleeve map.)
  - [`MatchMode::LongestSubset`](symbol:MatchMode): longest authored subset of the pool. `{A,B,C,D}` with `{A,B}` and `{A,B,D}` selects `{A,B,D}`.
  - Same length (and same `priority`): [`Tie::Refuse`](symbol:Tie) by default (specification hole → [`Hit::Tie`](symbol:Hit), not a silent winner). [`Tie::AuthorOrder`](symbol:Tie) for YAML-list order. Explicit `priority` beats author order.
- [`Fleet<K, M>`](symbol:Fleet): `BTreeMap` of runtimes. Fifty symbols are fifty machines, not one AND tree.
- [`changed`](symbol:changed) / [`Runtime::apply_if`](symbol:Runtime::apply_if): host classifies; Newton steps only when an XOR child moves. Sticky/arm aging stays in the host (or `Model`), not in the history sidecar.

Refused: YAML charts, “sleeve” as a Newton node type, subset match inside `update`, SCXML, `And` of the universe.

## Consequences

- Truth (nested ADTs) and policy (table) can stack without merging.
- A desk that already owns `HashMap<u128, Sleeve>` keeps it; they can feed `rt.project().raw()` as the key. They do not have to use [`ChordTable`](symbol:ChordTable).
- Longest-subset is available for authors who want “named superstate on a chord, ignore extra atoms.” Exact remains the fail-closed sleeve product.
- Same-length different combinations are **not** resolved by Hamming weight (they tied) and **not** by lexicographic bits (arbitrary). Refuse or author order / priority.

## Related

- [[docs/concepts/chord-and-superstate]]
- [[docs/concepts/configuration-versus-policy]]
- [[docs/adr/0002-xor-enums-and-and-structs]]
- [[docs/adr/0009-no-scxml-interpreter]]
- [[docs/adr/0011-core-crate-is-not-a-broker]]
- symbol:Bits
- symbol:ChordTable
- symbol:Fleet

The **policy kernel** (exact ROM, hysteretic scores, Fold) is `newtonian-core`, not this crate. `ChordTable` longest-subset is a host helper. Kernel law is exact key. See sibling `docs/adr/0016-newtonian-core-is-the-policy-kernel` in `newton-core`.

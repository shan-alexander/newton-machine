---
tags: [policy, bits, sleeve, superstate, host]
node_type: concept
aliases: [sleeve, superstate, chord]
---
# Chord and superstate

A **pool** is the set of flags that are true together (`{A,B,C,D}`). A **chord** (sleeve, overlay) is an *authored* subset the host has named a behaviour for (`{A,B}`, `{A,C,D}`, `{A,B,D}`). A **superstate** in this sense is not a Harel parent: it is the winning chord, recomputed from the pool. The chart still names XOR/AND *truth*. The table names *what to do*.

```text
typed config  --project-->  Bits (u128)  --lookup-->  Hit { sleeve | miss | tie }
     Newton                     host key                  host policy
```

[`Machine::update`](symbol:Machine::update) never sees the table.

## Longest subset

Pool `{A,B}` with a row `{A,B}` → that row.

Pool `{A,B,C,D}` with rows `{A,B}` (`N=2`) and `{A,B,D}` (`N=3`) → `{A,B,D}`. Extra atoms (`C`) do not invent a new sleeve; they also do not block a more specific authored chord.

If the table *also* has `{A,C,D}` (`N=3`), pool `{A,B,C,D}` is a **tie** at length 3 (`ABD` vs `ACD`). Longest-match does not pick between them. That is the same-length race.

Unauthored pool under **exact** match → miss (Quant). Unauthored extra atoms under **longest subset** are ignored if some row is still a subset.

## Same-length race

`{A,B}` vs `{A,C}` against pool `{A,B,C}`: both length 2. Hamming weight cannot pick. Lexicographic bit order would pick arbitrarily.

Newton default: **[`Tie::Refuse`](symbol:Tie)** — return [`Hit::Tie`](symbol:Hit), force the author to add priority, a more specific row, or accept [`Tie::AuthorOrder`](symbol:Tie) (first inserted, like a YAML list). Priority is an `i16` on the row; higher wins among equal `N`.

This is not “match ordering in `update`.” Ordering is a **table** property.

## What is not a chord

- Two XOR children of one region (`Offline` and `Online`) — illegal; the type forbids it.
- `Desk { quad, ema }` — that *is* the pool as an AND of XOR regions. `QuadOversold ∩ Split` is a value of the product, not a third parent. Policy may *label* that value (`OversoldSuperLift`) without promoting it into the enum.
- Sticky / `{tf,bars}` arm — host `Model` or score clocks. Harel history is “restore last child on re-enter,” not “stay in-play N bars while raw is false.”
- Fifty symbols — [`Fleet`](symbol:Fleet), not `And` of 50 desks.

## Category change (lift)

Classify in the host. [`changed`](symbol:changed) / [`Runtime::apply_if`](symbol:Runtime::apply_if) skip `apply` when the XOR child did not move. A 5s pulse is not a fire.

## Related

- [[docs/adr/0023-chord-table-is-host-policy]]
- [[docs/concepts/configuration-versus-policy]]
- [[docs/concepts/and-node]]
- symbol:Bits
- symbol:ChordTable
- symbol:Fleet

Family: a **policy kernel** (exact sleeve ROM, gated scores) belongs in `newtonian-core`. This crate projects truth to `Bits`. Longest-subset here is opt-in host policy, not kernel law.

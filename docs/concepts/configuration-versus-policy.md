---
tags: [policy, score, host, uca]
node_type: concept
aliases: [behavioral engine, scoring engine]
---
# Configuration versus policy

A Newton machine names **what is true**. A host policy names **what to do**.

`Desk { quad, ema }` is an AND of orthogonal facts. `QuadOversold ∩ AboveTwoOrMore` is a **product** of those facts, not a third parent in the tree. Promoting every interesting conjunction into a chart node is the mega-enum we refused in [[docs/adr/0002-xor-enums-and-and-structs]].

## Layers

| Layer | Owns | Example |
| --- | --- | --- |
| Configuration (Newton) | typed chart + sidecar | `QuadOversold`, `AboveTwoOrMore` |
| Policy / score (host) | `(config, model) -> label, score, Cmd` | `OversoldLift`, `score = +3` |
| Gateway (host, not crate) | admit/refuse `Cmd::Submit` | L2 firewall |
| Venue | the wire | COD, fill |

`view()` may *display* an overlap. `update` must not flatten overlaps into XOR children. `Cmd` from the machine is journal/alert intent unless the host policy emits order intent — and even then the gateway is the authority. See [[docs/concepts/intent-and-authority]].

## When a parent machine *is* warranted

A second Newton machine whose XOR is `Idle | LongBias | ShortBias | Locked`, and whose **context** is the child desk (or a score), is modal *policy*. It still does not place orders. Do not merge it into the indicator AND node.

A “behavioral engine” that maps many simultaneous long/short *features* through a scoring model is a **function** (or that parent policy machine), not `newton-machine` core.

When the host has a *table* of named conjunctions (sleeves), project the config with [`Machine::project`](symbol:Machine::project) / [`Bits`](symbol:Bits) and look up [`ChordTable`](symbol:ChordTable) **after** `apply`. Exact key = fail-closed sleeve map. Longest subset = “named superstate on a chord.” Neither is a chart node. See [[docs/concepts/chord-and-superstate]].

`prev` copied from `rt.machine()` before `apply` is host scratch (edge detection). It is **not** the sidecar. The sidecar is `rt.history()`, written on opted-in `exit`. `rt.snapshot()` clones `{config, context, history}` after the step.

## Related

- [[docs/adr/0012-intent-versus-authority]]
- [[docs/concepts/and-node]]
- examples/aapl_1m.rs

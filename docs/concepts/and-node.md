---
tags: [harel, struct, orthogonal]
node_type: concept
---
# AND node

An AND node (orthogonal composite) has **all** children active at once. In a Newton machine that is a `struct`. Each field is typically an XOR region.

```rust
struct Session {
    auth: Auth, // XOR
    sync: Sync, // XOR; orthogonal to auth
}
```

No threads, no `Arc<Mutex>`, no parallel runtime. See [[docs/concepts/virtual-concurrency]].

Two grains:

| Grain | When |
| --- | --- |
| Handwritten `struct { auth, sync }` | Shared `Model` (one `ticks`) |
| `And<L, R>` | Regions are already machines; models/histories split; `AndNode` tags `in_state` |

`And` offers the same `Msg` left-then-right on one RTC clock. A region cannot assign the sibling's configuration. See [[docs/adr/0021-and-combinator]]. symbol:And

Fifty symbols are **not** an AND node. Use [`Fleet`](symbol:Fleet). Named conjunctions of flags (sleeves) are a host [[docs/concepts/chord-and-superstate]], not extra XOR children.

A single `Msg` is offered to each field in document order (field order) during one [[docs/concepts/run-to-completion]] step. That is how a bar can update regime while an order is working and risk is already throttled, without flattening five dimensions into one mega-enum.

## Related

- [[docs/concepts/xor-region]]
- [[docs/adr/0007-virtual-concurrency-not-threads]]
- [[docs/edge_cases/cross-region-lca]]

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

A single `Msg` is offered to each field in document order (field order) during one [[docs/concepts/run-to-completion]] step. That is how a bar can update regime while an order is working and risk is already throttled, without flattening five dimensions into one mega-enum.

## Related

- [[docs/concepts/xor-region]]
- [[docs/adr/0007-virtual-concurrency-not-threads]]
- [[docs/edge_cases/cross-region-lca]]

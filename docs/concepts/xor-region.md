---
tags: [harel, enum]
node_type: concept
---
# XOR region

An XOR (exclusive-or) region has **exactly one** active child. In a Newton machine that is an `enum`.

```rust
enum Conn {
    Offline,
    Connecting { attempt: u8 },
    Online(Session),
}
```

Pattern matching is dispatch. Events that the leaf does not consume bubble to the parent via [[docs/concepts/outcome-vocabulary]] `Super`.

Default child is the declared initial variant (`Offline` above). History, if opted in, may override that on re-entry. See [[docs/concepts/inertial-history]].

## Related

- [[docs/concepts/and-node]]
- [[docs/concepts/configuration-space]]
- [[docs/adr/0002-xor-enums-and-and-structs]]

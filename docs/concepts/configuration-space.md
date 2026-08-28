---
tags: [harel, adt]
node_type: concept
---
# Configuration space

A Harel configuration is a **set** of simultaneously active states: every ancestor on the path, plus one child per XOR region, plus all children of an AND node. That set is a tree-shaped product, not a DAG of overlapping ownership.

"Overlapping" in this family does **not** mean two parents sharing a child. It means the machine occupies several nodes at once because of hierarchy and orthogonality.

In a Newton machine the configuration space **is** the type:

- Path through XOR nodes → nested `enum` variants.
- AND node → `struct` whose fields are the active regions.

The type is the active configuration. You cannot be `Offline` and `Online` at once. You can be `Online` with `Auth::SignedIn` *and* `Sync::Dirty` at once, because those are fields of `Session`.

Keep the live configuration `Copy` or cheap `Clone`. Put heavy data in [[docs/concepts/extended-state]].

## Related

- [[docs/concepts/xor-region]]
- [[docs/concepts/and-node]]
- [[docs/adr/0002-xor-enums-and-and-structs]]
- [[docs/goals/typed-harel-configurations]]

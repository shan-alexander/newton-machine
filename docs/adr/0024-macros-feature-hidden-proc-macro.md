---
tags: [macros, api, 0.2.0]
node_type: adr
aliases: [macros, derive Machine, derive Topology]
---
# 0024 Macros feature: hidden proc-macro crate

## Status

Accepted

Supersedes the *timing* of [[docs/adr/0010-handwritten-expansion-before-macros]] (the engine is now real). Does **not** supersede: expansion must remain valid Rust; no runtime `add_state("foo")`; no SCXML interpreter.

## Context

`0.1.1` is the handwritten engine. Authors still duplicate `Node` vs chart enum, `Topology::parent`, `in_state` (including ancestors / Root), and `perform(self, from, dest, to, ctx, hist)` node ids. That tax does not encode UCA. `update` matching on `Msg` **does** — a DSL that replaces the enum would hide the configuration type.

`hsmc` / `hsmc-macros` show demand for less typing (`statechart!`). Their `during:` activities are async I/O **in the chart**. Their v0.1 explicitly omits orthogonal regions and history. Copying that DSL would train authors away from ADTs and violate law 2.

Rust cannot export a proc-macro from a normal `rlib`. A second *product* crate is sprawl. A second *package* that users never `cargo add` is not.

QuantSys (paused) will `cargo add newton-machine` as a battle host. Macros must generate the same kinematics that host will debug: `cargo expand` looks like `examples/connection.rs`.

## Decision

### Grain

| Package | crates.io | User-facing |
| --- | --- | --- |
| `newton-machine` | yes | Engine. Feature `macros` re-exports proc-macros. |
| `newton-machine-macros` | yes (required so the feature resolves) | **Implementation detail.** Not in the README install story. |

```toml
newton-machine = { version = "0.2", features = ["macros"] }
```

Declarative `macro_rules!` (`perform!`) live in `newton-machine` (no extra crate). Proc-macros (`#[derive(Topology)]`, `#[derive(IntoNode)]`, `#[machine]`) live in the hidden package.

### What we generate (valuable)

1. **`#[derive(Topology)]`** on the **node-id enum**. Exactly one `#[topology(root)]`. Other variants `#[topology(parent = Variant)]`. Compile error on missing parent, two roots, or a cycle. Impl `Topology for TheEnum { type Node = Self; }`.
2. **`#[derive(IntoNode)]`** on the **chart enum**. `#[into_node(NodeId)]`. Maps variant names to `NodeId` variants (`Connecting { .. }` → `Node::Connecting`). Impl `IntoNode`.
3. **`#[machine(...)]`** on **`impl Chart { init; update; view; … }`**. Rewrites into `impl Machine for Chart` with associated types, ancestor-walk `in_state` / `configuration` via `IntoNode` + `Topology`, and `impl Topology for Chart` forwarding to `NodeId` (unless `no_topology`). Extra inherent methods stay on `impl Chart`.
4. **`perform!(chart, dest, ctx, hist)`** — `macro_rules!` using `IntoNode::node`. Deletes the repeated from/to node arguments.

`in_state` is **ancestor walk** from `node()`, not a generated mega-`match`. Root is in the configuration because it is on every path. Same law as handwritten connection.

### What we refuse

- `statechart! { state Idle { on(Go) => Busy } }` as the source of truth.
- `during:` / `async` in `update` / I/O in generated entry.
- String ids, `add_state("foo")`.
- Absorbing YAML / chord tables into the macro (host / policy kernel).
- Typestate lattice as the interior.
- `unsafe` in expansion.
- Inherent `init` forwarding from a `#[derive(Machine)]` on the type (trait/inherent recursion trap). The **impl-block attribute** *is* the Machine derive.

### Errors

`syn::Error` spanned on the user’s token. Messages state the law (“XOR/node cycle: Online → …”, “two `#[topology(root)]` variants”). `trybuild` may follow; v0.2.0 prefers compile_error with stable wording.

### Tests

Handwritten `tests/connection.rs` remains the spec. Macro tests: a small XOR using derives + `perform!`, same `apply` behaviour. Expansion must be copy-pasteable.

## Consequences

- 0.2.0 is additive if authors ignore `macros`; new public items: `IntoNode`, `perform!`, feature `macros`.
- Publish **macros package first**, then `newton-machine` 0.2.0.
- Handwritten charts stay first-class. Macros are optional sugar.

## Related

- [[docs/adr/0010-handwritten-expansion-before-macros]]
- [[docs/adr/0009-no-scxml-interpreter]]
- [[docs/adr/0002-xor-enums-and-and-structs]]
- symbol:Machine
- symbol:Topology
- symbol:perform
- symbol:IntoNode

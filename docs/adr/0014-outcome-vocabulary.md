---
tags: [outcome, dispatch]
node_type: adr
---
# 0014 Outcome vocabulary

## Status

Accepted

## Context

statig's `Super` is the right bubbling idea but is tied to its runtime. A Newton `update` is ordinary Rust. Regions need a small enum so parents can be offered the same message without a global table.

## Decision

```text
enum Outcome<S, C> {
    Handled,
    Super,                      // defer to parent XOR
    Transition { to: S, cmd: C },
    Internal(C),                // stay; run action only
}
```

- Offer the event to the innermost active leaf of each orthogonal region.
- If that leaf does not consume it, bubble (`Super`) to its XOR parent.
- When a transition fires, compute LCA of source and target (engine in 0.1.0; handwritten in 0.0.0).
- Exit source → LCA (inner first). Enter LCA → target (outer first). Collect cmds; do not execute until the step finishes.

symbol:Outcome

## Consequences

- 0.0.0 exposes the enum so authors can write `match` by hand (`tests/connection.rs`).
- The future engine consumes this vocabulary; it does not replace it with strings.

## Related

- [[docs/concepts/outcome-vocabulary]]
- [[docs/concepts/least-common-ancestor-transition]]
- [[docs/adr/0003-tea-is-the-only-mutation-protocol]]

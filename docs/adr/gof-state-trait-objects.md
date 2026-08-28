---
tags: [gof, rejected]
node_type: alternative
---
# GoF State trait objects

## Status

Rejected. See [[docs/adr/0002-xor-enums-and-and-structs]].

## Context

The Gang of Four State pattern puts behavior in `Box<dyn State>` (or a vtable of `handle(event)`). Many OO Harel ports do this.

## Why it was considered

- Open for extension: add a state without touching a match.
- Familiar textbook shape.

## Why it is refused

- Fights ownership: who owns the next state, the context, the history?
- Hides the configuration. You cannot `match` a product of orthogonal regions.
- History snapshots become `Box<dyn Any>` or worse, `Box<dyn Fn>`.
- Illegal configurations are representable (`Offline` and `Online` both boxed somewhere).
- Dispatch is a vtable hop instead of an enum match the compiler already optimizes.

Prefer `enum` + `match`. Composition, not inheritance.

## Related

- [[docs/goals/typed-harel-configurations]]
- [[docs/concepts/configuration-space]]

---
tags: [cmd, alloc, no-std]
node_type: adr
---
# 0018 Cmd stacks four atoms then heaps

## Status

Accepted

Amends [[docs/adr/0004-commands-as-data]] and [[docs/adr/0013-no-std-alloc-and-serde]].

## Context

Without `alloc`, `Cmd::and` of two `Single`s **dropped** the right-hand intent. That violates the conservation law (every effect is returned). `perform` concatenates exit + enter; two commands is the common case. Silent drop is a footgun the compiler cannot see.

GoF Command is data, not `Box<dyn Fn>`. Heap is for *many* atoms, not for the existence of a second atom.

## Decision

- `INLINE_CAP` = 4. Stack storage always. symbol:Cmd symbol:INLINE_CAP
- `and` that would exceed 4 **spills to `Vec`** when `alloc` is on, and **panics** with `CmdOverflow` when it is not. Panic is programmer error (too many effects on a no-heap build), like `Vec` index. symbol:CmdOverflow
- `Cmd::try_and` is the fallible form (serde, bulk builders).
- Representation is private. Compare and iterate atoms. Do not match `None` / `Single` / `Batch`.
- Serde is a sequence of atoms, not the stack/heap layout.

## Consequences

- `perform` of exit+enter never needs a heap.
- No silent drop. Tests: four fit; five panics without `alloc`, heaps with it.
- `Sub::Many` still needs `alloc` (listeners are not the same conservation law).

## Related

- [[docs/concepts/command-as-data]]
- [[docs/goals/effects-never-leave-the-host]]

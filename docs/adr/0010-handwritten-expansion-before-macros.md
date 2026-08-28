---
tags: [macros, api]
node_type: adr
---
# 0010 Handwritten expansion before macros

## Status

Accepted

## Context

A proc-macro that expands a chart DSL into XOR/AND ADTs plus `update` is attractive. A runtime builder is not. Macros that cannot be debugged without expansion are a tax. 0.0.0 has no engine yet, so a macro would freeze a DSL around vapor.

## Decision

- 0.0.0 ships handwritten types and a handwritten example (`tests/connection.rs`).
- A proc-macro, if any, lands **after** the RTC/LCA engine is real and the expanded form is a readable crate someone can copy.
- The expanded form must remain valid Rust so people can debug without the macro.
- A runtime builder (`add_state("foo")`) is refused for all versions of core.

## Consequences

- Authors of 0.0.0 write enums and a `match`. That is the teaching API.
- We will not publish `newton-machine-macros` until the expansion is something we would merge as a PR.

## Related

- [[docs/goals/elm-shaped-public-api]]
- [[docs/plans/v0-crate-roadmap]]
- [[docs/adr/0009-no-scxml-interpreter]]

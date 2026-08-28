---
tags: [api, tea, goals]
node_type: goal
---
# Elm-shaped public API

Other engineers should consume a Newton machine the way they consume Elm: one model, one message type, one `update`, a pure `view`, and subscriptions that exist only while needed.

## Goals

- Hide LCA, bubbling, and history writes behind `update`. Authors of a *machine* may see [[docs/concepts/outcome-vocabulary]]; users of a machine must not.
- Provide two entry points with **one semantics**: `step` (owned, Elm-pure) and `apply` (`&mut`, hot path). See [[docs/adr/0003-tea-is-the-only-mutation-protocol]].
- Keep `Cmd` and `Sub` as data ([[docs/concepts/command-as-data]], [[docs/concepts/subscriptions]]).
- Expose `in_state` / configuration queries for tests and operator consoles without string ids on the hot path.
- Remain readable without a proc-macro. Macros, if any, come after the handwritten expansion is a good crate. See [[docs/adr/0010-handwritten-expansion-before-macros]].

## Non-goals

- A runtime builder (`add_state("foo")`).
- Callbacks, observers, or `Box<dyn Fn>` in the public loop.
- Making `view` mean HTML. The associated `View` type is a projection; TUI, JSON, and "nothing" are all valid.

## Related

- symbol:Machine
- symbol:apply
- symbol:step
- [[docs/concepts/newtonian-state-machine]]

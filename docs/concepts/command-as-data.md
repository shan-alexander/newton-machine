---
tags: [cmd, elm]
node_type: concept
---
# Command as data

Elm `Cmd` = GoF Command without the OO ceremony. `update` returns a description of work. The host executes it and turns the result into another `Msg`.

That is also Newton's third law: every applied force has an equal-and-opposite reaction *returned*, never a hidden call.

`Cmd<C>` is an ordered bag of atoms (`C` = `HttpConnect`, `Submit`, `Persist`). Up to four live on the stack so `perform` never needs a heap; more spill to `Vec` with `alloc` or panic without. Representation is private — iterate, do not match variants. Nothing in this type can open a socket. See [[docs/adr/0018-cmd-inline-then-heap]].

symbol:Cmd

## Related

- [[docs/adr/0004-commands-as-data]]
- [[docs/concepts/subscriptions]]
- [[docs/concepts/intent-and-authority]]
- [[docs/goals/effects-never-leave-the-host]]

---
tags: [cmd, elm]
node_type: concept
---
# Command as data

Elm `Cmd` = GoF Command without the OO ceremony. `update` returns a description of work. The host executes it and turns the result into another `Msg`.

That is also Newton's third law: every applied force has an equal-and-opposite reaction *returned*, never a hidden call.

`Cmd<C>`: `None` | `Single(C)` | `Batch(Vec<C>)`. `C` is the author's vocabulary (`HttpConnect`, `Submit`, `Persist`). Nothing in this type can open a socket.

symbol:Cmd

## Related

- [[docs/adr/0004-commands-as-data]]
- [[docs/concepts/subscriptions]]
- [[docs/concepts/intent-and-authority]]
- [[docs/goals/effects-never-leave-the-host]]

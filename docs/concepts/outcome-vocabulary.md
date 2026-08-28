---
tags: [outcome, dispatch]
node_type: concept
---
# Outcome vocabulary

Per-region result of offering a `Msg`:

| Variant | Meaning |
| --- | --- |
| `Handled` | Consumed, stayed, no command |
| `Super` | Defer to parent XOR |
| `Transition { to, cmd }` | Leave this region for `to` |
| `Internal(cmd)` | Stay; run an action only |

This is statig's Super idea, made an explicit enum so `update` stays ordinary Rust. Guards stay `if`s on `&Context`.

symbol:Outcome

## Related

- [[docs/adr/0014-outcome-vocabulary]]
- [[docs/concepts/run-to-completion]]
- [[docs/concepts/least-common-ancestor-transition]]

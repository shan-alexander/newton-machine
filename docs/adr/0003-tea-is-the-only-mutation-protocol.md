---
tags: [tea, update]
node_type: adr
---
# 0003 TEA is the only mutation protocol

## Status

Accepted

## Context

Harel does not specify how a running process mutates. Callbacks, actor messages, and "the chart calls HTTP in an entry action" all exist in the wild. Elm's TEA is the mutation protocol that keeps `update` a function, tests deterministic, and snapshots serializable.

## Decision

- The only door is TEA: `Msg` in; new configuration + model + `Cmd` out.
- Public trait: `init`, `update`, `view`, `subscriptions`, `in_state`. symbol:Machine
- Two entry points, **one semantics**:
  - `apply(&mut machine, &mut model, msg) -> Cmd` — hot path.
  - `step(machine, model, msg) -> (machine, model, Cmd)` — Elm-pure; replace + apply.
- Do not invent a second meaning for the two functions.
- Guards are ordinary Rust `if`s on `&Model`, not a mini-language.
- One external `Msg` is one [[docs/concepts/run-to-completion]] step. Internal follow-ups may drain inside `update`; the caller still sees one step.

## Consequences

- Hosts (tests, brokers, TUI runtimes) all look the same: feed messages, execute commands.
- Time-travel and crash restore are the same function as live.
- Re-entrancy of `update` is forbidden. See [[docs/edge_cases/re-entrancy-of-update]].

## Related

- [[docs/goals/elm-shaped-public-api]]
- [[docs/adr/0004-commands-as-data]]
- symbol:apply
- symbol:step

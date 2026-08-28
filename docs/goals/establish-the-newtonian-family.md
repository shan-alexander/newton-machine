---
tags: [family, uca, goals]
node_type: goal
aliases: [family, UCA family]
---
# Establish the Newtonian family

Publish a named architectural family for Rust: the **Newtonian state machine**, whose articulate name is [[docs/concepts/unidirectional-configuration-architecture]].

Harel already defined the kinematics: configurations, XOR/AND decomposition, history, run-to-completion. Elm already defined a mutation protocol: `Msg → (Model, Cmd)`. This crate names the **conjunction** — four laws Harel never required and most Harel tools violate.

## Goals

- Name the family honestly: an architectural family, not a new computational class. If you strip the laws you are back at Harel; if you strip hierarchy you are back at TEA.
- Encode the four laws in types that other engineers can consume without an interpreter:
  1. The configuration is the type ([[docs/concepts/xor-region]], [[docs/concepts/and-node]]).
  2. The only mutation protocol is TEA ([[docs/adr/0003-tea-is-the-only-mutation-protocol]]).
  3. Memory is inertial and external ([[docs/concepts/inertial-history]]).
  4. Typestate is a façade ([[docs/concepts/typestate-facade]]).
- Make live, replay, and test share one `update` ([[docs/concepts/phase-space-snapshot]]).
- Keep the public API Elm-shaped so authors call `init` / `update` / `view` / `subscriptions` / `in_state`, not an SCXML document API. See [[docs/goals/elm-shaped-public-api]].
- Ship first as `newton-machine` `0.0.0` on crates.io: laws as types, no stability claim. See [[docs/goals/publish-an-honest-0-0-0-crate]] and [[docs/adr/0001-crate-identity-and-versioning]].

## Non-goals

- Inventing a new automaton that "Harel did not cover."
- Claiming the Rust ecosystem has never seen XOR enums. The novelty is the **closed set of constraints**, not any one encoding.
- A visual chart editor, SCXML interchange, or STATEMATE fidelity. See [[docs/goals/non-goals-of-newton-machine]].

## Related

- [[docs/concepts/newtonian-state-machine]]
- [[docs/goals/when-to-choose-newton-over-harel]]
- [[docs/plans/v0-crate-roadmap]]
- symbol:Machine

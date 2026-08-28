---
tags: [family, newton]
node_type: concept
aliases: [Newton machine, newton-machine, Newtonian]
---
# Newtonian state machine

Family name: **Newtonian state machine** (short: **Newton machine**).

"Newtonian" is not decoration. It is the metaphor that distinguishes the family from Harel.

| Newton | In the machine |
| --- | --- |
| Configuration space | The ADT tree is the whole control state |
| First law (inertia) | A composite keeps its last configuration in the sidecar until a transition writes a new one |
| Second law | A `Msg` is an applied force; `update` is the only accelerator |
| Third law | Every effect is an equal-and-opposite `Cmd` returned to the host, never a hidden call |
| Classical snapshot | `{config, context, history}` is a complete phase-space point; replay is the same trajectory |
| Absolute frame | One run-to-completion clock per machine; regions are logically concurrent, not threaded |

Harel drew the orbits. A Newton machine states the **laws of motion** so those orbits are reproducible, measurable, and restorable.

Avoid "Newtonian statechart" if you want the distinction to stay sharp. **Chart** = Harel's visual kinematics. **Machine** = UCA dynamics. The family is machines.

Crate: `newton-machine`. See [[docs/adr/0001-crate-identity-and-versioning]].

## Related

- [[docs/concepts/unidirectional-configuration-architecture]]
- [[docs/goals/when-to-choose-newton-over-harel]]
- [[docs/concepts/inertial-history]]
- [[docs/concepts/intent-and-authority]]

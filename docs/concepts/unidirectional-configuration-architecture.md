---
tags: [uca, family]
node_type: concept
aliases: [UCA, uca]
---
# Unidirectional Configuration Architecture

**Unidirectional Configuration Architecture (UCA)** is the articulate name of the family.

A UCA machine is one whose control state is a typed configuration in a [[docs/concepts/configuration-space]], whose applied forces are messages, whose reactions are commands, and whose memory of prior configurations is inertial — stored beside the live system, not inside it.

## Four laws

1. **The configuration is the type.** XOR is an enum, AND is a struct. Illegal simultaneous children are unrepresentable. No string ids on the hot path.
2. **The only mutation protocol is TEA.** `Msg → (config, context, Cmd)`. Entry/exit/transition actions emit data. The host executes I/O. `update` is a pure step.
3. **Memory is inertial and external.** History is a sidecar of discriminants and small snapshots, not live variants and not closures. The persistable phase space is exactly `{config, context, history}`.
4. **Typestate is a façade, not the machine.** Public phase types may hide illegal methods. The interior stays a configuration tree so orthogonality and deep history do not explode into a type lattice.

Those four laws are why a trading journal, a crash snapshot, and a unit test can share one function. A Harel chart in STATEMATE or SCXML does not give you that. An Elm counter does not give you orthogonal regions. `statig` does not give you a persistable sidecar. That conjunction is the family.

## What it is not

Not a new computational class. If you strip the laws, you are back at Harel. If you strip hierarchy, you are back at TEA. Honesty matters, because overclaiming "never invented anywhere" would make the name look like branding. In the Rust ecosystem, as a prescribed machine with those laws, it has not been named or standardized. That is enough.

## Related

- [[docs/concepts/newtonian-state-machine]]
- [[docs/goals/establish-the-newtonian-family]]
- symbol:Machine

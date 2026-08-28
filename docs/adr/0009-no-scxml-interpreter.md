---
tags: [scxml, scope]
node_type: adr
---
# 0009 No SCXML interpreter

## Status

Accepted

## Context

W3C SCXML is the interchange format for Harel tools. Interpreters give fidelity and multi-language codegen. They also throw away the type encoding, put string ids on the hot path, and produce the wrong API for app engineers (`add_state("foo")`).

## Decision

- `newton-machine` will not parse, interpret, or emit SCXML in core.
- If interchange is ever needed, it is a separate crate that *compiles* a document into ADTs, never a runtime interpreter.
- Steal the *shape* of elm-statecharts (messages into `update`), not the interpreter.

Rejected: [[docs/adr/scxml-interpreter]].

## Consequences

- Domain experts who must have a diagram use a Harel tool and hand-translate, or wait for optional codegen.
- This is the selection criterion in [[docs/goals/when-to-choose-newton-over-harel]].

## Related

- [[docs/adr/0010-handwritten-expansion-before-macros]]
- [[docs/goals/non-goals-of-newton-machine]]

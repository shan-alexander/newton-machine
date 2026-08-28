---
tags: [context, datamodel]
node_type: concept
aliases: [Context, datamodel]
---
# Extended state

Harel "datamodel." Elm's model-that-is-not-the-chart. In a Newton machine this is `Machine::Model`.

Put here: last N bars or their sufficient statistics, position qty and average price, pending order ids, daily loss, high-water mark, size limits, last error. Numbers, ids, small buffers.

Do **not** put here: sockets, broker sessions, or the live XOR/AND tree. The tree is `Self`. Resources belong to the host.

Keeping `Model` off the enums is what makes the configuration cheap to clone and journal.

## Related

- [[docs/concepts/phase-space-snapshot]]
- [[docs/concepts/configuration-space]]
- [[docs/adr/0002-xor-enums-and-and-structs]]

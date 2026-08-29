---
tags: [sub, elm, host]
node_type: adr
---
# 0022 Sub::diff is the host's start/stop

## Status

Accepted

## Context

Elm `subscriptions` is a pure bag. The *host* diffs consecutive bags and starts/stops listeners. Without [`Sub::diff`](symbol:Sub) every author writes a buggy set-difference. That is the first thing a live program misses.

## Decision

- `old.diff(&new) -> Diff { start, stop }` using [`PartialEq`] on the listener vocabulary (not closures, not pointer identity).
- Duplicates treated as a set.
- More than one start or stop atom at once needs `alloc` (same cap as `Sub::and`).
- The crate still does not own sockets. Diff is data.

## Related

- [[docs/concepts/subscriptions]]
- [[docs/adr/0004-commands-as-data]]
- symbol:Diff

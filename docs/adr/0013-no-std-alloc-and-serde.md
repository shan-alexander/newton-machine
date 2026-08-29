---
tags: [no-std, features]
node_type: adr
---
# 0013 no_std alloc and serde

## Status

Accepted

## Context

The core should run in embedded and in a trading process. `Vec` for `Cmd::Batch` needs `alloc`. Serde is required for journals and forbidden as a hard dependency for `no_std` binaries that do not persist.

## Decision

- `#![no_std]`. `alloc` is a feature (on by default via `std`).
- Default feature set: `std` → `alloc`.
- `serde` is optional, default-off, `default-features = false` + `derive`. See [[docs/references/crates/serde]].
- `Cmd` always stacks up to 4 atoms. Heap batch and `Sub::Many` need `alloc`. Without `alloc`, `Cmd::and` of 5+ **panics** (no silent drop). See [[docs/adr/0018-cmd-inline-then-heap]].
- No `unsafe`.

## Consequences

- Embedded hosts can emit up to four atoms per step without a heap.
- Journals enable `serde`.
- docs.rs builds with `all-features`.

## Related

- [[docs/goals/publish-an-honest-0-0-0-crate]]
- [[docs/adr/0016-dual-license-msrv-edition]]
- symbol:Cmd

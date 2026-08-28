---
tags: [license, msrv]
node_type: adr
---
# 0016 Dual license MSRV edition

## Status

Accepted

## Context

Rust library crates expected on crates.io are dual-licensed MIT OR Apache-2.0. An MSRV lets docs.rs and CI pin a toolchain. Edition 2021 is the conservative choice at MSRV 1.80.

## Decision

- License: **MIT OR Apache-2.0**.
- Edition: **2021**.
- MSRV: **1.80** (same floor as rustbrain's documented MSRV; we do not need nightly).
- `unsafe` is forbidden in this crate.
- Copyright holder on license files: "Newton Machine Contributors."

## Consequences

- Downstream Apache-only and MIT-only shops can both depend on us.
- We will not bump MSRV in a patch of `0.1.x` without a note in CHANGELOG.
- Edition 2024 is deferred until MSRV policy says so.

## Related

- [[docs/goals/publish-an-honest-0-0-0-crate]]
- [[docs/adr/0013-no-std-alloc-and-serde]]

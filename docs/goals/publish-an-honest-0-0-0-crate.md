---
tags: [crates-io, versioning, goals]
node_type: goal
---
# Publish an honest 0.0.0 crate

`newton-machine` `0.0.0` is the design crate for crates.io and docs.rs. It encodes UCA as types, compiles, and documents the family. It does **not** claim a finished RTC/LCA engine or a stable API.

## Goals

- Crate name `newton-machine` (hyphen). `newton` is taken on crates.io by a 2017 physics simulator; we do not squat or confuse it. See [[docs/adr/0001-crate-identity-and-versioning]].
- Version `0.0.0` means: public types exist, SemVer is not in force, names may change before `0.1.0`.
- README states the four laws, the Elm-shaped API, the crate boundary, and the sentence: *A Newton machine may lock itself. Only the gateway may lock the wire. Only the venue can save you when both are dead.*
- rustdoc is the docs.rs home page. The rustbrain graph under `docs/` is the in-repo second brain, not a substitute for rustdoc.
- Dual license MIT OR Apache-2.0, MSRV 1.80, edition 2021. See [[docs/adr/0016-dual-license-msrv-edition]].
- Optional `serde` and `no_std`+`alloc` features from day one of the type surface. See [[docs/adr/0013-no-std-alloc-and-serde]].

## Non-goals

- Publishing `0.0.0` as if it were production-ready trading infrastructure.
- A `newton-trading` crate in this version. Vocabulary may be documented; sockets will not ship.
- Stability promises, `#[non_exhaustive]` as a fig leaf, or "1.0-quality" macros.

## Related

- [[docs/plans/v0-crate-roadmap]]
- [[changelog]]
- [[docs/goals/establish-the-newtonian-family]]

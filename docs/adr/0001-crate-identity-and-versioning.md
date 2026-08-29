---
tags: [crates-io, versioning]
node_type: adr
---
# 0001 Crate identity and versioning

## Status

Accepted

## Context

The family needs a crates.io name. `newton` is already published (2017, a Newtonian physics simulator). `newton-core` would hide the family name from search. The workspace directory is `newton-machine`. Version `0.0.0` is the honest label for "laws as types, engine not claimed."

## Decision

- Publish the core crate as **`newton-machine`**.
- `0.1.0` is the engine (RTC/LCA/`Runtime`). The API is **not** SemVer-stable until a real downstream consumer has survived a minor bump. Do not claim `1.0`.
- Do not publish `newton` or try to take over the physics-simulator crate.
- An optional later crate `newton-trading` may hold equity-bar vocabulary. It is not this crate.
- docs.rs will render crate-level rustdoc. The rustbrain graph under `docs/` is the in-repo brain; it is not a second product.

## Consequences

- Search on crates.io for "newton machine" / "statechart" / "elm" should hit this crate.
- Downstream must pin exact versions while we are in `0.x`.
- README and rustdoc must say `0.1.0` is the engine and is **not** SemVer-stable.
- Set `repository` in `Cargo.toml` only after a GitHub remote exists.

## Related

- [[docs/goals/publish-an-honest-0-0-0-crate]]
- [[docs/plans/v0-crate-roadmap]]
- [[docs/adr/0011-core-crate-is-not-a-broker]]

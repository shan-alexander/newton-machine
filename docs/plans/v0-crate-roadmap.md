---
tags: [roadmap, 0.1.0]
node_type: plan
status: in_progress
---
# v0 crate roadmap

## Status

in_progress

## Intent

Land `newton-machine` as a publishable family crate: laws as types at `0.0.0`, compiled RTC/LCA at `0.1.0`, trading companion never in core.

Unlocks [[docs/goals/establish-the-newtonian-family]] and [[docs/goals/publish-an-honest-0-0-0-crate]].

## Backlog

- [ ] Optional proc-macro whose expansion remains readable. See [[docs/adr/0010-handwritten-expansion-before-macros]].
- [ ] Optional `newton-trading` companion (vocabulary only, no sockets). See [[docs/adr/0011-core-crate-is-not-a-broker]].
- [ ] File-backed `HistoryStore` example in the host, not in core.
- [ ] First crates.io publish when the owner chooses. See [[docs/adr/0001-crate-identity-and-versioning]].

## In Progress

- [/] SemVer caution: 0.1.x may still break before 0.2

## QA

- [x] `cargo test` (default, `--no-default-features`, `alloc`, `serde`)
- [x] `cargo clippy --all-targets -- -D warnings`
- [x] `cargo run --example connection`

## Done

- [x] Crate identity `newton-machine`
- [x] Dual license MIT OR Apache-2.0
- [x] README family sentence and four laws
- [x] Founding goals, ADRs, concepts, edge cases in rustbrain
- [x] Compiled RTC + LCA engine (`0.1.0`). See [[docs/adr/0017-engine-is-topology-rtc-runtime]]
- [x] Internal-event drain cap. See [[docs/edge_cases/internal-event-storms]]

## Cancelled

- [~] SCXML interpreter in core
- [~] Thread-per-region runtime
- [~] GoF `Box<dyn State>` core

## Blocked

- [!] crates.io publish credentials / GitHub remote (host concern, not a crate law)

## Priority / order

1. Use handwritten machines (`tests/connection.rs`) until duplication forces a macro.
2. Macro only after the engine's expansion is copy-pasteable Rust (it is, as of 0.1.0).
3. Trading companion never in core.

## Out of scope

- Broker, gateway, watchdog, FIX.
- Visual chart editor.
- Claiming `1.0`.

## Related

- [[docs/goals/non-goals-of-newton-machine]]
- [[changelog]]
- [[docs/adr/0001-crate-identity-and-versioning]]
- [[docs/adr/0017-engine-is-topology-rtc-runtime]]
- [[docs/concepts/newtonian-state-machine]]

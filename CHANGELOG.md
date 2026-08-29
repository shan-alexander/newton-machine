# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Fixed

- Ancestor walks deeper than `MAX_DEPTH` now panic instead of silently computing a wrong LCA in release builds.
- `Tape::last` is available without `alloc` (the tape previously recorded into an unreadable field).

### Added

- `Sub::{len, is_empty}`; `Cmd::Iter` is `ExactSizeIterator`; `Hash` on `Storm` / `HistoryKind`; `Debug`/`Clone` on `Inbox`; `PartialEq` on `Chain`.
- CI: rustdoc `-D warnings`, serde without `std`, MSRV 1.80 lib tests.
- `Cargo.toml` `repository` / `homepage` → `shan-alexander/newton-machine`.
- `And<L, R>` first-class Harel AND (document order, split model/history, `AndNode`).
- `Machine::try_update`, `Runtime::try_apply`, `unwrap_storm`. Storm is panic-by-default.
- `Cmd::try_and` / `CmdOverflow`. Stack cap 4; heap with `alloc`.
- `Sub::diff` / `Diff { start, stop }` (Elm host start/stop).
- Criterion wall-clock benches (`encodings`, `apply`, `harel`, `compose`). GitHub-only. `encodings` includes a handwritten-fields floor vs `Runtime` vs mega-enum vs HashSet ids vs `Box<dyn>`.
- `Bits` configuration projection (`u128`); `Machine::project` / `Runtime::project`.
- `ChordTable` host sleeve lookup: exact or longest-subset, `Tie::Refuse` or `AuthorOrder` (`alloc`).
- `Fleet<K, M>`: N runtimes, one `Msg` vocabulary (`alloc`).
- `changed` / `Runtime::apply_if`: category-change (lift) gate.

### Changed

- `examples/aapl_1m` HOST incremental EMA + stochastic (const packs / `push` state). No `finance-solution` dev-dependency.
- `Cmd` representation is private. `and` never silently drops atoms.
- `orthogonal` example uses `And<AuthM, SyncM>`.
- Connection chart uses `try_update` + `unwrap_storm` (no `.expect` on Storm).

### Removed

- `Runtime::model_mut` / `history_mut`. Inject facts as `Msg`.

## [0.1.0] - 2026-08-28

### Added

- Harel kinematics: `Topology`, `lca` / `paths` / `exit_path` / `enter_path`, `Transitional`, `perform`.
- Run-to-completion: `Inbox`, `rtc` / `rtc_n`, `Storm` (drain cap 32).
- `Runtime` owning `{config, context, history}`: `boot`, `apply`, `snapshot`, `restore`, `persist`, `load`.
- `Boot` from `Machine::init`; `update` takes `&mut History`.
- `Combine` for concatenating commands; `Cmd` / `Sub` iterators; `Tape` host recorder.
- History helpers: `record_shallow` / `record_deep` / `restore_shallow` / `restore_deep`.
- `Outcome::{or_else, into_parts, map_target, map_cmd}`.
- GitHub `examples/` (counter, connection, orthogonal, replay, storm). Not in the crates.io package.
- Engine-backed `tests/connection.rs`.
- `Boot::new` constructor.
- ADR 0017: engine is Topology + RTC + Runtime, not an interpreter.

### Changed

- Breaking relative to 0.0.0: `init` returns `Boot`; `update` / `step` / `apply` take history.

## [0.0.0] - 2026-08-27

### Added

- Initial design crate. Family laws as types (`Machine`, `Cmd`, `Sub`, `Outcome`, `Snapshot`, `HistoryStore`).
- Architecture notes under `docs/` (goals, ADRs, concepts, edge cases).

# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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

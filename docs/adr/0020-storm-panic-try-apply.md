---
tags: [rtc, storm, error]
node_type: adr
---
# 0020 Storm panics by default; try_apply is the opt-in

## Status

Accepted

## Context

An internal-event storm is a chart bug (always-true guard), not I/O. `Vec[i]` panics; `slice.get` is the opt-in. If `Machine::update` stays `-> Cmd` and swallows `rtc`'s `Result`, `Runtime` cannot see [`Storm`](symbol:Storm). Making every `apply` a `Result` taxes machines that never queue follow-ups.

## Decision

- Simple machines: `update -> Cmd`, no `rtc`. [`try_update`](symbol:Machine::try_update) default is `Ok(self.update(...))`.
- Machines that use `rtc` **override** `try_update` and implement `update` as [`unwrap_storm`](symbol:unwrap_storm)`(self.try_update(...))`.
- [`Runtime::apply`](symbol:Runtime::apply) does **not** wrap `rtc`. It calls `update`.
- [`Runtime::try_apply`](symbol:Runtime::try_apply) calls `try_update`. Hosts that must Halt use this. If the machine did not override `try_update`, a panicking `update` still panics here.
- Do not `.expect("no follow-ups")` as the documented happy path.
- Do not make every `apply` a `Result` in 0.1.x.

## Consequences

- Storm `Display` names the bug and points at `try_apply`.
- A machine that maps Storm to `Cmd::none()` is still possible if it neither panics nor overrides `try_update` — that is an author lie, not an engine lie.

## Related

- [[docs/concepts/run-to-completion]]
- [[docs/edge_cases/internal-event-storms]]
- [[docs/adr/0003-tea-is-the-only-mutation-protocol]]

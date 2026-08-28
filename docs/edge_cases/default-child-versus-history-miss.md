---
tags: [history]
node_type: edge_case
---
# Default child versus history miss

Opted-in shallow history with `None` in the sidecar (first entry, or history cleared) must fall back to the declared initial child. Treating `None` as a bug produces a panic on the first `Online` entry.

## Trap

`last_auth.unwrap()` in `enter_session`. First login panics. Or worse: `unwrap_or(SignedIn)` invents a user id.

## Law

- Missing history = default child. Always.
- Shallow history of `SignedIn` restores the discriminant, not the user id. The user id lives in `Model` or in the `Msg` that caused re-entry (`Authed(user)`).
- Clearing history (explicit `Resume` policy after halt) is allowed; it is not an error.

## Related

- [[docs/concepts/inertial-history]]
- [[docs/adr/0005-history-as-sidecar]]
- tests/connection.rs

//! Phase-space point of a Newton machine: `{config, context, history}`.
//!
//! That triple is the complete serializable state. Command handlers, view
//! functions, and sockets are not in it. Replay is the same trajectory because
//! the snapshot is classical, not a bag of closures.
//!
// rustbrain: [[docs/concepts/phase-space-snapshot]] [[docs/adr/0006-snapshot-is-the-phase-space]]

/// A restorable point in the machine's phase space.
///
/// - `C` is the configuration tree (XOR enums / AND structs).
/// - `X` is extended state (Harel datamodel / Elm model-that-is-not-the-chart).
/// - `H` is the history sidecar (discriminants and small snapshots).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Snapshot<C, X, H> {
    /// Live configuration: answers "where am I now?"
    pub config: C,
    /// Extended state: numbers, ids, buffers that are not control state.
    pub context: X,
    /// Inertial memory: answers "where was this composite last time I left it?"
    pub history: H,
}

impl<C, X, H> Snapshot<C, X, H> {
    /// Build a snapshot from its three parts.
    #[inline]
    pub const fn new(config: C, context: X, history: H) -> Self {
        Self {
            config,
            context,
            history,
        }
    }
}

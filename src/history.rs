//! History as a sidecar, not as live configuration.
//!
//! Do not keep history inside the active enums. Mixing them makes every clone,
//! persist, and equality check pay for ghosts. Persist after a run-to-completion
//! step completes, not per micro-action.
//!
// rustbrain: [[docs/concepts/inertial-history]]
// rustbrain: [[docs/adr/0005-history-as-sidecar]]
// rustbrain: [[docs/adr/0015-persist-after-rtc]]
// rustbrain: [[docs/edge_cases/history-of-resource-owning-states]]

use crate::snapshot::Snapshot;

/// Result of [`HistoryStore::load`].
pub type LoadResult<C, X, H, E> = Result<Option<Snapshot<C, X, H>>, E>;

/// Write a shallow discriminant on exit. Missing history is not an error.
#[inline]
pub fn record_shallow<D>(slot: &mut Option<D>, disc: D) {
    *slot = Some(disc);
}

/// Write a deep subtree snapshot on exit. Must be cheap [`Clone`].
#[inline]
pub fn record_deep<T: Clone>(slot: &mut Option<T>, value: &T) {
    *slot = Some(value.clone());
}

/// Restore shallow history, or the declared default child on a miss.
///
/// Missing history is not an error.
#[inline]
pub fn restore_shallow<D, T>(last: Option<D>, default: T, restore: impl FnOnce(D) -> T) -> T {
    match last {
        Some(disc) => restore(disc),
        None => default,
    }
}

/// Restore a deep subtree, or `default` on a miss.
#[inline]
pub fn restore_deep<T: Clone>(last: Option<&T>, default: T) -> T {
    last.cloned().unwrap_or(default)
}

/// How a composite records where it was when it was last exited.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum HistoryKind {
    /// No memory. Re-entry uses the declared initial child.
    #[default]
    None,
    /// Store a discriminant. Re-enter that child, then that child's default descendant.
    Shallow,
    /// Store the subtree configuration. Only for `Copy` or trivially `Clone` subtrees.
    Deep,
}

/// Load and save phase-space snapshots. Swap in-memory for a file without
/// changing `update`.
///
/// Implementations must not execute commands. A store is inertial memory, not a
/// host.
pub trait HistoryStore {
    /// Configuration type stored in each snapshot.
    type Config;
    /// Extended-state type stored in each snapshot.
    type Context;
    /// History-sidecar type stored in each snapshot.
    type History;
    /// Store failure.
    type Error;

    /// Read the last completed snapshot, if any.
    fn load(&self) -> LoadResult<Self::Config, Self::Context, Self::History, Self::Error>;

    /// Write a snapshot taken *after* run-to-completion finished.
    fn save(
        &mut self,
        snap: &Snapshot<Self::Config, Self::Context, Self::History>,
    ) -> Result<(), Self::Error>;
}

/// Infallible error for [`MemoryStore`]. Never constructed.
pub type StoreError = core::convert::Infallible;

/// In-session history. Same snapshot type you would persist to disk.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct MemoryStore<C, X, H> {
    snap: Option<Snapshot<C, X, H>>,
}

impl<C, X, H> MemoryStore<C, X, H> {
    /// Empty store.
    #[inline]
    pub const fn new() -> Self {
        Self { snap: None }
    }
}

impl<C, X, H> HistoryStore for MemoryStore<C, X, H>
where
    C: Clone,
    X: Clone,
    H: Clone,
{
    type Config = C;
    type Context = X;
    type History = H;
    type Error = StoreError;

    fn load(&self) -> LoadResult<C, X, H, StoreError> {
        Ok(self.snap.clone())
    }

    fn save(&mut self, snap: &Snapshot<C, X, H>) -> Result<(), StoreError> {
        self.snap = Some(snap.clone());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{HistoryStore, MemoryStore};
    use crate::snapshot::Snapshot;

    #[test]
    fn save_then_load() {
        let mut store = MemoryStore::new();
        store
            .save(&Snapshot::new(1u8, 2u8, 3u8))
            .expect("memory store");
        let loaded = store.load().expect("memory store");
        assert_eq!(loaded, Some(Snapshot::new(1, 2, 3)));
    }
}

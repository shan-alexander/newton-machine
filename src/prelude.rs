//! Common imports for authors of a Newton machine.

pub use crate::cmd::Cmd;
pub use crate::combine::Combine;
pub use crate::history::{
    record_deep, record_shallow, restore_deep, restore_shallow, HistoryKind, HistoryStore,
    MemoryStore,
};
pub use crate::host::Tape;
pub use crate::machine::{apply, step, Boot, Machine};
pub use crate::outcome::Outcome;
pub use crate::rtc::{rtc, rtc_n, Inbox, Storm, DEFAULT_DRAIN_CAP};
pub use crate::runtime::Runtime;
pub use crate::snapshot::Snapshot;
pub use crate::sub::Sub;
pub use crate::topology::{lca, paths, Topology, MAX_DEPTH};
pub use crate::transition::{perform, Transitional};

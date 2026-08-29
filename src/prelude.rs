//! Common imports for authors of a Newton machine.

pub use crate::and::{And, AndHistory, AndNode};
pub use crate::bits::Bits;
#[cfg(feature = "alloc")]
pub use crate::chord::{ChordTable, Hit, MatchMode, Tie};
pub use crate::cmd::{Cmd, INLINE_CAP};
pub use crate::combine::Combine;
#[cfg(feature = "alloc")]
pub use crate::fleet::Fleet;
pub use crate::history::{
    record_deep, record_shallow, restore_deep, restore_shallow, HistoryKind, HistoryStore,
    MemoryStore,
};
pub use crate::host::{changed, Tape};
pub use crate::machine::{apply, step, try_apply, try_step, Boot, Machine};
pub use crate::outcome::Outcome;
pub use crate::rtc::{rtc, rtc_n, unwrap_storm, Inbox, Storm, DEFAULT_DRAIN_CAP};
pub use crate::runtime::Runtime;
pub use crate::snapshot::Snapshot;
pub use crate::sub::{Diff, Sub};
pub use crate::topology::{lca, paths, IntoNode, Topology, MAX_DEPTH};
pub use crate::transition::{perform, Transitional};

#[cfg(feature = "macros")]
pub use newton_machine_macros::{machine, IntoNode, Topology};

//! Newtonian state machines: typed Harel configurations driven by an Elm loop.
//!
//! Unidirectional Configuration Architecture (UCA): XOR is an `enum`, AND is a
//! `struct`, [`update`](Machine::update) is the only door, history is a sidecar,
//! commands are data.
//!
//! # Four laws
//!
//! 1. The configuration is the type (`enum` XOR, `struct` AND).
//! 2. The only mutation protocol is TEA (`Msg` in, `Cmd` out).
//! 3. History is inertial and external (a sidecar, not live variants).
//! 4. Typestate is a façade; the interior is a configuration tree.
//!
//! A Newton machine **may lock itself**. Only the host gateway may lock the
//! wire. Only the venue can save you when both are dead.
//!
//! # Minimal machine
//!
//! ```
//! use newton_machine::prelude::*;
//!
//! struct Chart;
//! struct Model;
//!
//! impl Machine for Chart {
//!     type Flags = ();
//!     type Model = Model;
//!     type Msg = ();
//!     type Cmd = Cmd<()>;
//!     type View = ();
//!     type History = ();
//!     type NodeId = &'static str;
//!
//!     fn init(_: ()) -> Boot<Self> {
//!         Boot::new(Chart, Model, (), Cmd::none())
//!     }
//!
//!     fn update(&mut self, _: &mut Model, _: &mut (), _: ()) -> Cmd<()> {
//!         Cmd::none()
//!     }
//!
//!     fn view(&self, _: &Model) {}
//!
//!     fn in_state(&self, _: &'static str) -> bool {
//!         true
//!     }
//! }
//!
//! let (mut rt, _) = Runtime::<Chart>::boot(());
//! let _ = rt.apply(());
//! ```
//!
//! Authors of hierarchical machines implement [`Topology`] and [`Transitional`],
//! then call [`perform()`] from [`Machine::update`]. Internal follow-ups go through
//! [`rtc()`] (return [`Storm`] via [`Machine::try_update`]; [`unwrap_storm`]
//! in `update`). Orthogonal regions: [`And`].
//!
//! GitHub carries runnable demos under `examples/` (not part of the crates.io
//! package). See the crate README.
//!
//! # Crate map
//!
//! | Module | Role |
//! | --- | --- |
//! | [`mod@machine`] / [`runtime`] | Elm loop: `init`, `update`, `try_apply`, `view` |
//! | [`topology`] / [`transition`] | Harel LCA: parent tree, exit/enter, `perform` |
//! | [`and`] | First-class Harel AND: [`And`] |
//! | [`rtc()`] / [`mod@rtc`] | Run-to-completion drain with a storm cap |
//! | [`cmd`] / [`sub`] / [`host`] | Effects as data; the host executes them |
//! | [`bits`] / chord / fleet | Host keys: bitset projection, chord table, N runtimes (`alloc`) |
//! | [`snapshot`] / [`history`] | `{config, context, history}` phase space |
//! | feature `macros` | `#[derive(Topology)]`, `#[derive(IntoNode)]`, `#[machine]`, [`perform!`] |
//!
//! [`prelude`] re-exports the types an author needs.

// rustbrain: [[docs/concepts/unidirectional-configuration-architecture]]
// rustbrain: [[docs/concepts/newtonian-state-machine]]
// rustbrain: [[docs/adr/0017-engine-is-topology-rtc-runtime]]
// rustbrain: [[docs/adr/0018-cmd-inline-then-heap]]
// rustbrain: [[docs/adr/0020-storm-panic-try-apply]]
// rustbrain: [[docs/adr/0021-and-combinator]]
// rustbrain: [[docs/adr/0022-sub-diff]]
// rustbrain: [[docs/adr/0023-chord-table-is-host-policy]]
// rustbrain: [[docs/adr/0024-macros-feature-hidden-proc-macro]]
// symbol:Machine symbol:Runtime symbol:Outcome symbol:Snapshot symbol:perform

#![no_std]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![forbid(unsafe_code)]
#![warn(missing_docs)]

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

pub mod and;
pub mod bits;
#[cfg(feature = "alloc")]
#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
pub mod chord;
pub mod cmd;
pub mod combine;
#[cfg(feature = "alloc")]
#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
pub mod fleet;
pub mod history;
pub mod host;
pub mod machine;
pub mod outcome;
pub mod prelude;
pub mod rtc;
pub mod runtime;
pub mod snapshot;
pub mod sub;
pub mod topology;
pub mod transition;

pub use and::{And, AndHistory, AndNode};
pub use bits::Bits;
#[cfg(feature = "alloc")]
pub use chord::{ChordTable, Hit, MatchMode, Tie};
pub use cmd::{Cmd, CmdOverflow, INLINE_CAP};
pub use combine::Combine;
#[cfg(feature = "alloc")]
pub use fleet::Fleet;
pub use history::{
    record_deep, record_shallow, restore_deep, restore_shallow, HistoryKind, HistoryStore,
    LoadResult, MemoryStore, StoreError,
};
pub use host::{changed, Tape};
pub use machine::{apply, step, try_apply, try_step, Boot, Machine};
pub use outcome::Outcome;
pub use rtc::{rtc, rtc_n, unwrap_storm, Inbox, Storm, DEFAULT_DRAIN_CAP};
pub use runtime::Runtime;
pub use snapshot::Snapshot;
pub use sub::{Diff, Sub};
pub use topology::{
    ancestors, enter_path, exit_path, lca, paths, Chain, IntoNode, Paths, Topology, MAX_DEPTH,
};
pub use transition::{perform, Transitional};

#[cfg(feature = "macros")]
#[cfg_attr(docsrs, doc(cfg(feature = "macros")))]
pub use newton_machine_macros::{machine, IntoNode, Topology};

mod mac;

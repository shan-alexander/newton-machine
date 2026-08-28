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
//! then call [`perform`] from [`Machine::update`]. Internal follow-ups go through
//! [`rtc()`].
//!
//! GitHub carries runnable demos under `examples/` (not part of the crates.io
//! package). See the crate README.
//!
//! # Crate map
//!
//! | Module | Role |
//! | --- | --- |
//! | [`machine`] / [`runtime`] | Elm loop: `init`, `update`, `view`, `subscriptions` |
//! | [`topology`] / [`transition`] | Harel LCA: parent tree, exit/enter, `perform` |
//! | [`rtc()`] / [`mod@rtc`] | Run-to-completion drain with a storm cap |
//! | [`cmd`] / [`sub`] / [`host`] | Effects as data; the host executes them |
//! | [`snapshot`] / [`history`] | `{config, context, history}` phase space |
//!
//! [`prelude`] re-exports the types an author needs.

// rustbrain: [[docs/concepts/unidirectional-configuration-architecture]]
// rustbrain: [[docs/concepts/newtonian-state-machine]]
// rustbrain: [[docs/adr/0017-engine-is-topology-rtc-runtime]]
// symbol:Machine symbol:Runtime symbol:Outcome symbol:Snapshot symbol:perform

#![no_std]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![forbid(unsafe_code)]
#![warn(missing_docs)]

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

pub mod cmd;
pub mod combine;
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

pub use cmd::Cmd;
pub use combine::Combine;
pub use history::{
    record_deep, record_shallow, restore_deep, restore_shallow, HistoryKind, HistoryStore,
    LoadResult, MemoryStore, StoreError,
};
pub use host::Tape;
pub use machine::{apply, step, Boot, Machine};
pub use outcome::Outcome;
pub use rtc::{rtc, rtc_n, Inbox, Storm, DEFAULT_DRAIN_CAP};
pub use runtime::Runtime;
pub use snapshot::Snapshot;
pub use sub::Sub;
pub use topology::{
    ancestors, enter_path, exit_path, lca, paths, Chain, Paths, Topology, MAX_DEPTH,
};
pub use transition::{perform, Transitional};

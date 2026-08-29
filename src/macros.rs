//! Feature `macros`: what an author (or AI agent) enables, writes, and gets back.
//!
//! **Always read this module before generating a Newton chart with macros.**
//! docs.rs builds with `--all-features`, so this page is on the crate root
//! sidebar as [`macros`](crate::macros).
//!
//! The git repository is a **Cargo workspace** (`newton-machine` +
//! `newton-machine-macros`). Application authors still depend on **one**
//! crates.io crate:
//!
//! ```toml
//! newton-machine = { version = "0.2", features = ["macros"] }
//! ```
//!
//! ```bash
//! cargo add newton-machine --features macros
//! ```
//!
//! Do **not** `cargo add newton-machine-macros`. That package exists only
//! because rustc requires `proc-macro = true` to be its own crate type.
//!
//! # When to use macros vs handwritten [`Machine`](crate::Machine)
//!
//! | Situation | Do this |
//! | --- | --- |
//! | Three-variant enum, no tree | Handwritten `impl Machine`. Macros add noise. |
//! | XOR tree, LCA/`perform`, `in_state` including ancestors | Enable `macros`. |
//! | Need a YAML/SCXML chart language | Stop. That is not this crate. |
//!
//! # Recipe (copy this)
//!
//! 1. `Node` id enum — `#[derive(Topology)]`, one `#[topology(root)]`,
//!    every other variant `#[topology(parent = …)]`.
//! 2. Chart enum — `#[derive(IntoNode)]`, `#[into_node(Node)]`. Variant
//!    **names** must match `Node` (payloads may differ).
//! 3. `impl Transitional for Chart` — you still write `exit` / `enter`
//!    (or leave the trait defaults). Macros do **not** invent I/O here.
//! 4. `#[machine(model, msg, cmd, view, node_id, …)] impl Chart { init; update; view }`
//! 5. LCA moves: `newton_machine::perform!(self, dest, model, hist)`
//!    (not `perform!` after `use prelude::*` — that glob imports the
//!    **function** [`crate::perform()`]).
//!
//! ```
//! # #[cfg(feature = "macros")]
//! # {
//! use newton_machine::prelude::*;
//!
//! #[derive(Clone, Copy, Debug, PartialEq, Eq, Topology)]
//! enum Node {
//!     #[topology(root)]
//!     Root,
//!     #[topology(parent = Root)]
//!     Off,
//!     #[topology(parent = Root)]
//!     On,
//! }
//!
//! #[derive(Clone, Copy, Debug, PartialEq, Eq, IntoNode)]
//! #[into_node(Node)]
//! enum Chart {
//!     Off,
//!     On,
//! }
//!
//! impl Transitional for Chart {
//!     type Ctx = ();
//!     type Hist = ();
//!     type Cmd = Cmd<()>;
//! }
//!
//! #[machine(model = (), msg = (), cmd = Cmd<()>, view = bool, node_id = Node)]
//! impl Chart {
//!     fn init(_: ()) -> Boot<Self> {
//!         Boot::new(Chart::Off, (), (), Cmd::none())
//!     }
//!     fn update(&mut self, m: &mut (), h: &mut (), _: ()) -> Cmd<()> {
//!         let dest = match *self {
//!             Chart::Off => Chart::On,
//!             Chart::On => Chart::Off,
//!         };
//!         newton_machine::perform!(self, dest, m, h)
//!     }
//!     fn view(&self, _: &()) -> bool {
//!         matches!(self, Chart::On)
//!     }
//! }
//!
//! let (mut rt, _) = Runtime::<Chart>::boot(());
//! assert!(rt.in_state(Node::Off) && rt.in_state(Node::Root));
//! assert!(!rt.in_state(Node::On));
//! let _ = rt.apply(());
//! assert!(rt.in_state(Node::On) && rt.view());
//! # }
//! ```
//!
//! # What the macros emit (so you can debug)
//!
//! | You write | Generated (readable; `cargo expand`) |
//! | --- | --- |
//! | `#[derive(Topology)]` on `Node` | `impl Topology for Node { type Node = Self; fn parent(...) }` |
//! | `#[derive(IntoNode)]` on `Chart` | `impl IntoNode for Chart { fn node(&self) -> Node }` |
//! | `#[machine] impl Chart { init, update, view }` | `impl Machine for Chart` with your three methods + `in_state` / `configuration` as **ancestor walk** from `node()` + `impl Topology for Chart` forwarding to `Node` |
//! | `newton_machine::perform!(self, dest, ctx, hist)` | `perform(self, self.node(), dest, dest.node(), ctx, hist)` |
//!
//! `update` stays **your** `match`. Entry/exit stay **your** [`crate::Transitional`].
//! History sidecar, `Cmd`, and host I/O are unchanged.
//!
//! # What to expect at runtime
//!
//! - `in_state(Root)` is true for every live configuration (Root is on every path).
//! - `in_state(leaf)` is true only when `node()` is that leaf or a descendant.
//! - `configuration(&mut buf)` writes leaf then parents, inner first.
//! - Cycle / two roots / unknown `parent =` → **compile error** on the variant, not a runtime panic.
//! - Missing `init` / `update` / `view` in the `#[machine] impl` → compile error naming the method.
//!
//! # What the macros will never do
//!
//! - Emit `async` or I/O inside `update` / `enter` / `exit`.
//! - Accept a `statechart! { on(Go) => Busy }` document as the source of truth.
//! - Generate YAML, string ids, or a `HashMap` of states.
//! - Replace [`And`](crate::And) or a handwritten AND `struct`.
//! - Write the history sidecar for you (`Transitional::exit` still does).
//!
//! # Attribute cheat sheet
//!
//! `#[machine(…)]` keys: `model`, `msg`, `cmd`, `view`, `node_id` (required);
//! `flags` and `history` default to `()`; `no_topology` skips forwarding
//! `Topology` onto the chart (if you already impl it by hand).
//!
//! Handwritten charts remain first-class. See `examples/connection.rs` on
//! GitHub and `tests/macros_xor.rs` in the crate.

#[cfg(feature = "macros")]
#[cfg_attr(docsrs, doc(cfg(feature = "macros")))]
pub use newton_machine_macros::{machine, IntoNode, Topology};

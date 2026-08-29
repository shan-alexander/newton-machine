//! # Counter — the Elm loop, and nothing else
//!
//! Run: `cargo run --example counter`
//!
//! This is the smallest Newton machine. If you already know The Elm
//! Architecture (TEA), you already know this file. Hierarchy, orthogonal
//! regions, and history are **not** here; they arrive in `connection.rs`.
//!
//! ## The loop
//!
//! ```text
//!   init  →  view
//!              ↑
//!    Msg  →  update  →  (new model, Cmd)
//!              ↑             │
//!              └─────────────┘  host executes Cmd, feeds results back as Msg
//! ```
//!
//! Newton names for the same loop:
//!
//! | Elm        | Newton machine                         | Physics metaphor      |
//! | ---------- | -------------------------------------- | --------------------- |
//! | `Msg`      | `Machine::Msg`                         | applied force         |
//! | `update`   | the only accelerator                   | second law            |
//! | `Cmd`      | data returned to the host, never I/O   | equal-and-opposite    |
//! | `Model`    | extended state                         | numbers, not control  |
//!
//! In this example the **configuration** (`Self` = `Counter`) has no variants.
//! All interesting data is the integer `Model`. That is legal: UCA does not
//! require a Harel tree. Use a Newton machine for a counter only as a teaching
//! step; a bare `i32` and a `match` would do for production.
//!
//! ## Not a Harel chart
//!
//! A Harel/statechart tool would still allow a one-state chart, but it would
//! usually be a document of ids plus an interpreter (or generated switch),
//! with entry actions that might call I/O. This file is **classic TEA**: one
//! `update`, no XOR tree, no LCA, no history. The Newton piece is the *loop
//! law* — `Runtime` owns the triple, `Msg` is the only force, `Cmd` would be
//! data if we had any.
//!
//! Alternatives we did not use: `Box<dyn State>`, a `"Idle"` string id, a
//! typestate `Counter<N>` phantom, or incrementing from `main`. Those all
//! break “one door” or “the configuration is a value you can snapshot.”
//!
//! Expected output:
//!
//! ```text
//! init  0
//! view  1
//! view  2
//! view  1
//! ```

use newton_machine::prelude::*;

/// The configuration tree. A unit struct: there is only one "state."
///
/// In `connection.rs` this becomes an `enum` (XOR). Here it exists so
/// `Runtime` has something to own besides the integer.
struct Counter;

/// Forces the host (this `main`) applies. Clicks, ticks, fills — all `Msg`.
///
/// `update` is the only function allowed to change the model. `main` must
/// not do `*model += 1` itself.
#[derive(Clone, Copy)]
enum Msg {
    Inc,
    Dec,
}

impl Machine for Counter {
    /// Construction input. Elm calls this "flags." `()` = none.
    type Flags = ();
    /// Extended state: the integer. Heavy or frequently changing data
    /// belongs here, not in the configuration type.
    type Model = i32;
    type Msg = Msg;
    /// No effects. `()` implements [`Combine`], so RTC helpers still work.
    /// `connection.rs` uses `Cmd<HostCmd>` instead — descriptions of I/O.
    type Cmd = ();
    /// What `view` returns. Need not be HTML; here it is the integer itself.
    type View = i32;
    /// No history sidecar. `()` means "no composite opted into inertia."
    type History = ();
    /// `in_state` query key. Unused; a Harel chart would use a node-id enum.
    type NodeId = ();

    /// First configuration, first model, empty history, no entry command.
    ///
    /// [`Boot::new`] is `(machine, model, history, cmd)`. [`Runtime::boot`]
    /// calls this once, then you only ever `apply` messages.
    fn init(_: ()) -> Boot<Self> {
        Boot::new(Counter, 0, (), ())
    }

    /// The only door. `&mut self` is the configuration (unchanged here).
    /// `&mut i32` is the model. The ignored `&mut ()` is history.
    ///
    /// Must not perform I/O. We return `()` — no command for the host.
    fn update(&mut self, model: &mut i32, _: &mut (), msg: Msg) {
        match msg {
            Msg::Inc => *model += 1,
            Msg::Dec => *model -= 1,
        }
    }

    /// Pure projection. Same model ⇒ same view. Safe to call anytime.
    fn view(&self, model: &i32) -> i32 {
        *model
    }

    /// Always "in" the one node. Hierarchical charts use this for
    /// `in_state(Node::Online)` tests; see `connection.rs`.
    fn in_state(&self, _: ()) -> bool {
        true
    }
}

fn main() {
    // TEA runtime: owns {config, model, history} = {Counter, i32, ()}.
    // Not Tokio, not a GUI. Just the owner of the triple + `apply`.
    // The discarded value is the entry `Cmd` from `init` (here `()`).
    let (mut rt, _) = Runtime::<Counter>::boot(());
    println!("init  {}", rt.view());

    // Each `apply` is one run-to-completion step: one Msg, then view.
    // The host decides *when* messages happen (this for-loop). The
    // machine decides *what they mean*.
    for msg in [Msg::Inc, Msg::Inc, Msg::Dec] {
        rt.apply(msg);
        println!("view  {}", rt.view());
    }
}

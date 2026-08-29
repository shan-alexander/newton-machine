//! # Connection — XOR tree, LCA transitions, history, commands as data
//!
//! Run: `cargo run --example connection`
//!
//! Read [`counter.rs`](counter.rs) first. That file is TEA. This file adds the
//! rest of a Newton machine:
//!
//! 1. **XOR configuration** — an `enum`. You cannot be `Offline` and `Online`.
//! 2. **`Topology` + `perform`** — Harel exit/enter around a typed assignment.
//! 3. **History sidecar** — inertia: remember `Sync` across logout.
//! 4. **`Cmd` as data** — `HttpConnect` / `Persist` are *intent*. `main` prints
//!    them instead of opening a socket. That is the third law.
//!
//! `Session` is an AND node (a `struct`): `auth` and `sync` are both active
//! while `Online`. This example barely uses that (see `orthogonal.rs`). It
//! still stores both fields so a later `Dirty` does not have to flatten
//! `Online` into a mega-enum of every combination.
//!
//! ## The chart
//!
//! ```text
//!            Root
//!          /  |  \
//!    Offline  Connecting  Online ── Session { auth, sync }
//! ```
//!
//! `Node` is the id tree [`Topology::parent`] walks. `Conn` is the live
//! configuration (the type). They stay in sync by construction: each `Conn`
//! variant maps to one `Node`.
//!
//! ## One transition, in order
//!
//! `Msg::Connect` with `Conn::Offline` calls [`perform`]:
//!
//! 1. LCA of `Offline` and `Connecting` is `Root` (both are children of root).
//! 2. **Exit** `Offline` (inner first). Our `exit` is a no-op there.
//! 3. **Install** `Conn::Connecting`.
//! 4. **Enter** `Connecting` (outer first) → `Cmd::HttpConnect`.
//!
//! The host (`main`) prints `HttpConnect`. A real host would start TLS, then
//! feed `Msg::Authed(user)` back in. `update` never opens the socket.
//!
//! ## History (first law)
//!
//! On `Logout`, `exit(Online)` writes `last_sync`. On the next `Authed`, we
//! **read** that sidecar *before* `perform` and build `dest` with the old
//! `Sync`. Re-entry is not the declared default (`Idle`); it is wherever we
//! were. Missing history still defaults to `Idle` (first login).
//!
//! `Dirty` while `Online` does **not** call `perform`. It assigns a field
//! inside the AND node. LCA would be `Online` itself — nothing to exit — so a
//! direct assignment is the honest encoding.
//!
//! ## LCA (least common ancestor), in this tree
//!
//! LCA is the deepest node that is an ancestor of both source and target
//! (a node counts as its own ancestor). `perform` exits *up to but not
//! including* the LCA, assigns `dest`, then enters *down from* the LCA:
//!
//! ```text
//! Msg::Connect:  Offline → Connecting
//!                LCA = Root  ⇒  exit Offline, enter Connecting
//!                (we do not “exit the whole machine”)
//!
//! Msg::Authed:   Connecting → Online
//!                LCA = Root  ⇒  exit Connecting, enter Online
//!
//! Msg::Logout:   Online → Offline
//!                LCA = Root  ⇒  exit Online (record last_sync), enter Offline
//!
//! Msg::Dirty:    stay Online, only Session.sync changes
//!                LCA would be Online  ⇒  nothing to exit; field assign
//! ```
//!
//! A Harel chart specifies this same geometry. A SCXML engine would look up
//! `"Offline"` / `"Connecting"` in a table and might run `<script>` on enter.
//! Here the geometry is `Topology::parent` + nested ADTs, and enter returns
//! `Cmd::HttpConnect` instead of calling TLS.
//!
//! ## Newton vs Harel (same chart, different machine)
//!
//! This *chart* is Harel: XOR `Conn`, AND `Session`, history on leaving
//! `Online`, RTC (one `apply` per `Msg`). The *machine* is Newton/UCA where
//! Harel-as-usually-coded is not:
//!
//! | Harel / SCXML typical           | This file                              |
//! | ------------------------------- | -------------------------------------- |
//! | string ids, current-set         | `Conn` enum is the configuration       |
//! | enter Connecting → call HTTP    | enter returns `Cmd::HttpConnect`       |
//! | history inside live state       | `History.last_sync` sidecar            |
//! | interpreter heap as snapshot    | `{Conn, (), History}` after each apply |
//!
//! If you needed a PDF of legal arrows for a standards body, draw Harel. If
//! you need crash-restart without replaying a socket, you needed this split.
//!
//! ## Expected output
//!
//! ```text
//! view=offline      cmd=none
//! view=connecting   cmd=HttpConnect
//! view=online       cmd=Persist
//! view=online/dirty cmd=none
//! view=offline      cmd=Persist
//! view=connecting   cmd=HttpConnect
//! view=online/dirty cmd=Persist    ← sync restored from the sidecar
//! ```

use newton_machine::prelude::*;

// ---------------------------------------------------------------------------
// Configuration: XOR = enum, AND = struct
// ---------------------------------------------------------------------------

/// Live control state. Exactly one variant is active — that is XOR.
///
/// `Online` *contains* [`Session`], so while we are online the AND fields
/// `auth` and `sync` are also active. Illegal combos (`Offline` + `SignedIn`)
/// cannot be written down.
#[derive(Clone, Debug, PartialEq, Eq)]
enum Conn {
    Offline,
    Connecting,
    Online(Session),
}

/// AND node: every field is an orthogonal region, all live at once.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Session {
    auth: Auth,
    sync: Sync,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Auth {
    SignedIn { user: u64 },
}

/// Discrete mode we want to remember across a logout. Values, not sockets.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Sync {
    Idle,
    Dirty,
}

// ---------------------------------------------------------------------------
// Sidecar, messages, host commands, node ids
// ---------------------------------------------------------------------------

/// Inertia. Not part of `Conn`: mixing ghosts into the live enum would make
/// every clone and `PartialEq` pay for yesterday.
///
/// Written in `exit(Online)`, read when building the next `Online` dest.
#[derive(Clone, Debug, Default)]
struct History {
    last_sync: Option<Sync>,
}

/// Forces. `Authed` is a *fact from the host* (the HTTP layer succeeded).
/// `Connect` / `Logout` / `Dirty` are user or strategy intent.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Msg {
    Connect,
    Authed(u64),
    Dirty,
    Logout,
}

/// Reactions. `update` returns these; it does not call HTTP or disk.
///
/// A production host maps `HttpConnect` → a real request and later
/// `apply(Msg::Authed(id))`. Here the host is `println`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum HostCmd {
    HttpConnect,
    Persist,
}

/// Ids for the topology walk. Prefer a `Copy` enum, never a `String` on
/// the hot path. `Root` exists so `Offline` and `Connecting` share a parent
/// (the LCA of a sibling transition).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Node {
    Root,
    Offline,
    Connecting,
    Online,
}

// ---------------------------------------------------------------------------
// Harel kinematics: parent tree + exit/enter
// ---------------------------------------------------------------------------

impl Topology for Conn {
    type Node = Node;

    /// Tree used by [`lca`] / [`perform`]. Depth 1: everything hangs off Root.
    fn parent(node: Node) -> Option<Node> {
        match node {
            Node::Root => None,
            _ => Some(Node::Root),
        }
    }
}

impl Transitional for Conn {
    type Ctx = ();
    type Hist = History;
    type Cmd = Cmd<HostCmd>;

    /// Inner-first, still in the *old* configuration.
    ///
    /// Record history here (first law). Return commands here if leaving
    /// should flush (`Persist` on leaving `Online`). Do not open sockets.
    fn exit(&mut self, node: Node, _: &mut (), hist: &mut History) -> Cmd<HostCmd> {
        if node == Node::Online {
            if let Conn::Online(s) = self {
                hist.last_sync = Some(s.sync);
            }
            return Cmd::single(HostCmd::Persist);
        }
        Cmd::none()
    }

    /// Outer-first, on the *already installed* destination.
    ///
    /// `HttpConnect` is entry of `Connecting`: "host, please dial."
    /// `Persist` on entering `Online`: "host, please journal."
    fn enter(&mut self, node: Node, _: &mut (), _: &mut History) -> Cmd<HostCmd> {
        match node {
            Node::Connecting => Cmd::single(HostCmd::HttpConnect),
            Node::Online => Cmd::single(HostCmd::Persist),
            _ => Cmd::none(),
        }
    }
}

// ---------------------------------------------------------------------------
// TEA: the only mutation protocol
// ---------------------------------------------------------------------------

impl Machine for Conn {
    type Flags = ();
    /// No extra numbers in this demo. A real connection would keep retry
    /// counts, last error, bytes — here, in `Model`, not in `Conn`.
    type Model = ();
    type Msg = Msg;
    type Cmd = Cmd<HostCmd>;
    type View = &'static str;
    type History = History;
    type NodeId = Node;

    fn init(_: ()) -> Boot<Self> {
        // Start Offline. Entry of Offline produces no Cmd.
        Boot::new(Conn::Offline, (), History::default(), Cmd::none())
    }

    fn update(&mut self, model: &mut (), hist: &mut History, msg: Msg) -> Cmd<HostCmd> {
        match (&*self, msg) {
            // Sibling transition Offline → Connecting. LCA = Root.
            (Conn::Offline, Msg::Connect) => perform(
                self,
                Node::Offline,
                Conn::Connecting,
                Node::Connecting,
                model,
                hist,
            ),

            // Connecting → Online. Build dest FIRST (history read), then
            // perform (exit Connecting, install, enter Online).
            (Conn::Connecting, Msg::Authed(user)) => {
                let sync = hist.last_sync.unwrap_or(Sync::Idle);
                let dest = Conn::Online(Session {
                    auth: Auth::SignedIn { user },
                    sync,
                });
                perform(self, Node::Connecting, dest, Node::Online, model, hist)
            }

            // Stay in Online; only the AND field `sync` changes. No LCA.
            (Conn::Online(s), Msg::Dirty) => {
                *self = Conn::Online(Session {
                    auth: s.auth,
                    sync: Sync::Dirty,
                });
                Cmd::none()
            }

            // Leave the AND node as a whole. exit(Online) records last_sync.
            (Conn::Online(_), Msg::Logout) => perform(
                self,
                Node::Online,
                Conn::Offline,
                Node::Offline,
                model,
                hist,
            ),

            // Unknown pair: ignore. No implicit "reset."
            _ => Cmd::none(),
        }
    }

    fn view(&self, _: &()) -> &'static str {
        match self {
            Conn::Offline => "offline",
            Conn::Connecting => "connecting",
            Conn::Online(Session {
                sync: Sync::Dirty, ..
            }) => "online/dirty",
            Conn::Online(_) => "online",
        }
    }

    fn in_state(&self, id: Node) -> bool {
        matches!(
            (self, id),
            (Conn::Offline, Node::Offline)
                | (Conn::Connecting, Node::Connecting)
                | (Conn::Online(_), Node::Online)
                | (_, Node::Root)
        )
    }
}

fn dump(rt: &Runtime<Conn>, cmd: &Cmd<HostCmd>) {
    print!("view={:<12} cmd=", rt.view());
    let mut any = false;
    for c in cmd.iter() {
        if any {
            print!(",");
        }
        print!("{c:?}");
        any = true;
    }
    if !any {
        print!("none");
    }
    println!();
}

fn main() {
    println!("# connection  (cmd is intent; a real host would execute I/O)\n");

    // boot = init + own the triple. `cmd` is the entry command of Offline.
    let (mut rt, cmd) = Runtime::<Conn>::boot(());
    dump(&rt, &cmd);

    // Scripted host. In production these Msgs come from UI, HTTP, a bar feed.
    for msg in [
        Msg::Connect,
        Msg::Authed(7),
        Msg::Dirty,
        Msg::Logout,
        Msg::Connect,
        Msg::Authed(7),
    ] {
        let cmd = rt.apply(msg);
        dump(&rt, &cmd);
    }
}

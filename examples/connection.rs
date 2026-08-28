//! Connection chart: XOR + AND, LCA `perform`, history sidecar.
//!
//! Commands print as intent. Nothing opens a socket.
//!
//! ```text
//! cargo run --example connection
//! ```

use newton_machine::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq)]
enum Conn {
    Offline,
    Connecting,
    Online(Session),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Session {
    auth: Auth,
    sync: Sync,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Auth {
    SignedIn { user: u64 },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Sync {
    Idle,
    Dirty,
}

#[derive(Clone, Debug, Default)]
struct History {
    last_sync: Option<Sync>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Msg {
    Connect,
    Authed(u64),
    Dirty,
    Logout,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum HostCmd {
    HttpConnect,
    Persist,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Node {
    Root,
    Offline,
    Connecting,
    Online,
}

impl Topology for Conn {
    type Node = Node;
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

    fn exit(&mut self, node: Node, _: &mut (), hist: &mut History) -> Cmd<HostCmd> {
        if node == Node::Online {
            if let Conn::Online(s) = self {
                hist.last_sync = Some(s.sync);
            }
            return Cmd::single(HostCmd::Persist);
        }
        Cmd::none()
    }

    fn enter(&mut self, node: Node, _: &mut (), _: &mut History) -> Cmd<HostCmd> {
        match node {
            Node::Connecting => Cmd::single(HostCmd::HttpConnect),
            Node::Online => Cmd::single(HostCmd::Persist),
            _ => Cmd::none(),
        }
    }
}

impl Machine for Conn {
    type Flags = ();
    type Model = ();
    type Msg = Msg;
    type Cmd = Cmd<HostCmd>;
    type View = &'static str;
    type History = History;
    type NodeId = Node;

    fn init(_: ()) -> Boot<Self> {
        Boot::new(Conn::Offline, (), History::default(), Cmd::none())
    }

    fn update(&mut self, model: &mut (), hist: &mut History, msg: Msg) -> Cmd<HostCmd> {
        match (&*self, msg) {
            (Conn::Offline, Msg::Connect) => perform(
                self,
                Node::Offline,
                Conn::Connecting,
                Node::Connecting,
                model,
                hist,
            ),
            (Conn::Connecting, Msg::Authed(user)) => {
                let sync = hist.last_sync.unwrap_or(Sync::Idle);
                let dest = Conn::Online(Session {
                    auth: Auth::SignedIn { user },
                    sync,
                });
                perform(self, Node::Connecting, dest, Node::Online, model, hist)
            }
            (Conn::Online(s), Msg::Dirty) => {
                *self = Conn::Online(Session {
                    auth: s.auth,
                    sync: Sync::Dirty,
                });
                Cmd::none()
            }
            (Conn::Online(_), Msg::Logout) => perform(
                self,
                Node::Online,
                Conn::Offline,
                Node::Offline,
                model,
                hist,
            ),
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
    println!("# connection  (cmd is intent; host would execute I/O)\n");
    let (mut rt, cmd) = Runtime::<Conn>::boot(());
    dump(&rt, &cmd);
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

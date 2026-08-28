//! XOR/AND tree driven by Topology + Transitional + Runtime.
//!
//! symbol:Machine symbol:perform symbol:Runtime symbol:Outcome

use newton_machine::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq)]
enum Conn {
    Offline,
    Connecting { attempt: u8 },
    Online(Session),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Session {
    auth: Auth,
    sync: Sync,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Auth {
    Anonymous,
    SignedIn { user: u64 },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Sync {
    Idle,
    #[allow(dead_code)]
    Fetching,
    Dirty,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum AuthDisc {
    Anonymous,
    SignedIn,
}

impl From<&Auth> for AuthDisc {
    fn from(auth: &Auth) -> Self {
        match auth {
            Auth::Anonymous => AuthDisc::Anonymous,
            Auth::SignedIn { .. } => AuthDisc::SignedIn,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct History {
    last_auth: Option<AuthDisc>,
    last_session: Option<Session>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct Context {
    last_error: Option<&'static str>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Msg {
    Connect,
    Authed(u64),
    Logout,
    Tick,
    MarkDirty,
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
    Anonymous,
    SignedIn,
    Idle,
    Fetching,
    Dirty,
}

impl Conn {
    fn node(&self) -> Node {
        match self {
            Conn::Offline => Node::Offline,
            Conn::Connecting { .. } => Node::Connecting,
            Conn::Online(_) => Node::Online,
        }
    }

    fn session_dest(history: &History, user: u64) -> Session {
        let auth = restore_shallow(history.last_auth, Auth::SignedIn { user }, |d| match d {
            AuthDisc::Anonymous => Auth::Anonymous,
            AuthDisc::SignedIn => Auth::SignedIn { user },
        });
        let sync = history.last_session.map(|s| s.sync).unwrap_or(Sync::Idle);
        Session { auth, sync }
    }

    fn react(&mut self, model: &mut Context, history: &mut History, msg: Msg) -> Cmd<HostCmd> {
        match (&*self, msg) {
            (Conn::Offline, Msg::Connect) => {
                let dest = Conn::Connecting { attempt: 1 };
                let to = dest.node();
                perform(self, Node::Offline, dest, to, model, history)
            }
            (Conn::Connecting { .. }, Msg::Authed(user)) => {
                let dest = Conn::Online(Self::session_dest(history, user));
                let to = dest.node();
                perform(self, Node::Connecting, dest, to, model, history)
            }
            (Conn::Online(session), Msg::MarkDirty) => {
                let from = match session.sync {
                    Sync::Idle => Node::Idle,
                    Sync::Fetching => Node::Fetching,
                    Sync::Dirty => Node::Dirty,
                };
                match session.sync {
                    Sync::Idle => {
                        let dest = Conn::Online(Session {
                            auth: session.auth,
                            sync: Sync::Dirty,
                        });
                        perform(self, from, dest, Node::Dirty, model, history)
                    }
                    _ => Cmd::none(),
                }
            }
            (Conn::Online(_), Msg::Logout) => {
                let dest = Conn::Offline;
                perform(self, Node::Online, dest, Node::Offline, model, history)
            }
            (_, Msg::Tick) => {
                model.last_error = None;
                Cmd::none()
            }
            _ => Cmd::none(),
        }
    }
}

impl Topology for Conn {
    type Node = Node;

    fn parent(node: Node) -> Option<Node> {
        match node {
            Node::Root => None,
            Node::Offline | Node::Connecting | Node::Online => Some(Node::Root),
            Node::Anonymous | Node::SignedIn | Node::Idle | Node::Fetching | Node::Dirty => {
                Some(Node::Online)
            }
        }
    }
}

impl Transitional for Conn {
    type Ctx = Context;
    type Hist = History;
    type Cmd = Cmd<HostCmd>;

    fn exit(&mut self, node: Node, _ctx: &mut Context, hist: &mut History) -> Cmd<HostCmd> {
        if node == Node::Online {
            if let Conn::Online(session) = self {
                record_shallow(&mut hist.last_auth, AuthDisc::from(&session.auth));
                record_deep(&mut hist.last_session, session);
            }
            return Cmd::single(HostCmd::Persist);
        }
        Cmd::none()
    }

    fn enter(&mut self, node: Node, _ctx: &mut Context, _hist: &mut History) -> Cmd<HostCmd> {
        match node {
            Node::Connecting => Cmd::single(HostCmd::HttpConnect),
            Node::Online => Cmd::single(HostCmd::Persist),
            _ => Cmd::none(),
        }
    }
}

impl Machine for Conn {
    type Flags = ();
    type Model = Context;
    type Msg = Msg;
    type Cmd = Cmd<HostCmd>;
    type View = &'static str;
    type History = History;
    type NodeId = Node;

    fn init(_flags: ()) -> Boot<Self> {
        Boot::new(
            Conn::Offline,
            Context::default(),
            History::default(),
            Cmd::none(),
        )
    }

    fn update(&mut self, model: &mut Context, history: &mut History, msg: Msg) -> Cmd<HostCmd> {
        rtc(msg, |msg, inbox| {
            let _ = inbox;
            self.react(model, history, msg)
        })
        .expect("connection chart does not enqueue follow-ups")
    }

    fn view(&self, _model: &Context) -> &'static str {
        match self {
            Conn::Offline => "offline",
            Conn::Connecting { .. } => "connecting",
            Conn::Online(_) => "online",
        }
    }

    fn subscriptions(&self, _model: &Context) -> Sub<Msg> {
        match self {
            Conn::Connecting { .. } | Conn::Online(_) => Sub::single(Msg::Tick),
            Conn::Offline => Sub::none(),
        }
    }

    fn in_state(&self, id: Node) -> bool {
        let mut buf = [Node::Root; 8];
        let n = self.configuration(&mut buf);
        buf[..n].contains(&id)
    }

    fn configuration(&self, out: &mut [Node]) -> usize {
        let mut n = 0;
        let mut push = |id: Node| {
            if n < out.len() {
                out[n] = id;
                n += 1;
            }
        };
        push(Node::Root);
        match self {
            Conn::Offline => push(Node::Offline),
            Conn::Connecting { .. } => push(Node::Connecting),
            Conn::Online(s) => {
                push(Node::Online);
                push(match s.auth {
                    Auth::Anonymous => Node::Anonymous,
                    Auth::SignedIn { .. } => Node::SignedIn,
                });
                push(match s.sync {
                    Sync::Idle => Node::Idle,
                    Sync::Fetching => Node::Fetching,
                    Sync::Dirty => Node::Dirty,
                });
            }
        }
        n
    }
}

#[test]
fn connect_auth_logout_restores_deep_sync() {
    let (mut rt, cmd) = Runtime::<Conn>::boot(());
    assert!(cmd.is_none());
    assert!(rt.in_state(Node::Offline));
    assert!(rt.subscriptions().is_none());

    let cmd = rt.apply(Msg::Connect);
    assert_eq!(cmd, Cmd::single(HostCmd::HttpConnect));
    assert!(rt.in_state(Node::Connecting));
    assert!(!rt.subscriptions().is_none());

    let cmd = rt.apply(Msg::Authed(7));
    assert_eq!(cmd, Cmd::single(HostCmd::Persist));
    assert!(rt.in_state(Node::Online));
    assert!(rt.in_state(Node::SignedIn));
    assert!(rt.in_state(Node::Idle));

    let cmd = rt.apply(Msg::MarkDirty);
    assert!(cmd.is_none());
    assert!(rt.in_state(Node::Online));
    assert!(rt.in_state(Node::Dirty));
    assert!(rt.in_state(Node::SignedIn));

    let cmd = rt.apply(Msg::Logout);
    assert_eq!(cmd, Cmd::single(HostCmd::Persist));
    assert!(rt.in_state(Node::Offline));
    assert_eq!(
        rt.history().last_session.as_ref().unwrap().sync,
        Sync::Dirty
    );

    let _ = rt.apply(Msg::Connect);
    let _ = rt.apply(Msg::Authed(7));
    assert!(rt.in_state(Node::Dirty));
    assert!(rt.in_state(Node::SignedIn));

    let mut store = MemoryStore::new();
    rt.persist(&mut store).unwrap();
    let restored = Runtime::<Conn>::load(&store).unwrap().unwrap();
    assert_eq!(restored.machine(), rt.machine());
    assert_eq!(restored.history(), rt.history());
}

#[test]
fn history_miss_uses_default_child() {
    let (mut rt, _) = Runtime::<Conn>::boot(());
    let _ = rt.apply(Msg::Connect);
    let _ = rt.apply(Msg::Authed(1));
    assert!(rt.in_state(Node::SignedIn));
}

#[test]
fn mark_dirty_does_not_exit_online() {
    let (mut rt, _) = Runtime::<Conn>::boot(());
    let _ = rt.apply(Msg::Connect);
    let _ = rt.apply(Msg::Authed(1));
    assert!(rt.history().last_session.is_none());
    let _ = rt.apply(Msg::MarkDirty);
    assert!(
        rt.history().last_session.is_none(),
        "orthogonal leaf transition must not record Online history"
    );
}

#[test]
fn tape_records_host_intent() {
    let (mut rt, _) = Runtime::<Conn>::boot(());
    let mut tape = Tape::new();
    tape.record(rt.apply(Msg::Connect));
    tape.record(rt.apply(Msg::Authed(1)));
    #[cfg(feature = "alloc")]
    assert_eq!(tape.as_slice(), &[HostCmd::HttpConnect, HostCmd::Persist]);
}

#[test]
fn step_and_apply_same_semantics() {
    let boot = Conn::init(());
    let (m, _model, _hist, cmd) = step(boot.machine, boot.model, boot.history, Msg::Connect);
    assert_eq!(cmd, Cmd::single(HostCmd::HttpConnect));

    let boot = Conn::init(());
    let mut machine = boot.machine;
    let mut model = boot.model;
    let mut history = boot.history;
    let cmd = apply(&mut machine, &mut model, &mut history, Msg::Connect);
    assert_eq!(cmd, Cmd::single(HostCmd::HttpConnect));
    assert_eq!(machine, m);
}

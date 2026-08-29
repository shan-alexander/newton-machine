//! First-class AND: two machines, one Msg, document order.

use newton_machine::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Auth {
    Anon,
    User,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Sync {
    Idle,
    Busy,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Msg {
    Login,
    Tick,
}

struct AuthM(Auth);

impl Machine for AuthM {
    type Flags = ();
    type Model = ();
    type Msg = Msg;
    type Cmd = Cmd<&'static str>;
    type View = Auth;
    type History = ();
    type NodeId = &'static str;

    fn init(_: ()) -> Boot<Self> {
        Boot::new(AuthM(Auth::Anon), (), (), Cmd::none())
    }

    fn update(&mut self, _: &mut (), _: &mut (), msg: Msg) -> Cmd<&'static str> {
        if let (Auth::Anon, Msg::Login) = (self.0, msg) {
            self.0 = Auth::User;
            Cmd::single("auth")
        } else {
            Cmd::none()
        }
    }

    fn view(&self, _: &()) -> Auth {
        self.0
    }

    fn in_state(&self, id: &'static str) -> bool {
        matches!((id, self.0), ("anon", Auth::Anon) | ("user", Auth::User))
    }
}

struct SyncM(Sync);

impl Machine for SyncM {
    type Flags = ();
    type Model = u32;
    type Msg = Msg;
    type Cmd = Cmd<&'static str>;
    type View = Sync;
    type History = ();
    type NodeId = &'static str;

    fn init(_: ()) -> Boot<Self> {
        Boot::new(SyncM(Sync::Idle), 0, (), Cmd::none())
    }

    fn update(&mut self, ticks: &mut u32, _: &mut (), msg: Msg) -> Cmd<&'static str> {
        if msg == Msg::Tick {
            *ticks += 1;
            self.0 = match self.0 {
                Sync::Idle => Sync::Busy,
                Sync::Busy => Sync::Idle,
            };
            Cmd::single("sync")
        } else {
            Cmd::none()
        }
    }

    fn view(&self, _: &u32) -> Sync {
        self.0
    }

    fn in_state(&self, id: &'static str) -> bool {
        matches!((id, self.0), ("idle", Sync::Idle) | ("busy", Sync::Busy))
    }
}

type Session = And<AuthM, SyncM>;

#[test]
fn document_order_left_then_right() {
    let (mut rt, _) = Runtime::<Session>::boot(((), ()));
    assert!(rt.in_state(AndNode::Left("anon")));
    assert!(rt.in_state(AndNode::Right("idle")));

    let cmd = rt.apply(Msg::Tick);
    assert_eq!(cmd.iter().copied().collect::<Vec<_>>(), vec!["sync"]);
    assert_eq!(rt.view(), (Auth::Anon, Sync::Busy));
    assert_eq!(rt.model().1, 1);

    let cmd = rt.apply(Msg::Login);
    assert_eq!(cmd.iter().copied().collect::<Vec<_>>(), vec!["auth"]);
    assert!(rt.in_state(AndNode::Left("user")));
    assert!(rt.in_state(AndNode::Right("busy")));

    let cmd = rt.apply(Msg::Tick);
    assert_eq!(cmd.iter().copied().collect::<Vec<_>>(), vec!["sync"]);
    assert_eq!(rt.view(), (Auth::User, Sync::Idle));
}

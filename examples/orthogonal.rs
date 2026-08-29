//! AND node: [`And<L, R>`] is first-class orthogonality.
//!
//! Regions are two machines, not threads. Both see `Tick` / `Login` in
//! document order (left, then right) on one RTC clock.
//!
//! ```text
//! cargo run --example orthogonal
//! ```
//!
//! A handwritten `struct { auth, sync }` with one shared `Model` is still
//! valid. Use [`And`] when the regions are already machines.

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

#[derive(Clone, Copy, Debug)]
enum Msg {
    Login,
    Tick,
}

struct AuthM(Auth);

impl Machine for AuthM {
    type Flags = ();
    type Model = ();
    type Msg = Msg;
    type Cmd = ();
    type View = Auth;
    type History = ();
    type NodeId = &'static str;

    fn init(_: ()) -> Boot<Self> {
        Boot::new(AuthM(Auth::Anon), (), (), ())
    }

    fn update(&mut self, _: &mut (), _: &mut (), msg: Msg) {
        if let (Auth::Anon, Msg::Login) = (self.0, msg) {
            self.0 = Auth::User;
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
    type Cmd = ();
    type View = Sync;
    type History = ();
    type NodeId = &'static str;

    fn init(_: ()) -> Boot<Self> {
        Boot::new(SyncM(Sync::Idle), 0, (), ())
    }

    fn update(&mut self, ticks: &mut u32, _: &mut (), msg: Msg) {
        if matches!(msg, Msg::Tick) {
            *ticks += 1;
            self.0 = match self.0 {
                Sync::Idle => Sync::Busy,
                Sync::Busy => Sync::Idle,
            };
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

fn main() {
    println!("# orthogonal  (And<Auth, Sync> — first-class Harel AND)\n");
    let (mut rt, _) = Runtime::<Session>::boot(((), ()));
    println!("start  {:?}", rt.view());
    rt.apply(Msg::Tick);
    println!("tick   {:?}  ticks={}", rt.view(), rt.model().1);
    rt.apply(Msg::Login);
    println!("login  {:?}", rt.view());
    rt.apply(Msg::Tick);
    println!("tick   {:?}", rt.view());
    println!(
        "in_state user={} busy={}",
        rt.in_state(AndNode::Left("user")),
        rt.in_state(AndNode::Right("busy"))
    );
}

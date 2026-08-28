//! AND node: two XOR regions, one message, one RTC step.
//!
//! Regions are struct fields, not threads. Both see `Tick` in field order.
//!
//! ```text
//! cargo run --example orthogonal
//! ```

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
struct Session {
    auth: Auth,
    sync: Sync,
}

#[derive(Clone, Copy, Debug)]
enum Msg {
    Login,
    Tick,
}

impl Session {
    fn offer(&mut self, msg: Msg) {
        if let (Auth::Anon, Msg::Login) = (self.auth, msg) {
            self.auth = Auth::User;
        }
        match (self.sync, msg) {
            (Sync::Idle, Msg::Tick) => self.sync = Sync::Busy,
            (Sync::Busy, Msg::Tick) => self.sync = Sync::Idle,
            _ => {}
        }
    }
}

impl Machine for Session {
    type Flags = ();
    type Model = u32;
    type Msg = Msg;
    type Cmd = ();
    type View = (Auth, Sync, u32);
    type History = ();
    type NodeId = &'static str;

    fn init(_: ()) -> Boot<Self> {
        Boot::new(
            Session {
                auth: Auth::Anon,
                sync: Sync::Idle,
            },
            0,
            (),
            (),
        )
    }

    fn update(&mut self, ticks: &mut u32, _: &mut (), msg: Msg) {
        if matches!(msg, Msg::Tick) {
            *ticks += 1;
        }
        self.offer(msg);
    }

    fn view(&self, ticks: &u32) -> (Auth, Sync, u32) {
        (self.auth, self.sync, *ticks)
    }

    fn in_state(&self, id: &'static str) -> bool {
        match id {
            "anon" => self.auth == Auth::Anon,
            "user" => self.auth == Auth::User,
            "idle" => self.sync == Sync::Idle,
            "busy" => self.sync == Sync::Busy,
            _ => false,
        }
    }
}

fn main() {
    println!("# orthogonal  (auth and sync are fields of one struct)\n");
    let (mut rt, _) = Runtime::<Session>::boot(());
    println!("start  {:?}", rt.view());
    rt.apply(Msg::Tick);
    println!("tick   {:?}", rt.view());
    rt.apply(Msg::Login);
    println!("login  {:?}", rt.view());
    rt.apply(Msg::Tick);
    println!("tick   {:?}", rt.view());
    println!(
        "in_state user={} busy={}",
        rt.in_state("user"),
        rt.in_state("busy")
    );
}

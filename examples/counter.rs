//! Smallest Newton machine: Elm TEA, no Harel tree.
//!
//! ```text
//! cargo run --example counter
//! ```

use newton_machine::prelude::*;

struct Counter;

#[derive(Clone, Copy)]
enum Msg {
    Inc,
    Dec,
}

impl Machine for Counter {
    type Flags = ();
    type Model = i32;
    type Msg = Msg;
    type Cmd = ();
    type View = i32;
    type History = ();
    type NodeId = ();

    fn init(_: ()) -> Boot<Self> {
        Boot::new(Counter, 0, (), ())
    }

    fn update(&mut self, model: &mut i32, _: &mut (), msg: Msg) {
        match msg {
            Msg::Inc => *model += 1,
            Msg::Dec => *model -= 1,
        }
    }

    fn view(&self, model: &i32) -> i32 {
        *model
    }

    fn in_state(&self, _: ()) -> bool {
        true
    }
}

fn main() {
    let (mut rt, _) = Runtime::<Counter>::boot(());
    println!("init  {}", rt.view());
    for msg in [Msg::Inc, Msg::Inc, Msg::Dec] {
        rt.apply(msg);
        println!("view  {}", rt.view());
    }
}

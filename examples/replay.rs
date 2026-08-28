//! Persist `{config, context, history}` and restore on another `Runtime`.
//!
//! Live and replay share `update`. The store is RAM; a host would use a file.
//!
//! ```text
//! cargo run --example replay
//! ```

use newton_machine::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Phase {
    Idle,
    Armed,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct Ctx {
    bars: u8,
}

impl Machine for Phase {
    type Flags = ();
    type Model = Ctx;
    type Msg = ();
    type Cmd = ();
    type View = (Phase, u8);
    type History = ();
    type NodeId = Phase;

    fn init(_: ()) -> Boot<Self> {
        Boot::new(Phase::Idle, Ctx::default(), (), ())
    }

    fn update(&mut self, ctx: &mut Ctx, _: &mut (), _: ()) {
        ctx.bars = ctx.bars.saturating_add(1);
        if ctx.bars >= 2 {
            *self = Phase::Armed;
        }
    }

    fn view(&self, ctx: &Ctx) -> (Phase, u8) {
        (*self, ctx.bars)
    }

    fn in_state(&self, id: Phase) -> bool {
        *self == id
    }
}

fn main() {
    println!("# replay  (MemoryStore is the journal)\n");
    let (mut live, _) = Runtime::<Phase>::boot(());
    live.apply(());
    live.apply(());
    println!("live     {:?}", live.view());

    let mut store = MemoryStore::new();
    live.persist(&mut store).expect("in-memory store");

    let mut restored = Runtime::<Phase>::load(&store)
        .expect("in-memory store")
        .expect("snapshot after two bars");
    println!("restored {:?}", restored.view());
    assert_eq!(live.view(), restored.view());
    assert!(restored.in_state(Phase::Armed));

    restored.apply(());
    println!("+1 bar   {:?}", restored.view());
}

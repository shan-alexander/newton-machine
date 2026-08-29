//! Storm: apply panics; try_apply returns Err when try_update is overridden.

use newton_machine::prelude::*;

struct Loop;

impl Machine for Loop {
    type Flags = ();
    type Model = ();
    type Msg = u8;
    type Cmd = ();
    type View = ();
    type History = ();
    type NodeId = ();

    fn init(_: ()) -> Boot<Self> {
        Boot::new(Loop, (), (), ())
    }

    fn try_update(&mut self, _: &mut (), _: &mut (), msg: u8) -> Result<(), Storm> {
        rtc_n::<_, (), _, 4>(msg, |m, inbox| {
            inbox.push(m.wrapping_add(1));
        })
    }

    fn update(&mut self, model: &mut (), history: &mut (), msg: u8) {
        unwrap_storm(self.try_update(model, history, msg));
    }

    fn view(&self, _: &()) {}

    fn in_state(&self, _: ()) -> bool {
        true
    }
}

#[test]
fn try_apply_returns_storm() {
    let (mut rt, _) = Runtime::<Loop>::boot(());
    let err = rt.try_apply(0).unwrap_err();
    assert_eq!(err.cap, 4);
    assert_eq!(err.drained, 4);
}

#[test]
#[should_panic(expected = "internal event storm")]
fn apply_panics_on_storm() {
    let (mut rt, _) = Runtime::<Loop>::boot(());
    rt.apply(0);
}

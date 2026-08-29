//! Macro XOR chart: Topology + IntoNode + #[machine] + perform!
//!
//! The handwritten twin is the toggle in benches/apply.rs. Same kinematics.

use newton_machine::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Topology)]
enum Node {
    #[topology(root)]
    Root,
    #[topology(parent = Root)]
    Off,
    #[topology(parent = Root)]
    On,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, IntoNode)]
#[into_node(Node)]
enum Chart {
    Off,
    On,
}

#[derive(Clone, Copy, Debug)]
enum Msg {
    Toggle,
}

impl Transitional for Chart {
    type Ctx = u32;
    type Hist = ();
    type Cmd = Cmd<u8>;

    fn enter(&mut self, node: Node, ctx: &mut u32, _: &mut ()) -> Cmd<u8> {
        if node == Node::On {
            *ctx += 1;
            Cmd::single(1)
        } else {
            Cmd::none()
        }
    }
}

#[machine(
    model = u32,
    msg = Msg,
    cmd = Cmd<u8>,
    view = bool,
    node_id = Node
)]
impl Chart {
    fn init(_: ()) -> Boot<Self> {
        Boot::new(Chart::Off, 0, (), Cmd::none())
    }

    fn update(&mut self, model: &mut u32, hist: &mut (), msg: Msg) -> Cmd<u8> {
        match (*self, msg) {
            (Chart::Off, Msg::Toggle) => newton_machine::perform!(self, Chart::On, model, hist),
            (Chart::On, Msg::Toggle) => newton_machine::perform!(self, Chart::Off, model, hist),
        }
    }

    fn view(&self, _: &u32) -> bool {
        matches!(self, Chart::On)
    }
}

#[test]
fn toggle_and_in_state() {
    let (mut rt, _) = Runtime::<Chart>::boot(());
    assert!(rt.in_state(Node::Off));
    assert!(rt.in_state(Node::Root));
    assert!(!rt.in_state(Node::On));
    assert!(!rt.view());

    let cmd = rt.apply(Msg::Toggle);
    assert_eq!(cmd.len(), 1);
    assert!(rt.in_state(Node::On));
    assert!(rt.in_state(Node::Root));
    assert!(!rt.in_state(Node::Off));
    assert_eq!(*rt.model(), 1);
    assert!(rt.view());

    let _ = rt.apply(Msg::Toggle);
    assert!(rt.in_state(Node::Off));
    assert!(!rt.view());
}

#[test]
fn configuration_lists_leaf_then_root() {
    let (rt, _) = Runtime::<Chart>::boot(());
    let mut out = [Node::Root; 4];
    let n = rt.configuration(&mut out);
    assert_eq!(&out[..n], &[Node::Off, Node::Root]);
}

//! LCA transition: exit inner-first, install destination, enter outer-first.
//!
//! History is recorded in `exit` of opted-in composites and restored when the
//! author **builds** `dest` (usually from the sidecar, before [`perform`]).
//!
// rustbrain: [[docs/concepts/least-common-ancestor-transition]]
// rustbrain: [[docs/adr/0005-history-as-sidecar]]
// rustbrain: [[docs/edge_cases/cross-region-lca]]

use crate::combine::Combine;
use crate::topology::{paths, Topology};

/// Configuration that can run Harel exit/enter actions around an assignment.
///
/// Separate from [`crate::Machine`]: TEA is the mutation protocol; this is the
/// kinematics of one transition inside `update`.
pub trait Transitional: Topology + Sized {
    /// Extended state (Harel datamodel).
    type Ctx;
    /// History sidecar.
    type Hist;
    /// Collected commands. Usually [`crate::Cmd`].
    type Cmd: Combine;

    /// Exit `node` (inner first). Record history here. Must not perform I/O.
    fn exit(&mut self, node: Self::Node, ctx: &mut Self::Ctx, hist: &mut Self::Hist) -> Self::Cmd {
        let _ = (node, ctx, hist);
        Self::Cmd::none()
    }

    /// Enter `node` (outer first) on the **already installed** destination.
    /// Must not perform I/O.
    fn enter(&mut self, node: Self::Node, ctx: &mut Self::Ctx, hist: &mut Self::Hist) -> Self::Cmd {
        let _ = (node, ctx, hist);
        Self::Cmd::none()
    }
}

/// Exit `from` → LCA, assign `dest`, enter LCA → `dest_node`.
///
/// Build `dest` first (history **reads**). [`Transitional::exit`] may then
/// **write** history for the nodes being left.
#[doc(alias = "LCA")]
#[doc(alias = "Harel")]
pub fn perform<T: Transitional>(
    chart: &mut T,
    from: T::Node,
    dest: T,
    dest_node: T::Node,
    ctx: &mut T::Ctx,
    hist: &mut T::Hist,
) -> T::Cmd {
    let p = paths(from, dest_node, T::parent);
    let mut cmd = T::Cmd::none();
    for n in p.exits.iter() {
        cmd = cmd.combine(chart.exit(n, ctx, hist));
    }
    *chart = dest;
    for n in p.enters.iter() {
        cmd = cmd.combine(chart.enter(n, ctx, hist));
    }
    cmd
}

#[cfg(test)]
mod tests {
    use super::{perform, Transitional};
    use crate::cmd::Cmd;
    use crate::topology::Topology;

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    enum Node {
        Root,
        A,
        B,
        B1,
    }

    #[derive(Clone, Debug, PartialEq, Eq)]
    enum Chart {
        A,
        B { deep: bool },
    }

    impl Chart {
        fn node(&self) -> Node {
            match self {
                Chart::A => Node::A,
                Chart::B { deep: false } => Node::B,
                Chart::B { deep: true } => Node::B1,
            }
        }
    }

    impl Topology for Chart {
        type Node = Node;
        fn parent(node: Node) -> Option<Node> {
            match node {
                Node::Root => None,
                Node::A | Node::B => Some(Node::Root),
                Node::B1 => Some(Node::B),
            }
        }
    }

    #[derive(Default)]
    struct Log {
        buf: [Option<&'static str>; 8],
        n: usize,
    }

    impl Log {
        fn push(&mut self, s: &'static str) {
            self.buf[self.n] = Some(s);
            self.n += 1;
        }

        fn eq(&self, expected: &[&'static str]) -> bool {
            if self.n != expected.len() {
                return false;
            }
            self.buf[..self.n]
                .iter()
                .zip(expected)
                .all(|(a, b)| *a == Some(*b))
        }
    }

    impl Transitional for Chart {
        type Ctx = Log;
        type Hist = ();
        type Cmd = Cmd<()>;

        fn exit(&mut self, node: Node, ctx: &mut Log, _: &mut ()) -> Cmd<()> {
            ctx.push(match node {
                Node::A => "exit A",
                Node::B => "exit B",
                Node::B1 => "exit B1",
                Node::Root => "exit Root",
            });
            Cmd::none()
        }

        fn enter(&mut self, node: Node, ctx: &mut Log, _: &mut ()) -> Cmd<()> {
            ctx.push(match node {
                Node::A => "enter A",
                Node::B => "enter B",
                Node::B1 => "enter B1",
                Node::Root => "enter Root",
            });
            Cmd::none()
        }
    }

    #[test]
    fn sibling_exit_then_enter() {
        let mut chart = Chart::A;
        let mut log = Log::default();
        let dest = Chart::B { deep: false };
        let to = dest.node();
        let _ = perform(&mut chart, Node::A, dest, to, &mut log, &mut ());
        assert!(log.eq(&["exit A", "enter B"]));
        assert_eq!(chart, Chart::B { deep: false });
    }

    #[test]
    fn descend_enters_outer_first() {
        let mut chart = Chart::A;
        let mut log = Log::default();
        let dest = Chart::B { deep: true };
        let _ = perform(&mut chart, Node::A, dest, Node::B1, &mut log, &mut ());
        assert!(log.eq(&["exit A", "enter B", "enter B1"]));
    }

    #[test]
    fn ascend_exits_inner_first() {
        let mut chart = Chart::B { deep: true };
        let mut log = Log::default();
        let _ = perform(&mut chart, Node::B1, Chart::A, Node::A, &mut log, &mut ());
        assert!(log.eq(&["exit B1", "exit B", "enter A"]));
    }
}

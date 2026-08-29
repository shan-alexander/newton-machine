//! Chart tree as node ids: parent, LCA, exit and enter paths.
//!
//! Cost is O(depth), not O(number of states). No string table on the hot path.
//!
// rustbrain: [[docs/concepts/least-common-ancestor-transition]]
// rustbrain: [[docs/adr/0002-xor-enums-and-and-structs]]

/// Maximum ancestor walk. Charts deeper than this panic: a truncated walk
/// would compute the wrong LCA.
pub const MAX_DEPTH: usize = 32;

/// A short path of node ids, stored on the stack.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Chain<N: Copy> {
    nodes: [Option<N>; MAX_DEPTH],
    len: usize,
}

impl<N: Copy> Chain<N> {
    const fn empty() -> Self {
        Self {
            nodes: [None; MAX_DEPTH],
            len: 0,
        }
    }

    fn push(&mut self, node: N) {
        assert!(
            self.len < MAX_DEPTH,
            "topology deeper than MAX_DEPTH ({MAX_DEPTH}): ancestor walk truncated, LCA would be wrong"
        );
        self.nodes[self.len] = Some(node);
        self.len += 1;
    }

    /// Number of nodes in the chain.
    #[inline]
    pub const fn len(&self) -> usize {
        self.len
    }

    /// True when the chain is empty (source and target share the LCA only).
    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Iterate inner-first or outer-first depending on how the chain was built.
    pub fn iter(&self) -> impl Iterator<Item = N> + '_ {
        self.nodes[..self.len].iter().copied().flatten()
    }

    /// True if `node` appears in this chain.
    pub fn contains(&self, node: N) -> bool
    where
        N: PartialEq,
    {
        self.iter().any(|n| n == node)
    }

    fn reverse(&mut self) {
        let mut i = 0;
        let mut j = self.len.saturating_sub(1);
        while i < j {
            self.nodes.swap(i, j);
            i += 1;
            j -= 1;
        }
    }
}

/// Walk `node`, then its parent, then its grandparent, stopping at the root.
pub fn ancestors<N, F>(mut node: N, parent: F) -> Chain<N>
where
    N: Copy,
    F: Fn(N) -> Option<N>,
{
    let mut chain = Chain::empty();
    loop {
        chain.push(node);
        match parent(node) {
            Some(p) => node = p,
            None => break,
        }
    }
    chain
}

/// Least common ancestor of `a` and `b` under `parent`.
///
/// The deepest node that is an ancestor of both (`a` is an ancestor of `a`).
/// [`crate::perform`] exits source → LCA (not including LCA) and enters
/// LCA → target (not including LCA), so a parent you are staying in is not
/// exited. If the nodes are disjoint (a broken topology), returns the root of
/// `b`.
pub fn lca<N, F>(a: N, b: N, parent: F) -> N
where
    N: Copy + PartialEq,
    F: Fn(N) -> Option<N>,
{
    let a_anc = ancestors(a, &parent);
    let mut x = b;
    loop {
        if a_anc.contains(x) {
            return x;
        }
        match parent(x) {
            Some(p) => x = p,
            None => return x,
        }
    }
}

/// Nodes to exit: `from` toward `lca`, inner first, **excluding** `lca`.
pub fn exit_path<N, F>(from: N, lca_node: N, parent: F) -> Chain<N>
where
    N: Copy + PartialEq,
    F: Fn(N) -> Option<N>,
{
    let mut chain = Chain::empty();
    let mut x = from;
    while x != lca_node {
        chain.push(x);
        match parent(x) {
            Some(p) => x = p,
            None => break,
        }
    }
    chain
}

/// Nodes to enter: `lca` toward `to`, outer first, **excluding** `lca`.
pub fn enter_path<N, F>(lca_node: N, to: N, parent: F) -> Chain<N>
where
    N: Copy + PartialEq,
    F: Fn(N) -> Option<N>,
{
    let mut chain = exit_path(to, lca_node, parent);
    chain.reverse();
    chain
}

/// Precomputed exit/enter chains for one transition.
#[derive(Clone, Copy, Debug)]
pub struct Paths<N: Copy> {
    /// Least common ancestor. Not exited, not re-entered.
    pub lca: N,
    /// Inner first, excluding [`Paths::lca`].
    pub exits: Chain<N>,
    /// Outer first, excluding [`Paths::lca`].
    pub enters: Chain<N>,
}

/// Compute [`Paths`] for a transition from `from` to `to`.
pub fn paths<N, F>(from: N, to: N, parent: F) -> Paths<N>
where
    N: Copy + PartialEq,
    F: Fn(N) -> Option<N>,
{
    let lca_node = lca(from, to, &parent);
    Paths {
        lca: lca_node,
        exits: exit_path(from, lca_node, &parent),
        enters: enter_path(lca_node, to, parent),
    }
}

/// A chart tree: each node has at most one parent.
///
/// Implement on the configuration type (`enum` XOR / `struct` AND).
pub trait Topology {
    /// Compact node id. Prefer a `Copy` enum, not a `String`.
    type Node: Copy + PartialEq;

    /// Parent in the chart. The root returns `None`.
    fn parent(node: Self::Node) -> Option<Self::Node>;

    /// [`lca`] using [`Topology::parent`].
    fn lca(a: Self::Node, b: Self::Node) -> Self::Node {
        lca(a, b, Self::parent)
    }

    /// [`paths`] using [`Topology::parent`].
    fn paths(from: Self::Node, to: Self::Node) -> Paths<Self::Node> {
        paths(from, to, Self::parent)
    }
}

#[cfg(test)]
mod tests {
    use super::{enter_path, exit_path, lca, paths, Topology};

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    enum N {
        Root,
        Off,
        On,
        Auth,
        Anon,
        In,
        Sync,
        Idle,
        Dirty,
    }

    fn parent(n: N) -> Option<N> {
        use N::*;
        match n {
            Root => None,
            Off | On => Some(Root),
            Auth | Sync => Some(On),
            Anon | In => Some(Auth),
            Idle | Dirty => Some(Sync),
        }
    }

    struct Tree;
    impl Topology for Tree {
        type Node = N;
        fn parent(node: N) -> Option<N> {
            parent(node)
        }
    }

    #[test]
    fn lca_self() {
        assert_eq!(lca(N::On, N::On, parent), N::On);
    }

    #[test]
    fn lca_siblings() {
        assert_eq!(lca(N::Off, N::On, parent), N::Root);
    }

    #[test]
    fn lca_deep() {
        assert_eq!(lca(N::Anon, N::In, parent), N::Auth);
        assert_eq!(lca(N::Anon, N::Dirty, parent), N::On);
        assert_eq!(lca(N::Off, N::In, parent), N::Root);
        assert_eq!(lca(N::Idle, N::Dirty, parent), N::Sync);
    }

    #[test]
    fn exit_inner_first() {
        let chain = exit_path(N::In, N::Root, parent);
        let mut it = chain.iter();
        assert_eq!(it.next(), Some(N::In));
        assert_eq!(it.next(), Some(N::Auth));
        assert_eq!(it.next(), Some(N::On));
        assert_eq!(it.next(), None);
    }

    #[test]
    fn enter_outer_first() {
        let chain = enter_path(N::Root, N::In, parent);
        let mut it = chain.iter();
        assert_eq!(it.next(), Some(N::On));
        assert_eq!(it.next(), Some(N::Auth));
        assert_eq!(it.next(), Some(N::In));
        assert_eq!(it.next(), None);
    }

    #[test]
    fn same_node_empty_paths() {
        let p = paths(N::On, N::On, parent);
        assert!(p.exits.is_empty());
        assert!(p.enters.is_empty());
        assert_eq!(p.lca, N::On);
    }

    #[test]
    fn topology_trait_matches_free_fns() {
        assert_eq!(Tree::lca(N::Anon, N::Dirty), N::On);
    }
}

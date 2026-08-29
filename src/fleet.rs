//! Many runtimes, one message vocabulary. Not an `And` of N machines.
//!
//! Orthogonal regions ([`And`](crate::And)`<L,R>`) are **one** configuration
//! with several XOR children. Fifty connections, devices, or entities are
//! **fifty** configurations. Putting them in one `And` tree is the lattice
//! this crate refused. A [`Fleet`] is a `BTreeMap` of
//! [`Runtime`] values sharing `M::Msg`.
//!
//! The host still classifies and still executes `Cmd`. This type only
//! owns the triples so one member’s history is not dropped when another
//! member receives a message.
//!
// rustbrain: [[docs/adr/0023-chord-table-is-host-policy]]
// rustbrain: [[docs/adr/0007-virtual-concurrency-not-threads]]

use alloc::collections::BTreeMap;

use crate::machine::Machine;
use crate::runtime::Runtime;

/// N independent Newton machines, keyed by the host (`id`, `session`, `device`, …).
pub struct Fleet<K, M: Machine> {
    inner: BTreeMap<K, Runtime<M>>,
}

impl<K: Ord, M: Machine> Fleet<K, M> {
    /// Empty fleet.
    pub const fn new() -> Self {
        Self {
            inner: BTreeMap::new(),
        }
    }

    /// Boot `key` from flags. Replaces a previous runtime for that key.
    pub fn boot(&mut self, key: K, flags: M::Flags) -> M::Cmd {
        let (rt, cmd) = Runtime::boot(flags);
        self.inner.insert(key, rt);
        cmd
    }

    /// Apply `msg` to one member. `None` if `key` was never booted.
    pub fn apply(&mut self, key: &K, msg: M::Msg) -> Option<M::Cmd> {
        self.inner.get_mut(key).map(|rt| rt.apply(msg))
    }

    /// [`Runtime::apply_if`](crate::Runtime::apply_if) on one member.
    pub fn apply_if(&mut self, key: &K, gate: bool, msg: M::Msg) -> Option<M::Cmd> {
        self.inner
            .get_mut(key)
            .and_then(|rt| rt.apply_if(gate, msg))
    }

    /// Borrow one runtime.
    pub fn get(&self, key: &K) -> Option<&Runtime<M>> {
        self.inner.get(key)
    }

    /// Borrow one runtime mutably (restore, persist).
    pub fn get_mut(&mut self, key: &K) -> Option<&mut Runtime<M>> {
        self.inner.get_mut(key)
    }

    /// Drop a member.
    pub fn remove(&mut self, key: &K) -> Option<Runtime<M>> {
        self.inner.remove(key)
    }

    /// How many runtimes.
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// True when no members.
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Iterate in key order.
    pub fn iter(&self) -> alloc::collections::btree_map::Iter<'_, K, Runtime<M>> {
        self.inner.iter()
    }
}

impl<K: Ord, M: Machine> Default for Fleet<K, M> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::Fleet;
    use crate::cmd::Cmd;
    use crate::machine::{Boot, Machine};

    struct Tick;

    impl Machine for Tick {
        type Flags = ();
        type Model = u32;
        type Msg = ();
        type Cmd = Cmd<()>;
        type View = u32;
        type History = ();
        type NodeId = ();

        fn init(_: ()) -> Boot<Self> {
            Boot::new(Tick, 0, (), Cmd::none())
        }

        fn update(&mut self, n: &mut u32, _: &mut (), _: ()) -> Cmd<()> {
            *n += 1;
            Cmd::none()
        }

        fn view(&self, n: &u32) -> u32 {
            *n
        }

        fn in_state(&self, _: ()) -> bool {
            true
        }
    }

    #[test]
    fn two_symbols_do_not_share_model() {
        let mut f = Fleet::<&str, Tick>::new();
        let _ = f.boot("east", ());
        let _ = f.boot("west", ());
        f.apply(&"east", ());
        f.apply(&"east", ());
        f.apply(&"west", ());
        assert_eq!(f.get(&"east").unwrap().view(), 2);
        assert_eq!(f.get(&"west").unwrap().view(), 1);
    }

    #[test]
    fn apply_missing_is_none() {
        let mut f = Fleet::<&str, Tick>::new();
        assert!(f.apply(&"east", ()).is_none());
    }
}

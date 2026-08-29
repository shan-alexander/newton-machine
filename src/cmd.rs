//! Commands as data. The host executes them; `update` never performs I/O.
//!
//! GoF Command without objects or closures: an ordered bag of atoms `C`.
//! Concatenation is [`Cmd::and`]. Up to [`INLINE_CAP`] atoms live on the
//! stack so `perform` (exit + enter) never needs a heap. More than that
//! spills to `Vec` when `alloc` is on, and **panics** when it is not —
//! silent drop would violate the conservation law.
//!
// rustbrain: [[docs/concepts/command-as-data]]
// rustbrain: [[docs/adr/0004-commands-as-data]]
// rustbrain: [[docs/adr/0018-cmd-inline-then-heap]]
// rustbrain: [[docs/goals/effects-never-leave-the-host]]

#[cfg(feature = "alloc")]
use alloc::vec::Vec;

/// Stack capacity before [`Cmd::and`] needs a heap (`alloc`) or panics.
///
/// Two is the common `perform` case (exit cmd + enter cmd). Four covers a
/// nested enter path without `Vec`.
pub const INLINE_CAP: usize = 4;

/// `Cmd::and` needed more atoms than [`INLINE_CAP`] and `alloc` is off.
///
/// Programmer error at the call site (too many effects in one step on a
/// no-heap build). [`Cmd::and`] panics with this message. [`Cmd::try_and`]
/// returns it so serde / bulk builders can fail without aborting.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CmdOverflow {
    /// Stack cap ([`INLINE_CAP`]).
    pub cap: usize,
    /// Atoms that would have been in the result.
    pub attempted: usize,
}

impl core::fmt::Display for CmdOverflow {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "Cmd::and needs {} atoms; stack cap is {} (enable feature `alloc` for a heap batch, or emit fewer effects per step)",
            self.attempted, self.cap
        )
    }
}

#[cfg(feature = "std")]
impl std::error::Error for CmdOverflow {}

#[derive(Clone)]
enum Inner<C> {
    Inline {
        slots: [Option<C>; INLINE_CAP],
        len: u8,
    },
    #[cfg(feature = "alloc")]
    Heap(Vec<C>),
}

/// A description of work the host should perform after a run-to-completion step.
///
/// `C` is the author's command vocabulary (`Submit`, `Persist`, `Alert`, …).
/// Nothing in this type can open a socket. Representation (stack vs heap) is
/// private: compare and iterate atoms, do not match variants.
#[must_use]
#[derive(Clone)]
pub struct Cmd<C> {
    inner: Inner<C>,
}

impl<C> Cmd<C> {
    /// No effect.
    #[inline]
    pub const fn none() -> Self {
        Self {
            inner: Inner::Inline {
                slots: [None, None, None, None],
                len: 0,
            },
        }
    }

    /// One command.
    #[inline]
    pub const fn single(cmd: C) -> Self {
        Self {
            inner: Inner::Inline {
                slots: [Some(cmd), None, None, None],
                len: 1,
            },
        }
    }

    /// True when this command describes no work.
    #[inline]
    pub fn is_none(&self) -> bool {
        self.is_empty()
    }

    /// How many atoms will the host execute.
    pub fn len(&self) -> usize {
        match &self.inner {
            Inner::Inline { len, .. } => *len as usize,
            #[cfg(feature = "alloc")]
            Inner::Heap(v) => v.len(),
        }
    }

    /// True when [`Cmd::len`] is 0.
    #[inline]
    pub fn is_empty(&self) -> bool {
        match &self.inner {
            Inner::Inline { len, .. } => *len == 0,
            #[cfg(feature = "alloc")]
            Inner::Heap(v) => v.is_empty(),
        }
    }

    /// Iterate atoms in document order.
    pub fn iter(&self) -> Iter<'_, C> {
        match &self.inner {
            Inner::Inline { slots, len } => Iter {
                kind: IterKind::Inline(slots[..*len as usize].iter()),
            },
            #[cfg(feature = "alloc")]
            Inner::Heap(v) => Iter {
                kind: IterKind::Heap(v.iter()),
            },
        }
    }

    /// Concatenate in document order. [`Cmd::none`] is the identity.
    ///
    /// Panics if the result would exceed [`INLINE_CAP`] and the `alloc`
    /// feature is off. That is a chart/host bug, not a dropped intent.
    pub fn and(self, other: Self) -> Self {
        match self.try_and(other) {
            Ok(cmd) => cmd,
            Err(e) => panic!("{e}"),
        }
    }

    /// Fallible [`Cmd::and`]. Does not panic.
    pub fn try_and(self, other: Self) -> Result<Self, CmdOverflow> {
        if self.is_empty() {
            return Ok(other);
        }
        if other.is_empty() {
            return Ok(self);
        }
        let attempted = self.len() + other.len();
        if attempted <= INLINE_CAP {
            let mut slots: [Option<C>; INLINE_CAP] = [None, None, None, None];
            let mut i = 0usize;
            for a in self {
                slots[i] = Some(a);
                i += 1;
            }
            for a in other {
                slots[i] = Some(a);
                i += 1;
            }
            return Ok(Self {
                inner: Inner::Inline {
                    slots,
                    len: i as u8,
                },
            });
        }
        #[cfg(feature = "alloc")]
        {
            let mut v = Vec::with_capacity(attempted);
            v.extend(self);
            v.extend(other);
            Ok(Self {
                inner: Inner::Heap(v),
            })
        }
        #[cfg(not(feature = "alloc"))]
        {
            Err(CmdOverflow {
                cap: INLINE_CAP,
                attempted,
            })
        }
    }
}

impl<C> Default for Cmd<C> {
    fn default() -> Self {
        Self::none()
    }
}

impl<C> From<C> for Cmd<C> {
    fn from(cmd: C) -> Self {
        Self::single(cmd)
    }
}

impl<C: PartialEq> PartialEq for Cmd<C> {
    fn eq(&self, other: &Self) -> bool {
        self.len() == other.len() && self.iter().eq(other.iter())
    }
}

impl<C: Eq> Eq for Cmd<C> {}

impl<C: core::fmt::Debug> core::fmt::Debug for Cmd<C> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_list().entries(self.iter()).finish()
    }
}

enum IterKind<'a, C> {
    Inline(core::slice::Iter<'a, Option<C>>),
    #[cfg(feature = "alloc")]
    Heap(core::slice::Iter<'a, C>),
}

/// Borrowed iterator over [`Cmd`] atoms.
pub struct Iter<'a, C> {
    kind: IterKind<'a, C>,
}

impl<'a, C> Iterator for Iter<'a, C> {
    type Item = &'a C;

    fn next(&mut self) -> Option<Self::Item> {
        match &mut self.kind {
            IterKind::Inline(it) => it.next().and_then(|s| s.as_ref()),
            #[cfg(feature = "alloc")]
            IterKind::Heap(it) => it.next(),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let n = self.len();
        (n, Some(n))
    }
}

impl<C> ExactSizeIterator for Iter<'_, C> {
    fn len(&self) -> usize {
        match &self.kind {
            IterKind::Inline(it) => it.len(),
            #[cfg(feature = "alloc")]
            IterKind::Heap(it) => it.len(),
        }
    }
}

enum IntoKind<C> {
    Inline {
        slots: [Option<C>; INLINE_CAP],
        i: u8,
        len: u8,
    },
    #[cfg(feature = "alloc")]
    Heap(alloc::vec::IntoIter<C>),
}

/// Owned iterator over [`Cmd`] atoms.
pub struct IntoIter<C> {
    kind: IntoKind<C>,
}

impl<C> Iterator for IntoIter<C> {
    type Item = C;

    fn next(&mut self) -> Option<Self::Item> {
        match &mut self.kind {
            IntoKind::Inline { slots, i, len } => {
                if *i >= *len {
                    return None;
                }
                let item = slots[*i as usize].take();
                *i += 1;
                item
            }
            #[cfg(feature = "alloc")]
            IntoKind::Heap(it) => it.next(),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let n = match &self.kind {
            IntoKind::Inline { i, len, .. } => (*len).saturating_sub(*i) as usize,
            #[cfg(feature = "alloc")]
            IntoKind::Heap(it) => it.len(),
        };
        (n, Some(n))
    }
}

impl<C> ExactSizeIterator for IntoIter<C> {}

impl<C> IntoIterator for Cmd<C> {
    type Item = C;
    type IntoIter = IntoIter<C>;

    fn into_iter(self) -> Self::IntoIter {
        match self.inner {
            Inner::Inline { slots, len } => IntoIter {
                kind: IntoKind::Inline { slots, i: 0, len },
            },
            #[cfg(feature = "alloc")]
            Inner::Heap(v) => IntoIter {
                kind: IntoKind::Heap(v.into_iter()),
            },
        }
    }
}

#[cfg(feature = "serde")]
mod serde_impl {
    use super::Cmd;
    use serde::de::{SeqAccess, Visitor};
    use serde::ser::SerializeSeq;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    impl<C: Serialize> Serialize for Cmd<C> {
        fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
            let mut seq = serializer.serialize_seq(Some(self.len()))?;
            for a in self.iter() {
                seq.serialize_element(a)?;
            }
            seq.end()
        }
    }

    impl<'de, C: Deserialize<'de>> Deserialize<'de> for Cmd<C> {
        fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
            struct CmdVisitor<C>(core::marker::PhantomData<C>);

            impl<'de, C: Deserialize<'de>> Visitor<'de> for CmdVisitor<C> {
                type Value = Cmd<C>;

                fn expecting(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                    f.write_str("a sequence of command atoms")
                }

                fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Cmd<C>, A::Error> {
                    let mut cmd = Cmd::none();
                    while let Some(atom) = seq.next_element()? {
                        cmd = cmd
                            .try_and(Cmd::single(atom))
                            .map_err(serde::de::Error::custom)?;
                    }
                    Ok(cmd)
                }
            }

            deserializer.deserialize_seq(CmdVisitor(core::marker::PhantomData))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Cmd, INLINE_CAP};

    #[test]
    fn none_is_identity() {
        let a = Cmd::single(1u8);
        assert_eq!(Cmd::none().and(a.clone()), a);
        assert_eq!(a.clone().and(Cmd::none()), a);
    }

    #[test]
    fn and_fits_on_stack() {
        let cmd = Cmd::single(1).and(Cmd::single(2)).and(Cmd::single(3));
        assert_eq!(cmd.len(), 3);
        let mut it = cmd.iter();
        assert_eq!(it.next(), Some(&1));
        assert_eq!(it.next(), Some(&2));
        assert_eq!(it.next(), Some(&3));
        assert_eq!(it.next(), None);
    }

    #[test]
    fn four_fit() {
        let mut cmd = Cmd::none();
        for i in 0..INLINE_CAP {
            cmd = cmd.and(Cmd::single(i as u8));
        }
        assert_eq!(cmd.len(), INLINE_CAP);
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn five_spills_to_heap() {
        let mut cmd = Cmd::none();
        for i in 0..=INLINE_CAP {
            cmd = cmd.and(Cmd::single(i as u8));
        }
        assert_eq!(cmd.len(), INLINE_CAP + 1);
        assert_eq!(cmd.iter().copied().count(), INLINE_CAP + 1);
    }

    #[cfg(not(feature = "alloc"))]
    #[test]
    #[should_panic(expected = "stack cap")]
    fn five_panics_without_heap() {
        let mut cmd = Cmd::none();
        for i in 0..=INLINE_CAP {
            cmd = cmd.and(Cmd::single(i as u8));
        }
    }
}

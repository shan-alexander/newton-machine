//! Commands as data. The host executes them; `update` never performs I/O.
//!
//! This is Elm's `Cmd` and also the conservation law that keeps snapshots
//! serializable: functions are not part of the phase space.

// rustbrain: [[docs/concepts/command-as-data]]
// rustbrain: [[docs/adr/0004-commands-as-data]]
// rustbrain: [[docs/goals/effects-never-leave-the-host]]
// rustbrain: [[docs/adr/0011-core-crate-is-not-a-broker]]

#[cfg(feature = "alloc")]
use alloc::vec::Vec;

/// A description of work the host should perform after a run-to-completion step.
///
/// `C` is the author's command vocabulary (`Submit`, `Persist`, `Alert`, …).
/// Nothing in this type can open a socket.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[must_use]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Cmd<C> {
    /// No effect.
    #[default]
    None,
    /// One command.
    Single(C),
    /// Several commands, in document order, still unexecuted.
    #[cfg(feature = "alloc")]
    #[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
    Batch(Vec<C>),
}

impl<C> Cmd<C> {
    /// No effect.
    #[inline]
    pub const fn none() -> Self {
        Self::None
    }

    /// One command.
    #[inline]
    pub const fn single(cmd: C) -> Self {
        Self::Single(cmd)
    }

    /// True when this command describes no work.
    #[inline]
    pub const fn is_none(&self) -> bool {
        matches!(self, Self::None)
    }

    /// Iterate atoms in document order.
    pub fn iter(&self) -> Iter<'_, C> {
        match self {
            Self::None => Iter {
                one: None,
                #[cfg(feature = "alloc")]
                batch: None,
            },
            Self::Single(c) => Iter {
                one: Some(c),
                #[cfg(feature = "alloc")]
                batch: None,
            },
            #[cfg(feature = "alloc")]
            Self::Batch(v) => Iter {
                one: None,
                batch: Some(v.iter()),
            },
        }
    }

    /// How many atoms will the host execute.
    pub fn len(&self) -> usize {
        match self {
            Self::None => 0,
            Self::Single(_) => 1,
            #[cfg(feature = "alloc")]
            Self::Batch(v) => v.len(),
        }
    }

    /// True when [`Cmd::len`] is 0.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Combine two commands without executing either.
    ///
    /// `None` is the identity. Two non-empty commands become a batch when
    /// the `alloc` feature is enabled; without `alloc`, the right-hand
    /// command is kept only if the left is `None`.
    pub fn and(self, other: Self) -> Self {
        match (self, other) {
            (Self::None, other) => other,
            (this, Self::None) => this,
            #[cfg(feature = "alloc")]
            (Self::Single(a), Self::Single(b)) => Self::Batch(alloc::vec![a, b]),
            #[cfg(feature = "alloc")]
            (Self::Single(a), Self::Batch(mut rest)) => {
                rest.insert(0, a);
                Self::Batch(rest)
            }
            #[cfg(feature = "alloc")]
            (Self::Batch(mut left), Self::Single(b)) => {
                left.push(b);
                Self::Batch(left)
            }
            #[cfg(feature = "alloc")]
            (Self::Batch(mut left), Self::Batch(right)) => {
                left.extend(right);
                Self::Batch(left)
            }
            #[cfg(not(feature = "alloc"))]
            (this, _) => this,
        }
    }
}

impl<C> From<C> for Cmd<C> {
    fn from(cmd: C) -> Self {
        Self::Single(cmd)
    }
}

/// Borrowed iterator over [`Cmd`] atoms.
pub struct Iter<'a, C> {
    one: Option<&'a C>,
    #[cfg(feature = "alloc")]
    batch: Option<core::slice::Iter<'a, C>>,
}

impl<'a, C> Iterator for Iter<'a, C> {
    type Item = &'a C;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(c) = self.one.take() {
            return Some(c);
        }
        #[cfg(feature = "alloc")]
        if let Some(ref mut batch) = self.batch {
            return batch.next();
        }
        None
    }
}

/// Owned iterator over [`Cmd`] atoms.
pub struct IntoIter<C> {
    one: Option<C>,
    #[cfg(feature = "alloc")]
    batch: Option<alloc::vec::IntoIter<C>>,
}

impl<C> Iterator for IntoIter<C> {
    type Item = C;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(c) = self.one.take() {
            return Some(c);
        }
        #[cfg(feature = "alloc")]
        if let Some(ref mut batch) = self.batch {
            return batch.next();
        }
        None
    }
}

impl<C> IntoIterator for Cmd<C> {
    type Item = C;
    type IntoIter = IntoIter<C>;

    fn into_iter(self) -> Self::IntoIter {
        match self {
            Self::None => IntoIter {
                one: None,
                #[cfg(feature = "alloc")]
                batch: None,
            },
            Self::Single(c) => IntoIter {
                one: Some(c),
                #[cfg(feature = "alloc")]
                batch: None,
            },
            #[cfg(feature = "alloc")]
            Self::Batch(v) => IntoIter {
                one: None,
                batch: Some(v.into_iter()),
            },
        }
    }
}

#[cfg(all(test, feature = "alloc"))]
mod tests {
    use super::Cmd;

    #[test]
    fn none_is_identity() {
        let a = Cmd::single(1);
        assert_eq!(Cmd::none().and(a.clone()), a);
        assert_eq!(a.clone().and(Cmd::none()), a);
    }

    #[test]
    fn and_batches() {
        let cmd = Cmd::single(1).and(Cmd::single(2)).and(Cmd::single(3));
        assert_eq!(cmd, Cmd::Batch(alloc::vec![1, 2, 3]));
    }
}

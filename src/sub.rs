//! Subscriptions as data. Listeners exist only while the configuration needs them.
//!
//! Elm already got this right: `subscriptions` is a pure function of the current
//! model. When a Newton machine is `Locked`, the host should see a smaller `Sub`
//! and drop timers it no longer requested.
//!
// rustbrain: [[docs/concepts/subscriptions]]

#[cfg(feature = "alloc")]
use alloc::vec::Vec;

/// A description of ongoing listeners the host should maintain.
///
/// `L` is the author's listener vocabulary (`BarFeed`, `FillFeed`, `Clock`, …).
/// The runtime maps listeners to `Msg` values. This type never holds closures.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Sub<L> {
    /// Listen to nothing.
    #[default]
    None,
    /// One listener.
    Single(L),
    /// Several listeners.
    #[cfg(feature = "alloc")]
    #[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
    Many(Vec<L>),
}

impl<L> Sub<L> {
    /// Listen to nothing.
    #[inline]
    pub const fn none() -> Self {
        Self::None
    }

    /// One listener.
    #[inline]
    pub const fn single(listener: L) -> Self {
        Self::Single(listener)
    }

    /// Several listeners. Without `alloc`, only `single` / `none` exist.
    #[cfg(feature = "alloc")]
    pub fn many(items: impl IntoIterator<Item = L>) -> Self {
        let v: alloc::vec::Vec<L> = items.into_iter().collect();
        match v.len() {
            0 => Self::None,
            1 => Self::Single(v.into_iter().next().expect("len 1")),
            _ => Self::Many(v),
        }
    }

    /// Borrowed listeners in document order.
    pub fn iter(&self) -> SubIter<'_, L> {
        match self {
            Self::None => SubIter {
                one: None,
                #[cfg(feature = "alloc")]
                rest: None,
            },
            Self::Single(l) => SubIter {
                one: Some(l),
                #[cfg(feature = "alloc")]
                rest: None,
            },
            #[cfg(feature = "alloc")]
            Self::Many(v) => SubIter {
                one: None,
                rest: Some(v.iter()),
            },
        }
    }

    /// True when no listeners are requested.
    #[inline]
    pub const fn is_none(&self) -> bool {
        matches!(self, Self::None)
    }
}

/// Borrowed iterator over [`Sub`] listeners.
pub struct SubIter<'a, L> {
    one: Option<&'a L>,
    #[cfg(feature = "alloc")]
    rest: Option<core::slice::Iter<'a, L>>,
}

impl<'a, L> Iterator for SubIter<'a, L> {
    type Item = &'a L;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(l) = self.one.take() {
            return Some(l);
        }
        #[cfg(feature = "alloc")]
        if let Some(ref mut rest) = self.rest {
            return rest.next();
        }
        None
    }
}

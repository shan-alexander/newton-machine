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

    /// How many listeners the host should maintain.
    pub fn len(&self) -> usize {
        match self {
            Self::None => 0,
            Self::Single(_) => 1,
            #[cfg(feature = "alloc")]
            Self::Many(v) => v.len(),
        }
    }

    /// True when [`Sub::len`] is 0.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Concatenate listener bags. Two non-empty bags require `alloc`.
    pub fn and(self, other: Self) -> Self {
        match (self, other) {
            (Self::None, x) | (x, Self::None) => x,
            #[cfg(feature = "alloc")]
            (a, b) => {
                let mut v = Vec::new();
                match a {
                    Self::None => {}
                    Self::Single(x) => v.push(x),
                    Self::Many(xs) => v.extend(xs),
                }
                match b {
                    Self::None => {}
                    Self::Single(x) => v.push(x),
                    Self::Many(xs) => v.extend(xs),
                }
                Self::many(v)
            }
            #[cfg(not(feature = "alloc"))]
            (Self::Single(_), Self::Single(_)) => {
                panic!("Sub::and of two listeners needs feature `alloc`")
            }
        }
    }

    /// True if `needle` is among the requested listeners.
    pub fn contains(&self, needle: &L) -> bool
    where
        L: PartialEq,
    {
        self.iter().any(|l| l == needle)
    }

    /// Elm host diff: listeners to **start** (`new` \ `self`) and **stop**
    /// (`self` \ `new`). Equality is [`PartialEq`] on `L`, not pointer
    /// identity. Duplicates are treated as a set.
    ///
    /// More than one start or stop atom at once needs `alloc` (same as
    /// [`Sub::and`]).
    pub fn diff(&self, new: &Self) -> Diff<L>
    where
        L: PartialEq + Clone,
    {
        let mut start = Self::none();
        let mut stop = Self::none();
        for l in new.iter() {
            if !self.contains(l) {
                start = start.and(Self::single(l.clone()));
            }
        }
        for l in self.iter() {
            if !new.contains(l) {
                stop = stop.and(Self::single(l.clone()));
            }
        }
        Diff { start, stop }
    }
}

/// Result of [`Sub::diff`]: host should start `start` and stop `stop`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Diff<L> {
    /// In `new` but not in the previous bag.
    pub start: Sub<L>,
    /// In the previous bag but not in `new`.
    pub stop: Sub<L>,
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

#[cfg(test)]
mod tests {
    use super::Sub;

    #[test]
    fn diff_none_to_one() {
        let old = Sub::<&str>::none();
        let new = Sub::single("clock");
        let d = old.diff(&new);
        assert_eq!(d.start, Sub::single("clock"));
        assert!(d.stop.is_none());
    }

    #[test]
    fn diff_one_to_none() {
        let old = Sub::single("clock");
        let new = Sub::<&str>::none();
        let d = old.diff(&new);
        assert!(d.start.is_none());
        assert_eq!(d.stop, Sub::single("clock"));
    }

    #[test]
    fn diff_unchanged() {
        let s = Sub::single("clock");
        let d = s.diff(&s);
        assert!(d.start.is_none());
        assert!(d.stop.is_none());
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn diff_swap() {
        let old = Sub::many(["clock", "feed"]);
        let new = Sub::many(["feed", "keys"]);
        let d = old.diff(&new);
        assert!(d.start.contains(&"keys"));
        assert!(!d.start.contains(&"feed"));
        assert!(d.stop.contains(&"clock"));
        assert!(!d.stop.contains(&"feed"));
    }
}

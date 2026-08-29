//! Compact configuration key. Not a Harel node, not a string id.
//!
//! A Newton machine names **what is true** as nested ADTs. A host policy
//! table ([`ChordTable`](crate::ChordTable), or the host’s own map) indexes
//! a bitset projected from that configuration. [`Bits`] is that projection:
//! up to 128 independent flags, `Copy`, no heap.
//!
//! Orthogonal XOR children occupy **disjoint** bit ranges. Two children of
//! one XOR must not both be set — that would reconstruct the illegal
//! simultaneous XOR the type system already forbids. The crate does not
//! check that; the author’s [`Machine::project`](crate::Machine::project)
//! mapping is the contract.
//!
// rustbrain: [[docs/concepts/chord-and-superstate]]
// rustbrain: [[docs/adr/0023-chord-table-is-host-policy]]

/// Compact configuration key (`u128` bitmask).
///
/// Bit *i* is “flag *i* is in the live pool.” Interpretation of *i* is
/// the author’s (XOR child, host classifier, overlay id). The crate
/// never interns strings here.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Bits(u128);

impl Bits {
    /// No flags.
    pub const EMPTY: Self = Self(0);

    /// Wrap a raw mask.
    #[inline]
    pub const fn from_u128(raw: u128) -> Self {
        Self(raw)
    }

    /// Underlying mask.
    #[inline]
    pub const fn raw(self) -> u128 {
        self.0
    }

    /// Single bit `1 << index`. Panics if `index >= 128`.
    #[inline]
    pub fn bit(index: u32) -> Self {
        assert!(
            index < 128,
            "Bits::bit index {index} >= 128 (this is a mapping bug, not a chart bug)"
        );
        Self(1u128 << index)
    }

    /// Union of `indices`.
    pub fn from_indices(indices: impl IntoIterator<Item = u32>) -> Self {
        let mut b = Self::EMPTY;
        for i in indices {
            b.insert(i);
        }
        b
    }

    /// Set bit `index`.
    #[inline]
    pub fn insert(&mut self, index: u32) {
        *self = self.union(Self::bit(index));
    }

    /// True when every bit of `subset` is set here (`subset ⊆ self`).
    #[inline]
    pub const fn contains(self, subset: Self) -> bool {
        self.0 & subset.0 == subset.0
    }

    /// At least one shared bit.
    #[inline]
    pub const fn intersects(self, other: Self) -> bool {
        self.0 & other.0 != 0
    }

    /// Bitwise or.
    #[inline]
    pub const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    /// Bitwise and.
    #[inline]
    pub const fn intersection(self, other: Self) -> Self {
        Self(self.0 & other.0)
    }

    /// Bits in `self` and not in `other`.
    #[inline]
    pub const fn difference(self, other: Self) -> Self {
        Self(self.0 & !other.0)
    }

    /// Hamming weight (number of set flags).
    #[inline]
    pub const fn count(self) -> u32 {
        self.0.count_ones()
    }

    /// True when no flags are set.
    #[inline]
    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }
}

impl core::ops::BitOr for Bits {
    type Output = Self;
    #[inline]
    fn bitor(self, rhs: Self) -> Self {
        self.union(rhs)
    }
}

impl core::ops::BitAnd for Bits {
    type Output = Self;
    #[inline]
    fn bitand(self, rhs: Self) -> Self {
        self.intersection(rhs)
    }
}

impl core::ops::BitOrAssign for Bits {
    #[inline]
    fn bitor_assign(&mut self, rhs: Self) {
        *self = self.union(rhs);
    }
}

impl FromIterator<u32> for Bits {
    fn from_iter<I: IntoIterator<Item = u32>>(iter: I) -> Self {
        Self::from_indices(iter)
    }
}

#[cfg(test)]
mod tests {
    use super::Bits;

    #[test]
    fn subset() {
        let ab = Bits::from_indices([0, 1]);
        let a = Bits::bit(0);
        assert!(ab.contains(a));
        assert!(!a.contains(ab));
        assert_eq!(ab.count(), 2);
    }

    #[test]
    fn union_is_or() {
        let p = Bits::bit(0) | Bits::bit(3);
        assert!(p.contains(Bits::bit(3)));
        assert!(!p.contains(Bits::bit(1)));
    }

    #[test]
    #[should_panic]
    fn bit_128_panics() {
        let _ = Bits::bit(128);
    }
}

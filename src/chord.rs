//! Host policy table over a [`Bits`](crate::Bits) pool.
//!
//! This is **not** a Harel node and **not** a YAML chart. The Newton
//! machine projects truth; the host looks up “what to do.” Quant-style
//! desks keep this table in their own crate (often `HashMap` + exact
//! key). This type exists so a Newton host does not have to invent the
//! subset rule, and so two same-length chords have an explicit tie law.
//!
//! [`Machine::update`](crate::Machine::update) must not call this.
//!
// rustbrain: [[docs/concepts/chord-and-superstate]]
// rustbrain: [[docs/adr/0023-chord-table-is-host-policy]]
// rustbrain: [[docs/concepts/configuration-versus-policy]]

use alloc::vec::Vec;

use crate::bits::Bits;

/// How [`ChordTable::lookup`] treats an unauthored pool.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MatchMode {
    /// Pool must **equal** a row mask. Unauthored chord → [`Hit::Miss`].
    /// This is the QuantSys sleeve map.
    Exact,
    /// Longest authored **subset** of the pool wins. `{A,B,C,D}` with rows
    /// `{A,B}` and `{A,B,D}` selects `{A,B,D}` (`N=3 > 2`). Unauthored
    /// atoms are ignored, not a miss, as long as some row is a subset.
    LongestSubset,
}

/// Same-length (and same-priority) race.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tie {
    /// First inserted row among the tied winners. Deterministic, silent.
    AuthorOrder,
    /// Two winners is a bug. Lookup returns [`Hit::Tie`]. Fail-closed
    /// desks use this so `{A,B}` vs `{A,C}` against pool `{A,B,C}` cannot
    /// pick a sleeve by accident.
    Refuse,
}

/// Result of [`ChordTable::lookup`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Hit<'a, T> {
    /// No authored row matched.
    Miss,
    /// Unique winner under the table’s [`MatchMode`] / [`Tie`].
    Hit {
        /// Row mask that won.
        mask: Bits,
        /// Row priority.
        priority: i16,
        /// Authored payload (sleeve, overlay, label, …).
        value: &'a T,
    },
    /// [`Tie::Refuse`] and more than one row shared the winning
    /// `(length, priority)`.
    Tie {
        /// Popcount of the tied masks.
        length: u32,
        /// How many rows tied.
        n: usize,
    },
}

#[derive(Clone, Debug)]
struct Row<T> {
    mask: Bits,
    priority: i16,
    value: T,
}

/// Ordered table of authored chords (sleeves). Linear scan: sleeve
/// tables are tens of rows, not millions. A desk that already has
/// `HashMap<u128, Sleeve>` for [`MatchMode::Exact`] should keep it.
#[derive(Clone, Debug)]
pub struct ChordTable<T> {
    rows: Vec<Row<T>>,
    mode: MatchMode,
    tie: Tie,
}

impl<T> ChordTable<T> {
    /// Empty table. Default: [`MatchMode::LongestSubset`] + [`Tie::Refuse`].
    ///
    /// Refuse-on-tie is the conservative Newton default: same-length
    /// different combinations are a specification hole, not a race the
    /// crate should invent a winner for. Quant exact maps never hit a
    /// tie (keys are unique). Desks that want YAML-list order pass
    /// [`Tie::AuthorOrder`].
    pub fn new() -> Self {
        Self {
            rows: Vec::new(),
            mode: MatchMode::LongestSubset,
            tie: Tie::Refuse,
        }
    }

    /// [`MatchMode::Exact`] + [`Tie::AuthorOrder`] (ties cannot occur).
    pub fn exact() -> Self {
        Self {
            rows: Vec::new(),
            mode: MatchMode::Exact,
            tie: Tie::AuthorOrder,
        }
    }

    /// Current match mode.
    pub fn mode(&self) -> MatchMode {
        self.mode
    }

    /// Current tie law.
    pub fn tie(&self) -> Tie {
        self.tie
    }

    /// Set [`MatchMode`].
    pub fn set_mode(&mut self, mode: MatchMode) -> &mut Self {
        self.mode = mode;
        self
    }

    /// Set [`Tie`].
    pub fn set_tie(&mut self, tie: Tie) -> &mut Self {
        self.tie = tie;
        self
    }

    /// Insert `mask → value` at priority `0`. Same mask replaces and
    /// returns the previous payload.
    pub fn insert(&mut self, mask: Bits, value: T) -> Option<T> {
        self.insert_pri(mask, 0, value)
    }

    /// Insert with an explicit priority. Higher priority wins among
    /// equal popcount. Same mask: replace (priority updated).
    pub fn insert_pri(&mut self, mask: Bits, priority: i16, value: T) -> Option<T> {
        if let Some(row) = self.rows.iter_mut().find(|r| r.mask == mask) {
            row.priority = priority;
            return Some(core::mem::replace(&mut row.value, value));
        }
        self.rows.push(Row {
            mask,
            priority,
            value,
        });
        None
    }

    /// Number of authored rows.
    pub fn len(&self) -> usize {
        self.rows.len()
    }

    /// True when no rows.
    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }

    /// Lookup `pool` (the live configuration projection).
    ///
    /// Ranking for [`MatchMode::LongestSubset`]:
    ///
    /// 1. `row.mask ⊆ pool`
    /// 2. greater [`Bits::count`]
    /// 3. greater `priority`
    /// 4. earlier insert ([`Tie::AuthorOrder`]) or [`Hit::Tie`] ([`Tie::Refuse`])
    pub fn lookup(&self, pool: Bits) -> Hit<'_, T> {
        match self.mode {
            MatchMode::Exact => self.lookup_exact(pool),
            MatchMode::LongestSubset => self.lookup_longest(pool),
        }
    }

    fn lookup_exact(&self, pool: Bits) -> Hit<'_, T> {
        for row in &self.rows {
            if row.mask == pool {
                return Hit::Hit {
                    mask: row.mask,
                    priority: row.priority,
                    value: &row.value,
                };
            }
        }
        Hit::Miss
    }

    fn lookup_longest(&self, pool: Bits) -> Hit<'_, T> {
        let mut best_len = 0u32;
        let mut best_pri = i16::MIN;
        let mut best: Option<usize> = None;
        let mut n_at_best = 0usize;

        for (i, row) in self.rows.iter().enumerate() {
            if !pool.contains(row.mask) {
                continue;
            }
            let len = row.mask.count();
            if best.is_none() || len > best_len || (len == best_len && row.priority > best_pri) {
                best_len = len;
                best_pri = row.priority;
                best = Some(i);
                n_at_best = 1;
            } else if len == best_len && row.priority == best_pri {
                n_at_best += 1;
            }
        }

        match best {
            None => Hit::Miss,
            Some(_) if n_at_best > 1 && self.tie == Tie::Refuse => Hit::Tie {
                length: best_len,
                n: n_at_best,
            },
            Some(i) => {
                let row = &self.rows[i];
                Hit::Hit {
                    mask: row.mask,
                    priority: row.priority,
                    value: &row.value,
                }
            }
        }
    }
}

impl<T> Default for ChordTable<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::{ChordTable, Hit, MatchMode, Tie};
    use crate::bits::Bits;

    fn ab() -> Bits {
        Bits::from_indices([0, 1])
    }
    fn acd() -> Bits {
        Bits::from_indices([0, 2, 3])
    }
    fn abd() -> Bits {
        Bits::from_indices([0, 1, 3])
    }
    fn abcd() -> Bits {
        Bits::from_indices([0, 1, 2, 3])
    }
    fn ac() -> Bits {
        Bits::from_indices([0, 2])
    }

    fn table() -> ChordTable<&'static str> {
        let mut t = ChordTable::new();
        t.insert(ab(), "AB");
        t.insert(acd(), "ACD");
        t.insert(abd(), "ABD");
        t
    }

    #[test]
    fn pool_ab_hits_ab() {
        match table().lookup(ab()) {
            Hit::Hit { value, .. } => assert_eq!(*value, "AB"),
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn pool_abcd_longer_beats_shorter() {
        // Only {A,B} (2) and {A,B,D} (3): longest wins. (Adding {A,C,D}
        // as well is a *tie* at N=3 — see pool_abcd_acd_abd_tie.)
        let mut t = ChordTable::new();
        t.insert(ab(), "AB");
        t.insert(abd(), "ABD");
        match t.lookup(abcd()) {
            Hit::Hit { value, mask, .. } => {
                assert_eq!(*value, "ABD");
                assert_eq!(mask, abd());
            }
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn pool_abcd_acd_abd_tie() {
        // User's full table: {A,B}, {A,C,D}, {A,B,D}. Pool {A,B,C,D}
        // has two length-3 subsets. Default Refuse → Tie, not a guess.
        match table().lookup(abcd()) {
            Hit::Tie { length, n } => {
                assert_eq!(length, 3);
                assert_eq!(n, 2);
            }
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn same_length_refuse_is_tie() {
        let mut t = ChordTable::new();
        t.insert(ab(), "AB");
        t.insert(ac(), "AC");
        match t.lookup(Bits::from_indices([0, 1, 2])) {
            Hit::Tie { length, n } => {
                assert_eq!(length, 2);
                assert_eq!(n, 2);
            }
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn same_length_author_order_picks_first() {
        let mut t = ChordTable::new();
        t.set_tie(Tie::AuthorOrder);
        t.insert(ab(), "AB");
        t.insert(ac(), "AC");
        match t.lookup(Bits::from_indices([0, 1, 2])) {
            Hit::Hit { value, .. } => assert_eq!(*value, "AB"),
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn priority_beats_author_order() {
        let mut t = ChordTable::new();
        t.set_tie(Tie::AuthorOrder);
        t.insert(ab(), "AB");
        t.insert_pri(ac(), 1, "AC");
        match t.lookup(Bits::from_indices([0, 1, 2])) {
            Hit::Hit { value, .. } => assert_eq!(*value, "AC"),
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn exact_unauthored_is_miss() {
        let mut t = ChordTable::exact();
        t.insert(ab(), "AB");
        assert!(matches!(t.lookup(abcd()), Hit::Miss));
        match t.lookup(ab()) {
            Hit::Hit { value, .. } => assert_eq!(*value, "AB"),
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn empty_table_misses() {
        let t = ChordTable::<&str>::new();
        assert!(matches!(t.lookup(ab()), Hit::Miss));
    }

    #[test]
    fn exact_mode_round_trip() {
        let mut t = ChordTable::new();
        t.set_mode(MatchMode::Exact);
        t.insert(abcd(), "ALL");
        match t.lookup(abcd()) {
            Hit::Hit { value, .. } => assert_eq!(*value, "ALL"),
            other => panic!("{other:?}"),
        }
    }
}

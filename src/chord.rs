//! Host policy table over a [`Bits`] pool.
//!
//! A Newton machine names **what is true** (nested ADTs). After `apply`,
//! the host often needs **what to do** for the *combination* of live flags:
//! “armed and degraded”, “airborne and combat”, “session live and drain
//! requested”. Promoting every conjunction into an XOR child is the
//! mega-enum this crate refused. [`ChordTable`] is that missing layer:
//! an authored table of **chords** (named subsets of a bit pool), looked
//! up *outside* [`Machine::update`](crate::Machine::update).
//!
//! This is **not** a Harel node, **not** a YAML chart, and **not** a
//! domain product (not a broker, not a renderer, not a behavior tree). The
//! payload `T` is whatever the host stores: a label, a `Cmd` template, a
//! function id, a clip name. Linear scan is deliberate: these tables are
//! tens of rows. A host that already has `HashMap<u128, T>` for
//! [`MatchMode::Exact`] should keep it and feed
//! [`Bits::raw`](crate::Bits::raw) as the key.
//!
//! # Pool, chord, hit
//!
//! - **Pool** — bits that are true together right now (`{A,B,C,D}`),
//!   usually [`Runtime::project`](crate::Runtime::project).
//! - **Chord** — an authored subset the host named (`{A,B}`, `{A,B,D}`).
//! - **Hit** — the row the table selects, or miss, or tie.
//!
//! ```text
//! typed config  --project-->  Bits  --lookup-->  Hit { value | miss | tie }
//!      Newton                   key                   host policy
//! ```
//!
//! # Match modes
//!
//! [`MatchMode::Exact`] — pool must **equal** a row. Unauthored pool →
//! [`Hit::Miss`]. Fail-closed: nothing fires unless you listed that
//! combination.
//!
//! [`MatchMode::LongestSubset`] — longest authored **subset** of the pool
//! wins. `{A,B,C,D}` with rows `{A,B}` (`N=2`) and `{A,B,D}` (`N=3`)
//! selects `{A,B,D}`. Extra atoms do not invent a row; they also do not
//! block a more specific authored chord.
//!
//! Same length (and same [`insert_pri`](ChordTable::insert_pri) priority)
//! is a **specification hole**, not a race the crate should invent.
//! Default [`Tie::Refuse`] returns [`Hit::Tie`]. [`Tie::AuthorOrder`]
//! picks the first inserted row (list order).
//!
//! Note: `{A,C,D}` and `{A,B,D}` are both `N=3`. Against pool `{A,B,C,D}`
//! that is a tie, not “prefer ABD because we mentioned it in a comment.”
//!
//! # What this is not
//!
//! - Not called from `update`. The chart does not own the table.
//! - Not longest-match-as-a-Harel-parent. The winning chord is a **label
//!   the host computed**, not a new XOR child.
//! - Not a domain engine. A protocol host, a game, a robot sequencer, and
//!   a trading desk can all use the same type; only `T` and the bit
//!   mapping change.
//!
// rustbrain: [[docs/concepts/chord-and-superstate]]
// rustbrain: [[docs/adr/0023-chord-table-is-host-policy]]
// rustbrain: [[docs/concepts/configuration-versus-policy]]

use alloc::vec::Vec;

use crate::bits::Bits;

/// How [`ChordTable::lookup`] treats an unauthored pool.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MatchMode {
    /// Pool must **equal** a row mask. Unauthored combination → [`Hit::Miss`].
    ///
    /// Use this when every interesting product is listed and silence is
    /// the correct default (fail-closed).
    Exact,
    /// Longest authored **subset** of the pool wins.
    ///
    /// `{A,B,C,D}` with rows `{A,B}` and `{A,B,D}` selects `{A,B,D}`
    /// (`N=3 > 2`). Unauthored extra atoms are ignored, not a miss, as
    /// long as some row is still a subset. Same-length leftover is
    /// [`Tie`], not a guess.
    LongestSubset,
}

/// Same-length (and same-priority) race among subsets of the pool.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tie {
    /// First inserted row among the tied winners. Deterministic, silent.
    ///
    /// Use when the table is an ordered list and “first matching row”
    /// is the documented policy.
    AuthorOrder,
    /// Two winners is a bug. Lookup returns [`Hit::Tie`].
    ///
    /// Fail-closed default: `{A,B}` vs `{A,C}` against pool `{A,B,C}`
    /// must not pick a payload by accident. Add a more specific row,
    /// set [`ChordTable::insert_pri`], or switch to [`Tie::AuthorOrder`].
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
        /// Row priority (higher wins among equal popcount).
        priority: i16,
        /// Authored payload (label, overlay, `Cmd` template, …).
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

/// Ordered table of authored chords.
///
/// Linear scan: policy tables are tens of rows, not millions. If you
/// already index exact keys in a `HashMap<u128, T>`, keep that map and
/// skip this type.
///
/// # Example
///
/// Protocol flags: bit 0 = `live`, bit 1 = `degraded`, bit 2 = `drain`.
/// A host might fire “degraded_live” only when both live and degraded
/// are on, even if drain is also on (longest subset), or require the
/// exact mask (exact mode).
///
/// ```
/// # #[cfg(feature = "alloc")]
/// # {
/// use newton_machine::prelude::*;
///
/// let live = Bits::bit(0);
/// let degraded = Bits::bit(1);
/// let drain = Bits::bit(2);
///
/// let mut t = ChordTable::new(); // longest subset, refuse ties
/// t.insert(live | degraded, "degraded_live");
///
/// let pool = live | degraded | drain;
/// match t.lookup(pool) {
///     Hit::Hit { value, .. } => assert_eq!(*value, "degraded_live"),
///     other => panic!("{other:?}"),
/// }
/// # }
/// ```
#[derive(Clone, Debug)]
pub struct ChordTable<T> {
    rows: Vec<Row<T>>,
    mode: MatchMode,
    tie: Tie,
}

impl<T> ChordTable<T> {
    /// Empty table. Default: [`MatchMode::LongestSubset`] + [`Tie::Refuse`].
    ///
    /// Refuse-on-tie is the conservative default: two same-length
    /// chords against a fatter pool are a hole in the table, not a
    /// winner the crate should invent. Hosts that want list order pass
    /// [`Tie::AuthorOrder`]. Hosts that want fail-closed exact keys use
    /// [`ChordTable::exact`].
    pub fn new() -> Self {
        Self {
            rows: Vec::new(),
            mode: MatchMode::LongestSubset,
            tie: Tie::Refuse,
        }
    }

    /// [`MatchMode::Exact`] + [`Tie::AuthorOrder`] (ties cannot occur:
    /// keys are unique).
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
    /// 1. `row.mask ⊆ pool` ([`Bits::contains`])
    /// 2. greater [`Bits::count`] (longer chord)
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
        // Full table: {A,B}, {A,C,D}, {A,B,D}. Pool {A,B,C,D}
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

//! Host-side helpers. The crate still never opens a socket.
//!
//! A tape records commands so live and test hosts share one `update`.
//!
// rustbrain: [[docs/adr/0011-core-crate-is-not-a-broker]]
// rustbrain: [[docs/concepts/intent-and-authority]]
// rustbrain: [[docs/concepts/chord-and-superstate]]

use crate::cmd::Cmd;

#[cfg(feature = "alloc")]
use alloc::vec::Vec;

/// Category-change gate: `Some(next)` when `prev != next`.
///
/// Host classifies (TA, scores, IR). Newton `apply`s only when an XOR
/// child actually moves. A tick that leaves `Neutral` as `Neutral` is
/// silence, not a `Msg`.
#[inline]
pub fn changed<T: PartialEq>(prev: &T, next: T) -> Option<T> {
    if *prev == next {
        None
    } else {
        Some(next)
    }
}

/// Records commands in document order. Does not execute them.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Tape<C> {
    #[cfg(feature = "alloc")]
    items: Vec<C>,
    #[cfg(not(feature = "alloc"))]
    last: Option<C>,
}

impl<C> Tape<C> {
    /// Empty tape.
    pub const fn new() -> Self {
        Self {
            #[cfg(feature = "alloc")]
            items: Vec::new(),
            #[cfg(not(feature = "alloc"))]
            last: None,
        }
    }

    /// Append every atom in `cmd`. Without `alloc`, only the last atom is
    /// kept on the tape (the `Cmd` itself no longer drops atoms on `and`).
    pub fn record(&mut self, cmd: Cmd<C>) {
        #[cfg(feature = "alloc")]
        {
            self.items.extend(cmd);
        }
        #[cfg(not(feature = "alloc"))]
        {
            for c in cmd {
                self.last = Some(c);
            }
        }
    }

    /// Recorded atoms, in order (empty slice without `alloc` if none).
    #[cfg(feature = "alloc")]
    pub fn as_slice(&self) -> &[C] {
        &self.items
    }

    /// Most recently recorded atom, if any.
    ///
    /// Without `alloc`, the tape only keeps this value (see [`Tape::record`]).
    pub fn last(&self) -> Option<&C> {
        #[cfg(feature = "alloc")]
        {
            self.items.last()
        }
        #[cfg(not(feature = "alloc"))]
        {
            self.last.as_ref()
        }
    }
}

#[cfg(feature = "alloc")]
impl<C> Tape<C> {
    /// Consume the tape.
    pub fn into_vec(self) -> Vec<C> {
        self.items
    }
}

#[cfg(test)]
mod tests {
    use super::Tape;
    use crate::cmd::Cmd;

    #[test]
    fn changed_is_none_when_equal() {
        assert_eq!(super::changed(&3u8, 3), None);
        assert_eq!(super::changed(&3u8, 4), Some(4));
    }

    #[test]
    fn last_tracks_record() {
        let mut tape = Tape::new();
        assert!(tape.last().is_none());
        tape.record(Cmd::single(1u8));
        assert_eq!(tape.last(), Some(&1));
        tape.record(Cmd::single(2));
        assert_eq!(tape.last(), Some(&2));
    }
}

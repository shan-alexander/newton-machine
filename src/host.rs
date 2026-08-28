//! Host-side helpers. The crate still never opens a socket.
//!
//! A tape records commands so live and test hosts share one `update`.
//!
// rustbrain: [[docs/adr/0011-core-crate-is-not-a-broker]]
// rustbrain: [[docs/concepts/intent-and-authority]]

use crate::cmd::Cmd;

#[cfg(feature = "alloc")]
use alloc::vec::Vec;

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

    /// Append every atom in `cmd`. Without `alloc`, only the last atom is kept
    /// when `cmd` is a batch (same limitation as [`Cmd::and`]).
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
}

#[cfg(feature = "alloc")]
impl<C> Tape<C> {
    /// Consume the tape.
    pub fn into_vec(self) -> Vec<C> {
        self.items
    }
}

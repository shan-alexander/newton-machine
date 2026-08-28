//! Associative combination of commands collected during a step.
//!
// rustbrain: [[docs/concepts/command-as-data]] [[docs/adr/0004-commands-as-data]]

use crate::cmd::Cmd;

/// A command (or command-like) value that can be concatenated with identity
/// [`Combine::none`].
///
/// [`Cmd`] implements this. `()` implements it for machines with no effects.
pub trait Combine: Sized {
    /// Identity: no work.
    fn none() -> Self;

    /// Concatenate in document order. [`Combine::none`] is the identity.
    fn combine(self, other: Self) -> Self;
}

impl<C> Combine for Cmd<C> {
    #[inline]
    fn none() -> Self {
        Self::None
    }

    #[inline]
    fn combine(self, other: Self) -> Self {
        self.and(other)
    }
}

impl Combine for () {
    #[inline]
    fn none() -> Self {}

    #[inline]
    fn combine(self, _other: Self) {}
}

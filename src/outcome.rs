//! Per-region dispatch result. Parents see `Super`; the step sees a transition.
//!
//! Offer an event to the innermost active leaf of each orthogonal region. If that
//! leaf does not consume it, bubble to its XOR parent. That is `statig`'s Super
//! idea, made an explicit enum so `update` stays ordinary Rust.
//!
// rustbrain: [[docs/concepts/outcome-vocabulary]] [[docs/adr/0014-outcome-vocabulary]]

/// Result of offering a message to one region of the configuration tree.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Outcome<S, C> {
    /// The region consumed the message and stayed put, with no command.
    Handled,
    /// Defer to the parent XOR region.
    Super,
    /// Leave this region for `to`, carrying a command.
    Transition {
        /// Destination configuration for this region (not necessarily the root).
        to: S,
        /// Command collected on the way out. May be [`crate::Cmd::None`].
        cmd: C,
    },
    /// Stay in this region; run an action only.
    Internal(C),
}

impl<S, C> Outcome<S, C> {
    /// Consumed, stayed, no command.
    #[inline]
    pub const fn handled() -> Self {
        Self::Handled
    }

    /// Defer to the parent XOR.
    #[inline]
    pub const fn super_() -> Self {
        Self::Super
    }

    /// Leave this region.
    #[inline]
    pub const fn transition(to: S, cmd: C) -> Self {
        Self::Transition { to, cmd }
    }

    /// Stay; run an action only.
    #[inline]
    pub const fn internal(cmd: C) -> Self {
        Self::Internal(cmd)
    }

    /// True when a parent should be offered the same message.
    #[inline]
    pub const fn defers(&self) -> bool {
        matches!(self, Self::Super)
    }

    /// If this is [`Outcome::Super`], run `parent`; otherwise keep `self`.
    pub fn or_else(self, parent: impl FnOnce() -> Self) -> Self {
        match self {
            Self::Super => parent(),
            other => other,
        }
    }

    /// Split a destination (if transitioning) from the command payload.
    pub fn into_parts(self) -> (Option<S>, C)
    where
        C: crate::combine::Combine,
    {
        match self {
            Self::Handled | Self::Super => (None, C::none()),
            Self::Internal(cmd) => (None, cmd),
            Self::Transition { to, cmd } => (Some(to), cmd),
        }
    }

    /// Map the destination type.
    pub fn map_target<T>(self, f: impl FnOnce(S) -> T) -> Outcome<T, C> {
        match self {
            Self::Handled => Outcome::Handled,
            Self::Super => Outcome::Super,
            Self::Transition { to, cmd } => Outcome::Transition { to: f(to), cmd },
            Self::Internal(cmd) => Outcome::Internal(cmd),
        }
    }

    /// Map the command type.
    pub fn map_cmd<D>(self, f: impl FnOnce(C) -> D) -> Outcome<S, D> {
        match self {
            Self::Handled => Outcome::Handled,
            Self::Super => Outcome::Super,
            Self::Transition { to, cmd } => Outcome::Transition { to, cmd: f(cmd) },
            Self::Internal(cmd) => Outcome::Internal(f(cmd)),
        }
    }
}

//! Elm loop around a Harel configuration tree.
//!
//! `Self` is the configuration (XOR enums / AND structs). `Model` is extended
//! state. `History` is the sidecar. That split keeps `Clone` / serde honest:
//! chart + context + history serialize; behavior does not.
//!
//! Two entry points, one semantics:
//!
//! - [`step`] — Elm-pure: owned in, owned out. Tests, time-travel, persist-after-event.
//! - [`apply`] — hot path: `&mut`, no clone of context.
//!
//! [`step`] is replace + [`apply`]. Do not invent a second semantics.

// rustbrain: [[docs/concepts/newtonian-state-machine]]
// rustbrain: [[docs/adr/0003-tea-is-the-only-mutation-protocol]]
// rustbrain: [[docs/goals/elm-shaped-public-api]]

use crate::rtc::Storm;
use crate::sub::Sub;

/// Result of [`Machine::init`]: configuration, extended state, sidecar, entry commands.
#[derive(Clone, Debug)]
pub struct Boot<M: Machine> {
    /// Live configuration tree.
    pub machine: M,
    /// Extended state (Harel datamodel).
    pub model: M::Model,
    /// Inertial history sidecar.
    pub history: M::History,
    /// Commands from entering the initial configuration. Host executes these.
    pub cmd: M::Cmd,
}

impl<M: Machine> Boot<M> {
    /// Assemble the four parts returned by [`Machine::init`].
    #[inline]
    pub const fn new(machine: M, model: M::Model, history: M::History, cmd: M::Cmd) -> Self {
        Self {
            machine,
            model,
            history,
            cmd,
        }
    }
}

/// A Newtonian state machine.
///
/// Implementors encode the live configuration in `Self`. Heavy data belongs in
/// `Model`. Opt-in history belongs in `History` (`()` if unused).
///
/// `update` must not perform I/O. Entry/exit/transition actions return
/// [`crate::Cmd`] values. The host executes them.
#[doc(alias = "Elm")]
#[doc(alias = "TEA")]
#[doc(alias = "MVU")]
pub trait Machine {
    /// Construction input (Elm flags). Use `()` if none.
    type Flags;
    /// Extended state: Harel datamodel, not the chart.
    type Model;
    /// Applied force. User actions and host facts become messages.
    type Msg;
    /// Reaction. Data the host will execute after the step.
    type Cmd;
    /// Projection for humans or a renderer. Need not be HTML.
    type View;
    /// Inertial sidecar. Use `()` when no composite opted into history.
    type History;
    /// Identifier for [`Machine::in_state`]. Prefer a compact enum, not a `String`.
    type NodeId;

    /// First configuration, first model, first history, first command.
    fn init(flags: Self::Flags) -> Boot<Self>
    where
        Self: Sized;

    /// The only accelerator. Mutate configuration and model; return commands.
    ///
    /// Must not perform I/O. Must not call the host.
    ///
    /// Simple machines (no internal follow-ups) implement this and leave
    /// [`Machine::try_update`] at the default.
    ///
    /// Machines that drain [`crate::rtc()`] **must** override
    /// [`Machine::try_update`] and implement `update` as
    /// [`crate::rtc::unwrap_storm`]`(self.try_update(...))`. Storm is a chart
    /// bug, not a quiet `.expect`. Hosts that must Halt rather than die call
    /// [`crate::Runtime::try_apply`].
    fn update(
        &mut self,
        model: &mut Self::Model,
        history: &mut Self::History,
        msg: Self::Msg,
    ) -> Self::Cmd;

    /// Fallible [`Machine::update`]. Default: `Ok(self.update(...))`.
    ///
    /// Override when `update` uses [`crate::rtc()`] so [`crate::Runtime::try_apply`]
    /// can return [`Storm`] instead of panicking.
    fn try_update(
        &mut self,
        model: &mut Self::Model,
        history: &mut Self::History,
        msg: Self::Msg,
    ) -> Result<Self::Cmd, Storm> {
        Ok(self.update(model, history, msg))
    }

    /// Pure projection of configuration + model.
    fn view(&self, model: &Self::Model) -> Self::View;

    /// Listeners that should exist *now*. The host diffs this against the last
    /// value and starts or stops timers, feeds, and sockets.
    fn subscriptions(&self, _model: &Self::Model) -> Sub<Self::Msg> {
        Sub::none()
    }

    /// Query whether a node is in the active configuration.
    fn in_state(&self, id: Self::NodeId) -> bool;

    /// Write active node ids into `out`. Returns how many were written.
    ///
    /// Default: writes nothing. Used for diagnostics, not the hot path.
    fn configuration(&self, out: &mut [Self::NodeId]) -> usize {
        let _ = out;
        0
    }

    /// Compact bitset of the live configuration. Host
    /// [`crate::ChordTable`]s (or a host `HashMap`) index this;
    /// [`crate::Machine::update`] does not.
    ///
    /// Default: empty. Override when the host needs a `u128` key. Orthogonal
    /// XOR children must occupy disjoint bits.
    fn project(&self) -> crate::bits::Bits {
        crate::bits::Bits::EMPTY
    }
}

/// Hot path: one message, in place, one command.
///
/// Same function body as [`step`].
#[inline]
pub fn apply<M: Machine>(
    machine: &mut M,
    model: &mut M::Model,
    history: &mut M::History,
    msg: M::Msg,
) -> M::Cmd {
    machine.update(model, history, msg)
}

/// [`apply`] that surfaces [`Storm`] instead of panicking.
#[inline]
pub fn try_apply<M: Machine>(
    machine: &mut M,
    model: &mut M::Model,
    history: &mut M::History,
    msg: M::Msg,
) -> Result<M::Cmd, Storm> {
    machine.try_update(model, history, msg)
}

/// Elm-pure path: owned machine, model, and history in; owned quadruple out.
///
/// Use this in tests, journals, and anywhere you persist after the event.
#[inline]
pub fn step<M: Machine>(
    mut machine: M,
    mut model: M::Model,
    mut history: M::History,
    msg: M::Msg,
) -> (M, M::Model, M::History, M::Cmd) {
    let cmd = machine.update(&mut model, &mut history, msg);
    (machine, model, history, cmd)
}

/// [`step`] that surfaces [`Storm`] instead of panicking.
#[inline]
#[allow(clippy::type_complexity)]
pub fn try_step<M: Machine>(
    mut machine: M,
    mut model: M::Model,
    mut history: M::History,
    msg: M::Msg,
) -> Result<(M, M::Model, M::History, M::Cmd), Storm> {
    let cmd = machine.try_update(&mut model, &mut history, msg)?;
    Ok((machine, model, history, cmd))
}

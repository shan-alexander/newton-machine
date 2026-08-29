//! First-class Harel AND: two machines, one message, one RTC clock.
//!
//! GoF Composite as a product type. Regions are **not** threads. The same
//! `Msg` is offered left then right (document order). Cross-talk is `Msg` /
//! model, never assigning the sibling's configuration — each side owns its
//! `Self`.
//!
//! Shared datamodel (one `ticks: u32` both regions read) is still a
//! handwritten `struct { auth, sync }` with one `Model`. [`And`] is the
//! engine combinator: independent machines glued under one pulse.
//!
// rustbrain: [[docs/concepts/and-node]]
// rustbrain: [[docs/adr/0021-and-combinator]]
// rustbrain: [[docs/adr/0007-virtual-concurrency-not-threads]]

use crate::combine::Combine;
use crate::machine::{Boot, Machine};
use crate::rtc::{unwrap_storm, Storm};
use crate::sub::Sub;

/// Product of two region sidecars.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AndHistory<L, R> {
    /// Left region's inertial memory.
    pub left: L,
    /// Right region's inertial memory.
    pub right: R,
}

/// [`Machine::NodeId`] for [`And`]: tag the child so `in_state` cannot mix
/// regions.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AndNode<L, R> {
    /// A node in the left region.
    Left(L),
    /// A node in the right region.
    Right(R),
}

/// Orthogonal pair: `left` then `right` see each `Msg`.
///
/// `Msg` must be [`Clone`] (offered twice). `Cmd` must be [`Combine`].
/// Models and histories are **split** — shared facts belong in a parent
/// handwritten struct, or in `Msg`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct And<L, R> {
    /// First region in document order.
    pub left: L,
    /// Second region in document order.
    pub right: R,
}

impl<L, R> And<L, R> {
    /// Glue two machines. Does not run `init`.
    #[inline]
    pub const fn new(left: L, right: R) -> Self {
        Self { left, right }
    }
}

impl<L, R> Machine for And<L, R>
where
    L: Machine,
    R: Machine<Msg = L::Msg, Cmd = L::Cmd>,
    L::Msg: Clone,
    L::Cmd: Combine,
{
    type Flags = (L::Flags, R::Flags);
    type Model = (L::Model, R::Model);
    type Msg = L::Msg;
    type Cmd = L::Cmd;
    type View = (L::View, R::View);
    type History = AndHistory<L::History, R::History>;
    type NodeId = AndNode<L::NodeId, R::NodeId>;

    fn init((lf, rf): Self::Flags) -> Boot<Self> {
        let left = L::init(lf);
        let right = R::init(rf);
        Boot::new(
            And {
                left: left.machine,
                right: right.machine,
            },
            (left.model, right.model),
            AndHistory {
                left: left.history,
                right: right.history,
            },
            left.cmd.combine(right.cmd),
        )
    }

    fn try_update(
        &mut self,
        model: &mut Self::Model,
        history: &mut Self::History,
        msg: Self::Msg,
    ) -> Result<Self::Cmd, Storm> {
        let a = self
            .left
            .try_update(&mut model.0, &mut history.left, msg.clone())?;
        let b = self
            .right
            .try_update(&mut model.1, &mut history.right, msg)?;
        Ok(a.combine(b))
    }

    fn update(
        &mut self,
        model: &mut Self::Model,
        history: &mut Self::History,
        msg: Self::Msg,
    ) -> Self::Cmd {
        unwrap_storm(self.try_update(model, history, msg))
    }

    fn view(&self, model: &Self::Model) -> Self::View {
        (self.left.view(&model.0), self.right.view(&model.1))
    }

    fn subscriptions(&self, model: &Self::Model) -> Sub<Self::Msg> {
        self.left
            .subscriptions(&model.0)
            .and(self.right.subscriptions(&model.1))
    }

    fn in_state(&self, id: Self::NodeId) -> bool {
        match id {
            AndNode::Left(n) => self.left.in_state(n),
            AndNode::Right(n) => self.right.in_state(n),
        }
    }
}

//! Owns `{config, context, history}` and runs the Elm loop.
//!
//! Persist **after** a successful step, never from an entry action.
//!
// rustbrain: [[docs/concepts/phase-space-snapshot]]
// rustbrain: [[docs/adr/0015-persist-after-rtc]]
// rustbrain: [[docs/edge_cases/panic-during-rtc]]

use crate::history::HistoryStore;
use crate::machine::{Boot, Machine};
use crate::rtc::Storm;
use crate::snapshot::Snapshot;
use crate::sub::Sub;

/// Running Newton machine: configuration + extended state + history sidecar.
#[derive(Clone, Debug)]
pub struct Runtime<M: Machine> {
    machine: M,
    model: M::Model,
    history: M::History,
}

impl<M: Machine> Runtime<M> {
    /// Boot from flags. Returns the runtime and the initial entry commands.
    pub fn boot(flags: M::Flags) -> (Self, M::Cmd) {
        let Boot {
            machine,
            model,
            history,
            cmd,
        } = M::init(flags);
        (
            Self {
                machine,
                model,
                history,
            },
            cmd,
        )
    }

    /// Wrap an already-constructed triple (tests, restore scaffolding).
    pub const fn new(machine: M, model: M::Model, history: M::History) -> Self {
        Self {
            machine,
            model,
            history,
        }
    }

    /// From [`Machine::init`].
    pub fn from_boot(boot: Boot<M>) -> (Self, M::Cmd) {
        (
            Self {
                machine: boot.machine,
                model: boot.model,
                history: boot.history,
            },
            boot.cmd,
        )
    }

    /// Hot path: one external message, one command. Does not execute the command.
    ///
    /// Storm panics if the chart's `update` panics on [`Storm`] (the default
    /// for machines that call [`crate::rtc::unwrap_storm`]). Use
    /// [`Runtime::try_apply`] to Halt instead of die.
    pub fn apply(&mut self, msg: M::Msg) -> M::Cmd {
        self.machine.update(&mut self.model, &mut self.history, msg)
    }

    /// Apply `msg` only when `gate` is true (category-change / lift).
    ///
    /// The host classifies. A tick that does not move an XOR child is
    /// not a message. If `gate` is false, `msg` is dropped and this returns
    /// `None` — prefer [`crate::host::changed`] so you only build `msg` on
    /// a real change.
    pub fn apply_if(&mut self, gate: bool, msg: M::Msg) -> Option<M::Cmd> {
        if gate {
            Some(self.apply(msg))
        } else {
            None
        }
    }

    /// [`Runtime::apply`] that returns [`Storm`] instead of panicking.
    ///
    /// Only useful if the machine **overrides** [`Machine::try_update`].
    /// The default `try_update` calls `update`, so a panicking `update`
    /// still panics here.
    pub fn try_apply(&mut self, msg: M::Msg) -> Result<M::Cmd, Storm> {
        self.machine
            .try_update(&mut self.model, &mut self.history, msg)
    }

    /// Pure view.
    pub fn view(&self) -> M::View {
        self.machine.view(&self.model)
    }

    /// Subscriptions for the current configuration.
    pub fn subscriptions(&self) -> Sub<M::Msg> {
        self.machine.subscriptions(&self.model)
    }

    /// Whether `id` is in the active configuration.
    pub fn in_state(&self, id: M::NodeId) -> bool {
        self.machine.in_state(id)
    }

    /// [`Machine::project`](crate::Machine::project): compact key for a
    /// host [`ChordTable`](crate::ChordTable) (or the host’s own map).
    /// Not a Harel node.
    #[inline]
    pub fn project(&self) -> crate::bits::Bits {
        self.machine.project()
    }

    /// Active node ids written into `out`.
    pub fn configuration(&self, out: &mut [M::NodeId]) -> usize {
        self.machine.configuration(out)
    }

    /// Borrow the configuration tree.
    #[inline]
    pub fn machine(&self) -> &M {
        &self.machine
    }

    /// Borrow extended state.
    #[inline]
    pub fn model(&self) -> &M::Model {
        &self.model
    }

    /// Borrow the history sidecar.
    #[inline]
    pub fn history(&self) -> &M::History {
        &self.history
    }
}

impl<M: Machine> Runtime<M>
where
    M: Clone,
    M::Model: Clone,
    M::History: Clone,
{
    /// Phase-space point after a completed step.
    pub fn snapshot(&self) -> Snapshot<M, M::Model, M::History> {
        Snapshot::new(
            self.machine.clone(),
            self.model.clone(),
            self.history.clone(),
        )
    }

    /// Replace the live triple. Host must still reconcile the world
    /// (broker, sockets) after restore.
    pub fn restore(&mut self, snap: Snapshot<M, M::Model, M::History>) {
        self.machine = snap.config;
        self.model = snap.context;
        self.history = snap.history;
    }

    /// Save after RTC completed. Does not execute commands.
    pub fn persist<S>(&self, store: &mut S) -> Result<(), S::Error>
    where
        S: HistoryStore<Config = M, Context = M::Model, History = M::History>,
    {
        store.save(&self.snapshot())
    }

    /// Load a snapshot if the store has one. Does not reconcile the host.
    pub fn load<S>(store: &S) -> Result<Option<Self>, S::Error>
    where
        S: HistoryStore<Config = M, Context = M::Model, History = M::History>,
    {
        Ok(store.load()?.map(|snap| Self {
            machine: snap.config,
            model: snap.context,
            history: snap.history,
        }))
    }
}

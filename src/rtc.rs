//! Run-to-completion drain of internal follow-up messages.
//!
//! One external `Msg` is one step from the caller's view. Follow-ups may be
//! queued inside the step. The drain is capped so a loop cannot hang the host.
//!
// rustbrain: [[docs/concepts/run-to-completion]]
// rustbrain: [[docs/edge_cases/internal-event-storms]]
// rustbrain: [[docs/adr/0003-tea-is-the-only-mutation-protocol]]

use crate::combine::Combine;

/// Default cap: inbox size and maximum messages processed per step.
pub const DEFAULT_DRAIN_CAP: usize = 32;

/// An internal-event storm: the drain cap was exceeded.
///
/// This is a bug in the machine (a guard that always retriggers), not a host
/// I/O error. The previous snapshot remains the last honest point.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Storm {
    /// Cap that was exceeded.
    pub cap: u16,
    /// Messages processed before abort (including the first).
    pub drained: u16,
}

impl core::fmt::Display for Storm {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "internal event storm: drained {} with cap {} (chart bug; Runtime::try_apply to Halt instead of die)",
            self.drained, self.cap
        )
    }
}

/// Panic on [`Storm`]. Use in [`crate::Machine::update`] when the chart
/// uses [`rtc`]. Hosts that must Halt rather than die call
/// [`crate::Runtime::try_apply`].
///
/// Storm is a programmer error (always-true guard), like indexing past a
/// slice. `try_apply` is the `slice.get` opt-in.
pub fn unwrap_storm<T>(result: Result<T, Storm>) -> T {
    result.unwrap_or_else(|s| panic!("{s}"))
}

#[cfg(feature = "std")]
impl std::error::Error for Storm {}

/// Bounded FIFO of follow-up messages. Does not allocate.
///
/// `push` that would exceed `N` sets [`Inbox::overflowed`] instead of growing.
#[derive(Clone, Debug)]
pub struct Inbox<M, const N: usize = DEFAULT_DRAIN_CAP> {
    slots: [Option<M>; N],
    start: usize,
    len: usize,
    overflowed: bool,
}

impl<M, const N: usize> Inbox<M, N> {
    /// Empty inbox.
    pub fn new() -> Self {
        Self {
            slots: core::array::from_fn(|_| None),
            start: 0,
            len: 0,
            overflowed: false,
        }
    }

    /// Queue a follow-up. Silent overflow: see [`Inbox::overflowed`].
    pub fn push(&mut self, msg: M) {
        if self.len >= N {
            self.overflowed = true;
            return;
        }
        let i = (self.start + self.len) % N;
        self.slots[i] = Some(msg);
        self.len += 1;
    }

    /// Pop the next message (FIFO).
    pub fn pop(&mut self) -> Option<M> {
        if self.len == 0 {
            return None;
        }
        let msg = self.slots[self.start].take();
        self.start = (self.start + 1) % N;
        self.len -= 1;
        msg
    }

    /// True when a [`push`](Inbox::push) was dropped because the inbox was full.
    #[inline]
    pub const fn overflowed(&self) -> bool {
        self.overflowed
    }

    /// Messages currently waiting.
    #[inline]
    pub const fn len(&self) -> usize {
        self.len
    }

    /// True when nothing is waiting.
    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }
}

impl<M, const N: usize> Default for Inbox<M, N> {
    fn default() -> Self {
        Self::new()
    }
}

/// Drain `first` and any follow-ups `body` pushes, concatenating commands.
///
/// Cap is [`DEFAULT_DRAIN_CAP`]. Use [`rtc_n`] to pick another.
pub fn rtc<M, C, F>(first: M, body: F) -> Result<C, Storm>
where
    C: Combine,
    F: FnMut(M, &mut Inbox<M>) -> C,
{
    rtc_n::<M, C, F, DEFAULT_DRAIN_CAP>(first, body)
}

/// [`rtc`] with an explicit inbox size and drain cap `N`.
pub fn rtc_n<M, C, F, const N: usize>(first: M, mut body: F) -> Result<C, Storm>
where
    C: Combine,
    F: FnMut(M, &mut Inbox<M, N>) -> C,
{
    let mut inbox = Inbox::<M, N>::new();
    inbox.push(first);
    let mut cmd = C::none();
    let mut drained: u16 = 0;
    let cap = N as u16;
    while let Some(msg) = inbox.pop() {
        if drained >= cap {
            return Err(Storm { cap, drained });
        }
        drained = drained.saturating_add(1);
        cmd = cmd.combine(body(msg, &mut inbox));
        if inbox.overflowed() {
            return Err(Storm { cap, drained });
        }
    }
    Ok(cmd)
}

#[cfg(test)]
mod tests {
    use super::{rtc, rtc_n, Inbox, Storm};
    use crate::cmd::Cmd;

    #[test]
    fn single_message() {
        let cmd = rtc::<_, Cmd<u8>, _>(1, |m, _| Cmd::single(m)).unwrap();
        assert_eq!(cmd, Cmd::single(1));
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn follow_up_drained() {
        let cmd = rtc::<_, Cmd<u8>, _>(1, |m, inbox| {
            if m == 1 {
                inbox.push(2);
            }
            Cmd::single(m)
        })
        .unwrap();
        assert_eq!(cmd, Cmd::single(1).and(Cmd::single(2)));
    }

    #[test]
    fn storm_on_loop() {
        let err = rtc_n::<_, (), _, 4>(0, |m, inbox| {
            inbox.push(m + 1);
        })
        .unwrap_err();
        assert_eq!(err, Storm { cap: 4, drained: 4 });
    }

    #[test]
    fn inbox_fifo() {
        let mut inbox = Inbox::<u8, 4>::new();
        inbox.push(1);
        inbox.push(2);
        assert_eq!(inbox.pop(), Some(1));
        assert_eq!(inbox.pop(), Some(2));
        assert!(inbox.is_empty());
    }
}

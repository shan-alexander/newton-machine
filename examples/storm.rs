//! # Storm — the RTC drain cap
//!
//! Run: `cargo run --example storm`
//!
//! **RTC** (run-to-completion): one external message is finished before the
//! next external message starts. Harel also allows *internal* follow-ups
//! during that step (`rtc` drains an [`Inbox`]).
//!
//! A transition that always queues another message (`0 → 1 → 2 → …` forever)
//! would never return to the host. The **drain cap** (default 32, here 4)
//! turns that into [`Storm`] instead of a wedged process.
//!
//! A Harel interpreter might spin on internal events until a tool-specific
//! timeout — or never. UCA makes the bound part of the step.

use newton_machine::prelude::*;

fn main() {
    println!("# storm  (rtc drain cap)\n");
    let ok = rtc::<u8, Cmd<u8>, _>(1, |m, inbox| {
        if m < 3 {
            inbox.push(m + 1);
        }
        Cmd::single(m)
    });
    println!("bounded chain: {ok:?}");

    let err = rtc_n::<u8, (), _, 4>(0, |m, inbox| {
        inbox.push(m.wrapping_add(1));
    });
    println!("loop:           {err:?}");
}

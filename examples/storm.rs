//! Internal follow-ups are capped. A loop is a `Storm`, not a hang.
//!
//! ```text
//! cargo run --example storm
//! ```

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

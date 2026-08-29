//! Declarative macros. Always available (no `macros` feature).
//!
//! `perform!` needs [`crate::IntoNode`]. Proc-macro derives live behind
//! feature `macros`.

/// LCA [`crate::perform()`] using [`crate::IntoNode::node`]
/// so authors do not name `from` / `to` twice.
///
/// ```ignore
/// perform!(self, dest, model, hist);
/// // same as:
/// // let from = self.node();
/// // let to = dest.node();
/// // perform(self, from, dest, to, model, hist);
/// ```
///
/// Requires [`crate::IntoNode`] on the chart type (`#[derive(IntoNode)]` or handwritten).
///
/// Glob-importing [`prelude`](crate::prelude) brings the **function**
/// [`crate::perform()`], so invoke the macro as `newton_machine::perform!(...)`.
#[macro_export]
macro_rules! perform {
    ($chart:expr, $dest:expr, $ctx:expr, $hist:expr $(,)?) => {{
        let __dest = $dest;
        let __from = $crate::IntoNode::node(&*$chart);
        let __to = $crate::IntoNode::node(&__dest);
        $crate::perform($chart, __from, __dest, __to, $ctx, $hist)
    }};
}

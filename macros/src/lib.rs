//! Proc-macros for `newton-machine`. Depend on `newton-machine` with
//! `features = ["macros"]`; do not `cargo add` this crate.

use proc_macro::TokenStream;

mod into_node;
mod machine;
mod topology;

/// Parent tree on a **node-id** enum.
///
/// Exactly one variant is `#[topology(root)]`. Every other variant is
/// `#[topology(parent = OtherVariant)]`. Cycle or unknown parent → compile error.
///
/// ```ignore
/// #[derive(Clone, Copy, PartialEq, Eq, Topology)]
/// enum Node {
///     #[topology(root)]
///     Root,
///     #[topology(parent = Root)]
///     Off,
///     #[topology(parent = Root)]
///     On,
/// }
/// ```
#[proc_macro_derive(Topology, attributes(topology))]
pub fn derive_topology(input: TokenStream) -> TokenStream {
    topology::expand(input)
}

/// Map chart enum variants to a topology node enum **by name**.
///
/// ```ignore
/// #[derive(IntoNode)]
/// #[into_node(Node)]
/// enum Chart {
///     Off,
///     On { n: u8 },
/// }
/// // Chart::Off.node() == Node::Off
/// // Chart::On { .. }.node() == Node::On
/// ```
#[proc_macro_derive(IntoNode, attributes(into_node))]
pub fn derive_into_node(input: TokenStream) -> TokenStream {
    into_node::expand(input)
}

/// Fill [`newton_machine::Machine`] from an inherent `impl` that already
/// contains `init`, `update`, and `view`.
///
/// Associated types come from the attribute. `in_state` / `configuration`
/// walk ancestors via `IntoNode` + `Topology`. Unless `no_topology` is set,
/// `Topology` is forwarded from `node_id` (derive `Topology` on that enum).
///
/// ```ignore
/// #[machine(
///     model = u32,
///     msg = Msg,
///     cmd = Cmd<u8>,
///     view = bool,
///     node_id = Node,
/// )]
/// impl Chart {
///     fn init(_: ()) -> Boot<Self> { /* ... */ }
///     fn update(&mut self, model: &mut u32, hist: &mut (), msg: Msg) -> Cmd<u8> { /* ... */ }
///     fn view(&self, model: &u32) -> bool { /* ... */ }
/// }
/// ```
///
/// Defaults: `flags = ()`, `history = ()`. Extra methods in the impl stay
/// as inherent methods on the chart type.
#[proc_macro_attribute]
pub fn machine(attr: TokenStream, item: TokenStream) -> TokenStream {
    machine::expand(attr, item)
}

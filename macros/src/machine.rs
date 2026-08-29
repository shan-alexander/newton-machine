use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{parse_macro_input, Error, FnArg, Ident, ImplItem, ItemImpl, Result, Token, Type};

struct Args {
    flags: Type,
    model: Option<Type>,
    msg: Option<Type>,
    cmd: Option<Type>,
    view: Option<Type>,
    history: Type,
    node_id: Option<Type>,
    no_topology: bool,
}

impl Default for Args {
    fn default() -> Self {
        Self {
            flags: syn::parse_quote!(()),
            model: None,
            msg: None,
            cmd: None,
            view: None,
            history: syn::parse_quote!(()),
            node_id: None,
            no_topology: false,
        }
    }
}

struct ArgAssign {
    key: Ident,
    value: ArgValue,
}

enum ArgValue {
    Type(Box<Type>),
    Flag,
}

impl Parse for ArgAssign {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let key: Ident = input.parse()?;
        if input.peek(Token![=]) {
            let _: Token![=] = input.parse()?;
            let value = input.parse()?;
            Ok(Self {
                key,
                value: ArgValue::Type(Box::new(value)),
            })
        } else {
            Ok(Self {
                key,
                value: ArgValue::Flag,
            })
        }
    }
}

impl Parse for Args {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let mut args = Args::default();
        if input.is_empty() {
            return Err(input.error(
                "#[machine] needs associated types: model, msg, cmd, view, node_id (flags/history default to ())",
            ));
        }
        let list = Punctuated::<ArgAssign, Token![,]>::parse_terminated(input)?;
        for a in list {
            match a.key.to_string().as_str() {
                "flags" => args.flags = ty(a)?,
                "model" => args.model = Some(ty(a)?),
                "msg" => args.msg = Some(ty(a)?),
                "cmd" => args.cmd = Some(ty(a)?),
                "view" => args.view = Some(ty(a)?),
                "history" => args.history = ty(a)?,
                "node_id" => args.node_id = Some(ty(a)?),
                "no_topology" => args.no_topology = true,
                other => {
                    return Err(Error::new_spanned(
                        a.key,
                        format!("unknown #[machine] key `{other}`"),
                    ));
                }
            }
        }
        Ok(args)
    }
}

fn ty(a: ArgAssign) -> Result<Type> {
    match a.value {
        ArgValue::Type(t) => Ok(*t),
        ArgValue::Flag => Err(Error::new_spanned(a.key, "expected `key = Type`")),
    }
}

pub fn expand(attr: TokenStream, item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(attr as Args);
    let impl_block = parse_macro_input!(item as ItemImpl);
    match expand_inner(args, impl_block) {
        Ok(ts) => ts.into(),
        Err(e) => e.to_compile_error().into(),
    }
}

fn expand_inner(args: Args, mut impl_block: ItemImpl) -> Result<TokenStream2> {
    if impl_block.trait_.is_some() {
        return Err(Error::new_spanned(
            &impl_block,
            "#[machine] applies to `impl Chart { ... }`, not `impl Machine for Chart`",
        ));
    }

    let model = args.model.ok_or_else(|| {
        Error::new_spanned(&impl_block.self_ty, "#[machine] requires `model = Type`")
    })?;
    let msg = args.msg.ok_or_else(|| {
        Error::new_spanned(&impl_block.self_ty, "#[machine] requires `msg = Type`")
    })?;
    let cmd = args.cmd.ok_or_else(|| {
        Error::new_spanned(&impl_block.self_ty, "#[machine] requires `cmd = Type`")
    })?;
    let view = args.view.ok_or_else(|| {
        Error::new_spanned(&impl_block.self_ty, "#[machine] requires `view = Type`")
    })?;
    let node_id = args.node_id.ok_or_else(|| {
        Error::new_spanned(
            &impl_block.self_ty,
            "#[machine] requires `node_id = Type` (the topology id enum, or `()`)",
        )
    })?;

    let mut init = None;
    let mut update = None;
    let mut view_fn = None;
    let mut extras = Vec::new();

    for item in impl_block.items.drain(..) {
        match item {
            ImplItem::Fn(f) if f.sig.ident == "init" => init = Some(f),
            ImplItem::Fn(f) if f.sig.ident == "update" => update = Some(f),
            ImplItem::Fn(f) if f.sig.ident == "view" => view_fn = Some(f),
            other => extras.push(other),
        }
    }

    let init = init.ok_or_else(|| {
        Error::new_spanned(
            &impl_block.self_ty,
            "#[machine] impl must contain `fn init(...) -> Boot<Self>`",
        )
    })?;
    let update = update.ok_or_else(|| {
        Error::new_spanned(
            &impl_block.self_ty,
            "#[machine] impl must contain `fn update(&mut self, model, history, msg) -> Cmd`",
        )
    })?;
    let view_fn = view_fn.ok_or_else(|| {
        Error::new_spanned(
            &impl_block.self_ty,
            "#[machine] impl must contain `fn view(&self, model) -> View`",
        )
    })?;

    check_receiver(&update, "update")?;
    check_receiver(&view_fn, "view")?;
    if has_self_receiver(&init) {
        return Err(Error::new_spanned(
            &init.sig,
            "`init` is associated (`fn init(flags) -> Boot<Self>`), not a method",
        ));
    }

    let self_ty = &impl_block.self_ty;
    let (impl_g, _, where_c) = impl_block.generics.split_for_impl();
    let flags = &args.flags;
    let history = &args.history;

    let extras_impl = if extras.is_empty() {
        quote! {}
    } else {
        quote! {
            impl #impl_g #self_ty #where_c {
                #(#extras)*
            }
        }
    };

    let topology_impl = if args.no_topology {
        quote! {}
    } else {
        quote! {
            impl #impl_g ::newton_machine::Topology for #self_ty #where_c {
                type Node = #node_id;

                fn parent(node: Self::Node) -> ::core::option::Option<Self::Node> {
                    <#node_id as ::newton_machine::Topology>::parent(node)
                }
            }
        }
    };

    Ok(quote! {
        #extras_impl

        #topology_impl

        impl #impl_g ::newton_machine::Machine for #self_ty #where_c {
            type Flags = #flags;
            type Model = #model;
            type Msg = #msg;
            type Cmd = #cmd;
            type View = #view;
            type History = #history;
            type NodeId = #node_id;

            #init
            #update
            #view_fn

            fn in_state(&self, id: Self::NodeId) -> bool {
                let mut n = <Self as ::newton_machine::IntoNode>::node(self);
                loop {
                    if n == id {
                        return true;
                    }
                    match <Self as ::newton_machine::Topology>::parent(n) {
                        ::core::option::Option::Some(p) => n = p,
                        ::core::option::Option::None => return false,
                    }
                }
            }

            fn configuration(&self, out: &mut [Self::NodeId]) -> usize {
                let mut n = <Self as ::newton_machine::IntoNode>::node(self);
                let mut i = 0usize;
                loop {
                    if i >= out.len() {
                        return i;
                    }
                    out[i] = n;
                    i += 1;
                    match <Self as ::newton_machine::Topology>::parent(n) {
                        ::core::option::Option::Some(p) => n = p,
                        ::core::option::Option::None => return i,
                    }
                }
            }
        }
    })
}

fn has_self_receiver(f: &syn::ImplItemFn) -> bool {
    matches!(f.sig.inputs.iter().next(), Some(FnArg::Receiver(_)))
}

fn check_receiver(f: &syn::ImplItemFn, name: &str) -> Result<()> {
    if !has_self_receiver(f) {
        return Err(Error::new_spanned(
            &f.sig,
            format!("`{name}` must be a method (`&mut self` / `&self`)"),
        ));
    }
    Ok(())
}

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Error, Fields, Ident, Result};

pub fn expand(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    match expand_inner(&input) {
        Ok(ts) => ts.into(),
        Err(e) => e.to_compile_error().into(),
    }
}

struct VariantMeta {
    ident: Ident,
    parent: Option<Ident>,
    is_root: bool,
}

fn expand_inner(input: &DeriveInput) -> Result<TokenStream2> {
    let name = &input.ident;
    let data = match &input.data {
        Data::Enum(e) => e,
        _ => {
            return Err(Error::new_spanned(
                name,
                "#[derive(Topology)] only supports enums (the node-id tree)",
            ));
        }
    };

    let mut metas = Vec::new();
    for v in &data.variants {
        if !matches!(v.fields, Fields::Unit) {
            return Err(Error::new_spanned(
                &v.ident,
                "Topology node ids must be unit variants (the id is the variant, not payload)",
            ));
        }
        let mut parent = None;
        let mut is_root = false;
        for attr in &v.attrs {
            if !attr.path().is_ident("topology") {
                continue;
            }
            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("root") {
                    is_root = true;
                    Ok(())
                } else if meta.path.is_ident("parent") {
                    let value = meta.value()?;
                    parent = Some(value.parse::<Ident>()?);
                    Ok(())
                } else {
                    Err(meta.error("expected #[topology(root)] or #[topology(parent = Variant)]"))
                }
            })?;
        }
        if is_root && parent.is_some() {
            return Err(Error::new_spanned(
                &v.ident,
                "#[topology(root)] cannot also set parent",
            ));
        }
        if !is_root && parent.is_none() {
            return Err(Error::new_spanned(
                &v.ident,
                "missing #[topology(root)] or #[topology(parent = …)]",
            ));
        }
        metas.push(VariantMeta {
            ident: v.ident.clone(),
            parent,
            is_root,
        });
    }

    let roots: Vec<_> = metas.iter().filter(|m| m.is_root).collect();
    match roots.len() {
        0 => {
            return Err(Error::new_spanned(
                name,
                "#[derive(Topology)] needs exactly one #[topology(root)] variant",
            ));
        }
        1 => {}
        _ => {
            return Err(Error::new_spanned(
                name,
                "#[derive(Topology)] allows only one #[topology(root)] variant",
            ));
        }
    }

    let idents: Vec<_> = metas.iter().map(|m| m.ident.to_string()).collect();
    for m in &metas {
        if let Some(p) = &m.parent {
            if !idents.iter().any(|s| s == &p.to_string()) {
                return Err(Error::new_spanned(
                    p,
                    format!("parent `{p}` is not a variant of `{name}`"),
                ));
            }
        }
    }

    if let Some(msg) = find_cycle(&metas) {
        return Err(Error::new_spanned(name, msg));
    }

    let arms = metas.iter().map(|m| {
        let ident = &m.ident;
        match &m.parent {
            None => quote! { #name::#ident => None },
            Some(p) => quote! { #name::#ident => Some(#name::#p) },
        }
    });

    let (impl_g, ty_g, where_c) = input.generics.split_for_impl();

    Ok(quote! {
        impl #impl_g ::newton_machine::Topology for #name #ty_g #where_c {
            type Node = Self;

            fn parent(node: Self) -> ::core::option::Option<Self> {
                match node {
                    #(#arms,)*
                }
            }
        }
    })
}

fn find_cycle(metas: &[VariantMeta]) -> Option<String> {
    use std::collections::HashMap;
    let parent_of: HashMap<String, String> = metas
        .iter()
        .filter_map(|m| {
            m.parent
                .as_ref()
                .map(|p| (m.ident.to_string(), p.to_string()))
        })
        .collect();

    for start in parent_of.keys() {
        let mut seen = Vec::new();
        let mut cur = start.clone();
        loop {
            if seen.contains(&cur) {
                seen.push(cur);
                return Some(format!(
                    "topology cycle (a node cannot be its own ancestor): {}",
                    seen.join(" → ")
                ));
            }
            seen.push(cur.clone());
            match parent_of.get(&cur) {
                Some(p) => cur = p.clone(),
                None => break,
            }
        }
    }
    None
}

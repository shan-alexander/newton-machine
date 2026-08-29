use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Error, Fields, PathArguments, Result, Type};

pub fn expand(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    match expand_inner(&input) {
        Ok(ts) => ts.into(),
        Err(e) => e.to_compile_error().into(),
    }
}

fn expand_inner(input: &DeriveInput) -> Result<TokenStream2> {
    let name = &input.ident;
    let node_ty = parse_into_node_ty(input)?;
    let data = match &input.data {
        Data::Enum(e) => e,
        _ => {
            return Err(Error::new_spanned(
                name,
                "#[derive(IntoNode)] only supports enums (XOR configuration)",
            ));
        }
    };

    let node_ident = type_to_ident(&node_ty).ok_or_else(|| {
        Error::new_spanned(
            &node_ty,
            "#[into_node(…)] expects a simple enum path, e.g. Node",
        )
    })?;

    let arms = data.variants.iter().map(|v| {
        let v_ident = &v.ident;
        let pat = match &v.fields {
            Fields::Unit => quote! { Self::#v_ident },
            Fields::Unnamed(_) => quote! { Self::#v_ident (..) },
            Fields::Named(_) => quote! { Self::#v_ident { .. } },
        };
        quote! { #pat => #node_ident::#v_ident }
    });

    let (impl_g, ty_g, where_c) = input.generics.split_for_impl();

    Ok(quote! {
        impl #impl_g ::newton_machine::IntoNode for #name #ty_g #where_c {
            type Node = #node_ty;

            fn node(&self) -> Self::Node {
                match self {
                    #(#arms,)*
                }
            }
        }
    })
}

fn parse_into_node_ty(input: &DeriveInput) -> Result<Type> {
    let mut found = None;
    for attr in &input.attrs {
        if !attr.path().is_ident("into_node") {
            continue;
        }
        found = Some(attr.parse_args::<Type>()?);
    }
    found.ok_or_else(|| {
        Error::new_spanned(
            &input.ident,
            "#[derive(IntoNode)] requires #[into_node(NodeEnum)]",
        )
    })
}

fn type_to_ident(ty: &Type) -> Option<syn::Ident> {
    match ty {
        Type::Path(p) => p.path.get_ident().cloned().or_else(|| {
            p.path.segments.last().and_then(|s| {
                if matches!(s.arguments, PathArguments::None) {
                    Some(s.ident.clone())
                } else {
                    None
                }
            })
        }),
        _ => None,
    }
}

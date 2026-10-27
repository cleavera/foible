use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{parse_macro_input, Attribute, Data, DeriveInput, Error, Fields, Generics, Type};

#[proc_macro_derive(Trace, attributes(from))]
pub fn derive_trace(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    expand(input)
        .unwrap_or_else(Error::into_compile_error)
        .into()
}

fn from_attr(attrs: &[Attribute]) -> Option<&Attribute> {
    attrs.iter().find(|attr| attr.path().is_ident("from"))
}

fn is_bare_type_param(ty: &Type, generics: &Generics) -> bool {
    match ty {
        Type::Path(path) if path.qself.is_none() => generics
            .type_params()
            .any(|param| path.path.is_ident(&param.ident)),
        _ => false,
    }
}

fn expand(input: DeriveInput) -> syn::Result<TokenStream2> {
    let name = &input.ident;

    let variants = match input.data {
        Data::Enum(data) => data.variants,
        Data::Struct(data) => {
            return Err(Error::new_spanned(
                data.struct_token,
                "Trace can only be derived on enums",
            ))
        }
        Data::Union(data) => {
            return Err(Error::new_spanned(
                data.union_token,
                "Trace can only be derived on enums",
            ))
        }
    };

    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    // `Trace: Debug`, so every type parameter must be `Debug` as well.
    let mut trace_generics = input.generics.clone();
    let type_params: Vec<_> = trace_generics
        .type_params()
        .map(|param| param.ident.clone())
        .collect();
    let trace_where = trace_generics.make_where_clause();
    for param in type_params {
        trace_where
            .predicates
            .push(syn::parse_quote!(#param: ::core::fmt::Debug));
    }
    let (trace_impl_generics, _, trace_where_clause) = trace_generics.split_for_impl();

    let mut match_arms = Vec::new();
    let mut from_impls = Vec::new();

    for variant in variants.iter() {
        let variant_ident = &variant.ident;

        if let Fields::Unnamed(fields) = &variant.fields {
            if fields.unnamed.len() == 1 {
                let inner_type = &fields.unnamed[0].ty;

                let label = format!("{}::{}", name, variant_ident);
                match_arms.push(quote! {
                    Self::#variant_ident(inner) => {
                        f.write_str(#label)?;
                        f.write_str("\n  ")?;
                        (&&&::foible::__private::Wrap(inner)).write_chain(f)
                    }
                });

                if let Some(attr) = from_attr(&variant.attrs) {
                    // Would overlap with core's `impl<T> From<T> for T`.
                    if is_bare_type_param(inner_type, &input.generics) {
                        return Err(Error::new_spanned(
                            attr,
                            "#[from] cannot be used on a field whose type is a bare generic parameter",
                        ));
                    }
                    from_impls.push(quote! {
                        impl #impl_generics ::core::convert::From<#inner_type> for #name #ty_generics #where_clause {
                            fn from(err: #inner_type) -> Self {
                                Self::#variant_ident(err)
                            }
                        }
                    });
                }
                continue;
            }
        }

        if let Some(attr) = from_attr(&variant.attrs) {
            return Err(Error::new_spanned(
                attr,
                "#[from] can only be used on a variant with exactly one unnamed field",
            ));
        }

        if let Fields::Unit = &variant.fields {
            let label = format!("{}::{}", name, variant_ident);
            match_arms.push(quote! {
                Self::#variant_ident => f.write_str(#label),
            });
        } else {
            let label = format!("{}::{}", name, variant_ident);
            match_arms.push(quote! {
                Self::#variant_ident { .. } => f.write_str(#label),
            });
        }
    }

    let expanded = quote! {
        impl #trace_impl_generics ::foible::Trace for #name #ty_generics #trace_where_clause {
            fn write_trace(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                use ::foible::__private::{ViaTrace as _, ViaDebug as _, ViaDisplay as _};
                match self {
                    #(#match_arms)*
                }
            }
        }

        #(#from_impls)*
    };

    Ok(expanded)
}

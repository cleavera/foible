use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields};

#[proc_macro_derive(Trace, attributes(from))]
pub fn derive_trace(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let variants = match input.data {
        Data::Enum(data) => data.variants,
        _ => panic!("Trace can only be derived on Enums"),
    };

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

                let has_from = variant
                    .attrs
                    .iter()
                    .any(|attr| attr.path().is_ident("from"));
                if has_from {
                    from_impls.push(quote! {
                        impl ::core::convert::From<#inner_type> for #name {
                            fn from(err: #inner_type) -> Self {
                                Self::#variant_ident(err)
                            }
                        }
                    });
                }
                continue;
            }
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
        impl ::foible::Trace for #name {
            fn write_trace(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                use ::foible::__private::{ViaTrace as _, ViaDebug as _, ViaDisplay as _};
                match self {
                    #(#match_arms)*
                }
            }
        }

        #(#from_impls)*
    };

    TokenStream::from(expanded)
}

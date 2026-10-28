use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote, quote_spanned};
use syn::{
    parse_macro_input, spanned::Spanned, Attribute, Data, DeriveInput, Error, Fields, Generics,
    Type,
};

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
        let label = format!("{}::{}", name, variant_ident);

        let fields: Vec<_> = variant.fields.iter().collect();
        let bindings: Vec<_> = (0..fields.len())
            .map(|i| format_ident!("__foible_{}", i))
            .collect();

        if let Some(attr) = from_attr(&variant.attrs) {
            let [field] = fields.as_slice() else {
                return Err(Error::new_spanned(
                    attr,
                    "#[from] can only be used on a variant with exactly one field",
                ));
            };
            let inner_type = &field.ty;
            // Would overlap with core's `impl<T> From<T> for T`.
            if is_bare_type_param(inner_type, &input.generics) {
                return Err(Error::new_spanned(
                    attr,
                    "#[from] cannot be used on a field whose type is a bare generic parameter",
                ));
            }
            let construct = match &field.ident {
                Some(field_name) => quote!(Self::#variant_ident { #field_name: err }),
                None => quote!(Self::#variant_ident(err)),
            };
            from_impls.push(quote! {
                impl #impl_generics ::core::convert::From<#inner_type> for #name #ty_generics #where_clause {
                    fn from(err: #inner_type) -> Self {
                        #construct
                    }
                }
            });
        }

        let pattern = match &variant.fields {
            Fields::Unit => quote!(Self::#variant_ident),
            Fields::Unnamed(_) => quote!(Self::#variant_ident(#(#bindings),*)),
            Fields::Named(_) => {
                let names = fields.iter().map(|field| &field.ident);
                quote!(Self::#variant_ident { #(#names: #bindings),* })
            }
        };

        // Autoref specialization: each field is wrapped and called through
        // `&&&`. Method lookup tries the receiver as written first, then
        // derefs one level at a time, and takes the first trait impl whose
        // bounds hold. The three tiers in `foible::__private` are implemented
        // for `&&Wrap` (Trace), `&Wrap` (Display) and `Wrap` (Debug), so
        // `&&&` makes them win in that order. Changing the number of `&`s or
        // the tiers' `Self` types shifts the priorities.
        let write_data = fields.iter().zip(&bindings).map(|(field, binding)| {
            let field_name = match &field.ident {
                Some(ident) => {
                    let ident = ident.to_string();
                    quote!(::core::option::Option::Some(#ident))
                }
                None => quote!(::core::option::Option::None),
            };
            quote! {
                (&&&::foible::__private::Wrap(#binding)).write_data(f, &mut sep, #field_name)?;
            }
        });

        // Spanned on the field so a second `Trace` field is reported there.
        let write_child = fields.iter().zip(&bindings).map(|(field, binding)| {
            quote_spanned! {field.span()=>
                let slot = (&&&::foible::__private::Wrap(#binding)).write_child(f, slot)?;
            }
        });

        match_arms.push(quote! {
            #pattern => {
                f.write_str(#label)?;
                let mut sep = ::foible::__private::DataSep::default();
                #(#write_data)*
                sep.finish(f)?;
                let slot = ::foible::__private::CauseSlot;
                #(#write_child)*
                let _ = slot;
                ::core::result::Result::Ok(())
            }
        });
    }

    let expanded = quote! {
        impl #trace_impl_generics ::foible::Trace for #name #ty_generics #trace_where_clause {
            #[allow(clippy::needless_borrow, clippy::borrow_deref_ref)]
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

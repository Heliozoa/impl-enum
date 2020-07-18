//! Macro for generating methods on an enum that call the same method on each variant.

use proc_macro::TokenStream;
use quote::ToTokens;
use syn::{
    parse::{Error, Parse, ParseStream},
    spanned::Spanned,
    Fields, FnArg, ItemEnum, Pat, TraitItemMethod,
};

#[proc_macro_attribute]
pub fn with_methods(arg: TokenStream, input: TokenStream) -> TokenStream {
    let input_methods = syn::parse_macro_input!(arg as TraitItemMethods).0;
    let input_enum = syn::parse_macro_input!(input as ItemEnum);

    let mut methods = vec![];
    let mut errors = vec![];
    for method in &input_methods {
        match make_method(method, &input_enum) {
            Ok(method) => methods.push(method),
            Err(error) => {
                let span = error.span;
                let message = error.message;
                errors.push(quote::quote_spanned! {
                        span.span() => compile_error!(#message);
                })
            }
        }
    }

    let enum_ident = &input_enum.ident;
    let enum_impl = quote::quote! {
        impl #enum_ident {
            #(#methods)*
        }
    };

    TokenStream::from(quote::quote! {
        #input_enum
        #enum_impl
        #(#errors)*
    })
}

struct TraitItemMethods(Vec<TraitItemMethod>);

impl Parse for TraitItemMethods {
    // loop over the input and try to parse trait methods
    fn parse(input: ParseStream) -> Result<Self, Error> {
        let mut methods = vec![];
        while !input.is_empty() {
            methods.push(input.parse()?);
        }

        Ok(TraitItemMethods(methods))
    }
}

fn make_method(
    input_method: &TraitItemMethod,
    input_enum: &ItemEnum,
) -> Result<impl ToTokens, MacroError> {
    let method_ident = &input_method.sig.ident;
    let method_arg_idents: Vec<_> = input_method
        .sig
        .inputs
        .iter()
        .filter_map(|i| match i {
            FnArg::Typed(t) => match &*t.pat {
                Pat::Ident(i) => Some(&i.ident),
                _ => None,
            },
            FnArg::Receiver(_) => None,
        })
        .collect();

    // make match arm for every variant
    let mut match_arms = vec![];
    for variant in &input_enum.variants {
        let variant_ident = &variant.ident;
        match &variant.fields {
            Fields::Named(fields) => {
                let first_field = fields
                    .named
                    .first()
                    .ok_or_else(|| MacroError {
                        span: Box::new(fields.clone()),
                        message: "variants must have at least one field",
                    })?
                    .ident
                    .as_ref()
                    .unwrap();
                match_arms.push(quote::quote! {
                    Self::#variant_ident { #first_field, .. } => #first_field.#method_ident ( #(#method_arg_idents,)* )
                });
            }
            Fields::Unnamed(fields) => {
                let _first_field = fields.unnamed.first().as_ref().ok_or_else(|| MacroError {
                    span: Box::new(fields.clone()),
                    message: "variants must have at least one field",
                })?;
                match_arms.push(quote::quote! {
                    Self::#variant_ident ( f_1, .. ) => f_1.#method_ident ( #(#method_arg_idents,)* )
                });
            }
            Fields::Unit => {
                return Err(MacroError {
                    span: Box::new(variant.clone()),
                    message: "variants must have at least one field",
                })
            }
        };
    }

    let method_signature = &input_method.sig;
    Ok(quote::quote! {
        #method_signature {
            match self {
                #(#match_arms),*
            }
        }
    })
}

struct MacroError {
    span: Box<dyn Spanned>,
    message: &'static str,
}

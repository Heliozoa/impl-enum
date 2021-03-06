//! Macro for generating methods on an enum that match on the enum and call the same method on each variant.
//!
//! ## Example
//! ```rust
//! // The variant of the writer is dynamically selected with an environment variable.
//! // Using the macro, we get the convenience of a trait object with the performance of an enum.
//!
//! use std::env;
//! use std::fs::File;
//! use std::io::Cursor;
//! use std::io::Write;
//!
//! #[impl_enum::with_methods {
//!     fn write_all(&mut self, buf: &[u8]) -> Result<(), std::io::Error> {}
//!     pub fn write(&mut self, buf: &[u8]) -> Result<usize, std::io::Error> {}
//! }]
//! enum Writer {
//!     Cursor(Cursor<Vec<u8>>),
//!     File(File),
//! }
//!
//! fn get_writer() -> Writer {
//!     if let Ok(path) = env::var("WRITER_FILE") {
//!         Writer::File(File::create(path).unwrap())
//!     } else {
//!         Writer::Cursor(Cursor::new(vec![]))
//!     }
//! }
//!
//! fn main() {
//!     let mut writer = get_writer();
//!     writer.write_all(b"hello!").unwrap();
//! }
//! ```
//!
//! The macro generates an impl block for the Writer enum equivalent to
//!
//! ```rust
//! # use std::fs::File;
//! # use std::io::Cursor;
//! # use std::io::Write;
//! #
//! # enum Writer {
//! #     Cursor(Cursor<Vec<u8>>),
//! #     File(File),
//! # }
//! #
//! impl Writer {
//!     fn write_all(&mut self, buf: &[u8]) -> Result<(), std::io::Error> {
//!         match self {
//!             Self::Cursor(cursor) => cursor.write_all(buf),
//!             Self::File(file) => file.write_all(buf),
//!         }
//!     }
//!
//!     pub fn write(&mut self, buf: &[u8]) -> Result<usize, std::io::Error> {
//!         match self {
//!             Self::Cursor(cursor) => cursor.write(buf),
//!             Self::File(file) => file.write(buf),
//!         }
//!     }
//! }
//! ```
//! This would be simple enough to write manually in this case, but with many variants and methods, maintaining such an impl can become tedious. The macro is intended to make such an enum easier to work with.
//!
//! Variants with named fields and multiple fields are also supported, the method is always called on the first field and the rest are ignored. Enums with variants with no fields are currently not supported.

use proc_macro::TokenStream;
use quote::ToTokens;
use syn::{
    parse::{Error, Parse, ParseStream},
    spanned::Spanned,
    Fields, FnArg, ItemEnum, ItemFn, Pat,
};

/// Generates an impl block for an enum containing the given methods, where the method is a simple match over all the variants, calling the same method on the matched variant's first field.
#[proc_macro_attribute]
pub fn with_methods(arg: TokenStream, input: TokenStream) -> TokenStream {
    let mut input_methods = syn::parse_macro_input!(arg as Methods).0;
    let input_enum = syn::parse_macro_input!(input as ItemEnum);

    // create methods and collect errors
    let mut errors = vec![];
    for method in input_methods.iter_mut() {
        if let Err(error) = add_block_to_fn(method, &input_enum) {
            let span = error.span;
            let message = error.message;
            errors.push(quote::quote_spanned! {
                    span.span() => compile_error!(#message);
            })
        }
    }

    // insert methods in an impl block
    let enum_ident = &input_enum.ident;
    let enum_impl = quote::quote! {
        impl #enum_ident {
            #(#input_methods)*
        }
    };

    // return the enum, impl and errors
    TokenStream::from(quote::quote! {
        #input_enum
        #enum_impl
        #(#errors)*
    })
}

struct Methods(Vec<ItemFn>);

impl Parse for Methods {
    // loop over the input and try to parse functions
    fn parse(input: ParseStream) -> Result<Self, Error> {
        let mut methods = vec![];
        while !input.is_empty() {
            methods.push(input.parse()?);
        }

        Ok(Methods(methods))
    }
}

// generates the method's block and sets the input_method's block to it
fn add_block_to_fn(input_method: &mut ItemFn, input_enum: &ItemEnum) -> Result<(), MacroError> {
    let method_ident = &input_method.sig.ident;
    let mut has_self_arg = false;
    let method_arg_idents: Vec<_> = input_method
        .sig
        .inputs
        .iter()
        .filter_map(|i| match i {
            FnArg::Typed(t) => match &*t.pat {
                Pat::Ident(i) => {
                    if i.ident == "self" {
                        has_self_arg = true;
                        None
                    } else {
                        Some(i.ident.to_token_stream())
                    }
                }
                _ => None,
            },
            FnArg::Receiver(_) => {
                has_self_arg = true;
                None
            }
        })
        .collect();

    let and = syn::Token![&](method_ident.span());
    let self_token = syn::Token![self](method_ident.span());
    if !has_self_arg {
        input_method.sig.inputs.insert(
            0,
            FnArg::Receiver(syn::Receiver {
                attrs: vec![],
                reference: Some((and, None)),
                mutability: None,
                self_token,
            }),
        );
    }

    // make match arm for every variant
    let mut match_arms = vec![];
    for variant in &input_enum.variants {
        let variant_ident = &variant.ident;
        match &variant.fields {
            // named fields, call on first field or error if no fields
            Fields::Named(fields) => {
                let mut first_field = fields
                    .named
                    .first()
                    .ok_or_else(|| MacroError {
                        span: Box::new(fields.clone()),
                        message: "variants must have at least one field".to_string(),
                    })?
                    .clone();
                let path = if let syn::Type::Path(path) = &mut first_field.ty {
                    path
                } else {
                    panic!();
                };
                for seg in &mut path.path.segments {
                    if let syn::PathArguments::AngleBracketed(gen) = &mut seg.arguments {
                        let colon2 = syn::Token![::](gen.span());
                        gen.colon2_token = Some(colon2);
                    }
                }
                let first_field_ident = first_field.ident.as_ref().unwrap();
                let first_field_type = &first_field.ty;

                let match_arm = if has_self_arg {
                    quote::quote! {
                        Self::#variant_ident { #first_field_ident, .. } => #first_field_type :: #method_ident (#first_field_ident, #(#method_arg_idents,)* )
                    }
                } else {
                    quote::quote! {
                        Self::#variant_ident { .. } => #first_field_type :: #method_ident (#(#method_arg_idents,)* )
                    }
                };
                match_arms.push(match_arm);
            }
            // unnamed fields, call on first field or error if no fields
            Fields::Unnamed(fields) => {
                let mut first_field = fields
                    .unnamed
                    .first()
                    .ok_or_else(|| MacroError {
                        span: Box::new(fields.clone()),
                        message: "variants must have at least one field".to_string(),
                    })?
                    .clone();
                let path = if let syn::Type::Path(path) = &mut first_field.ty {
                    path
                } else {
                    panic!();
                };
                for seg in &mut path.path.segments {
                    if let syn::PathArguments::AngleBracketed(gen) = &mut seg.arguments {
                        let colon2 = syn::Token![::](gen.span());
                        gen.colon2_token = Some(colon2);
                    }
                }

                let match_arm = if has_self_arg {
                    quote::quote! {
                        Self::#variant_ident ( f_1, .. ) => #first_field :: #method_ident (f_1, #(#method_arg_idents,)* )
                    }
                } else {
                    quote::quote! {
                        Self::#variant_ident ( .. ) => #first_field :: #method_ident ( #(#method_arg_idents,)* )
                    }
                };
                match_arms.push(match_arm);
            }
            // no fields, error
            Fields::Unit => {
                return Err(MacroError {
                    span: Box::new(variant.clone()),
                    message: "variants must have at least one field".to_string(),
                })
            }
        };
    }

    // generate new block for the function
    input_method.block = syn::parse(
        quote::quote!(
            {
                match self {
                    #(#match_arms),*
                }
            }
        )
        .into(),
    )
    .map_err(|e| MacroError {
        message: e.to_string(),
        span: Box::new(e.span()),
    })?;
    Ok(())
}

struct MacroError {
    span: Box<dyn Spanned>,
    message: String,
}

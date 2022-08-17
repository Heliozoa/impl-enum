//! Macro for generating methods on an enum that match on the enum
//! and call the same method on each variant.
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
//!     fn write_all(&mut self, buf: &[u8]) -> Result<(), std::io::Error>
//!     pub fn write(&mut self, buf: &[u8]) -> Result<usize, std::io::Error>
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
//! This would be simple enough to write manually in this case,
//! but with many variants and methods, maintaining such an impl can become tedious.
//! The macro is intended to make such an enum easier to work with.
//!
//! Variants with named fields and multiple fields are also supported,
//! the method is always called on the first field and the rest are ignored.
//! Enums with variants with no fields are currently not supported.

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::ToTokens;
use syn::{
    parse::{Error, Parse, ParseStream},
    spanned::Spanned,
    Fields, FnArg, ItemEnum, Receiver, Signature, Visibility,
};

pub fn with_methods_impl(arg: TokenStream, input: TokenStream) -> TokenStream {
    let input_methods = syn::parse_macro_input!(arg as Methods);
    let input_enum = syn::parse_macro_input!(input as ItemEnum);

    // construct the methods
    let mut methods = vec![];
    for (vis, sig) in input_methods.0 {
        match make_method(vis, sig, &input_enum) {
            Ok(method) => methods.push(method),
            Err(err) => return err.into_compile_error().into(),
        }
    }

    // construct the impl
    let enum_ident = &input_enum.ident;
    let (impl_generics, ty_generics, where_clause) = &input_enum.generics.split_for_impl();
    let enum_impl = quote::quote! {
        impl #impl_generics #enum_ident #ty_generics #where_clause {
            #(#methods)*
        }
    };

    // return the enum and impl
    TokenStream::from(quote::quote! {
        #input_enum
        #enum_impl
    })
}

struct Methods(Vec<(Visibility, Signature)>);

impl Parse for Methods {
    // loop over the input and try to parse functions
    fn parse(input: ParseStream) -> Result<Self, Error> {
        let mut methods = vec![];
        while !input.is_empty() {
            let vis: Visibility = input.parse()?;
            let sig: Signature = input.parse()?;
            methods.push((vis, sig));
        }

        Ok(Methods(methods))
    }
}

fn make_method(
    vis: Visibility,
    mut sig: Signature,
    input_enum: &ItemEnum,
) -> syn::Result<TokenStream2> {
    // turn receivers to __first for the call
    let method_call_args: Vec<_> = sig
        .inputs
        .iter()
        .map(|fa| match fa {
            FnArg::Typed(t) => t.pat.to_token_stream(),
            FnArg::Receiver(Receiver { self_token, .. }) => {
                quote::quote_spanned! { self_token.span() =>  __first }
            }
        })
        .collect();
    // add receiver if none for the signature
    if sig.receiver().is_none() {
        sig.inputs
            .insert(0, syn::parse_quote_spanned!(sig.inputs.span() => &self));
    }

    // make match arm for every variant
    let mut match_arms = vec![];
    for variant in &input_enum.variants {
        let first_field = match &variant.fields {
            Fields::Named(fields) => fields.named.first(),
            Fields::Unnamed(fields) => fields.unnamed.first(),
            Fields::Unit => {
                return Err(Error::new(
                    variant.span(),
                    "Unit variants are not supported",
                ))
            }
        }
        .ok_or_else(|| {
            Error::new(
                variant.fields.span(),
                "Enum variants must have at least one field",
            )
        })?;

        let variant_ident = &variant.ident;
        let first_field_type = &first_field.ty;
        let method_ident = &sig.ident;
        let match_arm = if let Some(first_field_ident) = &first_field.ident {
            quote::quote! {
                Self::#variant_ident { #first_field_ident: __first, .. }
                    => <#first_field_type> :: #method_ident (#(#method_call_args),* )
            }
        } else {
            quote::quote! {
                Self::#variant_ident ( __first, .. )
                    => <#first_field_type> :: #method_ident (#(#method_call_args),* )
            }
        };
        match_arms.push(match_arm);
    }

    // generate new block for the function
    let method = quote::quote! {
        #vis #sig {
            match self {
                #(#match_arms),*
            }
        }
    };
    Ok(method)
}

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::ToTokens;
use syn::{
    parse::{Error, Parse, ParseStream},
    spanned::Spanned,
    FnArg, ItemEnum, Receiver, Signature, Visibility,
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
    fn parse(input: ParseStream) -> Result<Self, Error> {
        // loop over the input and parse functions
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
    // rename receivers to __first for the call
    let method_call_args = sig
        .inputs
        .iter()
        .map(|fa| match fa {
            FnArg::Typed(t) => t.pat.to_token_stream(),
            FnArg::Receiver(Receiver { self_token, .. }) => {
                quote::quote_spanned! { self_token.span() =>  __first }
            }
        })
        .collect::<Vec<_>>();
    // add &self receiver if none for the signature
    if sig.receiver().is_none() {
        sig.inputs.insert(0, syn::parse_quote!(&self));
    }

    // make match arm for every variant
    let mut match_arms = vec![];
    for variant in &input_enum.variants {
        let first_field = super::first_field(variant)?;

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

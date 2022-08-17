use heck::ToSnakeCase;
use proc_macro::TokenStream;
use proc_macro2::{Ident, Span, TokenStream as TokenStream2};
use syn::{spanned::Spanned, Error, Fields, ItemEnum, Path};

pub fn as_dyn_impl(arg: TokenStream, input: TokenStream) -> TokenStream {
    let path = syn::parse_macro_input!(arg as Path);
    let input_enum = syn::parse_macro_input!(input as ItemEnum);

    // construct the methods
    let (as_arms, into_arms) = match make_arms(&input_enum) {
        Ok(match_arms) => match_arms,
        Err(err) => return err.into_compile_error().into(),
    };

    // construct the impl
    let target_ident = path
        .segments
        .last()
        .expect("empty path")
        .ident
        .to_string()
        .to_snake_case();
    let as_dyn = Ident::new(&format!("as_dyn_{target_ident}"), Span::call_site());
    let as_dyn_mut = Ident::new(&format!("as_dyn_{target_ident}_mut"), Span::call_site());
    let into_dyn = Ident::new(&format!("into_dyn_{target_ident}"), Span::call_site());

    let enum_ident = &input_enum.ident;
    let (impl_generics, ty_generics, where_clause) = &input_enum.generics.split_for_impl();
    let enum_impl = quote::quote! {
        impl #impl_generics #enum_ident #ty_generics #where_clause {
            fn #as_dyn (&self) -> &dyn #path {
                match self {
                    #(#as_arms),*
                }
            }
            fn #as_dyn_mut (&mut self) -> &mut dyn #path {
                match self {
                    #(#as_arms),*
                }
            }
            fn #into_dyn (self) -> Box<dyn #path> {
                match self {
                    #(#into_arms),*
                }
            }
        }
    };

    TokenStream::from(quote::quote! {
        #input_enum
        #enum_impl
    })
}

fn make_arms(input_enum: &ItemEnum) -> syn::Result<(Vec<TokenStream2>, Vec<TokenStream2>)> {
    let mut as_arms = vec![];
    let mut into_arms = vec![];

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
        if let Some(first_field_ident) = &first_field.ident {
            as_arms.push(quote::quote! {
                Self::#variant_ident { #first_field_ident: __first, .. } => __first as _
            });
            into_arms.push(quote::quote! {
                Self::#variant_ident { #first_field_ident: __first, .. } => Box::new(__first) as _
            });
        } else {
            as_arms.push(quote::quote! {
                Self::#variant_ident ( __first, .. ) => __first as _
            });
            into_arms.push(quote::quote! {
                Self::#variant_ident ( __first, .. ) => Box::new(__first) as _
            });
        };
    }

    Ok((as_arms, into_arms))
}

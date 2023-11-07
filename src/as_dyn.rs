use heck::ToSnakeCase;
use proc_macro::TokenStream;
use proc_macro2::{Ident, Span, TokenStream as TokenStream2};
use syn::{
    parse::{Parse, ParseStream},
    Error, ItemEnum, Path, Token,
};

pub fn as_dyn_impl(arg: TokenStream, input: TokenStream) -> TokenStream {
    let paths = syn::parse_macro_input!(arg as Paths);
    let input_enum = syn::parse_macro_input!(input as ItemEnum);

    let mut enum_impls = vec![];
    for path in paths.0 {
        match make_impl(&path, &input_enum) {
            Ok(enum_impl) => enum_impls.push(enum_impl),
            Err(err) => return err.into_compile_error().into(),
        };
    }

    TokenStream::from(quote::quote! {
        #input_enum
        #(#enum_impls)*
    })
}

struct Paths(Vec<Path>);

impl Parse for Paths {
    fn parse(input: ParseStream) -> Result<Self, Error> {
        // loop over the input and parse paths
        let paths = input
            .parse_terminated(Path::parse, Token![,])?
            .into_iter()
            .collect();

        Ok(Paths(paths))
    }
}

fn make_impl(path: &Path, input_enum: &ItemEnum) -> syn::Result<TokenStream2> {
    // construct the arms
    let (as_arms, into_arms) = make_arms(&input_enum)?;

    // construct the function names
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

    // construct the impl
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
    Ok(enum_impl)
}

fn make_arms(input_enum: &ItemEnum) -> syn::Result<(Vec<TokenStream2>, Vec<TokenStream2>)> {
    let mut as_arms = vec![];
    let mut into_arms = vec![];

    for variant in &input_enum.variants {
        let first_field = super::first_field(variant)?;

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
